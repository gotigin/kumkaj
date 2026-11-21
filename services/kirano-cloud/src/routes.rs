use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::db;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/me", get(me))
        .route("/v1/plans", get(plans))
        .route("/v1/me/subscription", get(subscription))
}

async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({
        "ok": true,
        "product": "kirano",
        "service": "kirano-cloud",
    }))
}

#[derive(Serialize)]
struct MeResponse {
    account_id: String,
    username: String,
    email: Option<String>,
    phone: Option<String>,
    display_name: Option<String>,
    product_code: &'static str,
    email_verified_hint: bool,
}

#[derive(Serialize)]
struct PlanResponse {
    plan_id: String,
    sort_order: i32,
    title: String,
    quota_gb: Option<i32>,
    bandwidth_band: String,
    price_inr_paise: Option<i64>,
    includes_live_cloud: bool,
}

#[derive(Serialize)]
struct SubscriptionResponse {
    plan_id: String,
    status: String,
    started_at: String,
    ends_at: Option<String>,
    title: String,
    quota_gb: Option<i32>,
    bandwidth_band: String,
    price_inr_paise: Option<i64>,
    includes_live_cloud: bool,
}

async fn me(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    match authed_account(&state, &headers).await {
        Ok(account) => Json(MeResponse {
            account_id: account.account_id.to_string(),
            username: account.username,
            email: account.email,
            phone: account.phone,
            display_name: account.display_name,
            product_code: state.product_code,
            email_verified_hint: account.email_verified_hint,
        })
        .into_response(),
        Err(resp) => resp,
    }
}

async fn plans(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(resp) = require_bearer(&state, &headers).await {
        return resp;
    }
    match db::list_plans(&state.pool).await {
        Ok(rows) => {
            let body: Vec<PlanResponse> = rows
                .into_iter()
                .map(|p| PlanResponse {
                    plan_id: p.plan_id,
                    sort_order: p.sort_order,
                    title: p.title,
                    quota_gb: p.quota_gb,
                    bandwidth_band: p.bandwidth_band,
                    price_inr_paise: p.price_inr_paise,
                    includes_live_cloud: p.includes_live_cloud,
                })
                .collect();
            Json(body).into_response()
        }
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn subscription(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let account = match authed_account(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    match db::subscription_for(&state.pool, account.account_id).await {
        Ok(Some(s)) => Json(SubscriptionResponse {
            plan_id: s.plan_id,
            status: s.status,
            started_at: s.started_at.to_rfc3339(),
            ends_at: s.ends_at.map(|t| t.to_rfc3339()),
            title: s.title,
            quota_gb: s.quota_gb,
            bandwidth_band: s.bandwidth_band,
            price_inr_paise: s.price_inr_paise,
            includes_live_cloud: s.includes_live_cloud,
        })
        .into_response(),
        Ok(None) => api_err(StatusCode::NOT_FOUND, "no subscription"),
        Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn require_bearer(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::auth::FirebaseClaims, Response> {
    let token = bearer_token(headers).ok_or_else(|| {
        api_err(
            StatusCode::UNAUTHORIZED,
            "missing Authorization: Bearer <firebase_id_token>",
        )
    })?;
    state.verifier.verify(&token).await.map_err(|e| {
        api_err(StatusCode::UNAUTHORIZED, e)
    })
}

async fn authed_account(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<db::Account, Response> {
    let claims = require_bearer(state, headers).await?;
    db::ensure_account(
        &state.pool,
        state.product_code,
        &state.verifier.issuer(),
        &claims,
    )
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ").or_else(|| raw.strip_prefix("bearer "))?;
    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn api_err(status: StatusCode, message: impl Into<String>) -> Response {
    let body = Json(serde_json::json!({ "error": message.into() }));
    (status, body).into_response()
}

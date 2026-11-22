use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::billing::{self, Provider};
use crate::db;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/me", get(me))
        .route("/v1/plans", get(plans))
        .route("/v1/me/subscription", get(subscription))
        .route("/v1/billing/config", get(billing_config))
        .route("/v1/billing/checkout", post(billing_checkout))
        .route("/v1/billing/sandbox/complete", post(billing_sandbox_complete))
        .route(
            "/v1/billing/webhooks/stripe",
            post(billing::handle_stripe_webhook),
        )
        .route(
            "/v1/billing/webhooks/paddle",
            post(billing::handle_paddle_webhook),
        )
}

async fn healthz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "ok": true,
        "product": "khaynjo",
        "service": "khaynjo-cloud",
        "billing_enabled": state.billing.as_ref().is_some_and(|b| b.enabled),
        "billing_mode": state
            .billing
            .as_ref()
            .map(|b| b.mode.as_str())
            .unwrap_or("off"),
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
    checkout_ready: bool,
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

#[derive(Deserialize)]
struct CheckoutBody {
    plan_id: String,
    #[serde(default)]
    provider: Option<String>,
}

#[derive(Deserialize)]
struct SandboxCompleteBody {
    checkout_id: String,
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
    let billing = state.billing.as_ref();
    match db::list_plans(&state.pool).await {
        Ok(rows) => {
            let body: Vec<PlanResponse> = rows
                .into_iter()
                .map(|p| {
                    let checkout_ready = billing.is_some_and(|b| {
                        b.enabled
                            && p.includes_live_cloud
                            && p.plan_id != "custom"
                            && (b.provider_enabled(Provider::Mock)
                                || (b.provider_enabled(Provider::Stripe)
                                    && p.stripe_price_id.is_some())
                                || (b.provider_enabled(Provider::Paddle)
                                    && p.paddle_price_id.is_some()))
                    });
                    PlanResponse {
                        plan_id: p.plan_id,
                        sort_order: p.sort_order,
                        title: p.title,
                        quota_gb: p.quota_gb,
                        bandwidth_band: p.bandwidth_band,
                        price_inr_paise: p.price_inr_paise,
                        includes_live_cloud: p.includes_live_cloud,
                        checkout_ready,
                    }
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

async fn billing_config(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(resp) = require_bearer(&state, &headers).await {
        return resp;
    }
    let Some(cfg) = state.billing.as_ref() else {
        return Json(serde_json::json!({
            "enabled": false,
            "mode": "off",
            "providers": [],
            "default_provider": null
        }))
        .into_response();
    };
    Json(billing::public_config(cfg)).into_response()
}

async fn billing_checkout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CheckoutBody>,
) -> Response {
    let account = match authed_account(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let Some(cfg) = state.billing.as_ref() else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "billing disabled");
    };
    let provider = match body.provider.as_deref() {
        Some(raw) => match Provider::parse(raw) {
            Some(p) => p,
            None => return api_err(StatusCode::BAD_REQUEST, "provider must be mock|stripe|paddle"),
        },
        None => cfg.default_provider,
    };
    match billing::start_checkout(&state.pool, cfg, &account, &body.plan_id, provider).await {
        Ok(out) => Json(out).into_response(),
        Err(e) => api_err(e.status(), e.message()),
    }
}

async fn billing_sandbox_complete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SandboxCompleteBody>,
) -> Response {
    let account = match authed_account(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let Some(cfg) = state.billing.as_ref() else {
        return api_err(StatusCode::SERVICE_UNAVAILABLE, "billing disabled");
    };
    let checkout_id = match Uuid::parse_str(&body.checkout_id) {
        Ok(id) => id,
        Err(_) => return api_err(StatusCode::BAD_REQUEST, "invalid checkout_id"),
    };
    match billing::complete_mock_checkout(&state.pool, cfg, &account, checkout_id).await {
        Ok(()) => {
            match db::subscription_for(&state.pool, account.account_id).await {
                Ok(Some(s)) => Json(serde_json::json!({
                    "ok": true,
                    "plan_id": s.plan_id,
                    "status": s.status,
                    "ends_at": s.ends_at.map(|t| t.to_rfc3339()),
                }))
                .into_response(),
                Ok(None) => api_err(StatusCode::INTERNAL_SERVER_ERROR, "subscription missing after complete"),
                Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        }
        Err(e) => api_err(e.status(), e.message()),
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
    state
        .verifier
        .verify(&token)
        .await
        .map_err(|e| api_err(StatusCode::UNAUTHORIZED, e))
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
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))?;
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

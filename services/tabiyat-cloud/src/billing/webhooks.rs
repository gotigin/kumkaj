use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::db;
use crate::mail;
use crate::AppState;

use super::config::Provider;
use super::{activate_subscription, BillingError};

async fn send_activation_mail(
    state: &AppState,
    account_id: Uuid,
    plan_id: &str,
) -> Result<(), String> {
    let account = db::account_by_id(&state.pool, account_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "account not found for mail".to_string())?;
    let email = account.email.unwrap_or_default();
    let plan = db::plan_by_id(&state.pool, plan_id)
        .await
        .map_err(|e| e.to_string())?;
    let title = plan
        .map(|p| p.title)
        .unwrap_or_else(|| plan_id.to_string());
    let ends = (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339();
    mail::send_plan_activated(&state.mail, &email, &title, &ends).await
}

pub async fn handle_paddle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(cfg) = state.billing.as_ref() else {
        return err(StatusCode::SERVICE_UNAVAILABLE, "billing disabled");
    };
    if !cfg.enabled {
        return err(StatusCode::SERVICE_UNAVAILABLE, "billing disabled");
    }
    let secret = match cfg.paddle_webhook_secret.as_deref() {
        Some(s) => s,
        None => return err(StatusCode::SERVICE_UNAVAILABLE, "paddle webhook not configured"),
    };
    let sig = match headers
        .get("Paddle-Signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s,
        None => return err(StatusCode::BAD_REQUEST, "missing Paddle-Signature"),
    };
    if let Err(e) = super::paddle::verify_signature(secret, sig, &body) {
        return err(StatusCode::BAD_REQUEST, e);
    }
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return err(StatusCode::BAD_REQUEST, e.to_string()),
    };
    let event_id = payload
        .get("event_id")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("notification_id").and_then(|v| v.as_str()))
        .unwrap_or("paddle_unknown");
    let event_type = payload
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if matches!(
        event_type,
        "transaction.completed" | "subscription.activated" | "subscription.created"
    ) {
        let data = payload.get("data").cloned().unwrap_or(Value::Null);
        let custom = data
            .get("custom_data")
            .cloned()
            .or_else(|| data.pointer("/transaction/custom_data").cloned())
            .unwrap_or(Value::Null);
        let account_id = custom
            .get("account_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        let plan_id = custom
            .get("plan_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let checkout_id = custom
            .get("checkout_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        let sub_id = data
            .get("subscription_id")
            .and_then(|v| v.as_str())
            .or_else(|| data.get("id").and_then(|v| v.as_str()))
            .map(str::to_string);
        if plan_id == "free" || plan_id == "custom" {
            return err(StatusCode::BAD_REQUEST, "refusing webhook activation for free/custom");
        }
        if let (Some(account_id), true) = (account_id, !plan_id.is_empty()) {
            if let Err(e) = activate_subscription(
                &state.pool,
                account_id,
                &plan_id,
                Provider::Paddle,
                sub_id.as_deref(),
                checkout_id,
                event_id,
                payload.clone(),
            )
            .await
            {
                return billing_err(e);
            }
            // Best-effort tabiyat confirmation (SES). Do not fail the webhook on mail errors.
            if let Err(e) = send_activation_mail(state.as_ref(), account_id, &plan_id).await {
                tracing::warn!(%account_id, error = %e, "plan confirmation email failed");
            }
        }
    }

    Json(serde_json::json!({ "received": true })).into_response()
}

fn err(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": message.into() }))).into_response()
}

fn billing_err(e: BillingError) -> Response {
    err(e.status(), e.message())
}

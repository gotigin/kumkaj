//! Moklanjo Cloud billing — sandbox first (Paddle + local mock). Stripe removed (India Tax incompatible).
//!
//! Product doctrine: free local forever; cloud subscription never hostages the till.
//! Ops: Paddle dashboard in Chrome profile **MEARAJ (Gotigin)**.

mod config;
mod mock;
mod paddle;
mod webhooks;

pub use config::{BillingConfig, BillingMode, Provider};
pub use webhooks::handle_paddle_webhook;

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::{self, Account};

pub struct CreatedCheckout {
    pub external_id: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct BillingPublicConfig {
    pub enabled: bool,
    pub mode: &'static str,
    pub providers: Vec<&'static str>,
    pub default_provider: &'static str,
    pub paddle_client_token: Option<String>,
    pub paddle_env: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub checkout_id: String,
    pub provider: &'static str,
    pub mode: &'static str,
    pub url: String,
    pub plan_id: String,
}

#[derive(Debug)]
pub enum BillingError {
    BadRequest(String),
    Unavailable(String),
    Upstream(String),
    Db(sqlx::Error),
}

impl BillingError {
    pub fn status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::BadRequest(m) | Self::Unavailable(m) | Self::Upstream(m) => m.clone(),
            Self::Db(e) => e.to_string(),
        }
    }
}

impl From<sqlx::Error> for BillingError {
    fn from(value: sqlx::Error) -> Self {
        Self::Db(value)
    }
}

pub fn public_config(cfg: &BillingConfig) -> BillingPublicConfig {
    BillingPublicConfig {
        enabled: cfg.enabled,
        mode: cfg.mode.as_str(),
        providers: cfg.enabled_providers(),
        default_provider: cfg.default_provider.as_str(),
        paddle_client_token: cfg.paddle_client_token.clone(),
        paddle_env: if cfg.paddle_ready() {
            Some(cfg.mode.paddle_env())
        } else {
            None
        },
    }
}

pub async fn start_checkout(
    pool: &PgPool,
    cfg: &BillingConfig,
    account: &Account,
    plan_id: &str,
    provider: Provider,
) -> Result<CheckoutResponse, BillingError> {
    if !cfg.enabled {
        return Err(BillingError::Unavailable(
            "billing is disabled on this API (set BILLING_ENABLED=1)".into(),
        ));
    }
    if plan_id == "free" || plan_id == "custom" {
        return Err(BillingError::BadRequest(format!(
            "plan `{plan_id}` is not sold via self-serve checkout"
        )));
    }
    if !cfg.provider_enabled(provider) {
        return Err(BillingError::BadRequest(format!(
            "provider `{}` is not configured for this API",
            provider.as_str()
        )));
    }

    let plan = db::plan_by_id(pool, plan_id)
        .await?
        .ok_or_else(|| BillingError::BadRequest(format!("unknown plan `{plan_id}`")))?;
    if !plan.includes_live_cloud {
        return Err(BillingError::BadRequest(
            "that plan does not include live cloud".into(),
        ));
    }

    let checkout_id = Uuid::new_v4();
    let created = match provider {
        Provider::Mock => {
            mock::create_checkout(cfg, checkout_id, account, &plan).await?
        }
        Provider::Paddle => {
            paddle::create_checkout(cfg, checkout_id, account, &plan).await?
        }
    };

    db::insert_checkout(
        pool,
        checkout_id,
        account.account_id,
        plan_id,
        provider.as_str(),
        &created.external_id,
        &created.url,
    )
    .await?;

    Ok(CheckoutResponse {
        checkout_id: checkout_id.to_string(),
        provider: provider.as_str(),
        mode: cfg.mode.as_str(),
        url: created.url,
        plan_id: plan_id.to_string(),
    })
}

/// Sandbox-only completion for mock checkouts (local E2E without Paddle UI).
pub async fn complete_mock_checkout(
    pool: &PgPool,
    cfg: &BillingConfig,
    account: &Account,
    checkout_id: Uuid,
) -> Result<(), BillingError> {
    if !cfg.enabled {
        return Err(BillingError::Unavailable("billing is disabled".into()));
    }
    if !matches!(cfg.mode, BillingMode::Sandbox) {
        return Err(BillingError::BadRequest(
            "sandbox complete is only available when BILLING_MODE=sandbox".into(),
        ));
    }
    if !cfg.allow_mock || !cfg.provider_enabled(Provider::Mock) {
        return Err(BillingError::Unavailable(
            "mock sandbox complete requires BILLING_ALLOW_MOCK=1 (local only)".into(),
        ));
    }
    let checkout = db::checkout_by_id(pool, checkout_id)
        .await?
        .ok_or_else(|| BillingError::BadRequest("unknown checkout_id".into()))?;
    if checkout.account_id != account.account_id {
        return Err(BillingError::BadRequest("checkout belongs to another account".into()));
    }
    if checkout.provider != Provider::Mock.as_str() {
        return Err(BillingError::BadRequest(
            "sandbox complete only works for mock checkouts; use the real gateway webhook otherwise"
                .into(),
        ));
    }
    if checkout.status == "completed" {
        return Ok(());
    }
    // Event id must be unique per checkout — a global id made every later
    // sandbox complete a no-op (plans stayed Free forever).
    let event_id = format!("mock.sandbox.complete.{checkout_id}");
    activate_subscription(
        pool,
        account.account_id,
        &checkout.plan_id,
        Provider::Mock,
        Some(&format!("mock_sub_{checkout_id}")),
        Some(checkout_id),
        &event_id,
        serde_json::json!({ "checkout_id": checkout_id }),
    )
    .await?;
    Ok(())
}

pub async fn activate_subscription(
    pool: &PgPool,
    account_id: Uuid,
    plan_id: &str,
    provider: Provider,
    external_subscription_id: Option<&str>,
    checkout_id: Option<Uuid>,
    event_id: &str,
    payload: serde_json::Value,
) -> Result<(), BillingError> {
    if plan_id == "free" || plan_id == "custom" {
        return Err(BillingError::BadRequest(
            "refusing to activate free/custom via billing event".into(),
        ));
    }
    if event_id.is_empty() || event_id.ends_with("_unknown") {
        return Err(BillingError::BadRequest("missing billing event id".into()));
    }
    let inserted = db::record_billing_event(pool, provider.as_str(), event_id, &payload).await?;
    if !inserted {
        // Idempotent replay only after a prior successful process.
        if db::billing_event_processed(pool, provider.as_str(), event_id).await? {
            return Ok(());
        }
        return Err(BillingError::Upstream(
            "billing event seen but not processed yet — retry".into(),
        ));
    }

    let now = Utc::now();
    let ends_at = now + Duration::days(365);
    db::upsert_paid_subscription(
        pool,
        account_id,
        plan_id,
        provider.as_str(),
        external_subscription_id,
        checkout_id,
        now,
        ends_at,
    )
    .await?;
    if let Some(cid) = checkout_id {
        db::mark_checkout_completed(pool, cid).await?;
    }
    db::mark_billing_event_processed(pool, provider.as_str(), event_id).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paid_year_is_365_days_from_activation() {
        let start = Utc::now();
        let end = start + Duration::days(365);
        assert!(end > start);
        assert_eq!((end - start).num_days(), 365);
    }

    #[test]
    fn mock_requires_explicit_allow() {
        let cfg = BillingConfig::from_env_map([
            ("BILLING_ENABLED", "1"),
            ("BILLING_MODE", "sandbox"),
        ]);
        assert!(cfg.enabled);
        assert!(!cfg.provider_enabled(Provider::Mock));
        let cfg = BillingConfig::from_env_map([
            ("BILLING_ENABLED", "1"),
            ("BILLING_MODE", "sandbox"),
            ("BILLING_ALLOW_MOCK", "1"),
        ]);
        assert!(cfg.provider_enabled(Provider::Mock));
        assert_eq!(cfg.default_provider, Provider::Mock);
    }

    #[test]
    fn paddle_enables_when_keys_present() {
        let cfg = BillingConfig::from_env_map([
            ("BILLING_ENABLED", "1"),
            ("BILLING_MODE", "sandbox"),
            ("PADDLE_API_KEY", "pdl_sdbx_x"),
            ("PADDLE_WEBHOOK_SECRET", "pdl_ntfsec_x"),
            ("PADDLE_CLIENT_TOKEN", "test_x"),
            ("BILLING_DEFAULT_PROVIDER", "paddle"),
        ]);
        assert_eq!(cfg.default_provider, Provider::Paddle);
        assert!(cfg.provider_enabled(Provider::Paddle));
        assert!(!cfg.provider_enabled(Provider::Mock));
        assert!(Provider::parse("stripe").is_none());
    }
}

//! Tabiyat Cloud API client — account, plans, sandbox billing checkout.
//!
//! Base URL defaults to the current Fargate task; override with
//! `TABIYAT_CLOUD_API_BASE` (default `https://api.tabiyat.shop`).
//! Paid upgrades: same Stripe/Paddle/mock flow as https://tabiyat.shop/account.

use serde::Deserialize;
use serde::Serialize;

use crate::cloud_auth::{self, AuthSession};

/// Stable HTTPS API (ALB). Override with `TABIYAT_CLOUD_API_BASE` if needed.
pub const DEFAULT_CLOUD_API_BASE: &str = "https://api.tabiyat.shop";

#[derive(Clone, Debug, Deserialize)]
pub struct MeResponse {
    pub account_id: String,
    pub username: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub display_name: Option<String>,
    pub product_code: String,
    pub email_verified_hint: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlanResponse {
    pub plan_id: String,
    pub sort_order: i32,
    pub title: String,
    pub quota_gb: Option<i32>,
    pub bandwidth_band: String,
    /// Null for Custom (quote).
    pub price_inr_paise: Option<i64>,
    pub includes_live_cloud: bool,
    #[serde(default)]
    pub checkout_ready: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubscriptionResponse {
    pub plan_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub ends_at: Option<String>,
    pub title: String,
    pub quota_gb: Option<i32>,
    pub bandwidth_band: String,
    pub price_inr_paise: Option<i64>,
    pub includes_live_cloud: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BillingConfigResponse {
    pub enabled: bool,
    pub mode: String,
    #[serde(default)]
    pub providers: Vec<String>,
    pub default_provider: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckoutResponse {
    pub checkout_id: String,
    pub provider: String,
    pub mode: String,
    pub url: String,
    pub plan_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SandboxCompleteResponse {
    pub ok: bool,
    pub plan_id: String,
    pub status: String,
    pub ends_at: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CloudBootstrap {
    pub session: AuthSession,
    pub me: MeResponse,
    pub subscription: Option<SubscriptionResponse>,
    pub plans: Vec<PlanResponse>,
    pub billing: Option<BillingConfigResponse>,
    pub created_auth: bool,
}

pub fn api_base() -> String {
    std::env::var("TABIYAT_CLOUD_API_BASE")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_CLOUD_API_BASE.trim_end_matches('/').to_string())
}

fn bearer_get<T: for<'de> Deserialize<'de>>(path: &str, id_token: &str) -> Result<T, String> {
    let url = format!("{}{path}", api_base());
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {id_token}"))
        .set("Accept", "application/json")
        .call()
        .map_err(|e| map_ureq_err(e, "TABIYAT_CLOUD_API_BASE"))?;
    resp.into_json::<T>()
        .map_err(|e| format!("Cloud response: {e}"))
}

fn bearer_post_json<T: for<'de> Deserialize<'de>, B: Serialize>(
    path: &str,
    id_token: &str,
    body: &B,
) -> Result<T, String> {
    let url = format!("{}{path}", api_base());
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {id_token}"))
        .set("Accept", "application/json")
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| map_ureq_err(e, "TABIYAT_CLOUD_API_BASE"))?;
    resp.into_json::<T>()
        .map_err(|e| format!("Cloud response: {e}"))
}

fn map_ureq_err(e: ureq::Error, env_hint: &str) -> String {
    match e {
        ureq::Error::Status(code, r) => {
            let text = r.into_string().unwrap_or_default();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(err) = v.get("error").and_then(|x| x.as_str()) {
                    return err.to_string();
                }
            }
            format!("Cloud HTTP {code}")
        }
        other => format!("Cloud API unreachable ({other}). Check {env_hint}."),
    }
}

pub fn fetch_me(id_token: &str) -> Result<MeResponse, String> {
    bearer_get("/v1/me", id_token)
}

pub fn fetch_plans(id_token: &str) -> Result<Vec<PlanResponse>, String> {
    bearer_get("/v1/plans", id_token)
}

pub fn fetch_subscription(id_token: &str) -> Result<SubscriptionResponse, String> {
    bearer_get("/v1/me/subscription", id_token)
}

pub fn fetch_billing_config(id_token: &str) -> Result<BillingConfigResponse, String> {
    bearer_get("/v1/billing/config", id_token)
}

#[derive(Serialize)]
struct CheckoutBody<'a> {
    plan_id: &'a str,
    provider: &'a str,
}

#[derive(Serialize)]
struct SandboxCompleteBody<'a> {
    checkout_id: &'a str,
}

/// Start checkout for a self-serve plan (`steady` / `swift` / `priority`).
pub fn start_checkout(
    id_token: &str,
    plan_id: &str,
    provider: &str,
) -> Result<CheckoutResponse, String> {
    bearer_post_json(
        "/v1/billing/checkout",
        id_token,
        &CheckoutBody { plan_id, provider },
    )
}

/// Mock sandbox only — activates the plan without Stripe/Paddle UI.
pub fn sandbox_complete(
    id_token: &str,
    checkout_id: &str,
) -> Result<SandboxCompleteResponse, String> {
    bearer_post_json(
        "/v1/billing/sandbox/complete",
        id_token,
        &SandboxCompleteBody { checkout_id },
    )
}

/// Checkout then open the gateway URL (or complete mock sandbox in-process).
pub fn purchase_plan(
    id_token: &str,
    plan_id: &str,
    provider: &str,
) -> Result<SubscriptionResponse, String> {
    let co = start_checkout(id_token, plan_id, provider)?;
    if provider == "mock" {
        let _ = sandbox_complete(id_token, &co.checkout_id)?;
        return fetch_subscription(id_token);
    }
    cloud_auth::open_url(&co.url)?;
    Err(
        "Browser opened for payment. When finished, sign in again to refresh your plan."
            .into(),
    )
}

pub fn format_plan_price_inr(paise: Option<i64>) -> String {
    match paise {
        None => "Quote".into(),
        Some(0) => "₹0".into(),
        Some(p) => format!("₹{}/yr ex-tax", p / 100),
    }
}

/// Sign in / register, ensure cloud account, load plan catalogue + billing config.
pub fn continue_and_ensure(
    email: &str,
    password: &str,
    display_name: Option<&str>,
) -> Result<CloudBootstrap, String> {
    let (session, created_auth) = cloud_auth::continue_email(email, password, display_name)?;
    ensure_with_session(session, created_auth)
}

pub fn ensure_with_session(
    session: AuthSession,
    created_auth: bool,
) -> Result<CloudBootstrap, String> {
    let session = cloud_auth::ensure_fresh_id_token(&session)?;
    let me = fetch_me(&session.id_token)?;
    let subscription = fetch_subscription(&session.id_token).ok();
    let plans = fetch_plans(&session.id_token).unwrap_or_default();
    let billing = fetch_billing_config(&session.id_token).ok();
    Ok(CloudBootstrap {
        session,
        me,
        subscription,
        plans,
        billing,
        created_auth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_base_trims_slash() {
        assert!(!api_base().ends_with('/'));
    }

    #[test]
    fn plan_deserializes_null_custom_price() {
        let raw = r#"{
            "plan_id":"custom","sort_order":5,"title":"Custom",
            "quota_gb":null,"bandwidth_band":"custom","price_inr_paise":null,
            "includes_live_cloud":true,"checkout_ready":false
        }"#;
        let p: PlanResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(p.plan_id, "custom");
        assert!(p.price_inr_paise.is_none());
        assert!(!p.checkout_ready);
        assert_eq!(format_plan_price_inr(p.price_inr_paise), "Quote");
    }

    #[test]
    fn plan_deserializes_steady_checkout_ready() {
        let raw = r#"{
            "plan_id":"steady","sort_order":2,"title":"Steady",
            "quota_gb":10,"bandwidth_band":"lower","price_inr_paise":600000,
            "includes_live_cloud":true,"checkout_ready":true
        }"#;
        let p: PlanResponse = serde_json::from_str(raw).unwrap();
        assert!(p.checkout_ready);
        assert_eq!(format_plan_price_inr(p.price_inr_paise), "₹6000/yr ex-tax");
    }
}

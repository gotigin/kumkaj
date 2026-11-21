//! Kirano Cloud API client (milestone 1: ensure account + plans).
//!
//! Base URL defaults to the current Fargate task; override with
//! `KIRANO_CLOUD_API_BASE` (IPs change on redeploy — prefer env/DNS later).

use serde::Deserialize;

use crate::cloud_auth::{self, AuthSession};

/// Last known public task URL (ap-south-1). Prefer `KIRANO_CLOUD_API_BASE`.
pub const DEFAULT_CLOUD_API_BASE: &str = "http://3.108.215.51:8080";

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
    pub price_inr_paise: i64,
    pub includes_live_cloud: bool,
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
    pub price_inr_paise: i64,
    pub includes_live_cloud: bool,
}

#[derive(Clone, Debug)]
pub struct CloudBootstrap {
    pub session: AuthSession,
    pub me: MeResponse,
    pub subscription: Option<SubscriptionResponse>,
    pub plans: Vec<PlanResponse>,
    pub created_auth: bool,
}

pub fn api_base() -> String {
    std::env::var("KIRANO_CLOUD_API_BASE")
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
        .map_err(|e| match e {
            ureq::Error::Status(code, r) => {
                let text = r.into_string().unwrap_or_default();
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(err) = v.get("error").and_then(|x| x.as_str()) {
                        return err.to_string();
                    }
                }
                format!("Cloud HTTP {code}")
            }
            other => format!("Cloud API unreachable ({other}). Check KIRANO_CLOUD_API_BASE."),
        })?;
    resp.into_json::<T>()
        .map_err(|e| format!("Cloud response: {e}"))
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

/// Sign in / register, ensure cloud account, load plan catalogue.
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
    Ok(CloudBootstrap {
        session,
        me,
        subscription,
        plans,
        created_auth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_base_trims_slash() {
        // Default constant has no trailing slash after trim.
        assert!(!api_base().ends_with('/'));
    }
}

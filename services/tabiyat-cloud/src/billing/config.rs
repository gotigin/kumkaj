//! Billing env — sandbox by default. Live keys must never ship in sandbox mode.
//! Paddle dashboard: Chrome profile **MEARAJ (Gotigin)**.
//! Stripe was removed: India business location is incompatible with Stripe Tax (CLOUD_PLANS).
//!
//! Mock unpaid activation requires explicit `BILLING_ALLOW_MOCK=1` (local smoke only).
//! Never set that on public AWS APIs.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingMode {
    Sandbox,
    Live,
}

impl BillingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sandbox => "sandbox",
            Self::Live => "live",
        }
    }

    pub fn paddle_env(self) -> &'static str {
        match self {
            Self::Sandbox => "sandbox",
            Self::Live => "production",
        }
    }

    pub fn paddle_api_base(self) -> &'static str {
        match self {
            Self::Sandbox => "https://sandbox-api.paddle.com",
            Self::Live => "https://api.paddle.com",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Mock,
    Paddle,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mock => "mock",
            Self::Paddle => "paddle",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "mock" => Some(Self::Mock),
            "paddle" => Some(Self::Paddle),
            // Reject legacy stripe — no longer a supported provider.
            "stripe" => None,
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BillingConfig {
    pub enabled: bool,
    pub mode: BillingMode,
    /// Local unpaid mock only when true (`BILLING_ALLOW_MOCK=1`).
    pub allow_mock: bool,
    pub default_provider: Provider,
    pub success_url: String,
    pub cancel_url: String,
    pub public_api_base: String,
    pub paddle_api_key: Option<String>,
    pub paddle_webhook_secret: Option<String>,
    pub paddle_client_token: Option<String>,
}

impl BillingConfig {
    pub fn from_env() -> Self {
        build_config(None)
    }

    pub fn from_env_map<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let map: HashMap<String, String> = pairs
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect();
        build_config(Some(map))
    }

    pub fn paddle_ready(&self) -> bool {
        nonempty(&self.paddle_api_key)
            && nonempty(&self.paddle_webhook_secret)
            && nonempty(&self.paddle_client_token)
    }

    pub fn provider_enabled(&self, provider: Provider) -> bool {
        match provider {
            Provider::Mock => self.allow_mock && matches!(self.mode, BillingMode::Sandbox),
            Provider::Paddle => self.paddle_ready(),
        }
    }

    pub fn enabled_providers(&self) -> Vec<&'static str> {
        [Provider::Mock, Provider::Paddle]
            .into_iter()
            .filter(|p| self.provider_enabled(*p))
            .map(Provider::as_str)
            .collect()
    }
}

fn nonempty(v: &Option<String>) -> bool {
    v.as_ref().is_some_and(|s| !s.is_empty())
}

fn truthy(v: &str) -> bool {
    matches!(v, "1" | "true" | "yes" | "on")
}

fn build_config(map: Option<HashMap<String, String>>) -> BillingConfig {
    let lookup = |key: &str| -> Option<String> {
        match &map {
            Some(m) => m.get(key).cloned().filter(|s| !s.is_empty()),
            None => std::env::var(key).ok().filter(|s| !s.is_empty()),
        }
    };

    let mode_raw = lookup("BILLING_MODE").unwrap_or_else(|| "sandbox".into());
    let mode = match mode_raw.to_ascii_lowercase().as_str() {
        "live" | "production" => BillingMode::Live,
        _ => BillingMode::Sandbox,
    };
    let mode_ok = matches!(
        mode_raw.to_ascii_lowercase().as_str(),
        "sandbox" | "test" | "live" | "production"
    );

    let enabled = mode_ok
        && matches!(
            lookup("BILLING_ENABLED")
                .unwrap_or_else(|| "0".into())
                .as_str(),
            "1" | "true" | "yes" | "on"
        );

    let allow_mock = truthy(&lookup("BILLING_ALLOW_MOCK").unwrap_or_else(|| "0".into()));

    let paddle_api_key = lookup("PADDLE_API_KEY");
    let paddle_webhook_secret = lookup("PADDLE_WEBHOOK_SECRET");
    let paddle_client_token = lookup("PADDLE_CLIENT_TOKEN");

    let paddle_ready = nonempty(&paddle_api_key)
        && nonempty(&paddle_webhook_secret)
        && nonempty(&paddle_client_token);

    let default_provider = match lookup("BILLING_DEFAULT_PROVIDER")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "paddle" if paddle_ready => Provider::Paddle,
        "mock" if allow_mock => Provider::Mock,
        "stripe" => {
            // Legacy env — ignore; Stripe is not supported for Gotigin.
            if paddle_ready {
                Provider::Paddle
            } else if allow_mock {
                Provider::Mock
            } else {
                Provider::Paddle
            }
        }
        _ if paddle_ready => Provider::Paddle,
        _ if allow_mock => Provider::Mock,
        _ => Provider::Paddle,
    };

    BillingConfig {
        enabled,
        mode,
        allow_mock,
        default_provider,
        success_url: lookup("BILLING_SUCCESS_URL")
            .unwrap_or_else(|| "http://127.0.0.1:8012/account?billing=success".into()),
        cancel_url: lookup("BILLING_CANCEL_URL")
            .unwrap_or_else(|| "http://127.0.0.1:8012/account?billing=cancel".into()),
        public_api_base: lookup("PUBLIC_API_BASE")
            .unwrap_or_else(|| "http://127.0.0.1:8080".into()),
        paddle_api_key,
        paddle_webhook_secret,
        paddle_client_token,
    }
}

//! Billing env — sandbox by default. Live keys must never ship in sandbox mode.
//! Stripe / Paddle Dashboards: Chrome profile **MEARAJ (Gotigin)**.

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
    Stripe,
    Paddle,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mock => "mock",
            Self::Stripe => "stripe",
            Self::Paddle => "paddle",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "mock" => Some(Self::Mock),
            "stripe" => Some(Self::Stripe),
            "paddle" => Some(Self::Paddle),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BillingConfig {
    pub enabled: bool,
    pub mode: BillingMode,
    pub default_provider: Provider,
    pub success_url: String,
    pub cancel_url: String,
    pub public_api_base: String,
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    pub stripe_publishable_key: Option<String>,
    pub paddle_api_key: Option<String>,
    pub paddle_webhook_secret: Option<String>,
    pub paddle_client_token: Option<String>,
}

impl BillingConfig {
    pub fn from_env() -> Self {
        build_config(None)
    }

    /// Test helper — only the provided map is read (process env ignored).
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

    pub fn stripe_ready(&self) -> bool {
        nonempty(&self.stripe_secret_key) && nonempty(&self.stripe_webhook_secret)
    }

    pub fn paddle_ready(&self) -> bool {
        nonempty(&self.paddle_api_key)
            && nonempty(&self.paddle_webhook_secret)
            && nonempty(&self.paddle_client_token)
    }

    pub fn provider_enabled(&self, provider: Provider) -> bool {
        match provider {
            Provider::Mock => matches!(self.mode, BillingMode::Sandbox),
            Provider::Stripe => self.stripe_ready(),
            Provider::Paddle => self.paddle_ready(),
        }
    }

    pub fn enabled_providers(&self) -> Vec<&'static str> {
        [Provider::Mock, Provider::Stripe, Provider::Paddle]
            .into_iter()
            .filter(|p| self.provider_enabled(*p))
            .map(Provider::as_str)
            .collect()
    }
}

fn nonempty(v: &Option<String>) -> bool {
    v.as_ref().is_some_and(|s| !s.is_empty())
}

fn build_config(map: Option<HashMap<String, String>>) -> BillingConfig {
    let lookup = |key: &str| -> Option<String> {
        match &map {
            Some(m) => m.get(key).cloned().filter(|s| !s.is_empty()),
            None => std::env::var(key).ok().filter(|s| !s.is_empty()),
        }
    };

    let mode = match lookup("BILLING_MODE")
        .unwrap_or_else(|| "sandbox".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "live" | "production" => BillingMode::Live,
        _ => BillingMode::Sandbox,
    };

    let enabled = matches!(
        lookup("BILLING_ENABLED")
            .unwrap_or_else(|| "0".into())
            .as_str(),
        "1" | "true" | "yes" | "on"
    );

    let stripe_secret_key = lookup("STRIPE_SECRET_KEY");
    let stripe_webhook_secret = lookup("STRIPE_WEBHOOK_SECRET");
    let stripe_publishable_key = lookup("STRIPE_PUBLISHABLE_KEY");
    let paddle_api_key = lookup("PADDLE_API_KEY");
    let paddle_webhook_secret = lookup("PADDLE_WEBHOOK_SECRET");
    let paddle_client_token = lookup("PADDLE_CLIENT_TOKEN");

    let stripe_ready = nonempty(&stripe_secret_key) && nonempty(&stripe_webhook_secret);
    let paddle_ready = nonempty(&paddle_api_key)
        && nonempty(&paddle_webhook_secret)
        && nonempty(&paddle_client_token);

    let default_provider = match lookup("BILLING_DEFAULT_PROVIDER")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "stripe" if stripe_ready => Provider::Stripe,
        "paddle" if paddle_ready => Provider::Paddle,
        "mock" => Provider::Mock,
        _ if stripe_ready => Provider::Stripe,
        _ if paddle_ready => Provider::Paddle,
        _ => Provider::Mock,
    };

    BillingConfig {
        enabled,
        mode,
        default_provider,
        success_url: lookup("BILLING_SUCCESS_URL")
            .unwrap_or_else(|| "http://127.0.0.1:8014/account?billing=success".into()),
        cancel_url: lookup("BILLING_CANCEL_URL")
            .unwrap_or_else(|| "http://127.0.0.1:8014/account?billing=cancel".into()),
        public_api_base: lookup("PUBLIC_API_BASE")
            .unwrap_or_else(|| "http://127.0.0.1:8080".into()),
        stripe_secret_key,
        stripe_webhook_secret,
        stripe_publishable_key,
        paddle_api_key,
        paddle_webhook_secret,
        paddle_client_token,
    }
}

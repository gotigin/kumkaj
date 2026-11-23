//! Stripe Checkout (subscription mode) + webhook helpers.
//! Sandbox keys: `sk_test_…` / `pk_test_…` from Chrome profile **MEARAJ (Gotigin)**.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::db::{Account, PlanRow};

use super::config::BillingConfig;
use super::{BillingError, CreatedCheckout};

type HmacSha256 = Hmac<Sha256>;

pub async fn create_checkout(
    cfg: &BillingConfig,
    checkout_id: Uuid,
    account: &Account,
    plan: &PlanRow,
) -> Result<CreatedCheckout, BillingError> {
    let secret = cfg
        .stripe_secret_key
        .as_deref()
        .ok_or_else(|| BillingError::Unavailable("STRIPE_SECRET_KEY missing".into()))?;
    let price_id = plan.stripe_price_id.as_deref().ok_or_else(|| {
        BillingError::BadRequest(format!(
            "plan `{}` has no stripe_price_id — create a yearly Price in Stripe sandbox and set it on the plans row",
            plan.plan_id
        ))
    })?;

    let client = reqwest::Client::new();
    let mut form = vec![
        ("mode".to_string(), "subscription".to_string()),
        ("success_url".to_string(), cfg.success_url.clone()),
        ("cancel_url".to_string(), cfg.cancel_url.clone()),
        ("client_reference_id".to_string(), account.account_id.to_string()),
        ("line_items[0][price]".to_string(), price_id.to_string()),
        ("line_items[0][quantity]".to_string(), "1".to_string()),
        // Stripe Tax is not available for India-based accounts; prices stay tax-exclusive in our catalog.
        // ("automatic_tax[enabled]", "true") — enable only when Tax is supported for the account country.
        (
            "metadata[account_id]".to_string(),
            account.account_id.to_string(),
        ),
        ("metadata[plan_id]".to_string(), plan.plan_id.clone()),
        ("metadata[checkout_id]".to_string(), checkout_id.to_string()),
        (
            "subscription_data[metadata][account_id]".to_string(),
            account.account_id.to_string(),
        ),
        (
            "subscription_data[metadata][plan_id]".to_string(),
            plan.plan_id.clone(),
        ),
        (
            "subscription_data[metadata][checkout_id]".to_string(),
            checkout_id.to_string(),
        ),
    ];
    if let Some(email) = account.email.as_deref() {
        form.push(("customer_email".to_string(), email.to_string()));
    }

    let res = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .basic_auth(secret, None::<&str>)
        .form(&form)
        .send()
        .await
        .map_err(|e| BillingError::Upstream(format!("stripe request failed: {e}")))?;
    let status = res.status();
    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| BillingError::Upstream(format!("stripe decode failed: {e}")))?;
    if !status.is_success() {
        return Err(BillingError::Upstream(format!(
            "stripe checkout error {}: {}",
            status,
            body
        )));
    }
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BillingError::Upstream("stripe response missing id".into()))?
        .to_string();
    let url = body
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BillingError::Upstream("stripe response missing url".into()))?
        .to_string();
    Ok(CreatedCheckout {
        external_id: id,
        url,
    })
}

/// Verify `Stripe-Signature` (t=…,v1=…).
pub fn verify_signature(
    secret: &str,
    signature_header: &str,
    body: &[u8],
    tolerance_secs: i64,
) -> Result<(), String> {
    let mut timestamp = None;
    let mut v1 = None;
    for part in signature_header.split(',') {
        let mut kv = part.trim().splitn(2, '=');
        let k = kv.next().unwrap_or("");
        let v = kv.next().unwrap_or("");
        match k {
            "t" => timestamp = Some(v),
            "v1" => v1 = Some(v),
            _ => {}
        }
    }
    let timestamp = timestamp.ok_or("missing t in Stripe-Signature")?;
    let v1 = v1.ok_or("missing v1 in Stripe-Signature")?;
    let ts: i64 = timestamp
        .parse()
        .map_err(|_| "invalid Stripe-Signature timestamp")?;
    let now = chrono::Utc::now().timestamp();
    if (now - ts).abs() > tolerance_secs {
        return Err("Stripe-Signature timestamp outside tolerance".into());
    }
    let signed = format!("{timestamp}.{}", String::from_utf8_lossy(body));
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| "invalid webhook secret")?;
    mac.update(signed.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    if !constant_time_eq(expected.as_bytes(), v1.as_bytes()) {
        return Err("Stripe-Signature mismatch".into());
    }
    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripe_signature_roundtrip() {
        let secret = "whsec_test_secret";
        let body = br#"{"id":"evt_1"}"#;
        let t = chrono::Utc::now().timestamp().to_string();
        let signed = format!("{t}.{}", String::from_utf8_lossy(body));
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed.as_bytes());
        let v1 = hex::encode(mac.finalize().into_bytes());
        let header = format!("t={t},v1={v1}");
        assert!(verify_signature(secret, &header, body, 300).is_ok());
        assert!(verify_signature(secret, &header, br#"{"id":"evt_2"}"#, 300).is_err());
    }
}

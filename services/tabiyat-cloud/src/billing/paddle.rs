//! Paddle Billing (sandbox-api) transaction checkout.
//! Sandbox account / keys: Chrome profile **MEARAJ (Gotigin)**.

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
    let api_key = cfg
        .paddle_api_key
        .as_deref()
        .ok_or_else(|| BillingError::Unavailable("PADDLE_API_KEY missing".into()))?;
    let price_id = plan.paddle_price_id.as_deref().ok_or_else(|| {
        BillingError::BadRequest(format!(
            "plan `{}` has no paddle_price_id — create a yearly Price in Paddle sandbox and set it on the plans row",
            plan.plan_id
        ))
    })?;

    let body = serde_json::json!({
        "items": [{ "price_id": price_id, "quantity": 1 }],
        "custom_data": {
            "account_id": account.account_id.to_string(),
            "plan_id": plan.plan_id,
            "checkout_id": checkout_id.to_string(),
            "product": "tabiyat",
        },
        "collection_mode": "automatic",
    });

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/transactions", cfg.mode.paddle_api_base()))
        .bearer_auth(api_key)
        .header("Paddle-Version", "1")
        .json(&body)
        .send()
        .await
        .map_err(|e| BillingError::Upstream(format!("paddle request failed: {e}")))?;
    let status = res.status();
    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| BillingError::Upstream(format!("paddle decode failed: {e}")))?;
    if !status.is_success() {
        return Err(BillingError::Upstream(format!(
            "paddle transaction error {}: {}",
            status, body
        )));
    }
    let data = body.get("data").unwrap_or(&body);
    let id = data
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BillingError::Upstream("paddle response missing data.id".into()))?
        .to_string();
    let url = data
        .pointer("/checkout/url")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            // Overlay checkout can also be opened client-side with Paddle.js;
            // still return a success deep-link so the API contract stays stable.
            format!(
                "{}&paddle_tx={id}&checkout_id={checkout_id}",
                cfg.success_url
            )
        });
    Ok(CreatedCheckout {
        external_id: id,
        url,
    })
}

/// Paddle Billing notification signature (ts;h1=…).
pub fn verify_signature(secret: &str, signature_header: &str, body: &[u8]) -> Result<(), String> {
    let mut ts = None;
    let mut h1 = None;
    for part in signature_header.split(';') {
        let mut kv = part.trim().splitn(2, '=');
        let k = kv.next().unwrap_or("");
        let v = kv.next().unwrap_or("");
        match k {
            "ts" => ts = Some(v),
            "h1" => h1 = Some(v),
            _ => {}
        }
    }
    let ts = ts.ok_or("missing ts in Paddle-Signature")?;
    let h1 = h1.ok_or("missing h1 in Paddle-Signature")?;
    let signed = format!("{ts}:{}", String::from_utf8_lossy(body));
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| "invalid paddle webhook secret")?;
    mac.update(signed.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    if !constant_time_eq(expected.as_bytes(), h1.as_bytes()) {
        return Err("Paddle-Signature mismatch".into());
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
    fn paddle_signature_roundtrip() {
        let secret = "pdl_ntfsec_test";
        let body = br#"{"event_id":"evt_1"}"#;
        let ts = "1700000000";
        let signed = format!("{ts}:{}", String::from_utf8_lossy(body));
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed.as_bytes());
        let h1 = hex::encode(mac.finalize().into_bytes());
        let header = format!("ts={ts};h1={h1}");
        assert!(verify_signature(secret, &header, body).is_ok());
        assert!(verify_signature(secret, &header, br#"{"event_id":"x"}"#).is_err());
    }
}

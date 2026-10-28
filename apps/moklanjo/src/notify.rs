//! Outbound notify hook (WhatsApp / SMS / webhook) — local-first stub.
//!
//! Records intent + payload on the outbox. Public track URL is also formed
//! here. SMS delivery is planned as a cloudless Local SMS Gateway (Android
//! sibling over LAN/Bluetooth) and optional paid Cloud SMS — see
//! `docs/SMS_MESSAGING.md`. Never gates booking or books.

use crate::domain::new_id;
use chrono::Local;

/// One queued notification.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct NotifyOutboxRow {
    pub id: String,
    pub created_at: String,
    pub channel: String,
    pub to_addr: String,
    pub template_key: String,
    pub payload_json: String,
    pub status: String,
    pub last_error: String,
}

/// Build a public track path for an AWB (served by the local track endpoint).
pub fn public_track_path(awb: &str) -> String {
    let enc: String = awb
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("/track/{enc}")
}

pub fn public_track_url(base: &str, awb: &str) -> String {
    let base = base.trim_end_matches('/');
    format!("{base}{}", public_track_path(awb))
}

/// Queue a booking / status alert (no network send yet).
pub fn draft_booking_alert(
    to_phone: &str,
    awb: &str,
    destination: &str,
    track_url: &str,
) -> NotifyOutboxRow {
    let payload = serde_json::json!({
        "awb": awb,
        "destination": destination,
        "track_url": track_url,
        "message": format!(
            "Your shipment {awb} to {destination} is booked. Track: {track_url}"
        ),
    });
    NotifyOutboxRow {
        id: new_id(),
        created_at: Local::now().to_rfc3339(),
        channel: "whatsapp_or_sms".into(),
        to_addr: to_phone.trim().into(),
        template_key: "booking_alert".into(),
        payload_json: payload.to_string(),
        status: "queued".into(),
        last_error: String::new(),
    }
}

/// Parse bulk CSV booking rows: `destination,consignee,actual_kg,cod_paise,note`
pub fn parse_bulk_booking_csv(raw: &str) -> Result<Vec<BulkBookingRow>, String> {
    let mut out = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if i == 0 && line.to_lowercase().contains("destination") {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if cols.len() < 2 {
            return Err(format!("Line {}: need at least destination,consignee", i + 1));
        }
        let actual_kg = cols
            .get(2)
            .and_then(|s| crate::precise::parse_non_negative(s))
            .unwrap_or(crate::precise::ZERO);
        let cod_paise: i64 = cols.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
        out.push(BulkBookingRow {
            destination: cols[0].into(),
            consignee: cols[1].into(),
            actual_kg,
            cod_paise,
            note: cols.get(4).unwrap_or(&"").to_string(),
        });
    }
    Ok(out)
}

#[derive(Clone, Debug, Default)]
pub struct BulkBookingRow {
    pub destination: String,
    pub consignee: String,
    pub actual_kg: rust_decimal::Decimal,
    pub cod_paise: i64,
    pub note: String,
}

/// Extract the human message from an outbox payload JSON.
pub fn message_from_payload(payload_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(payload_json)
        .ok()
        .and_then(|v| {
            v.get("message")
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| payload_json.trim().to_string())
}

/// Digits-only phone for wa.me (keeps leading country code if present).
pub fn phone_digits(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// `https://wa.me/<digits>?text=<urlencoded>` — open in WhatsApp when the
/// device has Net; app never requires it.
pub fn whatsapp_click_url(to_phone: &str, text: &str) -> String {
    let digits = phone_digits(to_phone);
    let enc = urlencoding_minimal(text);
    if digits.is_empty() {
        format!("https://wa.me/?text={enc}")
    } else {
        format!("https://wa.me/{digits}?text={enc}")
    }
}

/// SMS body ready to copy / hand to an Android `sms:` intent stub.
pub fn sms_body_from_row(row: &NotifyOutboxRow) -> String {
    message_from_payload(&row.payload_json)
}

/// Android `sms:` URI (cfg-gated callers may open it). No send from Rust.
pub fn android_sms_intent_uri(to_phone: &str, body: &str) -> String {
    let digits = phone_digits(to_phone);
    let enc = urlencoding_minimal(body);
    format!("sms:{digits}?body={enc}")
}

/// One-tap WhatsApp share for a booking receipt (Net optional on device).
pub fn whatsapp_receipt_share_url(to_phone: &str, receipt_text: &str) -> String {
    whatsapp_click_url(to_phone, receipt_text)
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bulk_skips_header() {
        let csv = "destination,consignee,actual_kg,cod_paise\nPune,Riya,2.5,0\n";
        let rows = parse_bulk_booking_csv(csv).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].destination, "Pune");
        assert_eq!(rows[0].actual_kg, "2.5".parse().unwrap());
    }

    #[test]
    fn track_path_sanitises() {
        assert_eq!(public_track_path("AWB-1"), "/track/AWB-1");
    }

    #[test]
    fn whatsapp_url_prefills() {
        let url = whatsapp_click_url("+91 98765 43210", "Your shipment AWB-1");
        assert!(url.starts_with("https://wa.me/919876543210?text="));
        assert!(url.contains("Your%20shipment"));
    }

    #[test]
    fn sms_intent_stub() {
        let u = android_sms_intent_uri("999", "hello");
        assert_eq!(u, "sms:999?body=hello");
    }
}

//! Outbound notify hook (WhatsApp / SMS / webhook) — local-first stub.
//!
//! Does not call messaging APIs yet. Records intent + payload so a later
//! connector can drain the outbox (order-ready / bill alerts). WhatsApp share
//! opens `wa.me` on the device (Net optional); SMS queues for Android SIM later.

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

/// Queue an order alert (no network send yet).
pub fn draft_order_alert(to_phone: &str, bill_no: &str, message: &str) -> NotifyOutboxRow {
    let payload = serde_json::json!({
        "bill_no": bill_no,
        "message": message,
    });
    NotifyOutboxRow {
        id: new_id(),
        created_at: Local::now().to_rfc3339(),
        channel: "whatsapp_or_sms".into(),
        to_addr: to_phone.trim().into(),
        template_key: "order_alert".into(),
        payload_json: payload.to_string(),
        status: "queued".into(),
        last_error: String::new(),
    }
}

/// One-tap WhatsApp share: URL for the device browser / app (Net optional).
pub fn whatsapp_share_url(bill_text: &str) -> String {
    let mut enc = String::new();
    for b in bill_text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => enc.push(b as char),
            b' ' => enc.push_str("%20"),
            _ => enc.push_str(&format!("%{b:02X}")),
        }
    }
    format!("https://wa.me/?text={enc}")
}

/// Queue SMS bill text for a later Android SIM drain (never requires Net now).
pub fn queue_sms_bill(to_phone: &str, bill_text: &str) -> NotifyOutboxRow {
    queue_sms_message(to_phone, bill_text, "e_bill")
}

/// Queue a free-text SMS (CRM campaign / birthday wish).
pub fn queue_sms_message(to_phone: &str, message: &str, template_key: &str) -> NotifyOutboxRow {
    let payload = serde_json::json!({
        "message": message,
        "intent": "sms_send",
    });
    NotifyOutboxRow {
        id: new_id(),
        created_at: Local::now().to_rfc3339(),
        channel: "sms".into(),
        to_addr: to_phone.trim().into(),
        template_key: template_key.into(),
        payload_json: payload.to_string(),
        status: "queued".into(),
        last_error: String::new(),
    }
}

/// Queue a WhatsApp intent (device can open `wa.me` when Net is up).
pub fn queue_whatsapp_message(to_phone: &str, message: &str, template_key: &str) -> NotifyOutboxRow {
    let payload = serde_json::json!({
        "message": message,
        "intent": "whatsapp_send",
        "whatsapp_url": whatsapp_share_url(message),
    });
    NotifyOutboxRow {
        id: new_id(),
        created_at: Local::now().to_rfc3339(),
        channel: "whatsapp".into(),
        to_addr: to_phone.trim().into(),
        template_key: template_key.into(),
        payload_json: payload.to_string(),
        status: "queued".into(),
        last_error: String::new(),
    }
}

/// Stub drain: marks queued rows as written to a local intent file path note
/// without calling the network. Safe offline.
pub fn drain_stub_mark_intent(row: &NotifyOutboxRow) -> NotifyOutboxRow {
    let mut out = row.clone();
    out.status = "intent_written".into();
    out.last_error = String::new();
    out.payload_json = serde_json::json!({
        "original": serde_json::from_str::<serde_json::Value>(&row.payload_json).unwrap_or_default(),
        "drain": "local_stub",
        "whatsapp_url": serde_json::from_str::<serde_json::Value>(&row.payload_json)
            .ok()
            .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(whatsapp_share_url))
            .unwrap_or_default(),
    })
    .to_string();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_alert_queues_payload() {
        let row = draft_order_alert(" 9999999999 ", "INV-MAIN-000001", "Your order is ready");
        assert_eq!(row.status, "queued");
        assert_eq!(row.to_addr, "9999999999");
        assert!(row.payload_json.contains("INV-MAIN-000001"));
    }

    #[test]
    fn sms_and_whatsapp_share_helpers() {
        let sms = queue_sms_bill("98", "Bill 1");
        assert_eq!(sms.channel, "sms");
        let url = whatsapp_share_url("Hello");
        assert!(url.starts_with("https://wa.me/?text="));
        let drained = drain_stub_mark_intent(&sms);
        assert_eq!(drained.status, "intent_written");
    }
}

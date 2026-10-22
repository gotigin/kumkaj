//! Outbound notify hook (WhatsApp / SMS / webhook) — local-first stub.
//!
//! Does not call messaging APIs yet. Records intent + payload so a later
//! connector can drain the outbox (order-ready / bill alerts).

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
}

//! Outbound notify hook (WhatsApp / SMS / webhook) — local-first stub.
//!
//! Does not call carrier or messaging APIs yet. Records intent + payload so a
//! later connector can drain the outbox. Public track URL is also formed here.

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
}

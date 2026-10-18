//! Public track-by-AWB JSON (local HTTP or file export).

use crate::domain::Sale;
use crate::manifest::{TrackingEvent, TrackingStatus};

#[derive(Clone, Debug, serde::Serialize)]
pub struct PublicTrackView {
    pub awb_no: String,
    pub status: String,
    pub status_label: String,
    pub destination: String,
    pub consignee_name: String,
    pub chargeable_weight_kg: rust_decimal::Decimal,
    pub booked_at: String,
    pub events: Vec<PublicTrackEvent>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PublicTrackEvent {
    pub at: String,
    pub status: String,
    pub status_label: String,
    pub note: String,
}

pub fn view_from_sale(sale: &Sale, events: &[TrackingEvent]) -> PublicTrackView {
    let status = TrackingStatus::parse(&sale.tracking_status);
    let mut evs: Vec<PublicTrackEvent> = events
        .iter()
        .map(|e| {
            let st = TrackingStatus::parse(&e.status);
            PublicTrackEvent {
                at: if e.user_at.trim().is_empty() {
                    e.created_at.clone()
                } else {
                    e.user_at.clone()
                },
                status: st.as_str().into(),
                status_label: st.label().into(),
                note: e.note.clone(),
            }
        })
        .collect();
    if evs.is_empty() {
        evs.push(PublicTrackEvent {
            at: sale.user_at.clone(),
            status: status.as_str().into(),
            status_label: status.label().into(),
            note: String::new(),
        });
    }
    PublicTrackView {
        awb_no: sale.awb_no.clone(),
        status: status.as_str().into(),
        status_label: status.label().into(),
        destination: sale.destination.clone(),
        consignee_name: sale.consignee_name.clone(),
        chargeable_weight_kg: sale.chargeable_weight_kg,
        booked_at: sale.user_at.clone(),
        events: evs,
    }
}

pub fn to_json(view: &PublicTrackView) -> Result<String, String> {
    serde_json::to_string_pretty(view).map_err(|e| e.to_string())
}

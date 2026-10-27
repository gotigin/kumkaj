//! Handover / dispatch lists — carrier bag **or** own-fleet rider run.
//!
//! A complete agency BOS covers both shapes:
//! - Hand a bag of AWBs to Delhivery / DTDC (carrier party).
//! - Assign AWBs to the shop's own riders / vans (local last-mile).
//!
//! Live GPS maps and AI route optimisation are progressive depth on this same
//! spine; the document + assignment model is the must-have.

use crate::domain::new_id;

/// How this manifest moves parcels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandoverKind {
    /// Named external carrier (party kind Carrier).
    Carrier,
    /// Shop's own rider / van.
    OwnFleet,
}

impl HandoverKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Carrier => "carrier",
            Self::OwnFleet => "own_fleet",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "own_fleet" | "fleet" | "rider" => Self::OwnFleet,
            _ => Self::Carrier,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Carrier => "Carrier handover",
            Self::OwnFleet => "Own-fleet dispatch",
        }
    }
}

/// Rider / van row for own-fleet dispatch.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Rider {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub vehicle_label: String,
    pub active: bool,
    pub note: String,
}

impl Rider {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            name: name.into(),
            phone: String::new(),
            vehicle_label: String::new(),
            active: true,
            note: String::new(),
        }
    }
}

/// Manifest / run sheet header.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: String,
    pub created_at: String,
    pub user_at: String,
    pub device_at: String,
    pub kind: String,
    /// Carrier party id when `kind = carrier`.
    pub carrier_party_id: String,
    pub carrier_name: String,
    /// Rider id when `kind = own_fleet`.
    pub rider_id: String,
    pub rider_name: String,
    pub note: String,
    pub status: String,
    pub line_count: i64,
}

/// One AWB on a manifest.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ManifestLine {
    pub id: String,
    pub manifest_id: String,
    pub sale_id: String,
    pub awb_no: String,
    pub destination: String,
    pub consignee_name: String,
}

/// Manifest row joined with live sale fields for run-sheet / DRS print.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ManifestLineDetail {
    pub line: ManifestLine,
    pub weight_kg: rust_decimal::Decimal,
    pub cod_paise: i64,
}

/// Fixed NDR (failed delivery) reason codes — desk chips / suggest list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NdrReason {
    DoorLocked,
    Refused,
    AddressWrong,
    CustomerUnavailable,
    Other,
}

impl NdrReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DoorLocked => "door_locked",
            Self::Refused => "refused",
            Self::AddressWrong => "address_wrong",
            Self::CustomerUnavailable => "customer_unavailable",
            Self::Other => "other",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DoorLocked => "Door locked",
            Self::Refused => "Refused",
            Self::AddressWrong => "Address wrong",
            Self::CustomerUnavailable => "Customer unavailable",
            Self::Other => "Other",
        }
    }

    /// Locale key for the reason chip / label.
    pub fn locale_key(self) -> &'static str {
        match self {
            Self::DoorLocked => "ndr_door_locked",
            Self::Refused => "ndr_refused",
            Self::AddressWrong => "ndr_address_wrong",
            Self::CustomerUnavailable => "ndr_customer_unavailable",
            Self::Other => "ndr_other",
        }
    }

    pub fn all() -> &'static [NdrReason] {
        &[
            Self::DoorLocked,
            Self::Refused,
            Self::AddressWrong,
            Self::CustomerUnavailable,
            Self::Other,
        ]
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "door_locked" | "door locked" => Self::DoorLocked,
            "refused" => Self::Refused,
            "address_wrong" | "address wrong" | "wrong_address" => Self::AddressWrong,
            "customer_unavailable" | "customer unavailable" | "unavailable" => {
                Self::CustomerUnavailable
            }
            _ => Self::Other,
        }
    }
}

/// Default max failed-delivery attempts before auto-RTO (overridable via shop meta).
pub const NDR_MAX_ATTEMPTS_DEFAULT: i64 = 3;

/// Plain-text carrier bag / rider DRS run sheet (receipt printer).
pub fn format_manifest_text(
    shop_name: &str,
    manifest: &Manifest,
    lines: &[ManifestLineDetail],
    money_fmt: &dyn Fn(i64) -> String,
) -> String {
    let shop = if shop_name.trim().is_empty() {
        "Your shop"
    } else {
        shop_name.trim()
    };
    let title = if manifest.kind == "own_fleet" {
        format!(
            "Rider run / DRS — {}",
            if manifest.rider_name.trim().is_empty() {
                "(rider)"
            } else {
                manifest.rider_name.trim()
            }
        )
    } else {
        format!(
            "Carrier bag — {}",
            if manifest.carrier_name.trim().is_empty() {
                "(carrier)"
            } else {
                manifest.carrier_name.trim()
            }
        )
    };
    let date = &manifest.created_at[..manifest.created_at.len().min(16)];
    let mut out = String::new();
    out.push_str(shop);
    out.push('\n');
    out.push_str(&title);
    out.push('\n');
    out.push_str(date);
    out.push('\n');
    if !manifest.note.trim().is_empty() {
        out.push_str("Note: ");
        out.push_str(manifest.note.trim());
        out.push('\n');
    }
    out.push_str("--------------------------------\n");
    let mut cod_total = 0i64;
    for (i, row) in lines.iter().enumerate() {
        cod_total = cod_total.saturating_add(row.cod_paise.max(0));
        let wt = crate::precise::format_kg(row.weight_kg);
        let cod = if row.cod_paise > 0 {
            format!(" · COD {}", money_fmt(row.cod_paise))
        } else {
            String::new()
        };
        out.push_str(&format!(
            "{}. {} · {} · {} · {} kg{}\n",
            i + 1,
            row.line.awb_no,
            row.line.destination,
            row.line.consignee_name,
            wt,
            cod
        ));
    }
    out.push_str("--------------------------------\n");
    out.push_str(&format!("Pieces: {}\n", lines.len()));
    out.push_str(&format!("COD total: {}\n", money_fmt(cod_total)));
    out.push('\n');
    out.push_str("Received by ________  Sign ________  Time ________\n");
    out
}

/// AWB tracking statuses — works for carrier portal *and* own riders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackingStatus {
    Booked,
    PickedUp,
    HandedOver,
    InTransit,
    OutForDelivery,
    Delivered,
    /// Non-delivery report — failed attempt; may reattempt or RTO.
    Ndr,
    Rto,
    Cancelled,
}

impl TrackingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Booked => "booked",
            Self::PickedUp => "picked_up",
            Self::HandedOver => "handed_over",
            Self::InTransit => "in_transit",
            Self::OutForDelivery => "out_for_delivery",
            Self::Delivered => "delivered",
            Self::Ndr => "ndr",
            Self::Rto => "rto",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Booked => "Booked",
            Self::PickedUp => "Picked up",
            Self::HandedOver => "Handed to carrier",
            Self::InTransit => "In transit",
            Self::OutForDelivery => "Out for delivery",
            Self::Delivered => "Delivered",
            Self::Ndr => "NDR",
            Self::Rto => "RTO",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn all() -> &'static [TrackingStatus] {
        &[
            Self::Booked,
            Self::PickedUp,
            Self::HandedOver,
            Self::InTransit,
            Self::OutForDelivery,
            Self::Delivered,
            Self::Ndr,
            Self::Rto,
            Self::Cancelled,
        ]
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "picked_up" => Self::PickedUp,
            "handed_over" => Self::HandedOver,
            "in_transit" => Self::InTransit,
            "out_for_delivery" => Self::OutForDelivery,
            "delivered" => Self::Delivered,
            "ndr" | "failed" | "failed_delivery" => Self::Ndr,
            "rto" => Self::Rto,
            "cancelled" => Self::Cancelled,
            _ => Self::Booked,
        }
    }
}

/// Append-only status / POD note on an AWB.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackingEvent {
    pub id: String,
    pub sale_id: String,
    pub awb_no: String,
    pub created_at: String,
    pub user_at: String,
    pub device_at: String,
    pub status: String,
    pub note: String,
    /// Optional local path / stub for POD photo.
    pub pod_ref: String,
    pub actor_name: String,
    /// Optional GPS from rider phone OS (0 = unset). No map tiles required.
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub lon: f64,
}

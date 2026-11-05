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
            Self::Ndr => "Failed delivery",
            Self::Rto => "Returned to sender",
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

    /// Locale key of the status as staff read it (never the stored code).
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Booked => "trk_booked",
            Self::PickedUp => "trk_picked_up",
            Self::HandedOver => "trk_handed_over",
            Self::InTransit => "trk_in_transit",
            Self::OutForDelivery => "out_for_delivery",
            Self::Delivered => "trk_delivered",
            Self::Ndr => "trk_failed",
            Self::Rto => "trk_rto",
            Self::Cancelled => "trk_cancelled",
        }
    }

    /// Delivered, returned and cancelled parcels are finished: no desk
    /// action moves them again.
    pub fn is_final(self) -> bool {
        matches!(self, Self::Delivered | Self::Rto | Self::Cancelled)
    }

    /// Still in the office — the only parcels a carrier bag or a rider run
    /// may take.
    pub fn is_dispatchable(self) -> bool {
        matches!(self, Self::Booked | Self::PickedUp)
    }

    /// Moves the desk may make from this status. The store refuses every
    /// other one, so a stray tap, a pasted list or a late rider sync can
    /// never send a delivered parcel out again (vision §18: the books say
    /// what really happened).
    pub fn can_move_to(self, next: TrackingStatus) -> bool {
        use TrackingStatus::*;
        if self == next {
            return false;
        }
        match self {
            // In the office: picked up, bagged for a carrier, out with a
            // rider, collected at the counter, or cancelled.
            Booked | PickedUp => matches!(
                next,
                PickedUp | HandedOver | InTransit | OutForDelivery | Delivered | Cancelled
            ),
            // The carrier has it: we only hear what happened next.
            HandedOver | InTransit => {
                matches!(next, InTransit | OutForDelivery | Delivered | Ndr | Rto)
            }
            OutForDelivery => matches!(next, Delivered | Ndr | Rto),
            // Failed attempt: out again, delivered after all, or sent back.
            Ndr => matches!(next, OutForDelivery | Delivered | Rto),
            Delivered | Rto | Cancelled => false,
        }
    }

    /// Which Ops board lane shows a parcel in this status.
    pub fn lane(self) -> OpsLane {
        match self {
            Self::Booked | Self::PickedUp => OpsLane::ToDispatch,
            Self::HandedOver | Self::InTransit => OpsLane::WithCarrier,
            Self::OutForDelivery => OpsLane::OutForDelivery,
            Self::Ndr => OpsLane::Failed,
            Self::Delivered | Self::Rto | Self::Cancelled => OpsLane::Done,
        }
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

/// The Ops board's lanes: where a parcel is in the office's day, in the
/// order the work happens. Each lane offers only the moves that fit it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OpsLane {
    /// Booked or picked up — still in the office, waiting for a bag or a run.
    #[default]
    ToDispatch,
    /// Handed to a carrier (or in transit) — we wait for their word.
    WithCarrier,
    /// Out with one of our riders.
    OutForDelivery,
    /// A delivery failed — try again or send it back.
    Failed,
    /// Delivered, returned or cancelled today.
    Done,
}

impl OpsLane {
    pub const ALL: [OpsLane; 5] = [
        OpsLane::ToDispatch,
        OpsLane::WithCarrier,
        OpsLane::OutForDelivery,
        OpsLane::Failed,
        OpsLane::Done,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }

    /// Stored statuses this lane shows.
    pub fn statuses(self) -> &'static [TrackingStatus] {
        use TrackingStatus::*;
        match self {
            Self::ToDispatch => &[Booked, PickedUp],
            Self::WithCarrier => &[HandedOver, InTransit],
            Self::OutForDelivery => &[OutForDelivery],
            Self::Failed => &[Ndr],
            Self::Done => &[Delivered, Rto, Cancelled],
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::ToDispatch => "lane_to_dispatch",
            Self::WithCarrier => "lane_with_carrier",
            Self::OutForDelivery => "out_for_delivery",
            Self::Failed => "lane_failed",
            Self::Done => "lane_done_today",
        }
    }

    pub fn hint_key(self) -> &'static str {
        match self {
            Self::ToDispatch => "lane_to_dispatch_hint",
            Self::WithCarrier => "lane_with_carrier_hint",
            Self::OutForDelivery => "lane_out_hint",
            Self::Failed => "lane_failed_hint",
            Self::Done => "lane_done_hint",
        }
    }
}

/// One parcel on the Ops board: the booking plus where it is now and since
/// when — read in one query so the board never asks per row.
#[derive(Clone, Debug, Default)]
pub struct OpsRow {
    pub sale_id: String,
    pub awb_no: String,
    pub destination: String,
    pub consignee_name: String,
    pub origin_name: String,
    pub status: String,
    pub piece_count: i64,
    pub chargeable_weight_kg: rust_decimal::Decimal,
    pub total_paise: i64,
    pub cod_paise: i64,
    /// Carrier party name (bag or booked-with) or rider name — who has it.
    pub holder: String,
    /// When the parcel last moved (booking time if it never did).
    pub moved_at: String,
    pub attempt_count: i64,
    pub ndr_reason: String,
    pub next_attempt_at: String,
}

impl OpsRow {
    pub fn tracking(&self) -> TrackingStatus {
        TrackingStatus::parse(&self.status)
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

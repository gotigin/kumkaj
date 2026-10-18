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

/// AWB tracking statuses — works for carrier portal *and* own riders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackingStatus {
    Booked,
    PickedUp,
    HandedOver,
    InTransit,
    OutForDelivery,
    Delivered,
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
}

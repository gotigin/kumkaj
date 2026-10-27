//! Rate master maths: zone × first-kg / extra-kg slabs + booking extras.
//!
//! `price_paise` on a product remains the flat ₹/kg (or piece) fallback when
//! slab fields are zero. When `first_kg` > 0, chargeable weight uses the slab.
//! Weight maths use [`rust_decimal::Decimal`] — no native floats.

use rust_decimal::Decimal;

use crate::domain::Money;
use crate::precise::{self, ZERO};

/// Default first-weight slab used when the rate has no explicit first kg.
pub fn default_first_kg() -> Decimal {
    Decimal::from(1)
}

/// Zone master row — destination bucket on rates and bookings.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub note: String,
    pub active: bool,
}

/// Built-in extra charge kinds a desk can add on a docket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtraChargeKind {
    Fuel,
    Docket,
    Oda,
    Packing,
    CodFee,
    /// Return-to-origin fee when auto-RTO finishes a Partial NDR.
    RtoFee,
    Other,
}

impl ExtraChargeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fuel => "fuel",
            Self::Docket => "docket",
            Self::Oda => "oda",
            Self::Packing => "packing",
            Self::CodFee => "cod_fee",
            Self::RtoFee => "rto_fee",
            Self::Other => "other",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Fuel => "Fuel surcharge",
            Self::Docket => "Docket fee",
            Self::Oda => "ODA",
            Self::Packing => "Packing",
            Self::CodFee => "COD fee",
            Self::RtoFee => "RTO fee",
            Self::Other => "Extra charge",
        }
    }

    pub fn all() -> &'static [ExtraChargeKind] {
        &[
            Self::Fuel,
            Self::Docket,
            Self::Oda,
            Self::Packing,
            Self::CodFee,
            Self::RtoFee,
            Self::Other,
        ]
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "fuel" => Self::Fuel,
            "docket" => Self::Docket,
            "oda" => Self::Oda,
            "packing" => Self::Packing,
            "cod_fee" | "cod" => Self::CodFee,
            "rto_fee" | "rto" => Self::RtoFee,
            _ => Self::Other,
        }
    }
}

/// Catalog row for a reusable extra (amount may be overridden at booking).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtraChargeType {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub default_paise: i64,
    pub active: bool,
}

/// Slab fields stored on a rate/product.
#[derive(Clone, Debug, Default, Copy)]
pub struct RateSlab {
    /// First weight included in `first_paise` (kg). 0 = use flat ₹/kg only.
    pub first_kg: Decimal,
    /// Amount for the first slab (paise).
    pub first_paise: i64,
    /// Amount per additional kg beyond `first_kg` (paise).
    pub extra_per_kg_paise: i64,
    /// Flat ₹/kg fallback when slabs are unset (`product.price`).
    pub flat_per_kg_paise: i64,
}

impl RateSlab {
    pub fn uses_slab(self) -> bool {
        self.first_kg > ZERO && (self.first_paise > 0 || self.extra_per_kg_paise > 0)
    }

    /// Freight for `chargeable_kg` under this slab (or flat ₹/kg).
    pub fn freight_paise(self, chargeable_kg: Decimal) -> Result<i64, String> {
        let kg = if chargeable_kg > ZERO {
            chargeable_kg
        } else {
            ZERO
        };
        if kg <= ZERO {
            return Ok(0);
        }
        if self.uses_slab() {
            let first = if self.first_kg > ZERO {
                self.first_kg
            } else {
                default_first_kg()
            };
            if kg <= first {
                return Ok(self.first_paise.max(0));
            }
            let extra_kg = kg - first;
            let extra = precise::mul_to_i64(extra_kg, Decimal::from(self.extra_per_kg_paise.max(0)))?;
            return self
                .first_paise
                .checked_add(extra.max(0))
                .ok_or_else(|| "Freight amount overflow".to_string());
        }
        precise::mul_to_i64(kg, Decimal::from(self.flat_per_kg_paise.max(0)))
    }

    pub fn freight_money(self, chargeable_kg: Decimal) -> Result<Money, String> {
        Ok(Money(self.freight_paise(chargeable_kg)?))
    }
}

/// One extra line the desk added to the booking.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookingExtra {
    pub kind: String,
    pub name: String,
    pub amount_paise: i64,
}

impl BookingExtra {
    pub fn from_kind(kind: ExtraChargeKind, amount_paise: i64) -> Self {
        Self {
            kind: kind.as_str().into(),
            name: kind.label().into(),
            amount_paise: amount_paise.max(0),
        }
    }
}

/// One carrier's cost vs our sell freight for the same zone/weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarrierCompareRow {
    pub carrier_party_id: String,
    pub carrier_name: String,
    pub cost_paise: i64,
    pub margin_paise: i64,
}

/// Build compare rows: sell freight vs each carrier's cost freight.
pub fn compare_carriers(sell_paise: i64, carriers: &[(String, String, i64)]) -> Vec<CarrierCompareRow> {
    carriers
        .iter()
        .map(|(id, name, cost)| {
            let cost_paise = (*cost).max(0);
            CarrierCompareRow {
                carrier_party_id: id.clone(),
                carrier_name: name.clone(),
                cost_paise,
                margin_paise: sell_paise.saturating_sub(cost_paise),
            }
        })
        .collect()
}

/// Format a one-line booking desk summary of sell + carrier margins.
pub fn format_carrier_compare(sell_paise: i64, rows: &[CarrierCompareRow], currency: &str) -> String {
    use crate::locale;
    let mut parts = vec![format!(
        "Our charge {}",
        locale::format_money(sell_paise, currency)
    )];
    for row in rows {
        parts.push(format!(
            "{} cost {} (margin {})",
            row.carrier_name,
            locale::format_money(row.cost_paise, currency),
            locale::format_money(row.margin_paise, currency)
        ));
    }
    parts.join(" · ")
}

/// Pincode serviceability hit grouped by zone.
#[derive(Clone, Debug, Default)]
pub struct PincodeZoneHit {
    pub pincode: String,
    pub zone_id: String,
    pub zone_name: String,
    pub carriers: Vec<(String, String)>,
    pub note: String,
}

impl PincodeZoneHit {
    pub fn serves_label(&self) -> String {
        let names: Vec<&str> = self
            .carriers
            .iter()
            .map(|(_, n)| n.as_str())
            .filter(|n| !n.is_empty())
            .collect();
        if names.is_empty() {
            format!("Zone {}", self.zone_name)
        } else {
            format!("Serves: {}", names.join(", "))
        }
    }
}

/// True when `raw` is a 6-digit Indian pincode.
pub fn is_india_pincode(raw: &str) -> bool {
    let t = raw.trim();
    t.len() == 6 && t.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn slab_first_then_extra() {
        let slab = RateSlab {
            first_kg: d("1"),
            first_paise: 50_00,
            extra_per_kg_paise: 20_00,
            flat_per_kg_paise: 0,
        };
        assert_eq!(slab.freight_paise(d("0.5")).unwrap(), 50_00);
        assert_eq!(slab.freight_paise(d("1")).unwrap(), 50_00);
        assert_eq!(slab.freight_paise(d("2.5")).unwrap(), 50_00 + 30_00);
    }

    #[test]
    fn flat_per_kg_when_no_slab() {
        let slab = RateSlab {
            first_kg: ZERO,
            first_paise: 0,
            extra_per_kg_paise: 0,
            flat_per_kg_paise: 40_00,
        };
        assert_eq!(slab.freight_paise(d("2")).unwrap(), 80_00);
    }

    #[test]
    fn odd_weight_extra_is_exact() {
        let slab = RateSlab {
            first_kg: d("1"),
            first_paise: 50_00,
            extra_per_kg_paise: 20_00,
            flat_per_kg_paise: 0,
        };
        // 1.5 kg → 0.5 extra × 2000 = 1000
        assert_eq!(slab.freight_paise(d("1.5")).unwrap(), 60_00);
    }

    #[test]
    fn compare_carriers_margins() {
        let rows = compare_carriers(
            100_00,
            &[
                ("d1".into(), "Delhivery".into(), 70_00),
                ("d2".into(), "DTDC".into(), 80_00),
            ],
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].margin_paise, 30_00);
        assert_eq!(rows[1].margin_paise, 20_00);
        let line = format_carrier_compare(100_00, &rows, "INR");
        assert!(line.contains("Delhivery"));
        assert!(line.contains("DTDC"));
    }
}

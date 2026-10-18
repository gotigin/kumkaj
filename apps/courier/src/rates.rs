//! Rate master maths: zone × first-kg / extra-kg slabs + booking extras.
//!
//! `price_paise` on a product remains the flat ₹/kg (or piece) fallback when
//! slab fields are zero. When `first_kg` > 0, chargeable weight uses the slab.

use crate::domain::Money;

/// Default first-weight slab used when the rate has no explicit first kg.
pub const DEFAULT_FIRST_KG: f64 = 1.0;

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
    pub first_kg: f64,
    /// Amount for the first slab (paise).
    pub first_paise: i64,
    /// Amount per additional kg beyond `first_kg` (paise).
    pub extra_per_kg_paise: i64,
    /// Flat ₹/kg fallback when slabs are unset (`product.price`).
    pub flat_per_kg_paise: i64,
}

impl RateSlab {
    pub fn uses_slab(self) -> bool {
        self.first_kg > 0.0 && (self.first_paise > 0 || self.extra_per_kg_paise > 0)
    }

    /// Freight for `chargeable_kg` under this slab (or flat ₹/kg).
    pub fn freight_paise(self, chargeable_kg: f64) -> i64 {
        let kg = chargeable_kg.max(0.0);
        if kg <= 0.0 {
            return 0;
        }
        if self.uses_slab() {
            let first = if self.first_kg > 0.0 {
                self.first_kg
            } else {
                DEFAULT_FIRST_KG
            };
            if kg <= first {
                return self.first_paise.max(0);
            }
            let extra_kg = kg - first;
            let extra = (extra_kg * self.extra_per_kg_paise as f64).round() as i64;
            return self.first_paise.saturating_add(extra.max(0));
        }
        (kg * self.flat_per_kg_paise as f64).round() as i64
    }

    pub fn freight_money(self, chargeable_kg: f64) -> Money {
        Money(self.freight_paise(chargeable_kg))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slab_first_then_extra() {
        let slab = RateSlab {
            first_kg: 1.0,
            first_paise: 50_00,
            extra_per_kg_paise: 20_00,
            flat_per_kg_paise: 0,
        };
        assert_eq!(slab.freight_paise(0.5), 50_00);
        assert_eq!(slab.freight_paise(1.0), 50_00);
        assert_eq!(slab.freight_paise(2.5), 50_00 + 30_00);
    }

    #[test]
    fn flat_per_kg_when_no_slab() {
        let slab = RateSlab {
            first_kg: 0.0,
            first_paise: 0,
            extra_per_kg_paise: 0,
            flat_per_kg_paise: 40_00,
        };
        assert_eq!(slab.freight_paise(2.0), 80_00);
    }
}

//! Courier booking desk maths: chargeable weight and AWB numbers.
//!
//! Industry rule (India carriers): chargeable kg = max(actual, volumetric),
//! then round **up** to the next half-kg slab. Volumetric = L×B×H / 5000 (cm).

use chrono::Local;
use crate::domain::{CartLine, Money, Product, QTY_MILLI_UNIT};
use crate::rates::{BookingExtra, ExtraChargeKind};

/// Volumetric divisor used by most Indian surface/air express carriers.
pub const VOLUMETRIC_DIVISOR: f64 = 5000.0;

/// Compute volumetric weight in kg from dimensions in centimetres.
pub fn volumetric_kg(length_cm: f64, breadth_cm: f64, height_cm: f64) -> f64 {
    if length_cm <= 0.0 || breadth_cm <= 0.0 || height_cm <= 0.0 {
        return 0.0;
    }
    (length_cm * breadth_cm * height_cm) / VOLUMETRIC_DIVISOR
}

/// Chargeable weight: max(actual, volumetric), rounded up to the next 0.5 kg.
pub fn chargeable_kg(actual_kg: f64, length_cm: f64, breadth_cm: f64, height_cm: f64) -> f64 {
    let actual = actual_kg.max(0.0);
    let vol = volumetric_kg(length_cm, breadth_cm, height_cm);
    round_up_half_kg(actual.max(vol))
}

fn round_up_half_kg(kg: f64) -> f64 {
    if kg <= 0.0 {
        return 0.0;
    }
    (kg * 2.0).ceil() / 2.0
}

/// Convert kg to milli-units (1000 = 1 kg) for cart lines.
pub fn kg_to_milli(kg: f64) -> i64 {
    if kg <= 0.0 {
        return 0;
    }
    (kg * 1000.0).round() as i64
}

/// Parse a positive decimal from a field (commas allowed as decimal separator).
pub fn parse_positive(raw: &str) -> Option<f64> {
    let s = raw.trim().replace(',', ".");
    if s.is_empty() {
        return None;
    }
    let v: f64 = s.parse().ok()?;
    if v.is_finite() && v >= 0.0 {
        Some(v)
    } else {
        None
    }
}

/// Human AWB for a new booking: `AWB-YYMMDD-NNNNNN` (unique enough for a counter).
pub fn format_awb(seq: u64) -> String {
    let day = Local::now().format("%y%m%d");
    format!("AWB-{day}-{seq:06}")
}

/// Apply chargeable kg + rate slab to the cart line just added.
pub fn apply_weight_and_slab(line: &mut CartLine, product: &Product, chargeable_kg: f64) {
    if chargeable_kg <= 0.0 || !product.is_weight_unit() {
        return;
    }
    let milli = kg_to_milli(chargeable_kg);
    line.qty_milli = milli;
    line.qty = crate::domain::milli_to_whole_qty(milli).max(1);
    let slab = product.rate_slab();
    if slab.uses_slab() {
        let freight = slab.freight_paise(chargeable_kg);
        line.line_total_override = Some(Money(freight));
        // Keep unit price as catalogue flat for display; override owns the total.
        line.unit_price = product.price;
    } else {
        line.line_total_override = None;
    }
}

/// Build a synthetic cart line for an extra charge (fuel, docket, …).
pub fn extra_as_line(extra: &BookingExtra) -> CartLine {
    CartLine {
        product_id: format!("extra:{}", extra.kind),
        name: extra.name.clone(),
        unit_price: Money(extra.amount_paise.max(0)),
        qty: 1,
        qty_milli: QTY_MILLI_UNIT,
        unit_label: String::new(),
        note: extra.kind.clone(),
        base_unit_paise: extra.amount_paise.max(0),
        applied_modifiers: Vec::new(),
        modifier_page: 0,
        line_total_override: None,
    }
}

pub fn parse_extra_amount(raw: &str) -> Option<i64> {
    let s = raw.trim().replace(',', ".");
    if s.is_empty() {
        return None;
    }
    // Rupees → paise when it looks like a major amount.
    if let Ok(v) = s.parse::<f64>() {
        if v.is_finite() && v >= 0.0 {
            return Some((v * 100.0).round() as i64);
        }
    }
    None
}

pub fn booking_extra(kind: ExtraChargeKind, amount_paise: i64) -> BookingExtra {
    BookingExtra::from_kind(kind, amount_paise)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volumetric_and_chargeable_match_tutorial_example() {
        // 60×25×40 cm, dead 6 kg → vol 12 kg → chargeable 12.
        let vol = volumetric_kg(60.0, 25.0, 40.0);
        assert!((vol - 12.0).abs() < 0.01);
        assert!((chargeable_kg(6.0, 60.0, 25.0, 40.0) - 12.0).abs() < 0.01);
    }

    #[test]
    fn rounds_up_to_half_kg() {
        assert!((chargeable_kg(0.2, 0.0, 0.0, 0.0) - 0.5).abs() < 0.01);
        assert!((chargeable_kg(1.1, 0.0, 0.0, 0.0) - 1.5).abs() < 0.01);
        assert!((chargeable_kg(2.0, 0.0, 0.0, 0.0) - 2.0).abs() < 0.01);
    }

    #[test]
    fn kg_milli_roundtrip() {
        assert_eq!(kg_to_milli(1.5), 1500);
        assert_eq!(kg_to_milli(0.5), 500);
    }
}

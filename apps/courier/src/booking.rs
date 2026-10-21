//! Courier booking desk maths: chargeable weight and AWB numbers.
//!
//! Trade rule (everywhere, constants vary): chargeable kg = max(actual,
//! volumetric), then round **up** to the shop's weight slab. Volumetric =
//! L×B×H (cm³) ÷ the shop's divisor. The defaults (÷5000, 0.5 kg slab) are
//! the Indian domestic convention; both are shop settings, and the desk's
//! entry units (cm, in, ft… / kg, g, lb, oz… or a custom unit with a
//! conversion) are shop choices too — the books always record kg.
//! All weight maths use [`rust_decimal::Decimal`] — no native floats.

use chrono::Local;
use rust_decimal::Decimal;

use crate::domain::{CartLine, Money, Product, QTY_MILLI_UNIT};
use crate::precise::{self, ZERO};
use crate::rates::{BookingExtra, ExtraChargeKind};

/// Default volumetric divisor (Indian surface/air express convention).
pub fn volumetric_divisor() -> Decimal {
    Decimal::from(5000)
}

/// Convert a value typed in the shop's chosen unit into the canonical
/// base (cm or kg) using a resolved factor (standard or custom).
pub fn to_base(value: Decimal, factor_to_base: Decimal) -> Decimal {
    value * factor_to_base
}

/// Convert a canonical base value (kg) back into the shop's unit for display.
pub fn from_base(value: Decimal, factor_to_base: Decimal) -> Decimal {
    if factor_to_base <= ZERO {
        return value;
    }
    value / factor_to_base
}

/// Compute volumetric weight in kg from dimensions in centimetres.
pub fn volumetric_kg(
    length_cm: Decimal,
    breadth_cm: Decimal,
    height_cm: Decimal,
    divisor: Decimal,
) -> Decimal {
    if length_cm <= ZERO || breadth_cm <= ZERO || height_cm <= ZERO {
        return ZERO;
    }
    let divisor = if divisor > ZERO {
        divisor
    } else {
        volumetric_divisor()
    };
    (length_cm * breadth_cm * height_cm) / divisor
}

/// Chargeable weight: max(actual, volumetric), rounded up to the slab.
pub fn chargeable_kg(
    actual_kg: Decimal,
    length_cm: Decimal,
    breadth_cm: Decimal,
    height_cm: Decimal,
    divisor: Decimal,
    slab_kg: Decimal,
) -> Decimal {
    let actual = if actual_kg > ZERO { actual_kg } else { ZERO };
    let vol = volumetric_kg(length_cm, breadth_cm, height_cm, divisor);
    let raw = if actual > vol { actual } else { vol };
    round_up_slab(raw, slab_kg)
}

fn round_up_slab(kg: Decimal, slab_kg: Decimal) -> Decimal {
    if kg <= ZERO {
        return ZERO;
    }
    let slab = if slab_kg > ZERO {
        slab_kg
    } else {
        Decimal::new(5, 1)
    };
    (kg / slab).ceil() * slab
}

/// Convert kg to milli-units (1000 = 1 kg) for cart lines.
pub fn kg_to_milli(kg: Decimal) -> Result<i64, String> {
    if kg <= ZERO {
        return Ok(0);
    }
    precise::to_i64(kg * Decimal::from(1000))
}

/// Parse a positive decimal from a field (commas allowed as decimal separator).
pub fn parse_positive(raw: &str) -> Option<Decimal> {
    precise::parse_non_negative(raw)
}

/// Human AWB for a new booking: `AWB-YYMMDD-NNNNNN` (unique enough for a counter).
pub fn format_awb(seq: u64) -> String {
    let day = Local::now().format("%y%m%d");
    format!("AWB-{day}-{seq:06}")
}

/// Apply chargeable kg + rate slab to the cart line just added.
pub fn apply_weight_and_slab(
    line: &mut CartLine,
    product: &Product,
    chargeable: Decimal,
) -> Result<(), String> {
    if chargeable <= ZERO || !product.is_weight_unit() {
        return Ok(());
    }
    let milli = kg_to_milli(chargeable)?;
    line.qty_milli = milli;
    line.qty = crate::domain::milli_to_whole_qty(milli).max(1);
    let slab = product.rate_slab();
    if slab.uses_slab() {
        let freight = slab.freight_paise(chargeable)?;
        line.line_total_override = Some(Money(freight));
        line.unit_price = product.price;
    } else {
        line.line_total_override = None;
    }
    Ok(())
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

/// Parse a major-unit amount into minor units for the shop currency.
pub fn parse_extra_amount(raw: &str, currency_code: &str) -> Option<i64> {
    crate::locale::parse_major_input(raw, currency_code)
        .ok()
        .filter(|&p| p >= 0)
}

pub fn booking_extra(kind: ExtraChargeKind, amount_paise: i64) -> BookingExtra {
    BookingExtra::from_kind(kind, amount_paise)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn volumetric_and_chargeable_match_tutorial_example() {
        // 60×25×40 cm, dead 6 kg → vol 12 kg → chargeable 12.
        let vol = volumetric_kg(d("60"), d("25"), d("40"), d("5000"));
        assert_eq!(vol, d("12"));
        assert_eq!(
            chargeable_kg(d("6"), d("60"), d("25"), d("40"), d("5000"), d("0.5")),
            d("12")
        );
    }

    #[test]
    fn rounds_up_to_half_kg() {
        assert_eq!(chargeable_kg(d("0.2"), ZERO, ZERO, ZERO, d("5000"), d("0.5")), d("0.5"));
        assert_eq!(chargeable_kg(d("1.1"), ZERO, ZERO, ZERO, d("5000"), d("0.5")), d("1.5"));
        assert_eq!(chargeable_kg(d("2.0"), ZERO, ZERO, ZERO, d("5000"), d("0.5")), d("2"));
    }

    #[test]
    fn shop_divisor_and_slab_are_honoured() {
        // ÷6000 (international air style): 60×25×40 → vol 10 kg.
        assert_eq!(volumetric_kg(d("60"), d("25"), d("40"), d("6000")), d("10"));
        // 1 kg slab: 1.1 kg actual → chargeable 2.
        assert_eq!(chargeable_kg(d("1.1"), ZERO, ZERO, ZERO, d("5000"), d("1")), d("2"));
    }

    #[test]
    fn inch_and_pound_entry_converts_to_canonical() {
        // 10 in = 25.4 cm; 10×10×10 in → 16,387.064 cm³ ÷ 5000 ≈ 3.277 kg.
        let in_cm = crate::domain::standard_factor_to_cm("in").unwrap();
        let side = to_base(d("10"), in_cm);
        assert_eq!(side, d("25.4"));
        let vol = volumetric_kg(side, side, side, d("5000"));
        assert!(vol > d("3.27") && vol < d("3.28"), "vol {vol}");
        // 10 lb = 4.5359237 kg → chargeable rounds to 5.0 at 0.5 slab.
        let lb_kg = crate::domain::standard_factor_to_kg("lb").unwrap();
        let actual = to_base(d("10"), lb_kg);
        assert_eq!(actual, d("4.5359237"));
        assert_eq!(chargeable_kg(actual, ZERO, ZERO, ZERO, d("5000"), d("0.5")), d("5"));
        // Display round-trip: 5 kg back in lb ≈ 11.02.
        let back = from_base(d("5"), lb_kg);
        assert!(back > d("11.02") && back < d("11.03"), "back {back}");
    }

    #[test]
    fn kg_milli_roundtrip() {
        assert_eq!(kg_to_milli(d("1.5")).unwrap(), 1500);
        assert_eq!(kg_to_milli(d("0.5")).unwrap(), 500);
    }

    #[test]
    fn parse_extra_uses_currency_decimals() {
        assert_eq!(parse_extra_amount("13,5", "EUR"), Some(1_350));
        assert_eq!(parse_extra_amount("1,234", "KWD"), Some(1_234));
        assert_eq!(parse_extra_amount("12.3456", "INR"), None);
    }
}

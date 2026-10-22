//! Exact decimal arithmetic for money, weights, and rates.
//!
//! Uses [`rust_decimal::Decimal`] (integer digits internally — no IEEE float).
//! Native `f32`/`f64` stay fine for layout chrome only.

use std::str::FromStr;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rusqlite::types::ValueRef;
use rusqlite::Row;

pub use rust_decimal::Decimal as Dec;

/// Zero constant used across money math.
pub const ZERO: Decimal = Decimal::ZERO;

/// Convert a finished decimal to `i64` (after rounding to integer).
///
/// Rejects values outside `i64` instead of wrapping or saturating.
pub fn to_i64(value: Decimal) -> Result<i64, String> {
    value
        .round()
        .to_i64()
        .ok_or_else(|| format!("Amount {value} does not fit in a 64-bit integer (overflow)"))
}

/// Convert an already-integral decimal to `i64`.
pub fn to_i64_exact(value: Decimal) -> Result<i64, String> {
    if value != value.trunc() {
        return Err(format!("Expected a whole number, got {value}"));
    }
    value
        .to_i64()
        .ok_or_else(|| format!("Amount {value} does not fit in a 64-bit integer (overflow)"))
}

/// Scale a major-unit decimal into minor units for a currency.
///
/// `major` must not carry non-zero digits beyond `currency_decimals`.
pub fn major_to_minor(major: Decimal, currency_decimals: u32) -> Result<i64, String> {
    let factor = Decimal::from(10u64.pow(currency_decimals));
    let scaled = major * factor;
    // Any fractional remainder after scaling → too many decimal places.
    if scaled != scaled.trunc() {
        // Allow tiny residue from normalize; reject real extra digits.
        let rounded = scaled.round_dp(0);
        if (scaled - rounded).abs() > Decimal::new(1, 12) {
            return Err("Too many decimal places".into());
        }
        return to_i64_exact(rounded);
    }
    to_i64_exact(scaled)
}

/// Parse a non-negative decimal from a field (`.` or `,` as decimal mark).
pub fn parse_non_negative(raw: &str) -> Option<Decimal> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\'')
        .collect();
    let last_dot = cleaned.rfind('.');
    let last_comma = cleaned.rfind(',');
    let normalized = match (last_dot, last_comma) {
        (Some(d), Some(c)) if d > c => cleaned.replace(',', ""),
        (Some(d), Some(c)) if c > d => cleaned.replace('.', "").replace(',', "."),
        (None, Some(_)) => cleaned.replace(',', "."),
        _ => cleaned,
    };
    let v = Decimal::from_str(&normalized).ok()?;
    if v.is_sign_negative() {
        None
    } else {
        Some(v)
    }
}

/// Persist a decimal as a plain string (exact).
pub fn dec_to_sql(value: Decimal) -> String {
    value.normalize().to_string()
}

/// Read a decimal from SQLite (TEXT preferred; REAL/INTEGER accepted for old rows).
pub fn dec_from_sql(row: &Row<'_>, idx: usize) -> Result<Decimal, rusqlite::Error> {
    match row.get_ref(idx)? {
        ValueRef::Null => Ok(ZERO),
        ValueRef::Integer(i) => Ok(Decimal::from(i)),
        ValueRef::Real(f) => Decimal::from_f64_retain(f).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                idx,
                rusqlite::types::Type::Real,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "non-finite REAL weight",
                )),
            )
        }),
        ValueRef::Text(bytes) => {
            let s = std::str::from_utf8(bytes).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    idx,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            if s.trim().is_empty() {
                return Ok(ZERO);
            }
            Decimal::from_str(s.trim()).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    idx,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        }
        ValueRef::Blob(_) => Err(rusqlite::Error::InvalidColumnType(
            idx,
            "decimal".into(),
            rusqlite::types::Type::Blob,
        )),
    }
}

/// Format a kg value for receipts / labels (trim trailing zeros).
pub fn format_kg(value: Decimal) -> String {
    let s = value.normalize().to_string();
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// `a * b` with overflow check when converting a money-like product to i64.
pub fn mul_to_i64(a: Decimal, b: Decimal) -> Result<i64, String> {
    to_i64(a * b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_i64_rejects_overflow() {
        let huge = Decimal::from_str("999999999999999999999").unwrap();
        assert!(to_i64(huge).is_err());
    }

    #[test]
    fn major_to_minor_honours_decimals() {
        let d = Decimal::from_str("13.5").unwrap();
        assert_eq!(major_to_minor(d, 2).unwrap(), 1350);
        assert_eq!(major_to_minor(d, 3).unwrap(), 13500);
    }

    #[test]
    fn major_to_minor_rejects_extra_fraction() {
        let d = Decimal::from_str("12.3456").unwrap();
        assert!(major_to_minor(d, 2).is_err());
    }

    #[test]
    fn parse_non_negative_comma() {
        assert_eq!(
            parse_non_negative("1,5").unwrap(),
            Decimal::from_str("1.5").unwrap()
        );
        assert!(parse_non_negative("-1").is_none());
    }
}

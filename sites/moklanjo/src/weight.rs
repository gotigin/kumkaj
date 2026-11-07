//! Chargeable weight for the "Weigh it right" chapter.
//!
//! Copied from Moklanjo's booking desk rules (apps/moklanjo/src/booking.rs —
//! copy, not import; a site never depends on an app crate):
//! volumetric = L × B × H (cm) ÷ divisor; chargeable = the higher of actual
//! and volumetric, rounded up to the slab. Defaults: ÷5000 domestic, ÷6000
//! international air, 0.5 kg slab — all of them the shop's to change in the app.
//!
//! Integer arithmetic throughout (grams, scaled by the divisor) so the page
//! never shows a float artefact like 1.7999 kg.

pub const DOMESTIC_DIVISOR: u64 = 5000;
pub const INTERNATIONAL_DIVISOR: u64 = 6000;
pub const SLAB_G: u64 = 500;

/// One parcel on the desk scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parcel {
    pub length_cm: u64,
    pub breadth_cm: u64,
    pub height_cm: u64,
    pub actual_g: u64,
    pub divisor: u64,
}

impl Parcel {
    /// The worked example in docs (MOKLANJO_WEBSITE.md §3 chapter 2).
    pub const EXAMPLE: Parcel = Parcel {
        length_cm: 30,
        breadth_cm: 20,
        height_cm: 15,
        actual_g: 1200,
        divisor: DOMESTIC_DIVISOR,
    };

    fn cm3(&self) -> u64 {
        self.length_cm * self.breadth_cm * self.height_cm
    }

    /// Volumetric weight in hundredths of a kg, rounded half up (display).
    pub fn volumetric_centi_kg(&self) -> u64 {
        let divisor = self.divisor.max(1);
        (self.cm3() * 100 * 2 + divisor) / (2 * divisor)
    }

    /// Chargeable weight in grams: max(actual, volumetric) rounded up to the
    /// slab. Compared exactly (both sides scaled by the divisor).
    pub fn chargeable_g(&self) -> u64 {
        chargeable_g(
            self.actual_g,
            self.length_cm,
            self.breadth_cm,
            self.height_cm,
            self.divisor,
            SLAB_G,
        )
    }

    /// True when the box's size, not its weight, sets the charge.
    pub fn volumetric_wins(&self) -> bool {
        self.cm3() * 1000 > self.actual_g * self.divisor.max(1)
    }
}

/// Same signature shape as the app's `chargeable_kg`, in grams.
pub fn chargeable_g(
    actual_g: u64,
    length_cm: u64,
    breadth_cm: u64,
    height_cm: u64,
    divisor: u64,
    slab_g: u64,
) -> u64 {
    let divisor = divisor.max(1);
    let slab_g = if slab_g == 0 { SLAB_G } else { slab_g };
    let volumetric_scaled = length_cm * breadth_cm * height_cm * 1000;
    let actual_scaled = actual_g * divisor;
    let raw = volumetric_scaled.max(actual_scaled);
    if raw == 0 {
        return 0;
    }
    raw.div_ceil(slab_g * divisor) * slab_g
}

/// 180 → "1.8", 1200 → "12", 328 → "3.28".
pub fn fmt_centi_kg(centi: u64) -> String {
    let whole = centi / 100;
    let frac = centi % 100;
    if frac == 0 {
        format!("{whole}")
    } else if frac % 10 == 0 {
        format!("{whole}.{}", frac / 10)
    } else {
        format!("{whole}.{frac:02}")
    }
}

/// Grams with one decimal of a kg, as the desk shows a chargeable weight:
/// 2000 → "2.0", 12000 → "12.0", 1500 → "1.5".
pub fn fmt_kg_1(g: u64) -> String {
    let tenths = (g + 50) / 100;
    format!("{}.{}", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parcel(l: u64, b: u64, h: u64, actual_g: u64, divisor: u64) -> Parcel {
        Parcel {
            length_cm: l,
            breadth_cm: b,
            height_cm: h,
            actual_g,
            divisor,
        }
    }

    // Pinned to the app's own tests (apps/moklanjo/src/booking.rs).
    #[test]
    fn tutorial_example_matches_the_app() {
        // 60×25×40 cm, dead 6 kg → vol 12 kg → chargeable 12.
        let p = parcel(60, 25, 40, 6000, DOMESTIC_DIVISOR);
        assert_eq!(p.volumetric_centi_kg(), 1200);
        assert_eq!(p.chargeable_g(), 12_000);
        assert!(p.volumetric_wins());
    }

    #[test]
    fn international_divisor_matches_the_app() {
        // ÷6000: 60×25×40 → vol 10 kg.
        let p = parcel(60, 25, 40, 6000, INTERNATIONAL_DIVISOR);
        assert_eq!(p.volumetric_centi_kg(), 1000);
        assert_eq!(p.chargeable_g(), 10_000);
    }

    #[test]
    fn rounds_up_to_half_kg_like_the_app() {
        assert_eq!(chargeable_g(200, 0, 0, 0, 5000, 500), 500);
        assert_eq!(chargeable_g(1100, 0, 0, 0, 5000, 500), 1500);
        assert_eq!(chargeable_g(2000, 0, 0, 0, 5000, 500), 2000);
        // 1 kg slab: 1.1 kg actual → 2.
        assert_eq!(chargeable_g(1100, 0, 0, 0, 5000, 1000), 2000);
    }

    #[test]
    fn website_example_is_one_point_eight_to_two() {
        let p = Parcel::EXAMPLE;
        assert_eq!(fmt_centi_kg(p.volumetric_centi_kg()), "1.8");
        assert_eq!(fmt_kg_1(p.chargeable_g()), "2.0");
        assert!(p.volumetric_wins());
    }

    #[test]
    fn heavy_small_box_charges_actual_weight() {
        let p = parcel(10, 10, 10, 3200, DOMESTIC_DIVISOR);
        assert!(!p.volumetric_wins());
        assert_eq!(p.chargeable_g(), 3500);
    }

    #[test]
    fn formats_trim_like_the_desk() {
        assert_eq!(fmt_centi_kg(180), "1.8");
        assert_eq!(fmt_centi_kg(1200), "12");
        assert_eq!(fmt_centi_kg(328), "3.28");
        assert_eq!(fmt_centi_kg(5), "0.05");
        assert_eq!(fmt_kg_1(500), "0.5");
        assert_eq!(fmt_kg_1(12_000), "12.0");
    }
}

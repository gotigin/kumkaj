//! Moklanjo shop start templates (same courier product, different shop face).
//!
//! Labels and lane names live here, in Moklanjo's own code. There is no
//! Kumkaj core / shared store: if another Kumkaj product needs the same
//! books concepts, it copies this code and diverges (see vision §1).
//!
//! Default [`MOKLANJO`] ships empty (owner adds parties). Authored packs such
//! as [`GLOBAL_PKG`] seed named carriers/consignors when applied — configure,
//! do not fork the binary.

use crate::domain::Money;

/// Reserved for optional future seed rows (currently unused on the default).
#[derive(Clone, Copy, Debug)]
pub struct SeedProduct {
    pub name: &'static str,
    pub price_major: i64,
    pub unit_label: &'static str,
}

/// Named party seeded when an authored app template is applied on an empty
/// (or missing-name) Parties list.
#[derive(Clone, Copy, Debug)]
pub struct SeedParty {
    pub name: &'static str,
    pub kind_label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct EntityTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub how_to: &'static str,
    pub label_overrides: &'static [(&'static str, &'static str)],
    pub qty_unit_hint: &'static str,
    pub lane_declared_label: &'static str,
    pub lane_owner_label: &'static str,
    pub seed_products: &'static [SeedProduct],
    pub seed_parties: &'static [SeedParty],
}

/// Default empty agency face for this binary.
pub const MOKLANJO: EntityTemplate = EntityTemplate {
    id: "moklanjo_agency",
    title: "Moklanjo",
    blurb: "Courier-agency books + booking desk: consignors, carriers, day book, AWB.",
    how_to: "Booking = walk-in parcels (weigh → rate → cash/account → AWB). Day book = today. Pay out = carrier bills & expenses. Rates, Parties, Reports, Team are tabs.",
    label_overrides: &[],
    qty_unit_hint: "kg",
    lane_declared_label: "Account booking",
    lane_owner_label: "Cash booking",
    seed_products: &[],
    seed_parties: &[],
};

/// Global Pkg App Template — carriers and consignors from the client desk list.
pub const GLOBAL_PKG: EntityTemplate = EntityTemplate {
    id: "global_pkg",
    title: "Global Pkg App Template",
    blurb: "Agency desk with the Global Pkg carriers and consignors pre-listed. Rates stay empty — add your sell cards after apply.",
    how_to: "Settings → Product → Global Pkg App Template. Parties get carriers (Delhivery, DTDC, …) and consignors (Sumit International, …). Then set rates and book as usual.",
    label_overrides: &[],
    qty_unit_hint: "kg",
    lane_declared_label: "Account booking",
    lane_owner_label: "Cash booking",
    seed_products: &[],
    seed_parties: &[
        SeedParty {
            name: "Delhivery",
            kind_label: "Carrier",
        },
        SeedParty {
            name: "DTDC",
            kind_label: "Carrier",
        },
        SeedParty {
            name: "Mahaveer",
            kind_label: "Carrier",
        },
        SeedParty {
            name: "St",
            kind_label: "Carrier",
        },
        SeedParty {
            name: "Professional",
            kind_label: "Carrier",
        },
        SeedParty {
            name: "Sumit International",
            kind_label: "Consignor",
        },
        SeedParty {
            name: "Bangles King",
            kind_label: "Consignor",
        },
        SeedParty {
            name: "Love India",
            kind_label: "Consignor",
        },
        SeedParty {
            name: "Balaji Enterprise",
            kind_label: "Consignor",
        },
        SeedParty {
            name: "Alisha Creation",
            kind_label: "Consignor",
        },
    ],
};

pub const TEMPLATES: &[EntityTemplate] = &[MOKLANJO, GLOBAL_PKG];

pub fn find(id: &str) -> Option<&'static EntityTemplate> {
    let id = id.trim();
    if id.is_empty() || id == MOKLANJO.id || id == "moklanjo" || id == "courier" {
        return Some(&MOKLANJO);
    }
    if id == GLOBAL_PKG.id || id == "global_prg" || id == "global-pkg" {
        return Some(&GLOBAL_PKG);
    }
    TEMPLATES.iter().find(|t| t.id == id)
}

pub fn resolve(id: &str) -> &'static EntityTemplate {
    find(id).unwrap_or(&MOKLANJO)
}

pub fn label_override(template: &EntityTemplate, key: &str) -> Option<&'static str> {
    template
        .label_overrides
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

pub fn seed_product_price(seed: &SeedProduct) -> Money {
    Money::from_rupees(seed.price_major)
}

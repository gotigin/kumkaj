//! Kumkaj Courier is one concrete product — no multi-trade template picker.
//!
//! Labels and lane names live here, in Courier's own code. There is no
//! Kumkaj core / shared store: if another Kumkaj product needs the same
//! books concepts, it copies this code and diverges (see vision §1).

use crate::domain::Money;

/// Reserved for optional future seed rows (currently unused — templates ship empty).
#[derive(Clone, Copy, Debug)]
pub struct SeedProduct {
    pub name: &'static str,
    pub price_major: i64,
    pub unit_label: &'static str,
}

/// Reserved for optional future seed parties (currently unused).
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

/// Sole product face for this binary.
pub const COURIER: EntityTemplate = EntityTemplate {
    id: "courier_agency",
    title: "Kumkaj Courier",
    blurb: "Courier books + booking desk: consignors, carriers, day book, AWB.",
    how_to: "Booking = walk-in parcels (weigh → rate → cash/account → AWB). Day book = today. Pay out = carrier bills & expenses. More → Rates, Parties, Bank book & P&L, Team.",
    label_overrides: &[],
    qty_unit_hint: "kg",
    lane_declared_label: "Account booking",
    lane_owner_label: "Cash booking",
    seed_products: &[],
    seed_parties: &[],
};

pub const TEMPLATES: &[EntityTemplate] = &[COURIER];

pub fn find(id: &str) -> Option<&'static EntityTemplate> {
    let id = id.trim();
    if id.is_empty() || id == COURIER.id || id == "courier" {
        Some(&COURIER)
    } else {
        // Pre-release: any old template id collapses to this product.
        Some(&COURIER)
    }
}

pub fn resolve(_id: &str) -> &'static EntityTemplate {
    &COURIER
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

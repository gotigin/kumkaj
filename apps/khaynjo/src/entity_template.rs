//! Khaynjo is one concrete product — no multi-trade template picker.
//!
//! Khaynjo (khaynjo.com) is the Kumkaj restaurant operating system:
//! floor/tables, ordering, KOT/kitchen, billing, and Khaynjo's own books
//! underneath. Labels and lane names live here, in Khaynjo's own code.
//! There is no Kumkaj core / shared store: this code was copied from
//! Courier and now diverges freely (see vision §1).
//!
//! Locked-template pattern: one product face per binary, templates ship
//! empty (the owner adds their own menu/parties).

use crate::domain::{Money, VenueMode};

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
    pub venue_mode: VenueMode,
    pub qty_unit_hint: &'static str,
    pub lane_declared_label: &'static str,
    pub lane_owner_label: &'static str,
    pub seed_products: &'static [SeedProduct],
    pub seed_parties: &'static [SeedParty],
}

/// Sole product face for this binary.
pub const KHAYNJO: EntityTemplate = EntityTemplate {
    id: "khaynjo_restaurant",
    title: "Khaynjo",
    blurb: "Restaurant operating system: tables, ordering, KOT, billing, books.",
    how_to: "Sell = counter billing (item → qty → pay/credit → receipt). Floor = tables & running orders; send KOT to the kitchen, add rounds, then bill. Books = day book, parties, stock, expenses & reports under More.",
    label_overrides: &[],
    venue_mode: VenueMode::Restaurant,
    qty_unit_hint: "",
    lane_declared_label: "Tax invoice",
    lane_owner_label: "Cash memo",
    seed_products: &[],
    seed_parties: &[],
};

pub const TEMPLATES: &[EntityTemplate] = &[KHAYNJO];

pub fn find(id: &str) -> Option<&'static EntityTemplate> {
    let id = id.trim();
    if id.is_empty() || id == KHAYNJO.id || id == "khaynjo" {
        Some(&KHAYNJO)
    } else {
        // Pre-release: any old template id collapses to this product.
        Some(&KHAYNJO)
    }
}

pub fn resolve(_id: &str) -> &'static EntityTemplate {
    &KHAYNJO
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

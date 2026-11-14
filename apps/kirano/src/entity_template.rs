//! Kirano is one concrete product — no multi-trade template picker.
//!
//! Kirano (kirano.shop) is the Kumkaj grocery / kirana operating system:
//! counter sell, stock, khata, supplier buys, and Kirano's own books
//! underneath. Labels live here, in Kirano's own code. There is no
//! Kumkaj core / shared store: this crate owns all grocery product
//! code and diverges freely (vision §1).
//!
//! Locked-template pattern: one product face per binary, templates ship
//! empty (the owner adds their own catalogue/parties).

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
pub const KIRANO: EntityTemplate = EntityTemplate {
    id: "kirano_kirana",
    title: "Kirano",
    blurb: "Grocery / kirana operating system: counter sell, stock, khata, books.",
    how_to: "Sell = counter billing (item → qty → pay/credit → receipt). Buy = restock from suppliers. Books = day book, parties, stock, expenses & reports. No tables or kitchen — this is a shop counter.",
    // Vertical words live in locale::en (this app's own English), so
    // every language translates them; an overlay here would show its
    // English in every language (template_overrides_stay_empty).
    label_overrides: &[],
    qty_unit_hint: "pc",
    lane_declared_label: "Tax invoice",
    lane_owner_label: "Cash memo",
    seed_products: &[],
    seed_parties: &[],
};

pub const TEMPLATES: &[EntityTemplate] = &[KIRANO];

pub fn find(id: &str) -> Option<&'static EntityTemplate> {
    let id = id.trim();
    if id.is_empty() || id == KIRANO.id || id == "kirano" || id == "kirano_restaurant" {
        Some(&KIRANO)
    } else {
        // Pre-release: any old template id collapses to this product.
        Some(&KIRANO)
    }
}

pub fn resolve(_id: &str) -> &'static EntityTemplate {
    &KIRANO
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

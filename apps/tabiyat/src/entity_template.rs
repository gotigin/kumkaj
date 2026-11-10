//! Tabiyat is one concrete product — no multi-trade template picker.
//!
//! Tabiyat (tabiyat.shop) is the Kumkaj pharmacy operating system:
//! counter sell, medicines, batch/expiry bookkeeping, khata, and
//! Tabiyat's own books. Copied from Kirano; diverges freely (vision §1).

use crate::domain::{Money, VenueMode};

#[derive(Clone, Copy, Debug)]
pub struct SeedProduct {
    pub name: &'static str,
    pub price_major: i64,
    pub unit_label: &'static str,
}

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

pub const TABIYAT: EntityTemplate = EntityTemplate {
    id: "tabiyat_pharmacy",
    title: "Tabiyat",
    blurb: "Pharmacy operating system: counter sell, medicines, batch/expiry, books.",
    how_to: "Sell = medicine billing (search name, salt or barcode → qty → pay/credit). Buy = restock with batch & expiry. Schedule H / Rx flags are your notes — not compliance. Books = day book, parties, stock & reports.",
    // Vertical words live in locale::en (this app's own English), so
    // every language translates them; an overlay here would show its
    // English in every language (template_overrides_stay_empty).
    label_overrides: &[],
    venue_mode: VenueMode::Counter,
    qty_unit_hint: "pc",
    lane_declared_label: "Tax invoice",
    lane_owner_label: "Cash memo",
    seed_products: &[],
    seed_parties: &[],
};

pub const TEMPLATES: &[EntityTemplate] = &[TABIYAT];

pub fn find(id: &str) -> Option<&'static EntityTemplate> {
    let id = id.trim();
    if id.is_empty() || id == TABIYAT.id || id == "tabiyat" || id == "tabiyat_pharmacy" {
        Some(&TABIYAT)
    } else {
        Some(&TABIYAT)
    }
}

pub fn resolve(_id: &str) -> &'static EntityTemplate {
    &TABIYAT
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

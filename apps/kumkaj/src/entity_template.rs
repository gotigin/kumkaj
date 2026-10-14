//! Start templates: concrete feel without vertical forks.
//!
//! A template is labels + feature defaults applied onto the same generic store
//! (items, parties/customers, ledgers, books). No sample catalogue or parties
//! are pre-loaded — the owner adds their own.

use crate::domain::{Money, VenueMode};

/// Reserved for optional future seed rows (currently unused — templates ship empty).
#[derive(Clone, Copy, Debug)]
pub struct SeedProduct {
    pub name: &'static str,
    /// Major currency units (rupees, dollars, …).
    pub price_major: i64,
    /// Empty = piece count; e.g. `"kg"` for rate × weight.
    pub unit_label: &'static str,
}

/// Reserved for optional future seed parties (currently unused — templates ship empty).
#[derive(Clone, Copy, Debug)]
pub struct SeedParty {
    pub name: &'static str,
    /// Freeform kind: `"Carrier"`, `"Retail"`, … — never a schema enum fork.
    pub kind_label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct EntityTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    /// Locale key → override string (English; other languages fall back).
    pub label_overrides: &'static [(&'static str, &'static str)],
    pub venue_mode: VenueMode,
    pub qty_unit_hint: &'static str,
    /// Owner-facing name for the shareable / account (GST) bill type.
    pub lane_declared_label: &'static str,
    /// Owner-facing name for the cash-memo / cash-booking bill type.
    pub lane_owner_label: &'static str,
    pub seed_products: &'static [SeedProduct],
    pub seed_parties: &'static [SeedParty],
}

pub const TEMPLATES: &[EntityTemplate] = &[
    EntityTemplate {
        // Stable id; UI label is "None" (bare minimal — enable features later).
        id: "custom",
        title: "None",
        blurb: "Bare minimal — add features later as you need them.",
        label_overrides: &[],
        venue_mode: VenueMode::Counter,
        qty_unit_hint: "",
        // Tax invoice path vs cash memo — same sales table, a bill-type tag.
        lane_declared_label: "Tax invoice",
        lane_owner_label: "Cash memo",
        seed_products: &[],
        seed_parties: &[],
    },
    EntityTemplate {
        id: "counter_shop",
        title: "Counter shop",
        blurb: "Retail / counter trade — fast sell, credit, simple day books.",
        label_overrides: &[],
        venue_mode: VenueMode::Counter,
        qty_unit_hint: "",
        lane_declared_label: "Tax invoice",
        lane_owner_label: "Cash memo",
        seed_products: &[],
        seed_parties: &[],
    },
    EntityTemplate {
        id: "courier_agency",
        title: "Courier agency",
        blurb: "Parties, kg rates, money in and out, day books, account vs cash booking.",
        label_overrides: &[
            ("credit", "Party credit"),
            ("credit_customers", "Parties"),
            ("customer_name_ph", "Party name"),
            ("new_customer_ph", "New party name"),
            ("owed", "outstanding"),
            ("items", "Rates & services"),
            ("empty_catalog", "No rates yet. Open Items to add freight or service rates."),
            ("qty", "Wt/Qty"),
            ("unit_at", "Rate @"),
        ],
        venue_mode: VenueMode::Counter,
        qty_unit_hint: "kg",
        // Courier-industry wording (Bellboy / GoInvoicing / Arna): not "white/black books".
        lane_declared_label: "Account booking",
        lane_owner_label: "Cash booking",
        seed_products: &[],
        seed_parties: &[],
    },
    EntityTemplate {
        id: "food_service",
        title: "Food service",
        blurb: "Tables & kitchen available — café or restaurant.",
        label_overrides: &[],
        venue_mode: VenueMode::Restaurant,
        qty_unit_hint: "",
        lane_declared_label: "Tax invoice",
        lane_owner_label: "Cash memo",
        seed_products: &[],
        seed_parties: &[],
    },
    EntityTemplate {
        id: "personal_books",
        title: "Personal books",
        blurb: "Household or personal money in and out.",
        label_overrides: &[
            ("credit_customers", "People"),
            ("credit", "On account"),
            ("customer_name_ph", "Person name"),
            ("new_customer_ph", "New person name"),
            ("items", "Categories"),
            ("sales", "Entries"),
        ],
        venue_mode: VenueMode::Counter,
        qty_unit_hint: "",
        lane_declared_label: "Tax invoice",
        lane_owner_label: "Cash memo",
        seed_products: &[],
        seed_parties: &[],
    },
];

pub fn find(id: &str) -> Option<&'static EntityTemplate> {
    TEMPLATES.iter().find(|t| t.id == id.trim())
}

pub fn resolve(id: &str) -> &'static EntityTemplate {
    find(id).unwrap_or(&TEMPLATES[0])
}

pub fn label_override(template: &EntityTemplate, key: &str) -> Option<&'static str> {
    template
        .label_overrides
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// Build a [`crate::domain::Product`] from a seed row (caller assigns id/branch).
pub fn seed_product_price(seed: &SeedProduct) -> Money {
    Money::from_rupees(seed.price_major)
}

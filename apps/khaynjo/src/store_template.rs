//! App templates: a configured shop as a pack, and a pack back into a shop.
//!
//! A pack is the folder `catalogue/private-src/<client>/` holds and
//! `publish-catalogue.sh` ships: `template.json` (the menu),
//! `shop_defaults.json` (everything else about the shop) and `images/`
//! (menu photos). [`Store::export_app_template`] writes one from this shop —
//! the client sets the shop up the way they want, we publish what they
//! exported. [`Store::apply_app_template_pack`] applies one on a new device.
//! Both directions use the same structs, and the round-trip test (export →
//! apply on a fresh shop → export again → identical files) keeps them equal:
//! a field the export writes but the import ignores fails that test.
//!
//! What never travels: customers and suppliers, sales, money, stock counts,
//! the audit log, PINs and passwords, pairing and devices, this device's
//! printers and print routes, and the device language. Stations, tax rates,
//! areas, items and staff are named in the pack, never by id — ids differ
//! on every device.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::{ops, Store};
use crate::domain::{
    new_id, Money, Product, StaffFeatures, StaffMember, StaffRole, STOCK_UNTRACKED,
};
use crate::media;
use crate::menu_templates::{
    self, MenuTemplate, TemplateCategory, TemplateItem, TemplateRecipeLine, PACK_APP, PACK_FORMAT,
};
use crate::print_share::{BillSlipFormat, KotSlipFormat, PrintFormat};

/// Books settings a pack carries, copied as stored. Every setting the
/// Server shares with its devices ([`super::SHOP_PREF_KEYS`]) plus the shop's
/// country and tax wording, document names and the Floor's look. Nothing
/// else in `meta` is ever read from a pack: device role, pairing, setup
/// flags and ids stay this device's own.
pub const TEMPLATE_PREF_EXTRA_KEYS: &[&str] = &[
    "country_code",
    "tax_pricing",
    "tax_label",
    "business_tax_id",
    "business_id_label",
    "digital_pay_label",
    "credit_label",
    "date_order",
    "time_format",
    "decimal_sep",
    "thousands_sep",
    "service_charge_bps",
    "org_name",
    "lane_declared_label",
    "lane_owner_label",
    "standard_print_em",
    "floor_view",
    "ui_floor_area_order",
    "ui_floor_columns",
    "ui_floor_columns_by_area",
    "ui_floor_zoom",
    "ui_floor_zoom_auto",
];

/// Every `prefs` key a pack may set ([`super::SHOP_PREF_KEYS`] first).
pub fn template_pref_keys() -> impl Iterator<Item = &'static str> {
    super::SHOP_PREF_KEYS
        .iter()
        .chain(TEMPLATE_PREF_EXTRA_KEYS.iter())
        .copied()
}

fn is_template_pref_key(key: &str) -> bool {
    template_pref_keys().any(|k| k == key)
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}
fn is_false(v: &bool) -> bool {
    !*v
}
fn is_true(v: &bool) -> bool {
    *v
}
fn yes() -> bool {
    true
}
fn zero_f64(v: &f64) -> bool {
    *v == 0.0
}
fn default_prep_mins() -> i64 {
    15
}

/// `shop_defaults.json`: everything about the shop except the menu.
///
/// The fields up to `logo_base64` are the hand-written (format 0) ones the
/// Rajdhani pack uses; the rest are written by "Export template". Every
/// field is optional — a section left out leaves the new shop's own.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShopDefaultsFile {
    /// [`PACK_FORMAT`] of an exported pack; 0 = hand-written.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub format: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub app: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub app_version: String,
    /// When the pack was exported (RFC 3339, local time).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub exported_at: String,
    /// The shop's name (venue title), e.g. "Rajdhani". Shown on the next
    /// screen (Set up Khaynjo), where the Owner can change it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop_name: Option<String>,
    /// The shop's logo — a WebP / PNG / JPEG as base64. Brand pictures ride
    /// inside this file; menu photos are files under `images/`.
    /// Stored as the shop mark ([`Store::set_venue_icon_bytes`]); the Owner
    /// can replace it on the next screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_login_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub staff: Vec<ShopDefaultsStaff>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deactivate_tax_rates: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub clear_product_tax: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_charge_bps: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_day_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_retention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_retention_hours: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_today_bar: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_tax_chrome: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_tax_invoice: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pay_methods: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav_hidden_csv: Option<String>,
    /// `"default"` → [`KotSlipFormat::default`] (hand-written packs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kot_slip: Option<String>,

    // ---- format 2 ("Export template") ----
    /// Books settings by key ([`template_pref_keys`]), values as stored.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub prefs: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stations: Option<Vec<StationSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_rates: Option<Vec<TaxRateSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Vec<UnitSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_conversions: Option<Vec<ConversionSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub areas: Option<Vec<AreaSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tables: Option<Vec<TableSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schemes: Option<Vec<SchemeSpec>>,
    /// Bill face: fields, sizes, header, FSSAI, footer, UPI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bill_format: Option<BillSlipFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kot_format: Option<KotSlipFormat>,
    /// Print layouts (type size, line width, margins, send path) — the
    /// looks only; printers and routes belong to each device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub print_formats: Option<Vec<PrintFormat>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_qr_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stamp_base64: Option<String>,
}

impl ShopDefaultsFile {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let d: Self =
            serde_json::from_str(raw).map_err(|e| format!("shop_defaults.json: {e}"))?;
        menu_templates::check_pack_header(d.format, &d.app)?;
        Ok(d)
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let raw =
            fs::read_to_string(path).map_err(|e| format!("Read {}: {e}", path.display()))?;
        Self::parse(&raw)
    }
}

/// One person on the team. `pin` is only for hand-written packs; an export
/// never writes one — the Owner gives each person a first PIN in Team.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShopDefaultsStaff {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role_title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub pin: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub login_id: String,
    /// `"all"` or `"role"` (default) — hand-written packs.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub features: String,
    /// Exact screens, rights and stations (stations by name). Wins over
    /// `features`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access: Option<StaffFeatures>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationSpec {
    pub name: String,
    #[serde(default = "default_prep_mins")]
    pub target_prep_mins: i64,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub picker_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxRateSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub code: String,
    pub bps: i64,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub active: bool,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub picker_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitSpec {
    pub code: String,
    pub name: String,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub picker_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionSpec {
    pub from: String,
    pub to: String,
    pub factor_milli: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaSpec {
    pub name: String,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub picker_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub area: String,
    /// Waiter section: the person's username (or name when they have none).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub waiter: String,
    #[serde(default, skip_serializing_if = "zero_f64")]
    pub x: f64,
    #[serde(default, skip_serializing_if = "zero_f64")]
    pub y: f64,
    #[serde(default, skip_serializing_if = "zero_f64")]
    pub w: f64,
    #[serde(default, skip_serializing_if = "zero_f64")]
    pub h: f64,
}

/// A combo / BOGO / percent scheme; items it names are item names.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SchemeSpec {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub trigger: serde_json::Value,
    #[serde(default)]
    pub reward: serde_json::Value,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub active: bool,
}

/// Which parts of the shop an export writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemplateSections {
    /// Categories, items, photos, stations, tax rates, units, schemes.
    pub menu: bool,
    /// Areas and tables.
    pub floor: bool,
    /// Bill and KOT faces, print layouts, payment QR, stamp.
    pub printing: bool,
    /// Books settings ([`template_pref_keys`]).
    pub settings: bool,
    /// Owner's name and username, staff with their access (no PINs).
    pub team: bool,
    /// Shop name and logo.
    pub brand: bool,
}

impl TemplateSections {
    pub const ALL: Self = Self {
        menu: true,
        floor: true,
        printing: true,
        settings: true,
        team: true,
        brand: true,
    };
}

impl Default for TemplateSections {
    fn default() -> Self {
        Self::ALL
    }
}

/// Counts for the summary an export shows before it saves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TemplateSummary {
    pub categories: usize,
    pub items: usize,
    pub photos: usize,
    pub stations: usize,
    pub tax_rates: usize,
    pub schemes: usize,
    pub areas: usize,
    pub tables: usize,
    pub staff: usize,
    pub settings: usize,
    pub print_faces: usize,
    pub logo: bool,
    pub payment_qr: bool,
    pub stamp: bool,
}

/// What [`Store::export_app_template`] writes into the pack.
#[derive(Clone, Debug)]
pub struct TemplateExport {
    /// `template.json` id; empty = from the shop's name.
    pub id: String,
    /// `template.json` title; empty = "<shop> template".
    pub title: String,
    pub sections: TemplateSections,
}

impl Default for TemplateExport {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            sections: TemplateSections::ALL,
        }
    }
}

/// `"Rajdhani Restaurant"` → `"rajdhani-restaurant"` (ASCII only; empty
/// when the name has no ASCII letters or digits).
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

/// Map item references inside a scheme's trigger / reward JSON: every
/// string under a key ending in `product_id`, every string in an array
/// under a key ending in `product_ids`.
fn map_product_refs(
    value: &mut serde_json::Value,
    map: &mut dyn FnMut(&str) -> Result<String, String>,
) -> Result<(), String> {
    match value {
        serde_json::Value::Object(obj) => {
            for (key, v) in obj.iter_mut() {
                if key.ends_with("product_id") {
                    if let serde_json::Value::String(s) = v {
                        if !s.trim().is_empty() {
                            *s = map(s)?;
                        }
                    }
                } else if key.ends_with("product_ids") {
                    if let serde_json::Value::Array(arr) = v {
                        for e in arr.iter_mut() {
                            if let serde_json::Value::String(s) = e {
                                if !s.trim().is_empty() {
                                    *s = map(s)?;
                                }
                            }
                        }
                    }
                } else {
                    map_product_refs(v, map)?;
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for e in arr.iter_mut() {
                map_product_refs(e, map)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn staff_handle(s: &StaffMember) -> String {
    if s.login_id.trim().is_empty() {
        s.name.trim().to_string()
    } else {
        s.login_id.trim().to_string()
    }
}

fn find_staff_by_handle<'a>(staff: &'a [StaffMember], handle: &str) -> Option<&'a StaffMember> {
    let h = handle.trim();
    staff
        .iter()
        .filter(|s| s.active)
        .find(|s| !s.login_id.is_empty() && s.login_id.eq_ignore_ascii_case(h))
        .or_else(|| {
            staff
                .iter()
                .filter(|s| s.active)
                .find(|s| s.name.trim().eq_ignore_ascii_case(h))
        })
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut text =
        serde_json::to_string_pretty(value).map_err(|e| format!("{}: {e}", path.display()))?;
    text.push('\n');
    fs::write(path, text).map_err(|e| format!("Write {}: {e}", path.display()))
}

/// Pack file name for a photo: its content hash, so the same photo is one
/// file and an export of an imported pack names it the same.
fn photo_rel(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let h = hex::encode(Sha256::digest(bytes));
    format!("images/{}.webp", &h[..16])
}

impl Store {
    // ------------------------------------------------------------ export

    /// Write this shop as an app-template pack into `out_dir` (created; must
    /// be empty if it exists): `template.json`, `shop_defaults.json` and the
    /// menu photos under `images/`. Owner only — it names the team.
    pub fn export_app_template(
        &self,
        out_dir: &Path,
        export: &TemplateExport,
        actor: &StaffMember,
    ) -> Result<TemplateSummary, String> {
        if !actor.is_owner() {
            return Err("Only the Owner can export the shop as a template".into());
        }
        if out_dir.exists()
            && fs::read_dir(out_dir)
                .map_err(|e| format!("{}: {e}", out_dir.display()))?
                .next()
                .is_some()
        {
            return Err(format!("{} is not empty", out_dir.display()));
        }
        fs::create_dir_all(out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;
        let (tmpl, defaults, photos, summary) = self.build_app_template(export, true)?;
        if !photos.is_empty() {
            fs::create_dir_all(out_dir.join("images")).map_err(|e| e.to_string())?;
        }
        for (rel, bytes) in &photos {
            fs::write(out_dir.join(rel), bytes).map_err(|e| format!("Write {rel}: {e}"))?;
        }
        write_json(&out_dir.join("template.json"), &tmpl)?;
        write_json(&out_dir.join("shop_defaults.json"), &defaults)?;
        self.audit(
            actor,
            "template.export",
            &format!("{} items, {} staff", summary.items, summary.staff),
        )?;
        Ok(summary)
    }

    /// What an export with these sections would carry (nothing written).
    pub fn app_template_summary(&self, sections: TemplateSections) -> Result<TemplateSummary, String> {
        let export = TemplateExport {
            sections,
            ..TemplateExport::default()
        };
        Ok(self.build_app_template(&export, false)?.3)
    }

    /// Build both pack files in memory. `with_photos` reads photo bytes
    /// (the export); the summary only counts them.
    #[allow(clippy::type_complexity)]
    fn build_app_template(
        &self,
        export: &TemplateExport,
        with_photos: bool,
    ) -> Result<(MenuTemplate, ShopDefaultsFile, BTreeMap<String, Vec<u8>>, TemplateSummary), String>
    {
        let sec = export.sections;
        let mut summary = TemplateSummary::default();
        let shop = self.venue_title()?;
        let id = if export.id.trim().is_empty() {
            let s = slug(&shop);
            if s.is_empty() { "shop".to_string() } else { s }
        } else {
            export.id.trim().to_string()
        };
        let title = if export.title.trim().is_empty() {
            if shop.trim().is_empty() {
                "Shop template".to_string()
            } else {
                format!("{} template", shop.trim())
            }
        } else {
            export.title.trim().to_string()
        };
        let mut tmpl = MenuTemplate {
            id,
            title,
            blurb: String::new(),
            format: PACK_FORMAT,
            app: PACK_APP.into(),
            categories: Vec::new(),
            items: Vec::new(),
        };
        let mut d = ShopDefaultsFile {
            format: PACK_FORMAT,
            app: PACK_APP.into(),
            app_version: env!("CARGO_PKG_VERSION").into(),
            exported_at: chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            ..ShopDefaultsFile::default()
        };
        let photos: std::cell::RefCell<BTreeMap<String, Vec<u8>>> = Default::default();
        let stations = self.list_stations()?;
        let station_name: HashMap<String, String> =
            stations.iter().map(|s| (s.id.clone(), s.name.clone())).collect();
        let staff = self.list_staff()?;

        if sec.menu {
            let rates = self.all_tax_rates()?;
            let rate_name: HashMap<String, String> =
                rates.iter().map(|r| (r.id.clone(), r.name.clone())).collect();
            let products = self.list_products(false)?;
            let product_name: HashMap<String, String> =
                products.iter().map(|p| (p.id.clone(), p.name.clone())).collect();
            let reorder = self.reorder_levels()?;
            let item_dir = self.item_images_dir()?;
            let photo_ref = |key: &str| -> Result<String, String> {
                let key = key.trim();
                if key.starts_with(crate::menu_images::SHELF_KEY_PREFIX) {
                    return Ok(key.to_string());
                }
                let Some(file) = key.strip_prefix("custom:") else {
                    return Ok(String::new());
                };
                let path = item_dir.join(file);
                if !path.is_file() {
                    return Ok(String::new());
                }
                let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let rel = photo_rel(&bytes);
                photos
                    .borrow_mut()
                    .insert(rel.clone(), if with_photos { bytes } else { Vec::new() });
                Ok(rel)
            };
            let item_spec = |p: &Product| -> Result<TemplateItem, String> {
                let paise = p.price.0;
                let recipe = self
                    .recipe(&p.id)?
                    .into_iter()
                    .map(|l| TemplateRecipeLine {
                        item: product_name.get(&l.item_id).cloned().unwrap_or(l.name),
                        qty_milli: l.qty_milli,
                    })
                    .collect();
                Ok(TemplateItem {
                    name: p.name.clone(),
                    aliases: p.aliases(),
                    price: paise / 100,
                    image: match p.image_key.as_deref() {
                        Some(k) => photo_ref(k)?,
                        None => String::new(),
                    },
                    price_paise: (paise % 100 != 0).then_some(paise),
                    kind: if p.kind == crate::domain::ITEM_KIND_DISH {
                        String::new()
                    } else {
                        p.kind.clone()
                    },
                    available: p.available,
                    unit: p.unit_label.clone(),
                    group: p.group_name.clone(),
                    station: station_name.get(&p.station_id).cloned().unwrap_or_default(),
                    tax_rate: rate_name.get(&p.tax_rate_id).cloned().unwrap_or_default(),
                    tax_bps: p.tax_bps,
                    track_stock: p.stock_qty != STOCK_UNTRACKED,
                    reorder_at: reorder.get(&p.id).copied(),
                    modifiers: p.modifier_groups(),
                    area_prices: p.area_prices(),
                    schedule: p.schedule(),
                    allergens: p.allergens.clone(),
                    recipe,
                })
            };
            let categories = self.list_categories()?;
            for c in &categories {
                let mut cat = TemplateCategory {
                    name: c.name.clone(),
                    aliases: c.aliases(),
                    image: photo_ref(&c.image_key)?,
                    items: Vec::new(),
                };
                for p in products.iter().filter(|p| p.category_id == c.id) {
                    cat.items.push(item_spec(p)?);
                }
                tmpl.categories.push(cat);
            }
            for p in products
                .iter()
                .filter(|p| !categories.iter().any(|c| c.id == p.category_id))
            {
                tmpl.items.push(item_spec(p)?);
            }
            summary.categories = tmpl.categories.len();
            summary.items = tmpl.item_count();

            d.stations = Some(
                stations
                    .iter()
                    .map(|s| StationSpec {
                        name: s.name.clone(),
                        target_prep_mins: s.target_prep_mins,
                        picker_enabled: s.picker_enabled,
                    })
                    .collect(),
            );
            summary.stations = stations.len();
            d.tax_rates = Some(
                rates
                    .iter()
                    .map(|r| TaxRateSpec {
                        name: r.name.clone(),
                        code: r.code.clone(),
                        bps: r.bps,
                        active: r.active,
                        picker_enabled: r.picker_enabled,
                    })
                    .collect(),
            );
            summary.tax_rates = rates.len();
            let units = self.list_units()?;
            let custom: Vec<&str> = units
                .iter()
                .filter(|u| !u.builtin)
                .map(|u| u.code.as_str())
                .collect();
            d.units = Some(
                units
                    .iter()
                    .map(|u| UnitSpec {
                        code: u.code.clone(),
                        name: u.name.clone(),
                        picker_enabled: u.picker_enabled,
                    })
                    .collect(),
            );
            // Built-in conversions are the same on every shop.
            d.unit_conversions = Some(
                self.list_unit_conversions()?
                    .into_iter()
                    .filter(|c| custom.contains(&c.from_code.as_str()) || custom.contains(&c.to_code.as_str()))
                    .map(|c| ConversionSpec {
                        from: c.from_code,
                        to: c.to_code,
                        factor_milli: c.factor_milli,
                    })
                    .collect(),
            );
            let mut schemes = Vec::new();
            for s in self.list_schemes()? {
                let mut trigger: serde_json::Value =
                    serde_json::from_str(&s.trigger_json).unwrap_or(serde_json::json!({}));
                let mut reward: serde_json::Value =
                    serde_json::from_str(&s.reward_json).unwrap_or(serde_json::json!({}));
                let mut to_name = |id: &str| -> Result<String, String> {
                    product_name
                        .get(id)
                        .cloned()
                        .ok_or_else(|| format!("Scheme {} names an item that is no longer on the menu", s.name))
                };
                map_product_refs(&mut trigger, &mut to_name)?;
                map_product_refs(&mut reward, &mut to_name)?;
                schemes.push(SchemeSpec {
                    name: s.name.clone(),
                    kind: s.kind.as_str().into(),
                    trigger,
                    reward,
                    active: s.active,
                });
            }
            summary.schemes = schemes.len();
            d.schemes = Some(schemes);
        }

        if sec.floor {
            let tables = self.list_tables()?;
            let mut areas: Vec<AreaSpec> = Vec::new();
            {
                let mut stmt = self
                    .conn
                    .prepare("SELECT name, picker_enabled FROM floor_areas ORDER BY name")
                    .map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? != 0)))
                    .map_err(|e| e.to_string())?;
                for row in rows {
                    let (name, picker_enabled) = row.map_err(|e| e.to_string())?;
                    areas.push(AreaSpec { name, picker_enabled });
                }
            }
            for t in &tables {
                let a = t.area.trim();
                if !a.is_empty() && !areas.iter().any(|x| x.name.eq_ignore_ascii_case(a)) {
                    areas.push(AreaSpec {
                        name: a.to_string(),
                        picker_enabled: true,
                    });
                }
            }
            let mut specs = Vec::new();
            for t in &tables {
                let (x, y, w, h) = self.table_layout(&t.id)?;
                let waiter = staff
                    .iter()
                    .find(|s| s.active && s.id == t.waiter_staff_id)
                    .map(staff_handle)
                    .unwrap_or_default();
                specs.push(TableSpec {
                    name: t.name.clone(),
                    area: t.area.clone(),
                    waiter,
                    x,
                    y,
                    w,
                    h,
                });
            }
            summary.areas = areas.len();
            summary.tables = specs.len();
            d.areas = Some(areas);
            d.tables = Some(specs);
        }

        if sec.settings {
            for key in template_pref_keys() {
                if let Some(v) = self.meta_get(key)? {
                    d.prefs.insert(key.to_string(), v);
                }
            }
            summary.settings = d.prefs.len();
        }

        if sec.printing {
            d.bill_format = Some(self.bill_slip_format()?);
            d.kot_format = Some(self.kot_slip_format()?);
            if let Some(base) = self.path.parent() {
                d.print_formats = Some(crate::print_share::load_device_printers(base).formats);
            }
            summary.print_faces = 2 + d.print_formats.as_ref().map_or(0, Vec::len);
            if let Some(path) = self.payment_qr_path()? {
                d.payment_qr_base64 =
                    Some(ops::base64_encode(&fs::read(&path).map_err(|e| e.to_string())?));
                summary.payment_qr = true;
            }
            if let Some(path) = self.print_stamp_path()? {
                d.stamp_base64 =
                    Some(ops::base64_encode(&fs::read(&path).map_err(|e| e.to_string())?));
                summary.stamp = true;
            }
        }

        if sec.team {
            if let Some(owner) = staff.iter().find(|s| s.is_owner() && s.active) {
                d.owner_name = Some(owner.name.clone());
                if !owner.login_id.trim().is_empty() {
                    d.owner_login_id = Some(owner.login_id.clone());
                }
            }
            for s in staff.iter().filter(|s| s.active && !s.is_owner()) {
                let mut access = s.features.clone();
                if let Some(ids) = access.kot_stations.as_mut() {
                    *ids = ids
                        .iter()
                        .filter_map(|id| station_name.get(id).cloned())
                        .collect();
                }
                d.staff.push(ShopDefaultsStaff {
                    name: s.name.clone(),
                    role: s.role.as_str().into(),
                    role_title: s.role_title.clone(),
                    pin: String::new(),
                    login_id: s.login_id.clone(),
                    features: String::new(),
                    access: Some(access),
                });
            }
            summary.staff = d.staff.len();
        }

        if sec.brand {
            let shop = shop.trim();
            if !shop.is_empty() {
                d.shop_name = Some(shop.to_string());
            }
            if let Some(file) = self
                .venue_icon_key()?
                .as_deref()
                .and_then(|k| k.strip_prefix("custom:").map(str::to_string))
            {
                let path = self.brand_images_dir()?.join(file);
                if path.is_file() {
                    d.logo_base64 =
                        Some(ops::base64_encode(&fs::read(&path).map_err(|e| e.to_string())?));
                    summary.logo = true;
                }
            }
        }

        let photos = photos.into_inner();
        summary.photos = photos.len();
        Ok((tmpl, d, photos, summary))
    }

    /// Every tax rate on this branch, switched off ones included.
    fn all_tax_rates(&self) -> Result<Vec<crate::domain::TaxRate>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, COALESCE(code, ''), bps, active, COALESCE(picker_enabled, 1)
                 FROM tax_rates WHERE branch_id = ?1 OR branch_id = ''
                 ORDER BY bps, name, code",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                Ok(crate::domain::TaxRate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    code: row.get(2)?,
                    bps: row.get(3)?,
                    active: row.get::<_, i64>(4)? != 0,
                    picker_enabled: row.get::<_, i64>(5).unwrap_or(1) != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------- apply

    /// Apply a downloaded catalogue pack: menu import + optional
    /// `shop_defaults.json` from that pack (no prestored client data in the app).
    /// `apply_shop_defaults: false` imports the menu items only (Menu screen →
    /// Private template): the pack's owner, staff, pay methods and hidden
    /// sections stay out even when its `shop_defaults.json` is present.
    ///
    /// Order: what items point at (stations, tax rates, units, areas) first,
    /// then the menu, then what points at items or people (recipes, schemes,
    /// staff, tables) and the settings.
    pub fn apply_app_template_pack(
        &self,
        pack_dir: &Path,
        apply_shop_defaults: bool,
    ) -> Result<super::CatalogueImport, String> {
        self.assert_server_writer("Apply app template")?;
        let tmpl = menu_templates::read_from_dir(pack_dir)
            .map_err(|e| format!("No usable template.json in {}: {e}", pack_dir.display()))?;
        tmpl.check_compatible()?;
        let defaults = if apply_shop_defaults {
            let path = pack_dir.join("shop_defaults.json");
            if !path.is_file() {
                return Err("Pack declares shop defaults but shop_defaults.json is missing".into());
            }
            Some(ShopDefaultsFile::read(&path)?)
        } else {
            None
        };
        if let Some(d) = &defaults {
            self.apply_template_catalogue_setup(d)?;
        }
        let summary = self.import_menu_template_in(&tmpl, Some(pack_dir))?;
        if let Some(d) = &defaults {
            self.apply_shop_defaults(d)?;
        }
        Ok(summary)
    }

    /// Apply prefs from a pack's `shop_defaults.json` on its own (no menu).
    pub fn apply_shop_defaults_file(&self, path: &Path) -> Result<(), String> {
        self.assert_server_writer("Apply shop defaults")?;
        let d = ShopDefaultsFile::read(path)?;
        self.apply_template_catalogue_setup(&d)?;
        self.apply_shop_defaults(&d)
    }

    fn template_actor(&self) -> Result<StaffMember, String> {
        self.list_staff()?
            .into_iter()
            .find(|s| s.is_owner())
            .ok_or_else(|| "Owner account missing".to_string())
    }

    /// Stations, tax rates, units, conversions and areas — the pack's list
    /// is the list. What the pack lacks is removed when nothing uses it
    /// (a fresh shop's seeded rates and T1…T24), else switched off / kept.
    fn apply_template_catalogue_setup(&self, d: &ShopDefaultsFile) -> Result<(), String> {
        let actor = self.template_actor()?;
        if let Some(specs) = &d.stations {
            let mut have = self.list_stations()?;
            for (i, spec) in specs.iter().enumerate() {
                let name = spec.name.trim();
                if name.is_empty() {
                    continue;
                }
                let id = match have.iter().find(|s| s.name.trim() == name) {
                    Some(s) => s.id.clone(),
                    None => {
                        let s = self.add_station(name)?;
                        let id = s.id.clone();
                        have.push(s);
                        id
                    }
                };
                self.conn
                    .execute(
                        "UPDATE kitchen_stations
                            SET sort_order = ?2, target_prep_mins = ?3, picker_enabled = ?4
                          WHERE id = ?1",
                        params![id, i as i64, spec.target_prep_mins.max(1), spec.picker_enabled as i64],
                    )
                    .map_err(|e| e.to_string())?;
            }
            for s in have {
                if !specs.iter().any(|x| x.name.trim() == s.name.trim()) {
                    self.remove_station(&s.id)?;
                }
            }
        }
        if let Some(specs) = &d.tax_rates {
            let branch = self.active_branch_id()?;
            let have = self.all_tax_rates()?;
            let mut kept: Vec<String> = Vec::new();
            for spec in specs {
                let name = spec.name.trim();
                if name.is_empty() {
                    continue;
                }
                if !(0..=10_000).contains(&spec.bps) {
                    return Err(format!("Tax rate {name}: must be between 0% and 100%"));
                }
                let found = have.iter().find(|r| {
                    !kept.contains(&r.id)
                        && if spec.code.trim().is_empty() {
                            r.name.trim() == name
                        } else {
                            r.code.eq_ignore_ascii_case(spec.code.trim())
                        }
                });
                let id = match found {
                    Some(r) => r.id.clone(),
                    None => {
                        let id = new_id();
                        self.conn
                            .execute(
                                "INSERT INTO tax_rates(id, branch_id, name, code, bps, active)
                                 VALUES(?1, ?2, ?3, ?4, ?5, 1)",
                                params![id, branch, name, spec.code.trim(), spec.bps],
                            )
                            .map_err(|e| e.to_string())?;
                        id
                    }
                };
                self.conn
                    .execute(
                        "UPDATE tax_rates SET name = ?2, code = ?3, bps = ?4, active = ?5,
                                picker_enabled = ?6 WHERE id = ?1",
                        params![
                            id,
                            name,
                            spec.code.trim(),
                            spec.bps,
                            spec.active as i64,
                            spec.picker_enabled as i64
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                kept.push(id);
            }
            for r in have.iter().filter(|r| !kept.contains(&r.id)) {
                let used: bool = self
                    .conn
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM products WHERE tax_rate_id = ?1)",
                        params![r.id],
                        |row| row.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                let sql = if used {
                    "UPDATE tax_rates SET active = 0, picker_enabled = 0 WHERE id = ?1"
                } else {
                    "DELETE FROM tax_rates WHERE id = ?1"
                };
                self.conn.execute(sql, params![r.id]).map_err(|e| e.to_string())?;
            }
        }
        if let Some(specs) = &d.units {
            let have = self.list_units()?;
            for spec in specs {
                let code = spec.code.trim().to_ascii_lowercase();
                if code.is_empty() {
                    continue;
                }
                match have.iter().find(|u| u.code == code) {
                    Some(u) if !u.builtin && u.name != spec.name.trim() => {
                        self.conn
                            .execute(
                                "UPDATE units SET name = ?2 WHERE code = ?1",
                                params![code, spec.name.trim()],
                            )
                            .map_err(|e| e.to_string())?;
                    }
                    Some(_) => {}
                    None => {
                        self.add_unit(&code, &spec.name)?;
                    }
                }
                self.conn
                    .execute(
                        "UPDATE units SET picker_enabled = ?2 WHERE code = ?1",
                        params![code, spec.picker_enabled as i64],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        if let Some(specs) = &d.unit_conversions {
            let have = self.list_unit_conversions()?;
            for spec in specs {
                let (from, to) = (
                    spec.from.trim().to_ascii_lowercase(),
                    spec.to.trim().to_ascii_lowercase(),
                );
                match have.iter().find(|c| c.from_code == from && c.to_code == to) {
                    Some(c) if c.factor_milli == spec.factor_milli => {}
                    Some(c) => {
                        self.conn
                            .execute(
                                "UPDATE unit_conversions SET factor_milli = ?2 WHERE id = ?1",
                                params![c.id, spec.factor_milli],
                            )
                            .map_err(|e| e.to_string())?;
                    }
                    None => {
                        self.add_unit_conversion(&from, &to, spec.factor_milli)?;
                    }
                }
            }
        }
        if let Some(specs) = &d.areas {
            for spec in specs {
                let name = spec.name.trim();
                if name.is_empty() {
                    continue;
                }
                self.ensure_floor_area(name)?;
                self.conn
                    .execute(
                        "UPDATE floor_areas SET picker_enabled = ?2 WHERE lower(name) = lower(?1)",
                        params![name, spec.picker_enabled as i64],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        let _ = actor;
        Ok(())
    }

    /// Recipes and reorder levels name other items: set once every item of
    /// the pack exists. Items the shop already had keep theirs.
    pub(super) fn apply_template_item_links(
        &self,
        tmpl: &MenuTemplate,
        added: &[(String, &TemplateItem)],
    ) -> Result<(), String> {
        let products = self.list_products(false)?;
        for (id, item) in added {
            if let Some(at) = item.reorder_at {
                self.conn
                    .execute(
                        "INSERT INTO stock_levels(product_id, reorder_at) VALUES(?1, ?2)
                         ON CONFLICT(product_id) DO UPDATE SET reorder_at = excluded.reorder_at",
                        params![id, at.max(0)],
                    )
                    .map_err(|e| e.to_string())?;
            }
            if item.recipe.is_empty() {
                continue;
            }
            self.conn
                .execute("DELETE FROM recipe_lines WHERE dish_id = ?1", params![id])
                .map_err(|e| e.to_string())?;
            for line in &item.recipe {
                let Some(stock) = products
                    .iter()
                    .find(|p| p.id != *id && p.name.trim() == line.item.trim())
                else {
                    return Err(format!(
                        "{}: recipe names {} — not in this template",
                        item.name, line.item
                    ));
                };
                self.conn
                    .execute(
                        "INSERT INTO recipe_lines(dish_id, item_id, qty_milli) VALUES(?1, ?2, ?3)
                         ON CONFLICT(dish_id, item_id) DO UPDATE SET qty_milli = excluded.qty_milli",
                        params![id, stock.id, line.qty_milli],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        let _ = tmpl;
        Ok(())
    }

    /// A template item as a new product (stations / tax rates by name).
    pub(super) fn product_from_template_item(
        &self,
        item: &TemplateItem,
        category_id: &str,
        stations: &[crate::domain::KitchenStation],
        rates: &[crate::domain::TaxRate],
    ) -> Product {
        let mut p = Product::new_basic(new_id(), item.name.trim(), Money(item.price_in_paise()));
        p.category_id = category_id.to_string();
        if !item.aliases.is_empty() {
            p.set_aliases(&item.aliases);
        }
        if !item.kind.trim().is_empty() {
            p.kind = item.kind.trim().to_string();
        }
        p.available = item.available;
        p.unit_label = item.unit.trim().to_string();
        p.group_name = item.group.trim().to_string();
        if let Some(s) = stations.iter().find(|s| s.name.trim() == item.station.trim()) {
            p.station_id = s.id.clone();
        }
        if let Some(r) = rates.iter().find(|r| r.name.trim() == item.tax_rate.trim()) {
            p.tax_rate_id = r.id.clone();
        }
        p.tax_bps = item.tax_bps.clamp(0, 10_000);
        p.stock_qty = if item.track_stock { 0 } else { STOCK_UNTRACKED };
        if !item.modifiers.is_empty() {
            p.set_modifier_groups(&item.modifiers);
        }
        if !item.area_prices.is_empty() {
            p.set_area_prices(&item.area_prices);
        }
        if !item.schedule.is_empty() {
            p.set_schedule(&item.schedule);
        }
        p.allergens = item.allergens.trim().to_string();
        p
    }

    /// Format-2 sections of `shop_defaults.json` that come after the menu.
    fn apply_template_after_menu(&self, d: &ShopDefaultsFile) -> Result<(), String> {
        let owner = self.template_actor()?;
        if let Some(specs) = &d.schemes {
            let products = self.list_products(false)?;
            let have = self.list_schemes()?;
            for spec in specs {
                let mut trigger = spec.trigger.clone();
                let mut reward = spec.reward.clone();
                let mut to_id = |name: &str| -> Result<String, String> {
                    products
                        .iter()
                        .find(|p| p.name.trim() == name.trim())
                        .map(|p| p.id.clone())
                        .ok_or_else(|| format!("Scheme {}: no item named {name}", spec.name))
                };
                map_product_refs(&mut trigger, &mut to_id)?;
                map_product_refs(&mut reward, &mut to_id)?;
                let id = have
                    .iter()
                    .find(|s| s.name.trim() == spec.name.trim())
                    .map(|s| s.id.clone())
                    .unwrap_or_else(new_id);
                self.upsert_scheme(&crate::schemes::Scheme {
                    id,
                    name: spec.name.trim().to_string(),
                    kind: crate::schemes::SchemeKind::parse(&spec.kind),
                    trigger_json: trigger.to_string(),
                    reward_json: reward.to_string(),
                    active: spec.active,
                })?;
            }
        }
        if let Some(specs) = &d.tables {
            let staff = self.list_staff()?;
            let have = self.list_tables()?;
            for spec in specs {
                let name = spec.name.trim();
                if name.is_empty() {
                    continue;
                }
                let id = match have.iter().find(|t| t.name.trim() == name) {
                    Some(t) => {
                        if t.area != spec.area.trim() {
                            self.set_table_area(&t.id, spec.area.trim())?;
                        }
                        t.id.clone()
                    }
                    None => self.add_table_in_area(name, spec.area.trim())?.id,
                };
                self.set_table_layout(&id, spec.x, spec.y, spec.w, spec.h, &owner)?;
                let waiter = if spec.waiter.trim().is_empty() {
                    String::new()
                } else {
                    find_staff_by_handle(&staff, &spec.waiter)
                        .map(|s| s.id.clone())
                        .unwrap_or_default()
                };
                self.set_table_waiter(&id, &waiter, &owner)?;
            }
            for t in have {
                if !specs.iter().any(|s| s.name.trim() == t.name.trim())
                    && self.running_order(&t.id)?.is_none()
                {
                    self.delete_table(&t.id)?;
                }
            }
        }
        for (key, value) in &d.prefs {
            if is_template_pref_key(key) {
                self.meta_set(key, value)?;
            }
        }
        if let Some(fmt) = &d.bill_format {
            self.set_bill_slip_format(fmt)?;
        }
        if let Some(fmt) = &d.kot_format {
            self.set_kot_slip_format(fmt)?;
        }
        // Layouts last: the bill / KOT faces above tune the built-in ones,
        // and the pack's layouts are what the shop printed with.
        if let (Some(formats), Some(base)) = (&d.print_formats, self.path.parent()) {
            let mut printers = crate::print_share::load_device_printers(base);
            for f in formats {
                match printers.formats.iter_mut().find(|x| x.id == f.id) {
                    Some(x) => *x = f.clone(),
                    None => printers.formats.push(f.clone()),
                }
            }
            crate::print_share::save_device_printers(base, &printers)?;
        }
        for (b64, file, set) in [
            (&d.payment_qr_base64, "payment_qr.webp", 0u8),
            (&d.stamp_base64, "print_stamp.webp", 1u8),
        ] {
            let Some(b64) = b64.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
                continue;
            };
            let bytes = media::decode_base64(b64).map_err(|e| format!("{file}: {e}"))?;
            media::encode_bytes_as_webp(&bytes, &self.brand_images_dir()?.join(file))?;
            let key = format!("custom:{file}");
            if set == 0 {
                self.set_payment_qr_key(Some(&key))?;
            } else {
                self.set_print_stamp_key(Some(&key))?;
            }
        }
        Ok(())
    }

    pub(super) fn apply_shop_defaults(&self, d: &ShopDefaultsFile) -> Result<(), String> {
        // The shop's own name and mark first: the screen after the import
        // (Owner PIN, then Set up Khaynjo) already shows them.
        if let Some(name) = d.shop_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            self.set_venue_title(name)?;
        }
        if let Some(logo) = d.logo_base64.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let bytes = media::decode_base64(logo).map_err(|e| format!("Shop logo: {e}"))?;
            self.set_venue_icon_bytes(&bytes)?;
        }
        if d.deactivate_tax_rates {
            self.conn
                .execute("UPDATE tax_rates SET active = 0, picker_enabled = 0", [])
                .map_err(|e| e.to_string())?;
        }
        if d.clear_product_tax {
            self.conn
                .execute("UPDATE products SET tax_rate_id = '', tax_bps = 0", [])
                .map_err(|e| e.to_string())?;
        }
        if let Some(bps) = d.service_charge_bps {
            self.set_service_charge_bps(bps)?;
        }
        let owner = self.template_actor()?;
        if let (Some(name), Some(login)) = (
            d.owner_name.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            d.owner_login_id.as_deref().map(str::trim).filter(|s| !s.is_empty()),
        ) {
            self.assert_login_id_free(login, Some(&owner.id))?;
            let n = self
                .conn
                .execute(
                    "UPDATE staff SET name = ?1, login_id = ?2 WHERE id = ?3 AND role = 'owner' AND active = 1",
                    params![name, login, owner.id],
                )
                .map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("Owner account missing".into());
            }
        } else if let Some(name) = d.owner_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            self.conn
                .execute(
                    "UPDATE staff SET name = ?1 WHERE id = ?2 AND role = 'owner' AND active = 1",
                    params![name, owner.id],
                )
                .map_err(|e| e.to_string())?;
        }
        let owner = self.template_actor()?;
        let stations = self.list_stations()?;
        let existing = self.list_staff()?;
        for hire in &d.staff {
            let role = StaffRole::parse(&hire.role)
                .ok_or_else(|| format!("Unknown staff role '{}'", hire.role))?;
            let features = match &hire.access {
                Some(access) => {
                    let mut f = access.clone();
                    if let Some(names) = f.kot_stations.as_mut() {
                        *names = names
                            .iter()
                            .filter_map(|n| {
                                stations
                                    .iter()
                                    .find(|s| s.name.trim() == n.trim())
                                    .map(|s| s.id.clone())
                            })
                            .collect();
                    }
                    f
                }
                None => match hire.features.trim() {
                    "all" => StaffFeatures::all(),
                    _ => StaffFeatures::for_role(role),
                },
            };
            let title = if hire.role_title.trim().is_empty() {
                role.default_title()
            } else {
                hire.role_title.trim()
            };
            let same = |s: &&StaffMember| {
                s.active
                    && ((!hire.login_id.trim().is_empty()
                        && s.login_id.eq_ignore_ascii_case(hire.login_id.trim()))
                        || s.name.eq_ignore_ascii_case(hire.name.trim())
                        || (role == StaffRole::Admin && s.role == StaffRole::Admin))
            };
            match existing.iter().find(same) {
                None => {
                    self.add_staff_from_template(
                        &super::StaffHire {
                            name: hire.name.trim(),
                            role,
                            role_title: title,
                            pin: hire.pin.trim(),
                            login_id: hire.login_id.trim(),
                            password: "",
                            features,
                        },
                        &owner,
                    )?;
                }
                Some(s) => {
                    if !hire.login_id.trim().is_empty() {
                        self.ensure_template_login(&s.id, hire.login_id.trim())?;
                    }
                    if hire.access.is_some() && !s.is_owner() {
                        self.conn
                            .execute(
                                "UPDATE staff SET features_json = ?1, role_title = ?2 WHERE id = ?3",
                                params![features.to_json(), title, s.id],
                            )
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        if let Some(mode) = d.business_day_mode.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            self.meta_set("business_day_mode", mode)?;
        }
        if let Some(v) = d.history_retention.as_deref() {
            self.meta_set("history_retention", v)?;
        }
        if let Some(v) = d.history_retention_hours.as_deref() {
            self.meta_set("history_retention_hours", v)?;
        }
        if let Some(v) = d.show_today_bar {
            self.meta_set("show_today_bar", if v { "1" } else { "0" })?;
        }
        if let Some(v) = d.show_tax_chrome {
            self.meta_set("show_tax_chrome", if v { "1" } else { "0" })?;
        }
        if let Some(v) = d.show_tax_invoice {
            self.meta_set("show_tax_invoice", if v { "1" } else { "0" })?;
        }
        if let Some(v) = d.pay_methods.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            self.meta_set("pay_methods", v)?;
        }
        if let Some(v) = d.nav_hidden_csv.as_deref() {
            self.set_nav_hidden_csv(v)?;
        }
        match (d.kot_format.is_some(), d.kot_slip.as_deref().map(str::trim).filter(|s| !s.is_empty())) {
            (true, _) => {}
            (false, Some("default") | None) => {
                let kot = KotSlipFormat::default().clamped();
                self.set_kot_slip_format(&kot)?;
            }
            (false, Some(other)) => {
                return Err(format!("Unknown kot_slip '{other}' in shop_defaults.json"));
            }
        }
        self.apply_template_after_menu(d)
    }
}

/// What `--check-template` found wrong with a pack (empty = fine).
pub fn lint_pack(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let tmpl = match menu_templates::read_from_dir(dir) {
        Ok(t) => t,
        Err(e) => return vec![e],
    };
    if let Err(e) = tmpl.check_compatible() {
        out.push(e);
    }
    let mut names: HashMap<String, usize> = HashMap::new();
    for item in tmpl.all_items() {
        *names.entry(item.name.trim().to_lowercase()).or_default() += 1;
        if item.name.trim().is_empty() {
            out.push("an item has no name".into());
        }
        if item.price_in_paise() < 0 {
            out.push(format!("{}: negative price", item.name));
        }
        let img = item.image.trim();
        if img.starts_with("images/") && !dir.join(img).is_file() {
            out.push(format!("{}: photo {img} is missing", item.name));
        }
        for line in &item.recipe {
            if !tmpl.all_items().any(|i| i.name.trim() == line.item.trim()) {
                out.push(format!("{}: recipe names {} — not in this pack", item.name, line.item));
            }
        }
    }
    for c in &tmpl.categories {
        let img = c.image.trim();
        if img.starts_with("images/") && !dir.join(img).is_file() {
            out.push(format!("category {}: photo {img} is missing", c.name));
        }
    }
    for (name, n) in names {
        if n > 1 {
            out.push(format!("{n} items are named {name} — recipes and schemes find the first"));
        }
    }
    let path = dir.join("shop_defaults.json");
    if path.is_file() {
        let raw = fs::read_to_string(&path).unwrap_or_default();
        match ShopDefaultsFile::parse(&raw) {
            Err(e) => out.push(e),
            Ok(d) => {
                for s in &d.staff {
                    if !s.pin.trim().is_empty() {
                        out.push(format!(
                            "staff {}: a PIN in the pack — remove it; the Owner sets first PINs in Team",
                            s.name
                        ));
                    }
                }
                for key in d.prefs.keys() {
                    if !is_template_pref_key(key) {
                        out.push(format!("prefs.{key}: not a setting a pack may carry"));
                    }
                }
                let stations: Vec<&str> = d
                    .stations
                    .iter()
                    .flatten()
                    .map(|s| s.name.trim())
                    .collect();
                if d.stations.is_some() {
                    for item in tmpl.all_items() {
                        if !item.station.is_empty() && !stations.contains(&item.station.trim()) {
                            out.push(format!("{}: station {} is not in stations", item.name, item.station));
                        }
                    }
                }
                for (field, b64) in [
                    ("logo_base64", &d.logo_base64),
                    ("payment_qr_base64", &d.payment_qr_base64),
                    ("stamp_base64", &d.stamp_base64),
                ] {
                    if let Some(b64) = b64 {
                        match media::decode_base64(b64) {
                            Ok(bytes) if image::load_from_memory(&bytes).is_ok() => {}
                            _ => out.push(format!("{field}: not a picture")),
                        }
                    }
                }
            }
        }
    }
    // Anything customer- or money-shaped has no place in a pack.
    for f in ["customers.json", "sales.json", "books.sqlite3"] {
        if dir.join(f).exists() {
            out.push(format!("{f}: shop data does not belong in a template"));
        }
    }
    out.sort();
    out
}

fn rupees(paise: i64) -> String {
    if paise % 100 == 0 {
        format!("{}", paise / 100)
    } else {
        format!("{}.{:02}", paise / 100, (paise % 100).abs())
    }
}

/// Readable differences between two packs (`--check-template new old`):
/// what the client changed since the published one.
pub fn diff_packs(new_dir: &Path, old_dir: &Path) -> Result<Vec<String>, String> {
    let new_t = menu_templates::read_from_dir(new_dir)?;
    let old_t = menu_templates::read_from_dir(old_dir)?;
    let mut out = Vec::new();
    let index = |t: &MenuTemplate| -> BTreeMap<String, TemplateItem> {
        t.all_items()
            .map(|i| (i.name.trim().to_string(), i.clone()))
            .collect()
    };
    let (n, o) = (index(&new_t), index(&old_t));
    for (name, item) in &n {
        match o.get(name) {
            None => out.push(format!("+ item {name} ₹{}", rupees(item.price_in_paise()))),
            Some(old) if old != item => {
                if old.price_in_paise() != item.price_in_paise() {
                    out.push(format!(
                        "~ item {name}: ₹{} → ₹{}",
                        rupees(old.price_in_paise()),
                        rupees(item.price_in_paise())
                    ));
                } else {
                    out.push(format!("~ item {name}"));
                }
            }
            _ => {}
        }
    }
    for name in o.keys().filter(|k| !n.contains_key(*k)) {
        out.push(format!("- item {name}"));
    }
    let read = |dir: &Path| -> Result<serde_json::Value, String> {
        let p = dir.join("shop_defaults.json");
        if !p.is_file() {
            return Ok(serde_json::json!({}));
        }
        let mut v: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&p).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("{}: {e}", p.display()))?;
        if let Some(obj) = v.as_object_mut() {
            obj.remove("exported_at");
            obj.remove("app_version");
        }
        Ok(v)
    };
    let (nd, od) = (read(new_dir)?, read(old_dir)?);
    let empty = serde_json::Map::new();
    let (nm, om) = (nd.as_object().unwrap_or(&empty), od.as_object().unwrap_or(&empty));
    let mut keys: Vec<&String> = nm.keys().chain(om.keys()).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        if key == "prefs" {
            let np = nm.get(key).and_then(|v| v.as_object()).unwrap_or(&empty);
            let op = om.get(key).and_then(|v| v.as_object()).unwrap_or(&empty);
            let mut pk: Vec<&String> = np.keys().chain(op.keys()).collect();
            pk.sort();
            pk.dedup();
            for k in pk {
                if np.get(k) != op.get(k) {
                    out.push(format!(
                        "~ setting {k}: {} → {}",
                        op.get(k).map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
                        np.get(k).map(|v| v.to_string()).unwrap_or_else(|| "—".into())
                    ));
                }
            }
        } else if nm.get(key) != om.get(key) {
            out.push(format!("~ {key}"));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Cart, ModifierGroup, ModifierGroupKind, ModifierOption, PayMethod, ScheduleWindow,
    };
    use crate::store::StaffHire;
    use std::path::PathBuf;

    fn scratch(tag: &str) -> PathBuf {
        let dir = crate::test_dirs::run_root().join(format!("khaynjo-template-{tag}-{}", new_id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn picture(dir: &Path, name: &str, rgb: [u8; 3], w: u32, h: u32) -> PathBuf {
        let mut img = image::RgbImage::new(w, h);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = image::Rgb([rgb[0].wrapping_add(x as u8), rgb[1].wrapping_add(y as u8), rgb[2]]);
        }
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    fn owner(store: &Store) -> StaffMember {
        store.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap()
    }

    /// A demo shop with something in every section a pack carries — plus a
    /// customer and a sale, which must not travel.
    fn configured_shop(dir: &Path) -> Store {
        let store = Store::open_path(&dir.join("books.sqlite3")).unwrap();
        let me = owner(&store);
        store.set_venue_title("Demo Kitchen").unwrap();
        store
            .set_venue_icon_bytes(&fs::read(picture(dir, "logo.png", [200, 40, 30], 64, 48)).unwrap())
            .unwrap();
        store
            .import_payment_qr(&picture(dir, "qr.png", [10, 10, 10], 40, 40))
            .unwrap();
        store
            .import_print_stamp(&picture(dir, "stamp.png", [20, 90, 200], 50, 30))
            .unwrap();

        let tandoor = store.add_station("Tandoor").unwrap();
        let chinese = store.add_station("Chinese").unwrap();
        store
            .conn
            .execute(
                "UPDATE kitchen_stations SET target_prep_mins = 22 WHERE id = ?1",
                params![tandoor.id],
            )
            .unwrap();
        store.set_station_picker_enabled(&chinese.id, false, &me).unwrap();
        let gst = store.add_tax_rate("Food GST", "food5", 500).unwrap();
        store.add_unit("crate", "Crate").unwrap();
        store.add_unit_conversion("crate", "kg", 25_000).unwrap();
        store.set_unit_picker_enabled("dozen", false, &me).unwrap();

        let starters = store.add_category("Starters").unwrap();
        store
            .set_category_aliases_if_empty(&starters.id, &["Shuruaat".to_string()])
            .unwrap();
        let key = store
            .import_category_image(&starters.id, &picture(dir, "cat.png", [90, 160, 30], 32, 32))
            .unwrap();
        store.set_category_image(&starters.id, &key).unwrap();
        let drinks = store.add_category("Drinks").unwrap();

        let mut tikka = Product::new_basic(new_id(), "Paneer Tikka", Money(12_550));
        tikka.category_id = starters.id.clone();
        tikka.station_id = tandoor.id.clone();
        tikka.tax_rate_id = gst.id.clone();
        tikka.tax_bps = 500;
        tikka.group_name = "Paneer Tikka".into();
        tikka.allergens = "dairy".into();
        tikka.set_aliases(&["PT".to_string()]);
        tikka.set_modifier_groups(&[ModifierGroup {
            name: "Size".into(),
            kind: ModifierGroupKind::Variation,
            min: 1,
            max: 1,
            options: vec![
                ModifierOption { name: "Half".into(), price_paise: 0, group: "Size".into() },
                ModifierOption { name: "Full".into(), price_paise: 8_000, group: "Size".into() },
            ],
        }]);
        tikka.set_area_prices(&[ModifierOption {
            name: "AC".into(),
            price_paise: 14_000,
            group: String::new(),
        }]);
        tikka.set_schedule(&[ScheduleWindow {
            days: vec![6, 7],
            start: "12:00".into(),
            end: "15:00".into(),
            price_paise: None,
        }]);
        tikka.image_key = Some(
            store
                .import_item_image(&tikka.id, &picture(dir, "tikka.png", [180, 120, 40], 48, 36))
                .unwrap(),
        );
        store.upsert_product(&tikka).unwrap();
        let mut lassi = Product::new_basic(new_id(), "Lassi", Money(6_000));
        lassi.category_id = drinks.id.clone();
        lassi.available = false;
        store.upsert_product(&lassi).unwrap();
        let mut cola = Product::new_basic(new_id(), "Cola", Money(4_000));
        cola.category_id = drinks.id.clone();
        cola.kind = crate::domain::ITEM_KIND_RESALE.into();
        store.upsert_product(&cola).unwrap();
        let mut paneer = Product::new_basic(new_id(), "Paneer", Money(0));
        paneer.kind = crate::domain::ITEM_KIND_RAW.into();
        paneer.unit_label = "kg".into();
        store.upsert_product(&paneer).unwrap();
        store.set_product_stock_qty(&paneer.id, 7).unwrap();
        store.set_reorder_level(&paneer.id, 3, &me).unwrap();
        store
            .set_recipe_from_text(&tikka.id, "Paneer=0.15", &me)
            .unwrap();
        store
            .upsert_scheme(&crate::schemes::Scheme::bogo(
                new_id(),
                "Lassi with Tikka",
                &tikka.id,
                1,
                &lassi.id,
                1,
            ))
            .unwrap();

        let ac1 = store.add_table_in_area("AC1", "AC").unwrap();
        store.set_table_layout(&ac1.id, 10.0, 20.0, 80.0, 60.0, &me).unwrap();
        let gone: Vec<String> = (13..=24).map(|n| format!("T{n}")).collect();
        store.remove_tables_named(&gone).unwrap();
        store.set_area_picker_enabled("Parcel", false, &me).unwrap();

        let mut access = StaffFeatures::for_role(StaffRole::Waiter);
        access.kot_stations = Some(vec![tandoor.id.clone()]);
        let ravi = store
            .add_staff_with_features(
                &StaffHire {
                    name: "Ravi",
                    role: StaffRole::Waiter,
                    role_title: "Captain",
                    pin: "4321",
                    login_id: "ravi",
                    password: "secret",
                    features: access,
                },
                &me,
            )
            .unwrap();
        store.set_table_waiter(&ac1.id, &ravi, &me).unwrap();

        store.set_pay_methods_csv("cash,upi").unwrap();
        store.set_show_tax_chrome(true).unwrap();
        store.set_nav_hidden_csv("buy,branches").unwrap();
        store.set_floor_columns(3).unwrap();
        let mut bill = store.bill_slip_format().unwrap();
        bill.header = "12 Main Road".into();
        bill.footer = "Visit again".into();
        bill.fssai = "11223344556677".into();
        store.set_bill_slip_format(&bill).unwrap();
        let mut kot = store.kot_slip_format().unwrap();
        kot.show_title = !kot.show_title;
        store.set_kot_slip_format(&kot).unwrap();
        store.set_standard_print_em(24.0).unwrap();

        // Shop data: never in a template.
        let customer = store.add_customer("Meera Shah", "9800000001").unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&store.list_products(false).unwrap().into_iter().find(|p| p.name == "Cola").unwrap());
        store
            .finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, Some(&customer.id), &me)
            .unwrap();
        store
    }

    fn pack_files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    let rel = p.strip_prefix(dir).unwrap().to_string_lossy().into_owned();
                    let mut bytes = fs::read(&p).unwrap();
                    if rel == "shop_defaults.json" {
                        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                        let obj = v.as_object_mut().unwrap();
                        assert!(obj.remove("exported_at").is_some());
                        assert!(obj.remove("app_version").is_some());
                        bytes = serde_json::to_vec_pretty(&v).unwrap();
                    }
                    out.insert(rel, bytes);
                }
            }
        }
        out
    }

    /// The guard for the whole feature: what an export writes, the import
    /// applies — a fresh shop given the pack exports the very same pack.
    #[test]
    fn an_exported_shop_applied_to_a_fresh_shop_exports_the_same_pack() {
        let base = scratch("roundtrip");
        let src_dir = base.join("source");
        fs::create_dir_all(&src_dir).unwrap();
        let source = configured_shop(&src_dir);
        let pack_a = base.join("pack-a");
        let summary = source
            .export_app_template(&pack_a, &TemplateExport::default(), &owner(&source))
            .unwrap();
        assert_eq!(summary.categories, 2);
        assert_eq!(summary.items, 4);
        assert_eq!(summary.photos, 2, "dish + category photo");
        assert_eq!(summary.staff, 1);
        assert!(summary.logo && summary.payment_qr && summary.stamp);
        assert_eq!(source.app_template_summary(TemplateSections::ALL).unwrap(), summary);

        let defaults = fs::read_to_string(pack_a.join("shop_defaults.json")).unwrap();
        let menu = fs::read_to_string(pack_a.join("template.json")).unwrap();
        for secret in ["Meera", "9800000001", "4321", "secret", "pin_hash", "password"] {
            assert!(!defaults.contains(secret) && !menu.contains(secret), "{secret} leaked");
        }
        assert!(!defaults.contains("\"pin\""), "an export never writes a PIN");
        // Every section really carries what the shop set (a field left out
        // of both export and import would still round-trip).
        let d = ShopDefaultsFile::parse(&defaults).unwrap();
        let t: MenuTemplate = serde_json::from_str(&menu).unwrap();
        let tikka = t.categories[0].items.iter().find(|i| i.name == "Paneer Tikka").unwrap();
        assert_eq!(t.categories[0].name, "Starters");
        assert_eq!(t.categories[0].aliases, vec!["Shuruaat".to_string()]);
        assert!(t.categories[0].image.starts_with("images/"));
        assert_eq!((tikka.price, tikka.price_paise), (125, Some(12_550)));
        assert_eq!((tikka.station.as_str(), tikka.tax_rate.as_str(), tikka.tax_bps), ("Tandoor", "Food GST", 500));
        assert_eq!(tikka.modifiers[0].options.len(), 2);
        assert_eq!(tikka.area_prices[0].name, "AC");
        assert_eq!(tikka.schedule[0].days, vec![6, 7]);
        assert_eq!(tikka.recipe, vec![TemplateRecipeLine { item: "Paneer".into(), qty_milli: 150 }]);
        assert_eq!((tikka.group.as_str(), tikka.allergens.as_str()), ("Paneer Tikka", "dairy"));
        let paneer = &t.items[0];
        assert_eq!((paneer.name.as_str(), paneer.kind.as_str(), paneer.unit.as_str()), ("Paneer", "raw", "kg"));
        assert!(paneer.track_stock);
        assert_eq!(paneer.reorder_at, Some(3));
        assert!(!t.categories[1].items.iter().find(|i| i.name == "Lassi").unwrap().available);
        let st = d.stations.as_ref().unwrap();
        assert_eq!((st[0].name.as_str(), st[0].target_prep_mins), ("Tandoor", 22));
        assert!(!st[1].picker_enabled);
        assert!(d.tax_rates.as_ref().unwrap().iter().any(|r| r.code == "food5" && r.bps == 500));
        assert!(d.units.as_ref().unwrap().iter().any(|u| u.code == "dozen" && !u.picker_enabled));
        assert_eq!(
            d.unit_conversions.as_ref().unwrap(),
            &vec![ConversionSpec { from: "crate".into(), to: "kg".into(), factor_milli: 25_000 }]
        );
        assert!(d.areas.as_ref().unwrap().iter().any(|a| a.name == "Parcel" && !a.picker_enabled));
        let ac1 = d.tables.as_ref().unwrap().iter().find(|t| t.name == "AC1").unwrap();
        assert_eq!((ac1.area.as_str(), ac1.waiter.as_str(), ac1.w), ("AC", "ravi", 80.0));
        assert!(!d.tables.as_ref().unwrap().iter().any(|t| t.name == "T13"));
        let scheme = &d.schemes.as_ref().unwrap()[0];
        assert_eq!(scheme.trigger["buy_product_id"], "Paneer Tikka");
        assert_eq!(scheme.reward["get_product_id"], "Lassi");
        assert_eq!(d.prefs["pay_methods"], "cash,upi");
        assert_eq!(d.prefs["show_tax_chrome"], "1");
        assert_eq!(d.prefs["ui_nav_hidden"], "buy,branches");
        assert_eq!(d.prefs["ui_floor_columns"], "3");
        assert_eq!(d.prefs["standard_print_em"], "24");
        let bill = d.bill_format.as_ref().unwrap();
        assert_eq!((bill.header.as_str(), bill.footer.as_str()), ("12 Main Road", "Visit again"));
        assert!(d.kot_format.as_ref().unwrap().show_title);
        assert_eq!(d.staff[0].role_title, "Captain");
        assert_eq!(d.staff[0].access.as_ref().unwrap().kot_stations, Some(vec!["Tandoor".to_string()]));
        assert_eq!(d.shop_name.as_deref(), Some("Demo Kitchen"));
        assert!(lint_pack(&pack_a).is_empty(), "{:?}", lint_pack(&pack_a));

        let dst_dir = base.join("fresh");
        fs::create_dir_all(&dst_dir).unwrap();
        let fresh = Store::open_path(&dst_dir.join("books.sqlite3")).unwrap();
        let done = fresh.apply_app_template_pack(&pack_a, true).unwrap();
        assert_eq!(done.items_added, 4);
        let pack_b = base.join("pack-b");
        fresh
            .export_app_template(&pack_b, &TemplateExport::default(), &owner(&fresh))
            .unwrap();
        let (a, b) = (pack_files(&pack_a), pack_files(&pack_b));
        assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
        for (rel, bytes) in &a {
            assert!(
                bytes == &b[rel],
                "{rel} differs after export → apply → export:\n--- source\n{}\n--- applied\n{}",
                String::from_utf8_lossy(bytes),
                String::from_utf8_lossy(&b[rel])
            );
        }
        assert!(diff_packs(&pack_b, &pack_a).unwrap().is_empty());

        // The fresh shop really has the setup, not just the same files.
        let tikka = fresh
            .list_products(false)
            .unwrap()
            .into_iter()
            .find(|p| p.name == "Paneer Tikka")
            .unwrap();
        assert_eq!(tikka.price, Money(12_550));
        assert_eq!(fresh.recipe(&tikka.id).unwrap().len(), 1);
        let ravi = fresh.list_staff().unwrap().into_iter().find(|s| s.login_id == "ravi").unwrap();
        assert!(!ravi.has_pin(), "PINs never travel");
        assert!(ravi.password_hash.is_empty());
        assert!(fresh.list_customers().unwrap().is_empty());
        assert_eq!(fresh.standard_print_em().unwrap(), 24.0);
        let _ = fs::remove_dir_all(base);
    }

    /// Before deploying a client's pack: download it from the staged site
    /// tree the way a new device does, apply it to a fresh shop, export —
    /// the result must equal the pack the client sent.
    ///
    /// ```text
    /// PACK_CDN="$PWD/sites/khaynjo/static/v1/catalogue" PACK_ID=<id> \
    /// PACK_SOURCE="$PWD/apps/khaynjo/catalogue/private-src/<client>" \
    ///   cargo test -p kumkaj-khaynjo --lib published_pack -- --ignored
    /// ```
    /// `PACK_ID` defaults to `id` in the source `template.json`. The paths
    /// are absolute because cargo runs this from `apps/khaynjo`. For exported
    /// packs (format 2): a hand-written pack such as Rajdhani's names shelf
    /// photos (`shelf:…`), which the export renames, so it always differs.
    #[test]
    #[ignore]
    fn published_pack_downloads_applies_and_exports_unchanged() {
        let (Ok(cdn), Ok(source)) = (std::env::var("PACK_CDN"), std::env::var("PACK_SOURCE")) else {
            panic!("set PACK_CDN and PACK_SOURCE");
        };
        let id = std::env::var("PACK_ID").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| {
            let raw = std::fs::read_to_string(Path::new(&source).join("template.json"))
                .expect("source template.json");
            let v: serde_json::Value = serde_json::from_str(&raw).expect("template json");
            v["id"].as_str().expect("template id").to_string()
        });
        let base = scratch("published");
        let (dir, man, _) = crate::catalogue_remote::download_pack_from(
            &crate::catalogue_remote::FsSource::new(&cdn),
            &base,
            crate::catalogue_remote::CatalogueVisibility::Public,
            &id,
            &|| false,
            &|_| {},
        )
        .unwrap();
        let store = Store::open_path(&base.join("books.sqlite3")).unwrap();
        store.apply_app_template_pack(&dir, man.applies_shop_defaults).unwrap();
        let again = base.join("again");
        let me = owner(&store);
        let export = TemplateExport {
            id: man.id.clone(),
            title: man.title.clone(),
            ..TemplateExport::default()
        };
        store.export_app_template(&again, &export, &me).unwrap();
        let changes = diff_packs(&again, Path::new(&source)).unwrap();
        assert!(changes.is_empty(), "the published pack applies differently:\n{}", changes.join("\n"));
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn a_person_without_a_pin_gets_a_first_one_from_the_owner_once() {
        let base = scratch("first-pin");
        let store = Store::open_path(&base.join("books.sqlite3")).unwrap();
        let me = owner(&store);
        let id = store
            .add_staff_from_template(
                &StaffHire {
                    name: "Asha",
                    role: StaffRole::Cashier,
                    role_title: "",
                    pin: "",
                    login_id: "asha",
                    password: "",
                    features: StaffFeatures::for_role(StaffRole::Cashier),
                },
                &me,
            )
            .unwrap();
        assert!(store.authenticate_staff_pin(&id, "").unwrap().is_none());
        assert!(store.authenticate_staff_pin(&id, "0000").unwrap().is_none());
        store.set_first_staff_pin(&id, "2580", &me).unwrap();
        assert!(store.authenticate_staff_pin(&id, "2580").unwrap().is_some());
        assert!(
            store.set_first_staff_pin(&id, "1111", &me).is_err(),
            "after the first PIN only the person changes it"
        );
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn another_apps_pack_or_a_newer_format_is_refused() {
        let base = scratch("refuse");
        let store = Store::open_path(&base.join("books.sqlite3")).unwrap();
        for (format, app, want) in [
            (2, "moklanjo", "not Khaynjo"),
            (PACK_FORMAT + 1, "khaynjo", "newer Khaynjo"),
        ] {
            let pack = base.join(format!("pack-{app}-{format}"));
            fs::create_dir_all(&pack).unwrap();
            fs::write(
                pack.join("template.json"),
                serde_json::json!({"id": "x", "title": "X", "format": format, "app": app,
                    "categories": [{"name": "C", "items": [{"name": "Tea", "price": 10}]}]})
                .to_string(),
            )
            .unwrap();
            let err = store.apply_app_template_pack(&pack, false).unwrap_err();
            assert!(err.contains(want), "{err}");
            assert!(store.list_products(false).unwrap().is_empty(), "nothing half-applied");
        }
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn lint_names_pins_unknown_settings_and_missing_photos() {
        let pack = scratch("lint");
        fs::write(
            pack.join("template.json"),
            serde_json::json!({"id": "x", "title": "X",
                "categories": [{"name": "C", "items": [{"name": "Tea", "image": "images/tea.webp"}]}]})
            .to_string(),
        )
        .unwrap();
        fs::write(
            pack.join("shop_defaults.json"),
            serde_json::json!({"staff": [{"name": "Asha", "role": "cashier", "pin": "1234"}],
                "prefs": {"device_role": "server"}})
            .to_string(),
        )
        .unwrap();
        let found = lint_pack(&pack).join("\n");
        assert!(found.contains("staff Asha: a PIN"), "{found}");
        assert!(found.contains("prefs.device_role"), "{found}");
        assert!(found.contains("images/tea.webp is missing"), "{found}");
        let _ = fs::remove_dir_all(pack);
    }

    #[test]
    fn every_setting_shared_with_devices_travels_in_a_pack() {
        for key in super::super::SHOP_PREF_KEYS {
            assert!(template_pref_keys().any(|k| k == *key), "{key}");
        }
        for key in ["owner_setup_complete", "device_role", "pairing_code", "language", "org_id"] {
            assert!(!is_template_pref_key(key), "{key} is this device's own");
        }
    }
}

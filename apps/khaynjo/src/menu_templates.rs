//! Named client menu templates (`assets/templates/<id>/template.json`).
//!
//! Separate from the big photo shelf (`menu_images`): these are concrete
//! restaurant menus — primary `name` plus optional aliases (e.g. English),
//! listed prices, and photos (`shelf:…` or a file under the template).
//! Public/demo folders under `assets/templates/`. Private client packs
//! (Rajdhani) are authored under `catalogue/private-src/` and published to
//! the CDN — they are not discovered here.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::{ModifierGroup, ModifierOption, ScheduleWindow};
use crate::menu_images::{self, SHELF_KEY_PREFIX};

/// Pack format this build writes and reads in full. `0` = a hand-written
/// pack (menu names, aliases, prices, photos only); `2` = written by
/// "Export template" ([`crate::store::Store::export_app_template`]) and
/// carrying every item field below. A pack newer than this is refused —
/// half-applying a shop's setup is worse than asking for an app update.
pub const PACK_FORMAT: u32 = 2;
/// The only app whose packs this build imports (`app` in both pack files).
pub const PACK_APP: &str = "khaynjo";

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}
fn is_zero_i64(v: &i64) -> bool {
    *v == 0
}
fn yes() -> bool {
    true
}
fn is_true(v: &bool) -> bool {
    *v
}
fn is_false(v: &bool) -> bool {
    !*v
}
fn is_dish(v: &String) -> bool {
    v.is_empty() || v == crate::domain::ITEM_KIND_DISH
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuTemplate {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub blurb: String,
    /// [`PACK_FORMAT`] of an exported pack; 0 / absent = hand-written.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub format: u32,
    /// [`PACK_APP`] for exported packs; empty = hand-written (any app reads it).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub app: String,
    #[serde(default)]
    pub categories: Vec<TemplateCategory>,
    /// Items in no category (raw materials, mostly). Older apps ignore this.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<TemplateItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateCategory {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// `shelf:…` or `images/….webp` under this template folder.
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub items: Vec<TemplateItem>,
}

/// One menu (or stock) item. Everything after `image` is optional and only
/// written by "Export template": a hand-written pack leaves them out and
/// gets today's defaults (a dish, on the menu, no station / tax / stock).
/// Stations, tax rates, areas and recipe items are named, never ids — ids
/// differ on every device.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TemplateItem {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Rupees (major units). `0` = unpriced / season rate.
    #[serde(default)]
    pub price: i64,
    #[serde(default)]
    pub image: String,
    /// Exact price in paise when it is not whole rupees (wins over `price`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_paise: Option<i64>,
    /// `dish` (default) / `resale` / `raw` — [`crate::domain::ITEM_KIND_DISH`].
    #[serde(default, skip_serializing_if = "is_dish")]
    pub kind: String,
    /// On the menu now (off = "out of stock" at the till).
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub available: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    /// Dish family ("Paneer Tikka" for Half and Full).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group: String,
    /// Kitchen station name (`stations` in `shop_defaults.json`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub station: String,
    /// Tax rate name (`tax_rates` in `shop_defaults.json`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tax_rate: String,
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub tax_bps: i64,
    /// Count stock for this item (the count itself never travels).
    #[serde(default, skip_serializing_if = "is_false")]
    pub track_stock: bool,
    /// Low-stock line when it differs from the app's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reorder_at: Option<i64>,
    /// Size / add-on groups with their choices and prices.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ModifierGroup>,
    /// Price by floor area: `name` = the area.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub area_prices: Vec<ModifierOption>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schedule: Vec<ScheduleWindow>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub allergens: String,
    /// What one portion uses up, by stock item name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipe: Vec<TemplateRecipeLine>,
}

/// One recipe ingredient: `qty_milli` thousandths of the item's unit.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateRecipeLine {
    pub item: String,
    pub qty_milli: i64,
}

impl Default for TemplateItem {
    fn default() -> Self {
        Self {
            name: String::new(),
            aliases: Vec::new(),
            price: 0,
            image: String::new(),
            price_paise: None,
            kind: String::new(),
            available: true,
            unit: String::new(),
            group: String::new(),
            station: String::new(),
            tax_rate: String::new(),
            tax_bps: 0,
            track_stock: false,
            reorder_at: None,
            modifiers: Vec::new(),
            area_prices: Vec::new(),
            schedule: Vec::new(),
            allergens: String::new(),
            recipe: Vec::new(),
        }
    }
}

impl TemplateItem {
    /// Price in paise: `price_paise` when set, else whole rupees.
    pub fn price_in_paise(&self) -> i64 {
        self.price_paise
            .unwrap_or_else(|| self.price.saturating_mul(100))
            .max(0)
    }
}

impl MenuTemplate {
    pub fn category_count(&self) -> usize {
        self.categories.len()
    }

    pub fn item_count(&self) -> usize {
        self.categories.iter().map(|c| c.items.len()).sum::<usize>() + self.items.len()
    }

    /// Every item, categorised first, then the uncategorised ones.
    pub fn all_items(&self) -> impl Iterator<Item = &TemplateItem> {
        self.categories
            .iter()
            .flat_map(|c| c.items.iter())
            .chain(self.items.iter())
    }

    /// Refuse a pack this build cannot apply in full: another app's, or a
    /// newer format. Hand-written packs (format 0, no app) always pass.
    pub fn check_compatible(&self) -> Result<(), String> {
        check_pack_header(self.format, &self.app)
    }

    /// Resolve a template-relative or shelf image key to a file on disk.
    pub fn resolve_image(&self, image: &str) -> Option<PathBuf> {
        resolve_template_image(&self.id, image)
    }

    /// Resolve images relative to a downloaded/cached pack directory.
    pub fn resolve_image_in(&self, pack_dir: &Path, image: &str) -> Option<PathBuf> {
        resolve_template_image_in(pack_dir, &self.id, image)
    }
}

/// Where named templates live (dev: crate `assets/templates`; packaged:
/// `assets/templates` beside the exe / package root — same Windows + Linux).
/// `KHAYNJO_TEMPLATES_DIR` overrides.
pub fn templates_root() -> Option<PathBuf> {
    crate::menu_images::packaged_assets_dir("assets/templates", "KHAYNJO_TEMPLATES_DIR")
}

fn load_one(dir: &Path) -> Option<MenuTemplate> {
    load_from_dir(dir)
}

/// Same rule for `template.json` and `shop_defaults.json` headers.
pub fn check_pack_header(format: u32, app: &str) -> Result<(), String> {
    let app = app.trim();
    if !app.is_empty() && !app.eq_ignore_ascii_case(PACK_APP) {
        return Err(format!("This template is for {app}, not Khaynjo"));
    }
    if format > PACK_FORMAT {
        return Err("This template needs a newer Khaynjo — update the app first".into());
    }
    Ok(())
}

/// Load one `template.json` from a pack folder (bundled, cached, or fixture).
pub fn load_from_dir(dir: &Path) -> Option<MenuTemplate> {
    read_from_dir(dir).ok()
}

/// [`load_from_dir`] that says why a pack could not be read.
pub fn read_from_dir(dir: &Path) -> Result<MenuTemplate, String> {
    let path = dir.join("template.json");
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("template.json: {e}"))?;
    let mut tmpl: MenuTemplate =
        serde_json::from_str(&raw).map_err(|e| format!("template.json: {e}"))?;
    if tmpl.id.trim().is_empty() {
        tmpl.id = dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
    }
    if tmpl.title.trim().is_empty() {
        tmpl.title = menu_images::title_from_kebab(&tmpl.id);
    }
    Ok(tmpl)
}

/// Every client template on disk, sorted by title.
pub fn all() -> &'static [MenuTemplate] {
    static ALL: std::sync::OnceLock<Vec<MenuTemplate>> = std::sync::OnceLock::new();
    if let Some(v) = ALL.get() {
        return v.as_slice();
    }
    let loaded = load_all_templates();
    // Do not cache empty while Android is still unpacking the APK pack.
    if loaded.is_empty() && !menu_images::menu_assets_ready() {
        return &[];
    }
    ALL.get_or_init(|| loaded).as_slice()
}

fn load_all_templates() -> Vec<MenuTemplate> {
    let Some(root) = templates_root() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        if let Some(tmpl) = load_one(&dir) {
            out.push(tmpl);
        }
    }
    out.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));
    out
}

pub fn find(id: &str) -> Option<&'static MenuTemplate> {
    let id = id.trim();
    all().iter().find(|t| t.id == id)
}

/// Turn an image field into a readable file path (bundled templates root).
pub fn resolve_template_image(template_id: &str, image: &str) -> Option<PathBuf> {
    let root = templates_root()?.join(template_id);
    resolve_template_image_in(&root, template_id, image)
}

/// Resolve `shelf:…` against the photo shelf, else `images/…` under `pack_dir`.
pub fn resolve_template_image_in(pack_dir: &Path, _template_id: &str, image: &str) -> Option<PathBuf> {
    let image = image.trim();
    if image.is_empty() {
        return None;
    }
    if let Some(rel) = image.strip_prefix(SHELF_KEY_PREFIX) {
        // Prefer images already copied into the pack (self-contained remote packs).
        let local = pack_dir.join("items").join(rel);
        if local.is_file() {
            return Some(local);
        }
        let root = menu_images::catalogue_root()?;
        let path = root.join(rel);
        return path.is_file().then_some(path);
    }
    let path = if Path::new(image).is_absolute() {
        PathBuf::from(image)
    } else {
        pack_dir.join(image)
    };
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rajdhani_authoring() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalogue/private-src/rajdhani")
    }

    #[test]
    fn rajdhani_template_loads_with_priced_dishes() {
        let tmpl = load_from_dir(&rajdhani_authoring()).expect("rajdhani authoring pack");
        assert_eq!(tmpl.title, "Rajdhani Menu Template");
        assert!(tmpl.category_count() >= 8, "{}", tmpl.category_count());
        assert!(tmpl.item_count() >= 100, "{}", tmpl.item_count());
        let first = &tmpl.categories[0].items[0];
        assert!(!first.name.is_empty());
        assert!(!first.aliases.is_empty(), "English alias expected");
        assert!(
            first.name.chars().any(|c| !c.is_ascii()),
            "primary name should be Gujarati: {}",
            first.name
        );
        assert!(
            first.aliases.iter().any(|a| a.is_ascii()),
            "English alias expected: {:?}",
            first.aliases
        );
        assert!(first.price > 0 || first.name.to_lowercase().contains("paplet"));
        let path = tmpl
            .resolve_image(&first.image)
            .expect("first dish photo resolves via public shelf");
        assert!(path.extension().and_then(|e| e.to_str()) == Some("webp"));
    }

    #[test]
    fn every_rajdhani_image_resolves() {
        let tmpl = load_from_dir(&rajdhani_authoring()).unwrap();
        for cat in &tmpl.categories {
            if !cat.image.is_empty() {
                assert!(
                    tmpl.resolve_image(&cat.image).is_some(),
                    "category {} image {}",
                    cat.name,
                    cat.image
                );
            }
            for item in &cat.items {
                if item.image.is_empty() {
                    continue;
                }
                assert!(
                    tmpl.resolve_image(&item.image).is_some(),
                    "item {} image {}",
                    item.name,
                    item.image
                );
            }
        }
    }

    #[test]
    fn bundled_templates_exclude_private_rajdhani() {
        assert!(
            find("rajdhani").is_none(),
            "Rajdhani must not ship under assets/templates"
        );
    }
}

//! Named catalogue templates (`assets/templates/<id>/template.json`).
//!
//! Kirano ships `neighbourhood/` — a starter grocery list with Hindi
//! aliases, example prices, and `shelf:` photos. Discovery is automatic.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::menu_images::{self, SHELF_KEY_PREFIX};

pub const PACK_APP: &str = "kirano";
pub const PACK_FORMAT: u32 = 2;

/// Extra meta keys a pack may set besides [`crate::store::SHOP_PREF_KEYS`].
pub fn template_pref_extra_keys() -> &'static [&'static str] {
    &[
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
        "show_tax_chrome",
        "show_tax_invoice",
        "pay_methods",
    ]
}

pub fn check_pack_header(format: u32, app: &str) -> Result<(), String> {
    let app = app.trim();
    if !app.is_empty() && !app.eq_ignore_ascii_case(PACK_APP) {
        return Err(format!("This template is for {app}, not Kirano"));
    }
    if format > PACK_FORMAT {
        return Err("This template needs a newer Kirano — update the app first".into());
    }
    Ok(())
}

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

pub fn load_from_dir(dir: &Path) -> Option<MenuTemplate> {
    read_from_dir(dir).ok()
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuTemplate {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub blurb: String,
    #[serde(default)]
    pub categories: Vec<TemplateCategory>,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateItem {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Rupees (major units). `0` = unpriced / set at the counter.
    #[serde(default)]
    pub price: i64,
    #[serde(default)]
    pub image: String,
    /// Sell unit (`kg`, `g`, `L`, `ml`, `pc`). Empty = each.
    #[serde(default)]
    pub unit: String,
}

impl MenuTemplate {
    pub fn category_count(&self) -> usize {
        self.categories.len()
    }

    pub fn item_count(&self) -> usize {
        self.categories.iter().map(|c| c.items.len()).sum()
    }

    /// Resolve a template-relative or shelf image key to a file on disk.
    pub fn resolve_image(&self, image: &str) -> Option<PathBuf> {
        resolve_template_image(&self.id, image)
    }

    pub fn resolve_image_in(&self, pack_dir: &Path, image: &str) -> Option<PathBuf> {
        resolve_template_image_in(pack_dir, &self.id, image)
    }
}

/// Where named templates live (dev: crate `assets/templates`; packaged:
/// `assets/templates` beside the exe / package root — same Windows + Linux).
/// `KIRANO_TEMPLATES_DIR` overrides.
pub fn templates_root() -> Option<PathBuf> {
    crate::menu_images::packaged_assets_dir("assets/templates", "KIRANO_TEMPLATES_DIR")
}

fn load_one(dir: &Path) -> Option<MenuTemplate> {
    let path = dir.join("template.json");
    let raw = std::fs::read_to_string(&path).ok()?;
    let mut tmpl: MenuTemplate = serde_json::from_str(&raw).ok()?;
    if tmpl.id.trim().is_empty() {
        tmpl.id = dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
    }
    if tmpl.title.trim().is_empty() {
        tmpl.title = menu_images::title_from_kebab(&tmpl.id);
    }
    Some(tmpl)
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

/// Turn an image field into a readable file path.
pub fn resolve_template_image(template_id: &str, image: &str) -> Option<PathBuf> {
    let root = templates_root()?.join(template_id);
    resolve_template_image_in(&root, template_id, image)
}

pub fn resolve_template_image_in(
    pack_dir: &Path,
    _template_id: &str,
    image: &str,
) -> Option<PathBuf> {
    let image = image.trim();
    if image.is_empty() {
        return None;
    }
    if let Some(rel) = image.strip_prefix(SHELF_KEY_PREFIX) {
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

/// Pack dir for a bundled template id.
pub fn bundled_pack_dir(id: &str) -> Option<PathBuf> {
    let root = templates_root()?;
    let dir = root.join(id);
    dir.is_dir().then_some(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_grocery_starter_ships() {
        let ids: Vec<&str> = all().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["neighbourhood"]);
    }


}

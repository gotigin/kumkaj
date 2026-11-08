//! Named catalogue templates (`assets/templates/<id>/template.json`).
//!
//! Tabiyat ships without a starter catalogue. Drop a folder here only if a
//! named import is wanted later — discovery is automatic.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::menu_images::{self, SHELF_KEY_PREFIX};

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
    /// Rupees (major units). `0` = unpriced / season rate.
    #[serde(default)]
    pub price: i64,
    #[serde(default)]
    pub image: String,
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
}

/// Where named templates live (dev: crate `assets/templates`; packaged:
/// `assets/templates` beside the exe / package root — same Windows + Linux).
/// `TABIYAT_TEMPLATES_DIR` overrides.
pub fn templates_root() -> Option<PathBuf> {
    crate::menu_images::packaged_assets_dir("assets/templates", "TABIYAT_TEMPLATES_DIR")
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
    ALL.get_or_init(|| {
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
    })
}

pub fn find(id: &str) -> Option<&'static MenuTemplate> {
    let id = id.trim();
    all().iter().find(|t| t.id == id)
}

/// Turn an image field into a readable file path.
pub fn resolve_template_image(template_id: &str, image: &str) -> Option<PathBuf> {
    let image = image.trim();
    if image.is_empty() {
        return None;
    }
    if let Some(rel) = image.strip_prefix(SHELF_KEY_PREFIX) {
        let root = menu_images::catalogue_root()?;
        let path = root.join(rel);
        return path.is_file().then_some(path);
    }
    let root = templates_root()?.join(template_id);
    let path = if Path::new(image).is_absolute() {
        PathBuf::from(image)
    } else {
        root.join(image)
    };
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_shop_ships_without_starter_catalogue() {
        assert!(
            find("rajdhani").is_none(),
            "restaurant starter catalogues must not ship on this counter product"
        );
        assert!(all().iter().all(|t| t.id != "rajdhani"));
    }


}

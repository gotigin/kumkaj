//! Named client menu templates (`assets/templates/<id>/template.json`).
//!
//! Separate from the big photo shelf (`menu_images`): these are concrete
//! restaurant menus — primary `name` plus optional aliases (e.g. English),
//! listed prices, and photos (`shelf:…` or a file under the template).
//! Drop another folder beside `rajdhani/` and it appears in the app.

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

/// Where named templates live (dev: crate `assets/templates`; packaged: beside
/// the executable). `KHAYNJO_TEMPLATES_DIR` overrides both.
pub fn templates_root() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("KHAYNJO_TEMPLATES_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/templates");
    if manifest.is_dir() {
        return Some(manifest);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let beside = parent.join("assets/templates");
        if beside.is_dir() {
            return Some(beside);
        }
    }
    None
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
    fn rajdhani_template_loads_with_priced_dishes() {
        let tmpl = find("rajdhani").expect("rajdhani template ships with the app");
        assert_eq!(tmpl.title, "Rajdhani");
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
            .expect("first dish photo resolves");
        assert!(path.extension().and_then(|e| e.to_str()) == Some("webp"));
    }

    #[test]
    fn every_rajdhani_image_resolves() {
        let tmpl = find("rajdhani").unwrap();
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
}

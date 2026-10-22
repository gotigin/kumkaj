//! Menu item / party thumb helpers for Khaynjo.
//!
//! No built-in art catalogue (that belonged to food-counter products). When a
//! name is set and no photo is chosen, UI shows **name initials**:
//! - one word → first two letters (`Dosa` → `DO`)
//! - two+ words → first letter of the first two (`Masala Dosa` → `MD`)

use std::path::{Path, PathBuf};

/// Initials for a missing photo when `name` is non-empty.
///
/// Empty / whitespace-only names yield `""` (caller shows no avatar text).
pub fn name_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().filter(|w| !w.is_empty()).collect();
    match words.as_slice() {
        [] => String::new(),
        [one] => {
            let mut chars = one.chars();
            let a = chars.next().map(char_upper).unwrap_or_default();
            let b = chars.next().map(char_upper).unwrap_or_default();
            format!("{a}{b}")
        }
        [first, second, ..] => {
            let a = first.chars().next().map(char_upper).unwrap_or_default();
            let b = second.chars().next().map(char_upper).unwrap_or_default();
            format!("{a}{b}")
        }
    }
}

fn char_upper(c: char) -> String {
    c.to_uppercase().collect()
}

pub fn describe_image(key: Option<&str>, name: &str) -> String {
    if has_photo_key(key) {
        return "Custom photo".into();
    }
    let initials = name_initials(name);
    if initials.is_empty() {
        "No image — type a name for initials, or pick a photo".into()
    } else {
        format!("Image: {initials} (from name)")
    }
}

/// True when `key` points at a real photo (file or pending pick).
pub fn has_photo_key(key: Option<&str>) -> bool {
    match key {
        None | Some("") => false,
        Some(k) if k.starts_with("pending_custom:") => true,
        Some(k) if k.starts_with("custom:") => true,
        Some(_) => false,
    }
}

/// Resolve a custom photo path; catalogue keys are not supported.
pub fn resolve_image_path(key: Option<&str>, _name: &str, _dir: &Path) -> PathBuf {
    match key {
        Some(k) if k.starts_with("custom:") => PathBuf::from(k.trim_start_matches("custom:")),
        Some(k) if k.starts_with("pending_custom:") => {
            PathBuf::from(k.trim_start_matches("pending_custom:"))
        }
        _ => PathBuf::new(),
    }
}

// ---------------------------------------------------------------------------
// Provided items catalogue (apps/khaynjo/assets/items/<category>/<dish>.webp)
//
// A browsing shelf only: picking a dish copies its name + photo into a
// brand-new item with a fresh UUID. The folder an item was found in is
// never forced on the owner as the item's category — he files the new
// item under any category he chooses (or none).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogueCategory {
    /// Folder name on disk (kebab-case), e.g. `south-indian`.
    pub key: String,
    /// Display label, e.g. `South Indian`.
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogueItem {
    /// Display dish name, e.g. `Masala Dosa`.
    pub name: String,
    pub path: PathBuf,
}

/// Where the provided items live. Dev runs read the workspace copy via
/// the compile-time manifest dir; packaged builds ship `assets/items`
/// next to the executable. `KHAYNJO_CATALOGUE_DIR` overrides both.
pub fn catalogue_root() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("KHAYNJO_CATALOGUE_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/items");
    if manifest.is_dir() {
        return Some(manifest);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let beside = parent.join("assets/items");
        if beside.is_dir() {
            return Some(beside);
        }
    }
    None
}

/// `masala-dosa` / `street-food-chaat` → `Masala Dosa` / `Street Food Chaat`.
pub fn title_from_kebab(s: &str) -> String {
    s.split('-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => {
                    let mut out: String = first.to_uppercase().collect();
                    out.push_str(chars.as_str());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn catalogue_categories() -> Vec<CatalogueCategory> {
    let Some(root) = catalogue_root() else {
        return Vec::new();
    };
    let mut out: Vec<CatalogueCategory> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let key = entry.file_name().to_string_lossy().into_owned();
                out.push(CatalogueCategory {
                    label: title_from_kebab(&key),
                    key,
                });
            }
        }
    }
    out.sort_by(|a, b| a.label.cmp(&b.label));
    out
}

pub fn catalogue_items(category_key: &str) -> Vec<CatalogueItem> {
    let Some(root) = catalogue_root() else {
        return Vec::new();
    };
    // The key comes from our own folder listing; refuse path tricks.
    if category_key.contains('/') || category_key.contains('\\') || category_key.contains("..") {
        return Vec::new();
    }
    let dir = root.join(category_key);
    let mut out: Vec<CatalogueItem> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("webp") {
                let stem = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                out.push(CatalogueItem {
                    name: title_from_kebab(&stem),
                    path,
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_titles_read_as_dish_names() {
        assert_eq!(title_from_kebab("masala-dosa"), "Masala Dosa");
        assert_eq!(title_from_kebab("street-food-chaat"), "Street Food Chaat");
        assert_eq!(title_from_kebab("thali"), "Thali");
        assert_eq!(title_from_kebab(""), "");
    }

    #[test]
    fn catalogue_items_rejects_path_tricks() {
        assert!(catalogue_items("../items").is_empty());
        assert!(catalogue_items("a/b").is_empty());
    }

    #[test]
    fn initials_one_word_uses_two_letters() {
        assert_eq!(name_initials("Express"), "EX");
        assert_eq!(name_initials("a"), "A");
        assert_eq!(name_initials("  tea  "), "TE");
    }

    #[test]
    fn initials_two_words_use_first_letters() {
        assert_eq!(name_initials("Masala Dosa"), "MD");
        assert_eq!(name_initials("masala  dosa"), "MD");
    }

    #[test]
    fn initials_empty_name() {
        assert_eq!(name_initials(""), "");
        assert_eq!(name_initials("   "), "");
    }

    #[test]
    fn only_custom_keys_count_as_photos() {
        assert!(!has_photo_key(None));
        assert!(!has_photo_key(Some("chicken")));
        assert!(has_photo_key(Some("custom:/tmp/x.webp")));
    }
}

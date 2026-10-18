//! Item artwork helpers for Kumkaj Courier.
//!
//! There is no built-in dish-art catalogue. When a name is set and no photo is
//! chosen, UI shows **name initials** instead of an empty black thumb:
//! - one word → first two letters (`Chicken` → `CH`)
//! - two+ words → first letter of the first two (`Chicken Biryani` → `CB`)

use makepad_widgets::Cx;
use std::path::{Path, PathBuf};

/// No built-in dish art in the courier product.
pub const MENU_IMAGE_KEYS: &[&str] = &[];

/// Generic fallback key (unused when keys are empty).
pub const PLACEHOLDER_KEY: &str = "placeholder";

pub fn is_builtin_key(_key: &str) -> bool {
    false
}

pub fn detect_image_key(_name: &str) -> &'static str {
    PLACEHOLDER_KEY
}

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
    if key.map(|k| !k.is_empty() && k != PLACEHOLDER_KEY)
        .unwrap_or(false)
    {
        return "Custom photo".into();
    }
    let initials = name_initials(name);
    if initials.is_empty() {
        "No image — type a name for initials, or pick a photo".into()
    } else {
        format!("Image: {initials} (from name)")
    }
}

pub fn next_image_key(_current: Option<&str>) -> Option<&'static str> {
    None
}

pub fn menu_dir_candidates(exe_dir: &Path) -> [PathBuf; 3] {
    [
        exe_dir.join("kumkaj-courier").join("resources").join("menu"),
        exe_dir.join("menu"),
        exe_dir.join("resources").join("menu"),
    ]
}

pub fn bundled_menu_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        for candidate in menu_dir_candidates(parent) {
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/menu")
}

pub fn bundled_image_path(key: &str) -> PathBuf {
    bundled_menu_dir().join(format!("{key}.webp"))
}

pub fn bundled_dependency_path(key: &str) -> String {
    format!("kumkaj-courier/resources/menu/{key}.webp")
}

pub fn resolve_image_path(_key: Option<&str>, _name: &str, _dir: &Path) -> PathBuf {
    PathBuf::new()
}

pub fn read_image_bytes(_cx: &Cx, _path: &Path) -> Option<Vec<u8>> {
    None
}

/// True when `key` points at a real photo (file or pending pick), not auto/placeholder.
pub fn has_photo_key(key: Option<&str>) -> bool {
    match key {
        None | Some("") => false,
        Some(PLACEHOLDER_KEY) => false,
        Some(k) if k.starts_with("pending_custom:") => true,
        Some(k) if k.starts_with("custom:") => true,
        Some(_) => false, // no built-in catalogue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn courier_has_no_menu_art_catalogue() {
        assert!(MENU_IMAGE_KEYS.is_empty());
        assert_eq!(detect_image_key("Chicken Biryani"), PLACEHOLDER_KEY);
    }

    #[test]
    fn initials_one_word_uses_two_letters() {
        assert_eq!(name_initials("Chicken"), "CH");
        assert_eq!(name_initials("a"), "A");
        assert_eq!(name_initials("  tea  "), "TE");
    }

    #[test]
    fn initials_two_words_use_first_letters() {
        assert_eq!(name_initials("Chicken Biryani"), "CB");
        assert_eq!(name_initials("express  parcel"), "EP");
    }

    #[test]
    fn initials_empty_name() {
        assert_eq!(name_initials(""), "");
        assert_eq!(name_initials("   "), "");
    }
}

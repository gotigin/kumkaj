//! Rate / party thumb helpers for Kumkaj Courier.
//!
//! No built-in art catalogue (that belonged to food-counter products). When a
//! name is set and no photo is chosen, UI shows **name initials**:
//! - one word → first two letters (`Express` → `EX`)
//! - two+ words → first letter of the first two (`Surface Zone` → `SZ`)

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_one_word_uses_two_letters() {
        assert_eq!(name_initials("Express"), "EX");
        assert_eq!(name_initials("a"), "A");
        assert_eq!(name_initials("  tea  "), "TE");
    }

    #[test]
    fn initials_two_words_use_first_letters() {
        assert_eq!(name_initials("Surface Zone"), "SZ");
        assert_eq!(name_initials("express  parcel"), "EP");
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

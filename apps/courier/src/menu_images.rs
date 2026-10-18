//! Built-in item artwork + name → image matching.
//!
//! Food keys remain available for food entities; the fallback
//! [`PLACEHOLDER_KEY`] is the generic Kumkaj mark (not a dish photo).

use makepad_widgets::Cx;
use std::path::{Path, PathBuf};

/// Optional built-in art keys under `resources/menu/{key}.webp`.
/// Food-named keys are for food entities only; unknown items use [`PLACEHOLDER_KEY`].
pub const MENU_IMAGE_KEYS: &[&str] = &[
    "bakery",
    "biryani",
    "brownie",
    "burger",
    "chai",
    "coffee",
    "curry",
    "dal",
    "dessert",
    "dosa",
    "fried_rice",
    "fries",
    "gulab_jamun",
    "ice_cream",
    "idli",
    "juice",
    "lassi",
    "lime_soda",
    "mocktail",
    "naan",
    "noodles",
    "paneer_tikka",
    "pasta",
    "pizza",
    "rice",
    "roti",
    "salad",
    "samosa",
    "sandwich",
    "snacks",
    "soup",
    "spring_rolls",
    "upma",
];

/// Generic fallback art: Kumkaj mark (business OS), never a vertical-specific photo.
pub const PLACEHOLDER_KEY: &str = "placeholder";

/// Extra phrase → key pairs (common food aliases for food entities).
const ALIASES: &[(&str, &str)] = &[
    ("masala chai", "chai"),
    ("filter coffee", "coffee"),
    ("fresh lime soda", "lime_soda"),
    ("lime soda", "lime_soda"),
    ("mango lassi", "lassi"),
    ("idli sambar", "idli"),
    ("masala dosa", "dosa"),
    ("plain dosa", "dosa"),
    ("vegetable upma", "upma"),
    ("veg upma", "upma"),
    ("vegetable samosa", "samosa"),
    ("veg samosa", "samosa"),
    ("paneer tikka", "paneer_tikka"),
    ("french fries", "fries"),
    ("veg spring rolls", "spring_rolls"),
    ("spring roll", "spring_rolls"),
    ("paneer butter masala", "curry"),
    ("butter chicken", "curry"),
    ("dal tadka", "dal"),
    ("dal fry", "dal"),
    ("veg kadai", "curry"),
    ("chicken curry", "curry"),
    ("tandoori roti", "roti"),
    ("butter naan", "naan"),
    ("garlic naan", "naan"),
    ("veg biryani", "biryani"),
    ("chicken biryani", "biryani"),
    ("mutton biryani", "biryani"),
    ("jeera rice", "rice"),
    ("veg fried rice", "fried_rice"),
    ("fried rice", "fried_rice"),
    ("gulab jamun", "gulab_jamun"),
    ("ice cream", "ice_cream"),
    ("brownie", "brownie"),
    ("tea", "chai"),
    ("coffee", "coffee"),
    ("pizza", "pizza"),
    ("burger", "burger"),
    ("pasta", "pasta"),
    ("noodles", "noodles"),
    ("sandwich", "sandwich"),
    ("salad", "salad"),
    ("soup", "soup"),
    ("juice", "juice"),
    ("lassi", "lassi"),
    ("mocktail", "mocktail"),
    ("soda", "lime_soda"),
    ("dessert", "dessert"),
    ("sweet", "dessert"),
    ("cake", "bakery"),
    ("bread", "bakery"),
    ("bakery", "bakery"),
    ("snack", "snacks"),
    ("chaat", "snacks"),
    ("tikka", "paneer_tikka"),
    ("roll", "spring_rolls"),
];

/// Candidate menu-art dirs beside a packaged desktop binary (first hit wins).
///
/// `MAKEPAD_PACKAGE_DIR=.` layout puts files at `kumkaj/resources/menu/` next to
/// the exe — not a bare `menu/` folder. Looking only for `menu/` / `resources/menu`
/// after that packaging change left Windows builds with blank item thumbs.
pub fn menu_dir_candidates(exe_dir: &Path) -> [PathBuf; 3] {
    [
        exe_dir.join("kumkaj").join("resources").join("menu"),
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

/// APK / packaged dependency path for a built-in menu image.
/// Android assets live at `makepad/kumkaj/resources/menu/{key}.webp`.
pub fn bundled_dependency_path(key: &str) -> String {
    format!("kumkaj/resources/menu/{key}.webp")
}

/// Read image bytes for a resolved filesystem path, falling back to packaged
/// Android/iOS assets when the desktop source tree is not on device.
pub fn read_image_bytes(cx: &Cx, path: &Path) -> Option<Vec<u8>> {
    if let Ok(bytes) = std::fs::read(path) {
        return Some(bytes);
    }
    // Cross-compiled Windows resolves CARGO_MANIFEST_DIR to a Linux path that
    // does not exist on the PC — retry the real dir beside the .exe.
    if let Some(name) = path.file_name() {
        let beside = bundled_menu_dir().join(name);
        if beside.as_path() != path
            && let Ok(bytes) = std::fs::read(&beside)
        {
            return Some(bytes);
        }
    }
    let path_str = path.to_string_lossy();
    // .../resources/menu/chai.webp → kumkaj/resources/menu/chai.webp
    if let Some(idx) = path_str.find("resources/menu/") {
        let rel = &path_str[idx..];
        let dep = format!("kumkaj/{rel}");
        if let Ok(data) = cx.get_dependency(&dep) {
            return Some((*data).clone());
        }
    }
    // Filename-only fallback (bundled_image_path that doesn't exist as a file)
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && name.ends_with(".webp")
    {
        let key = name.trim_end_matches(".webp");
        if is_builtin_key(key)
            && let Ok(data) = cx.get_dependency(&bundled_dependency_path(key))
        {
            return Some((*data).clone());
        }
    }
    // Last resort: placeholder from disk, then APK/assets dependency table.
    let placeholder_disk = bundled_image_path(PLACEHOLDER_KEY);
    if let Ok(bytes) = std::fs::read(&placeholder_disk) {
        return Some(bytes);
    }
    cx.get_dependency(&bundled_dependency_path(PLACEHOLDER_KEY))
        .ok()
        .map(|d| (*d).clone())
}

pub fn is_builtin_key(key: &str) -> bool {
    key == PLACEHOLDER_KEY || MENU_IMAGE_KEYS.contains(&key)
}

pub fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_us = false;
    for ch in name.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_us = false;
        } else if (c.is_whitespace() || c == '-' || c == '_') && !prev_us && !out.is_empty() {
            out.push('_');
            prev_us = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

/// Pick the best built-in key for a free-text item name.
pub fn detect_image_key(name: &str) -> &'static str {
    let lower = name.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return PLACEHOLDER_KEY;
    }

    // Longest alias phrase first.
    let mut best: Option<&str> = None;
    let mut best_len = 0usize;
    for (phrase, key) in ALIASES {
        if phrase.len() > best_len && lower.contains(phrase) && is_builtin_key(key) {
            best = Some(*key);
            best_len = phrase.len();
        }
    }
    if let Some(key) = best {
        return key;
    }

    let slug = slugify(&lower);
    if MENU_IMAGE_KEYS.contains(&slug.as_str()) {
        return MENU_IMAGE_KEYS
            .iter()
            .find(|k| **k == slug)
            .copied()
            .unwrap_or(PLACEHOLDER_KEY);
    }

    // Token / substring hit against keys (longest key wins).
    let mut best_key = PLACEHOLDER_KEY;
    let mut best_key_len = 0usize;
    for key in MENU_IMAGE_KEYS {
        if key.len() > best_key_len && (slug.contains(key) || lower.contains(&key.replace('_', " ")))
        {
            best_key = key;
            best_key_len = key.len();
        }
    }
    if best_key_len > 0 {
        return best_key;
    }

    PLACEHOLDER_KEY
}

/// Resolve stored image ref + name into a path used for loading.
///
/// The returned path is a **logical** key path for built-ins. On desktop it is a
/// real file under `resources/menu/`. On Android that file does not exist on disk
/// — `read_image_bytes` loads the same key from APK assets. Do **not** collapse
/// missing-on-disk built-ins to placeholder here, or ◀/▶ only changes the label
/// while every preview stays on placeholder.
///
/// - `None` / empty → auto-detect from name (or placeholder)
/// - builtin key → `…/resources/menu/{key}.webp` (logical)
/// - `custom:filename` → `{item_images_dir}/filename`
/// - absolute/relative path → as given if it exists
pub fn resolve_image_path(
    image_key: Option<&str>,
    name: &str,
    item_images_dir: &Path,
) -> PathBuf {
    match image_key.map(str::trim).filter(|s| !s.is_empty()) {
        Some(key) if key.starts_with("custom:") => {
            let file = &key["custom:".len()..];
            let path = item_images_dir.join(file);
            if path.is_file() {
                path
            } else {
                bundled_image_path(PLACEHOLDER_KEY)
            }
        }
        Some(key) if is_builtin_key(key) => bundled_image_path(key),
        Some(path_str) => {
            let path = PathBuf::from(path_str);
            if path.is_file() {
                path
            } else {
                bundled_image_path(detect_image_key(name))
            }
        }
        None => bundled_image_path(detect_image_key(name)),
    }
}

pub fn describe_image(image_key: Option<&str>, name: &str) -> String {
    match image_key.map(str::trim).filter(|s| !s.is_empty()) {
        Some(key) if key.starts_with("custom:") => "Custom photo".into(),
        Some(key) if key == PLACEHOLDER_KEY => "Image: Kumkaj".into(),
        Some(key) if is_builtin_key(key) => format!("Image: {key}"),
        Some(_) => "Custom path".into(),
        None => {
            let detected = detect_image_key(name);
            if detected == PLACEHOLDER_KEY {
                "Image: auto (Kumkaj)".into()
            } else {
                format!("Image: auto → {detected}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod resolve_tests {
        use super::*;
        use std::path::Path;

        #[test]
        fn builtin_keys_keep_key_path_even_if_file_missing() {
            // Android has no source-tree files; path must still encode the key so
            // read_image_bytes can load APK assets per dish.
            let dir = Path::new("/no/such/item_images");
            let path = resolve_image_path(Some("biryani"), "x", dir);
            assert!(
                path.to_string_lossy().ends_with("biryani.webp"),
                "got {}",
                path.display()
            );
            let auto = resolve_image_path(None, "Chicken Biryani", dir);
            assert!(
                auto.to_string_lossy().ends_with("biryani.webp"),
                "got {}",
                auto.display()
            );
        }
    }

    #[test]
    fn detects_common_dishes() {
        assert_eq!(detect_image_key("Chicken Biryani"), "biryani");
        assert_eq!(detect_image_key("Masala chai"), "chai");
        assert_eq!(detect_image_key("Garlic naan"), "naan");
        assert_eq!(detect_image_key("Random widget"), PLACEHOLDER_KEY);
    }

    #[test]
    fn packaged_layout_prefers_kumkaj_resources_menu() {
        let exe_dir = Path::new("/opt/kumkaj-dist");
        let [makepad, legacy, alt] = menu_dir_candidates(exe_dir);
        assert_eq!(makepad, PathBuf::from("/opt/kumkaj-dist/kumkaj/resources/menu"));
        assert_eq!(legacy, PathBuf::from("/opt/kumkaj-dist/menu"));
        assert_eq!(alt, PathBuf::from("/opt/kumkaj-dist/resources/menu"));
    }
}

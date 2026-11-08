//! Menu item / party thumb helpers for Kirano, and the bundled grocery shelf
//! (`assets/items/<folder>/<item>.webp`). Photos are WebP only.
//!
//! When a name is set and no photo is chosen, UI shows **name initials**:
//! - one word → first two letters (`Atta` → `AT`)
//! - two+ words → first letter of the first two (`Cooking Oil` → `CO`)

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
        // A photo from the bundled catalogue, referenced rather than copied
        // (the whole template is 1,400+ photos).
        Some(k) if k.starts_with(SHELF_KEY_PREFIX) => catalogue_root()
            .map(|root| root.join(k.trim_start_matches(SHELF_KEY_PREFIX)))
            .unwrap_or_default(),
        Some(k) if k.starts_with("custom:") => PathBuf::from(k.trim_start_matches("custom:")),
        Some(k) if k.starts_with("pending_custom:") => {
            PathBuf::from(k.trim_start_matches("pending_custom:"))
        }
        _ => PathBuf::new(),
    }
}

// ---------------------------------------------------------------------------
// Optional item photo shelf (apps/kirano/assets/items/<folder>/<item>.webp)
//
// Bundled, read-only art the item form searches: picking an entry copies its
// name + photo into the form for a brand-new item with a fresh UUID. The
// folder an entry lives in is suggested as the item's category — prefilled,
// editable, never overriding a category the owner typed.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShelfDish {
    /// Display item name, e.g. `Atta 5kg`.
    pub name: String,
    /// Display label of the shelf folder, e.g. `Staples`.
    pub folder: String,
    pub path: PathBuf,
}

/// Packaged `assets/…` lookup — same on Windows, Linux, and portable folders.
/// Tries: `env_key` (if set), cargo manifest (dev), beside the exe, one
/// level up (Linux `bin/` layout), `MAKEPAD_PACKAGE_DIR`, then cwd.
pub(crate) fn packaged_assets_dir(rel: &str, env_key: &str) -> Option<PathBuf> {
    if let Ok(dir) = std::env::var(env_key) {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    if manifest.is_dir() {
        return Some(manifest);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let beside = parent.join(rel);
            if beside.is_dir() {
                return Some(beside);
            }
            if let Some(grand) = parent.parent() {
                let up = grand.join(rel);
                if up.is_dir() {
                    return Some(up);
                }
            }
        }
    }
    if let Ok(pkg) = std::env::var("MAKEPAD_PACKAGE_DIR") {
        let p = PathBuf::from(pkg).join(rel);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(rel);
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

/// Where the provided items live. Dev runs read the workspace copy via
/// the compile-time manifest dir; packaged builds ship `assets/items`
/// next to the executable (Windows MSI / portable, Linux package root).
/// `KIRANO_CATALOGUE_DIR` overrides.
pub fn catalogue_root() -> Option<PathBuf> {
    packaged_assets_dir("assets/items", "KIRANO_CATALOGUE_DIR")
}

/// Image keys that point into the bundled catalogue: `shelf:<folder>/<file>`.
pub const SHELF_KEY_PREFIX: &str = "shelf:";

/// The image key for a catalogue photo (`shelf:staples/atta-5kg.webp`).
pub fn shelf_key(path: &Path) -> Option<String> {
    let root = catalogue_root()?;
    let rel = path.strip_prefix(&root).ok()?;
    Some(format!("{SHELF_KEY_PREFIX}{}", rel.to_string_lossy().replace('\\', "/")))
}

/// `atta-5kg` / `cooking-oil` → `Atta 5kg` / `Cooking Oil`.
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

/// Every photo on the shelf, sorted by folder then name. Scanned once per run
/// (it is scanned once and never changes while the app is open).
pub fn shelf() -> &'static [ShelfDish] {
    static SHELF: std::sync::OnceLock<Vec<ShelfDish>> = std::sync::OnceLock::new();
    SHELF.get_or_init(|| catalogue_root().map(|r| load_shelf(&r)).unwrap_or_default())
}

/// Scan `root/<folder>/<item>.webp`.
pub fn load_shelf(root: &Path) -> Vec<ShelfDish> {
    let mut out = Vec::new();
    let Ok(folders) = std::fs::read_dir(root) else {
        return out;
    };
    for folder in folders.flatten() {
        let dir = folder.path();
        if !dir.is_dir() {
            continue;
        }
        let label = title_from_kebab(&folder.file_name().to_string_lossy());
        let Ok(files) = std::fs::read_dir(&dir) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            if path.extension().and_then(|e| e.to_str()) != Some("webp") {
                continue;
            }
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push(ShelfDish {
                name: title_from_kebab(&stem),
                folder: label.clone(),
                path,
            });
        }
    }
    out.sort_by(|a, b| a.folder.cmp(&b.folder).then_with(|| a.name.cmp(&b.name)));
    out
}

/// A shelf folder offered as a category: its name and one of its items'
/// photos (the first by name), used the way an item's shelf photo is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShelfFolder {
    pub label: String,
    pub photo: PathBuf,
}

/// The shelf's folders in order, one entry each.
pub fn shelf_folders(items: &[ShelfDish]) -> Vec<ShelfFolder> {
    let mut out: Vec<ShelfFolder> = Vec::new();
    for d in items {
        if out.last().map(|f| f.label != d.folder).unwrap_or(true)
            && !out.iter().any(|f| f.label == d.folder)
        {
            out.push(ShelfFolder {
                label: d.folder.clone(),
                photo: d.path.clone(),
            });
        }
    }
    out
}

fn fold(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// How well `text` answers `query` — lower is better, `None` is no match.
/// Case- and spacing-insensitive: 0 exact, 1 starts with it, 2 a word starts
/// with it, 3 contains it, 4 contains every typed word (any order).
pub fn match_rank(text: &str, query: &str) -> Option<u8> {
    let q = fold(query);
    if q.is_empty() {
        return None;
    }
    let t = fold(text);
    if t == q {
        Some(0)
    } else if t.starts_with(&q) {
        Some(1)
    } else if t.split(' ').any(|w| w.starts_with(&q)) {
        Some(2)
    } else if t.contains(&q) {
        Some(3)
    } else if q.split(' ').all(|w| t.contains(w)) {
        Some(4)
    } else {
        None
    }
}

/// Shelf entries for the item picker, as indexes into `items`. A blank
/// query browses the whole shelf (folder, then name). Otherwise best
/// matches first: by item name, then entries whose shelf folder matches
/// (typing `atta` or `staples` finds that whole shelf too).
pub fn search_shelf(items: &[ShelfDish], query: &str) -> Vec<usize> {
    if fold(query).is_empty() {
        return (0..items.len()).collect();
    }
    let mut hits: Vec<(u8, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
            match_rank(&d.name, query)
                .or_else(|| match_rank(&d.folder, query).map(|r| r + 10))
                .map(|r| (r, i))
        })
        .collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| items[a.1].name.cmp(&items[b.1].name)));
    hits.into_iter().map(|(_, i)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_titles_read_as_item_names() {
        assert_eq!(title_from_kebab("atta-5kg"), "Atta 5kg");
        assert_eq!(title_from_kebab("cooking-oil"), "Cooking Oil");
        assert_eq!(title_from_kebab("soap"), "Soap");
        assert_eq!(title_from_kebab(""), "");
    }

    #[test]
    fn match_rank_orders_exact_prefix_word_substring_words() {
        assert_eq!(match_rank("Cooking Oil", "cooking  oil"), Some(0));
        assert_eq!(match_rank("Cooking Oil", "COO"), Some(1));
        assert_eq!(match_rank("Cooking Oil", "oil"), Some(2));
        assert_eq!(match_rank("Cooking Oil", "okin"), Some(3));
        assert_eq!(match_rank("Basmati Rice", "rice basmati"), Some(4));
        assert_eq!(match_rank("Cooking Oil", "soap"), None);
        assert_eq!(match_rank("Cooking Oil", "   "), None);
    }

    fn dish(name: &str, folder: &str) -> ShelfDish {
        ShelfDish {
            name: name.into(),
            folder: folder.into(),
            path: PathBuf::from(format!("{folder}/{name}.webp")),
        }
    }

    #[test]
    fn search_ranks_names_before_folders_and_blank_browses_all() {
        let shelf = vec![
            dish("Sona Rice", "Staples"),
            dish("Rice Bran Oil", "Oils"),
            dish("Brown Rice", "Staples"),
            dish("Soap", "Personal Care"),
        ];
        let names = |q: &str| -> Vec<&str> {
            search_shelf(&shelf, q).into_iter().map(|i| shelf[i].name.as_str()).collect()
        };
        // Prefix beat (Rice Bran…) ranks above mid-name word hits (Brown Rice).
        assert_eq!(names("rice"), vec!["Rice Bran Oil", "Brown Rice", "Sona Rice"]);
        // Only the folder matches: the whole shelf folder is offered.
        assert_eq!(names("personal care"), vec!["Soap"]);
        assert_eq!(names("pizza"), Vec::<&str>::new());
        assert_eq!(search_shelf(&shelf, " ").len(), 4);
    }

    #[test]
    fn each_shelf_folder_offers_one_photo() {
        let shelf = vec![
            dish("Atta 1kg", "Staples"),
            dish("Atta 5kg", "Staples"),
            dish("Soap", "Personal Care"),
        ];
        let folders = shelf_folders(&shelf);
        assert_eq!(folders.len(), 2);
        assert_eq!(folders[0].label, "Staples");
        assert_eq!(folders[0].photo, shelf[0].path);
        assert_eq!(folders[1].label, "Personal Care");
    }

    #[test]
    fn shelf_scan_reads_folders_and_skips_other_files() {
        let root = std::env::temp_dir().join(format!("kirano-shelf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("staples")).unwrap();
        std::fs::write(root.join("staples/atta-5kg.webp"), b"x").unwrap();
        std::fs::write(root.join("staples/notes.txt"), b"x").unwrap();
        let shelf = load_shelf(&root);
        assert_eq!(shelf, vec![ShelfDish {
            name: "Atta 5kg".into(),
            folder: "Staples".into(),
            path: root.join("staples/atta-5kg.webp"),
        }]);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn initials_one_word_uses_two_letters() {
        assert_eq!(name_initials("Express"), "EX");
        assert_eq!(name_initials("a"), "A");
        assert_eq!(name_initials("  tea  "), "TE");
    }

    #[test]
    fn initials_two_words_use_first_letters() {
        assert_eq!(name_initials("Cooking Oil"), "CO");
        assert_eq!(name_initials("cooking  oil"), "CO");
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

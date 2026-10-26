//! Menu item / party thumb helpers for Khaynjo, and the bundled dishes shelf
//! the item picker searches (see below).
//!
//! When a name is set and no photo is chosen, UI shows **name initials**:
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
// Provided dishes shelf (apps/khaynjo/assets/items/<folder>/<dish>.webp)
//
// Bundled, read-only art the item form searches: picking a dish copies its
// name + photo into the form for a brand-new item with a fresh UUID. The
// folder a dish lives in is suggested as the item's category — prefilled,
// editable, never overriding a category the owner typed.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShelfDish {
    /// Display dish name, e.g. `Masala Dosa`.
    pub name: String,
    /// Display label of the shelf folder, e.g. `South Indian`.
    pub folder: String,
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

/// Image keys that point into the bundled catalogue: `shelf:<folder>/<file>`.
pub const SHELF_KEY_PREFIX: &str = "shelf:";

/// The image key for a catalogue photo (`shelf:south-indian/masala-dosa.webp`).
pub fn shelf_key(path: &Path) -> Option<String> {
    let root = catalogue_root()?;
    let rel = path.strip_prefix(&root).ok()?;
    Some(format!("{SHELF_KEY_PREFIX}{}", rel.to_string_lossy().replace('\\', "/")))
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

/// Every dish on the shelf, sorted by folder then name. Scanned once per run
/// (it is bundled and never changes while the app is open).
pub fn shelf() -> &'static [ShelfDish] {
    static SHELF: std::sync::OnceLock<Vec<ShelfDish>> = std::sync::OnceLock::new();
    SHELF.get_or_init(|| catalogue_root().map(|r| load_shelf(&r)).unwrap_or_default())
}

/// Scan `root/<folder>/<dish>.webp`.
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

/// A shelf folder offered as a category: its name and one of its dishes'
/// photos (the first by name), used the way an item's shelf photo is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShelfFolder {
    pub label: String,
    pub photo: PathBuf,
}

/// The shelf's folders in order, one entry each.
pub fn shelf_folders(dishes: &[ShelfDish]) -> Vec<ShelfFolder> {
    let mut out: Vec<ShelfFolder> = Vec::new();
    for d in dishes {
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

/// Shelf dishes for the item picker, as indexes into `dishes`. A blank
/// query browses the whole shelf (folder, then name). Otherwise best
/// matches first: by dish name, then dishes whose shelf folder matches
/// (typing `biryani` or `south indian` finds that whole shelf too).
pub fn search_shelf(dishes: &[ShelfDish], query: &str) -> Vec<usize> {
    if fold(query).is_empty() {
        return (0..dishes.len()).collect();
    }
    let mut hits: Vec<(u8, usize)> = dishes
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
            match_rank(&d.name, query)
                .or_else(|| match_rank(&d.folder, query).map(|r| r + 10))
                .map(|r| (r, i))
        })
        .collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| dishes[a.1].name.cmp(&dishes[b.1].name)));
    hits.into_iter().map(|(_, i)| i).collect()
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
    fn match_rank_orders_exact_prefix_word_substring_words() {
        assert_eq!(match_rank("Masala Dosa", "masala  dosa"), Some(0));
        assert_eq!(match_rank("Masala Dosa", "MAS"), Some(1));
        assert_eq!(match_rank("Masala Dosa", "dos"), Some(2));
        assert_eq!(match_rank("Masala Dosa", "sala"), Some(3));
        assert_eq!(match_rank("Chicken Biryani", "biryani chicken"), Some(4));
        assert_eq!(match_rank("Masala Dosa", "idli"), None);
        assert_eq!(match_rank("Masala Dosa", "   "), None);
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
            dish("Ghee Rice", "Indian Rice"),
            dish("Rice Kheer", "Desserts"),
            dish("Fried Rice", "Chinese"),
            dish("Idli", "South Indian"),
        ];
        let names = |q: &str| -> Vec<&str> {
            search_shelf(&shelf, q).into_iter().map(|i| shelf[i].name.as_str()).collect()
        };
        assert_eq!(names("rice"), vec!["Rice Kheer", "Fried Rice", "Ghee Rice"]);
        // Only the folder matches: the whole shelf folder is offered.
        assert_eq!(names("south indian"), vec!["Idli"]);
        assert_eq!(names("pizza"), Vec::<&str>::new());
        assert_eq!(search_shelf(&shelf, " ").len(), 4);
    }

    #[test]
    fn each_shelf_folder_offers_one_photo() {
        let shelf = vec![
            dish("Bread Pakora", "Bakery Cafe"),
            dish("Croissant", "Bakery Cafe"),
            dish("Idli", "South Indian"),
        ];
        let folders = shelf_folders(&shelf);
        assert_eq!(folders.len(), 2);
        assert_eq!(folders[0].label, "Bakery Cafe");
        assert_eq!(folders[0].photo, shelf[0].path);
        assert_eq!(folders[1].label, "South Indian");
    }

    #[test]
    fn shelf_scan_reads_folders_and_skips_other_files() {
        let root = std::env::temp_dir().join(format!("khaynjo-shelf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("south-indian")).unwrap();
        std::fs::write(root.join("south-indian/masala-dosa.webp"), b"x").unwrap();
        std::fs::write(root.join("south-indian/notes.txt"), b"x").unwrap();
        let shelf = load_shelf(&root);
        assert_eq!(shelf, vec![ShelfDish {
            name: "Masala Dosa".into(),
            folder: "South Indian".into(),
            path: root.join("south-indian/masala-dosa.webp"),
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

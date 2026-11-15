//! Desktop command line for app templates — our side of "Export template".
//!
//! ```text
//! kumkaj-khaynjo --export-template <books.sqlite3 | books folder> <out folder>
//! kumkaj-khaynjo --check-template <pack folder> [<published pack folder>]
//! ```
//!
//! `--export-template` writes the same pack the in-app export writes, from a
//! client's books (a backup copy, say). It works on a private copy of the
//! books — opening a books file can upgrade it, and a client's file is never
//! changed here. `--check-template` lists what is wrong with a pack (PINs,
//! missing photos, settings a pack may not carry …) and, given the pack that
//! is live now, what the client changed. Exit code 1 when anything is wrong.
//!
//! Runs before the window opens (like `--uninstall-wipe`).

use std::fs;
use std::path::{Path, PathBuf};

use crate::store::{self, Store, TemplateExport};

pub const EXPORT_ARG: &str = "--export-template";
pub const CHECK_ARG: &str = "--check-template";

/// Handle a template command when the first argument is one. Returns
/// `Some(exit code)` when it was, `None` to start the app as usual.
pub fn try_run() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some(EXPORT_ARG) => Some(report(export(&args[1..]))),
        Some(CHECK_ARG) => Some(report(check(&args[1..]))),
        _ => None,
    }
}

fn report(result: Result<Vec<String>, String>) -> i32 {
    match result {
        Ok(lines) => {
            for l in &lines {
                println!("{l}");
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            fs::copy(&path, to.join(entry.file_name()))
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// Copy what an export reads — the books (with their WAL), menu and brand
/// pictures and this device's print layouts — into a scratch folder.
fn private_copy(books: &Path) -> Result<(PathBuf, PathBuf), String> {
    let db = if books.is_dir() { books.join("books.sqlite3") } else { books.to_path_buf() };
    if !db.is_file() {
        return Err(format!("No books at {}", db.display()));
    }
    let base = db.parent().unwrap_or(Path::new("."));
    let scratch = std::env::temp_dir().join(format!("khaynjo-export-{}", crate::domain::new_id()));
    fs::create_dir_all(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    let name = db.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    for suffix in ["", "-wal", "-shm"] {
        let from = base.join(format!("{name}{suffix}"));
        if from.is_file() {
            fs::copy(&from, scratch.join(format!("books.sqlite3{suffix}")))
                .map_err(|e| format!("{}: {e}", from.display()))?;
        }
    }
    for dir in ["item_images", "brand_images"] {
        if base.join(dir).is_dir() {
            copy_dir(&base.join(dir), &scratch.join(dir))?;
        }
    }
    if base.join("printers.json").is_file() {
        fs::copy(base.join("printers.json"), scratch.join("printers.json"))
            .map_err(|e| e.to_string())?;
    }
    Ok((scratch.clone(), scratch.join("books.sqlite3")))
}

fn export(args: &[String]) -> Result<Vec<String>, String> {
    let [books, out] = args else {
        return Err(format!("usage: kumkaj-khaynjo {EXPORT_ARG} <books.sqlite3 | books folder> <out folder>"));
    };
    let (scratch, db) = private_copy(Path::new(books))?;
    let result = (|| {
        let store = Store::open_path(&db)?;
        let owner = store
            .list_staff()?
            .into_iter()
            .find(|s| s.is_owner())
            .ok_or_else(|| "Owner account missing".to_string())?;
        let s = store.export_app_template(Path::new(out), &TemplateExport::default(), &owner)?;
        let mut lines = vec![format!(
            "{out}: {} categories, {} items, {} photos, {} stations, {} tax rates, {} schemes, {} areas, {} tables, {} staff, {} settings{}{}{}",
            s.categories,
            s.items,
            s.photos,
            s.stations,
            s.tax_rates,
            s.schemes,
            s.areas,
            s.tables,
            s.staff,
            s.settings,
            if s.logo { ", logo" } else { "" },
            if s.payment_qr { ", payment QR" } else { "" },
            if s.stamp { ", stamp" } else { "" },
        )];
        lines.extend(store::lint_pack(Path::new(out)).into_iter().map(|l| format!("check: {l}")));
        Ok(lines)
    })();
    let _ = fs::remove_dir_all(&scratch);
    result
}

fn check(args: &[String]) -> Result<Vec<String>, String> {
    let (pack, published) = match args {
        [pack] => (pack, None),
        [pack, published] => (pack, Some(published)),
        _ => {
            return Err(format!(
                "usage: kumkaj-khaynjo {CHECK_ARG} <pack folder> [<published pack folder>]"
            ))
        }
    };
    let problems = store::lint_pack(Path::new(pack));
    let mut lines: Vec<String> = Vec::new();
    if let Some(old) = published {
        lines.extend(store::diff_packs(Path::new(pack), Path::new(old))?);
        if lines.is_empty() {
            lines.push("no changes from the published pack".into());
        }
    }
    if problems.is_empty() {
        lines.push(format!("{pack}: ok"));
        Ok(lines)
    } else {
        for l in &lines {
            println!("{l}");
        }
        Err(problems.join("\n"))
    }
}

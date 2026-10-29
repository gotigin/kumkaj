//! Desktop uninstall / "Delete books data" helpers.
//!
//! Three independent categories (owner-facing):
//! - **trading** — business records (sales, payments, day books); keeps shop setup
//! - **schema** — shop setup databases (products, stations, staff, brand, empty books)
//! - **ui** — screen preferences (sidebar, amount eyes, window size, language)
//!
//! Invoked as `kumkaj-khaynjo.exe --uninstall-wipe <trading|schema|ui|…>`
//! before the MSI removes the exe (CustomAction runs Before="RemoveFiles").

use crate::local_servers::{self, DB_FILENAME};
use crate::store::{self, Store};
use std::fs;
use std::path::{Path, PathBuf};

pub const ARG: &str = "--uninstall-wipe";

/// If argv requests an uninstall wipe, run it and return `true` (caller exits).
pub fn try_run() -> bool {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some(ARG) {
        return false;
    }
    let mut trading = false;
    let mut schema = false;
    let mut ui = false;
    for a in args {
        match a.as_str() {
            "trading" | "data" | "records" => trading = true,
            "schema" | "setup" => schema = true,
            "ui" | "prefs" | "preferences" => ui = true,
            "all" => {
                trading = true;
                schema = true;
                ui = true;
            }
            _ => {}
        }
    }
    if !trading && !schema && !ui {
        return true;
    }
    let _ = run(trading, schema, ui);
    true
}

pub fn run(trading: bool, schema: bool, ui: bool) -> Result<(), String> {
    let base = store::data_dir()?;
    if !base.exists() {
        return Ok(());
    }
    if schema {
        wipe_schema(&base)?;
    } else if trading {
        wipe_trading(&base)?;
    }
    if ui {
        wipe_ui(&base, /*dbs_remain*/ !schema)?;
    }
    let _ = remove_dir_if_empty(&base);
    Ok(())
}

fn wipe_ui(base: &Path, clear_db_meta: bool) -> Result<(), String> {
    let _ = fs::remove_file(base.join("window_geom.json"));
    let _ = fs::remove_file(base.join("app_language.json"));
    if clear_db_meta {
        for db in books_db_paths(base) {
            if let Ok(store) = Store::open_path(&db) {
                let _ = store.uninstall_wipe_ui_prefs();
            }
        }
    }
    Ok(())
}

fn wipe_trading(base: &Path) -> Result<(), String> {
    for db in books_db_paths(base) {
        if let Ok(store) = Store::open_path(&db) {
            let _ = store.uninstall_wipe_trading_data();
        }
        if let Some(dir) = db.parent() {
            let _ = remove_tree(dir.join("attachments"));
            let _ = remove_tree(dir.join("backups"));
            let _ = remove_tree(dir.join("labels"));
        }
    }
    let _ = remove_tree(base.join("backups"));
    Ok(())
}

fn wipe_schema(base: &Path) -> Result<(), String> {
    let _ = remove_tree(base.join("servers"));
    let _ = remove_tree(base.join("client"));
    let _ = fs::remove_file(base.join("servers.json"));
    let _ = remove_tree(base.join("backups"));
    if let Ok(entries) = fs::read_dir(base) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".sqlite3")
                || name.ends_with(".sqlite3-wal")
                || name.ends_with(".sqlite3-shm")
            {
                let _ = fs::remove_file(&p);
            }
        }
    }
    Ok(())
}

fn books_db_paths(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let servers = base.join("servers");
    if let Ok(entries) = fs::read_dir(&servers) {
        for e in entries.flatten() {
            let db = e.path().join(DB_FILENAME);
            if db.is_file() {
                out.push(db);
            }
        }
    }
    let client = local_servers::client_db_path(base);
    if client.is_file() {
        out.push(client);
    }
    if let Ok(entries) = fs::read_dir(base) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("sqlite3") && p.is_file() {
                if !out.iter().any(|x| x == &p) {
                    out.push(p);
                }
            }
        }
    }
    out
}

fn remove_tree(path: PathBuf) -> std::io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn remove_dir_if_empty(path: &Path) -> std::io::Result<()> {
    if path.is_dir() && fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_base() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("khaynjo-uninstall-wipe-{nanos}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn schema_wipe_keeps_ui_files() {
        let base = temp_base();
        fs::write(base.join("window_geom.json"), "{}").unwrap();
        fs::write(base.join("app_language.json"), "{}").unwrap();
        let (info, store) = local_servers::create_local_server(&base, "Desk").unwrap();
        drop(store);
        assert!(local_servers::server_db_path(&base, &info.id).is_file());
        wipe_schema(&base).unwrap();
        assert!(!base.join("servers").exists());
        assert!(!base.join("servers.json").exists());
        assert!(base.join("window_geom.json").is_file());
        assert!(base.join("app_language.json").is_file());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn ui_wipe_removes_json_and_meta() {
        let base = temp_base();
        fs::write(base.join("window_geom.json"), "{}").unwrap();
        let (_info, store) = local_servers::create_local_server(&base, "Desk").unwrap();
        store.set_nav_order_csv("sell,books").unwrap();
        let db = store.db_path().to_path_buf();
        drop(store);
        wipe_ui(&base, true).unwrap();
        assert!(!base.join("window_geom.json").exists());
        let store = Store::open_path(&db).unwrap();
        assert!(store.nav_order_csv().unwrap().is_none());
        let _ = fs::remove_dir_all(base);
    }
}

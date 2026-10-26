//! Local multi-server registry: one SQLite books file per server on this device.
//!
//! Same phone can hold several businesses / products. Server Mode either creates
//! a new server (unique id) or opens one from this registry. Client Mode uses a
//! separate client cache DB and does not appear here.

use crate::domain::new_id;
use crate::store::{self, Store};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Books file name under `servers/{id}/` and `client/`.
pub const DB_FILENAME: &str = "moklanjo.sqlite3";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalServerInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct RegistryFile {
    #[serde(default)]
    active_id: Option<String>,
    #[serde(default)]
    servers: Vec<LocalServerInfo>,
}

fn registry_path(base: &Path) -> PathBuf {
    base.join("servers.json")
}

fn servers_root(base: &Path) -> PathBuf {
    base.join("servers")
}

pub fn server_db_path(base: &Path, server_id: &str) -> PathBuf {
    servers_root(base).join(server_id).join(DB_FILENAME)
}

pub fn client_db_path(base: &Path) -> PathBuf {
    base.join("client").join(DB_FILENAME)
}

/// App-global language choice: ONE per install, shared by every shop
/// (server DB) and every staff role on this device. It deliberately lives
/// outside the per-shop books file — language is a property of the app on
/// this device, not of a business or a user. `follow_platform` true means
/// "follow the OS language"; otherwise `language` is a fixed catalogue code.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppLanguagePref {
    follow_platform: bool,
    language: String,
}

fn app_language_path(base: &Path) -> PathBuf {
    base.join("app_language.json")
}

/// The remembered app-global language choice, if the user has made one.
pub fn load_app_language(base: &Path) -> Option<(bool, String)> {
    let raw = fs::read_to_string(app_language_path(base)).ok()?;
    let pref: AppLanguagePref = serde_json::from_str(&raw).ok()?;
    Some((pref.follow_platform, pref.language))
}

/// Remember the language choice for the whole app on this device.
pub fn save_app_language(base: &Path, follow_platform: bool, language: &str) -> Result<(), String> {
    fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let pref = AppLanguagePref {
        follow_platform,
        language: language.into(),
    };
    let raw = serde_json::to_string_pretty(&pref).map_err(|e| e.to_string())?;
    fs::write(app_language_path(base), raw).map_err(|e| e.to_string())
}

fn load_registry(base: &Path) -> Result<RegistryFile, String> {
    let path = registry_path(base);
    if !path.exists() {
        return Ok(RegistryFile::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_registry(base: &Path, reg: &RegistryFile) -> Result<(), String> {
    fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let path = registry_path(base);
    let raw = serde_json::to_string_pretty(reg).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())
}

pub fn list_local_servers(base: &Path) -> Result<Vec<LocalServerInfo>, String> {
    Ok(load_registry(base)?.servers)
}

pub fn active_server_id(base: &Path) -> Result<Option<String>, String> {
    Ok(load_registry(base)?.active_id)
}

pub fn set_active_server_id(base: &Path, id: &str) -> Result<(), String> {
    let mut reg = load_registry(base)?;
    if !reg.servers.iter().any(|s| s.id == id) {
        return Err("Server not found".into());
    }
    reg.active_id = Some(id.into());
    save_registry(base, &reg)
}

/// Create a new local server directory + empty DB, register it, set active.
pub fn create_local_server(base: &Path, name: &str) -> Result<(LocalServerInfo, Store), String> {
    let id = new_id();
    let db = server_db_path(base, &id);
    if let Some(parent) = db.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let store = Store::open_path(&db)?;
    let _ = store.set_device_role("server");
    let label = {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            let short = id.chars().take(8).collect::<String>();
            format!("Server {short}")
        } else {
            trimmed.to_string()
        }
    };
    // A typed name is the shop's; the generated label only names the server
    // in the list (it used to become the shop's name in the header and bills).
    if !name.trim().is_empty() && store.venue_title().unwrap_or_default().trim().is_empty() {
        let _ = store.set_venue_title(&label);
    }
    let info = LocalServerInfo {
        id: id.clone(),
        name: label,
        created_at: Local::now().to_rfc3339(),
    };
    let mut reg = load_registry(base)?;
    reg.servers.push(info.clone());
    reg.active_id = Some(id);
    save_registry(base, &reg)?;
    Ok((info, store))
}

pub fn open_server(base: &Path, server_id: &str) -> Result<Store, String> {
    let reg = load_registry(base)?;
    if !reg.servers.iter().any(|s| s.id == server_id) {
        return Err("Server not found".into());
    }
    let path = server_db_path(base, server_id);
    if !path.exists() {
        return Err("Server database missing".into());
    }
    let store = Store::open_path(&path)?;
    let _ = store.set_device_role("server");
    set_active_server_id(base, server_id)?;
    Ok(store)
}

pub fn open_client_store(base: &Path) -> Result<Store, String> {
    let path = client_db_path(base);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Store::open_path(&path)
}

/// Open the active server DB if one exists; otherwise `Err` so the shell can
/// wait for Server Mode create/pick (or Client Mode).
pub fn open_active_server(base: &Path) -> Result<Store, String> {
    let reg = load_registry(base)?;
    let Some(id) = reg.active_id.as_deref() else {
        return Err("No local server yet — choose Server Mode".into());
    };
    open_server(base, id)
}

/// Resolve data dir then open active server (desktop boot).
pub fn open_default_server() -> Result<Store, String> {
    let dir = store::data_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    open_active_server(&dir)
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
        let path = std::env::temp_dir().join(format!("kumkaj-servers-{nanos}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn create_second_server_lists_both() {
        let base = temp_base();
        assert!(list_local_servers(&base).unwrap().is_empty());
        let (a, _) = create_local_server(&base, "Tea stall").unwrap();
        let (b, _) = create_local_server(&base, "Moklanjo desk").unwrap();
        let list = list_local_servers(&base).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|s| s.id == a.id && s.name == "Tea stall"));
        assert!(list.iter().any(|s| s.id == b.id && s.name == "Moklanjo desk"));
        assert_eq!(active_server_id(&base).unwrap().as_deref(), Some(b.id.as_str()));
        let opened = open_server(&base, &a.id).unwrap();
        assert!(opened.db_path().ends_with(DB_FILENAME));
        assert_eq!(active_server_id(&base).unwrap().as_deref(), Some(a.id.as_str()));
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn app_language_is_one_global_choice_per_install() {
        let base = temp_base();
        // Nothing picked yet: no remembered choice.
        assert!(load_app_language(&base).is_none());
        save_app_language(&base, false, "hi").unwrap();
        assert_eq!(load_app_language(&base), Some((false, "hi".to_string())));
        // Switching to follow-the-OS overwrites the same single slot.
        save_app_language(&base, true, "en").unwrap();
        assert_eq!(load_app_language(&base), Some((true, "en".to_string())));
        // Corrupt file reads as "no choice" instead of failing boot.
        fs::write(app_language_path(&base), "not json").unwrap();
        assert!(load_app_language(&base).is_none());
        let _ = fs::remove_dir_all(base);
    }
}

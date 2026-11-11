//! Remote catalogue under `{product}/v1/catalogue` (public list + private codes).
//!
//! Docs: `apps/khaynjo/docs/CATALOGUE_URLS.md`. Fetches use HTTPS (`ureq`) or a
//! local tree via `KHAYNJO_CATALOGUE_DIR` (same layout as the CDN).

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::menu_images::SHELF_KEY_PREFIX;
use crate::menu_templates::{self, MenuTemplate};

pub const DEFAULT_CATALOGUE_BASE: &str = "https://khaynjo.com/v1/catalogue";
pub const PRIVATE_CODE_LEN: usize = 12;
const SCHEMA_V1: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogueKind {
    AppTemplate,
    MenuTemplate,
    Shelf,
}

impl Default for CatalogueKind {
    fn default() -> Self {
        Self::AppTemplate
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogueVisibility {
    Public,
    Private,
}

impl Default for CatalogueVisibility {
    fn default() -> Self {
        Self::Public
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicIndex {
    pub schema: u32,
    #[serde(default)]
    pub templates: Vec<PublicIndexEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicIndexEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub blurb: String,
    #[serde(default)]
    pub kind: CatalogueKind,
    #[serde(default)]
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogueManifest {
    pub schema: u32,
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub blurb: String,
    #[serde(default)]
    pub kind: CatalogueKind,
    #[serde(default)]
    pub visibility: CatalogueVisibility,
    #[serde(default)]
    pub applies_shop_defaults: bool,
    #[serde(default)]
    pub shop_defaults_id: String,
    #[serde(default = "default_template_file")]
    pub template: String,
    /// Relative path to a shelf file index (for `kind: shelf`), e.g. `../../items/index.json`.
    #[serde(default)]
    pub items_index: String,
    #[serde(default)]
    pub bytes: u64,
    #[serde(default)]
    pub sha256: String,
}

fn default_template_file() -> String {
    "template.json".into()
}

/// Default `https://khaynjo.com/v1/catalogue` — override with `KHAYNJO_CATALOGUE_BASE`.
pub fn catalogue_base_url() -> String {
    std::env::var("KHAYNJO_CATALOGUE_BASE")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_CATALOGUE_BASE.to_string())
}

/// Optional filesystem mirror of the CDN tree (`…/public/…`, `…/private/…`).
pub fn catalogue_fs_root() -> Option<PathBuf> {
    std::env::var_os("KHAYNJO_CATALOGUE_DIR").map(PathBuf::from)
}

pub fn public_index_url() -> String {
    format!("{}/public/index.json", catalogue_base_url())
}

pub fn public_template_url(id: &str) -> String {
    let id = id.trim().trim_matches('/');
    format!("{}/public/templates/{id}", catalogue_base_url())
}

pub fn private_template_url(code: &str) -> String {
    let code = code.trim();
    format!("{}/private/{code}", catalogue_base_url())
}

/// Exactly 12 ASCII digits.
pub fn normalize_private_code(raw: &str) -> Result<String, String> {
    let digits: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if digits.len() != PRIVATE_CODE_LEN {
        return Err(format!(
            "Private code must be {PRIVATE_CODE_LEN} digits"
        ));
    }
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err("Private code must be digits only".into());
    }
    Ok(digits)
}

pub fn parse_public_index(raw: &str) -> Result<PublicIndex, String> {
    let index: PublicIndex =
        serde_json::from_str(raw).map_err(|e| format!("Public index JSON: {e}"))?;
    if index.schema != SCHEMA_V1 {
        return Err(format!(
            "Unsupported catalogue index schema {} (want {SCHEMA_V1})",
            index.schema
        ));
    }
    Ok(index)
}

pub fn parse_manifest(raw: &str) -> Result<CatalogueManifest, String> {
    let man: CatalogueManifest =
        serde_json::from_str(raw).map_err(|e| format!("Catalogue manifest JSON: {e}"))?;
    if man.schema != SCHEMA_V1 {
        return Err(format!(
            "Unsupported catalogue manifest schema {} (want {SCHEMA_V1})",
            man.schema
        ));
    }
    if man.id.trim().is_empty() {
        return Err("Catalogue manifest missing id".into());
    }
    if man.title.trim().is_empty() {
        return Err("Catalogue manifest missing title".into());
    }
    Ok(man)
}

fn join_url(base: &str, rel: &str) -> String {
    let base = base.trim_end_matches('/');
    let rel = rel.trim_start_matches('/');
    format!("{base}/{rel}")
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(60))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(404, _) => "Not found".into(),
            ureq::Error::Status(status, _) => format!("HTTP {status}"),
            ureq::Error::Transport(t) => format!("Network: {t}"),
        })?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(64 * 1024 * 1024)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Read body: {e}"))?;
    Ok(buf)
}

fn http_get_text(url: &str) -> Result<String, String> {
    let bytes = http_get_bytes(url)?;
    String::from_utf8(bytes).map_err(|e| format!("UTF-8: {e}"))
}

fn fs_read_text(root: &Path, rel: &str) -> Result<String, String> {
    let path = root.join(rel.trim_start_matches('/'));
    fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "Not found".into()
        } else {
            format!("{}: {e}", path.display())
        }
    })
}

fn fs_read_bytes(root: &Path, rel: &str) -> Result<Vec<u8>, String> {
    let path = root.join(rel.trim_start_matches('/'));
    fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "Not found".into()
        } else {
            format!("{}: {e}", path.display())
        }
    })
}

/// Fetch public index (HTTP or `KHAYNJO_CATALOGUE_DIR`).
pub fn fetch_public_index() -> Result<PublicIndex, String> {
    let raw = if let Some(root) = catalogue_fs_root() {
        fs_read_text(&root, "public/index.json")?
    } else {
        http_get_text(&public_index_url())?
    };
    parse_public_index(&raw)
}

/// Peek a private pack’s manifest by 12-digit code (does not download the pack).
pub fn fetch_private_manifest(code: &str) -> Result<CatalogueManifest, String> {
    let code = normalize_private_code(code)?;
    let raw = if let Some(root) = catalogue_fs_root() {
        fs_read_text(&root, &format!("private/{code}/manifest.json"))?
    } else {
        http_get_text(&join_url(&private_template_url(&code), "manifest.json"))?
    };
    parse_manifest(&raw)
}

/// Peek a public pack’s manifest by id.
pub fn fetch_public_manifest(id: &str) -> Result<CatalogueManifest, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Template id required".into());
    }
    let raw = if let Some(root) = catalogue_fs_root() {
        fs_read_text(&root, &format!("public/templates/{id}/manifest.json"))?
    } else {
        http_get_text(&join_url(&public_template_url(id), "manifest.json"))?
    };
    parse_manifest(&raw)
}

fn cache_pack_dir(data_base: &Path, visibility: CatalogueVisibility, key: &str) -> PathBuf {
    match visibility {
        CatalogueVisibility::Public => data_base
            .join("catalogue")
            .join("public")
            .join("templates")
            .join(key),
        CatalogueVisibility::Private => data_base.join("catalogue").join("private").join(key),
    }
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Relative paths inside the pack to download (self-contained media).
/// `images/…` stay as-is; legacy `shelf:folder/file.webp` map to `items/…`.
fn collect_image_rels(tmpl: &MenuTemplate) -> Vec<String> {
    let mut out = Vec::new();
    let mut push = |s: &str| {
        let s = s.trim();
        if s.is_empty() {
            return;
        }
        let rel = if let Some(shelf) = s.strip_prefix(SHELF_KEY_PREFIX) {
            format!("items/{shelf}")
        } else {
            s.to_string()
        };
        if !out.iter().any(|x| x == &rel) {
            out.push(rel);
        }
    };
    for cat in &tmpl.categories {
        push(&cat.image);
        for item in &cat.items {
            push(&item.image);
        }
    }
    out
}

fn read_pack_text(visibility: CatalogueVisibility, key: &str, rel: &str) -> Result<String, String> {
    let pack_rel = match visibility {
        CatalogueVisibility::Public => format!("public/templates/{}", key.trim()),
        CatalogueVisibility::Private => {
            let code = normalize_private_code(key)?;
            format!("private/{code}")
        }
    };
    if let Some(root) = catalogue_fs_root() {
        return fs_read_text(&root, &format!("{pack_rel}/{rel}"));
    }
    let base = match visibility {
        CatalogueVisibility::Public => public_template_url(key.trim()),
        CatalogueVisibility::Private => private_template_url(&normalize_private_code(key)?),
    };
    http_get_text(&join_url(&base, rel))
}

fn read_pack_bytes(visibility: CatalogueVisibility, key: &str, rel: &str) -> Result<Vec<u8>, String> {
    let pack_rel = match visibility {
        CatalogueVisibility::Public => format!("public/templates/{}", key.trim()),
        CatalogueVisibility::Private => {
            let code = normalize_private_code(key)?;
            format!("private/{code}")
        }
    };
    if let Some(root) = catalogue_fs_root() {
        return fs_read_bytes(&root, &format!("{pack_rel}/{rel}"));
    }
    let base = match visibility {
        CatalogueVisibility::Public => public_template_url(key.trim()),
        CatalogueVisibility::Private => private_template_url(&normalize_private_code(key)?),
    };
    http_get_bytes(&join_url(&base, rel))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ItemsIndex {
    schema: u32,
    #[serde(default)]
    files: Vec<String>,
}

fn read_public_items_index() -> Result<ItemsIndex, String> {
    let raw = if let Some(root) = catalogue_fs_root() {
        fs_read_text(&root, "public/items/index.json")?
    } else {
        http_get_text(&join_url(&catalogue_base_url(), "public/items/index.json"))?
    };
    let index: ItemsIndex =
        serde_json::from_str(&raw).map_err(|e| format!("items index JSON: {e}"))?;
    if index.schema != SCHEMA_V1 {
        return Err(format!(
            "Unsupported items index schema {} (want {SCHEMA_V1})",
            index.schema
        ));
    }
    Ok(index)
}

fn read_public_item_bytes(rel: &str) -> Result<Vec<u8>, String> {
    let rel = rel.trim().trim_start_matches('/');
    if let Some(root) = catalogue_fs_root() {
        return fs_read_bytes(&root, &format!("public/items/{rel}"));
    }
    http_get_bytes(&join_url(
        &join_url(&catalogue_base_url(), "public/items"),
        rel,
    ))
}

/// Mirror `public/items/` into `{data}/catalogue/public/items/`.
pub fn download_public_shelf(
    data_base: &Path,
    cancel: &dyn Fn() -> bool,
) -> Result<PathBuf, String> {
    let index = read_public_items_index()?;
    let dest = data_base
        .join("catalogue")
        .join("public")
        .join("items");
    if dest.exists() {
        let _ = fs::remove_dir_all(&dest);
    }
    fs::create_dir_all(&dest).map_err(|e| format!("mkdir {}: {e}", dest.display()))?;
    write_file(
        &dest.join("index.json"),
        serde_json::to_string_pretty(&index)
            .map_err(|e| format!("serialize items index: {e}"))?
            .as_bytes(),
    )?;
    for rel in &index.files {
        if cancel() {
            return Err("Cancelled".into());
        }
        let bytes = read_public_item_bytes(rel)?;
        write_file(&dest.join(rel), &bytes)?;
    }
    Ok(dest)
}

/// Download a pack into `{data}/catalogue/…` and return the pack directory.
/// For `kind: shelf`, mirrors `public/items/` and returns that items directory.
pub fn download_pack(
    data_base: &Path,
    visibility: CatalogueVisibility,
    key: &str,
    cancel: &dyn Fn() -> bool,
) -> Result<(PathBuf, CatalogueManifest, MenuTemplate), String> {
    if cancel() {
        return Err("Cancelled".into());
    }
    let cache_key = match visibility {
        CatalogueVisibility::Public => key.trim().to_string(),
        CatalogueVisibility::Private => normalize_private_code(key)?,
    };
    if cache_key.is_empty() {
        return Err("Template id required".into());
    }

    let manifest_raw = read_pack_text(visibility.clone(), &cache_key, "manifest.json")?;
    let manifest = parse_manifest(&manifest_raw)?;
    if cancel() {
        return Err("Cancelled".into());
    }

    if matches!(manifest.kind, CatalogueKind::Shelf) {
        if !matches!(visibility, CatalogueVisibility::Public) {
            return Err("Shelf packs are public only".into());
        }
        let items_dir = download_public_shelf(data_base, cancel)?;
        let pack = cache_pack_dir(data_base, visibility, &cache_key);
        if pack.exists() {
            let _ = fs::remove_dir_all(&pack);
        }
        write_file(&pack.join("manifest.json"), manifest_raw.as_bytes())?;
        let stub = MenuTemplate {
            id: manifest.id.clone(),
            title: manifest.title.clone(),
            blurb: manifest.blurb.clone(),
            categories: Vec::new(),
        };
        write_file(
            &pack.join("template.json"),
            serde_json::to_string_pretty(&stub)
                .map_err(|e| format!("stub template: {e}"))?
                .as_bytes(),
        )?;
        return Ok((items_dir, manifest, stub));
    }

    let template_rel = manifest.template.trim();
    if template_rel.is_empty() {
        return Err("Catalogue manifest missing template".into());
    }
    let template_raw = read_pack_text(visibility.clone(), &cache_key, template_rel)?;
    if !manifest.sha256.trim().is_empty() {
        let got = sha256_hex(template_raw.as_bytes());
        if !got.eq_ignore_ascii_case(manifest.sha256.trim()) {
            return Err("Template checksum mismatch".into());
        }
    }
    let tmpl: MenuTemplate = serde_json::from_str(&template_raw)
        .map_err(|e| format!("template.json: {e}"))?;

    let dest = cache_pack_dir(data_base, visibility.clone(), &cache_key);
    if dest.exists() {
        let _ = fs::remove_dir_all(&dest);
    }
    write_file(&dest.join("manifest.json"), manifest_raw.as_bytes())?;
    write_file(&dest.join(template_rel), template_raw.as_bytes())?;

    for rel in collect_image_rels(&tmpl) {
        if cancel() {
            return Err("Cancelled".into());
        }
        let bytes = read_pack_bytes(visibility.clone(), &cache_key, &rel)?;
        write_file(&dest.join(&rel), &bytes)?;
    }

    Ok((dest, manifest, tmpl))
}

/// Load a previously cached (or fixture) pack directory.
pub fn load_pack_dir(dir: &Path) -> Result<(CatalogueManifest, MenuTemplate), String> {
    let man_raw = fs::read_to_string(dir.join("manifest.json"))
        .map_err(|e| format!("manifest.json: {e}"))?;
    let manifest = parse_manifest(&man_raw)?;
    let tmpl = menu_templates::load_from_dir(dir)
        .ok_or_else(|| format!("No template.json in {}", dir.display()))?;
    Ok((manifest, tmpl))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// `KHAYNJO_CATALOGUE_DIR` is process-global — serialize tests that mutate it.
    fn catalogue_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn private_code_must_be_twelve_digits() {
        assert!(normalize_private_code("123456789012").is_ok());
        assert!(normalize_private_code("123 456 789 012").is_ok());
        assert!(normalize_private_code("123").is_err());
        assert!(normalize_private_code("abcdefghijkl").is_err());
    }

    #[test]
    fn url_helpers_default_paths() {
        assert_eq!(
            public_index_url_for(DEFAULT_CATALOGUE_BASE),
            format!("{DEFAULT_CATALOGUE_BASE}/public/index.json")
        );
        assert!(
            public_template_url_for(DEFAULT_CATALOGUE_BASE, "rajdhani")
                .ends_with("/public/templates/rajdhani")
        );
        assert!(
            private_template_url_for(DEFAULT_CATALOGUE_BASE, "123456789012")
                .ends_with("/private/123456789012")
        );
    }

    fn public_index_url_for(base: &str) -> String {
        format!("{}/public/index.json", base.trim_end_matches('/'))
    }

    fn public_template_url_for(base: &str, id: &str) -> String {
        format!("{}/public/templates/{id}", base.trim_end_matches('/'))
    }

    fn private_template_url_for(base: &str, code: &str) -> String {
        format!("{}/private/{code}", base.trim_end_matches('/'))
    }

    #[test]
    fn parse_index_and_manifest_v1() {
        let index = parse_public_index(
            r#"{
              "schema": 1,
              "templates": [
                {"id": "starter", "title": "Starter", "kind": "app_template", "bytes": 10}
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(index.templates.len(), 1);
        assert_eq!(index.templates[0].id, "starter");

        let man = parse_manifest(
            r#"{
              "schema": 1,
              "id": "rajdhani",
              "title": "Rajdhani Menu Template",
              "visibility": "private",
              "applies_shop_defaults": true,
              "shop_defaults_id": "rajdhani"
            }"#,
        )
        .unwrap();
        assert_eq!(man.id, "rajdhani");
        assert!(man.applies_shop_defaults);
        assert_eq!(man.template, "template.json");
    }

    #[test]
    fn download_pack_from_fs_fixture() {
        let root = tempfile_fixture();
        let code = "123456789012";
        let pack = root.join("private").join(code);
        fs::create_dir_all(pack.join("images")).unwrap();
        fs::write(
            pack.join("manifest.json"),
            r#"{
              "schema": 1,
              "id": "demo",
              "title": "Demo Template",
              "visibility": "private",
              "kind": "app_template",
              "template": "template.json"
            }"#,
        )
        .unwrap();
        fs::write(
            pack.join("template.json"),
            r#"{
              "id": "demo",
              "title": "Demo Template",
              "categories": [
                {
                  "name": "Tea",
                  "items": [{"name": "Chai", "price": 20, "image": "images/chai.webp"}]
                }
              ]
            }"#,
        )
        .unwrap();
        fs::write(pack.join("images/chai.webp"), b"RIFF....WEBP").unwrap();

        let _guard = catalogue_env_lock();
        unsafe {
            std::env::set_var("KHAYNJO_CATALOGUE_DIR", &root);
        }
        let data = root.join("data");
        let result = download_pack(&data, CatalogueVisibility::Private, code, &|| false);
        unsafe {
            std::env::remove_var("KHAYNJO_CATALOGUE_DIR");
        }
        let (dest, man, tmpl) = result.unwrap();
        assert_eq!(man.title, "Demo Template");
        assert_eq!(tmpl.item_count(), 1);
        assert!(dest.join("template.json").is_file());
        assert!(dest.join("images/chai.webp").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn download_shelf_kind_mirrors_public_items() {
        let root = tempfile_fixture();
        let items = root.join("public/items/hot-beverages");
        fs::create_dir_all(&items).unwrap();
        fs::write(items.join("chai.webp"), b"RIFF....WEBP").unwrap();
        fs::write(
            root.join("public/items/index.json"),
            r#"{"schema":1,"folders":["hot-beverages"],"files":["hot-beverages/chai.webp"],"count":1}"#,
        )
        .unwrap();
        let pack = root.join("public/templates/khaynjo-items");
        fs::create_dir_all(&pack).unwrap();
        fs::write(
            pack.join("manifest.json"),
            r#"{
              "schema": 1,
              "id": "khaynjo-items",
              "title": "Khaynjo Items Template",
              "kind": "shelf",
              "visibility": "public",
              "items_index": "../../items/index.json"
            }"#,
        )
        .unwrap();
        fs::write(
            pack.join("template.json"),
            r#"{"id":"khaynjo-items","title":"Khaynjo Items Template","categories":[]}"#,
        )
        .unwrap();

        let _guard = catalogue_env_lock();
        unsafe {
            std::env::set_var("KHAYNJO_CATALOGUE_DIR", &root);
        }
        let data = root.join("data");
        let result = download_pack(&data, CatalogueVisibility::Public, "khaynjo-items", &|| false);
        unsafe {
            std::env::remove_var("KHAYNJO_CATALOGUE_DIR");
        }
        let (dest, man, _) = result.unwrap();
        assert_eq!(man.kind, CatalogueKind::Shelf);
        assert!(dest.join("hot-beverages/chai.webp").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn download_pack_fetches_shelf_items_into_cache() {
        let root = tempfile_fixture();
        let code = "987654321098";
        let pack = root.join("private").join(code);
        fs::create_dir_all(pack.join("items/hot-beverages")).unwrap();
        fs::write(
            pack.join("manifest.json"),
            r#"{
              "schema": 1,
              "id": "shelf-demo",
              "title": "Shelf Demo",
              "visibility": "private",
              "kind": "app_template",
              "template": "template.json",
              "applies_shop_defaults": true,
              "shop_defaults_id": "rajdhani"
            }"#,
        )
        .unwrap();
        fs::write(
            pack.join("template.json"),
            r#"{
              "id": "shelf-demo",
              "title": "Shelf Demo",
              "categories": [
                {
                  "name": "Tea",
                  "items": [{"name": "Chai", "price": 20, "image": "shelf:hot-beverages/chai.webp"}]
                }
              ]
            }"#,
        )
        .unwrap();
        fs::write(
            pack.join("items/hot-beverages/chai.webp"),
            b"RIFF....WEBP",
        )
        .unwrap();

        let _guard = catalogue_env_lock();
        unsafe {
            std::env::set_var("KHAYNJO_CATALOGUE_DIR", &root);
        }
        let data = root.join("data");
        let result = download_pack(&data, CatalogueVisibility::Private, code, &|| false);
        unsafe {
            std::env::remove_var("KHAYNJO_CATALOGUE_DIR");
        }
        let (dest, man, _) = result.unwrap();
        assert_eq!(man.shop_defaults_id, "rajdhani");
        assert!(dest.join("items/hot-beverages/chai.webp").is_file());
        let _ = fs::remove_dir_all(root);
    }

    fn tempfile_fixture() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "khaynjo-catalogue-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}

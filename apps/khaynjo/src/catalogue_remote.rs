//! Remote catalogue under `{product}/v1/catalogue` (public list + private codes).
//!
//! Docs: `apps/khaynjo/docs/CATALOGUE_URLS.md`. Fetches use HTTPS (`ureq`) or a
//! local tree via `KHAYNJO_CATALOGUE_DIR` (same layout as the CDN). Windows,
//! Linux and Android all take this same path — no platform reads APK-bundled
//! menu assets while `assets/CATALOGUE_REMOTE` is set.
//!
//! ## Resumable downloads
//!
//! Packs and the photo shelf download **file by file** into a staging
//! directory next to the final cache (`{dir}.part/`). Each file is written to
//! `…​.dl` and renamed into place, so a file present under its real name is
//! whole. `download.state.json` in the staging dir records which listing it
//! belongs to (sha256 of the manifest / items index) and which files are done.
//!
//! - Cancel, a dropped connection, a crash or an app restart leave the
//!   staging dir; the next attempt skips every file already on disk that
//!   matches the listed size / sha256 and fetches only the rest.
//! - A new publish (different listing sha) keeps files it can prove by
//!   sha256 and drops the rest; a different schema or source wipes staging.
//! - When every file is in, staging is renamed over the final dir in one
//!   step, so import / the shelf scanner never see a half pack.
//! - A finished pack whose listing still matches is reused without fetching
//!   files; offline, a finished pack is reused when the listing cannot load.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::menu_images::SHELF_KEY_PREFIX;
use crate::menu_templates::{self, MenuTemplate};

pub const DEFAULT_CATALOGUE_BASE: &str = "https://khaynjo.com/v1/catalogue";
pub const PRIVATE_CODE_LEN: usize = 12;
const SCHEMA_V1: u32 = 1;
/// `download.state.json` layout version — bump to discard old staging dirs.
const DOWNLOAD_STATE_SCHEMA: u32 = 1;
pub const DOWNLOAD_STATE_FILE: &str = "download.state.json";
/// Persist the state file at least this often while files stream in.
const STATE_SAVE_EVERY_FILES: usize = 24;
const STATE_SAVE_EVERY_BYTES: u64 = 4 * 1024 * 1024;
/// Extra tries per file on a transport error (flaky shop Wi-Fi / mobile data).
const FETCH_RETRIES: usize = 2;

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
    /// Total download size (every file in the pack / shelf).
    #[serde(default)]
    pub bytes: u64,
}

/// One file in a pack (relative to the pack dir) or the shelf (relative to
/// `public/items/`), as listed by `publish-catalogue.sh`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogueFile {
    pub path: String,
    /// Exact size; `0` = unknown (legacy listings).
    #[serde(default)]
    pub bytes: u64,
    /// Lowercase hex sha256; empty = unknown.
    #[serde(default)]
    pub sha256: String,
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
    /// Total download size of the pack (template + images; shelf: every photo).
    #[serde(default)]
    pub bytes: u64,
    /// sha256 of `template.json`.
    #[serde(default)]
    pub sha256: String,
    /// Every file in the pack (template + images) with size + sha256. Empty on
    /// older packs — then the list is derived from `template.json`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<CatalogueFile>,
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
    for f in &man.files {
        safe_rel(&f.path)?;
    }
    Ok(man)
}

fn join_url(base: &str, rel: &str) -> String {
    let base = base.trim_end_matches('/');
    let rel = rel.trim_start_matches('/');
    format!("{base}/{rel}")
}

// ---------------------------------------------------------------------------
// Sources: HTTPS or a filesystem mirror. Paths are relative to the
// catalogue base (`public/items/…`, `private/<code>/…`).
// ---------------------------------------------------------------------------

/// Where catalogue bytes come from. Tests inject counting / failing sources.
pub trait CatalogueSource {
    fn get(&self, rel: &str) -> Result<Vec<u8>, String>;

    fn get_text(&self, rel: &str) -> Result<String, String> {
        let bytes = self.get(rel)?;
        String::from_utf8(bytes).map_err(|e| format!("UTF-8: {e}"))
    }
}

/// HTTPS source with one keep-alive agent (1,400+ shelf files on one socket).
pub struct HttpSource {
    base: String,
    agent: ureq::Agent,
}

impl HttpSource {
    pub fn new(base: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(15))
            .timeout_read(std::time::Duration::from_secs(45))
            .build();
        Self {
            base: base.trim_end_matches('/').to_string(),
            agent,
        }
    }
}

impl CatalogueSource for HttpSource {
    fn get(&self, rel: &str) -> Result<Vec<u8>, String> {
        let url = join_url(&self.base, rel);
        let resp = self.agent.get(&url).call().map_err(|e| match e {
            ureq::Error::Status(404, _) => "Not found".to_string(),
            ureq::Error::Status(status, _) => format!("HTTP {status}"),
            ureq::Error::Transport(t) => format!("Network: {t}"),
        })?;
        let mut buf = Vec::new();
        resp.into_reader()
            .take(64 * 1024 * 1024)
            .read_to_end(&mut buf)
            .map_err(|e| format!("Network: read body: {e}"))?;
        Ok(buf)
    }
}

/// `KHAYNJO_CATALOGUE_DIR` mirror (staging tree, tests, offline installs).
pub struct FsSource {
    root: PathBuf,
    /// Per-file delay — `KHAYNJO_CATALOGUE_DELAY_MS` makes a local mirror as
    /// slow as shop internet so cancel / kill / resume can be tried by hand.
    delay: Option<std::time::Duration>,
}

impl FsSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            delay: None,
        }
    }

    fn with_env_delay(mut self) -> Self {
        self.delay = std::env::var("KHAYNJO_CATALOGUE_DELAY_MS")
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .filter(|&ms| ms > 0)
            .map(std::time::Duration::from_millis);
        self
    }
}

impl CatalogueSource for FsSource {
    fn get(&self, rel: &str) -> Result<Vec<u8>, String> {
        if let Some(d) = self.delay {
            std::thread::sleep(d);
        }
        let path = self.root.join(rel.trim_start_matches('/'));
        fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Not found".into()
            } else {
                format!("{}: {e}", path.display())
            }
        })
    }
}

/// The source every platform uses: `KHAYNJO_CATALOGUE_DIR` when set, else
/// HTTPS at `catalogue_base_url()`.
pub fn default_source() -> Box<dyn CatalogueSource> {
    match catalogue_fs_root() {
        Some(root) => Box::new(FsSource::new(root).with_env_delay()),
        None => Box::new(HttpSource::new(&catalogue_base_url())),
    }
}

/// True for errors worth one more try (dropped socket, timeout, 5xx).
fn is_retryable(err: &str) -> bool {
    err.starts_with("Network:") || err.starts_with("HTTP 5")
}

/// Fetch public index (HTTP or `KHAYNJO_CATALOGUE_DIR`).
pub fn fetch_public_index() -> Result<PublicIndex, String> {
    parse_public_index(&default_source().get_text("public/index.json")?)
}

/// Peek a private pack’s manifest by 12-digit code (does not download the pack).
pub fn fetch_private_manifest(code: &str) -> Result<CatalogueManifest, String> {
    let code = normalize_private_code(code)?;
    parse_manifest(&default_source().get_text(&format!("private/{code}/manifest.json"))?)
}

/// Peek a public pack’s manifest by id.
pub fn fetch_public_manifest(id: &str) -> Result<CatalogueManifest, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Template id required".into());
    }
    safe_rel(id)?;
    parse_manifest(
        &default_source().get_text(&format!("public/templates/{id}/manifest.json"))?,
    )
}

// ---------------------------------------------------------------------------
// Cache layout + small fs helpers
// ---------------------------------------------------------------------------

fn cache_pack_dir(data_base: &Path, visibility: &CatalogueVisibility, key: &str) -> PathBuf {
    match visibility {
        CatalogueVisibility::Public => data_base
            .join("catalogue")
            .join("public")
            .join("templates")
            .join(key),
        CatalogueVisibility::Private => data_base.join("catalogue").join("private").join(key),
    }
}

/// `{data}/catalogue/public/items` — what `menu_images::catalogue_root` scans.
pub fn cached_shelf_dir(data_base: &Path) -> PathBuf {
    data_base.join("catalogue").join("public").join("items")
}

/// Staging sibling: `…/items` → `…/items.part`.
fn sibling(dir: &Path, suffix: &str) -> PathBuf {
    let mut name = dir.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    dir.with_file_name(name)
}

/// Staging dir for a cache dir (exposed for tests / diagnostics).
pub fn staging_dir(final_dir: &Path) -> PathBuf {
    sibling(final_dir, ".part")
}

/// A listed path must stay inside the pack: plain relative components only.
fn safe_rel(rel: &str) -> Result<&str, String> {
    let rel = rel.trim();
    if rel.is_empty() || rel.contains('\\') || rel.starts_with('/') {
        return Err(format!("Unsafe catalogue path '{rel}'"));
    }
    let ok = Path::new(rel)
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    if !ok {
        return Err(format!("Unsafe catalogue path '{rel}'"));
    }
    Ok(rel)
}

/// Write via `<file>.dl` + rename so a file under its real name is whole.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let tmp = sibling(path, ".dl");
    fs::write(&tmp, bytes).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename {}: {e}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn sha256_file(path: &Path) -> Option<String> {
    let mut f = fs::File::open(path).ok()?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Some(hex::encode(h.finalize()))
}

/// Bytes match the listing (size when known, sha256 when known).
fn bytes_match(bytes: &[u8], want: &CatalogueFile) -> bool {
    (want.bytes == 0 || bytes.len() as u64 == want.bytes)
        && (want.sha256.is_empty() || sha256_hex(bytes).eq_ignore_ascii_case(&want.sha256))
}

/// A file on disk provably matches the listing. Unknown size *and* sha
/// proves nothing — the caller decides (state file / re-fetch).
fn file_matches(path: &Path, want: &CatalogueFile, check_sha: bool) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    if want.bytes > 0 && meta.len() != want.bytes {
        return false;
    }
    if want.bytes == 0 && want.sha256.is_empty() {
        return false;
    }
    if check_sha && !want.sha256.is_empty() {
        return sha256_file(path).is_some_and(|got| got.eq_ignore_ascii_case(&want.sha256));
    }
    true
}

// ---------------------------------------------------------------------------
// Download state + progress
// ---------------------------------------------------------------------------

/// Progress of one pack / shelf download (worker → UI).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DownloadProgress {
    pub files_done: usize,
    pub files_total: usize,
    /// Bytes on disk so far (including resumed files).
    pub bytes_done: u64,
    /// `0` when the listing has no sizes (legacy) — show file counts then.
    pub bytes_total: u64,
    /// Files found already complete when this attempt started.
    pub resumed_files: usize,
}

/// `download.state.json` — lives in the staging dir, then the final dir.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadState {
    pub schema: u32,
    /// Remote prefix, e.g. `public/items` or `public/templates/khaynjo-starter`.
    pub source: String,
    /// sha256 of the manifest / items index this file list came from.
    pub listing_sha256: String,
    #[serde(default)]
    pub complete: bool,
    /// Finished files: relative path → listed size / sha256.
    #[serde(default)]
    pub done: BTreeMap<String, DoneFile>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoneFile {
    #[serde(default)]
    pub bytes: u64,
    #[serde(default)]
    pub sha256: String,
}

impl DownloadState {
    fn new(source: &str, listing_sha256: &str) -> Self {
        Self {
            schema: DOWNLOAD_STATE_SCHEMA,
            source: source.to_string(),
            listing_sha256: listing_sha256.to_string(),
            complete: false,
            done: BTreeMap::new(),
        }
    }

    fn records(&self, f: &CatalogueFile) -> bool {
        self.done
            .get(f.path.as_str())
            .is_some_and(|d| d.bytes == f.bytes && d.sha256.eq_ignore_ascii_case(&f.sha256))
    }
}

/// Read a state file (missing / unreadable → `None`).
pub fn read_download_state(dir: &Path) -> Option<DownloadState> {
    let raw = fs::read_to_string(dir.join(DOWNLOAD_STATE_FILE)).ok()?;
    serde_json::from_str(&raw).ok()
}

fn save_state(dir: &Path, state: &DownloadState) -> Result<(), String> {
    let raw = serde_json::to_vec_pretty(state).map_err(|e| format!("download state: {e}"))?;
    write_atomic(&dir.join(DOWNLOAD_STATE_FILE), &raw)
}

/// Staging state to resume from, or a fresh one. Wipes staging when it
/// belongs to another source / schema, or to an older listing whose files
/// cannot be proven by sha256.
fn open_staging(
    part: &Path,
    source: &str,
    listing_sha: &str,
    files: &[CatalogueFile],
) -> Result<DownloadState, String> {
    if let Some(prev) = read_download_state(part) {
        if prev.schema == DOWNLOAD_STATE_SCHEMA && prev.source == source {
            if prev.listing_sha256 == listing_sha {
                return Ok(prev);
            }
            // New publish: keep bytes we can prove by hash, forget the rest.
            if !files.is_empty() && files.iter().all(|f| !f.sha256.is_empty()) {
                return Ok(DownloadState::new(source, listing_sha));
            }
        }
        fs::remove_dir_all(part).map_err(|e| format!("reset {}: {e}", part.display()))?;
    } else if part.exists() {
        // No state (crash before the first save, or a pre-resume build):
        // files are only kept when the listing can prove them by hash.
        let provable = !files.is_empty() && files.iter().all(|f| !f.sha256.is_empty());
        if !provable {
            fs::remove_dir_all(part).map_err(|e| format!("reset {}: {e}", part.display()))?;
        }
    }
    Ok(DownloadState::new(source, listing_sha))
}

/// Finished cache dir whose state still matches `listing_sha`, with every
/// listed file present at its listed size.
fn finished_matches(dir: &Path, source: &str, listing_sha: Option<&str>) -> bool {
    let Some(state) = read_download_state(dir) else {
        return false;
    };
    if state.schema != DOWNLOAD_STATE_SCHEMA || state.source != source || !state.complete {
        return false;
    }
    if let Some(sha) = listing_sha {
        if state.listing_sha256 != sha {
            return false;
        }
    }
    state.done.iter().all(|(rel, d)| {
        fs::metadata(dir.join(rel))
            .map(|m| m.is_file() && (d.bytes == 0 || m.len() == d.bytes))
            .unwrap_or(false)
    })
}

/// Bring every listed file into `part`, resuming from what is already there.
/// Never deletes `final_dir`; may copy hash-proven files out of it.
#[allow(clippy::too_many_arguments)]
fn sync_files(
    src: &dyn CatalogueSource,
    remote_prefix: &str,
    part: &Path,
    final_dir: &Path,
    state: &mut DownloadState,
    files: &[CatalogueFile],
    cancel: &dyn Fn() -> bool,
    progress: &dyn Fn(DownloadProgress),
) -> Result<(), String> {
    fs::create_dir_all(part).map_err(|e| format!("mkdir {}: {e}", part.display()))?;
    let mut p = DownloadProgress {
        files_total: files.len(),
        bytes_total: files.iter().map(|f| f.bytes).sum(),
        ..Default::default()
    };
    let mut unsaved_files = 0usize;
    let mut unsaved_bytes = 0u64;
    progress(p);

    for f in files {
        let rel = safe_rel(&f.path)?;
        let target = part.join(rel);

        // 1) Already here: recorded in state (size check only), or provable
        //    by hash (state lost / new listing).
        let here = if state.records(f) {
            fs::metadata(&target)
                .map(|m| m.is_file() && (f.bytes == 0 || m.len() == f.bytes))
                .unwrap_or(false)
        } else {
            file_matches(&target, f, true)
        };
        // 2) In the previous finished cache with the same hash: copy, no fetch.
        let copied = !here
            && !f.sha256.is_empty()
            && match fs::read(final_dir.join(rel)) {
                Ok(b) if bytes_match(&b, f) => write_atomic(&target, &b).is_ok(),
                _ => false,
            };

        if here || copied {
            p.resumed_files += 1;
        } else {
            if cancel() {
                save_state(part, state)?;
                return Err("Cancelled".into());
            }
            let remote = format!("{}/{rel}", remote_prefix.trim_end_matches('/'));
            let mut attempt = 0usize;
            let bytes = loop {
                match src.get(&remote) {
                    Ok(b) => break b,
                    Err(e) if attempt < FETCH_RETRIES && is_retryable(&e) && !cancel() => {
                        attempt += 1;
                        std::thread::sleep(std::time::Duration::from_millis(
                            400 * attempt as u64,
                        ));
                    }
                    Err(e) => {
                        let _ = save_state(part, state);
                        // A listed file missing is a publish fault, not a bad code.
                        return Err(if e == "Not found" {
                            format!("Missing on server: {rel}")
                        } else {
                            e
                        });
                    }
                }
            };
            if !bytes_match(&bytes, f) {
                let _ = save_state(part, state);
                return Err(format!("Checksum mismatch: {rel}"));
            }
            if let Err(e) = write_atomic(&target, &bytes) {
                let _ = save_state(part, state);
                return Err(e);
            }
            unsaved_files += 1;
            unsaved_bytes += bytes.len() as u64;
        }

        state.done.insert(
            rel.to_string(),
            DoneFile {
                bytes: f.bytes,
                sha256: f.sha256.to_ascii_lowercase(),
            },
        );
        p.files_done += 1;
        p.bytes_done += if f.bytes > 0 {
            f.bytes
        } else {
            fs::metadata(&target).map(|m| m.len()).unwrap_or(0)
        };
        if unsaved_files >= STATE_SAVE_EVERY_FILES || unsaved_bytes >= STATE_SAVE_EVERY_BYTES {
            save_state(part, state)?;
            unsaved_files = 0;
            unsaved_bytes = 0;
        }
        progress(p);
    }
    save_state(part, state)
}

/// Drop files a new listing no longer names (a dish removed from the shelf
/// must not survive a resume) and stray `.dl` temporaries.
fn prune_unlisted(part: &Path, files: &[CatalogueFile], keep: &[&str]) -> Result<(), String> {
    let listed: std::collections::HashSet<&str> = files.iter().map(|f| f.path.as_str()).collect();
    fn walk(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, base, out);
            } else if let Ok(rel) = p.strip_prefix(base) {
                out.push(rel.to_path_buf());
            }
        }
    }
    let mut found = Vec::new();
    walk(part, part, &mut found);
    for rel in found {
        let key = rel.to_string_lossy().replace('\\', "/");
        if listed.contains(key.as_str()) || keep.contains(&key.as_str()) {
            continue;
        }
        let p = part.join(&rel);
        fs::remove_file(&p).map_err(|e| format!("remove {}: {e}", p.display()))?;
    }
    Ok(())
}

/// Swap a finished staging dir over the final cache dir.
fn promote(part: &Path, final_dir: &Path) -> Result<(), String> {
    let old = sibling(final_dir, ".old");
    if old.exists() {
        let _ = fs::remove_dir_all(&old);
    }
    if final_dir.exists() {
        fs::rename(final_dir, &old)
            .map_err(|e| format!("rename {}: {e}", final_dir.display()))?;
    }
    if let Err(e) = fs::rename(part, final_dir) {
        if old.exists() {
            let _ = fs::rename(&old, final_dir);
        }
        return Err(format!("rename {}: {e}", part.display()));
    }
    let _ = fs::remove_dir_all(&old);
    Ok(())
}

// ---------------------------------------------------------------------------
// Shelf (`public/items/`)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemsIndex {
    pub schema: u32,
    #[serde(default)]
    pub folders: Vec<String>,
    /// `folder/file.webp` — kept for older app builds.
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub count: usize,
    /// Total bytes of every photo.
    #[serde(default)]
    pub bytes: u64,
    /// Same files with size + sha256 (resume / verify). Empty on old indexes.
    #[serde(default)]
    pub entries: Vec<CatalogueFile>,
}

impl ItemsIndex {
    /// The listing to download: `entries` when published, else bare `files`.
    pub fn listing(&self) -> Vec<CatalogueFile> {
        if !self.entries.is_empty() {
            return self.entries.clone();
        }
        self.files
            .iter()
            .map(|p| CatalogueFile {
                path: p.clone(),
                ..Default::default()
            })
            .collect()
    }
}

pub fn parse_items_index(raw: &str) -> Result<ItemsIndex, String> {
    let index: ItemsIndex =
        serde_json::from_str(raw).map_err(|e| format!("items index JSON: {e}"))?;
    if index.schema != SCHEMA_V1 {
        return Err(format!(
            "Unsupported items index schema {} (want {SCHEMA_V1})",
            index.schema
        ));
    }
    for f in index.listing() {
        safe_rel(&f.path)?;
    }
    Ok(index)
}

const SHELF_SOURCE: &str = "public/items";

/// Mirror `public/items/` into `{data}/catalogue/public/items/` (resumable).
pub fn download_public_shelf(
    data_base: &Path,
    cancel: &dyn Fn() -> bool,
) -> Result<PathBuf, String> {
    download_public_shelf_from(&*default_source(), data_base, cancel, &|_| {})
}

/// [`download_public_shelf`] with an explicit source and progress callback.
pub fn download_public_shelf_from(
    src: &dyn CatalogueSource,
    data_base: &Path,
    cancel: &dyn Fn() -> bool,
    progress: &dyn Fn(DownloadProgress),
) -> Result<PathBuf, String> {
    let dest = cached_shelf_dir(data_base);
    let index_raw = match src.get_text(&format!("{SHELF_SOURCE}/index.json")) {
        Ok(raw) => raw,
        // Offline: a finished shelf is still a shelf.
        Err(e) if e.starts_with("Network:") && finished_matches(&dest, SHELF_SOURCE, None) => {
            return Ok(dest);
        }
        Err(e) => return Err(e),
    };
    let index = parse_items_index(&index_raw)?;
    let listing_sha = sha256_hex(index_raw.as_bytes());
    if finished_matches(&dest, SHELF_SOURCE, Some(&listing_sha)) {
        let files = index.listing();
        let total: u64 = files.iter().map(|f| f.bytes).sum();
        progress(DownloadProgress {
            files_done: files.len(),
            files_total: files.len(),
            bytes_done: total,
            bytes_total: total,
            resumed_files: files.len(),
        });
        return Ok(dest);
    }
    if cancel() {
        return Err("Cancelled".into());
    }
    let files = index.listing();
    let part = staging_dir(&dest);
    let mut state = open_staging(&part, SHELF_SOURCE, &listing_sha, &files)?;
    sync_files(
        src,
        SHELF_SOURCE,
        &part,
        &dest,
        &mut state,
        &files,
        cancel,
        progress,
    )?;
    prune_unlisted(&part, &files, &[DOWNLOAD_STATE_FILE])?;
    write_atomic(&part.join("index.json"), index_raw.as_bytes())?;
    state.complete = true;
    save_state(&part, &state)?;
    promote(&part, &dest)?;
    // Dish search / Import complete template see the new shelf this run.
    crate::menu_images::refresh_shelf();
    Ok(dest)
}

// ---------------------------------------------------------------------------
// Packs (`public/templates/<id>/`, `private/<code>/`)
// ---------------------------------------------------------------------------

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

fn pack_source(visibility: &CatalogueVisibility, key: &str) -> String {
    match visibility {
        CatalogueVisibility::Public => format!("public/templates/{key}"),
        CatalogueVisibility::Private => format!("private/{key}"),
    }
}

/// Download a pack into `{data}/catalogue/…` and return the pack directory.
/// For `kind: shelf`, mirrors `public/items/` and returns that items directory.
pub fn download_pack(
    data_base: &Path,
    visibility: CatalogueVisibility,
    key: &str,
    cancel: &dyn Fn() -> bool,
) -> Result<(PathBuf, CatalogueManifest, MenuTemplate), String> {
    download_pack_from(&*default_source(), data_base, visibility, key, cancel, &|_| {})
}

/// [`download_pack`] with an explicit source and progress callback.
pub fn download_pack_from(
    src: &dyn CatalogueSource,
    data_base: &Path,
    visibility: CatalogueVisibility,
    key: &str,
    cancel: &dyn Fn() -> bool,
    progress: &dyn Fn(DownloadProgress),
) -> Result<(PathBuf, CatalogueManifest, MenuTemplate), String> {
    if cancel() {
        return Err("Cancelled".into());
    }
    let cache_key = match visibility {
        CatalogueVisibility::Public => {
            let id = key.trim();
            if id.is_empty() {
                return Err("Template id required".into());
            }
            if id.contains('/') {
                return Err(format!("Unsafe catalogue path '{id}'"));
            }
            safe_rel(id)?.to_string()
        }
        CatalogueVisibility::Private => normalize_private_code(key)?,
    };
    let source = pack_source(&visibility, &cache_key);
    let dest = cache_pack_dir(data_base, &visibility, &cache_key);

    let manifest_raw = match src.get_text(&format!("{source}/manifest.json")) {
        Ok(raw) => raw,
        Err(e) if e.starts_with("Network:") && finished_matches(&dest, &source, None) => {
            // Offline retry of a pack that already finished (e.g. import was
            // cancelled after the download) — use the cache.
            let (manifest, tmpl) = load_pack_dir(&dest)?;
            if matches!(manifest.kind, CatalogueKind::Shelf) {
                let shelf = cached_shelf_dir(data_base);
                if finished_matches(&shelf, SHELF_SOURCE, None) {
                    return Ok((shelf, manifest, tmpl));
                }
                return Err(e);
            }
            return Ok((dest, manifest, tmpl));
        }
        Err(e) => return Err(e),
    };
    let manifest = parse_manifest(&manifest_raw)?;
    if cancel() {
        return Err("Cancelled".into());
    }

    if matches!(manifest.kind, CatalogueKind::Shelf) {
        if !matches!(visibility, CatalogueVisibility::Public) {
            return Err("Shelf packs are public only".into());
        }
        let items_dir = download_public_shelf_from(src, data_base, cancel, progress)?;
        let stub = MenuTemplate {
            id: manifest.id.clone(),
            title: manifest.title.clone(),
            blurb: manifest.blurb.clone(),
            categories: Vec::new(),
        };
        let stub_raw =
            serde_json::to_vec_pretty(&stub).map_err(|e| format!("stub template: {e}"))?;
        // Pointer pack (manifest + stub) — tiny, written directly.
        let mut state = DownloadState::new(&source, &sha256_hex(manifest_raw.as_bytes()));
        write_atomic(&dest.join("manifest.json"), manifest_raw.as_bytes())?;
        write_atomic(&dest.join("template.json"), &stub_raw)?;
        state.done.insert(
            "template.json".into(),
            DoneFile {
                bytes: stub_raw.len() as u64,
                sha256: String::new(),
            },
        );
        state.complete = true;
        save_state(&dest, &state)?;
        return Ok((items_dir, manifest, stub));
    }

    let template_rel = safe_rel(manifest.template.trim())
        .map_err(|_| "Catalogue manifest missing template".to_string())?
        .to_string();
    let part = staging_dir(&dest);

    // Listing: published `files`, else derived from template.json.
    let (listing_sha, mut files) = if manifest.files.is_empty() {
        let template_raw = src.get_text(&format!("{source}/{template_rel}"))?;
        verify_template_sha(&manifest, template_raw.as_bytes())?;
        let tmpl: MenuTemplate =
            serde_json::from_str(&template_raw).map_err(|e| format!("template.json: {e}"))?;
        let mut files = vec![CatalogueFile {
            path: template_rel.clone(),
            bytes: template_raw.len() as u64,
            sha256: sha256_hex(template_raw.as_bytes()),
        }];
        for rel in collect_image_rels(&tmpl) {
            safe_rel(&rel)?;
            files.push(CatalogueFile {
                path: rel,
                ..Default::default()
            });
        }
        let mut h = Sha256::new();
        h.update(manifest_raw.as_bytes());
        h.update(template_raw.as_bytes());
        (hex::encode(h.finalize()), files)
    } else {
        (sha256_hex(manifest_raw.as_bytes()), manifest.files.clone())
    };
    if !files.iter().any(|f| f.path == template_rel) {
        files.insert(
            0,
            CatalogueFile {
                path: template_rel.clone(),
                bytes: 0,
                sha256: manifest.sha256.trim().to_ascii_lowercase(),
            },
        );
    }

    if !finished_matches(&dest, &source, Some(&listing_sha)) {
        let mut state = open_staging(&part, &source, &listing_sha, &files)?;
        // Template first (small) so its photo list can be checked, then all.
        let tmpl_entry: Vec<CatalogueFile> = files
            .iter()
            .filter(|f| f.path == template_rel)
            .cloned()
            .collect();
        sync_files(src, &source, &part, &dest, &mut state, &tmpl_entry, cancel, &|_| {})?;
        let template_bytes = fs::read(part.join(&template_rel))
            .map_err(|e| format!("template.json: {e}"))?;
        verify_template_sha(&manifest, &template_bytes)?;
        let tmpl: MenuTemplate = serde_json::from_slice(&template_bytes)
            .map_err(|e| format!("template.json: {e}"))?;
        // Photos the template names but the listing forgot — fetch anyway.
        for rel in collect_image_rels(&tmpl) {
            safe_rel(&rel)?;
            if !files.iter().any(|f| f.path == rel) {
                files.push(CatalogueFile {
                    path: rel,
                    ..Default::default()
                });
            }
        }
        sync_files(src, &source, &part, &dest, &mut state, &files, cancel, progress)?;
        prune_unlisted(&part, &files, &[DOWNLOAD_STATE_FILE])?;
        write_atomic(&part.join("manifest.json"), manifest_raw.as_bytes())?;
        state.complete = true;
        save_state(&part, &state)?;
        promote(&part, &dest)?;
    } else {
        let total: u64 = files.iter().map(|f| f.bytes).sum();
        progress(DownloadProgress {
            files_done: files.len(),
            files_total: files.len(),
            bytes_done: total,
            bytes_total: total,
            resumed_files: files.len(),
        });
    }

    let (manifest, tmpl) = load_pack_dir(&dest)?;
    Ok((dest, manifest, tmpl))
}

fn verify_template_sha(manifest: &CatalogueManifest, template: &[u8]) -> Result<(), String> {
    let want = manifest.sha256.trim();
    if !want.is_empty() && !sha256_hex(template).eq_ignore_ascii_case(want) {
        return Err("Template checksum mismatch".into());
    }
    Ok(())
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
    use std::cell::RefCell;
    use std::sync::Mutex;

    /// `KHAYNJO_CATALOGUE_DIR` is process-global — serialize tests that mutate it.
    fn catalogue_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn tempfile_fixture() -> PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "khaynjo-catalogue-{}-{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Fs mirror that counts fetches and can fail like a dropped connection.
    struct TestSource {
        inner: FsSource,
        fetched: RefCell<Vec<String>>,
        /// After this many *photo* fetches, every further fetch fails.
        fail_after: Option<usize>,
        fail_with: String,
    }

    impl TestSource {
        fn new(root: &Path) -> Self {
            Self {
                inner: FsSource::new(root),
                fetched: RefCell::new(Vec::new()),
                fail_after: None,
                fail_with: "Network: connection reset".into(),
            }
        }
        fn failing_after(root: &Path, n: usize, err: &str) -> Self {
            let mut s = Self::new(root);
            s.fail_after = Some(n);
            s.fail_with = err.into();
            s
        }
        /// Photo / pack file fetches (not manifests or the items index).
        fn file_fetches(&self) -> usize {
            self.fetched
                .borrow()
                .iter()
                .filter(|r| !r.ends_with("index.json") && !r.ends_with("manifest.json"))
                .count()
        }
    }

    impl CatalogueSource for TestSource {
        fn get(&self, rel: &str) -> Result<Vec<u8>, String> {
            let is_listing = rel.ends_with("index.json") || rel.ends_with("manifest.json");
            if let Some(n) = self.fail_after {
                if !is_listing && self.file_fetches() >= n {
                    return Err(self.fail_with.clone());
                }
            }
            let out = self.inner.get(rel);
            if out.is_ok() {
                self.fetched.borrow_mut().push(rel.to_string());
            }
            out
        }
    }

    fn entry_for(root: &Path, rel: &str) -> CatalogueFile {
        let bytes = fs::read(root.join(rel)).unwrap();
        CatalogueFile {
            path: rel.to_string(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        }
    }

    /// A shelf of `n` photos (distinct bytes) with a full `entries` index.
    fn write_shelf(cdn: &Path, names: &[(&str, &[u8])]) {
        let items = cdn.join("public/items");
        let mut files = Vec::new();
        let mut entries = Vec::new();
        let mut folders = Vec::new();
        for (rel, body) in names {
            let p = items.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, body).unwrap();
            files.push(rel.to_string());
            entries.push(entry_for(&items, rel));
            let folder = rel.split('/').next().unwrap().to_string();
            if !folders.contains(&folder) {
                folders.push(folder);
            }
        }
        let index = ItemsIndex {
            schema: 1,
            count: files.len(),
            bytes: entries.iter().map(|e| e.bytes).sum(),
            folders,
            files,
            entries,
        };
        fs::write(
            items.join("index.json"),
            serde_json::to_string_pretty(&index).unwrap(),
        )
        .unwrap();
    }

    fn photo(i: usize) -> Vec<u8> {
        format!("RIFF{i:04}WEBP-{}", "x".repeat(32 + i)).into_bytes()
    }

    fn ten_photos() -> Vec<(String, Vec<u8>)> {
        (0..10)
            .map(|i| {
                let folder = if i < 5 { "hot-beverages" } else { "snacks" };
                (format!("{folder}/dish-{i}.webp"), photo(i))
            })
            .collect()
    }

    fn write_ten(cdn: &Path) {
        let photos = ten_photos();
        let refs: Vec<(&str, &[u8])> = photos
            .iter()
            .map(|(p, b)| (p.as_str(), b.as_slice()))
            .collect();
        write_shelf(cdn, &refs);
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
            join_url(DEFAULT_CATALOGUE_BASE, "public/index.json"),
            format!("{DEFAULT_CATALOGUE_BASE}/public/index.json")
        );
        assert!(join_url(DEFAULT_CATALOGUE_BASE, "/public/templates/rajdhani")
            .ends_with("/v1/catalogue/public/templates/rajdhani"));
        assert!(join_url(&format!("{DEFAULT_CATALOGUE_BASE}/"), "private/123456789012")
            .ends_with("/v1/catalogue/private/123456789012"));
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
        assert!(man.files.is_empty());
    }

    #[test]
    fn listed_paths_must_stay_inside_the_pack() {
        for bad in ["../x.webp", "/etc/passwd", "a/../../b", "a\\b.webp", ""] {
            assert!(safe_rel(bad).is_err(), "{bad} should be rejected");
        }
        assert!(safe_rel("images/indian-mains/paneer-tikka.webp").is_ok());
        let err = parse_manifest(
            r#"{"schema":1,"id":"x","title":"X","files":[{"path":"../../evil","bytes":1}]}"#,
        )
        .unwrap_err();
        assert!(err.contains("Unsafe"), "{err}");
        let err = parse_items_index(
            r#"{"schema":1,"files":["ok/a.webp","../b.webp"]}"#,
        )
        .unwrap_err();
        assert!(err.contains("Unsafe"), "{err}");
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
        assert!(!staging_dir(&dest).exists(), "staging promoted away");
        assert!(read_download_state(&dest).unwrap().complete);
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
        assert_eq!(dest, cached_shelf_dir(&data));
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

    // --- resume ---------------------------------------------------------

    #[test]
    fn shelf_cancel_then_restart_fetches_only_the_rest() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        let data = cdn.join("data");
        let shelf = cached_shelf_dir(&data);

        // Attempt 1: cancel after 4 photos.
        let src = TestSource::new(&cdn);
        let err = download_public_shelf_from(&src, &data, &|| src.file_fetches() >= 4, &|_| {})
            .unwrap_err();
        assert_eq!(err, "Cancelled");
        assert_eq!(src.file_fetches(), 4);
        assert!(!shelf.exists(), "half shelf must not be visible to the scanner");
        let part = staging_dir(&shelf);
        let state = read_download_state(&part).expect("state saved on cancel");
        assert_eq!(state.done.len(), 4);
        assert!(!state.complete);

        // Attempt 2 = fresh process: only reads disk + a new source.
        let src2 = TestSource::new(&cdn);
        let seen = RefCell::new(DownloadProgress::default());
        let dest = download_public_shelf_from(&src2, &data, &|| false, &|p| *seen.borrow_mut() = p)
            .unwrap();
        assert_eq!(src2.file_fetches(), 6, "4 resumed, 6 fetched");
        assert_eq!(dest, shelf);
        assert!(!part.exists());
        let last = *seen.borrow();
        assert_eq!(last.files_done, 10);
        assert_eq!(last.files_total, 10);
        assert_eq!(last.resumed_files, 4);
        assert_eq!(last.bytes_done, last.bytes_total);
        assert!(last.bytes_total > 0);
        for (rel, body) in ten_photos() {
            assert_eq!(fs::read(shelf.join(&rel)).unwrap(), body, "{rel}");
        }
        assert!(read_download_state(&shelf).unwrap().complete);
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn shelf_network_drop_then_resume() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        let data = cdn.join("data");

        let flaky = TestSource::failing_after(&cdn, 7, "Network: connection reset");
        let err = download_public_shelf_from(&flaky, &data, &|| false, &|_| {}).unwrap_err();
        assert!(err.starts_with("Network:"), "{err}");
        assert_eq!(flaky.file_fetches(), 7);

        let src = TestSource::new(&cdn);
        download_public_shelf_from(&src, &data, &|| false, &|_| {}).unwrap();
        assert_eq!(src.file_fetches(), 3);
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn crash_before_state_save_resumes_by_hash() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        let data = cdn.join("data");
        let shelf = cached_shelf_dir(&data);
        let part = staging_dir(&shelf);

        let src = TestSource::new(&cdn);
        let _ = download_public_shelf_from(&src, &data, &|| src.file_fetches() >= 5, &|_| {});
        // Simulate a kill before the state file reached disk.
        fs::remove_file(part.join(DOWNLOAD_STATE_FILE)).unwrap();
        // …and a torn temp file from the write in flight.
        fs::create_dir_all(part.join("snacks")).unwrap();
        fs::write(part.join("snacks/dish-9.webp.dl"), b"torn").unwrap();

        let src2 = TestSource::new(&cdn);
        download_public_shelf_from(&src2, &data, &|| false, &|_| {}).unwrap();
        assert_eq!(src2.file_fetches(), 5, "five proven by sha256");
        assert!(!shelf.join("snacks/dish-9.webp.dl").exists(), "temp pruned");
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn corrupt_staged_file_is_fetched_again() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        let data = cdn.join("data");
        let part = staging_dir(&cached_shelf_dir(&data));

        let src = TestSource::new(&cdn);
        let _ = download_public_shelf_from(&src, &data, &|| src.file_fetches() >= 6, &|_| {});
        // Disk damage after the state recorded it as done (size changes).
        fs::write(part.join("hot-beverages/dish-1.webp"), b"bad").unwrap();

        let src2 = TestSource::new(&cdn);
        download_public_shelf_from(&src2, &data, &|| false, &|_| {}).unwrap();
        assert_eq!(src2.file_fetches(), 5, "4 missing + 1 damaged");
        let shelf = cached_shelf_dir(&data);
        assert_eq!(fs::read(shelf.join("hot-beverages/dish-1.webp")).unwrap(), photo(1));
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn finished_shelf_is_reused_and_offline_ok() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        let data = cdn.join("data");

        let src = TestSource::new(&cdn);
        download_public_shelf_from(&src, &data, &|| false, &|_| {}).unwrap();
        assert_eq!(src.file_fetches(), 10);

        let again = TestSource::new(&cdn);
        download_public_shelf_from(&again, &data, &|| false, &|_| {}).unwrap();
        assert_eq!(again.file_fetches(), 0, "same listing → no photo fetches");

        // Offline: the index itself cannot load, the finished shelf stands.
        struct Offline;
        impl CatalogueSource for Offline {
            fn get(&self, _rel: &str) -> Result<Vec<u8>, String> {
                Err("Network: offline".into())
            }
        }
        let dest = download_public_shelf_from(&Offline, &data, &|| false, &|_| {}).unwrap();
        assert!(dest.join("snacks/dish-7.webp").is_file());
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn new_publish_keeps_matching_photos_and_drops_removed() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        let data = cdn.join("data");
        let shelf = cached_shelf_dir(&data);
        download_public_shelf_from(&TestSource::new(&cdn), &data, &|| false, &|_| {}).unwrap();

        // Republish: drop dish-0, change dish-1, add dish-10.
        fs::remove_dir_all(cdn.join("public/items")).unwrap();
        let mut photos = ten_photos();
        photos.remove(0);
        photos[0].1 = b"RIFF-new-dish-1-WEBP".to_vec();
        photos.push(("snacks/dish-10.webp".into(), photo(10)));
        let refs: Vec<(&str, &[u8])> = photos
            .iter()
            .map(|(p, b)| (p.as_str(), b.as_slice()))
            .collect();
        write_shelf(&cdn, &refs);

        let src = TestSource::new(&cdn);
        download_public_shelf_from(&src, &data, &|| false, &|_| {}).unwrap();
        assert_eq!(src.file_fetches(), 2, "changed + added only");
        assert!(!shelf.join("hot-beverages/dish-0.webp").exists(), "removed dish pruned");
        assert_eq!(
            fs::read(shelf.join("hot-beverages/dish-1.webp")).unwrap(),
            b"RIFF-new-dish-1-WEBP"
        );
        assert!(shelf.join("snacks/dish-10.webp").is_file());
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn checksum_mismatch_is_refused() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        // Server file changes after the index was written.
        fs::write(cdn.join("public/items/snacks/dish-8.webp"), b"tampered").unwrap();
        let data = cdn.join("data");
        let err = download_public_shelf_from(&TestSource::new(&cdn), &data, &|| false, &|_| {})
            .unwrap_err();
        assert!(err.contains("dish-8"), "{err}");
        assert!(!cached_shelf_dir(&data).exists());
        let _ = fs::remove_dir_all(cdn);
    }

    /// App-template pack with a published `files` list (sizes + sha256).
    fn write_listed_pack(cdn: &Path, id: &str, n: usize) -> PathBuf {
        let pack = cdn.join("public/templates").join(id);
        fs::create_dir_all(pack.join("images/snacks")).unwrap();
        let mut items = Vec::new();
        for i in 0..n {
            let rel = format!("images/snacks/dish-{i}.webp");
            fs::write(pack.join(&rel), photo(i)).unwrap();
            items.push(format!(
                r#"{{"name":"Dish {i}","price":{p},"image":"{rel}"}}"#,
                p = 10 + i
            ));
        }
        let template = format!(
            r#"{{"id":"{id}","title":"Listed","categories":[{{"name":"Snacks","image":"images/snacks/dish-0.webp","items":[{}]}}]}}"#,
            items.join(",")
        );
        fs::write(pack.join("template.json"), &template).unwrap();
        let mut files = vec![entry_for(&pack, "template.json")];
        for i in 0..n {
            files.push(entry_for(&pack, &format!("images/snacks/dish-{i}.webp")));
        }
        let man = CatalogueManifest {
            schema: 1,
            id: id.into(),
            title: "Listed".into(),
            blurb: String::new(),
            kind: CatalogueKind::AppTemplate,
            visibility: CatalogueVisibility::Public,
            applies_shop_defaults: false,
            shop_defaults_id: String::new(),
            template: "template.json".into(),
            items_index: String::new(),
            bytes: files.iter().map(|f| f.bytes).sum(),
            sha256: sha256_hex(template.as_bytes()),
            files,
        };
        fs::write(
            pack.join("manifest.json"),
            serde_json::to_string_pretty(&man).unwrap(),
        )
        .unwrap();
        pack
    }

    #[test]
    fn app_template_pack_resumes_after_cancel() {
        let cdn = tempfile_fixture();
        write_listed_pack(&cdn, "listed", 8);
        let data = cdn.join("data");
        let dest = cache_pack_dir(&data, &CatalogueVisibility::Public, "listed");

        let src = TestSource::new(&cdn);
        // template.json + 2 photos, then cancel.
        let err = download_pack_from(
            &src,
            &data,
            CatalogueVisibility::Public,
            "listed",
            &|| src.file_fetches() >= 3,
            &|_| {},
        )
        .unwrap_err();
        assert_eq!(err, "Cancelled");
        assert!(!dest.exists());
        assert!(staging_dir(&dest).join("template.json").is_file());

        let src2 = TestSource::new(&cdn);
        let (dir, man, tmpl) = download_pack_from(
            &src2,
            &data,
            CatalogueVisibility::Public,
            "listed",
            &|| false,
            &|_| {},
        )
        .unwrap();
        assert_eq!(src2.file_fetches(), 6, "6 photos left");
        assert_eq!(dir, dest);
        assert_eq!(man.files.len(), 9);
        assert_eq!(tmpl.item_count(), 8);
        for i in 0..8 {
            assert!(dir.join(format!("images/snacks/dish-{i}.webp")).is_file());
        }
        // Second import (e.g. after the import step was cancelled): no fetch.
        let src3 = TestSource::new(&cdn);
        download_pack_from(&src3, &data, CatalogueVisibility::Public, "listed", &|| false, &|_| {})
            .unwrap();
        assert_eq!(src3.file_fetches(), 0);
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn listing_change_of_legacy_pack_restarts_cleanly() {
        let cdn = tempfile_fixture();
        let code = "111122223333";
        let pack = cdn.join("private").join(code);
        fs::create_dir_all(pack.join("images")).unwrap();
        let manifest = r#"{"schema":1,"id":"p","title":"P","visibility":"private","template":"template.json"}"#;
        fs::write(pack.join("manifest.json"), manifest).unwrap();
        let tmpl = |img: &str| {
            format!(
                r#"{{"id":"p","title":"P","categories":[{{"name":"C","items":[{{"name":"A","image":"images/a.webp"}},{{"name":"B","image":"{img}"}}]}}]}}"#
            )
        };
        fs::write(pack.join("template.json"), tmpl("images/b.webp")).unwrap();
        fs::write(pack.join("images/a.webp"), b"A").unwrap();
        fs::write(pack.join("images/b.webp"), b"B").unwrap();
        fs::write(pack.join("images/c.webp"), b"C").unwrap();
        let data = cdn.join("data");

        let src = TestSource::new(&cdn);
        let _ = download_pack_from(
            &src,
            &data,
            CatalogueVisibility::Private,
            code,
            &|| src.file_fetches() >= 2,
            &|_| {},
        );
        // Template changes on the server before the retry (legacy pack, no sha list).
        fs::write(pack.join("template.json"), tmpl("images/c.webp")).unwrap();
        let src2 = TestSource::new(&cdn);
        let (dir, _, t) = download_pack_from(
            &src2,
            &data,
            CatalogueVisibility::Private,
            code,
            &|| false,
            &|_| {},
        )
        .unwrap();
        assert!(dir.join("images/c.webp").is_file());
        assert!(!dir.join("images/b.webp").exists());
        assert_eq!(t.categories[0].items[1].image, "images/c.webp");
        let _ = fs::remove_dir_all(cdn);
    }

    /// Tiny HTTP/1.1 file server for `HttpSource` tests. Requests past
    /// `drop_after` get their socket closed with no reply (dead Wi-Fi).
    fn serve_dir(
        root: PathBuf,
        drop_after: Option<usize>,
    ) -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
        use std::io::{BufRead, BufReader, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = hits.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                loop {
                    let mut h = String::new();
                    if reader.read_line(&mut h).is_err() || h == "\r\n" || h.is_empty() {
                        break;
                    }
                }
                let n = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if drop_after.is_some_and(|d| n >= d) {
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    continue;
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/");
                let file = root.join(path.trim_start_matches('/'));
                let reply = match fs::read(&file) {
                    Ok(body) => {
                        let mut r = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .into_bytes();
                        r.extend_from_slice(&body);
                        r
                    }
                    Err(_) => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        .to_vec(),
                };
                let _ = stream.write_all(&reply);
            }
        });
        (base, hits)
    }

    #[test]
    fn http_source_downloads_resumes_and_maps_404() {
        let cdn = tempfile_fixture();
        write_ten(&cdn);
        let data = cdn.join("data");

        // Server dies after the index + 3 photos (+ retries also fail).
        let (base, _) = serve_dir(cdn.clone(), Some(4));
        let err = download_public_shelf_from(&HttpSource::new(&base), &data, &|| false, &|_| {})
            .unwrap_err();
        assert!(err.starts_with("Network:"), "{err}");
        let part = staging_dir(&cached_shelf_dir(&data));
        assert_eq!(read_download_state(&part).unwrap().done.len(), 3);

        // Connection back: resume over HTTP fetches the other 7.
        let (base, hits) = serve_dir(cdn.clone(), None);
        download_public_shelf_from(&HttpSource::new(&base), &data, &|| false, &|_| {}).unwrap();
        assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 1 + 7);
        for (rel, body) in ten_photos() {
            assert_eq!(fs::read(cached_shelf_dir(&data).join(&rel)).unwrap(), body);
        }

        // A bad private code is "Not found" (UI: Code not found)…
        let src = HttpSource::new(&base);
        assert_eq!(src.get("private/000000000000/manifest.json").unwrap_err(), "Not found");
        // …but a listed photo missing on the server is a publish fault.
        fs::remove_file(cdn.join("public/items/snacks/dish-9.webp")).unwrap();
        fs::remove_dir_all(cached_shelf_dir(&data)).unwrap();
        let err = download_public_shelf_from(&src, &data, &|| false, &|_| {}).unwrap_err();
        assert_eq!(err, "Missing on server: snacks/dish-9.webp");
        let _ = fs::remove_dir_all(cdn);
    }

    #[test]
    fn staged_tree_publishes_complete_listings() {
        // The in-repo static tree (publish-catalogue.sh output) must carry
        // full file lists so devices can resume; skipped when not staged.
        let cdn = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../sites/khaynjo/static/v1/catalogue");
        let Ok(index_raw) = fs::read_to_string(cdn.join("public/index.json")) else {
            return;
        };
        let index = parse_public_index(&index_raw).unwrap();
        for entry in &index.templates {
            let raw =
                fs::read_to_string(cdn.join(format!("public/templates/{}/manifest.json", entry.id)))
                    .unwrap();
            let man = parse_manifest(&raw).unwrap();
            assert_eq!(man.id, entry.id);
            assert_eq!(man.bytes, entry.bytes, "{}: index bytes = manifest bytes", entry.id);
            if man.kind == CatalogueKind::Shelf {
                let Ok(items_raw) = fs::read_to_string(cdn.join("public/items/index.json")) else {
                    continue; // shelf photos are synced at publish time, not committed
                };
                let items = parse_items_index(&items_raw).unwrap();
                assert_eq!(items.entries.len(), items.files.len());
                assert_eq!(items.bytes, man.bytes);
                assert_eq!(items.entries.iter().map(|e| e.bytes).sum::<u64>(), items.bytes);
                continue;
            }
            assert!(!man.files.is_empty(), "{}: manifest lists files", entry.id);
            let pack = cdn.join(format!("public/templates/{}", entry.id));
            let mut total = 0;
            for f in &man.files {
                let b = fs::read(pack.join(&f.path)).unwrap_or_else(|_| panic!("{}", f.path));
                assert!(bytes_match(&b, f), "{}: {} matches listing", entry.id, f.path);
                total += f.bytes;
            }
            assert_eq!(total, man.bytes, "{}: bytes = sum of files", entry.id);
            let tmpl: MenuTemplate =
                serde_json::from_slice(&fs::read(pack.join(&man.template)).unwrap()).unwrap();
            for rel in collect_image_rels(&tmpl) {
                assert!(
                    man.files.iter().any(|f| f.path == rel),
                    "{}: {rel} listed",
                    entry.id
                );
            }
        }
    }
}

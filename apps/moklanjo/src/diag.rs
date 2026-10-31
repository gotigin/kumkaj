//! Local diagnostics — privacy-first, owner-controllable.
//!
//! **Collect on device freely** (ring buffer + rotating JSONL). Secrets
//! (PIN, password, tokens) are always redacted. Free-text that looks like
//! phone / email is scrubbed.
//!
//! **Leaving the device** (export-for-share / “send”) requires an explicit
//! owner approval for that package. Nothing is uploaded automatically;
//! there is no third-party sink. Approval is single-use per package.
//!
//! Duplicated per app (no shared product crate). See docs/DEV.md § Diagnostics.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Soft cap on in-memory events (export still reads the file).
const RING_CAP: usize = 4_000;
/// Rotate the JSONL when it grows past this.
const FILE_ROTATE_BYTES: u64 = 2 * 1024 * 1024;
/// Support session default length.
const SUPPORT_DEFAULT: Duration = Duration::from_secs(30 * 60);
/// How long a send-approval token stays valid once granted.
const SEND_APPROVAL_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "error" | "err" => Some(Self::Error),
            "warn" | "warning" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cat {
    System,
    Nav,
    Store,
    Lan,
    Print,
    Ui,
    Sync,
    Pay,
}

impl Cat {
    pub const ALL: [Cat; 8] = [
        Cat::System,
        Cat::Nav,
        Cat::Store,
        Cat::Lan,
        Cat::Print,
        Cat::Ui,
        Cat::Sync,
        Cat::Pay,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Nav => "nav",
            Self::Store => "store",
            Self::Lan => "lan",
            Self::Print => "print",
            Self::Ui => "ui",
            Self::Sync => "sync",
            Self::Pay => "pay",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "system" => Some(Self::System),
            "nav" => Some(Self::Nav),
            "store" => Some(Self::Store),
            "lan" => Some(Self::Lan),
            "print" => Some(Self::Print),
            "ui" => Some(Self::Ui),
            "sync" => Some(Self::Sync),
            "pay" => Some(Self::Pay),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagConfig {
    /// Minimum level written (production default: Info).
    pub min_level: Level,
    /// Categories enabled. Empty = all.
    pub categories: Vec<Cat>,
    /// When set (unix secs), treat min_level as Debug until then.
    pub support_until_unix: Option<u64>,
    pub max_file_bytes: u64,
    pub max_ring: usize,
}

impl Default for DiagConfig {
    fn default() -> Self {
        Self {
            min_level: Level::Info,
            categories: Vec::new(),
            support_until_unix: None,
            max_file_bytes: FILE_ROTATE_BYTES,
            max_ring: RING_CAP,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub ts: String,
    pub level: Level,
    pub cat: Cat,
    pub code: String,
    pub msg: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct DiagStatus {
    pub dir: PathBuf,
    pub events_on_disk: u64,
    pub events_in_ring: usize,
    pub file_bytes: u64,
    pub min_level: Level,
    pub support_active: bool,
    pub support_remaining_secs: u64,
    pub send_approval_ready: bool,
    pub send_approval_remaining_secs: u64,
}

struct State {
    dir: PathBuf,
    app_id: String,
    app_version: String,
    config: DiagConfig,
    ring: VecDeque<Event>,
    /// Owner granted share-off-device for one export; expires.
    send_approved_until: Option<Instant>,
    file: Option<File>,
}

fn global() -> &'static Mutex<Option<State>> {
    static G: OnceLock<Mutex<Option<State>>> = OnceLock::new();
    G.get_or_init(|| Mutex::new(None))
}

fn now_rfc3339_pretty() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Scrub secrets and obvious PII from free text before it hits disk.
pub fn redact_text(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    // Never retain anything that looks like a PIN/password payload.
    if lower.contains("password")
        || lower.contains("passwd")
        || lower.contains("\"pin\"")
        || lower.contains("pin=")
        || lower.contains("pin:")
        || lower.contains("pairing code")
        || lower.contains("bearer ")
        || lower.contains("authorization:")
    {
        return "[redacted-secret]".into();
    }

    let mut out = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '-' || chars[i] == ' ')
            {
                i += 1;
            }
            let digit_count = chars[start..i]
                .iter()
                .filter(|c| c.is_ascii_digit())
                .count();
            // Phone / card-ish runs (6+ digits) → mask. Short counts (qty, ports) stay.
            if digit_count >= 6 {
                out.push_str("[num]");
            } else {
                for c in &chars[start..i] {
                    out.push(*c);
                }
            }
            continue;
        }
        // Crude email mask.
        if chars[i] == '@' && i > 0 {
            // rewrite back a bit
            let mut j = out.len();
            while j > 0 {
                let prev = out[..j].chars().last().unwrap_or(' ');
                if prev.is_ascii_alphanumeric() || prev == '.' || prev == '_' || prev == '-' {
                    let ch_len = prev.len_utf8();
                    j -= ch_len;
                } else {
                    break;
                }
            }
            out.truncate(j);
            out.push_str("[email]");
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '-')
            {
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Opaque short id for correlation (never the raw business id).
pub fn safe_id(raw: &str) -> String {
    use sha2::{Digest, Sha256};
    let dig = Sha256::digest(raw.as_bytes());
    hex::encode(&dig[..6])
}

fn config_path(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

fn events_path(dir: &Path) -> PathBuf {
    dir.join("events.jsonl")
}

fn load_config(dir: &Path) -> DiagConfig {
    let path = config_path(dir);
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => DiagConfig::default(),
    }
}

fn save_config(dir: &Path, cfg: &DiagConfig) {
    let _ = fs::create_dir_all(dir);
    if let Ok(raw) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(config_path(dir), raw);
    }
}

fn open_events_file(dir: &Path) -> Option<File> {
    let _ = fs::create_dir_all(dir);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path(dir))
        .ok()
}

fn rotate_if_needed(dir: &Path, max_bytes: u64, file: &mut Option<File>) {
    let path = events_path(dir);
    let len = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if len < max_bytes {
        return;
    }
    let bak = dir.join("events.jsonl.1");
    let _ = fs::remove_file(&bak);
    let _ = fs::rename(&path, &bak);
    *file = open_events_file(dir);
}

fn support_active(cfg: &DiagConfig) -> bool {
    match cfg.support_until_unix {
        Some(until) => unix_now() < until,
        None => false,
    }
}

fn effective_min_level(cfg: &DiagConfig) -> Level {
    if support_active(cfg) {
        Level::Debug
    } else {
        cfg.min_level
    }
}

fn cat_enabled(cfg: &DiagConfig, cat: Cat) -> bool {
    cfg.categories.is_empty() || cfg.categories.contains(&cat)
}

/// Initialise diagnostics under `{data_dir}/diag`. Safe to call more than once.
pub fn init(data_dir: &Path, app_id: &str, app_version: &str) {
    let dir = data_dir.join("diag");
    let _ = fs::create_dir_all(&dir);
    let config = load_config(&dir);
    let file = open_events_file(&dir);
    let mut ring = VecDeque::with_capacity(config.max_ring.min(RING_CAP));
    // Warm ring from tail of current file (best-effort).
    if let Ok(f) = File::open(events_path(&dir)) {
        let lines: Vec<String> = BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .collect();
        let start = lines.len().saturating_sub(config.max_ring.min(RING_CAP));
        for line in &lines[start..] {
            if let Ok(ev) = serde_json::from_str::<Event>(line) {
                ring.push_back(ev);
            }
        }
    }
    let mut g = global().lock().unwrap();
    *g = Some(State {
        dir,
        app_id: app_id.into(),
        app_version: app_version.into(),
        config,
        ring,
        send_approved_until: None,
        file,
    });
    drop(g);
    install_panic_hook();
    emit(
        Level::Info,
        Cat::System,
        "diag_init",
        "diagnostics ready (local only; send needs approval)",
        &[],
    );
}

fn install_panic_hook() {
    static HOOKED: OnceLock<()> = OnceLock::new();
    HOOKED.get_or_init(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let loc = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "unknown".into());
            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "non-string panic".into()
            };
            let msg = redact_text(&format!("panic at {loc}: {payload}"));
            // Best-effort write even if mutex is poisoned.
            if let Ok(mut g) = global().lock() {
                if let Some(st) = g.as_mut() {
                    let ev = Event {
                        ts: now_rfc3339_pretty(),
                        level: Level::Error,
                        cat: Cat::System,
                        code: "panic".into(),
                        msg: msg.clone(),
                        fields: vec![("loc".into(), redact_text(&loc))],
                    };
                    write_event(st, ev);
                    let panic_path = st.dir.join("last_panic.txt");
                    let _ = fs::write(
                        &panic_path,
                        format!("{}\n{}\n", now_rfc3339_pretty(), msg),
                    );
                }
            }
            prev(info);
        }));
    });
}

fn write_event(st: &mut State, ev: Event) {
    if let Ok(line) = serde_json::to_string(&ev) {
        rotate_if_needed(&st.dir, st.config.max_file_bytes, &mut st.file);
        if let Some(f) = st.file.as_mut() {
            let _ = writeln!(f, "{line}");
            let _ = f.flush();
        }
    }
    st.ring.push_back(ev);
    let cap = st.config.max_ring.min(RING_CAP).max(64);
    while st.ring.len() > cap {
        st.ring.pop_front();
    }
}

/// Core emit — redacts message and field values.
pub fn emit(level: Level, cat: Cat, code: &str, msg: &str, fields: &[(&str, &str)]) {
    let mut g = match global().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let Some(st) = g.as_mut() else {
        return;
    };
    if level > effective_min_level(&st.config) {
        return;
    }
    if !cat_enabled(&st.config, cat) {
        return;
    }
    let code = code.chars().take(64).collect::<String>();
    let msg = redact_text(msg);
    let mut cleaned = Vec::with_capacity(fields.len());
    for (k, v) in fields {
        let key = k.to_ascii_lowercase();
        let val = if key.contains("pin")
            || key.contains("pass")
            || key.contains("secret")
            || key.contains("token")
            || key.contains("auth")
        {
            "[redacted]".into()
        } else if key.ends_with("_id") || key == "id" {
            // Prefer opaque ids in logs.
            if v.len() > 12 {
                safe_id(v)
            } else {
                redact_text(v)
            }
        } else {
            redact_text(v)
        };
        cleaned.push(((*k).to_string(), val));
    }
    let ev = Event {
        ts: now_rfc3339_pretty(),
        level,
        cat,
        code,
        msg,
        fields: cleaned,
    };
    write_event(st, ev);
}

pub fn error(cat: Cat, code: &str, msg: impl AsRef<str>) {
    emit(Level::Error, cat, code, msg.as_ref(), &[]);
}
pub fn warn(cat: Cat, code: &str, msg: impl AsRef<str>) {
    emit(Level::Warn, cat, code, msg.as_ref(), &[]);
}
pub fn info(cat: Cat, code: &str, msg: impl AsRef<str>) {
    emit(Level::Info, cat, code, msg.as_ref(), &[]);
}
pub fn debug(cat: Cat, code: &str, msg: impl AsRef<str>) {
    emit(Level::Debug, cat, code, msg.as_ref(), &[]);
}
pub fn breadcrumb(code: &str, msg: impl AsRef<str>) {
    emit(Level::Info, Cat::Nav, code, msg.as_ref(), &[]);
}

pub fn with_config(f: impl FnOnce(&mut DiagConfig)) {
    let mut g = match global().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(st) = g.as_mut() {
        f(&mut st.config);
        save_config(&st.dir, &st.config);
    }
}

pub fn set_min_level(level: Level) {
    with_config(|c| c.min_level = level);
    info(
        Cat::System,
        "diag_level",
        format!("min_level={}", level.as_str()),
    );
}

pub fn set_categories(cats: Vec<Cat>) {
    with_config(|c| c.categories = cats);
    info(Cat::System, "diag_categories", "category filter updated");
}

pub fn start_support_session(duration: Option<Duration>) {
    let secs = duration.unwrap_or(SUPPORT_DEFAULT).as_secs();
    with_config(|c| c.support_until_unix = Some(unix_now().saturating_add(secs)));
    info(
        Cat::System,
        "support_session_start",
        format!("support session {secs}s — Debug level while active"),
    );
}

pub fn end_support_session() {
    with_config(|c| c.support_until_unix = None);
    info(Cat::System, "support_session_end", "support session ended");
}

/// Owner explicitly allows the next off-device share of a diagnostics package.
/// Single-use: consumed by [`export_for_send`]. Expires after a short TTL.
pub fn approve_send() {
    let mut g = match global().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(st) = g.as_mut() {
        st.send_approved_until = Some(Instant::now() + SEND_APPROVAL_TTL);
        let ev = Event {
            ts: now_rfc3339_pretty(),
            level: Level::Info,
            cat: Cat::System,
            code: "send_approved".into(),
            msg: "owner approved one diagnostics share off this device".into(),
            fields: vec![("ttl_secs".into(), SEND_APPROVAL_TTL.as_secs().to_string())],
        };
        write_event(st, ev);
    }
}

pub fn revoke_send_approval() {
    let mut g = match global().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(st) = g.as_mut() {
        st.send_approved_until = None;
    }
}

fn send_approval_active(st: &State) -> (bool, u64) {
    match st.send_approved_until {
        Some(until) => {
            let now = Instant::now();
            if now < until {
                (true, (until - now).as_secs())
            } else {
                (false, 0)
            }
        }
        None => (false, 0),
    }
}

pub fn status() -> Option<DiagStatus> {
    let g = global().lock().ok()?;
    let st = g.as_ref()?;
    let path = events_path(&st.dir);
    let file_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let events_on_disk = fs::read_to_string(&path)
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count() as u64)
        .unwrap_or(0);
    let (send_ok, send_rem) = send_approval_active(st);
    let support = support_active(&st.config);
    let support_rem = st
        .config
        .support_until_unix
        .map(|u| u.saturating_sub(unix_now()))
        .unwrap_or(0);
    Some(DiagStatus {
        dir: st.dir.clone(),
        events_on_disk,
        events_in_ring: st.ring.len(),
        file_bytes,
        min_level: effective_min_level(&st.config),
        support_active: support,
        support_remaining_secs: if support { support_rem } else { 0 },
        send_approval_ready: send_ok,
        send_approval_remaining_secs: send_rem,
    })
}

pub fn clear_logs() -> Result<(), String> {
    let mut g = global()
        .lock()
        .map_err(|_| "diagnostics lock poisoned".to_string())?;
    let st = g
        .as_mut()
        .ok_or_else(|| "diagnostics not initialised".to_string())?;
    st.ring.clear();
    st.file = None;
    let _ = fs::remove_file(events_path(&st.dir));
    let _ = fs::remove_file(st.dir.join("events.jsonl.1"));
    let _ = fs::remove_file(st.dir.join("last_panic.txt"));
    st.file = open_events_file(&st.dir);
    let ev = Event {
        ts: now_rfc3339_pretty(),
        level: Level::Info,
        cat: Cat::System,
        code: "diag_cleared".into(),
        msg: "diagnostic logs cleared by owner".into(),
        fields: vec![],
    };
    write_event(st, ev);
    Ok(())
}

fn collect_events_json(st: &State) -> Result<Vec<serde_json::Value>, String> {
    let mut out = Vec::new();
    let src = events_path(&st.dir);
    if src.is_file() {
        if let Ok(f) = File::open(&src) {
            for line in BufReader::new(f).lines().flatten() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                    out.push(v);
                }
            }
        }
    }
    let bak = st.dir.join("events.jsonl.1");
    if bak.is_file() {
        if let Ok(f) = File::open(&bak) {
            for line in BufReader::new(f).lines().flatten() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                    out.push(v);
                }
            }
        }
    }
    if out.is_empty() {
        for ev in &st.ring {
            out.push(serde_json::to_value(ev).map_err(|e| e.to_string())?);
        }
    }
    Ok(out)
}

fn manifest_json(st: &State, for_send: bool) -> serde_json::Value {
    serde_json::json!({
        "app_id": st.app_id,
        "app_version": st.app_version,
        "exported_at": now_rfc3339_pretty(),
        "for_send": for_send,
        "privacy": {
            "secrets_redacted": true,
            "phones_emails_scrubbed": true,
            "no_automatic_upload": true,
            "no_third_party": true,
            "purpose": "Resolve product issues and improve the app. Not sold or shared with third parties."
        },
        "config": st.config,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
}

fn write_package(st: &State, dest: &Path, for_send: bool) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let manifest = manifest_json(st, for_send);
    fs::write(
        dest.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let readme = if for_send {
        "Kumkaj diagnostics package (OWNER-APPROVED SHARE)\n\n\
         This folder was exported after the owner approved sharing it off the device.\n\
         Contents are redacted (no PINs/passwords; phone/email patterns scrubbed).\n\
         Use only to fix or improve the product. Do not forward to third parties.\n\
         Delete after the issue is resolved.\n\n\
         Platforms: Linux, Windows, macOS, Android — same format.\n"
    } else {
        "Kumkaj diagnostics package (LOCAL COPY)\n\n\
         This folder stays under your control. Sharing it off the device requires\n\
         a separate owner approval in Settings → Diagnostics.\n\n\
         Platforms: Linux, Windows, macOS, Android — same format.\n"
    };
    fs::write(dest.join("README.txt"), readme).map_err(|e| e.to_string())?;
    let src = events_path(&st.dir);
    if src.is_file() {
        fs::copy(&src, dest.join("events.jsonl")).map_err(|e| e.to_string())?;
    } else {
        let mut f = File::create(dest.join("events.jsonl")).map_err(|e| e.to_string())?;
        for ev in &st.ring {
            let line = serde_json::to_string(ev).map_err(|e| e.to_string())?;
            writeln!(f, "{line}").map_err(|e| e.to_string())?;
        }
    }
    let bak = st.dir.join("events.jsonl.1");
    if bak.is_file() {
        let _ = fs::copy(&bak, dest.join("events.jsonl.1"));
    }
    let panic_path = st.dir.join("last_panic.txt");
    if panic_path.is_file() {
        let _ = fs::copy(&panic_path, dest.join("last_panic.txt"));
    }
    Ok(())
}

/// Single-file `.kumkaj-diag.json` — works with Android save/share dialogs and
/// desktop USB / Downloads (folder exports alone are awkward on Android).
fn write_bundle_file(st: &State, path: &Path, for_send: bool) -> Result<(), String> {
    let panic_text = fs::read_to_string(st.dir.join("last_panic.txt")).unwrap_or_default();
    let bundle = serde_json::json!({
        "format": "kumkaj-diag-v1",
        "manifest": manifest_json(st, for_send),
        "events": collect_events_json(st)?,
        "last_panic": if panic_text.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(panic_text) },
    });
    fs::write(
        path,
        serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// Paths from an export: folder (desktop convenience) + single bundle file
/// (every platform — Android ACTION_CREATE_DOCUMENT, Windows/Linux/macOS save).
#[derive(Clone, Debug)]
pub struct ExportPaths {
    pub folder: PathBuf,
    /// Prefer this for share/save dialogs on all platforms.
    pub bundle: PathBuf,
}

fn export_inner(for_send: bool) -> Result<ExportPaths, String> {
    let mut g = global()
        .lock()
        .map_err(|_| "diagnostics lock poisoned".to_string())?;
    let st = g
        .as_mut()
        .ok_or_else(|| "diagnostics not initialised".to_string())?;
    if for_send {
        let (ok, _) = send_approval_active(st);
        if !ok {
            return Err(
                "Share blocked: owner must approve sending diagnostics off this device first"
                    .into(),
            );
        }
        st.send_approved_until = None;
    }
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let kind = if for_send { "share" } else { "local" };
    let folder = st.dir.join(format!("export-{kind}-{stamp}"));
    let bundle = st.dir.join(format!(
        "{}-diag-{kind}-{stamp}.kumkaj-diag.json",
        st.app_id.replace('.', "-")
    ));
    write_package(st, &folder, for_send)?;
    write_bundle_file(st, &bundle, for_send)?;
    let ev = Event {
        ts: now_rfc3339_pretty(),
        level: Level::Info,
        cat: Cat::System,
        code: if for_send {
            "export_for_send".into()
        } else {
            "export_local".into()
        },
        msg: format!(
            "diagnostics export folder={} bundle={}",
            folder.display(),
            bundle.display()
        ),
        fields: vec![
            ("os".into(), std::env::consts::OS.into()),
            ("for_send".into(), for_send.to_string()),
        ],
    };
    write_event(st, ev);
    Ok(ExportPaths { folder, bundle })
}

/// Local-only export (no send approval). Folder + single-file bundle.
pub fn export_local() -> Result<ExportPaths, String> {
    export_inner(false)
}

/// Off-device share package. **Requires** a prior [`approve_send`] that has not expired.
/// Consumes the approval (one package per approval).
pub fn export_for_send() -> Result<ExportPaths, String> {
    export_inner(true)
}

/// Wipe the whole diag directory (uninstall / owner clear-all).
pub fn wipe_dir(data_dir: &Path) {
    let dir = data_dir.join("diag");
    let _ = fs::remove_dir_all(&dir);
    if let Ok(mut g) = global().lock() {
        *g = None;
    }
}

/// Human summary for Settings.
pub fn status_text() -> String {
    match status() {
        None => "Diagnostics not started yet.".into(),
        Some(s) => {
            let support = if s.support_active {
                format!("Support session ON (~{} min left)", s.support_remaining_secs / 60)
            } else {
                "Support session off".into()
            };
            let send = if s.send_approval_ready {
                format!(
                    "Share approval ready (~{}s) — export once",
                    s.send_approval_remaining_secs
                )
            } else {
                "Share off-device: not approved".into()
            };
            format!(
                "Level {} · {} events · {:.1} KB on disk\n{support}\n{send}\nFolder: {}",
                s.min_level.as_str(),
                s.events_on_disk,
                s.file_bytes as f64 / 1024.0,
                s.dir.display()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;
    use std::time::Duration;

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<StdMutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| StdMutex::new(())).lock().unwrap_or_else(|e| e.into_inner())
    }

    fn temp_dir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "kumkaj-diag-test-{}-{}",
            std::process::id(),
            unix_now()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn setup() -> PathBuf {
        let base = temp_dir();
        // Drop any prior global so this test owns diagnostics.
        if let Ok(mut g) = global().lock() {
            *g = None;
        }
        init(&base, "test.app", "0.0.0");
        base
    }

    fn teardown(base: PathBuf) {
        wipe_dir(&base);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn redact_masks_phone_and_email_keeps_short_nums() {
        let s = redact_text("call 9876543210 or me@shop.example qty 12 port 8080");
        assert!(s.contains("[num]"), "{s}");
        assert!(s.contains("[email]"), "{s}");
        assert!(s.contains("12"), "{s}");
        assert!(s.contains("8080"), "{s}");
        assert!(!s.contains("9876543210"), "{s}");
        assert!(!s.contains("me@shop"), "{s}");
    }

    #[test]
    fn redact_kills_passwordish() {
        assert_eq!(redact_text("user password=secret"), "[redacted-secret]");
        assert_eq!(redact_text("pin:1234"), "[redacted-secret]");
    }

    #[test]
    fn safe_id_stable_and_short() {
        let a = safe_id("sale-uuid-aaaa");
        let b = safe_id("sale-uuid-aaaa");
        assert_eq!(a, b);
        assert_eq!(a.len(), 12);
        assert_ne!(a, safe_id("sale-uuid-bbbb"));
    }

    #[test]
    fn collect_without_send_approval_export_for_send_fails() {
        let _g = test_lock();
        let base = setup();
        info(Cat::Store, "unit", "hello");
        assert!(export_for_send().is_err());
        let local = export_local().expect("local export");
        assert!(local.folder.join("events.jsonl").is_file());
        assert!(local.folder.join("manifest.json").is_file());
        assert!(local.bundle.is_file());
        let raw = fs::read_to_string(&local.bundle).unwrap();
        assert!(raw.contains("kumkaj-diag-v1"));
        assert!(raw.contains("\"os\""));
        teardown(base);
    }

    #[test]
    fn send_requires_approval_then_consumes_it() {
        let _g = test_lock();
        let base = setup();
        warn(Cat::Lan, "lan_err", "timeout");
        approve_send();
        let share = export_for_send().expect("approved share");
        assert!(share.folder.join("README.txt").is_file());
        let readme = fs::read_to_string(share.folder.join("README.txt")).unwrap();
        assert!(readme.contains("OWNER-APPROVED"));
        assert!(share.bundle.is_file());
        assert!(export_for_send().is_err());
        teardown(base);
    }

    #[test]
    fn support_session_raises_effective_level() {
        let _g = test_lock();
        let base = setup();
        set_min_level(Level::Error);
        debug(Cat::Ui, "hidden", "should drop");
        start_support_session(Some(Duration::from_secs(120)));
        let st = status().unwrap();
        assert!(st.support_active);
        assert_eq!(st.min_level, Level::Debug);
        debug(Cat::Ui, "visible", "should keep");
        let text = fs::read_to_string(base.join("diag/events.jsonl")).unwrap();
        assert!(text.contains("visible"), "{text}");
        assert!(!text.contains("\"code\":\"hidden\""), "{text}");
        end_support_session();
        teardown(base);
    }

    #[test]
    fn clear_logs_resets_file() {
        let _g = test_lock();
        let base = setup();
        error(Cat::Pay, "pay_fail", "card declined path");
        clear_logs().unwrap();
        let text = fs::read_to_string(base.join("diag/events.jsonl")).unwrap();
        assert!(text.contains("diag_cleared"));
        assert!(!text.contains("pay_fail"));
        teardown(base);
    }

    #[test]
    fn category_filter() {
        let _g = test_lock();
        let base = setup();
        set_categories(vec![Cat::Lan]);
        // set_categories itself logs under System — clear filter noise then re-apply.
        clear_logs().unwrap();
        set_categories(vec![Cat::Lan]);
        info(Cat::Lan, "keep", "lan ok");
        info(Cat::Print, "drop", "print ok");
        let text = fs::read_to_string(base.join("diag/events.jsonl")).unwrap();
        assert!(text.contains("\"code\":\"keep\""), "{text}");
        assert!(!text.contains("\"code\":\"drop\""), "{text}");
        set_categories(vec![]);
        teardown(base);
    }

    #[test]
    fn level_ordering() {
        assert!(Level::Error < Level::Warn);
        assert!(Level::Warn < Level::Info);
        assert!(Level::Info < Level::Debug);
    }
}

//! Printing (local free POS — no cloud), the same on every device that can
//! hold the books or take orders.
//!
//! Owner nouns (Settings):
//! - **Printer** — one OS device in the pool (destination only).
//! - **Profile** — slip look ([`PrintFormat`]: Thermal / Portrait / Landscape).
//! - **Output** — under Receipt or AWB label: ordered printers + that place's
//!   Profile. Tries the first that works. Empty = don't print.
//!
//! No kitchen station groups. `printers.json` beside the books. Jobs go
//! through one worker; failures via [`take_problems`]. Tests never reach a
//! printer ([`PrintSink`]).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "android"))]
use std::process::Command;
use std::sync::{Mutex, OnceLock, mpsc};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Where printing goes. The app prints through the OS; unit tests capture
/// the jobs instead (2026-10-23: a LAN test's cancel slip went to the
/// developer's real printer on every `cargo test`). Capture keeps the test
/// as strict as paper — it asserts the exact text and count that would
/// have printed — without the OS ever seeing a job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrintSink {
    /// Hand the file to the OS (`lp`, print dialog, default opener).
    Os,
    /// Keep the job in memory for [`take_captured`]; nothing reaches the OS.
    Capture,
    /// Write the file only.
    Off,
}

/// One print job as it would have reached the printer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintJob {
    pub kind: PrintKind,
    pub path: PathBuf,
    pub text: String,
}

/// What a job is, so it goes to the printer meant for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrintKind {
    /// Booking receipts, bills, day slips.
    Receipt,
    /// AWB shipping labels.
    Label,
}

/// Which ordered route a job uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteKey {
    Receipt,
    Label,
}

pub const FORMAT_STANDARD: &str = "standard";
pub const FORMAT_STANDARD_PORTRAIT: &str = "standard_portrait";
pub const FORMAT_STANDARD_LANDSCAPE: &str = "standard_landscape";
pub const FORMAT_KOT: &str = "kot";

pub fn is_standard_document(id: &str) -> bool {
    matches!(
        id,
        FORMAT_STANDARD | FORMAT_STANDARD_PORTRAIT | FORMAT_STANDARD_LANDSCAPE
    )
}

pub fn normalize_format_id(id: &str) -> &str {
    if id == FORMAT_STANDARD {
        FORMAT_STANDARD_PORTRAIT
    } else {
        id
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrintFormat {
    pub id: String,
    pub label: String,
    #[serde(default = "default_true")]
    pub document: bool,
    #[serde(default)]
    pub landscape: bool,
    #[serde(default = "default_em")]
    pub em: f32,
    #[serde(default = "default_line")]
    pub line: u32,
    #[serde(default = "default_baseline")]
    pub baseline: f32,
    #[serde(default = "default_side")]
    pub side: f32,
}

fn default_true() -> bool { true }
fn default_em() -> f32 { STD_EM_NORMAL }
fn default_line() -> u32 { 40 }
fn default_baseline() -> f32 { STD_EM_NORMAL + 2.0 }
fn default_side() -> f32 { 12.0 }

/// Document type-size presets. Defaults favour common OS printers (narrow
/// trays) — wider side inset, ~40-char wrap.
pub const STD_EM_SMALL: f32 = 18.0;
pub const STD_EM_NORMAL: f32 = 20.0;
pub const STD_EM_LARGE: f32 = 26.0;

impl PrintFormat {
    pub fn standard() -> Self {
        Self::standard_portrait()
    }
    /// Non-deletable built-in. Portrait OS document — works on most queues.
    pub fn standard_portrait() -> Self {
        Self {
            id: FORMAT_STANDARD_PORTRAIT.into(),
            label: "Standard Portrait".into(),
            document: true,
            landscape: false,
            em: STD_EM_NORMAL,
            line: 40,
            baseline: STD_EM_NORMAL + 2.0,
            side: 12.0,
        }
    }
    /// Non-deletable built-in. Landscape OS document.
    pub fn standard_landscape() -> Self {
        Self {
            id: FORMAT_STANDARD_LANDSCAPE.into(),
            label: "Standard Landscape".into(),
            document: true,
            landscape: true,
            em: STD_EM_NORMAL,
            line: 40,
            baseline: STD_EM_NORMAL + 2.0,
            side: 12.0,
        }
    }
    /// Non-deletable built-in. Thermal / label slip metrics.
    pub fn kot() -> Self {
        Self {
            id: FORMAT_KOT.into(),
            label: "Thermal".into(),
            document: false,
            landscape: false,
            em: 20.0,
            line: 32,
            baseline: 22.0,
            side: 4.0,
        }
    }
    pub fn raster_metrics(&self) -> crate::receipt_raster::RasterMetrics {
        crate::receipt_raster::RasterMetrics { em: self.em, line: self.line as usize, baseline: self.baseline, side: self.side }
    }
    pub fn ensure_builtins(list: &mut Vec<PrintFormat>) {
        for f in list.iter_mut() {
            if f.id == FORMAT_STANDARD {
                f.id = FORMAT_STANDARD_PORTRAIT.into();
                if f.label.trim().eq_ignore_ascii_case("standard") {
                    f.label = "Standard Portrait".into();
                }
                f.document = true;
                f.landscape = false;
            }
        }
        if !list.iter().any(|f| f.id == FORMAT_STANDARD_PORTRAIT) {
            list.insert(0, PrintFormat::standard_portrait());
        }
        if !list.iter().any(|f| f.id == FORMAT_STANDARD_LANDSCAPE) {
            let idx = list
                .iter()
                .position(|f| f.id == FORMAT_STANDARD_PORTRAIT)
                .map(|i| i + 1)
                .unwrap_or(0);
            list.insert(idx, PrintFormat::standard_landscape());
        }
        if !list.iter().any(|f| f.id == FORMAT_KOT) {
            let idx = list
                .iter()
                .position(|f| f.id == FORMAT_STANDARD_LANDSCAPE)
                .map(|i| i + 1)
                .unwrap_or(list.len());
            list.insert(idx, PrintFormat::kot());
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrinterMode {
    #[default]
    Standard,
    Kot,
}

impl PrinterMode {
    pub fn from_legacy_wire_str(s: &str) -> Self {
        match s { "esc_pos" | "kot" => Self::Kot, _ => Self::Standard }
    }
}

/// Destination only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedPrinter {
    pub id: String,
    pub label: String,
    pub target: String,
    #[serde(default, alias = "wire", alias = "mode", skip_serializing)]
    pub _legacy_mode: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintRoutes {
    #[serde(default)]
    pub receipt: Vec<String>,
    #[serde(default)]
    pub label: Vec<String>,
    #[serde(default = "default_format_standard")]
    pub receipt_format: String,
    #[serde(default = "default_format_kot")]
    pub label_format: String,
}

fn default_format_standard() -> String { FORMAT_STANDARD_PORTRAIT.into() }
fn default_format_kot() -> String { FORMAT_KOT.into() }

impl Default for PrintRoutes {
    fn default() -> Self {
        Self {
            receipt: Vec::new(),
            label: Vec::new(),
            receipt_format: FORMAT_STANDARD_PORTRAIT.into(),
            label_format: FORMAT_KOT.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DevicePrinters {
    pub printers: Vec<NamedPrinter>,
    #[serde(default)]
    pub formats: Vec<PrintFormat>,
    pub routes: PrintRoutes,
}

impl Default for DevicePrinters {
    fn default() -> Self {
        Self {
            printers: Vec::new(),
            formats: vec![
                PrintFormat::standard_portrait(),
                PrintFormat::standard_landscape(),
                PrintFormat::kot(),
            ],
            routes: PrintRoutes::default(),
        }
    }
}

impl<'de> Deserialize<'de> for DevicePrinters {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(device_printers_from_value(value))
    }
}

fn device_printers_from_value(value: serde_json::Value) -> DevicePrinters {
    if value.get("printers").is_some() {
        #[derive(Deserialize)]
        struct New {
            #[serde(default)]
            printers: Vec<NamedPrinter>,
            #[serde(default)]
            formats: Vec<PrintFormat>,
            #[serde(default)]
            routes: PrintRoutes,
        }
        let parsed: New = serde_json::from_value(value).unwrap_or_else(|_| New {
            printers: Vec::new(),
            formats: Vec::new(),
            routes: PrintRoutes::default(),
        });
        let mut formats = parsed.formats;
        PrintFormat::ensure_builtins(&mut formats);
        let mut out = DevicePrinters {
            printers: parsed.printers,
            formats,
            routes: parsed.routes,
        };
        scrub_fake_default_printers(&mut out);
        if out.routes.receipt_format.trim().is_empty() {
            out.routes.receipt_format = FORMAT_STANDARD_PORTRAIT.into();
        }
        if out.routes.label_format.trim().is_empty() {
            out.routes.label_format = FORMAT_KOT.into();
        }
        if out.routes.receipt_format.trim() == FORMAT_STANDARD {
            out.routes.receipt_format = FORMAT_STANDARD_PORTRAIT.into();
        }
        PrintFormat::ensure_builtins(&mut out.formats);
        return out;
    }
    let receipt = value
        .get("receipt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    migrate_legacy(&receipt)
}

fn scrub_fake_default_printers(cfg: &mut DevicePrinters) {
    let fake: Vec<String> = cfg
        .printers
        .iter()
        .filter(|p| {
            p.target.trim().is_empty()
                && (p.id == "default" || p.label.eq_ignore_ascii_case("default"))
        })
        .map(|p| p.id.clone())
        .collect();
    for id in fake {
        cfg.remove_printer(&id);
    }
}

/// Turn the old single `receipt` string into a pool + routes.
/// Blank receipt → empty route (no invented "Default" printer).
pub fn migrate_legacy(receipt: &str) -> DevicePrinters {
    let mut printers = Vec::new();
    let receipt_ids = if receipt.trim().is_empty() {
        Vec::new()
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        printers.push(NamedPrinter {
            id: id.clone(),
            label: "Receipt printer".into(),
            target: receipt.trim().to_string(),
            _legacy_mode: None,
        });
        vec![id]
    };
    DevicePrinters {
        printers,
        formats: vec![
            PrintFormat::standard_portrait(),
            PrintFormat::standard_landscape(),
            PrintFormat::kot(),
        ],
        routes: PrintRoutes {
            receipt: receipt_ids,
            label: Vec::new(),
            receipt_format: FORMAT_STANDARD_PORTRAIT.into(),
            label_format: FORMAT_KOT.into(),
        },
    }
}

impl DevicePrinters {
    pub fn route_for(&self, key: RouteKey) -> &[String] {
        match key {
            RouteKey::Receipt => &self.routes.receipt,
            RouteKey::Label => &self.routes.label,
        }
    }

    pub fn targets_for(&self, key: RouteKey) -> Vec<PrinterTarget> {
        self.route_for(key)
            .iter()
            .filter_map(|id| {
                self.printers
                    .iter()
                    .find(|p| p.id == *id)
                    .map(|p| parse_target(&p.target))
            })
            .collect()
    }

    pub fn printer_by_id(&self, id: &str) -> Option<&NamedPrinter> {
        self.printers.iter().find(|p| p.id == id)
    }

    pub fn add_printer(&mut self, label: &str, target: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let tgt = target.trim().to_string();
        let label = {
            let t = label.trim();
            if !t.is_empty() {
                t.to_string()
            } else if !tgt.is_empty() {
                tgt.clone()
            } else {
                "Printer".into()
            }
        };
        self.printers.push(NamedPrinter {
            id: id.clone(),
            label,
            target: tgt,
            _legacy_mode: None,
        });
        id
    }

    pub fn format_id_for(&self, key: RouteKey) -> String {
        match key {
            RouteKey::Receipt => {
                let f = self.routes.receipt_format.trim();
                if f.is_empty() {
                    FORMAT_STANDARD_PORTRAIT.into()
                } else {
                    normalize_format_id(f).to_string()
                }
            }
            RouteKey::Label => {
                let f = self.routes.label_format.trim();
                if f.is_empty() { FORMAT_KOT.into() } else { f.to_string() }
            }
        }
    }

    pub fn format_by_id(&self, id: &str) -> PrintFormat {
        let id = normalize_format_id(id);
        self.formats.iter().find(|f| f.id == id).cloned().unwrap_or_else(|| {
            if id == FORMAT_KOT {
                PrintFormat::kot()
            } else if id == FORMAT_STANDARD_LANDSCAPE {
                PrintFormat::standard_landscape()
            } else {
                PrintFormat::standard_portrait()
            }
        })
    }

    pub fn format_for(&self, key: RouteKey) -> PrintFormat {
        self.format_by_id(&self.format_id_for(key))
    }

    pub fn set_route_format(&mut self, key: RouteKey, format_id: &str) {
        let id = format_id.trim();
        if id.is_empty() { return; }
        match key {
            RouteKey::Receipt => self.routes.receipt_format = id.to_string(),
            RouteKey::Label => self.routes.label_format = id.to_string(),
        }
    }

    pub fn add_format_from(&mut self, from_id: &str, label: &str) -> String {
        PrintFormat::ensure_builtins(&mut self.formats);
        let base = self.format_by_id(from_id);
        let id = format!("custom-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        let label = { let t = label.trim(); if t.is_empty() { format!("Custom ({})", base.label) } else { t.to_string() } };
        self.formats.push(PrintFormat {
            id: id.clone(),
            label,
            document: base.document,
            landscape: base.landscape,
            em: base.em,
            line: base.line,
            baseline: base.baseline,
            side: base.side,
        });
        id
    }

    pub fn destinations_for(&self, key: RouteKey) -> Vec<PrintDest> {
        let format = self.format_for(key);
        self.route_for(key)
            .iter()
            .filter_map(|id| {
                let p = self.printers.iter().find(|p| p.id == *id)?;
                Some(PrintDest { target: parse_target(&p.target), format: format.clone() })
            })
            .collect()
    }

    pub fn remove_printer(&mut self, id: &str) {
        self.printers.retain(|p| p.id != id);
        self.routes.receipt.retain(|x| x != id);
        self.routes.label.retain(|x| x != id);
    }

    pub fn route_mut(&mut self, key: RouteKey) -> &mut Vec<String> {
        match key {
            RouteKey::Receipt => &mut self.routes.receipt,
            RouteKey::Label => &mut self.routes.label,
        }
    }
}

/// Where one job goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrinterTarget {
    Default,
    Named(String),
    Network(String),
    Bluetooth(String),
    Usb(String),
    AndroidPrint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintDest {
    pub target: PrinterTarget,
    pub format: PrintFormat,
}

/// Port network receipt printers listen on for raw jobs.
pub const RAW_PRINT_PORT: u16 = 9100;

/// A printer as typed: blank = default, an IPv4 address (with or without a
/// port) = network printer, anything else = a system printer's name.
pub fn parse_target(typed: &str) -> PrinterTarget {
    let t = typed.trim();
    if t.is_empty() {
        return PrinterTarget::Default;
    }
    let lower = t.to_ascii_lowercase();
    if lower == "android-print:" || lower == "android-print" {
        return PrinterTarget::AndroidPrint;
    }
    if lower.starts_with("bt:") {
        return PrinterTarget::Bluetooth(t.to_string());
    }
    if lower.starts_with("usb:") {
        return PrinterTarget::Usb(t.to_string());
    }
    if let Ok(ip) = t.parse::<std::net::Ipv4Addr>() {
        return PrinterTarget::Network(format!("{ip}:{RAW_PRINT_PORT}"));
    }
    if let Ok(addr) = t.parse::<std::net::SocketAddrV4>() {
        return PrinterTarget::Network(addr.to_string());
    }
    if let Some((host, port)) = t.rsplit_once(':') {
        if !host.is_empty()
            && port.parse::<u16>().is_ok()
            && host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return PrinterTarget::Network(t.to_string());
        }
    }
    PrinterTarget::Named(t.to_string())
}

pub fn can_print_format(target: &PrinterTarget, format: &PrintFormat) -> bool {
    match target {
        PrinterTarget::Network(_) | PrinterTarget::Bluetooth(_) | PrinterTarget::Usb(_) => true,
        PrinterTarget::AndroidPrint => format.document && cfg!(target_os = "android"),
        PrinterTarget::Default | PrinterTarget::Named(_) => {
            if cfg!(target_os = "android") { format.document } else { true }
        }
    }
}

pub fn can_print(target: &PrinterTarget) -> bool {
    can_print_format(target, &PrintFormat::standard()) || can_print_format(target, &PrintFormat::kot())
}

pub fn can_print_mode(target: &PrinterTarget, mode: PrinterMode) -> bool {
    let f = match mode { PrinterMode::Standard => PrintFormat::standard(), PrinterMode::Kot => PrintFormat::kot() };
    can_print_format(target, &f)
}

fn default_route_for(kind: PrintKind) -> RouteKey {
    match kind {
        PrintKind::Receipt => RouteKey::Receipt,
        PrintKind::Label => RouteKey::Label,
    }
}

/// Whether `kind` prints on paper from this device as things are set now.
pub fn prints_here(kind: PrintKind) -> bool {
    prints_route(default_route_for(kind))
}

/// Whether this route has at least one printable target.
pub fn prints_route(key: RouteKey) -> bool {
    printers()
        .destinations_for(key)
        .iter()
        .any(|d| can_print_format(&d.target, &d.format))
}

static PRINTERS: Mutex<Option<DevicePrinters>> = Mutex::new(None);

/// Use these printers for the jobs that follow.
pub fn configure(printers: DevicePrinters) {
    *PRINTERS.lock().unwrap() = Some(printers);
}

/// The printers in force on this device.
pub fn printers() -> DevicePrinters {
    PRINTERS.lock().unwrap().clone().unwrap_or_default()
}

fn printers_path(base: &Path) -> PathBuf {
    base.join("printers.json")
}

/// This install's saved printers (none saved = the defaults).
pub fn load_device_printers(base: &Path) -> DevicePrinters {
    let mut printers: DevicePrinters = fs::read_to_string(printers_path(base))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    if printers.routes.receipt_format.trim() == FORMAT_STANDARD {
        printers.routes.receipt_format = FORMAT_STANDARD_PORTRAIT.into();
    }
    PrintFormat::ensure_builtins(&mut printers.formats);
    printers
}

/// Keep `printers` for this install and use them from now on.
pub fn save_device_printers(base: &Path, printers: &DevicePrinters) -> Result<(), String> {
    fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(printers).map_err(|e| e.to_string())?;
    fs::write(printers_path(base), raw).map_err(|e| e.to_string())?;
    configure(printers.clone());
    Ok(())
}

/// Load and use the printers saved for the install whose data lives in `base`.
pub fn use_device_dir(base: &Path) {
    configure(load_device_printers(base));
}

static SINK: std::sync::Mutex<Option<PrintSink>> = std::sync::Mutex::new(None);

thread_local! {
    // Per thread, so tests running in parallel never see each other's jobs.
    static CAPTURED: std::cell::RefCell<Vec<PrintJob>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The sink in force: set explicitly ([`set_sink`]), else
/// `KUMKAJ_PRINT_SINK=os|capture|off`, else Capture under `cargo test`
/// unit tests and Os in the app.
pub fn sink() -> PrintSink {
    if let Some(s) = *SINK.lock().unwrap() {
        return s;
    }
    match std::env::var("KUMKAJ_PRINT_SINK").as_deref() {
        Ok("os") => PrintSink::Os,
        Ok("capture") => PrintSink::Capture,
        Ok("off") => PrintSink::Off,
        _ if cfg!(test) => PrintSink::Capture,
        _ => PrintSink::Os,
    }
}

/// Choose where printing goes for this process (integration tests, demos).
pub fn set_sink(sink: PrintSink) {
    *SINK.lock().unwrap() = Some(sink);
}

/// Jobs captured on this thread since the last call (oldest first).
pub fn take_captured() -> Vec<PrintJob> {
    CAPTURED.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// What happened to a receipt: always written, sometimes sent to a printer.
pub struct ReceiptShare {
    pub path: PathBuf,
    /// True when the job went to a printer's queue. False where this device
    /// cannot print it (Android with no network printer: the caller offers
    /// the share sheet) and in tests.
    pub opened: bool,
}

/// Where receipts land. `app_files` is Makepad's data dir (`Cx::get_data_dir`,
/// `Activity.getFilesDir` on Android) — the temp dir is not somewhere an
/// Android file manager will look.
fn receipt_dir(app_files: Option<&Path>) -> PathBuf {
    match app_files {
        Some(base) => base.join("receipts"),
        None => std::env::temp_dir().join("kumkaj-receipts"),
    }
}

/// Write receipt text next to the shop's data and return the file.
pub fn write_receipt(text: &str, app_files: Option<&Path>) -> Result<PathBuf, String> {
    let dir = receipt_dir(app_files);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // Unique per job: two station KOTs sent in the same second must not
    // overwrite each other's file.
    let path = dir.join(format!(
        "receipt-{}-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S"),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Write the slip, then send it along the default route for `kind`.
pub fn print_text(kind: PrintKind, text: &str, app_files: Option<&Path>) -> Result<ReceiptShare, String> {
    print_route(kind, default_route_for(kind), text, app_files)
}

/// Write the slip, then try the ordered printers on `route` (fallback, not fan-out).
/// Empty route: file is written, nothing is queued.
pub fn print_route(
    kind: PrintKind,
    route: RouteKey,
    text: &str,
    app_files: Option<&Path>,
) -> Result<ReceiptShare, String> {
    let path = write_receipt(text, app_files)?;
    let opened = match sink() {
        PrintSink::Os => {
            let dests: Vec<PrintDest> = printers()
                .destinations_for(route)
                .into_iter()
                .filter(|d| can_print_format(&d.target, &d.format))
                .collect();
            if dests.is_empty() {
                false
            } else {
                queue(Job {
                    dests,
                    path: path.clone(),
                    text: text.to_string(),
                });
                true
            }
        }
        PrintSink::Capture => {
            if !printers().route_for(route).is_empty() {
                CAPTURED.with(|c| {
                    c.borrow_mut().push(PrintJob {
                        kind,
                        path: path.clone(),
                        text: text.to_string(),
                    })
                });
            }
            false
        }
        PrintSink::Off => false,
    };
    Ok(ReceiptShare { path, opened })
}

struct Job {
    /// Try in order; stop after the first success.
    dests: Vec<PrintDest>,
    path: PathBuf,
    text: String,
}

static PROBLEMS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static NOTIFY: Mutex<Option<fn()>> = Mutex::new(None);

/// Called (from the print thread) when a problem is waiting for the screen.
pub fn set_notify(hook: fn()) {
    *NOTIFY.lock().unwrap() = Some(hook);
}

fn note_problem(problem: String) {
    {
        let mut all = PROBLEMS.lock().unwrap();
        // The same failure on every KOT is said once.
        if all.last() == Some(&problem) {
            return;
        }
        all.push(problem);
    }
    if let Some(hook) = *NOTIFY.lock().unwrap() {
        hook();
    }
}

/// Printing problems since the last call (oldest first).
pub fn take_problems() -> Vec<String> {
    std::mem::take(&mut *PROBLEMS.lock().unwrap())
}

/// One thread prints, in the order the jobs came.
fn queue(job: Job) {
    static TX: OnceLock<Mutex<mpsc::Sender<Job>>> = OnceLock::new();
    let tx = TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        let _ = std::thread::Builder::new().name("print".into()).spawn(move || {
            for job in rx {
                let mut last_err: Option<(String, String)> = None;
                for dest in &job.dests {
                    match send_to_printer(&dest.format, &dest.target, &job.path, &job.text) {
                        Ok(()) => {
                            last_err = None;
                            break;
                        }
                        Err(e) => last_err = Some((describe(&dest.target), e)),
                    }
                }
                if let Some((who, e)) = last_err {
                    note_problem(format!("{who} did not print: {e}"));
                }
            }
        });
        Mutex::new(tx)
    });
    let _ = tx.lock().unwrap().send(job);
}

/// How a person names the printer.
pub fn describe(target: &PrinterTarget) -> String {
    match target {
        PrinterTarget::Default => "This computer's usual printer (blank address)".into(),
        PrinterTarget::Named(name) => format!("Printer \"{name}\""),
        PrinterTarget::Network(addr) => format!("Network printer {addr}"),
        PrinterTarget::Bluetooth(mac) => format!("Bluetooth printer {mac}"),
        PrinterTarget::Usb(id) => format!("USB printer {id}"),
        PrinterTarget::AndroidPrint => "Android print service".into(),
    }
}

fn send_to_printer(
    format: &PrintFormat,
    target: &PrinterTarget,
    path: &Path,
    text: &str,
) -> Result<(), String> {
    let bytes = escpos_job_format(format, text);
    match target {
        PrinterTarget::Network(addr) => {
            if bytes.len() > ESC_POS_JOB_MAX {
                return Err(format!("print job too large ({} bytes)", bytes.len()));
            }
            let sock: std::net::SocketAddr = addr.parse().map_err(|_| format!("{addr} is not a printer address"))?;
            let mut stream = std::net::TcpStream::connect_timeout(&sock, Duration::from_secs(4))
                .map_err(|e| format!("cannot reach it ({e}) — is it on and on this network?"))?;
            let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
            stream.write_all(&bytes).map_err(|e| e.to_string())?;
            stream.flush().map_err(|e| e.to_string())
        }
        PrinterTarget::Bluetooth(mac) => crate::print_bt::write_bytes(mac, &bytes),
        PrinterTarget::Usb(id) => crate::print_usb::write_bytes(id, &bytes),
        PrinterTarget::AndroidPrint => {
            if !format.document {
                return Err(format!(
                    "format \"{}\" needs Network, Bluetooth, or USB",
                    format.label
                ));
            }
            print_android_document(text, format, format.landscape)
        }
        PrinterTarget::Default => {
            if format.document {
                print_system_standard(None, path, text, format, format.landscape)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB",
                    format.label
                ))
            } else {
                print_system_raw(None, &bytes)
            }
        }
        PrinterTarget::Named(name) => {
            if format.document {
                print_system_standard(Some(name), path, text, format, format.landscape)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB",
                    format.label
                ))
            } else {
                print_system_raw(Some(name), &bytes)
            }
        }
    }
}

fn print_android_document(text: &str, format: &PrintFormat, landscape: bool) -> Result<(), String> {
    // Honour format em / line / baseline / side (not fixed RasterMetrics::standard).
    let png = crate::receipt_raster::png_metrics(text, &format.raster_metrics())?;
    #[cfg(target_os = "android")]
    {
        crate::print_android::print_png("Moklanjo", &png, landscape)
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (png, landscape);
        Err("Android print service is only on Android".into())
    }
}

/// Standard (document) jobs: always render with bundled Noto fonts to a PNG,
/// then hand that image to the OS printer — same glyphs on every platform.
fn print_system_standard(
    name: Option<&str>,
    path: &Path,
    text: &str,
    format: &PrintFormat,
    landscape: bool,
) -> Result<(), String> {
    let png = crate::receipt_raster::png_metrics(text, &format.raster_metrics())?;
    let png_path = path.with_extension("png");
    fs::write(&png_path, &png).map_err(|e| e.to_string())?;
    #[cfg(target_os = "android")]
    {
        let _ = name;
        crate::print_android::print_png("Moklanjo", &png, landscape)
    }
    #[cfg(not(target_os = "android"))]
    {
        print_system_image(name, &png_path, landscape)
    }
}

pub fn print_network(addr: &str, text: &str) -> Result<(), String> {
    print_network_mode(addr, PrinterMode::Kot, text)
}

pub fn print_network_mode(addr: &str, mode: PrinterMode, text: &str) -> Result<(), String> {
    let bytes = escpos_job_mode(mode, text);
    if bytes.len() > ESC_POS_JOB_MAX {
        return Err(format!(
            "print job too large ({} bytes) — refused so the printer is not flooded",
            bytes.len()
        ));
    }
    let sock: std::net::SocketAddr = addr
        .parse()
        .map_err(|_| format!("{addr} is not a printer address"))?;
    let mut stream = std::net::TcpStream::connect_timeout(&sock, Duration::from_secs(4))
        .map_err(|e| format!("cannot reach it ({e}) — is it on and on this network?"))?;
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    // Chunk writes — some heads drop mid-job when the whole payload lands at once.
    const CHUNK: usize = 4096;
    for piece in bytes.chunks(CHUNK) {
        stream.write_all(piece).map_err(|e| e.to_string())?;
    }
    stream.flush().map_err(|e| e.to_string())
}

/// Cap for one ESC/POS payload (text or raster bands).
const ESC_POS_JOB_MAX: usize = 512_000;

/// Thermal / network ESC/POS job (printer-agnostic).
///
/// ASCII-capable slips → plain text (LF only, never CR). Scripts without
/// built-in letters → Noto raster (`GS v 0`). Avoids binary-as-garbage
/// (`_____` / `-----` / cut-off lines) on heads that mishandle always-raster.
pub fn escpos_job(text: &str) -> Vec<u8> {
    escpos_job_format(&PrintFormat::kot(), text)
}

pub fn escpos_job_format(format: &PrintFormat, text: &str) -> Vec<u8> {
    let cleaned = text.replace('\r', "");
    if printer_has_letters(&cleaned) {
        // English text mode: map em → GS !; Small / custom → rasterise.
        if let Some((gs, col_div)) = escpos_gs_bang_for_em(format.em) {
            let cols = (format.line.max(8) as usize / col_div).max(8);
            escpos_job_text_cols_sized(&cleaned, cols, gs)
        } else {
            crate::receipt_raster::escpos_picture_metrics(&cleaned, &format.raster_metrics())
        }
    } else {
        crate::receipt_raster::escpos_picture_metrics(&cleaned, &format.raster_metrics())
    }
}

/// Normal (≈20 / ≈22 / ≈24) → GS ! 0; Large (≈26 / ≈28) → GS ! 0x11.
/// Other / custom → `None` (rasterise). Thermal default em 20 stays plain
/// ESC/POS for ASCII — Font A cannot go below 1×, so rasterising "Small"
/// English only bloated the job.
fn escpos_gs_bang_for_em(em: f32) -> Option<(u8, usize)> {
    if (em - 20.0).abs() < 1.5 || (em - 22.0).abs() < 1.5 || (em - 24.0).abs() < 1.5 {
        Some((0x00, 1))
    } else if (em - 26.0).abs() < 1.5 || (em - 28.0).abs() < 1.5 {
        Some((0x11, 2))
    } else {
        None
    }
}

pub fn escpos_job_mode(mode: PrinterMode, text: &str) -> Vec<u8> {
    match mode {
        PrinterMode::Standard => escpos_job_format(&PrintFormat::standard(), text),
        PrinterMode::Kot => escpos_job_format(&PrintFormat::kot(), text),
    }
}

/// Built-in-font ASCII job. Soft-wraps long lines to 32 cols (safe on 58 mm
/// and 80 mm Font A).
pub fn escpos_job_text(text: &str) -> Vec<u8> {
    escpos_job_text_cols_sized(text, 32, 0x00)
}

/// ASCII ESC/POS job with explicit `GS !` character size.
pub fn escpos_job_text_cols_sized(text: &str, cols: usize, gs_bang: u8) -> Vec<u8> {
    let cols = cols.max(8);
    let wrapped = wrap_escpos_cols(text, cols);
    let mut out = Vec::with_capacity(wrapped.len().saturating_add(48));
    out.extend_from_slice(&[0x1B, 0x40]); // ESC @: reset
    out.extend_from_slice(&[0x1B, 0x74, 0x00]); // ESC t 0 — PC437
    out.extend_from_slice(&[0x1B, 0x4D, 0x00]); // ESC M 0: Font A
    out.extend_from_slice(&[0x1D, 0x21, gs_bang]); // GS ! n: character size
    out.extend_from_slice(&[0x1B, 0x61, 0x00]); // ESC a 0: left align
    out.extend_from_slice(&[0x1B, 0x32]); // ESC 2: default line spacing
    out.extend(escpos_text(&wrapped));
    out.extend_from_slice(b"\n\n\n\n\n"); // LF only — never CR
    out.extend_from_slice(&[0x1D, 0x56, 0x00]); // GS V 0: full cut
    out
}

fn wrap_escpos_cols(text: &str, cols: usize) -> String {
    let cols = cols.max(8);
    let mut out = String::with_capacity(text.len().saturating_add(16));
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if line.chars().count() <= cols {
            out.push_str(line);
            continue;
        }
        let mut width = 0usize;
        let mut first_on_row = true;
        for word in line.split(' ') {
            let wlen = word.chars().count();
            if wlen == 0 {
                continue;
            }
            if first_on_row {
                out.push_str(word);
                width = wlen;
                first_on_row = false;
            } else if width + 1 + wlen <= cols {
                out.push(' ');
                out.push_str(word);
                width += 1 + wlen;
            } else {
                out.push('\n');
                out.push_str(word);
                width = wlen;
            }
        }
    }
    out
}

/// Same as [`escpos_job`] (raster).
pub fn escpos_job_picture(text: &str) -> Vec<u8> {
    escpos_job(text)
}

/// Every letter of `text` is one a receipt printer's built-in font has, or
/// one of the few symbols our slips use that can be spelt out.
pub fn printer_has_letters(text: &str) -> bool {
    text.chars().all(|c| c.is_ascii() || spelt_out(c).is_some())
}

/// Symbols our slips use, spelt in plain ASCII.
fn spelt_out(c: char) -> Option<&'static [u8]> {
    Some(match c {
        '₹' => b"Rs.",
        '—' | '–' | '·' | '•' => b"-",
        '×' | '✕' => b"x",
        '“' | '”' | '„' => b"\"",
        '‘' | '’' => b"'",
        '…' => b"...",
        '€' => b"EUR",
        '£' => b"GBP",
        '\u{a0}' | '\u{202f}' => b" ",
        _ => return None,
    })
}

/// Text a receipt printer's built-in font can show: printable ASCII + LF,
/// symbols spelt out, anything else as "?". **Never emit CR** (overstrike).
pub fn escpos_text(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\n' => out.push(b'\n'),
            '\r' | '\t' => {}
            c if c.is_ascii_graphic() || c == ' ' => out.push(c as u8),
            c => out.extend_from_slice(spelt_out(c).unwrap_or(b"?")),
        }
    }
    out
}

/// Print a Standard bill **image** (PNG from Noto raster) on the OS printer.
#[cfg(not(target_os = "android"))]
fn print_system_image(name: Option<&str>, path: &Path, landscape: bool) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let mut cmd = Command::new("lp");
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
        cmd.args([
            "-o",
            if landscape {
                "orientation-requested=4"
            } else {
                "orientation-requested=3"
            },
        ]);
        cmd.arg("--").arg(path);
        match cmd.output() {
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => Err(first_line(&out.stderr).unwrap_or_else(|| "the printing system refused the job".into())),
            Err(_) => {
                let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
                let _ = Command::new(opener).arg(path).spawn();
                Err("this computer has no printing system (CUPS) — the slip was opened instead".into())
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let quote = |s: &str| s.replace('\'', "''");
        let path_q = quote(&path.display().to_string());
        let land = if landscape { "$true" } else { "$false" };
        let mut script = format!(
            "Add-Type -AssemblyName System.Drawing; \
             $img = [System.Drawing.Image]::FromFile('{path_q}'); \
             $doc = New-Object System.Drawing.Printing.PrintDocument; \
             $doc.DocumentName = 'Moklanjo'; \
             $doc.DefaultPageSettings.Landscape = {land}; \
             $doc.DefaultPageSettings.Margins = New-Object System.Drawing.Printing.Margins(40,40,40,40);"
        );
        if let Some(name) = name {
            script.push_str(&format!(
                "$doc.PrinterSettings.PrinterName = '{}';",
                quote(name)
            ));
        }
        script.push_str(
            "$script:img = $img; \
             $doc.add_PrintPage({ param($s,$e) \
               $b = $e.MarginBounds; \
               $scale = [Math]::Min($b.Width / $script:img.Width, $b.Height / $script:img.Height); \
               $w = [int]($script:img.Width * $scale); \
               $h = [int]($script:img.Height * $scale); \
               $e.Graphics.DrawImage($script:img, $b.Left, $b.Top, $w, $h); \
               $e.HasMorePages = $false }); \
             $doc.Print(); \
             $img.Dispose(); \
             $doc.Dispose();",
        );
        let out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("PowerShell did not start ({e})"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(first_line(&out.stderr).unwrap_or_else(|| "Windows refused the job".into()))
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (name, path, landscape);
        Err("printing is not supported on this system".into())
    }
}

fn print_system_raw(name: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > ESC_POS_JOB_MAX {
        return Err(format!(
            "print job too large ({} bytes) — refused so the printer is not flooded",
            bytes.len()
        ));
    }
    let dir = std::env::temp_dir().join("kumkaj-thermal-raw");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let result = print_system_raw_path(name, &path);
    let _ = fs::remove_file(&path);
    result
}

fn print_system_raw_path(name: Option<&str>, path: &Path) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let mut cmd = Command::new("lp");
        cmd.args(["-o", "raw"]);
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
        cmd.arg("--").arg(path);
        match cmd.output() {
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => {
                let detail = String::from_utf8_lossy(&out.stderr);
                let line = detail.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("raw print refused");
                Err(format!("{line} — prefer host:9100 / bt: / usb: for thermal"))
            }
            Err(_) => Err("no printing system (CUPS) — use host:9100 / bt: / usb:".into()),
        }
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let Some(name) = name else {
            return Err(
                "pick the thermal printer by name, or use host:9100 / bt: / usb: in Settings → Printers"
                    .into(),
            );
        };
        let quote = |s: &str| s.replace('\'', "''");
        let path_s = quote(&path.display().to_string());
        let name_s = quote(name);
        let script = format!(
            r#"
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class KumkajRawPrint {{
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)]
  public class DOCINFOA {{
    [MarshalAs(UnmanagedType.LPStr)] public string pDocName;
    [MarshalAs(UnmanagedType.LPStr)] public string pOutputFile;
    [MarshalAs(UnmanagedType.LPStr)] public string pDataType;
  }}
  [DllImport("winspool.Drv", EntryPoint = "OpenPrinterA", SetLastError = true, CharSet = CharSet.Ansi)]
  public static extern bool OpenPrinter([MarshalAs(UnmanagedType.LPStr)] string szPrinter, out IntPtr hPrinter, IntPtr pd);
  [DllImport("winspool.Drv", EntryPoint = "ClosePrinter", SetLastError = true)]
  public static extern bool ClosePrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "StartDocPrinterA", SetLastError = true, CharSet = CharSet.Ansi)]
  public static extern bool StartDocPrinter(IntPtr hPrinter, int level, [In] DOCINFOA di);
  [DllImport("winspool.Drv", EntryPoint = "EndDocPrinter", SetLastError = true)]
  public static extern bool EndDocPrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "StartPagePrinter", SetLastError = true)]
  public static extern bool StartPagePrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "EndPagePrinter", SetLastError = true)]
  public static extern bool EndPagePrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "WritePrinter", SetLastError = true)]
  public static extern bool WritePrinter(IntPtr hPrinter, IntPtr pBytes, int dwCount, out int dwWritten);
}}
"@
$bytes = [IO.File]::ReadAllBytes('{path_s}')
$h = [IntPtr]::Zero
if (-not [KumkajRawPrint]::OpenPrinter('{name_s}', [ref]$h, [IntPtr]::Zero)) {{ throw "OpenPrinter failed" }}
try {{
  $di = New-Object KumkajRawPrint+DOCINFOA
  $di.pDocName = 'THERMAL'
  $di.pDataType = 'RAW'
  if (-not [KumkajRawPrint]::StartDocPrinter($h, 1, $di)) {{ throw "StartDocPrinter failed" }}
  try {{
    if (-not [KumkajRawPrint]::StartPagePrinter($h)) {{ throw "StartPagePrinter failed" }}
    try {{
      $pinned = [Runtime.InteropServices.GCHandle]::Alloc($bytes, [Runtime.InteropServices.GCHandleType]::Pinned)
      try {{
        $offset = 0
        while ($offset -lt $bytes.Length) {{
          $written = 0
          $ptr = [IntPtr]::Add($pinned.AddrOfPinnedObject(), $offset)
          $chunk = $bytes.Length - $offset
          if (-not [KumkajRawPrint]::WritePrinter($h, $ptr, $chunk, [ref]$written)) {{ throw "WritePrinter failed" }}
          if ($written -le 0) {{ throw "WritePrinter wrote nothing" }}
          $offset += $written
        }}
      }} finally {{ $pinned.Free() }}
    }} finally {{ [void][KumkajRawPrint]::EndPagePrinter($h) }}
  }} finally {{ [void][KumkajRawPrint]::EndDocPrinter($h) }}
}} finally {{ [void][KumkajRawPrint]::ClosePrinter($h) }}
"#,
            path_s = path_s,
            name_s = name_s,
        );
        let out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("PowerShell did not start ({e})"))?;
        if out.status.success() {
            Ok(())
        } else {
            let detail = String::from_utf8_lossy(&out.stderr);
            let line = detail.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("Windows raw print failed");
            Err(format!("{line} — prefer host:9100 / bt: / usb: for thermal"))
        }
    }
    #[cfg(target_os = "android")]
    {
        let _ = (name, path);
        Err("Android thermal needs Network, Bluetooth, or USB".into())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
    {
        let _ = (name, path);
        Err("printing is not supported on this system".into())
    }
}

#[cfg(not(target_os = "android"))]
fn first_line(bytes: &[u8]) -> Option<String> {
    String::from_utf8_lossy(bytes).lines().map(str::trim).find(|l| !l.is_empty()).map(str::to_string)
}

static FOUND: Mutex<Option<Vec<String>>> = Mutex::new(None);
static LOOKING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static FIND_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static FIND_WAS_CANCELLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Printers last found on this device (system names + network addresses on
/// port [`RAW_PRINT_PORT`]). Empty until a look has finished.
pub fn found_printers() -> Vec<String> {
    FOUND.lock().unwrap().clone().unwrap_or_default()
}

/// `true` while a background look is still running.
pub fn is_finding_printers() -> bool {
    LOOKING.load(std::sync::atomic::Ordering::SeqCst)
}

/// Ask the background Find worker to stop early (partial results still apply).
pub fn cancel_find_printers() {
    FIND_CANCEL.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Whether the last Find finished because Cancel was tapped.
pub fn find_printers_was_cancelled() -> bool {
    FIND_WAS_CANCELLED.load(std::sync::atomic::Ordering::SeqCst)
}

fn find_cancelled() -> bool {
    FIND_CANCEL.load(std::sync::atomic::Ordering::SeqCst)
}

/// Whether at least one look has finished (`found_printers` is meaningful).
pub fn printers_find_finished() -> bool {
    FOUND.lock().unwrap().is_some()
}

/// Look for printers on a worker thread (system list + this Wi‑Fi's raw
/// port-[`RAW_PRINT_PORT`] listeners). Works on Linux, macOS, Windows, and
/// Android (network scan only there). [`found_printers`] updates afterwards
/// and the screen is woken.
///
/// Only call from an explicit Find tap — never on Settings open / startup.
/// A /24 port scan saturates the NIC and makes the UI feel frozen even when
/// this runs off the UI thread.
pub fn find_printers_in_background() {
    if LOOKING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    FIND_CANCEL.store(false, std::sync::atomic::Ordering::SeqCst);
    FIND_WAS_CANCELLED.store(false, std::sync::atomic::Ordering::SeqCst);
    if std::thread::Builder::new()
        .name("find-printers".into())
        .spawn(|| {
            let mut found = system_printers();
            if !find_cancelled() {
                for addr in network_raw_printers() {
                    if !found.iter().any(|n| n == &addr) {
                        found.push(addr);
                    }
                }
            }
            if !find_cancelled() {
                for bt in crate::print_bt::list_printers() {
                    if !found.iter().any(|n| n == &bt) {
                        found.push(bt);
                    }
                }
            }
            if !find_cancelled() {
                for usb in crate::print_usb::list_printers() {
                    if !found.iter().any(|n| n == &usb) {
                        found.push(usb);
                    }
                }
            }
            #[cfg(target_os = "android")]
            if !find_cancelled() {
                for a in crate::print_android::list_print_services() {
                    if !found.iter().any(|n| n == &a) {
                        found.push(a);
                    }
                }
            }
            let cancelled = find_cancelled();
            FIND_WAS_CANCELLED.store(cancelled, std::sync::atomic::Ordering::SeqCst);
            found.sort();
            *FOUND.lock().unwrap() = Some(found);
            LOOKING.store(false, std::sync::atomic::Ordering::SeqCst);
            if let Some(hook) = *NOTIFY.lock().unwrap() {
                hook();
            }
        })
        .is_err()
    {
        LOOKING.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// System queues that still accept jobs (empty on Android — no drivers).
///
/// Prefer accepting/enabled queues. `lpstat -e` also lists CUPS-discovered
/// ghosts (removed USB, stale network names) that are not real destinations.
fn system_printers() -> Vec<String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let out = |args: &[&str]| -> String {
            Command::new("lpstat")
                .args(args)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                .unwrap_or_default()
        };
        let accepting: Vec<String> = out(&["-a"])
            .lines()
            .filter_map(|l| l.split_whitespace().next().map(str::to_string))
            .filter(|n| !n.is_empty())
            .collect();
        let disabled: std::collections::HashSet<String> = out(&["-p"])
            .lines()
            .filter(|l| l.contains(" disabled "))
            .filter_map(|l| {
                l.strip_prefix("printer ")
                    .and_then(|rest| rest.split_whitespace().next())
                    .map(str::to_string)
            })
            .collect();
        accepting
            .into_iter()
            .filter(|n| !disabled.contains(n))
            .collect()
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-Printer | Where-Object { -not $_.WorkOffline -and $_.PrinterStatus -ne 'Offline' } | ForEach-Object { $_.Name }",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// Hosts on this /24 that accept TCP on the raw ESC/POS port — the usual
/// way to find network receipt printers from phones and desktops alike.
///
/// Short per-host timeout and small chunks (many workers) so Find finishes
/// in ~0.5s instead of multi-second NIC saturation.
fn network_raw_printers() -> Vec<String> {
    use std::net::{SocketAddr, TcpStream, UdpSocket};
    use std::sync::Arc;
    let Some(ip) = (|| {
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("10.255.255.255:1").ok()?;
        match socket.local_addr().ok()?.ip() {
            std::net::IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_unspecified() => Some(v4),
            _ => None,
        }
    })() else {
        return Vec::new();
    };
    let o = ip.octets();
    let addrs: Vec<SocketAddr> = (1u8..=254)
        .filter(|h| *h != o[3])
        .map(|h| SocketAddr::from(([o[0], o[1], o[2], h], RAW_PRINT_PORT)))
        .collect();
    let found = Arc::new(Mutex::new(Vec::new()));
    let workers: Vec<_> = addrs
        .chunks(4)
        .map(|chunk| {
            let chunk = chunk.to_vec();
            let found = Arc::clone(&found);
            std::thread::spawn(move || {
                for addr in chunk {
                    if find_cancelled() {
                        break;
                    }
                    if TcpStream::connect_timeout(&addr, Duration::from_millis(80)).is_ok() {
                        found.lock().unwrap().push(addr.ip().to_string());
                    }
                }
            })
        })
        .collect();
    for w in workers {
        let _ = w.join();
    }
    let mut out = found.lock().unwrap().clone();
    out.sort();
    out.dedup();
    out
}

/// Desktop clipboard. Android goes through `Cx::copy_to_clipboard`, which is
/// wired to `ClipboardManager` in Makepad's activity, so callers that hold a
/// `Cx` should prefer that on every platform.
pub fn copy_text(text: &str) -> bool {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        if let Ok(mut clip) = arboard::Clipboard::new() {
            return clip.set_text(text.to_string()).is_ok();
        }
    }
    let _ = text;
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `configure` / Capture are process-global — serialise tests that touch them.
    static CFG_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn unit_tests_never_reach_the_os_printer() {
        let _g = CFG_LOCK.lock().unwrap();
        assert_eq!(sink(), PrintSink::Capture);
        let mut cfg = DevicePrinters::default();
        let id = cfg.add_printer("Desk", "Canon_G3010");
        *cfg.route_mut(RouteKey::Receipt) = vec![id];
        configure(cfg);
        let _ = take_captured();
        let dir = std::env::temp_dir().join(format!("courier-print-{}", std::process::id()));
        let share = print_text(PrintKind::Receipt, "Booking receipt — test", Some(&dir)).unwrap();
        assert!(!share.opened, "the OS was not handed the job");
        let jobs = take_captured();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].text, "Booking receipt — test");
        assert_eq!(jobs[0].kind, PrintKind::Receipt);
        assert!(take_captured().is_empty(), "taken once");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// What the owner types picks the kind of printer.
    #[test]
    fn typed_printers_read_as_meant() {
        assert_eq!(parse_target(""), PrinterTarget::Default);
        assert_eq!(parse_target("  "), PrinterTarget::Default);
        assert_eq!(parse_target("192.168.1.50"), PrinterTarget::Network("192.168.1.50:9100".into()));
        assert_eq!(parse_target("192.168.1.50:9101"), PrinterTarget::Network("192.168.1.50:9101".into()));
        assert_eq!(parse_target("EPSON TM-T82"), PrinterTarget::Named("EPSON TM-T82".into()));
        assert!(can_print(&PrinterTarget::Network("10.0.0.9:9100".into())));
    }

    /// A receipt printer's own font has no ₹ or dashes: they are spelt out,
    /// and anything it cannot draw is a ? — never bytes that garble the job.
    #[test]
    fn receipt_printer_text_is_plain() {
        assert_eq!(escpos_text("1 kg × 2 — ₹500\r\n"), b"1 kg x 2 - Rs.500\n".to_vec());
        assert_eq!(escpos_text("राजकोट"), b"??????".to_vec());
        assert!(printer_has_letters("1 kg × 2 — ₹500"));
        assert!(!printer_has_letters("राजकोट"));
        let text_hi = escpos_job_text("To: राजकोट");
        assert!(!text_hi.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]));
        assert!(text_hi.windows(6).any(|w| w == b"??????"));
        let ascii = escpos_job("AWB 7\nLine two\nLine three");
        assert_eq!(&ascii[..2], &[0x1B, 0x40]);
        assert!(
            !ascii.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
            "ASCII slips use plain ESC/POS text"
        );
        assert!(ascii.windows(3).any(|w| w == b"AWB"));
        assert!(ascii.ends_with(&[0x1D, 0x56, 0x00]));
        let indic = escpos_job("राजकोट");
        assert!(
            indic.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
            "scripts without built-in letters still use raster"
        );
        assert!(!escpos_text("A\nB").contains(&b'\r'));
        assert_eq!(escpos_text("A\nB"), b"A\nB".to_vec());
    }

    /// A network printer gets the whole job over one connection.
    #[test]
    fn network_printer_gets_the_job() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let got = std::thread::spawn(move || {
            let (mut conn, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut conn, &mut buf).unwrap();
            buf
        });
        print_network(&addr, "Receipt INV-MAIN-000012\n1 kg parcel").unwrap();
        assert_eq!(got.join().unwrap(), escpos_job("Receipt INV-MAIN-000012\n1 kg parcel"));
        // Nobody listening: said in words, not a hang.
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().to_string();
        assert!(print_network(&closed, "x").unwrap_err().contains("cannot reach"));
    }

    /// Printers belong to the install: saved beside the language choice and
    /// read back on the next start.
    #[test]
    fn device_printers_are_kept_per_install() {
        let _g = CFG_LOCK.lock().unwrap();
        let base = std::env::temp_dir().join(format!("courier-printers-{}", uuid::Uuid::new_v4().simple()));
        assert_eq!(load_device_printers(&base), DevicePrinters::default());
        let mut chosen = DevicePrinters::default();
        let desk = chosen.add_printer("Desk", "192.168.1.50");
        chosen.routes.receipt = vec![desk.clone()];
        chosen.routes.label = vec![desk];
        save_device_printers(&base, &chosen).unwrap();
        assert_eq!(load_device_printers(&base), chosen);
        assert_eq!(
            printers().targets_for(RouteKey::Receipt),
            vec![PrinterTarget::Network("192.168.1.50:9100".into())]
        );
        configure(DevicePrinters::default());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn legacy_receipt_migrates_and_empty_label_skips() {
        let _g = CFG_LOCK.lock().unwrap();
        let migrated = migrate_legacy("Canon_G3010");
        assert_eq!(
            migrated.targets_for(RouteKey::Receipt),
            vec![PrinterTarget::Named("Canon_G3010".into())]
        );
        assert!(migrated.route_for(RouteKey::Label).is_empty());
        let blank = migrate_legacy("");
        assert!(blank.printers.is_empty());
        assert!(blank.route_for(RouteKey::Receipt).is_empty());
        // Scrub old synthetic Default from a new-format file.
        let base = std::env::temp_dir().join(format!("courier-scrub-{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("printers.json"),
            r#"{"printers":[{"id":"default","label":"Default","target":""}],"routes":{"receipt":["default"],"label":[]}}"#,
        )
        .unwrap();
        let scrubbed = load_device_printers(&base);
        assert!(scrubbed.printers.is_empty());
        assert!(scrubbed.route_for(RouteKey::Receipt).is_empty());
        let _ = fs::remove_dir_all(base);
        configure(DevicePrinters::default());
        let _ = take_captured();
        let dir = std::env::temp_dir().join(format!("courier-label-skip-{}", uuid::Uuid::new_v4().simple()));
        let share = print_route(PrintKind::Label, RouteKey::Label, "AWB label", Some(&dir)).unwrap();
        assert!(!share.opened);
        assert!(take_captured().is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}

//! Printing (local free POS — no cloud), the same on every device that can
//! hold the books or take orders.
//!
//! Owner nouns (Settings):
//! - **Printer** — one OS device Find already knows (destination only).
//! - **Receipt format** and **label format** — how wide the job is drawn,
//!   portrait or landscape, and how the bytes are sent (OS document, raw,
//!   or ESC/POS). A receipt and an AWB label do not share a width: a
//!   receipt is a docket, a label is a sticker with a barcode.
//! - **Output** — under Receipt or AWB label: ordered printers. Tries the
//!   first that works. Empty = don't print.
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

/// How a job is handed to the device. Three ways cover a receipt and a label.
pub const DELIVERY_SYSTEM_SPOOLER: &str = "system_spooler";
pub const DELIVERY_RAW_DEVICE: &str = "raw_device";
pub const DELIVERY_ESCPOS: &str = "escpos";

/// Character columns for a content width in millimetres.
pub fn wrap_cols_for_paper_mm(mm: u32) -> u32 {
    let mm = mm.clamp(40, 210);
    let cols = ((mm as f32) * 0.55).round() as u32;
    cols.clamp(16, 120)
}

/// Dots across the content. 58 mm is 384 dots; 80 mm is 576. Wider scales from 80 mm.
pub fn paper_width_dots(mm: u32) -> usize {
    let mm = mm.clamp(40, 210);
    let dots = if mm <= 58 {
        384
    } else {
        (mm as usize * 576) / 80
    };
    dots.max(8) & !7
}

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
    /// 0 = legacy (receipt 80 mm, label 100 mm).
    #[serde(default)]
    pub content_width_mm: u32,
    /// Empty = OS document when `document` is set, otherwise ESC/POS.
    #[serde(default)]
    pub delivery: String,
}

fn default_true() -> bool { true }
fn default_em() -> f32 { STD_EM_NORMAL }
fn default_line() -> u32 { 40 }
fn default_baseline() -> f32 { STD_EM_NORMAL + 2.0 }
fn default_side() -> f32 { STD_SIDE }

/// Document type-size presets. Product built-ins are generic (any OS queue).
/// Narrow-tray tuning is owner/template choice — not assumed for every shop.
/// OS jobs still fit-to-page so PNGs are not printed ~16″ wide at 72 DPI.
pub const STD_EM_SMALL: f32 = 18.0;
pub const STD_EM_NORMAL: f32 = 20.0;
pub const STD_EM_LARGE: f32 = 26.0;
/// Generic side inset (dots on the 384-wide raster) for Standard formats.
pub const STD_SIDE: f32 = 16.0;
/// Wider inset for optional tight-tray templates only.
pub const STD_SIDE_NARROW_TRAY: f32 = 40.0;

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
            line: wrap_cols_for_paper_mm(80),
            baseline: STD_EM_NORMAL + 2.0,
            side: STD_SIDE,
            content_width_mm: 80,
            delivery: DELIVERY_SYSTEM_SPOOLER.into(),
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
            line: wrap_cols_for_paper_mm(80),
            baseline: STD_EM_NORMAL + 2.0,
            side: STD_SIDE,
            content_width_mm: 80,
            delivery: DELIVERY_SYSTEM_SPOOLER.into(),
        }
    }
    /// Non-deletable built-in. AWB label (id stays `kot` so old files load).
    pub fn kot() -> Self {
        Self {
            id: FORMAT_KOT.into(),
            label: "AWB label".into(),
            document: false,
            landscape: false,
            em: 20.0,
            line: wrap_cols_for_paper_mm(100),
            baseline: 22.0,
            side: 4.0,
            content_width_mm: 100,
            delivery: DELIVERY_ESCPOS.into(),
        }
    }
    pub fn delivery_id(&self) -> &str {
        let d = self.delivery.trim();
        if d.is_empty() {
            if self.document {
                DELIVERY_SYSTEM_SPOOLER
            } else {
                DELIVERY_ESCPOS
            }
        } else {
            d
        }
    }
    pub fn shown_content_width_mm(&self) -> u32 {
        if self.content_width_mm > 0 {
            self.content_width_mm.clamp(40, 210)
        } else if self.id == FORMAT_KOT {
            100
        } else if self.line <= 34 {
            58
        } else {
            80
        }
    }
    pub fn raster_metrics(&self) -> crate::receipt_raster::RasterMetrics {
        crate::receipt_raster::RasterMetrics {
            em: self.em,
            line: self.line as usize,
            baseline: self.baseline,
            side: self.side,
            width_dots: paper_width_dots(self.shown_content_width_mm()),
        }
    }
    pub fn is_label_format(&self) -> bool {
        self.id == FORMAT_KOT
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
        for f in list.iter_mut() {
            if is_standard_document(&f.id) && f.side <= 12.0 {
                f.side = STD_SIDE;
            }
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

    /// Portrait/landscape and delivery for the receipt, or for the AWB label.
    /// A receipt stays on the standard format. A label stays on the label format.
    pub fn set_job_drive(&mut self, label: bool, landscape: bool, delivery: &str) {
        let delivery = if delivery.trim().is_empty() {
            DELIVERY_SYSTEM_SPOOLER
        } else {
            delivery.trim()
        };
        let document = delivery == DELIVERY_SYSTEM_SPOOLER;
        for f in self.formats.iter_mut() {
            let hit = if label {
                f.id == FORMAT_KOT
            } else {
                is_standard_document(&f.id)
            };
            if hit {
                f.landscape = landscape;
                f.delivery = delivery.to_string();
                f.document = document;
            }
        }
        if label {
            self.routes.label_format = FORMAT_KOT.into();
        } else {
            self.routes.receipt_format = FORMAT_STANDARD_PORTRAIT.into();
        }
    }

    /// Content width for the receipt (both orientations) or the label. Not shared.
    pub fn set_content_width(&mut self, label: bool, mm: u32) {
        let mm = mm.clamp(40, 210);
        let cols = wrap_cols_for_paper_mm(mm);
        for f in self.formats.iter_mut() {
            let hit = if label {
                f.id == FORMAT_KOT
            } else {
                is_standard_document(&f.id)
            };
            if hit {
                f.content_width_mm = mm;
                f.line = cols;
            }
        }
        if label {
            self.routes.label_format = FORMAT_KOT.into();
        } else {
            self.routes.receipt_format = FORMAT_STANDARD_PORTRAIT.into();
        }
    }

    pub fn set_type_size(&mut self, label: bool, em: f32) {
        let em = em.clamp(12.0, 48.0);
        let baseline = em + 2.0;
        for f in self.formats.iter_mut() {
            let hit = if label {
                f.id == FORMAT_KOT
            } else {
                is_standard_document(&f.id)
            };
            if hit {
                f.em = em;
                f.baseline = baseline;
            }
        }
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
            content_width_mm: base.content_width_mm,
            delivery: base.delivery.clone(),
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
    let bytes = job_bytes(format, text);
    match target {
        PrinterTarget::Network(addr) => print_network_bytes(addr, &bytes),
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
    let png = slip_png(format, text)?;
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
    let png = slip_png(format, text)?;
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
    print_network_bytes(addr, &escpos_job_mode(mode, text))
}

/// Network `:9100`: connect (waiting while the printer finishes the last
/// slip), send one raster picture at a time, then confirm it took the job.
fn print_network_bytes(addr: &str, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > ESC_POS_JOB_MAX {
        return Err(format!(
            "print job too large ({} bytes) — refused so the printer is not flooded",
            bytes.len()
        ));
    }
    let sock: std::net::SocketAddr = addr
        .parse()
        .map_err(|_| format!("{addr} is not a printer address"))?;
    let mut stream = connect_printer(&sock)?;
    // A head busy with a long slip reads slowly; TCP holds our writes until
    // it does. That is waiting, not a failure.
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    // One picture at a time: a single burst printed the first band (about
    // three lines) and the head dropped the rest.
    crate::receipt_raster::write_escpos_paced(bytes, |piece| {
        stream.write_all(piece).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())
    })?;
    finish_printer_stream(addr, stream, bytes.len())
}

/// Open the printer's raw port. A printer still printing the previous slip
/// often refuses (or resets) a second connection until it is done: keep
/// trying for a while. A printer that never answers fails after two tries.
fn connect_printer(sock: &std::net::SocketAddr) -> Result<std::net::TcpStream, String> {
    use std::io::ErrorKind;
    let busy_until = std::time::Instant::now() + Duration::from_secs(15);
    let mut silent_tries = 0;
    loop {
        match std::net::TcpStream::connect_timeout(sock, Duration::from_secs(4)) {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                let busy = matches!(
                    e.kind(),
                    ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted
                );
                if !busy {
                    silent_tries += 1;
                }
                if (busy && std::time::Instant::now() < busy_until) || (!busy && silent_tries < 2) {
                    std::thread::sleep(Duration::from_millis(400));
                    continue;
                }
                return Err(format!("cannot reach it ({e}) — is it on and on this network?"));
            }
        }
    }
}

/// ESC/POS real-time status (DLE EOT 1) after the last byte: the answer
/// comes once the printer has read everything before it; "offline" (cover
/// open, out of paper, error) is a failure the desk must hear about. A
/// printer that does not answer is remembered, and later jobs half-close
/// and give it a moment to read before the next slip connects.
fn finish_printer_stream(addr: &str, mut stream: std::net::TcpStream, sent: usize) -> Result<(), String> {
    use std::io::Read;
    static ANSWERS: Mutex<Option<std::collections::HashMap<String, bool>>> = Mutex::new(None);
    let known = ANSWERS
        .lock()
        .unwrap()
        .get_or_insert_with(Default::default)
        .get(addr)
        .copied();
    if known != Some(false) {
        let wait = Duration::from_millis((1500 + sent as u64 / 4).min(12_000));
        let _ = stream.set_read_timeout(Some(wait));
        if stream.write_all(&[0x10, 0x04, 0x01]).and_then(|_| stream.flush()).is_ok() {
            let mut reply = [0u8; 1];
            let answered = matches!(stream.read(&mut reply), Ok(1));
            ANSWERS
                .lock()
                .unwrap()
                .get_or_insert_with(Default::default)
                .insert(addr.to_string(), answered);
            if answered {
                if let Some(problem) = escpos_status_problem(reply[0]) {
                    return Err(problem.into());
                }
                return Ok(());
            }
        }
    }
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let _ = stream.set_read_timeout(Some(Duration::from_millis(600)));
    let mut sink = [0u8; 64];
    while let Ok(n) = stream.read(&mut sink) {
        if n == 0 {
            break;
        }
    }
    Ok(())
}

/// A DLE EOT 1 reply that says the printer cannot print. The fixed bits
/// must match, so a stray byte is never read as a fault.
fn escpos_status_problem(reply: u8) -> Option<&'static str> {
    if reply & 0x93 != 0x12 {
        return None;
    }
    if reply & 0x08 != 0 {
        return Some("the printer says it is offline — check paper, the cover, and its light");
    }
    None
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

/// AWB sitting under an `AWB LABEL` heading, else empty.
pub fn awb_from_slip(text: &str) -> String {
    let mut prev = false;
    for line in text.lines() {
        let t = line.trim();
        if prev && !t.is_empty() && !t.starts_with('-') {
            return t.to_string();
        }
        prev = t.eq_ignore_ascii_case("AWB LABEL");
    }
    String::new()
}

fn slip_png(format: &PrintFormat, text: &str) -> Result<Vec<u8>, String> {
    let metrics = format.raster_metrics();
    if format.is_label_format() {
        crate::receipt_raster::png_label(text, &awb_from_slip(text), &metrics)
    } else {
        crate::receipt_raster::png_metrics(text, &metrics)
    }
}

fn job_bytes(format: &PrintFormat, text: &str) -> Vec<u8> {
    if format.is_label_format() {
        let cleaned = text.replace('\r', "");
        return crate::receipt_raster::escpos_label(
            &cleaned,
            &awb_from_slip(&cleaned),
            &format.raster_metrics(),
        );
    }
    escpos_job_format(format, text)
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
        // Without fit, CUPS often treats the PNG as 72 DPI (≈16″) and clips
        // both sides on A4/Letter / Canon-class trays.
        cmd.args(["-o", "fit-to-page"]);
        cmd.args(["-o", "print-scaling=fit"]);
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
            Ok(out) if out.status.success() => cups_confirm_after_lp(name, &out.stdout),
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
             $doc.DefaultPageSettings.Margins = New-Object System.Drawing.Printing.Margins(75,75,75,75);"
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
    // A CUPS queue that is a raw TCP socket: talk to that printer directly,
    // paced and confirmed like a network address (`lp` hands the socket
    // backend the whole slip in one burst).
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    if let Some(addr) = name.and_then(cups_socket_address) {
        return print_network_bytes(&addr, bytes);
    }
    let dir = std::env::temp_dir().join("kumkaj-thermal-raw");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let pieces = crate::receipt_raster::escpos_piece_lengths(bytes);
    let result = print_system_raw_path(name, &path, &pieces);
    let _ = fs::remove_file(&path);
    result
}

/// `lp` stdout looks like `request id is QueueName-123 (1 file(s))`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn parse_lp_request_id(stdout: &str) -> Option<String> {
    let rest = stdout.split("request id is ").nth(1)?;
    let id = rest.split_whitespace().next()?.trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

/// CUPS often accepts a job (`lp` exit 0) while the backend cannot reach the
/// device — classic message: "Waiting for printer to become available".
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_printer_unreachable(name: &str) -> Option<String> {
    let out = Command::new("lpstat").args(["-p", name]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let lower = text.to_ascii_lowercase();
    if !(lower.contains("waiting for printer to become available")
        || lower.contains("printer not connected")
        || lower.contains("unable to locate printer"))
    {
        return None;
    }
    let detail = text
        .lines()
        .map(str::trim)
        .find(|l| {
            let l = l.to_ascii_lowercase();
            l.contains("waiting") || l.contains("unable") || l.contains("not connected")
        })
        .unwrap_or("printer not reachable");
    Some(format!(
        "Printer \"{name}\" did not take the job ({detail}) — check cable/power, or use a Network/USB address in Settings → Printers"
    ))
}

/// After `lp` accepts the job: log it, and fail if CUPS already says the
/// printer is unreachable (otherwise the UI shows success with no paper).
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_confirm_after_lp(name: Option<&str>, lp_stdout: &[u8]) -> Result<(), String> {
    let stdout = String::from_utf8_lossy(lp_stdout);
    let job = parse_lp_request_id(&stdout);
    let who = name.unwrap_or("(default)");
    let job_s = job.as_deref().unwrap_or("?");
    eprintln!("[print] lp queued printer={who} job={job_s}");
    crate::diag::info(
        crate::diag::Cat::Print,
        "lp_queued",
        format!("printer={who} job={job_s}"),
    );
    let printer = match name {
        Some(n) => n.to_string(),
        None => match cups_default_printer() {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(blocked) = cups_printer_unreachable(&printer) {
            if let Some(job) = &job {
                let _ = Command::new("cancel").arg(job).output();
            }
            eprintln!("[print] CUPS unreachable: {blocked}");
            crate::diag::warn(crate::diag::Cat::Print, "cups_unreachable", &blocked);
            return Err(blocked);
        }
        let pending = job.as_deref().is_some_and(cups_job_pending);
        if !pending || std::time::Instant::now() >= deadline {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(300));
    }
}

/// The queue `lp` uses when no printer is named (`lpstat -d`).
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_default_printer() -> Option<String> {
    let out = Command::new("lpstat").arg("-d").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let name = text.rsplit(':').next()?.trim();
    (!name.is_empty() && !text.to_ascii_lowercase().contains("no system default"))
        .then(|| name.to_string())
}

/// Whether CUPS still lists `job` as queued or printing (`lpstat -o`).
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_job_pending(job: &str) -> bool {
    let Ok(out) = Command::new("lpstat").arg("-o").output() else {
        return false;
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|l| l.split_whitespace().next() == Some(job))
}

/// `host:port` of a CUPS queue whose device is a raw socket.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_socket_address(name: &str) -> Option<String> {
    use std::net::ToSocketAddrs;
    let out = Command::new("lpstat").args(["-v", name]).output().ok()?;
    let host_port = parse_cups_socket_uri(&String::from_utf8_lossy(&out.stdout))?;
    let addr = host_port.to_socket_addrs().ok()?.next()?;
    Some(addr.to_string())
}

/// The `host:port` in an `lpstat -v` line for a `socket://` device (port
/// 9100 when none is named). Any other backend: `None`.
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn parse_cups_socket_uri(lpstat: &str) -> Option<String> {
    let rest = lpstat.split("socket://").nth(1)?;
    let hostport: &str = rest
        .split(|c: char| c.is_whitespace() || c == '/' || c == '?')
        .next()?;
    if hostport.is_empty() {
        return None;
    }
    let has_port = if hostport.starts_with('[') {
        hostport.contains("]:")
    } else {
        hostport.contains(':')
    };
    Some(if has_port {
        hostport.to_string()
    } else {
        format!("{hostport}:{RAW_PRINT_PORT}")
    })
}

/// Windows script: write the bytes in `$pieces` order, one paced piece at a
/// time, each looped until WritePrinter took all of it.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn powershell_paced_write_loop(class: &str, pieces: &[usize]) -> String {
    let list = if pieces.is_empty() {
        "0".to_string()
    } else {
        pieces.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",")
    };
    let pause = crate::receipt_raster::PIECE_PAUSE_MS;
    format!(
        r#"$pieces = @({list})
        $offset = 0
        $first = $true
        foreach ($len in $pieces) {{
          if ($len -le 0) {{ continue }}
          if (-not $first) {{ Start-Sleep -Milliseconds {pause} }}
          $first = $false
          $end = $offset + $len
          while ($offset -lt $end) {{
            $written = 0
            $ptr = [IntPtr]::Add($pinned.AddrOfPinnedObject(), $offset)
            $chunk = $end - $offset
            if (-not [{class}]::WritePrinter($h, $ptr, $chunk, [ref]$written)) {{ throw "WritePrinter failed" }}
            if ($written -le 0) {{ throw "WritePrinter wrote nothing" }}
            $offset += $written
          }}
        }}
        if ($offset -ne $bytes.Length) {{ throw "WritePrinter sent $offset of $($bytes.Length) bytes" }}"#
    )
}

fn print_system_raw_path(name: Option<&str>, path: &Path, pieces: &[usize]) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // Other backends (USB, serial, IPP) hold the host back while the
        // head is busy; socket queues never get here.
        let _ = pieces;
        let mut cmd = Command::new("lp");
        cmd.args(["-o", "raw"]);
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
        cmd.arg("--").arg(path);
        match cmd.output() {
            Ok(out) if out.status.success() => cups_confirm_after_lp(name, &out.stdout),
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
        # One picture at a time with a pause (a head drops a burst).
        {write_loop}
      }} finally {{ $pinned.Free() }}
    }} finally {{ [void][KumkajRawPrint]::EndPagePrinter($h) }}
  }} finally {{ [void][KumkajRawPrint]::EndDocPrinter($h) }}
}} finally {{ [void][KumkajRawPrint]::ClosePrinter($h) }}
"#,
            path_s = path_s,
            name_s = name_s,
            write_loop = powershell_paced_write_loop("KumkajRawPrint", pieces),
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
        let _ = (name, path, pieces);
        Err("Android thermal needs Network, Bluetooth, or USB".into())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
    {
        let _ = (name, path, pieces);
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
mod pacing_tests {
    use super::*;

    #[test]
    fn a_cups_socket_queue_names_its_printer() {
        assert_eq!(
            parse_cups_socket_uri("device for Label: socket://192.168.1.60:9100").as_deref(),
            Some("192.168.1.60:9100")
        );
        assert_eq!(
            parse_cups_socket_uri("device for L: socket://label.local").as_deref(),
            Some("label.local:9100")
        );
        assert_eq!(parse_cups_socket_uri("device for L: usb://Zebra/ZD220"), None);
    }

    #[test]
    fn printer_status_offline_is_a_failure() {
        assert_eq!(escpos_status_problem(0x12), None);
        assert!(escpos_status_problem(0x1A).is_some());
        assert_eq!(escpos_status_problem(0xFF), None);
    }

    #[test]
    fn windows_raw_writer_sends_each_piece() {
        let script = powershell_paced_write_loop("KumkajRawPrint", &[2058, 3016, 8]);
        assert!(script.contains("$pieces = @(2058,3016,8)"));
        assert!(script.contains("Start-Sleep -Milliseconds 40"));
        assert!(script.contains("$offset -ne $bytes.Length"));
    }

    /// The whole slip, then only the status request; an "offline" answer
    /// is a failure.
    #[test]
    fn network_printer_gets_the_job_then_the_status_request() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let got = std::thread::spawn(move || {
            let (mut conn, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut conn, &mut buf).unwrap();
            buf
        });
        print_network(&addr, "AWB 123\nDelhi").unwrap();
        let mut want = escpos_job_mode(PrinterMode::Kot, "AWB 123\nDelhi");
        want.extend_from_slice(&[0x10, 0x04, 0x01]);
        assert_eq!(got.join().unwrap(), want);

        let offline = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = offline.local_addr().unwrap().to_string();
        std::thread::spawn(move || {
            let (mut conn, _) = offline.accept().unwrap();
            let mut got = Vec::new();
            let mut buf = [0u8; 4096];
            while let Ok(n) = std::io::Read::read(&mut conn, &mut buf) {
                if n == 0 {
                    break;
                }
                got.extend_from_slice(&buf[..n]);
                if got.ends_with(&[0x10, 0x04, 0x01]) {
                    let _ = conn.write_all(&[0x1A]);
                    break;
                }
            }
        });
        assert!(print_network(&addr, "AWB 124").unwrap_err().contains("offline"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `configure` / Capture are process-global — serialise tests that touch them.
    static CFG_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn a_label_is_a_picture_and_receipt_width_changes_the_page() {
        let label = PrintFormat::kot();
        let bytes = job_bytes(&label, "Shop\nAWB LABEL\nSAMPLE-1001\nTO\n");
        assert!(
            bytes.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
            "label bytes are a raster, so the bars survive"
        );
        assert!(paper_width_dots(100) > paper_width_dots(58));
        let mut receipt = PrintFormat::standard_portrait();
        receipt.content_width_mm = 80;
        let wide = receipt.raster_metrics().page_dots();
        receipt.content_width_mm = 58;
        assert!(wide > receipt.raster_metrics().page_dots());
    }

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
        // The whole job, then only the status request (DLE EOT 1).
        let mut want = escpos_job("Receipt INV-MAIN-000012\n1 kg parcel");
        want.extend_from_slice(&[0x10, 0x04, 0x01]);
        assert_eq!(got.join().unwrap(), want);
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

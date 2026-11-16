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
/// OS documents print at real size on the printer's own paper
/// (docs/PRINTER.md § Paper and preview).
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

/// A printer on this device and the paper in it (docs/PRINTER.md § Paper
/// and preview — owner, 2026-11-16: the printer owns its paper).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedPrinter {
    pub id: String,
    pub label: String,
    pub target: String,
    /// Width of the paper in it, mm; 0 = not known yet (Find fills it).
    #[serde(default, skip_serializing_if = "mm_unset")]
    pub paper_width_mm: f64,
    /// Length of one sheet, mm; 0 = a roll (each job as long as its content).
    #[serde(default, skip_serializing_if = "mm_unset")]
    pub paper_length_mm: f64,
    #[serde(default, alias = "wire", alias = "mode", skip_serializing)]
    pub _legacy_mode: Option<String>,
}


fn mm_unset(v: &f64) -> bool {
    !(*v > 0.0)
}

/// The default when a printer cannot say what paper it holds: the common
/// 80 mm receipt roll (owner, 2026-11-16). Never A4 or another sheet.
pub const DEFAULT_PAPER_WIDTH_MM: f64 = 80.0;

/// Narrowest and widest paper a printer may be set to, mm (a typo guard).
pub const PAPER_MM_MIN: f64 = 10.0;
pub const PAPER_MM_MAX: f64 = 2000.0;

/// mm to 0.1 mm, as people read and type it.
pub fn round_mm(mm: f64) -> f64 {
    (mm * 10.0).round() / 10.0
}

impl NamedPrinter {
    /// The paper set for this printer, if any.
    pub fn paper(&self) -> Option<crate::os_paper::PaperMm> {
        crate::os_paper::PaperMm::new(self.paper_width_mm, self.paper_length_mm)
    }

    /// A system print queue (or the system default) — the only printers
    /// with a driver to ask what paper they hold. ESC/POS devices (network
    /// raw, USB, Bluetooth) and Android's print services have none.
    pub fn has_driver(&self) -> bool {
        matches!(parse_target(&self.target), PrinterTarget::Named(_) | PrinterTarget::Default)
    }

    /// Give a printer with no paper one: the driver's current paper, else
    /// (nothing to ask, or a driver that cannot say) an 80 mm roll. A paper
    /// already set is never changed. `driver` is `None` when it was not asked
    /// (or not yet answered). `default_when_unanswered`: Find (and a job,
    /// which must print on something) give the default to a printer whose
    /// driver did not answer (owner, 2026-11-16); a screen repaint never
    /// does, so a moment's silence is not saved as its paper. Returns
    /// whether it changed.
    pub fn fill_paper(
        &mut self,
        driver: Option<&Result<crate::os_paper::PrinterPapers, String>>,
        default_when_unanswered: bool,
    ) -> bool {
        if self.paper().is_some() {
            return false;
        }
        let set = |p: &mut Self, w: f64, l: f64| {
            p.paper_width_mm = w;
            p.paper_length_mm = l;
            true
        };
        // Nothing to ask (ESC/POS devices, Android's print services — the
        // system dialog picks their paper): the default straight away.
        if !self.has_driver() {
            return set(self, DEFAULT_PAPER_WIDTH_MM, 0.0);
        }
        match driver {
            Some(Ok(papers)) => match papers.default_paper() {
                Some(p) => set(self, round_mm(p.width_mm), round_mm(p.height_mm)),
                None => set(self, DEFAULT_PAPER_WIDTH_MM, 0.0),
            },
            Some(Err(_)) if default_when_unanswered => set(self, DEFAULT_PAPER_WIDTH_MM, 0.0),
            _ => false,
        }
    }
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
    DevicePrinters::default()
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
            paper_width_mm: 0.0,
            paper_length_mm: 0.0,
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
                Some(PrintDest {
                    target: parse_target(&p.target),
                    format: format.clone(),
                    paper: p.paper(),
                })
            })
            .collect()
    }

    /// The paper in one printer, as the owner typed it on Settings →
    /// Printers: width mm, length mm (0 = a roll). Out-of-range values are
    /// refused (returns false), nothing is clamped into a size not typed.
    pub fn set_printer_paper_mm(&mut self, printer_id: &str, width_mm: f64, length_mm: f64) -> bool {
        let ok = |v: f64| (PAPER_MM_MIN..=PAPER_MM_MAX).contains(&v);
        if !ok(width_mm) || !(length_mm == 0.0 || ok(length_mm)) {
            return false;
        }
        match self.printers.iter_mut().find(|p| p.id == printer_id.trim()) {
            Some(p) => {
                p.paper_width_mm = round_mm(width_mm);
                p.paper_length_mm = round_mm(length_mm);
                true
            }
            None => false,
        }
    }

    /// Find's step for papers: every printer with no paper gets one from its
    /// driver's answer in `drivers` (target → answer), or the default where
    /// that is right (`NamedPrinter::fill_paper`). Never changes a paper
    /// already set. Returns how many printers got one.
    pub fn fill_missing_papers(
        &mut self,
        drivers: &std::collections::HashMap<String, Result<crate::os_paper::PrinterPapers, String>>,
        default_when_unanswered: bool,
    ) -> usize {
        let mut n = 0;
        for p in &mut self.printers {
            if p.fill_paper(drivers.get(p.target.trim()), default_when_unanswered) {
                n += 1;
            }
        }
        n
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
    /// OS document: the paper set for the printer (`None` until known; the
    /// send then asks the driver, as Find would). See [`NamedPrinter::fill_paper`].
    pub paper: Option<crate::os_paper::PaperMm>,
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
                    match send_to_printer(&dest.format, &dest.target, &job.path, &job.text, dest.paper) {
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
    paper: Option<crate::os_paper::PaperMm>,
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
                print_system_standard(None, path, text, format, format.landscape, 1, paper)
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
                print_system_standard(Some(name), path, text, format, format.landscape, 1, paper)
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

/// The paper a job goes on: the printer's own, or — not set yet — what Find
/// would give it from the same driver answer (the driver's paper, else an
/// 80 mm roll). Never a size of the app's choosing.
fn job_paper(
    paper: Option<crate::os_paper::PaperMm>,
    driver: &Result<crate::os_paper::PrinterPapers, String>,
) -> crate::os_paper::PaperMm {
    if let Some(p) = paper {
        return p;
    }
    let mut probe = NamedPrinter {
        id: String::new(),
        label: String::new(),
        target: String::new(),
        paper_width_mm: 0.0,
        paper_length_mm: 0.0,
        _legacy_mode: None,
    };
    probe.fill_paper(Some(driver), true);
    probe.paper().unwrap_or(crate::os_paper::PaperMm {
        width_mm: DEFAULT_PAPER_WIDTH_MM,
        length_mm: 0.0,
    })
}

/// Android: the upright slip and its real width go to the system print
/// dialog, where the owner picks the printer and the paper; the slip is drawn
/// at its real size from that paper's printable top-left (patches/makepad/
/// MakepadActivity-print.java `printSlipPages`). No paper is assumed.
fn print_android_document(text: &str, format: &PrintFormat, landscape: bool) -> Result<(), String> {
    let (png, mm) = os_document_slip(text, format)?;
    #[cfg(target_os = "android")]
    {
        crate::print_android::print_slip("Moklanjo", &png, mm, landscape)
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (png, mm, landscape);
        Err("Android print service is only on Android".into())
    }
}

/// The OS-document slip: the picture a receipt or label prints as (the
/// preview's), upright, and its width on paper in mm. Landscape is the
/// paper's business (each system turns it on the sheet).
fn os_document_slip(text: &str, format: &PrintFormat) -> Result<(Vec<u8>, f64), String> {
    let png = slip_png(format, text)?;
    Ok((png, os_document_content_mm(&format.raster_metrics())))
}

/// The slip's width on paper, in mm: its own dots at 7.2 per mm, so a px
/// is 1/96 inch on paper at every content width (docs/PRINTER.md lesson 22).
pub fn os_document_content_mm(metrics: &crate::receipt_raster::RasterMetrics) -> f64 {
    metrics.page_dots().max(1) as f64 * 80.0 / 576.0
}

/// OS document: the preview's slip picture at its real size on a page that
/// is exactly the printer's paper (Settings → Printers; docs/PRINTER.md §
/// Paper and preview), from the top-left of its printable area. Plain UTF-8
/// text depended on each driver's fonts (Indic became □).
fn print_system_standard(
    name: Option<&str>,
    path: &Path,
    text: &str,
    format: &PrintFormat,
    landscape: bool,
    copies: u32,
    paper: Option<crate::os_paper::PaperMm>,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = (name, path, paper);
        for _ in 0..copies.max(1) {
            print_android_document(text, format, landscape)?;
        }
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        // The driver lays the sheets out (its own printable area per page).
        let (png, mm) = os_document_slip(text, format)?;
        let slip = image::load_from_memory(&png).map_err(|e| e.to_string())?.to_luma8();
        let (w, h) = slip.dimensions();
        let gaps = crate::os_paper::line_gaps(slip.as_raw(), w as usize, h as usize);
        let len = crate::os_paper::ink_end(slip.as_raw(), w as usize, h as usize).max(1);
        let want = match paper {
            Some(p) => p,
            None => job_paper(paper, &crate::os_paper::query(name.unwrap_or(""))),
        };
        let png_path = path.with_extension("png");
        fs::write(&png_path, &png).map_err(|e| e.to_string())?;
        print_windows_slip(name, &png_path, mm, landscape, copies.max(1), want, &gaps, len)
    }
    #[cfg(all(not(target_os = "android"), not(target_os = "windows")))]
    {
        let (png, mm) = os_document_slip(text, format)?;
        print_cups_slip(name, path, &png, mm, landscape, copies.max(1), paper)
    }
}

/// CUPS (Linux, macOS): the page is the printer's paper — the driver's own
/// size when one matches, else `Custom.<w>x<l>mm`, a roll as long as the
/// slip (`os_paper::page_for`). The slip is cut into sheets exactly that
/// page's printable width, fitted (1:1) into its printable area at the
/// top-left; every sheet goes in one job with the page named. The preview
/// is drawn from the same plan.
#[cfg(all(not(target_os = "android"), not(target_os = "windows")))]
fn print_cups_slip(
    name: Option<&str>,
    path: &Path,
    png: &[u8],
    content_mm: f64,
    landscape: bool,
    copies: u32,
    paper: Option<crate::os_paper::PaperMm>,
) -> Result<(), String> {
    let slip = image::load_from_memory(png).map_err(|e| e.to_string())?.to_luma8();
    let (queue, driver) = match crate::os_paper::query_cups(name) {
        Ok((queue, papers)) => (Some(queue), Ok(papers)),
        Err(why) => (None, Err(why)),
    };
    let want = job_paper(paper, &driver);
    let plan = crate::os_paper::page_for(want, driver.as_ref().ok(), slip_mm(&slip, content_mm), landscape);
    if plan.fallback {
        let mm = crate::os_paper::mm_text;
        let asked = if want.is_roll() {
            format!("{} mm roll", mm(want.width_mm))
        } else {
            format!("{} × {} mm", mm(want.width_mm), mm(want.length_mm))
        };
        note_problem(format!(
            "{}: its driver has no {asked} paper and takes no other size — printed on its own paper, {} ({} × {} mm)",
            queue.as_deref().or(name).unwrap_or("printer"),
            plan.page.label,
            mm(plan.page.width_mm),
            mm(plan.page.height_mm)
        ));
    }
    if let Err(why) = &driver {
        // Still the printer's paper, from the top-left — never centred; only
        // the edges it cannot reach are not known (CUPS fits the page into
        // them, a few % smaller at most).
        note_problem(format!(
            "{why} — printed on the printer's paper ({}) from the top-left; the edges the printer cannot reach are unknown",
            plan.media
        ));
    }
    let (sheets, px_per_mm) = crate::os_paper::os_document_sheets(&slip, content_mm, &plan.page, landscape);
    let ppm = (px_per_mm * 1000.0).round() as u32;
    let ppi = ((px_per_mm * 25.4).round() as u32).max(1);
    let mut files = Vec::with_capacity(sheets.len());
    for (i, sheet) in sheets.into_iter().enumerate() {
        let mut buf = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageLuma8(sheet)
            .write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        let file = if i == 0 {
            path.with_extension("png")
        } else {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("receipt");
            path.with_file_name(format!("{stem}-{}.png", i + 1))
        };
        fs::write(&file, png_with_density(buf.into_inner(), ppm)).map_err(|e| e.to_string())?;
        files.push(file);
    }
    run_lp(queue.as_deref().or(name), copies, &lp_sheet_options(&plan.media, ppi), &files)
}

/// The upright slip's width and inked length on paper, mm.
pub fn slip_mm(slip: &image::GrayImage, content_mm: f64) -> (f64, f64) {
    let (w, h) = slip.dimensions();
    let px_per_mm = f64::from(w.max(1)) / content_mm.max(1.0);
    let len = crate::os_paper::ink_end(slip.as_raw(), w as usize, h as usize).max(1);
    (content_mm, f64::from(len) / px_per_mm)
}

/// `lp` with the given options and sheets, one job.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn run_lp(
    name: Option<&str>,
    copies: u32,
    options: &[String],
    files: &[std::path::PathBuf],
) -> Result<(), String> {
    let mut cmd = Command::new("lp");
    if let Some(name) = name {
        cmd.args(["-d", name]);
    }
    if copies > 1 {
        cmd.args(["-n", &copies.to_string(), "-o", "collate=true"]);
    }
    cmd.args(options);
    cmd.arg("--");
    for f in files {
        cmd.arg(f);
    }
    match cmd.output() {
        Ok(out) if out.status.success() => lp_job_queued(name, &out.stdout),
        Ok(out) => Err(first_line(&out.stderr).unwrap_or_else(|| "the printing system refused the job".into())),
        Err(_) => {
            let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
            if let Some(first) = files.first() {
                let _ = Command::new(opener).arg(first).spawn();
            }
            Err("this computer has no printing system (CUPS) — the slip was opened instead".into())
        }
    }
}

/// `lp` options for one OS-document sheet: the page the sheets were cut
/// for, named so the driver uses it; each sheet is exactly that page's
/// printable width, so `fit` is 1:1 and `position` puts it at the printable
/// top-left (without `position`, or with `print-scaling=none`, CUPS centres
/// a picture — lesson 21). Portrait: landscape sheets were turned before
/// they were written.
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn lp_sheet_options(media: &str, ppi: u32) -> Vec<String> {
    [
        format!("media={media}"),
        "print-scaling=fit".to_string(),
        format!("ppi={ppi}"),
        "position=top-left".to_string(),
        "orientation-requested=3".to_string(),
    ]
    .into_iter()
    .flat_map(|o| ["-o".to_string(), o])
    .collect()
}

/// Windows: the printer's paper — the driver's own size that matches, else
/// a custom `PaperSize` (a roll as long as the slip) — and each page drawn
/// from the top-left of its printable area at the slip's real size; the
/// next page starts where the last one ended, in the gap between lines.
#[cfg(target_os = "windows")]
#[allow(clippy::too_many_arguments)]
fn print_windows_slip(
    name: Option<&str>,
    path: &Path,
    content_mm: f64,
    landscape: bool,
    copies: u32,
    paper: crate::os_paper::PaperMm,
    gaps: &[u32],
    len: u32,
) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script = windows_slip_script(
        &path.display().to_string(),
        name,
        paper,
        content_mm,
        landscape,
        copies,
        gaps,
        len,
    );
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("PowerShell did not start ({e})"))?;
    if out.status.success() {
        // The driver printed on a paper other than the printer's: say so.
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(note) = line.trim().strip_prefix("KUMKAJ-NOTE: ") {
                note_problem(format!("{}: {note}", name.unwrap_or("printer")));
            }
        }
        Ok(())
    } else {
        Err(first_line(&out.stderr).unwrap_or_else(|| "Windows refused the job".into()))
    }
}

/// The PowerShell for [`print_windows_slip`]. Sizes are in hundredths of an
/// inch (the printer Graphics' unit); the origin is the printable area's
/// top-left (`OriginAtMargins` false). The page: the driver's size within
/// 0.6 mm of the printer's paper, else a custom one; a roll is first set
/// long, its printable area read, then set to the slip's length plus the
/// edges it cannot reach. Nothing is fitted: the slip's px per hundredth of
/// an inch come from its own mm.
#[cfg(any(target_os = "windows", test))]
#[allow(clippy::too_many_arguments)]
fn windows_slip_script(
    path: &str,
    printer: Option<&str>,
    paper: crate::os_paper::PaperMm,
    content_mm: f64,
    landscape: bool,
    copies: u32,
    gaps: &[u32],
    len: u32,
) -> String {
    let q = |s: &str| s.replace('\'', "''");
    let mut s = format!(
        "Add-Type -AssemblyName System.Drawing; \
         $img = [System.Drawing.Image]::FromFile('{}'); \
         $doc = New-Object System.Drawing.Printing.PrintDocument; \
         $doc.DocumentName = 'Moklanjo'; ",
        q(path)
    );
    if let Some(n) = printer.map(str::trim).filter(|n| !n.is_empty()) {
        s.push_str(&format!("$doc.PrinterSettings.PrinterName = '{}'; ", q(n)));
    }
    let gaps_ps = gaps.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",");
    s.push_str(&format!(
        "$doc.DefaultPageSettings.Landscape = {land}; \
         $doc.PrinterSettings.Copies = [int16]{copies}; \
         $doc.OriginAtMargins = $false; \
         $script:img = $img; \
         $script:per = $img.Width / ({mm:.4} / 25.4 * 100.0); \
         $W = {w:.4}; $L = {l:.4}; \
         if ({land}) {{ $along = {mm:.4} / 25.4 * 100.0 }} else {{ $along = {len} / $script:per }}; \
         $script:note = ''; \
         $own = $null; \
         if ($L -gt 0) {{ foreach ($p in $doc.PrinterSettings.PaperSizes) {{ \
           if ([Math]::Abs($p.Width - $W) -le 2.4 -and [Math]::Abs($p.Height - $L) -le 2.4) {{ $own = $p; break }} }} }}; \
         $canCustom = @($doc.PrinterSettings.PaperSizes | Where-Object {{ $_.Kind -eq 'Custom' }}).Count -gt 0; \
         if ($own) {{ $doc.DefaultPageSettings.PaperSize = $own }} elseif (-not $canCustom) {{ \
           $script:note = 'KUMKAJ-NOTE: its driver has no such paper and takes no other size; printed on its own paper, ' + $doc.DefaultPageSettings.PaperSize.PaperName }} else {{ \
           if ($L -gt 0) {{ $H = $L }} else {{ $H = $along + 100 }}; \
           $doc.DefaultPageSettings.PaperSize = New-Object System.Drawing.Printing.PaperSize('Custom', [int][Math]::Round($W), [int][Math]::Ceiling($H)); \
           if ($L -le 0) {{ \
             $a = $doc.DefaultPageSettings.PrintableArea; $b = $doc.DefaultPageSettings.Bounds; \
             if ({land}) {{ $edges = $a.X + ($b.Width - $a.X - $a.Width) }} else {{ $edges = $a.Y + ($b.Height - $a.Y - $a.Height) }}; \
             if ($edges -lt 0) {{ $edges = 0 }}; \
             $doc.DefaultPageSettings.PaperSize = New-Object System.Drawing.Printing.PaperSize('Custom', [int][Math]::Round($W), [int][Math]::Ceiling($along + $edges + 2)) }} }}; \
         $script:gaps = @({gaps_ps}); \
         $script:len = {len}; \
         $script:y = 0; \
         $doc.add_PrintPage({{ param($s,$e) \
           $a = $e.PageSettings.PrintableArea; \
           $room = [int][Math]::Floor($a.Height * $script:per); \
           if ($room -lt 1) {{ $room = 1 }}; \
           $wPx = [Math]::Min($script:img.Width, [Math]::Floor($a.Width * $script:per)); \
           $start = $script:y; $limit = $start + $room; \
           if ($limit -ge $script:len) {{ $end = $script:len }} else {{ \
             $end = $limit; $floor = $start + [Math]::Floor($room / 2); \
             foreach ($g in $script:gaps) {{ if ($g -gt $floor -and $g -le $limit) {{ $end = $g }} }} \
           }}; \
           $src = New-Object System.Drawing.RectangleF(0, $start, $wPx, ($end - $start)); \
           $dst = New-Object System.Drawing.RectangleF(0, 0, ($wPx / $script:per), (($end - $start) / $script:per)); \
           $e.Graphics.DrawImage($script:img, $dst, $src, [System.Drawing.GraphicsUnit]::Pixel); \
           $script:y = $end; \
           $e.HasMorePages = ($script:y -lt $script:len) }}); \
         $doc.Print(); \
         if ($script:note) {{ Write-Output $script:note }}; \
         $img.Dispose(); \
         $doc.Dispose();",
        land = if landscape { "$true" } else { "$false" },
        mm = content_mm,
        w = paper.width_mm / 0.254,
        l = paper.length_mm / 0.254,
    ));
    s
}


#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn png_crc(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c ^= u32::from(b);
        for _ in 0..8 {
            if c & 1 != 0 {
                c = (c >> 1) ^ 0xEDB8_8320;
            } else {
                c >>= 1;
            }
        }
    }
    !c
}

/// Store pixels-per-metre so the printer lays the slip out at `content_mm`
/// instead of assuming 72 dpi and clipping the page down to two rules.
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn png_with_density(png: Vec<u8>, ppm: u32) -> Vec<u8> {
    if ppm == 0 || png.windows(4).any(|w| w == *b"pHYs") {
        return png;
    }
    let Some(at) = png.windows(4).position(|w| w == *b"IHDR") else {
        return png;
    };
    if at < 4 || at + 4 > png.len() {
        return png;
    }
    let len = u32::from_be_bytes(png[at - 4..at].try_into().unwrap_or([0; 4])) as usize;
    let after = at + 4 + len + 4;
    if after > png.len() {
        return png;
    }
    let mut body = Vec::with_capacity(13);
    body.extend_from_slice(b"pHYs");
    body.extend(ppm.to_be_bytes());
    body.extend(ppm.to_be_bytes());
    body.push(1);
    let mut chunk = Vec::with_capacity(21);
    chunk.extend((9u32).to_be_bytes());
    chunk.extend_from_slice(&body);
    chunk.extend(png_crc(&body).to_be_bytes());
    let mut out = Vec::with_capacity(png.len() + chunk.len());
    out.extend_from_slice(&png[..after]);
    out.extend_from_slice(&chunk);
    out.extend_from_slice(&png[after..]);
    out
}






pub fn print_network(addr: &str, text: &str) -> Result<(), String> {
    print_network_mode(addr, PrinterMode::Kot, text)
}

pub fn print_network_mode(addr: &str, mode: PrinterMode, text: &str) -> Result<(), String> {
    let format = match mode {
        PrinterMode::Standard => PrintFormat::standard(),
        PrinterMode::Kot => PrintFormat::kot(),
    };
    print_network_bytes(addr, &escpos_job_format(&format, text))
}

/// ESC/POS to a network printer: connect to its raw port, send the
/// whole job, close — the standard way. No retries, status probing or
/// draining layered on top (2026-11-15).
fn print_network_bytes(addr: &str, bytes: &[u8]) -> Result<(), String> {
    let sock: std::net::SocketAddr = addr.parse().map_err(|_| format!("{addr} is not a printer address"))?;
    let mut stream = std::net::TcpStream::connect_timeout(&sock, Duration::from_secs(5))
        .map_err(|e| format!("cannot reach it ({e}) — is it on and on this network?"))?;
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    stream.write_all(bytes).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    let _ = stream.shutdown(std::net::Shutdown::Write);
    Ok(())
}





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
    let metrics = format.raster_metrics();
    // The slip as a picture — the same picture as the preview — in standard
    // ESC/POS raster (`GS v 0`), whatever the text or the printer.
    crate::receipt_raster::escpos_picture_metrics(&cleaned, &metrics)
}











/// ESC/POS to a system printer: the whole job as one raw job on the
/// queue the owner picked (`lp -o raw`, Windows `WritePrinter`). Never
/// routed around that queue (docs/PRINTER.md "What went wrong" 9).
fn print_system_raw(name: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    let dir = std::env::temp_dir().join("kumkaj-kot-raw");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let result = print_system_raw_path(name, &path);
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


/// `lp` accepted the job: the queue has it, and the system prints it. Log
/// its id. The app does not watch, judge or cancel a queued job — the queue
/// is the owner's (2026-11-15).
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn lp_job_queued(name: Option<&str>, lp_stdout: &[u8]) -> Result<(), String> {
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
    Ok(())
}







/// Windows script: hand `$bytes` to WritePrinter until all of it is taken
/// (it may take a job in parts) — one job, no pauses.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn powershell_write_loop(class: &str) -> String {
    format!(
        r#"$offset = 0
        while ($offset -lt $bytes.Length) {{
          $written = 0
          $ptr = [IntPtr]::Add($pinned.AddrOfPinnedObject(), $offset)
          if (-not [{class}]::WritePrinter($h, $ptr, $bytes.Length - $offset, [ref]$written)) {{ throw "WritePrinter failed" }}
          if ($written -le 0) {{ throw "WritePrinter wrote nothing" }}
          $offset += $written
        }}"#
    )
}

fn print_system_raw_path(name: Option<&str>, path: &Path) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // A raw job on the picked queue: the bytes reach the device as-is.
        let mut cmd = Command::new("lp");
        cmd.args(["-o", "raw"]);
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
        cmd.arg("--").arg(path);
        match cmd.output() {
            Ok(out) if out.status.success() => lp_job_queued(name, &out.stdout),
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
            write_loop = powershell_write_loop("KumkajRawPrint"),
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
            // What paper each system printer's driver says it holds, read
            // here (off the screen), so the merge can fill a printer that
            // has none. A paper already set is never changed by it
            // (docs/PRINTER.md § Paper and preview).
            for target in &found {
                if find_cancelled() {
                    break;
                }
                if matches!(parse_target(target), PrinterTarget::Named(_) | PrinterTarget::Default) {
                    crate::os_paper::remember(target, crate::os_paper::query(target));
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



    /// The Windows raw writer hands the whole job to WritePrinter, no pauses.
    #[test]
    fn windows_raw_writer_sends_the_whole_job() {
        let script = powershell_write_loop("KumkajRawPrint");
        assert!(script.contains("[KumkajRawPrint]::WritePrinter"));
        assert!(!script.contains("Start-Sleep") && !script.contains("$pieces"));
    }


    /// ESC/POS is the slip picture in standard raster for any text.
    #[test]
    fn an_escpos_job_is_the_slip_picture() {
        for text in ["AWB 123\nDelhi", "AWB\nદિલ્હી"] {
            let job = escpos_job(text);
            assert_eq!(&job[..2], &[0x1B, 0x40], "{text}: reset first");
            assert!(job.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]), "{text}: raster");
            assert!(!job.windows(2).any(|w| w == [0x10, 0x04]), "{text}: no status query");
        }
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
        let dir = crate::test_dirs::run_root().join(format!("courier-print-{}", std::process::id()));
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
        // Exactly the job: nothing appended (no status query), nothing split.
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
        let base = crate::test_dirs::run_root().join(format!("courier-printers-{}", uuid::Uuid::new_v4().simple()));
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
    fn synthetic_default_printer_is_scrubbed() {
        let _g = CFG_LOCK.lock().unwrap();
        // A pool entry named Default with no address is never a real printer.
        let base = crate::test_dirs::run_root().join(format!("courier-scrub-{}", uuid::Uuid::new_v4().simple()));
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
        let dir = crate::test_dirs::run_root().join(format!("courier-label-skip-{}", uuid::Uuid::new_v4().simple()));
        let share = print_route(PrintKind::Label, RouteKey::Label, "AWB label", Some(&dir)).unwrap();
        assert!(!share.opened);
        assert!(take_captured().is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}

/// The printer's paper (owner, 2026-11-16; docs/PRINTER.md § Paper and
/// preview) — the same model and checks as Khaynjo's.
#[cfg(test)]
mod paper_tests {
    use super::*;

    /// Find's merge here: a printer whose address is saved already stays one row.
    fn ensure(d: &mut DevicePrinters, label: &str, target: &str) -> String {
        match d.printers.iter().find(|p| p.target.trim().eq_ignore_ascii_case(target.trim())) {
            Some(p) => p.id.clone(),
            None => d.add_printer(label, target),
        }
    }

    #[test]
    fn os_document_names_the_paper_and_starts_at_the_printable_top_left() {
        let opts = lp_sheet_options("Card", 548);
        let has = |o: &str| opts.windows(2).any(|w| w[0] == "-o" && w[1] == o);
        assert!(has("media=Card"), "{opts:?}");
        // Each sheet is the printable width, so fit is 1:1; without
        // `position` CUPS centres a picture (PRINTER.md lesson 21).
        assert!(has("print-scaling=fit"), "{opts:?}");
        assert!(has("position=top-left"), "{opts:?}");
        assert!(has("ppi=548"), "{opts:?}");
        // Never `print-scaling=none`: CUPS centres the page with it.
        assert!(!opts.iter().any(|o| o == "print-scaling=none"), "{opts:?}");
    }

    /// A job's paper is the printer's; not set yet, it is what Find would
    /// give it from the same driver answer — never a size of the app's own.
    #[test]
    fn a_job_goes_on_the_printers_paper_or_what_find_would_give_it() {
        let ppd = "*DefaultPageSize: Letter\n\
*PageSize Letter/Letter: \"\"\n*PaperDimension Letter/Letter: \"612 792\"\n";
        let driver = Ok(crate::os_paper::parse_ppd(ppd));
        let set = crate::os_paper::PaperMm { width_mm: 101.6, length_mm: 0.0 };
        assert_eq!(job_paper(Some(set), &driver), set, "the printer's own paper wins");
        let p = job_paper(None, &driver);
        assert!((p.width_mm - 215.9).abs() < 0.05 && (p.length_mm - 279.4).abs() < 0.05, "{p:?}");
        // No answer: the 80 mm roll, never A4.
        let p = job_paper(None, &Err("down".into()));
        assert_eq!(p, crate::os_paper::PaperMm { width_mm: 80.0, length_mm: 0.0 });
    }

    /// Find fills a printer with no paper and never changes one already set,
    /// however many times it runs (owner, 2026-11-16).
    #[test]
    fn find_twice_keeps_every_paper() {
        let ppd = "*DefaultPageSize: A4\n*PageSize A4/A4: \"\"\n*PaperDimension A4/A4: \"595 842\"\n";
        let mut d = DevicePrinters::default();
        let office = ensure(&mut d, "Office", "Office");
        let thermal = ensure(&mut d, "192.168.1.50:9100", "192.168.1.50:9100");
        // Android's print service has no driver to ask: the default.
        let android = ensure(&mut d, "Android print", "android-print:");
        let set = ensure(&mut d, "Counter", "Counter");
        assert!(d.set_printer_paper_mm(&set, 104.8, 0.0));
        let mut answers = std::collections::HashMap::new();
        answers.insert("Office".to_string(), Ok(crate::os_paper::parse_ppd(ppd)));
        // The counter's driver now says A4: its set paper stays.
        answers.insert("Counter".to_string(), Ok(crate::os_paper::parse_ppd(ppd)));
        for _ in 0..2 {
            d.fill_missing_papers(&answers, true);
            ensure(&mut d, "Office", "Office");
            ensure(&mut d, "Counter", "Counter");
        }
        let paper = |id: &str| d.printer_by_id(id).and_then(|p| p.paper());
        let o = paper(&office).unwrap();
        // Exactly what the driver says: 595 × 842 pt is 209.9 × 297.0 mm.
        assert!((o.width_mm - 209.9).abs() < 0.05 && (o.length_mm - 297.0).abs() < 0.05, "{o:?}");
        assert_eq!(paper(&thermal), crate::os_paper::PaperMm::new(80.0, 0.0), "ESC/POS: 80 mm roll");
        assert_eq!(paper(&android), crate::os_paper::PaperMm::new(80.0, 0.0), "nothing to ask: 80 mm roll");
        assert_eq!(paper(&set), crate::os_paper::PaperMm::new(104.8, 0.0), "set paper kept");
        // Saved and loaded again: the papers are still there.
        let json = serde_json::to_value(&d).unwrap();
        let back: DevicePrinters = serde_json::from_value(json).unwrap();
        assert_eq!(back.printer_by_id(&set).and_then(|p| p.paper()), crate::os_paper::PaperMm::new(104.8, 0.0));
        // A screen repaint never saves a default from a silent driver.
        let mut quiet = DevicePrinters::default();
        let q = ensure(&mut quiet, "Quiet", "Quiet");
        let mut silent = std::collections::HashMap::new();
        silent.insert("Quiet".to_string(), Err("down".to_string()));
        assert_eq!(quiet.fill_missing_papers(&silent, false), 0);
        assert!(quiet.printer_by_id(&q).unwrap().paper().is_none());
        assert_eq!(quiet.fill_missing_papers(&silent, true), 1, "Find gives the default");
    }

    /// Windows: the page is the printer's paper — the driver's size that
    /// matches, else a custom one (or, with no custom sizes, the driver's own
    /// paper, said so) — each page at the slip's real size from the
    /// printable area, never shrunk.
    #[test]
    fn windows_os_document_draws_real_size_on_the_printers_paper() {
        let card = crate::os_paper::PaperMm { width_mm: 101.6, length_mm: 152.4 };
        let s = windows_slip_script(r"C:\Shop's\slip.png", Some("Front desk"), card, 80.0, false, 2, &[30, 75], 120);
        assert!(s.contains(r"FromFile('C:\Shop''s\slip.png')"), "{s}");
        assert!(s.contains("PrinterName = 'Front desk'"), "{s}");
        assert!(s.contains("$W = 400.0000; $L = 600.0000;"), "{s}");
        assert!(s.contains("PrintableArea") && s.contains("OriginAtMargins = $false"), "{s}");
        assert!(s.contains("$dst = New-Object System.Drawing.RectangleF(0, 0,"), "{s}");
        assert!(!s.contains("PageBounds"), "never the page: the printable area");
        assert!(s.contains("$_.Kind -eq 'Custom'") && s.contains("KUMKAJ-NOTE: "), "{s}");
        assert!(s.contains("$img.Width / (80.0000 / 25.4 * 100.0)"), "{s}");
        let roll = crate::os_paper::PaperMm { width_mm: 80.0, length_mm: 0.0 };
        let r = windows_slip_script("a.png", None, roll, 80.0, true, 1, &[], 10);
        assert!(r.contains("$L = 0.0000;") && r.contains("$along + $edges + 2"), "{r}");
    }

    /// The OS document is the slip at its own dots ÷ 7.2 mm wide, so a px is
    /// 1/96 inch on paper at every content width (PRINTER.md lesson 22).
    #[test]
    fn os_document_slip_is_7_2_dots_per_mm_at_every_width() {
        for mm in [58u32, 80, 104, 112, 210] {
            let mut f = PrintFormat::standard_portrait();
            f.content_width_mm = mm;
            f.line = wrap_cols_for_paper_mm(mm);
            let m = f.raster_metrics();
            let content = os_document_content_mm(&m);
            assert!((content - m.page_dots() as f64 / 7.2).abs() < 1e-9, "{mm}: {content}");
        }
    }
}

/// The picture a receipt or label prints as — the same dots as the job —
/// upright, one gray byte per dot (0 = ink). Drawn straight from the raster
/// (no PNG round trip on the screen's thread).
pub fn slip_gray(format: &PrintFormat, text: &str) -> image::GrayImage {
    let metrics = format.raster_metrics();
    // As each job draws it: OS documents the text as given, ESC/POS without CRs.
    let cleaned = if format.document {
        text.to_string()
    } else {
        text.replace('\r', "")
    };
    let (rows, height) = if format.is_label_format() {
        crate::receipt_raster::draw_label(&cleaned, &awb_from_slip(&cleaned), &metrics)
    } else {
        crate::receipt_raster::draw_metrics(&cleaned, &metrics)
    };
    let width = metrics.page_dots().max(8);
    let row_bytes = width / 8;
    let height = height.max(1);
    let mut gray = image::GrayImage::from_pixel(width as u32, height as u32, image::Luma([255]));
    for y in 0..height {
        let Some(row) = rows.get(y * row_bytes..(y + 1) * row_bytes) else {
            break;
        };
        for (i, byte) in row.iter().enumerate() {
            if *byte == 0 {
                continue;
            }
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    gray.put_pixel((i * 8 + bit) as u32, y as u32, image::Luma([0]));
                }
            }
        }
    }
    gray
}

/// Body-type height in the print picture, in its pixels (one per dot).
pub fn preview_type_px(em: f32) -> f64 {
    f64::from(crate::receipt_raster::px_dots(em))
}

/// One scale for the whole print picture on screen.
///
/// `column` is the settings width. `viewport` is how tall the picture may be
/// before the page scrolls. `type_px` is the body type inside the picture.
/// A print that fits at a readable size is fitted. A print that would become
/// unreadable if fitted to the viewport is held at a readable size instead,
/// still no wider than the column, and the page scrolls for the rest.
pub fn preview_frame(png_w: f64, png_h: f64, column: f64, viewport: f64, type_px: f64) -> (f64, f64) {
    const READABLE: f64 = 15.0;
    let png_w = png_w.max(1.0);
    let png_h = png_h.max(1.0);
    let column = column.max(40.0);
    let viewport = viewport.max(160.0);
    let fit_w = column / png_w;
    let fit = fit_w.min(viewport / png_h);
    let type_px = type_px.max(1.0);
    let scale = if type_px * fit >= READABLE {
        fit
    } else {
        (READABLE / type_px).min(fit_w)
    };
    let w = (png_w * scale).clamp(40.0, column);
    let h = png_h * (w / png_w);
    (w, h.max(1.0))
}

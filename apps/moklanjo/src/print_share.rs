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
#[cfg(any(target_os = "linux", target_os = "macos"))]
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
    /// Where the content sits on a paper bigger than it (owner,
    /// 2026-11-17): top / middle / bottom × left / centre / right — the
    /// paper's own sides (docs/PRINTER.md § Paper and preview). Top-left,
    /// where content always sat, is not written.
    #[serde(default, skip_serializing_if = "crate::os_paper::Place::is_top_left")]
    pub place: crate::os_paper::Place,
}

/// How `format`'s slip is laid on its sheet: turned or not, and the
/// format's place.
pub fn slip_lay(format: &PrintFormat) -> crate::os_paper::Lay {
    crate::os_paper::Lay { landscape: format.landscape, place: format.place }
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
            place: crate::os_paper::Place::default(),
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
            place: crate::os_paper::Place::default(),
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
            place: crate::os_paper::Place::default(),
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
    /// The virtual paper: the paper the owner says this printer holds, mm
    /// wide. Only the preview and the place grid use it — it is never sent
    /// to the printer, which prints on its own paper as its driver is set
    /// (owner, 2026-11-17). Set on Settings → Printers, filled by Find while
    /// it is 0, never changed by Find once set. Files before 2026-11-17 call
    /// it `paper_width_mm`.
    #[serde(default, alias = "paper_width_mm", skip_serializing_if = "mm_unset")]
    pub virtual_paper_width_mm: f64,
    /// The virtual paper's length, mm; 0 = a roll (each job as long as its
    /// content). Formerly `paper_length_mm`.
    #[serde(default, alias = "paper_length_mm", skip_serializing_if = "mm_unset")]
    pub virtual_paper_length_mm: f64,
    /// The paper this printer's driver reports it is set to — its own page,
    /// what an OS document really prints on — as the system last said
    /// (owner, 2026-11-17). Shown read-only as *Reported* beside the virtual
    /// paper the owner sets; never typed. Replaced whenever the driver
    /// answers (Find asks every one afresh). 0 = not reported: an ESC/POS
    /// device has no driver to ask, and Android asks in its print dialog.
    #[serde(default, skip_serializing_if = "mm_unset")]
    pub reported_width_mm: f64,
    #[serde(default, skip_serializing_if = "mm_unset")]
    pub reported_length_mm: f64,
    /// The driver's own name for that paper.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reported_name: String,
    /// The edges of that paper the printer cannot reach, mm: left, top,
    /// right, bottom — kept so the preview need not wait for the driver.
    #[serde(default, skip_serializing_if = "no_edges")]
    pub reported_edges_mm: [f64; 4],
    #[serde(default, alias = "wire", alias = "mode", skip_serializing)]
    pub _legacy_mode: Option<String>,
}


fn mm_unset(v: &f64) -> bool {
    !(*v > 0.0)
}

fn no_edges(v: &[f64; 4]) -> bool {
    v.iter().all(|m| !(*m > 0.0))
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
    /// The virtual paper set for this printer, if any.
    pub fn virtual_paper(&self) -> Option<crate::os_paper::PaperMm> {
        crate::os_paper::PaperMm::new(self.virtual_paper_width_mm, self.virtual_paper_length_mm)
    }

    /// The printer's own page as its driver last reported it, if it did.
    pub fn reported_paper(&self) -> Option<crate::os_paper::Paper> {
        (self.reported_width_mm > 0.0 && self.reported_length_mm > 0.0).then(|| crate::os_paper::Paper {
            id: self.reported_name.clone(),
            label: self.reported_name.clone(),
            width_mm: self.reported_width_mm,
            height_mm: self.reported_length_mm,
            margins_mm: self.reported_edges_mm,
        })
    }

    /// Keep a driver's answer as this printer's reported paper: the page it
    /// is set to, or nothing when it could not say (the screens then read
    /// "N/A"). Never touches the virtual paper. Returns whether it changed.
    pub fn set_reported(&mut self, driver: &Result<crate::os_paper::PrinterPapers, String>) -> bool {
        // Hundredths of a mm: steady from one answer to the next, and finer
        // than any printer's dot.
        let tidy = |mm: f64| (mm.max(0.0) * 100.0).round() / 100.0;
        let (w, l, name, edges) = match driver.as_ref().ok().and_then(|d| d.default_paper()) {
            Some(p) => (tidy(p.width_mm), tidy(p.height_mm), p.label.trim().to_string(), p.margins_mm.map(tidy)),
            None => (0.0, 0.0, String::new(), [0.0; 4]),
        };
        let changed = self.reported_width_mm != w
            || self.reported_length_mm != l
            || self.reported_name != name
            || self.reported_edges_mm != edges;
        self.reported_width_mm = w;
        self.reported_length_mm = l;
        self.reported_name = name;
        self.reported_edges_mm = edges;
        changed
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
        if self.virtual_paper().is_some() {
            return false;
        }
        let set = |p: &mut Self, w: f64, l: f64| {
            p.virtual_paper_width_mm = w;
            p.virtual_paper_length_mm = l;
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
            virtual_paper_width_mm: 0.0,
            virtual_paper_length_mm: 0.0,
            reported_width_mm: 0.0,
            reported_length_mm: 0.0,
            reported_name: String::new(),
            reported_edges_mm: [0.0; 4],
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

    /// Where the receipt (both orientations) or the label sits on a paper
    /// bigger than it (owner, 2026-11-17). Kept with this device's
    /// printers, like portrait / landscape.
    pub fn set_job_place(&mut self, label: bool, place: crate::os_paper::Place) {
        for f in self.formats.iter_mut() {
            let hit = if label { f.id == FORMAT_KOT } else { is_standard_document(&f.id) };
            if hit {
                f.place = place;
            }
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
            place: base.place,
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
                    virtual_paper: p.virtual_paper(),
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
                p.virtual_paper_width_mm = round_mm(width_mm);
                p.virtual_paper_length_mm = round_mm(length_mm);
                true
            }
            None => false,
        }
    }

    /// What the drivers answered (`drivers`: target → answer), taken into
    /// the saved printers: each one's reported paper is replaced by its
    /// driver's answer (`NamedPrinter::set_reported`), and a printer with no
    /// virtual paper gets one ([`Self::fill_missing_papers`]). Returns how
    /// many printers changed.
    pub fn take_driver_answers(
        &mut self,
        drivers: &std::collections::HashMap<String, Result<crate::os_paper::PrinterPapers, String>>,
        default_when_unanswered: bool,
    ) -> usize {
        let mut n = 0;
        for p in &mut self.printers {
            let answer = drivers.get(p.target.trim());
            let reported = p.has_driver() && answer.is_some_and(|a| p.set_reported(a));
            let filled = p.fill_paper(answer, default_when_unanswered);
            if reported || filled {
                n += 1;
            }
        }
        n
    }

    /// Find's step for virtual papers: every printer with none gets one from
    /// its driver's answer in `drivers` (target → answer), or the default
    /// where that is right (`NamedPrinter::fill_paper`). Never changes a
    /// paper already set. Returns how many printers got one.
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

    /// Toggle a pool printer on a receipt or label route. Returns whether it is attached after.
    pub fn toggle_printer_on_route(&mut self, key: RouteKey, printer_id: &str) -> Option<bool> {
        let pid = printer_id.trim();
        if pid.is_empty() || self.printer_by_id(pid).is_none() {
            return None;
        }
        let route = self.route_mut(key);
        if let Some(i) = route.iter().position(|id| id == pid) {
            route.remove(i);
            Some(false)
        } else {
            route.push(pid.to_string());
            Some(true)
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
    /// The printer's virtual paper (Settings → Printers): where the format's
    /// place is measured and how wide an ESC/POS roll is. Never sent to the
    /// printer.
    pub virtual_paper: Option<crate::os_paper::PaperMm>,
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
                    match send_to_printer(dest, &job.path, &job.text) {
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

fn send_to_printer(dest: &PrintDest, path: &Path, text: &str) -> Result<(), String> {
    let (format, target) = (&dest.format, &dest.target);
    let thermal = || job_bytes_placed(format, text, dest.virtual_paper);
    match target {
        PrinterTarget::Network(addr) => print_network_bytes(addr, &thermal()),
        PrinterTarget::Bluetooth(mac) => crate::print_bt::write_bytes(mac, &thermal()),
        PrinterTarget::Usb(id) => crate::print_usb::write_bytes(id, &thermal()),
        PrinterTarget::AndroidPrint => {
            if !format.document {
                return Err(format!(
                    "format \"{}\" needs Network, Bluetooth, or USB",
                    format.label
                ));
            }
            print_android_document(text, format, dest.virtual_paper)
        }
        PrinterTarget::Default => {
            if format.document {
                print_system_standard(None, path, text, format, 1, dest)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB",
                    format.label
                ))
            } else {
                print_system_raw(None, &thermal())
            }
        }
        PrinterTarget::Named(name) => {
            if format.document {
                print_system_standard(Some(name), path, text, format, 1, dest)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB",
                    format.label
                ))
            } else {
                print_system_raw(Some(name), &thermal())
            }
        }
    }
}

/// Android: the upright slip and its real width go to the system print
/// dialog, where the owner picks the printer and its paper. The slip is
/// drawn at its real size on the part of that paper the virtual paper
/// covers, at the format's place, page after page — the same rule as
/// `os_paper::sheet_for` / `plan_sheets`, worked out where the dialog's
/// paper is known (patches/makepad/KumkajSlipLayout.java).
fn print_android_document(
    text: &str,
    format: &PrintFormat,
    virtual_paper: Option<crate::os_paper::PaperMm>,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let png = slip_png(format, text)?;
        crate::print_android::print_slip(
            "Moklanjo",
            &png,
            os_document_content_mm(&format.raster_metrics()),
            slip_lay(format),
            virtual_paper,
        )
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (text, format, virtual_paper);
        Err("Android print service is only on Android".into())
    }
}

/// The OS-document slip: the preview's own picture, upright, as grey
/// pixels (no PNG round trip), and its width on paper in mm. Landscape is
/// the sheet's business (`os_paper::render_sheets` turns the page), so the
/// slip itself is never turned here.
#[cfg(not(target_os = "android"))]
fn os_document_slip(text: &str, format: &PrintFormat) -> Result<(image::GrayImage, f64), String> {
    Ok((slip_gray(format, text), os_document_content_mm(&format.raster_metrics())))
}

/// The slip's width on paper, in mm: its own dots at 7.2 per mm, so a px
/// is 1/96 inch on paper at every content width (rule 2). The rounded
/// wrap-column count printed 70 mm as 71, 112 as 113, and a 384-dot slip
/// at 58 mm made its text ~8 % larger (docs/PRINTER.md lesson 22).
pub fn os_document_content_mm(metrics: &crate::receipt_raster::RasterMetrics) -> f64 {
    // `DOTS_PER_MM` (576 / 80) in f64: 7.2 has no exact f32, and 576 dots
    // must come out 80 mm, not 80.000002.
    metrics.page_dots().max(1) as f64 * 80.0 / 576.0
}

/// OS document, the standard way (owner, 2026-11-17): the job prints on the
/// printer's own paper exactly as its driver is set — the app names no
/// paper and makes none up. The slip is the preview's own picture at its
/// real size, laid on the part of the printer's page that the virtual
/// paper covers (`os_paper::Sheet`) at the format's place; a longer slip
/// goes on to the printer's next page. Plain UTF-8 text depended on each
/// driver's fonts (Indic became □).
fn print_system_standard(
    name: Option<&str>,
    path: &Path,
    text: &str,
    format: &PrintFormat,
    copies: u32,
    dest: &PrintDest,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = (name, path);
        for _ in 0..copies.max(1) {
            print_android_document(text, format, dest.virtual_paper)?;
        }
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        // Through the Windows print API itself (print_win.rs): the pages are
        // laid out for the page the printer's own device context reports.
        let _ = path;
        let (slip, content_mm) = os_document_slip(text, format)?;
        crate::print_win::print_pages(name, "Moklanjo", copies.max(1), None, |own| {
            os_document_pages(&slip, content_mm, own, format, dest.virtual_paper)
        })
    }
    #[cfg(all(not(target_os = "android"), not(target_os = "windows")))]
    {
        let (slip, content_mm) = os_document_slip(text, format)?;
        print_cups_pdf(name, path, &slip, content_mm, copies.max(1), format, dest.virtual_paper)
    }
}

/// An OS-document job's pages for the printer's own page `own` (as its
/// driver is set): the slip cut for the part of that page the virtual
/// paper covers, at the format's place (`os_paper::sheet_for`) — one grey
/// picture per page, each to go at the page's printable corner — and their
/// px per mm. A printer with no virtual paper set: its own page is all
/// paper.
#[cfg(not(target_os = "android"))]
fn os_document_pages(
    slip: &image::GrayImage,
    content_mm: f64,
    own: &crate::os_paper::Paper,
    format: &PrintFormat,
    virtual_paper: Option<crate::os_paper::PaperMm>,
) -> (Vec<image::GrayImage>, f64) {
    let lay = slip_lay(format);
    let sheet = crate::os_paper::sheet_for(
        virtual_paper.unwrap_or_else(|| own.as_virtual()),
        Some(own),
        slip_mm(slip, content_mm),
        lay.landscape,
    );
    crate::os_paper::os_document_sheets(slip, content_mm, &sheet, lay)
}

/// CUPS (Linux, macOS): one PDF for the job, handed to `lp` on the queue
/// the owner picked. The queue's own paper is read from its driver
/// (`os_paper::query_cups`: its PPD and the owner's `lpoptions` default)
/// and never named in the job, so the driver uses the paper it is set to.
/// Each PDF page is that paper's printable area with the page picture at
/// its real size from the top-left (`slip_pdf`). A queue whose driver
/// cannot be read is sent nothing: without its page the slip would land
/// wherever the system put it.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn print_cups_pdf(
    name: Option<&str>,
    path: &Path,
    slip: &image::GrayImage,
    content_mm: f64,
    copies: u32,
    format: &PrintFormat,
    virtual_paper: Option<crate::os_paper::PaperMm>,
) -> Result<(), String> {
    let (queue, papers) = crate::os_paper::query_cups(name)?;
    let own = papers
        .default_paper()
        .ok_or_else(|| format!("the driver of {queue} does not say which paper it is set to"))?;
    let file = path.with_extension("pdf");
    fs::write(&file, os_document_pdf(slip, content_mm, own, format, virtual_paper)).map_err(|e| e.to_string())?;
    run_lp(Some(&queue), copies, &lp_pdf_options(), &[file])
}

/// The job's PDF for a queue whose own paper is `own` ([`print_cups_pdf`]).
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn os_document_pdf(
    slip: &image::GrayImage,
    content_mm: f64,
    own: &crate::os_paper::Paper,
    format: &PrintFormat,
    virtual_paper: Option<crate::os_paper::PaperMm>,
) -> Vec<u8> {
    let (pages, px_per_mm) = os_document_pages(slip, content_mm, own, format, virtual_paper);
    crate::slip_pdf::slip_pdf(own.printable_mm(), &pages, px_per_mm)
}

/// The upright slip's width and inked length on paper, mm.
pub fn slip_mm(slip: &image::GrayImage, content_mm: f64) -> (f64, f64) {
    let (w, h) = slip.dimensions();
    let px_per_mm = f64::from(w.max(1)) / content_mm.max(1.0);
    let len = crate::os_paper::ink_end(slip.as_raw(), w as usize, h as usize).max(1);
    (content_mm, f64::from(len) / px_per_mm)
}

/// `lp` with the given options and files, one job.
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

/// `lp` options for an OS-document PDF. No paper is named: the job prints
/// on the queue's own paper as its driver is set (owner, 2026-11-17). Each
/// PDF page is that paper's printable area, so `fit` lays it there 1:1
/// from the top-left (`slip_pdf` says why this rule and no other);
/// `fit-to-page` says the same to filters older than `print-scaling`.
/// Portrait: landscape pages were turned before they were written.
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn lp_pdf_options() -> Vec<String> {
    ["print-scaling=fit", "fit-to-page", "position=top-left", "orientation-requested=3"]
        .into_iter()
        .flat_map(|o| ["-o".to_string(), o.to_string()])
        .collect()
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

/// The slip as a PNG, for Android's system print dialog.
#[cfg(target_os = "android")]
fn slip_png(format: &PrintFormat, text: &str) -> Result<Vec<u8>, String> {
    let metrics = format.raster_metrics();
    if format.is_label_format() {
        crate::receipt_raster::png_label(text, &awb_from_slip(text), &metrics)
    } else {
        crate::receipt_raster::png_metrics(text, &metrics)
    }
}

/// The job as 1-bit rows — the same picture as the preview (a label with
/// its bars) — for standard ESC/POS raster (`GS v 0`), and its width in dots.
fn job_drawn(format: &PrintFormat, text: &str) -> ((Vec<u8>, usize), usize) {
    let cleaned = text.replace('\r', "");
    let metrics = format.raster_metrics();
    let drawn = if format.is_label_format() {
        crate::receipt_raster::draw_label(&cleaned, &awb_from_slip(&cleaned), &metrics)
    } else {
        crate::receipt_raster::draw_metrics(&cleaned, &metrics)
    };
    (drawn, metrics.page_dots())
}

/// The ESC/POS job at the format's place across the printer's roll (`roll`:
/// its virtual paper, when set; a roll is as long as the slip, so only
/// across counts — `os_paper::place_in`). A slip at the left — where every
/// slip sat before places existed, and still does unless the owner picks
/// another — is sent byte for byte as before.
fn job_bytes_placed(format: &PrintFormat, text: &str, roll: Option<crate::os_paper::PaperMm>) -> Vec<u8> {
    let (drawn, width) = job_drawn(format, text);
    let Some(roll) = roll else {
        return crate::receipt_raster::escpos_from_drawn(drawn, width);
    };
    // 576 dots per 80 mm, as the picture is drawn.
    let roll_dots = (roll.width_mm * 576.0 / 80.0).round().max(8.0) as usize;
    let (x, _) = crate::os_paper::place_in(format.place, width as u32, 0, roll_dots as u32, 0, false);
    if x == 0 {
        return crate::receipt_raster::escpos_from_drawn(drawn, width);
    }
    let placed = crate::receipt_raster::place_drawn(drawn, width, roll_dots, x);
    crate::receipt_raster::escpos_from_drawn(placed, roll_dots)
}

pub fn escpos_job_format(format: &PrintFormat, text: &str) -> Vec<u8> {
    let cleaned = text.replace('\r', "");
    let metrics = format.raster_metrics();
    // The slip as a picture — the same picture as the preview — in standard
    // ESC/POS raster (`GS v 0`), whatever the text or the printer.
    crate::receipt_raster::escpos_picture_metrics(&cleaned, &metrics)
}











/// ESC/POS to a system printer: the whole job as one raw job on the
/// queue the owner picked (`lp -o raw`; on Windows a "RAW" spooler job,
/// `print_win::print_raw`). Never routed around that queue
/// (docs/PRINTER.md "What went wrong" 9).
fn print_system_raw(name: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let dir = std::env::temp_dir().join("kumkaj-kot-raw");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
        fs::write(&path, bytes).map_err(|e| e.to_string())?;
        // A raw job on the picked queue: the bytes reach the device as-is.
        let mut cmd = Command::new("lp");
        cmd.args(["-o", "raw"]);
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
        cmd.arg("--").arg(&path);
        let result = match cmd.output() {
            Ok(out) if out.status.success() => lp_job_queued(name, &out.stdout),
            Ok(out) => {
                let detail = first_line(&out.stderr).unwrap_or_else(|| "raw print refused".into());
                Err(format!(
                    "{detail} — for kitchen tickets prefer a network address (host:9100) in Settings → Printers"
                ))
            }
            Err(_) => Err(
                "no printing system (CUPS) — add the kitchen printer as host:9100 in Settings → Printers"
                    .into(),
            ),
        };
        let _ = fs::remove_file(&path);
        result
    }
    #[cfg(target_os = "windows")]
    {
        // The bytes must reach the device as they are: a named queue takes
        // them as a raw job. With no name there is no queue to hand them to.
        let Some(name) = name else {
            return Err(
                "pick the kitchen printer by name, or use a network address (host:9100) in Settings → Printers"
                    .into(),
            );
        };
        crate::print_win::print_raw(name, "KOT", bytes).map_err(|detail| {
            format!("{detail} — for kitchen tickets prefer a network address (host:9100) in Settings → Printers")
        })
    }
    #[cfg(target_os = "android")]
    {
        let _ = (name, bytes);
        Err(
            "Android has no system printer drivers for kitchen tickets — use a network address (host:9100) in Settings → Printers"
                .into(),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
    {
        let _ = (name, bytes);
        Err("printing is not supported on this system".into())
    }
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









#[cfg(any(target_os = "linux", target_os = "macos"))]
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
    // Every driver is asked afresh by a Find: what a printer reported
    // before is not shown as what it reports now.
    crate::os_paper::forget_all();
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

/// System printer destinations (empty on Android — no drivers).
///
/// Linux/macOS: union of `lpstat -e` (every CUPS destination the OS knows)
/// and accepting queues from `lpstat -a`, minus queues `lpstat -p` marks
/// disabled. Using only `-a` dropped valid destinations (e.g. alternate
/// Brother_* names that `-e` lists) so Find looked like “only the one I
/// already added”.
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
        // `lpstat -p`: "printer NAME is idle.  enabled …" or "… disabled …"
        let disabled: std::collections::HashSet<String> = out(&["-p"])
            .lines()
            .filter(|l| l.contains(" disabled "))
            .filter_map(|l| {
                l.strip_prefix("printer ")
                    .and_then(|rest| rest.split_whitespace().next())
                    .map(str::to_string)
            })
            .collect();
        let mut names: Vec<String> = out(&["-e"])
            .lines()
            .map(str::trim)
            .filter(|n| !n.is_empty() && !disabled.contains(*n))
            .map(str::to_string)
            .collect();
        // Accepting queues — cover cases where `-e` is empty or incomplete.
        for n in out(&["-a"])
            .lines()
            .filter_map(|l| l.split_whitespace().next().map(str::to_string))
            .filter(|n| !n.is_empty() && !disabled.contains(n))
        {
            if !names.iter().any(|x| x == &n) {
                names.push(n);
            }
        }
        names.sort();
        names.dedup();
        names
    }
    #[cfg(target_os = "windows")]
    {
        // Windows' own list; one that works offline or is reported
        // offline is left out — a removed or unplugged ghost.
        crate::print_win::printers()
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
        let bytes = job_bytes_placed(&label, "Shop\nAWB LABEL\nSAMPLE-1001\nTO\n", None);
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

    /// The job is one PDF, sent the standard way: no paper named (the queue
    /// prints on its own, as its driver is set) and its page — the printable
    /// area — laid there 1:1 from the top-left (owner, 2026-11-17).
    #[test]
    fn os_document_names_no_paper_and_lays_its_page_on_the_printable_area() {
        let opts = lp_pdf_options();
        let has = |o: &str| opts.windows(2).any(|w| w[0] == "-o" && w[1] == o);
        assert!(!opts.iter().any(|o| o.starts_with("media=") || o.starts_with("PageSize=")), "{opts:?}");
        // The PDF page is the printable area, so fit is 1:1.
        assert!(has("print-scaling=fit"), "{opts:?}");
        assert!(has("fit-to-page"), "{opts:?}");
        assert!(has("position=top-left"), "{opts:?}");
        assert!(has("orientation-requested=3"), "{opts:?}");
        // Never `none`: the filters then leave a page to the printer's
        // corner, or shrink one the size of the paper (measured 2026-11-17).
        assert!(!opts.iter().any(|o| o == "print-scaling=none"), "{opts:?}");
    }

    /// A made-up driver's page: 595 × 842 pt, reachable 12 pt in from the
    /// sides and 24 pt from top and bottom.
    fn check_page() -> crate::os_paper::Paper {
        let pt = crate::os_paper::PT_MM;
        crate::os_paper::Paper {
            id: "Sheet".into(),
            label: "Sheet".into(),
            width_mm: 595.0 * pt,
            height_mm: 842.0 * pt,
            margins_mm: [12.0 * pt, 24.0 * pt, 12.0 * pt, 24.0 * pt],
        }
    }

    /// What CUPS is handed: every PDF page is the printable area of the
    /// printer's own page — never the virtual paper, never a size of the
    /// app's own — and a half-size virtual sheet only says where pages end.
    #[test]
    fn the_job_pdf_is_the_printers_own_printable_area_whatever_the_virtual_paper() {
        let mut f = PrintFormat::standard_portrait();
        f.line = wrap_cols_for_paper_mm(80);
        let mut text = String::from("CASH MEMO\n");
        for i in 0..60 {
            text.push_str(&format!("Item number {}\t1\t80.00\t80.00\n", i + 1));
        }
        let (slip, mm) = os_document_slip(&text, &f).unwrap();
        let own = check_page();
        let pages_of = |pdf: &[u8]| String::from_utf8_lossy(pdf).matches("/Type /Page ").count();
        // On its own page the bill is longer than one page.
        let whole = os_document_pdf(&slip, mm, &own, &f, None);
        let n_whole = pages_of(&whole);
        assert!(n_whole >= 2, "{n_whole}");
        // A half-length virtual sheet: more pages, the same PDF page size.
        let half = crate::os_paper::PaperMm { width_mm: 105.0, length_mm: 148.0 };
        let halved = os_document_pdf(&slip, mm, &own, &f, Some(half));
        assert!(pages_of(&halved) > n_whole, "pages end at the real sheet's length");
        for pdf in [&whole, &halved] {
            let text = String::from_utf8_lossy(pdf);
            assert_eq!(text.matches("/MediaBox [0 0 571 794]").count(), pages_of(pdf), "the printable area of the printer's own page");
        }
    }

    /// The reported paper is what the driver says; the virtual paper is the
    /// owner's. A driver's answer replaces the first and never the second.
    #[test]
    fn a_drivers_answer_is_kept_as_the_reported_paper_and_never_as_the_virtual_one() {
        let ppd = "*DefaultPageSize: Big\n*PageSize Big/Big sheet: \"\"\n*PaperDimension Big/Big sheet: \"595 842\"\n\
*ImageableArea Big/Big sheet: \"12 24 583 818\"\n*PageSize Small/Small card: \"\"\n*PaperDimension Small/Small card: \"288 432\"\n";
        let mut answers = std::collections::HashMap::new();
        answers.insert("Office".to_string(), Ok(crate::os_paper::parse_ppd(ppd)));
        let mut d = DevicePrinters::default();
        let office = ensure(&mut d, "Office", "Office");
        let thermal = ensure(&mut d, "192.168.50.20:9100", "192.168.50.20:9100");
        // The owner said: a half-size sheet is in the office printer.
        assert!(d.set_printer_paper_mm(&office, 105.0, 148.0));
        assert_eq!(d.take_driver_answers(&answers, true), 2, "the office printer's report; the thermal's default roll");
        let p = d.printer_by_id(&office).unwrap();
        assert_eq!(p.virtual_paper(), crate::os_paper::PaperMm::new(105.0, 148.0), "the owner's paper is kept");
        let reported = p.reported_paper().expect("reported");
        assert_eq!(reported.label, "Big sheet");
        assert!((reported.width_mm - 209.9).abs() < 0.01 && (reported.height_mm - 297.04).abs() < 0.01, "{reported:?}");
        assert!((reported.margins_mm[0] - 4.23).abs() < 0.01 && (reported.margins_mm[1] - 8.47).abs() < 0.01);
        // A device with no driver reports nothing.
        assert!(d.printer_by_id(&thermal).unwrap().reported_paper().is_none());
        // The same answer again changes nothing.
        assert_eq!(d.take_driver_answers(&answers, true), 0);
        // It is saved with the printer, and read back.
        let back: DevicePrinters = device_printers_from_value(serde_json::to_value(&d).unwrap());
        assert_eq!(back.printer_by_id(&office).unwrap().reported_paper(), Some(reported));
        let saved = serde_json::to_string(&d).unwrap();
        assert!(saved.contains("\"reported_width_mm\""), "{saved}");
        assert_eq!(saved.matches("reported_name").count(), 1, "nothing written for a printer that reported nothing");
        // The driver now set to the small card: the report follows it.
        answers.insert("Office".to_string(), Ok(crate::os_paper::parse_ppd(&ppd.replace("*DefaultPageSize: Big", "*DefaultPageSize: Small"))));
        assert_eq!(d.take_driver_answers(&answers, true), 1);
        let p = d.printer_by_id(&office).unwrap();
        assert_eq!(p.reported_paper().unwrap().label, "Small card");
        assert_eq!(p.virtual_paper(), crate::os_paper::PaperMm::new(105.0, 148.0));
        // The driver cannot be read any more: not reported.
        answers.insert("Office".to_string(), Err("the printing system did not answer".into()));
        assert_eq!(d.take_driver_answers(&answers, true), 1);
        assert!(d.printer_by_id(&office).unwrap().reported_paper().is_none());
        assert_eq!(d.printer_by_id(&office).unwrap().virtual_paper(), crate::os_paper::PaperMm::new(105.0, 148.0));
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
            // A second Find adds nothing and changes nothing.
            ensure(&mut d, "Office", "Office");
            ensure(&mut d, "Counter", "Counter");
        }
        let paper = |id: &str| d.printer_by_id(id).and_then(|p| p.virtual_paper());
        let o = paper(&office).unwrap();
        // Exactly what the driver says: 595 × 842 pt is 209.9 × 297.0 mm.
        assert!((o.width_mm - 209.9).abs() < 0.05 && (o.length_mm - 297.0).abs() < 0.05, "{o:?}");
        assert_eq!(paper(&thermal), crate::os_paper::PaperMm::new(80.0, 0.0), "ESC/POS: 80 mm roll");
        assert_eq!(paper(&android), crate::os_paper::PaperMm::new(80.0, 0.0), "nothing to ask: 80 mm roll");
        assert_eq!(paper(&set), crate::os_paper::PaperMm::new(104.8, 0.0), "set paper kept");
        // Saved and loaded again: the papers are still there.
        let json = serde_json::to_value(&d).unwrap();
        let back: DevicePrinters = serde_json::from_value(json).unwrap();
        assert_eq!(back.printer_by_id(&set).and_then(|p| p.virtual_paper()), crate::os_paper::PaperMm::new(104.8, 0.0));
        // A screen repaint never saves a default from a silent driver.
        let mut quiet = DevicePrinters::default();
        let q = ensure(&mut quiet, "Quiet", "Quiet");
        let mut silent = std::collections::HashMap::new();
        silent.insert("Quiet".to_string(), Err("down".to_string()));
        assert_eq!(quiet.fill_missing_papers(&silent, false), 0);
        assert!(quiet.printer_by_id(&q).unwrap().virtual_paper().is_none());
        assert_eq!(quiet.fill_missing_papers(&silent, true), 1, "Find gives the default");
    }

    /// No paper is assumed: on every page a driver may be set to — a
    /// 4-inch card, a narrow strip, a half sheet, a big sheet — the slip
    /// keeps its own size (never fitted to the paper), starts at the
    /// printable top-left, and is cut only past the printable edge
    /// (docs/PRINTER.md § Paper, lesson 24).
    #[test]
    fn os_document_keeps_its_real_size_on_paper_of_any_size() {
        let text = "Shop\nButter Roti\t2\nTotal 100\n";
        let mut f = PrintFormat::standard_portrait();
        f.line = wrap_cols_for_paper_mm(80);
        let (slip, mm) = os_document_slip(text, &f).unwrap();
        let ink = |img: &image::GrayImage| {
            let (w, h) = img.dimensions();
            let xs = (0..w).filter(|&x| (0..h).any(|y| img.get_pixel(x, y).0[0] < 128));
            let (mut lo, mut hi) = (u32::MAX, 0);
            for x in xs {
                lo = lo.min(x);
                hi = hi.max(x);
            }
            (lo, hi)
        };
        let (slip_lo, slip_hi) = ink(&slip);
        for (w, h, m) in [
            (101.6, 152.4, [6.35, 12.7, 6.35, 12.7]),
            (58.0, 3000.0, [1.5, 0.0, 1.5, 0.0]),
            (148.0, 210.0, [4.2, 4.2, 4.2, 4.2]),
            (330.0, 480.0, [5.0, 5.0, 5.0, 5.0]),
        ] {
            let own = crate::os_paper::Paper {
                id: "Any".into(),
                label: "Any".into(),
                width_mm: w,
                height_mm: h,
                margins_mm: m,
            };
            // No virtual paper set: the printer's own page is all paper.
            let (sheets, ppm) = os_document_pages(&slip, mm, &own, &f, None);
            // The slip's own density: nothing scaled.
            assert!((ppm - f64::from(slip.width()) / mm).abs() < 1e-9);
            let (pw, _) = own.printable_mm();
            assert_eq!(sheets[0].width(), (pw * ppm).floor() as u32, "{w} mm paper");
            let (lo, hi) = ink(&sheets[0]);
            assert_eq!(lo, slip_lo, "{w} mm paper: starts where the slip starts");
            if slip.width() <= sheets[0].width() {
                assert_eq!(hi, slip_hi, "{w} mm paper: the whole slip, its own size");
            } else {
                assert!(hi < sheets[0].width(), "{w} mm paper: cut at the printable edge");
            }
        }
    }

    /// `MOKLANJO_PAPER_MM`: `<w>x<l>` (mm) for a sheet, `<w>` for a roll.
    fn paper_mm_var(v: &str) -> crate::os_paper::PaperMm {
        let (w, l) = v.split_once('x').unwrap_or((v, "0"));
        crate::os_paper::PaperMm::new(w.trim().parse().unwrap_or(80.0), l.trim().parse().unwrap_or(0.0))
            .expect("a paper width")
    }

    /// `MOKLANJO_PLACE`: `<down>,<across>` (0 top / start … 2 bottom / end).
    fn place_var(v: &str) -> crate::os_paper::Place {
        let (d, a) = v.split_once(',').unwrap_or(("0", "0"));
        crate::os_paper::Place::new(d.trim().parse().unwrap_or(0), a.trim().parse().unwrap_or(0))
    }

    /// The sample bill both manual checks print.
    fn check_bill(mm: u32, lines: usize) -> (String, PrintFormat) {
        let mut text = String::from("CASH MEMO\nBill No.: 1\n");
        for i in 0..lines {
            text.push_str(&format!("Item number {}\t1\t80.00\t80.00\n", i + 1));
        }
        text.push_str("Grand Total: 640.00\nThanks\n");
        let mut f = PrintFormat::standard_portrait();
        f.line = wrap_cols_for_paper_mm(mm);
        f.content_width_mm = mm;
        (text, f)
    }

    /// The format of the sample bill, as the check's settings say:
    /// `MOKLANJO_BILL_MM` wide, `MOKLANJO_LANDSCAPE=1`, `MOKLANJO_PLACE`.
    fn check_format(var: &dyn Fn(&str) -> String) -> (String, PrintFormat) {
        let (text, mut f) = check_bill(
            var("MOKLANJO_BILL_MM").parse().unwrap_or(80),
            var("MOKLANJO_LINES").parse().unwrap_or(8),
        );
        f.landscape = var("MOKLANJO_LANDSCAPE") == "1";
        f.place = place_var(&var("MOKLANJO_PLACE"));
        (text, f)
    }

    /// The virtual paper of the check: `MOKLANJO_PAPER_MM`, or none set.
    fn check_virtual(var: &dyn Fn(&str) -> String) -> Option<crate::os_paper::PaperMm> {
        Some(var("MOKLANJO_PAPER_MM")).filter(|v| !v.trim().is_empty()).map(|v| paper_mm_var(&v))
    }

    /// What a check writes beside its print: the slip, the app's own preview
    /// of the job on `own` (the page it printed on) and the numbers.
    fn write_check_preview(
        out: &std::path::Path,
        slip: &image::GrayImage,
        content_mm: f64,
        own: &crate::os_paper::Paper,
        f: &PrintFormat,
        virtual_paper: Option<crate::os_paper::PaperMm>,
        more: &str,
    ) {
        let lay = slip_lay(f);
        let virtual_mm = virtual_paper.unwrap_or_else(|| own.as_virtual());
        let sheet = crate::os_paper::sheet_for(virtual_mm, Some(own), slip_mm(slip, content_mm), lay.landscape);
        let (pages, ppm) = crate::os_paper::os_document_sheets(slip, content_mm, &sheet, lay);
        let preview = crate::os_paper::preview_picture(slip, 1, content_mm, Some(&sheet), 0.0, None, lay);
        slip.save(out.join("slip.png")).unwrap();
        preview.gray.save(out.join("preview.png")).unwrap();
        // Each page's picture as the print lays it at the printable corner.
        for (i, page) in pages.iter().enumerate() {
            page.save(out.join(format!("page-{}.png", i + 1))).unwrap();
        }
        std::fs::write(
            out.join("info.txt"),
            format!(
                "own_page={} {:.2}x{:.2} mm margins={:?}\nvirtual={}x{} mm\narea={:.2}x{:.2} mm\n\
                 place={:?} landscape={}\ncontent_mm={content_mm}\npx_per_mm={ppm}\npreview_px_per_mm={}\npages={}\n{more}",
                own.label,
                own.width_mm,
                own.height_mm,
                own.margins_mm,
                virtual_mm.width_mm,
                virtual_mm.length_mm,
                sheet.area_mm.0,
                sheet.area_mm.1,
                f.place,
                f.landscape,
                ppm / preview.per_px,
                pages.len(),
            ),
        )
        .unwrap();
    }

    /// Manual check, never run by `cargo test`: the PDF the app hands CUPS
    /// for a sample bill — on a driver file's own paper (its default) and a
    /// virtual paper — with its `lp` options and the app's preview of the
    /// same job, written to a folder. Nothing is printed; `cupsfilter` on
    /// the PDF shows where the printing system puts it.
    /// `MOKLANJO_PPD=<file.ppd> MOKLANJO_PAPER_MM=<w>x<l> or <w> (the virtual
    /// paper; blank = none) MOKLANJO_PLACE=<down>,<across> MOKLANJO_BILL_MM=<mm>
    /// MOKLANJO_LINES=<n> MOKLANJO_LANDSCAPE=1 MOKLANJO_CHECK_OUT=<dir> cargo
    /// test -p kumkaj-moklanjo --lib write_os_document_pdf -- --ignored`
    #[test]
    #[ignore]
    fn write_os_document_pdf() {
        let var = |k: &str| std::env::var(k).unwrap_or_default();
        let ppd = std::fs::read_to_string(var("MOKLANJO_PPD")).expect("MOKLANJO_PPD");
        let out = std::path::PathBuf::from(var("MOKLANJO_CHECK_OUT"));
        std::fs::create_dir_all(&out).unwrap();
        let papers = crate::os_paper::parse_ppd(&ppd);
        let own = papers.default_paper().cloned().expect("the driver's own paper");
        let (text, f) = check_format(&var);
        let virtual_paper = check_virtual(&var);
        let (slip, content_mm) = os_document_slip(&text, &f).unwrap();
        std::fs::write(out.join("job.pdf"), os_document_pdf(&slip, content_mm, &own, &f, virtual_paper)).unwrap();
        let more = format!("lp_options={}\n", lp_pdf_options().join(" "));
        write_check_preview(&out, &slip, content_mm, &own, &f, virtual_paper, &more);
    }

    /// Manual check on Windows, never run by `cargo test`: a sample bill
    /// sent through the app's own Windows print path with the driver's
    /// output written to a FILE — only ever a file-only printer
    /// ("Microsoft Print to PDF", the default, or the XPS writer), never a
    /// real one — the app's preview of the same job on the page that
    /// printer reported, and what `print_win::papers` reads from its driver.
    /// `MOKLANJO_CHECK_OUT=<dir>` and the settings of
    /// [`write_os_document_pdf`]; `MOKLANJO_COPIES=<n>`.
    #[cfg(target_os = "windows")]
    #[test]
    #[ignore]
    fn windows_print_check() {
        let var = |k: &str| std::env::var(k).unwrap_or_default();
        let out = std::path::PathBuf::from(var("MOKLANJO_CHECK_OUT"));
        std::fs::create_dir_all(&out).unwrap();
        let printer = Some(var("MOKLANJO_WIN_PRINTER"))
            .filter(|p| !p.trim().is_empty())
            .unwrap_or("Microsoft Print to PDF".into());
        assert!(
            ["Microsoft Print to PDF", "Microsoft XPS Document Writer"].contains(&printer.as_str()),
            "a check prints to a file-only printer, never a real one"
        );
        let file = out.join(if printer.contains("XPS") { "job.oxps" } else { "job.pdf" });
        let read = match crate::print_win::papers(Some(&printer)) {
            Ok(p) => {
                let mut lines = format!("default\t{}\n", p.default_id);
                for x in &p.papers {
                    lines.push_str(&format!(
                        "{}\t{:.2}\t{:.2}\t{:?}\n",
                        x.id, x.width_mm, x.height_mm, x.margins_mm
                    ));
                }
                lines
            }
            Err(why) => format!("error\t{why}\n"),
        };
        std::fs::write(out.join("papers.txt"), read).unwrap();
        let (text, f) = check_format(&var);
        let virtual_paper = check_virtual(&var);
        let (slip, content_mm) = os_document_slip(&text, &f).unwrap();
        let copies: u32 = var("MOKLANJO_COPIES").parse().unwrap_or(1);
        let mut printed_on = None;
        let result = crate::print_win::print_pages(Some(&printer), "Moklanjo check", copies, Some(&file), |own| {
            printed_on = Some(own.clone());
            os_document_pages(&slip, content_mm, own, &f, virtual_paper)
        });
        if let Some(own) = &printed_on {
            write_check_preview(&out, &slip, content_mm, own, &f, virtual_paper, &format!("result={result:?}\n"));
        } else {
            std::fs::write(out.join("info.txt"), format!("result={result:?}\n")).unwrap();
        }
        std::fs::write(out.join("done.txt"), "done\n").unwrap();
    }

    /// The `GS v 0` bands of an ESC/POS job: width in bytes, all rows.
    fn gs_v0_rows(job: &[u8]) -> (usize, Vec<u8>) {
        let (mut at, mut width, mut rows) = (2, 0, Vec::new());
        while job.len() >= at + 8 && job[at..at + 4] == [0x1D, 0x76, 0x30, 0x00] {
            width = usize::from(job[at + 4]) | usize::from(job[at + 5]) << 8;
            let h = usize::from(job[at + 6]) | usize::from(job[at + 7]) << 8;
            rows.extend_from_slice(&job[at + 8..at + 8 + width * h]);
            at += 8 + width * h;
        }
        (width, rows)
    }

    /// A format's place is kept with it; top-left (where content always
    /// sat) is not written, so older files and builds read the same.
    #[test]
    fn a_formats_place_is_saved_and_top_left_is_not_written() {
        let mut f = PrintFormat::standard_portrait();
        let plain = serde_json::to_string(&f).unwrap();
        assert!(!plain.contains("place"), "{plain}");
        let old: PrintFormat = serde_json::from_str(&plain).unwrap();
        assert!(old.place.is_top_left());
        f.place = crate::os_paper::Place::new(1, 2);
        let back: PrintFormat = serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();
        assert_eq!(back.place, crate::os_paper::Place::new(1, 2));
    }

    /// ESC/POS: a roll is as long as the slip, so only across counts.
    /// Top-start, or a roll whose width is not known, sends the slip exactly
    /// as before; centred on a wider roll every dot moves to its middle.
    #[test]
    fn escpos_places_the_slip_across_a_wider_roll() {
        let mut f = PrintFormat::standard_portrait();
        f.document = false;
        f.content_width_mm = 58;
        f.line = wrap_cols_for_paper_mm(58);
        let text = "Tea\nCoffee\n";
        let roll = Some(crate::os_paper::PaperMm { width_mm: 80.0, length_mm: 0.0 });
        let plain = job_bytes_placed(&f, text, None);
        assert_eq!(job_bytes_placed(&f, text, roll), plain, "top-start: unchanged");
        f.place = crate::os_paper::Place::new(2, 1);
        assert_eq!(job_bytes_placed(&f, text, None), plain, "roll not known: unchanged");
        let (w0, rows0) = gs_v0_rows(&plain);
        let (w1, rows1) = gs_v0_rows(&job_bytes_placed(&f, text, roll));
        assert_eq!(w1, 576 / 8, "the 80 mm roll's width");
        let shift = (576 - w0 * 8) / 2;
        let dot = |rows: &[u8], w: usize, y: usize, d: usize| rows[y * w + d / 8] & (0x80 >> (d % 8)) != 0;
        let height = rows0.len() / w0;
        assert_eq!(rows1.len() / w1, height);
        let mut ink = 0;
        for y in 0..height {
            for d in 0..w0 * 8 {
                assert_eq!(dot(&rows1, w1, y, d + shift), dot(&rows0, w0, y, d), "row {y} dot {d}");
                ink += usize::from(dot(&rows0, w0, y, d));
            }
        }
        assert!(ink > 0, "the slip has ink");
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

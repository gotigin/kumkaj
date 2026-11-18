//! Printing (local free POS — no cloud), the same on every device that can
//! hold the books or take orders.
//!
//! Owner nouns (Settings → **Printers**):
//! - **Real printer** — one OS device from Find (destination only).
//! - **Config** — layout (portrait/landscape, metrics) + send path (OS
//!   document, ESC/POS, …). Not kitchen KOT content. Never bound to a device.
//! - **Virtual printer** — real printer + config.
//!
//! See `docs/PRINTER.md`. Place attach is separate. Legacy route slots still
//! drive runtime until attach UI lands.
//!
//! A printer `target` is a system name, `host:9100`, `bt:MAC`, `usb:VID:PID`,
//! or `android-print:`. Blank = this computer's usual OS printer — never
//! invent a pool entry named "Default". Jobs go through one worker thread;
//! failures surface via [`take_problems`].

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::Command;
use std::sync::{Mutex, mpsc};
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
    /// Bills, receipts, shift and day slips.
    Bill,
}

/// Which ordered route a job uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteKey {
    Bill,
}

/// Built-in format ids (always present; owner may customize their metrics).
/// Legacy `"standard"` migrates to portrait on load.
pub const FORMAT_STANDARD: &str = "standard";
pub const FORMAT_STANDARD_PORTRAIT: &str = "standard_portrait";
pub const FORMAT_STANDARD_LANDSCAPE: &str = "standard_landscape";
pub const FORMAT_KOT: &str = "kot";

/// Document (office) Standard formats — not thermal KOT.
pub fn is_builtin_format(id: &str) -> bool {
    matches!(
        id,
        FORMAT_KOT | FORMAT_STANDARD | FORMAT_STANDARD_PORTRAIT | FORMAT_STANDARD_LANDSCAPE
    )
}

pub fn is_standard_document(id: &str) -> bool {
    matches!(
        id,
        FORMAT_STANDARD | FORMAT_STANDARD_PORTRAIT | FORMAT_STANDARD_LANDSCAPE
    )
}

/// Map legacy `"standard"` → portrait.
pub fn normalize_format_id(id: &str) -> &str {
    if id == FORMAT_STANDARD {
        FORMAT_STANDARD_PORTRAIT
    } else {
        id
    }
}

/// One Output under a place: which printer + which profile (code name only;
/// Settings shows the two fields, not this word).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintOutput {
    pub printer_id: String,
    pub profile_id: String,
}

/// Named list of kitchen stations for bulk Output apply (setup only).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StationGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub station_ids: Vec<String>,
}

/// Known delivery method ids (open set — add more without a named Config catalog).
pub const DELIVERY_SYSTEM_SPOOLER: &str = "system_spooler";
pub const DELIVERY_RAW_DEVICE: &str = "raw_device";
pub const DELIVERY_ESCPOS: &str = "escpos";

/// Inline values on a virtual printer (layout + delivery + content width).
/// Not a separate Config — see `docs/PRINTER.md`.
#[derive(Clone, Debug, PartialEq)]
pub struct VirtualPrinterValues {
    pub landscape: bool,
    pub em: f32,
    pub line: u32,
    pub side: f32,
    /// Delivery method id — see `DELIVERY_*` (extensible string, not a closed enum).
    pub delivery: String,
    /// Content width in millimetres (stored as `paper_width_mm`).
    /// Legacy VP field — product bill/KOT widths live on the slip formats.
    pub paper_width_mm: u32,
}

impl Default for VirtualPrinterValues {
    fn default() -> Self {
        Self {
            landscape: false,
            em: STD_EM_NORMAL,
            line: 40,
            side: STD_SIDE,
            delivery: DELIVERY_SYSTEM_SPOOLER.into(),
            paper_width_mm: 80,
        }
    }
}

impl VirtualPrinterValues {
    pub fn document_flag(&self) -> bool {
        self.delivery.trim().is_empty() || self.delivery == DELIVERY_SYSTEM_SPOOLER
    }

    pub fn normalize(mut self) -> Self {
        let d = self.delivery.trim();
        self.delivery = if d.is_empty() {
            DELIVERY_SYSTEM_SPOOLER.into()
        } else {
            d.to_string()
        };
        // Free sizes — clamp only. Do not snap to Small/Normal/Large buckets.
        self.em = self.em.clamp(12.0, 48.0);
        self.paper_width_mm = self.paper_width_mm.clamp(40, 210);
        self.line = wrap_cols_for_paper_mm(self.paper_width_mm);
        self.side = self.side.clamp(0.0, 48.0);
        self
    }
}

/// Character columns for a content width in millimetres (owner mm on the VP).
pub fn wrap_cols_for_paper_mm(mm: u32) -> u32 {
    let mm = mm.clamp(40, 210);
    // ~0.55 cols/mm at Font A–ish density; 58→32, 80→44-ish.
    let cols = ((mm as f32) * 0.55).round() as u32;
    cols.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX)
}

/// Dots across the content for a millimetre content width.
/// 58 mm is 384 dots; 80 mm is 576. Wider content scales from 80 mm.
pub fn paper_width_dots(mm: u32) -> usize {
    let mm = mm.clamp(40, 210);
    let dots = if mm <= 58 {
        384
    } else {
        (mm as usize * 576) / 80
    };
    dots.max(8) & !7
}

/// Page dots for a wrap-column count produced by [`wrap_cols_for_paper_mm`].
pub fn paper_dots_for_wrap_cols(cols: u32) -> usize {
    let cols = cols.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX);
    let mm = ((cols as f32) / 0.55).round() as u32;
    paper_width_dots(mm.clamp(40, 210))
}

/// Wrap columns for the first printer on this route, or 80 mm when none is set.
pub fn route_wrap_cols(key: RouteKey) -> u32 {
    printers()
        .destinations_for(key)
        .first()
        .map(|d| d.format.line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX))
        .unwrap_or_else(|| wrap_cols_for_paper_mm(80))
}

/// Virtual printer: real printer + **inline** configuration values.
/// There is no separate named Config object — see `docs/PRINTER.md`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualPrinter {
    pub id: String,
    /// Owner-facing name (defaults from the real printer label).
    pub name: String,
    /// [`NamedPrinter::id`].
    pub printer_id: String,
    /// Layout: landscape when true, else portrait.
    #[serde(default)]
    pub landscape: bool,
    #[serde(default = "default_em")]
    pub em: f32,
    #[serde(default = "default_line")]
    pub line: u32,
    #[serde(default = "default_side")]
    pub side: f32,
    /// Delivery method id (`DELIVERY_*` or future ids).
    #[serde(default = "default_delivery")]
    pub delivery: String,
    /// Content width in mm (owner-set on this VP). Stored key stays
    /// `paper_width_mm` so existing printer files keep loading.
    #[serde(default = "default_paper_width_mm")]
    pub paper_width_mm: u32,
    /// Derived for the current worker from [`Self::delivery`].
    #[serde(default = "default_true")]
    pub document: bool,
    /// Legacy only (old files that pointed at a format id). Prefer inline fields.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub config_id: String,
}

fn default_paper_width_mm() -> u32 {
    80
}

fn default_delivery() -> String {
    DELIVERY_SYSTEM_SPOOLER.into()
}

/// Route slot / Output encode: `printer_id` or `printer_id@profile_id`.
fn encode_route_slot(printer_id: &str, format_id: &str) -> String {
    let p = printer_id.trim();
    let f = format_id.trim();
    if p.is_empty() {
        return String::new();
    }
    if f.is_empty() {
        p.to_string()
    } else {
        format!("{p}@{f}")
    }
}

fn decode_route_slot(slot: &str) -> (String, String) {
    let s = slot.trim();
    if let Some((p, f)) = s.rsplit_once('@') {
        if !p.is_empty() && !f.is_empty() {
            return (p.to_string(), f.to_string());
        }
    }
    (s.to_string(), String::new())
}

/// Provisional worker delivery flags used by today’s print path.
/// **Not** the product model — Settings must not present these as the only
/// choices. See `docs/PRINTER.md`: delivery knobs on a virtual printer are an
/// open set as we add capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrintSendPath {
    /// System spooler page path (`lp` / Out-Printer / PrintManager).
    #[default]
    OsDocument,
    /// Raw ESC/POS when the current worker uses that path.
    EscPos,
}

impl PrintSendPath {
    pub fn from_document_flag(document: bool) -> Self {
        if document {
            Self::OsDocument
        } else {
            Self::EscPos
        }
    }

    pub fn is_document(self) -> bool {
        matches!(self, Self::OsDocument)
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::OsDocument => "config_send_os",
            Self::EscPos => "config_send_escpos",
        }
    }
}

impl VirtualPrinter {
    pub fn values(&self) -> VirtualPrinterValues {
        let paper = if self.paper_width_mm == 0 {
            // Legacy rows: infer mm from stored wrap columns.
            if self.line <= 34 {
                58
            } else {
                80
            }
        } else {
            self.paper_width_mm
        };
        VirtualPrinterValues {
            landscape: self.landscape,
            em: self.em,
            line: self.line,
            side: self.side,
            delivery: if self.delivery.trim().is_empty() {
                if self.document {
                    DELIVERY_SYSTEM_SPOOLER.into()
                } else {
                    DELIVERY_ESCPOS.into()
                }
            } else {
                self.delivery.clone()
            },
            paper_width_mm: paper,
        }
        .normalize()
    }

    pub fn apply_values(&mut self, values: VirtualPrinterValues) {
        let v = values.normalize();
        self.landscape = v.landscape;
        self.em = v.em;
        self.line = v.line;
        self.side = v.side;
        self.delivery = v.delivery.clone();
        self.paper_width_mm = v.paper_width_mm;
        self.document = v.document_flag();
    }

    pub fn send_path(&self) -> PrintSendPath {
        PrintSendPath::from_document_flag(self.values().document_flag())
    }
}

/// One print config: layout + send path (not kitchen KOT content).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrintFormat {
    pub id: String,
    pub label: String,
    /// When true → [`PrintSendPath::OsDocument`]; false → ESC/POS raw.
    /// Kept for serde; prefer [`PrintFormat::send_path`].
    #[serde(default = "default_true")]
    pub document: bool,
    /// Page / slip orientation (layout — independent of send path).
    #[serde(default)]
    pub landscape: bool,
    /// How this job is sent: `system_spooler`, `raw_device`, or `escpos`.
    /// Empty follows [`Self::document`] (older files).
    #[serde(default)]
    pub delivery: String,
    #[serde(default = "default_em")]
    pub em: f32,
    #[serde(default = "default_line")]
    pub line: u32,
    #[serde(default = "default_baseline")]
    pub baseline: f32,
    #[serde(default = "default_side")]
    pub side: f32,
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

/// Bill / Standard document type-size presets (Settings chips).
/// Product built-ins are **generic** (any OS queue). Narrow-tray shops
/// (e.g. some inkjets) tune side/em on a Profile —
/// never assume one owner's Canon in product defaults. OS jobs still
/// fit-to-page so PNGs are not printed ~16″ wide at 72 DPI.
pub const STD_EM_SMALL: f32 = 18.0;
pub const STD_EM_NORMAL: f32 = 20.0;
pub const STD_EM_LARGE: f32 = 26.0;
/// Generic side inset (dots on the 384-wide raster) for Standard formats.
pub const STD_SIDE: f32 = 16.0;

impl PrintFormat {
    /// Alias for [`standard_portrait`] (legacy API name).
    pub fn standard() -> Self {
        Self::standard_portrait()
    }

    pub fn send_path(&self) -> PrintSendPath {
        PrintSendPath::from_document_flag(self.document)
    }

    pub fn set_send_path(&mut self, path: PrintSendPath) {
        self.document = path.is_document();
    }

    /// Delivery id for this job. Bill and KOT each keep their own.
    pub fn delivery_id(&self) -> &str {
        if !self.delivery.trim().is_empty() {
            return self.delivery.trim();
        }
        if self.document {
            DELIVERY_SYSTEM_SPOOLER
        } else {
            DELIVERY_ESCPOS
        }
    }

    /// Built-in: portrait layout + OS document send (works on most queues).
    /// Default wrap matches a 210 mm page bill.
    pub fn standard_portrait() -> Self {
        Self {
            id: FORMAT_STANDARD_PORTRAIT.into(),
            label: "Portrait · OS document".into(),
            document: true,
            landscape: false,
            delivery: DELIVERY_SYSTEM_SPOOLER.into(),
            em: STD_EM_NORMAL,
            line: wrap_cols_for_paper_mm(210),
            baseline: STD_EM_NORMAL + 2.0,
            side: STD_SIDE,
            place: Default::default(),
        }
    }

    /// Built-in: landscape layout + OS document send.
    /// Default wrap matches a 210 mm page bill.
    pub fn standard_landscape() -> Self {
        Self {
            id: FORMAT_STANDARD_LANDSCAPE.into(),
            label: "Landscape · OS document".into(),
            document: true,
            landscape: true,
            delivery: DELIVERY_SYSTEM_SPOOLER.into(),
            em: STD_EM_NORMAL,
            line: wrap_cols_for_paper_mm(210),
            baseline: STD_EM_NORMAL + 2.0,
            side: STD_SIDE,
            place: Default::default(),
        }
    }

    /// Built-in: portrait layout + ESC/POS send (id kept as `kot` for serde).
    /// Default wrap matches an 80 mm kitchen slip; type size matches FOH KOTs.
    pub fn kot() -> Self {
        Self {
            id: FORMAT_KOT.into(),
            label: "Portrait · ESC/POS".into(),
            document: false,
            landscape: false,
            delivery: DELIVERY_ESCPOS.into(),
            em: KOT_EM_LARGE as f32,
            line: wrap_cols_for_paper_mm(80),
            baseline: (KOT_EM_LARGE as f32) + 2.0,
            side: 8.0,
            place: Default::default(),
        }
    }

    /// The print picture's metrics. `line` on a format is wrap columns (it
    /// sets the width); the row pitch comes from the type size, the same as
    /// the preview ([`metrics_at_content_mm`]). Using the column count as the
    /// pitch spread a plain 80 mm slip to 1.5× line spacing, and a page to
    /// bands of white, so paper and preview disagreed.
    pub fn raster_metrics(&self) -> crate::receipt_raster::RasterMetrics {
        crate::receipt_raster::RasterMetrics {
            em: self.em,
            line: raster_pitch_for_em(self.em),
            baseline: self.baseline,
            side: self.side,
            width_dots: paper_dots_for_wrap_cols(self.line),
        }
    }

    /// Built-ins: portrait/landscape OS document + ESC/POS. Legacy `"standard"` → portrait.
    pub fn ensure_builtins(list: &mut Vec<PrintFormat>) {
        for f in list.iter_mut() {
            if f.id == FORMAT_STANDARD {
                f.id = FORMAT_STANDARD_PORTRAIT.into();
                f.document = true;
                f.landscape = false;
            }
            // Refresh stock labels (old "KOT" / "Standard Portrait" names).
            if is_builtin_format(&f.id) {
                let stock = match f.id.as_str() {
                    FORMAT_STANDARD_PORTRAIT | FORMAT_STANDARD => PrintFormat::standard_portrait(),
                    FORMAT_STANDARD_LANDSCAPE => PrintFormat::standard_landscape(),
                    FORMAT_KOT => PrintFormat::kot(),
                    _ => continue,
                };
                let old = f.label.trim();
                if old.eq_ignore_ascii_case("kot")
                    || old.eq_ignore_ascii_case("standard")
                    || old.eq_ignore_ascii_case("standard portrait")
                    || old.eq_ignore_ascii_case("standard landscape")
                    || old.is_empty()
                {
                    f.label = stock.label;
                }
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

/// Font weight for one KOT slip field (Bill & KOT format).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KotFontWeight {
    Light,
    #[default]
    Regular,
    Bold,
}

impl KotFontWeight {
    pub fn locale_key(self) -> &'static str {
        match self {
            KotFontWeight::Light => "kot_weight_light",
            KotFontWeight::Regular => "kot_weight_regular",
            KotFontWeight::Bold => "kot_weight_bold",
        }
    }
}

/// Where a field sits on the line. Start and end follow the writing
/// direction (left/right in English, the other way in Arabic or Urdu).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KotAlign {
    #[default]
    Start,
    Center,
    End,
}

impl KotAlign {
    pub const ALL: [KotAlign; 3] = [KotAlign::Start, KotAlign::Center, KotAlign::End];

    pub fn locale_key(self) -> &'static str {
        match self {
            KotAlign::Start => "kot_align_start",
            KotAlign::Center => "kot_align_center",
            KotAlign::End => "kot_align_end",
        }
    }

    /// Letter stored on the print line. `rtl` swaps start and end so the
    /// raster, which places dots from the left, still honours the direction.
    fn mark(self, rtl: bool) -> char {
        match self {
            KotAlign::Center => 'C',
            KotAlign::Start if rtl => 'E',
            KotAlign::End if rtl => 'S',
            KotAlign::Start => 'S',
            KotAlign::End => 'E',
        }
    }

    fn from_mark(c: char) -> Self {
        match c {
            'C' => KotAlign::Center,
            'E' => KotAlign::End,
            _ => KotAlign::Start,
        }
    }

    fn line_align(self) -> crate::receipt_raster::LineAlign {
        match self {
            KotAlign::Start => crate::receipt_raster::LineAlign::Start,
            KotAlign::Center => crate::receipt_raster::LineAlign::Center,
            KotAlign::End => crate::receipt_raster::LineAlign::End,
        }
    }
}

/// One item column (docs/PRINTER.md "Item columns"): a border box
/// `width_mm` wide with `start_px` / `end_px` of space inside it, its text
/// aligned start / center / end — all following the writing direction.
/// Columns sit side by side from the start edge; past the content width
/// they are cut.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlipColumn {
    pub width_mm: f32,
    #[serde(default = "default_slip_col_space")]
    pub start_px: u32,
    #[serde(default = "default_slip_col_space")]
    pub end_px: u32,
    #[serde(default)]
    pub align: KotAlign,
}

pub const SLIP_COL_WIDTH_MIN_MM: f32 = 1.0;
pub const SLIP_COL_WIDTH_MAX_MM: f32 = 210.0;
pub const SLIP_COL_SPACE_MAX_PX: u32 = 200;

fn default_slip_col_space() -> u32 {
    4
}

impl SlipColumn {
    pub fn new(width_mm: f32, align: KotAlign) -> Self {
        Self {
            width_mm,
            start_px: default_slip_col_space(),
            end_px: default_slip_col_space(),
            align,
        }
        .clamped()
    }

    pub fn clamped(mut self) -> Self {
        let w = if self.width_mm.is_finite() { self.width_mm } else { SLIP_COL_WIDTH_MIN_MM };
        self.width_mm = ((w * 10.0).round() / 10.0).clamp(SLIP_COL_WIDTH_MIN_MM, SLIP_COL_WIDTH_MAX_MM);
        self.start_px = self.start_px.min(SLIP_COL_SPACE_MAX_PX);
        self.end_px = self.end_px.min(SLIP_COL_SPACE_MAX_PX);
        self
    }

    /// This column in dots on a slip `content_mm` wide (the print's own dots
    /// per mm — the same picture the preview shows).
    fn column_box(&self, content_mm: u32) -> crate::receipt_raster::ColumnBox {
        crate::receipt_raster::ColumnBox {
            width: self.width_mm * content_dots_per_mm(content_mm),
            start: crate::receipt_raster::px_dots(self.start_px as f32),
            end: crate::receipt_raster::px_dots(self.end_px as f32),
            align: self.align.line_align(),
        }
    }
}

/// Dots per millimetre for a column on a `content_mm` slip: the picture's
/// own 7.2 per mm, so a 30 mm column is 30 mm at every content width. A
/// slip of 384 dots or fewer (58 mm and narrower) is the 58 mm head's
/// picture whatever its mm, and keeps its columns spread over that mm
/// (docs/PRINTER.md lesson 22).
pub fn content_dots_per_mm(content_mm: u32) -> f32 {
    let mm = content_mm.clamp(40, 210);
    let dots = paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(mm));
    if dots <= 384 {
        dots as f32 / mm as f32
    } else {
        crate::receipt_raster::DOTS_PER_MM
    }
}

/// Millimetres inside the side margins of a `content_mm` slip, for columns
/// that start out filling it.
fn fill_room_mm(content_mm: u32, side_px: f32) -> f32 {
    let mm = content_mm.clamp(40, 210);
    // The picture's own dots inside its two side margins (dots).
    let dots = paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(mm)) as f32;
    ((dots - 2.0 * side_px.max(0.0)) / content_dots_per_mm(mm)).max(4.0)
}

/// Shares of the room, rounded down to half millimetres.
fn fill_shares(room_mm: f32, shares: &[f32]) -> Vec<f32> {
    shares.iter().map(|s| ((room_mm * s) * 2.0).floor() / 2.0).collect()
}

/// The bill's item columns (Item, Qty, Price, Amount), each the owner's.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BillItemColumns {
    pub name: SlipColumn,
    pub qty: SlipColumn,
    pub price: SlipColumn,
    pub amount: SlipColumn,
}

impl BillItemColumns {
    /// Columns that fill a `content_mm` bill inside its margins — where a new
    /// format, or one saved before columns existed, starts.
    pub fn filling(content_mm: u32) -> Self {
        let w = fill_shares(fill_room_mm(content_mm, STD_SIDE), &[0.46, 0.14, 0.19, 0.21]);
        Self {
            name: SlipColumn::new(w[0], KotAlign::Start),
            qty: SlipColumn::new(w[1], KotAlign::End),
            price: SlipColumn::new(w[2], KotAlign::End),
            amount: SlipColumn::new(w[3], KotAlign::End),
        }
    }

    pub fn get(&self, col: BillItemCol) -> &SlipColumn {
        match col {
            BillItemCol::Name => &self.name,
            BillItemCol::Qty => &self.qty,
            BillItemCol::Price => &self.price,
            BillItemCol::Amount => &self.amount,
        }
    }

    pub fn get_mut(&mut self, col: BillItemCol) -> &mut SlipColumn {
        match col {
            BillItemCol::Name => &mut self.name,
            BillItemCol::Qty => &mut self.qty,
            BillItemCol::Price => &mut self.price,
            BillItemCol::Amount => &mut self.amount,
        }
    }

    fn clamped(mut self) -> Self {
        for col in BillItemCol::ALL {
            let c = self.get_mut(col);
            *c = c.clamped();
        }
        self
    }
}


/// Type size, weight, and alignment for one field on one KOT kind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KotFieldStyle {
    /// Thermal / raster em (free within [`KOT_EM_MIN`]..=[`KOT_EM_MAX`]).
    #[serde(default = "default_kot_em")]
    pub size_em: u32,
    #[serde(default)]
    pub weight: KotFontWeight,
    #[serde(default)]
    pub align: KotAlign,
}

impl Default for KotFieldStyle {
    fn default() -> Self {
        Self {
            size_em: KOT_EM_NORMAL,
            weight: KotFontWeight::Regular,
            align: KotAlign::Start,
        }
    }
}

impl KotFieldStyle {
    pub fn light(size_em: u32) -> Self {
        Self {
            size_em,
            weight: KotFontWeight::Light,
            align: KotAlign::Start,
        }
    }
    pub fn regular(size_em: u32) -> Self {
        Self {
            size_em,
            weight: KotFontWeight::Regular,
            align: KotAlign::Start,
        }
    }
    pub fn bold(size_em: u32) -> Self {
        Self {
            size_em,
            weight: KotFontWeight::Bold,
            align: KotAlign::Start,
        }
    }
    pub fn regular_center(size_em: u32) -> Self {
        Self {
            size_em,
            weight: KotFontWeight::Regular,
            align: KotAlign::Center,
        }
    }
    pub fn bold_center(size_em: u32) -> Self {
        Self {
            size_em,
            weight: KotFontWeight::Bold,
            align: KotAlign::Center,
        }
    }
}

/// Shared slip size limits (bill and printer formats).
pub const KOT_CHARS_MIN: u32 = 16;
/// Wide enough for a 210 mm page (~0.55 cols/mm). 58 mm is 32, 80 mm is 44.
pub const KOT_CHARS_MAX: u32 = 128;
/// Sizes are literal px (dots), free from 1 to 200 — a typo guard, not a
/// design limit (Khaynjo owner, 2026-11-15). It was 12–48, a limit nobody chose.
pub const KOT_EM_MIN: u32 = 1;
pub const KOT_EM_MAX: u32 = 200;
pub const KOT_EM_SMALL: u32 = 20;
pub const KOT_EM_NORMAL: u32 = 24;
pub const KOT_EM_LARGE: u32 = 28;
fn default_kot_em() -> u32 { KOT_EM_NORMAL }

/// Private-use prefix on each rendered KOT line: size + weight for ESC/POS.
/// Preview / raster strip these via [`strip_kot_style_marks`].
const KOT_STYLE_MARK: char = '\u{E000}';
/// Document-level `\t` column metrics. One line at the top of a slip:
/// `\u{E001}T{gap}|{min0}|{min1}|{min2}|{min3}|{S|C|E}`. Survives the print
/// queue (text-only jobs) so bill/KOT column air is not lost after format.
const SLIP_TAB_MARK: char = '\u{E001}';

fn kot_style_prefix(style: &KotFieldStyle, rtl: bool) -> String {
    kot_style_prefix_strike(style, rtl, false)
}

fn kot_style_prefix_strike(style: &KotFieldStyle, rtl: bool, strike: bool) -> String {
    let size = style.size_em.clamp(KOT_EM_MIN, KOT_EM_MAX);
    // Weight letters must not collide with a digit (T = thin/light).
    let weight = match style.weight {
        KotFontWeight::Bold => 'B',
        KotFontWeight::Light => 'T',
        KotFontWeight::Regular => 'R',
    };
    // Two digits so 16 and 24 stay different sizes on the print, not one bucket.
    // Align is S / C / E after the weight. Strike is always a trailing `0`/`1`
    // so a body that starts with `K` (`KOT - 12`) is never eaten as a flag.
    // `rtl` swaps start and end.
    format!(
        "{KOT_STYLE_MARK}{size:02}{weight}{}{}",
        style.align.mark(rtl),
        if strike { '1' } else { '0' }
    )
}

/// A line on the bill. Same ink controls as a kitchen field: show, size, weight, align.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BillStyleField {
    Shop,
    Header,
    Fssai,
    Title,
    Customer,
    BillNo,
    Time,
    Cashier,
    Items,
    Total,
    Footer,
    Upi,
}

impl BillStyleField {
    pub const ALL: [BillStyleField; 12] = [
        BillStyleField::Shop,
        BillStyleField::Header,
        BillStyleField::Fssai,
        BillStyleField::Title,
        BillStyleField::Customer,
        BillStyleField::BillNo,
        BillStyleField::Time,
        BillStyleField::Cashier,
        BillStyleField::Items,
        BillStyleField::Total,
        BillStyleField::Footer,
        BillStyleField::Upi,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BillStyleField::Shop => "Shop",
            BillStyleField::Header => "Address",
            BillStyleField::Fssai => "FSSAI",
            BillStyleField::Title => "Title",
            BillStyleField::Customer => "Customer",
            BillStyleField::BillNo => "Bill no",
            BillStyleField::Time => "Time",
            BillStyleField::Cashier => "Cashier",
            BillStyleField::Items => "Items",
            BillStyleField::Total => "Total",
            BillStyleField::Footer => "Footer",
            BillStyleField::Upi => "UPI",
        }
    }

    /// The line's name in the owner's language: its `locale` key.
    pub fn label_key(self) -> &'static str {
        match self {
            BillStyleField::Shop => "bill_line_shop",
            BillStyleField::Header => "bill_line_address",
            BillStyleField::Fssai => "bill_line_fssai",
            BillStyleField::Title => "bill_line_title",
            BillStyleField::Customer => "bill_line_customer",
            BillStyleField::BillNo => "bill_line_bill_no",
            BillStyleField::Time => "bill_line_time",
            BillStyleField::Cashier => "bill_line_cashier",
            BillStyleField::Items => "bill_line_items",
            BillStyleField::Total => "bill_line_total",
            BillStyleField::Footer => "bill_line_footer",
            BillStyleField::Upi => "bill_line_upi",
        }
    }

    /// The owner types this field. The others are filled from the sale.
    pub fn has_text(self) -> bool {
        matches!(
            self,
            BillStyleField::Header
                | BillStyleField::Fssai
                | BillStyleField::Title
                | BillStyleField::Footer
                | BillStyleField::Upi
        )
    }

    /// Meta fields that may share a line as left\\tright (Cashier | Bill no, …).
    pub fn is_pairable(self) -> bool {
        matches!(
            self,
            BillStyleField::Customer
                | BillStyleField::BillNo
                | BillStyleField::Time
                | BillStyleField::Cashier
        )
    }

    pub fn id(self) -> &'static str {
        match self {
            BillStyleField::Shop => "shop",
            BillStyleField::Header => "header",
            BillStyleField::Fssai => "fssai",
            BillStyleField::Title => "title",
            BillStyleField::Customer => "customer",
            BillStyleField::BillNo => "bill_no",
            BillStyleField::Time => "time",
            BillStyleField::Cashier => "cashier",
            BillStyleField::Items => "items",
            BillStyleField::Total => "total",
            BillStyleField::Footer => "footer",
            BillStyleField::Upi => "upi",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.id() == id)
    }

    pub fn value_hint(self) -> &'static str {
        match self {
            BillStyleField::Header => "Address | phone",
            BillStyleField::Fssai => "FSSAI licence no.",
            BillStyleField::Title => "Title (blank = CASH MEMO)",
            BillStyleField::Footer => "Footer (e.g. Thank you, visit again)",
            BillStyleField::Upi => "UPI ID | payee name",
            _ => "",
        }
    }
}

/// How each dish prints when Items is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BillItemLayout {
    /// Name / qty×price / amount each on its own line.
    Stacked,
    /// One table row: Item | Qty | Price | Amount (shown columns only).
    #[default]
    Columns,
}

/// One piece column on a Columns layout bill.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BillItemCol {
    Name,
    Qty,
    Price,
    Amount,
}

impl BillItemCol {
    pub const ALL: [BillItemCol; 4] = [
        BillItemCol::Name,
        BillItemCol::Qty,
        BillItemCol::Price,
        BillItemCol::Amount,
    ];

    pub fn id(self) -> &'static str {
        match self {
            BillItemCol::Name => "name",
            BillItemCol::Qty => "qty",
            BillItemCol::Price => "price",
            BillItemCol::Amount => "amount",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim() {
            "name" | "item" => Some(BillItemCol::Name),
            "qty" | "quantity" => Some(BillItemCol::Qty),
            "price" | "rate" => Some(BillItemCol::Price),
            "amount" | "amt" | "total" => Some(BillItemCol::Amount),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            BillItemCol::Name => "Dish name",
            BillItemCol::Qty => "Qty",
            BillItemCol::Price => "Price",
            BillItemCol::Amount => "Amount",
        }
    }
}

/// One of the bill's three totals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BillTotalKind {
    Qty,
    Subtotal,
    Grand,
}

impl BillTotalKind {
    pub const ALL: [BillTotalKind; 3] = [BillTotalKind::Qty, BillTotalKind::Subtotal, BillTotalKind::Grand];
}

/// Where a total's caption goes while its number sits under an item column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillTotalCaption {
    /// Before the number, in the number's own place (`Total Qty: 3`).
    WithNumber,
    /// Not printed: the number alone.
    Hidden,
    /// Under another item column, on the number's row.
    Under(BillItemCol),
}

/// Where one total prints. Kept as plain ids (like the column order), so a
/// format from a newer build still reads: an id this build does not know
/// is the total on its own line.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillTotalPlace {
    /// The item column the number sits under (`name`, `qty`, `price`,
    /// `amount`); empty = the total has its own line, as before.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub number: String,
    /// Where the caption goes while the number sits under a column: empty =
    /// before the number, `none` = not printed, or an item column's id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub caption: String,
}

impl BillTotalPlace {
    /// Nothing set: the total on its own line with its caption.
    pub fn is_default(&self) -> bool {
        self.number.is_empty() && self.caption.is_empty()
    }

    pub fn number_col(&self) -> Option<BillItemCol> {
        BillItemCol::from_id(&self.number)
    }

    pub fn set_number_col(&mut self, col: Option<BillItemCol>) {
        self.number = col.map(|c| c.id().to_string()).unwrap_or_default();
    }

    pub fn caption_place(&self) -> BillTotalCaption {
        match self.caption.trim() {
            "none" => BillTotalCaption::Hidden,
            id => BillItemCol::from_id(id).map_or(BillTotalCaption::WithNumber, BillTotalCaption::Under),
        }
    }

    pub fn set_caption_place(&mut self, place: BillTotalCaption) {
        self.caption = match place {
            BillTotalCaption::WithNumber => String::new(),
            BillTotalCaption::Hidden => "none".into(),
            BillTotalCaption::Under(col) => col.id().to_string(),
        };
    }

    fn clamped(mut self) -> Self {
        let number = self.number_col();
        self.set_number_col(number);
        let caption = self.caption_place();
        self.set_caption_place(caption);
        self
    }
}

/// Show, size, weight, and alignment for one bill line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BillFieldStyle {
    #[serde(default = "default_true")]
    pub show: bool,
    #[serde(default = "default_bill_em")]
    pub size_em: u32,
    #[serde(default)]
    pub weight: KotFontWeight,
    #[serde(default)]
    pub align: KotAlign,
}

fn default_bill_em() -> u32 {
    16
}

impl BillFieldStyle {
    fn face(size_em: u32, weight: KotFontWeight, align: KotAlign) -> Self {
        Self {
            show: true,
            size_em,
            weight,
            align,
        }
    }
}

impl Default for BillFieldStyle {
    fn default() -> Self {
        Self::face(16, KotFontWeight::Regular, KotAlign::Start)
    }
}

/// What the bill prints, and how each line looks. One editor, same as the KOT.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BillSlipFormat {
    #[serde(default)]
    pub header: String,
    #[serde(default)]
    pub fssai: String,
    #[serde(default)]
    pub footer: String,
    #[serde(default)]
    pub upi_vpa: String,
    #[serde(default)]
    pub upi_payee: String,
    /// Blank prints CASH MEMO (or TAX INVOICE on a declared bill).
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub shop: BillFieldStyle,
    #[serde(default)]
    pub header_style: BillFieldStyle,
    #[serde(default)]
    pub fssai_style: BillFieldStyle,
    #[serde(default)]
    pub title_style: BillFieldStyle,
    #[serde(default)]
    pub customer: BillFieldStyle,
    #[serde(default)]
    pub bill_no: BillFieldStyle,
    #[serde(default)]
    pub time: BillFieldStyle,
    #[serde(default)]
    pub cashier: BillFieldStyle,
    #[serde(default)]
    pub items: BillFieldStyle,
    #[serde(default)]
    pub total: BillFieldStyle,
    #[serde(default)]
    pub footer_style: BillFieldStyle,
    #[serde(default)]
    pub upi: BillFieldStyle,
    /// Columns (Item|Qty|Price|Amount) or stacked lines per piece.
    #[serde(default)]
    pub item_layout: BillItemLayout,
    /// Print the dish name (column or own line).
    #[serde(default = "default_true")]
    pub item_show_name: bool,
    /// Print the quantity column / piece.
    #[serde(default = "default_true")]
    pub item_show_qty: bool,
    /// Print the unit price column / piece.
    #[serde(default = "default_true")]
    pub item_show_price: bool,
    /// Print the line amount (qty × price) column / piece.
    #[serde(default = "default_true")]
    pub item_show_amount: bool,
    /// Print the dish note under the item.
    #[serde(default = "default_true")]
    pub item_show_note: bool,
    /// Column header row (`Item`, `Qty.`, `Price`, `Amount`) when layout is Columns.
    #[serde(default = "default_true")]
    pub item_show_header: bool,
    /// Header text for the name column. Empty → `Item`.
    #[serde(default = "default_bill_header_item")]
    pub item_header_item: String,
    /// Header text for the qty column. Empty → `Qty.`.
    #[serde(default = "default_bill_header_qty")]
    pub item_header_qty: String,
    /// Header text for the price column. Empty → `Price`.
    #[serde(default = "default_bill_header_price")]
    pub item_header_price: String,
    /// Header text for the amount column. Empty → `Amount`.
    #[serde(default = "default_bill_header_amount")]
    pub item_header_amount: String,
    /// Print order of piece columns (`name`, `qty`, `price`, `amount`),
    /// from the start edge. Empty → product default.
    #[serde(default)]
    pub item_col_order: Vec<String>,
    /// Draw thin vertical rules in the gaps between columns.
    #[serde(default)]
    pub item_col_sep: bool,
    /// `Total Qty: N` under the items.
    #[serde(default = "default_true")]
    pub total_show_qty: bool,
    /// `Sub Total` line.
    #[serde(default = "default_true")]
    pub total_show_subtotal: bool,
    /// `Grand Total` / TOTAL line (also follows the Total field's show).
    #[serde(default = "default_true")]
    pub total_show_grand: bool,
    /// Caption before the qty count. Empty → `Total Qty`.
    #[serde(default = "default_bill_total_qty_label")]
    pub total_qty_label: String,
    /// Caption for the subtotal / taxable-value line. Empty → `Sub Total`.
    #[serde(default = "default_bill_total_subtotal_label")]
    pub total_subtotal_label: String,
    /// Caption for the charged total. Empty → `Grand Total` (running) /
    /// `Total` (settled receipt uses [`Self::total_paid_label`]).
    #[serde(default = "default_bill_total_grand_label")]
    pub total_grand_label: String,
    /// Caption for the settled sale total line. Empty → `Total`.
    #[serde(default = "default_bill_total_paid_label")]
    pub total_paid_label: String,
    /// Where each total prints (owner, 2026-11-17: "Total Qty in the same
    /// column as that of Qty", "Grand Total in the same column as Amount",
    /// all "in single row"): on its own line, as before, or under the item
    /// columns ([`BillTotalRows`]). Not written while it has its own line.
    #[serde(default, skip_serializing_if = "BillTotalPlace::is_default")]
    pub total_qty_place: BillTotalPlace,
    #[serde(default, skip_serializing_if = "BillTotalPlace::is_default")]
    pub total_subtotal_place: BillTotalPlace,
    #[serde(default, skip_serializing_if = "BillTotalPlace::is_default")]
    pub total_grand_place: BillTotalPlace,
    /// Field-id pairs printed on one row when both show. A save without it
    /// gets the product default (cashier+bill_no); an empty list
    /// means no pairs — every line on its own row (the owner sets Alone).
    #[serde(default = "default_bill_meta_pairs")]
    pub meta_pairs: Vec<[String; 2]>,
    /// Print order of the lines. Empty means [`BillStyleField::ALL`].
    #[serde(default)]
    pub field_order: Vec<String>,
    /// Identical copies of each bill. Clamped to 1..=3.
    #[serde(default = "default_bill_copies")]
    pub copies: u32,
    /// Content width in millimetres for the bill only (KOT has its own).
    /// Default 210 mm (page-width guest bill).
    #[serde(default = "default_bill_content_width_mm")]
    pub content_width_mm: u32,
    /// The owner's item columns. `None` only in saves from before columns;
    /// [`Self::clamped`] fills them to the content width.
    #[serde(default)]
    pub item_columns: Option<BillItemColumns>,
}

fn default_bill_meta_pairs() -> Vec<[String; 2]> {
    vec![["cashier".into(), "bill_no".into()]]
}

fn default_bill_copies() -> u32 {
    1
}

fn default_bill_content_width_mm() -> u32 {
    210
}

fn default_bill_header_item() -> String {
    "Item".into()
}
fn default_bill_header_qty() -> String {
    "Qty.".into()
}
fn default_bill_header_price() -> String {
    "Price".into()
}
fn default_bill_header_amount() -> String {
    "Amount".into()
}

const BILL_HEADER_LABEL_MAX: usize = 24;

fn scrub_bill_header_label(raw: &str, fallback: &str) -> String {
    let t: String = raw.chars().take(BILL_HEADER_LABEL_MAX).collect();
    let t = t.trim().to_string();
    if t.is_empty() {
        fallback.to_string()
    } else {
        t
    }
}

fn default_bill_total_qty_label() -> String {
    "Total Qty".into()
}
fn default_bill_total_subtotal_label() -> String {
    "Sub Total".into()
}
fn default_bill_total_grand_label() -> String {
    "Grand Total".into()
}
fn default_bill_total_paid_label() -> String {
    "Total".into()
}
fn default_bill_item_col_order() -> Vec<String> {
    BillItemCol::ALL
        .iter()
        .map(|c| c.id().to_string())
        .collect()
}
/// Clamp a column gap em (the old `T` directive of jobs queued before
/// owner columns).
fn clamp_bill_col_gap_em(v: f32) -> f32 {
    if !v.is_finite() {
        return 1.15;
    }
    v.clamp(0.2, 3.0)
}

/// Clamp a numeric column min-width em (old `T` directive).
fn clamp_bill_col_min_em(v: f32) -> f32 {
    if !v.is_finite() {
        return 0.0;
    }
    v.clamp(0.0, 24.0)
}

impl Default for BillSlipFormat {
    fn default() -> Self {
        Self {
            header: String::new(),
            fssai: String::new(),
            footer: "Thanks".into(),
            upi_vpa: String::new(),
            upi_payee: String::new(),
            title: String::new(),
            shop: BillFieldStyle::face(22, KotFontWeight::Bold, KotAlign::Center),
            header_style: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Center),
            fssai_style: BillFieldStyle::face(12, KotFontWeight::Regular, KotAlign::Center),
            // Off by default — common FOH bills go address → Name → meta
            // without a CASH MEMO banner. Turn Title on when you want one.
            title_style: BillFieldStyle {
                show: false,
                size_em: 18,
                weight: KotFontWeight::Bold,
                align: KotAlign::Center,
            },
            customer: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Start),
            bill_no: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Start),
            time: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Start),
            cashier: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Start),
            // Body items a touch larger than meta — readable Qty/Price/Amount.
            items: BillFieldStyle::face(16, KotFontWeight::Regular, KotAlign::Start),
            total: BillFieldStyle::face(18, KotFontWeight::Bold, KotAlign::Start),
            footer_style: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Center),
            upi: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Center),
            item_layout: BillItemLayout::Columns,
            item_show_name: true,
            item_show_qty: true,
            item_show_price: true,
            item_show_amount: true,
            item_show_note: true,
            item_show_header: true,
            item_header_item: default_bill_header_item(),
            item_header_qty: default_bill_header_qty(),
            item_header_price: default_bill_header_price(),
            item_header_amount: default_bill_header_amount(),
            item_col_order: default_bill_item_col_order(),
            item_col_sep: false,
            total_show_qty: true,
            total_show_subtotal: true,
            total_show_grand: true,
            total_qty_label: default_bill_total_qty_label(),
            total_subtotal_label: default_bill_total_subtotal_label(),
            total_grand_label: default_bill_total_grand_label(),
            total_paid_label: default_bill_total_paid_label(),
            total_qty_place: BillTotalPlace::default(),
            total_subtotal_place: BillTotalPlace::default(),
            total_grand_place: BillTotalPlace::default(),
            meta_pairs: default_bill_meta_pairs(),
            field_order: Vec::new(),
            copies: 1,
            content_width_mm: 210,
            item_columns: None,
        }
    }
}

/// Pieces of one bill dish line. Layout decides columns vs stacked lines.
#[derive(Clone, Debug, Default)]
pub struct BillItemPieces<'a> {
    pub name: &'a str,
    pub qty: &'a str,
    pub price: &'a str,
    pub amount: &'a str,
    pub note: &'a str,
}

/// Build one dish from the owner's per-piece toggles and layout.
pub fn format_bill_item_block(fmt: &BillSlipFormat, pieces: BillItemPieces<'_>) -> String {
    match fmt.item_layout {
        BillItemLayout::Columns => format_bill_item_columns(fmt, pieces),
        BillItemLayout::Stacked => format_bill_item_stacked(fmt, pieces),
    }
}

fn format_bill_item_stacked(fmt: &BillSlipFormat, pieces: BillItemPieces<'_>) -> String {
    let mut lines: Vec<String> = Vec::new();
    if fmt.item_show_name {
        let name = pieces.name.trim();
        if !name.is_empty() {
            lines.push(name.to_string());
        }
    }
    if fmt.item_show_qty || fmt.item_show_price {
        let qty = pieces.qty.trim();
        let price = pieces.price.trim();
        let rate = match (fmt.item_show_qty, fmt.item_show_price, qty.is_empty(), price.is_empty()) {
            (true, true, false, false) => format!("{qty} x {price}"),
            (true, _, false, _) => qty.to_string(),
            (_, true, _, false) => price.to_string(),
            _ => String::new(),
        };
        if !rate.is_empty() {
            lines.push(rate);
        }
    }
    if fmt.item_show_amount {
        let amount = pieces.amount.trim();
        if !amount.is_empty() {
            lines.push(amount.to_string());
        }
    }
    push_bill_note_line(&mut lines, fmt, pieces.note);
    lines.join("\n")
}

fn format_bill_item_columns(fmt: &BillSlipFormat, pieces: BillItemPieces<'_>) -> String {
    let mut cells: Vec<&str> = Vec::new();
    for col in fmt.ordered_item_cols() {
        let cell = match col {
            BillItemCol::Name => pieces.name.trim(),
            BillItemCol::Qty => pieces.qty.trim(),
            BillItemCol::Price => pieces.price.trim(),
            BillItemCol::Amount => pieces.amount.trim(),
        };
        cells.push(cell);
    }
    let mut out = if cells.iter().all(|c| c.is_empty()) {
        String::new()
    } else {
        column_line(&cells)
    };
    if fmt.item_show_note {
        let note = pieces.note.trim();
        if !note.is_empty() {
            let note_line = if note.starts_with('(') {
                note.to_string()
            } else {
                format!("({note})")
            };
            if out.is_empty() {
                out = note_line;
            } else {
                out.push('\n');
                out.push_str(&note_line);
            }
        }
    }
    out
}

fn push_bill_note_line(lines: &mut Vec<String>, fmt: &BillSlipFormat, note: &str) {
    if !fmt.item_show_note {
        return;
    }
    let note = note.trim();
    if note.is_empty() {
        return;
    }
    if note.starts_with('(') {
        lines.push(note.to_string());
    } else {
        lines.push(format!("({note})"));
    }
}

/// Column header for Items when layout is Columns and header is on.
pub fn format_bill_item_header(fmt: &BillSlipFormat) -> String {
    if fmt.item_layout != BillItemLayout::Columns || !fmt.item_show_header {
        return String::new();
    }
    let mut cells: Vec<String> = Vec::new();
    for col in fmt.ordered_item_cols() {
        cells.push(match col {
            BillItemCol::Name => scrub_bill_header_label(&fmt.item_header_item, "Item"),
            BillItemCol::Qty => scrub_bill_header_label(&fmt.item_header_qty, "Qty."),
            BillItemCol::Price => scrub_bill_header_label(&fmt.item_header_price, "Price"),
            BillItemCol::Amount => scrub_bill_header_label(&fmt.item_header_amount, "Amount"),
        });
    }
    column_line(&cells)
}

/// Cells of an item-column line, tab-separated. A lone column still ends in
/// a tab, so it lays out in its column (width, spaces, alignment) like any
/// other column line instead of as a plain line.
fn column_line<S: AsRef<str>>(cells: &[S]) -> String {
    let mut out = cells.iter().map(|c| c.as_ref()).collect::<Vec<_>>().join("\t");
    if cells.len() == 1 {
        out.push('\t');
    }
    out
}

/// The totals of one bill as lines, each total where the owner placed it.
///
/// A total with its own line prints as it always did (`Total Qty: 3`). A
/// total set under an item column goes on a row laid out on those columns
/// — its number in that column's box, with the column's width, spaces and
/// alignment — in the Total line's size and weight. Totals share such a row
/// until one needs a box that is taken; that one starts the next row. So
/// Total qty under Qty with Grand total under Amount is one row, and Sub
/// total and Grand total both under Amount are two.
///
/// Its caption sits before the number, or under another column of the same
/// row, or is not printed ([`BillTotalCaption`]). A caption-and-amount line
/// with no setting of its own (tax, service, rounding, a payment) follows
/// Sub total's place, so the amounts of a bill stay in one column.
///
/// Only when the items print in columns, and only under a column that is
/// shown: otherwise the total has its own line.
pub struct BillTotalRows<'a> {
    fmt: &'a BillSlipFormat,
    /// The shown item columns in print order; none when items are stacked.
    cols: Vec<BillItemCol>,
    lines: Vec<String>,
    /// The column row being filled, one cell per shown column; empty when
    /// no row is open.
    row: Vec<String>,
}

impl<'a> BillTotalRows<'a> {
    pub fn new(fmt: &'a BillSlipFormat) -> Self {
        let cols = if fmt.item_layout == BillItemLayout::Columns { fmt.ordered_item_cols() } else { Vec::new() };
        Self { fmt, cols, lines: Vec::new(), row: Vec::new() }
    }

    /// One of the three totals. `joined` is the line it prints on its own
    /// (`Total Qty: 3`); `caption` and `number` are its two parts.
    pub fn total(&mut self, kind: BillTotalKind, caption: &str, number: &str, joined: &str) {
        let place = self.fmt.total_place(kind).clone();
        self.put(&place, caption, number, joined, false);
    }

    /// A caption-and-amount line with no place of its own (tax, service,
    /// rounding, a payment): it follows Sub total's, and always keeps its
    /// caption — an amount nobody can name is no use on a bill.
    pub fn amount(&mut self, caption: &str, number: &str, joined: &str) {
        let place = self.fmt.total_subtotal_place.clone();
        self.put(&place, caption, number, joined, true);
    }

    /// A line of words (a scheme's name, a note): always its own line.
    pub fn words(&mut self, text: &str) {
        self.close_row();
        if !text.trim().is_empty() {
            self.lines.push(text.to_string());
        }
    }

    /// The lines in print order, for the Total line's style.
    pub fn finish(mut self) -> Vec<String> {
        self.close_row();
        self.lines
    }

    fn index_of(&self, col: BillItemCol) -> Option<usize> {
        self.cols.iter().position(|c| *c == col)
    }

    fn close_row(&mut self) {
        if self.row.iter().any(|cell| !cell.is_empty()) {
            self.lines.push(column_line(&self.row));
        }
        self.row.clear();
    }

    fn put(&mut self, place: &BillTotalPlace, caption: &str, number: &str, joined: &str, keep_caption: bool) {
        let Some(at) = place.number_col().and_then(|col| self.index_of(col)) else {
            self.words(joined);
            return;
        };
        // The caption's own box, when it has one that is shown and is not
        // the number's.
        let caption_at = match place.caption_place() {
            BillTotalCaption::Under(col) => self.index_of(col).filter(|i| *i != at),
            _ => None,
        };
        let with_number = caption_at.is_none() && (keep_caption || place.caption_place() != BillTotalCaption::Hidden);
        let taken = |row: &[String], i: usize| row.get(i).is_some_and(|cell| !cell.is_empty());
        if taken(&self.row, at) || caption_at.is_some_and(|i| taken(&self.row, i)) {
            self.close_row();
        }
        if self.row.is_empty() {
            self.row = vec![String::new(); self.cols.len()];
        }
        // One line a cell: a cell is laid out as its column's own text.
        let flat = |text: &str| text.replace(['\n', '\t', '|'], " ").trim().to_string();
        self.row[at] = flat(if with_number { joined } else { number });
        if let Some(i) = caption_at {
            self.row[i] = flat(caption);
        }
    }
}

impl BillSlipFormat {
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::default();
        }
        serde_json::from_str::<Self>(trimmed)
            .unwrap_or_default()
            .clamped()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.clone().clamped()).unwrap_or_else(|_| "{}".into())
    }

    /// Millimetres used for bill preview / print wrap.
    pub fn shown_content_width_mm(&self) -> u32 {
        let mm = if self.content_width_mm > 0 {
            self.content_width_mm
        } else {
            210
        };
        mm.clamp(40, 210)
    }

    /// Apply wrap width onto a Standard (bill) print format.
    pub fn apply_to_print_format(&self, format: &mut PrintFormat) {
        format.line = wrap_cols_for_paper_mm(self.shown_content_width_mm());
    }

    pub fn clamped(mut self) -> Self {
        for field in BillStyleField::ALL {
            let s = self.style_mut(field);
            s.size_em = s.size_em.clamp(KOT_EM_MIN, KOT_EM_MAX);
        }
        self.copies = self.copies.clamp(1, 3);
        self.content_width_mm = self.shown_content_width_mm();
        self.item_header_item =
            scrub_bill_header_label(&self.item_header_item, "Item");
        self.item_header_qty =
            scrub_bill_header_label(&self.item_header_qty, "Qty.");
        self.item_header_price =
            scrub_bill_header_label(&self.item_header_price, "Price");
        self.item_header_amount =
            scrub_bill_header_label(&self.item_header_amount, "Amount");
        self.total_qty_label =
            scrub_bill_header_label(&self.total_qty_label, "Total Qty");
        self.total_subtotal_label =
            scrub_bill_header_label(&self.total_subtotal_label, "Sub Total");
        self.total_grand_label =
            scrub_bill_header_label(&self.total_grand_label, "Grand Total");
        self.total_paid_label =
            scrub_bill_header_label(&self.total_paid_label, "Total");
        for kind in BillTotalKind::ALL {
            let place = self.total_place_mut(kind);
            *place = std::mem::take(place).clamped();
        }
        self.item_col_order = {
            let mut seen = std::collections::HashSet::new();
            let mut order = Vec::new();
            for id in &self.item_col_order {
                if let Some(col) = BillItemCol::from_id(id) {
                    if seen.insert(col) {
                        order.push(col.id().to_string());
                    }
                }
            }
            for col in BillItemCol::ALL {
                if seen.insert(col) {
                    order.push(col.id().to_string());
                }
            }
            order
        };
        let mut order = Vec::new();
        for id in &self.field_order {
            if let Some(field) = BillStyleField::from_id(id) {
                if !order.iter().any(|have| have == id) {
                    order.push(field.id().to_string());
                }
            }
        }
        for field in BillStyleField::ALL {
            let id = field.id();
            if !order.iter().any(|have| have == id) {
                order.push(id.to_string());
            }
        }
        self.field_order = order;
        // Only real pairs of pairable fields; none left means none (Alone).
        let mut pairs = Vec::new();
        for pair in &self.meta_pairs {
            let a = BillStyleField::from_id(pair[0].trim());
            let b = BillStyleField::from_id(pair[1].trim());
            if let (Some(a), Some(b)) = (a, b) {
                if a.is_pairable() && b.is_pairable() && a != b {
                    let ids = [a.id().to_string(), b.id().to_string()];
                    if !pairs.iter().any(|have: &[String; 2]| have == &ids) {
                        pairs.push(ids);
                    }
                }
            }
        }
        self.meta_pairs = pairs;
        // The owner's columns; saves from before columns start filling the
        // content width (then keep their mm whatever the width).
        self.item_columns = Some(self.item_columns().clamped());
        self
    }

    /// The item columns (filled to the content width when unset).
    pub fn item_columns(&self) -> BillItemColumns {
        self.item_columns
            .unwrap_or_else(|| BillItemColumns::filling(self.shown_content_width_mm()))
    }

    /// Shown columns' settings in print order (matches the cells of every
    /// item line and the header).
    fn ordered_slip_columns(&self) -> Vec<SlipColumn> {
        let cols = self.item_columns();
        self.ordered_item_cols().into_iter().map(|c| *cols.get(c)).collect()
    }

    /// Shown piece columns in print order. First flexes left; rest pack right.
    pub fn ordered_item_cols(&self) -> Vec<BillItemCol> {
        let order = if self.item_col_order.is_empty() {
            default_bill_item_col_order()
        } else {
            self.item_col_order.clone()
        };
        order
            .iter()
            .filter_map(|id| BillItemCol::from_id(id))
            .filter(|col| match col {
                BillItemCol::Name => self.item_show_name,
                BillItemCol::Qty => self.item_show_qty,
                BillItemCol::Price => self.item_show_price,
                BillItemCol::Amount => self.item_show_amount,
            })
            .collect()
    }

    /// Move a piece column earlier/later in [`Self::item_col_order`].
    pub fn move_item_col(&mut self, col: BillItemCol, earlier: bool) {
        if self.item_col_order.is_empty() {
            self.item_col_order = default_bill_item_col_order();
        }
        let Some(i) = self
            .item_col_order
            .iter()
            .position(|id| BillItemCol::from_id(id) == Some(col))
        else {
            return;
        };
        if earlier && i > 0 {
            self.item_col_order.swap(i, i - 1);
        } else if !earlier && i + 1 < self.item_col_order.len() {
            self.item_col_order.swap(i, i + 1);
        }
    }

    pub fn caption_total_qty(&self) -> String {
        scrub_bill_header_label(&self.total_qty_label, "Total Qty")
    }
    pub fn caption_subtotal(&self) -> String {
        scrub_bill_header_label(&self.total_subtotal_label, "Sub Total")
    }
    pub fn caption_grand_total(&self) -> String {
        scrub_bill_header_label(&self.total_grand_label, "Grand Total")
    }
    pub fn caption_paid_total(&self) -> String {
        scrub_bill_header_label(&self.total_paid_label, "Total")
    }

    /// Where `kind` prints.
    pub fn total_place(&self, kind: BillTotalKind) -> &BillTotalPlace {
        match kind {
            BillTotalKind::Qty => &self.total_qty_place,
            BillTotalKind::Subtotal => &self.total_subtotal_place,
            BillTotalKind::Grand => &self.total_grand_place,
        }
    }

    pub fn total_place_mut(&mut self, kind: BillTotalKind) -> &mut BillTotalPlace {
        match kind {
            BillTotalKind::Qty => &mut self.total_qty_place,
            BillTotalKind::Subtotal => &mut self.total_subtotal_place,
            BillTotalKind::Grand => &mut self.total_grand_place,
        }
    }

    /// Pair `field` with `partner`, or clear its pair when `partner` is None.
    pub fn set_meta_partner(&mut self, field: BillStyleField, partner: Option<BillStyleField>) {
        if !field.is_pairable() {
            return;
        }
        self.meta_pairs
            .retain(|pair| pair[0] != field.id() && pair[1] != field.id());
        if let Some(other) = partner {
            if other.is_pairable() && other != field {
                self.meta_pairs
                    .retain(|pair| pair[0] != other.id() && pair[1] != other.id());
                self.meta_pairs
                    .push([field.id().to_string(), other.id().to_string()]);
            }
        }
    }

    /// Partner field when `field` is configured to share a meta line.
    pub fn meta_pair_of(&self, field: BillStyleField) -> Option<BillStyleField> {
        for pair in &self.meta_pairs {
            let (Some(a), Some(b)) = (
                BillStyleField::from_id(&pair[0]),
                BillStyleField::from_id(&pair[1]),
            ) else {
                continue;
            };
            if a == field {
                return Some(b);
            }
            if b == field {
                return Some(a);
            }
        }
        None
    }

    pub fn ordered(&self) -> Vec<BillStyleField> {
        self.clone()
            .clamped()
            .field_order
            .iter()
            .filter_map(|id| BillStyleField::from_id(id))
            .collect()
    }

    pub fn style(&self, field: BillStyleField) -> &BillFieldStyle {
        match field {
            BillStyleField::Shop => &self.shop,
            BillStyleField::Header => &self.header_style,
            BillStyleField::Fssai => &self.fssai_style,
            BillStyleField::Title => &self.title_style,
            BillStyleField::Customer => &self.customer,
            BillStyleField::BillNo => &self.bill_no,
            BillStyleField::Time => &self.time,
            BillStyleField::Cashier => &self.cashier,
            BillStyleField::Items => &self.items,
            BillStyleField::Total => &self.total,
            BillStyleField::Footer => &self.footer_style,
            BillStyleField::Upi => &self.upi,
        }
    }

    pub fn style_mut(&mut self, field: BillStyleField) -> &mut BillFieldStyle {
        match field {
            BillStyleField::Shop => &mut self.shop,
            BillStyleField::Header => &mut self.header_style,
            BillStyleField::Fssai => &mut self.fssai_style,
            BillStyleField::Title => &mut self.title_style,
            BillStyleField::Customer => &mut self.customer,
            BillStyleField::BillNo => &mut self.bill_no,
            BillStyleField::Time => &mut self.time,
            BillStyleField::Cashier => &mut self.cashier,
            BillStyleField::Items => &mut self.items,
            BillStyleField::Total => &mut self.total,
            BillStyleField::Footer => &mut self.footer_style,
            BillStyleField::Upi => &mut self.upi,
        }
    }

    pub fn text(&self, field: BillStyleField) -> String {
        match field {
            BillStyleField::Header => self.header.clone(),
            BillStyleField::Fssai => self.fssai.clone(),
            BillStyleField::Title => self.title.clone(),
            BillStyleField::Footer => self.footer.clone(),
            BillStyleField::Upi => {
                if self.upi_payee.trim().is_empty() {
                    self.upi_vpa.clone()
                } else if self.upi_vpa.trim().is_empty() {
                    self.upi_payee.clone()
                } else {
                    format!("{} | {}", self.upi_vpa.trim(), self.upi_payee.trim())
                }
            }
            _ => String::new(),
        }
    }

    pub fn set_text(&mut self, field: BillStyleField, raw: &str) {
        match field {
            BillStyleField::Header => self.header = raw.trim().to_string(),
            BillStyleField::Fssai => self.fssai = raw.trim().to_string(),
            BillStyleField::Title => self.title = raw.trim().to_string(),
            BillStyleField::Footer => self.footer = raw.trim().to_string(),
            BillStyleField::Upi => {
                let mut parts = raw.split('|');
                self.upi_vpa = parts.next().unwrap_or("").trim().to_string();
                self.upi_payee = parts.next().unwrap_or("").trim().to_string();
            }
            _ => {}
        }
    }

    /// Header stored as one line per address part (the books' column).
    pub fn header_lines(&self) -> String {
        self.header
            .split('|')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The bill the Preview and Test bill both print.
    pub fn render_sample(&self, shop: &str, when: &str) -> String {
        let _stage = BillStage::begin();
        let mut out = String::new();
        append_bill_field(&mut out, self, BillStyleField::Shop, shop, false);
        append_bill_field(&mut out, self, BillStyleField::Header, &self.header, false);
        if !self.fssai.trim().is_empty() {
            append_bill_field(
                &mut out,
                self,
                BillStyleField::Fssai,
                &format!("FSSAI Lic. No: {}", self.fssai.trim()),
                false,
            );
        }
        let title = if self.title.trim().is_empty() {
            "CASH MEMO"
        } else {
            self.title.trim()
        };
        append_bill_field(&mut out, self, BillStyleField::Title, title, false);
        append_bill_field(&mut out, self, BillStyleField::Customer, "Name:", false);
        append_bill_field(
            &mut out,
            self,
            BillStyleField::Time,
            &format!("Date: {when}"),
            false,
        );
        append_bill_field(&mut out, self, BillStyleField::Cashier, "Cashier: biller", false);
        append_bill_field(&mut out, self, BillStyleField::BillNo, "Bill No.: SAMPLE", false);
        append_bill_like(&mut out, self, BillStyleField::Items, "--------------------", false);
        let header = format_bill_item_header(self);
        if !header.is_empty() {
            append_bill_like(&mut out, self, BillStyleField::Items, &header, false);
        }
        let first = format_bill_item_block(
            self,
            BillItemPieces {
                name: "Paracetamol",
                qty: "2",
                price: "40.00",
                amount: "80.00",
                note: "",
            },
        );
        append_bill_field(&mut out, self, BillStyleField::Items, &first, false);
        let second = format_bill_item_block(
            self,
            BillItemPieces {
                name: "Cough syrup",
                qty: "1",
                price: "20.00",
                amount: "20.00",
                note: "",
            },
        );
        append_bill_field(&mut out, self, BillStyleField::Items, &second, false);
        append_bill_like(&mut out, self, BillStyleField::Items, "--------------------", false);
        let mut totals = BillTotalRows::new(self);
        if self.total_show_qty {
            let caption = self.caption_total_qty();
            totals.total(BillTotalKind::Qty, &caption, "3", &format!("{caption}: 3"));
        }
        if self.total_show_subtotal {
            let caption = self.caption_subtotal();
            totals.total(BillTotalKind::Subtotal, &caption, "100.00", &format!("{caption}: 100.00"));
        }
        if self.total_show_grand {
            let caption = self.caption_grand_total();
            totals.total(BillTotalKind::Grand, &caption, "100.00", &format!("{caption}: 100.00"));
        }
        for line in totals.finish() {
            append_bill_like(&mut out, self, BillStyleField::Total, &line, false);
        }
        append_bill_field(&mut out, self, BillStyleField::Footer, &self.footer, false);
        if !self.upi_vpa.trim().is_empty() {
            append_bill_field(
                &mut out,
                self,
                BillStyleField::Upi,
                &format!("UPI: {}", self.upi_vpa.trim()),
                false,
            );
        }
        if !self.upi_payee.trim().is_empty() {
            append_bill_field(&mut out, self, BillStyleField::Upi, &self.upi_payee, false);
        }
        bill_stage_finish(self, false)
    }
}

fn bill_mark(style: &BillFieldStyle, rtl: bool) -> String {
    let face = KotFieldStyle {
        size_em: style.size_em.clamp(KOT_EM_MIN, KOT_EM_MAX),
        weight: style.weight,
        align: style.align,
    };
    kot_style_prefix(&face, rtl)
}

/// A pair line: two halves, each its own field's mark and text
/// (`{mark}first\t{mark}second`). The first field takes the start half, so
/// in RTL it is written second (the right half).
fn append_pair(
    out: &mut String,
    first: (&BillFieldStyle, &str),
    second: (&BillFieldStyle, &str),
    rtl: bool,
) {
    let (left, right) = if rtl { (second, first) } else { (first, second) };
    let flat = |t: &str| t.replace(['\n', '\t'], " ").trim().to_string();
    out.push_str(&bill_mark(left.0, rtl));
    out.push_str(&flat(left.1));
    out.push('\t');
    out.push_str(&bill_mark(right.0, rtl));
    out.push_str(&flat(right.1));
    out.push('\n');
}

fn append_marked(out: &mut String, style: &BillFieldStyle, text: &str, rtl: bool) {
    let mark = bill_mark(style, rtl);
    for part in text.split(['\n', '|']) {
        // Tabs stay: on a column line an empty first or last cell is a
        // column left blank (a totals row; a lone column's line).
        let part = part.trim_matches(|c: char| c.is_whitespace() && c != '\t');
        if part.trim().is_empty() {
            continue;
        }
        out.push_str(&mark);
        out.push_str(part);
        out.push('\n');
    }
}

thread_local! {
    static BILL_STAGE: RefCell<Option<Vec<(BillStyleField, String, bool)>>> = const { RefCell::new(None) };
}

/// While this is alive, bill lines are held and written in [`BillSlipFormat::ordered`].
pub struct BillStage;

impl BillStage {
    pub fn begin() -> Self {
        BILL_STAGE.with(|slot| *slot.borrow_mut() = Some(Vec::new()));
        BillStage
    }
}

impl Drop for BillStage {
    fn drop(&mut self) {
        BILL_STAGE.with(|slot| *slot.borrow_mut() = None);
    }
}

pub fn bill_stage_finish(fmt: &BillSlipFormat, rtl: bool) -> String {
    let rows = BILL_STAGE.with(|slot| slot.borrow_mut().take()).unwrap_or_default();
    let mut out = String::new();
    if fmt.item_layout == BillItemLayout::Columns {
        out.push_str(&encode_slip_columns(
            &fmt.ordered_slip_columns(),
            fmt.shown_content_width_mm(),
            rtl,
            fmt.item_col_sep,
        ));
    }
    let mut consumed: std::collections::HashSet<BillStyleField> =
        std::collections::HashSet::new();
    let order = fmt.ordered();
    for field in &order {
        if consumed.contains(field) {
            continue;
        }
        // Pairable meta: one left\tright line when both partners have a
        // single shown text (Cashier | Bill no).
        if field.is_pairable() {
            if let Some(other) = fmt.meta_pair_of(*field) {
                if !consumed.contains(&other)
                    && order.contains(&other)
                    && fmt.style(*field).show
                    && fmt.style(other).show
                {
                    let left_texts: Vec<&str> = rows
                        .iter()
                        .filter(|(got, _, respect)| {
                            *got == *field && (!*respect || fmt.style(*field).show)
                        })
                        .map(|(_, t, _)| t.as_str())
                        .collect();
                    let right_texts: Vec<&str> = rows
                        .iter()
                        .filter(|(got, _, respect)| {
                            *got == other && (!*respect || fmt.style(other).show)
                        })
                        .map(|(_, t, _)| t.as_str())
                        .collect();
                    if left_texts.len() == 1
                        && right_texts.len() == 1
                        && !left_texts[0].contains('\t')
                        && !right_texts[0].contains('\t')
                    {
                        // Emit once, in field-order: whichever comes first is left.
                        let field_i = order.iter().position(|f| *f == *field).unwrap_or(0);
                        let other_i = order.iter().position(|f| *f == other).unwrap_or(0);
                        let (first_f, first_t, second_f, second_t) = if field_i <= other_i {
                            (*field, left_texts[0], other, right_texts[0])
                        } else {
                            (other, right_texts[0], *field, left_texts[0])
                        };
                        append_pair(
                            &mut out,
                            (fmt.style(first_f), first_t),
                            (fmt.style(second_f), second_t),
                            rtl,
                        );
                        consumed.insert(*field);
                        consumed.insert(other);
                        continue;
                    }
                }
            }
        }
        for (got, text, respect_show) in &rows {
            if *got != *field {
                continue;
            }
            if *respect_show && !fmt.style(*field).show {
                continue;
            }
            append_marked(&mut out, fmt.style(*field), text, rtl);
        }
    }
    out
}

fn stage_bill_line(field: BillStyleField, text: &str, respect_show: bool) -> bool {
    BILL_STAGE.with(|slot| {
        let mut stage = slot.borrow_mut();
        if let Some(rows) = stage.as_mut() {
            if !text.trim().is_empty() {
                rows.push((field, text.to_string(), respect_show));
            }
            true
        } else {
            false
        }
    })
}

/// One bill field, if it is shown. Empty text prints nothing.
pub fn append_bill_field(
    out: &mut String,
    fmt: &BillSlipFormat,
    field: BillStyleField,
    text: &str,
    rtl: bool,
) {
    if stage_bill_line(field, text, true) {
        return;
    }
    let style = fmt.style(field);
    if !style.show {
        return;
    }
    append_marked(out, style, text, rtl);
}

/// A rule or tax line that uses `field`'s size, weight, and alignment.
/// It follows that field's show/hide, the same as the field's own text.
pub fn append_bill_like(
    out: &mut String,
    fmt: &BillSlipFormat,
    field: BillStyleField,
    text: &str,
    rtl: bool,
) {
    append_bill_field(out, fmt, field, text, rtl);
}

/// Same type size / weight / align as `field`, but always prints — used for
/// fiscal lines (e.g. TAX INVOICE) that must appear even when Title is off.
pub fn append_bill_styled(
    out: &mut String,
    fmt: &BillSlipFormat,
    field: BillStyleField,
    text: &str,
    rtl: bool,
) {
    if text.trim().is_empty() {
        return;
    }
    if stage_bill_line(field, text, false) {
        return;
    }
    append_marked(out, fmt.style(field), text, rtl);
}

/// One line's type size, weight, alignment, and optional strike, from the mark.
struct KotLineStyle {
    size_em: u32,
    weight: KotFontWeight,
    align: KotAlign,
    strike: bool,
}

/// Alignment, point size, and body. Unmarked lines are 14 pt, start-aligned.
pub fn kot_preview_metrics(line: &str) -> (KotAlign, u32, &str) {
    let (style, body) = take_kot_line_style(line);
    match style {
        Some(style) => (style.align, style.size_em, body),
        None => (KotAlign::Start, 14, body),
    }
}

/// `Some` when `line` starts with a style mark. Body is the rest of the line.
fn take_kot_line_style(line: &str) -> (Option<KotLineStyle>, &str) {
    let mut chars = line.chars();
    if chars.next() != Some(KOT_STYLE_MARK) {
        return (None, line);
    }
    let rest = chars.as_str();
    let b = rest.as_bytes();
    // Current: two or three digits + weight (`24B`, `120R`). Older jobs:
    // S/N/L + weight.
    if b.len() >= 3 && b[0].is_ascii_digit() && b[1].is_ascii_digit() {
        let digits = if b.len() >= 4 && b[2].is_ascii_digit() { 3 } else { 2 };
        let size = b[..digits].iter().fold(0u32, |n, d| n * 10 + (d - b'0') as u32);
        let weight = match b[digits] as char {
            'B' => KotFontWeight::Bold,
            'T' => KotFontWeight::Light,
            _ => KotFontWeight::Regular,
        };
        let at = digits + 1;
        let (align, mut skip) = if b.len() > at && matches!(b[at] as char, 'S' | 'C' | 'E') {
            (KotAlign::from_mark(b[at] as char), at + 1)
        } else {
            (KotAlign::Start, at)
        };
        // Trailing `0`/`1` = strike flag (current). Bare marks without a digit
        // keep strike off and leave the body intact (`KOT - 12`).
        let strike = if b.len() > skip && matches!(b[skip] as char, '0' | '1') {
            let s = b[skip] as char == '1';
            skip += 1;
            s
        } else {
            false
        };
        return (
            Some(KotLineStyle {
                size_em: size.clamp(KOT_EM_MIN, KOT_EM_MAX),
                weight,
                align,
                strike,
            }),
            &rest[skip..],
        );
    }
    if b.len() >= 2 && matches!(b[0] as char, 'S' | 'N' | 'L') {
        let size = match b[0] as char {
            'S' => KOT_EM_SMALL,
            'L' => KOT_EM_LARGE,
            _ => KOT_EM_NORMAL,
        };
        let weight = match b[1] as char {
            'B' => KotFontWeight::Bold,
            'T' => KotFontWeight::Light,
            _ => KotFontWeight::Regular,
        };
        return (
            Some(KotLineStyle {
                size_em: size,
                weight,
                align: KotAlign::Start,
                strike: false,
            }),
            &rest[2..],
        );
    }
    (None, line)
}

/// Encode column metrics as a leading slip directive (not drawn) — the
/// old `T` form, kept to test that jobs queued before columns still print.
#[cfg(test)]
fn encode_slip_tab_layout(tab: &crate::receipt_raster::TabColumnLayout) -> String {
    let align = match tab.cell_align {
        crate::receipt_raster::LineAlign::Center => 'C',
        crate::receipt_raster::LineAlign::End => 'E',
        crate::receipt_raster::LineAlign::Start => 'S',
    };
    let sep = if tab.sep { '1' } else { '0' };
    format!(
        "{SLIP_TAB_MARK}T{:.2}|{:.2}|{:.2}|{:.2}|{:.2}|{align}|{sep}\n",
        tab.gap_em, tab.min_em[0], tab.min_em[1], tab.min_em[2], tab.min_em[3]
    )
}

/// The owner's item columns as the slip's leading directive (not drawn):
/// `\u{E001}C{rtl}{sep}|{width dots},{start px},{end px},{S|C|E}|…`, in the
/// order of the cells on every item line. Widths are the print's dots for
/// `content_mm`; alignments are logical (start / end follow `rtl`).
fn encode_slip_columns(cols: &[SlipColumn], content_mm: u32, rtl: bool, sep: bool) -> String {
    use std::fmt::Write as _;
    let mut out = format!("{SLIP_TAB_MARK}C{}{}", u8::from(rtl), u8::from(sep));
    for c in cols {
        let b = c.column_box(content_mm);
        let a = match c.align {
            KotAlign::Start => 'S',
            KotAlign::Center => 'C',
            KotAlign::End => 'E',
        };
        let _ = write!(out, "|{:.1},{},{},{a}", b.width, c.start_px, c.end_px);
    }
    out.push('\n');
    out
}

fn parse_slip_columns(payload: &str) -> Option<crate::receipt_raster::OwnerColumns> {
    let mut parts = payload.split('|');
    let flags = parts.next()?.as_bytes();
    let rtl = flags.first() == Some(&b'1');
    let sep = flags.get(1) == Some(&b'1');
    let mut cols = Vec::new();
    for part in parts {
        let f: Vec<&str> = part.split(',').collect();
        if f.len() < 4 {
            return None;
        }
        let width = f[0].parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0.0)?;
        // The directive carries the owner's px; the box is dots like its
        // width (a 4 px space drew 4 dots — half its size — until 2026-11-15).
        let start = crate::receipt_raster::px_dots(f[1].parse::<u32>().ok()?.min(SLIP_COL_SPACE_MAX_PX) as f32);
        let end = crate::receipt_raster::px_dots(f[2].parse::<u32>().ok()?.min(SLIP_COL_SPACE_MAX_PX) as f32);
        let align = KotAlign::from_mark(f[3].chars().next().unwrap_or('S')).line_align();
        cols.push(crate::receipt_raster::ColumnBox { width, start, end, align });
    }
    (!cols.is_empty()).then_some(crate::receipt_raster::OwnerColumns { rtl, cols, sep })
}

/// Pull a leading `\u{E001}…` directive off `text`: the owner's columns
/// (`C`) or, from jobs queued before columns, the old layout (`T`).
/// Returns the layout + body.
fn take_slip_tab_layout(text: &str) -> (Option<crate::receipt_raster::LineColumns>, &str) {
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return (None, text);
    };
    let chunk = first.strip_suffix('\n').unwrap_or(first);
    let mut chars = chunk.chars();
    if chars.next() != Some(SLIP_TAB_MARK) {
        return (None, text);
    }
    let rest = chars.as_str();
    let body = text.get(first.len()..).unwrap_or("");
    if let Some(payload) = rest.strip_prefix('C') {
        return match parse_slip_columns(payload) {
            Some(cols) => (Some(crate::receipt_raster::LineColumns::Owner(std::sync::Arc::new(cols))), body),
            None => (None, body),
        };
    }
    match take_legacy_slip_tab_layout(text) {
        (Some(tab), body) => (Some(crate::receipt_raster::LineColumns::Legacy(tab)), body),
        (None, body) => (None, body),
    }
}

/// The old `\u{E001}T…` directive (jobs queued before owner columns).
fn take_legacy_slip_tab_layout(
    text: &str,
) -> (
    Option<crate::receipt_raster::TabColumnLayout>,
    &str,
) {
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return (None, text);
    };
    let (chunk, _) = match first.strip_suffix('\n') {
        Some(body) => (body, true),
        None => (first, false),
    };
    let mut chars = chunk.chars();
    if chars.next() != Some(SLIP_TAB_MARK) {
        return (None, text);
    }
    let rest = chars.as_str();
    if !rest.starts_with('T') {
        return (None, text);
    }
    let payload = &rest[1..];
    let parts: Vec<&str> = payload.split('|').collect();
    if parts.len() < 6 {
        return (None, text);
    }
    let parse = |s: &str| s.parse::<f32>().ok().filter(|v| v.is_finite());
    let (Some(gap), Some(m0), Some(m1), Some(m2), Some(m3)) = (
        parse(parts[0]),
        parse(parts[1]),
        parse(parts[2]),
        parse(parts[3]),
        parse(parts[4]),
    ) else {
        return (None, text);
    };
    let cell_align = match parts[5].chars().next().unwrap_or('E') {
        'C' => crate::receipt_raster::LineAlign::Center,
        'S' => crate::receipt_raster::LineAlign::Start,
        _ => crate::receipt_raster::LineAlign::End,
    };
    let sep = parts
        .get(6)
        .and_then(|s| s.chars().next())
        .map(|c| c == '1')
        .unwrap_or(false);
    let tab = crate::receipt_raster::TabColumnLayout {
        gap_em: clamp_bill_col_gap_em(gap),
        min_em: [
            clamp_bill_col_min_em(m0),
            clamp_bill_col_min_em(m1),
            clamp_bill_col_min_em(m2),
            clamp_bill_col_min_em(m3),
        ],
        cell_align,
        sep,
    };
    let body_start = first.len();
    (Some(tab), text.get(body_start..).unwrap_or(""))
}

/// Remove per-field style marks (Settings preview, plain expand).
/// When a line was marked strike, approximate it with combining long-stroke
/// overlays on the name (left of `\t`) so the on-screen preview shows the cue.
pub fn strip_kot_style_marks(text: &str) -> String {
    let (_, body) = take_slip_tab_layout(text);
    let mut out = String::with_capacity(body.len());
    for line in body.split_inclusive('\n') {
        let (chunk, nl) = match line.strip_suffix('\n') {
            Some(body) => (body, true),
            None => (line, false),
        };
        if chunk.starts_with(SLIP_TAB_MARK) {
            continue;
        }
        let (style, bare) = take_kot_line_style(chunk);
        // A pair line's second half carries its own mark after the tab.
        let paired;
        let bare = match bare.split_once('\t') {
            Some((first, rest)) if rest.starts_with(KOT_STYLE_MARK) => {
                paired = format!("{first}\t{}", take_kot_line_style(rest).1);
                paired.as_str()
            }
            _ => bare,
        };
        if style.map(|s| s.strike).unwrap_or(false) {
            out.push_str(&strike_approx_text(bare));
        } else {
            out.push_str(bare);
        }
        if nl {
            out.push('\n');
        }
    }
    out
}

/// Combining long stroke (U+0336) after each name character; qty after `\t` stays clear.
fn strike_approx_text(line: &str) -> String {
    let (name, qty) = match line.split_once('\t') {
        Some((n, q)) => (n, Some(q)),
        None => (line, None),
    };
    let mut out = String::with_capacity(name.len().saturating_mul(2));
    for ch in name.chars() {
        out.push(ch);
        if !ch.is_whitespace() {
            out.push('\u{0336}');
        }
    }
    if let Some(q) = qty {
        out.push('\t');
        out.push_str(q);
    }
    out
}

/// Qty column text. `x3`, `×3`, and `x1×` become `3` / `1`. `2 kg` stays.
/// The `x` is a piece-count marker, never part of the dish name.
pub fn kot_qty_column(qty: &str) -> String {
    let q = qty.trim();
    let rest = q.trim_start_matches(['×', '✕', 'x', 'X', ' ']);
    let mut end = 0usize;
    let mut seen_digit = false;
    let mut seen_dot = false;
    for (i, ch) in rest.char_indices() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            end = i + ch.len_utf8();
        } else if (ch == '.' || ch == ',') && seen_digit && !seen_dot {
            seen_dot = true;
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    if !seen_digit {
        return q.to_string();
    }
    let number = rest[..end].trim_end_matches(['.', ',']);
    let after = rest[end..]
        .trim()
        .trim_start_matches(['×', '✕', 'x', 'X'])
        .trim();
    if after.is_empty() {
        number.to_string()
    } else {
        format!("{number} {after}")
    }
}

/// Monospace qty field width — fits the `Qty.` header.
pub const KOT_QTY_COLS: usize = 4;

/// Expand `name\tqty` lines to a fixed right qty column for ESC/POS text
/// printers and on-screen preview (monospace cells). Raster keeps the tab
/// and aligns in dots instead.
pub fn expand_kot_column_tabs(text: &str, cols: usize) -> String {
    let cols = cols.max(8);
    let (tab, body) = take_slip_tab_layout(text);
    let tab = match tab {
        Some(crate::receipt_raster::LineColumns::Legacy(tab)) => tab,
        _ => crate::receipt_raster::TabColumnLayout::LEGACY,
    };
    let mut out = String::with_capacity(body.len().saturating_add(16));
    let mut first = true;
    for line in body.lines() {
        if line.starts_with(SLIP_TAB_MARK) {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        out.push_str(&expand_kot_column_tab_line(line, cols, tab));
    }
    if body.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn split_kot_style_prefix(line: &str) -> (String, &str) {
    let (style, body) = take_kot_line_style(line);
    match style {
        Some(style) => (
            kot_style_prefix_strike(
                &KotFieldStyle {
                    size_em: style.size_em,
                    weight: style.weight,
                    align: style.align,
                },
                false,
                style.strike,
            ),
            body,
        ),
        None => (String::new(), line),
    }
}

fn expand_kot_column_tab_line(
    line: &str,
    cols: usize,
    tab: crate::receipt_raster::TabColumnLayout,
) -> String {
    let (mark, body) = split_kot_style_prefix(line);
    if !body.contains('\t') {
        return format!("{mark}{body}");
    }
    let parts: Vec<&str> = body.split('\t').collect();
    let gap_cols = tab.gap_em.round().clamp(1.0, 4.0) as usize;
    let gap = if tab.sep {
        let side = gap_cols.saturating_sub(1) / 2;
        format!(
            "{}|{}",
            " ".repeat(side),
            " ".repeat(gap_cols.saturating_sub(1 + side))
        )
    } else {
        " ".repeat(gap_cols)
    };
    let fmt_cell = |text: &str, width: usize| -> String {
        match tab.cell_align {
            crate::receipt_raster::LineAlign::Center => {
                format!("{:^width$}", text, width = width)
            }
            crate::receipt_raster::LineAlign::Start => {
                format!("{:<width$}", text, width = width)
            }
            crate::receipt_raster::LineAlign::End => {
                format!("{:>width$}", text, width = width)
            }
        }
    };
    if parts.len() == 2 {
        let right = parts[1].trim();
        let left = parts[0].trim_end();
        let min_w = tab.min_em[0].round().max(0.0) as usize;
        let qty_w = KOT_QTY_COLS
            .max(right.chars().count())
            .max(min_w);
        let name_w = cols.saturating_sub(qty_w.saturating_add(gap_cols.saturating_sub(1)));
        let left = {
            let n = left.chars().count();
            if n <= name_w {
                left.to_string()
            } else {
                left.chars().take(name_w.saturating_sub(1)).collect::<String>() + "…"
            }
        };
        let mid = name_w.saturating_sub(left.chars().count());
        return format!(
            "{mark}{left}{}{}",
            " ".repeat(mid),
            fmt_cell(right, qty_w)
        );
    }
    // Bill Item | Qty | Price | Amount (or any 3+ cells): pack trailing
    // cells from the right; first cell flexes.
    let rights: Vec<&str> = parts[1..].iter().map(|p| p.trim()).collect();
    let right_widths: Vec<usize> = rights
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let min_w = tab
                .min_em
                .get(i)
                .copied()
                .unwrap_or(0.0)
                .round()
                .max(0.0) as usize;
            r.chars().count().max(4).max(min_w)
        })
        .collect();
    let rights_w: usize = right_widths.iter().sum::<usize>()
        + gap_cols.saturating_mul(rights.len().max(1));
    let name_w = cols.saturating_sub(rights_w);
    let left = {
        let raw = parts[0].trim_end();
        let n = raw.chars().count();
        if n <= name_w {
            raw.to_string()
        } else if name_w == 0 {
            String::new()
        } else {
            raw.chars().take(name_w.saturating_sub(1)).collect::<String>() + "…"
        }
    };
    let mut line_out = format!(
        "{mark}{left}{}",
        " ".repeat(name_w.saturating_sub(left.chars().count()))
    );
    for (r, w) in rights.iter().zip(right_widths.iter()) {
        line_out.push_str(&gap);
        line_out.push_str(&fmt_cell(r, *w));
    }
    line_out
}

/// Legacy alias — prefer [`PrintFormat`] / format ids.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrinterMode {
    #[default]
    Standard,
    Kot,
}

pub type PrinterWire = PrinterMode;

impl PrinterMode {
    pub fn format_id(self) -> &'static str {
        match self {
            Self::Standard => FORMAT_STANDARD_PORTRAIT,
            Self::Kot => FORMAT_KOT,
        }
    }

    pub fn from_legacy_wire_str(s: &str) -> Self {
        match s {
            "esc_pos" | "kot" => Self::Kot,
            _ => Self::Standard,
        }
    }
}

/// One named printer in this device's pool (destination and its paper).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedPrinter {
    pub id: String,
    pub label: String,
    /// Blank = system default; name = system printer; `ip` / `ip:port` = network;
    /// `bt:MAC`, `usb:VID:PID`, `android-print:`.
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
    /// Ignored on load (legacy); printers no longer store a mode.
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

/// How a place's ordered printer list is tried.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoutePrintMode {
    /// Try in order; stop after the first success (default).
    #[default]
    First,
    /// Send to every listed printer; collect per-printer failures.
    All,
}

impl RoutePrintMode {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "all" | "every" | "fanout" => Self::All,
            _ => Self::First,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::All => "all",
        }
    }
}

/// Ordered printer ids for each print feature, plus which format each uses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintRoutes {
    #[serde(default)]
    pub bill: Vec<String>,
    /// Station id → ordered printer ids. Key `""` is unassigned items only.
    #[serde(default)]
    pub kot: BTreeMap<String, Vec<String>>,
    /// Format id for Bill (default [`FORMAT_STANDARD_PORTRAIT`]).
    #[serde(default = "default_format_standard")]
    pub bill_format: String,
    /// Station id → format id for that kitchen (missing → [`FORMAT_KOT`]).
    #[serde(default)]
    pub kot_format: BTreeMap<String, String>,
    /// Bill place: first success vs all printers.
    #[serde(default)]
    pub bill_print_mode: RoutePrintMode,
    /// Station id → print mode (missing → [`RoutePrintMode::First`]).
    #[serde(default)]
    pub kot_print_mode: BTreeMap<String, RoutePrintMode>,
}

fn default_format_standard() -> String { FORMAT_STANDARD_PORTRAIT.into() }

impl Default for PrintRoutes {
    fn default() -> Self {
        Self {
            bill: Vec::new(),
            kot: BTreeMap::from([("".into(), Vec::new())]),
            bill_format: FORMAT_STANDARD_PORTRAIT.into(),
            kot_format: BTreeMap::new(),
            bill_print_mode: RoutePrintMode::First,
            kot_print_mode: BTreeMap::new(),
        }
    }
}

/// This device's real printers, configs (formats), virtual printers, and
/// legacy place routes (runtime until place-attach UI exists).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DevicePrinters {
    /// Real printers (OS / network destinations).
    pub printers: Vec<NamedPrinter>,
    /// Configs (slip look + send path). Built-ins: portrait, landscape, KOT.
    #[serde(default)]
    pub formats: Vec<PrintFormat>,
    /// Virtual printers: real printer + config.
    #[serde(default)]
    pub virtual_printers: Vec<VirtualPrinter>,
    pub routes: PrintRoutes,
    /// Legacy; discarded on load (station grouping removed).
    #[serde(default, skip_serializing)]
    pub station_groups: Vec<StationGroup>,
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
            virtual_printers: Vec::new(),
            routes: PrintRoutes::default(),
            station_groups: Vec::new(),
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
        struct NamedRaw {
            id: String,
            #[serde(default)]
            label: String,
            #[serde(default)]
            target: String,
            #[serde(default, alias = "paper_width_mm")]
            virtual_paper_width_mm: f64,
            #[serde(default, alias = "paper_length_mm")]
            virtual_paper_length_mm: f64,
            #[serde(default)]
            reported_width_mm: f64,
            #[serde(default)]
            reported_length_mm: f64,
            #[serde(default)]
            reported_name: String,
            #[serde(default)]
            reported_edges_mm: [f64; 4],
        }
        #[derive(Deserialize)]
        struct New {
            #[serde(default)]
            printers: Vec<NamedRaw>,
            #[serde(default)]
            formats: Vec<PrintFormat>,
            #[serde(default)]
            virtual_printers: Vec<VirtualPrinter>,
            #[serde(default)]
            routes: PrintRoutes,
            #[serde(default)]
            station_groups: Vec<StationGroup>,
        }
        let parsed: New = serde_json::from_value(value).unwrap_or_else(|_| New {
            printers: Vec::new(),
            formats: Vec::new(),
            virtual_printers: Vec::new(),
            routes: PrintRoutes::default(),
            station_groups: Vec::new(),
        });
        let mut formats = parsed.formats;
        PrintFormat::ensure_builtins(&mut formats);
        let mut out = DevicePrinters {
            printers: parsed
                .printers
                .into_iter()
                .map(|r| NamedPrinter {
                    id: r.id,
                    label: r.label,
                    target: r.target,
                    virtual_paper_width_mm: r.virtual_paper_width_mm,
                    virtual_paper_length_mm: r.virtual_paper_length_mm,
                    reported_width_mm: r.reported_width_mm,
                    reported_length_mm: r.reported_length_mm,
                    reported_name: r.reported_name,
                    reported_edges_mm: r.reported_edges_mm,
                    _legacy_mode: None,
                })
                .collect(),
            formats,
            virtual_printers: parsed.virtual_printers,
            routes: parsed.routes,
            station_groups: Vec::new(), // discarded
        };
        let _ = parsed.station_groups;
        scrub_fake_default_printers(&mut out);
        if out.routes.bill_format.trim().is_empty() {
            out.routes.bill_format = FORMAT_STANDARD_PORTRAIT.into();
        }
        out.migrate_legacy_standard_ids();
        out.collapse_one_output_per_printer();
        out.flatten_routes_to_real_printers();
        if !out.routes.kot.contains_key("") {
            out.routes.kot.insert(String::new(), Vec::new());
        }
        return out;
    }
    DevicePrinters::default()
}

/// Drop the old synthetic pool entry labeled "Default" with a blank target —
/// that was never a real printer and only confused Settings.
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
    /// Ordered printer ids for this route (empty = don't print that place).
    ///
    /// Each Bill / kitchen station is independent. If this station already has
    /// its own map entry — even an empty one after the owner cleared it — that
    /// list is used as-is (no silent copy onto every other station).
    ///
    /// Only when the station key is **missing** (legacy / never configured)
    /// do we fall back: default kitchen `""`, then Bill.
    pub fn print_mode_for(&self, key: RouteKey) -> RoutePrintMode {
        match key {
            RouteKey::Bill => self.routes.bill_print_mode,
        }
    }

    pub fn set_print_mode(&mut self, key: RouteKey, mode: RoutePrintMode) {
        match key {
            RouteKey::Bill => self.routes.bill_print_mode = mode,
        }
    }

    pub fn route_for(&self, key: RouteKey) -> &[String] {
        match key {
            RouteKey::Bill => &self.routes.bill,
        }
    }

    /// Resolve route ids to print targets (skips unknown ids).
    pub fn targets_for(&self, key: RouteKey) -> Vec<PrinterTarget> {
        self.destinations_for(key)
            .into_iter()
            .map(|d| d.target)
            .collect()
    }

    /// Build a transient format from a virtual printer's inline values.
    ///
    /// Layout (portrait/landscape) + delivery come from the VP. Type size /
    /// wrap / margins are **not** VP knobs (`docs/PRINTER.md`) — callers that
    /// print a place must overlay those from [`Self::format_for`] via
    /// [`Self::destinations_for`].
    pub fn format_from_virtual(&self, vp: &VirtualPrinter) -> PrintFormat {
        let v = vp.values();
        PrintFormat {
            id: format!("vp:{}", vp.id),
            label: vp.name.clone(),
            document: v.document_flag(),
            landscape: v.landscape,
            delivery: v.delivery.clone(),
            em: v.em,
            line: v.line,
            baseline: v.em + 2.0,
            side: v.side,
            place: Default::default(),
        }
    }

    /// Format id for this route (Bill → bill_format; kitchen → kot_format / KOT).
    pub fn format_id_for(&self, key: RouteKey) -> String {
        match key {
            RouteKey::Bill => {
                let f = self.routes.bill_format.trim();
                if f.is_empty() {
                    FORMAT_STANDARD_PORTRAIT.into()
                } else {
                    normalize_format_id(f).to_string()
                }
            }
        }
    }

    pub fn format_by_id(&self, id: &str) -> PrintFormat {
        let id = normalize_format_id(id);
        self.formats
            .iter()
            .find(|f| f.id == id)
            .cloned()
            .unwrap_or_else(|| {
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
        if id.is_empty() {
            return;
        }
        match key {
            RouteKey::Bill => self.routes.bill_format = id.to_string(),
        }
    }

    /// Set the place Profile and rewrite every Output on that place to it.
    pub fn set_place_profile(&mut self, key: RouteKey, profile_id: &str) {
        let id = profile_id.trim();
        if id.is_empty() {
            return;
        }
        self.set_route_format(key, id);
        let vp_ids: std::collections::HashSet<String> =
            self.virtual_printers.iter().map(|v| v.id.clone()).collect();
        let rewrite = |slots: &mut Vec<String>| {
            for slot in slots.iter_mut() {
                let raw = slot.trim();
                if vp_ids.contains(raw) {
                    continue;
                }
                let (pid, _) = decode_route_slot(slot);
                if !pid.is_empty() {
                    *slot = encode_route_slot(&pid, id);
                }
            }
        };
        match key {
            RouteKey::Bill => rewrite(&mut self.routes.bill),
        }
    }

    /// One Output per printer on each place (first profile kept). Migrates the
    /// old three-face assign list (same device as KOT + Portrait + Landscape).
    /// Virtual-printer attach slots stay one-per-VP id.
    pub fn collapse_one_output_per_printer(&mut self) {
        let vp_ids: std::collections::HashSet<String> =
            self.virtual_printers.iter().map(|v| v.id.clone()).collect();
        let collapse = |slots: &mut Vec<String>| {
            let mut seen = std::collections::HashSet::new();
            let mut out = Vec::new();
            for slot in slots.iter() {
                let raw = slot.trim();
                if vp_ids.contains(raw) {
                    if seen.insert(raw.to_string()) {
                        out.push(raw.to_string());
                    }
                    continue;
                }
                let (pid, fmt) = decode_route_slot(slot);
                if pid.is_empty() || !seen.insert(pid.clone()) {
                    continue;
                }
                out.push(if fmt.is_empty() {
                    pid
                } else {
                    encode_route_slot(&pid, &fmt)
                });
            }
            *slots = out;
        };
        collapse(&mut self.routes.bill);
        for slots in self.routes.kot.values_mut() {
            collapse(slots);
        }
    }

    /// Seed virtual printers once from legacy route slots (unique pairs).
    pub fn ensure_virtual_printers_from_routes(&mut self) {
        if !self.virtual_printers.is_empty() {
            return;
        }
        let known: std::collections::HashSet<String> =
            self.printers.iter().map(|p| p.id.clone()).collect();
        let mut seen = std::collections::HashSet::new();
        let mut pairs: Vec<(String, String)> = Vec::new();
        let mut consider = |slot: &str, default_cfg: &str| {
            let (pid, fmt) = decode_route_slot(slot);
            if pid.is_empty() || !known.contains(&pid) {
                return;
            }
            let cfg = if fmt.is_empty() {
                default_cfg.to_string()
            } else {
                fmt
            };
            if seen.insert((pid.clone(), cfg.clone())) {
                pairs.push((pid, cfg));
            }
        };
        let bill_fmt = self.routes.bill_format.clone();
        for slot in self.routes.bill.clone() {
            consider(&slot, &bill_fmt);
        }
        let kot_lists: Vec<(String, Vec<String>)> = self
            .routes
            .kot
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (station_id, list) in kot_lists {
            let default = self
                .routes
                .kot_format
                .get(&station_id)
                .cloned()
                .unwrap_or_else(|| FORMAT_KOT.to_string());
            for slot in list {
                consider(&slot, &default);
            }
        }
        for (pid, cfg_id) in pairs {
            let fmt = self.format_by_id(&cfg_id);
            let values = VirtualPrinterValues {
                landscape: fmt.landscape,
                em: fmt.em,
                line: fmt.line,
                side: fmt.side,
                delivery: if fmt.document {
                    DELIVERY_SYSTEM_SPOOLER.into()
                } else {
                    DELIVERY_ESCPOS.into()
                },
                paper_width_mm: if fmt.line <= 34 { 58 } else { 80 },
            };
            let _ = self.add_virtual_printer("", &pid, values);
        }
    }

    /// Create a virtual printer: real printer + inline configuration values.
    pub fn add_virtual_printer(
        &mut self,
        name: &str,
        printer_id: &str,
        values: VirtualPrinterValues,
    ) -> Result<String, String> {
        let pid = printer_id.trim();
        if pid.is_empty() || self.printer_by_id(pid).is_none() {
            return Err("pick a real printer".into());
        }
        let real_label = self
            .printer_by_id(pid)
            .map(|p| {
                let l = p.label.trim();
                if l.is_empty() {
                    p.target.clone()
                } else {
                    l.to_string()
                }
            })
            .unwrap_or_else(|| pid.to_string());
        let name = {
            let t = name.trim();
            if t.is_empty() {
                real_label
            } else {
                t.to_string()
            }
        };
        let id = uuid::Uuid::new_v4().to_string();
        let mut vp = VirtualPrinter {
            id: id.clone(),
            name,
            printer_id: pid.to_string(),
            landscape: false,
            em: STD_EM_NORMAL,
            line: 40,
            side: STD_SIDE,
            delivery: DELIVERY_SYSTEM_SPOOLER.into(),
            paper_width_mm: 80,
            document: true,
            config_id: String::new(),
        };
        vp.apply_values(values);
        self.virtual_printers.push(vp);
        Ok(id)
    }

    pub fn update_virtual_printer_values(
        &mut self,
        id: &str,
        values: VirtualPrinterValues,
    ) -> bool {
        if let Some(v) = self.virtual_printers.iter_mut().find(|v| v.id == id) {
            v.apply_values(values);
            true
        } else {
            false
        }
    }

    pub fn remove_virtual_printer(&mut self, id: &str) {
        let id = id.trim();
        self.virtual_printers.retain(|v| v.id != id);
        self.routes.bill.retain(|s| s.trim() != id);
        for slots in self.routes.kot.values_mut() {
            slots.retain(|s| s.trim() != id);
        }
    }

    /// Portrait/landscape and delivery for the bill (`standard`) or the kitchen ticket.
    pub fn set_job_drive(&mut self, standard: bool, landscape: bool, delivery: &str) {
        let delivery = if delivery.trim().is_empty() {
            DELIVERY_SYSTEM_SPOOLER
        } else {
            delivery.trim()
        };
        let document = delivery == DELIVERY_SYSTEM_SPOOLER;
        for f in self.formats.iter_mut() {
            let hit = if standard {
                is_standard_document(&f.id)
            } else {
                f.id == FORMAT_KOT
            };
            if hit {
                f.landscape = landscape;
                f.delivery = delivery.to_string();
                f.document = document;
            }
        }
    }

    /// Where the bill (`standard`) or the kitchen ticket sits on a paper
    /// bigger than it (owner, 2026-11-17): on the formats
    /// [`Self::set_job_drive`] turns, and on the format the route uses.
    pub fn set_job_place(&mut self, standard: bool, place: crate::os_paper::Place) {
        let routed: Vec<String> = if standard { vec![self.format_id_for(RouteKey::Bill)] } else { Vec::new() };
        for f in self.formats.iter_mut() {
            let hit = if standard {
                is_standard_document(&f.id)
            } else {
                f.id == FORMAT_KOT
            };
            if hit || routed.contains(&f.id) {
                f.place = place;
            }
        }
    }

    /// Toggle a real printer on a bill or kitchen route. Returns whether it is attached after.
    pub fn toggle_printer_on_route(&mut self, key: RouteKey, printer_id: &str) -> Option<bool> {
        let pid = printer_id.trim();
        if pid.is_empty() || self.printer_by_id(pid).is_none() {
            return None;
        }
        let slots: Vec<String> = self.route_for(key).to_vec();
        let found = slots.iter().position(|s| {
            let raw = s.trim();
            raw == pid
                || decode_route_slot(raw).0 == pid
                || self
                    .virtual_printer_by_id(raw)
                    .is_some_and(|v| v.printer_id == pid)
        });
        let route = self.route_mut(key);
        if let Some(i) = found {
            route.remove(i);
            Some(false)
        } else {
            route.push(pid.to_string());
            Some(true)
        }
    }

    /// Rewrite virtual-printer route slots to the real printer they point at.
    /// Returns whether the file changed.
    pub fn flatten_routes_to_real_printers(&mut self) -> bool {
        let vp_to_printer: std::collections::HashMap<String, String> = self
            .virtual_printers
            .iter()
            .map(|v| (v.id.clone(), v.printer_id.clone()))
            .collect();
        let known: std::collections::HashSet<String> =
            self.printers.iter().map(|p| p.id.clone()).collect();
        let mut changed = !self.virtual_printers.is_empty();
        let rewrite = |slots: &mut Vec<String>, changed: &mut bool| {
            let mut out = Vec::new();
            for slot in slots.iter() {
                let raw = slot.trim();
                let pid = if let Some(p) = vp_to_printer.get(raw) {
                    p.clone()
                } else {
                    decode_route_slot(raw).0
                };
                if pid.is_empty() || !known.contains(&pid) {
                    *changed = true;
                    continue;
                }
                if raw != pid {
                    *changed = true;
                }
                if out.iter().any(|s: &String| s == &pid) {
                    *changed = true;
                    continue;
                }
                out.push(pid);
            }
            *slots = out;
        };
        rewrite(&mut self.routes.bill, &mut changed);
        for slots in self.routes.kot.values_mut() {
            rewrite(slots, &mut changed);
        }
        self.virtual_printers.clear();
        changed
    }

    /// Toggle a virtual printer on a place route (append or remove). Returns whether attached after.
    pub fn toggle_virtual_on_route(&mut self, key: RouteKey, vp_id: &str) -> Option<bool> {
        let vid = vp_id.trim();
        if vid.is_empty() || self.virtual_printer_by_id(vid).is_none() {
            return None;
        }
        let route = self.route_mut(key);
        if let Some(i) = route.iter().position(|s| s.trim() == vid) {
            route.remove(i);
            Some(false)
        } else {
            route.push(vid.to_string());
            Some(true)
        }
    }

    /// Reorder two virtual-printer slots on a place (by index in the route).
    pub fn swap_route_slots(&mut self, key: RouteKey, from: usize, to: usize) -> bool {
        if from == to {
            return false;
        }
        let route = self.route_mut(key);
        if from >= route.len() || to >= route.len() {
            return false;
        }
        route.swap(from, to);
        true
    }

    pub fn virtual_printer_by_id(&self, id: &str) -> Option<&VirtualPrinter> {
        self.virtual_printers.iter().find(|v| v.id == id)
    }

    /// Resolve route slots to a real printer plus the Bill or KOT format.
    /// Layout, delivery, type size, and content width come from that format.
    /// A leftover virtual-printer id still finds its real printer.
    pub fn destinations_for(&self, key: RouteKey) -> Vec<PrintDest> {
        let place_default = self.format_for(key);
        self.route_for(key)
            .iter()
            .filter_map(|slot| {
                let raw = slot.trim();
                if raw.is_empty() {
                    return None;
                }
                let pid = if let Some(vp) = self.virtual_printer_by_id(raw) {
                    vp.printer_id.clone()
                } else {
                    decode_route_slot(raw).0
                };
                let p = self.printers.iter().find(|p| p.id == pid)?;
                Some(PrintDest {
                    target: parse_target(&p.target),
                    format: place_default.clone(),
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


    pub fn printer_by_id(&self, id: &str) -> Option<&NamedPrinter> {
        let (pid, _) = decode_route_slot(id);
        self.printers.iter().find(|p| p.id == pid)
    }

    /// Encode / decode helpers for per-assignment format on a route slot.
    pub fn encode_route_slot(printer_id: &str, format_id: &str) -> String {
        encode_route_slot(printer_id, format_id)
    }

    pub fn decode_route_slot(slot: &str) -> (String, String) {
        decode_route_slot(slot)
    }

    pub fn add_format_from(&mut self, from_id: &str, label: &str) -> String {
        PrintFormat::ensure_builtins(&mut self.formats);
        let base = self.format_by_id(from_id);
        let id = format!("custom-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        let label = {
            let t = label.trim();
            if t.is_empty() { format!("Custom ({})", base.label) } else { t.to_string() }
        };
        self.formats.push(PrintFormat {
            id: id.clone(),
            label,
            document: base.document,
            landscape: base.landscape,
            delivery: base.delivery.clone(),
            em: base.em,
            line: base.line,
            baseline: base.baseline,
            side: base.side,
            place: base.place,
        });
        id
    }

    /// Owner-created config (Printers screen). Defaults: portrait + OS document.
    /// Not a stock “KOT / Portrait / Landscape” type — owner supplies the name.
    pub fn add_owner_config(&mut self, label: &str) -> String {
        PrintFormat::ensure_builtins(&mut self.formats);
        let n = self
            .formats
            .iter()
            .filter(|f| !is_builtin_format(&f.id))
            .count()
            + 1;
        let label = {
            let t = label.trim();
            if t.is_empty() {
                format!("Config {n}")
            } else {
                t.to_string()
            }
        };
        let id = format!("custom-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        self.formats.push(PrintFormat {
            id: id.clone(),
            label,
            document: true,
            landscape: false,
            delivery: DELIVERY_SYSTEM_SPOOLER.into(),
            em: STD_EM_NORMAL,
            line: 40,
            baseline: STD_EM_NORMAL + 2.0,
            side: STD_SIDE,
            place: Default::default(),
        });
        id
    }

    /// Owner configs only (hide legacy built-in ids from the Printers list).
    pub fn owner_configs(&self) -> impl Iterator<Item = &PrintFormat> {
        self.formats.iter().filter(|f| !is_builtin_format(&f.id))
    }

    pub fn update_format(&mut self, format: PrintFormat) {
        if let Some(slot) = self.formats.iter_mut().find(|f| f.id == format.id) {
            *slot = format;
        }
    }

    /// Current Standard type size (portrait preferred; landscape fallback).
    pub fn standard_em(&self) -> f32 {
        self.formats
            .iter()
            .find(|f| f.id == FORMAT_STANDARD_PORTRAIT)
            .or_else(|| self.formats.iter().find(|f| is_standard_document(&f.id)))
            .map(|f| f.em)
            .unwrap_or(STD_EM_NORMAL)
    }

    /// Wrap columns on Standard (bill) formats only.
    pub fn set_bill_content_line(&mut self, cols: u32) {
        let cols = cols.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX);
        for f in self.formats.iter_mut() {
            if is_standard_document(&f.id) {
                f.line = cols;
            }
        }
    }

    /// Set type size on all Standard document formats (portrait + landscape).
    /// Baseline tracks em + 2 so line spacing stays readable.
    pub fn set_standard_em(&mut self, em: f32) {
        // Free font size — clamp only; do not snap to Small/Normal/Large buckets.
        let em = em.clamp(12.0, 48.0);
        let baseline = em + 2.0;
        for f in self.formats.iter_mut() {
            if is_standard_document(&f.id) {
                f.em = em;
                f.baseline = baseline;
            }
        }
    }

    pub fn remove_format(&mut self, id: &str) {
        if id == FORMAT_KOT
            || id == FORMAT_STANDARD
            || id == FORMAT_STANDARD_PORTRAIT
            || id == FORMAT_STANDARD_LANDSCAPE
        {
            return;
        }
        self.formats.retain(|f| f.id != id);
        self.virtual_printers.retain(|v| v.config_id != id);
        if self.routes.bill_format == id {
            self.routes.bill_format = FORMAT_STANDARD_PORTRAIT.into();
        }
        for v in self.routes.kot_format.values_mut() {
            if v == id {
                *v = FORMAT_KOT.into();
            }
        }
    }

    /// Rewrite legacy `@standard` slots and bill_format to portrait.
    pub fn migrate_legacy_standard_ids(&mut self) {
        PrintFormat::ensure_builtins(&mut self.formats);
        if self.routes.bill_format.trim() == FORMAT_STANDARD {
            self.routes.bill_format = FORMAT_STANDARD_PORTRAIT.into();
        }
        let rewrite = |slots: &mut Vec<String>| {
            for slot in slots.iter_mut() {
                let (pid, fmt) = decode_route_slot(slot);
                if fmt == FORMAT_STANDARD {
                    *slot = encode_route_slot(&pid, FORMAT_STANDARD_PORTRAIT);
                }
            }
        };
        rewrite(&mut self.routes.bill);
        for slots in self.routes.kot.values_mut() {
            rewrite(slots);
        }
    }

    /// Same destination = same printer. Target is compared case-insensitively
    /// (CUPS / Windows names); blank target falls back to label.
    ///
    /// System queue names that differ only by `_` vs `-` (Find often lists
    /// both) collapse to one pool entry. Network / BT / USB / Android targets
    /// keep `:` and are not rewritten.
    fn printer_dedupe_key(label: &str, target: &str) -> String {
        let t = target.trim();
        let raw = if !t.is_empty() {
            t
        } else {
            label.trim()
        };
        Self::normalize_dest_key(raw)
    }

    /// Lowercase destination; for bare queue names, treat `_` and `-` alike.
    pub fn normalize_dest_key(raw: &str) -> String {
        let s = raw.trim().to_ascii_lowercase();
        if s.is_empty() {
            return s;
        }
        if s.contains(':') {
            s
        } else {
            s.replace('_', "-")
        }
    }

    /// Display-name key for route uniqueness: same shown name + format must
    /// not appear twice on one place (Standard twice for one Brother is out;
    /// Standard + KOT for the same Brother is fine).
    pub fn route_name_format_key(display_name: &str, format_id: &str) -> String {
        format!(
            "{}@@{}",
            Self::normalize_dest_key(display_name),
            format_id.trim().to_ascii_lowercase()
        )
    }

    /// Add a printer, or return the existing id when the same destination is
    /// already saved. `newly_added` is false for duplicates.
    pub fn ensure_printer(&mut self, label: &str, target: &str) -> (String, bool) {
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
        let key = Self::printer_dedupe_key(&label, &tgt);
        if key.is_empty() {
            return (String::new(), false);
        }
        if let Some(p) = self
            .printers
            .iter()
            .find(|p| Self::printer_dedupe_key(&p.label, &p.target) == key)
        {
            return (p.id.clone(), false);
        }
        let id = uuid::Uuid::new_v4().to_string();
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
        (id, true)
    }

    pub fn add_printer(&mut self, label: &str, target: &str) -> String {
        self.ensure_printer(label, target).0
    }

    /// Compat for call sites that still pass a discarded mode.
    pub fn add_printer_with_mode(&mut self, label: &str, target: &str, _mode: PrinterMode) -> String {
        self.add_printer(label, target)
    }

    /// Collapse duplicate destinations (same target/name) and remap routes.
    /// Also drops a second route slot that is the same shown name + format
    /// (Brother_…@standard and Brother-…@standard → keep the first).
    ///
    /// Virtual-printer place-attach slots (bare VP ids) must be kept as-is —
    /// treating them as printer ids drops every Kitchen-station attach on save.
    pub fn dedupe_printers(&mut self) {
        let mut keep_ids: Vec<String> = Vec::new();
        let mut remap: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut by_key: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut kept: Vec<NamedPrinter> = Vec::new();
        for p in self.printers.drain(..) {
            let key = Self::printer_dedupe_key(&p.label, &p.target);
            if key.is_empty() {
                continue;
            }
            if let Some(keep) = by_key.get(&key) {
                // The kept row takes a paper only the duplicate had: a paper
                // once set is never lost (docs/PRINTER.md § Paper and preview).
                if let Some(k) = kept.iter_mut().find(|k| &k.id == keep) {
                    if k.virtual_paper().is_none() && p.virtual_paper().is_some() {
                        k.virtual_paper_width_mm = p.virtual_paper_width_mm;
                        k.virtual_paper_length_mm = p.virtual_paper_length_mm;
                    }
                }
                remap.insert(p.id, keep.clone());
            } else {
                by_key.insert(key, p.id.clone());
                keep_ids.push(p.id.clone());
                remap.insert(p.id.clone(), p.id.clone());
                kept.push(p);
            }
        }
        self.printers = kept;
        let vp_ids: std::collections::HashSet<String> =
            self.virtual_printers.iter().map(|v| v.id.clone()).collect();
        let map_slot = |slot: &mut String| {
            let raw = slot.trim();
            if vp_ids.contains(raw) {
                return;
            }
            let (pid, fmt) = decode_route_slot(slot);
            if let Some(n) = remap.get(&pid) {
                *slot = encode_route_slot(n, &fmt);
            }
        };
        for slot in &mut self.routes.bill {
            map_slot(slot);
        }
        self.routes.bill.dedup();
        self.routes.bill.retain(|slot| {
            let raw = slot.trim();
            if vp_ids.contains(raw) {
                return true;
            }
            let (pid, _) = decode_route_slot(slot);
            keep_ids.iter().any(|k| k == &pid)
        });
        for ids in self.routes.kot.values_mut() {
            for slot in ids.iter_mut() {
                map_slot(slot);
            }
            ids.dedup();
            ids.retain(|slot| {
                let raw = slot.trim();
                if vp_ids.contains(raw) {
                    return true;
                }
                let (pid, _) = decode_route_slot(slot);
                keep_ids.iter().any(|k| k == &pid)
            });
        }
        let _ = keep_ids;
        self.dedupe_route_name_formats();
    }

    /// On each place, keep one slot per shown name + format.
    fn dedupe_route_name_formats(&mut self) {
        let vp_ids: std::collections::HashSet<String> =
            self.virtual_printers.iter().map(|v| v.id.clone()).collect();
        let default_bill = self.format_id_for(RouteKey::Bill);
        Self::dedupe_slots_by_name_format(
            &mut self.routes.bill,
            &self.printers,
            &vp_ids,
            &default_bill,
        );
    }

    fn dedupe_slots_by_name_format(
        slots: &mut Vec<String>,
        printers: &[NamedPrinter],
        vp_ids: &std::collections::HashSet<String>,
        default_fmt: &str,
    ) {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for slot in slots.drain(..) {
            let raw = slot.trim();
            // Place-attach virtual printers: one slot per VP id, never decode
            // as printer_id@format (that drops every station attach on save).
            if vp_ids.contains(raw) {
                if seen.insert(format!("vp:{raw}")) {
                    out.push(raw.to_string());
                }
                continue;
            }
            // The place owns the format (`destinations_for` ignores any
            // `@format` on a slot), and load keeps bare printer ids. Save the
            // same canonical form so a draft and its reload are equal; the
            // same printer twice on one place is one slot.
            let (pid, _) = decode_route_slot(&slot);
            let fmt = default_fmt.to_string();
            let Some(p) = printers.iter().find(|p| p.id == pid) else {
                continue;
            };
            let name = {
                let t = p.target.trim();
                if !t.is_empty() {
                    t
                } else if !p.label.trim().is_empty() {
                    p.label.trim()
                } else {
                    pid.as_str()
                }
            };
            let nf = Self::route_name_format_key(name, &fmt);
            if seen.insert(nf) {
                out.push(pid);
            }
        }
        *slots = out;
    }

    pub fn remove_printer(&mut self, id: &str) {
        self.printers.retain(|p| p.id != id);
        let removed_vp: std::collections::HashSet<String> = self
            .virtual_printers
            .iter()
            .filter(|v| v.printer_id == id)
            .map(|v| v.id.clone())
            .collect();
        self.virtual_printers.retain(|v| v.printer_id != id);
        let drop_slot = |slot: &String| {
            let raw = slot.trim();
            removed_vp.contains(raw) || decode_route_slot(slot).0 == id
        };
        self.routes.bill.retain(|s| !drop_slot(s));
        for ids in self.routes.kot.values_mut() {
            ids.retain(|s| !drop_slot(s));
        }
    }

    pub fn route_mut(&mut self, key: RouteKey) -> &mut Vec<String> {
        match key {
            RouteKey::Bill => &mut self.routes.bill,
        }
    }
}

/// Where one job goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrinterTarget {
    /// The device's default system printer / Android PrintManager.
    Default,
    /// A system printer by name (desktop CUPS / Windows).
    Named(String),
    /// Network receipt printer `ip:port` (ESC/POS over TCP).
    Network(String),
    /// Bluetooth SPP `bt:AA:BB:CC:DD:EE:FF`.
    Bluetooth(String),
    /// USB host `usb:VID:PID` or `usb:VID:PID:serial`.
    Usb(String),
    /// Android PrintManager Standard path.
    AndroidPrint,
}

/// One place to send a job: address/name plus the owner's mode.
#[derive(Clone, Debug, PartialEq)]
pub struct PrintDest {
    pub target: PrinterTarget,
    pub format: PrintFormat,
    /// The printer's virtual paper (Settings → Printers): where the format's
    /// place is measured and how wide an ESC/POS roll is. Never sent to the
    /// printer.
    pub virtual_paper: Option<crate::os_paper::PaperMm>,
}

/// Whether this format may use OS / Android document print.
pub fn format_allows_document(format: &PrintFormat) -> bool {
    format.document
}

/// Port network receipt printers listen on for raw jobs.
pub const RAW_PRINT_PORT: u16 = 9100;

/// Parse the owner's target string into a transport.
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
    // host:port where host is not a bare IPv4 SocketAddrV4 (hostname)
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

/// Whether this device can auto-print to `target` for the given mode.
pub fn can_print_format(target: &PrinterTarget, format: &PrintFormat) -> bool {
    match target {
        PrinterTarget::Network(_) | PrinterTarget::Bluetooth(_) | PrinterTarget::Usb(_) => true,
        PrinterTarget::AndroidPrint => format.document && cfg!(target_os = "android"),
        PrinterTarget::Default | PrinterTarget::Named(_) => {
            if cfg!(target_os = "android") {
                format.document
            } else {
                true
            }
        }
    }
}

/// Whether this device can print to `target` with some built-in format.
pub fn can_print(target: &PrinterTarget) -> bool {
    can_print_format(target, &PrintFormat::standard())
        || can_print_format(target, &PrintFormat::kot())
}

pub fn can_print_mode(target: &PrinterTarget, mode: PrinterMode) -> bool {
    let f = match mode {
        PrinterMode::Standard => PrintFormat::standard(),
        PrinterMode::Kot => PrintFormat::kot(),
    };
    can_print_format(target, &f)
}

fn default_route_for(kind: PrintKind) -> RouteKey {
    match kind {
        PrintKind::Bill => RouteKey::Bill,
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
    #[cfg(test)]
    if TEST_PRINTERS_OVERRIDE.with(|o| o.borrow().is_some()) {
        TEST_PRINTERS_OVERRIDE.with(|o| *o.borrow_mut() = Some(printers));
        return;
    }
    *PRINTERS.lock().unwrap() = Some(printers);
}

/// The printers in force on this device.
pub fn printers() -> DevicePrinters {
    #[cfg(test)]
    if let Some(p) = TEST_PRINTERS_OVERRIDE.with(|o| o.borrow().clone()) {
        return p;
    }
    PRINTERS.lock().unwrap().clone().unwrap_or_default()
}

#[cfg(test)]
thread_local! {
    /// [`with_test_printers`] config for this thread only — other test
    /// threads rewrite the global `PRINTERS` without the test lock
    /// (`use_device_dir`, app-state reloads). Same fix as Khaynjo.
    static TEST_PRINTERS_OVERRIDE: std::cell::RefCell<Option<DevicePrinters>> =
        const { std::cell::RefCell::new(None) };
}

fn printers_path(base: &Path) -> PathBuf {
    base.join("printers.json")
}

/// Directory that owns `printers.json` for these books (`…/servers/<id>/` or
/// `…/client/`). Never the install root — each shop's printers stay separate.
pub fn printers_dir_for_books(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

/// Printers saved beside these books (none saved = the defaults).
pub fn load_device_printers(base: &Path) -> DevicePrinters {
    let mut printers: DevicePrinters = fs::read_to_string(printers_path(base))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    printers.migrate_legacy_standard_ids();
    printers.dedupe_printers();
    printers.flatten_routes_to_real_printers();
    printers
}

/// Keep `printers` beside these books and use them from now on.
pub fn save_device_printers(base: &Path, printers: &DevicePrinters) -> Result<(), String> {
    configure(write_device_printers(base, printers)?);
    Ok(())
}

/// Write `printers.json` for `base` without changing the setup in force.
/// For a one-off file migration read on a print path (possibly the LAN
/// thread): replacing the live setup there swapped in the file's copy in
/// the middle of a print and could undo a Settings change.
pub fn write_device_printers(base: &Path, printers: &DevicePrinters) -> Result<DevicePrinters, String> {
    let mut printers = printers.clone();
    printers.dedupe_printers();
    PrintFormat::ensure_builtins(&mut printers.formats);
    fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(&printers).map_err(|e| e.to_string())?;
    fs::write(printers_path(base), raw).map_err(|e| e.to_string())?;
    Ok(printers)
}

/// Load and use printers for `base` (a books folder, or install root before
/// any shop is open).
pub fn use_device_dir(base: &Path) {
    configure(load_device_printers(base));
}

/// Open the printers that belong to `db_path`'s shop.
pub fn use_books_printers(db_path: &Path) {
    use_device_dir(&printers_dir_for_books(db_path));
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
    /// True when the job went to a printer's queue (in tests: was captured).
    /// False where this device cannot print it (Android with no network
    /// printer: the caller offers the share sheet).
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
    // The file is for people (opened, shared, kept). Print jobs carry the
    // marked text themselves.
    fs::write(&path, strip_kot_style_marks(text)).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Write the slip, then send it along the default route for `kind`.
pub fn print_text(kind: PrintKind, text: &str, app_files: Option<&Path>) -> Result<ReceiptShare, String> {
    print_route(kind, default_route_for(kind), text, app_files)
}

/// Settings → Printers → Print a test page: one selected device only.
/// Uses the Bill place format for layout/delivery. Never fans out to every
/// printer on a place — that was the "all or none" trap on the pool screen.
pub fn print_one_printer(
    cfg: &DevicePrinters,
    printer_id: &str,
    kind: PrintKind,
    text: &str,
    app_files: Option<&Path>,
) -> Result<ReceiptShare, String> {
    let path = write_receipt(text, app_files)?;
    let Some(p) = cfg.printer_by_id(printer_id) else {
        return Ok(ReceiptShare {
            path,
            opened: false,
        });
    };
    let format = cfg.format_for(RouteKey::Bill);
    let dest = PrintDest {
        target: parse_target(&p.target),
        format,
        virtual_paper: p.virtual_paper(),
    };
    let opened = match sink() {
        PrintSink::Os => {
            if !can_print_format(&dest.target, &dest.format) {
                false
            } else {
                match queue(Job {
                    dests: vec![dest],
                    mode: RoutePrintMode::First,
                    path: path.clone(),
                    text: text.to_string(),
                    copies: 1,
                    done: confirm_channel(),
                }) {
                    Ok(()) => true,
                    Err(e) => {
                        note_problem(e);
                        false
                    }
                }
            }
        }
        PrintSink::Capture => {
            CAPTURED.with(|c| {
                c.borrow_mut().push(PrintJob {
                    kind,
                    path: path.clone(),
                    text: text.to_string(),
                })
            });
            true
        }
        PrintSink::Off => false,
    };
    Ok(ReceiptShare { path, opened })
}

/// Write the slip, then try the ordered printers on `route` (fallback, not fan-out).
/// Empty route: file is written, nothing is queued; KOTs get a one-line note.
///
/// How each printer is driven is that printer's mode (Settings), not Bill vs KOT.
///
/// `opened` means the job was handed to the print worker (or captured in tests).
/// The worker talks to CUPS / the device off the UI thread; a later failure
/// surfaces via [`take_problems`]. Never treat a dead worker as success.
pub fn print_route(
    kind: PrintKind,
    route: RouteKey,
    text: &str,
    app_files: Option<&Path>,
) -> Result<ReceiptShare, String> {
    print_route_copies(kind, route, text, app_files, 1)
}

/// Like [`print_route`], but the worker sends `copies` identical slips
/// back-to-back (KOT multi-copy) without re-writing the receipt file or
/// inserting artificial delays between copies.
pub fn print_route_copies(
    kind: PrintKind,
    route: RouteKey,
    text: &str,
    app_files: Option<&Path>,
    copies: u32,
) -> Result<ReceiptShare, String> {
    print_route_using(&printers(), kind, route, text, app_files, copies)
}

/// [`print_route_copies`] against a given printer setup. Test prints and
/// kitchen tickets adjust a copy of the shop's setup for one job. Writing
/// that copy into the shared setup and back raced the LAN thread and a
/// Settings save: either one could put back an older setup.
pub fn print_route_using(
    cfg: &DevicePrinters,
    kind: PrintKind,
    route: RouteKey,
    text: &str,
    app_files: Option<&Path>,
    copies: u32,
) -> Result<ReceiptShare, String> {
    let path = write_receipt(text, app_files)?;
    let copies = copies.max(1);
    let opened = match sink() {
        PrintSink::Os => {
            let dests: Vec<PrintDest> = cfg
                .destinations_for(route)
                .into_iter()
                .filter(|d| can_print_format(&d.target, &d.format))
                .collect();
            if dests.is_empty() {
                false
            } else {
                let mode = cfg.print_mode_for(route);
                match queue(Job {
                    dests,
                    mode,
                    path: path.clone(),
                    text: text.to_string(),
                    copies,
                    done: confirm_channel(),
                }) {
                    Ok(()) => true,
                    Err(e) => {
                        note_problem(e);
                        false
                    }
                }
            }
        }
        PrintSink::Capture => {
            // Prefer the resolved route (station → "" → Bill), not only the
            // exact key — same as Os path via destinations_for. A captured job
            // counts as handed to a printer, so callers take the same path
            // they would on a device with that printer.
            if !cfg.route_for(route).is_empty() {
                CAPTURED.with(|c| {
                    c.borrow_mut().push(PrintJob {
                        kind,
                        path: path.clone(),
                        text: text.to_string(),
                    })
                });
                true
            } else {
                false
            }
        }
        PrintSink::Off => false,
    };
    Ok(ReceiptShare { path, opened })
}

struct Job {
    /// Ordered destinations for this place.
    dests: Vec<PrintDest>,
    /// First success vs send to every destination.
    mode: RoutePrintMode,
    path: PathBuf,
    text: String,
    /// Identical slips to send (KOT copies). Worker sends back-to-back.
    copies: u32,
    /// Someone is waiting for this job's paper (see [`ConfirmPaper`]): the
    /// worker answers with every printer that did not take it.
    done: Option<mpsc::Sender<Vec<String>>>,
}

thread_local! {
    /// Jobs queued on this thread while a [`ConfirmPaper`] is open.
    static CONFIRMING: RefCell<Option<Vec<mpsc::Receiver<Vec<String>>>>> = const { RefCell::new(None) };
}

/// Wait for the paper of every job queued on this thread, so a Server can
/// tell the device that asked whether the kitchen or the bill printer really
/// took it. Only the LAN thread opens one — after the books committed, never
/// on the UI thread (a dead printer would freeze the till for seconds).
///
/// Jobs still print exactly as before; the Server's own screen still hears a
/// failure through [`take_problems`].
pub struct ConfirmPaper(());

impl ConfirmPaper {
    pub fn start() -> Self {
        CONFIRMING.with(|c| *c.borrow_mut() = Some(Vec::new()));
        Self(())
    }

    /// Every printer that did not take a job, plus a "still printing" note
    /// for jobs the printer has not finished within `wait`. Empty = all
    /// paper confirmed (or nothing was printed).
    pub fn finish(self, wait: Duration) -> Vec<String> {
        let pending = CONFIRMING.with(|c| c.borrow_mut().take()).unwrap_or_default();
        let deadline = std::time::Instant::now() + wait;
        let mut problems: Vec<String> = Vec::new();
        let mut slow = 0usize;
        for rx in pending {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            match rx.recv_timeout(left) {
                Ok(errs) => {
                    for e in errs {
                        if !problems.contains(&e) {
                            problems.push(e);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => slow += 1,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    problems.push("print worker stopped — restart the shop device, then print again".into());
                }
            }
        }
        if slow > 0 {
            problems.push(format!(
                "{slow} slip(s) still printing — the shop device says if a printer fails"
            ));
        }
        problems
    }
}

impl Drop for ConfirmPaper {
    fn drop(&mut self) {
        CONFIRMING.with(|c| *c.borrow_mut() = None);
    }
}

/// A done channel for this job when the thread is confirming paper.
fn confirm_channel() -> Option<mpsc::Sender<Vec<String>>> {
    CONFIRMING.with(|c| {
        let mut slot = c.borrow_mut();
        let waiting = slot.as_mut()?;
        let (tx, rx) = mpsc::channel();
        waiting.push(rx);
        Some(tx)
    })
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

/// Say a printing problem on this device's screen (the same channel the
/// print worker uses), e.g. a waiter's bill with no bill printer here.
pub fn report_problem(problem: String) {
    note_problem(problem);
}

/// Printing problems since the last call (oldest first).
pub fn take_problems() -> Vec<String> {
    std::mem::take(&mut *PROBLEMS.lock().unwrap())
}

/// One thread prints, in the order the jobs came.
/// Returns Err when the worker cannot be started or has stopped — callers
/// must not show "printed" / "test page sent" in that case.
fn queue(job: Job) -> Result<(), String> {
    static TX: Mutex<Option<mpsc::Sender<Job>>> = Mutex::new(None);

    fn ensure_worker(slot: &mut Option<mpsc::Sender<Job>>) -> Result<(), String> {
        if slot.is_some() {
            return Ok(());
        }
        let (tx, rx) = mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name("print".into())
            .spawn(move || {
                for job in rx {
                    // One send with N identical slips in one payload — do not
                    // open N TCP/`lp` sessions (that made copy 2/3 feel delayed).
                    let copies = job.copies.max(1);
                    let mut failed: Vec<String> = Vec::new();
                    match job.mode {
                        RoutePrintMode::First => {
                            let mut last_err: Option<(String, String)> = None;
                            for dest in &job.dests {
                                match send_to_printer(dest, &job.path, &job.text, copies) {
                                    Ok(()) => {
                                        last_err = None;
                                        break;
                                    }
                                    Err(e) => last_err = Some((describe(&dest.target), e)),
                                }
                            }
                            if let Some((who, e)) = last_err {
                                failed.push(format!("{who} did not print: {e}"));
                            }
                        }
                        RoutePrintMode::All => {
                            // Every printer on the place gets the slip; each
                            // one that did not take it is named.
                            for dest in &job.dests {
                                if let Err(e) = send_to_printer(dest, &job.path, &job.text, copies) {
                                    failed.push(format!(
                                        "{} did not print: {e}",
                                        describe(&dest.target)
                                    ));
                                }
                            }
                        }
                    }
                    for problem in &failed {
                        note_problem(problem.clone());
                    }
                    if let Some(done) = job.done {
                        // Nobody waiting any more (timed out) is fine.
                        let _ = done.send(failed);
                    }
                }
            })
            .map_err(|e| format!("print worker failed to start ({e})"))?;
        *slot = Some(tx);
        Ok(())
    }

    let mut slot = TX.lock().unwrap();
    ensure_worker(&mut slot)?;
    match slot.as_ref().unwrap().send(job) {
        Ok(()) => Ok(()),
        Err(mpsc::SendError(job)) => {
            // Worker died (panic / exit). Start a fresh one and retry once.
            *slot = None;
            ensure_worker(&mut slot)?;
            slot.as_ref()
                .unwrap()
                .send(job)
                .map_err(|_| {
                    "print worker stopped — restart the app, then try again".into()
                })
        }
    }
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

fn send_to_printer(dest: &PrintDest, path: &Path, text: &str, copies: u32) -> Result<(), String> {
    let copies = copies.max(1);
    let (format, target) = (&dest.format, &dest.target);
    // Delivery is the format's own flag. Bill text and kitchen text use the
    // same sender; neither one is tied to a page printer or a thermal head.
    let as_page = format.document;
    let thermal = || replicate_escpos_copies(&escpos_job_placed(format, text, dest.virtual_paper), copies);
    match target {
        PrinterTarget::Network(addr) => print_network_bytes(addr, &thermal()),
        PrinterTarget::Bluetooth(mac) => crate::print_bt::write_bytes(mac, &thermal()),
        PrinterTarget::Usb(id) => crate::print_usb::write_bytes(id, &thermal()),
        PrinterTarget::AndroidPrint => {
            if !as_page {
                return Err(format!(
                    "format \"{}\" needs Network, Bluetooth, or USB — Android print is for document formats",
                    format.label
                ));
            }
            // Document path: OS copies count when available; else repeat.
            for _ in 0..copies {
                print_android_document(text, format, dest.virtual_paper)?;
            }
            Ok(())
        }
        PrinterTarget::Default => {
            if as_page {
                print_system_standard(None, path, text, format, copies, dest)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB — not the system print service",
                    format.label
                ))
            } else {
                print_system_raw(None, &thermal())
            }
        }
        PrinterTarget::Named(name) => {
            if as_page {
                print_system_standard(Some(name), path, text, format, copies, dest)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB — not a system printer name",
                    format.label
                ))
            } else {
                print_system_raw(Some(name), &thermal())
            }
        }
    }
}

/// N identical ESC/POS slips back-to-back (each already ends with cut).
fn replicate_escpos_copies(one: &[u8], copies: u32) -> Vec<u8> {
    let n = copies.max(1) as usize;
    if n == 1 {
        return one.to_vec();
    }
    let mut out = Vec::with_capacity(one.len().saturating_mul(n));
    for _ in 0..n {
        out.extend_from_slice(one);
    }
    out
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
        let metrics = format.raster_metrics();
        let (png, _, _) = slip_page_png(text, &metrics, false)?;
        crate::print_android::print_slip(
            "Tabiyat",
            &png,
            os_document_content_mm(&metrics),
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
    let metrics = format.raster_metrics();
    let (px, w, h) = slip_page_gray(text, &metrics, false);
    let slip = image::GrayImage::from_raw(w, h, px).ok_or("the slip could not be drawn")?;
    Ok((slip, os_document_content_mm(&metrics)))
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
        crate::print_win::print_pages(name, "Tabiyat", copies.max(1), None, |own| {
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

/// Network `:9100` — ESC/POS slip (KOT metrics by default).
pub fn print_network(addr: &str, text: &str) -> Result<(), String> {
    print_network_format(addr, &PrintFormat::kot(), text)
}

pub fn print_network_format(addr: &str, format: &PrintFormat, text: &str) -> Result<(), String> {
    print_network_bytes(addr, &escpos_job_format(format, text))
}

pub fn print_network_mode(addr: &str, mode: PrinterMode, text: &str) -> Result<(), String> {
    let f = match mode {
        PrinterMode::Standard => PrintFormat::standard(),
        PrinterMode::Kot => PrintFormat::kot(),
    };
    print_network_format(addr, &f, text)
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
/// - Slips the built-in font can show (ASCII + ₹/dashes spelt out) go as
///   **plain text** — LF only, never CR (CR overstrike → last line +
///   `_`/`-` smear on many heads).
/// - Scripts the head has no letters for (Gujarati, Hindi, …) go as a
///   **raster picture** (`GS v 0`) drawn with Noto.
///
/// Always-raster used to dump binary as garbage (`_____`, `-----`, cut-off
/// letters) on heads that do not speak `GS v 0` well. Prefer text when safe.
pub fn escpos_job(text: &str) -> Vec<u8> {
    escpos_job_format(&PrintFormat::kot(), text)
}

pub fn escpos_job_format(format: &PrintFormat, text: &str) -> Vec<u8> {
    let (drawn, width) = escpos_drawn(format, text);
    crate::receipt_raster::escpos_from_drawn(drawn, width)
}

/// The slip as a picture — the same picture as the preview — drawn 1-bit
/// for standard ESC/POS raster (`GS v 0`), whatever the text or the
/// printer; and its width in dots.
fn escpos_drawn(format: &PrintFormat, text: &str) -> ((Vec<u8>, usize), usize) {
    let cleaned = text.replace('\r', "");
    let metrics = format.raster_metrics();
    let drawn = if cleaned.contains(KOT_STYLE_MARK) {
        crate::receipt_raster::draw_lines(&styled_slip_lines(&cleaned, &metrics), &metrics)
    } else {
        crate::receipt_raster::draw_metrics(&strip_kot_style_marks(&cleaned), &metrics)
    };
    (drawn, metrics.page_dots())
}

/// [`escpos_job_format`] at the format's place across the printer's roll
/// (`roll`: its virtual paper, when set; a roll is as long as the slip, so
/// only across counts — `os_paper::place_in`). A slip at the left — where
/// every slip sat before places existed, and still does unless the owner
/// picks another — is sent byte for byte as before.
pub fn escpos_job_placed(format: &PrintFormat, text: &str, roll: Option<crate::os_paper::PaperMm>) -> Vec<u8> {
    let (drawn, width) = escpos_drawn(format, text);
    let Some(roll) = roll else {
        return crate::receipt_raster::escpos_from_drawn(drawn, width);
    };
    let roll_dots = (roll.width_mm as f32 * crate::receipt_raster::DOTS_PER_MM).round().max(8.0) as usize;
    let (x, _) = crate::os_paper::place_in(format.place, width as u32, 0, roll_dots as u32, 0, false);
    if x == 0 {
        return crate::receipt_raster::escpos_from_drawn(drawn, width);
    }
    let placed = crate::receipt_raster::place_drawn(drawn, width, roll_dots, x);
    crate::receipt_raster::escpos_from_drawn(placed, roll_dots)
}

/// Millimetres across the raster. Dots at or under a 58 mm head are 58 mm.
pub fn content_mm_for_dots(dots: usize) -> u32 {
    if dots <= 384 {
        58
    } else {
        ((dots as u32) * 80 / 576).clamp(40, 210)
    }
}

/// The slip picture the preview and the document printer both use.
/// Landscape turns that picture so the content width is the vertical side.
pub fn slip_page_png(
    text: &str,
    metrics: &crate::receipt_raster::RasterMetrics,
    landscape: bool,
) -> Result<(Vec<u8>, u32, u32), String> {
    let png = crate::receipt_raster::png_lines(&styled_slip_lines(text, metrics), metrics)?;
    let (mut w, mut h) = png_pixel_size(&png).unwrap_or((1, 1));
    if !landscape {
        return Ok((png, w, h));
    }
    let img = image::load_from_memory(&png).map_err(|e| e.to_string())?;
    let turned = img.rotate90();
    w = turned.width();
    h = turned.height();
    let mut buf = std::io::Cursor::new(Vec::new());
    turned
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok((buf.into_inner(), w, h))
}

/// [`slip_page_png`] as grey pixels (one byte each) for the on-screen
/// preview: same page, same quarter turn, no PNG round trip.
pub fn slip_page_gray(
    text: &str,
    metrics: &crate::receipt_raster::RasterMetrics,
    landscape: bool,
) -> (Vec<u8>, u32, u32) {
    let (px, w, h) = crate::receipt_raster::gray_lines(&styled_slip_lines(text, metrics), metrics);
    if !landscape {
        return (px, w, h);
    }
    // `image`'s rotate90 (clockwise): old (x, y) lands on (h - 1 - y, x).
    let (ow, oh) = (w as usize, h as usize);
    let mut out = vec![255u8; px.len()];
    for y in 0..oh {
        let src = &px[y * ow..(y + 1) * ow];
        for (x, &v) in src.iter().enumerate() {
            if v != 255 {
                out[x * oh + (oh - 1 - y)] = v;
            }
        }
    }
    (out, h, w)
}

/// Identity of one slip preview picture: same text, size, and turn → the
/// same pixels, so the preview can skip drawing it again.
pub fn slip_preview_key(
    text: &str,
    metrics: &crate::receipt_raster::RasterMetrics,
    landscape: bool,
    type_px: f64,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    format!("{metrics:?}").hash(&mut h);
    landscape.hash(&mut h);
    type_px.to_bits().hash(&mut h);
    h.finish()
}

fn png_pixel_size(png: &[u8]) -> Option<(u32, u32)> {
    if png.len() < 24 {
        return None;
    }
    Some((
        u32::from_be_bytes([png[16], png[17], png[18], png[19]]),
        u32::from_be_bytes([png[20], png[21], png[22], png[23]]),
    ))
}

/// One scale for the whole print picture on screen.
///
/// `column` is the settings width. `viewport` is how tall the picture may be
/// before the page scrolls. `type_px` is the body type inside the picture.
/// The same scale is applied to width, height, type, and margins.
///
/// A print that fits at a readable size is fitted. A print that would become
/// unreadable if fitted to the viewport is held at a readable size instead,
/// still no wider than the column, and the page scrolls for the rest.
/// Body-type height in the print picture, in pixels.
pub fn preview_type_px(em: f32, width_dots: usize) -> f64 {
    crate::receipt_raster::drawn_type_px(em, width_dots)
}

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

/// `base` supplies side margin and type size. Width is the content millimetres.
pub fn metrics_at_content_mm(
    base: &PrintFormat,
    mm: u32,
) -> crate::receipt_raster::RasterMetrics {
    let mut m = base.raster_metrics();
    // Same dot width the printer uses for this millimetre setting, so the
    // preview is a scale of the printed slip rather than a second layout.
    m.width_dots = paper_width_dots(mm);
    m.baseline = m.em * 0.92;
    m
}

/// Row pitch (dots) for a type size: the preview's and the printer's.
pub fn raster_pitch_for_em(em: f32) -> usize {
    ((em * 1.45).round() as usize).max(1)
}

/// One slip line's size, weight, and alignment. Start/Center/End are shifts
/// across `default.width_dots`, which is the selected content width.
/// A leading [`SLIP_TAB_MARK`] directive (from the bill/KOT format) sets
/// `\t` column gap / min widths for every tab line on the slip.
pub fn styled_slip_lines(
    text: &str,
    default: &crate::receipt_raster::RasterMetrics,
) -> Vec<crate::receipt_raster::StyledLine> {
    let (tab, body) = take_slip_tab_layout(text);
    body.lines()
        .filter(|line| {
            if line.starts_with(SLIP_TAB_MARK) {
                return false;
            }
            let (_, body) = take_kot_line_style(line);
            !body.trim().is_empty()
        })
        .map(|line| {
            let (style, body) = take_kot_line_style(line);
            let (em, weight, align, strike) = raster_line_style(style.as_ref(), default.em);
            // A pair line carries its second field's own mark after the tab.
            if let Some((first, rest)) = body.split_once('\t')
                && rest.starts_with(KOT_STYLE_MARK)
            {
                let (second, text) = take_kot_line_style(rest);
                let (em2, weight2, align2, _) = raster_line_style(second.as_ref(), default.em);
                return crate::receipt_raster::StyledLine {
                    text: first.to_string(),
                    em,
                    weight,
                    align,
                    strike,
                    tab: Some(crate::receipt_raster::LineColumns::Pair(
                        crate::receipt_raster::PairHalf {
                            text: text.to_string(),
                            em: em2,
                            weight: weight2,
                            align: align2,
                        },
                    )),
                };
            }
            crate::receipt_raster::StyledLine {
                text: body.to_string(),
                em,
                weight,
                align,
                strike,
                tab: if body.contains('\t') { tab.clone() } else { None },
            }
        })
        .collect()
}

/// A line mark as the raster's size, weight, alignment and strike.
fn raster_line_style(
    style: Option<&KotLineStyle>,
    default_em: f32,
) -> (
    f32,
    crate::receipt_raster::RasterWeight,
    crate::receipt_raster::LineAlign,
    bool,
) {
    match style {
        Some(style) => (
            style.size_em as f32,
            match style.weight {
                KotFontWeight::Light => crate::receipt_raster::RasterWeight::Light,
                KotFontWeight::Bold => crate::receipt_raster::RasterWeight::Bold,
                KotFontWeight::Regular => crate::receipt_raster::RasterWeight::Regular,
            },
            style.align.line_align(),
            style.strike,
        ),
        None => (
            default_em,
            crate::receipt_raster::RasterWeight::Regular,
            crate::receipt_raster::LineAlign::Start,
            false,
        ),
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
/// Marks cancelled immediately and wakes the UI — do not wait for the /24
/// scan join, or Cancel feels dead while Looking… stays up.
pub fn cancel_find_printers() {
    FIND_CANCEL.store(true, std::sync::atomic::Ordering::SeqCst);
    FIND_WAS_CANCELLED.store(true, std::sync::atomic::Ordering::SeqCst);
    if let Some(hook) = *NOTIFY.lock().unwrap() {
        hook();
    }
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

fn publish_found_partial(found: &[String]) {
    let mut sorted = found.to_vec();
    sorted.sort();
    sorted.dedup();
    *FOUND.lock().unwrap() = Some(sorted);
    if let Some(hook) = *NOTIFY.lock().unwrap() {
        hook();
    }
}

/// Look for printers on a worker thread (system list + this Wi‑Fi's raw
/// port-[`RAW_PRINT_PORT`] listeners). Works on Linux, macOS, Windows, and
/// Android (network scan only there).
///
/// **Publishes as soon as system queues are known**, then again after the
/// network/BT/USB passes — so Settings is not stuck on Looking… for the
/// whole /24 scan. Only call from an explicit Find tap.
pub fn find_printers_in_background() {
    if LOOKING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    FIND_CANCEL.store(false, std::sync::atomic::Ordering::SeqCst);
    FIND_WAS_CANCELLED.store(false, std::sync::atomic::Ordering::SeqCst);
    // Drop the previous Find result immediately so the UI cannot keep
    // showing an old/partial list (or look like the saved pool) mid-search.
    *FOUND.lock().unwrap() = None;
    // Every driver is asked afresh by a Find: what a printer reported
    // before is not shown as what it reports now.
    crate::os_paper::forget_all();
    if let Some(hook) = *NOTIFY.lock().unwrap() {
        hook();
    }
    if std::thread::Builder::new()
        .name("find-printers".into())
        .spawn(|| {
            let mut found = system_printers();
            // System names first — do not wait for the /24 scan.
            publish_found_partial(&found);
            if !find_cancelled() {
                for addr in network_raw_printers() {
                    if !found.iter().any(|n| n == &addr) {
                        found.push(addr);
                    }
                }
                publish_found_partial(&found);
            }
            // Skip BT/USB/Android service probes once Cancel was tapped —
            // bluetoothctl / PnP can block for seconds with no cancel check.
            if !find_cancelled() {
                for bt in crate::print_bt::list_printers() {
                    if find_cancelled() {
                        break;
                    }
                    if !found.iter().any(|n| n == &bt) {
                        found.push(bt);
                    }
                }
                publish_found_partial(&found);
            }
            if !find_cancelled() {
                for usb in crate::print_usb::list_printers() {
                    if find_cancelled() {
                        break;
                    }
                    if !found.iter().any(|n| n == &usb) {
                        found.push(usb);
                    }
                }
                publish_found_partial(&found);
            }
            #[cfg(target_os = "android")]
            if !find_cancelled() {
                for a in crate::print_android::list_print_services() {
                    if find_cancelled() {
                        break;
                    }
                    if !found.iter().any(|n| n == &a) {
                        found.push(a);
                    }
                }
                publish_found_partial(&found);
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
            // Keep an earlier Cancel mark — do not clear FIND_WAS_CANCELLED.
            if cancelled {
                FIND_WAS_CANCELLED.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            publish_found_partial(&found);
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
/// Bounded worker pool (not one thread per chunk) so Cancel can stop the
/// scan without thrashing the NIC / thread table. Each connect is short
/// and checks the cancel flag between hosts.
fn network_raw_printers() -> Vec<String> {
    use std::net::{SocketAddr, TcpStream, UdpSocket};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    if find_cancelled() {
        return Vec::new();
    }
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
    let addrs: Arc<Vec<SocketAddr>> = Arc::new(
        (1u8..=254)
            .filter(|h| *h != o[3])
            .map(|h| SocketAddr::from(([o[0], o[1], o[2], h], RAW_PRINT_PORT)))
            .collect(),
    );
    let next = Arc::new(AtomicUsize::new(0));
    let found = Arc::new(Mutex::new(Vec::new()));
    // 16 workers × ~16 hosts × 50ms ≈ 0.8s worst case; Cancel aborts the
    // next host pick so joins finish within one connect timeout.
    const WORKERS: usize = 16;
    let workers: Vec<_> = (0..WORKERS)
        .map(|_| {
            let addrs = Arc::clone(&addrs);
            let next = Arc::clone(&next);
            let found = Arc::clone(&found);
            std::thread::spawn(move || {
                loop {
                    if find_cancelled() {
                        break;
                    }
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    let Some(addr) = addrs.get(i) else {
                        break;
                    };
                    if TcpStream::connect_timeout(addr, Duration::from_millis(50)).is_ok() {
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
            return clip.set_text(strip_kot_style_marks(text)).is_ok();
        }
    }
    let _ = text;
    false
}

/// `configure` / Capture are process-global. Must live at module scope (not
/// inside a generic fn) so every caller shares one mutex — a `static` inside
/// a generic is per-monomorphization and would not serialise tests.
#[cfg(test)]
static TEST_PRINTERS_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` with these printers configured; restore defaults afterwards.
#[cfg(test)]
pub fn with_test_printers<R>(cfg: DevicePrinters, f: impl FnOnce() -> R) -> R {
    let _g = TEST_PRINTERS_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let prev = TEST_PRINTERS_OVERRIDE.with(|o| o.borrow_mut().replace(cfg));
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    TEST_PRINTERS_OVERRIDE.with(|o| *o.borrow_mut() = prev);
    match out {
        Ok(v) => v,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_preview_pixels_are_the_printed_png() {
        // The preview skips the PNG round trip; it must still be the print picture.
        let mut f = PrintFormat::standard_portrait();
        f.line = wrap_cols_for_paper_mm(80);
        let text = "Shop\n**Bill 12**\nButter Roti\t2\nDal makhani\t1\n";
        let metrics = f.raster_metrics();
        for landscape in [false, true] {
            let (png, pw, ph) = slip_page_png(text, &metrics, landscape).expect("png");
            let (gray, gw, gh) = slip_page_gray(text, &metrics, landscape);
            assert_eq!((gw, gh), (pw, ph), "landscape={landscape}");
            let decoded = image::load_from_memory(&png).expect("decode").to_luma8();
            assert_eq!(decoded.as_raw(), &gray, "landscape={landscape}");
        }
        let a = slip_preview_key(text, &metrics, false, 20.0);
        assert_eq!(a, slip_preview_key(text, &metrics, false, 20.0));
        assert_ne!(a, slip_preview_key(text, &metrics, true, 20.0));
        assert_ne!(a, slip_preview_key("Shop\n", &metrics, false, 20.0));
    }

    #[test]
    fn unit_tests_never_reach_the_os_printer() {
        let mut cfg = DevicePrinters::default();
        let counter = cfg.add_printer("Counter", "192.168.1.50");
        *cfg.route_mut(RouteKey::Bill) = vec![counter];
        with_test_printers(cfg, || {
            assert_eq!(sink(), PrintSink::Capture);
            let dir = crate::test_dirs::run_root().join(format!("tabiyat-print-{}", std::process::id()));
            let _ = take_problems();
            let _ = take_captured();
            let share = print_text(PrintKind::Bill, "Bill — test", Some(&dir)).unwrap();
            assert!(share.opened, "captured as if a printer took it");
            let jobs = take_captured();
            assert_eq!(jobs.len(), 1);
            // Capture / Named keep the slip as written; Network ESC/POS
            // sanitizes inside escpos_job (see receipt_printer_text_is_plain).
            assert_eq!(jobs[0].text, "Bill — test");
            assert_eq!(jobs[0].kind, PrintKind::Bill);
            assert!(take_captured().is_empty(), "taken once");
            let _ = std::fs::remove_dir_all(dir);
        });
    }

    #[test]
    fn test_page_prints_only_the_selected_printer() {
        let mut cfg = DevicePrinters::default();
        let a = cfg.add_printer("A", "192.168.1.10");
        let b = cfg.add_printer("B", "192.168.1.11");
        *cfg.route_mut(RouteKey::Bill) = vec![a.clone(), b.clone()];
        with_test_printers(cfg.clone(), || {
            let _ = take_captured();
            let dir = crate::test_dirs::run_root()
                .join(format!("tabiyat-one-prn-{}", uuid::Uuid::new_v4().simple()));
            let share = print_one_printer(
                &cfg,
                &a,
                PrintKind::Bill,
                "test one",
                Some(&dir),
            )
            .unwrap();
            assert!(share.opened);
            let jobs = take_captured();
            assert_eq!(jobs.len(), 1, "must not fan out to every Bill printer");
            assert_eq!(jobs[0].text, "test one");
            let _ = fs::remove_dir_all(dir);
        });
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



    #[test]
    fn route_picks_format_printers_are_destinations_only() {
        let mut cfg = DevicePrinters::default();
        let kitchen = cfg.add_printer("Kitchen", "EPSON_TM");
        let office = cfg.add_printer("Office", "Canon_G3010");
        cfg.routes.bill = vec![kitchen.clone(), office.clone()];
        assert_eq!(cfg.format_id_for(RouteKey::Bill), FORMAT_STANDARD_PORTRAIT);
        let bill = cfg.destinations_for(RouteKey::Bill);
        assert_eq!(bill[0].format.id, FORMAT_STANDARD_PORTRAIT);
        assert_eq!(bill[1].format.id, FORMAT_STANDARD_PORTRAIT);
        cfg.set_route_format(RouteKey::Bill, FORMAT_KOT);
        assert_eq!(cfg.destinations_for(RouteKey::Bill)[0].format.id, FORMAT_KOT);
        let custom = cfg.add_format_from(FORMAT_STANDARD_PORTRAIT, "Wide bill");
        cfg.set_route_format(RouteKey::Bill, &custom);
        assert_eq!(cfg.format_id_for(RouteKey::Bill), custom);
    }

    #[test]
    fn collapse_and_place_profile_keep_one_output_per_printer() {
        let mut cfg = DevicePrinters::default();
        let a = cfg.add_printer("Brother", "Brother_QL");
        cfg.routes.bill = vec![
            encode_route_slot(&a, FORMAT_KOT),
            encode_route_slot(&a, FORMAT_STANDARD_PORTRAIT),
            encode_route_slot(&a, FORMAT_STANDARD_LANDSCAPE),
        ];
        cfg.collapse_one_output_per_printer();
        assert_eq!(cfg.routes.bill.len(), 1);
        assert_eq!(
            cfg.routes.bill[0],
            encode_route_slot(&a, FORMAT_KOT),
            "keeps first profile"
        );
        cfg.set_place_profile(RouteKey::Bill, FORMAT_STANDARD_PORTRAIT);
        assert_eq!(cfg.format_id_for(RouteKey::Bill), FORMAT_STANDARD_PORTRAIT);
        assert_eq!(
            cfg.routes.bill[0],
            encode_route_slot(&a, FORMAT_STANDARD_PORTRAIT)
        );
        let vp = cfg
            .add_virtual_printer(
                "Counter",
                &a,
                VirtualPrinterValues {
                    landscape: false,
                    delivery: DELIVERY_ESCPOS.into(),
                    paper_width_mm: 80,
                    ..VirtualPrinterValues::default()
                },
            )
            .expect("virtual");
        assert_eq!(cfg.virtual_printers.len(), 1);
        let v = cfg.virtual_printer_by_id(&vp).unwrap();
        assert!(!v.landscape);
        assert!(!v.document);
        cfg.ensure_virtual_printers_from_routes();
        assert_eq!(cfg.virtual_printers.len(), 1, "does not reseed when non-empty");
    }

    #[test]
    fn typed_schemes_parse() {
        assert_eq!(
            parse_target("bt:AA:BB:CC:DD:EE:FF"),
            PrinterTarget::Bluetooth("bt:AA:BB:CC:DD:EE:FF".into())
        );
        assert_eq!(
            parse_target("usb:04b8:0e15"),
            PrinterTarget::Usb("usb:04b8:0e15".into())
        );
        assert_eq!(parse_target("android-print:"), PrinterTarget::AndroidPrint);
        assert!(can_print_mode(
            &PrinterTarget::Bluetooth("bt:AA:BB:CC:DD:EE:FF".into()),
            PrinterMode::Kot
        ));
        assert!(can_print_mode(
            &PrinterTarget::Usb("usb:04b8:0e15".into()),
            PrinterMode::Standard
        ));
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
        print_network(&addr, "KOT 12 · D3\n2 x Thali").unwrap();
        // Exactly the job: nothing appended (no status query), nothing split.
        assert_eq!(got.join().unwrap(), escpos_job("KOT 12 · D3\n2 x Thali"));
        // Nobody listening: said in words, not a hang.
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().to_string();
        assert!(print_network(&closed, "x").unwrap_err().contains("cannot reach"));
    }


    /// A Server answering a waiter waits for the paper: a printer that took
    /// the job says nothing, one that cannot be reached is named — and outside a
    /// confirm (the till's own UI thread) nothing ever waits.
    #[test]
    fn confirm_paper_reports_what_the_printer_did() {
        // A printer that takes the job (reads it all), and an address where
        // nothing listens.
        fn taking_printer() -> String {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap().to_string();
            std::thread::spawn(move || {
                let (mut conn, _) = listener.accept().unwrap();
                let mut got = Vec::new();
                let _ = std::io::Read::read_to_end(&mut conn, &mut got);
            });
            addr
        }
        let nobody = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().to_string();
        let kot = PrintFormat {
            id: FORMAT_KOT.into(),
            label: "KOT".into(),
            document: false,
            landscape: false,
            delivery: String::new(),
            em: STD_EM_NORMAL,
            line: 32,
            baseline: STD_EM_NORMAL + 2.0,
            side: 4.0,
            place: Default::default(),
        };
        let job = |addr: String| Job {
            dests: vec![PrintDest { target: PrinterTarget::Network(addr), format: kot.clone(), virtual_paper: None }],
            mode: RoutePrintMode::First,
            path: PathBuf::new(),
            text: "KOT 1\n1 x Chaas".into(),
            copies: 1,
            done: confirm_channel(),
        };
        assert!(confirm_channel().is_none(), "no confirm open: nobody waits");
        let paper = ConfirmPaper::start();
        queue(job(taking_printer())).unwrap();
        assert_eq!(paper.finish(Duration::from_secs(10)), Vec::<String>::new());
        let paper = ConfirmPaper::start();
        queue(job(nobody)).unwrap();
        let said = paper.finish(Duration::from_secs(10));
        assert!(
            said.iter().any(|p| p.contains("did not print") && p.contains("cannot reach")),
            "{said:?}"
        );
        assert!(confirm_channel().is_none(), "finish closes the confirm");
        let _ = take_problems();
    }

    #[test]
    fn weak_standard_side_margins_upgrade_to_product_default() {
        let mut list = vec![PrintFormat {
            id: FORMAT_STANDARD_PORTRAIT.into(),
            label: "Standard Portrait".into(),
            document: true,
            landscape: false,
            delivery: String::new(),
            em: STD_EM_NORMAL,
            line: 40,
            baseline: STD_EM_NORMAL + 2.0,
            side: 12.0,
            place: Default::default(),
        }];
        PrintFormat::ensure_builtins(&mut list);
        let p = list.iter().find(|f| f.id == FORMAT_STANDARD_PORTRAIT).unwrap();
        assert_eq!(p.side, STD_SIDE);
        assert_eq!(p.em, STD_EM_NORMAL); // em untouched — only broken side migrates
        let mut kept = vec![PrintFormat {
            side: 28.0,
            em: STD_EM_LARGE,
            baseline: STD_EM_LARGE + 2.0,
            ..PrintFormat::standard_portrait()
        }];
        PrintFormat::ensure_builtins(&mut kept);
        assert_eq!(kept[0].side, 28.0);
        assert_eq!(kept[0].em, STD_EM_LARGE);
    }



    /// ESC/POS is the slip picture in standard raster, for any text: reset,
    /// `GS v 0` bands, feed, cut — no text-or-picture guess.
    #[test]
    fn an_escpos_job_is_the_slip_picture() {
        for text in ["Bill 7\n2 x Tea\n1 x Roti", "Bill\nખીચડી x 2"] {
            let job = escpos_job(text);
            assert_eq!(&job[..2], &[0x1B, 0x40], "{text}: reset first");
            assert!(job.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]), "{text}: raster");
            assert!(job.ends_with(&[0x1D, 0x56, 0x00]), "{text}: cut last");
            assert!(!job.windows(2).any(|w| w == [0x10, 0x04]), "{text}: no status query");
        }
    }



    /// Alone means alone (owner, 2026-11-15): with both pairs cleared,
    /// Cashier and Bill no each print on their own row, through a save and
    /// a reload. A save from before pairs existed keeps the product pairs.
    #[test]
    fn a_line_set_alone_stays_on_its_own_row() {
        let mut fmt = BillSlipFormat::default();
        fmt.set_meta_partner(BillStyleField::Time, None);
        fmt.set_meta_partner(BillStyleField::Cashier, None);
        let fmt = fmt.clamped();
        assert!(fmt.meta_pairs.is_empty(), "{:?}", fmt.meta_pairs);
        let back = BillSlipFormat::parse(&serde_json::to_string(&fmt).unwrap());
        assert_eq!(back.meta_pair_of(BillStyleField::Cashier), None);
        assert_eq!(back.meta_pair_of(BillStyleField::Time), None);
        let mut text = String::new();
        {
            let _stage = BillStage::begin();
            append_bill_field(&mut text, &back, BillStyleField::Cashier, "Cashier: Asha", false);
            append_bill_field(&mut text, &back, BillStyleField::BillNo, "Bill No.: 12", false);
            text.push_str(&bill_stage_finish(&back, false));
        }
        let rows: Vec<&str> = text.lines().filter(|l| l.contains("Cashier") || l.contains("Bill No")).collect();
        assert_eq!(rows.len(), 2, "two rows: {rows:?}");
        assert!(rows.iter().all(|r| !r.contains('\t')), "not a pair: {rows:?}");
        let old = BillSlipFormat::parse("{}");
        assert_eq!(old.meta_pair_of(BillStyleField::Cashier), Some(BillStyleField::BillNo));
    }

    /// Printers belong to each shop's books folder and reload from there.
    #[test]
    fn device_printers_are_kept_per_books_dir() {
        let base = crate::test_dirs::run_root().join(format!("tabiyat-printers-{}", uuid::Uuid::new_v4().simple()));
        assert_eq!(load_device_printers(&base), DevicePrinters::default());
        let mut chosen = DevicePrinters::default();
        let _spare = chosen.add_printer("Spare", "192.168.1.50");
        let counter = chosen.add_printer("Counter", "Canon_G3010");
        chosen.routes.bill = vec![counter.clone()];
        // Routes hold bare printer ids; the place owns the format.
        chosen.dedupe_printers();
        PrintFormat::ensure_builtins(&mut chosen.formats);
        // save_device_printers also configure()s — hold the test lock for the
        // whole round-trip so a parallel empty-route test cannot see these.
        with_test_printers(chosen.clone(), || {
            save_device_printers(&base, &chosen).unwrap();
            assert_eq!(load_device_printers(&base), chosen);
            assert_eq!(
                printers().targets_for(RouteKey::Bill),
                vec![PrinterTarget::Named("Canon_G3010".into())]
            );
        });
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn same_destination_is_not_added_twice() {
        let mut cfg = DevicePrinters::default();
        let (a, first) = cfg.ensure_printer("CounterPrinter", "CounterPrinter");
        let (b, second) = cfg.ensure_printer("CounterPrinter", "CounterPrinter");
        assert!(first);
        assert!(!second);
        assert_eq!(a, b);
        assert_eq!(cfg.printers.len(), 1);
        // Pre-existing duplicates collapse on dedupe.
        cfg.printers.push(NamedPrinter {
            id: "dup".into(),
            label: "CounterPrinter".into(),
            target: "CounterPrinter".into(),
            virtual_paper_width_mm: 0.0,
            virtual_paper_length_mm: 0.0,
            reported_width_mm: 0.0,
            reported_length_mm: 0.0,
            reported_name: String::new(),
            reported_edges_mm: [0.0; 4],
            _legacy_mode: None,
        });
        *cfg.route_mut(RouteKey::Bill) = vec![a.clone(), "dup".into()];
        cfg.dedupe_printers();
        assert_eq!(cfg.printers.len(), 1);
        // One slot for the one printer. The place owns the format, so the
        // slot is the bare printer id — the same form a reload gives.
        assert_eq!(cfg.route_for(RouteKey::Bill), &[a.clone()][..]);
    }

    #[test]
    fn cups_queue_underscore_and_hyphen_are_one_printer() {
        let mut cfg = DevicePrinters::default();
        let (a, first) = cfg.ensure_printer("Brother_DCP_L2640DW", "Brother_DCP_L2640DW");
        let (b, second) =
            cfg.ensure_printer("Brother_DCP-L2640DW", "Brother_DCP-L2640DW");
        assert!(first);
        assert!(!second);
        assert_eq!(a, b);
        assert_eq!(cfg.printers.len(), 1);
        // Network / USB keep distinct colons — not rewritten into one.
        let (n1, _) = cfg.ensure_printer("a", "192.168.1.10:9100");
        let (n2, added) = cfg.ensure_printer("b", "192.168.1.11:9100");
        assert!(added);
        assert_ne!(n1, n2);
    }

    #[test]
    fn route_name_format_key_collapses_underscore_hyphen() {
        assert_eq!(
            DevicePrinters::route_name_format_key("Brother_DCP_L2640DW", FORMAT_STANDARD),
            DevicePrinters::route_name_format_key("Brother_DCP-L2640DW", FORMAT_STANDARD)
        );
        assert_ne!(
            DevicePrinters::route_name_format_key("Brother_DCP_L2640DW", FORMAT_STANDARD),
            DevicePrinters::route_name_format_key("Brother_DCP_L2640DW", FORMAT_KOT)
        );
    }
}

#[cfg(test)]
mod column_tests {
    use super::*;

    /// The owner's item columns ride the slip text to the raster: a new
    /// format fills its content width; the directive carries each shown
    /// column in print order, in the print's dots (docs/PRINTER.md "Item
    /// columns").
    #[test]
    fn bill_item_columns_ride_the_slip_to_the_raster() {
        use crate::receipt_raster::LineColumns;
        let fmt = BillSlipFormat::default().clamped();
        let cols = fmt.item_columns();
        let sum: f32 = BillItemCol::ALL.iter().map(|c| cols.get(*c).width_mm).sum();
        let room = fill_room_mm(210, STD_SIDE);
        assert!(sum <= room + 0.01 && sum > room - 2.5, "new columns fill 210 mm ({sum} of {room})");
        assert_eq!(cols.name.align, KotAlign::Start);
        assert_eq!(cols.amount.align, KotAlign::End);
        let raw = fmt.render_sample("Shop", "01/01/26 12:00");
        assert!(raw.starts_with(SLIP_TAB_MARK), "bill sample carries its columns:\n{raw}");
        let (parsed, body) = take_slip_tab_layout(&raw);
        let Some(LineColumns::Owner(owner)) = parsed else {
            panic!("owner columns directive, got {parsed:?}");
        };
        assert_eq!(owner.cols.len(), 4);
        assert!(!owner.rtl);
        let dpm = content_dots_per_mm(210);
        assert!((owner.cols[0].width - cols.name.width_mm * dpm).abs() < 0.11);
        // Spaces are typed in px and drawn in dots, like the width.
        assert!((owner.cols[3].start - crate::receipt_raster::px_dots(cols.amount.start_px as f32)).abs() < 1e-3);
        assert!(!body.starts_with(SLIP_TAB_MARK));
        let lines = styled_slip_lines(&raw, &PrintFormat::standard().raster_metrics());
        assert!(
            lines
                .iter()
                .filter(|l| l.text.contains('\t'))
                .all(|l| matches!(l.tab, Some(LineColumns::Owner(_)))),
            "item lines keep the columns through styled_slip_lines"
        );
        let mut hidden_price = fmt.clone();
        hidden_price.item_show_price = false;
        let (parsed, _) = take_slip_tab_layout(&hidden_price.render_sample("Shop", "x"));
        let Some(LineColumns::Owner(owner)) = parsed else { panic!() };
        assert_eq!(owner.cols.len(), 3, "a hidden column takes no room");
        assert!((owner.cols[2].width - cols.amount.width_mm * dpm).abs() < 0.11);
        let plain = strip_kot_style_marks(&raw);
        assert!(!plain.contains(SLIP_TAB_MARK) && !plain.contains(KOT_STYLE_MARK));
    }


    /// Saves from before columns start with columns filling their own
    /// content width; after that a column keeps its mm whatever the width.
    #[test]
    fn old_saves_get_columns_that_fill_their_width_then_keep_them() {
        let old = BillSlipFormat::parse(r#"{"content_width_mm": 80}"#);
        let cols = old.item_columns.expect("filled on load");
        let sum: f32 = BillItemCol::ALL.iter().map(|c| cols.get(*c).width_mm).sum();
        let room = fill_room_mm(80, STD_SIDE);
        assert!(sum <= room + 0.01 && sum > room - 2.5, "{sum} of {room}");
        let mut wider = old.clone();
        wider.content_width_mm = 150;
        assert_eq!(wider.clamped().item_columns, Some(cols), "width never moves a column");
    }


    /// Sizes are literal from 1 to 200 px, and three-digit sizes survive the
    /// line mark (owner, 2026-11-15).
    #[test]
    fn sizes_from_1_to_200_survive_the_line_mark() {
        for size in [1u32, 6, 12, 99, 100, 150, 200] {
            let mark = kot_style_prefix(&KotFieldStyle::bold(size), false);
            let line = format!("{mark}KOT - 12");
            let (style, body) = take_kot_line_style(&line);
            let style = style.expect("mark");
            assert_eq!(style.size_em, size);
            assert_eq!(style.weight, KotFontWeight::Bold);
            assert_eq!(body, "KOT - 12");
        }
        let mut fmt = BillSlipFormat::default();
        fmt.items.size_em = 500;
        assert_eq!(fmt.clamped().items.size_em, KOT_EM_MAX);
        assert_eq!((KOT_EM_MIN, KOT_EM_MAX), (1, 200));
    }


    /// Columns of a drawn slip that carry ink (page-x of every inked dot).
    fn inked_xs(text: &str, mm: u32) -> (Vec<usize>, usize, f32) {
        let mut m = PrintFormat::standard().raster_metrics();
        m.width_dots = paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(mm));
        let (rows, height) = crate::receipt_raster::draw_lines(&styled_slip_lines(text, &m), &m);
        let w = m.width_dots;
        let mut xs = Vec::new();
        for y in 0..height {
            for x in 0..w {
                if rows[y * (w / 8) + x / 8] & (0x80 >> (x % 8)) != 0 {
                    xs.push(x);
                }
            }
        }
        (xs, height, m.side)
    }

    fn col(width_mm: f32, align: KotAlign) -> SlipColumn {
        SlipColumn { width_mm, start_px: 0, end_px: 0, align }
    }

    /// The owner's example (2026-11-15): Item 30 + Qty 30 + Price 20 mm on a
    /// 56 mm content width — Item whole, Qty cut at the content edge, no
    /// Price. Nothing is squeezed or moved to make it fit.
    #[test]
    fn columns_past_the_content_width_are_cut_there() {
        let mm = 56;
        let cols = [col(30.0, KotAlign::Start), col(30.0, KotAlign::Start), col(20.0, KotAlign::Start)];
        let text = format!(
            "{}{}MMMMMMMMMMMM\tMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM\tMMMMMMMM\n",
            encode_slip_columns(&cols, mm, false, false),
            kot_style_prefix(&KotFieldStyle::regular(24), false),
        );
        let (xs, _, side) = inked_xs(&text, mm);
        let w = paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(mm));
        let end = w as f32 - side;
        assert!(xs.iter().all(|&x| (x as f32) < end.ceil()), "nothing past the content edge");
        let b_start = side + 30.0 * content_dots_per_mm(mm);
        assert!(xs.iter().any(|&x| (x as f32) > b_start + 4.0), "Qty is printed");
        assert!(xs.iter().any(|&x| (x as f32) >= end - 6.0), "Qty runs to the edge and is cut there");
    }

    /// A cell longer than its column wraps inside it; no ink leaves the box.
    #[test]
    fn a_long_cell_wraps_inside_its_column() {
        let mm = 80;
        let cols = [col(20.0, KotAlign::Start), col(20.0, KotAlign::End)];
        let mark = kot_style_prefix(&KotFieldStyle::regular(24), false);
        let one = format!("{}{mark}Tea\t\n", encode_slip_columns(&cols, mm, false, false));
        let long = format!(
            "{}{mark}Paneer Butter Masala Special\t\n",
            encode_slip_columns(&cols, mm, false, false)
        );
        let (_, h1, _) = inked_xs(&one, mm);
        let (xs, h, side) = inked_xs(&long, mm);
        assert!(h >= 3 * h1, "wrapped onto several rows ({h} vs {h1})");
        let box_end = side + 20.0 * content_dots_per_mm(mm);
        assert!(xs.iter().all(|&x| (x as f32) < box_end.ceil()), "ink stays in its column");
    }

    /// Only one item column shown: it still lays out in its column — its
    /// width, spaces and alignment — not as a plain line.
    /// A job queued before item columns (the old `T` directive) still
    /// parses to its column metrics, and its body prints as it was.
    #[test]
    fn jobs_queued_before_columns_still_parse() {
        let tab = crate::receipt_raster::TabColumnLayout {
            gap_em: 1.5,
            min_em: [3.0, 5.0, 6.0, 0.0],
            cell_align: crate::receipt_raster::LineAlign::End,
            sep: true,
        };
        let text = format!("{}Naan\t2\t40\t80\n", encode_slip_tab_layout(&tab));
        let (cols, body) = take_slip_tab_layout(&text);
        assert_eq!(cols, Some(crate::receipt_raster::LineColumns::Legacy(tab)));
        assert_eq!(body, "Naan\t2\t40\t80\n");
    }

    #[test]
    fn a_lone_item_column_keeps_its_width() {
        let mut fmt = BillSlipFormat::parse(r#"{"content_width_mm": 80}"#);
        fmt.item_show_qty = false;
        fmt.item_show_price = false;
        fmt.item_show_amount = false;
        let mut cols = fmt.item_columns();
        cols.name = SlipColumn { width_mm: 20.0, start_px: 0, end_px: 0, align: KotAlign::Start };
        fmt.item_columns = Some(cols);
        let row = format_bill_item_block(
            &fmt,
            BillItemPieces { name: "Paneer Butter Masala Special", qty: "1", price: "1", amount: "1", note: "" },
        );
        assert_eq!(row, "Paneer Butter Masala Special\t");
        let text = format!(
            "{}{}{row}\n",
            encode_slip_columns(&fmt.ordered_slip_columns(), 80, false, false),
            kot_style_prefix(&KotFieldStyle::regular(24), false),
        );
        let (xs, _, side) = inked_xs(&text, 80);
        let box_end = side + 20.0 * content_dots_per_mm(80);
        assert!(!xs.is_empty() && xs.iter().all(|&x| (x as f32) < box_end.ceil()), "wrapped inside its 20 mm");
    }

    /// Right to left: the first column sits at the right edge.
    #[test]
    fn rtl_columns_start_at_the_right() {
        let mm = 80;
        let cols = [col(20.0, KotAlign::Start), col(10.0, KotAlign::Start)];
        let text = format!(
            "{}{}MMMM\t\n",
            encode_slip_columns(&cols, mm, true, false),
            kot_style_prefix(&KotFieldStyle::regular(24), true),
        );
        let (xs, _, side) = inked_xs(&text, mm);
        let w = paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(mm)) as f32;
        let box_start = w - side - 20.0 * content_dots_per_mm(mm);
        assert!(!xs.is_empty());
        assert!(xs.iter().all(|&x| (x as f32) >= box_start.floor()), "first column at the right");
        // Start alignment in RTL hugs the right edge of the column.
        assert!(xs.iter().any(|&x| (x as f32) > w - side - 12.0));
    }

    /// Pair halves: each in its half, each at its own size.
    #[test]
    fn pair_halves_sit_in_their_halves_at_their_sizes() {
        let mut fmt = BillSlipFormat::default();
        fmt.time.size_em = 20;
        fmt.cashier.size_em = 40;
        let mut text = String::new();
        append_pair(&mut text, (&fmt.time, "MMM"), (&fmt.cashier, "MMM"), false);
        let (xs, h, side) = inked_xs(&text, 80);
        assert_eq!(
            h,
            (crate::receipt_raster::px_dots(40.0) * 1.45).round() as usize,
            "as tall as the larger half"
        );
        let w = paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(80)) as f32;
        let mid = side + (w - 2.0 * side) * 0.5;
        assert!(xs.iter().any(|&x| (x as f32) < mid) && xs.iter().any(|&x| (x as f32) >= mid));
    }


    /// A pair line keeps each field's own size and alignment, and the first
    /// field takes the start half (the right half in RTL).
    #[test]
    fn pair_lines_carry_both_fields_styles() {
        use crate::receipt_raster::LineColumns;
        let mut fmt = BillSlipFormat::default();
        fmt.time.size_em = 20;
        fmt.cashier.size_em = 30;
        fmt.cashier.align = KotAlign::End;
        let pair = |rtl: bool| {
            let mut out = String::new();
            append_pair(&mut out, (&fmt.time, "Date: 1"), (&fmt.cashier, "Cashier: T2"), rtl);
            let lines = styled_slip_lines(&out, &PrintFormat::standard().raster_metrics());
            assert_eq!(lines.len(), 1);
            let l = &lines[0];
            let Some(LineColumns::Pair(half)) = &l.tab else { panic!("pair") };
            (l.text.clone(), l.em, half.text.clone(), half.em, half.align)
        };
        let (a, a_em, b, b_em, b_align) = pair(false);
        assert_eq!((a.as_str(), a_em, b.as_str(), b_em), ("Date: 1", 20.0, "Cashier: T2", 30.0));
        assert_eq!(b_align, crate::receipt_raster::LineAlign::End);
        let (a, _, b, _, _) = pair(true);
        assert_eq!((a.as_str(), b.as_str()), ("Cashier: T2", "Date: 1"), "RTL: first field on the right");
        let mut plain = String::new();
        append_pair(&mut plain, (&fmt.time, "Date: 1"), (&fmt.cashier, "Cashier: T2"), false);
        assert_eq!(strip_kot_style_marks(&plain).trim_end(), "Date: 1\tCashier: T2");
    }


    /// The three sample totals through the rows builder.
    fn sample_total_lines(fmt: &BillSlipFormat) -> Vec<String> {
        let mut totals = BillTotalRows::new(fmt);
        totals.total(BillTotalKind::Qty, "Total Qty", "3", "Total Qty: 3");
        totals.total(BillTotalKind::Subtotal, "Sub Total", "100.00", "Sub Total: 100.00");
        totals.total(BillTotalKind::Grand, "Grand Total", "100.00", "Grand Total: 100.00");
        totals.finish()
    }

    fn place(number: &str, caption: &str) -> BillTotalPlace {
        BillTotalPlace { number: number.into(), caption: caption.into() }
    }

    /// Nothing placed: every total on its own line, word for word as before.
    #[test]
    fn totals_keep_their_own_lines_until_the_owner_places_them() {
        let fmt = BillSlipFormat::default();
        assert_eq!(sample_total_lines(&fmt), ["Total Qty: 3", "Sub Total: 100.00", "Grand Total: 100.00"]);
        let json = fmt.to_json();
        assert!(!json.contains("total_qty_place") && !json.contains("total_grand_place"), "{json}");
    }

    /// The owner's ask (2026-11-17): Total Qty in the Qty column, Grand
    /// Total in the Amount column, in a single row.
    #[test]
    fn totals_under_their_columns_share_one_row() {
        let mut fmt = BillSlipFormat::default();
        fmt.total_show_subtotal = false;
        fmt.total_qty_place = place("qty", "none");
        fmt.total_grand_place = place("amount", "name");
        let mut totals = BillTotalRows::new(&fmt);
        totals.total(BillTotalKind::Qty, "Total Qty", "3", "Total Qty: 3");
        totals.total(BillTotalKind::Grand, "Grand Total", "100.00", "Grand Total: 100.00");
        // Item | Qty | Price | Amount.
        assert_eq!(totals.finish(), ["Grand Total\t3\t\t100.00"]);
        // All three in one row, each under a column of its own.
        fmt.total_show_subtotal = true;
        fmt.total_subtotal_place = place("price", "none");
        fmt.total_grand_place = place("amount", "none");
        assert_eq!(sample_total_lines(&fmt), ["\t3\t100.00\t100.00"]);
        // The places are kept with the format.
        let back = BillSlipFormat::parse(&fmt.to_json());
        assert_eq!(back.total_qty_place, place("qty", "none"));
        assert_eq!(back.total_subtotal_place, place("price", "none"));
        assert_eq!(back.total_grand_place, place("amount", "none"));
    }

    /// A box holds one total: the next one for it starts the next row.
    #[test]
    fn two_totals_for_one_column_take_two_rows() {
        let mut fmt = BillSlipFormat::default();
        fmt.total_qty_place = place("qty", "none");
        fmt.total_subtotal_place = place("amount", "price");
        fmt.total_grand_place = place("amount", "price");
        assert_eq!(sample_total_lines(&fmt), ["\t3\tSub Total\t100.00", "\t\tGrand Total\t100.00"]);
        // A caption with its number is one cell, as the line reads alone.
        fmt.total_grand_place = place("amount", "");
        assert_eq!(sample_total_lines(&fmt), ["\t3\tSub Total\t100.00", "\t\t\tGrand Total: 100.00"]);
        // A caption whose own box is taken waits for the next row too.
        fmt.total_qty_place = place("qty", "price");
        fmt.total_subtotal_place = place("amount", "price");
        fmt.total_show_grand = false;
        let mut totals = BillTotalRows::new(&fmt);
        totals.total(BillTotalKind::Qty, "Total Qty", "3", "Total Qty: 3");
        totals.total(BillTotalKind::Subtotal, "Sub Total", "100.00", "Sub Total: 100.00");
        assert_eq!(totals.finish(), ["\t3\tTotal Qty\t", "\t\tSub Total\t100.00"]);
    }

    /// Only under a column that prints: a hidden column, an id this build
    /// does not know, or stacked items leave the total on its own line.
    #[test]
    fn a_total_with_no_column_to_sit_under_has_its_own_line() {
        let own = ["Total Qty: 3", "Sub Total: 100.00", "Grand Total: 100.00"];
        let mut fmt = BillSlipFormat::default();
        fmt.total_qty_place = place("qty", "none");
        fmt.total_subtotal_place = place("price", "");
        fmt.total_grand_place = place("amount", "name");
        fmt.item_layout = BillItemLayout::Stacked;
        assert_eq!(sample_total_lines(&fmt), own);
        fmt.item_layout = BillItemLayout::Columns;
        fmt.item_show_qty = false;
        fmt.item_show_price = false;
        fmt.item_show_amount = false;
        assert_eq!(sample_total_lines(&fmt), own);
        // A caption set under a hidden column goes with its number.
        fmt.item_show_amount = true;
        fmt.item_show_name = false;
        assert_eq!(sample_total_lines(&fmt), ["Total Qty: 3", "Sub Total: 100.00", "Grand Total: 100.00\t"]);
        // From a newer build: an unknown id reads as "own line" and the
        // rest of the format is kept.
        let read = BillSlipFormat::parse(r#"{"footer":"Thanks","total_qty_place":{"number":"tray","caption":"left"}}"#);
        assert_eq!(read.footer, "Thanks");
        assert!(read.total_qty_place.is_default(), "{:?}", read.total_qty_place);
    }

    /// Tax, service, rounding and payments have no setting of their own:
    /// they follow Sub total, and always keep their caption.
    #[test]
    fn the_other_amounts_follow_sub_total() {
        let mut fmt = BillSlipFormat::default();
        let lines = |fmt: &BillSlipFormat| {
            let mut totals = BillTotalRows::new(fmt);
            totals.total(BillTotalKind::Subtotal, "Sub Total", "100.00", "Sub Total 100.00");
            totals.amount("CGST 2.5%", "2.50", "CGST 2.5% 2.50");
            totals.words("(GST inclusive)");
            totals.total(BillTotalKind::Grand, "Total", "105.00", "Total 105.00");
            totals.amount("Cash", "105.00", "Cash 105.00");
            totals.finish()
        };
        assert_eq!(
            lines(&fmt),
            ["Sub Total 100.00", "CGST 2.5% 2.50", "(GST inclusive)", "Total 105.00", "Cash 105.00"],
            "as before while Sub total has its own line"
        );
        fmt.total_subtotal_place = place("amount", "price");
        fmt.total_grand_place = place("amount", "price");
        assert_eq!(
            lines(&fmt),
            [
                "\t\tSub Total\t100.00",
                "\t\tCGST 2.5%\t2.50",
                "(GST inclusive)",
                "\t\tTotal\t105.00",
                "\t\tCash\t105.00",
            ]
        );
        // Sub total without its caption: the others still say what they are.
        fmt.total_subtotal_place = place("amount", "none");
        assert_eq!(lines(&fmt)[..2], ["\t\t\t100.00", "\t\t\tCGST 2.5% 2.50"]);
    }

    /// Through the whole bill: the row keeps its empty first cells, is laid
    /// on the item columns, and each number is drawn in its column's box.
    #[test]
    fn a_placed_total_is_drawn_inside_its_column() {
        let mm = 80;
        let mut fmt = BillSlipFormat::parse(r#"{"content_width_mm": 80}"#);
        let mut cols = fmt.item_columns();
        cols.name = col(30.0, KotAlign::Start);
        cols.qty = col(10.0, KotAlign::End);
        cols.price = col(15.0, KotAlign::End);
        cols.amount = col(17.0, KotAlign::End);
        fmt.item_columns = Some(cols);
        fmt.total_show_subtotal = false;
        fmt.total_show_grand = false;
        fmt.total_qty_place = place("qty", "none");
        let text = fmt.render_sample("Shop", "01/01/26 12:00");
        let row = text
            .lines()
            .find(|l| take_kot_line_style(l).1.starts_with("\t3"))
            .unwrap_or_else(|| panic!("no totals row:\n{text}"));
        assert_eq!(take_kot_line_style(row).1, "\t3\t\t");
        let m = PrintFormat::standard().raster_metrics();
        let styled = styled_slip_lines(&text, &m);
        let line = styled.iter().find(|l| l.text.starts_with("\t3")).unwrap();
        assert!(matches!(line.tab, Some(crate::receipt_raster::LineColumns::Owner(_))), "on the item columns");
        // The Total line's own size and weight.
        assert_eq!(line.em, fmt.total.size_em as f32);
        // Drawn alone under the columns' directive: all its ink in the Qty box.
        let alone = format!("{}{row}\n", encode_slip_columns(&fmt.ordered_slip_columns(), mm, false, false));
        let (xs, _, side) = inked_xs(&alone, mm);
        let dpm = content_dots_per_mm(mm);
        let (box_start, box_end) = (side + 30.0 * dpm, side + 40.0 * dpm);
        assert!(!xs.is_empty());
        assert!(
            xs.iter().all(|&x| (x as f32) >= box_start.floor() && (x as f32) < box_end.ceil()),
            "ink {}..{} outside the Qty box {box_start}..{box_end}",
            xs.iter().min().unwrap(),
            xs.iter().max().unwrap()
        );
        // End-aligned like the column's quantities: it ends at the box's end.
        assert!((*xs.iter().max().unwrap() as f32) > box_end - 6.0);
    }

    /// A lone column's line ends in a tab; the stage keeps it, so the line
    /// is laid out in its column and not as a plain line.
    #[test]
    fn a_lone_column_line_keeps_its_tab_through_the_bill() {
        let mut fmt = BillSlipFormat::default();
        fmt.item_show_qty = false;
        fmt.item_show_price = false;
        fmt.item_show_amount = false;
        let text = strip_kot_style_marks(&fmt.render_sample("Shop", "01/01/26 12:00"));
        assert!(text.lines().any(|l| l == "Paracetamol\t"), "{text}");
    }
}

/// The printer's paper (owner, 2026-11-16; docs/PRINTER.md § Paper and
/// preview) — the same model and checks as Khaynjo's.
#[cfg(test)]
mod paper_tests {
    use super::*;

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
        let (office, _) = d.ensure_printer("Office", "Office");
        let (thermal, _) = d.ensure_printer("192.168.50.20:9100", "192.168.50.20:9100");
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
        let (office, _) = d.ensure_printer("Office", "Office");
        let (thermal, _) = d.ensure_printer("192.168.1.50:9100", "192.168.1.50:9100");
        // Android's print service has no driver to ask: the default.
        let (android, _) = d.ensure_printer("Android print", "android-print:");
        let (set, _) = d.ensure_printer("Counter", "Counter");
        assert!(d.set_printer_paper_mm(&set, 104.8, 0.0));
        let mut answers = std::collections::HashMap::new();
        answers.insert("Office".to_string(), Ok(crate::os_paper::parse_ppd(ppd)));
        // The counter's driver now says A4: its set paper stays.
        answers.insert("Counter".to_string(), Ok(crate::os_paper::parse_ppd(ppd)));
        for _ in 0..2 {
            d.fill_missing_papers(&answers, true);
            // A second Find adds nothing and changes nothing.
            d.ensure_printer("Office", "Office");
            d.ensure_printer("Counter", "Counter");
            d.dedupe_printers();
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
        let (q, _) = quiet.ensure_printer("Quiet", "Quiet");
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

    /// `TABIYAT_PAPER_MM`: `<w>x<l>` (mm) for a sheet, `<w>` for a roll.
    fn paper_mm_var(v: &str) -> crate::os_paper::PaperMm {
        let (w, l) = v.split_once('x').unwrap_or((v, "0"));
        crate::os_paper::PaperMm::new(w.trim().parse().unwrap_or(80.0), l.trim().parse().unwrap_or(0.0))
            .expect("a paper width")
    }

    /// `TABIYAT_PLACE`: `<down>,<across>` (0 top / start … 2 bottom / end).
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
        (text, f)
    }

    /// The format of the sample bill, as the check's settings say:
    /// `TABIYAT_BILL_MM` wide, `TABIYAT_LANDSCAPE=1`, `TABIYAT_PLACE`.
    fn check_format(var: &dyn Fn(&str) -> String) -> (String, PrintFormat) {
        let (text, mut f) = check_bill(
            var("TABIYAT_BILL_MM").parse().unwrap_or(80),
            var("TABIYAT_LINES").parse().unwrap_or(8),
        );
        f.landscape = var("TABIYAT_LANDSCAPE") == "1";
        f.place = place_var(&var("TABIYAT_PLACE"));
        (text, f)
    }

    /// The virtual paper of the check: `TABIYAT_PAPER_MM`, or none set.
    fn check_virtual(var: &dyn Fn(&str) -> String) -> Option<crate::os_paper::PaperMm> {
        Some(var("TABIYAT_PAPER_MM")).filter(|v| !v.trim().is_empty()).map(|v| paper_mm_var(&v))
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
    /// `TABIYAT_PPD=<file.ppd> TABIYAT_PAPER_MM=<w>x<l> or <w> (the virtual
    /// paper; blank = none) TABIYAT_PLACE=<down>,<across> TABIYAT_BILL_MM=<mm>
    /// TABIYAT_LINES=<n> TABIYAT_LANDSCAPE=1 TABIYAT_CHECK_OUT=<dir> cargo
    /// test -p kumkaj-tabiyat --lib write_os_document_pdf -- --ignored`
    #[test]
    #[ignore]
    fn write_os_document_pdf() {
        let var = |k: &str| std::env::var(k).unwrap_or_default();
        let ppd = std::fs::read_to_string(var("TABIYAT_PPD")).expect("TABIYAT_PPD");
        let out = std::path::PathBuf::from(var("TABIYAT_CHECK_OUT"));
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
    /// `TABIYAT_CHECK_OUT=<dir>` and the settings of
    /// [`write_os_document_pdf`]; `TABIYAT_COPIES=<n>`.
    #[cfg(target_os = "windows")]
    #[test]
    #[ignore]
    fn windows_print_check() {
        let var = |k: &str| std::env::var(k).unwrap_or_default();
        let out = std::path::PathBuf::from(var("TABIYAT_CHECK_OUT"));
        std::fs::create_dir_all(&out).unwrap();
        let printer = Some(var("TABIYAT_WIN_PRINTER"))
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
        let copies: u32 = var("TABIYAT_COPIES").parse().unwrap_or(1);
        let mut printed_on = None;
        let result = crate::print_win::print_pages(Some(&printer), "Tabiyat check", copies, Some(&file), |own| {
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
        let mut f = PrintFormat::kot();
        f.line = wrap_cols_for_paper_mm(58);
        let text = "Tea\nCoffee\n";
        let roll = Some(crate::os_paper::PaperMm { width_mm: 80.0, length_mm: 0.0 });
        let plain = escpos_job_format(&f, text);
        assert_eq!(escpos_job_placed(&f, text, roll), plain, "top-start: unchanged");
        f.place = crate::os_paper::Place::new(2, 1);
        assert_eq!(escpos_job_placed(&f, text, None), plain, "roll not known: unchanged");
        let (w0, rows0) = gs_v0_rows(&plain);
        let (w1, rows1) = gs_v0_rows(&escpos_job_placed(&f, text, roll));
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
            f.line = wrap_cols_for_paper_mm(mm);
            let m = f.raster_metrics();
            let content = os_document_content_mm(&m);
            assert!((content - m.page_dots() as f64 / 7.2).abs() < 1e-9, "{mm}: {content}");
        }
    }
}

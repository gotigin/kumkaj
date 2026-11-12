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
#[cfg(not(target_os = "android"))]
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
    /// Kitchen tickets and cancel slips (no prices).
    Kot,
    /// Bills, receipts, shift and day slips.
    Bill,
}

/// Which ordered route a job uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteKey<'a> {
    Bill,
    /// Kitchen ticket for this station id (`""` = unassigned items only).
    Kot { station_id: &'a str },
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
pub fn route_wrap_cols(key: RouteKey<'_>) -> u32 {
    printers()
        .destinations_for(key)
        .first()
        .map(|d| d.format.line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX))
        .unwrap_or_else(|| wrap_cols_for_paper_mm(80))
}

/// Widest kitchen-route paper, so the KOT preview is not stuck at 58 mm.
pub fn kot_preview_wrap_cols() -> u32 {
    let cfg = printers();
    let mut best = 0u32;
    let mut consider = |sid: &str| {
        if let Some(d) = cfg
            .destinations_for(RouteKey::Kot { station_id: sid })
            .first()
        {
            best = best.max(d.format.line);
        }
    };
    consider("");
    let ids: Vec<String> = cfg.routes.kot.keys().cloned().collect();
    for id in ids {
        consider(&id);
    }
    if best == 0 {
        wrap_cols_for_paper_mm(80)
    } else {
        best.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX)
    }
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
}

fn default_true() -> bool { true }
fn default_em() -> f32 { STD_EM_NORMAL }
fn default_line() -> u32 { 40 }
fn default_baseline() -> f32 { STD_EM_NORMAL + 2.0 }
fn default_side() -> f32 { STD_SIDE }

/// Bill / Standard document type-size presets (Settings chips).
/// Product built-ins are **generic** (any OS queue). Narrow-tray shops
/// (e.g. some inkjets) tune side/em on a Profile or via App Template —
/// never assume one owner's Canon in product defaults. OS jobs still
/// fit-to-page so PNGs are not printed ~16″ wide at 72 DPI.
pub const STD_EM_SMALL: f32 = 18.0;
pub const STD_EM_NORMAL: f32 = 20.0;
pub const STD_EM_LARGE: f32 = 26.0;
/// Generic side inset (dots on the 384-wide raster) for Standard formats.
pub const STD_SIDE: f32 = 16.0;
/// Wider inset used only by optional templates (e.g. Rajdhani) for tight trays.
pub const STD_SIDE_NARROW_TRAY: f32 = 40.0;

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
    pub const ALL: [KotFontWeight; 3] = [
        KotFontWeight::Light,
        KotFontWeight::Regular,
        KotFontWeight::Bold,
    ];

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

/// Which slip family is being printed / edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KotSlipKind {
    #[default]
    Original,
    Duplicate,
    Modified,
    Cancelled,
}

impl KotSlipKind {
    pub const ALL: [KotSlipKind; 4] = [
        KotSlipKind::Original,
        KotSlipKind::Duplicate,
        KotSlipKind::Modified,
        KotSlipKind::Cancelled,
    ];

    pub fn locale_key(self) -> &'static str {
        match self {
            KotSlipKind::Original => "kot_kind_original",
            KotSlipKind::Duplicate => "kot_kind_duplicate",
            KotSlipKind::Modified => "kot_kind_modified",
            KotSlipKind::Cancelled => "kot_kind_cancelled",
        }
    }

    /// Product default banner for this kind (empty = none).
    pub fn default_banner(self) -> &'static str {
        match self {
            KotSlipKind::Original => "",
            KotSlipKind::Duplicate => KOT_BANNER_DUPLICATE,
            KotSlipKind::Modified => KOT_BANNER_MODIFIED,
            KotSlipKind::Cancelled => KOT_BANNER_CANCELLED,
        }
    }

    /// Product default item-name prefix for this kind (empty = none).
    pub fn default_item_prefix(self) -> &'static str {
        match self {
            KotSlipKind::Original | KotSlipKind::Duplicate => "",
            KotSlipKind::Modified => KOT_PREFIX_MODIFIED,
            KotSlipKind::Cancelled => KOT_PREFIX_CANCELLED,
        }
    }

    /// Product default: strike dish names (Cancelled only).
    pub fn default_strike_items(self) -> bool {
        matches!(self, KotSlipKind::Cancelled)
    }
}

/// Recovery / "print selected again" banner (not a stored kind field).
pub const KOT_BANNER_REPRINT: &str = "*** REPRINT - SAME ORDER ***";
pub const KOT_BANNER_DUPLICATE: &str = "Duplicate";
pub const KOT_BANNER_MODIFIED: &str = "MODIFIED/UPDATED KOT";
pub const KOT_BANNER_CANCELLED: &str = "PARTIALLY SHIFTED/CANCELLED";
pub const KOT_PREFIX_MODIFIED: &str = "[NEW] ";
pub const KOT_PREFIX_CANCELLED: &str = "[CANCELLED] ";
pub const KOT_BANNER_MAX: usize = 72;
pub const KOT_PREFIX_MAX: usize = 24;

/// Field id for typography editing (matches show toggles / order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KotStyleField {
    Title,
    Shop,
    Time,
    Waiter,
    Station,
    Table,
    KotNo,
    Area,
    ItemQty,
}

impl KotStyleField {
    pub const ALL: [KotStyleField; 9] = [
        KotStyleField::Title,
        KotStyleField::Shop,
        KotStyleField::Time,
        KotStyleField::Waiter,
        KotStyleField::Station,
        KotStyleField::Table,
        KotStyleField::KotNo,
        KotStyleField::Area,
        KotStyleField::ItemQty,
    ];

    pub fn locale_key(self) -> &'static str {
        match self {
            KotStyleField::Title => "kot_style_title",
            KotStyleField::Shop => "kot_show_shop",
            KotStyleField::Time => "kot_show_time",
            KotStyleField::Waiter => "kot_show_waiter",
            KotStyleField::Station => "kot_show_station",
            KotStyleField::Table => "kot_show_table",
            KotStyleField::KotNo => "kot_show_kot_no",
            KotStyleField::Area => "kot_show_area",
            KotStyleField::ItemQty => "kot_style_item_qty",
        }
    }
}

/// Per-field typography for one KOT kind (original / duplicate / modified / cancelled).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KotKindStyles {
    #[serde(default)]
    pub title: KotFieldStyle,
    #[serde(default)]
    pub shop: KotFieldStyle,
    #[serde(default)]
    pub time: KotFieldStyle,
    #[serde(default)]
    pub waiter: KotFieldStyle,
    #[serde(default)]
    pub station: KotFieldStyle,
    #[serde(default)]
    pub table: KotFieldStyle,
    #[serde(default)]
    pub kot_no: KotFieldStyle,
    #[serde(default)]
    pub area: KotFieldStyle,
    #[serde(default)]
    pub item_qty: KotFieldStyle,
    /// Kind banner above the body (editable). Empty → no banner for Original;
    /// other kinds use [`KotSlipKind::default_banner`] until the owner clears it
    /// after [`KotSlipFormat::kind_chrome_v`] ≥ 1.
    #[serde(default)]
    pub banner: String,
    /// Prepended to each dish name (`[CANCELLED] `, `[NEW] `, …). Qty stays plain.
    #[serde(default)]
    pub item_prefix: String,
    /// Legacy single flag (older saves). Migrated into [`Self::strike_item_name`]
    /// on load — notes are never implied by this alone.
    #[serde(default)]
    pub strike_items: bool,
    /// Strike the dish name only (not qty, not the note line). Default on for Cancelled.
    #[serde(default)]
    pub strike_item_name: bool,
    /// Strike the dish note line only. Default off — notes stay readable on cancel slips.
    #[serde(default)]
    pub strike_item_note: bool,
}

impl Default for KotKindStyles {
    /// Rajdhani-like FOH kitchen slip: large bold body, centered header
    /// (time / KOT / area / table), qty column on the right.
    fn default() -> Self {
        Self {
            title: KotFieldStyle::bold_center(KOT_EM_LARGE),
            shop: KotFieldStyle::bold_center(KOT_EM_LARGE),
            time: KotFieldStyle::regular_center(KOT_EM_NORMAL),
            waiter: KotFieldStyle::regular(KOT_EM_NORMAL),
            station: KotFieldStyle::regular(KOT_EM_NORMAL),
            table: KotFieldStyle::bold_center(KOT_EM_XLARGE),
            kot_no: KotFieldStyle::bold_center(KOT_EM_LARGE),
            area: KotFieldStyle::bold_center(KOT_EM_LARGE),
            item_qty: KotFieldStyle::bold(KOT_EM_LARGE),
            banner: String::new(),
            item_prefix: String::new(),
            strike_items: false,
            strike_item_name: false,
            strike_item_note: false,
        }
    }
}

impl KotKindStyles {
    /// Lift legacy `strike_items` into name-only strike; keep name/note independent.
    pub fn normalize_strike_flags(&mut self) {
        if self.strike_items && !self.strike_item_name && !self.strike_item_note {
            self.strike_item_name = true;
        }
        // Keep the legacy flag as an alias of name strike for older readers.
        self.strike_items = self.strike_item_name;
    }
}

impl KotKindStyles {
    pub fn get(&self, field: KotStyleField) -> &KotFieldStyle {
        match field {
            KotStyleField::Title => &self.title,
            KotStyleField::Shop => &self.shop,
            KotStyleField::Time => &self.time,
            KotStyleField::Waiter => &self.waiter,
            KotStyleField::Station => &self.station,
            KotStyleField::Table => &self.table,
            KotStyleField::KotNo => &self.kot_no,
            KotStyleField::Area => &self.area,
            KotStyleField::ItemQty => &self.item_qty,
        }
    }

    pub fn get_mut(&mut self, field: KotStyleField) -> &mut KotFieldStyle {
        match field {
            KotStyleField::Title => &mut self.title,
            KotStyleField::Shop => &mut self.shop,
            KotStyleField::Time => &mut self.time,
            KotStyleField::Waiter => &mut self.waiter,
            KotStyleField::Station => &mut self.station,
            KotStyleField::Table => &mut self.table,
            KotStyleField::KotNo => &mut self.kot_no,
            KotStyleField::Area => &mut self.area,
            KotStyleField::ItemQty => &mut self.item_qty,
        }
    }
}

/// Styles for every KOT kind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KotKindStyleSet {
    #[serde(default)]
    pub original: KotKindStyles,
    #[serde(default)]
    pub duplicate: KotKindStyles,
    #[serde(default)]
    pub modified: KotKindStyles,
    #[serde(default)]
    pub cancelled: KotKindStyles,
}

impl Default for KotKindStyleSet {
    fn default() -> Self {
        let mut duplicate = KotKindStyles::default();
        duplicate.title = KotFieldStyle::bold_center(KOT_EM_XLARGE);
        duplicate.banner = KOT_BANNER_DUPLICATE.into();
        let mut modified = KotKindStyles::default();
        modified.title = KotFieldStyle::bold_center(KOT_EM_XLARGE);
        modified.item_qty = KotFieldStyle::bold(KOT_EM_XLARGE);
        modified.banner = KOT_BANNER_MODIFIED.into();
        modified.item_prefix = KOT_PREFIX_MODIFIED.into();
        let mut cancelled = KotKindStyles::default();
        cancelled.title = KotFieldStyle::bold_center(KOT_EM_XLARGE);
        cancelled.item_qty = KotFieldStyle::light(KOT_EM_LARGE);
        cancelled.banner = KOT_BANNER_CANCELLED.into();
        cancelled.item_prefix = KOT_PREFIX_CANCELLED.into();
        cancelled.strike_item_name = true;
        cancelled.strike_item_note = false;
        cancelled.strike_items = true;
        Self {
            original: KotKindStyles::default(),
            duplicate,
            modified,
            cancelled,
        }
    }
}

impl KotKindStyleSet {
    pub fn get(&self, kind: KotSlipKind) -> &KotKindStyles {
        match kind {
            KotSlipKind::Original => &self.original,
            KotSlipKind::Duplicate => &self.duplicate,
            KotSlipKind::Modified => &self.modified,
            KotSlipKind::Cancelled => &self.cancelled,
        }
    }

    pub fn get_mut(&mut self, kind: KotSlipKind) -> &mut KotKindStyles {
        match kind {
            KotSlipKind::Original => &mut self.original,
            KotSlipKind::Duplicate => &mut self.duplicate,
            KotSlipKind::Modified => &mut self.modified,
            KotSlipKind::Cancelled => &mut self.cancelled,
        }
    }

    /// One-shot upgrade: fill empty banner/prefix and set product strike defaults.
    fn apply_product_chrome_defaults(&mut self) {
        for kind in KotSlipKind::ALL {
            let styles = self.get_mut(kind);
            if styles.banner.trim().is_empty() {
                styles.banner = kind.default_banner().into();
            }
            if styles.item_prefix.trim().is_empty() {
                styles.item_prefix = kind.default_item_prefix().into();
            }
            let strike_name = kind.default_strike_items();
            styles.strike_item_name = strike_name;
            styles.strike_item_note = false;
            styles.strike_items = strike_name;
        }
    }

    pub fn normalize_strike_flags(&mut self) {
        for kind in KotSlipKind::ALL {
            self.get_mut(kind).normalize_strike_flags();
        }
    }
}

/// Owner-tunable kitchen slip layout. Always a standard KOT (no prices;
/// dishes always print). Metrics are clamped wide so most heads fit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KotSlipFormat {
    /// Print the big title line (`title` or default). Off for compact slips.
    #[serde(default)]
    pub show_title: bool,
    /// Print the shop / place name under the title.
    #[serde(default)]
    pub show_shop_name: bool,
    #[serde(default = "default_true")]
    pub show_time: bool,
    /// Off in the compact product default (time / KOT no / area / table).
    #[serde(default)]
    pub show_waiter: bool,
    #[serde(default)]
    pub show_station: bool,
    /// Print table line when the ticket has a table.
    #[serde(default = "default_true")]
    pub show_table: bool,
    /// Print KOT number when numbered.
    #[serde(default = "default_true")]
    pub show_kot_no: bool,
    /// Floor area / service type (e.g. Dine In).
    #[serde(default = "default_true")]
    pub show_area: bool,
    /// `Item` / `Qty.` column headers above the dish list.
    #[serde(default = "default_true")]
    pub show_item_qty_header: bool,
    /// Soft-wrap width in characters. Clamped to [`KOT_CHARS_MIN`]..=[`KOT_CHARS_MAX`].
    /// Follows [`Self::content_width_mm`] when that is set.
    #[serde(default = "default_kot_chars")]
    pub chars_per_line: u32,
    /// Content width in millimetres for the kitchen ticket only (bill has its own).
    /// `0` means an older slip that only stored [`Self::chars_per_line`].
    /// New shops use [`Default`] (`80`).
    #[serde(default)]
    pub content_width_mm: u32,
    /// Thermal type size (raster). Clamped to [`KOT_EM_MIN`]..=[`KOT_EM_MAX`].
    #[serde(default = "default_kot_em")]
    pub em: u32,
    /// Side margin for raster (px). Wider = more inset on the paper.
    #[serde(default = "default_kot_side")]
    pub side: f32,
    /// How many identical slips to send per KOT. Clamped to [`KOT_COPIES_MIN`]..=[`KOT_COPIES_MAX`].
    #[serde(default = "default_kot_copies")]
    pub copies: u32,
    /// Optional title under the cancel/reprint banner. Empty → `KOT - KITCHEN ORDER`.
    /// Max [`KOT_TITLE_MAX`] chars; never prices.
    #[serde(default)]
    pub title: String,
    /// Optional kitchen note under the items (never prices). Max [`KOT_FOOTER_MAX`] chars.
    #[serde(default)]
    pub footer: String,
    /// Field order for header lines (ids: time, kot_no, area, table, shop, waiter, station).
    /// Empty → built-in order.
    #[serde(default)]
    pub field_order: Vec<String>,
    /// Per KOT kind (original / duplicate / modified / cancelled) field typography.
    #[serde(default)]
    pub kind_styles: KotKindStyleSet,
    /// `0` = older save without kind banner/prefix/strike chrome → apply product
    /// defaults once in [`Self::clamped`]. `1` = chrome already owned by the shop.
    #[serde(default)]
    pub kind_chrome_v: u32,
    /// Space between Item and Qty columns (em). Default comfortable for FOH.
    #[serde(default = "default_kot_item_col_gap_em")]
    pub item_col_gap_em: f32,
    /// Minimum Qty column width (em) so short digits align under `Qty.`.
    #[serde(default = "default_kot_item_qty_min_em")]
    pub item_qty_min_em: f32,
    /// Align qty digits inside the Qty slot.
    #[serde(default = "default_kot_item_num_align")]
    pub item_num_align: KotAlign,
    /// Header text for the Item column. Empty → `Item`.
    #[serde(default = "default_bill_header_item")]
    pub item_header_item: String,
    /// Header text for the Qty column. Empty → `Qty.`.
    #[serde(default = "default_bill_header_qty")]
    pub item_header_qty: String,
}

fn default_kot_item_col_gap_em() -> f32 {
    0.7
}
fn default_kot_item_qty_min_em() -> f32 {
    3.5
}
fn default_kot_item_num_align() -> KotAlign {
    KotAlign::End
}

/// KOT qty column min-width presets (em).
pub const KOT_QTY_MIN_COMPACT: f32 = 2.8;
pub const KOT_QTY_MIN_COMFORT: f32 = 3.5;
pub const KOT_QTY_MIN_WIDE: f32 = 4.5;
/// KOT column gap presets (em). Tight matches legacy; Comfortable is product default.
pub const KOT_COL_GAP_TIGHT: f32 = 0.35;
pub const KOT_COL_GAP_COMFORT: f32 = 0.7;
pub const KOT_COL_GAP_WIDE: f32 = 1.2;

pub const KOT_CHARS_MIN: u32 = 16;
/// Wide enough for a 210 mm page (~0.55 cols/mm). 58 mm is 32, 80 mm is 44.
pub const KOT_CHARS_MAX: u32 = 128;
pub const KOT_CHARS_58MM: u32 = 32;
pub const KOT_CHARS_80MM: u32 = 42;
pub const KOT_EM_MIN: u32 = 12;
pub const KOT_EM_MAX: u32 = 48;
pub const KOT_EM_SMALL: u32 = 20;
pub const KOT_EM_NORMAL: u32 = 24;
pub const KOT_EM_LARGE: u32 = 28;
/// Body / table size for the product KOT default (matches common FOH slips).
pub const KOT_EM_XLARGE: u32 = 32;
pub const KOT_FOOTER_MAX: usize = 80;
pub const KOT_TITLE_MAX: usize = 48;
pub const KOT_COPIES_MIN: u32 = 1;
pub const KOT_COPIES_MAX: u32 = 5;
pub const KOT_DEFAULT_TITLE: &str = "KOT - KITCHEN ORDER";
pub const KOT_SIDE_MIN: f32 = 0.0;
pub const KOT_SIDE_MAX: f32 = 48.0;

fn default_kot_chars() -> u32 { wrap_cols_for_paper_mm(80) }
fn default_kot_em() -> u32 { KOT_EM_NORMAL }
fn default_kot_copies() -> u32 { KOT_COPIES_MIN }
fn default_kot_side() -> f32 { 4.0 }

fn scrub_kot_text(raw: &str, max: usize) -> String {
    raw.chars()
        .filter(|c| *c != '₹' && *c != '$' && *c != '€')
        .collect::<String>()
        .trim()
        .chars()
        .take(max)
        .collect()
}

impl Default for KotSlipFormat {
    /// Product-wide compact kitchen slip (time, KOT no, area, table, Item/Qty).
    /// Matches common Indian FOH KOTs; shops can turn fields back on in Settings.
    fn default() -> Self {
        Self {
            show_title: false,
            show_shop_name: false,
            show_time: true,
            show_waiter: false,
            show_station: false,
            show_table: true,
            show_kot_no: true,
            show_area: true,
            show_item_qty_header: true,
            chars_per_line: wrap_cols_for_paper_mm(80),
            content_width_mm: 80,
            // Match common FOH kitchen slips (Rajdhani sample): large bold body.
            em: KOT_EM_LARGE,
            side: 8.0,
            copies: KOT_COPIES_MIN,
            title: String::new(),
            footer: String::new(),
            field_order: vec![
                "time".into(),
                "kot_no".into(),
                "area".into(),
                "table".into(),
            ],
            kind_styles: KotKindStyleSet::default(),
            kind_chrome_v: 1,
            item_col_gap_em: default_kot_item_col_gap_em(),
            item_qty_min_em: default_kot_item_qty_min_em(),
            item_num_align: default_kot_item_num_align(),
            item_header_item: default_bill_header_item(),
            item_header_qty: default_bill_header_qty(),
        }
    }
}

impl KotSlipFormat {
    /// Alias for [`Default`] (kept for call sites / templates).
    pub fn rajdhani_default() -> Self {
        Self::default().clamped()
    }

    /// Raster / ESC/POS column metrics for Item | Qty.
    pub fn item_tab_layout(&self) -> crate::receipt_raster::TabColumnLayout {
        crate::receipt_raster::TabColumnLayout {
            gap_em: clamp_bill_col_gap_em(self.item_col_gap_em),
            min_em: [
                clamp_bill_col_min_em(self.item_qty_min_em),
                0.0,
                0.0,
                0.0,
            ],
            cell_align: match self.item_num_align {
                KotAlign::Center => crate::receipt_raster::LineAlign::Center,
                KotAlign::End => crate::receipt_raster::LineAlign::End,
                KotAlign::Start => crate::receipt_raster::LineAlign::Start,
            },
            sep: false,
        }
    }

    pub fn set_item_col_gap_preset(&mut self, gap_em: f32) {
        self.item_col_gap_em = clamp_bill_col_gap_em(gap_em);
    }

    pub fn item_col_gap_preset(&self) -> f32 {
        let g = clamp_bill_col_gap_em(self.item_col_gap_em);
        let opts = [KOT_COL_GAP_TIGHT, KOT_COL_GAP_COMFORT, KOT_COL_GAP_WIDE];
        *opts
            .iter()
            .min_by(|a, b| {
                (*a - g)
                    .abs()
                    .partial_cmp(&(*b - g).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&KOT_COL_GAP_COMFORT)
    }

    pub fn set_item_qty_min_preset(&mut self, min_em: f32) {
        self.item_qty_min_em = clamp_bill_col_min_em(min_em);
    }

    pub fn item_qty_min_preset(&self) -> f32 {
        let have = clamp_bill_col_min_em(self.item_qty_min_em);
        let opts = [KOT_QTY_MIN_COMPACT, KOT_QTY_MIN_COMFORT, KOT_QTY_MIN_WIDE];
        *opts
            .iter()
            .min_by(|a, b| {
                (*a - have)
                    .abs()
                    .partial_cmp(&(*b - have).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&KOT_QTY_MIN_COMFORT)
    }

    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::default();
        }
        serde_json::from_str::<Self>(trimmed)
            .unwrap_or_default()
            .clamped()
    }

    /// Millimetres to show for this slip. Older slips with no stored width
    /// read as 58 mm when the column count is the narrow default, otherwise 80 mm.
    pub fn shown_content_width_mm(&self) -> u32 {
        if self.content_width_mm > 0 {
            self.content_width_mm.clamp(40, 210)
        } else if self.chars_per_line <= 34 {
            58
        } else {
            80
        }
    }

    pub fn clamped(mut self) -> Self {
        if self.content_width_mm > 0 {
            self.content_width_mm = self.content_width_mm.clamp(40, 210);
            self.chars_per_line = wrap_cols_for_paper_mm(self.content_width_mm);
        }
        self.chars_per_line = self.chars_per_line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX);
        self.em = self.em.clamp(KOT_EM_MIN, KOT_EM_MAX);
        self.copies = self.copies.clamp(KOT_COPIES_MIN, KOT_COPIES_MAX);
        self.side = self.side.clamp(KOT_SIDE_MIN, KOT_SIDE_MAX);
        self.item_col_gap_em = clamp_bill_col_gap_em(self.item_col_gap_em);
        self.item_qty_min_em = clamp_bill_col_min_em(self.item_qty_min_em);
        self.item_header_item =
            scrub_bill_header_label(&self.item_header_item, "Item");
        self.item_header_qty =
            scrub_bill_header_label(&self.item_header_qty, "Qty.");
        // Strip currency marks — a KOT must never carry prices.
        self.title = scrub_kot_text(&self.title, KOT_TITLE_MAX);
        self.footer = scrub_kot_text(&self.footer, KOT_FOOTER_MAX);
        for kind in KotSlipKind::ALL {
            let styles = self.kind_styles.get_mut(kind);
            styles.banner = scrub_kot_text(&styles.banner, KOT_BANNER_MAX);
            styles.item_prefix = scrub_kot_text(&styles.item_prefix, KOT_PREFIX_MAX);
            for field in KotStyleField::ALL {
                let s = styles.get_mut(field);
                s.size_em = s.size_em.clamp(KOT_EM_MIN, KOT_EM_MAX);
            }
        }
        // Older saves stored the same stock styles on every kind, so Duplicate /
        // Modified / Cancelled looked like dead tabs. Only lift that untouched copy.
        let stock = KotKindStyles::default();
        if self.kind_styles.original == stock
            && self.kind_styles.duplicate == stock
            && self.kind_styles.modified == stock
            && self.kind_styles.cancelled == stock
        {
            self.kind_styles = KotKindStyleSet::default();
            self.kind_chrome_v = 1;
        }
        // Older slips had hardcoded banners and no item prefix / strike. Lift once.
        if self.kind_chrome_v < 1 {
            self.kind_styles.apply_product_chrome_defaults();
            self.kind_chrome_v = 1;
        }
        // Legacy `strike_items` → name-only; notes stay independently optional.
        self.kind_styles.normalize_strike_flags();
        self
    }

    pub fn title_line(&self) -> &str {
        let t = self.title.trim();
        if t.is_empty() {
            KOT_DEFAULT_TITLE
        } else {
            t
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.clone().clamped()).unwrap_or_else(|_| "{}".into())
    }

    /// Apply type size / wrap width onto the built-in KOT print format.
    pub fn apply_to_print_format(&self, format: &mut PrintFormat) {
        self.apply_to_print_format_for_kind(format, KotSlipKind::Original);
    }

    /// Like [`Self::apply_to_print_format`], using the body (item/qty) size for `kind`.
    pub fn apply_to_print_format_for_kind(&self, format: &mut PrintFormat, kind: KotSlipKind) {
        let c = self.clone().clamped();
        let body_em = c
            .kind_styles
            .get(kind)
            .item_qty
            .size_em
            .clamp(KOT_EM_MIN, KOT_EM_MAX);
        // Keep legacy `em` in sync for older readers; prefer kind body size.
        format.em = body_em as f32;
        format.line = c.chars_per_line;
        format.baseline = (body_em as f32) + 2.0;
        format.side = c.side;
    }

    /// Style for one field on one kind (clamped size).
    pub fn field_style(&self, kind: KotSlipKind, field: KotStyleField) -> KotFieldStyle {
        let mut s = self.kind_styles.get(kind).get(field).clone();
        s.size_em = s.size_em.clamp(KOT_EM_MIN, KOT_EM_MAX);
        s
    }

    /// Whether this field prints on the slip (show/hide).
    pub fn field_shown(&self, field: KotStyleField) -> bool {
        match field {
            KotStyleField::Title => self.show_title,
            KotStyleField::Shop => self.show_shop_name,
            KotStyleField::Time => self.show_time,
            KotStyleField::Waiter => self.show_waiter,
            KotStyleField::Station => self.show_station,
            KotStyleField::Table => self.show_table,
            KotStyleField::KotNo => self.show_kot_no,
            KotStyleField::Area => self.show_area,
            KotStyleField::ItemQty => self.show_item_qty_header,
        }
    }

    pub fn set_field_shown(&mut self, field: KotStyleField, shown: bool) {
        match field {
            KotStyleField::Title => self.show_title = shown,
            KotStyleField::Shop => self.show_shop_name = shown,
            KotStyleField::Time => self.show_time = shown,
            KotStyleField::Waiter => self.show_waiter = shown,
            KotStyleField::Station => self.show_station = shown,
            KotStyleField::Table => self.show_table = shown,
            KotStyleField::KotNo => self.show_kot_no = shown,
            KotStyleField::Area => self.show_area = shown,
            KotStyleField::ItemQty => self.show_item_qty_header = shown,
        }
    }

    /// Soft-wrap one line to `chars_per_line` (break on spaces when possible).
    pub fn wrap_line(&self, line: &str) -> Vec<String> {
        let cols = self.chars_per_line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX) as usize;
        wrap_text_cols(line, cols)
    }

    /// On-screen Preview type size (pt). Thermal `em` still drives print raster.
    pub fn preview_font_pt(&self) -> f32 {
        let em = self.em.clamp(KOT_EM_MIN, KOT_EM_MAX);
        if em <= KOT_EM_SMALL {
            11.0
        } else if em >= KOT_EM_LARGE {
            17.0
        } else {
            13.5
        }
    }

    /// On-screen paper strip width (px) from wrap columns + Preview type size.
    pub fn preview_paper_width_px(&self) -> f64 {
        let cols = self.chars_per_line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX) as f64;
        let font = f64::from(self.preview_font_pt());
        // ~0.62em per character — wide enough that 58 mm vs 80 mm is obvious.
        (cols * font * 0.62 + 28.0).clamp(168.0, 460.0)
    }

    /// Sample kitchen slip for Settings preview (never prices).
    pub fn render_sample(&self, shop_name: &str) -> String {
        self.render_sample_kind(shop_name, KotSlipKind::Original)
    }

    /// Preview for a chosen KOT kind (Settings kind chips).
    pub fn render_sample_kind(&self, shop_name: &str, kind: KotSlipKind) -> String {
        self.render(KotSlipContent {
            kind,
            cancel: kind == KotSlipKind::Cancelled,
            reprint: kind == KotSlipKind::Duplicate,
            shop_name,
            kot_no: 12,
            kot_nos_label: None,
            when: "04/11/26 22:47",
            created_by: "Ramesh",
            table_label: "D1",
            station_name: "Kitchen",
            area_label: "Dine In",
            items: &[
                "Butter Roti|3",
                "Roasted Papad|2",
                "Plain Roti|2",
                "Butter Pav|2",
            ],
        })
    }

    pub fn render(&self, c: KotSlipContent<'_>) -> String {
        self.render_dir(c, false)
    }

    /// `rtl` swaps start and end on the printed dots.
    pub fn render_dir(&self, c: KotSlipContent<'_>, rtl: bool) -> String {
        let fmt = self.clone().clamped();
        let kind = if c.cancel {
            KotSlipKind::Cancelled
        } else {
            c.kind
        };
        let styles = fmt.kind_styles.get(kind);
        let cols = fmt.chars_per_line as usize;
        let rule = "-".repeat(cols);
        let compact = !fmt.show_title;
        let mut out = String::new();
        out.push_str(&encode_slip_tab_layout(&fmt.item_tab_layout()));
        let push_styled = |out: &mut String, style: &KotFieldStyle, line: &str, _center: bool| {
            // Alignment is in the style mark and applied in dots (and in the
            // preview). Space-padding a short date left a wide gap.
            let mark = kot_style_prefix(style, rtl);
            for part in fmt.wrap_line(line) {
                if part.trim().is_empty() {
                    continue;
                }
                out.push_str(&mark);
                out.push_str(&part);
                out.push('\n');
            }
        };
        if c.reprint {
            // Recovery reprint uses Duplicate chrome — honour the owner's
            // Duplicate banner when set; else the stock reprint line.
            let banner = styles.banner.trim();
            let line = if !banner.is_empty() {
                banner
            } else {
                KOT_BANNER_REPRINT
            };
            push_styled(&mut out, &styles.title, line, false);
        } else {
            let banner = styles.banner.trim();
            if !banner.is_empty() {
                push_styled(&mut out, &styles.title, banner, false);
            }
        }
        // Shop title prints on every kind, under the kind banner. Cancelled used
        // to skip it, so a custom title never appeared in that preview.
        if fmt.show_title {
            push_styled(&mut out, &styles.title, fmt.title_line(), false);
        }
        let emit_field = |out: &mut String, id: &str| {
            let style_field = match id {
                "shop" => KotStyleField::Shop,
                "kot_no" => KotStyleField::KotNo,
                "time" => KotStyleField::Time,
                "waiter" => KotStyleField::Waiter,
                "table" => KotStyleField::Table,
                "station" => KotStyleField::Station,
                "area" => KotStyleField::Area,
                _ => return,
            };
            let style = styles.get(style_field);
            match id {
                "shop" if fmt.show_shop_name => {
                    let shop = c.shop_name.trim();
                    if !shop.is_empty() {
                        push_styled(out, style, shop, true);
                    }
                }
                "kot_no" if fmt.show_kot_no => {
                    if let Some(label) = c.kot_nos_label.filter(|s| !s.trim().is_empty()) {
                        push_styled(out, style, label, true);
                    } else if c.kot_no > 0 {
                        if c.reprint {
                            push_styled(out, style, &format!("Was KOT No: {}", c.kot_no), true);
                        } else if compact {
                            push_styled(out, style, &format!("KOT - {}", c.kot_no), true);
                        } else {
                            push_styled(out, style, &format!("KOT No: {}", c.kot_no), false);
                        }
                    }
                }
                "time" if fmt.show_time && !c.when.is_empty() => {
                    // Align comes from the field style (product default: Center).
                    if compact {
                        push_styled(out, style, c.when, true);
                    } else {
                        push_styled(out, style, &format!("Time: {}", c.when), false);
                    }
                }
                "waiter" if fmt.show_waiter && !c.created_by.is_empty() => {
                    // Who sent *this* KOT — not every line author, not the re-printer.
                    push_styled(out, style, &format!("Waiter: {}", c.created_by), false);
                }
                "table" if fmt.show_table && !c.table_label.is_empty() => {
                    if compact {
                        push_styled(out, style, &format!("Table No: {}", c.table_label), true);
                    } else {
                        push_styled(out, style, &format!("Table: {}", c.table_label), false);
                    }
                }
                "station" if fmt.show_station && !c.station_name.is_empty() => {
                    push_styled(out, style, &format!("Station: {}", c.station_name), false);
                }
                "area" if fmt.show_area && !c.area_label.trim().is_empty() => {
                    push_styled(out, style, c.area_label.trim(), true);
                }
                _ => {}
            }
        };
        // Compact default order matches the client sample; never shop-first.
        let order: Vec<&str> = if fmt.field_order.is_empty() {
            vec!["time", "kot_no", "area", "table", "shop", "waiter", "station"]
        } else {
            fmt.field_order.iter().map(String::as_str).collect()
        };
        for id in &order {
            emit_field(&mut out, id);
        }
        for id in ["time", "kot_no", "area", "table", "shop", "waiter", "station"] {
            if !order.iter().any(|x| *x == id) {
                emit_field(&mut out, id);
            }
        }
        out.push_str(&rule);
        out.push('\n');
        let item_style = &styles.item_qty;
        let item_mark = kot_style_prefix(item_style, rtl);
        let item_mark_strike_name =
            kot_style_prefix_strike(item_style, rtl, styles.strike_item_name);
        let item_mark_strike_note =
            kot_style_prefix_strike(item_style, rtl, styles.strike_item_note);
        let prefix = styles.item_prefix.as_str();
        // Item / Qty. as two columns. `\t` marks the split: proportional
        // Noto raster right-aligns the qty in dots; ESC/POS text + preview
        // expand the tab to a fixed monospace qty field (see
        // expand_kot_column_tabs). Never pad with char-count spaces alone —
        // Gujarati names made "Qty." and the digits drift (kot.jpeg).
        if fmt.show_item_qty_header {
            // Header never takes the kind prefix or strike.
            out.push_str(&item_mark);
            out.push_str(&scrub_bill_header_label(&fmt.item_header_item, "Item"));
            out.push('\t');
            out.push_str(&scrub_bill_header_label(&fmt.item_header_qty, "Qty."));
            out.push('\n');
        }
        for item in c.items {
            let (raw_name, qty) = kot_item_parts(item);
            let (name, note) = kot_split_trailing_note(&raw_name);
            let labeled = if prefix.is_empty() {
                name
            } else if name.is_empty() {
                prefix.trim_end().to_string()
            } else {
                format!("{prefix}{name}")
            };
            let name_mark = if styles.strike_item_name {
                &item_mark_strike_name
            } else {
                &item_mark
            };
            if qty.is_empty() {
                out.push_str(name_mark);
                out.push_str(&labeled);
                out.push('\n');
            } else {
                out.push_str(name_mark);
                out.push_str(&labeled);
                out.push('\t');
                out.push_str(&qty);
                out.push('\n');
            }
            // Note is its own line so strike on the name never paints through it.
            if !note.is_empty() {
                let note_mark = if styles.strike_item_note {
                    &item_mark_strike_note
                } else {
                    &item_mark
                };
                out.push_str(note_mark);
                out.push('(');
                out.push_str(&note);
                out.push_str(")\n");
            }
        }
        if !fmt.footer.is_empty() {
            out.push_str(&rule);
            out.push('\n');
            push_styled(&mut out, item_style, &fmt.footer, false);
        }
        out
    }

    /// Old product default: title / shop / waiter / station all on.
    /// Used once to migrate shops onto the compact client sample.
    pub fn looks_like_legacy_verbose(&self) -> bool {
        self.show_title
            || (self.show_shop_name && self.show_waiter && self.show_station)
            || (self.field_order.is_empty()
                && self.show_waiter
                && self.show_station
                && !self.show_area)
    }

    /// Pre–FOH type sizes: small body, left-aligned headers, regular items.
    /// Shops still on that stock get one-shot lift to Rajdhani-like defaults.
    pub fn looks_like_pre_foh_type(&self) -> bool {
        let o = &self.kind_styles.original;
        self.em <= KOT_EM_SMALL
            && o.time.size_em <= KOT_EM_SMALL
            && o.item_qty.size_em <= KOT_EM_NORMAL
            && o.item_qty.weight == KotFontWeight::Regular
            && o.table.size_em <= KOT_EM_LARGE
            && o.table.align == KotAlign::Start
            && o.kot_no.align == KotAlign::Start
    }

    /// Apply the product FOH type defaults without wiping field show/order.
    pub fn apply_foh_type_defaults(&mut self) {
        let chrome = self.kind_styles.clone();
        self.kind_styles = KotKindStyleSet::default();
        // Keep owner-edited banners / prefixes / strike when already owned.
        if self.kind_chrome_v >= 1 {
            for kind in KotSlipKind::ALL {
                let from = chrome.get(kind);
                let to = self.kind_styles.get_mut(kind);
                if !from.banner.is_empty() {
                    to.banner = from.banner.clone();
                }
                if !from.item_prefix.is_empty() {
                    to.item_prefix = from.item_prefix.clone();
                }
                to.strike_item_name = from.strike_item_name;
                to.strike_item_note = from.strike_item_note;
                to.strike_items = from.strike_items;
            }
        }
        self.em = KOT_EM_LARGE;
        self.side = self.side.max(8.0);
        if self.content_width_mm == 0 {
            self.content_width_mm = 80;
            self.chars_per_line = wrap_cols_for_paper_mm(80);
        }
        self.kind_chrome_v = self.kind_chrome_v.max(1);
    }
}

/// Fields for one kitchen slip body (real KOT or Settings sample).
pub struct KotSlipContent<'a> {
    /// Which typography / banner family to use.
    pub kind: KotSlipKind,
    pub cancel: bool,
    /// Recovery slip after selective reprint (banner + "Was KOT No").
    pub reprint: bool,
    pub shop_name: &'a str,
    pub kot_no: i64,
    /// When set, replaces the usual KOT No line (e.g. `"Was KOT No: 12, 15"`).
    pub kot_nos_label: Option<&'a str>,
    pub when: &'a str,
    pub created_by: &'a str,
    pub table_label: &'a str,
    pub station_name: &'a str,
    /// Floor area / service type (Dine In, Parcel, …).
    pub area_label: &'a str,
    /// Dish lines. Prefer `Name|Qty` for column layout; plain text still works.
    pub items: &'a [&'a str],
}

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
    Table,
    Items,
    Total,
    Footer,
    Upi,
}

impl BillStyleField {
    pub const ALL: [BillStyleField; 13] = [
        BillStyleField::Shop,
        BillStyleField::Header,
        BillStyleField::Fssai,
        BillStyleField::Title,
        BillStyleField::Customer,
        BillStyleField::BillNo,
        BillStyleField::Time,
        BillStyleField::Cashier,
        BillStyleField::Table,
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
            BillStyleField::Table => "Table",
            BillStyleField::Items => "Items",
            BillStyleField::Total => "Total",
            BillStyleField::Footer => "Footer",
            BillStyleField::Upi => "UPI",
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

    /// Meta fields that may share a line as left\\tright (Date | Table, …).
    pub fn is_pairable(self) -> bool {
        matches!(
            self,
            BillStyleField::Customer
                | BillStyleField::BillNo
                | BillStyleField::Time
                | BillStyleField::Cashier
                | BillStyleField::Table
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
            BillStyleField::Table => "table",
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
    pub table: BillFieldStyle,
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
    /// Space between item columns, in em of the Items type size.
    /// Product default ~1.15 (comfortable); legacy tight pack was ~0.35.
    #[serde(default = "default_bill_item_col_gap_em")]
    pub item_col_gap_em: f32,
    /// Minimum Qty column width (em). Keeps short digits from crowding Price.
    #[serde(default = "default_bill_item_qty_min_em")]
    pub item_qty_min_em: f32,
    /// Minimum Price column width (em).
    #[serde(default = "default_bill_item_price_min_em")]
    pub item_price_min_em: f32,
    /// Minimum Amount column width (em).
    #[serde(default = "default_bill_item_amount_min_em")]
    pub item_amount_min_em: f32,
    /// Align numbers inside each Qty/Price/Amount slot (End = flush-right).
    #[serde(default = "default_bill_item_num_align")]
    pub item_num_align: KotAlign,
    /// Print order of piece columns (`name`, `qty`, `price`, `amount`).
    /// Empty → product default. First shown column flexes on the left; the
    /// rest pack from the right.
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
    /// Field-id pairs printed on one line as `left\tright` when both show.
    /// Empty falls back to product defaults (time+table, cashier+bill_no).
    #[serde(default)]
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
}

fn default_bill_meta_pairs() -> Vec<[String; 2]> {
    vec![
        ["time".into(), "table".into()],
        ["cashier".into(), "bill_no".into()],
    ]
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

fn default_bill_item_col_gap_em() -> f32 {
    1.15
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
fn default_bill_item_qty_min_em() -> f32 {
    3.5
}
fn default_bill_item_price_min_em() -> f32 {
    7.5
}
fn default_bill_item_amount_min_em() -> f32 {
    8.5
}
fn default_bill_item_num_align() -> KotAlign {
    KotAlign::End
}

/// Preset gap chips for Settings (Tight / Comfortable / Wide).
pub const BILL_COL_GAP_TIGHT: f32 = 0.5;
pub const BILL_COL_GAP_COMFORT: f32 = 1.15;
pub const BILL_COL_GAP_WIDE: f32 = 1.75;

/// Preset number-column min widths (qty, price, amount) in em.
pub const BILL_NUM_MIN_COMPACT: [f32; 3] = [2.8, 6.0, 7.0];
pub const BILL_NUM_MIN_COMFORT: [f32; 3] = [3.5, 7.5, 8.5];
pub const BILL_NUM_MIN_WIDE: [f32; 3] = [4.5, 9.0, 10.5];

/// Clamp a column gap em into a sane printable range.
pub fn clamp_bill_col_gap_em(v: f32) -> f32 {
    if !v.is_finite() {
        return BILL_COL_GAP_COMFORT;
    }
    v.clamp(0.2, 3.0)
}

/// Clamp a numeric column min-width em.
pub fn clamp_bill_col_min_em(v: f32) -> f32 {
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
            table: BillFieldStyle::face(14, KotFontWeight::Regular, KotAlign::Start),
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
            item_col_gap_em: default_bill_item_col_gap_em(),
            item_qty_min_em: default_bill_item_qty_min_em(),
            item_price_min_em: default_bill_item_price_min_em(),
            item_amount_min_em: default_bill_item_amount_min_em(),
            item_num_align: default_bill_item_num_align(),
            item_col_order: default_bill_item_col_order(),
            item_col_sep: false,
            total_show_qty: true,
            total_show_subtotal: true,
            total_show_grand: true,
            total_qty_label: default_bill_total_qty_label(),
            total_subtotal_label: default_bill_total_subtotal_label(),
            total_grand_label: default_bill_total_grand_label(),
            total_paid_label: default_bill_total_paid_label(),
            meta_pairs: default_bill_meta_pairs(),
            field_order: Vec::new(),
            copies: 1,
            content_width_mm: 210,
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
        cells.join("\t")
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
    cells.join("\t")
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
        self.item_col_gap_em = clamp_bill_col_gap_em(self.item_col_gap_em);
        self.item_qty_min_em = clamp_bill_col_min_em(self.item_qty_min_em);
        self.item_price_min_em = clamp_bill_col_min_em(self.item_price_min_em);
        self.item_amount_min_em = clamp_bill_col_min_em(self.item_amount_min_em);
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
        if self.meta_pairs.is_empty() {
            self.meta_pairs = default_bill_meta_pairs();
        } else {
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
            self.meta_pairs = if pairs.is_empty() {
                default_bill_meta_pairs()
            } else {
                pairs
            };
        }
        self
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

    /// Raster / ESC/POS column metrics for Items (and other `\t` lines on the bill).
    /// Mins follow the trailing (right-packed) columns in print order.
    pub fn item_tab_layout(&self) -> crate::receipt_raster::TabColumnLayout {
        let cols = self.ordered_item_cols();
        let mut min_em = [0.0f32; 4];
        // Skip the flex (left) column; mins apply to right-packed slots.
        for (i, col) in cols.iter().skip(1).enumerate() {
            if i >= min_em.len() {
                break;
            }
            min_em[i] = match col {
                BillItemCol::Name => 0.0,
                BillItemCol::Qty => clamp_bill_col_min_em(self.item_qty_min_em),
                BillItemCol::Price => clamp_bill_col_min_em(self.item_price_min_em),
                BillItemCol::Amount => clamp_bill_col_min_em(self.item_amount_min_em),
            };
        }
        crate::receipt_raster::TabColumnLayout {
            gap_em: clamp_bill_col_gap_em(self.item_col_gap_em),
            min_em,
            cell_align: match self.item_num_align {
                KotAlign::Center => crate::receipt_raster::LineAlign::Center,
                KotAlign::End => crate::receipt_raster::LineAlign::End,
                KotAlign::Start => crate::receipt_raster::LineAlign::Start,
            },
            sep: self.item_col_sep,
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

    /// Pair `field` with `partner`, or clear its pair when `partner` is None.
    pub fn set_meta_partner(&mut self, field: BillStyleField, partner: Option<BillStyleField>) {
        if !field.is_pairable() {
            return;
        }
        if self.meta_pairs.is_empty() {
            self.meta_pairs = default_bill_meta_pairs();
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

    /// Apply a gap preset chip (Tight / Comfortable / Wide).
    pub fn set_item_col_gap_preset(&mut self, gap_em: f32) {
        self.item_col_gap_em = clamp_bill_col_gap_em(gap_em);
    }

    /// Apply a number-column width preset (Compact / Comfortable / Wide).
    pub fn set_item_num_min_preset(&mut self, mins: [f32; 3]) {
        self.item_qty_min_em = clamp_bill_col_min_em(mins[0]);
        self.item_price_min_em = clamp_bill_col_min_em(mins[1]);
        self.item_amount_min_em = clamp_bill_col_min_em(mins[2]);
    }

    /// Which gap preset chip is selected (nearest).
    pub fn item_col_gap_preset(&self) -> f32 {
        let g = clamp_bill_col_gap_em(self.item_col_gap_em);
        let opts = [BILL_COL_GAP_TIGHT, BILL_COL_GAP_COMFORT, BILL_COL_GAP_WIDE];
        *opts
            .iter()
            .min_by(|a, b| {
                (*a - g)
                    .abs()
                    .partial_cmp(&(*b - g).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&BILL_COL_GAP_COMFORT)
    }

    /// Which number-min preset chip is selected (nearest by L1).
    pub fn item_num_min_preset(&self) -> [f32; 3] {
        let have = [
            clamp_bill_col_min_em(self.item_qty_min_em),
            clamp_bill_col_min_em(self.item_price_min_em),
            clamp_bill_col_min_em(self.item_amount_min_em),
        ];
        let opts = [
            BILL_NUM_MIN_COMPACT,
            BILL_NUM_MIN_COMFORT,
            BILL_NUM_MIN_WIDE,
        ];
        *opts
            .iter()
            .min_by(|a, b| {
                let da = (a[0] - have[0]).abs()
                    + (a[1] - have[1]).abs()
                    + (a[2] - have[2]).abs();
                let db = (b[0] - have[0]).abs()
                    + (b[1] - have[1]).abs()
                    + (b[2] - have[2]).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&BILL_NUM_MIN_COMFORT)
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
            BillStyleField::Table => &self.table,
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
            BillStyleField::Table => &mut self.table,
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
        append_bill_field(&mut out, self, BillStyleField::Table, "Dine In: D1", false);
        append_bill_field(&mut out, self, BillStyleField::Cashier, "Cashier: biller", false);
        append_bill_field(&mut out, self, BillStyleField::BillNo, "Bill No.: SAMPLE", false);
        append_bill_like(&mut out, self, BillStyleField::Items, "--------------------", false);
        let header = format_bill_item_header(self);
        if !header.is_empty() {
            append_bill_like(&mut out, self, BillStyleField::Items, &header, false);
        }
        let tea = format_bill_item_block(
            self,
            BillItemPieces {
                name: "Tea",
                qty: "2",
                price: "40.00",
                amount: "80.00",
                note: "",
            },
        );
        append_bill_field(&mut out, self, BillStyleField::Items, &tea, false);
        let coffee = format_bill_item_block(
            self,
            BillItemPieces {
                name: "Coffee",
                qty: "1",
                price: "20.00",
                amount: "20.00",
                note: "",
            },
        );
        append_bill_field(&mut out, self, BillStyleField::Items, &coffee, false);
        append_bill_like(&mut out, self, BillStyleField::Items, "--------------------", false);
        if self.total_show_qty {
            append_bill_like(
                &mut out,
                self,
                BillStyleField::Total,
                &format!("{}: 3", self.caption_total_qty()),
                false,
            );
        }
        if self.total_show_subtotal {
            append_bill_like(
                &mut out,
                self,
                BillStyleField::Total,
                &format!("{}: 100.00", self.caption_subtotal()),
                false,
            );
        }
        if self.total_show_grand {
            append_bill_field(
                &mut out,
                self,
                BillStyleField::Total,
                &format!("{}: 100.00", self.caption_grand_total()),
                false,
            );
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

fn append_marked(out: &mut String, style: &BillFieldStyle, text: &str, rtl: bool) {
    let mark = bill_mark(style, rtl);
    for part in text.split(['\n', '|']) {
        let part = part.trim();
        if part.is_empty() {
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
        out.push_str(&encode_slip_tab_layout(&fmt.item_tab_layout()));
    }
    let mut consumed: std::collections::HashSet<BillStyleField> =
        std::collections::HashSet::new();
    let order = fmt.ordered();
    for field in &order {
        if consumed.contains(field) {
            continue;
        }
        // Pairable meta: one left\tright line when both partners have a
        // single shown text (Date | Table, Cashier | Bill no).
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
                        let (left_f, left_t, right_t) = if field_i <= other_i {
                            (*field, left_texts[0], right_texts[0])
                        } else {
                            (other, right_texts[0], left_texts[0])
                        };
                        append_marked(
                            &mut out,
                            fmt.style(left_f),
                            &format!("{left_t}\t{right_t}"),
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

/// Body text, alignment, and point size, for the on-screen slip preview.
pub fn kot_preview_line(line: &str) -> (KotAlign, &str) {
    let (align, _, body) = kot_preview_metrics(line);
    (align, body)
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
    // Current: two digits + weight (`24B`). Older jobs: S/N/L + weight.
    if b.len() >= 3 && b[0].is_ascii_digit() && b[1].is_ascii_digit() {
        let size = (b[0] - b'0') as u32 * 10 + (b[1] - b'0') as u32;
        let weight = match b[2] as char {
            'B' => KotFontWeight::Bold,
            'T' => KotFontWeight::Light,
            _ => KotFontWeight::Regular,
        };
        let (align, mut skip) = if b.len() >= 4 && matches!(b[3] as char, 'S' | 'C' | 'E') {
            (KotAlign::from_mark(b[3] as char), 4)
        } else {
            (KotAlign::Start, 3)
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

/// Encode column metrics as a leading slip directive (not drawn).
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

/// Pull a leading `\u{E001}T…` directive off `text`. Returns layout + body.
fn take_slip_tab_layout(
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

#[cfg(test)]
mod kot_slip_format_tests {
    use super::*;

    #[test]
    fn clamps_width_and_strips_currency_from_footer() {
        let fmt = KotSlipFormat {
            // Legacy path: mm unset so chars_per_line is clamped on its own.
            content_width_mm: 0,
            chars_per_line: 400,
            em: 5,
            footer: "Rush ₹100".into(),
            ..KotSlipFormat::default()
        }
        .clamped();
        assert_eq!(fmt.chars_per_line, KOT_CHARS_MAX);
        assert_eq!(fmt.em, KOT_EM_MIN);
        assert_eq!(fmt.footer, "Rush 100");
    }

    #[test]
    fn duplicate_reprint_uses_owner_banner_when_set() {
        let mut fmt = KotSlipFormat::default();
        fmt.kind_styles.duplicate.banner = "*** CUSTOM DUPLICATE ***".into();
        let text =
            strip_kot_style_marks(&fmt.render_sample_kind("Shop", KotSlipKind::Duplicate));
        assert!(
            text.contains("CUSTOM DUPLICATE"),
            "owner Duplicate banner must print on recovery reprint:\n{text}"
        );
        assert!(
            !text.contains("REPRINT - SAME ORDER"),
            "stock reprint line must not override a custom banner:\n{text}"
        );
    }

    #[test]
    fn cancelled_kind_defaults_prefix_banner_and_strike_mark() {
        let raw = KotSlipFormat::default().render_sample_kind("Shop", KotSlipKind::Cancelled);
        let text = strip_kot_style_marks(&raw);
        assert!(
            text.contains(KOT_BANNER_CANCELLED),
            "cancelled banner:\n{text}"
        );
        // Prefix is on the raw body before combining-strike preview overlays.
        assert!(
            raw.contains("[CANCELLED]Butter Roti\t") || raw.contains("[CANCELLED] Butter Roti\t"),
            "cancelled item prefix on raw slip:\n{raw}"
        );
        assert!(
            raw.lines().any(|l| {
                let (style, body) = take_kot_line_style(l);
                style.map(|s| s.strike).unwrap_or(false) && body.contains('\t')
            }),
            "item lines carry strike:\n{raw}"
        );
        let modified_raw =
            KotSlipFormat::default().render_sample_kind("Shop", KotSlipKind::Modified);
        let modified = strip_kot_style_marks(&modified_raw);
        assert!(modified.contains(KOT_BANNER_MODIFIED), "{modified}");
        assert!(
            modified_raw.contains("[NEW]Butter Roti\t") || modified_raw.contains("[NEW] Butter Roti\t"),
            "{modified_raw}"
        );
    }

    #[test]
    fn cancelled_strike_hits_name_not_note_by_default() {
        let mut fmt = KotSlipFormat::default();
        let styles = fmt.kind_styles.get_mut(KotSlipKind::Cancelled);
        assert!(styles.strike_item_name);
        assert!(!styles.strike_item_note);
        let raw = fmt.render_dir(
            KotSlipContent {
                kind: KotSlipKind::Cancelled,
                cancel: true,
                reprint: false,
                shop_name: "Shop",
                kot_no: 1,
                kot_nos_label: None,
                when: "07/11/26 12:00",
                created_by: "",
                table_label: "D1",
                station_name: "",
                area_label: "Dine In",
                items: &["Butter Roti (less spicy)|x2"],
            },
            false,
        );
        let name_struck = raw.lines().any(|l| {
            let (style, body) = take_kot_line_style(l);
            style.map(|s| s.strike).unwrap_or(false) && body.contains("Butter Roti") && body.contains('\t')
        });
        let note_struck = raw.lines().any(|l| {
            let (style, body) = take_kot_line_style(l);
            style.map(|s| s.strike).unwrap_or(false) && body.contains("less spicy")
        });
        let note_clear = raw.lines().any(|l| {
            let (style, body) = take_kot_line_style(l);
            !style.map(|s| s.strike).unwrap_or(false) && body.contains("(less spicy)")
        });
        assert!(name_struck, "dish name should strike:\n{raw}");
        assert!(!note_struck, "note must not strike by default:\n{raw}");
        assert!(note_clear, "note prints as its own clear line:\n{raw}");
    }

    #[test]
    fn bill_and_kot_column_headers_are_editable() {
        let mut bill = BillSlipFormat::default();
        bill.item_header_item = "Particulars".into();
        bill.item_header_qty = "Qty".into();
        bill.item_header_price = "Rate".into();
        bill.item_header_amount = "Amt".into();
        assert_eq!(
            format_bill_item_header(&bill),
            "Particulars\tQty\tRate\tAmt"
        );
        let mut kot = KotSlipFormat::default();
        kot.item_header_item = "Dish".into();
        kot.item_header_qty = "Nos".into();
        let raw = kot.render_sample("Shop");
        let text = strip_kot_style_marks(&raw);
        assert!(
            text.lines().any(|l| l == "Dish\tNos"),
            "KOT header labels must print:\n{text}"
        );
    }

    #[test]
    fn bill_column_order_sep_labels_and_meta_pair_are_owned() {
        let mut fmt = BillSlipFormat::default();
        fmt.item_col_order = vec![
            "name".into(),
            "amount".into(),
            "qty".into(),
            "price".into(),
        ];
        let row = format_bill_item_block(
            &fmt,
            BillItemPieces {
                name: "Tea",
                qty: "2",
                price: "40.00",
                amount: "80.00",
                note: "",
            },
        );
        assert_eq!(row, "Tea\t80.00\t2\t40.00");
        assert_eq!(
            format_bill_item_header(&fmt),
            "Item\tAmount\tQty.\tPrice"
        );
        let tab = fmt.item_tab_layout();
        // Right packs: amount, qty, price → mins amount, qty, price.
        assert!((tab.min_em[0] - 8.5).abs() < 0.01);
        assert!((tab.min_em[1] - 3.5).abs() < 0.01);
        assert!((tab.min_em[2] - 7.5).abs() < 0.01);
        fmt.item_col_sep = true;
        assert!(fmt.item_tab_layout().sep);
        fmt.total_qty_label = "Nos".into();
        fmt.total_grand_label = "NET".into();
        let sample = strip_kot_style_marks(&fmt.render_sample("Shop", "01/01/26 12:00"));
        assert!(sample.contains("Nos: 3"), "custom qty label:\n{sample}");
        assert!(sample.contains("NET: 100.00"), "custom grand label:\n{sample}");
        fmt.set_meta_partner(BillStyleField::Time, Some(BillStyleField::Cashier));
        assert_eq!(
            fmt.meta_pair_of(BillStyleField::Time),
            Some(BillStyleField::Cashier)
        );
        assert_eq!(fmt.meta_pair_of(BillStyleField::Table), None);
        fmt.set_meta_partner(BillStyleField::Time, None);
        assert_eq!(fmt.meta_pair_of(BillStyleField::Time), None);
    }

    #[test]
    fn bill_column_metrics_defaults_and_tab_directive_round_trip() {
        let fmt = BillSlipFormat::default();
        assert!((fmt.item_col_gap_em - BILL_COL_GAP_COMFORT).abs() < 0.01);
        assert_eq!(fmt.item_num_align, KotAlign::End);
        assert_eq!(fmt.items.size_em, 16);
        let tab = fmt.item_tab_layout();
        assert!((tab.gap_em - 1.15).abs() < 0.01);
        assert!((tab.min_em[0] - 3.5).abs() < 0.01);
        assert!((tab.min_em[1] - 7.5).abs() < 0.01);
        assert!((tab.min_em[2] - 8.5).abs() < 0.01);
        let mut hidden_price = fmt.clone();
        hidden_price.item_show_price = false;
        let tab2 = hidden_price.item_tab_layout();
        // Qty then Amount — Amount keeps amount min, not price min.
        assert!((tab2.min_em[0] - 3.5).abs() < 0.01);
        assert!((tab2.min_em[1] - 8.5).abs() < 0.01);
        assert_eq!(tab2.min_em[2], 0.0);
        let raw = fmt.render_sample("Shop", "01/01/26 12:00");
        assert!(
            raw.starts_with(SLIP_TAB_MARK),
            "bill sample must carry column metrics for the print queue:\n{raw}"
        );
        let (parsed, body) = take_slip_tab_layout(&raw);
        let parsed = parsed.expect("tab layout");
        assert!((parsed.gap_em - tab.gap_em).abs() < 0.02);
        assert!(!body.starts_with(SLIP_TAB_MARK));
        let lines = styled_slip_lines(&raw, &PrintFormat::standard().raster_metrics());
        assert!(
            lines.iter().any(|l| l.text.contains('\t') && l.tab.is_some()),
            "tab lines must keep layout through styled_slip_lines"
        );
        let plain = strip_kot_style_marks(&raw);
        assert!(!plain.contains(SLIP_TAB_MARK));
    }

    #[test]
    fn bill_item_columns_and_stacked_honour_piece_toggles() {
        let mut fmt = BillSlipFormat::default();
        assert_eq!(fmt.item_layout, BillItemLayout::Columns);
        let full = format_bill_item_block(
            &fmt,
            BillItemPieces {
                name: "Thali",
                qty: "5",
                price: "₹200.00",
                amount: "₹1,000.00",
                note: "less spicy",
            },
        );
        assert_eq!(full, "Thali\t5\t₹200.00\t₹1,000.00\n(less spicy)");
        assert_eq!(
            format_bill_item_header(&fmt),
            "Item\tQty.\tPrice\tAmount"
        );
        fmt.item_show_qty = false;
        fmt.item_show_note = false;
        let no_qty = format_bill_item_block(
            &fmt,
            BillItemPieces {
                name: "Thali",
                qty: "5",
                price: "₹200.00",
                amount: "₹1,000.00",
                note: "less spicy",
            },
        );
        assert_eq!(no_qty, "Thali\t₹200.00\t₹1,000.00");
        fmt.item_layout = BillItemLayout::Stacked;
        fmt.item_show_qty = true;
        fmt.item_show_price = true;
        fmt.item_show_amount = true;
        let stacked = format_bill_item_block(
            &fmt,
            BillItemPieces {
                name: "Thali",
                qty: "5",
                price: "₹200.00",
                amount: "₹1,000.00",
                note: "",
            },
        );
        assert_eq!(stacked, "Thali\n5 x ₹200.00\n₹1,000.00");
    }

    #[test]
    fn product_defaults_use_page_bill_and_80mm_kot() {
        assert_eq!(BillSlipFormat::default().shown_content_width_mm(), 210);
        let kot = KotSlipFormat::default();
        assert_eq!(kot.shown_content_width_mm(), 80);
        assert_eq!(kot.em, KOT_EM_LARGE);
        let body = &kot.kind_styles.original;
        assert_eq!(body.item_qty.size_em, KOT_EM_LARGE);
        assert_eq!(body.item_qty.weight, KotFontWeight::Bold);
        assert_eq!(body.table.align, KotAlign::Center);
        assert_eq!(body.kot_no.align, KotAlign::Center);
        assert!(!BillSlipFormat::default().title_style.show);
        assert_eq!(BillSlipFormat::default().footer, "Thanks");
        assert_eq!(KOT_BANNER_DUPLICATE, "Duplicate");
    }

    #[test]
    fn pre_foh_type_is_lifted_to_large_centered_defaults() {
        let mut old = KotSlipFormat {
            em: KOT_EM_SMALL,
            kind_styles: {
                let mut s = KotKindStyleSet::default();
                // Simulate the old stock (small, left, regular items).
                s.original = KotKindStyles {
                    title: KotFieldStyle::bold(KOT_EM_NORMAL),
                    shop: KotFieldStyle::regular(KOT_EM_NORMAL),
                    time: KotFieldStyle::regular(KOT_EM_SMALL),
                    waiter: KotFieldStyle::regular(KOT_EM_SMALL),
                    station: KotFieldStyle::regular(KOT_EM_SMALL),
                    table: KotFieldStyle::bold(KOT_EM_LARGE),
                    kot_no: KotFieldStyle::regular(KOT_EM_NORMAL),
                    area: KotFieldStyle::bold(KOT_EM_NORMAL),
                    item_qty: KotFieldStyle::regular(KOT_EM_NORMAL),
                    banner: String::new(),
                    item_prefix: String::new(),
                    strike_items: false,
                    strike_item_name: false,
                    strike_item_note: false,
                };
                s
            },
            ..KotSlipFormat::default()
        };
        // em SMALL + left table/kot overrides the new Default() fields above.
        old.em = KOT_EM_SMALL;
        assert!(old.looks_like_pre_foh_type());
        old.apply_foh_type_defaults();
        assert!(!old.looks_like_pre_foh_type());
        assert_eq!(old.em, KOT_EM_LARGE);
        assert_eq!(old.kind_styles.original.table.align, KotAlign::Center);
        assert_eq!(old.kind_styles.original.item_qty.weight, KotFontWeight::Bold);
    }

    #[test]
    fn bill_sample_pairs_meta_and_prints_column_header() {
        let raw = BillSlipFormat::default().render_sample("Rajdhani", "06/11/26 10:49");
        let text = strip_kot_style_marks(&raw);
        assert!(
            text.contains("Date: 06/11/26 10:49\tDine In: D1")
                || text.contains("Date: 06/11/26 10:49") && text.contains("Dine In: D1"),
            "meta should pair or at least print both:\n{text}"
        );
        assert!(
            text.contains("Item\tQty.\tPrice\tAmount") || text.contains("Item"),
            "column header:\n{text}"
        );
        assert!(
            text.contains("Tea\t2\t40.00\t80.00") || text.contains("Tea"),
            "columnar tea row:\n{text}"
        );
        assert!(text.contains("Total Qty: 3"), "total qty:\n{text}");
        assert!(text.contains("Grand Total"), "grand total:\n{text}");
    }

    #[test]
    fn sample_preview_never_shows_prices_and_keeps_kot_no() {
        let raw = KotSlipFormat::default().render_sample("Khaynjo");
        let cols = KotSlipFormat::default().chars_per_line as usize;
        let text = strip_kot_style_marks(&expand_kot_column_tabs(&raw, cols));
        // Product default is the compact slip (kot.jpeg): no title banner.
        assert!(!text.contains("KOT - KITCHEN ORDER"));
        assert!(text.contains("KOT - 12"));
        assert!(text.contains("Table No: D1"));
        assert!(text.contains("Dine In"));
        assert!(text.contains("Item"));
        assert!(text.contains("Qty."));
        assert!(text.contains("Butter Roti"));
        assert!(!text.contains("By:"));
        assert!(!text.contains("Station:"));
        // One rule under the header — not a second rule under the items.
        let rules = text
            .lines()
            .filter(|l| !l.is_empty() && l.chars().all(|c| c == '-'))
            .count();
        assert_eq!(rules, 1, "unexpected trailing rule:\n{text}");
        let date = text
            .lines()
            .find(|l| l.contains("04/11/26"))
            .unwrap_or("");
        assert!(
            !date.starts_with(' '),
            "the date starts at the left edge:\n{text}"
        );
        assert!(!text.contains('₹'));
        assert!(!text.to_lowercase().contains("total"));
        // Qty. header and every qty share the same right edge (monospace expand).
        let item_lines: Vec<&str> = text
            .lines()
            .filter(|l| l.contains("Qty.") || l.contains("Butter") || l.contains("Papad") || l.contains("Roti") || l.contains("Pav"))
            .filter(|l| !l.contains("Table") && !l.contains("KOT -"))
            .collect();
        assert!(item_lines.len() >= 2, "expected header + items:\n{text}");
        let ends: Vec<usize> = item_lines.iter().map(|l| l.chars().count()).collect();
        assert!(
            ends.windows(2).all(|w| w[0] == w[1]),
            "Item/Qty lines must share one right edge, got {ends:?}:\n{text}"
        );
        assert!(raw.contains('\t'), "render keeps tab for raster columns");
    }

    #[test]
    fn qty_column_is_a_count_not_an_x_prefix() {
        assert_eq!(kot_qty_column("x3"), "3");
        assert_eq!(kot_qty_column("×2"), "2");
        assert_eq!(kot_qty_column("X12"), "12");
        assert_eq!(kot_qty_column("3"), "3");
        assert_eq!(kot_qty_column("2 kg"), "2 kg");
        let text = KotSlipFormat::default().render(KotSlipContent {
            kind: KotSlipKind::Original,
            cancel: false,
            reprint: false,
            shop_name: "Shop",
            kot_no: 1,
            kot_nos_label: None,
            when: "now",
            created_by: "",
            table_label: "D1",
            station_name: "",
            area_label: "",
            items: &["Butter Roti|x3"],
        });
        assert!(text.contains("Butter Roti\t3"), "qty column must be the bare count:\n{text}");
        assert!(!text.contains("Butter Roti\tx"), "x prefix must not reach the qty column:\n{text}");
        assert_eq!(kot_item_parts("x1× ઈસી B.B.Q."), ("ઈસી B.B.Q.".into(), "1".into()));
        assert_eq!(kot_item_parts("ઈસી B.B.Q. x1"), ("ઈસી B.B.Q.".into(), "1".into()));
        assert_eq!(kot_item_parts("Butter Roti|x1×"), ("Butter Roti".into(), "1".into()));
        assert_eq!(
            kot_item_parts("x2 Butter Naan"),
            ("Butter Naan".into(), "2".into()),
            "digit qty then name (no ×) still splits Item | Qty"
        );
        assert_eq!(kot_qty_column("x1×"), "1");
        let glued = KotSlipFormat::default().render(KotSlipContent {
            kind: KotSlipKind::Original,
            cancel: false,
            reprint: false,
            shop_name: "Shop",
            kot_no: 1,
            kot_nos_label: None,
            when: "now",
            created_by: "",
            table_label: "D1",
            station_name: "",
            area_label: "",
            items: &["x1× ઈસી B.B.Q.", "ઈસી B.B.Q. x1"],
        });
        let bare = strip_kot_style_marks(&glued);
        assert!(bare.contains("ઈસી B.B.Q.\t1"), "count stays in the qty column:\n{bare}");
        assert!(!bare.contains("x1"), "x1 must not sit on the dish name:\n{bare}");
    }

    #[test]
    fn start_center_end_shift_across_the_content_width() {
        fn first_ink(align: crate::receipt_raster::LineAlign, dots: usize) -> usize {
            let page = crate::receipt_raster::RasterMetrics {
                em: 24.0,
                line: 36,
                baseline: 26.0,
                side: 4.0,
                width_dots: dots,
            };
            let lines = [crate::receipt_raster::StyledLine {
                text: "T2".into(),
                em: 24.0,
                weight: crate::receipt_raster::RasterWeight::Regular,
                align,
                strike: false,
                tab: None,
            }];
            let (rows, height) = crate::receipt_raster::draw_lines(&lines, &page);
            let width = page.page_dots();
            let row_bytes = width / 8;
            for y in 0..height {
                for x in 0..width {
                    if rows[y * row_bytes + x / 8] & (0x80 >> (x % 8)) != 0 {
                        return x;
                    }
                }
            }
            0
        }
        use crate::receipt_raster::LineAlign;
        let start = first_ink(LineAlign::Start, 576);
        let center = first_ink(LineAlign::Center, 576);
        let end = first_ink(LineAlign::End, 576);
        let center_narrow = first_ink(LineAlign::Center, 384);
        assert!(center > start + 40, "center {center} vs start {start}");
        assert!(end > center + 40, "end {end} vs center {center}");
        assert!(
            center > center_narrow + 40,
            "a wider content width moves center right ({center} vs {center_narrow})"
        );
        let mut fmt = KotSlipFormat::default();
        fmt.content_width_mm = 80;
        fmt.chars_per_line = wrap_cols_for_paper_mm(80);
        let styles = fmt.kind_styles.get_mut(KotSlipKind::Original);
        styles.time.align = KotAlign::Center;
        let text = fmt.render_sample_kind("Shop", KotSlipKind::Original);
        let metrics = PrintFormat {
            line: fmt.chars_per_line,
            ..PrintFormat::kot()
        }
        .raster_metrics();
        let lines = styled_slip_lines(&text, &metrics);
        assert!(
            lines.iter().any(|l| {
                l.text.contains("04/11/26") && l.align == crate::receipt_raster::LineAlign::Center
            }),
            "center is what the printer draws, at the content width"
        );
        assert_eq!(metrics.page_dots(), paper_width_dots(80));
    }

    #[test]
    fn bill_and_kitchen_use_the_same_escpos_width() {
        let text = "\u{E000}30BSCASH MEMO\n\u{E000}30BS--------------------\n\u{E000}30BSTea\n";
        let cols = wrap_cols_for_paper_mm(210);
        let bill = PrintFormat {
            line: cols,
            em: 30.0,
            document: false,
            delivery: DELIVERY_ESCPOS.into(),
            ..PrintFormat::standard_portrait()
        };
        let kot = PrintFormat {
            line: cols,
            em: 30.0,
            document: false,
            ..PrintFormat::kot()
        };
        let width = |job: &[u8]| {
            let gs = job
                .windows(8)
                .find(|w| w[..4] == [0x1D, 0x76, 0x30, 0x00])
                .expect("raster header");
            (gs[4] as usize + ((gs[5] as usize) << 8)) * 8
        };
        let dots = paper_width_dots(210);
        assert_eq!(width(&escpos_job_format(&bill, text)), dots);
        assert_eq!(width(&escpos_job_format(&kot, text)), dots);
    }

    #[test]
    fn preview_fits_a_big_page_and_refuses_an_unreadable_fit() {
        // A short wide page fits in the column and in the window. Type stays
        // large, so the fit scale is used and the card is not the full column
        // if the height limit is tighter.
        let (w, h) = preview_frame(3000.0, 600.0, 480.0, 500.0, 160.0);
        assert!(w <= 480.0 + 0.5, "wider than the column: {w}");
        assert!(h <= 500.0 + 0.5, "taller than the window slot: {h}");
        let scale = w / 3000.0;
        assert!((h / 600.0 - scale).abs() < 0.01, "width and height must share one scale");
        assert!(160.0 * scale >= 15.0, "fitted type went unreadably small");

        // Fitting this long slip into the window would make the type tiny.
        // Hold a readable size, stay inside the column, and let the page scroll.
        let (w, h) = preview_frame(1000.0, 8000.0, 400.0, 500.0, 40.0);
        let scale = w / 1000.0;
        assert!(40.0 * scale >= 14.5, "type {scale} is still unreadably small");
        assert!(w <= 400.0 + 0.5);
        assert!(h > 500.0, "the readable slip should extend past the window and scroll");
        assert!((h / 8000.0 - scale).abs() < 0.01);
    }

    #[test]
    fn page_png_names_its_density_and_still_decodes() {
        let png = crate::receipt_raster::png_document("CASH MEMO\n--------------------\nTotal 100\n")
            .unwrap();
        let stamped = png_with_density(png, 4724);
        assert!(stamped.windows(4).any(|w| w == *b"pHYs"));
        image::load_from_memory(&stamped).expect("density chunk broke the PNG");
    }

    #[test]
    fn field_size_and_weight_change_the_printed_slip() {
        let slip = |style: KotFieldStyle| {
            let mut fmt = KotSlipFormat::default();
            let styles = fmt.kind_styles.get_mut(KotSlipKind::Original);
            for field in KotStyleField::ALL {
                *styles.get_mut(field) = style.clone();
            }
            fmt.render(KotSlipContent {
                kind: KotSlipKind::Original,
                cancel: false,
                reprint: false,
                shop_name: "Shop",
                kot_no: 4,
                kot_nos_label: None,
                when: "now",
                created_by: "A",
                table_label: "D1",
                station_name: "Kitchen",
                area_label: "Dine In",
                items: &["Tea|2"],
            })
        };
        let raster_dots_high = |job: &[u8]| -> usize {
            let mut h = 0usize;
            let mut i = 0usize;
            while i + 8 <= job.len() {
                if job[i..i + 4] == [0x1D, 0x76, 0x30, 0x00] {
                    h += job[i + 6] as usize + ((job[i + 7] as usize) << 8);
                    i += 8;
                } else {
                    i += 1;
                }
            }
            h
        };
        let small = escpos_job_format(&PrintFormat::kot(), &slip(KotFieldStyle::light(16)));
        let large = escpos_job_format(&PrintFormat::kot(), &slip(KotFieldStyle::bold(40)));
        let mid_bold = escpos_job_format(&PrintFormat::kot(), &slip(KotFieldStyle::bold(24)));
        let mid_light = escpos_job_format(&PrintFormat::kot(), &slip(KotFieldStyle::light(24)));
        assert!(
            raster_dots_high(&large) > raster_dots_high(&small),
            "40pt must print taller than 16pt ({} vs {})",
            raster_dots_high(&large),
            raster_dots_high(&small)
        );
        assert_ne!(
            mid_bold, mid_light,
            "Bold and Light at the same size must not be the same print job"
        );
    }

    #[test]
    fn wider_paper_is_more_dots_and_keeps_the_cancel_banner_on_one_line() {
        assert_eq!(paper_width_dots(58), 384);
        assert_eq!(paper_width_dots(80), 576);
        assert!(paper_width_dots(112) > paper_width_dots(80));
        assert!(paper_width_dots(210) > paper_width_dots(112));
        assert_eq!(
            paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(58)),
            384
        );
        assert_eq!(
            paper_dots_for_wrap_cols(wrap_cols_for_paper_mm(80)),
            576
        );
        let mut fmt = KotSlipFormat::default();
        fmt.chars_per_line = wrap_cols_for_paper_mm(80);
        let text = strip_kot_style_marks(&fmt.render(KotSlipContent {
            kind: KotSlipKind::Cancelled,
            cancel: true,
            reprint: false,
            shop_name: "Shop",
            kot_no: 1,
            kot_nos_label: None,
            when: "now",
            created_by: "A",
            table_label: "D1",
            station_name: "Kitchen",
            area_label: "Dine In",
            items: &["Tea|1"],
        }));
        assert!(
            text.lines().any(|l| l.trim() == KOT_BANNER_CANCELLED),
            "banner must stay one line on 80 mm:\n{text}"
        );
        // The ticket is drawn, not counted in characters. At the default
        // bold title size a long cancel banner may clip on 58 mm and fit 80 mm.
        let banner = KOT_BANNER_CANCELLED;
        let draw = |dots: usize| {
            let page = crate::receipt_raster::RasterMetrics {
                em: KOT_EM_NORMAL as f32,
                line: 36,
                baseline: 22.0,
                side: 4.0,
                width_dots: dots,
            };
            let lines = [crate::receipt_raster::StyledLine {
                text: banner.into(),
                em: KOT_EM_NORMAL as f32,
                weight: crate::receipt_raster::RasterWeight::Bold,
                align: crate::receipt_raster::LineAlign::Start,
                strike: false,
                tab: None,
            }];
            crate::receipt_raster::draw_lines(&lines, &page).1
        };
        let pitch = ((KOT_EM_NORMAL as f32 * 1.45).round() as usize).clamp(16, 96);
        let h58 = draw(384);
        let h80 = draw(576);
        // Long cancelled banner may wrap on a narrow head; 80 mm must not be taller.
        assert!(h58 >= pitch, "58 mm draws the cancel banner:\n{banner}");
        assert!(
            h80 <= h58,
            "80 mm must not need more rows than 58 mm ({h80} vs {h58})"
        );
        assert_eq!(h80, pitch, "80 mm keeps the cancel banner on one line");
    }

    #[test]
    fn sample_slip_qty_header_shares_the_tab_with_counts() {
        let raw = KotSlipFormat::default().render_sample_kind("Shop", KotSlipKind::Original);
        let text = strip_kot_style_marks(&raw);
        let header = text
            .lines()
            .find(|l| l.split_once('\t').is_some_and(|(l, _)| l.trim() == "Item"))
            .unwrap_or_else(|| panic!("Item header must be a tab column:\n{text}"));
        let dish = text
            .lines()
            .find(|l| l.starts_with("Butter Roti\t"))
            .unwrap_or_else(|| panic!("dish count must be a tab column:\n{text}"));
        assert_eq!(header.split_once('\t').unwrap().1.trim(), "Qty.");
        assert_eq!(kot_qty_column(dish.split_once('\t').unwrap().1), "3");
        assert!(!text.contains("Table: D1\t"), "table code stays on its own line");
    }

    #[test]
    fn expand_qty_column_right_aligns_digits_under_header() {
        let line = expand_kot_column_tabs("Item\tQty.\nTea\t3\nLonger dish name\t12", 24);
        let lines: Vec<&str> = line.lines().collect();
        assert_eq!(lines[0].chars().count(), 24);
        assert_eq!(lines[1].chars().count(), 24);
        assert_eq!(lines[2].chars().count(), 24);
        assert!(lines[0].ends_with("Qty."));
        assert!(lines[1].ends_with("   3"));
        assert!(lines[2].ends_with("  12"));
    }

    #[test]
    fn partial_json_does_not_revive_legacy_verbose_defaults() {
        // Old serde defaults used to fill missing keys as show_title/waiter=true.
        let fmt = KotSlipFormat::parse(r#"{"chars_per_line":32}"#);
        assert!(!fmt.show_title);
        assert!(!fmt.show_shop_name);
        assert!(!fmt.show_waiter);
        assert!(!fmt.show_station);
        assert!(fmt.show_area);
        assert!(fmt.show_item_qty_header);
        assert!(!fmt.looks_like_legacy_verbose());
    }

    #[test]
    fn wrap_hard_breaks_overlong_tokens() {
        let fmt = KotSlipFormat {
            chars_per_line: 24,
            ..KotSlipFormat::default()
        };
        let rows = fmt.wrap_line(&"x".repeat(30));
        assert!(rows.iter().all(|r| r.chars().count() <= 24));
        assert!(rows.len() >= 2);
    }

    #[test]
    fn preview_rule_tracks_paper_width() {
        let narrow = KotSlipFormat {
            content_width_mm: 58,
            chars_per_line: KOT_CHARS_58MM,
            ..KotSlipFormat::default()
        }
        .render_sample("Shop");
        let wide = KotSlipFormat {
            content_width_mm: 80,
            chars_per_line: wrap_cols_for_paper_mm(80),
            ..KotSlipFormat::default()
        }
        .render_sample("Shop");
        let rule_n = "-".repeat(wrap_cols_for_paper_mm(58) as usize);
        let rule_w = "-".repeat(wrap_cols_for_paper_mm(80) as usize);
        assert!(narrow.contains(&rule_n));
        assert!(!narrow.contains(&rule_w));
        assert!(wide.contains(&rule_w));
        assert!(narrow.contains("Butter Roti"));
        assert!(wide.contains("Butter Roti"));
        assert!(rule_w.len() > rule_n.len());
    }

    #[test]
    fn preview_font_scales_with_em() {
        let s = KotSlipFormat {
            em: KOT_EM_SMALL,
            ..KotSlipFormat::default()
        };
        let m = KotSlipFormat {
            em: KOT_EM_NORMAL,
            ..KotSlipFormat::default()
        };
        let l = KotSlipFormat {
            em: KOT_EM_LARGE,
            ..KotSlipFormat::default()
        };
        assert!(s.preview_font_pt() < m.preview_font_pt());
        assert!(m.preview_font_pt() < l.preview_font_pt());
        assert!(s.preview_paper_width_px() < l.preview_paper_width_px());
    }

    #[test]
    fn optional_table_kot_no_title_and_copies() {
        let hidden = KotSlipFormat {
            show_title: true,
            show_table: false,
            show_kot_no: false,
            title: "KITCHEN FIRE".into(),
            copies: 9,
            ..KotSlipFormat::default()
        }
        .clamped();
        assert_eq!(hidden.copies, KOT_COPIES_MAX);
        let text = hidden.render_sample("Shop");
        assert!(text.contains("KITCHEN FIRE"));
        assert!(!text.contains("KOT No:"));
        assert!(!text.contains("Table:"));
        assert!(text.contains("KITCHEN FIRE"));
        let cancelled = hidden.render_sample_kind("Shop", KotSlipKind::Cancelled);
        assert!(
            cancelled.contains("KITCHEN FIRE"),
            "custom title must show on a cancelled slip:\n{cancelled}"
        );
        assert!(
            cancelled.contains(KOT_BANNER_CANCELLED),
            "cancelled kind banner:\n{cancelled}"
        );
        assert!(
            cancelled.contains(KOT_PREFIX_CANCELLED.trim_end())
                || cancelled.contains("[CANCELLED]"),
            "cancelled item prefix:\n{cancelled}"
        );
        let reprint = hidden.render(KotSlipContent {
            kind: KotSlipKind::Original,
            cancel: false,
            reprint: true,
            shop_name: "Shop",
            kot_no: 3,
            kot_nos_label: None,
            when: "now",
            created_by: "A",
            table_label: "T9",
            station_name: "Grill",
            area_label: "",
            items: &["Tea|1"],
        });
        assert!(reprint.contains("*** REPRINT - SAME ORDER ***"));
        // Hidden flags still suppress even on reprint.
        assert!(!reprint.contains("Was KOT No"));
        assert!(!reprint.contains("Table:"));
    }
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

/// Split a trailing kitchen note baked as `Name (note)` — used so strike can
/// hit the name without painting through the note (and the reverse).
pub fn kot_split_trailing_note(name: &str) -> (String, String) {
    let name = name.trim();
    if !name.ends_with(')') {
        return (name.to_string(), String::new());
    }
    let Some(open) = name.rfind(" (") else {
        return (name.to_string(), String::new());
    };
    let note = name[open + 2..name.len() - 1].trim();
    if note.is_empty() {
        return (name.to_string(), String::new());
    }
    (name[..open].trim_end().to_string(), note.to_string())
}

/// Dish name and qty column. Strips a count glued on either side of the name
/// (`x1× Dish`, `Dish x1`, `Dish|x1`) so the name prints alone.
pub fn kot_item_parts(item: &str) -> (String, String) {
    let item = item.trim();
    let (raw_name, raw_qty) = item.rsplit_once('|').unwrap_or((item, ""));
    let raw_name = raw_name.trim();
    let (name, peeled) = peel_leading_kot_qty(raw_name)
        .or_else(|| peel_trailing_x_qty(raw_name))
        .unwrap_or_else(|| (raw_name.to_string(), String::new()));
    let qty = if raw_qty.trim().is_empty() {
        peeled
    } else {
        kot_qty_column(raw_qty)
    };
    (name, qty)
}

/// `x1× Name`, `1× Name`, `1.25 kg× Rice` → (name, qty).
fn peel_leading_kot_qty(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    let mut rest = s;
    for mark in ['x', 'X', '×', '✕'] {
        if let Some(stripped) = rest.strip_prefix(mark) {
            if stripped.starts_with(|c: char| c.is_ascii_digit()) {
                rest = stripped;
            }
            break;
        }
    }
    if !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
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
        return None;
    }
    let number = rest[..end].trim_end_matches(['.', ',']);
    let after = rest[end..].trim_start();
    let (unit, name) = if let Some(name) = after
        .strip_prefix('×')
        .or_else(|| after.strip_prefix('✕'))
        .or_else(|| after.strip_prefix('x'))
        .or_else(|| after.strip_prefix('X'))
    {
        (String::new(), name.trim().to_string())
    } else if after.is_empty() {
        return None;
    } else if !after.contains(['×', '✕'])
        && after
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || !c.is_ascii())
    {
        // "x2 Butter Naan" / "2 Paneer" — digit qty then dish name, no × mark.
        (String::new(), after.to_string())
    } else {
        let (token, tail) = after.split_once(char::is_whitespace)?;
        let tail = tail.trim_start();
        if let Some(idx) = token.find(['×', '✕', 'x', 'X']) {
            if idx == 0 {
                return None;
            }
            let unit = token[..idx].to_string();
            let glued = &token[idx + token[idx..].chars().next()?.len_utf8()..];
            let name = if glued.is_empty() {
                tail.to_string()
            } else if tail.is_empty() {
                glued.to_string()
            } else {
                format!("{glued} {tail}")
            };
            (unit, name)
        } else if let Some(name) = tail
            .strip_prefix('×')
            .or_else(|| tail.strip_prefix('✕'))
            .or_else(|| tail.strip_prefix('x'))
            .or_else(|| tail.strip_prefix('X'))
        {
            (token.to_string(), name.trim().to_string())
        } else {
            return None;
        }
    };
    if name.is_empty() || number.is_empty() {
        return None;
    }
    let qty = if unit.is_empty() {
        number.to_string()
    } else {
        format!("{number} {unit}")
    };
    Some((name, qty))
}

/// `Dish x1` / `Dish × 1` → (dish, count). A bare trailing number stays on the name.
fn peel_trailing_x_qty(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    if chars.is_empty() {
        return None;
    }
    let mut i = chars.len();
    while i > 0 && chars[i - 1].1.is_ascii_digit() {
        i -= 1;
    }
    if i == chars.len() || i == 0 {
        return None;
    }
    let number = s[chars[i].0..].to_string();
    let mut j = i;
    while j > 0 && chars[j - 1].1.is_whitespace() {
        j -= 1;
    }
    if j == 0 || !matches!(chars[j - 1].1, 'x' | 'X' | '×' | '✕') {
        return None;
    }
    let name = s[..chars[j - 1].0].trim();
    if name.is_empty() {
        return None;
    }
    Some((name.to_string(), number))
}

/// Monospace qty field width — fits the `Qty.` header.
pub const KOT_QTY_COLS: usize = 4;

/// Expand `name\tqty` lines to a fixed right qty column for ESC/POS text
/// printers and on-screen preview (monospace cells). Raster keeps the tab
/// and aligns in dots instead.
pub fn expand_kot_column_tabs(text: &str, cols: usize) -> String {
    let cols = cols.max(8);
    let (tab, body) = take_slip_tab_layout(text);
    let tab = tab.unwrap_or(crate::receipt_raster::TabColumnLayout::LEGACY);
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

fn wrap_text_cols(line: &str, cols: usize) -> Vec<String> {
    let cols = cols.max(8);
    if line.chars().count() <= cols {
        return vec![line.to_string()];
    }
    let mut rows = Vec::new();
    let mut cur = String::new();
    let mut width = 0usize;
    let flush = |rows: &mut Vec<String>, cur: &mut String, width: &mut usize| {
        if !cur.is_empty() {
            rows.push(std::mem::take(cur));
            *width = 0;
        }
    };
    let push_chunked = |rows: &mut Vec<String>, word: &str| {
        let chars: Vec<char> = word.chars().collect();
        for chunk in chars.chunks(cols) {
            rows.push(chunk.iter().collect());
        }
    };
    for word in line.split(' ') {
        let wlen = word.chars().count();
        if wlen == 0 {
            continue;
        }
        if wlen > cols {
            flush(&mut rows, &mut cur, &mut width);
            push_chunked(&mut rows, word);
            continue;
        }
        if cur.is_empty() {
            cur.push_str(word);
            width = wlen;
        } else if width + 1 + wlen <= cols {
            cur.push(' ');
            cur.push_str(word);
            width += 1 + wlen;
        } else {
            flush(&mut rows, &mut cur, &mut width);
            cur.push_str(word);
            width = wlen;
        }
    }
    flush(&mut rows, &mut cur, &mut width);
    if rows.is_empty() {
        rows.push(String::new());
    }
    rows
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

/// One named printer in this device's pool (destination only).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedPrinter {
    pub id: String,
    pub label: String,
    /// Blank = system default; name = system printer; `ip` / `ip:port` = network;
    /// `bt:MAC`, `usb:VID:PID`, `android-print:`.
    pub target: String,
    /// Ignored on load (legacy); printers no longer store a mode.
    #[serde(default, alias = "wire", alias = "mode", skip_serializing)]
    pub _legacy_mode: Option<String>,
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
    // Legacy `{ "kot": "…", "bill": "…" }`.
    let kot = value
        .get("kot")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let bill = value
        .get("bill")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    migrate_legacy(&kot, &bill)
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

/// Turn old single-string fields into a pool + routes.
/// Blank fields stay empty routes (don't invent a printer named "Default").
pub fn migrate_legacy(kot: &str, bill: &str) -> DevicePrinters {
    let mut printers = Vec::new();
    let mut push = |target: &str, label: &str| -> Option<String> {
        let t = target.trim();
        if t.is_empty() {
            return None;
        }
        if let Some(existing) = printers.iter().find(|p: &&NamedPrinter| p.target == t) {
            return Some(existing.id.clone());
        }
        let id = uuid::Uuid::new_v4().to_string();
        printers.push(NamedPrinter {
            id: id.clone(),
            label: label.into(),
            target: t.to_string(),
            _legacy_mode: None,
        });
        Some(id)
    };
    let bill_ids = push(bill, "Bill printer").into_iter().collect();
    let kot_ids = if kot.trim().is_empty() {
        Vec::new()
    } else {
        let label = if bill.trim() == kot.trim() {
            "Bill printer"
        } else {
            "Kitchen printer"
        };
        push(kot, label).into_iter().collect()
    };
    DevicePrinters {
        printers,
        formats: vec![
            PrintFormat::standard_portrait(),
            PrintFormat::standard_landscape(),
            PrintFormat::kot(),
        ],
        routes: PrintRoutes {
            bill: bill_ids,
            kot: BTreeMap::from([("".into(), kot_ids)]),
            bill_format: FORMAT_STANDARD_PORTRAIT.into(),
            kot_format: BTreeMap::new(),
            bill_print_mode: RoutePrintMode::First,
            kot_print_mode: BTreeMap::new(),
        },
        virtual_printers: Vec::new(),
        station_groups: Vec::new(),
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
    pub fn print_mode_for(&self, key: RouteKey<'_>) -> RoutePrintMode {
        match key {
            RouteKey::Bill => self.routes.bill_print_mode,
            RouteKey::Kot { station_id } => self
                .routes
                .kot_print_mode
                .get(station_id)
                .copied()
                .or_else(|| self.routes.kot_print_mode.get("").copied())
                .unwrap_or(RoutePrintMode::First),
        }
    }

    pub fn set_print_mode(&mut self, key: RouteKey<'_>, mode: RoutePrintMode) {
        match key {
            RouteKey::Bill => self.routes.bill_print_mode = mode,
            RouteKey::Kot { station_id } => {
                self.routes
                    .kot_print_mode
                    .insert(station_id.to_string(), mode);
            }
        }
    }

    pub fn route_for(&self, key: RouteKey<'_>) -> &[String] {
        match key {
            RouteKey::Bill => &self.routes.bill,
            RouteKey::Kot { station_id } => {
                // Named kitchen: ONLY that place's Outputs. Empty / missing =
                // don't print — never silently use Bill / another kitchen's
                // printer (that ignored the owner's Select printer choice).
                if !station_id.is_empty() {
                    return self
                        .routes
                        .kot
                        .get(station_id)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                }
                // Default kitchen `""` (Order "Kitchen (no station)"): only its
                // own Outputs — never a named station's printers.
                if let Some(own) = self.routes.kot.get("") {
                    if !own.is_empty() {
                        return own.as_slice();
                    }
                }
                // Bill fallback only for single-kitchen shops (no named routes
                // yet). Once stations exist, unassigned with an empty `""`
                // list prints nowhere — do not wander onto the counter printer
                // and look like a KOT hit.
                let has_named = self.routes.kot.keys().any(|k| !k.is_empty());
                if has_named {
                    return &[];
                }
                self.routes.bill.as_slice()
            }
        }
    }

    /// Ensure every kitchen station has its own route entry (starts empty =
    /// don't print until the owner adds printers for that station).
    pub fn ensure_station_routes(&mut self, station_ids: impl IntoIterator<Item = impl AsRef<str>>) {
        if !self.routes.kot.contains_key("") {
            self.routes.kot.insert(String::new(), Vec::new());
        }
        for id in station_ids {
            self.routes
                .kot
                .entry(id.as_ref().to_string())
                .or_insert_with(Vec::new);
        }
    }

    /// Drop a station's KOT route when the station is removed (pool printers stay).
    pub fn remove_station_route(&mut self, station_id: &str) {
        let id = station_id.trim();
        if id.is_empty() {
            return;
        }
        self.routes.kot.remove(id);
        self.routes.kot_format.remove(id);
        self.routes.kot_print_mode.remove(id);
    }

    /// Resolve route ids to print targets (skips unknown ids).
    pub fn targets_for(&self, key: RouteKey<'_>) -> Vec<PrinterTarget> {
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
        }
    }

    /// Format id for this route (Bill → bill_format; kitchen → kot_format / KOT).
    pub fn format_id_for(&self, key: RouteKey<'_>) -> String {
        match key {
            RouteKey::Bill => {
                let f = self.routes.bill_format.trim();
                if f.is_empty() {
                    FORMAT_STANDARD_PORTRAIT.into()
                } else {
                    normalize_format_id(f).to_string()
                }
            }
            RouteKey::Kot { station_id } => {
                if let Some(f) = self.routes.kot_format.get(station_id) {
                    let f = f.trim();
                    if !f.is_empty() {
                        return f.to_string();
                    }
                }
                if !station_id.is_empty() {
                    if let Some(f) = self.routes.kot_format.get("") {
                        let f = f.trim();
                        if !f.is_empty() {
                            return f.to_string();
                        }
                    }
                }
                FORMAT_KOT.into()
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

    pub fn format_for(&self, key: RouteKey<'_>) -> PrintFormat {
        self.format_by_id(&self.format_id_for(key))
    }

    pub fn set_route_format(&mut self, key: RouteKey<'_>, format_id: &str) {
        let id = format_id.trim();
        if id.is_empty() {
            return;
        }
        match key {
            RouteKey::Bill => self.routes.bill_format = id.to_string(),
            RouteKey::Kot { station_id } => {
                self.routes.kot_format.insert(station_id.to_string(), id.to_string());
            }
        }
    }

    /// Set the place Profile and rewrite every Output on that place to it.
    pub fn set_place_profile(&mut self, key: RouteKey<'_>, profile_id: &str) {
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
            RouteKey::Kot { station_id } => {
                if let Some(slots) = self.routes.kot.get_mut(station_id) {
                    rewrite(slots);
                }
            }
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

    /// Copy Outputs (+ profile + print mode) from `from` onto each kitchen
    /// station id (Bill is never a group target).
    pub fn apply_outputs_to_stations(&mut self, from: RouteKey<'_>, station_ids: &[String]) {
        let slots: Vec<String> = self.route_for(from).to_vec();
        let profile = self.format_id_for(from);
        let mode = self.print_mode_for(from);
        for sid in station_ids {
            let sid = sid.trim();
            if sid.is_empty() {
                continue;
            }
            self.routes.kot.insert(sid.to_string(), slots.clone());
            self.routes.kot_format.insert(sid.to_string(), profile.clone());
            self.routes.kot_print_mode.insert(sid.to_string(), mode);
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

    /// Toggle a real printer on a bill or kitchen route. Returns whether it is attached after.
    pub fn toggle_printer_on_route(&mut self, key: RouteKey<'_>, printer_id: &str) -> Option<bool> {
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
    pub fn toggle_virtual_on_route(&mut self, key: RouteKey<'_>, vp_id: &str) -> Option<bool> {
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
    pub fn swap_route_slots(&mut self, key: RouteKey<'_>, from: usize, to: usize) -> bool {
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
    pub fn destinations_for(&self, key: RouteKey<'_>) -> Vec<PrintDest> {
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
                })
            })
            .collect()
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

    /// Wrap columns on the built-in kitchen format only.
    pub fn set_kot_content_line(&mut self, cols: u32) {
        let cols = cols.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX);
        for f in self.formats.iter_mut() {
            if f.id == FORMAT_KOT {
                f.line = cols;
            }
        }
    }

    /// Same wrap on bill and KOT (tests / forced parity). Prefer the
    /// bill-only / KOT-only setters in product paths.
    pub fn set_content_line(&mut self, cols: u32) {
        self.set_bill_content_line(cols);
        self.set_kot_content_line(cols);
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
        let stations: Vec<String> = self.routes.kot.keys().cloned().collect();
        for sid in stations {
            let default = self.format_id_for(RouteKey::Kot {
                station_id: sid.as_str(),
            });
            if let Some(slots) = self.routes.kot.get_mut(&sid) {
                Self::dedupe_slots_by_name_format(slots, &self.printers, &vp_ids, &default);
            }
        }
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

    pub fn route_mut(&mut self, key: RouteKey<'_>) -> &mut Vec<String> {
        match key {
            RouteKey::Bill => &mut self.routes.bill,
            RouteKey::Kot { station_id } => self
                .routes
                .kot
                .entry(station_id.to_string())
                .or_default(),
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

fn default_route_for(kind: PrintKind) -> RouteKey<'static> {
    match kind {
        PrintKind::Bill => RouteKey::Bill,
        PrintKind::Kot => RouteKey::Kot { station_id: "" },
    }
}

/// Whether `kind` prints on paper from this device as things are set now.
pub fn prints_here(kind: PrintKind) -> bool {
    prints_route(default_route_for(kind))
}

/// Whether this route has at least one printable target.
pub fn prints_route(key: RouteKey<'_>) -> bool {
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
    /// [`with_test_printers`] config for this thread only. The global
    /// `PRINTERS` is shared by every test thread, and plenty of code
    /// (`use_device_dir`, app-state reloads) rewrites it without the test
    /// lock — a parallel test opening books blanked the routes mid-run
    /// (`batch_cancel_prints_one_slip_per_kitchen` captured nothing).
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

/// If this shop has no usable printers yet but the install still has the old
/// shared file, copy it so an upgrade does not wipe routes. An empty
/// `printers.json` created by an earlier load/save still counts as "missing"
/// when the legacy file actually has a pool — otherwise every shop looked
/// printer-less after the per-books split. Later shops that already have
/// their own pool are left alone (no cross-server bleed).
pub fn migrate_legacy_printers(install_base: &Path, books_dir: &Path) {
    let dest = printers_path(books_dir);
    let legacy = printers_path(install_base);
    if !legacy.is_file() || legacy == dest {
        return;
    }
    let legacy_cfg = load_device_printers(install_base);
    if legacy_cfg.printers.is_empty() {
        return;
    }
    let shop_cfg = if dest.is_file() {
        load_device_printers(books_dir)
    } else {
        DevicePrinters::default()
    };
    if !shop_cfg.printers.is_empty() {
        return;
    }
    let _ = fs::create_dir_all(books_dir);
    let _ = fs::copy(&legacy, &dest);
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

/// Open the printers that belong to `db_path`'s shop. Migrates the legacy
/// install-level file into this shop once when missing.
pub fn use_books_printers(db_path: &Path, install_base: Option<&Path>) {
    let books_dir = printers_dir_for_books(db_path);
    if let Some(install) = install_base {
        migrate_legacy_printers(install, &books_dir);
    }
    use_device_dir(&books_dir);
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
    route: RouteKey<'_>,
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
    route: RouteKey<'_>,
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
    route: RouteKey<'_>,
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
                if kind == PrintKind::Kot {
                    note_problem(
                        NO_KOT_PRINTER.into(),
                    );
                }
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
                if kind == PrintKind::Kot {
                    note_problem(
                        NO_KOT_PRINTER.into(),
                    );
                }
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

/// A kitchen station with no printer attached: where to fix it (the Kitchen
/// screen's station list, not Settings → Printers, which only finds devices).
pub const NO_KOT_PRINTER: &str =
    "No printer for that kitchen station — Kitchen → Kitchen stations → Printers for this station";

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
                                match send_to_printer(
                                    &dest.format,
                                    &dest.target,
                                    &job.path,
                                    &job.text,
                                    copies,
                                ) {
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
                                if let Err(e) = send_to_printer(
                                    &dest.format,
                                    &dest.target,
                                    &job.path,
                                    &job.text,
                                    copies,
                                ) {
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

fn send_to_printer(
    format: &PrintFormat,
    target: &PrinterTarget,
    path: &Path,
    text: &str,
    copies: u32,
) -> Result<(), String> {
    let copies = copies.max(1);
    // Delivery is the format's own flag. Bill text and kitchen text use the
    // same sender; neither one is tied to a page printer or a thermal head.
    let as_page = format.document;
    let thermal = || replicate_escpos_copies(&escpos_job_format(format, text), copies);
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
                print_android_document(text, format, format.landscape)?;
            }
            Ok(())
        }
        PrinterTarget::Default => {
            if as_page {
                print_system_standard(None, path, text, format, format.landscape, copies)
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
                print_system_standard(Some(name), path, text, format, format.landscape, copies)
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

fn print_android_document(text: &str, format: &PrintFormat, landscape: bool) -> Result<(), String> {
    // Same Noto PNG as desktop. Kitchen marks keep size, weight, and alignment
    // across the content width. A bill uses the format's type size and width.
    let metrics = format.raster_metrics();
    let png = if text.contains(KOT_STYLE_MARK) {
        crate::receipt_raster::png_lines(&styled_slip_lines(text, &metrics), &metrics)?
    } else {
        crate::receipt_raster::png_metrics(text, &metrics)?
    };
    #[cfg(target_os = "android")]
    {
        crate::print_android::print_png("Khaynjo", &png, landscape)
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (png, landscape);
        Err("Android print service is only on Android".into())
    }
}

/// Standard (document) jobs: always render with bundled Noto fonts to a PNG,
/// then hand that image to the OS printer. Plain UTF-8 text depended on each
/// driver's fonts (Gujarati/Hindi became □ before qty on Brother etc.).
fn print_system_standard(
    name: Option<&str>,
    path: &Path,
    text: &str,
    format: &PrintFormat,
    landscape: bool,
    copies: u32,
) -> Result<(), String> {
    let metrics = format.raster_metrics();
    // A kitchen ticket carries a size, a weight, and Start/Center/End on each
    // line. Drawing the stripped text left-aligned threw those away, so the
    // chips did nothing on an OS-document printer. Bills have no marks and
    // still use this format's type size and content width.
    let png = if text.contains(KOT_STYLE_MARK) {
        crate::receipt_raster::png_lines(&styled_slip_lines(text, &metrics), &metrics)?
    } else {
        crate::receipt_raster::png_metrics(text, &metrics)?
    };
    let mm = content_mm_for_dots(metrics.page_dots());
    let px = png_pixel_size(&png).unwrap_or((1, 1)).0.max(1);
    let ppm = ((px as f64) * 1000.0 / mm.max(1) as f64).round() as u32;
    let png = png_with_density(png, ppm);
    let png_path = path.with_extension("png");
    fs::write(&png_path, &png).map_err(|e| e.to_string())?;
    #[cfg(target_os = "android")]
    {
        let _ = name;
        for _ in 0..copies.max(1) {
            crate::print_android::print_png("Khaynjo", &png, landscape)?;
        }
        Ok(())
    }
    #[cfg(not(target_os = "android"))]
    {
        let mm = content_mm_for_dots(format.raster_metrics().page_dots());
        print_system_image(name, &png_path, landscape, copies.max(1), mm)
    }
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

fn print_network_bytes(addr: &str, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > ESC_POS_JOB_MAX {
        return Err(format!(
            "print job too large ({} bytes) — refused so the kitchen printer is not flooded",
            bytes.len()
        ));
    }
    let sock: std::net::SocketAddr = addr.parse().map_err(|_| format!("{addr} is not a printer address"))?;
    let mut stream = connect_printer(&sock)?;
    // A head busy with a long ticket reads slowly; TCP holds our writes until
    // it does. That is waiting, not a failure.
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    // One raster picture at a time. A single burst prints the first band
    // (about three lines) and the head drops the rest of the ticket.
    crate::receipt_raster::write_escpos_paced(bytes, |piece| {
        stream.write_all(piece).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())
    })?;
    finish_printer_stream(addr, stream, bytes.len())
}

/// Open the printer's raw port. A printer still printing the previous ticket
/// often refuses (or resets) a second connection until it is done: keep
/// trying for a while instead of failing a ticket that would print a moment
/// later. A printer that never answers fails after two tries.
fn connect_printer(sock: &std::net::SocketAddr) -> Result<std::net::TcpStream, String> {
    use std::io::ErrorKind;
    let busy_until = std::time::Instant::now() + Duration::from_secs(6);
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

/// ESC/POS real-time status (DLE EOT 1) after the last byte. The printer
/// answers only once it has read everything before it, so the answer means
/// the ticket reached the head; an "offline" answer (cover open, out of
/// paper, error) is a failure the counter must hear about. A printer that
/// does not answer is remembered, and later jobs only half-close and give it
/// a moment to read, so the next ticket never lands on top of this one.
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
        // Long jobs take longer to read: wait for the answer accordingly.
        let wait = Duration::from_millis((1500 + sent as u64 / 4).min(12_000));
        let _ = stream.set_read_timeout(Some(wait));
        if stream.write_all(&[0x10, 0x04, 0x01]).and_then(|_| stream.flush()).is_ok() {
            let mut reply = [0u8; 1];
            match stream.read(&mut reply) {
                Ok(1) => {
                    ANSWERS
                        .lock()
                        .unwrap()
                        .get_or_insert_with(Default::default)
                        .insert(addr.to_string(), true);
                    if let Some(problem) = escpos_status_problem(reply[0]) {
                        return Err(problem.into());
                    }
                    return Ok(());
                }
                _ => {
                    ANSWERS
                        .lock()
                        .unwrap()
                        .get_or_insert_with(Default::default)
                        .insert(addr.to_string(), false);
                }
            }
        }
    }
    // No status: stop sending, let the printer finish reading, then close.
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
/// (bit 1 and 4 set, bit 0 and 7 clear) must match, so another device's
/// stray byte is never read as a fault.
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
    let cleaned = text.replace('\r', "");
    let plain = strip_kot_style_marks(&cleaned);
    let metrics = format.raster_metrics();
    // A slip carries a size and a weight on every line. Draw those with Noto
    // so 16 and 32, and Light and Bold, are different ink. Unmarked text
    // stays on the built-in font when the head has the letters. The same
    // encoder is used for a bill and for a kitchen ticket.
    if cleaned.contains(KOT_STYLE_MARK) {
        return escpos_picture_styled(&cleaned, &metrics);
    }
    if printer_has_letters(&plain) {
        if let Some((gs, col_div)) = escpos_gs_bang_for_em(format.em) {
            let cols = (format.line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX) as usize / col_div).max(8);
            let expanded = expand_kot_column_tabs(&cleaned, cols);
            escpos_job_text_cols_sized(&expanded, cols, gs)
        } else {
            crate::receipt_raster::escpos_picture_metrics(&plain, &metrics)
        }
    } else {
        crate::receipt_raster::escpos_picture_metrics(&plain, &metrics)
    }
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

fn png_pixel_size(png: &[u8]) -> Option<(u32, u32)> {
    if png.len() < 24 {
        return None;
    }
    Some((
        u32::from_be_bytes([png[16], png[17], png[18], png[19]]),
        u32::from_be_bytes([png[20], png[21], png[22], png[23]]),
    ))
}

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
    ((em * 1.45).round() as usize).clamp(16, 96)
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
            let (em, weight, align, strike) = match style {
                Some(style) => (
                    style.size_em as f32,
                    match style.weight {
                        KotFontWeight::Light => crate::receipt_raster::RasterWeight::Light,
                        KotFontWeight::Bold => crate::receipt_raster::RasterWeight::Bold,
                        KotFontWeight::Regular => crate::receipt_raster::RasterWeight::Regular,
                    },
                    match style.align {
                        KotAlign::Center => crate::receipt_raster::LineAlign::Center,
                        KotAlign::End => crate::receipt_raster::LineAlign::End,
                        KotAlign::Start => crate::receipt_raster::LineAlign::Start,
                    },
                    style.strike,
                ),
                None => (
                    default.em,
                    crate::receipt_raster::RasterWeight::Regular,
                    crate::receipt_raster::LineAlign::Start,
                    false,
                ),
            };
            crate::receipt_raster::StyledLine {
                text: body.to_string(),
                em,
                weight,
                align,
                strike,
                tab: if body.contains('\t') { tab } else { None },
            }
        })
        .collect()
}

/// Raster job that keeps each line's point size, weight, and alignment.
fn escpos_picture_styled(text: &str, default: &crate::receipt_raster::RasterMetrics) -> Vec<u8> {
    crate::receipt_raster::escpos_picture_lines(&styled_slip_lines(text, default), default)
}

/// Map format `em` to ESC/POS `GS !` n and a wrap-column divisor.
/// Normal → 1×1; Large → 2×2. Small / custom return `None` so the caller
/// rasterises (built-in Font A cannot go below 1×).
fn escpos_gs_bang_for_em(em: f32) -> Option<(u8, usize)> {
    // Shared buckets: Standard 18/22/26 · KOT 20/24/28.
    if (em - STD_EM_NORMAL).abs() < 1.5 || (em - KOT_EM_NORMAL as f32).abs() < 1.5 {
        Some((0x00, 1)) // 1×1
    } else if (em - STD_EM_LARGE).abs() < 1.5 || (em - KOT_EM_LARGE as f32).abs() < 1.5 {
        Some((0x11, 2)) // 2× width and height — wrap at half columns
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
/// and 80 mm Font A) so heads do not mid-word wrap into incomplete lines.
pub fn escpos_job_text(text: &str) -> Vec<u8> {
    escpos_job_text_cols(text, KOT_CHARS_58MM as usize)
}

/// Like [`escpos_job_text`], with an explicit wrap width (clamped).
pub fn escpos_job_text_cols(text: &str, cols: usize) -> Vec<u8> {
    escpos_job_text_cols_sized(text, cols, 0x00)
}

/// ASCII ESC/POS job with explicit `GS !` character size (`n`: high nibble =
/// width−1, low nibble = height−1; `0x00` = 1×1, `0x11` = 2×2).
pub fn escpos_job_text_cols_sized(text: &str, cols: usize, gs_bang: u8) -> Vec<u8> {
    let cols = (cols as u32).clamp(KOT_CHARS_MIN, KOT_CHARS_MAX) as usize;
    let plain = strip_kot_style_marks(text);
    let wrapped = wrap_escpos_cols(&plain, cols);
    let mut out = Vec::with_capacity(wrapped.len().saturating_add(48));
    out.extend_from_slice(&[0x1B, 0x40]); // ESC @: reset
    out.extend_from_slice(&[0x1B, 0x74, 0x00]); // ESC t 0: PC437
    out.extend_from_slice(&[0x1B, 0x4D, 0x00]); // ESC M 0: Font A
    out.extend_from_slice(&[0x1D, 0x21, gs_bang]); // GS ! n: character size
    out.extend_from_slice(&[0x1B, 0x61, 0x00]); // ESC a 0: left align
    out.extend_from_slice(&[0x1B, 0x32]); // ESC 2: default line spacing
    out.extend(escpos_text(&wrapped));
    out.extend_from_slice(b"\n\n\n\n\n"); // LF only — never CR
    out.extend_from_slice(&[0x1D, 0x56, 0x00]); // GS V 0: full cut
    out
}

/// Soft-wrap each input line to `cols` (break on spaces when possible).
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

/// Text a receipt printer's built-in font can show: printable ASCII only
/// (space–`~` and LF), symbols spelt out, anything else as "?".
/// **Never emit CR** — on many ESC/POS heads CR reprints on the same row
/// (overstrike); stacked lines collapse to the last one with `_`-like smear.
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
fn print_system_image(
    name: Option<&str>,
    path: &Path,
    landscape: bool,
    copies: u32,
    content_mm: u32,
) -> Result<(), String> {
    let copies = copies.max(1);
    let content_mm = content_mm.clamp(40, 210);
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let mut cmd = Command::new("lp");
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
        if copies > 1 {
            cmd.args(["-n", &copies.to_string()]);
        }
        // A slip stays its own width. A page (about A4) is fitted onto the
        // sheet so the whole bill is on the paper, not a clipped corner of
        // two rules. The PNG carries the same density in pHYs.
        let px = png_pixel_size(&std::fs::read(path).unwrap_or_default()).unwrap_or((1, 1)).0;
        let ppi = ((px as f64) * 25.4 / content_mm as f64)
            .round()
            .clamp(72.0, 1200.0) as u32;
        let scaling = if content_mm >= 180 {
            "print-scaling=fit"
        } else {
            "print-scaling=none"
        };
        cmd.args(["-o", scaling]);
        cmd.args(["-o", &format!("ppi={ppi}")]);
        // 3 = portrait, 4 = landscape (IPP orientation-requested).
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
        // Draw the PNG with System.Drawing — Out-Printer on UTF-8 text had no
        // Indic fonts; the image already has our Noto glyphs.
        let mut script = format!(
            "Add-Type -AssemblyName System.Drawing; \
             $img = [System.Drawing.Image]::FromFile('{path_q}'); \
             $doc = New-Object System.Drawing.Printing.PrintDocument; \
             $doc.DocumentName = 'Khaynjo'; \
             $doc.DefaultPageSettings.Landscape = {land}; \
             $doc.PrinterSettings.Copies = [int16]{copies}; \
             $doc.DefaultPageSettings.Margins = New-Object System.Drawing.Printing.Margins(20,20,20,20);"
        );
        if let Some(name) = name {
            script.push_str(&format!(
                "$doc.PrinterSettings.PrinterName = '{}';",
                quote(name)
            ));
        }
        script.push_str(&format!(
            "$script:img = $img; \
             $doc.add_PrintPage({{ param($s,$e) \
               $b = $e.MarginBounds; \
               $want = [int]({content_mm} / 25.4 * 100); \
               $w = [Math]::Min($want, $b.Width); \
               $h = [int]($w * $script:img.Height / $script:img.Width); \
               if ($h -gt $b.Height) {{ $h = $b.Height; $w = [int]($h * $script:img.Width / $script:img.Height) }}; \
               $e.Graphics.DrawImage($script:img, $b.Left, $b.Top, $w, $h); \
               $e.HasMorePages = $false }}); \
             $doc.Print(); \
             $img.Dispose(); \
             $doc.Dispose();"
        ));
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
        let _ = (name, path);
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

/// CUPS often accepts a job (`lp` exit 0) while the backend cannot reach the
/// device — classic message: "Waiting for printer to become available".
/// That must not look like a successful print in the app.
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
    // `lp` returns when CUPS has the job, before the backend has tried the
    // device. Watch the job briefly (this runs on the print worker, never the
    // UI): done → printed; the queue says the printer cannot be reached →
    // cancel and fail. Copies are one job, so this runs once per ticket.
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
            // Done, or still printing a long page: the device has it.
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

/// Send ESC/POS bytes to a system printer as a **raw** job (KOT on desktop queues).
fn print_system_raw(name: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > ESC_POS_JOB_MAX {
        return Err(format!(
            "print job too large ({} bytes) — refused so the kitchen printer is not flooded",
            bytes.len()
        ));
    }
    // A CUPS queue that is a raw TCP socket: talk to that printer directly,
    // paced and confirmed like a network address. `lp` hands the whole job
    // to the socket backend in one burst, and reports success once queued.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    if let Some(addr) = name.and_then(cups_socket_address) {
        return print_network_bytes(&addr, bytes);
    }
    let dir = std::env::temp_dir().join("kumkaj-kot-raw");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let pieces = crate::receipt_raster::escpos_piece_lengths(bytes);
    let result = print_system_raw_path(name, &path, &pieces);
    let _ = fs::remove_file(&path);
    result
}

/// Windows script: write the bytes in `$pieces` order, one paced piece at a
/// time ([`crate::receipt_raster::PIECE_PAUSE_MS`] between them). Each piece
/// loops until WritePrinter took all of it.
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
        // Raw spool for other backends (USB, serial, IPP): bytes go to the
        // device as-is. USB and serial hold the host back while the head is
        // busy; socket queues never get here (see `print_system_raw`).
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
                let detail = first_line(&out.stderr).unwrap_or_else(|| "raw print refused".into());
                Err(format!(
                    "{detail} — for kitchen tickets prefer a network address (host:9100) in Settings → Printers"
                ))
            }
            Err(_) => Err(
                "no printing system (CUPS) — add the kitchen printer as host:9100 in Settings → Printers"
                    .into(),
            ),
        }
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let quote = |s: &str| s.replace('\'', "''");
        let path_s = quote(&path.display().to_string());
        // RAW datatype via winspool — page drivers must not reinterpret the slip.
        let script = if let Some(name) = name {
            format!(
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
if (-not [KumkajRawPrint]::OpenPrinter('{name}', [ref]$h, [IntPtr]::Zero)) {{ throw "OpenPrinter failed" }}
try {{
  $di = New-Object KumkajRawPrint+DOCINFOA
  $di.pDocName = 'KOT'
  $di.pDataType = 'RAW'
  if (-not [KumkajRawPrint]::StartDocPrinter($h, 1, $di)) {{ throw "StartDocPrinter failed" }}
  try {{
    if (-not [KumkajRawPrint]::StartPagePrinter($h)) {{ throw "StartPagePrinter failed" }}
    try {{
      $pinned = [Runtime.InteropServices.GCHandle]::Alloc($bytes, [Runtime.InteropServices.GCHandleType]::Pinned)
      try {{
        # One picture at a time with a pause (a head drops a burst), and each
        # piece looped until WritePrinter took every byte of it.
        {write_loop}
      }} finally {{ $pinned.Free() }}
    }} finally {{ [void][KumkajRawPrint]::EndPagePrinter($h) }}
  }} finally {{ [void][KumkajRawPrint]::EndDocPrinter($h) }}
}} finally {{ [void][KumkajRawPrint]::ClosePrinter($h) }}
"#,
                path_s = path_s,
                name = quote(name),
                write_loop = powershell_paced_write_loop("KumkajRawPrint", pieces),
            )
        } else {
            // No name → default printer via Out-Printer still page-layouts; refuse
            // with a clear fix (pick a named queue or host:9100).
            return Err(
                "pick the kitchen printer by name, or use a network address (host:9100) in Settings → Printers"
                    .into(),
            );
        };
        let out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("PowerShell did not start ({e})"))?;
        if out.status.success() {
            Ok(())
        } else {
            let detail = first_line(&out.stderr).unwrap_or_else(|| "Windows raw print failed".into());
            Err(format!(
                "{detail} — for kitchen tickets prefer a network address (host:9100) in Settings → Printers"
            ))
        }
    }
    #[cfg(target_os = "android")]
    {
        let _ = (name, path, pieces);
        Err(
            "Android has no system printer drivers for kitchen tickets — use a network address (host:9100) in Settings → Printers"
                .into(),
        )
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
    {
        let _ = (name, path, pieces);
        Err("printing is not supported on this system".into())
    }
}

/// `host:port` of a CUPS queue whose device is a raw socket
/// (`lpstat -v NAME` → `device for NAME: socket://192.168.1.50:9100`).
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_socket_address(name: &str) -> Option<String> {
    use std::net::ToSocketAddrs;
    let out = Command::new("lpstat").args(["-v", name]).output().ok()?;
    let host_port = parse_cups_socket_uri(&String::from_utf8_lossy(&out.stdout))?;
    let addr = host_port.to_socket_addrs().ok()?.next()?;
    Some(addr.to_string())
}

/// The `host:port` in an `lpstat -v` line for a `socket://` device (port
/// 9100 when the URI names none). Any other backend: `None`.
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn parse_cups_socket_uri(lpstat: &str) -> Option<String> {
    let rest = lpstat.split("socket://").nth(1)?;
    let hostport: &str = rest
        .split(|c: char| c.is_whitespace() || c == '/' || c == '?')
        .next()?;
    if hostport.is_empty() {
        return None;
    }
    // `[v6]:port`, `[v6]`, `host:port`, `host`.
    if hostport.starts_with('[') {
        return Some(if hostport.contains("]:") {
            hostport.to_string()
        } else {
            format!("{hostport}:{RAW_PRINT_PORT}")
        });
    }
    Some(if hostport.contains(':') {
        hostport.to_string()
    } else {
        format!("{hostport}:{RAW_PRINT_PORT}")
    })
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
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // Skip WorkOffline and Offline — those are removed / unplugged ghosts.
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
    // Thread-local: other test threads rewriting the global config can no
    // longer change what this test prints to.
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
    fn unit_tests_never_reach_the_os_printer() {
        let mut cfg = DevicePrinters::default();
        let kitchen = cfg.add_printer("Kitchen", "192.168.1.50");
        *cfg.route_mut(RouteKey::Kot { station_id: "" }) = vec![kitchen];
        with_test_printers(cfg, || {
            assert_eq!(sink(), PrintSink::Capture);
            let dir = std::env::temp_dir().join(format!("khaynjo-print-{}", std::process::id()));
            let _ = take_problems();
            let _ = take_captured();
            let share = print_text(PrintKind::Kot, "KOT — test", Some(&dir)).unwrap();
            assert!(share.opened, "captured as if a printer took it");
            let jobs = take_captured();
            assert_eq!(jobs.len(), 1);
            // Capture / Named keep the slip as written; Network ESC/POS
            // sanitizes inside escpos_job (see receipt_printer_text_is_plain).
            assert_eq!(jobs[0].text, "KOT — test");
            assert_eq!(jobs[0].kind, PrintKind::Kot);
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
            let dir = std::env::temp_dir()
                .join(format!("khaynjo-one-prn-{}", uuid::Uuid::new_v4().simple()));
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

    #[test]
    fn empty_kot_route_skips_capture() {
        with_test_printers(DevicePrinters::default(), || {
            // empty kot + empty bill = don't print
            let _ = take_problems();
            let _ = take_captured();
            let dir = std::env::temp_dir().join(format!("khaynjo-print-skip-{}", uuid::Uuid::new_v4().simple()));
            let share = print_route(
                PrintKind::Kot,
                RouteKey::Kot { station_id: "" },
                "KOT — skip",
                Some(&dir),
            )
            .unwrap();
            assert!(!share.opened);
            assert!(take_captured().is_empty());
            assert!(!take_problems().is_empty());
            let _ = fs::remove_dir_all(dir);
        });
    }

    #[test]
    fn named_kitchen_never_steals_bill_printer() {
        let mut cfg = DevicePrinters::default();
        let p = cfg.add_printer("Counter", "192.168.1.50");
        cfg.routes.bill = vec![p.clone()];
        // No named kitchens yet: default kitchen `""` with no Outputs still
        // uses Bill (single-printer counter shops).
        assert_eq!(
            cfg.route_for(RouteKey::Kot { station_id: "" }),
            &[p.clone()]
        );
        // Named kitchen with no route entry: don't print — never Bill.
        assert!(cfg
            .route_for(RouteKey::Kot {
                station_id: "station-a"
            })
            .is_empty());
        // Explicit empty station entry: still don't print.
        cfg.ensure_station_routes(["station-a"]);
        assert!(cfg
            .route_for(RouteKey::Kot {
                station_id: "station-a"
            })
            .is_empty());
        assert_eq!(cfg.route_for(RouteKey::Bill), &[p][..]);
        // Named kitchen with Outputs: unassigned must not fall back to Bill,
        // and must never use that named station's printer.
        let kitchen = cfg.add_printer("KOT", "192.168.1.60");
        *cfg.route_mut(RouteKey::Kot {
            station_id: "station-a",
        }) = vec![kitchen.clone()];
        assert_eq!(
            cfg.route_for(RouteKey::Kot {
                station_id: "station-a"
            }),
            &[kitchen.clone()][..]
        );
        assert!(
            cfg.route_for(RouteKey::Kot { station_id: "" }).is_empty(),
            "unassigned must not steal Bill or a named station's printer"
        );
        // Explicit default-kitchen Outputs still serve true unassigned slips.
        *cfg.route_mut(RouteKey::Kot { station_id: "" }) = vec![kitchen.clone()];
        assert_eq!(
            cfg.route_for(RouteKey::Kot { station_id: "" }),
            &[kitchen][..]
        );
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
        assert_eq!(escpos_text("Thali × 2 — ₹500\r\n"), b"Thali x 2 - Rs.500\n".to_vec());
        assert_eq!(escpos_text("ખીચડી"), b"?????".to_vec());
        assert!(printer_has_letters("Thali × 2 — ₹500"));
        assert!(!printer_has_letters("ખીચડી"));
        // Built-in-font path stays ASCII (even when forced); Indic slips use raster.
        let text_job = escpos_job_text("2 x ખીચડી");
        assert!(!text_job.windows(3).any(|w| w == [0x1D, 0x76, 0x30]));
        assert!(text_job.windows(5).any(|w| w == b"?????"));
        assert!(text_job.len() < 256, "text job must stay tiny, got {}", text_job.len());
        let ascii_job = escpos_job("KOT 7\n2 x Thali\n1 x Roti");
        assert_eq!(&ascii_job[..2], &[0x1B, 0x40]);
        assert!(
            !ascii_job.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
            "ASCII slips use plain ESC/POS text — not GS v 0 raster"
        );
        assert!(ascii_job.windows(5).any(|w| w == b"Thali"));
        assert!(ascii_job.ends_with(&[0x1D, 0x56, 0x00]));
        assert!(ascii_job.len() < ESC_POS_JOB_MAX);
        let indic = escpos_job("KOT\nખીચડી x 2");
        assert!(
            indic.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
            "scripts without built-in letters still use raster"
        );
        assert_eq!(escpos_text("A\u{1b}B\n"), b"A?B\n".to_vec());
        // CR must never appear — it overstrikes on many heads.
        assert!(!escpos_text("A\nB").contains(&b'\r'));
        assert_eq!(escpos_text("A\nB"), b"A\nB".to_vec());
    }


    /// ASCII thermal jobs stay plain ESC/POS; Indic still uses GS v 0 raster.
    #[test]
    fn escpos_jobs_prefer_text_when_printer_has_letters() {
        let ascii = escpos_job("KOT 1\n2 x Tea\n1 x Roti");
        assert_eq!(&ascii[..2], &[0x1B, 0x40], "reset");
        assert!(
            !ascii.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
            "plain text for ASCII"
        );
        let indic = escpos_job("ખીચડી");
        assert!(
            indic.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
            "raster for scripts without built-in letters"
        );
        assert!(indic.len() < ESC_POS_JOB_MAX);
        assert!(escpos_text("Café").iter().all(|&b| b != 0xC3));
    }

    #[test]
    fn route_picks_format_printers_are_destinations_only() {
        let mut cfg = DevicePrinters::default();
        let kitchen = cfg.add_printer("Kitchen", "EPSON_TM");
        let office = cfg.add_printer("Office", "Canon_G3010");
        cfg.routes.bill = vec![kitchen.clone(), office.clone()];
        *cfg.route_mut(RouteKey::Kot { station_id: "" }) = vec![office.clone()];
        assert_eq!(cfg.format_id_for(RouteKey::Bill), FORMAT_STANDARD_PORTRAIT);
        assert_eq!(cfg.format_id_for(RouteKey::Kot { station_id: "" }), FORMAT_KOT);
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
        // The whole job, then only the status request (DLE EOT 1) that asks
        // the printer to confirm it read everything.
        let mut want = escpos_job("KOT 12 · D3\n2 x Thali");
        want.extend_from_slice(&[0x10, 0x04, 0x01]);
        assert_eq!(got.join().unwrap(), want);
        // Nobody listening: said in words, not a hang.
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().to_string();
        assert!(print_network(&closed, "x").unwrap_err().contains("cannot reach"));
    }

    /// A printer that answers the status request: "online" is success, and
    /// "offline" (out of paper, cover open) is a failure, never "printed".
    #[test]
    fn network_printer_status_reply_decides_the_outcome() {
        fn fake_printer(reply: u8) -> String {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap().to_string();
            std::thread::spawn(move || {
                let (mut conn, _) = listener.accept().unwrap();
                let mut got = Vec::new();
                let mut buf = [0u8; 4096];
                while let Ok(n) = std::io::Read::read(&mut conn, &mut buf) {
                    if n == 0 {
                        break;
                    }
                    got.extend_from_slice(&buf[..n]);
                    if got.ends_with(&[0x10, 0x04, 0x01]) {
                        let _ = conn.write_all(&[reply]);
                        break;
                    }
                }
            });
            addr
        }
        assert!(print_network(&fake_printer(0x12), "KOT 1\n1 x Chaas").is_ok());
        let err = print_network(&fake_printer(0x1A), "KOT 2\n1 x Chaas").unwrap_err();
        assert!(err.contains("offline"), "{err}");
    }

    /// A Server answering a waiter waits for the paper: a printer that took
    /// the job says nothing, one that is offline is named — and outside a
    /// confirm (the till's own UI thread) nothing ever waits.
    #[test]
    fn confirm_paper_reports_what_the_printer_did() {
        fn fake_printer(reply: u8) -> String {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap().to_string();
            std::thread::spawn(move || {
                let (mut conn, _) = listener.accept().unwrap();
                let mut got = Vec::new();
                let mut buf = [0u8; 4096];
                while let Ok(n) = std::io::Read::read(&mut conn, &mut buf) {
                    if n == 0 {
                        break;
                    }
                    got.extend_from_slice(&buf[..n]);
                    if got.ends_with(&[0x10, 0x04, 0x01]) {
                        let _ = conn.write_all(&[reply]);
                        break;
                    }
                }
            });
            addr
        }
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
        };
        let job = |addr: String| Job {
            dests: vec![PrintDest { target: PrinterTarget::Network(addr), format: kot.clone() }],
            mode: RoutePrintMode::First,
            path: PathBuf::new(),
            text: "KOT 1\n1 x Chaas".into(),
            copies: 1,
            done: confirm_channel(),
        };
        assert!(confirm_channel().is_none(), "no confirm open: nobody waits");
        let paper = ConfirmPaper::start();
        queue(job(fake_printer(0x12))).unwrap();
        assert_eq!(paper.finish(Duration::from_secs(10)), Vec::<String>::new());
        let paper = ConfirmPaper::start();
        queue(job(fake_printer(0x1A))).unwrap();
        let said = paper.finish(Duration::from_secs(10));
        assert!(
            said.iter().any(|p| p.contains("did not print") && p.contains("offline")),
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

    fn picture_widths(job: &[u8]) -> Vec<usize> {
        let mut out = Vec::new();
        let mut i = 0;
        while i + 8 <= job.len() {
            if job[i..i + 4] == [0x1D, 0x76, 0x30, 0x00] {
                let w = job[i + 4] as usize + ((job[i + 5] as usize) << 8);
                let h = job[i + 6] as usize + ((job[i + 7] as usize) << 8);
                out.push(w);
                i += 8 + w * h;
                continue;
            }
            i += 1;
        }
        out
    }

    /// A bill and a kitchen ticket are the same kind of slip: at a 210 mm
    /// content width both go out through the same encoder at the same width,
    /// whichever built-in format the route names.
    #[test]
    fn bill_and_kitchen_escpos_are_the_same_width_at_210mm() {
        let mut cfg = DevicePrinters::default();
        cfg.set_content_line(wrap_cols_for_paper_mm(210));
        let as_escpos = |f: PrintFormat| PrintFormat {
            document: false,
            delivery: DELIVERY_ESCPOS.into(),
            ..f
        };
        let bill_fmt = as_escpos(cfg.format_by_id(FORMAT_STANDARD_PORTRAIT));
        let kot_fmt = as_escpos(cfg.format_by_id(FORMAT_KOT));
        let bill = BillSlipFormat::default().render_sample("Rajdhani", "07/11/2026 12:30");
        let kot = KotSlipFormat::default().render_sample_kind("Rajdhani", KotSlipKind::Original);
        let want = paper_width_dots(210) / 8;
        for (what, job) in [
            ("bill", escpos_job_format(&bill_fmt, &bill)),
            ("kitchen", escpos_job_format(&kot_fmt, &kot)),
        ] {
            let widths = picture_widths(&job);
            assert!(!widths.is_empty(), "{what} slip drew no picture");
            assert!(widths.iter().all(|w| *w == want), "{what}: {widths:?} != {want}");
        }
    }

    #[test]
    fn a_cups_socket_queue_names_its_printer() {
        assert_eq!(
            parse_cups_socket_uri("device for Kitchen: socket://192.168.1.50:9100\n").as_deref(),
            Some("192.168.1.50:9100")
        );
        assert_eq!(
            parse_cups_socket_uri("device for K: socket://kitchen.local/?contimeout=30").as_deref(),
            Some("kitchen.local:9100")
        );
        assert_eq!(
            parse_cups_socket_uri("device for K: socket://[fe80::1]:9100").as_deref(),
            Some("[fe80::1]:9100")
        );
        assert_eq!(parse_cups_socket_uri("device for Canon: usb://Canon/G3010?serial=1"), None);
        assert_eq!(parse_cups_socket_uri("device for Brother: ipp://10.0.0.3/ipp/print"), None);
    }

    /// DLE EOT 1: only a reply with the fixed bits is read, and only the
    /// offline bit is a failure.
    #[test]
    fn printer_status_offline_is_a_failure() {
        assert_eq!(escpos_status_problem(0x12), None);
        assert!(escpos_status_problem(0x1A).is_some());
        // Not a status byte (fixed bits wrong): never a false alarm.
        assert_eq!(escpos_status_problem(0x08), None);
        assert_eq!(escpos_status_problem(0xFF), None);
    }

    /// The Windows raw writer sends the same pieces, in order, with a pause.
    #[test]
    fn windows_raw_writer_sends_each_piece() {
        let job = crate::receipt_raster::escpos_picture(&"ખીચડી\n".repeat(12));
        let pieces = crate::receipt_raster::escpos_piece_lengths(&job);
        let script = powershell_paced_write_loop("KumkajRawPrint", &pieces);
        let list = pieces.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        assert!(script.contains(&format!("$pieces = @({list})")));
        assert!(script.contains("Start-Sleep -Milliseconds 40"));
        assert!(script.contains("[KumkajRawPrint]::WritePrinter"));
        assert!(script.contains("$offset -ne $bytes.Length"));
    }

    /// Printers belong to each shop's books folder and reload from there.
    #[test]
    fn device_printers_are_kept_per_books_dir() {
        let base = std::env::temp_dir().join(format!("khaynjo-printers-{}", uuid::Uuid::new_v4().simple()));
        assert_eq!(load_device_printers(&base), DevicePrinters::default());
        let mut chosen = DevicePrinters::default();
        let kitchen = chosen.add_printer("Kitchen", "192.168.1.50");
        let counter = chosen.add_printer("Counter", "Canon_G3010");
        chosen.routes.bill = vec![counter.clone()];
        *chosen.route_mut(RouteKey::Kot { station_id: "" }) = vec![kitchen.clone()];
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
    fn legacy_install_printers_copy_once_into_empty_shop() {
        let install = std::env::temp_dir().join(format!(
            "khaynjo-pr-legacy-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let shop_a = install.join("servers/a");
        let shop_b = install.join("servers/b");
        fs::create_dir_all(&shop_a).unwrap();
        fs::create_dir_all(&shop_b).unwrap();
        let mut shared = DevicePrinters::default();
        let p = shared.add_printer("Shared", "192.168.1.9");
        shared.routes.bill = vec![p];
        save_device_printers(&install, &shared).unwrap();
        // Empty shop file (created by a premature save) still gets the legacy pool.
        save_device_printers(&shop_a, &DevicePrinters::default()).unwrap();
        migrate_legacy_printers(&install, &shop_a);
        migrate_legacy_printers(&install, &shop_b);
        assert_eq!(load_device_printers(&shop_a).routes.bill.len(), 1);
        assert_eq!(load_device_printers(&shop_b).routes.bill.len(), 1);
        // Later edits stay per-shop.
        let mut only_a = DevicePrinters::default();
        let q = only_a.add_printer("A-only", "10.0.0.1");
        only_a.routes.bill = vec![q];
        save_device_printers(&shop_a, &only_a).unwrap();
        assert_eq!(load_device_printers(&shop_a).printers.len(), 1);
        assert_eq!(
            load_device_printers(&shop_a).printers[0].label,
            "A-only"
        );
        assert_ne!(
            load_device_printers(&shop_b).printers[0].label,
            "A-only"
        );
        let _ = fs::remove_dir_all(install);
    }

    #[test]
    fn legacy_printers_json_migrates() {
        let migrated = migrate_legacy("192.168.1.50", "Canon_G3010");
        assert_eq!(migrated.printers.len(), 2);
        assert_eq!(
            migrated.targets_for(RouteKey::Bill),
            vec![PrinterTarget::Named("Canon_G3010".into())]
        );
        assert_eq!(
            migrated.targets_for(RouteKey::Kot { station_id: "" }),
            vec![PrinterTarget::Network("192.168.1.50:9100".into())]
        );
        // Blank fields → empty routes (no invented "Default" printer).
        let blank = migrate_legacy("", "");
        assert!(blank.printers.is_empty());
        assert!(blank.route_for(RouteKey::Bill).is_empty());
        assert!(blank.route_for(RouteKey::Kot { station_id: "" }).is_empty());
        // Round-trip through serde (legacy file shape).
        let base = std::env::temp_dir().join(format!("khaynjo-legacy-{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("printers.json"),
            r#"{"kot":"10.0.0.9","bill":""}"#,
        )
        .unwrap();
        let loaded = load_device_printers(&base);
        assert_eq!(
            loaded.targets_for(RouteKey::Kot { station_id: "" }),
            vec![PrinterTarget::Network("10.0.0.9:9100".into())]
        );
        assert!(loaded.route_for(RouteKey::Bill).is_empty());
        // Scrub old synthetic Default from a new-format file.
        fs::write(
            base.join("printers.json"),
            r#"{"printers":[{"id":"default","label":"Default","target":""}],"routes":{"bill":["default"],"kot":{"":[]}}}"#,
        )
        .unwrap();
        let scrubbed = load_device_printers(&base);
        assert!(scrubbed.printers.is_empty());
        assert!(scrubbed.route_for(RouteKey::Bill).is_empty());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn each_station_has_its_own_kot_route() {
        let mut cfg = DevicePrinters::default();
        let main = cfg.add_printer("Main", "192.168.1.10");
        let tandoor = cfg.add_printer("Tandoor", "192.168.1.11");
        *cfg.route_mut(RouteKey::Kot { station_id: "" }) = vec![main.clone()];
        *cfg.route_mut(RouteKey::Kot {
            station_id: "st-tandoor",
        }) = vec![tandoor.clone()];
        assert_eq!(
            cfg.targets_for(RouteKey::Kot {
                station_id: "st-tandoor"
            }),
            vec![PrinterTarget::Network("192.168.1.11:9100".into())]
        );
        // Named kitchen with no route entry: don't steal default kitchen / Bill.
        assert!(cfg
            .route_for(RouteKey::Kot {
                station_id: "unknown"
            })
            .is_empty());
        // Default kitchen `""` still has its own Outputs.
        assert_eq!(
            cfg.route_for(RouteKey::Kot { station_id: "" }),
            &[main.clone()][..]
        );
        // Explicit empty list = this station prints nowhere.
        *cfg.route_mut(RouteKey::Kot {
            station_id: "st-bar",
        }) = Vec::new();
        assert!(cfg
            .route_for(RouteKey::Kot {
                station_id: "st-bar"
            })
            .is_empty());
    }

    #[test]
    fn changing_station_printer_replaces_not_appends() {
        let mut cfg = DevicePrinters::default();
        let a = cfg.add_printer("A", "192.168.1.10");
        let b = cfg.add_printer("B", "192.168.1.11");
        let key = RouteKey::Kot {
            station_id: "hot",
        };
        *cfg.route_mut(key) = vec![a];
        // Owner picks a different printer for Hot — must replace.
        *cfg.route_mut(key) = vec![b.clone()];
        assert_eq!(cfg.route_for(key), &[b][..]);
        assert_eq!(
            cfg.targets_for(key),
            vec![PrinterTarget::Network("192.168.1.11:9100".into())]
        );
    }

    #[test]
    fn dedupe_keeps_virtual_printer_station_attach() {
        let mut cfg = DevicePrinters::default();
        let real = cfg.add_printer("80Series2", "80Series2");
        let vp = cfg
            .add_virtual_printer(
                "KOTPrinter",
                &real,
                VirtualPrinterValues {
                    landscape: false,
                    em: STD_EM_NORMAL,
                    line: 40,
                    side: STD_SIDE,
                    delivery: DELIVERY_ESCPOS.into(),
                    paper_width_mm: 80,
                },
            )
            .unwrap();
        let key = RouteKey::Kot {
            station_id: "kitchen-kot",
        };
        assert_eq!(cfg.toggle_virtual_on_route(key, &vp), Some(true));
        assert_eq!(cfg.route_for(key), &[vp.clone()][..]);
        assert!(!cfg.destinations_for(key).is_empty());
        // save_device_printers always dedupes — must not wipe place→VP attach.
        cfg.dedupe_printers();
        assert_eq!(cfg.route_for(key), &[vp.clone()][..]);
        assert!(!cfg.destinations_for(key).is_empty());
    }

    #[test]
    fn vp_destination_uses_place_slip_metrics_not_vp_leftovers() {
        let mut cfg = DevicePrinters::default();
        let real = cfg.add_printer("80Series2", "80Series2");
        // Stale VP metrics (UI no longer edits these — Bill & KOT format owns them).
        let _vp = cfg
            .add_virtual_printer(
                "KOTPrinter",
                &real,
                VirtualPrinterValues {
                    landscape: true,
                    em: 20.0,
                    line: 40,
                    side: 16.0,
                    delivery: DELIVERY_SYSTEM_SPOOLER.into(),
                    paper_width_mm: 80,
                },
            )
            .unwrap();
        let key = RouteKey::Kot {
            station_id: "hot",
        };
        assert_eq!(cfg.toggle_printer_on_route(key, &real), Some(true));
        // Bill & KOT format owns layout, delivery, type size, and content width.
        if let Some(f) = cfg.formats.iter_mut().find(|f| f.id == FORMAT_KOT) {
            f.landscape = true;
            f.document = true;
            f.delivery = DELIVERY_SYSTEM_SPOOLER.into();
            f.em = 28.0;
            f.line = 32;
            f.side = 4.0;
            f.baseline = 30.0;
        }
        let dest = &cfg.destinations_for(key)[0];
        assert!(dest.format.landscape, "layout comes from the KOT format");
        assert!(dest.format.document, "delivery comes from the KOT format");
        assert_eq!(dest.format.em, 28.0);
        assert_eq!(dest.format.line, 32);
        assert_eq!(dest.format.side, 4.0);
    }

    #[test]
    fn save_load_keeps_virtual_printer_station_attach() {
        let dir = std::env::temp_dir().join(format!(
            "khaynjo-vp-route-{}",
            uuid::Uuid::new_v4()
        ));
        let _ = fs::create_dir_all(&dir);
        let mut cfg = DevicePrinters::default();
        let real = cfg.add_printer("80Series2", "80Series2");
        let vp = cfg
            .add_virtual_printer(
                "KOTPrinter",
                &real,
                VirtualPrinterValues {
                    landscape: false,
                    em: STD_EM_NORMAL,
                    line: 40,
                    side: STD_SIDE,
                    delivery: DELIVERY_ESCPOS.into(),
                    paper_width_mm: 80,
                },
            )
            .unwrap();
        let key = RouteKey::Kot {
            station_id: "4d746da7-52b3-49e1-a5ad-5160af639127",
        };
        assert_eq!(cfg.toggle_virtual_on_route(key, &vp), Some(true));
        save_device_printers(&dir, &cfg).unwrap();
        let loaded = load_device_printers(&dir);
        assert_eq!(loaded.route_for(key), &[real][..]);
        assert!(loaded.virtual_printers.is_empty());
        assert!(!loaded.destinations_for(key).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn same_destination_is_not_added_twice() {
        let mut cfg = DevicePrinters::default();
        let (a, first) = cfg.ensure_printer("80Series2", "80Series2");
        let (b, second) = cfg.ensure_printer("80Series2", "80Series2");
        assert!(first);
        assert!(!second);
        assert_eq!(a, b);
        assert_eq!(cfg.printers.len(), 1);
        // Pre-existing duplicates collapse on dedupe.
        cfg.printers.push(NamedPrinter {
            id: "dup".into(),
            label: "80Series2".into(),
            target: "80Series2".into(),
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

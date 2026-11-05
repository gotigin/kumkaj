//! Printing (local free POS — no cloud), the same on every device that can
//! hold the books or take orders.
//!
//! Each device keeps a **pool** of named printers and **routes** (ordered
//! printer ids per feature) in `printers.json` beside the language choice.
//! A route tries the first printer, then the next on failure; an empty
//! route means that feature does not print. KOT routes are per kitchen
//! station (`""` = items with no station assigned — not a shared printer for
//! every kitchen). Each station's route is independent.
//!
//! A printer `target` is a system printer name, or a network address
//! (`192.168.1.50` / `host:port`, ESC/POS). Blank target means this computer's
//! usual OS printer — never invent a pool entry named "Default". Jobs go
//! through one worker thread; failures surface via [`take_problems`].
//!
//! How a job is sent:
//! - **Format** (Standard, KOT, or a custom format) = slip shape. Chosen per
//!   route (Bill → Standard, kitchens → KOT by default); owner can change or
//!   add custom formats.
//! - **Printer target** = transport only: system name / blank, `host:9100`,
//!   `bt:MAC`, `usb:VID:PID`, or `android-print:` (document formats on Android).
//! Routes choose which printers Bill / each kitchen use.

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

/// Route slot: `printer_id` or `printer_id@format_id`.
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

/// One slip format: how the ticket is drawn / sent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrintFormat {
    pub id: String,
    pub label: String,
    /// When true, system / Android PrintManager use a document path; thermal
    /// targets still get ESC/POS raster with these metrics.
    #[serde(default = "default_true")]
    pub document: bool,
    /// Office document orientation (ignored for thermal KOT).
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
fn default_em() -> f32 { 22.0 }
fn default_line() -> u32 { 32 }
fn default_baseline() -> f32 { 24.0 }
fn default_side() -> f32 { 8.0 }

/// Bill / Standard document type-size presets (Settings chips).
/// Applied to every Standard Portrait/Landscape format so orientation
/// routes keep the same em. Rajdhani template seeds Small (18).
pub const STD_EM_SMALL: f32 = 18.0;
pub const STD_EM_NORMAL: f32 = 22.0;
pub const STD_EM_LARGE: f32 = 26.0;

impl PrintFormat {
    /// Alias for [`standard_portrait`] (legacy API name).
    pub fn standard() -> Self {
        Self::standard_portrait()
    }

    pub fn standard_portrait() -> Self {
        Self {
            id: FORMAT_STANDARD_PORTRAIT.into(),
            label: "Standard Portrait".into(),
            document: true,
            landscape: false,
            em: 22.0,
            line: 32,
            baseline: 24.0,
            side: 8.0,
        }
    }

    pub fn standard_landscape() -> Self {
        Self {
            id: FORMAT_STANDARD_LANDSCAPE.into(),
            label: "Standard Landscape".into(),
            document: true,
            landscape: true,
            em: 22.0,
            line: 32,
            baseline: 24.0,
            side: 8.0,
        }
    }

    pub fn kot() -> Self {
        Self {
            id: FORMAT_KOT.into(),
            label: "KOT".into(),
            document: false,
            landscape: false,
            em: 24.0,
            line: 36,
            baseline: 26.0,
            side: 4.0,
        }
    }

    pub fn raster_metrics(&self) -> crate::receipt_raster::RasterMetrics {
        crate::receipt_raster::RasterMetrics {
            em: self.em,
            line: self.line as usize,
            baseline: self.baseline,
            side: self.side,
        }
    }

    /// Built-ins: Portrait, Landscape, KOT. Legacy `"standard"` → Portrait.
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

/// Owner-tunable kitchen slip layout. Always a standard KOT (no prices;
/// dishes always print). Metrics are clamped wide so most heads fit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KotSlipFormat {
    /// Print the big title line (`title` or default). Off for compact slips.
    #[serde(default = "default_true")]
    pub show_title: bool,
    /// Print the shop / place name under the title.
    #[serde(default = "default_true")]
    pub show_shop_name: bool,
    #[serde(default = "default_true")]
    pub show_time: bool,
    #[serde(default = "default_true")]
    pub show_waiter: bool,
    #[serde(default = "default_true")]
    pub show_station: bool,
    /// Print table line when the ticket has a table.
    #[serde(default = "default_true")]
    pub show_table: bool,
    /// Print KOT number when numbered.
    #[serde(default = "default_true")]
    pub show_kot_no: bool,
    /// Floor area / service type (e.g. Dine In).
    #[serde(default)]
    pub show_area: bool,
    /// `Item` / `Qty.` column headers above the dish list.
    #[serde(default)]
    pub show_item_qty_header: bool,
    /// Soft-wrap width in characters. Clamped to [`KOT_CHARS_MIN`]..=[`KOT_CHARS_MAX`].
    #[serde(default = "default_kot_chars")]
    pub chars_per_line: u32,
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
}

pub const KOT_CHARS_MIN: u32 = 16;
pub const KOT_CHARS_MAX: u32 = 80;
pub const KOT_CHARS_58MM: u32 = 32;
pub const KOT_CHARS_80MM: u32 = 42;
pub const KOT_EM_MIN: u32 = 12;
pub const KOT_EM_MAX: u32 = 48;
pub const KOT_EM_SMALL: u32 = 20;
pub const KOT_EM_NORMAL: u32 = 24;
pub const KOT_EM_LARGE: u32 = 28;
pub const KOT_FOOTER_MAX: usize = 80;
pub const KOT_TITLE_MAX: usize = 48;
pub const KOT_COPIES_MIN: u32 = 1;
pub const KOT_COPIES_MAX: u32 = 5;
pub const KOT_DEFAULT_TITLE: &str = "KOT - KITCHEN ORDER";
pub const KOT_SIDE_MIN: f32 = 0.0;
pub const KOT_SIDE_MAX: f32 = 48.0;

fn default_kot_chars() -> u32 { KOT_CHARS_58MM }
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
    fn default() -> Self {
        Self {
            show_title: true,
            show_shop_name: true,
            show_time: true,
            show_waiter: true,
            show_station: true,
            show_table: true,
            show_kot_no: true,
            show_area: false,
            show_item_qty_header: false,
            chars_per_line: KOT_CHARS_58MM,
            em: KOT_EM_NORMAL,
            side: 4.0,
            copies: KOT_COPIES_MIN,
            title: String::new(),
            footer: String::new(),
            field_order: Vec::new(),
        }
    }
}

impl KotSlipFormat {
    /// Compact layout matching the Rajdhani client's preferred other-app slip.
    pub fn rajdhani_default() -> Self {
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
            chars_per_line: KOT_CHARS_58MM,
            em: KOT_EM_SMALL,
            side: 4.0,
            copies: 1,
            title: String::new(),
            footer: String::new(),
            field_order: vec![
                "time".into(),
                "kot_no".into(),
                "area".into(),
                "table".into(),
            ],
        }
        .clamped()
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

    pub fn clamped(mut self) -> Self {
        self.chars_per_line = self.chars_per_line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX);
        self.em = self.em.clamp(KOT_EM_MIN, KOT_EM_MAX);
        self.copies = self.copies.clamp(KOT_COPIES_MIN, KOT_COPIES_MAX);
        self.side = self.side.clamp(KOT_SIDE_MIN, KOT_SIDE_MAX);
        // Strip currency marks — a KOT must never carry prices.
        self.title = scrub_kot_text(&self.title, KOT_TITLE_MAX);
        self.footer = scrub_kot_text(&self.footer, KOT_FOOTER_MAX);
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
        let c = self.clone().clamped();
        format.em = c.em as f32;
        format.line = c.chars_per_line;
        format.baseline = (c.em as f32) + 2.0;
        format.side = c.side;
        format.document = false;
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
        self.render(KotSlipContent {
            cancel: false,
            reprint: false,
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
        let fmt = self.clone().clamped();
        let cols = fmt.chars_per_line as usize;
        let rule = "-".repeat(cols);
        let mut out = String::new();
        if c.reprint {
            out.push_str("*** REPRINT - SAME ORDER ***\n");
        }
        if c.cancel {
            out.push_str("*** CANCEL - STOP THIS ITEM ***\n");
        } else if fmt.show_title {
            out.push_str(fmt.title_line());
            out.push('\n');
        }
        let push = |out: &mut String, line: &str| {
            for part in fmt.wrap_line(line) {
                out.push_str(&part);
                out.push('\n');
            }
        };
        let emit_field = |out: &mut String, id: &str| {
            match id {
                "shop" if fmt.show_shop_name => {
                    let shop = c.shop_name.trim();
                    if !shop.is_empty() {
                        push(out, shop);
                    }
                }
                "kot_no" if fmt.show_kot_no => {
                    if let Some(label) = c.kot_nos_label.filter(|s| !s.trim().is_empty()) {
                        push(out, label);
                    } else if c.kot_no > 0 {
                        if c.reprint {
                            push(out, &format!("Was KOT No: {}", c.kot_no));
                        } else if !fmt.show_title {
                            // Compact other-app style: "KOT - 178"
                            push(out, &format!("KOT - {}", c.kot_no));
                        } else {
                            push(out, &format!("KOT No: {}", c.kot_no));
                        }
                    }
                }
                "time" if fmt.show_time && !c.when.is_empty() => {
                    if fmt.show_title {
                        push(out, &format!("Time: {}", c.when));
                    } else {
                        push(out, c.when);
                    }
                }
                "waiter" if fmt.show_waiter && !c.created_by.is_empty() => {
                    push(out, &format!("By: {}", c.created_by));
                }
                "table" if fmt.show_table && !c.table_label.is_empty() => {
                    if fmt.show_title {
                        push(out, &format!("Table: {}", c.table_label));
                    } else {
                        push(out, &format!("Table No: {}", c.table_label));
                    }
                }
                "station" if fmt.show_station && !c.station_name.is_empty() => {
                    push(out, &format!("Station: {}", c.station_name));
                }
                "area" if fmt.show_area && !c.area_label.trim().is_empty() => {
                    push(out, c.area_label.trim());
                }
                _ => {}
            }
        };
        let order: Vec<&str> = if fmt.field_order.is_empty() {
            vec![
                "shop", "kot_no", "time", "waiter", "area", "table", "station",
            ]
        } else {
            fmt.field_order.iter().map(String::as_str).collect()
        };
        for id in &order {
            emit_field(&mut out, id);
        }
        // Emit any enabled fields not listed in field_order.
        for id in ["shop", "kot_no", "time", "waiter", "area", "table", "station"] {
            if !order.iter().any(|x| *x == id) {
                emit_field(&mut out, id);
            }
        }
        out.push_str(&rule);
        out.push('\n');
        if fmt.show_item_qty_header {
            let left = "Item";
            let right = "Qty.";
            let pad = cols.saturating_sub(left.len() + right.len());
            push(&mut out, &format!("{left}{}{right}", " ".repeat(pad)));
        }
        for item in c.items {
            if let Some((name, qty)) = item.rsplit_once('|') {
                let name = name.trim();
                let qty = qty.trim();
                let pad = cols.saturating_sub(name.chars().count() + qty.chars().count());
                push(&mut out, &format!("{name}{}{qty}", " ".repeat(pad)));
            } else {
                push(&mut out, item);
            }
        }
        out.push_str(&rule);
        out.push('\n');
        if !fmt.footer.is_empty() {
            push(&mut out, &fmt.footer);
        }
        out
    }
}

/// Fields for one kitchen slip body (real KOT or Settings sample).
pub struct KotSlipContent<'a> {
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

#[cfg(test)]
mod kot_slip_format_tests {
    use super::*;

    #[test]
    fn clamps_width_and_strips_currency_from_footer() {
        let fmt = KotSlipFormat {
            chars_per_line: 99,
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
    fn sample_preview_never_shows_prices_and_keeps_kot_no() {
        let text = KotSlipFormat::default().render_sample("Khaynjo");
        assert!(text.contains("KOT - KITCHEN ORDER"));
        assert!(text.contains("KOT No: 12"));
        assert!(text.contains("Table: D1"));
        assert!(text.contains("Butter Roti"));
        assert!(!text.contains('₹'));
        assert!(!text.to_lowercase().contains("total"));
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
            chars_per_line: KOT_CHARS_58MM,
            ..KotSlipFormat::default()
        }
        .render_sample("Shop");
        let wide = KotSlipFormat {
            chars_per_line: KOT_CHARS_80MM,
            ..KotSlipFormat::default()
        }
        .render_sample("Shop");
        let rule_n = "-".repeat(KOT_CHARS_58MM as usize);
        let rule_w = "-".repeat(KOT_CHARS_80MM as usize);
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
        let reprint = hidden.render(KotSlipContent {
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

/// This device's printer pool, formats, and routes (Settings → Printers).
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
            routes: parsed.routes,
        };
        scrub_fake_default_printers(&mut out);
        if out.routes.bill_format.trim().is_empty() {
            out.routes.bill_format = FORMAT_STANDARD_PORTRAIT.into();
        }
        out.migrate_legacy_standard_ids();
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
                if let Some(own) = self.routes.kot.get(station_id) {
                    // Named kitchen: empty list = don't print there (owner's choice).
                    if !station_id.is_empty() {
                        return own.as_slice();
                    }
                    // Default kitchen `""`: use it when non-empty; else Bill.
                    if !own.is_empty() {
                        return own.as_slice();
                    }
                } else if !station_id.is_empty() {
                    // Station never configured — try default kitchen, then Bill.
                    if let Some(kitchen) = self.routes.kot.get("") {
                        if !kitchen.is_empty() {
                            return kitchen.as_slice();
                        }
                    }
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
    }

    /// Resolve route ids to print targets (skips unknown ids).
    pub fn targets_for(&self, key: RouteKey<'_>) -> Vec<PrinterTarget> {
        self.destinations_for(key)
            .into_iter()
            .map(|d| d.target)
            .collect()
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

    /// Resolve route slots to target + that slot's format.
    /// Slot strings are `printer_id` (legacy → place default format) or
    /// `printer_id@format_id` (per-assignment format).
    pub fn destinations_for(&self, key: RouteKey<'_>) -> Vec<PrintDest> {
        let place_default = self.format_for(key);
        self.route_for(key)
            .iter()
            .filter_map(|slot| {
                let (pid, fmt_id) = decode_route_slot(slot);
                let p = self.printers.iter().find(|p| p.id == pid)?;
                let format = if fmt_id.is_empty() {
                    place_default.clone()
                } else {
                    self.format_by_id(&fmt_id)
                };
                Some(PrintDest {
                    target: parse_target(&p.target),
                    format,
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
            em: base.em,
            line: base.line,
            baseline: base.baseline,
            side: base.side,
        });
        id
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

    /// Set type size on all Standard document formats (portrait + landscape).
    /// Baseline tracks em + 2 so line spacing stays readable.
    pub fn set_standard_em(&mut self, em: f32) {
        let em = if em <= (STD_EM_SMALL + STD_EM_NORMAL) / 2.0 {
            STD_EM_SMALL
        } else if em >= (STD_EM_NORMAL + STD_EM_LARGE) / 2.0 {
            STD_EM_LARGE
        } else {
            STD_EM_NORMAL
        };
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
        let map_slot = |slot: &mut String| {
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
            let (pid, _) = decode_route_slot(slot);
            keep_ids.iter().any(|k| k == &pid)
        });
        for ids in self.routes.kot.values_mut() {
            for slot in ids.iter_mut() {
                map_slot(slot);
            }
            ids.dedup();
            ids.retain(|slot| {
                let (pid, _) = decode_route_slot(slot);
                keep_ids.iter().any(|k| k == &pid)
            });
        }
        let _ = keep_ids;
        self.dedupe_route_name_formats();
    }

    /// On each place, keep one slot per shown name + format.
    fn dedupe_route_name_formats(&mut self) {
        let default_bill = self.format_id_for(RouteKey::Bill);
        Self::dedupe_slots_by_name_format(&mut self.routes.bill, &self.printers, &default_bill);
        let stations: Vec<String> = self.routes.kot.keys().cloned().collect();
        for sid in stations {
            let default = self.format_id_for(RouteKey::Kot {
                station_id: sid.as_str(),
            });
            if let Some(slots) = self.routes.kot.get_mut(&sid) {
                Self::dedupe_slots_by_name_format(slots, &self.printers, &default);
            }
        }
    }

    fn dedupe_slots_by_name_format(
        slots: &mut Vec<String>,
        printers: &[NamedPrinter],
        default_fmt: &str,
    ) {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for slot in slots.drain(..) {
            let (pid, fmt) = decode_route_slot(&slot);
            let fmt = if fmt.is_empty() {
                default_fmt.to_string()
            } else {
                fmt
            };
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
            let enc = encode_route_slot(&pid, &fmt);
            if seen.insert(nf) {
                out.push(enc);
            }
        }
        *slots = out;
    }

    pub fn remove_printer(&mut self, id: &str) {
        self.printers.retain(|p| p.id != id);
        self.routes.bill.retain(|slot| decode_route_slot(slot).0 != id);
        for ids in self.routes.kot.values_mut() {
            ids.retain(|slot| decode_route_slot(slot).0 != id);
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
    *PRINTERS.lock().unwrap() = Some(printers);
}

/// The printers in force on this device.
pub fn printers() -> DevicePrinters {
    PRINTERS.lock().unwrap().clone().unwrap_or_default()
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
    printers
}

/// Keep `printers` beside these books and use them from now on.
pub fn save_device_printers(base: &Path, printers: &DevicePrinters) -> Result<(), String> {
    let mut printers = printers.clone();
    printers.dedupe_printers();
    PrintFormat::ensure_builtins(&mut printers.formats);
    fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(&printers).map_err(|e| e.to_string())?;
    fs::write(printers_path(base), raw).map_err(|e| e.to_string())?;
    configure(printers);
    Ok(())
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
    fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Write the slip, then send it along the default route for `kind`.
pub fn print_text(kind: PrintKind, text: &str, app_files: Option<&Path>) -> Result<ReceiptShare, String> {
    print_route(kind, default_route_for(kind), text, app_files)
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
    let path = write_receipt(text, app_files)?;
    let opened = match sink() {
        PrintSink::Os => {
            let dests: Vec<PrintDest> = printers()
                .destinations_for(route)
                .into_iter()
                .filter(|d| can_print_format(&d.target, &d.format))
                .collect();
            if dests.is_empty() {
                if kind == PrintKind::Kot {
                    note_problem(
                        "No printer for that kitchen place — assign one in Settings → Printers → Places"
                            .into(),
                    );
                }
                false
            } else {
                let mode = printers().print_mode_for(route);
                match queue(Job {
                    kind,
                    dests,
                    mode,
                    path: path.clone(),
                    text: text.to_string(),
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
            if !printers().route_for(route).is_empty() {
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
                        "No printer for that kitchen place — assign one in Settings → Printers → Places"
                            .into(),
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
    /// Kept for problem notes / future per-kind wording.
    #[allow(dead_code)]
    kind: PrintKind,
    /// Ordered destinations for this place.
    dests: Vec<PrintDest>,
    /// First success vs send to every destination.
    mode: RoutePrintMode,
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
                    match job.mode {
                        RoutePrintMode::First => {
                            let mut last_err: Option<(String, String)> = None;
                            for dest in &job.dests {
                                match send_to_printer(
                                    &dest.format,
                                    &dest.target,
                                    &job.path,
                                    &job.text,
                                ) {
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
                        RoutePrintMode::All => {
                            let mut any_ok = false;
                            let mut errors: Vec<String> = Vec::new();
                            for dest in &job.dests {
                                match send_to_printer(
                                    &dest.format,
                                    &dest.target,
                                    &job.path,
                                    &job.text,
                                ) {
                                    Ok(()) => any_ok = true,
                                    Err(e) => {
                                        errors.push(format!(
                                            "{} did not print: {e}",
                                            describe(&dest.target)
                                        ));
                                    }
                                }
                            }
                            if !any_ok {
                                for e in errors {
                                    note_problem(e);
                                }
                            } else {
                                for e in errors {
                                    note_problem(e);
                                }
                            }
                        }
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
) -> Result<(), String> {
    let bytes = escpos_job_format(format, text);
    match target {
        PrinterTarget::Network(addr) => print_network_bytes(addr, &bytes),
        PrinterTarget::Bluetooth(mac) => crate::print_bt::write_bytes(mac, &bytes),
        PrinterTarget::Usb(id) => crate::print_usb::write_bytes(id, &bytes),
        PrinterTarget::AndroidPrint => {
            if !format.document {
                return Err(format!(
                    "format \"{}\" needs Network, Bluetooth, or USB — Android print is for document formats",
                    format.label
                ));
            }
            print_android_document(text, format.landscape)
        }
        PrinterTarget::Default => {
            if format.document {
                print_system_standard(None, path, text, format.landscape)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB — not the system print service",
                    format.label
                ))
            } else {
                print_system_raw(None, &bytes)
            }
        }
        PrinterTarget::Named(name) => {
            if format.document {
                print_system_standard(Some(name), path, text, format.landscape)
            } else if cfg!(target_os = "android") {
                Err(format!(
                    "format \"{}\" on Android needs Network, Bluetooth, or USB — not a system printer name",
                    format.label
                ))
            } else {
                print_system_raw(Some(name), &bytes)
            }
        }
    }
}

fn print_android_document(text: &str, landscape: bool) -> Result<(), String> {
    // Same Noto PNG as desktop Standard — never rely on the device font.
    let png = crate::receipt_raster::png_document(text)?;
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
    landscape: bool,
) -> Result<(), String> {
    let png = crate::receipt_raster::png_document(text)?;
    let png_path = path.with_extension("png");
    fs::write(&png_path, &png).map_err(|e| e.to_string())?;
    #[cfg(target_os = "android")]
    {
        let _ = name;
        crate::print_android::print_png("Khaynjo", &png, landscape)
    }
    #[cfg(not(target_os = "android"))]
    {
        print_system_image(name, &png_path, landscape)
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
    let mut stream = std::net::TcpStream::connect_timeout(&sock, Duration::from_secs(4))
        .map_err(|e| format!("cannot reach it ({e}) — is it on and on this network?"))?;
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    // Chunk writes — some heads drop mid-job when the whole raster lands at once.
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
    if printer_has_letters(&cleaned) {
        let cols = format.line.clamp(KOT_CHARS_MIN, KOT_CHARS_MAX) as usize;
        escpos_job_text_cols(&cleaned, cols)
    } else {
        crate::receipt_raster::escpos_picture_metrics(&cleaned, &format.raster_metrics())
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
    let cols = (cols as u32).clamp(KOT_CHARS_MIN, KOT_CHARS_MAX) as usize;
    let wrapped = wrap_escpos_cols(text, cols);
    let mut out = Vec::with_capacity(wrapped.len().saturating_add(48));
    out.extend_from_slice(&[0x1B, 0x40]); // ESC @: reset
    out.extend_from_slice(&[0x1B, 0x74, 0x00]); // ESC t 0: PC437
    out.extend_from_slice(&[0x1B, 0x4D, 0x00]); // ESC M 0: Font A
    out.extend_from_slice(&[0x1D, 0x21, 0x00]); // GS ! 0: normal size
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
fn print_system_image(name: Option<&str>, path: &Path, landscape: bool) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let mut cmd = Command::new("lp");
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
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
    let Some(printer) = name else {
        return Ok(());
    };
    // Backend updates printer-state-message shortly after accept.
    std::thread::sleep(Duration::from_millis(500));
    if let Some(blocked) = cups_printer_unreachable(printer) {
        if let Some(job) = &job {
            let _ = Command::new("cancel").arg(job).output();
        }
        eprintln!("[print] CUPS unreachable: {blocked}");
        crate::diag::warn(crate::diag::Cat::Print, "cups_unreachable", &blocked);
        return Err(blocked);
    }
    Ok(())
}

/// Send ESC/POS bytes to a system printer as a **raw** job (KOT on desktop queues).
fn print_system_raw(name: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > ESC_POS_JOB_MAX {
        return Err(format!(
            "print job too large ({} bytes) — refused so the kitchen printer is not flooded",
            bytes.len()
        ));
    }
    let dir = std::env::temp_dir().join("kumkaj-kot-raw");
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
        // Raw spool: bytes go to the device as-is (same idea as host:9100).
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
        # WritePrinter often returns success after only part of the buffer —
        # loop until every byte is accepted or the ticket prints truncated.
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
                name = quote(name),
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
        let _ = (name, path);
        Err(
            "Android has no system printer drivers for kitchen tickets — use a network address (host:9100) in Settings → Printers"
                .into(),
        )
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
            return clip.set_text(text.to_string()).is_ok();
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
    configure(cfg);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    configure(DevicePrinters::default());
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
    fn missing_station_falls_back_to_bill_printer() {
        let mut cfg = DevicePrinters::default();
        let p = cfg.add_printer("Counter", "192.168.1.50");
        cfg.routes.bill = vec![p.clone()];
        // No KOT map entries at all — legacy fallthrough to Bill.
        assert_eq!(
            cfg.route_for(RouteKey::Kot { station_id: "" }),
            &[p.clone()]
        );
        assert_eq!(
            cfg.route_for(RouteKey::Kot {
                station_id: "station-a"
            }),
            &[p.clone()]
        );
        // An explicit empty station entry means don't print there — not Bill.
        cfg.ensure_station_routes(["station-a"]);
        assert!(cfg
            .route_for(RouteKey::Kot {
                station_id: "station-a"
            })
            .is_empty());
        assert_eq!(cfg.route_for(RouteKey::Bill), &[p][..]);
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
        assert_eq!(got.join().unwrap(), escpos_job("KOT 12 · D3\n2 x Thali"));
        // Nobody listening: said in words, not a hang.
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().to_string();
        assert!(print_network(&closed, "x").unwrap_err().contains("cannot reach"));
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
        // save/load fill bare ids with place default format (`id@format`).
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
        // Unknown station (no map entry) still falls back to default kitchen.
        assert_eq!(
            cfg.route_for(RouteKey::Kot {
                station_id: "unknown"
            }),
            &[main.clone()][..]
        );
        // Explicit empty list = this station prints nowhere (not a shared pool).
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
        // Bare id is filled with the place's default format on dedupe.
        assert_eq!(
            cfg.route_for(RouteKey::Bill),
            &[encode_route_slot(&a, FORMAT_STANDARD_PORTRAIT)][..]
        );
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

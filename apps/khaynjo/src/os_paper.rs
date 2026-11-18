//! How a slip lands on paper, and the preview of it (docs/PRINTER.md §
//! Paper and preview). Nothing in here knows a size: no A4, no Letter, no
//! 210 mm.
//!
//! **The printer's own paper.** An OS document prints the standard way: on
//! the printer's own page exactly as its driver is set ([`query`] reads it;
//! the screens call it the *reported* paper). The app never names, makes up
//! or changes a page size (owner, 2026-11-17).
//!
//! **The virtual paper.** Each printer also holds a virtual paper on
//! Settings → Printers ([`PaperMm`]: width, and length or a roll) — the
//! paper the owner says is in it; Find fills a blank one from the driver.
//! It is never sent to the printer. It says how much of the printer's page
//! is really paper: a page is laid on the [`Sheet`]'s area — what the
//! printer reaches of its own page, from that page's printable corner, and
//! no further than the virtual paper.
//!
//! On that area the slip picture goes at its real size — its px are 1/96
//! inch (rule 2) — at the format's [`Place`] (top-left unless the owner
//! picked another). Nothing is fitted or shrunk: a bill longer than the
//! area goes on to the next page, split between lines; a bill wider than it
//! is cut at its edge, the same as a column past the content width. The
//! preview ([`preview_picture`]) is drawn from the same plan
//! ([`plan_sheets`]), so it is the print — and what the print cuts off is
//! drawn past the edge, faded, so a slip wider than the paper never looks
//! as if it fits (owner, 2026-11-17).

use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::time::Duration;

/// One paper size a printer's driver offers.
#[derive(Clone, Debug, PartialEq)]
pub struct Paper {
    /// The driver's own name for it (CUPS `PageSize` choice, the Windows
    /// form name).
    pub id: String,
    /// The driver's name for people ("4 x 6 in", "A5", …).
    pub label: String,
    pub width_mm: f64,
    pub height_mm: f64,
    /// Edges the printer cannot reach, mm: left, top, right, bottom.
    pub margins_mm: [f64; 4],
}

/// PostScript points (PPD sizes, PDF pages) to mm.
pub const PT_MM: f64 = 25.4 / 72.0;

impl Paper {
    /// Width and height the printer can reach, mm.
    pub fn printable_mm(&self) -> (f64, f64) {
        let [l, t, r, b] = self.margins_mm;
        (
            (self.width_mm - l - r).max(1.0),
            (self.height_mm - t - b).max(1.0),
        )
    }

    /// The same sheet turned a quarter clockwise, as a landscape slip is
    /// (`slip_page_png` turns it with `rotate90`). A point (x, y) on the
    /// turned sheet lands on (width - y, x) of the upright one, so its left
    /// edge is the upright top, its top is the upright right, and so on.
    pub fn turned(&self) -> Paper {
        let [l, t, r, b] = self.margins_mm;
        Paper {
            id: self.id.clone(),
            label: self.label.clone(),
            width_mm: self.height_mm,
            height_mm: self.width_mm,
            margins_mm: [t, r, b, l],
        }
    }

    /// This page as a virtual paper: all of it is paper (a printer with no
    /// virtual paper set).
    pub fn as_virtual(&self) -> PaperMm {
        PaperMm { width_mm: self.width_mm, length_mm: self.height_mm }
    }
}

/// A number of mm for people: whole when whole, else one decimal.
pub fn mm_text(mm: f64) -> String {
    let r = (mm * 10.0).round() / 10.0;
    if (r - r.round()).abs() < 1e-9 {
        format!("{}", r.round() as i64)
    } else {
        format!("{r:.1}")
    }
}

/// The sizes one printer's driver offers and the one it is set to now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrinterPapers {
    /// The driver's current paper (`id` of one of `papers`).
    pub default_id: String,
    pub papers: Vec<Paper>,
}

impl PrinterPapers {
    pub fn by_id(&self, id: &str) -> Option<&Paper> {
        let id = id.trim();
        if id.is_empty() {
            return None;
        }
        self.papers.iter().find(|p| p.id == id)
    }

    /// The printer's own paper: the one its driver is set to now — what a
    /// job prints on, and what the screens show as *reported*.
    pub fn default_paper(&self) -> Option<&Paper> {
        self.by_id(&self.default_id).or_else(|| self.papers.first())
    }
}

/// A printer's virtual paper (Settings → Printers): width, and the sheet's
/// length, mm; a length of 0 is a roll.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaperMm {
    pub width_mm: f64,
    pub length_mm: f64,
}

impl PaperMm {
    /// `None` while the width is not known.
    pub fn new(width_mm: f64, length_mm: f64) -> Option<Self> {
        (width_mm > 0.0).then_some(Self {
            width_mm,
            length_mm: if length_mm > 0.0 { length_mm } else { 0.0 },
        })
    }

    pub fn is_roll(&self) -> bool {
        self.length_mm <= 0.0
    }
}

/// Where the content sits on a paper bigger than it (owner, 2026-11-17):
/// one of nine places, top / middle / bottom × left / centre / right, kept
/// per format (Bill, KOT). Left and right are the paper's own sides in
/// every language — the screen's grid is a picture of the paper, and the
/// picked square is where the slip lands. Top-left is where content sat
/// before places existed, so nothing moves until the owner picks another.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Place {
    /// 0 top, 1 middle, 2 bottom.
    #[serde(default)]
    pub down: u8,
    /// 0 left, 1 centre, 2 right.
    #[serde(default)]
    pub across: u8,
}

impl Place {
    /// Clamped to the nine places.
    pub fn new(down: u8, across: u8) -> Self {
        Self { down: down.min(2), across: across.min(2) }
    }

    /// Top-left: where content sat before places existed.
    pub fn is_top_left(&self) -> bool {
        *self == Self::default()
    }
}

/// How a slip is laid on its sheet: turned (landscape) or not, and its
/// [`Place`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lay {
    pub landscape: bool,
    pub place: Place,
}

/// Where a `w` × `h` slip goes in an area `aw` × `ah` (px): its left edge
/// and top, from the area's top-left. Across: at the left, the centre or
/// the right of the area — a slip wider than the area hangs over by the
/// same rule (left: cut at the right; right: cut at the left, a negative
/// left edge). Down counts only when `down` (the whole slip on one sheet
/// that has a length) and it fits.
pub fn place_in(place: Place, w: u32, h: u32, aw: u32, ah: u32, down: bool) -> (i64, u32) {
    let free = i64::from(aw) - i64::from(w);
    let x = free * i64::from(place.across.min(2)) / 2;
    let y = if down && h <= ah { (ah - h) * u32::from(place.down.min(2)) / 2 } else { 0 };
    (x, y)
}

/// `src` laid with its left edge at `x` (negative: it starts past the left)
/// on a white strip `w` wide, cut at the strip's edges — an ESC/POS roll.
pub fn place_on(src: &image::GrayImage, x: i64, w: u32) -> image::GrayImage {
    let (sw, sh) = src.dimensions();
    let w = w.max(1);
    let mut out = image::GrayImage::from_pixel(w, sh.max(1), image::Luma([255]));
    let from = (-x).max(0).min(i64::from(sw)) as u32;
    let to = x.max(0).min(i64::from(w)) as u32;
    let n = sw.saturating_sub(from).min(w.saturating_sub(to)) as usize;
    let raw = src.as_raw();
    for y in 0..sh {
        let s = (y * sw + from) as usize;
        let d = (y * w + to) as usize;
        out.as_mut()[d..d + n].copy_from_slice(&raw[s..s + n]);
    }
    out
}

/// The sheet a job lands on, upright, mm: the virtual paper (Settings →
/// Printers — the paper the owner says the printer holds) and, on it, the
/// area a page is laid on: what the printer reaches of its own page (as
/// its driver is set) from that page's printable corner, and no further
/// than the virtual paper. Pages are cut for this area, the place is
/// measured on it and the preview draws it, so the print and the preview
/// are one picture (owner, 2026-11-17). Never sent to the printer: a job
/// prints on the printer's own paper.
#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    /// The virtual paper's width, mm.
    pub width_mm: f64,
    /// Its length, mm; 0 = a roll (each page as long as it needs).
    pub length_mm: f64,
    /// Edges the printer cannot reach — its own page's — mm: left, top,
    /// right, bottom. 0 while its driver has not said.
    pub margins_mm: [f64; 4],
    /// Width and height of the area a page is laid on, mm.
    pub area_mm: (f64, f64),
}

/// The [`Sheet`] for a printer's virtual paper. `own` is the printer's own
/// page as its driver is set (`None` while it has not said: the virtual
/// paper alone, edges unknown). `slip_mm` is the slip's width and inked
/// length, mm, upright: a roll's page is as long as the slip needs — its
/// length, or its width when it lies across the roll (landscape) — and
/// never longer than the printer's own page.
pub fn sheet_for(virtual_paper: PaperMm, own: Option<&Paper>, slip_mm: (f64, f64), landscape: bool) -> Sheet {
    let margins = own.map(|p| p.margins_mm).unwrap_or([0.0; 4]);
    let [l, t, r, b] = margins;
    let virtual_w = (virtual_paper.width_mm - l - r).max(1.0);
    let virtual_h = if virtual_paper.is_roll() {
        // Half a millimetre more than the slip, so rounding to whole rows
        // never leaves its last line for a second page.
        (if landscape { slip_mm.0 } else { slip_mm.1 }) + 0.5
    } else {
        (virtual_paper.length_mm - t - b).max(1.0)
    };
    let area_mm = match own.map(Paper::printable_mm) {
        Some((w, h)) => (w.min(virtual_w), h.min(virtual_h)),
        None => (virtual_w, virtual_h),
    };
    Sheet {
        width_mm: virtual_paper.width_mm,
        length_mm: virtual_paper.length_mm.max(0.0),
        margins_mm: margins,
        area_mm,
    }
}

// ---------- CUPS (Linux, macOS): the queue's PPD ----------

/// Read the sizes a PPD lists (`*PageSize` order), their `*PaperDimension`
/// and `*ImageableArea` (points, origin bottom-left), and `*DefaultPageSize`.
/// A size with no imageable area takes `*HWMargins` (else no margins: the
/// driver did not say).
pub fn parse_ppd(ppd: &str) -> PrinterPapers {
    let mut default_id = String::new();
    let mut order: Vec<String> = Vec::new();
    let mut labels: HashMap<String, String> = HashMap::new();
    let mut dims: HashMap<String, (f64, f64)> = HashMap::new();
    let mut areas: HashMap<String, [f64; 4]> = HashMap::new();
    let mut hw: Option<[f64; 4]> = None;
    let numbers = |v: &str| -> Vec<f64> {
        v.trim()
            .trim_matches('"')
            .split_whitespace()
            .filter_map(|n| n.parse::<f64>().ok())
            .collect()
    };
    for raw in ppd.lines() {
        let line = raw.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix("*DefaultPageSize:") {
            default_id = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = line.strip_prefix("*HWMargins:") {
            let n = numbers(rest);
            if n.len() == 4 {
                // left bottom right top
                hw = Some([n[0], n[1], n[2], n[3]]);
            }
            continue;
        }
        let (kind, rest) = if let Some(r) = line.strip_prefix("*PageSize ") {
            (0, r)
        } else if let Some(r) = line.strip_prefix("*PaperDimension ") {
            (1, r)
        } else if let Some(r) = line.strip_prefix("*ImageableArea ") {
            (2, r)
        } else {
            continue;
        };
        let Some((spec, value)) = rest.split_once(':') else {
            continue;
        };
        let (name, label) = match spec.split_once('/') {
            Some((n, l)) => (n.trim(), l.trim()),
            None => (spec.trim(), ""),
        };
        if name.is_empty() {
            continue;
        }
        match kind {
            0 => {
                if !order.iter().any(|n| n == name) {
                    order.push(name.to_string());
                }
                if !label.is_empty() {
                    labels.insert(name.to_string(), label.to_string());
                }
            }
            1 => {
                let n = numbers(value);
                if n.len() == 2 && n[0] > 0.0 && n[1] > 0.0 {
                    dims.insert(name.to_string(), (n[0], n[1]));
                }
                if !label.is_empty() {
                    labels.entry(name.to_string()).or_insert_with(|| label.to_string());
                }
            }
            _ => {
                let n = numbers(value);
                if n.len() == 4 {
                    areas.insert(name.to_string(), [n[0], n[1], n[2], n[3]]);
                }
            }
        }
    }
    // Sizes with a dimension but no *PageSize line still count.
    let mut names = order;
    let mut extra: Vec<&String> = dims.keys().filter(|k| !names.contains(k)).collect();
    extra.sort();
    names.extend(extra.into_iter().cloned());
    let papers: Vec<Paper> = names
        .into_iter()
        .filter_map(|name| {
            let (w, h) = *dims.get(&name)?;
            let margins_pt = match areas.get(&name) {
                // llx lly urx ury → left top right bottom
                Some([llx, lly, urx, ury]) => [*llx, h - *ury, w - *urx, *lly],
                None => match hw {
                    Some([l, b, r, t]) => [l, t, r, b],
                    None => [0.0; 4],
                },
            };
            let margins_mm = margins_pt.map(|m| (m * PT_MM).max(0.0));
            Some(Paper {
                label: labels.get(&name).cloned().unwrap_or_else(|| name.clone()),
                id: name,
                width_mm: w * PT_MM,
                height_mm: h * PT_MM,
                margins_mm,
            })
        })
        .collect();
    PrinterPapers { default_id, papers }
}

/// The size an `lpoptions` default names, matched to the driver's list:
/// a choice name (`PageSize=A5`), or a self-describing PWG media name
/// (`media=iso_a5_148x210mm`, `na_index-4x6_4x6in`) matched by its size.
pub fn resolve_media_name(papers: &PrinterPapers, value: &str) -> Option<String> {
    let v = value.split(',').next().unwrap_or("").trim().trim_matches('\'').trim_matches('"');
    if v.is_empty() {
        return None;
    }
    if let Some(p) = papers.papers.iter().find(|p| p.id == v) {
        return Some(p.id.clone());
    }
    let (w, h) = pwg_size_mm(v)?;
    papers
        .papers
        .iter()
        .find(|p| (p.width_mm - w).abs() < 0.6 && (p.height_mm - h).abs() < 0.6)
        .map(|p| p.id.clone())
}

/// `…_<w>x<h>mm` / `…_<w>x<h>in` (PWG 5101.1 self-describing names).
fn pwg_size_mm(name: &str) -> Option<(f64, f64)> {
    let last = name.rsplit('_').next()?;
    let (nums, unit) = if let Some(n) = last.strip_suffix("mm") {
        (n, 1.0)
    } else if let Some(n) = last.strip_suffix("in") {
        (n, 25.4)
    } else {
        return None;
    };
    let (w, h) = nums.split_once('x')?;
    Some((w.parse::<f64>().ok()? * unit, h.parse::<f64>().ok()? * unit))
}

/// The page-size default an `lpoptions -p <queue>` line sets, if any.
pub fn lpoptions_media(line: &str) -> Option<String> {
    for tok in line.split_whitespace() {
        for key in ["PageSize=", "media="] {
            if let Some(v) = tok.strip_prefix(key) {
                let v = v.trim_matches('\'').trim_matches('"');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Where the CUPS scheduler answers: `CUPS_SERVER` when it names a host,
/// else this computer.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_base() -> String {
    match std::env::var("CUPS_SERVER") {
        Ok(s) if !s.trim().is_empty() && !s.trim().starts_with('/') => {
            let s = s.trim();
            if s.contains(':') {
                format!("http://{s}")
            } else {
                format!("http://{s}:631")
            }
        }
        _ => "http://localhost:631".into(),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn url_part(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The queue `lp` uses when none is named.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn cups_default_queue() -> Result<String, String> {
    let out = std::process::Command::new("lpstat")
        .arg("-d")
        .output()
        .map_err(|e| format!("lpstat did not start ({e})"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .find_map(|l| l.split_once(':').map(|(_, q)| q.trim().to_string()))
        .filter(|q| !q.is_empty())
        .ok_or_else(|| "this computer has no default printer".into())
}

/// A CUPS queue's papers: its PPD from the scheduler (sizes, printable
/// areas, the driver default), then the owner's own `lpoptions` default.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn query_cups(queue: Option<&str>) -> Result<(String, PrinterPapers), String> {
    let queue = match queue.map(str::trim).filter(|q| !q.is_empty()) {
        Some(q) => q.to_string(),
        None => cups_default_queue()?,
    };
    let url = format!("{}/printers/{}.ppd", cups_base(), url_part(&queue));
    // Said plainly on screen; never a URL or a library's error text.
    let body = ureq::get(&url)
        .timeout(Duration::from_secs(4))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(404, _) => {
                format!("{queue} is not on this computer's printing system")
            }
            ureq::Error::Status(code, _) => {
                format!("the printing system answered {code} for {queue}")
            }
            ureq::Error::Transport(_) => "the printing system did not answer".to_string(),
        })?
        .into_string()
        .map_err(|e| format!("the paper sizes of {queue} could not be read ({e})"))?;
    let mut papers = parse_ppd(&body);
    if papers.papers.is_empty() {
        return Err(format!("the driver of {queue} lists no paper sizes"));
    }
    if let Ok(out) = std::process::Command::new("lpoptions").args(["-p", &queue]).output() {
        let line = String::from_utf8_lossy(&out.stdout);
        if let Some(name) = lpoptions_media(&line) {
            if let Some(id) = resolve_media_name(&papers, &name) {
                papers.default_id = id;
            }
        }
    }
    Ok((queue, papers))
}

/// One printer's papers from this system (`target` blank = the default
/// printer). Android has no list here: its print dialog asks for the paper.
pub fn query(target: &str) -> Result<PrinterPapers, String> {
    let name = Some(target.trim()).filter(|t| !t.is_empty());
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        query_cups(name).map(|(_, p)| p)
    }
    #[cfg(target_os = "windows")]
    {
        // Through the Windows print API itself (print_win.rs).
        crate::print_win::papers(name)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = name;
        Err("the paper is chosen in the system print dialog".into())
    }
}

// ---------- Papers for the screen (read off the UI thread) ----------

enum Slot {
    Reading,
    Ready(Result<PrinterPapers, String>),
}

static CACHE: Mutex<Option<HashMap<String, Slot>>> = Mutex::new(None);
static NOTIFY: Mutex<Option<fn()>> = Mutex::new(None);
static READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// True once after a printer's papers arrived (the screen repaints them).
pub fn take_ready() -> bool {
    READY.swap(false, std::sync::atomic::Ordering::SeqCst)
}

/// Called (from a reading thread) when a printer's papers are ready.
pub fn set_notify(hook: fn()) {
    *NOTIFY.lock().unwrap() = Some(hook);
}

/// The papers read for `target`, if ready. Starts reading them (once, on
/// its own thread) when they are not; the screen is told when they are.
pub fn papers_for(target: &str) -> Option<Result<PrinterPapers, String>> {
    let key = target.trim().to_string();
    let mut guard = CACHE.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    match map.get(&key) {
        Some(Slot::Ready(r)) => return Some(r.clone()),
        Some(Slot::Reading) => return None,
        None => {}
    }
    if cfg!(test) {
        return None;
    }
    map.insert(key.clone(), Slot::Reading);
    drop(guard);
    let spawned = std::thread::Builder::new()
        .name("paper-sizes".into())
        .spawn(move || {
            let result = query(&key);
            if let Some(map) = CACHE.lock().unwrap().as_mut() {
                map.insert(key, Slot::Ready(result));
            }
            READY.store(true, std::sync::atomic::Ordering::SeqCst);
            if let Some(hook) = *NOTIFY.lock().unwrap() {
                hook();
            }
        });
    if spawned.is_err() {
        if let Some(map) = CACHE.lock().unwrap().as_mut() {
            map.remove(target.trim());
        }
    }
    None
}

/// Keep a driver's answer read elsewhere (Find's own thread), so the
/// screen does not ask again.
pub fn remember(target: &str, answer: Result<PrinterPapers, String>) {
    let mut guard = CACHE.lock().unwrap();
    guard
        .get_or_insert_with(HashMap::new)
        .insert(target.trim().to_string(), Slot::Ready(answer));
}

/// Every driver answer read so far (target → answer); none is asked for.
pub fn answers() -> HashMap<String, Result<PrinterPapers, String>> {
    let guard = CACHE.lock().unwrap();
    guard
        .as_ref()
        .map(|m| {
            m.iter()
                .filter_map(|(k, s)| match s {
                    Slot::Ready(r) => Some((k.clone(), r.clone())),
                    Slot::Reading => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Read every printer's papers again (after Find, or a driver change).
pub fn forget_all() {
    if let Some(map) = CACHE.lock().unwrap().as_mut() {
        map.retain(|_, s| matches!(s, Slot::Reading));
    }
}

/// Try again the printers whose papers could not be read.
pub fn forget_failed() {
    if let Some(map) = CACHE.lock().unwrap().as_mut() {
        map.retain(|_, s| !matches!(s, Slot::Ready(Err(_))));
    }
}

// ---------- Placing the slip on the paper ----------

/// Rows where a sheet may end: the middle row of each blank band (a row
/// with no ink) between the slip's lines.
pub fn line_gaps(gray: &[u8], w: usize, h: usize) -> Vec<u32> {
    let w = w.max(1);
    let ink = |y: usize| gray[y * w..(y + 1) * w].iter().any(|&v| v < 200);
    let mut out = Vec::new();
    let mut y = 0usize;
    while y < h {
        if ink(y) {
            y += 1;
            continue;
        }
        let start = y;
        while y < h && !ink(y) {
            y += 1;
        }
        out.push(((start + y) / 2) as u32);
    }
    out
}

/// Last row with ink, plus one (0 for a blank slip).
pub fn ink_end(gray: &[u8], w: usize, h: usize) -> u32 {
    let w = w.max(1);
    (0..h)
        .rev()
        .find(|&y| gray[y * w..(y + 1) * w].iter().any(|&v| v < 200))
        .map(|y| y as u32 + 1)
        .unwrap_or(0)
}

/// Where each sheet ends along a slip `len` px long when a sheet holds
/// `page_len` px: in the last blank band that falls in the sheet's second
/// half, so no line is cut in two; at `page_len` only when a sheet has no
/// such band (one picture taller than half a sheet).
pub fn page_ends(len: u32, page_len: u32, gaps: &[u32]) -> Vec<u32> {
    let page_len = page_len.max(1);
    let mut ends = Vec::new();
    let mut start = 0u32;
    while start < len {
        let limit = start.saturating_add(page_len);
        if limit >= len {
            ends.push(len);
            break;
        }
        let floor = start + page_len / 2;
        let end = gaps
            .iter()
            .rev()
            .copied()
            .find(|&g| g > floor && g <= limit)
            .unwrap_or(limit);
        ends.push(end);
        start = end;
    }
    if ends.is_empty() {
        ends.push(len.max(1));
    }
    ends
}

/// The slip on `sheet`, one grey picture per page, each the area's width at
/// the slip's own density (`content_mm` is the slip's width on paper), the
/// slip at its [`Place`] (`lay`; top-left by default). Portrait pages are
/// as long as their part of the slip — the whole area's height when the
/// slip sits lower than the top; a landscape slip is laid on the turned
/// area and turned back, filling the area so it lands where it was laid.
/// Each picture goes at the printable corner of the printer's own page.
/// Returns the pages and the slip's px per mm.
pub fn os_document_sheets(
    slip: &image::GrayImage,
    content_mm: f64,
    sheet: &Sheet,
    lay: Lay,
) -> (Vec<image::GrayImage>, f64) {
    let plan = plan_sheets(slip, content_mm, sheet, lay);
    (render_sheets(slip, &plan), plan.px_per_mm)
}

/// Where a slip's pages end on a [`Sheet`], how big each page is and where
/// the slip sits on it, in the slip's own pixels — the print's geometry,
/// shared with the preview.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetPlan {
    pub px_per_mm: f64,
    /// The area a page is laid on, as the slip lies on it (turned for
    /// landscape): width along its lines, height along its rows; mm.
    pub area_mm: (f64, f64),
    /// That area, px.
    pub sheet_w: u32,
    pub sheet_h: u32,
    /// Row (in the slip) after the last row of each page.
    pub ends: Vec<u32>,
    pub lay: Lay,
    /// A roll has no length: down does not count there.
    pub roll: bool,
    /// The slip's width and inked length, px.
    pub slip_w: u32,
    pub len: u32,
    /// The slip's left edge and top on each page ([`place_in`]).
    pub x: i64,
    pub y: u32,
}

pub fn plan_sheets(slip: &image::GrayImage, content_mm: f64, sheet: &Sheet, lay: Lay) -> SheetPlan {
    let (sw, sh) = slip.dimensions();
    let px_per_mm = f64::from(sw.max(1)) / content_mm.max(1.0);
    let raw = slip.as_raw();
    let len = ink_end(raw, sw as usize, sh as usize).max(1);
    let gaps = line_gaps(raw, sw as usize, sh as usize);
    let (aw, ah) = sheet.area_mm;
    let area_mm = if lay.landscape { (ah, aw) } else { (aw, ah) };
    plan_on(px_per_mm, area_mm, sheet.length_mm <= 0.0, lay, sw, len, |page_len| {
        page_ends(len, page_len, &gaps)
    })
}

/// The plan for the same slip drawn `1 / div` the size (its dots, for the
/// preview): the same pages breaking at the same lines, the slip at the
/// same place.
fn plan_scaled(plan: &SheetPlan, div: u32) -> SheetPlan {
    let div = div.max(1);
    let ends: Vec<u32> = plan.ends.iter().map(|e| e.div_ceil(div)).collect();
    plan_on(
        plan.px_per_mm / f64::from(div),
        plan.area_mm,
        plan.roll,
        plan.lay,
        plan.slip_w.div_ceil(div),
        plan.len.div_ceil(div),
        |_| ends,
    )
}

fn plan_on(
    px_per_mm: f64,
    area_mm: (f64, f64),
    roll: bool,
    lay: Lay,
    slip_w: u32,
    len: u32,
    ends: impl FnOnce(u32) -> Vec<u32>,
) -> SheetPlan {
    // Floor: a page never reaches past the area's edge.
    let sheet_w = ((area_mm.0 * px_per_mm).floor() as u32).max(1);
    let sheet_h = ((area_mm.1 * px_per_mm).floor() as u32).max(1);
    let ends = ends(sheet_h);
    let (x, y) = place_in(lay.place, slip_w, len, sheet_w, sheet_h, !roll && ends.len() == 1);
    SheetPlan { px_per_mm, area_mm, sheet_w, sheet_h, ends, lay, roll, slip_w, len, x, y }
}

/// The pages of `plan`, one grey picture each, the area's width (the slip
/// at its place, cut at the area's edges). Portrait pages are as long as
/// their part, or the whole area's height when the slip sits lower than
/// the top; a landscape slip is laid on the turned area and turned back,
/// filling the area so it lands where it was laid.
pub fn render_sheets(slip: &image::GrayImage, plan: &SheetPlan) -> Vec<image::GrayImage> {
    let (sw, sh) = slip.dimensions();
    let raw = slip.as_raw();
    let (pw, ph) = (plan.sheet_w, plan.sheet_h);
    // The slip's columns that land on the page, and where they land.
    let from = (-plan.x).max(0).min(i64::from(sw)) as u32;
    let to = plan.x.max(0).min(i64::from(pw)) as u32;
    let copy_w = sw.saturating_sub(from).min(pw.saturating_sub(to));
    let mut sheets = Vec::with_capacity(plan.ends.len());
    let mut start = 0u32;
    for &end in &plan.ends {
        let part = end.saturating_sub(start).max(1);
        let tall = if plan.lay.landscape || plan.y > 0 { ph } else { part };
        let mut page = image::GrayImage::from_pixel(pw, tall, image::Luma([255]));
        for y in 0..part.min(tall.saturating_sub(plan.y)) {
            let sy = start + y;
            if sy >= sh {
                break;
            }
            let src = &raw[(sy * sw + from) as usize..(sy * sw + from + copy_w) as usize];
            let dst_at = ((plan.y + y) * pw + to) as usize;
            page.as_mut()[dst_at..dst_at + copy_w as usize].copy_from_slice(src);
        }
        sheets.push(if plan.lay.landscape {
            image::imageops::rotate90(&page)
        } else {
            page
        });
        start = end;
    }
    sheets
}

/// Grey of the preview's paper outline, the paper the printer cannot reach,
/// and the space between two pages (also all that is not paper).
pub const PREVIEW_EDGE: u8 = 140;
pub const PREVIEW_UNREACHABLE: u8 = 236;
pub const PREVIEW_BETWEEN: u8 = 200;
/// Black ink that does not print — past the area the printer reaches —
/// drawn half as dark ([`paste_faded`]).
pub const PREVIEW_CUT_BLACK: u8 = 128;
/// The preview picture's limits, px (a texture taller than ~8k fails on
/// some GPUs; wider than the screen is wasted).
const PREVIEW_MAX_W: u32 = 1800;
const PREVIEW_MAX_H: u32 = 8000;

/// One preview of a slip on a printer's paper, and how many of the slip's
/// own pixels each preview pixel stands for (divide the slip's type height
/// by it for the readable check).
pub struct PaperPicture {
    pub gray: image::GrayImage,
    pub per_px: f64,
}

/// The preview: the content on a printer's virtual paper as it prints.
///
/// `slip` is the upright slip picture, `dot` how many of its pixels are one
/// printer dot (it is drawn in whole blocks), `content_mm` its width on
/// paper. **OS document**: `sheet` is the printer's [`Sheet`]; the pages
/// are [`plan_sheets`]' — the print's own plan — each drawn on the virtual
/// paper: the area the printer reaches white, the rest of the paper shaded
/// (its edges, and all of it past the printer's own page), a gap between
/// pages. A roll's paper is as long as each page. **ESC/POS** (`sheet` =
/// `None`): the roll `roll_mm` wide, the picture the printer gets
/// (`escpos_picture`, turned when landscape) at its place across the roll,
/// cut at the roll's edges. Drawn at the printer's dots, enlarged in whole
/// blocks so it stays sharp.
///
/// What the print cuts off is never hidden: the rest of a slip wider than
/// the area is drawn where it would have gone, faded ([`paste_faded`]) —
/// over the shaded paper and past the paper's edge, on the grey that is not
/// paper. White paper shows only what prints.
pub fn preview_picture(
    slip: &image::GrayImage,
    dot: u32,
    content_mm: f64,
    sheet: Option<&Sheet>,
    roll_mm: f64,
    escpos_picture: Option<&image::GrayImage>,
    lay: Lay,
) -> PaperPicture {
    let dot = dot.max(1);
    let gray = |w: u32, h: u32, v: u8| image::GrayImage::from_pixel(w.max(1), h.max(1), image::Luma([v]));
    let canvas = match sheet {
        Some(sheet) => {
            let plan = plan_scaled(&plan_sheets(slip, content_mm, sheet, lay), dot);
            let small = dots_of(slip, dot);
            let pages = render_sheets(&small, &plan);
            let wholes = whole_sheets(&small, &plan);
            let ppm = plan.px_per_mm;
            let px = |mm: f64| (mm * ppm).round() as u32;
            let pw = px(sheet.width_mm).max(1);
            let [ml, mt, _, mb] = sheet.margins_mm.map(px);
            // The area as it lies on the upright paper.
            let (aw, ah) = if lay.landscape {
                (plan.sheet_h, plan.sheet_w)
            } else {
                (plan.sheet_w, plan.sheet_h)
            };
            let is_roll = sheet.length_mm <= 0.0;
            let gap = px(4.0).max(2);
            let line = px(0.35).max(1);
            // Each page's paper and the area on it: a sheet's own length; a
            // roll as long as the page, and the half millimetre a roll's
            // page has past its last line (`sheet_for`) — so the paper's
            // drawn edge never lies over the ink.
            let past = px(0.5).max(line);
            let sizes: Vec<(u32, u32)> = pages
                .iter()
                .map(|p| {
                    if is_roll {
                        (mt + p.height() + past + mb, p.height() + past)
                    } else {
                        (px(sheet.length_mm).max(1), ah)
                    }
                })
                .collect();
            // How far the slip reaches along its lines, from the paper's
            // edge: past the paper before it or after it — beside an
            // upright page, above or below a turned one. That room is
            // drawn too, never hidden.
            let along_from = i64::from(if lay.landscape { mt } else { ml }) + plan.x;
            let along_to = along_from + i64::from(small.width());
            let before = (-along_from).max(0) as u32;
            let after = |size: u32| (along_to - i64::from(size)).max(0) as u32;
            let slot = |paper_h: u32| if lay.landscape { before + paper_h + after(paper_h) } else { paper_h };
            let cw = if lay.landscape { pw } else { before + pw + after(pw) };
            let ch = sizes.iter().map(|s| slot(s.0)).sum::<u32>() + gap * (sizes.len().max(1) as u32 - 1);
            let mut c = gray(cw, ch, PREVIEW_BETWEEN);
            let mut next = 0u32;
            for (i, page) in pages.iter().enumerate() {
                let (paper_h, area_h) = sizes[i];
                let (left, top) = if lay.landscape { (0, next + before) } else { (before, next) };
                fill(&mut c, left, top, pw, paper_h, PREVIEW_UNREACHABLE);
                // The area the printer reaches is the paper's own white.
                fill(&mut c, left + ml, top + mt, aw, area_h, 255);
                // The whole slip faded, then the printed page over the
                // area: what stays faded is what the print cuts off.
                if let Some((whole, wx, wy)) = wholes.get(i) {
                    paste_faded(&mut c, whole, i64::from(left + ml) + wx, i64::from(top + mt) + wy);
                }
                paste(&mut c, page, left + ml, top + mt, left + ml + aw, top + mt + area_h);
                outline(&mut c, left, top, pw, paper_h, line);
                next += slot(paper_h) + gap;
            }
            c
        }
        None => {
            let pic = escpos_picture.unwrap_or(slip);
            let small = dots_of(pic, dot);
            let ppm = f64::from(slip.width().max(1)) / content_mm.max(1.0) / f64::from(dot);
            let rw = ((roll_mm * ppm).round() as u32).max(1);
            let (sw, rh) = (small.width(), small.height().max(1));
            // A roll is as long as the slip: only across counts.
            let (x, _) = place_in(lay.place, sw, rh, rw, rh, false);
            let before = (-x).max(0) as u32;
            let after = (x + i64::from(sw) - i64::from(rw)).max(0) as u32;
            // Past the roll is not paper; what the roll cuts off shows there.
            let mut c = gray(before + rw + after, rh, PREVIEW_BETWEEN);
            fill(&mut c, before, 0, rw, rh, 255);
            paste_faded(&mut c, &small, i64::from(before) + x, 0);
            paste(&mut c, &place_on(&small, x, rw), before, 0, before + rw, rh);
            outline(&mut c, before, 0, rw, rh, ((0.35 * ppm).round() as u32).max(1));
            c
        }
    };
    // Whole blocks up to the slip's own size while it fits; else smaller.
    let (cw, ch) = canvas.dimensions();
    let mut up = dot;
    while up > 1 && (cw * up > PREVIEW_MAX_W || ch * up > PREVIEW_MAX_H) {
        up -= 1;
    }
    if up > 1 {
        return PaperPicture { gray: blocks(&canvas, up), per_px: f64::from(dot) / f64::from(up) };
    }
    if cw > PREVIEW_MAX_W || ch > PREVIEW_MAX_H {
        let k = (f64::from(cw) / f64::from(PREVIEW_MAX_W)).max(f64::from(ch) / f64::from(PREVIEW_MAX_H));
        let (nw, nh) = (((f64::from(cw) / k) as u32).max(1), ((f64::from(ch) / k) as u32).max(1));
        let small = image::imageops::resize(&canvas, nw, nh, image::imageops::FilterType::Triangle);
        return PaperPicture { gray: small, per_px: f64::from(dot) * k };
    }
    PaperPicture { gray: canvas, per_px: f64::from(dot) }
}

/// Each page's rows of the slip at their whole width — what the page
/// shows and what it cuts off — turned like the page, with where their
/// top-left lands from the area's top-left (negative: before it).
fn whole_sheets(slip: &image::GrayImage, plan: &SheetPlan) -> Vec<(image::GrayImage, i64, i64)> {
    let (sw, sh) = slip.dimensions();
    let raw = slip.as_raw();
    let mut out = Vec::with_capacity(plan.ends.len());
    let mut start = 0u32;
    for &end in &plan.ends {
        let mut rows = end.saturating_sub(start).max(1).min(sh.saturating_sub(start));
        if plan.lay.landscape || plan.y > 0 {
            rows = rows.min(plan.sheet_h.saturating_sub(plan.y));
        }
        let rows = rows.max(1);
        let mut part = image::GrayImage::from_pixel(sw.max(1), rows, image::Luma([255]));
        let (s, n) = ((start * sw) as usize, (rows * sw) as usize);
        if s + n <= raw.len() {
            part.as_mut()[..n].copy_from_slice(&raw[s..s + n]);
        }
        out.push(if plan.lay.landscape {
            // A point (x, y) of the turned page lands at (tall - 1 - y, x).
            let tall = i64::from(plan.sheet_h);
            (image::imageops::rotate90(&part), tall - i64::from(plan.y) - i64::from(rows), plan.x)
        } else {
            (part, plan.x, i64::from(plan.y))
        });
        start = end;
    }
    out
}

/// `src` at (x, y) as ink that does not print: half as dark, over what is
/// already there, so the paper's edge and the grey past it stay visible.
fn paste_faded(c: &mut image::GrayImage, src: &image::GrayImage, x: i64, y: i64) {
    let (cw, ch) = (i64::from(c.width()), i64::from(c.height()));
    for (sx, sy, p) in src.enumerate_pixels() {
        let (cx, cy) = (x + i64::from(sx), y + i64::from(sy));
        if cx < 0 || cy < 0 || cx >= cw || cy >= ch {
            continue;
        }
        let faded = 255 - (255 - p.0[0]) / 2;
        let px = c.get_pixel_mut(cx as u32, cy as u32);
        px.0[0] = px.0[0].min(faded);
    }
}

/// A picture drawn in `dot` × `dot` blocks, back to one pixel per block.
fn dots_of(img: &image::GrayImage, dot: u32) -> image::GrayImage {
    if dot <= 1 {
        return img.clone();
    }
    let (w, h) = (img.width() as usize, img.height() as usize);
    let d = dot as usize;
    let (ow, oh) = (w.div_ceil(d).max(1), h.div_ceil(d).max(1));
    let raw = img.as_raw();
    let mut out = vec![255u8; ow * oh];
    for y in 0..oh {
        let src = &raw[(y * d).min(h.saturating_sub(1)) * w..];
        for (x, px) in out[y * ow..(y + 1) * ow].iter_mut().enumerate() {
            *px = src[(x * d).min(w.saturating_sub(1))];
        }
    }
    image::GrayImage::from_raw(ow as u32, oh as u32, out).unwrap_or_default()
}

/// Each pixel as an `up` × `up` block (sharp, like the slip's own picture).
fn blocks(img: &image::GrayImage, up: u32) -> image::GrayImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let u = up.max(1) as usize;
    let nw = w * u;
    let raw = img.as_raw();
    let mut out = vec![255u8; nw * h * u];
    for y in 0..h {
        let top = y * u * nw;
        for (x, &v) in raw[y * w..(y + 1) * w].iter().enumerate() {
            out[top + x * u..top + (x + 1) * u].fill(v);
        }
        for k in 1..u {
            out.copy_within(top..top + nw, top + k * nw);
        }
    }
    image::GrayImage::from_raw(nw as u32, (h * u) as u32, out).unwrap_or_default()
}

fn fill(c: &mut image::GrayImage, x: u32, y: u32, w: u32, h: u32, v: u8) {
    let (cw, ch) = c.dimensions();
    let (x0, x1) = (x.min(cw) as usize, (x + w).min(cw) as usize);
    if x0 >= x1 {
        return;
    }
    let stride = cw as usize;
    let buf: &mut [u8] = c;
    for yy in y..(y + h).min(ch) {
        let row = yy as usize * stride;
        buf[row + x0..row + x1].fill(v);
    }
}

/// `src` at (x, y), cut at (x_end, y_end).
fn paste(c: &mut image::GrayImage, src: &image::GrayImage, x: u32, y: u32, x_end: u32, y_end: u32) {
    let (cw, ch) = c.dimensions();
    let (x_end, y_end) = (x_end.min(cw), y_end.min(ch));
    if x >= x_end {
        return;
    }
    let n = src.width().min(x_end - x) as usize;
    let (sw, stride) = (src.width() as usize, cw as usize);
    let s = src.as_raw();
    let buf: &mut [u8] = c;
    for sy in 0..src.height() {
        let yy = y + sy;
        if yy >= y_end {
            break;
        }
        let d = yy as usize * stride + x as usize;
        let so = sy as usize * sw;
        buf[d..d + n].copy_from_slice(&s[so..so + n]);
    }
}

fn outline(c: &mut image::GrayImage, x: u32, y: u32, w: u32, h: u32, t: u32) {
    fill(c, x, y, w, t, PREVIEW_EDGE);
    fill(c, x, (y + h).saturating_sub(t), w, t, PREVIEW_EDGE);
    fill(c, x, y, t, h, PREVIEW_EDGE);
    fill(c, (x + w).saturating_sub(t), y, t, h, PREVIEW_EDGE);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up driver (no real printer's file): three sizes, one with no
    /// imageable area, HWMargins for that one.
    const PPD: &str = "*PPD-Adobe: \"4.3\"\n\
*HWMargins: 12 24 12 24\n\
*DefaultPageSize: Small\n\
*PageSize Small/Small card: \"<</PageSize[288 432]>>setpagedevice\"\n\
*PageSize Mid/Mid sheet: \"<</PageSize[420 595]>>setpagedevice\"\n\
*PageSize Strip/Strip 58: \"<</PageSize[164 800]>>setpagedevice\"\n\
*ImageableArea Small/Small card: \"18 36 270 396\"\n\
*ImageableArea Mid/Mid sheet: \"0 0 420 595\"\n\
*PaperDimension Small/Small card: \"288 432\"\n\
*PaperDimension Mid/Mid sheet: \"420 595\"\n\
*PaperDimension Strip/Strip 58: \"164 800\"\n";

    fn slip(w: u32, lines: &[(u32, u32)], h: u32) -> image::GrayImage {
        let mut img = image::GrayImage::from_pixel(w, h, image::Luma([255]));
        for &(y0, y1) in lines {
            for y in y0..y1 {
                for x in 0..w {
                    img.put_pixel(x, y, image::Luma([0]));
                }
            }
        }
        img
    }

    /// A slip white but for two marks in its inked rows: at its left end and
    /// at its right end.
    fn marked(w: u32, h: u32, ink_rows: u32) -> image::GrayImage {
        image::GrayImage::from_fn(w, h, |x, y| {
            image::Luma([if y < ink_rows && (x < 10 || x >= w - 10) { 0 } else { 255 }])
        })
    }

    fn paper(w: f64, h: f64, m: [f64; 4]) -> Paper {
        Paper { id: "P".into(), label: "P".into(), width_mm: w, height_mm: h, margins_mm: m }
    }

    /// The sheet of a printer whose own page and virtual paper are both
    /// `w` × `h`, with edges `m` it cannot reach.
    fn sheet(w: f64, h: f64, m: [f64; 4]) -> Sheet {
        sheet_for(PaperMm { width_mm: w, length_mm: h }, Some(&paper(w, h, m)), (0.0, 0.0), false)
    }

    fn turned() -> Lay {
        Lay { landscape: true, ..Lay::default() }
    }

    fn placed(down: u8, across: u8) -> Lay {
        Lay { place: Place::new(down, across), ..Lay::default() }
    }

    #[test]
    fn a_ppd_gives_every_size_its_printable_area_and_the_default() {
        let p = parse_ppd(PPD);
        assert_eq!(p.default_id, "Small");
        let ids: Vec<&str> = p.papers.iter().map(|x| x.id.as_str()).collect();
        assert_eq!(ids, ["Small", "Mid", "Strip"]);
        let small = p.by_id("Small").unwrap();
        assert_eq!(small.label, "Small card");
        assert!((small.width_mm - 101.6).abs() < 0.01, "{}", small.width_mm);
        assert!((small.height_mm - 152.4).abs() < 0.01);
        // left 18, top 432-396=36, right 288-270=18, bottom 36 (points)
        let want = [18.0, 36.0, 18.0, 36.0].map(|v: f64| v * PT_MM);
        for (a, b) in small.margins_mm.iter().zip(want) {
            assert!((a - b).abs() < 1e-6);
        }
        assert!(p.by_id("Mid").unwrap().margins_mm.iter().all(|m| *m == 0.0));
        // No imageable area → HWMargins (left bottom right top).
        let strip = p.by_id("Strip").unwrap();
        let want = [12.0, 24.0, 12.0, 24.0].map(|v: f64| v * PT_MM);
        for (a, b) in strip.margins_mm.iter().zip(want) {
            assert!((a - b).abs() < 1e-6);
        }
        // The printer's own paper is the one its driver is set to.
        assert_eq!(p.default_paper().unwrap().id, "Small");
    }

    /// Owner, 2026-11-17: a job prints on the printer's own page; the
    /// virtual paper only says how much of that page is really paper.
    #[test]
    fn a_page_is_laid_on_the_part_of_the_printers_page_that_is_paper() {
        let big = paper(210.0, 297.0, [5.0; 4]);
        // A half-size sheet in a printer set to the big page.
        let half = sheet_for(PaperMm { width_mm: 105.0, length_mm: 148.0 }, Some(&big), (80.0, 50.0), false);
        assert_eq!((half.width_mm, half.length_mm), (105.0, 148.0));
        assert_eq!(half.margins_mm, [5.0; 4], "the printer's own edges");
        assert_eq!(half.area_mm, (95.0, 138.0), "no further than the virtual paper");
        // The same sizes: the printer's whole printable area.
        let same = sheet_for(PaperMm { width_mm: 210.0, length_mm: 297.0 }, Some(&big), (80.0, 50.0), false);
        assert_eq!(same.area_mm, (200.0, 287.0));
        // A printer whose page is smaller than the virtual paper reaches
        // only its own page.
        let small = paper(148.0, 210.0, [4.0; 4]);
        let wide = sheet_for(PaperMm { width_mm: 210.0, length_mm: 297.0 }, Some(&small), (80.0, 50.0), false);
        assert_eq!(wide.area_mm, (140.0, 202.0));
        assert_eq!((wide.width_mm, wide.length_mm), (210.0, 297.0), "the paper drawn is still the virtual one");
        // A roll: as long as the slip needs, never longer than the page.
        let roll = PaperMm { width_mm: 80.0, length_mm: 0.0 };
        let short = sheet_for(roll, Some(&big), (72.0, 50.0), false);
        assert_eq!(short.area_mm, (70.0, 50.5));
        assert_eq!(short.length_mm, 0.0);
        let long = sheet_for(roll, Some(&big), (72.0, 900.0), false);
        assert_eq!(long.area_mm, (70.0, 287.0), "a longer slip goes on to the next page");
        // Across a roll (landscape) the slip's width runs along it.
        let across = sheet_for(roll, Some(&big), (72.0, 900.0), true);
        assert_eq!(across.area_mm, (70.0, 72.5));
        // The driver has not said: the virtual paper alone, edges unknown.
        let blind = sheet_for(PaperMm { width_mm: 104.8, length_mm: 241.3 }, None, (80.0, 50.0), false);
        assert_eq!(blind.margins_mm, [0.0; 4]);
        assert_eq!(blind.area_mm, (104.8, 241.3));
        assert_eq!(sheet_for(roll, None, (72.0, 300.0), false).area_mm, (80.0, 300.5));
    }

    /// The preview is the print (owner, 2026-11-16: it must never fool):
    /// its areas hold exactly the printed pages' pixels, at the printer's
    /// margins, breaking where the print breaks.
    #[test]
    fn the_preview_shows_exactly_the_printed_sheets_on_the_paper() {
        // 10 px per mm, one px per dot: 30 lines of 20 px, 10 px apart.
        let lines: Vec<(u32, u32)> = (0..30).map(|i| (i * 30, i * 30 + 20)).collect();
        let s = slip(400, &lines, 900);
        let page = sheet(50.0, 40.0, [2.0, 3.0, 4.0, 5.0]);
        let printed = os_document_sheets(&s, 40.0, &page, Lay::default()).0;
        assert!(printed.len() > 2, "a long slip goes on more sheets");
        let pic = preview_picture(&s, 1, 40.0, Some(&page), 0.0, None, Lay::default());
        assert_eq!(pic.per_px, 1.0);
        let (pw, ph, gap) = (500u32, 400u32, 40u32);
        assert_eq!(pic.gray.width(), pw);
        assert_eq!(pic.gray.height(), ph * printed.len() as u32 + gap * (printed.len() as u32 - 1));
        for (i, sheet) in printed.iter().enumerate() {
            let top = i as u32 * (ph + gap);
            for y in 0..sheet.height() {
                for x in 0..sheet.width() {
                    assert_eq!(
                        pic.gray.get_pixel(20 + x, top + 30 + y),
                        sheet.get_pixel(x, y),
                        "sheet {i} at {x},{y}"
                    );
                }
            }
            // The edge the printer cannot reach is shaded, not the slip.
            assert_eq!(pic.gray.get_pixel(10, top + 30).0[0], PREVIEW_UNREACHABLE);
        }
        // Between two sheets: the space between, not paper.
        assert_eq!(pic.gray.get_pixel(pw / 2, ph + gap / 2).0[0], PREVIEW_BETWEEN);
    }

    #[test]
    fn a_preview_in_printer_dots_is_the_same_picture_at_any_dot_size() {
        let s1 = slip(80, &[(0, 6), (10, 16)], 30);
        // The same slip drawn as 2 × 2 blocks (as the app's picture is).
        let s2 = image::GrayImage::from_fn(160, 60, |x, y| *s1.get_pixel(x / 2, y / 2));
        let page = sheet(12.0, 9.0, [1.0; 4]);
        let one = preview_picture(&s1, 1, 8.0, Some(&page), 0.0, None, Lay::default());
        let two = preview_picture(&s2, 2, 8.0, Some(&page), 0.0, None, Lay::default());
        assert_eq!(two.per_px, 1.0, "drawn back up in whole blocks");
        assert_eq!(two.gray, blocks(&one.gray, 2));
    }

    #[test]
    fn an_escpos_preview_is_the_roll_with_the_picture_from_its_left_edge() {
        // A 90 mm slip on an 80 mm roll: cut at the roll's edge, and the
        // 10 mm the roll cuts off shown past it, faded.
        let s = slip(900, &[(0, 10)], 40);
        let pic = preview_picture(&s, 1, 90.0, None, 80.0, None, Lay::default());
        assert_eq!(pic.gray.width(), 900, "the roll and what it cuts off");
        assert_eq!(pic.gray.height(), 40);
        assert_eq!(pic.gray.get_pixel(5, 5).0[0], 0, "from the left edge");
        assert_eq!(pic.gray.get_pixel(790, 5).0[0], 0, "ink up to the roll's edge");
        assert_eq!(pic.gray.get_pixel(798, 5).0[0], PREVIEW_EDGE, "the roll's own edge");
        assert_eq!(pic.gray.get_pixel(850, 5).0[0], PREVIEW_CUT_BLACK, "cut off, faded");
        assert_eq!(pic.gray.get_pixel(850, 20).0[0], PREVIEW_BETWEEN, "past the roll is not paper");
    }

    #[test]
    fn a_slip_that_fits_draws_nothing_past_the_paper() {
        let s = slip(500, &[(0, 20)], 40);
        let p = sheet(62.0, 50.0, [1.0; 4]);
        let pic = preview_picture(&s, 1, 50.0, Some(&p), 0.0, None, Lay::default());
        assert_eq!(pic.gray.dimensions(), (620, 500));
    }

    /// Owner, 2026-11-17: a 210 mm bill on 105 mm paper, drawn cut at the
    /// paper's edge with nothing past it, looked as if it fit the paper.
    #[test]
    fn a_slip_wider_than_the_paper_shows_what_the_print_cuts_off() {
        // 10 px/mm: a 100 mm slip on 62 mm paper (60 mm reachable).
        let s = slip(1000, &[(0, 20)], 40);
        let p = sheet(62.0, 50.0, [1.0; 4]);
        let pic = preview_picture(&s, 1, 100.0, Some(&p), 0.0, None, Lay::default());
        let c = &pic.gray;
        assert_eq!(c.dimensions(), (1010, 500), "the paper, and the slip's whole width past it");
        // The paper holds exactly the printed sheet.
        let (sheets, _) = os_document_sheets(&s, 100.0, &p, Lay::default());
        for y in 0..sheets[0].height() {
            for x in 0..sheets[0].width() {
                assert_eq!(c.get_pixel(10 + x, 10 + y), sheets[0].get_pixel(x, y), "({x}, {y})");
            }
        }
        assert_eq!(c.get_pixel(612, 15).0[0], PREVIEW_CUT_BLACK, "cut off, in the paper's own margin");
        assert_eq!(c.get_pixel(618, 15).0[0], PREVIEW_EDGE, "the paper's edge stays drawn");
        assert_eq!(c.get_pixel(800, 15).0[0], PREVIEW_CUT_BLACK, "cut off, past the paper");
        assert_eq!(c.get_pixel(800, 35).0[0], PREVIEW_BETWEEN, "past the paper is not paper");
    }

    #[test]
    fn a_turned_slip_wider_than_the_paper_shows_the_cut_off_part_below_it() {
        // Landscape: the slip's width runs down the paper (60 mm reachable).
        let s = slip(1000, &[(0, 20)], 40);
        let p = sheet(50.0, 62.0, [1.0; 4]);
        let pic = preview_picture(&s, 1, 100.0, Some(&p), 0.0, None, turned());
        let c = &pic.gray;
        assert_eq!(c.dimensions(), (500, 1010), "the paper, and the rest of the slip below it");
        // Slip (x, 5) lands at (10 + 479 - 5, 10 + x).
        assert_eq!(c.get_pixel(484, 110).0[0], 0, "printed");
        assert_eq!(c.get_pixel(484, 618).0[0], PREVIEW_EDGE, "the paper's bottom edge");
        assert_eq!(c.get_pixel(484, 910).0[0], PREVIEW_CUT_BLACK, "cut off, below the paper");
        assert_eq!(c.get_pixel(464, 910).0[0], PREVIEW_BETWEEN, "no ink there");
    }

    /// A printer whose own page is narrower than the virtual paper prints
    /// only on its page: the rest of the paper is shaded and the ink that
    /// would land there faded — it must not look printed.
    #[test]
    fn paper_past_the_printers_own_page_is_shaded_and_its_ink_faded() {
        // 10 px/mm: an 80 mm slip; the page 62 mm (60 reachable), the
        // virtual paper 100 mm.
        let s = slip(800, &[(0, 20)], 40);
        let own = paper(62.0, 50.0, [1.0; 4]);
        let p = sheet_for(PaperMm { width_mm: 100.0, length_mm: 50.0 }, Some(&own), (80.0, 2.0), false);
        assert_eq!(p.area_mm, (60.0, 48.0));
        let (sheets, _) = os_document_sheets(&s, 80.0, &p, Lay::default());
        assert_eq!(sheets[0].width(), 600, "cut for the printer's own page");
        let pic = preview_picture(&s, 1, 80.0, Some(&p), 0.0, None, Lay::default());
        let c = &pic.gray;
        assert_eq!(c.dimensions(), (1000, 500), "the virtual paper");
        assert_eq!(c.get_pixel(10 + 300, 15).0[0], 0, "printed");
        assert_eq!(c.get_pixel(10 + 700, 15).0[0], PREVIEW_CUT_BLACK, "on paper the printer cannot reach");
        assert_eq!(c.get_pixel(10 + 700, 35).0[0], PREVIEW_UNREACHABLE);
        assert_eq!(c.get_pixel(900, 15).0[0], PREVIEW_UNREACHABLE, "paper, but never printed");
        assert_eq!(c.get_pixel(10 + 300, 35).0[0], 255, "paper the printer reaches");
    }

    /// A half-size sheet in a printer set to a bigger page: pages break at
    /// the real sheet's length, so every sheet gets a whole page and no
    /// lines are lost off its end.
    #[test]
    fn a_shorter_virtual_sheet_breaks_pages_at_its_own_length() {
        // 10 px/mm: 60 lines of 20 px, 10 px apart = 180 mm of slip.
        let lines: Vec<(u32, u32)> = (0..60).map(|i| (i * 30, i * 30 + 20)).collect();
        let s = slip(800, &lines, 1800);
        let big = paper(210.0, 297.0, [4.0; 4]);
        let half = sheet_for(PaperMm { width_mm: 105.0, length_mm: 148.0 }, Some(&big), (80.0, 180.0), false);
        let (sheets, _) = os_document_sheets(&s, 80.0, &half, Lay::default());
        assert!(sheets.len() >= 2, "one big page would lose the lines past the real sheet");
        let mut total = 0;
        for sheet in &sheets {
            assert!(sheet.height() <= 1400, "fits the real sheet: 148 - 4 - 4 mm");
            total += sheet.height();
        }
        assert_eq!(total, 60 * 30 - 10, "every row printed once");
        // On the big page itself the whole slip is one page.
        let whole = sheet_for(PaperMm { width_mm: 210.0, length_mm: 297.0 }, Some(&big), (80.0, 180.0), false);
        assert_eq!(os_document_sheets(&s, 80.0, &whole, Lay::default()).0.len(), 1);
    }

    #[test]
    fn the_nine_places_are_the_papers_own_sides() {
        // A 40 × 50 slip in a 100 × 200 area.
        let at = |down, across| place_in(Place::new(down, across), 40, 50, 100, 200, true);
        assert_eq!(at(0, 0), (0, 0), "top left: where content always sat");
        assert_eq!(at(0, 1), (30, 0), "top centre");
        assert_eq!(at(0, 2), (60, 0), "top right");
        assert_eq!(at(1, 0), (0, 75), "middle left");
        assert_eq!(at(2, 2), (60, 150), "bottom right");
        // Down only on one sheet with a length that holds the slip.
        assert_eq!(place_in(Place::new(2, 0), 40, 50, 100, 200, false), (0, 0));
        assert_eq!(place_in(Place::new(2, 0), 40, 250, 100, 200, true), (0, 0));
        // Wider than the area: it hangs over by the same rule.
        assert_eq!(place_in(Place::new(0, 0), 150, 50, 100, 200, true), (0, 0), "left: cut at the right");
        assert_eq!(place_in(Place::new(0, 1), 150, 50, 100, 200, true), (-25, 0), "centre: cut at both");
        assert_eq!(place_in(Place::new(0, 2), 150, 50, 100, 200, true), (-50, 0), "right: cut at the left");
        assert!(Place::default().is_top_left());
    }

    #[test]
    fn a_placed_slip_prints_there_and_the_preview_shows_it_there() {
        // 10 px/mm: a 40 mm slip (20 rows of ink) on 62 × 50 mm paper.
        let s = slip(400, &[(0, 20)], 40);
        let p = sheet(62.0, 50.0, [1.0; 4]);
        let (sheets, _) = os_document_sheets(&s, 40.0, &p, placed(1, 1));
        assert_eq!(sheets.len(), 1);
        // Placed lower than the top: the page is the whole area.
        assert_eq!(sheets[0].dimensions(), (600, 480));
        // Centre: (600 - 400) / 2; middle: (480 - 20) / 2.
        assert_eq!(sheets[0].get_pixel(100, 230).0[0], 0);
        assert_eq!(sheets[0].get_pixel(99, 230).0[0], 255);
        assert_eq!(sheets[0].get_pixel(100, 229).0[0], 255);
        assert_eq!(sheets[0].get_pixel(499, 249).0[0], 0);
        assert_eq!(sheets[0].get_pixel(500, 249).0[0], 255);
        let pic = preview_picture(&s, 1, 40.0, Some(&p), 0.0, None, placed(1, 1));
        assert_eq!(pic.gray.dimensions(), (620, 500), "it fits: nothing past the paper");
        for (x, y) in [(0, 0), (100, 230), (499, 249), (300, 470)] {
            assert_eq!(pic.gray.get_pixel(10 + x, 10 + y), sheets[0].get_pixel(x, y), "({x}, {y})");
        }
        // Top-left is the page as it always was.
        let (top, _) = os_document_sheets(&s, 40.0, &p, Lay::default());
        assert_eq!(top[0].dimensions(), (600, 20));
        assert_eq!(top[0].get_pixel(0, 0).0[0], 0);
    }

    /// The place is measured on the real sheet, not on the bigger page the
    /// printer is set to: centre is the centre of the half-size sheet.
    #[test]
    fn a_place_is_measured_on_the_virtual_paper() {
        let s = slip(400, &[(0, 20)], 40);
        let big = paper(210.0, 297.0, [5.0; 4]);
        let half = sheet_for(PaperMm { width_mm: 105.0, length_mm: 148.0 }, Some(&big), (40.0, 2.0), false);
        let (sheets, _) = os_document_sheets(&s, 40.0, &half, placed(1, 1));
        // The area: 95 × 138 mm. Centre: (950 - 400) / 2 = 275; middle:
        // (1380 - 20) / 2 = 680.
        assert_eq!(sheets[0].dimensions(), (950, 1380));
        assert_eq!(sheets[0].get_pixel(275, 680).0[0], 0);
        assert_eq!(sheets[0].get_pixel(274, 680).0[0], 255);
        assert_eq!(sheets[0].get_pixel(275, 679).0[0], 255);
    }

    #[test]
    fn a_roll_or_a_long_slip_starts_at_the_top_whatever_the_place() {
        let s = slip(400, &[(0, 20)], 40);
        let own = paper(62.0, 50.0, [1.0; 4]);
        let roll = sheet_for(PaperMm { width_mm: 62.0, length_mm: 0.0 }, Some(&own), (40.0, 2.0), false);
        let (sheets, _) = os_document_sheets(&s, 40.0, &roll, placed(2, 1));
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheets[0].height(), 20, "as long as the slip");
        assert_eq!(sheets[0].get_pixel(100, 0).0[0], 0, "centred across, at the top");
        // The roll's paper in the preview is as long as its page.
        let pic = preview_picture(&s, 1, 40.0, Some(&roll), 0.0, None, placed(2, 1));
        assert_eq!(pic.gray.dimensions(), (620, 10 + 20 + 5 + 10), "and the half millimetre past its last line");
        // 30 lines on 25 mm sheets: more than one sheet starts at the top.
        let lines: Vec<(u32, u32)> = (0..30).map(|i| (i * 30, i * 30 + 20)).collect();
        let long = slip(100, &lines, 900);
        let small = sheet(12.0, 27.0, [1.0; 4]);
        let (sheets, _) = os_document_sheets(&long, 10.0, &small, placed(2, 0));
        assert!(sheets.len() > 1);
        assert_eq!(sheets[0].get_pixel(0, 0).0[0], 0);
    }

    /// A roll longer than the printer's own page goes on to its next page,
    /// as any document does; the preview shows each page on its own paper.
    #[test]
    fn a_roll_slip_longer_than_the_printers_page_goes_on_more_pages() {
        let lines: Vec<(u32, u32)> = (0..30).map(|i| (i * 30, i * 30 + 20)).collect();
        let s = slip(100, &lines, 900);
        let own = paper(12.0, 27.0, [1.0; 4]);
        let roll = sheet_for(PaperMm { width_mm: 12.0, length_mm: 0.0 }, Some(&own), (10.0, 89.0), false);
        assert_eq!(roll.area_mm, (10.0, 25.0));
        let (sheets, _) = os_document_sheets(&s, 10.0, &roll, Lay::default());
        assert!(sheets.len() >= 4);
        assert!(sheets.iter().all(|p| p.height() <= 250));
        assert_eq!(sheets.iter().map(|p| p.height()).sum::<u32>(), 30 * 30 - 10);
        let pic = preview_picture(&s, 1, 10.0, Some(&roll), 0.0, None, Lay::default());
        let papers: u32 = sheets.iter().map(|p| 10 + p.height() + 5 + 10).sum();
        assert_eq!(pic.gray.height(), papers + 40 * (sheets.len() as u32 - 1));
    }

    #[test]
    fn a_right_placed_slip_wider_than_the_paper_keeps_its_right_end() {
        // 10 px/mm: a 100 mm slip on 62 mm paper (60 mm reachable).
        let s = marked(1000, 40, 20);
        let p = sheet(62.0, 50.0, [1.0; 4]);
        let (sheets, _) = os_document_sheets(&s, 100.0, &p, placed(0, 2));
        assert_eq!(sheets[0].get_pixel(595, 5).0[0], 0, "its right end prints");
        assert_eq!(sheets[0].get_pixel(5, 5).0[0], 255, "its left end is cut off");
        let pic = preview_picture(&s, 1, 100.0, Some(&p), 0.0, None, placed(0, 2));
        // 400 px cut off before the area: 390 of them past the paper.
        assert_eq!(pic.gray.dimensions(), (1010, 500));
        assert_eq!(pic.gray.get_pixel(5, 15).0[0], PREVIEW_CUT_BLACK, "cut off, before the paper");
        assert_eq!(pic.gray.get_pixel(390 + 10 + 595, 15).0[0], 0, "printed");
        // Top-left (the default) keeps its left end, as it always did.
        let (kept, _) = os_document_sheets(&s, 100.0, &p, Lay::default());
        assert_eq!(kept[0].get_pixel(5, 5).0[0], 0);
    }

    #[test]
    fn an_escpos_slip_narrower_than_the_roll_goes_where_it_is_placed() {
        // 10 px/mm: a 58 mm slip on an 80 mm roll, centred across.
        let s = slip(580, &[(0, 10)], 40);
        let pic = preview_picture(&s, 1, 58.0, None, 80.0, None, placed(2, 1));
        assert_eq!(pic.gray.dimensions(), (800, 40), "the roll, nothing past it");
        assert_eq!(pic.gray.get_pixel(50, 5).0[0], 255, "blank roll before the slip");
        assert_eq!(pic.gray.get_pixel(150, 5).0[0], 0, "the slip from (800 - 580) / 2");
        assert_eq!(pic.gray.get_pixel(750, 5).0[0], 255, "blank roll after it");
    }

    #[test]
    fn an_lpoptions_default_names_a_size_by_choice_or_by_its_size() {
        let p = parse_ppd(PPD);
        assert_eq!(lpoptions_media("copies=1 PageSize=Mid sides=one-sided").as_deref(), Some("Mid"));
        assert_eq!(resolve_media_name(&p, "Mid").as_deref(), Some("Mid"));
        // 288 × 432 pt = 4 × 6 in
        assert_eq!(resolve_media_name(&p, "na_index-4x6_4x6in").as_deref(), Some("Small"));
        assert_eq!(
            resolve_media_name(&p, "custom_x_148.17x209.9mm").as_deref(),
            Some("Mid")
        );
        assert_eq!(resolve_media_name(&p, "iso_a3_297x420mm"), None);
    }

    #[test]
    fn a_sheet_is_the_printable_width_at_the_slips_own_size_on_any_paper() {
        // 10 px per mm: an 80 mm slip, 400 px long.
        let s = slip(800, &[(0, 30), (50, 80)], 400);
        for (w, h, m) in [
            (101.6, 152.4, [6.35, 12.7, 6.35, 12.7]),
            (80.0, 297.0, [2.0, 2.0, 2.0, 2.0]),
            (148.0, 210.0, [5.0, 5.0, 5.0, 5.0]),
            (297.0, 420.0, [4.0, 4.0, 4.0, 4.0]),
            (57.0, 1000.0, [0.0; 4]),
        ] {
            let p = sheet(w, h, m);
            let (sheets, ppm) = os_document_sheets(&s, 80.0, &p, Lay::default());
            assert!((ppm - 10.0).abs() < 1e-9);
            let (pw, _) = paper(w, h, m).printable_mm();
            assert_eq!(sheets.len(), 1, "{w}×{h}");
            assert_eq!(sheets[0].width(), (pw * 10.0).floor() as u32, "{w}×{h}");
            // Real size: the first line starts at the sheet's top-left.
            assert_eq!(sheets[0].get_pixel(0, 0).0[0], 0);
            // Printed length is the slip's ink, not the paper.
            assert_eq!(sheets[0].height(), 80);
            // Narrower paper cuts the slip at the printable edge.
            let ink_w = (0..sheets[0].width())
                .filter(|&x| sheets[0].get_pixel(x, 10).0[0] == 0)
                .count() as u32;
            assert_eq!(ink_w, 800.min(sheets[0].width()), "{w}×{h}");
        }
    }

    #[test]
    fn a_long_slip_goes_on_to_more_sheets_between_lines() {
        // 30 lines of 20 px with 10 px gaps: 900 px; a sheet holds 250 px.
        let lines: Vec<(u32, u32)> = (0..30).map(|i| (i * 30, i * 30 + 20)).collect();
        let s = slip(100, &lines, 900);
        let p = sheet(10.0 + 2.0, 25.0 + 2.0, [1.0; 4]);
        let (sheets, _) = os_document_sheets(&s, 10.0, &p, Lay::default());
        assert!(sheets.len() >= 4);
        let mut total = 0;
        for (i, sheet) in sheets.iter().enumerate() {
            assert!(sheet.height() <= 250);
            total += sheet.height();
            if i + 1 < sheets.len() {
                // Ends in a blank band: the last row has no ink.
                let last = sheet.height() - 1;
                assert!((0..sheet.width()).all(|x| sheet.get_pixel(x, last).0[0] == 255));
            }
        }
        // Every row of the slip (to its last ink) is printed once.
        assert_eq!(total, 30 * 30 - 10);
    }

    #[test]
    fn a_picture_taller_than_half_a_sheet_is_cut_at_the_sheet() {
        let s = slip(50, &[(0, 900)], 900);
        let ends = page_ends(900, 250, &line_gaps(s.as_raw(), 50, 900));
        assert_eq!(ends, [250, 500, 750, 900]);
    }

    /// The same numbers as patches/makepad/KumkajSlipLayoutTest.java: the
    /// Android print dialog breaks a slip's pages exactly where the desktop
    /// does.
    #[test]
    fn page_breaks_match_the_android_layout() {
        // 30 lines of 20 px, 10 px apart, on pages that hold 250 px: each
        // blank band's middle row is a place a page may end.
        let gaps: Vec<u32> = (0..29).map(|i| i * 30 + 25).collect();
        assert_eq!(page_ends(890, 250, &gaps), [235, 475, 715, 890]);
        // No blank band: cut at the page.
        assert_eq!(page_ends(900, 250, &[]), [250, 500, 750, 900]);
        // Places, as KumkajSlipLayout.plan computes them.
        assert_eq!(place_in(Place::new(1, 1), 400, 20, 950, 1380, true), (275, 680));
        assert_eq!(place_in(Place::new(0, 2), 150, 50, 100, 200, true), (-50, 0));
    }

    #[test]
    fn a_landscape_slip_fills_the_turned_printable_area_from_its_corner() {
        // 10 px/mm, slip 40 mm wide × 10 mm of ink.
        let s = slip(400, &[(0, 100)], 100);
        let m = [3.0, 4.0, 5.0, 6.0];
        let p = sheet(100.0, 150.0, m);
        let (sheets, _) = os_document_sheets(&s, 40.0, &p, turned());
        assert_eq!(sheets.len(), 1);
        let (pw, ph) = paper(100.0, 150.0, m).printable_mm();
        // Upright sheet = the printable area.
        assert_eq!(sheets[0].width(), (pw * 10.0).floor() as u32);
        assert_eq!(sheets[0].height(), (ph * 10.0).floor() as u32);
        // Laid at the turned top-left = the upright top-right.
        let w = sheets[0].width();
        assert_eq!(sheets[0].get_pixel(w - 1, 0).0[0], 0);
        assert_eq!(sheets[0].get_pixel(0, 0).0[0], 255);
    }

    #[test]
    fn mm_reads_whole_or_with_one_decimal() {
        assert_eq!(mm_text(148.0), "148");
        assert_eq!(mm_text(101.6), "101.6");
        assert_eq!(mm_text(88.89999), "88.9");
    }
}

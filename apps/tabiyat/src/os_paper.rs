//! The paper a job prints on, and the preview of the content on it
//! (docs/PRINTER.md § Paper and preview). Nothing in here knows a size: no
//! A4, no Letter, no 210 mm. Each printer holds the paper set for it on
//! Settings → Printers ([`PaperMm`]: width, and length or a roll); Find
//! fills a blank one from the driver ([`query`]) — the driver also says how
//! much of a sheet the printer can reach and which sizes it has names for.
//!
//! An OS document is the slip picture at its real size — its px are 1/96 inch
//! (rule 2) — on a page exactly the printer's paper ([`page_for`]), at the
//! top-left of its printable area. Nothing is fitted, shrunk or centred: a
//! bill longer than the sheet goes on to the next sheet, split between lines;
//! a bill wider than the printable area is cut at its edge, the same as a
//! column past the content width. The preview ([`preview_picture`]) is drawn
//! from the same sheet plan, so it is the print.

use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::time::Duration;

/// One paper size a printer's driver offers.
#[derive(Clone, Debug, PartialEq)]
pub struct Paper {
    /// The driver's own name for it (CUPS `PageSize` choice, Windows
    /// `PaperName`). Sent back with every job so the driver uses it.
    pub id: String,
    /// The driver's name for people ("4 x 6 in", "A5", …).
    pub label: String,
    pub width_mm: f64,
    pub height_mm: f64,
    /// Edges the printer cannot reach, mm: left, top, right, bottom.
    pub margins_mm: [f64; 4],
}

/// PostScript points (PPD sizes) to mm.
const PT_MM: f64 = 25.4 / 72.0;
/// Windows sizes are in hundredths of an inch.
const HUNDREDTH_IN_MM: f64 = 0.254;

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

/// The sizes one printer's driver offers and the one it uses now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrinterPapers {
    /// The driver's current paper (`id` of one of `papers`).
    pub default_id: String,
    pub papers: Vec<Paper>,
    /// Edges the printer cannot reach on a size the driver has no name for
    /// (PPD `*HWMargins`), mm: left, top, right, bottom.
    pub custom_margins_mm: Option<[f64; 4]>,
    /// Whether the driver takes a size it has no name for (PPD
    /// `*CustomPageSize True`). Without it the page is the driver's own
    /// paper (`page_for`), in the print and the preview alike.
    pub custom_ok: bool,
}

impl PrinterPapers {
    pub fn by_id(&self, id: &str) -> Option<&Paper> {
        let id = id.trim();
        if id.is_empty() {
            return None;
        }
        self.papers.iter().find(|p| p.id == id)
    }

    /// The driver's current paper.
    pub fn default_paper(&self) -> Option<&Paper> {
        self.by_id(&self.default_id).or_else(|| self.papers.first())
    }

    /// The driver's own size for `width` × `length` mm (within 0.6 mm).
    pub fn matching(&self, width_mm: f64, length_mm: f64) -> Option<&Paper> {
        self.papers
            .iter()
            .find(|p| (p.width_mm - width_mm).abs() < 0.6 && (p.height_mm - length_mm).abs() < 0.6)
    }

    /// Margins for a size the driver has no name for: its `*HWMargins`, else
    /// those of its current paper (the same printer reaches the same edges).
    pub fn margins_for_custom(&self) -> [f64; 4] {
        self.custom_margins_mm
            .or_else(|| self.default_paper().map(|p| p.margins_mm))
            .unwrap_or([0.0; 4])
    }
}

/// The paper set for one printer (Settings → Printers): width, and the
/// sheet's length, mm; a length of 0 is a roll.
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

/// The page one job is sent on: the name the system gets for it and the
/// page with the edges the printer cannot reach.
#[derive(Clone, Debug, PartialEq)]
pub struct PagePlan {
    /// The driver's own size name, or `Custom.<w>x<l>mm` (CUPS).
    pub media: String,
    /// Whether `media` is the driver's own size.
    pub driver_size: bool,
    pub page: Paper,
    /// The driver has no size like the printer's paper and takes no other
    /// size: `page` is the driver's own paper instead (the job says so).
    pub fallback: bool,
}

fn mm_num(mm: f64) -> String {
    let s = format!("{:.2}", (mm * 100.0).round() / 100.0);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The page for a slip on a printer's `paper`: exactly that paper — the
/// driver's own size when one matches, else a custom size with the
/// printer's margins. A roll is one page as long as the slip (`slip_mm` =
/// its width and inked length, mm, upright) plus the margins; a landscape
/// slip lies across it, so the roll is then as long as the slip is wide.
/// `driver` is `None` when the driver could not be read: margins unknown (0).
pub fn page_for(
    paper: PaperMm,
    driver: Option<&PrinterPapers>,
    slip_mm: (f64, f64),
    landscape: bool,
) -> PagePlan {
    let width = paper.width_mm;
    if !paper.is_roll() {
        if let Some(own) = driver.and_then(|d| d.matching(width, paper.length_mm)) {
            return PagePlan { media: own.id.clone(), driver_size: true, page: own.clone(), fallback: false };
        }
    }
    // A driver that takes no size it has no name for prints on its own
    // paper whatever is asked: plan that page, so the preview shows it too.
    if let Some(own) = driver.filter(|d| !d.custom_ok).and_then(|d| d.default_paper()) {
        return PagePlan { media: own.id.clone(), driver_size: true, page: own.clone(), fallback: true };
    }
    let margins = driver.map(|d| d.margins_for_custom()).unwrap_or([0.0; 4]);
    let length = if paper.is_roll() {
        let along = if landscape { slip_mm.0 } else { slip_mm.1 };
        // Half a millimetre more than the slip, so rounding to whole rows
        // never leaves its last line for a second page.
        along + margins[1] + margins[3] + 0.5
    } else {
        paper.length_mm
    };
    let media = format!("Custom.{}x{}mm", mm_num(width), mm_num(length));
    PagePlan {
        media: media.clone(),
        driver_size: false,
        fallback: false,
        page: Paper {
            id: media,
            label: String::new(),
            width_mm: width,
            height_mm: length,
            margins_mm: margins,
        },
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
    let custom_margins_mm = hw.map(|[l, b, r, t]| [l, t, r, b].map(|m| (m * PT_MM).max(0.0)));
    let custom_ok = ppd.lines().any(|l| l.trim_start().starts_with("*CustomPageSize True"));
    PrinterPapers { default_id, papers, custom_margins_mm, custom_ok }
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

// ---------- Windows: the driver's PaperSizes ----------

/// PowerShell that prints the driver's paper sizes, one per line:
/// `D<TAB>default name` then `P<TAB>name<TAB>w<TAB>h<TAB>x<TAB>y<TAB>pw<TAB>ph`
/// (hundredths of an inch; x, y, pw, ph = that paper's printable area).
#[cfg(any(target_os = "windows", test))]
pub fn windows_papers_script(printer: Option<&str>) -> String {
    let set_name = match printer.map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) => format!("$ps.PrinterName = '{}';", n.replace('\'', "''")),
        None => String::new(),
    };
    format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
         Add-Type -AssemblyName System.Drawing; \
         $ps = New-Object System.Drawing.Printing.PrinterSettings; {set_name} \
         if (-not $ps.IsValid) {{ exit 3 }}; \
         $t = [char]9; \
         'D' + $t + $ps.DefaultPageSettings.PaperSize.PaperName; \
         if (@($ps.PaperSizes | Where-Object {{ $_.Kind -eq 'Custom' }}).Count -gt 0) {{ 'C' + $t + '1' }}; \
         foreach ($p in $ps.PaperSizes) {{ \
           $pg = New-Object System.Drawing.Printing.PageSettings($ps); \
           $pg.Landscape = $false; $pg.PaperSize = $p; \
           try {{ $a = $pg.PrintableArea }} catch {{ $a = New-Object System.Drawing.RectangleF(0,0,$p.Width,$p.Height) }}; \
           'P' + $t + $p.PaperName + $t + $p.Width + $t + $p.Height + $t + $a.X + $t + $a.Y + $t + $a.Width + $t + $a.Height \
         }}"
    )
}

/// Read [`windows_papers_script`]'s lines.
pub fn parse_windows_papers(out: &str) -> PrinterPapers {
    let mut default_id = String::new();
    let mut custom_ok = false;
    let mut papers: Vec<Paper> = Vec::new();
    for line in out.lines() {
        let f: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
        match f.first().copied() {
            Some("D") if f.len() >= 2 => default_id = f[1].trim().to_string(),
            // The driver lists a user-defined ("Custom") size.
            Some("C") => custom_ok = true,
            Some("P") if f.len() >= 8 => {
                let n: Vec<f64> = f[2..8]
                    .iter()
                    .map(|v| v.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0))
                    .collect();
                let (w, h, x, y, pw, ph) = (n[0], n[1], n[2], n[3], n[4], n[5]);
                let name = f[1].trim();
                if name.is_empty() || w <= 0.0 || h <= 0.0 || papers.iter().any(|p| p.id == name) {
                    continue;
                }
                let (pw, ph) = if pw > 0.0 && ph > 0.0 { (pw, ph) } else { (w, h) };
                let margins = [x, y, (w - x - pw).max(0.0), (h - y - ph).max(0.0)];
                papers.push(Paper {
                    id: name.to_string(),
                    label: name.to_string(),
                    width_mm: w * HUNDREDTH_IN_MM,
                    height_mm: h * HUNDREDTH_IN_MM,
                    margins_mm: margins.map(|m| m.max(0.0) * HUNDREDTH_IN_MM),
                });
            }
            _ => {}
        }
    }
    PrinterPapers { default_id, papers, custom_margins_mm: None, custom_ok }
}

#[cfg(target_os = "windows")]
pub fn query_windows(printer: Option<&str>) -> Result<PrinterPapers, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &windows_papers_script(printer),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("PowerShell did not start ({e})"))?;
    if !out.status.success() {
        return Err("Windows did not give this printer's paper sizes".into());
    }
    let papers = parse_windows_papers(&String::from_utf8_lossy(&out.stdout));
    if papers.papers.is_empty() {
        return Err("the printer's driver lists no paper sizes".into());
    }
    Ok(papers)
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
        query_windows(name)
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

/// The slip on `paper`, one grey picture per sheet, each exactly the
/// printable area's width at the slip's own density (`content_mm` is the
/// slip's width on paper), the slip at its top-left. Portrait sheets are as
/// long as their part of the slip; a landscape slip is laid on the turned
/// sheet and turned back, filling the printable area so it lands where it
/// was laid. Returns the sheets and the slip's px per mm.
pub fn os_document_sheets(
    slip: &image::GrayImage,
    content_mm: f64,
    paper: &Paper,
    landscape: bool,
) -> (Vec<image::GrayImage>, f64) {
    let plan = plan_sheets(slip, content_mm, paper, landscape);
    (render_sheets(slip, &plan), plan.px_per_mm)
}

/// Where a slip's sheets end on `paper` and how big each sheet is, in the
/// slip's own pixels — the print's geometry, shared with the preview.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetPlan {
    pub px_per_mm: f64,
    /// Printable width and height of the (turned, for landscape) sheet, px.
    pub sheet_w: u32,
    pub sheet_h: u32,
    /// Row (in the slip) after the last row of each sheet.
    pub ends: Vec<u32>,
    pub landscape: bool,
}

pub fn plan_sheets(slip: &image::GrayImage, content_mm: f64, paper: &Paper, landscape: bool) -> SheetPlan {
    let (sw, sh) = slip.dimensions();
    let px_per_mm = f64::from(sw.max(1)) / content_mm.max(1.0);
    let sheet = if landscape { paper.turned() } else { paper.clone() };
    let (pw_mm, ph_mm) = sheet.printable_mm();
    // Floor: a sheet never reaches past the printable edge.
    let sheet_w = ((pw_mm * px_per_mm).floor() as u32).max(1);
    let sheet_h = ((ph_mm * px_per_mm).floor() as u32).max(1);
    let raw = slip.as_raw();
    let len = ink_end(raw, sw as usize, sh as usize).max(1);
    let gaps = line_gaps(raw, sw as usize, sh as usize);
    SheetPlan { px_per_mm, sheet_w, sheet_h, ends: page_ends(len, sheet_h, &gaps), landscape }
}

/// The plan for the same slip drawn `1 / div` the size (its dots, for the
/// preview): the same sheets breaking at the same lines.
fn plan_scaled(plan: &SheetPlan, paper: &Paper, div: u32) -> SheetPlan {
    let div = div.max(1);
    let px_per_mm = plan.px_per_mm / f64::from(div);
    let sheet = if plan.landscape { paper.turned() } else { paper.clone() };
    let (pw_mm, ph_mm) = sheet.printable_mm();
    SheetPlan {
        px_per_mm,
        sheet_w: ((pw_mm * px_per_mm).floor() as u32).max(1),
        sheet_h: ((ph_mm * px_per_mm).floor() as u32).max(1),
        ends: plan.ends.iter().map(|e| e.div_ceil(div)).collect(),
        landscape: plan.landscape,
    }
}

/// The sheets of `plan`, one grey picture each, exactly the printable width
/// (the slip at its top-left, cut at its edge). Portrait sheets are as long
/// as their part; a landscape slip is laid on the turned sheet and turned
/// back, filling the printable area so it lands where it was laid.
pub fn render_sheets(slip: &image::GrayImage, plan: &SheetPlan) -> Vec<image::GrayImage> {
    let (sw, sh) = slip.dimensions();
    let raw = slip.as_raw();
    let (pw, ph) = (plan.sheet_w, plan.sheet_h);
    let copy_w = sw.min(pw);
    let mut sheets = Vec::with_capacity(plan.ends.len());
    let mut start = 0u32;
    for &end in &plan.ends {
        let part = end.saturating_sub(start).max(1);
        let tall = if plan.landscape { ph } else { part };
        let mut page = image::GrayImage::from_pixel(pw, tall, image::Luma([255]));
        for y in 0..part.min(tall) {
            let sy = start + y;
            if sy >= sh {
                break;
            }
            let src = &raw[(sy * sw) as usize..(sy * sw + copy_w) as usize];
            let dst_at = (y * pw) as usize;
            page.as_mut()[dst_at..dst_at + copy_w as usize].copy_from_slice(src);
        }
        sheets.push(if plan.landscape {
            image::imageops::rotate90(&page)
        } else {
            page
        });
        start = end;
    }
    sheets
}

/// Grey of the preview's paper outline, the edges the printer cannot reach,
/// and the space between two sheets.
pub const PREVIEW_EDGE: u8 = 140;
pub const PREVIEW_UNREACHABLE: u8 = 236;
pub const PREVIEW_BETWEEN: u8 = 200;
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

/// The preview: the content on a printer's paper as it prints.
///
/// `slip` is the upright slip picture, `dot` how many of its pixels are one
/// printer dot (it is drawn in whole blocks), `content_mm` its width on
/// paper. **OS document**: `page` is [`page_for`]'s page; the sheets are
/// [`plan_sheets`]' — the print's own plan — each drawn on the whole paper
/// at the printable top-left, the edges it cannot reach shaded, a gap
/// between sheets. **ESC/POS** (`page` = `None`): the roll `roll_mm` wide,
/// the picture the printer gets (`escpos_picture`, turned when landscape)
/// from its left edge, cut at the roll's edge. Drawn at the printer's dots,
/// enlarged in whole blocks so it stays sharp.
pub fn preview_picture(
    slip: &image::GrayImage,
    dot: u32,
    content_mm: f64,
    page: Option<&Paper>,
    landscape: bool,
    roll_mm: f64,
    escpos_picture: Option<&image::GrayImage>,
) -> PaperPicture {
    let dot = dot.max(1);
    let gray = |w: u32, h: u32, v: u8| image::GrayImage::from_pixel(w.max(1), h.max(1), image::Luma([v]));
    let canvas = match page {
        Some(page) => {
            let full = plan_sheets(slip, content_mm, page, landscape);
            let plan = plan_scaled(&full, page, dot);
            let small = dots_of(slip, dot);
            let sheets = render_sheets(&small, &plan);
            let ppm = plan.px_per_mm;
            let pw = ((page.width_mm * ppm).round() as u32).max(1);
            let ph = ((page.height_mm * ppm).round() as u32).max(1);
            let gap = ((4.0 * ppm).round() as u32).max(2);
            let n = sheets.len().max(1) as u32;
            let mut c = gray(pw, ph * n + gap * (n - 1), PREVIEW_BETWEEN);
            let [ml, mt, mr, mb] = page.margins_mm.map(|m| (m * ppm).round() as u32);
            for (i, sheet) in sheets.iter().enumerate() {
                let top = i as u32 * (ph + gap);
                fill(&mut c, 0, top, pw, ph, PREVIEW_UNREACHABLE);
                // The printable area is the paper's own white.
                let (aw, ah) = (pw.saturating_sub(ml + mr), ph.saturating_sub(mt + mb));
                fill(&mut c, ml, top + mt, aw, ah, 255);
                paste(&mut c, sheet, ml, top + mt, ml + aw, top + mt + ah);
                outline(&mut c, 0, top, pw, ph, ((0.35 * ppm).round() as u32).max(1));
            }
            c
        }
        None => {
            let pic = escpos_picture.unwrap_or(slip);
            let small = dots_of(pic, dot);
            let ppm = f64::from(slip.width().max(1)) / content_mm.max(1.0) / f64::from(dot);
            let rw = ((roll_mm * ppm).round() as u32).max(1);
            let rh = small.height().max(1);
            let mut c = gray(rw, rh, 255);
            paste(&mut c, &small, 0, 0, rw, rh);
            outline(&mut c, 0, 0, rw, rh, ((0.35 * ppm).round() as u32).max(1));
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
*CustomPageSize True: \"pop pop pop pop pop\"\n\
*DefaultPageSize: Small\n\
*PageSize Small/Small card: \"<</PageSize[288 432]>>setpagedevice\"\n\
*PageSize Mid/Mid sheet: \"<</PageSize[420 595]>>setpagedevice\"\n\
*PageSize Strip/Strip 58: \"<</PageSize[164 800]>>setpagedevice\"\n\
*ImageableArea Small/Small card: \"18 36 270 396\"\n\
*ImageableArea Mid/Mid sheet: \"0 0 420 595\"\n\
*PaperDimension Small/Small card: \"288 432\"\n\
*PaperDimension Mid/Mid sheet: \"420 595\"\n\
*PaperDimension Strip/Strip 58: \"164 800\"\n";

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
    }

    #[test]
    fn the_page_is_the_printers_paper_named_by_the_driver_when_it_can() {
        let p = parse_ppd(PPD);
        // 288 × 432 pt = 101.6 × 152.4 mm: the driver's own "Small".
        let own = page_for(PaperMm { width_mm: 101.6, length_mm: 152.4 }, Some(&p), (80.0, 50.0), false);
        assert!(own.driver_size);
        assert_eq!(own.media, "Small");
        assert_eq!(own.page.margins_mm, p.by_id("Small").unwrap().margins_mm);
        // No size of that name: a custom page with the driver's HWMargins.
        let custom = page_for(PaperMm { width_mm: 104.8, length_mm: 241.3 }, Some(&p), (80.0, 50.0), false);
        assert!(!custom.driver_size);
        assert_eq!(custom.media, "Custom.104.8x241.3mm");
        let hw = [12.0, 24.0, 12.0, 24.0].map(|v: f64| v * PT_MM);
        for (a, b) in custom.page.margins_mm.iter().zip(hw) {
            assert!((a - b).abs() < 1e-9);
        }
        // A roll: one page as long as the slip and the edges it cannot reach.
        let roll = page_for(PaperMm { width_mm: 80.0, length_mm: 0.0 }, Some(&p), (72.0, 300.0), false);
        let want = 300.0 + hw[1] + hw[3] + 0.5;
        assert!((roll.page.height_mm - want).abs() < 1e-9, "{}", roll.page.height_mm);
        assert!(roll.media.starts_with("Custom.80x"), "{}", roll.media);
        // Across a roll (landscape) the slip's width runs along it.
        let across = page_for(PaperMm { width_mm: 80.0, length_mm: 0.0 }, Some(&p), (72.0, 300.0), true);
        assert!((across.page.height_mm - (72.0 + hw[1] + hw[3] + 0.5)).abs() < 1e-9);
        // Driver not readable: still the printer's paper, edges unknown.
        let blind = page_for(PaperMm { width_mm: 104.8, length_mm: 241.3 }, None, (80.0, 50.0), false);
        assert_eq!(blind.page.margins_mm, [0.0; 4]);
        assert_eq!(blind.media, "Custom.104.8x241.3mm");
        assert!(!blind.fallback);
    }

    /// A driver that takes no custom size prints on its own paper whatever
    /// is asked (Windows "Microsoft Print to PDF" put an 80 mm roll on A4,
    /// 2026-11-16): the page — and so the preview — is that paper, flagged.
    #[test]
    fn a_driver_with_no_custom_sizes_puts_the_page_on_its_own_paper() {
        let p = parse_ppd(&PPD.replace("*CustomPageSize True: \"pop pop pop pop pop\"\n", ""));
        assert!(!p.custom_ok);
        let roll = page_for(PaperMm { width_mm: 80.0, length_mm: 0.0 }, Some(&p), (72.0, 300.0), false);
        assert!(roll.fallback);
        assert_eq!(roll.media, "Small");
        assert_eq!(roll.page, p.by_id("Small").unwrap().clone());
        // A size it names still matches, with no fallback.
        let own = page_for(PaperMm { width_mm: 148.2, length_mm: 209.9 }, Some(&p), (80.0, 50.0), false);
        assert!(!own.fallback && own.media == "Mid", "{own:?}");
    }

    /// The preview is the print (owner, 2026-11-16: it must never fool):
    /// its printable areas hold exactly the printed sheets' pixels, at the
    /// printer's margins, breaking where the print breaks.
    #[test]
    fn the_preview_shows_exactly_the_printed_sheets_on_the_paper() {
        // 10 px per mm, one px per dot: 30 lines of 20 px, 10 px apart.
        let lines: Vec<(u32, u32)> = (0..30).map(|i| (i * 30, i * 30 + 20)).collect();
        let s = slip(400, &lines, 900);
        let page = paper(50.0, 40.0, [2.0, 3.0, 4.0, 5.0]);
        let printed = os_document_sheets(&s, 40.0, &page, false).0;
        assert!(printed.len() > 2, "a long slip goes on more sheets");
        let pic = preview_picture(&s, 1, 40.0, Some(&page), false, 0.0, None);
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
        let page = paper(12.0, 9.0, [1.0; 4]);
        let one = preview_picture(&s1, 1, 8.0, Some(&page), false, 0.0, None);
        let two = preview_picture(&s2, 2, 8.0, Some(&page), false, 0.0, None);
        assert_eq!(two.per_px, 1.0, "drawn back up in whole blocks");
        assert_eq!(two.gray, blocks(&one.gray, 2));
    }

    #[test]
    fn an_escpos_preview_is_the_roll_with_the_picture_from_its_left_edge() {
        // A 90 mm slip on an 80 mm roll: cut at the roll's edge.
        let s = slip(900, &[(0, 10)], 40);
        let pic = preview_picture(&s, 1, 90.0, None, false, 80.0, None);
        assert_eq!(pic.gray.width(), 800);
        assert_eq!(pic.gray.height(), 40);
        assert_eq!(pic.gray.get_pixel(5, 5).0[0], 0, "from the left edge");
        assert_eq!(pic.gray.get_pixel(790, 5).0[0], 0, "ink up to the roll's edge");
        assert_eq!(pic.gray.get_pixel(798, 5).0[0], PREVIEW_EDGE, "the roll's own edge");
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
    fn windows_paper_lines_read_in_hundredths_of_an_inch() {
        let out = "D\tCard 4x6\r\nP\tCard 4x6\t400\t600\t16.5\t16.5\t367\t567\r\nP\tWide\t850\t1100\t0\t0\t0\t0\n";
        let p = parse_windows_papers(out);
        assert_eq!(p.default_id, "Card 4x6");
        let c = p.by_id("Card 4x6").unwrap();
        assert!((c.width_mm - 101.6).abs() < 1e-6);
        assert!((c.margins_mm[0] - 16.5 * 0.254).abs() < 1e-6);
        assert!((c.margins_mm[2] - (400.0 - 16.5 - 367.0) * 0.254).abs() < 1e-6);
        // No printable area given → the whole sheet.
        assert!(p.by_id("Wide").unwrap().margins_mm.iter().all(|m| *m == 0.0));
        // No user-defined size listed: the driver takes none.
        assert!(!p.custom_ok);
        assert!(parse_windows_papers("D\tA4\nC\t1\nP\tA4\t827\t1169\t0\t0\t827\t1169\n").custom_ok);
        // The script names the printer safely.
        assert!(windows_papers_script(Some("O'Brien")).contains("'O''Brien'"));
    }

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

    fn paper(w: f64, h: f64, m: [f64; 4]) -> Paper {
        Paper { id: "P".into(), label: "P".into(), width_mm: w, height_mm: h, margins_mm: m }
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
            let p = paper(w, h, m);
            let (sheets, ppm) = os_document_sheets(&s, 80.0, &p, false);
            assert!((ppm - 10.0).abs() < 1e-9);
            let (pw, _) = p.printable_mm();
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
        let p = paper(10.0 + 2.0, 25.0 + 2.0, [1.0; 4]);
        let (sheets, _) = os_document_sheets(&s, 10.0, &p, false);
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

    #[test]
    fn a_landscape_slip_fills_the_turned_printable_area_from_its_corner() {
        // 10 px/mm, slip 40 mm wide × 10 mm of ink.
        let s = slip(400, &[(0, 100)], 100);
        let p = paper(100.0, 150.0, [3.0, 4.0, 5.0, 6.0]);
        let (sheets, _) = os_document_sheets(&s, 40.0, &p, true);
        assert_eq!(sheets.len(), 1);
        let (pw, ph) = p.printable_mm();
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

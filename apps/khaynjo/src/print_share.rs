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
//! The owner picks each printer (system name or `host:9100`). We do not force
//! one size or one driver mode for every shop:
//! - **Network `host:9100`** → short ASCII ESC/POS over TCP (every OS, incl. Android).
//! - **Named / default system printer** → that OS print path (large office
//!   printers and small KOT queues the owner configured — paper size included).
//! Kitchen vs bill is which **route** they assign in Settings, not a hardcoded
//! device class in code.

use std::collections::BTreeMap;
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

/// One named printer in this device's pool.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedPrinter {
    pub id: String,
    pub label: String,
    /// Blank = system default; name = system printer; `ip` / `ip:port` = ESC/POS.
    pub target: String,
}

/// Ordered printer ids for each print feature.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintRoutes {
    #[serde(default)]
    pub bill: Vec<String>,
    /// Station id → ordered printer ids. Key `""` is unassigned items only.
    #[serde(default)]
    pub kot: BTreeMap<String, Vec<String>>,
}

/// This device's printer pool and routes (Settings → Printers).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DevicePrinters {
    pub printers: Vec<NamedPrinter>,
    pub routes: PrintRoutes,
}

impl Default for DevicePrinters {
    fn default() -> Self {
        // Nothing configured yet: empty pool, empty routes (don't print until
        // the owner adds real printers and assigns them).
        Self {
            printers: Vec::new(),
            routes: PrintRoutes {
                bill: Vec::new(),
                kot: BTreeMap::from([("".into(), Vec::new())]),
            },
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
            routes: PrintRoutes,
        }
        let parsed: New = serde_json::from_value(value).unwrap_or_else(|_| New {
            printers: Vec::new(),
            routes: PrintRoutes::default(),
        });
        let mut out = DevicePrinters {
            printers: parsed.printers,
            routes: parsed.routes,
        };
        scrub_fake_default_printers(&mut out);
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
        routes: PrintRoutes {
            bill: bill_ids,
            kot: BTreeMap::from([("".into(), kot_ids)]),
        },
    }
}

impl DevicePrinters {
    /// Ordered printer ids for this route (empty = don't print).
    ///
    /// KOT lookup order (owner can still leave a route empty on purpose by
    /// clearing Bill too):
    /// 1. that station's own list
    /// 2. default kitchen `""` (unassigned + stations with no printers yet)
    /// 3. **Bill** — so a shop with one printer on Bill still gets every KOT
    pub fn route_for(&self, key: RouteKey<'_>) -> &[String] {
        match key {
            RouteKey::Bill => &self.routes.bill,
            RouteKey::Kot { station_id } => {
                let own = self
                    .routes
                    .kot
                    .get(station_id)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);
                let kitchen = if !own.is_empty() {
                    own
                } else if !station_id.is_empty() {
                    self.routes
                        .kot
                        .get("")
                        .map(|v| v.as_slice())
                        .unwrap_or(&[])
                } else {
                    own
                };
                if !kitchen.is_empty() {
                    kitchen
                } else {
                    self.routes.bill.as_slice()
                }
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
        });
        id
    }

    pub fn remove_printer(&mut self, id: &str) {
        self.printers.retain(|p| p.id != id);
        self.routes.bill.retain(|x| x != id);
        for ids in self.routes.kot.values_mut() {
            ids.retain(|x| x != id);
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
    /// The device's default system printer.
    Default,
    /// A system printer by name.
    Named(String),
    /// A network receipt printer, `ip:port`, spoken to directly (ESC/POS).
    Network(String),
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
    if let Ok(ip) = t.parse::<std::net::Ipv4Addr>() {
        return PrinterTarget::Network(format!("{ip}:{RAW_PRINT_PORT}"));
    }
    if let Ok(addr) = t.parse::<std::net::SocketAddrV4>() {
        return PrinterTarget::Network(addr.to_string());
    }
    PrinterTarget::Named(t.to_string())
}

/// Whether this device can print to `target` without a person: a network
/// printer always; a system printer wherever the OS has printer drivers.
pub fn can_print(target: &PrinterTarget) -> bool {
    matches!(target, PrinterTarget::Network(_)) || !cfg!(target_os = "android")
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
    printers().targets_for(key).iter().any(can_print)
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
    fs::read_to_string(printers_path(base))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
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
/// Text is kept as the shop wrote it. **Network** targets turn it into short
/// ASCII ESC/POS inside [`print_network`] / [`escpos_job`]. **Named / default**
/// targets hand the UTF-8 file to the OS printer the owner picked — we never
/// force raw USB, a paper size, or ESC/POS onto every KOT.
pub fn print_route(
    kind: PrintKind,
    route: RouteKey<'_>,
    text: &str,
    app_files: Option<&Path>,
) -> Result<ReceiptShare, String> {
    let path = write_receipt(text, app_files)?;
    let opened = match sink() {
        PrintSink::Os => {
            let targets: Vec<PrinterTarget> = printers()
                .targets_for(route)
                .into_iter()
                .filter(can_print)
                .collect();
            if targets.is_empty() {
                if kind == PrintKind::Kot {
                    note_problem(
                        "KOTs are not printed on this device — add a kitchen printer in Settings → Printers"
                            .into(),
                    );
                }
                false
            } else {
                queue(Job {
                    kind,
                    targets,
                    path: path.clone(),
                    text: text.to_string(),
                });
                true
            }
        }
        PrintSink::Capture => {
            // Capture still records one job (the slip that would have printed).
            // Prefer the resolved route (station → "" → Bill), not only the
            // exact key — same as Os path via targets_for. A captured job
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
                        "KOTs are not printed on this device — add a kitchen printer in Settings → Printers"
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
    kind: PrintKind,
    /// Try in order; stop after the first success.
    targets: Vec<PrinterTarget>,
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
                for target in &job.targets {
                    match send_to_printer(job.kind, target, &job.path, &job.text) {
                        Ok(()) => {
                            last_err = None;
                            break;
                        }
                        Err(e) => last_err = Some((describe(target), e)),
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
    }
}

fn send_to_printer(
    kind: PrintKind,
    target: &PrinterTarget,
    path: &Path,
    text: &str,
) -> Result<(), String> {
    // Kind only selects the route; the *target* chooses the wire format.
    // Never: "KOT ⇒ always raw" (that tied shops to one concrete USB stack).
    let _ = kind;
    match target {
        // Direct thermal / kitchen over LAN — ESC/POS ASCII, every OS.
        PrinterTarget::Network(addr) => print_network(addr, text),
        // Whatever the OS already knows (Find printers / name). Paper size
        // and driver are the owner's — large laser or small queue alike.
        PrinterTarget::Default => print_system(None, path),
        PrinterTarget::Named(name) => print_system(Some(name), path),
    }
}

/// Send `text` to a network receipt printer: reset, the text, a few lines
/// to clear the tear bar, cut.
pub fn print_network(addr: &str, text: &str) -> Result<(), String> {
    print_network_bytes(addr, &escpos_job(text))
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
    stream.write_all(bytes).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

/// Hard cap: a two-item KOT is a few hundred bytes. Anything near a raster
/// dump (tens of KB+) is refused before it reaches the printer.
const ESC_POS_JOB_MAX: usize = 4_096;

/// Short ASCII ESC/POS job for kitchen / network thermals.
///
/// Never a `GS v 0` raster picture. Never UTF-8. Unknown letters → `?`.
/// OS-driver printing of KOTs is what caused endless unreadable slips.
pub fn escpos_job(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len().saturating_add(32));
    out.extend_from_slice(&[0x1B, 0x40]); // ESC @: reset printer state
    out.extend(escpos_text(text));
    // Feed past the tear bar; partial cut (GS V 1) — widely supported, safer
    // than full cut on some clones.
    out.extend_from_slice(b"\n\n\n\n");
    out.extend_from_slice(&[0x1D, 0x56, 0x01]);
    // Guard: never accidentally ship a raster header.
    debug_assert!(
        !out.windows(3).any(|w| w == [0x1D, 0x76, 0x30]),
        "escpos_job must not contain GS v 0 raster"
    );
    out
}

/// Same slip as a one-bit picture (`GS v 0`). Only for explicit call sites
/// that know the printer handles raster — never the default network path.
pub fn escpos_job_picture(text: &str) -> Vec<u8> {
    crate::receipt_raster::escpos_picture(text)
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
/// (space–`~` and newline), with the few symbols our slips use spelt out,
/// anything else as "?". Control bytes never go through — they switch the
/// printer into graphics/command modes and look like endless garbage.
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

/// Print a file on a system printer (`None` = the default one).
/// Named/default targets use this — owner-configured large or small printers.
fn print_system(name: Option<&str>, path: &Path) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let mut cmd = Command::new("lp");
        if let Some(name) = name {
            cmd.args(["-d", name]);
        }
        cmd.arg("--").arg(path);
        match cmd.output() {
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => Err(first_line(&out.stderr).unwrap_or_else(|| "the printing system refused the job".into())),
            Err(_) => {
                // No printing system at all: at least show the slip.
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
        let mut script = format!(
            "Get-Content -LiteralPath '{}' -Raw -Encoding UTF8 | Out-Printer",
            quote(&path.display().to_string())
        );
        if let Some(name) = name {
            script.push_str(&format!(" -Name '{}'", quote(name)));
        }
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
    #[cfg(target_os = "android")]
    {
        let _ = (name, path);
        Err("this device has no printer drivers — give a network printer's address in Settings → Printers".into())
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

/// Printers last found on this device (system names + network addresses on
/// port [`RAW_PRINT_PORT`]). Empty until a look has finished.
pub fn found_printers() -> Vec<String> {
    FOUND.lock().unwrap().clone().unwrap_or_default()
}

/// `true` while a background look is still running.
pub fn is_finding_printers() -> bool {
    LOOKING.load(std::sync::atomic::Ordering::SeqCst)
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
    if std::thread::Builder::new()
        .name("find-printers".into())
        .spawn(|| {
            let mut found = system_printers();
            for addr in network_raw_printers() {
                if !found.iter().any(|n| n == &addr) {
                    found.push(addr);
                }
            }
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
        // First token of each `lpstat -a` line is a queue that accepts jobs.
        let accepting: Vec<String> = out(&["-a"])
            .lines()
            .filter_map(|l| l.split_whitespace().next().map(str::to_string))
            .filter(|n| !n.is_empty())
            .collect();
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
        accepting
            .into_iter()
            .filter(|n| !disabled.contains(n))
            .collect()
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
    // ~64 workers × 4 hosts × 80ms ≈ 0.3s worst case (was 8×32×200ms ≈ 6s).
    let workers: Vec<_> = addrs
        .chunks(4)
        .map(|chunk| {
            let chunk = chunk.to_vec();
            let found = Arc::clone(&found);
            std::thread::spawn(move || {
                for addr in chunk {
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
    fn empty_kot_falls_back_to_bill_printer() {
        let mut cfg = DevicePrinters::default();
        let p = cfg.add_printer("Counter", "192.168.1.50");
        cfg.routes.bill = vec![p.clone()];
        // No KOT routes filled — Bill still prints kitchen tickets.
        assert_eq!(
            cfg.route_for(RouteKey::Kot { station_id: "" }),
            &[p.clone()]
        );
        assert_eq!(
            cfg.route_for(RouteKey::Kot {
                station_id: "station-a"
            }),
            &[p]
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
        // Network / KOT jobs stay short ASCII (no GS v 0 raster).
        let job_guj = escpos_job("2 x ખીચડી");
        assert!(!job_guj.windows(3).any(|w| w == [0x1D, 0x76, 0x30]));
        assert!(job_guj.windows(5).any(|w| w == b"?????"));
        assert!(job_guj.len() < 256, "KOT job must stay tiny, got {}", job_guj.len());
        let picture = escpos_job_picture("2 x ખીચડી");
        assert!(picture.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]));
        let job = escpos_job("KOT 7\n2 x Thali\n1 x Roti");
        assert_eq!(&job[..2], &[0x1B, 0x40]);
        assert!(job.ends_with(&[0x1D, 0x56, 0x01]));
        assert!(job.len() < ESC_POS_JOB_MAX);
        // Control bytes in the slip body must not pass through.
        assert_eq!(escpos_text("A\u{1b}B\n"), b"A?B\n".to_vec());
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

    /// Printers belong to the install: saved beside the language choice and
    /// read back on the next start.
    #[test]
    fn device_printers_are_kept_per_install() {
        let base = std::env::temp_dir().join(format!("khaynjo-printers-{}", uuid::Uuid::new_v4().simple()));
        assert_eq!(load_device_printers(&base), DevicePrinters::default());
        let mut chosen = DevicePrinters::default();
        let kitchen = chosen.add_printer("Kitchen", "192.168.1.50");
        let counter = chosen.add_printer("Counter", "Canon_G3010");
        chosen.routes.bill = vec![counter.clone()];
        *chosen.route_mut(RouteKey::Kot { station_id: "" }) = vec![kitchen.clone()];
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
        // Station with no printers of its own uses the default kitchen route.
        assert_eq!(
            cfg.route_for(RouteKey::Kot {
                station_id: "unknown"
            }),
            &[main.clone()][..]
        );
        *cfg.route_mut(RouteKey::Kot {
            station_id: "st-bar",
        }) = Vec::new();
        assert_eq!(
            cfg.route_for(RouteKey::Kot {
                station_id: "st-bar"
            }),
            &[main][..]
        );
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
}

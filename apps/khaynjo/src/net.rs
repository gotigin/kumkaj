//! Local LAN Client/Server sync for Kumkaj (no cloud).
//!
//! Server listens on `DEFAULT_PORT` and answers with JSON over HTTP/1.1.
//! Client uses `ureq` to pair, pull catalog/ledger/sales, and push sales/settlements.

use crate::domain::{
    Category, CreditLedgerEntry, Customer, DayTotals, DiningTable,
    KitchenStation, KotTicket, PayMethod, Product, RunningOrder, Sale, SaleDraft, StaffMember,
    TaxRate,
};
use crate::fiscal::FiscalSettings;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const DEFAULT_PORT: u16 = 7847;

/// UDP port a waiter's device asks "where is the shop's server?" on. The
/// server answers from its own Wi‑Fi address — no internet, no typing an IP
/// (Petpooja's captain app scans for it the same way).
pub const DISCOVERY_PORT: u16 = 7848;
const DISCOVERY_ASK: &str = "KHAYNJO_DISCOVER?";
const DISCOVERY_APP: &str = "khaynjo";

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static LAN_ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncSnapshot {
    pub venue_title: String,
    pub products: Vec<Product>,
    pub customers: Vec<Customer>,
    /// Names and rights only — PIN hashes never leave the Server.
    pub staff: Vec<StaffMember>,
    #[serde(default)]
    pub ledger: Vec<CreditLedgerEntry>,
    #[serde(default)]
    pub tables: Vec<DiningTable>,
    #[serde(default)]
    pub recent_sales: Vec<Sale>,
    #[serde(default)]
    pub today: DayTotals,
    #[serde(default)]
    pub outstanding_credit_paise: i64,
    #[serde(default)]
    pub day_close_text: String,
    #[serde(default)]
    pub open_kots: Vec<KotTicket>,
    /// Menu groupings and kitchen stations of the Server's branch.
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub stations: Vec<KitchenStation>,
    /// Server's branch / business identity, shown on the Client header.
    #[serde(default)]
    pub branch_name: String,
    #[serde(default)]
    pub org_name: String,
    /// Country pack in force on the Server, so a terminal prints the same tax
    /// wording, pay-rail name and business id as the till.
    #[serde(default)]
    pub fiscal: FiscalSettings,
    #[serde(default)]
    pub tax_rates: Vec<TaxRate>,
    /// Every table's open order, line by line, so a waiter's device shows
    /// what each table has and the counter can bill it.
    #[serde(default)]
    pub running_orders: Vec<RunningOrder>,
    /// The Server's own id (kept in its books): a device that lost the
    /// Server's address finds it again by this, never by its name.
    #[serde(default)]
    pub server_id: String,
    /// This snapshot's revision. A device sends it back (`If-None-Match`)
    /// and a Server whose books are the same answers 304 instead of
    /// sending them all again. Not part of what is hashed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rev: String,
}

/// A table action a waiter's device asks the Server to do. Rights are
/// checked on the Server; `approval_pin` is a manager's PIN typed on the
/// device when the person's right is "With approval".
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RunningAction {
    Cancel {
        line_id: String,
        #[serde(default)]
        qty_milli: Option<i64>,
        reason: String,
        #[serde(default)]
        prepared: bool,
    },
    Complimentary {
        line_id: String,
        on: bool,
        #[serde(default)]
        reason: String,
    },
    MoveLines {
        line_ids: Vec<String>,
        to_table_id: String,
    },
    /// Some lines to a separate bill on a companion table.
    Split {
        line_ids: Vec<String>,
    },
    MoveTable {
        from_table_id: String,
        to_table_id: String,
    },
    Merge {
        from_table_id: String,
        into_table_id: String,
    },
    PrintBill {
        table_id: String,
    },
    /// Paid / empty table cleared — free for the next guests.
    TableReady {
        table_id: String,
    },
    /// Guests sat down — occupy without opening a running order.
    SeatTable {
        table_id: String,
    },
    /// Accidental Print bill — back to an open table (or free if nothing left).
    UndoBill {
        table_id: String,
    },
    /// Punch or − on a table ticket before Send KOT (pending running line).
    BumpPending {
        table_id: String,
        product_id: String,
        delta_milli: i64,
        #[serde(default)]
        area: String,
    },
    /// Seat unsent cart lines onto a table as pending (pick-table / change).
    SeatPending {
        table_id: String,
        lines: Vec<crate::domain::CartLine>,
    },
    /// Move pending (unsent) lines from one table to another (Change table).
    ReseatPending {
        from_table_id: String,
        to_table_id: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteRunningRequest {
    #[serde(default)]
    pub approval_pin: String,
    pub request: RunningAction,
}

/// A shop server found on this Wi‑Fi.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoundServer {
    /// `ip:port` to pair with.
    pub host: String,
    /// The restaurant's name, so the waiter picks the right one.
    pub venue: String,
    /// The Server's id (empty from an older Server).
    #[serde(default)]
    pub server_id: String,
}

/// One tender inside a split bill. Amounts are minor units and must sum to
/// the charged total on the Server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteTender {
    pub pay: String,
    pub amount_paise: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteSaleRequest {
    pub draft: SaleDraft,
    pub pay: String,
    pub customer_id: Option<String>,
    /// When set and customer_id is missing, Server creates/finds by name.
    #[serde(default)]
    pub customer_name: Option<String>,
    /// Prefer Server table by name (Client UUIDs differ).
    #[serde(default)]
    pub table_name: Option<String>,
    pub table_id: Option<String>,
    /// Split tenders. Empty means a single tender in `pay`.
    #[serde(default)]
    pub payments: Vec<RemoteTender>,
    /// Settle & Save: cash handed over, and a tip (staff's, not a sale).
    #[serde(default)]
    pub tendered_paise: i64,
    #[serde(default)]
    pub tip_paise: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteSettleRequest {
    pub customer_id: Option<String>,
    #[serde(default)]
    pub customer_name: Option<String>,
    pub amount_paise: i64,
    pub pay: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteUndoRequest {
    pub sale_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteSaleAction {
    pub sale_id: String,
    #[serde(default)]
    pub reason: String,
    /// A manager's PIN when the signed-in person may not void/refund;
    /// checked on the Server.
    #[serde(default)]
    pub approval_pin: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteKotAdvance {
    pub kot_id: String,
}

/// Correct a bill the Server owns. The Client never edits its own sales cache.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteAmendRequest {
    pub sale_id: String,
    pub draft: SaleDraft,
    pub pay: String,
    pub reason: String,
}

/// A waiter flags a table "guests are done — bill, please".
/// The Server marks it Payment pending; release still needs payment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteTableBillingRequest {
    #[serde(default)]
    pub table_id: Option<String>,
    #[serde(default)]
    pub table_name: Option<String>,
}

/// Fire a kitchen ticket from a terminal's cart without charging it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteKotRequest {
    pub draft: SaleDraft,
    /// Prefer Server table by name (Client UUIDs differ).
    #[serde(default)]
    pub table_name: Option<String>,
    #[serde(default)]
    pub table_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairOk {
    pub ok: bool,
    pub venue_title: String,
    pub port: u16,
    /// This phone's own key to the Server (the pairing code is not kept).
    #[serde(default)]
    pub device_token: String,
    /// The Server's id, kept by the device to find it again.
    #[serde(default)]
    pub server_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairRequest {
    pub code: String,
    /// Shown in the Server's list of paired phones.
    #[serde(default)]
    pub device_name: String,
}

/// A PIN typed on a phone, checked by the Server. `staff_id` None = Owner.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    #[serde(default)]
    pub staff_id: Option<String>,
    pub pin: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginOk {
    pub session: String,
    pub staff: StaffMember,
}

/// Headers a phone sends: its pairing, who is signed in, and the request's
/// own id (a retry with the same id is answered, not done again).
pub const HEADER_DEVICE: &str = "X-Kumkaj-Device";
pub const HEADER_SESSION: &str = "X-Kumkaj-Session";
pub const HEADER_REQUEST: &str = "X-Kumkaj-Request";
/// The snapshot revision a device already holds (`/sync` answers 304 when
/// the books are still that).
pub const HEADER_IF_NONE_MATCH: &str = "If-None-Match";

/// Longest a device's `/wait` is held open before the Server answers anyway.
pub const WAIT_MAX_SECS: u64 = 30;

/// Replies at least this big are gzipped for a device that accepts it (a
/// busy shop's books are ~1 MB of JSON; gzipped, about a tenth).
const GZIP_FROM_BYTES: usize = 8 * 1024;

/// Largest request the Server reads (a big bill is a few KB).
const MAX_REQUEST_BYTES: usize = 4 * 1024 * 1024;

pub fn server_is_running() -> bool {
    SERVER_RUNNING.load(Ordering::SeqCst) && LAN_ENABLED.load(Ordering::SeqCst)
}

// ---------------------------------------------------------- Windows firewall

/// Inbound rule that lets phones on the shop Wi‑Fi reach this PC. The MSI
/// installs per user (no admin), so the app adds it on request through one
/// Windows admin prompt.
pub const FIREWALL_RULE: &str = "Kumkaj Khaynjo LAN";

/// Other devices can reach the listener through this computer's firewall:
/// on Windows our rule exists; on Linux firewalld or ufw (when one is on)
/// has our ports open. True where there is no firewall to speak of (or no
/// way to tell — never nag for nothing). Blocking (runs `netsh` /
/// `firewall-cmd`): call it off the UI thread.
pub fn firewall_allows_phones() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("netsh")
            .args(["advfirewall", "firewall", "show", "rule"])
            .arg(format!("name={FIREWALL_RULE}"))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            // Cannot tell (no netsh): do not nag.
            .map(|o| o.status.success())
            .unwrap_or(true)
    }
    #[cfg(target_os = "linux")]
    {
        match linux_firewall() {
            LinuxFirewall::Firewalld => std::process::Command::new("firewall-cmd")
                .arg(format!("--query-port={DEFAULT_PORT}/tcp"))
                .output()
                .map(|o| o.status.success())
                .unwrap_or(true),
            LinuxFirewall::Ufw => {
                OPENED_HERE.load(Ordering::SeqCst)
                    || ["/etc/ufw/user.rules", "/lib/ufw/user.rules"]
                        .iter()
                        .filter_map(|f| std::fs::read_to_string(f).ok())
                        .any(|rules| ufw_rules_allow(&rules, DEFAULT_PORT))
            }
            LinuxFirewall::None => true,
        }
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        true
    }
}

/// Which Linux firewall guards this computer (one that is on).
#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxFirewall {
    Firewalld,
    Ufw,
    None,
}

#[cfg(target_os = "linux")]
fn linux_firewall() -> LinuxFirewall {
    let firewalld = std::process::Command::new("firewall-cmd")
        .arg("--state")
        .output()
        .is_ok_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "running");
    if firewalld {
        return LinuxFirewall::Firewalld;
    }
    let ufw = std::fs::read_to_string("/etc/ufw/ufw.conf").is_ok_and(|conf| ufw_conf_enabled(&conf));
    if ufw { LinuxFirewall::Ufw } else { LinuxFirewall::None }
}

/// Our ports were opened from this app in this run (ufw's rules file is not
/// always readable without root).
#[cfg(target_os = "linux")]
static OPENED_HERE: AtomicBool = AtomicBool::new(false);

/// ufw's own switch in `/etc/ufw/ufw.conf`.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn ufw_conf_enabled(conf: &str) -> bool {
    conf.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .any(|l| l.replace(' ', "").eq_ignore_ascii_case("ENABLED=yes"))
}

/// ufw's rules let TCP `port` in.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn ufw_rules_allow(rules: &str, port: u16) -> bool {
    let want = format!("--dport {port} ");
    rules.lines().any(|l| l.contains("-j ACCEPT") && l.contains(&want) && !l.contains("-p udp"))
}

/// Add the inbound rule (TCP 7847 + UDP 7848, this exe only, local subnet only, every
/// network profile — shop Wi‑Fi is often marked Public). This exe's older
/// rules go first: "Cancel" on Windows' own first prompt leaves block rules,
/// and a block rule beats an allow rule. Blocking until the person answers
/// the admin prompt: call it off the UI thread.
pub fn allow_phones_through_firewall() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe = exe.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$exe = '{exe}'; $rule = '{FIREWALL_RULE}';
             netsh advfirewall firewall delete rule name=all program=\"$exe\" | Out-Null
             netsh advfirewall firewall delete rule name=\"$rule\" | Out-Null
             netsh advfirewall firewall add rule name=\"$rule\" dir=in action=allow protocol=TCP localport={tcp} program=\"$exe\" remoteip=localsubnet profile=any | Out-Null
             netsh advfirewall firewall add rule name=\"$rule\" dir=in action=allow protocol=UDP localport={udp} program=\"$exe\" remoteip=localsubnet profile=any | Out-Null",
            tcp = DEFAULT_PORT,
            udp = DISCOVERY_PORT,
        );
        let encoded = base64_utf16le(&script);
        let outer = format!(
            "Start-Process powershell -Verb RunAs -Wait -WindowStyle Hidden \
             -ArgumentList '-NoProfile','-EncodedCommand','{encoded}'"
        );
        let out = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &outer])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("Windows did not allow it (the admin prompt was declined)".into());
        }
        if !firewall_allows_phones() {
            return Err("The firewall rule was not added — ask whoever runs this PC's admin account".into());
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        // One admin prompt (polkit) for both ports, like Windows' UAC.
        let script = match linux_firewall() {
            LinuxFirewall::Firewalld => format!(
                "firewall-cmd --permanent --add-port={DEFAULT_PORT}/tcp --add-port={DISCOVERY_PORT}/udp && firewall-cmd --reload"
            ),
            LinuxFirewall::Ufw => format!("ufw allow {DEFAULT_PORT}/tcp && ufw allow {DISCOVERY_PORT}/udp"),
            LinuxFirewall::None => return Ok(()),
        };
        let out = std::process::Command::new("pkexec")
            .args(["sh", "-c", &script])
            .output()
            .map_err(|e| format!("No admin prompt on this computer (pkexec: {e}) — open TCP {DEFAULT_PORT} and UDP {DISCOVERY_PORT} in its firewall"))?;
        if !out.status.success() {
            return Err(match out.status.code() {
                Some(126) | Some(127) => "The admin prompt was declined".to_string(),
                _ => String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("The firewall did not change").to_string(),
            });
        }
        OPENED_HERE.store(true, Ordering::SeqCst);
        if !firewall_allows_phones() {
            return Err(format!("The firewall still blocks TCP {DEFAULT_PORT} — open it in the firewall settings"));
        }
        Ok(())
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Ok(())
    }
}

/// PowerShell `-EncodedCommand` wants base64 of UTF‑16LE; no quoting games.
#[cfg(any(windows, test))]
fn base64_utf16le(text: &str) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ABC[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Spawn (or resume) the shop LAN listener. Extra tills stay off until this is called.
pub fn ensure_server(db_path: PathBuf) -> Result<(), String> {
    LAN_ENABLED.store(true, Ordering::SeqCst);
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }
    if SERVER_RUNNING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let path = db_path;
    // Made now, so the first device to find this Server learns its id.
    if let Ok(store) = Store::open_path(&path) {
        let _ = store.lan_server_id();
    }
    let venue_path = path.clone();
    let _ = thread::Builder::new()
        .name("khaynjo-discovery".into())
        .spawn(move || run_discovery_responder(&venue_path));
    let watch_path = path.clone();
    let _ = thread::Builder::new()
        .name("khaynjo-books-watch".into())
        .spawn(move || watch_books(&watch_path, &|| LAN_ENABLED.load(Ordering::SeqCst)));
    let _handle: JoinHandle<()> = thread::Builder::new()
        .name("kumkaj-lan".into())
        .spawn(move || {
            if let Err(e) = run_server_loop(path) {
                eprintln!("Kumkaj LAN server stopped: {e}");
            }
            SERVER_RUNNING.store(false, Ordering::SeqCst);
        })
        .map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(50));
    Ok(())
}

/// Stop accepting extra tills. The shop books stay on this device (same role a
/// future cloud drain will sync). The listen thread exits on the next accept poll.
pub fn stop_server() {
    LAN_ENABLED.store(false, Ordering::SeqCst);
}

fn run_server_loop(db_path: PathBuf) -> Result<(), String> {
    // Phones depend on this PC: it must not sleep while they can join.
    let _awake = KeepAwake::hold();
    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    let listener = TcpListener::bind(addr).map_err(|e| format!("bind {DEFAULT_PORT}: {e}"))?;
    serve(listener, db_path, &|| LAN_ENABLED.load(Ordering::SeqCst))
}

/// Answer devices on `listener` while `keep_going` says so, one thread per
/// request. The shop's listener and the tests' own (on a spare port) run
/// this same loop.
pub fn serve(listener: TcpListener, db_path: PathBuf, keep_going: &dyn Fn() -> bool) -> Result<(), String> {
    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;
    let guard = Arc::new(Guard::default());
    while keep_going() {
        match listener.accept() {
            Ok((stream, peer)) => {
                // Accepted sockets inherit non-blocking on some systems.
                let _ = stream.set_nonblocking(false);
                let path = db_path.clone();
                let guard = Arc::clone(&guard);
                let _ = thread::spawn(move || {
                    if let Err(e) = handle_client(stream, &path, &peer.ip().to_string(), &guard) {
                        eprintln!("Kumkaj LAN request error: {e}");
                    }
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                eprintln!("Kumkaj LAN accept: {e}");
                thread::sleep(Duration::from_millis(80));
            }
        }
    }
    Ok(())
}

// ---- Books revision: answer "has anything changed?" without the books ------

/// The Server's books as devices last saw them: per books file, a revision
/// that changes whenever anything on the snapshot does, and a bell for the
/// devices waiting on `/wait` (and for this device's own screens).
struct BooksWatch {
    revs: Mutex<std::collections::HashMap<PathBuf, String>>,
    changed: Condvar,
    hook: Mutex<Option<fn()>>,
}

fn books_watch() -> &'static BooksWatch {
    static WATCH: OnceLock<BooksWatch> = OnceLock::new();
    WATCH.get_or_init(|| BooksWatch {
        revs: Mutex::new(std::collections::HashMap::new()),
        changed: Condvar::new(),
        hook: Mutex::new(None),
    })
}

/// Called (from a Server thread) when the books changed — a waiter's KOT,
/// a payment from a tablet — so this device's own screens refresh at once.
pub fn set_books_changed_hook(hook: fn()) {
    *books_watch().hook.lock().unwrap() = Some(hook);
}

/// A snapshot's revision: its hash, short (the books are the same exactly
/// when their snapshot is).
fn snapshot_rev(json: &str) -> String {
    hex::encode(&Sha256::digest(json.as_bytes())[..10])
}

/// Record the revision of the books at `db_path`; ring the bell when it moved.
fn note_rev(db_path: &Path, rev: &str) {
    let watch = books_watch();
    {
        let mut revs = watch.revs.lock().unwrap();
        if revs.get(db_path).is_some_and(|r| r == rev) {
            return;
        }
        revs.insert(db_path.to_path_buf(), rev.to_string());
    }
    watch.changed.notify_all();
    if let Some(hook) = *watch.hook.lock().unwrap() {
        hook();
    }
}

/// The current books as a snapshot, its JSON (without the revision) and
/// the revision.
fn snapshot_json(store: &Store) -> Result<(String, String), String> {
    let snap = build_snapshot(store)?;
    let json = serde_json::to_string(&snap).map_err(|e| e.to_string())?;
    let rev = snapshot_rev(&json);
    Ok((json, rev))
}

/// How often the watcher looks for new commits (one cheap read each time).
const WATCH_EVERY: Duration = Duration::from_millis(300);
/// Rebuilt at least this often anyway: the day can roll over, and a paid
/// table frees itself after a while, with no one writing.
const REBUILD_EVERY: Duration = Duration::from_secs(30);

/// Keep the books' revision current while `keep_going` says so. SQLite's
/// `data_version` moves whenever any other connection (the counter's
/// screen, a device's request) commits; only then is the snapshot rebuilt
/// and hashed, and waiting devices are woken only if it really changed.
pub(crate) fn watch_books(db_path: &Path, keep_going: &dyn Fn() -> bool) {
    let Ok(store) = Store::open_path(db_path) else {
        return;
    };
    let mut seen = None;
    let mut built = Instant::now();
    while keep_going() {
        let version = store.data_version().ok();
        if version != seen || built.elapsed() >= REBUILD_EVERY {
            seen = version;
            built = Instant::now();
            if let Ok((_, rev)) = snapshot_json(&store) {
                note_rev(db_path, &rev);
            }
        }
        thread::sleep(WATCH_EVERY);
    }
}

/// `/wait`: answer at once if the books are no longer `rev`, else when they
/// change, else after `secs` — with the revision they have then.
fn wait_for_change(db_path: &Path, rev: &str, secs: u64) -> Result<Reply, String> {
    let watch = books_watch();
    if !watch.revs.lock().unwrap().contains_key(db_path) {
        // No watcher has looked yet (just started): look now.
        let store = Store::open_path(db_path)?;
        note_rev(db_path, &snapshot_json(&store)?.1);
    }
    let deadline = Instant::now() + Duration::from_secs(secs.clamp(1, WAIT_MAX_SECS));
    let mut revs = watch.revs.lock().unwrap();
    while revs.get(db_path).is_some_and(|r| r == rev) {
        let Some(left) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        revs = watch.changed.wait_timeout(revs, left).unwrap().0;
    }
    let now = revs.get(db_path).cloned().unwrap_or_default();
    Ok(Reply::json(serde_json::json!({ "rev": now })))
}

/// `/sync`: the books, or 304 when the device already has this revision.
fn sync_reply(store: &Store, have: Option<&str>) -> Result<Reply, String> {
    let (json, rev) = snapshot_json(store)?;
    note_rev(store.db_path(), &rev);
    if have.map(|h| h.trim().trim_matches('"')) == Some(rev.as_str()) {
        return Ok(Reply {
            status: 304,
            body: String::new(),
            content_type: "application/json",
        });
    }
    // The revision goes in front; it was not part of what was hashed.
    let body = format!("{{\"rev\":\"{rev}\",{}", &json[1..]);
    Ok(Reply {
        status: 200,
        body,
        content_type: "application/json",
    })
}

/// Holds the computer awake (no sleep, no idle suspend) while the LAN server
/// runs; let go when it stops. Best effort: where the system offers no way,
/// nothing happens.
struct KeepAwake {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    child: Option<std::process::Child>,
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetThreadExecutionState(flags: u32) -> u32;
}

#[cfg(windows)]
const ES_CONTINUOUS: u32 = 0x8000_0000;
#[cfg(windows)]
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;

impl KeepAwake {
    fn hold() -> Self {
        #[cfg(windows)]
        {
            // Per thread: this server thread holds it until it ends.
            unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED) };
            Self {}
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::process::{Command, Stdio};
            let pid = std::process::id().to_string();
            // The helper ends with this app even if the app is killed.
            let mut cmd = if cfg!(target_os = "linux") {
                let mut c = Command::new("systemd-inhibit");
                c.args(["--what=sleep:idle", "--who=Kumkaj Khaynjo", "--why=Devices are connected to this shop's server", "--mode=block"]);
                c.args(["tail", &format!("--pid={pid}"), "-f", "/dev/null"]);
                c
            } else {
                let mut c = Command::new("caffeinate");
                c.args(["-i", "-w", &pid]);
                c
            };
            let child = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().ok();
            Self { child }
        }
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        {
            Self {}
        }
    }
}

impl Drop for KeepAwake {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn build_snapshot(store: &Store) -> Result<SyncSnapshot, String> {
    let mut recent_sales = store.recent_sales(40)?;
    for sale in &mut recent_sales {
        sale.lines = store.sale_lines(&sale.id).unwrap_or_default();
    }
    let staff = store
        .list_staff()?
        .into_iter()
        .map(|s| StaffMember { pin_hash: String::new(), password_hash: String::new(), ..s })
        .collect();
    Ok(SyncSnapshot {
        venue_title: store.venue_title().unwrap_or_default(),
        products: store.list_products(true)?,
        customers: store.list_customers()?,
        staff,
        ledger: store.list_all_credit_ledger(2000)?,
        tables: store.list_tables()?,
        recent_sales,
        today: store.today_totals()?,
        outstanding_credit_paise: store.outstanding_credit_total()?,
        day_close_text: store.day_close_summary()?,
        open_kots: store.list_open_kots()?,
        categories: store.list_categories()?,
        stations: store.list_stations()?,
        branch_name: store.active_branch().map(|b| b.name).unwrap_or_default(),
        org_name: store.saved_org_name().unwrap_or_default(),
        fiscal: store.fiscal_settings()?,
        tax_rates: store.list_tax_rates()?,
        running_orders: store.list_running_orders()?,
        server_id: store.lan_server_id()?,
        rev: String::new(),
    })
}

/// Print just-sent KOTs at this (Server) device when the shop prints KOTs
/// automatically. Counter and waiter KOTs both come through here. Returns
/// the first print error, if any (the KOTs are sent either way).
pub fn print_sent_kots(store: &Store, kot_ids: &[String]) -> Option<String> {
    if !store.kot_auto_print() {
        return None;
    }
    kot_ids.iter().filter_map(|id| store.print_kot(id).err()).next()
}

/// Do a table action on the Server's books. Shared by the LAN endpoint and
/// the counter itself, so both obey the same rights. Returns ticket/bill
/// text to print when the action produced one.
pub fn apply_running_action(
    store: &Store,
    actor: &StaffMember,
    approval_pin: Option<&str>,
    action: &RunningAction,
) -> Result<Option<String>, String> {
    match action {
        RunningAction::Cancel { line_id, qty_milli, reason, prepared } => {
            let kot_id = store.cancel_running_line(
                line_id,
                *qty_milli,
                reason,
                *prepared,
                actor,
                approval_pin,
            )?;
            let _ = print_sent_kots(store, std::slice::from_ref(&kot_id));
            Ok(None)
        }
        RunningAction::Complimentary { line_id, on, reason } => {
            store.set_running_line_complimentary(line_id, *on, reason, actor, approval_pin)?;
            Ok(None)
        }
        RunningAction::MoveLines { line_ids, to_table_id } => {
            store.move_running_lines(line_ids, to_table_id, actor, approval_pin)?;
            Ok(None)
        }
        RunningAction::Split { line_ids } => {
            store.split_running_lines(line_ids, actor)?;
            Ok(None)
        }
        RunningAction::MoveTable { from_table_id, to_table_id } => {
            store.move_table(from_table_id, to_table_id, actor, approval_pin)?;
            Ok(None)
        }
        RunningAction::Merge { from_table_id, into_table_id } => {
            store.merge_tables(from_table_id, into_table_id, actor, approval_pin)?;
            Ok(None)
        }
        RunningAction::PrintBill { table_id } => {
            Ok(Some(store.print_running_bill(table_id, actor, approval_pin)?))
        }
        RunningAction::TableReady { table_id } => {
            store.free_table(table_id)?;
            store.audit(actor, "table.ready", table_id)?;
            Ok(None)
        }
        RunningAction::SeatTable { table_id } => {
            store.seat_table(table_id, actor)?;
            Ok(None)
        }
        RunningAction::UndoBill { table_id } => {
            store.undo_bill_print(table_id, actor)?;
            Ok(None)
        }
        RunningAction::BumpPending {
            table_id,
            product_id,
            delta_milli,
            area,
        } => {
            let product = store
                .get_product(product_id)?
                .ok_or_else(|| "Missing item".to_string())?;
            store.bump_pending_product(table_id, &product, *delta_milli, area, actor)?;
            Ok(None)
        }
        RunningAction::SeatPending { table_id, lines } => {
            store.seat_unsent_lines(table_id, lines, actor)?;
            Ok(None)
        }
        RunningAction::ReseatPending {
            from_table_id,
            to_table_id,
        } => {
            store.reseat_pending(from_table_id, to_table_id, actor)?;
            Ok(None)
        }
    }
}

/// Wrong pairing codes and PINs, counted per key ("pair:<ip>",
/// "login:<device>:<staff>"). Too many and the key waits: a 6‑digit code
/// cannot be guessed from the shop Wi‑Fi, nor a 4‑digit PIN from a phone.
#[derive(Default)]
pub struct Guard {
    fails: std::sync::Mutex<std::collections::HashMap<String, (u32, Option<std::time::Instant>)>>,
}

const GUARD_TRIES: u32 = 5;
const GUARD_WAIT: Duration = Duration::from_secs(300);
/// Every wrong code from anywhere: past this the pairing door shuts for all.
const GUARD_TRIES_ALL: u32 = 30;

impl Guard {
    fn blocked(&self, key: &str) -> Option<u64> {
        let fails = self.fails.lock().unwrap();
        let until = fails.get(key)?.1?;
        let left = until.saturating_duration_since(std::time::Instant::now());
        (!left.is_zero()).then(|| left.as_secs().max(1))
    }

    fn failed(&self, key: &str, limit: u32) {
        let mut fails = self.fails.lock().unwrap();
        let entry = fails.entry(key.to_string()).or_insert((0, None));
        if entry.1.is_some_and(|t| t <= std::time::Instant::now()) {
            *entry = (0, None);
        }
        entry.0 += 1;
        if entry.0 >= limit {
            entry.1 = Some(std::time::Instant::now() + GUARD_WAIT);
        }
    }

    fn passed(&self, key: &str) {
        self.fails.lock().unwrap().remove(key);
    }
}

/// One HTTP request, read whole (a body can arrive in several packets).
pub struct Request {
    pub method: String,
    pub path: String,
    headers: Vec<(String, String)>,
    pub body: String,
}

impl Request {
    pub fn new(method: &str, path: &str, headers: Vec<(String, String)>, body: &str) -> Self {
        Self { method: method.into(), path: path.into(), headers, body: body.into() }
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.is_empty())
    }

    /// The path without its query string.
    fn route(&self) -> &str {
        self.path.split('?').next().unwrap_or("/")
    }

    /// A query-string value (`/wait?rev=ab12&secs=25` → `query("rev")`).
    fn query(&self, key: &str) -> Option<&str> {
        self.path
            .split_once('?')?
            .1
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
    }

    /// The device reads gzip (ureq's `gzip` feature says so on its own).
    fn accepts_gzip(&self) -> bool {
        self.header("accept-encoding").is_some_and(|v| v.to_ascii_lowercase().contains("gzip"))
    }
}

fn read_request(stream: &mut impl Read) -> Result<Request, String> {
    let mut buf = Vec::with_capacity(8 * 1024);
    let mut chunk = [0u8; 16 * 1024];
    let head_end = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if buf.len() > 64 * 1024 {
            return Err("request head too large".into());
        }
        let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("connection closed before the request was complete".into());
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or("").split_whitespace();
    let method = first.next().unwrap_or("GET").to_string();
    let path = first.next().unwrap_or("/").to_string();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    let length = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    if length > MAX_REQUEST_BYTES {
        return Err("request too large".into());
    }
    let mut body = buf[head_end + 4..].to_vec();
    while body.len() < length {
        let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("connection closed before the request was complete".into());
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(length);
    let body = String::from_utf8(body).map_err(|_| "request body is not UTF-8".to_string())?;
    Ok(Request { method, path, headers, body })
}

/// What the Server answers: an HTTP status and a body (JSON by default).
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub content_type: &'static str,
}

impl Reply {
    fn json(value: serde_json::Value) -> Self {
        Self {
            status: 200,
            body: value.to_string(),
            content_type: "application/json",
        }
    }

    fn html(body: String) -> Self {
        Self {
            status: 200,
            body,
            content_type: "text/html; charset=utf-8",
        }
    }

    fn of<T: Serialize>(value: &T) -> Result<Self, String> {
        Ok(Self {
            status: 200,
            body: serde_json::to_string(value).map_err(|e| e.to_string())?,
            content_type: "application/json",
        })
    }

    fn done() -> Self {
        Self::json(serde_json::json!({ "ok": true }))
    }

    /// A refusal the person must read, in the Server's words. `code` tells
    /// the phone what to do about it ("pair_again", "sign_in_again", "wait").
    fn refuse(status: u16, message: &str, code: &str) -> Self {
        Self {
            status,
            body: serde_json::json!({ "ok": false, "error": message, "code": code }).to_string(),
            content_type: "application/json",
        }
    }
}

fn handle_client(mut stream: TcpStream, db_path: &Path, peer: &str, guard: &Guard) -> Result<(), String> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));
    let (reply, gzip) = match read_request(&mut stream) {
        Ok(req) => (answer(db_path, &req, peer, guard), req.accepts_gzip()),
        Err(e) => (Reply::refuse(400, &e, ""), false),
    };
    write_http(&mut stream, reply.status, reply.content_type, &reply.body, gzip)
}

/// Answer one request. Every outcome is a JSON reply — a failure is never a
/// dropped connection the phone has to guess about.
pub fn answer(db_path: &Path, req: &Request, peer: &str, guard: &Guard) -> Reply {
    match answer_inner(db_path, req, peer, guard) {
        Ok(reply) => reply,
        Err(e) => Reply::refuse(400, &e, ""),
    }
}

fn answer_inner(db_path: &Path, req: &Request, peer: &str, guard: &Guard) -> Result<Reply, String> {
    let route = req.route();
    let mut store = Store::open_path(db_path)?;
    // Public LAN HTML / guest pages — no pairing (restaurant Wi‑Fi only).
    if let Some(reply) = public_lan_page(&store, req)? {
        return Ok(reply);
    }
    match route {
        "/health" => return Ok(Reply::done()),
        "/hello" => {
            return Ok(Reply::json(serde_json::json!({
                "app": DISCOVERY_APP,
                "venue": store.listing_name().unwrap_or_default(),
                "server_id": store.lan_server_id().unwrap_or_default(),
            })));
        }
        "/pair" => return pair(&store, req, peer, guard),
        _ => {}
    }
    let Some(device) = store.lan_device_for_token(req.header(HEADER_DEVICE).unwrap_or_default())? else {
        return Ok(Reply::refuse(
            401,
            "This device is not paired with the shop any more — pair it again",
            "pair_again",
        ));
    };
    match route {
        "/sync" | "/today" => {
            store.touch_device_seen(&device.id)?;
            return sync_reply(&store, req.header(HEADER_IF_NONE_MATCH));
        }
        "/wait" => {
            store.touch_device_seen(&device.id)?;
            // Held open until the books change: let go of the books first.
            drop(store);
            let secs = req.query("secs").and_then(|s| s.parse().ok()).unwrap_or(WAIT_MAX_SECS);
            return wait_for_change(db_path, req.query("rev").unwrap_or_default(), secs);
        }
        "/login" => return login(&store, req, &device.id, guard),
        _ => {}
    }
    let Some(actor) = store.lan_session_staff(&device.id, req.header(HEADER_SESSION).unwrap_or_default())? else {
        return Ok(Reply::refuse(401, "Sign in again on this device", "sign_in_again"));
    };
    if req.method != "POST" {
        return Ok(Reply::refuse(404, "not found", ""));
    }
    let request_id = req.header(HEADER_REQUEST);
    if let Some(id) = request_id
        && let Some((status, body)) = store.lan_reply(id, &device.id)?
    {
        return Ok(Reply {
            status,
            body,
            content_type: "application/json",
        });
    }
    let reply = match write(&mut store, route, &req.body, &actor) {
        Ok(reply) => reply,
        Err(e) => Reply::refuse(400, &e, ""),
    };
    if let Some(id) = request_id
        && reply.status != 404
    {
        // A refusal is an answer too: a retry gets the same words.
        store.save_lan_reply(id, &device.id, reply.status, &reply.body)?;
    }
    Ok(reply)
}

fn pair(store: &Store, req: &Request, peer: &str, guard: &Guard) -> Result<Reply, String> {
    let key = format!("pair:{peer}");
    if let Some(left) = guard.blocked(&key).or_else(|| guard.blocked("pair:*")) {
        return Ok(Reply::refuse(
            429,
            &format!("Too many wrong pairing codes — wait {left}s, then try again"),
            "wait",
        ));
    }
    let asked: PairRequest = serde_json::from_str(&req.body).map_err(|e| format!("pair json: {e}"))?;
    if asked.code.trim().is_empty() || asked.code.trim() != store.pairing_code()? {
        guard.failed(&key, GUARD_TRIES);
        guard.failed("pair:*", GUARD_TRIES_ALL);
        return Ok(Reply::refuse(403, "Wrong pairing code — check it on the shop device", ""));
    }
    guard.passed(&key);
    let (_, token) = store.pair_lan_device(&asked.device_name)?;
    Reply::of(&PairOk {
        ok: true,
        venue_title: store.venue_title().unwrap_or_default(),
        port: DEFAULT_PORT,
        device_token: token,
        server_id: store.lan_server_id()?,
    })
}

fn login(store: &Store, req: &Request, device_id: &str, guard: &Guard) -> Result<Reply, String> {
    let asked: LoginRequest = serde_json::from_str(&req.body).map_err(|e| format!("login json: {e}"))?;
    let key = format!("login:{device_id}:{}", asked.staff_id.as_deref().unwrap_or("owner"));
    if let Some(left) = guard.blocked(&key) {
        return Ok(Reply::refuse(429, &format!("Too many wrong PINs — wait {left}s, then try again"), "wait"));
    }
    let Some(staff) = store.authenticate_lan_pin(asked.staff_id.as_deref(), &asked.pin)? else {
        guard.failed(&key, GUARD_TRIES);
        return Ok(Reply::refuse(403, "Wrong PIN", "wrong_pin"));
    };
    guard.passed(&key);
    let session = store.open_lan_session(device_id, &staff.id)?;
    store.audit(&staff, "lan.sign_in", device_id)?;
    Reply::of(&LoginOk {
        session,
        staff: StaffMember { pin_hash: String::new(), password_hash: String::new(), ..staff },
    })
}

/// The person who stands behind a void/refund from a phone: the signed-in
/// person when they may, else the manager whose PIN was typed (checked
/// here, never on the phone). Returns the actor and the reason to record.
fn approve_on_server(
    store: &Store,
    actor: &StaffMember,
    pin: &str,
    reason: &str,
) -> Result<(StaffMember, String), String> {
    if actor.role.can_void_refund() {
        return Ok((actor.clone(), reason.trim().to_string()));
    }
    let approver = store.verify_approval_pin(pin)?;
    let note = format!("{} · approved by {} for {}", reason.trim(), approver.name, actor.name);
    Ok((approver, note))
}

/// A signed-in phone's write, done as `actor`.
fn write(store: &mut Store, route: &str, body: &str, actor: &StaffMember) -> Result<Reply, String> {
    match route {
        "/sale" => {
            let req: RemoteSaleRequest = serde_json::from_str(body).map_err(|e| format!("sale json: {e}"))?;
            Reply::of(&remote_sale(store, &req, actor)?)
        }
        "/settle" => {
            let req: RemoteSettleRequest = serde_json::from_str(body).map_err(|e| format!("settle json: {e}"))?;
            let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
            let customer_id = if let Some(id) = req.customer_id.clone() {
                id
            } else if let Some(name) = req.customer_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                store.find_or_create_customer(name)?.id
            } else {
                return Err("Customer required".into());
            };
            store.settle_credit(&customer_id, req.amount_paise, pay, actor, &req.detail)?;
            Ok(Reply::done())
        }
        "/undo" => {
            let req: RemoteUndoRequest = serde_json::from_str(body).map_err(|e| format!("undo json: {e}"))?;
            store.quick_undo_sale(&req.sale_id, actor)?;
            Ok(Reply::done())
        }
        "/void" | "/refund" => {
            let req: RemoteSaleAction = serde_json::from_str(body).map_err(|e| format!("sale action json: {e}"))?;
            let (who, reason) = approve_on_server(store, actor, &req.approval_pin, &req.reason)?;
            if route == "/void" {
                store.void_sale(&req.sale_id, &reason, &who)?;
            } else {
                store.refund_sale(&req.sale_id, &reason, &who)?;
            }
            Ok(Reply::done())
        }
        "/kot_advance" => {
            let req: RemoteKotAdvance = serde_json::from_str(body).map_err(|e| format!("kot json: {e}"))?;
            store.advance_kot(&req.kot_id, actor)?;
            Ok(Reply::done())
        }
        "/kot_recall" => {
            let req: RemoteKotAdvance = serde_json::from_str(body).map_err(|e| format!("kot recall json: {e}"))?;
            store.recall_kot(&req.kot_id, actor)?;
            Ok(Reply::done())
        }
        "/upi_display" => {
            // Second-device LAN: amount + VPA + URI (guest chrome deferred).
            #[derive(Deserialize)]
            struct UpiReq {
                #[serde(default)]
                amount_paise: i64,
            }
            let req: UpiReq = serde_json::from_str(if body.trim().is_empty() { "{}" } else { body })
                .map_err(|e| format!("upi json: {e}"))?;
            let vpa = store.upi_vpa()?;
            let payee = store.upi_payee()?;
            let uri = crate::upi_qr::upi_pay_uri(&vpa, &payee, req.amount_paise)?;
            Ok(Reply::json(serde_json::json!({
                "amount_paise": req.amount_paise,
                "vpa": vpa,
                "payee": payee,
                "upi_uri": uri,
            })))
        }
        "/amend" => {
            let req: RemoteAmendRequest = serde_json::from_str(body).map_err(|e| format!("amend json: {e}"))?;
            let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
            Reply::of(&store.amend_sale(&req.sale_id, &req.draft, pay, &req.reason, actor)?)
        }
        "/table_billing" => {
            let req: RemoteTableBillingRequest =
                serde_json::from_str(body).map_err(|e| format!("table billing json: {e}"))?;
            let table_id = resolve_table_id(store, req.table_name.as_deref(), req.table_id.as_deref())?
                .ok_or_else(|| "table not found".to_string())?;
            store.request_billing(&table_id, actor)?;
            Ok(Reply::done())
        }
        "/kot" => {
            let req: RemoteKotRequest = serde_json::from_str(body).map_err(|e| format!("kot json: {e}"))?;
            let table_id = resolve_table_id(store, req.table_name.as_deref(), req.table_id.as_deref())?;
            let kot_ids = store.send_kot(&req.draft, table_id.as_deref(), actor)?;
            // The counter prints the kitchen's paper the moment a waiter's
            // order lands — nobody carries anything to the kitchen.
            let _ = print_sent_kots(store, &kot_ids);
            Ok(Reply::json(serde_json::json!({ "ok": true, "kot_ids": kot_ids })))
        }
        "/running" => {
            let req: RemoteRunningRequest = serde_json::from_str(body).map_err(|e| format!("running json: {e}"))?;
            let pin = Some(req.approval_pin.as_str()).filter(|p| !p.trim().is_empty());
            let text = apply_running_action(store, actor, pin, &req.request)?;
            // A bill printed for a waiter prints at the counter, where the
            // printer is; the text also goes back to the device.
            if let Some(text) = &text {
                let _ = crate::print_share::print_text(crate::print_share::PrintKind::Bill, text, store.db_path().parent());
            }
            Ok(Reply::json(serde_json::json!({ "ok": true, "text": text })))
        }
        _ => Ok(Reply::refuse(404, "not found", "")),
    }
}

fn remote_sale(store: &mut Store, req: &RemoteSaleRequest, actor: &StaffMember) -> Result<Sale, String> {
    let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
    let table_id = resolve_table_id(store, req.table_name.as_deref(), req.table_id.as_deref())?;
    let customer_for = |store: &Store, needed: bool, what: &str| -> Result<Option<String>, String> {
        if !needed {
            return Ok(None);
        }
        if let Some(id) = req.customer_id.clone() {
            return Ok(Some(id));
        }
        match req.customer_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(name) => Ok(Some(store.find_or_create_customer(name)?.id)),
            None => Err(format!("{what} needs a customer")),
        }
    };
    let sale = if req.payments.is_empty() {
        let customer_id = customer_for(store, pay.is_credit(), "Credit sale")?;
        store.finalize_sale(&req.draft, pay, table_id.as_deref(), customer_id.as_deref(), actor)?
    } else {
        // Mixed Cash/UPI/Card/Credit (and legacy split) — amounts in `payments`.
        if !pay.is_split() && req.payments.len() > 1 {
            return Err("Multi-tender sale must use pay method split".into());
        }
        let mut parts = Vec::with_capacity(req.payments.len());
        for tender in &req.payments {
            let method = PayMethod::parse(&tender.pay).ok_or_else(|| format!("bad pay method {}", tender.pay))?;
            parts.push((method, tender.amount_paise));
        }
        let needs_credit = parts.iter().any(|(m, _)| m.is_credit());
        let customer_id = customer_for(store, needs_credit, "Credit part")?;
        store.finalize_sale_tenders(&req.draft, &parts, table_id.as_deref(), customer_id.as_deref(), actor)?
    };
    if req.tendered_paise > 0 || req.tip_paise > 0 {
        let cash: i64 = if req.payments.is_empty() {
            if req.pay == "cash" { sale.total_paise } else { 0 }
        } else {
            req.payments.iter().filter(|t| t.pay == "cash").map(|t| t.amount_paise).sum()
        };
        let change = (req.tendered_paise - cash - req.tip_paise).max(0);
        store.record_settlement(&sale.id, req.tendered_paise, change, req.tip_paise, "cash", actor)?;
    }
    Ok(sale)
}

/// Client table UUIDs come from an older snapshot, so match the Server's own
/// table by name first and only fall back to the id the terminal sent.
fn resolve_table_id(
    store: &Store,
    table_name: Option<&str>,
    table_id: Option<&str>,
) -> Result<Option<String>, String> {
    let tables = store.list_tables()?;
    if let Some(name) = table_name.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(t) = tables.iter().find(|t| t.name.eq_ignore_ascii_case(name)) {
            return Ok(Some(t.id.clone()));
        }
        if let Some(t) = tables.iter().find(|t| t.id == name) {
            return Ok(Some(t.id.clone()));
        }
        return Ok(None);
    }
    Ok(table_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|id| tables.iter().find(|t| t.id == id).map(|t| t.id.clone())))
}

fn write_http(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    gzip: bool,
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        _ => "Error",
    };
    let (body, encoding) = if gzip && body.len() >= GZIP_FROM_BYTES {
        (gzipped(body.as_bytes())?, "Content-Encoding: gzip\r\n")
    } else {
        (body.as_bytes().to_vec(), "")
    };
    // Every reply carries the Server's clock, so a device can tell its own is wrong.
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\n{encoding}{HEADER_TIME}: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        chrono::Utc::now().timestamp(),
        body.len()
    );
    stream.write_all(head.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&body).map_err(|e| e.to_string())?;
    Ok(())
}

/// Guest / secondary-screen pages on the Server LAN (no device token).
fn public_lan_page(store: &Store, req: &Request) -> Result<Option<Reply>, String> {
    let route = req.route();
    let path = req.path.split('?').next().unwrap_or(route);
    match (req.method.as_str(), path) {
        ("GET", "/display") => {
            let table = req.query("table").unwrap_or_default();
            let tid = if table.is_empty() {
                None
            } else {
                resolve_table_id(store, Some(table), None)?
            };
            Ok(Some(Reply::html(store.display_order_html(tid.as_deref())?)))
        }
        ("GET", "/tokens") => Ok(Some(Reply::html(store.tokens_ready_html()?))),
        ("GET", "/expo") => Ok(Some(Reply::html(store.expo_ready_html()?))),
        ("GET", "/kiosk") => Ok(Some(Reply::html(store.kiosk_html()?))),
        ("GET", "/feedback") => Ok(Some(Reply::html(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>Feedback</title><style>body{font-family:sans-serif;padding:1.5rem}button{margin:.25rem;padding:.75rem 1rem;font-size:1.2rem}</style></head>
<body><h1>How was your meal?</h1>
<form method="POST" action="/feedback"><input type="hidden" name="score" id="score" value="5"/>
<p><button type="button" onclick="document.getElementById('score').value=5">★★★★★</button>
<button type="button" onclick="document.getElementById('score').value=4">★★★★</button>
<button type="button" onclick="document.getElementById('score').value=3">★★★</button>
<button type="button" onclick="document.getElementById('score').value=2">★★</button>
<button type="button" onclick="document.getElementById('score').value=1">★</button></p>
<p><input name="note" placeholder="Optional note" style="width:100%;padding:.5rem"/></p>
<p><input name="table" placeholder="Table" style="width:100%;padding:.5rem"/></p>
<button type="submit">Send</button></form></body></html>"#
                .into(),
        ))),
        ("POST", "/feedback") => {
            let score = form_or_json_i64(&req.body, "score").unwrap_or(5);
            let note = form_or_json_str(&req.body, "note");
            let table = form_or_json_str(&req.body, "table");
            store.add_feedback(score, &note, &table)?;
            Ok(Some(Reply::html(
                "<!doctype html><html><body><h1>Thank you</h1></body></html>".into(),
            )))
        }
        ("POST", "/kiosk_order") => {
            #[derive(Deserialize)]
            struct Item {
                product_id: String,
            }
            #[derive(Deserialize)]
            struct Body {
                #[serde(default)]
                items: Vec<Item>,
            }
            let body: Body = serde_json::from_str(if req.body.trim().is_empty() {
                "{}"
            } else {
                &req.body
            })
            .unwrap_or(Body { items: Vec::new() });
            // Counter takeaway: seat on first Parcel / Takeaway table if any.
            let tables = store.list_tables()?;
            let tid = tables
                .iter()
                .find(|t| {
                    t.area.to_lowercase().contains("parcel")
                        || t.name.to_lowercase().contains("take")
                })
                .or_else(|| tables.first())
                .map(|t| t.id.clone());
            if let Some(tid) = tid {
                let ids: Vec<String> = body.items.into_iter().map(|i| i.product_id).collect();
                store.public_add_to_table(&tid, &ids)?;
                Ok(Some(Reply::json(serde_json::json!({
                    "ok": true,
                    "table_id": tid,
                    "note": "LAN kiosk — sent to counter order"
                }))))
            } else {
                Ok(Some(Reply::refuse(400, "No table for kiosk orders", "")))
            }
        }
        _ if path.starts_with("/t/") => {
            let rest = &path["/t/".len()..];
            if req.method == "GET" {
                let name = urlencoding_decode(rest);
                let tid = resolve_table_id(store, Some(&name), Some(&name))?
                    .ok_or_else(|| "Table not found".to_string())?;
                return Ok(Some(Reply::html(store.table_order_html(&tid)?)));
            }
            if req.method == "POST" && rest.contains("/add") {
                let table_part = rest.split("/add").next().unwrap_or("");
                let name = urlencoding_decode(table_part.trim_end_matches('/'));
                let tid = resolve_table_id(store, Some(&name), Some(&name))?
                    .ok_or_else(|| "Table not found".to_string())?;
                #[derive(Deserialize)]
                struct Item {
                    product_id: String,
                }
                #[derive(Deserialize)]
                struct Body {
                    #[serde(default)]
                    items: Vec<Item>,
                }
                let body: Body =
                    serde_json::from_str(if req.body.trim().is_empty() { "{}" } else { &req.body })
                        .unwrap_or(Body { items: Vec::new() });
                let ids: Vec<_> = body.items.into_iter().map(|i| i.product_id).collect();
                store.public_add_to_table(&tid, &ids)?;
                return Ok(Some(Reply::json(serde_json::json!({ "ok": true }))));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn form_or_json_str(body: &str, key: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            return s.to_string();
        }
    }
    body.split('&')
        .filter_map(|p| p.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| urlencoding_decode(v))
        .unwrap_or_default()
}

fn form_or_json_i64(body: &str, key: &str) -> Option<i64> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(n) = v.get(key).and_then(|x| x.as_i64()) {
            return Some(n);
        }
    }
    form_or_json_str(body, key).parse().ok()
}

fn urlencoding_decode(s: &str) -> String {
    let mut out = String::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                let hex = &s[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

fn gzipped(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::with_capacity(bytes.len() / 6), flate2::Compression::fast());
    encoder.write_all(bytes).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())
}

/// `192.168.1.10`, `192.168.1.10:7847` or a full URL → the URL requests go to.
pub fn base_url(host: &str) -> String {
    let host = host.trim().trim_end_matches('/');
    if host.starts_with("http://") || host.starts_with("https://") {
        host.to_string()
    } else if host.contains(':') {
        format!("http://{host}")
    } else {
        format!("http://{host}:{DEFAULT_PORT}")
    }
}

/// How a phone reaches its Server: the address, this phone's pairing token,
/// and the session of whoever is signed in (empty before sign-in).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Link {
    pub base: String,
    pub device: String,
    pub session: String,
    /// The Server's id, from pairing (empty from an older Server).
    pub server_id: String,
}

/// Why a phone's call did not go through.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanError {
    /// The Server could not be reached (Wi‑Fi down, the shop PC off or
    /// asleep). Nothing was refused; try again later.
    Offline(String),
    /// The Server answered no, in its own words. `code`: "pair_again",
    /// "sign_in_again", "wait", "wrong_pin" or empty.
    Refused { message: String, code: String },
}

impl LanError {
    pub fn is_offline(&self) -> bool {
        matches!(self, LanError::Offline(_))
    }

    pub fn code(&self) -> &str {
        match self {
            LanError::Offline(_) => "",
            LanError::Refused { code, .. } => code,
        }
    }
}

impl std::fmt::Display for LanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanError::Offline(_) => write!(f, "Can't reach the shop's server — check the Wi‑Fi and that the shop device is on"),
            LanError::Refused { message, .. } => write!(f, "{message}"),
        }
    }
}

impl From<LanError> for String {
    fn from(e: LanError) -> Self {
        e.to_string()
    }
}

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(3))
        .timeout(timeout)
        .build()
}

fn lan_error(e: ureq::Error) -> LanError {
    match e {
        ureq::Error::Status(_, resp) => {
            let body: serde_json::Value = resp.into_json().unwrap_or_default();
            LanError::Refused {
                message: body
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("The server refused that")
                    .to_string(),
                code: body.get("code").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            }
        }
        ureq::Error::Transport(t) => LanError::Offline(t.to_string()),
    }
}

/// The Server's clock on each reply (Unix seconds).
pub const HEADER_TIME: &str = "X-Kumkaj-Time";

/// A device clock this far off the Server's is said on its screen.
pub const CLOCK_WARN_SECS: i64 = 5 * 60;

static CLOCK_SKEW: std::sync::Mutex<Option<i64>> = std::sync::Mutex::new(None);

/// How far this device's clock is off the Server's (seconds; positive =
/// this device is ahead), from the last reply that said. `None` before.
pub fn clock_skew_secs() -> Option<i64> {
    *CLOCK_SKEW.lock().unwrap()
}

/// Device clock minus Server clock, from a reply's [`HEADER_TIME`].
pub fn skew_from(header: Option<&str>, device_now: i64) -> Option<i64> {
    header.and_then(|v| v.trim().parse::<i64>().ok()).map(|server| device_now - server)
}

fn note_server_time(resp: &ureq::Response) {
    if let Some(skew) = skew_from(resp.header(HEADER_TIME), chrono::Utc::now().timestamp()) {
        *CLOCK_SKEW.lock().unwrap() = Some(skew);
    }
}

fn reply_json<T: serde::de::DeserializeOwned>(resp: ureq::Response) -> Result<T, LanError> {
    note_server_time(&resp);
    // A cut-off reply is as good as no reply: the request id makes a retry safe.
    resp.into_json().map_err(|e| LanError::Offline(format!("reply: {e}")))
}

/// Pair with the code shown on the shop device. Returns the Server's
/// address, this phone's own token, and the first snapshot.
pub fn client_pair(host: &str, code: &str, device_name: &str) -> Result<(Link, SyncSnapshot), LanError> {
    let base = base_url(host);
    let resp = agent(Duration::from_secs(8))
        .post(&format!("{base}/pair"))
        .send_json(PairRequest { code: code.trim().to_string(), device_name: device_name.to_string() })
        .map_err(lan_error)?;
    let pair: PairOk = reply_json(resp)?;
    let link = Link { base, device: pair.device_token, session: String::new(), server_id: pair.server_id };
    let snap = client_sync(&link, "")?.ok_or_else(|| LanError::Offline("no books in the reply".into()))?;
    Ok((link, snap))
}

/// Sign in on the Server with a PIN typed on this phone.
pub fn client_login(link: &Link, staff_id: Option<&str>, pin: &str) -> Result<LoginOk, LanError> {
    let resp = agent(Duration::from_secs(8))
        .post(&format!("{}/login", link.base))
        .set(HEADER_DEVICE, &link.device)
        .send_json(LoginRequest { staff_id: staff_id.map(str::to_string), pin: pin.to_string() })
        .map_err(lan_error)?;
    reply_json(resp)
}

/// The Server's books — or `None` when they are still revision `have` (the
/// Server answers 304 and nothing is sent or rewritten). Empty `have`
/// always fetches them.
pub fn client_sync(link: &Link, have: &str) -> Result<Option<SyncSnapshot>, LanError> {
    let mut request = agent(Duration::from_secs(20))
        .get(&format!("{}/sync", link.base))
        .set(HEADER_DEVICE, &link.device);
    if !have.is_empty() {
        request = request.set(HEADER_IF_NONE_MATCH, have);
    }
    let resp = request.call().map_err(lan_error)?;
    if resp.status() == 304 {
        note_server_time(&resp);
        return Ok(None);
    }
    reply_json(resp).map(Some)
}

/// Wait (up to `secs`) until the Server's books are no longer revision
/// `rev`; returns the revision they have then. The Server answers the
/// moment something changes — how a device updates without polling.
pub fn client_wait(link: &Link, rev: &str, secs: u64) -> Result<String, LanError> {
    let resp = agent(Duration::from_secs(secs + 10))
        .get(&format!("{}/wait?rev={rev}&secs={secs}", link.base))
        .set(HEADER_DEVICE, &link.device)
        .call()
        .map_err(lan_error)?;
    let reply: serde_json::Value = reply_json(resp)?;
    Ok(reply["rev"].as_str().unwrap_or_default().to_string())
}

/// Send one write. `request_id` makes a retry safe: the Server answers a
/// repeat from its record instead of doing it again.
pub fn client_send(link: &Link, path: &str, body: &str, request_id: &str) -> Result<serde_json::Value, LanError> {
    let resp = agent(Duration::from_secs(15))
        .post(&format!("{}{path}", link.base))
        .set(HEADER_DEVICE, &link.device)
        .set(HEADER_SESSION, &link.session)
        .set(HEADER_REQUEST, request_id)
        .set("Content-Type", "application/json")
        .send_string(body)
        .map_err(lan_error)?;
    reply_json(resp)
}

/// Apply a sync snapshot into the local Client cache DB — as one
/// transaction. Row by row, the screen could read the menu or a khata
/// half-replaced (an empty menu for a moment), a failure midway left a
/// mix of old and new, and every row was its own disk sync (seconds on a
/// phone for a busy shop).
pub fn apply_snapshot(store: &Store, snap: &SyncSnapshot) -> Result<(), String> {
    store.in_transaction(|| {
        store.replace_catalog_from_sync(&snap.products)?;
        store.replace_menu_meta_from_sync(&snap.categories, &snap.stations)?;
        store.replace_customers_from_sync(&snap.customers)?;
        store.replace_credit_ledger_from_sync(&snap.ledger)?;
        store.replace_tables_from_sync(&snap.tables)?;
        store.replace_sales_cache_from_sync(&snap.recent_sales)?;
        store.replace_kots_from_sync(&snap.open_kots)?;
        store.replace_running_from_sync(&snap.running_orders)?;
        for s in &snap.staff {
            store.upsert_staff_cache(s)?;
        }
        if !snap.venue_title.trim().is_empty() {
            store.set_venue_title(&snap.venue_title)?;
        }
        if !snap.org_name.trim().is_empty() {
            store.set_org_name(&snap.org_name)?;
        }
        if !snap.fiscal.country_code.trim().is_empty() {
            store.replace_fiscal_from_sync(&snap.fiscal, &snap.tax_rates)?;
        }
        store.cache_server_day_totals(&snap.today, snap.outstanding_credit_paise)?;
        Ok(())
    })
}

// ---- Finding the shop's server on the Wi‑Fi --------------------------------

/// Answer "where is the server?" broadcasts while the LAN is on.
fn run_discovery_responder(db_path: &Path) {
    let Ok(socket) = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], DISCOVERY_PORT))) else {
        return;
    };
    let _ = socket.set_read_timeout(Some(Duration::from_millis(500)));
    let mut buf = [0u8; 256];
    while LAN_ENABLED.load(Ordering::SeqCst) {
        let Ok((n, from)) = socket.recv_from(&mut buf) else {
            continue;
        };
        if &buf[..n] != DISCOVERY_ASK.as_bytes() {
            continue;
        }
        let Ok(store) = Store::open_path(db_path) else {
            continue;
        };
        let reply = serde_json::json!({
            "app": DISCOVERY_APP,
            "venue": store.listing_name().unwrap_or_default(),
            "port": DEFAULT_PORT,
            "server_id": store.lan_server_id().unwrap_or_default(),
        });
        let _ = socket.send_to(reply.to_string().as_bytes(), from);
    }
}

/// This device's address on the local network (no packet is sent: a UDP
/// "connect" only picks the route). `None` when there is no network. The
/// Server shows it so a device can type it when nothing is found.
pub fn local_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("10.255.255.255:1").ok()?;
    match socket.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() => Some(ip),
        _ => None,
    }
}

/// Find Khaynjo shop servers on this Wi‑Fi: first a broadcast that servers
/// answer, then — when a router drops broadcasts — a quick knock on every
/// address of this /24 for the server port. Takes a few seconds at most.
pub fn discover_servers(wait: Duration) -> Vec<FoundServer> {
    let mut found = discover_by_broadcast(wait);
    if found.is_empty() {
        found = discover_by_scan();
    }
    found.sort_by(|a, b| a.host.cmp(&b.host));
    found.dedup_by(|a, b| a.host == b.host);
    found
}

/// A discovery or `/hello` answer from a Khaynjo server at `host`.
fn found_from(reply: &serde_json::Value, host: String) -> Option<FoundServer> {
    if reply.get("app").and_then(|a| a.as_str()) != Some(DISCOVERY_APP) {
        return None;
    }
    let text = |key: &str| reply.get(key).and_then(|v| v.as_str()).unwrap_or_default().to_string();
    Some(FoundServer { host, venue: text("venue"), server_id: text("server_id") })
}

fn discover_by_broadcast(wait: Duration) -> Vec<FoundServer> {
    let mut out = Vec::new();
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        return out;
    };
    let _ = socket.set_broadcast(true);
    let mut targets = vec![SocketAddr::from(([255, 255, 255, 255], DISCOVERY_PORT))];
    if let Some(ip) = local_ipv4() {
        let o = ip.octets();
        targets.push(SocketAddr::from(([o[0], o[1], o[2], 255], DISCOVERY_PORT)));
    }
    for t in &targets {
        let _ = socket.send_to(DISCOVERY_ASK.as_bytes(), t);
    }
    let deadline = std::time::Instant::now() + wait;
    let mut buf = [0u8; 1024];
    while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
        let _ = socket.set_read_timeout(Some(left.max(Duration::from_millis(10))));
        let Ok((n, from)) = socket.recv_from(&mut buf) else {
            break;
        };
        let Ok(reply) = serde_json::from_slice::<serde_json::Value>(&buf[..n]) else {
            continue;
        };
        let port = reply.get("port").and_then(|p| p.as_u64()).unwrap_or(DEFAULT_PORT as u64);
        out.extend(found_from(&reply, format!("{}:{port}", from.ip())));
    }
    out
}

fn discover_by_scan() -> Vec<FoundServer> {
    let Some(ip) = local_ipv4() else {
        return Vec::new();
    };
    let o = ip.octets();
    let addrs = (1..=254)
        .filter(|h| *h != o[3])
        .map(|h| SocketAddr::from(([o[0], o[1], o[2], h], DEFAULT_PORT)))
        .collect();
    scan_for_servers(addrs)
}

/// Knock on each address (16 at a time, 300 ms each) and ask the ones that
/// open for `/hello`. Routers that drop broadcasts still let this through.
fn scan_for_servers(addrs: Vec<SocketAddr>) -> Vec<FoundServer> {
    let found = Arc::new(std::sync::Mutex::new(Vec::new()));
    let workers: Vec<_> = addrs
        .chunks(16)
        .map(|chunk| {
            let chunk = chunk.to_vec();
            let found = Arc::clone(&found);
            thread::spawn(move || {
                for addr in chunk {
                    if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_err() {
                        continue;
                    }
                    let url = format!("http://{addr}/hello");
                    let Ok(resp) = ureq::get(&url).timeout(Duration::from_secs(2)).call() else {
                        continue;
                    };
                    let Ok(reply) = resp.into_json::<serde_json::Value>() else {
                        continue;
                    };
                    found.lock().unwrap().extend(found_from(&reply, addr.to_string()));
                }
            })
        })
        .collect();
    for w in workers {
        let _ = w.join();
    }
    let out = found.lock().unwrap().clone();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powershell_encoded_command_is_base64_of_utf16le() {
        assert_eq!(base64_utf16le("hi"), "aABpAA==");
        assert_eq!(base64_utf16le("a"), "YQA=");
        assert_eq!(base64_utf16le(""), "");
    }
    use crate::domain::{new_id, Cart, Money, Product, StaffRole};

    // ------------------------------------------------ the phone ↔ Server door

    fn http(method: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Request {
        Request {
            method: method.into(),
            path: path.into(),
            headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            body: body.into(),
        }
    }

    fn body(reply: &Reply) -> serde_json::Value {
        serde_json::from_str(&reply.body).unwrap()
    }

    /// A Server with a waiter (2222), a manager (9999), a ₹200 Thali and D1.
    fn lan_shop() -> (PathBuf, Store, Guard) {
        let path = std::env::temp_dir().join(format!("khaynjo-lan-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.add_staff("Wasim", StaffRole::Waiter, "Captain", "2222", &owner).unwrap();
        store.add_staff("Mona", StaffRole::Manager, "Manager", "9999", &owner).unwrap();
        store
            .upsert_product(&Product::new_basic(new_id(), "Thali", Money::from_rupees(200)))
            .unwrap();
        store.add_table_in_area("D1", "Dine In").unwrap();
        (path, store, Guard::default())
    }

    fn staff_id(store: &Store, name: &str) -> String {
        store.list_staff().unwrap().into_iter().find(|s| s.name == name).unwrap().id
    }

    /// Pair, then sign `name` in with `pin`: (device token, session).
    fn paired_phone(path: &Path, store: &Store, guard: &Guard, name: &str, pin: &str) -> (String, String) {
        let code = store.pairing_code().unwrap();
        let paired = answer(path, &http("POST", "/pair", &[], &format!(r#"{{"code":"{code}","device_name":"Tab 1"}}"#)), "10.0.0.5", guard);
        assert_eq!(paired.status, 200, "{}", paired.body);
        let device = body(&paired)["device_token"].as_str().unwrap().to_string();
        let login = format!(r#"{{"staff_id":"{}","pin":"{pin}"}}"#, staff_id(store, name));
        let signed = answer(path, &http("POST", "/login", &[(HEADER_DEVICE, &device)], &login), "10.0.0.5", guard);
        assert_eq!(signed.status, 200, "{}", signed.body);
        (device, body(&signed)["session"].as_str().unwrap().to_string())
    }

    #[test]
    fn wrong_pairing_codes_make_that_address_wait() {
        let (path, store, guard) = lan_shop();
        let wrong = if store.pairing_code().unwrap() == "000000" { "111111" } else { "000000" };
        let try_code = |code: &str, ip: &str| {
            answer(&path, &http("POST", "/pair", &[], &format!(r#"{{"code":"{code}"}}"#)), ip, &guard)
        };
        for _ in 0..GUARD_TRIES {
            assert_eq!(try_code(wrong, "10.0.0.9").status, 403);
        }
        // Even the right code waits now, from that address…
        let right = store.pairing_code().unwrap();
        let blocked = try_code(&right, "10.0.0.9");
        assert_eq!(blocked.status, 429);
        assert_eq!(body(&blocked)["code"], "wait");
        // …while the shop's own tablet elsewhere still pairs.
        assert_eq!(try_code(&right, "10.0.0.5").status, 200);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn phones_never_see_pin_hashes_and_sign_in_on_the_server() {
        let (path, store, guard) = lan_shop();
        // Without a pairing, nothing but the open probes answers.
        let sync = answer(&path, &http("GET", "/sync", &[], ""), "10.0.0.5", &guard);
        assert_eq!(sync.status, 401);
        assert_eq!(body(&sync)["code"], "pair_again");
        let (device, _) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let snap = answer(&path, &http("GET", "/sync", &[(HEADER_DEVICE, &device)], ""), "10.0.0.5", &guard);
        let snap: SyncSnapshot = serde_json::from_str(&snap.body).unwrap();
        assert!(!snap.staff.is_empty());
        assert!(snap.staff.iter().all(|s| s.pin_hash.is_empty() && s.password_hash.is_empty()));
        assert!(!snap_has_code(&snap, &store.pairing_code().unwrap()));
        // A wrong PIN is refused by the Server, and five of them wait.
        let wasim = staff_id(&store, "Wasim");
        let login = |pin: &str| {
            let b = format!(r#"{{"staff_id":"{wasim}","pin":"{pin}"}}"#);
            answer(&path, &http("POST", "/login", &[(HEADER_DEVICE, &device)], &b), "10.0.0.5", &guard)
        };
        for _ in 0..GUARD_TRIES {
            assert_eq!(body(&login("1234"))["code"], "wrong_pin");
        }
        assert_eq!(login("2222").status, 429, "the right PIN waits too, after five wrong ones");
        let _ = std::fs::remove_file(path);
    }

    fn snap_has_code(snap: &SyncSnapshot, code: &str) -> bool {
        serde_json::to_string(snap).unwrap().contains(&format!("\"{code}\""))
    }

    #[test]
    fn a_write_needs_a_sign_in_and_a_retry_is_done_once() {
        let (path, store, guard) = lan_shop();
        let (device, session) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let kot = serde_json::to_string(&RemoteKotRequest {
            draft: cart.to_kot_draft().unwrap(),
            table_name: Some("D1".into()),
            table_id: None,
        })
        .unwrap();
        // The phone says nothing about who it is: the session decides.
        let anonymous = answer(&path, &http("POST", "/kot", &[(HEADER_DEVICE, &device)], &kot), "10.0.0.5", &guard);
        assert_eq!(body(&anonymous)["code"], "sign_in_again");
        let send = || {
            answer(
                &path,
                &http("POST", "/kot", &[(HEADER_DEVICE, &device), (HEADER_SESSION, &session), (HEADER_REQUEST, "req-1")], &kot),
                "10.0.0.5",
                &guard,
            )
        };
        let first = send();
        assert_eq!(first.status, 200, "{}", first.body);
        // The reply was lost on the Wi‑Fi; the phone sends the same request again.
        let again = send();
        assert_eq!(again.body, first.body);
        let d1 = store.list_tables().unwrap().into_iter().find(|t| t.name == "D1").unwrap();
        let order = store.running_order(&d1.id).unwrap().unwrap();
        assert_eq!(order.lines.len(), 1, "one Thali, not two");
        let sends: Vec<_> = store.list_audit(50).unwrap().into_iter().filter(|a| a.action == "kot.send").collect();
        assert_eq!(sends.len(), 1);
        assert_eq!(sends[0].actor_name, "Wasim", "sent as the signed-in waiter");
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_forgotten_phone_is_refused_at_once() {
        let (path, store, guard) = lan_shop();
        let (device, _) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let id = store.list_lan_devices().unwrap()[0].id.clone();
        let owner = store.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap();
        store.forget_lan_device(&id, &owner).unwrap();
        let sync = answer(&path, &http("GET", "/sync", &[(HEADER_DEVICE, &device)], ""), "10.0.0.5", &guard);
        assert_eq!(body(&sync)["code"], "pair_again");
        // A new code: phones already paired would keep working, this one must pair again.
        let old = store.pairing_code().unwrap();
        assert_ne!(store.new_pairing_code(&owner).unwrap(), old);
        // Who joined, who was forgotten, who changed the code: all in the history.
        let history: Vec<_> = store.list_audit(50).unwrap().into_iter().map(|a| a.action).collect();
        for action in ["lan.pair", "lan.forget", "lan.new_code"] {
            assert!(history.iter().any(|a| a == action), "{action} missing from {history:?}");
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_phones_void_needs_a_managers_pin_checked_on_the_server() {
        let (path, store, guard) = lan_shop();
        let owner = store.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap();
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let mut books = Store::open_path(&path).unwrap();
        let sale = books
            .finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, None, &owner)
            .unwrap();
        let (device, session) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let void = |pin: &str, id: &str| {
            let b = serde_json::to_string(&RemoteSaleAction {
                sale_id: sale.id.clone(),
                reason: "Wrong table".into(),
                approval_pin: pin.into(),
            })
            .unwrap();
            answer(
                &path,
                &http("POST", "/void", &[(HEADER_DEVICE, &device), (HEADER_SESSION, &session), (HEADER_REQUEST, id)], &b),
                "10.0.0.5",
                &guard,
            )
        };
        assert_eq!(void("", "v1").status, 400, "a waiter alone may not void");
        assert_eq!(void("1234", "v2").status, 400, "nor with a made-up PIN");
        let ok = void("9999", "v3");
        assert_eq!(ok.status, 200, "{}", ok.body);
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// Lunch rush: a waiter's tablet sends KOTs, a manager's tablet takes
    /// counter payments and the counter PC bills — all at the same moment,
    /// each on its own connection as in the shop. Every write lands (no
    /// "database is locked"), each exactly once, and no two bills share a
    /// number.
    #[test]
    fn two_tablets_and_the_counter_writing_at_once_lose_and_double_nothing() {
        const EACH: usize = 12;
        let (path, store, guard) = lan_shop();
        let owner = store.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap();
        let (waiter_dev, waiter) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let (manager_dev, manager) = paired_phone(&path, &store, &guard, "Mona", "9999");
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let kot = serde_json::to_string(&RemoteKotRequest {
            draft: cart.to_kot_draft().unwrap(),
            table_name: Some("D1".into()),
            table_id: None,
        })
        .unwrap();
        let sale = serde_json::to_string(&RemoteSaleRequest {
            draft: cart.to_sale_draft().unwrap(),
            pay: "cash".into(),
            customer_id: None,
            customer_name: None,
            table_name: None,
            table_id: None,
            payments: Vec::new(),
            tendered_paise: 0,
            tip_paise: 0,
        })
        .unwrap();
        let draft = cart.to_sale_draft().unwrap();
        std::thread::scope(|s| {
            let tablet = |device: &str, session: &str, route: &'static str, body: &str, tag: &'static str| {
                let (path, guard) = (&path, &guard);
                let (device, session, body) = (device.to_string(), session.to_string(), body.to_string());
                move || {
                    for i in 0..EACH {
                        let id = format!("{tag}-{i}");
                        let headers = [(HEADER_DEVICE, device.as_str()), (HEADER_SESSION, session.as_str()), (HEADER_REQUEST, id.as_str())];
                        let first = answer(path, &http("POST", route, &headers, &body), "10.0.0.5", guard);
                        assert_eq!(first.status, 200, "{route} {i}: {}", first.body);
                        // Every other reply is "lost": the same request comes again.
                        if i % 2 == 0 {
                            let again = answer(path, &http("POST", route, &headers, &body), "10.0.0.5", guard);
                            assert_eq!(again.body, first.body, "{route} {i} retried");
                        }
                    }
                }
            };
            s.spawn(tablet(&waiter_dev, &waiter, "/kot", &kot, "kot"));
            s.spawn(tablet(&manager_dev, &manager, "/sale", &sale, "sale"));
            s.spawn(|| {
                let mut counter = Store::open_path(&path).unwrap();
                for _ in 0..EACH {
                    counter.finalize_sale(&draft, PayMethod::Cash, None, None, &owner).unwrap();
                }
            });
        });
        let sales = store.recent_sales(1000).unwrap();
        assert_eq!(sales.len(), 2 * EACH, "each payment once: the tablet's and the counter's");
        let bills: std::collections::HashSet<_> = sales.iter().map(|s| s.bill_no()).collect();
        assert_eq!(bills.len(), sales.len(), "no two bills share a number");
        let kots = store.list_audit(1000).unwrap().into_iter().filter(|a| a.action == "kot.send").count();
        assert_eq!(kots, EACH, "each KOT once");
        let d1 = store.list_tables().unwrap().into_iter().find(|t| t.name == "D1").unwrap();
        assert_eq!(store.running_order(&d1.id).unwrap().unwrap().lines.len(), EACH);
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// A full house: ten tablets, each at its own table, send KOTs and pull
    /// the books at the same moment, every other reply lost and sent again.
    /// Every KOT lands exactly once, on its own table, and every pull is
    /// answered — no "database is locked" with ten devices on the Wi‑Fi.
    #[test]
    fn ten_tablets_at_once_lose_and_double_nothing() {
        const TABLETS: usize = 10;
        const EACH: usize = 6;
        let (path, store, guard) = lan_shop();
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let draft = cart.to_kot_draft().unwrap();
        let tablets: Vec<(String, String, String)> = (0..TABLETS)
            .map(|n| {
                let table = format!("T{n}");
                store.add_table_in_area(&table, "Dine In").unwrap();
                let (device, session) = paired_phone(&path, &store, &guard, "Wasim", "2222");
                (device, session, table)
            })
            .collect();
        std::thread::scope(|s| {
            for (n, (device, session, table)) in tablets.iter().enumerate() {
                let (path, guard, draft) = (&path, &guard, &draft);
                s.spawn(move || {
                    let kot = serde_json::to_string(&RemoteKotRequest {
                        draft: draft.clone(),
                        table_name: Some(table.clone()),
                        table_id: None,
                    })
                    .unwrap();
                    for i in 0..EACH {
                        let id = format!("tab{n}-kot{i}");
                        let headers = [(HEADER_DEVICE, device.as_str()), (HEADER_SESSION, session.as_str()), (HEADER_REQUEST, id.as_str())];
                        let first = answer(path, &http("POST", "/kot", &headers, &kot), "10.0.0.5", guard);
                        assert_eq!(first.status, 200, "tablet {n} KOT {i}: {}", first.body);
                        if i % 2 == 0 {
                            let again = answer(path, &http("POST", "/kot", &headers, &kot), "10.0.0.5", guard);
                            assert_eq!(again.body, first.body, "tablet {n} KOT {i} retried");
                        }
                        let pulled = answer(path, &http("GET", "/sync", &[(HEADER_DEVICE, device.as_str())], ""), "10.0.0.5", guard);
                        assert_eq!(pulled.status, 200, "tablet {n} pull {i}: {}", pulled.body);
                    }
                });
            }
        });
        let kots = store.list_audit(10_000).unwrap().into_iter().filter(|a| a.action == "kot.send").count();
        assert_eq!(kots, TABLETS * EACH, "each KOT once");
        let tables = store.list_tables().unwrap();
        for (_, _, name) in &tablets {
            let table = tables.iter().find(|t| &t.name == name).unwrap();
            assert_eq!(store.running_order(&table.id).unwrap().unwrap().lines.len(), EACH, "{name}");
        }
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// Months of trading on one Server: 9,000 bills over 30 days (300 of
    /// them today), 1,000 customers, 300 dishes. Times what every device
    /// pays for on each change — the snapshot it pulls — and what the owner
    /// waits for: day close, today's totals, a backup. Opt-in (seeding takes
    /// a while): `cargo test -p kumkaj-khaynjo --release --lib -- --ignored
    /// months_of_trading --nocapture`.
    #[test]
    #[ignore]
    fn months_of_trading_stay_quick() {
        let (path, mut store, guard) = lan_shop();
        store.fast_writes_for_test();
        let owner = store.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap();
        for i in 0..300 {
            store
                .upsert_product(&Product::new_basic(new_id(), format!("Item {i}"), Money::from_rupees(50 + i)))
                .unwrap();
        }
        for i in 0..1000 {
            store.add_customer(&format!("Guest {i}"), &format!("98{i:08}")).unwrap();
        }
        let products = store.list_products(false).unwrap();
        let seeded = std::time::Instant::now();
        let now = chrono::Local::now();
        const BILLS: usize = 9000;
        for n in 0..BILLS {
            let mut cart = Cart::default();
            for k in 0..3 {
                cart.add_or_bump(&products[(n * 7 + k * 13) % products.len()]);
            }
            let sale = store.finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, None, &owner).unwrap();
            let days_ago = ((BILLS - 1 - n) / 300) as i64;
            if days_ago > 0 {
                store.backdate_sale_for_test(&sale.id, &(now - chrono::Duration::days(days_ago)).to_rfc3339());
            }
        }
        eprintln!("seeded {BILLS} bills in {:?}", seeded.elapsed());
        let mut slowest = std::time::Duration::ZERO;
        let mut timed = |what: &str, f: &mut dyn FnMut()| {
            let t = std::time::Instant::now();
            f();
            let took = t.elapsed();
            eprintln!("{what}: {took:?}");
            slowest = slowest.max(took);
        };
        let mut snap_bytes = 0;
        timed("snapshot", &mut || {
            snap_bytes = serde_json::to_vec(&build_snapshot(&store).unwrap()).unwrap().len();
        });
        eprintln!("snapshot size: {} KB", snap_bytes / 1024);
        // Every tablet pulls this on each change: it must not grow with the
        // months (it was 3 MB here while old kitchen tickets stayed open).
        assert!(snap_bytes < 600 * 1024, "snapshot {} KB", snap_bytes / 1024);
        let (device, _) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        timed("/sync answered", &mut || {
            let reply = answer(&path, &http("GET", "/sync", &[(HEADER_DEVICE, device.as_str())], ""), "10.0.0.5", &guard);
            assert_eq!(reply.status, 200);
        });
        timed("today's totals", &mut || {
            assert_eq!(store.today_totals().unwrap().sale_count, 300);
        });
        timed("day close", &mut || {
            store.day_close_summary().unwrap();
        });
        timed("day report", &mut || {
            store.restaurant_day_report_text().unwrap();
        });
        let backup = path.with_extension("backup.sqlite3");
        timed("backup", &mut || store.backup_to(&backup).unwrap());
        // Each of these happens while someone waits at the counter.
        assert!(slowest < std::time::Duration::from_secs(3), "slowest step took {slowest:?}");
        let _ = std::fs::remove_file(backup);
        let _ = std::fs::remove_file(path);
    }

    /// A Linux server's firewall is read the way ufw writes it: switched
    /// on in ufw.conf, our TCP port accepted in its rules.
    #[test]
    fn ufw_settings_are_read() {
        assert!(ufw_conf_enabled("# comment\nENABLED=yes\nLOGLEVEL=low\n"));
        assert!(!ufw_conf_enabled("ENABLED=no\n"));
        assert!(!ufw_conf_enabled("#ENABLED=yes\n"));
        let port = DEFAULT_PORT;
        let rules = format!(
            "### tuple ### allow tcp {port} 0.0.0.0/0 any 0.0.0.0/0 in\n-A ufw-user-input -p tcp --dport {port} -j ACCEPT\n"
        );
        assert!(ufw_rules_allow(&rules, port));
        assert!(!ufw_rules_allow(&rules, port + 1));
        let udp_only = format!("-A ufw-user-input -p udp --dport {port} -j ACCEPT\n");
        assert!(!ufw_rules_allow(&udp_only, port));
    }

    /// A device whose clock is wrong finds out: every reply carries the
    /// Server's clock, and the difference is read from it.
    #[test]
    fn replies_carry_the_server_clock() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let client = std::thread::spawn(move || {
            let mut raw = TcpStream::connect(addr).unwrap();
            let mut reply = Vec::new();
            raw.read_to_end(&mut reply).unwrap();
            String::from_utf8_lossy(&reply).to_string()
        });
        let (mut conn, _) = listener.accept().unwrap();
        write_http(&mut conn, 200, "application/json", "{}", false).unwrap();
        drop(conn);
        let head = client.join().unwrap();
        let line = head.lines().find(|l| l.starts_with(HEADER_TIME)).expect("time header");
        let server: i64 = line.split(':').nth(1).unwrap().trim().parse().unwrap();
        assert!((chrono::Utc::now().timestamp() - server).abs() < 5);
        assert_eq!(skew_from(Some("1000"), 1600), Some(600), "ten minutes ahead");
        assert_eq!(skew_from(Some(" 1000 "), 700), Some(-300));
        assert_eq!(skew_from(None, 700), None, "an older Server says nothing");
        assert_eq!(skew_from(Some("soon"), 700), None);
    }

    /// Wi‑Fi delivers a request in pieces; the Server reads it whole.
    #[test]
    fn a_request_in_several_packets_is_read_whole() {
        struct Trickle(Vec<u8>, usize);
        impl Read for Trickle {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let n = 7.min(buf.len()).min(self.0.len() - self.1);
                buf[..n].copy_from_slice(&self.0[self.1..self.1 + n]);
                self.1 += n;
                Ok(n)
            }
        }
        let body = format!(r#"{{"note":"{}"}}"#, "x".repeat(50_000));
        let raw = format!("POST /kot HTTP/1.1\r\nHost: a\r\nX-Kumkaj-Device: d1\r\nContent-Length: {}\r\n\r\n{body}", body.len());
        let req = read_request(&mut Trickle(raw.into_bytes(), 0)).unwrap();
        assert_eq!(req.route(), "/kot");
        assert_eq!(req.header("x-kumkaj-device"), Some("d1"));
        assert_eq!(req.body, body);
        // Cut short = refused, never half-parsed.
        let cut = "POST /kot HTTP/1.1\r\nContent-Length: 100\r\n\r\n{\"a\":";
        assert!(read_request(&mut Trickle(cut.as_bytes().to_vec(), 0)).is_err());
    }

    /// The waiter's tablet and the Server agree on the wire shape.
    #[test]
    fn running_actions_travel_as_tagged_json() {
        let req = RemoteRunningRequest {
            approval_pin: "9999".into(),
            request: RunningAction::Cancel {
                line_id: "l1".into(),
                qty_milli: Some(1000),
                reason: "Guest changed mind".into(),
                prepared: true,
            },
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["request"]["action"], "cancel");
        let back: RemoteRunningRequest = serde_json::from_value(json).unwrap();
        assert!(matches!(back.request, RunningAction::Cancel { prepared: true, .. }));
        // An older tablet that sends no PIN still parses.
        let old: RemoteRunningRequest = serde_json::from_str(
            r#"{"request":{"action":"print_bill","table_id":"t1"}}"#,
        )
        .unwrap();
        assert!(old.approval_pin.is_empty());
    }

    /// The LAN endpoint and the counter share one entry point, so a
    /// waiter's tablet obeys the same rights as the counter.
    #[test]
    fn table_actions_from_a_tablet_obey_the_servers_rights() {
        // Empty KOT routes skip print (and Capture); give the unassigned
        // kitchen a printer so the cancel slip is recorded.
        let mut printers = crate::print_share::DevicePrinters::default();
        let kitchen = printers.add_printer("Kitchen", "192.168.1.50");
        *printers.route_mut(crate::print_share::RouteKey::Kot { station_id: "" }) =
            vec![kitchen];
        crate::print_share::with_test_printers(printers, || {
            let path = std::env::temp_dir().join(format!("khaynjo-net-{}.sqlite3", new_id()));
            let store = Store::open_path(&path).unwrap();
            let owner = store.list_staff().unwrap().into_iter().next().unwrap();
            store.add_staff("Wasim", StaffRole::Waiter, "Captain", "2222", &owner).unwrap();
            store.add_staff("Mona", StaffRole::Manager, "Manager", "9999", &owner).unwrap();
            let waiter = store.list_staff().unwrap().into_iter().find(|s| s.name == "Wasim").unwrap();
            store
                .upsert_product(&Product::new_basic(new_id(), "Thali", Money::from_rupees(200)))
                .unwrap();
            let thali = store.list_products(false).unwrap().remove(0);
            let table = store.add_table_in_area("D1", "Dine In").unwrap();
            let mut cart = Cart::default();
            cart.add_or_bump(&thali);
            store.send_kot(&cart.to_kot_draft().unwrap(), Some(&table.id), &waiter).unwrap();
            let line = store.running_order(&table.id).unwrap().unwrap().lines[0].id.clone();

            let cancel = RunningAction::Cancel {
                line_id: line.clone(),
                qty_milli: None,
                reason: "Wrong punch".into(),
                prepared: false,
            };
            assert!(apply_running_action(&store, &waiter, None, &cancel).is_err());
            assert!(crate::print_share::take_captured().is_empty(), "a refused cancel prints nothing");
            apply_running_action(&store, &waiter, Some("9999"), &cancel).unwrap();
            // The kitchen's cancel slip printed once (captured, never the OS).
            let jobs = crate::print_share::take_captured();
            assert_eq!(jobs.len(), 1);
            assert!(jobs[0].text.contains("CANCEL") && jobs[0].text.contains("Thali"), "{}", jobs[0].text);
            // The only item was cancelled, so there is no bill to print.
            let print = RunningAction::PrintBill { table_id: table.id.clone() };
            assert!(apply_running_action(&store, &waiter, None, &print).is_err(), "nothing left to bill");
            // …and the table is free again on every tablet.
            let snapshot = build_snapshot(&store).unwrap();
            assert!(snapshot.running_orders.is_empty());
            let freed = store.list_tables().unwrap().into_iter().find(|t| t.id == table.id).unwrap();
            assert_eq!(freed.status, crate::domain::TableStatus::Available);
            let _ = std::fs::remove_file(path);
        });
    }

    /// Auto-print sends one paper per station, without prices; off means
    /// no paper. Checked against captured jobs — the OS printer is never
    /// touched by tests.
    #[test]
    fn sent_kots_print_one_paper_per_station_without_prices() {
        let path = std::env::temp_dir().join(format!("khaynjo-kotprint-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let tandoor = store.add_station("Tandoor").unwrap();
        let mut naan = Product::new_basic(new_id(), "Butter Naan", Money::from_rupees(45));
        naan.station_id = tandoor.id.clone();
        store.upsert_product(&naan).unwrap();
        store
            .upsert_product(&Product::new_basic(new_id(), "Chaas", Money::from_rupees(30)))
            .unwrap();
        let table = store.add_table_in_area("D1", "Dine In").unwrap();
        let mut cart = Cart::default();
        for p in store.list_products(false).unwrap() {
            cart.add_or_bump(&p);
        }
        let ids = store.send_kot(&cart.to_kot_draft().unwrap(), Some(&table.id), &owner).unwrap();
        assert_eq!(ids.len(), 2, "one KOT per station");

        // Each station (and the unassigned kitchen) needs its own route —
        // empty routes skip Capture, matching Settings.
        let mut printers = crate::print_share::DevicePrinters::default();
        let kitchen = printers.add_printer("Kitchen", "192.168.1.50");
        *printers.route_mut(crate::print_share::RouteKey::Kot { station_id: "" }) =
            vec![kitchen.clone()];
        *printers.route_mut(crate::print_share::RouteKey::Kot {
            station_id: &tandoor.id,
        }) = vec![kitchen];
        crate::print_share::with_test_printers(printers, || {
            let _ = crate::print_share::take_captured();
            assert_eq!(print_sent_kots(&store, &ids), None);
            let jobs = crate::print_share::take_captured();
            assert_eq!(jobs.len(), 2);
            assert!(jobs.iter().any(|j| j.text.contains("Station: Tandoor") && j.text.contains("Butter Naan")));
            assert!(jobs.iter().all(|j| j.text.contains("KOT No:") && j.text.contains("D1")));
            // No money on a kitchen ticket. (Not bare digits: the ticket's
            // time, e.g. 16:30, may contain them.)
            assert!(
                jobs.iter().all(|j| !j.text.contains('₹') && !j.text.contains("45.00") && !j.text.contains("30.00")),
                "no prices"
            );
            store.set_kot_auto_print(false).unwrap();
            assert_eq!(print_sent_kots(&store, &ids), None);
            assert!(crate::print_share::take_captured().is_empty());
        });
        let _ = std::fs::remove_file(path);
    }

    /// Stops a test's Server when dropped — also when an assertion fails,
    /// so a failing test fails instead of hanging.
    struct StopOnDrop<'a>(&'a AtomicBool);

    impl Drop for StopOnDrop<'_> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::SeqCst);
        }
    }

    /// A busy restaurant's year (30 000 bills, 2 000 customers with khata
    /// history) still syncs to a device in a blink. Before the indexes a
    /// pull took seconds per device; `hot_lookups_use_an_index_never_a_scan_of_every_row`
    /// is the exact guard, this the end-to-end check.
    #[test]
    fn a_busy_years_books_still_sync_quickly() {
        let path = std::env::temp_dir().join(format!("khaynjo-year-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        store.seed_busy_books_for_test(200, 150, 2000);
        let started = Instant::now();
        let (json, rev) = snapshot_json(&store).unwrap();
        let took = started.elapsed();
        assert!(took < Duration::from_millis(1500), "one pull took {took:?}");
        // Over the Wi‑Fi it travels gzipped: a fraction of the size.
        let small = gzipped(json.as_bytes()).unwrap();
        assert!(small.len() * 4 < json.len(), "{} → {} bytes", json.len(), small.len());
        // The same books, the same revision; a device holding it gets 304.
        assert_eq!(snapshot_json(&store).unwrap().1, rev);
        assert_eq!(sync_reply(&store, Some(&rev)).unwrap().status, 304);
        let _ = std::fs::remove_file(path);
    }

    /// A snapshot lands whole or not at all: one that fails partway (here a
    /// damaged table list) leaves the device's books as they were — never a
    /// menu half replaced.
    #[test]
    fn a_snapshot_lands_whole_or_not_at_all() {
        let (path, store, _) = lan_shop();
        let device_path = std::env::temp_dir().join(format!("khaynjo-device-{}.sqlite3", new_id()));
        let device = Store::open_path(&device_path).unwrap();
        let first = build_snapshot(&store).unwrap();
        apply_snapshot(&device, &first).unwrap();
        let mut broken = first.clone();
        broken.products.push(Product::new_basic(new_id(), "Chaas", Money::from_rupees(30)));
        let twin = broken.tables[0].clone();
        broken.tables.push(twin);
        assert!(apply_snapshot(&device, &broken).is_err(), "the same table twice cannot be written");
        let menu: Vec<_> = device.list_products(false).unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(menu, ["Thali"], "the menu is still the last whole snapshot's");
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(device_path);
    }

    /// The shop's real door: a Server on a spare port and eight devices
    /// pairing, signing in, sending KOTs and payments (each twice, as after
    /// a lost reply) and pulling the books at the same time, over real
    /// sockets — HTTP framing, gzip, 304 and `/wait` included. Every write
    /// lands exactly once.
    #[test]
    fn eight_devices_over_real_sockets_lose_and_double_nothing() {
        const DEVICES: usize = 8;
        const EACH: usize = 6;
        let (path, store, _) = lan_shop();
        // Enough customers that the books are worth gzipping.
        for i in 0..300 {
            store.find_or_create_customer(&format!("Guest {i}")).unwrap();
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let base = format!("http://{addr}");
        let running = AtomicBool::new(true);
        let code = store.pairing_code().unwrap();
        // A manager: takes orders and payments (a waiter may not take payment).
        let mona = staff_id(&store, "Mona");
        std::thread::scope(|s| {
            let db = path.clone();
            let running = &running;
            s.spawn(move || serve(listener, db, &|| running.load(Ordering::SeqCst)).unwrap());
            let _stop = StopOnDrop(running);
            let devices: Vec<_> = (0..DEVICES)
                .map(|d| {
                    let (base, code, mona) = (base.clone(), code.clone(), mona.clone());
                    s.spawn(move || {
                        let (mut link, snap) = client_pair(&base, &code, &format!("Tablet {d}")).unwrap();
                        assert!(!snap.rev.is_empty() && !link.server_id.is_empty());
                        link.session = client_login(&link, Some(&mona), "9999").unwrap().session;
                        let thali = snap.products.iter().find(|p| p.name == "Thali").unwrap().clone();
                        let mut cart = Cart::default();
                        cart.add_or_bump(&thali);
                        for i in 0..EACH {
                            let (route, body) = if i % 2 == 0 {
                                let kot = RemoteKotRequest {
                                    draft: cart.to_kot_draft().unwrap(),
                                    table_name: Some("D1".into()),
                                    table_id: None,
                                };
                                ("/kot", serde_json::to_string(&kot).unwrap())
                            } else {
                                let sale = RemoteSaleRequest {
                                    draft: cart.to_sale_draft().unwrap(),
                                    pay: "cash".into(),
                                    customer_id: None,
                                    customer_name: None,
                                    table_name: None,
                                    table_id: None,
                                    payments: Vec::new(),
                                    tendered_paise: 0,
                                    tip_paise: 0,
                                };
                                ("/sale", serde_json::to_string(&sale).unwrap())
                            };
                            let id = format!("device{d}-{i}");
                            let first = client_send(&link, route, &body, &id).unwrap();
                            // The reply was "lost": the same request again.
                            assert_eq!(client_send(&link, route, &body, &id).unwrap(), first);
                        }
                        assert!(client_sync(&link, "").unwrap().is_some());
                        link
                    })
                })
                .collect();
            let links: Vec<Link> = devices.into_iter().map(|h| h.join().unwrap()).collect();

            let books = Store::open_path(&path).unwrap();
            assert_eq!(books.recent_sales(1000).unwrap().len(), DEVICES * EACH / 2, "each payment once");
            let kots = books.list_audit(1000).unwrap().into_iter().filter(|a| a.action == "kot.send").count();
            assert_eq!(kots, DEVICES * EACH / 2, "each KOT once");
            assert_eq!(books.list_lan_devices().unwrap().len(), DEVICES);

            // Nothing changes now: a device holding the latest books gets
            // 304, and a wait is held open until it runs out.
            let latest = client_sync(&links[0], "").unwrap().unwrap();
            assert!(client_sync(&links[1], &latest.rev).unwrap().is_none(), "unchanged books: 304");
            let started = Instant::now();
            assert_eq!(client_wait(&links[2], &latest.rev, 1).unwrap(), latest.rev);
            assert!(started.elapsed() >= Duration::from_millis(900), "held open while nothing changed");

            // Big replies travel gzipped to a device that reads gzip.
            let mut raw = TcpStream::connect(addr).unwrap();
            write!(raw, "GET /sync HTTP/1.1\r\nHost: shop\r\n{HEADER_DEVICE}: {}\r\nAccept-Encoding: gzip\r\n\r\n", links[0].device)
                .unwrap();
            let mut reply = Vec::new();
            raw.read_to_end(&mut reply).unwrap();
            let head = String::from_utf8_lossy(&reply[..reply.windows(4).position(|w| w == b"\r\n\r\n").unwrap()]).to_string();
            assert!(head.starts_with("HTTP/1.1 200") && head.contains("Content-Encoding: gzip"), "{head}");
        });
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// A router that drops broadcasts still lets the scan through: knocking
    /// on each address finds the Server, with its id.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_scan_finds_a_server_that_hears_no_broadcasts() {
        let (path, store, _) = lan_shop();
        // On Linux all of 127.0.0.0/8 is this machine: a small "Wi‑Fi".
        let listener = TcpListener::bind("127.0.0.77:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let running = AtomicBool::new(true);
        std::thread::scope(|s| {
            let db = path.clone();
            let running = &running;
            s.spawn(move || serve(listener, db, &|| running.load(Ordering::SeqCst)).unwrap());
            let _stop = StopOnDrop(running);
            let addrs = (70..=85).map(|h| SocketAddr::from(([127, 0, 0, h], port))).collect();
            let found = scan_for_servers(addrs);
            assert_eq!(found.len(), 1, "{found:?}");
            assert_eq!(found[0].host, format!("127.0.0.77:{port}"));
            assert_eq!(found[0].server_id, store.lan_server_id().unwrap());
            // An unnamed restaurant is listed by the business name it saved, if any.
            assert_eq!(found[0].venue, store.listing_name().unwrap());
        });
        let _ = std::fs::remove_file(path);
    }
}

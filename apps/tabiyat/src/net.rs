//! Local LAN Client/Server sync for Kumkaj (no cloud).
//!
//! Server listens on `DEFAULT_PORT` and answers with JSON over HTTP/1.1.
//! Client uses `ureq` to pair, pull catalog/ledger/sales, and push sales/settlements.

use crate::diag::{self, Cat};
use crate::domain::{
    Category, CreditLedgerEntry, Customer, DayTotals, PayMethod, Product, Sale, SaleDraft,
    StaffMember, TaxRate,
};
use crate::fiscal::FiscalSettings;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const DEFAULT_PORT: u16 = 7877;

/// UDP port a waiter's device asks "where is the shop's server?" on. The
/// server answers from its own Wi‑Fi address — no internet, no typing an IP
/// (Petpooja's captain app scans for it the same way).
pub const DISCOVERY_PORT: u16 = 7878;
const DISCOVERY_ASK: &str = "TABIYAT_DISCOVER?";
const DISCOVERY_APP: &str = "tabiyat";

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static LAN_ENABLED: AtomicBool = AtomicBool::new(false);
/// Books file the live listener / discovery are bound to. `ensure_server`
/// must not keep advertising one shop while answering on another's books.
static SERVER_DB_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

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
    pub recent_sales: Vec<Sale>,
    #[serde(default)]
    pub today: DayTotals,
    #[serde(default)]
    pub outstanding_credit_paise: i64,
    #[serde(default)]
    pub day_close_text: String,
    /// Menu groupings of the Server's branch.
    #[serde(default)]
    pub categories: Vec<Category>,
    /// Server's branch / business identity, shown on the Client header.
    #[serde(default)]
    pub branch_name: String,
    #[serde(default)]
    pub org_name: String,
    /// Shop mark key (`custom:venue_icon.webp`) — bytes pulled via `/item-image`.
    /// Empty = use the default brand mark on the Client.
    #[serde(default)]
    pub venue_icon_key: String,
    /// Country pack in force on the Server, so a terminal prints the same tax
    /// wording, pay-rail name and business id as the till.
    #[serde(default)]
    pub fiscal: FiscalSettings,
    #[serde(default)]
    pub tax_rates: Vec<TaxRate>,
    /// The Server's own id (kept in its books): a device that lost the
    /// Server's address finds it again by this, never by its name.
    #[serde(default)]
    pub server_id: String,
    /// This snapshot's revision. A device sends it back (`If-None-Match`)
    /// and a Server whose books are the same answers 304 instead of
    /// sending them all again. Not part of what is hashed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rev: String,
    /// Monotonic tip from the live channel (`LAN_PROTOCOL` ≥ 2). Zero on
    /// older Servers / when unknown.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub seq: u64,
    /// Shop-wide Standard bill type size (portrait + landscape). Absent on
    /// older Servers — Client keeps its local cache.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard_print_em: Option<f32>,
    /// Bill face printed under the shop name (address / phone lines).
    #[serde(default)]
    pub bill_header: String,
    #[serde(default)]
    pub fssai_no: String,
    #[serde(default)]
    pub bill_footer: String,
    /// Customer-facing UPI for the payment QR (Owner-set on the Server).
    #[serde(default)]
    pub upi_vpa: String,
    #[serde(default)]
    pub upi_payee: String,
}

fn is_zero_u64(v: &u64) -> bool {
    *v == 0
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
    /// Split tenders. Empty means a single tender in `pay`.
    #[serde(default)]
    pub payments: Vec<RemoteTender>,
    /// Settle & Save: cash handed over, and a tip (staff's, not a sale).
    #[serde(default)]
    pub tendered_paise: i64,
    #[serde(default)]
    pub tip_paise: i64,
    /// Loyalty points to debit on the Server (Client already discounted the cart).
    #[serde(default)]
    pub redeem_loyalty_points: i64,
    /// Wallet paise to debit on the Server.
    #[serde(default)]
    pub redeem_wallet_paise: i64,
    /// Voucher code + paise to debit on the Server.
    #[serde(default)]
    pub redeem_voucher_code: String,
    #[serde(default)]
    pub redeem_voucher_paise: i64,
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

/// Correct a bill the Server owns. The Client never edits its own sales cache.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteAmendRequest {
    pub sale_id: String,
    pub draft: SaleDraft,
    pub pay: String,
    pub reason: String,
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

/// Client → Server: signed-in person changes their own unlock PIN.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteOwnPinRequest {
    pub pin: String,
}

/// Client → Server: signed-in person changes their optional login password.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteOwnPasswordRequest {
    pub password: String,
}

/// Headers a phone sends: its pairing, who is signed in, and the request's
/// own id (a retry with the same id is answered, not done again).
pub const HEADER_DEVICE: &str = "X-Kumkaj-Device";
pub const HEADER_SESSION: &str = "X-Kumkaj-Session";
pub const HEADER_REQUEST: &str = "X-Kumkaj-Request";
/// The snapshot revision a device already holds (`/sync` answers 304 when
/// the books are still that).
pub const HEADER_IF_NONE_MATCH: &str = "If-None-Match";
/// The LAN protocol both sides speak, on every request and every reply.
pub const HEADER_PROTOCOL: &str = "X-Kumkaj-Protocol";

/// What the Server and its devices say to each other (routes, bodies, the
/// snapshot). Bump it with any change an older build would misread; the
/// two sides then say "update the app" instead of failing quietly (an old
/// device read a newer snapshot as "no reply" and looked offline for ever).
/// Bumped to 2 for WebSocket `/ws` + revisioned deltas (`lan_ws.rs`).
/// Bumped to 3 for Client recovery KOT print (`POST /kot_reprint`).
/// Bumped to 4 for Client Fire course / Hold line / Waiter call + sale
/// loyalty/wallet/voucher redeem fields on `/sale`.
/// Older Clients keep working via `/wait` + `/sync`; mismatched peers get 426.
pub const LAN_PROTOCOL: u32 = 4;

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
pub const FIREWALL_RULE: &str = "Kumkaj Tabiyat LAN";

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
///
/// Binds TCP **before** discovery answers. A previous bug advertised the shop
/// on UDP while port 7847 was still held by another process — Clients found
/// the real venue name, typed the right pairing code, and got "Wrong pairing
/// code" from the wrong books on that port.
pub fn ensure_server(db_path: PathBuf) -> Result<(), String> {
    LAN_ENABLED.store(true, Ordering::SeqCst);
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        let same = SERVER_DB_PATH.lock().unwrap().as_ref() == Some(&db_path);
        if same {
            return Ok(());
        }
        // Different shop books — free the port and bind this one.
        stop_server();
        for _ in 0..100 {
            if !SERVER_RUNNING.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        if SERVER_RUNNING.load(Ordering::SeqCst) {
            return Err(format!(
                "Could not free port {DEFAULT_PORT} for this shop — restart the app"
            ));
        }
        *SERVER_DB_PATH.lock().unwrap() = None;
    }
    if SERVER_RUNNING.swap(true, Ordering::SeqCst) {
        let same = SERVER_DB_PATH.lock().unwrap().as_ref() == Some(&db_path);
        if same {
            return Ok(());
        }
        SERVER_RUNNING.store(false, Ordering::SeqCst);
        return Err("Shop LAN is starting — tap Allow extra tills again".into());
    }
    // Made now, so the first device to find this Server learns its id.
    if let Ok(store) = Store::open_path(&db_path) {
        let _ = store.lan_server_id();
    }
    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(e) => {
            SERVER_RUNNING.store(false, Ordering::SeqCst);
            return Err(format!(
                "Port {DEFAULT_PORT} is busy — close the other shop app on this computer, then Allow extra tills again ({e})"
            ));
        }
    };
    *SERVER_DB_PATH.lock().unwrap() = Some(db_path.clone());
    let venue_path = db_path.clone();
    let _ = thread::Builder::new()
        .name("tabiyat-discovery".into())
        .spawn(move || run_discovery_responder(&venue_path));
    let watch_path = db_path.clone();
    let _ = thread::Builder::new()
        .name("tabiyat-books-watch".into())
        .spawn(move || watch_books(&watch_path, &|| LAN_ENABLED.load(Ordering::SeqCst)));
    let _handle: JoinHandle<()> = thread::Builder::new()
        .name("kumkaj-lan".into())
        .spawn(move || {
            let _awake = KeepAwake::hold();
            if let Err(e) = serve(listener, db_path, &|| LAN_ENABLED.load(Ordering::SeqCst)) {
                eprintln!("Kumkaj LAN server stopped: {e}");
                diag::error(Cat::Lan, "lan_server_stopped", &e);
            }
            SERVER_RUNNING.store(false, Ordering::SeqCst);
            *SERVER_DB_PATH.lock().unwrap() = None;
        })
        .map_err(|e| {
            SERVER_RUNNING.store(false, Ordering::SeqCst);
            *SERVER_DB_PATH.lock().unwrap() = None;
            e.to_string()
        })?;
    Ok(())
}

/// Stop accepting extra tills. The shop books stay on this device (same role a
/// future cloud drain will sync). The listen thread exits on the next accept poll.
pub fn stop_server() {
    LAN_ENABLED.store(false, Ordering::SeqCst);
}

/// Answer devices on `listener` while `keep_going` says so, one thread per
/// request. The shop's listener and the tests' own (on a spare port) run
/// this same loop.
pub fn serve(listener: TcpListener, db_path: PathBuf, keep_going: &dyn Fn() -> bool) -> Result<(), String> {
    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;
    let guard = Arc::new(Guard::default());
    let active = Arc::new(AtomicUsize::new(0));
    while keep_going() {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                // Accepted sockets inherit non-blocking on some systems.
                let _ = stream.set_nonblocking(false);
                if active.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
                    // "Not now", said at once — the device tries again.
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
                    let busy = Reply::refuse(503, "The shop's server is busy — trying again", "retry");
                    let _ = write_http(&mut stream, busy.status, busy.content_type, &busy.body, false);
                    continue;
                }
                let path = db_path.clone();
                let guard = Arc::clone(&guard);
                let served = Served::start(&active);
                let _ = thread::spawn(move || {
                    let _served = served;
                    if let Err(e) = handle_client(stream, &path, &peer.ip().to_string(), &guard) {
                        eprintln!("Kumkaj LAN request error: {e}");
                        diag::warn(Cat::Lan, "lan_request_error", &e);
                    }
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                eprintln!("Kumkaj LAN accept: {e}");
                diag::warn(Cat::Lan, "lan_accept", &e.to_string());
                thread::sleep(Duration::from_millis(80));
            }
        }
    }
    Ok(())
}

/// Most requests served at once. A device holds one open (`/wait`) and
/// sends one more now and then: this covers a busy shop many times over,
/// while a flood from one misbehaving host cannot use up the Server's
/// threads (each connection is a thread).
const MAX_CONNECTIONS: usize = 64;

/// One connection being served, counted while it lives.
struct Served(Arc<AtomicUsize>);

impl Served {
    fn start(active: &Arc<AtomicUsize>) -> Self {
        active.fetch_add(1, Ordering::SeqCst);
        Self(Arc::clone(active))
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

// ---- Books revision: answer "has anything changed?" without the books ------

/// The Server's books as devices last saw them: per books file, a revision
/// that changes whenever anything on the snapshot does, and a bell for the
/// devices waiting on `/wait` (and for this device's own screens).
struct BooksWatch {
    revs: Mutex<std::collections::HashMap<PathBuf, String>>,
    changed: Condvar,
    hook: Mutex<Option<fn()>>,
    /// A write already rebuilt the revision — the watcher should look again
    /// without waiting out its poll sleep.
    nudge: AtomicBool,
}

fn books_watch() -> &'static BooksWatch {
    static WATCH: OnceLock<BooksWatch> = OnceLock::new();
    WATCH.get_or_init(|| BooksWatch {
        revs: Mutex::new(std::collections::HashMap::new()),
        changed: Condvar::new(),
        hook: Mutex::new(None),
        nudge: AtomicBool::new(false),
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
    watch.nudge.store(true, Ordering::Release);
    watch.changed.notify_all();
    if let Some(hook) = *watch.hook.lock().unwrap() {
        hook();
    }
}

/// Rebuild the snapshot revision now and wake every `/wait`er (and this
/// device's own screens). Call after any commit that changes what paired
/// devices see — do not leave them for the watcher's next poll.
pub fn publish_books(store: &Store) {
    if let Ok((json, rev)) = snapshot_json(store) {
        note_rev(store.db_path(), &rev);
        if let Ok(mut snap) = serde_json::from_str::<SyncSnapshot>(&json) {
            snap.rev = rev.clone();
            crate::lan_ws::on_books_changed(store.db_path(), &rev, &snap);
        }
    }
}

/// The current books as a snapshot, its JSON (without the revision) and
/// the revision.
pub fn snapshot_json_public(store: &Store) -> Result<(String, String), String> {
    snapshot_json(store)
}

fn snapshot_json(store: &Store) -> Result<(String, String), String> {
    let snap = build_snapshot(store)?;
    let json = serde_json::to_string(&snap).map_err(|e| e.to_string())?;
    let rev = snapshot_rev(&json);
    Ok((json, rev))
}

/// How often the watcher looks for new commits (one cheap read each time).
/// Interrupted early when `publish_books` / `note_rev` nudges the Condvar.
const WATCH_EVERY: Duration = Duration::from_millis(100);
/// Rebuilt at least this often anyway: the day can roll over, and a paid
/// table frees itself after a while, with no one writing.
const REBUILD_EVERY: Duration = Duration::from_secs(30);

/// Keep the books' revision current while `keep_going` says so. SQLite's
/// `data_version` moves whenever any other connection (the counter's
/// screen, a device's request) commits; only then is the snapshot rebuilt
/// and hashed, and waiting devices are woken only if it really changed.
/// Writes that already called `publish_books` also nudge this loop so a
/// second look does not wait out the full poll.
pub(crate) fn watch_books(db_path: &Path, keep_going: &dyn Fn() -> bool) {
    let Ok(store) = Store::open_path(db_path) else {
        return;
    };
    let mut seen = None;
    let mut built = Instant::now();
    while keep_going() {
        let nudged = books_watch().nudge.swap(false, Ordering::AcqRel);
        let version = store.data_version().ok();
        if nudged || version != seen || built.elapsed() >= REBUILD_EVERY {
            seen = version;
            built = Instant::now();
            if let Ok((json, rev)) = snapshot_json(&store) {
                note_rev(db_path, &rev);
                if let Ok(mut snap) = serde_json::from_str::<SyncSnapshot>(&json) {
                    snap.rev = rev.clone();
                    crate::lan_ws::on_books_changed(db_path, &rev, &snap);
                }
            }
        }
        let watch = books_watch();
        let revs = watch.revs.lock().unwrap();
        let (_guard, _) = watch.changed.wait_timeout(revs, WATCH_EVERY).unwrap();
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

/// `/item-image?key=…`: one dish/category photo as hex JSON (paired devices).
/// Sync carries keys only; Clients (esp. Android, no shelf pack) pull bytes here.
fn item_image_reply(store: &Store, key_raw: &str) -> Result<Reply, String> {
    let key = percent_decode(key_raw.trim());
    if key.is_empty() {
        return Ok(Reply::refuse(400, "image key required", ""));
    }
    // Only a photo file may leave this way — never `custom:../tabiyat.sqlite3`
    // (the books, PIN hashes included, sit one folder up).
    if !image_key_is_safe(&key) {
        return Ok(Reply::refuse(404, "unknown image key", ""));
    }
    let dir = store.item_images_dir()?;
    let path = match key.as_str() {
        k if k.starts_with("custom:") => {
            let name = &k["custom:".len()..];
            let item = dir.join(name);
            // Venue logo lives under brand_images/; dish photos under item_images/.
            if item.is_file() {
                item
            } else if let Ok(brand) = store.brand_images_dir() {
                brand.join(name)
            } else {
                item
            }
        }
        k if k.starts_with("shelf:") => crate::menu_images::resolve_image_path(Some(k), "", &dir),
        _ => {
            return Ok(Reply::refuse(404, "unknown image key", ""));
        }
    };
    if !path.is_file() {
        return Ok(Reply::refuse(404, "photo not on this shop", ""));
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    if bytes.is_empty() || bytes.len() > 4 * 1024 * 1024 {
        return Ok(Reply::refuse(404, "photo not on this shop", ""));
    }
    Ok(Reply::json(serde_json::json!({
        "ok": true,
        "hex": hex::encode(bytes),
    })))
}

/// `/item-image` keys the shop itself writes: `custom:<bare>.webp` (dish,
/// category, venue mark — one file name, no folder; older shops may still
/// hold a .png/.jpg) or `shelf:<folder>/<file>.webp` inside the bundled
/// catalogue. Anything that could climb out of those folders is refused.
fn image_key_is_safe(key: &str) -> bool {
    let plain = |part: &str| {
        !part.is_empty()
            && part.len() <= 128
            && !part.starts_with('.')
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    let photo = |name: &str| {
        name.rsplit_once('.')
            .is_some_and(|(_, ext)| crate::media::is_image_ext(ext))
    };
    if let Some(file) = key.strip_prefix("custom:") {
        return plain(file) && photo(file);
    }
    if let Some(rel) = key.strip_prefix(crate::menu_images::SHELF_KEY_PREFIX) {
        let parts: Vec<&str> = rel.split('/').collect();
        return (1..=3).contains(&parts.len())
            && parts.iter().all(|p| plain(p))
            && photo(rel);
    }
    false
}

/// Decode `%XX` and `+` in a query value (`shelf:a/b.webp` stays readable).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let h = |c: u8| -> Option<u8> {
                    match c {
                        b'0'..=b'9' => Some(c - b'0'),
                        b'a'..=b'f' => Some(c - b'a' + 10),
                        b'A'..=b'F' => Some(c - b'A' + 10),
                        _ => None,
                    }
                };
                if let (Some(a), Some(b)) = (h(bytes[i + 1]), h(bytes[i + 2])) {
                    out.push((a << 4) | b);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `/sync`: the books, or 304 when the device already has this revision.
fn sync_reply(store: &Store, have: Option<&str>) -> Result<Reply, String> {
    let (json, rev) = snapshot_json(store)?;
    note_rev(store.db_path(), &rev);
    if let Ok(mut snap) = serde_json::from_str::<SyncSnapshot>(&json) {
        snap.rev = rev.clone();
        crate::lan_ws::on_books_changed(store.db_path(), &rev, &snap);
    }
    if have.map(|h| h.trim().trim_matches('"')) == Some(rev.as_str()) {
        return Ok(Reply {
            status: 304,
            body: String::new(),
            content_type: "application/json",
        });
    }
    let seq = crate::lan_ws::tip_seq(store.db_path()).unwrap_or(0);
    // The revision goes in front; it was not part of what was hashed.
    let body = if seq > 0 {
        format!("{{\"rev\":\"{rev}\",\"seq\":{seq},{}", &json[1..])
    } else {
        format!("{{\"rev\":\"{rev}\",{}", &json[1..])
    };
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
                c.args(["--what=sleep:idle", "--who=Kumkaj Tabiyat", "--why=Devices are connected to this shop's server", "--mode=block"]);
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
        recent_sales,
        today: store.today_totals()?,
        outstanding_credit_paise: store.outstanding_credit_total()?,
        day_close_text: store.day_close_summary()?,
        categories: store.list_categories()?,
        branch_name: store.active_branch().map(|b| b.name).unwrap_or_default(),
        org_name: store.saved_org_name().unwrap_or_default(),
        venue_icon_key: store
            .venue_icon_key()?
            .unwrap_or_default(),
        fiscal: store.fiscal_settings()?,
        tax_rates: store.list_tax_rates()?,
        server_id: store.lan_server_id()?,
        rev: String::new(),
        seq: 0,
        standard_print_em: store.standard_print_em().ok(),
        bill_header: store.bill_header()?,
        fssai_no: store.fssai_no()?,
        bill_footer: store.bill_footer()?,
        upi_vpa: store.upi_vpa()?,
        upi_payee: store.upi_payee()?,
    })
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

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.is_empty())
    }

    /// The path without its query string.
    pub fn route(&self) -> &str {
        self.path.split('?').next().unwrap_or("/")
    }

    /// A query-string value (`/wait?rev=ab12&secs=25` → `query("rev")`).
    pub fn query(&self, key: &str) -> Option<&str> {
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
    pub fn refuse(status: u16, message: &str, code: &str) -> Self {
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
    let req = match read_request(&mut stream) {
        Ok(req) => req,
        Err(e) => {
            let reply = Reply::refuse(400, &e, "");
            return write_http(&mut stream, reply.status, reply.content_type, &reply.body, false);
        }
    };
    if crate::lan_ws::is_ws_upgrade(&req) {
        // Held open for the life of the Client — do not count against the
        // short HTTP read timeout forever (lan_ws sets its own).
        let _ = stream.set_read_timeout(None);
        return crate::lan_ws::handle_upgrade(stream, &req, db_path, peer, guard);
    }
    let gzip = req.accepts_gzip();
    let reply = answer(db_path, &req, peer, guard);
    write_http(&mut stream, reply.status, reply.content_type, &reply.body, gzip)
}

/// One write arriving over the live WebSocket (same rules as HTTP POST).
pub fn answer_ws_write(
    db_path: &Path,
    device_token: &str,
    session: &str,
    path: &str,
    body: &str,
    request_id: &str,
) -> Reply {
    let mut headers = vec![
        (HEADER_DEVICE.to_string(), device_token.to_string()),
        (HEADER_SESSION.to_string(), session.to_string()),
        (HEADER_PROTOCOL.to_string(), LAN_PROTOCOL.to_string()),
    ];
    if !request_id.is_empty() {
        headers.push((HEADER_REQUEST.to_string(), request_id.to_string()));
    }
    let req = Request::new("POST", path, headers, body);
    answer(db_path, &req, "ws", &Guard::default())
}

/// Answer one request. Every outcome is a JSON reply — a failure is never a
/// dropped connection the phone has to guess about. "Not now" (the books
/// busy, the disk full) is a 503 the device keeps its write for and sends
/// again; anything else is the Server's answer for good. A bug that panics
/// is a 500 — said once, never a request that hangs up on every retry and
/// holds up the device's whole queue behind it.
pub fn answer(db_path: &Path, req: &Request, peer: &str, guard: &Guard) -> Reply {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| answer_inner(db_path, req, peer, guard))) {
        Ok(Ok(reply)) => reply,
        Ok(Err(e)) if is_transient(&e) => Reply::refuse(503, &e, "retry"),
        Ok(Err(e)) => Reply::refuse(400, &e, ""),
        Err(_) => {
            diag::error(Cat::Lan, "lan_request_panic", req.route());
            Reply::refuse(
                500,
                "The shop's server could not do that (an error in the app) — tell the owner",
                "",
            )
        }
    }
}

/// Errors that mean "not now" rather than "no": the books were busy past
/// the wait, or the disk would not take the write. Nothing was done; the
/// same request may well go through in a moment.
fn is_transient(e: &str) -> bool {
    let e = e.to_ascii_lowercase();
    [
        "database is locked",
        "database table is locked",
        "disk i/o error",
        "database or disk is full",
        "unable to open database file",
    ]
    .iter()
    .any(|m| e.contains(m))
}

/// The device's protocol, from its request (none = this build's: devices
/// from before the header spoke protocol 1).
pub fn protocol_refusal_public(req: &Request) -> Option<Reply> {
    protocol_refusal(req)
}

fn protocol_refusal(req: &Request) -> Option<Reply> {
    let theirs = req.header(HEADER_PROTOCOL).map_or(Some(LAN_PROTOCOL), |v| v.trim().parse::<u32>().ok())?;
    if theirs == LAN_PROTOCOL {
        return None;
    }
    let message = if theirs < LAN_PROTOCOL {
        "This device runs an older Tabiyat than the shop's server — update the app on this device"
    } else {
        "The shop's server runs an older Tabiyat than this device — update the app on the server"
    };
    Some(Reply::refuse(426, message, "update_app"))
}

/// Where a bill finds its printer (docs/PRINTER.md → Bill tab).
pub const NO_BILL_PRINTER: &str =
    "Bill not printed — no bill printer. Settings → Bill & KOT format → Bill → Printers for the bill";

/// How long a device's write waits, after the books committed, for the
/// printer to confirm the paper. A working printer answers in well under a
/// second; a dead one takes ~8 s to give up, so past this the device hears
/// "still printing" (a warning) instead of holding the waiter's next KOT
/// behind it. The device's own wait is 15 s.
const PAPER_WAIT: Duration = Duration::from_secs(5);

/// Add what did not print to a device's reply, so the waiter's screen says
/// "not printed" instead of "reached the kitchen".
fn with_print_problem(reply: Reply, problems: &[String]) -> Reply {
    if problems.is_empty() || !(200..300).contains(&reply.status) {
        return reply;
    }
    let Ok(serde_json::Value::Object(mut body)) = serde_json::from_str::<serde_json::Value>(&reply.body) else {
        return reply;
    };
    body.insert("print_problem".into(), serde_json::Value::String(problems.join("; ")));
    Reply { body: serde_json::Value::Object(body).to_string(), ..reply }
}

fn answer_inner(db_path: &Path, req: &Request, peer: &str, guard: &Guard) -> Result<Reply, String> {
    let route = req.route();
    let store = Store::open_path(db_path)?;
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
                "protocol": LAN_PROTOCOL,
            })));
        }
        _ => {}
    }
    if let Some(refusal) = protocol_refusal(req) {
        return Ok(refusal);
    }
    if route == "/pair" {
        return pair(&store, req, peer, guard);
    }
    let Some(device) = store.lan_device_for_token(req.header(HEADER_DEVICE).unwrap_or_default())? else {
        return Ok(Reply::refuse(
            401,
            "This device is not paired with the shop any more — pair it again",
            "pair_again",
        ));
    };
    match route {
        "/sync" | "/today" => return sync_reply(&store, req.header(HEADER_IF_NONE_MATCH)),
        "/item-image" => return item_image_reply(&store, req.query("key").unwrap_or_default()),
        "/wait" => {
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
    // Paper queued while answering is waited for once the books have
    // committed.
    let paper = crate::print_share::ConfirmPaper::start();
    let reply = {
        // One device write is one unit of work, its reply record included:
        // a retry after a lost reply — or after a crash between the two —
        // is answered from the record, never done twice; and two devices
        // writing take turns instead of interleaving.
        let request = store.begin()?;
        if let Some(id) = request_id
            && let Some((status, body)) = store.lan_reply(id, &device.id)?
        {
            return Ok(Reply {
                status,
                body,
                content_type: "application/json",
            });
        }
        let attempt = store.begin()?;
        let reply = match write(&store, route, &req.body, &actor) {
            Ok(reply) => {
                attempt.commit().map_err(|e| e.to_string())?;
                reply
            }
            // Not done, and not refused: no record, the device asks again.
            Err(e) if is_transient(&e) => return Err(e),
            Err(e) => {
                // Whatever the refused write did before it stopped is undone.
                drop(attempt);
                Reply::refuse(400, &e, "")
            }
        };
        if let Some(id) = request_id
            && reply.status != 404
        {
            // A refusal is an answer too: a retry gets the same words.
            store.save_lan_reply(id, &device.id, reply.status, &reply.body)?;
        }
        request.commit().map_err(|e| e.to_string())?;
        reply
    };
    let mut problems: Vec<String> = Vec::new();
    // Wake `/wait`ers now — paired tablets must see the new sale without
    // waiting for the books watcher's next poll. The books are in; only the
    // paper is still on its way.
    if (200..300).contains(&reply.status) {
        publish_books(&store);
    }
    // The request's SQLite write lock is long gone: waiting here holds up
    // no other device.
    for problem in paper.finish(PAPER_WAIT) {
        if !problems.contains(&problem) {
            problems.push(problem);
        }
    }
    let reply = with_print_problem(reply, &problems);
    if !problems.is_empty()
        && let Some(id) = request_id
        && (200..300).contains(&reply.status)
    {
        // A retry after a lost reply hears about the paper too.
        let _ = store.save_lan_reply(id, &device.id, reply.status, &reply.body);
    }
    Ok(reply)
}

/// Pairing codes are six digits (leading zeros count). Strip junk and pad so
/// "012345" on the shop and "12345" typed on a phone still match.
pub fn normalize_pair_code(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
    if digits.is_empty() {
        return digits;
    }
    format!("{digits:0>6}")
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
    let asked_code = normalize_pair_code(&asked.code);
    let have_code = normalize_pair_code(&store.pairing_code()?);
    if asked_code.is_empty() || asked_code != have_code {
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

/// Credit collect on LAN: existing party only — never create from a name.
fn resolve_existing_settle_customer(
    store: &Store,
    req: &RemoteSettleRequest,
) -> Result<String, String> {
    if let Some(id) = req
        .customer_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return store
            .get_customer(id)?
            .filter(|c| c.active)
            .map(|c| c.id)
            .ok_or_else(|| "Customer not found — pick an existing party".to_string());
    }
    if let Some(name) = req
        .customer_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return store
            .find_customer_by_name(name)?
            .map(|c| c.id)
            .ok_or_else(|| "Customer not found — pick an existing party".to_string());
    }
    Err("Customer required".into())
}

/// A signed-in phone's write, done as `actor`.
fn write(store: &Store, route: &str, body: &str, actor: &StaffMember) -> Result<Reply, String> {
    match route {
        "/sale" => {
            let req: RemoteSaleRequest = serde_json::from_str(body).map_err(|e| format!("sale json: {e}"))?;
            Reply::of(&remote_sale(store, &req, actor)?)
        }
        "/settle" => {
            let req: RemoteSettleRequest = serde_json::from_str(body).map_err(|e| format!("settle json: {e}"))?;
            let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
            let customer_id = resolve_existing_settle_customer(store, &req)?;
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
        "/staff_pin" => {
            let req: RemoteOwnPinRequest =
                serde_json::from_str(body).map_err(|e| format!("staff_pin json: {e}"))?;
            store.set_staff_pin(&actor.id, &req.pin, actor)?;
            Ok(Reply::done())
        }
        "/staff_password" => {
            let req: RemoteOwnPasswordRequest =
                serde_json::from_str(body).map_err(|e| format!("staff_password json: {e}"))?;
            store.set_own_password(&req.password, actor)?;
            Ok(Reply::done())
        }
        "/books" => {
            let op: crate::books_op::BooksOp =
                serde_json::from_str(body).map_err(|e| format!("books json: {e}"))?;
            let message = store.apply_books_op(op, actor)?;
            Ok(Reply::json(serde_json::json!({ "ok": true, "message": message })))
        }
        _ => Ok(Reply::refuse(404, "not found", "")),
    }
}

fn remote_sale(store: &Store, req: &RemoteSaleRequest, actor: &StaffMember) -> Result<Sale, String> {
    let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
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
    let redeem = crate::store::SaleRedeem {
        loyalty_points: req.redeem_loyalty_points,
        wallet_paise: req.redeem_wallet_paise,
        voucher_code: req.redeem_voucher_code.clone(),
        voucher_paise: req.redeem_voucher_paise,
    };
    let needs_redeem = !redeem.is_empty();
    let sale = if req.payments.is_empty() {
        let customer_id = customer_for(
            store,
            pay.is_credit() || needs_redeem,
            if needs_redeem {
                "Loyalty / wallet / voucher"
            } else {
                "Credit sale"
            },
        )?;
        store.finalize_sale_redeem(
            &req.draft,
            pay,
            customer_id.as_deref(),
            actor,
            &redeem,
        )?
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
        let customer_id = customer_for(
            store,
            needs_credit || needs_redeem,
            if needs_redeem && !needs_credit {
                "Loyalty / wallet / voucher"
            } else {
                "Credit part"
            },
        )?;
        store.finalize_sale_tenders_redeem(
            &req.draft,
            &parts,
            customer_id.as_deref(),
            actor,
            &redeem,
        )?
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

pub fn write_http_public(
    stream: &mut impl Write,
    status: u16,
    content_type: &str,
    body: &str,
    gzip: bool,
) -> Result<(), String> {
    // TcpStream-specific path below; buffer via a temporary Vec for other Writers.
    let mut tmp = Vec::new();
    write_http_to(&mut tmp, status, content_type, body, gzip)?;
    stream.write_all(&tmp).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_http(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    gzip: bool,
) -> Result<(), String> {
    write_http_to(stream, status, content_type, body, gzip)
}

fn write_http_to(
    stream: &mut impl Write,
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
        426 => "Upgrade Required",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let (body, encoding) = if gzip && body.len() >= GZIP_FROM_BYTES {
        (gzipped(body.as_bytes())?, "Content-Encoding: gzip\r\n")
    } else {
        (body.as_bytes().to_vec(), "")
    };
    // Every reply carries the Server's clock, so a device can tell its own
    // is wrong, and the protocol it speaks.
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\n{encoding}{HEADER_TIME}: {}\r\n{HEADER_PROTOCOL}: {LAN_PROTOCOL}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
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
        ("GET", "/display") => Ok(Some(Reply::html(store.display_order_html()?))),
        ("GET", "/feedback") => Ok(Some(Reply::html(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>Feedback</title><style>body{font-family:sans-serif;padding:1.5rem}button{margin:.25rem;padding:.75rem 1rem;font-size:1.2rem}</style></head>
<body><h1>How was your visit?</h1>
<form method="POST" action="/feedback"><input type="hidden" name="score" id="score" value="5"/>
<p><button type="button" onclick="document.getElementById('score').value=5">★★★★★</button>
<button type="button" onclick="document.getElementById('score').value=4">★★★★</button>
<button type="button" onclick="document.getElementById('score').value=3">★★★</button>
<button type="button" onclick="document.getElementById('score').value=2">★★</button>
<button type="button" onclick="document.getElementById('score').value=1">★</button></p>
<p><input name="note" placeholder="Optional note" style="width:100%;padding:.5rem"/></p>
<button type="submit">Send</button></form></body></html>"#
                .into(),
        ))),
        ("POST", "/feedback") => {
            let score = form_or_json_i64(&req.body, "score").unwrap_or(5);
            let note = form_or_json_str(&req.body, "note");
            store.add_feedback(score, &note, "")?;
            Ok(Some(Reply::html(
                "<!doctype html><html><body><h1>Thank you</h1></body></html>".into(),
            )))
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

/// `%E0%AA%A1+1` → "ડ 1": bytes first, then UTF-8 — a guest's note or a
/// table named in Gujarati arrives whole, and a stray `%` never cuts a
/// character in two (that panicked the request).
fn urlencoding_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => match (hex(b[i + 1]), hex(b[i + 2])) {
                (Some(hi), Some(lo)) => {
                    out.push(hi << 4 | lo);
                    i += 2;
                }
                _ => out.push(b'%'),
            },
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
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
        ureq::Error::Status(status, resp) => {
            let body: serde_json::Value = resp.into_json().unwrap_or_default();
            // "Not now" (the books busy, the disk full): nothing was done or
            // refused — keep the write and send it again, like a lost reply.
            if status == 503 || body.get("code").and_then(|v| v.as_str()) == Some("retry") {
                let said = body.get("error").and_then(|v| v.as_str()).unwrap_or("server busy");
                return LanError::Offline(format!("the shop's server said: {said}"));
            }
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

/// A reply from a Server on another LAN protocol: said as "update the
/// app", never misread (a newer snapshot used to fail to parse and look
/// like no reply at all — "offline" for ever).
fn check_protocol(resp: &ureq::Response) -> Result<(), LanError> {
    let Some(theirs) = resp.header(HEADER_PROTOCOL).and_then(|v| v.trim().parse::<u32>().ok()) else {
        return Ok(());
    };
    if theirs == LAN_PROTOCOL {
        return Ok(());
    }
    let message = if theirs > LAN_PROTOCOL {
        "The shop's server runs a newer Tabiyat — update the app on this device"
    } else {
        "The shop's server runs an older Tabiyat — update the app on the server"
    };
    Err(LanError::Refused { message: message.into(), code: "update_app".into() })
}

fn reply_json<T: serde::de::DeserializeOwned>(resp: ureq::Response) -> Result<T, LanError> {
    note_server_time(&resp);
    check_protocol(&resp)?;
    // A cut-off reply is as good as no reply: the request id makes a retry safe.
    resp.into_json().map_err(|e| LanError::Offline(format!("reply: {e}")))
}

#[derive(Clone, Debug, Deserialize)]
struct HelloOk {
    #[serde(default)]
    app: String,
    #[serde(default)]
    venue: String,
    #[serde(default)]
    server_id: String,
}

fn client_hello(base: &str) -> Result<HelloOk, LanError> {
    let resp = agent(Duration::from_secs(5))
        .get(&format!("{base}/hello"))
        .set(HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())
        .call()
        .map_err(lan_error)?;
    reply_json(resp)
}

static PAIR_ABANDON: AtomicBool = AtomicBool::new(false);

/// Pair now sets this so `/hello`, `/pair`, and `/sync` stop between calls.
/// A request already on the wire still finishes; the result is then dropped.
pub fn set_pair_abandoned(abandoned: bool) {
    PAIR_ABANDON.store(abandoned, Ordering::SeqCst);
}

fn pair_stop() -> Result<(), LanError> {
    if PAIR_ABANDON.load(Ordering::SeqCst) {
        Err(LanError::Refused {
            message: "Cancelled".into(),
            code: "cancelled".into(),
        })
    } else {
        Ok(())
    }
}

/// Pair with the code shown on the shop device. Returns the Server's
/// address, this phone's own token, and the first snapshot.
///
/// `expect_server_id` is the id from Find-server when the person tapped a
/// row — `/hello` must match it so a busy port belonging to another process
/// cannot accept the right code for the wrong shop.
pub fn client_pair(
    host: &str,
    code: &str,
    device_name: &str,
    expect_server_id: Option<&str>,
) -> Result<(Link, SyncSnapshot), LanError> {
    let base = base_url(host);
    pair_stop()?;
    let hello = client_hello(&base)?;
    pair_stop()?;
    if !hello.app.is_empty() && hello.app != DISCOVERY_APP {
        return Err(LanError::Refused {
            message: "That address is not a Tabiyat shop server".into(),
            code: "wrong_shop".into(),
        });
    }
    if let Some(expect) = expect_server_id.map(str::trim).filter(|s| !s.is_empty()) {
        if hello.server_id != expect {
            let venue = hello.venue.trim();
            let message = if venue.is_empty() {
                "That address answered as a different shop than the one you picked — Find again and tap Use on the right row".to_string()
            } else {
                format!(
                    "That address answered as \"{venue}\", not the shop you picked — Find again and tap Use on the right row"
                )
            };
            return Err(LanError::Refused {
                message,
                code: "wrong_shop".into(),
            });
        }
    }
    let code = normalize_pair_code(code);
    if code.is_empty() {
        return Err(LanError::Refused {
            message: "Type the pairing code shown on the shop device".into(),
            code: String::new(),
        });
    }
    pair_stop()?;
    let resp = agent(Duration::from_secs(8))
        .post(&format!("{base}/pair"))
        .set(HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())
        .send_json(PairRequest {
            code,
            device_name: device_name.to_string(),
        })
        .map_err(lan_error)?;
    let pair: PairOk = reply_json(resp)?;
    let link = Link {
        base,
        device: pair.device_token,
        session: String::new(),
        server_id: pair.server_id,
    };
    pair_stop()?;
    let snap = client_sync(&link, "")?
        .ok_or_else(|| LanError::Offline("no books in the reply".into()))?;
    pair_stop()?;
    Ok((link, snap))
}

/// Sign in on the Server with a PIN typed on this phone.
pub fn client_login(link: &Link, staff_id: Option<&str>, pin: &str) -> Result<LoginOk, LanError> {
    let resp = agent(Duration::from_secs(8))
        .post(&format!("{}/login", link.base))
        .set(HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())
        .set(HEADER_DEVICE, &link.device)
        .send_json(LoginRequest { staff_id: staff_id.map(str::to_string), pin: pin.to_string() })
        .map_err(lan_error)?;
    reply_json(resp)
}

/// One menu photo from the Server (hex JSON). Empty/`Err` when missing.
pub fn client_fetch_item_image(link: &Link, key: &str) -> Result<Vec<u8>, LanError> {
    let url = format!(
        "{}/item-image?key={}",
        link.base,
        percent_encode(key.trim())
    );
    let resp = agent(Duration::from_secs(20))
        .get(&url)
        .set(HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())
        .set(HEADER_DEVICE, &link.device)
        .call()
        .map_err(lan_error)?;
    let value: serde_json::Value = reply_json(resp)?;
    if value.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(LanError::Refused {
            message: value
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("photo not on this shop")
                .to_string(),
            code: value
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    let hex = value
        .get("hex")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LanError::Refused {
            message: "photo reply had no bytes".into(),
            code: String::new(),
        })?;
    hex::decode(hex).map_err(|e| LanError::Refused {
        message: format!("photo reply corrupt: {e}"),
        code: String::new(),
    })
}

/// The Server's books — or `None` when they are still revision `have` (the
/// Server answers 304 and nothing is sent or rewritten). Empty `have`
/// always fetches them.
pub fn client_sync(link: &Link, have: &str) -> Result<Option<SyncSnapshot>, LanError> {
    let mut request = agent(Duration::from_secs(20))
        .get(&format!("{}/sync", link.base))
        .set(HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())
        .set(HEADER_DEVICE, &link.device);
    if !have.is_empty() {
        request = request.set(HEADER_IF_NONE_MATCH, have);
    }
    let resp = request.call().map_err(lan_error)?;
    if resp.status() == 304 {
        note_server_time(&resp);
        check_protocol(&resp)?;
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
        .set(HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())
        .set(HEADER_DEVICE, &link.device)
        .call()
        .map_err(lan_error)?;
    let reply: serde_json::Value = reply_json(resp)?;
    Ok(reply["rev"].as_str().unwrap_or_default().to_string())
}

/// Send one write. `request_id` makes a retry safe: the Server answers a
/// repeat from its record instead of doing it again.
pub fn client_send(link: &Link, path: &str, body: &str, request_id: &str) -> Result<serde_json::Value, LanError> {
    // Menu template / CSV imports encode photos and can run for minutes.
    let wait = if path == "/books" { 15 * 60 } else { 15 };
    let resp = agent(Duration::from_secs(wait))
        .post(&format!("{}{path}", link.base))
        .set(HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())
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
    // Switching shops: drop the previous Server's mark and LAN photo cache so
    // Lock/header never keep another restaurant's branding.
    store.prepare_client_for_server(&snap.server_id)?;
    store.in_transaction(|| {
        store.replace_catalog_from_sync(&snap.products)?;
        store.replace_menu_meta_from_sync(&snap.categories)?;
        store.replace_customers_from_sync(&snap.customers)?;
        store.replace_credit_ledger_from_sync(&snap.ledger)?;
        store.replace_sales_cache_from_sync(&snap.recent_sales)?;
        // Full replace — never leave another shop's people on Lock.
        store.replace_staff_cache_from_sync(&snap.staff)?;
        // Always write — empty title/org/logo means this shop has none (do not
        // keep the previous Server's identity on a re-pair).
        store.set_venue_title(&snap.venue_title)?;
        store.set_org_name_from_sync(&snap.org_name)?;
        let icon = snap.venue_icon_key.trim();
        store.set_venue_icon_key(if icon.is_empty() { None } else { Some(icon) })?;
        if !snap.fiscal.country_code.trim().is_empty() {
            store.replace_fiscal_from_sync(&snap.fiscal, &snap.tax_rates)?;
        }
        store.cache_server_day_totals(&snap.today, snap.outstanding_credit_paise)?;
        if let Some(em) = snap.standard_print_em {
            store.set_standard_print_em_from_sync(em)?;
        }
        // Always write — empty header/footer/UPI means this shop has none.
        store.set_bill_format_from_sync(
            &snap.bill_header,
            &snap.fssai_no,
            &snap.bill_footer,
            &snap.upi_vpa,
            &snap.upi_payee,
        )?;
        Ok(())
    })?;
    Ok(())
}

/// Pull the Server's shop mark into this Client's `brand_images/`. Call after
/// [`apply_snapshot`] while the pairing link is known.
pub fn client_pull_venue_icon(store: &Store, link: &Link, key: &str) -> Result<(), String> {
    let dir = store.brand_images_dir()?;
    let dest = dir.join("venue_icon.webp");
    let key = key.trim();
    if key.is_empty() {
        let _ = std::fs::remove_file(&dest);
        return Ok(());
    }
    match client_fetch_item_image(link, key) {
        Ok(bytes) if !bytes.is_empty() => {
            std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
            Ok(())
        }
        Ok(_) => Ok(()),
        Err(e) if e.is_offline() => Err(e.to_string()),
        Err(_) => {
            // Missing logo on the Server is fine — Client keeps the default mark.
            let _ = std::fs::remove_file(&dest);
            Ok(())
        }
    }
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

static DISCOVER_CANCEL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Ask [`discover_servers`] to stop early (partial results still return).
pub fn cancel_discover_servers() {
    DISCOVER_CANCEL.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn discover_cancelled() -> bool {
    DISCOVER_CANCEL.load(std::sync::atomic::Ordering::SeqCst)
}

/// Find Tabiyat shop servers on this Wi‑Fi: first a broadcast that servers
/// answer, then — when a router drops broadcasts — a quick knock on every
/// address of this /24 for the server port. Takes a few seconds at most.
pub fn discover_servers(wait: Duration) -> Vec<FoundServer> {
    DISCOVER_CANCEL.store(false, std::sync::atomic::Ordering::SeqCst);
    let mut found = discover_by_broadcast(wait);
    if found.is_empty() && !discover_cancelled() {
        found = discover_by_scan();
    }
    found.sort_by(|a, b| a.host.cmp(&b.host));
    found.dedup_by(|a, b| a.host == b.host);
    found
}

/// A discovery or `/hello` answer from a Tabiyat server at `host`.
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
        if discover_cancelled() {
            break;
        }
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
                    if discover_cancelled() {
                        break;
                    }
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
    fn item_image_keys_cannot_leave_the_photo_folders() {
        for ok in [
            "custom:venue_icon.webp",
            "custom:cat-6f1c2b9e-1a2b-4c3d-9e8f-0a1b2c3d4e5f.webp",
            "custom:old-photo.png",
            "shelf:south-indian/masala-dosa.webp",
        ] {
            assert!(image_key_is_safe(ok), "{ok}");
        }
        for bad in [
            "custom:../tabiyat.sqlite3",
            "custom:..\\tabiyat.sqlite3",
            "custom:sub/venue_icon.webp",
            "custom:/etc/passwd",
            "custom:.hidden.webp",
            "custom:tabiyat.sqlite3",
            "custom:",
            "shelf:../../tabiyat.sqlite3",
            "shelf:south-indian/../../x.webp",
            "shelf:/abs/x.webp",
            "other:x.webp",
        ] {
            assert!(!image_key_is_safe(bad), "{bad}");
        }
    }

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

    /// A Server with a cashier (2222), a manager (9999) and a ₹200 Thali.
    fn lan_shop() -> (PathBuf, Store, Guard) {
        let path = std::env::temp_dir().join(format!("tabiyat-lan-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.add_staff("Wasim", StaffRole::Cashier, "Cashier", "2222", &owner).unwrap();
        store.add_staff("Mona", StaffRole::Manager, "Manager", "9999", &owner).unwrap();
        store
            .upsert_product(&Product::new_basic(new_id(), "Thali", Money::from_rupees(200)))
            .unwrap();
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

    /// A device's cash sale of one Thali, as JSON for `/sale`.
    fn thali_sale(store: &Store) -> String {
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        serde_json::to_string(&RemoteSaleRequest {
            draft: cart.to_sale_draft().unwrap(),
            pay: "cash".into(),
            customer_id: None,
            customer_name: None,
            payments: Vec::new(),
            tendered_paise: 0,
            tip_paise: 0,
            redeem_loyalty_points: 0,
            redeem_wallet_paise: 0,
            redeem_voucher_code: String::new(),
            redeem_voucher_paise: 0,
        })
        .unwrap()
    }

    fn snap_has_code(snap: &SyncSnapshot, code: &str) -> bool {
        serde_json::to_string(snap).unwrap().contains(&format!("\"{code}\""))
    }

    #[test]
    fn a_write_needs_a_sign_in_and_a_retry_is_done_once() {
        let (path, store, guard) = lan_shop();
        let (device, session) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let sale = thali_sale(&store);
        // The phone says nothing about who it is: the session decides.
        let anonymous = answer(&path, &http("POST", "/sale", &[(HEADER_DEVICE, &device)], &sale), "10.0.0.5", &guard);
        assert_eq!(body(&anonymous)["code"], "sign_in_again");
        let send = || {
            answer(
                &path,
                &http("POST", "/sale", &[(HEADER_DEVICE, &device), (HEADER_SESSION, &session), (HEADER_REQUEST, "req-1")], &sale),
                "10.0.0.5",
                &guard,
            )
        };
        let first = send();
        assert_eq!(first.status, 200, "{}", first.body);
        // The reply was lost on the Wi‑Fi; the phone sends the same request again.
        let again = send();
        assert_eq!(again.body, first.body);
        let sales = store.recent_sales(10).unwrap();
        assert_eq!(sales.len(), 1, "one bill, not two");
        assert!(sales[0].created_by_name.starts_with("Wasim"), "billed as the signed-in cashier: {}", sales[0].created_by_name);
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// Books busy past the wait (a long write elsewhere) is "not now": a
    /// 503 the device keeps its payment for, with no reply recorded — so the
    /// same request, sent again once the books are free, is done (once).
    #[test]
    fn busy_books_are_a_retry_never_a_refusal() {
        let (path, store, guard) = lan_shop();
        let (device, session) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let sale = thali_sale(&store);
        let headers = [(HEADER_DEVICE, device.as_str()), (HEADER_SESSION, session.as_str()), (HEADER_REQUEST, "busy-1")];
        let send = || answer(&path, &http("POST", "/sale", &headers, &sale), "10.0.0.5", &guard);
        let holder = Store::open_path(&path).unwrap();
        let held = holder.begin().unwrap();
        let busy = send();
        assert_eq!(busy.status, 503, "{}", busy.body);
        assert_eq!(body(&busy)["code"], "retry");
        drop(held);
        let done = send();
        assert_eq!(done.status, 200, "{}", done.body);
        assert_eq!(send().body, done.body, "answered from the record");
        assert_eq!(store.recent_sales(10).unwrap().len(), 1);
        // On the device, a 503 keeps the write waiting like a lost reply.
        assert!(lan_error_for(503, &busy.body).is_offline());
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// A device's sync only reads: it never waits on (or fails behind) a
    /// write in progress elsewhere — the counter taking a payment, another
    /// tablet's KOT.
    #[test]
    fn a_sync_never_waits_on_a_writer() {
        let (path, store, guard) = lan_shop();
        let (device, _) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let holder = Store::open_path(&path).unwrap();
        let held = holder.begin().unwrap();
        let started = Instant::now();
        let sync = answer(&path, &http("GET", "/sync", &[(HEADER_DEVICE, &device)], ""), "10.0.0.5", &guard);
        assert_eq!(sync.status, 200, "{}", sync.body);
        assert!(started.elapsed() < Duration::from_secs(2), "took {:?}", started.elapsed());
        drop(held);
        let _ = std::fs::remove_file(path);
    }

    /// Guest pages post forms: Gujarati arrives whole, and a stray `%`
    /// before a non-ASCII letter is kept, not a panic.
    #[test]
    fn guest_form_text_arrives_whole() {
        assert_eq!(urlencoding_decode("%E0%AA%A1+1"), "ડ 1");
        assert_eq!(urlencoding_decode("100%"), "100%");
        assert_eq!(urlencoding_decode("%અ"), "%અ");
        assert_eq!(urlencoding_decode("a%2"), "a%2");
        assert_eq!(urlencoding_decode("%zz"), "%zz");
    }

    /// The device's side of a refusal, as `client_send` reads it.
    fn lan_error_for(status: u16, body: &str) -> LanError {
        let resp = ureq::Response::new(status, "x", body).unwrap();
        lan_error(ureq::Error::Status(status, resp))
    }

    /// Another version of the app on the other end is told plainly: update
    /// — never a reply misread as "offline". `/hello` still answers, with
    /// the protocol, so a scan lists the Server anyway.
    #[test]
    fn another_protocol_is_told_to_update_the_app() {
        let (path, store, guard) = lan_shop();
        let (device, _) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let newer = (LAN_PROTOCOL + 1).to_string();
        let sync = answer(&path, &http("GET", "/sync", &[(HEADER_DEVICE, &device), (HEADER_PROTOCOL, &newer)], ""), "10.0.0.5", &guard);
        assert_eq!(sync.status, 426);
        assert_eq!(body(&sync)["code"], "update_app");
        assert!(body(&sync)["error"].as_str().unwrap().contains("update the app on the server"));
        let pair = answer(&path, &http("POST", "/pair", &[(HEADER_PROTOCOL, "0")], "{}"), "10.0.0.5", &guard);
        assert_eq!(body(&pair)["code"], "update_app");
        let hello = answer(&path, &http("GET", "/hello", &[(HEADER_PROTOCOL, &newer)], ""), "10.0.0.5", &guard);
        assert_eq!(body(&hello)["protocol"], LAN_PROTOCOL);
        // Same protocol, or none said (a build from before the header): served.
        let same = answer(&path, &http("GET", "/sync", &[(HEADER_DEVICE, &device), (HEADER_PROTOCOL, &LAN_PROTOCOL.to_string())], ""), "10.0.0.5", &guard);
        assert_eq!(same.status, 200);
        assert_eq!(lan_error_for(426, &sync.body).code(), "update_app");
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
        let books = Store::open_path(&path).unwrap();
        let sale = books
            .finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, &owner)
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

    /// Rush hour: a cashier's tablet and a manager's tablet take payments
    /// and the counter PC bills — all at the same moment,
    /// each on its own connection as in the shop. Every write lands (no
    /// "database is locked"), each exactly once, and no two bills share a
    /// number.
    #[test]
    fn two_tablets_and_the_counter_writing_at_once_lose_and_double_nothing() {
        const EACH: usize = 12;
        let (path, store, guard) = lan_shop();
        let owner = store.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap();
        let (cashier_dev, cashier) = paired_phone(&path, &store, &guard, "Wasim", "2222");
        let (manager_dev, manager) = paired_phone(&path, &store, &guard, "Mona", "9999");
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let sale = thali_sale(&store);
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
            s.spawn(tablet(&cashier_dev, &cashier, "/sale", &sale, "cashier"));
            s.spawn(tablet(&manager_dev, &manager, "/sale", &sale, "manager"));
            s.spawn(|| {
                let counter = Store::open_path(&path).unwrap();
                for _ in 0..EACH {
                    counter.finalize_sale(&draft, PayMethod::Cash, None, &owner).unwrap();
                }
            });
        });
        let sales = store.recent_sales(1000).unwrap();
        assert_eq!(sales.len(), 3 * EACH, "each payment once: both tablets' and the counter's");
        let bills: std::collections::HashSet<_> = sales.iter().map(|s| s.bill_no()).collect();
        assert_eq!(bills.len(), sales.len(), "no two bills share a number");
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// A busy day: ten tills take payments and pull the books at the same
    /// moment, every other reply lost and sent again. Every payment lands
    /// exactly once and every pull is answered — no "database is locked"
    /// with ten devices on the Wi‑Fi.
    #[test]
    fn ten_tablets_at_once_lose_and_double_nothing() {
        const TABLETS: usize = 10;
        const EACH: usize = 6;
        let (path, store, guard) = lan_shop();
        let sale = thali_sale(&store);
        let tablets: Vec<(String, String)> = (0..TABLETS)
            .map(|_| paired_phone(&path, &store, &guard, "Wasim", "2222"))
            .collect();
        std::thread::scope(|s| {
            for (n, (device, session)) in tablets.iter().enumerate() {
                let (path, guard, sale) = (&path, &guard, &sale);
                s.spawn(move || {
                    for i in 0..EACH {
                        let id = format!("tab{n}-sale{i}");
                        let headers = [(HEADER_DEVICE, device.as_str()), (HEADER_SESSION, session.as_str()), (HEADER_REQUEST, id.as_str())];
                        let first = answer(path, &http("POST", "/sale", &headers, sale), "10.0.0.5", guard);
                        assert_eq!(first.status, 200, "till {n} sale {i}: {}", first.body);
                        if i % 2 == 0 {
                            let again = answer(path, &http("POST", "/sale", &headers, sale), "10.0.0.5", guard);
                            assert_eq!(again.body, first.body, "till {n} sale {i} retried");
                        }
                        let pulled = answer(path, &http("GET", "/sync", &[(HEADER_DEVICE, device.as_str())], ""), "10.0.0.5", guard);
                        assert_eq!(pulled.status, 200, "till {n} pull {i}: {}", pulled.body);
                    }
                });
            }
        });
        assert_eq!(store.recent_sales(10_000).unwrap().len(), TABLETS * EACH, "each payment once");
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    /// Months of trading on one Server: 9,000 bills over 30 days (300 of
    /// them today), 1,000 customers, 300 dishes. Times what every device
    /// pays for on each change — the snapshot it pulls — and what the owner
    /// waits for: day close, today's totals, a backup. Opt-in (seeding takes
    /// a while): `cargo test -p kumkaj-tabiyat --release --lib -- --ignored
    /// months_of_trading --nocapture`.
    #[test]
    #[ignore]
    fn months_of_trading_stay_quick() {
        let (path, store, guard) = lan_shop();
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
            let sale = store.finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, &owner).unwrap();
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
        let path = std::env::temp_dir().join(format!("tabiyat-year-{}.sqlite3", new_id()));
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

    /// Client sync carries the shop mark key; a second Server clears the
    /// previous shop's branding on re-pair.
    #[test]
    fn snapshot_carries_venue_icon_and_repair_clears_other_shop() {
        let (path, store, _) = lan_shop();
        store.set_venue_icon_key(Some("custom:venue_icon.webp")).unwrap();
        let brand = store.brand_images_dir().unwrap();
        std::fs::write(brand.join("venue_icon.webp"), b"webp-a").unwrap();
        let snap = build_snapshot(&store).unwrap();
        assert_eq!(snap.venue_icon_key, "custom:venue_icon.webp");

        let device_path =
            std::env::temp_dir().join(format!("tabiyat-device-logo-{}.sqlite3", new_id()));
        let device = Store::open_path(&device_path).unwrap();
        // Seed a leftover logo from a prior shop, then pair to this Server.
        device.set_paired_server_id("server-old").unwrap();
        let client_brand = device.brand_images_dir().unwrap().join("venue_icon.webp");
        std::fs::write(&client_brand, b"old-shop").unwrap();
        device.set_venue_icon_key(Some("custom:venue_icon.webp")).unwrap();

        let mut for_a = snap.clone();
        for_a.server_id = "server-a".into();
        apply_snapshot(&device, &for_a).unwrap();
        device.set_paired_server_id("server-a").unwrap();
        assert_eq!(
            device.venue_icon_key().unwrap().as_deref(),
            Some("custom:venue_icon.webp")
        );
        assert!(
            !client_brand.is_file(),
            "prepare must drop the previous shop's logo file"
        );

        let mut other = for_a.clone();
        other.server_id = "server-b".into();
        other.venue_title = "Other shop".into();
        other.venue_icon_key.clear();
        std::fs::write(&client_brand, b"should-go").unwrap();
        apply_snapshot(&device, &other).unwrap();
        assert!(device.venue_icon_key().unwrap().is_none());
        assert_eq!(device.venue_title().unwrap(), "Other shop");
        assert!(
            !client_brand.is_file(),
            "previous shop logo must not linger"
        );
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(device_path);
        let _ = std::fs::remove_dir_all(brand);
        let _ = std::fs::remove_dir_all(device.brand_images_dir().unwrap());
    }

    /// A snapshot lands whole or not at all: one that fails partway (here a
    /// damaged table list) leaves the device's books as they were — never a
    /// menu half replaced.
    #[test]
    fn a_snapshot_lands_whole_or_not_at_all() {
        let (path, store, _) = lan_shop();
        let owner = store.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap();
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        store.finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, &owner).unwrap();
        let device_path = std::env::temp_dir().join(format!("tabiyat-device-{}.sqlite3", new_id()));
        let device = Store::open_path(&device_path).unwrap();
        let first = build_snapshot(&store).unwrap();
        apply_snapshot(&device, &first).unwrap();
        let mut broken = first.clone();
        broken.products.push(Product::new_basic(new_id(), "Chaas", Money::from_rupees(30)));
        let twin = broken.recent_sales[0].clone();
        broken.recent_sales.push(twin);
        assert!(apply_snapshot(&device, &broken).is_err(), "the same bill twice cannot be written");
        let menu: Vec<_> = device.list_products(false).unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(menu, ["Thali"], "the menu is still the last whole snapshot's");
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(device_path);
    }

    /// The shop's real door: a Server on a spare port and eight devices
    /// pairing, signing in, sending payments (each twice, as after
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
        // A manager: takes payments.
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
                        let (mut link, snap) =
                            client_pair(&base, &code, &format!("Tablet {d}"), None).unwrap();
                        assert!(!snap.rev.is_empty() && !link.server_id.is_empty());
                        link.session = client_login(&link, Some(&mona), "9999").unwrap().session;
                        let thali = snap.products.iter().find(|p| p.name == "Thali").unwrap().clone();
                        let mut cart = Cart::default();
                        cart.add_or_bump(&thali);
                        for i in 0..EACH {
                            let sale = RemoteSaleRequest {
                                draft: cart.to_sale_draft().unwrap(),
                                pay: "cash".into(),
                                customer_id: None,
                                customer_name: None,
                                payments: Vec::new(),
                                tendered_paise: 0,
                                tip_paise: 0,
                                redeem_loyalty_points: 0,
                                redeem_wallet_paise: 0,
                                redeem_voucher_code: String::new(),
                                redeem_voucher_paise: 0,
                            };
                            let (route, body) = ("/sale", serde_json::to_string(&sale).unwrap());
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
            assert_eq!(books.recent_sales(1000).unwrap().len(), DEVICES * EACH, "each payment once");
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

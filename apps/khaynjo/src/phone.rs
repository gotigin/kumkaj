//! The phone's side of the shop LAN. One worker thread talks to the Server;
//! the screen never waits on the Wi‑Fi.
//!
//! - **Queued** writes (KOT, payment, "bill, please") are saved on the phone
//!   first (`lan_outbox`) and go to the Server in order — through a Wi‑Fi
//!   drop or an app restart. Each carries its own request id, so a retry
//!   after a lost reply is answered by the Server, not done twice. One the
//!   Server turns down when it finally arrives (the table was settled at
//!   the counter meanwhile) is kept on a "not accepted" list, with the
//!   Server's words, until someone has seen it — never dropped silently.
//! - **Now** writes (cancel, NC, move, print bill, void …) act on what the
//!   Server holds this moment, so they are never kept for later: without
//!   the Server they come back "can't reach the shop's server".
//! - The Server's books are pulled here too, and the screen is told when
//!   they landed. A second thread waits on the Server (`/wait`), so what
//!   another device did shows here at once; a pull the Server answers with
//!   "nothing changed" (304) sends and rewrites nothing.
//! - When the Server goes quiet, the worker looks for it on the Wi‑Fi by its
//!   id: a Server that came back on a new address (after a power cut, or the
//!   books moved to a new computer) is found again without pairing.

use crate::net::{self, FoundServer, LanError, Link, SyncSnapshot};
use crate::store::{LanQueued, Store};
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// What the screen does when a job's answer comes back.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Follow {
    /// Say `done` and refresh.
    #[default]
    Refresh,
    /// A payment: show its bill and offer Undo.
    Sale,
    /// A printed bill: keep its text as the last receipt.
    BillText,
    /// The order moved: open this table once the Server agrees.
    OpenTable(String),
}

/// One write for the Server.
#[derive(Clone, Debug, Default)]
pub struct Job {
    pub id: String,
    pub path: String,
    pub body: String,
    /// What the person did ("KOT for D3"), for the waiting list and errors.
    pub label: String,
    /// Status line once the Server agreed.
    pub done: String,
    pub follow: Follow,
    pub table_id: String,
    /// Whose sign-in sends it: a staff id, or "owner".
    pub staff_key: String,
    /// Keep it through a Wi‑Fi drop (KOT, payment); else it needs the Server now.
    pub queued: bool,
}

/// A job's answer, for the screen.
#[derive(Clone, Debug)]
pub struct Done {
    pub label: String,
    pub done: String,
    pub follow: Follow,
    pub reply: Result<serde_json::Value, LanError>,
}

/// How the link to the Server is, for the header badge.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkState {
    /// None until the first try.
    pub online: Option<bool>,
    /// Writes still waiting on this phone.
    pub waiting: usize,
    /// Waiting writes the Server would not accept (see the Connect screen).
    pub refused: usize,
    /// The Server forgot this phone: pair again.
    pub needs_pairing: bool,
    /// The sign-in that sends the waiting writes ran out: sign in again.
    pub needs_sign_in: bool,
    /// This device and the Server run app versions that cannot talk:
    /// update one of them. Waiting writes stay waiting meanwhile.
    pub needs_update: bool,
    /// When this device last knew it had the Server's latest books ("14:05";
    /// empty before the first time). Said while offline: how old the screen is.
    pub updated_at: String,
}

/// How the worker reaches the Server (the real Wi‑Fi, or a test's Server).
pub trait Transport: Send + Sync {
    fn send(&self, link: &Link, path: &str, body: &str, request_id: &str) -> Result<serde_json::Value, LanError>;
    /// The Server's books, or `None` while they are still revision `have`.
    fn sync(&self, link: &Link, have: &str) -> Result<Option<SyncSnapshot>, LanError>;
    /// Wait (up to `secs`) until the books are no longer revision `rev`.
    fn wait(&self, link: &Link, rev: &str, secs: u64) -> Result<String, LanError>;
    /// Shop servers answering on this Wi‑Fi now.
    fn find(&self) -> Vec<FoundServer>;
}

struct Wifi;

impl Transport for Wifi {
    fn send(&self, link: &Link, path: &str, body: &str, request_id: &str) -> Result<serde_json::Value, LanError> {
        net::client_send(link, path, body, request_id)
    }

    fn sync(&self, link: &Link, have: &str) -> Result<Option<SyncSnapshot>, LanError> {
        net::client_sync(link, have)
    }

    fn wait(&self, link: &Link, rev: &str, secs: u64) -> Result<String, LanError> {
        net::client_wait(link, rev, secs)
    }

    fn find(&self) -> Vec<FoundServer> {
        net::discover_servers(Duration::from_millis(1500))
    }
}

#[derive(Default)]
struct Shared {
    db_path: Option<PathBuf>,
    started: bool,
    /// "Now" jobs, in the order they were asked.
    now: VecDeque<Job>,
    /// Screen follow-ups for queued jobs (lost on restart: then a status line).
    follow: HashMap<String, (String, Follow)>,
    want_sync: bool,
    wake: bool,
    done: Vec<Done>,
    synced: bool,
    state: LinkState,
    notify: Option<fn()>,
    /// Revision of the books this phone holds (empty: fetch them whole).
    applied_rev: String,
    /// Where the waiting thread asks; set by the worker after each round.
    watch_link: Option<Link>,
    /// The Server has not answered since then.
    offline_since: Option<Instant>,
    /// When the worker last looked for the Server on the Wi‑Fi.
    last_find: Option<Instant>,
}

fn shared() -> &'static (Mutex<Shared>, Condvar) {
    static SHARED: OnceLock<(Mutex<Shared>, Condvar)> = OnceLock::new();
    SHARED.get_or_init(|| (Mutex::new(Shared::default()), Condvar::new()))
}

/// How often the worker looks again on its own (waiting writes, Wi‑Fi back).
const IDLE: Duration = Duration::from_secs(5);
/// How long one `/wait` is held open.
const WAIT_SECS: u64 = 25;
/// Silent this long, the Server is looked for on the Wi‑Fi by its id …
#[cfg(not(test))]
const REFIND_AFTER: Duration = Duration::from_secs(15);
#[cfg(test)]
const REFIND_AFTER: Duration = Duration::ZERO;
/// … and not again sooner than this (a search takes a few seconds).
#[cfg(not(test))]
const REFIND_EVERY: Duration = Duration::from_secs(60);
#[cfg(test)]
const REFIND_EVERY: Duration = Duration::ZERO;

/// Called (from the worker) whenever something changed for the screen.
pub fn set_notify(hook: fn()) {
    shared().0.lock().unwrap().notify = Some(hook);
}

/// Start the worker for this phone's books (once; later calls only wake it).
pub fn start(db_path: PathBuf) {
    let (lock, cvar) = shared();
    let mut sh = lock.lock().unwrap();
    sh.db_path = Some(db_path);
    sh.want_sync = true;
    sh.wake = true;
    if !sh.started {
        sh.started = true;
        let _ = std::thread::Builder::new()
            .name("khaynjo-phone".into())
            .spawn(|| worker(&Wifi));
        let _ = std::thread::Builder::new()
            .name("khaynjo-phone-watch".into())
            .spawn(|| watcher(&Wifi));
    }
    cvar.notify_all();
}

/// Hand a job to the worker. A queued job is on the phone's disk before
/// this returns.
pub fn send(store: &Store, job: Job) -> Result<(), String> {
    if job.queued {
        store.enqueue_lan(&LanQueued {
            id: job.id.clone(),
            path: job.path.clone(),
            body: job.body.clone(),
            label: job.label.clone(),
            table_id: job.table_id.clone(),
            staff_id: job.staff_key.clone(),
            ..Default::default()
        })?;
    }
    let (lock, cvar) = shared();
    let mut sh = lock.lock().unwrap();
    if job.queued {
        // The count is the worker's to change: it tells the screen when it does.
        sh.follow.insert(job.id.clone(), (job.done.clone(), job.follow.clone()));
    } else {
        sh.now.push_back(job);
    }
    sh.wake = true;
    cvar.notify_all();
    Ok(())
}

/// Ask for the Server's books (live screens call this on their timer).
pub fn want_sync() {
    let (lock, cvar) = shared();
    let mut sh = lock.lock().unwrap();
    sh.want_sync = true;
    sh.wake = true;
    cvar.notify_all();
}

/// Answers that came back since the last call.
pub fn take_done() -> Vec<Done> {
    std::mem::take(&mut shared().0.lock().unwrap().done)
}

/// True once after the Server's books landed in the phone's store.
pub fn take_synced() -> bool {
    std::mem::take(&mut shared().0.lock().unwrap().synced)
}

pub fn link_state() -> LinkState {
    shared().0.lock().unwrap().state.clone()
}

/// The person signed in again: waiting writes can go.
pub fn signed_in() {
    let (lock, cvar) = shared();
    let mut sh = lock.lock().unwrap();
    sh.state.needs_sign_in = false;
    sh.wake = true;
    cvar.notify_all();
}

/// Someone has read the "not accepted" list and dealt with it.
pub fn clear_refused(store: &Store) -> Result<(), String> {
    store.clear_lan_refused()?;
    shared().0.lock().unwrap().state.refused = 0;
    Ok(())
}

/// A different shop or a new pairing: the books held here are not that
/// Server's revision any more.
pub fn forget_revision() {
    shared().0.lock().unwrap().applied_rev.clear();
}

fn worker(transport: &dyn Transport) {
    let mut store: Option<(PathBuf, Store)> = None;
    let mut last_sync = Instant::now();
    loop {
        let (path, asked) = {
            let (lock, cvar) = shared();
            let mut sh = lock.lock().unwrap();
            if !sh.wake {
                sh = cvar.wait_timeout(sh, IDLE).unwrap().0;
            }
            // Woken on purpose: a write to send, a screen wanting the books,
            // a fresh sign-in.
            let woken = std::mem::take(&mut sh.wake);
            let interval = store.as_ref().and_then(|(_, s)| s.sync_interval_secs().ok()).unwrap_or(20).max(10);
            if sh.want_sync || last_sync.elapsed() >= Duration::from_secs(interval) {
                sh.want_sync = true;
                last_sync = Instant::now();
            }
            (sh.db_path.clone(), woken || sh.want_sync)
        };
        let Some(path) = path else {
            continue;
        };
        if store.as_ref().is_none_or(|(p, _)| *p != path) {
            store = Store::open_path(&path).ok().map(|s| (path.clone(), s));
        }
        let Some((_, books)) = store.as_ref() else {
            continue;
        };
        if !round_due(asked, link_state().online) {
            continue;
        }
        if pass(books, transport) {
            let hook = shared().0.lock().unwrap().notify;
            if let Some(hook) = hook {
                hook();
            }
        }
    }
}

/// Whether a worker round talks to the Server. Anything asked for goes at
/// once — a KOT, a refresh, the regular pull. Unasked rounds (the 5 s tick)
/// knock only while the Server answers: out of reach, the next pull or the
/// person's next action tries again, not every few seconds all evening.
/// (An earlier backoff also skipped asked-for rounds, so a tablet's KOT
/// could sit up to 20 s before reaching the kitchen.)
fn round_due(asked: bool, online: Option<bool>) -> bool {
    asked || online != Some(false)
}

/// The second thread: hold a `/wait` open on the Server and ask the worker
/// for the books the moment they change. Quiet while there is no Server to
/// ask; an older Server without `/wait` leaves the worker's own pull to it.
fn watcher(transport: &dyn Transport) {
    let mut seen = String::new();
    loop {
        let (link, applied) = {
            let sh = shared().0.lock().unwrap();
            (sh.watch_link.clone(), sh.applied_rev.clone())
        };
        let Some(link) = link else {
            std::thread::sleep(Duration::from_secs(2));
            continue;
        };
        if seen.is_empty() {
            seen = applied;
        }
        match transport.wait(&link, &seen, WAIT_SECS) {
            Ok(rev) if rev.is_empty() => std::thread::sleep(Duration::from_secs(20)),
            Ok(rev) => {
                if rev != seen {
                    seen = rev;
                    want_sync();
                }
            }
            Err(e) if e.is_offline() => std::thread::sleep(IDLE),
            Err(_) => std::thread::sleep(Duration::from_secs(60)),
        }
    }
}

fn link_of(store: &Store, staff_key: &str) -> Option<Link> {
    let base = store.server_url().ok()??;
    let device = store.client_device_token().ok()??;
    let session = store.saved_lan_session(staff_key).ok().flatten().unwrap_or_default();
    let server_id = store.paired_server_id().ok().flatten().unwrap_or_default();
    Some(Link { base, device, session, server_id })
}

/// The Server has been silent a while: look for it on the Wi‑Fi by its id
/// (never its name — two shops may share one). Exactly one answer from a
/// new address moves the pairing there; none, the same address, or two
/// computers claiming the same books (a copied backup running twice)
/// change nothing. True when the address changed.
fn refind_server(store: &Store, transport: &dyn Transport) -> bool {
    let (Ok(Some(id)), Ok(Some(base))) = (store.paired_server_id(), store.server_url()) else {
        return false;
    };
    {
        let mut sh = shared().0.lock().unwrap();
        let since = *sh.offline_since.get_or_insert_with(Instant::now);
        if since.elapsed() < REFIND_AFTER || sh.last_find.is_some_and(|t| t.elapsed() < REFIND_EVERY) {
            return false;
        }
        sh.last_find = Some(Instant::now());
    }
    let hosts: BTreeSet<String> = transport
        .find()
        .into_iter()
        .filter(|f| f.server_id == id)
        .map(|f| net::base_url(&f.host))
        .collect();
    match hosts.into_iter().collect::<Vec<_>>().as_slice() {
        [moved] if *moved != base => store.set_server_url(moved).is_ok(),
        _ => false,
    }
}

/// "14:05" today, "24 Oct 14:05" before.
fn when_text(at: &chrono::DateTime<chrono::Local>) -> String {
    if at.date_naive() == chrono::Local::now().date_naive() {
        at.format("%H:%M").to_string()
    } else {
        at.format("%d %b %H:%M").to_string()
    }
}

/// One round: waiting writes in order, then "now" writes, then the books.
/// Returns true when the screen has something new.
pub fn pass(store: &Store, transport: &dyn Transport) -> bool {
    let mut changed = false;
    let mut offline = false;
    let mut wrote = false;
    // "Pair again" / "sign in again" stand until the Server answers this
    // device again — a quiet round must not wipe them.
    let mut state = shared().0.lock().unwrap().state.clone();
    let mut reached = false;
    if state.updated_at.is_empty()
        && let Ok(Some(at)) = store.lan_synced_at()
        && let Ok(at) = chrono::DateTime::parse_from_rfc3339(&at)
    {
        // A restart while offline: say how old the books on this device are.
        state.updated_at = when_text(&at.with_timezone(&chrono::Local));
    }

    // 1. Waiting writes, oldest first. One that cannot go stops the rest:
    //    a payment must never reach the Server before its KOT.
    for item in store.lan_outbox().unwrap_or_default() {
        let Some(link) = link_of(store, &item.staff_id) else {
            break;
        };
        let reply = transport.send(&link, &item.path, &item.body, &item.id);
        match &reply {
            Err(LanError::Offline(e)) => {
                let _ = store.lan_outbox_retry_later(&item.id, e);
                offline = true;
                break;
            }
            Err(e) if e.code() == "sign_in_again" => {
                state.needs_sign_in = true;
                break;
            }
            Err(e) if e.code() == "pair_again" => {
                state.needs_pairing = true;
                break;
            }
            // Another version of the app on the Server: not a "no" to this
            // write — it waits (with the Server's words) until one is updated.
            Err(e) if e.code() == "update_app" => {
                let _ = store.lan_outbox_retry_later(&item.id, &e.to_string());
                state.needs_update = true;
                break;
            }
            // Turned down for good (it would be refused forever): off the
            // queue, onto the "not accepted" list with the Server's words.
            Err(e) => {
                let _ = store.lan_outbox_refused(&item.id, &e.to_string());
            }
            Ok(_) => {
                let _ = store.lan_outbox_done(&item.id);
            }
        }
        reached = true;
        state.needs_sign_in = false;
        state.needs_update = false;
        wrote = true;
        let (done, follow) = shared()
            .0
            .lock()
            .unwrap()
            .follow
            .remove(&item.id)
            .unwrap_or_else(|| (format!("{} — reached the shop", item.label), Follow::Refresh));
        shared().0.lock().unwrap().done.push(Done { label: item.label, done, follow, reply });
        changed = true;
    }

    // 2. "Now" writes. Behind a stuck queue they would act on an older
    //    order than the person sees, so they wait their turn — and fail
    //    plainly when the Server is out of reach.
    loop {
        let Some(job) = shared().0.lock().unwrap().now.pop_front() else {
            break;
        };
        let reply = if offline {
            Err(LanError::Offline("waiting writes ahead".into()))
        } else {
            match link_of(store, &job.staff_key) {
                Some(link) => transport.send(&link, &job.path, &job.body, &job.id),
                None => Err(LanError::Refused { message: "Pair this device with the shop first".into(), code: "pair_again".into() }),
            }
        };
        match &reply {
            Err(e) if e.is_offline() => offline = true,
            Err(e) if e.code() == "pair_again" => state.needs_pairing = true,
            Err(e) if e.code() == "update_app" => state.needs_update = true,
            Err(_) => reached = true,
            Ok(_) => {
                reached = true;
                wrote = true;
            }
        }
        shared().0.lock().unwrap().done.push(Done { label: job.label, done: job.done, follow: job.follow, reply });
        changed = true;
    }

    // 3. The Server's books, when asked or after a write — unless this
    //    phone already holds their latest revision.
    let want = std::mem::take(&mut shared().0.lock().unwrap().want_sync);
    if !offline && (want || wrote) {
        if let Some(link) = link_of(store, "") {
            let have = shared().0.lock().unwrap().applied_rev.clone();
            match transport.sync(&link, &have) {
                Ok(fresh) => {
                    reached = true;
                    let now = chrono::Local::now();
                    let _ = store.note_lan_synced(&now);
                    let _ = store.set_device_last_seen(&now.to_rfc3339());
                    state.updated_at = when_text(&now);
                    if let Some(snap) = fresh
                        && net::apply_snapshot(store, &snap).is_ok()
                    {
                        // Paired before Servers had ids: learn it now.
                        if !snap.server_id.is_empty() && matches!(store.paired_server_id(), Ok(None)) {
                            let _ = store.set_paired_server_id(&snap.server_id);
                        }
                        // Menu photos are keys only in the snapshot — pull bytes.
                        if let Some(link) = link_of(store, "") {
                            crate::lan_images::request_missing_from_store(store, &link);
                        }
                        let mut sh = shared().0.lock().unwrap();
                        sh.applied_rev = snap.rev;
                        sh.synced = true;
                        changed = true;
                    }
                }
                Err(e) if e.is_offline() => offline = true,
                Err(e) => {
                    state.needs_pairing |= e.code() == "pair_again";
                    state.needs_update |= e.code() == "update_app";
                }
            }
        }
    }

    if reached {
        state.needs_pairing = false;
        state.needs_update = false;
        shared().0.lock().unwrap().offline_since = None;
    }
    let tried = wrote || want || offline || store.lan_outbox().map(|o| !o.is_empty()).unwrap_or(false);
    if tried {
        state.online = Some(!offline && !state.needs_pairing);
    }
    // Gone quiet: perhaps the Server came back somewhere else.
    if offline && refind_server(store, transport) {
        let (lock, cvar) = shared();
        let mut sh = lock.lock().unwrap();
        sh.wake = true;
        sh.want_sync = true;
        cvar.notify_all();
    }
    state.waiting = store.lan_outbox().map(|o| o.len()).unwrap_or(0);
    state.refused = store.lan_refused().map(|r| r.len()).unwrap_or(0);
    let mut sh = shared().0.lock().unwrap();
    sh.watch_link = if state.needs_pairing { None } else { link_of(store, "") };
    if sh.state != state {
        sh.state = state;
        changed = true;
    }
    changed
}

#[cfg(test)]
pub(crate) fn reset_for_test() {
    let mut sh = shared().0.lock().unwrap();
    let notify = sh.notify;
    *sh = Shared { notify, ..Shared::default() };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{new_id, Cart, Money, Product, StaffRole};
    use crate::net::{Guard, RemoteKotRequest, Request};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// A write someone made goes on the very next round, however quiet the
    /// link was; only the unasked tick holds back while the Server is away.
    #[test]
    fn asked_rounds_are_never_held_back() {
        for online in [None, Some(true), Some(false)] {
            assert!(round_due(true, online));
        }
        assert!(round_due(false, None));
        assert!(round_due(false, Some(true)));
        assert!(!round_due(false, Some(false)));
    }

    /// The worker's state is one per process: these tests take turns.
    fn one_at_a_time() -> std::sync::MutexGuard<'static, ()> {
        static TURN: Mutex<()> = Mutex::new(());
        TURN.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The phone's "Wi‑Fi": straight into the Server's own request handler,
    /// or down.
    struct Bench {
        server: PathBuf,
        guard: Guard,
        up: AtomicBool,
        /// Drop the next reply after the Server did the work (lost on the air).
        lose_reply: AtomicBool,
        /// Where the Server answers; a phone knocking anywhere else hears nothing.
        at: Mutex<String>,
        /// What a search of the Wi‑Fi finds.
        found: Mutex<Vec<FoundServer>>,
    }

    impl Bench {
        fn call(&self, base: &str, method: &str, path: &str, headers: Vec<(String, String)>, body: &str) -> Result<serde_json::Value, LanError> {
            if !self.up.load(Ordering::SeqCst) || base != *self.at.lock().unwrap() {
                return Err(LanError::Offline("down".into()));
            }
            let reply = net::answer(&self.server, &Request::new(method, path, headers, body), "10.0.0.7", &self.guard);
            if self.lose_reply.swap(false, Ordering::SeqCst) {
                return Err(LanError::Offline("reply lost".into()));
            }
            if reply.status == 304 {
                return Ok(serde_json::Value::Null);
            }
            let value: serde_json::Value = serde_json::from_str(&reply.body).unwrap();
            if reply.status == 200 {
                Ok(value)
            } else {
                Err(LanError::Refused {
                    message: value["error"].as_str().unwrap_or_default().into(),
                    code: value["code"].as_str().unwrap_or_default().into(),
                })
            }
        }
    }

    impl Transport for Bench {
        fn send(&self, link: &Link, path: &str, body: &str, request_id: &str) -> Result<serde_json::Value, LanError> {
            let headers = vec![
                (net::HEADER_DEVICE.to_string(), link.device.clone()),
                (net::HEADER_SESSION.to_string(), link.session.clone()),
                (net::HEADER_REQUEST.to_string(), request_id.to_string()),
            ];
            self.call(&link.base, "POST", path, headers, body)
        }

        fn sync(&self, link: &Link, have: &str) -> Result<Option<SyncSnapshot>, LanError> {
            let mut headers = vec![(net::HEADER_DEVICE.to_string(), link.device.clone())];
            if !have.is_empty() {
                headers.push((net::HEADER_IF_NONE_MATCH.to_string(), have.to_string()));
            }
            let v = self.call(&link.base, "GET", "/sync", headers, "")?;
            Ok((!v.is_null()).then(|| serde_json::from_value(v).unwrap()))
        }

        fn wait(&self, link: &Link, rev: &str, secs: u64) -> Result<String, LanError> {
            let path = format!("/wait?rev={rev}&secs={secs}");
            let v = self.call(&link.base, "GET", &path, vec![(net::HEADER_DEVICE.to_string(), link.device.clone())], "")?;
            Ok(v["rev"].as_str().unwrap_or_default().to_string())
        }

        fn find(&self) -> Vec<FoundServer> {
            self.found.lock().unwrap().clone()
        }
    }

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("khaynjo-{name}-{}.sqlite3", new_id()))
    }

    /// A Server (waiter Wasim 2222, Thali ₹200, table D1) and a phone paired
    /// to it with Wasim signed in. Returns (bench, phone store, Wasim's id).
    fn shop_and_phone() -> (Bench, Store, String) {
        reset_for_test();
        let server_path = temp("server");
        let server = Store::open_path(&server_path).unwrap();
        let owner = server.list_staff().unwrap().into_iter().next().unwrap();
        server.add_staff("Wasim", StaffRole::Waiter, "Captain", "2222", &owner).unwrap();
        server.upsert_product(&Product::new_basic(new_id(), "Thali", Money::from_rupees(200))).unwrap();
        server.add_table_in_area("D1", "Dine In").unwrap();
        let bench = Bench {
            server: server_path,
            guard: Guard::default(),
            up: AtomicBool::new(true),
            lose_reply: AtomicBool::new(false),
            at: Mutex::new("http://shop".into()),
            found: Mutex::new(Vec::new()),
        };
        let code = server.pairing_code().unwrap();
        let paired = bench.call("http://shop", "POST", "/pair", vec![], &format!(r#"{{"code":"{code}"}}"#)).unwrap();
        let device = paired["device_token"].as_str().unwrap().to_string();
        let wasim = server.list_staff().unwrap().into_iter().find(|s| s.name == "Wasim").unwrap();
        let login = bench
            .call(
                "http://shop",
                "POST",
                "/login",
                vec![(net::HEADER_DEVICE.into(), device.clone())],
                &format!(r#"{{"staff_id":"{}","pin":"2222"}}"#, wasim.id),
            )
            .unwrap();
        let phone = Store::open_path(&temp("phone")).unwrap();
        phone.set_client_device_pairing("http://shop", &device).unwrap();
        phone.set_paired_server_id(paired["server_id"].as_str().unwrap()).unwrap();
        let ok: net::LoginOk = serde_json::from_value(login).unwrap();
        phone.remember_lan_sign_in(&ok.staff, "2222", &ok.session).unwrap();
        (bench, phone, wasim.id)
    }

    fn kot_job(server: &Path, staff_key: &str) -> Job {
        let books = Store::open_path(server).unwrap();
        let thali = books.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let body = serde_json::to_string(&RemoteKotRequest {
            draft: cart.to_kot_draft().unwrap(),
            table_name: Some("D1".into()),
            table_id: None,
        })
        .unwrap();
        Job {
            id: new_id(),
            path: "/kot".into(),
            body,
            label: "KOT for D1".into(),
            done: "KOT for D1 reached the kitchen".into(),
            staff_key: staff_key.into(),
            queued: true,
            ..Job::default()
        }
    }

    fn d1_lines(server: &Path) -> usize {
        let books = Store::open_path(server).unwrap();
        let d1 = books.list_tables().unwrap().into_iter().find(|t| t.name == "D1").unwrap();
        books.running_order(&d1.id).unwrap().map(|o| o.lines.len()).unwrap_or(0)
    }

    #[test]
    fn a_phone_keeps_orders_through_a_wifi_drop_and_sends_each_once() {
        let _turn = one_at_a_time();
        // Wi‑Fi down: the KOT waits on the phone, nothing is lost.
        let (bench, phone, wasim) = shop_and_phone();
        bench.up.store(false, Ordering::SeqCst);
        send(&phone, kot_job(&bench.server, &wasim)).unwrap();
        pass(&phone, &bench);
        assert_eq!(link_state().waiting, 1);
        assert_eq!(link_state().online, Some(false));
        assert_eq!(d1_lines(&bench.server), 0);
        assert!(take_done().is_empty(), "nothing to report yet");
        // A "now" action while down says so plainly instead of hanging.
        send(&phone, Job { id: new_id(), path: "/kot_advance".into(), body: r#"{"kot_id":"k"}"#.into(), staff_key: wasim.clone(), ..Job::default() }).unwrap();
        pass(&phone, &bench);
        let failed = take_done();
        assert_eq!(failed.len(), 1);
        assert!(failed[0].reply.as_ref().unwrap_err().is_offline());

        // Wi‑Fi back: the KOT goes, and the books come down.
        bench.up.store(true, Ordering::SeqCst);
        pass(&phone, &bench);
        assert_eq!(d1_lines(&bench.server), 1);
        let state = link_state();
        assert_eq!((state.online, state.waiting, state.refused), (Some(true), 0, 0));
        assert!(!state.updated_at.is_empty(), "the screen can say how fresh it is");
        let done = take_done();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].done, "KOT for D1 reached the kitchen");
        assert!(take_synced());
        assert!(phone.list_tables().unwrap().iter().any(|t| t.name == "D1"), "the Server's floor is on the phone");

        // The Server did the work but its reply was lost: the retry is
        // answered from the Server's record — still one KOT, not two.
        bench.lose_reply.store(true, Ordering::SeqCst);
        send(&phone, kot_job(&bench.server, &wasim)).unwrap();
        pass(&phone, &bench);
        assert_eq!(d1_lines(&bench.server), 2);
        assert_eq!(link_state().waiting, 1, "the phone could not tell it went");
        pass(&phone, &bench);
        assert_eq!(d1_lines(&bench.server), 2, "sent once");
        assert_eq!(link_state().waiting, 0);
        let answered = take_done();
        assert_eq!(answered.len(), 1);
        assert!(answered[0].reply.is_ok(), "the replayed answer is the first one");

        // A refusal leaves the queue (it would be refused forever) but is
        // never lost: it waits on the "not accepted" list, in the Server's
        // words, until someone has seen it.
        let mut bad = kot_job(&bench.server, &wasim);
        bad.body = r#"{"draft":{"lines":[]}}"#.into();
        send(&phone, bad).unwrap();
        pass(&phone, &bench);
        let refused = take_done();
        assert_eq!(refused.len(), 1);
        assert!(matches!(refused[0].reply, Err(LanError::Refused { .. })));
        assert_eq!(link_state().waiting, 0);
        assert_eq!(link_state().refused, 1);
        let kept = phone.lan_refused().unwrap();
        assert_eq!(kept[0].label, "KOT for D1");
        assert!(!kept[0].error.is_empty(), "the Server's words are kept");
        clear_refused(&phone).unwrap();
        assert_eq!(link_state().refused, 0);
        assert!(phone.lan_refused().unwrap().is_empty());

        // An expired sign-in keeps the order and asks to sign in again.
        send(&phone, kot_job(&bench.server, "someone-else")).unwrap();
        pass(&phone, &bench);
        assert!(link_state().needs_sign_in);
        assert_eq!(link_state().waiting, 1, "kept for after the sign-in");

        // The shop forgets this device: it is told to pair again, and a
        // quiet round later still says so.
        let books = Store::open_path(&bench.server).unwrap();
        let id = books.list_lan_devices().unwrap()[0].id.clone();
        let owner = books.list_staff().unwrap().into_iter().find(|s| s.is_owner()).unwrap();
        books.forget_lan_device(&id, &owner).unwrap();
        want_sync();
        pass(&phone, &bench);
        assert!(link_state().needs_pairing);
        pass(&phone, &bench);
        assert!(link_state().needs_pairing, "not wiped by a round with nothing to do");
        assert_eq!(link_state().online, Some(false));
        assert!(
            books.list_audit(20).unwrap().iter().any(|a| a.action == "lan.forget" && a.actor_name == owner.name),
            "who forgot which device is in the books' history"
        );
    }

    /// A pull while nothing changed sends and rewrites nothing; a change on
    /// the Server wakes a phone waiting on it at once — no 20 s poll.
    #[test]
    fn an_unchanged_server_sends_nothing_and_a_change_wakes_the_phone() {
        let _turn = one_at_a_time();
        let (bench, phone, _) = shop_and_phone();
        want_sync();
        pass(&phone, &bench);
        assert!(take_synced(), "the first pull brings the books");
        let rev = shared().0.lock().unwrap().applied_rev.clone();
        assert!(!rev.is_empty());
        want_sync();
        pass(&phone, &bench);
        assert!(!take_synced(), "the same books: 304, nothing sent or rewritten");

        let link = link_of(&phone, "").unwrap();
        assert_eq!(bench.wait(&link, &rev, 1).unwrap(), rev, "nothing changed: the wait runs out");
        let stop = AtomicBool::new(false);
        std::thread::scope(|s| {
            s.spawn(|| net::watch_books(&bench.server, &|| !stop.load(Ordering::SeqCst)));
            let woke = s.spawn(|| {
                let started = Instant::now();
                (bench.wait(&link, &rev, 20).unwrap(), started.elapsed())
            });
            std::thread::sleep(Duration::from_millis(500));
            // The counter adds a dish while the phone waits.
            Store::open_path(&bench.server)
                .unwrap()
                .upsert_product(&Product::new_basic(new_id(), "Chaas", Money::from_rupees(30)))
                .unwrap();
            let (now, took) = woke.join().unwrap();
            stop.store(true, Ordering::SeqCst);
            assert_ne!(now, rev);
            assert!(took < Duration::from_secs(10), "woken by the change, not the timeout ({took:?})");
        });
        want_sync();
        pass(&phone, &bench);
        assert!(take_synced());
        assert!(phone.list_products(false).unwrap().iter().any(|p| p.name == "Chaas"));
    }

    /// After a power cut the router gave the shop's computer a new address
    /// (or the books were restored on a new computer): the phone finds it
    /// again by its id — never its name — and carries on. Nothing is lost
    /// and nobody pairs again.
    #[test]
    fn a_server_on_a_new_address_is_found_again_by_its_id() {
        let _turn = one_at_a_time();
        let (bench, phone, wasim) = shop_and_phone();
        let id = Store::open_path(&bench.server).unwrap().lan_server_id().unwrap();
        *bench.at.lock().unwrap() = "http://192.168.1.23:7847".into();
        bench.found.lock().unwrap().extend([
            // The restaurant next door has the same name; not ours.
            FoundServer { host: "192.168.1.40:7847".into(), venue: "Rajdhani".into(), server_id: new_id() },
            FoundServer { host: "192.168.1.23:7847".into(), venue: "Rajdhani".into(), server_id: id },
        ]);
        send(&phone, kot_job(&bench.server, &wasim)).unwrap();
        pass(&phone, &bench);
        assert_eq!(phone.server_url().unwrap().as_deref(), Some("http://192.168.1.23:7847"));
        pass(&phone, &bench);
        assert_eq!(d1_lines(&bench.server), 1, "the waiting KOT reached the moved Server");
        assert_eq!(link_state().online, Some(true));
        assert_eq!(link_state().waiting, 0);
    }

    /// Two computers answering with the same books (a backup restored on a
    /// second computer while the first still runs): the phone does not
    /// guess between them; its orders keep waiting for a person to decide.
    #[test]
    fn two_computers_claiming_one_server_are_never_guessed_between() {
        let _turn = one_at_a_time();
        let (bench, phone, wasim) = shop_and_phone();
        let id = Store::open_path(&bench.server).unwrap().lan_server_id().unwrap();
        *bench.at.lock().unwrap() = "http://192.168.1.23:7847".into();
        bench.found.lock().unwrap().extend([
            FoundServer { host: "192.168.1.23:7847".into(), venue: String::new(), server_id: id.clone() },
            FoundServer { host: "192.168.1.24:7847".into(), venue: String::new(), server_id: id },
        ]);
        send(&phone, kot_job(&bench.server, &wasim)).unwrap();
        pass(&phone, &bench);
        assert_eq!(phone.server_url().unwrap().as_deref(), Some("http://shop"), "stays where it was");
        assert_eq!(link_state().waiting, 1, "the KOT keeps waiting — nothing lost");
        assert_eq!(d1_lines(&bench.server), 0);
    }

    /// Restarted while the Server is away, a device still says how old its
    /// books are: the time is kept in its own books, not only in memory.
    #[test]
    fn a_restart_while_offline_still_says_how_old_the_books_are() {
        let _turn = one_at_a_time();
        let (bench, phone, _) = shop_and_phone();
        want_sync();
        pass(&phone, &bench);
        let before = link_state().updated_at;
        assert!(!before.is_empty());
        assert!(phone.lan_synced_at().unwrap().is_some(), "kept in the device's books");
        // The app restarts; the Server has gone away meanwhile.
        reset_for_test();
        bench.up.store(false, Ordering::SeqCst);
        want_sync();
        pass(&phone, &bench);
        assert_eq!(link_state().online, Some(false));
        assert_eq!(link_state().updated_at, before);
    }

    /// A device paired before Servers had ids learns the id from its next
    /// pull, so it too can find a moved Server.
    #[test]
    fn a_device_paired_before_server_ids_learns_its_servers_id() {
        let _turn = one_at_a_time();
        let (bench, phone, _) = shop_and_phone();
        phone.set_paired_server_id("").unwrap();
        assert_eq!(phone.paired_server_id().unwrap(), None);
        want_sync();
        pass(&phone, &bench);
        let id = Store::open_path(&bench.server).unwrap().lan_server_id().unwrap();
        assert_eq!(phone.paired_server_id().unwrap(), Some(id));
    }
}

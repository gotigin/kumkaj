//! LAN realtime duplex (WebSocket on the same TCP port as HTTP).
//!
//! Protocol: docs/DEV.md § LAN (`LAN_PROTOCOL` ≥ 2). Server pushes revisioned
//! deltas; Clients fall back to `/wait` + `/sync` when the socket is down.

use crate::net::{
    self, apply_snapshot, Request, Reply, SyncSnapshot, HEADER_DEVICE, HEADER_PROTOCOL,
    HEADER_SESSION, LAN_PROTOCOL,
};
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tungstenite::protocol::{Role, WebSocket};
use tungstenite::{Error as WsError, Message};

const PING_EVERY: Duration = Duration::from_secs(15);
const SUB_QUEUE: usize = 8;

/// One domain-aware books patch. Absent fields are unchanged on the Client.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SyncPatch {
    pub seq: u64,
    pub prev_seq: u64,
    pub rev: String,
    /// When true, every domain below is a full replace (bootstrap / catch-up).
    #[serde(default)]
    pub full: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue_icon_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<crate::domain::Product>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customers: Option<Vec<crate::domain::Customer>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff: Option<Vec<crate::domain::StaffMember>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<Vec<crate::domain::CreditLedgerEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tables: Option<Vec<crate::domain::DiningTable>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_sales: Option<Vec<crate::domain::Sale>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub today: Option<crate::domain::DayTotals>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outstanding_credit_paise: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day_close_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_kots: Option<Vec<crate::domain::KotTicket>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<crate::domain::Category>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stations: Option<Vec<crate::domain::KitchenStation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fiscal: Option<crate::fiscal::FiscalSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_rates: Option<Vec<crate::domain::TaxRate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running_orders: Option<Vec<crate::domain::RunningOrder>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard_print_em: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bill_header: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fssai_no: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bill_footer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upi_vpa: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upi_payee: Option<String>,
}

impl SyncPatch {
    fn full_from(seq: u64, prev_seq: u64, snap: &SyncSnapshot) -> Self {
        Self {
            seq,
            prev_seq,
            rev: snap.rev.clone(),
            full: true,
            venue_title: Some(snap.venue_title.clone()),
            venue_icon_key: Some(snap.venue_icon_key.clone()),
            products: Some(snap.products.clone()),
            customers: Some(snap.customers.clone()),
            staff: Some(snap.staff.clone()),
            ledger: Some(snap.ledger.clone()),
            tables: Some(snap.tables.clone()),
            recent_sales: Some(snap.recent_sales.clone()),
            today: Some(snap.today.clone()),
            outstanding_credit_paise: Some(snap.outstanding_credit_paise),
            day_close_text: Some(snap.day_close_text.clone()),
            open_kots: Some(snap.open_kots.clone()),
            categories: Some(snap.categories.clone()),
            stations: Some(snap.stations.clone()),
            branch_name: Some(snap.branch_name.clone()),
            org_name: Some(snap.org_name.clone()),
            fiscal: Some(snap.fiscal.clone()),
            tax_rates: Some(snap.tax_rates.clone()),
            running_orders: Some(snap.running_orders.clone()),
            server_id: Some(snap.server_id.clone()),
            standard_print_em: snap.standard_print_em,
            bill_header: Some(snap.bill_header.clone()),
            fssai_no: Some(snap.fssai_no.clone()),
            bill_footer: Some(snap.bill_footer.clone()),
            upi_vpa: Some(snap.upi_vpa.clone()),
            upi_payee: Some(snap.upi_payee.clone()),
        }
    }

    fn diff(seq: u64, prev_seq: u64, before: &SyncSnapshot, after: &SyncSnapshot) -> Self {
        fn moved<T: Serialize>(a: &T, b: &T) -> bool {
            serde_json::to_vec(a).ok() != serde_json::to_vec(b).ok()
        }
        let mut p = Self {
            seq,
            prev_seq,
            rev: after.rev.clone(),
            full: false,
            ..Default::default()
        };
        if moved(&before.venue_title, &after.venue_title) {
            p.venue_title = Some(after.venue_title.clone());
        }
        if moved(&before.venue_icon_key, &after.venue_icon_key) {
            p.venue_icon_key = Some(after.venue_icon_key.clone());
        }
        if moved(&before.products, &after.products) {
            p.products = Some(after.products.clone());
        }
        if moved(&before.customers, &after.customers) {
            p.customers = Some(after.customers.clone());
        }
        if moved(&before.staff, &after.staff) {
            p.staff = Some(after.staff.clone());
        }
        if moved(&before.ledger, &after.ledger) {
            p.ledger = Some(after.ledger.clone());
        }
        if moved(&before.tables, &after.tables) {
            p.tables = Some(after.tables.clone());
        }
        if moved(&before.recent_sales, &after.recent_sales) {
            p.recent_sales = Some(after.recent_sales.clone());
        }
        if moved(&before.today, &after.today) {
            p.today = Some(after.today.clone());
        }
        if before.outstanding_credit_paise != after.outstanding_credit_paise {
            p.outstanding_credit_paise = Some(after.outstanding_credit_paise);
        }
        if moved(&before.day_close_text, &after.day_close_text) {
            p.day_close_text = Some(after.day_close_text.clone());
        }
        if moved(&before.open_kots, &after.open_kots) {
            p.open_kots = Some(after.open_kots.clone());
        }
        if moved(&before.categories, &after.categories) {
            p.categories = Some(after.categories.clone());
        }
        if moved(&before.stations, &after.stations) {
            p.stations = Some(after.stations.clone());
        }
        if moved(&before.branch_name, &after.branch_name) {
            p.branch_name = Some(after.branch_name.clone());
        }
        if moved(&before.org_name, &after.org_name) {
            p.org_name = Some(after.org_name.clone());
        }
        if moved(&before.fiscal, &after.fiscal) {
            p.fiscal = Some(after.fiscal.clone());
        }
        if moved(&before.tax_rates, &after.tax_rates) {
            p.tax_rates = Some(after.tax_rates.clone());
        }
        if moved(&before.running_orders, &after.running_orders) {
            p.running_orders = Some(after.running_orders.clone());
        }
        if moved(&before.server_id, &after.server_id) {
            p.server_id = Some(after.server_id.clone());
        }
        if before.standard_print_em != after.standard_print_em {
            p.standard_print_em = after.standard_print_em;
        }
        if moved(&before.bill_header, &after.bill_header) {
            p.bill_header = Some(after.bill_header.clone());
        }
        if moved(&before.fssai_no, &after.fssai_no) {
            p.fssai_no = Some(after.fssai_no.clone());
        }
        if moved(&before.bill_footer, &after.bill_footer) {
            p.bill_footer = Some(after.bill_footer.clone());
        }
        if moved(&before.upi_vpa, &after.upi_vpa) {
            p.upi_vpa = Some(after.upi_vpa.clone());
        }
        if moved(&before.upi_payee, &after.upi_payee) {
            p.upi_payee = Some(after.upi_payee.clone());
        }
        p
    }

    fn into_snapshot(self) -> Option<SyncSnapshot> {
        if !self.full {
            return None;
        }
        Some(SyncSnapshot {
            venue_title: self.venue_title.unwrap_or_default(),
            venue_icon_key: self.venue_icon_key.unwrap_or_default(),
            products: self.products.unwrap_or_default(),
            customers: self.customers.unwrap_or_default(),
            staff: self.staff.unwrap_or_default(),
            ledger: self.ledger.unwrap_or_default(),
            tables: self.tables.unwrap_or_default(),
            recent_sales: self.recent_sales.unwrap_or_default(),
            today: self.today.unwrap_or_default(),
            outstanding_credit_paise: self.outstanding_credit_paise.unwrap_or_default(),
            day_close_text: self.day_close_text.unwrap_or_default(),
            open_kots: self.open_kots.unwrap_or_default(),
            categories: self.categories.unwrap_or_default(),
            stations: self.stations.unwrap_or_default(),
            branch_name: self.branch_name.unwrap_or_default(),
            org_name: self.org_name.unwrap_or_default(),
            fiscal: self.fiscal.unwrap_or_default(),
            tax_rates: self.tax_rates.unwrap_or_default(),
            running_orders: self.running_orders.unwrap_or_default(),
            server_id: self.server_id.unwrap_or_default(),
            rev: self.rev,
            seq: self.seq,
            standard_print_em: self.standard_print_em,
            bill_header: self.bill_header.unwrap_or_default(),
            fssai_no: self.fssai_no.unwrap_or_default(),
            bill_footer: self.bill_footer.unwrap_or_default(),
            upi_vpa: self.upi_vpa.unwrap_or_default(),
            upi_payee: self.upi_payee.unwrap_or_default(),
        })
    }
}

/// Apply a patch into the Client cache (one transaction).
pub fn apply_patch(store: &Store, patch: &SyncPatch) -> Result<(), String> {
    if patch.full {
        let snap = patch
            .clone()
            .into_snapshot()
            .ok_or_else(|| "full patch missing books".to_string())?;
        return apply_snapshot(store, &snap);
    }
    store.in_transaction(|| {
        if let Some(products) = &patch.products {
            store.replace_catalog_from_sync(products)?;
        }
        if patch.categories.is_some() || patch.stations.is_some() {
            let cats = patch.categories.clone().unwrap_or_else(|| store.list_categories().unwrap_or_default());
            let stations = patch.stations.clone().unwrap_or_else(|| store.list_stations().unwrap_or_default());
            store.replace_menu_meta_from_sync(&cats, &stations)?;
        }
        if let Some(customers) = &patch.customers {
            store.replace_customers_from_sync(customers)?;
        }
        if let Some(ledger) = &patch.ledger {
            store.replace_credit_ledger_from_sync(ledger)?;
        }
        if let Some(tables) = &patch.tables {
            store.replace_tables_from_sync(tables)?;
        }
        if let Some(sales) = &patch.recent_sales {
            store.replace_sales_cache_from_sync(sales)?;
        }
        if let Some(kots) = &patch.open_kots {
            store.replace_kots_from_sync(kots)?;
        }
        if let Some(running) = &patch.running_orders {
            store.replace_running_from_sync(running)?;
        }
        if let Some(staff) = &patch.staff {
            // Deltas carry the full roster when staff is included.
            store.replace_staff_cache_from_sync(staff)?;
        }
        if let Some(title) = &patch.venue_title {
            store.set_venue_title(title)?;
        }
        if let Some(org) = &patch.org_name {
            store.set_org_name_from_sync(org)?;
        }
        if let Some(icon) = &patch.venue_icon_key {
            let t = icon.trim();
            store.set_venue_icon_key(if t.is_empty() { None } else { Some(t) })?;
        }
        if let (Some(fiscal), rates) = (&patch.fiscal, &patch.tax_rates)
            && !fiscal.country_code.trim().is_empty()
        {
            let rates = rates.clone().unwrap_or_default();
            store.replace_fiscal_from_sync(fiscal, &rates)?;
        }
        if let Some(today) = &patch.today {
            let out = patch.outstanding_credit_paise.unwrap_or(0);
            store.cache_server_day_totals(today, out)?;
        }
        if let Some(em) = patch.standard_print_em {
            store.set_standard_print_em_from_sync(em)?;
        }
        if patch.bill_header.is_some()
            || patch.fssai_no.is_some()
            || patch.bill_footer.is_some()
            || patch.upi_vpa.is_some()
            || patch.upi_payee.is_some()
        {
            let header = patch
                .bill_header
                .clone()
                .unwrap_or_else(|| store.bill_header().unwrap_or_default());
            let fssai = patch
                .fssai_no
                .clone()
                .unwrap_or_else(|| store.fssai_no().unwrap_or_default());
            let footer = patch
                .bill_footer
                .clone()
                .unwrap_or_else(|| store.bill_footer().unwrap_or_default());
            let upi_vpa = patch
                .upi_vpa
                .clone()
                .unwrap_or_else(|| store.upi_vpa().unwrap_or_default());
            let upi_payee = patch
                .upi_payee
                .clone()
                .unwrap_or_else(|| store.upi_payee().unwrap_or_default());
            store.set_bill_format_from_sync(&header, &fssai, &footer, &upi_vpa, &upi_payee)?;
        }
        Ok(())
    })
}

// ---- Server hub ------------------------------------------------------------

struct Sub {
    tx: SyncSender<String>,
}

struct Hub {
    subs: Mutex<Vec<Sub>>,
    /// Per books path: tip seq + last snapshot for diffs.
    tips: Mutex<HashMap<PathBuf, Tip>>,
}

#[derive(Clone)]
struct Tip {
    seq: u64,
    rev: String,
    snap: SyncSnapshot,
}

fn hub() -> &'static Hub {
    static HUB: OnceLock<Hub> = OnceLock::new();
    HUB.get_or_init(|| Hub {
        subs: Mutex::new(Vec::new()),
        tips: Mutex::new(HashMap::new()),
    })
}

fn subscribe() -> Receiver<String> {
    let (tx, rx) = mpsc::sync_channel(SUB_QUEUE);
    hub().subs.lock().unwrap().push(Sub { tx });
    rx
}

fn broadcast(text: &str) {
    let mut subs = hub().subs.lock().unwrap();
    subs.retain(|s| s.tx.try_send(text.to_string()).is_ok());
}

/// Called from `note_rev` after the content hash moved — bump seq, push delta.
pub fn on_books_changed(db_path: &Path, rev: &str, snap: &SyncSnapshot) {
    let mut tips = hub().tips.lock().unwrap();
    let path = db_path.to_path_buf();
    let (seq, _prev_seq, patch) = match tips.get(&path).cloned() {
        Some(prev) if prev.rev == rev => return,
        Some(prev) => {
            let seq = prev.seq.saturating_add(1);
            let patch = SyncPatch::diff(seq, prev.seq, &prev.snap, snap);
            (seq, prev.seq, patch)
        }
        None => {
            let patch = SyncPatch::full_from(1, 0, snap);
            (1, 0, patch)
        }
    };
    tips.insert(
        path,
        Tip {
            seq,
            rev: rev.to_string(),
            snap: snap.clone(),
        },
    );
    drop(tips);
    let mut msg = serde_json::to_value(&patch).unwrap_or_else(|_| serde_json::json!({}));
    if let Some(obj) = msg.as_object_mut() {
        obj.insert("t".into(), serde_json::json!("delta"));
    }
    broadcast(&msg.to_string());
}

fn tip_of(db_path: &Path) -> Option<Tip> {
    hub().tips.lock().unwrap().get(db_path).cloned()
}

/// Current live tip seq for `/sync` replies (0 if none yet).
pub fn tip_seq(db_path: &Path) -> Option<u64> {
    tip_of(db_path).map(|t| t.seq)
}

/// True when this HTTP request is a WebSocket upgrade to `/ws`.
pub fn is_ws_upgrade(req: &Request) -> bool {
    req.route() == "/ws"
        && req
            .header("upgrade")
            .is_some_and(|u| u.eq_ignore_ascii_case("websocket"))
}

/// Finish the handshake and serve one subscriber until disconnect.
pub fn handle_upgrade(
    mut stream: TcpStream,
    req: &Request,
    db_path: &Path,
    peer: &str,
    guard: &net::Guard,
) -> Result<(), String> {
    let _ = (peer, guard);
    let store = Store::open_path(db_path)?;
    if let Some(refusal) = net::protocol_refusal_public(req) {
        let _ = net::write_http_public(&mut stream, refusal.status, refusal.content_type, &refusal.body, false);
        return Ok(());
    }
    let device_token = req.header(HEADER_DEVICE).unwrap_or_default().to_string();
    let Some(_device) = store.lan_device_for_token(&device_token)? else {
        let refusal = Reply::refuse(
            401,
            "This device is not paired with the shop any more — pair it again",
            "pair_again",
        );
        let _ = net::write_http_public(&mut stream, refusal.status, refusal.content_type, &refusal.body, false);
        return Ok(());
    };
    let key = req
        .header("Sec-WebSocket-Key")
        .ok_or_else(|| "missing Sec-WebSocket-Key".to_string())?;
    let accept = tungstenite::handshake::derive_accept_key(key.as_bytes());
    write!(
        stream,
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {accept}\r\n\r\n"
    )
    .map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    let _ = stream.set_read_timeout(Some(PING_EVERY + Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));
    let mut ws = WebSocket::from_raw_socket(stream, Role::Server, None);
    let rx = subscribe();
    // Ensure tip exists so welcome carries a seq.
    if tip_of(db_path).is_none()
        && let Ok((json, rev)) = net::snapshot_json_public(&store)
        && let Ok(mut snap) = serde_json::from_str::<SyncSnapshot>(&json)
    {
        snap.rev = rev.clone();
        on_books_changed(db_path, &rev, &snap);
    }
    let tip = tip_of(db_path);
    let welcome = serde_json::json!({
        "t": "welcome",
        "seq": tip.as_ref().map(|t| t.seq).unwrap_or(0),
        "rev": tip.as_ref().map(|t| t.rev.as_str()).unwrap_or(""),
    });
    ws.send(Message::Text(welcome.to_string().into()))
        .map_err(|e| e.to_string())?;

    let have_seq = req
        .query("seq")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    if let Some(tip) = tip_of(db_path) {
        if have_seq == 0 || have_seq != tip.seq {
            // Cold start or gap: Client HTTP `/sync` then resumes deltas.
            let need = serde_json::json!({"t":"need_sync","seq": tip.seq, "rev": tip.rev});
            let _ = ws.send(Message::Text(need.to_string().into()));
        }
    }

    let mut last_ping = Instant::now();
    loop {
        // Push queued broadcasts.
        loop {
            match rx.try_recv() {
                Ok(text) => {
                    if ws.send(Message::Text(text.into())).is_err() {
                        return Ok(());
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
        }
        if last_ping.elapsed() >= PING_EVERY {
            if ws.send(Message::Ping(vec![].into())).is_err() {
                return Ok(());
            }
            last_ping = Instant::now();
        }
        match ws.read() {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_client_frame(
                    db_path,
                    &device_token,
                    text.as_ref(),
                    &mut ws,
                    req.header(HEADER_SESSION).unwrap_or_default(),
                ) {
                    let err = serde_json::json!({"t":"outbox_reply","ok":false,"error": e});
                    let _ = ws.send(Message::Text(err.to_string().into()));
                }
            }
            Ok(Message::Ping(p)) => {
                let _ = ws.send(Message::Pong(p));
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) | Err(WsError::ConnectionClosed) | Err(WsError::AlreadyClosed) => {
                return Ok(());
            }
            Err(WsError::Io(e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => return Ok(()),
            Ok(_) => {}
        }
    }
}

fn handle_client_frame(
    db_path: &Path,
    device_token: &str,
    text: &str,
    ws: &mut WebSocket<TcpStream>,
    session_hdr: &str,
) -> Result<(), String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let t = v.get("t").and_then(|x| x.as_str()).unwrap_or("");
    match t {
        "pong" | "ping" => {
            if t == "ping" {
                let _ = ws.send(Message::Text(r#"{"t":"pong"}"#.into()));
            }
            Ok(())
        }
        "outbox" => {
            let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let path = v.get("path").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let body = v
                .get("body")
                .map(|b| {
                    if b.is_string() {
                        b.as_str().unwrap_or("").to_string()
                    } else {
                        b.to_string()
                    }
                })
                .unwrap_or_default();
            let request_id = v
                .get("request_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let session = v
                .get("session")
                .and_then(|x| x.as_str())
                .unwrap_or(session_hdr)
                .to_string();
            let reply = net::answer_ws_write(db_path, device_token, &session, &path, &body, &request_id);
            let msg = serde_json::json!({
                "t": "outbox_reply",
                "id": id,
                "ok": (200..300).contains(&reply.status),
                "status": reply.status,
                "body": serde_json::from_str::<serde_json::Value>(&reply.body).unwrap_or(serde_json::json!({ "raw": reply.body })),
            });
            ws.send(Message::Text(msg.to_string().into()))
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        _ => {
            // A mistyped `t` must not leave the phone waiting forever on
            // an outbox id — answer so the write can leave the queue.
            if let Some(id) = v.get("id").and_then(|x| x.as_str()).filter(|s| !s.is_empty()) {
                let msg = serde_json::json!({
                    "t": "outbox_reply",
                    "id": id,
                    "ok": false,
                    "status": 400,
                    "body": { "error": format!("Unknown message type \"{t}\"") },
                });
                ws.send(Message::Text(msg.to_string().into()))
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        }
    }
}

// ---- Client watcher --------------------------------------------------------

/// Drive the phone's live link. Returns when the socket dies (caller falls
/// back to `/wait`).
pub fn client_watch_loop(
    link: &net::Link,
    have_seq: u64,
    have_rev: &str,
    mut on_delta: impl FnMut(SyncPatch) -> Result<(), String>,
    mut on_need_sync: impl FnMut(u64, &str),
    mut on_outbox_reply: impl FnMut(&str, bool, &serde_json::Value),
    outbox_rx: &Receiver<String>,
) -> Result<(), String> {
    let host = link
        .base
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    let _ = have_rev;
    let mut http_req = format!(
        "GET /ws?seq={have_seq} HTTP/1.1\r\nHost: {host}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n{HEADER_PROTOCOL}: {LAN_PROTOCOL}\r\n{HEADER_DEVICE}: {}\r\n",
        link.device
    );
    if !link.session.is_empty() {
        http_req.push_str(&format!("{HEADER_SESSION}: {}\r\n", link.session));
    }
    http_req.push_str("\r\n");

    let mut stream = TcpStream::connect(host).map_err(|e| e.to_string())?;
    let _ = stream.set_read_timeout(Some(PING_EVERY + Duration::from_secs(20)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));
    stream.write_all(http_req.as_bytes()).map_err(|e| e.to_string())?;
    // Read 101 response head.
    let mut buf = Vec::new();
    let mut chunk = [0u8; 2048];
    let head_end = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        let n = std::io::Read::read(&mut stream, &mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("ws handshake closed".into());
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.len() > 64 * 1024 {
            return Err("ws handshake too large".into());
        }
    };
    let head = String::from_utf8_lossy(&buf[..head_end]);
    if !head.starts_with("HTTP/1.1 101") {
        return Err(format!("ws upgrade refused: {}", head.lines().next().unwrap_or("")));
    }
    let mut ws = WebSocket::from_raw_socket(stream, Role::Client, None);
    let mut last_ping = Instant::now();
    loop {
        while let Ok(frame) = outbox_rx.try_recv() {
            if ws.send(Message::Text(frame.into())).is_err() {
                return Err("ws send failed".into());
            }
        }
        if last_ping.elapsed() >= PING_EVERY {
            let _ = ws.send(Message::Text(r#"{"t":"ping"}"#.into()));
            last_ping = Instant::now();
        }
        match ws.read() {
            Ok(Message::Text(text)) => {
                let v: serde_json::Value = match serde_json::from_str(text.as_ref()) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                match v.get("t").and_then(|t| t.as_str()).unwrap_or("") {
                    "welcome" => {}
                    "need_sync" => {
                        let seq = v.get("seq").and_then(|s| s.as_u64()).unwrap_or(0);
                        let rev = v.get("rev").and_then(|s| s.as_str()).unwrap_or("");
                        on_need_sync(seq, rev);
                    }
                    "delta" => {
                        if let Ok(patch) = serde_json::from_value::<SyncPatch>(v) {
                            on_delta(patch)?;
                        }
                    }
                    "outbox_reply" => {
                        let id = v.get("id").and_then(|s| s.as_str()).unwrap_or("");
                        let ok = v.get("ok").and_then(|s| s.as_bool()).unwrap_or(false);
                        let body = v.get("body").cloned().unwrap_or(serde_json::json!({}));
                        on_outbox_reply(id, ok, &body);
                    }
                    "ping" => {
                        let _ = ws.send(Message::Text(r#"{"t":"pong"}"#.into()));
                    }
                    _ => {}
                }
            }
            Ok(Message::Ping(p)) => {
                let _ = ws.send(Message::Pong(p));
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) | Err(WsError::ConnectionClosed) | Err(WsError::AlreadyClosed) => {
                return Err("ws closed".into());
            }
            Err(WsError::Io(e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e.to_string()),
            Ok(_) => {}
        }
    }
}

/// Build an outbox frame for the live socket.
pub fn outbox_frame(id: &str, path: &str, body: &str, request_id: &str, session: &str) -> String {
    serde_json::json!({
        "t": "outbox",
        "id": id,
        "path": path,
        "body": body,
        "request_id": request_id,
        "session": session,
    })
    .to_string()
}

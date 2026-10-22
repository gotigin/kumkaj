//! Local LAN Client/Server sync for Kumkaj (no cloud).
//!
//! Server listens on `DEFAULT_PORT` and answers with JSON over HTTP/1.1.
//! Client uses `ureq` to pair, pull catalog/ledger/sales, and push sales/settlements.

use crate::domain::{
    Category, CreditLedgerEntry, Customer, DayTotals,
    PayMethod, Product, Sale, SaleDraft, StaffMember, TaxRate,
};
use crate::fiscal::FiscalSettings;
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 7847;

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static LAN_ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncSnapshot {
    pub venue_title: String,
    pub pairing_code: String,
    pub products: Vec<Product>,
    pub customers: Vec<Customer>,
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
    /// Item groupings of the Server's branch.
    #[serde(default)]
    pub categories: Vec<Category>,
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
    pub actor_id: String,
    /// Split tenders. Empty means a single tender in `pay`.
    #[serde(default)]
    pub payments: Vec<RemoteTender>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteSettleRequest {
    pub customer_id: Option<String>,
    #[serde(default)]
    pub customer_name: Option<String>,
    pub amount_paise: i64,
    pub pay: String,
    pub actor_id: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteUndoRequest {
    pub sale_id: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteSaleAction {
    pub sale_id: String,
    pub actor_id: String,
    #[serde(default)]
    pub reason: String,
}

/// Correct a bill the Server owns. The Client never edits its own sales cache.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteAmendRequest {
    pub sale_id: String,
    pub draft: SaleDraft,
    pub pay: String,
    pub reason: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairOk {
    pub ok: bool,
    pub venue_title: String,
    pub port: u16,
}

pub fn server_is_running() -> bool {
    SERVER_RUNNING.load(Ordering::SeqCst) && LAN_ENABLED.load(Ordering::SeqCst)
}

/// Spawn (or resume) the shop LAN listener. Extra tills stay off until this is called.
pub fn ensure_server(db_path: PathBuf, pairing_code: String) -> Result<(), String> {
    LAN_ENABLED.store(true, Ordering::SeqCst);
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }
    if SERVER_RUNNING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let code = pairing_code;
    let path = db_path;
    let _handle: JoinHandle<()> = thread::Builder::new()
        .name("kumkaj-lan".into())
        .spawn(move || {
            if let Err(e) = run_server_loop(path, code) {
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

fn run_server_loop(db_path: PathBuf, pairing_code: String) -> Result<(), String> {
    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    let listener = TcpListener::bind(addr).map_err(|e| format!("bind {DEFAULT_PORT}: {e}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;
    let expected = Arc::new(pairing_code);
    while LAN_ENABLED.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let expected = Arc::clone(&expected);
                let path = db_path.clone();
                let _ = thread::spawn(move || {
                    if let Err(e) = handle_client(stream, &path, &expected) {
                        eprintln!("Kumkaj LAN request error: {e}");
                    }
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(80));
            }
            Err(e) => {
                eprintln!("Kumkaj LAN accept: {e}");
                thread::sleep(Duration::from_millis(80));
            }
        }
    }
    Ok(())
}

fn build_snapshot(store: &Store, pairing_code: &str) -> Result<SyncSnapshot, String> {
    let mut recent_sales = store.recent_sales(40)?;
    for sale in &mut recent_sales {
        sale.lines = store.sale_lines(&sale.id).unwrap_or_default();
    }
    Ok(SyncSnapshot {
        venue_title: store.venue_title().unwrap_or_default(),
        pairing_code: pairing_code.into(),
        products: store.list_products(true)?,
        customers: store.list_customers()?,
        staff: store.list_staff()?,
        ledger: store.list_all_credit_ledger(2000)?,
        recent_sales,
        today: store.today_totals()?,
        outstanding_credit_paise: store.outstanding_credit_total()?,
        day_close_text: store.day_close_summary()?,
        categories: store.list_categories()?,
        branch_name: store.active_branch().map(|b| b.name).unwrap_or_default(),
        org_name: store.org_name().unwrap_or_default(),
        fiscal: store.fiscal_settings()?,
        tax_rates: store.list_tax_rates()?,
    })
}

fn handle_client(mut stream: TcpStream, db_path: &Path, expected_code: &str) -> Result<(), String> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));
    let mut buf = vec![0u8; 256 * 1024];
    let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(());
    }
    let raw = String::from_utf8_lossy(&buf[..n]);
    let (method, path, body) = parse_http(&raw)?;
    let code = query_param(&path, "code").unwrap_or_default();
    // Public track-by-AWB is intentionally open on the LAN (no pairing code).
    let needs_code = !(path.starts_with("/health") || path.starts_with("/track"));
    if needs_code && code != expected_code {
        return write_http(&mut stream, 403, r#"{"ok":false,"error":"bad pairing code"}"#);
    }

    let store = Store::open_path(db_path)?;

    if path.starts_with("/health") {
        return write_http(&mut stream, 200, r#"{"ok":true}"#);
    }
    if path.starts_with("/track/") || path.starts_with("/track?") {
        let awb_raw = if let Some(rest) = path.strip_prefix("/track/") {
            rest.split('?').next().unwrap_or("").trim().to_string()
        } else {
            query_param(&path, "awb").unwrap_or_default()
        };
        let awb = urlencoding_decode(&awb_raw);
        match store.public_track(&awb)? {
            Some(view) => {
                let body = crate::track::to_json(&view)?;
                return write_http(&mut stream, 200, &body);
            }
            None => {
                return write_http(
                    &mut stream,
                    404,
                    r#"{"ok":false,"error":"AWB not found"}"#,
                );
            }
        }
    }
    if path.starts_with("/pair") {
        let venue = store.venue_title().unwrap_or_default();
        let resp = PairOk {
            ok: true,
            venue_title: venue,
            port: DEFAULT_PORT,
        };
        let body = serde_json::to_string(&resp).map_err(|e| e.to_string())?;
        return write_http(&mut stream, 200, &body);
    }
    if path.starts_with("/sync") || path.starts_with("/today") {
        let snap = build_snapshot(&store, expected_code)?;
        let body = serde_json::to_string(&snap).map_err(|e| e.to_string())?;
        return write_http(&mut stream, 200, &body);
    }
    if method == "POST" && path.starts_with("/sale") {
        let req: RemoteSaleRequest =
            serde_json::from_str(&body).map_err(|e| format!("sale json: {e}"))?;
        let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
        let actor = store
            .list_staff()?
            .into_iter()
            .find(|s| s.id == req.actor_id)
            .ok_or_else(|| "actor not found".to_string())?;
        let mut store = store;
        let sale = if req.payments.is_empty() {
            let customer_id = if pay.is_credit() {
                if let Some(id) = req.customer_id.clone() {
                    Some(id)
                } else if let Some(name) =
                    req.customer_name.as_deref().map(str::trim).filter(|s| !s.is_empty())
                {
                    Some(store.find_or_create_customer(name)?.id)
                } else {
                    return Err("Credit sale needs a customer".into());
                }
            } else {
                None
            };
            store.finalize_sale(
                &req.draft,
                pay,
                customer_id.as_deref(),
                &actor,
            )?
        } else {
            // Mixed Cash/UPI/Card/Credit (and legacy split) — amounts in `payments`.
            if !pay.is_split() && req.payments.len() > 1 {
                return Err("Multi-tender sale must use pay method split".into());
            }
            let mut parts = Vec::with_capacity(req.payments.len());
            for tender in &req.payments {
                let tender_method = PayMethod::parse(&tender.pay)
                    .ok_or_else(|| format!("bad pay method {}", tender.pay))?;
                parts.push((tender_method, tender.amount_paise));
            }
            let needs_credit = parts.iter().any(|(m, _)| m.is_credit());
            let customer_id = if needs_credit {
                if let Some(id) = req.customer_id.clone() {
                    Some(id)
                } else if let Some(name) =
                    req.customer_name.as_deref().map(str::trim).filter(|s| !s.is_empty())
                {
                    Some(store.find_or_create_customer(name)?.id)
                } else {
                    return Err("Credit part needs a customer".into());
                }
            } else {
                None
            };
            store.finalize_sale_tenders(
                &req.draft,
                &parts,
                customer_id.as_deref(),
                &actor,
            )?
        };
        let body = serde_json::to_string(&sale).map_err(|e| e.to_string())?;
        return write_http(&mut stream, 200, &body);
    }
    if method == "POST" && path.starts_with("/settle") {
        let req: RemoteSettleRequest =
            serde_json::from_str(&body).map_err(|e| format!("settle json: {e}"))?;
        let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
        let actor = store
            .list_staff()?
            .into_iter()
            .find(|s| s.id == req.actor_id)
            .ok_or_else(|| "actor not found".to_string())?;
        let mut store = store;
        let customer_id = if let Some(id) = req.customer_id.clone() {
            id
        } else if let Some(name) = req.customer_name.as_deref().map(str::trim).filter(|s| !s.is_empty())
        {
            store.find_or_create_customer(name)?.id
        } else {
            return Err("Customer required".into());
        };
        store.settle_credit(
            &customer_id,
            req.amount_paise,
            pay,
            &actor,
            &req.detail,
        )?;
        return write_http(&mut stream, 200, r#"{"ok":true}"#);
    }
    if method == "POST" && path.starts_with("/undo") {
        let req: RemoteUndoRequest =
            serde_json::from_str(&body).map_err(|e| format!("undo json: {e}"))?;
        let actor = store
            .list_staff()?
            .into_iter()
            .find(|s| s.id == req.actor_id)
            .ok_or_else(|| "actor not found".to_string())?;
        let mut store = store;
        store.quick_undo_sale(&req.sale_id, &actor)?;
        return write_http(&mut stream, 200, r#"{"ok":true}"#);
    }
    if method == "POST" && path.starts_with("/void") {
        let req: RemoteSaleAction =
            serde_json::from_str(&body).map_err(|e| format!("void json: {e}"))?;
        let actor = store
            .list_staff()?
            .into_iter()
            .find(|s| s.id == req.actor_id)
            .ok_or_else(|| "actor not found".to_string())?;
        let mut store = store;
        store.void_sale(&req.sale_id, &req.reason, &actor)?;
        return write_http(&mut stream, 200, r#"{"ok":true}"#);
    }
    if method == "POST" && path.starts_with("/refund") {
        let req: RemoteSaleAction =
            serde_json::from_str(&body).map_err(|e| format!("refund json: {e}"))?;
        let actor = store
            .list_staff()?
            .into_iter()
            .find(|s| s.id == req.actor_id)
            .ok_or_else(|| "actor not found".to_string())?;
        let mut store = store;
        store.refund_sale(&req.sale_id, &req.reason, &actor)?;
        return write_http(&mut stream, 200, r#"{"ok":true}"#);
    }
    if method == "POST" && path.starts_with("/amend") {
        let req: RemoteAmendRequest =
            serde_json::from_str(&body).map_err(|e| format!("amend json: {e}"))?;
        let pay = PayMethod::parse(&req.pay).ok_or_else(|| "bad pay method".to_string())?;
        let actor = store
            .list_staff()?
            .into_iter()
            .find(|s| s.id == req.actor_id)
            .ok_or_else(|| "actor not found".to_string())?;
        let mut store = store;
        let sale = store.amend_sale(&req.sale_id, &req.draft, pay, &req.reason, &actor)?;
        let body = serde_json::to_string(&sale).map_err(|e| e.to_string())?;
        return write_http(&mut stream, 200, &body);
    }
    write_http(&mut stream, 404, r#"{"ok":false,"error":"not found"}"#)
}

fn parse_http(raw: &str) -> Result<(String, String, String), String> {
    let mut parts = raw.split("\r\n\r\n");
    let head = parts.next().unwrap_or("");
    let body = parts.next().unwrap_or("").to_string();
    let first = head.lines().next().unwrap_or("");
    let mut sp = first.split_whitespace();
    let method = sp.next().unwrap_or("GET").to_string();
    let path = sp.next().unwrap_or("/").to_string();
    Ok((method, path, body))
}

fn query_param(path: &str, key: &str) -> Option<String> {
    let q = path.split('?').nth(1)?;
    for pair in q.split('&') {
        let mut kv = pair.splitn(2, '=');
        let k = kv.next()?;
        let v = kv.next().unwrap_or("");
        if k == key {
            return Some(urlencoding_decode(v));
        }
    }
    None
}

fn urlencoding_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &raw[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn write_http(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Error",
    };
    let resp = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(resp.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

fn base_url(host: &str) -> String {
    let host = host.trim().trim_end_matches('/');
    if host.starts_with("http://") || host.starts_with("https://") {
        host.to_string()
    } else if host.contains(':') {
        format!("http://{host}")
    } else {
        format!("http://{host}:{DEFAULT_PORT}")
    }
}

pub fn client_pair(host: &str, code: &str) -> Result<(String, SyncSnapshot), String> {
    let base = base_url(host);
    let url = format!("{base}/pair?code={code}");
    let pair: PairOk = ureq::get(&url)
        .timeout(Duration::from_secs(5))
        .call()
        .map_err(|e| format!("pair failed: {e}"))?
        .into_json()
        .map_err(|e| format!("pair json: {e}"))?;
    if !pair.ok {
        return Err("Server rejected pairing".into());
    }
    let snap = client_sync(&base, code)?;
    Ok((base, snap))
}

pub fn client_sync(server_url: &str, code: &str) -> Result<SyncSnapshot, String> {
    let base = base_url(server_url);
    let url = format!("{base}/sync?code={code}");
    ureq::get(&url)
        .timeout(Duration::from_secs(12))
        .call()
        .map_err(|e| format!("sync failed: {e}"))?
        .into_json()
        .map_err(|e| format!("sync json: {e}"))
}

pub fn client_push_sale(
    server_url: &str,
    code: &str,
    req: &RemoteSaleRequest,
) -> Result<Sale, String> {
    let base = base_url(server_url);
    let url = format!("{base}/sale?code={code}");
    ureq::post(&url)
        .timeout(Duration::from_secs(12))
        .send_json(req)
        .map_err(|e| format!("remote sale failed: {e}"))?
        .into_json()
        .map_err(|e| format!("sale json: {e}"))
}

pub fn client_push_settle(
    server_url: &str,
    code: &str,
    req: &RemoteSettleRequest,
) -> Result<(), String> {
    let base = base_url(server_url);
    let url = format!("{base}/settle?code={code}");
    let _ = ureq::post(&url)
        .timeout(Duration::from_secs(10))
        .send_json(req)
        .map_err(|e| format!("remote settle failed: {e}"))?;
    Ok(())
}

pub fn client_push_undo(
    server_url: &str,
    code: &str,
    req: &RemoteUndoRequest,
) -> Result<(), String> {
    let base = base_url(server_url);
    let url = format!("{base}/undo?code={code}");
    let _ = ureq::post(&url)
        .timeout(Duration::from_secs(10))
        .send_json(req)
        .map_err(|e| format!("remote undo failed: {e}"))?;
    Ok(())
}

pub fn client_push_void(
    server_url: &str,
    code: &str,
    req: &RemoteSaleAction,
) -> Result<(), String> {
    let base = base_url(server_url);
    let url = format!("{base}/void?code={code}");
    let _ = ureq::post(&url)
        .timeout(Duration::from_secs(10))
        .send_json(req)
        .map_err(|e| format!("remote void failed: {e}"))?;
    Ok(())
}

pub fn client_push_refund(
    server_url: &str,
    code: &str,
    req: &RemoteSaleAction,
) -> Result<(), String> {
    let base = base_url(server_url);
    let url = format!("{base}/refund?code={code}");
    let _ = ureq::post(&url)
        .timeout(Duration::from_secs(10))
        .send_json(req)
        .map_err(|e| format!("remote refund failed: {e}"))?;
    Ok(())
}

pub fn client_push_amend(
    server_url: &str,
    code: &str,
    req: &RemoteAmendRequest,
) -> Result<Sale, String> {
    let base = base_url(server_url);
    let url = format!("{base}/amend?code={code}");
    ureq::post(&url)
        .timeout(Duration::from_secs(12))
        .send_json(req)
        .map_err(|e| format!("remote amend failed: {e}"))?
        .into_json()
        .map_err(|e| format!("amend json: {e}"))
}

/// Apply a sync snapshot into the local Client cache DB.
pub fn apply_snapshot(store: &Store, snap: &SyncSnapshot) -> Result<(), String> {
    store.replace_catalog_from_sync(&snap.products)?;
    store.replace_categories_from_sync(&snap.categories)?;
    store.replace_customers_from_sync(&snap.customers)?;
    store.replace_credit_ledger_from_sync(&snap.ledger)?;
    store.replace_sales_cache_from_sync(&snap.recent_sales)?;
    for s in &snap.staff {
        store.upsert_staff_cache(s)?;
    }
    if let Some(owner) = snap.staff.iter().find(|s| s.is_owner()) {
        store.upsert_owner_pin_cache(owner)?;
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
}

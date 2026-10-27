//! Local SQLite persistence for Kumkaj (WAL, restart-safe sales).

use crate::domain::{
    format_qty_display, milli_to_whole_qty, new_id, normalize_party_roles, AttachmentEntity,
    AttachmentKind, AuditEvent, BooksOverview, Branch, BranchSnapshot,
    Category,
    CreditLedgerEntry, Customer,
    DayTotals, DiningTable, EntryAttachment, EventTimes, KitchenStation, KotStatus, KotTicket,
    Money, MoneyLane, MoneyMoveKind, MoneyMovement, OpenBill, OpsReport, OrderType,
    OrgConsolidatedReport,
    Cart, PartyAgingRow, PartyPositionRow, PayMethod, PeriodBooks, Product, Purchase, PurchaseLine,
    Sale, SaleDraft, SaleLine, SaleRevision, StaffFeatures, StaffMember, StaffRole,
    StockMovement, TableStatus, TaxRate, UnitConversion, UnitDef, VenueMode, STOCK_UNTRACKED,
};
use crate::media;
use crate::entity_template::{self, EntityTemplate};
use crate::fiscal::{self, CountryProfile, FiscalSettings, TaxPricing};
use crate::locale::{self, LanguageSource, LocaleSettings};
use chrono::{Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Store {
    conn: Connection,
    path: PathBuf,
}

#[path = "store_running.rs"]
mod running;
#[path = "store_service.rs"]
mod service;
#[path = "store_recipes.rs"]
mod recipes;
#[path = "store_ops.rs"]
mod ops;
pub use recipes::{milli_as_typed, recipe_as_typed, LowStock, RecipeLine, DEFAULT_REORDER_AT};
pub use service::{Reservation, ShiftClose};
pub use ops::{
    whatsapp_bill_url, GiftVoucher, PurchaseOrder, PurchaseOrderLine, StockTransferFile,
    WaiterCall, WaitlistEntry,
};
use running::natural_key;

/// Everything needed to create one staff member. Kept together so the hire
/// call stays readable instead of an eight-argument list.
pub struct StaffHire<'a> {
    pub name: &'a str,
    pub role: StaffRole,
    pub role_title: &'a str,
    pub pin: &'a str,
    pub login_id: &'a str,
    pub password: &'a str,
    pub features: StaffFeatures,
}

impl Store {
    /// Open the active local server DB.
    /// Prefer [`crate::local_servers`] when creating / switching servers.
    pub fn open_default() -> Result<Self, String> {
        crate::local_servers::open_default_server()
    }

    pub fn open_path(path: &Path) -> Result<Self, String> {
        let mut conn = Connection::open(path).map_err(|e| e.to_string())?;
        // Take the write lock when a transaction begins, so it waits out
        // another writer (busy_timeout) instead of failing midway in WAL.
        conn.set_transaction_behavior(rusqlite::TransactionBehavior::Immediate);
        conn.execute_batch(
            // busy_timeout: the LAN server writes from its own connection
            // per request; a write that meets another waits instead of
            // failing with "database is locked".
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;
             PRAGMA synchronous=FULL;
             PRAGMA foreign_keys=ON;",
        )
        .map_err(|e| e.to_string())?;
        let mut store = Self {
            conn,
            path: path.to_path_buf(),
        };
        store.migrate()?;
        store.ensure_seed_data()?;
        Ok(store)
    }

    pub fn db_path(&self) -> &Path {
        &self.path
    }

    /// Run `f` as one transaction: all of its writes land together or none
    /// do, other connections never see half of them, and the disk is synced
    /// once instead of once per row.
    pub fn in_transaction<T>(&self, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        let out = f()?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Create the current books schema (version **1**).
    ///
    /// Pre-release: there is no upgrade ladder from older numbered schemas.
    /// Delete local DBs from earlier builds if open fails after a schema change.
    fn migrate(&mut self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS meta (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS products (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    price_paise INTEGER NOT NULL CHECK(price_paise >= 0),
                    available INTEGER NOT NULL DEFAULT 1,
                    archived INTEGER NOT NULL DEFAULT 0,
                    image_key TEXT,
                    stock_qty INTEGER NOT NULL DEFAULT -1,
                    category_id TEXT NOT NULL DEFAULT '',
                    station_id TEXT NOT NULL DEFAULT '',
                    tax_bps INTEGER NOT NULL DEFAULT 0,
                    modifiers_json TEXT NOT NULL DEFAULT '[]',
                    aliases_json TEXT NOT NULL DEFAULT '[]',
                    branch_id TEXT NOT NULL DEFAULT '',
                    tax_rate_id TEXT NOT NULL DEFAULT '',
                    unit_label TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS sales (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    total_paise INTEGER NOT NULL,
                    pay_method TEXT NOT NULL,
                    status TEXT NOT NULL,
                    table_id TEXT,
                    lines_total_paise INTEGER,
                    rounding_paise INTEGER NOT NULL DEFAULT 0,
                    customer_id TEXT,
                    created_by_id TEXT NOT NULL DEFAULT '',
                    created_by_name TEXT NOT NULL DEFAULT '',
                    branch_id TEXT NOT NULL DEFAULT '',
                    order_type TEXT NOT NULL DEFAULT 'counter',
                    tax_paise INTEGER NOT NULL DEFAULT 0,
                    tax_pricing TEXT NOT NULL DEFAULT 'inclusive',
                    tax_label TEXT NOT NULL DEFAULT 'Tax',
                    net_paise INTEGER NOT NULL DEFAULT 0,
                    service_paise INTEGER NOT NULL DEFAULT 0,
                    invoice_no TEXT NOT NULL DEFAULT '',
                    money_lane TEXT NOT NULL DEFAULT 'declared',
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    server_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS sale_lines (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    sale_id TEXT NOT NULL,
                    product_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    unit_price_paise INTEGER NOT NULL,
                    qty INTEGER NOT NULL,
                    line_total_paise INTEGER NOT NULL,
                    note TEXT NOT NULL DEFAULT '',
                    qty_milli INTEGER NOT NULL DEFAULT 0,
                    unit_label TEXT NOT NULL DEFAULT '',
                    modifiers_json TEXT NOT NULL DEFAULT '[]'
                );
                CREATE TABLE IF NOT EXISTS sale_payments (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    pay_method TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS expense_payments (
                    id TEXT PRIMARY KEY,
                    expense_id TEXT NOT NULL,
                    pay_method TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS sale_revisions (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    revised_at TEXT NOT NULL,
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    reason TEXT NOT NULL,
                    previous_json TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS staff (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    role TEXT NOT NULL,
                    pin_hash TEXT NOT NULL,
                    active INTEGER NOT NULL DEFAULT 1,
                    role_title TEXT NOT NULL DEFAULT '',
                    features_json TEXT NOT NULL DEFAULT '',
                    login_id TEXT NOT NULL DEFAULT '',
                    password_hash TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS tables_dining (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    status TEXT NOT NULL,
                    active_sale_id TEXT,
                    branch_id TEXT NOT NULL DEFAULT '',
                    occupied_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS kot_tickets (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    status TEXT NOT NULL,
                    summary TEXT NOT NULL,
                    branch_id TEXT NOT NULL DEFAULT '',
                    station_id TEXT NOT NULL DEFAULT '',
                    table_id TEXT NOT NULL DEFAULT '',
                    total_paise INTEGER NOT NULL DEFAULT 0
                );
                CREATE TABLE IF NOT EXISTS audit_events (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    action TEXT NOT NULL,
                    detail TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS customers (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    phone TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT '',
                    active INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    branch_id TEXT NOT NULL DEFAULT '',
                    kind_label TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS credit_ledger (
                    id TEXT PRIMARY KEY,
                    customer_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL,
                    sale_id TEXT,
                    pay_method TEXT,
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    detail TEXT NOT NULL DEFAULT '',
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    server_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS expenses (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL,
                    category TEXT NOT NULL DEFAULT 'misc',
                    note TEXT NOT NULL DEFAULT '',
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    branch_id TEXT NOT NULL DEFAULT '',
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    server_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS branches (
                    id TEXT PRIMARY KEY,
                    org_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    code TEXT NOT NULL DEFAULT '',
                    active INTEGER NOT NULL DEFAULT 1,
                    is_this_device INTEGER NOT NULL DEFAULT 0,
                    last_snapshot_json TEXT NOT NULL DEFAULT '',
                    last_sync_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS categories (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    name TEXT NOT NULL,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    image_key TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS kitchen_stations (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    name TEXT NOT NULL,
                    sort_order INTEGER NOT NULL DEFAULT 0
                );
                CREATE TABLE IF NOT EXISTS stock_movements (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    product_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    delta_qty INTEGER NOT NULL,
                    reason TEXT NOT NULL DEFAULT 'adjust',
                    note TEXT NOT NULL DEFAULT '',
                    actor_id TEXT NOT NULL DEFAULT '',
                    actor_name TEXT NOT NULL DEFAULT '',
                    ref_id TEXT NOT NULL DEFAULT '',
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    server_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS purchases (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    supplier TEXT NOT NULL DEFAULT '',
                    total_paise INTEGER NOT NULL DEFAULT 0,
                    note TEXT NOT NULL DEFAULT '',
                    actor_id TEXT NOT NULL DEFAULT '',
                    actor_name TEXT NOT NULL DEFAULT '',
                    party_id TEXT NOT NULL DEFAULT '',
                    on_account INTEGER NOT NULL DEFAULT 0,
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    server_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS purchase_lines (
                    id TEXT PRIMARY KEY,
                    purchase_id TEXT NOT NULL,
                    product_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    qty INTEGER NOT NULL,
                    unit_cost_paise INTEGER NOT NULL DEFAULT 0,
                    qty_milli INTEGER NOT NULL DEFAULT 0,
                    unit_label TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS sync_outbox (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    payload_json TEXT NOT NULL DEFAULT '',
                    synced_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS tax_rates (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    name TEXT NOT NULL,
                    code TEXT NOT NULL DEFAULT '',
                    bps INTEGER NOT NULL DEFAULT 0,
                    active INTEGER NOT NULL DEFAULT 1
                );
                CREATE TABLE IF NOT EXISTS money_movements (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL,
                    note TEXT NOT NULL DEFAULT '',
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    branch_id TEXT NOT NULL DEFAULT '',
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    server_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS payable_ledger (
                    id TEXT PRIMARY KEY,
                    party_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL,
                    purchase_id TEXT,
                    pay_method TEXT,
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    detail TEXT NOT NULL DEFAULT '',
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    server_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS units (
                    id TEXT PRIMARY KEY,
                    code TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    builtin INTEGER NOT NULL DEFAULT 0,
                    picker_enabled INTEGER NOT NULL DEFAULT 1
                );
                CREATE TABLE IF NOT EXISTS unit_conversions (
                    id TEXT PRIMARY KEY,
                    from_code TEXT NOT NULL,
                    to_code TEXT NOT NULL,
                    factor_milli INTEGER NOT NULL,
                    UNIQUE(from_code, to_code)
                );
                CREATE INDEX IF NOT EXISTS idx_stock_movements_created
                    ON stock_movements(created_at);
                CREATE INDEX IF NOT EXISTS idx_sync_outbox_pending
                    ON sync_outbox(synced_at);
                CREATE INDEX IF NOT EXISTS idx_tax_rates_branch ON tax_rates(branch_id);
                CREATE INDEX IF NOT EXISTS idx_money_movements_day
                    ON money_movements(created_at);
                CREATE INDEX IF NOT EXISTS idx_payable_party ON payable_ledger(party_id);
                "#,
            )
            .map_err(|e| e.to_string())?;

        // LAN: phones paired to this Server (one token each, revocable), the
        // staff signed in on them, and the answer to every write a phone
        // sent (so a retry after a lost reply is answered, not done twice).
        // `lan_outbox` is the phone's side: writes waiting for the Server.
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS lan_devices (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL DEFAULT '',
                    token_hash TEXT NOT NULL UNIQUE,
                    paired_at TEXT NOT NULL,
                    last_seen_at TEXT NOT NULL DEFAULT '',
                    revoked_at TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS lan_sessions (
                    token_hash TEXT PRIMARY KEY,
                    device_id TEXT NOT NULL,
                    staff_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS lan_replies (
                    request_id TEXT PRIMARY KEY,
                    device_id TEXT NOT NULL,
                    status INTEGER NOT NULL,
                    body TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS lan_outbox (
                    id TEXT PRIMARY KEY,
                    seq INTEGER NOT NULL,
                    path TEXT NOT NULL,
                    body TEXT NOT NULL,
                    label TEXT NOT NULL DEFAULT '',
                    table_id TEXT NOT NULL DEFAULT '',
                    staff_id TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    attempts INTEGER NOT NULL DEFAULT 0,
                    last_error TEXT NOT NULL DEFAULT ''
                );
                -- A waiting write the Server turned down when it finally
                -- arrived (the table was settled at the counter meanwhile,
                -- the dish ran out …). Kept, with the Server's words, until
                -- someone has seen it: money taken offline is never dropped
                -- silently.
                CREATE TABLE IF NOT EXISTS lan_refused (
                    id TEXT PRIMARY KEY,
                    path TEXT NOT NULL,
                    body TEXT NOT NULL,
                    label TEXT NOT NULL DEFAULT '',
                    error TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    refused_at TEXT NOT NULL
                );
                "#,
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('schema_version', '1')",
                [],
            )
            .map_err(|e| e.to_string())?;
        // Always pin to current pre-release schema (drop any leftover 2–17 markers).
        // Written only when it differs: the LAN server opens the books for
        // every request, and an unconditional write was a disk sync each time.
        if self.meta_get("schema_version")?.as_deref() != Some("1") {
            self.meta_set("schema_version", "1")?;
        }
        // Pre-release column adds (no-op when CREATE already has them).
        for stmt in [
            "ALTER TABLE sales ADD COLUMN extras_json TEXT NOT NULL DEFAULT '[]'",
            "ALTER TABLE customers ADD COLUMN credit_limit_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE money_movements ADD COLUMN cleared_at TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE categories ADD COLUMN image_key TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE tables_dining ADD COLUMN occupied_at TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE kot_tickets ADD COLUMN table_id TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE kot_tickets ADD COLUMN total_paise INTEGER NOT NULL DEFAULT 0",
            // Floor areas (Dine In, Parcel, AC …) and the running order a KOT fed.
            "ALTER TABLE tables_dining ADD COLUMN area TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE tables_dining ADD COLUMN paid_at TEXT NOT NULL DEFAULT ''",
            // Dish group label (Half and Full are items of their own) and
            // per-area prices on an item.
            "ALTER TABLE products ADD COLUMN group_name TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE products ADD COLUMN area_prices_json TEXT NOT NULL DEFAULT '[]'",
            // Dish / resold good / raw material (existing items are dishes).
            "ALTER TABLE products ADD COLUMN kind TEXT NOT NULL DEFAULT 'dish'",
            "ALTER TABLE kot_tickets ADD COLUMN order_id TEXT NOT NULL DEFAULT ''",
            // Kitchen calls tickets by number; a cancel slip is its own kind;
            // the KOT carries who sent it.
            "ALTER TABLE kot_tickets ADD COLUMN kot_no INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE kot_tickets ADD COLUMN kind TEXT NOT NULL DEFAULT 'kot'",
            "ALTER TABLE kot_tickets ADD COLUMN created_by TEXT NOT NULL DEFAULT ''",
            // Settle & Save: what the guest handed over, change returned, tip.
            "ALTER TABLE sales ADD COLUMN tendered_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN change_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN tip_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN tip_method TEXT NOT NULL DEFAULT ''",
            // Bill discount taken off the lines, and why (reports, audit).
            "ALTER TABLE sales ADD COLUMN discount_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN discount_reason TEXT NOT NULL DEFAULT ''",
            // Shop-narrowed unit pick-lists (disabled units still convert).
            "ALTER TABLE units ADD COLUMN picker_enabled INTEGER NOT NULL DEFAULT 1",
            // Menu timings / happy-hour (Mon=1…Sun=7 windows).
            "ALTER TABLE products ADD COLUMN schedule_json TEXT NOT NULL DEFAULT '[]'",
            // KDS target prep minutes (ageing colours).
            "ALTER TABLE kitchen_stations ADD COLUMN target_prep_mins INTEGER NOT NULL DEFAULT 15",
            // Size / topping choices frozen on the sale line (reports).
            "ALTER TABLE sale_lines ADD COLUMN modifiers_json TEXT NOT NULL DEFAULT '[]'",
        ] {
            let _ = self.conn.execute(stmt, []);
        }
        // One-shot: open KOTs left on free tables (paid without kitchen Done)
        // stuck the next seating on "In kitchen". Close them.
        let _ = self.conn.execute(
            "UPDATE kot_tickets SET status = 'completed'
              WHERE status NOT IN ('completed', 'cancelled')
                AND table_id != ''
                AND table_id IN (
                    SELECT id FROM tables_dining WHERE status = 'available'
                )",
            [],
        );
        // Combos / BOGO / percent schemes (Local).
        let _ = self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schemes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                trigger_json TEXT NOT NULL DEFAULT '{}',
                reward_json TEXT NOT NULL DEFAULT '{}',
                active INTEGER NOT NULL DEFAULT 1
            );
            "#,
        );

        self.conn
            .execute_batch(
                r#"
                -- A table's open order between KOTs and payment
                -- (BUSINESS_BOOKS.md § Open orders are not books yet).
                -- Drawer handovers (BUSINESS_BOOKS.md § Shift and day-end cash).
                CREATE TABLE IF NOT EXISTS shift_closes (
                    id TEXT PRIMARY KEY,
                    started_at TEXT NOT NULL,
                    ended_at TEXT NOT NULL,
                    closed_by TEXT NOT NULL,
                    bills INTEGER NOT NULL,
                    cash_paise INTEGER NOT NULL,
                    upi_paise INTEGER NOT NULL,
                    card_paise INTEGER NOT NULL,
                    cash_tips_paise INTEGER NOT NULL,
                    cash_out_paise INTEGER NOT NULL,
                    float_paise INTEGER NOT NULL,
                    expected_paise INTEGER NOT NULL,
                    counted_paise INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS running_orders (
                    id TEXT PRIMARY KEY,
                    table_id TEXT NOT NULL,
                    status TEXT NOT NULL,
                    opened_at TEXT NOT NULL,
                    opened_by TEXT NOT NULL DEFAULT '',
                    printed_at TEXT NOT NULL DEFAULT '',
                    closed_at TEXT NOT NULL DEFAULT '',
                    sale_id TEXT NOT NULL DEFAULT '',
                    merged_into TEXT NOT NULL DEFAULT ''
                );
                CREATE INDEX IF NOT EXISTS idx_running_orders_table
                    ON running_orders(table_id, status);
                CREATE TABLE IF NOT EXISTS running_lines (
                    id TEXT PRIMARY KEY,
                    order_id TEXT NOT NULL,
                    kot_id TEXT NOT NULL DEFAULT '',
                    product_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    unit_price_paise INTEGER NOT NULL,
                    qty_milli INTEGER NOT NULL,
                    unit_label TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT '',
                    cancelled INTEGER NOT NULL DEFAULT 0,
                    complimentary INTEGER NOT NULL DEFAULT 0,
                    reason TEXT NOT NULL DEFAULT '',
                    prepared INTEGER NOT NULL DEFAULT 0,
                    changed_by TEXT NOT NULL DEFAULT '',
                    approved_by TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    created_by TEXT NOT NULL DEFAULT '',
                    seq INTEGER NOT NULL DEFAULT 0,
                    seat_no INTEGER NOT NULL DEFAULT 0,
                    held INTEGER NOT NULL DEFAULT 0,
                    course_no INTEGER NOT NULL DEFAULT 0
                );
                CREATE INDEX IF NOT EXISTS idx_running_lines_order
                    ON running_lines(order_id);
                -- Table bookings: who, how many, when; booked → seated / cancelled.
                CREATE TABLE IF NOT EXISTS reservations (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    phone TEXT NOT NULL DEFAULT '',
                    guests INTEGER NOT NULL DEFAULT 0,
                    at TEXT NOT NULL,
                    table_name TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL DEFAULT 'booked',
                    created_at TEXT NOT NULL,
                    created_by TEXT NOT NULL DEFAULT ''
                );
                -- Who took a delivery order out.
                CREATE TABLE IF NOT EXISTS sale_delivery (
                    sale_id TEXT PRIMARY KEY,
                    delivery_by TEXT NOT NULL
                );
                -- Companion tables made by splitting a bill (removed once paid).
                CREATE TABLE IF NOT EXISTS split_tables (
                    table_id TEXT PRIMARY KEY,
                    parent_id TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS notify_outbox (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    channel TEXT NOT NULL DEFAULT '',
                    to_addr TEXT NOT NULL DEFAULT '',
                    template_key TEXT NOT NULL DEFAULT '',
                    payload_json TEXT NOT NULL DEFAULT '{}',
                    status TEXT NOT NULL DEFAULT 'queued',
                    last_error TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS entry_attachments (
                    id TEXT PRIMARY KEY,
                    entity_kind TEXT NOT NULL,
                    entity_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    kind TEXT NOT NULL,
                    label TEXT NOT NULL DEFAULT '',
                    body TEXT NOT NULL DEFAULT '',
                    stored_name TEXT NOT NULL DEFAULT '',
                    mime TEXT NOT NULL DEFAULT '',
                    bytes INTEGER NOT NULL DEFAULT 0,
                    actor_id TEXT NOT NULL DEFAULT '',
                    actor_name TEXT NOT NULL DEFAULT ''
                );
                CREATE INDEX IF NOT EXISTS idx_attach_entity
                    ON entry_attachments(entity_kind, entity_id);
                "#,
            )
            .map_err(|e| e.to_string())?;
        self.conn.execute_batch(recipes::SCHEMA).map_err(|e| e.to_string())?;
        // Local/LAN ops tables + column adds on existing books (after CREATE).
        self.migrate_ops()?;
        // Lookups every sync and every screen makes. Without them a year of
        // bills made each device's sync scan every bill line and ledger row
        // (seconds per sync); `a_busy_years_books_still_sync_quickly` guards
        // this. Built once on existing books, then free.
        self.conn
            .execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_sale_lines_sale ON sale_lines(sale_id);
                CREATE INDEX IF NOT EXISTS idx_sale_payments_sale ON sale_payments(sale_id);
                CREATE INDEX IF NOT EXISTS idx_sales_created ON sales(created_at);
                CREATE INDEX IF NOT EXISTS idx_sale_revisions_sale ON sale_revisions(sale_id);
                CREATE INDEX IF NOT EXISTS idx_credit_ledger_customer ON credit_ledger(customer_id);
                CREATE INDEX IF NOT EXISTS idx_credit_ledger_created ON credit_ledger(created_at);
                CREATE INDEX IF NOT EXISTS idx_credit_ledger_sale ON credit_ledger(sale_id);
                CREATE INDEX IF NOT EXISTS idx_kot_tickets_table ON kot_tickets(table_id, created_at);
                CREATE INDEX IF NOT EXISTS idx_kot_tickets_open ON kot_tickets(created_at)
                    WHERE status NOT IN ('completed', 'cancelled');
                CREATE INDEX IF NOT EXISTS idx_kot_tickets_sale ON kot_tickets(sale_id);
                CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_events(created_at);
                CREATE INDEX IF NOT EXISTS idx_expense_payments_expense ON expense_payments(expense_id);
                CREATE INDEX IF NOT EXISTS idx_purchase_lines_purchase ON purchase_lines(purchase_id);
                CREATE INDEX IF NOT EXISTS idx_running_lines_created ON running_lines(created_at);
                -- `created_at LIKE '2026-10-25%'` (a day, a month) can only
                -- use a NOCASE index while LIKE is case-insensitive.
                CREATE INDEX IF NOT EXISTS idx_sales_day ON sales(created_at COLLATE NOCASE);
                CREATE INDEX IF NOT EXISTS idx_expenses_day ON expenses(created_at COLLATE NOCASE);
                CREATE INDEX IF NOT EXISTS idx_money_movements_like ON money_movements(created_at COLLATE NOCASE);
                CREATE INDEX IF NOT EXISTS idx_credit_ledger_day ON credit_ledger(created_at COLLATE NOCASE);
                CREATE INDEX IF NOT EXISTS idx_payable_ledger_day ON payable_ledger(created_at COLLATE NOCASE);
                "#,
            )
            .map_err(|e| e.to_string())?;

        for (key, value) in [
            ("venue_mode", "restaurant"),
            ("onboarding_complete", "0"),
            ("client_paired", "0"),
            ("next_invoice_seq", "1"),
            ("pin_reset_required", "0"),
            ("entity_template", "khaynjo_restaurant"),
            ("app", APP_MARK),
        ] {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO meta(key, value) VALUES(?1, ?2)",
                    params![key, value],
                )
                .map_err(|e| e.to_string())?;
        }

        let org_id = match self.meta_get("org_id")?.filter(|s| !s.trim().is_empty()) {
            Some(id) => id,
            None => {
                let id = new_id();
                self.meta_set("org_id", &id)?;
                id
            }
        };
        // The business name is only ever what the owner typed: nothing is
        // written for them (a stored "My business" read as if they had saved
        // it, and showed in the header as their business).

        let branch_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM branches", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if branch_count == 0 {
            self.conn
                .execute(
                    "INSERT INTO branches(
                        id, org_id, name, code, active, is_this_device,
                        last_snapshot_json, last_sync_at
                     ) VALUES(?1, ?2, 'Main', 'MAIN', 1, 1, '', '')",
                    params![new_id(), org_id],
                )
                .map_err(|e| e.to_string())?;
        }
        let active = self.meta_get("active_branch_id")?.unwrap_or_default();
        let active_ok: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM branches WHERE id = ?1",
                params![active],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if active.trim().is_empty() || active_ok == 0 {
            let fallback: Option<String> = self
                .conn
                .query_row(
                    "SELECT id FROM branches ORDER BY is_this_device DESC, name LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            if let Some(id) = fallback {
                self.meta_set("active_branch_id", &id)?;
            }
        }
        let branch_id = self.meta_get("active_branch_id")?.unwrap_or_default();

        for table in [
            "products",
            "sales",
            "tables_dining",
            "kot_tickets",
            "expenses",
            "customers",
        ] {
            self.conn
                .execute(
                    &format!(
                        "UPDATE {table} SET branch_id = ?1
                         WHERE branch_id IS NULL OR branch_id = ''"
                    ),
                    params![branch_id],
                )
                .map_err(|e| e.to_string())?;
        }

        let station_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM kitchen_stations WHERE branch_id = ?1",
                params![branch_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if station_count == 0 {
            for (i, name) in ["Hot", "Cold"].iter().enumerate() {
                self.conn
                    .execute(
                        "INSERT INTO kitchen_stations(id, branch_id, name, sort_order)
                         VALUES(?1, ?2, ?3, ?4)",
                        params![new_id(), branch_id, name, i as i64],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }

        let country = match self.meta_get("country_code")?.filter(|c| !c.trim().is_empty()) {
            Some(c) => c,
            None => {
                self.meta_set("country_code", fiscal::DEFAULT_COUNTRY)?;
                fiscal::DEFAULT_COUNTRY.to_string()
            }
        };
        let profile = fiscal::country_or_generic(&country);
        for (key, value) in [
            ("tax_pricing", profile.tax_pricing.as_str()),
            ("tax_label", profile.tax_name),
            ("business_tax_id", ""),
            ("business_id_label", profile.business_id_label),
            ("digital_pay_label", profile.digital_pay_label),
            ("credit_label", profile.credit_label),
            ("date_order", profile.date_order),
            ("decimal_sep", profile.decimal_sep),
            ("thousands_sep", profile.thousands_sep),
            ("service_charge_bps", "0"),
        ] {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO meta(key, value) VALUES(?1, ?2)",
                    params![key, value],
                )
                .map_err(|e| e.to_string())?;
        }

        let rate_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM tax_rates WHERE branch_id = ?1",
                params![branch_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if rate_count == 0 {
            self.seed_tax_rates(&branch_id, profile)?;
        }

        self.seed_builtin_units()?;

        for table in [
            "sales",
            "credit_ledger",
            "payable_ledger",
            "purchases",
            "expenses",
            "money_movements",
            "stock_movements",
        ] {
            self.install_event_times_trigger(table)?;
        }
        Ok(())
    }

    /// Fill blank `user_at` / `device_at` from `created_at` on insert (safety net).
    fn install_event_times_trigger(&self, table: &str) -> Result<(), String> {
        self.conn
            .execute_batch(&format!(
                "CREATE TRIGGER IF NOT EXISTS trg_{table}_event_times
                 AFTER INSERT ON {table}
                 FOR EACH ROW WHEN NEW.user_at = ''
                 BEGIN
                   UPDATE {table} SET
                     user_at = NEW.created_at,
                     device_at = CASE WHEN NEW.device_at = '' THEN NEW.created_at ELSE NEW.device_at END
                   WHERE rowid = NEW.rowid;
                 END;"
            ))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn seed_builtin_units(&self) -> Result<(), String> {
        let builtins: &[(&str, &str)] = &[
            ("pc", "Piece"),
            ("kg", "Kilogram"),
            ("g", "Gram"),
            ("t", "Tonne"),
            ("lb", "Pound"),
            ("oz", "Ounce"),
            ("l", "Litre"),
            ("ml", "Millilitre"),
            ("m", "Metre"),
            ("cm", "Centimetre"),
            ("mm", "Millimetre"),
            ("in", "Inch"),
            ("ft", "Foot"),
            ("dozen", "Dozen"),
        ];
        for (code, name) in builtins {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO units(id, code, name, builtin) VALUES(?1, ?2, ?3, 1)",
                    params![new_id(), code, name],
                )
                .map_err(|e| e.to_string())?;
        }
        // to_qty_milli = from_qty_milli * factor_milli / 1000
        let convs: &[(&str, &str, i64)] = &[
            ("kg", "g", 1_000_000),    // 1 kg = 1000 g
            ("g", "kg", 1),            // 1000 g → 1 kg (factor 0.001 → milli 1)
            ("l", "ml", 1_000_000),
            ("ml", "l", 1),
            ("m", "cm", 100_000),
            ("cm", "m", 10),
            ("mm", "cm", 100),
            ("cm", "mm", 10_000),
            ("in", "cm", 2_540),
            ("cm", "in", 394),
            ("ft", "cm", 30_480),
            ("cm", "ft", 33),
            ("ft", "in", 12_000),
            ("in", "ft", 83),
            ("t", "kg", 1_000_000),
            ("kg", "t", 1),
            ("lb", "kg", 454),
            ("kg", "lb", 2_205),
            ("oz", "kg", 28),
            ("kg", "oz", 35_274),
            ("lb", "oz", 16_000),
            ("oz", "lb", 63),
            ("dozen", "pc", 12_000),
        ];
        for (from, to, factor) in convs {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO unit_conversions(id, from_code, to_code, factor_milli)
                     VALUES(?1, ?2, ?3, ?4)",
                    params![new_id(), from, to, factor],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Upsert a country's rates onto a branch **by code**, so re-applying a
    /// pack refreshes the official rates without touching rates the seller
    /// added themselves.
    fn seed_tax_rates(&self, branch_id: &str, profile: &CountryProfile) -> Result<(), String> {
        for seed in profile.default_rates {
            let existing: Option<String> = self
                .conn
                .query_row(
                    "SELECT id FROM tax_rates WHERE branch_id = ?1 AND code = ?2",
                    params![branch_id, seed.code],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            match existing {
                Some(id) => {
                    self.conn
                        .execute(
                            "UPDATE tax_rates SET name = ?1, bps = ?2, active = 1 WHERE id = ?3",
                            params![seed.name, seed.bps, id],
                        )
                        .map_err(|e| e.to_string())?;
                }
                None => {
                    self.conn
                        .execute(
                            "INSERT INTO tax_rates(id, branch_id, name, code, bps, active)
                             VALUES(?1, ?2, ?3, ?4, ?5, 1)",
                            params![new_id(), branch_id, seed.name, seed.code, seed.bps],
                        )
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    fn ensure_seed_data(&mut self) -> Result<(), String> {
        // Remove placeholder catalogue from earlier prototypes (Item A–E).
        self.conn
            .execute("DELETE FROM products WHERE id LIKE 'seed-item-%'", [])
            .map_err(|e| e.to_string())?;

        // Fresh DB: default UI language from the OS; user confirms on first launch.
        if self.meta_get("language")?.is_none() {
            let code = locale::detect_platform_language();
            self.meta_set("language", &code)?;
        }
        if self.meta_get("currency")?.is_none() {
            self.meta_set("currency", "INR")?;
        }

        let staff_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM staff", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if staff_count == 0 {
            let hash = hash_pin("0000")?;
            self.conn
                .execute(
                    "INSERT INTO staff(id, name, role, pin_hash, active, role_title)
                     VALUES(?1, ?2, ?3, ?4, 1, ?5)",
                    params![
                        new_id(),
                        "Owner",
                        StaffRole::Owner.as_str(),
                        hash,
                        StaffRole::Owner.default_title()
                    ],
                )
                .map_err(|e| e.to_string())?;
            // Fresh shop: Owner must choose their PIN before anyone else is created.
            self.meta_set("owner_setup_complete", "0")?;
        } else if self.meta_get("owner_setup_complete")?.is_none() {
            // Existing shops already had an Owner PIN in use.
            self.meta_set("owner_setup_complete", "1")?;
        }

        let table_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tables_dining", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if table_count == 0 {
            let branch_id = self.active_branch_id()?;
            for n in 1..=6 {
                self.conn
                    .execute(
                        "INSERT INTO tables_dining(id, name, status, active_sale_id, branch_id)
                         VALUES(?1, ?2, ?3, NULL, ?4)",
                        params![
                            new_id(),
                            format!("T{n}"),
                            TableStatus::Available.as_str(),
                            branch_id
                        ],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn owner_setup_complete(&self) -> Result<bool, String> {
        Ok(self
            .meta_get("owner_setup_complete")?
            .map(|v| v.trim() == "1")
            .unwrap_or(true))
    }

    /// First-run on Server: Owner chooses their PIN. Marks setup complete and
    /// returns the Owner so the session can start. Does **not** mark onboarding
    /// complete — that happens after Fresh/Book wizard.
    pub fn complete_owner_setup(&self, pin: &str) -> Result<StaffMember, String> {
        if self.owner_setup_complete()? {
            return Err("Owner PIN is already set".into());
        }
        if !(4..=12).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("PIN must be 4–12 digits".into());
        }
        let owner = self
            .list_staff()?
            .into_iter()
            .find(|s| s.is_owner())
            .ok_or_else(|| "Owner account missing".to_string())?;
        let hash = hash_pin(pin)?;
        self.conn
            .execute(
                "UPDATE staff SET pin_hash = ?1 WHERE id = ?2",
                params![hash, owner.id],
            )
            .map_err(|e| e.to_string())?;
        self.meta_set("owner_setup_complete", "1")?;
        self.meta_set("onboarding_complete", "0")?;
        self.meta_set("pin_reset_required", "0")?;
        self.ensure_server_pairing_code()?;
        self.audit(&owner, "owner.setup", "Owner PIN set on this shop")?;
        self.list_staff()?
            .into_iter()
            .find(|s| s.id == owner.id)
            .ok_or_else(|| "Owner account missing after setup".to_string())
    }

    /// True while a forced PIN reset is still unanswered. The till must not
    /// open on a known fallback PIN while this flag is set.
    pub fn pin_reset_required(&self) -> Result<bool, String> {
        Ok(self
            .meta_get("pin_reset_required")?
            .map(|v| v.trim() == "1")
            .unwrap_or(false))
    }

    /// Owner picks a new PIN after a migration reset. Unlike first-run setup
    /// this runs on a shop that is already trading, so only the Owner's own PIN
    /// moves — staff PINs are re-issued from Team.
    pub fn complete_pin_reset(&self, pin: &str) -> Result<StaffMember, String> {
        if !self.pin_reset_required()? {
            return Err("No PIN reset is pending".into());
        }
        if !(4..=12).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("PIN must be 4–12 digits".into());
        }
        if pin == "0000" {
            return Err("Choose a PIN other than 0000".into());
        }
        let owner = self
            .list_staff()?
            .into_iter()
            .find(|s| s.is_owner())
            .ok_or_else(|| "Owner account missing".to_string())?;
        let hash = hash_pin(pin)?;
        self.conn
            .execute(
                "UPDATE staff SET pin_hash = ?1 WHERE id = ?2",
                params![hash, owner.id],
            )
            .map_err(|e| e.to_string())?;
        self.meta_set("pin_reset_required", "0")?;
        self.audit(&owner, "owner.pin_reset", "Owner PIN re-set after upgrade")?;
        self.list_staff()?
            .into_iter()
            .find(|s| s.id == owner.id)
            .ok_or_else(|| "Owner account missing after reset".to_string())
    }

    pub fn onboarding_complete(&self) -> Result<bool, String> {
        Ok(self
            .meta_get("onboarding_complete")?
            .map(|v| v.trim() == "1")
            .unwrap_or(true))
    }

    pub fn complete_onboarding(&self) -> Result<(), String> {
        self.meta_set("onboarding_complete", "1")
    }

    pub fn venue_mode(&self) -> Result<VenueMode, String> {
        Ok(VenueMode::parse(
            &self
                .meta_get("venue_mode")?
                .unwrap_or_else(|| "restaurant".into()),
        ))
    }

    pub fn set_venue_mode(&self, mode: VenueMode) -> Result<(), String> {
        self.meta_set("venue_mode", mode.as_str())
    }

    pub fn entity_template_id(&self) -> Result<String, String> {
        Ok(entity_template::KHAYNJO.id.into())
    }

    pub fn entity_template(&self) -> Result<&'static EntityTemplate, String> {
        Ok(&entity_template::KHAYNJO)
    }

    /// Lock this binary to Khaynjo (ignores other template ids).
    pub fn apply_entity_template(&self, _template_id: &str) -> Result<&'static EntityTemplate, String> {
        self.assert_server_writer("Apply start template")?;
        let tmpl = &entity_template::KHAYNJO;
        self.meta_set("entity_template", tmpl.id)?;
        self.meta_set("qty_unit_hint", tmpl.qty_unit_hint)?;
        self.meta_set("lane_declared_label", tmpl.lane_declared_label)?;
        self.meta_set("lane_owner_label", tmpl.lane_owner_label)?;
        self.set_venue_mode(tmpl.venue_mode)?;
        let _ = self.seed_template_products(tmpl)?;
        let _ = self.seed_template_parties(tmpl)?;
        Ok(tmpl)
    }

    fn seed_template_products(&self, tmpl: &EntityTemplate) -> Result<usize, String> {
        if tmpl.seed_products.is_empty() {
            return Ok(0);
        }
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE archived = 0",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if count > 0 {
            return Ok(0);
        }
        for seed in tmpl.seed_products {
            let mut product = Product::new_basic(
                new_id(),
                seed.name,
                entity_template::seed_product_price(seed),
            );
            product.unit_label = seed.unit_label.into();
            self.upsert_product(&product)?;
        }
        Ok(tmpl.seed_products.len())
    }

    fn seed_template_parties(&self, tmpl: &EntityTemplate) -> Result<usize, String> {
        if tmpl.seed_parties.is_empty() {
            return Ok(0);
        }
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM customers WHERE active = 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if count > 0 {
            return Ok(0);
        }
        for party in tmpl.seed_parties {
            self.add_customer_kind(party.name, "", party.kind_label)?;
        }
        Ok(tmpl.seed_parties.len())
    }

    /// No sample catalogue — owners add their own rates/items.
    /// Kept as a no-op so older UI/call sites stay harmless.
    pub fn seed_starter_products(&self) -> Result<usize, String> {
        self.assert_server_writer("Seed example items")?;
        Ok(0)
    }

    pub fn client_paired(&self) -> Result<bool, String> {
        Ok(self
            .meta_get("client_paired")?
            .map(|v| v.trim() == "1")
            .unwrap_or(false))
    }

    pub fn server_url(&self) -> Result<Option<String>, String> {
        Ok(self
            .meta_get("server_url")?
            .filter(|s| !s.trim().is_empty()))
    }

    pub fn pairing_code(&self) -> Result<String, String> {
        if let Some(code) = self.meta_get("pairing_code")?.filter(|s| !s.trim().is_empty()) {
            return Ok(code);
        }
        self.ensure_server_pairing_code()?;
        self.meta_get("pairing_code")?
            .ok_or_else(|| "pairing code missing".into())
    }

    pub fn ensure_server_pairing_code(&self) -> Result<String, String> {
        if let Some(code) = self.meta_get("pairing_code")?.filter(|s| !s.trim().is_empty()) {
            return Ok(code);
        }
        let mut n = [0u8; 3];
        getrandom_salt(&mut n)?;
        let code = format!("{:06}", (u32::from(n[0]) << 16 | u32::from(n[1]) << 8 | u32::from(n[2])) % 1_000_000);
        self.meta_set("pairing_code", &code)?;
        Ok(code)
    }

    /// Phone side: paired with a device token of its own (the pairing code
    /// is used once, at pairing, and is not kept).
    pub fn set_client_device_pairing(&self, server_url: &str, device_token: &str) -> Result<(), String> {
        let url = server_url.trim().trim_end_matches('/');
        if url.is_empty() {
            return Err("Server address required".into());
        }
        self.meta_set("server_url", url)?;
        self.meta_set("device_token", device_token.trim())?;
        self.meta_set("pairing_code", "")?;
        self.meta_set("client_paired", "1")?;
        Ok(())
    }

    pub fn client_device_token(&self) -> Result<Option<String>, String> {
        Ok(self.meta_get("device_token")?.filter(|s| !s.trim().is_empty()))
    }

    /// How often (seconds) the phone worker pulls the Server's books.
    pub fn sync_interval_secs(&self) -> Result<u64, String> {
        Ok(self
            .meta_get("sync_interval_secs")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(20))
    }

    pub fn set_sync_interval_secs(&self, secs: u64) -> Result<(), String> {
        self.meta_set("sync_interval_secs", &secs.to_string())
    }

    /// When this device last synced with the Server (RFC3339), kept through a restart.
    pub fn set_device_last_seen(&self, at: &str) -> Result<(), String> {
        self.meta_set("device_last_seen", at)
    }

    pub fn device_last_seen(&self) -> Result<Option<String>, String> {
        Ok(self.meta_get("device_last_seen")?.filter(|s| !s.trim().is_empty()))
    }

    /// Server side: record that a paired device made contact.
    pub fn touch_device_seen(&self, device_id: &str) -> Result<(), String> {
        let now = chrono::Local::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE lan_devices SET last_seen_at = ?1 WHERE id = ?2",
                rusqlite::params![now, device_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_client_pairing(&self) -> Result<(), String> {
        self.meta_set("device_token", "")?;
        self.meta_set("client_paired", "0")?;
        self.meta_set("server_url", "")?;
        self.meta_set("paired_server_id", "")?;
        Ok(())
    }

    /// The shop server this device paired with, by its id — never by its
    /// name or address. How the device finds it again when the address
    /// changes (a new address after a power cut, the books on a new computer).
    pub fn paired_server_id(&self) -> Result<Option<String>, String> {
        Ok(self.meta_get("paired_server_id")?.filter(|s| !s.trim().is_empty()))
    }

    pub fn set_paired_server_id(&self, id: &str) -> Result<(), String> {
        self.meta_set("paired_server_id", id.trim())
    }

    /// The paired server answered from another address: the pairing stays,
    /// only where to reach it changes.
    pub fn set_server_url(&self, server_url: &str) -> Result<(), String> {
        let url = server_url.trim().trim_end_matches('/');
        if url.is_empty() {
            return Err("Server address required".into());
        }
        self.meta_set("server_url", url)
    }

    /// These books' id on the shop Wi‑Fi, made once and kept in the books:
    /// a backup restored on a new computer is the same server to every
    /// paired device, which finds it again by this id.
    pub fn lan_server_id(&self) -> Result<String, String> {
        if let Some(id) = self.meta_get("lan_server_id")?.filter(|s| !s.trim().is_empty()) {
            return Ok(id);
        }
        // INSERT OR IGNORE: two first callers at once still agree on one id.
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('lan_server_id', ?1)",
                params![new_id()],
            )
            .map_err(|e| e.to_string())?;
        self.meta_get("lan_server_id")?
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "server id missing".into())
    }

    /// Changes whenever another connection commits to these books (SQLite's
    /// `data_version`) — the cheap "did anything change?" check.
    pub fn data_version(&self) -> Result<i64, String> {
        self.conn
            .query_row("PRAGMA data_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())
    }

    /// Client terminals are not the shop host — skip local Owner/onboarding wizards.
    pub fn prepare_as_client_terminal(&self) -> Result<(), String> {
        self.meta_set("owner_setup_complete", "1")?;
        self.meta_set("onboarding_complete", "1")?;
        Ok(())
    }

    /// Client may sell only when paired; Server always may.
    pub fn can_finalize_on_device(&self) -> Result<bool, String> {
        match self.device_role()?.as_deref() {
            Some("client") => self.client_paired(),
            Some("server") | None => Ok(true),
            Some(_) => Ok(true),
        }
    }

    /// True when this device is a paired-terminal Client rather than the till.
    pub fn is_client_device(&self) -> Result<bool, String> {
        Ok(self.device_role()?.as_deref() == Some("client"))
    }

    /// The Server owns every book: catalogue, stock, staff, branches and fiscal
    /// settings. A Client's tables are a sync cache that the next snapshot
    /// overwrites, so writing them locally silently loses the edit. Sell, void,
    /// refund and amend go over LAN instead; everything else is refused here so
    /// no UI path can bypass it.
    ///
    /// The `_from_sync` writers deliberately skip this check — that is the
    /// Server's own data arriving.
    fn assert_server_writer(&self, what: &str) -> Result<(), String> {
        if self.is_client_device()? {
            return Err(format!("{what} on the shop device — this is an extra till"));
        }
        Ok(())
    }

    pub fn outstanding_credit_total(&self) -> Result<i64, String> {
        let customers = self.list_customers()?;
        Ok(customers.iter().map(|c| c.balance_paise.max(0)).sum())
    }

    /// Seed opening credit balance as an explicit ledger entry.
    pub fn seed_opening_credit(
        &self,
        customer_id: &str,
        amount_paise: i64,
        actor: &StaffMember,
    ) -> Result<(), String> {
        if amount_paise <= 0 {
            return Err("Opening balance must be > 0".into());
        }
        let times = EventTimes::capture(None);
        self.conn
            .execute(
                "INSERT INTO credit_ledger(
                    id, customer_id, created_at, kind, amount_paise,
                    sale_id, pay_method, actor_id, actor_name, detail,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, 'opening', ?4, NULL, NULL, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    new_id(),
                    customer_id,
                    times.user_at,
                    amount_paise,
                    actor.id,
                    actor.name,
                    "Opening balance from book",
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "credit.opening",
            &format!("{customer_id} {amount_paise}"),
        )?;
        Ok(())
    }

    fn sale_tenders(&self, sale_id: &str) -> Result<Vec<(String, i64)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT pay_method, amount_paise FROM sale_payments
                 WHERE sale_id = ?1 ORDER BY rowid",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![sale_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Tax / account invoice text (same spine as receipt; clearer invoice header).
    pub fn format_invoice_text(&self, sale_id: &str) -> Result<String, String> {
        self.format_receipt_text(sale_id)
    }

    pub fn format_receipt_text(&self, sale_id: &str) -> Result<String, String> {
        let sale = self
            .get_sale(sale_id)?
            .ok_or_else(|| "Sale not found".to_string())?;
        let lines = self.sale_lines(sale_id)?;
        let venue = self.venue_title()?;
        let venue = if venue.trim().is_empty() {
            "Your shop"
        } else {
            venue.trim()
        };
        let currency = self.locale_settings()?.currency;
        let fiscal_settings = self.fiscal_settings()?;
        let pay = PayMethod::parse(&sale.pay_method);
        let tmpl = self.entity_template()?;
        let lane = MoneyLane::parse(&sale.money_lane);
        let is_invoice = matches!(lane, MoneyLane::Declared);
        let mut out = String::new();
        out.push_str(venue);
        out.push('\n');
        // Owner's bill format: address/phone lines and the FSSAI licence no.
        for line in self.bill_header_lines()? {
            out.push_str(&line);
            out.push('\n');
        }
        if is_invoice {
            out.push_str("TAX INVOICE\n");
        } else {
            out.push_str("CASH MEMO\n");
        }
        let business_id = fiscal_settings.business_tax_id.trim();
        if !business_id.is_empty() {
            out.push_str(&format!(
                "{} {business_id}\n",
                fiscal_settings.business_id_label
            ));
        }
        if !sale.invoice_no.trim().is_empty() {
            out.push_str("Invoice no: ");
            out.push_str(sale.invoice_no.trim());
            out.push('\n');
        }
        out.push_str("Bill: ");
        out.push_str(&sale.bill_no());
        out.push('\n');
        out.push_str(&fiscal_settings.format_date(&sale.created_at));
        if let Some(clock) = sale.created_at.get(11..19) {
            out.push(' ');
            out.push_str(clock);
        }
        out.push('\n');
        if let Some(cid) = sale.customer_id.as_deref().filter(|s| !s.is_empty()) {
            if let Some(party) = self.get_customer(cid)? {
                out.push_str("Bill to: ");
                out.push_str(&party.name);
                if !party.kind_label.trim().is_empty() {
                    out.push_str(" (");
                    out.push_str(party.kind_label.trim());
                    out.push(')');
                }
                out.push('\n');
                if !party.phone.trim().is_empty() {
                    out.push_str("Phone: ");
                    out.push_str(party.phone.trim());
                    out.push('\n');
                }
            }
        }
        let lane_label = match lane {
            MoneyLane::Declared => tmpl.lane_declared_label,
            MoneyLane::Owner => tmpl.lane_owner_label,
        };
        out.push_str("Bill type: ");
        out.push_str(lane_label);
        out.push('\n');
        {
            let order_kind = crate::domain::OrderType::parse(&sale.order_type);
            out.push_str("Order: ");
            out.push_str(order_kind.label());
            out.push('\n');
            if let Some(tid) = sale.table_id.as_deref().filter(|s| !s.is_empty()) {
                let table_name: Option<String> = self
                    .conn
                    .query_row(
                        "SELECT name FROM tables_dining WHERE id = ?1",
                        params![tid],
                        |r| r.get(0),
                    )
                    .optional()
                    .ok()
                    .flatten();
                if let Some(name) = table_name.filter(|s| !s.trim().is_empty()) {
                    out.push_str("Table: ");
                    out.push_str(name.trim());
                    out.push('\n');
                }
            }
        }
        out.push_str("--------------------\n");
        for line in &lines {
            let unit = Money(line.unit_price_paise).format_localized(&currency, &fiscal_settings);
            let total = Money(line.line_total_paise).format_localized(&currency, &fiscal_settings);
            let qty = format_qty_display(line.qty_milli, line.qty, &line.unit_label);
            out.push_str(&format!("{} {} @ {} = {}\n", line.name, qty, unit, total));
            if !line.note.is_empty() {
                out.push_str(&format!("  ({})\n", line.note));
            }
        }
        out.push_str("--------------------\n");
        if sale.rounding_paise != 0 {
            out.push_str(&format!(
                "Applied {}\nRound {}\n",
                Money(sale.lines_total_paise).format_localized(&currency, &fiscal_settings),
                Money(sale.rounding_paise).format_localized(&currency, &fiscal_settings)
            ));
        }
        // Inclusive shops read the bill top-down (Net → tax → Total, all
        // inside the quoted price); exclusive shops read it as Subtotal plus
        // tax. Either way the components add up to the Total line.
        let tax_label = sale.tax_label();
        if sale.tax_paise > 0 {
            let base_caption = if sale.tax_pricing().is_inclusive() {
                "Net"
            } else {
                "Subtotal"
            };
            out.push_str(&format!(
                "{base_caption} {}\n",
                sale.net().format_localized(&currency, &fiscal_settings)
            ));
            let components = fiscal_settings.tax_components(sale.tax_paise);
            if components.is_empty() {
                out.push_str(&format!(
                    "{tax_label} {}\n",
                    sale.tax().format_localized(&currency, &fiscal_settings)
                ));
            } else {
                // Countries that bill one tax as named halves (India's
                // CGST/SGST) show each part, then the combined figure.
                for (name, amount) in &components {
                    out.push_str(&format!("{name} {}\n", Money(*amount).format_localized(&currency, &fiscal_settings)));
                }
                out.push_str(&format!(
                    "{tax_label} {}\n",
                    sale.tax().format_localized(&currency, &fiscal_settings)
                ));
            }
        }
        if sale.service_paise > 0 {
            out.push_str(&format!(
                "Service {}\n",
                Money(sale.service_paise).format_localized(&currency, &fiscal_settings)
            ));
        }
        out.push_str(&format!(
            "Total {}\n",
            Money(sale.total_paise).format_localized(&currency, &fiscal_settings)
        ));
        let tenders = self.sale_tenders(sale_id)?;
        if tenders.len() > 1 {
            for (method, amount) in tenders {
                let label = PayMethod::parse(&method)
                    .map(|p| p.label_for(&fiscal_settings))
                    .unwrap_or(method);
                out.push_str(&format!(
                    "{label} {}\n",
                    Money(amount).format_localized(&currency, &fiscal_settings)
                ));
            }
        } else {
            out.push_str(&format!(
                "{}\n",
                match pay {
                    Some(p) => p.label_for(&fiscal_settings),
                    None => sale.pay_method.to_uppercase(),
                }
            ));
        }
        if sale.tax_paise > 0 && sale.tax_pricing().is_inclusive() {
            out.push_str(&format!("(Incl. {tax_label})\n"));
        }
        if sale.order_type() != OrderType::Counter {
            out.push_str(&format!("{}\n", sale.order_type().label()));
        }
        if !sale.created_by_name.is_empty() {
            out.push_str(&format!("By {}\n", sale.created_by_name));
        }
        // Settle & Save: cash handed over, change returned, tip.
        let (tendered, change, tip): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COALESCE(tendered_paise, 0), COALESCE(change_paise, 0),
                        COALESCE(tip_paise, 0)
                   FROM sales WHERE id = ?1",
                params![sale_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(|e| e.to_string())?;
        let m = |p: i64| Money(p).format_localized(&currency, &fiscal_settings);
        let (discount, discount_reason): (i64, String) = self
            .conn
            .query_row(
                "SELECT COALESCE(discount_paise, 0), COALESCE(discount_reason, '') FROM sales WHERE id = ?1",
                params![sale_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        if discount > 0 {
            out.push_str(&format!("Discount −{} ({discount_reason})\n", m(discount)));
        }
        if tendered > 0 {
            out.push_str(&format!("Paid {} · Change {}\n", m(tendered), m(change)));
        }
        if tip > 0 {
            out.push_str(&format!("Tip {} (thank you)\n", m(tip)));
        }
        let footer = self.bill_footer()?;
        if !footer.trim().is_empty() {
            out.push_str(footer.trim());
            out.push('\n');
        }
        if is_invoice {
            out.push_str("— End of invoice —\n");
        }
        let attach = self.attachments_summary_text(AttachmentEntity::Sale, sale_id)?;
        if !attach.is_empty() {
            out.push_str(&attach);
        }
        Ok(out)
    }

    pub fn get_customer(&self, id: &str) -> Result<Option<Customer>, String> {
        let row = self
            .conn
            .query_row(
                "SELECT id, name, phone, note, active, COALESCE(kind_label, ''),
                        COALESCE(credit_limit_paise, 0)
                 FROM customers WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((id, name, phone, note, active, kind_label, credit_limit_paise)) = row else {
            return Ok(None);
        };
        let balance_paise = self.customer_balance(&id)?;
        Ok(Some(Customer {
            id: id.clone(),
            name,
            phone,
            note,
            active: active != 0,
            balance_paise,
            kind_label,
            credit_limit_paise,
            loyalty_points: self.loyalty_points(&id).unwrap_or(0),
            wallet_paise: self.wallet_paise(&id).unwrap_or(0),
        }))
    }

    pub fn day_close_summary(&self) -> Result<String, String> {
        let t = self.today_totals()?;
        let currency = self.locale_settings()?.currency;
        let fiscal_settings = self.fiscal_settings()?;
        let outstanding = self.outstanding_credit_total()?;
        let venue = self.venue_title()?;
        let venue = if venue.trim().is_empty() {
            "Your shop"
        } else {
            venue.trim()
        };
        let tmpl = self.entity_template()?;
        let today = Local::now().format("%Y-%m-%d").to_string();
        let bank_net = self.money_movements_net_for_prefix(&format!("{today}%"))?;
        Ok(format!(
            "{venue} — day close\n\
             Sales: {}\n\
             Lines: {}\n\
             Cash: {}\n\
             {}: {}\n\
             Card: {}\n\
             {} sales today: {}\n\
             {} still owed: {}\n\
             {}: {}\n\
             {}: {}\n\
             {}: {}\n\
             Bank net today: {}\n\
             Bank balance: {}\n\
             Cash in hand (till, all recorded): {}\n\
             Rounding: {}\n",
            t.sale_count,
            t.lines_total().format_localized(&currency, &fiscal_settings),
            Money(t.cash_paise).format_localized(&currency, &fiscal_settings),
            fiscal_settings.digital_pay_label(),
            Money(t.upi_paise).format_localized(&currency, &fiscal_settings),
            Money(t.card_paise).format_localized(&currency, &fiscal_settings),
            fiscal_settings.credit_label(),
            Money(t.credit_paise).format_localized(&currency, &fiscal_settings),
            fiscal_settings.credit_label(),
            Money(outstanding).format_localized(&currency, &fiscal_settings),
            tmpl.lane_declared_label,
            Money(t.declared_paise).format_localized(&currency, &fiscal_settings),
            tmpl.lane_owner_label,
            Money(t.owner_lane_paise).format_localized(&currency, &fiscal_settings),
            self.tax_total_caption(&fiscal_settings),
            t.tax().format_localized(&currency, &fiscal_settings),
            Money(bank_net).format_localized(&currency, &fiscal_settings),
            Money(self.bank_balance_paise()?).format_localized(&currency, &fiscal_settings),
            Money(self.till_balance_paise()?).format_localized(&currency, &fiscal_settings),
            Money(t.rounding_paise).format_localized(&currency, &fiscal_settings),
        ))
        .map(|summary| {
            // Restaurant day close also names what was not a sale: tips
            // (staff's money) and what left the kitchen unbilled.
            let tips = self.today_tips_paise().unwrap_or(0);
            let mut out = summary;
            if tips > 0 {
                out.push_str(&format!(
                    "\nTips (staff's, not sales): {}",
                    Money(tips).format_localized(&currency, &fiscal_settings)
                ));
            }
            // Only when something was cancelled or given free today.
            if let Ok(report) = self.cancellations_and_nc_text()
                && report.lines().any(|l| l.starts_with("- "))
            {
                out.push('\n');
                out.push_str(report.trim_end());
            }
            // Who sold, how, when, what — and the discounts given.
            if let Ok(report) = self.restaurant_day_report_text() {
                out.push_str(report.trim_end());
            }
            out
        })
    }

    /// Record that the shop closed the day, and when.
    ///
    /// Deliberately *not* a freeze: a stall that sells one more chai after
    /// cashing up must still be able to ring it, so nothing is locked. What this
    /// buys is a timestamp plus a `day.close` audit line carrying the summary
    /// the owner actually signed off on. Returns the recorded time.
    pub fn commit_day_close(&self, actor: &StaffMember) -> Result<String, String> {
        self.commit_day_close_with_count(actor, None)
    }

    /// `counted_cash_paise`: optional drawer count. Compared to today's cash tender total.
    pub fn commit_day_close_with_count(
        &self,
        actor: &StaffMember,
        counted_cash_paise: Option<i64>,
    ) -> Result<String, String> {
        self.assert_server_writer("Record the day close")?;
        let mut summary = self.day_close_summary()?;
        // Expected drawer movement today: cash tenders in, minus cash
        // that left (cash expenses, cash payable settlements, till→bank).
        let today = Local::now().format("%Y-%m-%d").to_string();
        // The opening float stays in the drawer and cash tips sit there too
        // (they are the staff's) — both belong in what the drawer should hold.
        let expected_cash = self.opening_float_paise()?
            + self.today_totals()?.cash_paise
            + self.today_cash_tips_paise()?
            - self.cash_out_paise(Some(&today))?;
        if let Some(counted) = counted_cash_paise {
            let variance = counted - expected_cash;
            let currency = self.locale_settings()?.currency;
            let fiscal = self.fiscal_settings()?;
            summary.push_str(&format!(
                "Drawer counted: {}\nExpected cash (float + in − out today): {}\nVariance: {}\n",
                Money(counted).format_localized(&currency, &fiscal),
                Money(expected_cash).format_localized(&currency, &fiscal),
                Money(variance).format_localized(&currency, &fiscal),
            ));
            self.meta_set("last_day_close_counted_cash", &counted.to_string())?;
            self.meta_set("last_day_close_cash_variance", &variance.to_string())?;
        }
        let now = Local::now().to_rfc3339();
        self.meta_set("last_day_close_at", &now)?;
        let detail = summary
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        self.audit(actor, "day.close", &detail)?;
        Ok(now)
    }

    /// When the day close was last recorded, if ever.
    pub fn last_day_close_at(&self) -> Result<Option<String>, String> {
        Ok(self
            .meta_get("last_day_close_at")?
            .filter(|s| !s.trim().is_empty()))
    }

    /// `GST inside sales` vs `VAT collected` — inclusive tax was already in
    /// the price, exclusive tax was charged on top.
    pub fn tax_total_caption(&self, fiscal_settings: &FiscalSettings) -> String {
        if fiscal_settings.tax_pricing.is_inclusive() {
            format!("{} inside sales", fiscal_settings.tax_label())
        } else {
            format!("{} collected", fiscal_settings.tax_label())
        }
    }

    pub fn add_table(&self, name: &str) -> Result<DiningTable, String> {
        self.assert_server_writer("Add tables")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Table name required".into());
        }
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO tables_dining(id, name, status, active_sale_id, branch_id)
                 VALUES(?1, ?2, ?3, NULL, ?4)",
                params![
                    id,
                    name,
                    TableStatus::Available.as_str(),
                    self.active_branch_id()?
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(DiningTable {
            id,
            name: name.into(),
            status: TableStatus::Available,
            active_sale_id: None,
            occupied_at: String::new(),
            running_total_paise: 0,
            area: String::new(),
            paid_at: String::new(),
            floor_state: crate::domain::FloorState::Free,
            waiter_staff_id: String::new(),
            layout_x: 0.0,
            layout_y: 0.0,
            layout_w: 0.0,
            layout_h: 0.0,
        })
    }

    pub fn rename_table(&self, table_id: &str, name: &str) -> Result<(), String> {
        self.assert_server_writer("Rename tables")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Table name required".into());
        }
        let n = self
            .conn
            .execute(
                "UPDATE tables_dining SET name = ?1 WHERE id = ?2",
                params![name, table_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Table not found".into());
        }
        Ok(())
    }

    pub fn delete_table(&self, table_id: &str) -> Result<(), String> {
        self.assert_server_writer("Delete tables")?;
        let n = self
            .conn
            .execute("DELETE FROM tables_dining WHERE id = ?1", params![table_id])
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Table not found".into());
        }
        Ok(())
    }

    /// Apply a remote catalog snapshot (Client cache). Server branch ids mean
    /// nothing here, so cached items are re-tagged to this device's branch.
    pub fn replace_catalog_from_sync(&self, products: &[Product]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM products", [])
            .map_err(|e| e.to_string())?;
        for p in products {
            let mut local = p.clone();
            local.branch_id = String::new();
            self.write_product_row(&local)?;
        }
        Ok(())
    }

    /// Cache the Server's menu groupings / stations so a Client can route KOTs.
    pub fn replace_menu_meta_from_sync(
        &self,
        categories: &[Category],
        stations: &[KitchenStation],
    ) -> Result<(), String> {
        if categories.is_empty() && stations.is_empty() {
            return Ok(());
        }
        let branch_id = self.active_branch_id()?;
        self.conn
            .execute("DELETE FROM categories WHERE branch_id = ?1", params![branch_id])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "DELETE FROM kitchen_stations WHERE branch_id = ?1",
                params![branch_id],
            )
            .map_err(|e| e.to_string())?;
        for c in categories {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO categories(id, branch_id, name, sort_order, image_key)
                     VALUES(?1, ?2, ?3, ?4, ?5)",
                    params![c.id, branch_id, c.name, c.sort_order, c.image_key],
                )
                .map_err(|e| e.to_string())?;
        }
        for s in stations {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO kitchen_stations(id, branch_id, name, sort_order)
                     VALUES(?1, ?2, ?3, ?4)",
                    params![s.id, branch_id, s.name, s.sort_order],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn replace_customers_from_sync(&self, customers: &[Customer]) -> Result<(), String> {
        // Keep ledger; only refresh customer rows for display. Full ledger sync is future work.
        for c in customers {
            self.conn
                .execute(
                    "INSERT INTO customers(id, name, phone, note, active, created_at, branch_id, kind_label, credit_limit_paise)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                     ON CONFLICT(id) DO UPDATE SET
                       name=excluded.name,
                       phone=excluded.phone,
                       note=excluded.note,
                       active=excluded.active,
                       kind_label=excluded.kind_label,
                       credit_limit_paise=excluded.credit_limit_paise",
                    params![
                        c.id,
                        c.name,
                        c.phone,
                        c.note,
                        c.active as i64,
                        Local::now().to_rfc3339(),
                        self.active_branch_id()?,
                        c.kind_label,
                        c.credit_limit_paise
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Upsert a staff row from Server sync (Client cache). Does not touch Owner of this device.
    pub fn upsert_staff_cache(&self, member: &StaffMember) -> Result<(), String> {
        if member.is_owner() {
            return Ok(());
        }
        self.conn
            .execute(
                "INSERT INTO staff(id, name, role, pin_hash, active, role_title, features_json, login_id, password_hash)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name,
                   role=excluded.role,
                   pin_hash=excluded.pin_hash,
                   active=excluded.active,
                   role_title=excluded.role_title,
                   features_json=excluded.features_json,
                   login_id=excluded.login_id,
                   password_hash=excluded.password_hash",
                params![
                    member.id,
                    member.name,
                    member.role.as_str(),
                    member.pin_hash,
                    member.active as i64,
                    member.role_title,
                    member.features.to_json(),
                    member.login_id,
                    member.password_hash
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Copy Server Owner PIN hash onto the local Owner so unlock matches the shop.
    pub fn list_all_credit_ledger(&self, limit: i64) -> Result<Vec<CreditLedgerEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_name, detail,
                        COALESCE(NULLIF(user_at, ''), created_at),
                        COALESCE(NULLIF(device_at, ''), created_at),
                        COALESCE(server_at, '')
                 FROM credit_ledger ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(CreditLedgerEntry {
                    id: row.get(0)?,
                    customer_id: row.get(1)?,
                    created_at: row.get(2)?,
                    kind: row.get(3)?,
                    amount_paise: row.get(4)?,
                    sale_id: row.get(5)?,
                    pay_method: row.get(6)?,
                    actor_name: row.get(7)?,
                    detail: row.get(8)?,
                    user_at: row.get(9)?,
                    device_at: row.get(10)?,
                    server_at: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn replace_credit_ledger_from_sync(
        &self,
        entries: &[CreditLedgerEntry],
    ) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM credit_ledger", [])
            .map_err(|e| e.to_string())?;
        for e in entries {
            let times = e.event_times();
            self.conn
                .execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail,
                        user_at, device_at, server_at
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, '', ?8, ?9, ?10, ?11, ?12)",
                    params![
                        e.id,
                        e.customer_id,
                        times.user_at,
                        e.kind,
                        e.amount_paise,
                        e.sale_id,
                        e.pay_method,
                        e.actor_name,
                        e.detail,
                        times.user_at,
                        times.device_at,
                        times.server_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn replace_tables_from_sync(&self, tables: &[DiningTable]) -> Result<(), String> {
        if tables.is_empty() {
            return Ok(());
        }
        self.conn
            .execute("DELETE FROM tables_dining", [])
            .map_err(|e| e.to_string())?;
        for t in tables {
            self.conn
                .execute(
                    "INSERT INTO tables_dining(id, name, status, active_sale_id, branch_id, occupied_at, area, paid_at)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        t.id,
                        t.name,
                        t.status.as_str(),
                        t.active_sale_id,
                        self.active_branch_id()?,
                        t.occupied_at,
                        t.area,
                        t.paid_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Cache recent sales (without lines) so Client Sales/today views work after sync.
    pub fn replace_sales_cache_from_sync(&self, sales: &[Sale]) -> Result<(), String> {
        // Keep local structure simple: wipe cached completed sales then insert headers.
        // Do not delete sale_lines for sales we don't have — wipe sales that have no lines first.
        self.conn
            .execute(
                "DELETE FROM sale_lines WHERE sale_id IN (SELECT id FROM sales)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM sales", [])
            .map_err(|e| e.to_string())?;
        for s in sales {
            self.conn
                .execute(
                    "INSERT INTO sales(
                        id, created_at, total_paise, lines_total_paise, rounding_paise,
                        pay_method, status, table_id, customer_id, created_by_id, created_by_name,
                        branch_id, order_type, tax_paise, tax_pricing, tax_label, net_paise,
                        service_paise, invoice_no, money_lane
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
                    params![
                        s.id,
                        s.created_at,
                        s.total_paise,
                        s.lines_total_paise,
                        s.rounding_paise,
                        s.pay_method,
                        s.status,
                        s.table_id,
                        s.customer_id,
                        s.created_by_id,
                        s.created_by_name,
                        self.active_branch_id()?,
                        s.order_type().as_str(),
                        s.tax_paise,
                        s.tax_pricing().as_str(),
                        s.tax_label(),
                        s.net().paise(),
                        s.service_paise,
                        s.invoice_no,
                        if s.money_lane.trim().is_empty() { "declared".into() } else { s.money_lane.clone() }
                    ],
                )
                .map_err(|e| e.to_string())?;
            for line in &s.lines {
                self.conn
                    .execute(
                        "INSERT INTO sale_lines(sale_id, product_id, name, unit_price_paise, qty, line_total_paise, note, qty_milli, unit_label, modifiers_json)
                         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        params![
                            s.id,
                            line.product_id,
                            line.name,
                            line.unit_price_paise,
                            line.qty,
                            line.line_total_paise,
                            line.note,
                            if line.qty_milli > 0 { line.qty_milli } else { line.qty.saturating_mul(1000) },
                            line.unit_label,
                            if line.modifiers_json.trim().is_empty() {
                                "[]".into()
                            } else {
                                line.modifiers_json.clone()
                            },
                        ],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn cache_server_day_totals(
        &self,
        totals: &DayTotals,
        outstanding: i64,
    ) -> Result<(), String> {
        let json = serde_json::to_string(totals).map_err(|e| e.to_string())?;
        self.meta_set("cached_today_totals", &json)?;
        self.meta_set("cached_outstanding_credit", &outstanding.to_string())?;
        Ok(())
    }

    pub fn set_product_available(&self, product_id: &str, available: bool) -> Result<(), String> {
        self.assert_server_writer("Mark items sold out")?;
        let n = self
            .conn
            .execute(
                "UPDATE products SET available = ?1 WHERE id = ?2",
                params![available as i64, product_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Product not found".into());
        }
        Ok(())
    }

    /// Legacy meta key cleanup (unsent lines are pending `running_lines` now).
    /// Does not mark Occupied — seating without an order uses [`Self::seat_table`].
    pub fn save_table_draft(&self, table_id: &str, draft_json: &str) -> Result<(), String> {
        self.meta_set(&format!("table_draft_{table_id}"), draft_json)
    }

    pub fn load_table_draft(&self, table_id: &str) -> Result<Option<String>, String> {
        self.meta_get(&format!("table_draft_{table_id}"))
    }

    /// Drop legacy draft meta and clear a leftover `draft:` occupancy marker.
    pub fn clear_table_draft_meta(&self, table_id: &str) -> Result<(), String> {
        self.meta_set(&format!("table_draft_{table_id}"), "")?;
        let active: String = self
            .conn
            .query_row(
                "SELECT COALESCE(active_sale_id, '') FROM tables_dining WHERE id = ?1",
                params![table_id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        if active == format!("draft:{table_id}") {
            self.conn
                .execute(
                    "UPDATE tables_dining SET active_sale_id = NULL WHERE id = ?1",
                    params![table_id],
                )
                .map_err(|e| e.to_string())?;
            // Free only when nothing else holds the table.
            if self.running_order(table_id)?.is_none() {
                let status: String = self
                    .conn
                    .query_row(
                        "SELECT status FROM tables_dining WHERE id = ?1",
                        params![table_id],
                        |r| r.get(0),
                    )
                    .unwrap_or_default();
                if status == TableStatus::Occupied.as_str() {
                    let _ = self.free_table(table_id);
                }
            }
        }
        Ok(())
    }

    pub fn clear_table_draft(&self, table_id: &str) -> Result<(), String> {
        self.assert_server_writer("Clear a table")?;
        self.clear_table_draft_meta(table_id)?;
        if self.running_order(table_id)?.is_none() {
            self.free_table(table_id)?;
        }
        Ok(())
    }

    /// Whether this device prints a KOT the moment one is sent
    /// (Settings → kitchen printing; on by default — paper is how
    /// most kitchens still run).
    pub fn kot_auto_print(&self) -> bool {
        self.meta_get("kot_auto_print")
            .ok()
            .flatten()
            .map(|v| v != "0")
            .unwrap_or(true)
    }

    pub fn set_kot_auto_print(&self, on: bool) -> Result<(), String> {
        self.meta_set("kot_auto_print", if on { "1" } else { "0" })
    }

    /// Printable KOT text: table, time, station, items — never prices.
    pub fn format_kot_text(&self, kot_id: &str) -> Result<String, String> {
        let (created_at, summary, station_name, kot_no, kind, created_by): (
            String,
            String,
            String,
            i64,
            String,
            String,
        ) = self
            .conn
            .query_row(
                "SELECT k.created_at, k.summary, COALESCE(s.name, ''),
                        COALESCE(k.kot_no, 0), COALESCE(k.kind, 'kot'),
                        COALESCE(k.created_by, '')
                   FROM kot_tickets k
                   LEFT JOIN kitchen_stations s ON s.id = k.station_id
                  WHERE k.id = ?1",
                params![kot_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .map_err(|e| e.to_string())?;
        let when = created_at.get(..16).unwrap_or(&created_at).replace('T', " ");
        let mut out = String::from(if kind == "cancel" {
            "*** CANCEL — STOP THIS ITEM ***\n"
        } else {
            "KOT — KITCHEN ORDER\n"
        });
        if kot_no > 0 {
            out.push_str(&format!("KOT No: {kot_no}\n"));
        }
        out.push_str(&format!("Time: {when}\n"));
        if !created_by.is_empty() {
            out.push_str(&format!("By: {created_by}\n"));
        }
        if !station_name.is_empty() {
            out.push_str(&format!("Station: {station_name}\n"));
        }
        out.push_str("--------------------------------\n");
        for item in summary.split(", ") {
            out.push_str(&format!("{item}\n"));
        }
        out.push_str("--------------------------------\n");
        Ok(out)
    }

    /// Print a KOT on this device (writes the print file and sends it along
    /// this station's printer route — fallback order from Settings). Called
    /// by the server the moment a KOT is sent — from the counter or from a
    /// waiter's paired device — when auto-print is on.
    pub fn print_kot(&self, kot_id: &str) -> Result<std::path::PathBuf, String> {
        let text = self.format_kot_text(kot_id)?;
        let station_id: String = self
            .conn
            .query_row(
                "SELECT COALESCE(station_id, '') FROM kot_tickets WHERE id = ?1",
                rusqlite::params![kot_id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        let share = crate::print_share::print_route(
            crate::print_share::PrintKind::Kot,
            crate::print_share::RouteKey::Kot {
                station_id: &station_id,
            },
            &text,
            self.path.parent(),
        )?;
        Ok(share.path)
    }

    /// Charge a bill with any mix of Cash / UPI(rail) / Card / Credit.
    /// Parts must sum exactly to the charged (taxed) total. For a ₹0 bill
    /// (when [`Self::allow_zero_sales`]), a single zero tender is fine;
    /// otherwise each part must be positive. Credit portion needs
    /// `customer_id` (rest can be paid now; remainder on khata).
    pub fn finalize_sale_tenders(
        &mut self,
        draft: &SaleDraft,
        parts: &[(PayMethod, i64)],
        table_id: Option<&str>,
        customer_id: Option<&str>,
        actor: &StaffMember,
    ) -> Result<Sale, String> {
        if parts.is_empty() {
            return Err("Enter at least one payment amount".into());
        }
        if parts.iter().any(|(m, _)| m.is_split()) {
            return Err("Do not nest mixed pay".into());
        }
        let mut seen = [false; 4];
        for (m, _) in parts {
            let i = match m {
                PayMethod::Cash => 0,
                PayMethod::Upi => 1,
                PayMethod::Card => 2,
                PayMethod::Credit => 3,
                _ => return Err("Use Cash, UPI, Card, and/or Credit".into()),
            };
            if seen[i] {
                return Err("Each tender can appear only once".into());
            }
            seen[i] = true;
        }
        let credit_part = parts
            .iter()
            .find(|(m, _)| m.is_credit())
            .map(|(_, a)| *a)
            .unwrap_or(0);
        if credit_part > 0 && customer_id.is_none() {
            return Err("Pick a customer for the Credit part".into());
        }
        let charged = self.preview_sale_total(draft)?;
        let sum: i64 = parts.iter().map(|(_, a)| *a).sum();
        if sum != charged {
            return Err(format!(
                "Payment parts ({sum}) must equal bill ({charged})"
            ));
        }
        if charged == 0 {
            if !self.allow_zero_sales()? {
                return Err("Zero-value sales are turned off in Settings".into());
            }
            if parts.len() != 1 || parts[0].1 != 0 {
                return Err("A ₹0 bill needs one payment way at ₹0".into());
            }
        } else if parts.iter().any(|(_, a)| *a <= 0) {
            return Err("Each payment amount must be > 0".into());
        }
        let pay = if parts.len() == 1 {
            parts[0].0
        } else {
            // Legacy row label; receipt lists each tender from sale_payments.
            PayMethod::Split
        };
        self.write_sale(draft, pay, table_id, customer_id, actor, parts)
    }

    /// Old name — prefer `finalize_sale_tenders`.
    pub fn finalize_sale_split(
        &mut self,
        draft: &SaleDraft,
        parts: &[(PayMethod, i64)],
        table_id: Option<&str>,
        actor: &StaffMember,
    ) -> Result<Sale, String> {
        self.finalize_sale_tenders(draft, parts, table_id, None, actor)
    }

    pub fn add_expense(
        &self,
        amount_paise: i64,
        category: &str,
        note: &str,
        actor: &StaffMember,
    ) -> Result<String, String> {
        // Single-tender convenience (all cash — the counter default).
        self.add_expense_tenders(
            amount_paise,
            category,
            note,
            actor,
            &[(PayMethod::Cash, amount_paise)],
        )
    }

    /// Log an expense the way it was actually paid (ground truth,
    /// vision §18): one expense, one or more tender parts summing to
    /// the total — e.g. part bank, part cash. A supporting bill may
    /// cover only part of it; no document is required to save.
    pub fn add_expense_tenders(
        &self,
        amount_paise: i64,
        category: &str,
        note: &str,
        actor: &StaffMember,
        parts: &[(PayMethod, i64)],
    ) -> Result<String, String> {
        self.assert_server_writer("Log expenses")?;
        if amount_paise <= 0 {
            return Err("Expense must be > 0".into());
        }
        if !actor.is_owner() && !actor.role.can_void_refund() {
            return Err("Not allowed to log expenses".into());
        }
        if parts.is_empty() {
            return Err("Say how the expense was paid".into());
        }
        let sum: i64 = parts.iter().map(|(_, a)| *a).sum();
        if sum != amount_paise {
            return Err(format!(
                "Payment parts ({sum}) must equal the expense ({amount_paise})"
            ));
        }
        if parts.iter().any(|(m, a)| *a <= 0 || m.is_credit() || m.is_split()) {
            return Err("Each part must be > 0, paid by cash, bank, UPI/rail, or card".into());
        }
        let cat = {
            let c = category.trim();
            if c.is_empty() {
                "misc"
            } else {
                c
            }
        };
        let times = EventTimes::capture(None);
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO expenses(
                    id, created_at, amount_paise, category, note, actor_id, actor_name, branch_id,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id,
                    times.user_at,
                    amount_paise,
                    cat,
                    note.trim(),
                    actor.id,
                    actor.name,
                    self.active_branch_id()?,
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        for (method, amt) in parts {
            self.conn
                .execute(
                    "INSERT INTO expense_payments(id, expense_id, pay_method, amount_paise)
                     VALUES(?1, ?2, ?3, ?4)",
                    params![new_id(), id, method.as_str(), amt],
                )
                .map_err(|e| e.to_string())?;
        }
        self.audit(actor, "expense.add", &format!("{cat} {amount_paise}"))?;
        Ok(id)
    }

    /// How an expense was paid, part by part. Empty for expenses logged
    /// before tenders were recorded.
    pub fn expense_payments_for(
        &self,
        expense_id: &str,
    ) -> Result<Vec<(PayMethod, i64)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT pay_method, amount_paise FROM expense_payments
                 WHERE expense_id = ?1 ORDER BY rowid",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![expense_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let (m, a) = row.map_err(|e| e.to_string())?;
            if let Some(m) = PayMethod::parse(&m) {
                out.push((m, a));
            }
        }
        Ok(out)
    }

    pub fn today_expenses_total(&self) -> Result<(i64, i64), String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let prefix = format!("{today}%");
        let branch = self.active_branch_id()?;
        self.conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(amount_paise), 0) FROM expenses
                 WHERE created_at LIKE ?1 AND (branch_id = ?2 OR branch_id = '')",
                params![prefix, branch],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())
    }

    pub fn ops_report_today(&self) -> Result<OpsReport, String> {
        let mut day = self.today_totals()?;
        let today = Local::now().format("%Y-%m-%d").to_string();
        let prefix = format!("{today}%");
        let branch = self.active_branch_id()?;
        let mut cash = 0i64;
        let mut upi = 0i64;
        let mut card = 0i64;
        let mut credit = 0i64;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.pay_method, p.amount_paise
                 FROM sale_payments p
                 JOIN sales s ON s.id = p.sale_id
                 WHERE s.status = 'completed' AND s.created_at LIKE ?1
                   AND (s.branch_id = ?2 OR s.branch_id = '')",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![prefix, branch], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut any = false;
        for row in rows {
            let (m, a) = row.map_err(|e| e.to_string())?;
            any = true;
            match PayMethod::parse(&m) {
                Some(PayMethod::Cash) => cash += a,
                Some(PayMethod::Upi) => upi += a,
                Some(PayMethod::Card) => card += a,
                Some(PayMethod::Credit) => credit += a,
                _ => {}
            }
        }
        if any {
            day.cash_paise = cash;
            day.upi_paise = upi;
            day.card_paise = card;
            day.credit_paise = credit;
        }
        let (expense_count, expenses_paise) = self.today_expenses_total()?;
        let mut top_stmt = self
            .conn
            .prepare(
                "SELECT l.name, SUM(l.qty), SUM(l.line_total_paise)
                 FROM sale_lines l
                 JOIN sales s ON s.id = l.sale_id
                 WHERE s.status = 'completed' AND s.created_at LIKE ?1
                   AND (s.branch_id = ?2 OR s.branch_id = '')
                 GROUP BY l.name
                 ORDER BY SUM(l.qty) DESC
                 LIMIT 8",
            )
            .map_err(|e| e.to_string())?;
        let top_items = top_stmt
            .query_map(params![prefix, branch], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let bank_net_paise = self.money_movements_net_for_prefix(&prefix)?;
        Ok(OpsReport {
            day,
            outstanding_credit_paise: self.outstanding_credit_total()?,
            expenses_paise,
            expense_count,
            top_items,
            bank_net_paise,
        })
    }

    /// Post a cashbook / bank movement (generic till↔bank or bank in/out).
    pub fn post_money_movement(
        &self,
        kind: MoneyMoveKind,
        amount_paise: i64,
        note: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Post bank / cashbook movement")?;
        if amount_paise <= 0 {
            return Err("Amount must be > 0".into());
        }
        if !actor.is_owner() && !actor.role.can_void_refund() {
            return Err("Not allowed to post bank movements".into());
        }
        let times = EventTimes::capture(None);
        self.conn
            .execute(
                "INSERT INTO money_movements(
                    id, created_at, kind, amount_paise, note, actor_id, actor_name, branch_id,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    new_id(),
                    times.user_at,
                    kind.as_str(),
                    amount_paise,
                    note.trim(),
                    actor.id,
                    actor.name,
                    self.active_branch_id()?,
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "money.move",
            &format!("{} {amount_paise} {}", kind.as_str(), note.trim()),
        )?;
        Ok(())
    }

    pub fn list_money_movements(&self, limit: i64) -> Result<Vec<MoneyMovement>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, created_at, kind, amount_paise, note, actor_name,
                        COALESCE(NULLIF(user_at, ''), created_at),
                        COALESCE(NULLIF(device_at, ''), created_at),
                        COALESCE(server_at, ''), COALESCE(cleared_at, '')
                 FROM money_movements
                 WHERE branch_id = ?1 OR branch_id = ''
                 ORDER BY created_at DESC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch, limit], |row| {
                Ok(MoneyMovement {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    kind: row.get(2)?,
                    amount_paise: row.get(3)?,
                    note: row.get(4)?,
                    actor_name: row.get(5)?,
                    user_at: row.get(6)?,
                    device_at: row.get(7)?,
                    server_at: row.get(8)?,
                    cleared: !row.get::<_, String>(9)?.is_empty(),
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Bank book balance: in + till→bank − out − bank→till.
    pub fn bank_balance_paise(&self) -> Result<i64, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT kind, amount_paise FROM money_movements
                 WHERE branch_id = ?1 OR branch_id = ''",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut bal = 0i64;
        for row in rows {
            let (kind, amount) = row.map_err(|e| e.to_string())?;
            match MoneyMoveKind::parse(&kind) {
                Some(MoneyMoveKind::TillToBank) | Some(MoneyMoveKind::BankIn) => bal += amount,
                Some(MoneyMoveKind::BankToTill) | Some(MoneyMoveKind::BankOut) => bal -= amount,
                None => {}
            }
        }
        Ok(bal)
    }

    /// Tick (or untick) one bank movement against the bank statement
    /// (bank reconciliation). Cleared movements are what the bank has
    /// actually seen; the rest explain book-vs-bank difference.
    pub fn set_money_movement_cleared(
        &self,
        movement_id: &str,
        cleared: bool,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Reconcile bank movements")?;
        let stamp = if cleared {
            Local::now().to_rfc3339()
        } else {
            String::new()
        };
        let n = self
            .conn
            .execute(
                "UPDATE money_movements SET cleared_at = ?1 WHERE id = ?2",
                params![stamp, movement_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Bank movement not found".into());
        }
        self.audit(
            actor,
            if cleared { "bank.clear" } else { "bank.unclear" },
            movement_id,
        )?;
        Ok(())
    }

    /// Balance counting only statement-cleared movements — what the
    /// bank balance should match.
    pub fn bank_cleared_balance_paise(&self) -> Result<i64, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT kind, amount_paise FROM money_movements
                 WHERE (branch_id = ?1 OR branch_id = '')
                   AND COALESCE(cleared_at, '') <> ''",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut bal = 0i64;
        for row in rows {
            let (kind, amount) = row.map_err(|e| e.to_string())?;
            match MoneyMoveKind::parse(&kind) {
                Some(MoneyMoveKind::TillToBank) | Some(MoneyMoveKind::BankIn) => bal += amount,
                Some(MoneyMoveKind::BankToTill) | Some(MoneyMoveKind::BankOut) => bal -= amount,
                None => {}
            }
        }
        Ok(bal)
    }

    /// (book balance, cleared balance, uncleared count, uncleared net)
    /// — the whole reconciliation story in one line for Reports.
    pub fn bank_reconciliation(&self) -> Result<(i64, i64, i64, i64), String> {
        let book = self.bank_balance_paise()?;
        let cleared = self.bank_cleared_balance_paise()?;
        let branch = self.active_branch_id()?;
        let (count, net): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(CASE kind
                    WHEN 'till_to_bank' THEN amount_paise
                    WHEN 'bank_in' THEN amount_paise
                    WHEN 'bank_to_till' THEN -amount_paise
                    WHEN 'bank_out' THEN -amount_paise ELSE 0 END), 0)
                 FROM money_movements
                 WHERE (branch_id = ?1 OR branch_id = '')
                   AND COALESCE(cleared_at, '') = ''",
                params![branch],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        Ok((book, cleared, count, net))
    }

    // -----------------------------------------------------------------
    // CSV export / import (spreadsheet round-trip; bring-your-book
    // without retyping). Amounts are major units with 2 decimals.
    // -----------------------------------------------------------------

    /// Parties with their receivable / payable position.
    pub fn parties_csv(&self) -> Result<String, String> {
        let mut out = String::from("name,phone,kind,credit_limit,they_owe_us,we_owe_them,net\n");
        for p in self.list_customers()? {
            let ar = p.balance_paise.max(0);
            let ap = self.party_payable_balance(&p.id)?.max(0);
            out.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                csv_cell(&p.name),
                csv_cell(&p.phone),
                csv_cell(&p.kind_label),
                csv_money(p.credit_limit_paise),
                csv_money(ar),
                csv_money(ap),
                csv_money(ar - ap),
            ));
        }
        Ok(out)
    }

    /// Who owes whom, one row per party with anything open.
    pub fn outstanding_csv(&self) -> Result<String, String> {
        let mut out = String::from("party,kind,they_owe_us,we_owe_them,net,oldest_days\n");
        let days: std::collections::HashMap<String, i64> = self
            .party_aging_report()?
            .into_iter()
            .map(|r| (r.customer_id, r.days))
            .collect();
        for p in self.list_customers()? {
            let ar = p.balance_paise.max(0);
            let ap = self.party_payable_balance(&p.id)?.max(0);
            if ar == 0 && ap == 0 && p.balance_paise >= 0 {
                continue;
            }
            out.push_str(&format!(
                "{},{},{},{},{},{}\n",
                csv_cell(&p.name),
                csv_cell(&p.kind_label),
                csv_money(ar),
                csv_money(ap),
                csv_money(ar - ap),
                days.get(&p.id).copied().unwrap_or(0),
            ));
        }
        Ok(out)
    }

    /// Catalogue / rate items.
    pub fn items_csv(&self) -> Result<String, String> {
        let mut out = String::from("name,price,unit,available\n");
        for p in self.list_products(false)? {
            out.push_str(&format!(
                "{},{},{},{}\n",
                csv_cell(&p.name),
                csv_money(p.price.0),
                csv_cell(&p.unit_label),
                if p.available { "yes" } else { "no" },
            ));
        }
        Ok(out)
    }

    /// Today's books as metric,amount rows (day book in a spreadsheet).
    pub fn day_book_csv(&self) -> Result<String, String> {
        let t = self.today_totals()?;
        let (ec, exp) = self.today_expenses_total()?;
        let today = Local::now().format("%Y-%m-%d").to_string();
        let m = |v: i64| csv_money(v);
        Ok(format!(
            "metric,amount\nsales_count,{}\nrevenue,{}\ncash,{}\ndigital,{}\ncard,{}\ncredit_sales,{}\nexpenses_count,{}\nexpenses,{}\ncash_out_today,{}\nbank_balance,{}\ntill_cash_in_hand,{}\noutstanding_receivable,{}\n",
            t.sale_count,
            m(t.revenue_paise),
            m(t.cash_paise),
            m(t.upi_paise),
            m(t.card_paise),
            m(t.credit_paise),
            ec,
            m(exp),
            m(self.cash_out_paise(Some(&today))?),
            m(self.bank_balance_paise()?),
            m(self.till_balance_paise()?),
            m(self.outstanding_credit_total()?),
        ))
    }

    /// One party's receivable ledger, oldest first, running balance.
    pub fn party_ledger_csv(&self, customer_id: &str) -> Result<String, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT created_at, kind, detail, COALESCE(pay_method, ''), amount_paise
                 FROM credit_ledger WHERE customer_id = ?1 ORDER BY created_at, rowid",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![customer_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = String::from("date,kind,detail,paid_by,amount,balance\n");
        let mut bal = 0i64;
        for row in rows {
            let (at, kind, detail, pay, amount) = row.map_err(|e| e.to_string())?;
            bal += amount;
            out.push_str(&format!(
                "{},{},{},{},{},{}\n",
                csv_cell(&at[..at.len().min(10)]),
                csv_cell(&kind),
                csv_cell(&detail),
                csv_cell(&pay),
                csv_money(amount),
                csv_money(bal),
            ));
        }
        Ok(out)
    }

    /// Bring parties in from a spreadsheet export. Header row (with a
    /// `name` column) is honoured in any column order; without a header
    /// the order is name,phone,kind,note. Existing parties are not
    /// duplicated (same name + phone is skipped). Returns
    /// (added, skipped, first few problems).
    pub fn import_parties_csv(
        &self,
        text: &str,
    ) -> Result<(usize, usize, Vec<String>), String> {
        let rows = parse_csv(text);
        let mut added = 0usize;
        let mut skipped = 0usize;
        let mut problems: Vec<String> = Vec::new();
        let col = |names: &[&str], header: &Vec<String>| -> Option<usize> {
            header.iter().position(|h| {
                let h = h.trim().to_ascii_lowercase();
                names.iter().any(|n| *n == h)
            })
        };
        let mut existing: Vec<(String, String)> = self
            .list_customers()?
            .into_iter()
            .map(|p| (p.name.to_ascii_lowercase(), p.phone))
            .collect();
        let mut start = 0usize;
        let mut idx = (0usize, 1usize, 2usize, 3usize);
        if let Some(first) = rows.first() {
            let lower: Vec<String> = first.iter().map(|c| c.to_ascii_lowercase()).collect();
            if lower.iter().any(|c| c.trim() == "name") {
                idx = (
                    col(&["name", "party", "customer"], &lower).unwrap_or(0),
                    col(&["phone", "mobile", "contact"], &lower).unwrap_or(usize::MAX),
                    col(&["kind", "role", "kind_label", "type"], &lower)
                        .unwrap_or(usize::MAX),
                    col(&["note", "notes", "remark"], &lower).unwrap_or(usize::MAX),
                );
                start = 1;
            }
        }
        let get = |row: &Vec<String>, i: usize| -> String {
            if i == usize::MAX {
                String::new()
            } else {
                row.get(i).cloned().unwrap_or_default().trim().to_string()
            }
        };
        for (n, row) in rows.iter().skip(start).enumerate() {
            let name = get(row, idx.0);
            if name.is_empty() {
                continue;
            }
            let phone = get(row, idx.1);
            if existing
                .iter()
                .any(|(en, ep)| *en == name.to_ascii_lowercase() && *ep == phone)
            {
                skipped += 1;
                continue;
            }
            match self.add_customer_kind(&name, &phone, &get(row, idx.2)) {
                Ok(_) => {
                    added += 1;
                    existing.push((name.to_ascii_lowercase(), phone));
                }
                Err(e) => {
                    skipped += 1;
                    if problems.len() < 5 {
                        problems.push(format!("row {}: {e}", n + start + 1));
                    }
                }
            }
        }
        Ok((added, skipped, problems))
    }

    /// Bring catalogue / rate items in from a spreadsheet export.
    /// Columns: name, price (major units, e.g. 250.50), unit. Items
    /// whose name already exists are skipped, never overwritten.
    pub fn import_items_csv(&self, text: &str) -> Result<(usize, usize, Vec<String>), String> {
        let rows = parse_csv(text);
        let mut added = 0usize;
        let mut skipped = 0usize;
        let mut problems: Vec<String> = Vec::new();
        let mut existing: Vec<String> = self
            .list_products(true)?
            .into_iter()
            .map(|p| p.name.to_ascii_lowercase())
            .collect();
        let mut start = 0usize;
        let (mut i_name, mut i_price, mut i_unit) = (0usize, 1usize, 2usize);
        if let Some(first) = rows.first() {
            let lower: Vec<String> = first.iter().map(|c| c.to_ascii_lowercase()).collect();
            if lower.iter().any(|c| c.trim() == "name") {
                let find = |names: &[&str]| {
                    lower.iter().position(|h| names.iter().any(|n| *n == h.trim()))
                };
                i_name = find(&["name", "item", "product"]).unwrap_or(0);
                i_price = find(&["price", "rate", "amount"]).unwrap_or(usize::MAX);
                i_unit = find(&["unit", "unit_label"]).unwrap_or(usize::MAX);
                start = 1;
            }
        }
        let get = |row: &Vec<String>, i: usize| -> String {
            if i == usize::MAX {
                String::new()
            } else {
                row.get(i).cloned().unwrap_or_default().trim().to_string()
            }
        };
        for (n, row) in rows.iter().skip(start).enumerate() {
            let name = get(row, i_name);
            if name.is_empty() {
                continue;
            }
            if existing.iter().any(|e| *e == name.to_ascii_lowercase()) {
                skipped += 1;
                continue;
            }
            let price_major: f64 = get(row, i_price).parse().unwrap_or(0.0);
            let mut product =
                Product::new_basic(new_id(), &name, Money((price_major * 100.0).round() as i64));
            let unit = get(row, i_unit);
            if !unit.is_empty() {
                product.unit_label = unit;
            }
            match self.upsert_product(&product) {
                Ok(()) => {
                    added += 1;
                    existing.push(name.to_ascii_lowercase());
                }
                Err(e) => {
                    skipped += 1;
                    if problems.len() < 5 {
                        problems.push(format!("row {}: {e}", n + start + 1));
                    }
                }
            }
        }
        Ok((added, skipped, problems))
    }

    fn money_movements_net_for_prefix(&self, day_prefix: &str) -> Result<i64, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT kind, amount_paise FROM money_movements
                 WHERE created_at LIKE ?1 AND (branch_id = ?2 OR branch_id = '')",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![day_prefix, branch], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut net = 0i64;
        for row in rows {
            let (kind, amount) = row.map_err(|e| e.to_string())?;
            match MoneyMoveKind::parse(&kind) {
                Some(MoneyMoveKind::TillToBank) | Some(MoneyMoveKind::BankIn) => net += amount,
                Some(MoneyMoveKind::BankToTill) | Some(MoneyMoveKind::BankOut) => net -= amount,
                None => {}
            }
        }
        Ok(net)
    }

    /// Party-wise outstanding with aging buckets (0–7 / 8–30 / 31+ days).
    pub fn party_aging_report(&self) -> Result<Vec<PartyAgingRow>, String> {
        let parties = self.list_customers()?;
        let now = Local::now().date_naive();
        let mut rows = Vec::new();
        for party in parties {
            if party.balance_paise == 0 {
                continue;
            }
            if party.balance_paise < 0 {
                rows.push(PartyAgingRow {
                    customer_id: party.id,
                    name: party.name,
                    kind_label: party.kind_label,
                    balance_paise: party.balance_paise,
                    days: 0,
                    bucket: "advance".into(),
                });
                continue;
            }
            let oldest: Option<String> = self
                .conn
                .query_row(
                    "SELECT MIN(created_at) FROM credit_ledger
                     WHERE customer_id = ?1 AND amount_paise > 0",
                    params![party.id],
                    |r| r.get::<_, Option<String>>(0),
                )
                .map_err(|e| e.to_string())?;
            let days = oldest
                .as_deref()
                .and_then(|ts| NaiveDate::parse_from_str(&ts[..ts.len().min(10)], "%Y-%m-%d").ok())
                .map(|d| (now - d).num_days().max(0))
                .unwrap_or(0);
            let bucket = if days <= 7 {
                "current"
            } else if days <= 30 {
                "due"
            } else {
                "overdue"
            };
            rows.push(PartyAgingRow {
                customer_id: party.id,
                name: party.name,
                kind_label: party.kind_label,
                balance_paise: party.balance_paise,
                days,
                bucket: bucket.into(),
            });
        }
        rows.sort_by(|a, b| {
            b.days
                .cmp(&a.days)
                .then_with(|| b.balance_paise.cmp(&a.balance_paise))
        });
        Ok(rows)
    }

    pub fn party_aging_text(&self) -> Result<String, String> {
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let rows = self.party_aging_report()?;
        let mut out = String::from("Party outstanding (aging)\n");
        if rows.is_empty() {
            out.push_str("No balances.\n");
            return Ok(out);
        }
        let mut current = 0i64;
        let mut due = 0i64;
        let mut overdue = 0i64;
        let mut advance = 0i64;
        for row in &rows {
            match row.bucket.as_str() {
                "current" => current += row.balance_paise,
                "due" => due += row.balance_paise,
                "overdue" => overdue += row.balance_paise,
                "advance" => advance += row.balance_paise.abs(),
                _ => {}
            }
            let kind = if row.kind_label.trim().is_empty() {
                String::new()
            } else {
                format!(" · {}", row.kind_label.trim())
            };
            let age = if row.bucket == "advance" {
                "advance".into()
            } else {
                format!("{}d · {}", row.days, row.bucket)
            };
            out.push_str(&format!(
                "{}{} · {} · {}\n",
                row.name,
                kind,
                Money(row.balance_paise).format_localized(&currency, &fiscal),
                age
            ));
        }
        out.push_str(&format!(
            "—\n0–7d {} · 8–30d {} · 31+d {}\nAdvances held {}\n",
            Money(current).format_localized(&currency, &fiscal),
            Money(due).format_localized(&currency, &fiscal),
            Money(overdue).format_localized(&currency, &fiscal),
            Money(advance).format_localized(&currency, &fiscal),
        ));
        Ok(out)
    }

    /// Per-party AR + AP + net (one identity; dual-role peers included once).
    ///
    /// `net_paise = ar − ap`: positive → they owe us overall; negative → we owe them.
    pub fn party_position_report(&self) -> Result<Vec<PartyPositionRow>, String> {
        let parties = self.list_customers()?;
        let mut rows = Vec::new();
        for party in parties {
            let ar = party.balance_paise;
            let ap = self.party_payable_balance(&party.id)?;
            if ar == 0 && ap == 0 {
                continue;
            }
            rows.push(PartyPositionRow {
                customer_id: party.id,
                name: party.name,
                kind_label: party.kind_label,
                ar_paise: ar,
                ap_paise: ap,
                net_paise: ar.saturating_sub(ap),
            });
        }
        rows.sort_by(|a, b| {
            b.net_paise
                .abs()
                .cmp(&a.net_paise.abs())
                .then_with(|| a.name.cmp(&b.name))
        });
        Ok(rows)
    }

    pub fn party_position_text(&self) -> Result<String, String> {
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let rows = self.party_position_report()?;
        let mut out = String::from(
            "Party net position (one firm · AR and AP separate · net = who owes whom)\n",
        );
        if rows.is_empty() {
            out.push_str("No AR/AP balances.\n");
            return Ok(out);
        }
        for row in &rows {
            let kind = if row.kind_label.trim().is_empty() {
                String::new()
            } else {
                format!(" · {}", row.kind_label.trim())
            };
            let who = if row.net_paise > 0 {
                "they owe us"
            } else if row.net_paise < 0 {
                "we owe them"
            } else {
                "squared"
            };
            out.push_str(&format!(
                "{}{} · AR {} · AP {} · net {} ({})\n",
                row.name,
                kind,
                Money(row.ar_paise).format_localized(&currency, &fiscal),
                Money(row.ap_paise).format_localized(&currency, &fiscal),
                Money(row.net_paise.abs()).format_localized(&currency, &fiscal),
                who
            ));
        }
        Ok(out)
    }

    /// Items sold (sum of line quantities) for completed sales in an
    /// inclusive day range — the restaurant period-books volume metric.
    pub fn period_items_sold(&self, from_day: &str, to_day: &str) -> Result<i64, String> {
        let branch = self.active_branch_id()?;
        let from = format!("{from_day}T00:00:00");
        let to = format!("{to_day}T23:59:59.999999999");
        let total: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(l.qty), 0) FROM sale_lines l
                 JOIN sales s ON s.id = l.sale_id
                 WHERE s.status = 'completed'
                   AND s.created_at >= ?1 AND s.created_at <= ?2
                   AND (s.branch_id = ?3 OR s.branch_id = '')",
                params![from, to, branch],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(total)
    }

    /// Debit / credit style pack: AR (debit to parties) vs AP (credit to parties).
    pub fn debit_credit_report_text(&self) -> Result<String, String> {
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let rows = self.party_position_report()?;
        let mut out = String::from("Debit / credit (party books)\n");
        out.push_str("Debit = they owe us (AR) · Credit = we owe them (AP)\n");
        if rows.is_empty() {
            out.push_str("No open balances.\n");
            return Ok(out);
        }
        let mut debit_total = 0i64;
        let mut credit_total = 0i64;
        for row in &rows {
            let kind = if row.kind_label.trim().is_empty() {
                String::new()
            } else {
                format!(" · {}", row.kind_label.trim())
            };
            if row.ar_paise != 0 {
                out.push_str(&format!(
                    "Dr {}{} · {}\n",
                    row.name,
                    kind,
                    Money(row.ar_paise).format_localized(&currency, &fiscal)
                ));
                debit_total += row.ar_paise;
            }
            if row.ap_paise != 0 {
                out.push_str(&format!(
                    "Cr {}{} · {}\n",
                    row.name,
                    kind,
                    Money(row.ap_paise).format_localized(&currency, &fiscal)
                ));
                credit_total += row.ap_paise;
            }
        }
        out.push_str(&format!(
            "—\nTotal debit (AR) {} · Total credit (AP) {}\nNet {}\n",
            Money(debit_total).format_localized(&currency, &fiscal),
            Money(credit_total).format_localized(&currency, &fiscal),
            Money(debit_total.saturating_sub(credit_total)).format_localized(&currency, &fiscal),
        ));
        Ok(out)
    }

    /// Operating balance sheet snapshot (not statutory Schedule III).
    ///
    /// Assets: bank + cash in till + debtors (AR). Liabilities:
    /// creditors (AP). Balancing figure is the residual net position.
    pub fn balance_sheet_text(&self) -> Result<String, String> {
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let bank = self.bank_balance_paise()?;
        let till = self.till_balance_paise()?;
        let ar = self.outstanding_credit_total()?;
        let ap = self.outstanding_payable_total()?;
        let assets = bank.saturating_add(till).saturating_add(ar);
        let liabilities = ap;
        let net = assets.saturating_sub(liabilities);
        let venue = self.venue_title()?;
        let venue = if venue.trim().is_empty() {
            "Your shop"
        } else {
            venue.trim()
        };
        let today = Local::now().format("%Y-%m-%d").to_string();
        Ok(format!(
            "{venue} — balance sheet (operating)\n\
             As of {today}\n\
             Assets\n\
               Bank {}\n\
               Cash in hand (till) {}\n\
               Debtors (they owe us) {}\n\
               Total assets {}\n\
             Liabilities\n\
               Creditors (we owe them) {}\n\
               Total liabilities {}\n\
             Net position (assets − liabilities) {}\n\
             (Operating snapshot — statutory balance-sheet formats are a separate, later filing view.)\n",
            Money(bank).format_localized(&currency, &fiscal),
            Money(till).format_localized(&currency, &fiscal),
            Money(ar).format_localized(&currency, &fiscal),
            Money(assets).format_localized(&currency, &fiscal),
            Money(ap).format_localized(&currency, &fiscal),
            Money(liabilities).format_localized(&currency, &fiscal),
            Money(net).format_localized(&currency, &fiscal),
        ))
    }

    /// Books for an inclusive date range (`YYYY-MM-DD` … `YYYY-MM-DD`).
    pub fn period_books(&self, from_day: &str, to_day: &str) -> Result<PeriodBooks, String> {
        let branch = self.active_branch_id()?;
        let from = format!("{from_day}T00:00:00");
        let to = format!("{to_day}T23:59:59.999999999");
        let mut day = DayTotals::default();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT total_paise, COALESCE(tax_paise, 0), COALESCE(money_lane, 'declared')
                 FROM sales
                 WHERE status = 'completed'
                   AND created_at >= ?1 AND created_at <= ?2
                   AND (branch_id = ?3 OR branch_id = '')",
            )
            .map_err(|e| e.to_string())?;
        let sales = stmt
            .query_map(params![from, to, branch], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in sales {
            let (total, tax, lane) = row.map_err(|e| e.to_string())?;
            day.sale_count += 1;
            day.revenue_paise += total;
            day.tax_paise += tax;
            match MoneyLane::parse(&lane) {
                MoneyLane::Declared => day.declared_paise += total,
                MoneyLane::Owner => day.owner_lane_paise += total,
            }
        }
        let mut pay_stmt = self
            .conn
            .prepare(
                "SELECT p.pay_method, p.amount_paise
                 FROM sale_payments p
                 JOIN sales s ON s.id = p.sale_id
                 WHERE s.status = 'completed'
                   AND s.created_at >= ?1 AND s.created_at <= ?2
                   AND (s.branch_id = ?3 OR s.branch_id = '')",
            )
            .map_err(|e| e.to_string())?;
        let pays = pay_stmt
            .query_map(params![from, to, branch], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in pays {
            let (m, a) = row.map_err(|e| e.to_string())?;
            match PayMethod::parse(&m) {
                Some(PayMethod::Cash) => day.cash_paise += a,
                Some(PayMethod::Upi) => day.upi_paise += a,
                Some(PayMethod::Card) => day.card_paise += a,
                Some(PayMethod::Credit) => day.credit_paise += a,
                _ => {}
            }
        }
        let (expense_count, expenses_paise): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(amount_paise), 0) FROM expenses
                 WHERE created_at >= ?1 AND created_at <= ?2
                   AND (branch_id = ?3 OR branch_id = '')",
                params![from, to, branch],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        let mut bank_in = 0i64;
        let mut bank_out = 0i64;
        let mut mv = self
            .conn
            .prepare(
                "SELECT kind, amount_paise FROM money_movements
                 WHERE created_at >= ?1 AND created_at <= ?2
                   AND (branch_id = ?3 OR branch_id = '')",
            )
            .map_err(|e| e.to_string())?;
        let mvs = mv
            .query_map(params![from, to, branch], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in mvs {
            let (kind, amount) = row.map_err(|e| e.to_string())?;
            match MoneyMoveKind::parse(&kind) {
                Some(MoneyMoveKind::TillToBank) | Some(MoneyMoveKind::BankIn) => bank_in += amount,
                Some(MoneyMoveKind::BankToTill) | Some(MoneyMoveKind::BankOut) => bank_out += amount,
                None => {}
            }
        }
        let cogs_paise: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(total_paise), 0) FROM purchases
                 WHERE created_at >= ?1 AND created_at <= ?2
                   AND (branch_id = ?3 OR branch_id = '')",
                params![from, to, branch],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(PeriodBooks {
            from_day: from_day.into(),
            to_day: to_day.into(),
            sale_count: day.sale_count,
            revenue_paise: day.revenue_paise,
            declared_paise: day.declared_paise,
            owner_lane_paise: day.owner_lane_paise,
            cash_paise: day.cash_paise,
            upi_paise: day.upi_paise,
            card_paise: day.card_paise,
            credit_paise: day.credit_paise,
            tax_paise: day.tax_paise,
            cogs_paise,
            expenses_paise,
            expense_count,
            bank_in_paise: bank_in,
            bank_out_paise: bank_out,
            outstanding_credit_paise: self.outstanding_credit_total()?,
            outstanding_payable_paise: self.outstanding_payable_total()?,
        })
    }

    pub fn period_books_text(&self, from_day: &str, to_day: &str) -> Result<String, String> {
        let books = self.period_books(from_day, to_day)?;
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let tmpl = self.entity_template()?;
        let items_sold = self.period_items_sold(from_day, to_day)?;
        Ok(format!(
            "Period books {from_day} → {to_day}\n\
             Bills: {}\n\
             Items sold {}\n\
             Revenue {}\n\
             COGS {}\n\
             Gross (rev − COGS) {}\n\
             {} {}\n\
             {} {}\n\
             Cash {} · {} {} · Card {}\n\
             {} (credit / debit to party) {}\n\
             Expenses {} ({})\n\
             Net (gross − expenses) {}\n\
             Bank in {} · Bank out {}\n\
             Bank balance now {}\n\
             They owe us (debit / AR) {}\n\
             We owe them (credit / AP) {}\n\
             {}\n",
            books.sale_count,
            items_sold,
            Money(books.revenue_paise).format_localized(&currency, &fiscal),
            Money(books.cogs_paise).format_localized(&currency, &fiscal),
            Money(books.gross_paise()).format_localized(&currency, &fiscal),
            tmpl.lane_declared_label,
            Money(books.declared_paise).format_localized(&currency, &fiscal),
            tmpl.lane_owner_label,
            Money(books.owner_lane_paise).format_localized(&currency, &fiscal),
            Money(books.cash_paise).format_localized(&currency, &fiscal),
            fiscal.digital_pay_label(),
            Money(books.upi_paise).format_localized(&currency, &fiscal),
            Money(books.card_paise).format_localized(&currency, &fiscal),
            fiscal.credit_label(),
            Money(books.credit_paise).format_localized(&currency, &fiscal),
            Money(books.expenses_paise).format_localized(&currency, &fiscal),
            books.expense_count,
            Money(books.net_paise()).format_localized(&currency, &fiscal),
            Money(books.bank_in_paise).format_localized(&currency, &fiscal),
            Money(books.bank_out_paise).format_localized(&currency, &fiscal),
            Money(self.bank_balance_paise()?).format_localized(&currency, &fiscal),
            Money(books.outstanding_credit_paise).format_localized(&currency, &fiscal),
            Money(books.outstanding_payable_paise).format_localized(&currency, &fiscal),
            self.tax_total_caption(&fiscal),
        ))
    }

    /// Full books pack for Reports: ops + aging + recent bank posts.
    pub fn books_report_text(&self, period: &str) -> Result<String, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let (from, to, title) = match period {
            "month" => {
                let from = Local::now().format("%Y-%m-01").to_string();
                (from, today.clone(), "This month")
            }
            _ => (today.clone(), today.clone(), "Today"),
        };
        let mut out = format!("=== {title} ===\n");
        out.push_str(&self.period_books_text(&from, &to)?);
        out.push('\n');
        out.push_str(&self.debit_credit_report_text()?);
        out.push('\n');
        out.push_str(&self.party_aging_text()?);
        out.push('\n');
        out.push_str(&self.payable_aging_text()?);
        out.push('\n');
        out.push_str(&self.party_position_text()?);
        out.push('\n');
        out.push_str(&self.balance_sheet_text()?);
        out.push('\n');
        out.push_str("Recent bank / cashbook\n");
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let moves = self.list_money_movements(8)?;
        if moves.is_empty() {
            out.push_str("No bank posts yet.\n");
        } else {
            for m in moves {
                let label = MoneyMoveKind::parse(&m.kind)
                    .map(|k| k.label())
                    .unwrap_or(m.kind.as_str());
                let day = m.created_at.get(..10).unwrap_or(&m.created_at);
                out.push_str(&format!(
                    "{day} {label} {} {}\n",
                    Money(m.amount_paise).format_localized(&currency, &fiscal),
                    m.note
                ));
            }
        }
        Ok(out)
    }

    /// Party outstanding pack: debit/credit + AR aging + AP aging + net.
    pub fn outstanding_pack_text(&self) -> Result<String, String> {
        let mut out = String::from("=== Outstanding / debit–credit ===\n");
        out.push_str(&self.debit_credit_report_text()?);
        out.push('\n');
        out.push_str(&self.party_aging_text()?);
        out.push('\n');
        out.push_str(&self.payable_aging_text()?);
        out.push('\n');
        out.push_str(&self.party_position_text()?);
        Ok(out)
    }

    pub fn replace_kots_from_sync(&self, kots: &[KotTicket]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM kot_tickets", [])
            .map_err(|e| e.to_string())?;
        for k in kots {
            self.conn
                .execute(
                    "INSERT INTO kot_tickets(
                        id, sale_id, created_at, status, summary, branch_id, station_id,
                        table_id, total_paise, kot_no, kind, created_by
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        k.id,
                        k.sale_id,
                        k.created_at,
                        k.status.as_str(),
                        k.summary,
                        self.active_branch_id()?,
                        k.station_id,
                        k.table_id,
                        k.total_paise,
                        k.kot_no,
                        if k.kind.is_empty() { "kot" } else { k.kind.as_str() },
                        k.created_by
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn list_products(&self, include_archived: bool) -> Result<Vec<Product>, String> {
        let branch = self.active_branch_id()?;
        let sql = format!(
            "SELECT {PRODUCT_COLUMNS} FROM products
             WHERE (branch_id = ?1 OR branch_id = ''){}
             ORDER BY name",
            if include_archived {
                ""
            } else {
                " AND archived = 0"
            }
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], row_to_product)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn get_product(&self, id: &str) -> Result<Option<Product>, String> {
        self.conn
            .query_row(
                &format!("SELECT {PRODUCT_COLUMNS} FROM products WHERE id = ?1"),
                params![id],
                row_to_product,
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    /// Items in one category (empty id = uncategorised) for big menus.
    pub fn list_products_in_category(&self, category_id: &str) -> Result<Vec<Product>, String> {
        Ok(self
            .list_products(false)?
            .into_iter()
            .filter(|p| p.category_id == category_id)
            .collect())
    }

    pub fn archive_product(&self, product_id: &str) -> Result<(), String> {
        self.assert_server_writer("Remove items")?;
        let n = self
            .conn
            .execute(
                "UPDATE products SET archived = 1 WHERE id = ?1",
                params![product_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Product not found".into());
        }
        Ok(())
    }

    /// Items taken off the menu (archived). Remove is a reversible archive, so
    /// the row keeps everything it had — price, photo, category, station — and
    /// can come back exactly as it was. App-provided dishes (copied from the
    /// bundled shelf) and the owner's own items are restored the same way; the
    /// shelf itself is read-only art that is never deleted.
    pub fn list_removed_products(&self) -> Result<Vec<Product>, String> {
        Ok(self
            .list_products(true)?
            .into_iter()
            .filter(|p| p.archived)
            .collect())
    }

    /// Put a removed item back on the menu, unchanged.
    pub fn restore_product(&self, product_id: &str) -> Result<(), String> {
        self.assert_server_writer("Restore items")?;
        let n = self
            .conn
            .execute(
                "UPDATE products SET archived = 0 WHERE id = ?1",
                params![product_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Product not found".into());
        }
        Ok(())
    }

    pub fn language_source(&self) -> Result<LanguageSource, String> {
        Ok(match self.meta_get("language_source")?.as_deref() {
            Some(raw) => LanguageSource::parse(raw),
            // Existing installs kept a fixed language before this flag existed.
            None => LanguageSource::User,
        })
    }

    pub fn set_language_source(&self, source: LanguageSource) -> Result<(), String> {
        self.meta_set("language_source", source.as_str())
    }

    pub fn locale_settings(&self) -> Result<LocaleSettings, String> {
        let language = match self.language_source()? {
            LanguageSource::Platform => locale::detect_platform_language(),
            LanguageSource::User => self
                .meta_get("language")?
                .unwrap_or_else(|| "en".into()),
        };
        Ok(LocaleSettings {
            language,
            currency: self.meta_get("currency")?.unwrap_or_else(|| "INR".into()),
        })
    }

    /// Format minor units with the shop currency (defaults to INR — India first).
    pub fn money_display(&self, paise: i64) -> String {
        let code = self
            .locale_settings()
            .map(|l| l.currency)
            .unwrap_or_else(|_| "INR".into());
        Money(paise).format_with(&code)
    }

    pub fn set_locale_settings(&self, settings: &LocaleSettings) -> Result<(), String> {
        self.meta_set("language", &settings.language)?;
        self.meta_set("currency", &settings.currency)?;
        Ok(())
    }

    /// True once the first-run language screen has been confirmed (or the
    /// install already had a role / Owner PIN before that screen existed).
    pub fn language_setup_complete(&self) -> Result<bool, String> {
        if self.meta_get("language_setup_complete")?.as_deref() == Some("1") {
            return Ok(true);
        }
        // Existing installs: do not interrupt shops that already started.
        if self.device_role()?.is_some() || self.owner_setup_complete()? {
            self.meta_set("language_setup_complete", "1")?;
            if self.meta_get("language_source")?.is_none() {
                self.meta_set("language_source", LanguageSource::User.as_str())?;
            }
            return Ok(true);
        }
        Ok(false)
    }

    /// Ensure a catalogue language is stored before the first-run picker.
    /// Fresh DBs already get the OS default in `ensure_seed_data`; this is a
    /// safety net if meta was wiped mid-setup.
    pub fn seed_platform_language_if_needed(&self) -> Result<String, String> {
        if self.language_setup_complete()? {
            return Ok(self.locale_settings()?.language);
        }
        let platform = locale::detect_platform_language();
        if self.meta_get("language")?.is_none() {
            let settings = LocaleSettings {
                language: platform.clone(),
                currency: self
                    .meta_get("currency")?
                    .unwrap_or_else(|| "INR".into()),
            };
            self.set_locale_settings(&settings)?;
            return Ok(settings.language);
        }
        // While setup is open, keep the provisional pick in sync with OS when
        // the user has not switched to a fixed language yet.
        Ok(self
            .meta_get("language")?
            .unwrap_or(platform))
    }

    /// Finish first-run language: follow the OS, or lock a catalogue language.
    pub fn complete_language_setup(
        &self,
        source: LanguageSource,
        user_language: Option<&str>,
    ) -> Result<LocaleSettings, String> {
        match source {
            LanguageSource::Platform => {
                let code = locale::detect_platform_language();
                let mut settings = self.locale_settings()?;
                // Cache the current OS language; reads still re-detect while
                // source stays Platform.
                settings.language = code;
                self.set_locale_settings(&settings)?;
                self.set_language_source(LanguageSource::Platform)?;
            }
            LanguageSource::User => {
                let code = user_language
                    .map(str::trim)
                    .filter(|c| locale::language_by_code(c).is_some())
                    .unwrap_or("en");
                let mut settings = self.locale_settings()?;
                settings.language = code.into();
                self.set_locale_settings(&settings)?;
                self.set_language_source(LanguageSource::User)?;
            }
        }
        self.meta_set("language_setup_complete", "1")?;
        self.locale_settings()
    }

    /// Switch to a fixed catalogue language (Settings / first-run list).
    pub fn set_user_language(&self, code: &str) -> Result<LocaleSettings, String> {
        let code = if locale::language_by_code(code).is_some() {
            code.trim()
        } else {
            return Err("Unknown language".into());
        };
        let mut settings = self.locale_settings()?;
        settings.language = code.into();
        self.set_locale_settings(&settings)?;
        self.set_language_source(LanguageSource::User)?;
        self.locale_settings()
    }

    /// Follow the device language from now on.
    pub fn set_platform_language(&self) -> Result<LocaleSettings, String> {
        let code = locale::detect_platform_language();
        let mut settings = self.locale_settings()?;
        settings.language = code;
        self.set_locale_settings(&settings)?;
        self.set_language_source(LanguageSource::Platform)?;
        self.locale_settings()
    }

    // ------------------------------------------------------------- fiscal

    /// Country wording + tax mode for this shop. Kept beside `LocaleSettings`
    /// rather than inside it: a seller in Dubai may well want an English UI
    /// with AED prices and the UAE VAT pack, in any combination.
    pub fn fiscal_settings(&self) -> Result<FiscalSettings, String> {
        let country = self
            .meta_get("country_code")?
            .filter(|c| !c.trim().is_empty())
            .unwrap_or_else(|| fiscal::DEFAULT_COUNTRY.to_string());
        let profile = fiscal::country_or_generic(&country);
        let or_pack = |key: &str, fallback: &str| -> Result<String, String> {
            Ok(self
                .meta_get(key)?
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| fallback.to_string()))
        };
        Ok(FiscalSettings {
            country_code: profile.code.to_string(),
            tax_pricing: TaxPricing::parse(&or_pack(
                "tax_pricing",
                profile.tax_pricing.as_str(),
            )?),
            tax_label: or_pack("tax_label", profile.tax_name)?,
            business_tax_id: self.meta_get("business_tax_id")?.unwrap_or_default(),
            business_id_label: or_pack("business_id_label", profile.business_id_label)?,
            digital_pay_label: or_pack("digital_pay_label", profile.digital_pay_label)?,
            credit_label: or_pack("credit_label", profile.credit_label)?,
            date_order: or_pack("date_order", profile.date_order)?,
            decimal_sep: or_pack("decimal_sep", profile.decimal_sep)?,
            // A space and an empty string are both real grouping choices, so
            // this one is read verbatim and only falls back when unset.
            thousands_sep: self
                .meta_get("thousands_sep")?
                .unwrap_or_else(|| profile.thousands_sep.to_string()),
            service_charge_bps: self
                .meta_get("service_charge_bps")?
                .and_then(|v| v.trim().parse::<i64>().ok())
                .unwrap_or(0)
                .clamp(0, 10_000),
        })
    }

    pub fn set_fiscal_settings(&self, settings: &FiscalSettings) -> Result<(), String> {
        self.assert_server_writer("Change tax settings")?;
        self.write_fiscal_settings(settings)
    }

    fn write_fiscal_settings(&self, settings: &FiscalSettings) -> Result<(), String> {
        self.meta_set("country_code", settings.country_code.trim())?;
        self.meta_set("tax_pricing", settings.tax_pricing.as_str())?;
        self.meta_set("tax_label", settings.tax_label.trim())?;
        self.meta_set("business_tax_id", settings.business_tax_id.trim())?;
        self.meta_set("business_id_label", settings.business_id_label.trim())?;
        self.meta_set("digital_pay_label", settings.digital_pay_label.trim())?;
        self.meta_set("credit_label", settings.credit_label.trim())?;
        self.meta_set("date_order", settings.date_order.trim())?;
        self.meta_set("decimal_sep", settings.decimal_sep())?;
        self.meta_set("thousands_sep", settings.thousands_sep())?;
        self.meta_set(
            "service_charge_bps",
            &settings.service_charge_bps.clamp(0, 10_000).to_string(),
        )?;
        Ok(())
    }

    /// Switch this shop to another country pack.
    ///
    /// Rates are **upserted by code**, so official rates are refreshed in
    /// place. Rates the old pack brought in are retired rather than deleted —
    /// and a rate an item still points at is never retired, so no historical
    /// bill or live menu entry loses its tax. Currency and language move to
    /// the country's defaults too, because a seller changing country is almost
    /// always changing shop, not just tax wording; both stay editable on the
    /// same Settings screen afterwards. The neutral `XX` pack leaves currency
    /// and language alone.
    pub fn apply_country_pack(&self, code: &str) -> Result<FiscalSettings, String> {
        self.assert_server_writer("Change the country pack")?;
        let profile = fiscal::country_by_code(code)
            .ok_or_else(|| format!("Unknown country code \"{}\"", code.trim()))?;
        let business_tax_id = self.meta_get("business_tax_id")?.unwrap_or_default();
        let mut settings = FiscalSettings::from_profile(profile, business_tax_id);
        // A service charge is the house's own rule, so it survives a country
        // switch the way the business id does.
        settings.service_charge_bps = self.fiscal_settings()?.service_charge_bps;
        self.set_fiscal_settings(&settings)?;
        if !profile.default_currency.is_empty() {
            let mut locale = self.locale_settings()?;
            locale.currency = profile.default_currency.to_string();
            if !profile.default_language.is_empty() {
                locale.language = profile.default_language.to_string();
            }
            self.set_locale_settings(&locale)?;
        }
        let branch_id = self.active_branch_id()?;
        let keep: Vec<String> = profile
            .default_rates
            .iter()
            .map(|s| s.code.to_string())
            .collect();
        for rate in self.list_tax_rates()? {
            if keep.iter().any(|c| c.eq_ignore_ascii_case(&rate.code)) {
                continue;
            }
            let in_use: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM products WHERE tax_rate_id = ?1",
                    params![rate.id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if in_use == 0 {
                self.conn
                    .execute(
                        "UPDATE tax_rates SET active = 0 WHERE id = ?1",
                        params![rate.id],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        self.seed_tax_rates(&branch_id, profile)?;
        Ok(settings)
    }

    /// Override inclusive/exclusive without leaving the country pack — some
    /// shops price against the local norm (a UK caterer quoting net, say).
    pub fn set_tax_pricing(&self, pricing: TaxPricing) -> Result<(), String> {
        self.assert_server_writer("Change tax pricing")?;
        self.meta_set("tax_pricing", pricing.as_str())
    }

    /// House service charge in basis points (0 turns it off).
    pub fn set_service_charge_bps(&self, bps: i64) -> Result<(), String> {
        self.assert_server_writer("Change the service charge")?;
        self.meta_set("service_charge_bps", &bps.clamp(0, 10_000).to_string())
    }

    pub fn set_business_tax_id(&self, id: &str) -> Result<(), String> {
        self.assert_server_writer("Set the business id")?;
        self.meta_set("business_tax_id", id.trim())
    }

    /// Rates a seller can pick right now. Rates retired by a country switch
    /// stay in the table (old items keep resolving) but drop out of here.
    pub fn list_tax_rates(&self) -> Result<Vec<TaxRate>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, COALESCE(code, ''), bps, active,
                        COALESCE(picker_enabled, 1)
                 FROM tax_rates
                 WHERE (branch_id = ?1 OR branch_id = '') AND active = 1
                 ORDER BY bps, name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                Ok(TaxRate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    code: row.get(2)?,
                    bps: row.get(3)?,
                    active: row.get::<_, i64>(4)? != 0,
                    picker_enabled: row.get::<_, i64>(5).unwrap_or(1) != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn add_tax_rate(&self, name: &str, code: &str, bps: i64) -> Result<TaxRate, String> {
        self.assert_server_writer("Add tax rates")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Tax rate name required".into());
        }
        if !(0..=10_000).contains(&bps) {
            return Err("Tax rate must be between 0% and 100%".into());
        }
        let branch = self.active_branch_id()?;
        let code = code.trim().to_string();
        if !code.is_empty()
            && let Some(existing) = self
                .list_tax_rates()?
                .into_iter()
                .find(|r| r.code.eq_ignore_ascii_case(&code))
            {
                return Ok(existing);
            }
        let rate = TaxRate {
            id: new_id(),
            name: name.to_string(),
            code,
            bps,
            active: true,
            picker_enabled: true,
        };
        self.conn
            .execute(
                "INSERT INTO tax_rates(id, branch_id, name, code, bps, active)
                 VALUES(?1, ?2, ?3, ?4, ?5, 1)",
                params![rate.id, branch, rate.name, rate.code, rate.bps],
            )
            .map_err(|e| e.to_string())?;
        Ok(rate)
    }

    /// Client cache: adopt the Server's rate table and country wording so a
    /// terminal prints the same tax lines as the till it sells through.
    pub fn replace_fiscal_from_sync(
        &self,
        settings: &FiscalSettings,
        rates: &[TaxRate],
    ) -> Result<(), String> {
        self.write_fiscal_settings(settings)?;
        if rates.is_empty() {
            return Ok(());
        }
        let branch = self.active_branch_id()?;
        self.conn
            .execute("DELETE FROM tax_rates WHERE branch_id = ?1", params![branch])
            .map_err(|e| e.to_string())?;
        for r in rates {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO tax_rates(id, branch_id, name, code, bps, active)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                    params![r.id, branch, r.name, r.code, r.bps, r.active as i64],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Point an item at a rate. `tax_bps` is mirrored onto the product row so
    /// selling, reporting and sync never need the rate table to be present.
    pub fn set_product_tax_rate(&self, product_id: &str, rate_id: &str) -> Result<(), String> {
        self.assert_server_writer("Change item tax")?;
        let rate_id = rate_id.trim();
        if rate_id.is_empty() {
            self.conn
                .execute(
                    "UPDATE products SET tax_rate_id = '', tax_bps = 0 WHERE id = ?1",
                    params![product_id],
                )
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        let rate = self
            .list_tax_rates()?
            .into_iter()
            .find(|r| r.id == rate_id)
            .ok_or_else(|| "Tax rate not found".to_string())?;
        self.conn
            .execute(
                "UPDATE products SET tax_rate_id = ?1, tax_bps = ?2 WHERE id = ?3",
                params![rate.id, rate.bps, product_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Resolve seller input on the Items screen to a rate: an existing name or
    /// code first, then a typed percent, creating `{n}%` when it is new.
    pub fn resolve_tax_rate(&self, typed: &str) -> Result<Option<TaxRate>, String> {
        let typed = typed.trim();
        if typed.is_empty() {
            return Ok(None);
        }
        let rates = self.list_tax_rates()?;
        // "GST 18%" matches a rate name; "18%" and "18" match its code.
        let bare = typed.trim_end_matches('%').trim();
        if let Some(hit) = rates.iter().find(|r| r.matches(typed) || r.matches(bare)) {
            return Ok(Some(hit.clone()));
        }
        let typed = bare;
        let bps = crate::locale::parse_percent_bps(typed)
            .map_err(|_| format!("No tax rate named \"{typed}\" — type a name or a percent"))?;
        if !(0..=10_000).contains(&bps) {
            return Err("Tax % must be between 0 and 100".into());
        }
        if let Some(hit) = rates.into_iter().find(|r| r.bps == bps) {
            return Ok(Some(hit));
        }
        let label = self.fiscal_settings()?.tax_label().to_string();
        Ok(Some(self.add_tax_rate(
            &format!("{label} {typed}%"),
            typed,
            bps,
        )?))
    }

    /// Tax this draft carries under the shop's current pricing mode, plus the
    /// total that will actually be collected.
    fn resolve_draft_tax(&self, draft: &SaleDraft) -> Result<(i64, i64, FiscalSettings), String> {
        let fiscal_settings = self.fiscal_settings()?;
        let pricing = fiscal_settings.tax_pricing;
        let mut tax_paise = 0i64;
        for line in &draft.lines {
            let bps = self
                .get_product(&line.product_id)?
                .map(|p| p.tax_bps)
                .unwrap_or(0);
            tax_paise =
                tax_paise.saturating_add(fiscal::tax_on_amount(line.line_total_paise, bps, pricing));
        }
        let total_paise = fiscal::gross_with_tax(draft.total_paise, tax_paise, pricing)
            .saturating_add(fiscal_settings.service_on(draft.lines_total_paise));
        Ok((tax_paise, total_paise, fiscal_settings))
    }

    /// What this cart will actually be charged, tax and any house service
    /// charge included. Inclusive shops get the quoted total back; exclusive
    /// shops get it plus tax.
    pub fn preview_sale_total(&self, draft: &SaleDraft) -> Result<i64, String> {
        Ok(self.resolve_draft_tax(draft)?.1)
    }

    /// Staff id kept unlocked across process restarts / hot-reload rebuilds.
    /// Cleared only when the seller explicitly locks the app.
    pub fn set_active_session_staff_id(&self, staff_id: Option<&str>) -> Result<(), String> {
        match staff_id {
            Some(id) if !id.is_empty() => self.meta_set("active_session_staff_id", id),
            _ => self.meta_set("active_session_staff_id", ""),
        }
    }

    fn meta_get(&self, key: &str) -> Result<Option<String>, String> {
        self.conn
            .query_row(
                "SELECT value FROM meta WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    fn meta_set(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO meta(key, value) VALUES(?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![key, value],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// `server` (this device holds the books) | `client` (extra till).
    pub fn device_role(&self) -> Result<Option<String>, String> {
        self.meta_get("device_role")
    }

    pub fn set_device_role(&self, role: &str) -> Result<(), String> {
        match role {
            "server" | "client" => self.meta_set("device_role", role),
            _ => Err("device_role must be server or client".into()),
        }
    }

    /// Extra tills may pair on LAN. Default off until the owner allows them.
    pub fn extra_tills_allowed(&self) -> Result<bool, String> {
        if self.is_client_device()? {
            return Ok(false);
        }
        Ok(self
            .meta_get("extra_tills_allowed")?
            .as_deref()
            .map(str::trim)
            == Some("1"))
    }

    pub fn set_extra_tills_allowed(&self, allowed: bool) -> Result<(), String> {
        self.assert_server_writer("Change extra tills")?;
        self.meta_set("extra_tills_allowed", if allowed { "1" } else { "0" })
    }

    /// In-app venue name (lock screen, bills). Empty → neutral "Your shop"
    /// in UI. Does not change OS / Play Store app name. A local server's
    /// generated label ("Server af76697b") is not a restaurant's name: older
    /// books that took it as one read as unnamed.
    pub fn venue_title(&self) -> Result<String, String> {
        let title = self.meta_get("venue_title")?.unwrap_or_default();
        Ok(if is_generated_server_label(&title) { String::new() } else { title })
    }

    /// The name other devices list this restaurant by when they look for
    /// it: its own name, else the business name it saved ("" = neither; the
    /// list then shows the app name and the address).
    pub fn listing_name(&self) -> Result<String, String> {
        let venue = self.venue_title()?;
        if venue.trim().is_empty() {
            self.saved_org_name()
        } else {
            Ok(venue)
        }
    }

    pub fn set_venue_title(&self, title: &str) -> Result<(), String> {
        self.meta_set("venue_title", title.trim())
    }

    /// Optional custom mark: `custom:filename` under brand_images/. None → no in-app icon.
    pub fn venue_icon_key(&self) -> Result<Option<String>, String> {
        Ok(self
            .meta_get("venue_icon_key")?
            .filter(|s| !s.trim().is_empty()))
    }

    pub fn set_venue_icon_key(&self, key: Option<&str>) -> Result<(), String> {
        match key.map(str::trim).filter(|s| !s.is_empty()) {
            Some(k) => self.meta_set("venue_icon_key", k),
            None => self.meta_set("venue_icon_key", ""),
        }
    }

    /// User-arranged sidebar / bottom-bar order (CSV of nav tab keys).
    /// Stored in the books so the arrangement survives reinstall and
    /// follows the shop's data, not one binary install.
    pub fn nav_order_csv(&self) -> Result<Option<String>, String> {
        self.meta_get("ui_nav_order")
    }

    pub fn set_nav_order_csv(&self, csv: &str) -> Result<(), String> {
        self.meta_set("ui_nav_order", csv)
    }

    /// Whether ₹0 item prices and ₹0 bills (and a matching ₹0 tender) are
    /// allowed. Default **on** — complimentary / owner-wish free covers, and
    /// free menu items, still write a completed sale so Frequent can rank them.
    pub fn allow_zero_sales(&self) -> Result<bool, String> {
        Ok(match self.meta_get("allow_zero_sales")?.as_deref() {
            Some("0") | Some("false") | Some("off") => false,
            _ => true,
        })
    }

    pub fn set_allow_zero_sales(
        &self,
        allow: bool,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Zero-value sales setting")?;
        if !actor.is_owner() {
            return Err("Only the Owner can change zero-value sales".into());
        }
        self.meta_set("allow_zero_sales", if allow { "1" } else { "0" })?;
        Ok(())
    }

    /// Frequent-items strip: how many calendar days of completed sales to
    /// rank, and how many dishes to show. Defaults: 1 day, 24 items.
    pub fn frequent_config(&self) -> Result<(u32, u32), String> {
        let days = self
            .meta_get("frequent_days")?
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&d| (1..=90).contains(&d))
            .unwrap_or(1);
        let limit = self
            .meta_get("frequent_limit")?
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&n| (1..=100).contains(&n))
            .unwrap_or(24);
        Ok((days, limit))
    }

    pub fn set_frequent_config(
        &self,
        days: u32,
        limit: u32,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Frequent items settings")?;
        if !(1..=90).contains(&days) {
            return Err("Period must be 1–90 days".into());
        }
        if !(1..=100).contains(&limit) {
            return Err("Show between 1 and 100 items".into());
        }
        self.meta_set("frequent_days", &days.to_string())?;
        self.meta_set("frequent_limit", &limit.to_string())?;
        let _ = actor;
        Ok(())
    }

    /// Product ids ranked by quantity sold in completed bills over the last
    /// `days` (including today), capped at `limit`. Most frequent first.
    pub fn frequent_product_ids(&self, days: u32, limit: u32) -> Result<Vec<String>, String> {
        let days = days.max(1);
        let limit = limit.max(1) as i64;
        let start = (Local::now().date_naive() - chrono::Duration::days((days as i64) - 1))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .earliest()
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| Local::now().to_rfc3339());
        let mut stmt = self
            .conn
            .prepare(
                "SELECT sl.product_id,
                        SUM(CASE WHEN sl.qty_milli > 0 THEN sl.qty_milli ELSE sl.qty * 1000 END) AS sold
                   FROM sale_lines sl
                   JOIN sales s ON s.id = sl.sale_id
                  WHERE s.status = 'completed'
                    AND s.created_at >= ?1
                    AND sl.product_id != ''
                  GROUP BY sl.product_id
                  ORDER BY sold DESC
                  LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![start, limit], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn brand_images_dir(&self) -> Result<PathBuf, String> {
        let dir = self
            .path
            .parent()
            .unwrap_or(Path::new("."))
            .join("brand_images");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    pub fn import_venue_icon(&self, source: &Path) -> Result<String, String> {
        let dir = self.brand_images_dir()?;
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !media::is_image_ext(&ext) {
            return Err("Use jpg, jpeg, png, or webp (stored as WebP)".into());
        }
        let file_name = "venue_icon.webp".to_string();
        let dest = dir.join(&file_name);
        media::write_image_as_webp(source, &dest)?;
        let key = format!("custom:{file_name}");
        self.set_venue_icon_key(Some(&key))?;
        Ok(key)
    }

    // ---------------------------------------------------------------- org / branches

    pub fn org_id(&self) -> Result<String, String> {
        Ok(self.meta_get("org_id")?.unwrap_or_default())
    }

    /// The business name for reports and roll-ups: the one the owner saved,
    /// else the shop's own name, else [`DEFAULT_ORG_NAME`].
    pub fn org_name(&self) -> Result<String, String> {
        let saved = self.saved_org_name()?;
        if !saved.is_empty() {
            return Ok(saved);
        }
        let venue = self.venue_title()?;
        Ok(if venue.trim().is_empty() { DEFAULT_ORG_NAME.into() } else { venue })
    }

    /// The business name exactly as the owner saved it ("" = never saved).
    /// Books made before nothing was written for the owner hold
    /// [`DEFAULT_ORG_NAME`] — that was never theirs, so it reads as unsaved.
    pub fn saved_org_name(&self) -> Result<String, String> {
        let name = self.meta_get("org_name")?.unwrap_or_default();
        let name = name.trim();
        Ok(if name.is_empty() || name == DEFAULT_ORG_NAME || is_generated_server_label(name) {
            String::new()
        } else {
            name.to_string()
        })
    }

    pub fn set_org_name(&self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Business name required".into());
        }
        self.meta_set("org_name", name)
    }

    /// Branch this device is selling for. Every row written from here is tagged
    /// with it so one org can hold many shops in one schema.
    pub fn active_branch_id(&self) -> Result<String, String> {
        Ok(self.meta_get("active_branch_id")?.unwrap_or_default())
    }

    pub fn set_active_branch_id(&self, branch_id: &str) -> Result<(), String> {
        let branch_id = branch_id.trim();
        let found: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM branches WHERE id = ?1 AND active = 1",
                params![branch_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if found == 0 {
            return Err("Branch not found".into());
        }
        self.meta_set("active_branch_id", branch_id)
    }

    pub fn list_branches(&self) -> Result<Vec<Branch>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, org_id, name, code, active, is_this_device,
                        COALESCE(last_snapshot_json, ''), COALESCE(last_sync_at, '')
                 FROM branches
                 ORDER BY is_this_device DESC, name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Branch {
                    id: row.get(0)?,
                    org_id: row.get(1)?,
                    name: row.get(2)?,
                    code: row.get(3)?,
                    active: row.get::<_, i64>(4)? != 0,
                    is_this_device: row.get::<_, i64>(5)? != 0,
                    last_snapshot_json: row.get(6)?,
                    last_sync_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn active_branch(&self) -> Result<Branch, String> {
        let id = self.active_branch_id()?;
        self.list_branches()?
            .into_iter()
            .find(|b| b.id == id)
            .ok_or_else(|| "Active branch missing".to_string())
    }

    /// Add a sibling outlet. `is_this_device` stays 0 — only one branch sells
    /// from this device; others report in via snapshots.
    pub fn add_branch(&self, name: &str, code: &str) -> Result<Branch, String> {
        self.assert_server_writer("Add branches")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Branch name required".into());
        }
        let code = code.trim().to_uppercase();
        let clash: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM branches WHERE lower(name) = lower(?1)",
                params![name],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if clash > 0 {
            return Err("Branch name already used".into());
        }
        let branch = Branch {
            id: new_id(),
            org_id: self.org_id()?,
            name: name.to_string(),
            code,
            active: true,
            is_this_device: false,
            last_snapshot_json: String::new(),
            last_sync_at: String::new(),
        };
        self.conn
            .execute(
                "INSERT INTO branches(
                    id, org_id, name, code, active, is_this_device,
                    last_snapshot_json, last_sync_at
                 ) VALUES(?1, ?2, ?3, ?4, 1, 0, '', '')",
                params![branch.id, branch.org_id, branch.name, branch.code],
            )
            .map_err(|e| e.to_string())?;
        Ok(branch)
    }

    pub fn rename_branch(&self, branch_id: &str, name: &str) -> Result<(), String> {
        self.assert_server_writer("Rename branches")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Branch name required".into());
        }
        let n = self
            .conn
            .execute(
                "UPDATE branches SET name = ?1 WHERE id = ?2",
                params![name, branch_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Branch not found".into());
        }
        Ok(())
    }

    // ------------------------------------------------------- categories / stations

    pub fn list_categories(&self) -> Result<Vec<Category>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, COALESCE(branch_id, ''), name, sort_order,
                        COALESCE(image_key, '') FROM categories
                 WHERE branch_id = ?1 OR branch_id = ''
                 ORDER BY sort_order, name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    branch_id: row.get(1)?,
                    name: row.get(2)?,
                    sort_order: row.get(3)?,
                    image_key: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Bring in the whole provided menu template: every catalogue folder as a
    /// category (with a photo) and every dish as a menu item in it, priced ₹0
    /// for the owner to set, its photo referenced from the catalogue. What
    /// the shop already has (same name, ignoring case) is kept, never
    /// duplicated — importing twice adds nothing.
    pub fn import_catalogue(&self, dishes: &[crate::menu_images::ShelfDish]) -> Result<CatalogueImport, String> {
        self.assert_server_writer("Import the menu template")?;
        let mut summary = CatalogueImport::default();
        self.conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
        let result = (|| -> Result<(), String> {
            let mut categories = self.list_categories()?;
            let existing: std::collections::HashSet<String> = self
                .list_products(true)?
                .into_iter()
                .map(|p| p.name.trim().to_lowercase())
                .collect();
            let mut seen = existing;
            for dish in dishes {
                let cat_id = match categories
                    .iter()
                    .find(|c| c.name.trim().eq_ignore_ascii_case(dish.folder.trim()))
                {
                    Some(c) => {
                        if c.image_key.is_empty()
                            && let Some(key) = crate::menu_images::shelf_key(&dish.path)
                        {
                            self.set_category_image(&c.id, &key)?;
                        }
                        c.id.clone()
                    }
                    None => {
                        let mut c = self.add_category(&dish.folder)?;
                        if let Some(key) = crate::menu_images::shelf_key(&dish.path) {
                            self.set_category_image(&c.id, &key)?;
                            c.image_key = key;
                        }
                        summary.categories_added += 1;
                        let id = c.id.clone();
                        categories.push(c);
                        id
                    }
                };
                if !seen.insert(dish.name.trim().to_lowercase()) {
                    summary.items_kept += 1;
                    continue;
                }
                let mut p = Product::new_basic(new_id(), dish.name.trim(), Money(0));
                p.category_id = cat_id;
                p.image_key = crate::menu_images::shelf_key(&dish.path);
                self.upsert_product(&p)?;
                summary.items_added += 1;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                Ok(summary)
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    pub fn add_category(&self, name: &str) -> Result<Category, String> {
        self.assert_server_writer("Add categories")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Category name required".into());
        }
        let branch_id = self.active_branch_id()?;
        let next: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM categories WHERE branch_id = ?1",
                params![branch_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let cat = Category {
            id: new_id(),
            branch_id,
            name: name.to_string(),
            sort_order: next,
            image_key: String::new(),
        };
        self.conn
            .execute(
                "INSERT INTO categories(id, branch_id, name, sort_order) VALUES(?1, ?2, ?3, ?4)",
                params![cat.id, cat.branch_id, cat.name, cat.sort_order],
            )
            .map_err(|e| e.to_string())?;
        Ok(cat)
    }

    pub fn set_category_image(&self, category_id: &str, image_key: &str) -> Result<(), String> {
        self.assert_server_writer("Change category photo")?;
        let n = self
            .conn
            .execute(
                "UPDATE categories SET image_key = ?1 WHERE id = ?2",
                params![image_key, category_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Category not found".into());
        }
        Ok(())
    }

    /// Import an owner-picked category photo (stored as WebP next to the
    /// item photos, keyed by the category's UUID — categories are
    /// identified by id everywhere, never by name).
    /// Give a category a photo only if it has none — how a shelf folder
    /// picked as the category brings its photo (the owner's own photo is
    /// never replaced). Returns whether a photo was set.
    pub fn set_category_photo_if_missing(&self, category_id: &str, source: &Path) -> Result<bool, String> {
        let has_photo = self
            .list_categories()?
            .into_iter()
            .find(|c| c.id == category_id)
            .ok_or_else(|| "Category not found".to_string())?
            .image_key
            .trim()
            .len()
            > 0;
        if has_photo {
            return Ok(false);
        }
        let key = self.import_category_image(category_id, source)?;
        self.set_category_image(category_id, &key)?;
        Ok(true)
    }

    pub fn import_category_image(
        &self,
        category_id: &str,
        source: &Path,
    ) -> Result<String, String> {
        let dir = self.item_images_dir()?;
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !media::is_image_ext(&ext) {
            return Err("Use jpg, jpeg, png, or webp (stored as WebP)".into());
        }
        let file_name = format!("cat-{category_id}.webp");
        let dest = dir.join(&file_name);
        media::write_image_as_webp_fit(source, &dest, ITEM_PHOTO_EDGE)?;
        Ok(format!("custom:{file_name}"))
    }

    pub fn list_stations(&self) -> Result<Vec<KitchenStation>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, COALESCE(branch_id, ''), name, sort_order,
                        COALESCE(target_prep_mins, 15),
                        COALESCE(picker_enabled, 1)
                 FROM kitchen_stations
                 WHERE branch_id = ?1 OR branch_id = ''
                 ORDER BY sort_order, name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                Ok(KitchenStation {
                    id: row.get(0)?,
                    branch_id: row.get(1)?,
                    name: row.get(2)?,
                    sort_order: row.get(3)?,
                    target_prep_mins: row.get(4)?,
                    picker_enabled: row.get::<_, i64>(5).unwrap_or(1) != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn add_station(&self, name: &str) -> Result<KitchenStation, String> {
        self.assert_server_writer("Add kitchen stations")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Station name required".into());
        }
        let branch_id = self.active_branch_id()?;
        let next: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM kitchen_stations WHERE branch_id = ?1",
                params![branch_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let station = KitchenStation {
            id: new_id(),
            branch_id,
            name: name.to_string(),
            sort_order: next,
            target_prep_mins: 15,
            picker_enabled: true,
        };
        self.conn
            .execute(
                "INSERT INTO kitchen_stations(id, branch_id, name, sort_order, target_prep_mins)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                params![
                    station.id,
                    station.branch_id,
                    station.name,
                    station.sort_order,
                    station.target_prep_mins
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(station)
    }

    // ------------------------------------------------------------------- inventory

    /// Start (or stop, with −1) tracking stock for an item.
    pub fn set_product_stock_qty(&self, product_id: &str, qty: i64) -> Result<(), String> {
        self.assert_server_writer("Track stock")?;
        let qty = if qty < 0 { STOCK_UNTRACKED } else { qty };
        let n = self
            .conn
            .execute(
                "UPDATE products SET stock_qty = ?1 WHERE id = ?2",
                params![qty, product_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Product not found".into());
        }
        Ok(())
    }

    /// Move stock by `delta_qty` and append the reason to `stock_movements`.
    /// Untracked items (`stock_qty = -1`) still record the movement so the
    /// paper trail is complete, but their on-hand count stays untracked.
    /// Returns the on-hand quantity after the move.
    pub fn adjust_stock(
        &self,
        product_id: &str,
        delta_qty: i64,
        reason: &str,
        note: &str,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Adjust stock")?;
        if delta_qty == 0 {
            return Err("Stock change must not be zero".into());
        }
        let product = self
            .get_product(product_id)?
            .ok_or_else(|| "Product not found".to_string())?;
        let mut new_qty = product.stock_qty;
        if product.tracks_stock() {
            new_qty = product.stock_qty.saturating_add(delta_qty);
            if new_qty < 0 {
                return Err(format!(
                    "Not enough stock for {}: {} on hand",
                    product.name, product.stock_qty
                ));
            }
            self.conn
                .execute(
                    "UPDATE products SET stock_qty = ?1 WHERE id = ?2",
                    params![new_qty, product_id],
                )
                .map_err(|e| e.to_string())?;
        }
        let reason = {
            let r = reason.trim();
            if r.is_empty() {
                "adjust"
            } else {
                r
            }
        };
        self.record_stock_movement(product_id, delta_qty, reason, note.trim(), actor, "")?;
        self.audit(
            actor,
            "stock.adjust",
            &format!("{product_id} {delta_qty} {reason}"),
        )?;
        Ok(new_qty)
    }

    fn record_stock_movement(
        &self,
        product_id: &str,
        delta_qty: i64,
        reason: &str,
        note: &str,
        actor: &StaffMember,
        ref_id: &str,
    ) -> Result<(), String> {
        let times = EventTimes::capture(None);
        self.conn
            .execute(
                "INSERT INTO stock_movements(
                    id, branch_id, product_id, created_at, delta_qty,
                    reason, note, actor_id, actor_name, ref_id,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    new_id(),
                    self.active_branch_id()?,
                    product_id,
                    times.user_at,
                    delta_qty,
                    reason,
                    note,
                    actor.id,
                    actor.name,
                    ref_id,
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Goods received: records the bill, raises stock on tracked items and
    /// leaves one `purchase` movement per line.
    ///
    /// `lines` are `(product_id, qty_milli, unit_cost_paise)`.
    /// When `party_id` is set and `on_account`, the total posts to AP (we owe them).
    pub fn record_purchase(
        &self,
        supplier: &str,
        lines: &[(String, i64, i64)],
        note: &str,
        actor: &StaffMember,
    ) -> Result<Purchase, String> {
        self.record_purchase_ex(supplier, "", false, lines, note, actor)
    }

    pub fn record_purchase_ex(
        &self,
        supplier: &str,
        party_id: &str,
        on_account: bool,
        lines: &[(String, i64, i64)],
        note: &str,
        actor: &StaffMember,
    ) -> Result<Purchase, String> {
        self.assert_server_writer("Record purchases")?;
        if lines.is_empty() {
            return Err("Purchase needs at least one item".into());
        }
        if !actor.can_access_inventory() {
            return Err("Not allowed to record purchases".into());
        }
        let party_id = party_id.trim();
        if on_account && party_id.is_empty() {
            return Err("On-account purchase needs a supplier party".into());
        }
        if !party_id.is_empty() {
            let _ = self
                .get_customer(party_id)?
                .ok_or_else(|| "Supplier party not found".to_string())?;
        }
        let branch_id = self.active_branch_id()?;
        let times = EventTimes::capture(None);
        let created_at = times.user_at.clone();
        let purchase_id = new_id();
        let mut built = Vec::with_capacity(lines.len());
        for (product_id, qty_milli, unit_cost_paise) in lines {
            let qty_milli = if *qty_milli > 0 {
                *qty_milli
            } else {
                return Err("Purchase quantity must be > 0".into());
            };
            if *unit_cost_paise < 0 {
                return Err("Unit cost cannot be negative".into());
            }
            let product = self
                .get_product(product_id)?
                .ok_or_else(|| format!("Product not found: {product_id}"))?;
            let qty_whole = milli_to_whole_qty(qty_milli).max(1);
            built.push((
                PurchaseLine {
                    id: new_id(),
                    purchase_id: purchase_id.clone(),
                    product_id: product.id.clone(),
                    name: product.name.clone(),
                    qty: qty_whole,
                    unit_cost_paise: *unit_cost_paise,
                    qty_milli,
                    unit_label: product.unit_label.clone(),
                },
                product.tracks_stock(),
            ));
        }
        let total_paise: i64 = built
            .iter()
            .map(|(l, _)| l.line_cost_paise())
            .fold(0i64, |a, b| a.saturating_add(b));
        let supplier_name = if !party_id.is_empty() {
            self.get_customer(party_id)?
                .map(|c| c.name)
                .unwrap_or_else(|| supplier.trim().to_string())
        } else {
            supplier.trim().to_string()
        };
        self.conn
            .execute(
                "INSERT INTO purchases(
                    id, branch_id, created_at, supplier, total_paise, note, actor_id, actor_name,
                    party_id, on_account, user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    purchase_id,
                    branch_id,
                    created_at,
                    supplier_name,
                    total_paise,
                    note.trim(),
                    actor.id,
                    actor.name,
                    party_id,
                    if on_account { 1 } else { 0 },
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        for (line, tracked) in &built {
            self.conn
                .execute(
                    "INSERT INTO purchase_lines(
                        id, purchase_id, product_id, name, qty, unit_cost_paise, qty_milli, unit_label
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        line.id,
                        line.purchase_id,
                        line.product_id,
                        line.name,
                        line.qty,
                        line.unit_cost_paise,
                        line.qty_milli,
                        line.unit_label
                    ],
                )
                .map_err(|e| e.to_string())?;
            if *tracked {
                self.conn
                    .execute(
                        "UPDATE products SET stock_qty = stock_qty + ?1
                         WHERE id = ?2 AND stock_qty >= 0",
                        params![line.qty, line.product_id],
                    )
                    .map_err(|e| e.to_string())?;
            }
            self.record_stock_movement(
                &line.product_id,
                line.qty,
                "purchase",
                &supplier_name,
                actor,
                &purchase_id,
            )?;
        }
        if on_account && !party_id.is_empty() && total_paise > 0 {
            self.conn
                .execute(
                    "INSERT INTO payable_ledger(
                        id, party_id, created_at, kind, amount_paise, purchase_id,
                        pay_method, actor_id, actor_name, detail,
                        user_at, device_at, server_at
                     ) VALUES(?1, ?2, ?3, 'purchase', ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        new_id(),
                        party_id,
                        created_at,
                        total_paise,
                        purchase_id,
                        actor.id,
                        actor.name,
                        format!("Purchase {supplier_name}"),
                        times.user_at,
                        times.device_at,
                        times.server_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        self.audit(
            actor,
            "purchase.add",
            &format!("{purchase_id} {supplier_name} {total_paise}"),
        )?;
        Ok(Purchase {
            id: purchase_id,
            branch_id,
            created_at,
            user_at: times.user_at,
            device_at: times.device_at,
            server_at: times.server_at,
            supplier: supplier_name,
            party_id: party_id.to_string(),
            on_account,
            total_paise,
            note: note.trim().to_string(),
            actor_id: actor.id.clone(),
            actor_name: actor.name.clone(),
            lines: built.into_iter().map(|(l, _)| l).collect(),
        })
    }

    /// Tracked items at or below `threshold` (untracked items never appear).
    pub fn list_low_stock(&self, threshold: i64) -> Result<Vec<Product>, String> {
        Ok(self
            .list_products(false)?
            .into_iter()
            .filter(|p| p.tracks_stock() && p.stock_qty <= threshold)
            .collect())
    }

    pub fn list_stock_movements(&self, limit: i64) -> Result<Vec<StockMovement>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, COALESCE(m.branch_id, ''), m.product_id,
                        COALESCE(p.name, ''), m.created_at, m.delta_qty,
                        m.reason, m.note, m.actor_id, m.actor_name, m.ref_id,
                        COALESCE(NULLIF(m.user_at, ''), m.created_at),
                        COALESCE(NULLIF(m.device_at, ''), m.created_at),
                        COALESCE(m.server_at, '')
                 FROM stock_movements m
                 LEFT JOIN products p ON p.id = m.product_id
                 WHERE m.branch_id = ?1 OR m.branch_id = ''
                 ORDER BY m.created_at DESC, m.rowid DESC
                 LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch, limit], |row| {
                Ok(StockMovement {
                    id: row.get(0)?,
                    branch_id: row.get(1)?,
                    product_id: row.get(2)?,
                    product_name: row.get(3)?,
                    created_at: row.get(4)?,
                    delta_qty: row.get(5)?,
                    reason: row.get(6)?,
                    note: row.get(7)?,
                    actor_id: row.get(8)?,
                    actor_name: row.get(9)?,
                    ref_id: row.get(10)?,
                    user_at: row.get(11)?,
                    device_at: row.get(12)?,
                    server_at: row.get(13)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_purchases(&self, limit: i64) -> Result<Vec<Purchase>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, COALESCE(branch_id, ''), created_at, supplier,
                        total_paise, note, actor_id, actor_name,
                        COALESCE(party_id, ''), COALESCE(on_account, 0),
                        COALESCE(NULLIF(user_at, ''), created_at),
                        COALESCE(NULLIF(device_at, ''), created_at),
                        COALESCE(server_at, '')
                 FROM purchases
                 WHERE branch_id = ?1 OR branch_id = ''
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let mut purchases = stmt
            .query_map(params![branch, limit], |row| {
                Ok(Purchase {
                    id: row.get(0)?,
                    branch_id: row.get(1)?,
                    created_at: row.get(2)?,
                    supplier: row.get(3)?,
                    total_paise: row.get(4)?,
                    note: row.get(5)?,
                    actor_id: row.get(6)?,
                    actor_name: row.get(7)?,
                    party_id: row.get(8)?,
                    on_account: row.get::<_, i64>(9)? != 0,
                    user_at: row.get(10)?,
                    device_at: row.get(11)?,
                    server_at: row.get(12)?,
                    lines: Vec::new(),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for purchase in &mut purchases {
            purchase.lines = self.purchase_lines(&purchase.id)?;
        }
        Ok(purchases)
    }

    pub fn purchase_lines(&self, purchase_id: &str) -> Result<Vec<PurchaseLine>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, purchase_id, product_id, name, qty, unit_cost_paise,
                        COALESCE(qty_milli, 0), COALESCE(unit_label, '')
                 FROM purchase_lines WHERE purchase_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![purchase_id], |row| {
                Ok(PurchaseLine {
                    id: row.get(0)?,
                    purchase_id: row.get(1)?,
                    product_id: row.get(2)?,
                    name: row.get(3)?,
                    qty: row.get(4)?,
                    unit_cost_paise: row.get(5)?,
                    qty_milli: row.get(6)?,
                    unit_label: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------- cloud outbox

    /// Durable queue of local events for a future cloud drain. Nothing in the
    /// app reads it yet — it only has to survive restarts.
    pub fn enqueue_outbox(&self, event_type: &str, payload_json: &str) -> Result<String, String> {
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO sync_outbox(
                    id, branch_id, created_at, event_type, payload_json, synced_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, '')",
                params![
                    id,
                    self.active_branch_id()?,
                    Local::now().to_rfc3339(),
                    event_type,
                    payload_json
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    /// (id, event_type, payload_json) for events not yet pushed anywhere.
    pub fn list_pending_outbox(&self, limit: i64) -> Result<Vec<(String, String, String)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, event_type, payload_json FROM sync_outbox
                 WHERE synced_at = '' ORDER BY created_at ASC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Mark an outbox row synced and stamp `server_at` on the related books row
    /// (sale id from `sale.finalize` payload) when still empty.
    pub fn mark_outbox_synced(&self, id: &str) -> Result<(), String> {
        let server_at = Local::now().to_rfc3339();
        let row: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT event_type, payload_json FROM sync_outbox WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE sync_outbox SET synced_at = ?1 WHERE id = ?2",
                params![server_at, id],
            )
            .map_err(|e| e.to_string())?;
        if let Some((event_type, payload)) = row {
            if event_type == "sale.finalize" {
                let sale_id = serde_json::from_str::<Sale>(&payload)
                    .ok()
                    .map(|s| s.id)
                    .or_else(|| {
                        serde_json::from_str::<serde_json::Value>(&payload)
                            .ok()
                            .and_then(|v| v.get("id")?.as_str().map(str::to_string))
                    });
                if let Some(sale_id) = sale_id {
                    self.conn
                        .execute(
                            "UPDATE sales SET server_at = ?1
                             WHERE id = ?2 AND IFNULL(server_at, '') = ''",
                            params![server_at, sale_id],
                        )
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    // ------------------------------------------------- branch snapshots / roll-up

    /// This branch's day as a portable payload (file, QR, chat message) so a
    /// multi-branch owner can consolidate without hosting anything.
    pub fn export_branch_snapshot(&self) -> Result<BranchSnapshot, String> {
        let branch = self.active_branch()?;
        let report = self.ops_report_today()?;
        Ok(BranchSnapshot {
            branch_id: branch.id,
            branch_name: branch.name,
            branch_code: branch.code,
            org_id: self.org_id()?,
            org_name: self.org_name()?,
            exported_at: Local::now().to_rfc3339(),
            day: Local::now().format("%Y-%m-%d").to_string(),
            totals: report.day,
            outstanding_credit_paise: report.outstanding_credit_paise,
            expenses_paise: report.expenses_paise,
            expense_count: report.expense_count,
            top_items: report.top_items,
        })
    }

    pub fn export_branch_snapshot_json(&self) -> Result<String, String> {
        Ok(self.export_branch_snapshot()?.to_json())
    }

    /// Store another branch's exported day against its `branches` row, creating
    /// the row when this device has not seen that branch before.
    pub fn import_branch_snapshot(&self, json: &str) -> Result<Branch, String> {
        self.assert_server_writer("Import a branch day")?;
        let snapshot = BranchSnapshot::from_json(json)?;
        if snapshot.branch_id.trim().is_empty() {
            return Err("Snapshot has no branch id".into());
        }
        if snapshot.branch_id == self.active_branch_id()? {
            return Err("That snapshot is from this device's own branch".into());
        }
        let known: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM branches WHERE id = ?1",
                params![snapshot.branch_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let name = if snapshot.branch_name.trim().is_empty() {
            format!("Branch {}", &snapshot.branch_id[..8.min(snapshot.branch_id.len())])
        } else {
            snapshot.branch_name.trim().to_string()
        };
        if known == 0 {
            self.conn
                .execute(
                    "INSERT INTO branches(
                        id, org_id, name, code, active, is_this_device,
                        last_snapshot_json, last_sync_at
                     ) VALUES(?1, ?2, ?3, ?4, 1, 0, '', '')",
                    params![
                        snapshot.branch_id,
                        self.org_id()?,
                        name,
                        snapshot.branch_code.trim()
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        self.conn
            .execute(
                "UPDATE branches SET last_snapshot_json = ?1, last_sync_at = ?2
                 WHERE id = ?3",
                params![
                    snapshot.to_json(),
                    Local::now().to_rfc3339(),
                    snapshot.branch_id
                ],
            )
            .map_err(|e| e.to_string())?;
        self.list_branches()?
            .into_iter()
            .find(|b| b.id == snapshot.branch_id)
            .ok_or_else(|| "Branch missing after import".to_string())
    }

    /// Owner roll-up: this device's live day plus the latest snapshot from every
    /// other branch. Snapshots from an earlier day are still counted and the
    /// branch is flagged so the UI can label it "last known".
    pub fn consolidated_report(&self) -> Result<OrgConsolidatedReport, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let active_id = self.active_branch_id()?;
        let mut out = OrgConsolidatedReport {
            org_name: self.org_name()?,
            ..Default::default()
        };
        let local = self.ops_report_today()?;
        for branch in self.list_branches()? {
            if branch.id == active_id {
                out.totals.merge(&local.day);
                out.outstanding_credit_paise += local.outstanding_credit_paise;
                out.expenses_paise += local.expenses_paise;
                out.branches.push((branch.name, local.day.clone()));
                continue;
            }
            match BranchSnapshot::from_json(&branch.last_snapshot_json) {
                Ok(snapshot) => {
                    if snapshot.day != today {
                        out.stale_branches.push(branch.name.clone());
                    }
                    out.totals.merge(&snapshot.totals);
                    out.outstanding_credit_paise += snapshot.outstanding_credit_paise;
                    out.expenses_paise += snapshot.expenses_paise;
                    out.branches.push((branch.name, snapshot.totals));
                }
                Err(_) => {
                    out.missing_branches.push(branch.name.clone());
                    out.branches.push((branch.name, DayTotals::default()));
                }
            }
        }
        Ok(out)
    }

    /// Quick entry for variants: save "Paneer Tikka (Half)", "(Full)" …
    /// as items of their own (fresh UUIDs, own prices) grouped under the
    /// base's name, each with its own copy of `photo` when one is given.
    pub fn add_variant_items(
        &self,
        base: &Product,
        variants: &[crate::domain::ModifierOption],
        photo: Option<&Path>,
        actor: &StaffMember,
    ) -> Result<Vec<Product>, String> {
        self.assert_server_writer("Add items")?;
        if variants.is_empty() {
            return Err("Type the variants, e.g. Half=120, Full=200".into());
        }
        let mut made = Product::variant_items(base, variants);
        for item in made.iter_mut() {
            if let Some(src) = photo {
                item.image_key = Some(self.import_item_image(&item.id, src)?);
            }
            self.upsert_product(item)?;
            self.audit(actor, "menu.add", &item.name)?;
        }
        Ok(made)
    }

    pub fn upsert_product(&self, product: &Product) -> Result<(), String> {
        self.assert_server_writer("Change items")?;
        self.write_product_row(product)
    }

    fn write_product_row(&self, product: &Product) -> Result<(), String> {
        // Empty branch on the incoming product means "this device's branch" —
        // keeps legacy call sites and synced catalogues working.
        let branch_id = if product.branch_id.trim().is_empty() {
            self.active_branch_id()?
        } else {
            product.branch_id.clone()
        };
        let modifiers_json = if product.modifiers_json.trim().is_empty() {
            "[]".to_string()
        } else {
            product.modifiers_json.clone()
        };
        self.conn
            .execute(
                "INSERT INTO products(
                    id, name, price_paise, available, archived, image_key,
                    category_id, station_id, tax_bps, stock_qty, modifiers_json, branch_id,
                    tax_rate_id, aliases_json, unit_label, group_name, area_prices_json, kind,
                    schedule_json, allergens
                 )
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name,
                   price_paise=excluded.price_paise,
                   available=excluded.available,
                   archived=excluded.archived,
                   image_key=excluded.image_key,
                   category_id=excluded.category_id,
                   station_id=excluded.station_id,
                   tax_bps=excluded.tax_bps,
                   stock_qty=excluded.stock_qty,
                   modifiers_json=excluded.modifiers_json,
                   branch_id=excluded.branch_id,
                   tax_rate_id=excluded.tax_rate_id,
                   aliases_json=excluded.aliases_json,
                   unit_label=excluded.unit_label,
                   group_name=excluded.group_name,
                   area_prices_json=excluded.area_prices_json,
                   kind=excluded.kind,
                   schedule_json=excluded.schedule_json,
                   allergens=excluded.allergens",
                params![
                    product.id,
                    product.name,
                    product.price.paise(),
                    product.available as i64,
                    product.archived as i64,
                    product.image_key,
                    product.category_id,
                    product.station_id,
                    product.tax_bps,
                    product.stock_qty,
                    modifiers_json,
                    branch_id,
                    product.tax_rate_id,
                    if product.aliases_json.trim().is_empty() {
                        "[]".to_string()
                    } else {
                        product.aliases_json.clone()
                    },
                    product.unit_label,
                    product.group_name.trim(),
                    if product.area_prices_json.trim().is_empty() { "[]" } else { product.area_prices_json.as_str() },
                    if product.kind.trim().is_empty() { crate::domain::ITEM_KIND_DISH } else { product.kind.as_str() },
                    if product.schedule_json.trim().is_empty() { "[]" } else { product.schedule_json.as_str() },
                    product.allergens.trim(),
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn item_images_dir(&self) -> Result<PathBuf, String> {
        let dir = self.path.parent().unwrap_or(Path::new(".")).join("item_images");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    /// Root for entry attachments (notes are DB-only; files land under this tree).
    pub fn attachments_dir(&self) -> Result<PathBuf, String> {
        let dir = self
            .path
            .parent()
            .unwrap_or(Path::new("."))
            .join("attachments");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    fn attachment_entity_dir(
        &self,
        entity: AttachmentEntity,
        entity_id: &str,
    ) -> Result<PathBuf, String> {
        let dir = self
            .attachments_dir()?
            .join(entity.as_str())
            .join(entity_id.trim());
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    /// Import catalogue photo: JPG/PNG/WebP in → always stored as `.webp`.
    pub fn import_item_image(&self, product_id: &str, source: &Path) -> Result<String, String> {
        let dir = self.item_images_dir()?;
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !media::is_image_ext(&ext) {
            return Err("Use jpg, jpeg, png, or webp (stored as WebP)".into());
        }
        let file_name = format!("{product_id}.webp");
        let dest = dir.join(&file_name);
        media::write_image_as_webp_fit(source, &dest, ITEM_PHOTO_EDGE)?;
        Ok(format!("custom:{file_name}"))
    }

    pub fn add_attachment_note(
        &self,
        entity: AttachmentEntity,
        entity_id: &str,
        label: &str,
        body: &str,
        actor: &StaffMember,
    ) -> Result<EntryAttachment, String> {
        self.assert_server_writer("Add note")?;
        let entity_id = entity_id.trim();
        if entity_id.is_empty() {
            return Err("Select an entry first".into());
        }
        let body = body.trim();
        if body.is_empty() {
            return Err("Note text required".into());
        }
        let times = EventTimes::capture(None);
        let id = new_id();
        let label = if label.trim().is_empty() {
            "Note".into()
        } else {
            label.trim().to_string()
        };
        self.conn
            .execute(
                "INSERT INTO entry_attachments(
                    id, entity_kind, entity_id, created_at, user_at, device_at,
                    kind, label, body, stored_name, mime, bytes, actor_id, actor_name
                 ) VALUES(?1,?2,?3,?4,?5,?6,'note',?7,?8,'','text/plain',0,?9,?10)",
                params![
                    id,
                    entity.as_str(),
                    entity_id,
                    times.user_at,
                    times.user_at,
                    times.device_at,
                    label,
                    body,
                    actor.id,
                    actor.name
                ],
            )
            .map_err(|e| e.to_string())?;
        self.get_attachment(&id)?
            .ok_or_else(|| "Note missing after insert".into())
    }

    /// Photo → always WebP on disk.
    pub fn add_attachment_image(
        &self,
        entity: AttachmentEntity,
        entity_id: &str,
        source: &Path,
        label: &str,
        actor: &StaffMember,
    ) -> Result<EntryAttachment, String> {
        self.assert_server_writer("Attach photo")?;
        let entity_id = entity_id.trim();
        if entity_id.is_empty() {
            return Err("Select an entry first".into());
        }
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !media::is_image_ext(&ext) {
            return Err("Use jpg, jpeg, png, or webp".into());
        }
        let id = new_id();
        let stored_name = format!("{id}.webp");
        let dest = self.attachment_entity_dir(entity, entity_id)?.join(&stored_name);
        let bytes = media::write_image_as_webp(source, &dest)? as i64;
        let times = EventTimes::capture(None);
        let label = if label.trim().is_empty() {
            source
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Photo")
                .to_string()
        } else {
            label.trim().to_string()
        };
        self.insert_attachment_row(
            &id,
            entity,
            entity_id,
            &times,
            AttachmentKind::Image,
            &label,
            "",
            &stored_name,
            "image/webp",
            bytes,
            actor,
        )?;
        self.get_attachment(&id)?
            .ok_or_else(|| "Photo missing after insert".into())
    }

    /// PDF / txt copied as-is.
    pub fn add_attachment_file(
        &self,
        entity: AttachmentEntity,
        entity_id: &str,
        source: &Path,
        label: &str,
        actor: &StaffMember,
    ) -> Result<EntryAttachment, String> {
        self.assert_server_writer("Attach file")?;
        let entity_id = entity_id.trim();
        if entity_id.is_empty() {
            return Err("Select an entry first".into());
        }
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !media::is_doc_ext(&ext) {
            return Err("Use pdf or txt".into());
        }
        let id = new_id();
        let stored_name = format!("{id}.{ext}");
        let dest = self.attachment_entity_dir(entity, entity_id)?.join(&stored_name);
        fs::copy(source, &dest).map_err(|e| e.to_string())?;
        let bytes = fs::metadata(&dest).map(|m| m.len() as i64).unwrap_or(0);
        let mime = if ext == "pdf" {
            "application/pdf"
        } else {
            "text/plain"
        };
        let times = EventTimes::capture(None);
        let label = if label.trim().is_empty() {
            source
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Document")
                .to_string()
        } else {
            label.trim().to_string()
        };
        self.insert_attachment_row(
            &id,
            entity,
            entity_id,
            &times,
            AttachmentKind::File,
            &label,
            "",
            &stored_name,
            mime,
            bytes,
            actor,
        )?;
        self.get_attachment(&id)?
            .ok_or_else(|| "File missing after insert".into())
    }

    fn insert_attachment_row(
        &self,
        id: &str,
        entity: AttachmentEntity,
        entity_id: &str,
        times: &EventTimes,
        kind: AttachmentKind,
        label: &str,
        body: &str,
        stored_name: &str,
        mime: &str,
        bytes: i64,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO entry_attachments(
                    id, entity_kind, entity_id, created_at, user_at, device_at,
                    kind, label, body, stored_name, mime, bytes, actor_id, actor_name
                 ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                params![
                    id,
                    entity.as_str(),
                    entity_id,
                    times.user_at,
                    times.user_at,
                    times.device_at,
                    kind.as_str(),
                    label,
                    body,
                    stored_name,
                    mime,
                    bytes,
                    actor.id,
                    actor.name
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_attachment(&self, id: &str) -> Result<Option<EntryAttachment>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, entity_kind, entity_id, created_at, user_at, device_at,
                        kind, label, body, stored_name, mime, bytes, actor_id, actor_name
                 FROM entry_attachments WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let row = stmt
            .query_row(params![id], row_to_attachment)
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(row)
    }

    pub fn list_attachments(
        &self,
        entity: AttachmentEntity,
        entity_id: &str,
    ) -> Result<Vec<EntryAttachment>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, entity_kind, entity_id, created_at, user_at, device_at,
                        kind, label, body, stored_name, mime, bytes, actor_id, actor_name
                 FROM entry_attachments
                 WHERE entity_kind = ?1 AND entity_id = ?2
                 ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![entity.as_str(), entity_id.trim()], row_to_attachment)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn delete_attachment(&self, id: &str) -> Result<(), String> {
        self.assert_server_writer("Delete attachment")?;
        if let Some(att) = self.get_attachment(id)? {
            if !att.stored_name.is_empty() {
                if let Some(entity) = AttachmentEntity::parse(&att.entity_kind) {
                    let path = self
                        .attachment_entity_dir(entity, &att.entity_id)?
                        .join(&att.stored_name);
                    let _ = fs::remove_file(path);
                }
            }
        }
        self.conn
            .execute("DELETE FROM entry_attachments WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn attachment_abs_path(&self, att: &EntryAttachment) -> Result<PathBuf, String> {
        let entity = AttachmentEntity::parse(&att.entity_kind)
            .ok_or_else(|| "Unknown attachment entity".to_string())?;
        if att.stored_name.trim().is_empty() {
            return Err("No file on this attachment".into());
        }
        Ok(self
            .attachment_entity_dir(entity, &att.entity_id)?
            .join(&att.stored_name))
    }

    /// Short lines for receipts / reports.
    pub fn attachments_summary_text(
        &self,
        entity: AttachmentEntity,
        entity_id: &str,
    ) -> Result<String, String> {
        let list = self.list_attachments(entity, entity_id)?;
        if list.is_empty() {
            return Ok(String::new());
        }
        let mut out = String::from("Attachments:\n");
        for a in list {
            match a.kind.as_str() {
                "note" => {
                    let preview: String = a.body.chars().take(80).collect();
                    out.push_str(&format!("  note: {preview}\n"));
                }
                "image" => out.push_str(&format!("  image: {} ({})\n", a.label, a.stored_name)),
                _ => out.push_str(&format!("  file: {} ({})\n", a.label, a.stored_name)),
            }
        }
        Ok(out)
    }

    pub fn finalize_sale(
        &mut self,
        draft: &SaleDraft,
        pay: PayMethod,
        table_id: Option<&str>,
        customer_id: Option<&str>,
        actor: &StaffMember,
    ) -> Result<Sale, String> {
        self.write_sale(draft, pay, table_id, customer_id, actor, &[])
    }

    fn write_sale(
        &mut self,
        draft: &SaleDraft,
        pay: PayMethod,
        table_id: Option<&str>,
        customer_id: Option<&str>,
        actor: &StaffMember,
        parts: &[(PayMethod, i64)],
    ) -> Result<Sale, String> {
        if !self.can_finalize_on_device()? {
            return Err("Client is not paired to a Server — connect first".into());
        }
        if !actor.can_access_sell() {
            return Err("Not allowed to sell".into());
        }
        // Settle is rights-gated (Allowed / WithApproval / Denied). Waiter
        // preset is Denied; owner may allow Take payment on a handheld.
        self.check_right(actor, crate::domain::Right::Settle, None)?;
        // A discount is a right of its own, approved by PIN here on the
        // Server — never trusted from the device that sent it.
        let discount_approver = if draft.discount_paise > 0 {
            if draft.discount_reason.trim().is_empty() {
                return Err("Say why the bill is discounted".into());
            }
            let pin = Some(draft.discount_approval_pin.as_str()).filter(|p| !p.trim().is_empty());
            self.check_right(actor, crate::domain::Right::Discount, pin)?
        } else {
            None
        };
        draft.assert_balanced()?;
        if draft.total_paise == 0 && !self.allow_zero_sales()? {
            return Err("Zero-value sales are turned off in Settings".into());
        }
        let credit_part = parts
            .iter()
            .find(|(m, _)| m.is_credit())
            .map(|(_, a)| *a);
        let credit_amount = credit_part.unwrap_or_else(|| {
            if pay.is_credit() {
                // Taxed total is resolved below; untaxed draft total is enough
                // to know whether any credit is owed for the customer gate.
                draft.total_paise
            } else {
                0
            }
        });
        let needs_customer = credit_amount > 0;
        if needs_customer && customer_id.is_none() {
            return Err("Pick a customer for Credit".into());
        }
        let customer_id = if needs_customer { customer_id } else { None };
        if let Some(cid) = customer_id {
            let exists: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM customers WHERE id = ?1 AND active = 1",
                    params![cid],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if exists == 0 {
                return Err("Customer not found".into());
            }
            // Credit limit gate for khata (credit) bills.
            let credit_amount_estimate = credit_part.unwrap_or_else(|| {
                if pay.is_credit() {
                    draft.total_paise
                } else {
                    0
                }
            });
            if credit_amount_estimate > 0 {
                let cust = self
                    .get_customer(cid)?
                    .ok_or_else(|| "Customer not found".to_string())?;
                // Credit limit gate for account sales (0 limit = no limit set).
                if cust.credit_limit_paise > 0
                    && cust.balance_paise.saturating_add(credit_amount_estimate)
                        > cust.credit_limit_paise
                {
                    return Err(format!(
                        "Credit limit exceeded: outstanding {} / limit {} (paise)",
                        cust.balance_paise, cust.credit_limit_paise
                    ));
                }
            }
        }

        let branch_id = self.active_branch_id()?;
        let stations = self.list_stations()?;
        // The country pack decides whether tax is *extracted* from what the
        // seller quoted or *added* on top of it. Stock and kitchen routing come
        // from the same pass over the catalogue.
        let fiscal_settings = self.fiscal_settings()?;
        let pricing = fiscal_settings.tax_pricing;
        let mut tax_paise = 0i64;
        let mut by_station: Vec<(String, Vec<String>)> = Vec::new();
        let mut stock_hits: Vec<(String, i64)> = Vec::new();
        for line in &draft.lines {
            let product = self.get_product(&line.product_id)?;
            let (tax_bps, station_id, tracked) = product
                .as_ref()
                .map(|p| (p.tax_bps, p.station_id.clone(), p.tracks_stock()))
                .unwrap_or((0, String::new(), false));
            tax_paise = tax_paise
                .saturating_add(fiscal::tax_on_amount(line.line_total_paise, tax_bps, pricing));
            // Lines the kitchen already has (sent on the table's running
            // order, or a counter KOT before payment) are not fired again.
            if !line.kot_sent {
                let label = kot_line_label(line);
                match by_station.iter_mut().find(|(sid, _)| *sid == station_id) {
                    Some((_, items)) => items.push(label),
                    None => by_station.push((station_id, vec![label])),
                }
            }
            if tracked && line.qty > 0 {
                // Aggregate per item: two lines of the same product still have
                // to fit inside one on-hand count.
                match stock_hits.iter_mut().find(|(pid, _)| *pid == line.product_id) {
                    Some((_, qty)) => *qty = qty.saturating_add(line.qty),
                    None => stock_hits.push((line.product_id.clone(), line.qty)),
                }
            }
        }
        // Refuse the oversell before anything is written. The decrement used to
        // clamp at zero, which quietly sold stock the shop did not have and
        // left the count lying about it.
        for (product_id, qty) in &stock_hits {
            let product = self
                .get_product(product_id)?
                .ok_or_else(|| "Product not found".to_string())?;
            if product.stock_qty < *qty {
                return Err(format!(
                    "Not enough stock for {}: {} on hand, {qty} on the bill",
                    product.name, product.stock_qty
                ));
            }
        }
        // Inclusive: the charged total is exactly what the cart quoted.
        // Exclusive: tax lands on top, and that higher figure is what the
        // customer pays, what the credit ledger records, and what the day
        // totals count.
        let service_paise = fiscal_settings.service_on(draft.lines_total_paise);
        let total_paise = fiscal::gross_with_tax(draft.total_paise, tax_paise, pricing)
            .saturating_add(service_paise);
        let net_paise = fiscal::net_of_tax(draft.total_paise, tax_paise, pricing);
        let tax_label = fiscal_settings.tax_label().to_string();

        let sale_id = new_id();
        let branch_code = self.active_branch()?.code;
        let times = EventTimes::capture(draft.user_at.as_deref());
        let created_at = times.user_at.clone(); // books day = user time
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;

        // Reserved inside the sale's transaction so a crash can never hand the
        // same number to two bills.
        let invoice_no = reserve_invoice_no(&tx, &branch_code)?;
        let extras_json = if draft.extras_json.trim().is_empty() {
            "[]".into()
        } else {
            draft.extras_json.clone()
        };
        let created_by_name = actor.lock_label();
        tx.execute(
            "INSERT INTO sales(
                id, created_at, total_paise, lines_total_paise, rounding_paise,
                pay_method, status, table_id, customer_id, created_by_id, created_by_name,
                branch_id, order_type, tax_paise, tax_pricing, tax_label, net_paise,
                service_paise, invoice_no, money_lane, user_at, device_at, server_at,
                extras_json
             )
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'completed', ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
            params![
                sale_id,
                created_at,
                total_paise,
                draft.lines_total_paise,
                draft.rounding_paise,
                pay.as_str(),
                table_id,
                customer_id,
                actor.id,
                created_by_name,
                branch_id,
                draft.order_type.as_str(),
                tax_paise,
                pricing.as_str(),
                tax_label,
                net_paise,
                service_paise,
                invoice_no,
                draft.money_lane.as_str(),
                times.user_at,
                times.device_at,
                times.server_at,
                extras_json
            ],
        )
        .map_err(|e| e.to_string())?;
        if draft.discount_paise > 0 {
            let reason = match &discount_approver {
                Some(a) => format!("{} (approved by {})", draft.discount_reason.trim(), a.name),
                None => draft.discount_reason.trim().to_string(),
            };
            tx.execute(
                "UPDATE sales SET discount_paise = ?2, discount_reason = ?3 WHERE id = ?1",
                params![sale_id, draft.discount_paise, reason],
            )
            .map_err(|e| e.to_string())?;
        }

        if !draft.notify_phone.trim().is_empty() {
            let row = crate::notify::draft_order_alert(
                draft.notify_phone.trim(),
                &invoice_no,
                &format!("Your bill {invoice_no} is recorded. Thank you!"),
            );
            tx.execute(
                "INSERT INTO notify_outbox(
                    id, created_at, channel, to_addr, template_key, payload_json, status, last_error
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, '')",
                params![
                    row.id,
                    row.created_at,
                    row.channel,
                    row.to_addr,
                    row.template_key,
                    row.payload_json,
                    row.status
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        let tenders: Vec<(&str, i64)> = if parts.is_empty() {
            vec![(pay.as_str(), total_paise)]
        } else {
            let sum: i64 = parts.iter().map(|(_, amount)| *amount).sum();
            if sum != total_paise {
                return Err(format!(
                    "Split parts ({sum}) must equal bill ({total_paise})"
                ));
            }
            parts
                .iter()
                .map(|(method, amount)| (method.as_str(), *amount))
                .collect()
        };
        for (method, amount) in tenders {
            tx.execute(
                "INSERT INTO sale_payments(id, sale_id, pay_method, amount_paise)
                 VALUES(?1, ?2, ?3, ?4)",
                params![new_id(), sale_id, method, amount],
            )
            .map_err(|e| e.to_string())?;
        }

        for line in &draft.lines {
            tx.execute(
                "INSERT INTO sale_lines(sale_id, product_id, name, unit_price_paise, qty, line_total_paise, note, qty_milli, unit_label, modifiers_json)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    sale_id,
                    line.product_id,
                    line.name,
                    line.unit_price_paise,
                    line.qty,
                    line.line_total_paise,
                    line.note,
                    if line.qty_milli > 0 { line.qty_milli } else { line.qty.saturating_mul(1000) },
                    line.unit_label,
                    if line.modifiers_json.trim().is_empty() {
                        "[]".into()
                    } else {
                        line.modifiers_json.clone()
                    },
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        let table_prefix = match table_id {
            Some(tid) => tx
                .query_row(
                    "SELECT name FROM tables_dining WHERE id = ?1",
                    params![tid],
                    |r| r.get::<_, String>(0),
                )
                .optional()
                .map_err(|e| e.to_string())?
                .map(|n| format!("{n} · "))
                .unwrap_or_default(),
            None => String::new(),
        };

        // Tracked items only: untracked (−1) stock stays untracked. The
        // `stock_qty >= ?1` guard makes the check above race-proof — a second
        // till that took the last unit first leaves 0 rows updated here.
        for (product_id, qty) in &stock_hits {
            let moved = tx
                .execute(
                    "UPDATE products SET stock_qty = stock_qty - ?1
                     WHERE id = ?2 AND stock_qty >= ?1",
                    params![qty, product_id],
                )
                .map_err(|e| e.to_string())?;
            if moved == 0 {
                return Err(format!("Stock ran out while charging {product_id}"));
            }
            tx.execute(
                "INSERT INTO stock_movements(
                    id, branch_id, product_id, created_at, delta_qty,
                    reason, note, actor_id, actor_name, ref_id,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, 'sale', '', ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    new_id(),
                    branch_id,
                    product_id,
                    created_at,
                    -qty,
                    actor.id,
                    actor.name,
                    sale_id,
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        if !draft.delivery_by.trim().is_empty() {
            tx.execute(
                "INSERT INTO sale_delivery(sale_id, delivery_by) VALUES(?1, ?2)",
                params![sale_id, draft.delivery_by.trim()],
            )
            .map_err(|e| e.to_string())?;
        }
        // What each dish used from the shelf (its recipe).
        let portions: Vec<(String, i64)> =
            draft.lines.iter().map(|l| (l.product_id.clone(), l.qty)).collect();
        recipes::apply_recipe_usage(&tx, &portions, "recipe", &sale_id, &branch_id, actor, &created_at)?;

        // Counter mode: skip kitchen tickets so takeaway cash flow stays quiet.
        let venue_mode = VenueMode::parse(
            &tx.query_row(
                "SELECT value FROM meta WHERE key = 'venue_mode'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "restaurant".into()),
        );

        if let Some(tid) = table_id {
            // Close kitchen tickets already sent this visit (and leftovers)
            // *before* any new finalize KOTs, so pay-then-cook tickets stay open
            // and the next seating is not stuck "In kitchen".
            tx.execute(
                "UPDATE kot_tickets SET status = ?1
                  WHERE table_id = ?2 AND status NOT IN ('completed', 'cancelled')",
                params![KotStatus::Completed.as_str(), tid],
            )
            .map_err(|e| e.to_string())?;
        }

        if venue_mode.creates_kot() {
            // One ticket per station on the bill so a big kitchen does not have
            // to read the whole order at every counter. Items with no station
            // land on a single unassigned ticket. Only lines not already sent
            // as a KOT (`kot_sent`) are fired here.
            for (station_id, items) in &by_station {
                let station_name = stations
                    .iter()
                    .find(|s| s.id == *station_id)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let head = if station_name.is_empty() {
                    String::new()
                } else {
                    format!("[{station_name}] ")
                };
                tx.execute(
                    "INSERT INTO kot_tickets(
                        id, sale_id, created_at, status, summary, branch_id, station_id,
                        table_id, total_paise
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        new_id(),
                        sale_id,
                        created_at,
                        KotStatus::New.as_str(),
                        format!("{table_prefix}{head}{}", items.join(", ")),
                        branch_id,
                        station_id,
                        // Never tie a finalize-time KOT to a table id: the
                        // table is freed on pay, and a leftover open ticket
                        // stuck the next seating on "In kitchen". Counter
                        // tickets (no table) already used an empty id.
                        "",
                        0
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        if let Some(tid) = table_id {
            // Payment recorded → table is Free for the next guests (no
            // stuck "Paid / Ready" step — that confused every service test).
            tx.execute(
                "UPDATE tables_dining
                    SET status = ?1, active_sale_id = NULL, occupied_at = '', paid_at = ''
                  WHERE id = ?2",
                params![TableStatus::Available.as_str(), tid],
            )
            .map_err(|e| e.to_string())?;
            Store::settle_running_order_tx(&tx, tid, &sale_id)?;
            tx.execute(
                "INSERT INTO meta(key, value) VALUES(?1, '')
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![format!("table_draft_{tid}")],
            )
            .map_err(|e| e.to_string())?;
        }

        // Full Credit bill, or Credit portion of a mixed bill.
        let credit_amount = credit_part.unwrap_or_else(|| {
            if pay.is_credit() {
                total_paise
            } else {
                0
            }
        });
        if credit_amount > 0
            && let Some(cid) = customer_id {
                // Prepaid covers the credit portion first; remainder is new udhaar.
                let bal: i64 = tx
                    .query_row(
                        "SELECT COALESCE(SUM(amount_paise), 0) FROM credit_ledger WHERE customer_id = ?1",
                        params![cid],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                let prepaid = (-bal).max(0);
                let applied = credit_amount.min(prepaid);
                let on_credit = credit_amount - applied;
                if applied > 0 {
                    tx.execute(
                        "INSERT INTO credit_ledger(
                            id, customer_id, created_at, kind, amount_paise,
                            sale_id, pay_method, actor_id, actor_name, detail,
                            user_at, device_at, server_at
                         ) VALUES(?1, ?2, ?3, 'apply', ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            new_id(),
                            cid,
                            created_at,
                            applied,
                            sale_id,
                            actor.id,
                            actor.name,
                            format!("Applied advance on sale {sale_id}"),
                            times.user_at,
                            times.device_at,
                            times.server_at
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
                if on_credit > 0 {
                    tx.execute(
                        "INSERT INTO credit_ledger(
                            id, customer_id, created_at, kind, amount_paise,
                            sale_id, pay_method, actor_id, actor_name, detail,
                            user_at, device_at, server_at
                         ) VALUES(?1, ?2, ?3, 'sale', ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            new_id(),
                            cid,
                            created_at,
                            on_credit,
                            sale_id,
                            actor.id,
                            actor.name,
                            format!("Credit on sale {sale_id}"),
                            times.user_at,
                            times.device_at,
                            times.server_at
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }

        let audit_detail = if draft.rounding_paise != 0 {
            format!(
                "{sale_id} charged={total_paise} lines={} round={} {} cust={}",
                draft.lines_total_paise,
                draft.rounding_paise,
                pay.as_str(),
                customer_id.unwrap_or("-")
            )
        } else {
            format!(
                "{sale_id} {total_paise} {} cust={}",
                pay.as_str(),
                customer_id.unwrap_or("-")
            )
        };
        tx.execute(
            "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
             VALUES(?1, ?2, ?3, ?4, 'sale.finalize', ?5)",
            params![
                new_id(),
                created_at,
                actor.id,
                actor.name,
                audit_detail
            ],
        )
        .map_err(|e| e.to_string())?;

        let sale = Sale {
            id: sale_id.clone(),
            invoice_no,
            created_at: created_at.clone(),
            user_at: times.user_at.clone(),
            device_at: times.device_at.clone(),
            server_at: times.server_at.clone(),
            lines_total_paise: draft.lines_total_paise,
            rounding_paise: draft.rounding_paise,
            total_paise,
            pay_method: pay.as_str().into(),
            status: "completed".into(),
            table_id: table_id.map(|s| s.to_string()),
            customer_id: customer_id.map(|s| s.to_string()),
            created_by_id: actor.id.clone(),
            created_by_name,
            branch_id: branch_id.clone(),
            order_type: draft.order_type.as_str().into(),
            tax_paise,
            tax_pricing: pricing.as_str().into(),
            tax_label,
            net_paise,
            service_paise,
            money_lane: draft.money_lane.as_str().into(),
            extras_json,
            lines: draft.lines.clone(),
        };

        // Durable queue for a future cloud push — committed with the sale so a
        // crash can never lose the event while keeping the sale.
        tx.execute(
            "INSERT INTO sync_outbox(
                id, branch_id, created_at, event_type, payload_json, synced_at
             ) VALUES(?1, ?2, ?3, 'sale.finalize', ?4, '')",
            params![
                new_id(),
                branch_id,
                created_at,
                serde_json::to_string(&sale).map_err(|e| e.to_string())?
            ],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        // Local loyalty: 1 point per ₹100 paid when a customer is on the bill.
        if let Some(cid) = customer_id {
            let _ = self.earn_loyalty_on_sale(cid, total_paise, actor);
        }

        Ok(sale)
    }

    pub fn get_sale(&self, sale_id: &str) -> Result<Option<Sale>, String> {
        let mut sale = self
            .conn
            .query_row(
                &format!("SELECT {SALE_COLUMNS} FROM sales WHERE id = ?1"),
                params![sale_id],
                row_to_sale,
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(ref mut s) = sale {
            s.lines = self.sale_lines(&s.id)?;
        }
        Ok(sale)
    }

    /// Replace the live sale with a new draft. Previous live state is appended
    /// to `sale_revisions` (who / when / reason / full previous snapshot).
    pub fn amend_sale(
        &mut self,
        sale_id: &str,
        draft: &SaleDraft,
        pay: PayMethod,
        reason: &str,
        actor: &StaffMember,
    ) -> Result<Sale, String> {
        // A Client's sales table is a sync cache, so amending it locally would
        // be overwritten by the next snapshot. Terminals go through `/amend`.
        self.assert_server_writer("Amend sales")?;
        if !actor.role.can_amend_sale() {
            return Err("Not allowed to amend sales".into());
        }
        let reason = reason.trim();
        if reason.is_empty() {
            return Err("Amend reason required".into());
        }
        draft.assert_balanced()?;
        if draft.total_paise == 0 && !self.allow_zero_sales()? {
            return Err("Zero-value sales are turned off in Settings".into());
        }

        let previous = self
            .get_sale(sale_id)?
            .ok_or_else(|| "Sale not found".to_string())?;
        if previous.status != "completed" {
            return Err("Only completed sales can be amended".into());
        }

        // Re-price the amended bill through the same country pack the till
        // uses, so an exclusive-tax shop's corrected total still carries tax.
        let (tax_paise, total_paise, fiscal_settings) = self.resolve_draft_tax(draft)?;
        let pricing = fiscal_settings.tax_pricing;
        let service_paise = fiscal_settings.service_on(draft.lines_total_paise);
        let net_paise = fiscal::net_of_tax(draft.total_paise, tax_paise, pricing);
        let tax_label = fiscal_settings.tax_label().to_string();

        let previous_json =
            serde_json::to_string(&previous).map_err(|e| e.to_string())?;
        let revised_at = Local::now().to_rfc3339();
        let revision_id = new_id();

        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO sale_revisions(
                id, sale_id, revised_at, actor_id, actor_name, reason, previous_json
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                revision_id,
                sale_id,
                revised_at,
                actor.id,
                actor.name,
                reason,
                previous_json
            ],
        )
        .map_err(|e| e.to_string())?;

        let updated = tx
            .execute(
                "UPDATE sales SET
                    total_paise = ?1,
                    lines_total_paise = ?2,
                    rounding_paise = ?3,
                    pay_method = ?4,
                    tax_paise = ?5,
                    tax_pricing = ?6,
                    tax_label = ?7,
                    net_paise = ?8,
                    service_paise = ?9
                 WHERE id = ?10 AND status = 'completed'",
                params![
                    total_paise,
                    draft.lines_total_paise,
                    draft.rounding_paise,
                    pay.as_str(),
                    tax_paise,
                    pricing.as_str(),
                    tax_label,
                    net_paise,
                    service_paise,
                    sale_id
                ],
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            return Err("Sale not found or not amendable".into());
        }

        // Correcting a bill corrects the shelf too: a line dropped from 3 to 1
        // puts two back, a line raised takes more out. Refused outright when
        // the correction would book stock the shop does not have.
        apply_amend_stock(&tx, sale_id, &previous, draft, actor, &revised_at)?;

        tx.execute("DELETE FROM sale_lines WHERE sale_id = ?1", params![sale_id])
            .map_err(|e| e.to_string())?;
        for line in &draft.lines {
            tx.execute(
                "INSERT INTO sale_lines(sale_id, product_id, name, unit_price_paise, qty, line_total_paise, note, qty_milli, unit_label, modifiers_json)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    sale_id,
                    line.product_id,
                    line.name,
                    line.unit_price_paise,
                    line.qty,
                    line.line_total_paise,
                    line.note,
                    if line.qty_milli > 0 { line.qty_milli } else { line.qty.saturating_mul(1000) },
                    line.unit_label,
                    if line.modifiers_json.trim().is_empty() {
                        "[]".into()
                    } else {
                        line.modifiers_json.clone()
                    },
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        // Keep credit ledger in sync when amending a credit sale's charged total.
        if previous.pay_method == "credit" {
            if pay.is_credit() {
                tx.execute(
                    "UPDATE credit_ledger SET amount_paise = ?1
                     WHERE sale_id = ?2 AND kind = 'sale'",
                    params![total_paise, sale_id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                tx.execute(
                    "DELETE FROM credit_ledger WHERE sale_id = ?1 AND kind = 'sale'",
                    params![sale_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        tx.execute(
            "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
             VALUES(?1, ?2, ?3, ?4, 'sale.amend', ?5)",
            params![
                new_id(),
                revised_at,
                actor.id,
                actor.name,
                format!(
                    "{sale_id}: {reason} · was {} now {total_paise} · rev {revision_id}",
                    previous.total_paise
                )
            ],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        Ok(Sale {
            id: sale_id.to_string(),
            // Amending corrects a bill, it does not issue a new one.
            invoice_no: previous.invoice_no,
            created_at: previous.created_at,
            user_at: previous.user_at,
            device_at: previous.device_at,
            server_at: previous.server_at,
            lines_total_paise: draft.lines_total_paise,
            rounding_paise: draft.rounding_paise,
            total_paise,
            pay_method: pay.as_str().into(),
            status: "completed".into(),
            table_id: previous.table_id,
            customer_id: previous.customer_id,
            // Authorship is immutable — amend never rewrites who created the sale.
            created_by_id: previous.created_by_id,
            created_by_name: previous.created_by_name,
            branch_id: previous.branch_id,
            order_type: previous.order_type,
            tax_paise,
            tax_pricing: pricing.as_str().into(),
            tax_label,
            net_paise,
            service_paise,
            money_lane: draft.money_lane.as_str().into(),
            extras_json: previous.extras_json,
            lines: draft.lines.clone(),
        })
    }

    pub fn list_sale_revisions(&self, sale_id: &str) -> Result<Vec<SaleRevision>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, sale_id, revised_at, actor_id, actor_name, reason, previous_json
                 FROM sale_revisions
                 WHERE sale_id = ?1
                 ORDER BY revised_at DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![sale_id], |row| {
                let previous_json: String = row.get(6)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    previous_json,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let (id, sale_id, revised_at, actor_id, actor_name, reason, previous_json) =
                row.map_err(|e| e.to_string())?;
            let previous: Sale =
                serde_json::from_str(&previous_json).map_err(|e| e.to_string())?;
            out.push(SaleRevision {
                id,
                sale_id,
                revised_at,
                actor_id,
                actor_name,
                reason,
                previous,
            });
        }
        Ok(out)
    }

    pub fn void_sale(
        &mut self,
        sale_id: &str,
        reason: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        if !actor.role.can_void_refund() {
            return Err("Not allowed to void".into());
        }
        let reason = reason.trim();
        if reason.is_empty() {
            return Err("Void reason required".into());
        }
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        let updated = tx
            .execute(
                "UPDATE sales SET status = 'voided' WHERE id = ?1 AND status = 'completed'",
                params![sale_id],
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            return Err("Sale not found or already voided".into());
        }
        let now = Local::now().to_rfc3339();
        // Reverse credit sale debt and any prepaid that was applied.
        {
            let mut stmt = tx
                .prepare(
                    "SELECT customer_id, kind, amount_paise FROM credit_ledger
                     WHERE sale_id = ?1 AND kind IN ('sale', 'apply')",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![sale_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            drop(stmt);
            for (cid, kind, amount) in rows {
                tx.execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail
                     ) VALUES(?1, ?2, ?3, 'void', ?4, ?5, NULL, ?6, ?7, ?8)",
                    params![
                        new_id(),
                        cid,
                        now,
                        -amount,
                        sale_id,
                        actor.id,
                        actor.name,
                        format!("Void {kind} {sale_id}: {reason}")
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        restore_stock_for_sale(&tx, sale_id, "void", actor, &now)?;
        tx.execute(
            "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
             VALUES(?1, ?2, ?3, ?4, 'sale.void', ?5)",
            params![
                new_id(),
                now,
                actor.id,
                actor.name,
                format!("{sale_id}: {reason}")
            ],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Post-charge snackbar undo: void a just-recorded sale for the seller
    /// who charged it (or Owner/Manager). Fixed reason; no manual reason UI.
    pub fn quick_undo_sale(
        &mut self,
        sale_id: &str,
        actor: &StaffMember,
    ) -> Result<Sale, String> {
        if !actor.role.can_sell() {
            return Err("Not allowed to undo".into());
        }
        let sale = self
            .get_sale(sale_id)?
            .ok_or_else(|| "Sale not found".to_string())?;
        if sale.status != "completed" {
            return Err("Sale already voided".into());
        }
        if sale.created_by_id != actor.id && !actor.role.can_void_refund() {
            return Err("Only the seller can undo this sale".into());
        }
        // Reuse void path with a fixed reason; temporarily allow via owner-level
        // by calling the same SQL as void_sale without the manager gate.
        let reason = "Quick undo";
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        let updated = tx
            .execute(
                "UPDATE sales SET status = 'voided' WHERE id = ?1 AND status = 'completed'",
                params![sale_id],
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            return Err("Sale not found or already voided".into());
        }
        let now = Local::now().to_rfc3339();
        {
            let mut stmt = tx
                .prepare(
                    "SELECT customer_id, kind, amount_paise FROM credit_ledger
                     WHERE sale_id = ?1 AND kind IN ('sale', 'apply')",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![sale_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            drop(stmt);
            for (cid, kind, amount) in rows {
                tx.execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail
                     ) VALUES(?1, ?2, ?3, 'void', ?4, ?5, NULL, ?6, ?7, ?8)",
                    params![
                        new_id(),
                        cid,
                        now,
                        -amount,
                        sale_id,
                        actor.id,
                        actor.name,
                        format!("Quick undo {kind} {sale_id}")
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        // Drop open kitchen tickets for this mistaken charge.
        tx.execute(
            "UPDATE kot_tickets SET status = 'cancelled'
             WHERE sale_id = ?1 AND status IN ('new', 'preparing', 'ready')",
            params![sale_id],
        )
        .map_err(|e| e.to_string())?;
        restore_stock_for_sale(&tx, sale_id, "undo", actor, &now)?;
        tx.execute(
            "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
             VALUES(?1, ?2, ?3, ?4, 'sale.quick_undo', ?5)",
            params![
                new_id(),
                now,
                actor.id,
                actor.name,
                format!("{sale_id}: {reason}")
            ],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(sale)
    }

    pub fn refund_sale(
        &mut self,
        sale_id: &str,
        reason: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        if !actor.role.can_void_refund() {
            return Err("Not allowed to refund".into());
        }
        let reason = reason.trim();
        if reason.is_empty() {
            return Err("Refund reason required".into());
        }
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        let updated = tx
            .execute(
                "UPDATE sales SET status = 'refunded' WHERE id = ?1 AND status = 'completed'",
                params![sale_id],
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            return Err("Sale not found or not refundable".into());
        }
        let now = Local::now().to_rfc3339();
        restore_stock_for_sale(&tx, sale_id, "refund", actor, &now)?;
        tx.execute(
            "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
             VALUES(?1, ?2, ?3, ?4, 'sale.refund', ?5)",
            params![
                new_id(),
                now,
                actor.id,
                actor.name,
                format!("{sale_id}: {reason}")
            ],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn today_totals(&self) -> Result<DayTotals, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let prefix = format!("{today}%");
        let branch = self.active_branch_id()?;
        let mut totals = DayTotals::default();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT pay_method, total_paise,
                        COALESCE(lines_total_paise, total_paise),
                        COALESCE(rounding_paise, 0),
                        COALESCE(tax_paise, 0),
                        COALESCE(money_lane, 'declared')
                 FROM sales
                 WHERE status = 'completed' AND created_at LIKE ?1
                   AND (branch_id = ?2 OR branch_id = '')",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![prefix, branch], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (method, charged, lines_total, rounding, tax, lane) =
                row.map_err(|e| e.to_string())?;
            totals.sale_count += 1;
            totals.revenue_paise += charged;
            totals.lines_total_paise += lines_total;
            totals.rounding_paise += rounding;
            totals.tax_paise += tax;
            match MoneyLane::parse(&lane) {
                MoneyLane::Owner => totals.owner_lane_paise += charged,
                MoneyLane::Declared => totals.declared_paise += charged,
            }
            match PayMethod::parse(&method) {
                Some(PayMethod::Cash) => totals.cash_paise += charged,
                Some(PayMethod::Upi) => totals.upi_paise += charged,
                Some(PayMethod::Card) => totals.card_paise += charged,
                // Bank-tender sales are bank money, like card — counted
                // with card totals in the day view (no separate field).
                Some(PayMethod::Bank) => totals.card_paise += charged,
                Some(PayMethod::Credit) => totals.credit_paise += charged,
                Some(PayMethod::Split) | None => {}
            }
        }
        // Split tenders live in sale_payments (sales.pay_method is just "split").
        {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT p.pay_method, p.amount_paise
                     FROM sale_payments p
                     JOIN sales s ON s.id = p.sale_id
                     WHERE s.status = 'completed' AND s.created_at LIKE ?1
                       AND (s.branch_id = ?2 OR s.branch_id = '')
                       AND s.pay_method = 'split'",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![prefix, branch], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                let (method, amount) = row.map_err(|e| e.to_string())?;
                match PayMethod::parse(&method) {
                    Some(PayMethod::Cash) => totals.cash_paise += amount,
                    Some(PayMethod::Upi) => totals.upi_paise += amount,
                    Some(PayMethod::Card) => totals.card_paise += amount,
                    Some(PayMethod::Credit) => totals.credit_paise += amount,
                    _ => {}
                }
            }
        }
        // Settlements and advances are money in today, not sales revenue.
        {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT pay_method, ABS(amount_paise)
                     FROM credit_ledger
                     WHERE kind IN ('settle', 'advance')
                       AND created_at LIKE ?1
                       AND amount_paise < 0
                       AND pay_method IS NOT NULL",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![prefix], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                let (method, amount) = row.map_err(|e| e.to_string())?;
                match PayMethod::parse(&method) {
                    Some(PayMethod::Cash) => totals.cash_paise += amount,
                    Some(PayMethod::Upi) => totals.upi_paise += amount,
                    Some(PayMethod::Card) => totals.card_paise += amount,
                    _ => {}
                }
            }
        }
        Ok(totals)
    }

    /// Cash that physically left the till in a period (`None` = all
    /// time): cash parts of expenses, cash payable settlements, and
    /// till → bank moves. Sales-side cash in is in [`Self::today_totals`]
    /// / the till balance below.
    fn cash_out_paise(&self, day_prefix: Option<&str>) -> Result<i64, String> {
        let like = match day_prefix {
            Some(d) => format!("{d}%"),
            None => "%".to_string(),
        };
        let branch = self.active_branch_id()?;
        let one = |sql: &str| -> Result<i64, String> {
            self.conn
                .query_row(sql, params![like, branch], |r| r.get::<_, i64>(0))
                .map_err(|e| e.to_string())
        };
        let expenses = one(
            "SELECT COALESCE(SUM(p.amount_paise), 0) FROM expense_payments p
             JOIN expenses e ON e.id = p.expense_id
             WHERE p.pay_method = 'cash' AND e.created_at LIKE ?1
               AND (e.branch_id = ?2 OR e.branch_id = '')",
        )?;
        // payable_ledger carries no branch column (party-level books).
        let payable: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM payable_ledger
                 WHERE kind = 'settle' AND pay_method = 'cash' AND amount_paise < 0
                   AND created_at LIKE ?1",
                params![like],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        let to_bank = one(
            "SELECT COALESCE(SUM(amount_paise), 0) FROM money_movements
             WHERE kind = 'till_to_bank' AND created_at LIKE ?1
               AND (branch_id = ?2 OR branch_id = '')",
        )?;
        Ok(expenses + payable + to_bank)
    }

    /// Cash physically in the till right now, from every recorded cash
    /// movement since the books began (perpetual till — vision §18):
    /// cash sales + split cash parts + cash settlements in, minus cash
    /// expenses, cash payable settlements, and till → bank moves, plus
    /// bank → till moves. Only as true as the entries: that is the
    /// point — cash that moved must be recorded to be counted.
    pub fn till_balance_paise(&self) -> Result<i64, String> {
        let branch = self.active_branch_id()?;
        let one = |sql: &str| -> Result<i64, String> {
            self.conn
                .query_row(sql, params![branch], |r| r.get::<_, i64>(0))
                .map_err(|e| e.to_string())
        };
        let cash_sales = one(
            "SELECT COALESCE(SUM(total_paise), 0) FROM sales
             WHERE status = 'completed' AND pay_method = 'cash'
               AND (branch_id = ?1 OR branch_id = '')",
        )?;
        let split_cash = one(
            "SELECT COALESCE(SUM(p.amount_paise), 0) FROM sale_payments p
             JOIN sales s ON s.id = p.sale_id
             WHERE s.status = 'completed' AND s.pay_method = 'split'
               AND p.pay_method = 'cash' AND (s.branch_id = ?1 OR s.branch_id = '')",
        )?;
        // credit_ledger carries no branch column (party-level books).
        let settles_in: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM credit_ledger
                 WHERE kind IN ('settle', 'advance') AND amount_paise < 0
                   AND pay_method = 'cash'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        let from_bank = one(
            "SELECT COALESCE(SUM(amount_paise), 0) FROM money_movements
             WHERE kind = 'bank_to_till' AND (branch_id = ?1 OR branch_id = '')",
        )?;
        let out = self.cash_out_paise(None)?;
        Ok(cash_sales + split_cash + settles_in + from_bank - out)
    }

    pub fn recent_sales(&self, limit: i64) -> Result<Vec<Sale>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {SALE_COLUMNS} FROM sales
                 WHERE branch_id = ?1 OR branch_id = ''
                 ORDER BY created_at DESC LIMIT ?2"
            ))
            .map_err(|e| e.to_string())?;
        let sales = stmt
            .query_map(params![branch, limit], row_to_sale)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(sales)
    }

    pub fn sale_lines(&self, sale_id: &str) -> Result<Vec<SaleLine>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT product_id, name, unit_price_paise, qty, line_total_paise,
                        COALESCE(note, ''), COALESCE(qty_milli, 0), COALESCE(unit_label, ''),
                        COALESCE(modifiers_json, '[]')
                 FROM sale_lines WHERE sale_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![sale_id], |row| {
                let qty: i64 = row.get(3)?;
                let mut qty_milli: i64 = row.get(6)?;
                if qty_milli <= 0 {
                    qty_milli = qty.saturating_mul(1000);
                }
                Ok(SaleLine {
                    product_id: row.get(0)?,
                    name: row.get(1)?,
                    unit_price_paise: row.get(2)?,
                    qty,
                    line_total_paise: row.get(4)?,
                    note: row.get(5)?,
                    qty_milli,
                    unit_label: row.get(7)?,
                    kot_sent: false,
                    modifiers_json: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_staff(&self) -> Result<Vec<StaffMember>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, role, pin_hash, active, role_title,
                        COALESCE(features_json, ''), COALESCE(login_id, ''), COALESCE(password_hash, '')
                 FROM staff WHERE active = 1 ORDER BY
                 CASE role WHEN 'owner' THEN 0 ELSE 1 END, name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let role_s: String = row.get(2)?;
                let role = StaffRole::parse(&role_s).unwrap_or(StaffRole::Cashier);
                let mut role_title: String = row.get(5)?;
                if role_title.trim().is_empty() {
                    role_title = role.default_title().to_string();
                }
                let features_json: String = row.get(6)?;
                let features = if features_json.trim().is_empty() {
                    StaffFeatures::for_role(role)
                } else {
                    StaffFeatures::from_json(&features_json)
                };
                Ok(StaffMember {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    role_title,
                    role,
                    pin_hash: row.get(3)?,
                    active: row.get::<_, i64>(4)? != 0,
                    login_id: row.get(7)?,
                    password_hash: row.get(8)?,
                    features,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_non_owner_staff(&self) -> Result<Vec<StaffMember>, String> {
        Ok(self
            .list_staff()?
            .into_iter()
            .filter(|s| !s.is_owner())
            .collect())
    }

    /// Unlock as Owner — PIN must match an active owner account.
    // ------------------------------------------------------------ LAN: Server

    /// Pair a phone: a fresh token (only its hash is kept here).
    pub fn pair_lan_device(&self, name: &str) -> Result<(String, String), String> {
        let id = new_id();
        let token = random_token()?;
        let name = name.trim();
        let name = if name.is_empty() { "Phone" } else { name };
        self.conn
            .execute(
                "INSERT INTO lan_devices(id, name, token_hash, paired_at, last_seen_at)
                 VALUES(?1, ?2, ?3, ?4, ?4)",
                params![id, name, lan_token_hash(&token), Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        // Who joined the shop and when, in the books' own history.
        self.audit_as(&id, name, "lan.pair", "paired with the shop's code")?;
        Ok((id, token))
    }

    /// The paired, not forgotten phone holding `token` (and note it was seen).
    pub fn lan_device_for_token(&self, token: &str) -> Result<Option<LanDevice>, String> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        let device = self
            .conn
            .query_row(
                "SELECT id, name, paired_at, last_seen_at FROM lan_devices
                  WHERE token_hash = ?1 AND revoked_at = ''",
                params![lan_token_hash(token)],
                LanDevice::from_row,
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(d) = &device {
            // Noted at most every half minute: a device asks every few
            // seconds, and each note is a disk write.
            let now = Local::now();
            let _ = self.conn.execute(
                "UPDATE lan_devices SET last_seen_at = ?2 WHERE id = ?1 AND last_seen_at < ?3",
                params![d.id, now.to_rfc3339(), (now - chrono::Duration::seconds(LAN_SEEN_EVERY_SECS)).to_rfc3339()],
            );
        }
        Ok(device)
    }

    /// Phones paired to this Server, most recently seen first.
    pub fn list_lan_devices(&self) -> Result<Vec<LanDevice>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, paired_at, last_seen_at FROM lan_devices
                  WHERE revoked_at = '' ORDER BY last_seen_at DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], LanDevice::from_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    /// Forget a phone (lost, stolen, staff left): its token and sign-ins
    /// stop working at once; it must pair again with the current code.
    pub fn forget_lan_device(&self, device_id: &str, actor: &StaffMember) -> Result<(), String> {
        let name: String = self
            .conn
            .query_row("SELECT name FROM lan_devices WHERE id = ?1", params![device_id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        self.conn
            .execute(
                "UPDATE lan_devices SET revoked_at = ?2 WHERE id = ?1",
                params![device_id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM lan_sessions WHERE device_id = ?1", params![device_id])
            .map_err(|e| e.to_string())?;
        self.audit(actor, "lan.forget", &name)
    }

    /// A new pairing code; phones already paired keep working.
    pub fn new_pairing_code(&self, actor: &StaffMember) -> Result<String, String> {
        self.meta_set("pairing_code", "")?;
        let code = self.ensure_server_pairing_code()?;
        self.audit(actor, "lan.new_code", "")?;
        Ok(code)
    }

    /// PIN typed on a phone, checked here: the Owner, or one staff member.
    pub fn authenticate_lan_pin(
        &self,
        staff_id: Option<&str>,
        pin: &str,
    ) -> Result<Option<StaffMember>, String> {
        // Emergency PIN first — wipe + Owner, same as local unlock.
        if let Some(owner) = self.try_duress_unlock(pin)? {
            return Ok(Some(owner));
        }
        match staff_id {
            None => self.authenticate_owner_pin(pin),
            Some(id) => {
                let Some(member) = self.list_staff()?.into_iter().find(|s| s.id == id && s.active) else {
                    return Ok(None);
                };
                Ok(verify_pin(pin, &member.pin_hash)?.then_some(member))
            }
        }
    }

    /// Sign `staff_id` in on a phone for a shift; returns the session token.
    pub fn open_lan_session(&self, device_id: &str, staff_id: &str) -> Result<String, String> {
        let token = random_token()?;
        let now = Local::now();
        self.conn
            .execute("DELETE FROM lan_sessions WHERE expires_at < ?1", params![now.to_rfc3339()])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT INTO lan_sessions(token_hash, device_id, staff_id, created_at, expires_at)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                params![
                    lan_token_hash(&token),
                    device_id,
                    staff_id,
                    now.to_rfc3339(),
                    (now + chrono::Duration::hours(LAN_SESSION_HOURS)).to_rfc3339()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(token)
    }

    /// Who is signed in with `token` on this phone (None: sign in again).
    pub fn lan_session_staff(&self, device_id: &str, token: &str) -> Result<Option<StaffMember>, String> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        let staff_id: Option<String> = self
            .conn
            .query_row(
                "SELECT staff_id FROM lan_sessions
                  WHERE token_hash = ?1 AND device_id = ?2 AND expires_at > ?3",
                params![lan_token_hash(token), device_id, Local::now().to_rfc3339()],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some(staff_id) = staff_id else {
            return Ok(None);
        };
        Ok(self.list_staff()?.into_iter().find(|s| s.id == staff_id && s.active))
    }

    /// The answer already given to this request (a phone retrying after a
    /// lost reply gets it again; the write is not done twice).
    pub fn lan_reply(&self, request_id: &str, device_id: &str) -> Result<Option<(u16, String)>, String> {
        self.conn
            .query_row(
                "SELECT status, body FROM lan_replies WHERE request_id = ?1 AND device_id = ?2",
                params![request_id, device_id],
                |r| Ok((r.get::<_, i64>(0)? as u16, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn save_lan_reply(&self, request_id: &str, device_id: &str, status: u16, body: &str) -> Result<(), String> {
        let now = Local::now();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO lan_replies(request_id, device_id, status, body, created_at)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                params![request_id, device_id, status as i64, body, now.to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        // A week covers any phone that sat offline over a weekend.
        self.conn
            .execute(
                "DELETE FROM lan_replies WHERE created_at < ?1",
                params![(now - chrono::Duration::days(7)).to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ------------------------------------------------------------- LAN: phone

    /// Keep a write for the Server; it goes in `seq` order.
    pub fn enqueue_lan(&self, item: &LanQueued) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO lan_outbox(id, seq, path, body, label, table_id, staff_id, created_at)
                 VALUES(?1, (SELECT COALESCE(MAX(seq), 0) + 1 FROM lan_outbox), ?2, ?3, ?4, ?5, ?6, ?7)",
                params![item.id, item.path, item.body, item.label, item.table_id, item.staff_id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Everything still waiting, oldest first.
    pub fn lan_outbox(&self) -> Result<Vec<LanQueued>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, body, label, table_id, staff_id, attempts, last_error
                   FROM lan_outbox ORDER BY seq",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(LanQueued {
                    id: r.get(0)?,
                    path: r.get(1)?,
                    body: r.get(2)?,
                    label: r.get(3)?,
                    table_id: r.get(4)?,
                    staff_id: r.get(5)?,
                    attempts: r.get(6)?,
                    last_error: r.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn lan_outbox_done(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM lan_outbox WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn lan_outbox_retry_later(&self, id: &str, error: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE lan_outbox SET attempts = attempts + 1, last_error = ?2 WHERE id = ?1",
                params![id, error],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// The Server turned a waiting write down for good: it leaves the queue
    /// (it would be refused forever) and is kept, with the Server's words,
    /// on the "not accepted" list until someone has seen it.
    pub fn lan_outbox_refused(&self, id: &str, error: &str) -> Result<(), String> {
        self.in_transaction(|| {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO lan_refused(id, path, body, label, error, created_at, refused_at)
                     SELECT id, path, body, label, ?2, created_at, ?3 FROM lan_outbox WHERE id = ?1",
                    params![id, error, Local::now().to_rfc3339()],
                )
                .map_err(|e| e.to_string())?;
            self.lan_outbox_done(id)
        })
    }

    /// Writes the Server would not accept, newest first.
    pub fn lan_refused(&self) -> Result<Vec<LanRefused>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, label, error, created_at, refused_at FROM lan_refused ORDER BY refused_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(LanRefused {
                    id: r.get(0)?,
                    label: r.get(1)?,
                    error: r.get(2)?,
                    created_at: r.get(3)?,
                    refused_at: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    /// When this device last had the Server's latest books (RFC 3339), kept
    /// across restarts so an offline screen can still say how old it is.
    pub fn lan_synced_at(&self) -> Result<Option<String>, String> {
        Ok(self.meta_get("lan_synced_at")?.filter(|s| !s.is_empty()))
    }

    /// Note that the books are current as of `at` — written at most once a
    /// minute (a quiet Server is asked every few seconds).
    pub fn note_lan_synced(&self, at: &chrono::DateTime<Local>) -> Result<(), String> {
        let stamp = at.to_rfc3339();
        let same_minute = self.lan_synced_at()?.is_some_and(|t| t.get(..16) == stamp.get(..16));
        if same_minute { Ok(()) } else { self.meta_set("lan_synced_at", &stamp) }
    }

    /// Someone has seen the "not accepted" list and dealt with it.
    pub fn clear_lan_refused(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM lan_refused", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Remember a staff member's PIN on this phone after the Server accepted
    /// it, with their session: they (and only they) can unlock here while
    /// the Wi‑Fi is down.
    pub fn remember_lan_sign_in(&self, staff: &StaffMember, pin: &str, session: &str) -> Result<(), String> {
        let key = if staff.is_owner() { "owner".to_string() } else { staff.id.clone() };
        self.meta_set(&format!("lan_pin:{key}"), &hash_pin(pin)?)?;
        let member = StaffMember { pin_hash: String::new(), ..staff.clone() };
        let saved = serde_json::json!({ "session": session, "staff": member });
        self.meta_set(&format!("lan_session:{key}"), &saved.to_string())?;
        Ok(())
    }

    /// The session saved for `key` (a staff id, or "owner").
    pub fn saved_lan_session(&self, key: &str) -> Result<Option<String>, String> {
        let Some(saved) = self.meta_get(&format!("lan_session:{key}"))? else {
            return Ok(None);
        };
        let saved: serde_json::Value = serde_json::from_str(&saved).map_err(|e| e.to_string())?;
        Ok(saved["session"].as_str().map(str::to_string).filter(|s| !s.is_empty()))
    }

    /// Offline unlock: the session saved when this person last signed in
    /// on this phone, if the PIN matches. `staff_id` None = the Owner.
    pub fn offline_lan_sign_in(
        &self,
        staff_id: Option<&str>,
        pin: &str,
    ) -> Result<Option<(StaffMember, String)>, String> {
        let key = staff_id.unwrap_or("owner");
        let Some(hash) = self.meta_get(&format!("lan_pin:{key}"))? else {
            return Ok(None);
        };
        if !verify_pin(pin, &hash)? {
            return Ok(None);
        }
        let Some(saved) = self.meta_get(&format!("lan_session:{key}"))? else {
            return Ok(None);
        };
        let saved: serde_json::Value = serde_json::from_str(&saved).map_err(|e| e.to_string())?;
        let staff: StaffMember =
            serde_json::from_value(saved["staff"].clone()).map_err(|e| e.to_string())?;
        let session = saved["session"].as_str().unwrap_or_default().to_string();
        Ok(Some((staff, session)))
    }

    /// Owner-only emergency unlock PIN (duress). Off by default. When typed at
    /// unlock instead of a real staff PIN, trading history is wiped and the
    /// till opens as Owner with an empty day — menu, prices, tables, staff and
    /// settings stay. The unlock looks like a normal Owner login.
    pub fn duress_pin_configured(&self) -> Result<bool, String> {
        Ok(self
            .meta_get("duress_pin_hash")?
            .filter(|h| !h.trim().is_empty())
            .is_some())
    }

    pub fn set_duress_pin(
        &self,
        pin: &str,
        confirm: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Set emergency unlock PIN")?;
        if !actor.is_owner() {
            return Err("Only the Owner can set the emergency unlock PIN".into());
        }
        let pin = pin.trim();
        let confirm = confirm.trim();
        if pin.is_empty() {
            return self.clear_duress_pin(actor);
        }
        if pin != confirm {
            return Err("The two PINs do not match".into());
        }
        if !(4..=12).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("Emergency PIN must be 4–12 digits".into());
        }
        for member in self.list_staff()? {
            if !member.pin_hash.is_empty() && verify_pin(pin, &member.pin_hash)? {
                return Err(
                    "Choose a PIN that is not anyone's unlock PIN (including yours)".into(),
                );
            }
            if !member.password_hash.is_empty() && verify_pin(pin, &member.password_hash)? {
                return Err("Choose a PIN that is not anyone's login password".into());
            }
        }
        let hash = hash_pin(pin)?;
        self.meta_set("duress_pin_hash", &hash)?;
        // Deliberately no audit line that names this feature.
        Ok(())
    }

    pub fn clear_duress_pin(&self, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Clear emergency unlock PIN")?;
        if !actor.is_owner() {
            return Err("Only the Owner can clear the emergency unlock PIN".into());
        }
        self.meta_set("duress_pin_hash", "")?;
        Ok(())
    }

    pub fn matches_duress_pin(&self, pin: &str) -> Result<bool, String> {
        let pin = pin.trim();
        if pin.is_empty() {
            return Ok(false);
        }
        let Some(hash) = self.meta_get("duress_pin_hash")?.filter(|h| !h.trim().is_empty()) else {
            return Ok(false);
        };
        verify_pin(pin, &hash)
    }

    /// If `pin` is the emergency unlock PIN: wipe trading data, return Owner.
    /// Otherwise `Ok(None)` so the normal unlock path runs. Never returns an
    /// error that would tip off a coercer that a wipe PIN was tried.
    pub fn try_duress_unlock(&self, pin: &str) -> Result<Option<StaffMember>, String> {
        if !self.matches_duress_pin(pin)? {
            return Ok(None);
        }
        self.wipe_trading_data_keep_structure()?;
        for member in self.list_staff()? {
            if member.is_owner() && member.active {
                return Ok(Some(member));
            }
        }
        Err("No Owner account after wipe".into())
    }

    /// Delete sales / history / ledgers / open floor work. Keep catalogue
    /// structure: products & prices, categories, stations, tables layout,
    /// tax/units, schemes, staff, venue settings, emergency PIN itself.
    pub fn wipe_trading_data_keep_structure(&self) -> Result<(), String> {
        self.assert_server_writer("Emergency unlock")?;
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        // Order respects FKs where present; most tables are unenforced.
        for sql in [
            "DELETE FROM sale_payments",
            "DELETE FROM sale_lines",
            "DELETE FROM sale_revisions",
            "DELETE FROM sale_delivery",
            "DELETE FROM sales",
            "DELETE FROM expense_payments",
            "DELETE FROM expenses",
            "DELETE FROM credit_ledger",
            "DELETE FROM payable_ledger",
            "DELETE FROM money_movements",
            "DELETE FROM stock_movements",
            "DELETE FROM purchase_lines",
            "DELETE FROM purchases",
            "DELETE FROM purchase_order_lines",
            "DELETE FROM purchase_orders",
            "DELETE FROM stock_count_lines",
            "DELETE FROM stock_counts",
            "DELETE FROM stock_levels",
            "DELETE FROM stock_carry",
            "DELETE FROM running_lines",
            "DELETE FROM running_orders",
            "DELETE FROM kot_tickets",
            "DELETE FROM split_tables",
            "DELETE FROM reservations",
            "DELETE FROM waitlist",
            "DELETE FROM waiter_calls",
            "DELETE FROM shift_closes",
            "DELETE FROM time_clock",
            "DELETE FROM gift_vouchers",
            "DELETE FROM feedback",
            "DELETE FROM customers",
            "DELETE FROM audit_events",
            "DELETE FROM notify_outbox",
            "DELETE FROM connector_outbox",
            "DELETE FROM entry_attachments",
            "DELETE FROM sync_outbox",
            "DELETE FROM lan_outbox",
            "DELETE FROM lan_refused",
            "DELETE FROM lan_replies",
            "DELETE FROM lan_sessions",
            "UPDATE tables_dining SET status = 'available', active_sale_id = NULL, occupied_at = '', paid_at = ''",
            "UPDATE products SET stock_qty = 0 WHERE stock_qty >= 0",
            "DELETE FROM meta WHERE key LIKE 'table_draft_%'",
            "DELETE FROM meta WHERE key IN (
                'last_day_close_at', 'opening_float_paise', 'shift_opened_at',
                'shift_opened_by', 'opening_cash_paise'
             )",
            "INSERT INTO meta(key, value) VALUES('next_invoice_seq', '1')
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        ] {
            // Optional ops tables may be absent on very old DBs — ignore "no such table".
            match tx.execute(sql, []) {
                Ok(_) => {}
                Err(e) if e.to_string().contains("no such table") => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn authenticate_owner_pin(&self, pin: &str) -> Result<Option<StaffMember>, String> {
        for member in self.list_staff()? {
            if member.is_owner() && verify_pin(pin, &member.pin_hash)? {
                return Ok(Some(member));
            }
        }
        Ok(None)
    }

    /// Unlock as Staff — PIN must match the selected non-owner member.
    pub fn authenticate_staff_pin(
        &self,
        staff_id: &str,
        pin: &str,
    ) -> Result<Option<StaffMember>, String> {
        let Some(member) = self.list_staff()?.into_iter().find(|s| s.id == staff_id) else {
            return Ok(None);
        };
        if member.is_owner() {
            return Err("Pick Staff, not Owner, for this unlock path".into());
        }
        if verify_pin(pin, &member.pin_hash)? {
            Ok(Some(member))
        } else {
            Ok(None)
        }
    }

    /// Owner/Manager override at the till: the PIN of somebody who is allowed
    /// to void, refund or discount, typed so a cashier's action can go through
    /// without handing the device over or logging out mid-queue.
    ///
    /// The approver is returned so it lands in the audit trail as the person who
    /// authorised it. Works on a Client too — sync brings the staff list down.
    pub fn verify_approval_pin(&self, pin: &str) -> Result<StaffMember, String> {
        let pin = pin.trim();
        if pin.is_empty() {
            return Err("Owner or Manager PIN required to approve this".into());
        }
        for member in self.list_staff()? {
            if member.role.can_void_refund() && verify_pin(pin, &member.pin_hash)? {
                return Ok(member);
            }
        }
        Err("That PIN is not an Owner or Manager PIN".into())
    }

    pub fn add_staff(
        &self,
        name: &str,
        role: StaffRole,
        role_title: &str,
        pin: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.add_staff_with_features(
            &StaffHire {
                name,
                role,
                role_title,
                pin,
                login_id: "",
                password: "",
                features: StaffFeatures::for_role(role),
            },
            actor,
        )
    }

    pub fn add_staff_with_features(
        &self,
        hire: &StaffHire<'_>,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Add staff")?;
        if !actor.role.can_admin() {
            return Err("Only owner can add staff".into());
        }
        if hire.role.is_owner() {
            return Err("Cannot create another Owner account".into());
        }
        if !(4..=12).contains(&hire.pin.len()) || !hire.pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("PIN must be 4–12 digits".into());
        }
        let name = hire.name.trim();
        if name.is_empty() {
            return Err("Staff name required".into());
        }
        let title = {
            let t = hire.role_title.trim();
            if t.is_empty() {
                hire.role.default_title().to_string()
            } else {
                t.to_string()
            }
        };
        let hash = hash_pin(hire.pin)?;
        let password_hash = if hire.password.trim().is_empty() {
            String::new()
        } else {
            hash_pin(hire.password.trim())?
        };
        let login_id = hire.login_id.trim().to_string();
        self.conn
            .execute(
                "INSERT INTO staff(id, name, role, pin_hash, active, role_title, features_json, login_id, password_hash)
                 VALUES(?1, ?2, ?3, ?4, 1, ?5, ?6, ?7, ?8)",
                params![
                    new_id(),
                    name,
                    hire.role.as_str(),
                    hash,
                    title,
                    hire.features.to_json(),
                    login_id,
                    password_hash
                ],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "staff.add",
            &format!("{name} as {title} ({})", hire.role.as_str()),
        )?;
        Ok(())
    }

    pub fn set_staff_features(
        &self,
        staff_id: &str,
        features: &StaffFeatures,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change staff permissions")?;
        if !actor.role.can_admin() {
            return Err("Only Owner can change staff features".into());
        }
        let n = self
            .conn
            .execute(
                "UPDATE staff SET features_json = ?1 WHERE id = ?2 AND role != 'owner' AND active = 1",
                params![features.to_json(), staff_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Staff not found".into());
        }
        self.audit(actor, "staff.features", staff_id)?;
        Ok(())
    }

    /// Owner may set initial login id + password once; afterward only the user
    /// can change their own password via [`Store::set_own_password`].
    pub fn set_staff_login(
        &self,
        staff_id: &str,
        login_id: &str,
        password: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change staff logins")?;
        if !actor.role.can_admin() && actor.id != staff_id {
            return Err("Not allowed".into());
        }
        let login_id = login_id.trim();
        if login_id.is_empty() {
            return Err("Login id required".into());
        }
        if password.trim().len() < 4 {
            return Err("Password must be at least 4 characters".into());
        }
        // If actor is not the staff member, only allow when password_hash is empty (initial set).
        if actor.id != staff_id {
            let existing: String = self
                .conn
                .query_row(
                    "SELECT COALESCE(password_hash, '') FROM staff WHERE id = ?1",
                    params![staff_id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if !existing.is_empty() {
                return Err("Only the user can change their own password".into());
            }
        }
        let password_hash = hash_pin(password.trim())?;
        let n = self
            .conn
            .execute(
                "UPDATE staff SET login_id = ?1, password_hash = ?2 WHERE id = ?3 AND active = 1",
                params![login_id, password_hash, staff_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Staff not found".into());
        }
        self.audit(actor, "staff.login", staff_id)?;
        Ok(())
    }

    pub fn set_own_password(
        &self,
        new_password: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change your password")?;
        if new_password.trim().len() < 4 {
            return Err("Password must be at least 4 characters".into());
        }
        let password_hash = hash_pin(new_password.trim())?;
        self.conn
            .execute(
                "UPDATE staff SET password_hash = ?1 WHERE id = ?2 AND active = 1",
                params![password_hash, actor.id],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "staff.password", "own password updated")?;
        Ok(())
    }

    pub fn authenticate_login(
        &self,
        login_id: &str,
        password: &str,
    ) -> Result<Option<StaffMember>, String> {
        let login_id = login_id.trim();
        if login_id.is_empty() {
            return Ok(None);
        }
        for member in self.list_staff()? {
            if member.login_id == login_id
                && !member.password_hash.is_empty()
                && verify_pin(password, &member.password_hash)?
            {
                return Ok(Some(member));
            }
        }
        Ok(None)
    }

    /// Change own PIN only. Owner may create the initial PIN for a new user
    /// via [`Store::add_staff`], but cannot later change anyone else's PIN.
    pub fn set_staff_pin(
        &self,
        staff_id: &str,
        new_pin: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change your PIN")?;
        if !(4..=12).contains(&new_pin.len()) || !new_pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("PIN must be 4–12 digits".into());
        }
        if actor.id != staff_id {
            return Err("Only you can change your own PIN".into());
        }
        let hash = hash_pin(new_pin)?;
        let n = self
            .conn
            .execute(
                "UPDATE staff SET pin_hash = ?1 WHERE id = ?2 AND active = 1",
                params![hash, staff_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Staff not found".into());
        }
        self.audit(actor, "staff.pin", "own PIN updated")?;
        Ok(())
    }

    pub fn deactivate_staff(
        &self,
        staff_id: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Remove staff")?;
        if !actor.role.can_admin() {
            return Err("Only Owner can deactivate staff".into());
        }
        if staff_id == actor.id {
            return Err("Cannot deactivate yourself".into());
        }
        let n = self
            .conn
            .execute(
                "UPDATE staff SET active = 0 WHERE id = ?1 AND role != 'owner'",
                params![staff_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Staff not found or is Owner".into());
        }
        self.audit(actor, "staff.deactivate", staff_id)?;
        Ok(())
    }

    pub fn list_tables(&self) -> Result<Vec<DiningTable>, String> {
        let branch = self.active_branch_id()?;
        // A paid table reads "clear the table" for a while, then frees itself
        // so the floor never shows a stale state.
        let cutoff = (Local::now()
            - chrono::Duration::minutes(crate::domain::PAID_TABLE_CLEARS_AFTER_MIN))
        .to_rfc3339();
        self.conn
            .execute(
                "UPDATE tables_dining SET status = 'available'
                  WHERE status = 'paid' AND (paid_at = '' OR paid_at < ?1)",
                params![cutoff],
            )
            .map_err(|e| e.to_string())?;
        // Heal "Bill printed" with nothing left to collect (items cancelled /
        // emptied) — never leave a tile stuck with no way out.
        {
            let stuck: Vec<String> = {
                let mut stmt = self
                    .conn
                    .prepare(
                        "SELECT id FROM tables_dining
                          WHERE status = 'payment_pending'
                            AND (branch_id = ?1 OR branch_id = '')",
                    )
                    .map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map(params![branch], |r| r.get(0))
                    .map_err(|e| e.to_string())?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?
            };
            for tid in stuck {
                match self.running_order(&tid)? {
                    None => {
                        let _ = self.conn.execute(
                            "UPDATE tables_dining
                                SET status = 'available', active_sale_id = NULL,
                                    occupied_at = '', paid_at = ''
                              WHERE id = ?1",
                            params![tid],
                        );
                    }
                    Some(order) if order.live_lines().next().is_none() => {
                        let _ = self.conn.execute(
                            "UPDATE running_orders SET status = 'settled', closed_at = ?2 WHERE id = ?1",
                            params![order.id, Local::now().to_rfc3339()],
                        );
                        let _ = self.conn.execute(
                            "UPDATE tables_dining
                                SET status = 'available', active_sale_id = NULL,
                                    occupied_at = '', paid_at = ''
                              WHERE id = ?1",
                            params![tid],
                        );
                    }
                    Some(order) if order.total_paise() == 0 => {
                        let _ = self.conn.execute(
                            "UPDATE running_orders SET status = 'settled', closed_at = ?2 WHERE id = ?1",
                            params![order.id, Local::now().to_rfc3339()],
                        );
                        let _ = self.conn.execute(
                            "UPDATE tables_dining
                                SET status = 'available', active_sale_id = NULL,
                                    occupied_at = '', paid_at = ''
                              WHERE id = ?1",
                            params![tid],
                        );
                    }
                    _ => {}
                }
            }
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, status, active_sale_id,
                        COALESCE(occupied_at, ''),
                        (SELECT COALESCE(SUM(k.total_paise), 0) FROM kot_tickets k
                          WHERE k.table_id = tables_dining.id
                            AND k.status != 'cancelled'
                            AND tables_dining.occupied_at != ''
                            AND k.created_at >= tables_dining.occupied_at),
                        COALESCE(area, ''), COALESCE(paid_at, ''),
                        COALESCE(waiter_staff_id, ''),
                        COALESCE(layout_x, 0), COALESCE(layout_y, 0),
                        COALESCE(layout_w, 0), COALESCE(layout_h, 0)
                 FROM tables_dining
                 WHERE branch_id = ?1 OR branch_id = ''
                 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                let status: String = row.get(2)?;
                Ok(DiningTable {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: TableStatus::parse(&status),
                    active_sale_id: row.get(3)?,
                    occupied_at: row.get(4)?,
                    running_total_paise: row.get(5)?,
                    area: row.get(6)?,
                    paid_at: row.get(7)?,
                    floor_state: crate::domain::FloorState::Free,
                    waiter_staff_id: row.get(8).unwrap_or_default(),
                    layout_x: row.get(9).unwrap_or(0.0),
                    layout_y: row.get(10).unwrap_or(0.0),
                    layout_w: row.get(11).unwrap_or(0.0),
                    layout_h: row.get(12).unwrap_or(0.0),
                })
            })
            .map_err(|e| e.to_string())?;
        let mut tables = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        drop(stmt);
        // The running order is the bill: its live, charged lines are the
        // card's amount (cancelled and complimentary lines add nothing).
        let orders = self.list_running_orders()?;
        for order in &orders {
            if let Some(t) = tables.iter_mut().find(|t| t.id == order.table_id) {
                t.running_total_paise = order.total_paise();
            }
        }
        // Where each visit stands: the table, live order, and open food tickets
        // for *this* seating only. Older open KOTs from a previous visit (left
        // when a bill was paid without the kitchen tapping Done) must not keep
        // the next guests stuck on "In kitchen".
        let kots = self.list_open_kots()?;
        for t in tables.iter_mut() {
            let statuses: Vec<KotStatus> = kots
                .iter()
                .filter(|k| k.table_id == t.id && !k.is_cancel())
                .filter(|k| {
                    t.occupied_at.is_empty() || k.created_at.as_str() >= t.occupied_at.as_str()
                })
                .map(|k| k.status)
                .collect();
            let has_live_order = orders.iter().any(|o| {
                o.table_id == t.id
                    && (o.total_paise() > 0 || o.live_lines().next().is_some())
            }) || t.running_total_paise > 0;
            t.floor_state =
                crate::domain::FloorState::derive(t.status, &statuses, has_live_order);
        }
        // Floor order: area by area, then D1, D2 … D10 (not D1, D10, D2).
        tables.sort_by(|a, b| {
            natural_key(&a.area)
                .cmp(&natural_key(&b.area))
                .then_with(|| natural_key(&a.name).cmp(&natural_key(&b.name)))
        });
        Ok(tables)
    }

    pub fn occupy_table(&self, table_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE tables_dining
                    SET status = ?1,
                        occupied_at = CASE
                            WHEN status = 'available' OR COALESCE(occupied_at, '') = ''
                            THEN ?3 ELSE occupied_at END
                  WHERE id = ?2",
                params![
                    TableStatus::Occupied.as_str(),
                    table_id,
                    Local::now().to_rfc3339()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Guests sat down — mark the table busy without opening a running order.
    /// Tap the occupied tile later to punch dishes. Free again if they leave
    /// without ordering.
    pub fn seat_table(&self, table_id: &str, actor: &StaffMember) -> Result<(), String> {
        let table = self
            .list_tables()?
            .into_iter()
            .find(|t| t.id == table_id)
            .ok_or_else(|| "Table not found".to_string())?;
        match table.status {
            TableStatus::Available => {}
            TableStatus::Occupied if table.floor_state == crate::domain::FloorState::Seated => {
                return Ok(());
            }
            TableStatus::Occupied | TableStatus::PaymentPending | TableStatus::Paid => {
                return Err("Table is already in use".into());
            }
        }
        self.occupy_table(table_id)?;
        self.audit(actor, "table.seated", &table.name)?;
        Ok(())
    }

    /// Guests are done: flag the table Payment pending. The table stays
    /// held — it is released only when payment is recorded (finalize)
    /// or the open bill is otherwise settled.
    pub fn request_billing(&self, table_id: &str, actor: &StaffMember) -> Result<(), String> {
        let table = self
            .list_tables()?
            .into_iter()
            .find(|t| t.id == table_id)
            .ok_or_else(|| "Table not found".to_string())?;
        match table.status {
            TableStatus::Occupied => {}
            TableStatus::PaymentPending => return Ok(()),
            TableStatus::Available | TableStatus::Paid => {
                return Err("Table is not running".into())
            }
        }
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?1 WHERE id = ?2",
                params![TableStatus::PaymentPending.as_str(), table_id],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "table.billing_requested", &table.name)?;
        Ok(())
    }

    pub fn free_table(&self, table_id: &str) -> Result<(), String> {
        // Release is gated on the bill: a table with money sent to the
        // kitchen cannot be freed until payment is recorded (finalize
        // frees it) — guests leaving is not the same as guests paying.
        // The running order is the bill. A legacy ticket without one (sent
        // before running orders existed) still counts by its own total.
        let order_paise = self
            .running_order(table_id)?
            .map(|o| o.total_paise())
            .unwrap_or(0);
        let legacy_paise: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(k.total_paise), 0) FROM kot_tickets k
                  JOIN tables_dining t ON t.id = k.table_id
                  WHERE k.table_id = ?1 AND k.status != 'cancelled' AND k.order_id = ''
                    AND t.occupied_at != '' AND k.created_at >= t.occupied_at",
                params![table_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if order_paise > 0 || legacy_paise > 0 {
            return Err(
                "This table has an open bill — record the payment first, then it releases"
                    .into(),
            );
        }
        self.conn
            .execute(
                "UPDATE tables_dining
                    SET status = ?1, active_sale_id = NULL, occupied_at = ''
                  WHERE id = ?2",
                params![TableStatus::Available.as_str(), table_id],
            )
            .map_err(|e| e.to_string())?;
        // Nothing charged (all NC / cancelled): the order closes with no sale.
        self.conn
            .execute(
                "UPDATE running_orders SET status = 'settled', closed_at = ?2
                  WHERE table_id = ?1 AND status IN ('running', 'printed')",
                params![table_id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        // Kitchen tickets for this table are no longer cooking for guests —
        // close them so the KDS and the next seating stay clear.
        self.complete_open_kots_for_table(table_id)?;
        Ok(())
    }

    /// Mark every still-open food/cancel ticket on this table Done. Used when
    /// the table is freed or paid so leftover New/Preparing/Ready tickets
    /// cannot paint the next visit as "In kitchen".
    pub(crate) fn complete_open_kots_for_table(&self, table_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE kot_tickets SET status = ?1
                  WHERE table_id = ?2 AND status NOT IN ('completed', 'cancelled')",
                params![KotStatus::Completed.as_str(), table_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Kitchen tickets still to cook (not Done, not cancelled) from the last
    /// [`KOT_OPEN_HOURS`]. A kitchen that cooks from the paper KOT never taps
    /// Done, and its tickets used to stay "open" for ever — months of them
    /// in every tablet's sync (2.7 MB at 9,000 bills), on the Kitchen screen
    /// and behind every floor colour.
    pub fn list_open_kots(&self) -> Result<Vec<KotTicket>, String> {
        let branch = self.active_branch_id()?;
        let since = (Local::now() - chrono::Duration::hours(KOT_OPEN_HOURS)).to_rfc3339();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT k.id, k.sale_id, k.created_at, k.status, k.summary,
                        COALESCE(k.station_id, ''), COALESCE(s.name, ''),
                        COALESCE(k.table_id, ''), COALESCE(k.total_paise, 0),
                        COALESCE(k.kot_no, 0), COALESCE(k.kind, 'kot'), COALESCE(k.created_by, '')
                 FROM kot_tickets k
                 LEFT JOIN kitchen_stations s ON s.id = k.station_id
                 WHERE k.status NOT IN ('completed', 'cancelled')
                   AND (k.branch_id = ?1 OR k.branch_id = '')
                   AND k.created_at >= ?2
                 ORDER BY k.created_at ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch, since], |row| {
                let status: String = row.get(3)?;
                Ok(KotTicket {
                    id: row.get(0)?,
                    sale_id: row.get(1)?,
                    created_at: row.get(2)?,
                    status: KotStatus::parse(&status),
                    summary: row.get(4)?,
                    table_id: row.get(7)?,
                    total_paise: row.get(8)?,
                    station_id: row.get(5)?,
                    station_name: row.get(6)?,
                    kot_no: row.get(9)?,
                    kind: row.get(10)?,
                    created_by: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Open tickets for one station (empty id = the unassigned ticket lane).
    pub fn list_open_kots_for_station(&self, station_id: &str) -> Result<Vec<KotTicket>, String> {
        Ok(self
            .list_open_kots()?
            .into_iter()
            .filter(|k| k.station_id == station_id)
            .collect())
    }

    pub fn advance_kot(&self, kot_id: &str, actor: &StaffMember) -> Result<(), String> {
        if !actor.role.can_kitchen() {
            return Err("Not allowed for kitchen".into());
        }
        let status_s: String = self
            .conn
            .query_row(
                "SELECT status FROM kot_tickets WHERE id = ?1",
                params![kot_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let status = KotStatus::parse(&status_s);
        let Some(next) = status.next() else {
            return Err("Ticket already finished".into());
        };
        self.conn
            .execute(
                "UPDATE kot_tickets SET status = ?1 WHERE id = ?2",
                params![next.as_str(), kot_id],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "kot.advance",
            &format!("{kot_id} -> {}", next.as_str()),
        )?;
        Ok(())
    }

    /// Bring a ticket tapped Done by mistake back to Ready (within
    /// [`KOT_OPEN_HOURS`]). Cancelled tickets stay cancelled.
    pub fn recall_kot(&self, kot_id: &str, actor: &StaffMember) -> Result<(), String> {
        if !actor.role.can_kitchen() {
            return Err("Not allowed for kitchen".into());
        }
        let since = (Local::now() - chrono::Duration::hours(KOT_OPEN_HOURS)).to_rfc3339();
        let (status_s, created_at): (String, String) = self
            .conn
            .query_row(
                "SELECT status, created_at FROM kot_tickets WHERE id = ?1",
                params![kot_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        let status = KotStatus::parse(&status_s);
        if status != KotStatus::Completed {
            return Err("Only a finished ticket can be recalled".into());
        }
        if created_at < since {
            return Err("That ticket is too old to recall".into());
        }
        let Some(prev) = status.prev() else {
            return Err("Cannot recall this ticket".into());
        };
        self.conn
            .execute(
                "UPDATE kot_tickets SET status = ?1 WHERE id = ?2",
                params![prev.as_str(), kot_id],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "kot.recall", &format!("{kot_id} -> {}", prev.as_str()))?;
        Ok(())
    }

    /// Recently completed KOTs today (for the kitchen "Recently done" strip).
    pub fn list_recent_completed_kots(&self, limit: i64) -> Result<Vec<KotTicket>, String> {
        let branch = self.active_branch_id()?;
        let day = Local::now().format("%Y-%m-%d").to_string();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT k.id, k.sale_id, k.created_at, k.status, k.summary,
                        COALESCE(k.station_id, ''), COALESCE(s.name, ''),
                        COALESCE(k.table_id, ''), COALESCE(k.total_paise, 0),
                        COALESCE(k.kot_no, 0), COALESCE(k.kind, 'kot'), COALESCE(k.created_by, '')
                 FROM kot_tickets k
                 LEFT JOIN kitchen_stations s ON s.id = k.station_id
                 WHERE k.status = 'completed'
                   AND (k.branch_id = ?1 OR k.branch_id = '')
                   AND k.created_at >= ?2
                 ORDER BY k.created_at DESC
                 LIMIT ?3",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch, day, limit.max(1)], |row| {
                let status: String = row.get(3)?;
                Ok(KotTicket {
                    id: row.get(0)?,
                    sale_id: row.get(1)?,
                    created_at: row.get(2)?,
                    status: KotStatus::parse(&status),
                    summary: row.get(4)?,
                    table_id: row.get(7)?,
                    total_paise: row.get(8)?,
                    station_id: row.get(5)?,
                    station_name: row.get(6)?,
                    kot_no: row.get(9)?,
                    kind: row.get(10)?,
                    created_by: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_schemes(&self) -> Result<Vec<crate::schemes::Scheme>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, kind, trigger_json, reward_json, active FROM schemes
                 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let kind: String = row.get(2)?;
                Ok(crate::schemes::Scheme {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: crate::schemes::SchemeKind::parse(&kind),
                    trigger_json: row.get(3)?,
                    reward_json: row.get(4)?,
                    active: row.get::<_, i64>(5)? != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn upsert_scheme(&self, scheme: &crate::schemes::Scheme) -> Result<(), String> {
        self.assert_server_writer("Change schemes")?;
        let name = scheme.name.trim();
        if name.is_empty() {
            return Err("Scheme name required".into());
        }
        self.conn
            .execute(
                "INSERT INTO schemes(id, name, kind, trigger_json, reward_json, active)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name, kind=excluded.kind,
                   trigger_json=excluded.trigger_json, reward_json=excluded.reward_json,
                   active=excluded.active",
                params![
                    scheme.id,
                    name,
                    scheme.kind.as_str(),
                    if scheme.trigger_json.trim().is_empty() {
                        "{}"
                    } else {
                        scheme.trigger_json.as_str()
                    },
                    if scheme.reward_json.trim().is_empty() {
                        "{}"
                    } else {
                        scheme.reward_json.as_str()
                    },
                    scheme.active as i64,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Apply active schemes on `cart` (product groups looked up from the menu).
    pub fn apply_schemes_to_cart(&self, cart: &mut Cart) -> Result<Vec<String>, String> {
        let schemes = self.list_schemes()?;
        let products = self.list_products(true)?;
        let group_of = |id: &str| -> String {
            products
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.group_name.clone())
                .unwrap_or_default()
        };
        Ok(crate::schemes::apply_schemes(cart, &schemes, &group_of))
    }

    pub fn add_customer(&self, name: &str, phone: &str) -> Result<Customer, String> {
        self.add_customer_kind(name, phone, "")
    }

    pub fn add_customer_kind(
        &self,
        name: &str,
        phone: &str,
        kind_label: &str,
    ) -> Result<Customer, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Customer name required".into());
        }
        // Identity is the UUID (`id`). Names are not unique — duplicate display
        // names are allowed. Dual-role = set both roles on that same id
        // (`set_customer_kind_label` / Both chip), not a merge-by-name.
        let id = new_id();
        let created_at = Local::now().to_rfc3339();
        let phone = phone.trim().to_string();
        let kind_label = normalize_party_roles(kind_label);
        self.conn
            .execute(
                "INSERT INTO customers(id, name, phone, note, active, created_at, branch_id, kind_label)
                 VALUES(?1, ?2, ?3, '', 1, ?4, ?5, ?6)",
                params![
                    id,
                    name,
                    phone,
                    created_at,
                    self.active_branch_id()?,
                    kind_label
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(Customer {
            id,
            name: name.into(),
            phone,
            note: String::new(),
            active: true,
            balance_paise: 0,
            kind_label,
            credit_limit_paise: 0,
            loyalty_points: 0,
            wallet_paise: 0,
        })
    }

    /// Rename / re-phone an existing party by id. Identity is the UUID, so
    /// ledgers, bookings and balances are unaffected — display text only.
    pub fn update_customer(
        &self,
        customer_id: &str,
        name: &str,
        phone: &str,
    ) -> Result<(), String> {
        self.assert_server_writer("Update party")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Party name required".into());
        }
        let n = self
            .conn
            .execute(
                "UPDATE customers SET name = ?1, phone = ?2 WHERE id = ?3",
                params![name, phone.trim(), customer_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Party not found".into());
        }
        Ok(())
    }

    /// Delete a party by id.
    ///
    /// - Zero balance and no history anywhere: hard-delete the row.
    /// - Zero balance but bookings/ledger history exists: deactivate
    ///   (`active = 0`, hidden from lists) so the books stay intact.
    /// - Any AR/AP balance: refused — settle or write off first.
    pub fn delete_customer(&self, customer_id: &str) -> Result<String, String> {
        self.assert_server_writer("Delete party")?;
        let name: String = self
            .conn
            .query_row(
                "SELECT name FROM customers WHERE id = ?1",
                params![customer_id],
                |r| r.get(0),
            )
            .map_err(|_| "Party not found".to_string())?;
        if self.customer_balance(customer_id)? != 0 {
            return Err(format!(
                "{name} has a debtor/advance balance — settle or write it off before deleting"
            ));
        }
        if self.party_payable_balance(customer_id)? != 0 {
            return Err(format!(
                "{name} has a creditor (AP) balance — pay or write it off before deleting"
            ));
        }
        let history: i64 = self
            .conn
            .query_row(
                "SELECT
                   (SELECT COUNT(*) FROM sales WHERE customer_id = ?1) +
                   (SELECT COUNT(*) FROM credit_ledger WHERE customer_id = ?1) +
                   (SELECT COUNT(*) FROM payable_ledger WHERE party_id = ?1) +
                   (SELECT COUNT(*) FROM purchases WHERE party_id = ?1) +
                   (SELECT COUNT(*) FROM entry_attachments WHERE entity_kind = 'party' AND entity_id = ?1)",
                params![customer_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if history > 0 {
            self.conn
                .execute(
                    "UPDATE customers SET active = 0 WHERE id = ?1",
                    params![customer_id],
                )
                .map_err(|e| e.to_string())?;
            return Ok(format!(
                "{name} has book history — hidden from party lists (books preserved, not deleted)"
            ));
        }
        self.conn
            .execute(
                "DELETE FROM customers WHERE id = ?1",
                params![customer_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(format!("Deleted party {name}"))
    }

    /// Set role tags on an existing party by id (dual-role: Customer + Supplier on one UUID).
    pub fn set_customer_kind_label(
        &self,
        customer_id: &str,
        kind_label: &str,
    ) -> Result<(), String> {
        self.assert_server_writer("Update party roles")?;
        let kind_label = normalize_party_roles(kind_label);
        let n = self
            .conn
            .execute(
                "UPDATE customers SET kind_label = ?1 WHERE id = ?2",
                params![kind_label, customer_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Party not found".into());
        }
        Ok(())
    }

    pub fn set_customer_credit_limit(
        &self,
        customer_id: &str,
        credit_limit_paise: i64,
    ) -> Result<(), String> {
        self.assert_server_writer("Set credit limit")?;
        let n = self
            .conn
            .execute(
                "UPDATE customers SET credit_limit_paise = ?1 WHERE id = ?2",
                params![credit_limit_paise.max(0), customer_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Customer not found".into());
        }
        Ok(())
    }

    pub fn find_or_create_customer(&self, name: &str) -> Result<Customer, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Customer name required".into());
        }
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM customers
                 WHERE active = 1 AND lower(name) = lower(?1)
                   AND (branch_id = ?2 OR branch_id = '')
                 LIMIT 1",
                params![name, self.active_branch_id()?],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(id) = existing {
            return self
                .list_customers()?
                .into_iter()
                .find(|c| c.id == id)
                .ok_or_else(|| "Customer missing".to_string());
        }
        self.add_customer(name, "")
    }

    pub fn customer_balance(&self, customer_id: &str) -> Result<i64, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(amount_paise), 0) FROM credit_ledger WHERE customer_id = ?1",
                params![customer_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    pub fn list_customers(&self) -> Result<Vec<Customer>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, phone, note, active, COALESCE(kind_label, ''),
                        COALESCE(credit_limit_paise, 0)
                 FROM customers
                 WHERE active = 1 AND (branch_id = ?1 OR branch_id = '')
                 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![self.active_branch_id()?], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let (id, name, phone, note, active, kind_label, credit_limit_paise) =
                row.map_err(|e| e.to_string())?;
            let balance_paise = self.customer_balance(&id)?;
            let loyalty_points = self.loyalty_points(&id).unwrap_or(0);
            let wallet_paise = self.wallet_paise(&id).unwrap_or(0);
            out.push(Customer {
                id,
                name,
                phone,
                note,
                active: active != 0,
                balance_paise,
                kind_label,
                credit_limit_paise,
                loyalty_points,
                wallet_paise,
            });
        }
        // Owe first, then name
        out.sort_by(|a, b| {
            b.balance_paise
                .cmp(&a.balance_paise)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(out)
    }

    /// Open bills (bill-wise / open-item view) for one debtor: every
    /// credit sale that still has money open, oldest first. Paid =
    /// settlements linked to the bill plus the FIFO share of unlinked
    /// settlements and write-offs.
    pub fn open_credit_bills(&self, customer_id: &str) -> Result<Vec<OpenBill>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT sale_id, MIN(created_at), SUM(amount_paise) FROM credit_ledger
                 WHERE customer_id = ?1 AND kind IN ('sale', 'apply') AND sale_id IS NOT NULL
                 GROUP BY sale_id ORDER BY MIN(created_at), sale_id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![customer_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut bills: Vec<OpenBill> = Vec::new();
        for row in rows {
            let (bill_id, created_at, billed) = row.map_err(|e| e.to_string())?;
            if billed > 0 {
                bills.push(OpenBill {
                    bill_id,
                    created_at,
                    billed_paise: billed,
                    paid_paise: 0,
                    open_paise: billed,
                });
            }
        }
        // Explicitly linked settlements first.
        for bill in bills.iter_mut() {
            let linked: i64 = self
                .conn
                .query_row(
                    "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM credit_ledger
                     WHERE customer_id = ?1 AND kind IN ('settle', 'write_off')
                       AND sale_id = ?2",
                    params![customer_id, bill.bill_id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            bill.paid_paise = linked.min(bill.billed_paise);
            bill.open_paise = bill.billed_paise - bill.paid_paise;
        }
        // Unlinked settlements wash over the oldest open bills (FIFO).
        let mut pool: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM credit_ledger
                 WHERE customer_id = ?1 AND kind IN ('settle', 'write_off')
                   AND sale_id IS NULL",
                params![customer_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        for bill in bills.iter_mut() {
            if pool <= 0 {
                break;
            }
            let take = pool.min(bill.open_paise);
            bill.paid_paise += take;
            bill.open_paise -= take;
            pool -= take;
        }
        bills.retain(|b| b.open_paise > 0);
        Ok(bills)
    }

    /// Record money received from a customer against credit.
    ///
    /// - Outstanding debt first (kind `settle`).
    /// - Any excess becomes prepaid advance (kind `advance`, balance goes
    ///   negative = shop holds the customer's money).
    /// - With no debt, the full amount is an advance.
    pub fn settle_credit(
        &mut self,
        customer_id: &str,
        amount_paise: i64,
        pay: PayMethod,
        actor: &StaffMember,
        detail: &str,
    ) -> Result<(), String> {
        self.settle_credit_against(customer_id, amount_paise, pay, actor, detail, None)
    }

    /// Like [`Self::settle_credit`], but the settlement is allocated to
    /// one chosen open bill (bill-wise): the settle row carries the
    /// bill's sale id, capped at that bill's open amount; any remainder
    /// stays unlinked (FIFO) exactly as a plain settlement would.
    #[allow(clippy::too_many_arguments)]
    pub fn settle_credit_against(
        &mut self,
        customer_id: &str,
        amount_paise: i64,
        pay: PayMethod,
        actor: &StaffMember,
        detail: &str,
        against_sale_id: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Record credit settle / advance")?;
        if !actor.role.can_sell() {
            return Err("Not allowed".into());
        }
        if amount_paise <= 0 {
            return Err("Amount must be > 0".into());
        }
        if pay.is_credit() || pay.is_split() {
            return Err("Use Cash, UPI/rail, or Card".into());
        }
        let bal = self.customer_balance(customer_id)?;
        let times = EventTimes::capture(None);
        let detail = if detail.trim().is_empty() {
            format!("{}", pay.as_str())
        } else {
            detail.trim().to_string()
        };
        let outstanding = bal.max(0);
        let settle_part = amount_paise.min(outstanding);
        let advance_part = amount_paise - settle_part;
        if settle_part > 0 {
            // Bill-wise split: the part that fits the chosen bill is
            // linked to it; the rest stays unlinked (FIFO), as before.
            let mut linked: Vec<(Option<String>, i64)> = Vec::new();
            match against_sale_id {
                Some(sale_id) => {
                    let open = self
                        .open_credit_bills(customer_id)?
                        .into_iter()
                        .find(|b| b.bill_id == sale_id)
                        .map(|b| b.open_paise)
                        .unwrap_or(0);
                    let linked_part = settle_part.min(open);
                    if linked_part > 0 {
                        linked.push((Some(sale_id.to_string()), linked_part));
                    }
                    if settle_part > linked_part {
                        linked.push((None, settle_part - linked_part));
                    }
                }
                None => linked.push((None, settle_part)),
            }
            for (sale_link, part) in linked {
                self.conn
                    .execute(
                        "INSERT INTO credit_ledger(
                            id, customer_id, created_at, kind, amount_paise,
                            sale_id, pay_method, actor_id, actor_name, detail,
                            user_at, device_at, server_at
                         ) VALUES(?1, ?2, ?3, 'settle', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                        params![
                            new_id(),
                            customer_id,
                            times.user_at,
                            -part,
                            sale_link,
                            pay.as_str(),
                            actor.id,
                            actor.name,
                            format!("Settlement {detail}"),
                            times.user_at,
                            times.device_at,
                            times.server_at
                        ],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        if advance_part > 0 {
            self.conn
                .execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail,
                        user_at, device_at, server_at
                     ) VALUES(?1, ?2, ?3, 'advance', ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        new_id(),
                        customer_id,
                        times.user_at,
                        -advance_part,
                        pay.as_str(),
                        actor.id,
                        actor.name,
                        format!("Advance {detail}"),
                        times.user_at,
                        times.device_at,
                        times.server_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        self.audit(
            actor,
            if advance_part > 0 && settle_part == 0 {
                "credit.advance"
            } else if advance_part > 0 {
                "credit.settle_advance"
            } else {
                "credit.settle"
            },
            &format!("{customer_id} {} {}", amount_paise, pay.as_str()),
        )?;
        Ok(())
    }

    /// Clear AR without collecting cash (write-off / nil balance).
    /// `amount_paise == 0` means write off the full outstanding.
    pub fn write_off_credit(
        &mut self,
        customer_id: &str,
        amount_paise: i64,
        reason: &str,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Write off credit")?;
        if !actor.is_owner() && !actor.role.can_void_refund() {
            return Err("Only Owner/Manager can write off credit".into());
        }
        let bal = self.customer_balance(customer_id)?;
        let outstanding = bal.max(0);
        if outstanding <= 0 {
            return Err("Nothing to write off — balance is already clear".into());
        }
        let write = if amount_paise <= 0 {
            outstanding
        } else {
            amount_paise.min(outstanding)
        };
        let reason = reason.trim();
        if reason.is_empty() {
            return Err("Write-off needs a reason".into());
        }
        let times = EventTimes::capture(None);
        self.conn
            .execute(
                "INSERT INTO credit_ledger(
                    id, customer_id, created_at, kind, amount_paise,
                    sale_id, pay_method, actor_id, actor_name, detail,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, 'write_off', ?4, NULL, NULL, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    new_id(),
                    customer_id,
                    times.user_at,
                    -write,
                    actor.id,
                    actor.name,
                    reason,
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "credit.write_off",
            &format!("{customer_id} {write} {reason}"),
        )?;
        Ok(write)
    }

    pub fn party_payable_balance(&self, party_id: &str) -> Result<i64, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(amount_paise), 0) FROM payable_ledger WHERE party_id = ?1",
                params![party_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    pub fn outstanding_payable_total(&self) -> Result<i64, String> {
        let parties = self.list_customers()?;
        let mut total = 0i64;
        for p in parties {
            let bal = self.party_payable_balance(&p.id)?;
            if bal > 0 {
                total += bal;
            }
        }
        Ok(total)
    }

    /// Open bills for one creditor (bill-wise AP): every on-account
    /// purchase still open, oldest first. Same allocation rule as AR:
    /// linked settlements first, unlinked wash FIFO.
    pub fn open_payable_bills(&self, party_id: &str) -> Result<Vec<OpenBill>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT purchase_id, MIN(created_at), SUM(amount_paise) FROM payable_ledger
                 WHERE party_id = ?1 AND kind = 'purchase' AND purchase_id IS NOT NULL
                 GROUP BY purchase_id ORDER BY MIN(created_at), purchase_id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![party_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut bills: Vec<OpenBill> = Vec::new();
        for row in rows {
            let (bill_id, created_at, billed) = row.map_err(|e| e.to_string())?;
            if billed > 0 {
                bills.push(OpenBill {
                    bill_id,
                    created_at,
                    billed_paise: billed,
                    paid_paise: 0,
                    open_paise: billed,
                });
            }
        }
        for bill in bills.iter_mut() {
            let linked: i64 = self
                .conn
                .query_row(
                    "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM payable_ledger
                     WHERE party_id = ?1 AND kind IN ('settle', 'write_off')
                       AND purchase_id = ?2",
                    params![party_id, bill.bill_id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            bill.paid_paise = linked.min(bill.billed_paise);
            bill.open_paise = bill.billed_paise - bill.paid_paise;
        }
        let mut pool: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM payable_ledger
                 WHERE party_id = ?1 AND kind IN ('settle', 'write_off')
                   AND purchase_id IS NULL",
                params![party_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        for bill in bills.iter_mut() {
            if pool <= 0 {
                break;
            }
            let take = pool.min(bill.open_paise);
            bill.paid_paise += take;
            bill.open_paise -= take;
            pool -= take;
        }
        bills.retain(|b| b.open_paise > 0);
        Ok(bills)
    }

    /// Pay down AP (we owed them). Excess is not tracked as receivable here.
    pub fn settle_payable(
        &mut self,
        party_id: &str,
        amount_paise: i64,
        pay: PayMethod,
        actor: &StaffMember,
        detail: &str,
    ) -> Result<(), String> {
        self.settle_payable_against(party_id, amount_paise, pay, actor, detail, None)
    }

    /// Like [`Self::settle_payable`], allocated to one chosen open
    /// purchase bill (capped at its open amount; remainder unlinked).
    #[allow(clippy::too_many_arguments)]
    pub fn settle_payable_against(
        &mut self,
        party_id: &str,
        amount_paise: i64,
        pay: PayMethod,
        actor: &StaffMember,
        detail: &str,
        against_purchase_id: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Settle payable")?;
        if amount_paise <= 0 {
            return Err("Amount must be > 0".into());
        }
        if pay.is_credit() || pay.is_split() {
            return Err("Use Cash, UPI/rail, or Card".into());
        }
        let owed = self.party_payable_balance(party_id)?.max(0);
        if owed <= 0 {
            return Err("Nothing payable for this party".into());
        }
        let settle = amount_paise.min(owed);
        let times = EventTimes::capture(None);
        let detail = if detail.trim().is_empty() {
            pay.as_str().to_string()
        } else {
            detail.trim().to_string()
        };
        // Bill-wise split, mirroring settle_credit_against.
        let mut linked: Vec<(Option<String>, i64)> = Vec::new();
        match against_purchase_id {
            Some(purchase_id) => {
                let open = self
                    .open_payable_bills(party_id)?
                    .into_iter()
                    .find(|b| b.bill_id == purchase_id)
                    .map(|b| b.open_paise)
                    .unwrap_or(0);
                let linked_part = settle.min(open);
                if linked_part > 0 {
                    linked.push((Some(purchase_id.to_string()), linked_part));
                }
                if settle > linked_part {
                    linked.push((None, settle - linked_part));
                }
            }
            None => linked.push((None, settle)),
        }
        for (purchase_link, part) in linked {
            self.conn
                .execute(
                    "INSERT INTO payable_ledger(
                        id, party_id, created_at, kind, amount_paise, purchase_id,
                        pay_method, actor_id, actor_name, detail,
                        user_at, device_at, server_at
                     ) VALUES(?1, ?2, ?3, 'settle', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        new_id(),
                        party_id,
                        times.user_at,
                        -part,
                        purchase_link,
                        pay.as_str(),
                        actor.id,
                        actor.name,
                        format!("Payable settle {detail}"),
                        times.user_at,
                        times.device_at,
                        times.server_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        self.audit(
            actor,
            "payable.settle",
            &format!("{party_id} {settle} {}", pay.as_str()),
        )?;
        Ok(())
    }

    pub fn write_off_payable(
        &mut self,
        party_id: &str,
        amount_paise: i64,
        reason: &str,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Write off payable")?;
        if !actor.is_owner() && !actor.role.can_void_refund() {
            return Err("Only Owner/Manager can write off payables".into());
        }
        let owed = self.party_payable_balance(party_id)?.max(0);
        if owed <= 0 {
            return Err("Nothing to write off".into());
        }
        let write = if amount_paise <= 0 {
            owed
        } else {
            amount_paise.min(owed)
        };
        let reason = reason.trim();
        if reason.is_empty() {
            return Err("Write-off needs a reason".into());
        }
        let times = EventTimes::capture(None);
        self.conn
            .execute(
                "INSERT INTO payable_ledger(
                    id, party_id, created_at, kind, amount_paise, purchase_id,
                    pay_method, actor_id, actor_name, detail,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, 'write_off', ?4, NULL, NULL, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    new_id(),
                    party_id,
                    times.user_at,
                    -write,
                    actor.id,
                    actor.name,
                    reason,
                    times.user_at,
                    times.device_at,
                    times.server_at
                ],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "payable.write_off",
            &format!("{party_id} {write} {reason}"),
        )?;
        Ok(write)
    }

    pub fn payable_aging_text(&self) -> Result<String, String> {
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let mut out = String::from("Whom we owe (AP)\n");
        let mut any = false;
        for party in self.list_customers()? {
            let bal = self.party_payable_balance(&party.id)?;
            if bal <= 0 {
                continue;
            }
            any = true;
            let kind = if party.kind_label.trim().is_empty() {
                String::new()
            } else {
                format!(" · {}", party.kind_label.trim())
            };
            out.push_str(&format!(
                "{}{} — {}\n",
                party.name,
                kind,
                Money(bal).format_localized(&currency, &fiscal)
            ));
        }
        if !any {
            out.push_str("No open payables.\n");
        } else {
            out.push_str(&format!(
                "Total we owe {}\n",
                Money(self.outstanding_payable_total()?).format_localized(&currency, &fiscal)
            ));
        }
        Ok(out)
    }

    pub fn list_units(&self) -> Result<Vec<UnitDef>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, code, name, builtin, picker_enabled FROM units
                 ORDER BY builtin DESC, code",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(UnitDef {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    builtin: row.get::<_, i64>(3)? != 0,
                    picker_enabled: row.get::<_, i64>(4)? != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Units the shop turned on for Buy / Menu / conversion pickers.
    pub fn list_picker_units(&self) -> Result<Vec<UnitDef>, String> {
        Ok(self
            .list_units()?
            .into_iter()
            .filter(|u| u.picker_enabled)
            .collect())
    }

    /// Owner turns a unit on or off for pick-lists. Conversions still use it.
    pub fn set_unit_picker_enabled(
        &self,
        code: &str,
        enabled: bool,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change unit pickers")?;
        if !actor.is_owner() {
            return Err("Only the Owner can change which units appear in pickers".into());
        }
        let code = code.trim().to_ascii_lowercase();
        let n = self
            .conn
            .execute(
                "UPDATE units SET picker_enabled = ?1 WHERE code = ?2",
                params![if enabled { 1 } else { 0 }, code],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err(format!("Unknown unit '{code}'"));
        }
        self.audit(
            actor,
            "unit.picker",
            &format!("{code}={}", if enabled { "on" } else { "off" }),
        )?;
        Ok(())
    }

    pub fn add_unit(&self, code: &str, name: &str) -> Result<UnitDef, String> {
        self.assert_server_writer("Add unit")?;
        let code = code.trim().to_ascii_lowercase();
        let name = name.trim();
        if code.is_empty() || name.is_empty() {
            return Err("Unit code and name required".into());
        }
        // Currencies are not units: an exchange rate is a price at a
        // moment in time, not a fixed conversion, so money never enters
        // the unit tables.
        if crate::locale::currency_by_code(&code.to_ascii_uppercase()).is_some() {
            return Err(format!(
                "'{code}' is a currency, not a unit of measure — money is never converted like weight or length"
            ));
        }
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO units(id, code, name, builtin, picker_enabled) VALUES(?1, ?2, ?3, 0, 1)",
                params![id, code, name],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE") {
                    "Unit code already exists".into()
                } else {
                    e.to_string()
                }
            })?;
        Ok(UnitDef {
            id,
            code,
            name: name.into(),
            builtin: false,
            picker_enabled: true,
        })
    }

    pub fn list_unit_conversions(&self) -> Result<Vec<UnitConversion>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, from_code, to_code, factor_milli FROM unit_conversions
                 ORDER BY from_code, to_code",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(UnitConversion {
                    id: row.get(0)?,
                    from_code: row.get(1)?,
                    to_code: row.get(2)?,
                    factor_milli: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// `1 from = (factor_milli/1000) to` in milli-qty space.
    pub fn add_unit_conversion(
        &self,
        from_code: &str,
        to_code: &str,
        factor_milli: i64,
    ) -> Result<UnitConversion, String> {
        self.assert_server_writer("Add unit conversion")?;
        let from_code = from_code.trim().to_ascii_lowercase();
        let to_code = to_code.trim().to_ascii_lowercase();
        if from_code.is_empty() || to_code.is_empty() || from_code == to_code {
            return Err("Need two different unit codes".into());
        }
        if factor_milli <= 0 {
            return Err("Conversion factor must be > 0".into());
        }
        // A conversion links two known units of the same kind of
        // measure, and the link must keep every chain ending at a
        // standard unit — no dangling cross-family links, no loops.
        for code in [&from_code, &to_code] {
            let known: bool = self
                .conn
                .query_row("SELECT 1 FROM units WHERE code = ?1", params![code], |_| {
                    Ok(true)
                })
                .optional()
                .map_err(|e| e.to_string())?
                .unwrap_or(false);
            if !known {
                return Err(format!(
                    "Unit '{code}' does not exist — add the unit first"
                ));
            }
        }
        if crate::domain::standard_unit_family(&from_code).is_some() {
            return Err(format!(
                "'{from_code}' is a standard unit — its conversions are fixed and automatic. Define the other unit instead (e.g. 1 bag = 25 kg)"
            ));
        }
        let from_family = self.resolve_unit(&from_code).map(|(family, _)| family);
        let to_family = self.resolve_unit(&to_code).map(|(family, _)| family);
        if let (Some(a), Some(b)) = (&from_family, &to_family) {
            if a != b {
                return Err(format!(
                    "Cannot link {from_code} ({a}) to {to_code} ({b}) — different kinds of measure"
                ));
            }
        }
        if self.unit_reaches(&to_code, &from_code) {
            return Err(format!(
                "That would make a loop — '{to_code}' already leads back to '{from_code}'"
            ));
        }
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO unit_conversions(id, from_code, to_code, factor_milli)
                 VALUES(?1, ?2, ?3, ?4)",
                params![id, from_code, to_code, factor_milli],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE") {
                    "Conversion already exists".into()
                } else {
                    e.to_string()
                }
            })?;
        Ok(UnitConversion {
            id,
            from_code,
            to_code,
            factor_milli,
        })
    }

    /// Convert between two units of the same family by way of the
    /// family anchor (`qty × factor(from) ÷ factor(to)`), so chains and
    /// reverse directions work without hand-entered reverse rows and
    /// never drift on rounded factors.
    pub fn convert_qty_milli(
        &self,
        qty_milli: i64,
        from_code: &str,
        to_code: &str,
    ) -> Result<i64, String> {
        use rust_decimal::prelude::ToPrimitive;
        let from_code = from_code.trim().to_ascii_lowercase();
        let to_code = to_code.trim().to_ascii_lowercase();
        if from_code == to_code {
            return Ok(qty_milli);
        }
        let (from_family, from_factor) = self.resolve_unit(&from_code).ok_or_else(|| {
            format!("Unit '{from_code}' is not linked to a standard unit yet")
        })?;
        let (to_family, to_factor) = self.resolve_unit(&to_code).ok_or_else(|| {
            format!("Unit '{to_code}' is not linked to a standard unit yet")
        })?;
        if from_family != to_family {
            return Err(format!(
                "Cannot convert {from_code} ({from_family}) to {to_code} ({to_family}) — different kinds of measure"
            ));
        }
        let out = (rust_decimal::Decimal::from(qty_milli) * from_factor / to_factor).round();
        out.to_i64()
            .ok_or_else(|| "Converted quantity is too large".to_string())
    }

    /// Resolve a unit to `(family, factor converting 1 unit into the
    /// family anchor)` by walking its chain of conversions until a
    /// standard unit is reached — a custom unit may be based on another
    /// custom unit, as long as the chain ultimately lands on a standard.
    /// `None` when the chain dangles (never reaches a standard) or loops.
    pub fn resolve_unit(
        &self,
        code: &str,
    ) -> Option<(String, rust_decimal::Decimal)> {
        let code = code.trim().to_ascii_lowercase();
        self.resolve_unit_walk(&code, &mut std::collections::HashSet::new())
    }

    fn resolve_unit_walk(
        &self,
        code: &str,
        seen: &mut std::collections::HashSet<String>,
    ) -> Option<(String, rust_decimal::Decimal)> {
        if let Some((family, factor)) = crate::domain::standard_unit_family(code) {
            return Some((family.to_string(), factor));
        }
        if !seen.insert(code.to_string()) {
            return None; // loop in the chain — can never reach a standard
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT to_code, factor_milli FROM unit_conversions
                 WHERE from_code = ?1 ORDER BY to_code",
            )
            .ok()?;
        let rows = stmt
            .query_map(params![code], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .ok()?;
        let rows: Vec<(String, i64)> = rows.collect::<Result<_, _>>().ok()?;
        for (to_code, factor_milli) in rows {
            let step =
                rust_decimal::Decimal::from(factor_milli) / rust_decimal::Decimal::from(1000);
            let mut branch = seen.clone();
            if let Some((family, rest)) = self.resolve_unit_walk(&to_code, &mut branch) {
                return Some((family, step * rest));
            }
        }
        None
    }

    /// Can `from` already reach `to` by following conversion rows?
    /// Used to refuse a new row that would close a loop.
    fn unit_reaches(&self, from: &str, target: &str) -> bool {
        let mut seen = std::collections::HashSet::new();
        let mut stack = vec![from.to_string()];
        while let Some(code) = stack.pop() {
            if code == target {
                return true;
            }
            if !seen.insert(code.clone()) {
                continue;
            }
            if let Ok(mut stmt) = self.conn.prepare(
                "SELECT to_code FROM unit_conversions WHERE from_code = ?1",
            ) {
                if let Ok(rows) = stmt.query_map(params![code], |row| row.get::<_, String>(0)) {
                    for to in rows.flatten() {
                        stack.push(to);
                    }
                }
            }
        }
        false
    }

    /// Factor converting 1 `code` into centimetres, following the
    /// unit's chain to the length anchor. `None` when the unit is not
    /// a length or its chain never reaches a standard unit.
    pub fn factor_to_cm(&self, code: &str) -> Option<rust_decimal::Decimal> {
        self.resolve_unit(code)
            .filter(|(family, _)| family == "length")
            .map(|(_, factor)| factor)
    }

    /// Factor converting 1 `code` into kilograms; see [`Self::factor_to_cm`].
    pub fn factor_to_kg(&self, code: &str) -> Option<rust_decimal::Decimal> {
        self.resolve_unit(code)
            .filter(|(family, _)| family == "weight")
            .map(|(_, factor)| factor)
    }

    /// Honest Books home: money in/out, AR/AP, cash vs bank, period profit shape.
    /// Today's books figures as separate values, so the Books home can lay
    /// them out as stat cards instead of one paragraph of text.
    pub fn books_overview(&self) -> Result<BooksOverview, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let books = self.period_books(&today, &today)?;
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let tmpl = self.entity_template()?;
        let fmt = |paise: i64| Money(paise).format_localized(&currency, &fiscal);
        Ok(BooksOverview {
            money_in: fmt(books.revenue_paise),
            purchases: fmt(books.cogs_paise),
            expenses: fmt(books.expenses_paise),
            gross: fmt(books.gross_paise()),
            net: fmt(books.net_paise()),
            lane_declared_label: tmpl.lane_declared_label.to_string(),
            lane_declared: fmt(books.declared_paise),
            lane_owner_label: tmpl.lane_owner_label.to_string(),
            lane_owner: fmt(books.owner_lane_paise),
            cash: fmt(books.cash_paise),
            digital_label: fiscal.digital_pay_label().to_string(),
            digital: fmt(books.upi_paise),
            card: fmt(books.card_paise),
            they_owe: fmt(books.outstanding_credit_paise),
            we_owe: fmt(books.outstanding_payable_paise),
            bank_balance: fmt(self.bank_balance_paise()?),
            bank_net_today: fmt(books.bank_in_paise.saturating_sub(books.bank_out_paise)),
        })
    }

    pub fn books_overview_text(&self) -> Result<String, String> {
        let o = self.books_overview()?;
        Ok(format!(
            "Today — books at a glance\n\
             Money in (sales) {}\n\
             Money out — purchases (COGS) {} · expenses {}\n\
             Gross {} · Net {}\n\
             {} {} · {} {}\n\
             Cash {} · {} {} · Card {}\n\
             They owe us {}\n\
             We owe them {}\n\
             Bank balance {}\n\
             Bank net today {}\n",
            o.money_in,
            o.purchases,
            o.expenses,
            o.gross,
            o.net,
            o.lane_declared_label,
            o.lane_declared,
            o.lane_owner_label,
            o.lane_owner,
            o.cash,
            o.digital_label,
            o.digital,
            o.card,
            o.they_owe,
            o.we_owe,
            o.bank_balance,
            o.bank_net_today,
        ))
    }

    pub fn list_credit_ledger(
        &self,
        customer_id: &str,
        limit: i64,
    ) -> Result<Vec<CreditLedgerEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_name, detail,
                        COALESCE(NULLIF(user_at, ''), created_at),
                        COALESCE(NULLIF(device_at, ''), created_at),
                        COALESCE(server_at, '')
                 FROM credit_ledger WHERE customer_id = ?1
                 ORDER BY created_at DESC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![customer_id, limit], |row| {
                Ok(CreditLedgerEntry {
                    id: row.get(0)?,
                    customer_id: row.get(1)?,
                    created_at: row.get(2)?,
                    kind: row.get(3)?,
                    amount_paise: row.get(4)?,
                    sale_id: row.get(5)?,
                    pay_method: row.get(6)?,
                    actor_name: row.get(7)?,
                    detail: row.get(8)?,
                    user_at: row.get(9)?,
                    device_at: row.get(10)?,
                    server_at: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_audit(&self, limit: i64) -> Result<Vec<AuditEvent>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, created_at, actor_id, actor_name, action, detail
                 FROM audit_events ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(AuditEvent {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    actor_id: row.get(2)?,
                    actor_name: row.get(3)?,
                    action: row.get(4)?,
                    detail: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn audit(&self, actor: &StaffMember, action: &str, detail: &str) -> Result<(), String> {
        self.audit_as(&actor.id, &actor.name, action, detail)
    }

    /// An audit line by something that is not a staff member (a device
    /// pairing itself with the shop's code).
    fn audit_as(&self, actor_id: &str, actor_name: &str, action: &str, detail: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    new_id(),
                    Local::now().to_rfc3339(),
                    actor_id,
                    actor_name,
                    action,
                    detail
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Tests that seed months of trading: write without waiting on the disk.
    #[cfg(test)]
    pub(crate) fn fast_writes_for_test(&self) {
        self.conn.execute_batch("PRAGMA synchronous = OFF").unwrap();
    }

    /// Tests: move a kitchen ticket to an earlier time.
    #[cfg(test)]
    pub(crate) fn backdate_kot_for_test(&self, kot_id: &str, at: &str) {
        self.conn
            .execute("UPDATE kot_tickets SET created_at = ?1 WHERE id = ?2", params![at, kot_id])
            .unwrap();
    }

    /// Tests that seed months of trading: move a bill (and its kitchen
    /// tickets) to an earlier day.
    #[cfg(test)]
    pub(crate) fn backdate_sale_for_test(&self, sale_id: &str, at: &str) {
        self.conn
            .execute(
                "UPDATE sales SET created_at = ?1, user_at = ?1, device_at = ?1 WHERE id = ?2",
                params![at, sale_id],
            )
            .unwrap();
        self.conn
            .execute("UPDATE kot_tickets SET created_at = ?1 WHERE sale_id = ?2", params![at, sale_id])
            .unwrap();
    }

    /// Write a backup to `dest` and read it back: a file that does not pass
    /// the same check a restore makes (a full disk, a failing USB stick) is
    /// deleted and reported, never left looking like a backup.
    pub fn backup_to(&self, dest: &Path) -> Result<(), String> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let written = self
            .conn
            .execute("VACUUM INTO ?1", params![dest.to_string_lossy().as_ref()])
            .map_err(|e| format!("Could not write the backup to {}: {e}", dest.display()))
            .and_then(|_| check_backup_file(dest));
        if written.is_err() {
            let _ = fs::remove_file(dest);
        }
        written
    }

    /// Back up once a day — the first time the books are open 20 hours
    /// after the last one — into `backups/` beside the books, and into the
    /// owner's second folder (USB stick, synced folder) when one is set.
    /// Keeps the last [`AUTO_BACKUP_KEEP`]. Returns the new file, if made.
    pub fn auto_backup_if_due(&self) -> Result<Option<PathBuf>, String> {
        let now = Local::now();
        if let Some(last) = self
            .meta_get("last_backup_at")?
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
            && now.signed_duration_since(last) < chrono::Duration::hours(20)
        {
            return Ok(None);
        }
        let dir = self.auto_backup_dir();
        let file = dir.join(format!("auto-{}.sqlite3", now.format("%Y%m%d-%H%M%S%3f")));
        if let Err(e) = self.backup_to(&file) {
            // Said on the owner's screens until a backup works again.
            self.meta_set("backup_error", &e)?;
            return Err(e);
        }
        self.meta_set("backup_error", "")?;
        self.meta_set("last_backup_at", &now.to_rfc3339())?;
        prune_auto_backups(&dir, AUTO_BACKUP_KEEP);
        if let Some(extra) = self.backup_copy_dir()? {
            let copied = fs::create_dir_all(&extra)
                .and_then(|()| fs::copy(&file, extra.join(file.file_name().unwrap_or_default())))
                .map_err(|e| format!("Could not copy the backup to {}: {e}", extra.display()));
            match copied {
                Ok(_) => {
                    prune_auto_backups(&extra, AUTO_BACKUP_KEEP);
                    self.meta_set("backup_copy_error", "")?;
                    self.note_backup_copy_saved()?;
                }
                Err(e) => self.meta_set("backup_copy_error", &e)?,
            }
        }
        Ok(Some(file))
    }

    /// Where the automatic backups of these books go.
    pub fn auto_backup_dir(&self) -> PathBuf {
        self.path.parent().unwrap_or(Path::new(".")).join("backups")
    }

    pub fn last_backup_at(&self) -> Result<Option<String>, String> {
        Ok(self.meta_get("last_backup_at")?.filter(|s| !s.is_empty()))
    }

    /// The owner's second place for backups, if set.
    pub fn backup_copy_dir(&self) -> Result<Option<PathBuf>, String> {
        Ok(self.meta_get("backup_copy_dir")?.filter(|s| !s.trim().is_empty()).map(PathBuf::from))
    }

    pub fn set_backup_copy_dir(&self, dir: Option<&Path>) -> Result<(), String> {
        self.meta_set("backup_copy_dir", &dir.map(|d| d.to_string_lossy().into_owned()).unwrap_or_default())?;
        self.meta_set("backup_copy_error", "")
    }

    /// A copy of the books left this device (the second folder, or one the
    /// owner saved somewhere with "Save a backup copy").
    pub fn note_backup_copy_saved(&self) -> Result<(), String> {
        self.meta_set("backup_copy_at", &Local::now().to_rfc3339())
    }

    /// Days with no copy off this device, once that is [`COPY_DUE_DAYS`] or
    /// more: counted from the last copy, or — never copied — from the oldest
    /// backup still kept (how long the books have been here). A device that
    /// is lost or breaks takes every backup kept on it along.
    pub fn backup_copy_overdue_days(&self) -> Result<Option<i64>, String> {
        let now = Local::now();
        let since = match self.meta_get("backup_copy_at")?.filter(|s| !s.is_empty()) {
            Some(at) => chrono::DateTime::parse_from_rfc3339(&at).ok().map(|t| t.with_timezone(&Local)),
            None => fs::read_dir(self.auto_backup_dir())
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("auto-"))
                .filter_map(|e| e.metadata().ok()?.modified().ok())
                .min()
                .map(chrono::DateTime::<Local>::from),
        };
        Ok(since
            .map(|t| now.signed_duration_since(t).num_days())
            .filter(|days| *days >= COPY_DUE_DAYS))
    }

    /// Why the last copy to the second folder failed ("" = it did not).
    pub fn backup_copy_error(&self) -> Result<String, String> {
        Ok(self.meta_get("backup_copy_error")?.unwrap_or_default())
    }

    /// What the owner must know about keeping these books safe: a failing
    /// backup, no backup for days, or a disk filling up.
    pub fn backup_health(&self) -> Result<BackupHealth, String> {
        let last_at = self.last_backup_at()?;
        let days_since = last_at
            .as_deref()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .map(|t| Local::now().signed_duration_since(t).num_days());
        let books_mb = fs::metadata(&self.path).map(|m| m.len() / 1_048_576).unwrap_or(0);
        let free_mb = self.path.parent().and_then(free_space_bytes).map(|b| b / 1_048_576);
        Ok(BackupHealth {
            last_at,
            days_since,
            error: self.meta_get("backup_error")?.unwrap_or_default(),
            copy_error: self.backup_copy_error()?,
            free_mb,
            low_space: free_mb.is_some_and(|free| free < LOW_SPACE_MB.max(books_mb * 3)),
            copy_overdue_days: self.backup_copy_overdue_days()?,
        })
    }

    /// Restore into the active local server DB (Server Mode).
    pub fn restore_from(backup: &Path) -> Result<Self, String> {
        let dir = data_dir()?;
        let id = crate::local_servers::active_server_id(&dir)?
            .ok_or_else(|| "No local server to restore into".to_string())?;
        Self::restore_from_to_path(backup, &crate::local_servers::server_db_path(&dir, &id))
    }

    /// Replace the DB at `dest` with `backup` and reopen it. The caller must
    /// have dropped any [`Store`] on `dest` first.
    pub fn restore_from_to_path(backup: &Path, dest: &Path) -> Result<Self, String> {
        if !backup.exists() {
            return Err(format!("No backup at {}", backup.display()));
        }
        // A wrong or damaged file must never replace working books.
        check_backup_file(backup)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        // Copy to a sibling temp then rename, so a half-written restore can
        // never replace a working till DB.
        let tmp = dest.with_extension("restore.tmp");
        fs::copy(backup, &tmp).map_err(|e| e.to_string())?;
        fs::rename(&tmp, dest).map_err(|e| e.to_string())?;
        // `backup_to` writes a single self-contained file, so any -wal/-shm left
        // over from the replaced DB now describes pages that no longer exist.
        // Leaving them made SQLite replay a stale journal over fresh rows.
        for sidecar in ["-wal", "-shm"] {
            let mut name = dest.as_os_str().to_os_string();
            name.push(sidecar);
            let _ = fs::remove_file(PathBuf::from(name));
        }
        Self::open_path(dest)
    }

    /// Open the DB under Makepad's app data directory (`Cx::get_data_dir`,
    /// Android Activity.getFilesDir). That path is already app-scoped
    /// (`KUMKAJ_APP_ID`); do not nest a second vendor folder.
    pub fn open_in_data_dir(base: &Path) -> Result<Self, String> {
        fs::create_dir_all(base).map_err(|e| e.to_string())?;
        crate::local_servers::open_active_server(base)
    }

    // —— Outbound notification outbox ——


    pub fn queue_notify(&self, row: &crate::notify::NotifyOutboxRow) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO notify_outbox(
                    id, created_at, channel, to_addr, template_key, payload_json, status, last_error
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    row.id,
                    row.created_at,
                    row.channel,
                    row.to_addr,
                    row.template_key,
                    row.payload_json,
                    row.status,
                    row.last_error
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_notify_outbox(&self, limit: i64) -> Result<Vec<crate::notify::NotifyOutboxRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, created_at, channel, to_addr, template_key, payload_json, status, last_error
                 FROM notify_outbox ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit.max(1)], |r| {
                Ok(crate::notify::NotifyOutboxRow {
                    id: r.get(0)?,
                    created_at: r.get(1)?,
                    channel: r.get(2)?,
                    to_addr: r.get(3)?,
                    template_key: r.get(4)?,
                    payload_json: r.get(5)?,
                    status: r.get(6)?,
                    last_error: r.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

}

/// Reverse-DNS app id from this crate's `env/development.env` or `env/production.env`
/// (baked by `build.rs`). Dev: `com.khaynjo.debug`. Prod: `com.khaynjo`.
pub const APP_ID: &str = env!("KUMKAJ_APP_ID");
/// Launcher / window label from the same env file.
pub const APP_LABEL: &str = env!("KUMKAJ_APP_LABEL");

/// What reports call a business whose owner never named it.
pub const DEFAULT_ORG_NAME: &str = "My business";

/// "Server " + 8 hex digits: the label a local server gets when it is made
/// without a name (`local_servers::create_local_server`).
pub fn is_generated_server_label(name: &str) -> bool {
    name.trim()
        .strip_prefix("Server ")
        .is_some_and(|id| id.len() == 8 && id.chars().all(|c| c.is_ascii_hexdigit()))
}
/// `development` or `production`.
pub const APP_ENV: &str = env!("KUMKAJ_ENV");
/// Show the live W×H badge (development.env sets this to 1).
pub const SHOW_SIZE_BADGE: bool = const_str_eq(env!("KUMKAJ_SHOW_SIZE_BADGE"), "1");

const fn const_str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// What [`Store::import_catalogue`] did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CatalogueImport {
    pub categories_added: usize,
    pub items_added: usize,
    /// Dishes the shop already had by that name (left as they are).
    pub items_kept: usize,
}

/// Every column [`row_to_product`] expects, in order.
/// Longest edge a menu item / category photo is stored at (it is only ever
/// shown as a small thumb; the original stays wherever the owner picked it).
pub const ITEM_PHOTO_EDGE: u32 = 768;

const PRODUCT_COLUMNS: &str = "id, name, price_paise, available, archived, image_key,
     COALESCE(category_id, ''), COALESCE(station_id, ''), COALESCE(tax_bps, 0),
     COALESCE(stock_qty, -1), COALESCE(modifiers_json, '[]'), COALESCE(branch_id, ''),
     COALESCE(tax_rate_id, ''), COALESCE(aliases_json, '[]'), COALESCE(unit_label, ''),
     COALESCE(group_name, ''), COALESCE(area_prices_json, '[]'), COALESCE(kind, 'dish'),
     COALESCE(schedule_json, '[]'), COALESCE(allergens, '')";

/// Every column [`row_to_sale`] expects, in order.
const SALE_COLUMNS: &str = "id, created_at, total_paise,
     COALESCE(lines_total_paise, total_paise),
     COALESCE(rounding_paise, 0),
     pay_method, status, table_id, customer_id,
     COALESCE(created_by_id, ''), COALESCE(created_by_name, ''),
     COALESCE(branch_id, ''), COALESCE(order_type, 'counter'), COALESCE(tax_paise, 0),
     COALESCE(tax_pricing, 'inclusive'), COALESCE(tax_label, 'Tax'), COALESCE(net_paise, 0),
     COALESCE(service_paise, 0), COALESCE(invoice_no, ''),
     COALESCE(money_lane, 'declared'),
     COALESCE(NULLIF(user_at, ''), created_at),
     COALESCE(NULLIF(device_at, ''), created_at),
     COALESCE(server_at, ''),
     COALESCE(extras_json, '[]')";

/// Put a reversed sale's tracked units back on the shelf and leave a movement
/// behind. Without this, undoing a mistaken charge would strand stock the shop
/// still has — and now that overselling is refused outright, that stock would
/// be unsellable. Untracked items (`stock_qty` = −1) are skipped.
fn restore_stock_for_sale(
    tx: &Connection,
    sale_id: &str,
    reason: &str,
    actor: &StaffMember,
    at: &str,
) -> Result<(), String> {
    let branch_id: String = tx
        .query_row(
            "SELECT COALESCE(branch_id, '') FROM sales WHERE id = ?1",
            params![sale_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let sold: Vec<(String, i64)> = {
        let mut stmt = tx
            .prepare(
                "SELECT product_id, SUM(qty) FROM sale_lines
                 WHERE sale_id = ?1 GROUP BY product_id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![sale_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    // What the dishes used from the shelf goes back too.
    let returned: Vec<(String, i64)> = sold.iter().map(|(id, q)| (id.clone(), -q)).collect();
    recipes::apply_recipe_usage(tx, &returned, "recipe_return", sale_id, &branch_id, actor, at)?;
    for (product_id, qty) in sold {
        if qty <= 0 {
            continue;
        }
        let moved = tx
            .execute(
                "UPDATE products SET stock_qty = stock_qty + ?1
                 WHERE id = ?2 AND stock_qty >= 0",
                params![qty, product_id],
            )
            .map_err(|e| e.to_string())?;
        // 0 rows means the item is untracked or no longer in the catalogue —
        // there is no count to put anything back into.
        if moved == 0 {
            continue;
        }
        tx.execute(
            "INSERT INTO stock_movements(
                id, branch_id, product_id, created_at, delta_qty,
                reason, note, actor_id, actor_name, ref_id
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, '', ?7, ?8, ?9)",
            params![
                new_id(),
                branch_id,
                product_id,
                at,
                qty,
                reason,
                actor.id,
                actor.name,
                sale_id
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Move tracked stock by the difference between what a sale used to hold and
/// what the amendment says it holds now, leaving an `amend` movement behind.
/// Untracked items (`stock_qty` = −1) keep no count, so they are skipped.
///
/// Returns an error — which rolls the whole amendment back — when a raised line
/// would take the shelf below zero. Amending is how a mis-rung bill gets fixed,
/// and a fix must not be able to invent stock.
fn apply_amend_stock(
    tx: &Connection,
    sale_id: &str,
    previous: &Sale,
    draft: &SaleDraft,
    actor: &StaffMember,
    at: &str,
) -> Result<(), String> {
    let mut deltas: Vec<(String, i64)> = Vec::new();
    let mut bump = |product_id: &str, qty: i64| {
        match deltas.iter_mut().find(|(id, _)| id == product_id) {
            Some((_, sum)) => *sum = sum.saturating_add(qty),
            None => deltas.push((product_id.to_string(), qty)),
        }
    };
    // Positive = goes back on the shelf, negative = leaves the shelf.
    for line in &previous.lines {
        bump(&line.product_id, line.qty);
    }
    for line in &draft.lines {
        bump(&line.product_id, -line.qty);
    }

    let branch_id = previous.branch_id.clone();
    // Recipes follow the portions: a removed dish's usage goes back.
    let portions: Vec<(String, i64)> = deltas.iter().map(|(id, d)| (id.clone(), -d)).collect();
    recipes::apply_recipe_usage(tx, &portions, "recipe_amend", sale_id, &branch_id, actor, at)?;
    for (product_id, delta) in deltas {
        if delta == 0 {
            continue;
        }
        let tracked: Option<(String, i64)> = tx
            .query_row(
                "SELECT name, stock_qty FROM products WHERE id = ?1",
                params![product_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((name, on_hand)) = tracked else {
            continue;
        };
        if on_hand < 0 {
            continue;
        }
        let after = on_hand.saturating_add(delta);
        if after < 0 {
            return Err(format!(
                "Not enough stock for {name}: {on_hand} on hand, amend needs {}",
                delta.saturating_abs()
            ));
        }
        tx.execute(
            "UPDATE products SET stock_qty = ?1 WHERE id = ?2",
            params![after, product_id],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO stock_movements(
                id, branch_id, product_id, created_at, delta_qty,
                reason, note, actor_id, actor_name, ref_id
             ) VALUES(?1, ?2, ?3, ?4, ?5, 'amend', '', ?6, ?7, ?8)",
            params![
                new_id(),
                branch_id,
                product_id,
                at,
                delta,
                actor.id,
                actor.name,
                sale_id
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Take the shop's next invoice number and advance the counter. Kept as a free
/// function so it can run on the sale's own transaction handle.
fn reserve_invoice_no(tx: &Connection, branch_code: &str) -> Result<String, String> {
    let seq: i64 = tx
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM meta WHERE key = 'next_invoice_seq'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or(1)
        .max(1);
    tx.execute(
        "INSERT INTO meta(key, value) VALUES('next_invoice_seq', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![(seq + 1).to_string()],
    )
    .map_err(|e| e.to_string())?;
    let code = branch_code.trim().to_uppercase();
    let code = if code.is_empty() { "MAIN" } else { code.as_str() };
    Ok(format!("INV-{code}-{seq:06}"))
}


/// One CSV cell, quoted when it holds a comma, quote, or newline.
fn csv_cell(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Paise → major units with 2 decimals, for spreadsheet amounts.
fn csv_money(paise: i64) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let a = paise.abs();
    format!("{sign}{}.{:02}", a / 100, a % 100)
}

/// Minimal CSV reader: quoted cells, `""` escapes, CRLF tolerant.
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    let mut any = false;
    while let Some(c) = chars.next() {
        any = true;
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cell.push('"');
                } else {
                    quoted = false;
                }
            } else {
                cell.push(c);
            }
        } else {
            match c {
                '"' if cell.is_empty() => quoted = true,
                ',' => {
                    row.push(std::mem::take(&mut cell));
                }
                '\n' => {
                    row.push(std::mem::take(&mut cell));
                    rows.push(std::mem::take(&mut row));
                }
                '\r' => {}
                _ => cell.push(c),
            }
        }
    }
    if any && (!cell.is_empty() || !row.is_empty()) {
        row.push(cell);
        rows.push(row);
    }
    rows.retain(|r| r.iter().any(|c| !c.trim().is_empty()));
    rows
}

fn row_to_sale(row: &rusqlite::Row<'_>) -> rusqlite::Result<Sale> {
    Ok(Sale {
        id: row.get(0)?,
        created_at: row.get(1)?,
        total_paise: row.get(2)?,
        lines_total_paise: row.get(3)?,
        rounding_paise: row.get(4)?,
        pay_method: row.get(5)?,
        status: row.get(6)?,
        table_id: row.get(7)?,
        customer_id: row.get(8)?,
        created_by_id: row.get(9)?,
        created_by_name: row.get(10)?,
        branch_id: row.get(11)?,
        order_type: row.get(12)?,
        tax_paise: row.get(13)?,
        tax_pricing: row.get(14)?,
        tax_label: row.get(15)?,
        net_paise: row.get(16)?,
        service_paise: row.get(17)?,
        invoice_no: row.get(18)?,
        money_lane: row.get(19)?,
        user_at: row.get(20)?,
        device_at: row.get(21)?,
        server_at: row.get(22)?,
        extras_json: row.get(23)?,
        lines: Vec::new(),
    })
}

/// KOT/receipt label for one line: `2×Tea (less sugar)`.
fn kot_line_label(line: &SaleLine) -> String {
    if line.note.is_empty() {
        format!("{}×{}", line.qty, line.name)
    } else {
        format!("{}×{} ({})", line.qty, line.name, line.note)
    }
}

fn row_to_attachment(row: &rusqlite::Row<'_>) -> rusqlite::Result<EntryAttachment> {
    Ok(EntryAttachment {
        id: row.get(0)?,
        entity_kind: row.get(1)?,
        entity_id: row.get(2)?,
        created_at: row.get(3)?,
        user_at: row.get(4)?,
        device_at: row.get(5)?,
        kind: row.get(6)?,
        label: row.get(7)?,
        body: row.get(8)?,
        stored_name: row.get(9)?,
        mime: row.get(10)?,
        bytes: row.get(11)?,
        actor_id: row.get(12)?,
        actor_name: row.get(13)?,
    })
}

fn row_to_product(row: &rusqlite::Row<'_>) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get(0)?,
        name: row.get(1)?,
        price: Money(row.get(2)?),
        available: row.get::<_, i64>(3)? != 0,
        archived: row.get::<_, i64>(4)? != 0,
        image_key: row.get(5)?,
        category_id: row.get(6)?,
        station_id: row.get(7)?,
        tax_bps: row.get(8)?,
        stock_qty: row.get(9)?,
        modifiers_json: row.get(10)?,
        branch_id: row.get(11)?,
        tax_rate_id: row.get(12)?,
        aliases_json: row.get(13)?,
        unit_label: row.get(14)?,
        group_name: row.get(15)?,
        area_prices_json: row.get(16)?,
        kind: row.get(17)?,
        schedule_json: row.get(18)?,
        allergens: row.get(19).unwrap_or_default(),
    })
}

pub fn data_dir() -> Result<PathBuf, String> {
    if let Ok(override_dir) = std::env::var("KUMKAJ_DATA_DIR") {
        return Ok(PathBuf::from(override_dir));
    }
    if let Some(base) = dirs::data_dir() {
        return Ok(base.join(APP_ID));
    }
    // Android: `dirs` needs $HOME, which Makepad apps usually lack. Prefer
    // `Cx::get_data_dir()` (Activity.getFilesDir) via `Store::open_in_data_dir`.
    // This package-id fallback keeps early boot working before Cx exists.
    #[cfg(target_os = "android")]
    {
        return Ok(PathBuf::from(format!("/data/user/0/{APP_ID}/files")));
    }
    #[cfg(not(target_os = "android"))]
    {
        Err("No data directory".to_string())
    }
}

/// Hard floor for desktop window width/height (css px) on every desktop OS.
/// Matches the app_shell layout floor (`Fill{min: 320.}`); Android ignores window sizing.
pub const MIN_DESKTOP_WINDOW_CSS: f64 = 320.0;

/// Below this width the shell uses phone chrome (bottom tabs). Saved geoms narrower
/// than this are ignored on restore so a squeezed session cannot lock desktop
/// into a permanent phone frame.
pub const TABLET_SHELL_BREAKPOINT_CSS: f64 = 720.0;

/// First-launch / invalid-restore desktop default (not a phone frame).
pub const DEFAULT_DESKTOP_WINDOW_W: f64 = 1024.0;
pub const DEFAULT_DESKTOP_WINDOW_H: f64 = 768.0;

/// Last desktop window size/position (`window_geom.json` under [`data_dir`]).
/// Mobile ignores this — the OS owns the surface.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct WindowGeomPrefs {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
}

fn window_geom_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("window_geom.json"))
}

pub fn load_window_geom() -> Option<WindowGeomPrefs> {
    let path = window_geom_path().ok()?;
    let raw = fs::read_to_string(path).ok()?;
    let prefs: WindowGeomPrefs = serde_json::from_str(&raw).ok()?;
    if !prefs.width.is_finite()
        || !prefs.height.is_finite()
        || prefs.width < TABLET_SHELL_BREAKPOINT_CSS
        || prefs.height < MIN_DESKTOP_WINDOW_CSS
        || !prefs.x.is_finite()
        || !prefs.y.is_finite()
    {
        return None;
    }
    Some(prefs)
}

pub fn save_window_geom(prefs: WindowGeomPrefs) {
    if !prefs.width.is_finite()
        || !prefs.height.is_finite()
        || prefs.width < MIN_DESKTOP_WINDOW_CSS
        || prefs.height < MIN_DESKTOP_WINDOW_CSS
        || !prefs.x.is_finite()
        || !prefs.y.is_finite()
    {
        return;
    }
    let Ok(path) = window_geom_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(raw) = serde_json::to_string_pretty(&prefs) {
        let _ = fs::write(path, raw);
    }
}

/// A file is this app's books, whole: SQLite, passes its own check, and
/// holds the books' tables. Checked before a restore replaces anything.
pub fn check_backup_file(path: &Path) -> Result<(), String> {
    let not_books = || format!("{} is not a backup of these books", path.display());
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| not_books())?;
    let check: String = conn
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|_| not_books())?;
    if check != "ok" {
        return Err(format!("{} is damaged ({check}) — pick another backup", path.display()));
    }
    for table in ["meta", "sales", "products", "staff"] {
        let found: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                params![table],
                |r| r.get(0),
            )
            .map_err(|_| not_books())?;
        if found == 0 {
            return Err(not_books());
        }
    }
    // Another Kumkaj app's books have the same tables but are not a
    // restaurant's: never restore them over these.
    let meta = |key: &str| -> String {
        conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| r.get(0))
            .unwrap_or_default()
    };
    let (app, template) = (meta("app"), meta("entity_template"));
    if (!app.is_empty() && app != APP_MARK) || (!template.is_empty() && !template.starts_with("khaynjo")) {
        return Err(format!("{} is another app's backup, not these books", path.display()));
    }
    Ok(())
}

/// Stamped on these books (`meta.app`) so a restore can tell them from
/// another Kumkaj app's books.
const APP_MARK: &str = "khaynjo";

/// What the owner must know about keeping the books safe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackupHealth {
    /// When the last automatic backup was made (RFC 3339), if ever.
    pub last_at: Option<String>,
    /// Whole days since then.
    pub days_since: Option<i64>,
    /// Why the last automatic backup failed ("" = it did not).
    pub error: String,
    /// Why the copy to the second folder failed ("" = it did not).
    pub copy_error: String,
    /// Free space where the books live, in MB (None: the system won't say).
    pub free_mb: Option<u64>,
    /// No copy has left this device for this many days (see
    /// `Store::backup_copy_overdue_days`).
    pub copy_overdue_days: Option<i64>,
    /// Too little room left for the next backups and bills.
    pub low_space: bool,
}

/// Below this much free space (or three times the books' size, if more)
/// the owner is warned before backups start failing.
pub const LOW_SPACE_MB: u64 = 500;

/// Free space on the disk holding `path`, in bytes (None where the system
/// will not say).
pub fn free_space_bytes(path: &Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
        // SAFETY: a valid NUL-terminated path and a zeroed out-struct.
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } != 0 {
            return None;
        }
        #[allow(clippy::unnecessary_cast)]
        Some(stat.f_bavail as u64 * stat.f_frsize as u64)
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut free = 0u64;
        // SAFETY: a valid NUL-terminated wide path; the totals are optional.
        let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, std::ptr::null_mut(), std::ptr::null_mut()) };
        (ok != 0).then_some(free)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        None
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetDiskFreeSpaceExW(dir: *const u16, free_to_caller: *mut u64, total: *mut u64, total_free: *mut u64) -> i32;
}

/// Daily automatic backups kept (two weeks).
pub const AUTO_BACKUP_KEEP: usize = 14;

/// A week with every backup on one device is said to the owner.
pub const COPY_DUE_DAYS: i64 = 7;

/// A kitchen ticket untouched this long is no longer cooking: it leaves the
/// Kitchen screen, the floor colours and the devices' sync. Longer than any
/// service (lunch to late dinner), so nothing still on the pass disappears.
pub const KOT_OPEN_HOURS: i64 = 12;

/// Keep only the newest `keep` automatic backups in `dir` (names sort by time).
fn prune_auto_backups(dir: &Path, keep: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut autos: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            name.starts_with("auto-") && name.ends_with(".sqlite3")
        })
        .collect();
    autos.sort();
    let extra = autos.len().saturating_sub(keep);
    for old in autos.into_iter().take(extra) {
        let _ = fs::remove_file(old);
    }
}

pub fn default_backup_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("backups").join(format!(
        "kumkaj-backup-{}.sqlite3",
        Local::now().format("%Y%m%d-%H%M%S")
    )))
}

/// How long a phone sign-in lasts (a long shift plus a margin).
pub const LAN_SESSION_HOURS: i64 = 16;

/// A paired device's "last seen" is noted at most this often.
pub const LAN_SEEN_EVERY_SECS: i64 = 30;

/// A phone paired to this Server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanDevice {
    pub id: String,
    pub name: String,
    pub paired_at: String,
    pub last_seen_at: String,
}

impl LanDevice {
    fn from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self { id: r.get(0)?, name: r.get(1)?, paired_at: r.get(2)?, last_seen_at: r.get(3)? })
    }
}

/// A phone's write waiting for the Server. `id` is also the request id the
/// Server answers by, so sending it twice does it once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LanQueued {
    pub id: String,
    pub path: String,
    pub body: String,
    /// What the person did, in their words ("KOT for T3").
    pub label: String,
    pub table_id: String,
    pub staff_id: String,
    pub attempts: i64,
    pub last_error: String,
}

/// A waiting write the Server turned down, kept until someone has seen it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LanRefused {
    pub id: String,
    /// What the person did ("KOT for T3").
    pub label: String,
    /// The Server's words.
    pub error: String,
    pub created_at: String,
    pub refused_at: String,
}

fn random_token() -> Result<String, String> {
    let mut bytes = [0u8; 24];
    getrandom_salt(&mut bytes)?;
    Ok(hex::encode(bytes))
}

fn lan_token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(format!("kumkaj-lan|{}", token.trim()).as_bytes()))
}

pub fn hash_pin(pin: &str) -> Result<String, String> {
    // Local device PIN — must stay instant on the UI thread (Argon2 freezes the app).
    let mut salt = [0u8; 16];
    getrandom_salt(&mut salt)?;
    let salt_hex = hex::encode(salt);
    let digest = Sha256::digest(format!("kumkaj-pin|{salt_hex}|{pin}").as_bytes());
    Ok(format!("sha256:{salt_hex}:{}", hex::encode(digest)))
}

pub fn verify_pin(pin: &str, hash: &str) -> Result<bool, String> {
    let Some(rest) = hash.strip_prefix("sha256:") else {
        // Legacy Argon2 (or unknown) — reject without running a multi-second hash.
        // Schema v8 resets all PINs to sha256; unlock with 0000 after update.
        return Ok(false);
    };
    let mut parts = rest.splitn(2, ':');
    let (Some(salt_hex), Some(expected)) = (parts.next(), parts.next()) else {
        return Err("Corrupt PIN hash".into());
    };
    let digest = Sha256::digest(format!("kumkaj-pin|{salt_hex}|{pin}").as_bytes());
    Ok(hex::encode(digest) == expected)
}

fn getrandom_salt(out: &mut [u8]) -> Result<(), String> {
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(out);
    Ok(())
}

#[cfg(test)]
impl Store {
    /// A busy restaurant's books for the performance tests: `days` of
    /// trading at `bills_per_day` (three lines each; every tenth bill split
    /// cash + UPI, every twentieth on a customer's khata and settled later),
    /// a KOT and an audit row per bill, `customers` parties. Raw inserts in
    /// one transaction — seconds, not the minutes real bills would take.
    pub(crate) fn seed_busy_books_for_test(&self, days: i64, bills_per_day: i64, customers: usize) {
        let branch = self.active_branch_id().unwrap();
        let start = Local::now() - chrono::Duration::days(days - 1);
        let tx = self.conn.unchecked_transaction().unwrap();
        {
            let mut product = tx.prepare("INSERT INTO products(id, name, price_paise) VALUES(?1, ?2, ?3)").unwrap();
            let products: Vec<(String, String, i64)> = (0..300)
                .map(|i| (new_id(), format!("Dish {i}"), 5_000 + i * 10))
                .collect();
            for (id, name, price) in &products {
                product.execute(params![id, name, price]).unwrap();
            }
            let tables: Vec<String> = (0..30).map(|_| new_id()).collect();
            for (i, id) in tables.iter().enumerate() {
                tx.execute(
                    "INSERT INTO tables_dining(id, name, status, branch_id, area) VALUES(?1, ?2, 'available', ?3, 'Dine In')",
                    params![id, format!("S{i}"), branch],
                )
                .unwrap();
            }
            let parties: Vec<String> = (0..customers).map(|_| new_id()).collect();
            for (i, id) in parties.iter().enumerate() {
                tx.execute(
                    "INSERT INTO customers(id, name, phone, created_at, branch_id, kind_label)
                     VALUES(?1, ?2, ?3, ?4, ?5, 'Customer')",
                    params![id, format!("Guest {i}"), format!("98{i:08}"), start.to_rfc3339(), branch],
                )
                .unwrap();
            }
            let mut sale = tx
                .prepare(
                    "INSERT INTO sales(id, created_at, total_paise, pay_method, status, table_id,
                                       lines_total_paise, customer_id, created_by_name, branch_id,
                                       order_type, invoice_no, user_at, device_at)
                     VALUES(?1, ?2, ?3, ?4, 'completed', ?5, ?3, ?6, 'Owner', ?7, 'dine_in', ?8, ?2, ?2)",
                )
                .unwrap();
            let mut line = tx
                .prepare(
                    "INSERT INTO sale_lines(sale_id, product_id, name, unit_price_paise, qty, line_total_paise, qty_milli)
                     VALUES(?1, ?2, ?3, ?4, 1, ?4, 1000)",
                )
                .unwrap();
            let mut pay = tx
                .prepare("INSERT INTO sale_payments(id, sale_id, pay_method, amount_paise) VALUES(?1, ?2, ?3, ?4)")
                .unwrap();
            let mut kot = tx
                .prepare(
                    "INSERT INTO kot_tickets(id, sale_id, created_at, status, summary, branch_id, table_id, total_paise)
                     VALUES(?1, ?2, ?3, 'completed', '3 dishes', ?4, ?5, ?6)",
                )
                .unwrap();
            let mut audit = tx
                .prepare(
                    "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
                     VALUES(?1, ?2, '', 'Owner', 'sale.finalize', ?3)",
                )
                .unwrap();
            let mut ledger = tx
                .prepare(
                    "INSERT INTO credit_ledger(id, customer_id, created_at, kind, amount_paise, sale_id,
                                               pay_method, actor_id, actor_name, user_at, device_at)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, '', 'Owner', ?3, ?3)",
                )
                .unwrap();
            let mut n: i64 = 0;
            for day in 0..days {
                for b in 0..bills_per_day {
                    n += 1;
                    let at = (start
                        + chrono::Duration::days(day)
                        + chrono::Duration::minutes(-12 * 60 + b * 600 / bills_per_day.max(1)))
                    .to_rfc3339();
                    let sale_id = new_id();
                    let picks = [(n * 7) % 300, (n * 13) % 300, (n * 31) % 300];
                    let total: i64 = picks.iter().map(|&i| products[i as usize].2).sum();
                    let table = &tables[(n % 30) as usize];
                    let (method, customer) = if n % 20 == 0 {
                        ("credit", Some(parties[(n as usize / 20) % parties.len()].clone()))
                    } else if n % 10 == 0 {
                        ("split", None)
                    } else {
                        (["cash", "upi", "card"][(n % 3) as usize], None)
                    };
                    sale.execute(params![sale_id, at, total, method, table, customer, branch, format!("INV-MAIN-{n:06}")])
                        .unwrap();
                    for &i in &picks {
                        let (pid, name, price) = &products[i as usize];
                        line.execute(params![sale_id, pid, name, price]).unwrap();
                    }
                    if method == "split" {
                        pay.execute(params![new_id(), sale_id, "cash", total / 2]).unwrap();
                        pay.execute(params![new_id(), sale_id, "upi", total - total / 2]).unwrap();
                    }
                    if let Some(party) = &customer {
                        ledger
                            .execute(params![new_id(), party, at, "sale", total, sale_id, Option::<String>::None])
                            .unwrap();
                        ledger
                            .execute(params![new_id(), party, at, "settle", -total, Option::<String>::None, "cash"])
                            .unwrap();
                    }
                    kot.execute(params![new_id(), sale_id, at, branch, table, total]).unwrap();
                    audit.execute(params![new_id(), at, sale_id]).unwrap();
                }
            }
        }
        tx.commit().unwrap();
    }

    /// How SQLite will run `sql` (EXPLAIN QUERY PLAN, one detail per line).
    pub(crate) fn query_plan_for_test(&self, sql: &str) -> String {
        let mut stmt = self.conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let rows = stmt.query_map([], |r| r.get::<_, String>(3)).unwrap();
        rows.map(|r| r.unwrap()).collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AttachmentEntity, Cart, ModifierOption, MoneyLane, MoneyMoveKind, OrderType, PayMethod,
        Product, StaffRole, VenueMode,
    };
    use crate::fiscal::TaxPricing;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tiny_png_bytes() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9c, 0x63, 0x60, 0x60, 0x60, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0xf6, 0x17, 0x38,
            0x55, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ]
    }

    /// Every backup on the one device that holds the books is lost with
    /// it: after a week with no copy off the device, the owner is told —
    /// until a copy leaves (the second folder, or "Save a backup copy").
    #[test]
    fn a_week_without_a_copy_off_the_device_is_said() {
        let home = std::env::temp_dir().join(format!("kumkaj-copy-due-{}", new_id()));
        fs::create_dir_all(&home).unwrap();
        let store = Store::open_path(&home.join("books.sqlite3")).unwrap();
        assert_eq!(store.backup_copy_overdue_days().unwrap(), None, "no backups yet");
        let made = store.auto_backup_if_due().unwrap().unwrap();
        assert_eq!(store.backup_copy_overdue_days().unwrap(), None, "a new shop is not nagged");
        let ten_days_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(10 * 86_400);
        fs::File::options().write(true).open(&made).unwrap().set_modified(ten_days_ago).unwrap();
        assert!(store.backup_copy_overdue_days().unwrap().is_some_and(|d| d >= 9));
        assert!(store.backup_health().unwrap().copy_overdue_days.is_some());
        store.note_backup_copy_saved().unwrap();
        assert_eq!(store.backup_copy_overdue_days().unwrap(), None);
        let _ = fs::remove_dir_all(home);
    }

    /// One automatic backup a day, two weeks kept, and a copy in the
    /// owner's second folder — a failed copy is said, never silent.
    #[test]
    fn automatic_backups_run_daily_keep_two_weeks_and_copy_to_a_second_place() {
        // Its own folder: the backups land beside the books.
        let home = std::env::temp_dir().join(format!("kumkaj-books-{}", new_id()));
        fs::create_dir_all(&home).unwrap();
        let store = Store::open_path(&home.join("books.sqlite3")).unwrap();
        let dir = store.auto_backup_dir();
        assert!(dir.starts_with(&home));
        let made = store.auto_backup_if_due().unwrap().expect("first backup");
        assert!(made.exists());
        assert!(store.last_backup_at().unwrap().is_some());
        assert!(store.auto_backup_if_due().unwrap().is_none(), "not twice in a day");
        // Older backups beyond two weeks go.
        for day in 1..=20 {
            fs::write(dir.join(format!("auto-202501{day:02}-000000000.sqlite3")), b"old").unwrap();
        }
        let second = std::env::temp_dir().join(format!("kumkaj-second-{}", new_id()));
        store.set_backup_copy_dir(Some(&second)).unwrap();
        store.meta_set("last_backup_at", "2020-01-01T00:00:00+00:00").unwrap();
        let made = store.auto_backup_if_due().unwrap().unwrap();
        let kept = fs::read_dir(&dir).unwrap().count();
        assert_eq!(kept, AUTO_BACKUP_KEEP);
        assert!(made.exists(), "the newest is kept");
        assert!(second.join(made.file_name().unwrap()).exists(), "copied to the second place");
        assert_eq!(store.backup_copy_error().unwrap(), "");
        // The second place is gone (USB pulled): said, and the backup still made.
        let gone = std::env::temp_dir().join(format!("kumkaj-file-{}", new_id()));
        fs::write(&gone, b"not a folder").unwrap();
        store.set_backup_copy_dir(Some(&gone.join("sub"))).unwrap();
        store.meta_set("last_backup_at", "2020-01-01T00:00:00+00:00").unwrap();
        assert!(store.auto_backup_if_due().unwrap().is_some());
        assert!(store.backup_copy_error().unwrap().contains("Could not copy"));
        let _ = fs::remove_dir_all(&home);
        let _ = fs::remove_dir_all(&second);
        let _ = fs::remove_file(gone);
    }

    /// A wrong file is refused before it can replace the books.
    #[test]
    fn restore_refuses_a_file_that_is_not_these_books() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.upsert_product(&test_product("Tea", 20)).unwrap();
        drop(store);
        let junk = std::env::temp_dir().join(format!("kumkaj-junk-{}.sqlite3", new_id()));
        fs::write(&junk, b"not a database at all").unwrap();
        let refused = Store::restore_from_to_path(&junk, &path).err().unwrap_or_default();
        assert!(refused.contains("not a backup"), "{refused}");
        let other = std::env::temp_dir().join(format!("kumkaj-other-{}.sqlite3", new_id()));
        Connection::open(&other).unwrap().execute_batch("CREATE TABLE notes(x TEXT);").unwrap();
        assert!(Store::restore_from_to_path(&other, &path).is_err());
        // The books are untouched.
        let store = Store::open_path(&path).unwrap();
        assert!(store.list_products(false).unwrap().iter().any(|p| p.name == "Tea"));
        let _ = fs::remove_file(junk);
        let _ = fs::remove_file(other);
        let _ = fs::remove_file(path);
    }

    /// Another Kumkaj app's books have the same tables; restoring them over
    /// a restaurant's would swap one business for another. Refused, and the
    /// books stay as they were.
    #[test]
    fn restore_refuses_another_apps_backup() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.upsert_product(&test_product("Tea", 20)).unwrap();
        let other = std::env::temp_dir().join(format!("kumkaj-other-app-{}.sqlite3", new_id()));
        store.backup_to(&other).unwrap();
        drop(store);
        Connection::open(&other)
            .unwrap()
            .execute_batch("UPDATE meta SET value = 'courier' WHERE key = 'app';")
            .unwrap();
        let refused = Store::restore_from_to_path(&other, &path).err().unwrap_or_default();
        assert!(refused.contains("another app"), "{refused}");
        // An older backup without the stamp is still ours if its template is.
        Connection::open(&other)
            .unwrap()
            .execute_batch("DELETE FROM meta WHERE key = 'app';")
            .unwrap();
        assert!(check_backup_file(&other).is_ok());
        let store = Store::open_path(&path).unwrap();
        assert!(store.list_products(false).unwrap().iter().any(|p| p.name == "Tea"));
        let _ = fs::remove_file(other);
        let _ = fs::remove_file(path);
    }

    /// A backup that cannot be written (full disk, a file where the folder
    /// should be) leaves no file that looks like a backup, and the owner is
    /// told until a backup works again.
    #[test]
    fn a_failing_backup_is_reported_until_one_works() {
        let dir = std::env::temp_dir().join(format!("kumkaj-backup-health-{}", new_id()));
        fs::create_dir_all(&dir).unwrap();
        let store = Store::open_path(&dir.join("khaynjo.sqlite3")).unwrap();
        // Where the backups folder should be, there is a file.
        fs::write(dir.join("backups"), b"not a folder").unwrap();
        assert!(store.auto_backup_if_due().is_err());
        let health = store.backup_health().unwrap();
        assert!(!health.error.is_empty(), "the failure is kept for the owner");
        assert_eq!(health.last_at, None);
        assert!(health.free_mb.is_some(), "free space is known on this system");
        fs::remove_file(dir.join("backups")).unwrap();
        let made = store.auto_backup_if_due().unwrap().expect("a backup is due");
        check_backup_file(&made).unwrap();
        let health = store.backup_health().unwrap();
        assert!(health.error.is_empty(), "a working backup clears the warning");
        assert_eq!(health.days_since, Some(0));
        let _ = fs::remove_dir_all(dir);
    }

    /// The lookups every device sync and every busy screen makes find their
    /// rows through an index; none reads every bill, line or ledger row the
    /// shop ever made (that is what made a year-old shop's sync take
    /// seconds). Deterministic: it reads SQLite's plan, not a stopwatch.
    #[test]
    fn hot_lookups_use_an_index_never_a_scan_of_every_row() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let day = "'2026-10-25'";
        for (what, sql) in [
            ("a bill's lines", "SELECT name FROM sale_lines WHERE sale_id = 'x'".to_string()),
            ("a bill's tenders", "SELECT amount_paise FROM sale_payments WHERE sale_id = 'x'".into()),
            ("a customer's balance", "SELECT SUM(amount_paise) FROM credit_ledger WHERE customer_id = 'x'".into()),
            ("recent bills", "SELECT id FROM sales ORDER BY created_at DESC LIMIT 40".into()),
            ("recent khata entries", "SELECT id FROM credit_ledger ORDER BY created_at DESC LIMIT 2000".into()),
            ("today's bills (LIKE)", "SELECT id FROM sales WHERE status = 'completed' AND created_at LIKE '2026-10-25%'".into()),
            ("today's report (range)", format!("SELECT id FROM sales WHERE created_at >= {day} AND created_at < {day} || 'Z'")),
            ("today's money moves", "SELECT kind FROM money_movements WHERE created_at LIKE '2026-10-25%'".into()),
            ("today's expenses", "SELECT amount_paise FROM expenses WHERE created_at LIKE '2026-10-25%'".into()),
            ("today's settlements", "SELECT amount_paise FROM credit_ledger WHERE kind = 'settle' AND created_at LIKE '2026-10-25%'".into()),
            (
                "a table's running total",
                "SELECT SUM(total_paise) FROM kot_tickets WHERE table_id = 'x' AND status != 'cancelled' AND created_at >= '2026'".into(),
            ),
            (
                "open kitchen tickets",
                "SELECT k.id FROM kot_tickets k WHERE k.status NOT IN ('completed', 'cancelled') ORDER BY k.created_at ASC".into(),
            ),
            ("latest audit", "SELECT id FROM audit_events ORDER BY created_at DESC LIMIT 5".into()),
            (
                "today's cancellations",
                format!("SELECT id FROM running_lines WHERE cancelled = 1 AND created_at >= {day} AND created_at < {day} || 'Z'"),
            ),
        ] {
            let plan = store.query_plan_for_test(&sql);
            let scans_all = plan.lines().any(|l| l.starts_with("SCAN") && !l.contains("USING"));
            assert!(!scans_all, "{what} reads every row:\n{plan}");
        }
        let _ = fs::remove_file(path);
    }

    /// The LAN server and the counter write the same file from separate
    /// connections: a write that meets another waits, it does not fail
    /// with "database is locked".
    #[test]
    fn a_second_writer_waits_instead_of_failing_locked() {
        let path = temp_db();
        let a = Store::open_path(&path).unwrap();
        let tx = a.conn.unchecked_transaction().unwrap();
        tx.execute("INSERT INTO meta(key, value) VALUES ('lock_probe_a', '1')", []).unwrap();
        let p2 = path.clone();
        let other = std::thread::spawn(move || {
            let b = Store::open_path(&p2).unwrap();
            let tx = b.conn.unchecked_transaction()?;
            tx.execute("INSERT INTO meta(key, value) VALUES ('lock_probe_b', '1')", [])?;
            tx.commit()
        });
        std::thread::sleep(std::time::Duration::from_millis(300));
        tx.commit().unwrap();
        other.join().unwrap().unwrap();
        assert_eq!(a.meta_get("lock_probe_b").unwrap().as_deref(), Some("1"));
        let _ = std::fs::remove_file(path);
    }

    fn temp_db() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("kumkaj-smoke-{n}.sqlite3"))
    }

    fn test_product(name: &str, rupees: i64) -> Product {
        Product::new_basic(new_id(), name, Money::from_rupees(rupees))
    }

    #[test]
    fn client_books_debit_credit_and_balance_sheet() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Thali", 100)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let party = store
            .add_customer_kind("Sharma Family", "", "Customer")
            .unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.money_lane = MoneyLane::Declared;
        let sale = store
            .finalize_sale(&draft, PayMethod::Credit, None, Some(&party.id), &owner)
            .unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let items = store.period_items_sold(&today, &today).unwrap();
        assert_eq!(items, 1);
        let books = store.books_report_text("today").unwrap();
        assert!(books.contains("Debit / credit"), "{books}");
        assert!(books.contains("balance sheet"), "{books}");
        let invoice = store.format_invoice_text(&sale.id).unwrap();
        assert!(
            invoice.contains("TAX INVOICE") || invoice.contains("CASH MEMO"),
            "{invoice}"
        );
        let bs = store.balance_sheet_text().unwrap();
        assert!(bs.contains("Debtors"), "{bs}");
        assert!(bs.contains("Creditors"), "{bs}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ground_truth_tenders_till_bills_brs_and_csv() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();

        // One expense, settled part bank + part cash (ground truth §18).
        let exp = store
            .add_expense_tenders(
                150_00,
                "rent",
                "office rent",
                &owner,
                &[(PayMethod::Bank, 75_00), (PayMethod::Cash, 75_00)],
            )
            .unwrap();
        let parts = store.expense_payments_for(&exp).unwrap();
        assert_eq!(parts.len(), 2);
        assert!(store
            .add_expense_tenders(100_00, "rent", "", &owner, &[(PayMethod::Cash, 90_00)])
            .is_err());

        // Till: cash sale in, cash expense part out, till→bank out.
        store.upsert_product(&test_product("Thali", 100)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(store.till_balance_paise().unwrap(), 25_00);
        store
            .post_money_movement(MoneyMoveKind::TillToBank, 10_00, "", &owner)
            .unwrap();
        assert_eq!(store.till_balance_paise().unwrap(), 15_00);

        // Bill-wise: two credit bills; unlinked settle washes FIFO,
        // linked settle closes against the chosen bill only.
        let party = store
            .add_customer_kind("Regular Co", "", "Customer")
            .unwrap();
        let mut bill_ids = Vec::new();
        for _ in 0..2 {
            let mut cart = Cart::default();
            cart.add_or_bump(&p);
            let draft = cart.to_sale_draft().unwrap();
            let sale = store
                .finalize_sale(&draft, PayMethod::Credit, None, Some(&party.id), &owner)
                .unwrap();
            bill_ids.push(sale.id);
        }
        let open = store.open_credit_bills(&party.id).unwrap();
        assert_eq!(open.len(), 2);
        assert!(open.iter().all(|b| b.open_paise == 100_00));
        store
            .settle_credit(&party.id, 100_00, PayMethod::Cash, &owner, "")
            .unwrap();
        let open = store.open_credit_bills(&party.id).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].bill_id, bill_ids[1]);
        store
            .settle_credit_against(
                &party.id,
                40_00,
                PayMethod::Bank,
                &owner,
                "",
                Some(&bill_ids[1]),
            )
            .unwrap();
        let open = store.open_credit_bills(&party.id).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].open_paise, 60_00);
        assert_eq!(open[0].paid_paise, 40_00);

        // BRS: tick the bank-in against the statement; the till→bank
        // move stays uncleared, explaining book vs bank.
        store
            .post_money_movement(MoneyMoveKind::BankIn, 500_00, "deposit", &owner)
            .unwrap();
        let moves = store.list_money_movements(10).unwrap();
        let bank_in = moves.iter().find(|m| m.kind == "bank_in").unwrap();
        assert!(!bank_in.cleared);
        store
            .set_money_movement_cleared(&bank_in.id, true, &owner)
            .unwrap();
        let (book, cleared, uncleared_n, uncleared_net) = store.bank_reconciliation().unwrap();
        assert_eq!(book, 510_00);
        assert_eq!(cleared, 500_00);
        assert_eq!(uncleared_n, 1);
        assert_eq!(uncleared_net, 10_00);

        // CSV: spreadsheet in, spreadsheet out; duplicates skipped.
        let (added, skipped, _) = store
            .import_parties_csv("name,phone,kind\nAsha,999,Customer\nBala,,Supplier\nAsha,999,Customer\n")
            .unwrap();
        assert_eq!((added, skipped), (2, 1));
        assert!(store.parties_csv().unwrap().contains("Asha"));
        let (added, skipped, _) = store
            .import_items_csv("name,price,unit\nBox,25.50,pc\nBox,25.50,pc\n")
            .unwrap();
        assert_eq!((added, skipped), (1, 1));
        assert!(store.items_csv().unwrap().contains("Box,25.50,pc"));
        assert!(store.day_book_csv().unwrap().contains("till_cash_in_hand"));
        assert!(store
            .party_ledger_csv(&party.id)
            .unwrap()
            .contains("balance"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn entry_attachments_note_image_pdf_on_sale() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Thali", 100)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let sale = store
            .finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, None, &owner)
            .unwrap();

        store
            .add_attachment_note(AttachmentEntity::Sale, &sale.id, "Desk", "less spicy", &owner)
            .unwrap();

        let png_path = std::env::temp_dir().join(format!("attach-png-{}.png", sale.id));
        std::fs::write(&png_path, tiny_png_bytes()).unwrap();
        let img = store
            .add_attachment_image(AttachmentEntity::Sale, &sale.id, &png_path, "bill photo", &owner)
            .unwrap();
        assert_eq!(img.mime, "image/webp");
        assert!(img.stored_name.ends_with(".webp"));
        let abs = store.attachment_abs_path(&img).unwrap();
        assert!(abs.exists());
        let webp = std::fs::read(&abs).unwrap();
        assert_eq!(&webp[0..4], b"RIFF");
        assert_eq!(&webp[8..12], b"WEBP");

        let pdf_path = std::env::temp_dir().join(format!("attach-pdf-{}.pdf", sale.id));
        std::fs::write(&pdf_path, b"%PDF-1.4 test").unwrap();
        store
            .add_attachment_file(AttachmentEntity::Sale, &sale.id, &pdf_path, "invoice", &owner)
            .unwrap();

        let list = store.list_attachments(AttachmentEntity::Sale, &sale.id).unwrap();
        assert_eq!(list.len(), 3);
        let summary = store
            .attachments_summary_text(AttachmentEntity::Sale, &sale.id)
            .unwrap();
        assert!(summary.contains("note:"));
        assert!(summary.contains("image:"));
        assert!(summary.contains("file:"));

        let exe = std::env::temp_dir().join(format!("attach-bad-{}.exe", sale.id));
        std::fs::write(&exe, b"MZ").unwrap();
        assert!(store
            .add_attachment_image(AttachmentEntity::Sale, &sale.id, &exe, "", &owner)
            .is_err());
        assert!(store
            .add_attachment_file(AttachmentEntity::Sale, &sale.id, &exe, "", &owner)
            .is_err());

        let _ = std::fs::remove_file(png_path);
        let _ = std::fs::remove_file(pdf_path);
        let _ = std::fs::remove_file(exe);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn importing_the_menu_template_twice_adds_nothing_the_second_time() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let shelf = crate::menu_images::shelf();
        assert!(!shelf.is_empty(), "the catalogue ships with the app");
        let folders = crate::menu_images::shelf_folders(shelf).len();
        // The shop already sells one of the dishes: it is kept, not doubled.
        let own = Product::new_basic(new_id(), shelf[0].name.to_uppercase(), Money(9900));
        store.upsert_product(&own).unwrap();
        let first = store.import_catalogue(shelf).unwrap();
        assert_eq!(first.categories_added, folders);
        assert_eq!(first.items_kept, 1);
        assert_eq!(first.items_added + first.items_kept, shelf.len());
        let again = store.import_catalogue(shelf).unwrap();
        assert_eq!((again.categories_added, again.items_added), (0, 0));
        let products = store.list_products(false).unwrap();
        assert_eq!(products.len(), shelf.len());
        let photo = products.iter().find(|p| p.id != own.id).unwrap().image_key.clone().unwrap();
        assert!(photo.starts_with("shelf:"), "{photo}");
        assert!(crate::menu_images::resolve_image_path(Some(&photo), "", std::path::Path::new("")).is_file());
        assert!(store.list_categories().unwrap().iter().all(|c| c.image_key.starts_with("shelf:")));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn raw_material_stays_raw_and_off_the_menu_kind() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let rice = Product::new_raw_material("Rice", "kg");
        store.upsert_product(&rice).unwrap();
        let dosa = Product::new_basic(new_id(), "Masala Dosa", Money(12000));
        store.upsert_product(&dosa).unwrap();
        let all = store.list_products(false).unwrap();
        let rice_back = all.iter().find(|p| p.id == rice.id).unwrap();
        assert!(rice_back.is_raw());
        assert_eq!(rice_back.unit_label, "kg");
        assert_eq!(rice_back.stock_qty, 0, "raw material is counted from zero");
        let dosa_back = all.iter().find(|p| p.id == dosa.id).unwrap();
        assert_eq!(dosa_back.kind, crate::domain::ITEM_KIND_DISH);
        assert!(!dosa_back.is_bought(), "a kitchen never buys its own dish");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn category_takes_a_shelf_photo_only_when_it_has_none() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let png = std::env::temp_dir().join(format!("cat-photo-{}.png", new_id()));
        std::fs::write(&png, tiny_png_bytes()).unwrap();
        let cat = store.add_category("Bakery Cafe").unwrap();
        assert!(store.set_category_photo_if_missing(&cat.id, &png).unwrap());
        let key = store.list_categories().unwrap().into_iter().find(|c| c.id == cat.id).unwrap().image_key;
        assert!(key.starts_with("custom:cat-"), "{key}");
        // The owner's (or an earlier) photo is never replaced.
        assert!(!store.set_category_photo_if_missing(&cat.id, &png).unwrap());
        let _ = std::fs::remove_file(png);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn import_item_image_stores_webp() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let png_path = std::env::temp_dir().join("item-img-test.png");
        std::fs::write(&png_path, tiny_png_bytes()).unwrap();
        let key = store.import_item_image("prod1", &png_path).unwrap();
        assert!(key.ends_with(".webp"));
        let dest = store.item_images_dir().unwrap().join("prod1.webp");
        assert!(dest.exists());
        let _ = std::fs::remove_file(png_path);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn finalize_sale_survives_reopen() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Test item", 25)).unwrap();
        let products = store.list_products(false).unwrap();
        assert!(!products.is_empty());
        let p = &products[0];
        let mut cart = Cart::default();
        cart.add_or_bump(p);
        cart.add_or_bump(p);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Upi, None, None, &owner)
            .unwrap();
        assert!(sale.total_paise > 0);
        assert_eq!(sale.rounding_paise, 0);
        assert_eq!(sale.lines_total_paise, sale.total_paise);
        drop(store);

        let store2 = Store::open_path(&path).unwrap();
        let totals = store2.today_totals().unwrap();
        assert_eq!(totals.sale_count, 1);
        assert_eq!(totals.upi_paise, sale.total_paise);
        assert_eq!(totals.revenue_paise, sale.total_paise);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rounded_sale_keeps_line_amounts_and_rounding() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Tea", 13)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        for _ in 0..4 {
            cart.add_or_bump(&p);
        }
        assert_eq!(cart.line_sum().paise(), 5200);
        cart.set_charge_total(Money::from_rupees(50)).unwrap();
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(draft.lines_total_paise, 5200);
        assert_eq!(draft.rounding_paise, -200);
        assert_eq!(draft.total_paise, 5000);
        assert_eq!(draft.lines[0].unit_price_paise, 1300);
        assert_eq!(draft.lines[0].qty, 4);
        assert_eq!(draft.lines[0].line_total_paise, 5200);

        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(sale.total_paise, 5000);
        assert_eq!(sale.lines_total_paise, 5200);
        assert_eq!(sale.rounding_paise, -200);

        let totals = store.today_totals().unwrap();
        assert_eq!(totals.revenue_paise, 5000);
        assert_eq!(totals.lines_total_paise, 5200);
        assert_eq!(totals.rounding_paise, -200);
        assert_eq!(
            totals.lines_total_paise + totals.rounding_paise,
            totals.revenue_paise
        );

        let lines = store.sale_lines(&sale.id).unwrap();
        assert_eq!(lines[0].unit_price_paise, 1300);
        assert_eq!(lines[0].line_total_paise, 5200);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn pin_hash_roundtrip() {
        let hash = hash_pin("1234").unwrap();
        assert!(verify_pin("1234", &hash).unwrap());
        assert!(!verify_pin("0000", &hash).unwrap());
    }

    #[test]
    fn frequent_items_rank_by_recent_sales_and_keep_config() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        assert_eq!(store.frequent_config().unwrap(), (1, 24));
        store.set_frequent_config(7, 3, &owner).unwrap();
        assert_eq!(store.frequent_config().unwrap(), (7, 3));

        store.upsert_product(&test_product("Aloo", 40)).unwrap();
        store.upsert_product(&test_product("Biryani", 180)).unwrap();
        store.upsert_product(&test_product("Chai", 20)).unwrap();
        let products = store.list_products(false).unwrap();
        let aloo = products.iter().find(|p| p.name == "Aloo").unwrap();
        let biryani = products.iter().find(|p| p.name == "Biryani").unwrap();
        let chai = products.iter().find(|p| p.name == "Chai").unwrap();

        // Biryani sold most, then Chai, then Aloo once.
        for (p, n) in [(biryani, 5), (chai, 3), (aloo, 1)] {
            let mut cart = Cart::default();
            for _ in 0..n {
                cart.add_or_bump(p);
            }
            let draft = cart.to_sale_draft().unwrap();
            store
                .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
                .unwrap();
        }
        let ranked = store.frequent_product_ids(1, 10).unwrap();
        assert_eq!(ranked[0], biryani.id);
        assert_eq!(ranked[1], chai.id);
        assert_eq!(ranked[2], aloo.id);
        assert_eq!(store.frequent_product_ids(1, 2).unwrap().len(), 2);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn zero_value_sale_allowed_by_default_and_ranks_frequent() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        assert!(store.allow_zero_sales().unwrap());
        store.upsert_product(&test_product("Water", 0)).unwrap();
        let water = store.list_products(false).unwrap().into_iter().next().unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&water);
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(draft.total_paise, 0);
        store
            .finalize_sale_tenders(&draft, &[(PayMethod::Cash, 0)], None, None, &owner)
            .unwrap();
        let ranked = store.frequent_product_ids(1, 10).unwrap();
        assert_eq!(ranked, vec![water.id.clone()]);

        store.set_allow_zero_sales(false, &owner).unwrap();
        assert!(!store.allow_zero_sales().unwrap());
        assert!(store
            .finalize_sale_tenders(&draft, &[(PayMethod::Cash, 0)], None, None, &owner)
            .is_err());
        store.set_allow_zero_sales(true, &owner).unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn emergency_unlock_pin_wipes_sales_keeps_menu() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.complete_owner_setup("2468").unwrap();
        store
            .set_duress_pin("9999", "9999", &owner)
            .unwrap();
        assert!(store.set_duress_pin("2468", "2468", &owner).is_err());
        store.upsert_product(&test_product("Paneer", 120)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert!(store.today_totals().unwrap().revenue_paise > 0);

        let unlocked = store.try_duress_unlock("9999").unwrap().expect("owner");
        assert!(unlocked.is_owner());
        assert_eq!(store.today_totals().unwrap().revenue_paise, 0);
        assert_eq!(store.list_products(false).unwrap().len(), 1);
        assert_eq!(store.list_products(false).unwrap()[0].name, "Paneer");
        // Real Owner PIN still works afterwards; emergency PIN still set.
        assert!(store.authenticate_owner_pin("2468").unwrap().is_some());
        assert!(store.duress_pin_configured().unwrap());
        assert!(store.try_duress_unlock("2468").unwrap().is_none());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn backup_and_restore() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Test item", 25)).unwrap();
        let products = store.list_products(false).unwrap();
        let p = &products[0];
        let mut cart = Cart::default();
        cart.add_or_bump(p);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        let backup = std::env::temp_dir().join(format!(
            "kumkaj-backup-{}.sqlite3",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        store.backup_to(&backup).unwrap();
        drop(store);
        let restored = Store::open_path(&backup).unwrap();
        assert_eq!(restored.today_totals().unwrap().sale_count, 1);
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(backup);
    }

    #[test]
    fn void_requires_reason_and_role() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store
            .add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner)
            .unwrap();
        let cashier = store
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|s| s.role == StaffRole::Cashier)
            .unwrap();
        store.upsert_product(&test_product("Test item", 25)).unwrap();
        let products = store.list_products(false).unwrap();
        let p = &products[0];
        let mut cart = Cart::default();
        cart.add_or_bump(p);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Card, None, None, &owner)
            .unwrap();
        assert!(store.void_sale(&sale.id, "", &owner).is_err());
        assert!(store.void_sale(&sale.id, "mistake", &cashier).is_err());
        store.void_sale(&sale.id, "mistake", &owner).unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn amend_keeps_previous_snapshot_with_actor_and_time() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Tea", 13)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        for _ in 0..4 {
            cart.add_or_bump(&p);
        }
        cart.set_charge_total(Money::from_rupees(50)).unwrap();
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();

        // Amend: change charge to 48 and bump unit price path via new draft
        let mut amended = Cart::from_sale(&sale);
        amended.set_charge_total(Money::from_rupees(48)).unwrap();
        let new_draft = amended.to_sale_draft().unwrap();
        let updated = store
            .amend_sale(&sale.id, &new_draft, PayMethod::Upi, "customer paid less", &owner)
            .unwrap();
        assert_eq!(updated.total_paise, 4800);
        assert_eq!(updated.pay_method, "upi");
        assert_eq!(updated.lines_total_paise, 5200);
        assert_eq!(updated.rounding_paise, -400);

        let revs = store.list_sale_revisions(&sale.id).unwrap();
        assert_eq!(revs.len(), 1);
        assert_eq!(revs[0].actor_id, owner.id);
        assert_eq!(revs[0].reason, "customer paid less");
        assert!(!revs[0].revised_at.is_empty());
        assert_eq!(revs[0].previous.total_paise, 5000);
        assert_eq!(revs[0].previous.rounding_paise, -200);
        assert_eq!(revs[0].previous.lines[0].unit_price_paise, 1300);
        assert_eq!(revs[0].previous.pay_method, "cash");

        // Live sale still has tea @ 13
        let live = store.get_sale(&sale.id).unwrap().unwrap();
        assert_eq!(live.lines[0].unit_price_paise, 1300);
        assert_eq!(live.total_paise, 4800);
        assert_eq!(live.created_by_id, owner.id);
        assert!(!live.created_by_name.is_empty());
        // Amend must not rewrite original authorship
        assert_eq!(updated.created_by_id, owner.id);
        assert_eq!(updated.created_by_name, live.created_by_name);

        store
            .add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner)
            .unwrap();
        let cashier = store
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|s| s.role == StaffRole::Cashier)
            .unwrap();
        assert!(store
            .amend_sale(&sale.id, &new_draft, PayMethod::Cash, "nope", &cashier)
            .is_err());
        assert!(store
            .amend_sale(&sale.id, &new_draft, PayMethod::Cash, "", &owner)
            .is_err());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn owner_staff_auth_and_sale_authorship() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        assert!(owner.is_owner());
        assert!(store
            .authenticate_owner_pin("0000")
            .unwrap()
            .unwrap()
            .is_owner());
        assert!(store.authenticate_owner_pin("9999").unwrap().is_none());

        store
            .add_staff("Riya", StaffRole::Cashier, "Barista", "2222", &owner)
            .unwrap();
        let staff = store.list_non_owner_staff().unwrap();
        assert_eq!(staff.len(), 1);
        assert_eq!(staff[0].display_role(), "Barista");
        let staff_id = staff[0].id.clone();
        assert!(store
            .authenticate_staff_pin(&staff_id, "2222")
            .unwrap()
            .is_some());
        assert!(store
            .authenticate_staff_pin(&staff_id, "0000")
            .unwrap()
            .is_none());
        // Owner PIN must not unlock via staff path for owner id
        assert!(store.authenticate_staff_pin(&owner.id, "0000").is_err());

        store.upsert_product(&test_product("Lassi", 40)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        let cashier = store
            .authenticate_staff_pin(&staff_id, "2222")
            .unwrap()
            .unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Upi, None, None, &cashier)
            .unwrap();
        assert_eq!(sale.created_by_id, cashier.id);
        assert!(sale.created_by_name.contains("Riya"));
        assert!(sale.created_by_name.contains("Barista"));

        let mut amended = Cart::from_sale(&sale);
        amended.set_charge_total(Money::from_rupees(39)).unwrap();
        let new_draft = amended.to_sale_draft().unwrap();
        let updated = store
            .amend_sale(&sale.id, &new_draft, PayMethod::Upi, "price match", &owner)
            .unwrap();
        assert_eq!(updated.created_by_id, cashier.id);
        assert_eq!(updated.created_by_name, sale.created_by_name);

        // Owner must not change another person's PIN; only self can.
        assert!(store
            .set_staff_pin(&staff_id, "3333", &owner)
            .unwrap_err()
            .contains("own PIN"));
        store
            .set_staff_pin(&staff_id, "3333", &cashier)
            .unwrap();
        assert!(store
            .authenticate_staff_pin(&staff_id, "3333")
            .unwrap()
            .is_some());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn owner_first_run_pin_setup() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        assert!(!store.owner_setup_complete().unwrap());
        let owner = store.complete_owner_setup("1357").unwrap();
        assert!(owner.is_owner());
        assert!(store.owner_setup_complete().unwrap());
        assert!(store.authenticate_owner_pin("1357").unwrap().is_some());
        assert!(store.authenticate_owner_pin("0000").unwrap().is_none());
        assert!(store.complete_owner_setup("9999").is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sale_line_note_persists() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Tea", 13)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        cart.set_note(0, "less sugar").unwrap();
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        let lines = store.sale_lines(&sale.id).unwrap();
        assert_eq!(lines[0].note, "less sugar");
        let loaded = store.get_sale(&sale.id).unwrap().unwrap();
        assert_eq!(loaded.lines[0].note, "less sugar");
        let again = Cart::from_sale(&loaded);
        assert_eq!(again.lines[0].note, "less sugar");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn counter_mode_skips_kot_restaurant_creates() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.set_venue_mode(VenueMode::Counter).unwrap();
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert!(store.list_open_kots().unwrap().is_empty());

        store.set_venue_mode(VenueMode::Restaurant).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(store.list_open_kots().unwrap().len(), 1);
        let _ = std::fs::remove_file(path);
    }

    /// A kitchen that cooks from the paper KOT never taps Done. Yesterday's
    /// tickets must not stay "to cook" for ever: past [`KOT_OPEN_HOURS`] they
    /// leave the Kitchen screen and the devices' sync; today's stay.
    #[test]
    fn kitchen_tickets_nobody_closed_age_out() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let table = store.add_table("Counter 1").unwrap();
        store.upsert_product(&test_product("Masala Dosa", 120)).unwrap();
        let dosa = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&dosa);
        let draft = cart.to_sale_draft().unwrap();
        let old = store.send_kot(&draft, Some(&table.id), &owner).unwrap().remove(0);
        let fresh = store.send_kot(&draft, Some(&table.id), &owner).unwrap().remove(0);
        let yesterday = (Local::now() - chrono::Duration::hours(KOT_OPEN_HOURS + 1)).to_rfc3339();
        store.backdate_kot_for_test(&old, &yesterday);
        let open: Vec<_> = store.list_open_kots().unwrap().into_iter().map(|k| k.id).collect();
        assert_eq!(open, vec![fresh]);
        let _ = std::fs::remove_file(path);
    }

    /// Khaynjo ships as a restaurant: a fresh shop defaults to the
    /// Restaurant venue (KOT on), and table → KOT → advance works with no
    /// template ceremony beyond the locked Khaynjo face.
    #[test]
    fn khaynjo_fresh_shop_table_kot_flow() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        assert_eq!(store.venue_mode().unwrap(), VenueMode::Restaurant);
        assert_eq!(store.entity_template_id().unwrap(), "khaynjo_restaurant");
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();

        // Fresh shops seed T1..T6; add our own so the id is unambiguous.
        let table = store.add_table("Counter 1").unwrap();
        store.upsert_product(&test_product("Masala Dosa", 120)).unwrap();
        let dosa = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&dosa);
        let draft = cart.to_sale_draft().unwrap();
        let kot_id = store
            .send_kot(&draft, Some(&table.id), &owner)
            .unwrap()
            .remove(0);

        let open = store.list_open_kots().unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, kot_id);
        assert_eq!(open[0].status, crate::domain::KotStatus::New);
        assert!(open[0].summary.contains("Masala Dosa"));
        let tables = store.list_tables().unwrap();
        let ours = tables.iter().find(|t| t.id == table.id).unwrap();
        assert_eq!(ours.status, crate::domain::TableStatus::Occupied);

        store.advance_kot(&kot_id, &owner).unwrap();
        assert_eq!(
            store.list_open_kots().unwrap()[0].status,
            crate::domain::KotStatus::Preparing
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn recall_kot_restores_completed_to_ready() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let table = store.add_table("Recall 1").unwrap();
        store.upsert_product(&test_product("Poha", 50)).unwrap();
        let poha = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&poha);
        let draft = cart.to_sale_draft().unwrap();
        let kot_id = store.send_kot(&draft, Some(&table.id), &owner).unwrap().remove(0);
        store.advance_kot(&kot_id, &owner).unwrap(); // preparing
        store.advance_kot(&kot_id, &owner).unwrap(); // ready
        store.advance_kot(&kot_id, &owner).unwrap(); // completed
        assert!(store.list_open_kots().unwrap().is_empty());
        store.recall_kot(&kot_id, &owner).unwrap();
        let open = store.list_open_kots().unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, kot_id);
        assert_eq!(open[0].status, crate::domain::KotStatus::Ready);
        let recent = store.list_recent_completed_kots(5).unwrap();
        assert!(recent.iter().all(|k| k.id != kot_id));
        let _ = std::fs::remove_file(path);
    }

    /// The service-day lifecycle: waiter sends the KOT, guests finish,
    /// the table goes Payment pending, and it releases only when the
    /// counter records the payment — never just because guests left.
    #[test]
    fn table_payment_pending_releases_only_after_payment() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let table = store.add_table("Patio 9").unwrap();
        store.upsert_product(&test_product("Thali", 200)).unwrap();
        let thali = store
            .list_products(false)
            .unwrap()
            .into_iter()
            .find(|p| p.name == "Thali")
            .unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let draft = cart.to_sale_draft().unwrap();
        store
            .send_kot(&draft, Some(&table.id), &owner)
            .unwrap();

        let ours = |store: &Store| {
            store
                .list_tables()
                .unwrap()
                .into_iter()
                .find(|t| t.id == table.id)
                .unwrap()
        };
        let t = ours(&store);
        assert_eq!(t.status, crate::domain::TableStatus::Occupied);
        assert!(!t.occupied_at.is_empty());
        assert_eq!(t.running_total_paise, draft.total_paise);

        // Guests are done — waiter flags the table, it stays held.
        store.request_billing(&table.id, &owner).unwrap();
        assert_eq!(
            ours(&store).status,
            crate::domain::TableStatus::PaymentPending
        );
        // Leaving is not paying: a direct release is refused.
        assert!(store.free_table(&table.id).is_err());
        assert_eq!(
            ours(&store).status,
            crate::domain::TableStatus::PaymentPending
        );

        // The counter records the payment — the table is Free at once.
        store
            .finalize_sale(&draft, PayMethod::Cash, Some(&table.id), None, &owner)
            .unwrap();
        let t = ours(&store);
        assert_eq!(t.status, crate::domain::TableStatus::Available);
        assert!(t.occupied_at.is_empty());
        assert_eq!(t.running_total_paise, 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn starter_products_noop_and_opening_credit() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        assert_eq!(store.seed_starter_products().unwrap(), 0);
        assert!(store.list_products(false).unwrap().is_empty());
        let customer = store.add_customer("Raju", "").unwrap();
        store
            .seed_opening_credit(&customer.id, 5000, &owner)
            .unwrap();
        assert_eq!(store.customer_balance(&customer.id).unwrap(), 5000);
        assert_eq!(store.outstanding_credit_total().unwrap(), 5000);
        let _ = std::fs::remove_file(path);
    }

    /// Remove is an archive, so the owner can always put an item back — and an
    /// app-provided dish (copied from the bundled shelf) comes back with the
    /// price/photo/category the shop had set on it.
    #[test]
    fn removed_item_restores_exactly_as_it_was() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let mut product = test_product("Banana Bread", 120);
        product.category_id = "cat-bakery".into();
        product.image_key = Some("custom:banana-bread.webp".into());
        store.upsert_product(&product).unwrap();
        assert!(store.list_removed_products().unwrap().is_empty());

        store.archive_product(&product.id).unwrap();
        assert!(store.list_products(false).unwrap().is_empty());
        let removed = store.list_removed_products().unwrap();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].name, "Banana Bread");
        assert_eq!(removed[0].price, Money::from_rupees(120));

        store.restore_product(&product.id).unwrap();
        let back = store.list_products(false).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].id, product.id);
        assert_eq!(back[0].price, Money::from_rupees(120));
        assert_eq!(back[0].category_id, "cat-bakery");
        assert_eq!(back[0].image_key.as_deref(), Some("custom:banana-bread.webp"));
        assert!(store.list_removed_products().unwrap().is_empty());
        assert!(store.restore_product("missing").is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn update_customer_renames_without_touching_balance() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let customer = store.add_customer("Wrong Name", "").unwrap();
        store
            .seed_opening_credit(&customer.id, 5000, &owner)
            .unwrap();
        store
            .update_customer(&customer.id, "Right Name", "98200")
            .unwrap();
        let renamed = store
            .list_customers()
            .unwrap()
            .into_iter()
            .find(|c| c.id == customer.id)
            .unwrap();
        assert_eq!(renamed.name, "Right Name");
        assert_eq!(renamed.phone, "98200");
        assert_eq!(store.customer_balance(&customer.id).unwrap(), 5000);
        assert!(store.update_customer(&customer.id, "  ", "").is_err());
        assert!(store.update_customer("missing", "X", "").is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn delete_customer_guards_balance_and_history() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();

        // Unused party: hard delete.
        let unused = store.add_customer("Unused", "").unwrap();
        let msg = store.delete_customer(&unused.id).unwrap();
        assert!(msg.contains("Deleted party"), "got {msg}");
        assert!(store
            .list_customers()
            .unwrap()
            .iter()
            .all(|c| c.id != unused.id));

        // Balance blocks deletion; after write-off the history hides it.
        let owing = store.add_customer("Owing", "").unwrap();
        store.seed_opening_credit(&owing.id, 5000, &owner).unwrap();
        assert!(store.delete_customer(&owing.id).is_err());
        store
            .write_off_credit(&owing.id, 0, "clear small balance", &owner)
            .unwrap();
        assert_eq!(store.customer_balance(&owing.id).unwrap(), 0);
        let msg = store.delete_customer(&owing.id).unwrap();
        assert!(msg.contains("hidden"), "got {msg}");
        assert!(store
            .list_customers()
            .unwrap()
            .iter()
            .all(|c| c.id != owing.id));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn khaynjo_template_labels_without_seeds() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let tmpl = store.apply_entity_template("khaynjo_restaurant").unwrap();
        assert_eq!(tmpl.id, "khaynjo_restaurant");
        assert_eq!(store.entity_template_id().unwrap(), "khaynjo_restaurant");
        assert_eq!(tmpl.qty_unit_hint, "");
        assert_eq!(tmpl.lane_declared_label, "Tax invoice");
        assert_eq!(tmpl.lane_owner_label, "Cash memo");
        // No sample rates or parties — owner adds their own.
        assert!(store.list_products(false).unwrap().is_empty());
        assert!(store.list_customers().unwrap().is_empty());
        store.apply_entity_template("khaynjo_restaurant").unwrap();
        assert!(store.list_products(false).unwrap().is_empty());
        // Weight × rate: owner-added kg line uses 0.1 kg step on first tap.
        let mut freight = Product::new_basic(new_id(), "Veg Biryani", Money::from_rupees(40));
        freight.unit_label = "kg".into();
        store.upsert_product(&freight).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&freight);
        assert_eq!(cart.lines[0].qty_milli, 100);
        assert_eq!(
            cart.lines[0].actual_total().paise(),
            freight.price.paise() / 10
        );
        let mut draft = cart.to_sale_draft().unwrap();
        draft.money_lane = MoneyLane::Owner;
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        let day = store.today_totals().unwrap();
        assert!(day.owner_lane_paise > 0);
        assert_eq!(day.declared_paise, 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn khaynjo_product_has_no_seed_catalogue() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.apply_entity_template("anything").unwrap();
        assert_eq!(store.entity_template_id().unwrap(), "khaynjo_restaurant");
        assert!(store.list_products(false).unwrap().is_empty());
        assert!(store.list_customers().unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn dine_in_sale_assigns_bill_no_table_and_receipt() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        store.apply_entity_template("khaynjo_restaurant").unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let dosa = Product::new_basic(new_id(), "Masala Dosa", Money::from_rupees(120));
        store.upsert_product(&dosa).unwrap();
        let table = store.add_table("Window 1").unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&dosa);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.order_type = OrderType::DineIn;
        draft.money_lane = MoneyLane::Owner;
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, Some(&table.id), None, &owner)
            .unwrap();
        assert!(sale.invoice_no.starts_with("INV-"), "{}", sale.invoice_no);
        assert_eq!(sale.order_type, "dine_in");
        assert_eq!(sale.table_id.as_deref(), Some(table.id.as_str()));
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("Masala Dosa"), "{receipt}");
        assert!(receipt.contains("Window 1"), "{receipt}");
        assert!(receipt.contains("Dine-in"), "{receipt}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bank_posting_aging_period_and_rich_receipt() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        store.apply_entity_template("khaynjo_restaurant").unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store
            .post_money_movement(MoneyMoveKind::TillToBank, 10_000, "evening deposit", &owner)
            .unwrap();
        assert_eq!(store.bank_balance_paise().unwrap(), 10_000);
        store
            .post_money_movement(MoneyMoveKind::BankOut, 2_000, "office", &owner)
            .unwrap();
        assert_eq!(store.bank_balance_paise().unwrap(), 8_000);

        let mut freight = Product::new_basic(new_id(), "Veg Biryani", Money::from_rupees(40));
        freight.unit_label = "kg".into();
        store.upsert_product(&freight).unwrap();
        let party = store.add_customer_kind("Sumit Int'l", "", "Customer").unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&freight);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.money_lane = MoneyLane::Declared;
        let sale = store
            .finalize_sale(&draft, PayMethod::Credit, None, Some(&party.id), &owner)
            .unwrap();
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        // Receipt wording is "Bill to: <party>" (see format_receipt_text).
        // (The sibling copy of this test asserts "Party:" and fails there too.)
        assert!(receipt.contains("Bill to: Sumit"));
        assert!(receipt.contains("kg") || receipt.contains("0.1"));
        assert!(
            receipt.contains("TAX INVOICE")
                || receipt.contains("Bill type:")
                || receipt.contains("Tax invoice")
        );

        let aging = store.party_aging_report().unwrap();
        assert!(aging.iter().any(|r| r.name.contains("Sumit") && r.balance_paise > 0));
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let books = store.period_books(&today, &today).unwrap();
        assert_eq!(books.sale_count, 1);
        assert!(books.credit_paise > 0);
        let pack = store.books_report_text("today").unwrap();
        assert!(pack.contains("Period books") || pack.contains("Party outstanding"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn customer_advance_then_credit_sale_applies_prepaid() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Tea", 120)).unwrap();
        let customer = store.add_customer("Asha", "").unwrap();
        // ₹500 advance, no prior debt.
        store
            .settle_credit(&customer.id, 50_000, PayMethod::Upi, &owner, "")
            .unwrap();
        assert_eq!(store.customer_balance(&customer.id).unwrap(), -50_000);
        let day = store.today_totals().unwrap();
        assert_eq!(day.upi_paise, 50_000);
        assert_eq!(day.revenue_paise, 0);
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(
                &draft,
                PayMethod::Credit,
                None,
                Some(&customer.id),
                &owner,
            )
            .unwrap();
        // ₹120 sale consumed from advance → ₹380 held.
        assert_eq!(store.customer_balance(&customer.id).unwrap(), -38_000);
        // Overpay settle on someone who owes: seed debt then pay more.
        let owing = store.add_customer("Ravi", "").unwrap();
        store
            .seed_opening_credit(&owing.id, 20_000, &owner)
            .unwrap();
        store
            .settle_credit(&owing.id, 30_000, PayMethod::Cash, &owner, "")
            .unwrap();
        assert_eq!(store.customer_balance(&owing.id).unwrap(), -10_000);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn extra_tills_are_a_shop_switch_not_a_role() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.set_device_role("server").unwrap();
        assert!(!store.extra_tills_allowed().unwrap());
        store.set_extra_tills_allowed(true).unwrap();
        assert!(store.extra_tills_allowed().unwrap());
        store.set_extra_tills_allowed(false).unwrap();
        assert!(!store.extra_tills_allowed().unwrap());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unpaired_client_cannot_finalize() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        // A terminal's catalogue arrives from the Server, so stock the menu
        // before this device becomes a Client.
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        store.set_device_role("client").unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        assert!(store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .is_err());
        store
            .set_client_device_pairing("http://127.0.0.1:7847", "device-token")
            .unwrap();
        assert!(store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .is_ok());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn backup_restore_preserves_sale_on_reopen() {
        let path = temp_db();
        let backup = path.with_extension("bak.sqlite3");
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        store.backup_to(&backup).unwrap();
        drop(store);
        let restored_path = path.with_extension("restored.sqlite3");
        std::fs::copy(&backup, &restored_path).unwrap();
        let store2 = Store::open_path(&restored_path).unwrap();
        assert_eq!(store2.today_totals().unwrap().sale_count, 1);
        assert!(store2.get_sale(&sale.id).unwrap().is_some());
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(backup);
        let _ = std::fs::remove_file(restored_path);
    }

    #[test]
    fn mix_pay_cash_upi_card_and_expense_report() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Tea", 30)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale_tenders(
                &draft,
                &[
                    (PayMethod::Cash, 1000),
                    (PayMethod::Upi, 1000),
                    (PayMethod::Card, 1000),
                ],
                None,
                None,
                &owner,
            )
            .unwrap();
        assert_eq!(sale.pay_method, "split");
        store
            .add_expense(500, "milk", "morning", &owner)
            .unwrap();
        let report = store.ops_report_today().unwrap();
        assert_eq!(report.day.cash_paise, 1000);
        assert_eq!(report.day.upi_paise, 1000);
        assert_eq!(report.day.card_paise, 1000);
        assert_eq!(report.expenses_paise, 500);
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("Cash ₹10.00"), "{receipt}");
        assert!(receipt.contains("UPI ₹10.00"), "{receipt}");
        assert!(receipt.contains("Card ₹10.00"), "{receipt}");
        assert!(!receipt.contains("\nSplit\n"), "{receipt}");

        let err = store
            .finalize_sale_tenders(
                &draft,
                &[(PayMethod::Cash, 500), (PayMethod::Upi, 500)],
                None,
                None,
                &owner,
            )
            .unwrap_err();
        assert!(err.contains("must equal bill"), "{err}");
        assert_eq!(store.recent_sales(10).unwrap().len(), 1);

        // Part cash, rest on credit (khata).
        let customer = store.add_customer("Geeta", "").unwrap();
        let mut cart2 = Cart::default();
        cart2.add_or_bump(&p);
        let draft2 = cart2.to_sale_draft().unwrap();
        let sale2 = store
            .finalize_sale_tenders(
                &draft2,
                &[(PayMethod::Cash, 1000), (PayMethod::Credit, 2000)],
                None,
                Some(&customer.id),
                &owner,
            )
            .unwrap();
        assert_eq!(sale2.pay_method, "split");
        assert_eq!(store.customer_balance(&customer.id).unwrap(), 2000);
        let day = store.today_totals().unwrap();
        assert_eq!(day.credit_paise, 2000);
        let _ = std::fs::remove_file(path);
    }

    /// Names on screen are only what the owner saved: nothing is written
    /// for them, and the "My business" placeholder older books hold reads as
    /// never saved. Reports still have a name to print.
    #[test]
    fn only_names_the_owner_saved_show() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        assert_eq!(store.meta_get("org_name").unwrap().unwrap_or_default(), "");
        assert_eq!(store.saved_org_name().unwrap(), "");
        store.meta_set("org_name", DEFAULT_ORG_NAME).unwrap();
        assert_eq!(store.saved_org_name().unwrap(), "");
        store.set_venue_title("Rajdhani Thali").unwrap();
        assert_eq!(store.org_name().unwrap(), "Rajdhani Thali", "reports fall back to the shop's name");
        store.meta_set("venue_title", "Server af76697b").unwrap();
        assert_eq!(store.venue_title().unwrap(), "");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn branch_seed_and_switch() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let branches = store.list_branches().unwrap();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "Main");
        assert!(branches[0].is_this_device);
        assert_eq!(store.active_branch_id().unwrap(), branches[0].id);
        assert_eq!(store.org_name().unwrap(), DEFAULT_ORG_NAME);
        assert_eq!(store.saved_org_name().unwrap(), "");
        store.set_org_name("Kumkaj Chai Co").unwrap();
        assert_eq!(store.org_name().unwrap(), "Kumkaj Chai Co");
        assert_eq!(store.saved_org_name().unwrap(), "Kumkaj Chai Co");
        assert!(!store.org_id().unwrap().is_empty());

        let second = store.add_branch("MG Road", "mg02").unwrap();
        assert_eq!(second.code, "MG02");
        assert!(!second.is_this_device);
        assert_eq!(store.list_branches().unwrap().len(), 2);
        assert!(store.add_branch("mg road", "").is_err());

        // Items belong to the branch that created them.
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        assert_eq!(store.list_products(false).unwrap().len(), 1);
        store.set_active_branch_id(&second.id).unwrap();
        assert!(store.list_products(false).unwrap().is_empty());
        store.upsert_product(&test_product("Filter coffee", 25)).unwrap();
        let only = store.list_products(false).unwrap();
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].name, "Filter coffee");
        assert_eq!(only[0].branch_id, second.id);
        store.set_active_branch_id(&branches[0].id).unwrap();
        assert_eq!(store.list_products(false).unwrap()[0].name, "Tea");

        store.rename_branch(&second.id, "MG Road 2").unwrap();
        assert!(store
            .list_branches()
            .unwrap()
            .iter()
            .any(|b| b.name == "MG Road 2"));
        assert!(store.set_active_branch_id("nope").is_err());
        let _ = std::fs::remove_file(path);
    }

    /// Fresh DB is schema 1 with org/branch, fiscal pack, units, and triple times.
    #[test]
    fn fresh_db_is_schema_one_with_books_spine() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        assert_eq!(store.meta_get("schema_version").unwrap().unwrap(), "1");
        assert_eq!(store.fiscal_settings().unwrap().country_code, "IN");
        assert_eq!(store.list_tax_rates().unwrap().len(), 5);
        let branch = store.active_branch().unwrap();
        assert_eq!(branch.name, "Main");
        assert!(branch.is_this_device);
        assert_eq!(store.list_stations().unwrap().len(), 2);
        assert_eq!(store.list_tables().unwrap().len(), 6);
        assert!(store.list_units().unwrap().iter().any(|u| u.code == "kg"));

        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        assert_eq!(p.branch_id, branch.id);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert!(!sale.user_at.is_empty());
        assert!(!sale.device_at.is_empty());
        assert!(sale.server_at.is_empty());
        store.add_expense(500, "milk", "", &owner).unwrap();
        assert_eq!(store.today_totals().unwrap().sale_count, 1);
        assert_eq!(store.today_expenses_total().unwrap(), (1, 500));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn category_station_on_product() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        // No categories are seeded: a general shop names its own.
        assert!(store.list_categories().unwrap().is_empty());
        let stations = store.list_stations().unwrap();
        assert_eq!(
            stations.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["Hot", "Cold"]
        );
        let starters = store.add_category("Starters").unwrap();
        assert_eq!(starters.sort_order, 0);
        let tandoor = store.add_station("Tandoor").unwrap();
        assert!(store.add_category("  ").is_err());
        assert!(store.add_station("").is_err());

        let mut p = test_product("Paneer tikka", 220);
        p.category_id = starters.id.clone();
        p.station_id = tandoor.id.clone();
        p.tax_bps = 500;
        p.stock_qty = 12;
        p.set_modifiers(&[ModifierOption {
            name: "Extra spicy".into(),
            price_paise: 0,
            group: String::new(),
        }]);
        store.upsert_product(&p).unwrap();

        let loaded = store.get_product(&p.id).unwrap().unwrap();
        assert_eq!(loaded.category_id, starters.id);
        assert_eq!(loaded.station_id, tandoor.id);
        assert_eq!(loaded.tax_bps, 500);
        assert_eq!(loaded.stock_qty, 12);
        assert!(loaded.tracks_stock());
        assert_eq!(loaded.modifiers().len(), 1);
        assert_eq!(loaded.branch_id, store.active_branch_id().unwrap());
        assert_eq!(store.list_products_in_category(&starters.id).unwrap().len(), 1);
        // Legacy items stay untracked and uncategorised.
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        let tea = store
            .list_products(false)
            .unwrap()
            .into_iter()
            .find(|x| x.name == "Tea")
            .unwrap();
        assert_eq!(tea.stock_qty, STOCK_UNTRACKED);
        assert!(!tea.tracks_stock());
        assert!(tea.in_stock());
        assert!(tea.category_id.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn stock_purchase_adjust_and_sale_decrement() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let mut samosa = test_product("Samosa", 15);
        samosa.stock_qty = 10;
        store.upsert_product(&samosa).unwrap();
        let untracked = test_product("Tea", 10);
        store.upsert_product(&untracked).unwrap();

        // Wastage down, refuse going negative.
        assert_eq!(
            store
                .adjust_stock(&samosa.id, -3, "wastage", "dropped tray", &owner)
                .unwrap(),
            7
        );
        assert!(store
            .adjust_stock(&samosa.id, -100, "wastage", "", &owner)
            .unwrap_err()
            .contains("Not enough stock"));
        assert!(store.adjust_stock(&samosa.id, 0, "adjust", "", &owner).is_err());

        // Goods received raises stock and records the bill.
        let purchase = store
            .record_purchase(
                "Sharma Foods",
                &[(samosa.id.clone(), 5_000, 800)],
                "morning delivery",
                &owner,
            )
            .unwrap();
        assert_eq!(purchase.total_paise, 4000);
        assert_eq!(purchase.lines.len(), 1);
        assert_eq!(store.get_product(&samosa.id).unwrap().unwrap().stock_qty, 12);
        assert_eq!(store.list_purchases(10).unwrap()[0].id, purchase.id);
        assert!(store.record_purchase("X", &[], "", &owner).is_err());

        // Selling decrements tracked items only.
        let mut cart = Cart::default();
        let tracked = store.get_product(&samosa.id).unwrap().unwrap();
        cart.add_or_bump(&tracked);
        cart.add_or_bump(&tracked);
        cart.add_or_bump(&untracked);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(store.get_product(&samosa.id).unwrap().unwrap().stock_qty, 10);
        assert_eq!(
            store.get_product(&untracked.id).unwrap().unwrap().stock_qty,
            STOCK_UNTRACKED
        );

        let low = store.list_low_stock(10).unwrap();
        assert_eq!(low.len(), 1, "untracked items never show as low stock");
        assert_eq!(low[0].id, samosa.id);
        assert!(store.list_low_stock(9).unwrap().is_empty());

        let moves = store.list_stock_movements(20).unwrap();
        let reasons: Vec<&str> = moves.iter().map(|m| m.reason.as_str()).collect();
        assert!(reasons.contains(&"sale"));
        assert!(reasons.contains(&"purchase"));
        assert!(reasons.contains(&"wastage"));
        let sale_move = moves.iter().find(|m| m.reason == "sale").unwrap();
        assert_eq!(sale_move.delta_qty, -2);
        assert_eq!(sale_move.product_name, "Samosa");
        assert!(!sale_move.ref_id.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn tax_extracted_on_sale() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        // ₹105 quoted tax inclusive at 5% → ₹5 of tax sits inside the bill.
        let mut thali = test_product("Thali", 105);
        thali.tax_bps = 500;
        store.upsert_product(&thali).unwrap();
        let mut chutney = test_product("Chutney", 20);
        chutney.tax_bps = 0;
        store.upsert_product(&chutney).unwrap();

        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        cart.add_or_bump(&chutney);
        cart.set_order_type(OrderType::Takeaway);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Upi, None, None, &owner)
            .unwrap();
        assert_eq!(sale.total_paise, 12_500, "tax never inflates the bill");
        assert_eq!(sale.lines_total_paise, 12_500);
        assert_eq!(sale.tax_paise, 500);
        assert_eq!(sale.order_type(), OrderType::Takeaway);
        assert_eq!(sale.branch_id, store.active_branch_id().unwrap());

        let loaded = store.get_sale(&sale.id).unwrap().unwrap();
        assert_eq!(loaded.tax_paise, 500);
        assert_eq!(loaded.order_type, "takeaway");
        let totals = store.today_totals().unwrap();
        assert_eq!(totals.revenue_paise, 12_500);
        assert_eq!(totals.tax_paise, 500);
        assert_eq!(sale.net_paise, 12_000);
        assert_eq!(sale.tax_pricing(), TaxPricing::Inclusive);
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("(Incl. GST)"), "{receipt}");
        assert!(receipt.contains("Net "), "{receipt}");

        // Finalizing queues a durable event for a future cloud push.
        let pending = store.list_pending_outbox(10).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].1, "sale.finalize");
        assert!(sale.user_at.trim().len() >= 10, "user_at={}", sale.user_at);
        assert!(sale.device_at.trim().len() >= 10, "device_at={}", sale.device_at);
        assert!(sale.server_at.is_empty());
        store.mark_outbox_synced(&pending[0].0).unwrap();
        assert!(store.list_pending_outbox(10).unwrap().is_empty());
        let synced = store.get_sale(&sale.id).unwrap().unwrap();
        assert!(
            !synced.server_at.is_empty(),
            "cloud drain should stamp server_at"
        );
        store.enqueue_outbox("day.close", "{}").unwrap();
        assert_eq!(store.list_pending_outbox(10).unwrap().len(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inclusive_tax_unchanged_total() {
        // India is the default pack: the quoted price is the charged price.
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let fiscal_settings = store.fiscal_settings().unwrap();
        assert_eq!(fiscal_settings.country_code, "IN");
        assert_eq!(fiscal_settings.tax_pricing, TaxPricing::Inclusive);
        assert_eq!(fiscal_settings.tax_label(), "GST");
        assert_eq!(fiscal_settings.business_id_label, "GSTIN");

        // The pack seeds the Indian slab rates on this branch.
        let rates = store.list_tax_rates().unwrap();
        assert_eq!(rates.len(), 5);
        let eighteen = rates.iter().find(|r| r.bps == 1_800).unwrap().clone();
        assert_eq!(eighteen.percent_label(), "18%");

        let mut thali = test_product("Thali", 118);
        store.upsert_product(&thali).unwrap();
        store.set_product_tax_rate(&thali.id, &eighteen.id).unwrap();
        thali = store.get_product(&thali.id).unwrap().unwrap();
        assert_eq!(thali.tax_bps, 1_800);
        assert_eq!(thali.tax_rate_id, eighteen.id);

        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(store.preview_sale_total(&draft).unwrap(), 11_800);
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(sale.total_paise, 11_800, "inclusive tax never moves a bill");
        assert_eq!(sale.tax_paise, 1_800);
        assert_eq!(sale.net_paise, 10_000);
        assert_eq!(sale.tax_pricing, "inclusive");
        assert_eq!(sale.tax_label, "GST");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn apply_country_pack_gb_sets_vat_inclusive() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let settings = store.apply_country_pack("gb").unwrap();
        assert_eq!(settings.country_code, "GB");
        assert_eq!(settings.tax_label, "VAT");
        assert_eq!(settings.tax_pricing, TaxPricing::Inclusive);
        assert_eq!(settings.business_id_label, "VAT number");
        assert_eq!(settings.digital_pay_label, "Bank transfer");
        assert_eq!(settings.date_order, "dmy");
        // The pack moves currency/language too, and survives a reopen.
        assert_eq!(store.locale_settings().unwrap().currency, "GBP");
        assert_eq!(store.fiscal_settings().unwrap(), settings);

        // India's slabs are retired by the switch; the UK ones take over.
        let bps: Vec<i64> = store.list_tax_rates().unwrap().iter().map(|r| r.bps).collect();
        assert_eq!(bps, vec![0, 500, 2_000]);

        // A rate an item still uses is never retired, even when the new
        // country does not ship it.
        let mut curry = test_product("Curry", 100);
        curry.tax_bps = 2_000;
        store.upsert_product(&curry).unwrap();
        let twenty = store
            .list_tax_rates()
            .unwrap()
            .into_iter()
            .find(|r| r.bps == 2_000)
            .unwrap();
        store.set_product_tax_rate(&curry.id, &twenty.id).unwrap();
        store.apply_country_pack("IN").unwrap();
        let after = store.list_tax_rates().unwrap();
        assert!(after.iter().any(|r| r.id == twenty.id), "in-use rate kept");
        assert!(after.iter().any(|r| r.bps == 2_800), "IN 28% seeded");
        assert_eq!(store.locale_settings().unwrap().currency, "INR");
        assert!(store.apply_country_pack("ZZ").is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn exclusive_tax_increases_sale_total() {
        // UAE quotes net prices and adds 5% VAT at the till.
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let settings = store.apply_country_pack("AE").unwrap();
        assert_eq!(settings.tax_pricing, TaxPricing::Exclusive);
        assert_eq!(settings.business_id_label, "TRN");

        let mut shawarma = test_product("Shawarma", 100);
        shawarma.tax_bps = 500;
        store.upsert_product(&shawarma).unwrap();
        let mut water = test_product("Water", 20);
        water.tax_bps = 0;
        store.upsert_product(&water).unwrap();

        let mut cart = Cart::default();
        cart.add_or_bump(&shawarma);
        cart.add_or_bump(&water);
        let draft = cart.to_sale_draft().unwrap();
        draft.assert_balanced().unwrap();
        assert_eq!(draft.total_paise, 12_000, "cart quotes net prices");
        assert_eq!(store.preview_sale_total(&draft).unwrap(), 12_500);

        let sale = store
            .finalize_sale(&draft, PayMethod::Card, None, None, &owner)
            .unwrap();
        assert_eq!(sale.tax_paise, 500);
        assert_eq!(sale.total_paise, 12_500, "exclusive tax is added on top");
        assert_eq!(sale.net_paise, 12_000);
        assert_eq!(sale.lines_total_paise, 12_000);
        assert_eq!(sale.tax_pricing, "exclusive");

        // The payment row, the day totals and the receipt all follow the
        // taxed figure — not the price the seller quoted.
        let totals = store.today_totals().unwrap();
        assert_eq!(totals.revenue_paise, 12_500);
        assert_eq!(totals.card_paise, 12_500);
        assert_eq!(store.ops_report_today().unwrap().day.card_paise, 12_500);
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("Subtotal"), "{receipt}");
        assert!(receipt.contains("VAT"), "{receipt}");
        assert!(!receipt.contains("Incl."), "{receipt}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn exclusive_tax_credit_sale_owes_the_taxed_total() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.apply_country_pack("US").unwrap();
        // US ships no rate at all — the seller enters the one they collect.
        assert!(store.list_tax_rates().unwrap().is_empty());
        let rate = store.resolve_tax_rate("8.25").unwrap().unwrap();
        assert_eq!(rate.bps, 825);

        let mut burger = test_product("Burger", 20);
        store.upsert_product(&burger).unwrap();
        store.set_product_tax_rate(&burger.id, &rate.id).unwrap();
        burger = store.get_product(&burger.id).unwrap().unwrap();

        let customer = store.find_or_create_customer("Dana").unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&burger);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(
                &draft,
                PayMethod::Credit,
                None,
                Some(&customer.id),
                &owner,
            )
            .unwrap();
        assert_eq!(sale.tax_paise, 165);
        assert_eq!(sale.total_paise, 2_165);
        assert_eq!(store.customer_balance(&customer.id).unwrap(), 2_165);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn optional_service_charge_rides_on_top_of_the_lines() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        // Off by default: no country pack invents a service charge.
        assert_eq!(store.fiscal_settings().unwrap().service_charge_bps, 0);
        store.set_service_charge_bps(1_000).unwrap();
        store.upsert_product(&test_product("Thali", 100)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        // Split pay and the Sell strip both price off the preview, so it has
        // to carry the service charge too.
        assert_eq!(store.preview_sale_total(&draft).unwrap(), 11_000);
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(sale.service_paise, 1_000);
        assert_eq!(sale.total_paise, 11_000);
        assert_eq!(store.get_sale(&sale.id).unwrap().unwrap().service_paise, 1_000);
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("Service ₹10.00"), "{receipt}");
        assert!(receipt.contains("Total ₹110.00"), "{receipt}");

        // The house rule outlives a country switch, and turning it off leaves
        // the next bill at the quoted price.
        assert_eq!(store.apply_country_pack("US").unwrap().service_charge_bps, 1_000);
        store.set_service_charge_bps(0).unwrap();
        let plain = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(plain.service_paise, 0);
        assert_eq!(plain.total_paise, 10_000);
        assert!(!store
            .format_receipt_text(&plain.id)
            .unwrap()
            .contains("Service"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn apply_country_pack_de_sets_comma_decimal() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        // India writes 1,234.50; Germany writes 1.234,50.
        assert_eq!(store.fiscal_settings().unwrap().decimal_sep, ".");
        assert_eq!(store.fiscal_settings().unwrap().thousands_sep, ",");
        let settings = store.apply_country_pack("de").unwrap();
        assert_eq!(settings.decimal_sep, ",");
        assert_eq!(settings.thousands_sep, ".");
        assert_eq!(store.fiscal_settings().unwrap(), settings);

        store.upsert_product(&test_product("Brezel", 1_234)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("€1.234,00"), "{receipt}");
        assert!(store.day_close_summary().unwrap().contains("€1.234,00"));

        // A grouping space is a real choice, so it must survive the round
        // trip through meta rather than being trimmed away to nothing.
        let fr = store.apply_country_pack("FR").unwrap();
        assert_eq!(fr.thousands_sep, " ");
        assert_eq!(store.fiscal_settings().unwrap().thousands_sep, " ");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn receipt_shows_digital_pay_label_for_br_pix() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.apply_country_pack("BR").unwrap();
        store.upsert_product(&test_product("Pastel", 12)).unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        // Same `upi` tender in storage, printed as the local rail.
        let sale = store
            .finalize_sale(&draft, PayMethod::Upi, None, None, &owner)
            .unwrap();
        assert_eq!(sale.pay_method, "upi");
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("Pix"), "{receipt}");
        assert!(!receipt.contains("UPI"), "{receipt}");
        assert!(store.day_close_summary().unwrap().contains("Pix:"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn tax_split_india_cgst_sgst_on_receipt() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.set_business_tax_id("29ABCDE1234F1Z5").unwrap();
        let mut biryani = test_product("Biryani", 118);
        biryani.tax_bps = 1_800;
        store.upsert_product(&biryani).unwrap();

        let mut cart = Cart::default();
        cart.add_or_bump(&biryani);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Upi, None, None, &owner)
            .unwrap();
        assert_eq!(sale.tax_paise, 1_800);
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("GSTIN 29ABCDE1234F1Z5"), "{receipt}");
        assert!(receipt.contains("CGST"), "{receipt}");
        assert!(receipt.contains("SGST"), "{receipt}");
        assert!(receipt.contains("UPI"), "{receipt}");

        // Halves must add back to the GST charged.
        let halves = store.fiscal_settings().unwrap().tax_components(1_800);
        assert_eq!(halves, vec![("CGST", 900), ("SGST", 900)]);

        // Other countries print one tax line.
        store.apply_country_pack("SG").unwrap();
        assert!(store
            .fiscal_settings()
            .unwrap()
            .tax_components(900)
            .is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn tax_pricing_override_keeps_the_country_pack() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.apply_country_pack("GB").unwrap();
        store.set_tax_pricing(TaxPricing::Exclusive).unwrap();
        let settings = store.fiscal_settings().unwrap();
        assert_eq!(settings.country_code, "GB", "country is unchanged");
        assert_eq!(settings.tax_label, "VAT");
        assert_eq!(settings.tax_pricing, TaxPricing::Exclusive);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn typed_tax_resolves_to_a_named_rate() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        // Seeded Indian slabs match by code, by name and by percent.
        assert_eq!(store.resolve_tax_rate("18").unwrap().unwrap().bps, 1_800);
        assert_eq!(store.resolve_tax_rate("18%").unwrap().unwrap().bps, 1_800);
        assert_eq!(
            store.resolve_tax_rate("gst 12%").unwrap().unwrap().bps,
            1_200
        );
        assert!(store.resolve_tax_rate("  ").unwrap().is_none());
        // An unseen percent is created once and reused after that.
        let created = store.resolve_tax_rate("7.5").unwrap().unwrap();
        assert_eq!(created.bps, 750);
        assert_eq!(created.name, "GST 7.5%");
        assert_eq!(store.resolve_tax_rate("7.5").unwrap().unwrap().id, created.id);
        assert!(store.resolve_tax_rate("bogus").is_err());
        assert!(store.resolve_tax_rate("140").is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inclusive_sale_stores_net_and_tax_pricing() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        assert_eq!(store.meta_get("schema_version").unwrap().unwrap(), "1");
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let mut tea = test_product("Tea", 105);
        tea.tax_bps = 500;
        store.upsert_product(&tea).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&tea);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        let sale = store.recent_sales(1).unwrap().remove(0);
        assert_eq!(sale.total_paise, 10_500);
        assert_eq!(sale.tax_paise, 500);
        assert_eq!(sale.net_paise, 10_000);
        assert_eq!(sale.tax_pricing(), TaxPricing::Inclusive);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn branch_snapshot_import_consolidated() {
        let main_path = temp_db();
        let remote_path = temp_db();
        let mut main = Store::open_path(&main_path).unwrap();
        let mut remote = Store::open_path(&remote_path).unwrap();

        // Remote outlet: rename its own branch, sell ₹60 cash.
        let remote_branch = remote.list_branches().unwrap().remove(0);
        remote.rename_branch(&remote_branch.id, "MG Road").unwrap();
        let remote_owner = remote.list_staff().unwrap().into_iter().next().unwrap();
        remote.upsert_product(&test_product("Dosa", 60)).unwrap();
        let rp = remote.list_products(false).unwrap().remove(0);
        let mut rcart = Cart::default();
        rcart.add_or_bump(&rp);
        let rdraft = rcart.to_sale_draft().unwrap();
        remote
            .finalize_sale(&rdraft, PayMethod::Cash, None, None, &remote_owner)
            .unwrap();
        let snapshot_json = remote.export_branch_snapshot_json().unwrap();
        let snapshot = BranchSnapshot::from_json(&snapshot_json).unwrap();
        assert_eq!(snapshot.branch_name, "MG Road");
        assert_eq!(snapshot.totals.revenue_paise, 6000);
        assert!(!snapshot.exported_at.is_empty());

        // Main outlet: sell ₹40 UPI, then import the remote day.
        let owner = main.list_staff().unwrap().into_iter().next().unwrap();
        main.upsert_product(&test_product("Tea", 40)).unwrap();
        let p = main.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        main.finalize_sale(&draft, PayMethod::Upi, None, None, &owner)
            .unwrap();

        assert!(main.import_branch_snapshot("not json").is_err());
        assert!(main
            .import_branch_snapshot(&main.export_branch_snapshot_json().unwrap())
            .unwrap_err()
            .contains("own branch"));
        let imported = main.import_branch_snapshot(&snapshot_json).unwrap();
        assert_eq!(imported.name, "MG Road");
        assert!(!imported.is_this_device);
        assert!(!imported.last_sync_at.is_empty());
        assert_eq!(main.list_branches().unwrap().len(), 2);
        // Re-importing updates the same row rather than adding a branch.
        main.import_branch_snapshot(&snapshot_json).unwrap();
        assert_eq!(main.list_branches().unwrap().len(), 2);

        let report = main.consolidated_report().unwrap();
        assert_eq!(report.totals.sale_count, 2);
        assert_eq!(report.totals.revenue_paise, 10_000);
        assert_eq!(report.totals.upi_paise, 4000);
        assert_eq!(report.totals.cash_paise, 6000);
        assert_eq!(report.branches.len(), 2);
        assert!(report.stale_branches.is_empty());
        let mg = report
            .branches
            .iter()
            .find(|(name, _)| name == "MG Road")
            .unwrap();
        assert_eq!(mg.1.revenue_paise, 6000);
        // Local day view stays branch-scoped.
        assert_eq!(main.today_totals().unwrap().revenue_paise, 4000);

        // A branch with nothing imported yet is reported as missing, not zeroed silently.
        main.add_branch("Airport", "AIR").unwrap();
        let report = main.consolidated_report().unwrap();
        assert_eq!(report.missing_branches, vec!["Airport".to_string()]);
        assert_eq!(report.totals.revenue_paise, 10_000);
        let _ = std::fs::remove_file(main_path);
        let _ = std::fs::remove_file(remote_path);
    }

    #[test]
    fn multi_station_kot_split() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.set_venue_mode(VenueMode::Restaurant).unwrap();
        let stations = store.list_stations().unwrap();
        let hot = stations.iter().find(|s| s.name == "Hot").unwrap().clone();
        let cold = stations.iter().find(|s| s.name == "Cold").unwrap().clone();

        let mut biryani = test_product("Biryani", 180);
        biryani.station_id = hot.id.clone();
        store.upsert_product(&biryani).unwrap();
        let mut lassi = test_product("Lassi", 60);
        lassi.station_id = cold.id.clone();
        store.upsert_product(&lassi).unwrap();

        let mut cart = Cart::default();
        cart.add_or_bump(&biryani);
        cart.add_or_bump(&lassi);
        cart.set_note(1, "less sugar").unwrap();
        cart.set_order_type(OrderType::DineIn);
        let draft = cart.to_sale_draft().unwrap();
        store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();

        let kots = store.list_open_kots().unwrap();
        assert_eq!(kots.len(), 2, "one ticket per station on the bill");
        let hot_ticket = kots.iter().find(|k| k.station_id == hot.id).unwrap();
        let cold_ticket = kots.iter().find(|k| k.station_id == cold.id).unwrap();
        assert_eq!(hot_ticket.station_name, "Hot");
        assert!(hot_ticket.summary.contains("Biryani"));
        assert!(!hot_ticket.summary.contains("Lassi"));
        assert!(cold_ticket.summary.contains("Lassi (less sugar)"));
        assert!(!cold_ticket.summary.contains("Biryani"));
        assert_eq!(store.list_open_kots_for_station(&hot.id).unwrap().len(), 1);

        // Items with no station keep the old single-ticket behaviour.
        store.upsert_product(&test_product("Water", 20)).unwrap();
        let water = store
            .list_products(false)
            .unwrap()
            .into_iter()
            .find(|p| p.name == "Water")
            .unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&water);
        cart.add_or_bump(&biryani);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        let fresh: Vec<KotTicket> = store
            .list_open_kots()
            .unwrap()
            .into_iter()
            .filter(|k| k.sale_id == sale.id)
            .collect();
        assert_eq!(fresh.len(), 2);
        assert!(fresh.iter().any(|k| k.station_id.is_empty()));
        let _ = std::fs::remove_file(path);
    }

    /// Selling two of a one-on-hand item must fail outright. The old code
    /// clamped the decrement at zero, so the bill went through and the stock
    /// count quietly lied.
    #[test]
    fn selling_more_than_stock_is_refused_and_leaves_stock_alone() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let mut ladoo = test_product("Ladoo", 20);
        ladoo.stock_qty = 1;
        store.upsert_product(&ladoo).unwrap();
        let tracked = store.get_product(&ladoo.id).unwrap().unwrap();

        let mut cart = Cart::default();
        cart.add_or_bump(&tracked);
        cart.add_or_bump(&tracked);
        let draft = cart.to_sale_draft().unwrap();
        let err = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap_err();
        assert!(err.contains("Not enough stock"), "got {err}");
        assert_eq!(
            store.get_product(&ladoo.id).unwrap().unwrap().stock_qty,
            1,
            "a refused sale must not move stock"
        );
        assert!(store.recent_sales(10).unwrap().is_empty());

        // Selling exactly what is on hand still works and empties the shelf.
        let mut cart = Cart::default();
        cart.add_or_bump(&tracked);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(store.get_product(&ladoo.id).unwrap().unwrap().stock_qty, 0);

        // And the now-empty item cannot be added to another cart at all.
        let empty = store.get_product(&ladoo.id).unwrap().unwrap();
        assert!(!empty.in_stock());
        let mut cart = Cart::default();
        let err = cart.add_or_bump_in_stock(&empty).unwrap_err();
        assert!(err.contains("out of stock"), "got {err}");
        assert!(cart.is_empty());

        // Undoing that charge has to put the unit back, or strict enforcement
        // would leave stock the shop still has unsellable.
        store.quick_undo_sale(&sale.id, &owner).unwrap();
        assert_eq!(store.get_product(&ladoo.id).unwrap().unwrap().stock_qty, 1);
        let undo_move = store
            .list_stock_movements(20)
            .unwrap()
            .into_iter()
            .find(|m| m.reason == "undo")
            .expect("the return must leave a movement behind");
        assert_eq!(undo_move.delta_qty, 1);
        assert_eq!(undo_move.product_name, "Ladoo");
        let _ = std::fs::remove_file(path);
    }

    /// The cart itself stops the seller at the on-hand count, so the refusal
    /// lands on the grid rather than on the customer at the till.
    #[test]
    fn cart_refuses_to_bump_past_stock() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let mut samosa = test_product("Samosa", 15);
        samosa.stock_qty = 2;
        store.upsert_product(&samosa).unwrap();
        let tracked = store.get_product(&samosa.id).unwrap().unwrap();

        let mut cart = Cart::default();
        cart.add_or_bump_in_stock(&tracked).unwrap();
        cart.add_or_bump_in_stock(&tracked).unwrap();
        assert_eq!(cart.qty_of(&samosa.id), 2);
        let err = cart.add_or_bump_in_stock(&tracked).unwrap_err();
        assert!(err.contains("Only 2 of Samosa"), "got {err}");
        assert_eq!(cart.qty_of(&samosa.id), 2, "the refusal must not bump qty");

        // Untracked items stay unlimited.
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        let tea = store
            .list_products(false)
            .unwrap()
            .into_iter()
            .find(|p| p.name == "Tea")
            .unwrap();
        for _ in 0..25 {
            cart.add_or_bump_in_stock(&tea).unwrap();
        }
        assert_eq!(cart.qty_of(&tea.id), 25);
        let _ = std::fs::remove_file(path);
    }

    /// A Client's tables are a sync cache, so local catalogue edits would be
    /// silently overwritten. Every one of them must be refused outright.
    #[test]
    fn client_device_cannot_write_the_shop_books() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        // Stock the menu as the Server, then demote this device.
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        let tea = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&tea);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        store.set_device_role("client").unwrap();

        let refusals: Vec<String> = vec![
            store.upsert_product(&test_product("Coffee", 30)).unwrap_err(),
            store.archive_product(&tea.id).unwrap_err(),
            store.set_product_available(&tea.id, false).unwrap_err(),
            store.set_product_stock_qty(&tea.id, 5).unwrap_err(),
            store
                .adjust_stock(&tea.id, 3, "count", "", &owner)
                .unwrap_err(),
            store
                .record_purchase("Supplier", &[(tea.id.clone(), 2_000, 500)], "", &owner)
                .unwrap_err(),
            store.add_branch("MG Road", "MG").unwrap_err(),
            store.add_table("T9").unwrap_err(),
            store.add_category("Snacks").unwrap_err(),
            store.apply_country_pack("GB").unwrap_err(),
            store.set_business_tax_id("X").unwrap_err(),
            store.set_tax_pricing(TaxPricing::Exclusive).unwrap_err(),
            store.add_expense(500, "misc", "", &owner).unwrap_err(),
            store
                .add_staff("Ravi", StaffRole::Cashier, "Cashier", "1234", &owner)
                .unwrap_err(),
            store
                .amend_sale(&sale.id, &draft, PayMethod::Upi, "fix", &owner)
                .unwrap_err(),
        ];
        for err in &refusals {
            assert!(
                err.contains("this is an extra till"),
                "every refusal must name the reason, got {err}"
            );
        }

        // Nothing changed, and selling over the LAN is still allowed once paired.
        assert_eq!(store.list_products(false).unwrap().len(), 1);
        assert!(store.get_product(&tea.id).unwrap().unwrap().available);
        store
            .set_client_device_pairing("http://127.0.0.1:7847", "device-token")
            .unwrap();
        assert!(store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .is_ok());

        // Sync is the Server's own data arriving, so it must still land.
        store
            .replace_catalog_from_sync(&[test_product("Server item", 40)])
            .unwrap();
        assert_eq!(store.list_products(false).unwrap()[0].name, "Server item");
        let _ = std::fs::remove_file(path);
    }

    /// A backup must restore onto a DB path that has never seen this shop.
    #[test]
    fn restore_from_path_preserves_sale() {
        let path = temp_db();
        let backup = path.with_extension("backup.sqlite3");
        let fresh = path.with_extension("restored.sqlite3");

        let (sale_id, invoice_no, total_paise) = {
            let mut store = Store::open_path(&path).unwrap();
            let owner = store.list_staff().unwrap().into_iter().next().unwrap();
            store.upsert_product(&test_product("Thali", 120)).unwrap();
            let p = store.list_products(false).unwrap().remove(0);
            let mut cart = Cart::default();
            cart.add_or_bump(&p);
            let draft = cart.to_sale_draft().unwrap();
            let sale = store
                .finalize_sale(&draft, PayMethod::Upi, None, None, &owner)
                .unwrap();
            store.backup_to(&backup).unwrap();
            (sale.id, sale.invoice_no, sale.total_paise)
        };
        assert!(backup.exists());

        // An empty DB at the destination, complete with its own WAL, is exactly
        // what a reinstall looks like.
        {
            let store = Store::open_path(&fresh).unwrap();
            assert!(store.recent_sales(10).unwrap().is_empty());
        }

        let restored = Store::restore_from_to_path(&backup, &fresh).unwrap();
        let sale = restored
            .get_sale(&sale_id)
            .unwrap()
            .expect("the backed-up sale must come back");
        assert_eq!(sale.id, sale_id);
        assert_eq!(sale.total_paise, total_paise);
        assert_eq!(sale.invoice_no, invoice_no);
        assert_eq!(restored.sale_lines(&sale_id).unwrap().len(), 1);
        assert_eq!(restored.today_totals().unwrap().sale_count, 1);
        assert!(restored.format_receipt_text(&sale_id).unwrap().contains(&invoice_no));
        // The restored DB is a working till, not a read-only copy.
        assert_eq!(restored.db_path(), fresh.as_path());
        assert!(Store::restore_from_to_path(Path::new("/nope/none.sqlite3"), &fresh).is_err());

        for p in [path, backup, fresh] {
            let _ = std::fs::remove_file(p);
        }
    }

    /// Every bill gets a number a customer can quote back, and the counter
    /// survives a restart.
    #[test]
    fn sales_get_sequential_invoice_numbers() {
        let path = temp_db();
        let first = {
            let mut store = Store::open_path(&path).unwrap();
            let owner = store.list_staff().unwrap().into_iter().next().unwrap();
            store.upsert_product(&test_product("Chai", 10)).unwrap();
            let p = store.list_products(false).unwrap().remove(0);
            let mut cart = Cart::default();
            cart.add_or_bump(&p);
            let draft = cart.to_sale_draft().unwrap();
            let a = store
                .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
                .unwrap();
            let b = store
                .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
                .unwrap();
            assert_eq!(a.invoice_no, "INV-MAIN-000001");
            assert_eq!(b.invoice_no, "INV-MAIN-000002");
            // The receipt shows the invoice, not a truncated uuid.
            let receipt = store.format_receipt_text(&a.id).unwrap();
            assert!(
                receipt.contains("Bill: INV-MAIN-000001")
                    || receipt.contains("Invoice no: INV-MAIN-000001"),
                "got {receipt}"
            );
            assert!(!receipt.contains(&a.id[..8]));
            a.id
        };

        // Reopen: the counter carries on rather than reissuing 000001.
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        let third = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert_eq!(third.invoice_no, "INV-MAIN-000003");
        // Amending corrects a bill; it does not issue a new number.
        let amended = store
            .amend_sale(&first, &draft, PayMethod::Upi, "price fix", &owner)
            .unwrap();
        assert_eq!(amended.invoice_no, "INV-MAIN-000001");
        let _ = std::fs::remove_file(path);
    }

    /// When `pin_reset_required` is set (and PIN is the fallback), the till
    /// stays gated until Owner chooses a new PIN.
    #[test]
    fn pin_reset_required_forces_a_new_owner_pin() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        assert!(!store.pin_reset_required().unwrap());
        store.complete_owner_setup("4321").unwrap();
        store.meta_set("pin_reset_required", "1").unwrap();
        let fallback = hash_pin("0000").unwrap();
        store
            .conn
            .execute("UPDATE staff SET pin_hash = ?1", params![fallback])
            .unwrap();

        assert!(store.pin_reset_required().unwrap());
        assert!(store.authenticate_owner_pin("0000").unwrap().is_some());
        assert!(store.complete_pin_reset("0000").is_err());
        assert!(store.complete_pin_reset("12").is_err());
        let owner = store.complete_pin_reset("9182").unwrap();
        assert!(owner.is_owner());
        assert!(!store.pin_reset_required().unwrap());
        assert!(store.authenticate_owner_pin("9182").unwrap().is_some());
        assert!(store.authenticate_owner_pin("0000").unwrap().is_none());
        assert!(store.complete_pin_reset("5555").is_err(), "reset is one-shot");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn platform_and_user_language_sources() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let platform = store.set_platform_language().unwrap();
        assert_eq!(store.language_source().unwrap(), LanguageSource::Platform);
        assert_eq!(platform.language, locale::detect_platform_language());
        assert_eq!(
            store.locale_settings().unwrap().language,
            locale::detect_platform_language()
        );

        let user = store.set_user_language("hi").unwrap();
        assert_eq!(user.language, "hi");
        assert_eq!(store.language_source().unwrap(), LanguageSource::User);
        assert_eq!(store.locale_settings().unwrap().language, "hi");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unit_chains_anchor_cycles_families_and_currency() {
        use rust_decimal::Decimal;
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.complete_owner_setup("0000").unwrap();

        // A custom unit based on another custom unit resolves through
        // the chain to the standard anchor: 1 crate = 2 box, 1 box = 20 cm.
        store.add_unit("box", "Box").unwrap();
        store.add_unit("crate", "Crate").unwrap();
        // Dangling until linked: unusable, but the link may come later.
        assert_eq!(store.resolve_unit("crate"), None);
        store.add_unit_conversion("crate", "box", 2_000).unwrap();
        assert_eq!(store.resolve_unit("crate"), None);
        store.add_unit_conversion("box", "cm", 20_000).unwrap();
        assert_eq!(
            store.resolve_unit("crate"),
            Some(("length".to_string(), Decimal::from(40)))
        );
        assert_eq!(store.factor_to_cm("crate"), Some(Decimal::from(40)));
        // Conversions ride the chain in both directions via the anchor:
        // 1 crate = 40 cm, so 2 crate = 80 cm and 80 cm = 4 box.
        assert_eq!(store.convert_qty_milli(2_000, "crate", "cm").unwrap(), 80_000);
        assert_eq!(store.convert_qty_milli(80_000, "cm", "box").unwrap(), 4_000);

        // Loops are refused: box already leads (through crate's chain
        // in reverse reachability) — closing box → crate would cycle.
        assert!(store.add_unit_conversion("box", "crate", 500).is_err());

        // Cross-family links are refused once both sides are anchored:
        // bag is weight, cm is length.
        store.add_unit("bag", "Bag").unwrap();
        store.add_unit_conversion("bag", "kg", 25_000).unwrap();
        assert!(store.add_unit_conversion("bag", "cm", 10_000).is_err());
        assert!(store.convert_qty_milli(1_000, "bag", "cm").is_err());
        // A weight unit cannot pose as a dimension unit at the desk.
        assert_eq!(store.factor_to_cm("bag"), None);
        assert_eq!(store.factor_to_kg("bag"), Some(Decimal::from(25)));

        // Standard units are fixed: nobody redefines them, and no
        // conversion may be authored *from* one.
        assert!(store.add_unit_conversion("kg", "bag", 40).is_err());

        // Unknown codes must be added as units first.
        assert!(store.add_unit_conversion("sack", "kg", 10_000).is_err());

        // Currencies are never units.
        assert!(store.add_unit("usd", "Dollar").is_err());
        assert!(store.add_unit("inr", "Rupee").is_err());
    }

    #[test]
    fn write_off_ap_cogs_units_and_books_overview() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.complete_owner_setup("0000").unwrap();
        assert_eq!(store.meta_get("schema_version").unwrap().unwrap(), "1");

        // Units + conversion
        let units = store.list_units().unwrap();
        assert!(units.iter().any(|u| u.code == "kg"));
        store.add_unit("bag", "Bag").unwrap();
        store
            .add_unit_conversion("bag", "kg", 25_000)
            .unwrap(); // 1 bag = 25 kg
        let kg = store.convert_qty_milli(1_000, "bag", "kg").unwrap();
        assert_eq!(kg, 25_000);

        // Standard imperial units resolve to the anchors; a custom unit
        // becomes first-class once it has a conversion to the base.
        assert_eq!(
            store.factor_to_cm("in"),
            Some(rust_decimal::Decimal::new(254, 2))
        );
        assert_eq!(
            store.factor_to_kg("lb"),
            Some(rust_decimal::Decimal::new(45_359_237, 8))
        );
        store.add_unit("crate", "Crate").unwrap();
        store.add_unit_conversion("crate", "cm", 40_000).unwrap();
        assert_eq!(
            store.factor_to_cm("crate"),
            Some(rust_decimal::Decimal::from(40))
        );

        // AR write-off to nil
        let party = store
            .add_customer_kind("Bangles King", "", "Customer")
            .unwrap();
        store
            .seed_opening_credit(&party.id, 1_500, &owner)
            .unwrap();
        assert_eq!(store.customer_balance(&party.id).unwrap(), 1_500);
        let written = store
            .write_off_credit(&party.id, 0, "clear small balance", &owner)
            .unwrap();
        assert_eq!(written, 1_500);
        assert_eq!(store.customer_balance(&party.id).unwrap(), 0);

        // Purchase on account → AP + COGS
        store.upsert_product(&test_product("Tea", 10)).unwrap();
        let mut tea = store.list_products(false).unwrap().remove(0);
        tea.unit_label = "kg".into();
        store.upsert_product(&tea).unwrap();
        let supplier = store.add_customer_kind("Tea Traders", "", "Supplier").unwrap();
        let purchase = store
            .record_purchase_ex(
                "Tea Traders",
                &supplier.id,
                true,
                &[(tea.id.clone(), 2_000, 5_000)], // 2 kg @ 50
                "",
                &owner,
            )
            .unwrap();
        assert!(purchase.on_account);
        assert_eq!(purchase.total_paise, 10_000);
        assert_eq!(store.party_payable_balance(&supplier.id).unwrap(), 10_000);
        store
            .settle_payable(&supplier.id, 10_000, PayMethod::Cash, &owner, "")
            .unwrap();
        assert_eq!(store.party_payable_balance(&supplier.id).unwrap(), 0);

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let books = store.period_books(&today, &today).unwrap();
        assert!(books.cogs_paise >= 10_000);
        assert_eq!(books.gross_paise(), books.revenue_paise - books.cogs_paise);
        let overview = store.books_overview_text().unwrap();
        assert!(overview.contains("We owe them") || overview.contains("They owe us"));
        assert!(overview.contains("COGS") || overview.contains("purchases"));

        // Drawer variance on day close
        let _ = store
            .commit_day_close_with_count(&owner, Some(100))
            .unwrap();
        assert!(store.meta_get("last_day_close_counted_cash").unwrap().is_some());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn dual_role_party_roles_on_uuid_and_nets_ar_ap() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        // Names are not unique — two rows with the same display name stay distinct by UUID.
        let a = store
            .add_customer_kind("Peer Traders", "", "Customer")
            .unwrap();
        let other = store
            .add_customer_kind("Peer Traders", "", "Supplier")
            .unwrap();
        assert_ne!(a.id, other.id);

        // Dual role = set both roles on one id (not merge-by-name).
        store
            .set_customer_kind_label(&a.id, "Customer, Supplier")
            .unwrap();
        let peer = store.get_customer(&a.id).unwrap().unwrap();
        assert_eq!(peer.kind_label, "Customer, Supplier");

        let rate = test_product("Rice", 100);
        store.upsert_product(&rate).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&rate);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.money_lane = MoneyLane::Declared;
        store
            .finalize_sale(&draft, PayMethod::Credit, None, Some(&a.id), &owner)
            .unwrap();
        let ar = store.customer_balance(&a.id).unwrap();
        assert!(ar > 0);

        store
            .record_purchase_ex(
                "Supplier bill",
                &a.id,
                true,
                &[(rate.id.clone(), 1, 5_000)],
                "",
                &owner,
            )
            .unwrap();
        let ap = store.party_payable_balance(&a.id).unwrap();
        assert!(ap > 0);

        let positions = store.party_position_report().unwrap();
        let row = positions
            .iter()
            .find(|r| r.customer_id == a.id)
            .expect("peer in net report");
        assert_eq!(row.ar_paise, ar);
        assert_eq!(row.ap_paise, ap);
        assert_eq!(row.net_paise, ar - ap);

        let _ = std::fs::remove_file(path);
    }
}

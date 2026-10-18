//! Local SQLite persistence for Kumkaj (WAL, restart-safe sales).

use crate::domain::{
    format_qty_display, milli_to_whole_qty, new_id, AuditEvent, Branch,
    BranchSnapshot, Category, CreditLedgerEntry, Customer, DayTotals, DiningTable, EventTimes,
    KitchenStation, KotStatus, KotTicket, Money, MoneyLane, MoneyMoveKind, MoneyMovement, OpsReport,
    OrderType, OrgConsolidatedReport, PartyAgingRow, PayMethod, PeriodBooks, Product, Purchase,
    PurchaseLine, QTY_MILLI_UNIT, Sale, SaleDraft, SaleLine, SaleRevision, StaffFeatures,
    StaffMember, StaffRole, StockMovement, TableStatus, TaxRate, UnitConversion, UnitDef,
    VenueMode, STOCK_UNTRACKED,
};
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
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
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
                    server_at TEXT NOT NULL DEFAULT '',
                    awb_no TEXT NOT NULL DEFAULT '',
                    destination TEXT NOT NULL DEFAULT '',
                    consignee_name TEXT NOT NULL DEFAULT '',
                    actual_weight_kg TEXT NOT NULL DEFAULT '0',
                    volumetric_weight_kg TEXT NOT NULL DEFAULT '0',
                    chargeable_weight_kg TEXT NOT NULL DEFAULT '0',
                    tracking_status TEXT NOT NULL DEFAULT 'booked'
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
                    unit_label TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS sale_payments (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
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
                    branch_id TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS kot_tickets (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    status TEXT NOT NULL,
                    summary TEXT NOT NULL,
                    branch_id TEXT NOT NULL DEFAULT '',
                    station_id TEXT NOT NULL DEFAULT ''
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
                    sort_order INTEGER NOT NULL DEFAULT 0
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
                    builtin INTEGER NOT NULL DEFAULT 0
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

        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('schema_version', '1')",
                [],
            )
            .map_err(|e| e.to_string())?;
        // Always pin to current pre-release schema (drop any leftover 2–17 markers).
        self.meta_set("schema_version", "1")?;
        // Pre-release column adds (no-op when CREATE already has them).
        for stmt in [
            "ALTER TABLE sales ADD COLUMN awb_no TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN destination TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN consignee_name TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN actual_weight_kg TEXT NOT NULL DEFAULT '0'",
            "ALTER TABLE sales ADD COLUMN volumetric_weight_kg TEXT NOT NULL DEFAULT '0'",
            "ALTER TABLE sales ADD COLUMN chargeable_weight_kg TEXT NOT NULL DEFAULT '0'",
            "ALTER TABLE sales ADD COLUMN tracking_status TEXT NOT NULL DEFAULT 'booked'",
            "ALTER TABLE sales ADD COLUMN zone_id TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN cod_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN carrier_cost_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN period_bill_id TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN handover_kind TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN carrier_party_id TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN rider_id TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN pod_note TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN extras_json TEXT NOT NULL DEFAULT '[]'",
            "ALTER TABLE products ADD COLUMN zone_id TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE products ADD COLUMN first_kg TEXT NOT NULL DEFAULT '0'",
            "ALTER TABLE products ADD COLUMN first_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE products ADD COLUMN extra_per_kg_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE customers ADD COLUMN credit_limit_paise INTEGER NOT NULL DEFAULT 0",
        ] {
            let _ = self.conn.execute(stmt, []);
        }

        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS zones (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    note TEXT NOT NULL DEFAULT '',
                    active INTEGER NOT NULL DEFAULT 1
                );
                CREATE TABLE IF NOT EXISTS extra_charge_types (
                    id TEXT PRIMARY KEY,
                    kind TEXT NOT NULL DEFAULT 'other',
                    name TEXT NOT NULL,
                    default_paise INTEGER NOT NULL DEFAULT 0,
                    active INTEGER NOT NULL DEFAULT 1
                );
                CREATE TABLE IF NOT EXISTS riders (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    phone TEXT NOT NULL DEFAULT '',
                    vehicle_label TEXT NOT NULL DEFAULT '',
                    active INTEGER NOT NULL DEFAULT 1,
                    note TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS manifests (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    kind TEXT NOT NULL DEFAULT 'carrier',
                    carrier_party_id TEXT NOT NULL DEFAULT '',
                    carrier_name TEXT NOT NULL DEFAULT '',
                    rider_id TEXT NOT NULL DEFAULT '',
                    rider_name TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL DEFAULT 'open'
                );
                CREATE TABLE IF NOT EXISTS manifest_lines (
                    id TEXT PRIMARY KEY,
                    manifest_id TEXT NOT NULL,
                    sale_id TEXT NOT NULL,
                    awb_no TEXT NOT NULL DEFAULT '',
                    destination TEXT NOT NULL DEFAULT '',
                    consignee_name TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS tracking_events (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    awb_no TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL,
                    note TEXT NOT NULL DEFAULT '',
                    pod_ref TEXT NOT NULL DEFAULT '',
                    actor_name TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS period_bills (
                    id TEXT PRIMARY KEY,
                    party_id TEXT NOT NULL,
                    party_name TEXT NOT NULL DEFAULT '',
                    invoice_no TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    from_date TEXT NOT NULL DEFAULT '',
                    to_date TEXT NOT NULL DEFAULT '',
                    total_paise INTEGER NOT NULL DEFAULT 0,
                    sale_ids_json TEXT NOT NULL DEFAULT '[]',
                    note TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS cod_entries (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    awb_no TEXT NOT NULL DEFAULT '',
                    party_id TEXT NOT NULL DEFAULT '',
                    party_name TEXT NOT NULL DEFAULT '',
                    cod_paise INTEGER NOT NULL DEFAULT 0,
                    status TEXT NOT NULL DEFAULT 'pending',
                    created_at TEXT NOT NULL,
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    collected_at TEXT NOT NULL DEFAULT '',
                    remittance_id TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS cod_remittances (
                    id TEXT PRIMARY KEY,
                    party_id TEXT NOT NULL,
                    party_name TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    user_at TEXT NOT NULL DEFAULT '',
                    device_at TEXT NOT NULL DEFAULT '',
                    total_paise INTEGER NOT NULL DEFAULT 0,
                    pay_method TEXT NOT NULL DEFAULT 'cash',
                    note TEXT NOT NULL DEFAULT '',
                    entry_ids_json TEXT NOT NULL DEFAULT '[]'
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
                CREATE INDEX IF NOT EXISTS idx_sales_awb ON sales(awb_no);
                CREATE INDEX IF NOT EXISTS idx_tracking_sale ON tracking_events(sale_id);
                CREATE INDEX IF NOT EXISTS idx_cod_party ON cod_entries(party_id);
                CREATE INDEX IF NOT EXISTS idx_manifest_lines ON manifest_lines(manifest_id);
                "#,
            )
            .map_err(|e| e.to_string())?;

        // Seed common extras once (idempotent by kind).
        for (kind, name, paise) in [
            ("fuel", "Fuel surcharge", 0i64),
            ("docket", "Docket fee", 0),
            ("oda", "ODA", 0),
            ("packing", "Packing", 0),
            ("cod_fee", "COD fee", 0),
        ] {
            let exists: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM extra_charge_types WHERE kind = ?1",
                    params![kind],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if exists == 0 {
                let _ = self.conn.execute(
                    "INSERT INTO extra_charge_types(id, kind, name, default_paise, active)
                     VALUES(?1, ?2, ?3, ?4, 1)",
                    params![new_id(), kind, name, paise],
                );
            }
        }

        for (key, value) in [
            ("venue_mode", "counter"),
            ("onboarding_complete", "0"),
            ("client_paired", "0"),
            ("next_invoice_seq", "1"),
            ("pin_reset_required", "0"),
            ("entity_template", "courier_agency"),
            ("next_awb_seq", "1"),
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
        if self
            .meta_get("org_name")?
            .filter(|s| !s.trim().is_empty())
            .is_none()
        {
            let titled = self.meta_get("venue_title")?.unwrap_or_default();
            let name = if titled.trim().is_empty() {
                "My business".to_string()
            } else {
                titled.trim().to_string()
            };
            self.meta_set("org_name", &name)?;
        }

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
            ("l", "Litre"),
            ("ml", "Millilitre"),
            ("m", "Metre"),
            ("cm", "Centimetre"),
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
                .unwrap_or_else(|| "counter".into()),
        ))
    }

    pub fn set_venue_mode(&self, mode: VenueMode) -> Result<(), String> {
        self.meta_set("venue_mode", mode.as_str())
    }

    pub fn entity_template_id(&self) -> Result<String, String> {
        Ok(entity_template::COURIER.id.into())
    }

    pub fn entity_template(&self) -> Result<&'static EntityTemplate, String> {
        Ok(&entity_template::COURIER)
    }

    /// Lock this binary to Kumkaj Courier (ignores other template ids).
    pub fn apply_entity_template(&self, _template_id: &str) -> Result<&'static EntityTemplate, String> {
        self.assert_server_writer("Apply start template")?;
        let tmpl = &entity_template::COURIER;
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

    pub fn set_client_pairing(&self, server_url: &str, pairing_code: &str) -> Result<(), String> {
        let url = server_url.trim().trim_end_matches('/');
        if url.is_empty() {
            return Err("Server address required".into());
        }
        self.meta_set("server_url", url)?;
        self.meta_set("pairing_code", pairing_code.trim())?;
        self.meta_set("client_paired", "1")?;
        Ok(())
    }

    pub fn clear_client_pairing(&self) -> Result<(), String> {
        self.meta_set("client_paired", "0")?;
        self.meta_set("server_url", "")?;
        Ok(())
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
        let mut out = String::new();
        out.push_str(venue);
        out.push('\n');
        let business_id = fiscal_settings.business_tax_id.trim();
        if !business_id.is_empty() {
            out.push_str(&format!(
                "{} {business_id}\n",
                fiscal_settings.business_id_label
            ));
        }
        out.push_str(&fiscal_settings.format_date(&sale.created_at));
        if let Some(clock) = sale.created_at.get(11..19) {
            out.push(' ');
            out.push_str(clock);
        }
        out.push('\n');
        if let Some(cid) = sale.customer_id.as_deref().filter(|s| !s.is_empty()) {
            if let Some(party) = self.get_customer(cid)? {
                out.push_str("Party: ");
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
        let lane = MoneyLane::parse(&sale.money_lane);
        let lane_label = match lane {
            MoneyLane::Declared => tmpl.lane_declared_label,
            MoneyLane::Owner => tmpl.lane_owner_label,
        };
        out.push_str("Bill type: ");
        out.push_str(lane_label);
        out.push('\n');
        if !sale.awb_no.trim().is_empty() {
            out.push_str("AWB: ");
            out.push_str(sale.awb_no.trim());
            out.push('\n');
            // Plain-text barcode payload — label printers / scanners read the AWB.
            out.push_str("*");
            out.push_str(sale.awb_no.trim());
            out.push_str("*\n");
        }
        if !sale.destination.trim().is_empty() {
            out.push_str("To: ");
            out.push_str(sale.destination.trim());
            out.push('\n');
        }
        if !sale.consignee_name.trim().is_empty() {
            out.push_str("Consignee: ");
            out.push_str(sale.consignee_name.trim());
            out.push('\n');
        }
        if sale.chargeable_weight_kg > crate::precise::ZERO {
            out.push_str(&format!(
                "Weight: actual {} kg · vol {} kg · chargeable {} kg\n",
                crate::precise::format_kg(sale.actual_weight_kg),
                crate::precise::format_kg(sale.volumetric_weight_kg),
                crate::precise::format_kg(sale.chargeable_weight_kg)
            ));
        }
        if !sale.tracking_status.trim().is_empty() {
            out.push_str("Status: ");
            out.push_str(sale.tracking_status.trim());
            out.push('\n');
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
        out.push_str(&format!("Bill {}\n", sale.bill_no()));
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
            id,
            name,
            phone,
            note,
            active: active != 0,
            balance_paise,
            kind_label,
            credit_limit_paise,
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
            Money(t.rounding_paise).format_localized(&currency, &fiscal_settings),
        ))
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
        let expected_cash = self.today_totals()?.cash_paise;
        if let Some(counted) = counted_cash_paise {
            let variance = counted - expected_cash;
            let currency = self.locale_settings()?.currency;
            let fiscal = self.fiscal_settings()?;
            summary.push_str(&format!(
                "Drawer counted: {}\nExpected cash (tenders): {}\nVariance: {}\n",
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
                    "INSERT OR REPLACE INTO categories(id, branch_id, name, sort_order)
                     VALUES(?1, ?2, ?3, ?4)",
                    params![c.id, branch_id, c.name, c.sort_order],
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
    pub fn upsert_owner_pin_cache(&self, server_owner: &StaffMember) -> Result<(), String> {
        if !server_owner.is_owner() {
            return Ok(());
        }
        self.conn
            .execute(
                "UPDATE staff SET pin_hash = ?1, name = ?2 WHERE role = 'owner' AND active = 1",
                params![server_owner.pin_hash, server_owner.name],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

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
                    "INSERT INTO tables_dining(id, name, status, active_sale_id, branch_id)
                     VALUES(?1, ?2, ?3, ?4, ?5)",
                    params![
                        t.id,
                        t.name,
                        t.status.as_str(),
                        t.active_sale_id,
                        self.active_branch_id()?
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
                        "INSERT INTO sale_lines(sale_id, product_id, name, unit_price_paise, qty, line_total_paise, note, qty_milli, unit_label)
                         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            s.id,
                            line.product_id,
                            line.name,
                            line.unit_price_paise,
                            line.qty,
                            line.line_total_paise,
                            line.note,
                            if line.qty_milli > 0 { line.qty_milli } else { line.qty.saturating_mul(1000) },
                            line.unit_label
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

    pub fn save_table_draft(&self, table_id: &str, draft_json: &str) -> Result<(), String> {
        self.meta_set(&format!("table_draft_{table_id}"), draft_json)?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?1, active_sale_id = ?2 WHERE id = ?3",
                params![TableStatus::Occupied.as_str(), format!("draft:{table_id}"), table_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_table_draft(&self, table_id: &str) -> Result<Option<String>, String> {
        self.meta_get(&format!("table_draft_{table_id}"))
    }

    pub fn clear_table_draft(&self, table_id: &str) -> Result<(), String> {
        self.assert_server_writer("Clear a table")?;
        self.meta_set(&format!("table_draft_{table_id}"), "")?;
        self.free_table(table_id)?;
        Ok(())
    }

    /// Create a kitchen ticket from the current cart without charging (Tables & kitchen on).
    pub fn send_kot_from_cart(
        &self,
        draft: &SaleDraft,
        table_id: Option<&str>,
        actor: &StaffMember,
    ) -> Result<String, String> {
        if !self.venue_mode()?.creates_kot() {
            return Err("Turn on Tables & kitchen in Settings to send kitchen tickets".into());
        }
        if draft.lines.is_empty() {
            return Err("Cart is empty".into());
        }
        let summary = draft
            .lines
            .iter()
            .map(kot_line_label)
            .collect::<Vec<_>>()
            .join(", ");
        let table_label = if let Some(tid) = table_id {
            self.list_tables()?
                .into_iter()
                .find(|t| t.id == tid)
                .map(|t| format!("{} · ", t.name))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let kot_id = new_id();
        let created_at = Local::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO kot_tickets(
                    id, sale_id, created_at, status, summary, branch_id, station_id
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, '')",
                params![
                    kot_id,
                    format!("prepay:{}", actor.id),
                    created_at,
                    KotStatus::New.as_str(),
                    format!("{table_label}{summary}"),
                    self.active_branch_id()?
                ],
            )
            .map_err(|e| e.to_string())?;
        if let Some(tid) = table_id {
            let _ = self.save_table_draft(tid, "{}");
            self.conn
                .execute(
                    "UPDATE tables_dining SET status = ?1 WHERE id = ?2",
                    params![TableStatus::Occupied.as_str(), tid],
                )
                .map_err(|e| e.to_string())?;
        }
        self.audit(actor, "kot.send", &kot_id)?;
        Ok(kot_id)
    }

    /// Charge a bill with any mix of Cash / UPI(rail) / Card / Credit.
    /// Parts must be positive and sum exactly to the charged (taxed) total.
    /// Credit portion needs `customer_id` (rest can be paid now; remainder on khata).
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
        if parts.iter().any(|(_, a)| *a <= 0) {
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
    ) -> Result<(), String> {
        self.assert_server_writer("Log expenses")?;
        if amount_paise <= 0 {
            return Err("Expense must be > 0".into());
        }
        if !actor.is_owner() && !actor.role.can_void_refund() {
            return Err("Not allowed to log expenses".into());
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
        self.conn
            .execute(
                "INSERT INTO expenses(
                    id, created_at, amount_paise, category, note, actor_id, actor_name, branch_id,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    new_id(),
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
        self.audit(actor, "expense.add", &format!("{cat} {amount_paise}"))?;
        Ok(())
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
                        COALESCE(server_at, '')
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
        Ok(format!(
            "Period books {from_day} → {to_day}\n\
             Sales: {}\n\
             Revenue {}\n\
             COGS {}\n\
             Gross (rev − COGS) {}\n\
             {} {}\n\
             {} {}\n\
             Cash {} · {} {} · Card {}\n\
             {} sales {}\n\
             Expenses {} ({})\n\
             Net (gross − expenses) {}\n\
             Bank in {} · Bank out {}\n\
             Bank balance now {}\n\
             They owe us {}\n\
             We owe them {}\n\
             {}\n",
            books.sale_count,
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
        out.push_str(&self.party_aging_text()?);
        out.push('\n');
        out.push_str(&self.payable_aging_text()?);
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

    pub fn replace_kots_from_sync(&self, kots: &[KotTicket]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM kot_tickets", [])
            .map_err(|e| e.to_string())?;
        for k in kots {
            self.conn
                .execute(
                    "INSERT INTO kot_tickets(
                        id, sale_id, created_at, status, summary, branch_id, station_id
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        k.id,
                        k.sale_id,
                        k.created_at,
                        k.status.as_str(),
                        k.summary,
                        self.active_branch_id()?,
                        k.station_id
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
                "SELECT id, name, COALESCE(code, ''), bps, active
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

    /// In-app venue name (header / lock). Empty → neutral "Your shop" in UI.
    /// Does not change OS / Play Store app name.
    pub fn venue_title(&self) -> Result<String, String> {
        Ok(self.meta_get("venue_title")?.unwrap_or_default())
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
            .unwrap_or("png")
            .to_ascii_lowercase();
        let allowed = ["webp", "png", "jpg", "jpeg", "gif", "bmp"];
        if !allowed.contains(&ext.as_str()) {
            return Err("Use webp, png, jpg, gif, or bmp".into());
        }
        let file_name = format!("venue_icon.{ext}");
        let dest = dir.join(&file_name);
        fs::copy(source, &dest).map_err(|e| e.to_string())?;
        let key = format!("custom:{file_name}");
        self.set_venue_icon_key(Some(&key))?;
        Ok(key)
    }

    // ---------------------------------------------------------------- org / branches

    pub fn org_id(&self) -> Result<String, String> {
        Ok(self.meta_get("org_id")?.unwrap_or_default())
    }

    pub fn org_name(&self) -> Result<String, String> {
        Ok(self
            .meta_get("org_name")?
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "My business".into()))
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
                "SELECT id, COALESCE(branch_id, ''), name, sort_order FROM categories
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
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
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
        };
        self.conn
            .execute(
                "INSERT INTO categories(id, branch_id, name, sort_order) VALUES(?1, ?2, ?3, ?4)",
                params![cat.id, cat.branch_id, cat.name, cat.sort_order],
            )
            .map_err(|e| e.to_string())?;
        Ok(cat)
    }

    pub fn list_stations(&self) -> Result<Vec<KitchenStation>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, COALESCE(branch_id, ''), name, sort_order FROM kitchen_stations
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
        };
        self.conn
            .execute(
                "INSERT INTO kitchen_stations(id, branch_id, name, sort_order)
                 VALUES(?1, ?2, ?3, ?4)",
                params![station.id, station.branch_id, station.name, station.sort_order],
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
                    tax_rate_id, aliases_json, unit_label,
                    zone_id, first_kg, first_paise, extra_per_kg_paise
                 )
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
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
                   zone_id=excluded.zone_id,
                   first_kg=excluded.first_kg,
                   first_paise=excluded.first_paise,
                   extra_per_kg_paise=excluded.extra_per_kg_paise",
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
                    product.zone_id,
                    crate::precise::dec_to_sql(product.first_kg),
                    product.first_paise,
                    product.extra_per_kg_paise
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

    pub fn import_item_image(&self, product_id: &str, source: &Path) -> Result<String, String> {
        let dir = self.item_images_dir()?;
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("webp")
            .to_ascii_lowercase();
        let allowed = ["webp", "png", "jpg", "jpeg", "gif", "bmp"];
        if !allowed.contains(&ext.as_str()) {
            return Err("Use webp, png, jpg, gif, or bmp".into());
        }
        let file_name = format!("{product_id}.{ext}");
        let dest = dir.join(&file_name);
        fs::copy(source, &dest).map_err(|e| e.to_string())?;
        Ok(format!("custom:{file_name}"))
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
        draft.assert_balanced()?;
        let credit_part = parts
            .iter()
            .find(|(m, _)| m.is_credit())
            .map(|(_, a)| *a);
        let needs_customer = pay.is_credit() || credit_part.is_some();
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
            // Credit limit gate for account bookings.
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
                let gate = crate::consignments::CreditGate::check(
                    cust.balance_paise,
                    cust.credit_limit_paise,
                    credit_amount_estimate,
                );
                if let crate::consignments::CreditGate::OverLimit {
                    balance_paise,
                    limit_paise,
                    ..
                } = gate
                {
                    return Err(format!(
                        "Credit limit exceeded: outstanding {} / limit {} (paise)",
                        balance_paise, limit_paise
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
            let label = kot_line_label(line);
            match by_station.iter_mut().find(|(sid, _)| *sid == station_id) {
                Some((_, items)) => items.push(label),
                None => by_station.push((station_id, vec![label])),
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
        let awb_no = if draft.awb_no.trim().is_empty() {
            reserve_awb_no(&tx)?
        } else {
            draft.awb_no.trim().to_string()
        };
        let destination = draft.destination.trim().to_string();
        let consignee_name = draft.consignee_name.trim().to_string();
        let actual_weight_kg = draft.actual_weight_kg;
        let volumetric_weight_kg = draft.volumetric_weight_kg;
        let chargeable_weight_kg = draft.chargeable_weight_kg;
        let zone_id = draft.zone_id.trim().to_string();
        let cod_paise = draft.cod_paise.max(0);
        let carrier_cost_paise = draft.carrier_cost_paise.max(0);
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
                awb_no, destination, consignee_name,
                actual_weight_kg, volumetric_weight_kg, chargeable_weight_kg, tracking_status,
                zone_id, cod_paise, carrier_cost_paise, extras_json
             )
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'completed', ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, 'booked', ?29, ?30, ?31, ?32)",
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
                awb_no,
                destination,
                consignee_name,
                crate::precise::dec_to_sql(actual_weight_kg),
                crate::precise::dec_to_sql(volumetric_weight_kg),
                crate::precise::dec_to_sql(chargeable_weight_kg),
                zone_id,
                cod_paise,
                carrier_cost_paise,
                extras_json
            ],
        )
        .map_err(|e| e.to_string())?;

        // Initial tracking event.
        tx.execute(
            "INSERT INTO tracking_events(
                id, sale_id, awb_no, created_at, user_at, device_at,
                status, note, pod_ref, actor_name
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'booked', '', '', ?7)",
            params![
                new_id(),
                sale_id,
                awb_no,
                created_at,
                times.user_at,
                times.device_at,
                created_by_name
            ],
        )
        .map_err(|e| e.to_string())?;

        if cod_paise > 0 {
            let (party_id, party_name) = if let Some(cid) = customer_id {
                let name: String = tx
                    .query_row(
                        "SELECT name FROM customers WHERE id = ?1",
                        params![cid],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(|e| e.to_string())?
                    .unwrap_or_default();
                (cid.to_string(), name)
            } else {
                (String::new(), String::new())
            };
            tx.execute(
                "INSERT INTO cod_entries(
                    id, sale_id, awb_no, party_id, party_name, cod_paise, status,
                    created_at, user_at, device_at, collected_at, remittance_id, note
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8, ?9, '', '', '')",
                params![
                    new_id(),
                    sale_id,
                    awb_no,
                    party_id,
                    party_name,
                    cod_paise,
                    created_at,
                    times.user_at,
                    times.device_at
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        if !draft.notify_phone.trim().is_empty() {
            let track = crate::notify::public_track_path(&awb_no);
            let row = crate::notify::draft_booking_alert(
                draft.notify_phone.trim(),
                &awb_no,
                &destination,
                &track,
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
                "INSERT INTO sale_lines(sale_id, product_id, name, unit_price_paise, qty, line_total_paise, note, qty_milli, unit_label)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    sale_id,
                    line.product_id,
                    line.name,
                    line.unit_price_paise,
                    line.qty,
                    line.line_total_paise,
                    line.note,
                    if line.qty_milli > 0 { line.qty_milli } else { line.qty.saturating_mul(1000) },
                    line.unit_label
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

        // Counter mode: skip kitchen tickets so takeaway cash flow stays quiet.
        let venue_mode = VenueMode::parse(
            &tx.query_row(
                "SELECT value FROM meta WHERE key = 'venue_mode'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "counter".into()),
        );
        if venue_mode.creates_kot() {
            // One ticket per station on the bill so a big kitchen does not have
            // to read the whole order at every counter. Items with no station
            // land on a single unassigned ticket.
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
                        id, sale_id, created_at, status, summary, branch_id, station_id
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        new_id(),
                        sale_id,
                        created_at,
                        KotStatus::New.as_str(),
                        format!("{table_prefix}{head}{}", items.join(", ")),
                        branch_id,
                        station_id
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        if let Some(tid) = table_id {
            tx.execute(
                "UPDATE tables_dining SET status = ?1, active_sale_id = NULL WHERE id = ?2",
                params![TableStatus::Available.as_str(), tid],
            )
            .map_err(|e| e.to_string())?;
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
            awb_no,
            destination,
            consignee_name,
            actual_weight_kg,
            volumetric_weight_kg,
            chargeable_weight_kg,
            tracking_status: "booked".into(),
            zone_id,
            cod_paise,
            carrier_cost_paise,
            period_bill_id: String::new(),
            handover_kind: String::new(),
            carrier_party_id: String::new(),
            rider_id: String::new(),
            pod_note: String::new(),
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
                "INSERT INTO sale_lines(sale_id, product_id, name, unit_price_paise, qty, line_total_paise, note, qty_milli, unit_label)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    sale_id,
                    line.product_id,
                    line.name,
                    line.unit_price_paise,
                    line.qty,
                    line.line_total_paise,
                    line.note,
                    if line.qty_milli > 0 { line.qty_milli } else { line.qty.saturating_mul(1000) },
                    line.unit_label
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
            awb_no: previous.awb_no,
            destination: previous.destination,
            consignee_name: previous.consignee_name,
            actual_weight_kg: previous.actual_weight_kg,
            volumetric_weight_kg: previous.volumetric_weight_kg,
            chargeable_weight_kg: previous.chargeable_weight_kg,
            tracking_status: previous.tracking_status,
            zone_id: previous.zone_id,
            cod_paise: previous.cod_paise,
            carrier_cost_paise: previous.carrier_cost_paise,
            period_bill_id: previous.period_bill_id,
            handover_kind: previous.handover_kind,
            carrier_party_id: previous.carrier_party_id,
            rider_id: previous.rider_id,
            pod_note: previous.pod_note,
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
                        COALESCE(note, ''), COALESCE(qty_milli, 0), COALESCE(unit_label, '')
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
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, status, active_sale_id FROM tables_dining
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
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn occupy_table(&self, table_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?1 WHERE id = ?2",
                params![TableStatus::Occupied.as_str(), table_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn free_table(&self, table_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?1, active_sale_id = NULL WHERE id = ?2",
                params![TableStatus::Available.as_str(), table_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_open_kots(&self) -> Result<Vec<KotTicket>, String> {
        let branch = self.active_branch_id()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT k.id, k.sale_id, k.created_at, k.status, k.summary,
                        COALESCE(k.station_id, ''), COALESCE(s.name, '')
                 FROM kot_tickets k
                 LEFT JOIN kitchen_stations s ON s.id = k.station_id
                 WHERE k.status NOT IN ('completed', 'cancelled')
                   AND (k.branch_id = ?1 OR k.branch_id = '')
                 ORDER BY k.created_at ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![branch], |row| {
                let status: String = row.get(3)?;
                Ok(KotTicket {
                    id: row.get(0)?,
                    sale_id: row.get(1)?,
                    created_at: row.get(2)?,
                    status: KotStatus::parse(&status),
                    summary: row.get(4)?,
                    station_id: row.get(5)?,
                    station_name: row.get(6)?,
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
        let id = new_id();
        let created_at = Local::now().to_rfc3339();
        let phone = phone.trim().to_string();
        let kind_label = kind_label.trim().to_string();
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
        })
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
            out.push(Customer {
                id,
                name,
                phone,
                note,
                active: active != 0,
                balance_paise,
                kind_label,
                credit_limit_paise,
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
            self.conn
                .execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail,
                        user_at, device_at, server_at
                     ) VALUES(?1, ?2, ?3, 'settle', ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        new_id(),
                        customer_id,
                        times.user_at,
                        -settle_part,
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

    /// Pay down AP (we owed them). Excess is not tracked as receivable here.
    pub fn settle_payable(
        &mut self,
        party_id: &str,
        amount_paise: i64,
        pay: PayMethod,
        actor: &StaffMember,
        detail: &str,
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
        self.conn
            .execute(
                "INSERT INTO payable_ledger(
                    id, party_id, created_at, kind, amount_paise, purchase_id,
                    pay_method, actor_id, actor_name, detail,
                    user_at, device_at, server_at
                 ) VALUES(?1, ?2, ?3, 'settle', ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    new_id(),
                    party_id,
                    times.user_at,
                    -settle,
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
            .prepare("SELECT id, code, name, builtin FROM units ORDER BY builtin DESC, code")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(UnitDef {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    builtin: row.get::<_, i64>(3)? != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn add_unit(&self, code: &str, name: &str) -> Result<UnitDef, String> {
        self.assert_server_writer("Add unit")?;
        let code = code.trim().to_ascii_lowercase();
        let name = name.trim();
        if code.is_empty() || name.is_empty() {
            return Err("Unit code and name required".into());
        }
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO units(id, code, name, builtin) VALUES(?1, ?2, ?3, 0)",
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

    /// Convert qty_milli from `from_code` into `to_code` using a direct conversion row.
    pub fn convert_qty_milli(
        &self,
        qty_milli: i64,
        from_code: &str,
        to_code: &str,
    ) -> Result<i64, String> {
        let from_code = from_code.trim().to_ascii_lowercase();
        let to_code = to_code.trim().to_ascii_lowercase();
        if from_code == to_code {
            return Ok(qty_milli);
        }
        let factor: i64 = self
            .conn
            .query_row(
                "SELECT factor_milli FROM unit_conversions
                 WHERE from_code = ?1 AND to_code = ?2",
                params![from_code, to_code],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("No conversion {from_code} → {to_code}"))?;
        Ok((qty_milli.saturating_mul(factor)) / QTY_MILLI_UNIT)
    }

    /// Honest Books home: money in/out, AR/AP, cash vs bank, period profit shape.
    pub fn books_overview_text(&self) -> Result<String, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let books = self.period_books(&today, &today)?;
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let tmpl = self.entity_template()?;
        let expenses_today = books.expenses_paise;
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
            Money(books.revenue_paise).format_localized(&currency, &fiscal),
            Money(books.cogs_paise).format_localized(&currency, &fiscal),
            Money(expenses_today).format_localized(&currency, &fiscal),
            Money(books.gross_paise()).format_localized(&currency, &fiscal),
            Money(books.net_paise()).format_localized(&currency, &fiscal),
            tmpl.lane_declared_label,
            Money(books.declared_paise).format_localized(&currency, &fiscal),
            tmpl.lane_owner_label,
            Money(books.owner_lane_paise).format_localized(&currency, &fiscal),
            Money(books.cash_paise).format_localized(&currency, &fiscal),
            fiscal.digital_pay_label(),
            Money(books.upi_paise).format_localized(&currency, &fiscal),
            Money(books.card_paise).format_localized(&currency, &fiscal),
            Money(books.outstanding_credit_paise).format_localized(&currency, &fiscal),
            Money(books.outstanding_payable_paise).format_localized(&currency, &fiscal),
            Money(self.bank_balance_paise()?).format_localized(&currency, &fiscal),
            Money(books.bank_in_paise.saturating_sub(books.bank_out_paise))
                .format_localized(&currency, &fiscal),
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
        self.conn
            .execute(
                "INSERT INTO audit_events(id, created_at, actor_id, actor_name, action, detail)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    new_id(),
                    Local::now().to_rfc3339(),
                    actor.id,
                    actor.name,
                    action,
                    detail
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn backup_to(&self, dest: &Path) -> Result<(), String> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        self.conn
            .execute("VACUUM INTO ?1", params![dest.to_string_lossy().as_ref()])
            .map_err(|e| e.to_string())?;
        Ok(())
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

    // —— Courier BOS: zones, consignments, fleet/carrier ops, COD, labels ——

    pub fn list_zones(&self) -> Result<Vec<crate::rates::Zone>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, note, active FROM zones WHERE active = 1 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(crate::rates::Zone {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    note: r.get(2)?,
                    active: r.get::<_, i64>(3)? != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn upsert_zone(&self, zone: &crate::rates::Zone) -> Result<(), String> {
        self.assert_server_writer("Zones")?;
        self.conn
            .execute(
                "INSERT INTO zones(id, name, note, active) VALUES(?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name, note=excluded.note, active=excluded.active",
                params![zone.id, zone.name, zone.note, zone.active as i64],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_zone(&self, name: &str) -> Result<crate::rates::Zone, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Zone name required".into());
        }
        let zone = crate::rates::Zone {
            id: new_id(),
            name: name.into(),
            note: String::new(),
            active: true,
        };
        self.upsert_zone(&zone)?;
        Ok(zone)
    }

    pub fn list_extra_charge_types(&self) -> Result<Vec<crate::rates::ExtraChargeType>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, name, default_paise, active FROM extra_charge_types
                 WHERE active = 1 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(crate::rates::ExtraChargeType {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    name: r.get(2)?,
                    default_paise: r.get(3)?,
                    active: r.get::<_, i64>(4)? != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_riders(&self) -> Result<Vec<crate::manifest::Rider>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, phone, vehicle_label, active, note FROM riders
                 WHERE active = 1 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(crate::manifest::Rider {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    phone: r.get(2)?,
                    vehicle_label: r.get(3)?,
                    active: r.get::<_, i64>(4)? != 0,
                    note: r.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn upsert_rider(&self, rider: &crate::manifest::Rider) -> Result<(), String> {
        self.assert_server_writer("Riders")?;
        self.conn
            .execute(
                "INSERT INTO riders(id, name, phone, vehicle_label, active, note)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name, phone=excluded.phone,
                   vehicle_label=excluded.vehicle_label, active=excluded.active,
                   note=excluded.note",
                params![
                    rider.id,
                    rider.name,
                    rider.phone,
                    rider.vehicle_label,
                    rider.active as i64,
                    rider.note
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_rider(&self, name: &str, phone: &str, vehicle: &str) -> Result<crate::manifest::Rider, String> {
        let mut r = crate::manifest::Rider::new(name);
        r.phone = phone.trim().into();
        r.vehicle_label = vehicle.trim().into();
        if r.name.trim().is_empty() {
            return Err("Rider name required".into());
        }
        self.upsert_rider(&r)?;
        Ok(r)
    }

    pub fn search_consignments(
        &self,
        filter: &crate::consignments::ConsignmentFilter,
        limit: i64,
    ) -> Result<Vec<Sale>, String> {
        let mut sales = self.recent_sales(limit.max(50))?;
        sales.retain(|s| !s.awb_no.trim().is_empty() && filter.matches(s));
        Ok(sales)
    }

    pub fn find_sale_by_awb(&self, awb: &str) -> Result<Option<Sale>, String> {
        let awb = awb.trim();
        if awb.is_empty() {
            return Ok(None);
        }
        let mut sale = self
            .conn
            .query_row(
                &format!("SELECT {SALE_COLUMNS} FROM sales WHERE awb_no = ?1 LIMIT 1"),
                params![awb],
                row_to_sale,
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(ref mut s) = sale {
            s.lines = self.sale_lines(&s.id)?;
        }
        Ok(sale)
    }

    pub fn set_sale_carrier_cost(&self, sale_id: &str, carrier_cost_paise: i64) -> Result<(), String> {
        self.assert_server_writer("Carrier cost")?;
        self.conn
            .execute(
                "UPDATE sales SET carrier_cost_paise = ?1 WHERE id = ?2",
                params![carrier_cost_paise.max(0), sale_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn advance_tracking(
        &mut self,
        sale_id: &str,
        status: crate::manifest::TrackingStatus,
        note: &str,
        pod_ref: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Tracking")?;
        let sale = self
            .get_sale(sale_id)?
            .ok_or_else(|| "Sale not found".to_string())?;
        let times = EventTimes::capture(None);
        self.conn
            .execute(
                "UPDATE sales SET tracking_status = ?1, pod_note = CASE
                     WHEN ?2 = '' THEN pod_note ELSE ?2 END
                 WHERE id = ?3",
                params![status.as_str(), note.trim(), sale_id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT INTO tracking_events(
                    id, sale_id, awb_no, created_at, user_at, device_at,
                    status, note, pod_ref, actor_name
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    new_id(),
                    sale_id,
                    sale.awb_no,
                    times.user_at,
                    times.user_at,
                    times.device_at,
                    status.as_str(),
                    note.trim(),
                    pod_ref.trim(),
                    actor.lock_label()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_tracking_events(&self, sale_id: &str) -> Result<Vec<crate::manifest::TrackingEvent>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, sale_id, awb_no, created_at, user_at, device_at,
                        status, note, pod_ref, actor_name
                 FROM tracking_events WHERE sale_id = ?1 ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![sale_id], |r| {
                Ok(crate::manifest::TrackingEvent {
                    id: r.get(0)?,
                    sale_id: r.get(1)?,
                    awb_no: r.get(2)?,
                    created_at: r.get(3)?,
                    user_at: r.get(4)?,
                    device_at: r.get(5)?,
                    status: r.get(6)?,
                    note: r.get(7)?,
                    pod_ref: r.get(8)?,
                    actor_name: r.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn public_track(&self, awb: &str) -> Result<Option<crate::track::PublicTrackView>, String> {
        let Some(sale) = self.find_sale_by_awb(awb)? else {
            return Ok(None);
        };
        let events = self.list_tracking_events(&sale.id)?;
        Ok(Some(crate::track::view_from_sale(&sale, &events)))
    }

    /// Create a carrier bag or own-fleet run sheet and mark dockets handed over / out.
    pub fn create_manifest(
        &mut self,
        kind: crate::manifest::HandoverKind,
        carrier_party_id: &str,
        rider_id: &str,
        sale_ids: &[String],
        note: &str,
        actor: &StaffMember,
    ) -> Result<crate::manifest::Manifest, String> {
        self.assert_server_writer("Manifest")?;
        if sale_ids.is_empty() {
            return Err("Pick at least one AWB".into());
        }
        let times = EventTimes::capture(None);
        let id = new_id();
        let (carrier_name, rider_name) = match kind {
            crate::manifest::HandoverKind::Carrier => {
                let c = self
                    .get_customer(carrier_party_id)?
                    .ok_or_else(|| "Carrier party required".to_string())?;
                (c.name, String::new())
            }
            crate::manifest::HandoverKind::OwnFleet => {
                let riders = self.list_riders()?;
                let r = riders
                    .into_iter()
                    .find(|x| x.id == rider_id)
                    .ok_or_else(|| "Rider required for own-fleet dispatch".to_string())?;
                (String::new(), r.name)
            }
        };
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO manifests(
                id, created_at, user_at, device_at, kind,
                carrier_party_id, carrier_name, rider_id, rider_name, note, status
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'open')",
            params![
                id,
                times.user_at,
                times.user_at,
                times.device_at,
                kind.as_str(),
                carrier_party_id,
                carrier_name,
                rider_id,
                rider_name,
                note.trim()
            ],
        )
        .map_err(|e| e.to_string())?;

        let next_status = match kind {
            crate::manifest::HandoverKind::Carrier => {
                crate::manifest::TrackingStatus::HandedOver
            }
            crate::manifest::HandoverKind::OwnFleet => {
                crate::manifest::TrackingStatus::OutForDelivery
            }
        };
        let mut line_count = 0i64;
        for sid in sale_ids {
            let sale: Option<(String, String, String)> = tx
                .query_row(
                    "SELECT awb_no, destination, consignee_name FROM sales WHERE id = ?1",
                    params![sid],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            let Some((awb, dest, consignee)) = sale else {
                continue;
            };
            tx.execute(
                "INSERT INTO manifest_lines(
                    id, manifest_id, sale_id, awb_no, destination, consignee_name
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                params![new_id(), id, sid, awb, dest, consignee],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE sales SET tracking_status = ?1, handover_kind = ?2,
                     carrier_party_id = ?3, rider_id = ?4
                 WHERE id = ?5",
                params![
                    next_status.as_str(),
                    kind.as_str(),
                    carrier_party_id,
                    rider_id,
                    sid
                ],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT INTO tracking_events(
                    id, sale_id, awb_no, created_at, user_at, device_at,
                    status, note, pod_ref, actor_name
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '', ?9)",
                params![
                    new_id(),
                    sid,
                    awb,
                    times.user_at,
                    times.user_at,
                    times.device_at,
                    next_status.as_str(),
                    note.trim(),
                    actor.lock_label()
                ],
            )
            .map_err(|e| e.to_string())?;
            line_count += 1;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(crate::manifest::Manifest {
            id,
            created_at: times.user_at.clone(),
            user_at: times.user_at,
            device_at: times.device_at,
            kind: kind.as_str().into(),
            carrier_party_id: carrier_party_id.into(),
            carrier_name,
            rider_id: rider_id.into(),
            rider_name,
            note: note.trim().into(),
            status: "open".into(),
            line_count,
        })
    }

    pub fn list_manifests(&self, limit: i64) -> Result<Vec<crate::manifest::Manifest>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.created_at, m.user_at, m.device_at, m.kind,
                        m.carrier_party_id, m.carrier_name, m.rider_id, m.rider_name,
                        m.note, m.status,
                        (SELECT COUNT(*) FROM manifest_lines l WHERE l.manifest_id = m.id)
                 FROM manifests m ORDER BY m.created_at DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit.max(1)], |r| {
                Ok(crate::manifest::Manifest {
                    id: r.get(0)?,
                    created_at: r.get(1)?,
                    user_at: r.get(2)?,
                    device_at: r.get(3)?,
                    kind: r.get(4)?,
                    carrier_party_id: r.get(5)?,
                    carrier_name: r.get(6)?,
                    rider_id: r.get(7)?,
                    rider_name: r.get(8)?,
                    note: r.get(9)?,
                    status: r.get(10)?,
                    line_count: r.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn create_period_bill(
        &mut self,
        party_id: &str,
        from_date: &str,
        to_date: &str,
        note: &str,
        actor: &StaffMember,
    ) -> Result<crate::consignments::PeriodBill, String> {
        self.assert_server_writer("Period bill")?;
        let party = self
            .get_customer(party_id)?
            .ok_or_else(|| "Party not found".to_string())?;
        let filter = crate::consignments::ConsignmentFilter {
            party_id: party_id.into(),
            from_date: from_date.trim().into(),
            to_date: to_date.trim().into(),
            billed: "unbilled".into(),
            ..Default::default()
        };
        let sales = self.search_consignments(&filter, 2000)?;
        let unbilled: Vec<_> = sales
            .into_iter()
            .filter(|s| s.is_unbilled_account())
            .collect();
        if unbilled.is_empty() {
            return Err("No unbilled account dockets in range".into());
        }
        let total: i64 = unbilled.iter().map(|s| s.total_paise).sum();
        let sale_ids: Vec<String> = unbilled.iter().map(|s| s.id.clone()).collect();
        let times = EventTimes::capture(None);
        let id = new_id();
        let branch_code = self.active_branch()?.code;
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        let invoice_no = reserve_invoice_no(&tx, &branch_code)?;
        let sale_ids_json = serde_json::to_string(&sale_ids).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO period_bills(
                id, party_id, party_name, invoice_no, created_at, user_at, device_at,
                from_date, to_date, total_paise, sale_ids_json, note
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                id,
                party_id,
                party.name,
                invoice_no,
                times.user_at,
                times.user_at,
                times.device_at,
                from_date.trim(),
                to_date.trim(),
                total,
                sale_ids_json,
                note.trim()
            ],
        )
        .map_err(|e| e.to_string())?;
        for sid in &sale_ids {
            tx.execute(
                "UPDATE sales SET period_bill_id = ?1 WHERE id = ?2",
                params![id, sid],
            )
            .map_err(|e| e.to_string())?;
        }
        let _ = actor;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(crate::consignments::PeriodBill {
            id,
            party_id: party_id.into(),
            party_name: party.name,
            invoice_no,
            created_at: times.user_at.clone(),
            user_at: times.user_at,
            device_at: times.device_at,
            from_date: from_date.trim().into(),
            to_date: to_date.trim().into(),
            total_paise: total,
            sale_ids_json: serde_json::to_string(&sale_ids).unwrap_or_else(|_| "[]".into()),
            note: note.trim().into(),
        })
    }

    pub fn list_period_bills(&self, limit: i64) -> Result<Vec<crate::consignments::PeriodBill>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, party_id, party_name, invoice_no, created_at, user_at, device_at,
                        from_date, to_date, total_paise, sale_ids_json, note
                 FROM period_bills ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit.max(1)], |r| {
                Ok(crate::consignments::PeriodBill {
                    id: r.get(0)?,
                    party_id: r.get(1)?,
                    party_name: r.get(2)?,
                    invoice_no: r.get(3)?,
                    created_at: r.get(4)?,
                    user_at: r.get(5)?,
                    device_at: r.get(6)?,
                    from_date: r.get(7)?,
                    to_date: r.get(8)?,
                    total_paise: r.get(9)?,
                    sale_ids_json: r.get(10)?,
                    note: r.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_cod_entries(&self, status: &str) -> Result<Vec<crate::cod::CodEntry>, String> {
        let status = status.trim();
        if !(status.is_empty()
            || status == "any"
            || status == "pending"
            || status == "collected"
            || status == "remitted")
        {
            return Err("Unknown COD status".into());
        }
        let sql = "SELECT id, sale_id, awb_no, party_id, party_name, cod_paise, status,
                    created_at, user_at, device_at, collected_at, remittance_id, note
             FROM cod_entries
             WHERE (?1 = '' OR ?1 = 'any' OR status = ?1)
             ORDER BY created_at DESC LIMIT 500";
        let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![status], |r| {
                Ok(crate::cod::CodEntry {
                    id: r.get(0)?,
                    sale_id: r.get(1)?,
                    awb_no: r.get(2)?,
                    party_id: r.get(3)?,
                    party_name: r.get(4)?,
                    cod_paise: r.get(5)?,
                    status: r.get(6)?,
                    created_at: r.get(7)?,
                    user_at: r.get(8)?,
                    device_at: r.get(9)?,
                    collected_at: r.get(10)?,
                    remittance_id: r.get(11)?,
                    note: r.get(12)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn mark_cod_collected(&self, entry_id: &str) -> Result<(), String> {
        self.assert_server_writer("COD collect")?;
        let times = EventTimes::capture(None);
        self.conn
            .execute(
                "UPDATE cod_entries SET status = 'collected', collected_at = ?1 WHERE id = ?2 AND status = 'pending'",
                params![times.user_at, entry_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remit_cod(
        &mut self,
        party_id: &str,
        entry_ids: &[String],
        pay_method: &str,
        note: &str,
    ) -> Result<crate::cod::CodRemittance, String> {
        self.assert_server_writer("COD remit")?;
        if entry_ids.is_empty() {
            return Err("No COD entries selected".into());
        }
        let party = self
            .get_customer(party_id)?
            .ok_or_else(|| "Party not found".to_string())?;
        let times = EventTimes::capture(None);
        let id = new_id();
        let mut total = 0i64;
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;
        for eid in entry_ids {
            let row: Option<(i64, String)> = tx
                .query_row(
                    "SELECT cod_paise, party_id FROM cod_entries
                     WHERE id = ?1 AND status = 'collected'",
                    params![eid],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            let Some((paise, pid)) = row else {
                return Err(format!("COD entry {eid} not collected"));
            };
            if pid != party_id {
                return Err("COD entry belongs to another party".into());
            }
            total = total.saturating_add(paise);
            tx.execute(
                "UPDATE cod_entries SET status = 'remitted', remittance_id = ?1 WHERE id = ?2",
                params![id, eid],
            )
            .map_err(|e| e.to_string())?;
        }
        let entry_ids_json = serde_json::to_string(entry_ids).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO cod_remittances(
                id, party_id, party_name, created_at, user_at, device_at,
                total_paise, pay_method, note, entry_ids_json
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                party_id,
                party.name,
                times.user_at,
                times.user_at,
                times.device_at,
                total,
                pay_method.trim(),
                note.trim(),
                entry_ids_json
            ],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(crate::cod::CodRemittance {
            id,
            party_id: party_id.into(),
            party_name: party.name,
            created_at: times.user_at.clone(),
            user_at: times.user_at,
            device_at: times.device_at,
            total_paise: total,
            pay_method: pay_method.trim().into(),
            note: note.trim().into(),
            entry_ids_json: serde_json::to_string(entry_ids).unwrap_or_else(|_| "[]".into()),
        })
    }

    pub fn write_awb_label(&self, sale_id: &str) -> Result<PathBuf, String> {
        let sale = self
            .get_sale(sale_id)?
            .ok_or_else(|| "Sale not found".to_string())?;
        if sale.awb_no.trim().is_empty() {
            return Err("Sale has no AWB".into());
        }
        let agency = self.venue_title().unwrap_or_default();
        let from = sale
            .customer_id
            .as_ref()
            .and_then(|id| self.get_customer(id).ok().flatten())
            .map(|c| c.name)
            .unwrap_or_else(|| "Walk-in".into());
        let service = sale
            .lines
            .first()
            .map(|l| l.name.clone())
            .unwrap_or_default();
        let currency_code = self
            .locale_settings()
            .map(|l| l.currency)
            .unwrap_or_else(|_| "INR".into());
        let label = crate::labels::AwbLabel {
            awb_no: sale.awb_no.clone(),
            from_name: from,
            to_name: sale.consignee_name.clone(),
            destination: sale.destination.clone(),
            weight_kg: sale.chargeable_weight_kg,
            pieces: 1,
            service,
            cod_paise: sale.cod_paise,
            agency_name: if agency.trim().is_empty() {
                "Kumkaj Courier".into()
            } else {
                agency
            },
            currency_code,
        };
        let dir = self
            .path
            .parent()
            .unwrap_or(Path::new("."))
            .join("labels");
        label.write_svg_file(&dir)
    }

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

    /// Ops summary text for Reports / More — keeps UI modules thin.
    pub fn courier_ops_summary(&self) -> Result<String, String> {
        let booked: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sales WHERE tracking_status = 'booked' AND awb_no != ''",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let out_delivery: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sales WHERE tracking_status = 'out_for_delivery'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let pending_cod: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(cod_paise),0) FROM cod_entries WHERE status = 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let riders = self.list_riders()?.len();
        let zones = self.list_zones()?.len();
        Ok(format!(
            "Ops: {booked} booked · {out_delivery} out for delivery · {riders} riders · {zones} zones\n\
             COD pending: {}",
            self.money_display(pending_cod)
        ))
    }
}

/// Reverse-DNS app id from this crate's `env/development.env` or `env/production.env`
/// (baked by `build.rs`). Dev: `com.kumkaj.courier.debug`. Prod: `com.kumkaj.courier`.
pub const APP_ID: &str = env!("KUMKAJ_APP_ID");
/// Launcher / window label from the same env file.
pub const APP_LABEL: &str = env!("KUMKAJ_APP_LABEL");
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

/// Every column [`row_to_product`] expects, in order.
const PRODUCT_COLUMNS: &str = "id, name, price_paise, available, archived, image_key,
     COALESCE(category_id, ''), COALESCE(station_id, ''), COALESCE(tax_bps, 0),
     COALESCE(stock_qty, -1), COALESCE(modifiers_json, '[]'), COALESCE(branch_id, ''),
     COALESCE(tax_rate_id, ''), COALESCE(aliases_json, '[]'), COALESCE(unit_label, ''),
     COALESCE(zone_id, ''), COALESCE(first_kg, 0), COALESCE(first_paise, 0),
     COALESCE(extra_per_kg_paise, 0)";

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
     COALESCE(awb_no, ''), COALESCE(destination, ''), COALESCE(consignee_name, ''),
     COALESCE(actual_weight_kg, 0), COALESCE(volumetric_weight_kg, 0),
     COALESCE(chargeable_weight_kg, 0), COALESCE(tracking_status, 'booked'),
     COALESCE(zone_id, ''), COALESCE(cod_paise, 0), COALESCE(carrier_cost_paise, 0),
     COALESCE(period_bill_id, ''), COALESCE(handover_kind, ''),
     COALESCE(carrier_party_id, ''), COALESCE(rider_id, ''),
     COALESCE(pod_note, ''), COALESCE(extras_json, '[]')";

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

fn reserve_awb_no(tx: &Connection) -> Result<String, String> {
    let seq: i64 = tx
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM meta WHERE key = 'next_awb_seq'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or(1)
        .max(1);
    tx.execute(
        "INSERT INTO meta(key, value) VALUES('next_awb_seq', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![(seq + 1).to_string()],
    )
    .map_err(|e| e.to_string())?;
    Ok(crate::booking::format_awb(seq as u64))
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
        awb_no: row.get(23)?,
        destination: row.get(24)?,
        consignee_name: row.get(25)?,
        actual_weight_kg: crate::precise::dec_from_sql(row, 26)?,
        volumetric_weight_kg: crate::precise::dec_from_sql(row, 27)?,
        chargeable_weight_kg: crate::precise::dec_from_sql(row, 28)?,
        tracking_status: row.get(29)?,
        zone_id: row.get(30)?,
        cod_paise: row.get(31)?,
        carrier_cost_paise: row.get(32)?,
        period_bill_id: row.get(33)?,
        handover_kind: row.get(34)?,
        carrier_party_id: row.get(35)?,
        rider_id: row.get(36)?,
        pod_note: row.get(37)?,
        extras_json: row.get(38)?,
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
        zone_id: row.get(15)?,
        first_kg: crate::precise::dec_from_sql(row, 16)?,
        first_paise: row.get(17)?,
        extra_per_kg_paise: row.get(18)?,
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

pub fn default_backup_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("backups").join(format!(
        "kumkaj-backup-{}.sqlite3",
        Local::now().format("%Y%m%d-%H%M%S")
    )))
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
mod tests {
    use super::*;
    use crate::domain::{
        Cart, ModifierOption, MoneyLane, MoneyMoveKind, OrderType, PayMethod, Product, StaffRole,
        VenueMode,
    };
    use crate::fiscal::TaxPricing;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn courier_template_labels_without_seeds_and_kg_lines() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let tmpl = store.apply_entity_template("courier_agency").unwrap();
        assert_eq!(tmpl.id, "courier_agency");
        assert_eq!(store.entity_template_id().unwrap(), "courier_agency");
        assert_eq!(tmpl.qty_unit_hint, "kg");
        assert_eq!(tmpl.lane_declared_label, "Account booking");
        assert_eq!(tmpl.lane_owner_label, "Cash booking");
        // No sample rates or parties — owner adds their own.
        assert!(store.list_products(false).unwrap().is_empty());
        assert!(store.list_customers().unwrap().is_empty());
        store.apply_entity_template("courier_agency").unwrap();
        assert!(store.list_products(false).unwrap().is_empty());
        // Weight × rate: owner-added kg line uses 0.1 kg step on first tap.
        let mut freight = Product::new_basic(new_id(), "Surface freight", Money::from_rupees(40));
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
    fn courier_product_has_no_seed_catalogue() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.apply_entity_template("anything").unwrap();
        assert_eq!(store.entity_template_id().unwrap(), "courier_agency");
        assert!(store.list_products(false).unwrap().is_empty());
        assert!(store.list_customers().unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn booking_assigns_awb_and_destination() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let mut freight = Product::new_basic(new_id(), "Surface Zone A", Money::from_rupees(40));
        freight.unit_label = "kg".into();
        store.upsert_product(&freight).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&freight);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.destination = "Bengaluru".into();
        draft.consignee_name = "Sunita".into();
        draft.actual_weight_kg = "5".parse().unwrap();
        draft.volumetric_weight_kg = "4.8".parse().unwrap();
        draft.chargeable_weight_kg = "5".parse().unwrap();
        draft.money_lane = MoneyLane::Owner;
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &owner)
            .unwrap();
        assert!(sale.awb_no.starts_with("AWB-"));
        assert_eq!(sale.destination, "Bengaluru");
        assert_eq!(sale.consignee_name, "Sunita");
        assert_eq!(sale.tracking_status, "booked");
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains(&sale.awb_no));
        assert!(receipt.contains("Bengaluru"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn courier_bos_zones_slabs_cod_fleet_track_and_bill() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let zone = store.add_zone("West").unwrap();
        let mut rate = Product::new_basic(new_id(), "Surface West", Money::from_rupees(40));
        rate.unit_label = "kg".into();
        rate.zone_id = zone.id.clone();
        rate.first_kg = "1".parse().unwrap();
        rate.first_paise = 50_00;
        rate.extra_per_kg_paise = 20_00;
        store.upsert_product(&rate).unwrap();
        assert_eq!(
            rate.rate_slab()
                .freight_paise("2.5".parse().unwrap())
                .unwrap(),
            80_00
        );

        let party = store.add_customer_kind("Acme Consignor", "", "Consignor").unwrap();
        store.set_customer_credit_limit(&party.id, 500_00).unwrap();
        let rider = store.add_rider("Ravi", "999", "Bike 12").unwrap();

        let mut cart = Cart::default();
        cart.add_or_bump(&rate);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.destination = "Pune".into();
        draft.consignee_name = "Neha".into();
        draft.chargeable_weight_kg = "2.5".parse().unwrap();
        draft.cod_paise = 100_00;
        draft.carrier_cost_paise = 60_00;
        draft.zone_id = zone.id;
        draft.money_lane = MoneyLane::Declared;
        let sale = store
            .finalize_sale(&draft, PayMethod::Credit, None, Some(&party.id), &owner)
            .unwrap();
        assert!(!sale.awb_no.is_empty());
        assert_eq!(sale.cod_paise, 100_00);
        assert_eq!(sale.margin_paise(), sale.total_paise - 60_00);

        let label = store.write_awb_label(&sale.id).unwrap();
        assert!(label.exists());

        let track = store.public_track(&sale.awb_no).unwrap().unwrap();
        assert_eq!(track.status, "booked");

        let manifest = store
            .create_manifest(
                crate::manifest::HandoverKind::OwnFleet,
                "",
                &rider.id,
                &[sale.id.clone()],
                "evening run",
                &owner,
            )
            .unwrap();
        assert_eq!(manifest.line_count, 1);
        let sale2 = store.get_sale(&sale.id).unwrap().unwrap();
        assert_eq!(sale2.tracking_status, "out_for_delivery");
        assert_eq!(sale2.handover_kind, "own_fleet");

        store
            .advance_tracking(
                &sale.id,
                crate::manifest::TrackingStatus::Delivered,
                "signed",
                "",
                &owner,
            )
            .unwrap();
        let cods = store.list_cod_entries("pending").unwrap();
        assert_eq!(cods.len(), 1);
        store.mark_cod_collected(&cods[0].id).unwrap();
        let rem = store
            .remit_cod(&party.id, &[cods[0].id.clone()], "cash", "weekly")
            .unwrap();
        assert_eq!(rem.total_paise, 100_00);

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let bill = store
            .create_period_bill(&party.id, &today, &today, "March", &owner)
            .unwrap();
        assert!(bill.total_paise > 0);
        let billed = store.get_sale(&sale.id).unwrap().unwrap();
        assert_eq!(billed.period_bill_id, bill.id);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bank_posting_aging_period_and_rich_receipt() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        store.apply_entity_template("courier_agency").unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store
            .post_money_movement(MoneyMoveKind::TillToBank, 10_000, "evening deposit", &owner)
            .unwrap();
        assert_eq!(store.bank_balance_paise().unwrap(), 10_000);
        store
            .post_money_movement(MoneyMoveKind::BankOut, 2_000, "office", &owner)
            .unwrap();
        assert_eq!(store.bank_balance_paise().unwrap(), 8_000);

        let mut freight = Product::new_basic(new_id(), "Surface freight", Money::from_rupees(40));
        freight.unit_label = "kg".into();
        store.upsert_product(&freight).unwrap();
        let party = store.add_customer_kind("Sumit Int'l", "", "Consignor").unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&freight);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.money_lane = MoneyLane::Declared;
        let sale = store
            .finalize_sale(&draft, PayMethod::Credit, None, Some(&party.id), &owner)
            .unwrap();
        let receipt = store.format_receipt_text(&sale.id).unwrap();
        assert!(receipt.contains("Party: Sumit"));
        assert!(receipt.contains("kg") || receipt.contains("0.1"));
        assert!(
            receipt.contains("Account booking")
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
            .set_client_pairing("http://127.0.0.1:7847", "123456")
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

    #[test]
    fn branch_seed_and_switch() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let branches = store.list_branches().unwrap();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "Main");
        assert!(branches[0].is_this_device);
        assert_eq!(store.active_branch_id().unwrap(), branches[0].id);
        assert_eq!(store.org_name().unwrap(), "My business");
        store.set_org_name("Kumkaj Chai Co").unwrap();
        assert_eq!(store.org_name().unwrap(), "Kumkaj Chai Co");
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
            .set_client_pairing("http://127.0.0.1:7847", "123456")
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
            assert!(receipt.contains("Bill INV-MAIN-000001"), "got {receipt}");
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

        // AR write-off to nil
        let party = store
            .add_customer_kind("Bangles King", "", "Consignor")
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
        let carrier = store.add_customer_kind("Delhivery", "", "Carrier").unwrap();
        let purchase = store
            .record_purchase_ex(
                "Delhivery",
                &carrier.id,
                true,
                &[(tea.id.clone(), 2_000, 5_000)], // 2 kg @ 50
                "",
                &owner,
            )
            .unwrap();
        assert!(purchase.on_account);
        assert_eq!(purchase.total_paise, 10_000);
        assert_eq!(store.party_payable_balance(&carrier.id).unwrap(), 10_000);
        store
            .settle_payable(&carrier.id, 10_000, PayMethod::Cash, &owner, "")
            .unwrap();
        assert_eq!(store.party_payable_balance(&carrier.id).unwrap(), 0);

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
}

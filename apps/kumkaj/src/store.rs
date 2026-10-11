//! Local SQLite persistence for Kumkaj (WAL, restart-safe sales).

use crate::domain::{
    format_qty_display, new_id, AuditEvent, Branch, BranchSnapshot, Category, CreditLedgerEntry,
    Customer, DayTotals, DiningTable, KitchenStation, KotStatus, KotTicket, Money, MoneyLane,
    MoneyMoveKind, MoneyMovement, OpsReport, OrderType, OrgConsolidatedReport, PartyAgingRow,
    PayMethod, PeriodBooks, Product, Purchase, PurchaseLine, Sale, SaleDraft, SaleLine,
    SaleRevision, StaffFeatures, StaffMember, StaffRole, StockMovement, TableStatus, TaxRate,
    VenueMode, STOCK_UNTRACKED,
};
use crate::entity_template::{self, EntityTemplate};
use crate::fiscal::{self, CountryProfile, FiscalSettings, TaxPricing};
use crate::locale::LocaleSettings;
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
    pub fn open_default() -> Result<Self, String> {
        let dir = data_dir()?;
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("kumkaj.sqlite3");
        Self::open_path(&path)
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
        store.rewrite_legacy_standalone()?;
        Ok(store)
    }

    pub fn db_path(&self) -> &Path {
        &self.path
    }

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
                    image_key TEXT
                );
                CREATE TABLE IF NOT EXISTS sales (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    total_paise INTEGER NOT NULL,
                    lines_total_paise INTEGER,
                    rounding_paise INTEGER NOT NULL DEFAULT 0,
                    pay_method TEXT NOT NULL,
                    status TEXT NOT NULL,
                    table_id TEXT,
                    customer_id TEXT
                );
                CREATE TABLE IF NOT EXISTS sale_lines (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    sale_id TEXT NOT NULL REFERENCES sales(id),
                    product_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    unit_price_paise INTEGER NOT NULL,
                    qty INTEGER NOT NULL,
                    line_total_paise INTEGER NOT NULL,
                    note TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS staff (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    role TEXT NOT NULL,
                    pin_hash TEXT NOT NULL,
                    active INTEGER NOT NULL DEFAULT 1
                );
                CREATE TABLE IF NOT EXISTS tables_dining (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    status TEXT NOT NULL,
                    active_sale_id TEXT
                );
                CREATE TABLE IF NOT EXISTS kot_tickets (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    status TEXT NOT NULL,
                    summary TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS audit_events (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    action TEXT NOT NULL,
                    detail TEXT NOT NULL
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
        // v2: optional product image key (built-in menu asset or custom:file)
        let has_image: bool = self
            .conn
            .prepare("PRAGMA table_info(products)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "image_key"))
            })
            .map_err(|e| e.to_string())?;
        if !has_image {
            self.conn
                .execute("ALTER TABLE products ADD COLUMN image_key TEXT", [])
                .map_err(|e| e.to_string())?;
            self.meta_set("schema_version", "2")?;
        }

        // v3: explicit rounding for perfect accounting
        // lines_total + rounding = charged total (sales.total_paise)
        let has_rounding: bool = self
            .conn
            .prepare("PRAGMA table_info(sales)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "rounding_paise"))
            })
            .map_err(|e| e.to_string())?;
        if !has_rounding {
            self.conn
                .execute_batch(
                    r#"
                    ALTER TABLE sales ADD COLUMN lines_total_paise INTEGER;
                    ALTER TABLE sales ADD COLUMN rounding_paise INTEGER NOT NULL DEFAULT 0;
                    UPDATE sales SET lines_total_paise = total_paise WHERE lines_total_paise IS NULL;
                    "#,
                )
                .map_err(|e| e.to_string())?;
            self.meta_set("schema_version", "3")?;
        }

        // v4: append-only sale revision history (owner/manager amend)
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS sale_revisions (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    revised_at TEXT NOT NULL,
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    reason TEXT NOT NULL,
                    previous_json TEXT NOT NULL
                );
                "#,
            )
            .map_err(|e| e.to_string())?;
        let ver = self.meta_get("schema_version")?.unwrap_or_else(|| "1".into());
        if ver.parse::<i64>().unwrap_or(1) < 4 {
            self.meta_set("schema_version", "4")?;
        }

        // v5: optional per-line note on sales
        let has_line_note: bool = self
            .conn
            .prepare("PRAGMA table_info(sale_lines)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "note"))
            })
            .map_err(|e| e.to_string())?;
        if !has_line_note {
            self.conn
                .execute(
                    "ALTER TABLE sale_lines ADD COLUMN note TEXT NOT NULL DEFAULT ''",
                    [],
                )
                .map_err(|e| e.to_string())?;
            self.meta_set("schema_version", "5")?;
        }

        // v6: customers + credit ledger
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS customers (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    phone TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT '',
                    active INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL
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
                    detail TEXT NOT NULL DEFAULT ''
                );
                "#,
            )
            .map_err(|e| e.to_string())?;
        let has_customer_id: bool = self
            .conn
            .prepare("PRAGMA table_info(sales)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "customer_id"))
            })
            .map_err(|e| e.to_string())?;
        if !has_customer_id {
            self.conn
                .execute("ALTER TABLE sales ADD COLUMN customer_id TEXT", [])
                .map_err(|e| e.to_string())?;
            self.meta_set("schema_version", "6")?;
        }

        // v7: Owner vs Staff — role_title + immutable sale authorship
        let has_role_title: bool = self
            .conn
            .prepare("PRAGMA table_info(staff)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "role_title"))
            })
            .map_err(|e| e.to_string())?;
        if !has_role_title {
            self.conn
                .execute(
                    "ALTER TABLE staff ADD COLUMN role_title TEXT NOT NULL DEFAULT ''",
                    [],
                )
                .map_err(|e| e.to_string())?;
            // Backfill titles from role preset
            for (role, title) in [
                ("owner", "Owner"),
                ("manager", "Manager"),
                ("cashier", "Cashier"),
                ("kitchen", "Kitchen"),
            ] {
                self.conn
                    .execute(
                        "UPDATE staff SET role_title = ?1 WHERE role = ?2 AND (role_title = '' OR role_title IS NULL)",
                        params![title, role],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        let has_created_by: bool = self
            .conn
            .prepare("PRAGMA table_info(sales)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "created_by_id"))
            })
            .map_err(|e| e.to_string())?;
        if !has_created_by {
            self.conn
                .execute(
                    "ALTER TABLE sales ADD COLUMN created_by_id TEXT NOT NULL DEFAULT ''",
                    [],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "ALTER TABLE sales ADD COLUMN created_by_name TEXT NOT NULL DEFAULT ''",
                    [],
                )
                .map_err(|e| e.to_string())?;
            self.meta_set("schema_version", "7")?;
        }

        // v8: replace Argon2 PIN hashes (blocked UI for seconds on unlock) with
        // fast salted SHA-256. Existing PINs are reset to 0000 — change under Team.
        let ver = self.meta_get("schema_version")?.unwrap_or_else(|| "1".into());
        if ver.parse::<i64>().unwrap_or(1) < 8 {
            let owner_hash = hash_pin("0000")?;
            // A shop that was already trading had a PIN only its Owner knew.
            // Handing it back as 0000 is an open till, so flag the reset and
            // make the next unlock force a fresh Owner PIN.
            let was_trading = self
                .meta_get("owner_setup_complete")?
                .map(|v| v.trim() == "1")
                .unwrap_or(false);
            // Reset every account to 0000 with the fast hasher.
            self.conn
                .execute("UPDATE staff SET pin_hash = ?1", params![owner_hash])
                .map_err(|e| e.to_string())?;
            if was_trading {
                self.meta_set("pin_reset_required", "1")?;
            }
            self.meta_set("schema_version", "8")?;
        }

        // v9: staff features / login, venue mode, onboarding, client pairing.
        let has_features: bool = self
            .conn
            .prepare("PRAGMA table_info(staff)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "features_json"))
            })
            .map_err(|e| e.to_string())?;
        if !has_features {
            self.conn
                .execute_batch(
                    r#"
                    ALTER TABLE staff ADD COLUMN features_json TEXT NOT NULL DEFAULT '';
                    ALTER TABLE staff ADD COLUMN login_id TEXT NOT NULL DEFAULT '';
                    ALTER TABLE staff ADD COLUMN password_hash TEXT NOT NULL DEFAULT '';
                    "#,
                )
                .map_err(|e| e.to_string())?;
            // Backfill role defaults.
            let staff_rows: Vec<(String, String)> = {
                let mut stmt = self
                    .conn
                    .prepare("SELECT id, role FROM staff")
                    .map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                    .map_err(|e| e.to_string())?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?
            };
            for (id, role_s) in staff_rows {
                let role = StaffRole::parse(&role_s).unwrap_or(StaffRole::Cashier);
                let feats = StaffFeatures::for_role(role);
                self.conn
                    .execute(
                        "UPDATE staff SET features_json = ?1 WHERE id = ?2",
                        params![feats.to_json(), id],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('venue_mode', 'counter')",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('onboarding_complete', '0')",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('client_paired', '0')",
                [],
            )
            .map_err(|e| e.to_string())?;
        let ver = self.meta_get("schema_version")?.unwrap_or_else(|| "1".into());
        if ver.parse::<i64>().unwrap_or(1) < 9 {
            // Existing shops (Owner PIN already set): skip onboarding wizard.
            if self
                .meta_get("owner_setup_complete")?
                .map(|v| v.trim() == "1")
                .unwrap_or(false)
            {
                let _ = self.meta_set("onboarding_complete", "1");
            }
            self.meta_set("schema_version", "9")?;
        }

        // v10: split payments, expenses, optional stock.
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS sale_payments (
                    id TEXT PRIMARY KEY,
                    sale_id TEXT NOT NULL,
                    pay_method TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS expenses (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL,
                    category TEXT NOT NULL DEFAULT 'misc',
                    note TEXT NOT NULL DEFAULT '',
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL
                );
                "#,
            )
            .map_err(|e| e.to_string())?;
        let has_stock: bool = self
            .conn
            .prepare("PRAGMA table_info(products)")
            .and_then(|mut stmt| {
                let names = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(names.iter().any(|n| n == "stock_qty"))
            })
            .map_err(|e| e.to_string())?;
        if !has_stock {
            self.conn
                .execute(
                    "ALTER TABLE products ADD COLUMN stock_qty INTEGER NOT NULL DEFAULT -1",
                    [],
                )
                .map_err(|e| e.to_string())?;
        }
        let ver = self.meta_get("schema_version")?.unwrap_or_else(|| "1".into());
        if ver.parse::<i64>().unwrap_or(1) < 10 {
            self.meta_set("schema_version", "10")?;
        }

        // v11: one org, many branches — the same app now covers a chai stall and
        // a multi-outlet restaurant. Everything stays local: branches roll up by
        // exporting/importing snapshots, and `sync_outbox` is the durable queue
        // a future cloud can drain without changing any of this.
        self.conn
            .execute_batch(
                r#"
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
                    ref_id TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS purchases (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    supplier TEXT NOT NULL DEFAULT '',
                    total_paise INTEGER NOT NULL DEFAULT 0,
                    note TEXT NOT NULL DEFAULT '',
                    actor_id TEXT NOT NULL DEFAULT '',
                    actor_name TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS purchase_lines (
                    id TEXT PRIMARY KEY,
                    purchase_id TEXT NOT NULL,
                    product_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    qty INTEGER NOT NULL,
                    unit_cost_paise INTEGER NOT NULL DEFAULT 0
                );
                CREATE TABLE IF NOT EXISTS sync_outbox (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    payload_json TEXT NOT NULL DEFAULT '',
                    synced_at TEXT NOT NULL DEFAULT ''
                );
                CREATE INDEX IF NOT EXISTS idx_stock_movements_created
                    ON stock_movements(created_at);
                CREATE INDEX IF NOT EXISTS idx_sync_outbox_pending
                    ON sync_outbox(synced_at);
                "#,
            )
            .map_err(|e| e.to_string())?;

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

        for (table, column, decl) in [
            ("products", "category_id", "TEXT NOT NULL DEFAULT ''"),
            ("products", "station_id", "TEXT NOT NULL DEFAULT ''"),
            ("products", "tax_bps", "INTEGER NOT NULL DEFAULT 0"),
            ("products", "modifiers_json", "TEXT NOT NULL DEFAULT '[]'"),
            ("products", "aliases_json", "TEXT NOT NULL DEFAULT '[]'"),
            ("products", "branch_id", "TEXT NOT NULL DEFAULT ''"),
            ("sales", "branch_id", "TEXT NOT NULL DEFAULT ''"),
            ("sales", "order_type", "TEXT NOT NULL DEFAULT 'counter'"),
            ("sales", "tax_paise", "INTEGER NOT NULL DEFAULT 0"),
            ("tables_dining", "branch_id", "TEXT NOT NULL DEFAULT ''"),
            ("kot_tickets", "branch_id", "TEXT NOT NULL DEFAULT ''"),
            ("kot_tickets", "station_id", "TEXT NOT NULL DEFAULT ''"),
            ("expenses", "branch_id", "TEXT NOT NULL DEFAULT ''"),
            ("customers", "branch_id", "TEXT NOT NULL DEFAULT ''"),
        ] {
            self.ensure_column(table, column, decl)?;
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
        // Keep `active_branch_id` pointing at a branch that really exists.
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

        // Legacy single-shop rows all belong to this device's branch.
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
        let ver = self.meta_get("schema_version")?.unwrap_or_else(|| "1".into());
        if ver.parse::<i64>().unwrap_or(1) < 11 {
            self.meta_set("schema_version", "11")?;
        }

        // v12: worldwide fiscal packs. Tax stops being India-shaped — a named
        // rate table per branch, inclusive *or* exclusive pricing frozen onto
        // each sale, and country wording for the tax, pay rail and business id.
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS tax_rates (
                    id TEXT PRIMARY KEY,
                    branch_id TEXT NOT NULL DEFAULT '',
                    name TEXT NOT NULL,
                    code TEXT NOT NULL DEFAULT '',
                    bps INTEGER NOT NULL DEFAULT 0,
                    active INTEGER NOT NULL DEFAULT 1
                );
                CREATE INDEX IF NOT EXISTS idx_tax_rates_branch ON tax_rates(branch_id);
                "#,
            )
            .map_err(|e| e.to_string())?;
        for (table, column, decl) in [
            ("products", "tax_rate_id", "TEXT NOT NULL DEFAULT ''"),
            ("sales", "tax_pricing", "TEXT NOT NULL DEFAULT 'inclusive'"),
            ("sales", "tax_label", "TEXT NOT NULL DEFAULT 'Tax'"),
            ("sales", "net_paise", "INTEGER NOT NULL DEFAULT 0"),
            ("sales", "service_paise", "INTEGER NOT NULL DEFAULT 0"),
        ] {
            self.ensure_column(table, column, decl)?;
        }

        let ver = self.meta_get("schema_version")?.unwrap_or(String::from("1"));
        let upgrading_to_12 = ver.parse::<i64>().unwrap_or(1) < 12;

        // Every shop that predates country packs was selling tax-inclusive, so
        // their taxable base is simply what they charged minus the tax inside.
        if upgrading_to_12 {
            self.conn
                .execute(
                    "UPDATE sales
                     SET net_paise = MAX(total_paise - COALESCE(tax_paise, 0), 0)
                     WHERE COALESCE(net_paise, 0) = 0",
                    [],
                )
                .map_err(|e| e.to_string())?;
        }

        // Country meta defaults to India so existing shops see no change at
        // all; applying another pack from Settings reseeds rates and wording.
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

        if upgrading_to_12 {
            self.meta_set("schema_version", "12")?;
        }

        // v13: human invoice numbers. A truncated UUID is not something a
        // customer can quote back, so every sale now also carries
        // `INV-{branch}-{seq}` drawn from a per-shop counter.
        self.ensure_column("sales", "invoice_no", "TEXT NOT NULL DEFAULT ''")?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('next_invoice_seq', '1')",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('pin_reset_required', '0')",
                [],
            )
            .map_err(|e| e.to_string())?;

        let ver = self.meta_get("schema_version")?.unwrap_or(String::from("1"));
        if ver.parse::<i64>().unwrap_or(1) < 13 {
            self.meta_set("schema_version", "13")?;
        }

        // v14: entity start templates — unit labels, party kind, milli qty,
        // dual money lanes (declared / owner). Same generic tables.
        self.ensure_column("products", "unit_label", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column("customers", "kind_label", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column("sale_lines", "qty_milli", "INTEGER NOT NULL DEFAULT 0")?;
        self.ensure_column("sale_lines", "unit_label", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column("sales", "money_lane", "TEXT NOT NULL DEFAULT 'declared'")?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('entity_template', 'custom')",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE sale_lines SET qty_milli = qty * 1000 WHERE COALESCE(qty_milli, 0) = 0",
                [],
            )
            .map_err(|e| e.to_string())?;
        let ver = self.meta_get("schema_version")?.unwrap_or(String::from("1"));
        if ver.parse::<i64>().unwrap_or(1) < 14 {
            self.meta_set("schema_version", "14")?;
        }

        // v15: cashbook / bank money movements + richer books reports.
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS money_movements (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    amount_paise INTEGER NOT NULL,
                    note TEXT NOT NULL DEFAULT '',
                    actor_id TEXT NOT NULL,
                    actor_name TEXT NOT NULL,
                    branch_id TEXT NOT NULL DEFAULT ''
                );
                CREATE INDEX IF NOT EXISTS idx_money_movements_day
                    ON money_movements(created_at);",
            )
            .map_err(|e| e.to_string())?;
        let ver = self.meta_get("schema_version")?.unwrap_or(String::from("1"));
        if ver.parse::<i64>().unwrap_or(1) < 15 {
            self.meta_set("schema_version", "15")?;
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

    fn table_has_column(&self, table: &str, column: &str) -> Result<bool, String> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .map_err(|e| e.to_string())?;
        let names = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(names.iter().any(|n| n == column))
    }

    fn ensure_column(&self, table: &str, column: &str, decl: &str) -> Result<(), String> {
        if self.table_has_column(table, column)? {
            return Ok(());
        }
        self.conn
            .execute(
                &format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"),
                [],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn ensure_seed_data(&mut self) -> Result<(), String> {
        // Remove placeholder catalogue from earlier prototypes (Item A–E).
        self.conn
            .execute("DELETE FROM products WHERE id LIKE 'seed-item-%'", [])
            .map_err(|e| e.to_string())?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('language', 'en')",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta(key, value) VALUES('currency', 'INR')",
                [],
            )
            .map_err(|e| e.to_string())?;

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

    /// True while an upgrade-time PIN reset is still unanswered. The till must
    /// not open on the fallback 0000 that the v8 migration wrote.
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
        Ok(self
            .meta_get("entity_template")?
            .unwrap_or_else(|| "custom".into()))
    }

    pub fn entity_template(&self) -> Result<&'static EntityTemplate, String> {
        Ok(entity_template::resolve(&self.entity_template_id()?))
    }

    /// Apply a start template: labels/features always; seeds only when empty.
    pub fn apply_entity_template(&self, template_id: &str) -> Result<&'static EntityTemplate, String> {
        self.assert_server_writer("Apply start template")?;
        let tmpl = entity_template::resolve(template_id);
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
        let now = Local::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO credit_ledger(
                    id, customer_id, created_at, kind, amount_paise,
                    sale_id, pay_method, actor_id, actor_name, detail
                 ) VALUES(?1, ?2, ?3, 'opening', ?4, NULL, NULL, ?5, ?6, ?7)",
                params![
                    new_id(),
                    customer_id,
                    now,
                    amount_paise,
                    actor.id,
                    actor.name,
                    "Opening balance from book"
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
                "SELECT id, name, phone, note, active, COALESCE(kind_label, '')
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
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((id, name, phone, note, active, kind_label)) = row else {
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
        self.assert_server_writer("Record the day close")?;
        let summary = self.day_close_summary()?;
        let now = Local::now().to_rfc3339();
        self.meta_set("last_day_close_at", &now)?;
        // Audit detail is one line so the log stays readable.
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
                    "INSERT INTO customers(id, name, phone, note, active, created_at, branch_id, kind_label)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(id) DO UPDATE SET
                       name=excluded.name,
                       phone=excluded.phone,
                       note=excluded.note,
                       active=excluded.active,
                       kind_label=excluded.kind_label",
                    params![
                        c.id,
                        c.name,
                        c.phone,
                        c.note,
                        c.active as i64,
                        Local::now().to_rfc3339(),
                        self.active_branch_id()?,
                        c.kind_label
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
                        sale_id, pay_method, actor_name, detail
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
            self.conn
                .execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, '', ?8, ?9)",
                    params![
                        e.id,
                        e.customer_id,
                        e.created_at,
                        e.kind,
                        e.amount_paise,
                        e.sale_id,
                        e.pay_method,
                        e.actor_name,
                        e.detail
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
        self.conn
            .execute(
                "INSERT INTO expenses(
                    id, created_at, amount_paise, category, note, actor_id, actor_name, branch_id
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    new_id(),
                    Local::now().to_rfc3339(),
                    amount_paise,
                    cat,
                    note.trim(),
                    actor.id,
                    actor.name,
                    self.active_branch_id()?
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
        self.conn
            .execute(
                "INSERT INTO money_movements(
                    id, created_at, kind, amount_paise, note, actor_id, actor_name, branch_id
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    new_id(),
                    Local::now().to_rfc3339(),
                    kind.as_str(),
                    amount_paise,
                    note.trim(),
                    actor.id,
                    actor.name,
                    self.active_branch_id()?
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
                "SELECT id, created_at, kind, amount_paise, note, actor_name
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
            expenses_paise,
            expense_count,
            bank_in_paise: bank_in,
            bank_out_paise: bank_out,
            outstanding_credit_paise: self.outstanding_credit_total()?,
        })
    }

    pub fn period_books_text(&self, from_day: &str, to_day: &str) -> Result<String, String> {
        let books = self.period_books(from_day, to_day)?;
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let tmpl = self.entity_template()?;
        let net = books.revenue_paise - books.expenses_paise;
        Ok(format!(
            "Period books {from_day} → {to_day}\n\
             Sales: {}\n\
             Revenue {}\n\
             {} {}\n\
             {} {}\n\
             Cash {} · {} {} · Card {}\n\
             {} sales {}\n\
             Expenses {} ({})\n\
             Net (rev − expenses) {}\n\
             Bank in {} · Bank out {}\n\
             Bank balance now {}\n\
             Still owed {}\n\
             {}\n",
            books.sale_count,
            Money(books.revenue_paise).format_localized(&currency, &fiscal),
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
            Money(net).format_localized(&currency, &fiscal),
            Money(books.bank_in_paise).format_localized(&currency, &fiscal),
            Money(books.bank_out_paise).format_localized(&currency, &fiscal),
            Money(self.bank_balance_paise()?).format_localized(&currency, &fiscal),
            Money(books.outstanding_credit_paise).format_localized(&currency, &fiscal),
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

    pub fn locale_settings(&self) -> Result<LocaleSettings, String> {
        Ok(LocaleSettings {
            language: self.meta_get("language")?.unwrap_or_else(|| "en".into()),
            currency: self.meta_get("currency")?.unwrap_or_else(|| "INR".into()),
        })
    }

    pub fn set_locale_settings(&self, settings: &LocaleSettings) -> Result<(), String> {
        self.meta_set("language", &settings.language)?;
        self.meta_set("currency", &settings.currency)?;
        Ok(())
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
        let percent: f64 = typed
            .parse()
            .map_err(|_| format!("No tax rate named \"{typed}\" — type a name or a percent"))?;
        if !(0.0..=100.0).contains(&percent) {
            return Err("Tax % must be between 0 and 100".into());
        }
        let bps = (percent * 100.0).round() as i64;
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

    /// `server` (this device is the shop) | `client` (extra till).
    /// A leftover `standalone` value is rewritten to `server` with extra tills off.
    pub fn device_role(&self) -> Result<Option<String>, String> {
        self.rewrite_legacy_standalone()?;
        self.meta_get("device_role")
    }

    pub fn set_device_role(&self, role: &str) -> Result<(), String> {
        match role {
            "server" | "client" => self.meta_set("device_role", role),
            _ => Err("device_role must be server or client".into()),
        }
    }

    /// Rewrite an old Standalone row to shop (`server`) with extra tills off.
    /// Cloud only ever syncs shop books vs extra-till cache — there is no third role.
    fn rewrite_legacy_standalone(&self) -> Result<(), String> {
        if self.meta_get("device_role")?.as_deref() != Some("standalone") {
            return Ok(());
        }
        self.meta_set("device_role", "server")?;
        if self.meta_get("extra_tills_allowed")?.is_none() {
            self.meta_set("extra_tills_allowed", "0")?;
        }
        Ok(())
    }

    /// Extra tills may pair on LAN. A new shop leaves this off. An existing
    /// `server` row with no flag keeps LAN on (shops that already had tills).
    pub fn extra_tills_allowed(&self) -> Result<bool, String> {
        if self.is_client_device()? {
            return Ok(false);
        }
        match self
            .meta_get("extra_tills_allowed")?
            .as_deref()
            .map(str::trim)
        {
            Some("1") => Ok(true),
            Some("0") => Ok(false),
            _ => Ok(self.meta_get("device_role")?.as_deref() == Some("server")),
        }
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
        self.conn
            .execute(
                "INSERT INTO stock_movements(
                    id, branch_id, product_id, created_at, delta_qty,
                    reason, note, actor_id, actor_name, ref_id
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    new_id(),
                    self.active_branch_id()?,
                    product_id,
                    Local::now().to_rfc3339(),
                    delta_qty,
                    reason,
                    note,
                    actor.id,
                    actor.name,
                    ref_id
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Goods received: records the bill, raises stock on tracked items and
    /// leaves one `purchase` movement per line.
    pub fn record_purchase(
        &self,
        supplier: &str,
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
        let branch_id = self.active_branch_id()?;
        let created_at = Local::now().to_rfc3339();
        let purchase_id = new_id();
        let mut built = Vec::with_capacity(lines.len());
        for (product_id, qty, unit_cost_paise) in lines {
            if *qty <= 0 {
                return Err("Purchase quantity must be > 0".into());
            }
            if *unit_cost_paise < 0 {
                return Err("Unit cost cannot be negative".into());
            }
            let product = self
                .get_product(product_id)?
                .ok_or_else(|| format!("Product not found: {product_id}"))?;
            built.push((
                PurchaseLine {
                    id: new_id(),
                    purchase_id: purchase_id.clone(),
                    product_id: product.id.clone(),
                    name: product.name.clone(),
                    qty: *qty,
                    unit_cost_paise: *unit_cost_paise,
                },
                product.tracks_stock(),
            ));
        }
        let total_paise: i64 = built
            .iter()
            .map(|(l, _)| l.line_cost_paise())
            .fold(0i64, |a, b| a.saturating_add(b));
        self.conn
            .execute(
                "INSERT INTO purchases(
                    id, branch_id, created_at, supplier, total_paise, note, actor_id, actor_name
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    purchase_id,
                    branch_id,
                    created_at,
                    supplier.trim(),
                    total_paise,
                    note.trim(),
                    actor.id,
                    actor.name
                ],
            )
            .map_err(|e| e.to_string())?;
        for (line, tracked) in &built {
            self.conn
                .execute(
                    "INSERT INTO purchase_lines(
                        id, purchase_id, product_id, name, qty, unit_cost_paise
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        line.id,
                        line.purchase_id,
                        line.product_id,
                        line.name,
                        line.qty,
                        line.unit_cost_paise
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
                supplier.trim(),
                actor,
                &purchase_id,
            )?;
        }
        self.audit(
            actor,
            "purchase.add",
            &format!("{purchase_id} {} {total_paise}", supplier.trim()),
        )?;
        Ok(Purchase {
            id: purchase_id,
            branch_id,
            created_at,
            supplier: supplier.trim().to_string(),
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
                        m.reason, m.note, m.actor_id, m.actor_name, m.ref_id
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
                        total_paise, note, actor_id, actor_name
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
                "SELECT id, purchase_id, product_id, name, qty, unit_cost_paise
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

    pub fn mark_outbox_synced(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE sync_outbox SET synced_at = ?1 WHERE id = ?2",
                params![Local::now().to_rfc3339(), id],
            )
            .map_err(|e| e.to_string())?;
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
                    tax_rate_id, aliases_json, unit_label
                 )
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
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
                   unit_label=excluded.unit_label",
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
                    product.unit_label
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
        let created_at = Local::now().to_rfc3339();
        let tx = self.conn.unchecked_transaction().map_err(|e| e.to_string())?;

        // Reserved inside the sale's transaction so a crash can never hand the
        // same number to two bills.
        let invoice_no = reserve_invoice_no(&tx, &branch_code)?;
        let created_by_name = actor.lock_label();
        tx.execute(
            "INSERT INTO sales(
                id, created_at, total_paise, lines_total_paise, rounding_paise,
                pay_method, status, table_id, customer_id, created_by_id, created_by_name,
                branch_id, order_type, tax_paise, tax_pricing, tax_label, net_paise,
                service_paise, invoice_no, money_lane
             )
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'completed', ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
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
                draft.money_lane.as_str()
            ],
        )
        .map_err(|e| e.to_string())?;

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
                    reason, note, actor_id, actor_name, ref_id
                 ) VALUES(?1, ?2, ?3, ?4, ?5, 'sale', '', ?6, ?7, ?8)",
                params![
                    new_id(),
                    branch_id,
                    product_id,
                    created_at,
                    -qty,
                    actor.id,
                    actor.name,
                    sale_id
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
                            sale_id, pay_method, actor_id, actor_name, detail
                         ) VALUES(?1, ?2, ?3, 'apply', ?4, ?5, NULL, ?6, ?7, ?8)",
                        params![
                            new_id(),
                            cid,
                            created_at,
                            applied,
                            sale_id,
                            actor.id,
                            actor.name,
                            format!("Applied advance on sale {sale_id}")
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
                if on_credit > 0 {
                    tx.execute(
                        "INSERT INTO credit_ledger(
                            id, customer_id, created_at, kind, amount_paise,
                            sale_id, pay_method, actor_id, actor_name, detail
                         ) VALUES(?1, ?2, ?3, 'sale', ?4, ?5, NULL, ?6, ?7, ?8)",
                        params![
                            new_id(),
                            cid,
                            created_at,
                            on_credit,
                            sale_id,
                            actor.id,
                            actor.name,
                            format!("Credit on sale {sale_id}")
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
        })
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
                "SELECT id, name, phone, note, active, COALESCE(kind_label, '')
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
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let (id, name, phone, note, active, kind_label) = row.map_err(|e| e.to_string())?;
            let balance_paise = self.customer_balance(&id)?;
            out.push(Customer {
                id,
                name,
                phone,
                note,
                active: active != 0,
                balance_paise,
                kind_label,
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
        let now = Local::now().to_rfc3339();
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
                        sale_id, pay_method, actor_id, actor_name, detail
                     ) VALUES(?1, ?2, ?3, 'settle', ?4, NULL, ?5, ?6, ?7, ?8)",
                    params![
                        new_id(),
                        customer_id,
                        now,
                        -settle_part,
                        pay.as_str(),
                        actor.id,
                        actor.name,
                        format!("Settlement {detail}")
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        if advance_part > 0 {
            self.conn
                .execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail
                     ) VALUES(?1, ?2, ?3, 'advance', ?4, NULL, ?5, ?6, ?7, ?8)",
                    params![
                        new_id(),
                        customer_id,
                        now,
                        -advance_part,
                        pay.as_str(),
                        actor.id,
                        actor.name,
                        format!("Advance {detail}")
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

    pub fn list_credit_ledger(
        &self,
        customer_id: &str,
        limit: i64,
    ) -> Result<Vec<CreditLedgerEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_name, detail
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

    pub fn restore_from(backup: &Path) -> Result<Self, String> {
        Self::restore_from_to_path(backup, &data_dir()?.join("kumkaj.sqlite3"))
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
        Self::open_path(&base.join("kumkaj.sqlite3"))
    }
}

/// Reverse-DNS app id from `env/development.env` or `env/production.env`
/// (baked by `build.rs`). Dev: `com.kumkaj.app.debug`. Prod: `com.kumkaj.app`.
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

/// Every column [`row_to_product`] expects, in order. COALESCE keeps rows
/// written before schema v11 readable.
const PRODUCT_COLUMNS: &str = "id, name, price_paise, available, archived, image_key,
     COALESCE(category_id, ''), COALESCE(station_id, ''), COALESCE(tax_bps, 0),
     COALESCE(stock_qty, -1), COALESCE(modifiers_json, '[]'), COALESCE(branch_id, ''),
     COALESCE(tax_rate_id, ''), COALESCE(aliases_json, '[]'), COALESCE(unit_label, '')";

/// Every column [`row_to_sale`] expects, in order.
const SALE_COLUMNS: &str = "id, created_at, total_paise,
     COALESCE(lines_total_paise, total_paise),
     COALESCE(rounding_paise, 0),
     pay_method, status, table_id, customer_id,
     COALESCE(created_by_id, ''), COALESCE(created_by_name, ''),
     COALESCE(branch_id, ''), COALESCE(order_type, 'counter'), COALESCE(tax_paise, 0),
     COALESCE(tax_pricing, 'inclusive'), COALESCE(tax_label, 'Tax'), COALESCE(net_paise, 0),
     COALESCE(service_paise, 0), COALESCE(invoice_no, ''),
     COALESCE(money_lane, 'declared')";

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
    fn custom_template_leaves_empty_seeds() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store.apply_entity_template("custom").unwrap();
        assert!(store.list_products(false).unwrap().is_empty());
        assert!(store.list_customers().unwrap().is_empty());
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
        assert!(store.extra_tills_allowed().unwrap());
        store.set_extra_tills_allowed(false).unwrap();
        assert!(!store.extra_tills_allowed().unwrap());
        store.set_extra_tills_allowed(true).unwrap();
        assert!(store.extra_tills_allowed().unwrap());
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

    /// Upgrading a single-shop DB must keep every existing row visible: the
    /// v11 backfill adopts them into the seeded Main branch.
    #[test]
    fn legacy_rows_adopt_main_branch_on_upgrade() {
        let path = temp_db();
        let sale_id = {
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
            store.add_expense(500, "milk", "", &owner).unwrap();
            sale.id
        };
        // Rewind this DB to look like schema v10.
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "DROP TABLE branches;
                 DROP TABLE categories;
                 DROP TABLE kitchen_stations;
                 DROP TABLE stock_movements;
                 DROP TABLE purchases;
                 DROP TABLE purchase_lines;
                 DROP TABLE sync_outbox;
                 DROP TABLE tax_rates;
                 ALTER TABLE products DROP COLUMN branch_id;
                 ALTER TABLE products DROP COLUMN tax_rate_id;
                 ALTER TABLE sales DROP COLUMN branch_id;
                 ALTER TABLE sales DROP COLUMN order_type;
                 ALTER TABLE sales DROP COLUMN tax_paise;
                 ALTER TABLE sales DROP COLUMN tax_pricing;
                 ALTER TABLE sales DROP COLUMN tax_label;
                 ALTER TABLE sales DROP COLUMN net_paise;
                 ALTER TABLE sales DROP COLUMN service_paise;
                 ALTER TABLE tables_dining DROP COLUMN branch_id;
                 ALTER TABLE expenses DROP COLUMN branch_id;
                 ALTER TABLE customers DROP COLUMN branch_id;
                 DELETE FROM meta WHERE key IN ('org_id', 'active_branch_id');
                 UPDATE meta SET value = '10' WHERE key = 'schema_version';",
            )
            .unwrap();
        }

        let store = Store::open_path(&path).unwrap();
        assert_eq!(store.meta_get("schema_version").unwrap().unwrap(), "15");
        // The India pack is the v12 default, so an upgraded shop keeps selling
        // exactly as it did before country packs existed.
        assert_eq!(store.fiscal_settings().unwrap().country_code, "IN");
        assert_eq!(store.list_tax_rates().unwrap().len(), 5);
        let branch = store.active_branch().unwrap();
        assert_eq!(branch.name, "Main");
        assert!(branch.is_this_device);
        let products = store.list_products(false).unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].branch_id, branch.id);
        assert_eq!(products[0].stock_qty, STOCK_UNTRACKED);
        let sale = store.get_sale(&sale_id).unwrap().unwrap();
        assert_eq!(sale.branch_id, branch.id);
        assert_eq!(sale.order_type(), OrderType::Counter);
        assert_eq!(sale.tax_paise, 0);
        assert_eq!(store.today_totals().unwrap().sale_count, 1);
        assert_eq!(store.today_expenses_total().unwrap(), (1, 500));
        assert_eq!(store.list_tables().unwrap().len(), 6);
        assert_eq!(store.list_stations().unwrap().len(), 2);
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
                &[(samosa.id.clone(), 5, 800)],
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
        store.mark_outbox_synced(&pending[0].0).unwrap();
        assert!(store.list_pending_outbox(10).unwrap().is_empty());
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
    fn legacy_inclusive_sales_backfill_net_on_upgrade() {
        let path = temp_db();
        {
            let mut store = Store::open_path(&path).unwrap();
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
            // Pretend this row predates schema v12.
            store
                .conn
                .execute("UPDATE sales SET net_paise = 0, tax_pricing = ''", [])
                .unwrap();
            store.meta_set("schema_version", "11").unwrap();
        }
        let store = Store::open_path(&path).unwrap();
        assert_eq!(store.meta_get("schema_version").unwrap().unwrap(), "15");
        let sale = store.recent_sales(1).unwrap().remove(0);
        assert_eq!(sale.total_paise, 10_500);
        assert_eq!(sale.tax_paise, 500);
        assert_eq!(sale.net_paise, 10_000, "net backfilled as total - tax");
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
                .record_purchase("Supplier", &[(tea.id.clone(), 2, 500)], "", &owner)
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

    /// The v8 upgrade handed every account back as 0000. A shop that was
    /// already trading must be forced to choose a new Owner PIN.
    #[test]
    fn migration_pin_reset_forces_a_new_owner_pin() {
        let path = temp_db();
        {
            let store = Store::open_path(&path).unwrap();
            // A fresh shop has its own first-run setup, so no reset is pending.
            assert!(!store.pin_reset_required().unwrap());
            store.complete_owner_setup("4321").unwrap();
            // Pretend this DB predates the PIN hasher swap.
            store.meta_set("schema_version", "7").unwrap();
        }

        let store = Store::open_path(&path).unwrap();
        assert!(
            store.pin_reset_required().unwrap(),
            "a trading shop reset to 0000 must demand a new PIN"
        );
        // The fallback PIN does open the account — which is exactly why the
        // flag has to gate the till.
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
}

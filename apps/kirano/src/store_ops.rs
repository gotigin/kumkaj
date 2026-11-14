//! Local / LAN ops beyond the day book: waitlist, loyalty, vouchers, clock,
//! stock count, POs, transfers, feedback, CRM segments, and optional Net stubs.
//!
//! Child of `store` — shares the connection. Counter and kitchen never wait
//! on these; outbox / Net stubs stay optional.

use super::Store;
use crate::domain::{
    birthday_in_range, new_id, Customer, Money, Product, StaffMember,
};
use crate::notify;
use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Local guest-book slice for CRM messages (Parties → Guests & messages).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrmSegment {
    /// Birthday falls in the next 7 days (incl. today).
    BirthdayThisWeek,
    /// Birthday month-day in the current calendar month.
    BirthdayThisMonth,
    /// Active customers with a phone number.
    WithPhone,
    /// Guests who hold any loyalty points.
    WithLoyalty,
    /// No completed sale in the last `days` (default 30).
    InactiveDays(i64),
    /// Highest paid-sale totals, limited to `n` (default 20).
    TopSpenders(usize),
}

impl CrmSegment {
    pub fn label(self) -> &'static str {
        match self {
            Self::BirthdayThisWeek => "Birthday this week",
            Self::BirthdayThisMonth => "Birthday this month",
            Self::WithPhone => "With phone",
            Self::WithLoyalty => "With loyalty points",
            Self::InactiveDays(_) => "Inactive 30+ days",
            Self::TopSpenders(_) => "Top spenders",
        }
    }
}

pub const OPS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS waitlist (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    party_size INTEGER NOT NULL DEFAULT 1,
    quoted_mins INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'waiting',
    table_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    seated_at TEXT NOT NULL DEFAULT '',
    cancelled_at TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS waiter_calls (
    id TEXT PRIMARY KEY,
    table_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    cleared_at TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS stock_counts (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    closed_at TEXT NOT NULL DEFAULT '',
    closed_by TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS stock_count_lines (
    id TEXT PRIMARY KEY,
    count_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    expected_qty INTEGER NOT NULL DEFAULT 0,
    counted_qty INTEGER NOT NULL,
    reason TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS gift_vouchers (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    amount_paise INTEGER NOT NULL,
    balance_paise INTEGER NOT NULL,
    issued_at TEXT NOT NULL,
    redeemed_at TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS time_clock (
    id TEXT PRIMARY KEY,
    staff_id TEXT NOT NULL,
    staff_name TEXT NOT NULL DEFAULT '',
    clock_in TEXT NOT NULL,
    clock_out TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS purchase_orders (
    id TEXT PRIMARY KEY,
    supplier_id TEXT NOT NULL DEFAULT '',
    supplier_name TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL,
    received_at TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS purchase_order_lines (
    id TEXT PRIMARY KEY,
    po_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    qty INTEGER NOT NULL,
    unit_cost_paise INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS feedback (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    score INTEGER NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    table_name TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS floor_areas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    picker_enabled INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS connector_outbox (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'queued',
    note TEXT NOT NULL DEFAULT ''
);
"#;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GiftVoucher {
    pub id: String,
    pub code: String,
    pub amount_paise: i64,
    pub balance_paise: i64,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: String,
    pub created_at: String,
    pub note: String,
    pub lines: Vec<PurchaseOrderLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseOrderLine {
    pub id: String,
    pub product_id: String,
    pub name: String,
    pub qty: i64,
    pub unit_cost_paise: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StockTransferFile {
    pub from_branch: String,
    pub to_branch: String,
    pub at: String,
    pub lines: Vec<StockTransferLine>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StockTransferLine {
    pub product_name: String,
    pub product_id: String,
    pub qty: i64,
}

impl Store {
    pub(crate) fn migrate_ops(&self) -> Result<(), String> {
        self.conn
            .execute_batch(OPS_SCHEMA)
            .map_err(|e| e.to_string())?;
        for stmt in [
            "ALTER TABLE running_lines ADD COLUMN seat_no INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE running_lines ADD COLUMN held INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE running_lines ADD COLUMN course_no INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE running_lines ADD COLUMN line_total_paise INTEGER NOT NULL DEFAULT -1",
            "ALTER TABLE products ADD COLUMN allergens TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE customers ADD COLUMN loyalty_points INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE customers ADD COLUMN wallet_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE customers ADD COLUMN birthday TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE tables_dining ADD COLUMN waiter_staff_id TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE tables_dining ADD COLUMN layout_x REAL NOT NULL DEFAULT 0",
            "ALTER TABLE tables_dining ADD COLUMN layout_y REAL NOT NULL DEFAULT 0",
            "ALTER TABLE tables_dining ADD COLUMN layout_w REAL NOT NULL DEFAULT 0",
            "ALTER TABLE tables_dining ADD COLUMN layout_h REAL NOT NULL DEFAULT 0",
            "ALTER TABLE kitchen_stations ADD COLUMN picker_enabled INTEGER NOT NULL DEFAULT 1",
            "ALTER TABLE tax_rates ADD COLUMN picker_enabled INTEGER NOT NULL DEFAULT 1",
            "ALTER TABLE purchases ADD COLUMN batch_code TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE purchases ADD COLUMN expires_on TEXT NOT NULL DEFAULT ''",
        ] {
            let _ = self.conn.execute(stmt, []);
        }
        Ok(())
    }

    // —— Waitlist ——

    // —— Server sections / waiter call / layout ——

    // —— Coursing / seats ——

    // —— Stock count variance + tip pool ——

    pub fn start_stock_count(&self, actor: &StaffMember) -> Result<String, String> {
        self.assert_server_writer("Stock count")?;
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO stock_counts(id, started_at) VALUES(?1, ?2)",
                params![id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "stock.count.start", &id)?;
        Ok(id)
    }

    pub fn record_stock_count_line(
        &self,
        count_id: &str,
        product: &Product,
        counted_qty: i64,
        reason: &str,
    ) -> Result<(), String> {
        self.assert_server_writer("Stock count")?;
        let expected = if product.stock_qty < 0 {
            0
        } else {
            product.stock_qty
        };
        self.conn
            .execute(
                "INSERT INTO stock_count_lines(
                    id, count_id, product_id, name, expected_qty, counted_qty, reason
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    new_id(),
                    count_id,
                    product.id,
                    product.name,
                    expected,
                    counted_qty,
                    reason.trim()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn close_stock_count(&self, count_id: &str, actor: &StaffMember) -> Result<String, String> {
        self.assert_server_writer("Close stock count")?;
        self.conn
            .execute(
                "UPDATE stock_counts SET closed_at = ?2, closed_by = ?3 WHERE id = ?1",
                params![count_id, Local::now().to_rfc3339(), actor.name],
            )
            .map_err(|e| e.to_string())?;
        let report = self.stock_count_variance_text(count_id)?;
        self.audit(actor, "stock.count.close", count_id)?;
        Ok(report)
    }

    pub fn stock_count_variance_text(&self, count_id: &str) -> Result<String, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, expected_qty, counted_qty, reason FROM stock_count_lines
                  WHERE count_id = ?1 ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![count_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = String::from("Stock count variance\n");
        for row in rows {
            let (name, exp, counted, reason) = row.map_err(|e| e.to_string())?;
            let var = counted - exp;
            out.push_str(&format!(
                "- {name}: expected {exp} · counted {counted} · var {var:+}{}\n",
                if reason.is_empty() {
                    String::new()
                } else {
                    format!(" · {reason}")
                }
            ));
        }
        Ok(out)
    }

    pub fn tip_pooling_report_text(&self) -> Result<String, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COALESCE(created_by_name, '—'), COALESCE(SUM(tip_paise), 0)
                   FROM sales
                  WHERE status = 'completed' AND tip_paise > 0
                    AND created_at LIKE ?1 || '%'
                  GROUP BY COALESCE(created_by_name, '—')
                  ORDER BY 2 DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![today], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let mut out = format!("Tip pool · {today}\n");
        let mut total = 0i64;
        for row in rows {
            let (who, tip) = row.map_err(|e| e.to_string())?;
            total += tip;
            out.push_str(&format!(
                "- {who}: {}\n",
                Money(tip).format_localized(&currency, &fiscal)
            ));
        }
        out.push_str(&format!(
            "Total tips {}\n",
            Money(total).format_localized(&currency, &fiscal)
        ));
        Ok(out)
    }

    // —— Bulk price ——

    pub fn bulk_price_category(
        &self,
        category_id: &str,
        percent: Option<i64>,
        set_paise: Option<i64>,
        actor: &StaffMember,
    ) -> Result<usize, String> {
        self.assert_server_writer("Bulk price")?;
        let mut products = self.list_products_in_category(category_id)?;
        if products.is_empty() {
            return Err("No items in that category".into());
        }
        for p in &mut products {
            if let Some(set) = set_paise {
                if set < 0 {
                    return Err("Price cannot be negative".into());
                }
                p.price = Money(set);
            } else if let Some(pct) = percent {
                let base = p.price.paise();
                let next = base + base.saturating_mul(pct) / 100;
                if next < 0 {
                    return Err("Price cannot be negative".into());
                }
                p.price = Money(next);
            } else {
                return Err("Give a percent or a set amount".into());
            }
            self.upsert_product(p)?;
        }
        self.audit(
            actor,
            "menu.bulk_price",
            &format!("{} item(s)", products.len()),
        )?;
        Ok(products.len())
    }

    // —— Pick-lists (areas / stations / tax) ——

    pub fn set_tax_rate_picker_enabled(
        &self,
        rate_id: &str,
        enabled: bool,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Tax picker")?;
        self.conn
            .execute(
                "UPDATE tax_rates SET picker_enabled = ?1 WHERE id = ?2",
                params![enabled as i64, rate_id],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "tax.picker", rate_id)?;
        Ok(())
    }

    pub fn list_picker_tax_rates(&self) -> Result<Vec<crate::domain::TaxRate>, String> {
        Ok(self
            .list_tax_rates()?
            .into_iter()
            .filter(|r| r.picker_enabled)
            .collect())
    }

    // —— Loyalty + wallet ——

    pub fn loyalty_points(&self, customer_id: &str) -> Result<i64, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(loyalty_points, 0) FROM customers WHERE id = ?1",
                params![customer_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    pub fn wallet_paise(&self, customer_id: &str) -> Result<i64, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(wallet_paise, 0) FROM customers WHERE id = ?1",
                params![customer_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    /// Earn 1 point per ₹10 of sale (local rule).
    pub fn earn_loyalty_on_sale(
        &self,
        customer_id: &str,
        sale_paise: i64,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Loyalty")?;
        let pts = (sale_paise / 1000).max(0);
        if pts == 0 {
            return self.loyalty_points(customer_id);
        }
        self.conn
            .execute(
                "UPDATE customers SET loyalty_points = COALESCE(loyalty_points, 0) + ?2 WHERE id = ?1",
                params![customer_id, pts],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "loyalty.earn", &format!("{customer_id}+{pts}"))?;
        self.loyalty_points(customer_id)
    }

    /// Redeem points: 1 point = ₹1. Returns paise redeemed.
    pub fn redeem_loyalty(
        &self,
        customer_id: &str,
        points: i64,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Redeem loyalty")?;
        let points = points.max(0);
        let have = self.loyalty_points(customer_id)?;
        if points > have {
            return Err(format!("Only {have} points available"));
        }
        self.conn
            .execute(
                "UPDATE customers SET loyalty_points = loyalty_points - ?2 WHERE id = ?1",
                params![customer_id, points],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "loyalty.redeem", &format!("{customer_id}-{points}"))?;
        Ok(points * 100)
    }

    pub fn wallet_top_up(
        &self,
        customer_id: &str,
        paise: i64,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Wallet top-up")?;
        if !actor.is_owner() {
            return Err("Not allowed to top up wallets".into());
        }
        if paise <= 0 {
            return Err("Top-up must be positive".into());
        }
        self.conn
            .execute(
                "UPDATE customers SET wallet_paise = COALESCE(wallet_paise, 0) + ?2 WHERE id = ?1",
                params![customer_id, paise],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "wallet.topup", &format!("{customer_id} {paise}"))?;
        self.wallet_paise(customer_id)
    }

    pub fn wallet_spend(
        &self,
        customer_id: &str,
        paise: i64,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Wallet spend")?;
        if paise <= 0 {
            return Err("Spend must be positive".into());
        }
        let have = self.wallet_paise(customer_id)?;
        if paise > have {
            return Err(format!(
                "Wallet has only {}",
                Money(have).format_localized(
                    &self.locale_settings()?.currency,
                    &self.fiscal_settings()?
                )
            ));
        }
        self.conn
            .execute(
                "UPDATE customers SET wallet_paise = wallet_paise - ?2 WHERE id = ?1",
                params![customer_id, paise],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "wallet.spend", customer_id)?;
        self.wallet_paise(customer_id)
    }

    // —— Gift vouchers ——

    pub fn issue_voucher(
        &self,
        code: &str,
        amount_paise: i64,
        actor: &StaffMember,
    ) -> Result<GiftVoucher, String> {
        self.assert_server_writer("Issue voucher")?;
        if !actor.is_owner() {
            return Err("Not allowed to issue vouchers".into());
        }
        let code = code.trim().to_uppercase();
        if code.is_empty() {
            return Err("Voucher code required".into());
        }
        if amount_paise <= 0 {
            return Err("Voucher amount must be positive".into());
        }
        let v = GiftVoucher {
            id: new_id(),
            code: code.clone(),
            amount_paise,
            balance_paise: amount_paise,
            active: true,
        };
        self.conn
            .execute(
                "INSERT INTO gift_vouchers(id, code, amount_paise, balance_paise, issued_at, active)
                 VALUES(?1, ?2, ?3, ?3, ?4, 1)",
                params![v.id, v.code, amount_paise, Local::now().to_rfc3339()],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE") {
                    "That voucher code already exists".into()
                } else {
                    e.to_string()
                }
            })?;
        self.audit(actor, "voucher.issue", &format!("{code} {amount_paise}"))?;
        Ok(v)
    }

    pub fn voucher_balance(&self, code: &str) -> Result<i64, String> {
        let code = code.trim().to_uppercase();
        self.conn
            .query_row(
                "SELECT balance_paise FROM gift_vouchers
                  WHERE code = ?1 AND active = 1",
                params![code],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Voucher not found".to_string())
    }

    pub fn redeem_voucher(
        &self,
        code: &str,
        amount_paise: i64,
        actor: &StaffMember,
    ) -> Result<i64, String> {
        self.assert_server_writer("Redeem voucher")?;
        let code = code.trim().to_uppercase();
        let (id, bal): (String, i64) = self
            .conn
            .query_row(
                "SELECT id, balance_paise FROM gift_vouchers
                  WHERE code = ?1 AND active = 1",
                params![code],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Voucher not found".to_string())?;
        let take = amount_paise.max(0).min(bal);
        if take <= 0 {
            return Err("Nothing left on this voucher".into());
        }
        let left = bal - take;
        self.conn
            .execute(
                "UPDATE gift_vouchers SET balance_paise = ?2,
                    redeemed_at = CASE WHEN ?2 = 0 THEN ?3 ELSE redeemed_at END,
                    active = CASE WHEN ?2 = 0 THEN 0 ELSE 1 END
                  WHERE id = ?1",
                params![id, left, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "voucher.redeem", &code)?;
        Ok(take)
    }

    // —— Time clock ——

    pub fn clock_in(&self, staff_id: &str, pin: &str) -> Result<String, String> {
        self.assert_server_writer("Clock in")?;
        let staff = self
            .authenticate_lan_pin(Some(staff_id), pin)?
            .ok_or_else(|| "Wrong PIN".to_string())?;
        let open: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM time_clock WHERE staff_id = ?1 AND clock_out = '' LIMIT 1",
                params![staff.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if open.is_some() {
            return Err(format!("{} is already clocked in", staff.name));
        }
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO time_clock(id, staff_id, staff_name, clock_in)
                 VALUES(?1, ?2, ?3, ?4)",
                params![id, staff.id, staff.name, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn clock_out(&self, staff_id: &str, pin: &str) -> Result<(), String> {
        self.assert_server_writer("Clock out")?;
        let staff = self
            .authenticate_lan_pin(Some(staff_id), pin)?
            .ok_or_else(|| "Wrong PIN".to_string())?;
        let n = self
            .conn
            .execute(
                "UPDATE time_clock SET clock_out = ?2
                  WHERE staff_id = ?1 AND clock_out = ''",
                params![staff.id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err(format!("{} is not clocked in", staff.name));
        }
        Ok(())
    }

    pub fn time_clock_hours_text(&self, day: &str) -> Result<String, String> {
        let day = if day.trim().is_empty() {
            Local::now().format("%Y-%m-%d").to_string()
        } else {
            day.trim().to_string()
        };
        let mut stmt = self
            .conn
            .prepare(
                "SELECT staff_name, clock_in, clock_out FROM time_clock
                  WHERE clock_in LIKE ?1 || '%' ORDER BY clock_in",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![day], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = format!("Time clock · {day}\n");
        for row in rows {
            let (name, cin, cout) = row.map_err(|e| e.to_string())?;
            let mins = clock_minutes(&cin, &cout);
            out.push_str(&format!(
                "- {name}: {} → {} · {mins} min\n",
                &cin[..cin.len().min(16)].replace('T', " "),
                if cout.is_empty() {
                    "open".into()
                } else {
                    cout[..cout.len().min(16)].replace('T', " ")
                }
            ));
        }
        Ok(out)
    }

    // —— Product allergens ——

    pub fn set_product_allergens(
        &self,
        product_id: &str,
        allergens: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Allergens")?;
        let tags = allergens
            .split(|c: char| c == ',' || c == ';' || c == '/')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        self.conn
            .execute(
                "UPDATE products SET allergens = ?2 WHERE id = ?1",
                params![product_id, tags],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "menu.allergens", product_id)?;
        Ok(())
    }

    pub fn product_allergens(&self, product_id: &str) -> Result<String, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(allergens, '') FROM products WHERE id = ?1",
                params![product_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
            .map(|o| o.unwrap_or_default())
    }

    // —— Expiry / batch ——

    pub fn set_purchase_batch_expiry(
        &self,
        purchase_id: &str,
        batch_code: &str,
        expires_on: &str,
    ) -> Result<(), String> {
        self.assert_server_writer("Purchase batch")?;
        self.conn
            .execute(
                "UPDATE purchases SET batch_code = ?2, expires_on = ?3 WHERE id = ?1",
                params![purchase_id, batch_code.trim(), expires_on.trim()],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn expiry_alert_text(&self) -> Result<String, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COALESCE(supplier, ''), COALESCE(batch_code, ''), expires_on, total_paise
                   FROM purchases
                  WHERE expires_on != '' AND expires_on <= date(?1, '+7 days')
                  ORDER BY expires_on",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![today], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = String::from("Expiry / batch alerts (next 7 days)\n");
        let mut n = 0;
        for row in rows {
            let (sup, batch, exp, total) = row.map_err(|e| e.to_string())?;
            n += 1;
            out.push_str(&format!(
                "- {exp} · {sup} · batch {batch} · {total} paise\n"
            ));
        }
        if n == 0 {
            out.push_str("None due.\n");
        }
        Ok(out)
    }

    /// Soft near-expiry hint for one item (optional batch dates from Buy).
    pub fn product_near_expiry_hint(&self, product_id: &str) -> Result<Option<String>, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let row: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT p.expires_on, COALESCE(p.batch_code, '')
                   FROM purchases p
                   JOIN purchase_lines pl ON pl.purchase_id = p.id
                  WHERE pl.product_id = ?1
                    AND p.expires_on != ''
                    AND p.expires_on <= date(?2, '+90 days')
                  ORDER BY p.expires_on
                  LIMIT 1",
                params![product_id, today],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(row.map(|(exp, batch)| {
            if batch.trim().is_empty() {
                format!("near expiry {exp}")
            } else {
                format!("near expiry {exp} · batch {batch}")
            }
        }))
    }

    // —— Purchase orders ——

    pub fn create_purchase_order(
        &self,
        supplier_name: &str,
        lines: &[(String, String, i64, i64)],
        note: &str,
        actor: &StaffMember,
    ) -> Result<PurchaseOrder, String> {
        self.assert_server_writer("Purchase order")?;
        if lines.is_empty() {
            return Err("Add at least one line".into());
        }
        let id = new_id();
        let at = Local::now().to_rfc3339();
        let supplier = self.find_or_create_customer(supplier_name.trim()).ok();
        let supplier_id = supplier.as_ref().map(|c| c.id.clone()).unwrap_or_default();
        self.conn
            .execute(
                "INSERT INTO purchase_orders(
                    id, supplier_id, supplier_name, status, created_at, note, created_by
                 ) VALUES(?1, ?2, ?3, 'open', ?4, ?5, ?6)",
                params![
                    id,
                    supplier_id,
                    supplier_name.trim(),
                    at,
                    note.trim(),
                    actor.name
                ],
            )
            .map_err(|e| e.to_string())?;
        let mut po_lines = Vec::new();
        for (pid, name, qty, cost) in lines {
            let lid = new_id();
            self.conn
                .execute(
                    "INSERT INTO purchase_order_lines(
                        id, po_id, product_id, name, qty, unit_cost_paise
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                    params![lid, id, pid, name, *qty, *cost],
                )
                .map_err(|e| e.to_string())?;
            po_lines.push(PurchaseOrderLine {
                id: lid,
                product_id: pid.clone(),
                name: name.clone(),
                qty: *qty,
                unit_cost_paise: *cost,
            });
        }
        self.audit(actor, "po.create", &id)?;
        Ok(PurchaseOrder {
            id,
            supplier_id,
            supplier_name: supplier_name.trim().into(),
            status: "open".into(),
            created_at: at,
            note: note.trim().into(),
            lines: po_lines,
        })
    }

    pub fn receive_purchase_order(
        &self,
        po_id: &str,
        actor: &StaffMember,
    ) -> Result<String, String> {
        self.assert_server_writer("Receive PO")?;
        let (status, supplier): (String, String) = self
            .conn
            .query_row(
                "SELECT status, supplier_name FROM purchase_orders WHERE id = ?1",
                params![po_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| "Purchase order not found".to_string())?;
        if status != "open" {
            return Err("PO already received or cancelled".into());
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT product_id, name, qty, unit_cost_paise FROM purchase_order_lines
                  WHERE po_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let lines: Vec<(String, String, i64, i64)> = stmt
            .query_map(params![po_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let purchase_lines: Vec<(String, i64, i64)> = lines
            .iter()
            .map(|(pid, _name, qty, cost)| (pid.clone(), qty * 1000, *cost))
            .collect();
        let purchase = self.record_purchase_ex(
            &supplier,
            "",
            false,
            &purchase_lines,
            &format!("PO {po_id}"),
            actor,
        )?;
        self.conn
            .execute(
                "UPDATE purchase_orders SET status = 'received', received_at = ?2 WHERE id = ?1",
                params![po_id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "po.receive", po_id)?;
        Ok(purchase.id)
    }

    pub fn supplier_price_history_text(&self, supplier_name: &str) -> Result<String, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.created_at, pl.name, pl.unit_cost_paise, pl.qty
                   FROM purchases p
                   JOIN purchase_lines pl ON pl.purchase_id = p.id
                  WHERE lower(p.supplier) = lower(?1)
                  ORDER BY p.created_at DESC LIMIT 40",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![supplier_name.trim()], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = format!("Supplier price history · {}\n", supplier_name.trim());
        for row in rows {
            let (at, name, cost, qty) = row.map_err(|e| e.to_string())?;
            out.push_str(&format!(
                "- {} · {name} ×{qty} @ {cost} paise\n",
                &at[..at.len().min(10)]
            ));
        }
        Ok(out)
    }

    // —— Stock transfer file ——

    pub fn export_stock_transfer(
        &self,
        to_branch: &str,
        lines: &[(String, i64)],
        actor: &StaffMember,
    ) -> Result<String, String> {
        self.assert_server_writer("Stock transfer")?;
        let mut file = StockTransferFile {
            from_branch: self.active_branch_id()?,
            to_branch: to_branch.trim().into(),
            at: Local::now().to_rfc3339(),
            lines: Vec::new(),
        };
        for (pid, qty) in lines {
            let p = self
                .get_product(pid)?
                .ok_or_else(|| format!("Unknown product {pid}"))?;
            if *qty <= 0 {
                continue;
            }
            self.adjust_stock(pid, -qty, "transfer_out", to_branch, actor)?;
            file.lines.push(StockTransferLine {
                product_name: p.name,
                product_id: pid.clone(),
                qty: *qty,
            });
        }
        if file.lines.is_empty() {
            return Err("No lines to transfer".into());
        }
        serde_json::to_string_pretty(&file).map_err(|e| e.to_string())
    }

    pub fn import_stock_transfer(
        &self,
        json: &str,
        actor: &StaffMember,
    ) -> Result<usize, String> {
        self.assert_server_writer("Import transfer")?;
        let file: StockTransferFile =
            serde_json::from_str(json).map_err(|e| format!("Bad transfer file: {e}"))?;
        let mut n = 0;
        for line in &file.lines {
            let product = if !line.product_id.is_empty() {
                self.get_product(&line.product_id)?
            } else {
                self.list_products(false)?
                    .into_iter()
                    .find(|p| p.name.eq_ignore_ascii_case(&line.product_name))
            };
            let Some(p) = product else {
                continue;
            };
            self.adjust_stock(&p.id, line.qty, "transfer_in", &file.from_branch, actor)?;
            n += 1;
        }
        Ok(n)
    }

    // —— Feedback ——

    pub fn add_feedback(
        &self,
        score: i64,
        note: &str,
        table_name: &str,
    ) -> Result<(), String> {
        let score = score.clamp(1, 5);
        self.conn
            .execute(
                "INSERT INTO feedback(id, created_at, score, note, table_name)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                params![
                    new_id(),
                    Local::now().to_rfc3339(),
                    score,
                    note.trim(),
                    table_name.trim()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn feedback_report_text(&self) -> Result<String, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT score, note, table_name, created_at FROM feedback
                  ORDER BY created_at DESC LIMIT 30",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = String::from("Guest feedback\n");
        for row in rows {
            let (score, note, table, at) = row.map_err(|e| e.to_string())?;
            out.push_str(&format!(
                "- {score}/5 · {table} · {} · {note}\n",
                &at[..at.len().min(16)].replace('T', " ")
            ));
        }
        Ok(out)
    }

    // —— Optional connectors ——

    pub fn enqueue_connector(
        &self,
        kind: &str,
        payload: serde_json::Value,
        note: &str,
    ) -> Result<String, String> {
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO connector_outbox(id, created_at, kind, payload_json, status, note)
                 VALUES(?1, ?2, ?3, ?4, 'queued', ?5)",
                params![
                    id,
                    Local::now().to_rfc3339(),
                    kind.trim(),
                    payload.to_string(),
                    note.trim()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn enqueue_card_terminal(
        &self,
        amount_paise: i64,
        note: &str,
    ) -> Result<String, String> {
        self.enqueue_connector(
            "card_terminal",
            serde_json::json!({ "amount_paise": amount_paise }),
            note,
        )
    }

    pub fn enqueue_payment_link(
        &self,
        amount_paise: i64,
        note: &str,
    ) -> Result<String, String> {
        self.enqueue_connector(
            "payment_link",
            serde_json::json!({ "amount_paise": amount_paise }),
            note,
        )
    }

    /// Offline aggregator file drop (Zomato/Swiggy-style JSON). Never blocks counter.
    pub fn import_aggregator_orders_file(&self, json: &str) -> Result<usize, String> {
        self.assert_server_writer("Import aggregator orders")?;
        let val: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Bad aggregator file: {e}"))?;
        let orders = val
            .as_array()
            .cloned()
            .or_else(|| val.get("orders").and_then(|o| o.as_array()).cloned())
            .unwrap_or_else(|| vec![val]);
        let mut n = 0;
        for o in orders {
            self.enqueue_connector("aggregator_inbox", o, "file drop")?;
            n += 1;
        }
        Ok(n)
    }

    /// Read `backups/import_aggregator_orders.json` and enqueue each order.
    /// Same drop-file pattern as branch day import — Local only; never Net.
    pub fn import_aggregator_orders_drop_file(&self) -> Result<usize, String> {
        let path = self
            .path
            .parent()
            .unwrap_or(Path::new("."))
            .join("backups")
            .join("import_aggregator_orders.json");
        if !path.exists() {
            return Err("Missing backups/import_aggregator_orders.json".into());
        }
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        self.import_aggregator_orders_file(&json)
    }

    /// Net poll stub — disabled by default; only enqueues a wake note.
    pub fn aggregator_net_poll_stub(&self, enabled: bool) -> Result<(), String> {
        if !enabled {
            return Ok(());
        }
        let _ = self.enqueue_connector(
            "aggregator_poll",
            serde_json::json!({ "enabled": true, "note": "optional when internet available" }),
            "stub",
        )?;
        Ok(())
    }

    // —— LAN display helpers ——

    pub fn display_order_html(&self) -> Result<String, String> {
        let title = self.venue_title().unwrap_or_default();
        let title = if title.trim().is_empty() { "Guest display".to_string() } else { title };
        // The shop's static payment QR (Settings → Bill format) and stamp.
        let brand = self.print_brand_kit_html().unwrap_or_default();
        Ok(format!(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>{title}</title>
<style>body{{font-family:sans-serif;padding:1.5rem;background:#111;color:#f5f5f5}}
h1{{font-size:1.6rem}}img{{background:#fff;padding:12px;border-radius:8px;max-width:240px}}
.brand-stamp img,.brand-pay-qr img{{max-width:180px}}
</style></head><body>
<h1>{title}</h1>
{brand}
<p style="opacity:.6;margin-top:2rem">LAN guest display · open this Server URL on a second device</p>
</body></html>"#,
            title = html_esc(&title),
        ))
    }

    /// Floor “Local ops” body — owner-facing only. No LAN paths, aggregator
    /// file paths, or connector jargon (those live in docs / ℹ tips).
    pub fn ops_panel_text(&self) -> Result<String, String> {
        let mut parts: Vec<String> = Vec::new();
        parts.push(self.tip_pooling_report_text()?);
        parts.push(self.expiry_alert_text()?);

        let fb = self.feedback_report_text()?;
        if fb.lines().skip(1).any(|l| !l.trim().is_empty()) {
            parts.push(fb);
        }
        let tc = self.time_clock_hours_text("")?;
        if tc.lines().skip(1).any(|l| !l.trim().is_empty()) {
            parts.push(tc);
        }

        let bday_week = self.crm_segment(CrmSegment::BirthdayThisWeek)?.len();
        if bday_week > 0 {
            parts.push(format!("Birthdays this week: {bday_week}"));
        }

        Ok(parts.join("\n\n"))
    }

    // —— CRM segments + campaign queue ——

    /// Guests matching a local CRM segment (customers only; suppliers skipped).
    pub fn crm_segment(&self, segment: CrmSegment) -> Result<Vec<Customer>, String> {
        let all = self.list_customers()?;
        // Guests: Customer role, or unlabeled parties (find_or_create leaves kind
        // empty). Pure suppliers stay out of birthday / loyalty blasts.
        let guests: Vec<Customer> = all
            .into_iter()
            .filter(|c| c.is_customer() || (!c.is_supplier() && c.kind_label.trim().is_empty()))
            .collect();
        let today = Local::now().date_naive();
        Ok(match segment {
            CrmSegment::BirthdayThisWeek => {
                let end = today + Duration::days(6);
                guests
                    .into_iter()
                    .filter(|c| birthday_in_range(&c.birthday, today, end))
                    .collect()
            }
            CrmSegment::BirthdayThisMonth => {
                let m = today.month();
                let last = NaiveDate::from_ymd_opt(today.year(), m, 1)
                    .and_then(|d| {
                        if m == 12 {
                            NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
                                .and_then(|n| n.pred_opt())
                        } else {
                            NaiveDate::from_ymd_opt(today.year(), m + 1, 1)
                                .and_then(|n| n.pred_opt())
                        }
                        .or(Some(d))
                    })
                    .unwrap_or(today);
                let first = NaiveDate::from_ymd_opt(today.year(), m, 1).unwrap_or(today);
                guests
                    .into_iter()
                    .filter(|c| birthday_in_range(&c.birthday, first, last))
                    .collect()
            }
            CrmSegment::WithPhone => guests
                .into_iter()
                .filter(|c| !c.phone.trim().is_empty())
                .collect(),
            CrmSegment::WithLoyalty => guests
                .into_iter()
                .filter(|c| c.loyalty_points > 0)
                .collect(),
            CrmSegment::InactiveDays(days) => {
                let days = days.max(1);
                let cutoff = (Local::now() - Duration::days(days)).to_rfc3339();
                let mut out = Vec::new();
                for c in guests {
                    let last: Option<String> = self
                        .conn
                        .query_row(
                            "SELECT MAX(created_at) FROM sales
                              WHERE customer_id = ?1 AND status = 'completed'",
                            params![c.id],
                            |r| r.get(0),
                        )
                        .map_err(|e| e.to_string())?;
                    let inactive = match last {
                        None => true,
                        Some(at) => at.as_str() < cutoff.as_str(),
                    };
                    if inactive {
                        out.push(c);
                    }
                }
                out
            }
            CrmSegment::TopSpenders(n) => {
                let n = n.max(1);
                let mut scored: Vec<(Customer, i64)> = Vec::new();
                for c in guests {
                    let spent: i64 = self
                        .conn
                        .query_row(
                            "SELECT COALESCE(SUM(total_paise), 0) FROM sales
                              WHERE customer_id = ?1 AND status = 'completed'",
                            params![c.id],
                            |r| r.get(0),
                        )
                        .unwrap_or(0);
                    if spent > 0 {
                        scored.push((c, spent));
                    }
                }
                scored.sort_by(|a, b| b.1.cmp(&a.1));
                scored.into_iter().take(n).map(|(c, _)| c).collect()
            }
        })
    }

    /// Preview lines for the CRM card (name · phone · points).
    pub fn crm_segment_preview(&self, segment: CrmSegment) -> Result<String, String> {
        let rows = self.crm_segment(segment)?;
        if rows.is_empty() {
            return Ok(format!("{}: none\n", segment.label()));
        }
        let mut out = format!("{}: {} guest(s)\n", segment.label(), rows.len());
        for c in rows.iter().take(40) {
            let phone = if c.phone.trim().is_empty() {
                "—"
            } else {
                c.phone.trim()
            };
            out.push_str(&format!(
                "· {} · {} · {} pts · bday {}\n",
                c.name,
                phone,
                c.loyalty_points,
                if c.birthday.is_empty() {
                    "—"
                } else {
                    c.birthday.as_str()
                }
            ));
        }
        if rows.len() > 40 {
            out.push_str(&format!("… and {} more\n", rows.len() - 40));
        }
        Ok(out)
    }

    /// Queue SMS or WhatsApp intents for every guest in the segment who has a phone.
    /// Body may use `{name}` and `{points}`. Owner-triggered — never auto-blasts.
    pub fn queue_segment_campaign(
        &self,
        segment: CrmSegment,
        body: &str,
        channel: &str,
        actor: &StaffMember,
    ) -> Result<usize, String> {
        self.assert_server_writer("CRM campaign")?;
        if !actor.is_owner() && !actor.can_access_sales() {
            return Err("Not allowed to run CRM campaigns".into());
        }
        let body = body.trim();
        if body.is_empty() {
            return Err("Type a message first".into());
        }
        let channel = channel.trim().to_ascii_lowercase();
        if channel != "sms" && channel != "whatsapp" {
            return Err("Channel must be sms or whatsapp".into());
        }
        let guests = self.crm_segment(segment)?;
        let mut n = 0usize;
        let template = match segment {
            CrmSegment::BirthdayThisWeek | CrmSegment::BirthdayThisMonth => "crm_birthday",
            CrmSegment::WithLoyalty => "crm_loyalty",
            _ => "crm_campaign",
        };
        for c in guests {
            let phone = c.phone.trim();
            if phone.is_empty() {
                continue;
            }
            let msg = body
                .replace("{name}", &c.name)
                .replace("{points}", &c.loyalty_points.to_string());
            let row = if channel == "sms" {
                notify::queue_sms_message(phone, &msg, template)
            } else {
                notify::queue_whatsapp_message(phone, &msg, template)
            };
            self.queue_notify(&row)?;
            n += 1;
        }
        self.audit(
            actor,
            "crm.campaign",
            &format!("{} {channel} ×{n}", segment.label()),
        )?;
        Ok(n)
    }
}

fn clock_minutes(cin: &str, cout: &str) -> i64 {
    let parse = |s: &str| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|d| d.timestamp())
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(&s.replace('T', " ")[..s.len().min(19)], "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|n| n.and_utc().timestamp())
            })
    };
    let Some(a) = parse(cin) else {
        return 0;
    };
    let b = if cout.is_empty() {
        Local::now().timestamp()
    } else {
        parse(cout).unwrap_or(a)
    };
    ((b - a) / 60).max(0)
}

fn html_esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn base64_encode(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let mut n = (chunk[0] as u32) << 16;
        if chunk.len() > 1 {
            n |= (chunk[1] as u32) << 8;
        }
        if chunk.len() > 2 {
            n |= chunk[2] as u32;
        }
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// WhatsApp share URL for a bill (opens device browser / app; Net optional).
pub fn whatsapp_bill_url(bill_text: &str) -> String {
    // Print marks (size, weight, align per line) are not part of the message.
    let bill_text = crate::print_share::strip_kot_style_marks(bill_text);
    let mut enc = String::new();
    for b in bill_text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                enc.push(b as char)
            }
            b' ' => enc.push_str("%20"),
            _ => enc.push_str(&format!("%{b:02X}")),
        }
    }
    format!("https://wa.me/?text={enc}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Product, StaffRole};

    struct Shop {
        store: Store,
        path: std::path::PathBuf,
        owner: StaffMember,
    }
    impl Drop for Shop {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
    fn shop() -> Shop {
        let path = std::env::temp_dir().join(format!("kirano-ops-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        Shop { store, path, owner }
    }

    #[test]
    fn loyalty_redeem_and_wallet() {
        let shop = shop();
        let c = shop.store.find_or_create_customer("Loyalty Guest").unwrap();
        shop.store
            .earn_loyalty_on_sale(&c.id, 25_000, &shop.owner)
            .unwrap();
        assert_eq!(shop.store.loyalty_points(&c.id).unwrap(), 25);
        let paise = shop.store.redeem_loyalty(&c.id, 10, &shop.owner).unwrap();
        assert_eq!(paise, 1000);
        assert_eq!(shop.store.loyalty_points(&c.id).unwrap(), 15);
        shop.store.wallet_top_up(&c.id, 5000, &shop.owner).unwrap();
        shop.store.wallet_spend(&c.id, 2000, &shop.owner).unwrap();
        assert_eq!(shop.store.wallet_paise(&c.id).unwrap(), 3000);
    }

    #[test]
    fn crm_birthday_segment_and_campaign_queue() {
        use crate::domain::birthday_in_range;
        use chrono::{Duration, Local};
        let shop = shop();
        let today = Local::now().date_naive();
        let md = format!("{:02}-{:02}", today.month(), today.day());
        let c = shop.store.find_or_create_customer("Bday Guest").unwrap();
        shop.store
            .update_customer_details(&c.id, "Bday Guest", "9998887777", Some(&md))
            .unwrap();
        assert!(birthday_in_range(&md, today, today + Duration::days(6)));
        let week = shop
            .store
            .crm_segment(CrmSegment::BirthdayThisWeek)
            .unwrap();
        assert!(week.iter().any(|g| g.id == c.id), "{week:?}");
        let n = shop
            .store
            .queue_segment_campaign(
                CrmSegment::BirthdayThisWeek,
                "Happy birthday {name}! Points: {points}",
                "sms",
                &shop.owner,
            )
            .unwrap();
        assert!(n >= 1);
        let outbox = shop.store.list_notify_outbox(20).unwrap();
        assert!(outbox.iter().any(|r| r.channel == "sms" && r.to_addr == "9998887777"));
        let phone_seg = shop.store.crm_segment(CrmSegment::WithPhone).unwrap();
        assert!(phone_seg.iter().any(|g| g.id == c.id));
    }

    #[test]
    fn po_receive_and_time_clock_hours() {
        let shop = shop();
        let rice = Product::new_raw_material("Rice", "kg");
        shop.store.upsert_product(&rice).unwrap();
        let po = shop
            .store
            .create_purchase_order(
                "Grain Co",
                &[(rice.id.clone(), rice.name.clone(), 10, 4000)],
                "",
                &shop.owner,
            )
            .unwrap();
        let bill = shop.store.receive_purchase_order(&po.id, &shop.owner).unwrap();
        assert!(!bill.is_empty());
        shop.store
            .add_staff("Wasim", StaffRole::Cashier, "Captain", "2222", &shop.owner)
            .unwrap();
        let waiter = shop
            .store
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|s| s.name == "Wasim")
            .unwrap();
        shop.store.clock_in(&waiter.id, "2222").unwrap();
        let text = shop.store.time_clock_hours_text("").unwrap();
        assert!(text.contains("Wasim"), "{text}");
    }

    #[test]
    fn whatsapp_url_encodes_bill() {
        let u = whatsapp_bill_url("Bill ₹100\nThanks");
        assert!(u.starts_with("https://wa.me/?text="));
        assert!(u.contains("Bill"));
    }

    #[test]
    fn lan_public_html_pages_ok() {
        let shop = shop();
        shop.store.set_venue_title("Corner Kirana").unwrap();
        let display = shop.store.display_order_html().unwrap();
        assert!(display.contains("<!doctype"), "{display}");
        assert!(display.contains("Corner Kirana"), "the shop's name heads the guest display");
        assert!(!display.contains("table"), "no table wording on a counter display: {display}");
        // The owner's panel says what happened in words; LAN paths and
        // connector jargon live in the docs and the ℹ tip, never here.
        let ops = shop.store.ops_panel_text().unwrap();
        assert!(!ops.contains("/display") && !ops.contains("Waitlist"), "{ops}");
    }
}

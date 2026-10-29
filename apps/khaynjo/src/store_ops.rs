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
pub struct WaitlistEntry {
    pub id: String,
    pub name: String,
    pub party_size: i64,
    pub quoted_mins: i64,
    pub status: String,
    pub table_id: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaiterCall {
    pub id: String,
    pub table_id: String,
    pub created_at: String,
    pub note: String,
}

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

    pub fn add_waitlist(
        &self,
        name: &str,
        party_size: i64,
        quoted_mins: i64,
        actor: &StaffMember,
    ) -> Result<WaitlistEntry, String> {
        self.assert_server_writer("Waitlist")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Guest name required".into());
        }
        let entry = WaitlistEntry {
            id: new_id(),
            name: name.into(),
            party_size: party_size.max(1),
            quoted_mins: quoted_mins.max(0),
            status: "waiting".into(),
            table_id: String::new(),
            created_at: Local::now().to_rfc3339(),
        };
        self.conn
            .execute(
                "INSERT INTO waitlist(id, name, party_size, quoted_mins, status, created_at, created_by)
                 VALUES(?1, ?2, ?3, ?4, 'waiting', ?5, ?6)",
                params![
                    entry.id,
                    entry.name,
                    entry.party_size,
                    entry.quoted_mins,
                    entry.created_at,
                    actor.name
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(entry)
    }

    pub fn list_waitlist(&self) -> Result<Vec<WaitlistEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, party_size, quoted_mins, status, table_id, created_at
                   FROM waitlist WHERE status = 'waiting' ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(WaitlistEntry {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    party_size: r.get(2)?,
                    quoted_mins: r.get(3)?,
                    status: r.get(4)?,
                    table_id: r.get(5)?,
                    created_at: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn seat_waitlist(
        &self,
        wait_id: &str,
        table_id: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Seat waitlist")?;
        let n = self
            .conn
            .execute(
                "UPDATE waitlist SET status = 'seated', table_id = ?2, seated_at = ?3
                  WHERE id = ?1 AND status = 'waiting'",
                params![wait_id, table_id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Waitlist entry not found or already seated".into());
        }
        let _ = self.occupy_table(table_id);
        self.audit(actor, "waitlist.seat", wait_id)?;
        Ok(())
    }

    pub fn cancel_waitlist(&self, wait_id: &str, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Cancel waitlist")?;
        let n = self
            .conn
            .execute(
                "UPDATE waitlist SET status = 'cancelled', cancelled_at = ?2
                  WHERE id = ?1 AND status = 'waiting'",
                params![wait_id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Waitlist entry not found".into());
        }
        self.audit(actor, "waitlist.cancel", wait_id)?;
        Ok(())
    }

    // —— Server sections / waiter call / layout ——

    pub fn set_table_waiter(
        &self,
        table_id: &str,
        staff_id: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Assign waiter")?;
        self.conn
            .execute(
                "UPDATE tables_dining SET waiter_staff_id = ?2 WHERE id = ?1",
                params![table_id, staff_id.trim()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "table.waiter", table_id)?;
        Ok(())
    }

    pub fn table_waiter_id(&self, table_id: &str) -> Result<String, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(waiter_staff_id, '') FROM tables_dining WHERE id = ?1",
                params![table_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    pub fn set_table_layout(
        &self,
        table_id: &str,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Floor layout")?;
        self.conn
            .execute(
                "UPDATE tables_dining
                    SET layout_x = ?2, layout_y = ?3, layout_w = ?4, layout_h = ?5
                  WHERE id = ?1",
                params![table_id, x, y, w.max(0.0), h.max(0.0)],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "table.layout", table_id)?;
        Ok(())
    }

    pub fn table_layout(&self, table_id: &str) -> Result<(f64, f64, f64, f64), String> {
        self.conn
            .query_row(
                "SELECT COALESCE(layout_x,0), COALESCE(layout_y,0),
                        COALESCE(layout_w,0), COALESCE(layout_h,0)
                   FROM tables_dining WHERE id = ?1",
                params![table_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(|e| e.to_string())
    }

    /// Floor display preference: `"grid"` (default) or `"layout"` (drawn).
    pub fn floor_view(&self) -> Result<String, String> {
        Ok(match self.meta_get("floor_view")?.as_deref() {
            Some("layout") => "layout".into(),
            _ => "grid".into(),
        })
    }

    pub fn set_floor_view(&self, view: &str) -> Result<(), String> {
        let v = if view.trim().eq_ignore_ascii_case("layout") {
            "layout"
        } else {
            "grid"
        };
        self.meta_set("floor_view", v)
    }

    /// When every table still has a zero layout box, pack into a 4-column
    /// canvas of 120×90 tiles and persist so the next open is stable.
    /// Returns how many tables were assigned.
    pub fn ensure_table_layouts_packed(&self, actor: &StaffMember) -> Result<usize, String> {
        let tables = self.list_tables()?;
        if tables.is_empty() {
            return Ok(0);
        }
        let all_zero = tables.iter().all(|t| {
            t.layout_x == 0.0 && t.layout_y == 0.0 && t.layout_w == 0.0 && t.layout_h == 0.0
        });
        if !all_zero {
            return Ok(0);
        }
        const COLS: usize = 4;
        const TW: f64 = 120.0;
        const TH: f64 = 90.0;
        const GAP: f64 = 10.0;
        for (i, t) in tables.iter().enumerate() {
            let col = i % COLS;
            let row = i / COLS;
            let x = col as f64 * (TW + GAP);
            let y = row as f64 * (TH + GAP);
            self.set_table_layout(&t.id, x, y, TW, TH, actor)?;
        }
        Ok(tables.len())
    }

    pub fn queue_waiter_call(
        &self,
        table_id: &str,
        note: &str,
        actor: &StaffMember,
    ) -> Result<WaiterCall, String> {
        self.assert_server_writer("Waiter call")?;
        let call = WaiterCall {
            id: new_id(),
            table_id: table_id.into(),
            created_at: Local::now().to_rfc3339(),
            note: note.trim().into(),
        };
        self.conn
            .execute(
                "INSERT INTO waiter_calls(id, table_id, created_at, note)
                 VALUES(?1, ?2, ?3, ?4)",
                params![call.id, call.table_id, call.created_at, call.note],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "waiter.call", table_id)?;
        Ok(call)
    }

    pub fn list_open_waiter_calls(&self) -> Result<Vec<WaiterCall>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, table_id, created_at, note FROM waiter_calls
                  WHERE cleared_at = '' ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(WaiterCall {
                    id: r.get(0)?,
                    table_id: r.get(1)?,
                    created_at: r.get(2)?,
                    note: r.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn clear_waiter_call(&self, call_id: &str, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Clear waiter call")?;
        self.conn
            .execute(
                "UPDATE waiter_calls SET cleared_at = ?2 WHERE id = ?1 AND cleared_at = ''",
                params![call_id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "waiter.clear", call_id)?;
        Ok(())
    }

    // —— Coursing / seats ——

    pub fn set_line_held(&self, line_id: &str, held: bool, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Hold course")?;
        let n = self
            .conn
            .execute(
                "UPDATE running_lines SET held = ?2 WHERE id = ?1 AND kot_id = '' AND cancelled = 0",
                params![line_id, held as i64],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Only unsent lines can be held".into());
        }
        self.audit(actor, if held { "line.hold" } else { "line.unhold" }, line_id)?;
        Ok(())
    }

    pub fn set_line_seat(&self, line_id: &str, seat_no: i64, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Assign seat")?;
        self.conn
            .execute(
                "UPDATE running_lines SET seat_no = ?2 WHERE id = ?1",
                params![line_id, seat_no.max(0)],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "line.seat", &format!("{line_id}:{seat_no}"))?;
        Ok(())
    }

    pub fn set_line_course(
        &self,
        line_id: &str,
        course_no: i64,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Set course")?;
        self.conn
            .execute(
                "UPDATE running_lines SET course_no = ?2 WHERE id = ?1 AND kot_id = ''",
                params![line_id, course_no.max(0)],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "line.course", line_id)?;
        Ok(())
    }

    /// Unhold pending lines of `course_no` and send only those to the kitchen.
    pub fn fire_course(
        &self,
        table_id: &str,
        course_no: i64,
        actor: &StaffMember,
    ) -> Result<Vec<String>, String> {
        self.assert_server_writer("Fire course")?;
        let order = self
            .running_order(table_id)?
            .ok_or_else(|| "No open order on that table".to_string())?;
        self.conn
            .execute(
                "UPDATE running_lines SET held = 0
                  WHERE order_id = ?1 AND kot_id = '' AND cancelled = 0 AND course_no = ?2",
                params![order.id, course_no],
            )
            .map_err(|e| e.to_string())?;
        self.send_kot_filtered(table_id, actor, Some(course_no), false)
    }

    /// Split open bill evenly across `n` companion bills (item round-robin).
    pub fn split_evenly(
        &self,
        table_id: &str,
        n: i64,
        actor: &StaffMember,
    ) -> Result<Vec<crate::domain::DiningTable>, String> {
        let n = n.max(2);
        let order = self
            .running_order(table_id)?
            .ok_or_else(|| "No open bill".to_string())?;
        let ids: Vec<String> = order.live_lines().map(|l| l.id.clone()).collect();
        if ids.len() < n as usize {
            return Err("Need at least as many items as guests to split evenly".into());
        }
        let mut companions = Vec::new();
        for i in 1..n {
            let chunk: Vec<String> = ids
                .iter()
                .enumerate()
                .filter(|(idx, _)| (*idx as i64) % n == i)
                .map(|(_, id)| id.clone())
                .collect();
            if chunk.is_empty() {
                continue;
            }
            companions.push(self.split_running_lines(&chunk, actor)?);
        }
        Ok(companions)
    }

    /// Split by seat number: each seat > 0 with lines gets a companion bill.
    pub fn split_by_seat(
        &self,
        table_id: &str,
        actor: &StaffMember,
    ) -> Result<Vec<crate::domain::DiningTable>, String> {
        let order = self
            .running_order(table_id)?
            .ok_or_else(|| "No open bill".to_string())?;
        let mut seats: Vec<i64> = order
            .live_lines()
            .map(|l| l.seat_no)
            .filter(|s| *s > 0)
            .collect();
        seats.sort_unstable();
        seats.dedup();
        if seats.len() < 2 {
            return Err("Assign seat numbers on at least two groups of items first".into());
        }
        let keep = seats[0];
        let mut companions = Vec::new();
        for seat in seats.into_iter().skip(1) {
            let chunk: Vec<String> = order
                .live_lines()
                .filter(|l| l.seat_no == seat)
                .map(|l| l.id.clone())
                .collect();
            if chunk.is_empty() {
                continue;
            }
            let _ = keep;
            companions.push(self.split_running_lines(&chunk, actor)?);
        }
        Ok(companions)
    }

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

    pub fn ensure_floor_area(&self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(());
        }
        let exists: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM floor_areas WHERE lower(name) = lower(?1) LIMIT 1",
                params![name],
                |_| Ok(true),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or(false);
        if exists {
            // Make sure an existing area is offered in pickers again.
            self.conn
                .execute(
                    "UPDATE floor_areas SET picker_enabled = 1 WHERE lower(name) = lower(?1)",
                    params![name],
                )
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        self.conn
            .execute(
                "INSERT INTO floor_areas(id, name, picker_enabled) VALUES(?1, ?2, 1)",
                params![new_id(), name],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_area_picker_enabled(
        &self,
        name: &str,
        enabled: bool,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Area picker")?;
        self.ensure_floor_area(name)?;
        self.conn
            .execute(
                "UPDATE floor_areas SET picker_enabled = ?1 WHERE lower(name) = lower(?2)",
                params![enabled as i64, name.trim()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "area.picker", name)?;
        Ok(())
    }

    pub fn list_picker_areas(&self) -> Result<Vec<(String, bool)>, String> {
        for a in self.list_areas()? {
            self.ensure_floor_area(&a)?;
        }
        let mut stmt = self
            .conn
            .prepare("SELECT name, picker_enabled FROM floor_areas ORDER BY name")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? != 0)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn set_station_picker_enabled(
        &self,
        station_id: &str,
        enabled: bool,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Station picker")?;
        self.conn
            .execute(
                "UPDATE kitchen_stations SET picker_enabled = ?1 WHERE id = ?2",
                params![enabled as i64, station_id],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "station.picker", station_id)?;
        Ok(())
    }

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

    pub fn list_picker_stations(&self) -> Result<Vec<crate::domain::KitchenStation>, String> {
        Ok(self
            .list_stations()?
            .into_iter()
            .filter(|s| s.picker_enabled)
            .collect())
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
        if paise <= 0 {
            return Err("Top-up must be positive".into());
        }
        self.conn
            .execute(
                "UPDATE customers SET wallet_paise = COALESCE(wallet_paise, 0) + ?2 WHERE id = ?1",
                params![customer_id, paise],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "wallet.topup", customer_id)?;
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
        self.audit(actor, "voucher.issue", &code)?;
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

    pub fn display_order_html(&self, table_id: Option<&str>) -> Result<String, String> {
        let (title, lines, total, upi) = if let Some(tid) = table_id.filter(|t| !t.is_empty()) {
            let order = self.running_order(tid)?;
            let label = self.table_label(tid)?;
            match order {
                Some(o) => {
                    let ls: Vec<String> = o
                        .live_lines()
                        .map(|l| {
                            let qty = crate::domain::format_qty_display(
                                l.qty_milli,
                                0,
                                &l.unit_label,
                            );
                            format!(
                                "<li>{qty}× {}{}</li>",
                                html_esc(&l.name),
                                if l.held { " <em>(held)</em>" } else { "" }
                            )
                        })
                        .collect();
                    let total = o.total_paise();
                    (label, ls, total, total)
                }
                None => (label, vec!["<li>No open order</li>".into()], 0, 0),
            }
        } else {
            (
                "Guest display".into(),
                vec!["<li>Open /display?table=NAME</li>".into()],
                0,
                0,
            )
        };
        // Static payment QR (Settings → Bill format) wins for the face; else
        // bake the bill amount into a dynamic UPI QR when VPA is set.
        let brand = self.print_brand_kit_html().unwrap_or_default();
        let mut qr = String::new();
        if self.payment_qr_path().ok().flatten().is_none() && upi > 0 {
            if let Ok(vpa) = self.upi_vpa() {
                if !vpa.trim().is_empty() {
                    if let Ok(uri) = crate::upi_qr::upi_pay_uri(&vpa, &self.upi_payee().unwrap_or_default(), upi)
                    {
                        if let Ok(png) = crate::upi_qr::upi_qr_png(&uri, 6) {
                            qr = format!(
                                "<p>Pay UPI</p><img alt=\"UPI QR\" src=\"data:image/png;base64,{}\"/>",
                                base64_encode(&png)
                            );
                        }
                    }
                }
            }
        }
        let currency = self.locale_settings()?.currency;
        let fiscal = self.fiscal_settings()?;
        let amount = Money(total).format_localized(&currency, &fiscal);
        Ok(format!(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>{title}</title>
<style>body{{font-family:sans-serif;padding:1.5rem;background:#111;color:#f5f5f5}}
h1{{font-size:1.6rem}}ul{{font-size:1.25rem;line-height:1.6}}
.total{{font-size:2rem;margin-top:1rem}}img{{background:#fff;padding:12px;border-radius:8px;max-width:240px}}
.brand-stamp img,.brand-pay-qr img{{max-width:180px}}
</style></head><body>
<h1>{title}</h1>
<ul>{}</ul>
<p class="total">Total {amount}</p>
{brand}{qr}
<p style="opacity:.6;margin-top:2rem">LAN guest display · open this Server URL on a second device</p>
</body></html>"#,
            lines.join("")
        ))
    }

    pub fn tokens_ready_html(&self) -> Result<String, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT kot_no, summary, created_at FROM kot_tickets
                  WHERE status = 'ready' AND kind = 'kot'
                    AND (table_id = '' OR table_id IN (
                        SELECT id FROM tables_dining WHERE lower(area) LIKE '%parcel%'
                           OR lower(name) LIKE '%take%' OR lower(name) LIKE '%parcel%'
                    ))
                  ORDER BY created_at DESC LIMIT 40",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut items = String::new();
        for row in rows {
            let (no, summary, _) = row.map_err(|e| e.to_string())?;
            let token = if no > 0 {
                no.to_string()
            } else {
                summary.chars().take(12).collect()
            };
            items.push_str(&format!(
                "<div class=\"tok\">{}</div>",
                html_esc(&token)
            ));
        }
        if items.is_empty() {
            items = "<p>No ready tokens</p>".into();
        }
        Ok(format!(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta http-equiv="refresh" content="5"/>
<title>Ready</title>
<style>body{{font-family:sans-serif;background:#0a0a0a;color:#fff;text-align:center;padding:2rem}}
h1{{font-size:2rem}}.tok{{display:inline-block;margin:.5rem;padding:1rem 1.5rem;font-size:2.5rem;
background:#1a7f37;border-radius:12px;min-width:4rem}}</style></head>
<body><h1>Ready for pickup</h1>{items}</body></html>"#
        ))
    }

    pub fn expo_ready_html(&self) -> Result<String, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COALESCE(t.name, ''), k.summary, COALESCE(s.name, ''), k.created_at
                   FROM kot_tickets k
                   LEFT JOIN tables_dining t ON t.id = k.table_id
                   LEFT JOIN kitchen_stations s ON s.id = k.station_id
                  WHERE k.status = 'ready' AND k.kind = 'kot'
                  ORDER BY k.created_at LIMIT 60",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut items = String::new();
        for row in rows {
            let (table, summary, station, _) = row.map_err(|e| e.to_string())?;
            items.push_str(&format!(
                "<li><strong>{}</strong> · {} · {}</li>",
                html_esc(&table),
                html_esc(&station),
                html_esc(&summary)
            ));
        }
        if items.is_empty() {
            items = "<li>Nothing ready on the pass</li>".into();
        }
        Ok(format!(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta http-equiv="refresh" content="4"/>
<title>Expo</title>
<style>body{{font-family:sans-serif;padding:1.5rem;background:#111;color:#eee}}
li{{font-size:1.3rem;margin:.4rem 0}}</style></head>
<body><h1>Pass / Expo — Ready</h1><ul>{items}</ul></body></html>"#
        ))}

    pub fn menu_public_json(&self) -> Result<String, String> {
        let items: Vec<_> = self
            .list_products(false)?
            .into_iter()
            .filter(|p| p.available && !p.is_raw() && p.kind != crate::domain::ITEM_KIND_RAW)
            .map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "name": p.name,
                    "price_paise": p.price.paise(),
                    "category_id": p.category_id,
                    "allergens": p.allergens,
                })
            })
            .collect();
        Ok(serde_json::json!({ "items": items }).to_string())
    }

    pub fn kiosk_html(&self) -> Result<String, String> {
        let menu = self.menu_public_json()?;
        Ok(format!(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>Kiosk</title>
<style>body{{font-family:sans-serif;padding:1rem}}button{{margin:.25rem;padding:.6rem 1rem;font-size:1rem}}
#cart{{margin-top:1rem;border-top:1px solid #ccc;padding-top:1rem}}</style></head>
<body><h1>Self-order (LAN)</h1><p>Wi‑Fi only — counter/takeaway. Guests on mobile data need Cloud later.</p>
<div id="menu"></div><div id="cart"><h2>Cart</h2><ul id="lines"></ul>
<button id="send">Send to counter</button><pre id="out"></pre></div>
<script>
const menu = {menu};
const cart = [];
const root = document.getElementById('menu');
for (const it of menu.items) {{
  const b = document.createElement('button');
  b.textContent = it.name + ' · ' + (it.price_paise/100).toFixed(2);
  b.onclick = () => {{ cart.push(it); render(); }};
  root.appendChild(b);
}}
function render() {{
  document.getElementById('lines').innerHTML = cart.map(c => '<li>'+c.name+'</li>').join('');
}}
document.getElementById('send').onclick = async () => {{
  const r = await fetch('/kiosk_order', {{method:'POST', headers:{{'Content-Type':'application/json'}},
    body: JSON.stringify({{ items: cart.map(c => ({{ product_id: c.id, name: c.name, price_paise: c.price_paise }})) }})}});
  document.getElementById('out').textContent = await r.text();
  cart.length = 0; render();
}};
</script></body></html>"#
        ))
    }

    pub fn table_order_html(&self, table_id: &str) -> Result<String, String> {
        let label = self.table_label(table_id)?;
        let menu = self.menu_public_json()?;
        Ok(format!(
            r#"<!doctype html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>{label}</title>
<style>body{{font-family:sans-serif;padding:1rem}}button{{display:block;width:100%;margin:.35rem 0;padding:.75rem;font-size:1.05rem}}
.note{{opacity:.7;font-size:.9rem}}</style></head>
<body><h1>Table {label}</h1>
<p class="note">LAN / restaurant Wi‑Fi only. Guests on mobile data need Kumkaj Cloud later.</p>
<div id="menu"></div><ul id="cart"></ul>
<button id="add">Add to order</button><pre id="out"></pre>
<script>
const tableId = {tid};
const menu = {menu};
const pick = [];
const root = document.getElementById('menu');
for (const it of menu.items) {{
  const b = document.createElement('button');
  b.textContent = it.name;
  b.onclick = () => {{ pick.push(it); document.getElementById('cart').innerHTML = pick.map(p=>'<li>'+p.name+'</li>').join(''); }};
  root.appendChild(b);
}}
document.getElementById('add').onclick = async () => {{
  const r = await fetch('/t/' + encodeURIComponent(tableId) + '/add', {{
    method:'POST', headers:{{'Content-Type':'application/json'}},
    body: JSON.stringify({{ items: pick.map(p => ({{ product_id: p.id }})) }})
  }});
  document.getElementById('out').textContent = await r.text();
  pick.length = 0; document.getElementById('cart').innerHTML = '';
}};
</script></body></html>"#,
            tid = serde_json::to_string(table_id).unwrap_or_else(|_| "\"\"".into()),
            label = html_esc(&label),
            menu = menu
        ))
    }

    /// Public LAN: add products to a table's running order (guest QR).
    pub fn public_add_to_table(
        &self,
        table_id: &str,
        product_ids: &[String],
    ) -> Result<(), String> {
        self.assert_server_writer("QR table order")?;
        let actor = self
            .list_staff()?
            .into_iter()
            .find(|s| s.role.can_admin())
            .ok_or_else(|| "No owner on books".to_string())?;
        let area = self
            .list_tables()?
            .into_iter()
            .find(|t| t.id == table_id)
            .map(|t| t.area)
            .unwrap_or_default();
        for pid in product_ids {
            let Some(p) = self.get_product(pid)? else {
                continue;
            };
            self.bump_pending_product(table_id, &p, 1000, &area, &actor)?;
        }
        Ok(())
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

        let calls = self.list_open_waiter_calls()?.len();
        let waits = self.list_waitlist()?.len();
        let bday_week = self.crm_segment(CrmSegment::BirthdayThisWeek)?.len();
        let mut counts = format!("Open waiter calls: {calls}\nWaitlist: {waits}");
        if bday_week > 0 {
            counts.push_str(&format!("\nBirthdays this week: {bday_week}"));
        }
        parts.push(counts);

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
    use crate::domain::{Cart, Money, Product, StaffRole};
    use crate::schemes::Scheme;

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
        let path = std::env::temp_dir().join(format!("khaynjo-ops-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        Shop { store, path, owner }
    }

    fn empty_draft() -> crate::domain::SaleDraft {
        crate::domain::SaleDraft {
            lines: Vec::new(),
            lines_total_paise: 0,
            rounding_paise: 0,
            total_paise: 0,
            tax_paise: 0,
            order_type: Default::default(),
            money_lane: Default::default(),
            user_at: None,
            extras_json: "[]".into(),
            notify_phone: String::new(),
            discount_paise: 0,
            discount_reason: String::new(),
            discount_approval_pin: String::new(),
            delivery_by: String::new(),
        }
    }

    #[test]
    fn waitlist_seat_and_cancel() {
        let shop = shop();
        let w = shop.store.add_waitlist("Ravi", 4, 15, &shop.owner).unwrap();
        assert_eq!(shop.store.list_waitlist().unwrap().len(), 1);
        let t = shop.store.add_table_in_area("W1", "Dine In").unwrap();
        shop.store.seat_waitlist(&w.id, &t.id, &shop.owner).unwrap();
        assert!(shop.store.list_waitlist().unwrap().is_empty());
        let w2 = shop.store.add_waitlist("Asha", 2, 10, &shop.owner).unwrap();
        shop.store.cancel_waitlist(&w2.id, &shop.owner).unwrap();
        assert!(shop.store.list_waitlist().unwrap().is_empty());
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
            .add_staff("Wasim", StaffRole::Waiter, "Captain", "2222", &shop.owner)
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
    fn fixed_combo_kot_expands_and_hold_skips_print() {
        let shop = shop();
        let thali = Product::new_basic(new_id(), "Thali Combo", Money::from_rupees(250));
        let rice = Product::new_basic(new_id(), "Rice", Money::from_rupees(80));
        let dal = Product::new_basic(new_id(), "Dal", Money::from_rupees(60));
        shop.store.upsert_product(&thali).unwrap();
        shop.store.upsert_product(&rice).unwrap();
        shop.store.upsert_product(&dal).unwrap();
        let scheme = Scheme::fixed_combo(
            new_id(),
            "Thali",
            &thali.id,
            &[(&rice.id, 1), (&dal.id, 1)],
            25_000,
        );
        shop.store.upsert_scheme(&scheme).unwrap();
        let table = shop.store.add_table_in_area("C1", "Dine In").unwrap();
        shop.store
            .seat_fixed_combo(&table.id, &thali.id, &shop.owner)
            .unwrap();
        let order = shop.store.running_order(&table.id).unwrap().unwrap();
        assert!(
            order.lines.iter().any(|l| l.held && l.product_id == rice.id),
            "components held until combo KOT"
        );
        shop.store
            .bump_pending_product(&table.id, &rice, 1000, "", &shop.owner)
            .unwrap();
        let extra = shop
            .store
            .running_order(&table.id)
            .unwrap()
            .unwrap()
            .lines
            .into_iter()
            .find(|l| l.product_id == rice.id && l.note.is_empty() && l.kot_id.is_empty())
            .unwrap();
        shop.store
            .set_line_held(&extra.id, true, &shop.owner)
            .unwrap();
        let kot_ids = shop
            .store
            .send_kot(&empty_draft(), Some(&table.id), &shop.owner)
            .unwrap();
        assert!(!kot_ids.is_empty());
        let text = shop.store.format_kot_text(&kot_ids[0]).unwrap();
        assert!(text.contains("Rice") || text.contains("Dal"), "{text}");
        let still_held = shop.store.running_order(&table.id).unwrap().unwrap();
        assert!(
            still_held
                .lines
                .iter()
                .any(|l| l.id == extra.id && l.kot_id.is_empty() && l.held),
            "held non-combo line never printed"
        );
    }

    #[test]
    fn split_evenly_makes_companions() {
        let shop = shop();
        let a = Product::new_basic(new_id(), "A", Money::from_rupees(100));
        let b = Product::new_basic(new_id(), "B", Money::from_rupees(100));
        let c = Product::new_basic(new_id(), "C", Money::from_rupees(100));
        for p in [&a, &b, &c] {
            shop.store.upsert_product(p).unwrap();
        }
        let t = shop.store.add_table_in_area("S1", "Dine In").unwrap();
        for p in [&a, &b, &c] {
            shop.store
                .bump_pending_product(&t.id, p, 1000, "", &shop.owner)
                .unwrap();
        }
        let mut cart = Cart::default();
        cart.lines = shop
            .store
            .running_order(&t.id)
            .unwrap()
            .unwrap()
            .to_cart_lines();
        shop.store
            .send_kot(&cart.to_sale_draft().unwrap(), Some(&t.id), &shop.owner)
            .unwrap();
        let companions = shop.store.split_evenly(&t.id, 3, &shop.owner).unwrap();
        assert_eq!(companions.len(), 2);
    }

    #[test]
    fn allergen_prints_on_kot_text() {
        let shop = shop();
        let mut p = Product::new_basic(new_id(), "Paneer", Money::from_rupees(180));
        shop.store.upsert_product(&p).unwrap();
        shop.store
            .set_product_allergens(&p.id, "dairy, nuts", &shop.owner)
            .unwrap();
        p.allergens = "dairy, nuts".into();
        let t = shop.store.add_table_in_area("A1", "").unwrap();
        shop.store
            .bump_pending_product(&t.id, &p, 1000, "", &shop.owner)
            .unwrap();
        let ids = shop
            .store
            .send_kot(&empty_draft(), Some(&t.id), &shop.owner)
            .unwrap();
        let text = shop.store.format_kot_text(&ids[0]).unwrap();
        assert!(
            text.to_lowercase().contains("dairy") || text.to_lowercase().contains("allergen"),
            "{text}"
        );
    }

    #[test]
    fn whatsapp_url_encodes_bill() {
        let u = whatsapp_bill_url("Bill ₹100\nThanks");
        assert!(u.starts_with("https://wa.me/?text="));
        assert!(u.contains("Bill"));
    }

    #[test]
    fn set_table_layout_round_trip() {
        let shop = shop();
        let t = shop.store.add_table_in_area("L1", "Dine In").unwrap();
        shop.store
            .set_table_layout(&t.id, 40.0, 80.0, 130.0, 95.0, &shop.owner)
            .unwrap();
        let (x, y, w, h) = shop.store.table_layout(&t.id).unwrap();
        assert_eq!((x, y, w, h), (40.0, 80.0, 130.0, 95.0));
        let listed = shop
            .store
            .list_tables()
            .unwrap()
            .into_iter()
            .find(|row| row.id == t.id)
            .unwrap();
        assert_eq!(listed.layout_x, 40.0);
        assert_eq!(listed.layout_y, 80.0);
        assert_eq!(listed.layout_w, 130.0);
        assert_eq!(listed.layout_h, 95.0);
    }

    #[test]
    fn ensure_table_layouts_packed_assigns_nonzero() {
        let shop = shop();
        let a = shop.store.add_table_in_area("P1", "Dine In").unwrap();
        let b = shop.store.add_table_in_area("P2", "Dine In").unwrap();
        let n = shop
            .store
            .ensure_table_layouts_packed(&shop.owner)
            .unwrap();
        assert!(n >= 2, "packed {n}");
        let (x1, y1, w1, h1) = shop.store.table_layout(&a.id).unwrap();
        let (x2, y2, w2, h2) = shop.store.table_layout(&b.id).unwrap();
        assert!(w1 > 0.0 && h1 > 0.0, "{w1}×{h1}");
        assert!(w2 > 0.0 && h2 > 0.0, "{w2}×{h2}");
        assert!(
            (x1, y1) != (x2, y2) || a.id == b.id,
            "distinct pack slots ({x1},{y1}) vs ({x2},{y2})"
        );
        // Second call must not reshuffle once positions exist.
        assert_eq!(
            shop.store.ensure_table_layouts_packed(&shop.owner).unwrap(),
            0
        );
        assert_eq!(shop.store.table_layout(&a.id).unwrap(), (x1, y1, w1, h1));
    }

    #[test]
    fn floor_view_meta_persists() {
        let shop = shop();
        assert_eq!(shop.store.floor_view().unwrap(), "grid");
        shop.store.set_floor_view("layout").unwrap();
        assert_eq!(shop.store.floor_view().unwrap(), "layout");
        shop.store.set_floor_view("grid").unwrap();
        assert_eq!(shop.store.floor_view().unwrap(), "grid");
    }

    #[test]
    fn fire_course_skips_other_held() {
        let shop = shop();
        let a = Product::new_basic(new_id(), "Soup", Money::from_rupees(80));
        let b = Product::new_basic(new_id(), "Grill", Money::from_rupees(200));
        shop.store.upsert_product(&a).unwrap();
        shop.store.upsert_product(&b).unwrap();
        let t = shop.store.add_table_in_area("F1", "Dine In").unwrap();
        shop.store
            .bump_pending_product(&t.id, &a, 1000, "", &shop.owner)
            .unwrap();
        shop.store
            .bump_pending_product(&t.id, &b, 1000, "", &shop.owner)
            .unwrap();
        let order = shop.store.running_order(&t.id).unwrap().unwrap();
        let soup = order
            .lines
            .iter()
            .find(|l| l.product_id == a.id)
            .unwrap()
            .id
            .clone();
        let grill = order
            .lines
            .iter()
            .find(|l| l.product_id == b.id)
            .unwrap()
            .id
            .clone();
        shop.store.set_line_course(&soup, 1, &shop.owner).unwrap();
        shop.store.set_line_course(&grill, 2, &shop.owner).unwrap();
        shop.store.set_line_held(&grill, true, &shop.owner).unwrap();
        let ids = shop.store.fire_course(&t.id, 1, &shop.owner).unwrap();
        assert!(!ids.is_empty());
        let text = shop.store.format_kot_text(&ids[0]).unwrap();
        assert!(text.contains("Soup"), "{text}");
        assert!(!text.contains("Grill"), "{text}");
        let still = shop.store.running_order(&t.id).unwrap().unwrap();
        assert!(
            still
                .lines
                .iter()
                .any(|l| l.id == grill && l.kot_id.is_empty() && l.held),
            "course 2 stayed held"
        );
    }

    #[test]
    fn lan_public_html_pages_ok() {
        let shop = shop();
        let t = shop.store.add_table_in_area("L1", "Dine In").unwrap();
        let display = shop.store.display_order_html(Some(&t.id)).unwrap();
        assert!(display.contains("<html") || display.contains("<!doctype"), "{display}");
        let tokens = shop.store.tokens_ready_html().unwrap();
        assert!(tokens.to_lowercase().contains("token") || tokens.contains("<html"), "{tokens}");
        let expo = shop.store.expo_ready_html().unwrap();
        assert!(expo.contains("<html") || expo.contains("<!doctype"), "{expo}");
        let kiosk = shop.store.kiosk_html().unwrap();
        assert!(kiosk.contains("<html") || kiosk.contains("menu"), "{kiosk}");
        let table = shop.store.table_order_html(&t.id).unwrap();
        assert!(table.contains("<html") || table.contains("L1"), "{table}");
        let ops = shop.store.ops_panel_text().unwrap();
        assert!(ops.contains("/display"), "{ops}");
    }
}

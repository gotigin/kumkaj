//! The restaurant's service day: opening float, shift handover, and the
//! owner's day report (who sold what, when, how, and what was given away).
//!
//! Mechanics: `BUSINESS_BOOKS.md` § Shift and day-end cash — float is the
//! business's money (not a sale), expected = float + cash taken − cash paid
//! out, variance is named and kept, tips are the staff's (not sales).

use super::Store;
use crate::domain::{Money, StaffMember};
use chrono::Local;
use rusqlite::params;

/// One drawer handover: what the closing person had, what was counted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShiftClose {
    pub started_at: String,
    pub ended_at: String,
    pub closed_by: String,
    pub bills: i64,
    pub cash_paise: i64,
    pub upi_paise: i64,
    pub card_paise: i64,
    pub cash_tips_paise: i64,
    pub cash_out_paise: i64,
    pub float_paise: i64,
    pub expected_paise: i64,
    pub counted_paise: i64,
}

impl ShiftClose {
    /// Counted − expected: negative = short, positive = over.
    pub fn variance_paise(&self) -> i64 {
        self.counted_paise - self.expected_paise
    }
}

/// A table booking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservation {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub guests: i64,
    /// Local time, "YYYY-MM-DD HH:MM".
    pub at: String,
    pub table_name: String,
    pub status: String,
}

/// "20:30" (today), "tomorrow 13:00" or "2026-10-26 20:30" → "YYYY-MM-DD HH:MM".
fn parse_booking_time(text: &str, now: chrono::NaiveDateTime) -> Result<String, String> {
    let t = text.trim().to_lowercase();
    let bad = || format!("Write the time as 20:30, tomorrow 13:00 or 2026-10-26 20:30 (not \"{}\")", text.trim());
    let (day, clock) = match t.rsplit_once(' ') {
        Some((d, c)) => (d.trim().to_string(), c.trim().to_string()),
        None => (String::new(), t.clone()),
    };
    let time = chrono::NaiveTime::parse_from_str(&clock, "%H:%M").map_err(|_| bad())?;
    let date = match day.as_str() {
        "" | "today" => now.date(),
        "tomorrow" => now.date() + chrono::Duration::days(1),
        d => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| bad())?,
    };
    Ok(date.and_time(time).format("%Y-%m-%d %H:%M").to_string())
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

impl Store {
    /// Cash put in the drawer this morning to give change.
    pub fn opening_float_paise(&self) -> Result<i64, String> {
        Ok(self
            .meta_get(&format!("opening_float_{}", today()))?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0))
    }

    pub fn set_opening_float(&self, paise: i64, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Set the opening cash")?;
        if paise < 0 {
            return Err("Opening cash cannot be negative".into());
        }
        self.meta_set(&format!("opening_float_{}", today()), &paise.to_string())?;
        let currency = self.locale_settings()?.currency;
        self.audit(actor, "day.float", &crate::locale::format_money(paise, &currency))?;
        Ok(())
    }

    /// Cash tips taken today (they sit in the drawer but are the staff's).
    pub fn today_cash_tips_paise(&self) -> Result<i64, String> {
        self.cash_tips_since(&format!("{}T00:00:00", today()))
    }

    fn cash_tips_since(&self, since: &str) -> Result<i64, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(tip_paise), 0) FROM sales
                  WHERE created_at >= ?1 AND status = 'completed' AND tip_method = 'cash'",
                params![since],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    /// Takings by tender since `since` (RFC3339): (bills, cash, upi, card).
    fn takings_since(&self, since: &str) -> Result<(i64, i64, i64, i64), String> {
        let bills: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sales WHERE created_at >= ?1 AND status = 'completed'",
                params![since],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let tender = |method: &str| -> Result<i64, String> {
            self.conn
                .query_row(
                    "SELECT
                       COALESCE((SELECT SUM(total_paise) FROM sales
                                  WHERE created_at >= ?1 AND status = 'completed'
                                    AND pay_method = ?2), 0)
                     + COALESCE((SELECT SUM(p.amount_paise) FROM sale_payments p
                                  JOIN sales s ON s.id = p.sale_id
                                  WHERE s.created_at >= ?1 AND s.status = 'completed'
                                    AND s.pay_method = 'split' AND p.pay_method = ?2), 0)",
                    params![since, method],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())
        };
        Ok((bills, tender("cash")?, tender("upi")?, tender("card")?))
    }

    /// Cash that left the drawer since `since`: cash expenses, cash paid to
    /// suppliers, till → bank.
    fn cash_out_since(&self, since: &str) -> Result<i64, String> {
        let one = |sql: &str| -> Result<i64, String> {
            self.conn
                .query_row(sql, params![since], |r| r.get::<_, i64>(0))
                .map_err(|e| e.to_string())
        };
        Ok(one(
            "SELECT COALESCE(SUM(p.amount_paise), 0) FROM expense_payments p
               JOIN expenses e ON e.id = p.expense_id
              WHERE p.pay_method = 'cash' AND e.created_at >= ?1",
        )? + one(
            "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM payable_ledger
              WHERE kind = 'settle' AND pay_method = 'cash' AND amount_paise < 0
                AND created_at >= ?1",
        )? + one(
            "SELECT COALESCE(SUM(amount_paise), 0) FROM money_movements
              WHERE kind = 'till_to_bank' AND created_at >= ?1",
        )?)
    }

    /// When the current shift started: the last shift close today, else the
    /// start of today.
    pub fn shift_started_at(&self) -> Result<String, String> {
        let start_of_day = format!("{}T00:00:00", today());
        Ok(self
            .meta_get("last_shift_close_at")?
            .filter(|t| t.as_str() >= start_of_day.as_str())
            .unwrap_or(start_of_day))
    }

    /// What the drawer should hold right now for this shift (no count yet).
    pub fn shift_so_far(&self, actor: &StaffMember) -> Result<ShiftClose, String> {
        let started_at = self.shift_started_at()?;
        let (bills, cash, upi, card) = self.takings_since(&started_at)?;
        let cash_tips = self.cash_tips_since(&started_at)?;
        let cash_out = self.cash_out_since(&started_at)?;
        // The float stays in the drawer across shifts (it is the outlet's
        // money), so every shift's expected cash includes it.
        let float = self.opening_float_paise()?;
        Ok(ShiftClose {
            started_at,
            ended_at: Local::now().to_rfc3339(),
            closed_by: actor.name.clone(),
            bills,
            cash_paise: cash,
            upi_paise: upi,
            card_paise: card,
            cash_tips_paise: cash_tips,
            cash_out_paise: cash_out,
            float_paise: float,
            expected_paise: float + cash + cash_tips - cash_out,
            counted_paise: 0,
        })
    }

    /// Hand the drawer over: record the count under the closing person's
    /// name, start the next shift, and return the handover slip text. The
    /// variance is kept as counted — never corrected.
    pub fn close_shift(&self, actor: &StaffMember, counted_paise: i64) -> Result<(ShiftClose, String), String> {
        self.assert_server_writer("Close the shift")?;
        if counted_paise < 0 {
            return Err("Counted cash cannot be negative".into());
        }
        let mut shift = self.shift_so_far(actor)?;
        shift.counted_paise = counted_paise;
        self.conn
            .execute(
                "INSERT INTO shift_closes(
                    id, started_at, ended_at, closed_by, bills, cash_paise, upi_paise,
                    card_paise, cash_tips_paise, cash_out_paise, float_paise,
                    expected_paise, counted_paise
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    crate::domain::new_id(),
                    shift.started_at,
                    shift.ended_at,
                    shift.closed_by,
                    shift.bills,
                    shift.cash_paise,
                    shift.upi_paise,
                    shift.card_paise,
                    shift.cash_tips_paise,
                    shift.cash_out_paise,
                    shift.float_paise,
                    shift.expected_paise,
                    shift.counted_paise
                ],
            )
            .map_err(|e| e.to_string())?;
        self.meta_set("last_shift_close_at", &shift.ended_at)?;
        let slip = self.format_shift_slip(&shift)?;
        self.audit(
            actor,
            "shift.close",
            &slip.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" · "),
        )?;
        Ok((shift, slip))
    }

    pub fn format_shift_slip(&self, s: &ShiftClose) -> Result<String, String> {
        let fiscal = self.fiscal_settings()?;
        let currency = self.locale_settings()?.currency;
        let m = |p: i64| Money(p).format_localized(&currency, &fiscal);
        let hm = |t: &str| t.get(11..16).unwrap_or(t).to_string();
        let variance = s.variance_paise();
        let mut out = String::from("SHIFT CLOSE\n");
        let venue = self.venue_title()?;
        if !venue.trim().is_empty() {
            out.push_str(&format!("{}\n", venue.trim()));
        }
        out.push_str(&format!("{} – {} · {}\n", hm(&s.started_at), hm(&s.ended_at), s.closed_by));
        out.push_str("--------------------------------\n");
        out.push_str(&format!("Bills: {}\n", s.bills));
        out.push_str(&format!(
            "Cash {} · {} {} · Card {}\n",
            m(s.cash_paise),
            fiscal.digital_pay_label(),
            m(s.upi_paise),
            m(s.card_paise)
        ));
        out.push_str(&format!("Opening cash (float): {}\n", m(s.float_paise)));
        if s.cash_tips_paise > 0 {
            out.push_str(&format!("Cash tips (staff's): {}\n", m(s.cash_tips_paise)));
        }
        if s.cash_out_paise > 0 {
            out.push_str(&format!("Cash paid out: {}\n", m(s.cash_out_paise)));
        }
        out.push_str(&format!("Expected in drawer: {}\n", m(s.expected_paise)));
        out.push_str(&format!("Counted: {}\n", m(s.counted_paise)));
        out.push_str(&format!(
            "{}: {}\n",
            if variance < 0 { "Short" } else if variance > 0 { "Over" } else { "Variance" },
            m(variance.abs())
        ));
        Ok(out)
    }

    /// The owner's day at a glance: by staff, by order type, by hour, top
    /// items, discounts.
    /// Book a table: guest name, phone, how many, when ("20:30", "tomorrow
    /// 13:00", "2026-10-26 20:30") and optionally which table.
    pub fn add_reservation(
        &self,
        name: &str,
        phone: &str,
        guests: i64,
        when: &str,
        table_name: &str,
        actor: &StaffMember,
    ) -> Result<Reservation, String> {
        self.assert_server_writer("Book tables")?;
        let name = name.trim();
        if name.is_empty() {
            return Err("Guest name required".into());
        }
        if guests < 0 {
            return Err("Guests can't be negative".into());
        }
        let table_name = table_name.trim();
        if !table_name.is_empty()
            && !self.list_tables()?.iter().any(|t| t.name.eq_ignore_ascii_case(table_name))
        {
            return Err(format!("No table named \"{table_name}\""));
        }
        let at = parse_booking_time(when, Local::now().naive_local())?;
        let r = Reservation {
            id: crate::domain::new_id(),
            name: name.to_string(),
            phone: phone.trim().to_string(),
            guests,
            at,
            table_name: table_name.to_string(),
            status: "booked".into(),
        };
        self.conn
            .execute(
                "INSERT INTO reservations(id, name, phone, guests, at, table_name, status, created_at, created_by)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'booked', ?7, ?8)",
                params![r.id, r.name, r.phone, r.guests, r.at, r.table_name, Local::now().to_rfc3339(), actor.name],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "reservation.add", &format!("{} · {} · {}", r.name, r.guests, r.at))?;
        Ok(r)
    }

    /// Bookings still to come (and those up to two hours late), soonest first.
    pub fn upcoming_reservations(&self) -> Result<Vec<Reservation>, String> {
        let from = (Local::now().naive_local() - chrono::Duration::hours(2))
            .format("%Y-%m-%d %H:%M")
            .to_string();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, phone, guests, at, table_name, status FROM reservations
                  WHERE status = 'booked' AND at >= ?1 ORDER BY at, name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![from], |r| {
                Ok(Reservation {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    phone: r.get(2)?,
                    guests: r.get(3)?,
                    at: r.get(4)?,
                    table_name: r.get(5)?,
                    status: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Guests arrived ("seated") or the booking is off ("cancelled").
    pub fn set_reservation_status(&self, id: &str, status: &str, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Change bookings")?;
        if !matches!(status, "seated" | "cancelled") {
            return Err("Unknown booking status".into());
        }
        let n = self
            .conn
            .execute(
                "UPDATE reservations SET status = ?2 WHERE id = ?1 AND status = 'booked'",
                params![id, status],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("That booking is no longer open".into());
        }
        self.audit(actor, &format!("reservation.{status}"), id)?;
        Ok(())
    }

    /// Names that took delivery orders out before, most used first.
    pub fn delivery_people(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT delivery_by FROM sale_delivery GROUP BY delivery_by ORDER BY COUNT(*) DESC LIMIT 30")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| r.get(0)).map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn restaurant_day_report_text(&self) -> Result<String, String> {
        let day = today();
        let fiscal = self.fiscal_settings()?;
        let currency = self.locale_settings()?.currency;
        let m = |p: i64| Money(p).format_localized(&currency, &fiscal);
        let grouped = |sql: &str| -> Result<Vec<(String, i64, i64)>, String> {
            let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![day], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
        };
        let by_staff = grouped(
            "SELECT created_by_name, COUNT(*), COALESCE(SUM(total_paise), 0) FROM sales
              WHERE created_at >= ?1 AND created_at < ?1 || 'Z' AND status = 'completed'
              GROUP BY created_by_name ORDER BY 3 DESC",
        )?;
        let by_type = grouped(
            "SELECT order_type, COUNT(*), COALESCE(SUM(total_paise), 0) FROM sales
              WHERE created_at >= ?1 AND created_at < ?1 || 'Z' AND status = 'completed'
              GROUP BY order_type ORDER BY 3 DESC",
        )?;
        let by_hour = grouped(
            "SELECT substr(created_at, 12, 2), COUNT(*), COALESCE(SUM(total_paise), 0) FROM sales
              WHERE created_at >= ?1 AND created_at < ?1 || 'Z' AND status = 'completed'
              GROUP BY 1 ORDER BY 1",
        )?;
        let top_items = grouped(
            "SELECT l.name, COALESCE(SUM(CASE WHEN l.qty_milli > 0 THEN l.qty_milli ELSE l.qty * 1000 END), 0),
                    COALESCE(SUM(l.line_total_paise), 0)
               FROM sale_lines l JOIN sales s ON s.id = l.sale_id
              WHERE s.created_at >= ?1 AND s.created_at < ?1 || 'Z' AND s.status = 'completed'
              GROUP BY l.name ORDER BY 3 DESC LIMIT 10",
        )?;
        // Dish families (Half + Full of one dish) totalled together.
        let by_group = grouped(
            "SELECT p.group_name,
                    COALESCE(SUM(CASE WHEN l.qty_milli > 0 THEN l.qty_milli ELSE l.qty * 1000 END), 0),
                    COALESCE(SUM(l.line_total_paise), 0)
               FROM sale_lines l
               JOIN sales s ON s.id = l.sale_id
               JOIN products p ON p.id = l.product_id
              WHERE s.created_at >= ?1 AND s.created_at < ?1 || 'Z' AND s.status = 'completed'
                AND p.group_name != ''
              GROUP BY p.group_name ORDER BY 3 DESC",
        )?;
        let by_delivery = grouped(
            "SELECT d.delivery_by, COUNT(*), COALESCE(SUM(s.total_paise), 0)
               FROM sales s JOIN sale_delivery d ON d.sale_id = s.id
              WHERE s.created_at >= ?1 AND s.created_at < ?1 || 'Z' AND s.status = 'completed'
              GROUP BY d.delivery_by ORDER BY 3 DESC",
        )?;
        let discounts = grouped(
            "SELECT discount_reason, COUNT(*), COALESCE(SUM(discount_paise), 0) FROM sales
              WHERE created_at >= ?1 AND created_at < ?1 || 'Z' AND status = 'completed' AND discount_paise > 0
              GROUP BY discount_reason ORDER BY 3 DESC",
        )?;
        let mut out = String::new();
        let section = |out: &mut String, title: &str, rows: &[(String, i64, i64)], label: &dyn Fn(&str) -> String, count: &dyn Fn(i64) -> String| {
            if rows.is_empty() {
                return;
            }
            out.push_str(&format!("\n{title}\n"));
            for (key, n, paise) in rows {
                out.push_str(&format!("  {} · {} · {}\n", label(key), count(*n), m(*paise)));
            }
        };
        let bills = |n: i64| format!("{n} bill{}", if n == 1 { "" } else { "s" });
        section(&mut out, "By staff", &by_staff, &|k| if k.is_empty() { "—".into() } else { k.to_string() }, &bills);
        section(
            &mut out,
            "By order type",
            &by_type,
            &|k| crate::domain::OrderType::parse(k).label().to_string(),
            &bills,
        );
        section(&mut out, "By hour", &by_hour, &|k| format!("{k}:00"), &bills);
        section(&mut out, "Deliveries by", &by_delivery, &|k| k.to_string(), &bills);
        section(
            &mut out,
            "Top items",
            &top_items,
            &|k| k.to_string(),
            &|milli| {
                let whole = milli / 1000;
                if milli % 1000 == 0 { format!("×{whole}") } else { format!("×{:.2}", milli as f64 / 1000.0) }
            },
        );
        section(
            &mut out,
            "By dish group",
            &by_group,
            &|k| k.to_string(),
            &|milli| {
                let whole = milli / 1000;
                if milli % 1000 == 0 { format!("×{whole}") } else { format!("×{:.2}", milli as f64 / 1000.0) }
            },
        );
        section(&mut out, "Discounts", &discounts, &|k| k.to_string(), &bills);
        // Schemes recorded on extras_json as {kind:"scheme", name}.
        let scheme_rows: Vec<(String, i64, i64)> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT extras_json FROM sales
                      WHERE created_at >= ?1 AND created_at < ?1 || 'Z' AND status = 'completed'
                        AND extras_json LIKE '%scheme%'",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![day], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            let mut counts: std::collections::BTreeMap<String, (i64, i64)> =
                std::collections::BTreeMap::new();
            for row in rows {
                let json = row.map_err(|e| e.to_string())?;
                let extras: Vec<serde_json::Value> =
                    serde_json::from_str(&json).unwrap_or_default();
                for ex in extras {
                    if ex.get("kind").and_then(|v| v.as_str()) == Some("scheme") {
                        let name = ex
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Scheme")
                            .to_string();
                        let e = counts.entry(name).or_insert((0, 0));
                        e.0 += 1;
                    }
                }
            }
            counts.into_iter().map(|(k, (n, p))| (k, n, p)).collect()
        };
        section(&mut out, "Schemes", &scheme_rows, &|k| k.to_string(), &bills);

        // Variations (Size Large) and add-ons (Toppings Cheese) from sale lines.
        let option_rows: Vec<(String, i64, i64)> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT l.modifiers_json,
                            CASE WHEN l.qty_milli > 0 THEN l.qty_milli ELSE l.qty * 1000 END
                       FROM sale_lines l
                       JOIN sales s ON s.id = l.sale_id
                      WHERE s.created_at >= ?1 AND s.created_at < ?1 || 'Z'
                        AND s.status = 'completed'
                        AND l.modifiers_json != '' AND l.modifiers_json != '[]'",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![day], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map_err(|e| e.to_string())?;
            let mut counts: std::collections::BTreeMap<String, (i64, i64)> =
                std::collections::BTreeMap::new();
            for row in rows {
                let (json, milli) = row.map_err(|e| e.to_string())?;
                let mods: Vec<crate::domain::ModifierOption> =
                    serde_json::from_str(&json).unwrap_or_default();
                let portions = (milli / 1000).max(1);
                for m in mods {
                    let key = if m.group.is_empty() || m.group == "Options" {
                        m.name
                    } else {
                        format!("{} · {}", m.group, m.name)
                    };
                    let e = counts.entry(key).or_insert((0, 0));
                    e.0 += portions;
                    e.1 += m.price_paise.saturating_mul(portions);
                }
            }
            let mut v: Vec<_> = counts.into_iter().map(|(k, (n, p))| (k, n, p)).collect();
            v.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.1.cmp(&a.1)));
            v.truncate(20);
            v
        };
        section(
            &mut out,
            "Options sold (sizes & toppings)",
            &option_rows,
            &|k| k.to_string(),
            &|n| format!("×{n}"),
        );
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{
        new_id, BillDiscount, Cart, DiscountKind, Money, PayMethod, Product, StaffRole,
    };
    use crate::store::Store;

    fn shop() -> (Store, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("khaynjo-service-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        store
            .upsert_product(&Product::new_basic(new_id(), "Thali", Money::from_rupees(200)))
            .unwrap();
        (store, path)
    }

    #[test]
    fn booking_times_read_the_way_people_say_them() {
        let now = chrono::NaiveDate::from_ymd_opt(2026, 10, 25).unwrap().and_hms_opt(12, 0, 0).unwrap();
        assert_eq!(super::parse_booking_time("20:30", now).unwrap(), "2026-10-25 20:30");
        assert_eq!(super::parse_booking_time("Tomorrow 13:00", now).unwrap(), "2026-10-26 13:00");
        assert_eq!(super::parse_booking_time("2026-11-01 19:15", now).unwrap(), "2026-11-01 19:15");
        assert!(super::parse_booking_time("evening", now).is_err());
    }

    /// A booking shows until the guests are seated or it is cancelled.
    #[test]
    fn a_booking_waits_until_seated_or_cancelled() {
        let (store, path) = shop();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        assert!(store.add_reservation("Mehta", "98", 4, "tomorrow 20:30", "Z9", &owner).is_err(), "no such table");
        let r = store.add_reservation("Mehta", "98765", 4, "tomorrow 20:30", "", &owner).unwrap();
        store.add_reservation("Shah", "", 2, "tomorrow 13:00", "", &owner).unwrap();
        let names: Vec<_> = store.upcoming_reservations().unwrap().into_iter().map(|r| r.name).collect();
        assert_eq!(names, vec!["Shah".to_string(), "Mehta".to_string()], "soonest first");
        store.set_reservation_status(&r.id, "seated", &owner).unwrap();
        assert!(store.set_reservation_status(&r.id, "cancelled", &owner).is_err());
        assert_eq!(store.upcoming_reservations().unwrap().len(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn day_report_totals_sizes_and_toppings_sold() {
        use crate::domain::{ModifierGroup, ModifierOption};
        let (mut store, path) = shop();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let mut pizza = store.list_products(false).unwrap().remove(0);
        pizza.name = "Pizza".into();
        pizza.price = Money(0);
        let groups = ModifierGroup::parse_human_list(
            "Size*: Small=100, Large=200 | Toppings+2: Cheese=40",
            "INR",
        )
        .unwrap();
        pizza.set_modifier_groups(&groups);
        store.upsert_product(&pizza).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&pizza);
        let large = ModifierOption {
            name: "Large".into(),
            price_paise: 20_000,
            group: "Size".into(),
        };
        let cheese = ModifierOption {
            name: "Cheese".into(),
            price_paise: 4_000,
            group: "Toppings".into(),
        };
        cart.toggle_modifier_in_groups(0, &large, &groups).unwrap();
        cart.toggle_modifier_in_groups(0, &cheese, &groups).unwrap();
        store
            .finalize_sale(
                &cart.to_sale_draft().unwrap(),
                PayMethod::Cash,
                None,
                None,
                &owner,
            )
            .unwrap();
        let report = store.restaurant_day_report_text().unwrap();
        assert!(
            report.contains("Options sold")
                && report.contains("Size · Large")
                && report.contains("Toppings · Cheese"),
            "{report}"
        );
        let _ = std::fs::remove_file(path);
    }

    /// A delivery order remembers who took it out; the day report totals
    /// by them and the name is offered next time.
    #[test]
    fn deliveries_are_totalled_by_who_took_them() {
        let (mut store, path) = shop();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        cart.set_order_type(crate::domain::OrderType::Delivery);
        cart.delivery_by = "  Ravi ".into();
        store.finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Cash, None, None, &owner).unwrap();
        let report = store.restaurant_day_report_text().unwrap();
        assert!(report.contains("Deliveries by") && report.contains("Ravi · 1 bill"), "{report}");
        assert_eq!(store.delivery_people().unwrap(), vec!["Ravi".to_string()]);
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn shift_handover_names_the_variance_and_starts_the_next_shift() {
        let (mut store, path) = shop();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let thali = store.list_products(false).unwrap().remove(0);
        store.set_opening_float(Money::from_rupees(2000).paise(), &owner).unwrap();
        for pay in [PayMethod::Cash, PayMethod::Cash, PayMethod::Upi] {
            let mut cart = Cart::default();
            cart.add_or_bump(&thali);
            store.finalize_sale(&cart.to_sale_draft().unwrap(), pay, None, None, &owner).unwrap();
        }
        let so_far = store.shift_so_far(&owner).unwrap();
        assert_eq!(so_far.bills, 3);
        assert_eq!(so_far.cash_paise, Money::from_rupees(400).paise());
        assert_eq!(so_far.upi_paise, Money::from_rupees(200).paise());
        assert_eq!(so_far.expected_paise, Money::from_rupees(2400).paise());
        // Counted ₹160 short: kept as counted, said plainly on the slip.
        let (shift, slip) = store.close_shift(&owner, Money::from_rupees(2240).paise()).unwrap();
        assert_eq!(shift.variance_paise(), -Money::from_rupees(160).paise());
        assert!(slip.contains("SHIFT CLOSE") && slip.contains("Short"), "{slip}");
        // The next shift starts empty (the float stays in the drawer).
        let next = store.shift_so_far(&owner).unwrap();
        assert_eq!(next.bills, 0);
        assert_eq!(next.expected_paise, Money::from_rupees(2000).paise());
        let _ = std::fs::remove_file(path);
    }

    /// A discount is reasoned, approved where the person's right says so
    /// (checked on the Server), spread over the lines for tax, and reported.
    #[test]
    fn discounts_need_a_reason_and_the_right_and_show_in_the_report() {
        let (mut store, path) = shop();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner).unwrap();
        store.add_staff("Mona", StaffRole::Manager, "Manager", "9999", &owner).unwrap();
        let cashier = store.list_staff().unwrap().into_iter().find(|s| s.name == "Cash").unwrap();
        let thali = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        cart.add_or_bump(&thali);
        cart.discount = Some(BillDiscount {
            kind: DiscountKind::Percent(1000),
            reason: String::new(),
            approval_pin: String::new(),
        });
        assert_eq!(cart.applied_total(), Money::from_rupees(360));
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(draft.lines.iter().map(|l| l.line_total_paise).sum::<i64>(), Money::from_rupees(360).paise());
        assert!(store.finalize_sale(&draft, PayMethod::Cash, None, None, &cashier).unwrap_err().contains("why"));
        cart.discount.as_mut().unwrap().reason = "Regular customer".into();
        let draft = cart.to_sale_draft().unwrap();
        assert!(store.finalize_sale(&draft, PayMethod::Cash, None, None, &cashier).unwrap_err().contains("approval"));
        cart.discount.as_mut().unwrap().approval_pin = "9999".into();
        let draft = cart.to_sale_draft().unwrap();
        let sale = store.finalize_sale(&draft, PayMethod::Cash, None, None, &cashier).unwrap();
        assert_eq!(sale.total_paise, Money::from_rupees(360).paise());
        let report = store.restaurant_day_report_text().unwrap();
        assert!(report.contains("Discounts") && report.contains("Regular customer (approved by Mona)"), "{report}");
        assert!(report.contains("By staff") && report.contains("Top items") && report.contains("Thali"), "{report}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_flat_discount_never_exceeds_the_bill_and_parses_both_ways() {
        assert_eq!(DiscountKind::parse("10%", "INR").unwrap(), DiscountKind::Percent(1000));
        assert_eq!(DiscountKind::parse("50", "INR").unwrap(), DiscountKind::Amount(5000));
        assert!(DiscountKind::parse("0", "INR").is_err());
        assert!(DiscountKind::parse("150%", "INR").is_err());
        let thali = Product::new_basic(new_id(), "Thali", Money::from_rupees(200));
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        cart.discount = Some(BillDiscount {
            kind: DiscountKind::Amount(Money::from_rupees(500).paise()),
            reason: "x".into(),
            approval_pin: String::new(),
        });
        assert_eq!(cart.discount_paise(), Money::from_rupees(200).paise());
        assert!(cart.to_sale_draft().is_err(), "a ₹0 bill is not a sale");
    }
}

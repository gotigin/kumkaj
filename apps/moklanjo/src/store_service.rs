//! Mid-day drawer handover and month books lock (courier desk).
//!
//! Mechanics: `BUSINESS_BOOKS.md` § Shift and day-end cash — float is the
//! business's money (not a sale), expected = float + cash taken − cash paid
//! out, variance is named and kept.

use super::Store;
use crate::domain::{Money, PayMethod, StaffMember};
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

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn month_key_from_timestamp(at: &str) -> Option<String> {
    let t = at.trim();
    if t.len() >= 7 && t.as_bytes().get(4) == Some(&b'-') {
        return Some(t[..7].to_string());
    }
    None
}

impl Store {
    pub fn books_locked_through(&self) -> Result<Option<String>, String> {
        Ok(self
            .meta_get("books_locked_through")?
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()))
    }

    pub fn lock_month(&self, yyyy_mm: &str, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Lock books through month")?;
        if !actor.is_owner() {
            return Err("Only the Owner can lock books".into());
        }
        let want = yyyy_mm.trim();
        if want.len() != 7 || want.as_bytes().get(4) != Some(&b'-') {
            return Err("Use YYYY-MM (e.g. 2026-03)".into());
        }
        let (_y, m) = want
            .split_once('-')
            .ok_or_else(|| "Use YYYY-MM".to_string())?;
        let m: u32 = m.parse().map_err(|_| "Month must be 01–12".to_string())?;
        if !(1..=12).contains(&m) {
            return Err("Month must be 01–12".into());
        }
        if let Some(prev) = self.books_locked_through()? {
            if want < prev.as_str() {
                return Err(format!("Books already locked through {prev} — pick that month or later"));
            }
        }
        self.meta_set("books_locked_through", want)?;
        self.audit(actor, "books.lock_month", want)?;
        Ok(())
    }

    pub fn assert_not_month_locked(&self, user_at: &str) -> Result<(), String> {
        let Some(locked) = self.books_locked_through()? else {
            return Ok(());
        };
        let Some(ev) = month_key_from_timestamp(user_at) else {
            return Ok(());
        };
        if ev <= locked {
            return Err(format!("Books are locked through {locked}"));
        }
        Ok(())
    }

    pub fn opening_float_paise(&self) -> Result<i64, String> {
        Ok(self
            .meta_get(&format!("opening_float_{}", today()))?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0))
    }

    /// Takings by tender since `since` (RFC3339): (bills, cash, upi, card).
    fn takings_since(&self, since: &str) -> Result<(i64, i64, i64, i64), String> {
        let branch = self.active_branch_id()?;
        let bills: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sales
                  WHERE created_at >= ?1 AND status = 'completed'
                    AND (branch_id = ?2 OR branch_id = '')",
                params![since, &branch],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let tender = |method: &str| -> Result<i64, String> {
            self.conn
                .query_row(
                    "SELECT
                       COALESCE((SELECT SUM(total_paise) FROM sales
                                  WHERE created_at >= ?1 AND status = 'completed'
                                    AND pay_method = ?2
                                    AND (branch_id = ?3 OR branch_id = '')), 0)
                     + COALESCE((SELECT SUM(p.amount_paise) FROM sale_payments p
                                  JOIN sales s ON s.id = p.sale_id
                                  WHERE s.created_at >= ?1 AND s.status = 'completed'
                                    AND s.pay_method = 'split' AND p.pay_method = ?2
                                    AND (s.branch_id = ?3 OR s.branch_id = '')), 0)",
                    params![since, method, branch],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())
        };
        let cash = tender(PayMethod::Cash.as_str())?;
        let upi = tender(PayMethod::Upi.as_str())?;
        let card = tender(PayMethod::Card.as_str())?
            + tender(PayMethod::Bank.as_str())?;
        Ok((bills, cash, upi, card))
    }

    fn cash_out_since(&self, since: &str) -> Result<i64, String> {
        let branch = self.active_branch_id()?;
        let expenses = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(p.amount_paise), 0) FROM expense_payments p
                   JOIN expenses e ON e.id = p.expense_id
                  WHERE p.pay_method = 'cash' AND e.created_at >= ?1
                    AND (e.branch_id = ?2 OR e.branch_id = '')",
                params![since, &branch],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        let payable = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(ABS(amount_paise)), 0) FROM payable_ledger
                  WHERE kind = 'settle' AND pay_method = 'cash' AND amount_paise < 0
                    AND created_at >= ?1",
                params![since],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        let to_bank = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount_paise), 0) FROM money_movements
                  WHERE kind = 'till_to_bank' AND created_at >= ?1
                    AND (branch_id = ?2 OR branch_id = '')",
                params![since, &branch],
                |r| r.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(expenses + payable + to_bank)
    }

    pub fn shift_started_at(&self) -> Result<String, String> {
        let start_of_day = format!("{}T00:00:00", today());
        Ok(self
            .meta_get("last_shift_close_at")?
            .filter(|t| t.as_str() >= start_of_day.as_str())
            .unwrap_or(start_of_day))
    }

    pub fn shift_so_far(&self, actor: &StaffMember) -> Result<ShiftClose, String> {
        let started_at = self.shift_started_at()?;
        let (bills, cash, upi, card) = self.takings_since(&started_at)?;
        let cash_out = self.cash_out_since(&started_at)?;
        let float = self.opening_float_paise()?;
        Ok(ShiftClose {
            started_at,
            ended_at: Local::now().to_rfc3339(),
            closed_by: actor.name.clone(),
            bills,
            cash_paise: cash,
            upi_paise: upi,
            card_paise: card,
            cash_tips_paise: 0,
            cash_out_paise: cash_out,
            float_paise: float,
            expected_paise: float + cash - cash_out,
            counted_paise: 0,
        })
    }

    pub fn close_shift(
        &self,
        actor: &StaffMember,
        counted_paise: i64,
    ) -> Result<(ShiftClose, String), String> {
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
            &slip
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" · "),
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
        out.push_str(&format!(
            "{} – {} · {}\n",
            hm(&s.started_at),
            hm(&s.ended_at),
            s.closed_by
        ));
        out.push_str("--------------------------------\n");
        out.push_str(&format!("Bookings: {}\n", s.bills));
        out.push_str(&format!(
            "Cash {} · {} {} · Card {}\n",
            m(s.cash_paise),
            fiscal.digital_pay_label(),
            m(s.upi_paise),
            m(s.card_paise)
        ));
        if s.float_paise > 0 {
            out.push_str(&format!("Opening cash (float): {}\n", m(s.float_paise)));
        }
        if s.cash_out_paise > 0 {
            out.push_str(&format!("Cash paid out: {}\n", m(s.cash_out_paise)));
        }
        out.push_str(&format!("Expected in drawer: {}\n", m(s.expected_paise)));
        out.push_str(&format!("Counted: {}\n", m(s.counted_paise)));
        out.push_str(&format!(
            "{}: {}\n",
            if variance < 0 {
                "Short"
            } else if variance > 0 {
                "Over"
            } else {
                "Variance"
            },
            m(variance.abs())
        ));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Cart, PayMethod, Product};
    use std::path::PathBuf;

    fn temp_db() -> PathBuf {
        crate::test_dirs::run_root().join(format!(
            "kumkaj-shift-{}.sqlite3",
            crate::domain::new_id()
        ))
    }

    fn test_product(name: &str, rupees: i64) -> Product {
        Product::new_basic(crate::domain::new_id(), name, Money::from_rupees(rupees))
    }

    #[test]
    fn close_shift_records_row_and_starts_next_shift() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store
            .upsert_product(&test_product("Parcel", 200))
            .unwrap();
        let parcel = store.list_products(false).unwrap().remove(0);
        for pay in [PayMethod::Cash, PayMethod::Upi] {
            let mut cart = Cart::default();
            cart.add_or_bump(&parcel);
            let draft = cart.to_sale_draft().unwrap();
            store.finalize_sale(&draft, pay, None, &owner).unwrap();
        }
        let so_far = store.shift_so_far(&owner).unwrap();
        assert_eq!(so_far.bills, 2);
        assert_eq!(so_far.cash_paise, Money::from_rupees(200).paise());
        let (shift, slip) = store
            .close_shift(&owner, Money::from_rupees(200).paise())
            .unwrap();
        assert_eq!(shift.counted_paise, Money::from_rupees(200).paise());
        assert!(slip.contains("SHIFT CLOSE"));
        let n: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM shift_closes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let next = store.shift_so_far(&owner).unwrap();
        assert_eq!(next.bills, 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn month_lock_blocks_backdated_finalize() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.lock_month("2099-01", &owner).unwrap();
        store
            .upsert_product(&test_product("Parcel", 100))
            .unwrap();
        let parcel = store.list_products(false).unwrap().remove(0);
        let mut cart = Cart::default();
        cart.add_or_bump(&parcel);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.user_at = Some("2099-01-15T10:00:00+05:30".into());
        assert!(store
            .finalize_sale(&draft, PayMethod::Cash, None, &owner)
            .is_err());
        let _ = std::fs::remove_file(path);
    }
}
//! A table's running order: KOT now, more KOTs, print the bill, settle.
//!
//! Petpooja's core loop (and every restaurant's): the waiter punches items
//! and sends a KOT; more KOTs follow; the counter prints the bill and then
//! settles it. Until 2026-10-23 Kirano kept only a KOT *summary* and cleared
//! the cart on send, so the counter had nothing to bill. The open order now
//! lives here, on the Server, line by line (`BUSINESS_BOOKS.md` § Open orders
//! are not books yet): cancelled and complimentary lines are kept with reason
//! and approver (vision §18), moves keep the same order id, and finalize
//! closes the order with its sale.
//!
//! Child module of `store` so it shares the connection and private helpers
//! without growing `store.rs` further.

use super::Store;
use crate::domain::{Right, RightLevel, StaffMember};
use rusqlite::params;

impl Store {
    /// May `actor` do `right`? `Allowed` passes; `WithApproval` needs an
    /// Owner/Manager PIN now (the approver is returned so the action can
    /// record them); `Denied` is refused. Enforced here, on the Server, so a
    /// paired terminal or a skipped screen cannot bypass it (vision §17).
    pub fn check_right(
        &self,
        actor: &StaffMember,
        right: Right,
        approval_pin: Option<&str>,
    ) -> Result<Option<StaffMember>, String> {
        match actor.rights().level(right) {
            RightLevel::Allowed => Ok(None),
            RightLevel::WithApproval => {
                let pin = approval_pin.map(str::trim).unwrap_or_default();
                if pin.is_empty() {
                    return Err(format!(
                        "{} needs a manager's approval — enter an Owner or Manager PIN",
                        right_words(right)
                    ));
                }
                Ok(Some(self.verify_approval_pin(pin)?))
            }
            RightLevel::Denied => Err(format!(
                "{} is not allowed for {} — ask the owner or a manager",
                right_words(right),
                actor.name
            )),
        }
    }

    /// Raw bill header text (address / phone lines), without the FSSAI line.
    pub fn bill_header(&self) -> Result<String, String> {
        Ok(self.meta_get("bill_header")?.unwrap_or_default())
    }

    /// Lines printed under the restaurant's name on every bill: address,
    /// tax id, food licence — whatever the owner typed (one per line).
    pub fn bill_header_lines(&self) -> Result<Vec<String>, String> {
        let mut lines: Vec<String> = self
            .bill_header()?
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        let fssai = self.meta_get("fssai_no")?.unwrap_or_default();
        if !fssai.trim().is_empty() {
            lines.push(format!("FSSAI Lic. No: {}", fssai.trim()));
        }
        Ok(lines)
    }

    pub fn bill_footer(&self) -> Result<String, String> {
        Ok(self.meta_get("bill_footer")?.unwrap_or_default())
    }

    pub fn fssai_no(&self) -> Result<String, String> {
        Ok(self.meta_get("fssai_no")?.unwrap_or_default())
    }

    /// Bill format plus optional UPI VPA / payee for the customer-facing QR.
    pub fn set_bill_format_ex(
        &self,
        header: &str,
        fssai_no: &str,
        footer: &str,
        upi_vpa: Option<&str>,
        upi_payee: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Change the bill format")?;
        self.meta_set("bill_header", header.trim())?;
        self.meta_set("fssai_no", fssai_no.trim())?;
        self.meta_set("bill_footer", footer.trim())?;
        if let Some(v) = upi_vpa {
            self.meta_set("upi_vpa", v.trim())?;
        }
        if let Some(p) = upi_payee {
            self.meta_set("upi_payee", p.trim())?;
        }
        Ok(())
    }

    /// Field show / size / weight / align, plus the typed title. Address,
    /// FSSAI, footer, and UPI stay in their own columns and are copied in.
    pub fn bill_slip_format(&self) -> Result<crate::print_share::BillSlipFormat, String> {
        let raw = self.meta_get("bill_slip_format")?.unwrap_or_default();
        let mut fmt = crate::print_share::BillSlipFormat::parse(&raw);
        let header = self
            .bill_header()?
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" | ");
        fmt.header = header;
        fmt.fssai = self.fssai_no()?;
        // Keep product default footer ("Thanks") when the shop has not set one.
        let meta_footer = self.bill_footer()?;
        if !meta_footer.trim().is_empty() {
            fmt.footer = meta_footer;
        }
        fmt.upi_vpa = self.upi_vpa()?;
        fmt.upi_payee = self.upi_payee()?;
        Ok(fmt.clamped())
    }

    pub fn set_bill_slip_format(
        &self,
        fmt: &crate::print_share::BillSlipFormat,
    ) -> Result<(), String> {
        let fmt = fmt.clone().clamped();
        self.set_bill_format_ex(
            &fmt.header_lines(),
            &fmt.fssai,
            &fmt.footer,
            Some(&fmt.upi_vpa),
            Some(&fmt.upi_payee),
        )?;
        self.meta_set("bill_slip_format", &fmt.to_json())?;
        // Sync bill wrap onto Standard formats only — KOT keeps its own width.
        if let Some(base) = self.path.parent() {
            let mut printers = crate::print_share::load_device_printers(base);
            printers.set_bill_content_line(crate::print_share::wrap_cols_for_paper_mm(
                fmt.shown_content_width_mm(),
            ));
            for f in printers.formats.iter_mut() {
                if crate::print_share::is_standard_document(&f.id) {
                    fmt.apply_to_print_format(f);
                }
            }
            crate::print_share::save_device_printers(base, &printers)?;
        }
        Ok(())
    }

    pub(crate) fn bill_rtl(&self) -> bool {
        self.locale_settings()
            .map(|l| crate::locale::is_rtl(&l.language))
            .unwrap_or(false)
    }

    /// Client cache after a sync snapshot carries the Server's bill face.
    pub fn set_bill_format_from_sync(
        &self,
        header: &str,
        fssai_no: &str,
        footer: &str,
        upi_vpa: &str,
        upi_payee: &str,
    ) -> Result<(), String> {
        self.meta_set("bill_header", header.trim())?;
        self.meta_set("fssai_no", fssai_no.trim())?;
        self.meta_set("bill_footer", footer.trim())?;
        self.meta_set("upi_vpa", upi_vpa.trim())?;
        self.meta_set("upi_payee", upi_payee.trim())?;
        Ok(())
    }

    pub fn upi_vpa(&self) -> Result<String, String> {
        Ok(self.meta_get("upi_vpa")?.unwrap_or_default())
    }

    pub fn upi_payee(&self) -> Result<String, String> {
        Ok(self.meta_get("upi_payee")?.unwrap_or_default())
    }

    /// Settle & Save details on a sale: cash the guest handed over, change
    /// returned, and a tip (owed to staff — not revenue, BUSINESS_BOOKS.md §
    /// Shift and day-end cash).
    pub fn record_settlement(
        &self,
        sale_id: &str,
        tendered_paise: i64,
        change_paise: i64,
        tip_paise: i64,
        tip_method: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        if tendered_paise < 0 || change_paise < 0 || tip_paise < 0 {
            return Err("Amounts cannot be negative".into());
        }
        self.conn
            .execute(
                "UPDATE sales SET tendered_paise = ?2, change_paise = ?3, tip_paise = ?4,
                        tip_method = ?5
                  WHERE id = ?1",
                params![sale_id, tendered_paise, change_paise, tip_paise, tip_method],
            )
            .map_err(|e| e.to_string())?;
        if tip_paise > 0 {
            let currency = self.locale_settings()?.currency;
            self.audit(
                actor,
                "bill.tip",
                &format!("{sale_id} · {}", crate::locale::format_money(tip_paise, &currency)),
            )?;
        }
        Ok(())
    }

    /// Tips taken today (paise) — shown on the day close, kept out of sales.
    pub fn today_tips_paise(&self) -> Result<i64, String> {
        let win = self.current_business_day_window()?;
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(tip_paise), 0) FROM sales
                  WHERE created_at >= ?1 AND created_at < ?2
                    AND status NOT IN ('voided', 'refunded')",
                params![win.start, win.end],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }
}

fn right_words(right: Right) -> &'static str {
    match right {
        Right::Settle => "Taking payment",
        Right::Discount => "Giving a discount",
    }
}

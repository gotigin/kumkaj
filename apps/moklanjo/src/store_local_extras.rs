//! Offline Local extras from courier research (2026-10-29): print brand kit,
//! carrier AWB pools, bulk status, settlement note fields, bill chase, GST CSV,
//! month mix, cargo piece count.

use super::*;
use crate::manifest::TrackingStatus;
use rusqlite::OptionalExtension;

/// One carrier stationery range (start–end inclusive).
#[derive(Debug, Clone)]
pub struct AwbPool {
    pub id: String,
    pub carrier_party_id: String,
    pub prefix: String,
    pub start_n: i64,
    pub end_n: i64,
    pub next_n: i64,
    pub label: String,
}

impl AwbPool {
    pub fn remaining(&self) -> i64 {
        (self.end_n - self.next_n + 1).max(0)
    }

    pub fn exhausted(&self) -> bool {
        self.next_n > self.end_n
    }
}

/// One unpaid account booking / open AR bill for the chase list.
#[derive(Debug, Clone)]
pub struct PendingBillRow {
    pub sale_id: String,
    pub invoice_no: String,
    pub awb_no: String,
    pub party_id: String,
    pub party_name: String,
    pub open_paise: i64,
    pub created_at: String,
    pub age_days: i64,
}

impl Store {
    pub(crate) fn migrate_local_extras_tables(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS awb_pools (
                    id TEXT PRIMARY KEY,
                    carrier_party_id TEXT NOT NULL DEFAULT '',
                    prefix TEXT NOT NULL DEFAULT '',
                    start_n INTEGER NOT NULL,
                    end_n INTEGER NOT NULL,
                    next_n INTEGER NOT NULL,
                    label TEXT NOT NULL DEFAULT ''
                );
                CREATE INDEX IF NOT EXISTS idx_awb_pools_carrier
                    ON awb_pools(carrier_party_id);
                "#,
            )
            .map_err(|e| e.to_string())?;
        let _ = self.conn.execute(
            "ALTER TABLE sales ADD COLUMN piece_count INTEGER NOT NULL DEFAULT 1",
            [],
        );
        Ok(())
    }

    // ----- Print brand kit (stamp + payment QR beside venue icon) -----

    pub fn receipt_stamp_key(&self) -> Result<Option<String>, String> {
        Ok(self
            .meta_get("receipt_stamp_key")?
            .filter(|s| !s.trim().is_empty()))
    }

    pub fn payment_qr_key(&self) -> Result<Option<String>, String> {
        Ok(self
            .meta_get("payment_qr_key")?
            .filter(|s| !s.trim().is_empty()))
    }

    pub fn set_receipt_stamp_key(&self, key: Option<&str>) -> Result<(), String> {
        match key.map(str::trim).filter(|s| !s.is_empty()) {
            Some(k) => self.meta_set("receipt_stamp_key", k),
            None => self.meta_set("receipt_stamp_key", ""),
        }
    }

    pub fn set_payment_qr_key(&self, key: Option<&str>) -> Result<(), String> {
        match key.map(str::trim).filter(|s| !s.is_empty()) {
            Some(k) => self.meta_set("payment_qr_key", k),
            None => self.meta_set("payment_qr_key", ""),
        }
    }

    pub fn import_receipt_stamp(&self, source: &Path) -> Result<String, String> {
        self.import_brand_face(source, "receipt_stamp.webp", "receipt_stamp_key")
    }

    pub fn import_payment_qr(&self, source: &Path) -> Result<String, String> {
        self.import_brand_face(source, "payment_qr.webp", "payment_qr_key")
    }

    fn import_brand_face(
        &self,
        source: &Path,
        file_name: &str,
        meta_key: &str,
    ) -> Result<String, String> {
        let dir = self.brand_images_dir()?;
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !media::is_image_ext(&ext) {
            return Err("Use jpg, jpeg, png, or webp (stored as WebP)".into());
        }
        let dest = dir.join(file_name);
        media::write_image_as_webp(source, &dest)?;
        let key = format!("custom:{file_name}");
        self.meta_set(meta_key, &key)?;
        Ok(key)
    }

    pub fn brand_image_path(&self, key: &str) -> Option<PathBuf> {
        let name = key.strip_prefix("custom:")?;
        self.brand_images_dir().ok().map(|d| d.join(name))
    }

    /// Lines for receipt / period-bill text when stamp or pay QR files exist.
    pub fn print_brand_face_lines(&self) -> Result<String, String> {
        let mut out = String::new();
        if let Some(key) = self.receipt_stamp_key()? {
            if let Some(path) = self.brand_image_path(&key) {
                out.push_str(&format!("Stamp image: {}\n", path.display()));
            } else {
                out.push_str("[Stamp on file — print with picture job]\n");
            }
        }
        if let Some(key) = self.payment_qr_key()? {
            if let Some(path) = self.brand_image_path(&key) {
                out.push_str(&format!("Payment QR: {}\n", path.display()));
            } else {
                out.push_str("[Payment QR on file — scan to pay]\n");
            }
        }
        Ok(out)
    }

    // ----- Carrier AWB pools -----

    pub fn list_awb_pools(&self) -> Result<Vec<AwbPool>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, carrier_party_id, prefix, start_n, end_n, next_n, label
                 FROM awb_pools ORDER BY label, prefix, start_n",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(AwbPool {
                    id: r.get(0)?,
                    carrier_party_id: r.get(1)?,
                    prefix: r.get(2)?,
                    start_n: r.get(3)?,
                    end_n: r.get(4)?,
                    next_n: r.get(5)?,
                    label: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn awb_pools_status_text(&self) -> Result<String, String> {
        let pools = self.list_awb_pools()?;
        if pools.is_empty() {
            return Ok("No carrier AWB pools set.\n".into());
        }
        let mut out = String::from("Carrier AWB pools\n");
        for p in pools {
            let name = if p.carrier_party_id.is_empty() {
                "(no carrier)".into()
            } else {
                self.get_customer(&p.carrier_party_id)?
                    .map(|c| c.name)
                    .unwrap_or_else(|| p.carrier_party_id.clone())
            };
            let warn = if p.exhausted() {
                " · EXHAUSTED"
            } else if p.remaining() <= 20 {
                " · LOW"
            } else {
                ""
            };
            let label = if p.label.is_empty() {
                String::new()
            } else {
                format!(" ({})", p.label)
            };
            out.push_str(&format!(
                "{name}{label} · {}{}–{} · next {} · left {}{}\n",
                p.prefix,
                p.start_n,
                p.end_n,
                p.next_n,
                p.remaining(),
                warn
            ));
        }
        Ok(out)
    }

    pub fn add_awb_pool(
        &self,
        carrier_party_id: &str,
        prefix: &str,
        start_n: i64,
        end_n: i64,
        label: &str,
    ) -> Result<AwbPool, String> {
        self.assert_server_writer("Add AWB pool")?;
        if end_n < start_n {
            return Err("End number must be ≥ start".into());
        }
        if start_n < 0 {
            return Err("Start must be ≥ 0".into());
        }
        let id = new_id();
        let prefix = prefix.trim();
        let label = label.trim();
        self.conn
            .execute(
                "INSERT INTO awb_pools(
                    id, carrier_party_id, prefix, start_n, end_n, next_n, label
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?4, ?6)",
                params![
                    id,
                    carrier_party_id.trim(),
                    prefix,
                    start_n,
                    end_n,
                    label
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(AwbPool {
            id,
            carrier_party_id: carrier_party_id.trim().into(),
            prefix: prefix.into(),
            start_n,
            end_n,
            next_n: start_n,
            label: label.into(),
        })
    }

    pub fn delete_awb_pool(&self, pool_id: &str) -> Result<(), String> {
        self.assert_server_writer("Delete AWB pool")?;
        self.conn
            .execute("DELETE FROM awb_pools WHERE id = ?1", params![pool_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Next number from a carrier's first non-exhausted pool, or shop-wide seq.
    pub fn allocate_awb_for_carrier(
        &self,
        carrier_party_id: &str,
    ) -> Result<(String, Option<String>), String> {
        self.assert_server_writer("Allocate AWB")?;
        let carrier = carrier_party_id.trim();
        if !carrier.is_empty() {
            let pool = self
                .conn
                .query_row(
                    "SELECT id, prefix, next_n, end_n FROM awb_pools
                     WHERE carrier_party_id = ?1 AND next_n <= end_n
                     ORDER BY next_n ASC LIMIT 1",
                    params![carrier],
                    |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, i64>(2)?,
                            r.get::<_, i64>(3)?,
                        ))
                    },
                )
                .optional()
                .map_err(|e| e.to_string())?;
            if let Some((id, prefix, next_n, end_n)) = pool {
                self.conn
                    .execute(
                        "UPDATE awb_pools SET next_n = ?1 WHERE id = ?2",
                        params![next_n + 1, id],
                    )
                    .map_err(|e| e.to_string())?;
                let awb = if prefix.is_empty() {
                    format!("{next_n}")
                } else {
                    format!("{prefix}{next_n}")
                };
                let warn = if end_n - next_n < 20 {
                    Some(format!(
                        "AWB pool low: {} left after this",
                        end_n - next_n
                    ))
                } else {
                    None
                };
                return Ok((awb, warn));
            }
            let any: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM awb_pools WHERE carrier_party_id = ?1",
                    params![carrier],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if any > 0 {
                return Err("Carrier AWB pool exhausted — add a new number range".into());
            }
        }
        let awb = super::reserve_awb_no(&self.conn)?;
        Ok((awb, None))
    }

    // ----- Bulk tracking -----

    pub fn parse_awb_list(raw: &str) -> Vec<String> {
        raw.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
            .map(|s| crate::courier_ops_view::normalize_awb_input(s))
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Advance many AWBs to one status. Returns (ok_count, error lines).
    pub fn bulk_advance_tracking(
        &self,
        awbs: &[String],
        status: TrackingStatus,
        note: &str,
        actor: &StaffMember,
    ) -> Result<(usize, Vec<String>), String> {
        self.assert_server_writer("Bulk tracking")?;
        let mut ok = 0usize;
        let mut errs = Vec::new();
        for raw in awbs {
            let awb = raw.trim();
            if awb.is_empty() {
                continue;
            }
            match self.find_sale_id_by_awb(awb) {
                Ok(Some(sale_id)) => {
                    match self.advance_tracking(&sale_id, status, note, "", actor) {
                        Ok(_) => ok += 1,
                        Err(e) => errs.push(format!("{awb}: {e}")),
                    }
                }
                Ok(None) => errs.push(format!("{awb}: not found")),
                Err(e) => errs.push(format!("{awb}: {e}")),
            }
        }
        if ok == 0 && errs.is_empty() {
            return Err("Paste or type one AWB per line".into());
        }
        Ok((ok, errs))
    }

    pub fn find_sale_id_by_awb(&self, awb: &str) -> Result<Option<String>, String> {
        let awb = awb.trim();
        if awb.is_empty() {
            return Ok(None);
        }
        self.conn
            .query_row(
                "SELECT id FROM sales WHERE awb_no = ?1 LIMIT 1",
                params![awb],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    // ----- Settlement extras (TDS / discount / cheque in detail) -----

    pub fn format_settle_detail(
        pay: PayMethod,
        note: &str,
        tds_paise: i64,
        discount_paise: i64,
        cheque_no: &str,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();
        let note = note.trim();
        if !note.is_empty() {
            parts.push(note.into());
        } else {
            parts.push(pay.as_str().into());
        }
        if tds_paise > 0 {
            parts.push(format!("TDS −{}", csv_money(tds_paise)));
        }
        if discount_paise > 0 {
            parts.push(format!("Discount −{}", csv_money(discount_paise)));
        }
        let chq = cheque_no.trim();
        if !chq.is_empty() {
            parts.push(format!("Cheque {chq}"));
        }
        parts.join(" · ")
    }

    /// Cash/UPI/card received + TDS + discount clear AR together.
    pub fn settle_clear_paise(received_paise: i64, tds_paise: i64, discount_paise: i64) -> i64 {
        received_paise
            .max(0)
            .saturating_add(tds_paise.max(0))
            .saturating_add(discount_paise.max(0))
    }

    // ----- Bill chase list -----

    pub fn pending_bills_chase(&self) -> Result<Vec<PendingBillRow>, String> {
        let parties = self.list_customers()?;
        let mut out = Vec::new();
        let today = Local::now().date_naive();
        for p in parties {
            let bills = self.open_credit_bills(&p.id)?;
            for b in bills {
                let age = NaiveDate::parse_from_str(b.created_at.get(..10).unwrap_or(""), "%Y-%m-%d")
                    .ok()
                    .map(|d| (today - d).num_days())
                    .unwrap_or(0);
                let (invoice_no, awb_no) = match self.get_sale(&b.bill_id)? {
                    Some(s) => (s.invoice_no, s.awb_no),
                    None => (String::new(), String::new()),
                };
                out.push(PendingBillRow {
                    sale_id: b.bill_id,
                    invoice_no,
                    awb_no,
                    party_id: p.id.clone(),
                    party_name: p.name.clone(),
                    open_paise: b.open_paise,
                    created_at: b.created_at,
                    age_days: age,
                });
            }
        }
        out.sort_by(|a, b| b.age_days.cmp(&a.age_days).then(b.open_paise.cmp(&a.open_paise)));
        Ok(out)
    }

    pub fn pending_bills_chase_text(&self) -> Result<String, String> {
        let rows = self.pending_bills_chase()?;
        let mut out = String::from("Unpaid invoices — chase list\n");
        out.push_str("Open → Parties (Pay selected) or Period bill for unbilled.\n\n");
        if rows.is_empty() {
            out.push_str("(none)\n");
            return Ok(out);
        }
        for r in rows {
            let inv = if r.invoice_no.trim().is_empty() {
                r.sale_id.chars().take(8).collect::<String>()
            } else {
                r.invoice_no.clone()
            };
            let awb = if r.awb_no.trim().is_empty() {
                "—".into()
            } else {
                r.awb_no.clone()
            };
            out.push_str(&format!(
                "{} · {} · AWB {} · open {} · {}d · open → Pay selected\n",
                r.party_name,
                inv,
                awb,
                csv_money(r.open_paise),
                r.age_days
            ));
        }
        Ok(out)
    }

    // ----- Month GST summary CSV (not GSTR filing) -----

    pub fn month_gst_summary_csv(&self, year_month: &str) -> Result<String, String> {
        let ym = year_month.trim();
        if ym.len() < 7 {
            return Err("Use YYYY-MM".into());
        }
        let prefix = format!("{}%", &ym[..7]);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COALESCE(invoice_no,''), COALESCE(awb_no,''), created_at,
                        net_paise, tax_paise, total_paise, COALESCE(tax_label,'GST'),
                        COALESCE(money_lane,'')
                 FROM sales
                 WHERE status = 'completed' AND created_at LIKE ?1
                 ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;
        let mut out = String::from(
            "section,invoice_no,awb_no,date,net,tax,total,tax_label,bill_type\n",
        );
        let mut tax_sum = 0i64;
        let mut net_sum = 0i64;
        let rows = stmt
            .query_map(params![prefix], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (inv, awb, at, net, tax, total, label, lane) = row.map_err(|e| e.to_string())?;
            net_sum += net;
            tax_sum += tax;
            out.push_str(&format!(
                "output,{},{},{},{},{},{},{},{}\n",
                csv_cell(&inv),
                csv_cell(&awb),
                csv_cell(at.get(..10).unwrap_or(&at)),
                csv_money(net),
                csv_money(tax),
                csv_money(total),
                csv_cell(&label),
                csv_cell(&lane)
            ));
        }
        let (purchase_total, purchase_n): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(total_paise),0), COUNT(*)
                 FROM purchases WHERE created_at LIKE ?1",
                params![prefix],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((0, 0));
        let (expense_total, expense_n): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount_paise),0), COUNT(*)
                 FROM expenses WHERE created_at LIKE ?1",
                params![prefix],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((0, 0));
        out.push_str(&format!(
            "summary,OUTPUT_TAX_TOTAL,,,{},{},,,,{purchase_n} purchases / {expense_n} expenses\n",
            csv_money(net_sum),
            csv_money(tax_sum),
        ));
        out.push_str(&format!(
            "input_style,purchases_spend,,,{},,,,{purchase_n},ITC from carrier bills — attach GST invoices for CA\n",
            csv_money(purchase_total),
        ));
        out.push_str(&format!(
            "input_style,expenses_spend,,,{},,,,{expense_n},running costs — ITC if billed with GST\n",
            csv_money(expense_total),
        ));
        out.push_str(
            "# Local sheet for your CA — Moklanjo does not file GSTR or e-invoice.\n",
        );
        Ok(out)
    }

    // ----- Month mix: courier / service + top clients -----

    pub fn month_mix_report_text(&self, year_month: &str) -> Result<String, String> {
        let ym = year_month.trim();
        if ym.len() < 7 {
            return Err("Use YYYY-MM".into());
        }
        let prefix = format!("{}%", &ym[..7]);
        let mut out = format!("Month mix — {}\n\n", &ym[..7]);

        out.push_str("By carrier\n");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COALESCE(NULLIF(carrier_party_id,''), '(own / unassigned)'),
                        COUNT(*), COALESCE(SUM(total_paise),0)
                 FROM sales WHERE status = 'completed' AND created_at LIKE ?1
                 GROUP BY 1 ORDER BY 3 DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![prefix], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (cid, n, sum) = row.map_err(|e| e.to_string())?;
            let name = if cid.starts_with('(') {
                cid
            } else {
                self.get_customer(&cid)?
                    .map(|c| c.name)
                    .unwrap_or(cid)
            };
            out.push_str(&format!("  {name}: {n} bookings · {}\n", csv_money(sum)));
        }

        out.push_str("\nBy service (Document · Surface · Air)\n");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, COALESCE(s.contents,''),
                        (SELECT name FROM sale_lines WHERE sale_id = s.id LIMIT 1),
                        s.total_paise
                 FROM sales s
                 WHERE s.status = 'completed' AND s.created_at LIKE ?1",
            )
            .map_err(|e| e.to_string())?;
        let mut by_svc: std::collections::HashMap<String, (i64, i64)> =
            std::collections::HashMap::new();
        let rows = stmt
            .query_map(params![prefix], |r| {
                Ok((
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (contents, rate, total) = row.map_err(|e| e.to_string())?;
            let key = classify_service_mix(rate.as_deref().unwrap_or(""), &contents);
            let e = by_svc.entry(key).or_insert((0, 0));
            e.0 += 1;
            e.1 += total;
        }
        let mut svcs: Vec<_> = by_svc.into_iter().collect();
        svcs.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
        for (name, (n, sum)) in svcs {
            out.push_str(&format!("  {name}: {n} · {}\n", csv_money(sum)));
        }

        out.push_str("\nTop consignors\n");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COALESCE(customer_id,''), COUNT(*), COALESCE(SUM(total_paise),0)
                 FROM sales WHERE status = 'completed' AND created_at LIKE ?1
                   AND customer_id IS NOT NULL AND customer_id != ''
                 GROUP BY 1 ORDER BY 3 DESC LIMIT 15",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![prefix], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (cid, n, sum) = row.map_err(|e| e.to_string())?;
            let name = self
                .get_customer(&cid)?
                .map(|c| c.name)
                .unwrap_or_else(|| cid.clone());
            out.push_str(&format!("  {name}: {n} · {}\n", csv_money(sum)));
        }
        Ok(out)
    }

    pub fn month_mix_report_csv(&self, year_month: &str) -> Result<String, String> {
        let text = self.month_mix_report_text(year_month)?;
        let mut out = String::from("line\n");
        for line in text.lines() {
            out.push_str(&csv_cell(line));
            out.push('\n');
        }
        Ok(out)
    }
}

fn classify_service_mix(rate_name: &str, contents: &str) -> String {
    let s = format!("{rate_name} {contents}").to_ascii_lowercase();
    if s.contains("air")
        || s.contains("express")
        || s.contains("intl")
        || s.contains("international")
    {
        "Air".into()
    } else if s.contains("doc") || s.contains("document") || s.contains("letter") {
        "Document".into()
    } else if s.contains("surface") || s.contains("road") || s.contains("ground") {
        "Surface".into()
    } else if s.contains("cargo") || s.contains("heavy") {
        "Cargo".into()
    } else if contents.trim().is_empty() && rate_name.trim().is_empty() {
        "(unspecified)".into()
    } else if !contents.trim().is_empty() {
        contents.trim().to_string()
    } else {
        rate_name.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("moklanjo-extras-{n}.sqlite3"))
    }

    #[test]
    fn awb_pool_allocates_and_exhausts() {
        let store = Store::open_path(&temp_db()).unwrap();
        store
            .add_awb_pool("carrier1", "DT", 100, 101, "DTDC pad")
            .unwrap();
        let (a, _) = store.allocate_awb_for_carrier("carrier1").unwrap();
        assert_eq!(a, "DT100");
        let (b, warn) = store.allocate_awb_for_carrier("carrier1").unwrap();
        assert_eq!(b, "DT101");
        assert!(warn.is_some());
        assert!(store.allocate_awb_for_carrier("carrier1").is_err());
    }

    #[test]
    fn settle_detail_carries_tds_cheque() {
        let d = Store::format_settle_detail(PayMethod::Cash, "partial", 5000, 1000, "CHQ-9");
        assert!(d.contains("TDS"));
        assert!(d.contains("Cheque CHQ-9"));
        assert!(d.contains("Discount"));
        assert_eq!(Store::settle_clear_paise(10_000, 100, 50), 10_150);
    }

    #[test]
    fn brand_stamp_import_sets_meta() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        store
            .set_receipt_stamp_key(Some("custom:receipt_stamp.webp"))
            .unwrap();
        store
            .set_payment_qr_key(Some("custom:payment_qr.webp"))
            .unwrap();
        assert!(store.receipt_stamp_key().unwrap().is_some());
        let faces = store.print_brand_face_lines().unwrap();
        assert!(faces.contains("Stamp"));
        assert!(faces.contains("QR") || faces.contains("Payment"));
    }

    #[test]
    fn month_gst_csv_has_header() {
        let store = Store::open_path(&temp_db()).unwrap();
        let ym = chrono::Local::now().format("%Y-%m").to_string();
        let csv = store.month_gst_summary_csv(&ym).unwrap();
        assert!(csv.contains("section,"));
        assert!(csv.contains("Local sheet for your CA"));
        assert!(csv.contains("input_style"));
    }

    #[test]
    fn bulk_advance_and_chase_smoke() {
        use crate::domain::{Cart, Money, MoneyLane, PayMethod, Product};
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store
            .upsert_product(&Product::new_basic(new_id(), "Surface", Money::from_rupees(50)))
            .unwrap();
        let p = store.list_products(false).unwrap().remove(0);
        let party = store
            .add_customer_kind("Chase Co", "", "Consignor")
            .unwrap();
        let mut awbs = Vec::new();
        for _ in 0..2 {
            let mut cart = Cart::default();
            cart.add_or_bump(&p);
            let mut draft = cart.to_sale_draft().unwrap();
            draft.destination = "Pune".into();
            draft.actual_weight_kg = "1".parse().unwrap();
            draft.chargeable_weight_kg = "1".parse().unwrap();
            draft.money_lane = MoneyLane::Owner;
            let sale = store
                .finalize_sale(&draft, PayMethod::Cash, None, &owner)
                .unwrap();
            awbs.push(sale.awb_no);
        }
        let (ok, errs) = store
            .bulk_advance_tracking(
                &awbs,
                TrackingStatus::OutForDelivery,
                "batch",
                &owner,
            )
            .unwrap();
        assert_eq!(ok, 2);
        assert!(errs.is_empty());

        let mut cart = Cart::default();
        cart.add_or_bump(&p);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.destination = "Nashik".into();
        draft.actual_weight_kg = "1".parse().unwrap();
        draft.chargeable_weight_kg = "1".parse().unwrap();
        draft.money_lane = MoneyLane::Declared;
        let _ = store
            .finalize_sale(&draft, PayMethod::Credit, Some(&party.id), &owner)
            .unwrap();
        let chase = store.pending_bills_chase_text().unwrap();
        assert!(chase.contains("Chase Co") || chase.contains("Unpaid"));
        let ym = chrono::Local::now().format("%Y-%m").to_string();
        let mix = store.month_mix_report_text(&ym).unwrap();
        assert!(mix.contains("By carrier"));
    }
}

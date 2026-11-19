//! Credit notes (CN) and debit notes (DN) — first-class AR/AP adjustments
//! beyond amend/void.

use super::*;
use crate::domain::{party_has_role, Customer, PARTY_ROLE_SUPPLIER};
use rusqlite::OptionalExtension;

/// One stored CN or DN document plus its books link.
#[derive(Debug, Clone)]
pub struct AdjustmentNote {
    pub id: String,
    pub kind: String,
    pub party_id: String,
    pub related_sale_id: String,
    pub amount_paise: i64,
    pub reason: String,
    pub invoice_no: String,
    pub created_at: String,
    pub actor_id: String,
    pub actor_name: String,
}

impl Store {
    fn reserve_adjustment_no(&self, meta_key: &str, prefix: &str) -> Result<String, String> {
        let seq: i64 = self
            .conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM meta WHERE key = ?1",
                params![meta_key],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or(1)
            .max(1);
        self.conn
            .execute(
                "INSERT INTO meta(key, value) VALUES(?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![meta_key, (seq + 1).to_string()],
            )
            .map_err(|e| e.to_string())?;
        Ok(format!("{prefix}-{seq:06}"))
    }

    fn party_uses_ar(&self, party: &Customer) -> bool {
        if party.is_consignor() {
            return true;
        }
        if party.is_carrier() || party_has_role(&party.kind_label, PARTY_ROLE_SUPPLIER) {
            return false;
        }
        true
    }

    fn lookup_party(&self, party_id: &str) -> Result<Customer, String> {
        self.list_customers()?
            .into_iter()
            .find(|p| p.id == party_id)
            .ok_or_else(|| "Party not found".to_string())
    }

    fn resolve_related_sale(
        &self,
        party_id: &str,
        related_sale_id: Option<&str>,
        awb_hint: Option<&str>,
    ) -> Result<String, String> {
        if let Some(id) = related_sale_id.map(str::trim).filter(|s| !s.is_empty()) {
            if self.get_sale(id)?.is_some() {
                return Ok(id.to_string());
            }
            return Err("Related sale id not found".into());
        }
        if let Some(awb) = awb_hint.map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(sale) = self.find_sale_by_awb(awb)? {
                if let Some(cid) = sale.customer_id.as_deref().filter(|s| !s.trim().is_empty()) {
                    if cid != party_id {
                        return Err("AWB belongs to another party".into());
                    }
                }
                return Ok(sale.id);
            }
            return Err("AWB not found on a booking".into());
        }
        Ok(String::new())
    }

    fn post_adjustment_ledger(
        &self,
        party_id: &str,
        party: &Customer,
        note_kind: &str,
        signed_amount: i64,
        related_sale_id: &str,
        detail: &str,
        actor: &StaffMember,
        times: &EventTimes,
    ) -> Result<String, String> {
        let ledger_id = new_id();
        let sale_link = if related_sale_id.is_empty() {
            None
        } else {
            Some(related_sale_id)
        };
        if self.party_uses_ar(party) {
            self.conn
                .execute(
                    "INSERT INTO credit_ledger(
                        id, customer_id, created_at, kind, amount_paise,
                        sale_id, pay_method, actor_id, actor_name, detail,
                        user_at, device_at, server_at
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        ledger_id,
                        party_id,
                        times.user_at,
                        note_kind,
                        signed_amount,
                        sale_link,
                        actor.id,
                        actor.name,
                        detail,
                        times.user_at,
                        times.device_at,
                        times.server_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO payable_ledger(
                        id, party_id, created_at, kind, amount_paise, purchase_id,
                        pay_method, actor_id, actor_name, detail,
                        user_at, device_at, server_at
                     ) VALUES(?1, ?2, ?3, ?4, ?5, NULL, NULL, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        ledger_id,
                        party_id,
                        times.user_at,
                        note_kind,
                        signed_amount,
                        actor.id,
                        actor.name,
                        detail,
                        times.user_at,
                        times.device_at,
                        times.server_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(ledger_id)
    }

    fn create_adjustment_note_inner(
        &self,
        kind: &str,
        party_id: &str,
        amount_paise: i64,
        reason: &str,
        related_sale_id: Option<&str>,
        awb_hint: Option<&str>,
        actor: &StaffMember,
    ) -> Result<AdjustmentNote, String> {
        self.assert_server_writer("Adjustment note")?;
        if !actor.is_owner() && !actor.role.can_void_refund() {
            return Err("Only Owner/Manager can issue adjustment notes".into());
        }
        if amount_paise <= 0 {
            return Err("Amount must be greater than zero".into());
        }
        let reason = reason.trim();
        if reason.is_empty() {
            return Err("Reason is required".into());
        }
        let party = self.lookup_party(party_id)?;
        let related = self.resolve_related_sale(party_id, related_sale_id, awb_hint)?;
        let invoice_no = if kind == "credit_note" {
            self.reserve_adjustment_no("next_cn_seq", "CN")?
        } else {
            self.reserve_adjustment_no("next_dn_seq", "DN")?
        };
        let signed = if kind == "credit_note" {
            -amount_paise
        } else {
            amount_paise
        };
        let times = EventTimes::capture(None);
        let awb_part = if related.is_empty() {
            String::new()
        } else if let Ok(Some(sale)) = self.get_sale(&related) {
            let awb = sale.awb_no.trim();
            if awb.is_empty() {
                String::new()
            } else {
                format!(" AWB {awb}")
            }
        } else {
            String::new()
        };
        let detail = format!("{invoice_no}: {reason}{awb_part}");
        let ledger_id = self.post_adjustment_ledger(
            party_id,
            &party,
            kind,
            signed,
            &related,
            &detail,
            actor,
            &times,
        )?;
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO adjustment_notes(
                    id, kind, party_id, related_sale_id, amount_paise, reason,
                    invoice_no, created_at, user_at, device_at, server_at,
                    actor_id, actor_name, ledger_id
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    id,
                    kind,
                    party_id,
                    related,
                    amount_paise,
                    reason,
                    invoice_no,
                    times.user_at,
                    times.user_at,
                    times.device_at,
                    times.server_at,
                    actor.id,
                    actor.name,
                    ledger_id
                ],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            &format!("adjustment.{kind}"),
            &format!("{party_id} {amount_paise} {invoice_no}"),
        )?;
        Ok(AdjustmentNote {
            id,
            kind: kind.to_string(),
            party_id: party_id.to_string(),
            related_sale_id: related,
            amount_paise,
            reason: reason.to_string(),
            invoice_no,
            created_at: times.user_at,
            actor_id: actor.id.clone(),
            actor_name: actor.name.clone(),
        })
    }

    pub fn create_credit_note(
        &self,
        party_id: &str,
        amount_paise: i64,
        reason: &str,
        related_sale_id: Option<&str>,
        awb_hint: Option<&str>,
        actor: &StaffMember,
    ) -> Result<AdjustmentNote, String> {
        self.create_adjustment_note_inner(
            "credit_note",
            party_id,
            amount_paise,
            reason,
            related_sale_id,
            awb_hint,
            actor,
        )
    }

    pub fn create_debit_note(
        &self,
        party_id: &str,
        amount_paise: i64,
        reason: &str,
        related_sale_id: Option<&str>,
        awb_hint: Option<&str>,
        actor: &StaffMember,
    ) -> Result<AdjustmentNote, String> {
        self.create_adjustment_note_inner(
            "debit_note",
            party_id,
            amount_paise,
            reason,
            related_sale_id,
            awb_hint,
            actor,
        )
    }

    pub fn list_adjustment_notes(&self, limit: i64) -> Result<Vec<AdjustmentNote>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, party_id, related_sale_id, amount_paise, reason,
                        invoice_no, created_at, actor_id, actor_name
                 FROM adjustment_notes ORDER BY created_at DESC, rowid DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit.max(1)], |r| {
                Ok(AdjustmentNote {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    party_id: r.get(2)?,
                    related_sale_id: r.get(3)?,
                    amount_paise: r.get(4)?,
                    reason: r.get(5)?,
                    invoice_no: r.get(6)?,
                    created_at: r.get(7)?,
                    actor_id: r.get(8)?,
                    actor_name: r.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn list_adjustment_notes_for_party(
        &self,
        party_id: &str,
        limit: i64,
    ) -> Result<Vec<AdjustmentNote>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, party_id, related_sale_id, amount_paise, reason,
                        invoice_no, created_at, actor_id, actor_name
                 FROM adjustment_notes WHERE party_id = ?1
                 ORDER BY created_at DESC, rowid DESC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![party_id, limit.max(1)], |r| {
                Ok(AdjustmentNote {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    party_id: r.get(2)?,
                    related_sale_id: r.get(3)?,
                    amount_paise: r.get(4)?,
                    reason: r.get(5)?,
                    invoice_no: r.get(6)?,
                    created_at: r.get(7)?,
                    actor_id: r.get(8)?,
                    actor_name: r.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn adjustment_notes_text_for_party(&self, party_id: &str) -> Result<String, String> {
        let notes = self.list_adjustment_notes_for_party(party_id, 8)?;
        if notes.is_empty() {
            return Ok("(no adjustment notes yet)".into());
        }
        let mut out = String::new();
        for n in notes {
            let tag = if n.kind == "credit_note" {
                "CN"
            } else {
                "DN"
            };
            out.push_str(&format!(
                "{} · {} · {} · {}\n",
                n.invoice_no,
                tag,
                self.money_display(n.amount_paise),
                n.reason
            ));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Money, Product};

    fn temp_db() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("moklanjo_adj_{}.db", new_id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn credit_and_debit_note_ledger_effects() {
        let path = temp_db();
        let store = Store::open_path(&path).unwrap();
        let owner = store.complete_owner_setup("0000").unwrap();
        let party = store
            .add_customer_kind("Acme Sender", "", "Consignor")
            .unwrap();
        store.seed_opening_credit(&party.id, 10_000, &owner).unwrap();
        assert_eq!(store.customer_balance(&party.id).unwrap(), 10_000);

        let cn = store
            .create_credit_note(&party.id, 1_500, "rate rebate", None, None, &owner)
            .unwrap();
        assert!(cn.invoice_no.starts_with("CN-"));
        assert_eq!(store.customer_balance(&party.id).unwrap(), 8_500);

        let dn = store
            .create_debit_note(&party.id, 500, "ODA surcharge", None, None, &owner)
            .unwrap();
        assert!(dn.invoice_no.starts_with("DN-"));
        assert_eq!(store.customer_balance(&party.id).unwrap(), 9_000);

        let list = store.list_adjustment_notes(10).unwrap();
        assert_eq!(list.len(), 2);

        let carrier = store.add_customer_kind("BlueDart", "", "Carrier").unwrap();
        let mut p = Product::default();
        p.name = "Fuel line".into();
        p.price = Money(2_000);
        p.unit_label = "pc".into();
        store.upsert_product(&p).unwrap();
        let pid = store.list_products(false).unwrap()[0].id.clone();
        store
            .record_purchase_ex(
                "BlueDart",
                &carrier.id,
                true,
                &[(pid, 1_000, 2_000)],
                "",
                &owner,
            )
            .unwrap();
        assert_eq!(store.party_payable_balance(&carrier.id).unwrap(), 2_000);
        store
            .create_credit_note(&carrier.id, 200, "recon", None, None, &owner)
            .unwrap();
        assert_eq!(store.party_payable_balance(&carrier.id).unwrap(), 1_800);
        store
            .create_debit_note(&carrier.id, 100, "top-up", None, None, &owner)
            .unwrap();
        assert_eq!(store.party_payable_balance(&carrier.id).unwrap(), 1_900);

        let _ = std::fs::remove_file(path);
    }
}
//! Parked counter-desk drafts — cart snapshot, not books until finalize.

use super::*;
use chrono::Local;
use rusqlite::{OptionalExtension, params};

impl Store {
    fn counter_draft_from_row(
        id: String,
        status: String,
        opened_at: String,
        updated_at: String,
        opened_by: String,
        title: String,
        payload_json: String,
        sale_id: String,
    ) -> Result<CounterDraft, String> {
        let snapshot: CounterDeskSnapshot =
            serde_json::from_str(&payload_json).map_err(|e| format!("Bad counter draft: {e}"))?;
        Ok(CounterDraft {
            id,
            status,
            opened_at,
            updated_at,
            opened_by,
            title,
            snapshot,
            sale_id,
        })
    }

    fn write_counter_draft_row(&self, draft: &CounterDraft) -> Result<(), String> {
        let payload = serde_json::to_string(&draft.snapshot)
            .map_err(|e| format!("Serialize counter draft: {e}"))?;
        self.conn
            .execute(
                "INSERT INTO counter_drafts (
                    id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                    status = excluded.status,
                    updated_at = excluded.updated_at,
                    opened_by = excluded.opened_by,
                    title = excluded.title,
                    payload_json = excluded.payload_json,
                    sale_id = excluded.sale_id",
                params![
                    draft.id,
                    draft.status,
                    draft.opened_at,
                    draft.updated_at,
                    draft.opened_by,
                    draft.title,
                    payload,
                    draft.sale_id,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn upsert_counter_draft(&self, draft: &CounterDraft) -> Result<(), String> {
        self.assert_server_writer("Save counter draft")?;
        self.write_counter_draft_row(draft)
    }

    pub fn get_counter_draft(&self, id: &str) -> Result<Option<CounterDraft>, String> {
        self.conn
            .query_row(
                "SELECT id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id
                 FROM counter_drafts WHERE id = ?1",
                params![id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                        r.get::<_, String>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())?
            .map(
                |(id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id)| {
                    Self::counter_draft_from_row(
                        id,
                        status,
                        opened_at,
                        updated_at,
                        opened_by,
                        title,
                        payload_json,
                        sale_id,
                    )
                },
            )
            .transpose()
    }

    pub fn list_open_counter_drafts(&self) -> Result<Vec<CounterDraft>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id
                 FROM counter_drafts WHERE status = 'draft' ORDER BY updated_at DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let (id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id) =
                row.map_err(|e| e.to_string())?;
            out.push(Self::counter_draft_from_row(
                id,
                status,
                opened_at,
                updated_at,
                opened_by,
                title,
                payload_json,
                sale_id,
            )?);
        }
        Ok(out)
    }

    pub fn discard_counter_draft(&self, id: &str, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Discard counter draft")?;
        let now = Local::now().to_rfc3339();
        let n = self
            .conn
            .execute(
                "UPDATE counter_drafts SET status = 'discarded', updated_at = ?2
                 WHERE id = ?1 AND status = 'draft'",
                params![id, now],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("No open counter draft with that id".into());
        }
        let _ = actor;
        Ok(())
    }

    pub fn close_counter_draft_as_sale(&self, draft_id: &str, sale_id: &str) -> Result<(), String> {
        self.assert_server_writer("Finalize counter draft")?;
        if sale_id.trim().is_empty() {
            return Err("sale_id required".into());
        }
        let n = self.conn.execute(
            "DELETE FROM counter_drafts WHERE id = ?1 AND status = 'draft'",
            params![draft_id],
        ).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("No open counter draft with that id".into());
        }
        let _ = sale_id;
        Ok(())
    }

    pub fn park_counter_draft_from_snapshot(
        &self,
        id: Option<&str>,
        snap: CounterDeskSnapshot,
        actor: &StaffMember,
    ) -> Result<CounterDraft, String> {
        self.assert_server_writer("Park counter draft")?;
        if snap.cart.lines.is_empty() {
            return Err("Nothing to park — add items to the cart".into());
        }
        let now = Local::now().to_rfc3339();
        let title = snap.display_title();
        let opened_by = actor.id.clone();
        let (draft_id, opened_at) = if let Some(existing) = id {
            if let Some(prev) = self.get_counter_draft(existing)? {
                if prev.status != "draft" {
                    return Err("That counter draft is no longer open".into());
                }
                (prev.id, prev.opened_at)
            } else {
                (existing.to_string(), now.clone())
            }
        } else {
            (new_id(), now.clone())
        };
        let draft = CounterDraft {
            id: draft_id,
            status: "draft".into(),
            opened_at,
            updated_at: now,
            opened_by,
            title,
            snapshot: snap,
            sale_id: String::new(),
        };
        self.upsert_counter_draft(&draft)?;
        Ok(draft)
    }

    /// Client mirror: replace open drafts with the Server snapshot.
    pub fn replace_counter_drafts_from_sync(&self, drafts: &[CounterDraft]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM counter_drafts WHERE status = 'draft'", [])
            .map_err(|e| e.to_string())?;
        for draft in drafts {
            self.write_counter_draft_row(draft)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Cart, Product, Money};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        crate::test_dirs::run_root().join(format!("khaynjo-counter-draft-{n}.sqlite3"))
    }

    #[test]
    fn park_list_discard() {
        let store = Store::open_path(&temp_db()).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let mut snap = CounterDeskSnapshot {
            cart: Cart::default(),
            selected_customer_id: None,
            customer_label: String::new(),
        };
        assert!(store.park_counter_draft_from_snapshot(None, snap.clone(), &owner).is_err());
        let mut p = Product::default();
        p.name = "Chai".into();
        p.price = Money(2_000);
        p.available = true;
        store.upsert_product(&p).unwrap();
        let rate = store.list_products(false).unwrap().into_iter().next().unwrap();
        snap.cart.add_or_bump(&rate);
        let draft = store
            .park_counter_draft_from_snapshot(None, snap, &owner)
            .unwrap();
        assert_eq!(store.list_open_counter_drafts().unwrap().len(), 1);
        store.discard_counter_draft(&draft.id, &owner).unwrap();
        assert!(store.list_open_counter_drafts().unwrap().is_empty());
    }
}
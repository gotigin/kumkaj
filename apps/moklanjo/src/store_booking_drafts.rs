//! Parked booking-desk drafts — cart + form snapshot, not books until finalize.

use super::*;
use chrono::Local;
use rusqlite::{OptionalExtension, params};

impl Store {
    fn booking_draft_from_row(
        id: String,
        status: String,
        opened_at: String,
        updated_at: String,
        opened_by: String,
        title: String,
        payload_json: String,
        sale_id: String,
    ) -> Result<BookingDraft, String> {
        let snapshot: BookingDeskSnapshot =
            serde_json::from_str(&payload_json).map_err(|e| format!("Bad booking draft: {e}"))?;
        Ok(BookingDraft {
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

    fn write_booking_draft_row(&self, draft: &BookingDraft) -> Result<(), String> {
        let payload = serde_json::to_string(&draft.snapshot)
            .map_err(|e| format!("Serialize booking draft: {e}"))?;
        self.conn
            .execute(
                "INSERT INTO booking_drafts (
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

    pub fn upsert_booking_draft(&self, draft: &BookingDraft) -> Result<(), String> {
        self.assert_server_writer("Save booking draft")?;
        self.write_booking_draft_row(draft)
    }

    pub fn get_booking_draft(&self, id: &str) -> Result<Option<BookingDraft>, String> {
        self.conn
            .query_row(
                "SELECT id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id
                 FROM booking_drafts WHERE id = ?1",
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
            .map(|(id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id)| {
                Self::booking_draft_from_row(
                    id,
                    status,
                    opened_at,
                    updated_at,
                    opened_by,
                    title,
                    payload_json,
                    sale_id,
                )
            })
            .transpose()
    }

    pub fn list_open_booking_drafts(&self) -> Result<Vec<BookingDraft>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, status, opened_at, updated_at, opened_by, title, payload_json, sale_id
                 FROM booking_drafts WHERE status = 'draft' ORDER BY updated_at DESC",
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
            out.push(Self::booking_draft_from_row(
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

    pub fn discard_booking_draft(&self, id: &str, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Discard booking draft")?;
        let now = Local::now().to_rfc3339();
        let n = self
            .conn
            .execute(
                "UPDATE booking_drafts SET status = 'discarded', updated_at = ?2
                 WHERE id = ?1 AND status = 'draft'",
                params![id, now],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("No open booking draft with that id".into());
        }
        let _ = actor;
        Ok(())
    }

    pub fn discard_all_open_booking_drafts(&self) -> Result<usize, String> {
        self.assert_server_writer("Discard booking drafts")?;
        let now = Local::now().to_rfc3339();
        let n = self
            .conn
            .execute(
                "UPDATE booking_drafts SET status = 'discarded', updated_at = ?1
                 WHERE status = 'draft'",
                params![now],
            )
            .map_err(|e| e.to_string())?;
        Ok(n)
    }

    pub fn close_booking_draft_as_sale(&self, draft_id: &str, sale_id: &str) -> Result<(), String> {
        self.assert_server_writer("Finalize booking draft")?;
        if sale_id.trim().is_empty() {
            return Err("sale_id required".into());
        }
        let n = self
            .conn
            .execute("DELETE FROM booking_drafts WHERE id = ?1 AND status = 'draft'", params![draft_id])
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("No open booking draft with that id".into());
        }
        let _ = sale_id;
        Ok(())
    }

    pub fn park_booking_draft_from_snapshot(
        &self,
        id: Option<&str>,
        snap: BookingDeskSnapshot,
        actor: &StaffMember,
    ) -> Result<BookingDraft, String> {
        self.assert_server_writer("Park booking draft")?;
        if snap.is_empty() {
            return Err("Nothing to park — add charges or booking details".into());
        }
        let now = Local::now().to_rfc3339();
        let title = snap.display_title();
        let opened_by = actor.id.clone();
        let (draft_id, opened_at) = if let Some(existing) = id {
            if let Some(prev) = self.get_booking_draft(existing)? {
                if prev.status != "draft" {
                    return Err("That booking draft is no longer open".into());
                }
                (prev.id, prev.opened_at)
            } else {
                (existing.to_string(), now.clone())
            }
        } else {
            (new_id(), now.clone())
        };
        let draft = BookingDraft {
            id: draft_id,
            status: "draft".into(),
            opened_at,
            updated_at: now,
            opened_by,
            title,
            snapshot: snap,
            sale_id: String::new(),
        };
        self.upsert_booking_draft(&draft)?;
        Ok(draft)
    }

    /// Client mirror: replace open drafts with the Server snapshot.
    pub fn replace_booking_drafts_from_sync(&self, drafts: &[BookingDraft]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM booking_drafts WHERE status = 'draft'", [])
            .map_err(|e| e.to_string())?;
        for draft in drafts {
            self.write_booking_draft_row(draft)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Cart, Money, PayMethod, Product};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        crate::test_dirs::run_root().join(format!("moklanjo-booking-draft-{n}.sqlite3"))
    }

    #[test]
    fn park_empty_fails() {
        let store = Store::open_path(&temp_db()).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let snap = BookingDeskSnapshot {
            cart: Cart::default(),
            selected_customer_id: None,
            booking_scope_international: false,
            origin_name: String::new(),
            origin_phone: String::new(),
            origin_address: String::new(),
            destination: String::new(),
            consignee: String::new(),
            consignee_phone: String::new(),
            consignee_address: String::new(),
            actual_kg: String::new(),
            length_cm: String::new(),
            breadth_cm: String::new(),
            height_cm: String::new(),
            cod: String::new(),
            carrier_cost: String::new(),
            declared: String::new(),
            contents: String::new(),
            eway: String::new(),
            insurance: false,
            notify_phone: String::new(),
            fuel: String::new(),
            docket: String::new(),
            zone: String::new(),
            pieces: String::new(),
            carrier_pick: String::new(),
        };
        assert!(store.park_booking_draft_from_snapshot(None, snap, &owner).is_err());
    }

    #[test]
    fn park_list_get_discard() {
        let store = Store::open_path(&temp_db()).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let mut snap = BookingDeskSnapshot {
            cart: Cart::default(),
            selected_customer_id: None,
            booking_scope_international: false,
            origin_name: "Acme Sender".into(),
            origin_phone: String::new(),
            origin_address: String::new(),
            destination: "Mumbai".into(),
            consignee: String::new(),
            consignee_phone: String::new(),
            consignee_address: String::new(),
            actual_kg: String::new(),
            length_cm: String::new(),
            breadth_cm: String::new(),
            height_cm: String::new(),
            cod: String::new(),
            carrier_cost: String::new(),
            declared: String::new(),
            contents: String::new(),
            eway: String::new(),
            insurance: false,
            notify_phone: String::new(),
            fuel: String::new(),
            docket: String::new(),
            zone: String::new(),
            pieces: String::new(),
            carrier_pick: String::new(),
        };
        let draft = store
            .park_booking_draft_from_snapshot(None, snap.clone(), &owner)
            .unwrap();
        assert_eq!(draft.title, "Acme Sender");
        let listed = store.list_open_booking_drafts().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, draft.id);
        let got = store.get_booking_draft(&draft.id).unwrap().unwrap();
        assert_eq!(got.snapshot.destination, "Mumbai");
        snap.destination = "Pune".into();
        let updated = store
            .park_booking_draft_from_snapshot(Some(&draft.id), snap, &owner)
            .unwrap();
        assert_eq!(updated.id, draft.id);
        assert_eq!(updated.snapshot.destination, "Pune");
        store.discard_booking_draft(&draft.id, &owner).unwrap();
        assert!(store.list_open_booking_drafts().unwrap().is_empty());
    }

    #[test]
    fn list_sales_in_business_day_respects_window() {
        let store = Store::open_path(&temp_db()).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        assert!(store.list_sales_in_business_day(50).unwrap().is_empty());
        let mut p = Product::default();
        p.name = "Parcel 1kg".into();
        p.price = Money(10_000);
        p.available = true;
        store.upsert_product(&p).unwrap();
        let rate = store.list_products(false).unwrap().into_iter().next().unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&rate);
        let draft = cart.to_sale_draft().unwrap();
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, &owner)
            .unwrap();
        let today = store.list_sales_in_business_day(50).unwrap();
        assert_eq!(today.len(), 1);
        assert_eq!(today[0].id, sale.id);
    }

    #[test]
    fn close_as_sale_removes_from_open_list() {
        let store = Store::open_path(&temp_db()).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let snap = BookingDeskSnapshot {
            cart: Cart::default(),
            selected_customer_id: None,
            booking_scope_international: false,
            origin_name: "Shipper".into(),
            origin_phone: String::new(),
            origin_address: String::new(),
            destination: String::new(),
            consignee: String::new(),
            consignee_phone: String::new(),
            consignee_address: String::new(),
            actual_kg: String::new(),
            length_cm: String::new(),
            breadth_cm: String::new(),
            height_cm: String::new(),
            cod: String::new(),
            carrier_cost: String::new(),
            declared: String::new(),
            contents: String::new(),
            eway: String::new(),
            insurance: false,
            notify_phone: String::new(),
            fuel: String::new(),
            docket: String::new(),
            zone: String::new(),
            pieces: String::new(),
            carrier_pick: String::new(),
        };
        let draft = store
            .park_booking_draft_from_snapshot(None, snap, &owner)
            .unwrap();
        store
            .close_booking_draft_as_sale(&draft.id, "sale-test-1")
            .unwrap();
        assert!(store.list_open_booking_drafts().unwrap().is_empty());
        assert!(store.get_booking_draft(&draft.id).unwrap().is_none());
    }
}
//! When to buy more: a **reorder level** per item says when stock is low
//! (default [`DEFAULT_REORDER_AT`]); low items show on the Day book, Buy and
//! Inventory.

use super::Store;
use crate::domain::{Product, StaffMember};
use rusqlite::{params, OptionalExtension};

/// Low-stock line for items with no reorder level of their own.
pub const DEFAULT_REORDER_AT: i64 = 5;

/// A stock item at or below its reorder level.
#[derive(Clone, Debug)]
pub struct LowStock {
    pub product: Product,
    pub reorder_at: i64,
}

impl Store {
    /// Set when an item counts as low (whole units).
    pub fn set_reorder_level(
        &self,
        product_id: &str,
        reorder_at: i64,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change reorder levels")?;
        if reorder_at < 0 {
            return Err("Reorder level can't be negative".into());
        }
        self.conn
            .execute(
                "INSERT INTO stock_levels(product_id, reorder_at) VALUES(?1, ?2)
                 ON CONFLICT(product_id) DO UPDATE SET reorder_at = excluded.reorder_at",
                params![product_id, reorder_at],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, "stock.reorder_level", &format!("{product_id} {reorder_at}"))?;
        Ok(())
    }

    /// The item's reorder level, or [`DEFAULT_REORDER_AT`].
    pub fn reorder_level(&self, product_id: &str) -> Result<i64, String> {
        Ok(self
            .conn
            .query_row(
                "SELECT reorder_at FROM stock_levels WHERE product_id = ?1",
                params![product_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or(DEFAULT_REORDER_AT))
    }

    /// Every reorder level the owner set, by item id.
    pub fn reorder_levels(&self) -> Result<std::collections::HashMap<String, i64>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT product_id, reorder_at FROM stock_levels")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Counted items at or below their reorder level, emptiest first.
    pub fn low_stock(&self) -> Result<Vec<LowStock>, String> {
        let mut out = Vec::new();
        for product in self.list_products(false)? {
            if !product.tracks_stock() {
                continue;
            }
            let reorder_at = self.reorder_level(&product.id)?;
            if product.stock_qty <= reorder_at {
                out.push(LowStock { product, reorder_at });
            }
        }
        out.sort_by_key(|l| (l.product.stock_qty - l.reorder_at, l.product.name.to_lowercase()));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::new_id;

    #[test]
    fn each_item_has_its_own_reorder_level() {
        let path = crate::test_dirs::run_root().join(format!("tabiyat-reorder-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        for name in ["Rice", "Sugar"] {
            store.upsert_product(&Product::new_resale_stock(name, "kg")).unwrap();
        }
        for p in store.list_products(false).unwrap() {
            store.adjust_stock(&p.id, 2, "count", "", &owner).unwrap();
        }
        let rice = store.list_products(false).unwrap().into_iter().find(|p| p.name == "Rice").unwrap();
        // Both items hold 2, under the default of 5.
        assert_eq!(store.low_stock().unwrap().len(), 2);
        store.set_reorder_level(&rice.id, 1, &owner).unwrap();
        let low: Vec<_> = store.low_stock().unwrap().into_iter().map(|l| l.product.name).collect();
        assert_eq!(low, vec!["Sugar".to_string()]);
        assert_eq!(store.reorder_level(&rice.id).unwrap(), 1);
        let _ = std::fs::remove_file(path);
    }
}

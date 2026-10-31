//! What a dish uses up, and when to buy more.
//!
//! A **recipe** says how much of each stock item (raw material or resold
//! good) one portion of a dish uses: "Rice=0.2, Paneer=0.15" in each item's
//! own unit. Selling the dish takes that off the shelf; voiding or refunding
//! puts it back; amending moves the difference. Stock is counted in whole
//! units (5 kg rice = 5), so the part of a unit already used is carried per
//! item (`stock_carry`) and a whole unit comes off each time it adds up to
//! one. The kitchen's reality wins over the books: usage never refuses a
//! sale — a shelf that runs dry stops at 0 and the movement says how much
//! was short, so someone counts it.
//!
//! A **reorder level** per item says when stock is low (default
//! [`DEFAULT_REORDER_AT`]); low items show on the Day book, Buy and
//! Inventory.

use super::Store;
use crate::domain::{new_id, Product, StaffMember, QTY_MILLI_UNIT};
use rusqlite::{params, Connection, OptionalExtension};

/// Low-stock line for items with no reorder level of their own.
pub const DEFAULT_REORDER_AT: i64 = 5;

/// One ingredient of a dish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeLine {
    pub item_id: String,
    pub name: String,
    pub unit_label: String,
    /// Per portion, in thousandths of the item's unit (0.2 kg = 200).
    pub qty_milli: i64,
}

/// A stock item at or below its reorder level.
#[derive(Clone, Debug)]
pub struct LowStock {
    pub product: Product,
    pub reorder_at: i64,
}

pub(super) const SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS recipe_lines (
        dish_id TEXT NOT NULL,
        item_id TEXT NOT NULL,
        qty_milli INTEGER NOT NULL,
        PRIMARY KEY (dish_id, item_id)
    );
    CREATE TABLE IF NOT EXISTS stock_carry (
        product_id TEXT PRIMARY KEY,
        used_milli INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS stock_levels (
        product_id TEXT PRIMARY KEY,
        reorder_at INTEGER NOT NULL
    );
"#;

/// "0.2" / "1" / "0.125" → thousandths. Refuses negatives and junk.
fn parse_milli(text: &str) -> Result<i64, String> {
    let t = text.trim().replace(',', ".");
    let value: f64 = t
        .parse()
        .map_err(|_| format!("\"{}\" is not a quantity", text.trim()))?;
    if !value.is_finite() || value <= 0.0 {
        return Err(format!("\"{}\" must be more than 0", text.trim()));
    }
    Ok((value * QTY_MILLI_UNIT as f64).round() as i64)
}

/// Thousandths back to how the owner typed them: 200 → "0.2", 1000 → "1".
pub fn milli_as_typed(milli: i64) -> String {
    let whole = milli / QTY_MILLI_UNIT;
    let frac = milli % QTY_MILLI_UNIT;
    if frac == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{frac:03}").trim_end_matches('0').to_string()
    }
}

/// A recipe as the owner types it: "Rice=0.2, Paneer=0.15".
pub fn recipe_as_typed(lines: &[RecipeLine]) -> String {
    lines
        .iter()
        .map(|l| format!("{}={}", l.name, milli_as_typed(l.qty_milli)))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Store {
    /// The dish's recipe, in the order the items were named.
    pub fn recipe(&self, dish_id: &str) -> Result<Vec<RecipeLine>, String> {
        recipe_on(&self.conn, dish_id)
    }

    /// Replace a dish's recipe. `text` is "Item=qty, Item=qty" with each qty
    /// in that item's unit; every item must already be a stock item (bought
    /// on Buy or counted on Inventory). Blank clears the recipe.
    pub fn set_recipe_from_text(
        &self,
        dish_id: &str,
        text: &str,
        actor: &StaffMember,
    ) -> Result<Vec<RecipeLine>, String> {
        self.assert_server_writer("Change recipes")?;
        let items: Vec<Product> = self
            .list_products(false)?
            .into_iter()
            .filter(|p| p.id != dish_id && (p.is_bought() || p.tracks_stock()))
            .collect();
        let mut rows: Vec<(String, i64)> = Vec::new();
        for part in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let (name, qty) = part
                .rsplit_once('=')
                .ok_or_else(|| format!("Write \"{part}\" as Item=quantity, e.g. Rice=0.2"))?;
            let name = name.trim();
            let item = items
                .iter()
                .find(|p| p.matches_label(name))
                .ok_or_else(|| {
                    format!("No stock item named \"{name}\" — add it on Buy or Inventory first")
                })?;
            let milli = parse_milli(qty)?;
            match rows.iter_mut().find(|(id, _)| *id == item.id) {
                Some((_, q)) => *q += milli,
                None => rows.push((item.id.clone(), milli)),
            }
        }
        let tx = self.begin()?;
        tx.execute("DELETE FROM recipe_lines WHERE dish_id = ?1", params![dish_id])
            .map_err(|e| e.to_string())?;
        for (item_id, milli) in &rows {
            tx.execute(
                "INSERT INTO recipe_lines(dish_id, item_id, qty_milli) VALUES(?1, ?2, ?3)",
                params![dish_id, item_id, milli],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        self.audit(actor, "recipe.set", &format!("{dish_id} {}", text.trim()))?;
        self.recipe(dish_id)
    }

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

pub(super) fn recipe_on(conn: &Connection, dish_id: &str) -> Result<Vec<RecipeLine>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT r.item_id, COALESCE(p.name, ''), COALESCE(p.unit_label, ''), r.qty_milli
               FROM recipe_lines r LEFT JOIN products p ON p.id = r.item_id
              WHERE r.dish_id = ?1 ORDER BY r.rowid",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![dish_id], |r| {
            Ok(RecipeLine {
                item_id: r.get(0)?,
                name: r.get(1)?,
                unit_label: r.get(2)?,
                qty_milli: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Take what `portions` of each dish use off the shelf (`portions` < 0 puts
/// it back). `dishes` is (dish id, portions); `reason` is the movement's
/// reason ("recipe", "recipe_void", "recipe_amend").
pub(super) fn apply_recipe_usage(
    tx: &Connection,
    dishes: &[(String, i64)],
    reason: &str,
    ref_id: &str,
    branch_id: &str,
    actor: &StaffMember,
    at: &str,
) -> Result<(), String> {
    // Total thousandths per item over every dish on the bill.
    let mut usage: Vec<(String, i64)> = Vec::new();
    for (dish_id, portions) in dishes {
        if *portions == 0 {
            continue;
        }
        for line in recipe_on(tx, dish_id)? {
            let milli = line.qty_milli.saturating_mul(*portions);
            match usage.iter_mut().find(|(id, _)| *id == line.item_id) {
                Some((_, m)) => *m = m.saturating_add(milli),
                None => usage.push((line.item_id, milli)),
            }
        }
    }
    for (item_id, milli) in usage {
        if milli == 0 {
            continue;
        }
        let on_hand: Option<i64> = tx
            .query_row("SELECT stock_qty FROM products WHERE id = ?1", params![item_id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        // Untracked (−1) or gone from the catalogue: nothing to count.
        let Some(on_hand) = on_hand.filter(|q| *q >= 0) else {
            continue;
        };
        let carried: i64 = tx
            .query_row(
                "SELECT used_milli FROM stock_carry WHERE product_id = ?1",
                params![item_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or(0);
        let total = carried.saturating_add(milli);
        let whole = total.div_euclid(QTY_MILLI_UNIT);
        let carry = total.rem_euclid(QTY_MILLI_UNIT);
        let after = (on_hand - whole).max(0);
        let short = (whole - on_hand).max(0);
        tx.execute(
            "INSERT INTO stock_carry(product_id, used_milli) VALUES(?1, ?2)
             ON CONFLICT(product_id) DO UPDATE SET used_milli = excluded.used_milli",
            params![item_id, carry],
        )
        .map_err(|e| e.to_string())?;
        if after == on_hand {
            continue;
        }
        tx.execute(
            "UPDATE products SET stock_qty = ?1 WHERE id = ?2",
            params![after, item_id],
        )
        .map_err(|e| e.to_string())?;
        let note = if short > 0 {
            format!("shelf ran out — {short} more used than counted; count it")
        } else {
            String::new()
        };
        tx.execute(
            "INSERT INTO stock_movements(
                id, branch_id, product_id, created_at, delta_qty,
                reason, note, actor_id, actor_name, ref_id
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![new_id(), branch_id, item_id, at, after - on_hand, reason, note, actor.id, actor.name, ref_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Cart, Money, PayMethod, Product};

    fn kitchen() -> (Store, std::path::PathBuf, StaffMember, Product) {
        let path = std::env::temp_dir().join(format!("khaynjo-recipe-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.upsert_product(&Product::new_raw_material("Rice", "kg")).unwrap();
        store.upsert_product(&Product::new_raw_material("Paneer", "kg")).unwrap();
        let dish = Product::new_basic(new_id(), "Paneer Pulao", Money::from_rupees(180));
        store.upsert_product(&dish).unwrap();
        for p in store.list_products(false).unwrap().into_iter().filter(|p| p.is_raw()) {
            store.adjust_stock(&p.id, 2, "count", "", &owner).unwrap();
        }
        (store, path, owner, dish)
    }

    fn on_hand(store: &Store, name: &str) -> i64 {
        store.list_products(false).unwrap().into_iter().find(|p| p.name == name).unwrap().stock_qty
    }

    #[test]
    fn selling_a_dish_uses_its_recipe_and_a_void_puts_it_back() {
        let (store, path, owner, dish) = kitchen();
        let set = store.set_recipe_from_text(&dish.id, "rice=0.4, Paneer=0.25", &owner).unwrap();
        assert_eq!(recipe_as_typed(&set), "Rice=0.4, Paneer=0.25");
        assert!(store.set_recipe_from_text(&dish.id, "Saffron=0.01", &owner).is_err());
        assert!(store.set_recipe_from_text(&dish.id, "Rice=-1", &owner).is_err());

        let mut cart = Cart::default();
        cart.add_or_bump(&dish);
        cart.add_or_bump(&dish); // two portions: 0.8 kg rice, 0.5 kg paneer
        let draft = cart.to_sale_draft().unwrap();
        let first = store.finalize_sale(&draft, PayMethod::Cash, None, None, &owner).unwrap();
        // Not a whole kilo yet: still 2 on the shelf, the part is carried.
        assert_eq!(on_hand(&store, "Rice"), 2);
        let second = store.finalize_sale(&draft, PayMethod::Cash, None, None, &owner).unwrap();
        // 1.6 kg rice used → one whole kilo off; 1.0 kg paneer → one off.
        assert_eq!(on_hand(&store, "Rice"), 1);
        assert_eq!(on_hand(&store, "Paneer"), 1);
        // Voiding the second bill puts its usage back.
        store.void_sale(&second.id, "wrong table", &owner).unwrap();
        assert_eq!(on_hand(&store, "Rice"), 2);
        assert_eq!(on_hand(&store, "Paneer"), 2);
        let _ = first;

        // The kitchen never waits on the books: a dry shelf stops at 0.
        for _ in 0..4 {
            store.finalize_sale(&draft, PayMethod::Cash, None, None, &owner).unwrap();
        }
        assert_eq!(on_hand(&store, "Rice"), 0);
        let low: Vec<_> = store.low_stock().unwrap().into_iter().map(|l| l.product.name).collect();
        assert!(low.contains(&"Rice".to_string()), "{low:?}");
        let _ = crate::print_share::take_captured();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn each_item_has_its_own_reorder_level() {
        let (store, path, owner, _) = kitchen();
        let rice = store.list_products(false).unwrap().into_iter().find(|p| p.name == "Rice").unwrap();
        // Both raw items hold 2, under the default of 5.
        assert_eq!(store.low_stock().unwrap().len(), 2);
        store.set_reorder_level(&rice.id, 1, &owner).unwrap();
        let low: Vec<_> = store.low_stock().unwrap().into_iter().map(|l| l.product.name).collect();
        assert_eq!(low, vec!["Paneer".to_string()]);
        assert_eq!(store.reorder_level(&rice.id).unwrap(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn quantities_round_trip_as_typed() {
        assert_eq!(parse_milli("0.2").unwrap(), 200);
        assert_eq!(parse_milli("1,5").unwrap(), 1500);
        assert_eq!(milli_as_typed(200), "0.2");
        assert_eq!(milli_as_typed(1500), "1.5");
        assert_eq!(milli_as_typed(2000), "2");
        assert_eq!(milli_as_typed(125), "0.125");
    }
}

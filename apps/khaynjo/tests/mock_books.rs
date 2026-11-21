//! Build a Gotigin-internal demo restaurant books file (~21 days of trading)
//! via the public Store API, then `VACUUM INTO` a restoreable backup.
//!
//! ```text
//! MOCK_BOOKS_OUT=<workspace>/local/mock-books \
//!   KUMKAJ_PRINT_SINK=off cargo test -p kumkaj-khaynjo --test mock_books -- --ignored --nocapture
//! ```
use chrono::{Duration, Local, NaiveTime};
use kumkaj_khaynjo::domain::*;
use kumkaj_khaynjo::print_share::{self, PrintSink};
use kumkaj_khaynjo::store::Store;
use std::path::{Path, PathBuf};
use std::process::Command;

fn out_root() -> PathBuf {
    if let Ok(p) = std::env::var("MOCK_BOOKS_OUT") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../local/mock-books")
}

fn day_stamp(days_ago: i64, hour: u32, minute: u32) -> String {
    let d = Local::now().date_naive() - Duration::days(days_ago);
    let t = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
    d.and_time(t)
        .and_local_timezone(Local)
        .single()
        .unwrap()
        .to_rfc3339()
}

fn day_key(days_ago: i64) -> String {
    (Local::now().date_naive() - Duration::days(days_ago))
        .format("%Y-%m-%d")
        .to_string()
}

/// Expenses / purchases cannot take `user_at` on the public API; stamp the
/// intended books day into the note, then rewrite after Store closes.
fn stamp_day_notes(db: &Path) {
    let sql = r#"
UPDATE expenses
SET created_at = substr(note, 5, 10) || 'T14:30:00+05:30',
    user_at    = substr(note, 5, 10) || 'T14:30:00+05:30',
    device_at  = substr(note, 5, 10) || 'T14:30:00+05:30',
    note       = substr(note, 16)
WHERE note LIKE 'DAY:____-__-__|%';
UPDATE purchases
SET created_at = substr(note, 5, 10) || 'T10:00:00+05:30',
    user_at    = substr(note, 5, 10) || 'T10:00:00+05:30',
    device_at  = substr(note, 5, 10) || 'T10:00:00+05:30',
    note       = substr(note, 16)
WHERE note LIKE 'DAY:____-__-__|%';
"#;
    let status = Command::new("sqlite3")
        .arg(db)
        .arg(sql)
        .status()
        .expect("sqlite3 required to stamp expense/purchase days");
    assert!(status.success(), "sqlite3 stamp failed");
}

fn summarize(db: &Path) {
    let q = r#"
SELECT 'sales', COUNT(*) FROM sales WHERE status='completed'
UNION ALL SELECT 'sale_lines', COUNT(*) FROM sale_lines
UNION ALL SELECT 'customers', COUNT(*) FROM customers
UNION ALL SELECT 'products', COUNT(*) FROM products
UNION ALL SELECT 'staff', COUNT(*) FROM staff
UNION ALL SELECT 'expenses', COUNT(*) FROM expenses
UNION ALL SELECT 'purchases', COUNT(*) FROM purchases
UNION ALL SELECT 'tables', COUNT(*) FROM tables_dining
UNION ALL SELECT 'sale_days', COUNT(DISTINCT substr(created_at,1,10)) FROM sales WHERE status='completed';
"#;
    let out = Command::new("sqlite3").arg(db).arg(q).output().unwrap();
    print!("{}", String::from_utf8_lossy(&out.stdout));
}

#[test]
#[ignore]
fn generate_mock_books() {
    print_share::set_sink(PrintSink::Off);
    let root = out_root();
    let work = root.join(".work/khaynjo");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let live = work.join("books.sqlite3");
    let dest = root.join("khaynjo/khaynjo.sqlite3");
    if dest.exists() {
        std::fs::remove_file(&dest).unwrap();
    }
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();

    let live_path = {
        let s = Store::open_path(&live).unwrap();
        s.complete_owner_setup("1234").unwrap();
        s.complete_onboarding().unwrap();
        s.set_venue_title("Gotigin Demo Kitchen").unwrap();
        s.set_org_name("Gotigin Demo Kitchen Pvt Ltd").unwrap();

        let owner = s
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|m| m.is_owner())
            .unwrap();
        s.add_staff("Priya", StaffRole::Manager, "Manager", "2222", &owner)
            .unwrap();
        s.add_staff("Wasim", StaffRole::Waiter, "Captain", "3333", &owner)
            .unwrap();
        s.add_staff("Cash", StaffRole::Cashier, "Cashier", "4444", &owner)
            .unwrap();
        s.add_staff("Chef", StaffRole::Kitchen, "Kitchen", "5555", &owner)
            .unwrap();
        let staff = s.list_staff().unwrap();
        let waiter = staff.iter().find(|m| m.name == "Wasim").unwrap().clone();
        let cashier = staff.iter().find(|m| m.name == "Cash").unwrap().clone();

        for t in s.list_tables().unwrap() {
            let _ = s.delete_table(&t.id);
        }
        s.add_tables_range("D", 1, 12, "Dine In").unwrap();
        s.add_tables_range("P", 1, 6, "Parcel").unwrap();
        let tables = s.list_tables().unwrap();
        let dine: Vec<_> = tables
            .iter()
            .filter(|t| t.area == "Dine In")
            .cloned()
            .collect();

        let mains = s.add_category("Mains").unwrap().id;
        let south = s.add_category("South Indian").unwrap().id;
        let starters = s.add_category("Starters").unwrap().id;
        let breads = s.add_category("Breads").unwrap().id;
        let drinks = s.add_category("Drinks").unwrap().id;
        let sweets = s.add_category("Sweets").unwrap().id;

        let dish_defs = [
            ("Dal Makhani", 220, &mains, false, 0),
            ("Paneer Butter Masala", 260, &mains, false, 0),
            ("Veg Biryani", 210, &mains, false, 0),
            ("Chicken Curry", 280, &mains, false, 0),
            ("Gujarati Thali", 250, &mains, false, 0),
            ("Masala Dosa", 120, &south, false, 0),
            ("Idli Sambar", 80, &south, false, 0),
            ("Medu Vada", 70, &south, false, 0),
            ("Uttapam", 110, &south, false, 0),
            ("Paneer Tikka (Half)", 140, &starters, false, 0),
            ("Paneer Tikka (Full)", 240, &starters, false, 0),
            ("Hara Bhara Kabab", 160, &starters, false, 0),
            ("Butter Naan", 40, &breads, false, 0),
            ("Roti", 20, &breads, false, 0),
            ("Buttermilk", 30, &drinks, false, 0),
            ("Masala Chai", 20, &drinks, false, 0),
            ("Fresh Lime Soda", 60, &drinks, false, 0),
            ("Bottled Water", 20, &drinks, true, 120),
            ("Gulab Jamun", 50, &sweets, false, 0),
            ("Ice Cream", 80, &sweets, true, 36),
        ];
        for (name, rs, cat, resale, qty) in dish_defs {
            let mut p = Product::new_basic(new_id(), name, Money::from_rupees(rs));
            p.category_id = cat.clone();
            if resale {
                p.kind = ITEM_KIND_RESALE.into();
                p.stock_qty = qty;
            }
            s.upsert_product(&p).unwrap();
        }
        let mut raw_ids = Vec::new();
        for (name, unit, qty) in [
            ("Rice", "kg", 80),
            ("Paneer", "kg", 12),
            ("Milk", "litre", 40),
            ("Cooking oil", "litre", 25),
            ("Onion", "kg", 50),
            ("Gas cylinder", "pcs", 4),
        ] {
            let mut m = Product::new_raw_material(name, unit);
            m.stock_qty = qty;
            s.upsert_product(&m).unwrap();
            raw_ids.push(m.id);
        }

        let parties = [
            ("Mehta family", "98250 11111", 200_000_00),
            ("Shah Traders", "98250 22222", 200_000_00),
            ("Neha Patel", "98250 33333", 200_000_00),
            ("Walk-in Club", "98250 44444", 200_000_00),
            ("Office Lunch Co", "98250 55555", 200_000_00),
        ];
        let mut customers = Vec::new();
        for (name, phone, limit) in parties {
            let c = s.add_customer(name, phone).unwrap();
            s.set_customer_credit_limit(&c.id, limit).unwrap();
            customers.push(c);
        }

        let prods = s.list_products(false).unwrap();
        let menu: Vec<_> = prods
            .iter()
            .filter(|p| p.kind != ITEM_KIND_RAW)
            .cloned()
            .collect();
        assert!(menu.len() >= 15);

        const DAYS: i64 = 21;
        let mut sale_n = 0u32;
        let mut rng = 0xC0FFEE_u64;
        let mut next = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            rng
        };

        for days_ago in (0..DAYS).rev() {
            let key = day_key(days_ago);
            let bills = 10 + (next() % 9) as usize; // 10–18
            for b in 0..bills {
                let hour = 11 + ((next() % 10) as u32);
                let minute = (next() % 60) as u32;
                let mut cart = Cart::default();
                let n_lines = 1 + (next() % 4) as usize;
                for _ in 0..n_lines {
                    let p = &menu[(next() as usize) % menu.len()];
                    cart.add_or_bump(p);
                    if next() % 3 == 0 {
                        cart.add_or_bump(p);
                    }
                }
                if next() % 11 == 0 && cart.lines_applied_paise() > 100_00 {
                    cart.discount = Some(BillDiscount {
                        kind: DiscountKind::Amount(50_00),
                        reason: "Demo goodwill".into(),
                        approval_pin: String::new(),
                    });
                }
                let mut draft = cart.to_sale_draft().unwrap();
                draft.user_at = Some(day_stamp(days_ago, hour, minute));

                let pay_roll = next() % 10;
                let (pay, cust) = match pay_roll {
                    0 | 1 => (
                        PayMethod::Credit,
                        Some(customers[(next() as usize) % customers.len()].id.as_str()),
                    ),
                    2 | 3 => (PayMethod::Upi, None),
                    4 => (PayMethod::Card, None),
                    _ => (PayMethod::Cash, None),
                };
                let table = if next() % 4 == 0 {
                    None
                } else {
                    Some(dine[(next() as usize) % dine.len()].id.as_str())
                };
                // Waiters cannot Settle; discounts need Right::Discount.
                let actor = if draft.discount_paise > 0 {
                    &owner
                } else {
                    &cashier
                };
                s.finalize_sale(&draft, pay, table, cust, actor).unwrap();
                sale_n += 1;

                if b == 0 && days_ago % 2 == 0 {
                    let amt = 200_00 + ((next() % 8) as i64) * 50_00;
                    let note = format!("DAY:{key}|Gas / veg vendor");
                    s.add_expense(amt, "Supplies", &note, &owner).unwrap();
                }
                if b == 1 && days_ago % 5 == 0 {
                    let rice = prods.iter().find(|p| p.name == "Rice").unwrap();
                    let oil = prods.iter().find(|p| p.name == "Cooking oil").unwrap();
                    let note = format!("DAY:{key}|Weekly stock-in");
                    s.record_purchase(
                        "Wholesale Mart",
                        &[
                            (rice.id.clone(), 10_000, 45_00),
                            (oil.id.clone(), 5_000, 130_00),
                        ],
                        &note,
                        &owner,
                    )
                    .unwrap();
                }
            }
            if !customers.is_empty() {
                let c = &customers[(next() as usize) % customers.len()];
                let _ = s.settle_credit(&c.id, 2_000_00, PayMethod::Cash, &owner, "Partial settle");
            }
        }

        // Live floor: a few open KOTs so the desk is not empty after restore.
        let mut open = Cart::default();
        open.add_or_bump(&menu[0]);
        open.add_or_bump(&menu[1]);
        s.send_kot(&open.to_kot_draft().unwrap(), Some(&dine[0].id), &waiter)
            .unwrap();
        let mut open2 = Cart::default();
        open2.add_or_bump(&menu[2]);
        let kots = s
            .send_kot(&open2.to_kot_draft().unwrap(), Some(&dine[1].id), &waiter)
            .unwrap();
        for k in &kots {
            let _ = s.advance_kot(k, &owner);
        }

        println!("khaynjo seeded {sale_n} sales across {DAYS} days");
        live
    };

    stamp_day_notes(&live_path);
    {
        let s = Store::open_path(&live_path).unwrap();
        // Spot-check Store still opens and day totals exist.
        let _ = s.recent_sales(5).unwrap();
        s.backup_to(&dest).unwrap();
    }
    assert!(dest.exists());
    {
        let s = Store::open_path(&dest).unwrap();
        assert!(s.recent_sales(1).unwrap().len() >= 1);
    }
    println!("wrote {}", dest.display());
    summarize(&dest);
}

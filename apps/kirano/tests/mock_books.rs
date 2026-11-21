//! Build a Gotigin-internal demo kirana books file (~21 days of counter sales)
//! via the public Store API, then `VACUUM INTO` a restoreable backup.
//!
//! ```text
//! MOCK_BOOKS_OUT=<workspace>/local/mock-books \
//!   KUMKAJ_PRINT_SINK=off cargo test -p kumkaj-kirano --test mock_books -- --ignored --nocapture
//! ```
use chrono::{Duration, Local, NaiveTime};
use kumkaj_kirano::domain::*;
use kumkaj_kirano::print_share::{self, PrintSink};
use kumkaj_kirano::store::Store;
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

fn stamp_day_notes(db: &Path) {
    let sql = r#"
UPDATE expenses
SET created_at = substr(note, 5, 10) || 'T13:00:00+05:30',
    user_at    = substr(note, 5, 10) || 'T13:00:00+05:30',
    device_at  = substr(note, 5, 10) || 'T13:00:00+05:30',
    note       = substr(note, 16)
WHERE note LIKE 'DAY:____-__-__|%';
UPDATE purchases
SET created_at = substr(note, 5, 10) || 'T09:30:00+05:30',
    user_at    = substr(note, 5, 10) || 'T09:30:00+05:30',
    device_at  = substr(note, 5, 10) || 'T09:30:00+05:30',
    note       = substr(note, 16)
WHERE note LIKE 'DAY:____-__-__|%';
"#;
    let status = Command::new("sqlite3")
        .arg(db)
        .arg(sql)
        .status()
        .expect("sqlite3 required");
    assert!(status.success());
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
    let work = root.join(".work/kirano");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let live = work.join("books.sqlite3");
    let dest = root.join("kirano/kirano.sqlite3");
    if dest.exists() {
        std::fs::remove_file(&dest).unwrap();
    }
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();

    let live_path = {
        let s = Store::open_path(&live).unwrap();
        s.complete_owner_setup("1234").unwrap();
        s.complete_onboarding().unwrap();
        s.set_venue_title("Gotigin Neighbourhood Kirana").unwrap();
        s.set_org_name("Gotigin Kirana Mart").unwrap();

        let owner = s
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|m| m.is_owner())
            .unwrap();
        s.add_staff("Asha", StaffRole::Cashier, "Counter", "2222", &owner)
            .unwrap();
        s.add_staff("Ramesh", StaffRole::Manager, "Manager", "3333", &owner)
            .unwrap();
        let cashier = s
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|m| m.name == "Asha")
            .unwrap();

        let staples = s.add_category("Staples").unwrap().id;
        let oils = s.add_category("Oils").unwrap().id;
        let snacks = s.add_category("Snacks").unwrap().id;
        let personal = s.add_category("Personal care").unwrap().id;
        let dairy = s.add_category("Dairy").unwrap().id;
        let beverages = s.add_category("Beverages").unwrap().id;

        let items = [
            ("Atta 5kg", 280, &staples, false, 0),
            ("Atta 1kg", 60, &staples, false, 0),
            ("Rice 1kg", 55, &staples, false, 0),
            ("Rice 5kg", 260, &staples, false, 0),
            ("Toor dal 1kg", 140, &staples, false, 0),
            ("Sugar 1kg", 45, &staples, false, 0),
            ("Cooking oil 1L", 140, &oils, false, 0),
            ("Mustard oil 1L", 160, &oils, false, 0),
            ("Ghee 500ml", 320, &oils, false, 0),
            ("Namkeen 200g", 40, &snacks, true, 60),
            ("Biscuits pack", 30, &snacks, true, 80),
            ("Chips", 20, &snacks, true, 100),
            ("Soap", 25, &personal, true, 96),
            ("Shampoo sachet", 5, &personal, true, 200),
            ("Toothpaste", 55, &personal, true, 48),
            ("Milk 1L", 60, &dairy, true, 40),
            ("Curd 400g", 35, &dairy, true, 36),
            ("Butter 100g", 58, &dairy, true, 30),
            ("Tea powder 250g", 95, &beverages, false, 0),
            ("Bottled water", 20, &beverages, true, 120),
            ("Cold drink", 40, &beverages, true, 72),
        ];
        for (name, rs, cat, resale, qty) in items {
            let mut p = Product::new_basic(new_id(), name, Money::from_rupees(rs));
            p.category_id = cat.clone();
            if resale {
                p.kind = ITEM_KIND_RESALE.into();
                // High shelf so 21 days of mock bills do not hit zero mid-run.
                p.stock_qty = qty.max(2_000);
            }
            s.upsert_product(&p).unwrap();
        }

        let parties = [
            ("Sharma ji", "98250 10001", 200_000_00),
            ("Patel family", "98250 10002", 200_000_00),
            ("Building A-12", "98250 10003", 200_000_00),
            ("Tea stall next door", "98250 10004", 200_000_00),
            ("School canteen", "98250 10005", 200_000_00),
        ];
        let mut customers = Vec::new();
        for (name, phone, limit) in parties {
            let c = s.add_customer(name, phone).unwrap();
            s.set_customer_credit_limit(&c.id, limit).unwrap();
            customers.push(c);
        }

        let prods = s.list_products(false).unwrap();
        let resale: Vec<_> = prods
            .iter()
            .filter(|p| p.kind == ITEM_KIND_RESALE)
            .cloned()
            .collect();

        const DAYS: i64 = 21;
        let mut sale_n = 0u32;
        let mut rng = 0xB00C5_u64;
        let mut next = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            rng
        };

        for days_ago in (0..DAYS).rev() {
            let key = day_key(days_ago);
            let bills = 12 + (next() % 10) as usize; // 12–21
            for b in 0..bills {
                let mut cart = Cart::default();
                let n_lines = 1 + (next() % 5) as usize;
                for _ in 0..n_lines {
                    let p = &prods[(next() as usize) % prods.len()];
                    cart.add_or_bump(p);
                    if next() % 4 == 0 {
                        cart.add_or_bump(p);
                    }
                }
                let mut draft = cart.to_sale_draft().unwrap();
                draft.user_at = Some(day_stamp(
                    days_ago,
                    8 + ((next() % 12) as u32),
                    (next() % 60) as u32,
                ));
                let roll = next() % 10;
                let (pay, cust) = match roll {
                    0 | 1 => (
                        PayMethod::Credit,
                        Some(customers[(next() as usize) % customers.len()].id.as_str()),
                    ),
                    2 | 3 => (PayMethod::Upi, None),
                    _ => (PayMethod::Cash, None),
                };
                let actor = if next() % 3 == 0 { &owner } else { &cashier };
                s.finalize_sale(&draft, pay, cust, actor).unwrap();
                sale_n += 1;

                if b == 0 && days_ago % 2 == 0 {
                    s.add_expense(
                        150_00 + ((next() % 5) as i64) * 50_00,
                        "Shop running",
                        &format!("DAY:{key}|Electricity / packing"),
                        &owner,
                    )
                    .unwrap();
                }
                if b == 2 && days_ago % 4 == 0 && !resale.is_empty() {
                    let p = &resale[(next() as usize) % resale.len()];
                    s.record_purchase(
                        "City Wholesale",
                        &[(p.id.clone(), 24_000, (p.price.paise() * 70) / 100)],
                        &format!("DAY:{key}|Restock {0}", p.name),
                        &owner,
                    )
                    .unwrap();
                }
            }
            {
                let c = &customers[(next() as usize) % customers.len()];
                let _ = s.settle_credit(&c.id, 1_500_00, PayMethod::Cash, &owner, "Khata settle");
            }
        }

        println!("kirano seeded {sale_n} sales across {DAYS} days");
        live
    };

    stamp_day_notes(&live_path);
    {
        let s = Store::open_path(&live_path).unwrap();
        let _ = s.recent_sales(5).unwrap();
        s.backup_to(&dest).unwrap();
    }
    {
        let s = Store::open_path(&dest).unwrap();
        assert!(!s.recent_sales(1).unwrap().is_empty());
    }
    println!("wrote {}", dest.display());
    summarize(&dest);
}

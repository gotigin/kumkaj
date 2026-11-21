//! Build a Gotigin-internal demo pharmacy books file (~21 days of counter sales)
//! via the public Store API, then `VACUUM INTO` a restoreable backup.
//!
//! ```text
//! MOCK_BOOKS_OUT=<workspace>/local/mock-books \
//!   KUMKAJ_PRINT_SINK=off cargo test -p kumkaj-tabiyat --test mock_books -- --ignored --nocapture
//! ```
use chrono::{Duration, Local, NaiveTime};
use kumkaj_tabiyat::domain::*;
use kumkaj_tabiyat::print_share::{self, PrintSink};
use kumkaj_tabiyat::store::Store;
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
SET created_at = substr(note, 5, 10) || 'T13:15:00+05:30',
    user_at    = substr(note, 5, 10) || 'T13:15:00+05:30',
    device_at  = substr(note, 5, 10) || 'T13:15:00+05:30',
    note       = substr(note, 16)
WHERE note LIKE 'DAY:____-__-__|%';
UPDATE purchases
SET created_at = substr(note, 5, 10) || 'T09:45:00+05:30',
    user_at    = substr(note, 5, 10) || 'T09:45:00+05:30',
    device_at  = substr(note, 5, 10) || 'T09:45:00+05:30',
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
    let work = root.join(".work/tabiyat");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let live = work.join("books.sqlite3");
    let dest = root.join("tabiyat/tabiyat.sqlite3");
    if dest.exists() {
        std::fs::remove_file(&dest).unwrap();
    }
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();

    let live_path = {
        let s = Store::open_path(&live).unwrap();
        s.complete_owner_setup("1234").unwrap();
        s.complete_onboarding().unwrap();
        s.set_venue_title("Gotigin City Chemist").unwrap();
        s.set_org_name("Gotigin City Chemist").unwrap();

        let owner = s
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|m| m.is_owner())
            .unwrap();
        s.add_staff("Farida", StaffRole::Cashier, "Counter", "2222", &owner)
            .unwrap();
        s.add_staff("Dr Mehta", StaffRole::Manager, "Pharmacist", "3333", &owner)
            .unwrap();
        let cashier = s
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|m| m.name == "Farida")
            .unwrap();

        let otc = s.add_category("OTC").unwrap().id;
        let antibiotics = s.add_category("Antibiotics").unwrap().id;
        let chronic = s.add_category("Chronic").unwrap().id;
        let care = s.add_category("Personal care").unwrap().id;
        let baby = s.add_category("Baby care").unwrap().id;
        let devices = s.add_category("Devices").unwrap().id;

        let items = [
            ("Paracetamol 500mg strip", 30, &otc, false, 0),
            ("Ibuprofen 400mg strip", 45, &otc, false, 0),
            ("Cetirizine strip", 25, &otc, false, 0),
            ("ORS sachet", 20, &otc, false, 0),
            ("Cough syrup 100ml", 95, &otc, false, 0),
            ("Amoxicillin 250mg strip", 80, &antibiotics, false, 0),
            ("Azithromycin 500mg", 120, &antibiotics, false, 0),
            ("Ciprofloxacin strip", 90, &antibiotics, false, 0),
            ("Metformin 500mg", 55, &chronic, false, 0),
            ("Amlodipine 5mg", 40, &chronic, false, 0),
            ("Atorvastatin 10mg", 75, &chronic, false, 0),
            ("Insulin glargine", 450, &chronic, true, 18),
            ("Antiseptic cream", 45, &care, true, 36),
            ("Hand sanitizer 100ml", 50, &care, true, 48),
            ("Cotton roll", 35, &care, true, 40),
            ("Bandage pack", 30, &care, true, 40),
            ("Baby diapers M", 320, &baby, true, 24),
            ("Baby lotion", 180, &baby, true, 20),
            ("Digital thermometer", 250, &devices, true, 12),
            ("BP monitor arm", 1_450, &devices, true, 6),
            ("Glucometer strips 25", 480, &devices, true, 15),
        ];
        for (name, rs, cat, resale, qty) in items {
            let mut p = Product::new_basic(new_id(), name, Money::from_rupees(rs));
            p.category_id = cat.clone();
            if resale {
                p.kind = ITEM_KIND_RESALE.into();
                p.stock_qty = qty.max(2_000);
            }
            s.upsert_product(&p).unwrap();
        }

        let parties = [
            ("Patel family", "98250 20002", 200_000_00),
            ("Clinic walk-ins", "98250 20003", 200_000_00),
            ("Senior home", "98250 20004", 200_000_00),
            ("Dr Shah patients", "98250 20005", 200_000_00),
            ("Factory first-aid", "98250 20006", 200_000_00),
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
        let mut rng = 0x7AB1_7A71_u64;
        let mut next = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            rng
        };

        for days_ago in (0..DAYS).rev() {
            let key = day_key(days_ago);
            let bills = 11 + (next() % 9) as usize; // 11–19
            for b in 0..bills {
                let mut cart = Cart::default();
                let n_lines = 1 + (next() % 4) as usize;
                for _ in 0..n_lines {
                    let p = &prods[(next() as usize) % prods.len()];
                    cart.add_or_bump(p);
                    if next() % 5 == 0 {
                        cart.add_or_bump(p);
                    }
                }
                let mut draft = cart.to_sale_draft().unwrap();
                draft.user_at = Some(day_stamp(
                    days_ago,
                    9 + ((next() % 11) as u32),
                    (next() % 60) as u32,
                ));
                let roll = next() % 10;
                let (pay, cust) = match roll {
                    0 | 1 => (
                        PayMethod::Credit,
                        Some(customers[(next() as usize) % customers.len()].id.as_str()),
                    ),
                    2 | 3 | 4 => (PayMethod::Upi, None),
                    5 => (PayMethod::Card, None),
                    _ => (PayMethod::Cash, None),
                };
                let actor = if next() % 3 == 0 { &owner } else { &cashier };
                s.finalize_sale(&draft, pay, cust, actor).unwrap();
                sale_n += 1;

                if b == 0 && days_ago % 2 == 0 {
                    s.add_expense(
                        200_00 + ((next() % 6) as i64) * 50_00,
                        "Shop running",
                        &format!("DAY:{key}|Cold chain / rent share"),
                        &owner,
                    )
                    .unwrap();
                }
                if b == 1 && days_ago % 4 == 0 && !resale.is_empty() {
                    let p = &resale[(next() as usize) % resale.len()];
                    s.record_purchase(
                        "Pharma Dist.",
                        &[(p.id.clone(), 12_000, (p.price.paise() * 65) / 100)],
                        &format!("DAY:{key}|Restock {0}", p.name),
                        &owner,
                    )
                    .unwrap();
                }
            }
            {
                let c = &customers[(next() as usize) % customers.len()];
                let _ = s.settle_credit(&c.id, 2_000_00, PayMethod::Upi, &owner, "Account settle");
            }
        }

        println!("tabiyat seeded {sale_n} sales across {DAYS} days");
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

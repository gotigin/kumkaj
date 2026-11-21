//! Build a Gotigin-internal demo courier books file (~21 days of bookings)
//! via the public Store API, then `VACUUM INTO` a restoreable backup.
//!
//! ```text
//! MOCK_BOOKS_OUT=<workspace>/local/mock-books \
//!   KUMKAJ_PRINT_SINK=off cargo test -p kumkaj-moklanjo --test mock_books -- --ignored --nocapture
//! ```
use chrono::{Duration, Local, NaiveTime};
use kumkaj_moklanjo::domain::*;
use kumkaj_moklanjo::local_servers;
use kumkaj_moklanjo::manifest::{HandoverKind, TrackingStatus};
use kumkaj_moklanjo::print_share::{self, PrintSink};
use kumkaj_moklanjo::store::Store;
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
SET created_at = substr(note, 5, 10) || 'T15:00:00+05:30',
    user_at    = substr(note, 5, 10) || 'T15:00:00+05:30',
    device_at  = substr(note, 5, 10) || 'T15:00:00+05:30',
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
UNION ALL SELECT 'riders', COUNT(*) FROM riders
UNION ALL SELECT 'sale_days', COUNT(DISTINCT substr(created_at,1,10)) FROM sales WHERE status='completed';
"#;
    let out = Command::new("sqlite3").arg(db).arg(q).output().unwrap();
    print!("{}", String::from_utf8_lossy(&out.stdout));
}

fn book(
    s: &Store,
    rate: &Product,
    to: &str,
    consignee: &str,
    kg: &str,
    pay: PayMethod,
    party: Option<&str>,
    cod_rupees: i64,
    owner: &StaffMember,
    user_at: &str,
) -> Sale {
    let mut cart = Cart::default();
    cart.add_or_bump(rate);
    let line = cart.lines.last_mut().unwrap();
    kumkaj_moklanjo::booking::apply_weight_and_slab(line, rate, kg.parse().unwrap()).unwrap();
    let mut draft = cart.to_sale_draft().unwrap();
    draft.destination = to.into();
    draft.consignee_name = consignee.into();
    draft.chargeable_weight_kg = kg.parse().unwrap();
    draft.actual_weight_kg = kg.parse().unwrap();
    draft.zone_id = rate.zone_id.clone();
    draft.cod_paise = cod_rupees * 100;
    draft.user_at = Some(user_at.into());
    s.finalize_sale(&draft, pay, party, owner).unwrap()
}

#[test]
#[ignore]
fn generate_mock_books() {
    print_share::set_sink(PrintSink::Off);
    let root = out_root();
    let work = root.join(".work/moklanjo");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let dest = root.join("moklanjo/moklanjo.sqlite3");
    if dest.exists() {
        std::fs::remove_file(&dest).unwrap();
    }
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();

    let (live_path, _) = {
        let (_, s) = local_servers::create_local_server(&work, "Gotigin Swift Courier").unwrap();
        let owner = s.complete_owner_setup("1234").unwrap();
        s.complete_onboarding().unwrap();
        s.set_org_name("Gotigin Swift Couriers & Cargo").unwrap();
        s.add_staff("Asha", StaffRole::Cashier, "Counter", "2222", &owner)
            .unwrap();
        s.add_staff("Vikram", StaffRole::Manager, "Ops", "3333", &owner)
            .unwrap();

        let mut rates = Vec::new();
        for (zone, name, first, extra) in [
            ("Local", "Local 1 kg", 60, 20),
            ("West", "Surface West", 80, 35),
            ("North", "Express North", 120, 60),
            ("South", "Surface South", 90, 40),
            ("East", "Surface East", 95, 42),
        ] {
            let z = s.add_zone(zone).unwrap();
            let mut r = Product::new_basic(new_id(), name, Money::from_rupees(first));
            r.unit_label = "kg".into();
            r.zone_id = z.id.clone();
            r.first_kg = "1".parse().unwrap();
            r.first_paise = first * 100;
            r.extra_per_kg_paise = extra * 100;
            s.upsert_product(&r).unwrap();
            rates.push(r);
        }

        let party = |name: &str, phone: &str, kind: &str| s.add_customer_kind(name, phone, kind).unwrap();
        let mehta = party("Mehta Traders", "98250 11111", "Consignor");
        let textiles = party("Rajkot Textiles", "98250 22222", "Consignor");
        let pharma = party("Sunrise Pharma", "98250 33333", "Consignor");
        let garments = party("Saurashtra Garments", "98250 44444", "Consignor");
        let blueline = party("BlueLine Express", "1800 000 111", "Carrier");
        let _northstar = party("Northstar Logistics", "1800 000 222", "Carrier");
        for c in [&mehta, &textiles, &pharma, &garments] {
            s.set_customer_credit_limit(&c.id, 500_000_00).unwrap();
        }

        let ravi = s.add_rider("Ravi", "99000 11111", "Bike 12").unwrap();
        let kiran = s.add_rider("Kiran", "99000 22222", "Scooter 7").unwrap();
        let _neha = s.add_rider("Neha", "99000 33333", "Bike 3").unwrap();

        let cities = [
            "Rajkot", "Pune", "Delhi", "Bengaluru", "Surat", "Jaipur", "Ahmedabad", "Morbi",
            "Gondal", "Mumbai", "Indore", "Hyderabad",
        ];
        let names = [
            "Neha Shah",
            "Imran Ali",
            "Kapoor Stores",
            "Sunita Rao",
            "Hitesh Patel",
            "Farah Khan",
            "Arjun Mehra",
            "Pooja Desai",
            "Ramesh Joshi",
            "Anita Bhatt",
        ];
        let consignors = [&mehta, &textiles, &pharma, &garments];
        let mut rng = 0xA11CE_u64;
        let mut next = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            rng
        };

        const DAYS: i64 = 21;
        let mut sale_n = 0u32;
        let mut recent_for_run: Vec<String> = Vec::new();

        for days_ago in (0..DAYS).rev() {
            let key = day_key(days_ago);
            let n = 8 + (next() % 7) as usize; // 8–14
            for i in 0..n {
                let rate = &rates[(next() as usize) % rates.len()];
                let kg = match next() % 5 {
                    0 => "0.5",
                    1 => "1",
                    2 => "1.5",
                    3 => "2.5",
                    _ => "4",
                };
                let to = cities[(next() as usize) % cities.len()];
                let who = names[(next() as usize) % names.len()];
                let hour = 9 + ((next() % 10) as u32);
                let minute = (next() % 60) as u32;
                let at = day_stamp(days_ago, hour, minute);
                let roll = next() % 10;
                let (pay, party_id, cod) = match roll {
                    0 | 1 => (
                        PayMethod::Credit,
                        Some(consignors[(next() as usize) % consignors.len()].id.as_str()),
                        0,
                    ),
                    2 => (
                        PayMethod::Cash,
                        Some(pharma.id.as_str()),
                        300 + ((next() % 10) as i64) * 50,
                    ),
                    3 | 4 => (PayMethod::Upi, None, 0),
                    _ => (PayMethod::Cash, None, 0),
                };
                let sale = book(
                    &s, rate, to, who, kg, pay, party_id, cod, &owner, &at,
                );
                recent_for_run.push(sale.id.clone());
                sale_n += 1;

                // Deliver / NDR a subset so Ops boards look used.
                if i % 3 == 0 && days_ago > 0 {
                    let _ = s.advance_tracking(
                        &sale.id,
                        TrackingStatus::Delivered,
                        "signed",
                        "",
                        &owner,
                    );
                } else if i % 7 == 0 && days_ago > 1 {
                    let _ = s.create_manifest(
                        HandoverKind::OwnFleet,
                        "",
                        if next() % 2 == 0 { &ravi.id } else { &kiran.id },
                        &[sale.id.clone()],
                        "mock run",
                        &owner,
                    );
                    if next() % 4 == 0 {
                        let _ = s.record_ndr(&sale.id, "door_locked", &key, &owner);
                    }
                } else if i % 5 == 0 {
                    let _ = s.create_manifest(
                        HandoverKind::Carrier,
                        &blueline.id,
                        "",
                        &[sale.id.clone()],
                        "carrier bag",
                        &owner,
                    );
                }
            }
            if days_ago % 2 == 0 {
                let amt = 400_00 + ((next() % 6) as i64) * 100_00;
                s.add_expense(
                    amt,
                    "Carrier bill",
                    &format!("DAY:{key}|BlueLine / fuel"),
                    &owner,
                )
                .unwrap();
            }
            if days_ago % 5 == 0 {
                let _ = s.settle_credit(
                    &mehta.id,
                    1_000_00,
                    PayMethod::Upi,
                    &owner,
                    "Account settle",
                );
            }
        }

        // Leave a few booked (no advance) for the live desk.
        let _ = book(
            &s,
            &rates[0],
            "Rajkot",
            "Desk Sample",
            "1",
            PayMethod::Cash,
            None,
            0,
            &owner,
            &day_stamp(0, 16, 5),
        );
        sale_n += 1;

        println!("moklanjo seeded {sale_n} bookings across {DAYS} days");
        let live = s.db_path().to_path_buf();
        (live, recent_for_run.len())
    };

    // Prefer discovering the live DB under the server folder.
    let live = if live_path.exists() {
        live_path
    } else {
        find_sqlite(&work).expect("live moklanjo sqlite")
    };
    stamp_day_notes(&live);
    {
        let s = Store::open_path(&live).unwrap();
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

fn find_sqlite(dir: &Path) -> Option<PathBuf> {
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("sqlite3") {
                out.push(p);
            }
        }
    }
    let mut found = Vec::new();
    walk(dir, &mut found);
    found.into_iter().next()
}

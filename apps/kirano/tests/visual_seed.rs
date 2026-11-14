//! Seeds a realistic kirana counter (Atta, Rice, Oil, Soap, stock, one
//! cash sale and a khata party) into a shop database so the UI can be
//! looked at, not guessed (docs/DEV.md § Look at the screen). Opt-in only.
//!
//! Prefer a data folder (creates the local Server + Owner PIN **1234**):
//! ```text
//! SEED_DATA_DIR=<empty folder> KUMKAJ_PRINT_SINK=off cargo test -p
//! kumkaj-kirano --test visual_seed -- --ignored
//! ```
//! Then start with `KUMKAJ_DATA_DIR=<same folder>`, pick Server, unlock 1234.
//!
//! Or seed an existing sqlite path: `SEED_DB=<…/kirano.sqlite3> …`.
use kumkaj_kirano::local_servers;
use kumkaj_kirano::{domain::*, store::Store};
use std::path::PathBuf;

#[test]
#[ignore]
fn seed_visual() {
    let s = if let Ok(base) = std::env::var("SEED_DATA_DIR") {
        let base = PathBuf::from(base);
        std::fs::create_dir_all(&base).unwrap();
        let (_, store) = local_servers::create_local_server(&base, "Neighbourhood Kirana").unwrap();
        store.complete_owner_setup("1234").unwrap();
        store
    } else {
        let db = std::env::var("SEED_DB").expect("set SEED_DATA_DIR or SEED_DB");
        let store = Store::open_path(std::path::Path::new(&db)).unwrap();
        // Best-effort PIN so unlock works when seeding a bare sqlite.
        let _ = store.complete_owner_setup("1234");
        store
    };
    s.complete_onboarding().unwrap();
    s.set_venue_title("Neighbourhood Kirana").unwrap();
    let owner = s
        .list_staff()
        .unwrap()
        .into_iter()
        .find(|m| m.is_owner())
        .unwrap();

    let staples = s.add_category("Staples").unwrap().id;
    let oils = s.add_category("Oils").unwrap().id;
    let personal = s.add_category("Personal care").unwrap().id;
    let items = [
        ("Atta 5kg", 280, &staples, false, 0),
        ("Rice 1kg", 55, &staples, false, 0),
        ("Cooking oil 1L", 140, &oils, false, 0),
        ("Soap", 25, &personal, true, 48),
    ];
    for (name, rs, cat, resale, qty) in items {
        let mut p = Product::new_basic(new_id(), name, Money::from_rupees(rs));
        p.category_id = cat.clone();
        if resale {
            p.kind = ITEM_KIND_RESALE.into();
            p.stock_qty = qty;
        }
        s.upsert_product(&p).unwrap();
    }

    let party = s.add_customer("Sharma ji", "98250 10001").unwrap();
    s.set_customer_credit_limit(&party.id, 5_000_00).unwrap();

    let prods = s.list_products(false).unwrap();
    let by = |n: &str| prods.iter().find(|p| p.name == n).unwrap().clone();
    let mut cart = Cart::default();
    cart.add_or_bump(&by("Atta 5kg"));
    cart.add_or_bump(&by("Soap"));
    cart.add_or_bump(&by("Soap"));
    s.finalize_sale(
        &cart.to_sale_draft().unwrap(),
        PayMethod::Cash,
        None,
        &owner,
    )
    .unwrap();

    for p in s.list_products(false).unwrap() {
        println!(
            "{} ₹{} kind={} stock={}",
            p.name,
            p.price.paise() / 100,
            p.kind,
            p.stock_qty
        );
    }
}

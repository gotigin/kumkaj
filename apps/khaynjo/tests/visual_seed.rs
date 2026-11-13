//! Seeds a neutral demo restaurant ("Demo Kitchen": D1–D8, P1–P4, every
//! visit state, photos and initials — never a client's menu or staff) into
//! a shop database so the UI can be looked at, not guessed (docs/DEV.md
//! § Look at the screen). Opt-in only.
//!
//! Prefer a data folder (creates the local Server + Owner PIN **1234**):
//! ```text
//! SEED_DATA_DIR=<empty folder> KUMKAJ_PRINT_SINK=off cargo test -p
//! kumkaj-khaynjo --test visual_seed -- --ignored
//! ```
//! Then start with `KUMKAJ_DATA_DIR=<same folder>`, pick Server, unlock 1234.
//!
//! Or seed an existing sqlite path: `SEED_DB=<…/khaynjo.sqlite3> …`.
use kumkaj_khaynjo::local_servers;
use kumkaj_khaynjo::{domain::*, store::Store};
use std::path::PathBuf;

#[test]
#[ignore]
fn seed_visual() {
    let s = if let Ok(base) = std::env::var("SEED_DATA_DIR") {
        let base = PathBuf::from(base);
        std::fs::create_dir_all(&base).unwrap();
        let (_, store) = local_servers::create_local_server(&base, "Demo Kitchen").unwrap();
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
    s.set_venue_title("Demo Kitchen").unwrap();
    let owner = s
        .list_staff()
        .unwrap()
        .into_iter()
        .find(|m| m.is_owner())
        .unwrap();
    for t in s.list_tables().unwrap() {
        let _ = s.delete_table(&t.id);
    }
    s.add_tables_range("D", 1, 8, "Dine In").unwrap();
    s.add_tables_range("P", 1, 4, "Parcel").unwrap();
    let shelf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/items");
    // A real-sized menu in categories, so the order desk chips and tile
    // grid are looked at with something to show.
    let mains = s.add_category("Mains").unwrap().id;
    let south = s.add_category("South Indian").unwrap().id;
    let starters = s.add_category("Starters").unwrap().id;
    let drinks = s.add_category("Drinks").unwrap().id;
    let dishes = [
        (
            "Dal Makhani",
            220,
            &mains,
            Some("indian-mains/dal-makhani.webp"),
        ),
        ("Paneer Butter Masala", 260, &mains, None),
        ("Veg Biryani", 210, &mains, None),
        (
            "Masala Dosa",
            120,
            &south,
            Some("south-indian/masala-dosa.webp"),
        ),
        ("Idli Sambar", 80, &south, None),
        ("Medu Vada", 70, &south, None),
        ("Uttapam", 110, &south, None),
        ("Paneer Tikka (Half)", 140, &starters, None),
        ("Paneer Tikka (Full)", 240, &starters, None),
        ("Hara Bhara Kabab", 160, &starters, None),
        ("Buttermilk", 30, &drinks, None),
        ("Bottled Water", 20, &drinks, None),
        ("Masala Chai", 20, &drinks, None),
        ("Fresh Lime Soda", 60, &drinks, None),
    ];
    for (name, rs, cat, photo) in dishes {
        let mut p = Product::new_basic(new_id(), name, Money::from_rupees(rs));
        p.category_id = cat.clone();
        if name.starts_with("Paneer") {
            p.group_name = "Paneer Tikka".into();
        }
        // Bottled water is bought and resold as is; counted like stock.
        if name == "Bottled Water" {
            p.kind = ITEM_KIND_RESALE.into();
            p.stock_qty = 24;
        }
        if let Some(rel) = photo {
            let src = shelf.join(rel);
            if src.exists() {
                p.image_key = Some(s.import_item_image(&p.id, &src).unwrap());
            }
        }
        s.upsert_product(&p).unwrap();
    }
    // What the kitchen buys and uses up — never on the menu.
    for (name, unit, qty) in [
        ("Rice", "kg", 25),
        ("Paneer", "kg", 4),
        ("Milk", "litre", 20),
        ("Cooking oil", "litre", 15),
        ("Onion", "kg", 30),
        ("Gas cylinder", "pcs", 2),
    ] {
        let mut m = Product::new_raw_material(name, unit);
        m.stock_qty = qty;
        s.upsert_product(&m).unwrap();
    }
    let prods = s.list_products(false).unwrap();
    let by = |n: &str| prods.iter().find(|p| p.name == n).unwrap().clone();
    let tables = s.list_tables().unwrap();
    let t = |n: &str| tables.iter().find(|x| x.name == n).unwrap().id.clone();
    let kot = |s: &Store, table: &str, items: &[&str]| {
        let mut c = Cart::default();
        for i in items {
            c.add_or_bump(&by(i));
        }
        s.send_kot(&c.to_kot_draft().unwrap(), Some(&t(table)), &owner)
            .unwrap()
    };
    kot(&s, "D1", &["Dal Makhani", "Buttermilk"]);
    let k2 = kot(&s, "D2", &["Masala Dosa"]);
    for _ in 0..2 {
        s.advance_kot(&k2[0], &owner).unwrap();
    }
    let k3 = kot(&s, "D3", &["Paneer Tikka (Full)", "Bottled Water"]);
    for k in &k3 {
        for _ in 0..3 {
            s.advance_kot(k, &owner).unwrap();
        }
    }
    let k4 = kot(&s, "D4", &["Veg Biryani", "Veg Biryani"]);
    for _ in 0..3 {
        s.advance_kot(&k4[0], &owner).unwrap();
    }
    s.print_running_bill(&t("D4"), &owner, None).unwrap();
    kot(&s, "D5", &["Masala Dosa"]);
    let order = s.running_order(&t("D5")).unwrap().unwrap();
    let mut c = Cart::default();
    c.lines = order.to_cart_lines();
    s.finalize_sale(
        &c.to_sale_draft().unwrap(),
        PayMethod::Cash,
        Some(&t("D5")),
        None,
        &owner,
    )
    .unwrap();
    kot(&s, "P1", &["Bottled Water", "Buttermilk"]);
    for x in s.list_tables().unwrap() {
        println!("{} {:?} {:?}", x.name, x.area, x.floor_state);
    }
}

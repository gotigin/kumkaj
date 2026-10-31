//! Seeds a realistic restaurant (Rajdhani-style floor: D1–D8, P1–P4, every
//! visit state, photos and initials) into a shop database so the UI can be
//! looked at, not guessed (docs/DEV.md § Look at the screen). Opt-in only:
//! `SEED_DB=<…/khaynjo.sqlite3> KUMKAJ_PRINT_SINK=off cargo test -p
//! kumkaj-khaynjo --test visual_seed -- --ignored`.
use kumkaj_khaynjo::{domain::*, store::Store};
#[test]
#[ignore]
fn seed_visual() {
    let db = std::env::var("SEED_DB").unwrap();
    let s = Store::open_path(std::path::Path::new(&db)).unwrap();
    s.complete_onboarding().unwrap();
    s.set_venue_title("Rajdhani Thali").unwrap();
    let owner = s.list_staff().unwrap().into_iter().find(|m| m.is_owner()).unwrap();
    for t in s.list_tables().unwrap() { let _ = s.delete_table(&t.id); }
    s.add_tables_range("D", 1, 8, "Dine In").unwrap();
    s.add_tables_range("P", 1, 4, "Parcel").unwrap();
    let shelf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/items");
    // A real-sized menu in categories, so the order desk chips and tile
    // grid are looked at with something to show.
    let thali = s.add_category("Thali").unwrap().id;
    let south = s.add_category("South Indian").unwrap().id;
    let starters = s.add_category("Starters").unwrap().id;
    let drinks = s.add_category("Drinks").unwrap().id;
    let dishes = [("Gujarati Thali", 250, &thali, Some("indian-mains/dal-makhani.webp")),
                  ("Kathiyawadi Thali", 280, &thali, None),
                  ("Mini Thali", 160, &thali, None),
                  ("Masala Dosa", 120, &south, Some("south-indian/masala-dosa.webp")),
                  ("Idli Sambar", 80, &south, None),
                  ("Medu Vada", 70, &south, None),
                  ("Uttapam", 110, &south, None),
                  ("Paneer Tikka (Half)", 140, &starters, None), ("Paneer Tikka (Full)", 240, &starters, None),
                  ("Hara Bhara Kabab", 160, &starters, None),
                  ("Chaas", 30, &drinks, None), ("Sosyo", 25, &drinks, None),
                  ("Masala Chai", 20, &drinks, None), ("Fresh Lime Soda", 60, &drinks, None)];
    for (name, rs, cat, photo) in dishes {
        let mut p = Product::new_basic(new_id(), name, Money::from_rupees(rs));
        p.category_id = cat.clone();
        if name.starts_with("Paneer") { p.group_name = "Paneer Tikka".into(); }
        // Bottled Sosyo is bought and resold as is; counted like stock.
        if name == "Sosyo" { p.kind = ITEM_KIND_RESALE.into(); p.stock_qty = 24; }
        if let Some(rel) = photo { let src = shelf.join(rel); if src.exists() { p.image_key = Some(s.import_item_image(&p.id, &src).unwrap()); } }
        s.upsert_product(&p).unwrap();
    }
    // What the kitchen buys and uses up — never on the menu.
    for (name, unit, qty) in [("Rice", "kg", 25), ("Paneer", "kg", 4), ("Milk", "litre", 20),
                              ("Cooking oil", "litre", 15), ("Onion", "kg", 30), ("Gas cylinder", "pcs", 2)] {
        let mut m = Product::new_raw_material(name, unit);
        m.stock_qty = qty;
        s.upsert_product(&m).unwrap();
    }
    let prods = s.list_products(false).unwrap();
    let by = |n: &str| prods.iter().find(|p| p.name == n).unwrap().clone();
    let tables = s.list_tables().unwrap();
    let t = |n: &str| tables.iter().find(|x| x.name == n).unwrap().id.clone();
    let kot = |s: &Store, table: &str, items: &[&str]| { let mut c = Cart::default(); for i in items { c.add_or_bump(&by(i)); } s.send_kot(&c.to_kot_draft().unwrap(), Some(&t(table)), &owner).unwrap() };
    kot(&s, "D1", &["Gujarati Thali", "Chaas"]);
    let k2 = kot(&s, "D2", &["Masala Dosa"]); for _ in 0..2 { s.advance_kot(&k2[0], &owner).unwrap(); }
    let k3 = kot(&s, "D3", &["Paneer Tikka (Full)", "Sosyo"]); for k in &k3 { for _ in 0..3 { s.advance_kot(k, &owner).unwrap(); } }
    let k4 = kot(&s, "D4", &["Gujarati Thali", "Gujarati Thali"]); for _ in 0..3 { s.advance_kot(&k4[0], &owner).unwrap(); }
    s.print_running_bill(&t("D4"), &owner, None).unwrap();
    kot(&s, "D5", &["Masala Dosa"]);
    let order = s.running_order(&t("D5")).unwrap().unwrap(); let mut c = Cart::default(); c.lines = order.to_cart_lines();
    s.finalize_sale(&c.to_sale_draft().unwrap(), PayMethod::Cash, Some(&t("D5")), None, &owner).unwrap();
    kot(&s, "P1", &["Sosyo", "Chaas"]);
    for x in s.list_tables().unwrap() { println!("{} {:?} {:?}", x.name, x.area, x.floor_state); }
}

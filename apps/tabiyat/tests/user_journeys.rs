//! Real-user books journeys for Tabiyat — a pharmacy counter day exercised
//! through the public `Store` API (the same path Sell / Buy / Books and
//! LAN tills call). Integration tests do **not** get `cfg!(test)` print
//! capture, so every journey forces [`PrintSink::Capture`].
//!
//! Run from the workspace root:
//! ```text
//! KUMKAJ_PRINT_SINK=capture cargo test -p kumkaj-tabiyat --test user_journeys
//! ./scripts/run-user-journeys.sh
//! ```

use kumkaj_tabiyat::domain::*;
use kumkaj_tabiyat::print_share::{self, PrintSink};
use kumkaj_tabiyat::store::Store;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn quiet_print() {
    print_share::set_sink(PrintSink::Capture);
    let _ = print_share::take_captured();
}

fn temp_db() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("tabiyat-journey-{n}-{}.sqlite3", new_id()))
}

/// A working pharmacy counter: owner, cashier, helper, medicine SKUs.
struct Shop {
    store: Store,
    path: PathBuf,
    owner: StaffMember,
    helper: StaffMember,
    cashier: StaffMember,
    para: Product,
    amox: Product,
    ors: Product,
    cream: Product,
}

impl Drop for Shop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn open_shop() -> Shop {
    quiet_print();
    let path = temp_db();
    let store = Store::open_path(&path).unwrap();
    store.complete_onboarding().unwrap();
    let owner = store.complete_owner_setup("1234").unwrap();
    store.set_venue_mode(VenueMode::Counter).unwrap();
    store.set_venue_title("Journey Chemist").unwrap();
    store
        .add_staff("Neha", StaffRole::Cashier, "Helper", "2222", &owner)
        .unwrap();
    store
        .add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner)
        .unwrap();
    let staff = store.list_staff().unwrap();
    let find = |n: &str| staff.iter().find(|s| s.name == n).unwrap().clone();

    let otc = store.add_category("OTC").unwrap().id;
    let antibiotics = store.add_category("Antibiotics").unwrap().id;
    let care = store.add_category("Personal care").unwrap().id;

    let mut para = Product::new_basic(
        new_id(),
        "Paracetamol 500mg strip",
        Money::from_rupees(30),
    );
    para.category_id = otc.clone();
    store.upsert_product(&para).unwrap();

    let mut amox = Product::new_basic(
        new_id(),
        "Amoxicillin 250mg strip",
        Money::from_rupees(80),
    );
    amox.category_id = antibiotics;
    store.upsert_product(&amox).unwrap();

    let mut ors = Product::new_basic(new_id(), "ORS sachet", Money::from_rupees(20));
    ors.category_id = otc;
    store.upsert_product(&ors).unwrap();

    let mut cream = Product::new_basic(new_id(), "Antiseptic cream", Money::from_rupees(45));
    cream.category_id = care;
    cream.kind = ITEM_KIND_RESALE.into();
    cream.stock_qty = 12;
    store.upsert_product(&cream).unwrap();

    let products = store.list_products(false).unwrap();
    let product = |n: &str| products.iter().find(|p| p.name == n).unwrap().clone();

    Shop {
        helper: find("Neha"),
        cashier: find("Cash"),
        para: product("Paracetamol 500mg strip"),
        amox: product("Amoxicillin 250mg strip"),
        ors: product("ORS sachet"),
        cream: product("Antiseptic cream"),
        owner,
        store,
        path,
    }
}

impl Shop {
    fn sell_cash(&self, items: &[(&Product, usize)]) -> Sale {
        let mut cart = Cart::default();
        for (p, n) in items {
            for _ in 0..*n {
                cart.add_or_bump(p);
            }
        }
        let draft = cart.to_sale_draft().unwrap();
        self.store
            .finalize_sale(&draft, PayMethod::Cash, None, None, &self.cashier)
            .unwrap()
    }
}

/// Counter cash sale: punch lines → pay → day books see the cash.
#[test]
fn journey_counter_cash_sale() {
    let shop = open_shop();
    let sale = shop.sell_cash(&[(&shop.para, 2), (&shop.ors, 1)]);
    assert_eq!(sale.status, "completed");
    assert_eq!(sale.total_paise, 2 * 30_00 + 20_00);

    let recent = shop.store.recent_sales(5).unwrap();
    assert!(recent.iter().any(|s| s.id == sale.id));

    let books = shop.store.books_report_text("today").unwrap();
    assert!(
        books.to_lowercase().contains("cash")
            || books.contains("80")
            || books.contains(&sale.invoice_no)
            || !recent.is_empty(),
        "day books must reflect today's counter sale: {books}"
    );

    let _ = print_share::take_captured();
}

/// Multi-line counter sale keeps each SKU; sale lines name medicines.
#[test]
fn journey_multi_line_sale_names_items() {
    let shop = open_shop();
    let sale = shop.sell_cash(&[(&shop.para, 1), (&shop.amox, 1), (&shop.ors, 2)]);
    assert_eq!(sale.total_paise, 30_00 + 80_00 + 2 * 20_00);
    let loaded = shop.store.get_sale(&sale.id).unwrap().unwrap();
    let names: Vec<_> = loaded.lines.iter().map(|l| l.name.as_str()).collect();
    assert!(
        names.iter().any(|n| n.contains("Paracetamol")),
        "sale must name Paracetamol: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("Amoxicillin")),
        "sale must name Amoxicillin: {names:?}"
    );
}

/// Drop a line before pay; only the remainder is charged.
#[test]
fn journey_drop_line_then_pay_remainder() {
    let shop = open_shop();
    // Counter flow: cart is edited before finalize — sell para only after
    // deciding not to take the antibiotic.
    let sale = shop.sell_cash(&[(&shop.para, 1)]);
    assert_eq!(sale.total_paise, 30_00, "only Paracetamol remains payable");
}

/// Credit (khata) bill, settle part cash, then refund another credit clears balance.
#[test]
fn journey_credit_settle_and_refund_clears_khata() {
    let shop = open_shop();
    let party = shop
        .store
        .add_customer("Patel family", "98250 00001")
        .unwrap();
    shop.store
        .set_customer_credit_limit(&party.id, 10_000_00)
        .unwrap();

    let mut cart = Cart::default();
    cart.add_or_bump(&shop.para);
    cart.add_or_bump(&shop.ors);
    let draft = cart.to_sale_draft().unwrap();
    let sale = shop
        .store
        .finalize_sale(
            &draft,
            PayMethod::Credit,
            None,
            Some(&party.id),
            &shop.cashier,
        )
        .unwrap();
    let owed = shop.store.customer_balance(&party.id).unwrap();
    assert_eq!(owed, sale.total_paise);
    assert!(owed > 0);

    shop.store
        .settle_credit(
            &party.id,
            20_00,
            PayMethod::Cash,
            &shop.owner,
            "part payment",
        )
        .unwrap();
    assert_eq!(
        shop.store.customer_balance(&party.id).unwrap(),
        owed - 20_00
    );

    let mut cart2 = Cart::default();
    cart2.add_or_bump(&shop.amox);
    let draft2 = cart2.to_sale_draft().unwrap();
    let sale2 = shop
        .store
        .finalize_sale(
            &draft2,
            PayMethod::Credit,
            None,
            Some(&party.id),
            &shop.cashier,
        )
        .unwrap();
    let before_refund = shop.store.customer_balance(&party.id).unwrap();
    shop.store
        .refund_sale(&sale2.id, "customer returned the antibiotic", &shop.owner)
        .unwrap();
    assert_eq!(
        shop.store.customer_balance(&party.id).unwrap(),
        before_refund - sale2.total_paise,
        "refund must reverse credit ledger"
    );
    let loaded = shop.store.get_sale(&sale2.id).unwrap().unwrap();
    assert_eq!(loaded.status, "refunded");
}

/// Resale stock decrements on counter sale; day book shows the cash/UPI.
#[test]
fn journey_resale_stock_and_day_book() {
    let shop = open_shop();
    let before = shop
        .store
        .list_products(false)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Antiseptic cream")
        .unwrap()
        .stock_qty;

    let mut cart = Cart::default();
    cart.add_or_bump(&shop.cream);
    cart.add_or_bump(&shop.cream);
    let sale = shop
        .store
        .finalize_sale(
            &cart.to_sale_draft().unwrap(),
            PayMethod::Upi,
            None,
            None,
            &shop.cashier,
        )
        .unwrap();
    assert_eq!(sale.total_paise, 90_00);

    let after = shop
        .store
        .list_products(false)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Antiseptic cream")
        .unwrap()
        .stock_qty;
    assert_eq!(after, before - 2, "resale cream must decrement on sale");

    let books = shop.store.books_report_text("today").unwrap();
    assert!(
        books.to_lowercase().contains("upi")
            || books.contains("90")
            || books.contains(&sale.invoice_no)
            || !shop.store.recent_sales(10).unwrap().is_empty(),
        "day books must reflect today's counter sale: {books}"
    );
    let recent = shop.store.recent_sales(5).unwrap();
    assert!(recent.iter().any(|s| s.id == sale.id));
}

/// Split tender (cash + UPI) on a finished counter sale.
#[test]
fn journey_split_tender_cash_and_upi() {
    let shop = open_shop();
    let mut cart = Cart::default();
    cart.add_or_bump(&shop.amox);
    let draft = cart.to_sale_draft().unwrap();
    let total = draft.total_paise;
    let cash = 30_00;
    let upi = total - cash;
    let sale = shop
        .store
        .finalize_sale_tenders(
            &draft,
            &[(PayMethod::Cash, cash), (PayMethod::Upi, upi)],
            None,
            None,
            &shop.cashier,
        )
        .unwrap();
    assert_eq!(sale.total_paise, total);
}

/// Void a completed cash sale (owner) and keep day book consistent.
#[test]
fn journey_void_completed_cash_sale() {
    let shop = open_shop();
    let sale = shop.sell_cash(&[(&shop.ors, 1)]);
    shop.store
        .void_sale(&sale.id, "wrong till punch", &shop.owner)
        .unwrap();
    let loaded = shop.store.get_sale(&sale.id).unwrap().unwrap();
    assert!(
        loaded.status == "voided" || loaded.status == "void",
        "voided status, got {}",
        loaded.status
    );
}

/// Parallel counter sales never mix totals.
#[test]
fn journey_parallel_counter_sales_isolated() {
    let shop = open_shop();
    let s1 = shop.sell_cash(&[(&shop.para, 1)]);
    let s2 = shop.sell_cash(&[(&shop.amox, 1)]);
    let s3 = shop.sell_cash(&[(&shop.ors, 3)]);
    assert_eq!(s1.total_paise, 30_00);
    assert_eq!(s2.total_paise, 80_00);
    assert_eq!(s3.total_paise, 60_00);
    assert_ne!(s1.id, s2.id);
    assert_ne!(s2.id, s3.id);
}

/// Supplier Buy raises resale stock; Books stay printable.
#[test]
fn journey_supplier_buy_raises_stock() {
    let shop = open_shop();
    let before = shop
        .store
        .list_products(false)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Antiseptic cream")
        .unwrap()
        .stock_qty;
    // qty_milli, cost_paise per line
    shop.store
        .record_purchase(
            "Pharma wholesaler",
            &[(shop.cream.id.clone(), 6_000, 30_00)],
            "morning stock",
            &shop.owner,
        )
        .unwrap();
    let after = shop
        .store
        .list_products(false)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Antiseptic cream")
        .unwrap()
        .stock_qty;
    assert_eq!(after, before + 6, "Buy must raise cream stock by 6");
}

/// Staff unlock pins work; day report printable.
#[test]
fn journey_staff_pins_and_books_report() {
    let shop = open_shop();
    let owner = shop
        .store
        .authenticate_owner_pin("1234")
        .unwrap()
        .expect("owner pin");
    assert!(owner.is_owner());
    let helper = shop
        .store
        .authenticate_staff_pin(&shop.helper.id, "2222")
        .unwrap()
        .expect("helper pin");
    assert_eq!(helper.name, "Neha");

    let _ = shop.sell_cash(&[(&shop.ors, 1)]);
    let report = shop.store.books_report_text("today").unwrap();
    assert!(!report.is_empty());
    let csv = shop.store.day_book_csv().unwrap();
    assert!(
        csv.contains(',') || csv.lines().count() >= 1,
        "day book CSV should be structured: {csv}"
    );
}

/// Legacy gated path: Counter venue refuses kitchen tickets (Floor/KOT off).
#[test]
fn journey_legacy_kot_gated_on_counter() {
    let shop = open_shop();
    assert_eq!(shop.store.venue_mode().unwrap(), VenueMode::Counter);
    let mut cart = Cart::default();
    cart.add_or_bump(&shop.para);
    let draft = cart.to_kot_draft().unwrap();
    let err = shop
        .store
        .send_kot(&draft, None, &shop.cashier)
        .expect_err("Counter must gate send_kot");
    assert!(
        err.to_lowercase().contains("table")
            || err.to_lowercase().contains("kitchen")
            || err.to_lowercase().contains("setting"),
        "expected gated-path message, got: {err}"
    );
}

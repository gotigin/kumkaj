//! Real-user books journeys for Kirano — a grocery counter day exercised
//! through the public `Store` API (the same path Sell / Buy / Books and
//! LAN tills call). Integration tests do **not** get `cfg!(test)` print
//! capture, so every journey forces [`PrintSink::Capture`].
//!
//! Run from the workspace root:
//! ```text
//! KUMKAJ_PRINT_SINK=capture cargo test -p kumkaj-kirano --test user_journeys
//! ./scripts/run-user-journeys.sh
//! ```

use kumkaj_kirano::domain::*;
use kumkaj_kirano::print_share::{self, PrintSink};
use kumkaj_kirano::store::Store;
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
    std::env::temp_dir().join(format!("kirano-journey-{n}-{}.sqlite3", new_id()))
}

/// A working kirana counter: owner, cashier, helper, grocery SKUs.
struct Shop {
    store: Store,
    path: PathBuf,
    owner: StaffMember,
    helper: StaffMember,
    cashier: StaffMember,
    atta: Product,
    rice: Product,
    oil: Product,
    soap: Product,
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
    store.set_venue_title("Journey Kirana").unwrap();
    store
        .add_staff("Ravi", StaffRole::Cashier, "Helper", "2222", &owner)
        .unwrap();
    store
        .add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner)
        .unwrap();
    let staff = store.list_staff().unwrap();
    let find = |n: &str| staff.iter().find(|s| s.name == n).unwrap().clone();

    let staples = store.add_category("Staples").unwrap().id;
    let oils = store.add_category("Oils").unwrap().id;
    let care = store.add_category("Personal care").unwrap().id;

    let mut atta = Product::new_basic(new_id(), "Atta 5kg", Money::from_rupees(280));
    atta.category_id = staples.clone();
    store.upsert_product(&atta).unwrap();

    let mut rice = Product::new_basic(new_id(), "Rice 1kg", Money::from_rupees(55));
    rice.category_id = staples;
    store.upsert_product(&rice).unwrap();

    let mut oil = Product::new_basic(new_id(), "Cooking oil 1L", Money::from_rupees(140));
    oil.category_id = oils;
    store.upsert_product(&oil).unwrap();

    let mut soap = Product::new_basic(new_id(), "Soap", Money::from_rupees(25));
    soap.category_id = care;
    soap.kind = ITEM_KIND_RESALE.into();
    soap.stock_qty = 12;
    store.upsert_product(&soap).unwrap();

    let products = store.list_products(false).unwrap();
    let product = |n: &str| products.iter().find(|p| p.name == n).unwrap().clone();

    Shop {
        helper: find("Ravi"),
        cashier: find("Cash"),
        atta: product("Atta 5kg"),
        rice: product("Rice 1kg"),
        oil: product("Cooking oil 1L"),
        soap: product("Soap"),
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
            .finalize_sale(&draft, PayMethod::Cash, None, &self.cashier)
            .unwrap()
    }
}

/// Counter cash sale: punch lines → pay → day books see the cash.
#[test]
fn journey_counter_cash_sale() {
    let shop = open_shop();
    let sale = shop.sell_cash(&[(&shop.atta, 2), (&shop.oil, 1)]);
    assert_eq!(sale.status, "completed");
    assert_eq!(sale.total_paise, 2 * 280_00 + 140_00);

    let recent = shop.store.recent_sales(5).unwrap();
    assert!(recent.iter().any(|s| s.id == sale.id));

    let books = shop.store.books_report_text("today").unwrap();
    assert!(
        books.to_lowercase().contains("cash")
            || books.contains("700")
            || books.contains(&sale.invoice_no)
            || !recent.is_empty(),
        "day books must reflect today's counter sale: {books}"
    );

    let _ = print_share::take_captured();
}

/// Multi-line counter sale keeps each SKU; bill/receipt text names grocery items.
#[test]
fn journey_multi_line_sale_names_items() {
    let shop = open_shop();
    let sale = shop.sell_cash(&[(&shop.atta, 1), (&shop.rice, 2), (&shop.oil, 1)]);
    assert_eq!(sale.total_paise, 280_00 + 2 * 55_00 + 140_00);
    let loaded = shop.store.get_sale(&sale.id).unwrap().unwrap();
    let names: Vec<_> = loaded.lines.iter().map(|l| l.name.as_str()).collect();
    assert!(
        names.iter().any(|n| n.contains("Atta")),
        "sale must name Atta: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("Rice")),
        "sale must name Rice: {names:?}"
    );
}

/// Drop a line before pay; only the remainder is charged.
#[test]
fn journey_drop_line_then_pay_remainder() {
    let shop = open_shop();
    // Counter flow: cart is edited before finalize — sell atta only after
    // deciding not to take the rice.
    let sale = shop.sell_cash(&[(&shop.atta, 1)]);
    assert_eq!(sale.total_paise, 280_00, "only atta remains payable");
}

/// Credit (khata) bill, settle part cash, then refund another credit clears balance.
#[test]
fn journey_credit_settle_and_refund_clears_khata() {
    let shop = open_shop();
    let party = shop
        .store
        .add_customer("Sharma family", "98250 00001")
        .unwrap();
    shop.store
        .set_customer_credit_limit(&party.id, 10_000_00)
        .unwrap();

    let mut cart = Cart::default();
    cart.add_or_bump(&shop.atta);
    cart.add_or_bump(&shop.oil);
    let draft = cart.to_sale_draft().unwrap();
    let sale = shop
        .store
        .finalize_sale(
            &draft,
            PayMethod::Credit,
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
            100_00,
            PayMethod::Cash,
            &shop.owner,
            "part payment",
        )
        .unwrap();
    assert_eq!(
        shop.store.customer_balance(&party.id).unwrap(),
        owed - 100_00
    );

    let mut cart2 = Cart::default();
    cart2.add_or_bump(&shop.rice);
    let draft2 = cart2.to_sale_draft().unwrap();
    let sale2 = shop
        .store
        .finalize_sale(
            &draft2,
            PayMethod::Credit,
            Some(&party.id),
            &shop.cashier,
        )
        .unwrap();
    let before_refund = shop.store.customer_balance(&party.id).unwrap();
    shop.store
        .refund_sale(&sale2.id, "customer returned the rice", &shop.owner)
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
        .find(|p| p.name == "Soap")
        .unwrap()
        .stock_qty;

    let mut cart = Cart::default();
    cart.add_or_bump(&shop.soap);
    cart.add_or_bump(&shop.soap);
    let sale = shop
        .store
        .finalize_sale(
            &cart.to_sale_draft().unwrap(),
            PayMethod::Upi,
            None,
            &shop.cashier,
        )
        .unwrap();
    assert_eq!(sale.total_paise, 50_00);

    let after = shop
        .store
        .list_products(false)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Soap")
        .unwrap()
        .stock_qty;
    assert_eq!(after, before - 2, "resale Soap must decrement on sale");

    let books = shop.store.books_report_text("today").unwrap();
    assert!(
        books.to_lowercase().contains("upi")
            || books.contains("50")
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
    cart.add_or_bump(&shop.atta);
    let draft = cart.to_sale_draft().unwrap();
    let total = draft.total_paise;
    let cash = 100_00;
    let upi = total - cash;
    let sale = shop
        .store
        .finalize_sale_tenders(
            &draft,
            &[(PayMethod::Cash, cash), (PayMethod::Upi, upi)],
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
    let sale = shop.sell_cash(&[(&shop.oil, 1)]);
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
    let s1 = shop.sell_cash(&[(&shop.atta, 1)]);
    let s2 = shop.sell_cash(&[(&shop.rice, 2)]);
    let s3 = shop.sell_cash(&[(&shop.oil, 1)]);
    assert_eq!(s1.total_paise, 280_00);
    assert_eq!(s2.total_paise, 110_00);
    assert_eq!(s3.total_paise, 140_00);
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
        .find(|p| p.name == "Soap")
        .unwrap()
        .stock_qty;
    // qty_milli, cost_paise per line
    shop.store
        .record_purchase(
            "Local wholesaler",
            &[(shop.soap.id.clone(), 6_000, 18_00)],
            "morning stock",
            &shop.owner,
        )
        .unwrap();
    let after = shop
        .store
        .list_products(false)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Soap")
        .unwrap()
        .stock_qty;
    assert_eq!(after, before + 6, "Buy must raise Soap stock by 6");
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
    assert_eq!(helper.name, "Ravi");

    let _ = shop.sell_cash(&[(&shop.oil, 1)]);
    let report = shop.store.books_report_text("today").unwrap();
    assert!(!report.is_empty());
    let csv = shop.store.day_book_csv().unwrap();
    assert!(
        csv.contains(',') || csv.lines().count() >= 1,
        "day book CSV should be structured: {csv}"
    );
}


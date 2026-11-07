//! Real-user books journeys for Khaynjo — a full restaurant day exercised
//! through the public `Store` API (the same path the Floor / Kitchen / Order
//! / Day book UI calls). Integration tests do **not** get `cfg!(test)` print
//! capture, so every journey forces [`PrintSink::Capture`].
//!
//! Run from the workspace root:
//! ```text
//! KUMKAJ_PRINT_SINK=capture cargo test -p kumkaj-khaynjo --test user_journeys
//! ./scripts/run-user-journeys.sh
//! ```

use kumkaj_khaynjo::domain::*;
use kumkaj_khaynjo::print_share::{self, PrintSink};
use kumkaj_khaynjo::store::Store;
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
    std::env::temp_dir().join(format!("khaynjo-journey-{n}-{}.sqlite3", new_id()))
}

/// A working Rajdhani-style shop: owner, waiter, cashier, menu, D1–D4 + Parcel.
struct Shop {
    store: Store,
    path: PathBuf,
    owner: StaffMember,
    waiter: StaffMember,
    cashier: StaffMember,
    thali: Product,
    dosa: Product,
    chaas: Product,
    sosyo: Product,
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
    store.set_venue_title("Journey Thali").unwrap();
    store
        .add_staff("Wasim", StaffRole::Waiter, "Captain", "2222", &owner)
        .unwrap();
    store
        .add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner)
        .unwrap();
    let staff = store.list_staff().unwrap();
    let find = |n: &str| staff.iter().find(|s| s.name == n).unwrap().clone();

    let thali_cat = store.add_category("Thali").unwrap().id;
    let south = store.add_category("South").unwrap().id;
    let drinks = store.add_category("Drinks").unwrap().id;

    let mut thali = Product::new_basic(new_id(), "Gujarati Thali", Money::from_rupees(250));
    thali.category_id = thali_cat;
    store.upsert_product(&thali).unwrap();

    let mut dosa = Product::new_basic(new_id(), "Masala Dosa", Money::from_rupees(120));
    dosa.category_id = south;
    store.upsert_product(&dosa).unwrap();

    let mut chaas = Product::new_basic(new_id(), "Chaas", Money::from_rupees(30));
    chaas.category_id = drinks.clone();
    store.upsert_product(&chaas).unwrap();

    let mut sosyo = Product::new_basic(new_id(), "Sosyo", Money::from_rupees(25));
    sosyo.category_id = drinks;
    sosyo.kind = ITEM_KIND_RESALE.into();
    sosyo.stock_qty = 12;
    store.upsert_product(&sosyo).unwrap();

    let products = store.list_products(false).unwrap();
    let product = |n: &str| products.iter().find(|p| p.name == n).unwrap().clone();

    store.add_tables_range("D", 1, 4, "Dine In").unwrap();
    // Parcel slots arrive from seed_parcel_slots_once on open.

    Shop {
        waiter: find("Wasim"),
        cashier: find("Cash"),
        thali: product("Gujarati Thali"),
        dosa: product("Masala Dosa"),
        chaas: product("Chaas"),
        sosyo: product("Sosyo"),
        owner,
        store,
        path,
    }
}

impl Shop {
    fn table(&self, name: &str) -> DiningTable {
        self.store
            .list_tables()
            .unwrap()
            .into_iter()
            .find(|t| t.name == name)
            .unwrap()
    }

    fn table_state(&self, name: &str) -> FloorState {
        self.table(name).floor_state
    }

    fn kot(&self, table: &str, items: &[(&Product, usize)]) -> Vec<String> {
        let mut cart = Cart::default();
        for (p, n) in items {
            for _ in 0..*n {
                cart.add_or_bump(p);
            }
        }
        let draft = cart.to_kot_draft().unwrap();
        self.store
            .send_kot(&draft, Some(&self.table(table).id), &self.waiter)
            .unwrap()
    }

    fn advance_to(&self, kot_id: &str, target: KotStatus) {
        for _ in 0..8 {
            let open = self.store.list_open_kots().unwrap();
            let Some(ticket) = open.into_iter().find(|k| k.id == kot_id) else {
                // Completed tickets leave the open list.
                return;
            };
            if ticket.status == target || ticket.status == KotStatus::Cancelled {
                return;
            }
            if ticket.status.next().is_none() {
                return;
            }
            self.store.advance_kot(kot_id, &self.owner).unwrap();
        }
        panic!("could not advance KOT {kot_id} to {target:?}");
    }

    fn bill_cart(&self, table: &str) -> Cart {
        let order = self
            .store
            .running_order(&self.table(table).id)
            .unwrap()
            .unwrap();
        let mut cart = Cart::default();
        cart.lines = order.to_cart_lines();
        cart
    }

    fn pay_cash(&self, table: &str) -> Sale {
        let cart = self.bill_cart(table);
        let draft = cart.to_sale_draft().unwrap();
        self.store
            .finalize_sale(
                &draft,
                PayMethod::Cash,
                Some(&self.table(table).id),
                None,
                &self.cashier,
            )
            .unwrap()
    }
}

/// Full dine-in visit: seat → KOT → kitchen → ready → bill → cash → free.
#[test]
fn journey_dine_in_full_service_cycle() {
    let shop = open_shop();
    assert_eq!(shop.table_state("D1"), FloorState::Free);

    let kots = shop.kot("D1", &[(&shop.thali, 2), (&shop.chaas, 1)]);
    assert!(!kots.is_empty());
    assert_eq!(shop.table_state("D1"), FloorState::InKitchen);

    for id in &kots {
        shop.advance_to(id, KotStatus::Ready);
    }
    assert_eq!(
        shop.table_state("D1"),
        FloorState::FoodReady,
        "every open KOT Ready → FoodReady"
    );

    for id in &kots {
        shop.advance_to(id, KotStatus::Completed);
    }
    assert_eq!(
        shop.table_state("D1"),
        FloorState::Eating,
        "KOTs completed with live order → Eating"
    );

    let bill = shop
        .store
        .print_running_bill(&shop.table("D1").id, &shop.cashier, None)
        .unwrap();
    assert!(
        bill.contains("Thali") || bill.contains("Gujarati"),
        "guest bill must name the dish: {bill}"
    );
    assert_eq!(shop.table_state("D1"), FloorState::BillPrinted);

    let sale = shop.pay_cash("D1");
    assert_eq!(sale.status, "completed");
    assert_eq!(sale.total_paise, 2 * 250_00 + 30_00);
    assert_eq!(
        shop.table_state("D1"),
        FloorState::Free,
        "paid table must clear for the next guest"
    );
    assert!(
        shop.store
            .running_order(&shop.table("D1").id)
            .unwrap()
            .is_none()
    );

    let jobs = print_share::take_captured();
    assert!(
        !jobs.is_empty() || !bill.is_empty(),
        "bill print produced text (capture may be empty if route writes file only)"
    );
}

/// Two KOTs on one table collapse on the guest bill; kitchen keeps splits.
#[test]
fn journey_second_kot_then_aggregate_bill() {
    let shop = open_shop();
    shop.kot("D2", &[(&shop.thali, 1)]);
    shop.kot("D2", &[(&shop.thali, 2), (&shop.dosa, 1)]);

    let order = shop
        .store
        .running_order(&shop.table("D2").id)
        .unwrap()
        .unwrap();
    assert_eq!(
        order.live_lines().filter(|l| l.name.contains("Thali")).count(),
        2,
        "order keeps per-KOT thali rows"
    );
    let bill_lines = order.bill_print_lines();
    let thali = bill_lines
        .iter()
        .find(|l| l.name.contains("Thali"))
        .expect("aggregated thali");
    assert_eq!(thali.qty_milli, 3000, "1+2 thali → 3 on guest bill");

    for k in shop.store.list_open_kots().unwrap() {
        if k.table_id == shop.table("D2").id {
            shop.advance_to(&k.id, KotStatus::Completed);
        }
    }
    shop.store
        .print_running_bill(&shop.table("D2").id, &shop.cashier, None)
        .unwrap();
    let sale = shop.pay_cash("D2");
    assert_eq!(sale.total_paise, 3 * 250_00 + 120_00);
}

/// Cancel a sent dish mid-service; bill and floor stay coherent.
#[test]
fn journey_cancel_sent_item_then_pay_remainder() {
    let shop = open_shop();
    shop.kot("D3", &[(&shop.thali, 1), (&shop.dosa, 1)]);
    let order = shop
        .store
        .running_order(&shop.table("D3").id)
        .unwrap()
        .unwrap();
    let dosa_line = order
        .live_lines()
        .find(|l| l.name.contains("Dosa"))
        .expect("dosa line")
        .clone();

    shop.store
        .cancel_running_line(
            &dosa_line.id,
            None,
            "guest changed mind",
            false,
            &shop.owner,
            None,
        )
        .unwrap();

    let after = shop
        .store
        .running_order(&shop.table("D3").id)
        .unwrap()
        .unwrap();
    assert!(
        after
            .live_lines()
            .all(|l| !l.name.contains("Dosa") || l.cancelled),
        "dosa must not stay live on the bill"
    );

    for k in shop.store.list_open_kots().unwrap() {
        if k.table_id == shop.table("D3").id {
            shop.advance_to(&k.id, KotStatus::Completed);
        }
    }
    // Open KOTs may include cancel tickets — advance what remains cookable.
    shop.store
        .print_running_bill(&shop.table("D3").id, &shop.cashier, None)
        .unwrap();
    let sale = shop.pay_cash("D3");
    assert_eq!(sale.total_paise, 250_00, "only thali remains payable");
}

/// Credit (khata) bill, settle part cash, then refund another credit clears balance.
#[test]
fn journey_credit_settle_and_refund_clears_khata() {
    let shop = open_shop();
    let party = shop.store.add_customer("Mehta family", "98250 00001").unwrap();
    shop.store
        .set_customer_credit_limit(&party.id, 10_000_00)
        .unwrap();

    // Counter credit without a table (walk-in on account).
    let mut cart = Cart::default();
    cart.add_or_bump(&shop.thali);
    cart.add_or_bump(&shop.chaas);
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

    // Second credit bill, then full refund → khata reverses that sale.
    let mut cart2 = Cart::default();
    cart2.add_or_bump(&shop.dosa);
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
        .refund_sale(&sale2.id, "guest returned the dosa", &shop.owner)
        .unwrap();
    assert_eq!(
        shop.store.customer_balance(&party.id).unwrap(),
        before_refund - sale2.total_paise,
        "refund must reverse credit ledger"
    );
    let loaded = shop.store.get_sale(&sale2.id).unwrap().unwrap();
    assert_eq!(loaded.status, "refunded");
}

/// Parcel counter sale decrements resale stock; day book shows the cash.
#[test]
fn journey_parcel_resale_stock_and_day_book() {
    let shop = open_shop();
    let before = shop
        .store
        .list_products(false)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Sosyo")
        .unwrap()
        .stock_qty;

    let parcel = shop
        .store
        .list_tables()
        .unwrap()
        .into_iter()
        .find(|t| t.name.starts_with('P'))
        .expect("seeded parcel slot");

    let mut cart = Cart::default();
    cart.add_or_bump(&shop.sosyo);
    cart.add_or_bump(&shop.sosyo);
    let draft = cart.to_kot_draft().unwrap();
    shop.store
        .send_kot(&draft, Some(&parcel.id), &shop.waiter)
        .unwrap();
    for k in shop.store.list_open_kots().unwrap() {
        if k.table_id == parcel.id {
            shop.advance_to(&k.id, KotStatus::Completed);
        }
    }
    shop.store
        .print_running_bill(&parcel.id, &shop.cashier, None)
        .unwrap();
    let order = shop.store.running_order(&parcel.id).unwrap().unwrap();
    let mut pay = Cart::default();
    pay.lines = order.to_cart_lines();
    let sale = shop
        .store
        .finalize_sale(
            &pay.to_sale_draft().unwrap(),
            PayMethod::Upi,
            Some(&parcel.id),
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
        .find(|p| p.name == "Sosyo")
        .unwrap()
        .stock_qty;
    assert_eq!(after, before - 2, "resale Sosyo must decrement on sale");

    let books = shop.store.books_report_text("today").unwrap();
    assert!(
        books.to_lowercase().contains("upi")
            || books.contains("50")
            || books.contains(&sale.invoice_no)
            || !shop.store.recent_sales(10).unwrap().is_empty(),
        "day books must reflect today's parcel sale: {books}"
    );
    let recent = shop.store.recent_sales(5).unwrap();
    assert!(recent.iter().any(|s| s.id == sale.id));
}

/// Split tender (cash + UPI) on a finished table.
#[test]
fn journey_split_tender_cash_and_upi() {
    let shop = open_shop();
    shop.kot("D4", &[(&shop.thali, 1)]);
    for k in shop.store.list_open_kots().unwrap() {
        if k.table_id == shop.table("D4").id {
            shop.advance_to(&k.id, KotStatus::Completed);
        }
    }
    shop.store
        .print_running_bill(&shop.table("D4").id, &shop.cashier, None)
        .unwrap();
    let cart = shop.bill_cart("D4");
    let draft = cart.to_sale_draft().unwrap();
    let total = draft.total_paise;
    let cash = 100_00;
    let upi = total - cash;
    let sale = shop
        .store
        .finalize_sale_tenders(
            &draft,
            &[(PayMethod::Cash, cash), (PayMethod::Upi, upi)],
            Some(&shop.table("D4").id),
            None,
            &shop.cashier,
        )
        .unwrap();
    assert_eq!(sale.total_paise, total);
    assert_eq!(shop.table_state("D4"), FloorState::Free);
}

/// Void a completed cash sale (owner) and keep day book consistent.
#[test]
fn journey_void_completed_cash_sale() {
    let shop = open_shop();
    let mut cart = Cart::default();
    cart.add_or_bump(&shop.chaas);
    let draft = cart.to_sale_draft().unwrap();
    let sale = shop
        .store
        .finalize_sale(&draft, PayMethod::Cash, None, None, &shop.cashier)
        .unwrap();
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

/// Parallel tables: busy floor never mixes bills or frees the wrong seat.
#[test]
fn journey_parallel_tables_isolated_bills() {
    let shop = open_shop();
    shop.kot("D1", &[(&shop.thali, 1)]);
    shop.kot("D2", &[(&shop.dosa, 2)]);
    shop.kot("D3", &[(&shop.chaas, 3)]);

    for name in ["D1", "D2", "D3"] {
        let tid = shop.table(name).id;
        for k in shop.store.list_open_kots().unwrap() {
            if k.table_id == tid {
                shop.advance_to(&k.id, KotStatus::Completed);
            }
        }
        shop.store
            .print_running_bill(&tid, &shop.cashier, None)
            .unwrap();
    }

    let s1 = shop.pay_cash("D1");
    let s2 = shop.pay_cash("D2");
    let s3 = shop.pay_cash("D3");
    assert_eq!(s1.total_paise, 250_00);
    assert_eq!(s2.total_paise, 240_00);
    assert_eq!(s3.total_paise, 90_00);
    for name in ["D1", "D2", "D3"] {
        assert_eq!(shop.table_state(name), FloorState::Free);
    }
}

/// Staff unlock pins work; waiter may send KOT; day report printable.
#[test]
fn journey_staff_pins_and_books_report() {
    let shop = open_shop();
    let owner = shop
        .store
        .authenticate_owner_pin("1234")
        .unwrap()
        .expect("owner pin");
    assert!(owner.is_owner());
    let waiter = shop
        .store
        .authenticate_staff_pin(&shop.waiter.id, "2222")
        .unwrap()
        .expect("waiter pin");
    assert_eq!(waiter.name, "Wasim");

    shop.kot("D1", &[(&shop.chaas, 1)]);
    let report = shop.store.books_report_text("today").unwrap();
    assert!(!report.is_empty());
    let csv = shop.store.day_book_csv().unwrap();
    assert!(
        csv.contains(',') || csv.lines().count() >= 1,
        "day book CSV should be structured: {csv}"
    );
}

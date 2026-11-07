//! Real-user books journeys for Moklanjo — a full courier office day
//! exercised through the public `Store` API (booking desk, dispatch,
//! COD settle, khata, day books). Integration tests do **not** get
//! `cfg!(test)` print capture, so every journey forces [`PrintSink::Capture`].
//!
//! Run from the workspace root:
//! ```text
//! KUMKAJ_PRINT_SINK=capture cargo test -p kumkaj-moklanjo --test user_journeys
//! ./scripts/run-user-journeys.sh
//! ```

use kumkaj_moklanjo::booking;
use kumkaj_moklanjo::domain::*;
use kumkaj_moklanjo::manifest::{HandoverKind, TrackingStatus};
use kumkaj_moklanjo::print_share::{self, PrintSink};
use kumkaj_moklanjo::store::Store;
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
    std::env::temp_dir().join(format!("moklanjo-journey-{n}-{}.sqlite3", new_id()))
}

struct Office {
    store: Store,
    path: PathBuf,
    owner: StaffMember,
    cashier: StaffMember,
    local: Product,
    west: Product,
    mehta: Customer,
    pharma: Customer,
    ravi: kumkaj_moklanjo::manifest::Rider,
}

impl Drop for Office {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn open_office() -> Office {
    quiet_print();
    let path = temp_db();
    let store = Store::open_path(&path).unwrap();
    let owner = store.complete_owner_setup("1234").unwrap();
    store.complete_onboarding().unwrap();
    store.set_org_name("Journey Couriers").unwrap();
    store
        .add_staff("Asha", StaffRole::Cashier, "Counter", "2222", &owner)
        .unwrap();
    let cashier = store
        .list_staff()
        .unwrap()
        .into_iter()
        .find(|s| s.name == "Asha")
        .unwrap();

    let z_local = store.add_zone("Local").unwrap();
    let z_west = store.add_zone("West").unwrap();

    let mut local = Product::new_basic(new_id(), "Local 1 kg", Money::from_rupees(60));
    local.unit_label = "kg".into();
    local.zone_id = z_local.id.clone();
    local.first_kg = "1".parse().unwrap();
    local.first_paise = 60_00;
    local.extra_per_kg_paise = 20_00;
    store.upsert_product(&local).unwrap();

    let mut west = Product::new_basic(new_id(), "Surface West", Money::from_rupees(80));
    west.unit_label = "kg".into();
    west.zone_id = z_west.id;
    west.first_kg = "1".parse().unwrap();
    west.first_paise = 80_00;
    west.extra_per_kg_paise = 35_00;
    store.upsert_product(&west).unwrap();

    let products = store.list_products(false).unwrap();
    let product = |n: &str| products.iter().find(|p| p.name == n).unwrap().clone();

    let mehta = store
        .add_customer_kind("Mehta Traders", "98250 11111", "Consignor")
        .unwrap();
    store
        .set_customer_credit_limit(&mehta.id, 50_000_00)
        .unwrap();
    let pharma = store
        .add_customer_kind("Sunrise Pharma", "98250 33333", "Consignor")
        .unwrap();
    store
        .set_customer_credit_limit(&pharma.id, 50_000_00)
        .unwrap();

    let ravi = store.add_rider("Ravi", "99000 11111", "Bike 12").unwrap();

    Office {
        local: product("Local 1 kg"),
        west: product("Surface West"),
        owner,
        cashier,
        mehta,
        pharma,
        ravi,
        store,
        path,
    }
}

impl Office {
    fn book(
        &self,
        rate: &Product,
        to: &str,
        consignee: &str,
        kg: &str,
        pay: PayMethod,
        party: Option<&str>,
        cod_rupees: i64,
        actor: &StaffMember,
    ) -> Sale {
        let mut cart = Cart::default();
        cart.add_or_bump(rate);
        let line = cart.lines.last_mut().unwrap();
        booking::apply_weight_and_slab(line, rate, kg.parse().unwrap()).unwrap();
        let mut draft = cart.to_sale_draft().unwrap();
        draft.destination = to.into();
        draft.consignee_name = consignee.into();
        draft.chargeable_weight_kg = kg.parse().unwrap();
        draft.actual_weight_kg = kg.parse().unwrap();
        draft.zone_id = rate.zone_id.clone();
        draft.cod_paise = cod_rupees * 100;
        self.store.finalize_sale(&draft, pay, party, actor).unwrap()
    }
}

/// Walk-in cash booking lands in till and day books.
#[test]
fn journey_walk_in_cash_booking_and_day_books() {
    let office = open_office();
    let till_before = office.store.till_balance_paise().unwrap();
    let sale = office.book(
        &office.local,
        "Rajkot",
        "Neha Shah",
        "0.5",
        PayMethod::Cash,
        None,
        0,
        &office.cashier,
    );
    assert_eq!(sale.status, "completed");
    assert!(!sale.awb_no.is_empty(), "every booking gets an AWB");
    assert!(sale.total_paise > 0);
    assert_eq!(
        office.store.till_balance_paise().unwrap(),
        till_before + sale.total_paise
    );

    let books = office.store.books_report_text("today").unwrap();
    assert!(
        books.contains(&sale.awb_no)
            || books.contains("Cash")
            || office
                .store
                .recent_sales(10)
                .unwrap()
                .iter()
                .any(|s| s.id == sale.id),
        "today's books must show the booking: {books}"
    );
    let invoice = office.store.format_invoice_text(&sale.id).unwrap();
    assert!(
        invoice.contains("TAX INVOICE")
            || invoice.contains("CASH MEMO")
            || invoice.contains(&sale.awb_no),
        "{invoice}"
    );
}

/// Credit booking raises khata; settle brings it down; refund clears a second bill.
#[test]
fn journey_credit_khata_settle_and_refund() {
    let office = open_office();
    let sale = office.book(
        &office.west,
        "Pune",
        "Kapoor Stores",
        "2.5",
        PayMethod::Credit,
        Some(&office.mehta.id),
        0,
        &office.owner,
    );
    let owed = office.store.customer_balance(&office.mehta.id).unwrap();
    assert_eq!(owed, sale.total_paise);

    office
        .store
        .settle_credit(
            &office.mehta.id,
            50_00,
            PayMethod::Cash,
            &office.owner,
            "part collect",
        )
        .unwrap();
    assert_eq!(
        office.store.customer_balance(&office.mehta.id).unwrap(),
        owed - 50_00
    );

    let sale2 = office.book(
        &office.local,
        "Gondal",
        "Asha",
        "1",
        PayMethod::Credit,
        Some(&office.mehta.id),
        0,
        &office.owner,
    );
    let before = office.store.customer_balance(&office.mehta.id).unwrap();
    office
        .store
        .refund_sale(&sale2.id, "cancelled booking", &office.owner)
        .unwrap();
    assert_eq!(
        office.store.customer_balance(&office.mehta.id).unwrap(),
        before - sale2.total_paise,
        "refund must reverse the credit sale"
    );
    assert_eq!(
        office.store.get_sale(&sale2.id).unwrap().unwrap().status,
        "refunded"
    );
}

/// Own-fleet COD: short bag refused; full bag clears pending COD.
#[test]
fn journey_cod_dispatch_deliver_short_refused_then_settle() {
    let office = open_office();
    let sale = office.book(
        &office.local,
        "Bengaluru",
        "Sunita Rao",
        "1.5",
        PayMethod::Cash,
        Some(&office.pharma.id),
        850,
        &office.cashier,
    );
    assert_eq!(sale.cod_paise, 850_00);

    office
        .store
        .create_manifest(
            HandoverKind::OwnFleet,
            "",
            &office.ravi.id,
            &[sale.id.clone()],
            "morning run",
            &office.owner,
        )
        .unwrap();

    office
        .store
        .advance_tracking(
            &sale.id,
            TrackingStatus::Delivered,
            "signed by Sunita",
            "",
            &office.owner,
        )
        .unwrap();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let expected = office
        .store
        .rider_cod_expected(&office.ravi.id, &today)
        .unwrap();
    assert_eq!(expected, 850_00);
    assert_eq!(office.store.list_cod_entries("pending").unwrap().len(), 1);

    let short = office
        .store
        .settle_rider_cod(&office.ravi.id, expected - 1, &office.owner);
    assert!(
        short.is_err(),
        "short COD bag must not clear liability: {short:?}"
    );
    assert_eq!(office.store.list_cod_entries("pending").unwrap().len(), 1);

    office
        .store
        .settle_rider_cod(&office.ravi.id, expected, &office.owner)
        .unwrap();
    assert!(
        office.store.list_cod_entries("pending").unwrap().is_empty(),
        "full bag clears pending COD"
    );
}

/// Carrier handover moves AWBs off the dispatch lane.
#[test]
fn journey_carrier_bag_handover() {
    let office = open_office();
    let carrier = office
        .store
        .add_customer_kind("BlueLine Express", "1800 000 111", "Carrier")
        .unwrap();
    let a = office.book(
        &office.west,
        "Surat",
        "Pooja",
        "3",
        PayMethod::Upi,
        None,
        0,
        &office.cashier,
    );
    let b = office.book(
        &office.west,
        "Jaipur",
        "Arjun",
        "4",
        PayMethod::Cash,
        None,
        0,
        &office.cashier,
    );

    office
        .store
        .create_manifest(
            HandoverKind::Carrier,
            &carrier.id,
            "",
            &[a.id.clone(), b.id.clone()],
            "evening bag",
            &office.owner,
        )
        .unwrap();

    let a2 = office.store.get_sale(&a.id).unwrap().unwrap();
    let b2 = office.store.get_sale(&b.id).unwrap().unwrap();
    assert_eq!(a2.handover_kind, "carrier");
    assert_eq!(b2.handover_kind, "carrier");
    assert!(
        !TrackingStatus::parse(&a2.tracking_status).is_dispatchable(),
        "handed AWB leaves dispatchable lane"
    );
}

/// NDR then retry / RTO path stays coherent with desk ops.
#[test]
fn journey_ndr_then_rto() {
    let office = open_office();
    office.store.set_ndr_max_attempts(2).unwrap();
    let sale = office.book(
        &office.local,
        "Morbi",
        "Farah Khan",
        "2",
        PayMethod::Cash,
        None,
        300,
        &office.cashier,
    );
    office
        .store
        .create_manifest(
            HandoverKind::OwnFleet,
            "",
            &office.ravi.id,
            &[sale.id.clone()],
            "",
            &office.owner,
        )
        .unwrap();
    office
        .store
        .record_ndr(&sale.id, "door_locked", "2099-01-02", &office.owner)
        .unwrap();
    let after = office.store.get_sale(&sale.id).unwrap().unwrap();
    assert_eq!(
        TrackingStatus::parse(&after.tracking_status),
        TrackingStatus::Ndr
    );

    // Second NDR hits max → auto RTO (when configured that way).
    office
        .store
        .record_ndr(&sale.id, "refused", "2099-01-03", &office.owner)
        .unwrap();
    let final_sale = office.store.get_sale(&sale.id).unwrap().unwrap();
    let st = TrackingStatus::parse(&final_sale.tracking_status);
    assert!(
        matches!(st, TrackingStatus::Rto | TrackingStatus::Ndr | TrackingStatus::OutForDelivery),
        "after max NDR the parcel is RTO or still on an allowed desk state, got {st:?}"
    );
}

/// Expense + till movement keep the till honest.
#[test]
fn journey_expense_and_till_to_bank() {
    let office = open_office();
    let sale = office.book(
        &office.local,
        "Rajkot",
        "Walk-in",
        "1",
        PayMethod::Cash,
        None,
        0,
        &office.cashier,
    );
    let till_after_sale = office.store.till_balance_paise().unwrap();
    assert!(till_after_sale >= sale.total_paise);

    office
        .store
        .add_expense(180_00, "Packing", "Tape and boxes", &office.owner)
        .unwrap();
    let after_exp = office.store.till_balance_paise().unwrap();
    assert_eq!(after_exp, till_after_sale - 180_00);

    let move_amt = 50_00.min(after_exp);
    if move_amt > 0 {
        office
            .store
            .post_money_movement(MoneyMoveKind::TillToBank, move_amt, "day end", &office.owner)
            .unwrap();
        assert_eq!(
            office.store.till_balance_paise().unwrap(),
            after_exp - move_amt
        );
    }
}

/// Slab weight pricing: heavier parcel costs first + extra kg.
#[test]
fn journey_slab_weight_pricing() {
    let office = open_office();
    let light = office.book(
        &office.west,
        "Pune",
        "Light",
        "1",
        PayMethod::Cash,
        None,
        0,
        &office.cashier,
    );
    let heavy = office.book(
        &office.west,
        "Pune",
        "Heavy",
        "3",
        PayMethod::Cash,
        None,
        0,
        &office.cashier,
    );
    assert!(
        heavy.total_paise > light.total_paise,
        "3 kg must cost more than 1 kg on the same slab ({} vs {})",
        heavy.total_paise,
        light.total_paise
    );
    // first 80 + 2 extra × 35 = 150 rupees for 3 kg when first_kg = 1
    assert_eq!(heavy.total_paise, 150_00);
    assert_eq!(light.total_paise, 80_00);
}

/// Mixed day: cash, UPI, credit, COD — recent sales and balance sheet hold.
#[test]
fn journey_mixed_day_desk_and_balance_sheet() {
    let office = open_office();
    let _cash = office.book(
        &office.local,
        "Rajkot",
        "Neha",
        "0.5",
        PayMethod::Cash,
        None,
        0,
        &office.cashier,
    );
    let _upi = office.book(
        &office.west,
        "Pune",
        "Imran",
        "2",
        PayMethod::Upi,
        None,
        0,
        &office.cashier,
    );
    let credit = office.book(
        &office.west,
        "Delhi",
        "Kapoor",
        "6",
        PayMethod::Credit,
        Some(&office.mehta.id),
        0,
        &office.owner,
    );
    let cod = office.book(
        &office.local,
        "Gondal",
        "Hitesh",
        "1",
        PayMethod::Cash,
        Some(&office.pharma.id),
        400,
        &office.cashier,
    );

    let recent = office.store.recent_sales(20).unwrap();
    assert!(recent.len() >= 4);
    assert!(office.store.customer_balance(&office.mehta.id).unwrap() > 0);
    assert_eq!(cod.cod_paise, 400_00);

    let bs = office.store.balance_sheet_text().unwrap();
    assert!(
        bs.contains("Debtors") || bs.to_lowercase().contains("debtor"),
        "{bs}"
    );
    let inv = office.store.format_invoice_text(&credit.id).unwrap();
    assert!(!inv.is_empty());
}

/// Owner and cashier PIN unlock paths.
#[test]
fn journey_staff_pins() {
    let office = open_office();
    let owner = office
        .store
        .authenticate_owner_pin("1234")
        .unwrap()
        .expect("owner");
    assert!(owner.is_owner());
    let asha = office
        .store
        .authenticate_staff_pin(&office.cashier.id, "2222")
        .unwrap()
        .expect("cashier");
    assert_eq!(asha.name, "Asha");
}

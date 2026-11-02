//! Seeds a realistic courier office (consignors and carriers with logos,
//! zones and slab rates, a cashier, riders, and bookings in every desk state:
//! booked, on a rider run, delivered with COD, failed delivery, handed to a
//! carrier, on account) into a fresh data folder, so the UI can be looked
//! at, not guessed (docs/DEV.md § Look at the screen). Owner PIN 1234,
//! cashier Asha 2222. Opt-in only:
//! `SEED_DATA_DIR=<empty folder> KUMKAJ_PRINT_SINK=off cargo test -p
//! kumkaj-moklanjo --test visual_seed -- --ignored`, then start the app with
//! `KUMKAJ_DATA_DIR=<same folder>`.
use kumkaj_moklanjo::domain::*;
use kumkaj_moklanjo::local_servers;
use kumkaj_moklanjo::manifest::{HandoverKind, TrackingStatus};
use kumkaj_moklanjo::store::Store;
use std::path::{Path, PathBuf};

/// A small square logo in one colour with a lighter band, so photo thumbs
/// show as photos (not initials) on every list that has them.
fn logo(dir: &Path, name: &str, rgb: [u8; 3]) -> PathBuf {
    let mut img = image::RgbImage::new(256, 256);
    for (_, y, px) in img.enumerate_pixels_mut() {
        let band = (96..160).contains(&y);
        *px = image::Rgb(if band { rgb.map(|c| c.saturating_add(70)) } else { rgb });
    }
    let path = dir.join(format!("{name}.png"));
    img.save(&path).unwrap();
    path
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
) -> Sale {
    // Priced like the booking desk: chargeable kg on the rate's slab.
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
    s.finalize_sale(&draft, pay, party, owner).unwrap()
}

#[test]
#[ignore]
fn seed_visual() {
    let base = PathBuf::from(std::env::var("SEED_DATA_DIR").unwrap());
    std::fs::create_dir_all(&base).unwrap();
    let (_, s) = local_servers::create_local_server(&base, "Swift Courier").unwrap();
    let owner = s.complete_owner_setup("1234").unwrap();
    s.complete_onboarding().unwrap();
    s.set_org_name("Swift Couriers & Cargo").unwrap();
    s.add_staff("Asha", StaffRole::Cashier, "Counter", "2222", &owner).unwrap();

    let art = base.join("seed-art");
    std::fs::create_dir_all(&art).unwrap();

    // Zones and slab rates the desk prices from.
    let mut rates = Vec::new();
    for (zone, name, first, extra, photo) in [
        ("Local", "Local 1 kg", 60, 20, None),
        ("West", "Surface West", 80, 35, Some([20, 70, 150])),
        ("North", "Express North", 120, 60, None),
        ("South", "Surface South", 90, 40, None),
    ] {
        let z = s.add_zone(zone).unwrap();
        let mut r = Product::new_basic(new_id(), name, Money::from_rupees(first));
        r.unit_label = "kg".into();
        r.zone_id = z.id.clone();
        r.first_kg = "1".parse().unwrap();
        r.first_paise = first * 100;
        r.extra_per_kg_paise = extra * 100;
        if let Some(rgb) = photo {
            r.image_key = Some(s.import_item_image(&r.id, &logo(&art, name, rgb)).unwrap());
        }
        s.upsert_product(&r).unwrap();
        rates.push(r);
    }

    // Parties: consignors who book with us, carriers we hand bags to.
    let party = |name: &str, phone: &str, kind: &str, rgb: Option<[u8; 3]>| {
        let p = s.add_customer_kind(name, phone, kind).unwrap();
        if let Some(rgb) = rgb {
            let key = s.import_party_image(&p.id, &logo(&art, name, rgb)).unwrap();
            s.set_customer_image_key(&p.id, Some(&key)).unwrap();
        }
        p
    };
    let mehta = party("Mehta Traders", "98250 11111", "Consignor", Some([150, 60, 20]));
    let _textiles = party("Rajkot Textiles", "98250 22222", "Consignor", None);
    let pharma = party("Sunrise Pharma", "98250 33333", "Consignor", Some([20, 120, 60]));
    let blueline = party("BlueLine Express", "1800 000 111", "Carrier", Some([10, 40, 120]));
    let _northstar = party("Northstar Logistics", "1800 000 222", "Carrier", None);

    let ravi = s.add_rider("Ravi", "99000 11111", "Bike 12").unwrap();
    let kiran = s.add_rider("Kiran", "99000 22222", "Scooter 7").unwrap();

    // Walk-ins paid at the counter, an account booking, COD parcels.
    let [local, west, north, south] = [&rates[0], &rates[1], &rates[2], &rates[3]];
    let walk_in = book(&s, local, "Rajkot", "Neha Shah", "0.5", PayMethod::Cash, None, 0, &owner);
    let upi = book(&s, west, "Pune", "Imran Ali", "2.5", PayMethod::Upi, None, 0, &owner);
    let account = book(&s, north, "Delhi", "Kapoor Stores", "6", PayMethod::Credit, Some(&mehta.id), 0, &owner);
    let cod1 = book(&s, south, "Bengaluru", "Sunita Rao", "1.5", PayMethod::Cash, Some(&pharma.id), 850, &owner);
    let cod2 = book(&s, local, "Gondal", "Hitesh Patel", "1", PayMethod::Cash, None, 400, &owner);
    let ndr = book(&s, local, "Morbi", "Farah Khan", "2", PayMethod::Cash, None, 300, &owner);
    let carrier = book(&s, north, "Jaipur", "Arjun Mehra", "4", PayMethod::Cash, None, 0, &owner);
    let _booked = book(&s, west, "Surat", "Pooja Desai", "3", PayMethod::Cash, None, 0, &owner);

    // Own-fleet runs: Ravi delivers the COD parcels, Kiran's run fails once.
    s.create_manifest(
        HandoverKind::OwnFleet,
        "",
        &ravi.id,
        &[cod1.id.clone(), cod2.id.clone(), walk_in.id.clone()],
        "morning run",
        &owner,
    )
    .unwrap();
    s.advance_tracking(&cod1.id, TrackingStatus::Delivered, "signed by Sunita", "", &owner)
        .unwrap();
    s.advance_tracking(&walk_in.id, TrackingStatus::Delivered, "", "", &owner)
        .unwrap();
    s.create_manifest(HandoverKind::OwnFleet, "", &kiran.id, &[ndr.id.clone()], "", &owner)
        .unwrap();
    s.record_ndr(&ndr.id, "door_locked", "2099-01-02", &owner).unwrap();
    // A bag handed to a named carrier.
    s.create_manifest(
        HandoverKind::Carrier,
        &blueline.id,
        "",
        &[carrier.id.clone(), upi.id.clone()],
        "evening bag",
        &owner,
    )
    .unwrap();

    // Money out: a carrier bill and a running cost.
    s.add_expense(1_250_00, "Carrier bill", "BlueLine weekly", &owner).unwrap();
    s.add_expense(180_00, "Packing", "Tape and boxes", &owner).unwrap();

    for sale in s.recent_sales(20).unwrap() {
        println!("{} {} {} {}", sale.awb_no, sale.destination, sale.tracking_status, sale.total_paise);
    }
    let _ = account;
}

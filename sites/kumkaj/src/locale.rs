//! kumkaj.com words. English is the source (`EN`, keyed by a stable id);
//! every other catalogue language is `translations/<code>.tsv` in this crate
//! — one `key<TAB>text` line per `EN` key, in this order, `{placeholders}`
//! kept. The runtime (catalogue, `#l=` bridge, packs, the widget walker) is
//! shared with the other Kumkaj sites: `sites/_shared/i18n/site_i18n.rs`.
//!
//! **Page copy** in `ui.rs` is matched by its English text: a Label /
//! Markdown / Button whose text is an `EN` value shows that key's
//! translation. Text the site sets from code goes through `App::tr` /
//! `App::tr_en`. `tests::every_ui_string_is_keyed_or_skipped` fails when
//! `ui.rs` gains copy that is neither here nor in `UI_SKIP`.
//!
//! **New or changed English:** edit `EN`, run
//! `./scripts/scaffold-site-translations.sh kumkaj` (adds the key to every
//! pack as a marked stand-in), then write the translations for real
//! (docs/DEV.md § Languages) — the tests fail while stand-ins remain.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Every site key, in pack order.
pub const EN: &[(&str, &str)] = &[
    ("the_promise", "The promise"),
    ("kumkaj_business_operating_systems", "KUMKAJ · BUSINESS OPERATING SYSTEMS"),
    ("the_internet_went_down_the", "The internet went down.\nThe counter didn't."),
    ("kumkaj_is_the_brand_for", "Kumkaj is the brand for concrete business operating system apps — complete business management software that runs on your own devices. Each product is built for one trade: courier, restaurant, grocery, pharmacy."),
    ("choose_your_trade", "Choose your trade  ↓"),
    ("watch_it_keep_working", "Watch it keep working"),
    ("this_counter_right_now", "THIS COUNTER, RIGHT NOW"),
    ("internet", "Internet"),
    ("down", "Down"),
    ("taking_orders", "Taking orders"),
    ("yes", "Yes"),
    ("bills_and_dockets", "Bills and dockets"),
    ("saving", "Saving"),
    ("day_book", "Day book"),
    ("adding_up", "Adding up"),
    ("nothing_on_this_counter_waits", "Nothing on this counter waits for the cloud."),
    ("scroll_or_use_the_dots", "Scroll or use the dots to jump between sections."),
    ("n_02_built_for_your_trade", "02 · BUILT FOR YOUR TRADE"),
    ("a_desk_for_each_trade", "A desk for each trade.\nNot one app with a switch."),
    ("not_one_app_with_a", "Not one app with a trade switch. Each product is designed around one kind of counter, in that trade's own words."),
    ("courier_offices_franchise_counters_own", "COURIER OFFICES · FRANCHISE COUNTERS · OWN RIDERS"),
    ("book_label_hand_over_track", "Book → label → hand over → track → settle."),
    ("next_booking_in_one_tap", "Next booking in one tap"),
    ("actual_or_volumetric_weight_rounded", "Actual or volumetric weight, rounded your way"),
    ("rider_cash_on_delivery_expected", "Rider cash on delivery: expected against handed in"),
    ("visit_moklanjo_com", "Visit moklanjo.com  →"),
    ("restaurants_caf_s_quick_service", "RESTAURANTS · CAFÉS · QUICK SERVICE · BAKERIES"),
    ("table_kitchen_bill_day_close", "Table → kitchen → bill → day close."),
    ("every_table_s_minutes_and", "Every table's minutes and bill at a glance"),
    ("one_kot_per_kitchen_station", "One KOT per kitchen station — never prices"),
    ("cash_upi_part_payment_due", "Cash, UPI, part payment, Due and tip"),
    ("visit_khaynjo_com", "Visit khaynjo.com  →"),
    ("kirana_grocery_neighbourhood_shops", "KIRANA · GROCERY · NEIGHBOURHOOD SHOPS"),
    ("sell_khata_stock_day_book", "Sell → khata → stock → day book."),
    ("fast_counter_sell_with_short", "Fast counter sell with short codes"),
    ("customer_credit_and_supplier_dues", "Customer credit and supplier dues"),
    ("buy_into_stock_offline_forever", "Buy into stock — offline forever"),
    ("visit_kirano_shop", "Visit kirano.shop  →"),
    ("chemists_pharmacy_counters", "CHEMISTS · PHARMACY COUNTERS"),
    ("medicine_batch_bill_day_close", "Medicine → batch → bill → day close."),
    ("barcode_and_salt_composition_search", "Barcode and salt / composition search"),
    ("batch_and_expiry_alerts_on", "Batch and expiry alerts on purchases"),
    ("owner_schedule_h_rx_notes", "Owner Schedule H / Rx notes — not fake compliance"),
    ("visit_tabiyat_shop", "Visit tabiyat.shop  →"),
    ("n_03_your_books_stay_with", "03 · YOUR BOOKS STAY WITH YOU"),
    ("your_books_live_in_your", "Your books live in your shop."),
    ("sales_parties_and_the_day", "Sales, parties and the day book are written on the device at your counter — not on a server you rent."),
    ("on_your_device", "On your device"),
    ("every_sale_payment_and_docket", "Every sale, payment and docket is saved on the computer or tablet you own."),
    ("no_account_with_us", "No account with us"),
    ("install_it_and_start_no", "Install it and start. No sign-up, no card on file."),
    ("copies_you_keep", "Copies you keep"),
    ("save_a_backup_copy_whenever", "Save a backup copy whenever you like. It is a file, and it is yours."),
    ("extra_counters_and_tablets_join", "Extra counters and tablets join over your own Wi-Fi — with no internet in between."),
    ("n_04_try_it", "04 · TRY IT"),
    ("pull_the_plug_on_the", "Pull the plug on the internet."),
    ("kumkaj_apps_don_t_phone", "Kumkaj apps don't phone home to work. Cut the internet and watch what changes at the counter."),
    ("cut_the_internet", "Cut the internet"),
    ("internet_on", "Internet: on"),
    ("book_a_parcel_or_take", "Book a parcel or take an order"),
    ("save_the_bill_or_docket", "Save the bill or docket"),
    ("day_book_totals", "Day book totals"),
    ("find_an_old_sale", "Find an old sale"),
    ("the_internet_is_on_and", "The internet is on — and the counter isn't using it."),
    ("things_that_leave_the_shop", "Things that leave the shop, like a WhatsApp receipt, still need the internet to arrive."),
    ("n_05_price", "05 · PRICE"),
    ("free_forever_on_your_devices", "Free forever on your devices."),
    ("no_charge_per_seat_user", "No charge per seat, user, table, rider, parcel or local device. Not a trial."),
    ("on_your_own_devices", "On your own devices"),
    ("n_0_forever", "₹0, forever"),
    ("pay_only_for_what_you", "Pay only for what you use — the same rates for everyone."),
    ("planned", "Planned"),
    ("cloud_is_planned_and_optional", "Cloud is planned and optional. The counter works without it."),
    ("n_06_languages", "06 · LANGUAGES"),
    ("speaks_your_staff_s_language", "Speaks your staff's language."),
    ("the_same_115_language_catalogue", "The same 115-language catalogue as every Kumkaj app — Arabic and Urdu right to left. Native names below; “Food ready” samples for Hindi, Gujarati, Tamil, Bengali, Urdu, and English:"),
    ("same_shaping_engine_as_the", "Same shaping engine as the apps, so joined letters and right-to-left lines come out right."),
    ("n_07_who_makes_it", "07 · WHO MAKES IT"),
    ("made_under_gotigin", "Made under Gotigin."),
    ("trademark", "TRADEMARK"),
    ("memon_for_find_it_pronounced", "Memon for “find it” · pronounced go-tee-geen"),
    ("company", "COMPANY"),
    ("brand", "BRAND"),
    ("concrete_business_operating_system_apps", "Concrete business operating system apps"),
    ("products", "PRODUCTS"),
    ("courier_restaurant_grocery_pharmacy", "Courier · restaurant · grocery · pharmacy"),
    ("courier_desk_book_label_hand", "Courier desk — book, label, hand over, settle."),
    ("restaurant_system_table_kitchen_bill", "Restaurant system — table, kitchen, bill, day close."),
    ("grocery_counter_sell_khata_stock", "Grocery counter — sell, khata, stock, day book."),
    ("pharmacy_counter_batch_expiry_bill", "Pharmacy counter — batch, expiry, bill, day close."),
    ("privacy", "Privacy"),
    ("terms", "Terms"),
    ("cancellation", "Cancellation"),
    ("cloud_shop", "Cloud shop"),
    ("account", "Account"),
    ("chapter_built_for_your_trade", "Built for your trade"),
    ("chapter_your_books", "Your books"),
    ("chapter_try_it_offline", "Try it offline"),
    ("chapter_price", "Price"),
    ("chapter_languages", "Languages"),
    ("chapter_who_makes_it", "Who makes it"),
    ("toy_bring_back", "Bring the internet back"),
    ("toy_internet_off", "Internet: off"),
    ("toy_verdict_off", "Internet off — and nothing on the counter changed."),
    ("acct_page_title", "Account — Kumkaj"),
    ("acct_lede", "Each Kumkaj product has its own account (and its own cloud). Choose the trade you run — sign-in and checkout stay on that product site."),
    ("acct_product_accounts", "Product accounts"),
    ("acct_moklanjo_blurb", "Courier OS account"),
    ("acct_khaynjo_blurb", "Restaurant OS account"),
    ("acct_kirano_blurb", "Kirana counter account"),
    ("acct_tabiyat_blurb", "Pharmacy counter account"),
    ("acct_open_account", "Open account"),
    ("shop_page_title", "Shop — Kumkaj"),
    ("shop_lede", "Yearly cloud plans — same five shapes for every trade (Free forever, Steady ₹6,000, Swift ₹12,000, Priority ₹24,000 / year ex-tax, or Custom quote). Tax is added at checkout. Each product keeps its own shop and account."),
    ("shop_product_shops", "Product cloud shops"),
    ("shop_moklanjo_blurb", "Courier desk cloud for moklanjo.com"),
    ("shop_khaynjo_blurb", "Restaurant cloud for khaynjo.com"),
    ("shop_kirano_blurb", "Kirana counter cloud for kirano.shop"),
    ("shop_tabiyat_blurb", "Pharmacy counter cloud for tabiyat.shop"),
    ("shop_open_shop", "Open shop"),
];

/// `ui.rs` text that is not site copy: template defaults the page replaces,
/// names, numbers, mock-ups and text the code overwrites at run time.
#[cfg_attr(not(test), allow(dead_code))]
pub const UI_SKIP: &[&str] = &[
    "Row",
    "Working",
    "—",
    "Bullet",
    "Language",
    "Word",
    "Kumkaj",
    "1 / 7",
    "Moklanjo",
    "Khaynjo",
    "Kirano",
    "Tabiyat",
    "Kumkaj Cloud",
    "Gotigin",
    "Where Technology Meets Discovery",
    "GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED",
    "support@gotigin.com",
    "Director · mearaj@gmail.com",
    "Moklanjo · Khaynjo · Kirano · Tabiyat",
    "© 2026 GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED",
    "EN",
    "English",
    "Food ready",
    "हिन्दी",
    "खाना तैयार",
    "বাংলা",
    "খাবার তৈরি",
    "తెలుగు",
    "मराठी",
    "தமிழ்",
    "உணவு தயார்",
    "اردو",
    "کھانا تیار",
    "ગુજરાતી",
    "જમવાનું તૈયાર",
    "ಕನ್ನಡ",
    "മലയാളം",
    "ଓଡ଼ିଆ",
    "ਪੰਜਾਬੀ",
    "অসমীয়া",
    "मैथिली",
    "संस्कृतम्",
    "كٲشُر",
    "नेपाली",
    "سنڌي",
    "कोंकणी",
    "डोगरी",
    "ꯃꯤꯇꯩꯂꯣꯟ",
    "ᱥᱟᱱᱛᱟᱲᱤ",
    "बर'",
    "भोजपुरी",
    "अवधी",
    "मगही",
    "छत्तीसगढ़ी",
    "राजस्थानी",
    "ತುಳು",
    "ককবরক",
    "Mizo ṭawng",
    "Khasi",
    "गोंयची कोंकणी",
    "བོད་སྐད་",
    "සිංහල",
    "ދިވެހި",
    "پښتو",
    "فارسی",
    "中文",
    "繁體中文",
    "粵語",
    "日本語",
    "한국어",
    "Tiếng Việt",
    "ไทย",
    "Bahasa Indonesia",
    "Bahasa Melayu",
    "Filipino",
    "မြန်မာ",
    "ខ្មែរ",
    "ລາວ",
    "العربية",
    "עברית",
    "Türkçe",
    "Azərbaycan",
    "Қазақша",
    "Oʻzbekcha",
    "Кыргызча",
    "Тоҷикӣ",
    "Türkmençe",
    "Монгол",
    "Español",
    "Français",
    "Português",
    "Deutsch",
    "Italiano",
    "Русский",
    "Українська",
    "Polski",
    "Nederlands",
    "Română",
    "Magyar",
    "Čeština",
    "Slovenčina",
    "Български",
    "Српски",
    "Hrvatski",
    "Bosanski",
    "Slovenščina",
    "Ελληνικά",
    "Svenska",
    "Norsk",
    "Dansk",
    "Suomi",
    "Íslenska",
    "Gaeilge",
    "Cymraeg",
    "Català",
    "Euskara",
    "Galego",
    "Lietuvių",
    "Latviešu",
    "Eesti",
    "Malti",
    "Shqip",
    "Македонски",
    "Беларуская",
    "ქართული",
    "Հայերեն",
    "Kiswahili",
    "አማርኛ",
    "Hausa",
    "Yorùbá",
    "Igbo",
    "isiZulu",
    "isiXhosa",
    "Afrikaans",
    "Soomaali",
    "Ikinyarwanda",
    "Malagasy",
    "Português (Brasil)",
    "Español (México)",
    "Kreyòl ayisyen",
    "Runa Simi",
    "Avañe'ẽ",
];

/// English for `key` ("" for an unknown key — tests forbid that in code).
pub fn en(key: &str) -> &'static str {
    EN.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap_or("")
}

/// The key whose English is exactly `english` (page copy lookup).
pub fn key_for(english: &str) -> Option<&'static str> {
    static REVERSE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    REVERSE
        .get_or_init(|| EN.iter().map(|(k, v)| (*v, *k)).collect())
        .get(english)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site_i18n::checks;

    const DIR: &str = env!("CARGO_MANIFEST_DIR");

    fn keys() -> Vec<&'static str> {
        EN.iter().map(|(k, _)| *k).collect()
    }

    #[test]
    fn keys_are_unique_and_english_is_unique() {
        let mut seen_k = std::collections::HashSet::new();
        let mut seen_v = std::collections::HashSet::new();
        for (k, v) in EN {
            assert!(seen_k.insert(*k), "duplicate key {k}");
            assert!(seen_v.insert(*v), "duplicate English {v:?} (two keys would fight over one widget)");
            assert!(!v.trim().is_empty(), "{k}: empty English");
        }
    }

    #[test]
    fn every_catalogue_language_translates_every_key() {
        checks::every_language_translates_every_key(DIR, &keys(), &en);
    }

    #[test]
    fn no_text_is_left_in_english_in_most_packs() {
        checks::no_text_is_left_in_english(DIR, &keys(), &en, &[]);
    }

    #[test]
    fn every_translation_draws_with_the_site_fonts() {
        checks::every_pack_draws_with_its_fonts(DIR, &crate::script_font_files);
    }

    /// Every quoted `body:` / `text:` / `empty_text:` string in `ui.rs` is an
    /// `EN` value or listed in `UI_SKIP` (or is blank / numbers / an icon).
    #[test]
    fn every_ui_string_is_keyed_or_skipped() {
        checks::every_ui_string_is_keyed_or_skipped(include_str!("ui.rs"), &key_for, UI_SKIP);
    }

    /// HTML pages (`data-i18n`) and scripts (`T("key", "English")`).
    #[test]
    fn html_and_script_copy_match_en() {
        checks::html_and_script_copy_match_en(DIR, &["static/account/index.html", "static/shop/index.html"], &[], &en);
    }
}

//! kirano.shop words. English is the source (`EN`, keyed by a stable id);
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
//! `./scripts/scaffold-site-translations.sh kirano` (adds the key to every
//! pack as a marked stand-in), then write the translations for real
//! (docs/DEV.md § Languages) — the tests fail while stand-ins remain.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Every site key, in pack order.
pub const EN: &[(&str, &str)] = &[
    ("example", "Example"),
    ("download_free", "Download free"),
    ("whatsapp_for_a_demo", "WhatsApp for a demo"),
    ("free_on_android_windows_and", "Free on Android, Windows and Linux."),
    ("the_grocery_counter", "the grocery counter"),
    ("busy_counter", "Busy counter"),
    ("kirano_the_grocery_counter", "KIRANO · THE GROCERY COUNTER"),
    ("busy_counter_no_internet_still", "Busy counter. No internet. Still billing."),
    ("sell_khata_stock_and_the", "Sell, khata, stock and the day book — on your own devices. Books · Sell · Buy · Items. No floor, no kitchen, no tables."),
    ("tap_the_bill_to_walk", "Tap the bill to walk a counter sale"),
    ("cash_memo", "Cash memo"),
    ("ready", "Ready"),
    ("total", "Total"),
    ("tap_to_add_the_first", "Tap to add the first item"),
    ("example_grocery_lines_not_a", "Example grocery lines, not a live till. Kirano follows the same path: sell → settle → day book."),
    ("n_2_counter_sell", "2 · COUNTER SELL"),
    ("item_quantity_pay_three_taps", "Item, quantity, pay — three taps."),
    ("upi_qr_ready", "UPI QR ready"),
    ("example_bill", "Example bill"),
    ("search_by_name_or_short", "Search by name or short code"),
    ("type_atta_at5_or_an", "Type atta, AT5 or an alias — the line lands on the bill."),
    ("kg_g_pc_l_ml", "kg, g, pc, L, ml"),
    ("quantity_in_the_unit_the", "Quantity in the unit the packet uses; price follows."),
    ("cash_upi_or_split", "Cash, UPI or split"),
    ("settle_on_the_counter_the", "Settle on the counter; the day book records the tender."),
    ("n_3_khata", "3 · KHATA"),
    ("credit_without_a_crm_put", "Credit without a CRM — put it on their khata."),
    ("ramesh_customer", "Ramesh · Customer"),
    ("due_from_earlier", "Due from earlier"),
    ("today_s_sale_due", "+ today's sale (Due)"),
    ("balance", "Balance"),
    ("settle_later_in_parties_part", "Settle later in Parties — part cash, part UPI, clear the line."),
    ("due_on_the_bill", "Due on the bill"),
    ("what_s_left_goes_on", "What's left goes on the customer's khata — no separate ledger app."),
    ("one_party_two_roles", "One party, two roles"),
    ("customer_ar_and_supplier_ap", "Customer AR and supplier AP share the same party book."),
    ("who_when_how_much", "Who, when, how much"),
    ("every_credit_and_settlement_stays", "Every credit and settlement stays in the day book."),
    ("n_4_buy_stock", "4 · BUY & STOCK"),
    ("buy_from_the_wholesaler_stock", "Buy from the wholesaler — stock and who you owe update together."),
    ("purchase_sharma_wholesale", "Purchase · Sharma Wholesale"),
    ("stock_50_kg_ap_7", "Stock +50 kg · AP ₹ 7,200"),
    ("stock_12_l_ap_1", "Stock +12 L · AP ₹ 1,560"),
    ("resale_stock_not_kitchen_raw", "Resale stock — not kitchen raw. Optional batch and expiry on the buy."),
    ("buy_creates_stock", "Buy creates stock"),
    ("receive_the_goods_the_catalogue", "Receive the goods; the catalogue quantity rises."),
    ("who_we_owe", "Who we owe"),
    ("supplier_balance_sits_on_the", "Supplier balance sits on the same party as customer khata."),
    ("low_stock_at_a_glance", "Low stock at a glance"),
    ("inventory_shows_what_is_running", "Inventory shows what is running out before the next rush."),
    ("n_5_day_book", "5 · DAY BOOK"),
    ("cash_vs_upi_counted_and", "Cash vs UPI, counted — and the difference named."),
    ("today_s_drawer", "Today's drawer"),
    ("opening_float", "Opening float"),
    ("cash_sales", "+ cash sales"),
    ("upi_sales", "+ UPI sales"),
    ("cash_paid_out", "− cash paid out"),
    ("expected_cash", "Expected cash"),
    ("counted", "Counted"),
    ("short", "Short"),
    ("tenders_stay_separate", "Tenders stay separate"),
    ("cash_and_upi_never_mix", "Cash and UPI never mix into one mystery total."),
    ("shift_close", "Shift close"),
    ("close_the_shift_print_the", "Close the shift, print the slip, start the next one clean."),
    ("gst_facing_bills", "GST-facing bills"),
    ("tax_invoice_or_cash_memo", "Tax invoice or cash memo — the day report carries both."),
    ("books_sell_buy_items_four", "Books · Sell · Buy · Items — four screens, and no restaurant floor."),
    ("n_6_internet_down", "6 · INTERNET DOWN?"),
    ("the_internet_drops_the_counter", "The internet drops. The counter keeps selling."),
    ("router_no_internet", "Router: no internet"),
    ("main_counter_selling_and_settling", "Main counter: selling and settling"),
    ("second_till_joined_on_your", "Second till: joined on your own Wi-Fi"),
    ("books_live_on_the_shop", "Books live on the shop devices"),
    ("when_the_internet_drops_mid", "When the internet drops mid-rush, nothing at the counter waits for it."),
    ("one_server_lan_clients", "One Server, LAN Clients"),
    ("extra_tills_share_the_same", "Extra tills share the same books over the shop network."),
    ("n_7_free_forever_locally", "7 · FREE FOREVER LOCALLY"),
    ("free_forever_on_your_devices", "Free forever on your devices."),
    ("no_per_bill_charge", "No per-bill charge"),
    ("no_fee_per_sale_item", "No fee per sale, item, till or staff PIN on the local install."),
    ("android_windows_linux", "Android, Windows, Linux"),
    ("the_same_kirano_on_the", "The same Kirano on the phone behind the counter and the desktop till."),
    ("optional_kumkaj_cloud", "Optional Kumkaj Cloud"),
    ("yearly_steady_6k_swift_12k", "Yearly Steady ₹6k · Swift ₹12k · Priority ₹24k ex-tax (or Custom quote). Shop at /shop."),
    ("n_8_start", "8 · START"),
    ("put_kirano_on_the_counter", "Put Kirano on the counter."),
    ("download_free_or_whatsapp_for", "Download free, or WhatsApp for a walk-through with your shop type and city."),
    ("privacy", "Privacy"),
    ("terms", "Terms"),
    ("cancellation", "Cancellation"),
    ("cloud_shop", "Cloud shop"),
    ("account", "Account"),
    ("chapter_counter_sell", "Counter sell"),
    ("chapter_khata", "Khata"),
    ("chapter_buy_stock", "Buy & stock"),
    ("chapter_day_book", "Day book"),
    ("chapter_extra_till", "Extra till"),
    ("chapter_offline", "Offline"),
    ("chapter_start", "Start"),
    ("bill_selling", "Selling"),
    ("bill_settle", "Settle"),
    ("bill_paid", "Paid"),
    ("bill_next_item", "Tap to add the next item"),
    ("bill_settle_hint", "Tap to settle — cash or UPI"),
    ("bill_cash_return", "Cash ₹ 600 · return ₹ 15"),
    ("bill_paid_line", "Paid · day book updated"),
    ("lan_eyebrow", "6 · EXTRA TILL"),
    ("lan_title", "Another till on the shop Wi-Fi."),
    ("cta_installers_note", "Installers go up here once each one passes a clean-machine install. Until then, ask for a demo on WhatsApp."),
    ("cta_whatsapp_soon", "The WhatsApp demo line opens soon. Your message will read: “{message}”."),
    ("acct_page_title", "Account — Kirano"),
    ("acct_lede", "One account for the site and apps — same form whether you’re new or returning."),
    ("acct_email", "Email"),
    ("acct_name_optional", "Name (optional)"),
    ("acct_password", "Password"),
    ("acct_continue", "Continue"),
    ("acct_forgot", "Forgot password?"),
    ("acct_reset_hint", "We’ll email a reset link. Use the same address you signed up with."),
    ("acct_send_reset", "Send reset link"),
    ("acct_cancel", "Cancel"),
    ("acct_or_continue", "or continue with"),
    ("acct_phone", "Phone"),
    ("acct_mobile", "Mobile number"),
    ("acct_send_sms", "Send SMS code"),
    ("acct_sms_code", "SMS code"),
    ("acct_confirm_code", "Confirm code"),
    ("acct_signed_in", "Signed in"),
    ("acct_signed_in_as", "Signed in as"),
    ("acct_status", "Status"),
    ("acct_sign_in_methods", "Sign-in methods"),
    ("acct_cloud_username", "Cloud username"),
    ("acct_plan", "Plan"),
    ("acct_resend_verify", "Resend verification email"),
    ("acct_sign_out", "Sign out"),
    ("acct_cloud_sandbox", "Kumkaj Cloud (sandbox)"),
    ("acct_current_password", "Current password"),
    ("acct_email_for_password", "Email for password sign-in"),
    ("acct_new_password", "New password"),
    ("acct_confirm_new_password", "Confirm new password"),
    ("acct_add_way", "Add another way to sign in"),
    ("acct_link_hint", "Link methods to the same account so you can recover access if one fails."),
    ("acct_link_google", "Link Google"),
    ("acct_link_microsoft", "Link Microsoft"),
    ("acct_link_github", "Link GitHub"),
    ("acct_link_phone", "Link phone"),
    ("acct_send_sms_link", "Send SMS to link"),
    ("acct_confirm_link_phone", "Confirm & link phone"),
    ("shop_page_title", "Shop — Kirano"),
    ("shop_lede", "Yearly cloud plans — tax added at checkout. The full Kirano app on your own devices stays free forever, with no ads."),
    ("shop_cart", "Cart"),
    ("shop_cart_empty_hint", "Your cart is empty. Add Steady, Swift, or Priority."),
    ("shop_checkout", "Checkout"),
    ("shop_clear", "Clear"),
    ("shop_checkout_hint", "Checkout signs you in on Account, then starts payment (sandbox mock or Paddle when keys are set)."),
    ("acct_billing_hint", "Yearly plans — tax added at checkout by the gateway. Local use stays free forever. Custom is a quote (may be below Steady or above Priority)."),
    ("shop_cloud_plans", "Cloud plans"),
    ("acct_federated", "Federated sign-in"),
    ("shop_pricing_soon", "Pricing coming soon"),
    ("shop_request_quote", "Request quote"),
    ("shop_add_to_cart", "Add to cart"),
    ("shop_notify_later", "Notify me later"),
    ("shop_included", "Included"),
    ("shop_in_cart", "{plan} is in your cart."),
    ("shop_remove", "Remove"),
    ("shop_cart_is_empty", "Cart is empty."),
    ("shop_prices_unpublished", "Self-serve prices are not published for this product yet. Request a Custom quote or check back later."),
    ("shop_cart_cleared", "Cart cleared."),
    ("plan_free_title", "Free forever"),
    ("plan_free_blurb", "Full local product on your devices. No cloud fee."),
    ("plan_steady_blurb", "Live cloud · 10 GB · lower bandwidth"),
    ("plan_swift_blurb", "Live cloud · 25 GB · mid bandwidth"),
    ("plan_priority_blurb", "Live cloud · 50 GB · highest standard bandwidth"),
    ("plan_custom_title", "Custom"),
    ("plan_custom_blurb", "Agreed quota & bandwidth. Quote may be below Steady or above Priority."),
];

/// `ui.rs` text that is not site copy: template defaults the page replaces,
/// names, numbers, mock-ups and text the code overwrites at run time.
#[cfg_attr(not(test), allow(dead_code))]
pub const UI_SKIP: &[&str] = &[
    "SELL · COUNTER",
    "Point",
    "Detail",
    "Item",
    "Kirano",
    "1 / 8",
    "Cash memo · 12:42",
    "1  Atta 5 kg          280",
    "1  Sunflower oil 1 L  145",
    "1  Toor dal 1 kg      160",
    "Total                585",
    "Toor dal 50 kg",
    "Sunflower oil 12 × 1 L",
    "kumkaj.com  →",
    "© 2026 GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED",
    "support@gotigin.com",
    "Gotigin",
    "Kumkaj",
    "EN",
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
        checks::html_and_script_copy_match_en(DIR, &["static/account/index.html", "static/shop/index.html"], &["client/shop-app.js"], &en);
        // shop-app.js builds these keys from the catalogue ids.
        for id in ["free", "steady", "swift", "priority", "custom"] {
            assert!(!en(&format!("plan_{id}_blurb")).is_empty(), "plan_{id}_blurb missing");
        }
        assert!(!en("plan_free_title").is_empty() && !en("plan_custom_title").is_empty());
    }
}

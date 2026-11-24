//! tabiyat.shop words. English is the source (`EN`, keyed by a stable id);
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
//! `./scripts/scaffold-site-translations.sh tabiyat` (adds the key to every
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
    ("the_pharmacy_counter", "the pharmacy counter"),
    ("pharmacy_counter", "Pharmacy counter"),
    ("tabiyat_the_pharmacy_counter", "TABIYAT · THE PHARMACY COUNTER"),
    ("pharmacy_counter_offline_your_books", "Pharmacy counter. Offline. Your books."),
    ("medicines_barcode_and_salt_search", "Medicines, barcode and salt search, batch and expiry for your own books — on your devices. Owner flags only; never fake Drugs Act compliance."),
    ("tap_the_bill_to_walk", "Tap the bill to walk a counter sale"),
    ("cash_memo", "Cash memo"),
    ("ready", "Ready"),
    ("total", "Total"),
    ("tap_to_add_the_first", "Tap to add the first medicine"),
    ("example_pharmacy_lines_not_a", "Example pharmacy lines, not a live till. Tabiyat follows the same path: sell → settle → day book."),
    ("n_2_counter_sell", "2 · COUNTER SELL"),
    ("medicine_quantity_pay_three_taps", "Medicine, quantity, pay — three taps."),
    ("upi_qr_ready", "UPI QR ready"),
    ("example_bill", "Example bill"),
    ("fast_counter_billing", "Fast counter billing"),
    ("add_the_line_set_the", "Add the line, set the qty, settle — cash, UPI or split."),
    ("gst_tax_invoice_or_cash", "GST Tax invoice or cash memo"),
    ("the_bill_lane_you_need", "The bill lane you need for the sale."),
    ("day_book_follows", "Day book follows"),
    ("every_tender_lands_in_books", "Every tender lands in Books when you close the sale."),
    ("n_3_barcode_salt", "3 · BARCODE & SALT"),
    ("scan_the_barcode_or_find", "Scan the barcode — or find it by salt."),
    ("search", "Search"),
    ("composition_paracetamol_strip_of_10", "Composition: Paracetamol · strip of 10"),
    ("salt_amoxicillin", "Salt: amoxicillin"),
    ("ranks_barcode_name_and_composition", "Ranks barcode, name and composition together."),
    ("wedge_scanner_or_type_enter", "Wedge scanner or type Enter — the line lands on the bill."),
    ("barcode_first", "Barcode first"),
    ("scan_or_paste_the_code", "Scan or paste the code; the catalogue match comes up."),
    ("salt_composition", "Salt / composition"),
    ("type_the_salt_when_the", "Type the salt when the brand name is not what the customer said."),
    ("owner_rx_schedule_h_flags", "Owner Rx / Schedule H flags"),
    ("reminders_you_set_not_a", "Reminders you set — not a statutory register, not medical advice."),
    ("n_4_batch_expiry", "4 · BATCH & EXPIRY"),
    ("batch_and_expiry_on_your", "Batch and expiry on your books — alerts for the owner."),
    ("buy_batch_a12", "Buy · batch A12"),
    ("expires_2027_03_alert_in", "Expires 2027-03 · alert in 7 days"),
    ("ors_sachets_batch_b04", "ORS sachets · batch B04"),
    ("expires_2026_11_near_expiry", "Expires 2026-11 · near-expiry strip on Books"),
    ("shop_bookkeeping_only_not_drugs", "Shop bookkeeping only — not Drugs Act compliance."),
    ("batch_on_buy", "Batch on buy"),
    ("record_batch_and_expiry_when", "Record batch and expiry when stock comes in."),
    ("near_expiry_alerts", "Near-expiry alerts"),
    ("books_and_sell_can_hint", "Books and Sell can hint before the date passes."),
    ("owner_practice", "Owner practice"),
    ("fefo_hints_and_rx_notes", "FEFO hints and Rx notes are yours to follow — Tabiyat records what you enter."),
    ("n_5_khata", "5 · KHATA"),
    ("customer_credit_and_supplier_dues", "Customer credit and supplier dues in one party book."),
    ("mehta_clinic_customer", "Mehta Clinic · Customer"),
    ("due_from_earlier", "Due from earlier"),
    ("today_s_sale_due", "+ today's sale (Due)"),
    ("balance", "Balance"),
    ("settle_later_part_cash_part", "Settle later — part cash, part UPI. Supplier AP shares the same Parties screen."),
    ("due_on_the_bill", "Due on the bill"),
    ("leave_a_balance_on_khata", "Leave a balance on khata without a separate ledger app."),
    ("supplier_buys_too", "Supplier buys too"),
    ("who_you_owe_updates_when", "Who you owe updates when stock arrives."),
    ("auditable", "Auditable"),
    ("who_when_and_how_much", "Who, when and how much stays in the day book."),
    ("n_6_internet_down", "6 · INTERNET DOWN?"),
    ("the_internet_drops_the_counter", "The internet drops. The counter keeps selling."),
    ("router_no_internet", "Router: no internet"),
    ("main_counter_selling_and_settling", "Main counter: selling and settling"),
    ("second_till_joined_on_your", "Second till: joined on your own Wi-Fi"),
    ("books_live_on_the_shop", "Books live on the shop devices"),
    ("when_the_internet_drops_nothing", "When the internet drops, nothing at the pharmacy counter waits for it."),
    ("one_server_lan_clients", "One Server, LAN Clients"),
    ("extra_tills_share_the_same", "Extra tills share the same books over the shop network."),
    ("n_7_free_forever_locally", "7 · FREE FOREVER LOCALLY"),
    ("free_forever_on_your_devices", "Free forever on your devices."),
    ("no_per_bill_charge", "No per-bill charge"),
    ("no_fee_per_sale_medicine", "No fee per sale, medicine, till or staff PIN on the local install."),
    ("android_windows_linux", "Android, Windows, Linux"),
    ("the_same_tabiyat_on_the", "The same Tabiyat on the phone and the desktop till."),
    ("optional_kumkaj_cloud", "Optional Kumkaj Cloud"),
    ("yearly_steady_6k_swift_12k", "Yearly Steady ₹6k · Swift ₹12k · Priority ₹24k ex-tax (or Custom quote). Shop at /shop."),
    ("n_8_start", "8 · START"),
    ("put_tabiyat_on_the_counter", "Put Tabiyat on the counter."),
    ("download_free_or_whatsapp_for", "Download free, or WhatsApp for a walk-through with your pharmacy and city."),
    ("privacy", "Privacy"),
    ("terms", "Terms"),
    ("cancellation", "Cancellation"),
    ("cloud_shop", "Cloud shop"),
    ("account", "Account"),
    ("chapter_counter_sell", "Counter sell"),
    ("chapter_barcode_salt", "Barcode & salt"),
    ("chapter_batch_expiry", "Batch & expiry"),
    ("chapter_khata", "Khata"),
    ("chapter_extra_till", "Extra till"),
    ("chapter_offline", "Offline"),
    ("chapter_start", "Start"),
    ("bill_selling", "Selling"),
    ("bill_settle", "Settle"),
    ("bill_paid", "Paid"),
    ("bill_next_line", "Tap to add the next line"),
    ("bill_settle_hint", "Tap to settle — cash or UPI"),
    ("bill_upi_paid", "UPI ₹ 162 · paid"),
    ("bill_paid_line", "Paid · day book updated"),
    ("lan_eyebrow", "6 · EXTRA TILL"),
    ("lan_title", "Another till on the shop Wi-Fi."),
    ("cta_installers_note", "Installers go up here once each one passes a clean-machine install. Until then, ask for a demo on WhatsApp."),
    ("cta_whatsapp_soon", "The WhatsApp demo line opens soon. Your message will read: “{message}”."),
    ("acct_page_title", "Account — Tabiyat"),
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
    ("shop_page_title", "Shop — Tabiyat"),
    ("shop_lede", "Yearly cloud plans — tax added at checkout. The full Tabiyat app on your own devices stays free forever, with no ads."),
    ("shop_cart", "Cart"),
    ("shop_cart_empty_hint", "Your cart is empty. Add Steady, Swift, or Priority."),
    ("shop_checkout", "Checkout"),
    ("shop_clear", "Clear"),
    ("shop_checkout_hint", "Checkout signs you in on Account, then starts payment (sandbox mock or Paddle when keys are set)."),
    ("acct_billing_hint", "Yearly plans — tax added at checkout by the gateway. Local use stays free forever. Custom is a quote (may be below Steady or above Priority)."),
    ("shop_cloud_plans", "Cloud plans"),
    ("acct_federated", "Federated sign-in"),
    ("acct_err_invalid_email", "Enter a valid email address."),
    ("acct_err_weak_password", "Use a password with at least 6 characters."),
    ("acct_err_wrong_password", "Wrong password."),
    ("acct_err_invalid_credential", "Email or password is wrong."),
    ("acct_err_popup_closed", "Sign-in window was closed."),
    ("acct_err_popup_cancelled", "Sign-in was cancelled."),
    ("acct_err_unauthorized_domain", "This domain is not authorized for Firebase Auth."),
    ("acct_err_method_disabled", "That sign-in method is not enabled yet."),
    ("acct_err_too_many", "Too many attempts. Try again later."),
    ("acct_err_phone_format", "Use a full phone number with country code (e.g. +91…)."),
    ("acct_err_sms_code", "That SMS code is wrong or expired."),
    ("acct_err_no_account", "No account with that email. Continue to create one, or use another method."),
    ("acct_err_recent_login", "For security, sign in again, then retry this change."),
    ("acct_err_credential_in_use", "That sign-in method is already used by another account."),
    ("acct_err_email_in_use", "That email is already used by another account."),
    ("acct_err_already_linked", "That method is already linked to this account."),
    ("acct_err_exists_other_method", "An account already exists with this email using a different sign-in method. Sign in with that method first, then link this one from your account page."),
    ("acct_err_generic", "Something went wrong."),
    ("acct_method_password", "Email & password"),
    ("acct_pw_hint_change", "Change your password. You’ll confirm with the current one."),
    ("acct_pw_hint_add", "Add a password so you can also sign in with email (useful if a provider is unavailable)."),
    ("acct_pw_change", "Change password"),
    ("acct_pw_add", "Add password"),
    ("acct_billing_off", "Billing is not enabled on the cloud API yet."),
    ("acct_billing_no_provider", "No real billing provider is enabled on the cloud API (mock-only is local)."),
    ("price_quote", "Quote"),
    ("price_per_year", "₹{amount}/yr ex-tax"),
    ("acct_pay_received", "Payment received — refreshing your plan…"),
    ("acct_pay_declined", "Payment was declined or failed"),
    ("acct_pay_failed_detail", "{detail}. Your plan was not changed."),
    ("acct_checkout_closed", "Checkout closed — no payment completed. Your plan is unchanged."),
    ("acct_checkout_failed", "Checkout failed"),
    ("acct_pay_with", "Pay with {provider}"),
    ("acct_plan_until", "{plan} · until {date}"),
    ("acct_unavailable", "Unavailable"),
    ("acct_continue_hint", "Continue with email, a provider, or phone."),
    ("acct_signed_in_fallback", "(signed in)"),
    ("acct_email_verified", "Email verified"),
    ("acct_email_unverified", "Email not verified yet — check your inbox"),
    ("acct_signed_in_phone", "Signed in with phone"),
    ("acct_none_listed", "None listed yet"),
    ("acct_signed_in_status", "You’re signed in."),
    ("acct_created", "Account created. Check your email to verify when you can."),
    ("acct_reset_sent", "If an account exists for that email, a reset link is on its way. Check your inbox."),
    ("acct_another_method", "another method"),
    ("acct_exists_via", "An account for {email} already exists via {method}. Sign in that way first, then link this provider from your account page."),
    ("acct_enter_current_pw", "Enter your current password."),
    ("acct_relogin_retry", "Sign out and sign in again, then retry."),
    ("acct_pw_mismatch", "New password and confirmation do not match."),
    ("acct_pw_updated", "Password updated."),
    ("acct_enter_email_pw", "Enter an email to use with password sign-in."),
    ("acct_pw_added", "Password added. You can also sign in with email now."),
    ("acct_method_linked", "Sign-in method linked."),
    ("acct_phone_linked", "Phone linked to this account."),
    ("acct_verify_sent", "Verification email sent — check your inbox."),
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
    "Tabiyat",
    "1 / 8",
    "Cash memo · 12:42",
    "1  Paracetamol 500     28",
    "1  Amoxicillin 250     86",
    "4  ORS sachets         48",
    "Total               162",
    "8901030… · Paracetamol 500 mg",
    "Amoxicillin 250 mg · 10 × 10",
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
        // Three product names: they stay as written in every script; only
        // the list punctuation changes (、 in zh/ja, و in ar/ur).
        checks::no_text_is_left_in_english(DIR, &keys(), &en, &["android_windows_linux"]);
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
        checks::html_and_script_copy_match_en(DIR, &["static/account/index.html", "static/shop/index.html"], &["client/shop-app.js", "client/account-app.js"], &en);
        // shop-app.js builds these keys from the catalogue ids.
        for id in ["free", "steady", "swift", "priority", "custom"] {
            assert!(!en(&format!("plan_{id}_blurb")).is_empty(), "plan_{id}_blurb missing");
        }
        assert!(!en("plan_free_title").is_empty() && !en("plan_custom_title").is_empty());
    }
}

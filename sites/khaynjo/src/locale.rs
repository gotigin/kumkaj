//! khaynjo.com words. English is the source (`EN`, keyed by a stable id);
//! every other catalogue language is `translations/<code>.tsv` in this crate
//! — one `key<TAB>text` line per `EN` key, in this order, `{placeholders}`
//! kept. The runtime (catalogue, `#l=` bridge, packs, the widget walker) is
//! shared with the other Kumkaj sites: `sites/_shared/i18n/site_i18n.rs`.
//!
//! **Page copy** in `ui.rs` is matched by its English text: a Label /
//! Markdown / Button whose text is an `EN` value shows that key's
//! translation. Text the site sets from code goes through `App::tr`.
//! `tests::every_ui_string_is_keyed_or_skipped` fails when `ui.rs` gains
//! copy that is neither here nor in `UI_SKIP` (names, numbers, mock-ups).
//!
//! **New or changed English:** edit `EN`, run
//! `./scripts/scaffold-ui-translations.sh --site khaynjo` (adds the key to
//! every pack as a marked stand-in), then write the translations for real
//! (docs/DEV.md § Languages) — the tests fail while stand-ins remain.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Every site key, in pack order.
pub const EN: &[(&str, &str)] = &[
    ("example", "Example"),
    ("free", "Free"),
    ("photo_coming", "Photo coming"),
    ("get_it_on_google_play", "Get it on Google Play"),
    ("whatsapp_for_a_demo", "WhatsApp for a demo"),
    ("menus", "Menus"),
    ("how_it_works", "How it works"),
    ("features", "Features"),
    ("offline", "Offline"),
    ("pricing", "Pricing"),
    ("faq", "FAQ"),
    ("login", "Login"),
    ("get_it_free", "Get it free"),
    ("restaurant_pos_free_forever", "Restaurant POS · free forever"),
    ("lunch_rush_no_internet_no", "Lunch rush. No internet. No problem."),
    ("khaynjo_runs_your_floor_kitchen", "Khaynjo runs your floor, kitchen and counter on devices you already own — from table to kitchen to bill, with or without the internet."),
    ("android_on_google_play_today", "Android on Google Play today · Windows and Linux packages coming."),
    ("works_with_no_internet", "Works with no internet"),
    ("your_books_stay_on_your", "Your books stay on your devices"),
    ("n_115_languages_on_screen", "115+ languages on screen"),
    ("your_floor_at_a_glance", "Your floor at a glance"),
    ("tap_a_table_to_walk", "Tap a table to walk it through a visit. Same colours and words as the app — and a table frees only once it is paid."),
    ("every_country", "EVERY COUNTRY"),
    ("every_country_at_your_counter", "Every country at your counter"),
    ("one_famous_dish_from_every", "One famous dish from every country — the English name, the name in their own script, and the dish in English. Put those words on your menu. The world keeps moving past; pause or search when you want to look."),
    ("find_a_country_or_dish", "Find a country or dish — e.g. India, biryani, Japan"),
    ("one_famous_dish_per_country", "One famous dish per country — the parade loops the whole world."),
    ("how_a_service_runs", "HOW A SERVICE RUNS"),
    ("one_table_start_to_finish", "One table, start to finish."),
    ("watch_a_table_go_from", "Watch a table go from Seated to Paid — the same steps and colours your staff see in the app."),
    ("occupied", "Occupied"),
    ("in_kitchen", "In kitchen"),
    ("food_ready", "Food ready"),
    ("served", "Served"),
    ("bill_printed", "Bill printed"),
    ("paid", "Paid"),
    ("seat_the_table_take_the", "Seat the table, take the order"),
    ("mark_the_table_occupied_the", "Mark the table Occupied the moment guests sit. Add dishes, pick a size, add extras, and type a note for the kitchen — right at the table."),
    ("sizes_and_add_ons", "Sizes and add-ons"),
    ("pick_one_size_add_optional", "Pick one size; add optional extras up to the limit you set."),
    ("a_note_per_dish", "A note per dish"),
    ("less_spicy_travels_with_the", "“Less spicy” travels with the dish it belongs to."),
    ("t4_occupied", "T4 · Occupied"),
    ("note_less_salt", "Note: less salt"),
    ("one_ticket_per_kitchen_station", "One ticket per kitchen station"),
    ("send_the_order_and_each", "Send the order and each station gets only its own dishes — numbered per day, with the table, the waiter and every note. Never a price: the kitchen cooks, the counter bills."),
    ("prints_itself", "Prints itself"),
    ("send_and_the_ticket_prints", "Send, and the ticket prints at the right station — nobody carries it."),
    ("example_ticket", "Example ticket"),
    ("the_kitchen_sees_what_to", "The kitchen sees what to cook now"),
    ("on_the_kitchen_screen_tickets", "On the kitchen screen tickets move New → Preparing → Ready with one tap. Older tickets warm in colour, “To cook now” counts every dish waiting, and mains can hold until the starters go."),
    ("new", "New"),
    ("preparing", "Preparing"),
    ("ready", "Ready"),
    ("example_kitchen_screen", "Example kitchen screen"),
    ("settle_any_way_the_guest", "Settle any way the guest pays"),
    ("cash_with_change_to_return", "Cash with change to return, UPI with the amount already in the QR, part payment, or Due on the customer's khata. Split by item, by seat or evenly. The table frees only when its bill is paid."),
    ("tips_stay_apart", "Tips stay apart"),
    ("a_tip_is_recorded_on", "A tip is recorded on its own — never mixed into sales."),
    ("t4_bill_printed", "T4 · Bill printed"),
    ("bill", "Bill"),
    ("customer_paid_cash", "Customer paid (cash)"),
    ("return_to_customer", "Return to customer"),
    ("close_the_day_you_can", "Close the day you can trust"),
    ("count_the_drawer_against_the", "Count the drawer against the float, cash taken, tips and cash paid out — short or over is named. Print the shift slip, then open the day report by staff, order type, hour and dish, with cancels, comps and tips."),
    ("shift_close", "Shift close"),
    ("opening_float", "Opening float"),
    ("cash_taken", "+ cash taken"),
    ("cash_tips", "+ cash tips"),
    ("cash_paid_out", "− cash paid out"),
    ("expected_in_the_drawer", "Expected in the drawer"),
    ("counted", "Counted"),
    ("short", "Short"),
    ("everything_else_the_floor_needs", "EVERYTHING ELSE THE FLOOR NEEDS"),
    ("built_for_a_real_service", "Built for a real service."),
    ("the_changes_that_happen_every", "The changes that happen every lunch — handled in the open, with who, when and why kept."),
    ("move_and_merge_tables", "Move and merge tables"),
    ("guests_change_tables_the_order", "Guests change tables — the order and its minutes go with them. Two tables, one bill."),
    ("cancel_after_the_kot", "Cancel after the KOT"),
    ("with_a_reason_cooked_or", "With a reason, cooked or not, and who approved it. The kitchen gets a cancel slip."),
    ("complimentary_dishes", "Complimentary dishes"),
    ("a_comp_stays_on_the", "A comp stays on the bill at ₹ 0, with the reason and the approver."),
    ("rights_per_person", "Rights per person"),
    ("each_action_is_allowed_allowed", "Each action is allowed, allowed with a manager PIN, or not allowed — person by person."),
    ("your_menu_your_words", "Your menu, your words"),
    ("search_by_name_or_short", "Search by name or short code, with aliases in another language — “ptk” finds Paneer tikka."),
    ("area_prices_and_happy_hour", "Area prices and happy hour"),
    ("a_dearer_price_in_the", "A dearer price in the AC room; breakfast menus and happy-hour prices by the clock."),
    ("customer_khata", "Customer khata"),
    ("part_payments_and_due_go", "Part payments and Due go on the customer's account in Parties, settled later."),
    ("reports_that_answer_questions", "Reports that answer questions"),
    ("sales_by_staff_order_type", "Sales by staff, order type, hour and dish — with cancels, comps and tips listed."),
    ("start_with_a_menu", "START WITH A MENU"),
    ("your_menu_ready_in_minutes", "Your menu, ready in minutes."),
    ("set_up_your_counter_and", "Set up your counter and pick a starter menu with photos — then change names, prices and codes to your own. Free to download from khaynjo.com."),
    ("starter_menu", "Starter menu"),
    ("khaynjo_starter_menu", "Khaynjo starter menu"),
    ("world_cuisines_japanese_mexican", "World cuisines — Japanese, Mexican, Italian, Indian, Thai and more — each dish with its photo."),
    ("n_14_dishes_11_categories", "14 dishes · 11 categories"),
    ("cafe_starter", "Café starter"),
    ("tea_coffee_cold_drinks_bakery", "Tea, coffee, cold drinks, bakery and snacks — ready for a counter café."),
    ("n_22_dishes_4_categories", "22 dishes · 4 categories"),
    ("photo_shelf", "Photo shelf"),
    ("khaynjo_dish_photos", "Khaynjo dish photos"),
    ("the_full_public_photo_shelf", "The full public photo shelf — pick the picture for any dish you add."),
    ("n_1_459_dish_photos_25", "1,459 dish photos · 25 kinds of kitchen"),
    ("internet_down", "INTERNET DOWN?"),
    ("the_internet_went_down_the", "The internet went down. The counter didn't."),
    ("orders_kitchen_tickets_bills_and", "Orders, kitchen tickets, bills and the day book live on your own devices. When the internet drops in the middle of lunch, nothing in your restaurant waits for it — and there is no account with us to sign in to."),
    ("right_now_mid_lunch", "Right now, mid-lunch"),
    ("router_no_internet", "Router: no internet"),
    ("counter_taking_orders_and_settling", "Counter: taking orders and settling bills"),
    ("day_book_every_bill_saved", "Day book: every bill saved on this device"),
    ("captains_tablets_and_the_kitchen", "Captains' tablets and the kitchen screen: working together on your own Wi-Fi"),
    ("eyebrow_pricing", "PRICING"),
    ("free_forever_on_your_devices", "Free forever on your devices."),
    ("software_capability_is_not_what", "Software capability is not what we sell. Run the whole of Khaynjo on your own devices at no cost — nothing is locked behind a plan."),
    ("khaynjo_on_your_devices", "Khaynjo on your devices"),
    ("forever", "forever"),
    ("every_feature", "Every feature"),
    ("floor_kitchen_billing_day_close", "Floor, kitchen, billing, day close, reports — the full app."),
    ("no_per_seat_charge", "No per-seat charge"),
    ("no_charge_per_table_waiter", "No charge per table, waiter, order or device."),
    ("no_account_with_us", "No account with us"),
    ("day_to_day_service_needs", "Day-to-day service needs no sign-up and no internet."),
    ("kumkaj_cloud_optional", "Kumkaj Cloud · optional"),
    ("yearly_cloud_if_you_want", "Yearly cloud if you want it"),
    ("steady_6_000_swift_12", "Steady ₹6,000 · Swift ₹12,000 · Priority ₹24,000 / year ex-tax (tax at checkout). Custom is a quote — may be below Steady or above Priority. Local Khaynjo stays free forever either way."),
    ("shop_plans_at_shop_sign", "Shop plans at /shop · sign in at /account"),
    ("open_cloud_shop", "Open cloud shop"),
    ("questions", "QUESTIONS"),
    ("asked_before_every_first_service", "Asked before every first service."),
    ("do_i_need_the_internet", "Do I need the internet?"),
    ("no_orders_kots_bills_and", "No. Orders, KOTs, bills and the day book work with no internet. You need it once to download the app and, if you want one, a starter menu."),
    ("where_are_my_bills_kept", "Where are my bills kept?"),
    ("on_your_own_device_in", "On your own device, in your shop — not on our servers. Make backup copies from Settings and keep them wherever you like."),
    ("which_devices_does_it_run", "Which devices does it run on?"),
    ("android_phones_and_tablets_today", "Android phones and tablets today, from Google Play. Windows and Linux packages are on their way."),
    ("does_it_print_kots_and", "Does it print KOTs and bills?"),
    ("yes_on_thermal_printers_you", "Yes — on thermal printers you set up in Settings → Printers, one kitchen ticket per station."),
    ("can_my_staff_use_their", "Can my staff use their own language?"),
    ("yes_the_screens_come_in", "Yes. The screens come in 115+ languages, Arabic and Urdu right-to-left, and dish names can carry aliases in another script."),
    ("what_does_it_cost", "What does it cost?"),
    ("nothing_on_your_own_devices", "Nothing on your own devices — every feature, forever. Optional Kumkaj Cloud is yearly Steady / Swift / Priority (or a Custom quote); open /shop to buy."),
    ("ready_for_lunch", "READY FOR LUNCH?"),
    ("run_your_next_service_on", "Run your next service on Khaynjo."),
    ("free_on_google_play_want", "Free on Google Play. Want a walk-through first? Message us on WhatsApp."),
    ("privacy", "Privacy"),
    ("terms", "Terms"),
    ("cancellation", "Cancellation"),
    ("photo_credits", "Photo credits"),
    ("cloud_shop", "Cloud shop"),
    ("account", "Account"),
    ("cta_installers_note", "Installers go up here once each one passes a clean-machine install. Until then, ask for a demo on WhatsApp."),
    ("cta_whatsapp_soon", "The WhatsApp demo line opens soon. Your message will read: “{message}”."),
    ("shelf_no_match", "No country or dish matches that search."),
    ("shelf_search_note", "Search pauses the parade; clear it to resume or stay paused as you left it."),
    ("floor_meta_min", "{min} min"),
    ("floor_meta_min_bill", "{min} min · ₹ {amount}"),
    ("floor_meta_paid", "₹ {amount} paid"),
    // ---- HTML pages: account, shop (data-i18n; site-i18n.js) and the shop script.
    ("acct_page_title", "Account — Khaynjo"),
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
    ("shop_page_title", "Shop — Khaynjo"),
    ("shop_lede", "Yearly cloud plans — tax added at checkout. The full Khaynjo app on your own devices stays free forever, with no ads."),
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
/// names, numbers and the mock tickets / kitchen cards (example print).
#[cfg_attr(not(test), allow(dead_code))]
pub const UI_SKIP: &[&str] = &[
    "Pill",
    "Claim",
    "Point",
    "Detail",
    "Feature",
    "Question?",
    "Answer.",
    "KOT",
    "Items",
    "Column",
    "Line",
    "Country",
    "Local name",
    "Famous dish",
    "T1",
    "Khaynjo",
    "Gotigin",
    "Kumkaj",
    "kumkaj.com",
    "Google Play",
    "support@gotigin.com",
    "EN",
    "© 2026 GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED",
    "Paneer tikka — Full",
    "+ extra mint chutney",
    "Masala chaas × 2",
    "Butter naan × 4",
    "KOT 27 · HOT KITCHEN",
    "Table T4 · by Captain 1",
    "1  Paneer tikka (Full)",
    "   + extra mint chutney",
    "2  Masala chaas",
    "   note: less salt",
    "4  Butter naan",
    "KOT 31 · T6 · 1 min",
    "2 Veg biryani · 1 Raita",
    "KOT 30 · T2 · 3 min",
    "3 Masala dosa",
    "KOT 27 · T4 · 9 min",
    "1 Paneer tikka · 4 Butter naan",
    "KOT 24 · T1 · 17 min",
    "2 Dal makhani — mains held",
    "KOT 22 · T3 · 21 min",
    "2 Gulab jamun",
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

    #[test]
    fn words_are_marked_where_the_script_has_no_spaces() {
        checks::words_are_marked_in_scripts_without_spaces(DIR);
    }

    /// Every quoted `body:` / `text:` / `empty_text:` string in `ui.rs` is an
    /// `EN` value or listed in `UI_SKIP` (or is blank / numbers / an icon).
    /// Account / shop pages (`data-i18n`) and the shop script (`T("key", "English")`).
    #[test]
    fn html_and_script_copy_match_en() {
        checks::html_and_script_copy_match_en(
            DIR,
            &["static/account/index.html", "static/shop/index.html"],
            &["client/shop-app.js", "client/account-app.js"],
            &en,
        );
        // shop-app.js builds these keys from the catalogue ids.
        for id in ["free", "steady", "swift", "priority", "custom"] {
            assert!(!en(&format!("plan_{id}_blurb")).is_empty(), "plan_{id}_blurb missing");
        }
        assert!(!en("plan_free_title").is_empty() && !en("plan_custom_title").is_empty());
    }

    #[test]
    fn every_ui_string_is_keyed_or_skipped() {
        checks::every_ui_string_is_keyed_or_skipped(include_str!("ui.rs"), &key_for, UI_SKIP);
    }
}

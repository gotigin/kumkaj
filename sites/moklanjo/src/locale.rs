//! moklanjo.com words. English is the source (`EN`, keyed by a stable id);
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
//! `./scripts/scaffold-site-translations.sh moklanjo` (adds the key to every
//! pack as a marked stand-in), then write the translations for real
//! (docs/DEV.md § Languages) — the tests fail while stand-ins remain.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Every site key, in pack order.
pub const EN: &[(&str, &str)] = &[
    ("example", "Example"),
    ("length", "Length"),
    ("download_free", "Download free"),
    ("whatsapp_for_a_demo", "WhatsApp for a demo"),
    ("free_on_android_windows_and", "Free on Android, Windows and Linux."),
    ("courier_operating_system", "courier operating system"),
    ("parcel_journey", "PARCEL JOURNEY"),
    ("walk_in", "Walk-in"),
    ("weigh", "Weigh"),
    ("price", "Price"),
    ("label", "Label"),
    ("hand_over", "Hand over"),
    ("track", "Track"),
    ("offline", "Offline"),
    ("start", "Start"),
    ("moklanjo_courier_operating_system", "MOKLANJO · COURIER OPERATING SYSTEM"),
    ("book_move_and_settle_every", "Book, move and settle every parcel — one courier OS on your own devices."),
    ("parties_books_cod_labels_awb", "Parties, books, COD, labels, AWB pools, tracking, riders and billing — the full counter-to-settled loop, with or without the internet. Free on your devices."),
    ("the_walk_in_loop", "THE WALK-IN LOOP"),
    ("actual_and_volumetric_side_by", "Actual and volumetric, side by side."),
    ("charge", "Charge"),
    ("your_rate_card_cash_or", "Your rate card, cash or account."),
    ("next_booking", "Next booking"),
    ("one_tap_clears_the_counter", "One tap clears the counter for the next walk-in."),
    ("scroll_the_parcel_journey_rail", "Scroll — the parcel journey rail above tracks where you are in the story."),
    ("n_2_weigh_it_right", "2 · WEIGH IT RIGHT"),
    ("actual_or_volumetric_whichever_is", "Actual or volumetric — whichever is higher, rounded your way."),
    ("try_a_box", "Try a box"),
    ("domestic", "Domestic"),
    ("international", "International"),
    ("breadth", "Breadth"),
    ("height", "Height"),
    ("weight_on_the_scale", "Weight on the scale"),
    ("chargeable_weight", "CHARGEABLE WEIGHT"),
    ("the_box_s_size_sets", "The box's size sets the charge."),
    ("higher_of_the_two_rounded", "Higher of the two, rounded up to the next 0.5 kg."),
    ("in_the_app_the_divisor", "In the app the divisor, the rounding step and your units (cm or inches, kg or lb) are the shop's to set. The International chip switches ÷6000 for one booking without touching Settings."),
    ("n_3_price_it", "3 · PRICE IT"),
    ("your_rate_cards_your_party", "Your rate cards, your party rates — and your margin on every docket."),
    ("rate_card", "Rate card"),
    ("zone_service", "Zone · service"),
    ("first_0_5_kg", "First 0.5 kg"),
    ("each_0_5_kg", "Each 0.5 kg"),
    ("local_surface", "Local · Surface"),
    ("within_state_express", "Within state · Express"),
    ("metro_air", "Metro · Air"),
    ("zone_service_weight_slab", "Zone × service × weight slab"),
    ("first_slab_and_each_extra", "First slab and each extra slab, per zone and service."),
    ("a_consignor_s_own_card", "A consignor's own card"),
    ("regular_senders_get_their_rates", "Regular senders get their rates; the default card covers everyone else."),
    ("margin_at_the_desk", "Margin at the desk"),
    ("your_sell_price_against_the", "Your sell price against the carrier's price, on every docket."),
    ("n_4_label_it", "4 · LABEL IT"),
    ("awb_from_your_own_number", "AWB from your own number pools. Label, scan, find."),
    ("number_pools_per_carrier", "Number pools per carrier"),
    ("enter_each_carrier_s_start", "Enter each carrier's start–end series once; every booking takes the next number."),
    ("code_128_labels", "Code 128 labels"),
    ("the_awb_goes_on_the", "The AWB goes on the box as a barcode and in plain digits."),
    ("scan_to_find", "Scan to find"),
    ("scan_a_label_to_open", "Scan a label to open its docket and act on it."),
    ("n_5_hand_over", "5 · HAND OVER"),
    ("hand_it_to_a_carrier", "Hand it to a carrier, or send your own rider. Same system."),
    ("into_a_carrier_bag", "Into a carrier bag"),
    ("the_manifest_holds_exactly_the", "The manifest holds exactly the parcels you picked — all of them or none. A parcel booked with another carrier cannot slip into this bag."),
    ("onto_your_own_rider_s", "Onto your own rider's run"),
    ("pick_the_parcels_print_the", "Pick the parcels, print the run sheet, and the rider works from the same app in rider mode — proof of delivery included."),
    ("n_6_track_and_prove", "6 · TRACK AND PROVE"),
    ("every_move_recorded_and_only", "Every move recorded — and only moves that make sense."),
    ("booked", "Booked"),
    ("with_carrier", "With carrier"),
    ("out_for_delivery", "Out for delivery"),
    ("delivered", "Delivered"),
    ("booked_with_carrier_out_for", "Booked → With carrier → Out for delivery → Delivered"),
    ("failed", "Failed"),
    ("reason_out_again_or_return", "→ reason → out again, or return to sender."),
    ("impossible_moves_refused", "Impossible moves refused"),
    ("delivered_returned_and_cancelled", "Delivered, returned and cancelled are final."),
    ("proof_of_delivery", "Proof of delivery"),
    ("a_photo_and_the_receiver", "A photo and the receiver's signature on the docket."),
    ("failed_deliveries", "Failed deliveries"),
    ("reasons_attempts_out_again_or", "Reasons, attempts, out again, or back to the sender."),
    ("n_7_money_comes_home", "7 · MONEY COMES HOME"),
    ("cod_you_can_see_expected", "COD you can see: expected, handed in, owed to each consignor."),
    ("rider_cash_end_of_run", "Rider cash, end of run"),
    ("rider", "Rider"),
    ("expected", "Expected"),
    ("handed_in", "Handed in"),
    ("a_14_drops", "A · 14 drops"),
    ("b_9_drops", "B · 9 drops"),
    ("short_200_on_rider_b", "Short ₹ 200 on Rider B — named, not lost in a total."),
    ("cod_register", "COD register"),
    ("collected_against_paid_out_per", "Collected against paid out, per consignor, with remittance batches."),
    ("rider_settlement", "Rider settlement"),
    ("expected_against_handed_in_and", "Expected against handed in, and the commission slip."),
    ("books_that_close", "Books that close"),
    ("day_book_month_bill_per", "Day book, month bill per account, and a GST summary CSV for your accountant."),
    ("n_8_internet_down", "8 · INTERNET DOWN?"),
    ("keep_booking", "Keep booking."),
    ("booking_labels_scanning_cod_and", "Booking, labels, scanning, COD and the books all run on your devices. When the internet drops, nothing at the counter waits for it."),
    ("office_router_no_internet", "Office router: no internet"),
    ("desk_still_booking", "Desk: still booking"),
    ("second_counter_and_rider_phone", "Second counter and rider phone: joined over the office Wi-Fi"),
    ("free_forever_on_your_devices", "Free forever on your devices"),
    ("nothing_per_rider_user_or", "Nothing per rider, user or parcel. Optional Kumkaj Cloud: Steady ₹6k · Swift ₹12k · Priority ₹24k/yr ex-tax — /shop."),
    ("n_9_start", "9 · START"),
    ("one_courier_os_your_parcels", "One courier OS. Your parcels. Your books."),
    ("moklanjo_is_a_complete_courier", "Moklanjo is a complete courier operating system under Kumkaj — Android, Windows and Linux. Riders use the same app in rider mode."),
    ("trademark", "TRADEMARK"),
    ("memon_for_find_it_pronounced", "Memon for “find it” · pronounced go-tee-geen"),
    ("company", "COMPANY"),
    ("brand", "BRAND"),
    ("concrete_business_operating_system", "Concrete business operating system apps"),
    ("product", "PRODUCT"),
    ("courier_operating_system_under", "Courier operating system under Kumkaj"),
    ("privacy", "Privacy"),
    ("terms", "Terms"),
    ("cancellation", "Cancellation"),
    ("cloud_shop", "Cloud shop"),
    ("account", "Account"),
    ("weigh_it_right", "Weigh it right"),
    ("price_it", "Price it"),
    ("label_it", "Label it"),
    ("track_and_prove", "Track and prove"),
    ("money_comes_home", "Money comes home"),
    ("internet_down", "Internet down"),
    ("bead_hand", "Hand"),
    ("bead_offline_short", "Offln"),
    ("weight_cm", "{n} cm"),
    ("weight_kg", "{kg} kg"),
    ("weight_working", "{l} × {b} × {h} ÷ {d} = {kg} kg"),
    ("weight_on_scale", "On the scale: {kg} kg"),
    ("weight_scale_wins", "The weight on the scale sets the charge."),
    ("cta_installers_note", "Installers go up here once each one passes a clean-machine install. Until then, ask for a demo on WhatsApp."),
    ("cta_whatsapp_soon", "The WhatsApp demo line opens soon. Your message will read: “{message}”."),
    ("acct_page_title", "Account — Moklanjo"),
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
    ("shop_page_title", "Shop — Moklanjo"),
    ("shop_lede", "Yearly cloud plans — tax added at checkout. The full Moklanjo app on your own devices stays free forever, with no ads."),
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
    "Fact",
    "Detail",
    "Choice",
    "Step",
    "Moklanjo",
    "1 / 9 · Walk-in",
    "30 cm",
    "2.0 kg",
    "30 × 20 × 15 ÷ 5000 = 1.8 kg",
    "On the scale: 1.2 kg",
    "7004 1182 35",
    "TO  Pune 411001  ·  1 piece  ·  2.0 kg chargeable",
    "Gotigin",
    "Where Technology Meets Discovery",
    "GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED",
    "support@gotigin.com",
    "Director · mearaj@gmail.com",
    "Kumkaj",
    "© 2026 GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED",
    "kumkaj.com  →",
    "AWB",
    "COD",
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

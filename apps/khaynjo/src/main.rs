// Production Windows: GUI subsystem (no console). Dev keeps a console for logs.
// `kumkaj_hide_console` is set by build.rs when KUMKAJ_ENV=production.
#![cfg_attr(
    all(target_os = "windows", kumkaj_hide_console),
    windows_subsystem = "windows"
)]

pub use makepad_widgets;

// Android compiles this file into the library (`crate`). Desktop binary
// links the same package's library as `kumkaj_khaynjo`.
#[cfg(target_os = "android")]
use crate::{
    diag, domain, entity_template, fiscal, locale, local_servers, menu_images, net, notify, phone,
    precise, print_share, schemes, snack, startup, store, suggest, thumbs, upi_qr,
};
#[cfg(not(target_os = "android"))]
use kumkaj_khaynjo::{
    diag, domain, entity_template, fiscal, locale, local_servers, menu_images, net, notify, phone,
    precise, print_share, schemes, snack, startup, store, suggest, thumbs, uninstall_wipe, upi_qr,
};
use snack::SnackKind;

use domain::{
    new_id, AttachmentEntity, Cart, CartLine, Customer, DiningTable, KotStatus, Money, MoneyLane,
    MoneyMoveKind, OrderType, PayMethod, Product, Sale, SaleRevision, StaffFeatures, StaffMember,
    StaffRole, VenueMode,
};
use fiscal::{FiscalSettings, TaxPricing};
use locale::{LanguageSource, LocaleSettings, CURRENCIES, LANGUAGES};
use local_servers::LocalServerInfo;
use store::{default_backup_path, Store};
use makepad_widgets::makepad_platform::event::finger::TouchState;
use makepad_widgets::*;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

// Desktop: intercept `--uninstall-wipe …` before Makepad opens a window
// (MSI / "Delete books data" call this while the exe is still installed).
#[cfg(not(target_os = "android"))]
fn main() {
    if uninstall_wipe::try_run() {
        return;
    }
    desktop_entry::app_main();
}

#[cfg(not(target_os = "android"))]
mod desktop_entry {
    use super::App;
    use makepad_widgets::*;
    app_main!(App);
}

#[cfg(target_os = "android")]
app_main!(App);

// Logical modules (runtime, app_state, widgets, …) — one scope via include!.
include!("runtime.rs");
include!("app_state.rs");
include!("types.rs");
include!("info_help.rs");
include!("amount_visibility.rs");
include!("app_state_impl.rs");
include!("ui_script.rs");
include!("helpers.rs");
include!("widgets/product_menu_floor.rs");
include!("widgets/report_sidebar.rs");
include!("widgets/pick_lists.rs");
include!("helpers_books.rs");
include!("widgets/catalog_cart.rs");
include!("widgets/sales_people.rs");
include!("widgets/misc_lists.rs");
include!("app/defs.rs");
include!("app/impl_01.rs");
include!("app/impl_02.rs");
include!("app/impl_03.rs");
include!("app/impl_04.rs");
include!("app/impl_05.rs");
include!("app/impl_06.rs");
include!("app/impl_07.rs");
include!("app/impl_08.rs");
include!("helpers_image.rs");
include!("app/chevrons.rs");
include!("app/match_event.rs");
include!("app/main_impl.rs");
include!("shell_tests/nav_order_tests.rs");
include!("shell_tests/ux_rules_tests.rs");
include!("shell_tests/menu_window_tests.rs");
include!("shell_tests/floor_tests.rs");

// Production Windows: GUI subsystem (no console). Dev keeps a console for logs.
// `kumkaj_hide_console` is set by build.rs when KUMKAJ_ENV=production.
#![cfg_attr(
    all(target_os = "windows", kumkaj_hide_console),
    windows_subsystem = "windows"
)]

pub use makepad_widgets;

// Android compiles this file into the library (`crate`). Desktop binary
// links the same package's library as `kumkaj_moklanjo`.
#[cfg(target_os = "android")]
use crate::{
    booking, courier_ops_view, diag, domain, entity_template, fiscal, local_servers, locale,
    menu_images, net, notify, phone, precise, print_share, rates, snack, startup, store, suggest,
    thumbs,
};
#[cfg(not(target_os = "android"))]
use kumkaj_moklanjo::{
    booking, courier_ops_view, diag, domain, entity_template, fiscal, local_servers, locale,
    menu_images, net, notify, phone, precise, print_share, rates, snack, startup, store, suggest,
    thumbs, uninstall_wipe,
};
use snack::SnackKind;

use domain::{
    AttachmentEntity, Cart, Customer, Money, MoneyLane, MoneyMoveKind, PayMethod, Product, Sale,
    SaleRevision, StaffFeatures, StaffMember, StaffRole, new_id,
};
use fiscal::{FiscalSettings, TaxPricing};
use local_servers::LocalServerInfo;
use locale::{CURRENCIES, LANGUAGES, LanguageSource, LocaleSettings};
use makepad_widgets::makepad_platform::event::finger::TouchState;
use makepad_widgets::*;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use store::{Store, default_backup_path};

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
include!("amount_visibility.rs");
include!("app_state_impl.rs");
include!("ui_script.rs");
include!("helpers.rs");
include!("helpers_books.rs");
include!("widgets/product_menu_floor.rs");
include!("widgets/sales_people.rs");
include!("widgets/pick_lists.rs");
include!("widgets/misc_lists.rs");
include!("widgets/report_sidebar.rs");
include!("info_help.rs");
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
include!("app/impl_ha_01.rs");
include!("app/impl_ha_02.rs");
include!("app/impl_ha_03.rs");
include!("app/impl_ha_04.rs");
include!("app/impl_ha_05.rs");
include!("app/impl_ha_06.rs");
include!("app/match_misc.rs");
include!("app/main_impl.rs");
include!("shell_tests/nav_order_tests.rs");
include!("shell_tests/report_text_tests.rs");

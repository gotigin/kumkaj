// Production Windows: GUI subsystem (no console). Dev keeps a console for logs.
// `kumkaj_hide_console` is set by build.rs when KUMKAJ_ENV=production.
#![cfg_attr(
    all(target_os = "windows", kumkaj_hide_console),
    windows_subsystem = "windows"
)]

pub use makepad_widgets;

// Android compiles this file into the library. The binary refers to that
// library as `kumkaj`; inside the library the same path is `crate`.
#[cfg(target_os = "android")]
use crate as kumkaj;

use kumkaj::domain::{
    new_id, Cart, Customer, DiningTable, KotTicket, Money, MoneyLane, MoneyMoveKind, OrderType,
    PayMethod, Product, Sale, SaleRevision, StaffFeatures, StaffMember, StaffRole, VenueMode,
};
use kumkaj::entity_template;
use kumkaj::{domain, fiscal, locale, local_servers};
use kumkaj::fiscal::{FiscalSettings, TaxPricing};
use kumkaj::locale::{LanguageSource, LocaleSettings, CURRENCIES, LANGUAGES};
use kumkaj::local_servers::LocalServerInfo;
use kumkaj::menu_images;
use kumkaj::net;
use kumkaj::print_share;
use kumkaj::store::{self, default_backup_path, Store};
use makepad_widgets::makepad_platform::event::finger::TouchState;
use makepad_widgets::*;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

app_main!(App);

fn app_state() -> &'static Mutex<AppState> {
    static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(AppState::boot()))
}

/// Pending desktop window restore. Applied at Startup via `configure_window`,
/// then re-checked on the first `WindowGeomChange` (Windows sometimes creates
/// with the DSL default and ignores create_inner_size until a post-create resize).
fn pending_window_geom() -> &'static Mutex<Option<store::WindowGeomPrefs>> {
    static PENDING: OnceLock<Mutex<Option<store::WindowGeomPrefs>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(None))
}

/// Edge-swipe tracking: left edge → back, right edge → forward.
#[derive(Clone, Copy)]
enum SwipeNav {
    Back,
    Forward,
}

fn swipe_origin() -> &'static Mutex<Option<(u64, DVec2, SwipeNav)>> {
    static ORIGIN: OnceLock<Mutex<Option<(u64, DVec2, SwipeNav)>>> = OnceLock::new();
    ORIGIN.get_or_init(|| Mutex::new(None))
}

/// Desktop mouse/trackpad edge swipe (Makepad has no Event::FingerUp at app level).
fn mouse_swipe_origin() -> &'static Mutex<Option<(DVec2, SwipeNav)>> {
    static ORIGIN: OnceLock<Mutex<Option<(DVec2, SwipeNav)>>> = OnceLock::new();
    ORIGIN.get_or_init(|| Mutex::new(None))
}

/// Linux often emits both MouseButton::BACK and XF86Back for one click.
fn nav_gesture_ok() -> bool {
    static LAST: OnceLock<Mutex<Option<std::time::Instant>>> = OnceLock::new();
    let last = LAST.get_or_init(|| Mutex::new(None));
    let mut guard = last.lock().unwrap();
    let now = std::time::Instant::now();
    if let Some(prev) = *guard
        && now.duration_since(prev) < Duration::from_millis(140)
    {
        return false;
    }
    *guard = Some(now);
    true
}

/// Void and refund are Owner/Manager work. A cashier alone at
/// the counter should not have to hand the device over or log the manager in
/// mid-queue, so an Owner/Manager PIN typed on the spot stands in: the approver
/// becomes the recorded actor, and the note says who asked for it.
///
/// Returns the actor to record and the reason to store. When the actor may
/// already do this, nothing changes and no PIN is read.
fn approve_for(
    store: &Store,
    actor: &StaffMember,
    pin: &str,
    reason: &str,
) -> Result<(StaffMember, String), String> {
    if actor.role.can_void_refund() {
        return Ok((actor.clone(), reason.trim().to_string()));
    }
    let approver = store.verify_approval_pin(pin)?;
    Ok((
        approver.clone(),
        format!(
            "{} · approved by {} for {}",
            reason.trim(),
            approver.name,
            actor.name
        ),
    ))
}

fn store_client_paired(store: Option<&Store>) -> bool {
    store
        .and_then(|s| s.client_paired().ok())
        .unwrap_or(false)
}

/// After soft-keyboard DidHide, WindowGotFocus may restore the TextInput.
/// Suppress that for one event cycle so the keyboard stays down.
fn ime_suppress_refocus() -> &'static Mutex<bool> {
    static FLAG: OnceLock<Mutex<bool>> = OnceLock::new();
    FLAG.get_or_init(|| Mutex::new(false))
}

struct AppState {
    store: Option<Store>,
    products: Vec<Product>,
    cart: Cart,
    /// Editable copy of the selected sale for owner/manager amend.
    amend_cart: Cart,
    amend_pay: PayMethod,
    sale_revisions: Vec<SaleRevision>,
    selected_table_id: Option<String>,
    selected_customer_id: Option<String>,
    customers: Vec<Customer>,
    tables: Vec<DiningTable>,
    kots: Vec<KotTicket>,
    recent_sales: Vec<Sale>,
    staff: Vec<StaffMember>,
    session: Option<StaffMember>,
    /// Lock screen: Owner PIN vs selected Staff PIN.
    lock_as_owner: bool,
    /// Selected non-owner staff id when unlocking as Staff.
    lock_staff_id: Option<String>,
    /// Permission preset when owner adds staff (Team screen).
    draft_staff_role: StaffRole,
    draft_features: StaffFeatures,
    /// Staff id selected on Team for feature edits.
    team_staff_id: Option<String>,
    /// Whether this device holds the books (shop/server) or is an extra till.
    device_role: DeviceRole,
    /// True only until the owner picks shop or extra till once.
    must_confirm_device_role: bool,
    /// DeviceSetup / Settings: follow OS language vs a fixed catalogue pick.
    language_follow_platform: bool,
    /// True while Server Mode is waiting for create / pick among local servers.
    picking_server: bool,
    /// Local servers registered on this device (multi-business).
    local_servers: Vec<LocalServerInfo>,
    /// App data root (desktop `data_dir` or Android files dir).
    data_base: Option<PathBuf>,
    /// Shop device: extra tills may pair on LAN. Off for a new shop.
    extra_tills_allowed: bool,
    /// False until Owner finishes first-run PIN setup (Server shop).
    owner_setup_complete: bool,
    /// False until Fresh/Book onboarding finishes.
    onboarding_complete: bool,
    venue_mode: VenueMode,
    client_paired: bool,
    /// Book-import wizard step: 0=choose, 1=items hint, 2=credit opening.
    onboard_step: u8,
    /// Active start template id (`custom`, `courier_agency`, …).
    entity_template_id: String,
    /// Bill type for the open cart (Account booking / Tax invoice vs Cash booking / Cash memo).
    money_lane: MoneyLane,
    /// Buy tab: purchase posts to AP when true.
    buy_on_account: bool,
    day_close_text: String,
    /// Inventory screen body (low stock + recent movements).
    ops_panel_text: String,
    /// Branches screen body (org, branch list, roll-up).
    branches_text: String,
    /// Kitchen station filter; None = every station.
    selected_station_id: Option<String>,
    screen: Screen,
    status: String,
    last_receipt: String,
    /// Sale id eligible for the post-charge undo snackbar (cleared on dismiss/timeout/undo).
    undo_sale_id: Option<String>,
    undo_snack_text: String,
    undo_timer: Timer,
    pin_buffer: String,
    /// When true, lock screen shows digits instead of bullets.
    pin_visible: bool,
    selected_sale_id: Option<String>,
    locale: LocaleSettings,
    /// Country pack in force: tax wording and maths, pay-rail name, id label.
    fiscal: FiscalSettings,
    /// Code the seller is browsing with ‹ › on Settings, before they apply
    /// it. Empty means the hint describes the pack actually in force.
    country_browse: String,
    /// In-app venue name (empty → neutral "Your shop"). OS / package name stays Kumkaj.
    venue_title: String,
    /// `custom:…` under brand_images/, or None when no in-app mark is set.
    venue_icon_key: Option<String>,
    /// Pending custom icon path from picker (before Save brand).
    draft_venue_icon_path: Option<PathBuf>,
    /// None = auto-detect from name; Some(key) = built-in or custom:…
    draft_image_key: Option<String>,
    /// Catalogue id loaded into the Items form. None means the form will add.
    editing_product_id: Option<String>,
    /// True from the moment a charge starts until it succeeds or fails. A
    /// double-tapped Cash button used to bill the cart twice.
    pay_in_flight: bool,
    /// Consecutive wrong PINs on this process. Reset by any success.
    failed_unlocks: u32,
    /// While set, unlock is refused — brute-forcing a short PIN should not be
    /// a matter of seconds. In memory only: the till is local, and a restart
    /// costing an attacker the shop's own downtime is fine.
    unlock_blocked_until: Option<std::time::Instant>,
    /// Owner must choose a new PIN before the till opens (post-migration reset).
    pin_reset_required: bool,
    /// Browser-style screen history for Back / Forward (mouse, swipe, keys).
    nav_back: Vec<Screen>,
    nav_forward: Vec<Screen>,
}

/// Wrong PINs tolerated before [`UNLOCK_BLOCK`] kicks in.
const UNLOCK_ATTEMPT_LIMIT: u32 = 5;
const UNLOCK_BLOCK: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeviceRole {
    Unset,
    Server,
    Client,
}

impl DeviceRole {
    fn from_meta(value: Option<&str>) -> Self {
        match value.map(str::trim) {
            Some("server") | Some("standalone") => Self::Server,
            Some("client") => Self::Client,
            _ => Self::Unset,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Client => "client",
            Self::Unset => "",
        }
    }

    fn is_host(self) -> bool {
        matches!(self, Self::Server)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    DeviceSetup,
    Lock,
    Onboard,
    ClientPair,
    /// Books home — today's position (money in/out, owed, cash vs bank).
    Books,
    Pos,
    /// Money out — purchase + expense.
    Buy,
    Menu,
    More,
    Floor,
    Kitchen,
    Sales,
    Credit,
    Team,
    Settings,
    DayClose,
    Reports,
    Inventory,
    Branches,
}

/// Post-unlock home from cross-category commons ([`docs/DESIGN.md`]).
fn unlocked_home() -> Screen {
    Screen::Books
}

fn screen_keeps_history(screen: Screen) -> bool {
    !matches!(
        screen,
        Screen::DeviceSetup | Screen::Lock | Screen::Onboard | Screen::ClientPair
    )
}

impl AppState {
    fn boot() -> Self {
        let mut state = Self {
            store: None,
            products: Vec::new(),
            cart: Cart::default(),
            amend_cart: Cart::default(),
            amend_pay: PayMethod::Cash,
            sale_revisions: Vec::new(),
            selected_table_id: None,
            selected_customer_id: None,
            customers: Vec::new(),
            tables: Vec::new(),
            kots: Vec::new(),
            recent_sales: Vec::new(),
            staff: Vec::new(),
            session: None,
            lock_as_owner: true,
            lock_staff_id: None,
            draft_staff_role: StaffRole::Cashier,
            draft_features: StaffFeatures::for_role(StaffRole::Cashier),
            team_staff_id: None,
            device_role: DeviceRole::Unset,
            must_confirm_device_role: true,
            language_follow_platform: true,
            picking_server: false,
            local_servers: Vec::new(),
            data_base: None,
            owner_setup_complete: false,
            onboarding_complete: false,
            venue_mode: VenueMode::Counter,
            client_paired: false,
            extra_tills_allowed: false,
            onboard_step: 0,
            entity_template_id: "custom".into(),
            money_lane: MoneyLane::Declared,
            buy_on_account: false,
            day_close_text: String::new(),
            ops_panel_text: String::new(),
            branches_text: String::new(),
            selected_station_id: None,
            screen: Screen::DeviceSetup,
            status: String::new(),
            last_receipt: "No sale yet".into(),
            undo_sale_id: None,
            undo_snack_text: String::new(),
            undo_timer: Timer::empty(),
            pin_buffer: String::new(),
            pin_visible: false,
            selected_sale_id: None,
            locale: LocaleSettings::default(),
            fiscal: FiscalSettings::default(),
            country_browse: String::new(),
            venue_title: String::new(),
            venue_icon_key: None,
            draft_venue_icon_path: None,
            draft_image_key: None,
            editing_product_id: None,
            pay_in_flight: false,
            failed_unlocks: 0,
            unlock_blocked_until: None,
            pin_reset_required: false,
            nav_back: Vec::new(),
            nav_forward: Vec::new(),
        };
        if let Ok(dir) = store::data_dir() {
            state.data_base = Some(dir.clone());
            let _ = state.refresh_local_servers();
        }
        match Store::open_default() {
            Ok(store) => {
                state.status = format!("Local DB: {}", store.db_path().display());
                state.store = Some(store);
                let _ = state.reload_all();
                // Always require PIN after a process start. Session restore was
                // skipping the lock screen and jumping straight into Sell.
                if let Some(store) = state.store.as_mut() {
                    let _ = store.set_active_session_staff_id(None);
                }
                state.session = None;
                state.pin_buffer.clear();
                state.route_after_stored_role();
            }
            Err(e) => {
                // No local server yet is normal — DeviceSetup will create/pick.
                state.status = if e.contains("No local server") {
                    String::new()
                } else {
                    format!("Storage: {e}")
                };
                state.session = None;
                state.must_confirm_device_role = true;
                state.apply_platform_language_preview();
                state.screen = Screen::DeviceSetup;
                state.pin_buffer.clear();
            }
        }
        state
    }

    fn refresh_local_servers(&mut self) -> Result<(), String> {
        let base = self
            .data_base
            .clone()
            .or_else(|| store::data_dir().ok())
            .ok_or_else(|| "No data directory".to_string())?;
        self.data_base = Some(base.clone());
        self.local_servers = local_servers::list_local_servers(&base)?;
        Ok(())
    }

    fn reload_all(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        self.products = store.list_products(false)?;
        self.tables = store.list_tables()?;
        self.kots = match self.selected_station_id.as_deref() {
            Some(station_id) => store.list_open_kots_for_station(station_id)?,
            None => store.list_open_kots()?,
        };
        self.recent_sales = store.recent_sales(30)?;
        self.staff = store.list_staff()?;
        self.customers = store.list_customers().unwrap_or_default();
        self.locale = store.locale_settings()?;
        self.fiscal = store.fiscal_settings()?;
        self.device_role = DeviceRole::from_meta(store.device_role()?.as_deref());
        self.language_follow_platform =
            store.language_source().unwrap_or(LanguageSource::User) == LanguageSource::Platform;
        self.extra_tills_allowed = store.extra_tills_allowed().unwrap_or(false);
        self.venue_title = store.venue_title()?;
        self.venue_icon_key = store.venue_icon_key()?;
        self.owner_setup_complete = store.owner_setup_complete().unwrap_or(true);
        self.onboarding_complete = store.onboarding_complete().unwrap_or(true);
        self.venue_mode = store.venue_mode().unwrap_or(VenueMode::Counter);
        self.client_paired = store.client_paired().unwrap_or(false);
        self.pin_reset_required = store.pin_reset_required().unwrap_or(false);
        self.entity_template_id = store
            .entity_template_id()
            .unwrap_or_else(|_| "custom".into());
        Ok(())
    }

    fn uses_lan(&self) -> bool {
        self.device_role == DeviceRole::Server && self.extra_tills_allowed
    }

    /// Every process start: language + Server/Client on one screen.
    /// UI opens in the platform language so non-English users can read it;
    /// changing language updates this screen immediately.
    fn route_after_stored_role(&mut self) {
        self.must_confirm_device_role = true;
        self.picking_server = false;
        let _ = self.refresh_local_servers();
        self.apply_platform_language_preview();
        self.screen = Screen::DeviceSetup;
        self.status = match self.device_role {
            DeviceRole::Server => self.t("last_server_mode"),
            DeviceRole::Client => self.t("last_client_mode"),
            DeviceRole::Unset => self.t("device_use_q"),
        };
        // Warm LAN only after they confirm Server + extra tills on Lock path.
        // Do not start the server while the chooser is open.
    }

    /// DeviceSetup always previews in the OS language until the user picks otherwise.
    fn apply_platform_language_preview(&mut self) {
        self.language_follow_platform = true;
        let platform = locale::detect_platform_language();
        self.locale.language = platform;
    }

    /// Guard for anything the shop device alone may write. An extra till's catalogue,
    /// stock, staff and settings tables are a sync cache, so editing them here
    /// would vanish on the next snapshot.
    fn require_server_role(&self, what: &str) -> Result<(), String> {
        if self.device_role == DeviceRole::Client {
            return Err(format!("{what} on the shop device — this is an extra till"));
        }
        Ok(())
    }

    /// LAN details a Client needs to ask the Server to do the writing.
    fn server_link(&self) -> Result<(String, String), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let url = store
            .server_url()?
            .ok_or_else(|| "Not paired to a Server".to_string())?;
        Ok((url, store.pairing_code()?))
    }

    /// Item lookup for the ops screens: exact name first, then unique-ish
    /// "contains" so sellers can type "chai" for "Masala Chai".
    fn find_product_by_name(&self, name: &str) -> Result<Product, String> {
        let needle = name.trim();
        if needle.is_empty() {
            return Err("Type an item name first".into());
        }
        let lower = needle.to_lowercase();
        let exact: Vec<&Product> = self
            .products
            .iter()
            .filter(|p| p.matches_label(needle))
            .collect();
        if exact.len() > 1 {
            return Err(format!(
                "More than one item is called \"{needle}\". Names are labels; pick the item on the list."
            ));
        }
        if let Some(product) = exact.first() {
            return Ok((*product).clone());
        }
        let partial: Vec<&Product> = self
            .products
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&lower)
                    || p.aliases()
                        .iter()
                        .any(|alias| alias.to_lowercase().contains(&lower))
            })
            .collect();
        if partial.len() > 1 {
            return Err(format!(
                "\"{needle}\" matches more than one item. Type more of the name."
            ));
        }
        partial
            .first()
            .cloned()
            .cloned()
            .ok_or_else(|| format!("No item matches \"{needle}\""))
    }

    fn refresh_inventory_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let low = store.list_low_stock(5)?;
        let movements = store.list_stock_movements(12)?;
        let tracked: Vec<&Product> = self.products.iter().filter(|p| p.tracks_stock()).collect();
        let mut lines = Vec::new();
        if tracked.is_empty() {
            lines.push("No item is stock tracked yet — type a name and tap Start tracking.".into());
        } else {
            lines.push("On hand:".to_string());
            for p in tracked.iter().take(20) {
                lines.push(format!("  {} · {}", p.name, p.stock_qty));
            }
        }
        lines.push(String::new());
        if low.is_empty() {
            lines.push("Low stock: nothing at or below 5.".to_string());
        } else {
            lines.push("Low stock (5 or less):".to_string());
            for p in &low {
                lines.push(format!("  {} · {} left", p.name, p.stock_qty));
            }
        }
        lines.push(String::new());
        if movements.is_empty() {
            lines.push("No stock movements yet.".to_string());
        } else {
            lines.push("Recent movements:".to_string());
            for m in &movements {
                let when = &m.created_at[..m.created_at.len().min(16)];
                let sign = if m.delta_qty >= 0 { "+" } else { "" };
                let note = if m.note.trim().is_empty() {
                    String::new()
                } else {
                    format!(" ({})", m.note.trim())
                };
                lines.push(format!(
                    "  {when} {} {sign}{} {}{note}",
                    m.product_name, m.delta_qty, m.reason
                ));
            }
        }
        let purchases = store.list_purchases(5)?;
        lines.push(String::new());
        if purchases.is_empty() {
            lines.push("No purchases yet.".to_string());
        } else {
            lines.push("Recent purchases:".to_string());
            for purchase in &purchases {
                let line_count = store.purchase_lines(&purchase.id)?.len();
                let supplier = if purchase.supplier.trim().is_empty() {
                    "Supplier"
                } else {
                    purchase.supplier.trim()
                };
                lines.push(format!(
                    "  {supplier} · {} · {line_count} lines",
                    self.money(purchase.total())
                ));
            }
        }
        self.ops_panel_text = lines.join("\n");
        Ok(())
    }

    /// Today's cash-up, with the last recorded close underneath it so the owner
    /// can tell a fresh day from one already signed off.
    fn refresh_day_close_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let mut text = store.day_close_summary()?;
        match store.last_day_close_at()? {
            Some(at) => {
                let shown = at[..at.len().min(19)].replace('T', " ");
                text.push_str(&format!("\nLast recorded close: {shown}\n"));
            }
            None => text.push_str("\nNo day close recorded yet.\n"),
        }
        self.day_close_text = text;
        Ok(())
    }

    /// Books home: money in/out, AR/AP, cash vs bank (not sales-only day close).
    fn refresh_books_overview(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        self.day_close_text = store.books_overview_text()?;
        Ok(())
    }

    fn refresh_branches_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let org = store.org_name()?;
        let branches = store.list_branches()?;
        let active_id = store.active_branch_id()?;
        let mut lines = vec![format!("Business: {org}"), String::new(), "Branches:".to_string()];
        for b in &branches {
            let marker = if b.id == active_id { "●" } else { "○" };
            let state = if b.is_this_device || b.id == active_id {
                "selling from this device".to_string()
            } else if b.last_sync_at.trim().is_empty() {
                "no snapshot imported yet".to_string()
            } else {
                format!(
                    "snapshot {}",
                    &b.last_sync_at[..b.last_sync_at.len().min(16)]
                )
            };
            lines.push(format!("  {marker} {} · {state}", b.label()));
        }
        self.branches_text = lines.join("\n");
        Ok(())
    }

    fn display_venue_title(&self) -> String {
        let t = self.venue_title.trim();
        if t.is_empty() {
            "Your shop".into()
        } else {
            t.to_string()
        }
    }

    /// Owner-chosen in-app mark only. Never falls back to the Kumkaj package icon.
    fn venue_icon_path(&self) -> Option<PathBuf> {
        match self.venue_icon_key.as_deref() {
            Some(key) if key.starts_with("custom:") => {
                let file = &key["custom:".len()..];
                self.store
                    .as_ref()
                    .and_then(|s| s.brand_images_dir().ok())
                    .map(|d| d.join(file))
                    .filter(|p| p.is_file())
            }
            _ => None,
        }
    }

    fn actor(&self) -> Result<StaffMember, String> {
        self.session
            .clone()
            .ok_or_else(|| "Unlock with PIN first".to_string())
    }

    fn money(&self, m: Money) -> String {
        m.format_localized(&self.locale.currency, &self.fiscal)
    }

    /// Decimal mark the seller's country writes money with. Edit fields are
    /// filled with it; parsing accepts either mark regardless.
    fn decimal_sep(&self) -> &str {
        self.fiscal.decimal_sep()
    }

    /// Tax the open cart will have added at the till. Always 0 for a shop that
    /// prices tax-inclusive, where the strip above already shows the real bill.
    fn cart_tax_estimate_paise(&self) -> i64 {
        if self.fiscal.tax_pricing.is_inclusive() {
            return 0;
        }
        self.cart
            .lines
            .iter()
            .map(|line| {
                let bps = self
                    .products
                    .iter()
                    .find(|p| p.id == line.product_id)
                    .map(|p| p.tax_bps)
                    .unwrap_or(0);
                fiscal::tax_on_amount(line.applied_total().paise(), bps, TaxPricing::Exclusive)
            })
            .sum()
    }

    /// Today's ops report, worded for the shop's country pack.
    fn ops_report_text(&self, report: &domain::OpsReport) -> String {
        let tax_caption = if self.fiscal.tax_pricing.is_inclusive() {
            format!("{} inside today's sales", self.fiscal.tax_label())
        } else {
            format!("{} collected today", self.fiscal.tax_label())
        };
        let mut lines = vec![
            format!("Sales today: {}", report.day.sale_count),
            format!("Cash {}", self.money(Money(report.day.cash_paise))),
            format!(
                "{} {}",
                self.fiscal.digital_pay_label(),
                self.money(Money(report.day.upi_paise))
            ),
            format!("Card {}", self.money(Money(report.day.card_paise))),
            format!(
                "{} sales {}",
                self.fiscal.credit_label(),
                self.money(Money(report.day.credit_paise))
            ),
            format!(
                "{} {}",
                self.lane_declared_label(),
                self.money(Money(report.day.declared_paise))
            ),
            format!(
                "{} {}",
                self.lane_owner_label(),
                self.money(Money(report.day.owner_lane_paise))
            ),
            format!(
                "Still owed {}",
                self.money(Money(report.outstanding_credit_paise))
            ),
            format!(
                "Expenses {} ({})",
                self.money(Money(report.expenses_paise)),
                report.expense_count
            ),
            format!(
                "Bank net today {}",
                self.money(Money(report.bank_net_paise))
            ),
            format!("{tax_caption} {}", self.money(Money(report.day.tax_paise))),
            "Top items:".into(),
        ];
        for (name, qty, rev) in &report.top_items {
            lines.push(format!("  {name} ×{qty} · {}", self.money(Money(*rev))));
        }
        lines.push("Use Month / Aging / Bank below for full books".into());
        lines.join("\n")
    }

    /// One line under the totals strip: silent unless exclusive tax is about
    /// to change what the customer actually pays.
    fn cart_tax_estimate_line(&self) -> String {
        let tax = self.cart_tax_estimate_paise();
        let service = self.fiscal.service_on(self.cart.line_sum().paise());
        if tax <= 0 && service <= 0 {
            return " ".to_string();
        }
        let mut parts = Vec::new();
        if tax > 0 {
            parts.push(format!("+ {} {}", self.fiscal.tax_label(), self.money(Money(tax))));
        }
        if service > 0 {
            parts.push(format!("+ Service {}", self.money(Money(service))));
        }
        format!(
            "{} · pay {}",
            parts.join(" "),
            self.money(Money(self.cart.rounded_total().paise() + tax + service))
        )
    }

    fn t(&self, key: &str) -> String {
        let tmpl = entity_template::resolve(&self.entity_template_id);
        locale::t_overlay(&self.locale.language, key, tmpl.label_overrides)
    }

    fn lane_declared_label(&self) -> String {
        entity_template::resolve(&self.entity_template_id)
            .lane_declared_label
            .into()
    }

    fn lane_owner_label(&self) -> String {
        entity_template::resolve(&self.entity_template_id)
            .lane_owner_label
            .into()
    }

    fn today_label(&self) -> String {
        match self.store.as_ref().and_then(|s| s.today_totals().ok()) {
            Some(t) => {
                let outstanding = self
                    .store
                    .as_ref()
                    .and_then(|s| s.outstanding_credit_total().ok())
                    .unwrap_or(0);
                let mut lines = vec![format!(
                    "{}: {} · {}",
                    self.t("today_prefix"),
                    t.sale_count,
                    self.money(t.revenue())
                )];
                if t.declared_paise != 0 || t.owner_lane_paise != 0 {
                    lines.push(format!(
                        "{} {} · {} {}",
                        self.lane_declared_label(),
                        self.money(Money(t.declared_paise)),
                        self.lane_owner_label(),
                        self.money(Money(t.owner_lane_paise))
                    ));
                }
                lines.push(format!(
                    "{} {} · {} {} · {} {}",
                    self.t("cash"),
                    self.money(t.cash()),
                    self.t("upi"),
                    self.money(t.upi()),
                    self.t("card"),
                    self.money(t.card())
                ));
                if t.credit_paise != 0 || outstanding != 0 {
                    lines.push(format!(
                        "{} {} {} · {} {}",
                        self.t("credit"),
                        self.t("today_mid"),
                        self.money(t.credit()),
                        self.t("owed"),
                        self.money(Money(outstanding))
                    ));
                }
                if t.rounding_paise != 0 {
                    lines.push(format!(
                        "{} {}",
                        self.t("round_word"),
                        self.money(t.rounding())
                    ));
                }
                lines.join("\n")
            }
            None => format!("{}: —", self.t("today_prefix")),
        }
    }
}

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Multi-script UI fonts: Makepad falls through FontFamily members when a
    // glyph is missing. Cover Indian scheduled scripts + famous world scripts.
    // `icons` (Font Awesome) supplies ✕ ⌫ ✓ etc. that Noto Sans lacks.
    // CJK (zh/ja/ko) comes from Makepad's LXGW WenKai.
    let WorldFonts = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/NotoSans-Regular.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        telugu := FontMember{res: crate_resource("self:resources/fonts/NotoSansTelugu-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        kannada := FontMember{res: crate_resource("self:resources/fonts/NotoSansKannada-Regular.ttf") asc: 0.0 desc: 0.0}
        malayalam := FontMember{res: crate_resource("self:resources/fonts/NotoSansMalayalam-Regular.ttf") asc: 0.0 desc: 0.0}
        gurmukhi := FontMember{res: crate_resource("self:resources/fonts/NotoSansGurmukhi-Regular.ttf") asc: 0.0 desc: 0.0}
        odia := FontMember{res: crate_resource("self:resources/fonts/NotoSansOriya-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
        sinhala := FontMember{res: crate_resource("self:resources/fonts/NotoSansSinhala-Regular.ttf") asc: 0.0 desc: 0.0}
        myanmar := FontMember{res: crate_resource("self:resources/fonts/NotoSansMyanmar-Regular.ttf") asc: 0.0 desc: 0.0}
        khmer := FontMember{res: crate_resource("self:resources/fonts/NotoSansKhmer-Regular.ttf") asc: 0.0 desc: 0.0}
        lao := FontMember{res: crate_resource("self:resources/fonts/NotoSansLao-Regular.ttf") asc: 0.0 desc: 0.0}
        tibetan := FontMember{res: crate_resource("self:resources/fonts/NotoSansTibetan-Regular.ttf") asc: 0.0 desc: 0.0}
        ethiopic := FontMember{res: crate_resource("self:resources/fonts/NotoSansEthiopic-Regular.ttf") asc: 0.0 desc: 0.0}
        thaana := FontMember{res: crate_resource("self:resources/fonts/NotoSansThaana-Regular.ttf") asc: 0.0 desc: 0.0}
        georgian := FontMember{res: crate_resource("self:resources/fonts/NotoSansGeorgian-Regular.ttf") asc: 0.0 desc: 0.0}
        armenian := FontMember{res: crate_resource("self:resources/fonts/NotoSansArmenian-Regular.ttf") asc: 0.0 desc: 0.0}
        meetei := FontMember{res: crate_resource("self:resources/fonts/NotoSansMeeteiMayek-Regular.ttf") asc: 0.0 desc: 0.0}
        olchiki := FontMember{res: crate_resource("self:resources/fonts/NotoSansOlChiki-Regular.ttf") asc: 0.0 desc: 0.0}
        chinese := FontMember{res: crate_resource("makepad_widgets:resources/LXGWWenKaiRegular.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("self:resources/fonts/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("makepad_widgets:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
    }
    let WorldFontsBold = FontFamily{
        latin := FontMember{res: crate_resource("self:resources/fonts/NotoSans-Regular.ttf") asc: -0.1 desc: 0.0}
        devanagari := FontMember{res: crate_resource("self:resources/fonts/NotoSansDevanagari-Regular.ttf") asc: 0.0 desc: 0.0}
        bengali := FontMember{res: crate_resource("self:resources/fonts/NotoSansBengali-Regular.ttf") asc: 0.0 desc: 0.0}
        tamil := FontMember{res: crate_resource("self:resources/fonts/NotoSansTamil-Regular.ttf") asc: 0.0 desc: 0.0}
        telugu := FontMember{res: crate_resource("self:resources/fonts/NotoSansTelugu-Regular.ttf") asc: 0.0 desc: 0.0}
        gujarati := FontMember{res: crate_resource("self:resources/fonts/NotoSansGujarati-Regular.ttf") asc: 0.0 desc: 0.0}
        kannada := FontMember{res: crate_resource("self:resources/fonts/NotoSansKannada-Regular.ttf") asc: 0.0 desc: 0.0}
        malayalam := FontMember{res: crate_resource("self:resources/fonts/NotoSansMalayalam-Regular.ttf") asc: 0.0 desc: 0.0}
        gurmukhi := FontMember{res: crate_resource("self:resources/fonts/NotoSansGurmukhi-Regular.ttf") asc: 0.0 desc: 0.0}
        odia := FontMember{res: crate_resource("self:resources/fonts/NotoSansOriya-Regular.ttf") asc: 0.0 desc: 0.0}
        arabic := FontMember{res: crate_resource("self:resources/fonts/NotoSansArabic-Regular.ttf") asc: 0.0 desc: 0.0}
        hebrew := FontMember{res: crate_resource("self:resources/fonts/NotoSansHebrew-Regular.ttf") asc: 0.0 desc: 0.0}
        thai := FontMember{res: crate_resource("self:resources/fonts/NotoSansThai-Regular.ttf") asc: 0.0 desc: 0.0}
        sinhala := FontMember{res: crate_resource("self:resources/fonts/NotoSansSinhala-Regular.ttf") asc: 0.0 desc: 0.0}
        myanmar := FontMember{res: crate_resource("self:resources/fonts/NotoSansMyanmar-Regular.ttf") asc: 0.0 desc: 0.0}
        khmer := FontMember{res: crate_resource("self:resources/fonts/NotoSansKhmer-Regular.ttf") asc: 0.0 desc: 0.0}
        lao := FontMember{res: crate_resource("self:resources/fonts/NotoSansLao-Regular.ttf") asc: 0.0 desc: 0.0}
        tibetan := FontMember{res: crate_resource("self:resources/fonts/NotoSansTibetan-Regular.ttf") asc: 0.0 desc: 0.0}
        ethiopic := FontMember{res: crate_resource("self:resources/fonts/NotoSansEthiopic-Regular.ttf") asc: 0.0 desc: 0.0}
        thaana := FontMember{res: crate_resource("self:resources/fonts/NotoSansThaana-Regular.ttf") asc: 0.0 desc: 0.0}
        georgian := FontMember{res: crate_resource("self:resources/fonts/NotoSansGeorgian-Regular.ttf") asc: 0.0 desc: 0.0}
        armenian := FontMember{res: crate_resource("self:resources/fonts/NotoSansArmenian-Regular.ttf") asc: 0.0 desc: 0.0}
        meetei := FontMember{res: crate_resource("self:resources/fonts/NotoSansMeeteiMayek-Regular.ttf") asc: 0.0 desc: 0.0}
        olchiki := FontMember{res: crate_resource("self:resources/fonts/NotoSansOlChiki-Regular.ttf") asc: 0.0 desc: 0.0}
        chinese := FontMember{res: crate_resource("makepad_widgets:resources/LXGWWenKaiBold.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("self:resources/fonts/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("makepad_widgets:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
    }
    mod.theme.font_regular.font_family = WorldFonts
    mod.theme.font_label.font_family = WorldFonts
    mod.theme.font_bold.font_family = WorldFontsBold
    mod.theme.font_italic.font_family = WorldFonts
    mod.theme.font_bold_italic.font_family = WorldFontsBold
    // Widget classes copy the theme face when they are defined, which is
    // before this script runs. Point the classes themselves at WorldFonts
    // so every label, button, and field actually falls through to the
    // script fonts. A theme-only assignment leaves IBM Plex in place and
    // Gujarati, Devanagari, and the other scripts draw as garbage.
    mod.widgets.Label.draw_text.text_style.font_family = WorldFonts
    mod.widgets.Button.draw_text.text_style.font_family = WorldFonts
    mod.widgets.TextInput.draw_text.text_style.font_family = WorldFonts
    mod.widgets.TextInputFlat.draw_text.text_style.font_family = WorldFonts
    // Contrast law: never pair two lights or two darks — theme OR local styles.
    // Light face → dark ink (#1a1c1e / #45464e / #75777f) in EVERY state.
    // Navy face → white / on-primary-container ink in EVERY state.
    // Makepad mixes color→color_hover→color_down; mismatched lightness = wash.
    // Never peach/cream fills (#ffdbcf/#ffb59c) or light glyphs on light downs.
    mod.theme.color_bg_app = #xd5d7dc
    mod.theme.color_app_caption_bar = #x1b2a4a
    mod.theme.color_text = #x1a1c1e
    mod.theme.color_text_hover = #x1a1c1e
    mod.theme.color_text_focus = #x1a1c1e
    mod.theme.color_text_down = #x1a1c1e
    mod.theme.color_text_hl = #x1a1c1e
    mod.theme.color_text_disabled = #x75777f
    mod.theme.color_text_val = #x45464e
    mod.theme.color_text_meta = #x75777f
    mod.theme.color_label_inner = #x1a1c1e
    mod.theme.color_label_inner_hover = #x1a1c1e
    mod.theme.color_label_inner_down = #x1a1c1e
    mod.theme.color_label_inner_focus = #x1a1c1e
    mod.theme.color_label_inner_active = #x1a1c1e
    mod.theme.color_text_placeholder = #x75777f
    mod.theme.color_text_placeholder_hover = #x45464e
    // Caret must be dark on cool/white wells (stock theme uses white = invisible).
    mod.theme.color_text_cursor = #x041534
    mod.theme.color_cursor = #x041534
    mod.theme.color_cursor_focus = #x041534
    mod.theme.color_cursor_empty = #x041534
    mod.theme.color_cursor_border = #x041534

    // Stock TextInput caret + placeholder colors are baked from theme at
    // widget-def time (often white on gray wells). Re-bind after our theme so
    // EVERY TextInput shows a visible caret and readable placeholders.
    // Do NOT use set_type_default() here (Android Animator UAF).
    mod.widgets.TextInputFlat = mod.widgets.TextInputFlat{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x041534
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_focus: #x1a1c1e
            color_down: #x1a1c1e
            color_empty: #x75777f
            color_empty_hover: #x45464e
            color_empty_focus: #x45464e
        }
    }
    mod.widgets.TextInput = mod.widgets.TextInput{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x041534
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_focus: #x1a1c1e
            color_down: #x1a1c1e
            color_empty: #x75777f
            color_empty_hover: #x45464e
            color_empty_focus: #x45464e
        }
    }
    mod.theme.color_icon = #x1a1c1e
    mod.theme.color_icon_active = #x1a1c1e
    mod.theme.color_icon_inactive = #x75777f
    mod.theme.color_icon_disabled = #xc5c6cf
    // Cool inset wells — mid gray, never near-white cream on white cards.
    mod.theme.color_inset = #xd5d7dc
    mod.theme.color_inset_hover = #xc9ccd3
    mod.theme.color_inset_down = #xbec1c8
    mod.theme.color_inset_focus = #xffffff
    mod.theme.color_inset_empty = #xd5d7dc
    // Solid light faces — color_2 must match color (never transparent wash).
    mod.theme.color_outset = #xffffff
    mod.theme.color_outset_hover = #xf3f3f6
    mod.theme.color_outset_down = #xe8e8ea
    mod.theme.color_outset_focus = #xffffff
    mod.theme.color_outset_1 = #xffffff
    mod.theme.color_outset_1_hover = #xf3f3f6
    mod.theme.color_outset_1_down = #xe8e8ea
    mod.theme.color_outset_1_focus = #xffffff
    // Same stop as color_outset — transparent color_2 washes light-on-light.
    mod.theme.color_outset_2 = #xffffff
    mod.theme.color_outset_2_hover = #xf3f3f6
    mod.theme.color_outset_2_down = #xe8e8ea
    mod.theme.color_outset_2_focus = #xffffff
    // Selection band: opaque mid-blue on light page (plan: visible, not pale wash).
    // Dark ink (#x1a1c1e) stays readable on this band.
    mod.theme.color_selection = #x3d6fd9FF
    mod.theme.color_selection_hover = #x3d6fd9FF
    mod.theme.color_selection_down = #x2f5bb8FF
    mod.theme.color_selection_focus = #x3d6fd9FF
    mod.theme.color_selection_empty = #x3d6fd9FF
    mod.theme.color_selection_disabled = #xe8e8ea

    // Default Label ink. Keep stock flow — forcing Down drops glyphs on Android GLES.
    mod.widgets.Label = set_type_default() do mod.widgets.Label{
        height: Fit
        draw_text +: {
            color: #x1a1c1e
        }
    }

    // Default Button: light face + dark ink (identical in every pointer state).
    // Set color_2* = color* so the bevel gradient never washes to a second light.
    // Label inset: Makepad 2 Fill + Ellipsis can pin ink to the face edge when only
    // layout padding is set — keep padding *and* label_walk.margin on every face.
    mod.widgets.Button = set_type_default() do mod.widgets.Button{
        align: Center
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xf3f3f6
            color_down: #xe8e8ea
            color_focus: #xffffff
            color_disabled: #xe8e8ea
            color_2: #xffffff
            color_2_hover: #xf3f3f6
            color_2_down: #xe8e8ea
            color_2_focus: #xffffff
            color_2_disabled: #xe8e8ea
            border_color: #xc5c6cf
            border_color_hover: #xc5c6cf
            border_color_down: #xc5c6cf
            border_color_focus: #xc5c6cf
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            color_disabled: #x75777f
        }
    }

    // NEVER call set_type_default on TextInput — it rebuilds Animator apply
    // objects and Android then panics with GenVec use-after-free on NextFrame,
    // freezing the whole UI (PIN keypad included). Style fields via theme +
    // per-instance blink_speed/padding only (see FieldInput).

    // Makepad 2 ScrollYView: wheel/trackpad scroll only. Drag-scroll steals
    // text selection gestures; selectable WrapLabels need the drag.
    mod.widgets.ScrollYView = mod.widgets.ViewBase {
        scroll_bars: mod.widgets.ScrollBars {
            show_scroll_x: false show_scroll_y: true
            scroll_bar_y.drag_scrolling: false
        }
    }

    // Horizontal chip/tab strip. Wheel/scrollbar scroll only — drag-scroll
    // co-captures and steals Button taps after a 2px move (Makepad 2).
    mod.widgets.ScrollXView = mod.widgets.ViewBase {
        scroll_bars: mod.widgets.ScrollBars {
            show_scroll_x: true show_scroll_y: false
            scroll_bar_x.drag_scrolling: false
            scroll_bar_x.bar_size: 4.
        }
    }

    // Custom window chrome (Wayland / Windows): stock Makepad uses black
    // min/max/close glyphs on a light caption — invisible on our navy UI.
    mod.widgets.Window = set_type_default() do mod.widgets.Window{
        pass.clear_color: #xd5d7dc
        caption_bar +: {
            draw_bg.color: #x1b2a4a
            caption_label +: {
                label +: {
                    draw_text.color: #xffffff
                }
            }
            windows_buttons +: {
                min +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xd9dadc
                        bg_color: #00000000
                        bg_color_hover: #x24365e
                        bg_color_down: #x0f172a
                    }
                }
                max +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xd9dadc
                        bg_color: #00000000
                        bg_color_hover: #x24365e
                        bg_color_down: #x0f172a
                    }
                }
                close +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xffffff
                        bg_color: #00000000
                        bg_color_hover: #xE81123
                        // Dark red — never light pink (#F1707A) under white glyph
                        bg_color_down: #x93000a
                    }
                }
            }
            web_fullscreen +: {
                fullscreen +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xd9dadc
                    }
                }
            }
        }
    }

    let NavBtn = Button{
        width: Fill height: 52
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x1b2a4a
            color_hover: #x24365e
            color_down: #x0f172a
            color_focus: #x1b2a4a
            color_disabled: #x64748b
            color_2: #x1b2a4a
            color_2_hover: #x24365e
            color_2_down: #x0f172a
            color_2_focus: #x1b2a4a
            color_2_disabled: #x64748b
            border_color: #x1b2a4a
            border_color_hover: #x24365e
            border_color_down: #x0f172a
            border_color_focus: #x1b2a4a
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xe8e8ea
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 15.}
        }
    }

    // Compact back control for More child screens (+ header).
    let BackBtn = Button{
        width: 128 height: 40
        align: Center
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x1b2a4a
            color_hover: #x24365e
            color_down: #x0f172a
            color_focus: #x1b2a4a
            color_2: #x1b2a4a
            color_2_hover: #x24365e
            color_2_down: #x0f172a
            color_2_focus: #x1b2a4a
            border_color: #x1b2a4a
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            text_style +: {font_size: 14.}
        }
    }

    let TabBtn = Button{
        width: Fill height: 56
        align: Center
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 8. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x1b2a4a
            color_down: #x0f172a
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x1b2a4a
            color_2_down: #x0f172a
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        // Always light ink — sits on navy bottom_nav only (never on a light face).
        draw_text +: {
            color: #xb7c6ee
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #x75777f
            text_style +: {font_size: 13.}
        }
    }

    // Secondary pay — navy + white (never white face on white cart).
    let PayBtn = Button{
        width: Fill height: 52
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x1b2a4a
            color_hover: #x24365e
            color_down: #x0f172a
            color_focus: #x1b2a4a
            color_disabled: #x64748b
            color_2: #x1b2a4a
            color_2_hover: #x24365e
            color_2_down: #x0f172a
            color_2_focus: #x1b2a4a
            color_2_disabled: #x64748b
            border_color: #x1b2a4a
            border_color_hover: #x24365e
            border_color_down: #x0f172a
            border_color_focus: #x1b2a4a
            border_radius: 16.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xe8e8ea
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 14.}
        }
    }

    let PinKey = Button{
        width: Fill height: 56
        align: Center
        padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 8. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xf3f3f6
            color_down: #xe8e8ea
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xf3f3f6
            color_2_down: #xe8e8ea
            color_2_focus: #xffffff
            border_color: #xc5c6cf
            border_color_hover: #x75777f
            border_color_down: #x75777f
            border_color_focus: #xc5c6cf
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            text_style +: {font_size: 22.}
        }
    }

    // PIN pad OK — same narrow cell as digit keys; tight insets so "OK" never becomes "...".
    let PinOkKey = Button{
        width: Fill height: 56
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 8. right: 8.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x1b2a4a
            color_hover: #x24365e
            color_down: #x041534
            color_focus: #x1b2a4a
            color_2: #x1b2a4a
            color_2_hover: #x24365e
            color_2_down: #x041534
            color_2_focus: #x1b2a4a
            border_color: #x1b2a4a
            border_radius: 8.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            max_lines: 1
            text_style +: {font_size: 18.}
        }
    }

    // Side-by-side row actions — same height/margins so mixed primary/secondary
    // never sit on different baselines (Makepad Fill+theme margin misaligns).
    // Defined as full Button (not PinOkKey alias) so padding always applies.
    // Prefer ActionStack when labels are longer than ~10 chars at 320px width.
    let RowPrimaryBtn = Button{
        width: Fill height: 52
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x1b2a4a
            color_hover: #x24365e
            color_down: #x041534
            color_focus: #x1b2a4a
            color_2: #x1b2a4a
            color_2_hover: #x24365e
            color_2_down: #x041534
            color_2_focus: #x1b2a4a
            border_color: #x1b2a4a
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 15.}
        }
    }
    let RowSecondaryBtn = Button{
        width: Fill height: 52
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xe2e2e5
            color_hover: #xd9dadc
            color_down: #xc5c6cf
            color_focus: #xe2e2e5
            color_2: #xe2e2e5
            color_2_hover: #xd9dadc
            color_2_down: #xc5c6cf
            color_2_focus: #xe2e2e5
            border_color: #xc5c6cf
            border_radius: 14.
            border_size: 1.
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 15.}
        }
    }
    // Prefer ActionStack for CTAs. ActionRow only for 2 ultra-short labels
    // (≤4 chars) — Fill NavBtn/PayBtn in a Right row becomes "..." at ~320px.
    let ActionRow = View{
        width: Fill height: Fit
        flow: Right spacing: 8
        align: Align{y: 0.5}
    }
    // Full-width stacked actions — required when labels are longer than ~4 chars.
    let ActionStack = View{
        width: Fill height: Fit
        flow: Down spacing: 8
    }

    // Chip for horizontal strips: Fit width (scroll, don't crush). Cap width so
    // long labels ellipsis instead of stealing the whole strip.
    // Fit.max is Option<FitBound>; bare numbers / bare Abs() are not in scope —
    // use FitBound.Abs (same pattern as Flow.Right).
    let ChipBtn = Button{
        width: Fit{max: FitBound.Abs(160.)} height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xe2e2e5
            color_hover: #xd9dadc
            color_down: #xc5c6cf
            color_focus: #xe2e2e5
            color_2: #xe2e2e5
            color_2_hover: #xd9dadc
            color_2_down: #xc5c6cf
            color_2_focus: #xe2e2e5
            border_color: #xc5c6cf
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 13.}
        }
    }
    let ChipBtnActive = Button{
        width: Fit{max: FitBound.Abs(160.)} height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x041534
            color_hover: #x1b2a4a
            color_down: #x041534
            color_focus: #x041534
            color_2: #x041534
            color_2_hover: #x1b2a4a
            color_2_down: #x041534
            color_2_focus: #x041534
            border_color: #x041534
            border_radius: 10.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 13.}
        }
    }
    // Inner row for ChipStrip — Fit-wide so the strip can scroll horizontally.
    let ChipStripRow = View{
        width: Fit height: Fill
        flow: Right spacing: 8
        align: Align{y: 0.5}
        padding: Inset{left: 0. right: 12. top: 4. bottom: 4.}
    }
    // Horizontally scrollable chip/tab strip. Children: ChipStripRow { ChipBtn… }.
    let ChipStrip = ScrollXView{
        width: Fill height: 52
    }

    let RoleChip = Button{
        width: Fill height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xe2e2e5
            color_hover: #xd9dadc
            color_down: #xc5c6cf
            color_focus: #xe2e2e5
            color_2: #xe2e2e5
            color_2_hover: #xd9dadc
            color_2_down: #xc5c6cf
            color_2_focus: #xe2e2e5
            border_color: #xc5c6cf
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 13.}
        }
    }

    let RoleChipActive = Button{
        width: Fill height: 40
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 14. right: 14. top: 8. bottom: 8.}
        align: Center
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 14. right: 14.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x041534
            color_hover: #x1b2a4a
            color_down: #x041534
            color_focus: #x041534
            color_2: #x041534
            color_2_hover: #x1b2a4a
            color_2_down: #x041534
            color_2_focus: #x041534
            border_color: #x041534
            border_radius: 10.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 13.}
        }
    }


    // Styled field — padding/blink/caret/placeholder for ALL app inputs. Do not
    // touch draw_bg color tracks (Animator owns those; mutating them on Android
    // risks use-after-free). Every FieldInput inherits a visible caret and dark
    // placeholder on the gray well.
    let FieldInput = TextInput{
        width: Fill
        height: Fit
        padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
        blink_speed: 0.5
        draw_cursor +: {
            color: #x041534
        }
        // Flat opaque selection — color_2 must match (no light gradient wash).
        draw_selection +: {
            gradient_fill_horizontal: 0.0
            color: #x3d6fd9FF
            color_hover: #x3d6fd9FF
            color_focus: #x3d6fd9FF
            color_down: #x2f5bb8FF
            color_empty: #x3d6fd9FF
            color_2: #x3d6fd9FF
            color_2_hover: #x3d6fd9FF
            color_2_focus: #x3d6fd9FF
            color_2_down: #x2f5bb8FF
            color_2_empty: #x3d6fd9FF
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_focus: #x1a1c1e
            color_down: #x1a1c1e
            color_empty: #x75777f
            color_empty_hover: #x45464e
            color_empty_focus: #x45464e
            text_style +: {font_size: 16.}
        }
    }

    // Compact cart / totals field — same caret as FieldInput; tighter padding.
    let CartField = FieldInput{
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xf3f3f6
            color_focus: #xffffff
            color_down: #xe8e8ea
            color_empty: #xffffff
            border_color: #x5c6570
            border_color_hover: #x041534
            border_color_focus: #x041534
            border_radius: 10.
            border_size: 2.
        }
        draw_text +: {
            color: #x1a1c1e
            color_empty: #x75777f
            color_empty_hover: #x45464e
            color_empty_focus: #x45464e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            text_style +: {font_size: 15.}
        }
    }

    // Full-width selectable copy (Markdown TextFlow) with real `visible`
    // support — stock Markdown/TextFlow ignore set_visible. Use `body:`.
    // Leave Button / product-card Labels as Label so taps stay clean.
    mod.widgets.WrapLabelBase = #(WrapLabel::register_widget(vm))
    mod.widgets.WrapLabel = set_type_default() do mod.widgets.WrapLabelBase{
        width: Fill
        height: Fit
        padding: 0.
        selectable: true
        paragraph_spacing: 0.
        pre_code_spacing: 0.
        font_size: 14.
        font_color: #x1a1c1e
        draw_text +: {
            color: #x1a1c1e
        }
        draw_block +: {
            // Opaque mid-blue (RRGGBBAA) — readable dark ink on top, clear on white page.
            selection_color: #x3d6fd9FF
        }
    }
    let WrapLabel = mod.widgets.WrapLabel{}

    // Sell token: Fit height — list row estimate must stay >= natural size.
    let ProductCard = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        flow: Down spacing: 8
        align: Align{x: 0.5 y: 0.0}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        thumb := Image{
            width: 64 height: 64
            margin: Inset{top: 4. bottom: 4. left: 4. right: 4.}
            fit: ImageFit.CropToFill
            min_width: 64
            min_height: 64
        }
        name := Label{
            width: Fill
            text: "Item"
            align: Align{x: 0.5}
            flow: Right{wrap: false}
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x1a1c1e
            draw_text.text_style.font_size: 13.
        }
        price := Label{
            width: Fill
            text: "0"
            align: Align{x: 0.5}
            flow: Right{wrap: false}
            max_lines: 1
            draw_text.color: #xa23e18
            draw_text.text_style.font_size: 15.
        }
        // Terracotta + white — never light-on-light; big enough to tap.
        add := Button{
            text: "ADD"
            width: Fill height: 48
            align: Center
            padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
            label_walk: Walk{
                width: Fit
                height: Fit
                margin: Inset{left: 16. right: 16.}
            }
            icon_walk: Walk{width: 0. height: 0.}
            draw_bg +: {
                color: #xa23e18
                color_hover: #x822801
                color_down: #x6f2000
                color_focus: #xa23e18
                color_disabled: #xc5c6cf
                color_2: #xa23e18
                color_2_hover: #x822801
                color_2_down: #x6f2000
                color_2_focus: #xa23e18
                color_2_disabled: #xc5c6cf
                border_color: #xa23e18
                border_color_hover: #x822801
                border_color_down: #x6f2000
                border_color_focus: #xa23e18
                border_radius: 12.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
                color_disabled: #x75777f
                text_style +: {font_size: 15.}
            }
        }
    }

    // Items tab: manage catalogue — remove instead of add-to-cart
    let MenuCard = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
        flow: Down spacing: 4
        draw_bg.color: #ffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        name := Label{
            width: Fill
            height: Fit
            padding: 0.
            align: Align{x: 0.5}
            text: "Item"
            flow: Right{wrap: false}
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x1a1c1e
            draw_text.text_style.font_size: 18.
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 10
            align: Align{y: 0.0}
            View{
                width: Fill height: Fit
                flow: Right spacing: 10
                align: Align{y: 0.5}
                thumb := Image{
                width: 56 height: 56
                margin: Inset{top: 10. bottom: 2. left: 2. right: 4.}
                fit: ImageFit.CropToFill
                min_width: 56
                min_height: 56
            }
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                price := Label{
                    width: Fill
                    text: "0"
                    flow: Right{wrap: false}
                    max_lines: 1
                    draw_text.color: #xa23e18
                    draw_text.text_style.font_size: 14.
                }
                stock := Label{
                    width: Fill
                    text: " "
                    flow: Right{wrap: false}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #x45464e
                    draw_text.text_style.font_size: 12.
                }
            }
            }
            View{
                width: Fit height: Fit
                flow: Down spacing: 8
                align: Align{x: 0.5}
                remove := Button{
                    text: "✕"
                    width: 48 height: 44
                    align: Align{x: 0.5 y: 0.5}
                    draw_bg +: {
                        color: #x1b2a4a
                        color_hover: #x24365e
                        color_down: #x041534
                        color_focus: #x1b2a4a
                        color_2: #x1b2a4a
                        color_2_hover: #x24365e
                        color_2_down: #x041534
                        color_2_focus: #x1b2a4a
                        border_color: #x1b2a4a
                        border_radius: 16.
                        border_size: 0.
                    }
                    draw_text +: {
                        color: #xffffff
                        color_hover: #ffffff
                        color_down: #xffffff
                        color_focus: #xffffff
                        text_style +: {font_size: 20.}
                    }
                }
                sold_out := Button{
                    text: "Sold Out"
                    width: Fit height: 44
                    align: Align{x: 0.5 y: 0.5}
                    padding: Inset{left: 16. right: 16. top: 8. bottom: 8.}
            draw_bg +: {
                color: #xa23e18
                color_hover: #xb84a20
                color_down: #x7a2e12
                color_focus: #xa23e18
                color_2: #xa23e18
                color_2_hover: #xb84a20
                color_2_down: #x7a2e12
                color_2_focus: #xa23e18
                border_color: #xa23e18
                border_radius: 16.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
                        text_style +: {font_size: 13.}
                    }
                }
            }
        }
    }

    let CartRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
        flow: Down spacing: 8
        // Mid gray well on white ledger — never white-in-white.
        draw_bg.color: #xe8e8ea
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        label := WrapLabel{
            body: "Item"
            font_color: #x041534
            font_size: 16.
        }
        // Stitch qty dial: Volume − [N] +  (screen 03 / layout spec qty_stepper)
        RoundedView{
            width: Fill height: Fit
            padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
            flow: Right spacing: 8
            align: Align{y: 0.5}
            draw_bg.color: #xd5d7dc
            draw_bg.border_radius: 12.
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                Label{
                    text: "QTY"
                    draw_text.color: #x45464e
                    draw_text.text_style.font_size: 10.
                }
            }
            minus := Button{
                text: "−"
                width: 44 height: 44
                draw_bg +: {
                    color: #xffffff
                    color_hover: #xf3f3f6
                    color_down: #xe2e2e5
                    color_focus: #xffffff
                    color_2: #xffffff
                    color_2_hover: #xf3f3f6
                    color_2_down: #xe2e2e5
                    color_2_focus: #xffffff
                    border_color: #x5c6570
                    border_color_hover: #x45464e
                    border_color_down: #x45464e
                    border_color_focus: #x5c6570
                    border_radius: 12.
                    border_size: 2.
                }
                draw_text +: {
                    color: #x041534
                    color_hover: #x041534
                    color_down: #x041534
                    color_focus: #x041534
                    text_style +: {font_size: 22.}
                }
            }
            qty_label := Label{
                width: 40
                text: "1"
                draw_text.color: #x041534
                draw_text.text_style.font_size: 26.
            }
            plus := Button{
                text: "+"
                width: 44 height: 44
                draw_bg +: {
                    color: #x041534
                    color_hover: #x1b2a4a
                    color_down: #x0a1a3a
                    color_focus: #x041534
                    color_2: #x041534
                    color_2_hover: #x1b2a4a
                    color_2_down: #x0a1a3a
                    color_2_focus: #x041534
                    border_color: #x041534
                    border_color_hover: #x1b2a4a
                    border_color_down: #x0a1a3a
                    border_color_focus: #x041534
                    border_radius: 12.
                    border_size: 0.
                }
                draw_text +: {
                    color: #xffffff
                    color_hover: #xffffff
                    color_down: #xffffff
                    color_focus: #xffffff
                    text_style +: {font_size: 22.}
                }
            }
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 8
            align: Align{y: 0.5}
            Label{
                text: "Unit @"
                draw_text.color: #x1a1c1e
                draw_text.text_style.font_size: 13.
            }
            unit_price := CartField{
                empty_text: "0"
                text: "0"
            }
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 8
            line_actual_label := WrapLabel{
                body: "Actual 0"
                font_color: #x1a1c1e
                font_size: 13.
            }
            View{
                width: Fill height: Fit
                flow: Down spacing: 4
                Label{
                    text: "Applied"
                    draw_text.color: #xa23e18
                    draw_text.text_style.font_size: 11.
                }
                line_total := CartField{
                    empty_text: "0"
                    text: "0"
                }
            }
        }
        mod_row := View{
            width: Fill height: Fit
            flow: Right spacing: 6
            align: Align{y: 0.5}
            mod_prev := RoleChip{text: "‹" width: 36}
            mod_0 := RoleChip{text: ""}
            mod_1 := RoleChip{text: ""}
            mod_2 := RoleChip{text: ""}
            mod_next := RoleChip{text: "›" width: 36}
        }
        note := CartField{
            height: 48
            padding: Inset{left: 10. right: 10. top: 10. bottom: 10.}
            empty_text: "Note (optional)"
            text: ""
            draw_text +: {
                text_style +: {font_size: 14.}
            }
        }
    }

    // Action on a white/light card — ALWAYS navy face + white ink (never default white Button).
    let CardActionBtn = Button{
        width: Fill height: 48
        draw_bg +: {
            color: #x041534
            color_hover: #x1b2a4a
            color_down: #x0a1a3a
            color_focus: #x041534
            color_disabled: #x64748b
            color_2: #x041534
            color_2_hover: #x1b2a4a
            color_2_down: #x0a1a3a
            color_2_focus: #x041534
            color_2_disabled: #x64748b
            border_color: #x041534
            border_color_hover: #x1b2a4a
            border_color_down: #x0a1a3a
            border_color_focus: #x041534
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xe8e8ea
            text_style +: {font_size: 15.}
        }
    }

    let SaleRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 10. bottom: 10.}
        flow: Down spacing: 8
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        label := WrapLabel{
            body: "sale"
            font_color: #x1a1c1e
            font_size: 13.
        }
        pick := CardActionBtn{text: "Select" height: 40}
    }

    let CustomerRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 8. top: 8. bottom: 8.}
        flow: Right spacing: 8
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        label := WrapLabel{
            body: "customer"
            font_color: #x1a1c1e
            font_size: 14.
        }
        pick := Button{
            text: "Pick"
            width: 72 height: 44
            draw_bg +: {
                color: #x041534
                color_hover: #x1b2a4a
                color_down: #x0a1a3a
                color_focus: #x041534
                color_2: #x041534
                color_2_hover: #x1b2a4a
                color_2_down: #x0a1a3a
                color_2_focus: #x041534
                border_color: #x041534
                border_radius: 12.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
            }
        }
    }

    // Whole-row language pick (no separate Use button). Same inset law as
    // every other face: padding *and* label_walk.margin. Long labels wrap
    // onto the next line inside that inset (no ellipsis / crop).
    let LanguageRow = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        enable_long_press: true
        draw_bg +: {
            color: #xffffff
            color_hover: #xf3f3f6
            color_down: #xe8e8ea
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xf3f3f6
            color_2_down: #xe8e8ea
            color_2_focus: #xffffff
            border_color: #x5c6570
            border_color_hover: #x45464e
            border_color_down: #x45464e
            border_color_focus: #x5c6570
            border_radius: 14.
            border_size: 2.
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            ink_centered: false
            text_style +: {font_size: 14.}
        }
    }

    let LocaleRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 16. right: 8. top: 8. bottom: 8.}
        flow: Right spacing: 8
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        label := WrapLabel{
            body: "locale"
            font_color: #x1a1c1e
            font_size: 14.
        }
        pick := Button{
            text: "Use"
            width: 72 height: 44
            draw_bg +: {
                color: #x041534
                color_hover: #x1b2a4a
                color_down: #x0a1a3a
                color_focus: #x041534
                color_2: #x041534
                color_2_hover: #x1b2a4a
                color_2_down: #x0a1a3a
                color_2_focus: #x041534
                border_color: #x041534
                border_radius: 12.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
            }
        }
    }

    let TableCard = RoundedView{
        width: Fill height: Fit
        padding: 14
        flow: Down spacing: 8
        draw_bg.color: #xffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        name := WrapLabel{body: "T1" font_color: #x1a1c1e font_size: 20.}
        status := WrapLabel{body: "available" font_color: #x45464e}
        action := CardActionBtn{text: "Use"}
    }

    let KotRow = RoundedView{
        width: Fill height: Fit
        padding: 14
        flow: Down spacing: 8
        draw_bg.color: #xffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5c6570
        label := WrapLabel{body: "kot" font_color: #x1a1c1e font_size: 15.}
        advance := CardActionBtn{text: "Advance"}
    }

    let MoreRow = Button{
        width: Fill height: 58
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 16. right: 16. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        // So a touch-drag that scrolls the page is not also treated as a click.
        enable_long_press: true
        draw_bg +: {
            color: #xffffff
            color_hover: #xf3f3f6
            color_down: #xe8e8ea
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #xf3f3f6
            color_2_down: #xe8e8ea
            color_2_focus: #xffffff
            border_color: #x5c6570
            border_color_hover: #x45464e
            border_color_down: #x45464e
            border_color_focus: #x5c6570
            border_radius: 16.
            border_size: 2.
        }
        draw_text +: {
            color: #x1a1c1e
            color_hover: #x1a1c1e
            color_down: #x1a1c1e
            color_focus: #x1a1c1e
            max_lines: 1
            text_overflow: TextOverflow.Ellipsis
            text_style +: {font_size: 16.}
        }
    }

    // Lists size to content so the parent page ScrollYView scrolls form + rows together.
    // These PortalLists must NOT scroll themselves — otherwise wheel/drag over the
    // list eats the gesture and the page bottom (next cart lines, pay) is unreachable.
    mod.widgets.ProductListBase = #(ProductList::register_widget(vm))
    mod.widgets.ProductList = set_type_default() do mod.widgets.ProductListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 8
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                flow: Right spacing: 12
                padding: Inset{bottom: 8.}
                texture_caching: false
                c0 := ProductCard{}
                c1 := ProductCard{}
                c2 := ProductCard{}
            }
        }
    }

    mod.widgets.MenuListBase = #(MenuList::register_widget(vm))
    mod.widgets.MenuList = set_type_default() do mod.widgets.MenuListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                card := MenuCard{}
            }
        }
    }

    mod.widgets.CartListBase = #(CartList::register_widget(vm))
    mod.widgets.CartList = set_type_default() do mod.widgets.CartListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 8
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                CartRow{}
            }
        }
    }

    mod.widgets.SaleListBase = #(SaleList::register_widget(vm))
    mod.widgets.SaleList = set_type_default() do mod.widgets.SaleListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 8
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                SaleRow{}
            }
        }
    }

    mod.widgets.CustomerListBase = #(CustomerList::register_widget(vm))
    mod.widgets.CustomerList = set_type_default() do mod.widgets.CustomerListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                CustomerRow{}
            }
        }
    }

    mod.widgets.LanguageListBase = #(LanguageList::register_widget(vm))
    mod.widgets.LanguageList = set_type_default() do mod.widgets.LanguageListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 10.}
                texture_caching: false
                pick := LanguageRow{text: "language"}
            }
        }
    }

    mod.widgets.ServerListBase = #(ServerList::register_widget(vm))
    mod.widgets.ServerList = set_type_default() do mod.widgets.ServerListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 10.}
                texture_caching: false
                LocaleRow{}
            }
        }
    }

    mod.widgets.CurrencyListBase = #(CurrencyList::register_widget(vm))
    mod.widgets.CurrencyList = set_type_default() do mod.widgets.CurrencyListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 10.}
                texture_caching: false
                LocaleRow{}
            }
        }
    }

    mod.widgets.TableListBase = #(TableList::register_widget(vm))
    mod.widgets.TableList = set_type_default() do mod.widgets.TableListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                TableCard{}
            }
        }
    }

    mod.widgets.KotListBase = #(KotList::register_widget(vm))
    mod.widgets.KotList = set_type_default() do mod.widgets.KotListBase{
        width: Fill height: Fit
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            drag_scrolling: false
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{bar_size: 0.}
            Item := View{
                width: Fill height: Fit
                padding: Inset{bottom: 8.}
                texture_caching: false
                KotRow{}
            }
        }
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "Kumkaj"
                // First-run desktop default (not a phone frame). Later launches
                // restore last size/position from window_geom.json in App code.
                // Layout content still never goes below 320 css px (app_shell).
                window.inner_size: vec2(1024, 768)
                pass.clear_color: #xd5d7dc
                caption_bar +: {
                    draw_bg.color: #x1b2a4a
                    // Left cluster in the title bar: size · Kumkaj (not over content).
                    caption_label: View{
                        width: Fill height: Fill
                        flow: Right
                        align: Align{x: 0.0 y: 0.5}
                        padding: Inset{left: 10. right: 10.}
                        spacing: 8
                        dev_size_badge := View{
                            width: Fit height: Fit
                            align: Align{y: 0.5}
                            dev_size_label := Label{
                                width: Fit height: Fit
                                text: "—"
                                draw_text.color: #xfe8357
                                draw_text.text_style.font_size: 11.
                            }
                        }
                        label := Label{
                            text: "Kumkaj"
                            draw_text.color: #xffffff
                        }
                    }
                }
                body +: {
                    width: Fill height: Fill
                    flow: Down spacing: 0
                    // Min layout width 320 css px. Narrower viewports scroll horizontally
                    // instead of crushing faces below that floor.
                    app_hscroll := ScrollXView{
                        width: Fill height: Fill
                        app_shell := View{
                            width: Fill{min: 320.} height: Fill
                            flow: Down spacing: 0

                    header := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
                        flow: Down spacing: 8
                        draw_bg.color: #x041534

                        screen_back := BackBtn{
                            text: "← Back"
                            visible: false
                            width: 128 height: 40
                        }

                        View{
                            width: Fill height: Fit
                            flow: Right spacing: 10
                            align: Align{y: 0.5}
                            brand_mark := Image{
                                width: 36 height: 36
                                fit: ImageFit.Stretch
                                visible: false
                            }
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 2
                                brand_title := WrapLabel{
                                    body: "Your shop"
                                    font_color: #xffffff
                                    font_size: 18.
                                }
                                session_label := WrapLabel{
                                    body: "Sell"
                                    font_color: #x8392b7
                                    font_size: 11.
                                }
                            }
                            shop_badge := Label{
                                text: "Shop"
                                draw_text.color: #xfe8357
                                draw_text.text_style.font_size: 11.
                            }
                        }
                    }

                    today_bar := SolidView{
                        width: Fill height: Fit
                        flow: Down
                        padding: Inset{left: 16. right: 16. top: 8. bottom: 8.}
                        draw_bg.color: #xe8e8ea
                        today_label := WrapLabel{
                            width: Fill
                            body: "Today: —"
                            font_color: #x1a1c1e
                            font_size: 12.
                        }
                    }

                    body_stack := View{
                        width: Fill height: Fill
                        flow: Overlay
                        padding: 0.

                        device_setup_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: true

                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 12. bottom: 120.}

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 12. right: 12. top: 8. bottom: 8.}
                                    flow: Right
                                    align: Align{y: 0.5}
                                    draw_bg.color: #x1b2a4a
                                    draw_bg.border_radius: 10.
                                    View{
                                        width: Fill height: Fit
                                        device_setup_banner := WrapLabel{
                                            body: "THIS DEVICE"
                                            font_color: #xffffff
                                            font_size: 10.
                                        }
                                    }
                                    Label{
                                        text: "LOCAL"
                                        draw_text.color: #xc59b27
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
                                    flow: Down spacing: 14
                                    draw_bg.color: #xd5d7dc
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xe2e2e5
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 12
                                            align: Align{y: 0.5}
                                            setup_brand_mark := Image{
                                                width: 48 height: 48
                                                fit: ImageFit.Stretch
                                                visible: false
                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 4
                                                setup_brand_title := WrapLabel{
                                                    body: "Your shop"
                                                    font_color: #x041534
                                                    font_size: 18.
                                                }
                                                device_use_question := WrapLabel{
                                                    body: "How will this device be used?"
                                                    font_color: #x45464e
                                                    font_size: 12.
                                                }
                                            }
                                        }
                                    }

                                    setup_lang_title := WrapLabel{
                                        body: "Language"
                                        font_color: #x1a1c1e
                                        font_size: 15.
                                    }
                                    setup_lang_hint := WrapLabel{
                                        width: Fill
                                        body: "Follow this device’s language, or pick one yourself."
                                        font_color: #x45464e
                                        font_size: 12.
                                    }
                                    ChipStrip{
                                        ChipStripRow{
                                            View{
                                                width: Fit height: 40
                                                flow: Overlay
                                                lang_setup_platform := ChipBtn{text: "Use platform native language"}
                                                lang_setup_platform_on := ChipBtnActive{text: "Use platform native language" visible: false}
                                            }
                                            View{
                                                width: Fit height: 40
                                                flow: Overlay
                                                lang_setup_user := ChipBtn{text: "Use user selected language"}
                                                lang_setup_user_on := ChipBtnActive{text: "Use user selected language" visible: false}
                                            }
                                        }
                                    }
                                    setup_lang_selected := WrapLabel{
                                        body: ""
                                        font_color: #x1a1c1e
                                        font_size: 13.
                                    }
                                    setup_lang_list := mod.widgets.LanguageList{}

                                    device_section_label := WrapLabel{
                                        body: "THIS DEVICE"
                                        font_color: #x1a1c1e
                                        font_size: 11.
                                    }

                                    ActionStack{
                                        device_role_server := RowPrimaryBtn{text: "Server Mode"}
                                        device_role_client := RowSecondaryBtn{text: "Client Mode"}
                                    }

                                    device_setup_help := WrapLabel{
                                        width: Fill
                                        body: "Server Mode (standalone): this device holds the books. Client Mode: pair to a Server. Asked every launch — pick again or switch. Extra tills stay off until you allow them in Backup & settings."
                                        font_color: #x45464e
                                        font_size: 12.
                                    }

                                    server_pick_section := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        server_pick_title := WrapLabel{
                                            body: "Choose a server"
                                            font_color: #x1a1c1e
                                            font_size: 15.
                                        }
                                        server_pick_hint := WrapLabel{
                                            width: Fill
                                            body: "This device already has servers. Open one, or create a new server for another business."
                                            font_color: #x45464e
                                            font_size: 12.
                                        }
                                        server_name_input := FieldInput{empty_text: "New server name (optional)"}
                                        server_create_new := NavBtn{text: "Create new server"}
                                        setup_server_list := mod.widgets.ServerList{}
                                        server_pick_cancel := BackBtn{text: "Back"}
                                    }

                                    device_setup_status := WrapLabel{
                                        body: " "
                                        font_color: #xa23e18
                                        font_size: 12.
                                    }
                                }
                            }
                        }

                        lock_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false

                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 12. bottom: 120.}

                                // Offline strip (Stitch screen 01)
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 12. right: 12. top: 8. bottom: 8.}
                                    flow: Right
                                    align: Align{y: 0.5}
                                    draw_bg.color: #x1b2a4a
                                    draw_bg.border_radius: 10.
                                    View{
                                        width: Fill height: Fit
                                        WrapLabel{
                                            body: "MAKEPAD 2 · OFFLINE"
                                            font_color: #xffffff
                                            font_size: 10.
                                        }
                                    }
                                    Label{
                                        text: "LOCAL"
                                        draw_text.color: #xc59b27
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                // Brand + lock card
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
                                    flow: Down spacing: 14
                                    draw_bg.color: #xd5d7dc
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xe2e2e5
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 12
                                            align: Align{y: 0.5}
                                            lock_brand_mark := Image{
                                                width: 48 height: 48
                                                fit: ImageFit.Stretch
                                                visible: false
                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 4
                                                lock_brand_title := WrapLabel{
                                                    body: "Your shop"
                                                    font_color: #x041534
                                                    font_size: 18.
                                                }
                                                WrapLabel{
                                                    body: "Counter 01 · Locked"
                                                    font_color: #x45464e
                                                    font_size: 12.
                                                }
                                            }
                                            Label{
                                                text: "Shop"
                                                draw_text.color: #xa23e18
                                                draw_text.text_style.font_size: 10.
                                            }
                                        }
                                    }

                                    ChipStrip{
                                        ChipStripRow{
                                            role_owner := ChipBtnActive{
                                                text: "Owner"
                                                padding: Inset{left: 18. right: 18.}
                                            }
                                            role_staff := ChipBtn{
                                                text: "Staff"
                                                visible: false
                                                padding: Inset{left: 18. right: 18.}
                                            }
                                        }
                                    }

                                    lock_staff_row := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 8
                                        visible: false
                                        WrapLabel{
                                            body: "Who is unlocking?"
                                            font_color: #x1a1c1e
                                            font_size: 11.
                                        }
                                        lock_staff_label := WrapLabel{
                                            body: "No staff yet — unlock as Owner, then Team"
                                            font_color: #x1a1c1e
                                            font_size: 14.
                                        }
                                        ActionStack{
                                            lock_staff_prev := NavBtn{text: "Previous staff"}
                                            lock_staff_next := NavBtn{text: "Next staff"}
                                        }
                                    }

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 12. right: 12. top: 16. bottom: 16.}
                                        flow: Down spacing: 10
                                        align: Align{x: 0.5}
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xe2e2e5
                                        pin_prompt := WrapLabel{
                                            body: "ENTER OWNER PASSCODE"
                                            font_color: #x1a1c1e
                                            font_size: 11.
                                        }
                                        pin_label := Label{
                                            text: "—"
                                            draw_text.color: #x041534
                                            draw_text.text_style.font_size: 26.
                                        }
                                        pin_count := WrapLabel{
                                            body: "0 / 12 digits · at least 4, then OK"
                                            font_color: #x45464e
                                            font_size: 12.
                                        }
                                        pin_reveal := ChipBtn{
                                            text: "Show PIN"
                                            padding: Inset{left: 14. right: 14.}
                                        }
                                        pin_hint := WrapLabel{
                                            body: "4–12 digits · keyboard or pad · Enter = OK · tap Show PIN to check"
                                            font_color: #x45464e
                                            font_size: 12.
                                        }
                                        pin_status := WrapLabel{
                                            body: " "
                                            font_color: #xa23e18
                                            font_size: 12.
                                        }
                                    }

                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        pin_1 := PinKey{text: "1"}
                                        pin_2 := PinKey{text: "2"}
                                        pin_3 := PinKey{text: "3"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        pin_4 := PinKey{text: "4"}
                                        pin_5 := PinKey{text: "5"}
                                        pin_6 := PinKey{text: "6"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        pin_7 := PinKey{text: "7"}
                                        pin_8 := PinKey{text: "8"}
                                        pin_9 := PinKey{text: "9"}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        pin_clear := PinKey{text: "⌫"}
                                        pin_0 := PinKey{text: "0"}
                                        pin_ok := PinOkKey{text: "OK"}
                                    }

                                    WrapLabel{
                                        body: "Local SQLite · Free Forever · Zero Cloud"
                                        font_color: #x1a1c1e
                                        font_size: 11.
                                    }
                                    WrapLabel{
                                        body: "Or unlock with login id + password"
                                        font_color: #x45464e
                                        font_size: 11.
                                    }
                                    lock_login_id := FieldInput{empty_text: "Login id"}
                                    lock_login_password := FieldInput{empty_text: "Password"}
                                    lock_login_ok := NavBtn{text: "Unlock with login"}
                                }
                            }
                        }

                        onboard_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 12. bottom: 120.}
                                WrapLabel{
                                    body: "Set up your shop"
                                    font_color: #x1a1c1e
                                    font_size: 22.
                                }
                                onboard_status := WrapLabel{
                                    body: "Name this place if you want, then Start fresh or Bring my book."
                                    font_color: #x45464e
                                    font_size: 13.
                                }
                                onboard_venue_input := FieldInput{empty_text: "Name of this place (optional)"}
                                onboard_org_input := FieldInput{empty_text: "Organization (optional)"}
                                onboard_save_venue := NavBtn{text: "Save names"}
                                WrapLabel{
                                    body: "Start template — how this place should feel (None = bare minimal)"
                                    font_color: #x1a1c1e
                                    font_size: 15.
                                }
                                ChipStrip{
                                    ChipStripRow{
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            onboard_tmpl_custom := ChipBtn{text: "None" visible: false}
                                            onboard_tmpl_custom_on := ChipBtnActive{text: "None"}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            onboard_tmpl_counter := ChipBtn{text: "Counter"}
                                            onboard_tmpl_counter_on := ChipBtnActive{text: "Counter" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            onboard_tmpl_courier := ChipBtn{text: "Courier"}
                                            onboard_tmpl_courier_on := ChipBtnActive{text: "Courier" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            onboard_tmpl_food := ChipBtn{text: "Food"}
                                            onboard_tmpl_food_on := ChipBtnActive{text: "Food" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            onboard_tmpl_personal := ChipBtn{text: "Personal"}
                                            onboard_tmpl_personal_on := ChipBtnActive{text: "Personal" visible: false}
                                        }
                                    }
                                }
                                onboard_tmpl_hint := WrapLabel{
                                    body: "None is bare minimal. Other templates only label and seed the same books — not separate apps."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                WrapLabel{
                                    body: "How do you want to start? (tap one — it stays highlighted)"
                                    font_color: #x1a1c1e
                                    font_size: 15.
                                }
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_fresh := RowSecondaryBtn{text: "Start fresh"}
                                    onboard_fresh_on := RowPrimaryBtn{text: "Start fresh" visible: false}
                                }
                                View{
                                    width: Fill height: 52
                                    flow: Overlay
                                    onboard_book := RowSecondaryBtn{text: "Bring my book"}
                                    onboard_book_on := RowPrimaryBtn{text: "Bring my book" visible: false}
                                }
                                WrapLabel{
                                    body: "Fresh = empty catalogue (add rates later in Items). Book = type opening rates and who owes you below."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                onboard_fresh_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    WrapLabel{
                                        body: "Fresh selected — catalogue stays empty. Tap Done when ready."
                                        font_color: #x45464e
                                        font_size: 12.
                                    }
                                }
                                onboard_book_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    WrapLabel{body: "Book — add product / rate" font_color: #x1a1c1e font_size: 15.}
                                    onboard_item_name := FieldInput{empty_text: "Item name"}
                                    onboard_item_price := FieldInput{empty_text: "Price"}
                                    onboard_add_item := NavBtn{text: "Add item"}
                                    WrapLabel{body: "Book — opening credit" font_color: #x1a1c1e font_size: 15.}
                                    onboard_cust_name := FieldInput{empty_text: "Customer name"}
                                    onboard_cust_bal := FieldInput{empty_text: "Amount owed"}
                                    onboard_add_credit := NavBtn{text: "Add opening balance"}
                                }
                                onboard_done := PinOkKey{text: "Done — open books"}
                                WrapLabel{
                                    body: "Later: Settings → apply your country pack for tax and pay labels."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                            }
                        }

                        client_pair_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 12. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 10
                                    align: Align{y: 0.5}
                                    client_pair_back := BackBtn{text: "Back"}
                                    WrapLabel{
                                        body: "Connect to the Server"
                                        font_color: #x1a1c1e
                                        font_size: 22.
                                    }
                                }
                                client_pair_status := WrapLabel{
                                    body: "Enter the Server IP and 6-digit pairing code. Or tap Back to choose Server Mode instead."
                                    font_color: #x45464e
                                    font_size: 13.
                                }
                                client_host_input := FieldInput{empty_text: "Server IP (e.g. 192.168.1.10)"}
                                client_code_input := FieldInput{empty_text: "Pairing code"}
                                client_pair_btn := PinOkKey{text: "Pair now"}
                                client_sync_btn := NavBtn{text: "Refresh from shop"}
                                client_unpair := NavBtn{text: "Forget this shop"}
                                client_skip_lock := NavBtn{text: "Unlock (after pairing)"}
                            }
                        }

                        // Books home — today's position (commons: not Sell-first).
                        books_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                books_heading := WrapLabel{
                                    body: "Books"
                                    font_color: #x1a1c1e
                                    font_size: 22.
                                }
                                books_help := WrapLabel{
                                    width: Fill
                                    body: "Today: money in, purchases/expenses out, they owe us / we owe them, cash vs bank, gross and net."
                                    font_color: #x45464e
                                    font_size: 13.
                                }
                                books_label := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                ActionStack{
                                    books_to_sell := RowPrimaryBtn{text: "Sell (money in)"}
                                    books_to_buy := RowPrimaryBtn{text: "Buy (money out)"}
                                    books_day_close := RowSecondaryBtn{text: "Day close"}
                                    books_reports := RowSecondaryBtn{text: "Reports & bank"}
                                    books_credit := RowSecondaryBtn{text: "Parties / credit"}
                                    books_refresh := RowSecondaryBtn{text: "Refresh"}
                                }
                            }
                        }

                        // Seller: tap items → pay. Whole page scrolls.
                        pos_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}

                                // Operational strip (Stitch screen 02)
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
                                    flow: Right spacing: 8
                                    align: Align{y: 0.5}
                                    draw_bg.color: #xf3f3f6
                                    draw_bg.border_radius: 12.
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 2
                                        session_chip := WrapLabel{
                                            body: "Counter"
                                            font_color: #x1a1c1e
                                            font_size: 12.
                                        }
                                        table_chip := WrapLabel{
                                            body: "Ready to sell"
                                            font_color: #x1a1c1e
                                            font_size: 11.
                                        }
                                    }
                                    Label{
                                        text: "LOCAL"
                                        draw_text.color: #xc59b27
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                empty_catalog := WrapLabel{
                                    body: "No items yet. Open Items to add your first product."
                                    font_color: #x1a1c1e
                                    font_size: 14.
                                }
                                product_list := mod.widgets.ProductList{}

                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 12
                                    align: Align{y: 0.5}
                                    padding: Inset{top: 4. bottom: 4.}
                                    WrapLabel{
                                        width: Fit
                                        body: "Active ledger"
                                        font_color: #x45464e
                                        font_size: 13.
                                    }
                                    View{width: Fill height: 1}
                                    receipt_label := WrapLabel{
                                        width: Fit
                                        body: "Ready"
                                        font_color: #x45464e
                                        font_size: 13.
                                    }
                                    clear_cart := Button{
                                        text: "Clear"
                                        width: Fit height: 32
                                        padding: Inset{left: 14. right: 14. top: 4. bottom: 4.}
                                        draw_bg +: {
                                            color: #xe2e2e5
                                            color_hover: #xd9dadc
                                            color_down: #xc5c6cf
                                            color_focus: #xe2e2e5
                                            border_color: #xc5c6cf
                                            border_radius: 8.
                                        }
                                        draw_text +: {
                                            color: #x93000a
                                            color_hover: #x93000a
                                            color_down: #x93000a
                                            color_focus: #x93000a
                                            text_style +: {font_size: 13.}
                                        }
                                    }
                                }

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
                                    flow: Down spacing: 10
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 16.
                                    draw_bg.border_size: 2.
                                    draw_bg.border_color: #x5c6570
                                    cart_list := mod.widgets.CartList{}
                                    cart_empty_hint := WrapLabel{
                                        body: "Cart empty — tap + ADD on an item, then set QTY with − / +"
                                        font_color: #x45464e
                                        font_size: 12.
                                    }

                                    // Triple totals strip (Actual | Applied | Rounded)
                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
                                        flow: Right spacing: 4
                                        draw_bg.color: #xe8e8ea
                                        draw_bg.border_radius: 12.
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            WrapLabel{
                                                body: "ACTUAL TOTAL"
                                                font_color: #x45464e
                                                font_size: 10.
                                            }
                                            cart_natural_label := WrapLabel{
                                                body: "0"
                                                font_color: #x1a1c1e
                                                font_size: 14.
                                            }
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            WrapLabel{
                                                body: "APPLIED TOTAL"
                                                font_color: #x45464e
                                                font_size: 10.
                                            }
                                            cart_applied_label := WrapLabel{
                                                body: "0"
                                                font_color: #x1a1c1e
                                                font_size: 14.
                                            }
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            cart_charge_caption := WrapLabel{
                                                body: "ROUNDED"
                                                font_color: #xa23e18
                                                font_size: 10.
                                            }
                                            cart_total_input := CartField{
                                                empty_text: "0"
                                                text: "0"
                                                draw_bg +: {
                                                    border_color: #xa23e18
                                                    border_color_hover: #x822801
                                                    border_color_focus: #x041534
                                                    border_radius: 8.
                                                    border_size: 1.
                                                }
                                                draw_text +: {
                                                    color: #x822801
                                                    color_hover: #x822801
                                                    color_down: #x822801
                                                    color_focus: #x822801
                                                    color_empty: #x45464e
                                                    color_empty_hover: #x1a1c1e
                                                    color_empty_focus: #x1a1c1e
                                                    text_style +: {font_size: 16.}
                                                }
                                            }
                                        }
                                    }

                                    // Exclusive-tax shops quote net prices, so the strip above is
                                    // not what the customer pays — this line is.
                                    tax_estimate_label := WrapLabel{
                                        body: " "
                                        font_color: #xa23e18
                                        font_size: 13.
                                    }

                                    // Amounts only — full bill in one field, or mix that adds up.
                                    WrapLabel{
                                        body: "Type how much in each (cash / UPI / card / credit). Must add up to the bill. Credit needs a customer above."
                                        font_color: #x45464e
                                        font_size: 11.
                                    }
                                    money_lane_label := WrapLabel{
                                        body: "Bill: Tax invoice"
                                        font_color: #x45464e
                                        font_size: 12.
                                    }
                                    ChipStrip{
                                        ChipStripRow{
                                            View{
                                                width: Fit height: 40
                                                flow: Overlay
                                                lane_declared := ChipBtn{text: "Tax invoice"}
                                                lane_declared_on := ChipBtnActive{text: "Tax invoice" visible: false}
                                            }
                                            View{
                                                width: Fit height: 40
                                                flow: Overlay
                                                lane_owner := ChipBtn{text: "Cash memo"}
                                                lane_owner_on := ChipBtnActive{text: "Cash memo" visible: false}
                                            }
                                        }
                                    }
                                    mix_cash_input := FieldInput{empty_text: "Cash amount"}
                                    mix_upi_input := FieldInput{empty_text: "UPI amount"}
                                    mix_card_input := FieldInput{empty_text: "Card amount"}
                                    mix_credit_input := FieldInput{empty_text: "Credit / khata amount"}
                                    ActionStack{
                                        pay_mix := PayBtn{text: "Take payment"}
                                        send_kot := RowPrimaryBtn{text: "Send KOT" visible: false}
                                    }

                                    // How this bill is served (hidden unless Tables & kitchen is on).
                                    order_type_row := ChipStrip{
                                        visible: false
                                        ChipStripRow{
                                            View{
                                                width: Fit height: 40
                                                flow: Overlay
                                                order_counter := ChipBtn{text: "Counter"}
                                                order_counter_on := ChipBtnActive{text: "Counter" visible: false}
                                            }
                                            View{
                                                width: Fit height: 40
                                                flow: Overlay
                                                order_dine := ChipBtn{text: "Dine-in"}
                                                order_dine_on := ChipBtnActive{text: "Dine-in" visible: false}
                                            }
                                            View{
                                                width: Fit height: 40
                                                flow: Overlay
                                                order_takeaway := ChipBtn{text: "Takeaway"}
                                                order_takeaway_on := ChipBtnActive{text: "Takeaway" visible: false}
                                            }
                                        }
                                    }
                                    order_type_label := WrapLabel{
                                        body: "Order: Counter"
                                        visible: false
                                        font_color: #x45464e
                                        font_size: 12.
                                    }
                                }

                                cart_round_hint := WrapLabel{
                                    body: "Rounded starts equal to Applied. Edit only to round the bill."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                credit_chip := WrapLabel{
                                    body: "No customer yet — type a name, then tap Credit"
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                customer_name_input := FieldInput{
                                    empty_text: "Customer name"
                                }
                                customer_use := NavBtn{text: "Add / use this name"}
                                customer_clear := NavBtn{text: "Clear customer"}
                                customers_empty_hint := WrapLabel{
                                    body: "Saved customers show up here for quick pick."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                customer_list_sell := mod.widgets.CustomerList{}
                            }
                        }

                        menu_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                items_heading := WrapLabel{body: "Your items" font_color: #x1a1c1e font_size: 20.}
                                items_help_label := WrapLabel{body: "Tap an item to edit it. New item clears the form so you can add a different one. Sales already recorded keep the name they were charged with." font_color: #x1a1c1e}
                                menu_mode_label := WrapLabel{body: "Adding a new item" font_color: #x041534 font_size: 14.}
                                menu_name_input := FieldInput{
                                    empty_text: "Item name — type it in your language"
                                }
                                menu_aliases_input := FieldInput{
                                    empty_text: "Aliases — other names, comma separated"
                                }
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 8
                                    menu_price_input := FieldInput{empty_text: "0"}
                                    menu_unit_input := FieldInput{empty_text: "Unit (kg or blank)"}
                                    menu_new := Button{
                                        text: "New item"
                                        width: Fit height: 52
                                        visible: false
                                        padding: Inset{left: 18. right: 18.}
                                        draw_bg +: {
                                            color: #xffffff
                                            color_hover: #xf3f3f6
                                            color_down: #xe8e8ea
                                            color_focus: #xffffff
                                            color_2: #xffffff
                                            color_2_hover: #xf3f3f6
                                            color_2_down: #xe8e8ea
                                            color_2_focus: #xffffff
                                            border_color: #xc5c6cf
                                            border_radius: 14.
                                            border_size: 1.
                                        }
                                        draw_text +: {
                                            color: #x041534
                                            color_hover: #x041534
                                            color_down: #x041534
                                            color_focus: #x041534
                                            text_style +: {font_size: 15.}
                                        }
                                    }
                                    menu_add := NavBtn{text: "Add" width: Fit padding: Inset{left: 18. right: 18.}}
                                }
                                menu_tax_input := FieldInput{
                                    empty_text: "Tax rate name or % — Standard, Reduced, 0, 5, 12, 18, 20"
                                }
                                menu_category_input := FieldInput{empty_text: "Category (optional, created if new)"}
                                menu_station_row := View{
                                    width: Fill height: Fit
                                    visible: false
                                    menu_station_input := FieldInput{empty_text: "Kitchen station (optional, created if new)"}
                                }
                                menu_modifiers_input := FieldInput{empty_text: "Options (optional) — Large=20, Gift wrap=10"}
                                image_hint := WrapLabel{
                                    body: "Image: auto (from name)"
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                draft_preview := Image{
                                    width: 96 height: 96
                                    fit: ImageFit.CropToFill
                                    min_width: 96
                                    min_height: 96
                                }
                                ActionStack{
                                    image_auto := NavBtn{text: "Auto from name"}
                                    image_prev := NavBtn{text: "Previous image"}
                                    image_next := NavBtn{text: "Next image"}
                                    image_pick := NavBtn{text: "Pick photo"}
                                }
                                menu_empty := WrapLabel{
                                    body: "No items yet — add one above."
                                    font_color: #x1a1c1e
                                    font_size: 14.
                                }
                                product_list_menu := mod.widgets.MenuList{}
                            }
                        }

                        // Money out — purchase + expense (peer of Sell).
                        buy_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                buy_heading := WrapLabel{
                                    body: "Buy"
                                    font_color: #x1a1c1e
                                    font_size: 22.
                                }
                                buy_help := WrapLabel{
                                    width: Fill
                                    body: "Money out: goods received (purchase) or running costs (expense). Same books as Sell."
                                    font_color: #x45464e
                                    font_size: 13.
                                }
                                WrapLabel{body: "Goods received" font_color: #x1a1c1e font_size: 15.}
                                buy_item_input := FieldInput{empty_text: "Item name"}
                                buy_qty_input := FieldInput{empty_text: "Qty (e.g. 1.5 for kg)"}
                                buy_cost_input := FieldInput{empty_text: "Unit cost"}
                                buy_supplier_input := FieldInput{empty_text: "Supplier name (or party)"}
                                buy_party_kind := FieldInput{empty_text: "New party kind (Supplier / Carrier)"}
                                buy_note_input := FieldInput{empty_text: "Note (optional)"}
                                ChipStrip{
                                    ChipStripRow{
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            buy_paid := ChipBtn{text: "Paid now"}
                                            buy_paid_on := ChipBtnActive{text: "Paid now" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            buy_on_account := ChipBtn{text: "On account (AP)"}
                                            buy_on_account_on := ChipBtnActive{text: "On account (AP)" visible: false}
                                        }
                                    }
                                }
                                buy_purchase := NavBtn{text: "Record purchase"}
                                WrapLabel{body: "Expense" font_color: #x1a1c1e font_size: 15.}
                                buy_expense_amount := FieldInput{empty_text: "Amount"}
                                buy_expense_note := FieldInput{empty_text: "Note (fuel, rent, …)"}
                                buy_expense_add := NavBtn{text: "Add expense"}
                                buy_to_inventory := NavBtn{text: "Inventory & stock adjust"}
                            }
                        }

                        more_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                more_heading := WrapLabel{body: "More" font_color: #x1a1c1e font_size: 22.}
                                    WrapLabel{
                                        body: "Catalogue, sales, credit, team, backup — plus stock and extra tills when needed."
                                        font_color: #x45464e
                                    }
                                more_items := MoreRow{text: "Items"}
                                more_tables := MoreRow{text: "Tables"}
                                more_kitchen := MoreRow{text: "Kitchen"}
                                more_sales := MoreRow{text: "Sales & refunds"}
                                more_day_close := MoreRow{text: "Day close"}
                                more_reports := MoreRow{text: "Reports & bank"}
                                more_inventory := MoreRow{text: "Inventory & stock"}
                                more_branches := MoreRow{text: "Branches & org"}
                                more_credit := MoreRow{text: "Credit customers"}
                                more_team := MoreRow{text: "Team & PINs"}
                                more_settings := MoreRow{text: "Backup & settings"}
                                more_lock := MoreRow{text: "Lock app"}
                                more_mode_hint := WrapLabel{
                                    body: " "
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                            }
                        }

                        floor_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    floor_back := BackBtn{text: "Back"}
                                    floor_title := WrapLabel{body: "Tables" font_color: #x1a1c1e font_size: 20.}
                                }
                                clear_table := NavBtn{text: "Counter / takeaway"}
                                floor_mode_hint := WrapLabel{
                                    body: " "
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                floor_table_name := FieldInput{empty_text: "New table name"}
                                floor_add_table := NavBtn{text: "Add table"}
                                floor_rename_table := NavBtn{text: "Rename selected"}
                                floor_delete_table := NavBtn{text: "Delete selected table"}
                                table_list := mod.widgets.TableList{}
                            }
                        }

                        kitchen_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    kitchen_back := BackBtn{text: "Back"}
                                    kitchen_title := WrapLabel{body: "Kitchen" font_color: #x1a1c1e font_size: 20.}
                                }
                                kitchen_mode_hint := WrapLabel{
                                    body: " "
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ActionStack{
                                    kitchen_station_all := NavBtn{text: "All stations"}
                                    kitchen_station_cycle := NavBtn{text: "Next station"}
                                }
                                kitchen_station_label := WrapLabel{
                                    body: "Station: all"
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                kot_list := mod.widgets.KotList{}
                            }
                        }

                        sales_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    sales_back := BackBtn{text: "Back"}
                                    sales_title := WrapLabel{body: "Sales" font_color: #x1a1c1e font_size: 20.}
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Select a sale to amend, void, or refund. Amendments keep full history."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                sale_selected_label := WrapLabel{
                                    body: "No sale selected"
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                receipt_share_btn := NavBtn{text: "Copy receipt"}
                                receipt_print_btn := NavBtn{text: "Print / open"}
                                day_close_from_sales := NavBtn{text: "Day close summary"}
                                amend_cart_list := mod.widgets.CartList{}
                                amend_natural_label := WrapLabel{
                                    body: "Actual Total 0"
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                amend_applied_label := WrapLabel{
                                    body: "Applied Total 0"
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{y: 0.5}
                                        WrapLabel{
                                            body: "Rounded Total"
                                            font_color: #xa23e18
                                            font_size: 14.
                                        }
                                        amend_total_input := CartField{
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        empty_text: "0"
                                        text: "0"
                                        draw_bg +: {
                                            border_color: #xa23e18
                                            border_color_hover: #x822801
                                            border_color_focus: #x041534
                                            border_radius: 12.
                                            border_size: 2.
                                        }
                                        draw_text +: {
                                            color: #x822801
                                            color_hover: #x822801
                                            color_down: #x822801
                                            color_focus: #x822801
                                            color_empty: #x45464e
                                            color_empty_hover: #x1a1c1e
                                            color_empty_focus: #x1a1c1e
                                            text_style +: {font_size: 16.}
                                        }
                                    }
                                }
                                amend_round_hint := WrapLabel{
                                    body: "Rounded Total starts equal to Applied Total. Edit only to round the bill."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                ActionStack{
                                    amend_pay_cash := PayBtn{text: "Cash"}
                                    amend_pay_upi := PayBtn{text: "UPI"}
                                    amend_pay_card := PayBtn{text: "Card"}
                                }
                                reason_input := FieldInput{
                                    empty_text: "Reason for amend / void / refund"
                                }
                                approval_pin_input := FieldInput{
                                    empty_text: "Manager PIN to approve (only if you cannot void)"
                                }
                                approval_hint := WrapLabel{
                                    body: " "
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ActionStack{
                                    amend_btn := NavBtn{text: "Amend"}
                                    void_btn := NavBtn{text: "Void"}
                                    refund_btn := NavBtn{text: "Refund"}
                                }
                                sale_history_label := WrapLabel{
                                    body: "History: —"
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                sale_list := mod.widgets.SaleList{}
                            }
                        }

                        credit_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    credit_back := BackBtn{text: "Back"}
                                    credit_title := WrapLabel{body: "Credit" font_color: #x1a1c1e font_size: 20.}
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Parties who owe you (AR). Settle with cash, write off to nil, or take advance. Kind = Carrier / Consignor / Supplier / …"
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                credit_summary_label := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                credit_add_name := FieldInput{empty_text: "Party name"}
                                credit_add_kind := FieldInput{empty_text: "Kind (Consignor, Carrier, Supplier, …)"}
                                credit_add_btn := NavBtn{text: "Add party"}
                                customer_list_credit := mod.widgets.CustomerList{}
                                WrapLabel{body: "Pay selected (settle debt / take advance)" font_color: #x1a1c1e font_size: 15.}
                                settle_amount_input := FieldInput{empty_text: "Amount received"}
                                ActionStack{
                                    settle_cash := NavBtn{text: "Pay cash"}
                                    settle_upi := NavBtn{text: "Pay UPI"}
                                    settle_card := NavBtn{text: "Pay card"}
                                }
                                WrapLabel{body: "Write off (clear without payment)" font_color: #x1a1c1e font_size: 15.}
                                writeoff_amount_input := FieldInput{empty_text: "Amount (blank = full balance)"}
                                writeoff_reason_input := FieldInput{empty_text: "Reason (required)"}
                                credit_writeoff := NavBtn{text: "Write off / nil balance"}
                                WrapLabel{body: "Pay what we owe them (AP)" font_color: #x1a1c1e font_size: 15.}
                                payable_amount_input := FieldInput{empty_text: "Amount we pay"}
                                ActionStack{
                                    payable_cash := NavBtn{text: "Pay AP cash"}
                                    payable_upi := NavBtn{text: "Pay AP UPI"}
                                    payable_card := NavBtn{text: "Pay AP card"}
                                }
                                payable_writeoff := NavBtn{text: "Write off AP"}
                            }
                        }

                        team_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    team_back := BackBtn{text: "Back"}
                                    team_title := WrapLabel{body: "Team" font_color: #x1a1c1e font_size: 20.}
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Owner creates each person with a name, role title, permission preset, and starting PIN. Owner cannot change anyone's PIN later — only that person can update their own PIN."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                staff_label := WrapLabel{body: "" font_color: #x1a1c1e}
                                staff_name_input := FieldInput{empty_text: "Staff name / id"}
                                staff_title_input := FieldInput{empty_text: "Role title (e.g. Barista)"}
                                staff_pin_input := FieldInput{empty_text: "Starting PIN (4–12 digits)"}
                                WrapLabel{
                                    body: "Permission preset (what they can access)"
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Cashier: sell. Manager: sell + items + void/amend."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ChipStrip{
                                    ChipStripRow{
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            staff_role_cashier := ChipBtn{text: "Cashier"}
                                            staff_role_cashier_on := ChipBtnActive{text: "Cashier" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            staff_role_manager := ChipBtn{text: "Manager"}
                                            staff_role_manager_on := ChipBtnActive{text: "Manager" visible: false}
                                        }
                                        staff_role_waiter_slot := View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            visible: false
                                            staff_role_waiter := ChipBtn{text: "Waiter"}
                                            staff_role_waiter_on := ChipBtnActive{text: "Waiter" visible: false}
                                        }
                                        staff_role_kitchen_slot := View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            visible: false
                                            staff_role_kitchen := ChipBtn{text: "Kitchen"}
                                            staff_role_kitchen_on := ChipBtnActive{text: "Kitchen" visible: false}
                                        }
                                    }
                                }
                                staff_add_cashier := NavBtn{text: "Add staff"}
                                staff_set_login := NavBtn{text: "Set login on selected"}
                                staff_deactivate := NavBtn{text: "Remove selected"}
                                WrapLabel{
                                    body: "Optional login (id + password). Owner sets once; only the user can change password later."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                staff_login_input := FieldInput{empty_text: "Login id (optional)"}
                                staff_password_input := FieldInput{empty_text: "Starting password (optional)"}
                                WrapLabel{
                                    body: "Feature access for selected staff (tap after adding)"
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                ChipStrip{
                                    ChipStripRow{
                                        feat_sell := ChipBtn{text: "Sell"}
                                        feat_items := ChipBtn{text: "Items"}
                                        feat_sales := ChipBtn{text: "Sales"}
                                        feat_credit := ChipBtn{text: "Credit"}
                                        feat_floor := ChipBtn{text: "Floor" visible: false}
                                        feat_kitchen := ChipBtn{text: "Kitchen" visible: false}
                                        feat_team := ChipBtn{text: "Team"}
                                        feat_backup := ChipBtn{text: "Backup"}
                                        feat_inventory := ChipBtn{text: "Inventory"}
                                        feat_branches := ChipBtn{text: "Branches"}
                                    }
                                }
                                feat_save := NavBtn{text: "Save features for selected staff"}
                                WrapLabel{
                                    body: "Change my PIN (only yours)"
                                    font_color: #x1a1c1e
                                    font_size: 15.
                                }
                                my_pin_input := FieldInput{empty_text: "New PIN (4–12 digits)"}
                                change_my_pin := NavBtn{text: "Update my PIN"}
                                my_password_input := FieldInput{empty_text: "New password (optional)"}
                                change_my_password := NavBtn{text: "Update my password"}
                            }
                        }

                        settings_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 12
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    settings_back := BackBtn{text: "Back"}
                                    settings_title := WrapLabel{body: "Language & currency" font_color: #x1a1c1e font_size: 20.}
                                }
                                locale_label := WrapLabel{body: "" font_color: #x1a1c1e}
                                WrapLabel{body: "Country & tax" font_color: #x1a1c1e font_size: 20.}
                                WrapLabel{
                                    width: Fill
                                    body: "A country pack sets the tax name and maths, the instant-pay rail, and the business id printed on bills. Kumkaj does not file taxes or issue government e-invoices anywhere."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                fiscal_label := WrapLabel{body: "" font_color: #x1a1c1e font_size: 13.}
                                country_code_input := FieldInput{
                                    empty_text: "Country code e.g. IN, GB, US, AE, BR"
                                }
                                ActionStack{
                                    country_prev := NavBtn{text: "Previous country"}
                                    country_next := NavBtn{text: "Next country"}
                                }
                                country_apply := NavBtn{text: "Apply country pack"}
                                country_hint := WrapLabel{
                                    body: " "
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ActionStack{
                                    tax_inclusive_btn := NavBtn{text: "Tax inside price"}
                                    tax_exclusive_btn := NavBtn{text: "Tax added at till"}
                                }
                                business_tax_id_input := FieldInput{empty_text: "Business tax id"}
                                service_bps_input := FieldInput{
                                    empty_text: "Service % optional 0/10/12"
                                }
                                fiscal_save := NavBtn{text: "Save business id & service %"}
                                WrapLabel{body: "Language" font_color: #x1a1c1e font_size: 15.}
                                WrapLabel{
                                    width: Fill
                                    body: "Follow this device’s language, or pick one yourself."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                ChipStrip{
                                    ChipStripRow{
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            settings_lang_platform := ChipBtn{text: "Use platform native language"}
                                            settings_lang_platform_on := ChipBtnActive{text: "Use platform native language" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            settings_lang_user := ChipBtn{text: "Use user selected language"}
                                            settings_lang_user_on := ChipBtnActive{text: "Use user selected language" visible: false}
                                        }
                                    }
                                }
                                settings_lang_selected := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                language_list := mod.widgets.LanguageList{}
                                WrapLabel{body: "Currency" font_color: #x1a1c1e font_size: 15.}
                                WrapLabel{
                                    width: Fill
                                    body: "Scroll and tap Use for any supported currency."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                currency_list := mod.widgets.CurrencyList{}
                                WrapLabel{body: "Tables & kitchen" font_color: #x1a1c1e font_size: 20.}
                                venue_mode_label := WrapLabel{
                                    body: "Off — no tables or kitchen tickets"
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ActionStack{
                                    mode_counter := RowPrimaryBtn{text: "Off"}
                                    mode_restaurant := RowSecondaryBtn{text: "Tables & kitchen"}
                                }
                                WrapLabel{body: "Extra tills" font_color: #x1a1c1e font_size: 20.}
                                server_pair_label := WrapLabel{
                                    body: " "
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                ActionStack{
                                    allow_extra_tills := RowPrimaryBtn{text: "Allow extra tills"}
                                    stop_extra_tills := RowSecondaryBtn{text: "Stop extra tills"}
                                }
                                settings_open_branches := NavBtn{text: "Branches & org"}
                                WrapLabel{body: "Start template" font_color: #x1a1c1e font_size: 20.}
                                settings_tmpl_label := WrapLabel{
                                    body: "Current: None"
                                    font_color: #x45464e
                                    font_size: 13.
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Switch labels and defaults without wiping sales. Catalogue and parties stay yours — nothing is auto-seeded."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ChipStrip{
                                    ChipStripRow{
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            settings_tmpl_custom := ChipBtn{text: "None" visible: false}
                                            settings_tmpl_custom_on := ChipBtnActive{text: "None"}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            settings_tmpl_counter := ChipBtn{text: "Counter"}
                                            settings_tmpl_counter_on := ChipBtnActive{text: "Counter" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            settings_tmpl_courier := ChipBtn{text: "Courier"}
                                            settings_tmpl_courier_on := ChipBtnActive{text: "Courier" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            settings_tmpl_food := ChipBtn{text: "Food"}
                                            settings_tmpl_food_on := ChipBtnActive{text: "Food" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            settings_tmpl_personal := ChipBtn{text: "Personal"}
                                            settings_tmpl_personal_on := ChipBtnActive{text: "Personal" visible: false}
                                        }
                                    }
                                }
                                WrapLabel{body: "Your brand (in-app)" font_color: #x1a1c1e font_size: 20.}
                                WrapLabel{
                                    width: Fill
                                    body: "Shown on screens inside the app. The installed app name and icon on this device stay as the platform listing."
                                    font_color: #x1a1c1e
                                    font_size: 12.
                                }
                                venue_title_input := FieldInput{empty_text: "Name of this place"}
                                venue_brand_hint := WrapLabel{
                                    body: "Icon: none — choose one"
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ActionStack{
                                    venue_icon_pick := RowPrimaryBtn{text: "Choose icon"}
                                    venue_icon_clear := RowSecondaryBtn{text: "Clear icon"}
                                }
                                venue_brand_save := NavBtn{text: "Save brand"}
                                WrapLabel{body: "Units" font_color: #x1a1c1e font_size: 20.}
                                WrapLabel{
                                    width: Fill
                                    body: "Codes for item labels (kg, piece, bag). Conversions: 1 from = factor to (e.g. 1 bag = 25 kg)."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                units_label := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                unit_code_input := FieldInput{empty_text: "Code (bag)"}
                                unit_name_input := FieldInput{empty_text: "Name (Bag)"}
                                unit_add := NavBtn{text: "Add unit"}
                                conv_from_input := FieldInput{empty_text: "From code (bag)"}
                                conv_to_input := FieldInput{empty_text: "To code (kg)"}
                                conv_factor_input := FieldInput{empty_text: "Factor (25 = 1 bag → 25 kg)"}
                                conv_add := NavBtn{text: "Add conversion"}
                                WrapLabel{body: "Backup" font_color: #x1a1c1e font_size: 20.}
                                db_path_label := WrapLabel{body: "" font_color: #x1a1c1e}
                                audit_label := WrapLabel{body: "" font_color: #x1a1c1e}
                                backup_btn := NavBtn{text: "Create backup"}
                                backup_path_label := WrapLabel{body: "" font_color: #x1a1c1e}
                                WrapLabel{
                                    width: Fill
                                    body: "Restore: put restore.sqlite3 in the backups folder, then tap Restore."
                                    font_color: #x1a1c1e
                                }
                                restore_btn := NavBtn{text: "Restore backup"}
                            }
                        }

                        day_close_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    day_close_back := BackBtn{text: "Back"}
                                    day_close_title := WrapLabel{body: "Day close" font_color: #x1a1c1e font_size: 20.}
                                }
                                day_close_label := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 14.
                                }
                                WrapLabel{body: "Drawer count (optional)" font_color: #x1a1c1e font_size: 15.}
                                day_close_counted_input := FieldInput{empty_text: "Counted cash in drawer"}
                                day_close_copy := NavBtn{text: "Copy day close"}
                                day_close_commit := NavBtn{text: "Record day close"}
                                WrapLabel{
                                    width: Fill
                                    body: "Recording a day close saves this summary (and drawer variance if counted) to the audit log. It does not stop selling."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                            }
                        }

                        reports_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    reports_back := BackBtn{text: "Back"}
                                    reports_title := WrapLabel{body: "Reports" font_color: #x1a1c1e font_size: 20.}
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Day books, party aging, bank posts, and period P&L — same books for every entity."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ActionStack{
                                    reports_today := RowPrimaryBtn{text: "Today books"}
                                    reports_month := RowSecondaryBtn{text: "This month"}
                                    reports_aging := RowSecondaryBtn{text: "Party aging"}
                                }
                                reports_label := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                WrapLabel{body: "Log expense" font_color: #x1a1c1e font_size: 15.}
                                expense_amount_input := FieldInput{empty_text: "Amount"}
                                expense_note_input := FieldInput{empty_text: "Note (fuel, office, …)"}
                                expense_add := NavBtn{text: "Add expense"}
                                WrapLabel{body: "Bank / cashbook" font_color: #x1a1c1e font_size: 15.}
                                bank_amount_input := FieldInput{empty_text: "Amount"}
                                bank_note_input := FieldInput{empty_text: "Note (deposit, withdrawal, …)"}
                                ActionStack{
                                    bank_till_to_bank := RowPrimaryBtn{text: "Till → bank"}
                                    bank_bank_to_till := RowSecondaryBtn{text: "Bank → till"}
                                    bank_in := RowSecondaryBtn{text: "Bank in"}
                                    bank_out := RowSecondaryBtn{text: "Bank out"}
                                }
                                reports_copy := NavBtn{text: "Copy report"}
                                reports_refresh := NavBtn{text: "Refresh report"}
                            }
                        }

                        inventory_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    inventory_back := BackBtn{text: "Back"}
                                    inventory_title := WrapLabel{body: "Inventory" font_color: #x1a1c1e font_size: 20.}
                                }
                                inventory_label := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Type the item name (part of it is enough), a quantity, then adjust. Items that are not counted yet need Start tracking once."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                inv_item_input := FieldInput{empty_text: "Item name"}
                                inv_qty_input := FieldInput{empty_text: "Quantity"}
                                inv_note_input := FieldInput{empty_text: "Reason / note (wastage, count, …)"}
                                ActionStack{
                                    inv_adjust_minus := NavBtn{text: "− Stock"}
                                    inv_adjust_plus := NavBtn{text: "+ Stock"}
                                    inv_track := NavBtn{text: "Start tracking"}
                                }
                                WrapLabel{body: "Goods received" font_color: #x1a1c1e font_size: 15.}
                                inv_cost_input := FieldInput{empty_text: "Unit cost"}
                                inv_supplier_input := FieldInput{empty_text: "Supplier"}
                                ActionStack{
                                    inv_purchase := NavBtn{text: "Record purchase"}
                                    inv_refresh := NavBtn{text: "Refresh"}
                                }
                            }
                        }

                        branches_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    branches_back := BackBtn{text: "Back"}
                                    branches_title := WrapLabel{body: "Branches" font_color: #x1a1c1e font_size: 20.}
                                }
                                org_name_input := FieldInput{empty_text: "Business name"}
                                org_save := NavBtn{text: "Save business name"}
                                branches_label := WrapLabel{
                                    body: ""
                                    font_color: #x1a1c1e
                                    font_size: 13.
                                }
                                branch_name_input := FieldInput{empty_text: "New branch name"}
                                branch_code_input := FieldInput{empty_text: "Short code (MG02)"}
                                branch_add := NavBtn{text: "Add branch"}
                                branch_switch_input := FieldInput{empty_text: "Branch name to sell from"}
                                branch_switch := NavBtn{text: "Switch this device"}
                                WrapLabel{
                                    width: Fill
                                    body: "Export writes branch_snapshot_CODE_DATE.json into the backups folder and copies it to the clipboard. To import another outlet's day, save its file as import_branch_snapshot.json in the same folder, then tap Import."
                                    font_color: #x45464e
                                    font_size: 12.
                                }
                                ActionStack{
                                    branch_export := RowPrimaryBtn{text: "Export day"}
                                    branch_import := RowPrimaryBtn{text: "Import day"}
                                    branch_consolidated := RowSecondaryBtn{text: "Roll-up"}
                                }
                            }
                        }

                    }

                    // Post-charge undo snackbar (above tabs; auto-hides after 10s).
                    undo_snackbar := RoundedView{
                        visible: false
                        width: Fill height: Fit
                        padding: Inset{left: 12. right: 8. top: 10. bottom: 10.}
                        margin: Inset{left: 8. right: 8. bottom: 4.}
                        flow: Right spacing: 8
                        align: Align{y: 0.5}
                        draw_bg.color: #x1b2a4a
                        draw_bg.border_radius: 12.
                        undo_snack_label := WrapLabel{
                            width: Fill
                            body: "Sale recorded"
                            font_color: #xffffff
                            font_size: 13.
                        }
                        undo_snack_btn := Button{
                            text: "Undo"
                            width: Fit height: 36
                            padding: Inset{left: 14. right: 14.}
                            draw_bg +: {
                                color: #xc59b27
                                color_hover: #xd4ae3f
                                color_down: #xa88420
                                color_focus: #xc59b27
                                color_2: #xc59b27
                                color_2_hover: #xd4ae3f
                                color_2_down: #xa88420
                                color_2_focus: #xc59b27
                                border_radius: 10.
                                border_size: 0.
                            }
                            draw_text +: {
                                color: #x041534
                                color_hover: #x041534
                                color_down: #x041534
                                color_focus: #x041534
                                text_style +: {font_size: 13.}
                            }
                        }
                        undo_snack_dismiss := Button{
                            text: "✕"
                            width: 36 height: 36
                            draw_bg +: {
                                color: #x24365e
                                color_hover: #x2f4575
                                color_down: #x0f172a
                                color_focus: #x24365e
                                color_2: #x24365e
                                color_2_hover: #x2f4575
                                color_2_down: #x0f172a
                                color_2_focus: #x24365e
                                border_radius: 10.
                                border_size: 0.
                            }
                            draw_text +: {
                                color: #xffffff
                                color_hover: #xffffff
                                color_down: #xffffff
                                color_focus: #xffffff
                                text_style +: {font_size: 14.}
                            }
                        }
                    }

                    // docs/DESIGN.md · BUSINESS_BOOKS.md: Books · Sell · Buy · More
                    bottom_nav := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 4. right: 4. top: 4. bottom: 8.}
                        flow: Right spacing: 0
                        draw_bg.color: #x041534
                        nav_books := TabBtn{text: "Books"}
                        nav_sell := TabBtn{text: "Sell"}
                        nav_buy := TabBtn{text: "Buy"}
                        nav_more := TabBtn{text: "More"}
                    }
                    } // app_shell
                    } // app_hscroll
                }
            }
        }
    }
}





fn typed_aliases(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Parse `1`, `1.5`, `1,5` into qty milli (1000 = 1 whole).
fn parse_qty_to_milli(text: &str) -> Result<i64, String> {
    let t = text.trim().replace(',', ".");
    if t.is_empty() {
        return Err("Quantity required".into());
    }
    if t.contains('.') {
        let v: f64 = t
            .parse()
            .map_err(|_| "Quantity must be a number".to_string())?;
        if v <= 0.0 {
            return Err("Quantity must be > 0".into());
        }
        Ok((v * 1000.0).round() as i64)
    } else {
        let whole: i64 = t
            .parse()
            .map_err(|_| "Quantity must be a number".to_string())?;
        if whole <= 0 {
            return Err("Quantity must be > 0".into());
        }
        Ok(domain::qty_to_milli(whole))
    }
}

fn modifiers_as_typed(mods: &[domain::ModifierOption], currency: &str) -> String {
    mods.iter()
        .map(|modifier| {
            let price = locale::format_major_input(modifier.price_paise, currency);
            if price.is_empty() || price == "0" {
                modifier.name.clone()
            } else {
                format!("{}={price}", modifier.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn menu_tax_text(store: &Store, product: &Product) -> String {
    if !product.tax_rate_id.is_empty()
        && let Ok(rates) = store.list_tax_rates()
        && let Some(rate) = rates.iter().find(|rate| rate.id == product.tax_rate_id)
    {
        return rate.name.clone();
    }
    locale::format_percent_bps(product.tax_bps, ".")
}

fn name_for_id<'a>(mut pairs: impl Iterator<Item = (&'a str, &'a str)>, id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    pairs
        .find(|(row_id, _)| *row_id == id)
        .map(|(_, name)| name.to_string())
        .unwrap_or_default()
}

fn item_images_dir() -> std::path::PathBuf {
    app_state()
        .lock()
        .unwrap()
        .store
        .as_ref()
        .and_then(|s| s.item_images_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Start a thumbnail decode for this widget and path.
///
/// Returns true only when a new decode was queued. PortalList rebinds every
/// draw; calling the loader again while a decode is in flight calls
/// `begin_async_load`, which redraws, which calls the loader again.
fn load_thumb_at(cx: &mut Cx, image: &ImageRef, path: &std::path::Path) -> bool {
    static REQUESTED: OnceLock<Mutex<std::collections::HashMap<u64, std::path::PathBuf>>> =
        OnceLock::new();
    let requested = REQUESTED.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let uid = image.widget_uid().0;
    {
        let map = requested.lock().unwrap();
        if map.get(&uid).is_some_and(|p| p.as_path() == path) {
            return false;
        }
    }
    // Record the request before the call. The async start redraws immediately,
    // and that redraw must not queue the same file again.
    requested.lock().unwrap().insert(uid, path.to_path_buf());
    let queued = if path.is_file() {
        // File read stays on this thread; the decode itself does not.
        // Android still decodes inline inside Makepad, so the once-per-path
        // guard above is what keeps a redraw from decoding the file again.
        image.load_image_file_by_path_async(cx, path).is_ok()
    } else if let Some(bytes) = menu_images::read_image_bytes(cx, path) {
        image
            .load_image_from_data_async(cx, path, std::sync::Arc::new(bytes))
            .is_ok()
    } else {
        false
    };
    if !queued {
        requested.lock().unwrap().remove(&uid);
    }
    queued
}

fn list_content_height(count: usize, row: f64, gap: f64) -> f64 {
    let n = count as f64;
    if n <= 0.0 {
        0.0
    } else {
        n * row + (n - 1.0) * gap
    }
}

fn remember_content_row(cx: &mut Cx, view: &mut View, slot: &Mutex<f64>, used: f64, measured: f64) {
    if measured > 1.0 && (measured - used).abs() > 1.0 {
        *slot.lock().unwrap() = measured;
        view.redraw(cx);
    }
}

fn product_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed clears thumb + name + price + the ADD face. The draw replaces it
    // with the measured row.
    HEIGHT.get_or_init(|| Mutex::new(260.0))
}

fn menu_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(220.0))
}

fn cart_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(320.0))
}

/// Nested PortalList inside a page ScrollYView: claim the wheel/trackpad so the
/// outer page does not scroll at the same time as the inner list.
fn claim_nested_scroll(cx: &Cx, area: Area, event: &Event) {
    let Event::Scroll(e) = event else {
        return;
    };
    if !area.is_valid(cx) {
        return;
    }
    if area.clipped_rect(cx).contains(e.abs) {
        e.handled_y.set(true);
        if e.scroll.x != 0.0 {
            e.handled_x.set(true);
        }
    }
}

fn claim_list_nested_scroll(cx: &mut Cx, host: &View, event: &Event) {
    let list_area = host.widget(cx, ids!(list)).area();
    let area = if list_area.is_valid(cx) {
        list_area
    } else {
        host.area()
    };
    // Claim before the list handles the event so the page ScrollYView never
    // races the same wheel tick; PortalList also marks handled when it moves.
    claim_nested_scroll(cx, area, event);
}

/// Walk the country list one step from `from`, wrapping at both ends. A code
/// that matches nothing (half-typed, say) resumes from the applied pack.
fn step_country(from: &str, step: i64, applied: &str) -> &'static fiscal::CountryProfile {
    let list = fiscal::countries();
    let at = list
        .iter()
        .position(|c| c.code.eq_ignore_ascii_case(from.trim()))
        .or_else(|| list.iter().position(|c| c.code.eq_ignore_ascii_case(applied.trim())))
        .unwrap_or(0) as i64;
    let n = list.len() as i64;
    &list[(((at + step) % n + n) % n) as usize]
}

/// Expand-to-content lists: let the page ScrollYView own wheel/trackpad.
/// Forwarding Scroll into PortalList makes it eat the gesture at the bottom edge
/// so Tea / pay / anything below never comes into view.
fn page_owns_scroll(event: &Event) -> bool {
    matches!(event, Event::Scroll(_))
}

fn load_product_thumb(cx: &mut Cx, item: &WidgetRef, product: &Product) {
    let dir = item_images_dir();
    let path = menu_images::resolve_image_path(product.image_key.as_deref(), &product.name, &dir);
    if load_thumb_at(cx, &item.image(cx, ids!(thumb)), &path) {
        // CachedView otherwise keeps the previous occupant's thumbnail.
        item.as_view().redraw_texture_cache();
    }
}

#[derive(Script, ScriptHook, Widget)]
struct ProductList {
    #[deref]
    view: View,
}

impl Widget for ProductList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (products, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.products.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let rows = products.len().div_ceil(3);
        let row = *product_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(rows, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows);
                while let Some(item_id) = list.next_visible_item(cx) {
                    // PortalList keeps asking for items until the viewport is
                    // filled. Past our row count, drawing a zero-height row
                    // (all cards invisible) never fills it → infinite loop and
                    // OS "app not responding" after unlock shows Sell.
                    if item_id >= rows {
                        continue;
                    }
                    let item = list.item(cx, item_id, id!(Item));
                    for (col, path) in [
                        (0usize, &[id!(c0)][..]),
                        (1, &[id!(c1)][..]),
                        (2, &[id!(c2)][..]),
                    ] {
                        let idx = item_id * 3 + col;
                        let card = item.widget(cx, path);
                        if let Some(product) = products.get(idx) {
                            card.set_visible(cx, true);
                            card.label(cx, ids!(name)).set_text(cx, &product.name);
                            card.label(cx, ids!(price))
                                .set_text(cx, &product.price.format_localized(&currency, &fiscal));
                            load_product_thumb(cx, &card, product);
                            let add = card.button(cx, ids!(add));
                            if product.available {
                                add.set_text(cx, &locale::t(&lang, "add_upper"));
                                add.set_enabled(cx, true);
                            } else {
                                add.set_text(cx, &locale::t(&lang, "na"));
                                add.set_enabled(cx, false);
                            }
                        } else {
                            card.set_visible(cx, false);
                        }
                    }
                    item.draw_all_unscoped(cx);
                }
                if rows > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / rows as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, product_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct MenuList {
    #[deref]
    view: View,
}

impl Widget for MenuList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (products, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.products.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let count = products.len();
        let row = *menu_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(product) = products.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.label(cx, ids!(name)).set_text(cx, &product.name);
                    item.label(cx, ids!(price))
                        .set_text(cx, &product.price.format_localized(&currency, &fiscal));
                    let stock_line = if product.tracks_stock() {
                        format!(
                            "{} {}",
                            product.stock_qty,
                            locale::t(&lang, "in_stock")
                        )
                    } else if product.available {
                        locale::t(&lang, "not_tracked")
                    } else {
                        locale::t(&lang, "marked_sold_out")
                    };
                    item.label(cx, ids!(stock)).set_text(cx, &stock_line);
                    load_product_thumb(cx, &item, product);
                    item.button(cx, ids!(sold_out)).set_text(
                        cx,
                        &locale::t(
                            &lang,
                            if product.available {
                                "sold_out"
                            } else {
                                "restock"
                            },
                        ),
                    );
                    item.button(cx, ids!(remove)).set_text(cx, "✕");
                    item.draw_all_unscoped(cx);
                }
                if count > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / count as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, menu_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct CartList {
    #[deref]
    view: View,
}

impl Widget for CartList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (lines, products, currency, fiscal) = {
            let st = app_state().lock().unwrap();
            let lines = if st.screen == Screen::Sales {
                st.amend_cart.lines.clone()
            } else {
                st.cart.lines.clone()
            };
            (
                lines,
                st.products.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
            )
        };
        let count = lines.len();
        let row = *cart_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0).max(56.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, lines.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(line) = lines.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let actual = line.actual_total();
                    let applied = line.applied_total();
                    let qty_milli = if line.qty_milli > 0 {
                        line.qty_milli
                    } else {
                        line.qty.saturating_mul(1000)
                    };
                    let qty_show = {
                        let whole = qty_milli / 1000;
                        let frac = qty_milli % 1000;
                        let num = if frac == 0 {
                            format!("{whole}")
                        } else {
                            format!("{whole}.{frac:03}")
                                .trim_end_matches('0')
                                .trim_end_matches('.')
                                .to_string()
                        };
                        let unit = line.unit_label.trim();
                        if unit.is_empty() {
                            num
                        } else {
                            format!("{num} {unit}")
                        }
                    };
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&line.name));
                    item.label(cx, ids!(qty_label))
                        .set_text(cx, &qty_show);
                    // Actual = @ × qty — always show the real money, never leave ₹0.
                    item.widget(cx, ids!(line_actual_label)).set_text(cx, &plain_display_text(&format!(
                            "Actual {}  ({qty_show} × {})",
                            actual.format_localized(&currency, &fiscal),
                            line.unit_price.format_localized(&currency, &fiscal)
                        ),));
                    let price_input = item.text_input(cx, ids!(unit_price));
                    if !cx.has_key_focus(price_input.area()) {
                        let price_text =
                            locale::format_major_input_ex(
                                line.unit_price.paise(),
                                &currency,
                                fiscal.decimal_sep(),
                            );
                        if price_input.text() != price_text {
                            price_input.set_text(cx, &price_text);
                        }
                    }
                    let line_total_input = item.text_input(cx, ids!(line_total));
                    // Never rewrite while focused — empty/"0" mid-edit must stay
                    // editable (same rule as unit price / note).
                    if !cx.has_key_focus(line_total_input.area()) {
                        let applied_text =
                            locale::format_major_input_ex(
                                applied.paise(),
                                &currency,
                                fiscal.decimal_sep(),
                            );
                        if line_total_input.text() != applied_text {
                            line_total_input.set_text(cx, &applied_text);
                        }
                    }
                    let note_input = item.text_input(cx, ids!(note));
                    if !cx.has_key_focus(note_input.area()) && note_input.text() != line.note {
                        note_input.set_text(cx, &line.note);
                    }
                    let mods = products
                        .iter()
                        .find(|p| p.id == line.product_id)
                        .map(|p| p.modifiers())
                        .unwrap_or_default();
                    item.view(cx, ids!(mod_row)).set_visible(cx, !mods.is_empty());
                    let pages = mods.len().div_ceil(domain::MODIFIER_PAGE).max(1);
                    let page = line.modifier_page % pages;
                    let start = page * domain::MODIFIER_PAGE;
                    item.button(cx, ids!(mod_prev))
                        .set_visible(cx, mods.len() > domain::MODIFIER_PAGE);
                    item.button(cx, ids!(mod_next))
                        .set_visible(cx, mods.len() > domain::MODIFIER_PAGE);
                    for (slot, path) in [
                        (0, ids!(mod_0)),
                        (1, ids!(mod_1)),
                        (2, ids!(mod_2)),
                    ] {
                        let button = item.button(cx, path);
                        match mods.get(start + slot) {
                            Some(modifier) => {
                                let on = line
                                    .applied_modifiers
                                    .iter()
                                    .any(|m| m.name == modifier.name);
                                let price = if modifier.price_paise == 0 {
                                    String::new()
                                } else {
                                    format!(
                                        " +{}",
                                        Money(modifier.price_paise)
                                            .format_localized(&currency, &fiscal)
                                    )
                                };
                                let mark = if on { "✓ " } else { "" };
                                button.set_text(cx, &format!("{mark}{}{price}", modifier.name));
                                button.set_visible(cx, true);
                            }
                            None => button.set_visible(cx, false),
                        }
                    }
                    item.draw_all_unscoped(cx);
                }
                if count > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / count as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, cart_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct SaleList {
    #[deref]
    view: View,
}

impl Widget for SaleList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (sales, currency, fiscal) = {
            let st = app_state().lock().unwrap();
            (st.recent_sales.clone(), st.locale.currency.clone(), st.fiscal.clone())
        };
        let walk = Walk {
            height: Size::Fixed(list_content_height(sales.len(), 132.0, 0.0)),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, sales.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(sale) = sales.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let text = if sale.rounding_paise != 0 {
                        format!(
                            "{} · Rounded {} (Applied {} · round {}) · {} · {} · by {}",
                            &sale.created_at[..sale.created_at.len().min(19)],
                            Money(sale.total_paise).format_localized(&currency, &fiscal),
                            Money(sale.lines_total_paise).format_localized(&currency, &fiscal),
                            Money(sale.rounding_paise).format_localized(&currency, &fiscal),
                            sale.pay_method,
                            sale.status,
                            if sale.created_by_name.is_empty() {
                                "—"
                            } else {
                                &sale.created_by_name
                            }
                        )
                    } else {
                        format!(
                            "{} · Rounded {} · {} · {} · by {}",
                            &sale.created_at[..sale.created_at.len().min(19)],
                            Money(sale.total_paise).format_localized(&currency, &fiscal),
                            sale.pay_method,
                            sale.status,
                            if sale.created_by_name.is_empty() {
                                "—"
                            } else {
                                &sale.created_by_name
                            }
                        )
                    };
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&text));
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct CustomerList {
    #[deref]
    view: View,
}

impl Widget for CustomerList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (customers, currency, fiscal) = {
            let st = app_state().lock().unwrap();
            (st.customers.clone(), st.locale.currency.clone(), st.fiscal.clone())
        };
        let walk = Walk {
            height: Size::Fixed(list_content_height(customers.len(), 92.0, 0.0).max(48.0)),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, customers.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(c) = customers.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let text = if c.balance_paise > 0 {
                        format!(
                            "{} · owes {}",
                            c.name,
                            Money(c.balance_paise).format_localized(&currency, &fiscal)
                        )
                    } else if c.balance_paise < 0 {
                        format!(
                            "{} · advance {}",
                            c.name,
                            Money(-c.balance_paise).format_localized(&currency, &fiscal)
                        )
                    } else {
                        format!("{} · clear", c.name)
                    };
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&text));
                    item.button(cx, ids!(pick)).set_text(cx, "Pick");
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct LanguageList {
    #[deref]
    view: View,
}

impl Widget for LanguageList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let selected = app_state().lock().unwrap().locale.language.clone();
        // Cap visible height so Settings still scrolls the page; list scrolls inside.
        let walk = Walk {
            height: Size::Fixed(320.0),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, LANGUAGES.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(lang) = LANGUAGES.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let mark = if lang.code == selected { "✓ " } else { "" };
                    item.button(cx, ids!(pick)).set_text(
                        cx,
                        &format!(
                            "{mark}{} · {} · {}",
                            lang.native_name, lang.name_en, lang.region
                        ),
                    );
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Pointer over this capped list = inner owns the wheel; page must not move.
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct ServerList {
    #[deref]
    view: View,
}

impl Widget for ServerList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (servers, open_label) = {
            let st = app_state().lock().unwrap();
            (st.local_servers.clone(), st.t("open_server"))
        };
        let walk = Walk {
            height: Size::Fixed(list_content_height(servers.len().max(1), 56.0, 0.0).min(280.0)),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, servers.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(server) = servers.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let short = server.id.chars().take(8).collect::<String>();
                    item.widget(cx, ids!(label)).set_text(
                        cx,
                        &plain_display_text(&format!("{} · {short}", server.name)),
                    );
                    item.button(cx, ids!(pick)).set_text(cx, &open_label);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct CurrencyList {
    #[deref]
    view: View,
}

impl Widget for CurrencyList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let selected = app_state().lock().unwrap().locale.currency.clone();
        let use_label = {
            let st = app_state().lock().unwrap();
            st.t("use")
        };
        let walk = Walk {
            height: Size::Fixed(280.0),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, CURRENCIES.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(cur) = CURRENCIES.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let mark = if cur.code == selected { "✓ " } else { "" };
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&format!("{mark}{} {} — {}", cur.symbol, cur.code, cur.name_en),));
                    item.button(cx, ids!(pick)).set_text(cx, &use_label);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct TableList {
    #[deref]
    view: View,
}

impl Widget for TableList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let tables = app_state().lock().unwrap().tables.clone();
        let walk = Walk {
            height: Size::Fixed(list_content_height(tables.len(), 162.0, 0.0)),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, tables.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(table) = tables.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.widget(cx, ids!(name))
                        .set_text(cx, &plain_display_text(&table.name));
                    item.widget(cx, ids!(status))
                        .set_text(cx, &plain_display_text(table.status.as_str()));
                    let action = if table.status == domain::TableStatus::Available {
                        "Seat / use"
                    } else {
                        "Select occupied"
                    };
                    item.button(cx, ids!(action)).set_text(cx, action);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct KotList {
    #[deref]
    view: View,
}

impl Widget for KotList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let kots = app_state().lock().unwrap().kots.clone();
        let walk = Walk {
            height: Size::Fixed(list_content_height(kots.len(), 172.0, 0.0)),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, kots.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(kot) = kots.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let text = format!("[{}] {}", kot.status.as_str(), kot.summary);
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&text));
                    item.button(cx, ids!(advance)).set_text(cx, "Advance");
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Selectable display text (Markdown) with Label-like `visible` support.
/// Stock Markdown/TextFlow ignore `set_visible`, which left empty-state copy
/// on screen after products/cart lines appeared.
#[derive(Script, ScriptHook, Widget)]
struct WrapLabel {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    markdown: Markdown,
    #[live(true)]
    #[visible]
    visible: bool,
}

impl Widget for WrapLabel {
    fn is_interactive(&self) -> bool {
        self.visible && self.markdown.text_flow.selectable
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.visible && event.requires_visibility() {
            return;
        }
        self.markdown.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        // Keep scroll blocked across redraws while a drag-select is live.
        if self.markdown.text_flow.is_drag_selecting() {
            cx.block_scrolling_except_within(Area::Empty);
        }
        self.markdown.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.markdown.text()
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        self.markdown.set_text(cx, v);
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

/// Escape Markdown markers so product/customer strings never italicize or become headings.
fn plain_display_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '*' | '_' | '`' | '#' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

impl App {
    fn set_status(&mut self, cx: &mut Cx, msg: impl Into<String>) {
        let msg = msg.into();
        {
            let mut st = app_state().lock().unwrap();
            st.status = msg.clone();
            // Sell has no footer strip — put pay/cart errors on the receipt line
            // so Split/Credit failures are visible on the counter screen.
            if matches!(
                st.screen,
                Screen::Pos | Screen::Credit | Screen::Sales | Screen::Menu
            ) {
                st.last_receipt = msg.clone();
            }
        }
        // Also surface on lock / device-setup labels.
        self.ui.widget(cx, ids!(pin_status)).set_text(cx, &plain_display_text(&msg));
        self.ui
            .widget(cx, ids!(device_setup_status)).set_text(cx, &plain_display_text(&msg));
        self.ui
            .widget(cx, ids!(receipt_label))
            .set_text(cx, &plain_display_text(&msg));
    }

    /// Open SQLite using Makepad's platform data dir (Android getFilesDir).
    /// `dirs::data_dir` is often None on Android (no $HOME), which left
    /// `store = None` so Owner PIN OK could never unlock.
    fn ensure_store(&mut self, cx: &mut Cx) {
        {
            let st = app_state().lock().unwrap();
            if st.store.is_some() {
                return;
            }
        }
        self.resolve_data_base(cx);
        let opened = if let Some(dir) = cx.get_data_dir() {
            let base = PathBuf::from(dir);
            app_state().lock().unwrap().data_base = Some(base.clone());
            local_servers::open_active_server(&base)
        } else {
            Store::open_default()
        };
        let mut st = app_state().lock().unwrap();
        let _ = st.refresh_local_servers();
        match opened {
            Ok(store) => {
                st.status = format!("Local DB: {}", store.db_path().display());
                st.store = Some(store);
                let _ = st.reload_all();
                if let Some(store) = st.store.as_mut() {
                    let _ = store.set_active_session_staff_id(None);
                }
                st.session = None;
                st.pin_buffer.clear();
                st.route_after_stored_role();
                log!(
                    "Kumkaj store ready at {}",
                    st.store
                        .as_ref()
                        .map(|s| s.db_path().display().to_string())
                        .unwrap_or_default()
                );
            }
            Err(e) => {
                if e.contains("No local server") {
                    st.must_confirm_device_role = true;
                    st.apply_platform_language_preview();
                    st.screen = Screen::DeviceSetup;
                    log!("Kumkaj: no local server yet — DeviceSetup");
                } else {
                    st.status = format!("Storage error: {e}");
                    log!("Kumkaj store open failed: {e}");
                }
            }
        }
    }

    fn sync_chrome(&mut self, cx: &mut Cx) {
        // After DSL hot-reload, visibility resets to defaults. If a session is
        // still active, never leave the UI stuck on the lock screen.
        {
            let mut st = app_state().lock().unwrap();
            if st.must_confirm_device_role {
                st.screen = Screen::DeviceSetup;
            } else if st.session.is_some() && st.screen == Screen::Lock {
                st.screen = unlocked_home();
            }
            // Follow-OS mode: keep UI language in sync with the device.
            if st.language_follow_platform {
                st.locale.language = locale::detect_platform_language();
            }
        }
        // Keep Books overview fresh when that tab is showing.
        {
            let mut st = app_state().lock().unwrap();
            if st.screen == Screen::Books && st.session.is_some() {
                let _ = st.refresh_books_overview();
            }
        }
        let (
            today,
            session,
            status,
            receipt,
            pin,
            db,
            screen,
            table_chip,
            staff_text,
            audit,
            locale_line,
            empty_msg,
            product_count,
            cart_count,
            total_input,
            round_hint,
            natural_label,
            applied_label,
            charge_caption,
            session_chip,
            lock_prompt,
            pin_hint,
            pin_count,
            pin_reveal_label,
            lock_staff_label,
            lock_as_owner,
            show_staff_role,
            undo_visible,
            undo_snack_text,
            bill_round_active,
        ) = {
            let mut st = app_state().lock().unwrap();
            let session = match &st.session {
                Some(s) => s.lock_label(),
                None => "Locked".into(),
            };
            let order_type_name = st.cart.order_type.label();
            let table_chip = match &st.selected_table_id {
                Some(id) => st
                    .tables
                    .iter()
                    .find(|t| &t.id == id)
                    .map(|t| format!("Table {} · Dine", t.name))
                    .unwrap_or_else(|| "Table selected · Dine".into()),
                None => {
                    if st.venue_mode == VenueMode::Restaurant {
                        format!("{} · {order_type_name}", st.t("counter_sale"))
                    } else {
                        st.t("ready_to_sell")
                    }
                }
            };
            let staff_text = st
                .staff
                .iter()
                .map(|s| s.lock_label())
                .collect::<Vec<_>>()
                .join("\n");
            let audit = st
                .store
                .as_ref()
                .and_then(|s| s.list_audit(5).ok())
                .map(|events| {
                    if events.is_empty() {
                        "Audit: (empty)".into()
                    } else {
                        format!(
                            "Audit: {}",
                            events
                                .iter()
                                .map(|e| format!("{} {}", e.action, e.detail))
                                .collect::<Vec<_>>()
                                .join(" · ")
                        )
                    }
                })
                .unwrap_or_else(|| "Audit: —".into());
            let lang = locale::language_by_code(&st.locale.language)
                .map(|l| format!("{} ({})", l.native_name, l.name_en))
                .unwrap_or_else(|| st.locale.language.clone());
            let cur = locale::currency_by_code(&st.locale.currency)
                .map(|c| format!("{} {} — {}", c.symbol, c.code, c.name_en))
                .unwrap_or_else(|| st.locale.currency.clone());
            let locale_line = format!(
                "{}: {}\n{}: {}\n{} languages · {} currencies",
                st.t("language"),
                lang,
                st.t("currency"),
                cur,
                LANGUAGES.len(),
                CURRENCIES.len()
            );
            let empty_msg = if st.products.is_empty() {
                "No items yet. Open Items to add your rates or products.".to_string()
            } else {
                st.t("empty_catalog")
            };
            let natural_label = st.money(st.cart.actual_total());
            let applied_label = st.money(st.cart.applied_total());
            let total_input =
                locale::format_major_input_ex(
                    st.cart.rounded_total().paise(),
                    &st.locale.currency,
                    st.decimal_sep(),
                );
            let round_hint = if st.cart.rounding().paise() != 0 {
                format!(
                    "Bill round {} · Rounded {}",
                    st.money(st.cart.rounding()),
                    st.money(st.cart.rounded_total())
                )
            } else if st.cart.applied_total().paise() != st.cart.actual_total().paise() {
                format!(
                    "Items @×qty {} · Applied {}. Rounded starts equal to Applied.",
                    st.money(st.cart.actual_total()),
                    st.money(st.cart.applied_total())
                )
            } else {
                "Rounded starts equal to Applied. Edit only to round the bill.".into()
            };
            let charge_caption = if st.cart.rounding().paise() != 0 {
                format!("ROUNDED ({})", st.money(st.cart.rounding()))
            } else {
                "ROUNDED".to_string()
            };
            // Variable 4–12 length: show only entered digits (masked or plain) + a count.
            let pin_n = st.pin_buffer.len().min(12);
            let pin_display = if pin_n == 0 {
                "—".to_string()
            } else if st.pin_visible {
                st.pin_buffer
                    .chars()
                    .take(12)
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                std::iter::repeat('●')
                    .take(pin_n)
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(if pin_n > 6 { " " } else { "  " })
            };
            let pin_count = if pin_n == 0 {
                "0 / 12 digits · at least 4, then OK".to_string()
            } else if pin_n < 4 {
                format!("{pin_n} / 12 digits · need {} more", 4 - pin_n)
            } else if pin_n >= 12 {
                "12 / 12 digits · press OK".to_string()
            } else {
                format!("{pin_n} / 12 digits · press OK when ready")
            };
            let pin_reveal_label = if st.pin_visible {
                "Hide PIN"
            } else {
                "Show PIN"
            };
            let session_chip = match &st.session {
                Some(s) => s.lock_label(),
                None => "Locked".into(),
            };
            let has_staff = st.staff.iter().any(|s| !s.is_owner());
            let owner_setup_complete = st.owner_setup_complete;
            let pin_reset_required = st.pin_reset_required;
            let lock_prompt = if !owner_setup_complete || pin_reset_required {
                "SET OWNER PIN".to_string()
            } else if st.lock_as_owner {
                "ENTER OWNER PASSCODE".to_string()
            } else {
                "ENTER STAFF PASSCODE".to_string()
            };
            let pin_hint = if !owner_setup_complete {
                "Choose 4–12 digits (keyboard or pad) · Enter/OK to save".to_string()
            } else if pin_reset_required {
                "New 4–12 digit Owner PIN (keyboard or pad) · Enter/OK".to_string()
            } else if st.device_role == DeviceRole::Server {
                if st.extra_tills_allowed {
                    let code = st
                        .store
                        .as_ref()
                        .and_then(|s| s.pairing_code().ok())
                        .unwrap_or_default();
                    format!("Shop device · pairing code {code}")
                } else {
                    "This device holds the books".to_string()
                }
            } else if st.device_role == DeviceRole::Client {
                if st.client_paired {
                    "Extra till paired — unlock to sell".to_string()
                } else {
                    "Extra till — pair to the shop device first".to_string()
                }
            } else {
                "Unlock to continue".to_string()
            };
            let lock_staff_label = if st.lock_as_owner {
                String::new()
            } else {
                st.staff
                    .iter()
                    .filter(|s| !s.is_owner())
                    .find(|s| Some(&s.id) == st.lock_staff_id.as_ref())
                    .map(|s| s.lock_label())
                    .or_else(|| {
                        st.staff
                            .iter()
                            .find(|s| !s.is_owner())
                            .map(|s| s.lock_label())
                    })
                    .unwrap_or_else(|| {
                        "No staff yet — unlock as Owner, then Team".into()
                    })
            };
            // Until Owner finishes setup, and until staff exist, only Owner
            // unlock. A pending PIN reset is the Owner's job too.
            let lock_as_owner = if !owner_setup_complete || !has_staff || pin_reset_required {
                st.lock_as_owner = true;
                true
            } else {
                st.lock_as_owner
            };
            let show_staff_role = owner_setup_complete && has_staff && !pin_reset_required;
            let undo_visible = st.undo_sale_id.is_some()
                && matches!(
                    st.screen,
                    Screen::Books
                        | Screen::Pos
                        | Screen::Buy
                        | Screen::Menu
                        | Screen::More
                        | Screen::Floor
                        | Screen::Kitchen
                        | Screen::Sales
                        | Screen::Credit
                        | Screen::Team
                        | Screen::Settings
                        | Screen::Inventory
                        | Screen::Branches
                );
            let undo_snack_text = st.undo_snack_text.clone();
            let bill_round_active = st.cart.charge_total.is_some();
            (
                st.today_label(),
                session,
                st.status.clone(),
                st.last_receipt.clone(),
                pin_display,
                st.store
                    .as_ref()
                    .map(|s| s.db_path().display().to_string())
                    .unwrap_or_else(|| "no db".into()),
                st.screen,
                table_chip,
                staff_text,
                audit,
                locale_line,
                empty_msg,
                st.products.len(),
                st.cart.lines.len(),
                total_input,
                round_hint,
                natural_label,
                applied_label,
                charge_caption,
                session_chip,
                lock_prompt,
                pin_hint,
                pin_count,
                pin_reveal_label,
                lock_staff_label,
                lock_as_owner,
                show_staff_role,
                undo_visible,
                undo_snack_text,
                bill_round_active,
            )
        };

        self.ui.widget(cx, ids!(today_label)).set_text(cx, &plain_display_text(&today));
        self.ui
            .widget(cx, ids!(session_label)).set_text(cx, &plain_display_text(&session));
        self.ui
            .widget(cx, ids!(receipt_label)).set_text(cx, &plain_display_text(&receipt));
        self.ui.label(cx, ids!(pin_label)).set_text(cx, &pin);
        self.ui
            .widget(cx, ids!(pin_prompt)).set_text(cx, &plain_display_text(&lock_prompt));
        self.ui
            .widget(cx, ids!(pin_count))
            .set_text(cx, &plain_display_text(&pin_count));
        self.ui
            .button(cx, ids!(pin_reveal))
            .set_text(cx, pin_reveal_label);
        self.ui.widget(cx, ids!(pin_hint)).set_text(cx, &plain_display_text(&pin_hint));
        self.ui
            .button(cx, ids!(role_staff))
            .set_visible(cx, show_staff_role);
        // On the lock screen, surface status (wrong PIN / no store) under the dots.
        let pin_status_text = if screen == Screen::Lock {
            if status.is_empty() || status.starts_with("Local DB:") {
                " ".to_string()
            } else {
                status.clone()
            }
        } else {
            " ".to_string()
        };
        let setup_status_text = if screen == Screen::DeviceSetup {
            if status.is_empty() || status.starts_with("Local DB:") {
                " ".to_string()
            } else {
                status.clone()
            }
        } else {
            " ".to_string()
        };
        self.ui
            .widget(cx, ids!(pin_status)).set_text(cx, &plain_display_text(&pin_status_text));
        self.ui
            .widget(cx, ids!(device_setup_status)).set_text(cx, &plain_display_text(&setup_status_text));
        self.ui
            .widget(cx, ids!(lock_staff_label)).set_text(cx, &plain_display_text(&lock_staff_label));
        self.ui
            .view(cx, ids!(lock_staff_row))
            .set_visible(cx, !lock_as_owner);
        self.ui.widget(cx, ids!(db_path_label)).set_text(cx, &plain_display_text(&db));
        self.ui.widget(cx, ids!(audit_label)).set_text(cx, &plain_display_text(&audit));
        self.ui
            .widget(cx, ids!(table_chip)).set_text(cx, &plain_display_text(&table_chip));
        self.ui
            .widget(cx, ids!(session_chip)).set_text(cx, &plain_display_text(&session_chip));
        self.ui
            .widget(cx, ids!(staff_label)).set_text(cx, &plain_display_text(&staff_text));
        self.ui
            .widget(cx, ids!(locale_label)).set_text(cx, &plain_display_text(&locale_line));

        // Country pack: the summary, the pay-rail wording on the Sell buttons,
        // and the exclusive-tax estimate under the totals strip.
        {
            let (fiscal_line, country_hint, tax_line, digital, business_id, service_pct) = {
                let st = app_state().lock().unwrap();
                (
                    st.fiscal.summary(),
                    match fiscal::country_by_code(&st.country_browse) {
                        Some(p) => format!(
                            "Browsing {} — tap Apply country pack to switch\n{}",
                            p.code,
                            p.apply_hint()
                        ),
                        None => format!(
                            "{}\n{} country packs · type a code or tap ‹ Browse ›",
                            st.fiscal.profile().apply_hint(),
                            fiscal::countries().len()
                        ),
                    },
                    st.cart_tax_estimate_line(),
                    st.fiscal.digital_pay_label().to_string(),
                    st.fiscal.business_tax_id.clone(),
                    locale::format_percent_bps(
                        st.fiscal.service_charge_bps,
                        st.fiscal.decimal_sep(),
                    ),
                )
            };
            self.ui.widget(cx, ids!(fiscal_label)).set_text(cx, &plain_display_text(&fiscal_line));
            self.ui
                .widget(cx, ids!(country_hint)).set_text(cx, &plain_display_text(&country_hint));
            self.ui
                .widget(cx, ids!(tax_estimate_label)).set_text(cx, &plain_display_text(&tax_line));
            self.ui
                .widget(cx, ids!(tax_estimate_label))
                .set_visible(cx, tax_line.trim() != "");
            self.ui.button(cx, ids!(amend_pay_upi)).set_text(cx, &digital);
            self.ui
                .button(cx, ids!(settle_upi))
                .set_text(cx, &format!("Pay {digital}"));
            for (path, value) in [
                (ids!(business_tax_id_input), business_id),
                (ids!(service_bps_input), service_pct),
            ] {
                let field = self.ui.text_input(cx, path);
                if !cx.has_key_focus(field.area()) && field.text() != value {
                    field.set_text(cx, &value);
                }
            }
        }
        self.ui
            .widget(cx, ids!(empty_catalog)).set_text(cx, &plain_display_text(&empty_msg));
        self.ui
            .widget(cx, ids!(menu_empty)).set_text(cx, &plain_display_text(&empty_msg));
        self.ui
            .widget(cx, ids!(empty_catalog))
            .set_visible(cx, product_count == 0);
        self.ui
            .widget(cx, ids!(menu_empty))
            .set_visible(cx, product_count == 0);
        self.ui
            .widget(cx, ids!(product_list))
            .set_visible(cx, product_count > 0);
        self.ui
            .widget(cx, ids!(product_list_menu))
            .set_visible(cx, product_count > 0);
        self.ui
            .widget(cx, ids!(cart_empty_hint))
            .set_visible(cx, cart_count == 0);
        self.ui
            .widget(cx, ids!(cart_list))
            .set_visible(cx, cart_count > 0);
        self.ui
            .widget(cx, ids!(cart_natural_label)).set_text(cx, &plain_display_text(&natural_label));
        self.ui
            .widget(cx, ids!(cart_applied_label)).set_text(cx, &plain_display_text(&applied_label));
        self.ui
            .widget(cx, ids!(cart_charge_caption)).set_text(cx, &plain_display_text(&charge_caption));
        self.ui
            .widget(cx, ids!(cart_round_hint)).set_text(cx, &plain_display_text(&round_hint));
        {
            let total_field = self.ui.text_input(cx, ids!(cart_total_input));
            let shown = total_field.text();
            if shown != total_input {
                if bill_round_active {
                    // Seller is editing an explicit bill round — don't clobber mid-type.
                    if !cx.has_key_focus(total_field.area()) {
                        total_field.set_text(cx, &total_input);
                    }
                } else {
                    // Default Rounded = Applied. Always mirror the model so a
                    // stale field from the previous line count cannot stick
                    // (especially when the input still holds key focus).
                    if cx.has_key_focus(total_field.area()) {
                        total_field.take_key_focus(cx);
                    }
                    total_field.set_text(cx, &total_input);
                }
            }
        }
        self.ui
            .view(cx, ids!(undo_snackbar))
            .set_visible(cx, undo_visible);
        self.ui
            .widget(cx, ids!(undo_snack_label)).set_text(cx, &plain_display_text(&undo_snack_text));

        let (credit_chip, credit_summary, empty_hint) = {
            let st = app_state().lock().unwrap();
            let credit_chip = match &st.selected_customer_id {
                Some(id) => st
                    .customers
                    .iter()
                    .find(|c| &c.id == id)
                    .map(|c| {
                        if c.balance_paise > 0 {
                            format!("{} · owes {}", c.name, st.money(c.balance()))
                        } else if c.balance_paise < 0 {
                            format!(
                                "{} · advance {}",
                                c.name,
                                st.money(Money(-c.balance_paise))
                            )
                        } else {
                            format!("{} · clear — Credit or take advance", c.name)
                        }
                    })
                    .unwrap_or_else(|| "Selected customer missing — pick again".into()),
                None => {
                    // A Credit tap with no name used to land only on the lock
                    // screen's status line, so the button looked dead.
                    if st.status.starts_with("Type a customer")
                        || st.status.starts_with("Pick a customer")
                        || st.status.starts_with("Customer name")
                    {
                        st.status.clone()
                    } else {
                        "No customer yet — type a name, then tap Credit".into()
                    }
                }
            };
            let open: i64 = st.customers.iter().map(|c| c.balance_paise.max(0)).sum();
            let prepaid: i64 = st
                .customers
                .iter()
                .map(|c| (-c.balance_paise).max(0))
                .sum();
            let mut credit_summary = format!(
                "{} customers · open credit {} · advances held {}",
                st.customers.len(),
                st.money(Money(open)),
                st.money(Money(prepaid))
            );
            if let Some(id) = &st.selected_customer_id
                && let Some(store) = st.store.as_ref()
                    && let Ok(entries) = store.list_credit_ledger(id, 6)
                        && !entries.is_empty() {
                            credit_summary.push_str("\nRecent:");
                            for entry in entries {
                                credit_summary.push_str(&format!(
                                    "\n  {} {} {}",
                                    &entry.created_at[..entry.created_at.len().min(10)],
                                    entry.kind,
                                    st.money(Money(entry.amount_paise))
                                ));
                            }
                        }
            let empty_hint = if st.customers.is_empty() {
                "No saved customers yet — type a name above."
            } else {
                "Tap Pick to use a saved customer."
            };
            (credit_chip, credit_summary, empty_hint)
        };
        self.ui
            .widget(cx, ids!(credit_chip)).set_text(cx, &plain_display_text(&credit_chip));
        self.ui
            .widget(cx, ids!(credit_summary_label)).set_text(cx, &plain_display_text(&credit_summary));
        self.ui
            .widget(cx, ids!(customers_empty_hint)).set_text(cx, &plain_display_text(empty_hint));

        let (sale_selected, sale_history, amend_total_text, amend_natural, amend_applied, amend_round) = {
            let st = app_state().lock().unwrap();
            let sale_selected = if let Some(id) = &st.selected_sale_id {
                let short = &id[..8.min(id.len())];
                let lines = st
                    .amend_cart
                    .lines
                    .iter()
                    .map(|l| {
                        if l.note.is_empty() {
                            format!("{}×{} @{}", l.qty, l.name, st.money(l.unit_price))
                        } else {
                            format!(
                                "{}×{} @{} ({})",
                                l.qty,
                                l.name,
                                st.money(l.unit_price),
                                l.note
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "Editing {short} · {} · pay {} · Actual {} · Applied {} · Rounded {}",
                    if lines.is_empty() { "—".into() } else { lines },
                    st.amend_pay.label_for(&st.fiscal),
                    st.money(st.amend_cart.actual_total()),
                    st.money(st.amend_cart.applied_total()),
                    st.money(st.amend_cart.rounded_total())
                )
            } else {
                "No sale selected".into()
            };
            let sale_history = if st.sale_revisions.is_empty() {
                "History: (none yet)".into()
            } else {
                let parts: Vec<String> = st
                    .sale_revisions
                    .iter()
                    .take(5)
                    .map(|r| {
                        format!(
                            "{} {} · was {} · {}",
                            &r.revised_at[..r.revised_at.len().min(19)],
                            r.actor_name,
                            st.money(Money(r.previous.total_paise)),
                            r.reason
                        )
                    })
                    .collect();
                format!("History: {}", parts.join(" | "))
            };
            let amend_total_text = locale::format_major_input_ex(
                st.amend_cart.rounded_total().paise(),
                &st.locale.currency,
                st.decimal_sep(),
            );
            let amend_natural = format!(
                "Actual Total {}",
                st.money(st.amend_cart.actual_total())
            );
            let amend_applied = format!(
                "Applied Total {}",
                st.money(st.amend_cart.applied_total())
            );
            let amend_round = if st.amend_cart.is_empty() {
                "Rounded Total starts equal to Applied Total.".into()
            } else if st.amend_cart.rounding().paise() != 0 {
                format!(
                    "Rounding {} · Rounded Total {}",
                    st.money(st.amend_cart.rounding()),
                    st.money(st.amend_cart.rounded_total())
                )
            } else {
                "Rounded Total equals Applied Total.".into()
            };
            (
                sale_selected,
                sale_history,
                amend_total_text,
                amend_natural,
                amend_applied,
                amend_round,
            )
        };
        {
            let may_void = {
                let st = app_state().lock().unwrap();
                st.session
                    .as_ref()
                    .map(|s| s.role.can_void_refund())
                    .unwrap_or(false)
            };
            self.ui
                .text_input(cx, ids!(approval_pin_input))
                .set_visible(cx, !may_void);
            self.ui.widget(cx, ids!(approval_hint)).set_text(
                cx,
                &plain_display_text(
                    "Void and refund need an Owner or Manager. Type their PIN above and the approval is recorded with your name.",
                ),
            );
            self.ui
                .widget(cx, ids!(approval_hint))
                .set_visible(cx, !may_void);
        }
        self.ui
            .widget(cx, ids!(sale_selected_label)).set_text(cx, &plain_display_text(&sale_selected));
        self.ui
            .widget(cx, ids!(sale_history_label)).set_text(cx, &plain_display_text(&sale_history));
        self.ui
            .widget(cx, ids!(amend_natural_label)).set_text(cx, &plain_display_text(&amend_natural));
        self.ui
            .widget(cx, ids!(amend_applied_label)).set_text(cx, &plain_display_text(&amend_applied));
        self.ui
            .widget(cx, ids!(amend_round_hint)).set_text(cx, &plain_display_text(&amend_round));
        {
            let field = self.ui.text_input(cx, ids!(amend_total_input));
            if !cx.has_key_focus(field.area()) {
                let shown = field.text();
                if shown != amend_total_text {
                    field.set_text(cx, &amend_total_text);
                }
            }
        }

        let image_hint = {
            let st = app_state().lock().unwrap();
            let name = self.ui.text_input(cx, ids!(menu_name_input)).text();
            menu_images::describe_image(st.draft_image_key.as_deref(), &name)
        };
        self.ui
            .widget(cx, ids!(image_hint)).set_text(cx, &plain_display_text(&image_hint));
        {
            let (draft_key, name) = {
                let st = app_state().lock().unwrap();
                (
                    st.draft_image_key.clone(),
                    self.ui.text_input(cx, ids!(menu_name_input)).text(),
                )
            };
            let dir = item_images_dir();
            let path = match draft_key.as_deref() {
                Some(key) if key.starts_with("pending_custom:") => {
                    std::path::PathBuf::from(&key["pending_custom:".len()..])
                }
                other => menu_images::resolve_image_path(other, &name, &dir),
            };
            load_thumb_at(cx, &self.ui.image(cx, ids!(draft_preview)), &path);
        }

        self.sync_venue_brand(cx);

        self.show_screen(cx, screen);
        {
            let st = app_state().lock().unwrap();
            let mode_hint = match st.venue_mode {
                VenueMode::Counter => "",
                VenueMode::Restaurant => {
                    "Tables and kitchen are on. Sales can create kitchen tickets."
                }
            };
            let floor_hint = match st.venue_mode {
                VenueMode::Counter => "Tables are off. Turn on Tables & kitchen in Settings first.",
                VenueMode::Restaurant => "Tap a table to attach the next sale, or Counter / takeaway.",
            };
            let kitchen_hint = match st.venue_mode {
                VenueMode::Counter => "No kitchen tickets. Turn on Tables & kitchen in Settings.",
                VenueMode::Restaurant => "Open kitchen tickets from sales.",
            };
            let pair_label = if st.device_role == DeviceRole::Server {
                if st.extra_tills_allowed {
                    let code = st
                        .store
                        .as_ref()
                        .and_then(|s| s.pairing_code().ok())
                        .unwrap_or_else(|| "------".into());
                    let listening = if net::server_is_running() {
                        "listening"
                    } else {
                        "not listening"
                    };
                    format!(
                        "Extra tills can join ({listening}). Pairing code: {code}. They connect to this device IP on port {}.",
                        net::DEFAULT_PORT
                    )
                } else {
                    "Extra tills are off. This shop's books stay on this device — the same books a future cloud sync will use. Allow extra tills if another phone should sell too.".into()
                }
            } else if st.device_role == DeviceRole::Client {
                if st.client_paired {
                    let url = st
                        .store
                        .as_ref()
                        .and_then(|s| s.server_url().ok())
                        .flatten()
                        .unwrap_or_default();
                    format!("Paired to {url}")
                } else {
                    "Not paired — connect to the shop device.".into()
                }
            } else {
                "Choose how this device is used.".into()
            };
            let mode_label = match st.venue_mode {
                VenueMode::Counter => "Off — no tables or kitchen tickets",
                VenueMode::Restaurant => "Tables & kitchen on",
            };
            let day_close = st.day_close_text.clone();
            let inventory_text = st.ops_panel_text.clone();
            let branches_body = st.branches_text.clone();
            let order_type_line = format!("Order: {}", st.cart.order_type.label());
            let station_line = match st.selected_station_id.as_deref() {
                Some(id) => {
                    let name = st
                        .store
                        .as_ref()
                        .and_then(|s| s.list_stations().ok())
                        .and_then(|list| list.into_iter().find(|s| s.id == id))
                        .map(|s| s.name)
                        .unwrap_or_else(|| "selected".into());
                    format!("Station: {name}")
                }
                None => "Station: all".to_string(),
            };
            let org_name = st
                .store
                .as_ref()
                .and_then(|s| s.org_name().ok())
                .unwrap_or_default();
            let tmpl = entity_template::resolve(&st.entity_template_id);
            let tmpl_line = format!("Current: {} — {}", tmpl.title, tmpl.blurb);
            let onboard_status = if st.onboarding_complete {
                "Setup complete".to_string()
            } else {
                match st.onboard_step {
                    1 => format!(
                        "Fresh · {} — catalogue starts empty; add items in Items, then Done.",
                        tmpl.title
                    ),
                    2 => format!(
                        "Book · {} — add items and opening credit, then Done.",
                        tmpl.title
                    ),
                    _ => format!(
                        "Template: {} — {}. Name the place if you want, then Start fresh or Bring my book.",
                        tmpl.title, tmpl.blurb
                    ),
                }
            };
            let cust_ph = st.t("customer_name_ph");
            let new_cust_ph = st.t("new_customer_ph");
            drop(st);
            self.ui
                .widget(cx, ids!(more_mode_hint)).set_text(cx, &plain_display_text(mode_hint));
            self.ui
                .widget(cx, ids!(more_mode_hint))
                .set_visible(cx, !mode_hint.is_empty());
            self.ui
                .widget(cx, ids!(floor_mode_hint)).set_text(cx, &plain_display_text(floor_hint));
            self.ui
                .widget(cx, ids!(kitchen_mode_hint)).set_text(cx, &plain_display_text(kitchen_hint));
            self.ui
                .widget(cx, ids!(server_pair_label)).set_text(cx, &plain_display_text(&pair_label));
            self.ui
                .widget(cx, ids!(venue_mode_label)).set_text(cx, &plain_display_text(mode_label));
            self.ui
                .widget(cx, ids!(settings_tmpl_label))
                .set_text(cx, &plain_display_text(&tmpl_line));
            let units_body = {
                let st = app_state().lock().unwrap();
                match st.store.as_ref() {
                    Some(store) => {
                        let units = store.list_units().unwrap_or_default();
                        let convs = store.list_unit_conversions().unwrap_or_default();
                        let mut lines = Vec::new();
                        lines.push("Units:".into());
                        for u in &units {
                            lines.push(format!(
                                "  {} — {}{}",
                                u.code,
                                u.name,
                                if u.builtin { " (built-in)" } else { "" }
                            ));
                        }
                        if !convs.is_empty() {
                            lines.push("Conversions (1 from = factor to):".into());
                            for c in &convs {
                                let factor = c.factor_milli as f64 / 1000.0;
                                lines.push(format!(
                                    "  1 {} = {} {}",
                                    c.from_code, factor, c.to_code
                                ));
                            }
                        }
                        lines.join("\n")
                    }
                    None => String::new(),
                }
            };
            self.ui
                .widget(cx, ids!(units_label))
                .set_text(cx, &plain_display_text(&units_body));
            self.ui
                .widget(cx, ids!(onboard_tmpl_hint))
                .set_text(
                    cx,
                    &plain_display_text(&format!(
                        "Selected: {} — {}",
                        tmpl.title, tmpl.blurb
                    )),
                );
            self.ui
                .text_input(cx, ids!(customer_name_input))
                .set_empty_text(cx, cust_ph.clone());
            self.ui
                .text_input(cx, ids!(onboard_cust_name))
                .set_empty_text(cx, cust_ph);
            self.ui
                .text_input(cx, ids!(credit_add_name))
                .set_empty_text(cx, new_cust_ph);
            self.ui
                .widget(cx, ids!(day_close_label)).set_text(cx, &plain_display_text(&day_close));
            self.ui
                .widget(cx, ids!(books_label)).set_text(cx, &plain_display_text(&day_close));
            self.ui
                .widget(cx, ids!(reports_label)).set_text(cx, &plain_display_text(&day_close));
            self.ui
                .widget(cx, ids!(inventory_label)).set_text(cx, &plain_display_text(&inventory_text));
            self.ui
                .widget(cx, ids!(branches_label)).set_text(cx, &plain_display_text(&branches_body));
            self.ui
                .widget(cx, ids!(order_type_label)).set_text(cx, &plain_display_text(&order_type_line));
            self.ui
                .widget(cx, ids!(kitchen_station_label)).set_text(cx, &plain_display_text(&station_line));
            {
                let field = self.ui.text_input(cx, ids!(org_name_input));
                if !cx.has_key_focus(field.area()) && field.text() != org_name {
                    field.set_text(cx, &org_name);
                }
            }
            self.ui
                .widget(cx, ids!(onboard_status)).set_text(cx, &plain_display_text(&onboard_status));
            self.ui
                .widget(cx, ids!(client_pair_status)).set_text(cx, &plain_display_text(if app_state().lock().unwrap().client_paired {
                        "Paired. Unlock to sell, or Refresh to pull the catalogue."
                    } else {
                        "Enter the shop device IP and 6-digit pairing code. Selling is blocked until paired."
                    },));
        }
        
        // Role-aware More rows / tabs
        {
            let (
                _sell,
                items,
                floor,
                kitchen,
                sales,
                credit,
                team,
                backup,
                inventory,
                branches,
                is_owner,
            ) = {
                let st = app_state().lock().unwrap();
                match &st.session {
                    Some(s) => (
                        s.can_access_sell(),
                        s.can_access_items(),
                        s.can_access_floor(),
                        s.can_access_kitchen(),
                        s.can_access_sales(),
                        s.can_access_credit(),
                        s.can_access_team() || s.is_owner(),
                        s.can_access_backup() || s.is_owner(),
                        s.can_access_inventory() || s.is_owner(),
                        s.can_access_branches() || s.is_owner(),
                        s.is_owner(),
                    ),
                    None => (
                        false, false, false, false, false, false, false, false, false, false,
                        false,
                    ),
                }
            };
            // A simple shop never opens Tables or Kitchen until Tables & kitchen is on.
            let (restaurant, is_client, is_shop, extra_tills, branch_count) = {
                let st = app_state().lock().unwrap();
                let branch_count = st
                    .store
                    .as_ref()
                    .and_then(|s| s.list_branches().ok())
                    .map(|b| b.len())
                    .unwrap_or(1);
                (
                    st.venue_mode == VenueMode::Restaurant,
                    st.device_role == DeviceRole::Client,
                    st.device_role == DeviceRole::Server,
                    st.extra_tills_allowed,
                    branch_count,
                )
            };
            self.ui.button(cx, ids!(nav_books)).set_enabled(cx, sales || is_owner || items);
            self.ui.button(cx, ids!(nav_buy)).set_enabled(cx, inventory || sales || is_owner);
            self.ui.button(cx, ids!(more_items)).set_visible(cx, items);
            self.ui
                .button(cx, ids!(more_tables))
                .set_visible(cx, floor && restaurant);
            self.ui
                .button(cx, ids!(more_kitchen))
                .set_visible(cx, kitchen && restaurant);
            self.ui.button(cx, ids!(send_kot)).set_visible(cx, restaurant);
            self.ui.view(cx, ids!(order_type_row)).set_visible(cx, restaurant);
            self.ui
                .widget(cx, ids!(order_type_label))
                .set_visible(cx, restaurant);
            self.ui
                .view(cx, ids!(menu_station_row))
                .set_visible(cx, restaurant);
            self.ui
                .view(cx, ids!(staff_role_waiter_slot))
                .set_visible(cx, restaurant);
            self.ui
                .view(cx, ids!(staff_role_kitchen_slot))
                .set_visible(cx, restaurant);
            self.ui.button(cx, ids!(feat_floor)).set_visible(cx, restaurant);
            self.ui
                .button(cx, ids!(feat_kitchen))
                .set_visible(cx, restaurant);
            self.ui
                .button(cx, ids!(allow_extra_tills))
                .set_visible(cx, is_shop && is_owner && !extra_tills);
            self.ui
                .button(cx, ids!(stop_extra_tills))
                .set_visible(cx, is_shop && is_owner && extra_tills);
            self.ui.button(cx, ids!(more_sales)).set_visible(cx, sales);
            self.ui.button(cx, ids!(more_day_close)).set_visible(cx, sales || is_owner);
            self.ui.button(cx, ids!(more_reports)).set_visible(cx, sales || is_owner);
            // Stock and branches are the Server's books. A Client's copy is a
            // sync cache, so every write there would be lost — do not offer it.
            self.ui
                .button(cx, ids!(more_inventory))
                .set_visible(cx, inventory && !is_client);
            self.ui
                .button(cx, ids!(more_branches))
                .set_visible(cx, branches && !is_client && branch_count > 1);
            self.ui.button(cx, ids!(more_credit)).set_visible(cx, credit);
            self.ui.button(cx, ids!(more_team)).set_visible(cx, team);
            self.ui.button(cx, ids!(more_settings)).set_visible(cx, backup);
        }

        self.lift_footer_above_system_bar(cx);
        self.repair_focused_carets(cx);
        self.ui.redraw(cx);
    }

    /// Android draws edge to edge. The Sell / Items / More bar is the last
    /// row, so without the navigation-bar inset it sits under the system bar.
    fn lift_footer_above_system_bar(&mut self, cx: &mut Cx) {
        let bottom = 8.0 + cx.display_context.safe_area_insets.bottom;
        let nav = self.ui.view(cx, ids!(bottom_nav));
        let changed = if let Some(mut inner) = nav.borrow_mut() {
            if (inner.layout.padding.bottom - bottom).abs() > 0.5 {
                inner.layout.padding.bottom = bottom;
                true
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            nav.redraw(cx);
        }
    }

    /// Makepad caret visibility is `(1 - blink) * focus` from TextInput animators.
    /// Key focus can survive hide/show / list rebind without a new `Hit::KeyFocus`,
    /// so the field stays typable but the caret never blinks. Repair every field
    /// that currently holds key focus — not just one screen's inputs.
    fn repair_focused_carets(&mut self, cx: &mut Cx) {
        let top_level = [
            ids!(cart_total_input),
            ids!(customer_name_input),
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
            ids!(amend_total_input),
            ids!(reason_input),
            ids!(approval_pin_input),
            ids!(credit_add_name),
            ids!(settle_amount_input),
            ids!(staff_name_input),
            ids!(staff_title_input),
            ids!(staff_pin_input),
            ids!(my_pin_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_station_input),
            ids!(menu_modifiers_input),
            ids!(inv_item_input),
            ids!(inv_qty_input),
            ids!(inv_note_input),
            ids!(inv_cost_input),
            ids!(inv_supplier_input),
            ids!(org_name_input),
            ids!(branch_name_input),
            ids!(branch_code_input),
            ids!(branch_switch_input),

            ids!(country_code_input),

            ids!(business_tax_id_input),
            ids!(service_bps_input),
            ids!(onboard_venue_input),
            ids!(onboard_org_input),
            ids!(mix_cash_input),
            ids!(mix_upi_input),
            ids!(mix_card_input),
            ids!(mix_credit_input),
        ];
        for path in top_level {
            let field = self.ui.text_input(cx, path);
            let area = field.area();
            if !area.is_empty() && cx.has_key_focus(area) {
                field.take_key_focus(cx);
            }
        }

        let (cart_n, amend_n) = {
            let st = app_state().lock().unwrap();
            (st.cart.lines.len(), st.amend_cart.lines.len())
        };
        for (list_path, n) in [
            (ids!(cart_list), cart_n),
            (ids!(amend_cart_list), amend_n),
        ] {
            let host = self.ui.widget(cx, list_path);
            let list = host.portal_list(cx, ids!(list));
            for i in 0..n {
                let item = list.item(cx, i, live_id!(Item));
                for field_id in [ids!(unit_price), ids!(line_total), ids!(note)] {
                    let field = item.text_input(cx, field_id);
                    let area = field.area();
                    if !area.is_empty() && cx.has_key_focus(area) {
                        field.take_key_focus(cx);
                    }
                }
            }
        }
        self.paint_chrome(cx);
        self.paint_choice_state(cx);
        self.sync_dev_size(cx);
    }

    /// Selected choice = ChipBtnActive visible; idle = ChipBtn visible.
    /// (Button face colors are private — visibility swap is the safe highlight.)
    fn paint_choice_pair(
        &mut self,
        cx: &mut Cx,
        idle: &[LiveId],
        active: &[LiveId],
        selected: bool,
    ) {
        self.ui.button(cx, idle).set_visible(cx, !selected);
        self.ui.button(cx, active).set_visible(cx, selected);
    }

    fn choice_chip_clicked(
        &self,
        cx: &Cx,
        actions: &Actions,
        idle: &[LiveId],
        active: &[LiveId],
    ) -> bool {
        self.ui.button(cx, idle).clicked(actions) || self.ui.button(cx, active).clicked(actions)
    }

    /// Bare-minimal **None** (`custom`) when the shop never picked a template.
    fn ensure_default_template(st: &mut AppState) -> Result<(), String> {
        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
        let stored = store.entity_template_id().unwrap_or_default();
        let mem = st.entity_template_id.trim();
        let stored = stored.trim();
        let selected = if !mem.is_empty() && mem != "custom" {
            mem
        } else if !stored.is_empty() && stored != "custom" {
            stored
        } else {
            "custom"
        };
        if selected != "custom" {
            if mem != selected {
                st.entity_template_id = selected.to_string();
            }
            return Ok(());
        }
        let tmpl = store.apply_entity_template("custom")?;
        st.entity_template_id = tmpl.id.to_string();
        st.venue_mode = tmpl.venue_mode;
        Ok(())
    }

    fn paint_choice_state(&mut self, cx: &mut Cx) {
        let (tmpl_id, money_lane, draft_role, order_type, buy_on_account, follow_platform, onboard_step) = {
            let st = app_state().lock().unwrap();
            let id = st.entity_template_id.trim();
            (
                if id.is_empty() {
                    "custom".to_string()
                } else {
                    st.entity_template_id.clone()
                },
                st.money_lane,
                st.draft_staff_role,
                st.cart.order_type,
                st.buy_on_account,
                st.language_follow_platform,
                st.onboard_step,
            )
        };
        let tmpl_pairs = [
            (
                ids!(onboard_tmpl_custom),
                ids!(onboard_tmpl_custom_on),
                "custom",
            ),
            (
                ids!(onboard_tmpl_counter),
                ids!(onboard_tmpl_counter_on),
                "counter_shop",
            ),
            (
                ids!(onboard_tmpl_courier),
                ids!(onboard_tmpl_courier_on),
                "courier_agency",
            ),
            (ids!(onboard_tmpl_food), ids!(onboard_tmpl_food_on), "food_service"),
            (
                ids!(onboard_tmpl_personal),
                ids!(onboard_tmpl_personal_on),
                "personal_books",
            ),
            (
                ids!(settings_tmpl_custom),
                ids!(settings_tmpl_custom_on),
                "custom",
            ),
            (
                ids!(settings_tmpl_counter),
                ids!(settings_tmpl_counter_on),
                "counter_shop",
            ),
            (
                ids!(settings_tmpl_courier),
                ids!(settings_tmpl_courier_on),
                "courier_agency",
            ),
            (
                ids!(settings_tmpl_food),
                ids!(settings_tmpl_food_on),
                "food_service",
            ),
            (
                ids!(settings_tmpl_personal),
                ids!(settings_tmpl_personal_on),
                "personal_books",
            ),
        ];
        for (idle, active, id) in tmpl_pairs {
            self.paint_choice_pair(cx, idle, active, tmpl_id == id);
        }
        self.paint_choice_pair(
            cx,
            ids!(onboard_fresh),
            ids!(onboard_fresh_on),
            onboard_step == 1,
        );
        self.paint_choice_pair(
            cx,
            ids!(onboard_book),
            ids!(onboard_book_on),
            onboard_step == 2,
        );
        self.ui
            .view(cx, ids!(onboard_fresh_panel))
            .set_visible(cx, onboard_step == 1);
        self.ui
            .view(cx, ids!(onboard_book_panel))
            .set_visible(cx, onboard_step == 2);
        self.paint_choice_pair(
            cx,
            ids!(lane_declared),
            ids!(lane_declared_on),
            matches!(money_lane, MoneyLane::Declared),
        );
        self.paint_choice_pair(
            cx,
            ids!(lane_owner),
            ids!(lane_owner_on),
            matches!(money_lane, MoneyLane::Owner),
        );
        self.paint_choice_pair(cx, ids!(buy_paid), ids!(buy_paid_on), !buy_on_account);
        self.paint_choice_pair(
            cx,
            ids!(lang_setup_platform),
            ids!(lang_setup_platform_on),
            follow_platform,
        );
        self.paint_choice_pair(
            cx,
            ids!(lang_setup_user),
            ids!(lang_setup_user_on),
            !follow_platform,
        );
        self.paint_choice_pair(
            cx,
            ids!(settings_lang_platform),
            ids!(settings_lang_platform_on),
            follow_platform,
        );
        self.paint_choice_pair(
            cx,
            ids!(settings_lang_user),
            ids!(settings_lang_user_on),
            !follow_platform,
        );
        // Language catalogue only when the user picks a fixed language.
        self.ui
            .widget(cx, ids!(setup_lang_list))
            .set_visible(cx, !follow_platform);
        self.ui
            .widget(cx, ids!(language_list))
            .set_visible(cx, !follow_platform);
        self.paint_choice_pair(
            cx,
            ids!(buy_on_account),
            ids!(buy_on_account_on),
            buy_on_account,
        );
        self.paint_choice_pair(
            cx,
            ids!(staff_role_cashier),
            ids!(staff_role_cashier_on),
            draft_role == StaffRole::Cashier,
        );
        self.paint_choice_pair(
            cx,
            ids!(staff_role_manager),
            ids!(staff_role_manager_on),
            draft_role == StaffRole::Manager,
        );
        self.paint_choice_pair(
            cx,
            ids!(staff_role_waiter),
            ids!(staff_role_waiter_on),
            draft_role == StaffRole::Waiter,
        );
        self.paint_choice_pair(
            cx,
            ids!(staff_role_kitchen),
            ids!(staff_role_kitchen_on),
            draft_role == StaffRole::Kitchen,
        );
        self.paint_choice_pair(
            cx,
            ids!(order_counter),
            ids!(order_counter_on),
            order_type == OrderType::Counter,
        );
        self.paint_choice_pair(
            cx,
            ids!(order_dine),
            ids!(order_dine_on),
            order_type == OrderType::DineIn,
        );
        self.paint_choice_pair(
            cx,
            ids!(order_takeaway),
            ids!(order_takeaway_on),
            order_type == OrderType::Takeaway,
        );
        // Keep lane labels current on both faces.
        let (declared, owner) = {
            let st = app_state().lock().unwrap();
            (st.lane_declared_label(), st.lane_owner_label())
        };
        for path in [ids!(lane_declared), ids!(lane_declared_on)] {
            self.ui.button(cx, path).set_text(cx, &declared);
        }
        for path in [ids!(lane_owner), ids!(lane_owner_on)] {
            self.ui.button(cx, path).set_text(cx, &owner);
        }
    }

    /// Every static label and button. List rows set their own text while drawing.
    fn paint_chrome(&mut self, cx: &mut Cx) {
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |key: &str| locale::t(&lang, key);
        self.ui.button(cx, ids!(nav_books)).set_text(cx, &s("books"));
        self.ui.button(cx, ids!(nav_sell)).set_text(cx, &s("sell"));
        self.ui.button(cx, ids!(nav_buy)).set_text(cx, &s("buy"));
        self.ui.button(cx, ids!(nav_more)).set_text(cx, &s("more"));
        self.ui.button(cx, ids!(more_items)).set_text(cx, &s("items"));
        self.ui.button(cx, ids!(books_to_sell)).set_text(cx, &s("sell_money_in"));
        self.ui.button(cx, ids!(books_to_buy)).set_text(cx, &s("buy_money_out"));
        self.ui.button(cx, ids!(books_day_close)).set_text(cx, &s("day_close"));
        self.ui.button(cx, ids!(books_reports)).set_text(cx, &s("reports_expenses"));
        self.ui.button(cx, ids!(books_credit)).set_text(cx, &s("credit_customers"));
        self.ui.button(cx, ids!(books_refresh)).set_text(cx, &s("refresh"));
        self.ui.button(cx, ids!(buy_purchase)).set_text(cx, &s("record_purchase"));
        self.ui.button(cx, ids!(buy_expense_add)).set_text(cx, &s("add_expense"));
        self.ui.button(cx, ids!(buy_to_inventory)).set_text(cx, &s("inventory_stock"));
        self.ui.widget(cx, ids!(books_heading)).set_text(cx, &plain_display_text(&s("books")));
        self.ui.widget(cx, ids!(books_help)).set_text(cx, &plain_display_text(&s("books_help")));
        self.ui.widget(cx, ids!(buy_heading)).set_text(cx, &plain_display_text(&s("buy")));
        self.ui.widget(cx, ids!(buy_help)).set_text(cx, &plain_display_text(&s("buy_help")));
        self.ui.button(cx, ids!(more_tables)).set_text(cx, &s("tables"));
        self.ui.button(cx, ids!(more_kitchen)).set_text(cx, &s("kitchen"));
        self.ui.button(cx, ids!(more_sales)).set_text(cx, &s("sales_refunds"));
        self.ui.button(cx, ids!(more_day_close)).set_text(cx, &s("day_close"));
        self.ui.button(cx, ids!(more_reports)).set_text(cx, &s("reports_expenses"));
        self.ui.button(cx, ids!(more_inventory)).set_text(cx, &s("inventory_stock"));
        self.ui.button(cx, ids!(more_branches)).set_text(cx, &s("branches_org"));
        self.ui.button(cx, ids!(more_credit)).set_text(cx, &s("credit_customers"));
        self.ui.button(cx, ids!(more_team)).set_text(cx, &s("team_pins"));
        self.ui.button(cx, ids!(more_settings)).set_text(cx, &s("backup_settings"));
        self.ui.button(cx, ids!(more_lock)).set_text(cx, &s("lock_app"));
        self.ui.button(cx, ids!(screen_back)).set_text(cx, &s("back"));
        for path in [
            ids!(floor_back),
            ids!(kitchen_back),
            ids!(sales_back),
            ids!(credit_back),
            ids!(team_back),
            ids!(settings_back),
            ids!(day_close_back),
            ids!(reports_back),
            ids!(inventory_back),
            ids!(branches_back),
            ids!(client_pair_back),
        ] {
            self.ui.button(cx, path).set_text(cx, &s("back"));
        }
        self.ui.button(cx, ids!(clear_cart)).set_text(cx, &s("clear"));
        self.ui.button(cx, ids!(pay_mix)).set_text(cx, "Take payment");
        {
            let st = app_state().lock().unwrap();
            let declared = st.lane_declared_label();
            let owner = st.lane_owner_label();
            let lane_body = match st.money_lane {
                MoneyLane::Declared => format!("Bill: {declared}"),
                MoneyLane::Owner => format!("Bill: {owner}"),
            };
            self.ui
                .widget(cx, ids!(money_lane_label))
                .set_text(cx, &plain_display_text(&lane_body));
            for path in [ids!(lane_declared), ids!(lane_declared_on)] {
                self.ui.button(cx, path).set_text(cx, &declared);
            }
            for path in [ids!(lane_owner), ids!(lane_owner_on)] {
                self.ui.button(cx, path).set_text(cx, &owner);
            }
        }
        self.ui.button(cx, ids!(send_kot)).set_text(cx, &s("send_kot"));
        for path in [ids!(order_counter), ids!(order_counter_on)] {
            self.ui.button(cx, path).set_text(cx, &s("counter"));
        }
        for path in [ids!(order_dine), ids!(order_dine_on)] {
            self.ui.button(cx, path).set_text(cx, &s("dine_in"));
        }
        for path in [ids!(order_takeaway), ids!(order_takeaway_on)] {
            self.ui.button(cx, path).set_text(cx, &s("takeaway"));
        }
        let (editing, editing_name) = {
            let st = app_state().lock().unwrap();
            let editing = st.editing_product_id.is_some();
            let name = st.editing_product_id.as_ref().and_then(|id| {
                st.products
                    .iter()
                    .find(|product| product.id == *id)
                    .map(|product| product.name.clone())
            });
            (editing, name)
        };
        self.ui.button(cx, ids!(menu_new)).set_visible(cx, editing);
        self.ui.button(cx, ids!(menu_new)).set_text(cx, &s("new_item"));
        self.ui.button(cx, ids!(menu_add)).set_text(
            cx,
            &s(if editing { "save" } else { "add_item" }),
        );
        self.ui.widget(cx, ids!(menu_mode_label)).set_text(cx, &plain_display_text(&match editing_name {
                Some(name) => format!("{}: {name}", s("editing_item")),
                None => s("adding_item"),
            },));
        self.ui.button(cx, ids!(image_auto)).set_text(cx, &s("auto"));
        self.ui.button(cx, ids!(image_pick)).set_text(cx, &s("photo"));
        self.ui.button(cx, ids!(role_owner)).set_text(cx, &s("owner"));
        self.ui.button(cx, ids!(role_staff)).set_text(cx, &s("staff"));
        self.ui.button(cx, ids!(lock_staff_prev)).set_text(cx, &s("prev"));
        self.ui.button(cx, ids!(lock_staff_next)).set_text(cx, &s("next"));
        self.ui.button(cx, ids!(pin_ok)).set_text(cx, &s("ok"));
        self.ui.button(cx, ids!(device_role_server)).set_text(cx, &s("this_shop"));
        self.ui.button(cx, ids!(device_role_client)).set_text(cx, &s("extra_till"));
        self.ui.button(cx, ids!(customer_clear)).set_text(cx, &s("clear"));
        self.ui.button(cx, ids!(undo_snack_btn)).set_text(cx, &s("undo"));
        self.ui.button(cx, ids!(mode_counter)).set_text(cx, &s("off"));
        self.ui.button(cx, ids!(mode_restaurant)).set_text(cx, &s("tables_kitchen"));
        self.ui.button(cx, ids!(inv_adjust_minus)).set_text(cx, &s("stock_minus"));
        self.ui.button(cx, ids!(inv_adjust_plus)).set_text(cx, &s("stock_plus"));
        self.ui.button(cx, ids!(inv_track)).set_text(cx, &s("start_tracking"));
        self.ui.button(cx, ids!(inv_purchase)).set_text(cx, &s("record_purchase"));
        self.ui.button(cx, ids!(inv_refresh)).set_text(cx, &s("refresh"));
        self.ui.button(cx, ids!(reports_refresh)).set_text(cx, &s("refresh"));
        self.ui.button(cx, ids!(settle_cash)).set_text(cx, &s("settle_cash"));
        self.ui.button(cx, ids!(settle_card)).set_text(cx, &s("settle_card"));
        self.ui.button(cx, ids!(amend_pay_cash)).set_text(cx, &s("cash"));
        self.ui.button(cx, ids!(amend_pay_card)).set_text(cx, &s("card"));
        self.ui.widget(cx, ids!(items_heading)).set_text(cx, &plain_display_text(&s("your_items")));
        self.ui.widget(cx, ids!(items_help_label)).set_text(cx, &plain_display_text(&s("items_help")));
        self.ui.widget(cx, ids!(more_heading)).set_text(cx, &plain_display_text(&s("more")));
        self.ui.widget(cx, ids!(floor_title)).set_text(cx, &plain_display_text(&s("tables")));
        self.ui.widget(cx, ids!(kitchen_title)).set_text(cx, &plain_display_text(&s("kitchen")));
        self.ui.widget(cx, ids!(sales_title)).set_text(cx, &plain_display_text(&s("sales")));
        self.ui.widget(cx, ids!(credit_title)).set_text(cx, &plain_display_text(&s("credit")));
        self.ui.widget(cx, ids!(team_title)).set_text(cx, &plain_display_text(&s("team")));
        self.ui.widget(cx, ids!(settings_title)).set_text(cx, &plain_display_text(&s("language_currency")));
        self.ui.widget(cx, ids!(day_close_title)).set_text(cx, &plain_display_text(&s("day_close")));
        self.ui.widget(cx, ids!(reports_title)).set_text(cx, &plain_display_text(&s("reports")));
        self.ui.widget(cx, ids!(inventory_title)).set_text(cx, &plain_display_text(&s("inventory")));
        self.ui.widget(cx, ids!(branches_title)).set_text(cx, &plain_display_text(&s("branches")));
        self.ui
            .widget(cx, ids!(setup_lang_title))
            .set_text(cx, &plain_display_text(&s("language")));
        self.ui
            .widget(cx, ids!(setup_lang_hint))
            .set_text(cx, &plain_display_text(&s("language_setup_hint")));
        self.ui
            .widget(cx, ids!(device_setup_banner))
            .set_text(cx, &plain_display_text(&s("this_device")));
        self.ui
            .widget(cx, ids!(device_use_question))
            .set_text(cx, &plain_display_text(&s("device_use_q")));
        self.ui
            .widget(cx, ids!(device_section_label))
            .set_text(cx, &plain_display_text(&s("this_device")));
        self.ui
            .widget(cx, ids!(device_setup_help))
            .set_text(cx, &plain_display_text(&s("device_setup_help")));
        self.ui
            .widget(cx, ids!(server_pick_title))
            .set_text(cx, &plain_display_text(&s("choose_server")));
        self.ui
            .widget(cx, ids!(server_pick_hint))
            .set_text(cx, &plain_display_text(&s("choose_server_hint")));
        self.ui
            .button(cx, ids!(server_create_new))
            .set_text(cx, &s("create_new_server"));
        self.ui
            .button(cx, ids!(server_pick_cancel))
            .set_text(cx, &s("back"));
        self.ui
            .text_input(cx, ids!(server_name_input))
            .set_empty_text(cx, s("new_server_name_ph"));
        let picking_server = app_state().lock().unwrap().picking_server;
        self.ui
            .view(cx, ids!(server_pick_section))
            .set_visible(cx, picking_server);
        self.ui
            .button(cx, ids!(device_role_server))
            .set_visible(cx, !picking_server);
        self.ui
            .button(cx, ids!(device_role_client))
            .set_visible(cx, !picking_server);
        self.ui
            .button(cx, ids!(lang_setup_platform))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(lang_setup_platform_on))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(lang_setup_user))
            .set_text(cx, &s("lang_pick_user"));
        self.ui
            .button(cx, ids!(lang_setup_user_on))
            .set_text(cx, &s("lang_pick_user"));
        self.ui
            .button(cx, ids!(settings_lang_platform))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(settings_lang_platform_on))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(settings_lang_user))
            .set_text(cx, &s("lang_pick_user"));
        self.ui
            .button(cx, ids!(settings_lang_user_on))
            .set_text(cx, &s("lang_pick_user"));
        let selected_lang_line = {
            let st = app_state().lock().unwrap();
            let code = if st.language_follow_platform {
                locale::detect_platform_language()
            } else {
                st.locale.language.clone()
            };
            let name = locale::language_by_code(&code)
                .map(|l| format!("{} · {}", l.native_name, l.name_en))
                .unwrap_or_else(|| code.clone());
            if st.language_follow_platform {
                format!("{}: {name}", st.t("lang_platform_now"))
            } else {
                format!("{}: {name}", st.t("selected_language"))
            }
        };
        self.ui
            .widget(cx, ids!(setup_lang_selected))
            .set_text(cx, &plain_display_text(&selected_lang_line));
        self.ui
            .widget(cx, ids!(settings_lang_selected))
            .set_text(cx, &plain_display_text(&selected_lang_line));
    }

    fn show_screen(&mut self, cx: &mut Cx, screen: Screen) {
        let pairs = [
            (ids!(device_setup_screen), screen == Screen::DeviceSetup),
            (ids!(lock_screen), screen == Screen::Lock),
            (ids!(onboard_screen), screen == Screen::Onboard),
            (ids!(client_pair_screen), screen == Screen::ClientPair),
            (ids!(books_screen), screen == Screen::Books),
            (ids!(pos_screen), screen == Screen::Pos),
            (ids!(buy_screen), screen == Screen::Buy),
            (ids!(menu_screen), screen == Screen::Menu),
            (ids!(more_screen), screen == Screen::More),
            (ids!(floor_screen), screen == Screen::Floor),
            (ids!(kitchen_screen), screen == Screen::Kitchen),
            (ids!(sales_screen), screen == Screen::Sales),
            (ids!(credit_screen), screen == Screen::Credit),
            (ids!(team_screen), screen == Screen::Team),
            (ids!(settings_screen), screen == Screen::Settings),
            (ids!(day_close_screen), screen == Screen::DayClose),
            (ids!(reports_screen), screen == Screen::Reports),
            (ids!(inventory_screen), screen == Screen::Inventory),
            (ids!(branches_screen), screen == Screen::Branches),
        ];
        for (path, visible) in pairs {
            self.ui.view(cx, path).set_visible(cx, visible);
        }
        let unlocked = !matches!(
            screen,
            Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
        );
        self.ui.view(cx, ids!(header)).set_visible(cx, unlocked);
        self.ui.view(cx, ids!(today_bar)).set_visible(cx, unlocked);
        self.ui.view(cx, ids!(bottom_nav)).set_visible(cx, unlocked);
        // Child screens carry their own Back control above the title.
        self.ui.button(cx, ids!(screen_back)).set_visible(cx, false);
    }

    fn sync_venue_brand(&mut self, cx: &mut Cx) {
        let (title, header_title, icon_path, icon_hint, saved_title) = {
            let st = app_state().lock().unwrap();
            let title = st.display_venue_title();
            // Multi-outlet owners need to see which shop this device is selling
            // for; a single Main-only setup keeps the plain venue name.
            let header_title = {
                let branch_count = st
                    .store
                    .as_ref()
                    .and_then(|s| s.list_branches().ok())
                    .map(|b| b.len())
                    .unwrap_or(0);
                match st.store.as_ref().and_then(|s| s.active_branch().ok()) {
                    Some(branch) if branch_count > 1 => {
                        let tag = if branch.code.trim().is_empty() {
                            branch.name.clone()
                        } else {
                            branch.code.clone()
                        };
                        format!("{title} · {tag}")
                    }
                    _ => title.clone(),
                }
            };
            let icon_path = st.venue_icon_path();
            let icon_hint = if st.draft_venue_icon_path.is_some() {
                "Icon: new file selected — tap Save brand".to_string()
            } else if icon_path.is_some() {
                "Icon: custom venue mark".to_string()
            } else {
                "Icon: none — choose one".to_string()
            };
            (title, header_title, icon_path, icon_hint, st.venue_title.clone())
        };
        self.ui
            .widget(cx, ids!(brand_title)).set_text(cx, &plain_display_text(&header_title));
        for id in [ids!(setup_brand_title), ids!(lock_brand_title)] {
            self.ui.widget(cx, id).set_text(cx, &plain_display_text(&title));
        }
        for id in [ids!(brand_mark), ids!(setup_brand_mark), ids!(lock_brand_mark)] {
            let img = self.ui.image(cx, id);
            if let Some(ref path) = icon_path {
                img.set_visible(cx, true);
                load_thumb_at(cx, &img, path);
            } else {
                img.set_visible(cx, false);
            }
        }
        self.ui
            .widget(cx, ids!(venue_brand_hint)).set_text(cx, &plain_display_text(&icon_hint));
        let field = self.ui.text_input(cx, ids!(venue_title_input));
        if !cx.has_key_focus(field.area()) {
            let shown = field.text();
            if shown != saved_title {
                field.set_text(cx, &saved_title);
            }
        }
    }

    fn save_venue_brand(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        let title = self.ui.text_input(cx, ids!(venue_title_input)).text();
        let outcome = (|| {
            let mut st = app_state().lock().unwrap();
            let actor = st
                .session
                .clone()
                .ok_or_else(|| "Unlock as Owner to change brand".to_string())?;
            if !actor.is_owner() {
                return Err("Only Owner can change in-app brand".to_string());
            }
            let store = st
                .store
                .as_ref()
                .ok_or_else(|| "Storage not ready".to_string())?;
            store.set_venue_title(&title)?;
            if let Some(path) = st.draft_venue_icon_path.clone() {
                let key = store.import_venue_icon(&path)?;
                st.venue_icon_key = Some(key);
                st.draft_venue_icon_path = None;
            }
            st.venue_title = title.trim().to_string();
            Ok("Brand saved".to_string())
        })();
        match outcome {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn clear_venue_icon(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        let outcome = (|| {
            let mut st = app_state().lock().unwrap();
            let actor = st
                .session
                .clone()
                .ok_or_else(|| "Unlock as Owner to change brand".to_string())?;
            if !actor.is_owner() {
                return Err("Only Owner can change in-app brand".to_string());
            }
            let store = st
                .store
                .as_ref()
                .ok_or_else(|| "Storage not ready".to_string())?;
            store.set_venue_icon_key(None)?;
            st.venue_icon_key = None;
            st.draft_venue_icon_path = None;
            Ok("Icon cleared".to_string())
        })();
        match outcome {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn pick_venue_icon(&mut self, cx: &mut Cx) {
        {
            let st = app_state().lock().unwrap();
            match &st.session {
                Some(a) if a.is_owner() => {}
                Some(_) => {
                    drop(st);
                    self.set_status(cx, "Only Owner can change in-app brand");
                    return;
                }
                None => {
                    drop(st);
                    self.set_status(cx, "Unlock as Owner to change brand");
                    return;
                }
            }
        }
        let dialog = FileDialog::new()
            .set_title("Choose venue icon".into())
            .add_filter(
                "Images".into(),
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                    "gif".into(),
                    "bmp".into(),
                ],
            )
            .set_id(live_id!(venue_icon_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening icon picker…");
    }

    /// Push history when moving between app screens; clear it for lock/setup.
    fn set_screen(&mut self, st: &mut AppState, next: Screen, record_history: bool) {
        if st.screen == next {
            return;
        }
        if record_history
            && screen_keeps_history(st.screen)
            && screen_keeps_history(next)
        {
            st.nav_back.push(st.screen);
            if st.nav_back.len() > 32 {
                st.nav_back.remove(0);
            }
            st.nav_forward.clear();
        } else if !screen_keeps_history(next) {
            st.nav_back.clear();
            st.nav_forward.clear();
        }
        st.screen = next;
    }

    fn go(&mut self, cx: &mut Cx, screen: Screen) {
        {
            let mut st = app_state().lock().unwrap();
            if st.must_confirm_device_role && screen != Screen::DeviceSetup {
                st.status = st.t("device_use_q");
                self.set_screen(&mut st, Screen::DeviceSetup, false);
            } else if st.device_role == DeviceRole::Client
                && !st.client_paired
                && !matches!(
                    screen,
                    Screen::ClientPair | Screen::DeviceSetup | Screen::Lock
                )
            {
                st.status = "Pair this extra till to the shop device first".into();
                self.set_screen(&mut st, Screen::ClientPair, false);
            } else if st.session.is_none()
                && !matches!(
                    screen,
                    Screen::Lock
                        | Screen::DeviceSetup
                        | Screen::Onboard
                        | Screen::ClientPair
                )
            {
                st.status = "Unlock first".into();
                self.set_screen(&mut st, Screen::Lock, false);
            } else if st.session.is_some()
                && st.owner_setup_complete
                && !st.onboarding_complete
                && st.device_role.is_host()
                && !matches!(
                    screen,
                    Screen::Onboard | Screen::DeviceSetup | Screen::Lock
                )
            {
                self.set_screen(&mut st, Screen::Onboard, false);
            } else {
                self.set_screen(&mut st, screen, true);
                if st.device_role == DeviceRole::Client
                    && matches!(
                        screen,
                        Screen::Menu
                            | Screen::Inventory
                            | Screen::Branches
                            | Screen::Team
                            | Screen::Settings
                    )
                {
                    st.status = "Read-only on this extra till — change it on the shop device".into();
                }
            }
        }
        self.sync_chrome(cx);
    }


    fn go_if_allowed(
        &mut self,
        cx: &mut Cx,
        screen: Screen,
        allowed: impl Fn(&StaffMember) -> bool,
    ) {
        let ok = {
            let st = app_state().lock().unwrap();
            st.session.as_ref().map(allowed).unwrap_or(false)
        };
        if ok {
            self.go(cx, screen);
        } else {
            self.set_status(cx, "Not allowed for your role");
        }
    }

    fn copy_text_to_clipboard(&mut self, cx: &mut Cx, text: &str) {
        // Makepad's clipboard op reaches Android's ClipboardManager, which
        // `arboard` cannot; the desktop call is the immediate-feedback path.
        cx.copy_to_clipboard(text);
        let desktop_ok = print_share::copy_text(text);
        println!("Kumkaj share:\n{text}");
        {
            let mut st = app_state().lock().unwrap();
            st.last_receipt = text.to_string();
            st.status = if desktop_ok || cfg!(target_os = "android") {
                "Copied to clipboard".into()
            } else {
                "Receipt in strip — use Print / share if clipboard unavailable".into()
            };
        }
        self.sync_chrome(cx);
    }

    fn print_or_share_receipt(&mut self, cx: &mut Cx, text: &str) {
        cx.copy_to_clipboard(text);
        let _ = print_share::copy_text(text);
        // Android: ACTION_SEND chooser. Other platforms ignore this and open
        // the file below.
        cx.share_text(text);
        let app_files = cx.get_data_dir().map(PathBuf::from);
        match print_share::open_receipt_for_print(text, app_files.as_deref()) {
            Ok(share) => {
                let mut st = app_state().lock().unwrap();
                st.last_receipt = text.to_string();
                st.status = if share.opened {
                    format!("Receipt ready · {}", share.path.display())
                } else if cfg!(target_os = "android") {
                    format!("Share sheet opened · saved to {}", share.path.display())
                } else {
                    format!("Receipt copied · saved to {}", share.path.display())
                };
            }
            Err(e) => {
                let mut st = app_state().lock().unwrap();
                st.last_receipt = text.to_string();
                st.status = format!("Receipt copied · file not saved ({e})");
            }
        }
        self.sync_chrome(cx);
    }

    /// True when a seller text field (not a button) holds key focus / soft keyboard.
    fn text_field_has_focus(&mut self, cx: &mut Cx) -> bool {
        let top_level = [
            ids!(cart_total_input),
            ids!(customer_name_input),
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
            ids!(amend_total_input),
            ids!(reason_input),
            ids!(approval_pin_input),
            ids!(credit_add_name),
            ids!(settle_amount_input),
            ids!(staff_name_input),
            ids!(staff_title_input),
            ids!(staff_pin_input),
            ids!(my_pin_input),
            ids!(venue_title_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_station_input),
            ids!(menu_modifiers_input),
            ids!(inv_item_input),
            ids!(inv_qty_input),
            ids!(inv_note_input),
            ids!(inv_cost_input),
            ids!(inv_supplier_input),
            ids!(org_name_input),
            ids!(branch_name_input),
            ids!(branch_code_input),
            ids!(branch_switch_input),

            ids!(country_code_input),

            ids!(business_tax_id_input),
            ids!(service_bps_input),
            ids!(onboard_venue_input),
            ids!(onboard_org_input),
            ids!(mix_cash_input),
            ids!(mix_upi_input),
            ids!(mix_card_input),
            ids!(mix_credit_input),
        ];
        for path in top_level {
            let field = self.ui.text_input(cx, path);
            let area = field.area();
            if !area.is_empty() && cx.has_key_focus(area) {
                return true;
            }
        }
        let (cart_n, amend_n) = {
            let st = app_state().lock().unwrap();
            (st.cart.lines.len(), st.amend_cart.lines.len())
        };
        for (list_path, n) in [
            (ids!(cart_list), cart_n),
            (ids!(amend_cart_list), amend_n),
        ] {
            let host = self.ui.widget(cx, list_path);
            let list = host.portal_list(cx, ids!(list));
            for i in 0..n {
                let item = list.item(cx, i, live_id!(Item));
                for field_id in [ids!(unit_price), ids!(line_total), ids!(note)] {
                    let field = item.text_input(cx, field_id);
                    let area = field.area();
                    if !area.is_empty() && cx.has_key_focus(area) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Natural back: dismiss soft-keyboard when a text field is focused,
    /// else pop screen history (same as mouse Back / left-edge swipe).
    fn navigate_back(&mut self, cx: &mut Cx) -> bool {
        // Do NOT treat generic key focus (buttons after tap) as a back step —
        // that forced a useless double swipe on Android.
        if self.text_field_has_focus(cx) {
            cx.set_key_focus(Area::Empty);
            cx.hide_text_ime();
            if let Ok(mut flag) = ime_suppress_refocus().lock() {
                *flag = true;
            }
            return true;
        }
        let dest = {
            let mut st = app_state().lock().unwrap();
            if let Some(prev) = st.nav_back.pop() {
                let current = st.screen;
                st.nav_forward.push(current);
                if st.nav_forward.len() > 32 {
                    st.nav_forward.remove(0);
                }
                Some(prev)
            } else {
                // First entry / empty stack: parent hierarchy, no forward entry.
                let fallback = match st.screen {
                    Screen::ClientPair => Some(Screen::DeviceSetup),
                    Screen::Floor
                    | Screen::Kitchen
                    | Screen::Sales
                    | Screen::Credit
                    | Screen::Team
                    | Screen::Settings
                    | Screen::DayClose
                    | Screen::Reports
                    | Screen::Inventory
                    | Screen::Branches => Some(Screen::More),
                    Screen::More | Screen::Menu | Screen::Buy => Some(unlocked_home()),
                    Screen::Books => None,
                    _ => None,
                };
                if let Some(prev) = fallback {
                    st.nav_forward.clear();
                    if prev == Screen::DeviceSetup {
                        st.must_confirm_device_role = true;
                        st.status =
                            "Pick Server Mode or Client Mode — Back left Client pairing".into();
                    }
                    Some(prev)
                } else {
                    None
                }
            }
        };
        if let Some(prev) = dest {
            {
                let mut st = app_state().lock().unwrap();
                if prev == Screen::DeviceSetup {
                    st.must_confirm_device_role = true;
                }
                self.set_screen(&mut st, prev, false);
            }
            self.sync_chrome(cx);
            true
        } else {
            false
        }
    }

    /// Mouse Forward / right-edge swipe / Alt+Right — redo a screen we backed out of.
    fn navigate_forward(&mut self, cx: &mut Cx) -> bool {
        let dest = {
            let mut st = app_state().lock().unwrap();
            st.nav_forward.pop().map(|next| {
                let current = st.screen;
                st.nav_back.push(current);
                if st.nav_back.len() > 32 {
                    st.nav_back.remove(0);
                }
                next
            })
        };
        if let Some(next) = dest {
            {
                let mut st = app_state().lock().unwrap();
                self.set_screen(&mut st, next, false);
            }
            self.sync_chrome(cx);
            true
        } else {
            false
        }
    }

    fn window_inner_size(&self, cx: &Cx) -> DVec2 {
        let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
        dvec2(
            if size.x > 1.0 { size.x } else { 1024.0 },
            if size.y > 1.0 { size.y } else { 768.0 },
        )
    }

    /// Desktop only: stash last size/position and apply it before CreateWindow.
    fn restore_window_geom(&self, cx: &mut Cx) {
        if cfg!(target_os = "android") {
            return;
        }
        let Some(prefs) = store::load_window_geom() else {
            *pending_window_geom().lock().unwrap() = None;
            return;
        };
        let min = store::MIN_DESKTOP_WINDOW_CSS;
        let prefs = store::WindowGeomPrefs {
            width: prefs.width.max(min),
            height: prefs.height.max(min),
            x: prefs.x,
            y: prefs.y,
        };
        *pending_window_geom().lock().unwrap() = Some(prefs);
        self.ui.window(cx, ids!(main_window)).configure_window(
            cx,
            dvec2(prefs.width, prefs.height),
            dvec2(prefs.x, prefs.y),
            false,
            store::APP_LABEL.to_string(),
        );
    }

    /// After the OS window exists, force the saved size if create used the DSL default.
    /// Returns true when a restore resize was requested (skip persisting that frame).
    fn apply_pending_window_geom_after_create(&self, cx: &mut Cx) -> bool {
        if cfg!(target_os = "android") {
            return false;
        }
        let Some(prefs) = pending_window_geom().lock().unwrap().take() else {
            return false;
        };
        let win = self.ui.window(cx, ids!(main_window));
        let size = win.get_inner_size(cx);
        let pos = win.get_position(cx);
        let size_mismatch =
            (size.x - prefs.width).abs() > 1.0 || (size.y - prefs.height).abs() > 1.0;
        let pos_mismatch = (pos.x - prefs.x).abs() > 1.0 || (pos.y - prefs.y).abs() > 1.0;
        if size_mismatch {
            win.resize(cx, dvec2(prefs.width, prefs.height));
        }
        if pos_mismatch {
            win.reposition(cx, dvec2(prefs.x, prefs.y));
        }
        size_mismatch || pos_mismatch
    }

    /// Desktop (Windows / Linux / macOS): never allow inner size below 320 css px.
    fn enforce_min_desktop_window_size(&self, cx: &mut Cx) {
        if cfg!(target_os = "android") {
            return;
        }
        let win = self.ui.window(cx, ids!(main_window));
        let size = win.get_inner_size(cx);
        let min = store::MIN_DESKTOP_WINDOW_CSS;
        let w = size.x.max(min);
        let h = size.y.max(min);
        if (w - size.x).abs() > 0.5 || (h - size.y).abs() > 0.5 {
            win.resize(cx, dvec2(w, h));
        }
    }

    fn persist_window_geom(&self, cx: &Cx) {
        if cfg!(target_os = "android") {
            return;
        }
        // Don't clobber a good saved size with the pre-restore create size.
        if pending_window_geom().lock().unwrap().is_some() {
            return;
        }
        let win = self.ui.window(cx, ids!(main_window));
        let size = win.get_inner_size(cx);
        let pos = win.get_position(cx);
        let min = store::MIN_DESKTOP_WINDOW_CSS;
        store::save_window_geom(store::WindowGeomPrefs {
            width: size.x.max(min),
            height: size.y.max(min),
            x: pos.x,
            y: pos.y,
        });
    }

    fn window_inner_width(&self, cx: &Cx) -> f64 {
        self.window_inner_size(cx).x
    }

    /// Caption-bar WxH (left of the Kumkaj title). Controlled by
    /// `KUMKAJ_SHOW_SIZE_BADGE` in `env/development.env` (not Cargo profile —
    /// Android dev builds use `--release` but still load development.env).
    fn sync_dev_size(&mut self, cx: &mut Cx) {
        let show = store::SHOW_SIZE_BADGE;
        self.ui.view(cx, ids!(dev_size_badge)).set_visible(cx, show);
        if !show {
            return;
        }
        let size = self.window_inner_size(cx);
        // Compact: fits the caption row next to the title without clipping.
        let text = format!("{:.0}×{:.0}", size.x, size.y);
        self.ui.label(cx, ids!(dev_size_label)).set_text(cx, &text);
    }

    fn edge_swipe_nav(
        &mut self,
        cx: &mut Cx,
        start: DVec2,
        end: DVec2,
        edge: SwipeNav,
    ) -> bool {
        let dx = end.x - start.x;
        let dy = (end.y - start.y).abs();
        match edge {
            SwipeNav::Back if dx > 72.0 && dx > dy * 1.5 => self.navigate_back(cx),
            SwipeNav::Forward if dx < -72.0 && (-dx) > dy * 1.5 => self.navigate_forward(cx),
            _ => false,
        }
    }

    /// Mouse Back/Forward, left-edge swipe → back, right-edge swipe → forward.
    /// TouchUpdate on phones; primary MouseDown/Up edge drag on desktop.
    fn pointer_navigates(&mut self, cx: &mut Cx, event: &Event) -> bool {
        let navigated = match event {
            Event::MouseDown(e) if e.button.is_back() => {
                *mouse_swipe_origin().lock().unwrap() = None;
                self.navigate_back(cx)
            }
            Event::MouseDown(e) if e.button.is_forward() => {
                *mouse_swipe_origin().lock().unwrap() = None;
                self.navigate_forward(cx)
            }
            Event::MouseDown(e) if e.button.is_primary() => {
                let width = self.window_inner_width(cx);
                let mut origin = mouse_swipe_origin().lock().unwrap();
                if e.abs.x < 48.0 {
                    *origin = Some((e.abs, SwipeNav::Back));
                } else if e.abs.x > width - 48.0 {
                    *origin = Some((e.abs, SwipeNav::Forward));
                } else {
                    *origin = None;
                }
                false
            }
            Event::MouseUp(e) if e.button.is_primary() => {
                let start_edge = mouse_swipe_origin().lock().unwrap().take();
                match start_edge {
                    Some((start, edge)) => self.edge_swipe_nav(cx, start, e.abs, edge),
                    None => false,
                }
            }
            Event::TouchUpdate(e) => {
                let width = self.window_inner_width(cx);
                let mut origin = swipe_origin().lock().unwrap();
                let mut nav: Option<SwipeNav> = None;
                let mut start_end: Option<(DVec2, DVec2)> = None;
                for touch in &e.touches {
                    match touch.state {
                        TouchState::Start => {
                            if touch.abs.x < 48.0 {
                                *origin = Some((touch.uid, touch.abs, SwipeNav::Back));
                            } else if touch.abs.x > width - 48.0 {
                                *origin = Some((touch.uid, touch.abs, SwipeNav::Forward));
                            }
                        }
                        TouchState::Stop => {
                            if let Some((uid, start, edge)) = *origin
                                && uid == touch.uid
                            {
                                start_end = Some((start, touch.abs));
                                nav = Some(edge);
                                *origin = None;
                            }
                        }
                        _ => {}
                    }
                }
                drop(origin);
                match (nav, start_end) {
                    (Some(edge), Some((start, end))) => {
                        self.edge_swipe_nav(cx, start, end, edge)
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        if navigated {
            // Claim the gesture slot so a paired XF86Back key doesn't step twice.
            let _ = nav_gesture_ok();
            true
        } else {
            false
        }
    }

    fn resolve_data_base(&mut self, cx: &mut Cx) {
        let mut st = app_state().lock().unwrap();
        if st.data_base.is_none() {
            if let Some(dir) = cx.get_data_dir() {
                st.data_base = Some(PathBuf::from(dir));
            } else if let Ok(dir) = store::data_dir() {
                st.data_base = Some(dir);
            }
        }
    }

    fn persist_setup_language(st: &mut AppState) {
        let follow = st.language_follow_platform;
        let code = st.locale.language.clone();
        if let Some(store) = st.store.as_ref() {
            let persisted = if follow {
                store.set_platform_language()
            } else {
                store.set_user_language(&code)
            };
            if let Ok(loc) = persisted {
                st.locale = loc;
            }
        }
    }

    /// Server Mode: create the first local server, or show create/pick when several exist.
    fn begin_server_mode(&mut self, cx: &mut Cx) {
        self.resolve_data_base(cx);
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.refresh_local_servers()?;
            if st.local_servers.is_empty() {
                drop(st);
                self.activate_new_server(cx, "")?;
            } else {
                st.picking_server = true;
                st.status = st.t("choose_server");
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn activate_new_server(&mut self, cx: &mut Cx, name: &str) -> Result<(), String> {
        self.resolve_data_base(cx);
        let base = {
            let st = app_state().lock().unwrap();
            st.data_base
                .clone()
                .ok_or_else(|| "No data directory".to_string())?
        };
        net::stop_server();
        let (_info, store) = local_servers::create_local_server(&base, name)?;
        {
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            let _ = st.reload_all();
            Self::persist_setup_language(&mut st);
            self.finish_as_server(&mut st, true);
        }
        self.sync_chrome(cx);
        Ok(())
    }

    fn activate_existing_server(&mut self, cx: &mut Cx, server_id: &str) -> Result<(), String> {
        self.resolve_data_base(cx);
        let base = {
            let st = app_state().lock().unwrap();
            st.data_base
                .clone()
                .ok_or_else(|| "No data directory".to_string())?
        };
        net::stop_server();
        let store = local_servers::open_server(&base, server_id)?;
        {
            let mut st = app_state().lock().unwrap();
            let previous = st.device_role;
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            let _ = st.reload_all();
            Self::persist_setup_language(&mut st);
            self.finish_as_server(&mut st, previous != DeviceRole::Server);
        }
        self.sync_chrome(cx);
        Ok(())
    }

    fn finish_as_server(&mut self, st: &mut AppState, switching_in: bool) {
        st.device_role = DeviceRole::Server;
        st.must_confirm_device_role = false;
        st.picking_server = false;
        st.lock_as_owner = true;
        st.pin_buffer.clear();
        st.nav_back.clear();
        st.nav_forward.clear();
        if let Some(store) = st.store.as_ref() {
            let _ = store.set_device_role("server");
            if switching_in {
                let _ = store.set_extra_tills_allowed(false);
                st.extra_tills_allowed = false;
                net::stop_server();
            } else if st.extra_tills_allowed {
                let code = store.pairing_code().unwrap_or_default();
                let path = store.db_path().to_path_buf();
                let _ = net::ensure_server(path, code);
            }
        }
        st.screen = Screen::Lock;
        st.status = if !st.owner_setup_complete {
            "Server Mode — set Owner PIN".into()
        } else if st.extra_tills_allowed {
            "Server Mode — extra tills can join. Unlock".into()
        } else {
            "Server Mode — unlock as Owner".into()
        };
    }

    fn begin_client_mode(&mut self, cx: &mut Cx) {
        self.resolve_data_base(cx);
        let result = (|| -> Result<(), String> {
            let base = {
                let st = app_state().lock().unwrap();
                st.data_base
                    .clone()
                    .ok_or_else(|| "No data directory".to_string())?
            };
            net::stop_server();
            let store = local_servers::open_client_store(&base)?;
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.reload_all();
            Self::persist_setup_language(&mut st);
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.prepare_as_client_terminal()?;
            st.device_role = DeviceRole::Client;
            st.must_confirm_device_role = false;
            st.owner_setup_complete = true;
            st.onboarding_complete = true;
            st.extra_tills_allowed = false;
            st.lock_as_owner = true;
            st.pin_buffer.clear();
            st.nav_back.clear();
            st.nav_forward.clear();
            let _ = st.reload_all();
            st.client_paired = store_client_paired(st.store.as_ref());
            if st.staff.iter().any(|s| !s.is_owner()) {
                st.lock_as_owner = false;
                if st.lock_staff_id.is_none() {
                    st.lock_staff_id = st
                        .staff
                        .iter()
                        .find(|s| !s.is_owner())
                        .map(|s| s.id.clone());
                }
            }
            if st.client_paired {
                st.screen = Screen::Lock;
                st.status = "Client Mode — unlock to sell".into();
            } else {
                st.screen = Screen::ClientPair;
                st.status = "Client Mode — pair to the Server before selling".into();
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn set_extra_tills(&mut self, cx: &mut Cx, allowed: bool) {
        self.ensure_store(cx);
        let err = {
            let mut st = app_state().lock().unwrap();
            if let Err(e) = st.require_server_role("Change extra tills") {
                Some(e)
            } else if st.store.is_none() {
                Some("Storage not ready".into())
            } else if let Err(e) = st
                .store
                .as_ref()
                .unwrap()
                .set_extra_tills_allowed(allowed)
            {
                Some(e)
            } else {
                st.extra_tills_allowed = allowed;
                if allowed {
                    let store = st.store.as_ref().unwrap();
                    let code = store.ensure_server_pairing_code().unwrap_or_default();
                    let path = store.db_path().to_path_buf();
                    if let Err(e) = net::ensure_server(path, code.clone()) {
                        st.status = format!("Extra tills on, LAN listen failed: {e}");
                    } else {
                        st.status = format!("Extra tills on · pairing code {code}");
                    }
                } else {
                    net::stop_server();
                    st.status = "Extra tills off — this device still holds the shop books".into();
                }
                None
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn pin_digit(&mut self, cx: &mut Cx, d: char) {
        {
            let mut st = app_state().lock().unwrap();
            if st.pin_buffer.len() < 12 {
                st.pin_buffer.push(d);
            }
            // Clear prior unlock error once the user starts typing again.
            if st.status.starts_with("Wrong")
                || st.status.starts_with("Enter at least")
                || st.status.starts_with("Storage")
                || st.status.starts_with("Add staff")
            {
                st.status.clear();
            }
        }
        self.sync_chrome(cx);
    }

    fn pin_backspace(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.pin_buffer.pop();
            if st.status.starts_with("Wrong")
                || st.status.starts_with("Enter at least")
                || st.status.starts_with("Storage")
                || st.status.starts_with("Add staff")
            {
                st.status.clear();
            }
        }
        self.sync_chrome(cx);
    }

    /// Map physical / numpad keys to a PIN digit. Only plain digit keys — no
    /// Shift/Ctrl/Alt chords.
    fn pin_key_digit(e: &KeyEvent) -> Option<char> {
        if e.modifiers.control || e.modifiers.alt || e.modifiers.logo || e.modifiers.shift {
            return None;
        }
        Some(match e.key_code {
            KeyCode::Key0 | KeyCode::Numpad0 => '0',
            KeyCode::Key1 | KeyCode::Numpad1 => '1',
            KeyCode::Key2 | KeyCode::Numpad2 => '2',
            KeyCode::Key3 | KeyCode::Numpad3 => '3',
            KeyCode::Key4 | KeyCode::Numpad4 => '4',
            KeyCode::Key5 | KeyCode::Numpad5 => '5',
            KeyCode::Key6 | KeyCode::Numpad6 => '6',
            KeyCode::Key7 | KeyCode::Numpad7 => '7',
            KeyCode::Key8 | KeyCode::Numpad8 => '8',
            KeyCode::Key9 | KeyCode::Numpad9 => '9',
            _ => return None,
        })
    }

    /// Seconds left on the wrong-PIN block, or None when unlock may proceed.
    fn unlock_block_remaining(&self) -> Option<u64> {
        let mut st = app_state().lock().unwrap();
        let until = st.unlock_blocked_until?;
        match until.checked_duration_since(std::time::Instant::now()) {
            Some(left) => Some(left.as_secs() + 1),
            None => {
                st.unlock_blocked_until = None;
                None
            }
        }
    }

    fn try_unlock(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        {
            let st = app_state().lock().unwrap();
            if st.must_confirm_device_role || st.device_role == DeviceRole::Unset {
                drop(st);
                self.go(cx, Screen::DeviceSetup);
                return;
            }
        }
        let (pin, lock_as_owner, lock_staff_id, has_store, owner_setup_complete, pin_reset) = {
            let st = app_state().lock().unwrap();
            (
                st.pin_buffer.clone(),
                st.lock_as_owner,
                st.lock_staff_id.clone(),
                st.store.is_some(),
                st.owner_setup_complete,
                st.pin_reset_required,
            )
        };
        if !has_store {
            self.set_status(cx, "Storage not ready — restart the app");
            return;
        }
        // Too many wrong PINs: sit out the block before trying again.
        if let Some(left) = self.unlock_block_remaining() {
            self.set_status(
                cx,
                format!("Too many wrong PINs — wait {left}s, then try again"),
            );
            return;
        }
        if pin.len() < 4 {
            self.set_status(cx, "Enter at least 4 digits, then OK");
            return;
        }

        // First-run Owner PIN setup (Server shop UX).
        if !owner_setup_complete {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let member = store.complete_owner_setup(&pin)?;
                let _ = store.set_active_session_staff_id(Some(&member.id));
                st.owner_setup_complete = true;
                st.onboarding_complete = false;
                st.onboard_step = 0;
                st.session = Some(member);
                st.pin_buffer.clear();
                st.pin_visible = false;
                st.lock_as_owner = true;
                st.screen = Screen::Onboard;
                st.status = "Owner ready — set up your shop".into();
                let _ = st.reload_all();
                Ok(())
            })();
            match result {
                Ok(()) => log!("Kumkaj owner setup ok"),
                Err(ref e) => {
                    log!("Kumkaj owner setup failed: {e}");
                    let mut st = app_state().lock().unwrap();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    drop(st);
                    self.set_status(cx, e.clone());
                }
            }
            self.sync_chrome(cx);
            return;
        }

        // The upgrade that replaced the PIN hasher handed every account back as
        // 0000. Reuse the first-run setup flow to get a real Owner PIN before
        // the till opens.
        if pin_reset {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let member = store.complete_pin_reset(&pin)?;
                let _ = store.set_active_session_staff_id(Some(&member.id));
                st.session = Some(member.clone());
                st.pin_buffer.clear();
                st.pin_visible = false;
                st.lock_as_owner = true;
                st.failed_unlocks = 0;
                st.screen = unlocked_home();
                st.status = format!("New Owner PIN set — welcome back, {}", member.lock_label());
                let _ = st.refresh_books_overview();
                let _ = st.reload_all();
                Ok(())
            })();
            match result {
                Ok(()) => log!("Kumkaj owner PIN reset ok"),
                Err(ref e) => {
                    log!("Kumkaj owner PIN reset failed: {e}");
                    let mut st = app_state().lock().unwrap();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    drop(st);
                    self.set_status(cx, e.clone());
                }
            }
            self.sync_chrome(cx);
            return;
        }

        // Auth runs without holding the app mutex (PIN verify must not freeze the UI).
        let auth = {
            let st = app_state().lock().unwrap();
            match st.store.as_ref() {
                Some(store) if lock_as_owner => store.authenticate_owner_pin(&pin),
                Some(store) => {
                    let staff_id = lock_staff_id.clone().or_else(|| {
                        st.staff
                            .iter()
                            .find(|s| !s.is_owner())
                            .map(|s| s.id.clone())
                    });
                    match staff_id {
                        Some(id) => store.authenticate_staff_pin(&id, &pin),
                        None => Ok(None),
                    }
                }
                None => Err("No store".into()),
            }
        };
        let result = {
            let mut st = app_state().lock().unwrap();
            if !st.lock_as_owner && st.lock_staff_id.is_none() {
                st.lock_staff_id = st
                    .staff
                    .iter()
                    .find(|s| !s.is_owner())
                    .map(|s| s.id.clone());
            }
            match auth {
                Ok(Some(member)) => {
                    if let Some(store) = st.store.as_ref() {
                        let _ = store.set_active_session_staff_id(Some(&member.id));
                    }
                    st.session = Some(member.clone());
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    st.failed_unlocks = 0;
                    st.unlock_blocked_until = None;
                    if st.device_role == DeviceRole::Client && !st.client_paired {
                        st.screen = Screen::ClientPair;
                        st.status = "Pair to the shop device before selling".into();
                    } else if st.device_role.is_host() && !st.onboarding_complete {
                        st.screen = Screen::Onboard;
                        st.status = format!("Welcome, {} — finish shop setup", member.lock_label());
                    } else {
                        st.screen = unlocked_home();
                        st.status = format!("Welcome, {}", member.lock_label());
                        let _ = st.refresh_books_overview();
                    }
                    if st.uses_lan()
                        && let Some(store) = st.store.as_ref() {
                            let code = store.pairing_code().unwrap_or_default();
                            let path = store.db_path().to_path_buf();
                            let _ = net::ensure_server(path, code);
                        }
                    Ok(())
                }
                Ok(None) => {
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    st.failed_unlocks = st.failed_unlocks.saturating_add(1);
                    if st.failed_unlocks >= UNLOCK_ATTEMPT_LIMIT {
                        st.failed_unlocks = 0;
                        st.unlock_blocked_until =
                            Some(std::time::Instant::now() + UNLOCK_BLOCK);
                        Err(format!(
                            "Too many wrong PINs — locked for {}s",
                            UNLOCK_BLOCK.as_secs()
                        ))
                    } else {
                        let left = UNLOCK_ATTEMPT_LIMIT - st.failed_unlocks;
                        Err(if lock_as_owner {
                            format!("Wrong Owner PIN — {left} tries left")
                        } else if st.lock_staff_id.is_none() {
                            "Add staff in Team first, or unlock as Owner".into()
                        } else {
                            format!("Wrong Staff PIN — {left} tries left")
                        })
                    }
                }
                Err(e) => Err(e),
            }
        };
        match result {
            Ok(()) => log!("Kumkaj unlock ok"),
            Err(ref e) => log!("Kumkaj unlock failed: {e}"),
        }
        match result {
            Ok(()) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn add_product_to_cart(&mut self, cx: &mut Cx, index: usize) {
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            if st.session.as_ref().map(|s| s.can_access_sell()) != Some(true) {
                Some("Not allowed to sell".into())
            } else if let Some(product) = st.products.get(index).cloned() {
                match st.cart.add_or_bump_in_stock(&product) {
                    Err(e) => Some(e),
                    Ok(()) => {
                        // Modifiers are a catalogue reminder, not a second screen
                        // to tap through: the seller writes the choice on the
                        // line's Note, which the kitchen ticket already carries.
                        let mods = product.modifiers();
                        if !mods.is_empty() {
                            st.status = format!(
                                "{} added · tap a modifier on the line",
                                product.name
                            );
                        }
                        None
                    }
                }
            } else {
                Some("Missing product".into())
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn handle_modifier_chips(
        &mut self,
        cx: &mut Cx,
        actions: &Actions,
        item: &WidgetRef,
        index: usize,
        amend: bool,
    ) {
        let slot = if item.button(cx, ids!(mod_0)).clicked(actions) {
            Some(0)
        } else if item.button(cx, ids!(mod_1)).clicked(actions) {
            Some(1)
        } else if item.button(cx, ids!(mod_2)).clicked(actions) {
            Some(2)
        } else {
            None
        };
        let page_delta = if item.button(cx, ids!(mod_next)).clicked(actions) {
            Some(1)
        } else if item.button(cx, ids!(mod_prev)).clicked(actions) {
            Some(-1)
        } else {
            None
        };
        if slot.is_none() && page_delta.is_none() {
            return;
        }
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let product_id = {
                let cart = if amend { &st.amend_cart } else { &st.cart };
                cart.lines
                    .get(index)
                    .map(|l| l.product_id.clone())
                    .ok_or_else(|| "Missing cart line".to_string())?
            };
            let mods = st
                .products
                .iter()
                .find(|p| p.id == product_id)
                .map(|p| p.modifiers())
                .unwrap_or_default();
            let cart = if amend {
                &mut st.amend_cart
            } else {
                &mut st.cart
            };
            if let Some(delta) = page_delta {
                cart.shift_modifier_page(index, delta, mods.len());
                return Ok(String::new());
            }
            let page = cart
                .lines
                .get(index)
                .map(|l| l.modifier_page)
                .unwrap_or(0);
            let pages = mods.len().div_ceil(domain::MODIFIER_PAGE).max(1);
            let start = (page % pages) * domain::MODIFIER_PAGE;
            let modifier = mods
                .get(start + slot.unwrap_or(0))
                .cloned()
                .ok_or_else(|| "No modifier there".to_string())?;
            let on = cart.toggle_modifier(index, &modifier)?;
            let price = st.money(modifier.price());
            Ok(if on {
                format!("{} added · {price}", modifier.name)
            } else {
                format!("{} removed", modifier.name)
            })
        })();
        match result {
            Ok(status) if !status.is_empty() => self.set_status(cx, status),
            Ok(_) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Cart `−`/`+`. Raising a tracked line stops at the on-hand count so the
    /// refusal lands on the button, not on the customer at the till.
    fn bump_cart_line(&mut self, cx: &mut Cx, index: usize, delta: i64) {
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            let blocked = if delta > 0 {
                match st.cart.lines.get(index).map(|l| l.product_id.clone()) {
                    Some(product_id) => st
                        .products
                        .iter()
                        .find(|p| p.id == product_id)
                        .filter(|p| p.tracks_stock())
                        .filter(|p| st.cart.qty_of(&product_id) + delta > p.stock_qty)
                        .map(|p| format!("Only {} of {} in stock", p.stock_qty, p.name)),
                    None => None,
                }
            } else {
                None
            };
            if blocked.is_none() {
                st.cart.bump_qty(index, delta);
            }
            blocked
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Single-tender charge (full bill one method). Sell UI uses amount fields +
    /// Take payment; kept for amend/automation paths that still need one tender.
    #[allow(dead_code)]
    fn pay(&mut self, cx: &mut Cx, method: PayMethod) {
        // A remote charge holds no lock while it waits on the LAN, so a second
        // tap would otherwise start a second bill for the same cart.
        if !self.begin_pay(cx) {
            return;
        }
        // Apply typed total even if seller didn't press Enter.
        let typed_total = self.ui.text_input(cx, ids!(cart_total_input)).text();
        let typed_customer = self.ui.text_input(cx, ids!(customer_name_input)).text();
        let result = (|| -> Result<(String, String), String> {
            enum Pending {
                LocalDone { sale: Sale, snack: String },
                Remote {
                    draft: domain::SaleDraft,
                    table_id: Option<String>,
                    table_name: Option<String>,
                    customer_id: Option<String>,
                    customer_name: Option<String>,
                    actor: StaffMember,
                    url: String,
                    code: String,
                },
            }
            let pending = {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_sell() {
                    return Err("Not allowed to sell".into());
                }
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
                let currency = st.locale.currency.clone();
                if st.cart.charge_total.is_some()
                    && let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                        && minor > 0 {
                            let _ = st.cart.set_charge_total(Money(minor));
                        }
                let mut draft = st.cart.to_sale_draft()?;
                draft.money_lane = st.money_lane;
                let table_id = st.selected_table_id.clone();
                let is_client = st.device_role == DeviceRole::Client;
                let customer_id = if method.is_credit() {
                    if let Some(id) = st.selected_customer_id.clone() {
                        Some(id)
                    } else if typed_customer.trim().is_empty() {
                        return Err("Type a customer name, then tap Credit".into());
                    } else if is_client {
                        // Leave the id unset so the Server resolves the name in
                        // its own customer book. Inventing one here would bill
                        // credit against a customer the Server has never heard of.
                        None
                    } else {
                        let name = typed_customer.trim();
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        let customer = store.find_or_create_customer(name)?;
                        st.selected_customer_id = Some(customer.id.clone());
                        Some(customer.id)
                    }
                } else {
                    None
                };
                if st.device_role == DeviceRole::Client {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    if !store.client_paired()? {
                        return Err("Client is not paired to a Server — connect first".into());
                    }
                    let url = store
                        .server_url()?
                        .ok_or_else(|| "Missing server URL".to_string())?;
                    let code = store.pairing_code()?;
                    let table_name = table_id.as_ref().and_then(|tid| {
                        st.tables.iter().find(|t| &t.id == tid).map(|t| t.name.clone())
                    });
                    let customer_name = if method.is_credit() {
                        Some(typed_customer.trim().to_string()).filter(|s| !s.is_empty())
                    } else {
                        None
                    };
                    Pending::Remote {
                        draft,
                        table_id,
                        table_name,
                        customer_id,
                        customer_name,
                        actor,
                        url,
                        code,
                    }
                } else {
                    let sale = {
                        let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                        store.finalize_sale(
                            &draft,
                            method,
                            table_id.as_deref(),
                            customer_id.as_deref(),
                            &actor,
                        )?
                    };
                    let amount = st.money(Money(sale.total_paise));
                    let tender = method.label_for(&st.fiscal);
                    let receipt = if sale.rounding_paise != 0 {
                        format!(
                            "Sale {} · Rounded {} (Applied {} · round {}) · {} · {}",
                            &sale.id[..8.min(sale.id.len())],
                            amount,
                            st.money(Money(sale.lines_total_paise)),
                            st.money(Money(sale.rounding_paise)),
                            tender,
                            &sale.created_at[..sale.created_at.len().min(19)]
                        )
                    } else {
                        format!(
                            "Sale {} · Rounded {} · {} · {}",
                            &sale.id[..8.min(sale.id.len())],
                            amount,
                            tender,
                            &sale.created_at[..sale.created_at.len().min(19)]
                        )
                    };
                    let full = st
                        .store
                        .as_ref()
                        .and_then(|s| s.format_receipt_text(&sale.id).ok())
                        .unwrap_or(receipt);
                    st.cart.clear();
                    st.selected_table_id = None;
                    if method.is_credit() {
                        st.selected_customer_id = None;
                    }
                    st.last_receipt = full;
                    st.status = format!("Recorded {}", method.label());
                    let snack = format!("{} · {} recorded", amount, method.label());
                    st.reload_all()?;
                    Pending::LocalDone { sale, snack }
                }
            };
            match pending {
                Pending::LocalDone { sale, snack } => Ok((sale.id, snack)),
                Pending::Remote {
                    draft,
                    table_id,
                    table_name,
                    customer_id,
                    customer_name,
                    actor,
                    url,
                    code,
                } => {
                    let req = net::RemoteSaleRequest {
                        draft,
                        pay: method.as_str().into(),
                        customer_id,
                        customer_name,
                        table_name,
                        table_id,
                        actor_id: actor.id.clone(),
                        payments: Vec::new(),
                    };
                    let sale = net::client_push_sale(&url, &code, &req)?;
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref()
                        && let Ok(snap) = net::client_sync(&url, &code)
                    {
                        let _ = net::apply_snapshot(store, &snap);
                    }
                    let amount = st.money(Money(sale.total_paise));
                    let receipt = st
                        .store
                        .as_ref()
                        .and_then(|s| s.format_receipt_text(&sale.id).ok())
                        .unwrap_or_else(|| {
                            format!(
                                "Sale {} · {} · {} (server)",
                                &sale.id[..8.min(sale.id.len())],
                                amount,
                                method.label()
                            )
                        });
                    st.cart.clear();
                    st.selected_table_id = None;
                    if method.is_credit() {
                        st.selected_customer_id = None;
                    }
                    st.last_receipt = receipt;
                    st.status = format!("Recorded {} on Server", method.label());
                    let snack = format!("{} · {} recorded", amount, method.label());
                    let sale_id = sale.id.clone();
                    let _ = st.reload_all();
                    Ok((sale_id, snack))
                }
            }
        })();
        self.end_pay();
        match result {
            Err(e) => {
                // The ledger line is what the seller is looking at. Pay
                // failures must show there, not only on the lock screen.
                app_state().lock().unwrap().last_receipt = e.clone();
                if e.contains("customer") {
                    self.ui
                        .text_input(cx, ids!(customer_name_input))
                        .set_key_focus(cx);
                }
                self.set_status(cx, e);
            }
            Ok((sale_id, snack)) => {
                if method.is_credit() {
                    self.ui
                        .text_input(cx, ids!(customer_name_input))
                        .set_text(cx, "");
                }
                self.show_undo_snackbar(cx, sale_id, snack);
            }
        }
        self.sync_chrome(cx);
    }

    /// Claim the charge slot. False means a charge is already running, and the
    /// caller must return without touching the cart.
    fn begin_pay(&mut self, cx: &mut Cx) -> bool {
        {
            let mut st = app_state().lock().unwrap();
            if st.pay_in_flight {
                st.status = "Charging — one moment".into();
            } else {
                st.pay_in_flight = true;
                return true;
            }
        }
        self.sync_chrome(cx);
        false
    }

    fn end_pay(&mut self) {
        app_state().lock().unwrap().pay_in_flight = false;
    }

    fn show_undo_snackbar(&mut self, cx: &mut Cx, sale_id: String, text: String) {
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
        }
        st.undo_sale_id = Some(sale_id);
        st.undo_snack_text = text;
        st.undo_timer = cx.start_timeout(10.0);
    }

    fn clear_undo_snackbar(&mut self, cx: &mut Cx) {
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
            st.undo_timer = Timer::empty();
        }
        st.undo_sale_id = None;
        st.undo_snack_text.clear();
    }

    fn undo_last_sale(&mut self, cx: &mut Cx) {
        let result = (|| -> Result<(), String> {
            let (is_client, url, code, sale_id, actor) = {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .undo_sale_id
                    .clone()
                    .ok_or_else(|| "Nothing to undo".to_string())?;
                let is_client = st.device_role == DeviceRole::Client;
                let url = st.store.as_ref().and_then(|s| s.server_url().ok()).flatten();
                let code = st.store.as_ref().and_then(|s| s.pairing_code().ok());
                (is_client, url, code, sale_id, actor)
            };
            if is_client {
                let url = url.ok_or_else(|| "Not paired".to_string())?;
                let code = code.ok_or_else(|| "Missing pairing code".to_string())?;
                net::client_push_undo(
                    &url,
                    &code,
                    &net::RemoteUndoRequest {
                        sale_id: sale_id.clone(),
                        actor_id: actor.id.clone(),
                    },
                )?;
                let mut st = app_state().lock().unwrap();
                if let Some(store) = st.store.as_ref()
                    && let Ok(snap) = net::client_sync(&url, &code) {
                        let _ = net::apply_snapshot(store, &snap);
                    }
                st.last_receipt = format!("Undone on Server {}", &sale_id[..8.min(sale_id.len())]);
                st.status = "Sale undone on Server".into();
                st.undo_sale_id = None;
                st.undo_snack_text.clear();
                let _ = st.reload_all();
                st.screen = Screen::Pos;
                return Ok(());
            }
            let mut st = app_state().lock().unwrap();
            let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
            let sale = store.quick_undo_sale(&sale_id, &actor)?;
            st.cart = Cart::from_sale(&sale);
            st.selected_table_id = sale.table_id.clone();
            st.selected_customer_id = sale.customer_id.clone();
            st.last_receipt = format!("Undone {}", &sale.id[..8.min(sale.id.len())]);
            st.status = "Sale undone — cart restored".into();
            st.undo_sale_id = None;
            st.undo_snack_text.clear();
            st.reload_all()?;
            st.screen = Screen::Pos;
            Ok(())
        })();
        {
            let mut st = app_state().lock().unwrap();
            if !st.undo_timer.is_empty() {
                cx.stop_timer(st.undo_timer);
                st.undo_timer = Timer::empty();
            }
        }
        if let Err(e) = result {
            self.set_status(cx, e);
            self.clear_undo_snackbar(cx);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_total_from_input(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, true);
    }

    /// Live Rounded edits must move totals immediately — waiting for blur
    /// leaves the field on the previous total and confuses the cashier.
    fn apply_cart_total_from_input_live(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, false);
    }

    fn apply_cart_total_from_input_inner(&mut self, cx: &mut Cx, report_errors: bool) {
        let total_field = self.ui.text_input(cx, ids!(cart_total_input));
        let text = total_field.text();
        let field_focused = cx.has_key_focus(total_field.area());
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.cart.is_empty() {
                return Err("Cart is empty".into());
            }
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(&text, &currency)?;
            if minor <= 0 {
                return Err("Rounded Total must be > 0".into());
            }
            let applied = st.cart.applied_total().paise();
            // Live `changed` can fire with a stale field after ADD. Ignore
            // mismatches when there is no explicit bill round and the seller
            // is not actively editing this field.
            if !report_errors
                && st.cart.charge_total.is_none()
                && minor != applied
                && !field_focused
            {
                return Ok(());
            }
            st.cart.set_charge_total(Money(minor))?;
            st.status = if st.cart.rounding().paise() == 0
                && st.cart.applied_total().paise() == st.cart.actual_total().paise()
            {
                "Rounded Total equals Applied Total".into()
            } else {
                format!(
                    "Actual {} · Applied {} · Rounded {} · round {}",
                    st.money(st.cart.actual_total()),
                    st.money(st.cart.applied_total()),
                    st.money(st.cart.rounded_total()),
                    st.money(st.cart.rounding())
                )
            };
            Ok(())
        })();
        if let Err(e) = result
            && report_errors {
                self.set_status(cx, e);
            }
        self.sync_chrome(cx);
    }

    fn apply_cart_unit_price(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_unit_price(index, Money(minor))?;
            st.status = format!(
                "Unit price set · line {}",
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.line_total())
                        .unwrap_or_default()
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_line_total(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_line_total(index, Money(minor))?;
            st.status = format!(
                "Line Applied {} · Actual {} · @ unchanged {}",
                st.money(Money(minor)),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.actual_total())
                        .unwrap_or(Money(0))
                ),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.unit_price)
                        .unwrap_or(Money(0))
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_note(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.cart.set_note(index, text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn clear_menu_form(&mut self, cx: &mut Cx) {
        for path in [
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_station_input),
            ids!(menu_modifiers_input),
        ] {
            self.ui.text_input(cx, path).set_text(cx, "");
        }
        let mut st = app_state().lock().unwrap();
        st.editing_product_id = None;
        st.draft_image_key = None;
    }

    /// Leave the item currently in the form and start a blank one.
    fn start_new_menu_item(&mut self, cx: &mut Cx) {
        self.clear_menu_form(cx);
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    /// Tap loads that item into the form. A different item replaces it.
    /// The same item stays as it is, so a second tap does not wipe unsaved edits.
    /// New item is the way back to adding.
    fn open_menu_item(&mut self, cx: &mut Cx, index: usize) {
        let loaded = {
            let st = app_state().lock().unwrap();
            let Some(product) = st.products.get(index).cloned() else {
                return;
            };
            if st.editing_product_id.as_deref() == Some(product.id.as_str()) {
                return;
            } else {
                let currency = st.locale.currency.clone();
                let store = st.store.as_ref();
                let tax = store
                    .map(|s| menu_tax_text(s, &product))
                    .unwrap_or_default();
                let category = store
                    .and_then(|s| s.list_categories().ok())
                    .map(|list| {
                        name_for_id(
                            list.iter().map(|row| (row.id.as_str(), row.name.as_str())),
                            &product.category_id,
                        )
                    })
                    .unwrap_or_default();
                let station = store
                    .and_then(|s| s.list_stations().ok())
                    .map(|list| {
                        name_for_id(
                            list.iter().map(|row| (row.id.as_str(), row.name.as_str())),
                            &product.station_id,
                        )
                    })
                    .unwrap_or_default();
                let aliases = product.aliases().join(", ");
                let modifiers = modifiers_as_typed(&product.modifiers(), &currency);
                let price = locale::format_major_input(product.price.paise(), &currency);
                Some((product, tax, category, station, aliases, modifiers, price))
            }
        };
        let Some((product, tax, category, station, aliases, modifiers, price)) = loaded else {
            return;
        };
        self.ui
            .text_input(cx, ids!(menu_name_input))
            .set_text(cx, &product.name);
        self.ui
            .text_input(cx, ids!(menu_aliases_input))
            .set_text(cx, &aliases);
        self.ui
            .text_input(cx, ids!(menu_price_input))
            .set_text(cx, &price);
        self.ui
            .text_input(cx, ids!(menu_unit_input))
            .set_text(cx, &product.unit_label);
        self.ui
            .text_input(cx, ids!(menu_tax_input))
            .set_text(cx, &tax);
        self.ui
            .text_input(cx, ids!(menu_category_input))
            .set_text(cx, &category);
        self.ui
            .text_input(cx, ids!(menu_station_input))
            .set_text(cx, &station);
        self.ui
            .text_input(cx, ids!(menu_modifiers_input))
            .set_text(cx, &modifiers);
        {
            let mut st = app_state().lock().unwrap();
            st.editing_product_id = Some(product.id.clone());
            st.draft_image_key = product.image_key.clone();
        }
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    fn add_menu_item(&mut self, cx: &mut Cx) {
        let name = self.ui.text_input(cx, ids!(menu_name_input)).text();
        let aliases_text = self.ui.text_input(cx, ids!(menu_aliases_input)).text();
        let price_text = self.ui.text_input(cx, ids!(menu_price_input)).text();
        let unit_text = self.ui.text_input(cx, ids!(menu_unit_input)).text();
        let tax_text = self.ui.text_input(cx, ids!(menu_tax_input)).text();
        let category_text = self.ui.text_input(cx, ids!(menu_category_input)).text();
        let station_text = self.ui.text_input(cx, ids!(menu_station_input)).text();
        let modifiers_text = self.ui.text_input(cx, ids!(menu_modifiers_input)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let updating = st.editing_product_id.is_some();
            st.require_server_role(if updating { "Change items" } else { "Add items" })?;
            let actor = st.actor()?;
            if !actor.role.can_manage_menu() {
                return Err("Not allowed to manage items".into());
            }
            let name = name.trim();
            if name.is_empty() {
                return Err("Name required".into());
            }
            let currency = st.locale.currency.clone();
            // Same parser the till uses, so `13,5` from a comma-decimal keyboard
            // prices an item exactly as it prices a bill.
            let price_minor = locale::parse_major_input(&price_text, &currency)?;
            if price_minor <= 0 {
                return Err("Price must be > 0".into());
            }
            let mut image_key = st.draft_image_key.clone();
            let editing_id = st.editing_product_id.clone();
            // Auto: leave None so display always re-detects from name.
            // Built-in / custom: persist chosen key.
            let product_id = editing_id.clone().unwrap_or_else(new_id);
            if let Some(key) = image_key.clone()
                && key.starts_with("pending_custom:") {
                    let src = std::path::PathBuf::from(&key["pending_custom:".len()..]);
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    image_key = Some(store.import_item_image(&product_id, &src)?);
                }
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            // Sellers type either a rate from their country pack ("Standard",
            // "GST 18%") or a bare percent, which becomes a named rate.
            let tax_rate = store.resolve_tax_rate(&tax_text)?;
            let category_id = match category_text.trim() {
                "" => String::new(),
                wanted => match store
                    .list_categories()?
                    .into_iter()
                    .find(|c| c.name.eq_ignore_ascii_case(wanted))
                {
                    Some(c) => c.id,
                    None => store.add_category(wanted)?.id,
                },
            };
            let station_id = match station_text.trim() {
                "" => String::new(),
                wanted => match store
                    .list_stations()?
                    .into_iter()
                    .find(|s| s.name.eq_ignore_ascii_case(wanted))
                {
                    Some(s) => s.id,
                    None => store.add_station(wanted)?.id,
                },
            };
            let modifiers =
                kumkaj::domain::ModifierOption::parse_human_list(&modifiers_text, &currency)?;
            let mut product = if let Some(id) = editing_id {
                store.get_product(&id)?.ok_or_else(|| {
                    "That item is no longer in Items".to_string()
                })?
            } else {
                Product::new_basic(product_id, name, Money(price_minor))
            };
            product.name = name.to_string();
            product.price = Money(price_minor);
            product.unit_label = unit_text.trim().to_string();
            product.image_key = image_key;
            product.set_aliases(&typed_aliases(&aliases_text));
            if let Some(rate) = &tax_rate {
                product.tax_rate_id = rate.id.clone();
                product.tax_bps = rate.bps;
            } else {
                product.tax_rate_id.clear();
                product.tax_bps = 0;
            }
            product.category_id = category_id;
            product.station_id = station_id;
            product.set_modifiers(&modifiers);
            store.upsert_product(&product)?;
            let verb = if updating { "menu.update" } else { "menu.add" };
            store.audit(&actor, verb, &product.name)?;
            st.draft_image_key = None;
            st.editing_product_id = None;
            st.reload_all()?;
            st.status = if updating {
                format!("Updated {}", product.name)
            } else {
                format!("Added {}", product.name)
            };
            Ok(())
        })();
        match result {
            Ok(()) => self.clear_menu_form(cx),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn remove_menu_item(&mut self, cx: &mut Cx, index: usize) {
        let was_editing = {
            let st = app_state().lock().unwrap();
            st.products.get(index).is_some_and(|product| {
                st.editing_product_id.as_deref() == Some(product.id.as_str())
            })
        };
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.require_server_role("Remove items")?;
            let actor = st.actor()?;
            if !actor.role.can_manage_menu() {
                return Err("Not allowed to manage items".into());
            }
            let product = st
                .products
                .get(index)
                .cloned()
                .ok_or_else(|| "Missing product".to_string())?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.archive_product(&product.id)?;
            store.audit(&actor, "menu.remove", &product.name)?;
            if st.editing_product_id.as_deref() == Some(product.id.as_str()) {
                st.editing_product_id = None;
                st.draft_image_key = None;
            }
            st.cart.lines.retain(|l| l.product_id != product.id);
            st.reload_all()?;
            st.status = format!("Removed {}", product.name);
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        } else if was_editing {
            self.clear_menu_form(cx);
        }
        self.sync_chrome(cx);
    }

    fn set_language(&mut self, cx: &mut Cx, code: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.language_follow_platform = false;
            st.locale.language = code.into();
            // On DeviceSetup, preview immediately; persist when Server/Client is chosen.
            // After unlock (Settings), persist as the fixed user language.
            if !st.must_confirm_device_role {
                if let Some(store) = st.store.as_ref() {
                    st.locale = store.set_user_language(code)?;
                }
            }
            st.status = format!("{}: {code}", st.t("language"));
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn set_currency(&mut self, cx: &mut Cx, code: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.locale.currency = code.into();
            if let Some(store) = st.store.as_ref() {
                store.set_locale_settings(&st.locale)?;
            }
            st.status = format!("Currency: {code}");
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn image_set_auto(&mut self, cx: &mut Cx) {
        app_state().lock().unwrap().draft_image_key = None;
        self.sync_chrome(cx);
    }

    fn image_cycle(&mut self, cx: &mut Cx, delta: isize) {
        let keys = menu_images::MENU_IMAGE_KEYS;
        let mut st = app_state().lock().unwrap();
        let idx = match st.draft_image_key.as_deref() {
            Some(k) if menu_images::is_builtin_key(k) && k != menu_images::PLACEHOLDER_KEY => keys
                .iter()
                .position(|x| *x == k)
                .unwrap_or(0),
            _ => 0,
        };
        let n = keys.len() as isize;
        let next = ((idx as isize + delta).rem_euclid(n)) as usize;
        st.draft_image_key = Some(keys[next].to_string());
        drop(st);
        self.sync_chrome(cx);
    }

    fn image_pick_file(&mut self, cx: &mut Cx) {
        // Native picker on every OS (Android SAF included). Do not use rfd —
        // it is desktop-only and left Android with a stub message.
        let dialog = FileDialog::new()
            .set_title("Choose item photo".into())
            .add_filter(
                "Images".into(),
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                    "gif".into(),
                    "bmp".into(),
                ],
            )
            .set_id(live_id!(item_photo_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening photo picker…");
    }

    /// Copy a Storage Access Framework URI into app storage. Real file paths
    /// are returned unchanged.
    fn materialize_picked_file(&mut self, cx: &mut Cx, path: &std::path::Path) -> Result<PathBuf, String> {
        let path_str = path.display().to_string();
        if !path_str.starts_with("content://") {
            return Ok(path.to_path_buf());
        }
        let bytes = cx
            .read_content_uri(&path_str)
            .ok_or_else(|| "Could not read that photo".to_string())?;
        let ext = image_extension(&bytes);
        let dir = cx
            .get_data_dir()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("picked_images");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let dest = dir.join(format!("photo-{}.{}", new_id(), ext));
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
        Ok(dest)
    }

    fn on_item_photo_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        match self.materialize_picked_file(cx, &path) {
            Ok(file) => {
                app_state().lock().unwrap().draft_image_key =
                    Some(format!("pending_custom:{}", file.display()));
                self.set_status(cx, format!("Photo selected: {}", file.display()));
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }
}

fn image_extension(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG") {
        "png"
    } else if bytes.starts_with(b"GIF8") {
        "gif"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else {
        "jpg"
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.sync_chrome(cx);
    }

    fn handle_timer(&mut self, cx: &mut Cx, e: &TimerEvent) {
        let is_ours = {
            let st = app_state().lock().unwrap();
            !st.undo_timer.is_empty() && st.undo_timer.is_timer(e).is_some()
        };
        if is_ours {
            self.clear_undo_snackbar(cx);
            self.sync_chrome(cx);
        }
    }

    fn handle_back_pressed(&mut self, cx: &mut Cx) -> bool {
        self.navigate_back(cx)
    }

    fn handle_key_down(&mut self, cx: &mut Cx, e: &KeyEvent) {
        if e.key_code == KeyCode::Escape
            || e.key_code == KeyCode::Back
            || (e.key_code == KeyCode::ArrowLeft && e.modifiers.alt)
        {
            if nav_gesture_ok() {
                let _ = self.navigate_back(cx);
            }
            return;
        }
        if e.key_code == KeyCode::ArrowRight && e.modifiers.alt {
            if nav_gesture_ok() {
                let _ = self.navigate_forward(cx);
            }
            return;
        }

        // Lock screen: digits (row + numpad), Enter = OK, Backspace = ⌫.
        let on_lock = app_state().lock().unwrap().screen == Screen::Lock;
        if on_lock && !e.is_repeat {
            if let Some(digit) = Self::pin_key_digit(e) {
                self.pin_digit(cx, digit);
                return;
            }
            if matches!(e.key_code, KeyCode::ReturnKey | KeyCode::NumpadEnter)
                && !e.modifiers.control
                && !e.modifiers.alt
                && !e.modifiers.logo
            {
                self.try_unlock(cx);
                return;
            }
            if e.key_code == KeyCode::Backspace
                && !e.modifiers.control
                && !e.modifiers.alt
                && !e.modifiers.logo
            {
                self.pin_backspace(cx);
            }
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for action in actions.iter() {
            if let Some(fd) = action.downcast_ref::<FileDialogAction>() {
                match fd {
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(item_photo_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_item_photo_picked(cx, path.clone());
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(venue_icon_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    app_state().lock().unwrap().draft_venue_icon_path = Some(file);
                                    self.set_status(cx, "Icon selected — tap Save brand");
                                    self.sync_chrome(cx);
                                }
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::FileCancelled { id }
                        if *id == live_id!(item_photo_pick)
                            || *id == live_id!(venue_icon_pick) =>
                    {
                        self.set_status(cx, "Picker cancelled");
                    }
                    _ => {}
                }
            }
        }
        if self.ui.button(cx, ids!(nav_books)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.refresh_books_overview()?;
                st.screen = Screen::Books;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(nav_sell)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Pos, |s| s.can_access_sell());
        }
        if self.ui.button(cx, ids!(nav_buy)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Buy, |s| {
                s.can_access_inventory() || s.can_access_sales() || s.is_owner()
            });
        }
        if self.ui.button(cx, ids!(nav_more)).clicked(actions) {
            self.go(cx, Screen::More);
        }
        if self.ui.button(cx, ids!(more_items)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Menu, |s| s.can_access_items());
        }
        if self.ui.button(cx, ids!(books_to_sell)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Pos, |s| s.can_access_sell());
        }
        if self.ui.button(cx, ids!(books_to_buy)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Buy, |s| {
                s.can_access_inventory() || s.can_access_sales() || s.is_owner()
            });
        }
        if self.ui.button(cx, ids!(books_refresh)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.refresh_books_overview()?;
                st.status = "Books refreshed".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(books_day_close)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.refresh_day_close_text()?;
                st.screen = Screen::DayClose;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(books_reports)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed".into());
                }
                let text = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .books_report_text("today")?;
                st.day_close_text = text;
                st.screen = Screen::Reports;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(books_credit)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Credit, |s| s.can_access_credit());
        }
        if self.ui.button(cx, ids!(buy_to_inventory)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed for your role".into());
                }
                st.refresh_inventory_text()?;
                st.screen = Screen::Inventory;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(buy_paid), ids!(buy_paid_on)) {
            app_state().lock().unwrap().buy_on_account = false;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(buy_on_account), ids!(buy_on_account_on)) {
            app_state().lock().unwrap().buy_on_account = true;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(buy_purchase)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(buy_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(buy_qty_input)).text();
            let cost_text = self.ui.text_input(cx, ids!(buy_cost_input)).text();
            let supplier = self.ui.text_input(cx, ids!(buy_supplier_input)).text();
            let party_kind = self.ui.text_input(cx, ids!(buy_party_kind)).text();
            let note = self.ui.text_input(cx, ids!(buy_note_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Record purchases")?;
                let actor = st.actor()?;
                let on_account = st.buy_on_account;
                let qty_milli = parse_qty_to_milli(&qty_text)?;
                let product = st.find_product_by_name(&name)?;
                let currency = st.locale.currency.clone();
                let unit_cost = match cost_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let mut party_id = String::new();
                if on_account || !supplier.trim().is_empty() {
                    if on_account {
                        let kind = if party_kind.trim().is_empty() {
                            "Supplier"
                        } else {
                            party_kind.trim()
                        };
                        let party = store.add_customer_kind(supplier.trim(), "", kind)?;
                        party_id = party.id;
                    }
                }
                let purchase = store.record_purchase_ex(
                    supplier.trim(),
                    &party_id,
                    on_account,
                    &[(product.id.clone(), qty_milli, unit_cost)],
                    note.trim(),
                    &actor,
                )?;
                st.reload_all()?;
                let _ = st.refresh_books_overview();
                let qty_show = domain::format_qty_display(qty_milli, 0, &product.unit_label);
                Ok(format!(
                    "Received {qty_show} {} · {}{}",
                    product.name,
                    st.money(Money(purchase.total_paise)),
                    if on_account { " · on account" } else { "" }
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(buy_qty_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(buy_cost_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(buy_expense_add)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(buy_expense_amount)).text();
            let note = self.ui.text_input(cx, ids!(buy_expense_note)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                if st.device_role == DeviceRole::Client {
                    return Err("Log expenses on the shop device — this is an extra till".into());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                store.add_expense(minor, "misc", &note, &actor)?;
                let _ = st.refresh_books_overview();
                st.status = "Expense logged".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(buy_expense_amount)).set_text(cx, "");
                self.ui.text_input(cx, ids!(buy_expense_note)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(more_tables)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Floor, |s| s.can_access_floor());
        }
        if self.ui.button(cx, ids!(more_kitchen)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Kitchen, |s| s.can_access_kitchen());
        }
        if self.ui.button(cx, ids!(more_sales)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Sales, |s| s.can_access_sales());
        }
        if self.ui.button(cx, ids!(more_day_close)).clicked(actions)
            || self.ui.button(cx, ids!(day_close_from_sales)).clicked(actions)
        {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.refresh_day_close_text()?;
                st.screen = Screen::DayClose;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(day_close_commit)).clicked(actions) {
            let counted_text = self.ui.text_input(cx, ids!(day_close_counted_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Record the day close")?;
                let actor = st.actor()?;
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to close the day".into());
                }
                let currency = st.locale.currency.clone();
                let counted = match counted_text.trim() {
                    "" => None,
                    other => Some(locale::parse_major_input(other, &currency)?),
                };
                let at = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .commit_day_close_with_count(&actor, counted)?;
                st.refresh_day_close_text()?;
                Ok(at)
            })();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(at) => {
                    let shown = at[..at.len().min(19)].replace('T', " ");
                    self.set_status(cx, format!("Day close recorded at {shown}"));
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(more_reports)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed".into());
                }
                let text = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .books_report_text("today")?;
                st.day_close_text = text;
                st.screen = Screen::Reports;
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(more_inventory)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed for your role".into());
                }
                st.refresh_inventory_text()?;
                st.screen = Screen::Inventory;
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(more_branches)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Not allowed for your role".into());
                }
                st.refresh_branches_text()?;
                st.screen = Screen::Branches;
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(more_credit)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Credit, |s| s.can_access_credit());
        }
        if self.ui.button(cx, ids!(more_team)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Team, |s| s.can_access_team() || s.is_owner());
        }
        if self.ui.button(cx, ids!(more_settings)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Settings, |s| s.can_access_backup() || s.is_owner());
        }
        for back_id in [
            ids!(screen_back),
            ids!(floor_back),
            ids!(kitchen_back),
            ids!(sales_back),
            ids!(credit_back),
            ids!(team_back),
            ids!(settings_back),
            ids!(day_close_back),
            ids!(reports_back),
            ids!(inventory_back),
            ids!(branches_back),
            ids!(client_pair_back),
        ] {
            if self.ui.button(cx, back_id).clicked(actions) {
                let _ = self.navigate_back(cx);
            }
        }
        if self.ui.button(cx, ids!(more_lock)).clicked(actions) {
            self.clear_undo_snackbar(cx);
            {
                let mut st = app_state().lock().unwrap();
                if let Some(store) = st.store.as_ref() {
                    let _ = store.set_active_session_staff_id(None);
                }
                st.session = None;
                st.screen = if st.must_confirm_device_role {
                    Screen::DeviceSetup
                } else {
                    Screen::Lock
                };
                st.pin_buffer.clear();
                st.status = "Locked".into();
            }
            self.sync_chrome(cx);
        }

        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(lang_setup_platform),
            ids!(lang_setup_platform_on),
        ) {
            let mut st = app_state().lock().unwrap();
            st.language_follow_platform = true;
            st.locale.language = locale::detect_platform_language();
            st.status = st.t("lang_follow_platform");
            drop(st);
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(lang_setup_user), ids!(lang_setup_user_on)) {
            let mut st = app_state().lock().unwrap();
            st.language_follow_platform = false;
            // Default the fixed pick to the current platform language.
            st.locale.language = locale::detect_platform_language();
            st.status = st.t("lang_pick_user");
            drop(st);
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(settings_lang_platform),
            ids!(settings_lang_platform_on),
        ) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change language")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                st.locale = store.set_platform_language()?;
                st.language_follow_platform = true;
                st.status = st.t("lang_follow_platform");
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(settings_lang_user),
            ids!(settings_lang_user_on),
        ) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change language")?;
                let platform = locale::detect_platform_language();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                // Enter user mode with platform language as the starting pick.
                st.locale = store.set_user_language(&platform)?;
                st.language_follow_platform = false;
                st.status = st.t("lang_pick_user");
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(device_role_server)).clicked(actions) {
            self.begin_server_mode(cx);
        }
        if self.ui.button(cx, ids!(device_role_client)).clicked(actions) {
            self.begin_client_mode(cx);
        }
        if self.ui.button(cx, ids!(server_create_new)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(server_name_input)).text();
            if let Err(e) = self.activate_new_server(cx, &name) {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(server_name_input))
                    .set_text(cx, "");
            }
        }
        if self.ui.button(cx, ids!(server_pick_cancel)).clicked(actions) {
            app_state().lock().unwrap().picking_server = false;
            self.sync_chrome(cx);
        }
        {
            let widget = self.ui.widget(cx, ids!(setup_server_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let id = app_state()
                        .lock()
                        .unwrap()
                        .local_servers
                        .get(item_id)
                        .map(|s| s.id.clone());
                    if let Some(id) = id {
                        if let Err(e) = self.activate_existing_server(cx, &id) {
                            self.set_status(cx, e);
                        }
                    }
                }
            }
        }
        if self.ui.button(cx, ids!(allow_extra_tills)).clicked(actions) {
            self.set_extra_tills(cx, true);
        }
        if self.ui.button(cx, ids!(stop_extra_tills)).clicked(actions) {
            self.set_extra_tills(cx, false);
        }
        for (idle, active, tmpl_id) in [
            (ids!(settings_tmpl_custom), ids!(settings_tmpl_custom_on), "custom"),
            (ids!(settings_tmpl_counter), ids!(settings_tmpl_counter_on), "counter_shop"),
            (ids!(settings_tmpl_courier), ids!(settings_tmpl_courier_on), "courier_agency"),
            (ids!(settings_tmpl_food), ids!(settings_tmpl_food_on), "food_service"),
            (ids!(settings_tmpl_personal), ids!(settings_tmpl_personal_on), "personal_books"),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, active) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Change start template")?;
                    let tmpl = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .apply_entity_template(tmpl_id)?;
                    st.entity_template_id = tmpl.id.to_string();
                    st.venue_mode = tmpl.venue_mode;
                    st.reload_all()?;
                    st.status = format!(
                        "Template: {} — {}",
                        tmpl.title, tmpl.blurb
                    );
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
                break;
            }
        }
        if self.ui.button(cx, ids!(settings_open_branches)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Branches, |s| s.can_access_branches() || s.is_owner());
        }

        for (id, digit) in [
            (ids!(pin_0), '0'),
            (ids!(pin_1), '1'),
            (ids!(pin_2), '2'),
            (ids!(pin_3), '3'),
            (ids!(pin_4), '4'),
            (ids!(pin_5), '5'),
            (ids!(pin_6), '6'),
            (ids!(pin_7), '7'),
            (ids!(pin_8), '8'),
            (ids!(pin_9), '9'),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                self.pin_digit(cx, digit);
            }
        }
        if self.ui.button(cx, ids!(pin_clear)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.pin_buffer.clear();
                if st.status.starts_with("Wrong")
                    || st.status.starts_with("Enter at least")
                    || st.status.starts_with("Storage")
                    || st.status.starts_with("Add staff")
                {
                    st.status.clear();
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(pin_reveal)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.pin_visible = !st.pin_visible;
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(pin_ok)).clicked(actions) {
            self.try_unlock(cx);
        }
        if self.ui.button(cx, ids!(role_owner)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.lock_as_owner = true;
                st.pin_buffer.clear();
                st.status = "Unlock as Owner".into();
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(role_staff)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                let has_staff = st.staff.iter().any(|s| !s.is_owner());
                if !st.owner_setup_complete || !has_staff {
                    st.lock_as_owner = true;
                    st.status = "Set up Owner first, then add staff in Team".into();
                } else {
                    st.lock_as_owner = false;
                    st.pin_buffer.clear();
                    if st.lock_staff_id.is_none() {
                        st.lock_staff_id = st
                            .staff
                            .iter()
                            .find(|s| !s.is_owner())
                            .map(|s| s.id.clone());
                    }
                    st.status = "Unlock as Staff".into();
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(lock_staff_prev)).clicked(actions)
            || self.ui.button(cx, ids!(lock_staff_next)).clicked(actions)
        {
            let forward = self.ui.button(cx, ids!(lock_staff_next)).clicked(actions);
            {
                let mut st = app_state().lock().unwrap();
                let ids: Vec<String> = st
                    .staff
                    .iter()
                    .filter(|s| !s.is_owner())
                    .map(|s| s.id.clone())
                    .collect();
                if !ids.is_empty() {
                    let cur = st
                        .lock_staff_id
                        .as_ref()
                        .and_then(|id| ids.iter().position(|x| x == id))
                        .unwrap_or(0);
                    let next = if forward {
                        (cur + 1) % ids.len()
                    } else {
                        (cur + ids.len() - 1) % ids.len()
                    };
                    st.lock_staff_id = Some(ids[next].clone());
                    st.pin_buffer.clear();
                }
            }
            self.sync_chrome(cx);
        }

        if self.choice_chip_clicked(cx, actions, ids!(lane_declared), ids!(lane_declared_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.money_lane = MoneyLane::Declared;
                st.cart.money_lane = MoneyLane::Declared;
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(lane_owner), ids!(lane_owner_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.money_lane = MoneyLane::Owner;
                st.cart.money_lane = MoneyLane::Owner;
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(pay_mix)).clicked(actions) && self.begin_pay(cx) {
            let typed_total = self.ui.text_input(cx, ids!(cart_total_input)).text();
            let cash_text = self.ui.text_input(cx, ids!(mix_cash_input)).text();
            let upi_text = self.ui.text_input(cx, ids!(mix_upi_input)).text();
            let card_text = self.ui.text_input(cx, ids!(mix_card_input)).text();
            let credit_text = self.ui.text_input(cx, ids!(mix_credit_input)).text();
            let typed_customer = self.ui.text_input(cx, ids!(customer_name_input)).text();
            enum Pending {
                Local { sale_id: String, snack: String },
                Remote {
                    req: Box<net::RemoteSaleRequest>,
                    url: String,
                    code: String,
                    summary: String,
                },
            }
            let parse_part = |text: &str, currency: &str| -> Result<i64, String> {
                let t = text.trim();
                if t.is_empty() {
                    return Ok(0);
                }
                locale::parse_major_input(t, currency)
            };
            let pending = (|| -> Result<Pending, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_sell() {
                    return Err("Not allowed to sell".into());
                }
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
                let currency = st.locale.currency.clone();
                if st.cart.charge_total.is_some()
                    && let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                        && minor > 0
                {
                    let _ = st.cart.set_charge_total(Money(minor));
                }
                let mut draft = st.cart.to_sale_draft()?;
                draft.money_lane = st.money_lane;
                let charged = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.preview_sale_total(&draft)?
                };
                let cash = parse_part(&cash_text, &currency)?;
                let upi = parse_part(&upi_text, &currency)?;
                let card = parse_part(&card_text, &currency)?;
                let credit = parse_part(&credit_text, &currency)?;
                if cash < 0 || upi < 0 || card < 0 || credit < 0 {
                    return Err("Payment amounts cannot be negative".into());
                }
                let mut parts: Vec<(PayMethod, i64)> = Vec::new();
                if cash > 0 {
                    parts.push((PayMethod::Cash, cash));
                }
                if upi > 0 {
                    parts.push((PayMethod::Upi, upi));
                }
                if card > 0 {
                    parts.push((PayMethod::Card, card));
                }
                if credit > 0 {
                    parts.push((PayMethod::Credit, credit));
                }
                if parts.is_empty() {
                    return Err(
                        "Type amounts in Cash / UPI / Card / Credit, then Take payment".into(),
                    );
                }
                let sum: i64 = parts.iter().map(|(_, a)| *a).sum();
                if sum != charged {
                    return Err(format!(
                        "Parts add to {} but bill is {} — adjust the amounts",
                        st.money(Money(sum)),
                        st.money(Money(charged))
                    ));
                }
                let is_client = st.device_role == DeviceRole::Client;
                let customer_id = if credit > 0 {
                    if let Some(id) = st.selected_customer_id.clone() {
                        Some(id)
                    } else if typed_customer.trim().is_empty() {
                        return Err(
                            "Credit part needs a customer — type a name or Pick one".into(),
                        );
                    } else if is_client {
                        None
                    } else {
                        let name = typed_customer.trim();
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        let customer = store.find_or_create_customer(name)?;
                        st.selected_customer_id = Some(customer.id.clone());
                        Some(customer.id)
                    }
                } else {
                    None
                };
                let customer_name = if credit > 0 {
                    let typed = typed_customer.trim();
                    if !typed.is_empty() {
                        Some(typed.to_string())
                    } else {
                        st.selected_customer_id.as_ref().and_then(|id| {
                            st.customers
                                .iter()
                                .find(|c| &c.id == id)
                                .map(|c| c.name.clone())
                        })
                    }
                } else {
                    None
                };
                let rail = st.fiscal.digital_pay_label().to_string();
                let summary = parts
                    .iter()
                    .map(|(m, a)| {
                        let label = match m {
                            PayMethod::Upi => rail.as_str(),
                            PayMethod::Cash => "Cash",
                            PayMethod::Card => "Card",
                            PayMethod::Credit => "Credit",
                            _ => m.label(),
                        };
                        format!("{} {}", label, st.money(Money(*a)))
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                let table_id = st.selected_table_id.clone();
                let pay_label = if parts.len() == 1 {
                    parts[0].0.as_str().to_string()
                } else {
                    PayMethod::Split.as_str().to_string()
                };
                if is_client {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    if !store.client_paired()? {
                        return Err("Client is not paired to a Server — connect first".into());
                    }
                    let url = store
                        .server_url()?
                        .ok_or_else(|| "Missing server URL".to_string())?;
                    let code = store.pairing_code()?;
                    let table_name = table_id.as_ref().and_then(|tid| {
                        st.tables.iter().find(|t| &t.id == tid).map(|t| t.name.clone())
                    });
                    let req = Box::new(net::RemoteSaleRequest {
                        draft,
                        pay: pay_label,
                        customer_id,
                        customer_name,
                        table_name,
                        table_id,
                        actor_id: actor.id.clone(),
                        payments: parts
                            .iter()
                            .map(|(method, amount)| net::RemoteTender {
                                pay: method.as_str().into(),
                                amount_paise: *amount,
                            })
                            .collect(),
                    });
                    return Ok(Pending::Remote {
                        req,
                        url,
                        code,
                        summary,
                    });
                }
                let sale = {
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    store.finalize_sale_tenders(
                        &draft,
                        &parts,
                        table_id.as_deref(),
                        customer_id.as_deref(),
                        &actor,
                    )?
                };
                let amount = st.money(Money(sale.total_paise));
                let full = st
                    .store
                    .as_ref()
                    .and_then(|s| s.format_receipt_text(&sale.id).ok())
                    .unwrap_or_else(|| format!("Sale {}", amount));
                st.cart.clear();
                st.selected_table_id = None;
                st.last_receipt = full;
                st.status = format!("Paid · {summary}");
                let snack = format!("{amount} · {summary}");
                st.reload_all()?;
                Ok(Pending::Local {
                    sale_id: sale.id,
                    snack,
                })
            })();
            let result = match pending {
                Err(e) => Err(e),
                Ok(Pending::Local { sale_id, snack }) => Ok((sale_id, snack)),
                Ok(Pending::Remote {
                    req,
                    url,
                    code,
                    summary,
                }) => (|| -> Result<(String, String), String> {
                    let sale = net::client_push_sale(&url, &code, &req)?;
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref()
                        && let Ok(snap) = net::client_sync(&url, &code)
                    {
                        let _ = net::apply_snapshot(store, &snap);
                    }
                    let amount = st.money(Money(sale.total_paise));
                    let full = st
                        .store
                        .as_ref()
                        .and_then(|s| s.format_receipt_text(&sale.id).ok())
                        .unwrap_or_else(|| format!("Sale {}", amount));
                    st.cart.clear();
                    st.selected_table_id = None;
                    st.last_receipt = full;
                    st.status = format!("Paid · {summary}");
                    st.reload_all()?;
                    Ok((sale.id, format!("{amount} · {summary}")))
                })()
            };
            self.end_pay();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok((sale_id, snack)) => {
                    for id in [
                        ids!(mix_cash_input),
                        ids!(mix_upi_input),
                        ids!(mix_card_input),
                        ids!(mix_credit_input),
                    ] {
                        self.ui.text_input(cx, id).set_text(cx, "");
                    }
                    self.show_undo_snackbar(cx, sale_id, snack);
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(send_kot)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
                let mut draft = st.cart.to_sale_draft()?;
                draft.money_lane = st.money_lane;
                let table_id = st.selected_table_id.clone();
                // The kitchen watches the Server's ticket list, so a terminal
                // posts the cart there instead of writing its own cache.
                if st.device_role == DeviceRole::Client {
                    let (url, code) = st.server_link()?;
                    let table_name = table_id.as_ref().and_then(|tid| {
                        st.tables.iter().find(|t| &t.id == tid).map(|t| t.name.clone())
                    });
                    let req = net::RemoteKotRequest {
                        draft,
                        table_name,
                        table_id: table_id.clone(),
                        actor_id: actor.id.clone(),
                    };
                    drop(st);
                    net::client_push_kot(&url, &code, &req)?;
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref()
                        && let Ok(snap) = net::client_sync(&url, &code) {
                            let _ = net::apply_snapshot(store, &snap);
                        }
                    st.cart.clear();
                    st.reload_all()?;
                    st.status = "Kitchen ticket sent to Server".into();
                    st.screen = Screen::Kitchen;
                    return Ok(());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let kot_id = store.send_kot_from_cart(&draft, table_id.as_deref(), &actor)?;
                // Park cart on table after sending to kitchen.
                if let Some(tid) = table_id.as_ref()
                    && let Ok(json) = serde_json::to_string(&draft) {
                        let _ = store.save_table_draft(tid, &json);
                    }
                st.reload_all()?;
                st.status = format!("Kitchen ticket {} sent", &kot_id[..8.min(kot_id.len())]);
                st.screen = Screen::Kitchen;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(undo_snack_btn)).clicked(actions) {
            self.undo_last_sale(cx);
        }
        if self.ui.button(cx, ids!(undo_snack_dismiss)).clicked(actions) {
            self.clear_undo_snackbar(cx);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(customer_use)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(customer_name_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                // The Server owns the customer book; a Client picks from what
                // sync brought down so both ends agree on the id.
                if st.device_role == DeviceRole::Client {
                    let needle = name.trim().to_lowercase();
                    let customer = st
                        .customers
                        .iter()
                        .find(|c| c.name.to_lowercase() == needle)
                        .cloned()
                        .ok_or_else(|| {
                            "No such customer here — add them on the Server, then Sync".to_string()
                        })?;
                    st.selected_customer_id = Some(customer.id);
                    st.status = format!("Credit customer: {}", customer.name);
                    return Ok(());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let customer = store.find_or_create_customer(&name)?;
                st.selected_customer_id = Some(customer.id.clone());
                st.status = format!("Credit customer: {}", customer.name);
                st.reload_all()?;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(customer_clear)).clicked(actions) {
            app_state().lock().unwrap().selected_customer_id = None;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(credit_add_btn)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(credit_add_name)).text();
            let kind = self.ui.text_input(cx, ids!(credit_add_kind)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add customers")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let customer = store.add_customer_kind(&name, "", kind.trim())?;
                st.selected_customer_id = Some(customer.id);
                st.reload_all()?;
                st.status = format!("Added {}", name);
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(credit_add_name))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(credit_add_kind))
                    .set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(credit_writeoff)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(writeoff_amount_input)).text();
            let reason = self.ui.text_input(cx, ids!(writeoff_reason_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Write off credit")?;
                let actor = st.actor()?;
                let customer_id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let currency = st.locale.currency.clone();
                let amount = match amount_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let written = st
                    .store
                    .as_mut()
                    .ok_or_else(|| "No store".to_string())?
                    .write_off_credit(&customer_id, amount, &reason, &actor)?;
                st.reload_all()?;
                let _ = st.refresh_books_overview();
                Ok(format!("Wrote off {}", st.money(Money(written))))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(writeoff_amount_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(writeoff_reason_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for (btn, method) in [
            (ids!(payable_cash), PayMethod::Cash),
            (ids!(payable_upi), PayMethod::Upi),
            (ids!(payable_card), PayMethod::Card),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(payable_amount_input)).text();
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Settle payable")?;
                    let actor = st.actor()?;
                    let party_id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a party first".to_string())?;
                    let currency = st.locale.currency.clone();
                    let amount = locale::parse_major_input(&amount_text, &currency)?;
                    st.store
                        .as_mut()
                        .ok_or_else(|| "No store".to_string())?
                        .settle_payable(&party_id, amount, method, &actor, "")?;
                    st.reload_all()?;
                    let _ = st.refresh_books_overview();
                    Ok(format!("AP paid {}", st.money(Money(amount))))
                })();
                match result {
                    Ok(msg) => {
                        self.ui
                            .text_input(cx, ids!(payable_amount_input))
                            .set_text(cx, "");
                        self.set_status(cx, msg);
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(payable_writeoff)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(writeoff_amount_input)).text();
            let reason = self.ui.text_input(cx, ids!(writeoff_reason_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Write off payable")?;
                let actor = st.actor()?;
                let party_id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let currency = st.locale.currency.clone();
                let amount = match amount_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let written = st
                    .store
                    .as_mut()
                    .ok_or_else(|| "No store".to_string())?
                    .write_off_payable(&party_id, amount, &reason, &actor)?;
                st.reload_all()?;
                let _ = st.refresh_books_overview();
                Ok(format!("AP wrote off {}", st.money(Money(written))))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }

        for list_id in [ids!(customer_list_sell), ids!(customer_list_credit)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let mut st = app_state().lock().unwrap();
                    if let Some(c) = st.customers.get(item_id).cloned() {
                        st.selected_customer_id = Some(c.id);
                        st.status = format!("Credit customer: {}", c.name);
                    }
                    drop(st);
                    self.sync_chrome(cx);
                }
            }
        }

        for (btn, method) in [
            (ids!(settle_cash), PayMethod::Cash),
            (ids!(settle_upi), PayMethod::Upi),
            (ids!(settle_card), PayMethod::Card),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(settle_amount_input)).text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    let customer_id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a customer first".to_string())?;
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&amount_text, &currency)?;
                    if st.device_role == DeviceRole::Client {
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        let url = store.server_url()?.ok_or_else(|| "Not paired".to_string())?;
                        let code = store.pairing_code()?;
                        let cust_name = st
                            .customers
                            .iter()
                            .find(|c| c.id == customer_id)
                            .map(|c| c.name.clone());
                        let req = net::RemoteSettleRequest {
                            customer_id: Some(customer_id.clone()),
                            customer_name: cust_name,
                            amount_paise: minor,
                            pay: method.as_str().into(),
                            actor_id: actor.id.clone(),
                            detail: String::new(),
                        };
                        drop(st);
                        net::client_push_settle(&url, &code, &req)?;
                        let mut st = app_state().lock().unwrap();
                        if let Some(store) = st.store.as_ref()
                            && let Ok(snap) = net::client_sync(&url, &code) {
                                let _ = net::apply_snapshot(store, &snap);
                            }
                        st.reload_all()?;
                        st.status = format!(
                            "Settled {} via {} on Server",
                            st.money(Money(minor)),
                            method.label()
                        );
                        return Ok(());
                    }
                    let bal_before = st
                        .customers
                        .iter()
                        .find(|c| c.id == customer_id)
                        .map(|c| c.balance_paise)
                        .unwrap_or(0);
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    store.settle_credit(&customer_id, minor, method, &actor, "")?;
                    st.reload_all()?;
                    let bal_after = st
                        .customers
                        .iter()
                        .find(|c| c.id == customer_id)
                        .map(|c| c.balance_paise)
                        .unwrap_or(0);
                    st.status = if bal_before <= 0 {
                        format!(
                            "Advance {} via {} · held {}",
                            st.money(Money(minor)),
                            method.label(),
                            st.money(Money((-bal_after).max(0)))
                        )
                    } else if bal_after < 0 {
                        format!(
                            "Paid {} via {} · debt cleared · advance {}",
                            st.money(Money(minor)),
                            method.label(),
                            st.money(Money(-bal_after))
                        )
                    } else if bal_after == 0 {
                        format!("Settled {} via {}", st.money(Money(minor)), method.label())
                    } else {
                        format!(
                            "Paid {} via {} · still owes {}",
                            st.money(Money(minor)),
                            method.label(),
                            st.money(Money(bal_after))
                        )
                    };
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.ui
                        .text_input(cx, ids!(settle_amount_input))
                        .set_text(cx, "");
                    // Surface settle/advance result on Sell receipt line too.
                    let msg = app_state().lock().unwrap().status.clone();
                    self.set_status(cx, msg);
                }
                self.sync_chrome(cx);
            }
        }

        if self.ui.button(cx, ids!(clear_cart)).clicked(actions) {
            app_state().lock().unwrap().cart.clear();
            self.sync_chrome(cx);
        }

        let total_input = self.ui.text_input(cx, ids!(cart_total_input));
        if total_input.changed(actions).is_some() {
            self.apply_cart_total_from_input_live(cx);
        }
        if total_input.returned(actions).is_some() || total_input.key_focus_lost(actions) {
            self.apply_cart_total_from_input(cx);
        }
        if self.ui.button(cx, ids!(menu_new)).clicked(actions) {
            self.start_new_menu_item(cx);
        }
        if self.ui.button(cx, ids!(menu_add)).clicked(actions) {
            self.add_menu_item(cx);
        }
        if self.ui.button(cx, ids!(image_auto)).clicked(actions) {
            self.image_set_auto(cx);
        }
        if self.ui.button(cx, ids!(image_prev)).clicked(actions) {
            self.image_cycle(cx, -1);
        }
        if self.ui.button(cx, ids!(image_next)).clicked(actions) {
            self.image_cycle(cx, 1);
        }
        if self.ui.button(cx, ids!(image_pick)).clicked(actions) {
            self.image_pick_file(cx);
        }
        if self.ui.button(cx, ids!(clear_table)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                if let Some(id) = st.selected_table_id.clone()
                    && st.device_role != DeviceRole::Client {
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.clear_table_draft(&id)?;
                    }
                st.selected_table_id = None;
                st.cart.clear();
                st.reload_all()?;
                st.status = "Counter — no table".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        // Sell list: 3-col token grid — each PortalList row holds c0/c1/c2.
        {
            let widget = self.ui.widget(cx, ids!(product_list));
            let list = widget.portal_list(cx, ids!(list));
            for (row_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(c0.add)).clicked(actions) {
                    self.add_product_to_cart(cx, row_id * 3);
                }
                if item.button(cx, ids!(c1.add)).clicked(actions) {
                    self.add_product_to_cart(cx, row_id * 3 + 1);
                }
                if item.button(cx, ids!(c2.add)).clicked(actions) {
                    self.add_product_to_cart(cx, row_id * 3 + 2);
                }
            }
        }
        // Items list: sold out / archive
        {
            let widget = self.ui.widget(cx, ids!(product_list_menu));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                let marked = item.button(cx, ids!(sold_out)).clicked(actions);
                let removed = item.button(cx, ids!(remove)).clicked(actions);
                if marked {
                    let result = (|| -> Result<(), String> {
                        let mut st = app_state().lock().unwrap();
                        st.require_server_role("Mark items sold out")?;
                        let product = st
                            .products
                            .get(item_id)
                            .cloned()
                            .ok_or_else(|| "Missing product".to_string())?;
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.set_product_available(&product.id, !product.available)?;
                        st.reload_all()?;
                        st.status = if product.available {
                            format!("{} marked sold out", product.name)
                        } else {
                            format!("{} restocked", product.name)
                        };
                        Ok(())
                    })();
                    if let Err(e) = result {
                        self.set_status(cx, e);
                    }
                    self.sync_chrome(cx);
                }
                if removed {
                    self.remove_menu_item(cx, item_id);
                } else if !marked
                    && item
                        .view(cx, ids!(card))
                        .finger_up(actions)
                        .is_some_and(|e| e.was_tap() && e.is_over)
                {
                    self.open_menu_item(cx, item_id);
                }
            }
        }

        for list_id in [ids!(language_list), ids!(setup_lang_list)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions)
                    && let Some(lang) = LANGUAGES.get(item_id) {
                        self.set_language(cx, lang.code);
                    }
            }
        }
        for list_id in [ids!(currency_list)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions)
                    && let Some(cur) = CURRENCIES.get(item_id) {
                        self.set_currency(cx, cur.code);
                    }
            }
        }

        let cart_widget = self.ui.widget(cx, ids!(cart_list));
        let cart_list = cart_widget.portal_list(cx, ids!(list));
        for (item_id, item) in cart_list.items_with_actions(actions) {
            if item.button(cx, ids!(minus)).clicked(actions) {
                self.bump_cart_line(cx, item_id, -1);
            }
            if item.button(cx, ids!(plus)).clicked(actions) {
                self.bump_cart_line(cx, item_id, 1);
            }
            let unit_input = item.text_input(cx, ids!(unit_price));
            if let Some((text, _)) = unit_input.returned(actions) {
                self.apply_cart_unit_price(cx, item_id, &text);
            } else if unit_input.key_focus_lost(actions) {
                let text = unit_input.text();
                self.apply_cart_unit_price(cx, item_id, &text);
            }
            let line_total_input = item.text_input(cx, ids!(line_total));
            if let Some((text, _)) = line_total_input.returned(actions) {
                self.apply_cart_line_total(cx, item_id, &text);
            } else if line_total_input.key_focus_lost(actions) {
                let text = line_total_input.text();
                self.apply_cart_line_total(cx, item_id, &text);
            }
            let note_input = item.text_input(cx, ids!(note));
            if let Some((text, _)) = note_input.returned(actions) {
                self.apply_cart_note(cx, item_id, &text);
            } else if note_input.key_focus_lost(actions) {
                let text = note_input.text();
                self.apply_cart_note(cx, item_id, &text);
            }
            self.handle_modifier_chips(cx, actions, &item, item_id, false);
        }

        let table_widget = self.ui.widget(cx, ids!(table_list));
        let table_list = table_widget.portal_list(cx, ids!(list));
        for (item_id, item) in table_list.items_with_actions(actions) {
            if item.button(cx, ids!(action)).clicked(actions) {
                let msg: Option<String> = {
                    let mut st = app_state().lock().unwrap();
                    if let Some(table) = st.tables.get(item_id).cloned() {
                        let prev = st.selected_table_id.clone();
                        let park_json = if prev.as_ref().map(|p| p != &table.id).unwrap_or(false)
                            && !st.cart.is_empty()
                        {
                            st.cart
                                .to_sale_draft()
                                .ok()
                                .and_then(|d| serde_json::to_string(&d).ok())
                        } else {
                            None
                        };
                        let draft_json = st
                            .store
                            .as_ref()
                            .and_then(|s| s.load_table_draft(&table.id).ok())
                            .flatten();
                        if let (Some(store), Some(prev), Some(json)) =
                            (st.store.as_ref(), prev.as_ref(), park_json.as_ref())
                        {
                            let _ = store.save_table_draft(prev, json);
                        }
                        if let Some(json) = draft_json.as_ref() {
                            if !json.trim().is_empty() && json.trim() != "{}" {
                                if let Ok(draft) =
                                    serde_json::from_str::<domain::SaleDraft>(json)
                                {
                                    let sale = Sale {
                                        id: String::new(),
                                        created_at: String::new(),
                                        lines_total_paise: draft.lines_total_paise,
                                        rounding_paise: draft.rounding_paise,
                                        total_paise: draft.total_paise,
                                        pay_method: String::new(),
                                        status: String::new(),
                                        table_id: Some(table.id.clone()),
                                        customer_id: None,
                                        created_by_id: String::new(),
                                        created_by_name: String::new(),
                                        lines: draft.lines.clone(),
                                        ..Sale::default()
                                    };
                                    st.cart = Cart::from_sale(&sale);
                                    if draft.rounding_paise != 0 {
                                        let _ = st
                                            .cart
                                            .set_charge_total(Money(draft.total_paise));
                                    }
                                }
                            } else if st.selected_table_id.as_deref() != Some(table.id.as_str())
                            {
                                st.cart.clear();
                            }
                        } else if st.selected_table_id.as_deref() != Some(table.id.as_str()) {
                            st.cart.clear();
                        }
                        if let Some(store) = st.store.as_ref() {
                            let _ = store.occupy_table(&table.id);
                        }
                        st.selected_table_id = Some(table.id.clone());
                        st.screen = Screen::Pos;
                        st.status = format!("Using table {}", table.name);
                        let _ = st.reload_all();
                        None
                    } else {
                        Some("Missing table".into())
                    }
                };
                if let Some(e) = msg {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }

        let kot_widget = self.ui.widget(cx, ids!(kot_list));
        let kot_list = kot_widget.portal_list(cx, ids!(list));
        for (item_id, item) in kot_list.items_with_actions(actions) {
            if item.button(cx, ids!(advance)).clicked(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    let kot_id = st
                        .kots
                        .get(item_id)
                        .map(|k| k.id.clone())
                        .ok_or_else(|| "Missing ticket".to_string())?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    if st.device_role == DeviceRole::Client {
                        let url = store.server_url()?.ok_or_else(|| "Not paired".to_string())?;
                        let code = store.pairing_code()?;
                        let actor_id = actor.id.clone();
                        drop(st);
                        net::client_push_kot_advance(
                            &url,
                            &code,
                            &net::RemoteKotAdvance {
                                kot_id: kot_id.clone(),
                                actor_id,
                            },
                        )?;
                        let mut st = app_state().lock().unwrap();
                        if let Some(store) = st.store.as_ref()
                            && let Ok(snap) = net::client_sync(&url, &code) {
                                let _ = net::apply_snapshot(store, &snap);
                            }
                        st.reload_all()?;
                        st.status = "Kitchen ticket updated on Server".into();
                        return Ok(());
                    }
                    store.advance_kot(&kot_id, &actor)?;
                    st.reload_all()?;
                    st.status = "Kitchen ticket updated".into();
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }

        let sale_widget = self.ui.widget(cx, ids!(sale_list));
        let sale_list = sale_widget.portal_list(cx, ids!(list));
        for (item_id, item) in sale_list.items_with_actions(actions) {
            if item.button(cx, ids!(pick)).clicked(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let sale_id = st
                        .recent_sales
                        .get(item_id)
                        .map(|s| s.id.clone())
                        .ok_or_else(|| "Missing sale".to_string())?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let sale = store
                        .get_sale(&sale_id)?
                        .ok_or_else(|| "Sale not found".to_string())?;
                    let revisions = store.list_sale_revisions(&sale_id)?;
                    let pay = PayMethod::parse(&sale.pay_method).unwrap_or(PayMethod::Cash);
                    let short = sale.id[..8.min(sale.id.len())].to_string();
                    st.amend_cart = Cart::from_sale(&sale);
                    st.amend_pay = pay;
                    st.sale_revisions = revisions;
                    st.selected_sale_id = Some(sale.id);
                    st.status = format!("Selected sale {short} — edit lines/total, then Amend");
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }

        let amend_widget = self.ui.widget(cx, ids!(amend_cart_list));
        let amend_list = amend_widget.portal_list(cx, ids!(list));
        for (item_id, item) in amend_list.items_with_actions(actions) {
            if item.button(cx, ids!(minus)).clicked(actions) {
                app_state()
                    .lock()
                    .unwrap()
                    .amend_cart
                    .bump_qty(item_id, -1);
                self.sync_chrome(cx);
            }
            if item.button(cx, ids!(plus)).clicked(actions) {
                app_state()
                    .lock()
                    .unwrap()
                    .amend_cart
                    .bump_qty(item_id, 1);
                self.sync_chrome(cx);
            }
            let unit_input = item.text_input(cx, ids!(unit_price));
            if let Some((text, _)) = unit_input.returned(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_unit_price(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            } else if unit_input.key_focus_lost(actions) {
                let text = unit_input.text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_unit_price(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            let line_total_input = item.text_input(cx, ids!(line_total));
            if let Some((text, _)) = line_total_input.returned(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_line_total(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            } else if line_total_input.key_focus_lost(actions) {
                let text = line_total_input.text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_line_total(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            let note_input = item.text_input(cx, ids!(note));
            if let Some((text, _)) = note_input.returned(actions) {
                let result = (|| -> Result<(), String> {
                    app_state().lock().unwrap().amend_cart.set_note(item_id, text)?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            } else if note_input.key_focus_lost(actions) {
                let text = note_input.text();
                let result = (|| -> Result<(), String> {
                    app_state().lock().unwrap().amend_cart.set_note(item_id, text)?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            self.handle_modifier_chips(cx, actions, &item, item_id, true);
        }

        if self.ui.button(cx, ids!(amend_pay_cash)).clicked(actions) {
            app_state().lock().unwrap().amend_pay = PayMethod::Cash;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(amend_pay_upi)).clicked(actions) {
            app_state().lock().unwrap().amend_pay = PayMethod::Upi;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(amend_pay_card)).clicked(actions) {
            app_state().lock().unwrap().amend_pay = PayMethod::Card;
            self.sync_chrome(cx);
        }

        let amend_total = self.ui.text_input(cx, ids!(amend_total_input));
        let amend_total_dirty = amend_total.changed(actions).is_some()
            || amend_total.returned(actions).is_some()
            || amend_total.key_focus_lost(actions);
        if amend_total_dirty {
            let text = amend_total.text();
            let report_errors = amend_total.returned(actions).is_some()
                || amend_total.key_focus_lost(actions);
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                if st.amend_cart.is_empty() {
                    return Ok(());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&text, &currency)?;
                st.amend_cart.set_charge_total(Money(minor))?;
                Ok(())
            })();
            if let Err(e) = result
                && report_errors {
                    self.set_status(cx, e);
                }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(amend_btn)).clicked(actions) {
            let reason = self.ui.text_input(cx, ids!(reason_input)).text();
            let typed_total = self.ui.text_input(cx, ids!(amend_total_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.role.can_amend_sale() {
                    return Err("Not allowed to amend sales".into());
                }
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                let currency = st.locale.currency.clone();
                if let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                    && minor > 0 {
                        let _ = st.amend_cart.set_charge_total(Money(minor));
                    }
                let draft = st.amend_cart.to_sale_draft()?;
                let pay = st.amend_pay;
                // The Server holds the sale and its revision history; a Client's
                // copy is a cache the next sync overwrites.
                if st.device_role == DeviceRole::Client {
                    let (url, code) = st.server_link()?;
                    let req = net::RemoteAmendRequest {
                        sale_id: sale_id.clone(),
                        draft,
                        pay: pay.as_str().into(),
                        reason: reason.clone(),
                        actor_id: actor.id.clone(),
                    };
                    drop(st);
                    let sale = net::client_push_amend(&url, &code, &req)?;
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref()
                        && let Ok(snap) = net::client_sync(&url, &code) {
                            let _ = net::apply_snapshot(store, &snap);
                        }
                    st.amend_cart = Cart::from_sale(&sale);
                    st.sale_revisions = st
                        .store
                        .as_ref()
                        .and_then(|s| s.list_sale_revisions(&sale_id).ok())
                        .unwrap_or_default();
                    st.reload_all()?;
                    st.status = format!(
                        "Amended on Server · Rounded Total {}",
                        st.money(Money(sale.total_paise))
                    );
                    return Ok(());
                }
                let (sale, revisions) = {
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let sale = store.amend_sale(&sale_id, &draft, pay, &reason, &actor)?;
                    let revisions = store.list_sale_revisions(&sale_id)?;
                    (sale, revisions)
                };
                st.amend_cart = Cart::from_sale(&sale);
                st.sale_revisions = revisions;
                st.reload_all()?;
                st.status = format!(
                    "Amended sale · Rounded Total {} · history kept",
                    st.money(Money(sale.total_paise))
                );
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(void_btn)).clicked(actions) {
            let reason = self.ui.text_input(cx, ids!(reason_input)).text();
            let pin = self.ui.text_input(cx, ids!(approval_pin_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let (approved_actor, reason) = approve_for(store, &actor, &pin, &reason)?;
                if st.device_role == DeviceRole::Client {
                    let url = store.server_url()?.ok_or_else(|| "Not paired".to_string())?;
                    let code = store.pairing_code()?;
                    drop(st);
                    net::client_push_void(
                        &url,
                        &code,
                        &net::RemoteSaleAction {
                            sale_id: sale_id.clone(),
                            actor_id: approved_actor.id.clone(),
                            reason: reason.clone(),
                        },
                    )?;
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref()
                        && let Ok(snap) = net::client_sync(&url, &code) {
                            let _ = net::apply_snapshot(store, &snap);
                        }
                    st.reload_all()?;
                    st.status = "Sale voided on Server".into();
                    return Ok(());
                }
                let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                store.void_sale(&sale_id, &reason, &approved_actor)?;
                st.reload_all()?;
                st.status = if approved_actor.id == actor.id {
                    "Sale voided".into()
                } else {
                    format!("Sale voided · approved by {}", approved_actor.name)
                };
                Ok(())
            })();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(()) => self
                    .ui
                    .text_input(cx, ids!(approval_pin_input))
                    .set_text(cx, ""),
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(refund_btn)).clicked(actions) {
            let reason = self.ui.text_input(cx, ids!(reason_input)).text();
            let pin = self.ui.text_input(cx, ids!(approval_pin_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let (approved_actor, reason) = approve_for(store, &actor, &pin, &reason)?;
                if st.device_role == DeviceRole::Client {
                    let url = store.server_url()?.ok_or_else(|| "Not paired".to_string())?;
                    let code = store.pairing_code()?;
                    drop(st);
                    net::client_push_refund(
                        &url,
                        &code,
                        &net::RemoteSaleAction {
                            sale_id: sale_id.clone(),
                            actor_id: approved_actor.id.clone(),
                            reason: reason.clone(),
                        },
                    )?;
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref()
                        && let Ok(snap) = net::client_sync(&url, &code) {
                            let _ = net::apply_snapshot(store, &snap);
                        }
                    st.reload_all()?;
                    st.status = "Sale refunded on Server".into();
                    return Ok(());
                }
                let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                store.refund_sale(&sale_id, &reason, &approved_actor)?;
                st.reload_all()?;
                st.status = if approved_actor.id == actor.id {
                    "Sale refunded".into()
                } else {
                    format!("Sale refunded · approved by {}", approved_actor.name)
                };
                Ok(())
            })();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(()) => self
                    .ui
                    .text_input(cx, ids!(approval_pin_input))
                    .set_text(cx, ""),
            }
            self.sync_chrome(cx);
        }

        if self.choice_chip_clicked(cx, actions, ids!(staff_role_cashier), ids!(staff_role_cashier_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Cashier;
                st.draft_features = StaffFeatures::for_role(StaffRole::Cashier);
            }
            self.set_status(cx, "Preset: Cashier");
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(staff_role_waiter), ids!(staff_role_waiter_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Waiter;
                st.draft_features = StaffFeatures::for_role(StaffRole::Waiter);
            }
            self.set_status(cx, "Preset: Waiter");
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(staff_role_manager), ids!(staff_role_manager_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Manager;
                st.draft_features = StaffFeatures::for_role(StaffRole::Manager);
            }
            self.set_status(cx, "Preset: Manager");
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(staff_role_kitchen), ids!(staff_role_kitchen_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Kitchen;
                st.draft_features = StaffFeatures::for_role(StaffRole::Kitchen);
            }
            self.set_status(cx, "Preset: Kitchen");
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(staff_add_cashier)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(staff_name_input)).text();
            let title = self.ui.text_input(cx, ids!(staff_title_input)).text();
            let pin = self.ui.text_input(cx, ids!(staff_pin_input)).text();
            let login = self.ui.text_input(cx, ids!(staff_login_input)).text();
            let password = self.ui.text_input(cx, ids!(staff_password_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add staff")?;
                if !st.owner_setup_complete {
                    return Err("Finish Owner PIN setup before adding staff".into());
                }
                let actor = st.actor()?;
                let role = st.draft_staff_role;
                let features = st.draft_features.clone();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_staff_with_features(
                    &store::StaffHire {
                        name: &name,
                        role,
                        role_title: &title,
                        pin: &pin,
                        login_id: &login,
                        password: &password,
                        features,
                    },
                    &actor,
                )?;
                st.reload_all()?;
                st.team_staff_id = st.staff.iter().rev().find(|s| !s.is_owner()).map(|s| s.id.clone());
                st.status = format!("Added {}", name.trim());
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(staff_name_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_title_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_pin_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_login_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_password_input)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(staff_set_login)).clicked(actions) {
            let login = self.ui.text_input(cx, ids!(staff_login_input)).text();
            let password = self.ui.text_input(cx, ids!(staff_password_input)).text();
            let result = (|| -> Result<(), String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Change staff logins")?;
                let actor = st.actor()?;
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| "Add or pick a staff member first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_staff_login(&staff_id, &login, &password, &actor)?;
                Ok(())
            })();
            match result {
                Ok(()) => self.set_status(cx, "Login saved"),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(staff_deactivate)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Remove staff")?;
                let actor = st.actor()?;
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| "Select a staff member first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.deactivate_staff(&staff_id, &actor)?;
                st.team_staff_id = None;
                st.reload_all()?;
                st.status = "Staff removed".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(change_my_pin)).clicked(actions) {
            let pin = self.ui.text_input(cx, ids!(my_pin_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change your PIN")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_staff_pin(&actor.id, &pin, &actor)?;
                st.status = "PIN updated".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(my_pin_input)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(change_my_password)).clicked(actions) {
            let password = self.ui.text_input(cx, ids!(my_password_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change your password")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_own_password(&password, &actor)?;
                st.status = "Password updated".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(my_password_input)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        for (id, toggle) in [
            (ids!(feat_sell), "sell"),
            (ids!(feat_items), "items"),
            (ids!(feat_sales), "sales"),
            (ids!(feat_credit), "credit"),
            (ids!(feat_floor), "floor"),
            (ids!(feat_kitchen), "kitchen"),
            (ids!(feat_team), "team"),
            (ids!(feat_backup), "backup"),
            (ids!(feat_inventory), "inventory"),
            (ids!(feat_branches), "branches"),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                let mut st = app_state().lock().unwrap();
                match toggle {
                    "sell" => st.draft_features.sell = !st.draft_features.sell,
                    "items" => st.draft_features.items = !st.draft_features.items,
                    "sales" => st.draft_features.sales = !st.draft_features.sales,
                    "credit" => st.draft_features.credit = !st.draft_features.credit,
                    "floor" => st.draft_features.floor = !st.draft_features.floor,
                    "kitchen" => st.draft_features.kitchen = !st.draft_features.kitchen,
                    "team" => st.draft_features.team = !st.draft_features.team,
                    "backup" => st.draft_features.backup = !st.draft_features.backup,
                    "inventory" => st.draft_features.inventory = !st.draft_features.inventory,
                    "branches" => st.draft_features.branches = !st.draft_features.branches,
                    _ => {}
                }
                st.status = format!("Features draft: {}", st.draft_features.to_json());
                drop(st);
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(feat_save)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change staff permissions")?;
                let actor = st.actor()?;
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .or_else(|| st.staff.iter().find(|s| !s.is_owner()).map(|s| s.id.clone()))
                    .ok_or_else(|| "Add staff first".to_string())?;
                let features = st.draft_features.clone();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_staff_features(&staff_id, &features, &actor)?;
                st.reload_all()?;
                st.status = "Staff features saved".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(mode_counter)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_venue_mode(VenueMode::Counter)?;
                st.venue_mode = VenueMode::Counter;
                st.status = "Tables & kitchen off".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(mode_restaurant)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_venue_mode(VenueMode::Restaurant)?;
                st.venue_mode = VenueMode::Restaurant;
                st.status = "Tables & kitchen on".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        for (id, step) in [(ids!(country_prev), -1i64), (ids!(country_next), 1i64)] {
            if self.ui.button(cx, id).clicked(actions) {
                let typed = self.ui.text_input(cx, ids!(country_code_input)).text();
                let next = {
                    let mut st = app_state().lock().unwrap();
                    let from = if typed.trim().is_empty() {
                        st.fiscal.country_code.clone()
                    } else {
                        typed
                    };
                    let next = step_country(&from, step, &st.fiscal.country_code);
                    st.country_browse = next.code.to_string();
                    st.status = format!("{} ({})", next.name_en, next.code);
                    next
                };
                self.ui
                    .text_input(cx, ids!(country_code_input))
                    .set_text(cx, next.code);
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(country_apply)).clicked(actions) {
            let code = self.ui.text_input(cx, ids!(country_code_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change the country pack")?;
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Only the Owner can change the country pack".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let settings = store.apply_country_pack(&code)?;
                store.audit(
                    &actor,
                    "fiscal.country",
                    &format!(
                        "{} {} {}",
                        settings.country_code,
                        settings.profile().tax_system.as_str(),
                        settings.tax_pricing.as_str()
                    ),
                )?;
                let name = settings.profile().name_en;
                st.reload_all()?;
                st.country_browse.clear();
                st.status = format!("{name} pack applied — check rates on Items");
                Ok(name.to_string())
            })();
            match result {
                Ok(_) => self
                    .ui
                    .text_input(cx, ids!(country_code_input))
                    .set_text(cx, ""),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for (id, pricing) in [
            (ids!(tax_inclusive_btn), TaxPricing::Inclusive),
            (ids!(tax_exclusive_btn), TaxPricing::Exclusive),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Change tax pricing")?;
                    let actor = st.actor()?;
                    if !actor.is_owner() {
                        return Err("Only the Owner can change tax pricing".into());
                    }
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.set_tax_pricing(pricing)?;
                    st.reload_all()?;
                    st.status = format!("{} — {}", st.fiscal.tax_label(), pricing.label());
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(fiscal_save)).clicked(actions) {
            let id_text = self.ui.text_input(cx, ids!(business_tax_id_input)).text();
            let service_text = self.ui.text_input(cx, ids!(service_bps_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Set the business id")?;
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Only the Owner can set the business id".into());
                }
                let service_bps = locale::parse_percent_bps(&service_text)?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_business_tax_id(&id_text)?;
                store.set_service_charge_bps(service_bps)?;
                st.reload_all()?;
                let id_status = if st.fiscal.business_tax_id.is_empty() {
                    format!("{} cleared", st.fiscal.business_id_label)
                } else {
                    format!("{} saved — it prints on every bill", st.fiscal.business_id_label)
                };
                st.status = if st.fiscal.service_charge_bps > 0 {
                    format!(
                        "{id_status} · service {}% added to every bill",
                        locale::format_percent_bps(
                            st.fiscal.service_charge_bps,
                            st.fiscal.decimal_sep()
                        )
                    )
                } else {
                    id_status
                };
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(onboard_save_venue)).clicked(actions) {
            let title = self.ui.text_input(cx, ids!(onboard_venue_input)).text();
            let org = self.ui.text_input(cx, ids!(onboard_org_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_venue_title(&title)?;
                let org = org.trim();
                if !org.is_empty() {
                    store.set_org_name(org)?;
                }
                st.venue_title = title.trim().to_string();
                st.status = if org.is_empty() {
                    "Shop name saved".into()
                } else {
                    "Shop and organization saved".into()
                };
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        for (idle, active, tmpl_id) in [
            (ids!(onboard_tmpl_custom), ids!(onboard_tmpl_custom_on), "custom"),
            (ids!(onboard_tmpl_counter), ids!(onboard_tmpl_counter_on), "counter_shop"),
            (ids!(onboard_tmpl_courier), ids!(onboard_tmpl_courier_on), "courier_agency"),
            (ids!(onboard_tmpl_food), ids!(onboard_tmpl_food_on), "food_service"),
            (ids!(onboard_tmpl_personal), ids!(onboard_tmpl_personal_on), "personal_books"),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, active) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let tmpl = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .apply_entity_template(tmpl_id)?;
                    st.entity_template_id = tmpl.id.to_string();
                    st.venue_mode = tmpl.venue_mode;
                    st.reload_all()?;
                    st.status = format!("Template: {} — {}", tmpl.title, tmpl.blurb);
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
                break; // one chip per event — never leave the wrong template selected
            }
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(onboard_fresh),
            ids!(onboard_fresh_on),
        ) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.onboard_step = 1;
                st.status = "Fresh selected — empty catalogue; tap Done — open books when ready".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(onboard_book), ids!(onboard_book_on)) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.onboard_step = 2;
                st.status =
                    "Book selected — add opening rates / credit below, then Done — open books".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(onboard_add_item)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(onboard_item_name)).text();
            let price = self.ui.text_input(cx, ids!(onboard_item_price)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&price, &currency).unwrap_or(0);
                if name.trim().is_empty() || minor <= 0 {
                    return Err("Item name and price required".into());
                }
                let p = Product {
                    id: new_id(),
                    name: name.trim().into(),
                    price: Money(minor),
                    available: true,
                    archived: false,
                    image_key: None,
                    ..Product::default()
                };
                st.store.as_ref().ok_or_else(|| "No store".to_string())?.upsert_product(&p)?;
                st.reload_all()?;
                st.status = format!("Added {}", p.name);
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            else {
                self.ui.text_input(cx, ids!(onboard_item_name)).set_text(cx, "");
                self.ui.text_input(cx, ids!(onboard_item_price)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(onboard_add_credit)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(onboard_cust_name)).text();
            let bal = self.ui.text_input(cx, ids!(onboard_cust_bal)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&bal, &currency)?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let customer = store.find_or_create_customer(name.trim())?;
                store.seed_opening_credit(&customer.id, minor, &actor)?;
                st.reload_all()?;
                st.status = format!("Opening credit for {}", customer.name);
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            else {
                self.ui.text_input(cx, ids!(onboard_cust_name)).set_text(cx, "");
                self.ui.text_input(cx, ids!(onboard_cust_bal)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(onboard_done)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.store.as_ref().ok_or_else(|| "No store".to_string())?.complete_onboarding()?;
                st.onboarding_complete = true;
                st.screen = unlocked_home();
                st.status = "Shop ready — your books are open".into();
                let _ = st.reload_all();
                let _ = st.refresh_books_overview();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(client_pair_btn)).clicked(actions) {
            let host = self.ui.text_input(cx, ids!(client_host_input)).text();
            let code = self.ui.text_input(cx, ids!(client_code_input)).text();
            let result = (|| -> Result<(), String> {
                let (base, snap) = net::client_pair(&host, code.trim())?;
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_client_pairing(&base, code.trim())?;
                net::apply_snapshot(store, &snap)?;
                st.client_paired = true;
                st.reload_all()?;
                st.status = format!("Paired to {} ({})", base, snap.venue_title);
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_sync_btn)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let url = store.server_url()?.ok_or_else(|| "Not paired".to_string())?;
                let code = store.pairing_code()?;
                drop(st);
                let snap = net::client_sync(&url, &code)?;
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                net::apply_snapshot(store, &snap)?;
                st.reload_all()?;
                st.status = "Synced from Server".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_unpair)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.clear_client_pairing()?;
                st.client_paired = false;
                st.status = "Forgot the Server — pair again to sell".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_skip_lock)).clicked(actions) {
            if app_state().lock().unwrap().client_paired {
                self.go(cx, Screen::Lock);
            } else {
                self.set_status(cx, "Pair first");
            }
        }

        if self.ui.button(cx, ids!(floor_add_table)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(floor_table_name)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add tables")?;
                st.store.as_ref().ok_or_else(|| "No store".to_string())?.add_table(&name)?;
                st.reload_all()?;
                st.status = format!("Added table {}", name.trim());
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            else { self.ui.text_input(cx, ids!(floor_table_name)).set_text(cx, ""); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(floor_rename_table)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(floor_table_name)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Rename tables")?;
                let id = st
                    .selected_table_id
                    .clone()
                    .ok_or_else(|| "Select a table first".to_string())?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .rename_table(&id, &name)?;
                st.reload_all()?;
                st.status = format!("Table renamed to {}", name.trim());
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(floor_table_name)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(floor_delete_table)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Delete tables")?;
                let id = st.selected_table_id.clone().ok_or_else(|| "Select a table first".to_string())?;
                st.store.as_ref().ok_or_else(|| "No store".to_string())?.delete_table(&id)?;
                st.selected_table_id = None;
                st.reload_all()?;
                st.status = "Table deleted".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(receipt_print_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .format_receipt_text(&sale_id)
            })();
            match result {
                Ok(text) => self.print_or_share_receipt(cx, &text),
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(receipt_share_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .format_receipt_text(&sale_id)
            })();
            match result {
                Ok(text) => self.copy_text_to_clipboard(cx, &text),
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(day_close_copy)).clicked(actions) {
            let text = app_state().lock().unwrap().day_close_text.clone();
            if text.is_empty() {
                self.set_status(cx, "Open Day close first");
            } else {
                self.copy_text_to_clipboard(cx, &text);
            }
        }
        if self.ui.button(cx, ids!(reports_today)).clicked(actions)
            || self.ui.button(cx, ids!(reports_month)).clicked(actions)
            || self.ui.button(cx, ids!(reports_aging)).clicked(actions)
            || self.ui.button(cx, ids!(reports_refresh)).clicked(actions)
        {
            let month = self.ui.button(cx, ids!(reports_month)).clicked(actions);
            let aging_only = self.ui.button(cx, ids!(reports_aging)).clicked(actions);
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                st.day_close_text = if aging_only {
                    store.party_aging_text()?
                } else if month {
                    store.books_report_text("month")?
                } else {
                    store.books_report_text("today")?
                };
                st.screen = Screen::Reports;
                st.status = if aging_only {
                    "Party aging".into()
                } else if month {
                    "This month books".into()
                } else {
                    "Today books".into()
                };
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(reports_copy)).clicked(actions) {
            let text = app_state().lock().unwrap().day_close_text.clone();
            if text.trim().is_empty() {
                self.set_status(cx, "Open a report first");
            } else {
                self.copy_text_to_clipboard(cx, &text);
            }
        }
        if self.ui.button(cx, ids!(expense_add)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(expense_amount_input)).text();
            let note = self.ui.text_input(cx, ids!(expense_note_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                if st.device_role == DeviceRole::Client {
                    return Err("Log expenses on the shop device — this is an extra till".into());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                store.add_expense(minor, "misc", &note, &actor)?;
                st.day_close_text = store.books_report_text("today")?;
                st.screen = Screen::Reports;
                st.status = "Expense logged".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(expense_amount_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(expense_note_input)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        for (id, kind) in [
            (ids!(bank_till_to_bank), MoneyMoveKind::TillToBank),
            (ids!(bank_bank_to_till), MoneyMoveKind::BankToTill),
            (ids!(bank_in), MoneyMoveKind::BankIn),
            (ids!(bank_out), MoneyMoveKind::BankOut),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(bank_amount_input)).text();
                let note = self.ui.text_input(cx, ids!(bank_note_input)).text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Post bank / cashbook")?;
                    let actor = st.actor()?;
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&amount_text, &currency)?;
                    st.store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .post_money_movement(kind, minor, &note, &actor)?;
                    st.day_close_text = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .books_report_text("today")?;
                    st.screen = Screen::Reports;
                    st.status = format!("{} posted", kind.label());
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.ui.text_input(cx, ids!(bank_amount_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(bank_note_input)).set_text(cx, "");
                }
                self.sync_chrome(cx);
            }
        }

        for (idle, active, order_type) in [
            (ids!(order_counter), ids!(order_counter_on), OrderType::Counter),
            (ids!(order_dine), ids!(order_dine_on), OrderType::DineIn),
            (ids!(order_takeaway), ids!(order_takeaway_on), OrderType::Takeaway),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, active) {
                {
                    let mut st = app_state().lock().unwrap();
                    st.cart.set_order_type(order_type);
                    st.status = format!("Order type: {}", order_type.label());
                }
                self.sync_chrome(cx);
            }
        }

        if self.ui.button(cx, ids!(kitchen_station_all)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.selected_station_id = None;
                st.reload_all()?;
                st.status = "Showing every station".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(kitchen_station_cycle)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let stations = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .list_stations()?;
                if stations.is_empty() {
                    return Err("No kitchen stations yet — add one on an item".into());
                }
                // None → first station → … → last station → None.
                let next = match st.selected_station_id.as_deref() {
                    None => Some(stations[0].id.clone()),
                    Some(current) => match stations.iter().position(|s| s.id == current) {
                        Some(i) if i + 1 < stations.len() => Some(stations[i + 1].id.clone()),
                        _ => None,
                    },
                };
                st.selected_station_id = next;
                st.reload_all()?;
                st.status = match st.selected_station_id.as_deref() {
                    Some(id) => stations
                        .iter()
                        .find(|s| s.id == id)
                        .map(|s| format!("Station: {}", s.name))
                        .unwrap_or_else(|| "Station selected".into()),
                    None => "Showing every station".into(),
                };
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        let inv_minus = self.ui.button(cx, ids!(inv_adjust_minus)).clicked(actions);
        let inv_plus = self.ui.button(cx, ids!(inv_adjust_plus)).clicked(actions);
        if inv_minus || inv_plus {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(inv_qty_input)).text();
            let note = self.ui.text_input(cx, ids!(inv_note_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Adjust stock")?;
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let qty: i64 = qty_text
                    .trim()
                    .parse()
                    .map_err(|_| "Quantity must be a whole number".to_string())?;
                if qty <= 0 {
                    return Err("Quantity must be more than zero".into());
                }
                let product = st.find_product_by_name(&name)?;
                if !product.tracks_stock() {
                    return Err(format!(
                        "{} is not stock tracked — tap Start tracking first",
                        product.name
                    ));
                }
                let delta = if inv_plus { qty } else { -qty };
                let note = note.trim().to_string();
                let reason = if note.is_empty() { "adjust" } else { note.as_str() };
                let on_hand = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .adjust_stock(&product.id, delta, reason, &note, &actor)?;
                st.reload_all()?;
                st.refresh_inventory_text()?;
                Ok(format!("{} · {on_hand} on hand", product.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_qty_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(inv_track)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(inv_qty_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Start stock tracking")?;
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let product = st.find_product_by_name(&name)?;
                // Start at zero, then book the typed count as an opening move so
                // the paper trail shows where the first quantity came from.
                let opening: i64 = match qty_text.trim() {
                    "" => 0,
                    other => other
                        .parse()
                        .map_err(|_| "Quantity must be a whole number".to_string())?,
                };
                if opening < 0 {
                    return Err("Quantity cannot be negative".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_product_stock_qty(&product.id, 0)?;
                if opening > 0 {
                    store.adjust_stock(&product.id, opening, "count", "opening count", &actor)?;
                }
                st.reload_all()?;
                st.refresh_inventory_text()?;
                Ok(format!("{} now tracked · {opening} on hand", product.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_qty_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(inv_purchase)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(inv_qty_input)).text();
            let cost_text = self.ui.text_input(cx, ids!(inv_cost_input)).text();
            let supplier = self.ui.text_input(cx, ids!(inv_supplier_input)).text();
            let note = self.ui.text_input(cx, ids!(inv_note_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Record purchases")?;
                let actor = st.actor()?;
                let qty_milli = parse_qty_to_milli(&qty_text)?;
                let product = st.find_product_by_name(&name)?;
                let currency = st.locale.currency.clone();
                let unit_cost = match cost_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let purchase = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .record_purchase(
                        supplier.trim(),
                        &[(product.id.clone(), qty_milli, unit_cost)],
                        note.trim(),
                        &actor,
                    )?;
                st.reload_all()?;
                st.refresh_inventory_text()?;
                let qty_show = domain::format_qty_display(qty_milli, 0, &product.unit_label);
                Ok(format!(
                    "Received {qty_show} {} · {}",
                    product.name,
                    st.money(Money(purchase.total_paise))
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_qty_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(inv_cost_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(inv_refresh)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.reload_all()?;
                st.refresh_inventory_text()?;
                st.status = "Stock refreshed".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(org_save)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(org_name_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Rename the business")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can rename the business".into());
                }
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .set_org_name(&name)?;
                st.refresh_branches_text()?;
                Ok("Business name saved".to_string())
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(branch_name_input)).text();
            let code = self.ui.text_input(cx, ids!(branch_code_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add branches")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can add branches".into());
                }
                let branch = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .add_branch(&name, &code)?;
                st.refresh_branches_text()?;
                Ok(format!("Added {}", branch.label()))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(branch_name_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(branch_code_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_switch)).clicked(actions) {
            let wanted = self.ui.text_input(cx, ids!(branch_switch_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Switch branch")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can switch branch".into());
                }
                let needle = wanted.trim().to_lowercase();
                if needle.is_empty() {
                    return Err("Type the branch name to switch to".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let branch = store
                    .list_branches()?
                    .into_iter()
                    .find(|b| {
                        b.name.to_lowercase() == needle
                            || b.code.to_lowercase() == needle
                            || b.name.to_lowercase().contains(&needle)
                    })
                    .ok_or_else(|| format!("No branch matches \"{}\"", wanted.trim()))?;
                store.set_active_branch_id(&branch.id)?;
                // Catalogue, tables and tickets are all branch scoped.
                st.cart.clear();
                st.selected_table_id = None;
                st.reload_all()?;
                st.refresh_branches_text()?;
                Ok(format!("Now selling for {}", branch.label()))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_export)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can export a branch day".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let branch = store.active_branch()?;
                let json = store.export_branch_snapshot_json()?;
                let dir = store::data_dir()?.join("backups");
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                let code = if branch.code.trim().is_empty() {
                    "BRANCH".to_string()
                } else {
                    branch.code.trim().to_uppercase()
                };
                let path = dir.join(format!(
                    "branch_snapshot_{code}_{}.json",
                    chrono::Local::now().format("%Y-%m-%d")
                ));
                std::fs::write(&path, &json).map_err(|e| e.to_string())?;
                let copied = print_share::copy_text(&json);
                Ok(format!(
                    "Snapshot: {}{}",
                    path.display(),
                    if copied { " (also copied)" } else { "" }
                ))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_import)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Import a branch day")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can import a branch day".into());
                }
                let path = store::data_dir()?
                    .join("backups")
                    .join("import_branch_snapshot.json");
                if !path.exists() {
                    return Err("Missing backups/import_branch_snapshot.json".into());
                }
                let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let branch = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .import_branch_snapshot(&json)?;
                st.refresh_branches_text()?;
                Ok(format!("Imported day for {}", branch.label()))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_consolidated)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can see the roll-up".into());
                }
                let report = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .consolidated_report()?;
                let mut lines = vec![
                    format!("{} — all outlets today", report.org_name),
                    format!(
                        "Sales {} · {}",
                        report.totals.sale_count,
                        st.money(report.totals.revenue())
                    ),
                    format!(
                        "Cash {} · UPI {} · Card {}",
                        st.money(report.totals.cash()),
                        st.money(report.totals.upi()),
                        st.money(report.totals.card())
                    ),
                    format!(
                        "Still owed {} · Expenses {}",
                        st.money(Money(report.outstanding_credit_paise)),
                        st.money(Money(report.expenses_paise))
                    ),
                    String::new(),
                    "Per branch:".to_string(),
                ];
                for (branch_name, totals) in &report.branches {
                    lines.push(format!(
                        "  {branch_name} · {} sales · {}",
                        totals.sale_count,
                        st.money(totals.revenue())
                    ));
                }
                if !report.stale_branches.is_empty() {
                    lines.push(format!(
                        "Last known only: {}",
                        report.stale_branches.join(", ")
                    ));
                }
                if !report.missing_branches.is_empty() {
                    lines.push(format!(
                        "No snapshot yet: {}",
                        report.missing_branches.join(", ")
                    ));
                }
                st.branches_text = lines.join("\n");
                st.status = "Roll-up ready".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(lock_login_ok)).clicked(actions) {
            let login = self.ui.text_input(cx, ids!(lock_login_id)).text();
            let password = self.ui.text_input(cx, ids!(lock_login_password)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                if st.pin_reset_required {
                    return Err("Owner must set a new PIN first".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let member = store
                    .authenticate_login(&login, &password)?
                    .ok_or_else(|| "Wrong login or password".to_string())?;
                let _ = store.set_active_session_staff_id(Some(&member.id));
                st.session = Some(member.clone());
                st.screen = if st.device_role.is_host() && !st.onboarding_complete {
                    Screen::Onboard
                } else {
                    unlocked_home()
                };
                st.status = format!("Welcome, {}", member.lock_label());
                let _ = st.reload_all();
                if st.screen == Screen::Books {
                    let _ = st.refresh_books_overview();
                }
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }



        if self.ui.button(cx, ids!(unit_add)).clicked(actions) {
            let code = self.ui.text_input(cx, ids!(unit_code_input)).text();
            let name = self.ui.text_input(cx, ids!(unit_name_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add units")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_unit(&code, &name)?;
                st.status = format!("Unit {code} added");
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.ui.text_input(cx, ids!(unit_code_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(unit_name_input)).set_text(cx, "");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(conv_add)).clicked(actions) {
            let from = self.ui.text_input(cx, ids!(conv_from_input)).text();
            let to = self.ui.text_input(cx, ids!(conv_to_input)).text();
            let factor_text = self.ui.text_input(cx, ids!(conv_factor_input)).text();
            let result = (|| -> Result<(), String> {
                let factor_milli = parse_qty_to_milli(&factor_text)
                    .map_err(|_| "Factor must be a number > 0".to_string())?;
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add unit conversion")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_unit_conversion(&from, &to, factor_milli)?;
                st.status = format!("1 {from} → {to} conversion added");
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.ui.text_input(cx, ids!(conv_from_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(conv_to_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(conv_factor_input)).set_text(cx, "");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(venue_brand_save)).clicked(actions) {
            self.save_venue_brand(cx);
        }
        if self.ui.button(cx, ids!(venue_icon_pick)).clicked(actions) {
            self.pick_venue_icon(cx);
        }
        if self.ui.button(cx, ids!(venue_icon_clear)).clicked(actions) {
            self.clear_venue_icon(cx);
        }

        if self.ui.button(cx, ids!(backup_btn)).clicked(actions) {
            let result = (|| -> Result<std::path::PathBuf, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let path = default_backup_path()?;
                store.backup_to(&path)?;
                Ok(path)
            })();
            match result {
                Ok(path) => {
                    self.ui
                        .widget(cx, ids!(backup_path_label)).set_text(cx, &plain_display_text(&format!("Backup: {}", path.display())));
                    self.set_status(cx, "Backup created");
                }
                Err(e) => self.set_status(cx, e),
            }
        }

        if self.ui.button(cx, ids!(restore_btn)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let restore_path = store::data_dir()?.join("backups").join("restore.sqlite3");
                if !restore_path.exists() {
                    return Err("Missing backups/restore.sqlite3".into());
                }
                {
                    let mut st = app_state().lock().unwrap();
                    // Restoring a Client would overwrite its pairing with the
                    // Server's own books.
                    st.require_server_role("Restore a backup")?;
                    st.store = None;
                }
                let new_store = Store::restore_from(&restore_path)?;
                let mut st = app_state().lock().unwrap();
                st.store = Some(new_store);
                st.reload_all()?;
                st.status = "Restored from backup".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if matches!(event, Event::Startup) {
            // Before CreateWindow: override the DSL default with the last session.
            self.restore_window_geom(cx);
            self.ensure_store(cx);
        }
        if matches!(event, Event::WindowGeomChange(_)) {
            let restoring = self.apply_pending_window_geom_after_create(cx);
            self.enforce_min_desktop_window_size(cx);
            self.sync_dev_size(cx);
            if !restoring {
                self.persist_window_geom(cx);
            }
        }
        self.match_event(cx, event);
        if self.pointer_navigates(cx, event) {
            return;
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
        // Hot reload re-applies DSL (visibility resets). Restore from AppState.
        if matches!(event, Event::LiveEdit | Event::ScriptReapply | Event::Startup) {
            self.sync_chrome(cx);
        }
    }
}

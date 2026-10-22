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
    domain, entity_template, fiscal, locale, local_servers, menu_images, net, precise,
    print_share, store,
};
#[cfg(not(target_os = "android"))]
use kumkaj_khaynjo::{
    domain, entity_template, fiscal, locale, local_servers, menu_images, net, precise,
    print_share, store,
};

use domain::{
    new_id, AttachmentEntity, Cart, Customer, DiningTable, KotStatus, Money, MoneyLane,
    MoneyMoveKind, OrderType, PayMethod, Product, Sale, SaleRevision, StaffFeatures, StaffMember,
    StaffRole, TableStatus, VenueMode,
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

/// Dragging the rail/main split: stores grab offset (pointer x − rail width).
fn rail_resize_drag() -> &'static Mutex<Option<f64>> {
    static DRAG: OnceLock<Mutex<Option<f64>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// In-progress sidebar / bottom-bar drag-to-swap (peeked at App level,
/// same style as the rail resize drag). `rail` picks the surface, `slot`
/// is the position the drag started from, `hover` the position under the
/// pointer right now (drop target).
#[derive(Clone)]
struct NavDrag {
    rail: bool,
    slot: usize,
    start: DVec2,
    /// Latest pointer pos while dragging; edge auto-scroll reads it
    /// on frames where the pointer itself is standing still.
    last: DVec2,
    /// Window pos of the grabbed item's face when the drag began; the
    /// floating ghost chip tracks `origin + (pointer - start)`.
    origin: DVec2,
    /// Every slot's face rect in the bar's *content* coordinates,
    /// captured at pickup. Widget areas are only trustworthy while
    /// the layout is settled — during edge auto-scroll they lag the
    /// scrolling content — so drag-time hit tests use these frozen
    /// rects plus the live scroll offset instead.
    rects: Vec<Rect>,
    active: bool,
    hover: Option<usize>,
}

fn nav_drag() -> &'static Mutex<Option<NavDrag>> {
    static DRAG: OnceLock<Mutex<Option<NavDrag>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// Pending frame callback that keeps edge auto-scroll alive while a
/// nav drag is held against the bar's edge (see `nav_drag_autoscroll`).
fn nav_autoscroll_frame() -> &'static Mutex<Option<NextFrame>> {
    static FRAME: OnceLock<Mutex<Option<NextFrame>>> = OnceLock::new();
    FRAME.get_or_init(|| Mutex::new(None))
}

/// Set when a drag (not a click) ends on a nav item, so the Button's
/// resulting `clicked` action is ignored once instead of navigating.
fn nav_suppress_click() -> &'static Mutex<bool> {
    static SUPPRESS: OnceLock<Mutex<bool>> = OnceLock::new();
    SUPPRESS.get_or_init(|| Mutex::new(false))
}

/// Side rail width floor while the user drags the divider.
const RAIL_WIDTH_MIN: f64 = 148.0;
/// Default rail width (fits short chrome labels without ellipsis).
const RAIL_WIDTH_DEFAULT: f64 = 220.0;
/// Main column floor while resizing the rail (css px).
const MAIN_CONTENT_MIN: f64 = 360.0;

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
    /// Active start template id (`custom`, `khaynjo_restaurant`, …).
    entity_template_id: String,
    /// Bill type for the open cart (Tax invoice vs Cash memo).
    money_lane: MoneyLane,
    /// Buy tab: purchase posts to AP when true.
    buy_on_account: bool,
    day_close_text: String,
    /// Inventory screen body (low stock + recent movements).
    ops_panel_text: String,
    /// Branches screen body (org, branch list, roll-up).
    branches_text: String,
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
    /// Last logged expense (for attaching bill photo/PDF).
    selected_expense_id: Option<String>,
    /// Pending file-dialog target for entry attachments.
    attach_pick: Option<AttachPickTarget>,
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
    /// Sidebar / bottom-bar arrangement (drag to swap; saved in the books).
    nav_order: Vec<NavTab>,
    /// Pending custom icon path from picker (before Save brand).
    draft_venue_icon_path: Option<PathBuf>,
    /// None = auto-detect from name; Some(key) = built-in or custom:…
    draft_image_key: Option<String>,
    /// Expense forms: how the expense was paid — first tender (gets the
    /// remainder) and the optional second tender for a split payment.
    expense_tender: PayMethod,
    expense_tender2: PayMethod,
    /// Open KOTs for the Kitchen screen (refreshed when the screen shows).
    open_kots: Vec<domain::KotTicket>,
    /// Kitchen stations for the Kitchen screen filter/labels.
    stations: Vec<domain::KitchenStation>,
    /// Shop categories for the Menu screen list (photo or initials avatar).
    categories: Vec<domain::Category>,
    /// Provided-items catalogue browsing (Menu screen picker): the
    /// shelf's categories, the one being browsed, and its dishes.
    catalogue_cats: Vec<menu_images::CatalogueCategory>,
    catalogue_cat: String,
    catalogue_items: Vec<menu_images::CatalogueItem>,
    /// Category whose photo is being picked (file-dialog round trip).
    category_photo_id: Option<String>,
    /// Category chosen via the Categories list's Use button: carried
    /// by UUID (identity), with the label it had when chosen. The
    /// form's text field is only a display of it — if the owner
    /// retypes the field, name resolution takes over again.
    form_category_id: Option<String>,
    form_category_label: String,
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
    /// Nested-flow history only (e.g. ClientPair ← DeviceSetup). Top-level peers
    /// do not push history — no mouse Back / edge-swipe between Books, Parties, …
    nav_back: Vec<Screen>,
    nav_forward: Vec<Screen>,
    /// User-resized side rail width (css px). Clamped in `apply_shell_layout`.
    rail_width: f64,
    /// Which bank-books report chip is selected (must match `day_close_text` body).
    books_report_kind: BooksReportKind,
}

/// Wrong PINs tolerated before [`UNLOCK_BLOCK`] kicks in.
const UNLOCK_ATTEMPT_LIMIT: u32 = 5;
const UNLOCK_BLOCK: Duration = Duration::from_secs(30);

/// File-dialog target after the user picks a photo or PDF for an entry.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AttachPickTarget {
    Image(AttachmentEntity),
    File(AttachmentEntity),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeviceRole {
    Unset,
    Server,
    Client,
}

impl DeviceRole {
    fn from_meta(value: Option<&str>) -> Self {
        match value.map(str::trim) {
            Some("server") => Self::Server,
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
    /// Dining floor — tables grid; tap a table to open its order.
    Floor,
    /// Kitchen display — open KOTs, advance New → Preparing → Ready → Done.
    Kitchen,
    /// Money out — purchase + expense.
    Buy,
    Menu,
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

/// Which chrome peer is selected. Former More hub items are first-class peers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavTab {
    Books,
    Sell,
    Buy,
    Menu,
    Floor,
    Kitchen,
    Sales,
    DayClose,
    Reports,
    Inventory,
    Branches,
    Credit,
    Team,
    Settings,
    Lock,
}

/// Bank-books sub-selection on the Reports screen (highlight must match body text).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum BooksReportKind {
    #[default]
    Today,
    Month,
    Aging,
    BalanceSheet,
}

/// Default sidebar / bottom-bar arrangement (matches the DSL slot order).
const NAV_TABS: [NavTab; 15] = [
    NavTab::Books,
    NavTab::Sell,
    NavTab::Buy,
    NavTab::Menu,
    NavTab::Floor,
    NavTab::Kitchen,
    NavTab::Sales,
    NavTab::DayClose,
    NavTab::Reports,
    NavTab::Inventory,
    NavTab::Branches,
    NavTab::Credit,
    NavTab::Team,
    NavTab::Settings,
    NavTab::Lock,
];

fn nav_tab_key(tab: NavTab) -> &'static str {
    match tab {
        NavTab::Books => "books",
        NavTab::Sell => "sell",
        NavTab::Buy => "buy",
        NavTab::Menu => "menu",
        NavTab::Floor => "floor",
        NavTab::Kitchen => "kitchen",
        NavTab::Sales => "sales",
        NavTab::DayClose => "day_close",
        NavTab::Reports => "reports",
        NavTab::Inventory => "inventory",
        NavTab::Branches => "branches",
        NavTab::Credit => "credit",
        NavTab::Team => "team",
        NavTab::Settings => "settings",
        NavTab::Lock => "lock",
    }
}

/// Saved order wins; unknown keys are dropped and any tab missing from the
/// saved list is appended in default order, so an older saved order can
/// never hide a newer tab.
fn parse_nav_order(csv: Option<&str>) -> Vec<NavTab> {
    let mut order: Vec<NavTab> = Vec::with_capacity(NAV_TABS.len());
    if let Some(csv) = csv {
        for key in csv.split(',') {
            let key = key.trim();
            if let Some(tab) = NAV_TABS.iter().find(|t| nav_tab_key(**t) == key)
                && !order.contains(tab)
            {
                order.push(*tab);
            }
        }
    }
    for tab in NAV_TABS {
        if !order.contains(&tab) {
            order.push(tab);
        }
    }
    order
}

fn nav_order_csv(order: &[NavTab]) -> String {
    order.iter().map(|t| nav_tab_key(*t)).collect::<Vec<_>>().join(",")
}

fn nav_tab_for(screen: Screen) -> Option<NavTab> {
    match screen {
        Screen::Books => Some(NavTab::Books),
        Screen::Pos => Some(NavTab::Sell),
        Screen::Floor => Some(NavTab::Floor),
        Screen::Kitchen => Some(NavTab::Kitchen),
        Screen::Buy => Some(NavTab::Buy),
        Screen::Menu => Some(NavTab::Menu),
        Screen::Sales => Some(NavTab::Sales),
        Screen::DayClose => Some(NavTab::DayClose),
        Screen::Reports => Some(NavTab::Reports),
        Screen::Inventory => Some(NavTab::Inventory),
        Screen::Branches => Some(NavTab::Branches),
        Screen::Credit => Some(NavTab::Credit),
        Screen::Team => Some(NavTab::Team),
        Screen::Settings => Some(NavTab::Settings),
        Screen::Lock => Some(NavTab::Lock),
        Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair => None,
    }
}

/// Only nested setup flows keep Back/Forward history. Former More destinations
/// are top-level peers — no stack, no swipe-back between them.
fn screen_keeps_history(screen: Screen) -> bool {
    matches!(screen, Screen::DeviceSetup | Screen::ClientPair)
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
            venue_mode: VenueMode::Restaurant,
            client_paired: false,
            extra_tills_allowed: false,
            onboard_step: 1,
            entity_template_id: "khaynjo_restaurant".into(),
            money_lane: MoneyLane::Declared,
            buy_on_account: false,
            day_close_text: String::new(),
            ops_panel_text: String::new(),
            branches_text: String::new(),
            screen: Screen::DeviceSetup,
            status: String::new(),
            last_receipt: "No sale yet".into(),
            undo_sale_id: None,
            undo_snack_text: String::new(),
            undo_timer: Timer::empty(),
            pin_buffer: String::new(),
            pin_visible: false,
            selected_sale_id: None,
            selected_expense_id: None,
            attach_pick: None,
            locale: LocaleSettings::default(),
            fiscal: FiscalSettings::default(),
            country_browse: String::new(),
            venue_title: String::new(),
            venue_icon_key: None,
            nav_order: NAV_TABS.to_vec(),
            draft_venue_icon_path: None,
            draft_image_key: None,
            expense_tender: PayMethod::Cash,
            expense_tender2: PayMethod::Bank,
            open_kots: Vec::new(),
            stations: Vec::new(),
            categories: Vec::new(),
            catalogue_cats: Vec::new(),
            catalogue_cat: String::new(),
            catalogue_items: Vec::new(),
            category_photo_id: None,
            form_category_id: None,
            form_category_label: String::new(),
            editing_product_id: None,
            pay_in_flight: false,
            failed_unlocks: 0,
            unlock_blocked_until: None,
            pin_reset_required: false,
            nav_back: Vec::new(),
            nav_forward: Vec::new(),
            rail_width: RAIL_WIDTH_DEFAULT,
            books_report_kind: BooksReportKind::Today,
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
        self.nav_order = parse_nav_order(store.nav_order_csv()?.as_deref());
        self.open_kots = store.list_open_kots().unwrap_or_default();
        self.stations = store.list_stations().unwrap_or_default();
        self.categories = store.list_categories().unwrap_or_default();
        self.owner_setup_complete = store.owner_setup_complete().unwrap_or(true);
        self.onboarding_complete = store.onboarding_complete().unwrap_or(true);
        // Khaynjo product: restaurant venue (tables + KOT). Counter-only
        // shops simply never open the floor; the venue stays Restaurant so
        // KOT creation is available the moment tables are enabled.
        if store.venue_mode().unwrap_or(VenueMode::Restaurant) != VenueMode::Restaurant {
            let _ = store.set_venue_mode(VenueMode::Restaurant);
        }
        self.venue_mode = VenueMode::Restaurant;
        self.client_paired = store.client_paired().unwrap_or(false);
        self.pin_reset_required = store.pin_reset_required().unwrap_or(false);
        self.entity_template_id = store
            .entity_template_id()
            .unwrap_or_else(|_| "custom".into());
        // After the store borrow ends: catalogue fields are &mut self.
        self.refresh_catalogue();
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

    /// Load the bank-books body for [`Self::books_report_kind`] into `day_close_text`.
    fn load_books_report(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or_else(|| "No store".to_string())?;
        self.day_close_text = match self.books_report_kind {
            BooksReportKind::Today => store.books_report_text("today")?,
            BooksReportKind::Month => store.books_report_text("month")?,
            BooksReportKind::Aging => store.outstanding_pack_text()?,
            BooksReportKind::BalanceSheet => store.balance_sheet_text()?,
        };
        Ok(())
    }

    /// Reports screen body (bank books / day reports).
    fn refresh_reports_facet(&mut self) -> Result<(), String> {
        self.load_books_report()
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

    /// Reload the provided-items shelf (folder scan). Keeps the
    /// owner's browsing place when the selected folder still exists.
    fn refresh_catalogue(&mut self) {
        self.catalogue_cats = menu_images::catalogue_categories();
        if !self.catalogue_cats.iter().any(|c| c.key == self.catalogue_cat) {
            self.catalogue_cat = self
                .catalogue_cats
                .first()
                .map(|c| c.key.clone())
                .unwrap_or_default();
        }
        self.catalogue_items = menu_images::catalogue_items(&self.catalogue_cat);
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
    #[allow(dead_code)] // Kept for a short till snapshot; Reports Today uses full books_report_text.
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

    /// Reset the order desk for the next order: empty cart, no table or
    /// customer attached. Called after a bill is paid / a KOT is sent.
    fn clear_order_desk(&mut self) {
        self.cart.clear();
        self.selected_table_id = None;
        self.selected_customer_id = None;
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
                // Markdown treats a single \\n as a soft break (often one
                // clipped line). Paragraph breaks so WrapLabel wraps each row
                // inside the bar instead of truncating at the right edge.
                lines.join("\n\n")
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
        chinese := FontMember{res: crate_resource("self:resources/fonts/LXGWWenKaiRegular.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("self:resources/fonts/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("self:resources/fonts/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
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
        chinese := FontMember{res: crate_resource("self:resources/fonts/LXGWWenKaiBold.ttf") asc: 0.0 desc: 0.0}
        icons := FontMember{res: crate_resource("self:resources/fonts/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
        emoji := FontMember{res: crate_resource("self:resources/fonts/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
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
    // Palette from apps/khaynjo/assets/khaynjo-app-icon.png (deep red,
    // maroon, red-orange, gold, warm cream — sampled with ImageMagick).
    // Light face → dark ink (#260B08 / #6B4A3F / #8A6A5C) in EVERY state.
    // Deep-red face (#9A0E10) → white ink in EVERY state.
    // Makepad mixes color→color_hover→color_down; mismatched lightness = wash.
    // Gold (#F69D2C) is the selection/accent band — dark ink stays readable on it.
    mod.theme.color_bg_app = #xF9F4F0
    mod.theme.color_app_caption_bar = #x9A0E10
    mod.theme.color_text = #x260B08
    mod.theme.color_text_hover = #x260B08
    mod.theme.color_text_focus = #x260B08
    mod.theme.color_text_down = #x260B08
    mod.theme.color_text_hl = #x260B08
    mod.theme.color_text_disabled = #x8A6A5C
    mod.theme.color_text_val = #x6B4A3F
    mod.theme.color_text_meta = #x8A6A5C
    mod.theme.color_label_inner = #x260B08
    mod.theme.color_label_inner_hover = #x260B08
    mod.theme.color_label_inner_down = #x260B08
    mod.theme.color_label_inner_focus = #x260B08
    mod.theme.color_label_inner_active = #x260B08
    mod.theme.color_text_placeholder = #x6B4A3F
    mod.theme.color_text_placeholder_hover = #x260B08
    // Caret must be dark on cool/white wells (stock theme uses white = invisible).
    mod.theme.color_text_cursor = #x69090C
    mod.theme.color_cursor = #x69090C
    mod.theme.color_cursor_focus = #x69090C
    mod.theme.color_cursor_empty = #x69090C
    mod.theme.color_cursor_border = #x69090C

    // Stock TextInput caret + placeholder colors are baked from theme at
    // widget-def time (often white on gray wells). Re-bind after our theme so
    // EVERY TextInput shows a visible caret and readable placeholders.
    // Do NOT use set_type_default() here (Android Animator UAF).
    mod.widgets.TextInputFlat = mod.widgets.TextInputFlat{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x69090C
        }
        draw_text +: {
            color: #x260B08
            color_hover: #x260B08
            color_focus: #x260B08
            color_down: #x260B08
            color_empty: #x8A6A5C
            color_empty_hover: #x6B4A3F
            color_empty_focus: #x6B4A3F
        }
    }
    mod.widgets.TextInput = mod.widgets.TextInput{
        blink_speed: 0.5
        draw_cursor +: {
            color: #x69090C
        }
        draw_text +: {
            color: #x260B08
            color_hover: #x260B08
            color_focus: #x260B08
            color_down: #x260B08
            color_empty: #x8A6A5C
            color_empty_hover: #x6B4A3F
            color_empty_focus: #x6B4A3F
        }
    }
    mod.theme.color_icon = #x260B08
    mod.theme.color_icon_active = #x260B08
    mod.theme.color_icon_inactive = #x8A6A5C
    mod.theme.color_icon_disabled = #xD8BBA6
    // Cool inset wells — mid gray, never near-white cream on white cards.
    // Inset = white wells on cool canvas (readable placeholders). Never match color_bg_app.
    mod.theme.color_inset = #xffffff
    mod.theme.color_inset_hover = #xF6E3D3
    mod.theme.color_inset_down = #xEFD2BC
    mod.theme.color_inset_focus = #xffffff
    mod.theme.color_inset_empty = #xffffff
    // Solid light faces — color_2 must match color (never transparent wash).
    mod.theme.color_outset = #xffffff
    mod.theme.color_outset_hover = #xF6E3D3
    mod.theme.color_outset_down = #xEFD2BC
    mod.theme.color_outset_focus = #xffffff
    mod.theme.color_outset_1 = #xffffff
    mod.theme.color_outset_1_hover = #xF6E3D3
    mod.theme.color_outset_1_down = #xEFD2BC
    mod.theme.color_outset_1_focus = #xffffff
    // Same stop as color_outset — transparent color_2 washes light-on-light.
    mod.theme.color_outset_2 = #xffffff
    mod.theme.color_outset_2_hover = #xF6E3D3
    mod.theme.color_outset_2_down = #xEFD2BC
    mod.theme.color_outset_2_focus = #xffffff
    // Selection band: opaque mid-blue on light page (plan: visible, not pale wash).
    // Dark ink (#x260B08) stays readable on this band.
    mod.theme.color_selection = #xF69D2CFF
    mod.theme.color_selection_hover = #xF69D2CFF
    mod.theme.color_selection_down = #xE8890CFF
    mod.theme.color_selection_focus = #xF69D2CFF
    mod.theme.color_selection_empty = #xF69D2CFF
    mod.theme.color_selection_disabled = #xEFD2BC

    // Default Label ink. Keep stock flow — forcing Down drops glyphs on Android GLES.
    mod.widgets.Label = set_type_default() do mod.widgets.Label{
        height: Fit
        draw_text +: {
            color: #x260B08
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
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_disabled: #xEFD2BC
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            color_2_disabled: #xEFD2BC
            border_color: #xD8BBA6
            border_color_hover: #x9A0E10
            border_color_down: #x69090C
            border_color_focus: #xD8BBA6
            border_radius: 12.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            color_disabled: #x8A6A5C
        }
    }

    // NEVER call set_type_default on TextInput — it rebuilds Animator apply
    // objects and Android then panics with GenVec use-after-free on NextFrame,
    // freezing the whole UI (PIN keypad included). Style fields via theme +
    // per-instance blink_speed/padding only (see FieldInput).

    // Makepad 2 ScrollYView: wheel/trackpad + app keyboard (↑/↓, PgUp/PgDn).
    // Drag-scroll steals text selection; selectable WrapLabels need the drag.
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
        pass.clear_color: #xF9F4F0
        caption_bar +: {
            draw_bg.color: #x9A0E10
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
                        color_down: #xEBD5C2
                        bg_color: #00000000
                        bg_color_hover: #xB43A12
                        bg_color_down: #x69090C
                    }
                }
                max +: {
                    draw_bg +: {
                        color: #xffffff
                        color_hover: #xffffff
                        color_down: #xEBD5C2
                        bg_color: #00000000
                        bg_color_hover: #xB43A12
                        bg_color_down: #x69090C
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
                        color_down: #xEBD5C2
                    }
                }
            }
        }
    }

    // Full-width CTA — wrap, never ellipsis. Makepad Fill+Ellipsis clips the
    // right of any multi-word / translated label on every platform.
    let NavBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_disabled: #xA9806E
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            color_2_disabled: #xA9806E
            border_color: #x9A0E10
            border_color_hover: #xB43A12
            border_color_down: #x69090C
            border_color_focus: #x9A0E10
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xEFD2BC
            ink_centered: true
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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

    // Side-rail faces. Makepad 2 Button has NO selected state — only
    // hover/down/focus animators. Selection = TabBtnActive visible via Overlay
    // swap. Idle TabBtn must NEVER use the selected navy fill on hover/focus
    // (that made previous + current tabs look active at once).
    // Wrap + no Ellipsis: long/translated labels grow the row instead of clipping.
    let TabBtn = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 10. right: 10.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xF2B183
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xF2B183
            color_disabled: #x8A6A5C
            text_style +: {font_size: 13.}
        }
    }
    // Bottom-nav chips for expanded More peers (scroll row; Fit width).
    // Fit min/max must be FitBound objects — bare numbers only work on Fill{min:}.
    let NavChip = Button{
        width: Fit{min: FitBound.Abs(76.)} height: 56
        align: Center
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x00000000
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #x00000000
            color_disabled: #x00000000
            color_2: #x00000000
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #x00000000
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xF2B183
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xF2B183
            color_disabled: #x8A6A5C
            text_style +: {font_size: 12.}
        }
    }
    let NavChipActive = Button{
        width: Fit{min: FitBound.Abs(76.)} height: 56
        align: Center
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        label_walk: Walk{
            width: Fit
            height: Fit
            margin: Inset{left: 4. right: 4.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xB43A12
            color_hover: #xB43A12
            color_down: #x9A0E10
            color_focus: #xB43A12
            color_disabled: #xEFD2BC
            color_2: #xB43A12
            color_2_hover: #xB43A12
            color_2_down: #x9A0E10
            color_2_focus: #xB43A12
            border_color: #x00000000
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #x8A6A5C
            text_style +: {font_size: 12.}
        }
    }
    let TabBtnActive = Button{
        width: Fill height: Fit
        align: Align{x: 0.0 y: 0.5}
        flow: Right{wrap: true}
        grab_key_focus: false
        reset_hover_on_click: true
        margin: Inset{top: 0. right: 0. bottom: 0. left: 0.}
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 10. right: 10.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #x9A0E10
            color_down: #x9A0E10
            color_focus: #x9A0E10
            color_disabled: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #x9A0E10
            color_2_down: #x9A0E10
            color_2_focus: #x9A0E10
            color_2_disabled: #x9A0E10
            border_color: #x9A0E10
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #x8A6A5C
            text_style +: {font_size: 13.}
        }
    }

    // Secondary pay — navy + white (never white face on white cart).
    let PayBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_disabled: #xA9806E
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            color_2_disabled: #xA9806E
            border_color: #x9A0E10
            border_color_hover: #xB43A12
            border_color_down: #x69090C
            border_color_focus: #x9A0E10
            border_radius: 16.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xEFD2BC
            ink_centered: true
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
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            border_color: #xD8BBA6
            border_color_hover: #x9A0E10
            border_color_down: #x69090C
            border_color_focus: #xD8BBA6
            border_radius: 8.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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
    // Prefer ActionStack when labels are longer than ~4 chars at 320px width.
    // Row faces wrap (no ellipsis) so half-width ActionRow still shows full text.
    let RowPrimaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }
    let RowSecondaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xEFD2BC
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 14.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            ink_centered: true
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
    // Equal-width fields only (price|unit, etc.). NEVER put NavBtn / Stack*
    // CTAs here — Fit+wrap beside Fill TextInputs collapses to a vertical
    // letter stack ("A d d r a t e"). Put CTAs in ActionStack under the row.
    let FormFieldRow = View{
        width: Fill height: Fit
        flow: Right spacing: 8
        align: Align{y: 0.5}
    }
    // ActionStack faces — same navy/gray as RowPrimary/Secondary, but height Fit
    // + wrap (no ellipsis). Use for any multi-word stacked CTA (language source,
    // Server/Client Mode, etc.). Never put those labels in ChipBtn.
    let StackPrimaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
            border_radius: 14.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }
    let StackSecondaryBtn = Button{
        width: Fill height: Fit
        margin: Inset{left: 0. right: 0. top: 0. bottom: 0.}
        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
        align: Align{x: 0.5 y: 0.5}
        flow: Right{wrap: true}
        label_walk: Walk{
            width: Fill
            height: Fit
            margin: Inset{left: 16. right: 16.}
        }
        icon_walk: Walk{width: 0. height: 0.}
        draw_bg +: {
            color: #xEFD2BC
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 14.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            ink_centered: true
            text_style +: {font_size: 15.}
        }
    }

    // Chip for horizontal strips ONLY: Fit width (scroll, don't crush). Cap so
    // short labels ellipsis instead of stealing the whole strip.
    // Fit.max is Option<FitBound> — use FitBound.Abs (same pattern as Flow.Right).
    // NEVER use ChipBtn for multi-word CTAs — ActionStack + StackPrimary/Secondary.
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
            color: #xEFD2BC
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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
            color: #xEFD2BC
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xEFD2BC
            color_2: #xEFD2BC
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xEFD2BC
            border_color: #xD8BBA6
            border_radius: 10.
            border_size: 1.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            border_color: #x9A0E10
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


    // Styled field — padding/blink/caret/placeholder for ALL app inputs.
    // White well on cool canvas so placeholders stay readable (never gray-on-gray).
    // Define draw_bg here only — do not mutate Animator color tracks at runtime
    // on Android (use-after-free).
    let FieldInput = TextInput{
        width: Fill
        height: Fit
        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
        blink_speed: 0.5
        draw_bg +: {
            color: #xffffff
            color_hover: #xF6E3D3
            color_focus: #xffffff
            color_down: #xEFD2BC
            color_empty: #xffffff
            border_color: #x5A4238
            border_color_hover: #x9A0E10
            border_color_focus: #x9A0E10
            border_radius: 12.
            border_size: 1.5
        }
        draw_cursor +: {
            color: #x69090C
        }
        // Flat opaque selection — color_2 must match (no light gradient wash).
        draw_selection +: {
            gradient_fill_horizontal: 0.0
            color: #xF69D2CFF
            color_hover: #xF69D2CFF
            color_focus: #xF69D2CFF
            color_down: #xE8890CFF
            color_empty: #xF69D2CFF
            color_2: #xF69D2CFF
            color_2_hover: #xF69D2CFF
            color_2_focus: #xF69D2CFF
            color_2_down: #xE8890CFF
            color_2_empty: #xF69D2CFF
        }
        draw_text +: {
            color: #x260B08
            color_hover: #x260B08
            color_focus: #x260B08
            color_down: #x260B08
            color_empty: #x6B4A3F
            color_empty_hover: #x260B08
            color_empty_focus: #x260B08
            text_style +: {font_size: 16.}
        }
    }

    // Compact cart / totals field — same caret as FieldInput; tighter padding.
    let CartField = FieldInput{
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        draw_bg +: {
            color: #xffffff
            color_hover: #xF6E3D3
            color_focus: #xffffff
            color_down: #xEFD2BC
            color_empty: #xffffff
            border_color: #x5A4238
            border_color_hover: #x69090C
            border_color_focus: #x69090C
            border_radius: 10.
            border_size: 2.
        }
        draw_text +: {
            color: #x260B08
            color_empty: #x8A6A5C
            color_empty_hover: #x6B4A3F
            color_empty_focus: #x6B4A3F
            color_hover: #x260B08
            color_down: #x260B08
            color_focus: #x260B08
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
        font_color: #x260B08
        draw_text +: {
            color: #x260B08
        }
        draw_block +: {
            // Opaque mid-blue (RRGGBBAA) — readable dark ink on top, clear on white page.
            selection_color: #xF69D2CFF
        }
    }
    let WrapLabel = mod.widgets.WrapLabel{}

    // Photo or name-initials avatar. When no photo: navy well + 1–2 letters
    // (see menu_images::name_initials). Size overridden per call site.
    let ThumbAvatar = RoundedView{
        width: 64 height: 64
        margin: Inset{top: 4. bottom: 4. left: 4. right: 4.}
        flow: Overlay
        align: Align{x: 0.5 y: 0.5}
        draw_bg.color: #x9A0E10
        draw_bg.border_radius: 12.
        draw_bg.border_size: 0.
        thumb_initials := Label{
            width: Fill height: Fill
            text: ""
            align: Align{x: 0.5 y: 0.5}
            draw_text.color: #xffffff
            draw_text.text_style.font_size: 22.
        }
        thumb := Image{
            width: Fill height: Fill
            fit: ImageFit.CropToFill
            visible: false
        }
    }

    // Sell token: Fit height — list row estimate must stay >= natural size.
    let ProductCard = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 10. right: 10. top: 12. bottom: 12.}
        flow: Down spacing: 8
        align: Align{x: 0.5 y: 0.0}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        ThumbAvatar{
            width: 64 height: 64
        }
        name := Label{
            width: Fill
            text: "Item"
            align: Align{x: 0.5}
            flow: Right{wrap: false}
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x260B08
            draw_text.text_style.font_size: 13.
        }
        price := Label{
            width: Fill
            text: "0"
            align: Align{x: 0.5}
            flow: Right{wrap: false}
            max_lines: 1
            draw_text.color: #xB43A12
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
                color: #xB43A12
                color_hover: #x822801
                color_down: #x6f2000
                color_focus: #xB43A12
                color_disabled: #xD8BBA6
                color_2: #xB43A12
                color_2_hover: #x822801
                color_2_down: #x6f2000
                color_2_focus: #xB43A12
                color_2_disabled: #xD8BBA6
                border_color: #xB43A12
                border_color_hover: #x822801
                border_color_down: #x6f2000
                border_color_focus: #xB43A12
                border_radius: 12.
                border_size: 0.
            }
            draw_text +: {
                color: #xffffff
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #xffffff
                color_disabled: #x8A6A5C
                text_style +: {font_size: 15.}
            }
        }
    }

    // Floor tab: one card per table — free (cream) or occupied (deep red)
    let FloorCard = RoundedView{
        cursor: MouseCursor.Hand
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xEFD2BC
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        View{
            width: Fill height: Fit
            flow: Down spacing: 2
            name := Label{
                width: Fill
                text: "Table"
                flow: Right{wrap: false}
                max_lines: 1
                draw_text.color: #x260B08
                draw_text.text_style.font_size: 18.
            }
            status := Label{
                width: Fill
                text: "Free"
                flow: Right{wrap: false}
                max_lines: 1
                draw_text.color: #x6B4A3F
                draw_text.text_style.font_size: 13.
            }
        }
        bill_req := Button{
            text: "Request bill"
            width: Fit height: 44
            padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
            draw_bg +: {
                color: #xffffff
                color_hover: #x9A0E10
                color_down: #x69090C
                color_focus: #xffffff
                color_2: #xffffff
                color_2_hover: #x9A0E10
                color_2_down: #x69090C
                color_2_focus: #xffffff
                border_radius: 12.
                border_size: 2.
                border_color: #x9A0E10
                border_color_hover: #x9A0E10
                border_color_down: #x69090C
                border_color_focus: #x9A0E10
            }
            draw_text +: {
                color: #x93000a
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #x93000a
                text_style +: {font_size: 14.}
            }
        }
        open := Button{
            text: "Open"
            width: Fit height: 44
            padding: Inset{left: 18. right: 18. top: 10. bottom: 10.}
            draw_bg +: {
                color: #xB43A12
                color_hover: #x822801
                color_down: #x6f2000
                color_focus: #xB43A12
                color_2: #xB43A12
                color_2_hover: #x822801
                color_2_down: #x6f2000
                color_2_focus: #xB43A12
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
    }

    // Kitchen tab: one card per open KOT — items + state, never prices
    let KotCard = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
        flow: Down spacing: 6
        draw_bg.color: #ffffff
        draw_bg.border_radius: 16.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        title := WrapLabel{
            width: Fill
            body: "Kitchen"
            font_color: #x69090C
            font_size: 14.
        }
        summary := WrapLabel{
            width: Fill
            body: ""
            font_color: #x260B08
            font_size: 15.
        }
        View{
            width: Fill height: Fit
            flow: Right spacing: 10
            align: Align{y: 0.5}
            state := Label{
                width: Fill
                text: "New"
                flow: Right{wrap: false}
                max_lines: 1
                draw_text.color: #x9A6206
                draw_text.text_style.font_size: 13.
            }
            advance := Button{
                text: "Start"
                width: Fit height: 44
                padding: Inset{left: 18. right: 18. top: 10. bottom: 10.}
                draw_bg +: {
                    color: #xB43A12
                    color_hover: #x822801
                    color_down: #x6f2000
                    color_focus: #xB43A12
                    color_2: #xB43A12
                    color_2_hover: #x822801
                    color_2_down: #x6f2000
                    color_2_focus: #xB43A12
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
        draw_bg.border_color: #x5A4238
        name := Label{
            width: Fill
            height: Fit
            padding: 0.
            align: Align{x: 0.5}
            text: "Item"
            flow: Right{wrap: false}
            max_lines: 1
            text_overflow: Ellipsis
            draw_text.color: #x260B08
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
                ThumbAvatar{
                    width: 56 height: 56
                    margin: Inset{top: 10. bottom: 2. left: 2. right: 4.}
                }
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                price := Label{
                    width: Fill
                    text: "0"
                    flow: Right{wrap: false}
                    max_lines: 1
                    draw_text.color: #xB43A12
                    draw_text.text_style.font_size: 14.
                }
                stock := Label{
                    width: Fill
                    text: " "
                    flow: Right{wrap: false}
                    max_lines: 1
                    text_overflow: Ellipsis
                    draw_text.color: #x6B4A3F
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
                        color: #x9A0E10
                        color_hover: #xB43A12
                        color_down: #x69090C
                        color_focus: #x9A0E10
                        color_2: #x9A0E10
                        color_2_hover: #xB43A12
                        color_2_down: #x69090C
                        color_2_focus: #x9A0E10
                        border_color: #x9A0E10
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
                color: #xB43A12
                color_hover: #xB43A12
                color_down: #x8A320C
                color_focus: #xB43A12
                color_2: #xB43A12
                color_2_hover: #xB43A12
                color_2_down: #x8A320C
                color_2_focus: #xB43A12
                border_color: #xB43A12
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
        draw_bg.color: #xEFD2BC
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        label := WrapLabel{
            body: "Item"
            font_color: #x69090C
            font_size: 16.
        }
        // Stitch qty dial: Volume − [N] +  (screen 03 / layout spec qty_stepper)
        RoundedView{
            width: Fill height: Fit
            padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
            flow: Right spacing: 8
            align: Align{y: 0.5}
            draw_bg.color: #xF9F4F0
            draw_bg.border_radius: 12.
            View{
                width: Fill height: Fit
                flow: Down spacing: 2
                Label{
                    text: "QTY"
                    draw_text.color: #x6B4A3F
                    draw_text.text_style.font_size: 10.
                }
            }
            minus := Button{
                text: "−"
                width: 44 height: 44
                draw_bg +: {
                    color: #xffffff
                    color_hover: #x9A0E10
                    color_down: #x69090C
                    color_focus: #xffffff
                    color_2: #xffffff
                    color_2_hover: #x9A0E10
                    color_2_down: #x69090C
                    color_2_focus: #xffffff
                    border_color: #x5A4238
                    border_color_hover: #x9A0E10
                    border_color_down: #x6B4A3F
                    border_color_focus: #x5A4238
                    border_radius: 12.
                    border_size: 2.
                }
                draw_text +: {
                    color: #x69090C
                    color_hover: #xffffff
                    color_down: #xffffff
                    color_focus: #x69090C
                    text_style +: {font_size: 22.}
                }
            }
            qty_label := Label{
                width: 40
                text: "1"
                draw_text.color: #x69090C
                draw_text.text_style.font_size: 26.
            }
            plus := Button{
                text: "+"
                width: 44 height: 44
                draw_bg +: {
                    color: #x9A0E10
                    color_hover: #xB43A12
                    color_down: #x69090C
                    color_focus: #x9A0E10
                    color_2: #x9A0E10
                    color_2_hover: #xB43A12
                    color_2_down: #x69090C
                    color_2_focus: #x9A0E10
                    border_color: #x9A0E10
                    border_color_hover: #xB43A12
                    border_color_down: #x69090C
                    border_color_focus: #x9A0E10
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
                draw_text.color: #x260B08
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
                font_color: #x260B08
                font_size: 13.
            }
            View{
                width: Fill height: Fit
                flow: Down spacing: 4
                Label{
                    text: "Applied"
                    draw_text.color: #xB43A12
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
            color: #x9A0E10
            color_hover: #xB43A12
            color_down: #x69090C
            color_focus: #x9A0E10
            color_disabled: #xA9806E
            color_2: #x9A0E10
            color_2_hover: #xB43A12
            color_2_down: #x69090C
            color_2_focus: #x9A0E10
            color_2_disabled: #xA9806E
            border_color: #x9A0E10
            border_color_hover: #xB43A12
            border_color_down: #x69090C
            border_color_focus: #x9A0E10
            border_radius: 12.
            border_size: 0.
        }
        draw_text +: {
            color: #xffffff
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #xffffff
            color_disabled: #xEFD2BC
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
        draw_bg.border_color: #x5A4238
        label := WrapLabel{
            body: "sale"
            font_color: #x260B08
            font_size: 13.
        }
        pick := CardActionBtn{text: "Select" height: 40}
    }

    let CustomerRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 16. right: 12. top: 12. bottom: 12.}
        flow: Right spacing: 12
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        View{
            width: Fill height: Fit
            flow: Down spacing: 2
            label := WrapLabel{
                body: "customer"
                font_color: #x260B08
                font_size: 14.
            }
            sublabel := WrapLabel{
                body: "id"
                font_color: #x6B4A3F
                font_size: 11.
            }
        }
        pick := Button{
            text: "Pick"
            width: 72 height: 44
            draw_bg +: {
                color: #x9A0E10
                color_hover: #xB43A12
                color_down: #x69090C
                color_focus: #x9A0E10
                color_2: #x9A0E10
                color_2_hover: #xB43A12
                color_2_down: #x69090C
                color_2_focus: #x9A0E10
                border_color: #x9A0E10
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
            color_hover: #x9A0E10
            color_down: #x69090C
            color_focus: #xffffff
            color_2: #xffffff
            color_2_hover: #x9A0E10
            color_2_down: #x69090C
            color_2_focus: #xffffff
            border_color: #x5A4238
            border_color_hover: #x9A0E10
            border_color_down: #x6B4A3F
            border_color_focus: #x5A4238
            border_radius: 14.
            border_size: 2.
        }
        draw_text +: {
            color: #x260B08
            color_hover: #xffffff
            color_down: #xffffff
            color_focus: #x260B08
            ink_centered: false
            text_style +: {font_size: 14.}
        }
    }

    let LocaleRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 16. right: 10. top: 10. bottom: 10.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        label := WrapLabel{
            width: Fill
            body: "locale"
            font_color: #x260B08
            font_size: 14.
        }
        pick := Button{
            text: "Use"
            width: 80 height: 48
            draw_bg +: {
                color: #x9A0E10
                color_hover: #xB43A12
                color_down: #x69090C
                color_focus: #x9A0E10
                color_2: #x9A0E10
                color_2_hover: #xB43A12
                color_2_down: #x69090C
                color_2_focus: #x9A0E10
                border_color: #x9A0E10
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

    // Avatar row for categories and catalogue dishes: photo when one
    // exists, otherwise the name-initials placeholder (ThumbAvatar).
    let CategoryRow = RoundedView{
        width: Fill height: Fit
        padding: Inset{left: 10. right: 10. top: 8. bottom: 8.}
        flow: Right spacing: 10
        align: Align{y: 0.5}
        draw_bg.color: #xffffff
        draw_bg.border_radius: 14.
        draw_bg.border_size: 2.
        draw_bg.border_color: #x5A4238
        ThumbAvatar{
            width: 48 height: 48
        }
        label := WrapLabel{
            width: Fill
            body: "item"
            font_color: #x260B08
            font_size: 14.
        }
        photo := Button{
            text: "Photo"
            width: Fit height: 40
            padding: Inset{left: 12. right: 12. top: 6. bottom: 6.}
            draw_bg +: {
                color: #xffffff
                color_hover: #x9A0E10
                color_down: #x69090C
                color_focus: #xffffff
                color_2: #xffffff
                color_2_hover: #x9A0E10
                color_2_down: #x69090C
                color_2_focus: #xffffff
                border_radius: 10.
                border_size: 2.
                border_color: #x9A0E10
                border_color_hover: #x9A0E10
                border_color_down: #x69090C
                border_color_focus: #x9A0E10
            }
            draw_text +: {
                color: #x93000a
                color_hover: #xffffff
                color_down: #xffffff
                color_focus: #x93000a
                text_style +: {font_size: 13.}
            }
        }
        pick := Button{
            text: "Use"
            width: 80 height: 48
            draw_bg +: {
                color: #x9A0E10
                color_hover: #xB43A12
                color_down: #x69090C
                color_focus: #x9A0E10
                color_2: #x9A0E10
                color_2_hover: #xB43A12
                color_2_down: #x69090C
                color_2_focus: #x9A0E10
                border_color: #x9A0E10
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

    mod.widgets.FloorListBase = #(FloorList::register_widget(vm))
    mod.widgets.FloorList = set_type_default() do mod.widgets.FloorListBase{
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
                floorcard := FloorCard{}
            }
        }
    }

    mod.widgets.KitchenListBase = #(KitchenList::register_widget(vm))
    mod.widgets.KitchenList = set_type_default() do mod.widgets.KitchenListBase{
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
                kotcard := KotCard{}
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
        margin: Inset{top: 4. bottom: 4.}
        list := PortalList{
            width: Fill height: Fill
            spacing: 6
            bounce_at_start: false
            bounce_at_end: false
            capture_overload: false
            scroll_bar: ScrollBar{}
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

    // Server pick rows use LocaleRow (~60px) + item pad — PortalList height
    // must be ≥ one full Open row or the pick button is clipped top/bottom.
    mod.widgets.ServerListBase = #(ServerList::register_widget(vm))
    mod.widgets.ServerList = set_type_default() do mod.widgets.ServerListBase{
        width: Fill height: Fit
        margin: Inset{top: 4. bottom: 4.}
        list := PortalList{
            width: Fill height: Fill
            spacing: 10
            scroll_bar: ScrollBar{}
            Item := View{
                width: Fill height: Fit
                padding: Inset{top: 2. bottom: 8.}
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

    mod.widgets.CatalogueCatListBase = #(CatalogueCatList::register_widget(vm))
    mod.widgets.CatalogueCatList = set_type_default() do mod.widgets.CatalogueCatListBase{
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
                padding: Inset{bottom: 10.}
                texture_caching: false
                LocaleRow{}
            }
        }
    }

    mod.widgets.CatalogueItemListBase = #(CatalogueItemList::register_widget(vm))
    mod.widgets.CatalogueItemList = set_type_default() do mod.widgets.CatalogueItemListBase{
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
                padding: Inset{bottom: 10.}
                texture_caching: false
                CategoryRow{}
            }
        }
    }

    mod.widgets.CategoryListBase = #(CategoryList::register_widget(vm))
    mod.widgets.CategoryList = set_type_default() do mod.widgets.CategoryListBase{
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
                padding: Inset{bottom: 10.}
                texture_caching: false
                CategoryRow{}
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
                pass.clear_color: #xF9F4F0
                caption_bar +: {
                    draw_bg.color: #x9A0E10
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
                            // Wide (≥720): side rail. Narrow: bottom tabs. See apply_shell_layout.
                            flow: Right spacing: 0

                    // Tablet / desktop nav. More hub expanded into its destinations.
                    nav_rail := SolidView{
                        visible: false
                        width: 220 height: Fill
                        draw_bg.color: #x69090C
                        rail_scroll := ScrollYView{
                            width: Fill height: Fill
                            View{
                                width: Fill height: Fit
                                padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
                                flow: Down spacing: 6
                                rail_books_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_books := TabBtn{text: "Books"}
                                    rail_books_on := TabBtnActive{text: "Books" visible: false}
                                }
                                rail_sell_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_sell := TabBtn{text: "Order"}
                                    rail_sell_on := TabBtnActive{text: "Order" visible: false}
                                }
                                rail_buy_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_buy := TabBtn{text: "Buy"}
                                    rail_buy_on := TabBtnActive{text: "Buy" visible: false}
                                }
                                rail_rates_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_rates := TabBtn{text: "Menu"}
                                    rail_rates_on := TabBtnActive{text: "Menu" visible: false}
                                }
                                rail_ops_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_ops := TabBtn{text: "Floor"}
                                    rail_ops_on := TabBtnActive{text: "Floor" visible: false}
                                }
                                rail_cod_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_cod := TabBtn{text: "Kitchen"}
                                    rail_cod_on := TabBtnActive{text: "Kitchen" visible: false}
                                }
                                rail_sales_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_sales := TabBtn{text: "Sales"}
                                    rail_sales_on := TabBtnActive{text: "Sales" visible: false}
                                }
                                rail_day_close_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_day_close := TabBtn{text: "Day close"}
                                    rail_day_close_on := TabBtnActive{text: "Day close" visible: false}
                                }
                                rail_reports_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_reports := TabBtn{text: "Reports"}
                                    rail_reports_on := TabBtnActive{text: "Reports" visible: false}
                                }
                                rail_inventory_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_inventory := TabBtn{text: "Stock"}
                                    rail_inventory_on := TabBtnActive{text: "Stock" visible: false}
                                }
                                rail_branches_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_branches := TabBtn{text: "Branches"}
                                    rail_branches_on := TabBtnActive{text: "Branches" visible: false}
                                }
                                rail_credit_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_credit := TabBtn{text: "Parties"}
                                    rail_credit_on := TabBtnActive{text: "Parties" visible: false}
                                }
                                rail_team_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_team := TabBtn{text: "Team"}
                                    rail_team_on := TabBtnActive{text: "Team" visible: false}
                                }
                                rail_settings_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_settings := TabBtn{text: "Settings"}
                                    rail_settings_on := TabBtnActive{text: "Settings" visible: false}
                                }
                                rail_lock_slot := View{
                                    width: Fill height: Fit
                                    flow: Overlay
                                    rail_lock := TabBtn{text: "Lock"}
                                    rail_lock_on := TabBtnActive{text: "Lock" visible: false}
                                }
                            }
                        }
                    }
                    // Drag handle between rail and main (wider than the hairline for hit).
                    rail_divider := SolidView{
                        visible: false
                        width: 6 height: Fill
                        cursor: MouseCursor.ColResize
                        draw_bg.color: #x9A0E10
                    }
                    main_column := View{
                        width: Fill height: Fill
                        flow: Down spacing: 0

                    header := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 16. right: 16. top: 12. bottom: 12.}
                        flow: Down spacing: 8
                        draw_bg.color: #x69090C

                        View{
                            width: Fill height: Fit
                            flow: Right spacing: 10
                            align: Align{y: 0.5}
                            brand_mark := Image{
                                width: 36 height: 36
                                fit: ImageFit.Stretch
                            }
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 2
                                brand_title := WrapLabel{
                                    width: Fill
                                    body: "Khaynjo"
                                    font_color: #xffffff
                                    font_size: 18.
                                }
                                session_label := WrapLabel{
                                    width: Fill
                                    body: "Sell"
                                    font_color: #xC4A18C
                                    font_size: 11.
                                }
                            }
                            // Fit + no ellipsis — never pin/crop against the
                            // window edge (header already has right padding).
                            shop_badge := Label{
                                width: Fit height: Fit
                                text: "Shop"
                                margin: Inset{left: 4. right: 0. top: 0. bottom: 0.}
                                draw_text.color: #xfe8357
                                draw_text.text_style.font_size: 11.
                            }
                        }
                    }

                    today_bar := SolidView{
                        width: Fill height: Fit
                        flow: Down
                        padding: Inset{left: 16. right: 16. top: 8. bottom: 8.}
                        draw_bg.color: #xEFD2BC
                        today_label := WrapLabel{
                            width: Fill
                            body: "Today: —"
                            font_color: #x260B08
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
                                    draw_bg.color: #x9A0E10
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
                                        draw_text.color: #xF7B043
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
                                    flow: Down spacing: 14
                                    draw_bg.color: #xF9F4F0
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xEFD2BC
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 12
                                            align: Align{y: 0.5}
                                            setup_brand_mark := Image{
                                                width: 48 height: 48
                                                fit: ImageFit.Stretch
                                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 4
                                                setup_brand_title := WrapLabel{
                                                    body: "Your shop"
                                                    font_color: #x69090C
                                                    font_size: 18.
                                                }
                                                device_use_question := WrapLabel{
                                                    body: "How will this device be used?"
                                                    font_color: #x6B4A3F
                                                    font_size: 12.
                                                }
                                            }
                                        }
                                    }

                                    setup_lang_title := WrapLabel{
                                        body: "Language"
                                        font_color: #x260B08
                                        font_size: 15.
                                    }
                                    setup_lang_hint := WrapLabel{
                                        width: Fill
                                        body: "Follow this device’s language, or pick one yourself."
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    ActionStack{
                                        View{
                                            width: Fill height: Fit
                                            flow: Overlay
                                            lang_setup_platform := StackSecondaryBtn{text: "Use platform native language"}
                                            lang_setup_platform_on := StackPrimaryBtn{text: "Use platform native language" visible: false}
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Overlay
                                            lang_setup_user := StackSecondaryBtn{text: "Use user selected language"}
                                            lang_setup_user_on := StackPrimaryBtn{text: "Use user selected language" visible: false}
                                        }
                                    }
                                    setup_lang_selected := WrapLabel{
                                        body: ""
                                        font_color: #x260B08
                                        font_size: 13.
                                    }
                                    setup_lang_list := mod.widgets.LanguageList{}

                                    device_section_label := WrapLabel{
                                        body: "THIS DEVICE"
                                        font_color: #x260B08
                                        font_size: 11.
                                    }

                                    ActionStack{
                                        device_role_server := StackSecondaryBtn{text: "Server Mode"}
                                        device_role_server_on := StackPrimaryBtn{text: "Server Mode" visible: false}
                                        device_role_client := StackSecondaryBtn{text: "Client Mode"}
                                        device_role_client_on := StackPrimaryBtn{text: "Client Mode" visible: false}
                                    }

                                    device_setup_help := WrapLabel{
                                        width: Fill
                                        body: "Server Mode: this device holds the books. Client Mode: pair to a Server. Asked every launch — pick again or switch. Extra tills stay off until you allow them in Backup & settings."
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }

                                    server_pick_section := View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 12
                                        visible: false
                                        server_pick_title := WrapLabel{
                                            body: "Choose a server"
                                            font_color: #x260B08
                                            font_size: 15.
                                        }
                                        server_pick_hint := WrapLabel{
                                            width: Fill
                                            body: "This device already has servers. Open one, or create a new server for another business."
                                            font_color: #x6B4A3F
                                            font_size: 12.
                                        }
                                        server_name_input := FieldInput{empty_text: "New server name (optional)"}
                                        server_create_new := NavBtn{text: "Create new server"}
                                        setup_server_list := mod.widgets.ServerList{}
                                        server_pick_cancel := BackBtn{text: "Back"}
                                    }

                                    device_setup_status := WrapLabel{
                                        body: " "
                                        font_color: #xB43A12
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
                                    draw_bg.color: #x9A0E10
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
                                        draw_text.color: #xF7B043
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                // Brand + lock card
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 16. bottom: 16.}
                                    flow: Down spacing: 14
                                    draw_bg.color: #xF9F4F0
                                    draw_bg.border_radius: 16.

                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        flow: Down spacing: 8
                                        draw_bg.color: #xffffff
                                        draw_bg.border_radius: 12.
                                        draw_bg.border_size: 1.
                                        draw_bg.border_color: #xEFD2BC
                                        View{
                                            width: Fill height: Fit
                                            flow: Right spacing: 12
                                            align: Align{y: 0.5}
                                            lock_brand_mark := Image{
                                                width: 48 height: 48
                                                fit: ImageFit.Stretch
                                                            }
                                            View{
                                                width: Fill height: Fit
                                                flow: Down spacing: 4
                                                lock_brand_title := WrapLabel{
                                                    body: "Your shop"
                                                    font_color: #x69090C
                                                    font_size: 18.
                                                }
                                                WrapLabel{
                                                    body: "Counter 01 · Locked"
                                                    font_color: #x6B4A3F
                                                    font_size: 12.
                                                }
                                            }
                                            Label{
                                                text: "Shop"
                                                draw_text.color: #xB43A12
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
                                            font_color: #x260B08
                                            font_size: 11.
                                        }
                                        lock_staff_label := WrapLabel{
                                            body: "No staff yet — unlock as Owner, then Team"
                                            font_color: #x260B08
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
                                        draw_bg.border_color: #xEFD2BC
                                        pin_prompt := WrapLabel{
                                            body: "ENTER OWNER PASSCODE"
                                            font_color: #x260B08
                                            font_size: 11.
                                        }
                                        pin_label := Label{
                                            text: "—"
                                            draw_text.color: #x69090C
                                            draw_text.text_style.font_size: 26.
                                        }
                                        pin_count := WrapLabel{
                                            body: "0 / 12 digits · at least 4, then OK"
                                            font_color: #x6B4A3F
                                            font_size: 12.
                                        }
                                        pin_reveal := ChipBtn{
                                            text: "Show PIN"
                                            padding: Inset{left: 14. right: 14.}
                                        }
                                        pin_hint := WrapLabel{
                                            body: "4–12 digits · keyboard or pad · Enter = OK · tap Show PIN to check"
                                            font_color: #x6B4A3F
                                            font_size: 12.
                                        }
                                        pin_status := WrapLabel{
                                            body: " "
                                            font_color: #xB43A12
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
                                        font_color: #x260B08
                                        font_size: 11.
                                    }
                                    WrapLabel{
                                        body: "Or unlock with login id + password"
                                        font_color: #x6B4A3F
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
                                    body: "Set up Khaynjo"
                                    font_color: #x260B08
                                    font_size: 22.
                                }
                                onboard_status := WrapLabel{
                                    body: "Name this counter if you want, then Start fresh or Bring my book."
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                onboard_venue_input := FieldInput{empty_text: "Restaurant name (optional)"}
                                onboard_org_input := FieldInput{empty_text: "Organization (optional)"}
                                onboard_save_venue := NavBtn{text: "Save names"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 12
                                    align: Align{y: 0.5}
                                    onboard_brand_mark := Image{
                                        width: 48 height: 48
                                        fit: ImageFit.Stretch
                                            }
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 4
                                        WrapLabel{
                                            body: "Shop logo (optional)"
                                            font_color: #x260B08
                                            font_size: 15.
                                        }
                                        onboard_icon_hint := WrapLabel{
                                            body: "Logo: Kumkaj mark (default) — choose your own if you have one"
                                            font_color: #x6B4A3F
                                            font_size: 12.
                                        }
                                    }
                                }
                                onboard_icon_pick := NavBtn{text: "Choose logo"}
                                WrapLabel{
                                    body: "Product: Khaynjo — restaurant OS: floor, KOT, billing & books (locked for this app)."
                                    font_color: #x260B08
                                    font_size: 15.
                                }
                                onboard_tmpl_hint := WrapLabel{
                                    body: "Order = counter billing. Floor = tables & KOT. Day book = today. Menu, Parties, Reports & Team sit in the side list."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                WrapLabel{
                                    body: "How do you want to start? (tap one — it stays highlighted)"
                                    font_color: #x260B08
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
                                    body: "Fresh = empty menu (add dishes later in Menu). Book = type opening items and who owes you below."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                onboard_fresh_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    WrapLabel{
                                        body: "Fresh selected — catalogue stays empty. Tap Done when ready."
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                }
                                onboard_book_panel := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    visible: false
                                    WrapLabel{body: "Book — add dish / item" font_color: #x260B08 font_size: 15.}
                                    onboard_item_name := FieldInput{empty_text: "Item name"}
                                    onboard_item_price := FieldInput{empty_text: "Price"}
                                    onboard_add_item := NavBtn{text: "Add item"}
                                    WrapLabel{body: "Book — opening credit" font_color: #x260B08 font_size: 15.}
                                    onboard_cust_name := FieldInput{empty_text: "Customer name"}
                                    onboard_cust_bal := FieldInput{empty_text: "Amount owed"}
                                    onboard_add_credit := NavBtn{text: "Add opening balance"}
                                }
                                onboard_done := PinOkKey{text: "Done — open books"}
                                WrapLabel{
                                    body: "Later: Settings → apply your country pack for tax and pay labels."
                                    font_color: #x6B4A3F
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
                                        font_color: #x260B08
                                        font_size: 22.
                                    }
                                }
                                client_pair_status := WrapLabel{
                                    body: "Enter the Server IP and 6-digit pairing code. Or tap Back to choose Server Mode instead."
                                    font_color: #x6B4A3F
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
                                    font_color: #x260B08
                                    font_size: 22.
                                }
                                books_help := WrapLabel{
                                    width: Fill
                                    body: "Today: money in, purchases/expenses out, they owe us / we owe them, cash vs bank, gross and net."
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                // Today's figures as stat cards (label/value
                                // rows) — set from Store::books_overview in
                                // sync_chrome. Replaces the old one-paragraph
                                // text wall.
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    Label{text: "TODAY" draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Money in (sales)" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_in_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Purchases (COGS)" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_purchases_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Expenses" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_expenses_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Gross" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_gross_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Net" draw_text.color: #x9A0E10 draw_text.text_style.font_size: 13.}
                                        books_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x9A0E10 draw_text.text_style.font_size: 15.}
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    Label{text: "BILLS" draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        books_declared_name := Label{width: Fill height: Fit text: "Tax invoice" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_declared_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        books_owner_name := Label{width: Fill height: Fit text: "Cash memo" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_owner_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    Label{text: "PAYMENTS TODAY" draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Cash" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_cash_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        books_digital_name := Label{width: Fill height: Fit text: "UPI" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_digital_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Card" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_card_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    Label{text: "OUTSTANDING" draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "They owe us" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_they_owe_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x137333 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "We owe them" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_we_owe_value := Label{width: Fit height: Fit text: "—" draw_text.color: #xB3261E draw_text.text_style.font_size: 13.}
                                    }
                                }
                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 14. right: 14. top: 10. bottom: 10.}
                                    flow: Down spacing: 7
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 14.
                                    Label{text: "BANK" draw_text.color: #x93705F draw_text.text_style.font_size: 10.}
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Bank balance" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_bank_balance_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Cash in hand (till)" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_till_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                    View{width: Fill height: Fit flow: Right spacing: 8 align: Align{y: 0.5}
                                        Label{width: Fill height: Fit text: "Bank net today" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 12.}
                                        books_bank_net_value := Label{width: Fit height: Fit text: "—" draw_text.color: #x260B08 draw_text.text_style.font_size: 13.}
                                    }
                                }
                                ActionStack{
                                    books_to_sell := RowSecondaryBtn{text: "Sell (money in)"}
                                    books_to_buy := RowSecondaryBtn{text: "Buy (money out)"}
                                    books_day_close := RowSecondaryBtn{text: "Day close"}
                                    books_reports := RowSecondaryBtn{text: "Reports & bank"}
                                    books_credit := RowSecondaryBtn{text: "Parties / credit"}
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
                                    draw_bg.color: #xF6E3D3
                                    draw_bg.border_radius: 12.
                                    View{
                                        width: Fill height: Fit
                                        flow: Down spacing: 2
                                        session_chip := WrapLabel{
                                            body: "Counter"
                                            font_color: #x260B08
                                            font_size: 12.
                                        }
                                        table_chip := WrapLabel{
                                            body: "Ready to order"
                                            font_color: #x260B08
                                            font_size: 11.
                                        }
                                    }
                                    Label{
                                        text: "LOCAL"
                                        draw_text.color: #x9A6206
                                        draw_text.text_style.font_size: 10.
                                    }
                                }

                                // apply_shell_layout sets flow (Right vs Down) + column widths.
                                // Default stacked full-width so first paint never leaves a Fixed gutter.
                                pos_split := View{
                                    width: Fill height: Fit
                                    flow: Down
                                    spacing: 12
                                pos_catalog := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                order_help := WrapLabel{
                                    body: "Tap an item to add it."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                empty_catalog := WrapLabel{
                                    body: "No items yet. Add your dishes and items to start taking orders."
                                    font_color: #x260B08
                                    font_size: 14.
                                }
                                catalog_cta := ActionStack{
                                    add_items_btn := StackSecondaryBtn{text: "Add dishes & items"}
                                }
                                product_list := mod.widgets.ProductList{}
                                } // pos_catalog

                                pos_cart_col := View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 10
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 12
                                    align: Align{y: 0.5}
                                    padding: Inset{top: 4. bottom: 4.}
                                    receipt_label := WrapLabel{
                                        width: Fit
                                        body: "Ready"
                                        font_color: #x6B4A3F
                                        font_size: 13.
                                    }
                                    View{width: Fill height: 1}
                                    clear_cart := Button{
                                        text: "Clear"
                                        width: Fit height: 32
                                        padding: Inset{left: 14. right: 14. top: 4. bottom: 4.}
                                        draw_bg +: {
                                            color: #xEFD2BC
                                            color_hover: #x9A0E10
                                            color_down: #x69090C
                                            color_focus: #xEFD2BC
                                            border_color: #xD8BBA6
                                            border_radius: 8.
                                        }
                                        draw_text +: {
                                            color: #x93000a
                                            color_hover: #xffffff
                                            color_down: #xffffff
                                            color_focus: #x93000a
                                            text_style +: {font_size: 13.}
                                        }
                                    }
                                }

                                // Order type: 2x2 segmented grid — all four always
                                // visible at ticket width, nothing to scroll, nothing
                                // clipped (a ChipStrip here half-hid chips at 1024px).
                                View{
                                    width: Fill height: Fit
                                    flow: Down spacing: 8
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        View{width: Fill height: 40 flow: Overlay
                                            otype_counter := ChipBtn{width: Fill text: "Counter"}
                                            otype_counter_on := ChipBtnActive{width: Fill text: "Counter" visible: false}
                                        }
                                        View{width: Fill height: 40 flow: Overlay
                                            otype_dinein := ChipBtn{width: Fill text: "Dine-in"}
                                            otype_dinein_on := ChipBtnActive{width: Fill text: "Dine-in" visible: false}
                                        }
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        View{width: Fill height: 40 flow: Overlay
                                            otype_takeaway := ChipBtn{width: Fill text: "Takeaway"}
                                            otype_takeaway_on := ChipBtnActive{width: Fill text: "Takeaway" visible: false}
                                        }
                                        View{width: Fill height: 40 flow: Overlay
                                            otype_delivery := ChipBtn{width: Fill text: "Delivery"}
                                            otype_delivery_on := ChipBtnActive{width: Fill text: "Delivery" visible: false}
                                        }
                                    }
                                }
                                order_table_label := WrapLabel{
                                    body: "Counter order"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                ActionStack{
                                    send_kot_btn := RowPrimaryBtn{text: "Send KOT to kitchen"}
                                    next_order_btn := RowSecondaryBtn{text: "Next order (clear desk)" visible: false}
                                }

                                RoundedView{
                                    width: Fill height: Fit
                                    padding: Inset{left: 16. right: 16. top: 14. bottom: 14.}
                                    flow: Down spacing: 10
                                    draw_bg.color: #xffffff
                                    draw_bg.border_radius: 16.
                                    draw_bg.border_size: 2.
                                    draw_bg.border_color: #x5A4238
                                    cart_list := mod.widgets.CartList{}
                                    cart_empty_hint := WrapLabel{
                                        body: "Order empty — tap items to add."
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }

                                    // Triple totals strip (Actual | Applied | Rounded)
                                    RoundedView{
                                        width: Fill height: Fit
                                        padding: Inset{left: 8. right: 8. top: 8. bottom: 8.}
                                        flow: Right spacing: 4
                                        draw_bg.color: #xEFD2BC
                                        draw_bg.border_radius: 12.
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            WrapLabel{
                                                body: "ACTUAL TOTAL"
                                                font_color: #x6B4A3F
                                                font_size: 10.
                                            }
                                            cart_natural_label := WrapLabel{
                                                body: "0"
                                                font_color: #x260B08
                                                font_size: 14.
                                            }
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            WrapLabel{
                                                body: "APPLIED TOTAL"
                                                font_color: #x6B4A3F
                                                font_size: 10.
                                            }
                                            cart_applied_label := WrapLabel{
                                                body: "0"
                                                font_color: #x260B08
                                                font_size: 14.
                                            }
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Down spacing: 2
                                            align: Align{x: 0.5}
                                            cart_charge_caption := WrapLabel{
                                                body: "ROUNDED"
                                                font_color: #xB43A12
                                                font_size: 10.
                                            }
                                            cart_total_input := CartField{
                                                empty_text: "0"
                                                text: "0"
                                                draw_bg +: {
                                                    border_color: #xB43A12
                                                    border_color_hover: #x822801
                                                    border_color_focus: #x69090C
                                                    border_radius: 8.
                                                    border_size: 1.
                                                }
                                                draw_text +: {
                                                    color: #x822801
                                                    color_hover: #x822801
                                                    color_down: #x822801
                                                    color_focus: #x822801
                                                    color_empty: #x6B4A3F
                                                    color_empty_hover: #x260B08
                                                    color_empty_focus: #x260B08
                                                    text_style +: {font_size: 16.}
                                                }
                                            }
                                        }
                                    }

                                    // Exclusive-tax shops quote net prices, so the strip above is
                                    // not what the customer pays — this line is.
                                    tax_estimate_label := WrapLabel{
                                        body: " "
                                        font_color: #xB43A12
                                        font_size: 13.
                                    }

                                    // Amounts only — full bill in one field, or mix that adds up.
                                    WrapLabel{
                                        body: "Split across cash / UPI / card / credit if needed. Credit needs a customer above."
                                        font_color: #x6B4A3F
                                        font_size: 11.
                                    }
                                    money_lane_label := WrapLabel{
                                        body: "Bill: Tax invoice"
                                        font_color: #x6B4A3F
                                        font_size: 12.
                                    }
                                    ActionStack{
                                        View{
                                            width: Fill height: Fit
                                            flow: Overlay
                                            lane_declared := StackSecondaryBtn{text: "Tax invoice"}
                                            lane_declared_on := StackPrimaryBtn{text: "Tax invoice" visible: false}
                                        }
                                        View{
                                            width: Fill height: Fit
                                            flow: Overlay
                                            lane_owner := StackSecondaryBtn{text: "Cash memo"}
                                            lane_owner_on := StackPrimaryBtn{text: "Cash memo" visible: false}
                                        }
                                    }
                                    mix_cash_input := FieldInput{empty_text: "Cash amount"}
                                    mix_upi_input := FieldInput{empty_text: "UPI amount"}
                                    mix_card_input := FieldInput{empty_text: "Card amount"}
                                    mix_credit_input := FieldInput{empty_text: "Credit / khata amount"}
                                    ActionStack{
                                        pay_mix := PayBtn{text: "Take payment"}
                                    }

                                }

                                cart_round_hint := WrapLabel{
                                    body: "Rounded starts equal to Applied. Edit only to round the bill."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                credit_chip := WrapLabel{
                                    body: "No customer yet — type a name, then tap Credit"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                customer_name_input := FieldInput{
                                    empty_text: "Customer name"
                                }
                                customer_use := NavBtn{text: "Add / use this name"}
                                customer_clear := NavBtn{text: "Clear customer"}
                                customers_empty_hint := WrapLabel{
                                    body: "Saved customers show up here for quick pick."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                customer_list_sell := mod.widgets.CustomerList{}
                                } // pos_cart_col
                                } // pos_split
                            }
                        }

                        floor_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                floor_heading := WrapLabel{body: "Floor" font_color: #x260B08 font_size: 20.}
                                floor_help := WrapLabel{
                                    body: "Tap a table to start or open its order. Running tables show time and amount. When guests are done, Request bill — the table shows Payment pending and frees only when the payment is recorded at the counter."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                floor_list := mod.widgets.FloorList{}
                                floor_add_heading := WrapLabel{body: "Add a table" font_color: #x260B08 font_size: 15.}
                                table_name_input := FieldInput{empty_text: "Table name (e.g. Patio 1, Counter 2)"}
                                table_add := NavBtn{text: "Add table"}
                            }
                        }

                        kitchen_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                kitchen_heading := WrapLabel{body: "Kitchen" font_color: #x260B08 font_size: 20.}
                                kitchen_help := WrapLabel{
                                    body: "Kitchen tickets (KOT), newest last. Start → Ready → Done moves a ticket along. Tickets never show prices."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                kitchen_list := mod.widgets.KitchenList{}
                                kitchen_empty := WrapLabel{
                                    body: "No open tickets. Send a KOT from the Ordering screen."
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                station_add_heading := WrapLabel{body: "Add a kitchen station" font_color: #x260B08 font_size: 15.}
                                station_name_input := FieldInput{empty_text: "Station name (e.g. Tandoor, Fryer, Cold)"}
                                station_add := NavBtn{text: "Add station"}
                            }
                        }

                        menu_screen := ScrollYView{
                            width: Fill height: Fill
                            visible: false
                            View{
                                width: Fill height: Fit
                                flow: Down spacing: 10
                                padding: Inset{left: 16. right: 16. top: 8. bottom: 120.}
                                items_heading := WrapLabel{body: "Your items" font_color: #x260B08 font_size: 20.}
                                items_help_label := WrapLabel{body: "Tap an item to edit it. New item clears the form so you can add a different one. Sales already recorded keep the name they were charged with." font_color: #x260B08}
                                menu_mode_label := WrapLabel{body: "Adding a new item" font_color: #x69090C font_size: 14.}
                                menu_name_input := FieldInput{
                                    empty_text: "Item name — type it in your language"
                                }
                                menu_aliases_input := FieldInput{
                                    empty_text: "Aliases — other names, comma separated"
                                }
                                FormFieldRow{
                                    menu_price_input := FieldInput{empty_text: "0"}
                                    menu_unit_input := FieldInput{empty_text: "Unit (plate / pc — blank = each)"}
                                }
                                ActionStack{
                                    menu_new := StackSecondaryBtn{
                                        text: "New item"
                                        visible: false
                                    }
                                    menu_add := StackPrimaryBtn{text: "Add item"}
                                }
                                menu_tax_input := FieldInput{
                                    empty_text: "Tax rate name or % — Standard, Reduced, 0, 5, 12, 18, 20"
                                }
                                menu_category_input := FieldInput{empty_text: "Category (optional, created if new)"}
                                menu_station_input := FieldInput{empty_text: "Kitchen station (optional, e.g. Tandoor — created if new)"}
                                menu_modifiers_input := FieldInput{empty_text: "Options (optional) — Extra cheese=40, Large=80"}
                                image_hint := WrapLabel{
                                    body: "Image: auto (from name)"
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                draft_preview := ThumbAvatar{
                                    width: 96 height: 96
                                    margin: Inset{top: 0. bottom: 0. left: 0. right: 0.}
                                }
                                ActionStack{
                                    image_clear := RowSecondaryBtn{text: "Clear photo"}
                                    image_pick := RowPrimaryBtn{text: "Pick photo"}
                                }
                                menu_empty := WrapLabel{
                                    body: "No items yet — add one above."
                                    font_color: #x260B08
                                    font_size: 14.
                                }
                                product_list_menu := mod.widgets.MenuList{}
                                WrapLabel{body: "Your categories" font_color: #x260B08 font_size: 15.}
                                categories_help := WrapLabel{
                                    body: "Tap Use to file the item above under that category. Photo gives the category a picture; without one it shows its initials."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                category_list := mod.widgets.CategoryList{}
                                WrapLabel{body: "Pick from our items" font_color: #x260B08 font_size: 15.}
                                catalogue_help := WrapLabel{
                                    body: "Browse the shelf and tap Use on a dish — its name and photo fill the form above. Set your price and your category, then Add item. The shelf folder is only for finding dishes; the category is always yours to choose."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                catalogue_cat_list := mod.widgets.CatalogueCatList{}
                                catalogue_item_list := mod.widgets.CatalogueItemList{}
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
                                    font_color: #x260B08
                                    font_size: 22.
                                }
                                buy_help := WrapLabel{
                                    width: Fill
                                    body: "Money out: goods received (purchase) or running costs (expense). Same books as Sell."
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                WrapLabel{body: "Goods received" font_color: #x260B08 font_size: 15.}
                                buy_item_input := FieldInput{empty_text: "Item name"}
                                buy_qty_input := FieldInput{empty_text: "Qty (e.g. 1.5 for kg)"}
                                buy_cost_input := FieldInput{empty_text: "Unit cost"}
                                buy_supplier_input := FieldInput{empty_text: "Supplier name (or party)"}
                                buy_party_kind := FieldInput{empty_text: "New party kind (Supplier)"}
                                buy_note_input := FieldInput{empty_text: "Note (optional)"}
                                // Multi-word — ActionStack, not ChipStrip (160px truncates).
                                ActionStack{
                                    View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        buy_paid := StackSecondaryBtn{text: "Paid now"}
                                        buy_paid_on := StackPrimaryBtn{text: "Paid now" visible: false}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        buy_on_account := StackSecondaryBtn{text: "On account (AP)"}
                                        buy_on_account_on := StackPrimaryBtn{text: "On account (AP)" visible: false}
                                    }
                                }
                                buy_purchase := NavBtn{text: "Record purchase"}
                                WrapLabel{body: "Expense" font_color: #x260B08 font_size: 15.}
                                buy_expense_amount := FieldInput{empty_text: "Amount"}
                                buy_expense_note := FieldInput{empty_text: "Note (fuel, rent, …)"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    Label{text: "Paid by" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 11.}
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t1_cash := ChipBtn{text: "Cash"}
                                        buy_exp_t1_cash_on := ChipBtnActive{text: "Cash" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t1_bank := ChipBtn{text: "Bank"}
                                        buy_exp_t1_bank_on := ChipBtnActive{text: "Bank" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t1_upi := ChipBtn{text: "UPI"}
                                        buy_exp_t1_upi_on := ChipBtnActive{text: "UPI" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t1_card := ChipBtn{text: "Card"}
                                        buy_exp_t1_card_on := ChipBtnActive{text: "Card" visible: false}
                                    }
                                }
                                buy_exp_part2 := FieldInput{empty_text: "Part by a second way (optional)"}
                                View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    Label{text: "Second way" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 11.}
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t2_cash := ChipBtn{text: "Cash"}
                                        buy_exp_t2_cash_on := ChipBtnActive{text: "Cash" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t2_bank := ChipBtn{text: "Bank"}
                                        buy_exp_t2_bank_on := ChipBtnActive{text: "Bank" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t2_upi := ChipBtn{text: "UPI"}
                                        buy_exp_t2_upi_on := ChipBtnActive{text: "UPI" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        buy_exp_t2_card := ChipBtn{text: "Card"}
                                        buy_exp_t2_card_on := ChipBtnActive{text: "Card" visible: false}
                                    }
                                }
                                buy_expense_add := NavBtn{text: "Add expense"}
                                buy_to_inventory := NavBtn{text: "Inventory & stock adjust"}
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
                                    sales_title := WrapLabel{body: "Sales" font_color: #x260B08 font_size: 20.}
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Select a sale to amend, void, or refund. Amendments keep full history."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                sale_selected_label := WrapLabel{
                                    body: "No sale selected"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                receipt_share_btn := NavBtn{text: "Copy receipt"}
                                receipt_print_btn := NavBtn{text: "Print / open"}
                                day_close_from_sales := NavBtn{text: "Day close summary"}
                                amend_cart_list := mod.widgets.CartList{}
                                amend_natural_label := WrapLabel{
                                    body: "Actual Total 0"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                amend_applied_label := WrapLabel{
                                    body: "Applied Total 0"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                    View{
                                        width: Fill height: Fit
                                        flow: Right spacing: 8
                                        align: Align{y: 0.5}
                                        WrapLabel{
                                            body: "Rounded Total"
                                            font_color: #xB43A12
                                            font_size: 14.
                                        }
                                        amend_total_input := CartField{
                                        padding: Inset{left: 14. right: 14. top: 14. bottom: 14.}
                                        empty_text: "0"
                                        text: "0"
                                        draw_bg +: {
                                            border_color: #xB43A12
                                            border_color_hover: #x822801
                                            border_color_focus: #x69090C
                                            border_radius: 12.
                                            border_size: 2.
                                        }
                                        draw_text +: {
                                            color: #x822801
                                            color_hover: #x822801
                                            color_down: #x822801
                                            color_focus: #x822801
                                            color_empty: #x6B4A3F
                                            color_empty_hover: #x260B08
                                            color_empty_focus: #x260B08
                                            text_style +: {font_size: 16.}
                                        }
                                    }
                                }
                                amend_round_hint := WrapLabel{
                                    body: "Rounded Total starts equal to Applied Total. Edit only to round the bill."
                                    font_color: #x260B08
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
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                ActionStack{
                                    amend_btn := NavBtn{text: "Amend"}
                                    void_btn := NavBtn{text: "Void"}
                                    refund_btn := NavBtn{text: "Refund"}
                                }
                                sale_history_label := WrapLabel{
                                    body: "History: —"
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                WrapLabel{body: "Notes & attachments (selected sale)" font_color: #x260B08 font_size: 15.}
                                sale_attach_note := FieldInput{empty_text: "Extra note on this bill / sale"}
                                ActionStack{
                                    sale_attach_add_note := RowPrimaryBtn{text: "Add note"}
                                    sale_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                    sale_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                }
                                sale_attach_list := WrapLabel{
                                    body: "(select a sale)"
                                    font_color: #x6B4A3F
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
                                    credit_title := WrapLabel{body: "Credit" font_color: #x260B08 font_size: 20.}
                                }
                                credit_help := WrapLabel{
                                    width: Fill
                                    body: "Each party has a unique id (names may repeat). A Customer runs a khata tab (owes us); a Supplier sells us stock (we owe them); Both does the two. Khata and payables stay separate; net who-owes-whom is in Reports."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                credit_summary_label := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                party_selected_label := WrapLabel{
                                    body: "New party — fill the form and tap Add party, or Pick a party from the list to edit it."
                                    font_color: #x9A0E10
                                    font_size: 13.
                                }
                                FormFieldRow{
                                    credit_add_name := FieldInput{empty_text: "Party name"}
                                    credit_add_phone := FieldInput{empty_text: "Phone"}
                                }
                                WrapLabel{
                                    body: "Account kind (tap one)"
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                ChipStrip{
                                    ChipStripRow{
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            credit_kind_customer := ChipBtn{text: "Customer"}
                                            credit_kind_customer_on := ChipBtnActive{text: "Customer" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            credit_kind_both := ChipBtn{text: "Both"}
                                            credit_kind_both_on := ChipBtnActive{text: "Both" visible: false}
                                        }
                                        View{
                                            width: Fit height: 40
                                            flow: Overlay
                                            credit_kind_supplier := ChipBtn{text: "Supplier"}
                                            credit_kind_supplier_on := ChipBtnActive{text: "Supplier" visible: false}
                                        }
                                    }
                                }
                                credit_add_kind := FieldInput{empty_text: "Roles (Customer, Supplier, Both…)"}
                                credit_add_btn := NavBtn{text: "Add party"}
                                ActionStack{
                                    party_form_clear := RowSecondaryBtn{text: "New party (clear form)"}
                                    party_delete := RowSecondaryBtn{text: "Delete selected party"}
                                }
                                customer_list_credit := mod.widgets.CustomerList{}
                                WrapLabel{body: "Credit limit (selected / named party)" font_color: #x260B08 font_size: 15.}
                                FormFieldRow{
                                    credit_limit_input := FieldInput{empty_text: "Credit limit (0 = none)"}
                                    credit_limit_party := FieldInput{empty_text: "Party name (or use selected)"}
                                }
                                ActionStack{
                                    credit_limit_set := RowPrimaryBtn{text: "Set credit limit"}
                                }
                                WrapLabel{body: "Pay selected (settle debt / take advance)" font_color: #x260B08 font_size: 15.}
                                settle_amount_input := FieldInput{empty_text: "Amount received"}
                                ActionStack{
                                    settle_cash := NavBtn{text: "Pay cash"}
                                    settle_upi := NavBtn{text: "Pay UPI"}
                                    settle_card := NavBtn{text: "Pay card"}
                                }
                                settle_open_bills := WrapLabel{
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                settle_bill_input := FieldInput{empty_text: "Against bill # above (optional)"}
                                WrapLabel{body: "Write off (clear without payment)" font_color: #x260B08 font_size: 15.}
                                writeoff_amount_input := FieldInput{empty_text: "Amount (blank = full balance)"}
                                writeoff_reason_input := FieldInput{empty_text: "Reason (required)"}
                                credit_writeoff := NavBtn{text: "Write off / nil balance"}
                                WrapLabel{body: "Pay what we owe them (AP)" font_color: #x260B08 font_size: 15.}
                                payable_amount_input := FieldInput{empty_text: "Amount we pay"}
                                ActionStack{
                                    payable_cash := NavBtn{text: "Pay AP cash"}
                                    payable_upi := NavBtn{text: "Pay AP UPI"}
                                    payable_card := NavBtn{text: "Pay AP card"}
                                }
                                payable_open_bills := WrapLabel{
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                payable_bill_input := FieldInput{empty_text: "Against bill # above (optional)"}
                                payable_writeoff := NavBtn{text: "Write off AP"}
                                WrapLabel{body: "Notes & attachments (selected party)" font_color: #x260B08 font_size: 15.}
                                party_attach_note := FieldInput{empty_text: "Note / GSTIN / agreement reminder"}
                                ActionStack{
                                    party_attach_add_note := RowPrimaryBtn{text: "Add note"}
                                    party_attach_photo := RowSecondaryBtn{text: "Attach photo"}
                                    party_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                }
                                party_attach_list := WrapLabel{
                                    body: "(select a party)"
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
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
                                    team_title := WrapLabel{body: "Team" font_color: #x260B08 font_size: 20.}
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Owner creates each person with a name, role title, permission preset, and starting PIN. Owner cannot change anyone's PIN later — only that person can update their own PIN."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                staff_label := WrapLabel{body: "" font_color: #x260B08}
                                staff_name_input := FieldInput{empty_text: "Staff name / id"}
                                staff_title_input := FieldInput{empty_text: "Role title (e.g. Barista)"}
                                staff_pin_input := FieldInput{empty_text: "Starting PIN (4–12 digits)"}
                                WrapLabel{
                                    body: "Permission preset (what they can access)"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Cashier: sell. Manager: sell + items + void/amend."
                                    font_color: #x6B4A3F
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
                                    }
                                }
                                staff_add_cashier := NavBtn{text: "Add staff"}
                                staff_set_login := NavBtn{text: "Set login on selected"}
                                staff_deactivate := NavBtn{text: "Remove selected"}
                                WrapLabel{
                                    body: "Optional login (id + password). Owner sets once; only the user can change password later."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                staff_login_input := FieldInput{empty_text: "Login id (optional)"}
                                staff_password_input := FieldInput{empty_text: "Starting password (optional)"}
                                WrapLabel{
                                    body: "Feature access for selected staff (tap after adding)"
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                ChipStrip{
                                    ChipStripRow{
                                        feat_sell := ChipBtn{text: "Sell"}
                                        feat_items := ChipBtn{text: "Items"}
                                        feat_sales := ChipBtn{text: "Sales"}
                                        feat_credit := ChipBtn{text: "Credit"}
                                        feat_team := ChipBtn{text: "Team"}
                                        feat_backup := ChipBtn{text: "Backup"}
                                        feat_inventory := ChipBtn{text: "Inventory"}
                                        feat_branches := ChipBtn{text: "Branches"}
                                    }
                                }
                                feat_save := NavBtn{text: "Save features for selected staff"}
                                WrapLabel{
                                    body: "Change my PIN (only yours)"
                                    font_color: #x260B08
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
                                    settings_title := WrapLabel{body: "Language & currency" font_color: #x260B08 font_size: 20.}
                                }
                                locale_label := WrapLabel{body: "" font_color: #x260B08}
                                WrapLabel{body: "Country & tax" font_color: #x260B08 font_size: 20.}
                                WrapLabel{
                                    width: Fill
                                    body: "A country pack sets the tax name and maths, the instant-pay rail, and the business id printed on bills. Kumkaj does not file taxes or issue government e-invoices anywhere."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                fiscal_label := WrapLabel{body: "" font_color: #x260B08 font_size: 13.}
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
                                    font_color: #x6B4A3F
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
                                WrapLabel{body: "Language" font_color: #x260B08 font_size: 15.}
                                WrapLabel{
                                    width: Fill
                                    body: "Follow this device’s language, or pick one yourself."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                ActionStack{
                                    View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        settings_lang_platform := StackSecondaryBtn{text: "Use platform native language"}
                                        settings_lang_platform_on := StackPrimaryBtn{text: "Use platform native language" visible: false}
                                    }
                                    View{
                                        width: Fill height: Fit
                                        flow: Overlay
                                        settings_lang_user := StackSecondaryBtn{text: "Use user selected language"}
                                        settings_lang_user_on := StackPrimaryBtn{text: "Use user selected language" visible: false}
                                    }
                                }
                                settings_lang_selected := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                language_list := mod.widgets.LanguageList{}
                                WrapLabel{body: "Currency" font_color: #x260B08 font_size: 15.}
                                WrapLabel{
                                    width: Fill
                                    body: "Scroll and tap Use for any supported currency."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                currency_list := mod.widgets.CurrencyList{}
                                WrapLabel{body: "Extra tills" font_color: #x260B08 font_size: 20.}
                                server_pair_label := WrapLabel{
                                    body: " "
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                ActionStack{
                                    allow_extra_tills := RowPrimaryBtn{text: "Allow extra tills"}
                                    stop_extra_tills := RowSecondaryBtn{text: "Stop extra tills"}
                                }
                                settings_open_branches := NavBtn{text: "Branches & org"}
                                WrapLabel{body: "Product" font_color: #x260B08 font_size: 20.}
                                settings_tmpl_label := WrapLabel{
                                    body: "Khaynjo"
                                    font_color: #x6B4A3F
                                    font_size: 13.
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "This app is Khaynjo only — restaurant operating system. Other trade faces ship as separate Kumkaj products."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                WrapLabel{body: "Your brand (in-app)" font_color: #x260B08 font_size: 20.}
                                WrapLabel{
                                    width: Fill
                                    body: "Shown on screens inside the app. The installed app name and icon on this device stay as the platform listing."
                                    font_color: #x260B08
                                    font_size: 12.
                                }
                                venue_title_input := FieldInput{empty_text: "Name of this place"}
                                venue_brand_hint := WrapLabel{
                                    body: "Icon: Kumkaj mark (default) — choose a custom one"
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                ActionStack{
                                    venue_icon_pick := RowPrimaryBtn{text: "Choose icon"}
                                    venue_icon_clear := RowSecondaryBtn{text: "Clear icon"}
                                }
                                venue_brand_save := NavBtn{text: "Save brand"}
                                WrapLabel{body: "Units" font_color: #x260B08 font_size: 20.}
                                WrapLabel{
                                    width: Fill
                                    body: "Codes for item labels (kg, piece, bag). Every unit must link — directly or through other units — to a standard (kg · cm · ml · pc), e.g. 1 crate = 2 box and 1 box = 20 cm. Conversions: 1 from = factor to. Currencies are never units."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                units_label := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                unit_code_input := FieldInput{empty_text: "Code (bag)"}
                                unit_name_input := FieldInput{empty_text: "Name (Bag)"}
                                unit_add := NavBtn{text: "Add unit"}
                                conv_from_input := FieldInput{empty_text: "From code (bag)"}
                                conv_to_input := FieldInput{empty_text: "To code (kg)"}
                                conv_factor_input := FieldInput{empty_text: "Factor (25 = 1 bag → 25 kg)"}
                                conv_add := NavBtn{text: "Add conversion"}
                                WrapLabel{body: "Kitchen printing" font_color: #x260B08 font_size: 20.}
                                WrapLabel{
                                    width: Fill
                                    body: "When a KOT is sent — at the counter or from a waiter's device — the kitchen's paper prints here by itself."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                kot_print_toggle := NavBtn{text: "Kitchen printing: On"}
                                WrapLabel{body: "Backup" font_color: #x260B08 font_size: 20.}
                                db_path_label := WrapLabel{body: "" font_color: #x260B08}
                                audit_label := WrapLabel{body: "" font_color: #x260B08}
                                backup_btn := NavBtn{text: "Create backup"}
                                backup_path_label := WrapLabel{body: "" font_color: #x260B08}
                                WrapLabel{
                                    width: Fill
                                    body: "Restore: put restore.sqlite3 in the backups folder, then tap Restore."
                                    font_color: #x260B08
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
                                    day_close_title := WrapLabel{body: "Day close" font_color: #x260B08 font_size: 20.}
                                }
                                day_close_label := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 14.
                                }
                                WrapLabel{body: "Drawer count (optional)" font_color: #x260B08 font_size: 15.}
                                day_close_counted_input := FieldInput{empty_text: "Counted cash in drawer"}
                                day_close_copy := NavBtn{text: "Copy day close"}
                                day_close_commit := NavBtn{text: "Record day close"}
                                WrapLabel{
                                    width: Fill
                                    body: "Recording a day close saves this summary (and drawer variance if counted) to the audit log. It does not stop selling."
                                    font_color: #x6B4A3F
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
                                    reports_title := WrapLabel{body: "Reports" font_color: #x260B08 font_size: 20.}
                                }
                                reports_help := WrapLabel{
                                    width: Fill
                                    body: "Day books, party aging, bank posts, and period P&L — same books for every entity."
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                reports_books_btns := ActionStack{
                                    reports_today := RowSecondaryBtn{text: "Today books"}
                                    reports_today_on := RowPrimaryBtn{text: "Today books" visible: false}
                                    reports_month := RowSecondaryBtn{text: "This month"}
                                    reports_month_on := RowPrimaryBtn{text: "This month" visible: false}
                                    reports_aging := RowSecondaryBtn{text: "Outstanding / Dr-Cr"}
                                    reports_aging_on := RowPrimaryBtn{text: "Outstanding / Dr-Cr" visible: false}
                                    reports_balance_sheet := RowSecondaryBtn{text: "Balance sheet"}
                                    reports_balance_sheet_on := RowPrimaryBtn{text: "Balance sheet" visible: false}
                                }
                                reports_label := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                reports_expense_heading := WrapLabel{body: "Log expense" font_color: #x260B08 font_size: 15.}
                                expense_amount_input := FieldInput{empty_text: "Amount"}
                                expense_note_input := FieldInput{empty_text: "Note (fuel, office, …)"}
                                exp_t1_row := View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    Label{text: "Paid by" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 11.}
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t1_cash := ChipBtn{text: "Cash"}
                                        exp_t1_cash_on := ChipBtnActive{text: "Cash" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t1_bank := ChipBtn{text: "Bank"}
                                        exp_t1_bank_on := ChipBtnActive{text: "Bank" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t1_upi := ChipBtn{text: "UPI"}
                                        exp_t1_upi_on := ChipBtnActive{text: "UPI" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t1_card := ChipBtn{text: "Card"}
                                        exp_t1_card_on := ChipBtnActive{text: "Card" visible: false}
                                    }
                                }
                                exp_part2 := FieldInput{empty_text: "Part by a second way (optional)"}
                                exp_t2_row := View{
                                    width: Fill height: Fit
                                    flow: Right spacing: 6
                                    align: Align{y: 0.5}
                                    Label{text: "Second way" draw_text.color: #x6B4A3F draw_text.text_style.font_size: 11.}
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t2_cash := ChipBtn{text: "Cash"}
                                        exp_t2_cash_on := ChipBtnActive{text: "Cash" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t2_bank := ChipBtn{text: "Bank"}
                                        exp_t2_bank_on := ChipBtnActive{text: "Bank" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t2_upi := ChipBtn{text: "UPI"}
                                        exp_t2_upi_on := ChipBtnActive{text: "UPI" visible: false}
                                    }
                                    View{width: Fit height: 40 flow: Overlay
                                        exp_t2_card := ChipBtn{text: "Card"}
                                        exp_t2_card_on := ChipBtnActive{text: "Card" visible: false}
                                    }
                                }
                                expense_add := NavBtn{text: "Add expense"}
                                WrapLabel{body: "Attachments (last expense)" font_color: #x260B08 font_size: 15.}
                                expense_attach_note := FieldInput{empty_text: "Extra note on last expense"}
                                ActionStack{
                                    expense_attach_add_note := RowPrimaryBtn{text: "Add note"}
                                    expense_attach_photo := RowSecondaryBtn{text: "Attach bill photo"}
                                    expense_attach_pdf := RowSecondaryBtn{text: "Attach PDF"}
                                }
                                expense_attach_list := WrapLabel{
                                    body: "(log an expense first)"
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                reports_bank_heading := WrapLabel{body: "Bank / cashbook" font_color: #x260B08 font_size: 15.}
                                bank_amount_input := FieldInput{empty_text: "Amount"}
                                bank_note_input := FieldInput{empty_text: "Note (deposit, withdrawal, …)"}
                                ActionStack{
                                    bank_till_to_bank := RowPrimaryBtn{text: "Till → bank"}
                                    bank_bank_to_till := RowSecondaryBtn{text: "Bank → till"}
                                    bank_in := RowSecondaryBtn{text: "Bank in"}
                                    bank_out := RowSecondaryBtn{text: "Bank out"}
                                }
                                reports_copy := NavBtn{text: "Copy report"}
                                WrapLabel{body: "Bank reconciliation (tick off against your bank statement)" font_color: #x260B08 font_size: 15.}
                                bank_recon_label := WrapLabel{
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                bank_uncleared_label := WrapLabel{
                                    body: ""
                                    font_color: #x6B4A3F
                                    font_size: 12.
                                }
                                bank_clear_input := FieldInput{empty_text: "Uncleared # above"}
                                ActionStack{
                                    bank_mark_cleared := RowPrimaryBtn{text: "Mark cleared"}
                                    bank_mark_uncleared := RowSecondaryBtn{text: "Unmark"}
                                }
                                WrapLabel{body: "Spreadsheet (CSV)" font_color: #x260B08 font_size: 15.}
                                ActionStack{
                                    csv_copy_day := RowPrimaryBtn{text: "Copy day book CSV"}
                                    csv_copy_outstanding := RowSecondaryBtn{text: "Copy outstanding CSV"}
                                }
                                ActionStack{
                                    csv_copy_parties := RowPrimaryBtn{text: "Copy parties CSV"}
                                    csv_copy_items := RowSecondaryBtn{text: "Copy items CSV"}
                                }
                                ActionStack{
                                    csv_copy_ledger := RowPrimaryBtn{text: "Copy party ledger CSV"}
                                    csv_import_parties := RowSecondaryBtn{text: "Import parties CSV…"}
                                }
                                csv_import_items := NavBtn{text: "Import items CSV…"}
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
                                    inventory_title := WrapLabel{body: "Inventory" font_color: #x260B08 font_size: 20.}
                                }
                                inventory_label := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
                                    font_size: 13.
                                }
                                WrapLabel{
                                    width: Fill
                                    body: "Type the item name (part of it is enough), a quantity, then adjust. Items that are not counted yet need Start tracking once."
                                    font_color: #x6B4A3F
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
                                WrapLabel{body: "Goods received" font_color: #x260B08 font_size: 15.}
                                inv_cost_input := FieldInput{empty_text: "Unit cost"}
                                inv_supplier_input := FieldInput{empty_text: "Supplier"}
                                inv_purchase := NavBtn{text: "Record purchase"}
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
                                    branches_title := WrapLabel{body: "Branches" font_color: #x260B08 font_size: 20.}
                                }
                                org_name_input := FieldInput{empty_text: "Business name"}
                                org_save := NavBtn{text: "Save business name"}
                                branches_label := WrapLabel{
                                    body: ""
                                    font_color: #x260B08
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
                                    font_color: #x6B4A3F
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
                        draw_bg.color: #x9A0E10
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
                                color: #xF7B043
                                color_hover: #xffffff
                                color_down: #xEFD2BC
                                color_focus: #xF7B043
                                color_2: #xF7B043
                                color_2_hover: #xffffff
                                color_2_down: #xEFD2BC
                                color_2_focus: #xF7B043
                                border_radius: 10.
                                border_size: 0.
                            }
                            draw_text +: {
                                color: #x69090C
                                color_hover: #x69090C
                                color_down: #x69090C
                                color_focus: #x69090C
                                text_style +: {font_size: 13.}
                            }
                        }
                        undo_snack_dismiss := Button{
                            text: "✕"
                            width: 36 height: 36
                            draw_bg +: {
                                color: #xB43A12
                                color_hover: #xB43A12
                                color_down: #x69090C
                                color_focus: #xB43A12
                                color_2: #xB43A12
                                color_2_hover: #xB43A12
                                color_2_down: #x69090C
                                color_2_focus: #xB43A12
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

                    // Phone chrome: all peers in one scroll row (Books · Order · Buy …).
                    // Selected peer = *Active face visible (paint_nav_tabs), not focus.
                    bottom_nav := SolidView{
                        width: Fill height: Fit
                        padding: Inset{left: 4. right: 4. top: 2. bottom: 6.}
                        draw_bg.color: #x69090C
                        bottom_nav_scroll := ScrollXView{
                            width: Fill height: Fit
                            // Whitespace lanes around the chips (top,
                            // bottom, ends, gaps): a press there belongs
                            // to the scroll view, a press on a chip is a
                            // tap or a drag — see pointer_nav_drag.
                            View{
                                width: Fit height: Fit
                                padding: Inset{left: 12. right: 12. top: 12. bottom: 12.}
                                flow: Right spacing: 6
                                nav_books_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_books := NavChip{text: "Books"}
                                    nav_books_on := NavChipActive{text: "Books" visible: false}
                                }
                                nav_sell_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_sell := NavChip{text: "Order"}
                                    nav_sell_on := NavChipActive{text: "Order" visible: false}
                                }
                                nav_buy_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_buy := NavChip{text: "Buy"}
                                    nav_buy_on := NavChipActive{text: "Buy" visible: false}
                                }
                                nav_menu_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_menu := NavChip{text: "Menu"}
                                    nav_menu_on := NavChipActive{text: "Menu" visible: false}
                                }
                                nav_floor_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_floor := NavChip{text: "Floor"}
                                    nav_floor_on := NavChipActive{text: "Floor" visible: false}
                                }
                                nav_kitchen_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_kitchen := NavChip{text: "Kitchen"}
                                    nav_kitchen_on := NavChipActive{text: "Kitchen" visible: false}
                                }
                                nav_sales_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_sales := NavChip{text: "Sales"}
                                    nav_sales_on := NavChipActive{text: "Sales" visible: false}
                                }
                                nav_day_close_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_day_close := NavChip{text: "Close"}
                                    nav_day_close_on := NavChipActive{text: "Close" visible: false}
                                }
                                nav_reports_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_reports := NavChip{text: "Reports"}
                                    nav_reports_on := NavChipActive{text: "Reports" visible: false}
                                }
                                nav_inventory_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_inventory := NavChip{text: "Stock"}
                                    nav_inventory_on := NavChipActive{text: "Stock" visible: false}
                                }
                                nav_branches_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_branches := NavChip{text: "Branches"}
                                    nav_branches_on := NavChipActive{text: "Branches" visible: false}
                                }
                                nav_credit_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_credit := NavChip{text: "Parties"}
                                    nav_credit_on := NavChipActive{text: "Parties" visible: false}
                                }
                                nav_team_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_team := NavChip{text: "Team"}
                                    nav_team_on := NavChipActive{text: "Team" visible: false}
                                }
                                nav_settings_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_settings := NavChip{text: "Settings"}
                                    nav_settings_on := NavChipActive{text: "Settings" visible: false}
                                }
                                nav_lock_slot := View{
                                    width: Fit{min: FitBound.Abs(72.)} height: Fit
                                    flow: Overlay
                                    nav_lock := NavChip{text: "Lock"}
                                    nav_lock_on := NavChipActive{text: "Lock" visible: false}
                                }
                            }
                        }
                    }
                    } // main_column
                    } // app_shell
                    } // app_hscroll

                    // Floating chip that follows the pointer while a nav item
                    // is dragged to another slot (see pointer_nav_drag). It
                    // lives in a Modal because a Modal paints on the overlay
                    // draw list — above every scroll view. A plain sibling
                    // was covered by the page scroll view's own pass (the
                    // ghost showed over the rail but not over the page, and
                    // not in bottom-bar mode at all). The modal's dim
                    // backdrop is overridden transparent and its content
                    // fills the window, so the ghost's abs_pos is in window
                    // coordinates and nothing else is visible.
                    nav_drag_modal := Modal{
                        can_dismiss: false
                        bg_view +: {
                            draw_bg +: {color: #x00000000}
                        }
                        content +: {
                            width: Fill height: Fill
                            flow: Overlay
                            nav_drag_ghost := RoundedView{
                                visible: false
                                width: Fit height: Fit
                                padding: Inset{left: 16. right: 16. top: 10. bottom: 10.}
                                draw_bg.color: #x9A0E10E6
                                draw_bg.border_radius: 12.
                                draw_bg.border_color: #xF2B183
                                draw_bg.border_size: 1.
                                nav_drag_ghost_label := Label{
                                    width: Fit height: Fit
                                    text: ""
                                    draw_text.color: #xffffff
                                    draw_text.text_style.font_size: 13.
                                }
                            }
                        }
                    }
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
fn set_view_walk(cx: &mut Cx, view: &ViewRef, walk: Walk) {
    if let Some(mut inner) = view.borrow_mut() {
        inner.walk = walk;
    }
    view.redraw(cx);
}

fn set_view_flow(cx: &mut Cx, view: &ViewRef, flow: Flow) {
    if let Some(mut inner) = view.borrow_mut() {
        inner.layout.flow = flow;
    }
    view.redraw(cx);
}

fn size_fill(weight: f64, min: f64) -> Size {
    Size::Fill {
        weight,
        min: Some(min),
        max: None,
    }
}

fn default_brand_mark_path() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        for candidate in [
            parent.join("kumkaj_khaynjo/resources/kumkaj-mark.webp"),
            parent.join("kumkaj-khaynjo/resources/kumkaj-mark.webp"),
            parent.join("resources/kumkaj-mark.webp"),
        ] {
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/kumkaj-mark.webp")
}

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
    } else {
        false
    };
    if !queued {
        requested.lock().unwrap().remove(&uid);
    }
    queued
}

/// Brand marks load inline on every sync. Two traps used to pin the
/// placeholder forever: the widgets' DSL `src` queued an async load of the
/// bundled mark on first draw whose completion overwrote the shop logo,
/// and the sync loader never marks the widget dirty, so a swapped texture
/// was not repainted until an unrelated full redraw. The brand widgets
/// therefore carry no DSL `src`, and every load here ends with a redraw.
fn load_brand_mark_at(cx: &mut Cx, image: &ImageRef, path: &std::path::Path) -> bool {
    let loaded = path.is_file() && image.load_image_file_by_path(cx, path).is_ok();
    if loaded {
        image.redraw(cx);
    }
    loaded
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

fn floor_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(90.0))
}

fn kot_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(150.0))
}

fn catalogue_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(72.0))
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

/// Resolve on-disk path for a product/draft image key (custom photo or empty).
fn image_key_path(key: Option<&str>, dir: &std::path::Path) -> std::path::PathBuf {
    match key {
        Some(k) if k.starts_with("pending_custom:") => {
            std::path::PathBuf::from(&k["pending_custom:".len()..])
        }
        Some(k) if k.starts_with("custom:") => dir.join(&k["custom:".len()..]),
        other => menu_images::resolve_image_path(other, "", dir),
    }
}

/// Bind `ThumbAvatar` (`thumb` Image + `thumb_initials` Label).
/// Photo when the file exists; otherwise name initials (1 word → 2 letters,
/// 2+ words → first letter of each of the first two).
fn apply_thumb_avatar(cx: &mut Cx, root: &WidgetRef, name: &str, path: &std::path::Path) {
    let img = root.image(cx, ids!(thumb));
    let initials_lbl = root.label(cx, ids!(thumb_initials));
    let has_file = path.is_file();
    let queued = has_file && load_thumb_at(cx, &img, path);
    if has_file {
        img.set_visible(cx, true);
        initials_lbl.set_visible(cx, false);
        initials_lbl.set_text(cx, "");
    } else {
        img.set_visible(cx, false);
        let initials = menu_images::name_initials(name);
        initials_lbl.set_text(cx, &initials);
        initials_lbl.set_visible(cx, !initials.is_empty());
    }
    if queued {
        root.as_view().redraw_texture_cache();
    }
}

fn load_product_thumb(cx: &mut Cx, item: &WidgetRef, product: &Product) {
    let dir = item_images_dir();
    let path = image_key_path(product.image_key.as_deref(), &dir);
    apply_thumb_avatar(cx, item, &product.name, &path);
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

/// Floor: one card per dining table. Cream face, dark ink (contrast law);
/// occupied tables say so in deep-red text and the button opens their bill.
#[derive(Script, ScriptHook, Widget)]
struct FloorList {
    #[deref]
    view: View,
}

impl Widget for FloorList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // (name, status_line, open_label, show_request_bill)
        let tables: Vec<(String, String, String, bool)> = {
            let st = app_state().lock().unwrap();
            let now = chrono::Local::now();
            st.tables
                .iter()
                .map(|t| {
                    let mins = chrono::DateTime::parse_from_rfc3339(&t.occupied_at)
                        .map(|at| {
                            now.signed_duration_since(at.with_timezone(&chrono::Local))
                                .num_minutes()
                                .max(0)
                        })
                        .unwrap_or(0);
                    let amount = if t.running_total_paise > 0 {
                        format!(" · {}", st.money(Money(t.running_total_paise)))
                    } else {
                        String::new()
                    };
                    match t.status {
                        TableStatus::Available => (
                            t.name.clone(),
                            "Free — tap to start an order".to_string(),
                            "Start order".to_string(),
                            false,
                        ),
                        TableStatus::Occupied => (
                            t.name.clone(),
                            format!("Running · {mins} min{amount}"),
                            "Open bill".to_string(),
                            true,
                        ),
                        TableStatus::PaymentPending => (
                            t.name.clone(),
                            format!(
                                "Payment pending · {mins} min{amount} — collect at counter"
                            ),
                            "Collect payment".to_string(),
                            false,
                        ),
                    }
                })
                .collect()
        };
        let count = tables.len();
        let row = *floor_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(table) = tables.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.label(cx, ids!(name)).set_text(cx, &table.0);
                    item.label(cx, ids!(status))
                        .set_text(cx, &plain_display_text(&table.1));
                    item.button(cx, ids!(open)).set_text(cx, &table.2);
                    item.widget(cx, ids!(bill_req)).set_visible(cx, table.3);
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
        remember_content_row(cx, &mut self.view, floor_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Provided-items shelf: category folders to browse (Show loads its dishes).
#[derive(Script, ScriptHook, Widget)]
struct CatalogueCatList {
    #[deref]
    view: View,
}

impl Widget for CatalogueCatList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (cats, selected) = {
            let st = app_state().lock().unwrap();
            (st.catalogue_cats.clone(), st.catalogue_cat.clone())
        };
        let count = cats.len();
        let row = *catalogue_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(cat) = cats.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let mark = if cat.key == selected { "✓ " } else { "" };
                    item.widget(cx, ids!(label))
                        .set_text(cx, &plain_display_text(&format!("{mark}{}", cat.label)));
                    item.button(cx, ids!(pick)).set_text(cx, "Show");
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
        remember_content_row(cx, &mut self.view, catalogue_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Dishes in the browsed shelf category. Use fills the item form with
/// the dish's name + photo (fresh UUID on Add; folder never becomes
/// the item's category unless the owner files it there).
#[derive(Script, ScriptHook, Widget)]
struct CatalogueItemList {
    #[deref]
    view: View,
}

impl Widget for CatalogueItemList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let items = {
            let st = app_state().lock().unwrap();
            st.catalogue_items.clone()
        };
        let count = items.len();
        let row = *catalogue_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(dish) = items.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    apply_thumb_avatar(cx, &item, &dish.name, &dish.path);
                    item.widget(cx, ids!(label))
                        .set_text(cx, &plain_display_text(&dish.name));
                    item.widget(cx, ids!(photo)).set_visible(cx, false);
                    item.button(cx, ids!(pick)).set_text(cx, "Use");
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
        remember_content_row(cx, &mut self.view, catalogue_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// The shop's own categories: avatar is the category's photo, else its
/// name initials — same placeholder rule as items.
#[derive(Script, ScriptHook, Widget)]
struct CategoryList {
    #[deref]
    view: View,
}

impl Widget for CategoryList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let cats = {
            let st = app_state().lock().unwrap();
            st.categories.clone()
        };
        let count = cats.len();
        let row = *catalogue_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(cat) = cats.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let dir = item_images_dir();
                    let path = if cat.image_key.is_empty() {
                        std::path::PathBuf::new()
                    } else {
                        image_key_path(Some(&cat.image_key), &dir)
                    };
                    apply_thumb_avatar(cx, &item, &cat.name, &path);
                    item.widget(cx, ids!(label))
                        .set_text(cx, &plain_display_text(&cat.name));
                    item.button(cx, ids!(pick)).set_text(cx, "Use");
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
        remember_content_row(cx, &mut self.view, catalogue_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Kitchen display: open KOTs in arrival order, items + state, never prices.
/// Advance walks New → Preparing → Ready → Done (ticket leaves the list).
#[derive(Script, ScriptHook, Widget)]
struct KitchenList {
    #[deref]
    view: View,
}

impl Widget for KitchenList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let kots = {
            let st = app_state().lock().unwrap();
            st.open_kots.clone()
        };
        let count = kots.len();
        let row = *kot_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(kot) = kots.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let station = if kot.station_name.is_empty() {
                        "Kitchen".to_string()
                    } else {
                        kot.station_name.clone()
                    };
                    let time = kot
                        .created_at
                        .split('T')
                        .nth(1)
                        .and_then(|t| t.split('.').next())
                        .unwrap_or(kot.created_at.as_str());
                    item.widget(cx, ids!(title))
                        .set_text(cx, &plain_display_text(&format!("{station} · {time}")));
                    // Summary is capped at draw: PortalList rows share one
                    // measured height, so an unbounded multi-line summary
                    // would make the measured row oscillate between draws.
                    // Full text stays on the store record (and prints).
                    let summary: String = kot.summary.chars().take(140).collect();
                    let summary = if summary.len() < kot.summary.len() {
                        format!("{summary}…")
                    } else {
                        summary
                    };
                    item.widget(cx, ids!(summary))
                        .set_text(cx, &plain_display_text(&summary));
                    let (state, next) = match kot.status {
                        KotStatus::New => ("NEW — waiting", "Start"),
                        KotStatus::Preparing => ("PREPARING", "Mark ready"),
                        KotStatus::Ready => ("READY — serve it", "Done"),
                        KotStatus::Completed => ("DONE", "Done"),
                        KotStatus::Cancelled => ("CANCELLED", "Done"),
                    };
                    item.label(cx, ids!(state)).set_text(cx, state);
                    item.button(cx, ids!(advance)).set_text(cx, next);
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
        remember_content_row(cx, &mut self.view, kot_row_height(), row, measured_row);
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
        let (customers, currency, fiscal, selected_id) = {
            let st = app_state().lock().unwrap();
            (
                st.customers.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.selected_customer_id.clone(),
            )
        };
        // Same capped-list shape as ServerList / LanguageList / CurrencyList:
        // room for up to 3 full rows, list scrolls inside (nested-scroll claim
        // in handle_event). CustomerRow ≈ 68px (12+44+12) + 8px item pad.
        const ROW: f64 = 76.0;
        const GAP: f64 = 6.0;
        let n = customers.len().max(1);
        let want = list_content_height(n.min(3), ROW, GAP).max(ROW + 16.0);
        let walk = Walk {
            height: Size::Fixed(want.min(240.0)),
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
                    let is_selected = selected_id.as_deref() == Some(c.id.as_str());
                    let mark = if is_selected { "✓ " } else { "" };
                    let text = if c.balance_paise > 0 {
                        format!(
                            "{}{} · owes {}",
                            mark,
                            c.name,
                            Money(c.balance_paise).format_localized(&currency, &fiscal)
                        )
                    } else if c.balance_paise < 0 {
                        format!(
                            "{}{} · advance {}",
                            mark,
                            c.name,
                            Money(-c.balance_paise).format_localized(&currency, &fiscal)
                        )
                    } else {
                        format!("{}{}", mark, c.name)
                    };
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&text));
                    let sub = if c.kind_label.trim().is_empty() {
                        format!("ID: {}", c.id)
                    } else {
                        format!("ID: {} · {}", c.id, c.kind_label.trim())
                    };
                    item.widget(cx, ids!(sublabel))
                        .set_text(cx, &plain_display_text(&sub));
                    item.button(cx, ids!(pick))
                        .set_text(cx, if is_selected { "Selected" } else { "Pick" });
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Capped list (3 rows): pointer over it = the list owns the wheel/drag
        // and scrolls internally; the page must not steal the gesture.
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
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
        // LocaleRow ≈ 60px (8+44+8) + item pad/spacing. Old 56px viewport
        // clipped the Open button so no full row was visible.
        const ROW: f64 = 72.0;
        const GAP: f64 = 10.0;
        let n = servers.len().max(1);
        // Always room for at least one full row; prefer two when possible.
        let want = list_content_height(n.min(3).max(1), ROW, GAP).max(ROW + 16.0);
        let walk = Walk {
            height: Size::Fixed(want.min(280.0)),
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
        // Keep the showing tab's figures fresh — data is always live, so
        // these screens need no Refresh button.
        {
            let mut st = app_state().lock().unwrap();
            if st.session.is_some() {
                match st.screen {
                    Screen::Books => {
                        let _ = st.refresh_books_overview();
                    }
                    Screen::Inventory => {
                        let _ = st.refresh_inventory_text();
                    }
                    Screen::Reports => {
                        let _ = st.refresh_reports_facet();
                    }
                    _ => {}
                }
            }
        }
        let (
            today,
            session_line,
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
            // Header second line: organisation (when the shop set one),
            // then who is signed in — "Org · Owner (Owner)".
            let org_line = st
                .store
                .as_ref()
                .and_then(|s| s.org_name().ok())
                .unwrap_or_default();
            let session_line = if org_line.trim().is_empty() {
                session.clone()
            } else {
                format!("{} · {}", org_line.trim(), session)
            };
            let table_chip = match st
                .selected_table_id
                .as_ref()
                .and_then(|id| st.tables.iter().find(|t| &t.id == id))
            {
                Some(t) => format!("{} · Dine-in", t.name),
                None => match st.cart.order_type {
                    OrderType::DineIn => "Dine-in — pick a table on Floor".to_string(),
                    other => format!("{} order", other.label()),
                },
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
                "No items yet. Open Menu to add your dishes and items.".to_string()
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
                session_line,
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
            .widget(cx, ids!(session_label)).set_text(cx, &plain_display_text(&session_line));
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
            .widget(cx, ids!(order_table_label)).set_text(cx, &plain_display_text(&table_chip));
        self.ui
            .widget(cx, ids!(session_chip)).set_text(cx, &plain_display_text(&session_chip));
        // Order desk: order type chips + expense tender chips.
        {
            let st = app_state().lock().unwrap();
            let cart_active = !st.cart.is_empty();
            self.ui
                .button(cx, ids!(next_order_btn))
                .set_visible(cx, cart_active);
            self.ui
                .widget(cx, ids!(kitchen_empty))
                .set_visible(cx, st.open_kots.is_empty());
            let order_type = st.cart.order_type;
            for (idle, on, kind) in [
                (ids!(otype_counter), ids!(otype_counter_on), OrderType::Counter),
                (ids!(otype_dinein), ids!(otype_dinein_on), OrderType::DineIn),
                (ids!(otype_takeaway), ids!(otype_takeaway_on), OrderType::Takeaway),
                (ids!(otype_delivery), ids!(otype_delivery_on), OrderType::Delivery),
            ] {
                self.paint_choice_pair(cx, idle, on, order_type == kind);
            }
            // Expense tender chips (both forms share the choice).
            let (exp_t1, exp_t2) = (st.expense_tender, st.expense_tender2);
            for (idle, on, m) in [
                (ids!(exp_t1_cash), ids!(exp_t1_cash_on), PayMethod::Cash),
                (ids!(exp_t1_bank), ids!(exp_t1_bank_on), PayMethod::Bank),
                (ids!(exp_t1_upi), ids!(exp_t1_upi_on), PayMethod::Upi),
                (ids!(exp_t1_card), ids!(exp_t1_card_on), PayMethod::Card),
                (ids!(buy_exp_t1_cash), ids!(buy_exp_t1_cash_on), PayMethod::Cash),
                (ids!(buy_exp_t1_bank), ids!(buy_exp_t1_bank_on), PayMethod::Bank),
                (ids!(buy_exp_t1_upi), ids!(buy_exp_t1_upi_on), PayMethod::Upi),
                (ids!(buy_exp_t1_card), ids!(buy_exp_t1_card_on), PayMethod::Card),
            ] {
                self.paint_choice_pair(cx, idle, on, exp_t1 == m);
            }
            for (idle, on, m) in [
                (ids!(exp_t2_cash), ids!(exp_t2_cash_on), PayMethod::Cash),
                (ids!(exp_t2_bank), ids!(exp_t2_bank_on), PayMethod::Bank),
                (ids!(exp_t2_upi), ids!(exp_t2_upi_on), PayMethod::Upi),
                (ids!(exp_t2_card), ids!(exp_t2_card_on), PayMethod::Card),
                (ids!(buy_exp_t2_cash), ids!(buy_exp_t2_cash_on), PayMethod::Cash),
                (ids!(buy_exp_t2_bank), ids!(buy_exp_t2_bank_on), PayMethod::Bank),
                (ids!(buy_exp_t2_upi), ids!(buy_exp_t2_upi_on), PayMethod::Upi),
                (ids!(buy_exp_t2_card), ids!(buy_exp_t2_card_on), PayMethod::Card),
            ] {
                self.paint_choice_pair(cx, idle, on, exp_t2 == m);
            }
            drop(st);
        }
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
            .widget(cx, ids!(catalog_cta))
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
        {
            let (selected_label, form_btn_label) = {
                let st = app_state().lock().unwrap();
                match st
                    .selected_customer_id
                    .as_ref()
                    .and_then(|id| st.customers.iter().find(|c| &c.id == id))
                {
                    Some(c) => {
                        let kind = if c.kind_label.trim().is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", c.kind_label.trim())
                        };
                        (
                            format!(
                                "Editing: {}{} — change the form, tap Save changes. Credit limit, settle, pay and notes below also use this party.",
                                c.name, kind
                            ),
                            "Save changes",
                        )
                    }
                    None => (
                        "New party — fill the form and tap Add party, or Pick a party from the list to edit it."
                            .to_string(),
                        "Add party",
                    ),
                }
            };
            self.ui
                .widget(cx, ids!(party_selected_label))
                .set_text(cx, &plain_display_text(&selected_label));
            self.ui
                .button(cx, ids!(credit_add_btn))
                .set_text(cx, form_btn_label);
        }
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
        {
            let attach_txt = {
                let st = app_state().lock().unwrap();
                match (&st.selected_sale_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => store
                        .attachments_summary_text(AttachmentEntity::Sale, id)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    _ => String::new(),
                }
            };
            let body = if attach_txt.is_empty() {
                "(no attachments yet — select a sale)".into()
            } else {
                attach_txt
            };
            self.ui
                .widget(cx, ids!(sale_attach_list))
                .set_text(cx, &plain_display_text(&body));
        }
        {
            let attach_txt = {
                let st = app_state().lock().unwrap();
                match (&st.selected_customer_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => store
                        .attachments_summary_text(AttachmentEntity::Party, id)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    _ => String::new(),
                }
            };
            let body = if attach_txt.is_empty() {
                "(no attachments yet — select a party)".into()
            } else {
                attach_txt
            };
            self.ui
                .widget(cx, ids!(party_attach_list))
                .set_text(cx, &plain_display_text(&body));
        }
        {
            // Bill-wise lists for the selected party (numbers match the
            // "Against bill #" inputs).
            let (ar_txt, ap_txt) = {
                let st = app_state().lock().unwrap();
                let fmt = |bills: Vec<domain::OpenBill>, st: &AppState| -> String {
                    if bills.is_empty() {
                        return String::new();
                    }
                    let parts: Vec<String> = bills
                        .iter()
                        .enumerate()
                        .map(|(i, b)| {
                            format!(
                                "{}) {} · {} open (of {})",
                                i + 1,
                                &b.created_at[..b.created_at.len().min(10)],
                                st.money(Money(b.open_paise)),
                                st.money(Money(b.billed_paise)),
                            )
                        })
                        .collect();
                    format!("Open bills: {}", parts.join(" · "))
                };
                match (&st.selected_customer_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => (
                        fmt(store.open_credit_bills(id).unwrap_or_default(), &st),
                        fmt(store.open_payable_bills(id).unwrap_or_default(), &st),
                    ),
                    _ => (String::new(), String::new()),
                }
            };
            self.ui
                .widget(cx, ids!(settle_open_bills))
                .set_text(cx, &plain_display_text(&ar_txt));
            self.ui
                .widget(cx, ids!(payable_open_bills))
                .set_text(cx, &plain_display_text(&ap_txt));
        }
        {
            // Bank reconciliation: book vs statement-cleared, plus the
            // uncleared movements numbered for the tick-off input.
            let (recon_txt, uncleared_txt) = {
                let st = app_state().lock().unwrap();
                match st.store.as_ref() {
                    Some(store) => {
                        let recon = match store.bank_reconciliation() {
                            Ok((book, cleared, n, _net)) => format!(
                                "Book balance {} · cleared (bank) {} · {} uncleared",
                                st.money(Money(book)),
                                st.money(Money(cleared)),
                                n,
                            ),
                            Err(_) => String::new(),
                        };
                        let uncleared = store
                            .list_money_movements(50)
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|m| !m.cleared)
                            .take(8)
                            .enumerate()
                            .map(|(i, m)| {
                                let kind = MoneyMoveKind::parse(&m.kind)
                                    .map(|k| k.label())
                                    .unwrap_or("Movement");
                                format!(
                                    "{}) {} {} {}",
                                    i + 1,
                                    &m.created_at[..m.created_at.len().min(10)],
                                    kind,
                                    st.money(Money(m.amount_paise)),
                                )
                            })
                            .collect::<Vec<_>>();
                        let uncleared = if uncleared.is_empty() {
                            String::new()
                        } else {
                            format!("Uncleared: {}", uncleared.join(" · "))
                        };
                        (recon, uncleared)
                    }
                    None => (String::new(), String::new()),
                }
            };
            self.ui
                .widget(cx, ids!(bank_recon_label))
                .set_text(cx, &plain_display_text(&recon_txt));
            self.ui
                .widget(cx, ids!(bank_uncleared_label))
                .set_text(cx, &plain_display_text(&uncleared_txt));
        }
        {
            let attach_txt = {
                let st = app_state().lock().unwrap();
                match (&st.selected_expense_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => store
                        .attachments_summary_text(AttachmentEntity::Expense, id)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    _ => String::new(),
                }
            };
            let body = if attach_txt.is_empty() {
                "(no attachments yet — log an expense)".into()
            } else {
                attach_txt
            };
            self.ui
                .widget(cx, ids!(expense_attach_list))
                .set_text(cx, &plain_display_text(&body));
        }
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
            let path = image_key_path(draft_key.as_deref(), &dir);
            let preview = self.ui.widget(cx, ids!(draft_preview));
            apply_thumb_avatar(cx, &preview, &name, &path);
        }

        self.sync_venue_brand(cx);

        self.show_screen(cx, screen);
        {
            let st = app_state().lock().unwrap();
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
            let day_close = st.day_close_text.clone();
            let inventory_text = st.ops_panel_text.clone();
            let branches_body = st.branches_text.clone();
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
                .widget(cx, ids!(server_pair_label)).set_text(cx, &plain_display_text(&pair_label));
            self.ui
                .widget(cx, ids!(settings_tmpl_label))
                .set_text(cx, &plain_display_text(&tmpl_line));
            {
                let st = app_state().lock().unwrap();
                let on = st
                    .store
                    .as_ref()
                    .map(|s| s.kot_auto_print())
                    .unwrap_or(true);
                self.ui.button(cx, ids!(kot_print_toggle)).set_text(
                    cx,
                    if on {
                        "Kitchen printing: On"
                    } else {
                        "Kitchen printing: Off"
                    },
                );
            }
            let units_body = {
                let st = app_state().lock().unwrap();
                match st.store.as_ref() {
                    Some(store) => {
                        let units = store.list_units().unwrap_or_default();
                        let convs = store.list_unit_conversions().unwrap_or_default();
                        let mut lines = Vec::new();
                        lines.push("Units:".into());
                        for u in &units {
                            let link = match store.resolve_unit(&u.code) {
                                Some((family, factor)) => {
                                    let anchor =
                                        domain::unit_family_anchor(&family).unwrap_or("");
                                    format!(
                                        " · 1 {} = {} {} ({})",
                                        u.code,
                                        precise::format_kg(factor),
                                        anchor,
                                        family
                                    )
                                }
                                None => {
                                    " · not linked to a standard unit yet".to_string()
                                }
                            };
                            lines.push(format!(
                                "  {} — {}{}{}",
                                u.code,
                                u.name,
                                if u.builtin { " (built-in)" } else { "" },
                                link
                            ));
                        }
                        if !convs.is_empty() {
                            lines.push("Conversions (1 from = factor to):".into());
                            for c in &convs {
                                let factor = precise::format_kg(
                                    rust_decimal::Decimal::from(c.factor_milli)
                                        / rust_decimal::Decimal::from(1000),
                                );
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
            let tmpl_hint = if tmpl.how_to.is_empty() {
                format!("Selected: {} — {}", tmpl.title, tmpl.blurb)
            } else {
                format!("{}\n{}\n\n{}", tmpl.title, tmpl.blurb, tmpl.how_to)
            };
            self.ui
                .widget(cx, ids!(onboard_tmpl_hint))
                .set_text(cx, &plain_display_text(&tmpl_hint));
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
            // Books home stat cards: each figure is its own label/value
            // pair (Store::books_overview) instead of one text paragraph.
            {
                let overview = {
                    let st = app_state().lock().unwrap();
                    st.store.as_ref().and_then(|s| s.books_overview().ok())
                };
                if let Some(o) = overview {
                    let mut set = |id: &[LiveId], text: &str| {
                        self.ui
                            .widget(cx, id)
                            .set_text(cx, &plain_display_text(text));
                    };
                    set(ids!(books_in_value), &o.money_in);
                    set(ids!(books_purchases_value), &o.purchases);
                    set(ids!(books_expenses_value), &o.expenses);
                    set(ids!(books_gross_value), &o.gross);
                    set(ids!(books_net_value), &o.net);
                    set(ids!(books_declared_name), &o.lane_declared_label);
                    set(ids!(books_declared_value), &o.lane_declared);
                    set(ids!(books_owner_name), &o.lane_owner_label);
                    set(ids!(books_owner_value), &o.lane_owner);
                    set(ids!(books_cash_value), &o.cash);
                    set(ids!(books_digital_name), &o.digital_label);
                    set(ids!(books_digital_value), &o.digital);
                    set(ids!(books_card_value), &o.card);
                    set(ids!(books_they_owe_value), &o.they_owe);
                    set(ids!(books_we_owe_value), &o.we_owe);
                    set(ids!(books_bank_balance_value), &o.bank_balance);
                    set(ids!(books_bank_net_value), &o.bank_net_today);
                }
                // Perpetual till: every recorded cash movement, all time.
                let till = {
                    let st = app_state().lock().unwrap();
                    st.store
                        .as_ref()
                        .and_then(|s| s.till_balance_paise().ok())
                        .map(|v| st.money(Money(v)))
                };
                if let Some(till) = till {
                    self.ui
                        .widget(cx, ids!(books_till_value))
                        .set_text(cx, &plain_display_text(&till));
                }
            }
            self.ui
                .widget(cx, ids!(reports_label)).set_text(cx, &plain_display_text(&day_close));
            self.ui
                .widget(cx, ids!(inventory_label)).set_text(cx, &plain_display_text(&inventory_text));
            self.ui
                .widget(cx, ids!(branches_label)).set_text(cx, &plain_display_text(&branches_body));
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
                        s.can_access_sales(),
                        s.can_access_credit(),
                        s.can_access_team() || s.is_owner(),
                        s.can_access_backup() || s.is_owner(),
                        s.can_access_inventory() || s.is_owner(),
                        s.can_access_branches() || s.is_owner(),
                        s.is_owner(),
                    ),
                    None => (false, false, false, false, false, false, false, false, false),
                }
            };
            let (is_client, is_shop, extra_tills, branch_count) = {
                let st = app_state().lock().unwrap();
                let branch_count = st
                    .store
                    .as_ref()
                    .and_then(|s| s.list_branches().ok())
                    .map(|b| b.len())
                    .unwrap_or(1);
                (
                    st.device_role == DeviceRole::Client,
                    st.device_role == DeviceRole::Server,
                    st.extra_tills_allowed,
                    branch_count,
                )
            };
            let _ = is_client;
            let books_ok = sales || is_owner || items;
            let buy_ok = inventory || sales || is_owner;
            // Progressive depth (vision §30): Floor/Kitchen peers exist only
            // when this shop runs tables & kitchen — a counter-only shop
            // never sees the floor.
            let floor_on = app_state().lock().unwrap().venue_mode.creates_kot();
            self.ui
                .button(cx, ids!(allow_extra_tills))
                .set_visible(cx, is_shop && is_owner && !extra_tills);
            self.ui
                .button(cx, ids!(stop_extra_tills))
                .set_visible(cx, is_shop && is_owner && extra_tills);
            // Slots are positions: visibility and enabled state follow the
            // tab the user's arrangement puts in each slot, on both surfaces.
            // (Stock/branches stay Server-only: a Client's copy is a sync
            // cache, so every write there would be lost.)
            let order = app_state().lock().unwrap().nav_order.clone();
            for rail in [false, true] {
                let wrappers = self.nav_slot_wrappers(rail);
                let pairs = if rail {
                    self.rail_nav_pairs()
                } else {
                    self.bottom_nav_pairs()
                };
                for (slot, tab) in order.iter().enumerate() {
                    let visible = match tab {
                        NavTab::Books | NavTab::Sell | NavTab::Buy | NavTab::Lock => true,
                        NavTab::Menu => items,
                        NavTab::Floor | NavTab::Kitchen => sales && floor_on,
                        NavTab::Sales => sales,
                        NavTab::DayClose | NavTab::Reports => sales || is_owner,
                        NavTab::Inventory => inventory && !is_client,
                        NavTab::Branches => branches && !is_client && branch_count > 1,
                        NavTab::Credit => credit,
                        NavTab::Team => team,
                        NavTab::Settings => backup,
                    };
                    self.ui.view(cx, wrappers[slot]).set_visible(cx, visible);
                    let enabled = match tab {
                        NavTab::Books => books_ok,
                        NavTab::Buy => buy_ok,
                        _ => true,
                    };
                    self.ui.button(cx, pairs[slot].0).set_enabled(cx, enabled);
                    self.ui.button(cx, pairs[slot].1).set_enabled(cx, enabled);
                }
            }
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
        // Reports title after paint_chrome so locale strings do not overwrite it.
        {
            let screen = app_state().lock().unwrap().screen;
            if screen == Screen::Reports {
                self.ui
                    .widget(cx, ids!(reports_title))
                    .set_text(cx, &plain_display_text("Reports"));
                self.ui.widget(cx, ids!(reports_help)).set_text(
                    cx,
                    &plain_display_text("Day books, party aging, bank posts, and period P&L."),
                );
            }
        }
        // Re-apply after chrome labels — hot-reload / focus must not leave the
        // wrong bottom-nav peer lit (e.g. Sell while Buy is showing).
        let screen = app_state().lock().unwrap().screen;
        self.paint_nav_tabs(cx, screen);
        self.sync_dev_size(cx);
    }

    /// Cut Makepad 2 hover/focus animators so a hidden or deselected face cannot
    /// keep looking "pressed/selected" after navigation.
    fn clear_button_animators(&self, cx: &mut Cx, path: &[LiveId]) {
        let btn = self.ui.button(cx, path);
        btn.reset_hover(cx);
        if let Some(mut inner) = btn.borrow_mut() {
            inner.animator_cut(cx, ids!(hover.off));
            inner.animator_cut(cx, ids!(focus.off));
        }
    }

    /// Selected choice = active face visible; idle face visible otherwise.
    /// Makepad 2 Button has no selected bit — visibility swap is the highlight,
    /// and animators are cleared so previous faces never stay lit.
    fn paint_choice_pair(
        &mut self,
        cx: &mut Cx,
        idle: &[LiveId],
        active: &[LiveId],
        selected: bool,
    ) {
        self.clear_button_animators(cx, idle);
        self.clear_button_animators(cx, active);
        self.ui.button(cx, idle).set_visible(cx, !selected);
        self.ui.button(cx, active).set_visible(cx, selected);
    }

    /// Expense tender parts from the form: the whole amount by the
    /// first tender, or split when a "second way" part is typed (the
    /// first tender takes the remainder).
    fn expense_parts(
        total: i64,
        split_text: &str,
        currency: &str,
        tender: PayMethod,
        tender2: PayMethod,
    ) -> Result<Vec<(PayMethod, i64)>, String> {
        let split_text = split_text.trim();
        if split_text.is_empty() {
            return Ok(vec![(tender, total)]);
        }
        let part2 = locale::parse_major_input(split_text, currency)?;
        if part2 <= 0 || part2 >= total {
            return Err("The second-way part must be less than the total".into());
        }
        if tender2 == tender {
            return Ok(vec![(tender, total)]);
        }
        Ok(vec![(tender, total - part2), (tender2, part2)])
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

    /// Which nav tab a click belongs to, honouring the user's arrangement:
    /// slots are positions, so a click on slot N means `order[N]` — on
    /// either surface (bottom bar or rail).
    fn nav_tab_clicked(&mut self, cx: &mut Cx, actions: &Actions) -> Option<NavTab> {
        if *nav_suppress_click().lock().unwrap() {
            *nav_suppress_click().lock().unwrap() = false;
            return None;
        }
        let order = app_state().lock().unwrap().nav_order.clone();
        let bottom = self.bottom_nav_pairs();
        let rail = self.rail_nav_pairs();
        for slot in 0..NAV_TABS.len() {
            if self.choice_chip_clicked(cx, actions, bottom[slot].0, bottom[slot].1)
                || self.choice_chip_clicked(cx, actions, rail[slot].0, rail[slot].1)
            {
                return Some(order[slot]);
            }
        }
        None
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
        let tmpl = store.apply_entity_template("khaynjo_restaurant")?;
        st.entity_template_id = tmpl.id.to_string();
        st.venue_mode = VenueMode::Restaurant;
        Ok(())
    }

    fn paint_choice_state(&mut self, cx: &mut Cx) {
        let (
            money_lane,
            draft_role,
            buy_on_account,
            follow_platform,
            onboard_step,
            books_report_kind,
            device_role,
            picking_server,
        ) = {
            let st = app_state().lock().unwrap();
            (
                st.money_lane,
                st.draft_staff_role,
                st.buy_on_account,
                st.language_follow_platform,
                st.onboard_step,
                st.books_report_kind,
                st.device_role,
                st.picking_server,
            )
        };
        self.paint_choice_pair(
            cx,
            ids!(reports_today),
            ids!(reports_today_on),
            books_report_kind == BooksReportKind::Today,
        );
        self.paint_choice_pair(
            cx,
            ids!(reports_month),
            ids!(reports_month_on),
            books_report_kind == BooksReportKind::Month,
        );
        self.paint_choice_pair(
            cx,
            ids!(reports_aging),
            ids!(reports_aging_on),
            books_report_kind == BooksReportKind::Aging,
        );
        self.paint_choice_pair(
            cx,
            ids!(reports_balance_sheet),
            ids!(reports_balance_sheet_on),
            books_report_kind == BooksReportKind::BalanceSheet,
        );
        if picking_server {
            for path in [
                ids!(device_role_server),
                ids!(device_role_server_on),
                ids!(device_role_client),
                ids!(device_role_client_on),
            ] {
                self.ui.button(cx, path).set_visible(cx, false);
            }
        } else {
            self.paint_choice_pair(
                cx,
                ids!(device_role_server),
                ids!(device_role_server_on),
                device_role == DeviceRole::Server,
            );
            self.paint_choice_pair(
                cx,
                ids!(device_role_client),
                ids!(device_role_client_on),
                device_role == DeviceRole::Client,
            );
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
        // Party-form kind chips: highlight whatever the kind field currently
        // holds (a picked party's saved kind, or the chip last tapped).
        let kind_norm = domain::normalize_party_roles(
            &self.ui.text_input(cx, ids!(credit_add_kind)).text(),
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_customer),
            ids!(credit_kind_customer_on),
            kind_norm == "Customer",
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_both),
            ids!(credit_kind_both_on),
            kind_norm == "Customer, Supplier",
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_supplier),
            ids!(credit_kind_supplier_on),
            kind_norm == "Supplier",
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
        // Short chrome labels for bottom chips + rail (wrap, no ellipsis).
        // Long More-row strings stay on more_* buttons below.
        let books = s("books");
        let sell = s("sell");
        let buy = s("buy");
        let rates = s("items");
        let sales = s("sales");
        let day_close = s("day_close");
        let reports = s("nav_reports");
        let inventory = s("inventory");
        let branches = s("branches");
        let credit = s("credit");
        let team = s("team");
        let settings = s("nav_settings");
        let lock = s("nav_lock");
        // Slots are positions; the tab (and so the label) each slot wears
        // comes from the user's saved arrangement.
        let order = app_state().lock().unwrap().nav_order.clone();
        let label_for = |tab: NavTab| -> &str {
            match tab {
                NavTab::Books => &books,
                NavTab::Sell => &sell,
                NavTab::Buy => &buy,
                NavTab::Menu => &rates,
                NavTab::Floor => "Floor",
                NavTab::Kitchen => "Kitchen",
                NavTab::Sales => &sales,
                NavTab::DayClose => &day_close,
                NavTab::Reports => &reports,
                NavTab::Inventory => &inventory,
                NavTab::Branches => &branches,
                NavTab::Credit => &credit,
                NavTab::Team => &team,
                NavTab::Settings => &settings,
                NavTab::Lock => &lock,
            }
        };
        for pairs in [self.bottom_nav_pairs(), self.rail_nav_pairs()] {
            for (slot, (idle, on)) in pairs.iter().enumerate() {
                let label = label_for(order[slot]);
                self.ui.button(cx, idle).set_text(cx, label);
                self.ui.button(cx, on).set_text(cx, label);
            }
        }
        self.ui.button(cx, ids!(books_to_sell)).set_text(cx, &s("sell_money_in"));
        self.ui.button(cx, ids!(books_to_buy)).set_text(cx, &s("buy_money_out"));
        self.ui.button(cx, ids!(books_day_close)).set_text(cx, &s("day_close"));
        self.ui.button(cx, ids!(books_reports)).set_text(cx, &s("reports_expenses"));
        self.ui.button(cx, ids!(books_credit)).set_text(cx, &s("credit_customers"));
        self.ui.button(cx, ids!(buy_purchase)).set_text(cx, &s("record_purchase"));
        self.ui.button(cx, ids!(buy_expense_add)).set_text(cx, &s("add_expense"));
        self.ui.button(cx, ids!(buy_to_inventory)).set_text(cx, &s("inventory_stock"));
        self.ui.widget(cx, ids!(books_heading)).set_text(cx, &plain_display_text(&s("books")));
        self.ui.widget(cx, ids!(books_help)).set_text(cx, &plain_display_text(&s("books_help")));
        self.ui.widget(cx, ids!(buy_heading)).set_text(cx, &plain_display_text(&s("buy")));
        self.ui.widget(cx, ids!(buy_help)).set_text(cx, &plain_display_text(&s("buy_help")));
        self.ui
            .button(cx, ids!(client_pair_back))
            .set_text(cx, &s("back"));
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
        self.ui.button(cx, ids!(image_clear)).set_text(cx, "Clear photo");
        self.ui.button(cx, ids!(image_pick)).set_text(cx, &s("photo"));
        self.ui.button(cx, ids!(role_owner)).set_text(cx, &s("owner"));
        self.ui.button(cx, ids!(role_staff)).set_text(cx, &s("staff"));
        self.ui.button(cx, ids!(lock_staff_prev)).set_text(cx, &s("prev"));
        self.ui.button(cx, ids!(lock_staff_next)).set_text(cx, &s("next"));
        self.ui.button(cx, ids!(pin_ok)).set_text(cx, &s("ok"));
        for path in [ids!(device_role_server), ids!(device_role_server_on)] {
            self.ui.button(cx, path).set_text(cx, &s("this_shop"));
        }
        for path in [ids!(device_role_client), ids!(device_role_client_on)] {
            self.ui.button(cx, path).set_text(cx, &s("extra_till"));
        }
        self.ui.button(cx, ids!(customer_clear)).set_text(cx, &s("clear"));
        self.ui.button(cx, ids!(undo_snack_btn)).set_text(cx, &s("undo"));
        self.ui.button(cx, ids!(inv_adjust_minus)).set_text(cx, &s("stock_minus"));
        self.ui.button(cx, ids!(inv_adjust_plus)).set_text(cx, &s("stock_plus"));
        self.ui.button(cx, ids!(inv_track)).set_text(cx, &s("start_tracking"));
        self.ui.button(cx, ids!(inv_purchase)).set_text(cx, &s("record_purchase"));
        self.ui.button(cx, ids!(settle_cash)).set_text(cx, &s("settle_cash"));
        self.ui.button(cx, ids!(settle_card)).set_text(cx, &s("settle_card"));
        self.ui.button(cx, ids!(amend_pay_cash)).set_text(cx, &s("cash"));
        self.ui.button(cx, ids!(amend_pay_card)).set_text(cx, &s("card"));
        self.ui.widget(cx, ids!(items_heading)).set_text(cx, &plain_display_text(&s("your_items")));
        self.ui.widget(cx, ids!(items_help_label)).set_text(cx, &plain_display_text(&s("items_help")));
        self.ui.widget(cx, ids!(sales_title)).set_text(cx, &plain_display_text(&s("sales")));
        self.ui.widget(cx, ids!(credit_title)).set_text(cx, &plain_display_text(&s("credit")));
        self.ui
            .widget(cx, ids!(credit_help))
            .set_text(cx, &plain_display_text(&s("credit_help")));
        self.ui
            .text_input(cx, ids!(credit_add_kind))
            .set_empty_text(cx, s("party_kind_ph"));
        self.ui.widget(cx, ids!(team_title)).set_text(cx, &plain_display_text(&s("team")));
        self.ui.widget(cx, ids!(settings_title)).set_text(cx, &plain_display_text(&s("language_currency")));
        self.ui.widget(cx, ids!(day_close_title)).set_text(cx, &plain_display_text(&s("day_close")));
        self.ui.widget(cx, ids!(reports_title)).set_text(cx, &plain_display_text(&s("reports")));
        self.ui
            .widget(cx, ids!(reports_help))
            .set_text(cx, &plain_display_text(&s("reports_help")));
        for path in [ids!(reports_today), ids!(reports_today_on)] {
            self.ui.button(cx, path).set_text(cx, &s("reports_today"));
        }
        for path in [ids!(reports_month), ids!(reports_month_on)] {
            self.ui.button(cx, path).set_text(cx, &s("reports_month"));
        }
        for path in [ids!(reports_aging), ids!(reports_aging_on)] {
            self.ui.button(cx, path).set_text(cx, &s("reports_aging"));
        }
        for path in [ids!(reports_balance_sheet), ids!(reports_balance_sheet_on)] {
            self.ui
                .button(cx, path)
                .set_text(cx, &s("reports_balance_sheet"));
        }
        self.ui.button(cx, ids!(expense_add)).set_text(cx, &s("add_expense"));
        self.ui
            .widget(cx, ids!(reports_expense_heading))
            .set_text(cx, &plain_display_text(&s("add_expense")));
        self.ui
            .widget(cx, ids!(reports_bank_heading))
            .set_text(cx, &plain_display_text(&s("bank_book")));
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
        // Device-role chip visibility (idle/active pair) is owned by paint_choice_state.
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

    /// LiveId path of the page `ScrollYView` for `screen`.
    fn screen_scroll_view(screen: Screen) -> &'static [LiveId] {
        match screen {
            Screen::DeviceSetup => ids!(device_setup_screen),
            Screen::Lock => ids!(lock_screen),
            Screen::Onboard => ids!(onboard_screen),
            Screen::ClientPair => ids!(client_pair_screen),
            Screen::Books => ids!(books_screen),
            Screen::Pos => ids!(pos_screen),
            Screen::Buy => ids!(buy_screen),
            Screen::Floor => ids!(floor_screen),
            Screen::Kitchen => ids!(kitchen_screen),
            Screen::Menu => ids!(menu_screen),
            Screen::Sales => ids!(sales_screen),
            Screen::Credit => ids!(credit_screen),
            Screen::Team => ids!(team_screen),
            Screen::Settings => ids!(settings_screen),
            Screen::DayClose => ids!(day_close_screen),
            Screen::Reports => ids!(reports_screen),
            Screen::Inventory => ids!(inventory_screen),
            Screen::Branches => ids!(branches_screen),
        }
    }

    /// Keyboard scroll for the active page ScrollYView.
    /// ↑/↓ move by a line step; PgUp jumps to top; PgDn jumps to bottom.
    /// Returns true when the key was consumed (including no-op at limits).
    fn handle_scroll_key(&mut self, cx: &mut Cx, e: &KeyEvent) -> bool {
        if !matches!(
            e.key_code,
            KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::PageUp | KeyCode::PageDown
        ) {
            return false;
        }
        if e.modifiers.control || e.modifiers.alt || e.modifiers.logo {
            return false;
        }
        // Text fields own arrows / page keys for caret movement.
        if !cx.key_focus().is_empty() {
            return false;
        }
        let screen = app_state().lock().unwrap().screen;
        let view = self.ui.view(cx, Self::screen_scroll_view(screen));
        let pos = view.get_scroll_pos();
        let total = view.get_scroll_view_total();
        let visible = view.get_scroll_view_visible();
        let max_y = (total.y - visible.y).max(0.0);
        // One comfortable line; hold-to-repeat uses is_repeat from the OS.
        const ARROW_STEP: f64 = 48.0;
        let new_y = match e.key_code {
            KeyCode::ArrowUp => (pos.y - ARROW_STEP).max(0.0),
            KeyCode::ArrowDown => (pos.y + ARROW_STEP).min(max_y),
            KeyCode::PageUp => 0.0,
            KeyCode::PageDown => max_y,
            _ => return false,
        };
        if (new_y - pos.y).abs() < 0.5 {
            return true;
        }
        view.set_scroll_pos(cx, dvec2(pos.x, new_y));
        view.redraw(cx);
        true
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
            (ids!(floor_screen), screen == Screen::Floor),
            (ids!(kitchen_screen), screen == Screen::Kitchen),
            (ids!(menu_screen), screen == Screen::Menu),
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
        self.apply_shell_layout(cx, unlocked);
        self.paint_nav_tabs(cx, screen);
    }

    /// Phone (<720): bottom tabs. Tablet/desktop (≥720): side rail.
    /// Order desk: side-by-side only when the *content* column fits two panes;
    /// otherwise stack full-width (never Fixed cart leaving empty right gutter).
    fn apply_shell_layout(&mut self, cx: &mut Cx, unlocked: bool) {
        const DESIGN_DESKTOP_W: f64 = store::DEFAULT_DESKTOP_WINDOW_W;
        // Two panes need ~280 + 280 + spacing inside the content column.
        const POS_SIDE_BY_SIDE_MIN_CONTENT: f64 = 640.0;
        const DIVIDER_W: f64 = 6.0;
        let window_w = {
            let w = self.window_inner_width(cx);
            if w >= 1.0 {
                w
            } else {
                DESIGN_DESKTOP_W
            }
        };
        let shell_wide = window_w >= store::TABLET_SHELL_BREAKPOINT_CSS;
        let show_rail = unlocked && shell_wide;
        let show_bottom = unlocked && !shell_wide;
        self.ui.view(cx, ids!(nav_rail)).set_visible(cx, show_rail);
        self.ui.view(cx, ids!(rail_divider)).set_visible(cx, show_rail);
        self.ui.view(cx, ids!(bottom_nav)).set_visible(cx, show_bottom);

        let rail_w = if show_rail {
            let preferred = app_state().lock().unwrap().rail_width;
            let max_rail = (window_w - MAIN_CONTENT_MIN - DIVIDER_W).max(RAIL_WIDTH_MIN);
            let clamped = preferred.clamp(RAIL_WIDTH_MIN, max_rail);
            {
                let mut st = app_state().lock().unwrap();
                st.rail_width = clamped;
            }
            set_view_walk(
                cx,
                &self.ui.view(cx, ids!(nav_rail)),
                Walk {
                    width: Size::Fixed(clamped),
                    height: Size::fill(),
                    ..Default::default()
                },
            );
            clamped
        } else {
            0.0
        };

        // Content width ≈ window − rail − divider − screen padding (16+16).
        let content_w = (window_w
            - if show_rail { rail_w + DIVIDER_W } else { 0.0 }
            - 32.0)
            .max(280.0);
        let side_by_side = content_w >= POS_SIDE_BY_SIDE_MIN_CONTENT;

        // Down = each child takes full row width. Right (no wrap) = share one row.
        // Avoid Right{wrap}+Fixed: wrapped cart stayed ~320px with a dead gutter.
        set_view_flow(
            cx,
            &self.ui.view(cx, ids!(pos_split)),
            if side_by_side {
                Flow::right()
            } else {
                Flow::Down
            },
        );

        let (catalog_w, cart_w) = if side_by_side {
            // Menu grid is the work surface (top POS apps give it the room);
            // the ticket only needs room for lines + tenders. Both Fill —
            // grow together across phone→ultrawide; no Fixed cap. Minimums
            // must stay well under POS_SIDE_BY_SIDE_MIN_CONTENT together
            // (mins + spacing > content width pushed the ticket off the
            // right edge at 1024px — Clear button clipped, 2026-10-22).
            (size_fill(1.6, 260.0), size_fill(1.0, 250.0))
        } else {
            let full = size_fill(100.0, content_w);
            (full, full)
        };
        set_view_walk(
            cx,
            &self.ui.view(cx, ids!(pos_catalog)),
            Walk {
                width: catalog_w,
                height: Size::fit(),
                ..Default::default()
            },
        );
        set_view_walk(
            cx,
            &self.ui.view(cx, ids!(pos_cart_col)),
            Walk {
                width: cart_w,
                height: Size::fit(),
                ..Default::default()
            },
        );
    }

    /// Nav slot hardware in DSL order. Slots are positions; which tab a
    /// slot shows comes from `AppState.nav_order` (drag to swap).
    fn bottom_nav_pairs(&self) -> [(&[LiveId], &[LiveId]); NAV_TABS.len()] {
        [
            (ids!(nav_books), ids!(nav_books_on)),
            (ids!(nav_sell), ids!(nav_sell_on)),
            (ids!(nav_buy), ids!(nav_buy_on)),
            (ids!(nav_menu), ids!(nav_menu_on)),
            (ids!(nav_floor), ids!(nav_floor_on)),
            (ids!(nav_kitchen), ids!(nav_kitchen_on)),
            (ids!(nav_sales), ids!(nav_sales_on)),
            (ids!(nav_day_close), ids!(nav_day_close_on)),
            (ids!(nav_reports), ids!(nav_reports_on)),
            (ids!(nav_inventory), ids!(nav_inventory_on)),
            (ids!(nav_branches), ids!(nav_branches_on)),
            (ids!(nav_credit), ids!(nav_credit_on)),
            (ids!(nav_team), ids!(nav_team_on)),
            (ids!(nav_settings), ids!(nav_settings_on)),
            (ids!(nav_lock), ids!(nav_lock_on)),
        ]
    }

    fn rail_nav_pairs(&self) -> [(&[LiveId], &[LiveId]); NAV_TABS.len()] {
        [
            (ids!(rail_books), ids!(rail_books_on)),
            (ids!(rail_sell), ids!(rail_sell_on)),
            (ids!(rail_buy), ids!(rail_buy_on)),
            (ids!(rail_rates), ids!(rail_rates_on)),
            (ids!(rail_ops), ids!(rail_ops_on)),
            (ids!(rail_cod), ids!(rail_cod_on)),
            (ids!(rail_sales), ids!(rail_sales_on)),
            (ids!(rail_day_close), ids!(rail_day_close_on)),
            (ids!(rail_reports), ids!(rail_reports_on)),
            (ids!(rail_inventory), ids!(rail_inventory_on)),
            (ids!(rail_branches), ids!(rail_branches_on)),
            (ids!(rail_credit), ids!(rail_credit_on)),
            (ids!(rail_team), ids!(rail_team_on)),
            (ids!(rail_settings), ids!(rail_settings_on)),
            (ids!(rail_lock), ids!(rail_lock_on)),
        ]
    }

    fn nav_slot_wrappers(&self, rail: bool) -> [&[LiveId]; NAV_TABS.len()] {
        if rail {
            [
                ids!(rail_books_slot),
                ids!(rail_sell_slot),
                ids!(rail_buy_slot),
                ids!(rail_rates_slot),
                ids!(rail_ops_slot),
                ids!(rail_cod_slot),
                ids!(rail_sales_slot),
                ids!(rail_day_close_slot),
                ids!(rail_reports_slot),
                ids!(rail_inventory_slot),
                ids!(rail_branches_slot),
                ids!(rail_credit_slot),
                ids!(rail_team_slot),
                ids!(rail_settings_slot),
                ids!(rail_lock_slot),
            ]
        } else {
            [
                ids!(nav_books_slot),
                ids!(nav_sell_slot),
                ids!(nav_buy_slot),
                ids!(nav_menu_slot),
                ids!(nav_floor_slot),
                ids!(nav_kitchen_slot),
                ids!(nav_sales_slot),
                ids!(nav_day_close_slot),
                ids!(nav_reports_slot),
                ids!(nav_inventory_slot),
                ids!(nav_branches_slot),
                ids!(nav_credit_slot),
                ids!(nav_team_slot),
                ids!(nav_settings_slot),
                ids!(nav_lock_slot),
            ]
        }
    }

    /// Exactly one nav peer highlighted for `screen` (bottom and/or rail).
    fn paint_nav_tabs(&mut self, cx: &mut Cx, screen: Screen) {
        let order = app_state().lock().unwrap().nav_order.clone();
        let active = nav_tab_for(screen);
        // While dragging, the slot under the pointer wears the active face
        // so the drop target is unmistakable.
        let drop = nav_drag()
            .lock()
            .unwrap()
            .as_ref()
            .filter(|d| d.active)
            .and_then(|d| d.hover.map(|slot| (d.rail, slot)));
        for (rail, pairs) in [(false, self.bottom_nav_pairs()), (true, self.rail_nav_pairs())] {
            for (slot, (idle, on)) in pairs.iter().enumerate() {
                self.clear_button_animators(cx, idle);
                self.clear_button_animators(cx, on);
                let selected = active == Some(order[slot]) || drop == Some((rail, slot));
                self.ui.button(cx, idle).set_visible(cx, !selected);
                self.ui.button(cx, on).set_visible(cx, selected);
            }
        }
    }

    fn sync_venue_brand(&mut self, cx: &mut Cx) {
        let (title, header_title, icon_path, icon_hint, saved_title) = {
            let st = app_state().lock().unwrap();
            // A shop that has not set its name yet still shows the app name
            // next to the logo — never a blank header.
            let title = if st.venue_title.trim().is_empty() {
                "Khaynjo".to_string()
            } else {
                st.display_venue_title()
            };
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
                "Icon: Kumkaj mark (default) — choose a custom one".to_string()
            };
            (title, header_title, icon_path, icon_hint, st.venue_title.clone())
        };
        self.ui
            .widget(cx, ids!(brand_title)).set_text(cx, &plain_display_text(&header_title));
        for id in [ids!(setup_brand_title), ids!(lock_brand_title)] {
            self.ui.widget(cx, id).set_text(cx, &plain_display_text(&title));
        }
        let default_mark = default_brand_mark_path();
        for id in [
            ids!(brand_mark),
            ids!(setup_brand_mark),
            ids!(lock_brand_mark),
            ids!(onboard_brand_mark),
        ] {
            let img = self.ui.image(cx, id);
            img.set_visible(cx, true);
            if let Some(ref path) = icon_path {
                load_brand_mark_at(cx, &img, path);
            } else {
                // Prefer filesystem / package bytes — `load_image_dep_by_path` is for
                // live deps and misses crate_resource-bundled marks.
                load_brand_mark_at(cx, &img, &default_mark);
            }
        }
        self.ui
            .widget(cx, ids!(venue_brand_hint)).set_text(cx, &plain_display_text(&icon_hint));
        let onboard_icon_hint = if icon_path.is_some() {
            "Logo: your shop logo — shown on screens and receipts"
        } else {
            "Logo: Kumkaj mark (default) — choose your own if you have one"
        };
        self.ui
            .widget(cx, ids!(onboard_icon_hint))
            .set_text(cx, &plain_display_text(onboard_icon_hint));
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

    /// Onboarding logo pick: same image dialog, but the chosen file is
    /// imported immediately (no draft/Save-brand step during setup).
    fn pick_onboard_icon(&mut self, cx: &mut Cx) {
        let dialog = FileDialog::new()
            .set_title("Choose shop logo".into())
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
            .set_id(live_id!(onboard_icon_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening logo picker…");
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
                // Empty stack: only nested setup can fall back. Top-level peers
                // (former More destinations) have no Back parent.
                match st.screen {
                    Screen::ClientPair => {
                        st.nav_forward.clear();
                        st.must_confirm_device_role = true;
                        st.status =
                            "Pick Server Mode or Client Mode — Back left Client pairing".into();
                        Some(Screen::DeviceSetup)
                    }
                    _ => None,
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
        // Never persist maximized/fullscreen geometry — that was rewriting
        // window_geom.json to ~2560×1048 (one ultrawide), so the next launch
        // opened already monitor-sized and the next maximize fought Mutter
        // across the 2×2 layout.
        if win.is_fullscreen(cx) {
            return;
        }
        let size = win.get_inner_size(cx);
        let pos = win.get_position(cx);
        if size.x >= 1920.0 && size.y >= 900.0 {
            return;
        }
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
    /// `KUMKAJ_SHOW_SIZE_BADGE` in this crate's `env/development.env` (not Cargo profile —
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

    /// Drag the rail divider; clamp so rail ≥ [`RAIL_WIDTH_MIN`] and main ≥ [`MAIN_CONTENT_MIN`].
    fn pointer_rail_resize(&mut self, cx: &mut Cx, event: &Event) -> bool {
        const DIVIDER_W: f64 = 6.0;
        let unlocked_rail = {
            let st = app_state().lock().unwrap();
            let unlocked = !matches!(
                st.screen,
                Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
            );
            let wide = self.window_inner_width(cx) >= store::TABLET_SHELL_BREAKPOINT_CSS;
            unlocked && wide
        };
        match event {
            Event::MouseDown(e) if e.button.is_primary() && unlocked_rail => {
                let rail_w = app_state().lock().unwrap().rail_width;
                let x = e.abs.x;
                if x >= rail_w - 2.0 && x <= rail_w + DIVIDER_W + 8.0 {
                    *rail_resize_drag().lock().unwrap() = Some(x - rail_w);
                    *mouse_swipe_origin().lock().unwrap() = None;
                    cx.set_cursor(MouseCursor::ColResize);
                    true
                } else {
                    false
                }
            }
            Event::MouseMove(e) if rail_resize_drag().lock().unwrap().is_some() => {
                let grab = rail_resize_drag().lock().unwrap().unwrap_or(0.0);
                let window_w = self.window_inner_width(cx);
                let max_rail = (window_w - MAIN_CONTENT_MIN - DIVIDER_W).max(RAIL_WIDTH_MIN);
                let new_w = (e.abs.x - grab).clamp(RAIL_WIDTH_MIN, max_rail);
                {
                    app_state().lock().unwrap().rail_width = new_w;
                }
                cx.set_cursor(MouseCursor::ColResize);
                self.apply_shell_layout(cx, true);
                true
            }
            Event::MouseUp(e) if e.button.is_primary() => {
                if rail_resize_drag().lock().unwrap().take().is_some() {
                    self.apply_shell_layout(cx, true);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Which nav slot (position) of a surface is under `pos`, if any.
    /// Which slot (if any) is under `pos`. `pad` grows the drawn face a
    /// few px — used for drop targeting, where landing in the hairline
    /// gap between two items should still count. Drag *pickup* passes
    /// 0: the bars keep whitespace lanes around the items for
    /// scrolling, and a padded pickup area would eat the lane.
    fn nav_slot_at(
        &mut self,
        cx: &mut Cx,
        rail: bool,
        pos: DVec2,
        pad_x: f64,
        pad_y: f64,
    ) -> Option<usize> {
        let pairs = if rail {
            self.rail_nav_pairs()
        } else {
            self.bottom_nav_pairs()
        };
        for (slot, (idle, on)) in pairs.iter().enumerate() {
            for face in [idle, on] {
                let rect = self.ui.button(cx, face).area().rect(cx);
                let hit = rect.size.x > 0.0
                    && rect.size.y > 0.0
                    && pos.x >= rect.pos.x - pad_x
                    && pos.x <= rect.pos.x + rect.size.x + pad_x
                    && pos.y >= rect.pos.y - pad_y
                    && pos.y <= rect.pos.y + rect.size.y + pad_y;
                if hit {
                    return Some(slot);
                }
            }
        }
        None
    }

    /// Drop target during a drag: which slot is under the window
    /// pointer, computed from the drag's frozen content rects and the
    /// bar's live scroll offset (widget areas lag during auto-scroll,
    /// so they cannot be trusted here). Held past the first/last item
    /// — e.g. against the bar's edge while it auto-scrolls — the
    /// nearest end item is the target.
    fn nav_drop_slot(&mut self, cx: &mut Cx, drag: &NavDrag, pos: DVec2) -> Option<usize> {
        let scroll = if drag.rail {
            self.ui.view(cx, ids!(rail_scroll)).get_scroll_pos()
        } else {
            self.ui.view(cx, ids!(bottom_nav_scroll)).get_scroll_pos()
        };
        let pos = DVec2 {
            x: pos.x + scroll.x,
            y: pos.y + scroll.y,
        };
        let mut first_rect: Option<Rect> = None;
        let mut last_rect: Option<Rect> = None;
        for (slot, rect) in drag.rects.iter().enumerate() {
            if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
                continue;
            }
            if first_rect.is_none() {
                first_rect = Some(*rect);
            }
            last_rect = Some(*rect);
            let hit = pos.x >= rect.pos.x - 2.0
                && pos.x <= rect.pos.x + rect.size.x + 2.0
                && pos.y >= rect.pos.y - 5.0
                && pos.y <= rect.pos.y + rect.size.y + 5.0;
            if hit {
                return Some(slot);
            }
        }
        if let (Some(first), Some(last)) = (first_rect, last_rect) {
            let last_slot = drag.rects.len() - 1;
            if drag.rail {
                let in_band = pos.x >= first.pos.x - 28.0
                    && pos.x <= first.pos.x + first.size.x + 28.0;
                if in_band && pos.y >= last.pos.y + last.size.y {
                    return Some(last_slot);
                }
                if in_band && pos.y <= first.pos.y {
                    return Some(0);
                }
            } else {
                let in_band = pos.y >= first.pos.y - 28.0
                    && pos.y <= first.pos.y + first.size.y + 28.0;
                if in_band && pos.x >= last.pos.x + last.size.x {
                    return Some(last_slot);
                }
                if in_band && pos.x <= first.pos.x {
                    return Some(0);
                }
            }
        }
        None
    }

    fn repaint_nav_tabs(&mut self, cx: &mut Cx) {
        let screen = app_state().lock().unwrap().screen;
        self.paint_nav_tabs(cx, screen);
    }

    /// Display label a nav slot wears for `tab` (same strings as the
    /// chrome paint in paint_chrome).
    fn nav_tab_label(&self, tab: NavTab) -> String {
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |key: &str| locale::t(&lang, key);
        match tab {
            NavTab::Books => s("books"),
            NavTab::Sell => s("sell"),
            NavTab::Buy => s("buy"),
            NavTab::Menu => s("items"),
            NavTab::Floor => "Floor".to_string(),
            NavTab::Kitchen => "Kitchen".to_string(),
            NavTab::Sales => s("sales"),
            NavTab::DayClose => s("day_close"),
            NavTab::Reports => s("nav_reports"),
            NavTab::Inventory => s("inventory"),
            NavTab::Branches => s("branches"),
            NavTab::Credit => s("credit"),
            NavTab::Team => s("team"),
            NavTab::Settings => s("nav_settings"),
            NavTab::Lock => s("nav_lock"),
        }
    }

    /// Rect of the face currently drawn for `slot` (idle or active).
    fn nav_slot_rect(&mut self, cx: &mut Cx, rail: bool, slot: usize) -> Option<Rect> {
        let pairs = if rail {
            self.rail_nav_pairs()
        } else {
            self.bottom_nav_pairs()
        };
        let (idle, on) = pairs[slot];
        for face in [idle, on] {
            let rect = self.ui.button(cx, face).area().rect(cx);
            if rect.size.x > 0.0 && rect.size.y > 0.0 {
                return Some(Rect {
                    pos: DVec2 {
                        x: rect.pos.x,
                        y: rect.pos.y,
                    },
                    size: rect.size,
                });
            }
        }
        None
    }

    fn place_nav_ghost(&mut self, cx: &mut Cx, pos: DVec2) {
        let ghost = self.ui.widget(cx, ids!(nav_drag_ghost));
        if let Some(mut view) = ghost.borrow_mut::<View>() {
            view.walk.abs_pos = Some(pos);
        }
        ghost.redraw(cx);
    }

    fn show_nav_ghost(&mut self, cx: &mut Cx, tab: NavTab, pos: DVec2) {
        let label = self.nav_tab_label(tab);
        self.ui
            .widget(cx, ids!(nav_drag_ghost_label))
            .set_text(cx, &label);
        self.place_nav_ghost(cx, pos);
        self.ui.view(cx, ids!(nav_drag_ghost)).set_visible(cx, true);
        // The ghost lives in a Modal so it paints on the overlay draw
        // list, above every scroll view (rail, page, bottom bar alike).
        self.ui.modal(cx, ids!(nav_drag_modal)).open(cx);
        cx.redraw_all();
    }

    fn hide_nav_ghost(&mut self, cx: &mut Cx) {
        self.ui.modal(cx, ids!(nav_drag_modal)).close(cx);
        self.ui.view(cx, ids!(nav_drag_ghost)).set_visible(cx, false);
    }

    /// Drag a sidebar / bottom-bar item onto another one to swap them.
    /// Scroll and drag are separated spatially, not by timing: the bars
    /// keep whitespace lanes around the items (padding and gaps inside
    /// the scroll view), so a press on whitespace belongs to the scroll
    /// view and never reaches this peek — it starts only when the press
    /// lands on an item. From an item, a plain click navigates and any
    /// real movement picks the item up; from then on the drag owns the
    /// gesture so the scroll view cannot scroll the drop target away
    /// mid-drag. The new arrangement is saved into the books, so it
    /// survives reinstall.
    /// Returns true when the event was consumed by the drag.
    fn pointer_nav_drag(&mut self, cx: &mut Cx, event: &Event) -> bool {
        match event {
            Event::MouseDown(e) if e.button.is_primary() => {
                *nav_drag().lock().unwrap() = None;
                // A fresh press starts a fresh gesture — never let a
                // leftover scroll-suppression eat this tap.
                *nav_suppress_click().lock().unwrap() = false;
                self.hide_nav_ghost(cx);
                let unlocked = {
                    let st = app_state().lock().unwrap();
                    !matches!(
                        st.screen,
                        Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                    )
                };
                if !unlocked {
                    return false;
                }
                // Only the visible surface can be dragged (mirrors
                // apply_shell_layout's rail-vs-bottom choice).
                let rail = self.window_inner_width(cx) >= store::TABLET_SHELL_BREAKPOINT_CSS;
                // The rail divider owns its own drag (resize) — stay out of it.
                if rail {
                    let rail_w = app_state().lock().unwrap().rail_width;
                    if e.abs.x >= rail_w - 2.0 && e.abs.x <= rail_w + 14.0 {
                        return false;
                    }
                }
                if let Some(slot) = self.nav_slot_at(cx, rail, e.abs, 0.0, 0.0) {
                    let origin = self
                        .nav_slot_rect(cx, rail, slot)
                        .map(|rect| rect.pos)
                        .unwrap_or(e.abs);
                    // Freeze every slot's rect in content coordinates
                    // (drawn rect + scroll right now, while settled);
                    // drag-time hit tests use these, not live areas.
                    let scroll = if rail {
                        self.ui.view(cx, ids!(rail_scroll)).get_scroll_pos()
                    } else {
                        self.ui.view(cx, ids!(bottom_nav_scroll)).get_scroll_pos()
                    };
                    let pairs = if rail {
                        self.rail_nav_pairs()
                    } else {
                        self.bottom_nav_pairs()
                    };
                    let mut rects = Vec::with_capacity(pairs.len());
                    for (idle, on) in &pairs {
                        let mut found = Rect::default();
                        for face in [idle, on] {
                            let rect = self.ui.button(cx, face).area().rect(cx);
                            if rect.size.x > 0.0 && rect.size.y > 0.0 {
                                found = Rect {
                                    pos: DVec2 {
                                        x: rect.pos.x + scroll.x,
                                        y: rect.pos.y + scroll.y,
                                    },
                                    size: rect.size,
                                };
                                break;
                            }
                        }
                        rects.push(found);
                    }
                    *nav_drag().lock().unwrap() = Some(NavDrag {
                        rail,
                        slot,
                        start: e.abs,
                        last: e.abs,
                        origin,
                        rects,
                        active: false,
                        hover: Some(slot),
                    });
                }
                false
            }
            Event::MouseMove(e) => {
                let mut repaint = false;
                let mut active = false;
                let mut ghost: Option<(bool, NavTab, DVec2)> = None;
                {
                    let mut guard = nav_drag().lock().unwrap();
                    if let Some(d) = guard.as_mut() {
                        let was_active = d.active;
                        if !d.active && (e.abs - d.start).length() > 10.0 {
                            d.active = true;
                        }
                        if d.active {
                            active = true;
                            d.last = e.abs;
                            if !was_active {
                                // Start the per-frame edge auto-scroll
                                // loop for this drag.
                                *nav_autoscroll_frame().lock().unwrap() =
                                    Some(cx.new_next_frame());
                            }
                            let hover = self.nav_drop_slot(cx, d, e.abs);
                            if hover != d.hover {
                                d.hover = hover;
                                repaint = true;
                            }
                            let tab = app_state().lock().unwrap().nav_order[d.slot];
                            ghost = Some((
                                !was_active,
                                tab,
                                d.origin + (e.abs - d.start),
                            ));
                        }
                    }
                }
                if active {
                    // Drag icon for the whole drag — the button's Hand
                    // hover cursor would otherwise stay on screen.
                    cx.set_cursor(MouseCursor::Grabbing);
                }
                if let Some((first, tab, pos)) = ghost {
                    if first {
                        self.show_nav_ghost(cx, tab, pos);
                    } else {
                        self.place_nav_ghost(cx, pos);
                    }
                }
                if repaint {
                    self.repaint_nav_tabs(cx);
                }
                active
            }
            Event::MouseUp(e) if e.button.is_primary() => {
                // Take in its own scope: an `if let` scrutinee guard would
                // stay locked across the block and deadlock the repaint
                // below (paint_nav_tabs reads the same mutex).
                let taken = { nav_drag().lock().unwrap().take() };
                if let Some(d) = taken {
                    self.hide_nav_ghost(cx);
                    if d.active {
                        // Drag over: back to the nav button's pointer when
                        // still over a slot, plain arrow anywhere else.
                        cx.set_cursor(if d.hover.is_some() {
                            MouseCursor::Hand
                        } else {
                            MouseCursor::Default
                        });
                        let swapped = match d.hover {
                            Some(target) if target != d.slot => {
                                let mut st = app_state().lock().unwrap();
                                st.nav_order.swap(d.slot, target);
                                if let Some(store) = st.store.as_ref() {
                                    let _ =
                                        store.set_nav_order_csv(&nav_order_csv(&st.nav_order));
                                }
                                true
                            }
                            _ => false,
                        };
                        if swapped {
                            self.set_status(cx, "Menu order saved");
                        }
                        // The release is consumed, so no Button click is
                        // generated — nothing to suppress afterwards.
                        *nav_suppress_click().lock().unwrap() = false;
                        self.sync_chrome(cx);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Edge auto-scroll, one call per frame while a nav drag is live:
    /// holding the dragged item against the bar's edge scrolls the bar
    /// underneath the stationary pointer — gently at the edge zone's
    /// border, faster the deeper the pointer sits in it — so an item
    /// that was off-screen when the drag began can still be reached
    /// and swapped with. The drop target is recomputed after every
    /// step, because the items just slid under the pointer.
    fn nav_drag_autoscroll(&mut self, cx: &mut Cx) {
        let (rail, last) = {
            let guard = nav_drag().lock().unwrap();
            match guard.as_ref() {
                Some(d) if d.active => (d.rail, d.last),
                _ => return, // drag over: let the frame loop die
            }
        };
        // Keep the loop alive for the whole drag.
        *nav_autoscroll_frame().lock().unwrap() = Some(cx.new_next_frame());
        let view = if rail {
            self.ui.view(cx, ids!(rail_scroll))
        } else {
            self.ui.view(cx, ids!(bottom_nav_scroll))
        };
        let rect = view.area().rect(cx);
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        const ZONE: f64 = 56.0;
        let (near_start, near_end) = if rail {
            (
                (ZONE - (last.y - rect.pos.y)).clamp(0.0, ZONE),
                (ZONE - (rect.pos.y + rect.size.y - last.y)).clamp(0.0, ZONE),
            )
        } else {
            (
                (ZONE - (last.x - rect.pos.x)).clamp(0.0, ZONE),
                (ZONE - (rect.pos.x + rect.size.x - last.x)).clamp(0.0, ZONE),
            )
        };
        let step = |depth: f64| {
            if depth > 0.0 {
                2.0 + depth / ZONE * 10.0
            } else {
                0.0
            }
        };
        let delta = step(near_end) - step(near_start);
        if delta == 0.0 {
            return;
        }
        let cur = view.get_scroll_pos();
        let total = view.get_scroll_view_total();
        let visible = view.get_scroll_view_visible();
        let mut next = cur;
        if rail {
            next.y = (cur.y + delta).clamp(0.0, (total.y - visible.y).max(0.0));
        } else {
            next.x = (cur.x + delta).clamp(0.0, (total.x - visible.x).max(0.0));
        }
        if next.x == cur.x && next.y == cur.y {
            return; // already at the scroll end
        }
        view.set_scroll_pos(cx, next);
        let mut repaint = false;
        {
            let mut guard = nav_drag().lock().unwrap();
            if let Some(d) = guard.as_mut() {
                let hover = self.nav_drop_slot(cx, d, d.last);
                if hover != d.hover {
                    d.hover = hover;
                    repaint = true;
                }
            }
        }
        if repaint {
            self.repaint_nav_tabs(cx);
        }
    }

    /// Mouse Back/Forward, left-edge swipe → back, right-edge swipe → forward.
    /// TouchUpdate on phones; primary MouseDown/Up edge drag on desktop.
    /// Top-level peers do not keep history, so Back/swipe is a no-op there
    /// (except dismissing the soft keyboard).
    fn pointer_navigates(&mut self, cx: &mut Cx, event: &Event) -> bool {
        if self.pointer_rail_resize(cx, event) {
            return true;
        }
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
                // Skip edge-swipe arming while the rail is showing — left edge
                // is the rail itself; swipe-back is for nested setup only.
                let skip_edge = {
                    let st = app_state().lock().unwrap();
                    let unlocked = !matches!(
                        st.screen,
                        Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                    );
                    unlocked && self.window_inner_width(cx) >= store::TABLET_SHELL_BREAKPOINT_CSS
                };
                let width = self.window_inner_width(cx);
                let mut origin = mouse_swipe_origin().lock().unwrap();
                if skip_edge {
                    *origin = None;
                } else if e.abs.x < 48.0 {
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
            let _ = store.set_device_role(DeviceRole::Server.as_str());
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
                st.onboard_step = 1;
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
        let _ = cx;
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            if st.session.as_ref().map(|s| s.can_access_sell()) != Some(true) {
                Some("Not allowed to order".into())
            } else if let Some(product) = st.products.get(index).cloned() {
                match st.cart.add_or_bump_in_stock(&product) {
                    Err(e) => Some(e),
                    Ok(()) => {
                        st.status = format!("{} added", product.name);
                        None
                    }
                }
            } else {
                Some("Missing item".into())
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
                let currency = st.locale.currency.clone();
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
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
                        return Err("Type a customer name, then tap Credit (khata)".into());
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
                    // A cash bill may still carry the picked customer's name.
                    st.selected_customer_id.clone()
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
                    st.status = format!("Billed {} · {}", sale.invoice_no, method.label());
                    let snack = format!("{} · {} billed", amount, method.label());
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
                    st.status = format!("Recorded {} · {} on Server", sale.invoice_no, method.label());
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
            ids!(menu_modifiers_input),
            ids!(menu_station_input),
        ] {
            self.ui.text_input(cx, path).set_text(cx, "");
        }
        let mut st = app_state().lock().unwrap();
        st.editing_product_id = None;
        st.draft_image_key = None;
        st.form_category_id = None;
        st.form_category_label.clear();
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
                let aliases = product.aliases().join(", ");
                let modifiers = modifiers_as_typed(&product.modifiers(), &currency);
                let price = locale::format_major_input(product.price.paise(), &currency);
                Some((product, tax, category, aliases, modifiers, price))
            }
        };
        let Some((product, tax, category, aliases, modifiers, price)) = loaded else {
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
            .text_input(cx, ids!(menu_modifiers_input))
            .set_text(cx, &modifiers);
        let station_name = {
            let st = app_state().lock().unwrap();
            st.stations
                .iter()
                .find(|s| s.id == product.station_id)
                .map(|s| s.name.clone())
                .unwrap_or_default()
        };
        self.ui
            .text_input(cx, ids!(menu_station_input))
            .set_text(cx, &station_name);
        {
            let mut st = app_state().lock().unwrap();
            st.editing_product_id = Some(product.id.clone());
            st.draft_image_key = product.image_key.clone();
            // The loaded item's category travels by UUID too.
            if product.category_id.is_empty() {
                st.form_category_id = None;
                st.form_category_label.clear();
            } else {
                st.form_category_id = Some(product.category_id.clone());
                st.form_category_label = category.clone();
            }
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
        let modifiers_text = self.ui.text_input(cx, ids!(menu_modifiers_input)).text();
        let station_text = self.ui.text_input(cx, ids!(menu_station_input)).text();
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
            // A category picked from the Categories list travels by
            // UUID; the typed text only has to still match what was
            // picked. Retyping the field hands control back to name
            // resolution (find-or-create by name — a typing
            // convenience, never the identity).
            let typed = category_text.trim();
            let picked_id = match (&st.form_category_id, &st.form_category_label) {
                (Some(id), label) if typed.eq_ignore_ascii_case(label) => store
                    .list_categories()?
                    .into_iter()
                    .find(|c| &c.id == id)
                    .map(|c| c.id),
                _ => None,
            };
            let category_id = match picked_id {
                Some(id) => id,
                None => match typed {
                    "" => String::new(),
                    wanted => match store
                        .list_categories()?
                        .into_iter()
                        .find(|c| c.name.eq_ignore_ascii_case(wanted))
                    {
                        Some(c) => c.id,
                        None => store.add_category(wanted)?.id,
                    },
                },
            };
            // Kitchen station by name (created if new), same pattern as category.
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
                domain::ModifierOption::parse_human_list(&modifiers_text, &currency)?;
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

    fn image_clear_photo(&mut self, cx: &mut Cx) {
        app_state().lock().unwrap().draft_image_key = None;
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
                ],
            )
            .set_id(live_id!(item_photo_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening photo picker…");
    }

    fn attach_pick_image(&mut self, cx: &mut Cx, entity: AttachmentEntity) {
        app_state().lock().unwrap().attach_pick = Some(AttachPickTarget::Image(entity));
        let dialog = FileDialog::new()
            .set_title("Attach photo".into())
            .add_filter(
                "Images".into(),
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                ],
            )
            .set_id(live_id!(entry_attach_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening photo picker…");
    }

    fn attach_pick_pdf(&mut self, cx: &mut Cx, entity: AttachmentEntity) {
        app_state().lock().unwrap().attach_pick = Some(AttachPickTarget::File(entity));
        let dialog = FileDialog::new()
            .set_title("Attach PDF".into())
            .add_filter("Documents".into(), vec!["pdf".into(), "txt".into()])
            .set_id(live_id!(entry_attach_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening file picker…");
    }

    /// Which row an attachment belongs to (sale / party / expense selection).
    fn entity_id_for_attach(&self, entity: AttachmentEntity) -> Result<String, String> {
        let st = app_state().lock().unwrap();
        let id = match entity {
            AttachmentEntity::Sale => st.selected_sale_id.clone(),
            AttachmentEntity::Party => st.selected_customer_id.clone(),
            AttachmentEntity::Expense => st.selected_expense_id.clone(),
            AttachmentEntity::Purchase => None,
        };
        id.filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "Select an entry first".into())
    }

    fn on_entry_attach_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        let target = app_state().lock().unwrap().attach_pick.take();
        let Some(target) = target else {
            self.set_status(cx, "Nothing to attach");
            return;
        };
        match self.materialize_picked_file(cx, &path) {
            Ok(file) => match target {
                AttachPickTarget::Image(entity) => {
                    let result = (|| -> Result<(), String> {
                        let entity_id = self.entity_id_for_attach(entity)?;
                        let mut st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.add_attachment_image(entity, &entity_id, &file, "", &actor)?;
                        st.status = "Photo attached (stored as WebP)".into();
                        Ok(())
                    })();
                    match result {
                        Ok(()) => self.set_status(cx, "Photo attached (stored as WebP)"),
                        Err(e) => self.set_status(cx, e),
                    }
                    self.sync_chrome(cx);
                }
                AttachPickTarget::File(entity) => {
                    let result = (|| -> Result<(), String> {
                        let entity_id = self.entity_id_for_attach(entity)?;
                        let mut st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.add_attachment_file(entity, &entity_id, &file, "", &actor)?;
                        st.status = "File attached".into();
                        Ok(())
                    })();
                    match result {
                        Ok(()) => self.set_status(cx, "File attached"),
                        Err(e) => self.set_status(cx, e),
                    }
                    self.sync_chrome(cx);
                }
            },
            Err(e) => self.set_status(cx, e),
        }
    }

    fn add_entry_note(
        &mut self,
        cx: &mut Cx,
        entity: AttachmentEntity,
        note_id: &[LiveId],
    ) {
        let body = self.ui.text_input(cx, note_id).text();
        let result = (|| -> Result<(), String> {
            let entity_id = self.entity_id_for_attach(entity)?;
            let mut st = app_state().lock().unwrap();
            let actor = st.actor()?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.add_attachment_note(entity, &entity_id, "", &body, &actor)?;
            st.status = "Note added".into();
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.ui.text_input(cx, note_id).set_text(cx, "");
                self.set_status(cx, "Note added");
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
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

    fn on_category_photo_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        let category_id = app_state().lock().unwrap().category_photo_id.take();
        let Some(category_id) = category_id else {
            self.set_status(cx, "Pick a category first");
            return;
        };
        let outcome = (|| -> Result<String, String> {
            let file = self.materialize_picked_file(cx, &path)?;
            let mut st = app_state().lock().unwrap();
            let store = st
                .store
                .as_ref()
                .ok_or_else(|| "Storage not ready".to_string())?;
            let key = store.import_category_image(&category_id, &file)?;
            store.set_category_image(&category_id, &key)?;
            st.categories = store.list_categories().unwrap_or_default();
            Ok("Category photo saved".to_string())
        })();
        match outcome {
            Ok(msg) => self.set_status(cx, msg),
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

        // Page ScrollYView: ↑/↓ step, PgUp → top, PgDn → bottom.
        // Skip when a TextInput (or other widget) holds key focus.
        if self.handle_scroll_key(cx, e) {
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
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(onboard_icon_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    let outcome = (|| -> Result<String, String> {
                                        let mut st = app_state().lock().unwrap();
                                        let store = st
                                            .store
                                            .as_ref()
                                            .ok_or_else(|| "Storage not ready".to_string())?;
                                        let key = store.import_venue_icon(&file)?;
                                        st.venue_icon_key = Some(key);
                                        Ok("Shop logo saved".to_string())
                                    })();
                                    match outcome {
                                        Ok(msg) => self.set_status(cx, msg),
                                        Err(e) => self.set_status(cx, e),
                                    }
                                    self.sync_chrome(cx);
                                }
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(category_photo_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_category_photo_picked(cx, path.clone());
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(entry_attach_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_entry_attach_picked(cx, path.clone());
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(csv_parties_pick)
                            || *id == live_id!(csv_items_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            let parties = *id == live_id!(csv_parties_pick);
                            let outcome = (|| -> Result<String, String> {
                                let text = std::fs::read_to_string(path)
                                    .map_err(|e| format!("Could not read the file: {e}"))?;
                                let mut st = app_state().lock().unwrap();
                                st.require_server_role("Import from CSV")?;
                                let store =
                                    st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                                let (added, skipped, problems) = if parties {
                                    store.import_parties_csv(&text)?
                                } else {
                                    store.import_items_csv(&text)?
                                };
                                st.reload_all()?;
                                let mut msg = format!(
                                    "Imported {added} new, skipped {skipped} (already there or unusable)"
                                );
                                if let Some(p) = problems.first() {
                                    msg.push_str(&format!(" · first problem: {p}"));
                                }
                                Ok(msg)
                            })();
                            match outcome {
                                Ok(msg) => self.set_status(cx, msg),
                                Err(e) => self.set_status(cx, e),
                            }
                            self.sync_chrome(cx);
                        }
                    }
                    FileDialogAction::FileCancelled { id }
                        if *id == live_id!(item_photo_pick)
                            || *id == live_id!(venue_icon_pick)
                            || *id == live_id!(onboard_icon_pick)
                            || *id == live_id!(csv_parties_pick)
                            || *id == live_id!(csv_items_pick)
                            || *id == live_id!(entry_attach_pick) =>
                    {
                        app_state().lock().unwrap().attach_pick = None;
                        self.set_status(cx, "Picker cancelled");
                    }
                    _ => {}
                }
            }
        }
        // Nav clicks honour the user's slot arrangement.
        let mut nav_click = self.nav_tab_clicked(cx, actions);
        if self.ui.button(cx, ids!(day_close_from_sales)).clicked(actions) {
            nav_click = Some(NavTab::DayClose);
        }
        match nav_click {
            Some(NavTab::Books) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.refresh_books_overview()?;
                    st.screen = Screen::Books;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                // Drop leftover focus so the old Sell face cannot look selected.
                cx.set_key_focus(Area::Empty);
                self.sync_chrome(cx);
            }
            Some(NavTab::Sell) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Pos, |s| s.can_access_sell());
            }
            Some(NavTab::Buy) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Buy, |s| {
                    s.can_access_inventory() || s.can_access_sales() || s.is_owner()
                });
            }
            Some(NavTab::Menu) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Menu, |s| s.can_access_items());
            }
            Some(NavTab::Floor) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Floor, |s| s.can_access_sell());
            }
            Some(NavTab::Kitchen) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Kitchen, |s| {
                    s.can_access_sell() || s.is_owner()
                });
            }
            Some(NavTab::Sales) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Sales, |s| s.can_access_sales());
            }
            Some(NavTab::Credit) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Credit, |s| s.can_access_credit());
            }
            Some(NavTab::Team) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Team, |s| s.can_access_team() || s.is_owner());
            }
            Some(NavTab::Settings) => {
                cx.set_key_focus(Area::Empty);
                self.go_if_allowed(cx, Screen::Settings, |s| {
                    s.can_access_backup() || s.is_owner()
                });
            }
            Some(NavTab::DayClose) => {
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
            Some(NavTab::Reports) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_sales() {
                        return Err("Not allowed".into());
                    }
                    st.refresh_reports_facet()?;
                    st.screen = Screen::Reports;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            Some(NavTab::Inventory) => {
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
            Some(NavTab::Branches) => {
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
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            Some(NavTab::Lock) => {
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
            None => {}
        }
        if self.ui.button(cx, ids!(books_to_sell)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Pos, |s| s.can_access_sell());
        }
        if self.ui.button(cx, ids!(books_to_buy)).clicked(actions) {
            self.go_if_allowed(cx, Screen::Buy, |s| {
                s.can_access_inventory() || s.can_access_sales() || s.is_owner()
            });
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
                st.refresh_reports_facet()?;
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
            let part2_text = self.ui.text_input(cx, ids!(buy_exp_part2)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.device_role == DeviceRole::Client {
                    return Err("Log expenses on the shop device — this is an extra till".into());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                let parts = Self::expense_parts(minor, &part2_text, &currency, st.expense_tender, st.expense_tender2)?;
                let expense_id = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_expense_tenders(minor, "misc", &note, &actor, &parts)?
                };
                st.selected_expense_id = Some(expense_id);
                let _ = st.refresh_books_overview();
                st.status = "Expense logged".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(buy_expense_amount)).set_text(cx, "");
                self.ui.text_input(cx, ids!(buy_expense_note)).set_text(cx, "");
                self.ui.text_input(cx, ids!(buy_exp_part2)).set_text(cx, "");
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
        if self.ui.button(cx, ids!(client_pair_back)).clicked(actions) {
            let _ = self.navigate_back(cx);
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
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(device_role_server),
            ids!(device_role_server_on),
        ) {
            self.begin_server_mode(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(device_role_client),
            ids!(device_role_client_on),
        ) {
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
                st.status = format!("Billed {} · {summary}", sale.invoice_no);
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
                    st.status = format!("Billed {} · {summary}", sale.invoice_no);
                    let snack = format!("{amount} · {summary}");
                    st.reload_all()?;
                    Ok((sale.id, snack))
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
        for ((chip, chip_on), kind) in [
            (
                (ids!(credit_kind_customer), ids!(credit_kind_customer_on)),
                "Customer",
            ),
            (
                (ids!(credit_kind_both), ids!(credit_kind_both_on)),
                "Both",
            ),
            (
                (ids!(credit_kind_supplier), ids!(credit_kind_supplier_on)),
                "Supplier",
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                self.ui
                    .text_input(cx, ids!(credit_add_kind))
                    .set_text(cx, kind);
                self.set_status(
                    cx,
                    match kind {
                        "Customer" => "Role: Customer — khata (they owe us). Same party can also supply stock.",
                        "Both" => "Roles: Customer + Supplier — one party, khata and payable ledgers, net in Reports.",
                        _ => "Role: Supplier — we owe them for purchases.",
                    },
                );
            }
        }
        if self.ui.button(cx, ids!(credit_add_btn)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(credit_add_name)).text();
            let phone = self.ui.text_input(cx, ids!(credit_add_phone)).text();
            let kind = self.ui.text_input(cx, ids!(credit_add_kind)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                if let Some(id) = st.selected_customer_id.clone() {
                    st.require_server_role("Edit party")?;
                    store.update_customer(&id, &name, &phone)?;
                    store.set_customer_kind_label(&id, kind.trim())?;
                    st.reload_all()?;
                    Ok(format!("Party updated: {}", name.trim()))
                } else {
                    st.require_server_role("Add customers")?;
                    let customer = store.add_customer_kind(&name, &phone, kind.trim())?;
                    st.selected_customer_id = Some(customer.id);
                    st.reload_all()?;
                    Ok(format!(
                        "Added {} — now editing it; tap New party to add another",
                        name.trim()
                    ))
                }
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_form_clear)).clicked(actions) {
            app_state().lock().unwrap().selected_customer_id = None;
            self.ui
                .text_input(cx, ids!(credit_add_name))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(credit_add_phone))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(credit_add_kind))
                .set_text(cx, "");
            self.set_status(cx, "New party — form cleared");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_delete)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Delete party")?;
                let id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let msg = store.delete_customer(&id)?;
                st.selected_customer_id = None;
                st.reload_all()?;
                Ok(msg)
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(credit_add_name)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(credit_add_phone)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(credit_add_kind)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(credit_limit_set)).clicked(actions) {
            let limit_text = self.ui.text_input(cx, ids!(credit_limit_input)).text();
            let named = self.ui.text_input(cx, ids!(credit_limit_party)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Set credit limit")?;
                let currency = st.locale.currency.clone();
                let limit_paise = if limit_text.trim().is_empty() {
                    0
                } else {
                    locale::parse_major_input(&limit_text, &currency)?
                };
                let party_name = if !named.trim().is_empty() {
                    named
                } else {
                    let id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a party or type a name".to_string())?;
                    st.customers
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.name.clone())
                        .ok_or_else(|| "Party not found".to_string())?
                };
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let customer = store.find_or_create_customer(party_name.trim())?;
                store.set_customer_credit_limit(&customer.id, limit_paise)?;
                st.reload_all()?;
                Ok(format!(
                    "Credit limit for {} set to {}",
                    customer.name,
                    st.money(Money(limit_paise))
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(credit_limit_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
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
                let bill_text = self.ui.text_input(cx, ids!(payable_bill_input)).text();
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
                    // Optional bill-wise allocation (bill # from the list).
                    let against = match bill_text.trim() {
                        "" => None,
                        other => {
                            let n: usize = other
                                .parse()
                                .map_err(|_| "Bill # must be a number from the list".to_string())?;
                            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            let bills = store.open_payable_bills(&party_id)?;
                            Some(
                                bills
                                    .get(n.wrapping_sub(1))
                                    .map(|b| b.bill_id.clone())
                                    .ok_or_else(|| "No such open bill".to_string())?,
                            )
                        }
                    };
                    st.store
                        .as_mut()
                        .ok_or_else(|| "No store".to_string())?
                        .settle_payable_against(
                            &party_id,
                            amount,
                            method,
                            &actor,
                            "",
                            against.as_deref(),
                        )?;
                    st.reload_all()?;
                    let _ = st.refresh_books_overview();
                    Ok(format!("AP paid {}", st.money(Money(amount))))
                })();
                match result {
                    Ok(msg) => {
                        self.ui
                            .text_input(cx, ids!(payable_amount_input))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(payable_bill_input))
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
                    let picked = {
                        let mut st = app_state().lock().unwrap();
                        let picked = st.customers.get(item_id).cloned();
                        if let Some(c) = &picked {
                            st.selected_customer_id = Some(c.id.clone());
                            st.status = format!("Credit customer: {}", c.name);
                        }
                        picked
                    };
                    if let Some(c) = picked {
                        self.ui
                            .text_input(cx, ids!(credit_add_name))
                            .set_text(cx, &c.name);
                        self.ui
                            .text_input(cx, ids!(credit_add_phone))
                            .set_text(cx, &c.phone);
                        self.ui
                            .text_input(cx, ids!(credit_add_kind))
                            .set_text(cx, &c.kind_label);
                    }
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
                let bill_text = self.ui.text_input(cx, ids!(settle_bill_input)).text();
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
                    // Optional bill-wise allocation: a bill # from the
                    // open-bills list under the party.
                    let against = match bill_text.trim() {
                        "" => None,
                        other => {
                            let n: usize = other
                                .parse()
                                .map_err(|_| "Bill # must be a number from the list".to_string())?;
                            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            let bills = store.open_credit_bills(&customer_id)?;
                            Some(
                                bills
                                    .get(n.wrapping_sub(1))
                                    .map(|b| b.bill_id.clone())
                                    .ok_or_else(|| "No such open bill".to_string())?,
                            )
                        }
                    };
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    store.settle_credit_against(
                        &customer_id,
                        minor,
                        method,
                        &actor,
                        "",
                        against.as_deref(),
                    )?;
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
                    self.ui
                        .text_input(cx, ids!(settle_bill_input))
                        .set_text(cx, "");
                    // Surface settle/advance result on Sell receipt line too.
                    let msg = app_state().lock().unwrap().status.clone();
                    self.set_status(cx, msg);
                }
                self.sync_chrome(cx);
            }
        }

        // Order type chips: Counter / Dine-in / Takeaway / Delivery.
        for ((chip, chip_on), kind) in [
            ((ids!(otype_counter), ids!(otype_counter_on)), OrderType::Counter),
            ((ids!(otype_dinein), ids!(otype_dinein_on)), OrderType::DineIn),
            ((ids!(otype_takeaway), ids!(otype_takeaway_on)), OrderType::Takeaway),
            ((ids!(otype_delivery), ids!(otype_delivery_on)), OrderType::Delivery),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                let mut st = app_state().lock().unwrap();
                st.cart.set_order_type(kind);
                if kind != OrderType::DineIn {
                    st.selected_table_id = None;
                }
                drop(st);
                self.sync_chrome(cx);
            }
        }
        // Expense tender chips: first tender and the optional second way.
        for ((chip, chip_on), m) in [
            ((ids!(exp_t1_cash), ids!(exp_t1_cash_on)), PayMethod::Cash),
            ((ids!(exp_t1_bank), ids!(exp_t1_bank_on)), PayMethod::Bank),
            ((ids!(exp_t1_upi), ids!(exp_t1_upi_on)), PayMethod::Upi),
            ((ids!(exp_t1_card), ids!(exp_t1_card_on)), PayMethod::Card),
            ((ids!(buy_exp_t1_cash), ids!(buy_exp_t1_cash_on)), PayMethod::Cash),
            ((ids!(buy_exp_t1_bank), ids!(buy_exp_t1_bank_on)), PayMethod::Bank),
            ((ids!(buy_exp_t1_upi), ids!(buy_exp_t1_upi_on)), PayMethod::Upi),
            ((ids!(buy_exp_t1_card), ids!(buy_exp_t1_card_on)), PayMethod::Card),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                app_state().lock().unwrap().expense_tender = m;
                self.sync_chrome(cx);
            }
        }
        for ((chip, chip_on), m) in [
            ((ids!(exp_t2_cash), ids!(exp_t2_cash_on)), PayMethod::Cash),
            ((ids!(exp_t2_bank), ids!(exp_t2_bank_on)), PayMethod::Bank),
            ((ids!(exp_t2_upi), ids!(exp_t2_upi_on)), PayMethod::Upi),
            ((ids!(exp_t2_card), ids!(exp_t2_card_on)), PayMethod::Card),
            ((ids!(buy_exp_t2_cash), ids!(buy_exp_t2_cash_on)), PayMethod::Cash),
            ((ids!(buy_exp_t2_bank), ids!(buy_exp_t2_bank_on)), PayMethod::Bank),
            ((ids!(buy_exp_t2_upi), ids!(buy_exp_t2_upi_on)), PayMethod::Upi),
            ((ids!(buy_exp_t2_card), ids!(buy_exp_t2_card_on)), PayMethod::Card),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                app_state().lock().unwrap().expense_tender2 = m;
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(clear_cart)).clicked(actions)
            || self.ui.button(cx, ids!(next_order_btn)).clicked(actions)
        {
            {
                let mut st = app_state().lock().unwrap();
                st.clear_order_desk();
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(add_items_btn)).clicked(actions) {
            self.go(cx, Screen::Menu);
        }
        // Order type chips on the desk.
        for ((chip, chip_on), kind) in [
            ((ids!(otype_counter), ids!(otype_counter_on)), OrderType::Counter),
            ((ids!(otype_dinein), ids!(otype_dinein_on)), OrderType::DineIn),
            ((ids!(otype_takeaway), ids!(otype_takeaway_on)), OrderType::Takeaway),
            ((ids!(otype_delivery), ids!(otype_delivery_on)), OrderType::Delivery),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                // Guard must drop before sync_chrome: it locks app_state()
                // itself, and std Mutex is non-reentrant — calling it with
                // `st` alive deadlocks the UI thread (found 2026-10-22:
                // order-type chip tap froze the app after unlock).
                {
                    let mut st = app_state().lock().unwrap();
                    st.cart.set_order_type(kind);
                    if kind != OrderType::DineIn {
                        st.selected_table_id = None;
                    }
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(send_kot_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.cart.is_empty() {
                    return Err("Order is empty — tap items first".into());
                }
                let draft = st.cart.to_sale_draft()?;
                let table_id = st.selected_table_id.clone();
                let kot_id = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.send_kot_from_cart(&draft, table_id.as_deref(), &actor)?
                };
                // The kitchen's paper appears at the counter by itself.
                let msg = if st.device_role.is_host()
                    && st
                        .store
                        .as_ref()
                        .map(|s| s.kot_auto_print())
                        .unwrap_or(false)
                {
                    match st.store.as_ref().unwrap().print_kot(&kot_id) {
                        Ok(_) => "KOT sent to kitchen · printed".to_string(),
                        Err(e) => format!("KOT sent to kitchen · print failed: {e}"),
                    }
                } else {
                    "KOT sent to kitchen".to_string()
                };
                st.cart.clear();
                st.open_kots = st
                    .store
                    .as_ref()
                    .map(|s| s.list_open_kots().unwrap_or_default())
                    .unwrap_or_default();
                st.tables = st
                    .store
                    .as_ref()
                    .map(|s| s.list_tables().unwrap_or_default())
                    .unwrap_or_default();
                Ok(msg)
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        // Floor: tap anywhere on a table card to start / continue its order.
        // Card-tap (not just the button) mirrors MenuList's card tap: inside a
        // PortalList item the plain Button::clicked path is not reliable.
        {
            let widget = self.ui.widget(cx, ids!(floor_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                // Request bill first: it sits inside the card, and the
                // card tap must not also open the order behind it.
                let bill_tapped = item.button(cx, ids!(bill_req)).clicked(actions);
                if bill_tapped {
                    let result = (|| -> Result<String, String> {
                        // Plan under lock; the LAN call itself holds no lock.
                        let (actor, table, remote) = {
                            let st = app_state().lock().unwrap();
                            let actor = st.actor()?;
                            let table = st
                                .tables
                                .get(item_id)
                                .cloned()
                                .ok_or_else(|| "Table gone".to_string())?;
                            let remote = if st.device_role == DeviceRole::Client {
                                Some(st.server_link()?)
                            } else {
                                None
                            };
                            (actor, table, remote)
                        };
                        if let Some((url, code)) = remote {
                            // Waiter's device: the Server owns table state.
                            net::client_push_table_billing(
                                &url,
                                &code,
                                &net::RemoteTableBillingRequest {
                                    actor_id: actor.id.clone(),
                                    table_id: Some(table.id.clone()),
                                    table_name: Some(table.name.clone()),
                                },
                            )?;
                            let mut st = app_state().lock().unwrap();
                            if let Some(store) = st.store.as_ref()
                                && let Ok(snap) = net::client_sync(&url, &code)
                            {
                                let _ = net::apply_snapshot(store, &snap);
                                st.tables = store.list_tables().unwrap_or_default();
                            }
                        } else {
                            let mut st = app_state().lock().unwrap();
                            let store =
                                st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            store.request_billing(&table.id, &actor)?;
                            st.tables = store.list_tables().unwrap_or_default();
                        }
                        Ok(format!(
                            "{} — payment pending. Collect at the counter; \
                             the table releases when the payment is recorded",
                            table.name
                        ))
                    })();
                    match result {
                        Ok(msg) => self.set_status(cx, msg),
                        Err(e) => self.set_status(cx, e),
                    }
                    self.sync_chrome(cx);
                    continue;
                }
                let tapped = item.button(cx, ids!(open)).clicked(actions)
                    || item
                        .view(cx, ids!(floorcard))
                        .finger_up(actions)
                        .is_some_and(|e| e.was_tap() && e.is_over);
                if tapped {
                    let picked = {
                        let mut st = app_state().lock().unwrap();
                        let picked = st.tables.get(item_id).cloned();
                        if let Some(t) = &picked {
                            st.selected_table_id = Some(t.id.clone());
                            st.cart.set_order_type(OrderType::DineIn);
                            st.status = format!("{} — tap items, then Send KOT", t.name);
                        }
                        picked.is_some()
                    };
                    if picked {
                        self.go(cx, Screen::Pos);
                    }
                    self.sync_chrome(cx);
                }
            }
        }
        // Kitchen: advance a ticket New → Preparing → Ready → Done.
        {
            let widget = self.ui.widget(cx, ids!(kitchen_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                let tapped = item.button(cx, ids!(advance)).clicked(actions)
                    || item
                        .view(cx, ids!(kotcard))
                        .finger_up(actions)
                        .is_some_and(|e| e.was_tap() && e.is_over);
                if tapped {
                    let result = (|| -> Result<(), String> {
                        let mut st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        let kot_id = st
                            .open_kots
                            .get(item_id)
                            .map(|k| k.id.clone())
                            .ok_or_else(|| "Ticket gone".to_string())?;
                        {
                            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            store.advance_kot(&kot_id, &actor)?;
                        }
                        st.open_kots = st
                            .store
                            .as_ref()
                            .map(|s| s.list_open_kots().unwrap_or_default())
                            .unwrap_or_default();
                        Ok(())
                    })();
                    if let Err(e) = result {
                        self.set_status(cx, e);
                    }
                    self.sync_chrome(cx);
                }
            }
        }
        if self.ui.button(cx, ids!(table_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(table_name_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add tables")?;
                let table = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_table(&name)?
                };
                st.tables = st
                    .store
                    .as_ref()
                    .map(|s| s.list_tables().unwrap_or_default())
                    .unwrap_or_default();
                Ok(format!("Table {} added", table.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(table_name_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(station_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(station_name_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add kitchen stations")?;
                let station = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_station(&name)?
                };
                st.stations = st
                    .store
                    .as_ref()
                    .map(|s| s.list_stations().unwrap_or_default())
                    .unwrap_or_default();
                Ok(format!("Station {} added", station.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(station_name_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        // Live initials on the draft thumb while typing an item name.
        if self
            .ui
            .text_input(cx, ids!(menu_name_input))
            .changed(actions)
            .is_some()
        {
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
        if self.ui.button(cx, ids!(image_clear)).clicked(actions) {
            self.image_clear_photo(cx);
        }
        if self.ui.button(cx, ids!(image_pick)).clicked(actions) {
            self.image_pick_file(cx);
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

        // Provided-items shelf: Show loads a folder's dishes.
        {
            let widget = self.ui.widget(cx, ids!(catalogue_cat_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let picked = {
                        let mut st = app_state().lock().unwrap();
                        if let Some(cat) = st.catalogue_cats.get(item_id).cloned() {
                            st.catalogue_cat = cat.key.clone();
                            st.catalogue_items =
                                menu_images::catalogue_items(&cat.key);
                            true
                        } else {
                            false
                        }
                    };
                    if picked {
                        self.sync_chrome(cx);
                    }
                }
            }
        }
        // Use on a dish: name + photo fill the item form above; the
        // fresh UUID is minted on Add, and the category stays the
        // owner's own choice in the form.
        {
            let widget = self.ui.widget(cx, ids!(catalogue_item_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let dish = {
                        let st = app_state().lock().unwrap();
                        st.catalogue_items.get(item_id).cloned()
                    };
                    if let Some(dish) = dish {
                        self.ui
                            .text_input(cx, ids!(menu_name_input))
                            .set_text(cx, &dish.name);
                        {
                            let mut st = app_state().lock().unwrap();
                            st.draft_image_key =
                                Some(format!("pending_custom:{}", dish.path.display()));
                            st.editing_product_id = None;
                        }
                        self.ui
                            .view(cx, ids!(menu_screen))
                            .set_scroll_pos(cx, dvec2(0.0, 0.0));
                        self.set_status(
                            cx,
                            format!(
                                "{} picked — set your price and category, then Add item",
                                dish.name
                            ),
                        );
                        self.sync_chrome(cx);
                    }
                }
            }
        }
        // Your categories: Use files the form's item under it; Photo
        // gives the category a picture (initials until then).
        {
            let widget = self.ui.widget(cx, ids!(category_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let cat = {
                        let st = app_state().lock().unwrap();
                        st.categories.get(item_id).cloned()
                    };
                    if let Some(cat) = cat {
                        self.ui
                            .text_input(cx, ids!(menu_category_input))
                            .set_text(cx, &cat.name);
                        {
                            // Identity travels by UUID, not by the typed name.
                            let mut st = app_state().lock().unwrap();
                            st.form_category_id = Some(cat.id.clone());
                            st.form_category_label = cat.name.clone();
                        }
                        self.set_status(
                            cx,
                            format!("Category set to {} — Add or save the item to file it there", cat.name),
                        );
                        self.sync_chrome(cx);
                    }
                }
                if item.button(cx, ids!(photo)).clicked(actions) {
                    let cat = {
                        let st = app_state().lock().unwrap();
                        st.categories.get(item_id).cloned()
                    };
                    if let Some(cat) = cat {
                        app_state().lock().unwrap().category_photo_id = Some(cat.id);
                        let dialog = FileDialog::new()
                            .set_title("Choose category photo".into())
                            .add_filter(
                                "Images".into(),
                                vec![
                                    "webp".into(),
                                    "png".into(),
                                    "jpg".into(),
                                    "jpeg".into(),
                                ],
                            )
                            .set_id(live_id!(category_photo_pick));
                        cx.open_select_file_dialog(dialog);
                        self.set_status(cx, "Opening photo picker…");
                    }
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
        if self.choice_chip_clicked(cx, actions, ids!(staff_role_manager), ids!(staff_role_manager_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Manager;
                st.draft_features = StaffFeatures::for_role(StaffRole::Manager);
            }
            self.set_status(cx, "Preset: Manager");
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
                st.status = format!("{name} pack applied — check taxes on Menu items");
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
                    "Book selected — add opening items / credit below, then Done — open books".into();
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
        {
            let today = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_today),
                ids!(reports_today_on),
            );
            let month = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_month),
                ids!(reports_month_on),
            );
            let aging = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_aging),
                ids!(reports_aging_on),
            );
            let bs = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_balance_sheet),
                ids!(reports_balance_sheet_on),
            );
            if today || month || aging || bs {
                let kind = if aging {
                    Some(BooksReportKind::Aging)
                } else if bs {
                    Some(BooksReportKind::BalanceSheet)
                } else if month {
                    Some(BooksReportKind::Month)
                } else if today {
                    Some(BooksReportKind::Today)
                } else {
                    None
                };
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    if let Some(k) = kind {
                        st.books_report_kind = k;
                    }
                    st.load_books_report()?;
                    st.screen = Screen::Reports;
                    st.status = match st.books_report_kind {
                        BooksReportKind::Aging => "Outstanding / debit-credit".into(),
                        BooksReportKind::BalanceSheet => "Balance sheet".into(),
                        BooksReportKind::Month => "This month books".into(),
                        BooksReportKind::Today => "Today books".into(),
                    };
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
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
            let part2_text = self.ui.text_input(cx, ids!(exp_part2)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.device_role == DeviceRole::Client {
                    return Err("Log expenses on the shop device — this is an extra till".into());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                let parts = Self::expense_parts(minor, &part2_text, &currency, st.expense_tender, st.expense_tender2)?;
                let expense_id = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_expense_tenders(minor, "misc", &note, &actor, &parts)?
                };
                st.selected_expense_id = Some(expense_id);
                st.day_close_text = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .books_report_text("today")?;
                st.screen = Screen::Reports;
                st.status = "Expense logged".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(expense_amount_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(expense_note_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(exp_part2)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(sale_attach_add_note)).clicked(actions) {
            self.add_entry_note(cx, AttachmentEntity::Sale, ids!(sale_attach_note));
        }
        if self.ui.button(cx, ids!(sale_attach_photo)).clicked(actions) {
            self.attach_pick_image(cx, AttachmentEntity::Sale);
        }
        if self.ui.button(cx, ids!(sale_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Sale);
        }
        if self.ui.button(cx, ids!(party_attach_add_note)).clicked(actions) {
            self.add_entry_note(cx, AttachmentEntity::Party, ids!(party_attach_note));
        }
        if self.ui.button(cx, ids!(party_attach_photo)).clicked(actions) {
            self.attach_pick_image(cx, AttachmentEntity::Party);
        }
        if self.ui.button(cx, ids!(party_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Party);
        }
        if self.ui.button(cx, ids!(expense_attach_add_note)).clicked(actions) {
            self.add_entry_note(cx, AttachmentEntity::Expense, ids!(expense_attach_note));
        }
        if self.ui.button(cx, ids!(expense_attach_photo)).clicked(actions) {
            self.attach_pick_image(cx, AttachmentEntity::Expense);
        }
        if self.ui.button(cx, ids!(expense_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Expense);
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

        // Bank reconciliation tick-off: # refers to the numbered
        // uncleared list under Bank / cashbook.
        for (btn, cleared) in [
            (ids!(bank_mark_cleared), true),
            (ids!(bank_mark_uncleared), false),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let num_text = self.ui.text_input(cx, ids!(bank_clear_input)).text();
                let result = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    st.require_server_role("Reconcile bank movements")?;
                    let actor = st.actor()?;
                    let n: usize = num_text
                        .trim()
                        .parse()
                        .map_err(|_| "Type the uncleared # from the list".to_string())?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let uncleared: Vec<String> = store
                        .list_money_movements(50)?
                        .into_iter()
                        .filter(|m| !m.cleared)
                        .map(|m| m.id)
                        .collect();
                    let id = uncleared
                        .get(n.wrapping_sub(1))
                        .cloned()
                        .ok_or_else(|| "No such uncleared movement".to_string())?;
                    store.set_money_movement_cleared(&id, cleared, &actor)?;
                    Ok(if cleared {
                        "Marked cleared — seen on the statement".into()
                    } else {
                        "Unmarked".into()
                    })
                })();
                match result {
                    Ok(msg) => {
                        self.ui.text_input(cx, ids!(bank_clear_input)).set_text(cx, "");
                        self.set_status(cx, msg);
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }

        // Spreadsheet (CSV): copy out, or bring parties / items in.
        for (btn, which) in [
            (ids!(csv_copy_day), "day"),
            (ids!(csv_copy_outstanding), "outstanding"),
            (ids!(csv_copy_parties), "parties"),
            (ids!(csv_copy_items), "items"),
            (ids!(csv_copy_ledger), "ledger"),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let text = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    match which {
                        "day" => store.day_book_csv(),
                        "outstanding" => store.outstanding_csv(),
                        "parties" => store.parties_csv(),
                        "items" => store.items_csv(),
                        _ => {
                            let id = st
                                .selected_customer_id
                                .clone()
                                .ok_or_else(|| "Pick a party first (Parties tab)".to_string())?;
                            store.party_ledger_csv(&id)
                        }
                    }
                })();
                match text {
                    Ok(text) => {
                        self.copy_text_to_clipboard(cx, &text);
                        self.set_status(cx, "CSV copied — paste into your spreadsheet");
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
        }
        if self.ui.button(cx, ids!(csv_import_parties)).clicked(actions) {
            let dialog = FileDialog::new()
                .set_title("Choose parties CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_parties_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
        }
        if self.ui.button(cx, ids!(csv_import_items)).clicked(actions) {
            let dialog = FileDialog::new()
                .set_title("Choose items CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_items_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
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
                st.status = match store.resolve_unit(&from) {
                    Some((family, factor)) => {
                        let anchor = domain::unit_family_anchor(&family).unwrap_or("");
                        format!(
                            "Added — now 1 {from} = {} {anchor} ({family})",
                            precise::format_kg(factor)
                        )
                    }
                    None => format!(
                        "1 {from} → {to} added — '{from}' is not linked to a standard unit yet"
                    ),
                };
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
        if self.ui.button(cx, ids!(onboard_icon_pick)).clicked(actions) {
            self.pick_onboard_icon(cx);
        }

        if self.ui.button(cx, ids!(kot_print_toggle)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let next = !store.kot_auto_print();
                store.set_kot_auto_print(next)?;
                Ok(if next {
                    "Kitchen printing on — KOTs print as they are sent".to_string()
                } else {
                    "Kitchen printing off — KOTs stay on the Kitchen screen".to_string()
                })
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
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
            let unlocked = {
                let st = app_state().lock().unwrap();
                !matches!(
                    st.screen,
                    Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                )
            };
            self.apply_shell_layout(cx, unlocked);
            if !restoring {
                self.persist_window_geom(cx);
            }
        }
        self.match_event(cx, event);
        if let Event::NextFrame(e) = event {
            let ours = nav_autoscroll_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if ours {
                self.nav_drag_autoscroll(cx);
            }
        }
        if self.pointer_nav_drag(cx, event) {
            return;
        }
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

#[cfg(test)]
mod nav_order_tests {
    use super::*;

    #[test]
    fn saved_order_wins_and_missing_tabs_append() {
        let order = parse_nav_order(Some("credit,books,bogus,credit"));
        assert_eq!(order.len(), NAV_TABS.len());
        assert_eq!(order[0], NavTab::Credit);
        assert_eq!(order[1], NavTab::Books);
        // Every tab appears exactly once.
        for tab in NAV_TABS {
            assert_eq!(order.iter().filter(|t| **t == tab).count(), 1);
        }
        // Round-trip through CSV is stable.
        assert_eq!(parse_nav_order(Some(&nav_order_csv(&order))), order);
    }

    #[test]
    fn no_saved_order_is_default() {
        assert_eq!(parse_nav_order(None), NAV_TABS.to_vec());
        assert_eq!(parse_nav_order(Some("")), NAV_TABS.to_vec());
    }
}

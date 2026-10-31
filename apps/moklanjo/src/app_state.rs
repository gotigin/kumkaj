struct AppState {
    store: Option<Store>,
    products: Vec<Product>,
    cart: Cart,
    /// Editable copy of the selected sale for owner/manager amend.
    amend_cart: Cart,
    amend_pay: PayMethod,
    sale_revisions: Vec<SaleRevision>,
    selected_customer_id: Option<String>,
    customers: Vec<Customer>,
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
    /// Office servers "Find" saw on this Wi‑Fi.
    found_servers: Vec<net::FoundServer>,
    /// Devices paired to this Server (Settings → Extra tills).
    lan_devices: Vec<store::LanDevice>,
    /// App data root (desktop `data_dir` or Android files dir).
    data_base: Option<PathBuf>,
    /// Settings → Printers: pool + routes being edited.
    printer_draft: print_share::DevicePrinters,
    /// Focused route: `receipt` or `label`.
    printer_route_key: String,
    /// Shop device: extra tills may pair on LAN. Off for a new shop.
    extra_tills_allowed: bool,
    /// False until Owner finishes first-run PIN setup (Server shop).
    owner_setup_complete: bool,
    /// False until Fresh/Book onboarding finishes.
    onboarding_complete: bool,
    client_paired: bool,
    /// Book-import wizard step: 0=choose, 1=items hint, 2=credit opening.
    onboard_step: u8,
    /// Active start template id (`custom`, `moklanjo_agency`, …).
    entity_template_id: String,
    /// Bill type for the open cart (Account booking / Tax invoice vs Cash booking / Cash memo).
    money_lane: MoneyLane,
    /// Buy tab: purchase posts to AP when true.
    buy_on_account: bool,
    /// Buy tab: kind for a new party when recording on account (Supplier / Carrier).
    buy_party_kind: String,
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
    snack_text: String,
    snack_kind: SnackKind,
    snack_timer: Timer,
    /// Refreshes Settings' paired devices ("connected now") while it shows.
    live_timer: Timer,
    pin_buffer: String,
    /// When true, lock screen shows digits instead of bullets.
    pin_visible: bool,
    selected_sale_id: Option<String>,
    /// Last logged expense (for attaching bill photo/PDF).
    selected_expense_id: Option<String>,
    /// Pending file-dialog target for entry attachments.
    attach_pick: Option<AttachPickTarget>,
    /// Ops desk: POD photo waiting for Out-for-delivery / Delivered.
    pending_ops_pod: Option<PathBuf>,
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
    /// Chrome peers hidden from rail / bottom bar (Settings + Lock stay shown).
    nav_hidden: Vec<NavTab>,
    /// Top eye for the signed-in user (persisted, applies on every screen).
    /// Hidden → mask all money values except [`amount_keys_peek`].
    /// Visible → apply each figure’s [`amount_keys_hidden`].
    amounts_screen_hidden: bool,
    /// Keys this user hid while the screen eye is showing (persisted per user).
    amount_keys_hidden: Vec<String>,
    /// Keys this user keeps visible while the screen eye is hiding (persisted
    /// per user). Same list across screens for that user.
    amount_keys_peek: Vec<String>,
    /// Books home accordion: which section bodies are open (`today`, `bookings`, …).
    books_sections_open: Vec<String>,
    /// DeviceSetup accordions (`language`, `role`). Default opens `role`.
    device_setup_sections_open: Vec<String>,
    /// Pending custom icon path from picker (before Save brand).
    draft_venue_icon_path: Option<PathBuf>,
    /// Android: a fresh backup waiting for "Save a backup copy" to be told
    /// where it goes.
    pending_backup_copy: Option<PathBuf>,
    /// Diagnostics single-file bundle waiting for Save (all platforms).
    pending_diag_export: Option<PathBuf>,
    /// None = auto-detect from name; Some(key) = built-in or custom:…
    draft_image_key: Option<String>,
    /// Pending party logo (`pending_custom:…` / `custom:…`) before Add/Save.
    draft_party_image_key: Option<String>,
    /// Country typed/picked on the party address form (drives state suggestions).
    party_form_country: String,
    /// Desk scope: false = Domestic (shop divisor, usually ÷5000);
    /// true = International (÷6000 air). Does not rewrite Settings.
    booking_scope_international: bool,
    /// Booking desk: consignor (= customer) + consignee address blocks
    /// (always on the form; values may stay empty) + destination / weights.
    booking_origin_name: String,
    booking_origin_phone: String,
    booking_origin_address: String,
    booking_destination: String,
    booking_consignee: String,
    booking_consignee_phone: String,
    booking_consignee_address: String,
    booking_actual_kg: String,
    booking_length_cm: String,
    booking_breadth_cm: String,
    booking_height_cm: String,
    booking_cod: String,
    booking_carrier_cost: String,
    booking_declared: String,
    booking_contents: String,
    booking_eway: String,
    /// Booking desk: parcel is insured (SegBtn Yes/No — not free text).
    booking_insurance: bool,
    booking_notify_phone: String,
    booking_fuel: String,
    booking_docket: String,
    booking_zone: String,
    /// Cargo / multi-piece count (blank → 1).
    booking_pieces: String,
    /// Carrier party name/id for AWB pool + cost compare.
    booking_carrier_pick: String,
    /// Last carrier-bill reconcile report (Reports → copy).
    last_carrier_bill_report: String,
    /// Last carrier compare rows from the booking desk (for "Use carrier cost").
    last_carrier_compare: Vec<rates::CarrierCompareRow>,
    /// Booking-desk measurement choices: the units the user types in, the
    /// volumetric divisor and the weight slab. Books always record kg/cm;
    /// these only adapt entry and display (see BookingMeasureSettings).
    measure_dim_unit: String,
    measure_weight_unit: String,
    measure_divisor: rust_decimal::Decimal,
    measure_slab_kg: rust_decimal::Decimal,
    /// Resolved factor: 1 `measure_dim_unit` = this many cm.
    measure_dim_factor: rust_decimal::Decimal,
    /// Resolved factor: 1 `measure_weight_unit` = this many kg.
    measure_weight_factor: rust_decimal::Decimal,
    /// Expense forms: how the expense was paid — first tender (gets the
    /// remainder) and the optional second tender for a split payment.
    expense_tender: PayMethod,
    expense_tender2: PayMethod,
    /// Last AWB after a successful book — shown until Next booking.
    last_awb: String,
    /// Catalogue id loaded into the Items form. None means the form will add.
    editing_product_id: Option<String>,
    /// True from the moment a charge starts until it succeeds or fails. A
    /// double-tapped Cash button used to bill the cart twice.
    pay_in_flight: bool,
    /// A device's PIN is with the Server; a second OK waits for the answer.
    lan_sign_in_busy: bool,
    /// What the last pairing try said, on the pairing screen.
    pair_note: String,
    /// Consecutive wrong PINs on this process. Reset by any success.
    failed_unlocks: u32,
    /// While set, unlock is refused — brute-forcing a short PIN should not be
    /// a matter of seconds. In memory only: the till is local, and a restart
    /// costing an attacker the shop's own downtime is fine.
    unlock_blocked_until: Option<std::time::Instant>,
    /// Owner must choose a new PIN before the till opens (post-migration reset).
    pin_reset_required: bool,
    /// Drill-down history (Settings → SidebarSettings, …). Chrome peers use
    /// `go_peer` and clear these; Back / mouse / Android pop via `navigate_back`.
    nav_back: Vec<Screen>,
    nav_forward: Vec<Screen>,
    /// User-resized side rail width (css px). Clamped in `apply_shell_layout`.
    rail_width: f64,
    /// Tablet/desktop: side rail hidden (zero width). Bottom nav unchanged.
    rail_collapsed: bool,
    /// Which Reports facet is showing (bank / consignments / COD / riders).
    reports_nav: ReportsNav,
    /// Which bank-books report chip is selected (must match `day_close_text` body).
    books_report_kind: BooksReportKind,
    /// The field whose "type or pick" list is open, and its rows.
    suggest_open: Option<SuggestField>,
    suggest_rows: Vec<suggest::SuggestRow>,
    /// Bumped on every refresh so the list scrolls back to the top.
    suggest_rev: u64,
}

/// Wrong PINs tolerated before [`UNLOCK_BLOCK`] kicks in.
const UNLOCK_ATTEMPT_LIMIT: u32 = 5;
const UNLOCK_BLOCK: Duration = Duration::from_secs(30);

/// File-dialog target after the user picks a photo or PDF for an entry.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AttachPickTarget {
    Image(AttachmentEntity),
    File(AttachmentEntity),
    OpsPod,
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

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
    /// Parties → Guests & messages: which CRM segment is previewed.
    crm_segment: store::CrmSegment,
    /// Order till: loyalty points to redeem on Take payment (1 pt = ₹1).
    pending_loyalty_points: i64,
    /// Order till: wallet paise to spend on Take payment.
    pending_wallet_paise: i64,
    /// Order till: voucher code + paise to redeem on Take payment.
    pending_voucher_code: String,
    pending_voucher_paise: i64,
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
    /// Devices paired to this Server (Settings → Extra tills).
    lan_devices: Vec<store::LanDevice>,
    /// App data root (desktop `data_dir` or Android files dir).
    data_base: Option<PathBuf>,
    /// Settings → Printers: pool + routes being edited (saved on Save printers).
    printer_draft: print_share::DevicePrinters,
    /// Which route is focused: `bill`, `kot:` (default kitchen), or `kot:<station_id>`.
    printer_route_key: String,
    /// Settings hub: `None` = section list; `Some(id)` = that panel (printers, brand, …).
    settings_panel: Option<&'static str>,
    /// Settings → Printers: pool row tapped for Remove / edit (None = last).
    printer_selected_id: Option<String>,
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
    /// Floor: show only tables assigned to this waiter name (empty = all).
    floor_waiter_filter: String,
    /// Floor map: grid of tiles, or drawn positions (`layout_x/y/w/h`).
    floor_view_layout: bool,
    /// Table selected for the drawn-layout editor (id).
    floor_layout_sel: Option<String>,
    /// Branches screen body (org, branch list, roll-up).
    branches_text: String,
    screen: Screen,
    status: String,
    last_receipt: String,
    /// Sale id eligible for the post-charge undo snackbar (cleared on dismiss/timeout/undo).
    undo_sale_id: Option<String>,
    undo_snack_text: String,
    undo_timer: Timer,
    /// General snackbar (info / success / warn / error) — separate from undo.
    snack_text: String,
    snack_kind: SnackKind,
    snack_timer: Timer,
    /// Keeps the floor and kitchen live: every few seconds the tables'
    /// visit states (and minutes) refresh; a tablet pulls from the Server.
    live_timer: Timer,
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
    /// Books home accordion: which section bodies are open (`today`, `bills`, …).
    books_sections_open: Vec<String>,
    /// Floor secondary accordions (`floor_ops`, `waitlist`, `resv`, `floor_add`).
    floor_sections_open: Vec<String>,
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
    /// Expense form: how it was paid, one row per way (as many as it took).
    pay_parts: Vec<PayPart>,
    /// Open KOTs for the Kitchen screen (refreshed when the screen shows).
    open_kots: Vec<domain::KotTicket>,
    /// Kitchen stations for the Kitchen screen filter/labels.
    stations: Vec<domain::KitchenStation>,
    /// Shop categories for the Menu screen list (photo or initials avatar).
    categories: Vec<domain::Category>,
    /// Item-form picker under the name field: whether it is open, and the
    /// rows it lists for the typed name (see [`DishPick`]).
    dish_picker_open: bool,
    dish_picks: Vec<DishPick>,
    /// Bumped whenever `dish_picks` is rebuilt, so the list scrolls to the top.
    dish_picks_rev: u64,
    /// Category picker under the item form's Category field (same shape as
    /// the item picker: open/closed, rows, scroll-to-top revision).
    category_picker_open: bool,
    category_picks: Vec<CategoryPick>,
    category_picks_rev: u64,
    /// A shelf folder's photo waiting to become the photo of the category
    /// the item is filed under — kept with the name it belongs to, so it is
    /// dropped the moment the owner types a different category.
    form_category_photo: Option<(String, PathBuf)>,
    /// The Category field was filled by the app (from a picked dish), not
    /// typed — the next pick may replace it; a typed one is never replaced.
    form_category_auto: bool,
    /// Order desk search: name, alias or short code ("" = whole menu).
    menu_filter: String,
    /// Order desk: the menu dropdown under the search box is open.
    menu_pick_open: bool,
    /// Bumped when the dropdown's rows change, so it scrolls back to the top.
    menu_pick_rev: u64,
    /// The sent line the ticket's action panel is working on, and what to do
    /// with it (cancel / complimentary / move). `None` = panel closed.
    line_action: Option<(String, LineAction)>,
    /// The ticket line whose details (price, note, Cancel / NC / Move) are
    /// open. One at a time; every other line stays a single compact row.
    open_cart_line: Option<usize>,
    /// Floor opened from Order to Choose / Change table. A tap returns to
    /// Order. Floor from nav does not set this — it only opens that ticket.
    picking_table: bool,
    /// `Some` = Change table (we left this table; pick opens the new ticket
    /// only). `None` = first Choose a table (seat unsent desk punches).
    picking_from_table_id: Option<String>,
    /// The field whose "type or pick" list is open, and its rows.
    suggest_open: Option<SuggestField>,
    suggest_rows: Vec<suggest::SuggestRow>,
    /// Bumped on every refresh so the list scrolls back to the top.
    suggest_rev: u64,
    /// A staff name was picked from the list: select them on this pass.
    team_select_pending: bool,
    pay_choice: PayChoice,
    /// Menu: "Import complete template" was tapped; asking to confirm.
    template_confirm: bool,
    /// Menu screen: the list's search ("Find in your menu").
    menu_list_filter: String,
    /// Menu screen: list only the dishes still at no price (the template
    /// imports every dish at 0 for the owner to price).
    menu_unpriced_only: bool,
    /// Item form: a good bought and sold as is (not a dish cooked here).
    form_resale: bool,
    /// KOT number and send time by KOT id, for the ticket's group headers.
    kot_heads: std::collections::HashMap<String, (i64, String)>,
    /// Raw materials the kitchen buys and uses up (never on the menu).
    raw_materials: Vec<Product>,
    /// Reorder levels the owner set (item id → whole units); others use
    /// [`store::DEFAULT_REORDER_AT`].
    reorder_levels: std::collections::HashMap<String, i64>,
    /// "Rice 0 kg, Paneer 1 kg" — what is at or below its reorder level.
    low_stock_line: String,
    /// Kitchen: "5 × Thali · 3 × Chaas" still to cook over open tickets.
    kitchen_counts_line: String,
    /// Table bookings still to come (Floor screen).
    reservations: Vec<store::Reservation>,
    /// Order desk category chip (category id); `None` = all items;
    /// `Some(MENU_FREQUENT)` = Frequent strip.
    menu_category: Option<String>,
    /// Cached Frequent ranking (product ids, most sold first) + config.
    frequent_ids: Vec<String>,
    frequent_days: u32,
    frequent_limit: u32,
    /// Cancel panel: the kitchen had already cooked it (wastage).
    line_cooked: bool,
    /// Ticket's Move / merge table panel is open.
    move_panel_open: bool,
    /// Splitting the table's bill: taps on sent lines pick them
    /// (`split_pick`, running line ids) for a separate bill.
    split_mode: bool,
    split_pick: Vec<String>,
    /// Ticket's discount panel is open.
    discount_panel_open: bool,
    /// Shop servers found on this Wi‑Fi (Connect screen → Find server).
    found_servers: Vec<net::FoundServer>,
    /// Category whose photo is being picked (file-dialog round trip).
    category_photo_id: Option<String>,
    /// Items taken off the menu (archived, not deleted). The Menu screen
    /// shows them with a Restore button so Remove is always reversible —
    /// the shelf's dishes are app-provided art that the app never removes.
    removed_products: Vec<domain::Product>,
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
    /// A device's PIN is with the Server; a second OK waits for the answer.
    lan_sign_in_busy: bool,
    /// What the last pairing try said, on the pairing screen.
    pair_note: String,
    /// The link state the header last showed (redraw when it changes).
    lan_shown: phone::LinkState,
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
    /// Which bank-books report chip is selected (must match `day_close_text` body).
    books_report_kind: BooksReportKind,
}

/// Wrong PINs tolerated before [`UNLOCK_BLOCK`] kicks in.
const UNLOCK_ATTEMPT_LIMIT: u32 = 5;
const UNLOCK_BLOCK: Duration = Duration::from_secs(30);

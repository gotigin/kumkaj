/// One row of the item form's category picker.
#[derive(Clone, Debug, PartialEq)]
enum CategoryPick {
    /// Use what was typed as a new category (created when the item saves).
    New(String),
    /// One of the owner's categories, by UUID.
    Mine(String),
    /// A shelf folder suggested as a new category, with its photo.
    Shelf(menu_images::ShelfFolder),
}

/// How often a Client till on Sell asks the Server for fresh books (seconds).
const LIVE_REFRESH_SECS: f64 = 3.0;

/// Content width the shell lays screens out at (set by `apply_shell_layout`).
fn content_width() -> &'static Mutex<f64> {
    static W: OnceLock<Mutex<f64>> = OnceLock::new();
    W.get_or_init(|| Mutex::new(760.0))
}

/// One way an expense was paid: the method and the amount as typed (blank
/// = whatever is left of the total).
#[derive(Clone, Debug, PartialEq, Eq)]
struct PayPart {
    method: PayMethod,
    amount: String,
}

impl Default for PayPart {
    fn default() -> Self {
        Self { method: PayMethod::Cash, amount: String::new() }
    }
}

/// Methods an expense can be paid by, in chip order.
const PART_METHODS: [PayMethod; 4] = [PayMethod::Cash, PayMethod::Upi, PayMethod::Card, PayMethod::Bank];

/// The parts as (method, paise), adding up to `total`. One part may be
/// blank: it takes what is left. Parts by the same way are merged.

/// Paise covered by pending loyalty / wallet / voucher on the till.
fn pending_till_redeem_paise(st: &AppState) -> i64 {
    st.pending_loyalty_points.saturating_mul(100)
        + st.pending_wallet_paise.max(0)
        + st.pending_voucher_paise.max(0)
}

/// Keep cart.discount in sync with pending till redemptions (Amount, no PIN).
fn apply_pending_till_discount(st: &mut AppState) {
    let paise = pending_till_redeem_paise(st);
    if paise <= 0 {
        if st
            .cart
            .discount
            .as_ref()
            .is_some_and(|d| d.reason.starts_with("Till redeem"))
        {
            st.cart.discount = None;
            st.cart.charge_total = None;
        }
        return;
    }
    let mut bits = Vec::new();
    if st.pending_loyalty_points > 0 {
        bits.push(format!("{} pts", st.pending_loyalty_points));
    }
    if st.pending_wallet_paise > 0 {
        bits.push(format!("wallet {}", st.money(Money(st.pending_wallet_paise))));
    }
    if st.pending_voucher_paise > 0 {
        bits.push(format!(
            "voucher {} {}",
            st.pending_voucher_code,
            st.money(Money(st.pending_voucher_paise))
        ));
    }
    st.cart.charge_total = None;
    st.cart.discount = Some(domain::BillDiscount {
        kind: domain::DiscountKind::Amount(paise),
        reason: format!("Till redeem · {}", bits.join(" · ")),
        approval_pin: String::new(),
    });
}

fn settle_pay_parts(total: i64, parts: &[PayPart], currency: &str) -> Result<Vec<(PayMethod, i64)>, String> {
    if total <= 0 {
        return Err("Type the amount first".into());
    }
    let mut known = 0i64;
    let mut blank: Option<PayMethod> = None;
    let mut out: Vec<(PayMethod, i64)> = Vec::new();
    for part in parts {
        if part.amount.trim().is_empty() {
            if blank.is_some() {
                return Err("Only one way can be left blank (it takes the rest)".into());
            }
            blank = Some(part.method);
            continue;
        }
        let paise = locale::parse_major_input(part.amount.trim(), currency)?;
        if paise <= 0 {
            return Err("Each way needs an amount above zero".into());
        }
        known += paise;
        out.push((part.method, paise));
    }
    match blank {
        Some(method) => {
            let rest = total - known;
            if rest <= 0 {
                return Err("The other ways already cover the amount — remove the blank one".into());
            }
            out.push((method, rest));
        }
        None if known != total => {
            return Err(format!(
                "The ways add up to {:.2}, not {:.2}",
                known as f64 / 100.0,
                total as f64 / 100.0
            ));
        }
        None => {}
    }
    let mut merged: Vec<(PayMethod, i64)> = Vec::new();
    for (m, a) in out {
        match merged.iter_mut().find(|(mm, _)| *mm == m) {
            Some(slot) => slot.1 += a,
            None => merged.push((m, a)),
        }
    }
    Ok(merged)
}

/// How the bill is paid, one tap on the ticket. A single method takes the
/// whole bill; Split opens the per-method amounts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum PayChoice {
    #[default]
    Cash,
    Upi,
    Card,
    Credit,
    Split,
}

/// A field whose value is usually something the shop already has. Each
/// has a [`SuggestList`] right under its input (see `suggest.rs`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SuggestField {
    MenuUnit,
    MenuTax,
    MenuGroup,
    BuyItem,
    BuySupplier,
    InvItem,
    CreditParty,
    TeamPick,
    /// Team → Add: copy role/title/features from an existing person.
    TeamCopyHire,
    /// Team → Edit: copy role/title/features onto the selected person.
    TeamCopyEdit,
    ConvFrom,
    ConvTo,
    BranchSwitch,
    BuyUnit,
    Country,
    PrinterTarget,
    /// Order desk: customer / khata party name.
    CustomerName,
    /// Settings: bulk price by category name.
    BulkCat,
    /// Settings → Printers: which place (Bill) is being edited.
    PrinterPlace,
    /// Settings → Places: check printers (× format) for the focused place.
    PrinterAssign,
    /// Bill & KOT → Bill: real printers that may print the bill.
    BillPrinters,
    /// Settings → Printers: which pool printers get a test page.
    TestPrinters,
}

impl SuggestField {
    const ALL: [SuggestField; 22] = [
        SuggestField::MenuUnit,
        SuggestField::MenuTax,
        SuggestField::MenuGroup,
        SuggestField::BuyItem,
        SuggestField::BuySupplier,
        SuggestField::InvItem,
        SuggestField::CreditParty,
        SuggestField::TeamPick,
        SuggestField::TeamCopyHire,
        SuggestField::TeamCopyEdit,
        SuggestField::ConvFrom,
        SuggestField::ConvTo,
        SuggestField::BranchSwitch,
        SuggestField::BuyUnit,
        SuggestField::Country,
        SuggestField::PrinterTarget,
        SuggestField::CustomerName,
        SuggestField::BulkCat,
        SuggestField::PrinterPlace,
        SuggestField::PrinterAssign,
        SuggestField::BillPrinters,
        SuggestField::TestPrinters,
    ];

    /// The `field:` a `SuggestList` names in the DSL.
    fn key(self) -> &'static str {
        match self {
            SuggestField::MenuUnit => "menu_unit",
            SuggestField::MenuTax => "menu_tax",
            SuggestField::MenuGroup => "menu_group",
            SuggestField::BuyItem => "buy_item",
            SuggestField::BuySupplier => "buy_supplier",
            SuggestField::InvItem => "inv_item",
            SuggestField::CreditParty => "credit_party",
            SuggestField::TeamPick => "team_pick",
            SuggestField::TeamCopyHire => "team_copy_hire",
            SuggestField::TeamCopyEdit => "team_copy_edit",
            SuggestField::ConvFrom => "conv_from",
            SuggestField::ConvTo => "conv_to",
            SuggestField::BranchSwitch => "branch_switch",
            SuggestField::BuyUnit => "buy_unit",
            SuggestField::Country => "country",
            SuggestField::PrinterTarget => "printer_target",
            SuggestField::CustomerName => "customer_name",
            SuggestField::BulkCat => "bulk_cat",
            SuggestField::PrinterPlace => "printer_place",
            SuggestField::PrinterAssign => "printer_assign",
            SuggestField::BillPrinters => "bill_printers",
            SuggestField::TestPrinters => "test_printers",
        }
    }

    fn input(self) -> &'static [LiveId] {
        match self {
            SuggestField::MenuUnit => ids!(menu_unit_input),
            SuggestField::MenuTax => ids!(menu_tax_input),
            SuggestField::MenuGroup => ids!(menu_group_input),
            SuggestField::BuyItem => ids!(buy_item_input),
            SuggestField::BuySupplier => ids!(buy_supplier_input),
            SuggestField::InvItem => ids!(inv_item_input),
            SuggestField::CreditParty => ids!(credit_limit_party),
            SuggestField::TeamPick => ids!(team_pick_input),
            SuggestField::TeamCopyHire => ids!(staff_copy_hire_input),
            SuggestField::TeamCopyEdit => ids!(staff_copy_edit_input),
            SuggestField::ConvFrom => ids!(conv_from_input),
            SuggestField::ConvTo => ids!(conv_to_input),
            SuggestField::BranchSwitch => ids!(branch_switch_input),
            SuggestField::BuyUnit => ids!(buy_unit_input),
            SuggestField::Country => ids!(country_code_input),
            SuggestField::PrinterTarget => ids!(printer_target_input),
            SuggestField::CustomerName => ids!(customer_name_input),
            SuggestField::BulkCat => ids!(bulk_cat_input),
            SuggestField::PrinterPlace => ids!(printer_place_input),
            SuggestField::PrinterAssign => ids!(printer_assign_input),
            SuggestField::BillPrinters => ids!(bill_printers_input),
            SuggestField::TestPrinters => ids!(printers_test_input),
        }
    }

    fn list(self) -> &'static [LiveId] {
        match self {
            SuggestField::MenuUnit => ids!(menu_unit_suggest),
            SuggestField::MenuTax => ids!(menu_tax_suggest),
            SuggestField::MenuGroup => ids!(menu_group_suggest),
            SuggestField::BuyItem => ids!(buy_item_suggest),
            SuggestField::BuySupplier => ids!(buy_supplier_suggest),
            SuggestField::InvItem => ids!(inv_item_suggest),
            SuggestField::CreditParty => ids!(credit_party_suggest),
            SuggestField::TeamPick => ids!(team_pick_suggest),
            SuggestField::TeamCopyHire => ids!(staff_copy_hire_suggest),
            SuggestField::TeamCopyEdit => ids!(staff_copy_edit_suggest),
            SuggestField::ConvFrom => ids!(conv_from_suggest),
            SuggestField::ConvTo => ids!(conv_to_suggest),
            SuggestField::BranchSwitch => ids!(branch_switch_suggest),
            SuggestField::BuyUnit => ids!(buy_unit_suggest),
            SuggestField::Country => ids!(country_suggest),
            SuggestField::PrinterTarget => ids!(printer_target_suggest),
            SuggestField::CustomerName => ids!(customer_name_suggest),
            SuggestField::BulkCat => ids!(bulk_cat_suggest),
            SuggestField::PrinterPlace => ids!(printer_place_suggest),
            SuggestField::PrinterAssign => ids!(printer_assign_suggest),
            SuggestField::BillPrinters => ids!(bill_printers_suggest),
            SuggestField::TestPrinters => ids!(printers_test_suggest),
        }
    }

    /// The dropdown button that opens and closes this list, for lists that
    /// a button opens rather than a type-in field. A press on it is "inside"
    /// the list: it toggles the list itself, so the outside-press rule must
    /// not close it first and let the button open it again.
    fn opener(self) -> Option<&'static [LiveId]> {
        match self {
            SuggestField::PrinterPlace => Some(ids!(printer_place_dd)),
            SuggestField::PrinterAssign => Some(ids!(printer_assign_dd)),
            SuggestField::BillPrinters => Some(ids!(bill_printers_dd)),
            SuggestField::TestPrinters => Some(ids!(printers_test_dd)),
            _ => None,
        }
    }

    /// A list that its own dropdown button opens — never a tap or typing in
    /// a field.
    fn opens_by_chrome(self) -> bool {
        self.opener().is_some()
    }
}

/// Settings → Where things print: tap one of these to put that pool printer
/// on the focused route (Bill / station). Slots match the DSL below.
const ROUTE_ADD_PRINTER_BTNS: [&[LiveId]; 16] = [
    ids!(route_add_prn_0),
    ids!(route_add_prn_1),
    ids!(route_add_prn_2),
    ids!(route_add_prn_3),
    ids!(route_add_prn_4),
    ids!(route_add_prn_5),
    ids!(route_add_prn_6),
    ids!(route_add_prn_7),
    ids!(route_add_prn_8),
    ids!(route_add_prn_9),
    ids!(route_add_prn_10),
    ids!(route_add_prn_11),
    ids!(route_add_prn_12),
    ids!(route_add_prn_13),
    ids!(route_add_prn_14),
    ids!(route_add_prn_15),
];

/// Bill + kitchen stations to edit (one place at a time).
/// Idle/active SegBtn pairs — selection by visibility, never by rewriting
/// Stack* labels (set_text on Fill+wrap freezes Settings; DEV.md bug 7).
const ROUTE_PLACE_BTNS: [(&[LiveId], &[LiveId]); 8] = [
    (ids!(route_place_0), ids!(route_place_0_on)),
    (ids!(route_place_1), ids!(route_place_1_on)),
    (ids!(route_place_2), ids!(route_place_2_on)),
    (ids!(route_place_3), ids!(route_place_3_on)),
    (ids!(route_place_4), ids!(route_place_4_on)),
    (ids!(route_place_5), ids!(route_place_5_on)),
    (ids!(route_place_6), ids!(route_place_6_on)),
    (ids!(route_place_7), ids!(route_place_7_on)),
];

/// Sentinel for the Frequent chip (not a real category id).
const MENU_FREQUENT: &str = "__frequent__";
/// Sentinel for the Added Item(s) chip — dishes currently on this order.
const MENU_ADDED: &str = "__added__";
/// Max category pages in Add dishes (All, Frequent, Added, + 24 category slots).
const MENU_PICK_PAGE_MAX: usize = 27;

/// One `ProductList` per horizontal page in the Add dishes pager (`pick_grid_*`).
const MENU_PICK_GRIDS: [&[LiveId]; MENU_PICK_PAGE_MAX] = [
    ids!(pick_grid_0),
    ids!(pick_grid_1),
    ids!(pick_grid_2),
    ids!(pick_grid_3),
    ids!(pick_grid_4),
    ids!(pick_grid_5),
    ids!(pick_grid_6),
    ids!(pick_grid_7),
    ids!(pick_grid_8),
    ids!(pick_grid_9),
    ids!(pick_grid_10),
    ids!(pick_grid_11),
    ids!(pick_grid_12),
    ids!(pick_grid_13),
    ids!(pick_grid_14),
    ids!(pick_grid_15),
    ids!(pick_grid_16),
    ids!(pick_grid_17),
    ids!(pick_grid_18),
    ids!(pick_grid_19),
    ids!(pick_grid_20),
    ids!(pick_grid_21),
    ids!(pick_grid_22),
    ids!(pick_grid_23),
    ids!(pick_grid_24),
    ids!(pick_grid_25),
    ids!(pick_grid_26),
];

/// Order desk category chip slots in DSL order (`cat_0`…`cat_23`).
const CATEGORY_CHIPS: [(&[LiveId], &[LiveId]); 24] = [
    (ids!(cat_0), ids!(cat_0_on)),
    (ids!(cat_1), ids!(cat_1_on)),
    (ids!(cat_2), ids!(cat_2_on)),
    (ids!(cat_3), ids!(cat_3_on)),
    (ids!(cat_4), ids!(cat_4_on)),
    (ids!(cat_5), ids!(cat_5_on)),
    (ids!(cat_6), ids!(cat_6_on)),
    (ids!(cat_7), ids!(cat_7_on)),
    (ids!(cat_8), ids!(cat_8_on)),
    (ids!(cat_9), ids!(cat_9_on)),
    (ids!(cat_10), ids!(cat_10_on)),
    (ids!(cat_11), ids!(cat_11_on)),
    (ids!(cat_12), ids!(cat_12_on)),
    (ids!(cat_13), ids!(cat_13_on)),
    (ids!(cat_14), ids!(cat_14_on)),
    (ids!(cat_15), ids!(cat_15_on)),
    (ids!(cat_16), ids!(cat_16_on)),
    (ids!(cat_17), ids!(cat_17_on)),
    (ids!(cat_18), ids!(cat_18_on)),
    (ids!(cat_19), ids!(cat_19_on)),
    (ids!(cat_20), ids!(cat_20_on)),
    (ids!(cat_21), ids!(cat_21_on)),
    (ids!(cat_22), ids!(cat_22_on)),
    (ids!(cat_23), ids!(cat_23_on)),
];

/// The page's visible window in screen coordinates (top, height): set by
/// the app every event, read by lists that draw only what is on screen.
fn page_viewport() -> &'static Mutex<(f64, f64)> {
    static V: OnceLock<Mutex<(f64, f64)>> = OnceLock::new();
    V.get_or_init(|| Mutex::new((0.0, 0.0)))
}

/// Rows `first..last` of a page-scrolled list that are inside the page's
/// window, given where the list starts on screen (`top`) and its fixed row
/// pitch. Lists draw only these and stand in for the rest with empty height.
fn page_window(top: f64, rows: usize, row_h: f64, gap: f64) -> (usize, usize) {
    let pitch = row_h + gap;
    let (vp_top, vp_h) = *page_viewport().lock().unwrap();
    if vp_h <= 1.0 {
        return (0, rows.min(8));
    }
    let from = ((vp_top - top) / pitch).floor().max(0.0) as usize;
    let to = (((vp_top + vp_h - top) / pitch).ceil().max(0.0) as usize + 1).min(rows);
    (from.min(rows), to)
}

/// Narrowest a dish tile may get in the menu window.
/// Prefer two readable image|text cards over three cramped ones.
const PRODUCT_TILE_MIN_W: f64 = 300.0;
/// Dish tile slots per row in the DSL (`c0`…`c5`).
const PRODUCT_MAX_COLUMNS: usize = 6;

/// Horizontal gap between cards in a ProductList row (keep in sync with DSL).
const PRODUCT_TILE_GAP: f64 = 20.0;
/// Right pad on each ProductList row: the vertical scroll bar's lane
/// (`bar_size` 10 + 4), so the bar never covers the last card.
const PRODUCT_LIST_BAR_LANE: f64 = 14.0;

/// Cards across a list `width` wide (the row keeps the bar lane free).
fn product_columns(width: f64) -> usize {
    let room = (width - PRODUCT_LIST_BAR_LANE).max(0.0);
    (((room + PRODUCT_TILE_GAP) / (PRODUCT_TILE_MIN_W + PRODUCT_TILE_GAP)).floor() as usize)
        .clamp(1, PRODUCT_MAX_COLUMNS)
}

/// Tiles per row the menu window's grid last drew with (taps map a row and
/// column back to a dish with it).
fn product_grid_cols() -> &'static Mutex<usize> {
    static C: OnceLock<Mutex<usize>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(2))
}

/// One row of the item form's picker (under the name field).
#[derive(Clone, Debug, PartialEq)]
enum DishPick {
    /// Keep what was typed as the owner's own item name.
    Own(String),
    /// An item already on the menu, by its UUID — opens it to edit.
    Menu(String),
    /// A shelf dish (index into `menu_images::shelf()`) — fills name + photo.
    Shelf(usize),
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    Sales,
    Credit,
    Team,
    Settings,
    /// Settings → Sidebar: order + show/hide chrome peers.
    SidebarSettings,
    DayClose,
    Reports,
    Inventory,
    Branches,
}

/// Which chrome peer is selected. Former More hub items are first-class peers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavTab {
    Books,
    Sell,
    Buy,
    Menu,
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
/// Team is Settings → Team (not a chrome peer).
const NAV_TABS: [NavTab; 12] = [
    NavTab::Books,
    NavTab::Sell,
    NavTab::Buy,
    NavTab::Menu,
    NavTab::Sales,
    NavTab::DayClose,
    NavTab::Reports,
    NavTab::Inventory,
    NavTab::Branches,
    NavTab::Credit,
    NavTab::Settings,
    NavTab::Lock,
];

fn nav_tab_key(tab: NavTab) -> &'static str {
    match tab {
        NavTab::Books => "books",
        NavTab::Sell => "sell",
        NavTab::Buy => "buy",
        NavTab::Menu => "menu",
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

/// Bundled colored WebP for a nav tab (`resources/nav/<key>.webp`).
/// Makepad cannot paint Noto Color Emoji in colour — real images only.
fn nav_tab_icon_bytes(tab: NavTab) -> &'static [u8] {
    match tab {
        NavTab::Books => include_bytes!("../resources/nav/books.webp").as_slice(),
        NavTab::Sell => include_bytes!("../resources/nav/sell.webp").as_slice(),
        NavTab::Buy => include_bytes!("../resources/nav/buy.webp").as_slice(),
        NavTab::Menu => include_bytes!("../resources/nav/menu.webp").as_slice(),
        NavTab::Sales => include_bytes!("../resources/nav/sales.webp").as_slice(),
        NavTab::DayClose => include_bytes!("../resources/nav/day_close.webp").as_slice(),
        NavTab::Reports => include_bytes!("../resources/nav/reports.webp").as_slice(),
        NavTab::Inventory => include_bytes!("../resources/nav/inventory.webp").as_slice(),
        NavTab::Branches => include_bytes!("../resources/nav/branches.webp").as_slice(),
        NavTab::Credit => include_bytes!("../resources/nav/credit.webp").as_slice(),
        NavTab::Team => include_bytes!("../resources/nav/team.webp").as_slice(),
        NavTab::Settings => include_bytes!("../resources/nav/settings.webp").as_slice(),
        NavTab::Lock => include_bytes!("../resources/nav/lock.webp").as_slice(),
    }
}

/// Screen / nav label text. Colored mark sits in a sibling `Image`, not in the string.
fn nav_tab_caption(_tab: NavTab, label: &str) -> String {
    label.to_string()
}

/// Decode a nav WebP into `img` only when the tab (or empty image) changed.
/// Re-loading + `redraw` on every chrome sync scheduled endless frames.
/// Returns `true` when bytes were freshly loaded (caller may need a
/// follow-up redraw so the texture is visible).
fn load_nav_icon_once(cx: &mut Cx, img: &ImageRef, tab: NavTab) -> bool {
    static LOADED: OnceLock<Mutex<std::collections::HashMap<u64, &'static str>>> = OnceLock::new();
    let loaded = LOADED.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let uid = img.widget_uid().0;
    let key = nav_tab_key(tab);
    {
        let map = loaded.lock().unwrap();
        if map.get(&uid).copied() == Some(key) {
            img.set_visible(cx, true);
            return false;
        }
    }
    let _ = img.load_image_from_data(cx, nav_tab_icon_bytes(tab));
    loaded.lock().unwrap().insert(uid, key);
    img.set_visible(cx, true);
    true
}

fn info_help_tooltip_options(widget_rect: Rect) -> CalloutTooltipOptions {
    // Prefer Bottom so long tips get the full readable wrap width. Only open
    // sideways when the icon is hard against an edge.
    let icon_right = widget_rect.pos.x + widget_rect.size.x;
    let position = if widget_rect.pos.x < 48.0 {
        TooltipPosition::Right
    } else if icon_right > 720.0 {
        TooltipPosition::Left
    } else {
        TooltipPosition::Bottom
    };
    CalloutTooltipOptions {
        position,
        text_color: vec4(1.0, 1.0, 1.0, 1.0),
        bg_color: vec4(0.412, 0.055, 0.063, 0.97), // #69090C
        triangle_height: 8.0,
    }
}

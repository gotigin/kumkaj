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

/// How often the floor, kitchen and order desk ask for fresh books (seconds).
/// Kept short so a paired till sees the Server's KOTs / tables without a long wait.
const LIVE_REFRESH_SECS: f64 = 3.0;

/// What the ticket's action panel does to one sent line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineAction {
    /// Cancel after KOT (reason, cooked or not, quantity).
    Cancel,
    /// Make complimentary (NC) — or charge again when it already is.
    Complimentary,
    /// Move to another table.
    Move,
}

/// One floor row: an area heading, or a row of table tiles (indexes into
/// `tables`, as many as the chosen columns — no upper limit).
/// Production Floor paints one selected area tab; this shape stays for
/// unit tests of the older multi-area row algorithm.
#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
enum FloorRow {
    Area(String),
    Tiles(Vec<usize>),
}

/// Every floor table tile is this size (css px), whatever the column count:
/// more columns than fit make the tables scroll sideways, never shrink.
const FLOOR_TILE_W: f64 = 280.0;
const FLOOR_TILE_H: f64 = 178.0;
/// Gap between tiles across, and between rows (the row's bottom padding).
const FLOOR_TILE_GAP: f64 = 10.0;
/// Tile slots in one row template. Only the columns on screen are drawn into
/// them (the rest of a wide row is a spacer), so this bounds the columns
/// *visible at once* (24 × 290 ≈ 7000px), not the columns a row may have.
const FLOOR_ROW_SLOTS: usize = 24;
/// Drawn floor: how many tiles the layout canvas can show at once.
const FLOOR_LAYOUT_SLOTS: usize = 24;
/// Auto-pack tile size when every table still has a zero layout box.
const FLOOR_LAYOUT_PACK_W: f64 = 120.0;
const FLOOR_LAYOUT_PACK_H: f64 = 90.0;

/// Content width the shell lays screens out at (set by `apply_shell_layout`).
fn content_width() -> &'static Mutex<f64> {
    static W: OnceLock<Mutex<f64>> = OnceLock::new();
    W.get_or_init(|| Mutex::new(760.0))
}

/// Columns the floor grid last drew with (taps map through the same count).
fn floor_grid_cols() -> &'static Mutex<usize> {
    static C: OnceLock<Mutex<usize>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(1))
}

/// The column the floor grid's first tile slot last drew (scrolled sideways,
/// slot 0 is not column 0) — taps map slot → column through it.
fn floor_first_col() -> &'static Mutex<usize> {
    static C: OnceLock<Mutex<usize>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(0))
}

/// The Floor tables' sideways viewport: `(left, width)` of `floor_scroll`
/// on screen, set every event (like [`page_viewport`] for the page).
fn floor_hviewport() -> &'static Mutex<(f64, f64)> {
    static V: OnceLock<Mutex<(f64, f64)>> = OnceLock::new();
    V.get_or_init(|| Mutex::new((0.0, 0.0)))
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
    MoveTable,
    LineTable,
    TableArea,
    MenuStation,
    /// Order desk: station for unsent dishes that have none on the menu item.
    KotStation,
    MenuUnit,
    MenuTax,
    MenuGroup,
    BuyItem,
    BuySupplier,
    InvItem,
    CreditParty,
    TeamPick,
    ConvFrom,
    ConvTo,
    BranchSwitch,
    BuyUnit,
    Country,
    DeliveryBy,
    PrinterTarget,
    /// Kitchen screen: pick a station to remove.
    StationRemove,
    /// Settings → Areas · stations: same list, own field.
    SettingsStationRemove,
    /// Order desk: customer / khata party name.
    CustomerName,
    /// Floor: reservation preferred table.
    ResvTable,
    /// Floor: filter tiles by waiter / staff name.
    FloorWaiter,
    /// Settings: bulk price by category name.
    BulkCat,
    /// Floor: tables per row (Fit screen, 1, 2 … or any typed number).
    FloorColumns,
    /// Settings → Printers: which place (Bill / kitchen) is being edited.
    PrinterPlace,
    /// Settings → Places: check printers (× format) for the focused place.
    PrinterAssign,
}

impl SuggestField {
    const ALL: [SuggestField; 29] = [
        SuggestField::MoveTable,
        SuggestField::LineTable,
        SuggestField::TableArea,
        SuggestField::MenuStation,
        SuggestField::KotStation,
        SuggestField::MenuUnit,
        SuggestField::MenuTax,
        SuggestField::MenuGroup,
        SuggestField::BuyItem,
        SuggestField::BuySupplier,
        SuggestField::InvItem,
        SuggestField::CreditParty,
        SuggestField::TeamPick,
        SuggestField::ConvFrom,
        SuggestField::ConvTo,
        SuggestField::BranchSwitch,
        SuggestField::BuyUnit,
        SuggestField::Country,
        SuggestField::DeliveryBy,
        SuggestField::PrinterTarget,
        SuggestField::StationRemove,
        SuggestField::SettingsStationRemove,
        SuggestField::CustomerName,
        SuggestField::ResvTable,
        SuggestField::FloorWaiter,
        SuggestField::BulkCat,
        SuggestField::FloorColumns,
        SuggestField::PrinterPlace,
        SuggestField::PrinterAssign,
    ];

    /// The `field:` a `SuggestList` names in the DSL.
    fn key(self) -> &'static str {
        match self {
            SuggestField::MoveTable => "move_table",
            SuggestField::LineTable => "line_table",
            SuggestField::TableArea => "table_area",
            SuggestField::MenuStation => "menu_station",
            SuggestField::KotStation => "kot_station",
            SuggestField::MenuUnit => "menu_unit",
            SuggestField::MenuTax => "menu_tax",
            SuggestField::MenuGroup => "menu_group",
            SuggestField::BuyItem => "buy_item",
            SuggestField::BuySupplier => "buy_supplier",
            SuggestField::InvItem => "inv_item",
            SuggestField::CreditParty => "credit_party",
            SuggestField::TeamPick => "team_pick",
            SuggestField::ConvFrom => "conv_from",
            SuggestField::ConvTo => "conv_to",
            SuggestField::BranchSwitch => "branch_switch",
            SuggestField::BuyUnit => "buy_unit",
            SuggestField::Country => "country",
            SuggestField::DeliveryBy => "delivery_by",
            SuggestField::PrinterTarget => "printer_target",
            SuggestField::StationRemove => "station_remove",
            SuggestField::SettingsStationRemove => "settings_station_remove",
            SuggestField::CustomerName => "customer_name",
            SuggestField::ResvTable => "resv_table",
            SuggestField::FloorWaiter => "floor_waiter",
            SuggestField::BulkCat => "bulk_cat",
            SuggestField::FloorColumns => "floor_cols",
            SuggestField::PrinterPlace => "printer_place",
            SuggestField::PrinterAssign => "printer_assign",
        }
    }

    fn input(self) -> &'static [LiveId] {
        match self {
            SuggestField::MoveTable => ids!(move_target_input),
            SuggestField::LineTable => ids!(line_target_input),
            SuggestField::TableArea => ids!(table_area_input),
            SuggestField::MenuStation => ids!(menu_station_input),
            SuggestField::KotStation => ids!(kot_station_input),
            SuggestField::MenuUnit => ids!(menu_unit_input),
            SuggestField::MenuTax => ids!(menu_tax_input),
            SuggestField::MenuGroup => ids!(menu_group_input),
            SuggestField::BuyItem => ids!(buy_item_input),
            SuggestField::BuySupplier => ids!(buy_supplier_input),
            SuggestField::InvItem => ids!(inv_item_input),
            SuggestField::CreditParty => ids!(credit_limit_party),
            SuggestField::TeamPick => ids!(team_pick_input),
            SuggestField::ConvFrom => ids!(conv_from_input),
            SuggestField::ConvTo => ids!(conv_to_input),
            SuggestField::BranchSwitch => ids!(branch_switch_input),
            SuggestField::BuyUnit => ids!(buy_unit_input),
            SuggestField::Country => ids!(country_code_input),
            SuggestField::DeliveryBy => ids!(delivery_by_input),
            SuggestField::PrinterTarget => ids!(printer_target_input),
            SuggestField::StationRemove => ids!(station_remove_input),
            SuggestField::SettingsStationRemove => ids!(settings_station_remove_input),
            SuggestField::CustomerName => ids!(customer_name_input),
            SuggestField::ResvTable => ids!(resv_table_input),
            SuggestField::FloorWaiter => ids!(floor_waiter_filter),
            SuggestField::BulkCat => ids!(bulk_cat_input),
            SuggestField::FloorColumns => ids!(floor_cols_input),
            SuggestField::PrinterPlace => ids!(printer_place_input),
            SuggestField::PrinterAssign => ids!(printer_assign_input),
        }
    }

    fn list(self) -> &'static [LiveId] {
        match self {
            SuggestField::MoveTable => ids!(move_target_suggest),
            SuggestField::LineTable => ids!(line_target_suggest),
            SuggestField::TableArea => ids!(table_area_suggest),
            SuggestField::MenuStation => ids!(menu_station_suggest),
            SuggestField::KotStation => ids!(kot_station_suggest),
            SuggestField::MenuUnit => ids!(menu_unit_suggest),
            SuggestField::MenuTax => ids!(menu_tax_suggest),
            SuggestField::MenuGroup => ids!(menu_group_suggest),
            SuggestField::BuyItem => ids!(buy_item_suggest),
            SuggestField::BuySupplier => ids!(buy_supplier_suggest),
            SuggestField::InvItem => ids!(inv_item_suggest),
            SuggestField::CreditParty => ids!(credit_party_suggest),
            SuggestField::TeamPick => ids!(team_pick_suggest),
            SuggestField::ConvFrom => ids!(conv_from_suggest),
            SuggestField::ConvTo => ids!(conv_to_suggest),
            SuggestField::BranchSwitch => ids!(branch_switch_suggest),
            SuggestField::BuyUnit => ids!(buy_unit_suggest),
            SuggestField::Country => ids!(country_suggest),
            SuggestField::DeliveryBy => ids!(delivery_by_suggest),
            SuggestField::PrinterTarget => ids!(printer_target_suggest),
            SuggestField::StationRemove => ids!(station_remove_suggest),
            SuggestField::SettingsStationRemove => ids!(settings_station_remove_suggest),
            SuggestField::CustomerName => ids!(customer_name_suggest),
            SuggestField::ResvTable => ids!(resv_table_suggest),
            SuggestField::FloorWaiter => ids!(floor_waiter_suggest),
            SuggestField::BulkCat => ids!(bulk_cat_suggest),
            SuggestField::FloorColumns => ids!(floor_cols_suggest),
            SuggestField::PrinterPlace => ids!(printer_place_suggest),
            SuggestField::PrinterAssign => ids!(printer_assign_suggest),
        }
    }
}

/// The tile slots of one floor row (`FloorList` `Item`), left to right:
/// slot `k` shows column `floor_first_col() + k`.
const FLOOR_SLOT_IDS: [&[LiveId]; FLOOR_ROW_SLOTS] = [
    ids!(t0), ids!(t1), ids!(t2), ids!(t3), ids!(t4), ids!(t5),
    ids!(t6), ids!(t7), ids!(t8), ids!(t9), ids!(t10), ids!(t11),
    ids!(t12), ids!(t13), ids!(t14), ids!(t15), ids!(t16), ids!(t17),
    ids!(t18), ids!(t19), ids!(t20), ids!(t21), ids!(t22), ids!(t23),
];

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

#[allow(dead_code)] // Legacy chip strip slots; place UI is printer_place_dd now.
const ROUTE_PLACE_SLOTS: [&[LiveId]; 8] = [
    ids!(route_place_slot_0),
    ids!(route_place_slot_1),
    ids!(route_place_slot_2),
    ids!(route_place_slot_3),
    ids!(route_place_slot_4),
    ids!(route_place_slot_5),
    ids!(route_place_slot_6),
    ids!(route_place_slot_7),
];

/// Sentinel for the Frequent chip (not a real category id).
const MENU_FREQUENT: &str = "__frequent__";
/// Sentinel for the Added Item(s) chip — dishes currently on this order.
const MENU_ADDED: &str = "__added__";

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

/// Floor area tab chips (`floor_area_0`…`floor_area_23`) — one selected area
/// at a time; that tab's body owns Columns + tables.
const FLOOR_AREA_CHIPS: [(&[LiveId], &[LiveId]); 24] = [
    (ids!(floor_area_0), ids!(floor_area_0_on)),
    (ids!(floor_area_1), ids!(floor_area_1_on)),
    (ids!(floor_area_2), ids!(floor_area_2_on)),
    (ids!(floor_area_3), ids!(floor_area_3_on)),
    (ids!(floor_area_4), ids!(floor_area_4_on)),
    (ids!(floor_area_5), ids!(floor_area_5_on)),
    (ids!(floor_area_6), ids!(floor_area_6_on)),
    (ids!(floor_area_7), ids!(floor_area_7_on)),
    (ids!(floor_area_8), ids!(floor_area_8_on)),
    (ids!(floor_area_9), ids!(floor_area_9_on)),
    (ids!(floor_area_10), ids!(floor_area_10_on)),
    (ids!(floor_area_11), ids!(floor_area_11_on)),
    (ids!(floor_area_12), ids!(floor_area_12_on)),
    (ids!(floor_area_13), ids!(floor_area_13_on)),
    (ids!(floor_area_14), ids!(floor_area_14_on)),
    (ids!(floor_area_15), ids!(floor_area_15_on)),
    (ids!(floor_area_16), ids!(floor_area_16_on)),
    (ids!(floor_area_17), ids!(floor_area_17_on)),
    (ids!(floor_area_18), ids!(floor_area_18_on)),
    (ids!(floor_area_19), ids!(floor_area_19_on)),
    (ids!(floor_area_20), ids!(floor_area_20_on)),
    (ids!(floor_area_21), ids!(floor_area_21_on)),
    (ids!(floor_area_22), ids!(floor_area_22_on)),
    (ids!(floor_area_23), ids!(floor_area_23_on)),
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

/// Floor grid pitch: area headings are short; tile rows are a tile + gap.
/// Production FloorList windows by area-block height; these helpers keep the
/// older FloorRow pitch algorithm under test.
#[cfg(test)]
fn floor_row_pitch(row: &FloorRow) -> f64 {
    match row {
        FloorRow::Area(_) => 34.0,
        FloorRow::Tiles(_) => FLOOR_TILE_H + FLOOR_TILE_GAP,
    }
}

/// Width of a floor row of `cols` tiles.
fn floor_grid_width(cols: usize) -> f64 {
    let cols = cols.max(1) as f64;
    cols * FLOOR_TILE_W + (cols - 1.0) * FLOOR_TILE_GAP
}

/// Columns `first..last` of a floor row that show in the sideways viewport
/// (`vp_left`, `vp_w`) when the row starts at `row_left` — at most
/// [`FLOOR_ROW_SLOTS`]. Before the viewport is known, the first columns.
fn floor_col_window(row_left: f64, vp_left: f64, vp_w: f64, cols: usize) -> (usize, usize) {
    if vp_w <= 1.0 {
        return (0, cols.min(FLOOR_ROW_SLOTS));
    }
    let pitch = FLOOR_TILE_W + FLOOR_TILE_GAP;
    let first = ((vp_left - row_left) / pitch).floor().max(0.0) as usize;
    let first = first.min(cols);
    let last = ((vp_left + vp_w - row_left) / pitch).ceil().max(0.0) as usize;
    let last = last.clamp(first, cols).min(first + FLOOR_ROW_SLOTS);
    (first, last)
}

/// Page-window for Floor's mixed header/tile rows. Returns
/// `(first, last, height_before_first, height_of_drawn_slice)` so the list can
/// expand to full content height while only painting on-screen rows.
#[cfg(test)]
fn floor_page_window(top: f64, rows: &[FloorRow], gap: f64) -> (usize, usize, f64, f64) {
    let n = rows.len();
    if n == 0 {
        return (0, 0, 0.0, 0.0);
    }
    let pitches: Vec<f64> = rows.iter().map(floor_row_pitch).collect();
    let mut starts = Vec::with_capacity(n);
    let mut y = top;
    for (i, &h) in pitches.iter().enumerate() {
        starts.push(y);
        y += h + if i + 1 < n { gap } else { 0.0 };
    }
    let (vp_top, vp_h) = *page_viewport().lock().unwrap();
    let (first, last) = if vp_h <= 1.0 {
        (0, n.min(6))
    } else {
        let vp_bot = vp_top + vp_h;
        let mut first = 0;
        while first < n && starts[first] + pitches[first] < vp_top {
            first += 1;
        }
        let mut last = first;
        while last < n && starts[last] < vp_bot {
            last += 1;
        }
        (first.saturating_sub(1), (last + 1).min(n))
    };
    let before = if first == 0 {
        0.0
    } else {
        starts[first] - top
    };
    let slice = if last <= first {
        0.0
    } else {
        let end = starts[last - 1] + pitches[last - 1];
        end - starts[first]
    };
    (first, last, before, slice)
}

/// Narrowest a dish tile may get in the menu window.
/// Prefer two readable image|text cards over three cramped ones.
const PRODUCT_TILE_MIN_W: f64 = 300.0;
/// Dish tile slots per row in the DSL (`c0`…`c5`).
const PRODUCT_MAX_COLUMNS: usize = 6;

/// Horizontal gap between cards in a ProductList row (keep in sync with DSL).
const PRODUCT_TILE_GAP: f64 = 20.0;

fn product_columns(width: f64) -> usize {
    (((width + PRODUCT_TILE_GAP) / (PRODUCT_TILE_MIN_W + PRODUCT_TILE_GAP)).floor() as usize)
        .clamp(1, PRODUCT_MAX_COLUMNS)
}

/// Tiles per row the menu window's grid last drew with (taps map a row and
/// column back to a dish with it).
fn product_grid_cols() -> &'static Mutex<usize> {
    static C: OnceLock<Mutex<usize>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(2))
}

/// How many whole tiles fit across `width` (or the content width) — the
/// "Fit screen" column count. At least 1 (the 375 css px app floor shows one).
fn floor_columns_for(width: f64) -> usize {
    let w = if width.is_finite() && width > 0.0 {
        width
    } else {
        *content_width().lock().unwrap()
    };
    (((w + FLOOR_TILE_GAP) / (FLOOR_TILE_W + FLOOR_TILE_GAP)).floor() as usize).max(1)
}

/// Columns the floor grid last drew (taps map through the same count).
fn floor_columns() -> usize {
    (*floor_grid_cols().lock().unwrap()).max(1)
}

/// Floor rows in order: each area's heading, then its tables in rows of
/// `cols` (the list is already sorted area by area, D1, D2 … D10).
#[cfg(test)]
fn floor_rows(tables: &[DiningTable], cols: usize) -> Vec<FloorRow> {
    let cols = cols.max(1);
    let named = tables.iter().any(|t| !t.area.is_empty());
    let mut rows = Vec::new();
    let mut i = 0;
    while i < tables.len() {
        let area = tables[i].area.clone();
        if named {
            rows.push(FloorRow::Area(area.clone()));
        }
        let group: Vec<usize> = (i..tables.len()).take_while(|&j| tables[j].area == area).collect();
        i += group.len();
        for chunk in group.chunks(cols) {
            rows.push(FloorRow::Tiles(chunk.to_vec()));
        }
    }
    rows
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
    /// Settings → Sidebar: order + show/hide chrome peers.
    SidebarSettings,
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
/// Team is Settings → Team (not a chrome peer).
const NAV_TABS: [NavTab; 14] = [
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

/// Bundled colored WebP for a nav tab (`resources/nav/<key>.webp`).
/// Makepad cannot paint Noto Color Emoji in colour — real images only.
fn nav_tab_icon_bytes(tab: NavTab) -> &'static [u8] {
    match tab {
        NavTab::Books => include_bytes!("../resources/nav/books.webp").as_slice(),
        NavTab::Sell => include_bytes!("../resources/nav/sell.webp").as_slice(),
        NavTab::Buy => include_bytes!("../resources/nav/buy.webp").as_slice(),
        NavTab::Menu => include_bytes!("../resources/nav/menu.webp").as_slice(),
        NavTab::Floor => include_bytes!("../resources/nav/floor.webp").as_slice(),
        NavTab::Kitchen => include_bytes!("../resources/nav/kitchen.webp").as_slice(),
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
    let icon_right = widget_rect.pos.x + widget_rect.size.x;
    let position = if widget_rect.pos.x < 120.0 {
        TooltipPosition::Right
    } else if icon_right > 640.0 {
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

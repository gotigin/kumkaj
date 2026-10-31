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
    More,
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
    Rates,
    Ops,
    Cod,
    Riders,
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

/// Which Reports-screen facet is open (ops/COD/riders share `Screen::Reports`).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ReportsNav {
    #[default]
    Bank,
    Ops,
    Cod,
    Riders,
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

/// A field whose value is usually something the counter already has. Each
/// has a [`SuggestList`] right under its input (see `suggest.rs`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SuggestField {
    BookingDestination,
    BookingZone,
    BookingCarrier,
    BookingOrigin,
    CustomerName,
    MenuUnit,
    MenuZone,
    MenuCarrier,
    MenuParty,
    MenuTax,
    MenuCategory,
    BuyItem,
    BuySupplier,
    InvItem,
    InvSupplier,
    CreditParty,
    TeamPick,
    Country,
    ConvFrom,
    ConvTo,
    MeasureDim,
    MeasureWeight,
    PrinterTarget,
    RouteAddPrinter,
    AwbPoolCarrier,
    OpsRider,
    OpsCarrier,
    OpsParty,
    /// Parties form: country / state / pincode for the address block.
    PartyCountry,
    PartyState,
    PartyPincode,
}

impl SuggestField {
    const ALL: [SuggestField; 31] = [
        SuggestField::BookingDestination,
        SuggestField::BookingZone,
        SuggestField::BookingCarrier,
        SuggestField::BookingOrigin,
        SuggestField::CustomerName,
        SuggestField::MenuUnit,
        SuggestField::MenuZone,
        SuggestField::MenuCarrier,
        SuggestField::MenuParty,
        SuggestField::MenuTax,
        SuggestField::MenuCategory,
        SuggestField::BuyItem,
        SuggestField::BuySupplier,
        SuggestField::InvItem,
        SuggestField::InvSupplier,
        SuggestField::CreditParty,
        SuggestField::TeamPick,
        SuggestField::Country,
        SuggestField::ConvFrom,
        SuggestField::ConvTo,
        SuggestField::MeasureDim,
        SuggestField::MeasureWeight,
        SuggestField::PrinterTarget,
        SuggestField::RouteAddPrinter,
        SuggestField::AwbPoolCarrier,
        SuggestField::OpsRider,
        SuggestField::OpsCarrier,
        SuggestField::OpsParty,
        SuggestField::PartyCountry,
        SuggestField::PartyState,
        SuggestField::PartyPincode,
    ];

    fn key(self) -> &'static str {
        match self {
            SuggestField::BookingDestination => "booking_destination",
            SuggestField::BookingZone => "booking_zone",
            SuggestField::BookingCarrier => "booking_carrier",
            SuggestField::BookingOrigin => "booking_origin",
            SuggestField::CustomerName => "customer_name",
            SuggestField::MenuUnit => "menu_unit",
            SuggestField::MenuZone => "menu_zone",
            SuggestField::MenuCarrier => "menu_carrier",
            SuggestField::MenuParty => "menu_party",
            SuggestField::MenuTax => "menu_tax",
            SuggestField::MenuCategory => "menu_category",
            SuggestField::BuyItem => "buy_item",
            SuggestField::BuySupplier => "buy_supplier",
            SuggestField::InvItem => "inv_item",
            SuggestField::InvSupplier => "inv_supplier",
            SuggestField::CreditParty => "credit_party",
            SuggestField::TeamPick => "team_pick",
            SuggestField::Country => "country",
            SuggestField::ConvFrom => "conv_from",
            SuggestField::ConvTo => "conv_to",
            SuggestField::MeasureDim => "measure_dim",
            SuggestField::MeasureWeight => "measure_weight",
            SuggestField::PrinterTarget => "printer_target",
            SuggestField::RouteAddPrinter => "route_add_printer",
            SuggestField::AwbPoolCarrier => "awb_pool_carrier",
            SuggestField::OpsRider => "ops_rider",
            SuggestField::OpsCarrier => "ops_carrier",
            SuggestField::OpsParty => "ops_party",
            SuggestField::PartyCountry => "party_country",
            SuggestField::PartyState => "party_state",
            SuggestField::PartyPincode => "party_pincode",
        }
    }

    fn input(self) -> &'static [LiveId] {
        match self {
            SuggestField::BookingDestination => ids!(booking_destination_input),
            SuggestField::BookingZone => ids!(booking_zone_input),
            SuggestField::BookingCarrier => ids!(booking_carrier_pick_input),
            SuggestField::BookingOrigin => ids!(booking_origin_name_input),
            SuggestField::CustomerName => ids!(customer_name_input),
            SuggestField::MenuUnit => ids!(menu_unit_input),
            SuggestField::MenuZone => ids!(menu_zone_input),
            SuggestField::MenuCarrier => ids!(menu_carrier_input),
            SuggestField::MenuParty => ids!(menu_party_input),
            SuggestField::MenuTax => ids!(menu_tax_input),
            SuggestField::MenuCategory => ids!(menu_category_input),
            SuggestField::BuyItem => ids!(buy_item_input),
            SuggestField::BuySupplier => ids!(buy_supplier_input),
            SuggestField::InvItem => ids!(inv_item_input),
            SuggestField::InvSupplier => ids!(inv_supplier_input),
            SuggestField::CreditParty => ids!(credit_limit_party),
            SuggestField::TeamPick => ids!(team_pick_input),
            SuggestField::Country => ids!(country_code_input),
            SuggestField::ConvFrom => ids!(conv_from_input),
            SuggestField::ConvTo => ids!(conv_to_input),
            SuggestField::MeasureDim => ids!(measure_dim_input),
            SuggestField::MeasureWeight => ids!(measure_weight_input),
            SuggestField::PrinterTarget => ids!(printer_target_input),
            SuggestField::RouteAddPrinter => ids!(route_add_printer_input),
            SuggestField::AwbPoolCarrier => ids!(awb_pool_carrier_input),
            SuggestField::OpsRider => ids!(ops_rider_input),
            SuggestField::OpsCarrier => ids!(ops_carrier_input),
            SuggestField::OpsParty => ids!(ops_party_input),
            SuggestField::PartyCountry => ids!(party_country_input),
            SuggestField::PartyState => ids!(party_state_input),
            SuggestField::PartyPincode => ids!(party_pincode_input),
        }
    }

    fn list(self) -> &'static [LiveId] {
        match self {
            SuggestField::BookingDestination => ids!(booking_destination_suggest),
            SuggestField::BookingZone => ids!(booking_zone_suggest),
            SuggestField::BookingCarrier => ids!(booking_carrier_suggest),
            SuggestField::BookingOrigin => ids!(booking_origin_suggest),
            SuggestField::CustomerName => ids!(customer_name_suggest),
            SuggestField::MenuUnit => ids!(menu_unit_suggest),
            SuggestField::MenuZone => ids!(menu_zone_suggest),
            SuggestField::MenuCarrier => ids!(menu_carrier_suggest),
            SuggestField::MenuParty => ids!(menu_party_suggest),
            SuggestField::MenuTax => ids!(menu_tax_suggest),
            SuggestField::MenuCategory => ids!(menu_category_suggest),
            SuggestField::BuyItem => ids!(buy_item_suggest),
            SuggestField::BuySupplier => ids!(buy_supplier_suggest),
            SuggestField::InvItem => ids!(inv_item_suggest),
            SuggestField::InvSupplier => ids!(inv_supplier_suggest),
            SuggestField::CreditParty => ids!(credit_party_suggest),
            SuggestField::TeamPick => ids!(team_pick_suggest),
            SuggestField::Country => ids!(country_suggest),
            SuggestField::ConvFrom => ids!(conv_from_suggest),
            SuggestField::ConvTo => ids!(conv_to_suggest),
            SuggestField::MeasureDim => ids!(measure_dim_suggest),
            SuggestField::MeasureWeight => ids!(measure_weight_suggest),
            SuggestField::PrinterTarget => ids!(printer_target_suggest),
            SuggestField::RouteAddPrinter => ids!(route_add_printer_suggest),
            SuggestField::AwbPoolCarrier => ids!(awb_pool_carrier_suggest),
            SuggestField::OpsRider => ids!(ops_rider_suggest),
            SuggestField::OpsCarrier => ids!(ops_carrier_suggest),
            SuggestField::OpsParty => ids!(ops_party_suggest),
            SuggestField::PartyCountry => ids!(party_country_suggest),
            SuggestField::PartyState => ids!(party_state_suggest),
            SuggestField::PartyPincode => ids!(party_pincode_suggest),
        }
    }
}

/// Default sidebar / bottom-bar arrangement (matches the DSL slot order).
const NAV_TABS: [NavTab; 16] = [
    NavTab::Books,
    NavTab::Sell,
    NavTab::Buy,
    NavTab::Rates,
    NavTab::Ops,
    NavTab::Cod,
    NavTab::Riders,
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
        NavTab::Rates => "rates",
        NavTab::Ops => "ops",
        NavTab::Cod => "cod",
        NavTab::Riders => "riders",
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
        NavTab::Rates => include_bytes!("../resources/nav/rates.webp").as_slice(),
        NavTab::Ops => include_bytes!("../resources/nav/ops.webp").as_slice(),
        NavTab::Cod => include_bytes!("../resources/nav/cod.webp").as_slice(),
        NavTab::Riders => include_bytes!("../resources/nav/riders.webp").as_slice(),
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

/// Screen / nav label text. Colored mark sits in a sibling `Image`, not in the string
/// (emoji colour does not survive Makepad text).
fn nav_tab_caption(_tab: NavTab, label: &str) -> String {
    label.to_string()
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
    order
        .iter()
        .map(|t| nav_tab_key(*t))
        .collect::<Vec<_>>()
        .join(",")
}

/// Peers that must stay on the chrome (cannot be hidden in Sidebar settings).
fn nav_tab_always_visible(tab: NavTab) -> bool {
    matches!(tab, NavTab::Settings | NavTab::Lock)
}

fn parse_nav_hidden(csv: Option<&str>) -> Vec<NavTab> {
    let mut hidden = Vec::new();
    if let Some(csv) = csv {
        for key in csv.split(',') {
            let key = key.trim();
            if let Some(tab) = NAV_TABS.iter().find(|t| nav_tab_key(**t) == key)
                && !nav_tab_always_visible(*tab)
                && !hidden.contains(tab)
            {
                hidden.push(*tab);
            }
        }
    }
    hidden
}

fn nav_hidden_csv(hidden: &[NavTab]) -> String {
    hidden
        .iter()
        .filter(|t| !nav_tab_always_visible(**t))
        .map(|t| nav_tab_key(*t))
        .collect::<Vec<_>>()
        .join(",")
}

fn nav_tab_is_visible(hidden: &[NavTab], tab: NavTab) -> bool {
    nav_tab_always_visible(tab) || !hidden.contains(&tab)
}

/// Chrome slot order: `nav_order` with hidden peers removed.
fn nav_display_order(order: &[NavTab], hidden: &[NavTab]) -> Vec<NavTab> {
    order
        .iter()
        .copied()
        .filter(|t| nav_tab_is_visible(hidden, *t))
        .collect()
}

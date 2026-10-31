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

/// What the signed-in person (and this device role) may open in the chrome.
#[derive(Clone, Copy, Debug, Default)]
struct NavCaps {
    sell: bool,
    items: bool,
    sales: bool,
    credit: bool,
    floor: bool,
    kitchen: bool,
    team: bool,
    backup: bool,
    inventory: bool,
    branches: bool,
    is_owner: bool,
    is_client: bool,
    /// Tables & kitchen peers exist only when this shop runs that mode.
    floor_on: bool,
    branch_count: usize,
}

impl NavCaps {
    fn from_session(
        session: Option<&StaffMember>,
        is_client: bool,
        floor_on: bool,
        branch_count: usize,
    ) -> Self {
        match session {
            Some(s) => Self {
                sell: s.can_access_sell(),
                items: s.can_access_items(),
                sales: s.can_access_sales(),
                credit: s.can_access_credit(),
                floor: s.can_access_floor() || s.is_owner(),
                kitchen: s.can_access_kitchen() || s.is_owner(),
                team: s.can_access_team() || s.is_owner(),
                backup: s.can_access_backup() || s.is_owner(),
                inventory: s.can_access_inventory() || s.is_owner(),
                branches: s.can_access_branches() || s.is_owner(),
                is_owner: s.is_owner(),
                is_client,
                floor_on,
                branch_count,
            },
            None => Self {
                is_client,
                floor_on,
                branch_count,
                ..Self::default()
            },
        }
    }

    fn books_ok(self) -> bool {
        self.sales || self.is_owner || self.items
    }

    fn buy_ok(self) -> bool {
        self.inventory || self.sales || self.is_owner
    }
}

/// Day totals strip ("Today: …") — Owner and anyone with Sales access.
/// Waiters / cashiers by default never see the shop's day's business.
fn session_may_see_today(session: Option<&StaffMember>) -> bool {
    match session {
        Some(s) => s.is_owner() || s.can_access_sales(),
        None => false,
    }
}

/// Counter (Take payment) dock — Settle right not Denied (Owner always may).
fn session_may_settle(session: Option<&StaffMember>) -> bool {
    match session {
        Some(s) => !matches!(
            s.rights().level(domain::Right::Settle),
            domain::RightLevel::Denied
        ),
        None => false,
    }
}

/// Print bill / reprint — Reprint right not Denied.
fn session_may_reprint(session: Option<&StaffMember>) -> bool {
    match session {
        Some(s) => !matches!(
            s.rights().level(domain::Right::Reprint),
            domain::RightLevel::Denied
        ),
        None => false,
    }
}

/// True when this peer may appear for the signed-in person on this device.
fn nav_tab_permitted(tab: NavTab, caps: NavCaps) -> bool {
    match tab {
        NavTab::Lock => true,
        // Only people who can open Settings — never a dead tab (waiters on
        // an extra till used to see it and get "Not allowed").
        NavTab::Settings => caps.backup || caps.is_owner || caps.sales,
        NavTab::Sell => caps.sell,
        NavTab::Books => caps.books_ok(),
        NavTab::Buy => caps.buy_ok(),
        NavTab::Menu => caps.items,
        NavTab::Floor => caps.floor && caps.floor_on,
        NavTab::Kitchen => caps.kitchen && caps.floor_on,
        NavTab::Sales => caps.sales,
        NavTab::DayClose | NavTab::Reports => caps.sales || caps.is_owner,
        NavTab::Inventory => caps.inventory && !caps.is_client,
        NavTab::Branches => caps.branches && !caps.is_client && caps.branch_count > 1,
        NavTab::Credit => caps.credit,
        NavTab::Team => caps.team,
    }
}

/// Chrome slots: preference order, minus user-hidden, minus no-access peers.
fn nav_chrome_order(order: &[NavTab], hidden: &[NavTab], caps: NavCaps) -> Vec<NavTab> {
    nav_display_order(order, hidden)
        .into_iter()
        .filter(|t| nav_tab_permitted(*t, caps))
        .collect()
}

/// Live chrome order for paint / click mapping (preference + rights).
fn current_nav_chrome_order() -> Vec<NavTab> {
    let st = app_state().lock().unwrap();
    let branch_count = st
        .store
        .as_ref()
        .and_then(|s| s.list_branches().ok())
        .map(|b| b.len())
        .unwrap_or(1);
    let caps = NavCaps::from_session(
        st.session.as_ref(),
        st.device_role == DeviceRole::Client,
        st.venue_mode.creates_kot(),
        branch_count,
    );
    nav_chrome_order(&st.nav_order, &st.nav_hidden, caps)
}

/// Password-style mask: every non-space character becomes a bullet so the
/// string keeps roughly the same width (no peek at figures when hidden).
/// **Only** for money values — never feed labels, headings, counts, or prose.
fn mask_sensitive_text(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_whitespace() { c } else { '●' })
        .collect()
}

/// Show `text` or bullet-mask it. Prefer [`display_money`] at call sites.
fn display_sensitive(text: &str, visible: bool) -> String {
    if visible {
        text.to_string()
    } else {
        mask_sensitive_text(text)
    }
}

/// Canonical path for a **discrete money figure** (books value, today slot, etc.).
/// Labels / headings / placeholders / counts / prose must never go through this.
fn display_money(text: &str, visible: bool) -> String {
    display_sensitive(text, visible)
}

/// Split `₹50.00 (2)` → (`₹50.00`, Some(`(2)`)); plain values keep no count.
fn split_trailing_count(v: &str) -> (&str, Option<&str>) {
    let t = v.trim();
    if let Some((rest, count)) = t.rsplit_once(' ')
        && count.starts_with('(')
        && count.ends_with(')')
        && count.len() > 2
        && count[1..count.len() - 1]
            .chars()
            .all(|c| c.is_ascii_digit())
    {
        (rest, Some(count))
    } else {
        (t, None)
    }
}

/// True when `v` is a formatted money amount (`₹120.00`, `€1 234,50`, `CHF1'234.50`),
/// optionally with a trailing `(N)` count. Rejects bare counts (`"3"`), times
/// (`12:30`), prose (`3 parcels`), and invoice-like ids.
fn looks_like_money_value(v: &str) -> bool {
    let (core, _) = split_trailing_count(v);
    if core.is_empty() {
        return false;
    }
    let rest = core
        .strip_prefix('-')
        .or_else(|| core.strip_prefix('+'))
        .unwrap_or(core);
    let Some(digit_start) = rest.find(|c: char| c.is_ascii_digit()) else {
        return false;
    };
    let prefix = &rest[..digit_start];
    let number = &rest[digit_start..];
    // `{symbol}{number}` — symbol is short (₹, $, CHF, KSh, د.ك) and has no spaces.
    if prefix.is_empty()
        || prefix.chars().count() > 5
        || prefix.contains(|c: char| c.is_whitespace())
        || prefix.contains(':')
        || prefix.contains('·')
        || prefix.contains('→')
    {
        return false;
    }
    number
        .chars()
        .all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == '\'' || c == ' ')
        && number.chars().any(|c| c.is_ascii_digit())
}

/// Mask only whitespace-separated money tokens; leave labels and prose.
fn mask_money_tokens(text: &str) -> String {
    text.split_inclusive(char::is_whitespace)
        .map(|part| {
            let trimmed_end = part.trim_end();
            let ws = &part[trimmed_end.len()..];
            if looks_like_money_value(trimmed_end) {
                format!("{}{}", mask_sensitive_text(trimmed_end), ws)
            } else {
                part.to_string()
            }
        })
        .collect()
}

/// Mask a Kv **value**: whole money figures (incl. space-grouped) → bullets;
/// keep trailing `(N)` counts and any non-money text.
fn mask_kv_amount(v: &str) -> String {
    let (core, count) = split_trailing_count(v);
    if looks_like_money_value(core) {
        let masked = mask_sensitive_text(core);
        match count {
            Some(c) => format!("{masked} {c}"),
            None => masked,
        }
    } else {
        mask_money_tokens(v)
    }
}

const BOOKS_SECTION_KEYS: &[&str] = &["today", "bills", "payments", "outstanding", "bank"];

fn parse_books_sections(csv: Option<&str>) -> Vec<String> {
    let Some(raw) = csv.filter(|s| !s.trim().is_empty()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for key in raw.split(',') {
        let key = key.trim();
        if BOOKS_SECTION_KEYS.contains(&key) && !out.iter().any(|k| k == key) {
            out.push(key.to_string());
        }
    }
    out
}

fn books_sections_csv(open: &[String]) -> String {
    open.join(",")
}

fn parse_amount_keys_hidden(csv: Option<&str>) -> Vec<String> {
    let Some(raw) = csv.filter(|s| !s.trim().is_empty()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for key in raw.split(',') {
        let key = key.trim();
        // Books slots (`books_in`, …) and report Kv keys (`kv:Bank`, …).
        // Commas cannot appear inside a key (CSV delimiter).
        if key.is_empty() {
            continue;
        }
        if !out.iter().any(|k| k == key) {
            out.push(key.to_string());
        }
    }
    out
}

/// Stable amount-visibility key for a ReportText Kv money row.
fn report_kv_amount_key(label: &str) -> String {
    format!("kv:{}", label.trim().replace(',', " "))
}

/// Persist a per-figure eye toggle for the signed-in user (no UI sync).
fn toggle_amount_key_prefs(key: &str) {
    let mut st = app_state().lock().unwrap();
    let user = st.session.as_ref().map(lan_key);
    if st.amounts_screen_hidden {
        if let Some(i) = st.amount_keys_peek.iter().position(|k| k == key) {
            st.amount_keys_peek.remove(i);
        } else {
            st.amount_keys_peek.push(key.to_string());
        }
        if let (Some(store), Some(user)) = (st.store.as_ref(), user.as_deref()) {
            let _ = store.set_amount_keys_peek_csv(
                user,
                &amount_keys_hidden_csv(&st.amount_keys_peek),
            );
        }
    } else {
        if let Some(i) = st.amount_keys_hidden.iter().position(|k| k == key) {
            st.amount_keys_hidden.remove(i);
        } else {
            st.amount_keys_hidden.push(key.to_string());
        }
        if let (Some(store), Some(user)) = (st.store.as_ref(), user.as_deref()) {
            let _ = store.set_amount_keys_hidden_csv(
                user,
                &amount_keys_hidden_csv(&st.amount_keys_hidden),
            );
        }
    }
}

fn amount_keys_hidden_csv(hidden: &[String]) -> String {
    hidden.join(",")
}

/// Whether one money **value** key should show its digits right now.
///
/// Encodes the screen-eye vs per-figure rules (see `docs/DEV.md` § Amount visibility):
/// - Screen icon **hidden** (`screen_hidden`) → values hidden by default; only keys in
///   `keys_peek` (session) are temporarily revealed.
/// - Screen icon **visible** → apply each figure's own persisted state: hide only keys
///   in `keys_hidden`. The screen eye does **not** force every figure visible.
///
/// Labels / headings / placeholders / counts / prose are out of scope — callers must
/// mask money only ([`display_money`], [`paint_amount_value`], ReportText Kv eyes).
fn amount_key_visible(
    screen_hidden: bool,
    keys_hidden: &[String],
    keys_peek: &[String],
    key: &str,
) -> bool {
    if screen_hidden {
        keys_peek.iter().any(|k| k == key)
    } else {
        !keys_hidden.iter().any(|k| k == key)
    }
}

/// Font Awesome solid (`WorldFonts` icons member).
/// Shown amounts: eye. Hidden amounts: lock (eye-slash looked odd at 14px).
const EYE_MARK_OPEN: &str = "\u{f06e}";
const EYE_MARK_HIDDEN: &str = "\u{f023}";

fn eye_glyph(visible: bool) -> &'static str {
    if visible {
        EYE_MARK_OPEN
    } else {
        EYE_MARK_HIDDEN
    }
}

/// Placeholder like `Phone (+91)` from the active country pack.
fn phone_placeholder(label: &str, country_code: &str) -> String {
    let dial = fiscal::calling_code(country_code);
    if dial.is_empty() {
        label.to_string()
    } else {
        format!("{label} ({dial})")
    }
}

/// Prepend the pack's calling code when the typed phone has no `+`.
fn with_calling_code(phone: &str, country_code: &str) -> String {
    let p = phone.trim();
    if p.is_empty() {
        return String::new();
    }
    if p.starts_with('+') {
        return p.to_string();
    }
    let dial = fiscal::calling_code(country_code);
    if dial.is_empty() {
        return p.to_string();
    }
    let local = p.trim_start_matches('0').trim();
    if local.is_empty() {
        return p.to_string();
    }
    format!("{dial}{local}")
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
        Screen::Settings | Screen::SidebarSettings => Some(NavTab::Settings),
        Screen::Lock => Some(NavTab::Lock),
        Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair => None,
    }
}

/// Whether arriving at `screen` may keep (or receive) drill-down history.
/// Lock and Onboard always clear stacks. Chrome peers still use `go_peer`
/// (clear then land with `record_history: false`); only `go` pushes.
/// Mouse Back / Alt+← / edge swipe / Android back pop via `navigate_back`.
fn screen_keeps_history(screen: Screen) -> bool {
    !matches!(screen, Screen::Lock | Screen::Onboard)
}

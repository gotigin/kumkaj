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

const BOOKS_SECTION_KEYS: &[&str] = &["today", "bookings", "payments", "outstanding", "bank"];

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

/// Resolve a typed country name or ISO code to a pack code.
fn country_code_from_label(label: &str) -> String {
    let t = label.trim();
    if t.is_empty() {
        return String::new();
    }
    let upper = t.to_ascii_uppercase();
    if let Some(c) = fiscal::COUNTRY_PROFILES
        .iter()
        .find(|c| c.code.eq_ignore_ascii_case(&upper))
    {
        return c.code.to_string();
    }
    if let Some(c) = fiscal::COUNTRY_PROFILES
        .iter()
        .find(|c| c.name_en.eq_ignore_ascii_case(t))
    {
        return c.code.to_string();
    }
    upper
}

/// Street + state + PIN + country → one address line for storage.
fn compose_party_address(street: &str, state: &str, pin: &str, country: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    let street = street.trim();
    let state = state.trim();
    let pin = pin.trim();
    let country = country.trim();
    if !street.is_empty() {
        parts.push(street);
    }
    if !state.is_empty() {
        parts.push(state);
    }
    if !pin.is_empty() {
        parts.push(pin);
    }
    if !country.is_empty() {
        parts.push(country);
    }
    parts.join(", ")
}

/// Best-effort split of a stored address into street / state / pin / country.
fn split_party_address(address: &str) -> (String, String, String, String) {
    let raw = address.trim();
    if raw.is_empty() {
        return (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        );
    }
    let mut parts: Vec<&str> = raw
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return (raw.to_string(), String::new(), String::new(), String::new());
    }
    let mut country = String::new();
    let mut state = String::new();
    let mut pin = String::new();
    // Trailing country name/code.
    if let Some(last) = parts.last().copied() {
        if !country_code_from_label(last).is_empty()
            && fiscal::COUNTRY_PROFILES.iter().any(|c| {
                c.code.eq_ignore_ascii_case(last) || c.name_en.eq_ignore_ascii_case(last)
            })
        {
            country = last.to_string();
            parts.pop();
        }
    }
    // Trailing PIN / ZIP (digits, optionally with spaces).
    if let Some(last) = parts.last().copied() {
        let digits: String = last.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 && digits.len() <= 10 && digits.len() >= last.len().saturating_sub(2) {
            pin = last.to_string();
            parts.pop();
        }
    }
    // Trailing admin area when we know the country pack.
    if let Some(last) = parts.last().copied() {
        let code = if country.is_empty() {
            String::new()
        } else {
            country_code_from_label(&country)
        };
        let areas = if code.is_empty() {
            &[][..]
        } else {
            fiscal::admin_areas(&code)
        };
        if areas.iter().any(|a| a.eq_ignore_ascii_case(last)) {
            state = last.to_string();
            parts.pop();
        }
    }
    (parts.join(", "), state, pin, country)
}

fn nav_tab_for(screen: Screen, reports_nav: ReportsNav) -> Option<NavTab> {
    match screen {
        Screen::Books => Some(NavTab::Books),
        Screen::Pos => Some(NavTab::Sell),
        Screen::Buy => Some(NavTab::Buy),
        Screen::Menu | Screen::More => Some(NavTab::Rates),
        Screen::Sales => Some(NavTab::Sales),
        Screen::DayClose => Some(NavTab::DayClose),
        Screen::Reports => Some(match reports_nav {
            ReportsNav::Bank => NavTab::Reports,
            ReportsNav::Ops => NavTab::Ops,
            ReportsNav::Cod => NavTab::Cod,
            ReportsNav::Riders => NavTab::Riders,
        }),
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

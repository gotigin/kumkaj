/// "5 × Thali · 3 × Chaas": what the kitchen still has to cook.
/// What the owner must hear about the books' safety on the device that
/// holds them: a failing backup, none for days, a disk filling up. Empty
/// when all is well.
/// The shop's own words for a printer test page — its saved name and the
/// app's language — so the owner sees their script print, not only English.
fn test_page_own_words(st: &AppState) -> String {
    let mut lines = vec![st.venue_title.trim().to_string()];
    let phrase = st.t("print_test_page");
    if phrase != "Print a test page" {
        lines.push(phrase);
    }
    lines.into_iter().filter(|l| !l.is_empty()).map(|l| l + "\n").collect()
}

fn parse_printer_route_key(key: &str) -> print_share::RouteKey<'_> {
    let key = key.trim();
    if key == "bill" {
        print_share::RouteKey::Bill
    } else if let Some(rest) = key.strip_prefix("kot:") {
        print_share::RouteKey::Kot { station_id: rest }
    } else {
        print_share::RouteKey::Bill
    }
}

/// What the owner sees in the route field — never a raw `kot:<uuid>`.
fn printer_route_display_name(
    key: &str,
    stations: &[domain::KitchenStation],
    t: impl Fn(&str) -> String,
) -> String {
    let key = key.trim();
    if key == "bill" {
        return t("route_bill");
    }
    if key == "kot:" || key.is_empty() {
        return t("route_default_kitchen");
    }
    if let Some(id) = key.strip_prefix("kot:") {
        if let Some(s) = stations.iter().find(|s| s.id == id) {
            return s.name.clone();
        }
    }
    key.to_string()
}

/// Normalize place Outputs: fill missing profile, one printer once.
/// Virtual-printer attach slots (bare VP ids) pass through unchanged —
/// rewriting them as printer_id@format deletes station attaches.
fn normalize_focused_route(st: &mut AppState) -> bool {
    let key_str = st.printer_route_key.clone();
    let key = parse_printer_route_key(&key_str);
    let default_fmt = st.printer_draft.format_id_for(key);
    let before = st.printer_draft.route_for(key).to_vec();
    let known: std::collections::HashSet<String> = st
        .printer_draft
        .printers
        .iter()
        .map(|p| p.id.clone())
        .collect();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for slot in &before {
        let raw = slot.trim();
        if raw.is_empty() {
            continue;
        }
        if st.printer_draft.virtual_printer_by_id(raw).is_some() {
            if seen.insert(raw.to_string()) {
                out.push(raw.to_string());
            }
            continue;
        }
        let (pid, fmt) = print_share::DevicePrinters::decode_route_slot(slot);
        if pid.is_empty() || !known.contains(&pid) || !seen.insert(pid.clone()) {
            continue;
        }
        let fmt = if fmt.is_empty() {
            default_fmt.clone()
        } else {
            fmt
        };
        out.push(print_share::DevicePrinters::encode_route_slot(&pid, &fmt));
    }
    if out == before {
        return false;
    }
    *st.printer_draft.route_mut(key) = out;
    let _ = st.save_printer_draft();
    true
}

/// Checklist: pick one real printer for a new virtual printer.
/// `subtitle` = printer id. Selected row uses "selected" action.
fn printer_assign_suggest_rows(
    draft: &print_share::DevicePrinters,
    selected_id: Option<&str>,
    t: impl Fn(&str) -> String,
) -> Vec<suggest::SuggestRow> {
    use suggest::SuggestRow;
    let use_l = t("use");
    let sel_l = t("route_tap_selected");
    let mut rows = Vec::new();
    for p in &draft.printers {
        let name = printer_display_name(p, &t);
        let action = if selected_id == Some(p.id.as_str()) {
            sel_l.clone()
        } else {
            use_l.clone()
        };
        rows.push(SuggestRow::new(name, p.id.clone()).with_action(action));
    }
    if rows.is_empty() {
        rows.push(SuggestRow::new(t("printers_pool_empty"), String::new()));
    }
    rows
}

/// Select the real printer used when creating/editing a virtual printer.
fn select_real_printer_for_virtual(st: &mut AppState, printer_id: &str) -> bool {
    let pid = printer_id.trim();
    if pid.is_empty() || st.printer_draft.printer_by_id(pid).is_none() {
        return false;
    }
    st.printer_selected_id = Some(pid.to_string());
    if let Some(id) = st.virtual_printer_sel.clone() {
        if let Some(v) = st.printer_draft.virtual_printers.iter_mut().find(|v| v.id == id) {
            v.printer_id = pid.to_string();
            let _ = st.save_printer_draft();
        }
    }
    true
}

fn virtual_printers_list_rows(
    draft: &print_share::DevicePrinters,
    selected: Option<&str>,
    t: impl Fn(&str) -> String,
) -> Vec<ReportRow> {
    let mut rows = Vec::new();
    for v in &draft.virtual_printers {
        let mark = if selected == Some(v.id.as_str()) {
            "› "
        } else {
            ""
        };
        let real = draft
            .printer_by_id(&v.printer_id)
            .map(|p| printer_display_name(p, &t))
            .unwrap_or_else(|| v.printer_id.clone());
        let vals = v.values();
        let layout = if vals.landscape {
            t("config_layout_landscape")
        } else {
            t("config_layout_portrait")
        };
        let delivery = match vals.delivery.as_str() {
            print_share::DELIVERY_RAW_DEVICE => t("vp_delivery_raw"),
            print_share::DELIVERY_ESCPOS => t("vp_delivery_escpos"),
            _ => t("vp_delivery_spooler"),
        };
        rows.push(ReportRow::Line(format!(
            "{mark}{} — {real} · {layout} · {delivery}",
            v.name,
        )));
    }
    if rows.is_empty() {
        rows.push(ReportRow::Line(t("virtual_printers_empty")));
    }
    rows
}

/// Checklist rows: every virtual printer; checked = attached to focused station/place.
/// `subtitle` = virtual printer id. Checked action = `dont_use` (tap clears); else `use`.
fn station_virtual_printer_suggest_rows(
    draft: &print_share::DevicePrinters,
    route_key: &str,
    t: impl Fn(&str) -> String,
) -> Vec<suggest::SuggestRow> {
    use suggest::SuggestRow;
    let key = parse_printer_route_key(route_key);
    let attached: Vec<String> = draft
        .route_for(key)
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let use_l = t("use");
    let clear_l = t("dont_use");
    let attached_ids: Vec<String> = attached
        .iter()
        .map(|slot| {
            draft
                .virtual_printer_by_id(slot)
                .map(|v| v.printer_id.clone())
                .unwrap_or_else(|| print_share::DevicePrinters::decode_route_slot(slot).0)
        })
        .filter(|id| !id.is_empty())
        .collect();
    let mut rows = Vec::new();
    for pid in &attached_ids {
        if let Some(p) = draft.printer_by_id(pid) {
            rows.push(
                SuggestRow::new(printer_display_name(p, &t), p.id.clone()).with_action(clear_l.clone()),
            );
        }
    }
    for p in &draft.printers {
        if attached_ids.iter().any(|id| id == &p.id) {
            continue;
        }
        rows.push(
            SuggestRow::new(printer_display_name(p, &t), p.id.clone()).with_action(use_l.clone()),
        );
    }
    if rows.is_empty() {
        rows.push(SuggestRow::new(t("station_vps_empty"), String::new()));
    }
    rows
}

/// Toggle VP on the focused station route (`printer_route_key`).
fn toggle_station_virtual_printer(st: &mut AppState, vp_id: &str) -> Result<String, String> {
    if st.device_role == DeviceRole::Client {
        return Err(st.t("printers_host_only"));
    }
    let key_str = st.printer_route_key.clone();
    let key_trim = key_str.trim();
    if key_trim != "bill" && !key_trim.starts_with("kot:") {
        return Err(st.t("station_print_pick_first"));
    }
    let key = parse_printer_route_key(&key_str);
    match st.printer_draft.toggle_printer_on_route(key, vp_id) {
        Some(true) => {
            st.save_printer_draft()?;
            Ok(st.t("station_vp_attached"))
        }
        Some(false) => {
            st.save_printer_draft()?;
            Ok(st.t("station_vp_detached"))
        }
        None => Err(st.t("station_vps_empty")),
    }
}

/// Reorder attached VPs on the focused station from checklist row indices.
fn reorder_station_virtual_priority(st: &mut AppState, from_row: usize, to_row: usize) -> bool {
    if st.device_role == DeviceRole::Client {
        return false;
    }
    let key_str = st.printer_route_key.clone();
    let key_trim = key_str.trim();
    if key_trim != "bill" && !key_trim.starts_with("kot:") {
        return false;
    }
    let clear_l = st.t("dont_use");
    let from_id = st
        .suggest_rows
        .get(from_row)
        .filter(|r| r.action == clear_l)
        .map(|r| r.subtitle.clone());
    let to_id = st
        .suggest_rows
        .get(to_row)
        .filter(|r| r.action == clear_l)
        .map(|r| r.subtitle.clone());
    let (Some(from_id), Some(to_id)) = (from_id, to_id) else {
        return false;
    };
    let key = parse_printer_route_key(&key_str);
    let route = st.printer_draft.route_mut(key);
    let from = route.iter().position(|s| s.trim() == from_id);
    let to = route.iter().position(|s| s.trim() == to_id);
    let (Some(from), Some(to)) = (from, to) else {
        return false;
    };
    if !st.printer_draft.swap_route_slots(key, from, to) {
        return false;
    }
    let _ = st.save_printer_draft();
    true
}

/// Bill + each kitchen station as (route key, label, selected) for the place
/// dropdown (`SuggestField::PrinterPlace`).
fn printer_place_choices(
    stations: &[domain::KitchenStation],
    focus: &str,
    t: impl Fn(&str) -> String,
) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    out.push(("bill".into(), t("route_bill"), focus.trim() == "bill"));
    for s in stations.iter().take(ROUTE_PLACE_BTNS.len().saturating_sub(1)) {
        let key = format!("kot:{}", s.id);
        let selected = focus.trim() == key;
        out.push((key, s.name.clone(), selected));
    }
    out
}

/// Kitchen destinations for unsent cart lines: `(station display name, line count)`.
/// Order follows `priority` station ids when provided; else first-seen.
/// Empty station → `unassigned` label (caller supplies it).
fn kot_order_ids(fmt: &print_share::KotSlipFormat) -> Vec<String> {
    if fmt.field_order.is_empty() {
        vec![
            "time".into(),
            "kot_no".into(),
            "area".into(),
            "table".into(),
            "shop".into(),
            "waiter".into(),
            "station".into(),
        ]
    } else {
        fmt.field_order.clone()
    }
}

fn ensure_kot_field_order(fmt: &mut print_share::KotSlipFormat) {
    if fmt.field_order.is_empty() {
        fmt.field_order = kot_order_ids(fmt);
    }
}

/// Human label for a KOT field-order id (Settings list — not raw `kot_no`).
fn kot_field_order_label(id: &str, t: &impl Fn(&str) -> String) -> String {
    match id {
        "time" => t("kot_field_time"),
        "kot_no" => t("kot_field_kot_no"),
        "area" => t("kot_field_area"),
        "table" => t("kot_field_table"),
        "shop" => t("kot_field_shop"),
        "waiter" => t("kot_field_waiter"),
        "station" => t("kot_field_station"),
        other => other.to_string(),
    }
}

fn pending_kot_destinations(
    products: &[domain::Product],
    stations: &[domain::KitchenStation],
    lines: &[domain::CartLine],
    unassigned: &str,
) -> Vec<(String, usize)> {
    pending_kot_destinations_ordered(products, stations, lines, unassigned, &[])
}

fn pending_kot_destinations_ordered(
    products: &[domain::Product],
    stations: &[domain::KitchenStation],
    lines: &[domain::CartLine],
    unassigned: &str,
    priority: &[String],
) -> Vec<(String, usize)> {
    let mut out: Vec<(String, String, usize)> = Vec::new(); // id, name, count
    for line in lines.iter().filter(|l| !l.is_sent() && !l.hold_send) {
        let sid = line.kitchen_station_id(products);
        let name = if sid.is_empty() {
            unassigned.to_string()
        } else {
            stations
                .iter()
                .find(|s| s.id == sid)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| unassigned.to_string())
        };
        match out.iter_mut().find(|(id, _, _)| id == sid) {
            Some((_, _, n)) => *n += 1,
            None => out.push((sid.to_string(), name, 1)),
        }
    }
    if !priority.is_empty() {
        let mut ordered = Vec::with_capacity(out.len());
        for id in priority {
            if let Some(i) = out.iter().position(|(sid, _, _)| sid == id) {
                ordered.push(out.remove(i));
            }
        }
        // Unassigned (empty id) and any unknown stations keep relative order after.
        ordered.append(&mut out);
        out = ordered;
    }
    out.into_iter().map(|(_, name, n)| (name, n)).collect()
}

/// Short line for the Send button / status: kitchen names only
/// (`Tandoor · Cold`). Counts live on the station list rows, not here.
fn format_kot_destinations(dests: &[(String, usize)]) -> String {
    dests
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn printers_found_line(t: impl Fn(&str) -> String) -> String {
    // Cancel is flagged immediately — prefer that over Looking… so the
    // label updates on the Cancel tap (worker may still be joining).
    if print_share::find_printers_was_cancelled() && print_share::is_finding_printers() {
        return t("cancelled");
    }
    if print_share::is_finding_printers() {
        return t("printers_looking");
    }
    if !print_share::printers_find_finished() {
        return t("printers_tap_find");
    }
    let found = print_share::found_printers();
    if found.is_empty() {
        if print_share::find_printers_was_cancelled() {
            t("cancelled")
        } else {
            t("printers_none_found")
        }
    } else {
        // Names are the vertical pool below. This line must not repeat them.
        String::new()
    }
}

/// Merge every name from the last successful Find into the device pool and
/// save. Returns `(newly_added, total_found)`. Does not remove printers that
/// Find missed (offline / typed host:9100 stay).
fn sync_found_printers_into_pool(st: &mut AppState) -> Result<(usize, usize), String> {
    let found = print_share::found_printers();
    let total = found.len();
    if total == 0 {
        return Ok((0, 0));
    }
    let mut newly = 0usize;
    for name in &found {
        let (_id, added) = st.printer_draft.ensure_printer(name, name);
        if added {
            newly += 1;
        }
    }
    st.printer_draft.dedupe_printers();
    st.save_printer_draft()?;
    Ok((newly, total))
}

/// Display name for a pool printer: OS / Find name (or typed address).
fn printer_display_name(p: &print_share::NamedPrinter, t: impl Fn(&str) -> String) -> String {
    let name = p.target.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    let label = p.label.trim();
    if !label.is_empty() {
        return label.to_string();
    }
    t("printer_target_default")
}

/// The printers added on this device. `selected` marks the row Remove deletes.
fn printers_pool_rows(
    draft: &print_share::DevicePrinters,
    selected: Option<&str>,
    t: impl Fn(&str) -> String,
) -> Vec<ReportRow> {
    if draft.printers.is_empty() {
        return vec![ReportRow::Line(t("printers_pool_empty"))];
    }
    draft
        .printers
        .iter()
        .map(|p| {
            let mark = if selected == Some(p.id.as_str()) {
                "› "
            } else {
                ""
            };
            ReportRow::Line(format!("{mark}{}", printer_display_name(p, &t)))
        })
        .collect()
}

/// Settings → areas / stations / tax rates On/Off (same pattern as units).
/// Value column uses short On/Off pills (tap the row to flip).
fn sibling_pickers_rows(store: &Store, t: impl Fn(&str) -> String) -> Vec<ReportRow> {
    let on_off = |on: bool| {
        if on {
            format!("● {}", t("unit_picker_on"))
        } else {
            format!("○ {}", t("unit_picker_off"))
        }
    };
    let mut rows = Vec::new();
    rows.push(ReportRow::Head("Areas".into()));
    for (name, on) in store.list_picker_areas().unwrap_or_default() {
        rows.push(ReportRow::Kv(format!("area:{name}"), on_off(on)));
    }
    rows.push(ReportRow::Head("Stations".into()));
    for s in store.list_stations().unwrap_or_default() {
        rows.push(ReportRow::Kv(
            format!("station:{} · {}", s.id, s.name),
            on_off(s.picker_enabled),
        ));
    }
    rows.push(ReportRow::Head("Tax rates".into()));
    for r in store.list_tax_rates().unwrap_or_default() {
        rows.push(ReportRow::Kv(
            format!("tax:{} · {}", r.id, r.name),
            on_off(r.picker_enabled),
        ));
    }
    rows
}

/// Settings → Units as rows: each family's units from smallest to largest
/// with what one of them is in the family's anchor (what books record),
/// On/Off for pickers, units not linked to a standard yet, then the
/// conversions grouped by the unit they start from.
fn units_report_rows(store: &Store, t: impl Fn(&str) -> String) -> Vec<ReportRow> {
    let units = store.list_units().unwrap_or_default();
    let convs = store.list_unit_conversions().unwrap_or_default();
    let on_off = |u: &domain::UnitDef| {
        if u.picker_enabled {
            format!("● {}", t("unit_picker_on"))
        } else {
            format!("○ {}", t("unit_picker_off"))
        }
    };
    let name = |u: &domain::UnitDef| {
        if u.builtin {
            format!("{} — {}", u.code, u.name)
        } else {
            format!("{} — {} (custom)", u.code, u.name)
        }
    };
    let mut rows = Vec::new();
    for family in ["weight", "length", "volume", "count"] {
        let anchor = domain::unit_family_anchor(family).unwrap_or("");
        let mut members: Vec<(rust_decimal::Decimal, ReportRow)> = units
            .iter()
            .filter_map(|u| {
                let (unit_family, factor) = store.resolve_unit(&u.code)?;
                (unit_family == family).then(|| {
                    let detail = if u.code == anchor {
                        "base unit".to_string()
                    } else {
                        format!("1 {} = {} {anchor}", u.code, precise::format_kg(factor))
                    };
                    (
                        factor,
                        ReportRow::Kv(format!("{} · {detail}", name(u)), on_off(u)),
                    )
                })
            })
            .collect();
        if members.is_empty() {
            continue;
        }
        members.sort_by(|a, b| a.0.cmp(&b.0));
        let mut title = family.to_string();
        title[..1].make_ascii_uppercase();
        rows.push(ReportRow::Head(format!("{title} ({anchor})")));
        rows.extend(members.into_iter().map(|(_, row)| row));
    }
    let unlinked: Vec<ReportRow> = units
        .iter()
        .filter(|u| store.resolve_unit(&u.code).is_none())
        .map(|u| ReportRow::Kv(name(u), on_off(u)))
        .collect();
    if !unlinked.is_empty() {
        rows.push(ReportRow::Head("Not linked to a standard unit yet".into()));
        rows.extend(unlinked);
    }
    if !convs.is_empty() {
        rows.push(ReportRow::Head("Conversions (1 from = factor to)".into()));
        for group in convs.chunk_by(|a, b| a.from_code == b.from_code) {
            let targets = group
                .iter()
                .map(|c| {
                    let factor = precise::format_kg(
                        rust_decimal::Decimal::from(c.factor_milli) / rust_decimal::Decimal::from(1000),
                    );
                    format!("{factor} {}", c.to_code)
                })
                .collect::<Vec<_>>()
                .join(" · ");
            rows.push(ReportRow::Line(format!("1 {} = {targets}", group[0].from_code)));
        }
    }
    rows
}

fn backup_warning(st: &AppState) -> String {
    let Some(store) = st.store.as_ref().filter(|_| st.device_role.is_host()) else {
        return String::new();
    };
    let Ok(health) = store.backup_health() else {
        return String::new();
    };
    let mut out = Vec::new();
    if !health.error.is_empty() {
        out.push(st.t("backup_failed").replace("{e}", &health.error));
    } else if let Some(days) = health.days_since.filter(|d| *d >= 2) {
        out.push(st.t("backup_overdue").replace("{n}", &days.to_string()));
    }
    if health.low_space {
        out.push(st.t("disk_low").replace("{mb}", &health.free_mb.unwrap_or(0).to_string()));
    }
    // Every backup kept on this one device goes with it if it is lost.
    if let Some(days) = health.copy_overdue_days {
        out.push(st.t("backup_copy_overdue").replace("{n}", &days.to_string()));
    }
    out.join(" ")
}

fn kitchen_counts_line(store: &Store) -> String {
    store
        .kitchen_all_day()
        .unwrap_or_default()
        .iter()
        .map(|(name, milli)| format!("{} × {name}", milli_as_count(*milli)))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// 2000 → "2", 1500 → "1.5" (order quantities in thousandths).
fn milli_as_count(milli: i64) -> String {
    if milli % 1000 == 0 {
        format!("{}", milli / 1000)
    } else {
        format!("{:.2}", milli as f64 / 1000.0).trim_end_matches('0').to_string()
    }
}


/// Turn a typed/picked label (or a stored key) into `bill` / `kot:` / `kot:<id>`.
fn resolve_printer_route_key(
    typed: &str,
    stations: &[domain::KitchenStation],
    t: impl Fn(&str) -> String,
) -> String {
    let typed = typed.trim();
    if typed.is_empty() {
        return "bill".into();
    }
    if typed == "bill" || typed == "kot:" || typed.starts_with("kot:") {
        return typed.to_string();
    }
    let bill = t("route_bill");
    let unassigned = t("route_default_kitchen");
    if typed.eq_ignore_ascii_case(&bill) {
        return "bill".into();
    }
    if typed.eq_ignore_ascii_case(&unassigned)
        || typed.eq_ignore_ascii_case("unassigned")
        || typed.eq_ignore_ascii_case("unassigned items (no station)")
    {
        return "kot:".into();
    }
    if let Some(s) = stations
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(typed))
    {
        return format!("kot:{}", s.id);
    }
    // Unknown label — keep editing Bill rather than inventing a key.
    "bill".into()
}

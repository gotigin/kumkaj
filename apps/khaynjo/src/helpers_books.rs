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

/// Stored key `bill` / `kot:` / `kot:<id>` → [`print_share::RouteKey`].
#[allow(dead_code)]
fn printer_route_key_as_route(key: &str) -> print_share::RouteKey<'_> {
    let key = key.trim();
    if key == "bill" || key.is_empty() {
        print_share::RouteKey::Bill
    } else if let Some(rest) = key.strip_prefix("kot:") {
        print_share::RouteKey::Kot { station_id: rest }
    } else {
        print_share::RouteKey::Bill
    }
}

/// Turn a typed/picked label (or a stored key) into `bill` / `kot:` / `kot:<id>`.
#[allow(dead_code)]
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

/// Add `printer` with `format_id` onto the focused place. Same printer may
/// appear twice with different formats. Same shown name + same format is
/// refused (Brother Standard twice — even if Find listed `_` and `-`).
/// Returns true only when newly added.
#[allow(dead_code)] // Kept for Clear/tests; UI toggles via `toggle_printer_assign_slot`.
fn add_printer_format_to_focused_route(
    st: &mut AppState,
    printer_id: &str,
    format_id: &str,
) -> bool {
    let pid = printer_id.trim();
    let fmt = format_id.trim();
    if pid.is_empty() || fmt.is_empty() || st.printer_draft.printer_by_id(pid).is_none() {
        return false;
    }
    let key_str = st.printer_route_key.clone();
    let key = parse_printer_route_key(&key_str);
    let slot = print_share::DevicePrinters::encode_route_slot(pid, fmt);
    let default_fmt = st.printer_draft.format_id_for(key);
    let name_fmt = {
        let p = st.printer_draft.printer_by_id(pid).unwrap();
        print_share::DevicePrinters::route_name_format_key(
            &printer_display_name(p, |_| String::new()),
            fmt,
        )
    };
    let already = st.printer_draft.route_for(key).iter().any(|s| {
        if s == &slot {
            return true;
        }
        let (opid, ofmt) = print_share::DevicePrinters::decode_route_slot(s);
        let ofmt = if ofmt.is_empty() {
            default_fmt.clone()
        } else {
            ofmt
        };
        st.printer_draft.printer_by_id(&opid).is_some_and(|p| {
            print_share::DevicePrinters::route_name_format_key(
                &printer_display_name(p, |_| String::new()),
                &ofmt,
            ) == name_fmt
        })
    });
    if already {
        return false;
    }
    st.printer_draft.route_mut(key).push(slot);
    if let Some(base) = st.data_base.as_ref() {
        let _ = print_share::save_device_printers(base, &st.printer_draft);
    }
    true
}

/// Remove the route slot at `index` on the focused place. Saves.
#[allow(dead_code)] // Checklist uncheck is the UI path now.
fn remove_focused_route_slot_at(st: &mut AppState, index: usize) -> Option<(String, String)> {
    let key_str = st.printer_route_key.clone();
    let key = parse_printer_route_key(&key_str);
    let place_fmt_label = st.printer_draft.format_for(key).label;
    let slot = {
        let route = st.printer_draft.route_mut(key);
        if index >= route.len() {
            return None;
        }
        route.remove(index)
    };
    let (pid, fmt) = print_share::DevicePrinters::decode_route_slot(&slot);
    let shown = st
        .printer_draft
        .printer_by_id(&pid)
        .map(|p| {
            let t = p.target.trim();
            if !t.is_empty() {
                t.to_string()
            } else if !p.label.trim().is_empty() {
                p.label.clone()
            } else {
                pid.clone()
            }
        })
        .unwrap_or_else(|| pid.clone());
    let flabel = if fmt.is_empty() {
        place_fmt_label
    } else {
        st.printer_draft.format_by_id(&fmt).label
    };
    if let Some(base) = st.data_base.as_ref() {
        let _ = print_share::save_device_printers(base, &st.printer_draft);
    }
    Some((shown, flabel))
}

/// Swap two printers on the focused place (Up/Down). Saves.
fn swap_focused_route_printer(st: &mut AppState, from: usize, to: usize) -> bool {
    if from == to {
        return false;
    }
    let key_str = st.printer_route_key.clone();
    let key = parse_printer_route_key(&key_str);
    let route = st.printer_draft.route_mut(key);
    if from >= route.len() || to >= route.len() {
        return false;
    }
    route.swap(from, to);
    if let Some(base) = st.data_base.as_ref() {
        let _ = print_share::save_device_printers(base, &st.printer_draft);
    }
    true
}

/// Printers on the focused place as `Name (KOT)` rows. Drag swaps; tap removes.
fn focused_route_printer_rows(
    draft: &print_share::DevicePrinters,
    focus: &str,
    _selected: Option<&str>,
    t: impl Fn(&str) -> String,
) -> Vec<ReportRow> {
    let key = parse_printer_route_key(focus);
    let place_fmt = draft.format_for(key);
    draft
        .route_for(key)
        .iter()
        .filter_map(|slot| {
            let (pid, fmt_id) = print_share::DevicePrinters::decode_route_slot(slot);
            let p = draft.printer_by_id(&pid)?;
            let name = printer_display_name(p, &t);
            let flabel = if fmt_id.is_empty() {
                place_fmt.label.clone()
            } else {
                draft.format_by_id(&fmt_id).label
            };
            Some(ReportRow::Line(format!("{name} ({flabel})")))
        })
        .collect()
}

/// Normalize place route: fill missing format, drop duplicate printer+format
/// and duplicate shown-name+format (Brother_… Standard + Brother-… Standard).
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
    let mut seen_slot = std::collections::HashSet::new();
    let mut seen_name_fmt = std::collections::HashSet::new();
    for slot in &before {
        let (pid, fmt) = print_share::DevicePrinters::decode_route_slot(slot);
        if pid.is_empty() || !known.contains(&pid) {
            continue;
        }
        let fmt = if fmt.is_empty() {
            default_fmt.clone()
        } else {
            fmt
        };
        let enc = print_share::DevicePrinters::encode_route_slot(&pid, &fmt);
        if !seen_slot.insert(enc.clone()) {
            continue;
        }
        let Some(p) = st.printer_draft.printer_by_id(&pid) else {
            continue;
        };
        let nf = print_share::DevicePrinters::route_name_format_key(
            &printer_display_name(p, |_| String::new()),
            &fmt,
        );
        if !seen_name_fmt.insert(nf) {
            continue;
        }
        out.push(enc);
    }
    if out == before {
        return false;
    }
    *st.printer_draft.route_mut(key) = out;
    if let Some(base) = st.data_base.as_ref() {
        let _ = print_share::save_device_printers(base, &st.printer_draft);
    }
    true
}

/// Checklist rows for PrinterAssign: assigned first (priority order), then
/// the rest. `value` = display `Name (KOT)`; `subtitle` = slot encode;
/// checked action = dont_use, unchecked = use.
fn printer_assign_suggest_rows(
    draft: &print_share::DevicePrinters,
    focus: &str,
    t: impl Fn(&str) -> String,
) -> Vec<suggest::SuggestRow> {
    use suggest::SuggestRow;
    let key = parse_printer_route_key(focus);
    let use_l = t("use");
    let clear_l = t("dont_use");
    let formats = [
        (print_share::FORMAT_KOT, t("printer_wire_escpos")),
        (
            print_share::FORMAT_STANDARD_PORTRAIT,
            t("printer_wire_os_portrait"),
        ),
        (
            print_share::FORMAT_STANDARD_LANDSCAPE,
            t("printer_wire_os_landscape"),
        ),
    ];
    let assigned_slots: Vec<String> = draft.route_for(key).to_vec();
    let mut rows = Vec::new();
    let mut used_slot = std::collections::HashSet::new();
    let mut used_name_fmt = std::collections::HashSet::new();
    for slot in &assigned_slots {
        let (pid, fmt) = print_share::DevicePrinters::decode_route_slot(slot);
        let Some(p) = draft.printer_by_id(&pid) else {
            continue;
        };
        let fmt = if fmt.is_empty() {
            draft.format_id_for(key)
        } else {
            fmt
        };
        let enc = print_share::DevicePrinters::encode_route_slot(&pid, &fmt);
        if !used_slot.insert(enc.clone()) {
            continue;
        }
        let name = printer_display_name(p, &t);
        let nf = print_share::DevicePrinters::route_name_format_key(&name, &fmt);
        if !used_name_fmt.insert(nf) {
            continue;
        }
        let flabel = draft.format_by_id(&fmt).label;
        rows.push(
            SuggestRow::new(format!("{name} ({flabel})"), enc).with_action(clear_l.clone()),
        );
    }
    for p in &draft.printers {
        let name = printer_display_name(p, &t);
        for (fid, flabel) in &formats {
            let enc = print_share::DevicePrinters::encode_route_slot(&p.id, fid);
            if used_slot.contains(&enc) {
                continue;
            }
            let nf = print_share::DevicePrinters::route_name_format_key(&name, fid);
            // Hide Use when this shown name + format is already assigned
            // (e.g. Brother_… Standard checked → don't offer Brother-… Standard).
            if used_name_fmt.contains(&nf) {
                continue;
            }
            rows.push(
                SuggestRow::new(format!("{name} ({flabel})"), enc).with_action(use_l.clone()),
            );
        }
    }
    rows
}

/// Toggle checklist row on/off for the focused place. Returns true if changed.
fn toggle_printer_assign_slot(st: &mut AppState, slot_enc: &str) -> bool {
    let enc = slot_enc.trim();
    if enc.is_empty() {
        return false;
    }
    let (pid, fmt) = print_share::DevicePrinters::decode_route_slot(enc);
    if pid.is_empty() || fmt.is_empty() {
        return false;
    }
    let key_str = st.printer_route_key.clone();
    let key = parse_printer_route_key(&key_str);
    let default_fmt = st.printer_draft.format_id_for(key);
    if let Some(i) = st.printer_draft.route_for(key).iter().position(|s| s == enc) {
        st.printer_draft.route_mut(key).remove(i);
    } else {
        // Refuse a second row with the same shown name + format.
        let name_fmt = match st.printer_draft.printer_by_id(&pid) {
            Some(p) => print_share::DevicePrinters::route_name_format_key(
                &printer_display_name(p, |_| String::new()),
                &fmt,
            ),
            None => return false,
        };
        let clash = st.printer_draft.route_for(key).iter().any(|s| {
            let (opid, ofmt) = print_share::DevicePrinters::decode_route_slot(s);
            let ofmt = if ofmt.is_empty() {
                default_fmt.clone()
            } else {
                ofmt
            };
            st.printer_draft.printer_by_id(&opid).is_some_and(|p| {
                print_share::DevicePrinters::route_name_format_key(
                    &printer_display_name(p, |_| String::new()),
                    &ofmt,
                ) == name_fmt
            })
        });
        if clash {
            return false;
        }
        st.printer_draft.route_mut(key).push(enc.to_string());
    }
    if let Some(base) = st.data_base.as_ref() {
        let _ = print_share::save_device_printers(base, &st.printer_draft);
    }
    true
}

/// Reorder two assigned slots (checklist indices into suggest_rows that are checked).
fn reorder_printer_assign_priority(st: &mut AppState, from_row: usize, to_row: usize) -> bool {
    if from_row == to_row {
        return false;
    }
    let clear_l = st.t("dont_use");
    let rows = st.suggest_rows.clone();
    let Some(from_slot) = rows.get(from_row).filter(|r| r.action == clear_l).map(|r| r.subtitle.clone()) else {
        return false;
    };
    let Some(to_slot) = rows.get(to_row).filter(|r| r.action == clear_l).map(|r| r.subtitle.clone()) else {
        return false;
    };
    let key_str = st.printer_route_key.clone();
    let key = parse_printer_route_key(&key_str);
    let route = st.printer_draft.route_mut(key);
    let Some(from_i) = route.iter().position(|s| s == &from_slot) else {
        return false;
    };
    let Some(to_i) = route.iter().position(|s| s == &to_slot) else {
        return false;
    };
    route.swap(from_i, to_i);
    if let Some(base) = st.data_base.as_ref() {
        let _ = print_share::save_device_printers(base, &st.printer_draft);
    }
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
        // Always name every discovery — Find already merged them into the pool.
        t("printers_found")
            .replace("{n}", &found.len().to_string())
            .replace("{list}", &found.join(" · "))
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
    let base = st
        .data_base
        .as_ref()
        .ok_or_else(|| "No store path to save printers".to_string())?;
    print_share::save_device_printers(base, &st.printer_draft)?;
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
fn sibling_pickers_rows(store: &Store, t: impl Fn(&str) -> String) -> Vec<ReportRow> {
    let on_off = |on: bool| {
        if on {
            t("unit_picker_on")
        } else {
            t("unit_picker_off")
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
            t("unit_picker_on")
        } else {
            t("unit_picker_off")
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

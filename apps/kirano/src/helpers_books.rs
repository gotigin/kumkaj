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

fn parse_printer_route_key(_key: &str) -> print_share::RouteKey {
    print_share::RouteKey::Bill
}

/// What the owner sees in the route field — never a raw `kot:<uuid>`.
fn printer_route_display_name(key: &str, t: impl Fn(&str) -> String) -> String {
    let key = key.trim();
    if key == "bill" || key.is_empty() {
        return t("route_bill");
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

/// Checklist rows: every virtual printer; checked = attached to the focused place.
/// `subtitle` = virtual printer id. Checked action = `dont_use` (tap clears); else `use`.
fn route_virtual_printer_suggest_rows(
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

/// Toggle VP on the focused route (`printer_route_key`).
fn toggle_route_virtual_printer(st: &mut AppState, vp_id: &str) -> Result<String, String> {
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

/// Reorder attached VPs on the focused route from checklist row indices.
fn reorder_route_virtual_priority(st: &mut AppState, from_row: usize, to_row: usize) -> bool {
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

/// The Bill place as (route key, label, selected) for the place
/// dropdown (`SuggestField::PrinterPlace`).
fn printer_place_choices(focus: &str, t: impl Fn(&str) -> String) -> Vec<(String, String, bool)> {
    vec![("bill".into(), t("route_bill"), focus.trim() == "bill")]
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

/// The printers Find saved on this device (read-only names). Selection for a
/// test page / Test KOT is the CheckList under the list.
fn printers_pool_rows(
    draft: &print_share::DevicePrinters,
    _selected: Option<&str>,
    t: impl Fn(&str) -> String,
) -> Vec<ReportRow> {
    if draft.printers.is_empty() {
        return vec![ReportRow::Line(t("printers_pool_empty"))];
    }
    draft
        .printers
        .iter()
        .map(|p| ReportRow::Line(printer_display_name(p, &t)))
        .collect()
}

/// Checklist: pool printers for test page / Test KOT. `subtitle` = printer id.
fn test_printers_suggest_rows(
    draft: &print_share::DevicePrinters,
    selected: &[String],
    t: impl Fn(&str) -> String,
) -> Vec<suggest::SuggestRow> {
    use suggest::SuggestRow;
    let use_l = t("use");
    let clear_l = t("dont_use");
    let mut rows = Vec::new();
    for p in &draft.printers {
        let name = printer_display_name(p, &t);
        let on = selected.iter().any(|id| id == &p.id);
        rows.push(
            SuggestRow::new(name, p.id.clone()).with_action(if on {
                clear_l.clone()
            } else {
                use_l.clone()
            }),
        );
    }
    if rows.is_empty() {
        rows.push(SuggestRow::new(t("printers_pool_empty"), String::new()));
    }
    rows
}

/// Toggle a pool printer in/out of the test-print selection.
fn toggle_test_printer(st: &mut AppState, printer_id: &str) -> Result<String, String> {
    if st.device_role == DeviceRole::Client {
        return Err(st.t("printers_host_only"));
    }
    let pid = printer_id.trim();
    if pid.is_empty() || st.printer_draft.printer_by_id(pid).is_none() {
        return Err(st.t("printers_pool_empty"));
    }
    let name = st
        .printer_draft
        .printer_by_id(pid)
        .map(|p| printer_display_name(p, |k| st.t(k)))
        .unwrap_or_else(|| pid.to_string());
    if let Some(i) = st.test_printer_ids.iter().position(|id| id == pid) {
        st.test_printer_ids.remove(i);
        if st.printer_selected_id.as_deref() == Some(pid) {
            st.printer_selected_id = st.test_printer_ids.first().cloned();
        }
        Ok(st.t("test_printer_cleared").replace("{name}", &name))
    } else {
        st.test_printer_ids.push(pid.to_string());
        st.printer_selected_id = Some(pid.to_string());
        Ok(st.t("test_printer_picked").replace("{name}", &name))
    }
}

/// Settings → tax rates On/Off (same pattern as units).
/// Value column uses short On/Off pills (tap the row to flip).
fn sibling_pickers_rows(store: &Store, t: impl Fn(&str) -> String) -> Vec<ReportRow> {
    let on_off = |on: bool| {
        if on {
            format!("● {}", t("unit_picker_on"))
        } else {
            format!("○ {}", t("unit_picker_off"))
        }
    };
    store
        .list_tax_rates()
        .unwrap_or_default()
        .into_iter()
        .map(|r| ReportRow::Kv(r.name, on_off(r.picker_enabled)))
        .collect()
}

/// What each `sibling_pickers_rows` row toggles, index for index:
/// `tax:<id>`. The rows show names only — they used to read
/// "tax:<uuid> · GST 5%" so the tap handler could parse its target back out
/// of the text.
fn sibling_picker_keys(store: &Store) -> Vec<Option<String>> {
    let mut keys = Vec::new();
    for r in store.list_tax_rates().unwrap_or_default() {
        keys.push(Some(format!("tax:{}", r.id)));
    }
    keys
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

/// 2000 → "2", 1500 → "1.5" (order quantities in thousandths).
fn milli_as_count(milli: i64) -> String {
    if milli % 1000 == 0 {
        format!("{}", milli / 1000)
    } else {
        format!("{:.2}", milli as f64 / 1000.0).trim_end_matches('0').to_string()
    }
}


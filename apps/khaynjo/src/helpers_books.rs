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

/// Assign a pool printer to the focused Bill/kitchen route (replaces any
/// previous printers there — shops expect one printer per place). Saves.
/// Returns false if the pool is empty or the name was not found.
fn add_printer_to_focused_route(st: &mut AppState, label_or_id: &str) -> bool {
    let pick = label_or_id.trim();
    if pick.is_empty() || st.printer_draft.printers.is_empty() {
        return false;
    }
    let Some(pid) = st
        .printer_draft
        .printers
        .iter()
        .find(|p| p.label.eq_ignore_ascii_case(pick) || p.id == pick)
        .map(|p| p.id.clone())
    else {
        return false;
    };
    let key_str = st.printer_route_key.clone();
    let key = parse_printer_route_key(&key_str);
    let route = st.printer_draft.route_mut(key);
    // Replace — append left the old printer first so "change" never stuck.
    *route = vec![pid];
    if let Some(base) = st.data_base.as_ref() {
        let _ = print_share::save_device_printers(base, &st.printer_draft);
    }
    true
}

/// Kitchen destinations for unsent cart lines: `(station display name, line count)`,
/// in first-seen order. Empty station → `unassigned` label (caller supplies it).
fn pending_kot_destinations(
    products: &[domain::Product],
    stations: &[domain::KitchenStation],
    lines: &[domain::CartLine],
    unassigned: &str,
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
    if print_share::is_finding_printers() {
        return t("printers_looking");
    }
    if !print_share::printers_find_finished() {
        return t("printers_tap_find");
    }
    let found = print_share::found_printers();
    if found.is_empty() {
        t("printers_none_found")
    } else {
        t("printers_found").replace("{list}", &found.join(" · "))
    }
}

/// The printers added on this device, in the order they are tried.
/// `selected` marks the row Remove will delete (tap a row to choose).
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
        .enumerate()
        .map(|(i, p)| {
            let target = if p.target.is_empty() {
                t("printer_target_default")
            } else {
                p.target.clone()
            };
            let mark = if selected == Some(p.id.as_str()) {
                "› "
            } else {
                ""
            };
            ReportRow::Kv(format!("{mark}{}. {}", i + 1, p.label), target)
        })
        .collect()
}

/// Where each thing prints: one row per route (bill, default kitchen, each
/// station), the printers in the order they are tried; › marks the route
/// being edited.
fn printer_route_rows(
    draft: &print_share::DevicePrinters,
    stations: &[domain::KitchenStation],
    focus: &str,
    t: impl Fn(&str) -> String,
) -> Vec<ReportRow> {
    let label_of = |id: &str| -> String {
        draft
            .printer_by_id(id)
            .map(|p| p.label.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let route_row = |title: &str, key: print_share::RouteKey<'_>, focused: bool| -> ReportRow {
        let ids = draft.route_for(key);
        let body = if ids.is_empty() {
            t("route_dont_print")
        } else {
            ids.iter()
                .map(|id| label_of(id))
                .collect::<Vec<_>>()
                .join(" → ")
        };
        let mark = if focused { "› " } else { "" };
        ReportRow::Kv(format!("{mark}{title}"), body)
    };
    let mut rows = Vec::new();
    rows.push(route_row(
        &t("route_bill"),
        print_share::RouteKey::Bill,
        focus.trim() == "bill",
    ));
    // Real kitchen stations only — never invent a "No station" row.
    // Dishes with no station fall through to Bill in `route_for`.
    for s in stations {
        let key = format!("kot:{}", s.id);
        // Station name alone — each station has its own printer list.
        rows.push(route_row(
            &s.name,
            print_share::RouteKey::Kot {
                station_id: &s.id,
            },
            focus.trim() == key,
        ));
    }
    rows
}

/// Printers the last search found, one per row (tap fills the Add form).
/// No rows while searching or when nothing was found — the sentence then
/// says so on its own.
fn printers_found_rows(t: impl Fn(&str) -> String) -> (String, Vec<ReportRow>) {
    let found = print_share::found_printers();
    if print_share::is_finding_printers() || !print_share::printers_find_finished() || found.is_empty() {
        return (printers_found_line(t), Vec::new());
    }
    let intro = t("printers_found")
        .replace("{list}", "")
        .trim_end_matches([':', ' ', '—', '-'])
        .trim()
        .to_string();
    (intro, found.into_iter().map(ReportRow::Line).collect())
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

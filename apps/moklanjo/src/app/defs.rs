/// Designed (left-to-right) `align.x` of labels/inputs, captured the
/// first time a right-to-left language right-aligns them, so switching
/// back restores the exact original layout.
static ORIG_TEXT_ALIGN: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, f64>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// Sideways-scrolling tab strips: (wrapper holding the ‹ › chevrons, the
/// scroll view itself).
const TAB_STRIPS: [(&[LiveId], &[LiveId]); 3] = [
    (ids!(feat_tabs), ids!(feat_tabs.tabs)),
    (ids!(lock_role_tabs), ids!(lock_role_tabs.tabs)),
    (ids!(bottom_nav_tabs), ids!(bottom_nav_scroll)),
];

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

/// Escape Markdown markers so product/customer strings never italicize or become headings.
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

fn parse_courier_route_key(key: &str) -> print_share::RouteKey {
    match key.trim().to_ascii_lowercase().as_str() {
        "label" | "awb" | "awb label" => print_share::RouteKey::Label,
        _ => print_share::RouteKey::Receipt,
    }
}

/// Printers the last search found, one per row, under the sentence that
/// introduces them ("Found on this device:"). No rows while searching or
/// when nothing was found — the sentence then says so on its own.
fn printers_found_rows(t: impl Fn(&str) -> String) -> (String, Vec<ReportRow>) {
    let found = print_share::found_printers();
    if print_share::is_finding_printers() || !print_share::printers_find_finished() || found.is_empty() {
        return (printers_found_line(t), Vec::new());
    }
    let intro = t("printers_found").replace("{list}", "");
    (intro.trim_end().to_string(), found.into_iter().map(ReportRow::Line).collect())
}

/// Settings → Units as rows: each family's units from smallest to largest
/// with what one of them is in the family's anchor (what books record),
/// units not linked to a standard yet, then the conversions grouped by the
/// unit they start from.
fn units_report_rows(store: &Store) -> Vec<ReportRow> {
    let units = store.list_units().unwrap_or_default();
    let convs = store.list_unit_conversions().unwrap_or_default();
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
                    let value = if u.code == anchor {
                        "base unit".to_string()
                    } else {
                        format!("1 {} = {} {anchor}", u.code, precise::format_kg(factor))
                    };
                    (factor, ReportRow::Kv(name(u), value))
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
        .map(|u| ReportRow::Line(name(u)))
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

/// The printers added on this device, in the order they are tried.
fn courier_printers_pool_rows(draft: &print_share::DevicePrinters) -> Vec<ReportRow> {
    if draft.printers.is_empty() {
        return vec![ReportRow::Line("(none yet — add one below)".into())];
    }
    draft
        .printers
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let target = if p.target.is_empty() {
                "this computer's usual printer (blank address)".to_string()
            } else {
                p.target.clone()
            };
            ReportRow::Kv(format!("{}. {}", i + 1, p.label), target)
        })
        .collect()
}

/// Where receipts and AWB labels print, the printers in the order they are
/// tried; › marks the route being edited.
fn courier_printer_route_rows(
    draft: &print_share::DevicePrinters,
    focus: &str,
    t: impl Fn(&str) -> String,
) -> Vec<ReportRow> {
    let label_of = |id: &str| -> String {
        draft
            .printer_by_id(id)
            .map(|p| p.label.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let route_row = |title: &str, key: print_share::RouteKey, focused: bool| -> ReportRow {
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
    let focus = focus.trim().to_ascii_lowercase();
    vec![
        route_row(
            &t("route_receipt"),
            print_share::RouteKey::Receipt,
            focus == "receipt" || focus.is_empty(),
        ),
        route_row(
            &t("route_label"),
            print_share::RouteKey::Label,
            focus == "label" || focus == "awb" || focus == "awb label",
        ),
    ]
}

fn plain_display_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '*' | '_' | '`' | '#' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

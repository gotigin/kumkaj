/// The Ops board's parcel cards for the lane on screen (expand-to-content:
/// the page owns the scroll, like the bookings list).
#[derive(Script, ScriptHook, Widget)]
struct OpsBoardList {
    #[deref]
    view: View,
}

fn ops_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(118.0))
}

fn run_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(64.0))
}

/// Which pre-coloured pill a status wears (`pill_0` … `pill_5` in `OpsCard`).
fn ops_pill_slot(status: manifest::TrackingStatus) -> usize {
    use manifest::TrackingStatus as T;
    match status {
        T::Booked | T::PickedUp => 0,
        T::HandedOver | T::InTransit => 1,
        T::OutForDelivery => 2,
        T::Ndr => 3,
        T::Delivered => 4,
        T::Rto | T::Cancelled => 5,
    }
}

/// A stamp as staff read it: the time alone today, date and time before —
/// in the shop's date order and clock (Settings → Country & tax).
fn ops_when(fiscal: &FiscalSettings, at: &str, today: &str) -> String {
    if at.get(..10) == Some(today) {
        let t = fiscal.format_time(at);
        if !t.is_empty() {
            return t;
        }
    }
    let dt = fiscal.format_datetime(at);
    if dt.is_empty() {
        at.get(..16).unwrap_or(at).replace('T', " ")
    } else {
        dt
    }
}

/// The three lines under an Ops card's AWB: where it goes and to whom; what
/// it is (pieces, kg, COD); since when and who has it (or, after a failed
/// attempt, which attempt, why, and when it goes out again).
fn ops_card_lines(
    row: &manifest::OpsRow,
    lang: &str,
    currency: &str,
    fiscal: &FiscalSettings,
    today: &str,
) -> (String, String, String) {
    let t = |k: &str| locale::t(lang, k);
    let money = |p: i64| Money(p).format_localized(currency, fiscal);
    let mut l1 = row.destination.trim().to_string();
    let who = row.consignee_name.trim();
    if !who.is_empty() {
        if l1.is_empty() {
            l1 = who.to_string();
        } else {
            l1 = format!("{l1} · {who}");
        }
    }
    let mut l2: Vec<String> = Vec::new();
    if row.piece_count > 1 {
        l2.push(t("pieces_short_fmt").replace("{n}", &row.piece_count.to_string()));
    }
    if !row.chargeable_weight_kg.is_zero() {
        l2.push(format!("{} kg", precise::format_kg(row.chargeable_weight_kg)));
    }
    if row.cod_paise > 0 {
        l2.push(format!("COD {}", money(row.cod_paise)));
    }
    let from = row.origin_name.trim();
    if !from.is_empty() {
        l2.push(format!("← {from}"));
    }
    let mut l3 = vec![ops_when(fiscal, &row.moved_at, today)];
    if !row.holder.trim().is_empty() {
        l3.push(row.holder.trim().to_string());
    }
    if row.tracking() == manifest::TrackingStatus::Ndr {
        l3.push(t("attempt_fmt").replace("{n}", &row.attempt_count.to_string()));
        if !row.ndr_reason.trim().is_empty() {
            let reason = manifest::NdrReason::parse(&row.ndr_reason);
            let shown = if reason == manifest::NdrReason::Other
                && !row.ndr_reason.trim().eq_ignore_ascii_case("other")
            {
                row.ndr_reason.trim().to_string()
            } else {
                t(reason.locale_key())
            };
            l3.push(shown);
        }
        if !row.next_attempt_at.trim().is_empty() {
            l3.push(format!("→ {}", fiscal.format_date(row.next_attempt_at.trim())));
        }
    }
    (l1, l2.join(" · "), l3.join(" · "))
}

impl Widget for OpsBoardList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (rows, picked, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.ops_rows.clone(),
                st.ops_picked.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let row_h = *ops_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(rows.len(), row_h, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = rows.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let status = row.tracking();
                    let on = picked.iter().any(|id| *id == row.sale_id);
                    item.label(cx, ids!(awb)).set_text(cx, &row.awb_no);
                    let slot = ops_pill_slot(status);
                    let pill_text = locale::t(&lang, status.label_key());
                    for (i, (pill, text)) in [
                        (ids!(pill_0), ids!(t0)),
                        (ids!(pill_1), ids!(t1)),
                        (ids!(pill_2), ids!(t2)),
                        (ids!(pill_3), ids!(t3)),
                        (ids!(pill_4), ids!(t4)),
                        (ids!(pill_5), ids!(t5)),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        item.view(cx, pill).set_visible(cx, i == slot);
                        if i == slot {
                            item.label(cx, text).set_text(cx, &pill_text);
                        }
                    }
                    item.label(cx, ids!(amount))
                        .set_text(cx, &Money(row.total_paise).format_localized(&currency, &fiscal));
                    let (l1, l2, l3) = ops_card_lines(row, &lang, &currency, &fiscal, &today);
                    item.label(cx, ids!(line1)).set_text(cx, &l1);
                    item.label(cx, ids!(line2)).set_text(cx, &l2);
                    item.label(cx, ids!(line2)).set_visible(cx, !l2.is_empty());
                    item.label(cx, ids!(line3)).set_text(cx, &l3);
                    item.view(cx, ids!(mark_on)).set_visible(cx, on);
                    item.view(cx, ids!(mark_off)).set_visible(cx, !on);
                    item.view(cx, ids!(edge)).set_visible(cx, on);
                    item.draw_all_unscoped(cx);
                }
                if !rows.is_empty() {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / rows.len() as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, ops_row_height(), row_h, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Recent carrier bags and rider runs, each with Print (the run sheet / DRS
/// the person picked — never just "the latest").
#[derive(Script, ScriptHook, Widget)]
struct RunList {
    #[deref]
    view: View,
}

impl Widget for RunList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (runs, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (st.ops_runs.clone(), st.fiscal.clone(), st.locale.language.clone())
        };
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let row_h = *run_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(runs.len(), row_h, 0.0)),
            ..walk
        };
        let print = locale::t(&lang, "print_word");
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, runs.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(m) = runs.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let (kind, who) = if m.kind == "own_fleet" {
                        (locale::t(&lang, "run_word"), m.rider_name.as_str())
                    } else {
                        (locale::t(&lang, "bag_word"), m.carrier_name.as_str())
                    };
                    item.label(cx, ids!(run_main)).set_text(cx, &format!("{kind} · {who}"));
                    let count = locale::parcels(&lang, m.line_count);
                    item.label(cx, ids!(run_sub))
                        .set_text(cx, &format!("{} · {count}", ops_when(&fiscal, &m.created_at, &today)));
                    item.button(cx, ids!(print)).set_text(cx, &print);
                    item.draw_all_unscoped(cx);
                }
                if !runs.is_empty() {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / runs.len() as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, run_row_height(), row_h, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct SaleList {
    #[deref]
    view: View,
}

impl Widget for SaleList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (sales, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.recent_sales.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let row = *sale_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(sales.len(), row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, sales.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(sale) = sales.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let money = |paise: i64| Money(paise).format_localized(&currency, &fiscal);
                    let method = match sale.pay_method.as_str() {
                        "cash" => locale::t(&lang, "cash"),
                        "upi" => fiscal.digital_pay_label().to_string(),
                        "card" => locale::t(&lang, "card"),
                        "credit" => locale::t(&lang, "pay_credit"),
                        "split" => locale::t(&lang, "split"),
                        other => other.to_string(),
                    };
                    // "AWB-… · Rajkot", or how it was paid when there is no AWB.
                    let main = match (sale.awb_no.trim(), sale.destination.trim()) {
                        ("", _) => method.clone(),
                        (awb, "") => awb.to_string(),
                        (awb, to) => format!("{awb} · {to}"),
                    };
                    // "13:00" today, "2026-10-21 19:05" before; how paid; by
                    // whom; the status only when it is not an ordinary one.
                    let at = sale.created_at.replace('T', " ");
                    let mut sub = if at.get(..10) == Some(today.as_str()) {
                        at.get(11..16).unwrap_or("").to_string()
                    } else {
                        at.get(..16).unwrap_or(&at).to_string()
                    };
                    if !sale.awb_no.trim().is_empty() {
                        sub.push_str(&format!(" · {method}"));
                    }
                    if !sale.created_by_name.is_empty() {
                        sub.push_str(&format!(" · {}", sale.created_by_name));
                    }
                    if sale.status != "completed" {
                        sub.push_str(&format!(" · {}", sale.status));
                    }
                    if sale.rounding_paise != 0 {
                        sub.push_str(&format!(
                            " · Applied {} · round {}",
                            money(sale.lines_total_paise),
                            money(sale.rounding_paise)
                        ));
                    }
                    item.label(cx, ids!(sale_main)).set_text(cx, &main);
                    item.label(cx, ids!(label)).set_text(cx, &sub);
                    item.label(cx, ids!(sale_amount)).set_text(cx, &money(sale.total_paise));
                    item.draw_all_unscoped(cx);
                }
                if !sales.is_empty() {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / sales.len() as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, sale_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Settings → Printers: the saved printers as `name · paper`, one list in two
/// parts around the open printer's Width / Length — `part: "before"` holds the
/// printers up to and including the open one (all of them while none is
/// open), `part: "after"` the rest. Any number of printers (owner,
/// 2026-11-16: "this shouldn't be restricted"); only the rows on screen draw.
#[derive(Script, ScriptHook, Widget)]
struct PrinterPaperRows {
    #[deref]
    view: View,
    #[live]
    part: String,
    /// Measured row pitch (a long name wraps at phone width).
    #[rust]
    row_px: f64,
}

/// The printers one part shows: its first printer's index, their lines and
/// which printer is open.
fn printer_paper_part(after: bool) -> (usize, Vec<String>, Option<usize>) {
    let st = app_state().lock().unwrap();
    let printers = &st.printer_draft.printers;
    let open = st
        .printer_paper_open
        .as_deref()
        .and_then(|id| printers.iter().position(|p| p.id == id));
    let range = match (after, open) {
        (false, Some(o)) => 0..o + 1,
        (false, None) => 0..printers.len(),
        (true, Some(o)) => o + 1..printers.len(),
        (true, None) => 0..0,
    };
    let start = range.start;
    let lines = printers[range]
        .iter()
        .map(|p| printer_paper_line(p, |k| st.t(k)))
        .collect();
    (start, lines, open)
}

impl PrinterPaperRows {
    /// The printer a row of this part stands for.
    fn printer_index(&self, item_id: usize) -> usize {
        printer_paper_part(self.part == "after").0 + item_id
    }
}

impl Widget for PrinterPaperRows {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (start, lines, open) = printer_paper_part(self.part == "after");
        if lines.is_empty() {
            return DrawStep::done();
        }
        let row = if self.row_px > 1.0 { self.row_px } else { 56.0 };
        let walk = Walk {
            height: Size::Fixed(list_content_height(lines.len(), row, 0.0)),
            ..walk
        };
        let mut measured = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, lines.len());
                let mut drew = 0usize;
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(line) = lines.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let is_open = open == Some(start + item_id);
                    item.button(cx, ids!(row)).set_text(cx, line);
                    item.button(cx, ids!(row_on)).set_text(cx, line);
                    item.button(cx, ids!(row)).set_visible(cx, !is_open);
                    item.button(cx, ids!(row_on)).set_visible(cx, is_open);
                    item.draw_all_unscoped(cx);
                    drew += 1;
                }
                if drew > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured = extent / drew as f64;
                    }
                }
            }
        }
        // Settles on the rows' own pitch (once; equal pitch draws no more).
        if measured > 1.0 && (measured - row).abs() > 1.0 {
            self.row_px = measured;
            self.view.redraw(cx);
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

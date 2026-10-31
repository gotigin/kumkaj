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

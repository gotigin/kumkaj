/// Table bookings still to come ([`store::Reservation`]).
#[derive(Script, ScriptHook, Widget)]
struct ResvList {
    #[deref]
    view: View,
}

impl Widget for ResvList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (rows, seat, cancel, guests_word, lang) = {
            let st = app_state().lock().unwrap();
            (st.reservations.clone(), st.t("resv_seat"), st.t("cancel"), st.t("guests_word"), st.locale.language.clone())
        };
        let _ = lang;
        let row = *resv_row_height().lock().unwrap();
        let n = rows.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row, 0.0)),
            ..walk
        };
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, n);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(r) = rows.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    // "20:30 · Mehta · 4 guests · D3 · 98765"; another day shows its date.
                    let when = if r.at.starts_with(&today) { r.at[11..].to_string() } else { r.at.clone() };
                    let mut text = format!("{when} · {}", r.name);
                    if r.guests > 0 {
                        text.push_str(&format!(" · {} {guests_word}", r.guests));
                    }
                    for extra in [&r.table_name, &r.phone] {
                        if !extra.is_empty() {
                            text.push_str(&format!(" · {extra}"));
                        }
                    }
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&text));
                    item.button(cx, ids!(seat)).set_text(cx, &seat);
                    item.button(cx, ids!(cancel)).set_text(cx, &cancel);
                    item.draw_all_unscoped(cx);
                }
                if n > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / n as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, resv_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct CurrencyList {
    #[deref]
    view: View,
}

impl Widget for CurrencyList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let selected = app_state().lock().unwrap().locale.currency.clone();
        let use_label = {
            let st = app_state().lock().unwrap();
            st.t("use")
        };
        let walk = Walk {
            height: Size::Fixed(280.0),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, CURRENCIES.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(cur) = CURRENCIES.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let mark = if cur.code == selected { "✓ " } else { "" };
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&format!("{mark}{} {} — {}", cur.symbol, cur.code, cur.name_en),));
                    item.button(cx, ids!(pick)).set_text(cx, &use_label);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

/// Selectable display text (Markdown) with Label-like `visible` support.
/// Stock Markdown/TextFlow ignore `set_visible`, which left empty-state copy
/// on screen after products/cart lines appeared.
#[derive(Script, ScriptHook, Widget)]
struct WrapLabel {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    markdown: Markdown,
    #[live(true)]
    #[visible]
    visible: bool,
}

impl Widget for WrapLabel {
    fn is_interactive(&self) -> bool {
        self.visible && self.markdown.text_flow.selectable
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.visible && event.requires_visibility() {
            return;
        }
        self.markdown.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        // Keep scroll blocked across redraws while a drag-select is live.
        if self.markdown.text_flow.is_drag_selecting() {
            cx.block_scrolling_except_within(Area::Empty);
        }
        self.markdown.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.markdown.text()
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        self.markdown.set_text(cx, v);
    }
}

impl WrapLabel {
    /// Right-to-left support: paragraph alignment inside the flow
    /// (vendor patch: `TextFlow::paragraph_align_x`).
    pub fn set_text_align_x(&mut self, x: f64) {
        self.markdown.text_flow.paragraph_align_x = x;
    }
}

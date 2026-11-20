#[derive(Script, ScriptHook, Widget)]
struct CounterDraftsList {
    #[deref]
    view: View,
}

fn counter_drafts_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(148.0))
}

impl Widget for CounterDraftsList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (rows, cols, currency, fiscal, lang, visible) = {
            let st = app_state().lock().unwrap();
            let rows = st.counter_drafts_rows.clone();
            (
                rows,
                counter_drafts_column_count(st.counter_drafts_columns),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
                st.screen == Screen::Drafts,
            )
        };
        if !visible {
            return DrawStep::done();
        }
        let line_rows = rows.len().div_ceil(cols.max(1));
        let row_h = *counter_drafts_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(line_rows, row_h, 0.0)),
            ..walk
        };
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, line_rows);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id >= line_rows {
                        continue;
                    }
                    let item = list.item(cx, item_id, id!(Item));
                    for (col, path) in [
                        (0usize, &[id!(c0)][..]),
                        (1, &[id!(c1)][..]),
                        (2, &[id!(c2)][..]),
                        (3, &[id!(c3)][..]),
                    ] {
                        let card = item.widget(cx, path);
                        if col >= cols {
                            card.set_visible(cx, false);
                            continue;
                        }
                        let idx = item_id * cols + col;
                        if let Some(row) = rows.get(idx) {
                            card.set_visible(cx, true);
                            card
                                .label(cx, ids!(kind))
                                .set_text(cx, &locale::t(&lang, "drafts_card_kind"));
                            card.label(cx, ids!(title)).set_text(cx, &row.title);
                            card
                                .widget(cx, ids!(line2))
                                .set_text(cx, &plain_display_text(&row.line2));
                            card
                                .widget(cx, ids!(line3))
                                .set_text(cx, &plain_display_text(&row.line3));
                            let amount =
                                Money(row.amount_paise).format_localized(&currency, &fiscal);
                            card.label(cx, ids!(amount)).set_text(cx, &amount);
                            let when = fiscal.format_datetime(&row.at);
                            let time = if when.is_empty() {
                                row.at.get(..16).unwrap_or(&row.at).replace('T', " ")
                            } else if row.at.get(..10) == Some(today.as_str()) {
                                fiscal.format_time(&row.at)
                            } else {
                                when
                            };
                            card.label(cx, ids!(time)).set_text(cx, &time);
                            let discard = card.button(cx, ids!(discard_draft));
                            discard.set_text(cx, &locale::t(&lang, "drafts_discard"));
                            discard.set_visible(cx, true);
                        } else {
                            card.set_visible(cx, false);
                        }
                    }
                    item.draw_all_unscoped(cx);
                }
                if line_rows > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / line_rows as f64;
                    }
                }
            }
        }
        remember_content_row(
            cx,
            &mut self.view,
            counter_drafts_row_height(),
            row_h,
            measured_row,
        );
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}
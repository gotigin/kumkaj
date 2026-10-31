/// Under a "type or pick" field (see `suggest.rs`): closed it draws
/// nothing; open it is a capped scrolling list of what the counter
/// already has for that field. `field` names which [`SuggestField`] it serves.
#[derive(Script, ScriptHook, Widget)]
struct SuggestList {
    #[deref]
    view: View,
    #[live]
    field: String,
    #[rust]
    drawn_rev: u64,
}

impl SuggestList {
    fn is_open(&self) -> bool {
        app_state().lock().unwrap().suggest_open.map(|f| f.key()) == Some(self.field.as_str())
    }
}

impl Widget for SuggestList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (open, rows, rev, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.suggest_open.map(|f| f.key()) == Some(self.field.as_str()),
                st.suggest_rows.clone(),
                st.suggest_rev,
                st.locale.language.clone(),
            )
        };
        if !open || rows.is_empty() {
            return DrawStep::done();
        }
        let row_h = *suggest_row_height().lock().unwrap();
        let n = rows.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row_h, 0.0).min(SUGGEST_MAX_HEIGHT)),
            ..walk
        };
        let use_text = locale::t(&lang, "use");
        let new_rev = self.drawn_rev != rev;
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if new_rev {
                    list.set_first_id_and_scroll(0, 0.0);
                }
                list.set_item_range(cx, 0, n);
                let mut drew = 0usize;
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = rows.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.label(cx, ids!(title)).set_text(cx, &row.value);
                    item.label(cx, ids!(subtitle)).set_text(cx, &row.subtitle);
                    item.label(cx, ids!(subtitle)).set_visible(cx, !row.subtitle.is_empty());
                    let action = if row.action.trim().is_empty() {
                        use_text.as_str()
                    } else {
                        row.action.as_str()
                    };
                    item.label(cx, ids!(action)).set_text(cx, action);
                    item.draw_all_unscoped(cx);
                    drew += 1;
                }
                if drew > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / drew as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, suggest_row_height(), row_h, measured_row);
        self.drawn_rev = rev;
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.is_open() {
            return;
        }
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

/// Tallest a "type or pick" list gets before it scrolls (about six rows).
const SUGGEST_MAX_HEIGHT: f64 = 330.0;

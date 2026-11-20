const DRAFTS_COL_BTNS: [(&[LiveId], &[LiveId], u8); 3] = [
    (ids!(drafts_col_2), ids!(drafts_col_2_on), 2),
    (ids!(drafts_col_3), ids!(drafts_col_3_on), 3),
    (ids!(drafts_col_4), ids!(drafts_col_4_on), 4),
];

impl App {
    fn handle_counter_drafts_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (off, on, cols) in DRAFTS_COL_BTNS {
            if self.ui.button(cx, off).clicked(actions) || self.ui.button(cx, on).clicked(actions)
            {
                {
                    let mut st = app_state().lock().unwrap();
                    st.counter_drafts_columns = cols;
                }
                self.sync_chrome(cx);
                return;
            }
        }
        if self.ui.button(cx, ids!(drafts_refresh)).clicked(actions) {
            let msg = {
                let mut st = app_state().lock().unwrap();
                st.refresh_counter_drafts().map(|_| st.t("refresh"))
            };
            self.set_status(cx, msg.unwrap_or_else(|e| e));
            self.sync_chrome(cx);
            return;
        }
        if self.ui.button(cx, ids!(park_draft_btn)).clicked(actions) {
            let result = {
                let mut st = app_state().lock().unwrap();
                st.park_counter_desk().map(|_| st.t("drafts_parked"))
            };
            match &result {
                Ok(_) => {
                    let mut st = app_state().lock().unwrap();
                    st.clear_order_desk();
                }
                Err(_) => {}
            }
            match &result {
                Ok(m) => self.set_status(cx, m.clone()),
                Err(e) => self.set_status(cx, e.clone()),
            }
            self.sync_chrome(cx);
            return;
        }

        let cols = counter_drafts_column_count(app_state().lock().unwrap().counter_drafts_columns);
        let list = self
            .ui
            .widget(cx, ids!(counter_drafts_list))
            .portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            for col in 0..cols {
                let path = match col {
                    0 => ids!(c0),
                    1 => ids!(c1),
                    2 => ids!(c2),
                    _ => ids!(c3),
                };
                let card_w = item.view(cx, path);
                if card_w.button(cx, ids!(discard_draft)).clicked(actions) {
                    let card_index = item_id * cols + col;
                    let draft_id = {
                        let st = app_state().lock().unwrap();
                        st.counter_drafts_rows
                            .get(card_index)
                            .map(|c| c.id.clone())
                    };
                    let Some(id) = draft_id else {
                        continue;
                    };
                    let result = {
                        let mut st = app_state().lock().unwrap();
                        st.discard_counter_draft(&id)
                    };
                    match result {
                        Ok(m) => self.set_status(cx, m),
                        Err(e) => self.set_status(cx, e),
                    }
                    self.sync_chrome(cx);
                    return;
                }
                if card_w.finger_up(actions).is_some_and(finger_up_is_tap) {
                    let card_index = item_id * cols + col;
                    let pick = {
                        let st = app_state().lock().unwrap();
                        st.counter_drafts_rows.get(card_index).cloned()
                    };
                    let Some(card) = pick else {
                        continue;
                    };
                    let result = {
                        let mut st = app_state().lock().unwrap();
                        st.resume_counter_draft(&card.id)
                    };
                    if let Err(e) = result {
                        self.set_status(cx, e);
                        self.sync_chrome(cx);
                        return;
                    }
                    self.go(cx, Screen::Pos);
                    self.sync_chrome(cx);
                    return;
                }
            }
        }
    }

    fn paint_counter_drafts(&mut self, cx: &mut Cx) {
        let (cols, count, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.counter_drafts_columns,
                st.counter_drafts_rows.len(),
                st.locale.language.clone(),
            )
        };
        let t = |k: &str| locale::t(&lang, k);
        let col_value = if cols == 0 { 3 } else { cols };
        for (off, on, c) in DRAFTS_COL_BTNS {
            let active = c == col_value;
            let label = t("drafts_cols_fmt").replace("{n}", &c.to_string());
            self.ui.button(cx, off).set_text(cx, &label);
            self.ui.button(cx, on).set_text(cx, &label);
            self.ui.button(cx, off).set_visible(cx, !active);
            self.ui.button(cx, on).set_visible(cx, active);
        }
        let summary = t("drafts_summary_fmt").replace("{n}", &count.to_string());
        self.ui
            .widget(cx, ids!(drafts_summary))
            .set_text(cx, &plain_display_text(&summary));
        let empty = count == 0;
        self.ui.widget(cx, ids!(drafts_empty)).set_visible(cx, empty);
        if empty {
            self.ui
                .widget(cx, ids!(drafts_empty))
                .set_text(cx, &plain_display_text(&t("drafts_empty")));
        }
    }
}
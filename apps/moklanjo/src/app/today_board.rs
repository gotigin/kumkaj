const BOARD_FILTER_BTNS: [(&[LiveId], &[LiveId], TodayBoardFilter); 3] = [
    (ids!(board_filter_all), ids!(board_filter_all_on), TodayBoardFilter::All),
    (ids!(board_filter_draft), ids!(board_filter_draft_on), TodayBoardFilter::Draft),
    (ids!(board_filter_booked), ids!(board_filter_booked_on), TodayBoardFilter::Booked),
];

const BOARD_COL_BTNS: [(&[LiveId], &[LiveId], u8); 3] = [
    (ids!(board_col_2), ids!(board_col_2_on), 2),
    (ids!(board_col_3), ids!(board_col_3_on), 3),
    (ids!(board_col_4), ids!(board_col_4_on), 4),
];

impl App {
    fn handle_today_board_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (off, on, filter) in BOARD_FILTER_BTNS {
            if self.ui.button(cx, off).clicked(actions) || self.ui.button(cx, on).clicked(actions)
            {
                {
                    let mut st = app_state().lock().unwrap();
                    st.today_board_filter = filter;
                }
                self.sync_chrome(cx);
                return;
            }
        }
        for (off, on, cols) in BOARD_COL_BTNS {
            if self.ui.button(cx, off).clicked(actions) || self.ui.button(cx, on).clicked(actions)
            {
                {
                    let mut st = app_state().lock().unwrap();
                    st.today_board_columns = cols;
                }
                self.sync_chrome(cx);
                return;
            }
        }
        if self.ui.button(cx, ids!(board_refresh)).clicked(actions) {
            let msg = {
                let mut st = app_state().lock().unwrap();
                st.refresh_today_board().map(|_| st.t("refresh"))
            };
            self.set_status_as(cx, msg);
            self.sync_chrome(cx);
            return;
        }
        if self.ui.button(cx, ids!(park_draft_btn)).clicked(actions)
            || self.ui.button(cx, ids!(park_draft_top)).clicked(actions)
        {
            // Pull every typed field into AppState before dirty/park — otherwise
            // mid-entry text never counted and Park looked flaky.
            self.sync_booking_desk_from_ui(cx);
            let result = {
                let mut st = app_state().lock().unwrap();
                st.park_booking_desk().map(|_| st.t("board_parked"))
            };
            match &result {
                Ok(_) => {
                    {
                        let mut st = app_state().lock().unwrap();
                        st.clear_booking_desk();
                    }
                    self.clear_booking_desk_ui(cx);
                }
                Err(_) => {}
            }
            self.set_status_as(cx, result);
            self.sync_chrome(cx);
            return;
        }

        let cols = *today_board_drawn_cols().lock().unwrap();
        let list = self
            .ui
            .widget(cx, ids!(today_board_list))
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
                        let filter = st.today_board_filter;
                        st.today_board_rows
                            .iter()
                            .filter(|c| match filter {
                                TodayBoardFilter::All => true,
                                TodayBoardFilter::Draft => c.is_draft,
                                TodayBoardFilter::Booked => !c.is_draft,
                            })
                            .nth(card_index)
                            .filter(|c| c.is_draft)
                            .map(|c| c.id.clone())
                    };
                    let Some(id) = draft_id else {
                        continue;
                    };
                    let result = {
                        let mut st = app_state().lock().unwrap();
                        st.discard_today_board_draft(&id)
                    };
                    self.set_status_as(cx, result);
                    self.sync_chrome(cx);
                    return;
                }
                if card_w.finger_up(actions).is_some_and(finger_up_is_tap) {
                    let card_index = item_id * cols + col;
                    let pick = {
                        let st = app_state().lock().unwrap();
                        let filter = st.today_board_filter;
                        st.today_board_rows
                            .iter()
                            .filter(|c| match filter {
                                TodayBoardFilter::All => true,
                                TodayBoardFilter::Draft => c.is_draft,
                                TodayBoardFilter::Booked => !c.is_draft,
                            })
                            .nth(card_index)
                            .cloned()
                    };
                    let Some(card) = pick else {
                        continue;
                    };
                    if card.is_draft {
                        let result = {
                            let mut st = app_state().lock().unwrap();
                            st.resume_booking_draft(&card.id)
                        };
                        if let Err(e) = result {
                            self.set_status(cx, e);
                            self.sync_chrome(cx);
                            return;
                        }
                        self.fill_booking_desk_ui(cx);
                        self.go(cx, Screen::Pos);
                    } else {
                        {
                            let mut st = app_state().lock().unwrap();
                            st.selected_sale_id = Some(card.id);
                        }
                        self.go(cx, Screen::Sales);
                    }
                    self.sync_chrome(cx);
                    return;
                }
            }
        }
    }

    fn paint_today_board(&mut self, cx: &mut Cx) {
        let (filter, cols, rows, lang, currency, fiscal) = {
            let st = app_state().lock().unwrap();
            let visible: Vec<_> = st
                .today_board_rows
                .iter()
                .filter(|c| match st.today_board_filter {
                    TodayBoardFilter::All => true,
                    TodayBoardFilter::Draft => c.is_draft,
                    TodayBoardFilter::Booked => !c.is_draft,
                })
                .collect();
            let drafts = visible.iter().filter(|c| c.is_draft).count();
            let booked = visible.len() - drafts;
            (
                st.today_board_filter,
                st.today_board_columns,
                (visible.len(), drafts, booked),
                st.locale.language.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
            )
        };
        let t = |k: &str| locale::t(&lang, k);
        for (off, on, f) in BOARD_FILTER_BTNS {
            let active = f == filter;
            let label = match f {
                TodayBoardFilter::All => t("board_filter_all"),
                TodayBoardFilter::Draft => t("board_filter_draft"),
                TodayBoardFilter::Booked => t("board_filter_booked"),
            };
            self.ui.button(cx, off).set_text(cx, &label);
            self.ui.button(cx, on).set_text(cx, &label);
            self.ui.button(cx, off).set_visible(cx, !active);
            self.ui.button(cx, on).set_visible(cx, active);
        }
        // No pick yet: light the count the Board draws with at this width.
        let col_value = today_board_column_count(cols, *content_width().lock().unwrap()) as u8;
        for (off, on, c) in BOARD_COL_BTNS {
            let active = c == col_value;
            let label = t("board_cols_fmt").replace("{n}", &c.to_string());
            self.ui.button(cx, off).set_text(cx, &label);
            self.ui.button(cx, on).set_text(cx, &label);
            self.ui.button(cx, off).set_visible(cx, !active);
            self.ui.button(cx, on).set_visible(cx, active);
        }
        let summary = t("board_summary_fmt")
            .replace("{total}", &rows.0.to_string())
            .replace("{drafts}", &rows.1.to_string())
            .replace("{booked}", &rows.2.to_string());
        self.ui
            .widget(cx, ids!(board_summary))
            .set_text(cx, &plain_display_text(&summary));
        let empty = rows.0 == 0;
        self.ui.widget(cx, ids!(board_empty)).set_visible(cx, empty);
        if empty {
            self.ui.widget(cx, ids!(board_empty)).set_text(
                cx,
                &plain_display_text(&t("board_empty")),
            );
        }
        let _ = (currency, fiscal);
    }
}
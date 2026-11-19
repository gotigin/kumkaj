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
    /// Rows kept while the close height tween finishes.
    #[rust]
    hold_rows: Vec<suggest::SuggestRow>,
}

impl SuggestList {
    fn is_open(&self) -> bool {
        app_state().lock().unwrap().suggest_open.map(|f| f.key()) == Some(self.field.as_str())
    }

    fn anim_key(&self) -> String {
        format!("suggest:{}", self.field)
    }
}

impl Widget for SuggestList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // The preview's printer list marks the one in use (✓) like a dropdown's pick.
        let paper_menu = self.field == "job_preview_paper";
        let (open, rows, rev, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.suggest_open.map(|f| f.key()) == Some(self.field.as_str()),
                st.suggest_rows.clone(),
                st.suggest_rev,
                st.locale.language.clone(),
            )
        };
        if open && !rows.is_empty() {
            self.hold_rows = rows.clone();
        }
        let draw_rows = if open && !rows.is_empty() {
            &rows
        } else {
            &self.hold_rows
        };
        let logic_open = open && !draw_rows.is_empty();
        let row_h = *suggest_row_height().lock().unwrap();
        let n = draw_rows.len();
        let full_h = if n == 0 {
            0.0
        } else {
            list_content_height(n, row_h, 0.0).min(SUGGEST_MAX_HEIGHT)
        };
        let (height, should_draw) = list_height_tween(&self.anim_key(), logic_open, full_h);
        if list_height_anims()
            .lock()
            .unwrap()
            .contains_key(&self.anim_key())
        {
            schedule_ui_tween_frame(cx);
        }
        if !should_draw || height < 0.5 || draw_rows.is_empty() {
            if !should_draw {
                self.hold_rows.clear();
            }
            return DrawStep::done();
        }
        let walk = Walk {
            height: Size::Fixed(height),
            ..walk
        };
        let use_text = locale::t(&lang, "use");
        let new_rev = self.drawn_rev != rev && open;
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if new_rev {
                    list.set_first_id_and_scroll(0, 0.0);
                }
                list.set_item_range(cx, 0, n);
                let mut drew = 0usize;
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = draw_rows.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.label(cx, ids!(title)).set_text(cx, &row.value);
                    item.label(cx, ids!(subtitle)).set_text(cx, &row.subtitle);
                    item.label(cx, ids!(subtitle)).set_visible(cx, !row.subtitle.is_empty());
                    if paper_menu {
                        let on = row.action == "\u{2713}";
                        item.label(cx, ids!(action)).set_visible(cx, on);
                        item.label(cx, ids!(action)).set_text(cx, if on { "\u{2713}" } else { "" });
                    } else {
                        let action = if row.action.trim().is_empty() {
                            use_text.as_str()
                        } else {
                            row.action.as_str()
                        };
                        item.label(cx, ids!(action)).set_visible(cx, true);
                        item.label(cx, ids!(action)).set_text(cx, action);
                    }
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
        let live = list_height_anims()
            .lock()
            .unwrap()
            .contains_key(&self.anim_key());
        if self.is_open() || live {
            if live {
                schedule_ui_tween_frame(cx);
            }
            claim_list_nested_scroll(cx, &self.view, event);
            self.view.handle_event(cx, event, scope);
            claim_list_nested_scroll(cx, &self.view, event);
        }
    }
}

/// Tallest a "type or pick" list gets before it scrolls (about six rows).
const SUGGEST_MAX_HEIGHT: f64 = 330.0;

/// A control that opens a pick list under it — a "type or pick" field or a
/// dropdown button — wrapped so it reads as a dropdown: after the control
/// draws, a chevron is painted at its right edge, ˅ while the list is
/// closed and ˄ while it is open. The open state is read from the list's
/// own flag on every draw (and checked on every event), so the mark can
/// never be left pointing the wrong way — the old "Done · …" button faces
/// stayed after a list closed some other way (owner, 2026-11-14).
/// An open list with no rows draws nothing, so it reads as closed.
/// `field` is the list's `SuggestField` key. The mark is drawn, not a widget, so presses go
/// straight to the control inside (which keeps its own id).
#[derive(Script, ScriptHook, Widget)]
struct ComboField {
    #[deref]
    view: View,
    #[live]
    field: String,
    #[live]
    draw_closed: DrawQuad,
    #[live]
    draw_open: DrawQuad,
    /// Mark box width and its gap from the control's right edge.
    #[live(28.0)]
    mark_w: f64,
    #[live(10.0)]
    mark_right: f64,
    #[rust]
    drawn_open: Option<bool>,
}

impl ComboField {
    fn is_open(&self) -> bool {
        let st = app_state().lock().unwrap();
        st.suggest_open.map(|f| f.key()) == Some(self.field.as_str()) && !st.suggest_rows.is_empty()
    }
}

impl Widget for ComboField {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let open = self.is_open();
        self.drawn_open = Some(open);
        while self.view.draw_walk(cx, scope, walk).step().is_some() {}
        let rect = self.view.area().rect(cx);
        // Hidden or collapsed control (zero size): no mark either.
        if rect.size.x > self.mark_w * 2.0 && rect.size.y > 12.0 {
            let h = rect.size.y.min(36.0);
            let mark = Rect {
                pos: dvec2(
                    rect.pos.x + rect.size.x - self.mark_w - self.mark_right,
                    rect.pos.y + (rect.size.y - h) * 0.5,
                ),
                size: dvec2(self.mark_w, h),
            };
            if open {
                self.draw_open.draw_abs(cx, mark);
            } else {
                self.draw_closed.draw_abs(cx, mark);
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        if self.drawn_open.is_some_and(|was| was != self.is_open()) {
            self.drawn_open = None;
            self.view.redraw(cx);
        }
    }
}

/// Printer route checklists: ✓ rows and a Find printers row when the pool is empty.
#[derive(Script, ScriptHook, Widget)]
struct StationCheckList {
    #[deref]
    view: View,
    #[live]
    field: String,
    #[rust]
    was_open: bool,
    #[rust]
    hold_rows: Vec<suggest::SuggestRow>,
}

impl StationCheckList {
    fn is_open(&self) -> bool {
        app_state().lock().unwrap().suggest_open.map(|f| f.key()) == Some(self.field.as_str())
    }

    fn anim_key(&self) -> String {
        format!("station:{}", self.field)
    }
}

impl Widget for StationCheckList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (open, rows, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.suggest_open.map(|f| f.key()) == Some(self.field.as_str()),
                st.suggest_rows.clone(),
                st.locale.language.clone(),
            )
        };
        if open && !rows.is_empty() {
            self.hold_rows = rows.clone();
        }
        let draw_rows = if open && !rows.is_empty() {
            &rows
        } else {
            &self.hold_rows
        };
        let logic_open = open && !draw_rows.is_empty();
        let row_h = *suggest_row_height().lock().unwrap();
        let n = draw_rows.len();
        let full_h = if n == 0 {
            0.0
        } else {
            list_content_height(n, row_h, 0.0).min(SUGGEST_MAX_HEIGHT)
        };
        let (height, should_draw) = list_height_tween(&self.anim_key(), logic_open, full_h);
        if list_height_anims()
            .lock()
            .unwrap()
            .contains_key(&self.anim_key())
        {
            schedule_ui_tween_frame(cx);
        }
        if !should_draw || height < 0.5 || draw_rows.is_empty() {
            if !should_draw {
                self.was_open = false;
                self.hold_rows.clear();
            }
            return DrawStep::done();
        }
        let just_opened = !self.was_open && open;
        self.was_open = true;
        let walk = Walk {
            height: Size::Fixed(height),
            ..walk
        };
        let use_l = locale::t(&lang, "use");
        let route_pick = self.field == "receipt_printers" || self.field == "label_printers";
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if just_opened {
                    list.set_first_id_and_scroll(0, 0.0);
                }
                list.set_item_range(cx, 0, n);
                let mut drew = 0usize;
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = draw_rows.get(item_id) else {
                        continue;
                    };
                    if row.subtitle == FIND_PRINTERS_ROW {
                        let item = list.item(cx, item_id, id!(FindItem));
                        let btn = item.button(cx, ids!(find_printers_row));
                        if btn.text() != row.value {
                            btn.set_text(cx, &row.value);
                        }
                        item.draw_all_unscoped(cx);
                        drew += 1;
                        continue;
                    }
                    let item = list.item(cx, item_id, id!(Item));
                    item.widget(cx, ids!(title)).set_text(cx, &row.value);
                    item.widget(cx, ids!(subtitle)).set_visible(cx, false);
                    let action = row.action.trim();
                    let checked = !action.is_empty() && action != use_l;
                    item.view(cx, ids!(check_off)).set_visible(cx, !checked);
                    item.view(cx, ids!(check_on)).set_visible(cx, checked);
                    if route_pick && !checked {
                        item.label(cx, ids!(prio)).set_visible(cx, false);
                        item.button(cx, ids!(prio_up)).set_visible(cx, false);
                        item.button(cx, ids!(prio_down)).set_visible(cx, false);
                    } else if route_pick {
                        let mut ord = 0usize;
                        let mut attached_ids: Vec<usize> = Vec::new();
                        for (i, r) in draw_rows.iter().enumerate() {
                            let a = r.action.trim();
                            if a.is_empty() || a == use_l {
                                continue;
                            }
                            attached_ids.push(i);
                            ord += 1;
                            if i == item_id {
                                break;
                            }
                        }
                        let can_up = attached_ids.len() > 1 && attached_ids.first() != Some(&item_id);
                        let can_down = {
                            let last = draw_rows.iter().rposition(|r| {
                                let a = r.action.trim();
                                !a.is_empty() && a != use_l
                            });
                            last != Some(item_id)
                        };
                        item.label(cx, ids!(prio)).set_visible(cx, true);
                        item.label(cx, ids!(prio)).set_text(cx, &format!("#{ord}"));
                        item.button(cx, ids!(prio_up)).set_visible(cx, can_up);
                        item.button(cx, ids!(prio_down)).set_visible(cx, can_down);
                    }
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
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let live = list_height_anims()
            .lock()
            .unwrap()
            .contains_key(&self.anim_key());
        if !self.is_open() && !live {
            return;
        }
        if live {
            schedule_ui_tween_frame(cx);
        }
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

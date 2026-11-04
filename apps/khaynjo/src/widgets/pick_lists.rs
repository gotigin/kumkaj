/// Under a "type or pick" field (see `suggest.rs`): closed it draws
/// nothing; open it is a capped scrolling list of what the shop already
/// has for that field. `field` names which [`SuggestField`] it serves.
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
        let cols_menu = self.field == "floor_cols";
        let (open, rows, rev, lang, selected_cols) = {
            let st = app_state().lock().unwrap();
            (
                st.suggest_open.map(|f| f.key()) == Some(self.field.as_str()),
                st.suggest_rows.clone(),
                st.suggest_rev,
                st.locale.language.clone(),
                st.floor_columns,
            )
        };
        if !open || rows.is_empty() {
            return DrawStep::done();
        }
        let row_h = *suggest_row_height().lock().unwrap();
        let n = rows.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row_h, 6.0).min(SUGGEST_MAX_HEIGHT)),
            ..walk
        };
        let use_text = locale::t(&lang, "use");
        let new_rev = self.drawn_rev != rev;
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                // Always open from the top so every value is reachable by scroll.
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
                    if cols_menu {
                        let selected = selected_cols >= 1
                            && row.value.parse::<usize>().ok() == Some(selected_cols);
                        item.label(cx, ids!(action)).set_visible(cx, selected);
                        item.label(cx, ids!(action)).set_text(
                            cx,
                            if selected { "\u{2713}" } else { "" },
                        );
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

/// Order-desk kitchen dropdown: same open/rows as [`SuggestList`] for
/// `kot_station`, but each row is a checkbox item (checked = station is on
/// unsent dishes / Don't use).
#[derive(Script, ScriptHook, Widget)]
struct StationCheckList {
    #[deref]
    view: View,
    #[live]
    field: String,
    /// True while the menu was drawn open last frame — scroll resets only
    /// on open, never on checkbox refresh (that felt like a jump).
    #[rust]
    was_open: bool,
}

impl StationCheckList {
    fn is_open(&self) -> bool {
        app_state().lock().unwrap().suggest_open.map(|f| f.key()) == Some(self.field.as_str())
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
        if !open || rows.is_empty() {
            self.was_open = false;
            return DrawStep::done();
        }
        let just_opened = !self.was_open;
        self.was_open = true;
        let row_h = *suggest_row_height().lock().unwrap();
        let n = rows.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row_h, 0.0).min(SUGGEST_MAX_HEIGHT)),
            ..walk
        };
        let use_l = locale::t(&lang, "use");
        let unassigned = locale::t(&lang, "kot_unassigned");
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if just_opened {
                    list.set_first_id_and_scroll(0, 0.0);
                }
                list.set_item_range(cx, 0, n);
                let mut drew = 0usize;
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = rows.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let is_unassigned = row.value.eq_ignore_ascii_case(&unassigned);
                    item.label(cx, ids!(title)).set_text(cx, &row.value);
                    let printer_assign = self.field == "printer_assign";
                    let team_stations = self.field == "team_kot_stations";
                    // PrinterAssign keeps slot encode in subtitle — never show it.
                    item.label(cx, ids!(subtitle)).set_text(
                        cx,
                        if printer_assign { "" } else { &row.subtitle },
                    );
                    item.label(cx, ids!(subtitle)).set_visible(
                        cx,
                        !printer_assign && !row.subtitle.is_empty(),
                    );
                    let action = row.action.trim();
                    let checked = !action.is_empty() && action != use_l;
                    // Drawn box / filled+✓ — ☐ was missing from Noto (tofu).
                    item.view(cx, ids!(check_off)).set_visible(cx, !checked);
                    item.view(cx, ids!(check_on)).set_visible(cx, checked);
                    // Team person access: checkboxes only — no print-order ↑↓.
                    if team_stations || is_unassigned || (printer_assign && !checked) {
                        item.label(cx, ids!(prio)).set_visible(cx, false);
                        item.button(cx, ids!(prio_up)).set_visible(cx, false);
                        item.button(cx, ids!(prio_down)).set_visible(cx, false);
                    } else {
                        let mut ord = 0usize;
                        let mut station_ids: Vec<usize> = Vec::new();
                        for (i, r) in rows.iter().enumerate() {
                            if r.value.eq_ignore_ascii_case(&unassigned) {
                                continue;
                            }
                            if printer_assign {
                                let a = r.action.trim();
                                if a.is_empty() || a == use_l {
                                    continue;
                                }
                            }
                            station_ids.push(i);
                            ord += 1;
                            if i == item_id {
                                break;
                            }
                        }
                        let can_up = station_ids.len() > 1 && station_ids.first() != Some(&item_id);
                        let can_down = {
                            let last = if printer_assign {
                                rows.iter().rposition(|r| {
                                    let a = r.action.trim();
                                    !a.is_empty() && a != use_l
                                })
                            } else {
                                rows.iter().rposition(|r| {
                                    !r.value.eq_ignore_ascii_case(&unassigned)
                                })
                            };
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
        if !self.is_open() {
            return;
        }
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

/// The item form's category picker (see [`CategoryPick`]) — the item
/// picker's twin: closed it draws nothing; open it is a capped scrolling
/// viewport. Rows show the category's photo, else its initials; a shelf
/// folder shows one of its dishes.
#[derive(Script, ScriptHook, Widget)]
struct CategoryPickList {
    #[deref]
    view: View,
    #[rust]
    drawn_rev: u64,
}

impl Widget for CategoryPickList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (open, picks, rev, categories, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.category_picker_open,
                st.category_picks.clone(),
                st.category_picks_rev,
                st.categories.clone(),
                st.locale.language.clone(),
            )
        };
        if !open || picks.is_empty() {
            return DrawStep::done();
        }
        // Same DishRow card as the dish picker.
        let walk = Walk {
            height: Size::Fixed(
                list_content_height(picks.len(), DISH_ROW_H, DISH_ROW_GAP).min(DISH_PICK_MAX_HEIGHT),
            ),
            ..walk
        };
        let dir = item_images_dir();
        let use_text = locale::t(&lang, "use");
        let new_text = locale::t(&lang, "new_category_row");
        let mine_text = locale::t(&lang, "your_category");
        let shelf_text = locale::t(&lang, "suggested_category");
        let new_rev = self.drawn_rev != rev;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if new_rev {
                    list.set_first_id_and_scroll(0, 0.0);
                }
                list.set_item_range(cx, 0, picks.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(pick) = picks.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let (title, subtitle, initials_from, alias, photo) = match pick {
                        CategoryPick::New(name) => (
                            format!("“{name}”"),
                            new_text.clone(),
                            name.clone(),
                            None,
                            std::path::PathBuf::new(),
                        ),
                        CategoryPick::Mine(id) => match categories.iter().find(|c| &c.id == id) {
                            Some(c) => (
                                c.name.clone(),
                                mine_text.clone(),
                                c.name.clone(),
                                c.first_alias(),
                                if c.image_key.is_empty() {
                                    std::path::PathBuf::new()
                                } else {
                                    image_key_path(Some(&c.image_key), &dir)
                                },
                            ),
                            None => continue,
                        },
                        CategoryPick::Shelf(f) => (
                            f.label.clone(),
                            shelf_text.clone(),
                            f.label.clone(),
                            None,
                            f.photo.clone(),
                        ),
                    };
                    apply_thumb_avatar_kind(
                        cx,
                        &item,
                        &initials_from,
                        &photo,
                        thumbs::ThumbKind::Card,
                    );
                    item.label(cx, ids!(title)).set_text(cx, &title);
                    paint_first_alias(cx, &item, alias.as_deref());
                    item.label(cx, ids!(subtitle)).set_text(cx, &subtitle);
                    item.label(cx, ids!(action)).set_text(cx, &use_text);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        self.drawn_rev = rev;
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !app_state().lock().unwrap().category_picker_open {
            return;
        }
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

/// The item form's picker (see [`DishPick`]). Closed, it draws nothing. Open,
/// it is a scrolling viewport of at most [`DISH_PICK_MAX_HEIGHT`], so only the
/// rows on screen draw and only their card-sized photo copies are asked for.
#[derive(Script, ScriptHook, Widget)]
struct DishPickList {
    #[deref]
    view: View,
    /// `dish_picks_rev` this list last drew; a new one scrolls back to the top.
    #[rust]
    drawn_rev: u64,
}

/// Tallest the picker grows before it scrolls inside itself (css px).
const DISH_PICK_MAX_HEIGHT: f64 = 420.0;
/// Must match `DishRow` height and PortalList `spacing` on both pick lists.
const DISH_ROW_H: f64 = 118.0;
const DISH_ROW_GAP: f64 = 12.0;

impl Widget for DishPickList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (open, picks, rev, products, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.dish_picker_open,
                st.dish_picks.clone(),
                st.dish_picks_rev,
                st.products.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        if !open || picks.is_empty() {
            return DrawStep::done();
        }
        // DishRow is a fixed landscape photo card; PortalList spacing matches
        // DISH_ROW_GAP. Small results shrink the box; long ones scroll inside.
        let walk = Walk {
            height: Size::Fixed(
                list_content_height(picks.len(), DISH_ROW_H, DISH_ROW_GAP).min(DISH_PICK_MAX_HEIGHT),
            ),
            ..walk
        };
        let shelf = menu_images::shelf();
        let dir = item_images_dir();
        let use_text = locale::t(&lang, "use");
        let edit_text = locale::t(&lang, "edit");
        let own_text = locale::t(&lang, "own_item_name");
        let on_menu_text = locale::t(&lang, "on_your_menu");
        let new_rev = self.drawn_rev != rev;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if new_rev {
                    list.set_first_id_and_scroll(0, 0.0);
                }
                list.set_item_range(cx, 0, picks.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(pick) = picks.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let (title, subtitle, action, alias, photo) = match pick {
                        DishPick::Own(name) => (
                            format!("“{name}”"),
                            own_text.clone(),
                            use_text.clone(),
                            None,
                            std::path::PathBuf::new(),
                        ),
                        DishPick::Menu(id) => match products.iter().find(|p| &p.id == id) {
                            Some(p) => (
                                p.name.clone(),
                                format!(
                                    "{on_menu_text} · {}",
                                    p.price.format_localized(&currency, &fiscal)
                                ),
                                edit_text.clone(),
                                p.first_alias(),
                                image_key_path(p.image_key.as_deref(), &dir),
                            ),
                            None => continue,
                        },
                        DishPick::Shelf(i) => match shelf.get(*i) {
                            Some(d) => (
                                d.name.clone(),
                                d.folder.clone(),
                                use_text.clone(),
                                None,
                                d.path.clone(),
                            ),
                            None => continue,
                        },
                    };
                    let name_for_initials = match pick {
                        DishPick::Own(name) => name.as_str(),
                        _ => title.as_str(),
                    };
                    apply_thumb_avatar_kind(
                        cx,
                        &item,
                        name_for_initials,
                        &photo,
                        thumbs::ThumbKind::Card,
                    );
                    item.label(cx, ids!(title)).set_text(cx, &title);
                    paint_first_alias(cx, &item, alias.as_deref());
                    item.label(cx, ids!(subtitle)).set_text(cx, &subtitle);
                    item.label(cx, ids!(action)).set_text(cx, &action);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        self.drawn_rev = rev;
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !app_state().lock().unwrap().dish_picker_open {
            return;
        }
        // Pointer over the picker = it owns the wheel; the page must not move.
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

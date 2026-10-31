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
                    let (title, subtitle, initials_from, photo) = match pick {
                        CategoryPick::New(name) => (
                            format!("“{name}”"),
                            new_text.clone(),
                            name.clone(),
                            std::path::PathBuf::new(),
                        ),
                        CategoryPick::Mine(id) => match categories.iter().find(|c| &c.id == id) {
                            Some(c) => (
                                c.name.clone(),
                                mine_text.clone(),
                                c.name.clone(),
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
const DISH_ROW_H: f64 = 104.0;
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
                    let (title, subtitle, action, photo) = match pick {
                        DishPick::Own(name) => (
                            format!("“{name}”"),
                            own_text.clone(),
                            use_text.clone(),
                            std::path::PathBuf::new(),
                        ),
                        DishPick::Menu(id) => match products.iter().find(|p| &p.id == id) {
                            Some(p) => (
                                p.name.clone(),
                                format!("{on_menu_text} · {}", p.price.format_localized(&currency, &fiscal)),
                                edit_text.clone(),
                                image_key_path(p.image_key.as_deref(), &dir),
                            ),
                            None => continue,
                        },
                        DishPick::Shelf(i) => match shelf.get(*i) {
                            Some(d) => (d.name.clone(), d.folder.clone(), use_text.clone(), d.path.clone()),
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

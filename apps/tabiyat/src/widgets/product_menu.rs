#[derive(Script, ScriptHook, Widget)]
struct ProductList {
    #[deref]
    view: View,
    /// `menu_pick_rev` this grid last drew; a new one (another category, a
    /// new search, the window opened again) scrolls back to the top.
    #[rust]
    drawn_rev: u64,
    /// When set (Add dishes horizontal pages), draw this tab instead of `menu_category`.
    #[rust]
    page_category: Option<String>,
}

#[derive(Script, ScriptHook, Widget)]
struct MenuPickPager {
    #[deref]
    view: View,
    #[rust]
    last_scroll_x: f64,
}

/// How much of each dish the open order already has (milli-units), for the
/// cards' − n + and for telling a card tap from a miss.
fn cart_qty_by_product(lines: &[domain::CartLine]) -> std::collections::HashMap<String, i64> {
    let mut by = std::collections::HashMap::new();
    for line in lines {
        let milli = if line.qty_milli > 0 { line.qty_milli } else { line.qty.saturating_mul(1000) };
        *by.entry(line.product_id.clone()).or_insert(0) += milli;
    }
    by
}

impl Widget for ProductList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (products, area, currency, fiscal, lang, in_cart, rev) = {
            let st = app_state().lock().unwrap();
            let cat = self
                .page_category
                .as_deref()
                .or(st.menu_category.as_deref());
            (
                // Taps map through the same list (`add_product_to_cart`).
                st.visible_products_for(cat),
                String::new(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
                cart_qty_by_product(&st.cart.lines),
                st.menu_pick_rev,
            )
        };
        // As many tiles across as fit the window the grid sits in (never
        // fewer than two); taps read back the same count.
        let width = cx.turtle().next_walk_width(walk.width, walk.margin);
        let cols = product_columns(if width.is_finite() && width > 0.0 {
            width
        } else {
            *content_width().lock().unwrap()
        });
        *product_grid_cols().lock().unwrap() = cols;
        let rows = products.len().div_ceil(cols);
        let new_rev = self.drawn_rev != rev;
        let slots: [&[LiveId]; 6] = [ids!(c0), ids!(c1), ids!(c2), ids!(c3), ids!(c4), ids!(c5)];
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if new_rev {
                    list.set_first_id_and_scroll(0, 0.0);
                }
                list.set_item_range(cx, 0, rows);
                while let Some(item_id) = list.next_visible_item(cx) {
                    // PortalList keeps asking until its viewport is full; a
                    // zero-height row past the end would never fill it.
                    if item_id >= rows {
                        continue;
                    }
                    let item = list.item(cx, item_id, id!(Item));
                    for (col, path) in slots.iter().enumerate() {
                        let slot = item.widget(cx, *path);
                        // Slots past the column count take no room; an empty
                        // slot in the last row keeps its share so the last
                        // tiles stay the same width as the rows above.
                        slot.set_visible(cx, col < cols);
                        if col >= cols {
                            continue;
                        }
                        let tile = slot.view(cx, ids!(tile));
                        let Some(product) = products.get(item_id * cols + col) else {
                            tile.set_visible(cx, false);
                            continue;
                        };
                        tile.set_visible(cx, true);
                        let qty = in_cart.get(&product.id).copied().unwrap_or(0);
                        let on_order = qty > 0;
                        tile.view(cx, ids!(dim)).set_visible(cx, !product.available);
                        tile.view(cx, ids!(edge)).set_visible(cx, !on_order);
                        tile.view(cx, ids!(edge_on)).set_visible(cx, on_order);
                        tile.view(cx, ids!(stepper)).set_visible(cx, on_order);
                        tile.widget(cx, ids!(name))
                            .set_text(cx, &plain_display_text(&product.name));
                        paint_first_alias(cx, &tile, product.first_alias().as_deref());
                        let price = tile.label(cx, ids!(price));
                        let price_off = tile.label(cx, ids!(price_off));
                        price.set_visible(cx, product.available);
                        price_off.set_visible(cx, !product.available);
                        if product.available {
                            let now = chrono::Local::now();
                            price.set_text(
                                cx,
                                &product
                                    .price_now(&area, now)
                                    .format_localized(&currency, &fiscal),
                            );
                        } else {
                            price_off.set_text(cx, &locale::t(&lang, "sold_out"));
                        }
                        if on_order {
                            let shown = if qty % 1000 == 0 {
                                format!("{}", qty / 1000)
                            } else {
                                format!("{:.2}", qty as f64 / 1000.0)
                                    .trim_end_matches('0')
                                    .to_string()
                            };
                            tile.label(cx, ids!(step_qty)).set_text(cx, &shown);
                        }
                        load_product_card_photo(cx, &tile, product);
                        // The plate only when it carries initials (no photo)
                        // and the − n + is not in its place.
                        let initials = tile.label(cx, ids!(thumb_initials)).text();
                        tile.view(cx, ids!(initials_plate))
                            .set_visible(cx, !on_order && !initials.is_empty());
                    }
                    item.draw_all_unscoped(cx);
                }
            }
        }
        self.drawn_rev = rev;
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // The grid lives in the menu window, which holds the page behind it
        // still: the wheel and swipes scroll the grid alone.
        self.view.handle_event(cx, event, scope);
    }
}

impl MenuPickPager {
    fn page_count(&self) -> usize {
        let st = app_state().lock().unwrap();
        if !st.menu_filter.trim().is_empty() {
            return 1;
        }
        menu_pick_tab_keys(&st).len().min(MENU_PICK_PAGE_MAX)
    }

    fn snap_scroll_and_sync_tab(&mut self, cx: &mut Cx) {
        if menu_pick_scroll_programmatic() {
            return;
        }
        let searching = !app_state().lock().unwrap().menu_filter.trim().is_empty();
        if searching {
            return;
        }
        let vp_w = *menu_pick_viewport_w().lock().unwrap();
        if vp_w < 8.0 {
            return;
        }
        let scroll = self.view.view(cx, ids!(pick_scroll));
        let n = self.page_count().max(1);
        let x = scroll.get_scroll_pos().x;
        let idx = ((x / vp_w).round() as usize).min(n - 1);
        let snap_x = idx as f64 * vp_w;
        if (snap_x - x).abs() > 0.5 {
            scroll.set_scroll_pos(cx, dvec2(snap_x, 0.0));
        }
        queue_menu_pick_tab_from_scroll(idx);
        self.view.redraw(cx);
    }
}

impl Widget for MenuPickPager {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (keys, searching, sel_idx, rev) = {
            let st = app_state().lock().unwrap();
            let keys = menu_pick_tab_keys(&st);
            let searching = !st.menu_filter.trim().is_empty();
            let sel_idx = menu_pick_tab_index(&keys, &st.menu_category);
            (keys, searching, sel_idx, st.menu_pick_rev)
        };
        let n = if searching {
            1
        } else {
            keys.len().min(MENU_PICK_PAGE_MAX)
        };
        let vp_w = cx.turtle().next_walk_width(walk.width, walk.margin);
        if vp_w.is_finite() && vp_w > 8.0 {
            *menu_pick_viewport_w().lock().unwrap() = vp_w;
        }
        let scroll = self.view.view(cx, ids!(pick_scroll));
        let strip = scroll.view(cx, ids!(pick_strip));
        if searching {
            scroll.set_visible(cx, false);
        } else {
            scroll.set_visible(cx, n > 1);
        }
        if let Some(mut strip_v) = strip.borrow_mut() {
            strip_v.walk.width = Size::Fixed(vp_w * n as f64);
        }
        for i in 0..MENU_PICK_PAGE_MAX {
            let grid = strip.widget(cx, MENU_PICK_GRIDS[i]);
            let show = i < n;
            grid.set_visible(cx, show);
            if let Some(mut pl) = grid.borrow_mut::<ProductList>() {
                pl.walk.width = Size::Fixed(vp_w.max(1.0));
                let key = keys.get(i).cloned().flatten();
                pl.page_category = if searching {
                    None
                } else {
                    key.clone()
                };
                if pl.drawn_rev != rev {
                    pl.drawn_rev = 0;
                }
            }
        }
        if !searching && !menu_pick_scroll_programmatic() {
            let want_x = sel_idx as f64 * vp_w;
            let cur = scroll.get_scroll_pos().x;
            if (cur - want_x).abs() > 1.0
                && menu_pick_scroll_anim().lock().unwrap().is_none()
            {
                scroll.set_scroll_pos(cx, dvec2(want_x, 0.0));
                self.last_scroll_x = want_x;
            }
        }
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            let _ = step;
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let scroll = self.view.view(cx, ids!(pick_scroll));
        let before = scroll.get_scroll_pos().x;
        self.view.handle_event(cx, event, scope);
        let after = scroll.get_scroll_pos().x;
        if matches!(event, Event::Scroll(_)) {
            apply_hstrip_wheel(cx, &scroll, event);
        }
        if (after - before).abs() > 0.5 && !menu_pick_scroll_programmatic() {
            let vp_w = *menu_pick_viewport_w().lock().unwrap();
            if vp_w > 8.0 {
                let idx = ((after / vp_w).round() as usize).min(self.page_count().saturating_sub(1));
                queue_menu_pick_tab_from_scroll(idx);
            }
        }
        if let Event::Actions(actions) = event {
            if scroll.finger_up(actions).is_some() {
                self.last_scroll_x = after;
                self.snap_scroll_and_sync_tab(cx);
            }
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
struct MenuList {
    #[deref]
    view: View,
}

impl Widget for MenuList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (products, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            let all = st.products.clone();
            (
                st.menu_list_indices().into_iter().map(|i| all[i].clone()).collect::<Vec<_>>(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let count = products.len();
        // Fixed pitch, never a measured one: PortalList only draws the
        // visible slice, so extent/total-count under-measures and the
        // next list on the page starts early, overlapping the last rows
        // and stealing their taps. MenuCard height + PortalList spacing
        // must stay in sync. Only rows inside the page window are drawn.
        const ROW: f64 = 164.0;
        const GAP: f64 = 0.0; // MenuCard 152 + Item bottom pad 12
        let (first, last) = page_window(cx.turtle().pos().y, count, ROW, GAP);
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, ROW, GAP)),
            ..walk
        };
        if let Some(mut gap) = self.view.view(cx, ids!(top_gap)).borrow_mut() {
            gap.walk.height = Size::Fixed(first as f64 * (ROW + GAP));
        }
        if let Some(mut boxv) = self.view.view(cx, ids!(list_box)).borrow_mut() {
            boxv.walk.height = Size::Fixed(list_content_height(last.saturating_sub(first), ROW, GAP));
        }
        let editing_id = app_state().lock().unwrap().editing_product_id.clone();
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, first, last);
                list.set_first_id_and_scroll(first, 0.0);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < first || item_id >= last {
                        continue;
                    }
                    let Some(product) = products.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    // The item loaded in the form carries a ✓ so a tap's
                    // effect is visible in the list itself, not only in
                    // the form far above.
                    let shown_name = if editing_id.as_deref() == Some(product.id.as_str()) {
                        format!("✓ {}", product.name)
                    } else {
                        product.name.clone()
                    };
                    item.widget(cx, ids!(name)).set_text(cx, &plain_display_text(&shown_name));
                    paint_first_alias(cx, &item, product.first_alias().as_deref());
                    item.label(cx, ids!(price))
                        .set_text(cx, &product.price.format_localized(&currency, &fiscal));
                    let stock_line = if product.tracks_stock() {
                        format!(
                            "{} {}",
                            product.format_on_hand(),
                            locale::t(&lang, "in_stock")
                        )
                    } else if product.available {
                        locale::t(&lang, "not_tracked")
                    } else {
                        locale::t(&lang, "marked_sold_out")
                    };
                    item.label(cx, ids!(stock)).set_text(cx, &stock_line);
                    load_product_card_photo(cx, &item, product);
                    item.button(cx, ids!(sold_out)).set_visible(cx, true);
                    item.button(cx, ids!(sold_out)).set_text(
                        cx,
                        &locale::t(
                            &lang,
                            if product.available {
                                "sold_out"
                            } else {
                                "restock"
                            },
                        ),
                    );
                    item.button(cx, ids!(remove)).set_visible(cx, true);
                    item.button(cx, ids!(remove)).set_text(cx, "✕");
                    // Restore belongs to the RemovedList rows only.
                    item.button(cx, ids!(restore)).set_visible(cx, false);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Virtualized rows: redraw only when the page actually scrolls.
        // NextFrame / MouseMove here scheduled endless frames and froze the UI.
        if matches!(event, Event::Scroll(_)) {
            self.view.redraw(cx);
        }
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Items taken off the menu. Same card as [`MenuList`], one action: Restore.
/// Remove archives the row (it is never deleted), so restoring brings the item
/// back exactly as it was — including dishes copied from the app's shelf.
#[derive(Script, ScriptHook, Widget)]
struct RemovedList {
    #[deref]
    view: View,
}

impl Widget for RemovedList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (products, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.removed_products.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let count = products.len();
        // Fixed pitch (see MenuList): the same MenuCard, so the same row.
        const ROW: f64 = 164.0;
        const GAP: f64 = 0.0; // MenuCard 152 + Item bottom pad 12
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, ROW, GAP)),
            ..walk
        };
        let restore_text = locale::t(&lang, "restore_item");
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(product) = products.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.widget(cx, ids!(name)).set_text(cx, &plain_display_text(&product.name));
                    paint_first_alias(cx, &item, product.first_alias().as_deref());
                    item.label(cx, ids!(price))
                        .set_text(cx, &product.price.format_localized(&currency, &fiscal));
                    // Removed items are not for sale right now, so the stock
                    // line would only confuse — say what the row is instead.
                    item.label(cx, ids!(stock))
                        .set_text(cx, &locale::t(&lang, "removed_items"));
                    load_product_card_photo(cx, &item, product);
                    item.button(cx, ids!(sold_out)).set_visible(cx, false);
                    item.button(cx, ids!(remove)).set_visible(cx, false);
                    item.button(cx, ids!(restore)).set_visible(cx, true);
                    item.button(cx, ids!(restore)).set_text(cx, &restore_text);
                    item.draw_all_unscoped(cx);
                }
            }
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

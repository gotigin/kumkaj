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
    /// This grid's page in the Add dishes pager. Pages away from the window
    /// keep their room but draw nothing (`menu_pick_page_near`).
    #[rust]
    pager_page: Option<usize>,
}

#[derive(Script, ScriptHook, Widget)]
struct MenuPickPager {
    #[deref]
    view: View,
    #[rust]
    last_scroll_x: f64,
    /// A press on the dish pages, until it is a swipe, a scroll or a tap.
    #[rust]
    swipe: Option<PagerSwipe>,
}

/// One press on the dish pages. The dish list takes the press for its own
/// up/down scroll, so the pager reads the raw pointer: once the travel is
/// sideways it moves the strip with the finger / mouse, and the release
/// lands on the next or previous category (Khaynjo owner, 2026-11-15).
#[derive(Clone, Copy, Debug)]
struct PagerSwipe {
    start: DVec2,
    start_time: f64,
    start_x: f64,
    /// `None` until the press travelled `PAGER_SWIPE_DECIDE`; then sideways
    /// (`true`, the pager's) or up/down (`false`, the list's).
    sideways: Option<bool>,
}

/// Travel before a press counts as a swipe (under the 24 px tap travel, so
/// a short sideways swipe is never also a dish tap).
const PAGER_SWIPE_DECIDE: f64 = 12.0;
/// Release past this share of a page, or a flick, turns the page.
const PAGER_SWIPE_TURN_SHARE: f64 = 0.2;
const PAGER_FLICK_PX: f64 = 30.0;
const PAGER_FLICK_SPEED: f64 = 400.0;

/// Page a swipe lands on: the next / previous one when the release went far
/// enough or flicked, else back to where it started.
fn pager_swipe_target(start_page: usize, dx: f64, secs: f64, page_w: f64, pages: usize) -> usize {
    let speed = if secs > 0.0 { dx.abs() / secs } else { 0.0 };
    let turn = dx.abs() >= page_w * PAGER_SWIPE_TURN_SHARE
        || (dx.abs() >= PAGER_FLICK_PX && speed >= PAGER_FLICK_SPEED);
    let last = pages.saturating_sub(1);
    if !turn {
        start_page.min(last)
    } else if dx < 0.0 {
        (start_page + 1).min(last)
    } else {
        start_page.saturating_sub(1)
    }
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
        // Away from the window: keep the page's room, draw nothing.
        if let Some(page) = self.pager_page {
            let page_w = *menu_pick_viewport_w().lock().unwrap();
            let x = *menu_pick_scroll_now().lock().unwrap();
            if !menu_pick_page_near(page, x, page_w) {
                cx.walk_turtle(walk);
                return DrawStep::done();
            }
        }
        let _span = stall_span("dish_grid_draw");
        let (products, currency, fiscal, lang, in_cart, rev) = {
            let st = app_state().lock().unwrap();
            let cat = self
                .page_category
                .as_deref()
                .or(st.menu_category.as_deref());
            (
                // Taps map through the same list (`add_product_to_cart`).
                st.visible_products_for(cat),
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
                                    .price_now(now)
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
    /// Swipe the dish pages sideways with a finger or a mouse press-drag
    /// (owner, 2026-11-15): the strip follows the pointer; the release turns
    /// to the next / previous category or springs back, and the category
    /// tab follows. Returns true for a sideways move it took.
    fn pager_swipe(&mut self, cx: &mut Cx, scroll: &ViewRef, event: &Event) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(pointer) = popup_pointer(event) else {
            return false;
        };
        let pages = self.page_count();
        match pointer {
            PopupPointer::Press { abs, time } => {
                MENU_PICK_SWIPE_TOOK_PRESS.store(false, Relaxed);
                MENU_PICK_SWIPE_HELD.store(false, Relaxed);
                let on_pages = scroll.area().rect(cx).contains(abs);
                self.swipe = (pages > 1 && on_pages && !menu_pick_scroll_programmatic()).then(|| {
                    PagerSwipe {
                        start: abs,
                        start_time: time,
                        start_x: scroll.get_scroll_pos().x,
                        sideways: None,
                    }
                });
                false
            }
            PopupPointer::Move { abs } => {
                let Some(swipe) = self.swipe.as_mut() else {
                    return false;
                };
                let d = abs - swipe.start;
                if swipe.sideways.is_none() {
                    if d.length() < PAGER_SWIPE_DECIDE {
                        return false;
                    }
                    let sideways = d.x.abs() > d.y.abs();
                    swipe.sideways = Some(sideways);
                    if sideways {
                        MENU_PICK_SWIPE_HELD.store(true, Relaxed);
                        MENU_PICK_SWIPE_TOOK_PRESS.store(true, Relaxed);
                    }
                }
                if swipe.sideways != Some(true) {
                    return false;
                }
                let page_w = *menu_pick_viewport_w().lock().unwrap();
                let max_x = (page_w * pages.saturating_sub(1) as f64).max(0.0);
                let x = (swipe.start_x - d.x).clamp(0.0, max_x);
                scroll.set_scroll_pos(cx, dvec2(x, 0.0));
                *menu_pick_scroll_now().lock().unwrap() = x;
                self.view.redraw(cx);
                true
            }
            PopupPointer::Release { abs, time } => {
                let Some(swipe) = self.swipe.take() else {
                    return false;
                };
                if swipe.sideways == Some(true) {
                    let page_w = *menu_pick_viewport_w().lock().unwrap();
                    if page_w > 8.0 {
                        let start_page = (swipe.start_x / page_w).round() as usize;
                        let target = pager_swipe_target(
                            start_page,
                            abs.x - swipe.start.x,
                            time - swipe.start_time,
                            page_w,
                            pages,
                        );
                        let x = target as f64 * page_w;
                        scroll.set_scroll_pos(cx, dvec2(x, 0.0));
                        *menu_pick_scroll_now().lock().unwrap() = x;
                        self.last_scroll_x = x;
                        queue_menu_pick_tab_from_scroll(target);
                    }
                    MENU_PICK_SWIPE_HELD.store(false, Relaxed);
                    self.view.redraw(cx);
                }
                // The list and the card under the press still see the
                // release (their drag ends; the card's tap is refused by
                // `menu_pick_swipe_took_press`).
                false
            }
        }
    }

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
        // The grids live inside the scroll: hiding it while searching hid
        // the one results page too, so no item ever matched (owner
        // 2026-11-14). Searching = one page, no paging.
        scroll.set_visible(cx, true);
        if let Some(mut strip_v) = strip.borrow_mut() {
            strip_v.walk.width = Size::Fixed(vp_w * n as f64);
        }
        for i in 0..MENU_PICK_PAGE_MAX {
            let grid = strip.widget(cx, MENU_PICK_GRIDS[i]);
            let show = i < n;
            grid.set_visible(cx, show);
            if let Some(mut pl) = grid.borrow_mut::<ProductList>() {
                pl.walk.width = Size::Fixed(vp_w.max(1.0));
                pl.pager_page = Some(i);
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
                && !menu_pick_swipe_held()
                // A swipe's release queued its page: the category follows
                // on the next frame — do not pull the strip back first.
                && pending_menu_pick_tab().lock().unwrap().is_none()
            {
                scroll.set_scroll_pos(cx, dvec2(want_x, 0.0));
                self.last_scroll_x = want_x;
            }
        }
        *menu_pick_scroll_now().lock().unwrap() = scroll.get_scroll_pos().x;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            let _ = step;
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let scroll = self.view.view(cx, ids!(pick_scroll));
        // A sideways move of a held swipe is the pager's alone: the list
        // under it must not scroll up/down with the finger's wobble.
        if self.pager_swipe(cx, &scroll, event) {
            return;
        }
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

#[cfg(test)]
mod menu_pick_pager_tests {
    use super::*;

    /// Owner, 2026-11-15: a sideways swipe on the dishes turns the category
    /// page (and its tab) like the tabs do; a short one springs back.
    #[test]
    fn a_swipe_turns_one_page_or_springs_back() {
        let w = 700.0;
        // Far enough left → next page; right → previous page.
        assert_eq!(pager_swipe_target(3, -200.0, 0.5, w, 10), 4);
        assert_eq!(pager_swipe_target(3, 200.0, 0.5, w, 10), 2);
        // A short slow drag springs back; a short quick flick turns.
        assert_eq!(pager_swipe_target(3, -60.0, 0.6, w, 10), 3);
        assert_eq!(pager_swipe_target(3, -60.0, 0.1, w, 10), 4);
        // Never past the first or the last page.
        assert_eq!(pager_swipe_target(0, 300.0, 0.2, w, 10), 0);
        assert_eq!(pager_swipe_target(9, -300.0, 0.2, w, 10), 9);
    }

    /// Only the page in the window and its neighbours draw — drawing every
    /// category page made each dish tap redraw ~200 tiles.
    #[test]
    fn only_pages_near_the_window_draw() {
        let w = 700.0;
        let near = |x: f64| (0..27).filter(|&p| menu_pick_page_near(p, x, w)).collect::<Vec<_>>();
        assert_eq!(near(0.0), vec![0, 1]);
        assert_eq!(near(5.0 * w), vec![4, 5, 6]);
        // Mid-swipe between 5 and 6: both, plus one each side.
        assert_eq!(near(5.5 * w), vec![4, 5, 6, 7]);
        // No width yet (first frame): draw everything rather than nothing.
        assert!(menu_pick_page_near(20, 0.0, 0.0));
    }
}

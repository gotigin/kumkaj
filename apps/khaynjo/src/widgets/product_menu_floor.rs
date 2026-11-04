#[derive(Script, ScriptHook, Widget)]
struct ProductList {
    #[deref]
    view: View,
    /// `menu_pick_rev` this grid last drew; a new one (another category, a
    /// new search, the window opened again) scrolls back to the top.
    #[rust]
    drawn_rev: u64,
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
            (
                // Taps map through the same list (`add_product_to_cart`).
                st.visible_products(),
                st.current_area(),
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
                        tile.label(cx, ids!(name)).set_text(cx, &product.name);
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
        const ROW: f64 = 152.0;
        const GAP: f64 = 12.0;
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
                    item.label(cx, ids!(name)).set_text(cx, &shown_name);
                    paint_first_alias(cx, &item, product.first_alias().as_deref());
                    item.label(cx, ids!(price))
                        .set_text(cx, &product.price.format_localized(&currency, &fiscal));
                    let stock_line = if product.tracks_stock() {
                        format!(
                            "{} {}",
                            product.stock_qty,
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
        const ROW: f64 = 152.0;
        const GAP: f64 = 12.0;
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
                    item.label(cx, ids!(name)).set_text(cx, &product.name);
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

/// Floor: one card per dining table. Cream face, dark ink (contrast law);
/// occupied tables say so in deep-red text and the button opens their bill.
#[derive(Script, ScriptHook, Widget)]
struct FloorList {
    #[deref]
    view: View,
}

/// FA solid icons on the tile action button (no long labels).
fn floor_act_icon(state: domain::FloorState, total_paise: i64) -> Option<&'static str> {
    use domain::FloorState as F;
    match state {
        // user-plus — seat guests
        F::Free => Some("\u{f234}"),
        // check — free table
        F::Seated | F::Paid => Some("\u{f00c}"),
        F::BillPrinted if total_paise <= 0 => Some("\u{f00c}"),
        // undo — undo bill
        F::BillPrinted => Some("\u{f0e2}"),
        // file — request bill / payment pending
        F::InKitchen | F::FoodReady | F::Eating => Some("\u{f15b}"),
    }
}

fn floor_tile_labels(st: &AppState, t: &DiningTable) -> (String, String, String, Option<&'static str>, domain::FloorState, bool) {
    use domain::FloorState as F;
    let now = chrono::Local::now();
    let minutes_since = |at: &str| {
        chrono::DateTime::parse_from_rfc3339(at)
            .map(|at| {
                now.signed_duration_since(at.with_timezone(&chrono::Local))
                    .num_minutes()
                    .max(0)
            })
            .unwrap_or(0)
    };
    let state = t.floor_state;
    let words = match state {
        F::Free => st.t(state.tile_key()),
        F::Paid => format!("{} · {} min", st.t(state.tile_key()), minutes_since(&t.paid_at)),
        _ => format!("{} · {} min", st.t(state.tile_key()), minutes_since(&t.occupied_at)),
    };
    let meta = if t.running_total_paise > 0 {
        st.money(Money(t.running_total_paise))
    } else {
        String::new()
    };
    let action = floor_act_icon(state, t.running_total_paise);
    // Print / view when the table has a running order to bill.
    let bill_tools = t.running_total_paise > 0
        && matches!(
            state,
            F::InKitchen | F::FoodReady | F::Eating | F::BillPrinted | F::Seated
        );
    (t.name.clone(), words, meta, action, state, bill_tools)
}

fn floor_filtered_tables(st: &AppState) -> Vec<DiningTable> {
    let filter = st.floor_waiter_filter.trim().to_lowercase();
    if filter.is_empty() {
        return st.tables.clone();
    }
    st.tables
        .iter()
        .filter(|t| {
            let Some(store) = st.store.as_ref() else {
                return true;
            };
            let wid = store.table_waiter_id(&t.id).unwrap_or_default();
            if wid.is_empty() {
                return false;
            }
            store
                .list_staff()
                .unwrap_or_default()
                .into_iter()
                .any(|s| s.id == wid && s.name.to_lowercase().contains(&filter))
        })
        .cloned()
        .collect()
}

fn paint_floor_tile(
    cx: &mut Cx2d,
    tile: &WidgetRef,
    name: &str,
    words: &str,
    meta: &str,
    action: Option<&str>,
    state: domain::FloorState,
    bill_tools: bool,
    focused: bool,
) {
    use domain::FloorState as F;
    let backgrounds = [
        (F::Free, ids!(bg_free)),
        (F::Seated, ids!(bg_seated)),
        (F::InKitchen, ids!(bg_kitchen)),
        (F::FoodReady, ids!(bg_ready)),
        (F::Eating, ids!(bg_eating)),
        (F::BillPrinted, ids!(bg_bill)),
        (F::Paid, ids!(bg_paid)),
    ];
    for (s, bg) in &backgrounds {
        tile.view(cx, *bg).set_visible(cx, state == *s);
    }
    tile.view(cx, ids!(bg_focus)).set_visible(cx, focused);
    tile.label(cx, ids!(name)).set_text(cx, name);
    tile.label(cx, ids!(state)).set_text(cx, words);
    let meta_label = tile.label(cx, ids!(meta));
    meta_label.set_text(cx, meta);
    meta_label.set_visible(cx, !meta.is_empty());
    let act = tile.button(cx, ids!(act));
    act.set_visible(cx, action.is_some());
    if let Some(icon) = action {
        act.set_text(cx, icon);
    }
    tile.button(cx, ids!(print_ico)).set_visible(cx, bill_tools);
    tile.button(cx, ids!(view_ico)).set_visible(cx, bill_tools);
}

/// Tables grouped by area (name, indices into the filtered tables list).
fn floor_area_groups(tables: &[DiningTable]) -> Vec<(String, Vec<usize>)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < tables.len() {
        let area = tables[i].area.clone();
        let group: Vec<usize> = (i..tables.len())
            .take_while(|&j| tables[j].area == area)
            .collect();
        i += group.len();
        out.push((area, group));
    }
    out
}

fn floor_area_block_height(n_tables: usize, cols: usize) -> f64 {
    let cols = cols.max(1);
    let n_rows = n_tables.div_ceil(cols).max(1);
    // area_scroll pad (28+22 = Columns↔tables gap 10 + lane 18 + bottom 22)
    // + body pad (10+14) + tile rows + row gaps.
    // (Area name lives on the tab strip — no label height.)
    const SCROLL_PAD_Y: f64 = 28.0 + 22.0;
    const BODY_PAD_Y: f64 = 10.0 + 14.0;
    SCROLL_PAD_Y
        + BODY_PAD_Y
        + n_rows as f64 * (FLOOR_TILE_H + FLOOR_TILE_GAP)
        + 10.0 * n_rows.saturating_sub(1) as f64
}

impl Widget for FloorList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        use domain::FloorState as F;
        type Tile = (String, String, String, Option<&'static str>, F, bool);
        // One selected area tab at a time — that tab's Columns + tables.
        let (tables, tiles, focused, chosen, area_tab): (
            Vec<DiningTable>,
            Vec<Tile>,
            Vec<bool>,
            usize,
            String,
        ) = {
            let st = app_state().lock().unwrap();
            let tables = floor_filtered_tables(&st);
            let focus = st.floor_focus_table_id.clone();
            let focused = tables
                .iter()
                .map(|t| focus.as_ref() == Some(&t.id))
                .collect();
            let tiles = tables
                .iter()
                .map(|t| floor_tile_labels(&st, t))
                .collect();
            (
                tables,
                tiles,
                focused,
                st.floor_columns,
                st.floor_area_tab.clone(),
            )
        };
        let (_vp_left, vp_w) = *floor_hviewport().lock().unwrap();
        let cols = if chosen >= 1 {
            chosen
        } else if vp_w > 1.0 {
            floor_columns_for(vp_w)
        } else {
            floor_columns_for(cx.turtle().next_walk_width(walk.width, walk.margin))
        };
        *floor_grid_cols().lock().unwrap() = cols;
        let all_areas = floor_area_groups(&tables);
        // Indices into `tables` / `tiles` for the selected area only.
        let idxs: Vec<usize> = all_areas
            .iter()
            .find(|(a, _)| *a == area_tab)
            .map(|(_, idxs)| idxs.clone())
            .or_else(|| all_areas.first().map(|(_, idxs)| idxs.clone()))
            .unwrap_or_default();
        // Single area body (tab already names it) — no stacked area headings.
        let n = if idxs.is_empty() { 0 } else { 1 };
        let height = if n == 0 {
            0.0
        } else {
            floor_area_block_height(idxs.len(), cols)
        };
        let walk = Walk {
            width: Size::fill(),
            height: Size::Fixed(height.max(0.0)),
            ..walk
        };
        if let Some(mut gap) = self.view.view(cx, ids!(top_gap)).borrow_mut() {
            gap.walk.height = Size::Fixed(0.0);
        }
        if let Some(mut boxv) = self.view.view(cx, ids!(list_box)).borrow_mut() {
            boxv.walk.height = Size::Fixed(height.max(0.0));
        }
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, n);
                list.set_first_id_and_scroll(0, 0.0);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id >= n {
                        continue;
                    }
                    let item = list.item(cx, item_id, id!(Item));
                    // Area name lives on the tab strip, not above the tiles.
                    item.label(cx, ids!(area_label)).set_visible(cx, false);

                    let scroll = item.view(cx, ids!(area_scroll));
                    let body = item.view(cx, ids!(area_body));
                    // Body width = the Columns choice (or fewer if this area
                    // has fewer tables). Never the ScrollX viewport — Fill
                    // rows used to collapse to ~2 tiles on a phone so
                    // Columns=3 looked like 2 with the third clipped.
                    let cols_in_row = idxs.len().min(cols).max(1);
                    let grid_w = floor_grid_width(cols_in_row);
                    let n_rows = idxs.chunks(cols.max(1)).count().max(1);
                    // Body pad (top 10 + bottom 14) matches DSL — scroll lanes
                    // above/below the tiles inside the sideways strip.
                    const BODY_PAD_Y: f64 = 10.0 + 14.0;
                    let body_h = n_rows as f64 * (FLOOR_TILE_H + FLOOR_TILE_GAP)
                        + 10.0 * n_rows.saturating_sub(1) as f64
                        + BODY_PAD_Y;
                    if let Some(mut b) = body.borrow_mut() {
                        b.walk.width = Size::Fixed(grid_w);
                        b.walk.height = Size::Fixed(body_h.max(FLOOR_TILE_H + BODY_PAD_Y));
                    }
                    let scroll_x = scroll.get_scroll_pos().x;
                    let area_vp_w = {
                        let r = scroll.area().rect(cx);
                        if r.size.x > 1.0 {
                            r.size.x
                        } else {
                            vp_w.max(1.0)
                        }
                    };
                    // When Columns fits in one slot row, paint every column so
                    // the Fit/Fixed row is truly N tiles wide (ScrollX pans).
                    // Only window when N exceeds FLOOR_ROW_SLOTS.
                    let (first_col, last_col) = if cols_in_row <= FLOOR_ROW_SLOTS {
                        (0, cols_in_row)
                    } else {
                        floor_col_window(0.0, scroll_x, area_vp_w, cols_in_row)
                    };
                    *floor_first_col().lock().unwrap() = first_col;
                    let skipped = first_col as f64 * (FLOOR_TILE_W + FLOOR_TILE_GAP);
                    while let Some(step) = item.draw_walk(cx, scope, Walk::default()).step() {
                        if let Some(mut rows_list) = step.as_portal_list().borrow_mut() {
                            rows_list.set_item_range(cx, 0, n_rows);
                            rows_list.set_first_id_and_scroll(0, 0.0);
                            while let Some(row_id) = rows_list.next_visible_item(cx) {
                                if row_id >= n_rows {
                                    continue;
                                }
                                let row_item = rows_list.item(cx, row_id, id!(Item));
                                // Pin the row to the Columns grid width so
                                // ScrollX can pan across all N tiles.
                                if let Some(mut row) = row_item.borrow_mut::<View>() {
                                    row.walk.width = Size::Fixed(grid_w);
                                }
                                let left_gap = row_item.view(cx, ids!(left_gap));
                                left_gap.set_visible(cx, first_col > 0);
                                if let Some(mut v) = left_gap.borrow_mut() {
                                    v.walk.width =
                                        Size::Fixed((skipped - FLOOR_TILE_GAP).max(0.0));
                                }
                                let chunk_start = row_id * cols.max(1);
                                for (slot, path) in FLOOR_SLOT_IDS.iter().enumerate() {
                                    let tile = row_item.view(cx, *path);
                                    let col = first_col + slot;
                                    let shown = (col < last_col)
                                        .then(|| idxs.get(chunk_start + col))
                                        .flatten()
                                        .and_then(|i| tiles.get(*i).map(|t| (*i, t)));
                                    match shown {
                                        Some((ti, t)) => {
                                            tile.set_visible(cx, true);
                                            paint_floor_tile(
                                                cx,
                                                &tile,
                                                t.0.as_str(),
                                                t.1.as_str(),
                                                t.2.as_str(),
                                                t.3,
                                                t.4,
                                                t.5,
                                                focused.get(ti).copied().unwrap_or(false),
                                            );
                                        }
                                        None => tile.set_visible(cx, false),
                                    }
                                }
                                row_item.draw_all_unscoped(cx);
                            }
                        }
                    }
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if matches!(event, Event::Scroll(_)) {
            // Expand-to-content PortalList must not eat the wheel (page owns
            // Y). Apply X ourselves onto the area strip — claim alone left
            // the strip stuck while the page thought X was handled.
            let list = self.view.portal_list(cx, ids!(list));
            let item = list.item(cx, 0, id!(Item));
            let strip = item.view(cx, ids!(area_scroll));
            apply_hstrip_wheel(cx, &strip, event);
            self.view.redraw(cx);
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Drawn floor: tables at persisted (layout_x, layout_y, layout_w, layout_h).
#[derive(Script, ScriptHook, Widget)]
struct FloorLayoutList {
    #[deref]
    view: View,
}

impl Widget for FloorLayoutList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let slots = [
            ids!(t0), ids!(t1), ids!(t2), ids!(t3), ids!(t4), ids!(t5),
            ids!(t6), ids!(t7), ids!(t8), ids!(t9), ids!(t10), ids!(t11),
            ids!(t12), ids!(t13), ids!(t14), ids!(t15), ids!(t16), ids!(t17),
            ids!(t18), ids!(t19), ids!(t20), ids!(t21), ids!(t22), ids!(t23),
        ];
        let (tiles, canvas_h, selected): (Vec<_>, f64, Option<String>) = {
            let st = app_state().lock().unwrap();
            let tables = floor_filtered_tables(&st);
            let mut max_b = FLOOR_LAYOUT_PACK_H;
            let tiles: Vec<_> = tables
                .into_iter()
                .take(FLOOR_LAYOUT_SLOTS)
                .map(|t| {
                    let w = if t.layout_w > 0.0 {
                        t.layout_w
                    } else {
                        FLOOR_LAYOUT_PACK_W
                    };
                    let h = if t.layout_h > 0.0 {
                        t.layout_h
                    } else {
                        FLOOR_LAYOUT_PACK_H
                    };
                    max_b = max_b.max(t.layout_y + h);
                    let labels = floor_tile_labels(&st, &t);
                    (t.id, t.layout_x, t.layout_y, w, h, labels)
                })
                .collect();
            (tiles, max_b + 16.0, st.floor_layout_sel.clone())
        };
        let canvas = self.view.view(cx, ids!(canvas));
        if let Some(mut c) = canvas.borrow_mut() {
            c.walk.height = Size::Fixed(canvas_h.max(120.0));
        }
        for (slot, path) in slots.iter().enumerate() {
            let tile = canvas.view(cx, *path);
            let Some((id, x, y, w, h, (name, words, meta, action, state, bill_tools))) =
                tiles.get(slot)
            else {
                tile.set_visible(cx, false);
                continue;
            };
            tile.set_visible(cx, true);
            if let Some(mut v) = tile.borrow_mut() {
                v.walk.width = Size::Fixed(*w);
                v.walk.height = Size::Fixed(*h);
                v.walk.margin = Inset {
                    left: *x,
                    top: *y,
                    right: 0.0,
                    bottom: 0.0,
                };
                v.walk.abs_pos = None;
            }
            let mark = if selected.as_deref() == Some(id.as_str()) {
                format!("● {name}")
            } else {
                name.clone()
            };
            paint_floor_tile(
                cx,
                &tile,
                &mark,
                words,
                meta,
                *action,
                *state,
                *bill_tools,
                selected.as_deref() == Some(id.as_str()),
            );
        }
        let walk = Walk {
            height: Size::Fixed(canvas_h.max(120.0)),
            ..walk
        };
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

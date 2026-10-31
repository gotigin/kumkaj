#[derive(Script, ScriptHook, Widget)]
struct ProductList {
    #[deref]
    view: View,
}

impl Widget for ProductList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (products, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.booking_catalog_products(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let rows = products.len().div_ceil(3);
        let row = *product_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(rows, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows);
                while let Some(item_id) = list.next_visible_item(cx) {
                    // PortalList keeps asking for items until the viewport is
                    // filled. Past our row count, drawing a zero-height row
                    // (all cards invisible) never fills it → infinite loop and
                    // OS "app not responding" after unlock shows Sell.
                    if item_id >= rows {
                        continue;
                    }
                    let item = list.item(cx, item_id, id!(Item));
                    for (col, path) in [
                        (0usize, &[id!(c0)][..]),
                        (1, &[id!(c1)][..]),
                        (2, &[id!(c2)][..]),
                    ] {
                        let idx = item_id * 3 + col;
                        let card = item.widget(cx, path);
                        if let Some(product) = products.get(idx) {
                            card.set_visible(cx, true);
                            card.label(cx, ids!(name)).set_text(cx, &product.name);
                            card.label(cx, ids!(price))
                                .set_text(cx, &product.price.format_localized(&currency, &fiscal));
                            load_product_thumb(cx, &card, product);
                            let add = card.button(cx, ids!(add));
                            if product.available {
                                add.set_text(cx, &locale::t(&lang, "add_upper"));
                                add.set_enabled(cx, true);
                            } else {
                                add.set_text(cx, &locale::t(&lang, "na"));
                                add.set_enabled(cx, false);
                            }
                        } else {
                            card.set_visible(cx, false);
                        }
                    }
                    item.draw_all_unscoped(cx);
                }
                if rows > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / rows as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, product_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
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
            (
                st.products.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let count = products.len();
        let row = *menu_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(product) = products.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.label(cx, ids!(name)).set_text(cx, &product.name);
                    item.label(cx, ids!(price))
                        .set_text(cx, &product.price.format_localized(&currency, &fiscal));
                    let stock_line = if product.tracks_stock() {
                        format!("{} {}", product.stock_qty, locale::t(&lang, "in_stock"))
                    } else if product.available {
                        locale::t(&lang, "not_tracked")
                    } else {
                        locale::t(&lang, "marked_sold_out")
                    };
                    item.label(cx, ids!(stock)).set_text(cx, &stock_line);
                    load_product_thumb(cx, &item, product);
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
                    item.button(cx, ids!(remove)).set_text(cx, "✕");
                    item.draw_all_unscoped(cx);
                }
                if count > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / count as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, menu_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct CartList {
    #[deref]
    view: View,
}

impl Widget for CartList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (lines, products, currency, fiscal, lang) = {
            let st = app_state().lock().unwrap();
            let lines = if st.screen == Screen::Sales {
                st.amend_cart.lines.clone()
            } else {
                st.cart.lines.clone()
            };
            (
                lines,
                st.products.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
            )
        };
        let count = lines.len();
        let row = *cart_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0).max(56.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, lines.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(line) = lines.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let actual = line.actual_total();
                    let applied = line.applied_total();
                    let qty_milli = if line.qty_milli > 0 {
                        line.qty_milli
                    } else {
                        line.qty.saturating_mul(1000)
                    };
                    let qty_show = {
                        let whole = qty_milli / 1000;
                        let frac = qty_milli % 1000;
                        let num = if frac == 0 {
                            format!("{whole}")
                        } else {
                            format!("{whole}.{frac:03}")
                                .trim_end_matches('0')
                                .trim_end_matches('.')
                                .to_string()
                        };
                        let unit = line.unit_label.trim();
                        if unit.is_empty() {
                            num
                        } else {
                            format!("{num} {unit}")
                        }
                    };
                    item.widget(cx, ids!(label))
                        .set_text(cx, &plain_display_text(&line.name));
                    item.label(cx, ids!(qty_label)).set_text(cx, &qty_show);
                    item.label(cx, ids!(h_qty_cap))
                        .set_text(cx, &locale::t(&lang, "qty_cap"));
                    item.label(cx, ids!(h_unit_at_cap))
                        .set_text(cx, &locale::t(&lang, "unit_at_cap"));
                    item.label(cx, ids!(h_applied))
                        .set_text(cx, &locale::t(&lang, "applied"));
                    item.text_input(cx, ids!(note))
                        .set_empty_text(cx, locale::t(&lang, "note_ph"));
                    // Actual = @ × qty — always show the real money, never leave ₹0.
                    item.widget(cx, ids!(line_actual_label)).set_text(
                        cx,
                        &plain_display_text(&format!(
                            "{} {}  ({qty_show} × {})",
                            locale::t(&lang, "actual"),
                            actual.format_localized(&currency, &fiscal),
                            line.unit_price.format_localized(&currency, &fiscal)
                        )),
                    );
                    let price_input = item.text_input(cx, ids!(unit_price));
                    if !cx.has_key_focus(price_input.area()) {
                        let price_text = locale::format_major_input_ex(
                            line.unit_price.paise(),
                            &currency,
                            fiscal.decimal_sep(),
                        );
                        if price_input.text() != price_text {
                            price_input.set_text(cx, &price_text);
                        }
                    }
                    let line_total_input = item.text_input(cx, ids!(line_total));
                    // Never rewrite while focused — empty/"0" mid-edit must stay
                    // editable (same rule as unit price / note).
                    if !cx.has_key_focus(line_total_input.area()) {
                        let applied_text = locale::format_major_input_ex(
                            applied.paise(),
                            &currency,
                            fiscal.decimal_sep(),
                        );
                        if line_total_input.text() != applied_text {
                            line_total_input.set_text(cx, &applied_text);
                        }
                    }
                    let note_input = item.text_input(cx, ids!(note));
                    if !cx.has_key_focus(note_input.area()) && note_input.text() != line.note {
                        note_input.set_text(cx, &line.note);
                    }
                    let mods = products
                        .iter()
                        .find(|p| p.id == line.product_id)
                        .map(|p| p.modifiers())
                        .unwrap_or_default();
                    item.view(cx, ids!(mod_row))
                        .set_visible(cx, !mods.is_empty());
                    let pages = mods.len().div_ceil(domain::MODIFIER_PAGE).max(1);
                    let page = line.modifier_page % pages;
                    let start = page * domain::MODIFIER_PAGE;
                    item.button(cx, ids!(mod_prev))
                        .set_visible(cx, mods.len() > domain::MODIFIER_PAGE);
                    item.button(cx, ids!(mod_next))
                        .set_visible(cx, mods.len() > domain::MODIFIER_PAGE);
                    for (slot, path) in [(0, ids!(mod_0)), (1, ids!(mod_1)), (2, ids!(mod_2))] {
                        let button = item.button(cx, path);
                        match mods.get(start + slot) {
                            Some(modifier) => {
                                let on = line
                                    .applied_modifiers
                                    .iter()
                                    .any(|m| m.name == modifier.name);
                                let price = if modifier.price_paise == 0 {
                                    String::new()
                                } else {
                                    format!(
                                        " +{}",
                                        Money(modifier.price_paise)
                                            .format_localized(&currency, &fiscal)
                                    )
                                };
                                let mark = if on { "✓ " } else { "" };
                                button.set_text(cx, &format!("{mark}{}{price}", modifier.name));
                                button.set_visible(cx, true);
                            }
                            None => button.set_visible(cx, false),
                        }
                    }
                    item.draw_all_unscoped(cx);
                }
                if count > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / count as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, cart_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

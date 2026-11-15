/// The shop's own categories: avatar is the category's photo, else its
/// name initials — same placeholder rule as items.
#[derive(Script, ScriptHook, Widget)]
struct CategoryList {
    #[deref]
    view: View,
}

impl Widget for CategoryList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let cats = {
            let st = app_state().lock().unwrap();
            st.categories.clone()
        };
        let count = cats.len();
        // Full content height + page scroll. Fit rows (thumb / Use / wrap
        // name) are measured — a short Fixed pitch clips into the next
        // sibling the same way Inventory stock rows did.
        let row = *category_row_height().lock().unwrap();
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(cat) = cats.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let dir = item_images_dir();
                    let path = if cat.image_key.is_empty() {
                        std::path::PathBuf::new()
                    } else {
                        image_key_path(Some(&cat.image_key), &dir)
                    };
                    apply_thumb_avatar(cx, &item, &cat.name, &path);
                    item.widget(cx, ids!(label))
                        .set_text(cx, &plain_display_text(&cat.name));
                    paint_first_alias(cx, &item, cat.first_alias().as_deref());
                    item.button(cx, ids!(pick)).set_text(cx, &locale::t(&app_state().lock().unwrap().locale.language, "use"));
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
        remember_content_row(cx, &mut self.view, category_row_height(), row, measured_row);
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
    /// A row height change found while drawing needs one more frame: a
    /// redraw asked for from inside this draw is dropped, and closing a
    /// line then left the old tall gap under the ticket.
    #[rust]
    settle: NextFrame,
}

impl Widget for CartList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (lines, products, currency, fiscal, lang, open_line, qty_busy) = {
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
                st.open_cart_line,
                st.cart_qty_busy,
            )
        };
        let count = lines.len();
        // Full list; outer Order ScrollYView scrolls. Closed-row pitch; open
        // details add extra.
        const GAP: f64 = 8.0;
        // Heal only absurd leftover 320 seeds — never squash a learned tall pitch
        // (the old `> 140 → 96` clamp cropped cart lines).
        let row = {
            let mut g = cart_row_height().lock().unwrap();
            if *g > 280.0 {
                *g = 140.0;
            }
            *g
        };
        let open_extra = if open_line.is_some() {
            *cart_open_extra().lock().unwrap()
        } else {
            0.0
        };
        let (first, last) = (0usize, count);
        let content_h = (list_content_height(count, row, GAP) + open_extra).max(56.0);
        let walk = Walk {
            height: Size::Fixed(content_h),
            ..walk
        };
        if let Some(mut gap) = self.view.view(cx, ids!(top_gap)).borrow_mut() {
            gap.walk.height = Size::Fixed(0.0);
        }
        if let Some(mut boxv) = self.view.view(cx, ids!(list_box)).borrow_mut() {
            boxv.walk.height = Size::Fixed(content_h);
        }
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, first, last);
                // Do NOT reset scroll every frame — that fought remeasure and
                // made Added items flicker continuously.
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < first || item_id >= last {
                        continue;
                    }
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
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&line.name));
                    match products.iter().find(|p| p.id == line.product_id) {
                        Some(p) => load_product_thumb(cx, &item, p),
                        None => apply_thumb_avatar(cx, &item, &line.name, std::path::Path::new("")),
                    }
                    item.label(cx, ids!(qty_label))
                        .set_text(cx, &qty_show);
                    item.label(cx, ids!(h_unit_at_cap))
                        .set_text(cx, &locale::t(&lang, "unit_at_cap"));
                    item.label(cx, ids!(h_applied))
                        .set_text(cx, &locale::t(&lang, "applied"));
                    item.text_input(cx, ids!(note))
                        .set_empty_text(cx, locale::t(&lang, "note_ph"));
                    // The row says what the line is without opening it:
                    // "2 × ₹250 · less spicy". The amount is
                    // what the bill charges; a hand-set amount names the real one.
                    let mut meta = format!(
                        "{qty_show} × {}",
                        line.unit_price.format_localized(&currency, &fiscal)
                    );
                    for m in &line.applied_modifiers {
                        meta.push_str(&format!(" + {}", m.name));
                    }
                    if applied != actual {
                        meta.push_str(&format!(
                            " ({} {})",
                            locale::t(&lang, "actual"),
                            actual.format_localized(&currency, &fiscal)
                        ));
                    }
                    if !line.note.trim().is_empty() {
                        meta.push_str(&format!(" · {}", line.note.trim()));
                    }
                    item.widget(cx, ids!(line_meta)).set_text(cx, &plain_display_text(&meta));
                    item.label(cx, ids!(line_amount))
                        .set_text(cx, &applied.format_localized(&currency, &fiscal));
                    item.view(cx, ids!(details)).set_visible(cx, open_line == Some(item_id));
                    let price_input = item.text_input(cx, ids!(unit_price));
                    if !cx.has_key_focus(price_input.area()) {
                        let price_text =
                            locale::format_major_input_ex(
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
                        let applied_text =
                            locale::format_major_input_ex(
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
                    item.view(cx, ids!(mod_row)).set_visible(cx, !mods.is_empty());
                    let line_busy = qty_busy.map(|(i, _)| i) == Some(item_id);
                    let removing = qty_busy.map(|(_, r)| r).unwrap_or(false);
                    item.view(cx, ids!(stepper)).set_visible(cx, !line_busy);
                    item.view(cx, ids!(stepper_busy)).set_visible(cx, line_busy);
                    if line_busy {
                        item.widget(cx, ids!(qty_busy_label)).set_text(
                            cx,
                            &plain_display_text(&locale::t(
                                &lang,
                                if removing { "removing" } else { "updating_order" },
                            )),
                        );
                        item.button(cx, ids!(qty_busy_cancel))
                            .set_text(cx, &locale::t(&lang, "cancel"));
                    }
                    let pages = mods.len().div_ceil(domain::MODIFIER_PAGE).max(1);
                    let page = line.modifier_page % pages;
                    let start = page * domain::MODIFIER_PAGE;
                    item.button(cx, ids!(mod_prev))
                        .set_visible(cx, mods.len() > domain::MODIFIER_PAGE);
                    item.button(cx, ids!(mod_next))
                        .set_visible(cx, mods.len() > domain::MODIFIER_PAGE);
                    for (slot, idle, active) in [
                        (0, ids!(mod_0), ids!(mod_0_on)),
                        (1, ids!(mod_1), ids!(mod_1_on)),
                        (2, ids!(mod_2), ids!(mod_2_on)),
                    ] {
                        let idle_btn = item.button(cx, idle);
                        let active_btn = item.button(cx, active);
                        match mods.get(start + slot) {
                            Some(modifier) => {
                                let on = line.applied_modifiers.iter().any(|m| {
                                    m.name == modifier.name
                                        && (m.group.is_empty()
                                            || modifier.group.is_empty()
                                            || m.group == modifier.group)
                                });
                                let price = if modifier.price_paise == 0 {
                                    String::new()
                                } else {
                                    format!(
                                        " +{}",
                                        Money(modifier.price_paise)
                                            .format_localized(&currency, &fiscal)
                                    )
                                };
                                let label = if modifier.group.is_empty()
                                    || modifier.group == "Options"
                                {
                                    modifier.name.clone()
                                } else {
                                    format!("{}·{}", modifier.group, modifier.name)
                                };
                                let text = format!("{label}{price}");
                                idle_btn.set_text(cx, &text);
                                active_btn.set_text(cx, &text);
                                idle_btn.set_visible(cx, !on);
                                active_btn.set_visible(cx, on);
                            }
                            None => {
                                idle_btn.set_visible(cx, false);
                                active_btn.set_visible(cx, false);
                            }
                        }
                    }
                    item.draw_all_unscoped(cx);
                }
                let drew = last.saturating_sub(first);
                if drew > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / drew as f64;
                        // Reserved too short → learn + one settle frame (draw
                        // cannot grow the walk in-place without flicker).
                        // Closed rows learn their pitch. An open line learns
                        // only its own extra, so one tall details block does
                        // not stretch every other dish.
                        if open_line.is_some() {
                            // Match the drawn line. Too short clips its
                            // details; too tall is an empty band under them.
                            // Only trust extent when it is clearly inside or
                            // clearly past the box — a clip that reports the
                            // box height must not walk the extra forever.
                            let next = if extent > content_h + 1.0 {
                                Some((open_extra + extent - content_h + 8.0).min(640.0))
                            } else if content_h > extent + 24.0 {
                                let closed = (content_h - open_extra).max(0.0);
                                Some((extent - closed + 8.0).clamp(0.0, 640.0))
                            } else {
                                None
                            };
                            if let Some(next) = next {
                                if (next - open_extra).abs() > 8.0 {
                                    *cart_open_extra().lock().unwrap() = next;
                                    self.settle = cx.new_next_frame();
                                }
                            }
                        } else if extent > content_h + 1.0 {
                            *cart_row_height().lock().unwrap() = measured_row.max(row);
                            self.settle = cx.new_next_frame();
                        }
                    }
                }
            }
        }
        // Only learn closed-row pitch — open details must not inflate every line.
        if open_line.is_none() {
            remember_content_row(cx, &mut self.view, cart_row_height(), row, measured_row);
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.settle.is_event(event).is_some() {
            self.view.redraw(cx);
        }
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}


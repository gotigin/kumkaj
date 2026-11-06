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

/// Kitchen display: open KOTs in arrival order, items + state, never prices.
/// Advance walks New → Preparing → Ready → Done (ticket leaves the list).
#[derive(Script, ScriptHook, Widget)]
struct KitchenList {
    #[deref]
    view: View,
}

impl Widget for KitchenList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (kots, stations, lang) = {
            let st = app_state().lock().unwrap();
            (st.open_kots.clone(), st.stations.clone(), st.locale.language.clone())
        };
        let now = chrono::Local::now();
        let count = kots.len();
        // Fixed pitch + page window: drawing every KotCard (Overlay + 3 faces)
        // each frame froze Android touch-move when the list was expand-to-content
        // (DEV.md bug 6c). Page ScrollYView still owns the gesture. Do not
        // remeasure from the visible slice — that under-sizes total height.
        const GAP: f64 = 10.0;
        // KotCard is 200 high (DSL) plus the Item's 8 bottom pad. A Fit card
        // on a guessed 150 pitch cut the lower tickets off (DEV.md bug 8).
        const KOT_ROW: f64 = 208.0;
        let row = KOT_ROW;
        let (first, last) = page_window(cx.turtle().pos().y, count, row, GAP);
        let walk = Walk {
            height: Size::Fixed(list_content_height(count, row, GAP)),
            ..walk
        };
        if let Some(mut gap) = self.view.view(cx, ids!(top_gap)).borrow_mut() {
            gap.walk.height = Size::Fixed(first as f64 * (row + GAP));
        }
        if let Some(mut boxv) = self.view.view(cx, ids!(list_box)).borrow_mut() {
            boxv.walk.height =
                Size::Fixed(list_content_height(last.saturating_sub(first), row, GAP));
        }
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, first, last);
                list.set_first_id_and_scroll(first, 0.0);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < first || item_id >= last {
                        continue;
                    }
                    let Some(kot) = kots.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let station = if kot.station_name.is_empty() {
                        locale::t(&lang, "kitchen")
                    } else {
                        kot.station_name.clone()
                    };
                    let mins = chrono::DateTime::parse_from_rfc3339(&kot.created_at)
                        .map(|at| {
                            now.signed_duration_since(at.with_timezone(&chrono::Local))
                                .num_minutes()
                                .max(0)
                        })
                        .unwrap_or(0);
                    let target = stations
                        .iter()
                        .find(|s| s.id == kot.station_id)
                        .map(|s| s.target_prep_mins)
                        .unwrap_or(15);
                    let band = domain::kot_age_band(mins, target);
                    item.view(cx, ids!(kotcard.bg_ok))
                        .set_visible(cx, band == domain::KotAgeBand::Normal);
                    item.view(cx, ids!(kotcard.bg_amber))
                        .set_visible(cx, band == domain::KotAgeBand::Amber);
                    item.view(cx, ids!(kotcard.bg_red))
                        .set_visible(cx, band == domain::KotAgeBand::Red);
                    // KOT number first (kitchens call tickets by number); a
                    // cancel slip says so before anything else; who sent it.
                    let number = if kot.kot_no > 0 { format!("#{} · ", kot.kot_no) } else { String::new() };
                    let cancel = if kot.is_cancel() { "✕ CANCEL · " } else { "" };
                    let by = if kot.created_by.is_empty() { String::new() } else { format!(" · {}", kot.created_by) };
                    item.label(cx, ids!(kotcard.title)).set_text(
                        cx,
                        &format!("{cancel}{number}{station} · {mins} min{by}"),
                    );
                    // Summary is capped at draw: PortalList rows share one
                    // measured height, so an unbounded multi-line summary
                    // would make the measured row oscillate between draws.
                    // Full text stays on the store record (and prints).
                    let summary: String = kot.summary.chars().take(140).collect();
                    let summary = if summary.len() < kot.summary.len() {
                        format!("{summary}…")
                    } else {
                        summary
                    };
                    item.label(cx, ids!(kotcard.summary)).set_text(cx, &summary);
                    let (state, next) = match kot.status {
                        KotStatus::New => (
                            locale::t(&lang, "kot_new_waiting"),
                            locale::t(&lang, "kot_start_btn"),
                        ),
                        KotStatus::Preparing => (
                            locale::t(&lang, "kot_preparing"),
                            locale::t(&lang, "kot_mark_ready"),
                        ),
                        KotStatus::Ready => (
                            locale::t(&lang, "kot_ready_serve"),
                            locale::t(&lang, "kot_done_btn"),
                        ),
                        KotStatus::Completed => (
                            locale::t(&lang, "kot_done_cap"),
                            locale::t(&lang, "kot_done_btn"),
                        ),
                        KotStatus::Cancelled => (
                            locale::t(&lang, "kot_cancelled"),
                            locale::t(&lang, "kot_done_btn"),
                        ),
                    };
                    item.label(cx, ids!(kotcard.state)).set_text(cx, &state);
                    item.button(cx, ids!(kotcard.advance)).set_text(cx, &next);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Virtualized rows: redraw only when the page actually scrolls.
        if matches!(event, Event::Scroll(_)) {
            self.view.redraw(cx);
        }
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
        let (lines, products, stations, currency, fiscal, lang, open_line, kot_heads, split_pick) = {
            let st = app_state().lock().unwrap();
            let split_pick = if st.split_mode && st.screen == Screen::Pos { st.split_pick.clone() } else { Vec::new() };
            let lines = if st.screen == Screen::Sales {
                st.amend_cart.lines.clone()
            } else {
                st.cart.lines.clone()
            };
            (
                lines,
                st.products.clone(),
                st.stations.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.locale.language.clone(),
                st.open_cart_line,
                st.kot_heads.clone(),
                split_pick,
            )
        };
        let count = lines.len();
        // Full list; outer Order ScrollYView scrolls. Closed-row pitch + KOT
        // group headers ("Not sent yet" / "KOT n"); open details add extra.
        const GAP: f64 = 8.0;
        const KOT_HEAD_H: f64 = 22.0;
        // Heal only absurd leftover 320 seeds — never squash a learned tall pitch
        // (the old `> 140 → 96` clamp cropped cart lines into Kitchen).
        let row = {
            let mut g = cart_row_height().lock().unwrap();
            if *g > 280.0 {
                *g = 140.0;
            }
            *g
        };
        let head_n = lines
            .iter()
            .enumerate()
            .filter(|(i, line)| {
                *i == 0
                    || lines
                        .get(i - 1)
                        .map(|prev| prev.kot_id != line.kot_id)
                        .unwrap_or(true)
            })
            .count();
        let open_extra = if open_line.is_some() {
            *cart_open_extra().lock().unwrap()
        } else {
            0.0
        };
        let (first, last) = (0usize, count);
        let content_h = (list_content_height(count, row, GAP)
            + head_n as f64 * KOT_HEAD_H
            + open_extra)
            .max(56.0);
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
                    // A header where a new KOT group starts (sent lines come
                    // first, in KOT order; new lines last).
                    let group_starts = item_id == 0
                        || lines.get(item_id - 1).map(|p| p.kot_id != line.kot_id).unwrap_or(true);
                    let head = if !group_starts {
                        None
                    } else if line.kot_id == QUEUED_KOT {
                        Some(locale::t(&lang, "kot_waiting"))
                    } else if line.kot_id.is_empty() {
                        // Only worth saying when the kitchen already has
                        // some of this order.
                        lines
                            .iter()
                            .any(|l| !l.kot_id.is_empty())
                            .then(|| locale::t(&lang, "new_items_head"))
                    } else {
                        Some(match kot_heads.get(&line.kot_id) {
                            Some((no, time)) if *no > 0 => format!("KOT {no} · {time}"),
                            _ => locale::t(&lang, "line_sent_state"),
                        })
                    };
                    item.label(cx, ids!(kot_head)).set_visible(cx, head.is_some());
                    item.label(cx, ids!(kot_head)).set_text(cx, &head.unwrap_or_default());
                    item.label(cx, ids!(qty_label))
                        .set_text(cx, &qty_show);
                    item.label(cx, ids!(h_unit_at_cap))
                        .set_text(cx, &locale::t(&lang, "unit_at_cap"));
                    item.label(cx, ids!(h_applied))
                        .set_text(cx, &locale::t(&lang, "applied"));
                    item.text_input(cx, ids!(note))
                        .set_empty_text(cx, locale::t(&lang, "note_ph"));
                    // The row says what the line is without opening it:
                    // "2 × ₹250 · Sent to kitchen · less spicy". The amount is
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
                    if !line.running_line_id.is_empty() && split_pick.contains(&line.running_line_id) {
                        meta = format!("→ {} · {meta}", locale::t(&lang, "split_marked"));
                    }
                    // Sent or not is the group header's job; NC is the line's.
                    if line.complimentary {
                        meta.push_str(&format!(" · {}", locale::t(&lang, "nc_short")));
                    }
                    // Unsent lines: which kitchen gets this dish (from the menu item).
                    // Sent lines: whether this dish is in the next recovery print.
                    if !line.is_sent() {
                        if line.hold_send {
                            meta.push_str(&format!(" · {}", locale::t(&lang, "hold_send_meta")));
                        } else {
                            let sid = line.kitchen_station_id(&products);
                            let dest = if sid.is_empty() {
                                locale::t(&lang, "kot_unassigned")
                            } else {
                                stations
                                    .iter()
                                    .find(|s| s.id == sid)
                                    .map(|s| s.name.clone())
                                    .unwrap_or_else(|| locale::t(&lang, "kot_unassigned"))
                            };
                            meta.push_str(&format!(" · → {dest}"));
                        }
                    } else if line.reprint_skip {
                        meta.push_str(&format!(" · {}", locale::t(&lang, "reprint_skip_meta")));
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
                    // Sent to the kitchen: no ± (quantity changes only by a
                    // reasoned Cancel). Real table lines have a running id;
                    // counter-only KOTs use "counter" and cannot cancel here.
                    let has_running = line.is_sent()
                        && !line.running_line_id.is_empty()
                        && line.running_line_id != "counter"
                        && !line.running_line_id.starts_with(QUEUED_LINE);
                    let on_table = has_running;
                    let unsent = !line.is_sent();
                    let sent = line.is_sent();
                    item.view(cx, ids!(stepper)).set_visible(cx, unsent);
                    item.button(cx, ids!(hold_send_btn)).set_visible(cx, unsent);
                    item.button(cx, ids!(reprint_incl_btn)).set_visible(cx, sent);
                    item.button(cx, ids!(line_cancel_sum)).set_visible(cx, has_running);
                    if unsent {
                        item.button(cx, ids!(hold_send_btn)).set_text(
                            cx,
                            &locale::t(
                                &lang,
                                if line.hold_send {
                                    "hold_send_on"
                                } else {
                                    "hold_send"
                                },
                            ),
                        );
                    }
                    if sent {
                        item.button(cx, ids!(reprint_incl_btn)).set_text(
                            cx,
                            &locale::t(
                                &lang,
                                if line.reprint_skip {
                                    "reprint_skip"
                                } else {
                                    "reprint_incl"
                                },
                            ),
                        );
                    }
                    if has_running {
                        item.button(cx, ids!(line_cancel_sum))
                            .set_text(cx, &locale::t(&lang, "cancel_item_short"));
                    }
                    item.view(cx, ids!(sent_row)).set_visible(cx, on_table);
                    if on_table {
                        item.button(cx, ids!(line_cancel)).set_text(cx, &locale::t(&lang, "cancel_item_short"));
                        item.button(cx, ids!(line_nc)).set_text(
                            cx,
                            &locale::t(&lang, if line.complimentary { "charge_again_short" } else { "nc_short" }),
                        );
                        item.button(cx, ids!(line_move)).set_text(cx, &locale::t(&lang, "move_short"));
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
                        // Body pitch only — heads are reserved separately above.
                        let body = (extent - head_n as f64 * KOT_HEAD_H).max(0.0);
                        measured_row = body / drew as f64;
                        // Reserved too short → learn + one settle frame (draw
                        // cannot grow the walk in-place without flicker).
                        // Closed rows learn their pitch. An open line learns
                        // only its own extra, so one tall details block does
                        // not stretch every other dish — and does not clip
                        // Cancel / NC / Move into Kitchen station.
                        if open_line.is_some() {
                            // Match the drawn line. Too short clips Cancel /
                            // NC / Move. Too tall is the empty band under them.
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


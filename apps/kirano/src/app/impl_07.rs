impl App {
    fn handle_modifier_chips(
        &mut self,
        cx: &mut Cx,
        actions: &Actions,
        item: &WidgetRef,
        index: usize,
        amend: bool,
    ) {
        let slot = if item.button(cx, ids!(mod_0)).clicked(actions)
            || item.button(cx, ids!(mod_0_on)).clicked(actions)
        {
            Some(0)
        } else if item.button(cx, ids!(mod_1)).clicked(actions)
            || item.button(cx, ids!(mod_1_on)).clicked(actions)
        {
            Some(1)
        } else if item.button(cx, ids!(mod_2)).clicked(actions)
            || item.button(cx, ids!(mod_2_on)).clicked(actions)
        {
            Some(2)
        } else {
            None
        };
        let page_delta = if item.button(cx, ids!(mod_next)).clicked(actions) {
            Some(1)
        } else if item.button(cx, ids!(mod_prev)).clicked(actions) {
            Some(-1)
        } else {
            None
        };
        if slot.is_none() && page_delta.is_none() {
            return;
        }
        // Chip tap can fire while Unit @ / Applied still holds focus — commit
        // typed values before the modifier reprice + sync_chrome.
        if amend {
            self.commit_amend_row_fields(cx, index, item);
        } else {
            self.commit_cart_row_fields(cx, index, item);
        }
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let product_id = {
                let cart = if amend { &st.amend_cart } else { &st.cart };
                cart.lines
                    .get(index)
                    .map(|l| l.product_id.clone())
                    .ok_or_else(|| "Missing cart line".to_string())?
            };
            let (mods, groups) = st
                .products
                .iter()
                .find(|p| p.id == product_id)
                .map(|p| (p.modifiers(), p.modifier_groups()))
                .unwrap_or_default();
            let cart = if amend {
                &mut st.amend_cart
            } else {
                &mut st.cart
            };
            if let Some(delta) = page_delta {
                cart.shift_modifier_page(index, delta, mods.len());
                return Ok(String::new());
            }
            let page = cart
                .lines
                .get(index)
                .map(|l| l.modifier_page)
                .unwrap_or(0);
            let pages = mods.len().div_ceil(domain::MODIFIER_PAGE).max(1);
            let start = (page % pages) * domain::MODIFIER_PAGE;
            let modifier = mods
                .get(start + slot.unwrap_or(0))
                .cloned()
                .ok_or_else(|| "No modifier there".to_string())?;
            let on = cart.toggle_modifier_in_groups(index, &modifier, &groups)?;
            let price = st.money(modifier.price());
            let label = if modifier.group.is_empty() || modifier.group == "Options" {
                modifier.name.clone()
            } else {
                format!("{} · {}", modifier.group, modifier.name)
            };
            Ok(if on {
                format!("{label} · {price}")
            } else {
                format!("{label} removed")
            })
        })();
        match result {
            Ok(status) if !status.is_empty() => self.set_status(cx, status),
            Ok(_) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// "Entered ₹300 of ₹500 · ₹200 left" under the expense's ways.
    fn sync_parts_status(&mut self, cx: &mut Cx) {
        let total_text = self.ui.text_input(cx, ids!(buy_expense_amount)).text();
        let text = {
            let st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let total = locale::parse_major_input(total_text.trim(), &currency).unwrap_or(0);
            let entered: i64 = st
                .pay_parts
                .iter()
                .filter_map(|p| locale::parse_major_input(p.amount.trim(), &currency).ok())
                .sum();
            if total <= 0 || st.pay_parts.len() < 2 {
                String::new()
            } else {
                st.t("parts_status")
                    .replace("{a}", &st.money(Money(entered)))
                    .replace("{b}", &st.money(Money(total)))
                    .replace("{c}", &st.money(Money((total - entered).max(0))))
            }
        };
        self.ui.label(cx, ids!(buy_parts_status)).set_text(cx, &text);
        self.ui.button(cx, ids!(buy_parts_add)).set_text(cx, &app_state().lock().unwrap().t("add_another_way"));
    }

    /// Open one ticket line's details, or close them when it is already open.
    fn toggle_cart_line(&mut self, cx: &mut Cx, index: usize) {
        {
            let mut st = app_state().lock().unwrap();
            st.open_cart_line = if st.open_cart_line == Some(index) { None } else { Some(index) };
        }
        self.sync_chrome(cx);
    }

    /// Schedule −/+ so Progress + Cancel can paint before books work (DEV 7b).
    fn bump_cart_line(&mut self, cx: &mut Cx, index: usize, delta: i64) {
        let busy_msg = {
            let mut st = app_state().lock().unwrap();
            if st.cart_qty_busy.is_some() || st.pending_cart_bump.is_some() {
                return;
            }
            let Some(line) = st.cart.lines.get(index) else {
                return;
            };
            if line.is_sent() {
                return;
            }
            let removing = if delta < 0 {
                let step = line.bump_step_milli().saturating_mul(delta.abs());
                line.qty_milli <= step
            } else {
                false
            };
            let busy_msg = if removing {
                st.t("removing")
            } else {
                st.t("updating_order")
            };
            st.cart_qty_busy = Some((index, removing));
            st.pending_cart_bump = Some((index, delta));
            if !st.cart_qty_timer.is_empty() {
                cx.stop_timer(st.cart_qty_timer);
            }
            st.cart_qty_timer = cx.start_timeout(0.0);
            busy_msg
        };
        self.set_status_line_only(cx, &busy_msg);
        self.sync_chrome(cx);
    }

    fn cancel_cart_qty_busy(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.pending_cart_bump = None;
            st.cart_qty_busy = None;
            if !st.cart_qty_timer.is_empty() {
                cx.stop_timer(st.cart_qty_timer);
                st.cart_qty_timer = Timer::empty();
            }
        }
        self.set_status(cx, app_t("cancelled"));
        self.sync_chrome(cx);
    }

    fn run_pending_cart_bump(&mut self, cx: &mut Cx) {
        let pending = {
            let mut st = app_state().lock().unwrap();
            if !st.cart_qty_timer.is_empty() {
                st.cart_qty_timer = Timer::empty();
            }
            st.pending_cart_bump.take()
        };
        let Some((index, delta)) = pending else {
            return;
        };
        let done_msg = {
            let st = app_state().lock().unwrap();
            let removing = st.cart_qty_busy.map(|(_, r)| r).unwrap_or(false);
            st.t(if removing { "removed" } else { "order_updated" })
        };
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            match st.cart.lines.get(index).cloned() {
                None => None,
                Some(line) if line.is_sent() => None,
                Some(line) => {
                    let blocked = if delta > 0 {
                        st.products
                            .iter()
                            .find(|p| p.id == line.product_id)
                            .filter(|p| p.tracks_stock())
                            .filter(|p| {
                                let step = line.bump_step_milli();
                                st.cart.qty_milli_of(&line.product_id).saturating_add(step)
                                    > p.on_hand_milli()
                            })
                            .map(|p| format!("Only {} of {} in stock", p.format_on_hand(), p.name))
                    } else {
                        None
                    };
                    if let Some(e) = blocked {
                        Some(e)
                    } else {
                        let before_len = st.cart.lines.len();
                        st.cart.bump_qty(index, delta);
                        if st.cart.lines.len() < before_len {
                            match st.open_cart_line {
                                Some(open) if open == index => st.open_cart_line = None,
                                Some(open) if open > index => {
                                    st.open_cart_line = Some(open - 1);
                                }
                                _ => {}
                            }
                        }
                        st.status = done_msg.clone();
                        None
                    }
                }
            }
        };
        app_state().lock().unwrap().cart_qty_busy = None;
        if let Some(e) = err {
            self.set_status(cx, e);
        } else {
            let msg = app_state().lock().unwrap().status.clone();
            if !msg.is_empty() {
                self.set_status_line_only(cx, &msg);
            }
        }
        self.sync_chrome(cx);
    }

    /// Claim the charge slot. False means a charge is already running, and the
    /// caller must return without touching the cart.
    fn begin_pay(&mut self, cx: &mut Cx) -> bool {
        {
            let mut st = app_state().lock().unwrap();
            if st.pay_in_flight {
                st.status = "Charging — one moment".into();
            } else {
                st.pay_in_flight = true;
                return true;
            }
        }
        self.sync_chrome(cx);
        false
    }

    fn end_pay(&mut self) {
        app_state().lock().unwrap().pay_in_flight = false;
    }

    fn show_undo_snackbar(&mut self, cx: &mut Cx, sale_id: String, text: String) {
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
        }
        let secs = SnackKind::clamp_secs(st.snack_secs);
        st.undo_sale_id = Some(sale_id);
        st.undo_snack_text = text;
        st.undo_timer = cx.start_timeout(secs);
    }

    fn clear_undo_snackbar(&mut self, cx: &mut Cx) {
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
            st.undo_timer = Timer::empty();
        }
        st.undo_sale_id = None;
        st.undo_snack_text.clear();
    }

    fn undo_last_sale(&mut self, cx: &mut Cx) {
        let result = (|| -> Result<(), String> {
            let (is_client, sale_id, actor) = {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .undo_sale_id
                    .clone()
                    .ok_or_else(|| "Nothing to undo".to_string())?;
                (st.device_role == DeviceRole::Client, sale_id, actor)
            };
            if is_client {
                let mut st = app_state().lock().unwrap();
                st.lan_job(
                    "/undo",
                    &net::RemoteUndoRequest { sale_id: sale_id.clone() },
                    "Undo".into(),
                    "Sale undone on the shop's books".into(),
                    phone::Follow::Refresh,
                    false,
                )?;
                st.status = "Undoing…".into();
                st.undo_sale_id = None;
                st.undo_snack_text.clear();
                return Ok(());
            }
            let mut st = app_state().lock().unwrap();
            let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
            let sale = store.quick_undo_sale(&sale_id, &actor)?;
            st.cart = Cart::from_sale(&sale);
            st.open_cart_line = None;
            st.selected_customer_id = sale.customer_id.clone();
            st.last_receipt = format!("Undone {}", &sale.id[..8.min(sale.id.len())]);
            st.status = "Sale undone — cart restored".into();
            st.undo_sale_id = None;
            st.undo_snack_text.clear();
            st.reload_all()?;
            Ok(())
        })();
        {
            let mut st = app_state().lock().unwrap();
            if !st.undo_timer.is_empty() {
                cx.stop_timer(st.undo_timer);
                st.undo_timer = Timer::empty();
            }
        }
        if let Err(e) = result {
            self.set_status(cx, e);
            self.clear_undo_snackbar(cx);
        } else {
            self.go_peer(cx, Screen::Pos);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_total_from_input(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, true);
    }

    /// Live Rounded edits must move totals immediately — waiting for blur
    /// leaves the field on the previous total and confuses the cashier.
    fn apply_cart_total_from_input_live(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, false);
    }

    fn apply_cart_total_from_input_inner(&mut self, cx: &mut Cx, report_errors: bool) {
        let total_field = self.ui.text_input(cx, ids!(cart_total_input));
        let text = total_field.text();
        let field_focused = cx.has_key_focus(total_field.area());
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.cart.is_empty() {
                return Err("Cart is empty".into());
            }
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(&text, &currency)?;
            if minor < 0 {
                return Err("Rounded Total cannot be negative".into());
            }
            if minor == 0 {
                let allow = st
                    .store
                    .as_ref()
                    .and_then(|s| s.allow_zero_sales().ok())
                    .unwrap_or(true);
                if !allow {
                    return Err("Zero-value sales are turned off in Settings".into());
                }
            }
            let applied = st.cart.applied_total().paise();
            // Live `changed` can fire with a stale field after ADD. Ignore
            // mismatches when there is no explicit bill round and the seller
            // is not actively editing this field.
            if !report_errors
                && st.cart.charge_total.is_none()
                && minor != applied
                && !field_focused
            {
                return Ok(());
            }
            st.cart.set_charge_total(Money(minor))?;
            st.status = if st.cart.rounding().paise() == 0
                && st.cart.applied_total().paise() == st.cart.actual_total().paise()
            {
                "Rounded Total equals Applied Total".into()
            } else {
                format!(
                    "Actual {} · Applied {} · Rounded {} · round {}",
                    st.money(st.cart.actual_total()),
                    st.money(st.cart.applied_total()),
                    st.money(st.cart.rounded_total()),
                    st.money(st.cart.rounding())
                )
            };
            Ok(())
        })();
        if let Err(e) = result
            && report_errors {
                self.set_status(cx, e);
            }
        self.sync_chrome(cx);
    }

    fn apply_cart_unit_price(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_unit_price(index, Money(minor))?;
            st.status = format!(
                "Unit price set · line {}",
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.line_total())
                        .unwrap_or_default()
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_line_total(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_line_total(index, Money(minor))?;
            st.status = format!(
                "Line Applied {} · Actual {} · @ unchanged {}",
                st.money(Money(minor)),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.actual_total())
                        .unwrap_or(Money(0))
                ),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.unit_price)
                        .unwrap_or(Money(0))
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_note(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.cart.set_note(index, text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Pull Unit @ / Applied / note from a ticket row into the cart before any
    /// action that sync-paints (close, Cancel/NC/Move, ±, modifiers, pay, Send
    /// KOT). Quiet — no status spam; caller syncs when needed.
    fn commit_cart_row_fields(&mut self, cx: &mut Cx, index: usize, item: &WidgetRef) {
        let unit_text = item.text_input(cx, ids!(unit_price)).text();
        let total_text = item.text_input(cx, ids!(line_total)).text();
        let note_text = item.text_input(cx, ids!(note)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if index >= st.cart.lines.len() {
                return Ok(());
            }
            let currency = st.locale.currency.clone();
            let unit_minor = locale::parse_major_input(&unit_text, &currency)?;
            st.cart.set_unit_price(index, Money(unit_minor))?;
            let total_minor = locale::parse_major_input(&total_text, &currency)?;
            st.cart.set_line_total(index, Money(total_minor))?;
            st.cart.set_note(index, note_text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
    }

    fn commit_amend_row_fields(&mut self, cx: &mut Cx, index: usize, item: &WidgetRef) {
        let unit_text = item.text_input(cx, ids!(unit_price)).text();
        let total_text = item.text_input(cx, ids!(line_total)).text();
        let note_text = item.text_input(cx, ids!(note)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if index >= st.amend_cart.lines.len() {
                return Ok(());
            }
            let currency = st.locale.currency.clone();
            let unit_minor = locale::parse_major_input(&unit_text, &currency)?;
            st.amend_cart.set_unit_price(index, Money(unit_minor))?;
            let total_minor = locale::parse_major_input(&total_text, &currency)?;
            st.amend_cart.set_line_total(index, Money(total_minor))?;
            st.amend_cart.set_note(index, note_text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
    }

    /// Commit the open Order line from its widgets — for Take payment / Send
    /// KOT before the cart portal loop has run this frame.
    fn commit_open_cart_line_fields(&mut self, cx: &mut Cx) {
        let Some(index) = app_state().lock().unwrap().open_cart_line else {
            return;
        };
        let host = self.ui.widget(cx, ids!(cart_list));
        let list = host.portal_list(cx, ids!(list));
        let item = list.item(cx, index, live_id!(Item));
        self.commit_cart_row_fields(cx, index, &item);
    }

    fn clear_menu_form(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.dish_picker_open = false;
            st.category_picker_open = false;
            st.form_category_photo = None;
            st.form_category_auto = false;
        }
        for path in [
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_barcode_input),
            ids!(menu_hsn_input),
            ids!(menu_composition_input),
            ids!(menu_mrp_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_modifiers_input),
            ids!(menu_variations_input),
            ids!(menu_group_input),
            ids!(menu_schedule_input),
            ids!(menu_recipe_input),
        ] {
            self.ui.text_input(cx, path).set_text(cx, "");
        }
        let mut st = app_state().lock().unwrap();
        st.editing_product_id = None;
        // A new grocery item is bought to sell and counted. "Don't count stock" is the exception.
        st.form_resale = true;
        st.draft_image_key = None;
        st.form_category_id = None;
        st.form_category_label.clear();
    }

    /// Leave the item currently in the form and start a blank one.
    fn start_new_menu_item(&mut self, cx: &mut Cx) {
        self.clear_menu_form(cx);
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    /// Tap loads that item into the form. A different item replaces it.
    /// The same item stays as it is, so a second tap does not wipe unsaved edits.
    /// New item is the way back to adding.
    fn open_menu_item(&mut self, cx: &mut Cx, index: usize) {
        app_state().lock().unwrap().dish_picker_open = false;
        let loaded = {
            let st = app_state().lock().unwrap();
            let Some(product) = st.products.get(index).cloned() else {
                return;
            };
            if st.editing_product_id.as_deref() == Some(product.id.as_str()) {
                return;
            } else {
                let currency = st.locale.currency.clone();
                let store = st.store.as_ref();
                let tax = store
                    .map(|s| menu_tax_text(s, &product))
                    .unwrap_or_default();
                let category = store
                    .and_then(|s| s.list_categories().ok())
                    .map(|list| {
                        name_for_id(
                            list.iter().map(|row| (row.id.as_str(), row.name.as_str())),
                            &product.category_id,
                        )
                    })
                    .unwrap_or_default();
                let aliases = product.aliases().join(", ");
                let modifiers = modifiers_as_typed(&product.modifier_groups(), &currency);
                let price = locale::format_major_input(product.price.paise(), &currency);
                let mrp = if product.mrp_paise > 0 {
                    locale::format_major_input(product.mrp_paise, &currency)
                } else {
                    String::new()
                };
                let schedule = schedule_as_typed(&product.schedule(), &currency);
                let recipe = store
                    .and_then(|s| s.recipe(&product.id).ok())
                    .map(|r| store::recipe_as_typed(&r))
                    .unwrap_or_default();
                Some((product, tax, category, aliases, modifiers, price, mrp, schedule, recipe))
            }
        };
        let Some((product, tax, category, aliases, modifiers, price, mrp, schedule, recipe)) = loaded else {
            return;
        };
        self.ui.text_input(cx, ids!(menu_recipe_input)).set_text(cx, &recipe);
        self.ui.text_input(cx, ids!(menu_schedule_input)).set_text(cx, &schedule);
        // Sizes are items of their own: the quick-entry field starts empty
        // (filling it adds more sizes as new items).
        self.ui.text_input(cx, ids!(menu_variations_input)).set_text(cx, "");
        self.ui
            .text_input(cx, ids!(menu_group_input))
            .set_text(cx, &product.group_name);
        self.ui
            .text_input(cx, ids!(menu_name_input))
            .set_text(cx, &product.name);
        self.ui
            .text_input(cx, ids!(menu_aliases_input))
            .set_text(cx, &aliases);
        self.ui
            .text_input(cx, ids!(menu_barcode_input))
            .set_text(cx, &product.barcode);
        self.ui
            .text_input(cx, ids!(menu_hsn_input))
            .set_text(cx, &product.hsn_code);
        self.ui
            .text_input(cx, ids!(menu_composition_input))
            .set_text(cx, &product.composition);
        self.ui
            .text_input(cx, ids!(menu_price_input))
            .set_text(cx, &price);
        self.ui
            .text_input(cx, ids!(menu_mrp_input))
            .set_text(cx, &mrp);
        self.ui
            .text_input(cx, ids!(menu_unit_input))
            .set_text(cx, &product.unit_label);
        self.ui
            .text_input(cx, ids!(menu_tax_input))
            .set_text(cx, &tax);
        self.ui
            .text_input(cx, ids!(menu_category_input))
            .set_text(cx, &category);
        self.ui
            .text_input(cx, ids!(menu_modifiers_input))
            .set_text(cx, &modifiers);
        {
            let mut st = app_state().lock().unwrap();
            st.editing_product_id = Some(product.id.clone());
            st.form_resale = product.kind == domain::ITEM_KIND_RESALE;
            st.draft_image_key = product.image_key.clone();
            // The loaded item's category travels by UUID too.
            if product.category_id.is_empty() {
                st.form_category_id = None;
                st.form_category_label.clear();
            } else {
                st.form_category_id = Some(product.category_id.clone());
                st.form_category_label = category.clone();
            }
            // The item's own category is the owner's choice, never replaced
            // by a later dish pick; no shelf photo pending.
            st.form_category_auto = false;
            st.form_category_photo = None;
            st.category_picker_open = false;
        }
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    /// Put a category in the item form's Category field. `id` links one of
    /// the owner's categories by UUID; `photo` is a shelf folder's photo for
    /// a category created on save; `auto` marks it as filled by the app (a
    /// later pick may replace it, a typed one never is).
    fn set_form_category(
        &mut self,
        cx: &mut Cx,
        name: &str,
        id: Option<String>,
        photo: Option<PathBuf>,
        auto: bool,
    ) {
        {
            let mut st = app_state().lock().unwrap();
            st.form_category_id = id;
            st.form_category_label = name.to_string();
            st.form_category_photo = photo.map(|p| (name.to_string(), p));
            st.form_category_auto = auto;
            st.category_picker_open = false;
        }
        self.ui.text_input(cx, ids!(menu_category_input)).set_text(cx, name);
    }

    /// A picked dish brings its shelf folder as the category, editable:
    /// the owner's own category of that name when there is one (by UUID),
    /// else the folder's name (created on save, with the folder's photo).
    /// A category the owner typed is left alone.
    fn prefill_category_from_dish(&mut self, cx: &mut Cx, dish: &menu_images::ShelfDish) {
        let typed = self.ui.text_input(cx, ids!(menu_category_input)).text();
        let (replace, existing, photo) = {
            let st = app_state().lock().unwrap();
            let replace = typed.trim().is_empty() || st.form_category_auto;
            let existing = st
                .categories
                .iter()
                .find(|c| c.name.trim().eq_ignore_ascii_case(dish.folder.trim()))
                .map(|c| (c.id.clone(), c.name.clone()));
            let photo = menu_images::shelf_folders(menu_images::shelf())
                .into_iter()
                .find(|f| f.label == dish.folder)
                .map(|f| f.photo);
            (replace, existing, photo)
        };
        if !replace || dish.folder.trim().is_empty() {
            return;
        }
        match existing {
            Some((id, name)) => self.set_form_category(cx, &name, Some(id), None, true),
            None => self.set_form_category(cx, &dish.folder, None, photo, true),
        }
    }

    /// Copy a shelf dish into the item form: name + photo, nothing saved yet.
    /// The shelf is bundled, read-only art — picking a dish writes no item,
    /// sets no price; its shelf folder is suggested as the category
    /// (editable, see [`Self::prefill_category_from_dish`]). The owner sets
    /// the price (blank is ₹0) and taps Add item.
    fn use_shelf_dish(&mut self, cx: &mut Cx, dish: &menu_images::ShelfDish) {
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let actor = st.actor()?;
            if !actor.is_owner() && !actor.can_access_items() {
                return Err("Not allowed to manage items".into());
            }
            // A fresh item. The form's own choices (category, station, unit,
            // tax) stay as the owner left them — only name + photo come from
            // the shelf.
            st.editing_product_id = None;
            st.draft_image_key = Some(format!("pending_custom:{}", dish.path.display()));
            Ok(format!("{} — set your price, then Add item", dish.name))
        })();
        match result {
            Ok(msg) => {
                self.ui
                    .text_input(cx, ids!(menu_name_input))
                    .set_text(cx, &dish.name);
                // The price belongs to the dish being added — never inherited
                // from whatever was in the form before.
                self.ui.text_input(cx, ids!(menu_price_input)).set_text(cx, "");
                self.prefill_category_from_dish(cx, dish);
                self.ui
                    .view(cx, ids!(menu_screen))
                    .set_scroll_pos(cx, dvec2(0.0, 0.0));
                self.set_status(cx, msg);
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Keep the typed text as the owner's own item name. A photo that came
    /// from a shelf dish picked earlier belonged to that dish, so it goes;
    /// a photo the owner picked from their own files stays.
    fn use_own_item_name(&mut self, cx: &mut Cx, name: &str) {
        {
            let mut st = app_state().lock().unwrap();
            let from_shelf = st
                .draft_image_key
                .as_deref()
                .and_then(|k| k.strip_prefix("pending_custom:"))
                .zip(menu_images::catalogue_root())
                .is_some_and(|(path, root)| std::path::Path::new(path).starts_with(root));
            if from_shelf {
                st.draft_image_key = None;
            }
        }
        self.ui.text_input(cx, ids!(menu_name_input)).set_text(cx, name);
        self.set_status(cx, format!("{name} — set your price, then Add item"));
    }

    /// Switch this device's active branch (list pick or typed name).
    fn switch_active_branch(&mut self, cx: &mut Cx, wanted: &str) -> Result<String, String> {
        let _ = cx;
        let mut st = app_state().lock().unwrap();
        let actor = st.actor()?;
        if !actor.can_access_branches() && !actor.is_owner() {
            return Err("Only Owner can switch branch".into());
        }
        let needle = wanted.trim().to_lowercase();
        if needle.is_empty() {
            return Err("Type the branch name to switch to".into());
        }
        let (id, label) = {
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            let branch = store
                .list_branches()?
                .into_iter()
                .find(|b| {
                    b.name.to_lowercase() == needle
                        || b.code.to_lowercase() == needle
                        || b.name.to_lowercase().contains(&needle)
                })
                .ok_or_else(|| format!("No branch matches \"{}\"", wanted.trim()))?;
            (branch.id.clone(), branch.label())
        };
        let msg = st.books_op(BooksOp::SwitchBranch { id })?;
        st.cart.clear();
        st.open_cart_line = None;
        if st.device_role != DeviceRole::Client {
            st.refresh_branches_text()?;
        }
        Ok(if st.device_role == DeviceRole::Client {
            msg
        } else {
            format!("Now selling for {label}")
        })
    }

}

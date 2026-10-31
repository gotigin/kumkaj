impl App {
    fn handle_actions_desk_menu(&mut self, cx: &mut Cx, actions: &Actions) {
        // Desk unit chips: switch the shop's entry units without leaving
        // the desk (saved via the same settings path as Booking measures).
        for ((chip, chip_on), code) in [
            ((ids!(wunit_kg), ids!(wunit_kg_on)), "kg"),
            ((ids!(wunit_lb), ids!(wunit_lb_on)), "lb"),
            ((ids!(wunit_g), ids!(wunit_g_on)), "g"),
            ((ids!(wunit_oz), ids!(wunit_oz_on)), "oz"),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                self.apply_measure_units(cx, None, Some(code.to_string()));
            }
        }
        for ((chip, chip_on), code) in [
            ((ids!(dunit_cm), ids!(dunit_cm_on)), "cm"),
            ((ids!(dunit_in), ids!(dunit_in_on)), "in"),
            ((ids!(dunit_ft), ids!(dunit_ft_on)), "ft"),
            ((ids!(dunit_m), ids!(dunit_m_on)), "m"),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                self.apply_measure_units(cx, Some(code.to_string()), None);
            }
        }
        // Expense tender chips: first tender and the optional second way.
        for ((chip, chip_on), m) in [
            ((ids!(exp_t1_cash), ids!(exp_t1_cash_on)), PayMethod::Cash),
            ((ids!(exp_t1_bank), ids!(exp_t1_bank_on)), PayMethod::Bank),
            ((ids!(exp_t1_upi), ids!(exp_t1_upi_on)), PayMethod::Upi),
            ((ids!(exp_t1_card), ids!(exp_t1_card_on)), PayMethod::Card),
            (
                (ids!(buy_exp_t1_cash), ids!(buy_exp_t1_cash_on)),
                PayMethod::Cash,
            ),
            (
                (ids!(buy_exp_t1_bank), ids!(buy_exp_t1_bank_on)),
                PayMethod::Bank,
            ),
            (
                (ids!(buy_exp_t1_upi), ids!(buy_exp_t1_upi_on)),
                PayMethod::Upi,
            ),
            (
                (ids!(buy_exp_t1_card), ids!(buy_exp_t1_card_on)),
                PayMethod::Card,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                app_state().lock().unwrap().expense_tender = m;
                self.sync_chrome(cx);
            }
        }
        for ((chip, chip_on), m) in [
            ((ids!(exp_t2_cash), ids!(exp_t2_cash_on)), PayMethod::Cash),
            ((ids!(exp_t2_bank), ids!(exp_t2_bank_on)), PayMethod::Bank),
            ((ids!(exp_t2_upi), ids!(exp_t2_upi_on)), PayMethod::Upi),
            ((ids!(exp_t2_card), ids!(exp_t2_card_on)), PayMethod::Card),
            (
                (ids!(buy_exp_t2_cash), ids!(buy_exp_t2_cash_on)),
                PayMethod::Cash,
            ),
            (
                (ids!(buy_exp_t2_bank), ids!(buy_exp_t2_bank_on)),
                PayMethod::Bank,
            ),
            (
                (ids!(buy_exp_t2_upi), ids!(buy_exp_t2_upi_on)),
                PayMethod::Upi,
            ),
            (
                (ids!(buy_exp_t2_card), ids!(buy_exp_t2_card_on)),
                PayMethod::Card,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                app_state().lock().unwrap().expense_tender2 = m;
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(clear_cart)).clicked(actions)
            || self.ui.button(cx, ids!(next_booking_btn)).clicked(actions)
        {
            {
                let mut st = app_state().lock().unwrap();
                st.clear_booking_desk();
            }
            self.clear_booking_desk_ui(cx);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(booking_bulk_load)).clicked(actions) {
            let raw = self.ui.text_input(cx, ids!(booking_bulk_input)).text();
            match notify::parse_bulk_booking_csv(&raw) {
                Ok(rows) if rows.is_empty() => {
                    self.set_status(cx, "Bulk CSV is empty — paste destination,consignee,…");
                }
                Ok(rows) => {
                    let first = &rows[0];
                    let n = rows.len();
                    {
                        let mut st = app_state().lock().unwrap();
                        st.booking_destination = first.destination.clone();
                        st.booking_consignee = first.consignee.clone();
                        st.booking_actual_kg = if first.actual_kg > precise::ZERO {
                            precise::format_kg(first.actual_kg)
                        } else {
                            String::new()
                        };
                        st.booking_cod = if first.cod_paise > 0 {
                            locale::format_major_input(first.cod_paise, &st.locale.currency)
                        } else {
                            String::new()
                        };
                        st.status = format!(
                            "Bulk: {n} row(s). Desk loaded with row 1 — pick rate & book, then load next."
                        );
                    }
                    self.ui
                        .text_input(cx, ids!(booking_destination_input))
                        .set_text(cx, &first.destination);
                    self.ui
                        .text_input(cx, ids!(booking_consignee_input))
                        .set_text(cx, &first.consignee);
                    let (kg_text, cod_text) = {
                        let st = app_state().lock().unwrap();
                        (
                            if first.actual_kg > precise::ZERO {
                                precise::format_kg(first.actual_kg)
                            } else {
                                String::new()
                            },
                            if first.cod_paise > 0 {
                                locale::format_major_input(first.cod_paise, &st.locale.currency)
                            } else {
                                String::new()
                            },
                        )
                    };
                    self.ui
                        .text_input(cx, ids!(booking_actual_kg_input))
                        .set_text(cx, &kg_text);
                    self.ui
                        .text_input(cx, ids!(booking_cod_input))
                        .set_text(cx, &cod_text);
                    // Drop the loaded row so the next tap advances through the paste.
                    let rest: Vec<String> = rows
                        .iter()
                        .skip(1)
                        .map(|r| {
                            format!(
                                "{},{},{},{},{}",
                                r.destination, r.consignee, r.actual_kg, r.cod_paise, r.note
                            )
                        })
                        .collect();
                    self.ui
                        .text_input(cx, ids!(booking_bulk_input))
                        .set_text(cx, &rest.join("\n"));
                    self.sync_chrome(cx);
                }
                Err(e) => self.set_status(cx, e),
            }
        }

        // Live chargeable kg as the desk fields change.
        for id in [
            ids!(booking_actual_kg_input),
            ids!(booking_length_cm_input),
            ids!(booking_breadth_cm_input),
            ids!(booking_height_cm_input),
        ] {
            if self.ui.text_input(cx, id).changed(actions).is_some() {
                let mut st = app_state().lock().unwrap();
                st.booking_actual_kg = self.ui.text_input(cx, ids!(booking_actual_kg_input)).text();
                st.booking_length_cm = self.ui.text_input(cx, ids!(booking_length_cm_input)).text();
                st.booking_breadth_cm = self
                    .ui
                    .text_input(cx, ids!(booking_breadth_cm_input))
                    .text();
                st.booking_height_cm = self.ui.text_input(cx, ids!(booking_height_cm_input)).text();
                drop(st);
                self.sync_chrome(cx);
                break;
            }
        }

        // Live initials on the draft thumb while typing an item name.
        if self
            .ui
            .text_input(cx, ids!(menu_name_input))
            .changed(actions)
            .is_some()
        {
            self.sync_chrome(cx);
        }

        let total_input = self.ui.text_input(cx, ids!(cart_total_input));
        if total_input.changed(actions).is_some() {
            self.apply_cart_total_from_input_live(cx);
        }
        if total_input.returned(actions).is_some() || total_input.key_focus_lost(actions) {
            self.apply_cart_total_from_input(cx);
        }
        if self.ui.button(cx, ids!(menu_new)).clicked(actions) {
            self.start_new_menu_item(cx);
        }
        if self.ui.button(cx, ids!(menu_add)).clicked(actions) {
            self.add_menu_item(cx);
        }
        if self.ui.button(cx, ids!(image_clear)).clicked(actions) {
            self.image_clear_photo(cx);
        }
        if self.ui.button(cx, ids!(image_pick)).clicked(actions) {
            self.image_pick_file(cx);
        }
        // Sell list: 3-col token grid — each PortalList row holds c0/c1/c2.
        {
            let widget = self.ui.widget(cx, ids!(product_list));
            let list = widget.portal_list(cx, ids!(list));
            for (row_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(c0.add)).clicked(actions) {
                    self.add_product_to_cart(cx, row_id * 3);
                }
                if item.button(cx, ids!(c1.add)).clicked(actions) {
                    self.add_product_to_cart(cx, row_id * 3 + 1);
                }
                if item.button(cx, ids!(c2.add)).clicked(actions) {
                    self.add_product_to_cart(cx, row_id * 3 + 2);
                }
            }
        }
        // Items list: sold out / archive
        {
            let widget = self.ui.widget(cx, ids!(product_list_menu));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                let marked = item.button(cx, ids!(sold_out)).clicked(actions);
                let removed = item.button(cx, ids!(remove)).clicked(actions);
                if marked {
                    let result = (|| -> Result<(), String> {
                        let mut st = app_state().lock().unwrap();
                        st.require_server_role("Mark items sold out")?;
                        let product = st
                            .products
                            .get(item_id)
                            .cloned()
                            .ok_or_else(|| "Missing product".to_string())?;
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.set_product_available(&product.id, !product.available)?;
                        st.reload_all()?;
                        st.status = if product.available {
                            format!("{} marked sold out", product.name)
                        } else {
                            format!("{} restocked", product.name)
                        };
                        Ok(())
                    })();
                    if let Err(e) = result {
                        self.set_status(cx, e);
                    }
                    self.sync_chrome(cx);
                }
                if removed {
                    self.remove_menu_item(cx, item_id);
                } else if !marked
                    && item
                        .view(cx, ids!(card))
                        .finger_up(actions)
                        .is_some_and(|e| e.was_tap() && e.is_over)
                {
                    self.open_menu_item(cx, item_id);
                }
            }
        }

        for list_id in [ids!(language_list), ids!(setup_lang_list)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions)
                    && let Some(lang) = LANGUAGES.get(item_id)
                {
                    self.set_language(cx, lang.code);
                }
            }
        }
        for list_id in [ids!(currency_list)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions)
                    && let Some(cur) = CURRENCIES.get(item_id)
                {
                    self.set_currency(cx, cur.code);
                }
            }
        }

        let cart_widget = self.ui.widget(cx, ids!(cart_list));
        let cart_list = cart_widget.portal_list(cx, ids!(list));
        for (item_id, item) in cart_list.items_with_actions(actions) {
            if item.button(cx, ids!(minus)).clicked(actions) {
                self.bump_cart_line(cx, item_id, -1);
            }
            if item.button(cx, ids!(plus)).clicked(actions) {
                self.bump_cart_line(cx, item_id, 1);
            }
            let unit_input = item.text_input(cx, ids!(unit_price));
            if let Some((text, _)) = unit_input.returned(actions) {
                self.apply_cart_unit_price(cx, item_id, &text);
            } else if unit_input.key_focus_lost(actions) {
                let text = unit_input.text();
                self.apply_cart_unit_price(cx, item_id, &text);
            }
            let line_total_input = item.text_input(cx, ids!(line_total));
            if let Some((text, _)) = line_total_input.returned(actions) {
                self.apply_cart_line_total(cx, item_id, &text);
            } else if line_total_input.key_focus_lost(actions) {
                let text = line_total_input.text();
                self.apply_cart_line_total(cx, item_id, &text);
            }
            let note_input = item.text_input(cx, ids!(note));
            if let Some((text, _)) = note_input.returned(actions) {
                self.apply_cart_note(cx, item_id, &text);
            } else if note_input.key_focus_lost(actions) {
                let text = note_input.text();
                self.apply_cart_note(cx, item_id, &text);
            }
            self.handle_modifier_chips(cx, actions, &item, item_id, false);
        }

        let sale_widget = self.ui.widget(cx, ids!(sale_list));
        let sale_list = sale_widget.portal_list(cx, ids!(list));
        for (item_id, item) in sale_list.items_with_actions(actions) {
            if item.button(cx, ids!(pick)).clicked(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let sale_id = st
                        .recent_sales
                        .get(item_id)
                        .map(|s| s.id.clone())
                        .ok_or_else(|| "Missing sale".to_string())?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let sale = store
                        .get_sale(&sale_id)?
                        .ok_or_else(|| "Sale not found".to_string())?;
                    let revisions = store.list_sale_revisions(&sale_id)?;
                    let pay = PayMethod::parse(&sale.pay_method).unwrap_or(PayMethod::Cash);
                    let short = sale.id[..8.min(sale.id.len())].to_string();
                    st.amend_cart = Cart::from_sale(&sale);
                    st.amend_pay = pay;
                    st.sale_revisions = revisions;
                    st.selected_sale_id = Some(sale.id);
                    st.status = format!("Selected sale {short} — edit lines/total, then Amend");
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }

        let amend_widget = self.ui.widget(cx, ids!(amend_cart_list));
        let amend_list = amend_widget.portal_list(cx, ids!(list));
        for (item_id, item) in amend_list.items_with_actions(actions) {
            if item.button(cx, ids!(minus)).clicked(actions) {
                app_state().lock().unwrap().amend_cart.bump_qty(item_id, -1);
                self.sync_chrome(cx);
            }
            if item.button(cx, ids!(plus)).clicked(actions) {
                app_state().lock().unwrap().amend_cart.bump_qty(item_id, 1);
                self.sync_chrome(cx);
            }
            let unit_input = item.text_input(cx, ids!(unit_price));
            if let Some((text, _)) = unit_input.returned(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_unit_price(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            } else if unit_input.key_focus_lost(actions) {
                let text = unit_input.text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_unit_price(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            let line_total_input = item.text_input(cx, ids!(line_total));
            if let Some((text, _)) = line_total_input.returned(actions) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_line_total(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            } else if line_total_input.key_focus_lost(actions) {
                let text = line_total_input.text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&text, &currency)?;
                    st.amend_cart.set_line_total(item_id, Money(minor))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            let note_input = item.text_input(cx, ids!(note));
            if let Some((text, _)) = note_input.returned(actions) {
                let result = (|| -> Result<(), String> {
                    app_state()
                        .lock()
                        .unwrap()
                        .amend_cart
                        .set_note(item_id, text)?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            } else if note_input.key_focus_lost(actions) {
                let text = note_input.text();
                let result = (|| -> Result<(), String> {
                    app_state()
                        .lock()
                        .unwrap()
                        .amend_cart
                        .set_note(item_id, text)?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
            self.handle_modifier_chips(cx, actions, &item, item_id, true);
        }

        if self.choice_chip_clicked(cx, actions, ids!(amend_pay_cash), ids!(amend_pay_cash_on)) {
            app_state().lock().unwrap().amend_pay = PayMethod::Cash;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(amend_pay_upi), ids!(amend_pay_upi_on)) {
            app_state().lock().unwrap().amend_pay = PayMethod::Upi;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(amend_pay_card), ids!(amend_pay_card_on)) {
            app_state().lock().unwrap().amend_pay = PayMethod::Card;
            self.sync_chrome(cx);
        }

        let amend_total = self.ui.text_input(cx, ids!(amend_total_input));
        let amend_total_dirty = amend_total.changed(actions).is_some()
            || amend_total.returned(actions).is_some()
            || amend_total.key_focus_lost(actions);
        if amend_total_dirty {
            let text = amend_total.text();
            let report_errors =
                amend_total.returned(actions).is_some() || amend_total.key_focus_lost(actions);
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                if st.amend_cart.is_empty() {
                    return Ok(());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&text, &currency)?;
                st.amend_cart.set_charge_total(Money(minor))?;
                Ok(())
            })();
            if let Err(e) = result
                && report_errors
            {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(amend_btn)).clicked(actions) {
            let reason = self.ui.text_input(cx, ids!(reason_input)).text();
            let pin = self.ui.text_input(cx, ids!(approval_pin_input)).text();
            let typed_total = self.ui.text_input(cx, ids!(amend_total_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                let currency = st.locale.currency.clone();
                if let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                    && minor > 0
                {
                    let _ = st.amend_cart.set_charge_total(Money(minor));
                }
                let draft = st.amend_cart.to_sale_draft()?;
                let pay = st.amend_pay;
                // The Server holds the sale and its revision history; a Client's
                // copy is a cache the next sync overwrites.
                if st.device_role == DeviceRole::Client {
                    let req = net::RemoteAmendRequest {
                        sale_id: sale_id.clone(),
                        draft,
                        pay: pay.as_str().into(),
                        reason: reason.clone(),
                        approval_pin: pin.trim().to_string(),
                    };
                    st.lan_job(
                        "/amend",
                        &req,
                        "Amend".into(),
                        "Amended on the office's books · history kept".into(),
                        phone::Follow::Refresh,
                        false,
                    )?;
                    st.status = "Amending…".into();
                    return Ok(());
                }
                let (sale, revisions) = {
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let (who, reason) =
                        approve_for(store, &actor, domain::Right::AmendSale, &pin, &reason)?;
                    let sale = store.amend_sale(&sale_id, &draft, pay, &reason, &who)?;
                    let revisions = store.list_sale_revisions(&sale_id)?;
                    (sale, revisions)
                };
                st.amend_cart = Cart::from_sale(&sale);
                st.sale_revisions = revisions;
                st.reload_all()?;
                st.status = format!(
                    "Amended sale · Rounded Total {} · history kept",
                    st.money(Money(sale.total_paise))
                );
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(void_btn)).clicked(actions) {
            let reason = self.ui.text_input(cx, ids!(reason_input)).text();
            let pin = self.ui.text_input(cx, ids!(approval_pin_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                // A device sends the manager's PIN as typed; only the Server
                // checks it (a device's word is never taken for it).
                if st.device_role == DeviceRole::Client {
                    st.lan_job(
                        "/void",
                        &net::RemoteSaleAction {
                            sale_id: sale_id.clone(),
                            reason: reason.trim().to_string(),
                            approval_pin: pin.trim().to_string(),
                        },
                        "Void".into(),
                        "Booking voided on the office's books".into(),
                        phone::Follow::Refresh,
                        false,
                    )?;
                    st.status = "Voiding…".into();
                    return Ok(());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let (approved_actor, reason) =
                    approve_for(store, &actor, domain::Right::VoidRefund, &pin, &reason)?;
                let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                store.void_sale(&sale_id, &reason, &approved_actor)?;
                st.reload_all()?;
                st.status = if approved_actor.id == actor.id {
                    "Sale voided".into()
                } else {
                    format!("Sale voided · approved by {}", approved_actor.name)
                };
                Ok(())
            })();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(()) => self
                    .ui
                    .text_input(cx, ids!(approval_pin_input))
                    .set_text(cx, ""),
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(refund_btn)).clicked(actions) {
            let reason = self.ui.text_input(cx, ids!(reason_input)).text();
            let pin = self.ui.text_input(cx, ids!(approval_pin_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                // A device sends the manager's PIN as typed; only the Server
                // checks it (a device's word is never taken for it).
                if st.device_role == DeviceRole::Client {
                    st.lan_job(
                        "/refund",
                        &net::RemoteSaleAction {
                            sale_id: sale_id.clone(),
                            reason: reason.trim().to_string(),
                            approval_pin: pin.trim().to_string(),
                        },
                        "Refund".into(),
                        "Booking refunded on the office's books".into(),
                        phone::Follow::Refresh,
                        false,
                    )?;
                    st.status = "Refunding…".into();
                    return Ok(());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let (approved_actor, reason) =
                    approve_for(store, &actor, domain::Right::VoidRefund, &pin, &reason)?;
                let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                store.refund_sale(&sale_id, &reason, &approved_actor)?;
                st.reload_all()?;
                st.status = if approved_actor.id == actor.id {
                    "Sale refunded".into()
                } else {
                    format!("Sale refunded · approved by {}", approved_actor.name)
                };
                Ok(())
            })();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(()) => self
                    .ui
                    .text_input(cx, ids!(approval_pin_input))
                    .set_text(cx, ""),
            }
            self.sync_chrome(cx);
        }

        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(staff_role_cashier),
            ids!(staff_role_cashier_on),
        ) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Cashier;
                st.draft_features = StaffFeatures::for_role(StaffRole::Cashier);
            }
            self.set_status(cx, "Preset: Cashier");
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(staff_role_manager),
            ids!(staff_role_manager_on),
        ) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Manager;
                st.draft_features = StaffFeatures::for_role(StaffRole::Manager);
            }
            self.set_status(cx, "Preset: Manager");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(staff_add_cashier)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(staff_name_input)).text();
            let title = self.ui.text_input(cx, ids!(staff_title_input)).text();
            let pin = self.ui.text_input(cx, ids!(staff_pin_input)).text();
            let login = self.ui.text_input(cx, ids!(staff_login_input)).text();
            let password = self.ui.text_input(cx, ids!(staff_password_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add staff")?;
                if !st.owner_setup_complete {
                    return Err("Finish Owner PIN setup before adding staff".into());
                }
                let actor = st.actor()?;
                let role = st.draft_staff_role;
                let features = st.draft_features.clone();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_staff_with_features(
                    &store::StaffHire {
                        name: &name,
                        role,
                        role_title: &title,
                        pin: &pin,
                        login_id: &login,
                        password: &password,
                        features,
                    },
                    &actor,
                )?;
                st.reload_all()?;
                st.team_staff_id = st
                    .staff
                    .iter()
                    .rev()
                    .find(|s| !s.is_owner())
                    .map(|s| s.id.clone());
                st.status = format!("Added {}", name.trim());
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(staff_name_input))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(staff_title_input))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(staff_pin_input))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(staff_login_input))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(staff_password_input))
                    .set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(staff_set_login)).clicked(actions) {
            let login = self.ui.text_input(cx, ids!(staff_login_input)).text();
            let password = self.ui.text_input(cx, ids!(staff_password_input)).text();
            let result = (|| -> Result<(), String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Change staff logins")?;
                let actor = st.actor()?;
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| "Add or pick a staff member first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_staff_login(&staff_id, &login, &password, &actor)?;
                Ok(())
            })();
            match result {
                Ok(()) => self.set_status(cx, "Login saved"),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(staff_deactivate)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Remove staff")?;
                let actor = st.actor()?;
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| "Select a staff member first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.deactivate_staff(&staff_id, &actor)?;
                st.team_staff_id = None;
                st.reload_all()?;
                st.status = "Staff removed".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(change_my_pin)).clicked(actions) {
            let pin = self.ui.text_input(cx, ids!(my_pin_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change your PIN")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_staff_pin(&actor.id, &pin, &actor)?;
                st.status = "PIN updated".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(my_pin_input)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        if self
            .ui
            .button(cx, ids!(change_my_password))
            .clicked(actions)
        {
            let password = self.ui.text_input(cx, ids!(my_password_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change your password")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_own_password(&password, &actor)?;
                st.status = "Password updated".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(my_password_input))
                    .set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(save_duress_pin)).clicked(actions) {
            let pin = self.ui.text_input(cx, ids!(duress_pin_input)).text();
            let confirm = self.ui.text_input(cx, ids!(duress_pin_confirm)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Set emergency unlock PIN")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_duress_pin(&pin, &confirm, &actor)?;
                st.status = st.t("duress_saved");
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(duress_pin_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(duress_pin_confirm)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(clear_duress_pin)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Clear emergency unlock PIN")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.clear_duress_pin(&actor)?;
                st.status = st.t("duress_cleared");
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        for (id, on, toggle) in [
            (ids!(feat_sell), ids!(feat_sell_on), "sell"),
            (ids!(feat_items), ids!(feat_items_on), "items"),
            (ids!(feat_sales), ids!(feat_sales_on), "sales"),
            (ids!(feat_credit), ids!(feat_credit_on), "credit"),
            (ids!(feat_team), ids!(feat_team_on), "team"),
            (ids!(feat_backup), ids!(feat_backup_on), "backup"),
            (ids!(feat_inventory), ids!(feat_inventory_on), "inventory"),
            (ids!(feat_branches), ids!(feat_branches_on), "branches"),
        ] {
            if self.choice_chip_clicked(cx, actions, id, on) {
                let mut st = app_state().lock().unwrap();
                match toggle {
                    "sell" => st.draft_features.sell = !st.draft_features.sell,
                    "items" => st.draft_features.items = !st.draft_features.items,
                    "sales" => st.draft_features.sales = !st.draft_features.sales,
                    "credit" => st.draft_features.credit = !st.draft_features.credit,
                    "team" => st.draft_features.team = !st.draft_features.team,
                    "backup" => st.draft_features.backup = !st.draft_features.backup,
                    "inventory" => st.draft_features.inventory = !st.draft_features.inventory,
                    "branches" => st.draft_features.branches = !st.draft_features.branches,
                    _ => {}
                }
                st.status = "Access changed — Save features for selected staff".into();
                drop(st);
                self.sync_chrome(cx);
            }
        }
        let team_pick_now =
            std::mem::take(&mut app_state().lock().unwrap().team_select_pending);
        if team_pick_now
            || self.ui.button(cx, ids!(team_pick_btn)).clicked(actions)
            || self.ui.text_input(cx, ids!(team_pick_input)).returned(actions).is_some()
        {
            let name = self.ui.text_input(cx, ids!(team_pick_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let needle = name.trim().to_lowercase();
                if needle.is_empty() {
                    return Err("Type the staff member's name".into());
                }
                let member = st
                    .staff
                    .iter()
                    .find(|s| !s.is_owner() && s.active && s.name.trim().to_lowercase() == needle)
                    .or_else(|| {
                        st.staff.iter().find(|s| {
                            !s.is_owner() && s.active && s.name.to_lowercase().contains(&needle)
                        })
                    })
                    .cloned()
                    .ok_or_else(|| format!("No staff named \"{}\"", name.trim()))?;
                st.team_staff_id = Some(member.id.clone());
                st.draft_staff_role = member.role;
                st.draft_features = member.features.clone();
                Ok(format!("Editing {} ({})", member.name, member.display_role()))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for (button, right) in [
            (ids!(right_void_btn), domain::Right::VoidRefund),
            (ids!(right_amend_btn), domain::Right::AmendSale),
        ] {
            if self.ui.button(cx, button).clicked(actions) {
                {
                    let mut st = app_state().lock().unwrap();
                    let mut rights = st.team_draft_rights();
                    rights.set_level(right, rights.level(right).cycle());
                    st.draft_features.rights = Some(rights);
                    st.status = "Rights changed — Save features for selected staff".into();
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(feat_save)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change staff permissions")?;
                let actor = st.actor()?;
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .or_else(|| {
                        st.staff
                            .iter()
                            .find(|s| !s.is_owner())
                            .map(|s| s.id.clone())
                    })
                    .ok_or_else(|| "Add staff first".to_string())?;
                let features = st.draft_features.clone();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_staff_features(&staff_id, &features, &actor)?;
                st.reload_all()?;
                st.status = "Staff features saved".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        for (id, step) in [(ids!(country_prev), -1i64), (ids!(country_next), 1i64)] {
            if self.ui.button(cx, id).clicked(actions) {
                let typed = self.ui.text_input(cx, ids!(country_code_input)).text();
                let next = {
                    let mut st = app_state().lock().unwrap();
                    let from = if typed.trim().is_empty() {
                        st.fiscal.country_code.clone()
                    } else {
                        typed
                    };
                    let next = step_country(&from, step, &st.fiscal.country_code);
                    st.country_browse = next.code.to_string();
                    st.status = format!("{} ({})", next.name_en, next.code);
                    next
                };
                self.ui
                    .text_input(cx, ids!(country_code_input))
                    .set_text(cx, next.code);
                self.sync_chrome(cx);
            }
        }
    }
}

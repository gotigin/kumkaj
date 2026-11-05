impl App {
    fn handle_actions_parties(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(undo_snack_btn)).clicked(actions) {
            self.undo_last_sale(cx);
        }
        if self
            .ui
            .button(cx, ids!(undo_snack_dismiss))
            .clicked(actions)
        {
            self.clear_undo_snackbar(cx);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(customer_use)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(customer_name_input)).text();
            let result = (|| -> Result<Option<domain::Customer>, String> {
                let mut st = app_state().lock().unwrap();
                let needle = name.trim();
                if needle.is_empty() {
                    return Err("Customer name required".into());
                }
                if let Some(customer) = st
                    .customers
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(needle))
                    .cloned()
                {
                    st.selected_customer_id = Some(customer.id.clone());
                    st.status = format!("Consignor: {}", customer.name);
                    return Ok(Some(customer));
                }
                // Create via books (Server local apply or Client LAN).
                st.books_op(BooksOp::AddParty {
                    name: needle.to_string(),
                    phone: String::new(),
                    address: String::new(),
                    kind: String::new(),
                    image_bytes: Vec::new(),
                    image_filename: String::new(),
                    image_key: None,
                })?;
                if let Some(customer) = st
                    .customers
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(needle))
                    .cloned()
                {
                    st.selected_customer_id = Some(customer.id.clone());
                    st.status = format!("Consignor: {}", customer.name);
                    Ok(Some(customer))
                } else {
                    // Client: create is on the shop; list refreshes after Sync.
                    st.status = format!("Added {needle}");
                    Ok(None)
                }
            })();
            match result {
                Ok(Some(customer)) => self.fill_booking_consignor_from_party(cx, &customer),
                Err(e) => self.set_status(cx, e),
                Ok(None) => {}
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(customer_clear)).clicked(actions) {
            app_state().lock().unwrap().selected_customer_id = None;
            self.sync_chrome(cx);
        }
        for ((chip, chip_on), kind) in [
            (
                (ids!(credit_kind_consignor), ids!(credit_kind_consignor_on)),
                "Consignor",
            ),
            (
                (ids!(credit_kind_carrier), ids!(credit_kind_carrier_on)),
                "Carrier",
            ),
            ((ids!(credit_kind_both), ids!(credit_kind_both_on)), "Both"),
            (
                (ids!(credit_kind_supplier), ids!(credit_kind_supplier_on)),
                "Supplier",
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                self.ui
                    .text_input(cx, ids!(credit_add_kind))
                    .set_text(cx, kind);
                let lang = app_state().lock().unwrap().locale.language.clone();
                self.set_status(
                    cx,
                    match kind {
                        "Consignor" => locale::t(&lang, "role_help_consignor"),
                        "Carrier" => locale::t(&lang, "role_help_carrier"),
                        "Both" => locale::t(&lang, "role_help_both"),
                        _ => locale::t(&lang, "role_help_supplier"),
                    },
                );
                // Repaint idle/active chip faces from the field we just set.
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(credit_add_btn)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(credit_add_name)).text();
            let phone_raw = self.ui.text_input(cx, ids!(credit_add_phone)).text();
            let street = self.ui.text_input(cx, ids!(credit_add_address)).text();
            let state = self.ui.text_input(cx, ids!(party_state_input)).text();
            let pin = self.ui.text_input(cx, ids!(party_pincode_input)).text();
            let country = self.ui.text_input(cx, ids!(party_country_input)).text();
            let address = compose_party_address(&street, &state, &pin, &country);
            let kind = self.ui.text_input(cx, ids!(credit_add_kind)).text();
            let result = (|| -> Result<(String, bool), String> {
                let mut st = app_state().lock().unwrap();
                let dial_country = if country.trim().is_empty() {
                    st.fiscal.country_code.clone()
                } else {
                    country_code_from_label(&country)
                };
                let phone = with_calling_code(&phone_raw, &dial_country);
                let pending = st.draft_party_image_key.clone();
                let (image_bytes, image_filename, image_key) = party_image_parts(&pending)?;
                if let Some(id) = st.selected_customer_id.clone() {
                    let msg = st.books_op(BooksOp::UpdateParty {
                        id,
                        name: name.clone(),
                        phone,
                        address: address.clone(),
                        kind: kind.clone(),
                        image_bytes,
                        image_filename,
                        image_key,
                    })?;
                    st.draft_party_image_key = None;
                    Ok((msg, false))
                } else {
                    let msg = st.books_op(BooksOp::AddParty {
                        name: name.clone(),
                        phone,
                        address: address.clone(),
                        kind: kind.clone(),
                        image_bytes,
                        image_filename,
                        image_key,
                    })?;
                    // Stay ready for the next party — do NOT leave this one
                    // selected. Otherwise the next "Add party" becomes
                    // "Save changes" and overwrites the same row (owner
                    // thought they added 12 and only saw one name).
                    st.selected_customer_id = None;
                    st.draft_party_image_key = None;
                    let status = if st.device_role == DeviceRole::Client {
                        msg
                    } else {
                        format!(
                            "Added {} — {} saved. Form cleared for the next party.",
                            name.trim(),
                            st.customers.len()
                        )
                    };
                    Ok((status, true))
                }
            })();
            match result {
                Ok((msg, clear_form)) => {
                    if clear_form {
                        {
                            let mut st = app_state().lock().unwrap();
                            st.draft_party_image_key = None;
                            st.party_form_country.clear();
                        }
                        self.ui
                            .text_input(cx, ids!(credit_add_name))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(credit_add_phone))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(credit_add_address))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(party_country_input))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(party_state_input))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(party_pincode_input))
                            .set_text(cx, "");
                        // Keep role chip (Consignor/Carrier/…) for the next add.
                        self.ui
                            .text_input(cx, ids!(party_custom_key))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(party_custom_value))
                            .set_text(cx, "");
                    }
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_form_clear)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.selected_customer_id = None;
                st.draft_party_image_key = None;
                st.party_form_country.clear();
            }
            self.ui
                .text_input(cx, ids!(credit_add_name))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(credit_add_phone))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(credit_add_address))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(party_country_input))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(party_state_input))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(party_pincode_input))
                .set_text(cx, "");
            // Role chip stays — batch-add carriers/consignors without re-tapping.
            self.ui
                .text_input(cx, ids!(party_custom_key))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(party_custom_value))
                .set_text(cx, "");
            self.set_status(cx, "New party — form cleared");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_image_clear)).clicked(actions) {
            self.party_image_clear_logo(cx);
        }
        if self.ui.button(cx, ids!(party_image_pick)).clicked(actions) {
            self.party_image_pick_file(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(booking_scope_domestic),
            ids!(booking_scope_domestic_on),
        ) {
            app_state().lock().unwrap().booking_scope_international = false;
            self.set_status(cx, "Domestic — volumetric ÷ shop divisor (usually 5000)");
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(booking_scope_intl),
            ids!(booking_scope_intl_on),
        ) {
            app_state().lock().unwrap().booking_scope_international = true;
            self.set_status(cx, "International — volumetric ÷6000 (air)");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_delete)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let msg = st.books_op(BooksOp::DeleteParty { id })?;
                st.selected_customer_id = None;
                Ok(msg)
            })();
            match result {
                Ok(msg) => {
                    app_state().lock().unwrap().party_form_country.clear();
                    self.ui
                        .text_input(cx, ids!(credit_add_name))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(credit_add_phone))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(credit_add_address))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(party_country_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(party_state_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(party_pincode_input))
                        .set_text(cx, "");
                    // Keep role chip for the next add.
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_custom_add)).clicked(actions) {
            let key = self.ui.text_input(cx, ids!(party_custom_key)).text();
            let value = self.ui.text_input(cx, ids!(party_custom_value)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                st.books_op(BooksOp::SetPartyField {
                    id,
                    key: key.clone(),
                    value,
                })
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(party_custom_key))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(party_custom_value))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_custom_remove)).clicked(actions) {
            let key = self.ui.text_input(cx, ids!(party_custom_key)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                st.books_op(BooksOp::RemovePartyField { id, key })
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(party_custom_key))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(party_custom_value))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(credit_limit_set)).clicked(actions) {
            let limit_text = self.ui.text_input(cx, ids!(credit_limit_input)).text();
            let named = self.ui.text_input(cx, ids!(credit_limit_party)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let currency = st.locale.currency.clone();
                let limit_paise = if limit_text.trim().is_empty() {
                    0
                } else {
                    locale::parse_major_input(&limit_text, &currency)?
                };
                let party_name = if !named.trim().is_empty() {
                    named
                } else {
                    let id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a party or type a name".to_string())?;
                    st.customers
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.name.clone())
                        .ok_or_else(|| "Party not found".to_string())?
                };
                st.books_op(BooksOp::SetCreditLimit {
                    party_name,
                    limit_paise,
                })
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(credit_limit_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self
            .ui
            .button(cx, ids!(credit_period_bill))
            .clicked(actions)
        {
            let named = self.ui.text_input(cx, ids!(credit_limit_party)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let party_name = if !named.trim().is_empty() {
                    named
                } else {
                    let id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a party or type a name".to_string())?;
                    st.customers
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.name.clone())
                        .ok_or_else(|| "Party not found".to_string())?
                };
                st.books_op(BooksOp::PeriodBill { party_name })
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(credit_writeoff)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(writeoff_amount_input)).text();
            let reason = self.ui.text_input(cx, ids!(writeoff_reason_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let customer_id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let currency = st.locale.currency.clone();
                let amount = match amount_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let msg = st.books_op(BooksOp::WriteOffCredit {
                    customer_id,
                    amount_paise: amount,
                    reason,
                })?;
                let _ = st.refresh_books_overview();
                Ok(msg)
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(writeoff_amount_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(writeoff_reason_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for (btn, method) in [
            (ids!(payable_cash), PayMethod::Cash),
            (ids!(payable_upi), PayMethod::Upi),
            (ids!(payable_card), PayMethod::Card),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(payable_amount_input)).text();
                let bill_text = self.ui.text_input(cx, ids!(payable_bill_input)).text();
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    let party_id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a party first".to_string())?;
                    let currency = st.locale.currency.clone();
                    let amount = locale::parse_major_input(&amount_text, &currency)?;
                    let bill_number = match bill_text.trim() {
                        "" => None,
                        other => Some(
                            other
                                .parse()
                                .map_err(|_| "Bill # must be a number from the list".to_string())?,
                        ),
                    };
                    let msg = st.books_op(BooksOp::SettlePayable {
                        party_id,
                        amount_paise: amount,
                        pay: method,
                        bill_number,
                    })?;
                    let _ = st.refresh_books_overview();
                    Ok(msg)
                })();
                match result {
                    Ok(msg) => {
                        self.ui
                            .text_input(cx, ids!(payable_amount_input))
                            .set_text(cx, "");
                        self.ui
                            .text_input(cx, ids!(payable_bill_input))
                            .set_text(cx, "");
                        self.set_status(cx, msg);
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(payable_writeoff)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(writeoff_amount_input)).text();
            let reason = self.ui.text_input(cx, ids!(writeoff_reason_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let party_id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let currency = st.locale.currency.clone();
                let amount = match amount_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let msg = st.books_op(BooksOp::WriteOffPayable {
                    party_id,
                    amount_paise: amount,
                    reason,
                })?;
                let _ = st.refresh_books_overview();
                Ok(msg)
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }

        for list_id in [ids!(customer_list_sell), ids!(customer_list_credit)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let picked = {
                        let mut st = app_state().lock().unwrap();
                        let picked = st.customers.get(item_id).cloned();
                        if let Some(c) = &picked {
                            st.selected_customer_id = Some(c.id.clone());
                            st.status = format!("Consignor: {}", c.name);
                        }
                        picked
                    };
                    if let Some(c) = picked {
                        let (street, state, pin, country) = split_party_address(&c.address);
                        self.ui
                            .text_input(cx, ids!(credit_add_name))
                            .set_text(cx, &c.name);
                        self.ui
                            .text_input(cx, ids!(credit_add_phone))
                            .set_text(cx, &c.phone);
                        self.ui
                            .text_input(cx, ids!(credit_add_address))
                            .set_text(cx, &street);
                        self.ui
                            .text_input(cx, ids!(party_state_input))
                            .set_text(cx, &state);
                        self.ui
                            .text_input(cx, ids!(party_pincode_input))
                            .set_text(cx, &pin);
                        self.ui
                            .text_input(cx, ids!(party_country_input))
                            .set_text(cx, &country);
                        self.ui
                            .text_input(cx, ids!(credit_add_kind))
                            .set_text(cx, &c.kind_label);
                        {
                            let mut st = app_state().lock().unwrap();
                            st.draft_party_image_key = if c.image_key.trim().is_empty() {
                                None
                            } else {
                                Some(c.image_key.clone())
                            };
                            st.party_form_country = country;
                        }
                        // Same party is the booking consignor / sender.
                        self.fill_booking_consignor_from_party(cx, &c);
                    }
                    self.sync_chrome(cx);
                }
            }
        }

        for (btn, method) in [
            (ids!(settle_cash), PayMethod::Cash),
            (ids!(settle_upi), PayMethod::Upi),
            (ids!(settle_card), PayMethod::Card),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(settle_amount_input)).text();
                let bill_text = self.ui.text_input(cx, ids!(settle_bill_input)).text();
                let tds_text = self.ui.text_input(cx, ids!(settle_tds_input)).text();
                let disc_text = self.ui.text_input(cx, ids!(settle_discount_input)).text();
                let cheque_text = self.ui.text_input(cx, ids!(settle_cheque_input)).text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    let customer_id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a customer first".to_string())?;
                    let currency = st.locale.currency.clone();
                    let received = locale::parse_major_input(&amount_text, &currency).unwrap_or(0);
                    let tds = locale::parse_major_input(&tds_text, &currency).unwrap_or(0);
                    let discount = locale::parse_major_input(&disc_text, &currency).unwrap_or(0);
                    let minor = store::Store::settle_clear_paise(received, tds, discount);
                    if minor <= 0 {
                        return Err("Enter amount received and/or TDS / discount".into());
                    }
                    let detail = store::Store::format_settle_detail(
                        method,
                        "",
                        tds,
                        discount,
                        &cheque_text,
                    );
                    if st.device_role == DeviceRole::Client {
                        let cust_name = st
                            .customers
                            .iter()
                            .find(|c| c.id == customer_id)
                            .map(|c| c.name.clone());
                        let req = net::RemoteSettleRequest {
                            customer_id: Some(customer_id.clone()),
                            customer_name: cust_name.clone(),
                            amount_paise: minor,
                            pay: method.as_str().into(),
                            detail: detail.clone(),
                        };
                        let amount = st.money(Money(minor));
                        let who = cust_name.unwrap_or_default();
                        st.lan_job(
                            "/settle",
                            &req,
                            format!("{who} paid {amount}"),
                            format!("{who}: settled {amount} via {}", method.label()),
                            phone::Follow::Refresh,
                            true,
                        )?;
                        st.status = format!("{who}: {amount} saved — sending to the office");
                        return Ok(());
                    }
                    let bal_before = st
                        .customers
                        .iter()
                        .find(|c| c.id == customer_id)
                        .map(|c| c.balance_paise)
                        .unwrap_or(0);
                    // Optional bill-wise allocation: a bill # from the
                    // open-bills list under the party.
                    let against = match bill_text.trim() {
                        "" => None,
                        other => {
                            let n: usize = other
                                .parse()
                                .map_err(|_| "Bill # must be a number from the list".to_string())?;
                            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            let bills = store.open_credit_bills(&customer_id)?;
                            Some(
                                bills
                                    .get(n.wrapping_sub(1))
                                    .map(|b| b.bill_id.clone())
                                    .ok_or_else(|| "No such open bill".to_string())?,
                            )
                        }
                    };
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    store.settle_credit_against(
                        &customer_id,
                        minor,
                        method,
                        &actor,
                        &detail,
                        against.as_deref(),
                    )?;
                    st.reload_all()?;
                    let bal_after = st
                        .customers
                        .iter()
                        .find(|c| c.id == customer_id)
                        .map(|c| c.balance_paise)
                        .unwrap_or(0);
                    st.status = if bal_before <= 0 {
                        format!(
                            "Advance {} via {} · held {}",
                            st.money(Money(minor)),
                            method.label(),
                            st.money(Money((-bal_after).max(0)))
                        )
                    } else if bal_after < 0 {
                        format!(
                            "Paid {} via {} · debt cleared · advance {}",
                            st.money(Money(minor)),
                            method.label(),
                            st.money(Money(-bal_after))
                        )
                    } else if bal_after == 0 {
                        format!("Settled {} via {}", st.money(Money(minor)), method.label())
                    } else {
                        format!(
                            "Paid {} via {} · still owes {}",
                            st.money(Money(minor)),
                            method.label(),
                            st.money(Money(bal_after))
                        )
                    };
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.ui
                        .text_input(cx, ids!(settle_amount_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(settle_bill_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(settle_tds_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(settle_discount_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(settle_cheque_input))
                        .set_text(cx, "");
                    // Surface settle/advance result on Sell receipt line too.
                    let msg = app_state().lock().unwrap().status.clone();
                    self.set_status(cx, msg);
                }
                self.sync_chrome(cx);
            }
        }

    }
}

/// Logo picked on this device: file bytes travel with the party write.
/// `None` leaves the stored logo. `Some("")` clears it.
fn party_image_parts(
    pending: &Option<String>,
) -> Result<(Vec<u8>, String, Option<String>), String> {
    match pending.as_deref() {
        Some(key) if key.starts_with("pending_custom:") => {
            let src = PathBuf::from(&key["pending_custom:".len()..]);
            let bytes = std::fs::read(&src).map_err(|e| format!("Could not read the logo: {e}"))?;
            let filename = src
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("logo.webp")
                .to_string();
            Ok((bytes, filename, None))
        }
        other => Ok((Vec::new(), String::new(), other.map(|s| s.to_string()))),
    }
}

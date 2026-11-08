impl App {
    fn handle_actions_settings_client(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(country_apply)).clicked(actions) {
            let code = self.ui.text_input(cx, ids!(country_code_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Only the Owner can change the country pack".into());
                }
                let msg = st.books_op(BooksOp::ApplyCountry {
                    code: code.trim().to_string(),
                })?;
                st.country_browse.clear();
                st.status = msg.clone();
                Ok(msg)
            })();
            match result {
                Ok(_) => self
                    .ui
                    .text_input(cx, ids!(country_code_input))
                    .set_text(cx, ""),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for (idle, active, pricing) in [
            (
                ids!(tax_inclusive_btn),
                ids!(tax_inclusive_btn_on),
                TaxPricing::InsidePrice,
            ),
            (
                ids!(tax_exclusive_btn),
                ids!(tax_exclusive_btn_on),
                TaxPricing::AddedAtTill,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, active) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() {
                        return Err("Only the Owner can change tax pricing".into());
                    }
                    let _msg = st.books_op(BooksOp::SetTaxPricing { pricing })?;
                    // Client LAN queues only — keep fiscal (and the chips) in
                    // step until the next snapshot, same as measure units.
                    if st.device_role == DeviceRole::Client {
                        st.fiscal.tax_pricing = pricing;
                    }
                    st.status = st.fiscal.tax_pricing.label_for(st.fiscal.tax_label());
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }
        for ((chip, chip_on), date_order, time_format) in [
            ((ids!(date_order_dmy), ids!(date_order_dmy_on)), "dmy", None),
            ((ids!(date_order_mdy), ids!(date_order_mdy_on)), "mdy", None),
            ((ids!(date_order_ymd), ids!(date_order_ymd_on)), "ymd", None),
            ((ids!(time_format_24h), ids!(time_format_24h_on)), "", Some("24h")),
            ((ids!(time_format_12h), ids!(time_format_12h_on)), "", Some("12h")),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    let date = if date_order.is_empty() {
                        st.fiscal.date_order.clone()
                    } else {
                        date_order.to_string()
                    };
                    let time = time_format
                        .map(str::to_string)
                        .unwrap_or_else(|| st.fiscal.time_format.clone());
                    let msg = st.books_op(BooksOp::SetDateTimeAppearance {
                        date_order: date.clone(),
                        time_format: time.clone(),
                    })?;
                    if st.device_role == DeviceRole::Client {
                        st.fiscal.date_order = fiscal::normalize_date_order(&date).to_string();
                        st.fiscal.time_format = fiscal::normalize_time_format(&time).to_string();
                    }
                    Ok(msg)
                })();
                match result {
                    Ok(msg) => self.set_status(cx, msg),
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(fiscal_save)).clicked(actions) {
            let id_text = self.ui.text_input(cx, ids!(business_tax_id_input)).text();
            let service_text = self.ui.text_input(cx, ids!(service_bps_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Only the Owner can set the business id".into());
                }
                let service_bps = locale::parse_percent_bps(&service_text)?;
                let msg = st.books_op(BooksOp::SetFiscal {
                    business_tax_id: id_text,
                    service_bps,
                })?;
                st.status = msg;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self
            .ui
            .button(cx, ids!(onboard_save_venue))
            .clicked(actions)
        {
            let title = self.ui.text_input(cx, ids!(onboard_venue_input)).text();
            let org = self.ui.text_input(cx, ids!(onboard_org_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_venue_title(&title)?;
                let org = org.trim();
                if !org.is_empty() {
                    store.set_org_name(org)?;
                }
                st.venue_title = title.trim().to_string();
                st.status = if org.is_empty() {
                    "Shop name saved".into()
                } else {
                    "Shop and organization saved".into()
                };
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(onboard_fresh), ids!(onboard_fresh_on)) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.onboard_step = 1;
                st.status =
                    "Fresh selected — empty catalogue; tap Done — open books when ready".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(onboard_book), ids!(onboard_book_on)) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.onboard_step = 2;
                st.status =
                    "Book selected — add opening rates / credit below, then Done — open books"
                        .into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(onboard_add_item)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(onboard_item_name)).text();
            let price = self.ui.text_input(cx, ids!(onboard_item_price)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&price, &currency).unwrap_or(0);
                if name.trim().is_empty() || minor <= 0 {
                    return Err("Item name and price required".into());
                }
                let p = Product {
                    id: new_id(),
                    name: name.trim().into(),
                    price: Money(minor),
                    available: true,
                    archived: false,
                    image_key: None,
                    ..Product::default()
                };
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .upsert_product(&p)?;
                st.reload_all()?;
                st.status = format!("Added {}", p.name);
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(onboard_item_name))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(onboard_item_price))
                    .set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self
            .ui
            .button(cx, ids!(onboard_add_credit))
            .clicked(actions)
        {
            let name = self.ui.text_input(cx, ids!(onboard_cust_name)).text();
            let bal = self.ui.text_input(cx, ids!(onboard_cust_bal)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&bal, &currency)?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let customer = store.find_or_create_customer(name.trim())?;
                store.seed_opening_credit(&customer.id, minor, &actor)?;
                st.reload_all()?;
                st.status = format!("Opening credit for {}", customer.name);
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(onboard_cust_name))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(onboard_cust_bal))
                    .set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(onboard_done)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .complete_onboarding()?;
                st.onboarding_complete = true;
                // Main screen is a root — drop DeviceSetup/Onboard from Back.
                st.nav_back.clear();
                st.nav_forward.clear();
                st.screen = unlocked_home_from(&mut st);
                st.status = "Shop ready — your books are open".into();
                let _ = st.reload_all();
                let _ = st.refresh_books_overview();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(client_find_btn)).clicked(actions) {
            let looking = app_state().lock().unwrap().t("finding_server");
            self.ui
                .widget(cx, ids!(client_found_label))
                .set_text(cx, &plain_display_text(&looking));
            self.ui.widget(cx, ids!(client_found_label)).set_visible(cx, true);
            app_state().lock().unwrap().found_servers.clear();
            self.ui.widget(cx, ids!(client_found_list)).redraw(cx);
            self.paint_submit_busy_chrome(cx);
            client_find_job();
            self.paint_submit_busy_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_find_cancel)).clicked(actions) {
            client_find_cancel_flag().store(true, std::sync::atomic::Ordering::SeqCst);
            self.paint_submit_busy_chrome(cx);
        }
        // Enter on host/code advances via form Tab chain; last field / button pairs.
        let skip_pair_submit = app_state().lock().unwrap().form_tab_skip_submit;
        let pair_enter = !skip_pair_submit
            && (self
                .ui
                .text_input(cx, ids!(client_name_input))
                .returned(actions)
                .is_some()
                || self
                    .ui
                    .text_input(cx, ids!(client_code_input))
                    .returned(actions)
                    .is_some()
                || self
                    .ui
                    .text_input(cx, ids!(client_host_input))
                    .returned(actions)
                    .is_some());
        if pair_enter || self.ui.button(cx, ids!(client_pair_btn)).clicked(actions) {
            let host = self.ui.text_input(cx, ids!(client_host_input)).text();
            let code = self.ui.text_input(cx, ids!(client_code_input)).text();
            // Three counter devices must not all be "Android device" in the office's list.
            let name = Some(self.ui.text_input(cx, ids!(client_name_input)).text())
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(device_name);
            let (expect_server_id, device_id) = {
                let st = app_state().lock().unwrap();
                let host_trim = host.trim().to_string();
                let expect = st
                    .found_servers
                    .iter()
                    .find(|f| f.host == host_trim)
                    .map(|f| f.server_id.clone())
                    .filter(|id| !id.trim().is_empty());
                let device_id = st
                    .store
                    .as_ref()
                    .and_then(|s| s.lan_client_id().ok())
                    .unwrap_or_default();
                (expect, device_id)
            };
            if host.trim().is_empty() || code.trim().is_empty() {
                self.set_status(cx, "Type the office device's address and its pairing code");
            } else {
                app_state().lock().unwrap().pair_note = "Pairing…".into();
                self.set_status(cx, "Pairing…");
                self.ui
                    .widget(cx, ids!(client_pair_status))
                    .set_text(cx, "Pairing…");
                self.ui.redraw(cx);
                let _ = std::thread::Builder::new()
                    .name("courier-pair".into())
                    .spawn(move || {
                        let reply = net::client_pair(
                            &host,
                            &code,
                            name.trim(),
                            &device_id,
                            expect_server_id.as_deref(),
                        );
                        *lan_once().lock().unwrap() = Some(LanOnce::Paired(reply));
                        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                    });
            }
        }
        if self.ui.button(cx, ids!(client_sync_btn)).clicked(actions) {
            phone::want_sync();
            self.set_status(cx, "Refreshing from the office…");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_connection_btn)).clicked(actions)
            || self.ui.button(cx, ids!(lock_client_connection)).clicked(actions)
        {
            self.go(cx, Screen::ClientPair);
        }
        {
            let widget = self.ui.widget(cx, ids!(client_found_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if !item.button(cx, ids!(pick)).clicked(actions) {
                    continue;
                }
                let host = app_state()
                    .lock()
                    .unwrap()
                    .found_servers
                    .get(item_id)
                    .map(|f| f.host.clone());
                if let Some(host) = host {
                    self.ui
                        .text_input(cx, ids!(client_host_input))
                        .set_text(cx, &host);
                    let msg = {
                        let st = app_state().lock().unwrap();
                        st.t("server_host_selected").replace("{host}", &host)
                    };
                    self.set_status(cx, msg);
                    self.ui.redraw(cx);
                }
            }
        }
        if self.ui.button(cx, ids!(client_clear_refused)).clicked(actions) {
            let result = {
                let st = app_state().lock().unwrap();
                st.store.as_ref().ok_or_else(|| "No store".to_string()).and_then(phone::clear_refused)
            };
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_unpair)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.clear_client_pairing()?;
                st.client_paired = false;
                st.status = "Forgot the Server — pair again to sell".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_skip_lock)).clicked(actions) {
            if app_state().lock().unwrap().client_paired {
                self.go(cx, Screen::Lock);
            } else {
                self.set_status(cx, "Pair first");
            }
        }

        if self.ui.button(cx, ids!(receipt_print_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .format_receipt_text(&sale_id)
            })();
            match result {
                Ok(text) => self.print_or_share_receipt(cx, &text),
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(receipt_share_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .format_receipt_text(&sale_id)
            })();
            match result {
                Ok(text) => self.copy_text_to_clipboard(cx, &text),
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(day_close_copy)).clicked(actions) {
            let text = app_state().lock().unwrap().day_close_text.clone();
            if text.is_empty() {
                self.set_status(cx, "Open Day close first");
            } else {
                self.copy_text_to_clipboard(cx, &text);
            }
        }
        {
            let today =
                self.choice_chip_clicked(cx, actions, ids!(reports_today), ids!(reports_today_on));
            let month =
                self.choice_chip_clicked(cx, actions, ids!(reports_month), ids!(reports_month_on));
            let aging =
                self.choice_chip_clicked(cx, actions, ids!(reports_aging), ids!(reports_aging_on));
            let bs = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_balance_sheet),
                ids!(reports_balance_sheet_on),
            );
            if today || month || aging || bs {
                let kind = if aging {
                    Some(BooksReportKind::Aging)
                } else if bs {
                    Some(BooksReportKind::BalanceSheet)
                } else if month {
                    Some(BooksReportKind::Month)
                } else if today {
                    Some(BooksReportKind::Today)
                } else {
                    None
                };
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.reports_nav = ReportsNav::Bank;
                    if let Some(k) = kind {
                        st.books_report_kind = k;
                    }
                    st.load_books_report()?;
                    st.nav_back.clear();
                    st.nav_forward.clear();
                    st.screen = Screen::Reports;
                    st.status = match st.books_report_kind {
                        BooksReportKind::Aging => st.t("book_kind_aging"),
                        BooksReportKind::BalanceSheet => st.t("book_kind_balance"),
                        BooksReportKind::Month => st.t("book_kind_month"),
                        BooksReportKind::Today => st.t("book_kind_today"),
                    };
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }

    }
}

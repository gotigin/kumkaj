impl App {
    fn handle_actions_settings_client(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(country_apply)).clicked(actions) {
            let code = self.ui.text_input(cx, ids!(country_code_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change the country pack")?;
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Only the Owner can change the country pack".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let settings = store.apply_country_pack(&code)?;
                store.audit(
                    &actor,
                    "fiscal.country",
                    &format!(
                        "{} {} {}",
                        settings.country_code,
                        settings.profile().tax_system.as_str(),
                        settings.tax_pricing.as_str()
                    ),
                )?;
                let name = settings.profile().name_en;
                st.reload_all()?;
                st.country_browse.clear();
                st.status = format!("{name} pack applied — check rates on Items");
                Ok(name.to_string())
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
                TaxPricing::Inclusive,
            ),
            (
                ids!(tax_exclusive_btn),
                ids!(tax_exclusive_btn_on),
                TaxPricing::Exclusive,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, active) {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Change tax pricing")?;
                    let actor = st.actor()?;
                    if !actor.is_owner() {
                        return Err("Only the Owner can change tax pricing".into());
                    }
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.set_tax_pricing(pricing)?;
                    st.reload_all()?;
                    st.status = format!("{} — {}", st.fiscal.tax_label(), pricing.label());
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(fiscal_save)).clicked(actions) {
            let id_text = self.ui.text_input(cx, ids!(business_tax_id_input)).text();
            let service_text = self.ui.text_input(cx, ids!(service_bps_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Set the business id")?;
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Only the Owner can set the business id".into());
                }
                let service_bps = locale::parse_percent_bps(&service_text)?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_business_tax_id(&id_text)?;
                store.set_service_charge_bps(service_bps)?;
                st.reload_all()?;
                let id_status = if st.fiscal.business_tax_id.is_empty() {
                    format!("{} cleared", st.fiscal.business_id_label)
                } else {
                    format!(
                        "{} saved — it prints on every bill",
                        st.fiscal.business_id_label
                    )
                };
                st.status = if st.fiscal.service_charge_bps > 0 {
                    format!(
                        "{id_status} · service {}% added to every bill",
                        locale::format_percent_bps(
                            st.fiscal.service_charge_bps,
                            st.fiscal.decimal_sep()
                        )
                    )
                } else {
                    id_status
                };
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
                st.screen = unlocked_home();
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
            self.set_report_rows(cx, ids!(client_found_list), Vec::new());
            let _ = std::thread::Builder::new()
                .name("courier-find-server".into())
                .spawn(|| {
                    let found = net::discover_servers(std::time::Duration::from_millis(3000));
                    *discovery_result().lock().unwrap() = Some(found);
                    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                });
        }
        let pair_enter = self
            .ui
            .text_input(cx, ids!(client_code_input))
            .returned(actions)
            .is_some()
            || self
                .ui
                .text_input(cx, ids!(client_host_input))
                .returned(actions)
                .is_some();
        if pair_enter || self.ui.button(cx, ids!(client_pair_btn)).clicked(actions) {
            let host = self.ui.text_input(cx, ids!(client_host_input)).text();
            let code = self.ui.text_input(cx, ids!(client_code_input)).text();
            // Three counter devices must not all be "Android device" in the office's list.
            let name = Some(self.ui.text_input(cx, ids!(client_name_input)).text())
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(device_name);
            if host.trim().is_empty() || code.trim().is_empty() {
                self.set_status(cx, "Type the office device's address and its pairing code");
            } else {
                app_state().lock().unwrap().pair_note = "Pairing…".into();
                self.set_status(cx, "Pairing…");
                let _ = std::thread::Builder::new()
                    .name("courier-pair".into())
                    .spawn(move || {
                        let reply = net::client_pair(&host, &code, name.trim());
                        *lan_once().lock().unwrap() = Some(LanOnce::Paired(reply));
                        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                    });
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_sync_btn)).clicked(actions) {
            phone::want_sync();
            self.set_status(cx, "Refreshing from the office…");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_connection_btn)).clicked(actions) {
            self.go(cx, Screen::ClientPair);
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

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        app_state().lock().unwrap().live_timer = cx.start_interval(LIVE_REFRESH_SECS);
        self.sync_chrome(cx);
    }

    fn handle_timer(&mut self, cx: &mut Cx, e: &TimerEvent) {
        // Floor / Kitchen / Order desk stay live: states, minutes, and what
        // waiters sent from their tablets. Only while one of them shows.
        // (A device's write also refreshes them at once — `BOOKS_CHANGED`.)
        let (live, settings) = {
            let st = app_state().lock().unwrap();
            let ours = st.live_timer.is_timer(e).is_some() && st.session.is_some();
            (
                ours && matches!(st.screen, Screen::Floor | Screen::Kitchen | Screen::Pos),
                ours && st.screen == Screen::Settings && st.device_role.is_host(),
            )
        };
        if live {
            {
                let mut st = app_state().lock().unwrap();
                if st.device_role == DeviceRole::Client {
                    st.pull_from_server();
                } else {
                    st.refresh_live_lists();
                }
            }
            self.ui.redraw(cx);
            return;
        }
        if settings {
            // Which devices are connected right now.
            {
                let mut st = app_state().lock().unwrap();
                if let Some(devices) = st.store.as_ref().and_then(|s| s.list_lan_devices().ok()) {
                    st.lan_devices = devices;
                }
            }
            self.ui.widget(cx, ids!(paired_list)).redraw(cx);
            return;
        }
        let (undo_hit, snack_hit) = {
            let st = app_state().lock().unwrap();
            (
                !st.undo_timer.is_empty() && st.undo_timer.is_timer(e).is_some(),
                !st.snack_timer.is_empty() && st.snack_timer.is_timer(e).is_some(),
            )
        };
        if undo_hit {
            self.clear_undo_snackbar(cx);
            self.sync_chrome(cx);
        }
        if snack_hit {
            self.clear_snack(cx);
        }
    }

    fn handle_back_pressed(&mut self, cx: &mut Cx) -> bool {
        self.navigate_back(cx)
    }

    fn handle_key_down(&mut self, cx: &mut Cx, e: &KeyEvent) {
        if e.key_code == KeyCode::Escape
            || e.key_code == KeyCode::Back
            || (e.key_code == KeyCode::ArrowLeft && e.modifiers.alt)
        {
            if nav_gesture_ok() {
                let _ = self.navigate_back(cx);
            }
            return;
        }
        if e.key_code == KeyCode::ArrowRight && e.modifiers.alt {
            if nav_gesture_ok() {
                let _ = self.navigate_forward(cx);
            }
            return;
        }

        // Page ScrollYView: ↑/↓ a line, PgUp/PgDn a screenful, Home/End the ends.
        // Skip when a TextInput (or other widget) holds key focus.
        if self.handle_scroll_key(cx, e) {
            return;
        }

        // Lock screen: Enter unlocks. Digits live in `lock_pin_input` (system
        // keyboard / IME) — no custom pad and no double-insert via KeyDown.
        let on_lock = app_state().lock().unwrap().screen == Screen::Lock;
        if on_lock
            && !e.is_repeat
            && matches!(e.key_code, KeyCode::ReturnKey | KeyCode::NumpadEnter)
            && !e.modifiers.control
            && !e.modifiers.alt
            && !e.modifiers.logo
        {
            self.sync_pin_buffer_from_field(cx);
            self.try_unlock(cx);
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.tab_chevron_clicks(cx, actions);
        for action in actions.iter() {
            if let Some(fd) = action.downcast_ref::<FileDialogAction>() {
                match fd {
                    // The only folder picker: the second place for backups.
                    FileDialogAction::FolderSelected(dir) => {
                        let result = (|| -> Result<String, String> {
                            let st = app_state().lock().unwrap();
                            st.require_server_role("Choose where backups go")?;
                            let store = st.store.as_ref().ok_or("No store")?;
                            store.set_backup_copy_dir(Some(dir))?;
                            Ok(st.t("backup_copy_to").replace("{dir}", &dir.display().to_string()))
                        })();
                        match result {
                            Ok(msg) | Err(msg) => self.set_status(cx, msg),
                        }
                        self.sync_chrome(cx);
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(restore_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            // Android's picker hands back a content:// URI:
                            // copy it beside the books first.
                            match self.materialize_picked_backup(cx, path) {
                                Ok(file) => self.restore_backup_from(cx, file),
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::SaveFileSelected { id, path }
                        if *id == live_id!(backup_save_pick) =>
                    {
                        self.write_backup_copy(cx, path);
                    }
                    FileDialogAction::SaveFileCancelled { id }
                        if *id == live_id!(backup_save_pick) =>
                    {
                        if let Some(file) = app_state().lock().unwrap().pending_backup_copy.take() {
                            let _ = std::fs::remove_file(file);
                        }
                    }
                    FileDialogAction::SaveFileSelected { id, path }
                        if *id == live_id!(diag_save_pick) =>
                    {
                        self.write_diag_export_copy(cx, path);
                    }
                    FileDialogAction::SaveFileCancelled { id }
                        if *id == live_id!(diag_save_pick) =>
                    {
                        // Keep the in-app bundle; just drop the pending pointer.
                        app_state().lock().unwrap().pending_diag_export = None;
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(item_photo_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_item_photo_picked(cx, path.clone());
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(venue_icon_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    invalidate_image_loads_for_path(&file);
                                    app_state().lock().unwrap().draft_venue_icon_path =
                                        Some(file);
                                    self.set_status(cx, "Icon selected — tap Save brand");
                                    self.sync_chrome(cx);
                                }
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(onboard_icon_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    let outcome = (|| -> Result<String, String> {
                                        let mut st = app_state().lock().unwrap();
                                        let store = st
                                            .store
                                            .as_ref()
                                            .ok_or_else(|| "Storage not ready".to_string())?;
                                        let key = store.import_venue_icon(&file)?;
                                        if let Ok(dir) = store.brand_images_dir() {
                                            invalidate_image_loads_for_path(
                                                &dir.join("venue_icon.webp"),
                                            );
                                        }
                                        invalidate_image_loads_for_path(&file);
                                        st.venue_icon_key = Some(key);
                                        Ok("Shop logo saved".to_string())
                                    })();
                                    match outcome {
                                        Ok(msg) => self.set_status(cx, msg),
                                        Err(e) => self.set_status(cx, e),
                                    }
                                    self.sync_chrome(cx);
                                }
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(print_stamp_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    let outcome = (|| -> Result<String, String> {
                                        let st = app_state().lock().unwrap();
                                        st.require_server_role("Change the bill format")?;
                                        let store = st
                                            .store
                                            .as_ref()
                                            .ok_or_else(|| "Storage not ready".to_string())?;
                                        let _ = store.import_print_stamp(&file)?;
                                        Ok("Stamp saved for bill faces".to_string())
                                    })();
                                    match outcome {
                                        Ok(msg) => {
                                            self.fill_bill_format_fields(cx);
                                            self.set_status(cx, msg);
                                        }
                                        Err(e) => self.set_status(cx, e),
                                    }
                                    self.sync_chrome(cx);
                                }
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(payment_qr_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    let outcome = (|| -> Result<String, String> {
                                        let st = app_state().lock().unwrap();
                                        st.require_server_role("Change the bill format")?;
                                        let store = st
                                            .store
                                            .as_ref()
                                            .ok_or_else(|| "Storage not ready".to_string())?;
                                        let _ = store.import_payment_qr(&file)?;
                                        Ok("Payment QR saved for bill faces".to_string())
                                    })();
                                    match outcome {
                                        Ok(msg) => {
                                            self.fill_bill_format_fields(cx);
                                            self.set_status(cx, msg);
                                        }
                                        Err(e) => self.set_status(cx, e),
                                    }
                                    self.sync_chrome(cx);
                                }
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(category_photo_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_category_photo_picked(cx, path.clone());
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(entry_attach_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_entry_attach_picked(cx, path.clone());
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(csv_parties_pick)
                            || *id == live_id!(csv_items_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            let parties = *id == live_id!(csv_parties_pick);
                            let outcome = (|| -> Result<String, String> {
                                let text = std::fs::read_to_string(path)
                                    .map_err(|e| format!("Could not read the file: {e}"))?;
                                let mut st = app_state().lock().unwrap();
                                st.require_server_role("Import from CSV")?;
                                let store =
                                    st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                                let (added, skipped, problems) = if parties {
                                    store.import_parties_csv(&text)?
                                } else {
                                    store.import_items_csv(&text)?
                                };
                                st.reload_all()?;
                                let mut msg = format!(
                                    "Imported {added} new, skipped {skipped} (already there or unusable)"
                                );
                                if let Some(p) = problems.first() {
                                    msg.push_str(&format!(" · first problem: {p}"));
                                }
                                Ok(msg)
                            })();
                            match outcome {
                                Ok(msg) => self.set_status(cx, msg),
                                Err(e) => self.set_status(cx, e),
                            }
                            self.sync_chrome(cx);
                        }
                    }
                    FileDialogAction::FileCancelled { id }
                        if *id == live_id!(item_photo_pick)
                            || *id == live_id!(venue_icon_pick)
                            || *id == live_id!(onboard_icon_pick)
                            || *id == live_id!(csv_parties_pick)
                            || *id == live_id!(csv_items_pick)
                            || *id == live_id!(entry_attach_pick) =>
                    {
                        app_state().lock().unwrap().attach_pick = None;
                        self.set_status(cx, "Picker cancelled");
                    }
                    _ => {}
                }
            }
        }
        // Whole-rail hide / restore (sidebar mode only).
        if self.ui.button(cx, ids!(rail_collapse_btn)).clicked(actions) {
            app_state().lock().unwrap().rail_collapsed = true;
            self.apply_shell_layout(cx, true);
        }
        if self.ui.button(cx, ids!(rail_expand_btn)).clicked(actions) {
            app_state().lock().unwrap().rail_collapsed = false;
            self.apply_shell_layout(cx, true);
        }
        // Nav clicks honour the user's slot arrangement.
        let nav_click = self.nav_tab_clicked(cx, actions);
        match nav_click {
            Some(NavTab::Books) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.refresh_books_overview()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    // Drop leftover focus so the old Sell face cannot look selected.
                    cx.set_key_focus(Area::Empty);
                    self.go_peer(cx, Screen::Books);
                }
            }
            Some(NavTab::Sell) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Pos, |s| s.can_access_sell());
            }
            Some(NavTab::Buy) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Buy, |s| {
                    s.can_access_inventory() || s.can_access_sales() || s.is_owner()
                });
            }
            Some(NavTab::Menu) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Menu, |s| s.can_access_items());
            }
            Some(NavTab::Floor) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Floor, |s| {
                    s.can_access_floor() || s.is_owner()
                });
            }
            Some(NavTab::Kitchen) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Kitchen, |s| {
                    s.can_access_kitchen() || s.is_owner()
                });
            }
            Some(NavTab::Sales) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Sales, |s| s.can_access_sales());
            }
            Some(NavTab::Credit) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Credit, |s| s.can_access_credit());
            }
            Some(NavTab::Team) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Team, |s| {
                    s.can_access_team() || s.is_owner()
                });
            }
            Some(NavTab::Settings) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Settings, |s| {
                    s.can_access_backup() || s.is_owner()
                });
            }
            Some(NavTab::DayClose) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_sales() {
                        return Err("Not allowed".into());
                    }
                    st.refresh_day_close_text()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::DayClose);
                }
            }
            Some(NavTab::Reports) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_sales() {
                        return Err("Not allowed".into());
                    }
                    st.refresh_reports_facet()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::Reports);
                }
            }
            Some(NavTab::Inventory) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.can_access_inventory() && !actor.is_owner() {
                        return Err("Not allowed for your role".into());
                    }
                    st.refresh_inventory_text()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::Inventory);
                }
            }
            Some(NavTab::Branches) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.can_access_branches() && !actor.is_owner() {
                        return Err("Not allowed for your role".into());
                    }
                    st.refresh_branches_text()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::Branches);
                }
            }
            Some(NavTab::Lock) => {
                self.clear_undo_snackbar(cx);
                let dest = {
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref() {
                        let _ = store.set_active_session_staff_id(None);
                    }
                    st.session = None;
                    st.pin_buffer.clear();
                    st.status = "Locked".into();
                    if st.must_confirm_device_role {
                        Screen::DeviceSetup
                    } else {
                        Screen::Lock
                    }
                };
                self.go_peer(cx, dest);
            }
            None => {}
        }
        if self.ui.button(cx, ids!(settings_hub_sidebar)).clicked(actions) {
            self.go_if_allowed(cx, Screen::SidebarSettings, |s| {
                s.can_access_backup() || s.is_owner()
            });
        }
        if self.ui.button(cx, ids!(sidebar_settings_back)).clicked(actions) {
            self.navigate_back(cx);
        }
        self.handle_sidebar_nav_list_actions(cx, actions);
        if std::mem::take(&mut *sidebar_order_dirty().lock().unwrap()) {
            let msg = app_state().lock().unwrap().t("sidebar_order_saved");
            self.set_status(cx, msg);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(amounts_eye)).clicked(actions) {
            self.toggle_amounts_screen(cx);
        }
        for (eye, key) in [
            (ids!(books_in_eye), "books_in"),
            (ids!(books_purchases_eye), "books_purchases"),
            (ids!(books_expenses_eye), "books_expenses"),
            (ids!(books_gross_eye), "books_gross"),
            (ids!(books_net_eye), "books_net"),
            (ids!(books_declared_eye), "books_declared"),
            (ids!(books_owner_eye), "books_owner"),
            (ids!(books_cash_eye), "books_cash"),
            (ids!(books_digital_eye), "books_digital"),
            (ids!(books_card_eye), "books_card"),
            (ids!(books_they_owe_eye), "books_they_owe"),
            (ids!(books_we_owe_eye), "books_we_owe"),
            (ids!(books_bank_balance_eye), "books_bank_balance"),
            (ids!(books_till_eye), "books_till"),
            (ids!(books_bank_net_eye), "books_bank_net"),
        ] {
            if self.ui.button(cx, eye).clicked(actions) {
                self.toggle_amount_key(cx, key);
            }
        }
        for (hdr, key) in [
            (ids!(books_sec_today_hdr), "today"),
            (ids!(books_sec_bills_hdr), "bills"),
            (ids!(books_sec_payments_hdr), "payments"),
            (ids!(books_sec_outstanding_hdr), "outstanding"),
            (ids!(books_sec_bank_hdr), "bank"),
        ] {
            if self.ui.button(cx, hdr).clicked(actions) {
                self.toggle_books_section(cx, key);
            }
        }
        for (hdr, key) in [
            (ids!(floor_ops_hdr), "floor_ops"),
            (ids!(waitlist_hdr), "waitlist"),
            (ids!(resv_hdr), "resv"),
            (ids!(floor_add_hdr), "floor_add"),
            (ids!(stations_hdr), "stations"),
        ] {
            if self.ui.button(cx, hdr).clicked(actions) {
                self.toggle_floor_section(cx, key);
            }
        }
        for (hdr, key) in [
            (ids!(setup_lang_hdr), "language"),
            (ids!(setup_role_hdr), "role"),
        ] {
            if self.ui.button(cx, hdr).clicked(actions) {
                self.toggle_device_setup_section(cx, key);
            }
        }
        if self.ui.button(cx, ids!(books_to_sell)).clicked(actions) {
            self.go_peer_if_allowed(cx, Screen::Pos, |s| s.can_access_sell());
        }
        if self.ui.button(cx, ids!(books_to_buy)).clicked(actions) {
            self.go_peer_if_allowed(cx, Screen::Buy, |s| {
                s.can_access_inventory() || s.can_access_sales() || s.is_owner()
            });
        }
        if self.ui.button(cx, ids!(books_day_close)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed".into());
                }
                st.refresh_day_close_text()?;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.go_peer(cx, Screen::DayClose);
            }
        }
        if self.ui.button(cx, ids!(books_reports)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed".into());
                }
                st.refresh_reports_facet()?;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.go_peer(cx, Screen::Reports);
            }
        }
        if self.ui.button(cx, ids!(books_credit)).clicked(actions) {
            self.go_peer_if_allowed(cx, Screen::Credit, |s| s.can_access_credit());
        }
        if self.choice_chip_clicked(cx, actions, ids!(buy_paid), ids!(buy_paid_on)) {
            app_state().lock().unwrap().buy_on_account = false;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(buy_on_account), ids!(buy_on_account_on)) {
            app_state().lock().unwrap().buy_on_account = true;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(buy_purchase)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(buy_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(buy_qty_input)).text();
            let cost_text = self.ui.text_input(cx, ids!(buy_cost_input)).text();
            let supplier = self.ui.text_input(cx, ids!(buy_supplier_input)).text();
            let party_kind = self.ui.text_input(cx, ids!(buy_party_kind)).text();
            let note = self.ui.text_input(cx, ids!(buy_note_input)).text();
            let unit_text = self.ui.text_input(cx, ids!(buy_unit_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Record purchases")?;
                let actor = st.actor()?;
                let on_account = st.buy_on_account;
                let qty_milli = parse_qty_to_milli(&qty_text)?;
                // What the kitchen buys: an item it already stocks, or a new
                // raw material by this name — never one of its own dishes.
                let product = st.stock_item_or_new(&name, &unit_text)?;
                let currency = st.locale.currency.clone();
                let unit_cost = match cost_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let mut party_id = String::new();
                if on_account || !supplier.trim().is_empty() {
                    if on_account {
                        let kind = if party_kind.trim().is_empty() {
                            "Supplier"
                        } else {
                            party_kind.trim()
                        };
                        // The supplier already on the books (picked or typed
                        // the same) — a new party only for a new name.
                        let existing = st
                            .customers
                            .iter()
                            .find(|c| c.active && c.name.trim().eq_ignore_ascii_case(supplier.trim()))
                            .map(|c| c.id.clone());
                        party_id = match existing {
                            Some(id) => id,
                            None => store.add_customer_kind(supplier.trim(), "", kind)?.id,
                        };
                    }
                }
                let purchase = store.record_purchase_ex(
                    supplier.trim(),
                    &party_id,
                    on_account,
                    &[(product.id.clone(), qty_milli, unit_cost)],
                    note.trim(),
                    &actor,
                )?;
                st.reload_all()?;
                let _ = st.refresh_books_overview();
                let qty_show = domain::format_qty_display(qty_milli, 0, &product.unit_label);
                Ok(format!(
                    "Received {qty_show} {} · {}{}",
                    product.name,
                    st.money(Money(purchase.total_paise)),
                    if on_account { " · on account" } else { "" }
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(buy_qty_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(buy_cost_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        // Inventory: tapping a stock row picks it for + / − Stock.
        {
            let widget = self.ui.widget(cx, ids!(stock_list));
            let list = widget.portal_list(cx, ids!(list));
            let mut picked = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item.view(cx, ids!(row)).finger_up(actions).is_some_and(|e| e.was_tap() && e.is_over) {
                    picked = app_state().lock().unwrap().stock_items().get(item_id).map(|p| p.name.clone());
                }
            }
            if let Some(name) = picked {
                self.ui.text_input(cx, ids!(inv_item_input)).set_text(cx, &name);
                self.ui.text_input(cx, ids!(inv_qty_input)).set_key_focus(cx);
            }
        }
        // Ways the expense was paid: pick a method per row, type its amount,
        // remove a row, or add another way.
        {
            let widget = self.ui.widget(cx, ids!(buy_parts));
            let list = widget.portal_list(cx, ids!(list));
            let mut changed = false;
            let mut remove: Option<usize> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                for (i, (idle, on)) in PART_CHIPS.iter().enumerate() {
                    if item.button(cx, idle).clicked(actions) || item.button(cx, on).clicked(actions) {
                        if let Some(part) = app_state().lock().unwrap().pay_parts.get_mut(item_id) {
                            part.method = PART_METHODS[i];
                        }
                        changed = true;
                    }
                }
                if let Some(text) = item.text_input(cx, ids!(part_amount)).changed(actions) {
                    if let Some(part) = app_state().lock().unwrap().pay_parts.get_mut(item_id) {
                        part.amount = text;
                    }
                    changed = true;
                }
                if item.button(cx, ids!(part_remove)).clicked(actions) {
                    remove = Some(item_id);
                }
            }
            if let Some(i) = remove {
                let mut st = app_state().lock().unwrap();
                if st.pay_parts.len() > 1 {
                    st.pay_parts.remove(i);
                }
                changed = true;
            }
            if self.ui.button(cx, ids!(buy_parts_add)).clicked(actions) {
                let mut st = app_state().lock().unwrap();
                // The next way not used yet (cash, then UPI, card, bank).
                let next = PART_METHODS
                    .iter()
                    .copied()
                    .find(|m| !st.pay_parts.iter().any(|p| p.method == *m))
                    .unwrap_or(PayMethod::Cash);
                st.pay_parts.push(PayPart { method: next, amount: String::new() });
                changed = true;
            }
            if changed || self.ui.text_input(cx, ids!(buy_expense_amount)).changed(actions).is_some() {
                self.ui.widget(cx, ids!(buy_parts)).redraw(cx);
                self.sync_parts_status(cx);
            }
        }
        if self.ui.button(cx, ids!(buy_expense_add)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(buy_expense_amount)).text();
            let note = self.ui.text_input(cx, ids!(buy_expense_note)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.device_role == DeviceRole::Client {
                    return Err("Log expenses on the shop device — this is an extra till".into());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                let parts = settle_pay_parts(minor, &st.pay_parts, &currency)?;
                let expense_id = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_expense_tenders(minor, "misc", &note, &actor, &parts)?
                };
                st.selected_expense_id = Some(expense_id);
                st.pay_parts = vec![PayPart::default()];
                let _ = st.refresh_books_overview();
                st.status = "Expense saved".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(buy_expense_amount)).set_text(cx, "");
                self.ui.text_input(cx, ids!(buy_expense_note)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(opening_float_save)).clicked(actions) {
            let text = self.ui.text_input(cx, ids!(opening_float_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Set the opening cash")?;
                let actor = st.actor()?;
                let paise = locale::parse_major_input(text.trim(), &st.locale.currency)?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .set_opening_float(paise, &actor)?;
                st.refresh_day_close_text()?;
                Ok(format!("Opening cash {} saved", st.money(Money(paise))))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(shift_close_btn)).clicked(actions) {
            let counted = self.ui.text_input(cx, ids!(shift_counted_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Close the shift")?;
                let actor = st.actor()?;
                let paise = locale::parse_major_input(counted.trim(), &st.locale.currency)
                    .map_err(|_| "Count the drawer and type the cash".to_string())?;
                let (shift, slip) = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .close_shift(&actor, paise)?;
                let dir = st.store.as_ref().and_then(|s| s.db_path().parent().map(|p| p.to_path_buf()));
                let _ = print_share::print_text(print_share::PrintKind::Bill, &slip, dir.as_deref());
                st.last_receipt = slip;
                let v = shift.variance_paise();
                Ok(if v == 0 {
                    "Shift closed — drawer matches".to_string()
                } else if v < 0 {
                    format!("Shift closed — short {}", st.money(Money(-v)))
                } else {
                    format!("Shift closed — over {}", st.money(Money(v)))
                })
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(shift_counted_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(day_close_commit)).clicked(actions) {
            let counted_text = self.ui.text_input(cx, ids!(day_close_counted_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Record the day close")?;
                let actor = st.actor()?;
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to close the day".into());
                }
                let currency = st.locale.currency.clone();
                let counted = match counted_text.trim() {
                    "" => None,
                    other => Some(locale::parse_major_input(other, &currency)?),
                };
                let at = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .commit_day_close_with_count(&actor, counted)?;
                st.refresh_day_close_text()?;
                Ok(at)
            })();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(at) => {
                    let shown = at[..at.len().min(19)].replace('T', " ");
                    self.set_status(cx, format!("Day close recorded at {shown}"));
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_pair_back)).clicked(actions) {
            let _ = self.navigate_back(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(lang_setup_platform),
            ids!(lang_setup_platform_on),
        ) {
            let mut st = app_state().lock().unwrap();
            st.language_follow_platform = true;
            st.locale.language = locale::detect_platform_language();
            st.status = st.t("lang_follow_platform");
            drop(st);
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(lang_setup_user), ids!(lang_setup_user_on)) {
            let mut st = app_state().lock().unwrap();
            st.language_follow_platform = false;
            // Default the fixed pick to the current platform language.
            st.locale.language = locale::detect_platform_language();
            st.status = st.t("lang_pick_user");
            drop(st);
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(settings_lang_platform),
            ids!(settings_lang_platform_on),
        ) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change language")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                st.locale = store.set_platform_language()?;
                st.language_follow_platform = true;
                st.save_remembered_language();
                st.status = st.t("lang_follow_platform");
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(settings_lang_user),
            ids!(settings_lang_user_on),
        ) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change language")?;
                let platform = locale::detect_platform_language();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                // Enter user mode with platform language as the starting pick.
                st.locale = store.set_user_language(&platform)?;
                st.language_follow_platform = false;
                st.save_remembered_language();
                st.status = st.t("lang_pick_user");
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(device_role_server),
            ids!(device_role_server_on),
        ) {
            self.begin_server_mode(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(device_role_client),
            ids!(device_role_client_on),
        ) {
            self.begin_client_mode(cx);
        }
        if self.ui.button(cx, ids!(server_create_new)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(server_name_input)).text();
            if let Err(e) = self.activate_new_server(cx, &name) {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(server_name_input))
                    .set_text(cx, "");
            }
        }
        if self.ui.button(cx, ids!(server_pick_cancel)).clicked(actions) {
            app_state().lock().unwrap().picking_server = false;
            self.sync_chrome(cx);
        }
        {
            let widget = self.ui.widget(cx, ids!(setup_server_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let id = app_state()
                        .lock()
                        .unwrap()
                        .local_servers
                        .get(item_id)
                        .map(|s| s.id.clone());
                    if let Some(id) = id {
                        if let Err(e) = self.activate_existing_server(cx, &id) {
                            self.set_status(cx, e);
                        }
                    }
                }
            }
        }
        if self.ui.button(cx, ids!(allow_extra_tills)).clicked(actions) {
            self.set_extra_tills(cx, true);
        }
        if self.ui.button(cx, ids!(stop_extra_tills)).clicked(actions) {
            self.set_extra_tills(cx, false);
        }
        if self.ui.button(cx, ids!(autostart_btn)).clicked(actions) {
            let on = !startup::is_on();
            let msg = match startup::set(on) {
                Ok(()) if on => app_state().lock().unwrap().t("autostart_done_on"),
                Ok(()) => app_state().lock().unwrap().t("autostart_done_off"),
                Err(e) => e,
            };
            self.set_status(cx, msg);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(backup_copy_btn)).clicked(actions) {
            if cfg!(target_os = "android") {
                self.save_backup_copy(cx);
            } else {
                let title = app_state().lock().unwrap().t("backup_copy_btn");
                cx.open_select_folder_dialog(FileDialog::new().set_title(title));
            }
        }
        if self.ui.button(cx, ids!(new_pair_code)).clicked(actions) {
            // Devices already paired keep working; the old code pairs no more.
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Change the pairing code")?;
                let actor = st.actor()?;
                let code = st.store.as_ref().ok_or("No store")?.new_pairing_code(&actor)?;
                Ok(code)
            })();
            match result {
                Ok(code) => self.set_status(
                    cx,
                    format!("New pairing code {code} — devices already paired keep working"),
                ),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        {
            let widget = self.ui.widget(cx, ids!(paired_list));
            let list = widget.portal_list(cx, ids!(list));
            let mut forget = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    forget = Some(item_id);
                }
            }
            if let Some(index) = forget {
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Forget a device")?;
                    let device = st.lan_devices.get(index).cloned().ok_or("Device gone")?;
                    let actor = st.actor()?;
                    let store = st.store.as_ref().ok_or("No store")?;
                    store.forget_lan_device(&device.id, &actor)?;
                    st.lan_devices = store.list_lan_devices()?;
                    Ok(format!("Forgot {} — it must pair again to work", device.name))
                })();
                match result {
                    Ok(msg) | Err(msg) => self.set_status(cx, msg),
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(allow_firewall)).clicked(actions) {
            firewall_job(|| net::allow_phones_through_firewall().map(|()| true));
            self.sync_chrome(cx);
        }

        if let Some(text) = self.ui.text_input(cx, ids!(lock_pin_input)).changed(actions) {
            let digits: String = text.chars().filter(|c| c.is_ascii_digit()).take(12).collect();
            if digits != text {
                self.ui.text_input(cx, ids!(lock_pin_input)).set_text(cx, &digits);
            }
            {
                let mut st = app_state().lock().unwrap();
                st.pin_buffer = digits;
                if st.status.starts_with("Wrong")
                    || st.status.starts_with("Enter at least")
                    || st.status.starts_with("Storage")
                    || st.status.starts_with("Add staff")
                {
                    st.status.clear();
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(pin_reveal)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.pin_visible = !st.pin_visible;
            }
            let show = app_state().lock().unwrap().pin_visible;
            self.ui
                .text_input(cx, ids!(lock_pin_input))
                .set_is_password(cx, !show);
            self.sync_chrome(cx);
        }
        // Soft-keyboard Done / physical Enter on the PIN field = Unlock.
        if self.ui.text_input(cx, ids!(lock_pin_input)).returned(actions).is_some()
            || self.ui.button(cx, ids!(pin_ok)).clicked(actions)
        {
            self.sync_pin_buffer_from_field(cx);
            self.try_unlock(cx);
        }
        if self.ui.text_input(cx, ids!(lock_login_password)).returned(actions).is_some() {
            self.try_unlock_with_login(cx);
        }
        if self.ui.button(cx, ids!(role_owner)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.lock_as_owner = true;
                st.pin_buffer.clear();
                st.status = "Unlock as Owner".into();
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(role_staff)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                let has_staff = st.staff.iter().any(|s| !s.is_owner());
                if !st.owner_setup_complete || !has_staff {
                    st.lock_as_owner = true;
                    st.status = "Set up Owner first, then add staff in Team".into();
                } else {
                    st.lock_as_owner = false;
                    st.pin_buffer.clear();
                    if st.lock_staff_id.is_none() {
                        st.lock_staff_id = st
                            .staff
                            .iter()
                            .find(|s| !s.is_owner())
                            .map(|s| s.id.clone());
                    }
                    st.status = "Unlock as Staff".into();
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(lock_staff_prev)).clicked(actions)
            || self.ui.button(cx, ids!(lock_staff_next)).clicked(actions)
        {
            let forward = self.ui.button(cx, ids!(lock_staff_next)).clicked(actions);
            {
                let mut st = app_state().lock().unwrap();
                let ids: Vec<String> = st
                    .staff
                    .iter()
                    .filter(|s| !s.is_owner())
                    .map(|s| s.id.clone())
                    .collect();
                if !ids.is_empty() {
                    let cur = st
                        .lock_staff_id
                        .as_ref()
                        .and_then(|id| ids.iter().position(|x| x == id))
                        .unwrap_or(0);
                    let next = if forward {
                        (cur + 1) % ids.len()
                    } else {
                        (cur + ids.len() - 1) % ids.len()
                    };
                    st.lock_staff_id = Some(ids[next].clone());
                    st.pin_buffer.clear();
                }
            }
            self.sync_chrome(cx);
        }

        if self.choice_chip_clicked(cx, actions, ids!(lane_declared), ids!(lane_declared_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.money_lane = MoneyLane::Declared;
                st.cart.money_lane = MoneyLane::Declared;
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(lane_owner), ids!(lane_owner_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.money_lane = MoneyLane::Owner;
                st.cart.money_lane = MoneyLane::Owner;
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(pay_mix)).clicked(actions) && self.begin_pay(cx) {
            let typed_total = self.ui.text_input(cx, ids!(cart_total_input)).text();
            let cash_text = self.ui.text_input(cx, ids!(mix_cash_input)).text();
            let upi_text = self.ui.text_input(cx, ids!(mix_upi_input)).text();
            let card_text = self.ui.text_input(cx, ids!(mix_card_input)).text();
            let credit_text = self.ui.text_input(cx, ids!(mix_credit_input)).text();
            let tendered_text = self.ui.text_input(cx, ids!(tendered_input)).text();
            let tip_text = self.ui.text_input(cx, ids!(tip_input)).text();
            let typed_customer = self.ui.text_input(cx, ids!(customer_name_input)).text();
            let typed_delivery = self.ui.text_input(cx, ids!(delivery_by_input)).text();
            app_state().lock().unwrap().cart.delivery_by = typed_delivery;
            enum Pending {
                Local { sale_id: String, snack: String },
                // A ₹0 table closed with no sale (all NC, or dishes at ₹0).
                Closed { msg: String },
                // Saved on this device, on its way to the counter.
                Queued,
            }
            let parse_part = |text: &str, currency: &str| -> Result<i64, String> {
                let t = text.trim();
                if t.is_empty() {
                    return Ok(0);
                }
                locale::parse_major_input(t, currency)
            };
            let pending = (|| -> Result<Pending, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_sell() {
                    return Err("Not allowed to sell".into());
                }
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
                st.require_cart_modifiers()?;
                if st.selected_table_id.is_some()
                    && st.cart.lines.iter().any(|l| !l.is_sent())
                {
                    return Err(
                        "Send the new items to the kitchen first (Send KOT), then Take payment"
                            .into(),
                    );
                }
                // ₹0 bill: write a completed sale when Settings allows it
                // (default on). When disallowed, a table still frees with no
                // sale row so service is not stuck on "Bill printed".
                let allow_zero = st
                    .store
                    .as_ref()
                    .and_then(|s| s.allow_zero_sales().ok())
                    .unwrap_or(true);
                if st.cart.total().paise() == 0 && st.cart.charge_total.is_none() && !allow_zero {
                    let Some(table_id) = st.selected_table_id.clone() else {
                        return Err(
                            "Nothing to charge — give these items a price on Menu, or allow ₹0 sales in Settings"
                                .into(),
                        );
                    };
                    if st.cart.lines.iter().any(|l| !l.is_sent()) {
                        return Err("Nothing to charge — send the KOT, then Pay closes the table".into());
                    }
                    let name = st
                        .tables
                        .iter()
                        .find(|t| t.id == table_id)
                        .map(|t| t.name.clone())
                        .unwrap_or_default();
                    let done = format!("{name} closed — nothing to charge, table is free");
                    let acted = st.table_action(
                        net::RunningAction::TableReady { table_id },
                        "",
                        &done,
                        phone::Follow::Refresh,
                    )?;
                    st.cart.clear();
                    st.open_cart_line = None;
                    st.selected_table_id = None;
                    return Ok(Pending::Closed {
                        msg: match acted {
                            Acted::Here(_) => done,
                            Acted::Sent => format!("{name} — sending…"),
                        },
                    });
                }
                let currency = st.locale.currency.clone();
                if st.cart.charge_total.is_some()
                    && let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                        && minor > 0
                {
                    let _ = st.cart.set_charge_total(Money(minor));
                }
                let (schemes, products) = match st.store.as_ref() {
                    Some(store) => (
                        store.list_schemes().unwrap_or_default(),
                        store.list_products(true).unwrap_or_default(),
                    ),
                    None => (Vec::new(), Vec::new()),
                };
                {
                    let group_of = |id: &str| -> String {
                        products
                            .iter()
                            .find(|p| p.id == id)
                            .map(|p| p.group_name.clone())
                            .unwrap_or_default()
                    };
                    let _ = schemes::apply_schemes(&mut st.cart, &schemes, &group_of);
                }
                let mut draft = st.cart.to_sale_draft()?;
                draft.money_lane = st.money_lane;
                let charged = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.preview_sale_total(&draft)?
                };
                // One method takes the whole bill; Split reads the amounts.
                let (cash, upi, card, credit) = match st.pay_choice {
                    PayChoice::Cash => (charged, 0, 0, 0),
                    PayChoice::Upi => (0, charged, 0, 0),
                    PayChoice::Card => (0, 0, charged, 0),
                    PayChoice::Credit => (0, 0, 0, charged),
                    PayChoice::Split => (
                        parse_part(&cash_text, &currency)?,
                        parse_part(&upi_text, &currency)?,
                        parse_part(&card_text, &currency)?,
                        parse_part(&credit_text, &currency)?,
                    ),
                };
                if cash < 0 || upi < 0 || card < 0 || credit < 0 {
                    return Err("Payment amounts cannot be negative".into());
                }
                let tendered = parse_part(&tendered_text, &currency)?;
                let tip = parse_part(&tip_text, &currency)?;
                if tendered < 0 || tip < 0 {
                    return Err("Customer paid and tip cannot be negative".into());
                }
                if tendered > 0 && tendered < cash + tip {
                    return Err(format!(
                        "Customer paid {} but the cash part and tip need {}",
                        st.money(Money(tendered)),
                        st.money(Money(cash + tip))
                    ));
                }
                let mut parts: Vec<(PayMethod, i64)> = Vec::new();
                if cash > 0 {
                    parts.push((PayMethod::Cash, cash));
                }
                if upi > 0 {
                    parts.push((PayMethod::Upi, upi));
                }
                if card > 0 {
                    parts.push((PayMethod::Card, card));
                }
                if credit > 0 {
                    parts.push((PayMethod::Credit, credit));
                }
                // ₹0 bill: one cash tender at zero so Frequent/day book still
                // get the sale (no customer needed for a free cover).
                if parts.is_empty() && charged == 0 {
                    parts.push((PayMethod::Cash, 0));
                }
                if parts.is_empty() {
                    return Err(
                        "Type amounts in Cash / UPI / Card / Credit, then Take payment".into(),
                    );
                }
                let sum: i64 = parts.iter().map(|(_, a)| *a).sum();
                if sum != charged {
                    return Err(format!(
                        "Parts add to {} but bill is {} — adjust the amounts",
                        st.money(Money(sum)),
                        st.money(Money(charged))
                    ));
                }
                let is_client = st.device_role == DeviceRole::Client;
                // Loyalty / wallet / voucher: debit balances before the sale is
                // written (discount already on the cart). Reverse on failure.
                let pending_pts = st.pending_loyalty_points;
                let pending_wallet = st.pending_wallet_paise;
                let pending_v_code = st.pending_voucher_code.clone();
                let pending_v_paise = st.pending_voucher_paise;
                if (pending_pts > 0 || pending_wallet > 0 || pending_v_paise > 0)
                    && st.selected_customer_id.is_none()
                {
                    return Err("Pick a customer to redeem loyalty, wallet or voucher".into());
                }
                if !is_client
                    && (pending_pts > 0 || pending_wallet > 0 || pending_v_paise > 0)
                {
                    let cid = st.selected_customer_id.clone().unwrap();
                    let actor = st.actor()?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    if pending_v_paise > 0 && !pending_v_code.is_empty() {
                        let bal = store.voucher_balance(&pending_v_code)?;
                        if bal < pending_v_paise {
                            return Err("Voucher balance changed — clear redeem and try again".into());
                        }
                    }
                    if pending_pts > 0 {
                        store.redeem_loyalty(&cid, pending_pts, &actor)?;
                    }
                    if pending_wallet > 0 {
                        store.wallet_spend(&cid, pending_wallet, &actor)?;
                    }
                    if pending_v_paise > 0 && !pending_v_code.is_empty() {
                        store.redeem_voucher(&pending_v_code, pending_v_paise, &actor)?;
                    }
                }
                let customer_id = if let Some(id) = st.selected_customer_id.clone() {
                    Some(id)
                } else if credit > 0 {
                    if typed_customer.trim().is_empty() {
                        return Err(
                            "Credit part needs a customer — type a name or Pick one".into(),
                        );
                    } else if is_client {
                        None
                    } else {
                        let name = typed_customer.trim();
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        let customer = store.find_or_create_customer(name)?;
                        st.selected_customer_id = Some(customer.id.clone());
                        Some(customer.id)
                    }
                } else {
                    None
                };
                let customer_name = {
                    let typed = typed_customer.trim();
                    if !typed.is_empty() {
                        Some(typed.to_string())
                    } else {
                        st.selected_customer_id.as_ref().and_then(|id| {
                            st.customers
                                .iter()
                                .find(|c| &c.id == id)
                                .map(|c| c.name.clone())
                        })
                    }
                };
                let rail = st.fiscal.digital_pay_label().to_string();
                let cash_label = st.t("cash");
                let card_label = st.t("card");
                let credit_label = st.t("pay_credit");
                let summary = parts
                    .iter()
                    .map(|(m, a)| {
                        let label = match m {
                            PayMethod::Upi => rail.as_str(),
                            PayMethod::Cash => &cash_label,
                            PayMethod::Card => &card_label,
                            PayMethod::Credit => &credit_label,
                            _ => m.label(),
                        };
                        format!("{} {}", label, st.money(Money(*a)))
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                let table_id = st.selected_table_id.clone();
                let pay_label = if parts.len() == 1 {
                    parts[0].0.as_str().to_string()
                } else {
                    PayMethod::Split.as_str().to_string()
                };
                if is_client {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    if !store.client_paired()? {
                        return Err("Client is not paired to a Server — connect first".into());
                    }
                    let table_name = table_id.as_ref().and_then(|tid| {
                        st.tables.iter().find(|t| &t.id == tid).map(|t| t.name.clone())
                    });
                    let amount = st.money(Money(draft.total_paise));
                    let req = net::RemoteSaleRequest {
                        draft,
                        pay: pay_label,
                        customer_id,
                        customer_name,
                        table_name: table_name.clone(),
                        table_id,
                        payments: parts
                            .iter()
                            .map(|(method, amount)| net::RemoteTender {
                                pay: method.as_str().into(),
                                amount_paise: *amount,
                            })
                            .collect(),
                        tendered_paise: tendered,
                        tip_paise: tip,
                    };
                    // Saved on this device first, then sent: a Wi‑Fi drop
                    // delays the bill, it never loses it.
                    let place = table_name.unwrap_or_else(|| "counter".into());
                    st.lan_job(
                        "/sale",
                        &req,
                        format!("Payment {amount} · {place}"),
                        format!("{amount} · {summary}"),
                        phone::Follow::Sale,
                        true,
                    )?;
                    st.cart.clear();
                    st.open_cart_line = None;
                    st.selected_table_id = None;
                    st.status = format!("{amount} · {summary} saved — sending to the counter");
                    return Ok(Pending::Queued);
                }
                let sale = {
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    store.finalize_sale_tenders(
                        &draft,
                        &parts,
                        table_id.as_deref(),
                        customer_id.as_deref(),
                        &actor,
                    )?
                };
                if tendered > 0 || tip > 0 {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let change = (tendered - cash - tip).max(0);
                    store.record_settlement(&sale.id, tendered, change, tip, "cash", &actor)?;
                }
                let amount = st.money(Money(sale.total_paise));
                let full = st
                    .store
                    .as_ref()
                    .and_then(|s| s.format_receipt_text(&sale.id).ok())
                    .unwrap_or_else(|| format!("Sale {}", amount));
                st.cart.clear();
                st.open_cart_line = None;
                st.selected_table_id = None;
                st.pending_loyalty_points = 0;
                st.pending_wallet_paise = 0;
                st.pending_voucher_code.clear();
                st.pending_voucher_paise = 0;
                st.last_receipt = full;
                st.status = format!("Billed {} · {summary}", sale.invoice_no);
                let snack = format!("{amount} · {summary}");
                st.reload_all()?;
                Ok(Pending::Local {
                    sale_id: sale.id,
                    snack,
                })
            })();
            let result = match pending {
                Err(e) => Err(e),
                Ok(Pending::Closed { msg }) => Err(msg),
                Ok(Pending::Local { sale_id, snack }) => Ok(Some((sale_id, snack))),
                Ok(Pending::Queued) => Ok(None),
            };
            self.end_pay();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(done) => {
                    for id in [
                        ids!(mix_cash_input),
                        ids!(mix_upi_input),
                        ids!(mix_card_input),
                        ids!(mix_credit_input),
                        ids!(tendered_input),
                        ids!(tip_input),
                    ] {
                        self.ui.text_input(cx, id).set_text(cx, "");
                    }
                    match done {
                        Some((sale_id, snack)) => self.show_undo_snackbar(cx, sale_id, snack),
                        // Queued on this device: say so where the seller looks.
                        None => {
                            let msg = app_state().lock().unwrap().status.clone();
                            self.set_status(cx, msg);
                        }
                    }
                }
            }
            self.sync_chrome(cx);
        }
        for id in [
            ids!(snack_dismiss_info),
            ids!(snack_dismiss_ok),
            ids!(snack_dismiss_warn),
            ids!(snack_dismiss_err),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                self.clear_snack(cx);
            }
        }
        if self.ui.button(cx, ids!(undo_snack_btn)).clicked(actions) {
            self.undo_last_sale(cx);
        }
        if self.ui.button(cx, ids!(undo_snack_dismiss)).clicked(actions) {
            self.clear_undo_snackbar(cx);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(customer_use)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(customer_name_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                // The Server owns the customer book; a Client picks from what
                // sync brought down so both ends agree on the id.
                if st.device_role == DeviceRole::Client {
                    let needle = name.trim().to_lowercase();
                    let customer = st
                        .customers
                        .iter()
                        .find(|c| c.name.to_lowercase() == needle)
                        .cloned()
                        .ok_or_else(|| {
                            "No such customer here — add them on the Server, then Sync".to_string()
                        })?;
                    st.selected_customer_id = Some(customer.id);
                    st.status = format!("Credit customer: {}", customer.name);
                    return Ok(());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let customer = store.find_or_create_customer(&name)?;
                st.selected_customer_id = Some(customer.id.clone());
                st.status = format!("Credit customer: {}", customer.name);
                st.reload_all()?;
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(customer_clear)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.selected_customer_id = None;
                st.pending_loyalty_points = 0;
                st.pending_wallet_paise = 0;
                st.pending_voucher_code.clear();
                st.pending_voucher_paise = 0;
                apply_pending_till_discount(&mut st);
            }
            self.sync_chrome(cx);
        }
        for ((chip, chip_on), kind) in [
            (
                (ids!(credit_kind_customer), ids!(credit_kind_customer_on)),
                "Customer",
            ),
            (
                (ids!(credit_kind_both), ids!(credit_kind_both_on)),
                "Both",
            ),
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
                        "Customer" => locale::t(&lang, "role_help_customer"),
                        "Both" => locale::t(&lang, "role_help_both"),
                        _ => locale::t(&lang, "role_help_supplier"),
                    },
                );
            }
        }
        if self.ui.button(cx, ids!(credit_add_btn)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(credit_add_name)).text();
            let phone_raw = self.ui.text_input(cx, ids!(credit_add_phone)).text();
            let kind = self.ui.text_input(cx, ids!(credit_add_kind)).text();
            let birthday = self.ui.text_input(cx, ids!(credit_add_birthday)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let phone = with_calling_code(&phone_raw, &st.fiscal.country_code);
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                if let Some(id) = st.selected_customer_id.clone() {
                    st.require_server_role("Edit party")?;
                    store.update_customer_details(&id, &name, &phone, Some(&birthday))?;
                    store.set_customer_kind_label(&id, kind.trim())?;
                    st.reload_all()?;
                    Ok(format!("Party updated: {}", name.trim()))
                } else {
                    st.require_server_role("Add customers")?;
                    let customer = store.add_customer_kind(&name, &phone, kind.trim())?;
                    if !birthday.trim().is_empty() {
                        store.update_customer_details(
                            &customer.id,
                            &customer.name,
                            &customer.phone,
                            Some(&birthday),
                        )?;
                    }
                    st.selected_customer_id = Some(customer.id);
                    st.reload_all()?;
                    Ok(format!(
                        "Added {} — now editing it; tap New party to add another",
                        name.trim()
                    ))
                }
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_form_clear)).clicked(actions) {
            app_state().lock().unwrap().selected_customer_id = None;
            self.ui
                .text_input(cx, ids!(credit_add_name))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(credit_add_phone))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(credit_add_birthday))
                .set_text(cx, "");
            self.ui
                .text_input(cx, ids!(credit_add_kind))
                .set_text(cx, "");
            self.set_status(cx, "New party — form cleared");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(party_delete)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Delete party")?;
                let id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let msg = store.delete_customer(&id)?;
                st.selected_customer_id = None;
                st.reload_all()?;
                Ok(msg)
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(credit_add_name)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(credit_add_phone)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(credit_add_kind)).set_text(cx, "");
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
                st.require_server_role("Set credit limit")?;
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
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let customer = store.find_or_create_customer(party_name.trim())?;
                store.set_customer_credit_limit(&customer.id, limit_paise)?;
                st.reload_all()?;
                Ok(format!(
                    "Credit limit for {} set to {}",
                    customer.name,
                    st.money(Money(limit_paise))
                ))
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
        if self.ui.button(cx, ids!(credit_writeoff)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(writeoff_amount_input)).text();
            let reason = self.ui.text_input(cx, ids!(writeoff_reason_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Write off credit")?;
                let actor = st.actor()?;
                let customer_id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let currency = st.locale.currency.clone();
                let amount = match amount_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let written = st
                    .store
                    .as_mut()
                    .ok_or_else(|| "No store".to_string())?
                    .write_off_credit(&customer_id, amount, &reason, &actor)?;
                st.reload_all()?;
                let _ = st.refresh_books_overview();
                Ok(format!("Wrote off {}", st.money(Money(written))))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(writeoff_amount_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(writeoff_reason_input)).set_text(cx, "");
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
                    st.require_server_role("Settle payable")?;
                    let actor = st.actor()?;
                    let party_id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a party first".to_string())?;
                    let currency = st.locale.currency.clone();
                    let amount = locale::parse_major_input(&amount_text, &currency)?;
                    // Optional bill-wise allocation (bill # from the list).
                    let against = match bill_text.trim() {
                        "" => None,
                        other => {
                            let n: usize = other
                                .parse()
                                .map_err(|_| "Bill # must be a number from the list".to_string())?;
                            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            let bills = store.open_payable_bills(&party_id)?;
                            Some(
                                bills
                                    .get(n.wrapping_sub(1))
                                    .map(|b| b.bill_id.clone())
                                    .ok_or_else(|| "No such open bill".to_string())?,
                            )
                        }
                    };
                    st.store
                        .as_mut()
                        .ok_or_else(|| "No store".to_string())?
                        .settle_payable_against(
                            &party_id,
                            amount,
                            method,
                            &actor,
                            "",
                            against.as_deref(),
                        )?;
                    st.reload_all()?;
                    let _ = st.refresh_books_overview();
                    Ok(format!("AP paid {}", st.money(Money(amount))))
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
                st.require_server_role("Write off payable")?;
                let actor = st.actor()?;
                let party_id = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a party first".to_string())?;
                let currency = st.locale.currency.clone();
                let amount = match amount_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let written = st
                    .store
                    .as_mut()
                    .ok_or_else(|| "No store".to_string())?
                    .write_off_payable(&party_id, amount, &reason, &actor)?;
                st.reload_all()?;
                let _ = st.refresh_books_overview();
                Ok(format!("AP wrote off {}", st.money(Money(written))))
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
                            st.status = format!("Credit customer: {}", c.name);
                        }
                        picked
                    };
                    if let Some(c) = picked {
                        self.ui
                            .text_input(cx, ids!(credit_add_name))
                            .set_text(cx, &c.name);
                        self.ui
                            .text_input(cx, ids!(credit_add_phone))
                            .set_text(cx, &c.phone);
                        self.ui
                            .text_input(cx, ids!(credit_add_birthday))
                            .set_text(cx, &c.birthday);
                        self.ui
                            .text_input(cx, ids!(credit_add_kind))
                            .set_text(cx, &c.kind_label);
                    }
                    self.sync_chrome(cx);
                }
            }
        }

        for (btn, seg) in [
            (ids!(crm_seg_bday_week), store::CrmSegment::BirthdayThisWeek),
            (ids!(crm_seg_bday_month), store::CrmSegment::BirthdayThisMonth),
            (ids!(crm_seg_phone), store::CrmSegment::WithPhone),
            (ids!(crm_seg_loyalty), store::CrmSegment::WithLoyalty),
            (ids!(crm_seg_inactive), store::CrmSegment::InactiveDays(30)),
            (ids!(crm_seg_top), store::CrmSegment::TopSpenders(20)),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                app_state().lock().unwrap().crm_segment = seg;
                self.sync_chrome(cx);
            }
        }
        for (btn, channel) in [
            (ids!(crm_queue_sms), "sms"),
            (ids!(crm_queue_wa), "whatsapp"),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let body = self.ui.text_input(cx, ids!(crm_message_input)).text();
                let result = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    st.require_server_role("CRM campaign")?;
                    let actor = st.actor()?;
                    let seg = st.crm_segment;
                    let n = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .queue_segment_campaign(seg, &body, channel, &actor)?;
                    Ok(format!("Queued {n} {channel} message(s) for {}", seg.label()))
                })();
                match result {
                    Ok(msg) => self.set_status(cx, msg),
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }

        if self.ui.button(cx, ids!(till_apply_redeem)).clicked(actions) {
            let pts_text = self.ui.text_input(cx, ids!(till_loyalty_input)).text();
            let wallet_text = self.ui.text_input(cx, ids!(till_wallet_input)).text();
            let voucher_code = self.ui.text_input(cx, ids!(till_voucher_input)).text();
            let voucher_amt = self.ui.text_input(cx, ids!(till_voucher_amt_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let cid = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a customer first".to_string())?;
                let currency = st.locale.currency.clone();
                let guest = st
                    .customers
                    .iter()
                    .find(|c| c.id == cid)
                    .cloned()
                    .ok_or_else(|| "Customer not found".to_string())?;
                let pts = if pts_text.trim().is_empty() {
                    0
                } else {
                    pts_text
                        .trim()
                        .parse::<i64>()
                        .map_err(|_| "Points must be a whole number".to_string())?
                };
                if pts < 0 {
                    return Err("Points cannot be negative".into());
                }
                if pts > guest.loyalty_points {
                    return Err(format!("Only {} points available", guest.loyalty_points));
                }
                let wallet = if wallet_text.trim().is_empty() {
                    0
                } else {
                    locale::parse_major_input(wallet_text.trim(), &currency)?
                };
                if wallet > guest.wallet_paise {
                    return Err(format!(
                        "Wallet only has {}",
                        st.money(Money(guest.wallet_paise))
                    ));
                }
                let (v_code, v_paise) = if voucher_code.trim().is_empty() {
                    (String::new(), 0)
                } else {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let bal = store.voucher_balance(voucher_code.trim())?;
                    if bal <= 0 {
                        return Err("Nothing left on this voucher".into());
                    }
                    let want = if voucher_amt.trim().is_empty() {
                        bal
                    } else {
                        locale::parse_major_input(voucher_amt.trim(), &currency)?.min(bal)
                    };
                    if want <= 0 {
                        return Err("Voucher amount must be above zero".into());
                    }
                    (voucher_code.trim().to_uppercase(), want)
                };
                st.pending_loyalty_points = pts;
                st.pending_wallet_paise = wallet;
                st.pending_voucher_code = v_code;
                st.pending_voucher_paise = v_paise;
                apply_pending_till_discount(&mut st);
                let pending = pending_till_redeem_paise(&st);
                Ok(if pending > 0 || !st.pending_voucher_code.is_empty() {
                    format!(
                        "Redeem ready {}{}",
                        st.money(Money(pending.max(0))),
                        if st.pending_voucher_code.is_empty() {
                            String::new()
                        } else {
                            format!(" · voucher {}", st.pending_voucher_code)
                        }
                    )
                } else {
                    "Nothing to redeem".into()
                })
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(till_clear_redeem)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.pending_loyalty_points = 0;
                st.pending_wallet_paise = 0;
                st.pending_voucher_code.clear();
                st.pending_voucher_paise = 0;
                apply_pending_till_discount(&mut st);
            }
            for id in [
                ids!(till_loyalty_input),
                ids!(till_wallet_input),
                ids!(till_voucher_input),
                ids!(till_voucher_amt_input),
            ] {
                self.ui.text_input(cx, id).set_text(cx, "");
            }
            self.set_status(cx, "Till redeem cleared");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(till_wallet_topup)).clicked(actions) {
            let amt = self.ui.text_input(cx, ids!(till_wallet_topup_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Wallet top-up")?;
                let actor = st.actor()?;
                let cid = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a customer first".to_string())?;
                let paise = locale::parse_major_input(amt.trim(), &st.locale.currency)?;
                let bal = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .wallet_top_up(&cid, paise, &actor)?;
                st.reload_all()?;
                Ok(format!("Wallet now {}", st.money(Money(bal))))
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(till_wallet_topup_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(till_issue_voucher)).clicked(actions) {
            let code = self.ui.text_input(cx, ids!(till_voucher_issue_code)).text();
            let amt = self.ui.text_input(cx, ids!(till_voucher_issue_amt_input)).text();
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Issue voucher")?;
                let actor = st.actor()?;
                let paise = locale::parse_major_input(amt.trim(), &st.locale.currency)?;
                let v = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .issue_voucher(code.trim(), paise, &actor)?;
                Ok(format!(
                    "Voucher {} issued for {}",
                    v.code,
                    st.money(Money(v.amount_paise))
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(till_voucher_issue_code))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(till_voucher_issue_amt_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }

        for (btn, method) in [
            (ids!(settle_cash), PayMethod::Cash),
            (ids!(settle_upi), PayMethod::Upi),
            (ids!(settle_card), PayMethod::Card),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(settle_amount_input)).text();
                let bill_text = self.ui.text_input(cx, ids!(settle_bill_input)).text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    let customer_id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a customer first".to_string())?;
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&amount_text, &currency)?;
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
                            detail: String::new(),
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
                        st.status = format!("{who}: {amount} saved — sending to the counter");
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
                        "",
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
                    // Surface settle/advance result on Sell receipt line too.
                    let msg = app_state().lock().unwrap().status.clone();
                    self.set_status(cx, msg);
                }
                self.sync_chrome(cx);
            }
        }

        // "Next order (clear desk)" under Send KOT is the one way to clear.
        if self.ui.button(cx, ids!(next_order_btn)).clicked(actions)
        {
            {
                let mut st = app_state().lock().unwrap();
                st.clear_order_desk();
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(add_items_btn)).clicked(actions) {
            // Same as menu_open_btn — overlay, not a Menu peer push onto history.
            self.open_menu_window(cx);
        }
        // Order type chips on the desk.
        for ((chip, chip_on), kind) in [
            ((ids!(otype_counter), ids!(otype_counter_on)), OrderType::Counter),
            ((ids!(otype_dinein), ids!(otype_dinein_on)), OrderType::DineIn),
            ((ids!(otype_takeaway), ids!(otype_takeaway_on)), OrderType::Takeaway),
            ((ids!(otype_delivery), ids!(otype_delivery_on)), OrderType::Delivery),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                // Guard must drop before sync_chrome: it locks app_state()
                // itself, and std Mutex is non-reentrant — calling it with
                // `st` alive deadlocks the UI thread (found 2026-10-22:
                // order-type chip tap froze the app after unlock).
                let seat_now = {
                    let mut st = app_state().lock().unwrap();
                    st.cart.set_order_type(kind);
                    if kind != OrderType::DineIn {
                        st.selected_table_id = None;
                    }
                    // Dine-in without a table: go and pick one now rather
                    // than say where to go.
                    kind == OrderType::DineIn && st.selected_table_id.is_none()
                };
                if seat_now {
                    self.pick_table_for_order(cx);
                } else {
                    self.sync_chrome(cx);
                }
            }
        }
        for ((chip, chip_on), c) in [
            ((ids!(pm_cash), ids!(pm_cash_on)), PayChoice::Cash),
            ((ids!(pm_upi), ids!(pm_upi_on)), PayChoice::Upi),
            ((ids!(pm_card), ids!(pm_card_on)), PayChoice::Card),
            ((ids!(pm_credit), ids!(pm_credit_on)), PayChoice::Credit),
            ((ids!(pm_split), ids!(pm_split_on)), PayChoice::Split),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                app_state().lock().unwrap().pay_choice = c;
                self.sync_chrome(cx);
            }
        }
        for ((chip, chip_on), resale) in [
            ((ids!(menu_kind_dish), ids!(menu_kind_dish_on)), false),
            ((ids!(menu_kind_resale), ids!(menu_kind_resale_on)), true),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                app_state().lock().unwrap().form_resale = resale;
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(template_import)).clicked(actions) {
            app_state().lock().unwrap().template_confirm = true;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(template_no)).clicked(actions) {
            app_state().lock().unwrap().template_confirm = false;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(template_yes)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.template_confirm = false;
                st.require_server_role("Import the menu template")?;
                let actor = st.actor()?;
                if !actor.role.can_manage_menu() {
                    return Err("Not allowed to manage items".into());
                }
                let done = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .import_catalogue(menu_images::shelf())?;
                st.reload_all()?;
                Ok(st
                    .t("template_imported")
                    .replace("{d}", &done.items_added.to_string())
                    .replace("{c}", &done.categories_added.to_string()))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(table_pick_btn)).clicked(actions) {
            self.pick_table_for_order(cx);
        }
        if self.ui.button(cx, ids!(floor_pick_back)).clicked(actions) {
            self.cancel_table_pick(cx);
        }
        // Category chips: All, Frequent, then the shop's categories.
        {
            let mut picked: Option<Option<String>> = None;
            if self.choice_chip_clicked(cx, actions, ids!(cat_all), ids!(cat_all_on)) {
                picked = Some(None);
            }
            if self.choice_chip_clicked(cx, actions, ids!(cat_frequent), ids!(cat_frequent_on)) {
                picked = Some(Some(MENU_FREQUENT.to_string()));
            }
            let cats = app_state().lock().unwrap().menu_categories();
            for (i, (idle, on)) in CATEGORY_CHIPS.iter().enumerate() {
                if self.choice_chip_clicked(cx, actions, idle, on)
                    && let Some(c) = cats.get(i)
                {
                    picked = Some(Some(c.id.clone()));
                }
            }
            if let Some(cat) = picked {
                let fill_frequent = {
                    let mut st = app_state().lock().unwrap();
                    st.menu_category = cat;
                    if st.menu_category.as_deref() == Some(MENU_FREQUENT) {
                        st.refresh_frequent_cache();
                        Some((st.frequent_days, st.frequent_limit))
                    } else {
                        None
                    }
                };
                if let Some((days, limit)) = fill_frequent {
                    self.ui
                        .text_input(cx, ids!(frequent_days_input))
                        .set_text(cx, &days.to_string());
                    self.ui
                        .text_input(cx, ids!(frequent_limit_input))
                        .set_text(cx, &limit.to_string());
                }
                {
                    let mut st = app_state().lock().unwrap();
                    st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
                }
                self.sync_chrome(cx);
            }
        }

        if self.ui.button(cx, ids!(frequent_config_save)).clicked(actions) {
            let days_s = self.ui.text_input(cx, ids!(frequent_days_input)).text();
            let limit_s = self.ui.text_input(cx, ids!(frequent_limit_input)).text();
            let result = (|| -> Result<(String, u32, u32), String> {
                let days: u32 = days_s
                    .trim()
                    .parse()
                    .map_err(|_| "Days must be a number (1–90)".to_string())?;
                let limit: u32 = limit_s
                    .trim()
                    .parse()
                    .map_err(|_| "Items must be a number (1–100)".to_string())?;
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Frequent items settings")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_frequent_config(days, limit, &actor)?;
                st.refresh_frequent_cache();
                st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
                let count = st.frequent_ids.len();
                Ok((
                    st.t("frequent_saved")
                        .replace("{d}", &st.frequent_days.to_string())
                        .replace("{n}", &st.frequent_limit.to_string())
                        .replace("{c}", &count.to_string()),
                    st.frequent_days,
                    st.frequent_limit,
                ))
            })();
            match result {
                Ok((msg, days, limit)) => {
                    self.ui
                        .text_input(cx, ids!(frequent_days_input))
                        .set_text(cx, &days.to_string());
                    self.ui
                        .text_input(cx, ids!(frequent_limit_input))
                        .set_text(cx, &limit.to_string());
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(area_new_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(area_new_input)).text();
            let result = (|| -> Result<String, String> {
                let name = name.trim();
                if name.is_empty() {
                    return Err("Type an area name (e.g. AC)".into());
                }
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add floor area")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.ensure_floor_area(name)?;
                let _ = st.reload_all();
                Ok(st.t("area_added").replace("{name}", name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(area_new_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(discount_toggle)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.discount_panel_open = !st.discount_panel_open;
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(discount_remove)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.cart.discount = None;
                st.discount_panel_open = false;
            }
            self.set_status(cx, "Discount removed");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(discount_apply)).clicked(actions) {
            let amount = self.ui.text_input(cx, ids!(discount_input)).text();
            let reason = self.ui.text_input(cx, ids!(discount_reason_input)).text();
            let pin = self.ui.text_input(cx, ids!(discount_pin_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.cart.is_empty() {
                    return Err("Add items first".into());
                }
                let kind = domain::DiscountKind::parse(&amount, &st.locale.currency)?;
                if reason.trim().is_empty() {
                    return Err("Say why the bill is discounted".into());
                }
                // Quick answer here; the Server checks the right again at
                // payment (a tablet's word is never taken for it).
                if st.device_role.is_host() {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let pin = Some(pin.as_str()).filter(|p| !p.trim().is_empty());
                    store.check_right(&actor, domain::Right::Discount, pin)?;
                }
                st.cart.charge_total = None;
                st.cart.discount = Some(domain::BillDiscount {
                    kind,
                    reason: reason.trim().to_string(),
                    approval_pin: pin.trim().to_string(),
                });
                st.discount_panel_open = false;
                Ok(format!("Discount {} applied", st.money(Money(st.cart.discount_paise()))))
            })();
            match result {
                Ok(msg) => {
                    for id in [ids!(discount_input), ids!(discount_reason_input), ids!(discount_pin_input)] {
                        self.ui.text_input(cx, id).set_text(cx, "");
                    }
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        // Table actions on the ticket (Print bill · Move / merge · line panel).
        if self.ui.button(cx, ids!(print_bill_btn)).clicked(actions) {
            self.print_table_bill(cx);
        }
        if self.ui.button(cx, ids!(table_split_btn)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.split_mode = !st.split_mode;
                st.split_pick.clear();
                st.open_cart_line = None;
                st.move_panel_open = false;
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(split_evenly_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let tid = st
                    .selected_table_id
                    .clone()
                    .ok_or_else(|| "Open a table first".to_string())?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or("No store")?;
                let n = store
                    .running_order(&tid)?
                    .map(|o| o.live_lines().count().max(2) as i64)
                    .unwrap_or(2);
                let companions = store.split_evenly(&tid, n.min(8), &actor)?;
                st.reload_all()?;
                Ok(format!("Split into {} companion bill(s)", companions.len() + 1))
            })();
            match result {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(fire_course_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let tid = st
                    .selected_table_id
                    .clone()
                    .ok_or_else(|| "Open a table first".to_string())?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or("No store")?;
                let ids = store.fire_course(&tid, 1, &actor)?;
                st.reload_all()?;
                Ok(if ids.is_empty() {
                    "Nothing to fire for course 1".into()
                } else {
                    format!("Fired course 1 · {} KOT(s)", ids.len())
                })
            })();
            match result {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(hold_item_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let idx = st
                    .open_cart_line
                    .ok_or_else(|| "Tap a line on the ticket first".to_string())?;
                let line_id = st
                    .cart
                    .lines
                    .get(idx)
                    .map(|l| l.running_line_id.clone())
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| "That line is not on the table order yet — Send KOT first or seat the table".to_string())?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or("No store")?;
                store.set_line_held(&line_id, true, &actor)?;
                st.reload_all()?;
                Ok("Line held — Send KOT skips it until Fire course".into())
            })();
            match result {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(waiter_call_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let tid = st
                    .selected_table_id
                    .clone()
                    .ok_or_else(|| "Open a table first".to_string())?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or("No store")?;
                store.queue_waiter_call(&tid, "service", &actor)?;
                let _ = st.refresh_inventory_text();
                Ok("Waiter call queued — clear from Floor ops".into())
            })();
            match result {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(split_cancel)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.split_mode = false;
                st.split_pick.clear();
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(split_go)).clicked(actions) {
            self.make_split_bill(cx);
        }
        if self.ui.button(cx, ids!(table_move_btn)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.move_panel_open = !st.move_panel_open;
                st.line_action = None;
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(move_table_go)).clicked(actions) {
            self.move_or_merge_table(cx, false);
        }
        if self.ui.button(cx, ids!(merge_table_go)).clicked(actions) {
            self.move_or_merge_table(cx, true);
        }
        if self.ui.button(cx, ids!(line_action_back)).clicked(actions) {
            app_state().lock().unwrap().line_action = None;
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(line_action_go)).clicked(actions) {
            self.confirm_line_action(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(line_cooked_raw),
            ids!(line_cooked_raw_on),
        ) {
            app_state().lock().unwrap().line_cooked = false;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(line_cooked_done),
            ids!(line_cooked_done_on),
        ) {
            app_state().lock().unwrap().line_cooked = true;
            self.sync_chrome(cx);
        }
        // The menu window: Add dishes opens it; Done, Escape, a tap outside
        // or Back closes it (the search starts fresh next time).
        if self.ui.button(cx, ids!(menu_open_btn)).clicked(actions) {
            self.open_menu_window(cx);
        }
        {
            let search = self.ui.text_input(cx, ids!(menu_search_input));
            let done = self.ui.button(cx, ids!(menu_pick_done)).clicked(actions);
            let dismissed = self.ui.modal(cx, ids!(menu_modal)).dismissed(actions);
            if done || dismissed || search.escaped(actions) {
                self.close_menu_window(cx);
            }
        }
        if let Some(text) = self.ui.text_input(cx, ids!(menu_search_input)).changed(actions) {
            {
                let mut st = app_state().lock().unwrap();
                // Typing looks through every category (a short code is often
                // in another one); a tab tapped afterwards narrows the results.
                if st.menu_filter.trim().is_empty() && !text.trim().is_empty() {
                    st.menu_category = None;
                }
                st.menu_filter = text;
                st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
            }
            self.sync_chrome(cx);
        }
        // Customer paid / tip: the change line follows live.
        let tendered_changed = self.ui.text_input(cx, ids!(tendered_input)).changed(actions).is_some();
        let tip_changed = self.ui.text_input(cx, ids!(tip_input)).changed(actions).is_some();
        let cash_changed = self.ui.text_input(cx, ids!(mix_cash_input)).changed(actions).is_some();
        if tendered_changed || tip_changed || cash_changed {
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(kot_set_station)).clicked(actions) {
            let text = self.ui.text_input(cx, ids!(kot_station_input)).text();
            match self.apply_kot_station_to_unassigned(cx, &text) {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(send_kot_btn)).clicked(actions) {
            // Station field filled → apply / change kitchen before send.
            {
                let station_text = self.ui.text_input(cx, ids!(kot_station_input)).text();
                if !station_text.trim().is_empty() {
                    if let Err(e) = self.apply_kot_station_to_unassigned(cx, &station_text) {
                        self.set_status(cx, e);
                        self.sync_chrome(cx);
                        return;
                    }
                }
            }
            // Only the new lines go; what the kitchen already has stays on
            // the table's running order. A waiter's tablet sends to the
            // Server (it used to write the ticket into its own cache, which
            // the next sync erased — the kitchen never saw it).
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                st.require_cart_modifiers()?;
                let draft = st.cart.to_kot_draft()?;
                let table_id = st.selected_table_id.clone();
                let unassigned = st.t("kot_unassigned");
                let dests = pending_kot_destinations(
                    &st.products,
                    &st.stations,
                    &st.cart.lines,
                    &unassigned,
                );
                let dest_line = format_kot_destinations(&dests);
                let kitchens = if dest_line.is_empty() {
                    st.t("kitchen")
                } else {
                    dest_line.clone()
                };
                let msg = if st.device_role == DeviceRole::Client {
                    let table_name = table_id.as_ref().and_then(|tid| {
                        st.tables.iter().find(|t| &t.id == tid).map(|t| t.name.clone())
                    });
                    let place = table_name.clone().unwrap_or_else(|| "counter".into());
                    // Saved on this device first: a Wi‑Fi drop delays the
                    // kitchen's paper, it never loses the order.
                    let job = st.lan_job(
                        "/kot",
                        &net::RemoteKotRequest { draft, table_name, table_id: table_id.clone() },
                        format!("KOT for {place} → {kitchens}"),
                        format!("KOT for {place} reached {kitchens}"),
                        phone::Follow::Refresh,
                        true,
                    )?;
                    if table_id.is_some() {
                        // On the table as "waiting to send" until the Server has it.
                        for line in st.cart.lines.iter_mut().filter(|l| !l.is_sent()) {
                            line.running_line_id = format!("{QUEUED_LINE}{job}");
                            line.kot_id = QUEUED_KOT.into();
                        }
                    } else {
                        st.cart.mark_new_lines_sent("counter");
                    }
                    return Ok(format!(
                        "KOT for {place} → {kitchens} saved — sending"
                    ));
                } else {
                    let kot_ids = {
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.send_kot(&draft, table_id.as_deref(), &actor)?
                    };
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let n = kot_ids.len();
                    match (store.kot_auto_print(), net::print_sent_kots(store, &kot_ids)) {
                        (true, None) => format!("{n} KOT(s) → {kitchens} · printed"),
                        (true, Some(e)) => format!("KOT → {kitchens} · print failed: {e}"),
                        (false, _) => format!("{n} KOT(s) → {kitchens}"),
                    }
                };
                match table_id {
                    // The table's order now holds them: show them as sent.
                    Some(tid) => {
                        st.reload_all()?;
                        st.load_table_cart(&tid)?;
                    }
                    // Counter order paid after: keep the ticket to charge,
                    // but the kitchen must not be told twice at payment.
                    None => st.cart.mark_new_lines_sent("counter"),
                }
                st.open_kots = st
                    .store
                    .as_ref()
                    .map(|s| s.list_open_kots().unwrap_or_default())
                    .unwrap_or_default();
                Ok(msg)
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        // Drawn layout (hidden): select a table for the editor; act button same as grid.
        {
            let layout_on = app_state().lock().unwrap().floor_view_layout;
            if layout_on {
                let widget = self.ui.widget(cx, ids!(floor_layout_list));
                let canvas = widget.view(cx, ids!(canvas));
                let slots = [
                    ids!(t0), ids!(t1), ids!(t2), ids!(t3), ids!(t4), ids!(t5),
                    ids!(t6), ids!(t7), ids!(t8), ids!(t9), ids!(t10), ids!(t11),
                    ids!(t12), ids!(t13), ids!(t14), ids!(t15), ids!(t16), ids!(t17),
                    ids!(t18), ids!(t19), ids!(t20), ids!(t21), ids!(t22), ids!(t23),
                ];
                let filtered_ids: Vec<String> = {
                    let st = app_state().lock().unwrap();
                    floor_filtered_tables(&st)
                        .into_iter()
                        .take(FLOOR_LAYOUT_SLOTS)
                        .map(|t| t.id)
                        .collect()
                };
                let mut tapped: Option<(usize, bool)> = None;
                for (slot, path) in slots.iter().enumerate() {
                    let tile = canvas.view(cx, *path);
                    if tile.button(cx, ids!(act)).clicked(actions) {
                        tapped = Some((slot, true));
                    } else if tile.finger_up(actions).is_some_and(|e| e.was_tap() && e.is_over) {
                        tapped = Some((slot, false));
                    }
                }
                if let Some((slot, action)) = tapped {
                    if let Some(tid) = filtered_ids.get(slot).cloned() {
                        let idx = {
                            let st = app_state().lock().unwrap();
                            st.tables.iter().position(|t| t.id == tid)
                        };
                        if let Some(idx) = idx {
                            if action {
                                let info = app_state()
                                    .lock()
                                    .unwrap()
                                    .tables
                                    .get(idx)
                                    .map(|t| (t.floor_state, t.running_total_paise));
                                match info {
                                    Some((domain::FloorState::Free, _)) => {
                                        self.seat_table(cx, idx)
                                    }
                                    Some((domain::FloorState::Seated, _))
                                    | Some((domain::FloorState::Paid, _)) => {
                                        self.mark_table_ready(cx, idx)
                                    }
                                    Some((domain::FloorState::BillPrinted, amount))
                                        if amount <= 0 =>
                                    {
                                        self.mark_table_ready(cx, idx)
                                    }
                                    Some((domain::FloorState::BillPrinted, _)) => {
                                        self.undo_table_bill(cx, idx)
                                    }
                                    Some(_) => self.request_table_bill(cx, idx),
                                    None => {}
                                }
                            } else if app_state().lock().unwrap().picking_table {
                                self.open_table(cx, idx);
                            } else {
                                self.select_floor_layout_table(cx, &tid);
                            }
                        }
                    }
                }
            }
        }
        // Layout editor: save / nudge / open.
        if self.ui.button(cx, ids!(layout_save)).clicked(actions) {
            self.save_floor_layout_fields(cx);
        }
        if self.ui.button(cx, ids!(layout_open)).clicked(actions) {
            let idx = {
                let st = app_state().lock().unwrap();
                st.floor_layout_sel
                    .as_ref()
                    .and_then(|id| st.tables.iter().position(|t| &t.id == id))
            };
            if let Some(idx) = idx {
                self.open_table(cx, idx);
            }
        }
        for (btn, dx, dy) in [
            (ids!(layout_nudge_left), -20.0, 0.0),
            (ids!(layout_nudge_right), 20.0, 0.0),
            (ids!(layout_nudge_up), 0.0, -20.0),
            (ids!(layout_nudge_down), 0.0, 20.0),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                self.nudge_floor_layout(cx, dx, dy);
            }
        }
        // Floor grid: tap a tile to open (or seat) that table; its small
        // action asks for the bill (running) or marks it ready (paid).
        if !app_state().lock().unwrap().floor_view_layout {
            let widget = self.ui.widget(cx, ids!(floor_list));
            let list = widget.portal_list(cx, ids!(list));
            let (rows, filtered_ids) = {
                let st = app_state().lock().unwrap();
                let tables = floor_filtered_tables(&st);
                let ids: Vec<String> = tables.iter().map(|t| t.id.clone()).collect();
                (floor_rows(&tables, floor_columns()), ids)
            };
            let slots = [
                ids!(t0), ids!(t1), ids!(t2), ids!(t3), ids!(t4), ids!(t5), ids!(t6), ids!(t7),
            ];
            let mut tapped: Option<(usize, bool)> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                let Some(FloorRow::Tiles(tables)) = rows.get(item_id) else {
                    continue;
                };
                for (slot, &table_idx) in tables.iter().enumerate() {
                    let tile = item.view(cx, slots[slot]);
                    // The action sits inside the tile: check it first so the
                    // tile tap behind it does not also open the order.
                    if tile.button(cx, ids!(act)).clicked(actions) {
                        tapped = Some((table_idx, true));
                    } else if tile.finger_up(actions).is_some_and(|e| e.was_tap() && e.is_over) {
                        tapped = Some((table_idx, false));
                    }
                }
            }
            if let Some((filt_idx, action)) = tapped {
                let idx = {
                    let st = app_state().lock().unwrap();
                    filtered_ids
                        .get(filt_idx)
                        .and_then(|id| st.tables.iter().position(|t| &t.id == id))
                };
                if let Some(idx) = idx {
                    let info = app_state()
                        .lock()
                        .unwrap()
                        .tables
                        .get(idx)
                        .map(|t| (t.floor_state, t.running_total_paise));
                    match (action, info) {
                        (true, Some((domain::FloorState::Free, _))) => {
                            self.seat_table(cx, idx)
                        }
                        (true, Some((domain::FloorState::Seated, _))) => {
                            self.mark_table_ready(cx, idx)
                        }
                        (true, Some((domain::FloorState::Paid, _))) => {
                            self.mark_table_ready(cx, idx)
                        }
                        (true, Some((domain::FloorState::BillPrinted, amount))) if amount <= 0 => {
                            self.mark_table_ready(cx, idx)
                        }
                        (true, Some((domain::FloorState::BillPrinted, _))) => {
                            self.undo_table_bill(cx, idx)
                        }
                        (true, Some(_)) => self.request_table_bill(cx, idx),
                        (false, Some(_)) => self.open_table(cx, idx),
                        _ => {}
                    }
                }
            }
        }
        // Kitchen: advance a ticket New → Preparing → Ready → Done.
        {
            let widget = self.ui.widget(cx, ids!(kitchen_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                let tapped = item.button(cx, ids!(kotcard.advance)).clicked(actions)
                    || item
                        .view(cx, ids!(kotcard))
                        .finger_up(actions)
                        .is_some_and(|e| e.was_tap() && e.is_over);
                if tapped {
                    let result = (|| -> Result<(), String> {
                        let mut st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        let kot_id = st
                            .open_kots
                            .get(item_id)
                            .map(|k| k.id.clone())
                            .ok_or_else(|| "Ticket gone".to_string())?;
                        // A kitchen tablet moves the Server's ticket (writing
                        // its own cache was undone by the next sync).
                        if st.device_role == DeviceRole::Client {
                            st.lan_job(
                                "/kot_advance",
                                &net::RemoteKotAdvance { kot_id },
                                "Kitchen ticket".into(),
                                "Ticket moved on".into(),
                                phone::Follow::Refresh,
                                false,
                            )?;
                        } else {
                            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            store.advance_kot(&kot_id, &actor)?;
                        }
                        // The ticket moving changes the table's colour.
                        let (kots, tables) = st
                            .store
                            .as_ref()
                            .map(|s| (s.list_open_kots().unwrap_or_default(), s.list_tables().unwrap_or_default()))
                            .unwrap_or_default();
                        st.open_kots = kots;
                        st.tables = tables;
                        Ok(())
                    })();
                    if let Err(e) = result {
                        self.set_status(cx, e);
                    }
                    self.sync_chrome(cx);
                }
            }
        }
        if self.ui.button(cx, ids!(table_range_add)).clicked(actions) {
            let range = self.ui.text_input(cx, ids!(table_range_input)).text();
            let area = self.ui.text_input(cx, ids!(table_area_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add tables")?;
                let (prefix, from, to) = parse_table_range(&range)?;
                let added = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_tables_range(&prefix, from, to, &area)?
                };
                st.reload_all()?;
                Ok(format!("{added} table(s) added ({prefix}{from}–{prefix}{to})"))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(table_range_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(table_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(table_name_input)).text();
            let area = self.ui.text_input(cx, ids!(table_area_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add tables")?;
                let table = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_table_in_area(&name, &area)?
                };
                st.tables = st
                    .store
                    .as_ref()
                    .map(|s| s.list_tables().unwrap_or_default())
                    .unwrap_or_default();
                Ok(format!("Table {} added", table.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(table_name_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(station_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(station_name_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add kitchen stations")?;
                let station = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_station(&name)?
                };
                st.stations = st
                    .store
                    .as_ref()
                    .map(|s| s.list_stations().unwrap_or_default())
                    .unwrap_or_default();
                // Own empty KOT route for this station — never inherit another kitchen's printers.
                st.printer_draft.ensure_station_routes([&station.id]);
                if let Some(base) = st.data_base.as_ref() {
                    let _ = print_share::save_device_printers(base, &st.printer_draft);
                }
                Ok(format!(
                    "Station {} added — set its printers in Settings → Where things print",
                    station.name
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(station_name_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(station_remove)).clicked(actions) {
            let typed = self.ui.text_input(cx, ids!(station_remove_input)).text();
            match self.remove_kitchen_station(cx, &typed) {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(station_remove_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(settings_station_add)).clicked(actions) {
            let name = self
                .ui
                .text_input(cx, ids!(settings_station_name_input))
                .text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add kitchen stations")?;
                let station = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_station(&name)?
                };
                st.stations = st
                    .store
                    .as_ref()
                    .map(|s| s.list_stations().unwrap_or_default())
                    .unwrap_or_default();
                st.printer_draft.ensure_station_routes([&station.id]);
                if let Some(base) = st.data_base.as_ref() {
                    let _ = print_share::save_device_printers(base, &st.printer_draft);
                }
                Ok(format!(
                    "Station {} added — set its printers in Settings → Kitchen printing",
                    station.name
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(settings_station_name_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(settings_station_remove)).clicked(actions) {
            let typed = self
                .ui
                .text_input(cx, ids!(settings_station_remove_input))
                .text();
            match self.remove_kitchen_station(cx, &typed) {
                Ok(msg) => {
                    self.ui
                        .text_input(cx, ids!(settings_station_remove_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        // Settings hub: one section at a time.
        {
            let hub_clicks: &[(&[LiveId], &str)] = &[
                (ids!(settings_hub_language), "language"),
                (ids!(settings_hub_country), "country"),
                (ids!(settings_hub_tills), "tills"),
                (ids!(settings_hub_brand), "brand"),
                (ids!(settings_hub_units), "units"),
                (ids!(settings_hub_pickers), "pickers"),
                (ids!(settings_hub_printers), "printers"),
                (ids!(settings_hub_bill), "bill"),
                (ids!(settings_hub_backup), "backup"),
                (ids!(settings_hub_diag), "diag"),
            ];
            let mut opened: Option<&'static str> = None;
            for (id, key) in hub_clicks {
                if self.ui.button(cx, id).clicked(actions) {
                    opened = Some(*key);
                    break;
                }
            }
            if let Some(key) = opened {
                app_state().lock().unwrap().settings_panel = Some(key);
                self.sync_settings_panels(cx);
            } else if self.ui.button(cx, ids!(settings_detail_back)).clicked(actions) {
                app_state().lock().unwrap().settings_panel = None;
                self.sync_settings_panels(cx);
            }
        }
        self.handle_diag_actions(cx, actions);
        // "Type or pick" fields (suggest.rs): focus opens the list (empty =
        // everything the shop has), typing narrows it, a tap fills the field,
        // Escape or focusing another field closes it. Losing focus alone does
        // not close it: tapping a row takes focus off the field first.
        {
            let open = app_state().lock().unwrap().suggest_open;
            let mut changed = false;
            let mut picked: Option<(SuggestField, String)> = None;
            for field in SuggestField::ALL {
                let input = self.ui.text_input(cx, field.input());
                let uid = input.widget_uid();
                let focused = actions
                    .filter_widget_actions_cast::<TextInputAction>(uid)
                    .any(|a| matches!(a, TextInputAction::KeyFocus));
                let typed = input.changed(actions);
                if focused || typed.is_some() {
                    let text = typed.unwrap_or_else(|| input.text());
                    app_state().lock().unwrap().open_suggest(field, &text);
                    changed = true;
                } else if input.escaped(actions) && open == Some(field) {
                    app_state().lock().unwrap().suggest_open = None;
                    changed = true;
                }
                if open == Some(field) {
                    let widget = self.ui.widget(cx, field.list());
                    let list = widget.portal_list(cx, ids!(list));
                    for (item_id, item) in list.items_with_actions(actions) {
                        if item
                            .view(cx, ids!(row))
                            .finger_up(actions)
                            .is_some_and(|e| e.was_tap() && e.is_over)
                        {
                            let value = app_state()
                                .lock()
                                .unwrap()
                                .suggest_rows
                                .get(item_id)
                                .map(|r| r.value.clone());
                            if let Some(v) = value {
                                picked = Some((field, v));
                            }
                        }
                    }
                }
            }
            // Another (ordinary) field took focus: the open list is done.
            if let Some(field) = open
                && !changed
            {
                let mine = self.ui.text_input(cx, field.input()).widget_uid();
                let elsewhere = actions.iter().filter_map(|a| a.as_widget_action()).any(|wa| {
                    wa.widget_uid != mine
                        && matches!(wa.cast::<TextInputAction>(), TextInputAction::KeyFocus)
                });
                if elsewhere {
                    app_state().lock().unwrap().suggest_open = None;
                    changed = true;
                }
            }
            if let Some((field, value)) = picked {
                self.ui.text_input(cx, field.input()).set_text(cx, &value);
                // A known stock item brings its own unit.
                if field == SuggestField::BuyItem {
                    let unit = app_state()
                        .lock()
                        .unwrap()
                        .find_stock_item(&value)
                        .ok()
                        .flatten()
                        .map(|p| p.unit_label.trim().to_string())
                        .unwrap_or_default();
                    if !unit.is_empty() {
                        self.ui.text_input(cx, ids!(buy_unit_input)).set_text(cx, &unit);
                    }
                }
                let mut st = app_state().lock().unwrap();
                st.suggest_open = None;
                // A picked country is shown with what it sets, ready to apply.
                if field == SuggestField::Country {
                    st.country_browse = value.clone();
                }
                // Picking a person to edit is the whole step: select them.
                if field == SuggestField::TeamPick {
                    st.team_select_pending = true;
                }
                if field == SuggestField::CustomerName {
                    if let Some(c) = st
                        .customers
                        .iter()
                        .find(|c| c.name.eq_ignore_ascii_case(value.trim()))
                    {
                        st.selected_customer_id = Some(c.id.clone());
                    }
                }
                if field == SuggestField::RoutePick {
                    st.printer_route_key =
                        resolve_printer_route_key(&value, &st.stations, |k| st.t(k));
                }
                changed = true;
                if field == SuggestField::KotStation {
                    // Use → put unsent dishes on this station; Don't use → clear it.
                    let clear = st
                        .suggest_rows
                        .iter()
                        .find(|r| r.value == value)
                        .map(|r| {
                            let a = r.action.trim();
                            !a.is_empty() && a != st.t("use")
                        })
                        .unwrap_or(false);
                    drop(st);
                    let result = if clear {
                        self.clear_kot_station_from_unsent(cx, &value)
                    } else {
                        self.apply_kot_station_to_unassigned(cx, &value)
                    };
                    match result {
                        Ok(msg) => self.set_status(cx, msg),
                        Err(e) => self.set_status(cx, e),
                    }
                }
            }
            if changed {
                self.sync_chrome(cx);
                for field in SuggestField::ALL {
                    self.ui.widget(cx, field.list()).redraw(cx);
                }
            }
        }
        // Category field: same picker behaviour as the item name — opens on
        // focus (empty = all categories + suggestions), narrows as you type,
        // Escape closes. Typing makes it the owner's own choice.
        {
            let input = self.ui.text_input(cx, ids!(menu_category_input));
            let focused = actions
                .filter_widget_actions_cast::<TextInputAction>(input.widget_uid())
                .any(|a| matches!(a, TextInputAction::KeyFocus));
            let typed = input.changed(actions);
            let escaped = input.escaped(actions);
            if focused || typed.is_some() || escaped {
                let text = typed.clone().unwrap_or_else(|| input.text());
                {
                    let mut st = app_state().lock().unwrap();
                    if typed.is_some() {
                        st.form_category_auto = false;
                        let still = st
                            .form_category_photo
                            .as_ref()
                            .is_some_and(|(label, _)| label.trim().eq_ignore_ascii_case(text.trim()));
                        if !still {
                            st.form_category_photo = None;
                        }
                    }
                    st.category_picker_open = !escaped;
                    if !escaped {
                        st.suggest_open = None;
                        st.dish_picker_open = false;
                        st.refresh_category_picks(&text);
                    }
                }
                self.sync_chrome(cx);
            }
        }
        {
            let widget = self.ui.widget(cx, ids!(category_pick_list));
            let list = widget.portal_list(cx, ids!(list));
            let mut picked = None;
            for (item_id, item) in list.items_with_actions(actions) {
                let tapped = item
                    .view(cx, ids!(dishrow))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over);
                if tapped {
                    picked = app_state().lock().unwrap().category_picks.get(item_id).cloned();
                }
            }
            if let Some(pick) = picked {
                match pick {
                    CategoryPick::New(name) => self.set_form_category(cx, &name, None, None, false),
                    CategoryPick::Mine(id) => {
                        let name = app_state()
                            .lock()
                            .unwrap()
                            .categories
                            .iter()
                            .find(|c| c.id == id)
                            .map(|c| c.name.clone())
                            .unwrap_or_default();
                        self.set_form_category(cx, &name, Some(id), None, false);
                    }
                    CategoryPick::Shelf(folder) => {
                        self.set_form_category(cx, &folder.label, None, Some(folder.photo), false)
                    }
                }
                self.sync_chrome(cx);
            }
        }
        // The picker opens as soon as the name field is focused — empty, it
        // browses every dish, so an owner who would rather not type (or not
        // type in English) can just pick. Typing narrows it (your own name
        // first); clearing the text goes back to browsing. Escape closes it.
        // Losing focus does not: tapping a row takes focus off the field
        // before the tap lands. Also keeps the draft thumb's initials live.
        {
            let name_input = self.ui.text_input(cx, ids!(menu_name_input));
            let focused = actions
                .filter_widget_actions_cast::<TextInputAction>(name_input.widget_uid())
                .any(|a| matches!(a, TextInputAction::KeyFocus));
            let typed = name_input.changed(actions);
            let escaped = name_input.escaped(actions);
            if focused || typed.is_some() || escaped {
                let text = typed.unwrap_or_else(|| name_input.text());
                {
                    let mut st = app_state().lock().unwrap();
                    st.dish_picker_open = !escaped;
                    if !escaped {
                        st.suggest_open = None;
                        st.category_picker_open = false;
                        st.refresh_dish_picks(&text);
                    }
                }
                self.sync_chrome(cx);
            }
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
        // Menu window grid (the window stays open for the next dish): a tap
        // anywhere on a card adds one, the same as its +; − takes one back.
        // A tap on − or + is that button's alone, never also the card's.
        {
            let widget = self.ui.widget(cx, ids!(product_list));
            let list = widget.portal_list(cx, ids!(list));
            let cols = *product_grid_cols().lock().unwrap();
            let slots: [&[LiveId]; 6] = [ids!(c0), ids!(c1), ids!(c2), ids!(c3), ids!(c4), ids!(c5)];
            enum CardTap {
                Add(usize),
                Minus(usize),
            }
            let mut tapped = None;
            for (row_id, item) in list.items_with_actions(actions) {
                for (col, path) in slots.iter().enumerate().take(cols) {
                    let index = row_id * cols + col;
                    let tile = item.widget(cx, *path).view(cx, ids!(tile));
                    if tile.button(cx, ids!(minus)).clicked(actions) {
                        tapped = Some(CardTap::Minus(index));
                    } else if tile.button(cx, ids!(plus)).clicked(actions)
                        || tile.finger_up(actions).is_some_and(|e| e.was_tap() && e.is_over)
                    {
                        tapped = Some(CardTap::Add(index));
                    }
                }
            }
            match tapped {
                Some(CardTap::Add(index)) => self.add_product_to_cart(cx, index),
                Some(CardTap::Minus(index)) => self.remove_product_from_cart(cx, index),
                None => {}
            }
        }
        if let Some(text) = self.ui.text_input(cx, ids!(menu_list_search)).changed(actions) {
            app_state().lock().unwrap().menu_list_filter = text;
            self.ui.widget(cx, ids!(product_list_menu)).redraw(cx);
        }
        if self.ui.button(cx, ids!(menu_unpriced_btn)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.menu_unpriced_only = !st.menu_unpriced_only;
            }
            self.ui.widget(cx, ids!(product_list_menu)).redraw(cx);
            self.sync_chrome(cx);
        }
        // Items list: sold out / archive
        {
            let widget = self.ui.widget(cx, ids!(product_list_menu));
            let list = widget.portal_list(cx, ids!(list));
            let rows = app_state().lock().unwrap().menu_list_indices();
            for (row_id, item) in list.items_with_actions(actions) {
                // Rows follow the search; map back to the product.
                let Some(&item_id) = rows.get(row_id) else {
                    continue;
                };
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
                    && item.view(cx, ids!(card)).finger_up(actions).is_some_and(|e| {
                        // was_tap() also demands <0.5s press time, which
                        // remote-desktop and accessibility clicks often
                        // exceed — the card then looked dead. A release
                        // over the card that did not travel is a tap.
                        e.is_over
                            && !e.has_long_press_occurred
                            && (e.abs_start - e.abs).length() < 24.0
                    })
                {
                    self.open_menu_item(cx, item_id);
                }
            }
        }
        // Removed items: Restore puts the item back exactly as it was.
        {
            let widget = self.ui.widget(cx, ids!(removed_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(restore)).clicked(actions) {
                    self.restore_menu_item(cx, item_id);
                }
            }
        }

        for list_id in [ids!(language_list), ids!(setup_lang_list)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions)
                    && let Some(lang) = LANGUAGES.get(item_id) {
                        self.set_language(cx, lang.code);
                    }
            }
        }
        for list_id in [ids!(currency_list)] {
            let widget = self.ui.widget(cx, list_id);
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions)
                    && let Some(cur) = CURRENCIES.get(item_id) {
                        self.set_currency(cx, cur.code);
                    }
            }
        }

        // Item picker under the name field: a tap on any row (the whole row
        // is the target) — keep your own name, open an item already on the
        // menu, or fill name + photo from a shelf dish. Nothing is saved
        // until Add item.
        {
            let widget = self.ui.widget(cx, ids!(dish_pick_list));
            let list = widget.portal_list(cx, ids!(list));
            let mut picked = None;
            for (item_id, item) in list.items_with_actions(actions) {
                let tapped = item
                    .view(cx, ids!(dishrow))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over);
                if tapped {
                    picked = app_state().lock().unwrap().dish_picks.get(item_id).cloned();
                }
            }
            if let Some(pick) = picked {
                app_state().lock().unwrap().dish_picker_open = false;
                match pick {
                    DishPick::Own(name) => self.use_own_item_name(cx, &name),
                    DishPick::Menu(id) => {
                        let index = app_state()
                            .lock()
                            .unwrap()
                            .products
                            .iter()
                            .position(|p| p.id == id);
                        if let Some(index) = index {
                            self.open_menu_item(cx, index);
                        }
                    }
                    DishPick::Shelf(index) => {
                        if let Some(dish) = menu_images::shelf().get(index) {
                            self.use_shelf_dish(cx, dish);
                        }
                    }
                }
                self.sync_chrome(cx);
            }
        }
        // Browse opens the picker on the whole shelf (or what is typed);
        // tapping it again closes it.
        // Your categories: Use files the form's item under it; Photo
        // gives the category a picture (initials until then).
        {
            let widget = self.ui.widget(cx, ids!(category_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(pick)).clicked(actions) {
                    let cat = {
                        let st = app_state().lock().unwrap();
                        st.categories.get(item_id).cloned()
                    };
                    if let Some(cat) = cat {
                        self.ui
                            .text_input(cx, ids!(menu_category_input))
                            .set_text(cx, &cat.name);
                        {
                            // Identity travels by UUID, not by the typed name.
                            let mut st = app_state().lock().unwrap();
                            st.form_category_id = Some(cat.id.clone());
                            st.form_category_label = cat.name.clone();
                        }
                        self.set_status(
                            cx,
                            format!("Category set to {} — Add or save the item to file it there", cat.name),
                        );
                        self.sync_chrome(cx);
                    }
                }
                if item.button(cx, ids!(photo)).clicked(actions) {
                    let cat = {
                        let st = app_state().lock().unwrap();
                        st.categories.get(item_id).cloned()
                    };
                    if let Some(cat) = cat {
                        app_state().lock().unwrap().category_photo_id = Some(cat.id);
                        let dialog = FileDialog::new()
                            .set_title("Choose category photo".into())
                            .add_filter(
                                "Images".into(),
                                vec![
                                    "webp".into(),
                                    "png".into(),
                                    "jpg".into(),
                                    "jpeg".into(),
                                ],
                            )
                            .set_id(live_id!(category_photo_pick));
                        cx.open_select_file_dialog(dialog);
                        self.set_status(cx, "Opening photo picker…");
                    }
                }
            }
        }

        let cart_widget = self.ui.widget(cx, ids!(cart_list));
        let cart_list = cart_widget.portal_list(cx, ids!(list));
        for (item_id, item) in cart_list.items_with_actions(actions) {
            let stepped = if item.button(cx, ids!(minus)).clicked(actions) {
                self.bump_cart_line(cx, item_id, -1);
                true
            } else if item.button(cx, ids!(plus)).clicked(actions) {
                self.bump_cart_line(cx, item_id, 1);
                true
            } else if item.button(cx, ids!(hold_send_btn)).clicked(actions) {
                {
                    let mut st = app_state().lock().unwrap();
                    if let Some(line) = st.cart.lines.get_mut(item_id) {
                        if !line.is_sent() {
                            line.hold_send = !line.hold_send;
                        }
                    }
                }
                self.sync_chrome(cx);
                true
            } else {
                false
            };
            // ± / Hold sit inside the row: a step must not also open the details.
            if !stepped
                && item
                    .view(cx, ids!(summary))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over)
            {
                self.toggle_cart_line(cx, item_id);
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
            for (button, action) in [
                (ids!(line_cancel), LineAction::Cancel),
                (ids!(line_nc), LineAction::Complimentary),
                (ids!(line_move), LineAction::Move),
            ] {
                if item.button(cx, button).clicked(actions) {
                    self.open_line_action(cx, item_id, action);
                }
            }
        }

        let sale_widget = self.ui.widget(cx, ids!(sale_list));
        let sale_list = sale_widget.portal_list(cx, ids!(list));
        for (item_id, item) in sale_list.items_with_actions(actions) {
            // The whole row picks the sale.
            if item.view(cx, ids!(row)).finger_up(actions).is_some_and(|e| e.was_tap() && e.is_over) {
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
                    st.open_cart_line = None;
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
                app_state()
                    .lock()
                    .unwrap()
                    .amend_cart
                    .bump_qty(item_id, -1);
                self.sync_chrome(cx);
            }
            if item.button(cx, ids!(plus)).clicked(actions) {
                app_state()
                    .lock()
                    .unwrap()
                    .amend_cart
                    .bump_qty(item_id, 1);
                self.sync_chrome(cx);
            } else if !item.button(cx, ids!(minus)).clicked(actions)
                && item
                    .view(cx, ids!(summary))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over)
            {
                self.toggle_cart_line(cx, item_id);
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
                    app_state().lock().unwrap().amend_cart.set_note(item_id, text)?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            } else if note_input.key_focus_lost(actions) {
                let text = note_input.text();
                let result = (|| -> Result<(), String> {
                    app_state().lock().unwrap().amend_cart.set_note(item_id, text)?;
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
            let report_errors = amend_total.returned(actions).is_some()
                || amend_total.key_focus_lost(actions);
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
                && report_errors {
                    self.set_status(cx, e);
                }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(amend_btn)).clicked(actions) {
            let reason = self.ui.text_input(cx, ids!(reason_input)).text();
            let typed_total = self.ui.text_input(cx, ids!(amend_total_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.role.can_amend_sale() {
                    return Err("Not allowed to amend sales".into());
                }
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                let currency = st.locale.currency.clone();
                if let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                    && minor > 0 {
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
                    };
                    st.lan_job(
                        "/amend",
                        &req,
                        "Amend".into(),
                        "Amended on the shop's books · history kept".into(),
                        phone::Follow::Refresh,
                        false,
                    )?;
                    st.status = "Amending…".into();
                    return Ok(());
                }
                let (sale, revisions) = {
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let sale = store.amend_sale(&sale_id, &draft, pay, &reason, &actor)?;
                    let revisions = store.list_sale_revisions(&sale_id)?;
                    (sale, revisions)
                };
                st.amend_cart = Cart::from_sale(&sale);
                st.open_cart_line = None;
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
                        "Sale voided on the shop's books".into(),
                        phone::Follow::Refresh,
                        false,
                    )?;
                    st.status = "Voiding…".into();
                    return Ok(());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let (approved_actor, reason) = approve_for(store, &actor, &pin, &reason)?;
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
                        "Sale refunded on the shop's books".into(),
                        phone::Follow::Refresh,
                        false,
                    )?;
                    st.status = "Refunding…".into();
                    return Ok(());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let (approved_actor, reason) = approve_for(store, &actor, &pin, &reason)?;
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

        for ((chip, chip_on), role, key) in [
            ((ids!(staff_role_waiter), ids!(staff_role_waiter_on)), StaffRole::Waiter, "role_waiter"),
            ((ids!(staff_role_kitchen), ids!(staff_role_kitchen_on)), StaffRole::Kitchen, "role_kitchen"),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                let label = {
                    let mut st = app_state().lock().unwrap();
                    st.draft_staff_role = role;
                    st.draft_features = StaffFeatures::for_role(role);
                    st.t(key)
                };
                self.set_status(cx, format!("Preset: {label}"));
                self.sync_chrome(cx);
            }
        }
        if self.choice_chip_clicked(cx, actions, ids!(staff_role_cashier), ids!(staff_role_cashier_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.draft_staff_role = StaffRole::Cashier;
                st.draft_features = StaffFeatures::for_role(StaffRole::Cashier);
            }
            self.set_status(cx, "Preset: Cashier");
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(staff_role_manager), ids!(staff_role_manager_on)) {
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
                st.team_staff_id = st.staff.iter().rev().find(|s| !s.is_owner()).map(|s| s.id.clone());
                st.status = format!("Added {}", name.trim());
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui.text_input(cx, ids!(staff_name_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_title_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_pin_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_login_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(staff_password_input)).set_text(cx, "");
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

        if self.ui.button(cx, ids!(change_my_password)).clicked(actions) {
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
                self.ui.text_input(cx, ids!(my_password_input)).set_text(cx, "");
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
            (ids!(feat_floor), ids!(feat_floor_on), "floor"),
            (ids!(feat_kitchen), ids!(feat_kitchen_on), "kitchen"),
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
                    "floor" => st.draft_features.floor = !st.draft_features.floor,
                    "kitchen" => st.draft_features.kitchen = !st.draft_features.kitchen,
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
        if self.ui.button(cx, ids!(feat_save)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change staff permissions")?;
                let actor = st.actor()?;
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .or_else(|| st.staff.iter().find(|s| !s.is_owner()).map(|s| s.id.clone()))
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
                st.status = format!("{name} pack applied — check taxes on Menu items");
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
        for (id, pricing) in [
            (ids!(tax_inclusive_btn), TaxPricing::Inclusive),
            (ids!(tax_exclusive_btn), TaxPricing::Exclusive),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
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
        for (id, allow) in [
            (ids!(zero_sales_on_btn), true),
            (ids!(zero_sales_off_btn), false),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Change zero-value sales")?;
                    let actor = st.actor()?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.set_allow_zero_sales(allow, &actor)?;
                    let msg = st.t(if allow {
                        "zero_sales_saved_on"
                    } else {
                        "zero_sales_saved_off"
                    });
                    st.status = msg.clone();
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
                    format!("{} saved — it prints on every bill", st.fiscal.business_id_label)
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

        if self.ui.button(cx, ids!(onboard_save_venue)).clicked(actions) {
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
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(onboard_fresh),
            ids!(onboard_fresh_on),
        ) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.onboard_step = 1;
                st.status = "Fresh selected — empty catalogue; tap Done — open books when ready".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(onboard_template), ids!(onboard_template_on)) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                st.onboard_step = 3;
                st.status = st.t("template_info")
                    .replace("{c}", &menu_images::shelf_folders(menu_images::shelf()).len().to_string())
                    .replace("{d}", &menu_images::shelf().len().to_string());
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
                    "Book selected — add opening items / credit below, then Done — open books".into();
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
                if name.trim().is_empty() {
                    return Err("Name required".into());
                }
                // Same rule as the Menu form: blank is ₹0, only negative is refused.
                let minor = match price.trim() {
                    "" => 0,
                    typed => locale::parse_major_input(typed, &currency)?,
                };
                if let Some(msg) = domain::menu_price_error(Money(minor)) {
                    return Err(msg);
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
                st.store.as_ref().ok_or_else(|| "No store".to_string())?.upsert_product(&p)?;
                st.reload_all()?;
                st.status = format!("Added {}", p.name);
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            else {
                self.ui.text_input(cx, ids!(onboard_item_name)).set_text(cx, "");
                self.ui.text_input(cx, ids!(onboard_item_price)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(onboard_add_credit)).clicked(actions) {
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
            if let Err(e) = result { self.set_status(cx, e); }
            else {
                self.ui.text_input(cx, ids!(onboard_cust_name)).set_text(cx, "");
                self.ui.text_input(cx, ids!(onboard_cust_bal)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(onboard_done)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                Self::ensure_default_template(&mut st)?;
                // Chose the template: bring in every category and dish now.
                let imported = if st.onboard_step == 3 {
                    Some(
                        st.store
                            .as_ref()
                            .ok_or_else(|| "No store".to_string())?
                            .import_catalogue(menu_images::shelf())?,
                    )
                } else {
                    None
                };
                st.store.as_ref().ok_or_else(|| "No store".to_string())?.complete_onboarding()?;
                st.onboarding_complete = true;
                st.screen = unlocked_home();
                st.status = match imported {
                    Some(done) => st
                        .t("template_imported")
                        .replace("{d}", &done.items_added.to_string())
                        .replace("{c}", &done.categories_added.to_string()),
                    None => "Shop ready — your books are open".into(),
                };
                if imported.is_some() {
                    // Peer land on Items (prices) — not a drill-down from Onboard.
                    st.nav_back.clear();
                    st.nav_forward.clear();
                    st.screen = Screen::Menu;
                }
                let _ = st.reload_all();
                let _ = st.refresh_books_overview();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
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
                .name("khaynjo-find-server".into())
                .spawn(|| {
                    let found = net::discover_servers(std::time::Duration::from_millis(3000));
                    *discovery_result().lock().unwrap() = Some(found);
                    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                });
        }
        if self.ui.button(cx, ids!(bill_format_save)).clicked(actions) {
            let header = self.ui.text_input(cx, ids!(bill_header_input)).text();
            let fssai = self.ui.text_input(cx, ids!(fssai_input)).text();
            let footer = self.ui.text_input(cx, ids!(bill_footer_input)).text();
            let upi_vpa = self.ui.text_input(cx, ids!(upi_vpa_input)).text();
            let upi_payee = self.ui.text_input(cx, ids!(upi_payee_input)).text();
            let result = (|| -> Result<(), String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Change the bill format")?;
                let lines = header
                    .split('|')
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .set_bill_format_ex(&lines, &fssai, &footer, Some(&upi_vpa), Some(&upi_payee))
            })();
            match result {
                Ok(()) => {
                    self.fill_bill_format_fields(cx);
                    self.set_status(cx, "Bill format saved");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(print_stamp_pick)).clicked(actions) {
            self.pick_print_brand_image(cx, live_id!(print_stamp_pick), "Choose stamp image");
        }
        if self.ui.button(cx, ids!(print_stamp_clear)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Change the bill format")?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .set_print_stamp_key(None)
            })();
            match result {
                Ok(()) => {
                    self.fill_bill_format_fields(cx);
                    self.set_status(cx, "Stamp cleared");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(payment_qr_pick)).clicked(actions) {
            self.pick_print_brand_image(cx, live_id!(payment_qr_pick), "Choose payment QR image");
        }
        if self.ui.button(cx, ids!(payment_qr_clear)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Change the bill format")?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .set_payment_qr_key(None)
            })();
            match result {
                Ok(()) => {
                    self.fill_bill_format_fields(cx);
                    self.set_status(cx, "Payment QR cleared");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(aggregator_import_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Import aggregator orders")?;
                let n = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .import_aggregator_orders_drop_file()?;
                Ok(format!("Imported {n} aggregator order(s) into the inbox"))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(scheme_add)).clicked(actions) {
            let text = self.ui.text_input(cx, ids!(scheme_bogo_input)).text();
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                st.require_server_role("Change schemes")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let (buy_name, buy_qty, get_name, get_qty) =
                    schemes::Scheme::parse_bogo_human(&text)
                        .ok_or_else(|| "Type like: Buy 2 Thali get 1 Chaas".to_string())?;
                let products = store.list_products(false)?;
                let find = |name: &str| {
                    products
                        .iter()
                        .find(|p| p.name.eq_ignore_ascii_case(name.trim()))
                        .map(|p| p.id.clone())
                        .ok_or_else(|| format!("No menu item named '{name}'"))
                };
                let buy_id = find(&buy_name)?;
                let get_id = find(&get_name)?;
                let scheme = schemes::Scheme::bogo(
                    domain::new_id(),
                    text.trim(),
                    &buy_id,
                    buy_qty,
                    &get_id,
                    get_qty,
                );
                store.upsert_scheme(&scheme)?;
                Ok(format!("Added scheme: {}", scheme.name))
            })();
            match result {
                Ok(msg) => {
                    self.fill_bill_format_fields(cx);
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(show_upi_qr)).clicked(actions) {
            self.open_upi_qr_modal(cx);
        }
        if self.ui.button(cx, ids!(upi_qr_close)).clicked(actions)
            || self.ui.modal(cx, ids!(upi_qr_modal)).dismissed(actions)
        {
            self.ui.modal(cx, ids!(upi_qr_modal)).close(cx);
        }
        if self.ui.button(cx, ids!(recall_kot_btn)).clicked(actions) {
            let typed = self.ui.text_input(cx, ids!(recall_kot_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let recent = store.list_recent_completed_kots(12)?;
                let kot_id = if let Ok(n) = typed.trim().trim_start_matches('#').parse::<i64>() {
                    recent
                        .iter()
                        .find(|k| k.kot_no == n)
                        .map(|k| k.id.clone())
                        .ok_or_else(|| format!("No finished KOT #{n} today"))?
                } else if !typed.trim().is_empty() {
                    recent
                        .iter()
                        .find(|k| k.id == typed.trim() || k.summary.contains(typed.trim()))
                        .map(|k| k.id.clone())
                        .or_else(|| recent.first().map(|k| k.id.clone()))
                        .ok_or_else(|| "No finished ticket to recall".to_string())?
                } else {
                    recent
                        .first()
                        .map(|k| k.id.clone())
                        .ok_or_else(|| "No finished ticket to recall".to_string())?
                };
                if st.device_role == DeviceRole::Client {
                    st.lan_job(
                        "/kot_recall",
                        &net::RemoteKotAdvance { kot_id },
                        "Recall ticket".into(),
                        "Ticket recalled".into(),
                        phone::Follow::Refresh,
                        false,
                    )?;
                } else {
                    store.recall_kot(&kot_id, &actor)?;
                    st.refresh_live_lists();
                }
                Ok(())
            })();
            match result {
                Ok(()) => self.set_status(cx, "Ticket recalled"),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        let team_pick_now =
            std::mem::take(&mut app_state().lock().unwrap().team_select_pending);
        if team_pick_now || self.ui.text_input(cx, ids!(team_pick_input)).returned(actions).is_some() {
            let name = self.ui.text_input(cx, ids!(team_pick_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let needle = name.trim().to_lowercase();
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
                st.draft_features = member.features.clone();
                Ok(format!("Editing {} ({})", member.name, member.role_title))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for (button, right) in [
            (ids!(right_settle_btn), domain::Right::Settle),
            (ids!(right_cancel_btn), domain::Right::CancelSentItem),
            (ids!(right_nc_btn), domain::Right::Complimentary),
            (ids!(right_move_btn), domain::Right::MoveTable),
            (ids!(right_reprint_btn), domain::Right::Reprint),
            (ids!(right_discount_btn), domain::Right::Discount),
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
            // Three tablets must not all be "Android device" in the shop's list.
            let name = Some(self.ui.text_input(cx, ids!(client_name_input)).text())
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(device_name);
            if host.trim().is_empty() || code.trim().is_empty() {
                self.set_status(cx, "Type the shop device's address and its pairing code");
            } else {
                app_state().lock().unwrap().pair_note = "Pairing…".into();
                self.set_status(cx, "Pairing…");
                let _ = std::thread::Builder::new()
                    .name("khaynjo-pair".into())
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
            self.set_status(cx, "Refreshing from the shop…");
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
        if self.ui.button(cx, ids!(receipt_whatsapp_btn)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let sale_id = st
                    .selected_sale_id
                    .clone()
                    .ok_or_else(|| "Select a sale first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let text = store.format_receipt_text(&sale_id)?;
                let url = store::whatsapp_bill_url(&text);
                let phone = store
                    .get_sale(&sale_id)?
                    .and_then(|s| s.customer_id)
                    .and_then(|cid| {
                        st.customers
                            .iter()
                            .find(|c| c.id == cid)
                            .map(|c| c.phone.clone())
                    })
                    .unwrap_or_default();
                let sms = notify::queue_sms_bill(&phone, &text);
                let _ = store.queue_notify(&sms);
                Ok(url)
            })();
            match result {
                Ok(url) => {
                    self.copy_text_to_clipboard(cx, &url);
                    #[cfg(target_os = "windows")]
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &url])
                        .spawn();
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open").arg(&url).spawn();
                    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
                    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                    self.set_status(cx, "WhatsApp share opened (Net optional on device)");
                }
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
            let today = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_today),
                ids!(reports_today_on),
            );
            let month = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_month),
                ids!(reports_month_on),
            );
            let aging = self.choice_chip_clicked(
                cx,
                actions,
                ids!(reports_aging),
                ids!(reports_aging_on),
            );
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

        if self.ui.button(cx, ids!(reports_copy)).clicked(actions) {
            let text = app_state().lock().unwrap().day_close_text.clone();
            if text.trim().is_empty() {
                self.set_status(cx, "Open a report first");
            } else {
                self.copy_text_to_clipboard(cx, &text);
            }
        }
        if self.ui.button(cx, ids!(sale_attach_add_note)).clicked(actions) {
            self.add_entry_note(cx, AttachmentEntity::Sale, ids!(sale_attach_note));
        }
        if self.ui.button(cx, ids!(sale_attach_photo)).clicked(actions) {
            self.attach_pick_image(cx, AttachmentEntity::Sale);
        }
        if self.ui.button(cx, ids!(sale_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Sale);
        }
        if self.ui.button(cx, ids!(party_attach_add_note)).clicked(actions) {
            self.add_entry_note(cx, AttachmentEntity::Party, ids!(party_attach_note));
        }
        if self.ui.button(cx, ids!(party_attach_photo)).clicked(actions) {
            self.attach_pick_image(cx, AttachmentEntity::Party);
        }
        if self.ui.button(cx, ids!(party_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Party);
        }
        if self.ui.button(cx, ids!(expense_attach_photo)).clicked(actions) {
            self.attach_pick_image(cx, AttachmentEntity::Expense);
        }
        if self.ui.button(cx, ids!(expense_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Expense);
        }
        for (id, kind) in [
            (ids!(bank_till_to_bank), MoneyMoveKind::TillToBank),
            (ids!(bank_bank_to_till), MoneyMoveKind::BankToTill),
            (ids!(bank_in), MoneyMoveKind::BankIn),
            (ids!(bank_out), MoneyMoveKind::BankOut),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(bank_amount_input)).text();
                let note = self.ui.text_input(cx, ids!(bank_note_input)).text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Post bank / cashbook")?;
                    let actor = st.actor()?;
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&amount_text, &currency)?;
                    st.store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .post_money_movement(kind, minor, &note, &actor)?;
                    st.day_close_text = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .books_report_text("today")?;
                    st.nav_back.clear();
                    st.nav_forward.clear();
                    st.screen = Screen::Reports;
                    st.status = format!("{} posted", kind.label());
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.ui.text_input(cx, ids!(bank_amount_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(bank_note_input)).set_text(cx, "");
                }
                self.sync_chrome(cx);
            }
        }

        // Bank reconciliation tick-off: # refers to the numbered
        // uncleared list under Bank / cashbook.
        for (btn, cleared) in [
            (ids!(bank_mark_cleared), true),
            (ids!(bank_mark_uncleared), false),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let num_text = self.ui.text_input(cx, ids!(bank_clear_input)).text();
                let result = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    st.require_server_role("Reconcile bank movements")?;
                    let actor = st.actor()?;
                    let n: usize = num_text
                        .trim()
                        .parse()
                        .map_err(|_| "Type the uncleared # from the list".to_string())?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let uncleared: Vec<String> = store
                        .list_money_movements(50)?
                        .into_iter()
                        .filter(|m| !m.cleared)
                        .map(|m| m.id)
                        .collect();
                    let id = uncleared
                        .get(n.wrapping_sub(1))
                        .cloned()
                        .ok_or_else(|| "No such uncleared movement".to_string())?;
                    store.set_money_movement_cleared(&id, cleared, &actor)?;
                    Ok(if cleared {
                        "Marked cleared — seen on the statement".into()
                    } else {
                        "Unmarked".into()
                    })
                })();
                match result {
                    Ok(msg) => {
                        self.ui.text_input(cx, ids!(bank_clear_input)).set_text(cx, "");
                        self.set_status(cx, msg);
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }

        // Spreadsheet (CSV): copy out, or bring parties / items in.
        for (btn, which) in [
            (ids!(csv_copy_day), "day"),
            (ids!(csv_copy_outstanding), "outstanding"),
            (ids!(csv_copy_parties), "parties"),
            (ids!(csv_copy_items), "items"),
            (ids!(csv_copy_ledger), "ledger"),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let text = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    match which {
                        "day" => store.day_book_csv(),
                        "outstanding" => store.outstanding_csv(),
                        "parties" => store.parties_csv(),
                        "items" => store.items_csv(),
                        _ => {
                            let id = st
                                .selected_customer_id
                                .clone()
                                .ok_or_else(|| "Pick a party first (Parties tab)".to_string())?;
                            store.party_ledger_csv(&id)
                        }
                    }
                })();
                match text {
                    Ok(text) => {
                        self.copy_text_to_clipboard(cx, &text);
                        self.set_status(cx, "CSV copied — paste into your spreadsheet");
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
        }
        if self.ui.button(cx, ids!(csv_import_parties)).clicked(actions) {
            let dialog = FileDialog::new()
                .set_title("Choose parties CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_parties_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
        }
        if self.ui.button(cx, ids!(csv_import_items)).clicked(actions) {
            let dialog = FileDialog::new()
                .set_title("Choose items CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_items_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
        }

        let inv_minus = self.ui.button(cx, ids!(inv_adjust_minus)).clicked(actions);
        let inv_plus = self.ui.button(cx, ids!(inv_adjust_plus)).clicked(actions);
        if inv_minus || inv_plus {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(inv_qty_input)).text();
            let note = self.ui.text_input(cx, ids!(inv_note_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Adjust stock")?;
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let qty: i64 = qty_text
                    .trim()
                    .parse()
                    .map_err(|_| "Quantity must be a whole number".to_string())?;
                if qty <= 0 {
                    return Err("Quantity must be more than zero".into());
                }
                // A new name on + Stock is a new raw material; − needs an item
                // the kitchen has. A resold good not counted yet starts at 0.
                let product = match st.find_stock_item(&name)? {
                    Some(p) => p,
                    None if inv_plus => st.stock_item_or_new(&name, "")?,
                    None => return Err(format!("Nothing called \"{}\" in stock", name.trim())),
                };
                if !product.tracks_stock() {
                    st.store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .set_product_stock_qty(&product.id, 0)?;
                }
                let delta = if inv_plus { qty } else { -qty };
                let note = note.trim().to_string();
                let reason = if note.is_empty() { "adjust" } else { note.as_str() };
                let on_hand = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .adjust_stock(&product.id, delta, reason, &note, &actor)?;
                st.reload_all()?;
                st.refresh_inventory_text()?;
                Ok(format!("{} · {on_hand} on hand", product.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_qty_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(waitlist_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(wait_name_input)).text();
            let party = self.ui.text_input(cx, ids!(wait_party_input)).text();
            let mins = self.ui.text_input(cx, ids!(wait_mins_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or("No store")?;
                let party_n: i64 = party.trim().parse().unwrap_or(2);
                let wait_n: i64 = mins.trim().parse().unwrap_or(15);
                store.add_waitlist(&name, party_n, wait_n, &actor)?;
                let _ = st.refresh_inventory_text();
                Ok(format!("Waitlist: {name}"))
            })();
            match result {
                Ok(m) => {
                    for id in [ids!(wait_name_input), ids!(wait_party_input), ids!(wait_mins_input)] {
                        self.ui.text_input(cx, id).set_text(cx, "");
                    }
                    self.set_status(cx, m);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(bulk_price_apply)).clicked(actions) {
            let cat = self.ui.text_input(cx, ids!(bulk_cat_input)).text();
            let pct = self.ui.text_input(cx, ids!(bulk_pct_input)).text();
            let set_amt = self.ui.text_input(cx, ids!(bulk_set_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Bulk price")?;
                let actor = st.actor()?;
                let store = st.store.as_ref().ok_or("No store")?;
                let pct_i: Option<i64> = pct.trim().parse().ok();
                let set_paise = if set_amt.trim().is_empty() {
                    None
                } else {
                    let major: i64 = set_amt
                        .trim()
                        .replace(',', "")
                        .parse()
                        .map_err(|_| "Set amount must be a number".to_string())?;
                    Some(domain::Money::from_rupees(major).paise())
                };
                let n = store.bulk_price_category(&cat, pct_i, set_paise, &actor)?;
                st.reload_all()?;
                Ok(format!("Updated {n} item(s) in {cat}"))
            })();
            match result {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(resv_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(resv_name_input)).text();
            let phone_raw = self.ui.text_input(cx, ids!(resv_phone_input)).text();
            let guests = self.ui.text_input(cx, ids!(resv_guests_input)).text();
            let when = self.ui.text_input(cx, ids!(resv_when_input)).text();
            let table = self.ui.text_input(cx, ids!(resv_table_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Book tables")?;
                let actor = st.actor()?;
                let phone = with_calling_code(&phone_raw, &st.fiscal.country_code);
                let guests: i64 = if guests.trim().is_empty() {
                    0
                } else {
                    guests.trim().parse().map_err(|_| "Guests must be a whole number".to_string())?
                };
                let r = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .add_reservation(&name, &phone, guests, &when, &table, &actor)?;
                st.reload_all()?;
                Ok(format!("Booked {} · {}", r.name, r.at))
            })();
            match result {
                Ok(msg) => {
                    for id in [ids!(resv_name_input), ids!(resv_phone_input), ids!(resv_guests_input), ids!(resv_when_input), ids!(resv_table_input)] {
                        self.ui.text_input(cx, id).set_text(cx, "");
                    }
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        {
            let widget = self.ui.widget(cx, ids!(resv_list));
            let list = widget.portal_list(cx, ids!(list));
            let mut acted = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item.button(cx, ids!(seat)).clicked(actions) {
                    acted = Some((item_id, "seated"));
                }
                if item.button(cx, ids!(cancel)).clicked(actions) {
                    acted = Some((item_id, "cancelled"));
                }
            }
            if let Some((index, status)) = acted {
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Change bookings")?;
                    let actor = st.actor()?;
                    let r = st.reservations.get(index).cloned().ok_or("Booking not found")?;
                    st.store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .set_reservation_status(&r.id, status, &actor)?;
                    st.reload_all()?;
                    Ok(if status == "seated" {
                        if r.table_name.is_empty() {
                            format!("{} seated — pick their table", r.name)
                        } else {
                            format!("{} seated at {} — tap {} to order", r.name, r.table_name, r.table_name)
                        }
                    } else {
                        format!("Booking for {} cancelled", r.name)
                    })
                })();
                match result {
                    Ok(msg) | Err(msg) => self.set_status(cx, msg),
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(inv_reorder_save)).clicked(actions)
            || self.ui.text_input(cx, ids!(inv_reorder_input)).returned(actions).is_some()
        {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let level_text = self.ui.text_input(cx, ids!(inv_reorder_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Change reorder levels")?;
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let level: i64 = level_text
                    .trim()
                    .parse()
                    .map_err(|_| "Reorder level must be a whole number".to_string())?;
                let product = st
                    .find_stock_item(&name)?
                    .ok_or_else(|| format!("Nothing called \"{}\" in stock", name.trim()))?;
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .set_reorder_level(&product.id, level, &actor)?;
                st.reload_all()?;
                let unit = product.unit_label.trim();
                Ok(format!("{} · buy more at {level} {unit}", product.name).trim_end().to_string())
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_reorder_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_add)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(branch_name_input)).text();
            let code = self.ui.text_input(cx, ids!(branch_code_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add branches")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can add branches".into());
                }
                let branch = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .add_branch(&name, &code)?;
                st.refresh_branches_text()?;
                Ok(format!("Added {}", branch.label()))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(branch_name_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(branch_code_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_switch)).clicked(actions) {
            let wanted = self.ui.text_input(cx, ids!(branch_switch_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Switch branch")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can switch branch".into());
                }
                let needle = wanted.trim().to_lowercase();
                if needle.is_empty() {
                    return Err("Type the branch name to switch to".into());
                }
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
                store.set_active_branch_id(&branch.id)?;
                // Catalogue, tables and tickets are all branch scoped.
                st.cart.clear();
                st.open_cart_line = None;
                st.selected_table_id = None;
                st.reload_all()?;
                st.refresh_branches_text()?;
                Ok(format!("Now selling for {}", branch.label()))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_export)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can export a branch day".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let branch = store.active_branch()?;
                let json = store.export_branch_snapshot_json()?;
                let dir = store::data_dir()?.join("backups");
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                let code = if branch.code.trim().is_empty() {
                    "BRANCH".to_string()
                } else {
                    branch.code.trim().to_uppercase()
                };
                let path = dir.join(format!(
                    "branch_snapshot_{code}_{}.json",
                    chrono::Local::now().format("%Y-%m-%d")
                ));
                std::fs::write(&path, &json).map_err(|e| e.to_string())?;
                let copied = print_share::copy_text(&json);
                Ok(format!(
                    "Snapshot: {}{}",
                    path.display(),
                    if copied { " (also copied)" } else { "" }
                ))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_import)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Import a branch day")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can import a branch day".into());
                }
                let path = store::data_dir()?
                    .join("backups")
                    .join("import_branch_snapshot.json");
                if !path.exists() {
                    return Err("Missing backups/import_branch_snapshot.json".into());
                }
                let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let branch = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .import_branch_snapshot(&json)?;
                st.refresh_branches_text()?;
                Ok(format!("Imported day for {}", branch.label()))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_consolidated)).clicked(actions) {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can see the roll-up".into());
                }
                let report = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .consolidated_report()?;
                let mut lines = vec![
                    format!("{} — all outlets today", report.org_name),
                    format!(
                        "Sales {} · {}",
                        report.totals.sale_count,
                        st.money(report.totals.revenue())
                    ),
                    format!(
                        "Cash {} · UPI {} · Card {}",
                        st.money(report.totals.cash()),
                        st.money(report.totals.upi()),
                        st.money(report.totals.card())
                    ),
                    format!(
                        "Still owed {} · Expenses {}",
                        st.money(Money(report.outstanding_credit_paise)),
                        st.money(Money(report.expenses_paise))
                    ),
                    String::new(),
                    "Per branch:".to_string(),
                ];
                for (branch_name, totals) in &report.branches {
                    lines.push(format!(
                        "  {branch_name} · {} sales · {}",
                        totals.sale_count,
                        st.money(totals.revenue())
                    ));
                }
                if !report.stale_branches.is_empty() {
                    lines.push(format!(
                        "Last known only: {}",
                        report.stale_branches.join(", ")
                    ));
                }
                if !report.missing_branches.is_empty() {
                    lines.push(format!(
                        "No snapshot yet: {}",
                        report.missing_branches.join(", ")
                    ));
                }
                st.branches_text = lines.join("\n");
                st.status = "Roll-up ready".into();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(lock_login_ok)).clicked(actions) {
            self.try_unlock_with_login(cx);
            self.sync_chrome(cx);
        }



        // Settings → Units: tap a unit row to turn it On/Off in pickers.
        {
            let widget = self.ui.widget(cx, ids!(units_label));
            let list = widget.portal_list(cx, ids!(list));
            let mut tapped: Option<usize> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item
                    .view(cx, ids!(row))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Change unit pickers")?;
                    let actor = st.actor()?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let rows = units_report_rows(store, |k| st.t(k));
                    let code = match rows.get(item_id) {
                        Some(ReportRow::Kv(k, _)) => k
                            .split(" — ")
                            .next()
                            .map(|c| c.trim().to_string())
                            .filter(|c| !c.is_empty()),
                        _ => None,
                    }
                    .ok_or_else(|| "Not a unit row".to_string())?;
                    let enabled = store
                        .list_units()?
                        .into_iter()
                        .find(|u| u.code == code)
                        .map(|u| u.picker_enabled)
                        .unwrap_or(true);
                    store.set_unit_picker_enabled(&code, !enabled, &actor)?;
                    st.status = if enabled {
                        format!("{code} hidden from pickers")
                    } else {
                        format!("{code} shown in pickers")
                    };
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }
        {
            let widget = self.ui.widget(cx, ids!(sibling_pickers_label));
            let list = widget.portal_list(cx, ids!(list));
            let mut tapped: Option<usize> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item
                    .view(cx, ids!(row))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Change pickers")?;
                    let actor = st.actor()?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let rows = sibling_pickers_rows(store, |k| st.t(k));
                    let key = match rows.get(item_id) {
                        Some(ReportRow::Kv(k, _)) => k.clone(),
                        _ => return Err("Not a picker row".into()),
                    };
                    if let Some(name) = key.strip_prefix("area:") {
                        let on = store
                            .list_picker_areas()?
                            .into_iter()
                            .find(|(n, _)| n == name)
                            .map(|(_, o)| o)
                            .unwrap_or(true);
                        store.set_area_picker_enabled(name, !on, &actor)?;
                        st.status = format!("Area {name} picker {}", if on { "off" } else { "on" });
                    } else if let Some(rest) = key.strip_prefix("station:") {
                        let id = rest.split(" · ").next().unwrap_or("").trim();
                        let on = store
                            .list_stations()?
                            .into_iter()
                            .find(|s| s.id == id)
                            .map(|s| s.picker_enabled)
                            .unwrap_or(true);
                        store.set_station_picker_enabled(id, !on, &actor)?;
                        st.status = format!("Station picker {}", if on { "off" } else { "on" });
                    } else if let Some(rest) = key.strip_prefix("tax:") {
                        let id = rest.split(" · ").next().unwrap_or("").trim();
                        let on = store
                            .list_tax_rates()?
                            .into_iter()
                            .find(|r| r.id == id)
                            .map(|r| r.picker_enabled)
                            .unwrap_or(true);
                        store.set_tax_rate_picker_enabled(id, !on, &actor)?;
                        st.status = format!("Tax picker {}", if on { "off" } else { "on" });
                    } else {
                        return Err("Unknown picker row".into());
                    }
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(unit_add)).clicked(actions) {
            let code = self.ui.text_input(cx, ids!(unit_code_input)).text();
            let name = self.ui.text_input(cx, ids!(unit_name_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add units")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_unit(&code, &name)?;
                st.status = format!("Unit {code} added");
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.ui.text_input(cx, ids!(unit_code_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(unit_name_input)).set_text(cx, "");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(conv_add)).clicked(actions) {
            let from = self.ui.text_input(cx, ids!(conv_from_input)).text();
            let to = self.ui.text_input(cx, ids!(conv_to_input)).text();
            let factor_text = self.ui.text_input(cx, ids!(conv_factor_input)).text();
            let result = (|| -> Result<(), String> {
                let factor_milli = parse_qty_to_milli(&factor_text)
                    .map_err(|_| "Factor must be a number > 0".to_string())?;
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add unit conversion")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_unit_conversion(&from, &to, factor_milli)?;
                st.status = match store.resolve_unit(&from) {
                    Some((family, factor)) => {
                        let anchor = domain::unit_family_anchor(&family).unwrap_or("");
                        format!(
                            "Added — now 1 {from} = {} {anchor} ({family})",
                            precise::format_kg(factor)
                        )
                    }
                    None => format!(
                        "1 {from} → {to} added — '{from}' is not linked to a standard unit yet"
                    ),
                };
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.ui.text_input(cx, ids!(conv_from_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(conv_to_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(conv_factor_input)).set_text(cx, "");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(venue_brand_save)).clicked(actions) {
            self.save_venue_brand(cx);
        }
        if self.ui.button(cx, ids!(venue_icon_pick)).clicked(actions) {
            self.pick_venue_icon(cx);
        }
        if self.ui.button(cx, ids!(venue_icon_clear)).clicked(actions) {
            self.clear_venue_icon(cx);
        }
        if self.ui.button(cx, ids!(onboard_icon_pick)).clicked(actions) {
            self.pick_onboard_icon(cx);
        }

        if self.ui.button(cx, ids!(find_printers_btn)).clicked(actions) {
            print_share::find_printers_in_background();
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(printer_add)).clicked(actions) {
            let label = self.ui.text_input(cx, ids!(printer_label_input)).text();
            let target = self.ui.text_input(cx, ids!(printer_target_input)).text();
            let saved = {
                let mut st = app_state().lock().unwrap();
                st.printer_draft.add_printer(&label, &target);
                // Persist immediately — "Add" without Save used to leave
                // printers.json empty while the UI looked configured.
                st.data_base
                    .as_ref()
                    .and_then(|base| {
                        print_share::save_device_printers(base, &st.printer_draft).ok()
                    })
                    .is_some()
            };
            self.ui.text_input(cx, ids!(printer_label_input)).set_text(cx, "");
            self.ui.text_input(cx, ids!(printer_target_input)).set_text(cx, "");
            if saved {
                self.set_status(cx, app_state().lock().unwrap().t("printers_saved"));
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(printer_remove)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                let id = st
                    .printer_selected_id
                    .clone()
                    .or_else(|| st.printer_draft.printers.last().map(|p| p.id.clone()));
                if let Some(id) = id {
                    st.printer_draft.remove_printer(&id);
                    st.printer_selected_id = None;
                }
                if let Some(base) = st.data_base.as_ref() {
                    let _ = print_share::save_device_printers(base, &st.printer_draft);
                }
            }
            self.sync_chrome(cx);
        }
        // Tap a found printer → fill the Add form (step 1).
        {
            let widget = self.ui.widget(cx, ids!(printers_found_list));
            let list = widget.portal_list(cx, ids!(list));
            let mut tapped: Option<usize> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item
                    .view(cx, ids!(row))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let found = print_share::found_printers();
                if let Some(name) = found.get(item_id).cloned() {
                    let label = name.clone();
                    self.ui
                        .text_input(cx, ids!(printer_label_input))
                        .set_text(cx, &label);
                    self.ui
                        .text_input(cx, ids!(printer_target_input))
                        .set_text(cx, &name);
                    let msg = app_state()
                        .lock()
                        .unwrap()
                        .t("printer_from_find")
                        .replace("{name}", &name);
                    self.set_status(cx, msg);
                }
            }
        }
        // Tap a printer in the pool → select it for Remove (› mark).
        {
            let widget = self.ui.widget(cx, ids!(printers_pool_label));
            let list = widget.portal_list(cx, ids!(list));
            let mut tapped: Option<usize> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item
                    .view(cx, ids!(row))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let mut st = app_state().lock().unwrap();
                if let Some(p) = st.printer_draft.printers.get(item_id).cloned() {
                    st.printer_selected_id = Some(p.id.clone());
                    drop(st);
                    self.ui
                        .text_input(cx, ids!(printer_label_input))
                        .set_text(cx, &p.label);
                    self.ui
                        .text_input(cx, ids!(printer_target_input))
                        .set_text(cx, &p.target);
                    self.set_status(cx, format!("{} — tap Remove to delete", p.label));
                    self.sync_chrome(cx);
                }
            }
        }
        // Tap Bill / kitchen row → that is what step 2 edits (no text field).
        {
            let widget = self.ui.widget(cx, ids!(printer_routes_label));
            let list = widget.portal_list(cx, ids!(list));
            let mut tapped: Option<usize> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item
                    .view(cx, ids!(row))
                    .finger_up(actions)
                    .is_some_and(|e| e.was_tap() && e.is_over)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let (key, name) = {
                    let st = app_state().lock().unwrap();
                    let mut keys: Vec<(String, String)> = Vec::new();
                    keys.push(("bill".into(), st.t("route_bill")));
                    keys.push(("kot:".into(), st.t("route_default_kitchen")));
                    for s in &st.stations {
                        keys.push((format!("kot:{}", s.id), s.name.clone()));
                    }
                    keys
                        .get(item_id)
                        .cloned()
                        .unwrap_or_else(|| ("bill".into(), st.t("route_bill")))
                };
                {
                    let mut st = app_state().lock().unwrap();
                    st.printer_route_key = key;
                }
                self.set_status(
                    cx,
                    app_state()
                        .lock()
                        .unwrap()
                        .t("route_editing")
                        .replace("{name}", &name),
                );
                self.sync_chrome(cx);
            }
        }
        {
            let mut tapped: Option<usize> = None;
            for (i, id) in ROUTE_ADD_PRINTER_BTNS.iter().enumerate() {
                if self.ui.button(cx, id).clicked(actions) {
                    tapped = Some(i);
                    break;
                }
            }
            if let Some(i) = tapped {
                let (ok, empty_pool, msg) = {
                    let mut st = app_state().lock().unwrap();
                    let empty_pool = st.printer_draft.printers.is_empty();
                    let label = st
                        .printer_draft
                        .printers
                        .get(i)
                        .map(|p| p.label.clone())
                        .unwrap_or_default();
                    let place = printer_route_display_name(
                        &st.printer_route_key,
                        &st.stations,
                        |k| st.t(k),
                    );
                    let ok = add_printer_to_focused_route(&mut st, &label);
                    let msg = if empty_pool {
                        st.t("printers_none_hint")
                    } else if ok {
                        st.t("route_assigned")
                            .replace("{place}", &place)
                            .replace("{printer}", &label)
                    } else {
                        String::new()
                    };
                    (ok, empty_pool, msg)
                };
                if empty_pool || ok {
                    self.set_status(cx, msg);
                }
                self.sync_chrome(cx);
            }
        }
        {
            let remove = self.ui.button(cx, ids!(route_remove)).clicked(actions);
            let clear = self.ui.button(cx, ids!(route_clear)).clicked(actions);
            if remove || clear {
                let msg = {
                    let mut st = app_state().lock().unwrap();
                    let key_str = st.printer_route_key.clone();
                    let key = parse_printer_route_key(&key_str);
                    let route = st.printer_draft.route_mut(key);
                    if clear {
                        route.clear();
                    } else if remove {
                        let _ = route.pop();
                    }
                    if let Some(base) = st.data_base.as_ref() {
                        let _ = print_share::save_device_printers(base, &st.printer_draft);
                    }
                    st.t("printers_saved")
                };
                self.set_status(cx, msg);
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(printers_save)).clicked(actions) {
            let result = {
                let st = app_state().lock().unwrap();
                let printers = st.printer_draft.clone();
                match st.data_base.as_ref() {
                    Some(base) => print_share::save_device_printers(base, &printers)
                        .map(|()| st.t("printers_saved")),
                    None => Err("Storage not ready".to_string()),
                }
            };
            match result {
                Ok(msg) | Err(msg) => self.set_status(cx, msg),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(print_test_btn)).clicked(actions) {
            // Test the first printer on the Bill route (or the first in the pool).
            let (dir, msg, own, target_desc) = {
                let st = app_state().lock().unwrap();
                let targets = st.printer_draft.targets_for(print_share::RouteKey::Bill);
                let desc = targets
                    .first()
                    .map(print_share::describe)
                    .or_else(|| {
                        st.printer_draft
                            .printers
                            .first()
                            .map(|p| print_share::describe(&print_share::parse_target(&p.target)))
                    })
                    .unwrap_or_else(|| "No printer".into());
                (st.data_base.clone(), st.t("test_page_sent"), test_page_own_words(&st), desc)
            };
            let when = chrono::Local::now().format("%d %b %Y %H:%M").to_string();
            let page = format!(
                "{app} - test page\n{own}{target_desc}\n{when}\n\nIf you can read this, printing works here.\n",
                app = store::APP_LABEL
            );
            match print_share::print_text(print_share::PrintKind::Bill, &page, dir.as_deref()) {
                Ok(_) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
        }
        for (want_on, idle, active) in [
            (false, ids!(kot_print_off), ids!(kot_print_off_on)),
            (true, ids!(kot_print_on), ids!(kot_print_on_on)),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, active) {
                let result = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.set_kot_auto_print(want_on)?;
                    Ok(if want_on {
                        "Kitchen printing on — KOTs print as they are sent".to_string()
                    } else {
                        "Kitchen printing off — KOTs stay on the Kitchen screen".to_string()
                    })
                })();
                match result {
                    Ok(msg) => self.set_status(cx, msg),
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }

        if self.ui.button(cx, ids!(backup_btn)).clicked(actions) {
            let result = (|| -> Result<std::path::PathBuf, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let path = default_backup_path()?;
                store.backup_to(&path)?;
                Ok(path)
            })();
            match result {
                Ok(path) => {
                    self.ui
                        .widget(cx, ids!(backup_path_label)).set_text(cx, &plain_display_text(&format!("Backup: {}", path.display())));
                    self.set_status(cx, "Backup created");
                }
                Err(e) => self.set_status(cx, e),
            }
        }

        if self.ui.button(cx, ids!(restore_btn)).clicked(actions) {
            // Pick the backup; the automatic ones are where it opens.
            let start = app_state().lock().unwrap().store.as_ref().map(|s| s.auto_backup_dir());
            let title = app_state().lock().unwrap().t("restore_backup");
            let mut dialog = FileDialog::new()
                .set_title(title)
                .add_filter("Backups".into(), vec!["sqlite3".into()])
                .set_id(live_id!(restore_pick));
            if let Some(dir) = start {
                dialog = dialog.set_location(dir);
            }
            cx.open_select_file_dialog(dialog);
        }
    }
}

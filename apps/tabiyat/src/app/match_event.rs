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
        let (undo_hit, snack_hit, cart_qty_hit) = {
            let st = app_state().lock().unwrap();
            (
                !st.undo_timer.is_empty() && st.undo_timer.is_timer(e).is_some(),
                !st.snack_timer.is_empty() && st.snack_timer.is_timer(e).is_some(),
                !st.cart_qty_timer.is_empty() && st.cart_qty_timer.is_timer(e).is_some(),
            )
        };
        if undo_hit {
            self.clear_undo_snackbar(cx);
            self.sync_chrome(cx);
        }
        if snack_hit {
            self.clear_snack(cx);
        }
        if cart_qty_hit {
            self.run_pending_cart_bump(cx);
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
        if e.key_code == KeyCode::Forward
            || (e.key_code == KeyCode::ArrowRight && e.modifiers.alt)
        {
            if nav_gesture_ok() {
                let _ = self.navigate_forward(cx);
            }
            return;
        }

        // Form FieldInput Tab / Shift+Tab (pair, lock, onboard, pay, settings).
        // Runs after Makepad NavControl; overwrites chrome focus via form_tab_anchor.
        if e.key_code == KeyCode::Tab
            && !e.modifiers.control
            && !e.modifiers.alt
            && !e.modifiers.logo
            && !e.is_repeat
        {
            if self.handle_form_field_tab(cx, e.modifiers.shift) {
                return;
            }
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
        let _span = stall_span("handle_actions");
        self.refresh_form_tab_anchor(cx);
        let form_returned_advanced = self.handle_form_field_returned(cx, actions);
        self.tab_chevron_clicks(cx, actions);
        for action in actions.iter() {
            if let Some(fd) = action.downcast_ref::<FileDialogAction>() {
                match fd {
                    // The only folder picker: the second place for backups.
                    FileDialogAction::FolderSelected(dir) => {
                        let result = (|| -> Result<String, String> {
                            let st = app_state().lock().unwrap();
                            st.require_server_host("Choose where backups go")?;
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
                                        let bytes = std::fs::read(&file).map_err(|e| {
                                            format!("Could not read the stamp: {e}")
                                        })?;
                                        let filename = file
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("stamp.webp")
                                            .to_string();
                                        let mut st = app_state().lock().unwrap();
                                        st.books_op(BooksOp::ImportPrintStamp { filename, bytes })
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
                                        let bytes = std::fs::read(&file).map_err(|e| {
                                            format!("Could not read the payment QR: {e}")
                                        })?;
                                        let filename = file
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("payment-qr.webp")
                                            .to_string();
                                        let mut st = app_state().lock().unwrap();
                                        st.books_op(BooksOp::ImportPaymentQr { filename, bytes })
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
                            let is_client = app_state().lock().unwrap().device_role
                                == DeviceRole::Client;
                            if is_client {
                                let sent = (|| -> Result<String, String> {
                                    let bytes = std::fs::read(path).map_err(|e| {
                                        format!("Could not read the file: {e}")
                                    })?;
                                    let filename = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("import.csv")
                                        .to_string();
                                    let mut st = app_state().lock().unwrap();
                                    st.books_op(BooksOp::ImportCsv {
                                        parties,
                                        filename,
                                        bytes,
                                    })
                                })();
                                match sent {
                                    Ok(msg) => self.set_status(cx, msg),
                                    Err(e) => self.set_status(cx, e),
                                }
                            } else {
                                let job = (|| -> Result<(PathBuf, PathBuf, bool), String> {
                                    let st = app_state().lock().unwrap();
                                    let actor = st.actor()?;
                                    // Same person rules as BooksOp::ImportCsv.
                                    if parties {
                                        if !actor.is_owner() && !actor.can_access_credit() {
                                            return Err("Not allowed to import parties".into());
                                        }
                                    } else if !actor.is_owner() && !actor.can_access_items()
                                    {
                                        return Err("Not allowed to import menu CSV".into());
                                    }
                                    let db = st
                                        .store
                                        .as_ref()
                                        .ok_or_else(|| "No store".to_string())?
                                        .db_path()
                                        .to_path_buf();
                                    Ok((db, path.clone(), parties))
                                })();
                                match job {
                                    Ok((db, path, parties)) => {
                                        self.set_status(
                                            cx,
                                            if parties {
                                                "Importing parties…"
                                            } else {
                                                "Importing items…"
                                            },
                                        );
                                        csv_import_job(db, path, parties);
                                    }
                                    Err(e) => self.set_status(cx, e),
                                }
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
        // Whole-rail hide / restore (sidebar mode only) — width tween.
        if self.ui.button(cx, ids!(rail_collapse_btn)).clicked(actions) {
            let from = app_state().lock().unwrap().rail_width;
            app_state().lock().unwrap().rail_collapsed = true;
            self.start_rail_width_anim(cx, from, 0.0);
            self.apply_shell_layout(cx, true);
        }
        if self.ui.button(cx, ids!(rail_expand_btn)).clicked(actions) {
            let to = app_state().lock().unwrap().rail_width.max(RAIL_WIDTH_MIN);
            app_state().lock().unwrap().rail_collapsed = false;
            self.start_rail_width_anim(cx, 0.0, to);
            self.apply_shell_layout(cx, true);
        }
        // Nav clicks honour the user's slot arrangement.
        let nav_click = self.nav_tab_clicked(cx, actions);
        match nav_click {
            Some(NavTab::Books) => {
                let allowed = {
                    let st = app_state().lock().unwrap();
                    match st.session.as_ref() {
                        Some(s) => s.is_owner() || s.can_access_sales() || s.can_access_items(),
                        None => false,
                    }
                };
                if !allowed {
                    self.set_status(cx, "Not allowed");
                } else {
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
                // Team is no longer a chrome peer — open via Settings → Team.
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Settings, |s| s.can_open_settings_shell());
            }
            Some(NavTab::Settings) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Settings, |s| s.can_open_settings_shell());
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
                    st.clear_team_edit_selection();
                    if st.suggest_open == Some(SuggestField::TeamKotStations) {
                        st.suggest_open = None;
                    }
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
            self.go_if_allowed(cx, Screen::SidebarSettings, |s| s.can_access_settings());
        }
        if self.ui.button(cx, ids!(settings_hub_team)).clicked(actions) {
            // Nested under Settings (history Back returns to the hub).
            // Manage team OR own credentials (My PIN) — admin block is hidden for the latter.
            self.go_if_allowed(cx, Screen::Team, |s| {
                s.can_manage_team() || s.can_change_own_credentials()
            });
        }
        if self.ui.button(cx, ids!(snack_secs_save)).clicked(actions) {
            let typed = self.ui.text_input(cx, ids!(snack_secs_input)).text();
            let result = (|| -> Result<String, String> {
                let secs = typed
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| "Enter seconds as a number (3–60)".to_string())?;
                let secs = SnackKind::clamp_secs(secs);
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let user = st
                    .session
                    .as_ref()
                    .map(lan_key)
                    .ok_or_else(|| "Unlock first".to_string())?;
                store.set_snack_secs(&user, secs)?;
                st.snack_secs = secs;
                Ok(st
                    .t("snack_secs_saved")
                    .replace("{n}", &format!("{}", secs.round() as i64)))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(sidebar_settings_back)).clicked(actions)
            || self.ui.button(cx, ids!(header_back)).clicked(actions)
        {
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
            (ids!(floor_add_hdr_ext), "floor_add"),
            (ids!(stations_hdr), "stations"),
            (ids!(stations_hdr_ext), "stations"),
        ] {
            if self.ui.button(cx, hdr).clicked(actions) {
                self.toggle_floor_section(cx, key);
            }
        }
        if self.ui.button(cx, ids!(table_actions_hdr)).clicked(actions) {
            self.toggle_order_section(cx, "table_actions");
        }
        for (hdr, key) in [
            (ids!(setup_lang_hdr), "language"),
            (ids!(setup_lang_hdr_ext), "language"),
            (ids!(setup_role_hdr), "role"),
            (ids!(setup_role_hdr_ext), "role"),
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
            let batch_code = self.ui.text_input(cx, ids!(buy_batch_input)).text();
            let expires_on = self.ui.text_input(cx, ids!(buy_expiry_input)).text();
            let unit_text = self.ui.text_input(cx, ids!(buy_unit_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to record purchases".into());
                }
                let on_account = st.buy_on_account;
                let qty_milli = parse_qty_to_milli(&qty_text)?;
                // Counter buy: existing stock SKU, or a new resale item by name.
                let existing = st.find_stock_item(&name)?;
                let currency = st.locale.currency.clone();
                let unit_cost = match cost_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let (product_id, new_item_name, new_item_unit) = match existing {
                    Some(p) => (p.id, String::new(), String::new()),
                    None => (
                        String::new(),
                        name.trim().to_string(),
                        unit_text.trim().to_string(),
                    ),
                };
                let mut party_id = String::new();
                let mut party_name = String::new();
                let mut party_kind_send = String::new();
                if on_account {
                    party_kind_send = if party_kind.trim().is_empty() {
                        "Supplier".into()
                    } else {
                        party_kind.trim().into()
                    };
                    party_name = supplier.trim().to_string();
                    party_id = st
                        .customers
                        .iter()
                        .find(|c| c.active && c.name.trim().eq_ignore_ascii_case(supplier.trim()))
                        .map(|c| c.id.clone())
                        .unwrap_or_default();
                }
                let msg = st.books_op(BooksOp::RecordPurchase {
                    supplier: supplier.trim().to_string(),
                    party_id,
                    party_name,
                    party_kind: party_kind_send,
                    on_account,
                    product_id,
                    new_item_name,
                    new_item_unit,
                    qty_milli,
                    unit_cost_paise: unit_cost,
                    note: note.trim().to_string(),
                    batch_code: batch_code.trim().to_string(),
                    expires_on: expires_on.trim().to_string(),
                })?;
                let _ = st.refresh_books_overview();
                Ok(msg)
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(buy_qty_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(buy_cost_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(buy_batch_input)).set_text(cx, "");
                    self.ui.text_input(cx, ids!(buy_expiry_input)).set_text(cx, "");
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
                if item.view(cx, ids!(row)).finger_up(actions).is_some_and(finger_up_is_tap) {
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
                let _actor = st.actor()?;
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                let parts = settle_pay_parts(minor, &st.pay_parts, &currency)?;
                let msg = st.books_op(BooksOp::LogExpense {
                    amount_paise: minor,
                    note: note.clone(),
                    category: "misc".into(),
                    parts,
                })?;
                let status = match msg.split_once(books_op::EXPENSE_ID_MARK) {
                    Some((status, id)) => {
                        if st.device_role != DeviceRole::Client {
                            st.selected_expense_id = Some(id.to_string());
                            let _ = st.refresh_books_overview();
                        }
                        status.to_string()
                    }
                    None => msg,
                };
                st.pay_parts = vec![PayPart::default()];
                st.status = status;
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
                let paise = locale::parse_major_input(text.trim(), &st.locale.currency)?;
                let msg = st.books_op(BooksOp::SetOpeningFloat { amount_paise: paise })?;
                st.refresh_day_close_text()?;
                Ok(msg)
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
                let paise = locale::parse_major_input(counted.trim(), &st.locale.currency)
                    .map_err(|_| "Count the drawer and type the cash".to_string())?;
                let msg = st.books_op(BooksOp::CloseShift { counted_paise: paise })?;
                let (status, slip) = books_op::shift_slip_parts(&msg);
                let mut status = status.to_string();
                if let Some(slip) = slip {
                    let dir = st
                        .store
                        .as_ref()
                        .and_then(|s| s.db_path().parent().map(|p| p.to_path_buf()));
                    if let Some(note) = shift_slip_paper_note(print_share::print_text(
                        print_share::PrintKind::Bill,
                        slip,
                        dir.as_deref(),
                    )) {
                        status = format!("{status} · {note}");
                    }
                    st.last_receipt = slip.to_string();
                }
                Ok(status)
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
                let actor = st.actor()?;
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to close the day".into());
                }
                let currency = st.locale.currency.clone();
                let counted = match counted_text.trim() {
                    "" => None,
                    other => Some(locale::parse_major_input(other, &currency)?),
                };
                let msg = st.books_op(BooksOp::CommitDayClose {
                    counted_cash_paise: counted,
                })?;
                st.refresh_day_close_text()?;
                Ok(msg)
            })();
            match result {
                Err(e) => self.set_status(cx, e),
                Ok(msg) => self.set_status(cx, msg),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_pair_back)).clicked(actions)
            || self.ui.button(cx, ids!(onboard_back)).clicked(actions)
        {
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
                st.books_op(BooksOp::SetPlatformLanguage)?;
                st.language_follow_platform = true;
                if st.device_role == DeviceRole::Client {
                    st.locale.language = locale::detect_platform_language();
                }
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
                let platform = locale::detect_platform_language();
                st.books_op(BooksOp::SetUserLanguage { code: platform.clone() })?;
                st.language_follow_platform = false;
                if st.device_role == DeviceRole::Client {
                    st.locale.language = platform;
                }
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
        if self.ui.button(cx, ids!(device_role_cancel)).clicked(actions) {
            request_role_open_cancel();
            self.paint_role_open_busy_chrome(cx);
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
        // Rajdhani App Template — Submit → status + Cancel; Skip → Owner PIN.
        if self.ui.button(cx, ids!(app_template_apply)).clicked(actions) {
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                // Ignore while busy.
            } else {
                let job = (|| -> Result<(PathBuf, String, String), String> {
                    let st = app_state().lock().unwrap();
                    if !st.awaiting_app_template {
                        return Err("App template is not offered now".into());
                    }
                    let tmpl = menu_templates::find("rajdhani")
                        .ok_or_else(|| "Rajdhani menu template missing".to_string())?;
                    let db = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .db_path()
                        .to_path_buf();
                    Ok((db, tmpl.id.clone(), tmpl.title.clone()))
                })();
                match job {
                    Ok((db, id, title)) => {
                        menu_import_job(
                            db,
                            MenuImportKind::AppTemplate { id, title },
                        );
                        self.paint_import_progress_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
        }
        if self.ui.button(cx, ids!(app_template_cancel)).clicked(actions) {
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                request_books_job_cancel();
                self.paint_import_progress_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(app_template_status)).clicked(actions) {
            // Progress face — ignore re-taps while importing.
        }
        if self.ui.button(cx, ids!(app_template_skip)).clicked(actions) {
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                // Skip hidden while busy.
            } else if app_state().lock().unwrap().awaiting_app_template {
                // Skip does not mark the template done — unfinished shops re-offer.
                self.finish_app_template_to_lock(cx, false);
            }
        }
        {
            let widget = self.ui.widget(cx, ids!(setup_server_list));
            let list = widget.portal_list(cx, ids!(list));
            for (item_id, item) in list.items_with_actions(actions) {
                let id = app_state()
                    .lock()
                    .unwrap()
                    .local_servers
                    .get(item_id)
                    .map(|s| s.id.clone());
                if item.button(cx, ids!(pick)).clicked(actions) {
                    if let Some(id) = id.clone() {
                        if let Err(e) = self.activate_existing_server(cx, &id) {
                            self.set_status(cx, e);
                        }
                    }
                }
                if item.button(cx, ids!(remove)).clicked(actions) {
                    if let Some(id) = id {
                        if let Err(e) = self.delete_existing_server(cx, &id) {
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
        if self.ui.button(cx, ids!(copy_pair_code)).clicked(actions) {
            // Digits are a WrapLabel (no soft keyboard) — Copy puts them on the clipboard.
            let code = {
                let st = app_state().lock().unwrap();
                st.store
                    .as_ref()
                    .and_then(|s| s.pairing_code().ok())
                    .unwrap_or_default()
            };
            if code.trim().is_empty() || code == "------" {
                self.set_status(cx, app_t("lan_tills_off"));
            } else {
                self.copy_text_to_clipboard(cx, &code);
            }
        }
        if self.ui.button(cx, ids!(new_pair_code)).clicked(actions) {
            // Devices already paired keep working; the old code pairs no more.
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                st.require_server_host("Change the pairing code")?;
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
                    st.require_server_host("Forget a device")?;
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
            let clear_status = {
                let mut st = app_state().lock().unwrap();
                st.pin_buffer = digits;
                if st.status.starts_with("Wrong")
                    || st.status.starts_with("Enter at least")
                    || st.status.starts_with("Storage")
                    || st.status.starts_with("Add staff")
                {
                    st.status.clear();
                    true
                } else {
                    false
                }
            };
            // Digits only — never full sync_chrome (SQL + Settings rebuild froze unlock).
            if clear_status {
                self.ui
                    .widget(cx, ids!(pin_status))
                    .set_text(cx, " ");
            }
        }
        if self.ui.button(cx, ids!(pin_reveal)).clicked(actions) {
            let show = {
                let mut st = app_state().lock().unwrap();
                st.pin_visible = !st.pin_visible;
                st.pin_visible
            };
            self.ui
                .text_input(cx, ids!(lock_pin_input))
                .set_is_password(cx, !show);
            self.ui
                .button(cx, ids!(pin_reveal))
                .set_text(cx, eye_glyph(show));
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
        if self.choice_chip_clicked(cx, actions, ids!(role_owner), ids!(role_owner_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.lock_as_owner = true;
                st.pin_buffer.clear();
                st.status = match st.store.as_ref() {
                    Some(store) => match store.set_lock_role_prefer_owner(true) {
                        Ok(()) => "Unlock as Owner".into(),
                        Err(e) => e,
                    },
                    None => "Unlock as Owner".into(),
                };
            }
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(role_staff), ids!(role_staff_on)) {
            {
                let mut st = app_state().lock().unwrap();
                let has_staff = st.staff.iter().any(|s| !s.is_owner());
                if !st.owner_setup_complete || !has_staff {
                    st.lock_as_owner = true;
                    st.status = "Set up Owner first, then add staff in Team".into();
                } else {
                    st.lock_as_owner = false;
                    if st.lock_staff_id.is_none() {
                        st.lock_staff_id = st
                            .staff
                            .iter()
                            .find(|s| !s.is_owner())
                            .map(|s| s.id.clone());
                    }
                    st.pin_buffer.clear();
                    st.status = match st.store.as_ref() {
                        Some(store) => match store.set_lock_role_prefer_owner(false) {
                            Ok(()) => "Unlock as Staff".into(),
                            Err(e) => e,
                        },
                        None => "Unlock as Staff".into(),
                    };
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(lock_staff_pick)).clicked(actions) {
            self.paint_lock_staff_sheet(cx);
            self.ui.modal(cx, ids!(lock_staff_modal)).open(cx);
        }
        if self.ui.button(cx, ids!(lock_staff_sheet_done)).clicked(actions)
            || self.ui.modal(cx, ids!(lock_staff_modal)).dismissed(actions)
        {
            self.ui.modal(cx, ids!(lock_staff_modal)).close(cx);
        }
        for (slot, path) in [
            (0usize, ids!(lock_s0)),
            (1, ids!(lock_s1)),
            (2, ids!(lock_s2)),
            (3, ids!(lock_s3)),
            (4, ids!(lock_s4)),
            (5, ids!(lock_s5)),
            (6, ids!(lock_s6)),
            (7, ids!(lock_s7)),
            (8, ids!(lock_s8)),
            (9, ids!(lock_s9)),
            (10, ids!(lock_s10)),
            (11, ids!(lock_s11)),
        ] {
            if self.ui.button(cx, path).clicked(actions) {
                {
                    let mut st = app_state().lock().unwrap();
                    let staff: Vec<(String, String)> = st
                        .staff
                        .iter()
                        .filter(|s| !s.is_owner())
                        .map(|s| (s.id.clone(), s.lock_label()))
                        .collect();
                    if let Some((id, _)) = staff.get(slot) {
                        st.lock_staff_id = Some(id.clone());
                        st.pin_buffer.clear();
                    }
                }
                self.ui.modal(cx, ids!(lock_staff_modal)).close(cx);
                self.sync_chrome(cx);
            }
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
            // Pay runs before the cart portal loop — commit open-line edits first.
            self.commit_open_cart_line_fields(cx);
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
                // Loyalty / wallet / voucher: discount already on the cart;
                // Server till debits inside finalize's transaction (same as
                // Client /sale redeem fields on the Server).
                let pending_pts = st.pending_loyalty_points;
                let pending_wallet = st.pending_wallet_paise;
                let pending_v_code = st.pending_voucher_code.clone();
                let pending_v_paise = st.pending_voucher_paise;
                if (pending_pts > 0 || pending_wallet > 0 || pending_v_paise > 0)
                    && st.selected_customer_id.is_none()
                {
                    return Err("Pick a customer to redeem loyalty, wallet or voucher".into());
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
                        redeem_loyalty_points: pending_pts,
                        redeem_wallet_paise: pending_wallet,
                        redeem_voucher_code: pending_v_code,
                        redeem_voucher_paise: pending_v_paise,
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
                    let redeem = crate::store::SaleRedeem {
                        loyalty_points: pending_pts,
                        wallet_paise: pending_wallet,
                        voucher_code: pending_v_code,
                        voucher_paise: pending_v_paise,
                    };
                    store.finalize_sale_tenders_redeem(
                        &draft,
                        &parts,
                        table_id.as_deref(),
                        customer_id.as_deref(),
                        &actor,
                        &redeem,
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
                    st.selected_customer_id = Some(customer.id);
                    st.status = format!("Credit customer: {}", customer.name);
                    return Ok(());
                }
                // Create via books (Server local apply or Client LAN).
                st.books_op(BooksOp::AddParty {
                    name: needle.to_string(),
                    phone: String::new(),
                    birthday: String::new(),
                    kind: String::new(),
                })?;
                if let Some(customer) = st
                    .customers
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(needle))
                    .cloned()
                {
                    st.selected_customer_id = Some(customer.id);
                    st.status = format!("Credit customer: {}", customer.name);
                } else {
                    // Client: create is on the shop; list refreshes after Sync.
                    st.status = format!("Added {needle}");
                }
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
                if let Some(id) = st.selected_customer_id.clone() {
                    st.books_op(BooksOp::UpdateParty {
                        id,
                        name: name.trim().to_string(),
                        phone,
                        birthday,
                        kind: kind.trim().to_string(),
                    })
                } else {
                    let before: std::collections::HashSet<String> =
                        st.customers.iter().map(|c| c.id.clone()).collect();
                    let msg = st.books_op(BooksOp::AddParty {
                        name: name.trim().to_string(),
                        phone,
                        birthday,
                        kind: kind.trim().to_string(),
                    })?;
                    if st.device_role != DeviceRole::Client
                        && let Some(id) = st
                            .customers
                            .iter()
                            .find(|c| !before.contains(&c.id))
                            .map(|c| c.id.clone())
                    {
                        st.selected_customer_id = Some(id);
                    }
                    Ok(msg)
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
                let currency = st.locale.currency.clone();
                let limit_paise = if limit_text.trim().is_empty() {
                    0
                } else {
                    locale::parse_major_input(&limit_text, &currency)?
                };
                let (customer_id, name) = if !named.trim().is_empty() {
                    let id = st
                        .customers
                        .iter()
                        .find(|c| c.active && c.name.trim().eq_ignore_ascii_case(named.trim()))
                        .map(|c| c.id.clone())
                        .unwrap_or_default();
                    (id, named.trim().to_string())
                } else {
                    let id = st
                        .selected_customer_id
                        .clone()
                        .ok_or_else(|| "Pick a party or type a name".to_string())?;
                    let name = st
                        .customers
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.name.clone())
                        .unwrap_or_default();
                    (id, name)
                };
                st.books_op(BooksOp::SetCreditLimit {
                    customer_id,
                    name,
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
                    let msg = st.books_op(BooksOp::SettlePayable {
                        party_id,
                        amount_paise: amount,
                        pay: method,
                        against_bill_id: against,
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

        for (idle, on, seg) in [
            (
                ids!(crm_seg_bday_week),
                ids!(crm_seg_bday_week_on),
                store::CrmSegment::BirthdayThisWeek,
            ),
            (
                ids!(crm_seg_bday_month),
                ids!(crm_seg_bday_month_on),
                store::CrmSegment::BirthdayThisMonth,
            ),
            (
                ids!(crm_seg_phone),
                ids!(crm_seg_phone_on),
                store::CrmSegment::WithPhone,
            ),
            (
                ids!(crm_seg_loyalty),
                ids!(crm_seg_loyalty_on),
                store::CrmSegment::WithLoyalty,
            ),
            (
                ids!(crm_seg_inactive),
                ids!(crm_seg_inactive_on),
                store::CrmSegment::InactiveDays(30),
            ),
            (
                ids!(crm_seg_top),
                ids!(crm_seg_top_on),
                store::CrmSegment::TopSpenders(20),
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, on) {
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
                    let mut st = app_state().lock().unwrap();
                    let seg = st.crm_segment;
                    let (kind, inactive_days, top_n) = match seg {
                        store::CrmSegment::BirthdayThisWeek => ("birthday_week", 0i64, 0usize),
                        store::CrmSegment::BirthdayThisMonth => ("birthday_month", 0, 0),
                        store::CrmSegment::WithPhone => ("phone", 0, 0),
                        store::CrmSegment::WithLoyalty => ("loyalty", 0, 0),
                        store::CrmSegment::InactiveDays(days) => ("inactive", days, 0),
                        store::CrmSegment::TopSpenders(n) => ("top", 0, n),
                    };
                    st.books_op(BooksOp::CrmCampaign {
                        kind: kind.to_string(),
                        inactive_days,
                        top_n,
                        body,
                        channel: channel.to_string(),
                    })
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
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Not allowed to top up wallets".into());
                }
                let cid = st
                    .selected_customer_id
                    .clone()
                    .ok_or_else(|| "Pick a customer first".to_string())?;
                let paise = locale::parse_major_input(amt.trim(), &st.locale.currency)?;
                st.books_op(BooksOp::WalletTopUp {
                    customer_id: cid,
                    amount_paise: paise,
                })
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
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Not allowed to issue vouchers".into());
                }
                let paise = locale::parse_major_input(amt.trim(), &st.locale.currency)?;
                st.books_op(BooksOp::IssueVoucher {
                    code: code.trim().to_string(),
                    amount_paise: paise,
                })
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
                    // Leaving a table desk: drop that ticket so a later
                    // Choose-a-table cannot seat those lines onto another.
                    if kind != OrderType::DineIn && st.selected_table_id.take().is_some() {
                        st.cart.clear();
                        st.open_cart_line = None;
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
        if self.choice_chip_clicked(cx, actions, ids!(menu_schedule_h), ids!(menu_schedule_h_on)) {
            let mut st = app_state().lock().unwrap();
            st.form_schedule_h = !st.form_schedule_h;
            drop(st);
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(menu_rx_required), ids!(menu_rx_required_on)) {
            let mut st = app_state().lock().unwrap();
            st.form_rx_required = !st.form_rx_required;
            drop(st);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(template_import)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                st.template_confirm = true;
                st.client_template_confirm = None;
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(template_no)).clicked(actions) {
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                request_books_job_cancel();
                self.paint_import_progress_chrome(cx);
            } else {
                app_state().lock().unwrap().template_confirm = false;
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(template_yes)).clicked(actions) {
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                // Progress button — ignore re-taps while the worker runs.
            } else {
                let job = (|| -> Result<Option<PathBuf>, String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_items() {
                        return Err("Not allowed to manage items".into());
                    }
                    if st.device_role == DeviceRole::Client {
                        st.books_op(BooksOp::ImportShelfTemplate)?;
                        st.template_confirm = false;
                        return Ok(None);
                    }
                    Ok(Some(
                        st.store
                            .as_ref()
                            .ok_or_else(|| "No store".to_string())?
                            .db_path()
                            .to_path_buf(),
                    ))
                })();
                match job {
                    Ok(None) => {
                        self.set_status(cx, "Importing the menu template on the shop device…");
                        self.sync_chrome(cx);
                    }
                    Ok(Some(db)) => {
                        app_state().lock().unwrap().template_confirm = true;
                        menu_import_job(db, MenuImportKind::Shelf);
                        // Progress only — full sync_chrome would hit SQLite while the worker writes.
                        self.paint_import_progress_chrome(cx);
                    }
                    Err(e) => {
                        app_state().lock().unwrap().template_confirm = false;
                        self.set_status(cx, e);
                        self.sync_chrome(cx);
                    }
                }
            }
        }
        // Named client templates (assets/templates/<id>/).
        {
            let named = menu_templates::all();
            let chip_slots = [
                (ids!(client_tmpl_0), ids!(client_tmpl_0_on)),
                (ids!(client_tmpl_1), ids!(client_tmpl_1_on)),
                (ids!(client_tmpl_2), ids!(client_tmpl_2_on)),
            ];
            for (i, (idle, on)) in chip_slots.iter().enumerate() {
                if named.get(i).is_some()
                    && self.choice_chip_clicked(cx, actions, *idle, *on)
                {
                    {
                        let mut st = app_state().lock().unwrap();
                        st.client_template_id = named[i].id.clone();
                        st.client_template_confirm = None;
                        st.template_confirm = false;
                    }
                    self.sync_chrome(cx);
                    break;
                }
            }
        }
        if self.ui.button(cx, ids!(client_template_import)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                let id = st.client_template_id.clone();
                if !id.is_empty() {
                    st.client_template_confirm = Some(id);
                    st.template_confirm = false;
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_template_no)).clicked(actions) {
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                request_books_job_cancel();
                self.paint_import_progress_chrome(cx);
            } else {
                app_state().lock().unwrap().client_template_confirm = None;
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(client_template_yes)).clicked(actions) {
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                // Progress button — ignore re-taps while the worker runs.
            } else {
                let job = (|| -> Result<Option<(PathBuf, String, String)>, String> {
                    let mut st = app_state().lock().unwrap();
                    let id = st
                        .client_template_confirm
                        .clone()
                        .unwrap_or_else(|| st.client_template_id.clone());
                    if id.is_empty() {
                        return Err("Pick a client menu first".into());
                    }
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_items() {
                        return Err("Not allowed to manage items".into());
                    }
                    let tmpl = menu_templates::find(&id)
                        .ok_or_else(|| format!("Unknown menu template '{id}'"))?;
                    if st.device_role == DeviceRole::Client {
                        st.books_op(BooksOp::ImportClientTemplate {
                            template_id: tmpl.id.clone(),
                        })?;
                        st.client_template_confirm = None;
                        st.template_confirm = false;
                        return Ok(None);
                    }
                    let db = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .db_path()
                        .to_path_buf();
                    Ok(Some((db, tmpl.id.clone(), tmpl.title.clone())))
                })();
                match job {
                    Ok(None) => {
                        self.set_status(cx, "Importing a menu template on the shop device…");
                        self.sync_chrome(cx);
                    }
                    Ok(Some((db, id, title))) => {
                        {
                            let mut st = app_state().lock().unwrap();
                            st.client_template_confirm = Some(id.clone());
                            st.template_confirm = false;
                        }
                        menu_import_job(db, MenuImportKind::Client { id, title });
                        // Progress only — do not sync_chrome (SQLite) while the worker runs.
                        self.paint_import_progress_chrome(cx);
                    }
                    Err(e) => {
                        app_state().lock().unwrap().client_template_confirm = None;
                        self.set_status(cx, e);
                        self.sync_chrome(cx);
                    }
                }
            }
        }
        if self.ui.button(cx, ids!(table_pick_btn)).clicked(actions) {
            self.pick_table_for_order(cx);
        }
        if self.ui.button(cx, ids!(floor_pick_back)).clicked(actions) {
            self.cancel_table_pick(cx);
        }
        // Floor area tabs — each owns its Columns + table body.
        // Slot order follows drag-to-swap (`floor_area_tab_names`).
        {
            let areas = self.floor_area_tab_names();
            let mut picked: Option<String> = None;
            for (i, (idle, on)) in FLOOR_AREA_CHIPS.iter().enumerate() {
                if self.choice_chip_clicked(cx, actions, idle, on)
                    && let Some(area) = areas.get(i)
                {
                    picked = Some(area.clone());
                }
            }
            if let Some(area) = picked {
                self.dismiss_soft_keyboard(cx);
                self.select_floor_area_tab(cx, area);
                // Category chips re-sync chrome after the pick so the active
                // face is painted in the same event — do the same here.
                self.sync_chrome(cx);
            }
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
            if self.choice_chip_clicked(cx, actions, ids!(cat_added), ids!(cat_added_on)) {
                picked = Some(Some(MENU_ADDED.to_string()));
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
                self.dismiss_soft_keyboard(cx);
                let (from_idx, to_idx, fill_frequent) = {
                    let mut st = app_state().lock().unwrap();
                    let keys = menu_pick_tab_keys(&st);
                    let from_idx = menu_pick_tab_index(&keys, &st.menu_category);
                    st.menu_category = cat;
                    let to_idx = menu_pick_tab_index(&keys, &st.menu_category);
                    let fill = if st.menu_category.as_deref() == Some(MENU_FREQUENT) {
                        st.refresh_frequent_cache();
                        Some((st.frequent_days, st.frequent_limit))
                    } else {
                        None
                    };
                    if from_idx == to_idx {
                        st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
                    }
                    (from_idx, to_idx, fill)
                };
                if let Some((days, limit)) = fill_frequent {
                    self.ui
                        .text_input(cx, ids!(frequent_days_input))
                        .set_text(cx, &days.to_string());
                    self.ui
                        .text_input(cx, ids!(frequent_limit_input))
                        .set_text(cx, &limit.to_string());
                }
                let searching = !app_state().lock().unwrap().menu_filter.trim().is_empty();
                if searching || from_idx == to_idx {
                    self.snap_menu_pick_scroll_to_category(cx);
                } else {
                    self.start_menu_pick_scroll_anim(cx, from_idx, to_idx);
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
                let msg = st.books_op(BooksOp::SetFrequent { days, limit })?;
                let text = if st.device_role == DeviceRole::Client {
                    msg
                } else {
                    st.refresh_frequent_cache();
                    st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
                    let count = st.frequent_ids.len();
                    st.t("frequent_saved")
                        .replace("{d}", &st.frequent_days.to_string())
                        .replace("{n}", &st.frequent_limit.to_string())
                        .replace("{c}", &count.to_string())
                };
                let shown_days = if st.device_role == DeviceRole::Client {
                    days
                } else {
                    st.frequent_days
                };
                let shown_limit = if st.device_role == DeviceRole::Client {
                    limit
                } else {
                    st.frequent_limit
                };
                Ok((text, shown_days, shown_limit))
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
                let msg = st.books_op(BooksOp::EnsureFloorArea {
                    name: name.to_string(),
                })?;
                Ok(if st.device_role == DeviceRole::Client {
                    msg
                } else {
                    st.t("area_added").replace("{name}", name)
                })
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
        if self.ui.button(cx, ids!(print_bill_btn)).clicked(actions)
            || self.ui.button(cx, ids!(print_bill_quick)).clicked(actions)
        {
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
                match st.table_action(
                    net::RunningAction::FireCourse {
                        table_id: tid,
                        course_no: 1,
                    },
                    "",
                    "Fired course 1",
                    phone::Follow::Refresh,
                )? {
                    Acted::Sent => Ok("Firing course 1…".into()),
                    Acted::Here(_) => Ok("Fired course 1".into()),
                }
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
                match st.table_action(
                    net::RunningAction::HoldLine {
                        line_id,
                        held: true,
                    },
                    "",
                    "Line held",
                    phone::Follow::Refresh,
                )? {
                    Acted::Sent => Ok("Holding line…".into()),
                    Acted::Here(_) => {
                        Ok("Line held — Send KOT skips it until Fire course".into())
                    }
                }
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
                match st.table_action(
                    net::RunningAction::WaiterCall {
                        table_id: tid,
                        kind: "service".into(),
                    },
                    "",
                    "Waiter call queued",
                    phone::Follow::Refresh,
                )? {
                    Acted::Sent => Ok("Calling waiter…".into()),
                    Acted::Here(_) => Ok("Waiter call queued — clear from Floor ops".into()),
                }
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
            let done_uid = self.ui.widget(cx, ids!(menu_pick_done)).widget_uid();
            // Any press, click, or release on Done closes the dish window.
            // The first action on that widget is not always the press.
            let done = actions.filter_widget_actions(done_uid).any(|action| {
                matches!(
                    action.cast::<ButtonAction>(),
                    ButtonAction::Pressed(_)
                        | ButtonAction::Clicked(_)
                        | ButtonAction::Released(_)
                )
            });
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
            // Grid + empty label only — full sync_chrome every key made Android
            // IME and scroll feel broken.
            self.refresh_menu_pick_search(cx);
        }
        // Wedge scanner / Enter: exact barcode → add one to the bill.
        if let Some((text, _)) = self.ui.text_input(cx, ids!(menu_search_input)).returned(actions) {
            let code = text.trim().to_string();
            if !code.is_empty() {
                let hit = {
                    let st = app_state().lock().unwrap();
                    st.store
                        .as_ref()
                        .and_then(|s| s.find_product_by_barcode(&code).ok().flatten())
                        .or_else(|| {
                            st.products.iter().find(|p| p.matches_barcode(&code)).cloned()
                        })
                        .or_else(|| {
                            st.products.iter().find(|p| {
                                p.name.trim().eq_ignore_ascii_case(&code)
                                    || p.aliases().iter().any(|a| a.trim().eq_ignore_ascii_case(&code))
                            }).cloned()
                        })
                };
                if let Some(p) = hit {
                    self.add_product_by_id(cx, &p.id);
                    self.ui.text_input(cx, ids!(menu_search_input)).set_text(cx, "");
                    {
                        let mut st = app_state().lock().unwrap();
                        st.menu_filter.clear();
                        st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
                    }
                    self.refresh_menu_pick_search(cx);
                }
            }
        }
        // Customer paid / tip: the change line follows live.
        let tendered_changed = self.ui.text_input(cx, ids!(tendered_input)).changed(actions).is_some();
        let tip_changed = self.ui.text_input(cx, ids!(tip_input)).changed(actions).is_some();
        let cash_changed = self.ui.text_input(cx, ids!(mix_cash_input)).changed(actions).is_some();
        if tendered_changed || tip_changed || cash_changed {
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(kot_station_dd)).clicked(actions) {
            let empty_msg = {
                let mut st = app_state().lock().unwrap();
                if st.suggest_open == Some(SuggestField::KotStation) {
                    st.suggest_open = None;
                    None
                } else {
                    st.open_suggest(SuggestField::KotStation, "");
                    let n = st
                        .store
                        .as_ref()
                        .and_then(|s| s.list_picker_stations().ok())
                        .unwrap_or_else(|| st.stations.clone())
                        .len();
                    if n == 0 {
                        Some(st.t("kot_station_none"))
                    } else {
                        None
                    }
                }
            };
            if let Some(msg) = empty_msg {
                self.set_status(cx, msg);
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(printer_place_dd)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                if st.suggest_open == Some(SuggestField::PrinterPlace) {
                    st.suggest_open = None;
                } else {
                    st.open_suggest(SuggestField::PrinterPlace, "");
                }
            }
            self.ui
                .widget(cx, ids!(printer_place_suggest))
                .redraw(cx);
            self.ui.redraw(cx);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(printer_assign_dd)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                let _ = normalize_focused_route(&mut st);
                if st.suggest_open == Some(SuggestField::PrinterAssign) {
                    st.suggest_open = None;
                } else if st.printer_draft.printers.is_empty() {
                    st.suggest_open = None;
                } else {
                    st.open_suggest(SuggestField::PrinterAssign, "");
                }
            }
            let empty = {
                let st = app_state().lock().unwrap();
                st.printer_draft.printers.is_empty()
            };
            if empty {
                self.set_status(cx, app_t("printers_none_hint"));
            }
            self.ui
                .widget(cx, ids!(printer_assign_suggest))
                .redraw(cx);
            self.ui.redraw(cx);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(reprint_kot_btn)).clicked(actions) {
            match self.reprint_open_kots(cx) {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(batch_cancel_btn)).clicked(actions) {
            let n = {
                let st = app_state().lock().unwrap();
                st.cart
                    .lines
                    .iter()
                    .filter(|l| {
                        l.cancel_pick
                            && l.is_sent()
                            && !l.running_line_id.is_empty()
                            && l.running_line_id != "counter"
                            && !l.running_line_id.starts_with(QUEUED_LINE)
                    })
                    .count()
            };
            if n == 0 {
                self.set_status(cx, app_t("batch_cancel_none"));
            } else {
                {
                    let mut st = app_state().lock().unwrap();
                    st.line_action = Some((String::new(), LineAction::CancelBatch));
                    st.line_cooked = false;
                }
                self.ui.text_input(cx, ids!(line_reason_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(line_qty_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(line_pin_input)).set_text(cx, "");
                self.sync_chrome(cx);
            }
        }
        if self
            .ui
            .button(cx, ids!(print_cancelled_kot_btn))
            .clicked(actions)
        {
            match self.print_all_cancelled_kots(cx) {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(send_kot_btn)).clicked(actions) {
            // After a KOT is sent, the face reads "Add dishes for next KOT".
            // That must open the menu — never re-run Send with nothing new
            // (which only showed "Nothing new to send").
            let open_menu_instead = {
                let st = app_state().lock().unwrap();
                let has_unsent = st
                    .cart
                    .lines
                    .iter()
                    .any(|l| !l.is_sent() && !l.hold_send && l.qty_milli > 0);
                !has_unsent
            };
            if open_menu_instead {
                self.open_menu_window(cx);
                // Same status path as + Add dishes so the waiter sees why.
                self.set_status(cx, app_t("add_dishes"));
                self.sync_chrome(cx);
            } else {
            // Send runs before the cart portal loop — commit open-line edits.
            self.commit_open_cart_line_fields(cx);
            // Station already applied via checkbox dropdown taps — no type-in
            // field to re-apply on Send.
            // Only the new lines go; what the kitchen already has stays on
            // the table's running order. A waiter's tablet sends to the
            // Server (it used to write the ticket into its own cache, which
            // the next sync erased — the kitchen never saw it).
            let result = (|| -> Result<(SnackKind, String), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                st.require_cart_modifiers()?;
                // Person rights: refuse send if any pending line's kitchen
                // is outside this user's allowed stations.
                {
                    let feats = actor.effective_features();
                    let products = st.products.clone();
                    for line in st.cart.lines.iter().filter(|l| !l.is_sent() && !l.hold_send) {
                        let sid = line.kitchen_station_id(&products);
                        if sid.is_empty() {
                            continue;
                        }
                        if !feats.allows_kot_station(sid) {
                            let name = st
                                .stations
                                .iter()
                                .find(|s| s.id == sid)
                                .map(|s| s.name.clone())
                                .unwrap_or_else(|| sid.to_string());
                            return Err(st
                                .t("kot_station_not_allowed")
                                .replace("{station}", &name));
                        }
                    }
                }
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
                // Stations this Send will hit — persist before lines flip to sent.
                let used_stations: Vec<String> = {
                    let products = st.products.clone();
                    st.cart
                        .lines
                        .iter()
                        .filter(|l| !l.is_sent() && !l.hold_send)
                        .map(|l| l.kitchen_station_id(&products).to_string())
                        .filter(|sid| !sid.is_empty())
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect()
                };
                let msg = if st.device_role == DeviceRole::Client {
                    let table_name = table_id.as_ref().and_then(|tid| {
                        st.tables.iter().find(|t| &t.id == tid).map(|t| t.name.clone())
                    });
                    let place = table_name.clone().unwrap_or_else(|| st.t("counter"));
                    let say = |key: &str| {
                        st.t(key)
                            .replace("{place}", &place)
                            .replace("{kitchens}", &kitchens)
                    };
                    let (job_label, job_done, queued_msg) =
                        (say("kot_job_label"), say("kot_job_reached"), say("kot_queued_shop"));
                    // Kitchen paper prints on the Server (its Settings → Printers),
                    // not from this till's local printer list.
                    let job = st.lan_job(
                        "/kot",
                        &net::RemoteKotRequest { draft, table_name, table_id: table_id.clone() },
                        job_label,
                        job_done,
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
                        // Queued until the shop device answers. `is_sent` is
                        // true so Send moves on; reprint waits for a real id.
                        st.cart.mark_new_lines_sent(QUEUED_KOT);
                    }
                    for sid in &used_stations {
                        st.select_kot_station(sid);
                    }
                    // Queued until the shop device answers — a pending state, so
                    // amber, as before (it used to be read off "shop device").
                    return Ok((SnackKind::Warn, queued_msg));
                } else {
                    let kot_ids = {
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.send_kot(&draft, table_id.as_deref(), &actor)?
                    };
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let paper = net::print_sent_kots(store, &kot_ids);
                    let msg = kot_sent_status(
                        &|key: &str| st.t(key),
                        &kitchens,
                        store.kot_auto_print(),
                        paper.as_deref(),
                    );
                    for sid in &used_stations {
                        st.select_kot_station(sid);
                    }
                    match &table_id {
                        // The table's order now holds them: show them as sent.
                        Some(tid) => {
                            st.reload_all()?;
                            st.load_table_cart(tid)?;
                        }
                        // Counter order paid after: keep the ticket to charge,
                        // but the kitchen must not be told twice at payment.
                        None => {
                            let marker = kot_ids.first().map(String::as_str).unwrap_or("counter");
                            st.cart.mark_new_lines_sent(marker);
                        }
                    }
                    st.open_kots = st
                        .store
                        .as_ref()
                        .map(|s| s.list_open_kots().unwrap_or_default())
                        .unwrap_or_default();
                    msg
                };
                Ok(msg)
            })();
            match result {
                Ok((SnackKind::Error, msg)) => self.show_error(cx, msg),
                Ok((SnackKind::Warn, msg)) => self.show_warn(cx, msg),
                Ok((_, msg)) => self.show_ok(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
            } // else: had unsent lines → Send KOT
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
                    } else if tile.finger_up(actions).is_some_and(finger_up_is_tap) {
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
            // One selected area tab — idxs into the filtered table list.
            let (idxs, filtered_ids) = {
                let st = app_state().lock().unwrap();
                let tables = floor_filtered_tables(&st);
                let ids: Vec<String> = tables.iter().map(|t| t.id.clone()).collect();
                let tab = st.floor_area_tab.as_str();
                let groups = floor_area_groups(&tables);
                let idxs = groups
                    .iter()
                    .find(|(a, _)| a.as_str() == tab)
                    .map(|(_, i)| i.clone())
                    .or_else(|| groups.first().map(|(_, i)| i.clone()))
                    .unwrap_or_default();
                (idxs, ids)
            };
            let cols = floor_columns();
            let vp_w = floor_hviewport().lock().unwrap().1;
            // act | print | view | open (tile body tap)
            // Nested PortalList (area → rows) stamps the *inner* group on
            // actions first; outer `list.items_with_actions` then never
            // matches. Walk the single area item + every row slot directly
            // (same as FloorLayoutList) — button/finger_up key off uid.
            let mut tapped: Option<(usize, &'static str)> = None;
            if !idxs.is_empty() {
                let item = list.item(cx, 0, id!(Item));
                let scroll = item.view(cx, ids!(area_scroll));
                let scroll_x = scroll.get_scroll_pos().x;
                let area_vp_w = {
                    let r = scroll.area().rect(cx);
                    if r.size.x > 1.0 {
                        r.size.x
                    } else {
                        vp_w.max(1.0)
                    }
                };
                let (zoom, zoom_auto) = {
                    let st = app_state().lock().unwrap();
                    (st.floor_zoom, st.floor_zoom_auto)
                };
                let scale = floor_effective_scale(zoom_auto, zoom, cols, area_vp_w);
                let first_col = if cols <= FLOOR_ROW_SLOTS {
                    0
                } else {
                    floor_col_window(0.0, scroll_x, area_vp_w, cols, scale).0
                };
                let rows_list = item.portal_list(cx, ids!(area_rows));
                let n_rows = idxs.chunks(cols.max(1)).count().max(1);
                for row_id in 0..n_rows {
                    let row_item = rows_list.item(cx, row_id, id!(Item));
                    let chunk_start = row_id * cols.max(1);
                    for (slot, path) in FLOOR_SLOT_IDS.iter().enumerate() {
                        let Some(&table_idx) = idxs.get(chunk_start + first_col + slot) else {
                            break;
                        };
                        let tile = row_item.view(cx, *path);
                        if tile.button(cx, ids!(act)).clicked(actions) {
                            tapped = Some((table_idx, "act"));
                        } else if tile.button(cx, ids!(print_ico)).clicked(actions) {
                            tapped = Some((table_idx, "print"));
                        } else if tile.button(cx, ids!(view_ico)).clicked(actions) {
                            tapped = Some((table_idx, "view"));
                        } else if tile.finger_up(actions).is_some_and(finger_up_is_tap) {
                            // Tap opens. Sideways press-drag (tiles / gaps /
                            // pads) scrolls after axis_lock ~8px and takes
                            // capture — not was_tap() which dies on a 6px
                            // jitter / slow press.
                            tapped = Some((table_idx, "open"));
                        }
                    }
                }
            }
            if let Some((filt_idx, kind)) = tapped {
                let idx = {
                    let st = app_state().lock().unwrap();
                    filtered_ids
                        .get(filt_idx)
                        .and_then(|id| st.tables.iter().position(|t| &t.id == id))
                };
                if let Some(idx) = idx {
                    {
                        let mut st = app_state().lock().unwrap();
                        if let Some(t) = st.tables.get(idx) {
                            st.floor_focus_table_id = Some(t.id.clone());
                        }
                    }
                    let info = app_state()
                        .lock()
                        .unwrap()
                        .tables
                        .get(idx)
                        .map(|t| (t.floor_state, t.running_total_paise));
                    match (kind, info) {
                        ("act", Some((domain::FloorState::Free, _))) => {
                            self.seat_table(cx, idx)
                        }
                        ("act", Some((domain::FloorState::Seated, _))) => {
                            self.mark_table_ready(cx, idx)
                        }
                        ("act", Some((domain::FloorState::Paid, _))) => {
                            self.mark_table_ready(cx, idx)
                        }
                        ("act", Some((domain::FloorState::BillPrinted, amount))) if amount <= 0 => {
                            self.mark_table_ready(cx, idx)
                        }
                        ("act", Some((domain::FloorState::BillPrinted, _))) => {
                            self.undo_table_bill(cx, idx)
                        }
                        ("act", Some(_)) => self.request_table_bill(cx, idx),
                        ("print", Some(_)) => self.print_floor_table_bill(cx, idx),
                        ("view", Some(_)) => self.view_floor_table_bill(cx, idx),
                        ("open", Some(_)) => self.open_table(cx, idx),
                        _ => {}
                    }
                }
            }
        }
        // Floor columns: pick-only dropdown (no soft keyboard). List pick
        // applies above; the button toggles the floating list.
        if self.ui.button(cx, ids!(floor_cols_dd)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                if st.suggest_open == Some(SuggestField::FloorColumns) {
                    st.suggest_open = None;
                } else {
                    st.open_suggest(SuggestField::FloorColumns, "");
                }
            }
            self.sync_chrome(cx);
        }
        // Floor zoom − / + (turns Auto off) and Auto fill-width.
        if self.ui.button(cx, ids!(floor_zoom_out)).clicked(actions) {
            self.nudge_floor_zoom(cx, -FLOOR_ZOOM_STEP);
        }
        if self.ui.button(cx, ids!(floor_zoom_in)).clicked(actions) {
            self.nudge_floor_zoom(cx, FLOOR_ZOOM_STEP);
        }
        if self.choice_chip_clicked(cx, actions, ids!(floor_zoom_auto), ids!(floor_zoom_auto_on))
        {
            let on = !app_state().lock().unwrap().floor_zoom_auto;
            self.set_floor_zoom_auto(cx, on);
        }
        // Floor toolbar: Print / View / Open for the focused table.
        if self.ui.button(cx, ids!(floor_tb_print)).clicked(actions) {
            if let Some(idx) = self.floor_focus_index() {
                self.print_floor_table_bill(cx, idx);
            }
        }
        if self.ui.button(cx, ids!(floor_tb_view)).clicked(actions) {
            if let Some(idx) = self.floor_focus_index() {
                self.view_floor_table_bill(cx, idx);
            }
        }
        if self.ui.button(cx, ids!(floor_tb_open)).clicked(actions) {
            if let Some(idx) = self.floor_focus_index() {
                self.open_table(cx, idx);
            }
        }
        if self.ui.button(cx, ids!(floor_bill_close)).clicked(actions) {
            self.ui.view(cx, ids!(floor_bill_view)).set_visible(cx, false);
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
                        .is_some_and(finger_up_is_tap);
                if tapped {
                    let result = (|| -> Result<(), String> {
                        let mut st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        let (kot_id, from) = st
                            .open_kots
                            .get(item_id)
                            .map(|k| (k.id.clone(), k.status.as_str().to_string()))
                            .ok_or_else(|| "Ticket gone".to_string())?;
                        // A kitchen tablet moves the Server's ticket (writing
                        // its own cache was undone by the next sync). `from`
                        // keeps a double tap on slow Wi‑Fi to one step.
                        if st.device_role == DeviceRole::Client {
                            st.lan_job(
                                "/kot_advance",
                                &net::RemoteKotAdvance { kot_id, from },
                                "Kitchen ticket".into(),
                                "Ticket moved on".into(),
                                phone::Follow::Refresh,
                                false,
                            )?;
                        } else {
                            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                            // A waiter's KOT can re-sort the list under the
                            // finger: only move the ticket from what it showed.
                            store.advance_kot_from(
                                &kot_id,
                                Some(crate::domain::KotStatus::parse(&from)),
                                &actor,
                            )?;
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
                let (prefix, from, to) = parse_table_range(&range)?;
                st.books_op(BooksOp::AddTablesRange {
                    prefix,
                    from,
                    to,
                    area,
                })
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
        if self.ui.button(cx, ids!(table_remove_btn)).clicked(actions) {
            let typed = self.ui.text_input(cx, ids!(table_remove_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let names: Vec<String> = match parse_table_range(&typed) {
                    Ok((prefix, from, to)) => (from..=to).map(|n| format!("{prefix}{n}")).collect(),
                    Err(_) if !typed.trim().is_empty() && !typed.contains('-') => vec![typed.trim().to_string()],
                    Err(e) => return Err(e),
                };
                st.books_op(BooksOp::RemoveTables { names })
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(table_remove_input)).set_text(cx, "");
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
                st.books_op(BooksOp::AddTable { name, area })
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
                let trimmed = name.trim().to_string();
                let msg = st.books_op(BooksOp::AddStation {
                    name: trimmed.clone(),
                })?;
                if st.device_role != DeviceRole::Client
                    && let Some(id) = st
                        .stations
                        .iter()
                        .find(|s| s.name.eq_ignore_ascii_case(&trimmed))
                        .map(|s| s.id.clone())
                {
                    st.printer_draft.ensure_station_routes([&id]);
                    let _ = st.save_printer_draft();
                }
                Ok(msg)
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
        if self.ui.button(cx, ids!(station_remove)).clicked(actions)
            || self
                .ui
                .text_input(cx, ids!(station_remove_input))
                .returned(actions)
                .is_some()
        {
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
        if self.ui.button(cx, ids!(station_vps_dd)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let key_ok = {
                    let st = app_state().lock().unwrap();
                    st.printer_route_key.trim().starts_with("kot:")
                        && st.printer_route_key.trim() != "kot:"
                };
                if !key_ok {
                    self.set_status(cx, app_t("station_print_pick_first"));
                } else {
                    {
                        let mut st = app_state().lock().unwrap();
                        if st.suggest_open == Some(SuggestField::StationVps) {
                            st.suggest_open = None;
                        } else {
                            st.open_suggest(SuggestField::StationVps, "");
                        }
                    }
                    self.ui.widget(cx, ids!(station_vps_suggest)).redraw(cx);
                    self.paint_station_print_attach(cx);
                }
            }
        }
        for (idle, on, mode) in [
            (
                ids!(station_print_first),
                ids!(station_print_first_on),
                print_share::RoutePrintMode::First,
            ),
            (
                ids!(station_print_all),
                ids!(station_print_all_on),
                print_share::RoutePrintMode::All,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, on) {
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    if st.device_role == DeviceRole::Client {
                        return Err(st.t("printers_host_only"));
                    }
                    let key_str = st.printer_route_key.clone();
                    if !key_str.trim().starts_with("kot:") || key_str.trim() == "kot:" {
                        return Err(st.t("station_print_pick_first"));
                    }
                    let key = parse_printer_route_key(&key_str);
                    st.printer_draft.set_print_mode(key, mode);
                    st.save_printer_draft()?;
                    Ok(st.t(match mode {
                        print_share::RoutePrintMode::All => "printer_mode_all",
                        print_share::RoutePrintMode::First => "printer_mode_first",
                    }))
                })();
                match result {
                    Ok(msg) => self.set_status(cx, msg),
                    Err(e) => self.set_status(cx, e),
                }
                self.paint_station_print_attach(cx);
                break;
            }
        }
        if self.ui.button(cx, ids!(settings_station_add)).clicked(actions) {
            let name = self
                .ui
                .text_input(cx, ids!(settings_station_name_input))
                .text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let trimmed = name.trim().to_string();
                let msg = st.books_op(BooksOp::AddStation {
                    name: trimmed.clone(),
                })?;
                if st.device_role != DeviceRole::Client
                    && let Some(id) = st
                        .stations
                        .iter()
                        .find(|s| s.name.eq_ignore_ascii_case(&trimmed))
                        .map(|s| s.id.clone())
                {
                    st.printer_draft.ensure_station_routes([&id]);
                    let _ = st.save_printer_draft();
                }
                Ok(msg)
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
        if self.ui.button(cx, ids!(settings_station_remove)).clicked(actions)
            || self
                .ui
                .text_input(cx, ids!(settings_station_remove_input))
                .returned(actions)
                .is_some()
        {
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
                (ids!(settings_hub_selling), "selling"),
                (ids!(settings_hub_alerts), "alerts"),
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
                self.open_settings_panel(cx, Some(key));
            } else if self.ui.button(cx, ids!(settings_detail_back)).clicked(actions) {
                self.open_settings_panel(cx, None);
            }
        }
        self.handle_diag_actions(cx, actions);
        // "Type or pick" fields (suggest.rs, rules in popup.rs): a tap on the
        // field opens the list (`route_popup_input`), typing narrows it, a
        // tap on a row fills the field, Escape / Back / an outside press /
        // focusing another field closes it. Focus alone opens nothing — a
        // modal closing hands focus back to its field and reopened the list.
        {
            let open = app_state().lock().unwrap().suggest_open;
            // Typing only narrows the open list — never full sync_chrome (IME /
            // Settings SQL freeze). Picks, Escape, and focus-elsewhere still
            // need chrome paint.
            let mut typed = false;
            let mut needs_chrome = false;
            let mut picked: Option<(SuggestField, String)> = None;
            for field in SuggestField::ALL {
                // Kitchen / place / printer-assign / team stations open via
                // chrome (dd or always-open), not type-in.
                if !field.opens_by_chrome() {
                    let input = self.ui.text_input(cx, field.input());
                    if let Some(text) = input.changed(actions) {
                        app_state().lock().unwrap().open_suggest(field, &text);
                        typed = true;
                    } else if input.escaped(actions) && open == Some(field) {
                        app_state().lock().unwrap().suggest_open = None;
                        needs_chrome = true;
                    }
                }
                if open == Some(field) {
                    let widget = self.ui.widget(cx, field.list());
                    let item_actions: Vec<(usize, _)> = widget
                        .portal_list(cx, ids!(list))
                        .items_with_actions(actions);
                    for (item_id, item) in item_actions {
                        if field == SuggestField::KotStation {
                            // ↑↓ swap with neighbouring *station* (skip Unassigned).
                            // Only the glyph buttons reorder — never the row body.
                            let up = item.button(cx, ids!(prio_up));
                            let down = item.button(cx, ids!(prio_down));
                            if up.clicked(actions) {
                                let to = {
                                    let st = app_state().lock().unwrap();
                                    let u = st.t("kot_unassigned");
                                    (0..item_id).rev().find(|&i| {
                                        st.suggest_rows
                                            .get(i)
                                            .is_some_and(|r| !r.value.eq_ignore_ascii_case(&u))
                                    })
                                };
                                if let Some(to) = to {
                                    self.reorder_kot_station_priority(cx, item_id, to);
                                    needs_chrome = true;
                                }
                                continue;
                            }
                            if down.clicked(actions) {
                                let to = {
                                    let st = app_state().lock().unwrap();
                                    let u = st.t("kot_unassigned");
                                    let n = st.suggest_rows.len();
                                    ((item_id + 1)..n).find(|&i| {
                                        st.suggest_rows
                                            .get(i)
                                            .is_some_and(|r| !r.value.eq_ignore_ascii_case(&u))
                                    })
                                };
                                if let Some(to) = to {
                                    self.reorder_kot_station_priority(cx, item_id, to);
                                    needs_chrome = true;
                                }
                                continue;
                            }
                            // Press/release on ↑↓ (even without a clean click)
                            // must not also toggle via the row finger_up.
                            if up.pressed(actions)
                                || down.pressed(actions)
                                || up.released(actions)
                                || down.released(actions)
                            {
                                continue;
                            }
                        }
                        if field == SuggestField::StationVps || field == SuggestField::BillPrinters {
                            let up = item.button(cx, ids!(prio_up));
                            let down = item.button(cx, ids!(prio_down));
                            if up.clicked(actions) {
                                let to = {
                                    let st = app_state().lock().unwrap();
                                    let clear_l = st.t("dont_use");
                                    (0..item_id).rev().find(|&i| {
                                        st.suggest_rows
                                            .get(i)
                                            .is_some_and(|r| r.action == clear_l)
                                    })
                                };
                                if let Some(to) = to {
                                    let mut st = app_state().lock().unwrap();
                                    if reorder_station_virtual_priority(&mut st, item_id, to) {
                                        st.open_suggest(field, "");
                                        drop(st);
                                        self.paint_station_print_attach(cx);
                                        self.sync_job_drive_ui(cx);
                                        self.ui.widget(cx, field.list()).redraw(cx);
                                    }
                                }
                                continue;
                            }
                            if down.clicked(actions) {
                                let to = {
                                    let st = app_state().lock().unwrap();
                                    let clear_l = st.t("dont_use");
                                    let n = st.suggest_rows.len();
                                    ((item_id + 1)..n).find(|&i| {
                                        st.suggest_rows
                                            .get(i)
                                            .is_some_and(|r| r.action == clear_l)
                                    })
                                };
                                if let Some(to) = to {
                                    let mut st = app_state().lock().unwrap();
                                    if reorder_station_virtual_priority(&mut st, item_id, to) {
                                        st.open_suggest(field, "");
                                        drop(st);
                                        self.paint_station_print_attach(cx);
                                        self.sync_job_drive_ui(cx);
                                        self.ui.widget(cx, field.list()).redraw(cx);
                                    }
                                }
                                continue;
                            }
                            if up.pressed(actions)
                                || down.pressed(actions)
                                || up.released(actions)
                                || down.released(actions)
                            {
                                continue;
                            }
                        }
                        // hit Button (hover wash) preferred; row finger_up fallback.
                        let row_tap = item.button(cx, ids!(hit)).clicked(actions)
                            || item
                                .view(cx, ids!(row))
                                .finger_up(actions)
                                .is_some_and(finger_up_is_tap);
                        if row_tap {
                            let value = app_state()
                                .lock()
                                .unwrap()
                                .suggest_rows
                                .get(item_id)
                                .map(|r| {
                                    if field == SuggestField::PrinterAssign
                                        || field == SuggestField::StationVps
                                        || field == SuggestField::BillPrinters
                                        || field == SuggestField::TestPrinters
                                        || field == SuggestField::KotTestPrinters
                                    {
                                        r.subtitle.clone()
                                    } else {
                                        r.value.clone()
                                    }
                                });
                            if let Some(v) = value {
                                picked = Some((field, v));
                            }
                        }
                    }
                }
            }
            // Another (ordinary) field took focus: the open list is done.
            if let Some(field) = open
                && !typed
                && !needs_chrome
            {
                let mine = self.ui.text_input(cx, field.input()).widget_uid();
                let elsewhere = actions.iter().filter_map(|a| a.as_widget_action()).any(|wa| {
                    wa.widget_uid != mine
                        && matches!(wa.cast::<TextInputAction>(), TextInputAction::KeyFocus)
                });
                if elsewhere {
                    app_state().lock().unwrap().suggest_open = None;
                    needs_chrome = true;
                }
            }
            if let Some((field, value)) = picked {
                if field.opener().is_none() {
                    self.ui.text_input(cx, field.input()).set_text(cx, &value);
                }
                if field == SuggestField::FloorColumns {
                    app_state().lock().unwrap().suggest_open = None;
                    self.apply_floor_columns(cx, &value);
                }
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
                // Kitchen / printer-assign / station-vps / team-stations checklists stay open.
                if field != SuggestField::KotStation
                    && field != SuggestField::PrinterAssign
                    && field != SuggestField::StationVps
                    && field != SuggestField::BillPrinters
                    && field != SuggestField::TestPrinters
                    && field != SuggestField::KotTestPrinters
                    && field != SuggestField::TeamKotStations
                {
                    st.suggest_open = None;
                }
                // A picked country is shown with what it sets, ready to apply.
                if field == SuggestField::Country {
                    st.country_browse = value.clone();
                }
                // Picking a person to edit is the whole step: select them.
                if field == SuggestField::TeamPick {
                    st.team_select_pending = true;
                }
                if field == SuggestField::TeamCopyHire {
                    st.team_copy_hire_pending = true;
                }
                if field == SuggestField::TeamCopyEdit {
                    st.team_copy_edit_pending = true;
                }
                if field == SuggestField::CustomerName {
                    let found = st
                        .customers
                        .iter()
                        .find(|c| c.name.eq_ignore_ascii_case(value.trim()))
                        .map(|c| (c.id.clone(), c.name.clone()));
                    if let Some((id, name)) = found {
                        st.selected_customer_id = Some(id);
                        st.status = format!("Credit customer: {name}");
                    }
                }
                needs_chrome = true;
                // Pick = do the thing. No second Confirm/Use/Remove/Switch button.
                let kot_clear = if field == SuggestField::KotStation {
                    st.suggest_rows
                        .iter()
                        .find(|r| r.value == value)
                        .map(|r| {
                            let a = r.action.trim();
                            !a.is_empty() && a != st.t("use")
                        })
                        .unwrap_or(false)
                } else {
                    false
                };
                drop(st);
                match field {
                    SuggestField::KotStation => {
                        let result = if kot_clear {
                            self.clear_kot_station_from_unsent(cx, &value)
                        } else {
                            self.apply_kot_station_to_unassigned(cx, &value)
                        };
                        match result {
                            Ok(msg) => self.set_status(cx, msg),
                            Err(e) => self.set_status(cx, e),
                        }
                        // Refresh checkbox states while the menu stays open.
                        app_state()
                            .lock()
                            .unwrap()
                            .open_suggest(SuggestField::KotStation, "");
                    }
                    SuggestField::TeamKotStations => {
                        let mut st = app_state().lock().unwrap();
                        if st.team_staff_id.is_none() {
                            drop(st);
                            self.set_status(cx, app_t("staff_copy_need_selected"));
                        } else {
                            let all_ids: Vec<String> =
                                st.stations.iter().map(|s| s.id.clone()).collect();
                            let station_id = st
                                .stations
                                .iter()
                                .find(|s| s.name.eq_ignore_ascii_case(value.trim()))
                                .map(|s| s.id.clone());
                            if let Some(sid) = station_id {
                                st.draft_features.toggle_kot_station(&sid, &all_ids);
                                st.status = st.t("team_stations_changed");
                                st.open_suggest(SuggestField::TeamKotStations, "");
                            }
                        }
                    }
                    SuggestField::StationRemove | SuggestField::SettingsStationRemove => {
                        let input = field.input();
                        match self.remove_kitchen_station(cx, &value) {
                            Ok(msg) => {
                                self.ui.text_input(cx, input).set_text(cx, "");
                                self.set_status(cx, msg);
                            }
                            Err(e) => self.set_status(cx, e),
                        }
                    }
                    SuggestField::BranchSwitch => {
                        match self.switch_active_branch(cx, &value) {
                            Ok(msg) => self.set_status(cx, msg),
                            Err(e) => self.set_status(cx, e),
                        }
                    }
                    SuggestField::PrinterPlace => {
                        let (key, name) = {
                            let st = app_state().lock().unwrap();
                            let choices =
                                printer_place_choices(&st.stations, "", |k| st.t(k));
                            choices
                                .into_iter()
                                .find(|(_, label, _)| label == &value)
                                .map(|(k, label, _)| (k, label))
                                .unwrap_or_else(|| ("bill".into(), st.t("route_bill")))
                        };
                        {
                            let mut st = app_state().lock().unwrap();
                            st.printer_route_key = key;
                            st.printer_route_sel = None;
                            let _ = normalize_focused_route(&mut st);
                        }
                        let msg = {
                            let st = app_state().lock().unwrap();
                            st.t("route_editing").replace("{name}", &name)
                        };
                        self.set_status(cx, msg);
                        self.paint_printers_panel(cx);
                    }
                    SuggestField::PrinterAssign => {
                        let msg = {
                            let mut st = app_state().lock().unwrap();
                            let shown = st
                                .suggest_rows
                                .iter()
                                .find(|r| r.subtitle == value)
                                .map(|r| r.value.clone())
                                .unwrap_or_else(|| value.clone());
                            if select_real_printer_for_virtual(&mut st, &value) {
                                st.t("real_printer_selected").replace("{name}", &shown)
                            } else {
                                String::new()
                            }
                        };
                        if !msg.is_empty() {
                            self.set_status(cx, msg);
                        }
                        app_state()
                            .lock()
                            .unwrap()
                            .open_suggest(SuggestField::PrinterAssign, "");
                        self.paint_printers_panel(cx);
                        self.ui
                            .widget(cx, ids!(printer_assign_suggest))
                            .redraw(cx);
                        needs_chrome = false;
                    }
                    SuggestField::StationPrint => {
                        let msg = {
                            let mut st = app_state().lock().unwrap();
                            let key = resolve_printer_route_key(&value, &st.stations, |k| st.t(k));
                            if !key.starts_with("kot:") || key == "kot:" {
                                st.t("station_print_pick_first")
                            } else {
                                st.printer_route_key = key.clone();
                                st.printer_draft.ensure_station_routes([key
                                    .strip_prefix("kot:")
                                    .unwrap_or("")]);
                                let _ = st.save_printer_draft();
                                let name = printer_route_display_name(&key, &st.stations, |k| st.t(k));
                                st.t("station_print_editing").replace("{name}", &name)
                            }
                        };
                        self.set_status(cx, msg);
                        self.paint_station_print_attach(cx);
                    }
                    SuggestField::StationVps | SuggestField::BillPrinters => {
                        let msg = {
                            let mut st = app_state().lock().unwrap();
                            let vp_id = st
                                .suggest_rows
                                .iter()
                                .find(|r| r.value == value || r.subtitle == value)
                                .map(|r| r.subtitle.clone())
                                .unwrap_or_else(|| value.clone());
                            match toggle_station_virtual_printer(&mut st, &vp_id) {
                                Ok(m) => {
                                    st.open_suggest(field, "");
                                    m
                                }
                                Err(e) => e,
                            }
                        };
                        self.set_status(cx, msg);
                        self.paint_station_print_attach(cx);
                        self.sync_job_drive_ui(cx);
                        self.ui.widget(cx, field.list()).redraw(cx);
                        needs_chrome = false;
                    }
                    SuggestField::TestPrinters | SuggestField::KotTestPrinters => {
                        let msg = {
                            let mut st = app_state().lock().unwrap();
                            let pid = st
                                .suggest_rows
                                .iter()
                                .find(|r| r.value == value || r.subtitle == value)
                                .map(|r| r.subtitle.clone())
                                .unwrap_or_else(|| value.clone());
                            match toggle_test_printer(&mut st, &pid) {
                                Ok(m) => {
                                    st.open_suggest(field, "");
                                    m
                                }
                                Err(e) => e,
                            }
                        };
                        self.set_status(cx, msg);
                        self.paint_printers_panel(cx);
                        self.sync_kot_format_ui(cx);
                        self.ui.widget(cx, field.list()).redraw(cx);
                        needs_chrome = false;
                    }
                    _ => {}
                }
            }
            if needs_chrome {
                self.sync_chrome(cx);
                self.redraw_suggest_lists(cx);
            } else if typed {
                // Narrow-only: list rows update without rebuilding the shell.
                self.redraw_suggest_lists(cx);
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
                    .is_some_and(finger_up_is_tap);
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
            let searching = !app_state().lock().unwrap().menu_filter.trim().is_empty();
            let page_count = if searching {
                1
            } else {
                let st = app_state().lock().unwrap();
                menu_pick_tab_keys(&st).len().min(MENU_PICK_PAGE_MAX)
            };
            let cols = *product_grid_cols().lock().unwrap();
            let slots: [&[LiveId]; 6] = [ids!(c0), ids!(c1), ids!(c2), ids!(c3), ids!(c4), ids!(c5)];
            enum CardTap {
                Add(usize),
                Minus(usize),
            }
            let mut tapped: Option<(usize, CardTap)> = None;
            'pages: for pi in 0..page_count {
                let widget = self.ui.widget(cx, MENU_PICK_GRIDS[pi]);
                let list = widget.portal_list(cx, ids!(list));
                for (row_id, item) in list.items_with_actions(actions) {
                    for (col, path) in slots.iter().enumerate().take(cols) {
                        let index = row_id * cols + col;
                        let tile = item.widget(cx, *path).view(cx, ids!(tile));
                        if tile.button(cx, ids!(minus)).clicked(actions) {
                            tapped = Some((pi, CardTap::Minus(index)));
                            break 'pages;
                        } else if tile.button(cx, ids!(plus)).clicked(actions)
                            || tile.finger_up(actions).is_some_and(finger_up_is_tap)
                        {
                            tapped = Some((pi, CardTap::Add(index)));
                            break 'pages;
                        }
                    }
                }
            }
            match tapped {
                Some((pi, CardTap::Add(index))) => {
                    self.align_menu_pick_cart_page(cx, pi);
                    self.add_product_to_cart(cx, index);
                }
                Some((pi, CardTap::Minus(index))) => {
                    self.align_menu_pick_cart_page(cx, pi);
                    self.remove_product_from_cart(cx, index);
                }
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
                        let product = st
                            .products
                            .get(item_id)
                            .cloned()
                            .ok_or_else(|| "Missing product".to_string())?;
                        let msg = st.books_op(BooksOp::SetProductAvailable {
                            id: product.id,
                            name: product.name.clone(),
                            available: !product.available,
                        })?;
                        st.status = msg;
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
                        .is_some_and(finger_up_is_tap)
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
                    .is_some_and(finger_up_is_tap);
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
            // Commit Unit @ / Applied / note before any close. A summary tap
            // while the field is focused used to run toggle→sync_chrome first,
            // paint the old model back into the input, then "apply" that stale
            // text — looks like a revert for Owner and Waiter alike.
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
            if item.button(cx, ids!(qty_busy_cancel)).clicked(actions) {
                self.cancel_cart_qty_busy(cx);
                continue;
            }
            let minus = item.button(cx, ids!(minus)).clicked(actions);
            let plus = item.button(cx, ids!(plus)).clicked(actions);
            let hold = item.button(cx, ids!(hold_send_btn)).clicked(actions);
            let reprint = item.button(cx, ids!(reprint_incl_btn)).clicked(actions);
            let cancel_pick = item.button(cx, ids!(cancel_pick_btn)).clicked(actions);
            let summary_tap = item
                .view(cx, ids!(summary))
                .finger_up(actions)
                .is_some_and(finger_up_is_tap);
            let line_action = [
                (ids!(line_cancel), LineAction::Cancel),
                (ids!(line_nc), LineAction::Complimentary),
                (ids!(line_move), LineAction::Move),
            ]
            .into_iter()
            .find(|(button, _)| item.button(cx, *button).clicked(actions));
            // Any sync-causing row action must see the typed fields first.
            if minus || plus || hold || reprint || cancel_pick || summary_tap || line_action.is_some()
            {
                self.commit_cart_row_fields(cx, item_id, &item);
            }
            let stepped = if minus {
                self.bump_cart_line(cx, item_id, -1);
                true
            } else if plus {
                self.bump_cart_line(cx, item_id, 1);
                true
            } else if hold {
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
            } else if reprint {
                {
                    let mut st = app_state().lock().unwrap();
                    if let Some(line) = st.cart.lines.get_mut(item_id) {
                        if line.is_sent() {
                            line.reprint_skip = !line.reprint_skip;
                        }
                    }
                }
                self.sync_chrome(cx);
                true
            } else if cancel_pick {
                {
                    let mut st = app_state().lock().unwrap();
                    if let Some(line) = st.cart.lines.get_mut(item_id) {
                        let has_running = line.is_sent()
                            && !line.running_line_id.is_empty()
                            && line.running_line_id != "counter"
                            && !line.running_line_id.starts_with(QUEUED_LINE);
                        if has_running {
                            line.cancel_pick = !line.cancel_pick;
                        }
                    }
                }
                self.sync_chrome(cx);
                true
            } else {
                false
            };
            // ± / Hold / In-print sit inside the row: a step must not also open the details.
            if !stepped && summary_tap {
                self.toggle_cart_line(cx, item_id);
            }
            self.handle_modifier_chips(cx, actions, &item, item_id, false);
            if let Some((_, action)) = line_action {
                self.open_line_action(cx, item_id, action);
            }
        }

        let sale_widget = self.ui.widget(cx, ids!(sale_list));
        let sale_list = sale_widget.portal_list(cx, ids!(list));
        for (item_id, item) in sale_list.items_with_actions(actions) {
            // The whole row picks the sale.
            if item.view(cx, ids!(row)).finger_up(actions).is_some_and(finger_up_is_tap) {
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
            let minus = item.button(cx, ids!(minus)).clicked(actions);
            let plus = item.button(cx, ids!(plus)).clicked(actions);
            let summary_tap = item
                .view(cx, ids!(summary))
                .finger_up(actions)
                .is_some_and(finger_up_is_tap);
            if minus || plus || summary_tap {
                self.commit_amend_row_fields(cx, item_id, &item);
            }
            if minus {
                app_state()
                    .lock()
                    .unwrap()
                    .amend_cart
                    .bump_qty(item_id, -1);
                self.sync_chrome(cx);
            }
            if plus {
                app_state()
                    .lock()
                    .unwrap()
                    .amend_cart
                    .bump_qty(item_id, 1);
                self.sync_chrome(cx);
            }
            if !minus && !plus && summary_tap {
                self.toggle_cart_line(cx, item_id);
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
            ((ids!(staff_role_cashier), ids!(staff_role_cashier_on)), StaffRole::Cashier, "role_cashier"),
            ((ids!(staff_role_manager), ids!(staff_role_manager_on)), StaffRole::Manager, "role_manager"),
            ((ids!(staff_role_admin), ids!(staff_role_admin_on)), StaffRole::Admin, "role_admin"),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                let label = {
                    let mut st = app_state().lock().unwrap();
                    // Hire Job chips write hire draft only — never edit draft.
                    // X12: defaults capped to actor ceiling (Owner = uncapped;
                    // non-Owner for_role stations/screens are clipped).
                    st.hire_staff_role = role;
                    st.hire_features = st
                        .session
                        .as_ref()
                        .map(|a| store::Store::features_for_hire(role, a))
                        .unwrap_or_else(|| StaffFeatures::for_role(role));
                    // Clear selection so hire vs edit stay distinct.
                    st.team_staff_id = None;
                    st.t(key)
                };
                self.ui.text_input(cx, ids!(team_pick_input)).set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(staff_edit_name_input))
                    .set_text(cx, "");
                // Role title follows the preset (edit afterward for Captain, etc.).
                self.ui
                    .text_input(cx, ids!(staff_title_input))
                    .set_text(cx, &label);
                self.set_status(cx, format!("Preset: {label}"));
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(staff_add_cashier)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(staff_name_input)).text();
            let title = self.ui.text_input(cx, ids!(staff_title_input)).text();
            let pin = self.ui.text_input(cx, ids!(staff_pin_input)).text();
            let login = self.ui.text_input(cx, ids!(staff_login_input)).text();
            let password = self.ui.text_input(cx, ids!(staff_password_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                if !st.owner_setup_complete {
                    return Err("Finish Owner PIN setup before adding staff".into());
                }
                let role = st.hire_staff_role;
                let features = st.hire_features.clone();
                let msg = st.books_op(BooksOp::HireStaff {
                    name: name.clone(),
                    role,
                    role_title: title.clone(),
                    pin: pin.clone(),
                    login_id: login.clone(),
                    password: password.clone(),
                    features,
                })?;
                let (status, hired_id) = books_op::hired_staff_parts(&msg);
                // Always select the person just hired — never "last non-owner".
                // Load edit draft only; hire Job chips keep hire_* untouched.
                if let Some(id) = hired_id.filter(|s| !s.is_empty()) {
                    let draft = st.staff.iter().find(|s| s.id == id).map(|m| {
                        (
                            store::Store::hire_role_from(m),
                            m.effective_features(),
                        )
                    });
                    st.team_staff_id = Some(id.to_string());
                    if let Some((role, features)) = draft {
                        st.draft_staff_role = role;
                        st.draft_features = features;
                    }
                }
                st.status = status.to_string();
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
                self.ui
                    .text_input(cx, ids!(staff_copy_hire_input))
                    .set_text(cx, "");
                // Hire and edit are separate: show who Add just selected for
                // features — do not leave hire fields looking like a second copy.
                let (pick_label, edit_name) = {
                    let st = app_state().lock().unwrap();
                    st.team_staff_id
                        .as_ref()
                        .and_then(|id| st.staff.iter().find(|s| &s.id == id))
                        .map(|s| (s.lock_label(), s.name.clone()))
                        .unwrap_or_default()
                };
                self.ui
                    .text_input(cx, ids!(team_pick_input))
                    .set_text(cx, &pick_label);
                self.ui
                    .text_input(cx, ids!(staff_edit_name_input))
                    .set_text(cx, &edit_name);
                self.ui
                    .text_input(cx, ids!(staff_edit_login_input))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(staff_edit_password_input))
                    .set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(staff_set_login)).clicked(actions) {
            let login = self.ui.text_input(cx, ids!(staff_edit_login_input)).text();
            let password = self
                .ui
                .text_input(cx, ids!(staff_edit_password_input))
                .text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| st.t("staff_copy_need_selected"))?;
                let who = st
                    .staff
                    .iter()
                    .find(|s| s.id == staff_id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| staff_id.clone());
                st.books_op(BooksOp::SetStaffLogin {
                    staff_id,
                    login_id: login,
                    password,
                })?;
                Ok(who)
            })();
            match result {
                Ok(who) => {
                    self.ui
                        .text_input(cx, ids!(staff_edit_login_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(staff_edit_password_input))
                        .set_text(cx, "");
                    self.set_status(cx, format!("Login saved for {who}"));
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(staff_deactivate)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| "Pick who to edit first — then Remove selected".to_string())?;
                let removed_name = st
                    .staff
                    .iter()
                    .find(|s| s.id == staff_id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| staff_id.clone());
                let msg = st.books_op(BooksOp::DeactivateStaff {
                    staff_id: staff_id.clone(),
                    name: removed_name.clone(),
                })?;
                st.team_staff_id = None;
                if st.device_role != DeviceRole::Client && st.staff.iter().any(|s| s.id == staff_id) {
                    return Err(format!(
                        "Could not remove {removed_name} — they are still on the team"
                    ));
                }
                Ok(msg)
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(team_pick_input)).set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(staff_edit_name_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(staff_copy_edit_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(change_my_pin)).clicked(actions)
            || self.ui.button(cx, ids!(change_my_password)).clicked(actions)
        {
            let pin = self.ui.text_input(cx, ids!(my_pin_input)).text();
            let password = self.ui.text_input(cx, ids!(my_password_input)).text();
            let pin_set = !pin.trim().is_empty();
            let password_set = !password.trim().is_empty();
            if !pin_set && !password_set {
                self.set_status(cx, "Type a new PIN, a new password, or both");
                self.sync_chrome(cx);
            } else {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_change_own_credentials() {
                    return Err("Not allowed".into());
                }
                let client = st.device_role == DeviceRole::Client;
                if pin_set {
                    if client {
                        st.lan_job(
                            "/staff_pin",
                            &net::RemoteOwnPinRequest { pin: pin.clone() },
                            "PIN".into(),
                            "PIN updated on the shop's books".into(),
                            phone::Follow::Refresh,
                            false,
                        )?;
                    } else {
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.set_staff_pin(&actor.id, &pin, &actor)?;
                    }
                }
                if password_set {
                    if client {
                        st.lan_job(
                            "/staff_password",
                            &net::RemoteOwnPasswordRequest {
                                password: password.clone(),
                            },
                            "Password".into(),
                            "Password updated on the shop's books".into(),
                            phone::Follow::Refresh,
                            false,
                        )?;
                    } else {
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.set_own_password(&password, &actor)?;
                    }
                }
                let msg = match (pin_set, password_set, client) {
                    (true, true, true) => "Updating PIN and password…".to_string(),
                    (true, false, true) => "Updating PIN…".to_string(),
                    (false, true, true) => "Updating password…".to_string(),
                    (true, true, false) => "PIN and password updated".to_string(),
                    (true, false, false) => "PIN updated".to_string(),
                    (false, true, false) => "Password updated".to_string(),
                    (false, false, _) => "Type a new PIN, a new password, or both".to_string(),
                };
                Ok(msg)
            })();
            match result {
                Ok(msg) => {
                    if pin_set {
                        self.ui.text_input(cx, ids!(my_pin_input)).set_text(cx, "");
                    }
                    if password_set {
                        self.ui.text_input(cx, ids!(my_password_input)).set_text(cx, "");
                    }
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
            }
        }

        if self.ui.button(cx, ids!(save_duress_pin)).clicked(actions) {
            let pin = self.ui.text_input(cx, ids!(duress_pin_input)).text();
            let confirm = self.ui.text_input(cx, ids!(duress_pin_confirm)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                if !st.session.as_ref().is_some_and(|s| s.is_owner()) {
                    return Err("Only the Owner can set the emergency unlock PIN".into());
                }
                st.books_op(BooksOp::SetDuressPin { pin, confirm })?;
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
                if !st.session.as_ref().is_some_and(|s| s.is_owner()) {
                    return Err("Only the Owner can clear the emergency unlock PIN".into());
                }
                st.books_op(BooksOp::ClearDuressPin)?;
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
                let msg = {
                    let mut st = app_state().lock().unwrap();
                    if st.team_staff_id.is_none() {
                        st.t("staff_copy_need_selected")
                    } else {
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
                        st.t("team_features_changed")
                    }
                };
                self.set_status(cx, msg);
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(feat_save)).clicked(actions) {
            let new_name = self.ui.text_input(cx, ids!(staff_edit_name_input)).text();
            let login = self.ui.text_input(cx, ids!(staff_edit_login_input)).text();
            let password = self
                .ui
                .text_input(cx, ids!(staff_edit_password_input))
                .text();
            let password_set = !password.trim().is_empty();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| st.t("staff_copy_need_selected"))?;
                let current_name = st
                    .staff
                    .iter()
                    .find(|s| s.id == staff_id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| staff_id.clone());
                if !new_name.trim().is_empty() && new_name.trim() != current_name.trim() {
                    st.books_op(BooksOp::RenameStaff {
                        staff_id: staff_id.clone(),
                        name: new_name.trim().to_string(),
                    })?;
                }
                if password_set {
                    st.books_op(BooksOp::SetStaffLogin {
                        staff_id: staff_id.clone(),
                        login_id: login,
                        password,
                    })?;
                }
                let name = st
                    .staff
                    .iter()
                    .find(|s| s.id == staff_id)
                    .map(|s| s.name.clone())
                    .unwrap_or(current_name);
                let features = st.draft_features.clone();
                st.books_op(BooksOp::SetStaffFeatures { staff_id, features })?;
                st.status = st.t("save_features_ok").replace("{name}", &name);
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else if password_set {
                self.ui
                    .text_input(cx, ids!(staff_edit_password_input))
                    .set_text(cx, "");
            }
            self.sync_chrome(cx);
        }

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
        for (id, pricing) in [
            (ids!(tax_inclusive_btn), TaxPricing::InsidePrice),
            (ids!(tax_exclusive_btn), TaxPricing::AddedAtTill),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
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
        for (id, allow) in [
            (ids!(zero_sales_on_btn), true),
            (ids!(zero_sales_off_btn), false),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    st.books_op(BooksOp::SetZeroSales { allow })?;
                    if st.device_role == DeviceRole::Client {
                        if let Some(store) = st.store.as_ref() {
                            store.set_allow_zero_sales_from_sync(allow)?;
                        }
                    }
                    let allow_now = st
                        .store
                        .as_ref()
                        .and_then(|s| s.allow_zero_sales().ok())
                        .unwrap_or(allow);
                    let msg = st.t(if allow_now {
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
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err("Only the Owner can set the business id".into());
                }
                let service_bps = locale::parse_percent_bps(&service_text)?;
                let msg = st.books_op(BooksOp::SetFiscal {
                    business_tax_id: id_text,
                    service_bps,
                })?;
                if st.device_role == DeviceRole::Client {
                    st.status = msg;
                    return Ok(());
                }
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
                // Menu catalogues import later on Menu (shelf / client
                // templates) — not during setup. Setup templates later =
                // whole shop face, not a dish dump.
                st.store.as_ref().ok_or_else(|| "No store".to_string())?.complete_onboarding()?;
                st.onboarding_complete = true;
                // Main screen is a root — drop DeviceSetup/Onboard from Back.
                st.nav_back.clear();
                st.nav_forward.clear();
                st.screen = unlocked_home_from(&st);
                st.status = "Shop ready — your books are open".into();
                let _ = st.reload_all();
                let _ = st.refresh_books_overview();
                Ok(())
            })();
            if let Err(e) = result { self.set_status(cx, e); }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(client_find_cancel)).clicked(actions) {
            net::cancel_discover_servers();
            client_find_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
            self.paint_submit_busy_chrome(cx);
        }
        if self.ui.button(cx, ids!(client_pair_cancel)).clicked(actions) {
            self.abandon_client_pair(cx);
        }
        if self.ui.button(cx, ids!(client_find_btn)).clicked(actions) {
            if client_find_busy().load(std::sync::atomic::Ordering::SeqCst) {
                // Looking… — ignore re-taps.
            } else {
                client_find_busy().store(true, std::sync::atomic::Ordering::SeqCst);
                client_find_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
                let looking = app_state().lock().unwrap().t("finding_server");
                self.ui
                    .widget(cx, ids!(client_found_label))
                    .set_text(cx, &plain_display_text(&looking));
                self.ui.widget(cx, ids!(client_found_label)).set_visible(cx, true);
                app_state().lock().unwrap().found_servers.clear();
                self.ui.widget(cx, ids!(client_found_list)).redraw(cx);
                self.paint_submit_busy_chrome(cx);
                let _ = std::thread::Builder::new()
                    .name("tabiyat-find-server".into())
                    .spawn(|| {
                        let found = net::discover_servers(std::time::Duration::from_millis(3000));
                        *discovery_result().lock().unwrap() = Some(found);
                        client_find_busy().store(false, std::sync::atomic::Ordering::SeqCst);
                        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                    });
            }
        }
        if self.ui.button(cx, ids!(bill_format_save)).clicked(actions) {
            // Read the open line now. Save is handled before the field
            // editors, so a size or address typed in this same press would
            // otherwise be saved as the previous value.
            self.read_bill_editor_into_draft(cx);
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let fmt = st.bill_slip_draft.clone().clamped();
                let (cur_vpa, cur_payee) = match st.store.as_ref() {
                    Some(store) => (
                        store.upi_vpa().unwrap_or_default(),
                        store.upi_payee().unwrap_or_default(),
                    ),
                    None => (String::new(), String::new()),
                };
                let upi_vpa = if fmt.upi_vpa.trim() == cur_vpa.trim() {
                    None
                } else {
                    Some(fmt.upi_vpa.clone())
                };
                let upi_payee = if fmt.upi_payee.trim() == cur_payee.trim() {
                    None
                } else {
                    Some(fmt.upi_payee.clone())
                };
                let em = fmt.style(print_share::BillStyleField::Items).size_em as f32;
                st.printer_draft.set_standard_em(em);
                st.books_op(BooksOp::SetBillFormat {
                    header: fmt.header_lines(),
                    fssai: fmt.fssai.clone(),
                    footer: fmt.footer.clone(),
                    upi_vpa,
                    upi_payee,
                    styles: fmt.to_json(),
                })?;
                st.books_op(BooksOp::SetStandardEm { em })?;
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.publish_print_formats();
                    self.fill_bill_format_fields(cx);
                    self.set_status(cx, "Bill format saved");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        // Bill & KOT format — Bill | KOT tabs under the section header.
        {
            let mut tab_changed = false;
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_kot_tab_bill),
                ids!(bill_kot_tab_bill_on),
            ) {
                app_state().lock().unwrap().bill_kot_tab_kot = false;
                tab_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_kot_tab_kot),
                ids!(bill_kot_tab_kot_on),
            ) {
                app_state().lock().unwrap().bill_kot_tab_kot = true;
                tab_changed = true;
            }
            if tab_changed {
                self.sync_bill_kot_tabs(cx);
                self.ui.redraw(cx);
            }
        }
        // KOT format: kind → tap a field row → show + size + weight → order/copies.
        {
            let mut slip_changed = false;
            let mut style_changed = false;
            // A typed size lands on the field still selected. Size does not
            // change the preview text, so it must not rebuild the slip.
            if let Some(size_now) = self
                .ui
                .text_input(cx, ids!(kot_field_size_input))
                .changed(actions)
            {
                if let Ok(raw) = size_now.trim().parse::<u32>() {
                    let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
                    let mut st = app_state().lock().unwrap();
                    let kind = st.kot_edit_kind;
                    let field = st.kot_style_field;
                    if st.kot_slip_draft.kind_styles.get(kind).get(field).size_em != em {
                        st.kot_slip_draft
                            .kind_styles
                            .get_mut(kind)
                            .get_mut(field)
                            .size_em = em;
                        if field == print_share::KotStyleField::ItemQty {
                            st.kot_slip_draft.em = em;
                        }
                        style_changed = true;
                    }
                }
            }
            for (idle, on, kind) in [
                (
                    ids!(kot_kind_original),
                    ids!(kot_kind_original_on),
                    print_share::KotSlipKind::Original,
                ),
                (
                    ids!(kot_kind_duplicate),
                    ids!(kot_kind_duplicate_on),
                    print_share::KotSlipKind::Duplicate,
                ),
                (
                    ids!(kot_kind_modified),
                    ids!(kot_kind_modified_on),
                    print_share::KotSlipKind::Modified,
                ),
                (
                    ids!(kot_kind_cancelled),
                    ids!(kot_kind_cancelled_on),
                    print_share::KotSlipKind::Cancelled,
                ),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    app_state().lock().unwrap().kot_edit_kind = kind;
                    slip_changed = true;
                    break;
                }
            }
            for (idle, on, field) in [
                (ids!(kot_pick_title), ids!(kot_pick_title_on), print_share::KotStyleField::Title),
                (ids!(kot_pick_shop), ids!(kot_pick_shop_on), print_share::KotStyleField::Shop),
                (ids!(kot_pick_time), ids!(kot_pick_time_on), print_share::KotStyleField::Time),
                (ids!(kot_pick_waiter), ids!(kot_pick_waiter_on), print_share::KotStyleField::Waiter),
                (ids!(kot_pick_station), ids!(kot_pick_station_on), print_share::KotStyleField::Station),
                (ids!(kot_pick_table), ids!(kot_pick_table_on), print_share::KotStyleField::Table),
                (ids!(kot_pick_no), ids!(kot_pick_no_on), print_share::KotStyleField::KotNo),
                (ids!(kot_pick_area), ids!(kot_pick_area_on), print_share::KotStyleField::Area),
                (ids!(kot_pick_qty), ids!(kot_pick_qty_on), print_share::KotStyleField::ItemQty),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    let mut st = app_state().lock().unwrap();
                    if st.kot_style_field != field {
                        st.kot_style_field = field;
                        style_changed = true;
                    }
                    break;
                }
            }
            for (i, idle, on) in [
                (0, ids!(kot_ord_0), ids!(kot_ord_0_on)),
                (1, ids!(kot_ord_1), ids!(kot_ord_1_on)),
                (2, ids!(kot_ord_2), ids!(kot_ord_2_on)),
                (3, ids!(kot_ord_3), ids!(kot_ord_3_on)),
                (4, ids!(kot_ord_4), ids!(kot_ord_4_on)),
                (5, ids!(kot_ord_5), ids!(kot_ord_5_on)),
                (6, ids!(kot_ord_6), ids!(kot_ord_6_on)),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    let mut st = app_state().lock().unwrap();
                    ensure_kot_field_order(&mut st.kot_slip_draft);
                    if i < st.kot_slip_draft.field_order.len() && st.kot_field_edit_idx != i {
                        st.kot_field_edit_idx = i;
                        style_changed = true;
                    }
                    break;
                }
            }
            if self.choice_chip_clicked(cx, actions, ids!(kot_field_show), ids!(kot_field_show_on))
            {
                let mut st = app_state().lock().unwrap();
                let field = st.kot_style_field;
                st.kot_slip_draft.set_field_shown(field, true);
                slip_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(kot_field_hide), ids!(kot_field_hide_on))
            {
                let mut st = app_state().lock().unwrap();
                let field = st.kot_style_field;
                st.kot_slip_draft.set_field_shown(field, false);
                slip_changed = true;
            }
            for (idle, on, weight) in [
                (
                    ids!(kot_field_weight_light),
                    ids!(kot_field_weight_light_on),
                    print_share::KotFontWeight::Light,
                ),
                (
                    ids!(kot_field_weight_reg),
                    ids!(kot_field_weight_reg_on),
                    print_share::KotFontWeight::Regular,
                ),
                (
                    ids!(kot_field_weight_bold),
                    ids!(kot_field_weight_bold_on),
                    print_share::KotFontWeight::Bold,
                ),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    let mut st = app_state().lock().unwrap();
                    let kind = st.kot_edit_kind;
                    let field = st.kot_style_field;
                    st.kot_slip_draft
                        .kind_styles
                        .get_mut(kind)
                        .get_mut(field)
                        .weight = weight;
                    style_changed = true;
                    break;
                }
            }
            for (idle, on, align) in [
                (ids!(kot_align_start), ids!(kot_align_start_on), print_share::KotAlign::Start),
                (ids!(kot_align_center), ids!(kot_align_center_on), print_share::KotAlign::Center),
                (ids!(kot_align_end), ids!(kot_align_end_on), print_share::KotAlign::End),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    let mut st = app_state().lock().unwrap();
                    let kind = st.kot_edit_kind;
                    let field = st.kot_style_field;
                    st.kot_slip_draft
                        .kind_styles
                        .get_mut(kind)
                        .get_mut(field)
                        .align = align;
                    slip_changed = true;
                    break;
                }
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_gap_tight),
                ids!(kot_item_gap_tight_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .kot_slip_draft
                    .set_item_col_gap_preset(print_share::KOT_COL_GAP_TIGHT);
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_gap_comfort),
                ids!(kot_item_gap_comfort_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .kot_slip_draft
                    .set_item_col_gap_preset(print_share::KOT_COL_GAP_COMFORT);
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_gap_wide),
                ids!(kot_item_gap_wide_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .kot_slip_draft
                    .set_item_col_gap_preset(print_share::KOT_COL_GAP_WIDE);
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_num_compact),
                ids!(kot_item_num_compact_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .kot_slip_draft
                    .set_item_qty_min_preset(print_share::KOT_QTY_MIN_COMPACT);
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_num_comfort),
                ids!(kot_item_num_comfort_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .kot_slip_draft
                    .set_item_qty_min_preset(print_share::KOT_QTY_MIN_COMFORT);
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_num_wide),
                ids!(kot_item_num_wide_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .kot_slip_draft
                    .set_item_qty_min_preset(print_share::KOT_QTY_MIN_WIDE);
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_num_start),
                ids!(kot_item_num_start_on),
            ) {
                app_state().lock().unwrap().kot_slip_draft.item_num_align =
                    print_share::KotAlign::Start;
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_num_center),
                ids!(kot_item_num_center_on),
            ) {
                app_state().lock().unwrap().kot_slip_draft.item_num_align =
                    print_share::KotAlign::Center;
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_item_num_end),
                ids!(kot_item_num_end_on),
            ) {
                app_state().lock().unwrap().kot_slip_draft.item_num_align =
                    print_share::KotAlign::End;
                slip_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(kot_item_hdr_item_input))
                .changed(actions)
            {
                app_state().lock().unwrap().kot_slip_draft.item_header_item = raw;
                slip_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(kot_item_hdr_qty_input))
                .changed(actions)
            {
                app_state().lock().unwrap().kot_slip_draft.item_header_qty = raw;
                slip_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(kot_item_gap_em_input))
                .changed(actions)
            {
                if let Ok(v) = raw.trim().parse::<f32>() {
                    app_state().lock().unwrap().kot_slip_draft.item_col_gap_em =
                        print_share::clamp_bill_col_gap_em(v);
                    slip_changed = true;
                }
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(kot_item_qty_min_input))
                .changed(actions)
            {
                if let Ok(v) = raw.trim().parse::<f32>() {
                    app_state().lock().unwrap().kot_slip_draft.item_qty_min_em =
                        print_share::clamp_bill_col_min_em(v);
                    slip_changed = true;
                }
            }
            // Move up/down reorder the print-order row that is selected.
            if self.ui.button(cx, ids!(kot_field_up)).clicked(actions) {
                let mut st = app_state().lock().unwrap();
                ensure_kot_field_order(&mut st.kot_slip_draft);
                let n = st.kot_slip_draft.field_order.len();
                if n > 0 {
                    let i = st.kot_field_edit_idx.min(n - 1);
                    if i > 0 {
                        st.kot_slip_draft.field_order.swap(i, i - 1);
                        st.kot_field_edit_idx = i - 1;
                        slip_changed = true;
                    }
                }
            }
            if self.ui.button(cx, ids!(kot_field_down)).clicked(actions) {
                let mut st = app_state().lock().unwrap();
                ensure_kot_field_order(&mut st.kot_slip_draft);
                let n = st.kot_slip_draft.field_order.len();
                if n > 0 {
                    let i = st.kot_field_edit_idx.min(n - 1);
                    if i + 1 < n {
                        st.kot_slip_draft.field_order.swap(i, i + 1);
                        st.kot_field_edit_idx = i + 1;
                        slip_changed = true;
                    }
                }
            }
            if self.choice_chip_clicked(cx, actions, ids!(kot_copies_1), ids!(kot_copies_1_on)) {
                app_state().lock().unwrap().kot_slip_draft.copies = 1;
                slip_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(kot_copies_2), ids!(kot_copies_2_on)) {
                app_state().lock().unwrap().kot_slip_draft.copies = 2;
                slip_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(kot_copies_3), ids!(kot_copies_3_on)) {
                app_state().lock().unwrap().kot_slip_draft.copies = 3;
                slip_changed = true;
            }
            if let Some(title_now) = self.ui.text_input(cx, ids!(kot_title_input)).changed(actions)
            {
                let mut st = app_state().lock().unwrap();
                if st.kot_slip_draft.title != title_now {
                    st.kot_slip_draft.title = title_now.clone();
                    // A typed title is meant to print. Hide stays available if they turn it off.
                    if !title_now.trim().is_empty() && !st.kot_slip_draft.show_title {
                        st.kot_slip_draft.show_title = true;
                    }
                    slip_changed = true;
                }
            }
            if let Some(footer_now) = self.ui.text_input(cx, ids!(kot_footer_input)).changed(actions)
            {
                let mut st = app_state().lock().unwrap();
                if st.kot_slip_draft.footer != footer_now {
                    st.kot_slip_draft.footer = footer_now;
                    slip_changed = true;
                }
            }
            if let Some(banner_now) =
                self.ui.text_input(cx, ids!(kot_banner_input)).changed(actions)
            {
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                let styles = st.kot_slip_draft.kind_styles.get_mut(kind);
                if styles.banner != banner_now {
                    styles.banner = banner_now;
                    slip_changed = true;
                }
            }
            if let Some(prefix_now) = self
                .ui
                .text_input(cx, ids!(kot_item_prefix_input))
                .changed(actions)
            {
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                let styles = st.kot_slip_draft.kind_styles.get_mut(kind);
                if styles.item_prefix != prefix_now {
                    styles.item_prefix = prefix_now;
                    slip_changed = true;
                }
            }
            if self.choice_chip_clicked(cx, actions, ids!(kot_strike_on), ids!(kot_strike_on_on))
            {
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                let styles = st.kot_slip_draft.kind_styles.get_mut(kind);
                styles.strike_item_name = true;
                styles.strike_items = true;
                slip_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(kot_strike_off), ids!(kot_strike_off_on))
            {
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                let styles = st.kot_slip_draft.kind_styles.get_mut(kind);
                styles.strike_item_name = false;
                styles.strike_items = false;
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_strike_note_on),
                ids!(kot_strike_note_on_on),
            ) {
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                st.kot_slip_draft.kind_styles.get_mut(kind).strike_item_note = true;
                slip_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(kot_strike_note_off),
                ids!(kot_strike_note_off_on),
            ) {
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                st.kot_slip_draft.kind_styles.get_mut(kind).strike_item_note = false;
                slip_changed = true;
            }
            if self.ui.button(cx, ids!(kot_apply_all)).clicked(actions) {
                // The open field's show, size, weight, and align land on
                // every other field of this KOT kind.
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                let src = st.kot_style_field;
                let style = st.kot_slip_draft.field_style(kind, src);
                let shown = st.kot_slip_draft.field_shown(src);
                for field in print_share::KotStyleField::ALL {
                    if field == src {
                        continue;
                    }
                    *st.kot_slip_draft.kind_styles.get_mut(kind).get_mut(field) = style.clone();
                    st.kot_slip_draft.set_field_shown(field, shown);
                }
                st.kot_slip_draft.em = st
                    .kot_slip_draft
                    .kind_styles
                    .get(kind)
                    .item_qty
                    .size_em;
                slip_changed = true;
            }
            if slip_changed || style_changed {
                // Size, weight, align, and show change the printed picture.
                self.sync_kot_format_ui(cx);
            }
        }
        if self.ui.button(cx, ids!(kot_format_save)).clicked(actions) {
            let title = self.ui.text_input(cx, ids!(kot_title_input)).text();
            let footer = self.ui.text_input(cx, ids!(kot_footer_input)).text();
            let banner = self.ui.text_input(cx, ids!(kot_banner_input)).text();
            let item_prefix = self.ui.text_input(cx, ids!(kot_item_prefix_input)).text();
            let size_now = self.ui.text_input(cx, ids!(kot_field_size_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.kot_slip_draft.title = title;
                st.kot_slip_draft.footer = footer;
                {
                    let kind = st.kot_edit_kind;
                    let styles = st.kot_slip_draft.kind_styles.get_mut(kind);
                    styles.banner = banner;
                    styles.item_prefix = item_prefix;
                }
                if let Ok(raw) = size_now.trim().parse::<u32>() {
                    let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
                    let kind = st.kot_edit_kind;
                    let field = st.kot_style_field;
                    st.kot_slip_draft
                        .kind_styles
                        .get_mut(kind)
                        .get_mut(field)
                        .size_em = em;
                    if field == print_share::KotStyleField::ItemQty {
                        st.kot_slip_draft.em = em;
                    }
                }
                let format = st.kot_slip_draft.clone();
                st.books_op(BooksOp::SetKotFormat { format })?;
                if st.device_role != DeviceRole::Client
                    && let Some(store) = st.store.as_ref()
                    && let Ok(saved) = store.kot_slip_format()
                {
                    st.kot_slip_draft = saved;
                }
                Ok(st.t("kot_format_saved"))
            })();
            match result {
                Ok(msg) => {
                    self.publish_print_formats();
                    self.sync_kot_format_ui(cx);
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        {
            // The other tab stays in the tree. Only the tab on screen may print,
            // so Test bill cannot send the kitchen-ticket preview.
            let on_kot = app_state().lock().unwrap().bill_kot_tab_kot;
            if on_kot && self.ui.button(cx, ids!(kot_format_test)).clicked(actions) {
                self.print_kot_format_preview(cx);
            }
            if !on_kot && self.ui.button(cx, ids!(bill_format_test)).clicked(actions) {
                self.print_bill_format_preview(cx);
            }
        }
        {
            let mut bill_changed = false;
            if let Some(text) = self
                .ui
                .text_input(cx, ids!(bill_field_value))
                .changed(actions)
            {
                let mut st = app_state().lock().unwrap();
                let field = st.bill_style_field;
                if field.has_text() {
                    st.bill_slip_draft.set_text(field, &text);
                    bill_changed = true;
                }
            }
            if let Some(size_now) = self
                .ui
                .text_input(cx, ids!(bill_field_size_input))
                .changed(actions)
            {
                if let Ok(raw) = size_now.trim().parse::<u32>() {
                    let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
                    let mut st = app_state().lock().unwrap();
                    let field = st.bill_style_field;
                    if st.bill_slip_draft.style(field).size_em != em {
                        st.bill_slip_draft.style_mut(field).size_em = em;
                        bill_changed = true;
                    }
                }
            }
            for (i, idle, on) in [
                (0, ids!(bill_pick_shop), ids!(bill_pick_shop_on)),
                (1, ids!(bill_pick_header), ids!(bill_pick_header_on)),
                (2, ids!(bill_pick_fssai), ids!(bill_pick_fssai_on)),
                (3, ids!(bill_pick_title), ids!(bill_pick_title_on)),
                (4, ids!(bill_pick_customer), ids!(bill_pick_customer_on)),
                (5, ids!(bill_pick_billno), ids!(bill_pick_billno_on)),
                (6, ids!(bill_pick_time), ids!(bill_pick_time_on)),
                (7, ids!(bill_pick_cashier), ids!(bill_pick_cashier_on)),
                (8, ids!(bill_pick_table), ids!(bill_pick_table_on)),
                (9, ids!(bill_pick_items), ids!(bill_pick_items_on)),
                (10, ids!(bill_pick_total), ids!(bill_pick_total_on)),
                (11, ids!(bill_pick_footer), ids!(bill_pick_footer_on)),
                (12, ids!(bill_pick_upi), ids!(bill_pick_upi_on)),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    self.commit_bill_field_value(cx);
                    let mut st = app_state().lock().unwrap();
                    let order = st.bill_slip_draft.ordered();
                    if let Some(field) = order.get(i).copied() {
                        st.bill_field_edit_idx = i;
                        if st.bill_style_field != field {
                            st.bill_style_field = field;
                        }
                        bill_changed = true;
                    }
                    break;
                }
            }
            for (i, idle, on) in [
                (0, ids!(bill_ord_0), ids!(bill_ord_0_on)),
                (1, ids!(bill_ord_1), ids!(bill_ord_1_on)),
                (2, ids!(bill_ord_2), ids!(bill_ord_2_on)),
                (3, ids!(bill_ord_3), ids!(bill_ord_3_on)),
                (4, ids!(bill_ord_4), ids!(bill_ord_4_on)),
                (5, ids!(bill_ord_5), ids!(bill_ord_5_on)),
                (6, ids!(bill_ord_6), ids!(bill_ord_6_on)),
                (7, ids!(bill_ord_7), ids!(bill_ord_7_on)),
                (8, ids!(bill_ord_8), ids!(bill_ord_8_on)),
                (9, ids!(bill_ord_9), ids!(bill_ord_9_on)),
                (10, ids!(bill_ord_10), ids!(bill_ord_10_on)),
                (11, ids!(bill_ord_11), ids!(bill_ord_11_on)),
                (12, ids!(bill_ord_12), ids!(bill_ord_12_on)),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    self.commit_bill_field_value(cx);
                    let mut st = app_state().lock().unwrap();
                    let order = st.bill_slip_draft.ordered();
                    if let Some(field) = order.get(i).copied() {
                        st.bill_field_edit_idx = i;
                        st.bill_style_field = field;
                        bill_changed = true;
                    }
                    break;
                }
            }
            if self.ui.button(cx, ids!(bill_field_up)).clicked(actions) {
                self.commit_bill_field_value(cx);
                let mut st = app_state().lock().unwrap();
                let n = st.bill_slip_draft.ordered().len();
                let i = st.bill_field_edit_idx.min(n.saturating_sub(1));
                if i > 0 {
                    st.bill_slip_draft.field_order = st
                        .bill_slip_draft
                        .ordered()
                        .iter()
                        .map(|f| f.id().to_string())
                        .collect();
                    st.bill_slip_draft.field_order.swap(i, i - 1);
                    st.bill_field_edit_idx = i - 1;
                    st.bill_style_field = print_share::BillStyleField::from_id(
                        &st.bill_slip_draft.field_order[i - 1],
                    )
                    .unwrap_or(st.bill_style_field);
                    bill_changed = true;
                }
            }
            if self.ui.button(cx, ids!(bill_field_down)).clicked(actions) {
                self.commit_bill_field_value(cx);
                let mut st = app_state().lock().unwrap();
                let n = st.bill_slip_draft.ordered().len();
                if n > 0 {
                    let i = st.bill_field_edit_idx.min(n - 1);
                    if i + 1 < n {
                        st.bill_slip_draft.field_order = st
                            .bill_slip_draft
                            .ordered()
                            .iter()
                            .map(|f| f.id().to_string())
                            .collect();
                        st.bill_slip_draft.field_order.swap(i, i + 1);
                        st.bill_field_edit_idx = i + 1;
                        st.bill_style_field = print_share::BillStyleField::from_id(
                            &st.bill_slip_draft.field_order[i + 1],
                        )
                        .unwrap_or(st.bill_style_field);
                        bill_changed = true;
                    }
                }
            }
            if self.choice_chip_clicked(cx, actions, ids!(bill_copies_1), ids!(bill_copies_1_on)) {
                app_state().lock().unwrap().bill_slip_draft.copies = 1;
                bill_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(bill_copies_2), ids!(bill_copies_2_on)) {
                app_state().lock().unwrap().bill_slip_draft.copies = 2;
                bill_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(bill_copies_3), ids!(bill_copies_3_on)) {
                app_state().lock().unwrap().bill_slip_draft.copies = 3;
                bill_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(bill_field_show), ids!(bill_field_show_on))
            {
                let mut st = app_state().lock().unwrap();
                let field = st.bill_style_field;
                st.bill_slip_draft.style_mut(field).show = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(cx, actions, ids!(bill_field_hide), ids!(bill_field_hide_on))
            {
                let mut st = app_state().lock().unwrap();
                let field = st.bill_style_field;
                st.bill_slip_draft.style_mut(field).show = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_layout_columns),
                ids!(bill_item_layout_columns_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_layout =
                    print_share::BillItemLayout::Columns;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_layout_stacked),
                ids!(bill_item_layout_stacked_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_layout =
                    print_share::BillItemLayout::Stacked;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_header_on),
                ids!(bill_item_header_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_header = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_header_off),
                ids!(bill_item_header_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_header = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_name_on),
                ids!(bill_item_name_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_name = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_name_off),
                ids!(bill_item_name_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_name = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_qty_on),
                ids!(bill_item_qty_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_qty = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_qty_off),
                ids!(bill_item_qty_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_qty = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_price_on),
                ids!(bill_item_price_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_price = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_price_off),
                ids!(bill_item_price_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_price = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_amount_on),
                ids!(bill_item_amount_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_amount = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_amount_off),
                ids!(bill_item_amount_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_amount = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_note_on),
                ids!(bill_item_note_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_note = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_note_off),
                ids!(bill_item_note_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_show_note = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_gap_tight),
                ids!(bill_item_gap_tight_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .bill_slip_draft
                    .set_item_col_gap_preset(print_share::BILL_COL_GAP_TIGHT);
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_gap_comfort),
                ids!(bill_item_gap_comfort_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .bill_slip_draft
                    .set_item_col_gap_preset(print_share::BILL_COL_GAP_COMFORT);
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_gap_wide),
                ids!(bill_item_gap_wide_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .bill_slip_draft
                    .set_item_col_gap_preset(print_share::BILL_COL_GAP_WIDE);
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_num_compact),
                ids!(bill_item_num_compact_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .bill_slip_draft
                    .set_item_num_min_preset(print_share::BILL_NUM_MIN_COMPACT);
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_num_comfort),
                ids!(bill_item_num_comfort_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .bill_slip_draft
                    .set_item_num_min_preset(print_share::BILL_NUM_MIN_COMFORT);
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_num_wide),
                ids!(bill_item_num_wide_on),
            ) {
                app_state()
                    .lock()
                    .unwrap()
                    .bill_slip_draft
                    .set_item_num_min_preset(print_share::BILL_NUM_MIN_WIDE);
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_num_start),
                ids!(bill_item_num_start_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_num_align =
                    print_share::KotAlign::Start;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_num_center),
                ids!(bill_item_num_center_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_num_align =
                    print_share::KotAlign::Center;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_num_end),
                ids!(bill_item_num_end_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_num_align =
                    print_share::KotAlign::End;
                bill_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_gap_em_input))
                .changed(actions)
            {
                if let Ok(v) = raw.trim().parse::<f32>() {
                    app_state().lock().unwrap().bill_slip_draft.item_col_gap_em =
                        print_share::clamp_bill_col_gap_em(v);
                    bill_changed = true;
                }
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_qty_min_input))
                .changed(actions)
            {
                if let Ok(v) = raw.trim().parse::<f32>() {
                    app_state().lock().unwrap().bill_slip_draft.item_qty_min_em =
                        print_share::clamp_bill_col_min_em(v);
                    bill_changed = true;
                }
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_price_min_input))
                .changed(actions)
            {
                if let Ok(v) = raw.trim().parse::<f32>() {
                    app_state().lock().unwrap().bill_slip_draft.item_price_min_em =
                        print_share::clamp_bill_col_min_em(v);
                    bill_changed = true;
                }
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_amount_min_input))
                .changed(actions)
            {
                if let Ok(v) = raw.trim().parse::<f32>() {
                    app_state().lock().unwrap().bill_slip_draft.item_amount_min_em =
                        print_share::clamp_bill_col_min_em(v);
                    bill_changed = true;
                }
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_hdr_item_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.item_header_item = raw;
                bill_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_hdr_qty_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.item_header_qty = raw;
                bill_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_hdr_price_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.item_header_price = raw;
                bill_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_item_hdr_amount_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.item_header_amount = raw;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_sep_on),
                ids!(bill_item_sep_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_col_sep = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_item_sep_off),
                ids!(bill_item_sep_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.item_col_sep = false;
                bill_changed = true;
            }
            for (i, idle, on) in [
                (0, ids!(bill_col_ord_0), ids!(bill_col_ord_0_on)),
                (1, ids!(bill_col_ord_1), ids!(bill_col_ord_1_on)),
                (2, ids!(bill_col_ord_2), ids!(bill_col_ord_2_on)),
                (3, ids!(bill_col_ord_3), ids!(bill_col_ord_3_on)),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    app_state().lock().unwrap().bill_item_col_edit_idx = i;
                    bill_changed = true;
                }
            }
            if self.ui.button(cx, ids!(bill_col_ord_up)).clicked(actions) {
                let mut st = app_state().lock().unwrap();
                if st.bill_slip_draft.item_col_order.is_empty() {
                    st.bill_slip_draft.item_col_order = print_share::BillItemCol::ALL
                        .iter()
                        .map(|c| c.id().to_string())
                        .collect();
                }
                let i = st
                    .bill_item_col_edit_idx
                    .min(st.bill_slip_draft.item_col_order.len().saturating_sub(1));
                if i > 0 {
                    st.bill_slip_draft.item_col_order.swap(i, i - 1);
                    st.bill_item_col_edit_idx = i - 1;
                    bill_changed = true;
                }
            }
            if self.ui.button(cx, ids!(bill_col_ord_down)).clicked(actions) {
                let mut st = app_state().lock().unwrap();
                if st.bill_slip_draft.item_col_order.is_empty() {
                    st.bill_slip_draft.item_col_order = print_share::BillItemCol::ALL
                        .iter()
                        .map(|c| c.id().to_string())
                        .collect();
                }
                let n = st.bill_slip_draft.item_col_order.len();
                let i = st.bill_item_col_edit_idx.min(n.saturating_sub(1));
                if i + 1 < n {
                    st.bill_slip_draft.item_col_order.swap(i, i + 1);
                    st.bill_item_col_edit_idx = i + 1;
                    bill_changed = true;
                }
            }
            for (idle, on, partner) in [
                (ids!(bill_pair_alone), ids!(bill_pair_alone_on), None),
                (ids!(bill_pair_customer), ids!(bill_pair_customer_on), Some(print_share::BillStyleField::Customer)),
                (ids!(bill_pair_billno), ids!(bill_pair_billno_on), Some(print_share::BillStyleField::BillNo)),
                (ids!(bill_pair_time), ids!(bill_pair_time_on), Some(print_share::BillStyleField::Time)),
                (ids!(bill_pair_cashier), ids!(bill_pair_cashier_on), Some(print_share::BillStyleField::Cashier)),
                (ids!(bill_pair_table), ids!(bill_pair_table_on), Some(print_share::BillStyleField::Table)),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    let mut st = app_state().lock().unwrap();
                    let field = st.bill_style_field;
                    if partner != Some(field) {
                        st.bill_slip_draft.set_meta_partner(field, partner);
                        bill_changed = true;
                    }
                }
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_total_qty_label_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.total_qty_label = raw;
                bill_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_total_subtotal_label_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.total_subtotal_label = raw;
                bill_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_total_grand_label_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.total_grand_label = raw;
                bill_changed = true;
            }
            if let Some(raw) = self
                .ui
                .text_input(cx, ids!(bill_total_paid_label_input))
                .changed(actions)
            {
                app_state().lock().unwrap().bill_slip_draft.total_paid_label = raw;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_total_qty_on),
                ids!(bill_total_qty_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.total_show_qty = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_total_qty_off),
                ids!(bill_total_qty_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.total_show_qty = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_total_subtotal_on),
                ids!(bill_total_subtotal_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.total_show_subtotal = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_total_subtotal_off),
                ids!(bill_total_subtotal_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.total_show_subtotal = false;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_total_grand_on),
                ids!(bill_total_grand_on_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.total_show_grand = true;
                bill_changed = true;
            }
            if self.choice_chip_clicked(
                cx,
                actions,
                ids!(bill_total_grand_off),
                ids!(bill_total_grand_off_on),
            ) {
                app_state().lock().unwrap().bill_slip_draft.total_show_grand = false;
                bill_changed = true;
            }
            for (idle, on, weight) in [
                (ids!(bill_w_light), ids!(bill_w_light_on), print_share::KotFontWeight::Light),
                (ids!(bill_w_reg), ids!(bill_w_reg_on), print_share::KotFontWeight::Regular),
                (ids!(bill_w_bold), ids!(bill_w_bold_on), print_share::KotFontWeight::Bold),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    let mut st = app_state().lock().unwrap();
                    let field = st.bill_style_field;
                    st.bill_slip_draft.style_mut(field).weight = weight;
                    bill_changed = true;
                    break;
                }
            }
            for (idle, on, align) in [
                (ids!(bill_a_start), ids!(bill_a_start_on), print_share::KotAlign::Start),
                (ids!(bill_a_center), ids!(bill_a_center_on), print_share::KotAlign::Center),
                (ids!(bill_a_end), ids!(bill_a_end_on), print_share::KotAlign::End),
            ] {
                if self.choice_chip_clicked(cx, actions, idle, on) {
                    let mut st = app_state().lock().unwrap();
                    let field = st.bill_style_field;
                    st.bill_slip_draft.style_mut(field).align = align;
                    bill_changed = true;
                    break;
                }
            }
            if self.ui.button(cx, ids!(bill_apply_all)).clicked(actions) {
                // The open line's show, size, weight, and align land on
                // every other bill line. Typed text stays on its own line.
                let mut st = app_state().lock().unwrap();
                let src = st.bill_style_field;
                let style = st.bill_slip_draft.style(src).clone();
                for field in print_share::BillStyleField::ALL {
                    if field == src {
                        continue;
                    }
                    *st.bill_slip_draft.style_mut(field) = style.clone();
                }
                bill_changed = true;
            }
            if bill_changed {
                self.sync_bill_format_ui(cx);
            }
        }
        {
            let size_now = self.ui.text_input(cx, ids!(std_size_input)).text();
            if let Ok(raw) = size_now.trim().parse::<f32>() {
                let em = raw.clamp(12.0, 48.0);
                // Release the guard before repainting: sync_std_print_ui locks
                // app_state itself (DEV.md bug 7a — typing a size hung Settings).
                let changed = {
                    let mut st = app_state().lock().unwrap();
                    let changed = (st.printer_draft.standard_em() - em).abs() > 0.05;
                    if changed {
                        st.printer_draft.set_standard_em(em);
                    }
                    changed
                };
                if changed {
                    self.sync_std_print_ui(cx);
                }
            }
        }
        {
            let picked = if self.ui.button(cx, ids!(content_width_58)).clicked(actions)
                || self.ui.button(cx, ids!(content_width_58_on)).clicked(actions)
            {
                Some(58u32)
            } else if self.ui.button(cx, ids!(content_width_80)).clicked(actions)
                || self.ui.button(cx, ids!(content_width_80_on)).clicked(actions)
            {
                Some(80)
            } else if self.ui.button(cx, ids!(content_width_112)).clicked(actions)
                || self.ui.button(cx, ids!(content_width_112_on)).clicked(actions)
            {
                Some(112)
            } else if self.ui.button(cx, ids!(content_width_210)).clicked(actions)
                || self.ui.button(cx, ids!(content_width_210_on)).clicked(actions)
            {
                Some(210)
            } else {
                None
            };
            if let Some(mm) = picked {
                self.ui
                    .text_input(cx, ids!(content_width_input))
                    .set_text(cx, &mm.to_string());
                self.apply_content_width(cx, mm);
            }
        }
        if let Some(text) = self
            .ui
            .text_input(cx, ids!(content_width_input))
            .changed(actions)
        {
            if let Ok(mm) = text.trim().parse::<u32>() {
                if (40..=210).contains(&mm) {
                    let current = {
                        let st = app_state().lock().unwrap();
                        if st.bill_kot_tab_kot {
                            st.kot_slip_draft.shown_content_width_mm()
                        } else {
                            st.bill_slip_draft.shown_content_width_mm()
                        }
                    };
                    if mm != current {
                        self.apply_content_width(cx, mm);
                    }
                }
            }
        }
        if self.ui.button(cx, ids!(bill_printers_dd)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                {
                    let mut st = app_state().lock().unwrap();
                    st.printer_route_key = "bill".into();
                    if st.suggest_open == Some(SuggestField::BillPrinters) {
                        st.suggest_open = None;
                    } else {
                        st.open_suggest(SuggestField::BillPrinters, "");
                    }
                }
                self.sync_job_drive_ui(cx);
                self.ui.widget(cx, ids!(bill_printers_suggest)).redraw(cx);
            }
        }
        if self.ui.button(cx, ids!(printers_test_dd)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let empty = {
                    let mut st = app_state().lock().unwrap();
                    if st.suggest_open == Some(SuggestField::TestPrinters) {
                        st.suggest_open = None;
                    } else if st.printer_draft.printers.is_empty() {
                        st.suggest_open = None;
                    } else {
                        st.open_suggest(SuggestField::TestPrinters, "");
                    }
                    st.printer_draft.printers.is_empty()
                };
                if empty {
                    self.set_status(cx, app_t("printers_none_hint"));
                }
                self.paint_printers_panel(cx);
                self.ui.widget(cx, ids!(printers_test_suggest)).redraw(cx);
            }
        }
        if self.ui.button(cx, ids!(kot_test_printers_dd)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let empty = {
                    let mut st = app_state().lock().unwrap();
                    if st.suggest_open == Some(SuggestField::KotTestPrinters) {
                        st.suggest_open = None;
                    } else if st.printer_draft.printers.is_empty() {
                        st.suggest_open = None;
                    } else {
                        st.open_suggest(SuggestField::KotTestPrinters, "");
                    }
                    st.printer_draft.printers.is_empty()
                };
                if empty {
                    self.set_status(cx, app_t("printers_none_hint"));
                }
                self.sync_kot_format_ui(cx);
                self.ui
                    .widget(cx, ids!(kot_test_printers_suggest))
                    .redraw(cx);
            }
        }
        for (standard, idle, on, landscape, delivery) in [
            (true, ids!(bill_layout_portrait), ids!(bill_layout_portrait_on), Some(false), None),
            (true, ids!(bill_layout_landscape), ids!(bill_layout_landscape_on), Some(true), None),
            (false, ids!(kot_layout_portrait), ids!(kot_layout_portrait_on), Some(false), None),
            (false, ids!(kot_layout_landscape), ids!(kot_layout_landscape_on), Some(true), None),
            (true, ids!(bill_deliv_os), ids!(bill_deliv_os_on), None, Some(print_share::DELIVERY_SYSTEM_SPOOLER)),
            (true, ids!(bill_deliv_raw), ids!(bill_deliv_raw_on), None, Some(print_share::DELIVERY_RAW_DEVICE)),
            (true, ids!(bill_deliv_escpos), ids!(bill_deliv_escpos_on), None, Some(print_share::DELIVERY_ESCPOS)),
            (false, ids!(kot_deliv_os), ids!(kot_deliv_os_on), None, Some(print_share::DELIVERY_SYSTEM_SPOOLER)),
            (false, ids!(kot_deliv_raw), ids!(kot_deliv_raw_on), None, Some(print_share::DELIVERY_RAW_DEVICE)),
            (false, ids!(kot_deliv_escpos), ids!(kot_deliv_escpos_on), None, Some(print_share::DELIVERY_ESCPOS)),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, on) {
                self.apply_job_drive(cx, standard, landscape, delivery);
            }
        }
        if self.ui.button(cx, ids!(std_print_save)).clicked(actions) {
            // Always via books_op so a Client updates the Server's printers.json
            // (and meta), never this till's local shop-wide printers file.
            let size_now = self.ui.text_input(cx, ids!(std_size_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                if let Ok(raw) = size_now.trim().parse::<f32>() {
                    st.printer_draft.set_standard_em(raw.clamp(12.0, 48.0));
                }
                let em = st.printer_draft.standard_em();
                st.books_op(BooksOp::SetStandardEm { em })?;
                if st.device_role != DeviceRole::Client {
                    st.printer_draft.set_standard_em(em);
                    print_share::configure(st.printer_draft.clone());
                }
                Ok(st.t("std_print_saved"))
            })();
            match result {
                Ok(msg) => {
                    self.sync_std_print_ui(cx);
                    self.set_status(cx, msg);
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
                let mut st = app_state().lock().unwrap();
                st.books_op(BooksOp::ClearPrintStamp)?;
                Ok(())
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
                let mut st = app_state().lock().unwrap();
                st.books_op(BooksOp::ClearPaymentQr)?;
                Ok(())
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
        if self.ui.button(cx, ids!(aggregator_import_cancel)).clicked(actions) {
            request_books_job_cancel();
            self.paint_submit_busy_chrome(cx);
        }
        if self.ui.button(cx, ids!(aggregator_import_btn)).clicked(actions) {
            if aggregator_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
                // Progress — ignore re-taps.
            } else {
                let is_client =
                    app_state().lock().unwrap().device_role == DeviceRole::Client;
                if is_client {
                    let sent = (|| -> Result<String, String> {
                        let mut st = app_state().lock().unwrap();
                        st.books_op(BooksOp::ImportAggregator)
                    })();
                    match sent {
                        Ok(msg) => self.set_status(cx, msg),
                        Err(e) => self.set_status(cx, e),
                    }
                } else {
                    let job = (|| -> Result<PathBuf, String> {
                        let st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        // Same person rules as BooksOp::ImportAggregator.
                        if !actor.is_owner() && !actor.can_access_settings() {
                            return Err("Not allowed to import aggregator orders".into());
                        }
                        Ok(st
                            .store
                            .as_ref()
                            .ok_or_else(|| "No store".to_string())?
                            .db_path()
                            .to_path_buf())
                    })();
                    match job {
                        Ok(db) => {
                            aggregator_import_job(db);
                            self.paint_submit_busy_chrome(cx);
                        }
                        Err(e) => self.set_status(cx, e),
                    }
                }
            }
        }
        if self.ui.button(cx, ids!(scheme_add)).clicked(actions) {
            let text = self.ui.text_input(cx, ids!(scheme_bogo_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let (buy_name, buy_qty, get_name, get_qty) =
                    schemes::Scheme::parse_bogo_human(&text)
                        .ok_or_else(|| "Type like: Buy 2 Thali get 1 Chaas".to_string())?;
                let products = st.products.clone();
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
                st.books_op(BooksOp::UpsertScheme { scheme })
            })();
            match result {
                Ok(msg) => {
                    self.sync_selling_panel(cx);
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
                        &net::RemoteKotAdvance { kot_id, from: String::new() },
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
        let (team_pick_now, team_copy_hire_now, team_copy_edit_now) = {
            let mut st = app_state().lock().unwrap();
            (
                std::mem::take(&mut st.team_select_pending),
                std::mem::take(&mut st.team_copy_hire_pending),
                std::mem::take(&mut st.team_copy_edit_pending),
            )
        };
        if team_pick_now || self.ui.text_input(cx, ids!(team_pick_input)).returned(actions).is_some()
        {
            let name = self.ui.text_input(cx, ids!(team_pick_input)).text();
            let result = (|| -> Result<(String, String, String, String), String> {
                let mut st = app_state().lock().unwrap();
                let member = Self::find_active_staff(&st, name.trim())?;
                // Non-Owner cannot open self for screens/rights/job edit.
                if st
                    .session
                    .as_ref()
                    .is_some_and(|s| !s.is_owner() && s.id == member.id)
                {
                    return Err(
                        "You cannot change your own screens or rights — ask the Owner".into(),
                    );
                }
                st.team_staff_id = Some(member.id.clone());
                // Edit draft only — hire Job chips keep hire_* untouched.
                st.draft_staff_role = store::Store::hire_role_from(&member);
                st.draft_features = member.effective_features();
                let msg = format!("Editing {}", member.lock_label());
                Ok((
                    msg,
                    member.name.clone(),
                    member.lock_label(),
                    member.login_id.clone(),
                ))
            })();
            match result {
                Ok((msg, display_name, pick_label, login_id)) => {
                    self.ui
                        .text_input(cx, ids!(team_pick_input))
                        .set_text(cx, &pick_label);
                    self.ui
                        .text_input(cx, ids!(staff_edit_name_input))
                        .set_text(cx, &display_name);
                    self.ui
                        .text_input(cx, ids!(staff_edit_login_input))
                        .set_text(cx, &login_id);
                    self.ui
                        .text_input(cx, ids!(staff_edit_password_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        let hire_copy_click = self.ui.button(cx, ids!(staff_copy_hire_btn)).clicked(actions)
            || team_copy_hire_now
            || self
                .ui
                .text_input(cx, ids!(staff_copy_hire_input))
                .returned(actions)
                .is_some();
        if hire_copy_click {
            let name = self.ui.text_input(cx, ids!(staff_copy_hire_input)).text();
            let result = (|| -> Result<(String, String, String), String> {
                let mut st = app_state().lock().unwrap();
                let source = Self::find_active_staff(&st, name.trim())?;
                // Owner is not a hire-copy source — Manager+screens would over-grant
                // a Team manager. Use a staff peer, or Job chips.
                if source.is_owner() {
                    return Err(
                        "Pick a staff member to copy — not the Owner".into(),
                    );
                }
                let role = store::Store::hire_role_from(&source);
                let title = if source.role_title.trim().is_empty() {
                    role.default_title().to_string()
                } else {
                    source.role_title.clone()
                };
                // Hire copy fills the Add form (hire draft only) — clear any
                // edit selection so screens/rights below do not look like they
                // belong to someone. Cap to actor ceiling so Add is hireable
                // (store still refuses crafted over-ceiling requests).
                let actor = st.actor()?.clone();
                st.team_staff_id = None;
                st.hire_staff_role = role;
                st.hire_features = store::Store::cap_team_features(
                    &source.effective_features(),
                    role,
                    &actor,
                );
                Ok((
                    st.t("staff_setup_copied_hire")
                        .replace("{name}", &source.lock_label()),
                    title,
                    source.lock_label(),
                ))
            })();
            match result {
                Ok((msg, title, source_label)) => {
                    self.ui.text_input(cx, ids!(team_pick_input)).set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(staff_edit_name_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(staff_copy_hire_input))
                        .set_text(cx, &source_label);
                    self.ui
                        .text_input(cx, ids!(staff_title_input))
                        .set_text(cx, &title);
                    // Keep name + PIN blank — this is a new person.
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(staff_rename_btn)).clicked(actions) {
            let new_name = self.ui.text_input(cx, ids!(staff_edit_name_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let staff_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| st.t("staff_rename_need_selected"))?;
                st.books_op(BooksOp::RenameStaff {
                    staff_id,
                    name: new_name,
                })?;
                st.status = st.t("staff_renamed");
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }
        let edit_copy_click = self.ui.button(cx, ids!(staff_copy_edit_btn)).clicked(actions)
            || team_copy_edit_now
            || self
                .ui
                .text_input(cx, ids!(staff_copy_edit_input))
                .returned(actions)
                .is_some();
        if edit_copy_click {
            let from_name = self.ui.text_input(cx, ids!(staff_copy_edit_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let target_id = st
                    .team_staff_id
                    .clone()
                    .ok_or_else(|| st.t("staff_copy_need_selected"))?;
                let source = Self::find_active_staff(&st, from_name.trim())?;
                if source.id == target_id {
                    return Err(st.t("staff_copy_same"));
                }
                let target = st
                    .staff
                    .iter()
                    .find(|s| s.id == target_id)
                    .cloned()
                    .ok_or_else(|| target_id.clone())?;
                if target.is_owner() {
                    return Err("Cannot change Owner setup this way".into());
                }
                let from_label = source.lock_label();
                let to_label = target.lock_label();
                let role = store::Store::hire_role_from(&source);
                let title = if source.is_owner() {
                    role.default_title().to_string()
                } else if source.role_title.trim().is_empty() {
                    role.default_title().to_string()
                } else {
                    source.role_title.clone()
                };
                let features = source.effective_features();
                st.books_op(BooksOp::ApplyStaffConfig {
                    staff_id: target_id,
                    role,
                    role_title: title,
                    features: features.clone(),
                    from_name: from_label.clone(),
                    to_name: to_label.clone(),
                })?;
                st.draft_staff_role = role;
                st.draft_features = features;
                Ok(if st.device_role == DeviceRole::Client {
                    format!("Copied setup from {from_label} onto {to_label}")
                } else {
                    st.t("staff_setup_copied_edit")
                        .replace("{from}", &from_label)
                        .replace("{to}", &to_label)
                })
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for (idle, on, right, level) in [
            (
                ids!(right_settle_allowed),
                ids!(right_settle_allowed_on),
                domain::Right::Settle,
                domain::RightLevel::Allowed,
            ),
            (
                ids!(right_settle_ask),
                ids!(right_settle_ask_on),
                domain::Right::Settle,
                domain::RightLevel::WithApproval,
            ),
            (
                ids!(right_settle_deny),
                ids!(right_settle_deny_on),
                domain::Right::Settle,
                domain::RightLevel::Denied,
            ),
            (
                ids!(right_cancel_allowed),
                ids!(right_cancel_allowed_on),
                domain::Right::CancelSentItem,
                domain::RightLevel::Allowed,
            ),
            (
                ids!(right_cancel_ask),
                ids!(right_cancel_ask_on),
                domain::Right::CancelSentItem,
                domain::RightLevel::WithApproval,
            ),
            (
                ids!(right_cancel_deny),
                ids!(right_cancel_deny_on),
                domain::Right::CancelSentItem,
                domain::RightLevel::Denied,
            ),
            (
                ids!(right_nc_allowed),
                ids!(right_nc_allowed_on),
                domain::Right::Complimentary,
                domain::RightLevel::Allowed,
            ),
            (
                ids!(right_nc_ask),
                ids!(right_nc_ask_on),
                domain::Right::Complimentary,
                domain::RightLevel::WithApproval,
            ),
            (
                ids!(right_nc_deny),
                ids!(right_nc_deny_on),
                domain::Right::Complimentary,
                domain::RightLevel::Denied,
            ),
            (
                ids!(right_move_allowed),
                ids!(right_move_allowed_on),
                domain::Right::MoveTable,
                domain::RightLevel::Allowed,
            ),
            (
                ids!(right_move_ask),
                ids!(right_move_ask_on),
                domain::Right::MoveTable,
                domain::RightLevel::WithApproval,
            ),
            (
                ids!(right_move_deny),
                ids!(right_move_deny_on),
                domain::Right::MoveTable,
                domain::RightLevel::Denied,
            ),
            (
                ids!(right_reprint_allowed),
                ids!(right_reprint_allowed_on),
                domain::Right::Reprint,
                domain::RightLevel::Allowed,
            ),
            (
                ids!(right_reprint_ask),
                ids!(right_reprint_ask_on),
                domain::Right::Reprint,
                domain::RightLevel::WithApproval,
            ),
            (
                ids!(right_reprint_deny),
                ids!(right_reprint_deny_on),
                domain::Right::Reprint,
                domain::RightLevel::Denied,
            ),
            (
                ids!(right_discount_allowed),
                ids!(right_discount_allowed_on),
                domain::Right::Discount,
                domain::RightLevel::Allowed,
            ),
            (
                ids!(right_discount_ask),
                ids!(right_discount_ask_on),
                domain::Right::Discount,
                domain::RightLevel::WithApproval,
            ),
            (
                ids!(right_discount_deny),
                ids!(right_discount_deny_on),
                domain::Right::Discount,
                domain::RightLevel::Denied,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, on) {
                let msg = {
                    let mut st = app_state().lock().unwrap();
                    if st.team_staff_id.is_none() {
                        st.t("staff_copy_need_selected")
                    } else {
                        let mut rights = st.team_draft_rights();
                        rights.set_level(right, level);
                        st.draft_features.rights = Some(rights);
                        st.t("team_rights_changed")
                    }
                };
                self.set_status(cx, msg);
                self.sync_chrome(cx);
            }
        }
        // Digits only — never rewrite the field while focused beyond a filter
        // (same idea as Lock PIN; avoids IME thrash on Android).
        if let Some(text) = self.ui.text_input(cx, ids!(client_code_input)).changed(actions) {
            // Shop codes are six digits (leading zeros count).
            let digits: String = text.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
            if digits != text {
                self.ui.text_input(cx, ids!(client_code_input)).set_text(cx, &digits);
            }
        }
        // Enter on host/code advances via `form_returned_advanced`; last field / button pairs.
        let pair_enter = !form_returned_advanced
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
            // Three tablets must not all be "Android device" in the shop's list.
            let name = Some(self.ui.text_input(cx, ids!(client_name_input)).text())
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(device_name);
            let expect_server_id = {
                let st = app_state().lock().unwrap();
                let host_trim = host.trim().to_string();
                st.found_servers
                    .iter()
                    .find(|f| f.host == host_trim)
                    .map(|f| f.server_id.clone())
                    .filter(|id| !id.trim().is_empty())
            };
            self.start_client_pair(cx, host, code, name, expect_server_id);
        }
        if self.ui.button(cx, ids!(client_sync_btn)).clicked(actions)
            && !client_pair_busy().load(std::sync::atomic::Ordering::SeqCst)
        {
            phone::want_sync();
            let msg = "Refreshing from the shop…";
            self.ui
                .widget(cx, ids!(client_pair_status))
                .set_text(cx, msg);
            self.set_status(cx, msg);
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
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&amount_text, &currency)?;
                    let msg = st.books_op(BooksOp::PostMoney {
                        kind: kind.as_str().to_string(),
                        amount_paise: minor,
                        note,
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.day_close_text = st
                            .store
                            .as_ref()
                            .ok_or_else(|| "No store".to_string())?
                            .books_report_text("today")?;
                    }
                    st.nav_back.clear();
                    st.nav_forward.clear();
                    st.screen = Screen::Reports;
                    st.status = msg;
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
                    let mut st = app_state().lock().unwrap();
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
                    st.books_op(BooksOp::SetMovementCleared { id, cleared })
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
                let existing = st.find_stock_item(&name)?;
                let (product_id, new_item_name, new_item_unit) = match existing {
                    Some(p) => (p.id, String::new(), String::new()),
                    None if inv_plus => (String::new(), name.trim().to_string(), String::new()),
                    None => return Err(format!("Nothing called \"{}\" in stock", name.trim())),
                };
                let delta = if inv_plus { qty } else { -qty };
                let msg = st.books_op(BooksOp::AdjustStock {
                    product_id,
                    new_item_name,
                    new_item_unit,
                    delta_qty: delta,
                    note,
                })?;
                if st.device_role != DeviceRole::Client {
                    st.refresh_inventory_text()?;
                }
                Ok(msg)
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
                let party_n: i64 = party.trim().parse().unwrap_or(2);
                let wait_n: i64 = mins.trim().parse().unwrap_or(15);
                let msg = st.books_op(BooksOp::AddWaitlist {
                    name: name.trim().to_string(),
                    party_size: party_n,
                    quoted_mins: wait_n,
                })?;
                if st.device_role != DeviceRole::Client {
                    let _ = st.refresh_inventory_text();
                }
                Ok(msg)
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
                let typed = cat.trim();
                let category_id = st
                    .store
                    .as_ref()
                    .ok_or("No store")?
                    .list_categories()?
                    .into_iter()
                    .find(|c| c.id == typed || c.name.eq_ignore_ascii_case(typed))
                    .map(|c| c.id)
                    .unwrap_or_else(|| typed.to_string());
                let percent: Option<i64> = pct.trim().parse().ok();
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
                st.books_op(BooksOp::BulkPrice {
                    category_id,
                    percent,
                    set_paise,
                })
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
                let phone = with_calling_code(&phone_raw, &st.fiscal.country_code);
                let guests: i64 = if guests.trim().is_empty() {
                    0
                } else {
                    guests.trim().parse().map_err(|_| "Guests must be a whole number".to_string())?
                };
                st.books_op(BooksOp::AddReservation {
                    name,
                    phone,
                    guests,
                    at: when,
                    table_name: table,
                })
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
                    let r = st.reservations.get(index).cloned().ok_or("Booking not found")?;
                    st.books_op(BooksOp::SetReservationStatus {
                        id: r.id,
                        status: status.to_string(),
                        guest_name: r.name,
                        table_name: r.table_name,
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
                st.books_op(BooksOp::SetReorder {
                    product_id: product.id,
                    level,
                    name: product.name,
                    unit_label: product.unit_label,
                })
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
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can add branches".into());
                }
                let msg = st.books_op(BooksOp::AddBranch { name, code })?;
                if st.device_role != DeviceRole::Client {
                    st.refresh_branches_text()?;
                }
                Ok(msg)
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
        if self.ui.button(cx, ids!(branch_switch)).clicked(actions)
            || self
                .ui
                .text_input(cx, ids!(branch_switch_input))
                .returned(actions)
                .is_some()
        {
            let wanted = self.ui.text_input(cx, ids!(branch_switch_input)).text();
            match self.switch_active_branch(cx, &wanted) {
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
                let msg = st.books_op(BooksOp::ImportBranchSnapshot { json })?;
                if st.device_role != DeviceRole::Client {
                    st.refresh_branches_text()?;
                }
                Ok(msg)
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
                    .is_some_and(finger_up_is_tap)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
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
                    let msg = st.books_op(BooksOp::SetUnitPicker {
                        code: code.clone(),
                        enabled: !enabled,
                    })?;
                    st.status = msg;
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
                    .is_some_and(finger_up_is_tap)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let rows = sibling_pickers_rows(store, |k| st.t(k));
                    let key = match rows.get(item_id) {
                        Some(ReportRow::Kv(k, _)) => k.clone(),
                        _ => return Err("Not a picker row".into()),
                    };
                    let msg = if let Some(name) = key.strip_prefix("area:") {
                        let on = store
                            .list_picker_areas()?
                            .into_iter()
                            .find(|(n, _)| n == name)
                            .map(|(_, o)| o)
                            .unwrap_or(true);
                        let name = name.to_string();
                        st.books_op(BooksOp::SetAreaPicker {
                            name,
                            enabled: !on,
                        })?
                    } else if let Some(rest) = key.strip_prefix("station:") {
                        let id = rest.split(" · ").next().unwrap_or("").trim().to_string();
                        let on = store
                            .list_stations()?
                            .into_iter()
                            .find(|s| s.id == id)
                            .map(|s| s.picker_enabled)
                            .unwrap_or(true);
                        st.books_op(BooksOp::SetStationPicker {
                            id,
                            enabled: !on,
                        })?
                    } else if let Some(rest) = key.strip_prefix("tax:") {
                        let id = rest.split(" · ").next().unwrap_or("").trim().to_string();
                        let on = store
                            .list_tax_rates()?
                            .into_iter()
                            .find(|r| r.id == id)
                            .map(|r| r.picker_enabled)
                            .unwrap_or(true);
                        st.books_op(BooksOp::SetTaxPicker {
                            id,
                            enabled: !on,
                        })?
                    } else {
                        return Err("Unknown picker row".into());
                    };
                    st.status = msg;
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
                let msg = st.books_op(BooksOp::AddUnit {
                    code: code.clone(),
                    name,
                })?;
                st.status = msg;
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
                let msg = st.books_op(BooksOp::AddUnitConversion {
                    from_code: from,
                    to_code: to,
                    factor_milli,
                })?;
                st.status = msg;
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

        if self.ui.button(cx, ids!(find_printers_cancel)).clicked(actions) {
            // Flag + wake only — never set_text Looking on the Find Stack*
            // button (that wrap path freezes Settings; DEV.md bug 7).
            print_share::cancel_find_printers();
            self.show_found_printers(cx);
            self.paint_submit_busy_chrome(cx);
        }
        if self.ui.button(cx, ids!(find_printers_btn)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else if print_share::is_finding_printers() {
                // Looking… — ignore re-taps.
            } else {
                print_share::find_printers_in_background();
                // Immediate Looking… on the found line — do not leave
                // "Tap Find…" over the saved-pool card (looked like Find
                // returned only the already-added printer).
                self.show_found_printers(cx);
                self.paint_submit_busy_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(printer_add_cancel)).clicked(actions) {
            // Legacy Cancel (async add removed) — clear any stuck busy flag.
            printer_add_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
            printer_add_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            *printer_add_status().lock().unwrap() = None;
            self.paint_submit_busy_chrome(cx);
        }
        // Add/Remove save printers.json on this thread. Refresh only the
        // printers panel — full sync_chrome froze Settings after Add.
        // Host-only: a Client must not write shop-wide printers.json.
        if self.ui.button(cx, ids!(printer_add)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let target = self.ui.text_input(cx, ids!(printer_target_input)).text();
                let result = {
                    let mut st = app_state().lock().unwrap();
                    if target.trim().is_empty() {
                        Err(st.t("printer_target_ph"))
                    } else {
                        let (_id, newly) = st.printer_draft.ensure_printer(&target, &target);
                        st.printer_selected_id = None;
                        if !newly {
                            Ok(st.t("printer_already_saved").replace("{name}", target.trim()))
                        } else {
                            st.save_printer_draft()
                                .map(|()| st.t("printers_saved"))
                        }
                    }
                };
                self.ui.text_input(cx, ids!(printer_target_input)).set_text(cx, "");
                printer_add_busy().store(false, std::sync::atomic::Ordering::SeqCst);
                *printer_add_status().lock().unwrap() = None;
                match result {
                    Ok(msg) | Err(msg) => self.set_status(cx, msg),
                }
                self.paint_submit_busy_chrome(cx);
                self.paint_printers_panel(cx);
            }
        }
        if self.ui.button(cx, ids!(printer_remove)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let result = {
                    let mut st = app_state().lock().unwrap();
                    let id = st
                        .printer_selected_id
                        .clone()
                        .or_else(|| st.printer_draft.printers.last().map(|p| p.id.clone()));
                    if let Some(id) = id {
                        st.printer_draft.remove_printer(&id);
                    }
                    st.printer_selected_id = None;
                    st.save_printer_draft()
                        .map(|()| st.t("printers_saved"))
                };
                printer_add_busy().store(false, std::sync::atomic::Ordering::SeqCst);
                *printer_add_status().lock().unwrap() = None;
                match result {
                    Ok(msg) | Err(msg) => self.set_status(cx, msg),
                }
                self.paint_submit_busy_chrome(cx);
                self.paint_printers_panel(cx);
            }
        }
        // Settings → Printers pool: tap a row to select one printer for
        // test page / Remove. Never "print on every device or none".
        {
            let widget = self.ui.widget(cx, ids!(printers_pool_label));
            let list = widget.portal_list(cx, ids!(list));
            let mut tapped: Option<usize> = None;
            for (item_id, item) in list.items_with_actions(actions) {
                if item
                    .view(cx, ids!(row))
                    .finger_up(actions)
                    .is_some_and(finger_up_is_tap)
                {
                    tapped = Some(item_id);
                }
            }
            if let Some(item_id) = tapped {
                let mut st = app_state().lock().unwrap();
                if st.device_role != DeviceRole::Client {
                    let rows = printers_pool_rows(
                        &st.printer_draft,
                        st.printer_selected_id.as_deref(),
                        |k| st.t(k),
                    );
                    if matches!(rows.get(item_id), Some(ReportRow::Kv(_, _))) {
                        if let Some(p) = st.printer_draft.printers.get(item_id).cloned() {
                            st.printer_selected_id = Some(p.id.clone());
                            let name = printer_display_name(&p, |k| st.t(k));
                            st.status = st.t("real_printer_selected").replace("{name}", &name);
                        }
                    }
                }
                drop(st);
                self.paint_printers_panel(cx);
            }
        }
        // Find auto-merges discoveries into the pool — no tap-to-add on the
        // found list. Legacy hidden place chips (kept for hot-reload ids) —
        // dropdown is SuggestField::PrinterPlace via printer_place_dd.
        {
            let mut tapped: Option<usize> = None;
            for (i, (idle, active)) in ROUTE_PLACE_BTNS.iter().enumerate() {
                if self.choice_chip_clicked(cx, actions, idle, active) {
                    tapped = Some(i);
                    break;
                }
            }
            if let Some(i) = tapped {
                let (key, name) = {
                    let st = app_state().lock().unwrap();
                    let choices = printer_place_choices(&st.stations, "", |k| st.t(k));
                    choices
                        .get(i)
                        .map(|(k, _, _)| {
                            let name = printer_route_display_name(k, &st.stations, |k| st.t(k));
                            (k.clone(), name)
                        })
                        .unwrap_or_else(|| ("bill".into(), st.t("route_bill")))
                };
                {
                    let mut st = app_state().lock().unwrap();
                    st.printer_route_key = key;
                    st.printer_route_sel = None;
                }
                let msg = {
                    let st = app_state().lock().unwrap();
                    st.t("route_editing").replace("{name}", &name)
                };
                self.set_status(cx, msg);
                self.paint_printers_panel(cx);
            }
        }
        // Assign / remove / priority: SuggestField::PrinterAssign checklist
        // (printer_assign_dd). Legacy +chips and tap-to-remove stay dormant.
        if self.ui.button(cx, ids!(route_clear)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    let key_str = st.printer_route_key.clone();
                    let place = printer_route_display_name(
                        &key_str,
                        &st.stations,
                        |k| st.t(k),
                    );
                    let key = parse_printer_route_key(&key_str);
                    st.printer_draft.route_mut(key).clear();
                    st.printer_route_sel = None;
                    st.save_printer_draft()?;
                    Ok(st.t("route_cleared").replace("{name}", &place))
                })();
                match result {
                    Ok(msg) => self.set_status(cx, msg),
                    Err(e) => self.set_status(cx, e),
                }
                self.paint_printers_panel(cx);
            }
        }
        // Virtual-printer draft: layout + delivery only (size/wrap/margins = content).
        let mut vp_values_changed = false;
        for (idle, on, landscape) in [
            (ids!(printer_wire_os), ids!(printer_wire_os_on), false),
            (
                ids!(printer_wire_os_landscape),
                ids!(printer_wire_os_landscape_on),
                true,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, on) {
                let mut st = app_state().lock().unwrap();
                if st.device_role == DeviceRole::Client {
                    drop(st);
                    self.set_status(cx, app_t("printers_host_only"));
                } else {
                    st.printer_vp_landscape = landscape;
                    vp_values_changed = true;
                }
                break;
            }
        }
        for (idle, on, delivery) in [
            (
                ids!(printer_mode_first),
                ids!(printer_mode_first_on),
                print_share::DELIVERY_SYSTEM_SPOOLER,
            ),
            (
                ids!(printer_mode_all),
                ids!(printer_mode_all_on),
                print_share::DELIVERY_RAW_DEVICE,
            ),
            (
                ids!(vp_deliv_escpos),
                ids!(vp_deliv_escpos_on),
                print_share::DELIVERY_ESCPOS,
            ),
        ] {
            if self.choice_chip_clicked(cx, actions, idle, on) {
                let mut st = app_state().lock().unwrap();
                if st.device_role == DeviceRole::Client {
                    drop(st);
                    self.set_status(cx, app_t("printers_host_only"));
                } else {
                    st.printer_vp_delivery = delivery.into();
                    vp_values_changed = true;
                }
                break;
            }
        }
        if vp_values_changed {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let values = print_share::VirtualPrinterValues {
                    landscape: st.printer_vp_landscape,
                    em: st.printer_vp_em,
                    line: st.printer_vp_line,
                    side: st.printer_vp_side,
                    delivery: st.printer_vp_delivery.clone(),
                    paper_width_mm: st.printer_vp_paper_width_mm,
                };
                if let Some(id) = st.virtual_printer_sel.clone() {
                    st.printer_draft.update_virtual_printer_values(&id, values);
                    st.save_printer_draft()?;
                    Ok(st.t("virtual_printer_updated"))
                } else {
                    Ok(st.t("virtual_draft_layout"))
                }
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.paint_printers_panel(cx);
        }
        {
            let picked = if self.ui.button(cx, ids!(vp_paper_58)).clicked(actions)
                || self.ui.button(cx, ids!(vp_paper_58_on)).clicked(actions)
            {
                Some(58u32)
            } else if self.ui.button(cx, ids!(vp_paper_80)).clicked(actions)
                || self.ui.button(cx, ids!(vp_paper_80_on)).clicked(actions)
            {
                Some(80)
            } else if self.ui.button(cx, ids!(vp_paper_112)).clicked(actions)
                || self.ui.button(cx, ids!(vp_paper_112_on)).clicked(actions)
            {
                Some(112)
            } else if self.ui.button(cx, ids!(vp_paper_210)).clicked(actions)
                || self.ui.button(cx, ids!(vp_paper_210_on)).clicked(actions)
            {
                Some(210)
            } else {
                None
            };
            if let Some(mm) = picked {
                let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
                if is_client {
                    self.set_status(cx, app_t("printers_host_only"));
                } else {
                    {
                        let mut st = app_state().lock().unwrap();
                        st.printer_vp_paper_width_mm = mm;
                        if let Some(id) = st.virtual_printer_sel.clone() {
                            let values = print_share::VirtualPrinterValues {
                                landscape: st.printer_vp_landscape,
                                em: st.printer_vp_em,
                                line: st.printer_vp_line,
                                side: st.printer_vp_side,
                                delivery: st.printer_vp_delivery.clone(),
                                paper_width_mm: mm,
                            };
                            st.printer_draft
                                .update_virtual_printer_values(&id, values);
                            let _ = st.save_printer_draft();
                        }
                    }
                    self.ui
                        .text_input(cx, ids!(vp_paper_width_input))
                        .set_text(cx, &mm.to_string());
                    self.paint_printers_panel(cx);
                }
            }
        }
        {
            let mm_now = self.ui.text_input(cx, ids!(vp_paper_width_input)).text();
            if let Ok(mm) = mm_now.trim().parse::<u32>() {
                let mut st = app_state().lock().unwrap();
                if st.printer_vp_paper_width_mm != mm && (40..=210).contains(&mm) {
                    st.printer_vp_paper_width_mm = mm;
                    if st.device_role != DeviceRole::Client {
                        if let Some(id) = st.virtual_printer_sel.clone() {
                            let values = print_share::VirtualPrinterValues {
                                landscape: st.printer_vp_landscape,
                                em: st.printer_vp_em,
                                line: st.printer_vp_line,
                                side: st.printer_vp_side,
                                delivery: st.printer_vp_delivery.clone(),
                                paper_width_mm: mm,
                            };
                            st.printer_draft.update_virtual_printer_values(&id, values);
                            let _ = st.save_printer_draft();
                        }
                    }
                }
            }
        }
        if self.ui.button(cx, ids!(virtual_printer_new_btn)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                {
                    let mut st = app_state().lock().unwrap();
                    st.virtual_printer_sel = None;
                    st.printer_vp_landscape = false;
                    st.printer_vp_em = print_share::STD_EM_NORMAL;
                    st.printer_vp_line = 40;
                    st.printer_vp_side = print_share::STD_SIDE;
                    st.printer_vp_delivery = print_share::DELIVERY_SYSTEM_SPOOLER.into();
                    st.printer_vp_paper_width_mm = 80;
                }
                self.ui
                    .text_input(cx, ids!(station_group_name_input))
                    .set_text(cx, "");
                self.set_status(cx, app_t("virtual_printer_new"));
                self.paint_printers_panel(cx);
            }
        }
        if self.ui.button(cx, ids!(station_group_save_btn)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let name = self.ui.text_input(cx, ids!(station_group_name_input)).text();
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    let Some(pid) = st.printer_selected_id.clone() else {
                        return Err(st.t("virtual_need_real"));
                    };
                    let values = print_share::VirtualPrinterValues {
                        landscape: st.printer_vp_landscape,
                        em: st.printer_vp_em,
                        line: st.printer_vp_line,
                        side: st.printer_vp_side,
                        delivery: st.printer_vp_delivery.clone(),
                        paper_width_mm: st.printer_vp_paper_width_mm,
                    };
                    if let Some(sel) = st.virtual_printer_sel.clone() {
                        if let Some(vp) = st
                            .printer_draft
                            .virtual_printers
                            .iter_mut()
                            .find(|v| v.id == sel)
                        {
                            if !name.trim().is_empty() {
                                vp.name = name.trim().to_string();
                            }
                            vp.printer_id = pid;
                            vp.apply_values(values);
                        }
                        st.save_printer_draft()?;
                        let label = st
                            .printer_draft
                            .virtual_printer_by_id(&sel)
                            .map(|v| v.name.clone())
                            .unwrap_or_default();
                        return Ok(st.t("virtual_printer_updated_named").replace("{name}", &label));
                    }
                    let t = name.trim();
                    if !t.is_empty() {
                        if let Some(existing) = st
                            .printer_draft
                            .virtual_printers
                            .iter()
                            .find(|v| v.name.eq_ignore_ascii_case(t) || v.id == t)
                            .map(|v| v.id.clone())
                        {
                            if let Some(vp) = st
                                .printer_draft
                                .virtual_printers
                                .iter_mut()
                                .find(|v| v.id == existing)
                            {
                                vp.printer_id = pid;
                                vp.apply_values(values);
                            }
                            st.virtual_printer_sel = Some(existing.clone());
                            st.save_printer_draft()?;
                            let label = st
                                .printer_draft
                                .virtual_printer_by_id(&existing)
                                .map(|v| v.name.clone())
                                .unwrap_or_default();
                            return Ok(
                                st.t("virtual_printer_updated_named").replace("{name}", &label)
                            );
                        }
                    }
                    let id = st.printer_draft.add_virtual_printer(&name, &pid, values)?;
                    st.virtual_printer_sel = Some(id);
                    st.save_printer_draft()?;
                    let label = st
                        .virtual_printer_sel
                        .as_ref()
                        .and_then(|id| st.printer_draft.virtual_printer_by_id(id))
                        .map(|v| v.name.clone())
                        .unwrap_or_default();
                    Ok(st.t("virtual_printer_created").replace("{name}", &label))
                })();
                match result {
                    Ok(msg) => {
                        self.ui
                            .text_input(cx, ids!(station_group_name_input))
                            .set_text(cx, "");
                        self.set_status(cx, msg);
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.paint_printers_panel(cx);
            }
        }
        if self.ui.button(cx, ids!(station_group_apply_btn)).clicked(actions) {
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            if is_client {
                self.set_status(cx, app_t("printers_host_only"));
            } else {
                let name = self.ui.text_input(cx, ids!(station_group_name_input)).text();
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    let id = {
                        let t = name.trim();
                        if let Some(sel) = st.virtual_printer_sel.clone() {
                            sel
                        } else if !t.is_empty() {
                            st.printer_draft
                                .virtual_printers
                                .iter()
                                .find(|v| v.name.eq_ignore_ascii_case(t) || v.id == t)
                                .map(|v| v.id.clone())
                                .ok_or_else(|| st.t("virtual_need_select"))?
                        } else {
                            return Err(st.t("virtual_need_select"));
                        }
                    };
                    let label = st
                        .printer_draft
                        .virtual_printer_by_id(&id)
                        .map(|v| v.name.clone())
                        .unwrap_or_default();
                    st.printer_draft.remove_virtual_printer(&id);
                    st.virtual_printer_sel = None;
                    st.save_printer_draft()?;
                    Ok(st.t("virtual_printer_deleted").replace("{name}", &label))
                })();
                match result {
                    Ok(msg) => {
                        self.ui
                            .text_input(cx, ids!(station_group_name_input))
                            .set_text(cx, "");
                        self.set_status(cx, msg);
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.paint_printers_panel(cx);
            }
        }
        if self.ui.button(cx, ids!(print_test_btn)).clicked(actions) {
            // Queue only — never wait for `lp` / PowerShell / TCP on the UI thread.
            // Host: every checked test printer. Client: Server prints the place.
            let is_client = app_state().lock().unwrap().device_role == DeviceRole::Client;
            let result = if is_client {
                (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    let route_key = st.printer_route_key.clone();
                    st.books_op(BooksOp::PrintTestPage { route_key })
                })()
            } else {
                let mut st = app_state().lock().unwrap();
                let pids = if st.test_printer_ids.is_empty() {
                    st.printer_selected_id.clone().into_iter().collect::<Vec<_>>()
                } else {
                    st.test_printer_ids.clone()
                };
                if pids.is_empty() {
                    let msg = st.t("test_page_pick_first");
                    drop(st);
                    self.set_status(cx, msg);
                    return;
                }
                if st.printers_base().is_some() {
                    if let Err(e) = st.save_printer_draft() {
                        drop(st);
                        self.set_status(cx, e);
                        return;
                    }
                } else {
                    st.reapply_kot_slip_format_onto_printer_draft();
                    print_share::configure(st.printer_draft.clone());
                }
                let own = test_page_own_words(&st);
                let dir = st.printers_base().or_else(|| st.data_base.clone());
                let when = st.fiscal.format_now();
                let draft = st.printer_draft.clone();
                let single_name = if pids.len() == 1 {
                    draft
                        .printer_by_id(&pids[0])
                        .map(|p| printer_display_name(p, |k| st.t(k)))
                } else {
                    None
                };
                drop(st);
                let mut ok = 0usize;
                let mut last_err: Option<String> = None;
                for pid in &pids {
                    let name = draft
                        .printer_by_id(pid)
                        .map(|p| {
                            let st = app_state().lock().unwrap();
                            printer_display_name(p, |k| st.t(k))
                        })
                        .unwrap_or_else(|| pid.clone());
                    let desc = draft
                        .printer_by_id(pid)
                        .map(|p| print_share::describe(&print_share::parse_target(&p.target)))
                        .unwrap_or_else(|| app_t("no_printer_assigned"));
                    let page = format!(
                        "{app} - test page\n{own}{name}\n{desc}\n{when}\n\nIf you can read this, printing works here.\n",
                        app = store::APP_LABEL,
                    );
                    match print_share::print_one_printer(
                        &draft,
                        pid,
                        print_share::PrintKind::Bill,
                        &page,
                        dir.as_deref(),
                    ) {
                        Ok(share) if share.opened => ok += 1,
                        Ok(_) => {
                            last_err = print_share::take_problems().into_iter().next();
                        }
                        Err(e) => last_err = Some(e),
                    }
                }
                if ok == 0 {
                    Err(last_err.unwrap_or_else(|| app_t("no_printer_assigned")))
                } else if let Some(name) = single_name.filter(|_| ok == 1) {
                    Ok(app_t("test_page_sent_one").replace("{name}", &name))
                } else {
                    Ok(app_t("test_page_sent_n").replace("{n}", &ok.to_string()))
                }
            };
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.choice_chip_clicked(cx, actions, ids!(kot_print_off), ids!(kot_print_on)) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let want_on = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    !store.kot_auto_print()
                };
                st.books_op(BooksOp::SetKotAutoPrint { on: want_on })?;
                if st.device_role == DeviceRole::Client {
                    if let Some(store) = st.store.as_ref() {
                        store.set_kot_auto_print_from_sync(want_on)?;
                    }
                }
                Ok(if want_on {
                    st.t("kot_print_on")
                } else {
                    st.t("kot_print_off")
                })
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.paint_printers_panel(cx);
        }

        if self.ui.button(cx, ids!(backup_cancel)).clicked(actions) {
            request_books_job_cancel();
            self.paint_submit_busy_chrome(cx);
        }
        if self.ui.button(cx, ids!(backup_btn)).clicked(actions) {
            if backup_busy().load(std::sync::atomic::Ordering::SeqCst) {
                // Creating… — ignore re-taps.
            } else {
                // VACUUM INTO can take seconds on a busy shop — never block the UI.
                // Owner or Backup screen only (Sales Settings alone is not enough).
                let job = (|| -> Result<(PathBuf, PathBuf), String> {
                    let st = app_state().lock().unwrap();
                    let actor = st
                        .session
                        .as_ref()
                        .ok_or_else(|| "Sign in required".to_string())?;
                    if !actor.is_owner() && !actor.can_access_backup() {
                        return Err("Not allowed to create a backup".into());
                    }
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let dest = default_backup_path()?;
                    Ok((store.db_path().to_path_buf(), dest))
                })();
                match job {
                    Ok((db, dest)) => {
                        self.set_status(cx, "Creating backup…");
                        backup_job(db, dest);
                        self.paint_submit_busy_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
        }

        if self.ui.button(cx, ids!(restore_cancel)).clicked(actions) {
            request_books_job_cancel();
            self.paint_submit_busy_chrome(cx);
        }
        if self.ui.button(cx, ids!(backup_copy_cancel)).clicked(actions) {
            request_books_job_cancel();
            self.paint_submit_busy_chrome(cx);
        }
        if self.ui.button(cx, ids!(csv_import_cancel)).clicked(actions) {
            request_books_job_cancel();
            self.paint_submit_busy_chrome(cx);
        }

        if self.ui.button(cx, ids!(restore_btn)).clicked(actions) {
            // Owner only (plus server host) — Backup Settings alone is not enough.
            let gate = {
                let st = app_state().lock().unwrap();
                if let Err(e) = st.require_server_host("Restore a backup") {
                    Err(e)
                } else if !st.session.as_ref().is_some_and(|a| a.is_owner()) {
                    Err(st.t("restore_owner_only"))
                } else {
                    Ok((
                        st.store.as_ref().map(|s| s.auto_backup_dir()),
                        st.t("restore_backup"),
                    ))
                }
            };
            match gate {
                Err(e) => self.set_status(cx, e),
                Ok((start, title)) => {
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

        // History retention (Settings → Backup).
        for (chip, chip_on, mode) in [
            (ids!(history_off), ids!(history_off_on), "off"),
            (ids!(history_day_close), ids!(history_day_close_on), "on_day_close"),
            (ids!(history_after_hours), ids!(history_after_hours_on), "after_hours"),
        ] {
            if self.choice_chip_clicked(cx, actions, chip, chip_on) {
                let hours_text = self.ui.text_input(cx, ids!(history_hours_input)).text();
                let hours = hours_text.trim().parse::<u32>().unwrap_or(4).max(1).min(168);
                let result = (|| -> Result<String, String> {
                    let mut st = app_state().lock().unwrap();
                    st.books_op(BooksOp::SetHistoryRetention {
                        mode: mode.into(),
                        hours,
                    })
                })();
                match result {
                    Ok(msg) => {
                        self.set_status(cx, msg);
                        self.paint_history_retention_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
        }
        if self.ui.button(cx, ids!(history_hours_save)).clicked(actions) {
            let hours_text = self.ui.text_input(cx, ids!(history_hours_input)).text();
            let hours = hours_text.trim().parse::<u32>().unwrap_or(4).max(1).min(168);
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.books_op(BooksOp::SetHistoryRetention {
                    mode: "after_hours".into(),
                    hours,
                })
            })();
            match result {
                Ok(_) => {
                    let msg = app_t("history_hours_saved");
                    self.set_status(cx, msg);
                    self.paint_history_retention_chrome(cx);
                }
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(history_clear_now)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.books_op(BooksOp::ClearTradingHistory)
            })();
            match result {
                Ok(msg) => {
                    let _ = app_state().lock().unwrap().reload_all();
                    self.set_status(cx, msg);
                    self.sync_chrome(cx);
                }
                Err(e) => self.set_status(cx, e),
            }
        }

        if self.ui.button(cx, ids!(wipe_trading)).clicked(actions)
            || self.ui.button(cx, ids!(wipe_trading_on)).clicked(actions)
        {
            let mut st = app_state().lock().unwrap();
            st.wipe_sel_trading = !st.wipe_sel_trading;
            drop(st);
            self.paint_wipe_data_chrome(cx);
        }
        if self.ui.button(cx, ids!(wipe_schema)).clicked(actions)
            || self.ui.button(cx, ids!(wipe_schema_on)).clicked(actions)
        {
            let mut st = app_state().lock().unwrap();
            st.wipe_sel_schema = !st.wipe_sel_schema;
            drop(st);
            self.paint_wipe_data_chrome(cx);
        }
        if self.ui.button(cx, ids!(wipe_ui)).clicked(actions)
            || self.ui.button(cx, ids!(wipe_ui_on)).clicked(actions)
        {
            let mut st = app_state().lock().unwrap();
            st.wipe_sel_ui = !st.wipe_sel_ui;
            drop(st);
            self.paint_wipe_data_chrome(cx);
        }
        if self.ui.button(cx, ids!(wipe_data_btn)).clicked(actions) {
            match self.apply_owner_local_wipe(cx) {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
        }
    }
}

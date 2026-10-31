impl App {
    fn handle_actions_nav_books_lock(&mut self, cx: &mut Cx, actions: &Actions) {
        // "Type or pick" fields: focus opens the list, typing narrows it, a
        // tap fills the field, Escape or focusing another field closes it.
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
                    let party_country = if matches!(
                        field,
                        SuggestField::PartyCountry | SuggestField::PartyState
                    ) {
                        if field == SuggestField::PartyCountry {
                            text.clone()
                        } else {
                            self.ui.text_input(cx, ids!(party_country_input)).text()
                        }
                    } else {
                        String::new()
                    };
                    {
                        let mut st = app_state().lock().unwrap();
                        if !party_country.is_empty()
                            || matches!(
                                field,
                                SuggestField::PartyCountry | SuggestField::PartyState
                            )
                        {
                            st.party_form_country = party_country;
                        }
                        st.open_suggest(field, &text);
                    }
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
                let fill_origin = {
                    let mut st = app_state().lock().unwrap();
                    st.suggest_open = None;
                    if field == SuggestField::Country {
                        st.country_browse = value.clone();
                    }
                    if field == SuggestField::PartyCountry {
                        st.party_form_country = value.clone();
                    }
                    let party = if matches!(
                        field,
                        SuggestField::BookingOrigin | SuggestField::CustomerName
                    ) {
                        st.customers
                            .iter()
                            .find(|c| c.name.eq_ignore_ascii_case(value.trim()))
                            .cloned()
                    } else {
                        None
                    };
                    if let Some(ref p) = party {
                        st.selected_customer_id = Some(p.id.clone());
                    }
                    (field == SuggestField::BookingOrigin).then_some(party).flatten()
                };
                if let Some(party) = fill_origin {
                    self.fill_booking_consignor_from_party(cx, &party);
                }
                if field == SuggestField::PartyCountry {
                    let dial_country = country_code_from_label(&value);
                    let code = if dial_country.is_empty() {
                        app_state().lock().unwrap().fiscal.country_code.clone()
                    } else {
                        dial_country
                    };
                    let ph = phone_placeholder(
                        &app_state().lock().unwrap().t("phone_ph"),
                        &code,
                    );
                    self.ui
                        .text_input(cx, ids!(credit_add_phone))
                        .set_empty_text(cx, ph);
                }
                changed = true;
            }
            if changed {
                self.sync_chrome(cx);
                for field in SuggestField::ALL {
                    self.ui.widget(cx, field.list()).redraw(cx);
                }
            }
        }
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
                            Ok(st
                                .t("backup_copy_to")
                                .replace("{dir}", &dir.display().to_string()))
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
                        if *id == live_id!(party_photo_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_party_photo_picked(cx, path.clone());
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
                        if *id == live_id!(print_stamp_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    let outcome = (|| -> Result<String, String> {
                                        let st = app_state().lock().unwrap();
                                        let store = st
                                            .store
                                            .as_ref()
                                            .ok_or_else(|| "Storage not ready".to_string())?;
                                        store.import_receipt_stamp(&file)?;
                                        Ok("Receipt stamp saved".into())
                                    })();
                                    match outcome {
                                        Ok(m) => self.set_status(cx, m),
                                        Err(e) => self.set_status(cx, e),
                                    }
                                    self.sync_chrome(cx);
                                }
                                Err(e) => self.set_status(cx, e),
                            }
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(print_qr_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            match self.materialize_picked_file(cx, path) {
                                Ok(file) => {
                                    let outcome = (|| -> Result<String, String> {
                                        let st = app_state().lock().unwrap();
                                        let store = st
                                            .store
                                            .as_ref()
                                            .ok_or_else(|| "Storage not ready".to_string())?;
                                        store.import_payment_qr(&file)?;
                                        Ok("Payment QR saved".into())
                                    })();
                                    match outcome {
                                        Ok(m) => self.set_status(cx, m),
                                        Err(e) => self.set_status(cx, e),
                                    }
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
                        if *id == live_id!(entry_attach_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            self.on_entry_attach_picked(cx, path.clone());
                        }
                    }
                    FileDialogAction::FileSelected { id, paths }
                        if *id == live_id!(csv_parties_pick)
                            || *id == live_id!(csv_items_pick)
                            || *id == live_id!(csv_pincodes_pick)
                            || *id == live_id!(csv_carrier_bill_pick) =>
                    {
                        if let Some(path) = paths.first() {
                            let kind = if *id == live_id!(csv_parties_pick) {
                                "parties"
                            } else if *id == live_id!(csv_items_pick) {
                                "items"
                            } else if *id == live_id!(csv_pincodes_pick) {
                                "pincodes"
                            } else {
                                "carrier_bill"
                            };
                            let outcome = (|| -> Result<String, String> {
                                let text = std::fs::read_to_string(path)
                                    .map_err(|e| format!("Could not read the file: {e}"))?;
                                let mut st = app_state().lock().unwrap();
                                st.require_server_role("Import from CSV")?;
                                let store =
                                    st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                                match kind {
                                    "carrier_bill" => {
                                        let report = store.reconcile_carrier_bill_csv(&text)?;
                                        let diffs = report
                                            .lines()
                                            .skip(1)
                                            .filter(|l| {
                                                l.contains("charge_diff")
                                                    || l.contains("weight_diff")
                                                    || l.contains("missing")
                                            })
                                            .count();
                                        let (applied, skipped) =
                                            store.apply_carrier_bill_costs(&text)?;
                                        st.last_carrier_bill_report = report;
                                        Ok(format!(
                                            "Carrier bill: {diffs} flagged · applied cost on {applied} · skipped {skipped}. Copy report from Reports."
                                        ))
                                    }
                                    other => {
                                        let (added, skipped, problems) = match other {
                                            "parties" => store.import_parties_csv(&text)?,
                                            "pincodes" => store.import_pincodes_csv(&text)?,
                                            _ => store.import_items_csv(&text)?,
                                        };
                                        st.reload_all()?;
                                        let mut msg = format!(
                                            "Imported {added} new, skipped {skipped} (already there or unusable)"
                                        );
                                        if let Some(p) = problems.first() {
                                            msg.push_str(&format!(" · first problem: {p}"));
                                        }
                                        Ok(msg)
                                    }
                                }
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
                            || *id == live_id!(party_photo_pick)
                            || *id == live_id!(venue_icon_pick)
                            || *id == live_id!(onboard_icon_pick)
                            || *id == live_id!(csv_parties_pick)
                            || *id == live_id!(csv_items_pick)
                            || *id == live_id!(csv_pincodes_pick)
                            || *id == live_id!(csv_carrier_bill_pick)
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
        // Nav clicks honour the user's slot arrangement; the legacy More
        // rows (hidden, kept for action wiring) still map to their tabs.
        let mut nav_click = self.nav_tab_clicked(cx, actions);
        if self.ui.button(cx, ids!(more_items)).clicked(actions) {
            nav_click = Some(NavTab::Rates);
        }
        if self.ui.button(cx, ids!(more_sales)).clicked(actions) {
            nav_click = Some(NavTab::Sales);
        }
        if self.ui.button(cx, ids!(more_day_close)).clicked(actions)
            || self
                .ui
                .button(cx, ids!(day_close_from_sales))
                .clicked(actions)
        {
            nav_click = Some(NavTab::DayClose);
        }
        if self.ui.button(cx, ids!(more_reports)).clicked(actions) {
            nav_click = Some(NavTab::Reports);
        }
        if self.ui.button(cx, ids!(more_ops)).clicked(actions) {
            nav_click = Some(NavTab::Ops);
        }
        if self.ui.button(cx, ids!(more_cod)).clicked(actions) {
            nav_click = Some(NavTab::Cod);
        }
        if self.ui.button(cx, ids!(more_riders)).clicked(actions) {
            nav_click = Some(NavTab::Riders);
        }
        if self.ui.button(cx, ids!(more_inventory)).clicked(actions) {
            nav_click = Some(NavTab::Inventory);
        }
        if self.ui.button(cx, ids!(more_branches)).clicked(actions) {
            nav_click = Some(NavTab::Branches);
        }
        if self.ui.button(cx, ids!(more_credit)).clicked(actions) {
            nav_click = Some(NavTab::Credit);
        }
        if self.ui.button(cx, ids!(more_team)).clicked(actions) {
            nav_click = Some(NavTab::Team);
        }
        if self.ui.button(cx, ids!(more_settings)).clicked(actions) {
            nav_click = Some(NavTab::Settings);
        }
        if self.ui.button(cx, ids!(more_lock)).clicked(actions) {
            nav_click = Some(NavTab::Lock);
        }
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
            Some(NavTab::Rates) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Menu, |s| s.can_access_items());
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
                self.go_peer_if_allowed(cx, Screen::Settings, |s| s.can_access_settings());
            }
            Some(NavTab::DayClose) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.refresh_day_close_text()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::DayClose);
                }
            }
            Some(NavTab::Ops)
            | Some(NavTab::Cod)
            | Some(NavTab::Riders)
            | Some(NavTab::Reports) => {
                let focus = match nav_click {
                    Some(NavTab::Ops) => ReportsNav::Ops,
                    Some(NavTab::Cod) => ReportsNav::Cod,
                    Some(NavTab::Riders) => ReportsNav::Riders,
                    _ => ReportsNav::Bank,
                };
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_sales() {
                        return Err("Not allowed".into());
                    }
                    st.reports_nav = focus;
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
        if self.ui.button(cx, ids!(sidebar_settings_open)).clicked(actions) {
            self.go_if_allowed(cx, Screen::SidebarSettings, |s| s.can_access_settings());
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
            (ids!(books_sec_bookings_hdr), "bookings"),
            (ids!(books_sec_payments_hdr), "payments"),
            (ids!(books_sec_outstanding_hdr), "outstanding"),
            (ids!(books_sec_bank_hdr), "bank"),
        ] {
            if self.ui.button(cx, hdr).clicked(actions) {
                self.toggle_books_section(cx, key);
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
                st.reports_nav = ReportsNav::Bank;
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
        if self.ui.button(cx, ids!(buy_to_inventory)).clicked(actions) {
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
        if self.choice_chip_clicked(cx, actions, ids!(buy_paid), ids!(buy_paid_on)) {
            app_state().lock().unwrap().buy_on_account = false;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(buy_on_account), ids!(buy_on_account_on)) {
            app_state().lock().unwrap().buy_on_account = true;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(buy_kind_supplier), ids!(buy_kind_supplier_on))
        {
            app_state().lock().unwrap().buy_party_kind = "Supplier".into();
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(buy_kind_carrier), ids!(buy_kind_carrier_on)) {
            app_state().lock().unwrap().buy_party_kind = "Carrier".into();
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(booking_insurance_no),
            ids!(booking_insurance_no_on),
        ) {
            app_state().lock().unwrap().booking_insurance = false;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(booking_insurance_yes),
            ids!(booking_insurance_yes_on),
        ) {
            app_state().lock().unwrap().booking_insurance = true;
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(route_pick_receipt),
            ids!(route_pick_receipt_on),
        ) {
            app_state().lock().unwrap().printer_route_key = "receipt".into();
            self.sync_chrome(cx);
        }
        if self.choice_chip_clicked(cx, actions, ids!(route_pick_label), ids!(route_pick_label_on))
        {
            app_state().lock().unwrap().printer_route_key = "label".into();
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(buy_purchase)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(buy_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(buy_qty_input)).text();
            let cost_text = self.ui.text_input(cx, ids!(buy_cost_input)).text();
            let supplier = self.ui.text_input(cx, ids!(buy_supplier_input)).text();
            let party_kind = app_state().lock().unwrap().buy_party_kind.clone();
            let note = self.ui.text_input(cx, ids!(buy_note_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Record purchases")?;
                let actor = st.actor()?;
                let on_account = st.buy_on_account;
                let qty_milli = parse_qty_to_milli(&qty_text)?;
                let product = st.find_product_by_name(&name)?;
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
                        let party = store.add_customer_kind(supplier.trim(), "", kind)?;
                        party_id = party.id;
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
                    self.ui
                        .text_input(cx, ids!(buy_cost_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(buy_expense_add)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(buy_expense_amount)).text();
            let note = self.ui.text_input(cx, ids!(buy_expense_note)).text();
            let part2_text = self.ui.text_input(cx, ids!(buy_exp_part2)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.device_role == DeviceRole::Client {
                    return Err("Log expenses on the shop device — this is an extra till".into());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                let parts = Self::expense_parts(
                    minor,
                    &part2_text,
                    &currency,
                    st.expense_tender,
                    st.expense_tender2,
                )?;
                let expense_id = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_expense_tenders(minor, "misc", &note, &actor, &parts)?
                };
                st.selected_expense_id = Some(expense_id);
                let _ = st.refresh_books_overview();
                st.status = "Expense logged".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(buy_expense_amount))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(buy_expense_note))
                    .set_text(cx, "");
                self.ui.text_input(cx, ids!(buy_exp_part2)).set_text(cx, "");
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
        if self
            .ui
            .button(cx, ids!(server_pick_cancel))
            .clicked(actions)
        {
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
                    Ok(format!(
                        "Forgot {} — it must pair again to work",
                        device.name
                    ))
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
        if self
            .ui
            .button(cx, ids!(settings_open_branches))
            .clicked(actions)
        {
            // Branches is a chrome peer root — not a Settings drill-down.
            self.go_peer_if_allowed(cx, Screen::Branches, |s| {
                s.can_access_branches() || s.is_owner()
            });
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
        if self.choice_chip_clicked(cx, actions, ids!(role_owner), ids!(role_owner_on)) {
            {
                let mut st = app_state().lock().unwrap();
                st.lock_as_owner = true;
                st.pin_buffer.clear();
                st.status = "Unlock as Owner".into();
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
            let typed_customer = self.ui.text_input(cx, ids!(customer_name_input)).text();
            self.sync_booking_desk_from_ui(cx);
            enum Pending {
                Local { sale_id: String, snack: String },
                // Saved on this device, on its way to the office.
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
                // Extras (fuel/docket) as cart lines before draft.
                let currency = st.locale.currency.clone();
                if let Some(p) =
                    booking::parse_extra_amount(&st.booking_fuel, &currency).filter(|p| *p > 0)
                {
                    let extra = booking::booking_extra(rates::ExtraChargeKind::Fuel, p);
                    if !st
                        .cart
                        .lines
                        .iter()
                        .any(|l| l.product_id.starts_with("extra:fuel"))
                    {
                        st.cart.lines.push(booking::extra_as_line(&extra));
                    }
                }
                if let Some(p) =
                    booking::parse_extra_amount(&st.booking_docket, &currency).filter(|p| *p > 0)
                {
                    let extra = booking::booking_extra(rates::ExtraChargeKind::Docket, p);
                    if !st
                        .cart
                        .lines
                        .iter()
                        .any(|l| l.product_id.starts_with("extra:docket"))
                    {
                        st.cart.lines.push(booking::extra_as_line(&extra));
                    }
                }
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
                let currency = st.locale.currency.clone();
                if st.cart.charge_total.is_some()
                    && let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                    && minor > 0
                {
                    let _ = st.cart.set_charge_total(Money(minor));
                }
                let mut draft = st.cart.to_sale_draft()?;
                draft.money_lane = st.money_lane;
                st.apply_booking_fields(&mut draft);
                let charged = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.preview_sale_total(&draft)?
                };
                let cash = parse_part(&cash_text, &currency)?;
                let upi = parse_part(&upi_text, &currency)?;
                let card = parse_part(&card_text, &currency)?;
                let credit = parse_part(&credit_text, &currency)?;
                if cash < 0 || upi < 0 || card < 0 || credit < 0 {
                    return Err("Payment amounts cannot be negative".into());
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
                let customer_id = if credit > 0 {
                    if let Some(id) = st.selected_customer_id.clone() {
                        Some(id)
                    } else if typed_customer.trim().is_empty() {
                        return Err("Credit part needs a customer — type a name or Pick one".into());
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
                let customer_name = if credit > 0 {
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
                } else {
                    None
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
                    let amount = st.money(Money(draft.total_paise));
                    let req = net::RemoteSaleRequest {
                        draft,
                        pay: pay_label,
                        customer_id,
                        customer_name,
                        payments: parts
                            .iter()
                            .map(|(method, amount)| net::RemoteTender {
                                pay: method.as_str().into(),
                                amount_paise: *amount,
                            })
                            .collect(),
                    };
                    // Saved on this device first, then sent: a Wi‑Fi drop
                    // delays the booking (and its AWB), it never loses it.
                    st.lan_job(
                        "/sale",
                        &req,
                        format!("Booking {amount}"),
                        format!("{amount} · {summary}"),
                        phone::Follow::Sale,
                        true,
                    )?;
                    st.cart.clear();
                    st.status = format!(
                        "{amount} · {summary} saved — the AWB comes when the office is reachable"
                    );
                    return Ok(Pending::Queued);
                }
                let sale = {
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    store.finalize_sale_tenders(&draft, &parts, customer_id.as_deref(), &actor)?
                };
                let amount = st.money(Money(sale.total_paise));
                let full = st
                    .store
                    .as_ref()
                    .and_then(|s| s.format_receipt_text(&sale.id).ok())
                    .unwrap_or_else(|| format!("Sale {}", amount));
                st.cart.clear();
                st.last_awb = sale.awb_no.clone();
                if let Some(store) = st.store.as_ref()
                    && !sale.awb_no.is_empty()
                {
                    let _ = store.write_awb_label(&sale.id);
                }
                st.last_receipt = full;
                st.status = if sale.awb_no.is_empty() {
                    format!("Booked · {summary}")
                } else {
                    format!("Booked · AWB {} · {summary}", sale.awb_no)
                };
                let snack = if sale.awb_no.is_empty() {
                    format!("{amount} · {summary}")
                } else {
                    format!("{amount} · AWB {} · {summary}", sale.awb_no)
                };
                st.reload_all()?;
                Ok(Pending::Local {
                    sale_id: sale.id,
                    snack,
                })
            })();
            let result = match pending {
                Err(e) => Err(e),
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
    }
}

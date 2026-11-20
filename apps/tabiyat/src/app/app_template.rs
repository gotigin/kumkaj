impl App {
  pub(crate) fn should_offer_app_template(st: &AppState) -> bool {
        if matches!(
            std::env::var("KUMKAJ_SMOKE_AUTO_SERVER").ok().as_deref(),
            Some("1" | "true" | "yes" | "on")
        ) || std::env::var("KUMKAJ_SMOKE_AUTO_UNLOCK").is_ok()
        {
            return false;
        }
        !st.owner_setup_complete
            && !st
                .store
                .as_ref()
                .and_then(|s| s.app_template_offer_done().ok())
                .unwrap_or(false)
    }

    fn enter_app_template_offer(&mut self, st: &mut AppState, switching_in: bool) {
        st.device_role = DeviceRole::Server;
        st.must_confirm_device_role = false;
        st.picking_server = false;
        st.awaiting_app_template = true;
        st.app_template_selected_id = menu_templates::all().first().map(|t| t.id.clone());
        st.device_setup_sections_open = vec!["role".into()];
        st.pin_buffer.clear();
        st.nav_back.clear();
        st.nav_forward.clear();
        if let Some(store) = st.store.as_ref() {
            let _ = store.set_device_role(DeviceRole::Server.as_str());
            if switching_in {
                if let Ok(Some(owner)) = store
                    .list_staff()
                    .map(|s| s.into_iter().find(|m| m.is_owner()))
                {
                    let _ = store.set_extra_tills_allowed(false, &owner);
                }
                st.extra_tills_allowed = false;
                net::stop_server();
            } else if st.extra_tills_allowed {
                let path = store.db_path().to_path_buf();
                if net::ensure_server(path).is_ok() {
                    firewall_job(|| Ok(net::firewall_allows_phones()));
                }
            }
        }
        st.screen = Screen::DeviceSetup;
        st.status = locale::t(&st.locale.language, "app_template_status");
    }

    fn finish_app_template_to_lock(&mut self, cx: &mut Cx, mark_done: bool) {
        {
            let mut st = app_state().lock().unwrap();
            st.awaiting_app_template = false;
            st.app_template_selected_id = None;
            let (staff, fiscal) = if let Some(store) = st.store.as_ref() {
                if mark_done {
                    let _ = store.set_app_template_offer_done();
                }
                (store.list_staff().ok(), store.fiscal_settings().ok())
            } else {
                (None, None)
            };
            if let Some(staff) = staff {
                st.staff = staff;
            }
            if let Some(fiscal) = fiscal {
                st.fiscal = fiscal;
            }
            self.finish_as_server(&mut st, false);
        }
        self.paint_role_open_busy_chrome(cx);
        self.sync_chrome(cx);
    }

    pub(crate) fn paint_app_template_chrome(&mut self, cx: &mut Cx, visible: bool) {
        self.ui
            .view(cx, ids!(app_template_section))
            .set_visible(cx, visible);
        if !visible {
            return;
        }
        let app_busy = matches!(
            menu_import_running().lock().unwrap().as_ref(),
            Some(MenuImportKind::AppTemplate { .. })
        );
        let bundled = menu_templates::all();
        let (
            heading,
            skip,
            import,
            help,
            public_help,
            sel,
            progress,
            cancel,
            from_dev,
            from_help,
        ) = {
            let st = app_state().lock().unwrap();
            let progress = if app_busy {
                let (done, total) = *menu_import_progress().lock().unwrap();
                if total > 0 {
                    st.t("template_import_progress")
                        .replace("{d}", &done.to_string())
                        .replace("{t}", &total.to_string())
                } else {
                    st.t("app_template_importing").replace("{title}", "…")
                }
            } else {
                String::new()
            };
            (
                st.t("app_template_heading"),
                st.t("app_template_skip"),
                st.t("app_template_import_selected"),
                st.t("app_template_blurb"),
                st.t("settings_app_tmpl_note"),
                st.app_template_selected_id.clone(),
                progress,
                st.t("cancel"),
                st.t("app_template_from_device"),
                st.t("app_template_from_device_help"),
            )
        };
        self.ui
            .widget(cx, ids!(app_template_heading))
            .set_text(cx, &plain_display_text(&heading));
        self.ui
            .widget(cx, ids!(app_template_blurb))
            .set_visible(cx, false);
        for (id, tip) in [
            (ids!(app_template_help), help.as_str()),
            (ids!(setup_public_tmpl_help), public_help.as_str()),
        ] {
            self.ui
                .widget(cx, id)
                .as_info_help()
                .set_tip(&plain_display_text(tip));
        }
        let chips = [
            (ids!(public_tmpl_0), ids!(public_tmpl_0_on), 0usize),
            (ids!(public_tmpl_1), ids!(public_tmpl_1_on), 1),
            (ids!(public_tmpl_2), ids!(public_tmpl_2_on), 2),
        ];
        for (idle, active, idx) in chips {
            if let Some(tmpl) = bundled.get(idx) {
                let on = sel.as_deref() == Some(tmpl.id.as_str());
                let label = tmpl.title.clone();
                self.ui.button(cx, idle).set_text(cx, &label);
                self.ui.button(cx, active).set_text(cx, &label);
                self.ui.button(cx, idle).set_visible(cx, !on && !app_busy);
                self.ui.button(cx, active).set_visible(cx, on && !app_busy);
            } else {
                self.ui.button(cx, idle).set_visible(cx, false);
                self.ui.button(cx, active).set_visible(cx, false);
            }
        }
        let show_import = sel.is_some() && !app_busy;
        self.ui
            .button(cx, ids!(public_tmpl_import))
            .set_text(cx, &import);
        self.ui
            .button(cx, ids!(public_tmpl_import))
            .set_visible(cx, show_import);
        self.ui
            .button(cx, ids!(app_template_from_device))
            .set_text(cx, &from_dev);
        self.ui
            .button(cx, ids!(app_template_from_device))
            .set_visible(cx, !app_busy);
        self.ui
            .widget(cx, ids!(app_template_from_device_help))
            .as_info_help()
            .set_tip(&plain_display_text(&from_help));
        self.ui
            .view(cx, ids!(app_template_busy))
            .set_visible(cx, app_busy);
        if app_busy {
            self.ui
                .widget(cx, ids!(app_template_status))
                .set_text(cx, &plain_display_text(&progress));
            self.ui
                .button(cx, ids!(app_template_cancel))
                .set_text(cx, &cancel);
            self.ui
                .button(cx, ids!(app_template_cancel))
                .set_visible(cx, true);
        }
        self.ui
            .button(cx, ids!(app_template_skip))
            .set_text(cx, &skip);
        self.ui
            .button(cx, ids!(app_template_skip))
            .set_visible(cx, !app_busy);
        self.sync_device_setup_sections(cx);
    }

    pub(crate) fn start_app_template_import(
        &mut self,
        cx: &mut Cx,
        pack_dir: std::path::PathBuf,
        title: String,
        wipe_trading_first: bool,
        settings_import: bool,
    ) {
        let db = {
            let st = app_state().lock().unwrap();
            st.store
                .as_ref()
                .map(|s| s.db_path().to_path_buf())
        };
        let Some(db) = db else {
            self.set_status(cx, "No store");
            return;
        };
        if settings_import {
            app_state().lock().unwrap().app_template_settings_import = true;
        }
        menu_import_job(
            db,
            MenuImportKind::AppTemplate {
                title,
                pack_dir,
                apply_shop_defaults: true,
                wipe_trading_first,
            },
        );
        self.paint_import_progress_chrome(cx);
        if settings_import {
            self.paint_settings_app_templates(cx);
        } else {
            self.paint_app_template_chrome(cx, true);
        }
    }

    pub(crate) fn open_app_template_folder_pick(&mut self, cx: &mut Cx, for_settings: bool) {
        if for_settings {
            let ok = (|| -> Result<(), String> {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.is_owner() {
                    return Err(st.t("wipe_owner_only"));
                }
                st.require_server_host(&st.t("app_template_from_device"))?;
                Ok(())
            })();
            if let Err(e) = ok {
                self.set_status(cx, e);
                return;
            }
        }
        {
            let mut st = app_state().lock().unwrap();
            st.folder_pick_for_app_template = true;
        }
        let title = app_t("app_template_from_device");
        cx.open_select_folder_dialog(FileDialog::new().set_title(title));
    }

    pub(crate) fn on_app_template_folder_selected(&mut self, cx: &mut Cx, dir: &std::path::Path) {
        let pending = (|| -> Result<Option<(std::path::PathBuf, String, bool)>, String> {
            let st = app_state().lock().unwrap();
            let tmpl = menu_templates::read_from_dir(dir)?;
            let name = template_client_name(&tmpl.title);
            let title = st.t("app_tmpl_named").replace("{name}", &name);
            if st.awaiting_app_template {
                return Ok(Some((dir.to_path_buf(), title, false)));
            }
            if st.actor()?.is_owner() {
                st.require_server_host(&st.t("app_template_from_device"))?;
            } else {
                return Err(st.t("wipe_owner_only"));
            }
            drop(st);
            let mut st = app_state().lock().unwrap();
            st.app_template_wipe_pending = Some(AppTemplateImportPending::PackDir {
                dir: dir.to_path_buf(),
                title,
            });
            Ok(None)
        })();
        match pending {
            Ok(Some((pack_dir, title, wipe))) => {
                self.start_app_template_import(cx, pack_dir, title, wipe, false);
            }
            Ok(None) => {
                self.paint_settings_app_templates(cx);
                self.sync_chrome(cx);
            }
            Err(e) => self.set_status(cx, e),
        }
    }

    pub(crate) fn paint_settings_app_templates(&mut self, cx: &mut Cx) -> String {
        let import_busy = matches!(
            menu_import_running().lock().unwrap().as_ref(),
            Some(MenuImportKind::AppTemplate { .. })
        ) && app_state().lock().unwrap().app_template_settings_import;
        let busy = import_busy;
        let bundled = menu_templates::all();
        let (
            heading,
            note,
            wipe_armed,
            progress,
            cancel,
            cont,
            abort,
            warn,
            import_l,
            from_dev,
            sel_id,
        ) = {
            let st = app_state().lock().unwrap();
            let progress = if busy {
                let (done, total) = *menu_import_progress().lock().unwrap();
                if total > 0 {
                    st.t("template_import_progress")
                        .replace("{d}", &done.to_string())
                        .replace("{t}", &total.to_string())
                } else {
                    st.t("app_template_importing").replace("{title}", "…")
                }
            } else {
                String::new()
            };
            (
                st.t("app_template_heading"),
                st.t("settings_app_tmpl_note"),
                st.app_template_wipe_pending.is_some(),
                progress,
                st.t("cancel"),
                st.t("app_template_wipe_continue"),
                st.t("app_template_wipe_cancel"),
                st.t("app_template_wipe_warn"),
                st.t("app_template_import_selected"),
                st.t("app_template_from_device"),
                st.settings_app_template_selected_id.clone(),
            )
        };
        self.ui
            .widget(cx, ids!(settings_app_tmpl_heading))
            .set_text(cx, &plain_display_text(&heading));
        self.ui
            .widget(cx, ids!(settings_app_tmpl_help))
            .as_info_help()
            .set_tip(&plain_display_text(&note));
        self.ui
            .widget(cx, ids!(settings_app_tmpl_note))
            .set_text(cx, &plain_display_text(&note));
        let chips = [
            (ids!(settings_public_tmpl_0), ids!(settings_public_tmpl_0_on), 0usize),
            (ids!(settings_public_tmpl_1), ids!(settings_public_tmpl_1_on), 1),
            (ids!(settings_public_tmpl_2), ids!(settings_public_tmpl_2_on), 2),
        ];
        for (idle, active, idx) in chips {
            if let Some(tmpl) = bundled.get(idx) {
                let on = sel_id.as_deref() == Some(tmpl.id.as_str());
                let label = template_client_name(&tmpl.title);
                self.ui.button(cx, idle).set_text(cx, &label);
                self.ui.button(cx, active).set_text(cx, &label);
                let show = !busy && !wipe_armed;
                self.ui.button(cx, idle).set_visible(cx, show && !on);
                self.ui.button(cx, active).set_visible(cx, show && on);
            } else {
                self.ui.button(cx, idle).set_visible(cx, false);
                self.ui.button(cx, active).set_visible(cx, false);
            }
        }
        let idle = !busy && !wipe_armed;
        self.ui
            .button(cx, ids!(settings_public_tmpl_import))
            .set_text(cx, &plain_display_text(&import_l));
        self.ui
            .button(cx, ids!(settings_public_tmpl_import))
            .set_visible(cx, idle && sel_id.is_some());
        self.ui
            .button(cx, ids!(settings_app_template_from_device))
            .set_text(cx, &plain_display_text(&from_dev));
        self.ui
            .button(cx, ids!(settings_app_template_from_device))
            .set_visible(cx, idle);
        self.ui
            .widget(cx, ids!(settings_app_tmpl_wipe_warn))
            .set_visible(cx, wipe_armed && !busy);
        if wipe_armed {
            self.ui
                .widget(cx, ids!(settings_app_tmpl_wipe_warn))
                .set_text(cx, &plain_display_text(&warn));
        }
        self.ui
            .button(cx, ids!(settings_app_tmpl_wipe_continue))
            .set_text(cx, &plain_display_text(&cont));
        self.ui
            .button(cx, ids!(settings_app_tmpl_wipe_cancel))
            .set_text(cx, &plain_display_text(&abort));
        self.ui
            .button(cx, ids!(settings_app_tmpl_wipe_continue))
            .set_visible(cx, wipe_armed && !busy);
        self.ui
            .button(cx, ids!(settings_app_tmpl_wipe_cancel))
            .set_visible(cx, wipe_armed && !busy);
        self.ui
            .view(cx, ids!(settings_app_tmpl_busy))
            .set_visible(cx, busy);
        if busy {
            self.ui
                .widget(cx, ids!(settings_app_tmpl_progress))
                .set_text(cx, &plain_display_text(&progress));
            self.ui
                .button(cx, ids!(settings_app_tmpl_cancel))
                .set_text(cx, &plain_display_text(&cancel));
        }
        progress
    }

    fn arm_settings_app_template_bundled(&mut self, _cx: &mut Cx) -> Result<(), String> {
        let mut st = app_state().lock().unwrap();
        let actor = st.actor()?;
        if !actor.is_owner() {
            return Err(st.t("wipe_owner_only"));
        }
        st.require_server_host(&st.t("app_template_import_selected"))?;
        let id = st
            .settings_app_template_selected_id
            .clone()
            .ok_or_else(|| "Pick a template".to_string())?;
        let title = menu_templates::find(&id)
            .map(|t| {
                st.t("app_tmpl_named")
                    .replace("{name}", &template_client_name(&t.title))
            })
            .unwrap_or_else(|| id.clone());
        st.app_template_wipe_pending = Some(AppTemplateImportPending::Bundled { id, title });
        Ok(())
    }

    pub(crate) fn run_settings_app_template_continue(&mut self, cx: &mut Cx) {
        let pending = app_state().lock().unwrap().app_template_wipe_pending.clone();
        let Some(pending) = pending else {
            return;
        };
        {
            let mut st = app_state().lock().unwrap();
            st.app_template_wipe_pending = None;
        }
        match pending {
            AppTemplateImportPending::Bundled { id, title } => {
                let pack = menu_templates::bundled_pack_dir(&id);
                if let Some(dir) = pack {
                    self.start_app_template_import(cx, dir, title, true, true);
                } else {
                    app_state().lock().unwrap().app_template_settings_import = false;
                    self.set_status(cx, "Template folder missing");
                    self.paint_settings_app_templates(cx);
                }
            }
            AppTemplateImportPending::PackDir { dir, title } => {
                self.start_app_template_import(cx, dir, title, true, true);
            }
        }
    }

    pub(crate) fn handle_app_template_setup_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let bundled = menu_templates::all();
        let chips = [
            (ids!(public_tmpl_0), ids!(public_tmpl_0_on), 0usize),
            (ids!(public_tmpl_1), ids!(public_tmpl_1_on), 1),
            (ids!(public_tmpl_2), ids!(public_tmpl_2_on), 2),
        ];
        for (idle, active, idx) in chips {
            if self.ui.button(cx, idle).clicked(actions)
                || self.ui.button(cx, active).clicked(actions)
            {
                if let Some(tmpl) = bundled.get(idx) {
                    let mut st = app_state().lock().unwrap();
                    if st.awaiting_app_template {
                        st.app_template_selected_id =
                            if st.app_template_selected_id.as_deref() == Some(tmpl.id.as_str()) {
                                None
                            } else {
                                Some(tmpl.id.clone())
                            };
                    }
                }
                self.paint_app_template_chrome(cx, true);
            }
        }
        if self.ui.button(cx, ids!(public_tmpl_import)).clicked(actions)
            && !menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
        {
            let job = (|| -> Result<(std::path::PathBuf, String), String> {
                let st = app_state().lock().unwrap();
                if !st.awaiting_app_template {
                    return Err("App template is not offered now".into());
                }
                let id = st
                    .app_template_selected_id
                    .clone()
                    .ok_or_else(|| "Pick a template".to_string())?;
                let dir = menu_templates::bundled_pack_dir(&id)
                    .ok_or_else(|| "Template folder missing".to_string())?;
                let title = menu_templates::find(&id)
                    .map(|t| {
                        st.t("app_tmpl_named")
                            .replace("{name}", &template_client_name(&t.title))
                    })
                    .unwrap_or(id);
                Ok((dir, title))
            })();
            match job {
                Ok((dir, title)) => self.start_app_template_import(cx, dir, title, false, false),
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(app_template_cancel)).clicked(actions)
            && menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
        {
            request_books_job_cancel();
            self.paint_import_progress_chrome(cx);
            self.paint_app_template_chrome(cx, true);
        }
        if self.ui.button(cx, ids!(app_template_skip)).clicked(actions)
            && !menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
            && app_state().lock().unwrap().awaiting_app_template
        {
            self.finish_app_template_to_lock(cx, false);
        }
        if self.ui.button(cx, ids!(app_template_from_device)).clicked(actions)
            && !menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
        {
            if cfg!(target_os = "android") {
                self.set_status(cx, app_t("app_template_from_device_help"));
            } else {
                self.open_app_template_folder_pick(cx, false);
            }
        }
    }

    pub(crate) fn handle_app_template_settings_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let bundled = menu_templates::all();
        let chips = [
            (ids!(settings_public_tmpl_0), ids!(settings_public_tmpl_0_on), 0usize),
            (ids!(settings_public_tmpl_1), ids!(settings_public_tmpl_1_on), 1),
            (ids!(settings_public_tmpl_2), ids!(settings_public_tmpl_2_on), 2),
        ];
        for (idle, active, idx) in chips {
            if self.ui.button(cx, idle).clicked(actions)
                || self.ui.button(cx, active).clicked(actions)
            {
                if let Some(tmpl) = bundled.get(idx) {
                    let mut st = app_state().lock().unwrap();
                    if st.settings_app_template_selected_id.as_deref() == Some(tmpl.id.as_str()) {
                        st.settings_app_template_selected_id = None;
                    } else {
                        st.settings_app_template_selected_id = Some(tmpl.id.clone());
                    }
                    st.app_template_wipe_pending = None;
                }
                self.paint_settings_app_templates(cx);
            }
        }
        if self.ui.button(cx, ids!(settings_public_tmpl_import)).clicked(actions)
            && !menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
        {
            match self.arm_settings_app_template_bundled(cx) {
                Ok(()) => {
                    let _ = self.paint_settings_app_templates(cx);
                }
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(settings_app_template_from_device)).clicked(actions)
            && !menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
        {
            if cfg!(target_os = "android") {
                self.set_status(cx, app_t("app_template_from_device_help"));
            } else {
                self.open_app_template_folder_pick(cx, true);
            }
        }
        if self.ui.button(cx, ids!(settings_app_tmpl_wipe_continue)).clicked(actions) {
            self.run_settings_app_template_continue(cx);
        }
        if self.ui.button(cx, ids!(settings_app_tmpl_wipe_cancel)).clicked(actions) {
            app_state().lock().unwrap().app_template_wipe_pending = None;
            self.paint_settings_app_templates(cx);
        }
        if self.ui.button(cx, ids!(settings_app_tmpl_cancel)).clicked(actions)
            && menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
        {
            request_books_job_cancel();
            self.paint_import_progress_chrome(cx);
            self.paint_settings_app_templates(cx);
        }
    }
}
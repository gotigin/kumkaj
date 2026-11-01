impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.sync_tab_chevrons(cx);
        {
            let r = self.ui.view(cx, ids!(body_stack)).area().rect(cx);
            *page_viewport().lock().unwrap() = (r.pos.y, r.size.y);
        }
        if matches!(event, Event::Startup) {
            // Before CreateWindow: override the DSL default with the last session.
            self.restore_window_geom(cx);
            self.ensure_store(cx);
            // Photo thumbs are made on worker threads; wake the UI when one lands.
            thumbs::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            lan_images::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            phone::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            print_share::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            // Printer Find is Settings → Find printers only (never at startup).
            net::set_books_changed_hook(|| {
                BOOKS_CHANGED.store(true, std::sync::atomic::Ordering::SeqCst);
                makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
            });
            // Started with the computer: devices and backups don't wait for
            // someone to unlock the screen.
            if startup::launched_at_boot() {
                let st = app_state().lock().unwrap();
                if let Some(store) = st.store.as_ref()
                    && store.device_role().ok().flatten().as_deref() == Some("server")
                {
                    let path = store.db_path().to_path_buf();
                    if store.extra_tills_allowed().unwrap_or(false) && net::ensure_server(path.clone()).is_ok() {
                        firewall_job(|| Ok(net::firewall_allows_phones()));
                    }
                    start_auto_backups(path);
                }
            }
        }
        if matches!(event, Event::Signal) && (thumbs::take_ready() || lan_images::take_ready()) {
            self.ui.redraw(cx);
        }
        if matches!(event, Event::Signal) {
            if let Some(result) = backup_once().lock().unwrap().take() {
                match result {
                    Ok(path) => {
                        self.ui.widget(cx, ids!(backup_path_label)).set_text(
                            cx,
                            &plain_display_text(&format!("Backup: {}", path.display())),
                        );
                        self.set_status(cx, "Backup created");
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
            if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst)
                || restore_busy().load(std::sync::atomic::Ordering::SeqCst)
                || csv_import_busy().load(std::sync::atomic::Ordering::SeqCst)
                || shelf_mat_busy().load(std::sync::atomic::Ordering::SeqCst)
                || backup_copy_busy().load(std::sync::atomic::Ordering::SeqCst)
                || backup_busy().load(std::sync::atomic::Ordering::SeqCst)
                || aggregator_import_busy().load(std::sync::atomic::Ordering::SeqCst)
                || diag_export_busy().load(std::sync::atomic::Ordering::SeqCst)
            {
                self.paint_import_progress_chrome(cx);
            } else if print_share::is_finding_printers()
                || printer_add_busy().load(std::sync::atomic::Ordering::SeqCst)
                || client_find_busy().load(std::sync::atomic::Ordering::SeqCst)
            {
                self.paint_submit_busy_chrome(cx);
            }
            if let Some(result) = printer_add_once().lock().unwrap().take() {
                match result {
                    Ok(()) => {
                        self.set_status(cx, app_state().lock().unwrap().t("printers_saved"));
                        self.sync_chrome(cx);
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_state().lock().unwrap().t("cancelled"));
                        self.sync_chrome(cx);
                    }
                    Err(e) => {
                        self.set_status(cx, e);
                        self.sync_chrome(cx);
                    }
                }
            }
            if let Some(result) = aggregator_import_once().lock().unwrap().take() {
                match result {
                    Ok(msg) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.set_status(cx, msg);
                        self.sync_chrome(cx);
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_state().lock().unwrap().t("cancelled"));
                        self.sync_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
            if let Some(result) = diag_export_once().lock().unwrap().take() {
                match result {
                    Ok((paths, for_send)) => {
                        self.sync_diag_panel(cx);
                        self.offer_diag_bundle_save(cx, paths, for_send);
                        self.sync_chrome(cx);
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_state().lock().unwrap().t("cancelled"));
                        self.sync_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
            if let Some(result) = backup_copy_once().lock().unwrap().take() {
                match result {
                    Ok((file, name)) => {
                        app_state().lock().unwrap().pending_backup_copy = Some(file);
                        let title = app_state().lock().unwrap().t("save_backup_copy");
                        cx.open_save_file_dialog(
                            FileDialog::new()
                                .set_title(title)
                                .set_filename(name)
                                .add_filter("Backups".into(), vec!["sqlite3".into()])
                                .set_id(live_id!(backup_save_pick)),
                        );
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
            if let Some(result) = restore_once().lock().unwrap().take() {
                match result {
                    Ok((backup, before)) => {
                        self.ensure_store(cx);
                        let _ = app_state().lock().unwrap().reload_all();
                        self.set_status(
                            cx,
                            format!(
                                "Restored from {} — the books before it are saved as {}",
                                backup.file_name().unwrap_or_default().to_string_lossy(),
                                before.file_name().unwrap_or_default().to_string_lossy()
                            ),
                        );
                        self.sync_chrome(cx);
                    }
                    Err(e) => {
                        if app_state().lock().unwrap().store.is_none() {
                            self.ensure_store(cx);
                        }
                        self.set_status(cx, e);
                        self.sync_chrome(cx);
                    }
                }
            }
            if let Some(result) = csv_import_once().lock().unwrap().take() {
                match result {
                    Ok(msg) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.set_status(cx, msg);
                        self.sync_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
            if let Some(result) = shelf_mat_once().lock().unwrap().take() {
                match result {
                    Ok(n) if n > 0 => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.set_status(cx, format!("Menu photos ready ({n})"));
                        self.ui.redraw(cx);
                    }
                    Ok(_) => {}
                    Err(e) => self.set_status(cx, e),
                }
            }
            // Menu import completion (progress is painted above while busy).
            if let Some(outcome) = menu_import_once().lock().unwrap().take() {
                let msg = match outcome.result {
                    Ok(done) => {
                        let reload = app_state().lock().unwrap().reload_all();
                        if let Err(e) = reload {
                            Err(e)
                        } else {
                            let st = app_state().lock().unwrap();
                            Ok(match &outcome.kind {
                                MenuImportKind::Shelf => st
                                    .t("template_imported")
                                    .replace("{d}", &done.items_added.to_string())
                                    .replace("{c}", &done.categories_added.to_string()),
                                MenuImportKind::Client { title, .. } => st
                                    .t("client_template_imported")
                                    .replace("{d}", &done.items_added.to_string())
                                    .replace("{c}", &done.categories_added.to_string())
                                    .replace("{title}", title),
                            })
                        }
                    }
                    Err(e) => Err(e),
                };
                {
                    let mut st = app_state().lock().unwrap();
                    st.template_confirm = false;
                    st.client_template_confirm = None;
                }
                match msg {
                    Ok(m) => self.set_status(cx, m),
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_state().lock().unwrap().t("cancelled"))
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
            if BACKUP_MADE.swap(false, std::sync::atomic::Ordering::SeqCst) {
                self.sync_chrome(cx);
            }
            if BOOKS_CHANGED.swap(false, std::sync::atomic::Ordering::SeqCst) {
                let host = {
                    let mut st = app_state().lock().unwrap();
                    let host = st.device_role.is_host() && st.session.is_some();
                    if host {
                        st.refresh_live_lists();
                    }
                    host
                };
                if host {
                    self.sync_chrome(cx);
                }
            }
            self.take_lan_answers(cx);
            // A printer that did not print says so where people look; newly
            // found printers show in Settings.
            if let Some(problem) = print_share::take_problems().pop() {
                self.set_status(cx, problem);
            }
            if app_state().lock().unwrap().screen == Screen::Settings {
                self.show_found_printers(cx);
                if print_share::find_printers_was_cancelled() {
                    self.set_status(cx, app_state().lock().unwrap().t("cancelled"));
                }
                self.paint_submit_busy_chrome(cx);
            }
            let fresh = std::mem::take(&mut firewall_state().lock().unwrap().fresh);
            if fresh {
                self.sync_chrome(cx);
            }
            let found = discovery_result().lock().unwrap().take();
            if let Some(found) = found {
                self.show_found_servers(cx, found);
                self.paint_submit_busy_chrome(cx);
            }
        }
        if matches!(event, Event::WindowGeomChange(_)) {
            let restoring = self.apply_pending_window_geom_after_create(cx);
            self.enforce_min_desktop_window_size(cx);
            self.sync_dev_size(cx);
            let unlocked = {
                let st = app_state().lock().unwrap();
                !matches!(
                    st.screen,
                    Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                )
            };
            self.apply_shell_layout(cx, unlocked);
            if !restoring {
                self.persist_window_geom(cx);
            }
        }
        self.match_event(cx, event);
        if let Event::NextFrame(e) = event {
            let ours = nav_autoscroll_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if ours {
                self.nav_drag_autoscroll(cx);
            }
            let accordion_ours = accordion_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if accordion_ours {
                self.tick_books_accordion(cx);
            }
        }
        if self.pointer_sidebar_settings_drag(cx, event) {
            return;
        }
        if self.pointer_nav_drag(cx, event) {
            return;
        }
        if self.pointer_navigates(cx, event) {
            return;
        }
        // Horizontal strips: claim sideways only while they can move; never
        // steal vertical page scroll when the pointer sits on chips/ticket.
        if matches!(event, Event::Scroll(_)) {
            for (_, scroller) in TAB_STRIPS {
                let tabs = self.ui.view(cx, scroller);
                claim_hstrip_nested_scroll(cx, &tabs, event);
            }
            for path in [
                ids!(bottom_nav_scroll),
                ids!(rail_scroll),
                ids!(app_hscroll),
                ids!(pos_ticket),
            ] {
                let strip = self.ui.view(cx, path);
                claim_hstrip_nested_scroll(cx, &strip, event);
            }
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
        // Hot reload re-applies DSL (visibility resets). Restore from AppState.
        if matches!(event, Event::LiveEdit | Event::ScriptReapply | Event::Startup) {
            self.sync_chrome(cx);
        }
    }
}

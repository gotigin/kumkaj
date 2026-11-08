impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.sync_tab_chevrons(cx);
        if matches!(event, Event::Startup) {
            // Before CreateWindow: override the DSL default with the last session.
            self.restore_window_geom(cx);
            self.ensure_store(cx);
            // Photo thumbs are made on worker threads; wake the UI when one lands.
            thumbs::set_notify(|| {
                makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal()
            });
            // Rate photos / party logos pulled from the Server (paired devices).
            lan_images::set_notify(|| {
                makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal()
            });
            phone::set_notify(|| {
                makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal()
            });
            print_share::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            // Printer Find is Settings → Find printers only (never at startup).
            net::set_books_changed_hook(|| {
                BOOKS_CHANGED.store(true, std::sync::atomic::Ordering::SeqCst);
                FACET_DIRTY.store(true, std::sync::atomic::Ordering::SeqCst);
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
                    if store.extra_tills_allowed().unwrap_or(false)
                        && net::ensure_server(path.clone()).is_ok()
                    {
                        firewall_job(|| Ok(net::firewall_allows_phones()));
                    }
                    start_auto_backups(path);
                }
            }
        }
        if matches!(event, Event::Signal) && (thumbs::take_ready() | lan_images::take_ready()) {
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
            if restore_busy().load(std::sync::atomic::Ordering::SeqCst)
                || csv_import_busy().load(std::sync::atomic::Ordering::SeqCst)
                || backup_copy_busy().load(std::sync::atomic::Ordering::SeqCst)
                || backup_busy().load(std::sync::atomic::Ordering::SeqCst)
                || branch_busy().load(std::sync::atomic::Ordering::SeqCst)
                || diag_export_busy().load(std::sync::atomic::Ordering::SeqCst)
                || printer_add_busy().load(std::sync::atomic::Ordering::SeqCst)
                || print_share::is_finding_printers()
                || client_find_busy().load(std::sync::atomic::Ordering::SeqCst)
            {
                if let Some(msg) = books_work_status().lock().unwrap().clone() {
                    self.set_status(cx, msg);
                }
                self.paint_submit_busy_chrome(cx);
            }
            if let Some(result) = printer_add_once().lock().unwrap().take() {
                if let Some(draft) = printer_add_result_draft().lock().unwrap().take() {
                    let mut st = app_state().lock().unwrap();
                    st.printer_draft = draft;
                }
                clear_books_work_status();
                printer_add_busy().store(false, std::sync::atomic::Ordering::SeqCst);
                match result {
                    Ok(()) => {
                        self.set_status(cx, app_t("printers_saved"));
                        self.paint_submit_busy_chrome(cx);
                        self.sync_chrome(cx);
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_t("cancelled"));
                        self.paint_submit_busy_chrome(cx);
                        self.sync_chrome(cx);
                    }
                    Err(e) => {
                        self.set_status(cx, e);
                        self.paint_submit_busy_chrome(cx);
                        self.sync_chrome(cx);
                    }
                }
            }
            if let Some(result) = branch_once().lock().unwrap().take() {
                match result {
                    Ok(msg) => {
                        let _ = app_state().lock().unwrap().refresh_branches_text();
                        self.set_status(cx, msg);
                        self.sync_chrome(cx);
                    }
                    Err(e) => {
                        self.set_status(cx, e);
                        self.paint_submit_busy_chrome(cx);
                    }
                }
            }
            if let Some(result) = diag_export_once().lock().unwrap().take() {
                match result {
                    Ok((paths, for_send)) => {
                        self.sync_diag_panel(cx);
                        self.offer_diag_bundle_save(cx, paths, for_send);
                        self.paint_submit_busy_chrome(cx);
                    }
                    Err(e) => {
                        self.show_error(cx, e);
                        self.paint_submit_busy_chrome(cx);
                    }
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
                if let Some(report) = carrier_bill_report_once().lock().unwrap().take() {
                    app_state().lock().unwrap().last_carrier_bill_report = report;
                }
                match result {
                    Ok(msg) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.set_status(cx, msg);
                        self.sync_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
                }
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
                // Empty KOT route is a setup gap, not success ("not printed"
                // used to match classify OK "printed" → green face).
                self.show_warn(cx, problem);
            }
            // Settings → Printers: repaint what Find found only when Find's
            // state moved. Rebuilding it with the pool and route cards on
            // every wake (photo thumbs, the device worker, LAN) redrew
            // Settings for nothing — Khaynjo learned the same, 2026-11.
            if app_state().lock().unwrap().screen == Screen::Settings {
                let now = (print_share::is_finding_printers(), print_share::found_printers());
                let before = found_printers_painted().lock().unwrap().clone();
                if before.as_ref() != Some(&now) {
                    let stopped = before.is_some_and(|(looking, _)| looking) && !now.0;
                    self.show_found_printers(cx);
                    if stopped && print_share::find_printers_was_cancelled() {
                        let msg = app_state().lock().unwrap().t("cancelled");
                        self.set_status(cx, msg);
                    }
                    self.paint_submit_busy_chrome(cx);
                }
            }
            let found = discovery_result().lock().unwrap().take();
            if let Some(found) = found {
                self.show_found_servers(cx, found);
            }
            if let Some(result) = role_open_once().lock().unwrap().take() {
                match result {
                    Ok(RoleOpenReady::Client(store)) => {
                        self.finish_client_mode_opened(cx, store)
                    }
                    Ok(RoleOpenReady::ServerNew(store)) => {
                        self.finish_server_new_opened(cx, store)
                    }
                    Ok(RoleOpenReady::ServerExisting {
                        store,
                        switching_in,
                    }) => self.finish_server_existing_opened(cx, store, switching_in),
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_t("cancelled"));
                        self.paint_role_open_busy_chrome(cx);
                        self.sync_chrome(cx);
                    }
                    Err(e) => {
                        self.set_status(cx, e);
                        self.paint_role_open_busy_chrome(cx);
                        self.sync_chrome(cx);
                    }
                }
            } else if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_role_open_busy_chrome(cx);
            }
            self.maybe_smoke_auto_server(cx);
            self.maybe_smoke_auto_unlock(cx);
            self.tick_smoke_script(cx);
            let fresh = std::mem::take(&mut firewall_state().lock().unwrap().fresh);
            if fresh {
                self.sync_chrome(cx);
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
                self.tick_ui_state_anims(cx);
            }
            self.tick_smoke_script(cx);
        }
        // Type-or-pick lists first (popup.rs): an outside press closes the
        // open list, a tap on a field opens its list. Before every pointer
        // handler below — some return early (a nav-tab tap). The press
        // itself still reaches whatever is under it.
        self.route_popup_input(cx, event);
        if self.pointer_sidebar_settings_drag(cx, event) {
            return;
        }
        if self.pointer_nav_drag(cx, event) {
            return;
        }
        if self.pointer_navigates(cx, event) {
            return;
        }
        // Sideways strips (tab strips, nav bars, the booking desk) need no
        // pre-claim: Makepad gives a wheel to the innermost scroll view first
        // and marks it handled only when that view moved, so the strip pans
        // and its parent takes over at the edge. Pre-marking X handled here
        // (claim_hstrip_nested_scroll, removed 2026-11-02) stopped the strip
        // itself — a trackpad moved the booking desk one step, then nothing.
        // Empty header (not Back or the amount eye) is a window drag, the same
        // as the caption bar: press and move, and the desktop moves the window.
        if let Event::WindowDragQuery(dq) = event {
            let header = self.ui.view(cx, ids!(header)).area().rect(cx);
            if header.size.x > 1.0 && header.contains(dq.abs) {
                let on_control = [ids!(header_back), ids!(amounts_eye)].into_iter().any(|id| {
                    let r = self.ui.button(cx, id).area().rect(cx);
                    r.size.x > 1.0 && r.contains(dq.abs)
                });
                if !on_control {
                    dq.response
                        .set(makepad_widgets::WindowDragQueryResponse::Caption);
                }
            }
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
        // A control in the 48 px edge strip (a button, a card, the booking
        // desk's sideways panes) took this press: the control wins, the edge
        // swipe disarms.
        self.disarm_edge_swipe_if_claimed(cx, event);
        // Hot reload re-applies DSL (visibility resets). Restore from AppState.
        if matches!(
            event,
            Event::LiveEdit | Event::ScriptReapply | Event::Startup
        ) {
            self.sync_chrome(cx);
        }
    }
}

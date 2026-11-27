/// Every control placed inside `header`. The rest of the header is a window
/// drag on desktop (`WindowDragQuery` → `Caption`): the OS takes a press there
/// for a window move before the button sees it. The collapsed sidebar's `>`
/// (`rail_expand_btn`) never opened on Windows / Linux until it was listed
/// (2026-11-10). Add every new header button here —
/// `nav_order_tests::header_buttons_are_not_window_drag` checks this list
/// against the DSL.
const HEADER_CONTROLS: &[&str] = &["rail_expand_btn", "header_back", "amounts_eye"];

/// Every control in `nav_rail` that is not a nav row (rows are never a drag).
/// The sidebar's empty top corner and the space under its last row move the
/// window like the empty header (owner, 2026-11-14: with the window's top
/// bar above the screen there was nothing left to drag). The gaps between
/// rows stay the sidebar's own, so a finger can still scroll a long one.
/// `nav_order_tests::rail_controls_are_not_window_drag` checks this list.
const RAIL_CONTROLS: &[&str] = &["rail_collapse_btn"];
/// The sidebar's right edge stays its scroll bar's.
const RAIL_SCROLLBAR_PX: f64 = 12.0;

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    /// One bad event must not end the app. A panic in the shell used to unwind
    /// out of the event loop: Android aborted (Play review "keeps crashing")
    /// and the desktop window closed. The panic hook has already written it
    /// to diagnostics (with a backtrace); the frame is dropped and the next
    /// tap works.
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        let started = Instant::now();
        let tree_before = cx.widget_tree().stats();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.handle_event_unguarded(cx, event)
        }));
        if outcome.is_err() {
            // A half-drawn frame or half-applied action: paint everything
            // again from AppState.
            cx.redraw_all();
        }
        // A job that ended without its own result line must not leave its
        // progress text behind ("Preparing desk…" on Lock, DEV.md 7b).
        if matches!(event, Event::Signal | Event::Actions(_) | Event::Timer(_)) {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.clear_stale_work_status(cx)
            }));
        }
        let tree_after = cx.widget_tree().stats();
        note_ui_stall(
            event,
            started.elapsed(),
            (
                tree_after.lookups.saturating_sub(tree_before.lookups),
                tree_after.cache_misses.saturating_sub(tree_before.cache_misses),
                tree_after.walk_nodes.saturating_sub(tree_before.walk_nodes),
                tree_after.invalidations.saturating_sub(tree_before.invalidations),
                tree_after.stores_skipped.saturating_sub(tree_before.stores_skipped),
            ),
        );
    }
}

impl App {
    /// While Opening client/server or Pairing: Cancel, scroll, and Back/Forward
    /// stay live on every OS (bug 7b). Skip the full Signal/match_event SQL path
    /// so the UI does not freeze until the worker finishes.
    /// Returns true when the event was fully handled here.
    fn drive_books_busy_shell(&mut self, cx: &mut Cx, event: &Event) -> bool {
        let role_busy = role_open_busy().load(std::sync::atomic::Ordering::SeqCst);
        let pair_busy = client_pair_busy().load(std::sync::atomic::Ordering::SeqCst);
        if !role_busy && !pair_busy {
            // Mid-finish can hold app_state — still deliver pointer events.
            if app_state().try_lock().is_err() {
                self.ui.handle_event(cx, event, &mut Scope::empty());
                return true;
            }
            return false;
        }

        if matches!(event, Event::Signal) {
            if thumbs::take_ready() | lan_images::take_ready() {
                self.ui.redraw(cx);
            }
            let found = discovery_result().lock().unwrap().take();
            if let Some(found) = found {
                self.show_found_servers(cx, found);
            }
            // Pair results land through LAN once.
            self.take_lan_answers(cx);
            if let Some(result) = role_open_once().lock().unwrap().take() {
                note_role_open_ui_took();
                match result {
                    Ok(RoleOpenReady::Client(store)) => {
                        if role_cancelled() {
                            drop_store_off_ui(Some(store));
                            end_role_open_busy();
                            self.set_status(cx, app_t("cancelled"));
                            self.paint_role_open_busy_chrome(cx);
                        } else {
                            self.finish_client_mode_opened(cx, store);
                        }
                    }
                    Ok(RoleOpenReady::ServerNew(store)) => {
                        self.finish_server_new_opened(cx, store)
                    }
                    Ok(RoleOpenReady::ServerExisting {
                        store,
                        switching_in,
                    }) => self.finish_server_existing_opened(cx, store, switching_in),
                    Ok(RoleOpenReady::ServerRestored(store)) => {
                        self.finish_server_restored_opened(cx, store)
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_t("cancelled"));
                        self.paint_role_open_busy_chrome(cx);
                    }
                    Err(e) => {
                        self.set_status(cx, e);
                        self.paint_role_open_busy_chrome(cx);
                        *deferred_role_open_sync_frame().lock().unwrap() =
                            Some(cx.new_next_frame());
                    }
                }
            } else if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_role_open_busy_chrome(cx);
            }
            if client_pair_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_pair_busy_chrome(cx);
            }
            return true;
        }

        if let Event::Actions(actions) = event {
            if role_busy {
                if self.ui.button(cx, ids!(device_role_cancel)).clicked(actions)
                    || self.ui.button(cx, ids!(header_back)).clicked(actions)
                {
                    request_role_open_cancel();
                    self.set_status(cx, app_t("cancelled"));
                    self.paint_role_open_busy_chrome(cx);
                }
            }
            if pair_busy {
                if self.ui.button(cx, ids!(client_pair_cancel)).clicked(actions)
                    || self.ui.button(cx, ids!(header_back)).clicked(actions)
                {
                    self.abandon_client_pair(cx);
                }
            }
            // Widget hit-testing for the Cancel face (no full handle_actions).
            self.ui.handle_event(cx, event, &mut Scope::empty());
            if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_role_open_busy_chrome(cx);
            }
            if client_pair_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_pair_busy_chrome(cx);
            }
            return true;
        }

        // Draw / keys / Android BackPressed need MatchEvent; skip Actions storm.
        if matches!(
            event,
            Event::Draw(_) | Event::KeyDown(_) | Event::KeyUp(_) | Event::BackPressed { .. }
        ) {
            self.match_event(cx, event);
            if let Event::Draw(_) = event {
                note_nav_draw();
            }
            if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_role_open_busy_chrome(cx);
            }
            if client_pair_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_pair_busy_chrome(cx);
            }
            return true;
        }

        if let Event::NextFrame(e) = event {
            self.finish_nav_veil(cx, e);
            let role_sync_ours = deferred_role_open_sync_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if role_sync_ours {
                *deferred_role_open_sync_frame().lock().unwrap() = None;
                // Light chrome only — full sync_chrome waits until not busy.
                if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
                    self.paint_role_open_busy_chrome(cx);
                } else {
                    self.sync_chrome(cx);
                }
            }
            return true;
        }

        // Mouse Back/Forward + edge swipe (Android / desktop).
        if self.pointer_navigates(cx, event) {
            if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_role_open_busy_chrome(cx);
            }
            if client_pair_busy().load(std::sync::atomic::Ordering::SeqCst) {
                self.paint_pair_busy_chrome(cx);
            }
            return true;
        }

        // Scroll, Cancel hit-test, accordion — no SQL.
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.disarm_edge_swipe_if_claimed(cx, event);
        if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
            self.paint_role_open_busy_chrome(cx);
        }
        if client_pair_busy().load(std::sync::atomic::Ordering::SeqCst) {
            self.paint_pair_busy_chrome(cx);
        }
        true
    }

    fn handle_event_unguarded(&mut self, cx: &mut Cx, event: &Event) {
        if self.drive_books_busy_shell(cx, event) {
            return;
        }
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
            // A printer's paper is read on a worker (CUPS / the Windows print API).
            os_paper::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            // Windows answers "start with this computer" on a worker (reg.exe).
            startup::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
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
        if matches!(event, Event::Signal) && os_paper::take_ready() {
            // A driver answered: what it reports replaces the printer's
            // reported paper, and a printer with no virtual paper gets its
            // driver's (never changing one already set); then every face
            // that shows a printer's paper and the preview are drawn again
            // (docs/PRINTER.md § Paper and preview).
            let open = {
                let mut st = app_state().lock().unwrap();
                if st.device_role != DeviceRole::Client
                    && st.printer_draft.take_driver_answers(&os_paper::answers(), false) > 0
                {
                    let saved = match st.data_base.clone() {
                        Some(base) => print_share::save_device_printers(&base, &st.printer_draft),
                        None => Err("Storage not ready".to_string()),
                    };
                    // Saving configures the print queue too.
                    if let Err(e) = saved {
                        log!("printer papers not saved: {e}");
                    }
                }
                st.suggest_open
            };
            if open == Some(SuggestField::JobPreviewPaper) {
                app_state()
                    .lock()
                    .unwrap()
                    .open_suggest(SuggestField::JobPreviewPaper, "");
                self.ui
                    .widget(cx, SuggestField::JobPreviewPaper.list())
                    .redraw(cx);
            }
            if app_state().lock().unwrap().screen == Screen::Settings {
                self.paint_printer_rows(cx);
                self.sync_job_preview(cx);
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
                let work = books_work_status().lock().unwrap().clone();
                if let Some(msg) = work {
                    self.show_work_status(cx, msg);
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
            let copy_written = backup_copy_write_once().lock().unwrap().take();
            if let Some(result) = copy_written {
                self.finish_backup_copy_write(cx, result);
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
            self.take_cloud_answers(cx);
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
            let printers_sheet = self.ui.modal(cx, ids!(printers_modal)).is_open();
            if printers_sheet || app_state().lock().unwrap().screen == Screen::Settings {
                let finding = print_share::is_finding_printers();
                // Armed by the Find tap; the scan wakes the UI only when it is
                // done (any repaint in between may already show the result,
                // so the painted list is no edge to trust).
                let finished = !finding
                    && find_printers_armed().swap(false, std::sync::atomic::Ordering::SeqCst);
                if finished {
                    if print_share::find_printers_was_cancelled() {
                        let msg = app_state().lock().unwrap().t("cancelled");
                        self.set_status(cx, msg);
                    } else {
                        // Find saves what it found (sheet and panel), ready to
                        // pick and to give a job — there is no manual add.
                        match self.save_found_printers_to_pool() {
                            Ok(0) => {}
                            Ok(_) => {
                                self.sync_printer_fields(cx);
                                self.set_status(cx, app_t("printers_saved"));
                            }
                            Err(e) => self.set_status(cx, e),
                        }
                    }
                }
                if printers_sheet && (finding || finished) {
                    self.paint_printers_modal(cx);
                }
                let now = (finding, print_share::found_printers());
                let before = found_printers_painted().lock().unwrap().clone();
                if before.as_ref() != Some(&now) {
                    self.show_found_printers(cx);
                    self.paint_submit_busy_chrome(cx);
                }
            }
            let found = discovery_result().lock().unwrap().take();
            if let Some(found) = found {
                self.show_found_servers(cx, found);
            }
            if let Some(result) = role_open_once().lock().unwrap().take() {
                note_role_open_ui_took();
                match result {
                    Ok(RoleOpenReady::Client(store)) => {
                        if role_cancelled() {
                            drop_store_off_ui(Some(store));
                            end_role_open_busy();
                            self.set_status(cx, app_t("cancelled"));
                            self.paint_role_open_busy_chrome(cx);
                        } else {
                            self.finish_client_mode_opened(cx, store);
                        }
                    }
                    Ok(RoleOpenReady::ServerNew(store)) => {
                        self.finish_server_new_opened(cx, store)
                    }
                    Ok(RoleOpenReady::ServerExisting {
                        store,
                        switching_in,
                    }) => self.finish_server_existing_opened(cx, store, switching_in),
                    Ok(RoleOpenReady::ServerRestored(store)) => {
                        self.finish_server_restored_opened(cx, store)
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_t("cancelled"));
                        self.paint_role_open_busy_chrome(cx);
                    }
                    Err(e) => {
                        self.set_status(cx, e);
                        self.paint_role_open_busy_chrome(cx);
                        *deferred_role_open_sync_frame().lock().unwrap() = Some(cx.new_next_frame());
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
        if let Event::Draw(_) = event {
            note_nav_draw();
        }
        if let Event::NextFrame(e) = event {
            self.finish_nav_veil(cx, e);
            let accordion_ours = accordion_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if accordion_ours {
                self.tick_ui_state_anims(cx);
            }
            let role_sync_ours = deferred_role_open_sync_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if role_sync_ours {
                *deferred_role_open_sync_frame().lock().unwrap() = None;
                self.sync_chrome(cx);
            }
            let books_overview_ours = deferred_books_overview_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if books_overview_ours {
                *deferred_books_overview_frame().lock().unwrap() = None;
                self.paint_books_overview_sql(cx);
            }
            self.tick_smoke_script(cx);
        }
        // Type-or-pick lists first (popup.rs): an outside press closes the
        // open list, a tap on a field opens its list. Before every pointer
        // handler below — some return early (a nav-tab tap). The press
        // itself still reaches whatever is under it.
        self.route_popup_input(cx, event);
        // Copy / Select all menu: same outside-press rule (selection_menu.rs).
        self.route_selection_menu_input(cx, event);
        if self.pointer_navigates(cx, event) {
            return;
        }
        // Sideways strips (tab strips, nav bars, the booking desk) need no
        // pre-claim: Makepad gives a wheel to the innermost scroll view first
        // and marks it handled only when that view moved, so the strip pans
        // and its parent takes over at the edge. Pre-marking X handled here
        // (claim_hstrip_nested_scroll, removed 2026-11-02) stopped the strip
        // itself — a trackpad moved the booking desk one step, then nothing.
        // Empty header (not a HEADER_CONTROLS button) is a window drag, the same
        // as the caption bar: press and move, and the desktop moves the window.
        if let Event::WindowDragQuery(dq) = event
            && self.window_drag_at(cx, dq.abs)
        {
            dq.response
                .set(makepad_widgets::WindowDragQueryResponse::Caption);
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

impl App {
    /// Window chrome with nothing to press: the empty header, and the
    /// sidebar's top corner and the space under its last row. A press there
    /// moves the window on desktop (`WindowDragQuery` → `Caption`). Page
    /// content never is — a press on its empty space starts a selection.
    fn window_drag_at(&self, cx: &mut Cx, abs: DVec2) -> bool {
        let over = |rect: Rect| rect.size.x > 1.0 && rect.contains(abs);
        let header = self.ui.view(cx, ids!(header)).area().rect(cx);
        if over(header) {
            return !HEADER_CONTROLS.iter().any(|name| {
                let r = self.ui.button(cx, &[LiveId::from_str(name)]).area().rect(cx);
                over(r)
            });
        }
        let rail = self.ui.view(cx, ids!(nav_rail));
        if !rail.visible() {
            return false;
        }
        let r = rail.area().rect(cx);
        if !over(r) || abs.x >= r.pos.x + r.size.x - RAIL_SCROLLBAR_PX {
            return false;
        }
        if RAIL_CONTROLS.iter().any(|name| {
            let b = self.ui.button(cx, &[LiveId::from_str(name)]).area().rect(cx);
            over(b)
        }) {
            return false;
        }
        let rows: Vec<Rect> = self
            .nav_slot_wrappers(true)
            .iter()
            .map(|path| self.ui.view(cx, path))
            .filter(|v| v.visible())
            .map(|v| v.area().rect(cx))
            .filter(|r| r.size.y > 1.0)
            .collect();
        let (Some(top), Some(bottom)) = (
            rows.iter().map(|r| r.pos.y).reduce(f64::min),
            rows.iter().map(|r| r.pos.y + r.size.y).reduce(f64::max),
        ) else {
            return true;
        };
        abs.y < top || abs.y > bottom
    }
}

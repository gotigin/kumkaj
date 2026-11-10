/// Every control placed inside `header`. The rest of the header is a window
/// drag on desktop (`WindowDragQuery` → `Caption`): the OS takes a press there
/// for a window move before the button sees it. The collapsed sidebar's `>`
/// (`rail_expand_btn`) never opened on Windows / Linux until it was listed
/// (2026-11-10). Add every new header button here —
/// `nav_order_tests::header_buttons_are_not_window_drag` checks this list
/// against the DSL.
const HEADER_CONTROLS: &[&str] = &["rail_expand_btn", "header_back", "amounts_eye"];

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
    fn handle_event_unguarded(&mut self, cx: &mut Cx, event: &Event) {
        // Opening the till or finishing DeviceSetup role open holds app_state
        // for a long time. Do not wait on that lock — Cancel must keep working.
        if app_state().try_lock().is_err() {
            if client_pair_busy().load(std::sync::atomic::Ordering::SeqCst) {
                if let Event::Actions(actions) = event {
                    if self.ui.button(cx, ids!(client_pair_cancel)).clicked(actions) {
                        self.abandon_client_pair(cx);
                    }
                }
                self.ui.handle_event(cx, event, &mut Scope::empty());
                self.paint_pair_busy_chrome(cx);
                return;
            }
            if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
                if let Event::Actions(actions) = event {
                    if self.ui.button(cx, ids!(device_role_cancel)).clicked(actions) {
                        request_role_open_cancel();
                    }
                }
                self.ui.handle_event(cx, event, &mut Scope::empty());
                self.paint_role_open_busy_chrome(cx);
                return;
            }
        }
        self.sync_tab_chevrons(cx);
        {
            // Page-window lists (Added items, Menu, …) need the *scrolling*
            // page rect — not body_stack — or the window is wrong and scroll
            // inside the cart feels broken.
            let screen = app_state().lock().unwrap().screen;
            let path = Self::screen_scroll_view(screen);
            let r = self.ui.view(cx, path).area().rect(cx);
            if r.size.y > 1.0 {
                *page_viewport().lock().unwrap() = (r.pos.y, r.size.y);
            } else {
                let r = self.ui.view(cx, ids!(body_stack)).area().rect(cx);
                *page_viewport().lock().unwrap() = (r.pos.y, r.size.y);
            }
            // The Floor tables scroll sideways in `floor_scroll`: its box is
            // the viewport the grid draws its visible columns for.
            if screen == Screen::Floor {
                let r = self.ui.view(cx, ids!(floor_scroll)).area().rect(cx);
                if r.size.x > 1.0 {
                    *floor_hviewport().lock().unwrap() = (r.pos.x, r.size.x);
                }
            }
        }
        if matches!(event, Event::Startup) {
            // Before CreateWindow: override the DSL default with the last session.
            self.restore_window_geom(cx);
            self.ensure_store(cx);
            // Photo thumbs are made on worker threads; wake the UI when one lands.
            thumbs::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            lan_images::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            // Windows answers "start with this computer" on a worker (reg.exe).
            startup::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
            phone::set_notify(|| makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal());
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
                    if store.extra_tills_allowed().unwrap_or(false) && net::ensure_server(path.clone()).is_ok() {
                        firewall_job(|| Ok(net::firewall_allows_phones()));
                    }
                    start_auto_backups(path);
                }
            }
        }
        if matches!(event, Event::Signal) && (thumbs::take_ready() || lan_images::take_ready()) {
            // Targeted redraw only. Full-tree `ui.redraw` while a FingerDown is
            // in flight drops the first ProductList / MenuList / dish-picker
            // tap after photos start landing (open menu → tap → nothing).
            for path in MENU_PICK_GRIDS {
                self.ui.widget(cx, path).redraw(cx);
            }
            for path in [
                ids!(product_list),
                ids!(product_list_menu),
                ids!(removed_list),
                ids!(dish_pick_list),
                ids!(category_pick_list),
                ids!(cart_list),
                ids!(amend_cart_list),
                ids!(category_list),
                ids!(draft_preview),
            ] {
                self.ui.widget(cx, path).redraw(cx);
            }
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
                if let Some(draft) = printer_add_result_draft().lock().unwrap().take() {
                    let mut st = app_state().lock().unwrap();
                    st.printer_draft = draft;
                    st.printer_selected_id = None;
                }
                *printer_add_status().lock().unwrap() = None;
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
            if let Some(result) = aggregator_import_once().lock().unwrap().take() {
                match result {
                    Ok(msg) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.set_status(cx, msg);
                        self.sync_chrome(cx);
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_t("cancelled"));
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
                        self.set_status(cx, app_t("cancelled"));
                        self.sync_chrome(cx);
                    }
                    Err(e) => self.set_status(cx, e),
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
                let was_app_template =
                    matches!(&outcome.kind, MenuImportKind::AppTemplate { .. });
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
                                MenuImportKind::Client { title, .. }
                                | MenuImportKind::AppTemplate { title, .. } => st
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
                    Ok(m) => {
                        self.set_status(cx, m);
                        if was_app_template {
                            // Cash memo by default — tax invoice is opt-in with tax off.
                            app_state().lock().unwrap().money_lane = MoneyLane::Owner;
                            self.finish_app_template_to_lock(cx, true);
                        } else {
                            self.sync_chrome(cx);
                        }
                    }
                    Err(e) if e == "Cancelled" => {
                        self.set_status(cx, app_t("cancelled"));
                        self.sync_chrome(cx);
                    }
                    Err(e) => {
                        self.set_status(cx, e);
                        self.sync_chrome(cx);
                    }
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
                        // refresh_live_lists updates Floor/Kitchen lists and
                        // live print config; while on Settings it leaves
                        // bill/KOT/printer drafts alone (do not refill form).
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
            let mut problems = print_share::take_problems();
            if !problems.is_empty() {
                // Empty KOT route is a setup gap, not success ("not printed"
                // used to match classify OK "printed" → green face). Two
                // kitchens failing are both named (the last three, once each).
                problems.dedup();
                let start = problems.len().saturating_sub(3);
                self.show_warn(cx, problems[start..].join("; "));
            }
            if app_state().lock().unwrap().screen == Screen::Settings {
                // Only rebuild the found list when Find is running or just
                // finished — thumbs / LAN Signals used to rebuild it every wake.
                let finding = print_share::is_finding_printers();
                static WAS_FINDING: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                static SAW_CANCEL: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                let was_finding = WAS_FINDING.swap(finding, std::sync::atomic::Ordering::SeqCst);
                let cancelled = print_share::find_printers_was_cancelled();
                if !cancelled {
                    SAW_CANCEL.store(false, std::sync::atomic::Ordering::SeqCst);
                }
                // Edge-trigger Cancel — the flag stays set until the next Find,
                // and used to re-paint / set_status on every Signal forever.
                let cancel_edge = cancelled
                    && !SAW_CANCEL.swap(true, std::sync::atomic::Ordering::SeqCst);
                if finding || was_finding || cancel_edge {
                    // Progressive Find: merge whatever is published so far
                    // into the pool and repaint — do not wait for the /24 scan.
                    let sync_msg = if !cancelled && print_share::printers_find_finished() {
                        let mut st = app_state().lock().unwrap();
                        match sync_found_printers_into_pool(&mut st) {
                            Ok((0, 0)) if !finding => Some(st.t("printers_none_found")),
                            Ok((new, n)) if was_finding && !finding => Some(
                                st.t("printers_find_synced")
                                    .replace("{n}", &n.to_string())
                                    .replace("{new}", &new.to_string()),
                            ),
                            Ok(_) => None,
                            Err(e) => Some(e),
                        }
                    } else {
                        None
                    };
                    if let Some(msg) = sync_msg {
                        self.set_status(cx, msg);
                    }
                    if self.ui.view(cx, ids!(settings_panel_printers)).visible() {
                        self.paint_printers_panel(cx);
                    } else {
                        self.show_found_printers(cx);
                    }
                }
                if cancel_edge {
                    self.set_status(cx, app_t("cancelled"));
                }
                if finding
                    || was_finding
                    || cancel_edge
                    || printer_add_busy().load(std::sync::atomic::Ordering::SeqCst)
                {
                    self.paint_submit_busy_chrome(cx);
                }
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
            if let Some(result) = role_open_once().lock().unwrap().take() {
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
            // Isolated UI smoke: auto Server on DeviceSetup, then auto unlock.
            self.maybe_smoke_auto_server(cx);
            self.maybe_smoke_auto_unlock(cx);
            self.tick_smoke_script(cx);
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
        self.apply_pending_menu_pick_tab(cx);
        self.match_event(cx, event);
        if let Event::NextFrame(e) = event {
            let floor_ours = floor_area_autoscroll_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if floor_ours {
                self.floor_area_drag_autoscroll(cx);
            }
            let accordion_ours = accordion_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if accordion_ours {
                self.tick_ui_state_anims(cx);
            }
            let menu_pick_ours = menu_pick_scroll_anim_frame()
                .lock()
                .unwrap()
                .map(|frame| e.set.contains(&frame))
                .unwrap_or(false);
            if menu_pick_ours {
                self.tick_menu_pick_scroll_anim(cx);
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
            self.tick_smoke_script(cx);
        }
        // Dropdowns first (popup.rs): an outside press closes the open list,
        // a tap on a type-or-pick field opens its list. Before every pointer
        // handler below — some of them return early (a nav-tab tap), which
        // used to carry an open list onto the next screen. The press itself
        // still reaches whatever is under it.
        self.route_popup_input(cx, event);
        // A sheet over the window (Add dishes, UPI QR) owns every press: the
        // nav tabs, Floor area tabs, sidebar list, and printer rows behind it
        // must not start a drag under the dishes the finger is scrolling.
        let sheet_open = self.window_sheet_open(cx);
        if !sheet_open {
            if self.pointer_route_printer_drag(cx, event) {
                return;
            }
            if self.pointer_floor_area_drag(cx, event) {
                return;
            }
        }
        if self.pointer_navigates(cx, event, sheet_open) {
            return;
        }
        // Sideways strips (tab strips, the order ticket, nav bars) take the
        // wheel and trackpad from Makepad itself: the innermost view that can
        // move gets the delta and marks it handled. A pre-claim here
        // (claim_hstrip_nested_scroll) marked the axis handled before
        // dispatch, so each strip moved one step and then never again.
        // Done and the dimmed area of an open sheet close it before the rest
        // of the UI sees the press (see `sheet_press`).
        if self.sheet_press(cx, event) {
            return;
        }
        // Empty header (not a HEADER_CONTROLS button) is a window drag, the same
        // as the caption bar: press and move, and the desktop moves the window.
        // Not under an open sheet: there the header is dimmed area, and a
        // press on it closes the sheet instead of moving the window.
        if let Event::WindowDragQuery(dq) = event
            && !sheet_open
        {
            let header = self.ui.view(cx, ids!(header)).area().rect(cx);
            if header.size.x > 1.0 && header.contains(dq.abs) {
                let on_control = HEADER_CONTROLS.iter().any(|name| {
                    let r = self.ui.button(cx, &[LiveId::from_str(name)]).area().rect(cx);
                    r.size.x > 1.0 && r.contains(dq.abs)
                });
                if !on_control {
                    dq.response
                        .set(makepad_widgets::WindowDragQueryResponse::Caption);
                }
            }
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
        // A control in the 48 px edge strip (Done, pay, send, a sideways tab
        // strip) took this press: the control wins, the edge swipe disarms.
        self.disarm_edge_swipe_if_claimed(cx, event);
        // Hot reload re-applies DSL (visibility resets). Restore from AppState.
        if matches!(event, Event::LiveEdit | Event::ScriptReapply | Event::Startup) {
            self.sync_chrome(cx);
        }
    }
}

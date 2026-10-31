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
        if matches!(event, Event::Signal) && thumbs::take_ready() {
            self.ui.redraw(cx);
        }
        if matches!(event, Event::Signal) {
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
            }
            let fresh = std::mem::take(&mut firewall_state().lock().unwrap().fresh);
            if fresh {
                self.sync_chrome(cx);
            }
            let found = discovery_result().lock().unwrap().take();
            if let Some(found) = found {
                self.show_found_servers(cx, found);
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

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Pages and work desks pan with a mouse press-drag: a drag on copy
        // pans; mouse selection starts on a 0.5 s still hold, double / triple
        // click, Ctrl+A or right-click (Khaynjo lesson, 2026-11-13 — without
        // this a drag that started on text selected instead of panning).
        makepad_widgets::text_flow::MOUSE_HOLD_TO_SELECT
            .store(true, std::sync::atomic::Ordering::Relaxed);
        app_state().lock().unwrap().live_timer = cx.start_interval(LIVE_REFRESH_SECS);
        self.sync_chrome(cx);
    }

    fn handle_timer(&mut self, cx: &mut Cx, e: &TimerEvent) {
        // Sell / Sales / Credit stay live on extra tills: catalogue and
        // day totals from the office. Host Settings still refreshes the
        // paired-device list. (A device's write also refreshes at once —
        // `BOOKS_CHANGED` / `/wait`.)
        let (live, settings) = {
            let st = app_state().lock().unwrap();
            let ours = st.live_timer.is_timer(e).is_some() && st.session.is_some();
            (
                ours
                    && matches!(st.screen, Screen::Pos | Screen::Sales | Screen::Credit)
                    && st.device_role == DeviceRole::Client,
                ours && st.screen == Screen::Settings && st.device_role.is_host(),
            )
        };
        if live {
            app_state().lock().unwrap().pull_from_server();
            self.ui.redraw(cx);
            return;
        }
        if settings {
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
        if e.key_code == KeyCode::Forward
            || (e.key_code == KeyCode::ArrowRight && e.modifiers.alt)
        {
            if nav_gesture_ok() {
                let _ = self.navigate_forward(cx);
            }
            return;
        }

        // Form FieldInput Tab / Shift+Tab (pair, lock, onboard, booking, settings).
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
        self.handle_thumb_decodes(cx, actions);
        self.refresh_form_tab_anchor(cx);
        let _ = self.handle_form_field_returned(cx, actions);
        self.handle_selection_actions(cx, actions);
        self.tab_chevron_clicks(cx, actions);
        self.handle_actions_nav_books_lock(cx, actions);
        self.handle_actions_parties(cx, actions);
        self.handle_actions_desk_menu(cx, actions);
        self.handle_actions_settings_client(cx, actions);
        self.handle_actions_ops(cx, actions);
        self.handle_actions_reports_backup(cx, actions);
    }
}

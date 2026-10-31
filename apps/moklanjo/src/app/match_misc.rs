impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        app_state().lock().unwrap().live_timer = cx.start_interval(LIVE_REFRESH_SECS);
        self.sync_chrome(cx);
    }

    fn handle_timer(&mut self, cx: &mut Cx, e: &TimerEvent) {
        // Settings shows which devices are connected right now.
        let settings = {
            let st = app_state().lock().unwrap();
            st.live_timer.is_timer(e).is_some()
                && st.session.is_some()
                && st.screen == Screen::Settings
                && st.device_role.is_host()
        };
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
        self.handle_actions_nav_books_lock(cx, actions);
        self.handle_actions_parties(cx, actions);
        self.handle_actions_desk_menu(cx, actions);
        self.handle_actions_settings_client(cx, actions);
        self.handle_actions_ops(cx, actions);
        self.handle_actions_reports_backup(cx, actions);
    }
}

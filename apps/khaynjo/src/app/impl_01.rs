impl App {
fn set_status(&mut self, cx: &mut Cx, msg: impl Into<String>) {
        let msg = msg.into();
        let kind = snack::classify(&msg);
        self.set_status_line_only(cx, &msg);
        if !msg.trim().is_empty() {
            let unlocked = app_state().lock().unwrap().session.is_some();
            if unlocked || matches!(kind, SnackKind::Error | SnackKind::Warn) {
                self.show_snack(cx, kind, msg);
            }
        }
    }

    fn set_status_line_only(&mut self, cx: &mut Cx, msg: &str) {
        {
            let mut st = app_state().lock().unwrap();
            st.status = msg.to_string();
            // Sell has no footer strip — put pay/cart errors on the receipt line
            // so Split/Credit failures are visible on the counter screen.
            if matches!(
                st.screen,
                Screen::Pos | Screen::Credit | Screen::Sales | Screen::Menu
            ) {
                st.last_receipt = msg.to_string();
            }
        }
        self.ui
            .widget(cx, ids!(pin_status))
            .set_text(cx, &plain_display_text(msg));
        self.ui
            .widget(cx, ids!(device_setup_status))
            .set_text(cx, &plain_display_text(msg));
        self.ui
            .widget(cx, ids!(receipt_label))
            .set_text(cx, &plain_display_text(&ticket_status_line(msg)));
    }

    /// Ephemeral chrome snack (info / success / warn / error). Undo snack stays separate.
    fn show_snack(&mut self, cx: &mut Cx, kind: SnackKind, msg: impl Into<String>) {
        let msg = msg.into();
        let msg = msg.trim().to_string();
        if msg.is_empty() {
            return;
        }
        {
            let mut st = app_state().lock().unwrap();
            if !st.snack_text.trim().is_empty() && kind.priority() < st.snack_kind.priority() {
                return;
            }
            if !st.snack_timer.is_empty() {
                cx.stop_timer(st.snack_timer);
            }
            let secs = kind.duration_secs(st.snack_secs);
            st.snack_text = msg;
            st.snack_kind = kind;
            st.snack_timer = cx.start_timeout(secs);
        }
        self.paint_app_snackbar(cx);
    }

    fn show_error(&mut self, cx: &mut Cx, msg: impl Into<String>) {
        let msg = msg.into();
        self.set_status_line_only(cx, &msg);
        self.show_snack(cx, SnackKind::Error, msg);
    }

    fn show_warn(&mut self, cx: &mut Cx, msg: impl Into<String>) {
        let msg = msg.into();
        self.set_status_line_only(cx, &msg);
        self.show_snack(cx, SnackKind::Warn, msg);
    }

    fn show_ok(&mut self, cx: &mut Cx, msg: impl Into<String>) {
        let msg = msg.into();
        self.set_status_line_only(cx, &msg);
        self.show_snack(cx, SnackKind::Success, msg);
    }

    fn clear_snack(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            if !st.snack_timer.is_empty() {
                cx.stop_timer(st.snack_timer);
                st.snack_timer = Timer::empty();
            }
            st.snack_text.clear();
            st.snack_kind = SnackKind::Info;
        }
        self.paint_app_snackbar(cx);
    }

    fn paint_app_snackbar(&mut self, cx: &mut Cx) {
        let (visible, text, kind) = {
            let mut st = app_state().lock().unwrap();
            // Never show a blank bar (whitespace-only / cleared mid-frame).
            if st.snack_text.trim().is_empty() {
                if !st.snack_text.is_empty() {
                    st.snack_text.clear();
                }
                (false, String::new(), SnackKind::Info)
            } else {
                (true, st.snack_text.clone(), st.snack_kind)
            }
        };
        self.ui
            .view(cx, ids!(app_snackbar))
            .set_visible(cx, visible);
        let faces = [
            (ids!(snack_face_info), ids!(snack_label_info), SnackKind::Info),
            (ids!(snack_face_ok), ids!(snack_label_ok), SnackKind::Success),
            (ids!(snack_face_warn), ids!(snack_label_warn), SnackKind::Warn),
            (ids!(snack_face_err), ids!(snack_label_err), SnackKind::Error),
        ];
        for (face, label, k) in faces {
            let on = visible && kind == k;
            self.ui.view(cx, face).set_visible(cx, on);
            if on {
                self.ui
                    .widget(cx, label)
                    .set_text(cx, &plain_display_text(&text));
            } else {
                self.ui.widget(cx, label).set_text(cx, "");
            }
        }
    }

    /// Open SQLite using Makepad's platform data dir (Android getFilesDir).
    /// `dirs::data_dir` is often None on Android (no $HOME), which left
    /// `store = None` so Owner PIN OK could never unlock.
    fn ensure_store(&mut self, cx: &mut Cx) {
        {
            let st = app_state().lock().unwrap();
            if st.store.is_some() {
                return;
            }
        }
        self.resolve_data_base(cx);
        let opened = if let Some(dir) = cx.get_data_dir() {
            let base = PathBuf::from(dir);
            app_state().lock().unwrap().data_base = Some(base.clone());
            print_share::use_device_dir(&base);
            diag::init(&base, store::APP_ID, env!("CARGO_PKG_VERSION"));
            local_servers::open_active_server(&base)
        } else {
            if let Ok(dir) = store::data_dir() {
                diag::init(&dir, store::APP_ID, env!("CARGO_PKG_VERSION"));
            }
            Store::open_default()
        };
        let mut st = app_state().lock().unwrap();
        let _ = st.refresh_local_servers();
        match opened {
            Ok(store) => {
                st.status = String::new();
                st.store = Some(store);
                let _ = st.reload_all();
                // Shelf keys → custom files while the pack is still on disk
                // (desktop Server). Clients then fetch over LAN; Android has
                // no assets/items pack of its own.
                if st.device_role == DeviceRole::Server
                    && let Some(store) = st.store.as_ref()
                {
                    let _ = store.materialize_shelf_photos();
                }
                if let Some(store) = st.store.as_mut() {
                    let _ = store.set_active_session_staff_id(None);
                }
                st.session = None;
                st.pin_buffer.clear();
                st.route_after_stored_role();
                let path = st
                    .store
                    .as_ref()
                    .map(|s| s.db_path().display().to_string())
                    .unwrap_or_default();
                log!("Kumkaj store ready at {path}");
                diag::info(diag::Cat::Store, "store_ready", "store open ok");
            }
            Err(e) => {
                if e.contains("No local server") {
                    st.must_confirm_device_role = true;
                    st.apply_platform_language_preview();
                    st.screen = Screen::DeviceSetup;
                    log!("Kumkaj: no local server yet — DeviceSetup");
                    diag::info(diag::Cat::Store, "no_local_server", "DeviceSetup");
                } else {
                    st.status = format!("Storage error: {e}");
                    log!("Kumkaj store open failed: {e}");
                    diag::error(diag::Cat::Store, "store_open_failed", &e);
                }
            }
        }
    }
}

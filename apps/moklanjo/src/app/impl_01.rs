impl App {
fn set_status(&mut self, cx: &mut Cx, msg: impl Into<String>) {
        // Never `set_status(cx, app_state().lock().unwrap().t(...))` — temporary
        // MutexGuard spans this call and deadlocks (use `app_t` / a `let` first).
        let msg = msg.into();
        let kind = snack::classify(&msg);
        self.set_status_line_only(cx, &msg);
        if !msg.trim().is_empty() {
            // Locked / pairing: still show Error, Warn, and Success. Only
            // suppress idle Info noise while there is no session.
            let unlocked = app_state().lock().unwrap().session.is_some();
            if unlocked || !matches!(kind, SnackKind::Info) {
                self.show_snack(cx, kind, msg);
            }
        }
    }

    /// Status with a known outcome: the face follows what happened, not
    /// words in the message ("Failed delivery · 2" recorded is a success).
    fn set_status_as(&mut self, cx: &mut Cx, result: Result<String, String>) {
        let (kind, msg) = match result {
            // "· 2 refused" — some went, some did not.
            Ok(msg) if msg.lines().next().is_some_and(|l| l.contains(" refused")) => {
                (SnackKind::Warn, msg)
            }
            Ok(msg) => (SnackKind::Success, msg),
            Err(e) => (SnackKind::Error, e),
        };
        self.set_status_line_only(cx, &msg);
        if !msg.trim().is_empty() {
            self.show_snack(cx, kind, msg);
        }
    }

    fn set_status_line_only(&mut self, cx: &mut Cx, msg: &str) {
        {
            let mut st = app_state().lock().unwrap();
            st.status = msg.to_string();
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
            .set_text(cx, &plain_display_text(msg));
    }

    fn show_snack(&mut self, cx: &mut Cx, kind: SnackKind, msg: impl Into<String>) {
        let msg = msg.into();
        if msg.trim().is_empty() {
            return;
        }
        {
            let mut st = app_state().lock().unwrap();
            if !st.snack_text.is_empty() && kind.priority() < st.snack_kind.priority() {
                return;
            }
            if !st.snack_timer.is_empty() {
                cx.stop_timer(st.snack_timer);
            }
            st.snack_text = msg;
            st.snack_kind = kind;
            let secs = kind.duration_secs(st.snack_secs);
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
            let st = app_state().lock().unwrap();
            (
                !st.snack_text.is_empty(),
                st.snack_text.clone(),
                st.snack_kind,
            )
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
                // Boot may have opened books without diagnostics; don't leave
                // Settings painting a fake Info chip forever.
                if let Some(base) = st.data_base.clone() {
                    drop(st);
                    diag::ensure_ready(&base, store::APP_ID, env!("CARGO_PKG_VERSION"));
                }
                return;
            }
        }
        self.resolve_data_base(cx);
        let opened = if let Some(dir) = cx.get_data_dir() {
            let base = PathBuf::from(dir);
            app_state().lock().unwrap().data_base = Some(base.clone());
            print_share::use_device_dir(&base);
            diag::ensure_ready(&base, store::APP_ID, env!("CARGO_PKG_VERSION"));
            local_servers::open_active_server(&base)
        } else {
            if let Ok(dir) = store::data_dir() {
                diag::ensure_ready(&dir, store::APP_ID, env!("CARGO_PKG_VERSION"));
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
                if let Some(store) = st.store.as_mut() {
                    let _ = store.set_active_session_staff_id(None);
                }
                st.session = None;
                st.clear_team_edit_selection();
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

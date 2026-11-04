impl App {
/// Active staff by exact name, else first substring match (Team pick / copy).
fn find_active_staff_by_name(st: &AppState, needle: &str) -> Option<StaffMember> {
        let n = needle.trim().to_lowercase();
        if n.is_empty() {
            return None;
        }
        st.staff
            .iter()
            .find(|s| s.active && s.name.trim().to_lowercase() == n)
            .or_else(|| {
                st.staff
                    .iter()
                    .find(|s| s.active && s.name.to_lowercase().contains(&n))
            })
            .cloned()
    }

fn nav_drag_autoscroll(&mut self, cx: &mut Cx) {
        let (rail, last) = {
            let guard = nav_drag().lock().unwrap();
            match guard.as_ref() {
                Some(d) if d.active => (d.rail, d.last),
                _ => return, // drag over: let the frame loop die
            }
        };
        // Keep the loop alive for the whole drag.
        *nav_autoscroll_frame().lock().unwrap() = Some(cx.new_next_frame());
        let view = if rail {
            self.ui.view(cx, ids!(rail_scroll))
        } else {
            self.ui.view(cx, ids!(bottom_nav_scroll))
        };
        let rect = view.area().rect(cx);
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        const ZONE: f64 = 56.0;
        let (near_start, near_end) = if rail {
            (
                (ZONE - (last.y - rect.pos.y)).clamp(0.0, ZONE),
                (ZONE - (rect.pos.y + rect.size.y - last.y)).clamp(0.0, ZONE),
            )
        } else {
            (
                (ZONE - (last.x - rect.pos.x)).clamp(0.0, ZONE),
                (ZONE - (rect.pos.x + rect.size.x - last.x)).clamp(0.0, ZONE),
            )
        };
        let step = |depth: f64| {
            if depth > 0.0 {
                2.0 + depth / ZONE * 10.0
            } else {
                0.0
            }
        };
        let delta = step(near_end) - step(near_start);
        if delta == 0.0 {
            return;
        }
        let cur = view.get_scroll_pos();
        let total = view.get_scroll_view_total();
        let visible = view.get_scroll_view_visible();
        let mut next = cur;
        if rail {
            next.y = (cur.y + delta).clamp(0.0, (total.y - visible.y).max(0.0));
        } else {
            next.x = (cur.x + delta).clamp(0.0, (total.x - visible.x).max(0.0));
        }
        if next.x == cur.x && next.y == cur.y {
            return; // already at the scroll end
        }
        view.set_scroll_pos(cx, next);
        let mut repaint = false;
        {
            let mut guard = nav_drag().lock().unwrap();
            if let Some(d) = guard.as_mut() {
                let hover = self.nav_drop_slot(cx, d, d.last);
                if hover != d.hover {
                    d.hover = hover;
                    repaint = true;
                }
            }
        }
        if repaint {
            self.repaint_nav_tabs(cx);
        }
    }

    /// Mouse Back/Forward, left-edge swipe → back, right-edge swipe → forward.
    /// TouchUpdate on phones; primary MouseDown/Up edge drag on desktop.
    /// Peer chrome and drill-downs both push visit history, so Back/Forward
    /// move between screens (empty stack is still a no-op, after dismissing
    /// soft keyboard / in-screen layers).
    fn pointer_navigates(&mut self, cx: &mut Cx, event: &Event) -> bool {
        if self.pointer_rail_resize(cx, event) {
            return true;
        }
        let navigated = match event {
            Event::MouseDown(e) if e.button.is_back() => {
                *mouse_swipe_origin().lock().unwrap() = None;
                self.navigate_back(cx)
            }
            Event::MouseDown(e) if e.button.is_forward() => {
                *mouse_swipe_origin().lock().unwrap() = None;
                self.navigate_forward(cx)
            }
            Event::MouseDown(e) if e.button.is_primary() => {
                // While the rail is showing, left edge is the rail — don't
                // arm swipe-back there. Right-edge forward still works.
                let rail_owns_left = {
                    let st = app_state().lock().unwrap();
                    let unlocked = !matches!(
                        st.screen,
                        Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                    );
                    unlocked && self.window_inner_width(cx) >= store::TABLET_SHELL_BREAKPOINT_CSS
                };
                let width = self.window_inner_width(cx);
                let mut origin = mouse_swipe_origin().lock().unwrap();
                if e.abs.x < 48.0 && !rail_owns_left {
                    *origin = Some((e.abs, SwipeNav::Back));
                } else if e.abs.x > width - 48.0 {
                    *origin = Some((e.abs, SwipeNav::Forward));
                } else {
                    *origin = None;
                }
                false
            }
            Event::MouseUp(e) if e.button.is_primary() => {
                let start_edge = mouse_swipe_origin().lock().unwrap().take();
                match start_edge {
                    Some((start, edge)) => self.edge_swipe_nav(cx, start, e.abs, edge),
                    None => false,
                }
            }
            Event::TouchUpdate(e) => {
                let width = self.window_inner_width(cx);
                let mut origin = swipe_origin().lock().unwrap();
                let mut nav: Option<SwipeNav> = None;
                let mut start_end: Option<(DVec2, DVec2)> = None;
                for touch in &e.touches {
                    match touch.state {
                        TouchState::Start => {
                            if touch.abs.x < 48.0 {
                                *origin = Some((touch.uid, touch.abs, SwipeNav::Back));
                            } else if touch.abs.x > width - 48.0 {
                                *origin = Some((touch.uid, touch.abs, SwipeNav::Forward));
                            }
                        }
                        TouchState::Stop => {
                            if let Some((uid, start, edge)) = *origin
                                && uid == touch.uid
                            {
                                start_end = Some((start, touch.abs));
                                nav = Some(edge);
                                *origin = None;
                            }
                        }
                        _ => {}
                    }
                }
                drop(origin);
                match (nav, start_end) {
                    (Some(edge), Some((start, end))) => {
                        self.edge_swipe_nav(cx, start, end, edge)
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        if navigated {
            // Claim the gesture slot so a paired XF86Back key doesn't step twice.
            let _ = nav_gesture_ok();
            true
        } else {
            false
        }
    }

    fn resolve_data_base(&mut self, cx: &mut Cx) {
        let mut st = app_state().lock().unwrap();
        if st.data_base.is_none() {
            if let Some(dir) = cx.get_data_dir() {
                print_share::use_device_dir(std::path::Path::new(&dir));
                st.data_base = Some(PathBuf::from(dir));
            } else if let Ok(dir) = store::data_dir() {
                print_share::use_device_dir(&dir);
                st.data_base = Some(dir);
            }
        }
    }

    fn persist_setup_language(st: &mut AppState) {
        let follow = st.language_follow_platform;
        let code = st.locale.language.clone();
        if let Some(store) = st.store.as_ref() {
            let persisted = if follow {
                store.set_platform_language()
            } else {
                store.set_user_language(&code)
            };
            if let Ok(loc) = persisted {
                st.locale = loc;
            }
            st.save_remembered_language();
        }
    }

    /// Server Mode: create the first local server, or show create/pick when several exist.
    fn begin_server_mode(&mut self, cx: &mut Cx) {
        if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        self.resolve_data_base(cx);
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.refresh_local_servers()?;
            if st.local_servers.is_empty() {
                drop(st);
                self.activate_new_server(cx, "")?;
            } else {
                st.picking_server = true;
                st.status = st.t("choose_server");
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn activate_new_server(&mut self, cx: &mut Cx, name: &str) -> Result<(), String> {
        if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        self.resolve_data_base(cx);
        let base = {
            let st = app_state().lock().unwrap();
            st.data_base
                .clone()
                .ok_or_else(|| "No data directory".to_string())?
        };
        server_new_job(base, name.to_string());
        self.paint_role_open_busy_chrome(cx);
        Ok(())
    }

    fn activate_existing_server(&mut self, cx: &mut Cx, server_id: &str) -> Result<(), String> {
        if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        self.resolve_data_base(cx);
        let (base, switching_in) = {
            let st = app_state().lock().unwrap();
            (
                st.data_base
                    .clone()
                    .ok_or_else(|| "No data directory".to_string())?,
                st.device_role != DeviceRole::Server,
            )
        };
        server_existing_job(base, server_id.to_string(), switching_in);
        self.paint_role_open_busy_chrome(cx);
        Ok(())
    }

    fn finish_server_new_opened(&mut self, cx: &mut Cx, store: Store) {
        {
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.session = None;
            st.onboarding_complete = false;
            st.owner_setup_complete = false;
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            // New server always reaches Owner PIN via App Template offer when shipped.
            if Self::should_offer_app_template(&st) {
                self.enter_app_template_offer(&mut st, true);
            } else {
                self.finish_as_server(&mut st, true);
            }
        }
        self.paint_role_open_busy_chrome(cx);
        self.sync_chrome(cx);
    }

    fn finish_server_existing_opened(&mut self, cx: &mut Cx, store: Store, switching_in: bool) {
        {
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            // Unfinished new shop (killed before PIN) re-offers until applied/skipped.
            if Self::should_offer_app_template(&st) {
                self.enter_app_template_offer(&mut st, switching_in);
            } else {
                self.finish_as_server(&mut st, switching_in);
            }
        }
        self.paint_role_open_busy_chrome(cx);
        self.sync_chrome(cx);
    }

    /// Brand-new Server before Owner PIN, and the Rajdhani menu template ships.
    fn should_offer_app_template(st: &AppState) -> bool {
        !st.owner_setup_complete
            && menu_templates::find("rajdhani").is_some()
            && !st
                .store
                .as_ref()
                .and_then(|s| s.rajdhani_app_template_done().ok())
                .unwrap_or(false)
    }

    /// DeviceSetup card before Lock — same Server attach as `finish_as_server`, no PIN yet.
    fn enter_app_template_offer(&mut self, st: &mut AppState, switching_in: bool) {
        st.device_role = DeviceRole::Server;
        st.must_confirm_device_role = false;
        st.picking_server = false;
        st.awaiting_app_template = true;
        st.pin_buffer.clear();
        st.nav_back.clear();
        st.nav_forward.clear();
        if let Some(store) = st.store.as_ref() {
            let _ = store.set_device_role(DeviceRole::Server.as_str());
            if switching_in {
                let _ = store.set_extra_tills_allowed(false);
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

    /// Mark the one-shot offer done and continue to Owner PIN.
    fn finish_app_template_to_lock(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.awaiting_app_template = false;
            let (staff, fiscal) = if let Some(store) = st.store.as_ref() {
                let _ = store.set_rajdhani_app_template_done();
                // After Apply: Owner may be "Asif" and tax rates retired — refresh.
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

    /// Idle: Submit + Skip. Busy: Submit replaced by status + Cancel.
    fn paint_app_template_chrome(&mut self, cx: &mut Cx, visible: bool) {
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
        let (heading, blurb, apply, skip, cancel, progress) = {
            let st = app_state().lock().unwrap();
            let title = menu_templates::find("rajdhani")
                .map(|t| t.title.clone())
                .unwrap_or_else(|| "Rajdhani Menu Template".into());
            let progress = if app_busy {
                let (done, total) = *menu_import_progress().lock().unwrap();
                if total > 0 {
                    st.t("template_import_progress")
                        .replace("{d}", &done.to_string())
                        .replace("{t}", &total.to_string())
                } else {
                    st.t("app_template_importing").replace("{title}", &title)
                }
            } else {
                String::new()
            };
            let blurb = if app_busy {
                st.t("app_template_importing").replace("{title}", &title)
            } else {
                st.t("app_template_blurb")
            };
            (
                st.t("app_template_heading"),
                blurb,
                st.t("app_template_apply"),
                st.t("app_template_skip"),
                st.t("cancel"),
                progress,
            )
        };
        self.ui
            .widget(cx, ids!(app_template_heading))
            .set_text(cx, &plain_display_text(&heading));
        self.ui
            .widget(cx, ids!(app_template_blurb))
            .set_text(cx, &plain_display_text(&blurb));
        self.ui
            .button(cx, ids!(app_template_apply))
            .set_text(cx, &apply);
        self.ui
            .button(cx, ids!(app_template_apply))
            .set_visible(cx, !app_busy);
        self.ui
            .view(cx, ids!(app_template_busy))
            .set_visible(cx, app_busy);
        if app_busy {
            self.ui
                .button(cx, ids!(app_template_status))
                .set_text(cx, &progress);
            self.ui
                .button(cx, ids!(app_template_cancel))
                .set_text(cx, &cancel);
        }
        self.ui
            .button(cx, ids!(app_template_skip))
            .set_text(cx, &skip);
        self.ui
            .button(cx, ids!(app_template_skip))
            .set_visible(cx, !app_busy);
    }

    /// Wipe a local server from this device. Owner PIN of **that** shop
    /// required (vision §17) — not staff, not an unlocked non-owner session.
    fn delete_existing_server(&mut self, cx: &mut Cx, server_id: &str) -> Result<(), String> {
        self.resolve_data_base(cx);
        let pin = self.ui.text_input(cx, ids!(server_remove_pin)).text();
        let base = {
            let st = app_state().lock().unwrap();
            st.data_base
                .clone()
                .ok_or_else(|| "No data directory".to_string())?
        };
        let target = local_servers::server_db_path(&base, server_id);
        // Prove Owner while the books file is still open / unlockable.
        {
            let st = app_state().lock().unwrap();
            if let Some(actor) = st.session.as_ref() {
                if !actor.is_owner() {
                    return Err(st.t("only_owner_remove_server"));
                }
            }
            let open_is_target = st
                .store
                .as_ref()
                .is_some_and(|s| s.db_path() == target.as_path());
            if open_is_target {
                let store = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "Storage not ready".to_string())?;
                if store.owner_setup_complete().unwrap_or(false) {
                    let pin = pin.trim();
                    if pin.is_empty() {
                        return Err(st.t("owner_pin_remove_server"));
                    }
                    if store.authenticate_owner_pin(pin)?.is_none() {
                        return Err(st.t("only_owner_remove_server"));
                    }
                }
            } else {
                drop(st);
                local_servers::assert_owner_may_delete(&base, server_id, &pin)?;
            }
        }
        let clearing = {
            let st = app_state().lock().unwrap();
            st.store
                .as_ref()
                .is_some_and(|s| s.db_path() == target.as_path())
        };
        if clearing {
            net::stop_server();
            let mut st = app_state().lock().unwrap();
            st.store = None;
            st.session = None;
            st.onboarding_complete = false;
            st.owner_setup_complete = false;
        }
        local_servers::delete_local_server(&base, server_id)?;
        self.ui
            .text_input(cx, ids!(server_remove_pin))
            .set_text(cx, "");
        {
            let mut st = app_state().lock().unwrap();
            let _ = st.refresh_local_servers();
            st.picking_server = true;
            st.status = st.t("choose_server");
        }
        self.sync_chrome(cx);
        Ok(())
    }

    fn finish_as_server(&mut self, st: &mut AppState, switching_in: bool) {
        st.device_role = DeviceRole::Server;
        st.must_confirm_device_role = false;
        st.picking_server = false;
        st.pin_buffer.clear();
        st.nav_back.clear();
        st.nav_forward.clear();
        // Parent of Lock so mouse/Android Back returns to Server/Client pick.
        st.nav_back.push(Screen::DeviceSetup);
        if let Some(store) = st.store.as_ref() {
            let _ = store.set_device_role(DeviceRole::Server.as_str());
            if switching_in {
                let _ = store.set_extra_tills_allowed(false);
                st.extra_tills_allowed = false;
                net::stop_server();
            } else if st.extra_tills_allowed {
                let path = store.db_path().to_path_buf();
                if net::ensure_server(path).is_ok() {
                    firewall_job(|| Ok(net::firewall_allows_phones()));
                }
            }
        }
        // First Server lock → Owner; else remembered chip.
        st.apply_lock_role_selection();
        st.screen = Screen::Lock;
        st.status = if !st.owner_setup_complete {
            locale::t(&st.locale.language, "status_server_set_pin")
        } else if st.extra_tills_allowed {
            locale::t(&st.locale.language, "status_server_tills")
        } else {
            locale::t(&st.locale.language, "status_server_unlock")
        };
    }

    /// Start Client Mode without blocking the UI (first open migrates the
    /// client cache — can take a long time on Android). Busy chrome + Cancel
    /// replace Server/Client until the worker finishes.
    fn begin_client_mode(&mut self, cx: &mut Cx) {
        if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        self.resolve_data_base(cx);
        let base = {
            let st = app_state().lock().unwrap();
            match st.data_base.clone() {
                Some(b) => b,
                None => {
                    self.set_status(cx, "No data directory");
                    return;
                }
            }
        };
        client_mode_job(base);
        self.paint_role_open_busy_chrome(cx);
    }

    /// Apply a Store opened on the client-mode worker (Signal handler).
    fn finish_client_mode_opened(&mut self, cx: &mut Cx, store: Store) {
        {
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            st.device_role = DeviceRole::Client;
            st.must_confirm_device_role = false;
            st.owner_setup_complete = true;
            st.onboarding_complete = true;
            st.extra_tills_allowed = false;
            st.pin_buffer.clear();
            st.nav_back.clear();
            st.nav_forward.clear();
            st.client_paired = store_client_paired(st.store.as_ref());
            st.apply_lock_role_selection();
            // Always ClientPair — Find / tap a shop / Pair. Never skip to Lock
            // and trap the till on a previous pairing (multi-server Wi‑Fi).
            st.nav_back.push(Screen::DeviceSetup);
            st.screen = Screen::ClientPair;
            st.status = if st.client_paired {
                locale::t(&st.locale.language, "status_client_unlock")
            } else {
                "Client Mode — find the Server, tap the shop you want, then Pair".into()
            };
        }
        self.paint_role_open_busy_chrome(cx);
        self.sync_chrome(cx);
    }

    fn paint_role_open_busy_chrome(&mut self, cx: &mut Cx) {
        let busy = role_open_busy().load(std::sync::atomic::Ordering::SeqCst);
        let cancel = app_t("cancel");
        let status = books_work_status()
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| app_t("opening_client"));
        let (show_pick, awaiting_tmpl) = {
            let st = app_state().lock().unwrap();
            (
                !busy && st.picking_server && !st.awaiting_app_template,
                st.awaiting_app_template,
            )
        };
        // Hide Server/Client while creating/opening or while App Template is offered.
        self.ui
            .view(cx, ids!(device_role_actions))
            .set_visible(cx, !busy && !show_pick && !awaiting_tmpl);
        self.ui
            .view(cx, ids!(server_pick_section))
            .set_visible(cx, show_pick);
        self.ui
            .view(cx, ids!(app_template_section))
            .set_visible(cx, awaiting_tmpl && !busy);
        self.ui
            .widget(cx, ids!(device_role_busy_label))
            .set_text(cx, &plain_display_text(&status));
        self.ui
            .widget(cx, ids!(device_role_busy_label))
            .set_visible(cx, busy);
        self.ui
            .button(cx, ids!(device_role_cancel))
            .set_text(cx, &cancel);
        self.ui
            .button(cx, ids!(device_role_cancel))
            .set_visible(cx, busy);
    }

    fn set_extra_tills(&mut self, cx: &mut Cx, allowed: bool) {
        self.ensure_store(cx);
        let err = {
            let mut st = app_state().lock().unwrap();
            if let Err(e) = st.require_server_role("Change extra tills") {
                Some(e)
            } else if st.store.is_none() {
                Some("Storage not ready".into())
            } else if let Err(e) = st
                .store
                .as_ref()
                .unwrap()
                .set_extra_tills_allowed(allowed)
            {
                Some(e)
            } else {
                st.extra_tills_allowed = allowed;
                if allowed {
                    let store = st.store.as_ref().unwrap();
                    let code = store.ensure_server_pairing_code().unwrap_or_default();
                    let path = store.db_path().to_path_buf();
                    if let Err(e) = net::ensure_server(path) {
                        st.status = format!("Extra tills on, LAN listen failed: {e}");
                    } else {
                        st.status = format!("Extra tills on · pairing code {code}");
                        firewall_job(|| Ok(net::firewall_allows_phones()));
                    }
                } else {
                    net::stop_server();
                    st.status = "Extra tills off — this device still holds the shop books".into();
                }
                None
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Copy digits from the lock PIN field into `pin_buffer` (max 12).
    fn sync_pin_buffer_from_field(&mut self, cx: &mut Cx) {
        let text = self.ui.text_input(cx, ids!(lock_pin_input)).text();
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).take(12).collect();
        let mut st = app_state().lock().unwrap();
        st.pin_buffer = digits;
    }

    /// Seconds left on the wrong-PIN block, or None when unlock may proceed.
    fn unlock_block_remaining(&self) -> Option<u64> {
        let mut st = app_state().lock().unwrap();
        let until = st.unlock_blocked_until?;
        match until.checked_duration_since(std::time::Instant::now()) {
            Some(left) => Some(left.as_secs() + 1),
            None => {
                st.unlock_blocked_until = None;
                None
            }
        }
    }

    /// What the device's LAN threads brought back: pairing, sign-in, the
    /// Server's answers to writes, fresh books.
    fn take_lan_answers(&mut self, cx: &mut Cx) {
        let once = lan_once().lock().unwrap().take();
        match once {
            Some(LanOnce::SignIn { staff_id, pin, reply }) => self.finish_lan_sign_in(cx, staff_id, pin, reply),
            Some(LanOnce::Paired(reply)) => match reply {
                Err(e) => {
                    let msg = e.to_string();
                    app_state().lock().unwrap().pair_note = msg.clone();
                    self.set_status(cx, msg);
                    self.sync_chrome(cx);
                }
                Ok((link, snap)) => {
                    // apply_snapshot is SQLite on a busy shop — never on the UI
                    // thread (Android freezes for seconds; DEV.md bug 7b).
                    let db = {
                        let st = app_state().lock().unwrap();
                        st.store.as_ref().map(|s| s.db_path().to_path_buf())
                    };
                    if let Some(db) = db {
                        let finishing = app_t("pair_finishing");
                        {
                            let mut st = app_state().lock().unwrap();
                            st.pair_note = finishing.clone();
                        }
                        self.set_status(cx, finishing);
                        self.sync_chrome(cx);
                        let _ = std::thread::Builder::new()
                            .name("khaynjo-pair-apply".into())
                            .spawn(move || {
                                let result = (|| -> Result<String, String> {
                                    let store = Store::open_path(&db)?;
                                    store.set_client_device_pairing(&link.base, &link.device)?;
                                    // Kept to find the Server again if its address changes.
                                    store.set_paired_server_id(&link.server_id)?;
                                    phone::forget_revision();
                                    net::apply_snapshot(&store, &snap)?;
                                    // The shop's name as it saved it; an unnamed shop is "the shop".
                                    let name = [snap.venue_title.trim(), snap.org_name.trim()]
                                        .into_iter()
                                        .find(|n| !n.is_empty())
                                        .unwrap_or("the shop")
                                        .to_string();
                                    Ok(name)
                                })();
                                *lan_once().lock().unwrap() = Some(LanOnce::PairApplied(result));
                                makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                            });
                    } else {
                        let msg = "No store".to_string();
                        app_state().lock().unwrap().pair_note = msg.clone();
                        self.set_status(cx, msg);
                        self.sync_chrome(cx);
                    }
                }
            },
            Some(LanOnce::PairApplied(result)) => {
                let outcome = (|| -> Result<String, String> {
                    let name = result?;
                    let mut st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    phone::start(store.db_path().to_path_buf());
                    st.client_paired = true;
                    st.reload_all()?;
                    // The person signs in with their shop PIN: this device's
                    // own sign-in is not one the Server knows. Drop any Who
                    // pick left from a previous shop's roster.
                    st.session = None;
                    st.lock_staff_id = None;
                    st.lock_as_owner = false;
                    st.screen = Screen::Lock;
                    Ok(format!("Paired with {name} — sign in with your shop PIN"))
                })();
                match outcome {
                    Ok(msg) => {
                        app_state().lock().unwrap().pair_note.clear();
                        self.set_status(cx, msg);
                    }
                    Err(e) => {
                        app_state().lock().unwrap().pair_note = e.clone();
                        self.set_status(cx, e);
                    }
                }
                self.sync_chrome(cx);
            }
            None => {}
        }
        // The shop forgot this device (Forget on the Server): back to pairing,
        // saying why. Waiting orders stay on the device for after.
        let forgotten = {
            let mut st = app_state().lock().unwrap();
            if st.device_role == DeviceRole::Client && st.client_paired && phone::link_state().needs_pairing {
                st.client_paired = false;
                if let Some(store) = st.store.as_ref() {
                    let _ = store.clear_client_pairing();
                }
                st.session = None;
                st.pair_note = st.t("device_forgotten");
                st.screen = Screen::ClientPair;
                true
            } else {
                false
            }
        };
        if forgotten {
            self.sync_chrome(cx);
        }
        let answers = phone::take_done();
        let synced = phone::take_synced();
        if answers.is_empty() && !synced {
            if phone::link_state() != app_state().lock().unwrap().lan_shown {
                app_state().lock().unwrap().lan_shown = phone::link_state();
                self.sync_chrome(cx);
            }
            return;
        }
        let mut snack = None;
        let mut said = None;
        let mut receipt = None;
        let mut open_table: Option<String> = None;
        {
            let mut st = app_state().lock().unwrap();
            for answer in answers {
                match answer.reply {
                    Err(e) => {
                        // The Server's own words, said about what it was.
                        said = Some(if e.is_offline() {
                            format!("{} — {e}", answer.label)
                        } else {
                            format!("{}: {e}", answer.label)
                        });
                        if e.code() == "pair_again" {
                            st.client_paired = false;
                        }
                    }
                    Ok(reply) => {
                        said = Some(answer.done.clone());
                        match answer.follow {
                            phone::Follow::Refresh => {}
                            phone::Follow::BillText => {
                                receipt = reply["text"].as_str().map(str::to_string);
                            }
                            phone::Follow::OpenTable(table_id) => {
                                open_table = Some(table_id);
                            }
                            phone::Follow::Sale => {
                                if let Ok(sale) = serde_json::from_value::<Sale>(reply) {
                                    said = Some(format!("Billed {} · {}", sale.invoice_no, answer.done));
                                    snack = Some((sale.id, answer.done));
                                }
                            }
                        }
                    }
                }
            }
            st.lan_shown = phone::link_state();
        }
        // Reload outside the answer loop so a long SQL pass does not nest
        // under other UI work holding `app_state` (felt like a Client freeze).
        if synced {
            let tid = {
                let mut st = app_state().lock().unwrap();
                if let Some(tid) = open_table.take() {
                    st.selected_table_id = Some(tid);
                }
                let _ = st.reload_all();
                FACET_DIRTY.store(true, std::sync::atomic::Ordering::SeqCst);
                st.selected_table_id.clone()
            };
            if let Some(tid) = tid {
                let _ = app_state().lock().unwrap().load_table_cart(&tid);
            }
        } else if let Some(tid) = open_table {
            let mut st = app_state().lock().unwrap();
            st.selected_table_id = Some(tid.clone());
            let _ = st.load_table_cart(&tid);
        }
        if let Some(msg) = said {
            self.set_status(cx, msg);
        }
        {
            // A bill's own text beats the status on the receipt line.
            let mut st = app_state().lock().unwrap();
            if let Some((sale_id, _)) = &snack
                && let Some(text) = st.store.as_ref().and_then(|s| s.format_receipt_text(sale_id).ok())
            {
                receipt = Some(text);
            }
            if let Some(text) = receipt {
                st.last_receipt = text;
            }
        }
        if let Some((sale_id, text)) = snack {
            self.show_undo_snackbar(cx, sale_id, text);
        }
        self.sync_chrome(cx);
    }

    fn try_unlock(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        {
            let st = app_state().lock().unwrap();
            if st.must_confirm_device_role || st.device_role == DeviceRole::Unset {
                drop(st);
                self.go(cx, Screen::DeviceSetup);
                return;
            }
        }
        let (pin, lock_as_owner, lock_staff_id, has_store, owner_setup_complete, pin_reset) = {
            let st = app_state().lock().unwrap();
            (
                st.pin_buffer.clone(),
                st.lock_as_owner,
                st.lock_staff_id.clone(),
                st.store.is_some(),
                st.owner_setup_complete,
                st.pin_reset_required,
            )
        };
        if !has_store {
            self.set_status(cx, "Storage not ready — restart the app");
            return;
        }
        // Too many wrong PINs: sit out the block before trying again.
        if let Some(left) = self.unlock_block_remaining() {
            self.set_status(
                cx,
                format!("Too many wrong PINs — wait {left}s, then try again"),
            );
            return;
        }
        if pin.len() < 4 {
            self.set_status(cx, "Enter at least 4 digits, then OK");
            return;
        }

        // First-run Owner PIN setup (Server shop UX).
        if !owner_setup_complete {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let member = store.complete_owner_setup(&pin)?;
                let _ = store.set_active_session_staff_id(Some(&member.id));
                st.owner_setup_complete = true;
                st.onboarding_complete = false;
                st.onboard_step = 1;
                st.session = Some(member);
                st.reload_amount_visibility();
                st.pin_buffer.clear();
                st.pin_visible = false;
                st.lock_as_owner = true;
                // Nested under DeviceSetup so Back returns to Server/Client.
                st.nav_back.clear();
                st.nav_forward.clear();
                st.nav_back.push(Screen::DeviceSetup);
                st.screen = Screen::Onboard;
                st.status = "Owner ready — set up your shop".into();
                let _ = st.reload_all();
                Ok(())
            })();
            match result {
                Ok(()) => log!("Kumkaj owner setup ok"),
                Err(ref e) => {
                    log!("Kumkaj owner setup failed: {e}");
                    let mut st = app_state().lock().unwrap();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    drop(st);
                    self.set_status(cx, e.clone());
                }
            }
            self.sync_chrome(cx);
            return;
        }

        // The upgrade that replaced the PIN hasher handed every account back as
        // 0000. Reuse the first-run setup flow to get a real Owner PIN before
        // the till opens.
        if pin_reset {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let member = store.complete_pin_reset(&pin)?;
                let _ = store.set_active_session_staff_id(Some(&member.id));
                st.session = Some(member.clone());
                st.reload_amount_visibility();
                st.pin_buffer.clear();
                st.pin_visible = false;
                st.lock_as_owner = true;
                st.failed_unlocks = 0;
                st.screen = unlocked_home();
                st.status = format!("New Owner PIN set — welcome back, {}", member.lock_label());
                let _ = st.refresh_books_overview();
                let _ = st.reload_all();
                Ok(())
            })();
            match result {
                Ok(()) => log!("Kumkaj owner PIN reset ok"),
                Err(ref e) => {
                    log!("Kumkaj owner PIN reset failed: {e}");
                    let mut st = app_state().lock().unwrap();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    drop(st);
                    self.set_status(cx, e.clone());
                }
            }
            self.sync_chrome(cx);
            return;
        }

        // A paired device signs in on the Server (PINs are checked there,
        // never on the device), off the UI thread. Sync strips PIN hashes —
        // never fall through to local authenticate_staff_pin on a Client.
        let (is_client_paired, lan_link, staff_id_for_lan) = {
            let st = app_state().lock().unwrap();
            let is_client_paired = st.device_role == DeviceRole::Client && st.client_paired;
            let staff_id = if lock_as_owner {
                None
            } else {
                lock_staff_id.clone().or_else(|| {
                    st.staff
                        .iter()
                        .find(|s| !s.is_owner())
                        .map(|s| s.id.clone())
                })
            };
            let link = is_client_paired
                .then(|| {
                    st.store.as_ref().and_then(|s| {
                        Some(net::Link {
                            base: s.server_url().ok()??,
                            device: s.client_device_token().ok()??,
                            ..net::Link::default()
                        })
                    })
                })
                .flatten();
            (is_client_paired, link, staff_id)
        };
        if is_client_paired {
            if let Some(link) = lan_link {
                {
                    let mut st = app_state().lock().unwrap();
                    if st.lan_sign_in_busy {
                        return;
                    }
                    st.lan_sign_in_busy = true;
                    st.status = "Checking your PIN with the shop…".into();
                }
                let pin = pin.clone();
                let staff_id = staff_id_for_lan.clone();
                let _ = std::thread::Builder::new()
                    .name("khaynjo-sign-in".into())
                    .spawn(move || {
                        let reply = net::client_login(&link, staff_id.as_deref(), &pin);
                        *lan_once().lock().unwrap() =
                            Some(LanOnce::SignIn { staff_id, pin, reply });
                        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                    });
                self.sync_chrome(cx);
                return;
            }
            // No URL/token but still marked paired: offline unlock only (remembered PIN).
            let (auth, note) = {
                let st = app_state().lock().unwrap();
                match st.store.as_ref() {
                    Some(store) => match store.offline_lan_sign_in(staff_id_for_lan.as_deref(), &pin)
                    {
                        Ok(Some((staff, _))) => (
                            Ok(Some(staff)),
                            Some(
                                "Offline — orders wait on this device until the Wi‑Fi is back"
                                    .to_string(),
                            ),
                        ),
                        Ok(None) => (
                            Err(
                                "Can't reach the shop's server — check the Wi‑Fi (or unlock once while online so this device remembers your PIN)"
                                    .into(),
                            ),
                            None,
                        ),
                        Err(e) => (Err(e), None),
                    },
                    None => (Err("No store".into()), None),
                }
            };
            self.finish_unlock(cx, auth, lock_as_owner, note);
            return;
        }

        // Auth runs without holding the app mutex (PIN verify must not freeze the UI).
        // Emergency unlock PIN is checked first (wipe + open as Owner, looks normal).
        let auth = {
            let st = app_state().lock().unwrap();
            match st.store.as_ref() {
                Some(store) => {
                    if let Ok(Some(owner)) = store.try_duress_unlock(&pin) {
                        Ok(Some(owner))
                    } else if lock_as_owner {
                        store.authenticate_owner_pin(&pin)
                    } else {
                        match &lock_staff_id {
                            Some(id) => store.authenticate_staff_pin(id, &pin),
                            None => Ok(None),
                        }
                    }
                }
                None => Err("No store".into()),
            }
        };
        self.finish_unlock(cx, auth, lock_as_owner, None);
    }

    /// The Server's answer to a device's sign-in. Out of reach: whoever last
    /// signed in on this device may unlock with the session saved then.
    fn finish_lan_sign_in(
        &mut self,
        cx: &mut Cx,
        staff_id: Option<String>,
        pin: String,
        reply: Result<net::LoginOk, net::LanError>,
    ) {
        let lock_as_owner = staff_id.is_none();
        let (auth, note) = {
            let mut st = app_state().lock().unwrap();
            st.lan_sign_in_busy = false;
            let Some(store) = st.store.as_ref() else {
                return;
            };
            match reply {
                Ok(ok) => {
                    let saved = store.remember_lan_sign_in(&ok.staff, &pin, &ok.session);
                    phone::signed_in();
                    (saved.map(|()| Some(ok.staff)), None)
                }
                Err(e) if e.is_offline() => match store.offline_lan_sign_in(staff_id.as_deref(), &pin) {
                    Ok(Some((staff, _))) => (
                        Ok(Some(staff)),
                        Some("Offline — orders wait on this device until the Wi‑Fi is back".to_string()),
                    ),
                    Ok(None) => (Err(format!("{e} — and this person has not signed in on this device before")), None),
                    Err(err) => (Err(err), None),
                },
                Err(e) if e.code() == "wrong_pin" => (Ok(None), None),
                Err(e) if e.code() == "pair_again" => {
                    // Forgotten by the shop: pair again, and say why.
                    let _ = store.clear_client_pairing();
                    st.client_paired = false;
                    st.pair_note = st.t("device_forgotten");
                    st.screen = Screen::ClientPair;
                    (Err(st.pair_note.clone()), None)
                }
                Err(e) => (Err(e.to_string()), None),
            }
        };
        self.finish_unlock(cx, auth, lock_as_owner, note);
    }

    fn finish_unlock(
        &mut self,
        cx: &mut Cx,
        auth: Result<Option<StaffMember>, String>,
        lock_as_owner: bool,
        note: Option<String>,
    ) {
        let result = {
            let mut st = app_state().lock().unwrap();
            if !st.lock_as_owner && st.lock_staff_id.is_none() {
                st.lock_staff_id = st
                    .staff
                    .iter()
                    .find(|s| !s.is_owner())
                    .map(|s| s.id.clone());
            }
            match auth {
                Ok(Some(member)) => {
                    if let Some(store) = st.store.as_ref() {
                        let _ = store.set_active_session_staff_id(Some(&member.id));
                    }
                    st.session = Some(member.clone());
                    st.reload_amount_visibility();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    st.failed_unlocks = 0;
                    st.unlock_blocked_until = None;
                    if st.device_role == DeviceRole::Client && !st.client_paired {
                        st.nav_back.clear();
                        st.nav_forward.clear();
                        st.nav_back.push(Screen::DeviceSetup);
                        st.screen = Screen::ClientPair;
                        st.status = "Pair to the shop device before selling".into();
                    } else if st.device_role.is_host() && !st.onboarding_complete {
                        st.nav_back.clear();
                        st.nav_forward.clear();
                        st.nav_back.push(Screen::DeviceSetup);
                        st.screen = Screen::Onboard;
                        st.status = format!("Welcome, {} — finish shop setup", member.lock_label());
                    } else {
                        // Main till is a visit root — no Back into setup.
                        st.nav_back.clear();
                        st.nav_forward.clear();
                        st.screen = unlocked_home();
                        st.status = match &note {
                            Some(note) => format!("Welcome, {} · {note}", member.lock_label()),
                            None => format!("Welcome, {}", member.lock_label()),
                        };
                        let _ = st.refresh_books_overview();
                    }
                    if st.device_role == DeviceRole::Client
                        && st.client_paired
                        && let Some(store) = st.store.as_ref()
                    {
                        phone::start(store.db_path().to_path_buf());
                    }
                    // The books live here: back them up every day.
                    if st.device_role.is_host()
                        && let Some(store) = st.store.as_ref()
                    {
                        start_auto_backups(store.db_path().to_path_buf());
                    }
                    if st.uses_lan()
                        && let Some(store) = st.store.as_ref() {
                            let path = store.db_path().to_path_buf();
                            if net::ensure_server(path).is_ok() {
                                firewall_job(|| Ok(net::firewall_allows_phones()));
                            }
                        }
                    Ok(())
                }
                Ok(None) => {
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    st.failed_unlocks = st.failed_unlocks.saturating_add(1);
                    if st.failed_unlocks >= UNLOCK_ATTEMPT_LIMIT {
                        st.failed_unlocks = 0;
                        st.unlock_blocked_until =
                            Some(std::time::Instant::now() + UNLOCK_BLOCK);
                        Err(format!(
                            "Too many wrong PINs — locked for {}s",
                            UNLOCK_BLOCK.as_secs()
                        ))
                    } else {
                        let left = UNLOCK_ATTEMPT_LIMIT - st.failed_unlocks;
                        Err(if lock_as_owner {
                            format!("Wrong Owner PIN — {left} tries left")
                        } else if st.lock_staff_id.is_none() {
                            "Add staff in Team first, or unlock as Owner".into()
                        } else {
                            let who = st
                                .lock_staff_id
                                .as_ref()
                                .and_then(|id| st.staff.iter().find(|s| &s.id == id))
                                .map(|s| s.lock_label())
                                .unwrap_or_else(|| "selected staff".into());
                            format!("Wrong PIN for {who} — {left} tries left")
                        })
                    }
                }
                Err(e) => Err(e),
            }
        };
        match result {
            Ok(()) => log!("Kumkaj unlock ok"),
            Err(ref e) => log!("Kumkaj unlock failed: {e}"),
        }
        match result {
            Ok(()) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Soft-keyboard Done / Unlock-with-login: same path as the login button.
    fn try_unlock_with_login(&mut self, cx: &mut Cx) {
        let login = self.ui.text_input(cx, ids!(lock_login_id)).text();
        let password = self.ui.text_input(cx, ids!(lock_login_password)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.pin_reset_required {
                return Err("Owner must set a new PIN first".into());
            }
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            // Emergency PIN may be typed in the password field (same digits).
            let member = if let Some(owner) = store.try_duress_unlock(&password)? {
                owner
            } else {
                store
                    .authenticate_login(&login, &password)?
                    .ok_or_else(|| "Wrong login or password".to_string())?
            };
            let _ = store.set_active_session_staff_id(Some(&member.id));
            st.session = Some(member.clone());
            st.reload_amount_visibility();
            st.screen = if st.device_role.is_host() && !st.onboarding_complete {
                Screen::Onboard
            } else {
                unlocked_home()
            };
            st.status = format!("Welcome, {}", member.lock_label());
            let _ = st.reload_all();
            if st.screen == Screen::Books {
                let _ = st.refresh_books_overview();
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Open the menu window over the order desk: every dish, "All items"
    /// picked, nothing typed. On a desktop the search box takes the keys at
    /// once; on a tablet the keyboard waits for a tap on it (it would hide
    /// half the dishes).
    fn open_upi_qr_modal(&mut self, cx: &mut Cx) {
        let result = (|| -> Result<(String, String, Vec<u8>), String> {
            let st = app_state().lock().unwrap();
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            let vpa = store.upi_vpa()?;
            let payee = store.upi_payee()?;
            let amount = {
                let mut cart = st.cart.clone();
                let _ = store.apply_schemes_to_cart(&mut cart);
                let draft = cart.to_sale_draft()?;
                store.preview_sale_total(&draft)?
            };
            let uri = upi_qr::upi_pay_uri(&vpa, &payee, amount)?;
            let png = upi_qr::upi_qr_png(&uri, 8)?;
            let title_amt = st.money(Money(amount));
            Ok((format!("UPI {title_amt}"), title_amt, png))
        })();
        match result {
            Ok((title, amount, png)) => {
                self.ui
                    .label(cx, ids!(upi_qr_title))
                    .set_text(cx, &title);
                self.ui
                    .label(cx, ids!(upi_qr_amount))
                    .set_text(cx, &amount);
                let img = self.ui.image(cx, ids!(upi_qr_image));
                let _ = img.load_image_from_data(cx, &png);
                img.redraw(cx);
                self.ui.modal(cx, ids!(upi_qr_modal)).open(cx);
            }
            Err(e) => self.set_status(cx, e),
        }
    }

    fn open_menu_window(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.menu_pick_open = true;
            st.menu_filter.clear();
            st.menu_category = None;
            st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
        }
        let search = self.ui.text_input(cx, ids!(menu_search_input));
        search.set_text(cx, "");
        self.ui.modal(cx, ids!(menu_modal)).open(cx);
        if !cfg!(target_os = "android") {
            search.set_key_focus(cx);
        }
        self.sync_chrome(cx);
    }

    /// Close the menu window; the next opening starts from the whole menu.
    fn close_menu_window(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.menu_pick_open = false;
            st.menu_filter.clear();
            st.menu_category = None;
            st.menu_pick_rev = st.menu_pick_rev.wrapping_add(1);
        }
        self.ui.text_input(cx, ids!(menu_search_input)).set_text(cx, "");
        self.ui.modal(cx, ids!(menu_modal)).close(cx);
        self.dismiss_soft_keyboard(cx);
        self.sync_chrome(cx);
    }

    /// After typing in Add dishes search: refresh the grid + empty-state label
    /// only — never full `sync_chrome` (that redraws the whole app every key
    /// and makes Android IME / scroll feel broken).
    fn refresh_menu_pick_search(&mut self, cx: &mut Cx) {
        let (show_empty, empty_msg) = {
            let st = app_state().lock().unwrap();
            let searching = !st.menu_filter.trim().is_empty();
            let products_empty = st.visible_products().is_empty();
            let frequent_empty = !searching
                && st.menu_category.as_deref() == Some(MENU_FREQUENT)
                && products_empty;
            let added_empty = !searching
                && st.menu_category.as_deref() == Some(MENU_ADDED)
                && products_empty;
            let no_match = searching && products_empty;
            let empty_msg = if no_match {
                st.t("no_dish_match")
                    .replace("{q}", st.menu_filter.trim())
            } else if frequent_empty {
                st.t("frequent_empty")
            } else if added_empty {
                st.t("added_items_empty")
            } else {
                String::new()
            };
            (no_match || frequent_empty || added_empty, empty_msg)
        };
        self.ui
            .label(cx, ids!(menu_no_match))
            .set_visible(cx, show_empty);
        if show_empty {
            self.ui.label(cx, ids!(menu_no_match)).set_text(cx, &empty_msg);
        }
        self.ui.widget(cx, ids!(product_list)).redraw(cx);
        self.repair_focused_carets(cx);
    }

    /// A tile in the menu window: one more of that dish on the order.
    fn add_product_to_cart(&mut self, cx: &mut Cx, index: usize) {
        self.dismiss_soft_keyboard(cx);
        let id = app_state().lock().unwrap().visible_products().get(index).map(|p| p.id.clone());
        if let Some(id) = id {
            self.add_product_by_id(cx, &id);
        }
    }

    /// A card's −: one fewer of that dish (only what the kitchen has not got).
    fn remove_product_from_cart(&mut self, cx: &mut Cx, index: usize) {
        self.dismiss_soft_keyboard(cx);
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            let product = st.visible_products().get(index).cloned();
            match product {
                None => None,
                Some(_) if st.session.as_ref().map(|s| s.can_access_sell()) != Some(true) => {
                    Some("Not allowed to order".into())
                }
                Some(product) => {
                    let step = st
                        .cart
                        .lines
                        .iter()
                        .rev()
                        .find(|l| l.product_id == product.id && !l.is_sent())
                        .map(|l| l.bump_step_milli())
                        .unwrap_or(1000);
                    if st.selected_table_id.is_some() {
                        match st.bump_table_pending(
                            &product.id,
                            -step,
                            &format!("One {} taken off", product.name),
                        ) {
                            Err(e) => Some(e),
                            Ok(Acted::Sent) => {
                                let _ = st.cart.remove_one_of(&product.id);
                                st.status = format!("One {} taken off", product.name);
                                None
                            }
                            Ok(Acted::Here(_)) => {
                                let left = st.cart.lines.iter().any(|l| l.product_id == product.id);
                                st.status = if left {
                                    format!("One {} taken off", product.name)
                                } else {
                                    format!("{} taken off the order", product.name)
                                };
                                None
                            }
                        }
                    } else {
                        match st.cart.remove_one_of(&product.id) {
                            Err(e) => Some(e),
                            Ok(()) => {
                                let left = st.cart.lines.iter().any(|l| l.product_id == product.id);
                                st.status = if left {
                                    format!("One {} taken off", product.name)
                                } else {
                                    format!("{} taken off the order", product.name)
                                };
                                None
                            }
                        }
                    }
                }
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn add_product_by_id(&mut self, cx: &mut Cx, product_id: &str) {
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            if st.session.as_ref().map(|s| s.can_access_sell()) != Some(true) {
                Some("Not allowed to order".into())
            } else if let Some(product) = st.products.iter().find(|p| p.id == product_id).cloned() {
                let area = st.current_area();
                let step = if product.is_weight_unit() { 100 } else { 1000 };
                let result = if st.selected_table_id.is_some() {
                    match st.cart.check_can_add(&product, product.price_in_area(&area)) {
                        Err(e) => Some(e),
                        Ok(()) => match st.bump_table_pending(
                            &product.id,
                            step,
                            &format!("{} added", product.name),
                        ) {
                            Err(e) => Some(e),
                            Ok(Acted::Sent) => {
                                let _ = st.cart.add_in_area_in_stock(&product, &area);
                                st.status = if product.price_in_area(&area).0 == 0 {
                                    format!(
                                        "{} added · it has no price yet — set it on Menu",
                                        product.name
                                    )
                                } else {
                                    format!("{} added", product.name)
                                };
                                None
                            }
                            Ok(Acted::Here(_)) => {
                                st.status = if product.price_in_area(&area).0 == 0 {
                                    format!(
                                        "{} added · it has no price yet — set it on Menu",
                                        product.name
                                    )
                                } else {
                                    format!("{} added", product.name)
                                };
                                None
                            }
                        },
                    }
                } else {
                    match st.cart.add_in_area_in_stock(&product, &area) {
                        Err(e) => Some(e),
                        Ok(()) => {
                            st.status = if product.price_in_area(&area).0 == 0 {
                                format!(
                                    "{} added · it has no price yet — set it on Menu",
                                    product.name
                                )
                            } else {
                                format!("{} added", product.name)
                            };
                            None
                        }
                    }
                };
                // Reuse last Kitchen choice so each new ticket need not reopen it.
                if result.is_none() {
                    st.apply_remembered_kot_to_unassigned();
                }
                result
            } else {
                Some("Missing item".into())
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// "Find the shop's server" answered: every reply is a tappable row.
    /// Several shops → clear the host so the person must tap the one they want.
    /// One shop → fill that host (they can still retap the row).
    fn show_found_servers(&mut self, cx: &mut Cx, found: Vec<net::FoundServer>) {
        let (text, show_label, fill_host, clear_host) = {
            let mut st = app_state().lock().unwrap();
            let (text, show_label, fill_host, clear_host) = if found.is_empty() {
                (st.t("no_server_found"), true, None, false)
            } else if found.len() == 1 {
                (st.t("servers_found_pick"), true, Some(found[0].host.clone()), false)
            } else {
                (st.t("servers_found_pick"), true, None, true)
            };
            st.found_servers = found;
            (text, show_label, fill_host, clear_host)
        };
        let field = self.ui.text_input(cx, ids!(client_host_input));
        let typing = self.text_field_has_focus(cx);
        if !typing {
            if clear_host {
                field.set_text(cx, "");
            } else if let Some(host) = fill_host {
                if field.text().trim().is_empty() || !cx.has_key_focus(field.area()) {
                    field.set_text(cx, &host);
                }
            }
        }
        self.ui
            .widget(cx, ids!(client_found_label))
            .set_text(cx, &plain_display_text(&text));
        self.ui
            .widget(cx, ids!(client_found_label))
            .set_visible(cx, show_label);
        self.ui.widget(cx, ids!(client_found_list)).redraw(cx);
        if self.text_field_has_focus(cx) {
            self.paint_submit_busy_chrome(cx);
            self.ui.redraw(cx);
        } else {
            self.sync_chrome(cx);
        }
    }

    /// Select a table for the drawn-layout editor and fill X/Y/W/H fields.
    fn select_floor_layout_table(&mut self, cx: &mut Cx, table_id: &str) {
        let box_ = {
            let mut st = app_state().lock().unwrap();
            st.floor_layout_sel = Some(table_id.to_string());
            st.tables
                .iter()
                .find(|t| t.id == table_id)
                .map(|t| (t.layout_x, t.layout_y, t.layout_w, t.layout_h))
        };
        if let Some((x, y, w, h)) = box_ {
            let w = if w > 0.0 { w } else { FLOOR_LAYOUT_PACK_W };
            let h = if h > 0.0 { h } else { FLOOR_LAYOUT_PACK_H };
            self.ui
                .text_input(cx, ids!(layout_x_input))
                .set_text(cx, &format!("{x:.0}"));
            self.ui
                .text_input(cx, ids!(layout_y_input))
                .set_text(cx, &format!("{y:.0}"));
            self.ui
                .text_input(cx, ids!(layout_w_input))
                .set_text(cx, &format!("{w:.0}"));
            self.ui
                .text_input(cx, ids!(layout_h_input))
                .set_text(cx, &format!("{h:.0}"));
        }
        self.sync_chrome(cx);
    }

    fn parse_layout_fields(&mut self, cx: &mut Cx) -> Result<(f64, f64, f64, f64), String> {
        let parse = |raw: String| -> Result<f64, String> {
            let t = raw.trim().replace(',', ".");
            if t.is_empty() {
                return Ok(0.0);
            }
            t.parse::<f64>()
                .map_err(|_| "Enter numbers for X Y W H".into())
        };
        let x = parse(self.ui.text_input(cx, ids!(layout_x_input)).text())?;
        let y = parse(self.ui.text_input(cx, ids!(layout_y_input)).text())?;
        let w = parse(self.ui.text_input(cx, ids!(layout_w_input)).text())?;
        let h = parse(self.ui.text_input(cx, ids!(layout_h_input)).text())?;
        Ok((x, y, w.max(40.0), h.max(40.0)))
    }

    fn save_floor_layout_fields(&mut self, cx: &mut Cx) {
        let result = (|| -> Result<(), String> {
            let (x, y, w, h) = self.parse_layout_fields(cx)?;
            let mut st = app_state().lock().unwrap();
            let tid = st
                .floor_layout_sel
                .clone()
                .ok_or_else(|| st.t("floor_layout_pick"))?;
            let actor = st.actor()?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.set_table_layout(&tid, x, y, w, h, &actor)?;
            st.tables = store.list_tables()?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.set_status(cx, {
                    let st = app_state().lock().unwrap();
                    st.t("floor_layout_saved")
                });
                self.sync_chrome(cx);
            }
            Err(e) => self.set_status(cx, e),
        }
    }

    fn nudge_floor_layout(&mut self, cx: &mut Cx, dx: f64, dy: f64) {
        let cur = match self.parse_layout_fields(cx) {
            Ok(v) => v,
            Err(e) => {
                self.set_status(cx, e);
                return;
            }
        };
        let (x, y, w, h) = (cur.0 + dx, (cur.1 + dy).max(0.0), cur.2, cur.3);
        self.ui
            .text_input(cx, ids!(layout_x_input))
            .set_text(cx, &format!("{x:.0}"));
        self.ui
            .text_input(cx, ids!(layout_y_input))
            .set_text(cx, &format!("{y:.0}"));
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let tid = st
                .floor_layout_sel
                .clone()
                .ok_or_else(|| st.t("floor_layout_pick"))?;
            let actor = st.actor()?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.set_table_layout(&tid, x.max(0.0), y, w, h, &actor)?;
            st.tables = store.list_tables()?;
            Ok(())
        })();
        match result {
            Ok(()) => self.sync_chrome(cx),
            Err(e) => self.set_status(cx, e),
        }
    }

    /// Open a table from the floor: what it already has is the ticket; new
    /// taps go on the next KOT. On a paid table this seats new guests.
    /// Pick-table mode: first Choose seats unsent punches; Change table only
    /// opens the picked ticket (no drag of the previous table's lines).
    fn open_table(&mut self, cx: &mut Cx, item_id: usize) {
        // `Some(was_picking)` → opened a table; `None` → no-op.
        let picked = (|| -> Result<Option<bool>, String> {
            let mut st = app_state().lock().unwrap();
            let Some(t) = st.tables.get(item_id).cloned() else {
                return Ok(None);
            };
            st.pull_from_server();
            st.line_action = None;
            st.move_panel_open = false;
            let picking = st.picking_table;
            let from = st.picking_from_table_id.clone();
            if picking {
                // First seat only: attach unsent punches. Change table never
                // moves lines (Move / merge does that on purpose).
                if pick_seats_desk_cart(from.as_deref()) {
                    st.seat_cart_onto_table(&t.id)?;
                }
                st.picking_table = false;
                st.picking_from_table_id = None;
                st.load_table_cart(&t.id)?;
                st.status = format!("{} — add items and Send KOT, or Print bill / Take payment", t.name);
            } else {
                // Floor from nav: open that table's Server ticket as-is.
                st.picking_from_table_id = None;
                st.load_table_cart(&t.id)?;
                st.status = if st.cart.is_empty() {
                    format!("{} — tap items, then Send KOT", t.name)
                } else {
                    format!("{} — add items and Send KOT, or Print bill / Take payment", t.name)
                };
            }
            Ok(Some(picking))
        })();
        match picked {
            // Pick-table drill-down: pop Floor → Pos (do not push Pos again).
            Ok(Some(true)) => {
                if !self.navigate_back(cx) {
                    self.go_peer(cx, Screen::Pos);
                }
            }
            // Floor browse → Sell peer root.
            Ok(Some(false)) => self.go_peer(cx, Screen::Pos),
            Ok(None) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Guests are done: the table shows Bill printed / collect payment on
    /// every device (a tablet asks the Server).
    fn request_table_bill(&mut self, cx: &mut Cx, item_id: usize) {
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let actor = st.actor()?;
            let table = st
                .tables
                .get(item_id)
                .cloned()
                .ok_or_else(|| "Table gone".to_string())?;
            let done = format!(
                "{} — payment pending. Collect at the counter; \
                 the table releases when the payment is recorded",
                table.name
            );
            if st.device_role == DeviceRole::Client {
                // Waiter's device: the Server owns table state. Kept through
                // a Wi‑Fi drop like a KOT.
                st.lan_job(
                    "/table_billing",
                    &net::RemoteTableBillingRequest {
                        table_id: Some(table.id.clone()),
                        table_name: Some(table.name.clone()),
                    },
                    format!("Bill request for {}", table.name),
                    done,
                    phone::Follow::Refresh,
                    true,
                )?;
                return Ok(format!("{} — asking the counter for the bill…", table.name));
            }
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.request_billing(&table.id, &actor)?;
            st.tables = store.list_tables().unwrap_or_default();
            Ok(done)
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Columns for the **selected area** tab: empty = fit the screen; a
    /// number ≥ 1 = that many (rows wider than the screen scroll sideways).
    /// Saved per area on this device only.
    fn apply_floor_columns(&mut self, cx: &mut Cx, text: &str) {
        let text = text.trim();
        let result = {
            let mut st = app_state().lock().unwrap();
            let area = st.floor_area_tab.clone();
            let cols = if text.is_empty() || text.eq_ignore_ascii_case(&st.t("floor_cols_fit")) {
                Some(0)
            } else {
                text.parse::<usize>().ok().filter(|n| *n >= 1)
            };
            match cols {
                Some(cols) => {
                    let saved = st
                        .store
                        .as_ref()
                        .map(|s| s.set_floor_columns_for_area(&area, cols));
                    if let Some(Err(e)) = saved {
                        Err(e)
                    } else {
                        st.floor_columns = cols;
                        Ok(())
                    }
                }
                None => Err(st.t("floor_cols_min")),
            }
        };
        self.sync_floor_columns_field(cx, true);
        match result {
            Ok(()) => {
                self.ui.widget(cx, ids!(floor_list)).redraw(cx);
                self.ui.view(cx, ids!(floor_scroll)).redraw(cx);
            }
            Err(e) => self.set_status(cx, e),
        }
    }

    /// Put this area's columns into the Floor field, unless the person is
    /// typing in it (`force` after applying / switching tabs).
    fn sync_floor_columns_field(&mut self, cx: &mut Cx, force: bool) {
        let input = self.ui.text_input(cx, ids!(floor_cols_input));
        if !force && cx.has_key_focus(input.area()) {
            return;
        }
        let cols = app_state().lock().unwrap().floor_columns;
        let want = if cols == 0 { String::new() } else { cols.to_string() };
        if input.text() != want {
            input.set_text(cx, &want);
        }
    }

    /// Select a floor area tab: load that area's Columns, paint the active
    /// chip face, and redraw that tab's table body.
    fn select_floor_area_tab(&mut self, cx: &mut Cx, area: String) {
        {
            let mut st = app_state().lock().unwrap();
            st.floor_area_tab = area.clone();
            st.floor_columns = st
                .store
                .as_ref()
                .map(|s| s.floor_columns_for_area(&area))
                .unwrap_or(0);
        }
        self.sync_floor_columns_field(cx, true);
        // Same as category chips: paint idle/active faces from state now —
        // a strip redraw alone keeps the previous face lit.
        self.paint_floor_area_tabs(cx);
        self.ui.widget(cx, ids!(floor_list)).redraw(cx);
        self.ui.view(cx, ids!(floor_scroll)).redraw(cx);
    }

    /// Present floor areas in the owner's drag-to-swap order.
    fn floor_area_tab_names(&self) -> Vec<String> {
        let st = app_state().lock().unwrap();
        let present: Vec<String> = floor_area_groups(&floor_filtered_tables(&st))
            .into_iter()
            .map(|(a, _)| a)
            .collect();
        order_floor_areas(&present, &st.floor_area_order)
    }

    /// Paint area tabs from the current floor tables; clamp the selection.
    /// Selected tab = `ChipBtnActive` visible (DESIGN § Selection /
    /// `paint_choice_pair`); never leave every face idle. While dragging,
    /// the drop-target slot also wears the active face (same as nav).
    fn paint_floor_area_tabs(&mut self, cx: &mut Cx) {
        let drop_slot = floor_area_drag()
            .lock()
            .unwrap()
            .as_ref()
            .filter(|d| d.active)
            .and_then(|d| d.hover);
        let (areas, selected, other_label, cols_changed) = {
            let mut st = app_state().lock().unwrap();
            let present: Vec<String> = floor_area_groups(&floor_filtered_tables(&st))
                .into_iter()
                .map(|(a, _)| a)
                .collect();
            let names = order_floor_areas(&present, &st.floor_area_order);
            let mut cols_changed = false;
            if names.is_empty() {
                st.floor_area_tab.clear();
            } else if !names.iter().any(|a| a == &st.floor_area_tab) {
                st.floor_area_tab = names[0].clone();
                let cols = st
                    .store
                    .as_ref()
                    .map(|s| s.floor_columns_for_area(&st.floor_area_tab))
                    .unwrap_or(0);
                cols_changed = st.floor_columns != cols;
                st.floor_columns = cols;
            }
            let sel = st.floor_area_tab.clone();
            let other = st.t("tables");
            (names, sel, other, cols_changed)
        };
        if cols_changed {
            self.sync_floor_columns_field(cx, true);
        }
        let show = !areas.is_empty();
        self.ui
            .view(cx, ids!(floor_area_strip))
            .set_visible(cx, show);
        self.ui
            .view(cx, ids!(floor_cols_box))
            .set_visible(cx, show);
        if !show {
            for (idle, on) in FLOOR_AREA_CHIPS.iter() {
                self.ui.button(cx, idle).set_visible(cx, false);
                self.ui.button(cx, on).set_visible(cx, false);
            }
            return;
        }
        for (i, (idle, on)) in FLOOR_AREA_CHIPS.iter().enumerate() {
            match areas.get(i) {
                Some(area) => {
                    let label = if area.is_empty() {
                        other_label.as_str()
                    } else {
                        area.as_str()
                    };
                    let on_sel = area.as_str() == selected.as_str() || drop_slot == Some(i);
                    self.ui.button(cx, idle).set_text(cx, label);
                    self.ui.button(cx, on).set_text(cx, label);
                    self.paint_choice_pair(cx, idle, on, on_sel);
                }
                None => {
                    self.ui.button(cx, idle).set_visible(cx, false);
                    self.ui.button(cx, on).set_visible(cx, false);
                }
            }
        }
        self.ui.view(cx, ids!(floor_area_strip)).redraw(cx);
        self.sync_tab_chevrons(cx);
    }

    /// Index of the floor-focused table in `st.tables`, if any.
    fn floor_focus_index(&self) -> Option<usize> {
        let st = app_state().lock().unwrap();
        let id = st.floor_focus_table_id.as_ref()?;
        st.tables.iter().position(|t| &t.id == id)
    }

    /// Floor: print the guest bill for `tables[item_id]` (waiter + owner).
    /// Does not require the Order cart to be open on that table.
    fn print_floor_table_bill(&mut self, cx: &mut Cx, item_id: usize) {
        let pin = self.ui.text_input(cx, ids!(move_pin_input)).text();
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let table = st
                .tables
                .get(item_id)
                .cloned()
                .ok_or_else(|| "Table gone".to_string())?;
            st.floor_focus_table_id = Some(table.id.clone());
            if !session_may_reprint(st.session.as_ref()) {
                return Err("Not allowed to print bills — ask the owner".into());
            }
            let done = "Bill printed — take payment when the guests pay";
            let acted = st.table_action(
                net::RunningAction::PrintBill {
                    table_id: table.id.clone(),
                },
                &pin,
                done,
                phone::Follow::BillText,
            )?;
            let Acted::Here(text) = acted else {
                return Ok("Printing the bill at the counter…".to_string());
            };
            let text = text.unwrap_or_default();
            let dir = st
                .store
                .as_ref()
                .and_then(|s| s.db_path().parent().map(|p| p.to_path_buf()));
            st.last_receipt = text.clone();
            match print_share::print_route(
                print_share::PrintKind::Bill,
                print_share::RouteKey::Bill,
                &text,
                dir.as_deref(),
            ) {
                Ok(share) if share.opened => Ok(done.to_string()),
                Ok(_) => Ok(format!(
                    "{done} · add a bill printer in Settings → Where things print"
                )),
                Err(e) => Ok(format!("{done} · paper not sent ({e})")),
            }
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Show the guest bill text on the Floor (no printer).
    fn view_floor_table_bill(&mut self, cx: &mut Cx, item_id: usize) {
        let result = (|| -> Result<String, String> {
            let st = app_state().lock().unwrap();
            let table = st
                .tables
                .get(item_id)
                .cloned()
                .ok_or_else(|| "Table gone".to_string())?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            let order = store
                .running_order(&table.id)?
                .ok_or_else(|| format!("{} — no open bill yet", table.name))?;
            store.format_running_bill_text(&order)
        })();
        match result {
            Ok(text) => {
                app_state().lock().unwrap().last_receipt = text.clone();
                // ReportText lays out the same lines the printer gets
                // (name / qty×rate / totals) — not one WrapLabel paragraph.
                self.ui.widget(cx, ids!(floor_bill_text)).set_text(cx, &text);
                self.ui.view(cx, ids!(floor_bill_view)).set_visible(cx, true);
                self.set_status(cx, "Bill on screen — Print from the toolbar when ready");
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Guests sat down — mark Free → Occupied without opening an order.
    fn seat_table(&mut self, cx: &mut Cx, item_id: usize) {
        let table = app_state()
            .lock()
            .unwrap()
            .tables
            .get(item_id)
            .map(|t| (t.id.clone(), t.name.clone()));
        let Some((table_id, name)) = table else {
            return;
        };
        let done = format!("{name} occupied — guests seated");
        let result = app_state().lock().unwrap().table_action(
            net::RunningAction::SeatTable { table_id },
            "",
            &done,
            phone::Follow::Refresh,
        );
        match result {
            Ok(Acted::Here(_)) => self.set_status(cx, done),
            Ok(Acted::Sent) => self.set_status(cx, format!("{name} — sending…")),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Free the table for the next guests (Paid tile, Seated empty, or Bill printed at ₹0).
    fn mark_table_ready(&mut self, cx: &mut Cx, item_id: usize) {
        let table = app_state()
            .lock()
            .unwrap()
            .tables
            .get(item_id)
            .map(|t| (t.id.clone(), t.name.clone()));
        let Some((table_id, name)) = table else {
            return;
        };
        let done = format!("{name} is free");
        let result = app_state().lock().unwrap().table_action(
            net::RunningAction::TableReady { table_id },
            "",
            &done,
            phone::Follow::Refresh,
        );
        match result {
            Ok(Acted::Here(_)) => self.set_status(cx, done),
            Ok(Acted::Sent) => self.set_status(cx, format!("{name} — sending…")),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Undo accidental Print bill — table goes back to open (or frees if empty).
    fn undo_table_bill(&mut self, cx: &mut Cx, item_id: usize) {
        let table = app_state()
            .lock()
            .unwrap()
            .tables
            .get(item_id)
            .map(|t| (t.id.clone(), t.name.clone()));
        let Some((table_id, name)) = table else {
            return;
        };
        let done = format!("{name} — bill undone, table open again");
        let result = app_state().lock().unwrap().table_action(
            net::RunningAction::UndoBill { table_id },
            "",
            &done,
            phone::Follow::Refresh,
        );
        match result {
            Ok(Acted::Here(_)) => self.set_status(cx, done),
            Ok(Acted::Sent) => self.set_status(cx, format!("{name} — sending…")),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Open the ticket's action panel for one sent line.
    fn open_line_action(&mut self, cx: &mut Cx, index: usize, action: LineAction) {
        {
            let mut st = app_state().lock().unwrap();
            let Some(line) = st.cart.lines.get(index).filter(|l| l.is_sent()).cloned() else {
                return;
            };
            if line.kot_id == QUEUED_KOT {
                st.status = format!("{} is waiting to send — it can be changed once the kitchen has it", line.name);
                return;
            }
            // Counter-only KOT (no table): nothing in running_lines to cancel.
            if line.running_line_id.is_empty() || line.running_line_id == "counter" {
                st.status = st.t("cancel_needs_table");
                return;
            }
            st.line_action = Some((line.running_line_id.clone(), action));
            st.line_cooked = false;
            st.move_panel_open = false;
            st.status = line.name.clone();
        }
        for id in [
            ids!(line_reason_input),
            ids!(line_qty_input),
            ids!(line_target_input),
            ids!(line_pin_input),
        ] {
            self.ui.text_input(cx, id).set_text(cx, "");
        }
        self.sync_chrome(cx);
    }
}

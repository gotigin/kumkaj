impl App {
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
    /// Top-level peers do not keep history, so Back/swipe is a no-op there
    /// (except dismissing the soft keyboard).
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
                // Skip edge-swipe arming while the rail is showing — left edge
                // is the rail itself; swipe-back is for nested setup only.
                let skip_edge = {
                    let st = app_state().lock().unwrap();
                    let unlocked = !matches!(
                        st.screen,
                        Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                    );
                    unlocked && self.window_inner_width(cx) >= store::TABLET_SHELL_BREAKPOINT_CSS
                };
                let width = self.window_inner_width(cx);
                let mut origin = mouse_swipe_origin().lock().unwrap();
                if skip_edge {
                    *origin = None;
                } else if e.abs.x < 48.0 {
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
        self.resolve_data_base(cx);
        let base = {
            let st = app_state().lock().unwrap();
            st.data_base
                .clone()
                .ok_or_else(|| "No data directory".to_string())?
        };
        net::stop_server();
        let (_info, store) = local_servers::create_local_server(&base, name)?;
        {
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            // Language is app-global: keep the choice showing on DeviceSetup
            // across the reload (which reads the new shop's own meta).
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            self.finish_as_server(&mut st, true);
        }
        self.sync_chrome(cx);
        Ok(())
    }

    fn activate_existing_server(&mut self, cx: &mut Cx, server_id: &str) -> Result<(), String> {
        self.resolve_data_base(cx);
        let base = {
            let st = app_state().lock().unwrap();
            st.data_base
                .clone()
                .ok_or_else(|| "No data directory".to_string())?
        };
        net::stop_server();
        let store = local_servers::open_server(&base, server_id)?;
        {
            let mut st = app_state().lock().unwrap();
            let previous = st.device_role;
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            // Language is app-global: keep the choice showing on DeviceSetup
            // across the reload (which reads this shop's own meta).
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            self.finish_as_server(&mut st, previous != DeviceRole::Server);
        }
        self.sync_chrome(cx);
        Ok(())
    }

    fn finish_as_server(&mut self, st: &mut AppState, switching_in: bool) {
        st.device_role = DeviceRole::Server;
        st.must_confirm_device_role = false;
        st.picking_server = false;
        st.lock_as_owner = true;
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
        st.screen = Screen::Lock;
        st.status = if !st.owner_setup_complete {
            locale::t(&st.locale.language, "status_server_set_pin")
        } else if st.extra_tills_allowed {
            locale::t(&st.locale.language, "status_server_tills")
        } else {
            locale::t(&st.locale.language, "status_server_unlock")
        };
    }

    fn begin_client_mode(&mut self, cx: &mut Cx) {
        self.resolve_data_base(cx);
        let result = (|| -> Result<(), String> {
            let base = {
                let st = app_state().lock().unwrap();
                st.data_base
                    .clone()
                    .ok_or_else(|| "No data directory".to_string())?
            };
            net::stop_server();
            let store = local_servers::open_client_store(&base)?;
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            // Language is app-global: keep the choice showing on DeviceSetup
            // across the reload (which reads the client cache's own meta).
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.prepare_as_client_terminal()?;
            // Saved like Server Mode is: a reload (pairing, sync) reads the
            // role back from the store and must not find it unset.
            store.set_device_role(DeviceRole::Client.as_str())?;
            st.device_role = DeviceRole::Client;
            st.must_confirm_device_role = false;
            st.owner_setup_complete = true;
            st.onboarding_complete = true;
            st.extra_tills_allowed = false;
            st.lock_as_owner = true;
            st.pin_buffer.clear();
            st.nav_back.clear();
            st.nav_forward.clear();
            let _ = st.reload_all();
            st.client_paired = store_client_paired(st.store.as_ref());
            if st.staff.iter().any(|s| !s.is_owner()) {
                st.lock_as_owner = false;
                if st.lock_staff_id.is_none() {
                    st.lock_staff_id = st
                        .staff
                        .iter()
                        .find(|s| !s.is_owner())
                        .map(|s| s.id.clone());
                }
            }
            if st.client_paired {
                st.screen = Screen::Lock;
                st.status = locale::t(&st.locale.language, "status_client_unlock");
            } else {
                st.screen = Screen::ClientPair;
                st.status = "Client Mode — pair to the Server before selling".into();
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
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
            Some(LanOnce::Paired(reply)) => {
                let result = (|| -> Result<String, String> {
                    let (link, snap) = reply?;
                    let mut st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.set_client_device_pairing(&link.base, &link.device)?;
                    // Kept to find the Server again if its address changes.
                    store.set_paired_server_id(&link.server_id)?;
                    phone::forget_revision();
                    net::apply_snapshot(store, &snap)?;
                    phone::start(store.db_path().to_path_buf());
                    st.client_paired = true;
                    st.reload_all()?;
                    // The person signs in with their shop PIN: this device's
                    // own sign-in is not one the Server knows.
                    st.session = None;
                    st.screen = Screen::Lock;
                    {
                        // The shop's name as it saved it; an unnamed shop is "the shop".
                        let name = [snap.venue_title.trim(), snap.org_name.trim()]
                            .into_iter()
                            .find(|n| !n.is_empty())
                            .unwrap_or("the shop")
                            .to_string();
                        Ok(format!("Paired with {name} — sign in with your shop PIN"))
                    }
                })();
                match result {
                    Ok(msg) => {
                        app_state().lock().unwrap().pair_note.clear();
                        self.set_status(cx, msg)
                    }
                    Err(e) => {
                        app_state().lock().unwrap().pair_note = e.clone();
                        self.set_status(cx, e)
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
                                st.selected_table_id = Some(table_id);
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
            if synced {
                let _ = st.reload_all();
            }
            if let Some(tid) = st.selected_table_id.clone() {
                let _ = st.load_table_cart(&tid);
            }
            st.lan_shown = phone::link_state();
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
        // never on the device), off the UI thread.
        let lan = {
            let st = app_state().lock().unwrap();
            let staff_id = if lock_as_owner {
                None
            } else {
                lock_staff_id
                    .clone()
                    .or_else(|| st.staff.iter().find(|s| !s.is_owner()).map(|s| s.id.clone()))
            };
            let link = (st.device_role == DeviceRole::Client && st.client_paired)
                .then(|| st.store.as_ref().and_then(|s| Some(net::Link {
                    base: s.server_url().ok()??,
                    device: s.client_device_token().ok()??,
                    ..net::Link::default()
                })))
                .flatten();
            link.map(|link| (link, staff_id))
        };
        if let Some((link, staff_id)) = lan {
            {
                let mut st = app_state().lock().unwrap();
                if st.lan_sign_in_busy {
                    return;
                }
                st.lan_sign_in_busy = true;
                st.status = "Checking your PIN with the shop…".into();
            }
            let pin = pin.clone();
            let _ = std::thread::Builder::new()
                .name("khaynjo-sign-in".into())
                .spawn(move || {
                    let reply = net::client_login(&link, staff_id.as_deref(), &pin);
                    *lan_once().lock().unwrap() = Some(LanOnce::SignIn { staff_id, pin, reply });
                    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                });
            self.sync_chrome(cx);
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
                        let staff_id = lock_staff_id.clone().or_else(|| {
                            st.staff
                                .iter()
                                .find(|s| !s.is_owner())
                                .map(|s| s.id.clone())
                        });
                        match staff_id {
                            Some(id) => store.authenticate_staff_pin(&id, &pin),
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
                        st.screen = Screen::ClientPair;
                        st.status = "Pair to the shop device before selling".into();
                    } else if st.device_role.is_host() && !st.onboarding_complete {
                        st.screen = Screen::Onboard;
                        st.status = format!("Welcome, {} — finish shop setup", member.lock_label());
                    } else {
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
                            format!("Wrong Staff PIN — {left} tries left")
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
        cx.set_key_focus(Area::Empty);
        self.sync_chrome(cx);
    }

    /// A tile in the menu window: one more of that dish on the order.
    fn add_product_to_cart(&mut self, cx: &mut Cx, index: usize) {
        let id = app_state().lock().unwrap().visible_products().get(index).map(|p| p.id.clone());
        if let Some(id) = id {
            self.add_product_by_id(cx, &id);
        }
    }

    /// A card's −: one fewer of that dish (only what the kitchen has not got).
    fn remove_product_from_cart(&mut self, cx: &mut Cx, index: usize) {
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
                if st.selected_table_id.is_some() {
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
                }
            } else {
                Some("Missing item".into())
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// "Find the shop's server" answered: fill the IP of the first one found
    /// and list them all (by restaurant name) so the waiter can confirm.
    fn show_found_servers(&mut self, cx: &mut Cx, found: Vec<net::FoundServer>) {
        let (text, rows) = {
            let mut st = app_state().lock().unwrap();
            let text = if found.is_empty() { st.t("no_server_found") } else { String::new() };
            // One row per restaurant found: its name, and the address.
            let rows = found
                .iter()
                .map(|f| {
                    let venue = if f.venue.trim().is_empty() { "Khaynjo" } else { f.venue.trim() };
                    ReportRow::Kv(venue.to_string(), f.host.clone())
                })
                .collect::<Vec<_>>();
            st.found_servers = found;
            (text, rows)
        };
        let first = app_state().lock().unwrap().found_servers.first().map(|f| f.host.clone());
        if let Some(host) = first {
            self.ui.text_input(cx, ids!(client_host_input)).set_text(cx, &host);
        }
        self.ui
            .widget(cx, ids!(client_found_label))
            .set_text(cx, &plain_display_text(&text));
        self.ui.widget(cx, ids!(client_found_label)).set_visible(cx, !text.is_empty());
        self.set_report_rows(cx, ids!(client_found_list), rows);
        self.sync_chrome(cx);
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

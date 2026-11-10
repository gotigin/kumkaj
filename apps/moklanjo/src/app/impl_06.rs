impl App {
    /// Resolve Team type-or-pick text to one active person.
    /// Prefer id → unique username → lock_label → unique display name.
    /// Never guess among duplicate names (that mistargeted rights/PIN).
    fn find_active_staff(st: &AppState, needle: &str) -> Result<StaffMember, String> {
        let raw = needle.trim();
        if raw.is_empty() {
            return Err("Type a staff name or username, or pick from the list".into());
        }
        if let Some(m) = st.staff.iter().find(|s| s.active && s.id == raw) {
            return Ok(m.clone());
        }
        let by_login: Vec<_> = st
            .staff
            .iter()
            .filter(|s| {
                s.active
                    && !s.login_id.is_empty()
                    && s.login_id.eq_ignore_ascii_case(raw)
            })
            .cloned()
            .collect();
        if by_login.len() == 1 {
            return Ok(by_login.into_iter().next().expect("len 1"));
        }
        if by_login.len() > 1 {
            return Err("Several people share that username — fix Team logins".into());
        }
        if let Some(m) = st
            .staff
            .iter()
            .find(|s| s.active && s.lock_label() == raw)
        {
            return Ok(m.clone());
        }
        let by_label: Vec<_> = st
            .staff
            .iter()
            .filter(|s| s.active && s.lock_label().eq_ignore_ascii_case(raw))
            .cloned()
            .collect();
        if by_label.len() == 1 {
            return Ok(by_label.into_iter().next().expect("len 1"));
        }
        let by_name: Vec<_> = st
            .staff
            .iter()
            .filter(|s| s.active && s.name.trim().eq_ignore_ascii_case(raw))
            .cloned()
            .collect();
        if by_name.len() == 1 {
            return Ok(by_name.into_iter().next().expect("len 1"));
        }
        if by_name.len() > 1 {
            return Err(format!(
                "Several people named \"{raw}\" — pick from the list or use their username"
            ));
        }
        let lower = raw.to_lowercase();
        let by_sub: Vec<_> = st
            .staff
            .iter()
            .filter(|s| {
                s.active
                    && (s.lock_label().to_lowercase().contains(&lower)
                        || s.name.to_lowercase().contains(&lower)
                        || (!s.login_id.is_empty()
                            && s.login_id.to_lowercase().contains(&lower)))
            })
            .cloned()
            .collect();
        if by_sub.len() == 1 {
            return Ok(by_sub.into_iter().next().expect("len 1"));
        }
        if by_sub.len() > 1 {
            return Err(format!(
                "Several matches for \"{raw}\" — pick from the list or type their username"
            ));
        }
        Err(format!("No staff matching \"{raw}\""))
    }

fn dismiss_soft_keyboard(&mut self, cx: &mut Cx) {
        cx.set_key_focus(Area::Empty);
        // Latch dismissed — `hide_text_ime` clears that latch and let the next
        // focused draw re-open the IME without a tap (felt stuck / random).
        cx.text_ime_was_dismissed();
        if let Ok(mut flag) = ime_suppress_refocus().lock() {
            *flag = true;
        }
    }

    fn navigate_back(&mut self, cx: &mut Cx) -> bool {
        // Soft keyboard: on Android, one Back dismisses IME first. On desktop,
        // mouse Back / Alt+← must still pop — a focused PIN/search field used
        // to swallow the gesture (looked like Back was broken on Lock).
        // Auth/setup always blur then continue (mouse Back also arrives as a
        // key within 140ms; returning early dropped the real pop).
        if self.text_field_has_focus(cx) {
            self.dismiss_soft_keyboard(cx);
            let auth = matches!(
                app_state().lock().unwrap().screen,
                Screen::Lock
                    | Screen::Onboard
                    | Screen::ClientPair
                    | Screen::DeviceSetup
            );
            #[cfg(target_os = "android")]
            if !auth {
                return true;
            }
            let _ = auth;
        }
        // In-screen layers before screen history (native / mouse / Android back).
        // Suggest list first — clear + redraw so the panel does not linger.
        if self.close_open_suggest(cx) {
            self.sync_chrome(cx);
            return true;
        }
        {
            let mut st = app_state().lock().unwrap();
            if st.picking_server {
                st.picking_server = false;
                drop(st);
                self.sync_chrome(cx);
                return true;
            }
            // Settings section → hub (never straight off Settings).
            if let Some(next) = settings_back_step(st.screen, st.settings_panel) {
                drop(st);
                self.open_settings_panel(cx, next);
                return true;
            }
            // DeviceSetup nested accordion (Language open) closes before leaving.
            if st.screen == Screen::DeviceSetup
                && st
                    .device_setup_sections_open
                    .iter()
                    .any(|k| k == "language")
            {
                st.device_setup_sections_open.retain(|k| k != "language");
                drop(st);
                self.sync_chrome(cx);
                return true;
            }
            // Onboard nested: Bring my book → back to Start fresh choice.
            if st.screen == Screen::Onboard && st.onboard_step == 2 {
                st.onboard_step = 1;
                drop(st);
                self.sync_chrome(cx);
                return true;
            }
        }
        let dest = {
            let mut st = app_state().lock().unwrap();
            if let Some(prev) = st.nav_back.pop() {
                let current = st.screen;
                // Never put Onboard on Forward — once Server/Client
                // (DeviceSetup) is open, shop setup must not return via Forward.
                if current != Screen::Onboard {
                    st.nav_forward.push(current);
                    if st.nav_forward.len() > 32 {
                        st.nav_forward.remove(0);
                    }
                }
                Some(prev)
            } else {
                // Empty stack: setup / SidebarSettings have a fixed parent.
                // Peers with no history stay put (platform Back is a no-op).
                match st.screen {
                    Screen::Lock | Screen::ClientPair | Screen::Onboard => {
                        st.nav_forward.clear();
                        st.must_confirm_device_role = true;
                        st.pin_buffer.clear();
                        st.status = match st.screen {
                            Screen::Onboard => {
                                "Pick Server Mode or Client Mode — Back left shop setup".into()
                            }
                            Screen::ClientPair => {
                                "Pick Server Mode or Client Mode — Back left Client pairing".into()
                            }
                            _ => "Pick Server Mode or Client Mode — Back left unlock".into(),
                        };
                        Some(Screen::DeviceSetup)
                    }
                    Screen::SidebarSettings => {
                        st.nav_forward.clear();
                        Some(Screen::Settings)
                    }
                    _ => None,
                }
            }
        };
        if let Some(prev) = dest {
            {
                let mut st = app_state().lock().unwrap();
                if prev == Screen::DeviceSetup {
                    st.must_confirm_device_role = true;
                    // Role picker is a fence: drop Forward (esp. Onboard) and
                    // any Onboard still under Back.
                    st.nav_forward.clear();
                    st.nav_back.retain(|s| *s != Screen::Onboard);
                }
                self.set_screen(&mut st, prev, false);
            }
            self.sync_chrome(cx);
            true
        } else {
            false
        }
    }

    /// Mouse Forward / right-edge swipe / Alt+Right — redo a screen we backed out of.
    fn navigate_forward(&mut self, cx: &mut Cx) -> bool {
        let dest = {
            let mut st = app_state().lock().unwrap();
            // Skip Onboard — shop setup is never restored by Forward after
            // Server/Client mode.
            loop {
                let Some(next) = st.nav_forward.pop() else {
                    break None;
                };
                if next == Screen::Onboard {
                    continue;
                }
                let current = st.screen;
                st.nav_back.push(current);
                if st.nav_back.len() > 32 {
                    st.nav_back.remove(0);
                }
                break Some(next);
            }
        };
        if let Some(next) = dest {
            {
                let mut st = app_state().lock().unwrap();
                self.set_screen(&mut st, next, false);
            }
            self.sync_chrome(cx);
            true
        } else {
            false
        }
    }

    fn window_inner_size(&self, cx: &Cx) -> DVec2 {
        let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
        dvec2(
            if size.x > 1.0 { size.x } else { 1024.0 },
            if size.y > 1.0 { size.y } else { 768.0 },
        )
    }

    /// Desktop only: stash last size/position and apply it before CreateWindow.
    fn restore_window_geom(&self, cx: &mut Cx) {
        if cfg!(target_os = "android") {
            return;
        }
        let Some(prefs) = store::load_window_geom() else {
            *pending_window_geom().lock().unwrap() = None;
            return;
        };
        let min = store::MIN_DESKTOP_WINDOW_CSS;
        let prefs = store::WindowGeomPrefs {
            width: prefs.width.max(min),
            height: prefs.height.max(min),
            x: prefs.x,
            y: prefs.y,
        };
        *pending_window_geom().lock().unwrap() = Some(prefs);
        self.ui.window(cx, ids!(main_window)).configure_window(
            cx,
            dvec2(prefs.width, prefs.height),
            dvec2(prefs.x, prefs.y),
            false,
            store::APP_LABEL.to_string(),
        );
    }

    /// After the OS window exists, force the saved size if create used the DSL default.
    /// Returns true when a restore resize was requested (skip persisting that frame).
    fn apply_pending_window_geom_after_create(&self, cx: &mut Cx) -> bool {
        if cfg!(target_os = "android") {
            return false;
        }
        let Some(prefs) = pending_window_geom().lock().unwrap().take() else {
            return false;
        };
        let win = self.ui.window(cx, ids!(main_window));
        let size = win.get_inner_size(cx);
        let pos = win.get_position(cx);
        let size_mismatch =
            (size.x - prefs.width).abs() > 1.0 || (size.y - prefs.height).abs() > 1.0;
        let pos_mismatch = (pos.x - prefs.x).abs() > 1.0 || (pos.y - prefs.y).abs() > 1.0;
        if size_mismatch {
            win.resize(cx, dvec2(prefs.width, prefs.height));
        }
        if pos_mismatch {
            win.reposition(cx, dvec2(prefs.x, prefs.y));
        }
        size_mismatch || pos_mismatch
    }

    /// Desktop (Windows / Linux / macOS): never allow inner size below 375 css px.
    fn enforce_min_desktop_window_size(&self, cx: &mut Cx) {
        if cfg!(target_os = "android") {
            return;
        }
        let win = self.ui.window(cx, ids!(main_window));
        let size = win.get_inner_size(cx);
        let min = store::MIN_DESKTOP_WINDOW_CSS;
        let w = size.x.max(min);
        let h = size.y.max(min);
        if (w - size.x).abs() > 0.5 || (h - size.y).abs() > 0.5 {
            win.resize(cx, dvec2(w, h));
        }
    }

    fn persist_window_geom(&self, cx: &Cx) {
        if cfg!(target_os = "android") {
            return;
        }
        // Don't clobber a good saved size with the pre-restore create size.
        if pending_window_geom().lock().unwrap().is_some() {
            return;
        }
        let win = self.ui.window(cx, ids!(main_window));
        // Never persist maximized/fullscreen geometry. A large floating
        // window is a real size and must be kept.
        if win.is_fullscreen(cx) {
            return;
        }
        let size = win.get_inner_size(cx);
        let pos = win.get_position(cx);
        let min = store::MIN_DESKTOP_WINDOW_CSS;
        store::save_window_geom(store::WindowGeomPrefs {
            width: size.x.max(min),
            height: size.y.max(min),
            x: pos.x,
            y: pos.y,
        });
    }

    fn window_inner_width(&self, cx: &Cx) -> f64 {
        self.window_inner_size(cx).x
    }

    /// Caption-bar WxH (left of the Kumkaj title). Controlled by
    /// `KUMKAJ_SHOW_SIZE_BADGE` in this crate's `env/development.env` (not Cargo profile —
    /// Android dev builds use `--release` but still load development.env).
    fn sync_dev_size(&mut self, cx: &mut Cx) {
        let show = store::SHOW_SIZE_BADGE;
        self.ui.view(cx, ids!(dev_size_badge)).set_visible(cx, show);
        if !show {
            return;
        }
        let size = self.window_inner_size(cx);
        // Compact: fits the caption row next to the title without clipping.
        let text = format!("{:.0}×{:.0}", size.x, size.y);
        self.ui.label(cx, ids!(dev_size_label)).set_text(cx, &text);
    }

    fn edge_swipe_nav(&mut self, cx: &mut Cx, start: DVec2, end: DVec2, edge: SwipeNav) -> bool {
        let dx = end.x - start.x;
        let dy = (end.y - start.y).abs();
        match edge {
            SwipeNav::Back if dx > 72.0 && dx > dy * 1.5 => self.navigate_back(cx),
            SwipeNav::Forward if dx < -72.0 && (-dx) > dy * 1.5 => self.navigate_forward(cx),
            _ => false,
        }
    }

    /// Drag the rail divider; clamp so rail ≥ [`RAIL_WIDTH_MIN`] and main ≥ [`MAIN_CONTENT_MIN`].
    fn pointer_rail_resize(&mut self, cx: &mut Cx, event: &Event) -> bool {
        const DIVIDER_W: f64 = 6.0;
        let unlocked_rail = {
            let st = app_state().lock().unwrap();
            let unlocked = !matches!(
                st.screen,
                Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
            );
            let wide = self.window_inner_width(cx) >= store::TABLET_SHELL_BREAKPOINT_CSS;
            unlocked && wide && !st.rail_collapsed && rail_width_anim().lock().unwrap().is_none()
        };
        match event {
            Event::MouseDown(e) if e.button.is_primary() && unlocked_rail => {
                *rail_width_anim().lock().unwrap() = None;
                let rail_w = app_state().lock().unwrap().rail_width;
                let x = e.abs.x;
                if x >= rail_w - 2.0 && x <= rail_w + DIVIDER_W + 8.0 {
                    *rail_resize_drag().lock().unwrap() = Some(x - rail_w);
                    *mouse_swipe_origin().lock().unwrap() = None;
                    cx.set_cursor(MouseCursor::ColResize);
                    true
                } else {
                    false
                }
            }
            Event::MouseMove(e) if rail_resize_drag().lock().unwrap().is_some() => {
                let grab = rail_resize_drag().lock().unwrap().unwrap_or(0.0);
                let window_w = self.window_inner_width(cx);
                let max_rail = (window_w - MAIN_CONTENT_MIN - DIVIDER_W).max(RAIL_WIDTH_MIN);
                let new_w = (e.abs.x - grab).clamp(RAIL_WIDTH_MIN, max_rail);
                {
                    app_state().lock().unwrap().rail_width = new_w;
                }
                cx.set_cursor(MouseCursor::ColResize);
                self.apply_shell_layout(cx, true);
                true
            }
            Event::MouseMove(e) if unlocked_rail => {
                // Hover affordance even when SolidView HoverIn was wiped by
                // window-edge leave or a peer SetCursor.
                let rail_w = app_state().lock().unwrap().rail_width;
                let x = e.abs.x;
                if x >= rail_w - 2.0 && x <= rail_w + DIVIDER_W + 8.0 {
                    cx.set_cursor(MouseCursor::ColResize);
                }
                false
            }
            Event::MouseUp(e) if e.button.is_primary() => {
                if rail_resize_drag().lock().unwrap().take().is_some() {
                    self.apply_shell_layout(cx, true);
                    true
                } else {
                    false
                }
            }
            // A finger is wider than the 6px divider, and a touch screen
            // never sends MouseDown for it.
            Event::TouchUpdate(e) => {
                let mut handled = false;
                for t in &e.touches {
                    match t.state {
                        TouchState::Start if unlocked_rail => {
                            *rail_width_anim().lock().unwrap() = None;
                            let rail_w = app_state().lock().unwrap().rail_width;
                            let x = t.abs.x;
                            if x >= rail_w - 12.0 && x <= rail_w + DIVIDER_W + 22.0 {
                                *rail_resize_drag().lock().unwrap() = Some(x - rail_w);
                                *mouse_swipe_origin().lock().unwrap() = None;
                                handled = true;
                            }
                        }
                        TouchState::Move if rail_resize_drag().lock().unwrap().is_some() => {
                            let grab = rail_resize_drag().lock().unwrap().unwrap_or(0.0);
                            let window_w = self.window_inner_width(cx);
                            let max_rail =
                                (window_w - MAIN_CONTENT_MIN - DIVIDER_W).max(RAIL_WIDTH_MIN);
                            let new_w = (t.abs.x - grab).clamp(RAIL_WIDTH_MIN, max_rail);
                            app_state().lock().unwrap().rail_width = new_w;
                            self.apply_shell_layout(cx, true);
                            handled = true;
                        }
                        TouchState::Stop => {
                            if rail_resize_drag().lock().unwrap().take().is_some() {
                                self.apply_shell_layout(cx, true);
                                handled = true;
                            }
                        }
                        _ => {}
                    }
                }
                handled
            }
            _ => false,
        }
    }

    /// Books home accordion bodies + header chevrons from `books_sections_open`.
    /// Skips visibility/height while a section is mid-animation.
    /// No-Sales sessions never see money sections (same as today bar).
    fn sync_books_sections(&mut self, cx: &mut Cx) {
        let (open, lang, may_see) = {
            let st = app_state().lock().unwrap();
            (
                st.books_sections_open.clone(),
                st.locale.language.clone(),
                session_may_see_today(st.session.as_ref()),
            )
        };
        let is_open = |key: &str| open.iter().any(|k| k == key);
        let s = |k: &str| locale::t(&lang, k);
        let animating = accordion_anims().lock().unwrap();
        let sections: &[(&str, &[LiveId], &[LiveId], &str)] = &[
            (
                "today",
                ids!(books_sec_today_hdr),
                ids!(books_sec_today_body),
                "caption_today",
            ),
            (
                "bookings",
                ids!(books_sec_bookings_hdr),
                ids!(books_sec_bookings_body),
                "caption_bookings",
            ),
            (
                "payments",
                ids!(books_sec_payments_hdr),
                ids!(books_sec_payments_body),
                "caption_payments_today",
            ),
            (
                "outstanding",
                ids!(books_sec_outstanding_hdr),
                ids!(books_sec_outstanding_body),
                "caption_outstanding",
            ),
            (
                "bank",
                ids!(books_sec_bank_hdr),
                ids!(books_sec_bank_body),
                "caption_bank",
            ),
        ];
        for (key, hdr, body, caption_key) in sections {
            self.ui.button(cx, hdr).set_visible(cx, may_see);
            if !may_see {
                self.ui.view(cx, body).set_visible(cx, false);
                continue;
            }
            let open = is_open(key);
            let mark = if open {
                ACCORDION_MARK_OPEN
            } else {
                ACCORDION_MARK_CLOSED
            };
            self.ui
                .button(cx, hdr)
                .set_text(cx, &format!("{mark}  {}", s(caption_key)));
            if animating.contains_key(*key) {
                continue;
            }
            self.ui.view(cx, body).set_visible(cx, open);
            if let Some(mut v) = self.ui.view(cx, body).borrow_mut() {
                v.walk.height = Size::fit();
            }
        }
    }

    /// LiveId path for any animated accordion body (books / setup).
    fn section_body_id(key: &str) -> Option<&'static [LiveId]> {
        match key {
            "today" => Some(ids!(books_sec_today_body)),
            "bookings" => Some(ids!(books_sec_bookings_body)),
            "payments" => Some(ids!(books_sec_payments_body)),
            "outstanding" => Some(ids!(books_sec_outstanding_body)),
            "bank" => Some(ids!(books_sec_bank_body)),
            "language" => Some(ids!(setup_lang_body)),
            "role" => Some(ids!(setup_role_body)),
            _ => None,
        }
    }

    fn set_section_body_height(&mut self, cx: &mut Cx, body: &[LiveId], height: Option<f64>) {
        if let Some(mut v) = self.ui.view(cx, body).borrow_mut() {
            v.walk.height = match height {
                Some(h) => Size::Fixed(h.max(0.0)),
                None => Size::fit(),
            };
        }
        self.ui.view(cx, body).redraw(cx);
    }

    /// Start a height tween for one accordion body (`will_open` after state flip).
    fn start_section_height_anim(&mut self, cx: &mut Cx, key: &str, will_open: bool) {
        let Some(body) = Self::section_body_id(key) else {
            return;
        };
        let measured = self.ui.view(cx, body).area().rect(cx).size.y;
        let content_h = if measured > 1.0 {
            measured
        } else {
            accordion_estimate_h(key)
        };
        let (from, to) = if will_open { (0.0, 1.0) } else { (1.0, 0.0) };
        if will_open {
            self.ui.view(cx, body).set_visible(cx, true);
            self.set_section_body_height(cx, body, Some(0.0));
        } else {
            self.set_section_body_height(cx, body, Some(content_h));
        }
        accordion_anims().lock().unwrap().insert(
            key.to_string(),
            AccordionAnim {
                from,
                to,
                content_h,
                started: std::time::Instant::now(),
            },
        );
        schedule_ui_tween_frame(cx);
        self.tick_ui_state_anims(cx);
    }

    /// Drive accordion height + rail width tweens; call from `Event::NextFrame`.
    fn tick_ui_state_anims(&mut self, cx: &mut Cx) {
        let now = std::time::Instant::now();
        let mut done_keys = Vec::new();
        let mut any_live = false;
        {
            let mut anims = accordion_anims().lock().unwrap();
            for (key, anim) in anims.iter_mut() {
                let Some(body) = Self::section_body_id(key) else {
                    done_keys.push(key.clone());
                    continue;
                };
                let (progress, done) = accordion_anim_progress(anim, now);
                let h = anim.content_h * progress;
                self.ui.view(cx, body).set_visible(cx, true);
                self.set_section_body_height(cx, body, Some(h));
                if done {
                    done_keys.push(key.clone());
                } else {
                    any_live = true;
                }
            }
            for key in &done_keys {
                let Some(anim) = anims.remove(key) else {
                    continue;
                };
                let Some(body) = Self::section_body_id(key) else {
                    continue;
                };
                if anim.to <= 0.0 {
                    self.ui.view(cx, body).set_visible(cx, false);
                    self.set_section_body_height(cx, body, None);
                } else {
                    self.ui.view(cx, body).set_visible(cx, true);
                    self.set_section_body_height(cx, body, None);
                }
            }
            any_live = any_live || !anims.is_empty();
        }
        if self.tick_rail_width_anim(cx, now) {
            any_live = true;
        }
        if list_height_anims_live() {
            any_live = true;
        }
        if any_live {
            schedule_ui_tween_frame(cx);
        } else {
            *accordion_frame().lock().unwrap() = None;
        }
        self.ui.redraw(cx);
    }

    /// Returns true while a rail width tween is still running.
    fn tick_rail_width_anim(&mut self, cx: &mut Cx, now: std::time::Instant) -> bool {
        let (done, to) = {
            let slot = rail_width_anim().lock().unwrap();
            let Some(anim) = slot.as_ref() else {
                return false;
            };
            let t = now.duration_since(anim.started).as_secs_f64() * 1000.0 / ACCORDION_ANIM_MS;
            (t >= 1.0, anim.to)
        };
        if done {
            *rail_width_anim().lock().unwrap() = None;
            if to > 0.0 {
                app_state().lock().unwrap().rail_width = to;
            }
        }
        let unlocked = {
            let st = app_state().lock().unwrap();
            !matches!(
                st.screen,
                Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
            )
        };
        self.apply_shell_layout(cx, unlocked);
        !done
    }

    fn start_rail_width_anim(&mut self, cx: &mut Cx, from: f64, to: f64) {
        *rail_width_anim().lock().unwrap() = Some(RailWidthAnim {
            from,
            to,
            started: std::time::Instant::now(),
        });
        schedule_ui_tween_frame(cx);
        self.tick_ui_state_anims(cx);
    }

    /// Top (screen) eye — persisted per signed-in user; applies on every screen.
    fn toggle_amounts_screen(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.amounts_screen_hidden = !st.amounts_screen_hidden;
            if let (Some(store), Some(user)) =
                (st.store.as_ref(), st.session.as_ref().map(lan_key))
            {
                let _ = store.set_amounts_screen_hidden(&user, st.amounts_screen_hidden);
            }
        }
        self.sync_chrome(cx);
    }

    /// Per-figure eye for one amount key (persisted per user):
    /// - Screen **hidden** → add / remove the key in `amount_keys_peek` (still visible).
    /// - Screen **visible** → add / remove the key in `amount_keys_hidden`.
    fn toggle_amount_key(&mut self, cx: &mut Cx, key: &str) {
        toggle_amount_key_prefs(key);
        self.sync_chrome(cx);
    }

    /// Canonical paint path for a discrete money figure + its eye control.
    /// Uses [`amount_key_visible`] + [`display_money`] — pass **only** the money
    /// string; keep the label widget separate and always visible.
    fn paint_amount_value(
        &mut self,
        cx: &mut Cx,
        value_id: &[LiveId],
        eye_id: &[LiveId],
        key: &str,
        text: &str,
    ) {
        let (screen_hidden, keys_hidden, keys_peek) = {
            let st = app_state().lock().unwrap();
            (
                st.amounts_screen_hidden,
                st.amount_keys_hidden.clone(),
                st.amount_keys_peek.clone(),
            )
        };
        let visible = amount_key_visible(screen_hidden, &keys_hidden, &keys_peek, key);
        self.ui
            .widget(cx, value_id)
            .set_text(cx, &plain_display_text(&display_money(text, visible)));
        self.ui.button(cx, eye_id).set_text(cx, eye_glyph(visible));
    }

    fn toggle_books_section(&mut self, cx: &mut Cx, key: &str) {
        if Self::section_body_id(key).is_none() {
            return;
        }
        let will_open = {
            let mut st = app_state().lock().unwrap();
            let open = if let Some(i) = st.books_sections_open.iter().position(|k| k == key) {
                st.books_sections_open.remove(i);
                false
            } else {
                st.books_sections_open.push(key.to_string());
                true
            };
            if let Some(store) = st.store.as_ref() {
                let _ = store.set_books_sections_csv(&books_sections_csv(&st.books_sections_open));
            }
            open
        };
        self.start_section_height_anim(cx, key, will_open);
        self.sync_books_sections(cx);
    }

    fn sync_device_setup_sections(&mut self, cx: &mut Cx) {
        let open = app_state()
            .lock()
            .unwrap()
            .device_setup_sections_open
            .clone();
        let is_open = |key: &str| open.iter().any(|k| k == key);
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |k: &str| locale::t(&lang, k);
        let animating = accordion_anims().lock().unwrap();
        let sections: &[(&str, &[LiveId], &[LiveId], &str)] = &[
            (
                "language",
                ids!(setup_lang_hdr),
                ids!(setup_lang_body),
                "language",
            ),
            (
                "role",
                ids!(setup_role_hdr),
                ids!(setup_role_body),
                "this_device",
            ),
        ];
        for (key, hdr, body, caption_key) in sections {
            let open = is_open(key);
            let mark = if open {
                ACCORDION_MARK_OPEN
            } else {
                ACCORDION_MARK_CLOSED
            };
            let caption = s(caption_key).to_uppercase();
            self.ui
                .button(cx, hdr)
                .set_text(cx, &format!("{mark}  {caption}"));
            if animating.contains_key(*key) {
                continue;
            }
            self.ui.view(cx, body).set_visible(cx, open);
            if let Some(mut v) = self.ui.view(cx, body).borrow_mut() {
                v.walk.height = Size::fit();
            }
        }
    }

    fn toggle_device_setup_section(&mut self, cx: &mut Cx, key: &str) {
        let will_open = {
            let mut st = app_state().lock().unwrap();
            if let Some(i) = st.device_setup_sections_open.iter().position(|k| k == key) {
                st.device_setup_sections_open.remove(i);
                false
            } else {
                st.device_setup_sections_open.push(key.to_string());
                true
            }
        };
        self.start_section_height_anim(cx, key, will_open);
        self.sync_device_setup_sections(cx);
        // Language accordion is a nested setup layer — show/hide header ‹.
        self.paint_header_back_chrome(cx);
    }

    /// Docs / debug / LOCAL badges stay out of the day-to-day work surface.
    fn hide_docs_and_debug_chrome(&mut self, cx: &mut Cx) {
        // Essay / debug WrapLabels stay off the work surface. Useful tips live
        // on InfoHelp ℹ (booking_help, buy_help, day_close_help, …).
        let help_ids: &[&[LiveId]] = &[
            ids!(h_feature_access_help),
            ids!(h_country_pack_help),
            ids!(h_language_setup_hint),
            ids!(h_currency_use_help),
            ids!(h_brand_shown_help),
            ids!(h_other_devices_help),
            ids!(h_other_devices_help_heading),
            ids!(h_units_help),
            ids!(h_measures_help_moklanjo),
            ids!(h_printers_help),
            ids!(h_restore_help),
            ids!(db_path_label),
            ids!(booking_scope_hint),
            ids!(booking_consignor_hint),
            ids!(h_mix_type_hint_moklanjo),
            ids!(cart_round_hint),
            ids!(amend_round_hint),
            ids!(approval_hint),
            ids!(h_preset_help_moklanjo),
            ids!(h_product_only_moklanjo),
            ids!(device_use_question),
            ids!(h_fresh_book_moklanjo),
            ids!(h_later_country_pack),
        ];
        for id in help_ids {
            self.ui.widget(cx, id).set_visible(cx, false);
        }
        for id in [
            ids!(h_local_badge),
            ids!(h_local_badge_2),
            ids!(h_local_badge_3),
        ] {
            self.ui.label(cx, id).set_visible(cx, false);
        }
    }

    /// Settings → Sidebar: pill toggles + Enable/Disable All.
    fn handle_sidebar_nav_list_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if app_state().lock().unwrap().screen != Screen::SidebarSettings {
            return;
        }
        if self.ui.button(cx, ids!(sidebar_enable_all)).clicked(actions) {
            self.set_all_sidebar_visibility(cx, true);
            return;
        }
        if self.ui.button(cx, ids!(sidebar_disable_all)).clicked(actions) {
            self.set_all_sidebar_visibility(cx, false);
            return;
        }
        let widget = self.ui.widget(cx, ids!(sidebar_nav_list));
        // A drag-reorder release over a toggle must not also flip visibility.
        let suppress = widget
            .as_sidebar_nav_list()
            .borrow()
            .map(|w| w.suppress_face_click)
            .unwrap_or(false);
        if suppress {
            if let Some(mut w) = widget.as_sidebar_nav_list().borrow_mut() {
                w.suppress_face_click = false;
            }
            return;
        }
        let list = widget.portal_list(cx, ids!(list));
        let order_len = app_state().lock().unwrap().nav_order.len();
        let mut changed_vis = false;
        let mut changed_order = false;
        for (item_id, item) in list.items_with_actions(actions) {
            if item_id >= order_len {
                continue;
            }
            let up = item.button(cx, ids!(move_up)).clicked(actions);
            let down = item.button(cx, ids!(move_down)).clicked(actions);
            if up || down {
                let mut st = app_state().lock().unwrap();
                let target = if up {
                    item_id.checked_sub(1)
                } else if item_id + 1 < st.nav_order.len() {
                    Some(item_id + 1)
                } else {
                    None
                };
                if let Some(target) = target {
                    st.nav_order.swap(item_id, target);
                    if let Some(store) = st.store.as_ref() {
                        let _ = store.set_nav_order_csv(&nav_order_csv(&st.nav_order));
                    }
                    changed_order = true;
                }
                break;
            }
            if !item.button(cx, ids!(toggle_hit)).clicked(actions) {
                continue;
            }
            let mut st = app_state().lock().unwrap();
            let tab = st.nav_order[item_id];
            if nav_tab_always_visible(tab) {
                continue;
            }
            if let Some(i) = st.nav_hidden.iter().position(|t| *t == tab) {
                st.nav_hidden.remove(i);
            } else {
                st.nav_hidden.push(tab);
            }
            if let Some(store) = st.store.as_ref() {
                let _ = store.set_nav_hidden_csv(&nav_hidden_csv(&st.nav_hidden));
            }
            changed_vis = true;
            break;
        }
        if changed_order {
            let msg = app_state().lock().unwrap().t("sidebar_order_saved");
            self.set_status(cx, msg);
            self.sync_chrome(cx);
        } else if changed_vis {
            let msg = app_state().lock().unwrap().t("sidebar_visibility_saved");
            self.set_status(cx, msg);
            self.sync_chrome(cx);
        }
    }

    /// Show or hide every chrome peer that is allowed to toggle
    /// (Settings and Lock stay on the bar).
    fn set_all_sidebar_visibility(&mut self, cx: &mut Cx, enable: bool) {
        {
            let mut st = app_state().lock().unwrap();
            st.nav_hidden = if enable {
                Vec::new()
            } else {
                NAV_TABS
                    .iter()
                    .copied()
                    .filter(|t| !nav_tab_always_visible(*t))
                    .collect()
            };
            if let Some(store) = st.store.as_ref() {
                let _ = store.set_nav_hidden_csv(&nav_hidden_csv(&st.nav_hidden));
            }
        }
        let msg = app_state().lock().unwrap().t("sidebar_visibility_saved");
        self.set_status(cx, msg);
        self.sync_chrome(cx);
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
        // Side buttons always consume the event (browser-style). Debounce
        // first so Linux XF86Back/Forward twins do not step twice.
        match event {
            Event::MouseDown(e) if e.button.is_back() => {
                *mouse_swipe_origin().lock().unwrap() = None;
                if nav_gesture_ok() {
                    let _ = self.navigate_back(cx);
                }
                return true;
            }
            Event::MouseDown(e) if e.button.is_forward() => {
                *mouse_swipe_origin().lock().unwrap() = None;
                if nav_gesture_ok() {
                    let _ = self.navigate_forward(cx);
                }
                return true;
            }
            _ => {}
        }
        let navigated = match event {
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
                    (Some(edge), Some((start, end))) => self.edge_swipe_nav(cx, start, end, edge),
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

    /// Runs after the UI handled a press. An edge swipe armed on that press
    /// stays armed only when nothing claimed it or the page's own scroll
    /// view did. A button, field, card, list, or sideways strip in the 48 px
    /// edge strip claimed it: the control wins. On a phone the booking desk
    /// pans sideways between Parcel and Counter, and a pan from the edge
    /// used to also go Back or Forward.
    fn disarm_edge_swipe_if_claimed(&mut self, cx: &mut Cx, event: &Event) {
        let is_press = match event {
            Event::MouseDown(e) => e.button.is_primary(),
            Event::TouchUpdate(e) => e.touches.iter().any(|t| t.state == TouchState::Start),
            _ => false,
        };
        if !is_press {
            return;
        }
        let claimed = event.pointer_claimed_area();
        if claimed.is_empty() {
            return;
        }
        let screen = app_state().lock().unwrap().screen;
        if claimed == self.ui.view(cx, Self::screen_scroll_view(screen)).area() {
            return;
        }
        match event {
            Event::MouseDown(_) => *mouse_swipe_origin().lock().unwrap() = None,
            Event::TouchUpdate(e) => {
                let mut origin = swipe_origin().lock().unwrap();
                if let Some((uid, _, _)) = *origin
                    && e.touches.iter().any(|t| t.state == TouchState::Start && t.uid == uid)
                {
                    *origin = None;
                }
            }
            _ => {}
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

    /// Server Mode (offline LAN): create the one local server if missing, else open it.
    /// No create/open/remove list — multi-server on one install is cloud-only.
    fn begin_server_mode(&mut self, cx: &mut Cx) {
        if role_open_busy().load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        self.resolve_data_base(cx);
        let result = (|| -> Result<(), String> {
            let base = {
                let mut st = app_state().lock().unwrap();
                st.picking_server = false;
                st.refresh_local_servers()?;
                st.data_base
                    .clone()
                    .ok_or_else(|| "No data directory".to_string())?
            };
            match local_servers::offline_server_id(&base)? {
                None => self.activate_new_server(cx, "")?,
                Some(id) => self.activate_existing_server(cx, &id)?,
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

    fn finish_server_new_opened(&mut self, cx: &mut Cx, store: Store) {
        self.paint_role_open_busy_chrome(cx);
        let old = {
            let mut st = app_state().lock().unwrap();
            let old = st.store.take();
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.session = None;
            st.clear_team_edit_selection();
            st.onboarding_complete = false;
            st.owner_setup_complete = false;
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            self.finish_as_server(&mut st, true);
            old
        };
        drop_store_off_ui(old);
        self.sync_chrome(cx);
        end_role_open_busy();
        self.paint_role_open_busy_chrome(cx);
        self.maybe_smoke_auto_unlock(cx);
    }
}

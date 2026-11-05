impl App {
fn print_or_share_receipt(&mut self, cx: &mut Cx, text: &str) {
        cx.copy_to_clipboard(text);
        let _ = print_share::copy_text(text);
        // Android: ACTION_SEND chooser, unless a network printer takes the
        // bill. Other platforms ignore this and open the file below.
        if !print_share::prints_here(print_share::PrintKind::Bill) {
            cx.share_text(text);
        }
        let app_files = cx.get_data_dir().map(PathBuf::from);
        match print_share::print_text(print_share::PrintKind::Bill, text, app_files.as_deref()) {
            Ok(share) => {
                let mut st = app_state().lock().unwrap();
                st.last_receipt = text.to_string();
                st.status = if share.opened {
                    format!("Receipt ready · {}", share.path.display())
                } else if cfg!(target_os = "android") {
                    format!("Share sheet opened · saved to {}", share.path.display())
                } else {
                    format!("Receipt copied · saved to {}", share.path.display())
                };
            }
            Err(e) => {
                let mut st = app_state().lock().unwrap();
                st.last_receipt = text.to_string();
                st.status = format!("Receipt copied · file not saved ({e})");
            }
        }
        self.sync_chrome(cx);
    }

    /// Drop soft-keyboard + key focus (Android: tap dish/chip/outside, Back).
    fn dismiss_soft_keyboard(&mut self, cx: &mut Cx) {
        cx.set_key_focus(Area::Empty);
        // Latch dismissed — `hide_text_ime` clears that latch and let the next
        // focused draw re-open the IME without a tap (felt stuck / random).
        cx.text_ime_was_dismissed();
        if let Ok(mut flag) = ime_suppress_refocus().lock() {
            *flag = true;
        }
    }

    /// Close an open type-or-pick list when the press lands outside its
    /// field + list (and the kitchen dropdown button). Focus-loss alone
    /// cannot close it: tapping a row steals focus before the pick lands.
    fn dismiss_suggest_on_outside_press(&mut self, cx: &mut Cx, event: &Event) {
        let abs = match event {
            Event::MouseDown(e) if e.button.is_primary() => e.abs,
            Event::TouchUpdate(e) => {
                match e
                    .touches
                    .iter()
                    .find(|t| matches!(t.state, TouchState::Start))
                {
                    Some(t) => t.abs,
                    None => return,
                }
            }
            _ => return,
        };
        let Some(field) = app_state().lock().unwrap().suggest_open else {
            return;
        };
        // Team station checklist stays open while a person is selected
        // (sync_chrome re-opens it; outside tap must not clear it).
        if field == SuggestField::TeamKotStations {
            return;
        }
        let inside = if field == SuggestField::FloorColumns {
            let r = self.ui.view(cx, ids!(floor_cols_box)).area().rect(cx);
            r.size.x > 0.0 && r.size.y > 0.0 && r.contains(abs)
        } else {
            let input = self.ui.text_input(cx, field.input()).area().rect(cx);
            let list = self.ui.widget(cx, field.list()).area().rect(cx);
            let in_input = input.size.x > 0.0 && input.contains(abs);
            let in_list = list.size.x > 0.0 && list.size.y > 0.0 && list.contains(abs);
            let in_extra = if field == SuggestField::KotStation {
                let r = self.ui.button(cx, ids!(kot_station_dd)).area().rect(cx);
                r.size.x > 0.0 && r.contains(abs)
            } else if field == SuggestField::PrinterPlace {
                let r = self.ui.button(cx, ids!(printer_place_dd)).area().rect(cx);
                r.size.x > 0.0 && r.contains(abs)
            } else if field == SuggestField::PrinterAssign {
                let r = self.ui.button(cx, ids!(printer_assign_dd)).area().rect(cx);
                r.size.x > 0.0 && r.contains(abs)
            } else {
                false
            };
            in_input || in_list || in_extra
        };
        if inside {
            return;
        }
        app_state().lock().unwrap().suggest_open = None;
        self.ui.widget(cx, field.list()).redraw(cx);
        self.ui.redraw(cx);
    }

    /// True when a seller text field (not a button) holds key focus / soft keyboard.
    fn text_field_has_focus(&mut self, cx: &mut Cx) -> bool {
        let top_level = [
            ids!(cart_total_input),
            ids!(customer_name_input),
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
            ids!(menu_search_input),
            ids!(menu_list_search),
            ids!(tendered_input),
            ids!(tip_input),
            ids!(kot_station_input),
            ids!(frequent_days_input),
            ids!(frequent_limit_input),
            ids!(amend_total_input),
            ids!(reason_input),
            ids!(approval_pin_input),
            ids!(credit_add_name),
            ids!(settle_amount_input),
            ids!(staff_name_input),
            ids!(staff_title_input),
            ids!(staff_pin_input),
            ids!(my_pin_input),
            ids!(lock_pin_input),
            ids!(lock_login_id),
            ids!(lock_login_password),
            // ClientPair — must count so Android Back dismisses IME first.
            ids!(client_host_input),
            ids!(client_code_input),
            ids!(client_name_input),
            ids!(server_name_input),
            ids!(server_remove_pin),
            ids!(venue_title_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_modifiers_input),
            ids!(inv_item_input),
            ids!(inv_qty_input),
            ids!(inv_note_input),
            ids!(org_name_input),
            ids!(branch_name_input),
            ids!(branch_code_input),
            ids!(branch_switch_input),

            ids!(country_code_input),

            ids!(business_tax_id_input),
            ids!(service_bps_input),
            ids!(onboard_venue_input),
            ids!(onboard_org_input),
            ids!(mix_cash_input),
            ids!(mix_upi_input),
            ids!(mix_card_input),
            ids!(mix_credit_input),
        ];
        for path in top_level {
            let field = self.ui.text_input(cx, path);
            let area = field.area();
            if !area.is_empty() && cx.has_key_focus(area) {
                return true;
            }
        }
        let (cart_n, amend_n) = {
            let st = app_state().lock().unwrap();
            (st.cart.lines.len(), st.amend_cart.lines.len())
        };
        for (list_path, n) in [
            (ids!(cart_list), cart_n),
            (ids!(amend_cart_list), amend_n),
        ] {
            let host = self.ui.widget(cx, list_path);
            let list = host.portal_list(cx, ids!(list));
            for i in 0..n {
                let item = list.item(cx, i, live_id!(Item));
                for field_id in [ids!(unit_price), ids!(line_total), ids!(note)] {
                    let field = item.text_input(cx, field_id);
                    let area = field.area();
                    if !area.is_empty() && cx.has_key_focus(area) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Natural back: dismiss soft-keyboard when a text field is focused,
    /// else nested in-screen layers (Settings panel → hub), else pop screen
    /// history (mouse Back / left-edge swipe / Android back).
    fn navigate_back(&mut self, cx: &mut Cx) -> bool {
        // Add dishes: Back first hides the keyboard (if search focused), then
        // closes the window — never both in one press.
        if app_state().lock().unwrap().menu_pick_open {
            if self.text_field_has_focus(cx) {
                self.dismiss_soft_keyboard(cx);
                return true;
            }
            self.close_menu_window(cx);
            return true;
        }
        // Soft keyboard: on Android outside auth/setup, one Back dismisses
        // IME first and stops. On desktop (every screen), blur then continue
        // — a focused search/PIN/rate field must not swallow mouse Back,
        // Alt+←, or header ‹ (looked broken on Lock and everywhere else).
        // Auth/setup on Android also blur then continue so Back still leaves
        // Lock / ClientPair / DeviceSetup.
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
        {
            let mut st = app_state().lock().unwrap();
            if st.suggest_open.is_some() {
                st.suggest_open = None;
                drop(st);
                self.sync_chrome(cx);
                return true;
            }
            if st.picking_server {
                st.picking_server = false;
                drop(st);
                self.sync_chrome(cx);
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
            // Order-desk layers only while on Pos (same scope as ‹ predicate).
            if st.screen == Screen::Pos && st.line_action.is_some() {
                st.line_action = None;
                drop(st);
                self.sync_chrome(cx);
                return true;
            }
            if st.screen == Screen::Pos && st.move_panel_open {
                st.move_panel_open = false;
                drop(st);
                self.sync_chrome(cx);
                return true;
            }
            if st.screen == Screen::Pos && st.split_mode {
                st.split_mode = false;
                st.split_pick.clear();
                drop(st);
                self.sync_chrome(cx);
                return true;
            }
            // Floor pick-table drill-down: cancel pick (same as "Back to order").
            if st.screen == Screen::Floor && st.picking_table {
                st.picking_table = false;
                if let Some(from) = st.picking_from_table_id.take() {
                    let _ = st.load_table_cart(&from);
                }
                drop(st);
                if !self.navigate_back(cx) {
                    self.go_peer(cx, Screen::Pos);
                }
                return true;
            }
            // Settings nested section → hub (never jump straight to Day book).
            if st.screen == Screen::Settings && st.settings_panel.is_some() {
                drop(st);
                self.open_settings_panel(cx, None);
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
                if prev != Screen::Floor {
                    let restore = if st.picking_table {
                        st.picking_from_table_id.take()
                    } else {
                        None
                    };
                    st.picking_table = false;
                    st.picking_from_table_id = None;
                    if let Some(from) = restore {
                        let _ = st.load_table_cart(&from);
                    }
                }
                if st.screen == Screen::Team && prev != Screen::Team {
                    st.team_staff_id = None;
                    if st.suggest_open == Some(SuggestField::TeamKotStations) {
                        st.suggest_open = None;
                    }
                }
                self.set_screen(&mut st, prev, false);
                if prev == Screen::Pos
                    && let Some(tid) = st.selected_table_id.clone()
                {
                    let _ = st.load_table_cart(&tid);
                }
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
                if next != Screen::Floor {
                    st.picking_table = false;
                }
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
        // Never persist maximized/fullscreen geometry — that was rewriting
        // window_geom.json to ~2560×1048 (one ultrawide), so the next launch
        // opened already monitor-sized and the next maximize fought Mutter
        // across the 2×2 layout.
        if win.is_fullscreen(cx) {
            return;
        }
        let size = win.get_inner_size(cx);
        let pos = win.get_position(cx);
        if size.x >= 1920.0 && size.y >= 900.0 {
            return;
        }
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

    fn edge_swipe_nav(
        &mut self,
        cx: &mut Cx,
        start: DVec2,
        end: DVec2,
        edge: SwipeNav,
    ) -> bool {
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
            unlocked && wide && !st.rail_collapsed
        };
        match event {
            Event::MouseDown(e) if e.button.is_primary() && unlocked_rail => {
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
            _ => false,
        }
    }

    /// Which nav slot (position) of a surface is under `pos`, if any.
    /// Which slot (if any) is under `pos`. `pad` grows the drawn face a
    /// few px — used for drop targeting, where landing in the hairline
    /// gap between two items should still count. Drag *pickup* passes
    /// 0: the bars keep whitespace lanes around the items for
    /// scrolling, and a padded pickup area would eat the lane.
    fn nav_slot_at(
        &mut self,
        cx: &mut Cx,
        rail: bool,
        pos: DVec2,
        pad_x: f64,
        pad_y: f64,
    ) -> Option<usize> {
        let pairs = if rail {
            self.rail_nav_pairs()
        } else {
            self.bottom_nav_pairs()
        };
        for (slot, (idle, on)) in pairs.iter().enumerate() {
            for face in [idle, on] {
                let rect = self.ui.button(cx, face).area().rect(cx);
                let hit = rect.size.x > 0.0
                    && rect.size.y > 0.0
                    && pos.x >= rect.pos.x - pad_x
                    && pos.x <= rect.pos.x + rect.size.x + pad_x
                    && pos.y >= rect.pos.y - pad_y
                    && pos.y <= rect.pos.y + rect.size.y + pad_y;
                if hit {
                    return Some(slot);
                }
            }
        }
        None
    }

    /// Drop target during a drag: which slot is under the window
    /// pointer, computed from the drag's frozen content rects and the
    /// bar's live scroll offset (widget areas lag during auto-scroll,
    /// so they cannot be trusted here). Held past the first/last item
    /// — e.g. against the bar's edge while it auto-scrolls — the
    /// nearest end item is the target.
    fn nav_drop_slot(&mut self, cx: &mut Cx, drag: &NavDrag, pos: DVec2) -> Option<usize> {
        let scroll = if drag.rail {
            self.ui.view(cx, ids!(rail_scroll)).get_scroll_pos()
        } else {
            self.ui.view(cx, ids!(bottom_nav_scroll)).get_scroll_pos()
        };
        let pos = DVec2 {
            x: pos.x + scroll.x,
            y: pos.y + scroll.y,
        };
        let mut first_rect: Option<Rect> = None;
        let mut last_rect: Option<Rect> = None;
        for (slot, rect) in drag.rects.iter().enumerate() {
            if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
                continue;
            }
            if first_rect.is_none() {
                first_rect = Some(*rect);
            }
            last_rect = Some(*rect);
            let hit = pos.x >= rect.pos.x - 2.0
                && pos.x <= rect.pos.x + rect.size.x + 2.0
                && pos.y >= rect.pos.y - 5.0
                && pos.y <= rect.pos.y + rect.size.y + 5.0;
            if hit {
                return Some(slot);
            }
        }
        if let (Some(first), Some(last)) = (first_rect, last_rect) {
            let last_slot = drag.rects.len() - 1;
            if drag.rail {
                let in_band = pos.x >= first.pos.x - 28.0
                    && pos.x <= first.pos.x + first.size.x + 28.0;
                if in_band && pos.y >= last.pos.y + last.size.y {
                    return Some(last_slot);
                }
                if in_band && pos.y <= first.pos.y {
                    return Some(0);
                }
            } else {
                let in_band = pos.y >= first.pos.y - 28.0
                    && pos.y <= first.pos.y + first.size.y + 28.0;
                if in_band && pos.x >= last.pos.x + last.size.x {
                    return Some(last_slot);
                }
                if in_band && pos.x <= first.pos.x {
                    return Some(0);
                }
            }
        }
        None
    }

    fn repaint_nav_tabs(&mut self, cx: &mut Cx) {
        let screen = app_state().lock().unwrap().screen;
        self.paint_nav_tabs(cx, screen);
    }

    /// Books home accordion bodies + header chevrons from `books_sections_open`.
    /// Skips visibility/height while a section is mid-animation.
    /// Waiters / no-Sales sessions never see money sections (same as today bar).
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
                "bills",
                ids!(books_sec_bills_hdr),
                ids!(books_sec_bills_body),
                "caption_bills",
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

    fn books_section_body_id(key: &str) -> Option<&'static [LiveId]> {
        match key {
            "today" => Some(ids!(books_sec_today_body)),
            "bills" => Some(ids!(books_sec_bills_body)),
            "payments" => Some(ids!(books_sec_payments_body)),
            "outstanding" => Some(ids!(books_sec_outstanding_body)),
            "bank" => Some(ids!(books_sec_bank_body)),
            _ => None,
        }
    }

    fn set_books_section_body_height(&mut self, cx: &mut Cx, body: &[LiveId], height: Option<f64>) {
        if let Some(mut v) = self.ui.view(cx, body).borrow_mut() {
            v.walk.height = match height {
                Some(h) => Size::Fixed(h.max(0.0)),
                None => Size::fit(),
            };
        }
        self.ui.view(cx, body).redraw(cx);
    }

    /// Drive accordion height animations; call from `Event::NextFrame`.
    fn tick_books_accordion(&mut self, cx: &mut Cx) {
        let now = std::time::Instant::now();
        let mut done_keys = Vec::new();
        let mut any_live = false;
        {
            let mut anims = accordion_anims().lock().unwrap();
            for (key, anim) in anims.iter_mut() {
                let Some(body) = Self::books_section_body_id(key) else {
                    done_keys.push(key.clone());
                    continue;
                };
                let t = now.duration_since(anim.started).as_secs_f64() * 1000.0 / ACCORDION_ANIM_MS;
                let p = accordion_ease(t);
                let progress = anim.from + (anim.to - anim.from) * p;
                let h = anim.content_h * progress;
                self.ui.view(cx, body).set_visible(cx, true);
                self.set_books_section_body_height(cx, body, Some(h));
                if t >= 1.0 {
                    done_keys.push(key.clone());
                } else {
                    any_live = true;
                }
            }
            for key in &done_keys {
                let Some(anim) = anims.remove(key) else {
                    continue;
                };
                let Some(body) = Self::books_section_body_id(key) else {
                    continue;
                };
                if anim.to <= 0.0 {
                    self.ui.view(cx, body).set_visible(cx, false);
                    self.set_books_section_body_height(cx, body, None);
                } else {
                    self.ui.view(cx, body).set_visible(cx, true);
                    self.set_books_section_body_height(cx, body, None);
                }
            }
            any_live = any_live || !anims.is_empty();
        }
        if any_live {
            *accordion_frame().lock().unwrap() = Some(cx.new_next_frame());
        } else {
            *accordion_frame().lock().unwrap() = None;
        }
        self.ui.redraw(cx);
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
        let Some(body) = Self::books_section_body_id(key) else {
            return;
        };
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
        let measured = self.ui.view(cx, body).area().rect(cx).size.y;
        let content_h = if measured > 1.0 {
            measured
        } else {
            accordion_estimate_h(key)
        };
        let (from, to) = if will_open { (0.0, 1.0) } else { (1.0, 0.0) };
        if will_open {
            self.ui.view(cx, body).set_visible(cx, true);
            self.set_books_section_body_height(cx, body, Some(0.0));
        } else {
            self.set_books_section_body_height(cx, body, Some(content_h));
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
        *accordion_frame().lock().unwrap() = Some(cx.new_next_frame());
        self.sync_books_sections(cx);
        self.tick_books_accordion(cx);
    }

    fn sync_floor_sections(&mut self, cx: &mut Cx) {
        let open = app_state().lock().unwrap().floor_sections_open.clone();
        let is_open = |key: &str| open.iter().any(|k| k == key);
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |k: &str| locale::t(&lang, k);
        let sections: &[(&str, &[LiveId], &[LiveId], &str)] = &[
            (
                "floor_ops",
                ids!(floor_ops_hdr),
                ids!(floor_ops_body),
                "ops_panel",
            ),
            (
                "waitlist",
                ids!(waitlist_hdr),
                ids!(waitlist_body),
                "waitlist",
            ),
            ("resv", ids!(resv_hdr), ids!(resv_body), "resv_heading"),
            (
                "floor_add",
                ids!(floor_add_hdr),
                ids!(floor_add_body),
                "add_a_table",
            ),
            (
                "stations",
                ids!(stations_hdr),
                ids!(stations_body),
                "stations_heading",
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
            self.ui.view(cx, body).set_visible(cx, open);
        }
    }

    fn toggle_floor_section(&mut self, cx: &mut Cx, key: &str) {
        {
            let mut st = app_state().lock().unwrap();
            if let Some(i) = st.floor_sections_open.iter().position(|k| k == key) {
                st.floor_sections_open.remove(i);
            } else {
                st.floor_sections_open.push(key.to_string());
            }
        }
        self.sync_floor_sections(cx);
        self.ui.redraw(cx);
    }

    fn sync_order_sections(&mut self, cx: &mut Cx) {
        let open = app_state().lock().unwrap().order_sections_open.clone();
        let is_open = |key: &str| open.iter().any(|k| k == key);
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |k: &str| locale::t(&lang, k);
        let open = is_open("table_actions");
        let mark = if open {
            ACCORDION_MARK_OPEN
        } else {
            ACCORDION_MARK_CLOSED
        };
        let caption = s("table_actions").to_uppercase();
        self.ui
            .button(cx, ids!(table_actions_hdr))
            .set_text(cx, &format!("{mark}  {caption}"));
        self.ui
            .view(cx, ids!(table_actions_body))
            .set_visible(cx, open);
    }

    fn toggle_order_section(&mut self, cx: &mut Cx, key: &str) {
        {
            let mut st = app_state().lock().unwrap();
            if let Some(i) = st.order_sections_open.iter().position(|k| k == key) {
                st.order_sections_open.remove(i);
            } else {
                st.order_sections_open.push(key.to_string());
            }
        }
        self.sync_order_sections(cx);
        self.ui.redraw(cx);
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
            self.ui.view(cx, body).set_visible(cx, open);
        }
    }

    fn toggle_device_setup_section(&mut self, cx: &mut Cx, key: &str) {
        {
            let mut st = app_state().lock().unwrap();
            if let Some(i) = st.device_setup_sections_open.iter().position(|k| k == key) {
                st.device_setup_sections_open.remove(i);
            } else {
                st.device_setup_sections_open.push(key.to_string());
            }
        }
        self.sync_device_setup_sections(cx);
        // Language accordion is a nested setup layer — show/hide header ‹.
        self.paint_header_back_chrome(cx);
        self.ui.redraw(cx);
    }

    /// Docs / debug / LOCAL badges stay out of the day-to-day work surface.
    /// Useful tips use InfoHelp ℹ (floor_help, kitchen_help, settings_lan_help, …).
    fn hide_docs_and_debug_chrome(&mut self, cx: &mut Cx) {
        let help_ids: &[&[LiveId]] = &[
            ids!(h_other_devices_help),
            ids!(h_other_devices_help_heading),
            // h_duress_help is InfoHelp ℹ — Owner must see it on Team (vision §17).
            ids!(h_optional_login_help),
            ids!(h_feature_access_help),
            ids!(h_rights_help),
            ids!(crm_help),
            ids!(removed_help),
            ids!(h_units_help),
            ids!(h_units_picker_help),
            ids!(sibling_pickers_help),
            ids!(h_language_setup_hint),
            ids!(h_currency_use_help),
            ids!(h_country_pack_help),
            ids!(h_brand_shown_help),
            ids!(h_printers_help),
            ids!(h_printers_routes_help),
            ids!(h_bill_format_help),
            ids!(h_restore_help),
            ids!(categories_help),
            ids!(split_help),
            ids!(move_help),
            ids!(order_help),
            ids!(upi_qr_help),
            ids!(h_mix_type_hint_khaynjo),
            ids!(cart_round_hint),
            ids!(amend_round_hint),
            ids!(approval_hint),
            ids!(frequent_config_help),
            ids!(db_path_label),
            ids!(h_product_only_khaynjo),
            ids!(device_use_question),
            ids!(h_fresh_book_khaynjo),
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
        for (item_id, item) in list.items_with_actions(actions) {
            if item_id >= order_len {
                continue;
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
        if changed_vis {
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

    /// Display label a nav slot wears for `tab` (same strings as the
    /// chrome paint in paint_chrome).
    fn nav_tab_label(&self, tab: NavTab) -> String {
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |key: &str| locale::t(&lang, key);
        match tab {
            NavTab::Books => s("books"),
            NavTab::Sell => s("sell"),
            NavTab::Buy => s("buy"),
            NavTab::Menu => s("nav_items"),
            NavTab::Floor => s("floor"),
            NavTab::Kitchen => s("kitchen"),
            NavTab::Sales => s("sales"),
            NavTab::DayClose => s("day_close"),
            NavTab::Reports => s("nav_reports"),
            NavTab::Inventory => s("inventory"),
            NavTab::Branches => s("branches"),
            NavTab::Credit => s("credit"),
            NavTab::Team => s("team"),
            NavTab::Settings => s("nav_settings"),
            NavTab::Lock => s("nav_lock"),
        }
    }

    /// Rect of the face currently drawn for `slot` (idle or active).
    fn nav_slot_rect(&mut self, cx: &mut Cx, rail: bool, slot: usize) -> Option<Rect> {
        let pairs = if rail {
            self.rail_nav_pairs()
        } else {
            self.bottom_nav_pairs()
        };
        let (idle, on) = pairs[slot];
        for face in [idle, on] {
            let rect = self.ui.button(cx, face).area().rect(cx);
            if rect.size.x > 0.0 && rect.size.y > 0.0 {
                return Some(Rect {
                    pos: DVec2 {
                        x: rect.pos.x,
                        y: rect.pos.y,
                    },
                    size: rect.size,
                });
            }
        }
        None
    }

    fn place_nav_ghost(&mut self, cx: &mut Cx, pos: DVec2) {
        let ghost = self.ui.widget(cx, ids!(nav_drag_ghost));
        if let Some(mut view) = ghost.borrow_mut::<View>() {
            view.walk.abs_pos = Some(pos);
        }
        ghost.redraw(cx);
    }

    fn show_nav_ghost(&mut self, cx: &mut Cx, tab: NavTab, pos: DVec2) {
        let label = nav_tab_caption(tab, &self.nav_tab_label(tab));
        self.ui
            .widget(cx, ids!(nav_drag_ghost_label))
            .set_text(cx, &label);
        let icon = self.ui.image(cx, ids!(nav_drag_ghost_icon));
        icon.set_visible(cx, true);
        load_nav_icon_once(cx, &icon, tab);
        self.place_nav_ghost(cx, pos);
        self.ui.view(cx, ids!(nav_drag_ghost)).set_visible(cx, true);
        // The ghost lives in a Modal so it paints on the overlay draw
        // list, above every scroll view (rail, page, bottom bar alike).
        self.ui.modal(cx, ids!(nav_drag_modal)).open(cx);
        cx.redraw_all();
    }

    /// Floor area chip ghost — same modal as nav, label only (no peer icon).
    fn show_floor_area_ghost(&mut self, cx: &mut Cx, label: &str, pos: DVec2) {
        self.ui
            .widget(cx, ids!(nav_drag_ghost_label))
            .set_text(cx, label);
        self.ui
            .image(cx, ids!(nav_drag_ghost_icon))
            .set_visible(cx, false);
        self.place_nav_ghost(cx, pos);
        self.ui.view(cx, ids!(nav_drag_ghost)).set_visible(cx, true);
        self.ui.modal(cx, ids!(nav_drag_modal)).open(cx);
        cx.redraw_all();
    }

    fn hide_nav_ghost(&mut self, cx: &mut Cx) {
        self.ui.modal(cx, ids!(nav_drag_modal)).close(cx);
        self.ui.view(cx, ids!(nav_drag_ghost)).set_visible(cx, false);
    }

    /// Settings → Sidebar list: same ghost-chip drag as the rail/bottom
    /// bar. Press on a row, move past the threshold → floating chip with
    /// icon+name; drop on another row to swap. Tap (no move) still
    /// toggles show/hide via the face button.
    /// Settings → Sidebar: drag a row's face to reorder (ghost chip), same
    /// contract as the rail/bottom bar. Touch and mouse share one path —
    /// Android only gets `TouchUpdate`.

    fn sidebar_settings_drag_press(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) {
                *settings_nav_drag().lock().unwrap() = None;
                if app_state().lock().unwrap().screen != Screen::SidebarSettings {
                    return;
                }
                // Press on the in-row toggle is a flip, not a reorder.
                if self.sidebar_settings_press_on_toggle(cx, abs) {
                    return;
                }
                if let Some(slot) = self.sidebar_settings_slot_at(cx, abs) {
                    *nav_drag().lock().unwrap() = None;
                    self.hide_nav_ghost(cx);
                    let origin = self
                        .sidebar_settings_row_rect(cx, slot)
                        .map(|rect| rect.pos)
                        .unwrap_or(abs);
                    let scroll = self
                        .ui
                        .view(cx, ids!(sidebar_settings_screen))
                        .get_scroll_pos();
                    let n = app_state().lock().unwrap().nav_order.len();
                    let mut rects = Vec::with_capacity(n);
                    for i in 0..n {
                        let mut found = Rect::default();
                        if let Some(rect) = self.sidebar_settings_row_rect(cx, i) {
                            found = Rect {
                                pos: DVec2 {
                                    x: rect.pos.x + scroll.x,
                                    y: rect.pos.y + scroll.y,
                                },
                                size: rect.size,
                            };
                        }
                        rects.push(found);
                    }
                    *settings_nav_drag().lock().unwrap() = Some(SettingsNavDrag {
                        slot,
                        start: abs,
                        last: abs,
                        origin,
                        rects,
                        active: false,
                        hover: Some(slot),
                        finger,
                    });
                }
                // armed; tap still goes to the button
                
    }

    fn sidebar_settings_drag_move(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) -> bool {
                let mut active = false;
                let mut ghost: Option<(bool, NavTab, DVec2)> = None;
                let mut suppress_list = false;
                {
                    let mut guard = settings_nav_drag().lock().unwrap();
                    if let Some(d) = guard.as_mut() {
                        if d.finger != finger {
                            return false;
                        }
                        let was_active = d.active;
                        if !d.active && (abs - d.start).length() > 10.0 {
                            d.active = true;
                            suppress_list = true;
                        }
                        if d.active {
                            active = true;
                            d.last = abs;
                            d.hover = self.sidebar_settings_drop_slot(cx, d, abs);
                            let tab = {
                                app_state()
                                    .lock()
                                    .unwrap()
                                    .nav_order
                                    .get(d.slot)
                                    .copied()
                            };
                            if let Some(tab) = tab {
                                ghost = Some((!was_active, tab, d.origin + (abs - d.start)));
                            }
                        }
                    }
                }
                if suppress_list {
                    let widget = self.ui.widget(cx, ids!(sidebar_nav_list));
                    if let Some(mut w) = widget.as_sidebar_nav_list().borrow_mut() {
                        w.suppress_face_click = true;
                    }
                }
                if active {
                    cx.set_cursor(MouseCursor::Grabbing);
                }
                if let Some((first, tab, pos)) = ghost {
                    if first {
                        self.show_nav_ghost(cx, tab, pos);
                    } else {
                        self.place_nav_ghost(cx, pos);
                    }
                }
                return active;
    }

    fn sidebar_settings_drag_release(&mut self, cx: &mut Cx, _abs: DVec2, finger: Option<u64>) -> bool {
                let taken = {
                    let mut guard = settings_nav_drag().lock().unwrap();
                    match guard.as_ref() {
                        Some(d) if d.finger == finger => guard.take(),
                        _ => None,
                    }
                };
                if let Some(d) = taken {
                    self.hide_nav_ghost(cx);
                    if d.active {
                        cx.set_cursor(MouseCursor::Default);
                        let swapped = match d.hover {
                            Some(target) if target != d.slot => {
                                let mut st = app_state().lock().unwrap();
                                if d.slot < st.nav_order.len() && target < st.nav_order.len() {
                                    st.nav_order.swap(d.slot, target);
                                    if let Some(store) = st.store.as_ref() {
                                        let _ =
                                            store.set_nav_order_csv(&nav_order_csv(&st.nav_order));
                                    }
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        };
                        if swapped {
                            let msg = app_state().lock().unwrap().t("sidebar_order_saved");
                            self.set_status(cx, msg);
                            self.sync_chrome(cx);
                        }
                        return true;
                    }
                }
                false
    }


    fn nav_drag_press(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) {
                *nav_drag().lock().unwrap() = None;
                *floor_area_drag().lock().unwrap() = None;
                // Do not clear settings_nav_drag here — that peek runs
                // first and may have just armed a press on the list.
                // A fresh press starts a fresh gesture — never let a
                // leftover scroll-suppression eat this tap.
                *nav_suppress_click().lock().unwrap() = false;
                // Only hide the ghost when this press is not already a
                // settings-list arm (otherwise we'd flash the chip away).
                if settings_nav_drag().lock().unwrap().is_none() {
                    self.hide_nav_ghost(cx);
                }
                let unlocked = {
                    let st = app_state().lock().unwrap();
                    !matches!(
                        st.screen,
                        Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                    )
                };
                if !unlocked {
                    return;
                }
                // Only the visible surface can be dragged (mirrors
                // apply_shell_layout's rail-vs-bottom choice).
                let shell_wide =
                    self.window_inner_width(cx) >= store::TABLET_SHELL_BREAKPOINT_CSS;
                let rail_collapsed = app_state().lock().unwrap().rail_collapsed;
                if shell_wide && rail_collapsed {
                    return;
                }
                let rail = shell_wide;
                // The rail divider owns its own drag (resize) — stay out of it.
                if rail {
                    let rail_w = app_state().lock().unwrap().rail_width;
                    if abs.x >= rail_w - 2.0 && abs.x <= rail_w + 14.0 {
                        return;
                    }
                }
                if let Some(slot) = self.nav_slot_at(cx, rail, abs, 0.0, 0.0) {
                    *settings_nav_drag().lock().unwrap() = None;
                    self.hide_nav_ghost(cx);
                    let origin = self
                        .nav_slot_rect(cx, rail, slot)
                        .map(|rect| rect.pos)
                        .unwrap_or(abs);
                    // Freeze every slot's rect in content coordinates
                    // (drawn rect + scroll right now, while settled);
                    // drag-time hit tests use these, not live areas.
                    let scroll = if rail {
                        self.ui.view(cx, ids!(rail_scroll)).get_scroll_pos()
                    } else {
                        self.ui.view(cx, ids!(bottom_nav_scroll)).get_scroll_pos()
                    };
                    let pairs = if rail {
                        self.rail_nav_pairs()
                    } else {
                        self.bottom_nav_pairs()
                    };
                    let mut rects = Vec::with_capacity(pairs.len());
                    for (idle, on) in &pairs {
                        let mut found = Rect::default();
                        for face in [idle, on] {
                            let rect = self.ui.button(cx, face).area().rect(cx);
                            if rect.size.x > 0.0 && rect.size.y > 0.0 {
                                found = Rect {
                                    pos: DVec2 {
                                        x: rect.pos.x + scroll.x,
                                        y: rect.pos.y + scroll.y,
                                    },
                                    size: rect.size,
                                };
                                break;
                            }
                        }
                        rects.push(found);
                    }
                    *nav_drag().lock().unwrap() = Some(NavDrag {
                        rail,
                        slot,
                        start: abs,
                        last: abs,
                        origin,
                        rects,
                        active: false,
                        hover: Some(slot),
                        finger,
                    });
                }
                // armed; tap still goes to the button
                
    }

    fn nav_drag_move(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) -> bool {
                let mut repaint = false;
                let mut active = false;
                let mut ghost: Option<(bool, NavTab, DVec2)> = None;
                {
                    let mut guard = nav_drag().lock().unwrap();
                    if let Some(d) = guard.as_mut() {
                        if d.finger != finger {
                            return false;
                        }
                        let was_active = d.active;
                        if !d.active && (abs - d.start).length() > 10.0 {
                            d.active = true;
                        }
                        if d.active {
                            active = true;
                            d.last = abs;
                            if !was_active {
                                // Start the per-frame edge auto-scroll
                                // loop for this drag.
                                *nav_autoscroll_frame().lock().unwrap() =
                                    Some(cx.new_next_frame());
                            }
                            let hover = self.nav_drop_slot(cx, d, abs);
                            if hover != d.hover {
                                d.hover = hover;
                                repaint = true;
                            }
                            // Same order the bar paints (rights-filtered), not
                            // the full preference list — else the ghost label
                            // is the wrong peer on a waiter / Android till.
                            let tab = current_nav_chrome_order().get(d.slot).copied();
                            if let Some(tab) = tab {
                                ghost = Some((
                                    !was_active,
                                    tab,
                                    d.origin + (abs - d.start),
                                ));
                            }
                        }
                    }
                }
                if active {
                    // Drag icon for the whole drag — the button's Hand
                    // hover cursor would otherwise stay on screen.
                    cx.set_cursor(MouseCursor::Grabbing);
                }
                if let Some((first, tab, pos)) = ghost {
                    if first {
                        self.show_nav_ghost(cx, tab, pos);
                    } else {
                        self.place_nav_ghost(cx, pos);
                    }
                }
                if repaint {
                    self.repaint_nav_tabs(cx);
                }
                return active;
    }

    fn nav_drag_release(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) -> bool {
                // Take in its own scope: an `if let` scrutinee guard would
                // stay locked across the block and deadlock the repaint
                // below (paint_nav_tabs reads the same mutex).
                let taken = {
                    let mut guard = nav_drag().lock().unwrap();
                    match guard.as_ref() {
                        Some(d) if d.finger == finger => guard.take(),
                        _ => None,
                    }
                };
                if let Some(mut d) = taken {
                    self.hide_nav_ghost(cx);
                    if d.active {
                        // Touch often ends with Stop and no Move on the drop
                        // slot — recompute the target from the lift point.
                        let drop_at = if abs.x != 0.0 || abs.y != 0.0 {
                            abs
                        } else {
                            d.last
                        };
                        d.hover = self.nav_drop_slot(cx, &d, drop_at);
                        // Drag over: back to the nav button's pointer when
                        // still over a slot, plain arrow anywhere else.
                        cx.set_cursor(if d.hover.is_some() {
                            MouseCursor::Hand
                        } else {
                            MouseCursor::Default
                        });
                        // Slots are chrome indices (what the bar shows), not
                        // raw preference-order indices — using display order
                        // here swapped the wrong peers and looked like a
                        // no-op on Android waiter tills.
                        let chrome = current_nav_chrome_order();
                        let swapped = match d.hover {
                            Some(target) if target != d.slot => {
                                let Some(ta) = chrome.get(d.slot).copied() else {
                                    *nav_suppress_click().lock().unwrap() = false;
                                    self.sync_chrome(cx);
                                    return true;
                                };
                                let Some(tb) = chrome.get(target).copied() else {
                                    *nav_suppress_click().lock().unwrap() = false;
                                    self.sync_chrome(cx);
                                    return true;
                                };
                                let mut st = app_state().lock().unwrap();
                                let Some(ia) = st.nav_order.iter().position(|t| *t == ta) else {
                                    drop(st);
                                    *nav_suppress_click().lock().unwrap() = false;
                                    self.sync_chrome(cx);
                                    return true;
                                };
                                let Some(ib) = st.nav_order.iter().position(|t| *t == tb) else {
                                    drop(st);
                                    *nav_suppress_click().lock().unwrap() = false;
                                    self.sync_chrome(cx);
                                    return true;
                                };
                                st.nav_order.swap(ia, ib);
                                if let Some(store) = st.store.as_ref() {
                                    let _ =
                                        store.set_nav_order_csv(&nav_order_csv(&st.nav_order));
                                }
                                true
                            }
                            _ => false,
                        };
                        if swapped {
                            self.set_status(cx, "Menu order saved");
                        }
                        // The release is consumed, so no Button click is
                        // generated — nothing to suppress afterwards.
                        *nav_suppress_click().lock().unwrap() = false;
                        self.sync_chrome(cx);
                        return true;
                    }
                }
                false
    }

    fn pointer_sidebar_settings_drag(&mut self, cx: &mut Cx, event: &Event) -> bool {
        enum Phase {
            Press(DVec2, Option<u64>),
            Move(DVec2, Option<u64>),
            Release(DVec2, Option<u64>),
        }
        let phases: Vec<Phase> = match event {
            Event::MouseDown(e) if e.button.is_primary() => {
                vec![Phase::Press(e.abs, None)]
            }
            Event::MouseMove(e) => vec![Phase::Move(e.abs, None)],
            Event::MouseUp(e) if e.button.is_primary() => {
                vec![Phase::Release(e.abs, None)]
            }
            Event::TouchUpdate(e) => {
                let mut out = Vec::new();
                for t in &e.touches {
                    match t.state {
                        TouchState::Start => out.push(Phase::Press(t.abs, Some(t.uid))),
                        TouchState::Move | TouchState::Stable => {
                            out.push(Phase::Move(t.abs, Some(t.uid)))
                        }
                        TouchState::Stop => out.push(Phase::Release(t.abs, Some(t.uid))),
                    }
                }
                out
            }
            _ => return false,
        };
        let mut consumed = false;
        for phase in phases {
            match phase {
                Phase::Press(abs, finger) => {
                    self.sidebar_settings_drag_press(cx, abs, finger);
                }
                Phase::Move(abs, finger) => {
                    if self.sidebar_settings_drag_move(cx, abs, finger) {
                        consumed = true;
                    }
                }
                Phase::Release(abs, finger) => {
                    if self.sidebar_settings_drag_release(cx, abs, finger) {
                        consumed = true;
                    }
                }
            }
        }
        consumed
    }


    fn sidebar_settings_row_rect(&mut self, cx: &mut Cx, index: usize) -> Option<Rect> {
        let list = self
            .ui
            .widget(cx, ids!(sidebar_nav_list))
            .portal_list(cx, ids!(list));
        let item = list.item(cx, index, id!(Item));
        let rect = item.button(cx, ids!(face)).area().rect(cx);
        if rect.size.x > 0.0 && rect.size.y > 0.0 {
            Some(rect)
        } else {
            None
        }
    }

    fn sidebar_settings_press_on_toggle(&mut self, cx: &mut Cx, abs: DVec2) -> bool {
        let list = self
            .ui
            .widget(cx, ids!(sidebar_nav_list))
            .portal_list(cx, ids!(list));
        let n = app_state().lock().unwrap().nav_order.len();
        for i in 0..n {
            let item = list.item(cx, i, id!(Item));
            let rect = item.button(cx, ids!(toggle_hit)).area().rect(cx);
            if rect.size.x > 0.0 && rect.size.y > 0.0 && rect.contains(abs) {
                return true;
            }
        }
        false
    }

    fn sidebar_settings_slot_at(&mut self, cx: &mut Cx, abs: DVec2) -> Option<usize> {
        let n = app_state().lock().unwrap().nav_order.len();
        for i in 0..n {
            if let Some(rect) = self.sidebar_settings_row_rect(cx, i) {
                if rect.contains(abs) {
                    return Some(i);
                }
            }
        }
        None
    }

    fn sidebar_settings_drop_slot(
        &mut self,
        cx: &mut Cx,
        drag: &SettingsNavDrag,
        pos: DVec2,
    ) -> Option<usize> {
        let scroll = self
            .ui
            .view(cx, ids!(sidebar_settings_screen))
            .get_scroll_pos();
        let pos = DVec2 {
            x: pos.x + scroll.x,
            y: pos.y + scroll.y,
        };
        let mut first_rect: Option<Rect> = None;
        let mut last_rect: Option<Rect> = None;
        for (slot, rect) in drag.rects.iter().enumerate() {
            if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
                continue;
            }
            if first_rect.is_none() {
                first_rect = Some(*rect);
            }
            last_rect = Some(*rect);
            let hit = pos.x >= rect.pos.x - 2.0
                && pos.x <= rect.pos.x + rect.size.x + 2.0
                && pos.y >= rect.pos.y - 5.0
                && pos.y <= rect.pos.y + rect.size.y + 5.0;
            if hit {
                return Some(slot);
            }
        }
        if let (Some(first), Some(last)) = (first_rect, last_rect) {
            let last_slot = drag.rects.len() - 1;
            let in_band = pos.x >= first.pos.x - 28.0 && pos.x <= first.pos.x + first.size.x + 28.0;
            if in_band && pos.y >= last.pos.y + last.size.y {
                return Some(last_slot);
            }
            if in_band && pos.y <= first.pos.y {
                return Some(0);
            }
        }
        None
    }

    /// Drag a sidebar / bottom-bar item onto another one to swap them.
    /// Scroll and drag are separated spatially, not by timing: the bars
    /// keep whitespace lanes around the items (padding and gaps inside
    /// the scroll view), so a press on whitespace belongs to the scroll
    /// view and never reaches this peek — it starts only when the press
    /// lands on an item. From an item, a plain click navigates and any
    /// real movement picks the item up; from then on the drag owns the
    /// gesture so the scroll view cannot scroll the drop target away
    /// mid-drag. The new arrangement is saved into the books, so it
    /// survives reinstall.
    /// Works for mouse (desktop) and touch (Android): phones only get
    /// `TouchUpdate`, so the same press/move/release path runs for both.
    /// Returns true when the event was consumed by the drag.
    fn pointer_nav_drag(&mut self, cx: &mut Cx, event: &Event) -> bool {
        enum Phase {
            Press(DVec2, Option<u64>),
            Move(DVec2, Option<u64>),
            Release(DVec2, Option<u64>),
        }
        let phases: Vec<Phase> = match event {
            Event::MouseDown(e) if e.button.is_primary() => {
                vec![Phase::Press(e.abs, None)]
            }
            Event::MouseMove(e) => vec![Phase::Move(e.abs, None)],
            Event::MouseUp(e) if e.button.is_primary() => {
                vec![Phase::Release(e.abs, None)]
            }
            Event::TouchUpdate(e) => {
                let mut out = Vec::new();
                for t in &e.touches {
                    match t.state {
                        TouchState::Start => out.push(Phase::Press(t.abs, Some(t.uid))),
                        TouchState::Move | TouchState::Stable => {
                            out.push(Phase::Move(t.abs, Some(t.uid)))
                        }
                        TouchState::Stop => out.push(Phase::Release(t.abs, Some(t.uid))),
                    }
                }
                out
            }
            _ => return false,
        };
        let mut consumed = false;
        for phase in phases {
            match phase {
                Phase::Press(abs, finger) => {
                    self.nav_drag_press(cx, abs, finger);
                }
                Phase::Move(abs, finger) => {
                    if self.nav_drag_move(cx, abs, finger) {
                        consumed = true;
                    }
                }
                Phase::Release(abs, finger) => {
                    if self.nav_drag_release(cx, abs, finger) {
                        consumed = true;
                    }
                }
            }
        }
        consumed
    }

    /// Slot under the pointer on the Floor area ChipStrip (exact hit, pad 0 —
    /// same rule as nav: padding would turn lane swipes into swaps).
    fn floor_area_slot_at(&mut self, cx: &mut Cx, abs: DVec2) -> Option<usize> {
        let n = self.floor_area_tab_names().len().min(FLOOR_AREA_CHIPS.len());
        for (i, (idle, on)) in FLOOR_AREA_CHIPS.iter().enumerate().take(n) {
            for face in [idle, on] {
                let rect = self.ui.button(cx, face).area().rect(cx);
                if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
                    continue;
                }
                if abs.x >= rect.pos.x
                    && abs.x <= rect.pos.x + rect.size.x
                    && abs.y >= rect.pos.y
                    && abs.y <= rect.pos.y + rect.size.y
                {
                    return Some(i);
                }
            }
        }
        None
    }

    fn floor_area_drop_slot(&mut self, cx: &mut Cx, drag: &FloorAreaDrag, pos: DVec2) -> Option<usize> {
        let scroll = self
            .ui
            .view(cx, ids!(floor_area_strip.tabs))
            .get_scroll_pos();
        let pos = DVec2 {
            x: pos.x + scroll.x,
            y: pos.y + scroll.y,
        };
        let mut first_rect: Option<Rect> = None;
        let mut last_rect: Option<Rect> = None;
        for (slot, rect) in drag.rects.iter().enumerate() {
            if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
                continue;
            }
            if first_rect.is_none() {
                first_rect = Some(*rect);
            }
            last_rect = Some(*rect);
            let hit = pos.x >= rect.pos.x - 2.0
                && pos.x <= rect.pos.x + rect.size.x + 2.0
                && pos.y >= rect.pos.y - 5.0
                && pos.y <= rect.pos.y + rect.size.y + 5.0;
            if hit {
                return Some(slot);
            }
        }
        if let (Some(first), Some(last)) = (first_rect, last_rect) {
            let last_slot = drag.rects.len().saturating_sub(1);
            let in_band = pos.y >= first.pos.y - 28.0
                && pos.y <= first.pos.y + first.size.y + 28.0;
            if in_band && pos.x >= last.pos.x + last.size.x {
                return Some(last_slot);
            }
            if in_band && pos.x <= first.pos.x {
                return Some(0);
            }
        }
        None
    }

    fn floor_area_drag_press(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) {
        *floor_area_drag().lock().unwrap() = None;
        if app_state().lock().unwrap().screen != Screen::Floor {
            return;
        }
        if let Some(slot) = self.floor_area_slot_at(cx, abs) {
            *nav_drag().lock().unwrap() = None;
            *settings_nav_drag().lock().unwrap() = None;
            self.hide_nav_ghost(cx);
            let scroll = self
                .ui
                .view(cx, ids!(floor_area_strip.tabs))
                .get_scroll_pos();
            let mut origin = abs;
            let mut rects = Vec::with_capacity(FLOOR_AREA_CHIPS.len());
            let n = self.floor_area_tab_names().len().min(FLOOR_AREA_CHIPS.len());
            for (i, (idle, on)) in FLOOR_AREA_CHIPS.iter().enumerate().take(n) {
                let mut found = Rect::default();
                for face in [idle, on] {
                    let rect = self.ui.button(cx, face).area().rect(cx);
                    if rect.size.x > 0.0 && rect.size.y > 0.0 {
                        found = Rect {
                            pos: DVec2 {
                                x: rect.pos.x + scroll.x,
                                y: rect.pos.y + scroll.y,
                            },
                            size: rect.size,
                        };
                        if i == slot {
                            origin = rect.pos;
                        }
                        break;
                    }
                }
                rects.push(found);
            }
            *floor_area_drag().lock().unwrap() = Some(FloorAreaDrag {
                slot,
                start: abs,
                last: abs,
                origin,
                rects,
                active: false,
                hover: Some(slot),
                finger,
            });
        }
    }

    fn floor_area_drag_move(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) -> bool {
        let mut repaint = false;
        let mut active = false;
        let mut ghost: Option<(bool, String, DVec2)> = None;
        {
            let mut guard = floor_area_drag().lock().unwrap();
            if let Some(d) = guard.as_mut() {
                if d.finger != finger {
                    return false;
                }
                let was_active = d.active;
                if !d.active && (abs - d.start).length() > 10.0 {
                    d.active = true;
                }
                if d.active {
                    active = true;
                    d.last = abs;
                    if !was_active {
                        *floor_area_autoscroll_frame().lock().unwrap() =
                            Some(cx.new_next_frame());
                    }
                    let hover = self.floor_area_drop_slot(cx, d, abs);
                    if hover != d.hover {
                        d.hover = hover;
                        repaint = true;
                    }
                    let label = {
                        let names = self.floor_area_tab_names();
                        names.get(d.slot).cloned().unwrap_or_default()
                    };
                    let show = if label.is_empty() {
                        app_state().lock().unwrap().t("tables")
                    } else {
                        label
                    };
                    ghost = Some((!was_active, show, d.origin + (abs - d.start)));
                }
            }
        }
        if active {
            cx.set_cursor(MouseCursor::Grabbing);
        }
        if let Some((first, label, pos)) = ghost {
            if first {
                self.show_floor_area_ghost(cx, &label, pos);
            } else {
                self.place_nav_ghost(cx, pos);
            }
        }
        if repaint {
            self.paint_floor_area_tabs(cx);
        }
        active
    }

    fn floor_area_drag_release(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) -> bool {
        let taken = {
            let mut guard = floor_area_drag().lock().unwrap();
            match guard.as_ref() {
                Some(d) if d.finger == finger => guard.take(),
                _ => None,
            }
        };
        if let Some(mut d) = taken {
            self.hide_nav_ghost(cx);
            if d.active {
                let drop_at = if abs.x != 0.0 || abs.y != 0.0 {
                    abs
                } else {
                    d.last
                };
                d.hover = self.floor_area_drop_slot(cx, &d, drop_at);
                cx.set_cursor(if d.hover.is_some() {
                    MouseCursor::Hand
                } else {
                    MouseCursor::Default
                });
                let swapped = match d.hover {
                    Some(target) if target != d.slot => {
                        let mut names = self.floor_area_tab_names();
                        if d.slot < names.len() && target < names.len() {
                            names.swap(d.slot, target);
                            let mut st = app_state().lock().unwrap();
                            st.floor_area_order = names.clone();
                            if let Some(store) = st.store.as_ref() {
                                let _ = store.set_floor_area_order_csv(&floor_area_order_csv(&names));
                            }
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                if swapped {
                    self.set_status(cx, "Area order saved");
                }
                self.sync_chrome(cx);
                return true;
            }
        }
        false
    }

    /// Floor area tabs: tap selects; press + move swaps (persists order).
    /// Same mouse/touch path as [`pointer_nav_drag`].
    fn pointer_floor_area_drag(&mut self, cx: &mut Cx, event: &Event) -> bool {
        enum Phase {
            Press(DVec2, Option<u64>),
            Move(DVec2, Option<u64>),
            Release(DVec2, Option<u64>),
        }
        let phases: Vec<Phase> = match event {
            Event::MouseDown(e) if e.button.is_primary() => {
                vec![Phase::Press(e.abs, None)]
            }
            Event::MouseMove(e) => vec![Phase::Move(e.abs, None)],
            Event::MouseUp(e) if e.button.is_primary() => {
                vec![Phase::Release(e.abs, None)]
            }
            Event::TouchUpdate(e) => {
                let mut out = Vec::new();
                for t in &e.touches {
                    match t.state {
                        TouchState::Start => out.push(Phase::Press(t.abs, Some(t.uid))),
                        TouchState::Move | TouchState::Stable => {
                            out.push(Phase::Move(t.abs, Some(t.uid)))
                        }
                        TouchState::Stop => out.push(Phase::Release(t.abs, Some(t.uid))),
                    }
                }
                out
            }
            _ => return false,
        };
        let mut consumed = false;
        for phase in phases {
            match phase {
                Phase::Press(abs, finger) => {
                    self.floor_area_drag_press(cx, abs, finger);
                }
                Phase::Move(abs, finger) => {
                    if self.floor_area_drag_move(cx, abs, finger) {
                        consumed = true;
                    }
                }
                Phase::Release(abs, finger) => {
                    if self.floor_area_drag_release(cx, abs, finger) {
                        consumed = true;
                    }
                }
            }
        }
        consumed
    }

    fn floor_area_drag_autoscroll(&mut self, cx: &mut Cx) {
        let last = {
            let guard = floor_area_drag().lock().unwrap();
            match guard.as_ref() {
                Some(d) if d.active => d.last,
                _ => return,
            }
        };
        *floor_area_autoscroll_frame().lock().unwrap() = Some(cx.new_next_frame());
        let view = self.ui.view(cx, ids!(floor_area_strip.tabs));
        let rect = view.area().rect(cx);
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        const ZONE: f64 = 56.0;
        let near_start = (ZONE - (last.x - rect.pos.x)).clamp(0.0, ZONE);
        let near_end = (ZONE - (rect.pos.x + rect.size.x - last.x)).clamp(0.0, ZONE);
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
        let next_x = (cur.x + delta).clamp(0.0, (total.x - visible.x).max(0.0));
        if (next_x - cur.x).abs() < 0.5 {
            return;
        }
        view.set_scroll_pos(cx, dvec2(next_x, cur.y));
        let mut repaint = false;
        {
            let mut guard = floor_area_drag().lock().unwrap();
            if let Some(d) = guard.as_mut() {
                let hover = self.floor_area_drop_slot(cx, d, d.last);
                if hover != d.hover {
                    d.hover = hover;
                    repaint = true;
                }
            }
        }
        if repaint {
            self.paint_floor_area_tabs(cx);
        }
    }

    fn route_printer_slot_at(&mut self, cx: &mut Cx, abs: DVec2) -> Option<usize> {
        let n = {
            let st = app_state().lock().unwrap();
            let key = parse_printer_route_key(&st.printer_route_key);
            st.printer_draft.route_for(key).len()
        };
        if n == 0 {
            return None;
        }
        let list = self
            .ui
            .widget(cx, ids!(route_assigned_list))
            .portal_list(cx, ids!(list));
        for i in 0..n {
            let item = list.item(cx, i, id!(Line));
            let rect = item.view(cx, ids!(row)).area().rect(cx);
            if rect.size.x > 0.0 && rect.size.y > 0.0 && rect.contains(abs) {
                return Some(i);
            }
        }
        None
    }

    fn route_printer_drop_slot(
        &self,
        d: &RoutePrinterDrag,
        abs: DVec2,
    ) -> Option<usize> {
        for (i, rect) in d.rects.iter().enumerate() {
            if rect.size.x > 0.0
                && rect.size.y > 0.0
                && abs.y >= rect.pos.y
                && abs.y <= rect.pos.y + rect.size.y
            {
                return Some(i);
            }
        }
        None
    }

    fn route_printer_drag_press(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) {
        *route_printer_drag().lock().unwrap() = None;
        if app_state().lock().unwrap().screen != Screen::Settings {
            return;
        }
        if !self.ui.view(cx, ids!(settings_panel_printers)).visible() {
            return;
        }
        let Some(slot) = self.route_printer_slot_at(cx, abs) else {
            return;
        };
        *nav_drag().lock().unwrap() = None;
        *settings_nav_drag().lock().unwrap() = None;
        *floor_area_drag().lock().unwrap() = None;
        self.hide_nav_ghost(cx);
        let list = self
            .ui
            .widget(cx, ids!(route_assigned_list))
            .portal_list(cx, ids!(list));
        let n = {
            let st = app_state().lock().unwrap();
            let key = parse_printer_route_key(&st.printer_route_key);
            st.printer_draft.route_for(key).len()
        };
        let mut origin = abs;
        let mut rects = Vec::with_capacity(n);
        let mut label = String::new();
        {
            let st = app_state().lock().unwrap();
            let key = parse_printer_route_key(&st.printer_route_key);
            let ids = st.printer_draft.route_for(key);
            if let Some(pid) = ids.get(slot) {
                if let Some(p) = st.printer_draft.printer_by_id(pid) {
                    label = if !p.target.trim().is_empty() {
                        p.target.clone()
                    } else {
                        p.label.clone()
                    };
                }
            }
        }
        for i in 0..n {
            let item = list.item(cx, i, id!(Line));
            let rect = item.view(cx, ids!(row)).area().rect(cx);
            let found = if rect.size.x > 0.0 && rect.size.y > 0.0 {
                if i == slot {
                    origin = rect.pos;
                }
                rect
            } else {
                Rect::default()
            };
            rects.push(found);
        }
        *route_printer_drag().lock().unwrap() = Some(RoutePrinterDrag {
            slot,
            start: abs,
            last: abs,
            origin,
            rects,
            active: false,
            hover: Some(slot),
            finger,
            label,
        });
    }

    fn route_printer_drag_move(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) -> bool {
        let mut active = false;
        let mut ghost: Option<(bool, String, DVec2)> = None;
        {
            let mut guard = route_printer_drag().lock().unwrap();
            if let Some(d) = guard.as_mut() {
                if d.finger != finger {
                    return false;
                }
                let was_active = d.active;
                if !d.active && (abs - d.start).length() > 10.0 {
                    d.active = true;
                }
                if d.active {
                    active = true;
                    d.last = abs;
                    d.hover = self.route_printer_drop_slot(d, abs);
                    ghost = Some((
                        !was_active,
                        d.label.clone(),
                        d.origin + (abs - d.start),
                    ));
                }
            }
        }
        if active {
            cx.set_cursor(MouseCursor::Grabbing);
        }
        if let Some((first, label, pos)) = ghost {
            if first {
                self.show_floor_area_ghost(cx, &label, pos);
            } else {
                self.place_nav_ghost(cx, pos);
            }
        }
        active
    }

    fn route_printer_drag_release(&mut self, cx: &mut Cx, abs: DVec2, finger: Option<u64>) -> bool {
        let taken = {
            let mut guard = route_printer_drag().lock().unwrap();
            match guard.as_ref() {
                Some(d) if d.finger == finger => guard.take(),
                _ => None,
            }
        };
        if let Some(mut d) = taken {
            self.hide_nav_ghost(cx);
            if d.active {
                *route_printer_tap_suppress().lock().unwrap() = true;
                let drop_at = if abs.x != 0.0 || abs.y != 0.0 {
                    abs
                } else {
                    d.last
                };
                d.hover = self.route_printer_drop_slot(&d, drop_at);
                cx.set_cursor(MouseCursor::Default);
                if let Some(target) = d.hover {
                    if target != d.slot {
                        let mut st = app_state().lock().unwrap();
                        let _ = swap_focused_route_printer(&mut st, d.slot, target);
                        drop(st);
                        self.paint_printers_panel(cx);
                    }
                }
                return true;
            }
        }
        false
    }

    fn pointer_route_printer_drag(&mut self, cx: &mut Cx, event: &Event) -> bool {
        enum Phase {
            Press(DVec2, Option<u64>),
            Move(DVec2, Option<u64>),
            Release(DVec2, Option<u64>),
        }
        let phases: Vec<Phase> = match event {
            Event::MouseDown(e) if e.button.is_primary() => {
                vec![Phase::Press(e.abs, None)]
            }
            Event::MouseMove(e) => vec![Phase::Move(e.abs, None)],
            Event::MouseUp(e) if e.button.is_primary() => {
                vec![Phase::Release(e.abs, None)]
            }
            Event::TouchUpdate(e) => {
                let mut out = Vec::new();
                for t in &e.touches {
                    match t.state {
                        TouchState::Start => out.push(Phase::Press(t.abs, Some(t.uid))),
                        TouchState::Move | TouchState::Stable => {
                            out.push(Phase::Move(t.abs, Some(t.uid)))
                        }
                        TouchState::Stop => out.push(Phase::Release(t.abs, Some(t.uid))),
                    }
                }
                out
            }
            _ => return false,
        };
        let mut consumed = false;
        for phase in phases {
            match phase {
                Phase::Press(abs, finger) => {
                    self.route_printer_drag_press(cx, abs, finger);
                }
                Phase::Move(abs, finger) => {
                    if self.route_printer_drag_move(cx, abs, finger) {
                        consumed = true;
                    }
                }
                Phase::Release(abs, finger) => {
                    if self.route_printer_drag_release(cx, abs, finger) {
                        consumed = true;
                    }
                }
            }
        }
        consumed
    }
}

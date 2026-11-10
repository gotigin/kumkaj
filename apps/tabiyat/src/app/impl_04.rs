impl App {
fn paint_chrome(&mut self, cx: &mut Cx) {
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |key: &str| locale::t(&lang, key);
        // Short chrome labels for bottom chips + rail (wrap, no ellipsis).
        // Page titles stay long (`items` = "Menu & items"); chrome uses `nav_items`.
        let floor = s("floor");
        let kitchen = s("kitchen");
        let books = s("books");
        let sell = s("sell");
        let buy = s("buy");
        let rates = s("nav_items");
        let sales = s("sales");
        let day_close = s("day_close");
        let reports = s("nav_reports");
        let inventory = s("inventory");
        let branches = s("branches");
        let credit = s("credit");
        let team = s("team");
        let settings = s("nav_settings");
        let lock = s("nav_lock");
        // Slots pack preference + rights (must match paint_nav_tabs).
        let order = current_nav_chrome_order();
        let label_for = |tab: NavTab| -> &str {
            match tab {
                NavTab::Books => &books,
                NavTab::Sell => &sell,
                NavTab::Buy => &buy,
                NavTab::Menu => &rates,
                NavTab::Floor => &floor,
                NavTab::Kitchen => &kitchen,
                NavTab::Sales => &sales,
                NavTab::DayClose => &day_close,
                NavTab::Reports => &reports,
                NavTab::Inventory => &inventory,
                NavTab::Branches => &branches,
                NavTab::Credit => &credit,
                NavTab::Team => &team,
                NavTab::Settings => &settings,
                NavTab::Lock => &lock,
            }
        };
        for pairs in [self.bottom_nav_pairs(), self.rail_nav_pairs()] {
            for (slot, (idle, on)) in pairs.iter().enumerate() {
                if let Some(tab) = order.get(slot).copied() {
                    let caption = nav_tab_caption(tab, label_for(tab));
                    self.ui.button(cx, idle).set_text(cx, &caption);
                    self.ui.button(cx, on).set_text(cx, &caption);
                }
            }
        }
        self.paint_nav_icons(cx, &order);
        self.paint_sidebar_settings_icons(cx);
        self.ui.button(cx, ids!(books_to_sell)).set_text(cx, &s("sell_money_in"));
        self.ui.button(cx, ids!(books_to_buy)).set_text(cx, &s("buy_money_out"));
        self.ui.button(cx, ids!(books_day_close)).set_text(cx, &s("day_close"));
        self.ui.button(cx, ids!(books_reports)).set_text(cx, &s("reports_expenses"));
        self.ui.button(cx, ids!(books_credit)).set_text(cx, &s("credit_customers"));
        // Same rights as chrome peers — never show Day close / Bank / Buy / …
        // to a Waiter who cannot open those screens (nav already hid the peers).
        {
            let caps = nav_caps_from(&app_state().lock().unwrap());
            let sales_ops = caps.sales || caps.is_owner;
            self.ui
                .view(cx, ids!(books_to_sell_slot))
                .set_visible(cx, caps.sell);
            self.ui
                .view(cx, ids!(books_to_buy_slot))
                .set_visible(cx, caps.buy_ok());
            self.ui
                .view(cx, ids!(books_day_close_slot))
                .set_visible(cx, sales_ops);
            self.ui
                .view(cx, ids!(books_reports_slot))
                .set_visible(cx, sales_ops);
            self.ui
                .view(cx, ids!(books_credit_slot))
                .set_visible(cx, caps.credit);
        }
        self.ui.button(cx, ids!(buy_purchase)).set_text(cx, &s("record_purchase"));
        self.ui.button(cx, ids!(buy_expense_add)).set_text(cx, &s("save_expense"));
        self.ui.widget(cx, ids!(h_buy_expense)).set_text(cx, &plain_display_text(&s("expense_heading")));
        self.ui.text_input(cx, ids!(buy_unit_input)).set_empty_text(cx, s("buy_unit_ph"));
        self.ui.text_input(cx, ids!(floor_cols_input)).set_empty_text(cx, s("floor_cols_ph"));
        self.ui.button(cx, ids!(buy_parts_add)).set_text(cx, &s("add_another_way"));
        {
            let help = self.ui.widget(cx, ids!(books_help)).as_info_help();
            help.set_title(cx, &s("books"));
            help.set_nav_icon(cx, NavTab::Books);
            help.set_tip(&plain_display_text(&s("books_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(sell_help)).as_info_help();
            help.set_title(cx, &s("sell"));
            help.set_nav_icon(cx, NavTab::Sell);
            help.set_tip(&plain_display_text(&s("order_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(buy_help)).as_info_help();
            help.set_title(cx, &s("buy"));
            help.set_nav_icon(cx, NavTab::Buy);
            help.set_tip(&plain_display_text(&s("buy_help")));
        }
        self.ui
            .button(cx, ids!(client_pair_back))
            .set_text(cx, &s("back"));
        // pay_mix says "Take payment · ₹265 · Cash" — set with the ticket (sync_chrome).
        {
            let st = app_state().lock().unwrap();
            let declared = st.lane_declared_label();
            let owner = st.lane_owner_label();
            let lane = match st.money_lane {
                MoneyLane::Declared => &declared,
                MoneyLane::Owner => &owner,
            };
            let lane_body = st.t("money_lane_bill").replace("{lane}", lane);
            self.ui
                .widget(cx, ids!(money_lane_label))
                .set_text(cx, &plain_display_text(&lane_body));
            for path in [ids!(lane_declared), ids!(lane_declared_on)] {
                self.ui.button(cx, path).set_text(cx, &declared);
            }
            for path in [ids!(lane_owner), ids!(lane_owner_on)] {
                self.ui.button(cx, path).set_text(cx, &owner);
            }
        }
        let (editing, editing_name) = {
            let st = app_state().lock().unwrap();
            let editing = st.editing_product_id.is_some();
            let name = st.editing_product_id.as_ref().and_then(|id| {
                st.products
                    .iter()
                    .find(|product| product.id == *id)
                    .map(|product| product.name.clone())
            });
            (editing, name)
        };
        self.ui.button(cx, ids!(menu_new)).set_visible(cx, editing);
        self.ui.button(cx, ids!(menu_new)).set_text(cx, &s("new_item"));
        self.ui.button(cx, ids!(menu_add)).set_text(
            cx,
            &s(if editing { "save" } else { "add_item" }),
        );
        self.ui.widget(cx, ids!(menu_mode_label)).set_text(cx, &plain_display_text(&match editing_name {
                Some(name) => format!("{}: {name}", s("editing_item")),
                None => s("adding_item"),
            },));
        self.ui.button(cx, ids!(image_clear)).set_text(cx, &s("clear_photo"));
        self.ui.button(cx, ids!(image_pick)).set_text(cx, &s("photo"));
        for path in [ids!(role_owner), ids!(role_owner_on)] {
            self.ui.button(cx, path).set_text(cx, &s("owner"));
        }
        for path in [ids!(role_staff), ids!(role_staff_on)] {
            self.ui.button(cx, path).set_text(cx, &s("staff"));
        }
        self.ui.button(cx, ids!(lock_staff_pick)).set_text(cx, &s("select_staff"));
        self.ui
            .widget(cx, ids!(lock_staff_sheet_title))
            .set_text(cx, &plain_display_text(&s("who_unlocking")));
        self.ui
            .button(cx, ids!(lock_staff_sheet_done))
            .set_text(cx, &s("pick_done"));
        self.ui.button(cx, ids!(pin_ok)).set_text(cx, &s("ok"));
        for path in [ids!(device_role_server), ids!(device_role_server_on)] {
            self.ui.button(cx, path).set_text(cx, &s("this_shop"));
        }
        for path in [ids!(device_role_client), ids!(device_role_client_on)] {
            self.ui.button(cx, path).set_text(cx, &s("extra_till"));
        }
        self.ui.button(cx, ids!(customer_clear)).set_text(cx, &s("clear"));
        self.ui.button(cx, ids!(undo_snack_btn)).set_text(cx, &s("undo"));
        self.ui.button(cx, ids!(inv_adjust_minus)).set_text(cx, &s("stock_minus"));
        self.ui.button(cx, ids!(inv_adjust_plus)).set_text(cx, &s("stock_plus"));
        self.ui.button(cx, ids!(inv_reorder_save)).set_text(cx, &s("set_reorder"));
        self.ui.button(cx, ids!(settle_cash)).set_text(cx, &s("settle_cash"));
        self.ui.button(cx, ids!(settle_card)).set_text(cx, &s("settle_card"));
        for (idle, on, key) in [
            (ids!(amend_pay_cash), ids!(amend_pay_cash_on), "cash"),
            (ids!(amend_pay_card), ids!(amend_pay_card_on), "card"),
        ] {
            self.ui.button(cx, idle).set_text(cx, &s(key));
            self.ui.button(cx, on).set_text(cx, &s(key));
        }
        {
            let help = self.ui.widget(cx, ids!(items_help_label)).as_info_help();
            help.set_title(cx, &s("items"));
            help.set_nav_icon(cx, NavTab::Menu);
            help.set_tip(&plain_display_text(&s("items_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(sales_help)).as_info_help();
            help.set_title(cx, &s("sales"));
            help.set_nav_icon(cx, NavTab::Sales);
            help.set_tip(&plain_display_text(&s("amend_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(credit_help)).as_info_help();
            help.set_title(cx, &s("credit"));
            help.set_nav_icon(cx, NavTab::Credit);
            help.set_tip(&plain_display_text(&s("credit_help")));
        }
        // Essay WrapLabels stay hidden; useful tips are InfoHelp ℹ above.
        for path in [
            ids!(h_country_pack_help),
            ids!(h_units_help),
            ids!(categories_help),
            ids!(h_brand_shown_help),
        ] {
            self.ui.widget(cx, path).set_visible(cx, false);
        }
        self.ui
            .text_input(cx, ids!(credit_add_kind))
            .set_empty_text(cx, s("party_kind_ph"));
        {
            let help = self.ui.widget(cx, ids!(team_help)).as_info_help();
            help.set_title(cx, &s("team"));
            help.set_nav_icon(cx, NavTab::Team);
            help.set_tip(&plain_display_text(&s("team_owner_help")));
        }
        self.ui.widget(cx, ids!(settings_title)).set_text(cx, &plain_display_text(&s("language_currency")));
        {
            let help = self.ui.widget(cx, ids!(day_close_help)).as_info_help();
            help.set_title(cx, &s("day_close"));
            help.set_nav_icon(cx, NavTab::DayClose);
            help.set_tip(&plain_display_text(&s("day_close_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(reports_help)).as_info_help();
            help.set_title(cx, &s("nav_reports"));
            help.set_nav_icon(cx, NavTab::Reports);
            help.set_tip(&plain_display_text(&s("reports_help")));
        }
        for path in [ids!(reports_today), ids!(reports_today_on)] {
            self.ui.button(cx, path).set_text(cx, &s("reports_today"));
        }
        for path in [ids!(reports_month), ids!(reports_month_on)] {
            self.ui.button(cx, path).set_text(cx, &s("reports_month"));
        }
        for path in [ids!(reports_aging), ids!(reports_aging_on)] {
            self.ui.button(cx, path).set_text(cx, &s("reports_aging"));
        }
        for path in [ids!(reports_balance_sheet), ids!(reports_balance_sheet_on)] {
            self.ui
                .button(cx, path)
                .set_text(cx, &s("reports_balance_sheet"));
        }
        self.ui
            .set_text(cx, &plain_display_text(&s("add_expense")));
        self.ui
            .widget(cx, ids!(reports_bank_heading))
            .set_text(cx, &plain_display_text(&s("bank_book")));
        {
            let help = self.ui.widget(cx, ids!(inventory_help)).as_info_help();
            help.set_title(cx, &s("inventory"));
            help.set_nav_icon(cx, NavTab::Inventory);
            help.set_tip(&plain_display_text(&s("stock_type_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(branches_help)).as_info_help();
            help.set_title(cx, &s("branches"));
            help.set_nav_icon(cx, NavTab::Branches);
            help.set_tip(&plain_display_text(&s("branch_export_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(sidebar_settings_help)).as_info_help();
            help.set_title(cx, &s("sidebar_settings"));
            help.set_nav_icon(cx, NavTab::Settings);
            help.set_tip(&plain_display_text(&s("sidebar_settings_help")));
        }
        self.ui
            .button(cx, ids!(sidebar_settings_back))
            .set_text(cx, &s("back"));
        self.ui
            .button(cx, ids!(sidebar_enable_all))
            .set_text(cx, &s("sidebar_enable_all"));
        self.ui
            .button(cx, ids!(sidebar_disable_all))
            .set_text(cx, &s("sidebar_disable_all"));
        {
            let help = self.ui.widget(cx, ids!(setup_lang_help)).as_info_help();
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("language_setup_hint")));
        }
        self.ui
            .widget(cx, ids!(device_setup_banner))
            .set_text(cx, &plain_display_text(&s("this_device")));
        self.ui
            .widget(cx, ids!(device_use_question))
            .set_visible(cx, false);
        {
            let help = self.ui.widget(cx, ids!(setup_device_help)).as_info_help();
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("device_setup_help")));
        }
        self.ui
            .widget(cx, ids!(server_pick_title))
            .set_text(cx, &plain_display_text(&s("choose_server")));
        {
            let help = self.ui.widget(cx, ids!(server_pick_help)).as_info_help();
            // Tip-only — title is the sibling WrapLabel (ℹ hugs the label).
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("choose_server_hint")));
        }
        {
            let help = self.ui.widget(cx, ids!(server_pair_help)).as_info_help();
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("lan_join_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(client_pair_help)).as_info_help();
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("pair_intro")));
        }
        self.ui
            .widget(cx, ids!(lock_login_title))
            .set_text(cx, &plain_display_text(&s("or_unlock_login")));
        {
            let help = self.ui.widget(cx, ids!(lock_login_help)).as_info_help();
            help.set_title(cx, "");
            // Tip for unlock-with-login (not Team's "optional login" essay).
            help.set_tip(&plain_display_text(&s("or_unlock_login")));
        }
        self.ui
            .button(cx, ids!(server_create_new))
            .set_text(cx, &s("create_new_server"));
        self.ui
            .button(cx, ids!(server_pick_cancel))
            .set_text(cx, &s("back"));
        self.ui
            .text_input(cx, ids!(server_name_input))
            .set_empty_text(cx, s("new_server_name_ph"));
        self.ui
            .text_input(cx, ids!(server_remove_pin))
            .set_empty_text(cx, s("server_remove_pin_ph"));
        let awaiting_app_template = {
            let st = app_state().lock().unwrap();
            st.awaiting_app_template
                && !role_open_busy().load(std::sync::atomic::Ordering::SeqCst)
        };
        // Offline LAN: multi-server pick hidden (cloud-only later).
        self.ui
            .view(cx, ids!(server_pick_section))
            .set_visible(cx, false);
        self.paint_app_template_chrome(cx, awaiting_app_template);
        // Device-role chip visibility (idle/active pair) is owned by paint_choice_state.
        self.ui
            .button(cx, ids!(lang_setup_platform))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(lang_setup_platform_on))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(lang_setup_user))
            .set_text(cx, &s("lang_pick_user"));
        self.ui
            .button(cx, ids!(lang_setup_user_on))
            .set_text(cx, &s("lang_pick_user"));
        self.ui
            .button(cx, ids!(settings_lang_platform))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(settings_lang_platform_on))
            .set_text(cx, &s("lang_follow_platform"));
        self.ui
            .button(cx, ids!(settings_lang_user))
            .set_text(cx, &s("lang_pick_user"));
        self.ui
            .button(cx, ids!(settings_lang_user_on))
            .set_text(cx, &s("lang_pick_user"));
        let selected_lang_line = {
            let st = app_state().lock().unwrap();
            let code = if st.language_follow_platform {
                locale::detect_platform_language()
            } else {
                st.locale.language.clone()
            };
            let name = locale::language_by_code(&code)
                .map(|l| format!("{} · {}", l.native_name, l.name_en))
                .unwrap_or_else(|| code.clone());
            if st.language_follow_platform {
                format!("{}: {name}", st.t("lang_platform_now"))
            } else {
                format!("{}: {name}", st.t("selected_language"))
            }
        };
        self.ui
            .widget(cx, ids!(setup_lang_selected))
            .set_text(cx, &plain_display_text(&selected_lang_line));
        self.ui
            .widget(cx, ids!(settings_lang_selected))
            .set_text(cx, &plain_display_text(&selected_lang_line));
    }

    /// LiveId path of the page `ScrollYView` for `screen`.
    fn screen_scroll_view(screen: Screen) -> &'static [LiveId] {
        match screen {
            Screen::DeviceSetup => ids!(device_setup_screen),
            Screen::Lock => ids!(lock_screen),
            Screen::Onboard => ids!(onboard_screen),
            Screen::ClientPair => ids!(client_pair_screen),
            Screen::Books => ids!(books_screen),
            Screen::Pos => ids!(pos_scroll),
            Screen::Buy => ids!(buy_screen),
            Screen::Floor => ids!(floor_screen),
            Screen::Kitchen => ids!(kitchen_screen),
            Screen::Menu => ids!(menu_screen),
            Screen::Sales => ids!(sales_screen),
            Screen::Credit => ids!(credit_screen),
            Screen::Team => ids!(team_screen),
            Screen::Settings => ids!(settings_screen),
            Screen::SidebarSettings => ids!(sidebar_settings_screen),
            Screen::DayClose => ids!(day_close_screen),
            Screen::Reports => ids!(reports_screen),
            Screen::Inventory => ids!(inventory_screen),
            Screen::Branches => ids!(branches_screen),
        }
    }

    /// Keyboard scroll for the active page ScrollYView.
    /// ↑/↓ move by a line step; PgUp jumps to top; PgDn jumps to bottom.
    /// Returns true when the key was consumed (including no-op at limits).
    fn handle_scroll_key(&mut self, cx: &mut Cx, e: &KeyEvent) -> bool {
        if !matches!(
            e.key_code,
            KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::ArrowLeft
                | KeyCode::ArrowRight
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End
        ) {
            return false;
        }
        if e.modifiers.control || e.modifiers.alt || e.modifiers.logo {
            return false;
        }
        // Only a text field owns these keys (caret movement). A button that
        // kept focus after a click must not — that silently disabled
        // Page Up / Page Down after any tap.
        if self.text_field_has_focus(cx) {
            return false;
        }
        let screen = app_state().lock().unwrap().screen;
        // ← / → pan the focused table's area (or the first area) one column.
        if matches!(e.key_code, KeyCode::ArrowLeft | KeyCode::ArrowRight) {
            if screen != Screen::Floor {
                return false;
            }
            let area_idx = {
                let st = app_state().lock().unwrap();
                let tables = floor_filtered_tables(&st);
                let areas = floor_area_groups(&tables);
                let want = st
                    .floor_focus_table_id
                    .as_ref()
                    .and_then(|id| tables.iter().position(|t| &t.id == id))
                    .and_then(|ti| {
                        areas.iter().position(|(_, idxs)| idxs.contains(&ti))
                    });
                want.unwrap_or(0)
            };
            let list = self.ui.widget(cx, ids!(floor_list)).portal_list(cx, ids!(list));
            let item = list.item(cx, area_idx, id!(Item));
            let view = item.view(cx, ids!(area_scroll));
            let pos = view.get_scroll_pos();
            let max_x = (view.get_scroll_view_total().x - view.get_scroll_view_visible().x).max(0.0);
            let (zoom, zoom_auto, cols) = {
                let st = app_state().lock().unwrap();
                (st.floor_zoom, st.floor_zoom_auto, floor_columns())
            };
            let (_vp_left, vp_w) = *floor_hviewport().lock().unwrap();
            let scale = floor_effective_scale(zoom_auto, zoom, cols, vp_w.max(1.0));
            let (tw, _) = floor_tile_wh(scale);
            let step = tw + FLOOR_TILE_GAP;
            let new_x = if e.key_code == KeyCode::ArrowLeft {
                (pos.x - step).max(0.0)
            } else {
                (pos.x + step).min(max_x)
            };
            if (new_x - pos.x).abs() >= 0.5 {
                view.set_scroll_pos(cx, dvec2(new_x, pos.y));
                view.redraw(cx);
            }
            return true;
        }
        let view = self.ui.view(cx, Self::screen_scroll_view(screen));
        let pos = view.get_scroll_pos();
        let total = view.get_scroll_view_total();
        let visible = view.get_scroll_view_visible();
        let max_y = (total.y - visible.y).max(0.0);
        // One comfortable line; hold-to-repeat uses is_repeat from the OS.
        const ARROW_STEP: f64 = 48.0;
        let new_y = match e.key_code {
            KeyCode::ArrowUp => (pos.y - ARROW_STEP).max(0.0),
            KeyCode::ArrowDown => (pos.y + ARROW_STEP).min(max_y),
            // A screenful, keeping a line of the old view for context — as
            // browsers and native apps do. Home / End go to the ends.
            KeyCode::PageUp => (pos.y - (visible.y - ARROW_STEP).max(ARROW_STEP)).max(0.0),
            KeyCode::PageDown => (pos.y + (visible.y - ARROW_STEP).max(ARROW_STEP)).min(max_y),
            KeyCode::Home => 0.0,
            KeyCode::End => max_y,
            _ => return false,
        };
        if (new_y - pos.y).abs() < 0.5 {
            return true;
        }
        view.set_scroll_pos(cx, dvec2(pos.x, new_y));
        view.redraw(cx);
        true
    }

    fn show_screen(&mut self, cx: &mut Cx, screen: Screen) {
        let pairs = [
            (ids!(device_setup_screen), screen == Screen::DeviceSetup),
            (ids!(lock_screen), screen == Screen::Lock),
            (ids!(onboard_screen), screen == Screen::Onboard),
            (ids!(client_pair_screen), screen == Screen::ClientPair),
            (ids!(books_screen), screen == Screen::Books),
            (ids!(pos_screen), screen == Screen::Pos),
            (ids!(buy_screen), screen == Screen::Buy),
            (ids!(floor_screen), screen == Screen::Floor),
            (ids!(kitchen_screen), screen == Screen::Kitchen),
            (ids!(menu_screen), screen == Screen::Menu),
            (ids!(sales_screen), screen == Screen::Sales),
            (ids!(credit_screen), screen == Screen::Credit),
            (ids!(team_screen), screen == Screen::Team),
            (ids!(settings_screen), screen == Screen::Settings),
            (
                ids!(sidebar_settings_screen),
                screen == Screen::SidebarSettings,
            ),
            (ids!(day_close_screen), screen == Screen::DayClose),
            (ids!(reports_screen), screen == Screen::Reports),
            (ids!(inventory_screen), screen == Screen::Inventory),
            (ids!(branches_screen), screen == Screen::Branches),
        ];
        for (path, visible) in pairs {
            self.ui.view(cx, path).set_visible(cx, visible);
        }
        let unlocked = !matches!(
            screen,
            Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
        );
        // Nested setup (Onboard, Client pair, open Language / server pick)
        // uses the same header ‹ as every other nested screen. Lock stays bare.
        let setup_back = setup_shows_header_back(&app_state().lock().unwrap());
        self.ui.view(cx, ids!(header)).set_visible(cx, unlocked || setup_back);
        self.ui
            .button(cx, ids!(amounts_eye))
            .set_visible(cx, unlocked);
        // Today strip: opt-in via shop meta (default off — quiet counter).
        let show_today = unlocked
            && session_may_see_today(app_state().lock().unwrap().session.as_ref())
            && app_state()
                .lock()
                .unwrap()
                .store
                .as_ref()
                .and_then(|s| s.show_today_bar().ok())
                .unwrap_or(false);
        self.ui.view(cx, ids!(today_bar)).set_visible(cx, show_today);
        self.apply_shell_layout(cx, unlocked);
        self.paint_nav_tabs(cx, screen);
    }

    /// Phone (<720): bottom tabs. Tablet/desktop (≥720): side rail.
    /// Owner/cashier: Added items | Counter always side by side under the
    /// sticky type strip (`pos_ticket` ScrollX pans when mins do not fit) —
    /// never stacked above/below on a narrow window. Waiters (Settle Denied):
    /// Counter dock hidden; cart takes the full ticket width.
    fn apply_shell_layout(&mut self, cx: &mut Cx, unlocked: bool) {
        const DESIGN_DESKTOP_W: f64 = store::DEFAULT_DESKTOP_WINDOW_W;
        const DIVIDER_W: f64 = 6.0;
        const DESK_LEFT_MIN: f64 = 600.0;
        const DESK_RIGHT_MIN: f64 = 520.0;
        const DESK_GAP: f64 = 16.0;
        let window_w = {
            let w = self.window_inner_width(cx);
            if w >= 1.0 {
                w
            } else {
                DESIGN_DESKTOP_W
            }
        };
        let shell_wide = window_w >= store::TABLET_SHELL_BREAKPOINT_CSS;
        let rail_collapsed = app_state().lock().unwrap().rail_collapsed;
        let max_rail = (window_w - MAIN_CONTENT_MIN - DIVIDER_W).max(RAIL_WIDTH_MIN);
        let preferred = {
            let mut st = app_state().lock().unwrap();
            let clamped = st.rail_width.clamp(RAIL_WIDTH_MIN, max_rail);
            st.rail_width = clamped;
            clamped
        };
        // Mid-tween width from collapse/expand; None when idle.
        let (anim_w, expanding) = {
            let slot = rail_width_anim().lock().unwrap();
            match slot.as_ref() {
                Some(anim) => {
                    let t = std::time::Instant::now()
                        .duration_since(anim.started)
                        .as_secs_f64()
                        * 1000.0
                        / ACCORDION_ANIM_MS;
                    let p = accordion_ease(t);
                    let w = (anim.from + (anim.to - anim.from) * p).max(0.0);
                    (Some(w), anim.to > 0.0)
                }
                None => (None, false),
            }
        };
        let rail_animating = anim_w.is_some();
        // Sidebar mode only: user can hide the whole rail (zero width).
        // Phone bottom tabs are never affected. During expand keep the rail
        // visible from the first frame; during collapse until width ≈ 0.
        let show_rail = unlocked
            && shell_wide
            && match anim_w {
                Some(w) => expanding || w > 0.5,
                None => !rail_collapsed,
            };
        let show_bottom = unlocked && !shell_wide;
        self.ui.view(cx, ids!(nav_rail)).set_visible(cx, show_rail);
        self.ui.view(cx, ids!(rail_divider)).set_visible(cx, show_rail);
        self.ui.view(cx, ids!(bottom_nav)).set_visible(cx, show_bottom);
        self.ui.button(cx, ids!(rail_expand_btn)).set_visible(
            cx,
            unlocked && shell_wide && rail_collapsed && !rail_animating,
        );
        self.ui
            .button(cx, ids!(rail_collapse_btn))
            .set_visible(cx, show_rail && !rail_collapsed && !rail_animating);

        let rail_w = if let Some(w) = anim_w {
            set_view_walk(
                cx,
                &self.ui.view(cx, ids!(nav_rail)),
                Walk {
                    width: Size::Fixed(w),
                    height: Size::fill(),
                    ..Default::default()
                },
            );
            w
        } else if show_rail {
            set_view_walk(
                cx,
                &self.ui.view(cx, ids!(nav_rail)),
                Walk {
                    width: Size::Fixed(preferred),
                    height: Size::fill(),
                    ..Default::default()
                },
            );
            preferred
        } else {
            0.0
        };

        // Content width ≈ window − rail − divider − screen padding (16+16).
        let content_w = (window_w
            - if show_rail { rail_w + DIVIDER_W } else { 0.0 }
            - 32.0)
            .max(280.0);
        *content_width().lock().unwrap() = content_w;

        for path in [ids!(pos_catalog), ids!(items_body), ids!(catalog_header)] {
            set_view_walk(
                cx,
                &self.ui.view(cx, path),
                Walk {
                    width: Size::fill(),
                    height: Size::fit(),
                    ..Default::default()
                },
            );
        }

        let may_settle =
            session_may_settle(app_state().lock().unwrap().session.as_ref());
        // Owner/cashier: always neighbours (ScrollX if the window is tight).
        // Waiter: no Counter → full-width cart.
        let ticket_row = self.ui.view(cx, ids!(pos_ticket_row));
        set_view_flow(
            cx,
            &ticket_row,
            if may_settle {
                Flow::right()
            } else {
                Flow::Down
            },
        );
        set_view_walk(
            cx,
            &ticket_row,
            Walk {
                width: if may_settle {
                    Size::fit()
                } else {
                    Size::fill()
                },
                height: Size::fill(),
                ..Default::default()
            },
        );
        let (left_w, right_w) = if !may_settle {
            (Size::fill(), Size::Fixed(0.0))
        } else {
            let desk_avail = (content_w - 32.0).max(200.0);
            let min_total = DESK_LEFT_MIN + DESK_RIGHT_MIN + DESK_GAP;
            if desk_avail >= min_total {
                let left = desk_avail * (DESK_LEFT_MIN / (DESK_LEFT_MIN + DESK_RIGHT_MIN));
                (Size::Fixed(left), Size::Fixed(desk_avail - DESK_GAP - left))
            } else {
                // Side by side at design mins — ScrollX pans; never stack.
                (Size::Fixed(DESK_LEFT_MIN), Size::Fixed(DESK_RIGHT_MIN))
            }
        };
        set_view_walk(
            cx,
            &self.ui.view(cx, ids!(pos_scroll)),
            Walk {
                width: left_w,
                height: Size::fill(),
                ..Default::default()
            },
        );
        set_view_walk(
            cx,
            &self.ui.view(cx, ids!(counter_dock)),
            Walk {
                width: right_w,
                height: Size::fill(),
                ..Default::default()
            },
        );
        self.ui.view(cx, ids!(pos_scroll)).set_visible(cx, true);
        self.ui.view(cx, ids!(counter_dock)).set_visible(cx, may_settle);
    }

    /// Nav slot hardware in DSL order. Slots are positions; which tab a
    /// slot shows comes from `AppState.nav_order` (drag to swap).
    fn bottom_nav_pairs(&self) -> [(&[LiveId], &[LiveId]); NAV_TABS.len()] {
        [
            (ids!(nav_books), ids!(nav_books_on)),
            (ids!(nav_sell), ids!(nav_sell_on)),
            (ids!(nav_buy), ids!(nav_buy_on)),
            (ids!(nav_menu), ids!(nav_menu_on)),
            (ids!(nav_floor), ids!(nav_floor_on)),
            (ids!(nav_kitchen), ids!(nav_kitchen_on)),
            (ids!(nav_sales), ids!(nav_sales_on)),
            (ids!(nav_day_close), ids!(nav_day_close_on)),
            (ids!(nav_reports), ids!(nav_reports_on)),
            (ids!(nav_inventory), ids!(nav_inventory_on)),
            (ids!(nav_branches), ids!(nav_branches_on)),
            (ids!(nav_credit), ids!(nav_credit_on)),
            (ids!(nav_settings), ids!(nav_settings_on)),
            (ids!(nav_lock), ids!(nav_lock_on)),
        ]
    }

    fn rail_nav_pairs(&self) -> [(&[LiveId], &[LiveId]); NAV_TABS.len()] {
        [
            (ids!(rail_books), ids!(rail_books_on)),
            (ids!(rail_sell), ids!(rail_sell_on)),
            (ids!(rail_buy), ids!(rail_buy_on)),
            (ids!(rail_rates), ids!(rail_rates_on)),
            (ids!(rail_ops), ids!(rail_ops_on)),
            (ids!(rail_cod), ids!(rail_cod_on)),
            (ids!(rail_sales), ids!(rail_sales_on)),
            (ids!(rail_day_close), ids!(rail_day_close_on)),
            (ids!(rail_reports), ids!(rail_reports_on)),
            (ids!(rail_inventory), ids!(rail_inventory_on)),
            (ids!(rail_branches), ids!(rail_branches_on)),
            (ids!(rail_credit), ids!(rail_credit_on)),
            (ids!(rail_settings), ids!(rail_settings_on)),
            (ids!(rail_lock), ids!(rail_lock_on)),
        ]
    }

    fn bottom_nav_icon_ids(&self) -> [&[LiveId]; NAV_TABS.len()] {
        [
            ids!(nav_books_icon),
            ids!(nav_sell_icon),
            ids!(nav_buy_icon),
            ids!(nav_menu_icon),
            ids!(nav_floor_icon),
            ids!(nav_kitchen_icon),
            ids!(nav_sales_icon),
            ids!(nav_day_close_icon),
            ids!(nav_reports_icon),
            ids!(nav_inventory_icon),
            ids!(nav_branches_icon),
            ids!(nav_credit_icon),
            ids!(nav_settings_icon),
            ids!(nav_lock_icon),
        ]
    }

    fn rail_nav_icon_ids(&self) -> [&[LiveId]; NAV_TABS.len()] {
        [
            ids!(rail_books_icon),
            ids!(rail_sell_icon),
            ids!(rail_buy_icon),
            ids!(rail_rates_icon),
            ids!(rail_ops_icon),
            ids!(rail_cod_icon),
            ids!(rail_sales_icon),
            ids!(rail_day_close_icon),
            ids!(rail_reports_icon),
            ids!(rail_inventory_icon),
            ids!(rail_branches_icon),
            ids!(rail_credit_icon),
            ids!(rail_settings_icon),
            ids!(rail_lock_icon),
        ]
    }

    fn paint_nav_icons(&mut self, cx: &mut Cx, order: &[NavTab]) {
        for icons in [self.bottom_nav_icon_ids(), self.rail_nav_icon_ids()] {
            for (slot, icon_id) in icons.iter().enumerate() {
                if let Some(tab) = order.get(slot).copied() {
                    load_nav_icon_once(cx, &self.ui.image(cx, icon_id), tab);
                }
            }
        }
    }

    /// Settings → Sidebar list icons: same load path as the rail, run
    /// from chrome sync (outside PortalList draw) so WebP textures are
    /// ready before the next paint.
    fn paint_sidebar_settings_icons(&mut self, cx: &mut Cx) {
        if app_state().lock().unwrap().screen != Screen::SidebarSettings {
            return;
        }
        let order = app_state().lock().unwrap().nav_order.clone();
        let list = self
            .ui
            .widget(cx, ids!(sidebar_nav_list))
            .portal_list(cx, ids!(list));
        for (i, tab) in order.iter().enumerate() {
            let item = list.item(cx, i, id!(Item));
            load_nav_icon_once(cx, &item.image(cx, ids!(icon)), *tab);
        }
    }

    fn nav_slot_wrappers(&self, rail: bool) -> [&[LiveId]; NAV_TABS.len()] {
        if rail {
            [
                ids!(rail_books_slot),
                ids!(rail_sell_slot),
                ids!(rail_buy_slot),
                ids!(rail_rates_slot),
                ids!(rail_ops_slot),
                ids!(rail_cod_slot),
                ids!(rail_sales_slot),
                ids!(rail_day_close_slot),
                ids!(rail_reports_slot),
                ids!(rail_inventory_slot),
                ids!(rail_branches_slot),
                ids!(rail_credit_slot),
                ids!(rail_settings_slot),
                ids!(rail_lock_slot),
            ]
        } else {
            [
                ids!(nav_books_slot),
                ids!(nav_sell_slot),
                ids!(nav_buy_slot),
                ids!(nav_menu_slot),
                ids!(nav_floor_slot),
                ids!(nav_kitchen_slot),
                ids!(nav_sales_slot),
                ids!(nav_day_close_slot),
                ids!(nav_reports_slot),
                ids!(nav_inventory_slot),
                ids!(nav_branches_slot),
                ids!(nav_credit_slot),
                ids!(nav_settings_slot),
                ids!(nav_lock_slot),
            ]
        }
    }

    /// Exactly one nav peer highlighted for `screen` (bottom and/or rail).
    /// Slot order packs preference + rights: peers the signed-in person cannot
    /// open never take a slot (draw used to re-show them after sync_chrome hid
    /// them — that left forbidden tabs visible on Client / staff unlocks).
    fn paint_nav_tabs(&mut self, cx: &mut Cx, screen: Screen) {
        let order = current_nav_chrome_order();
        let active = nav_tab_for(screen);
        // While dragging, the slot under the pointer wears the active face
        // so the drop target is unmistakable.
        let drop = nav_drag()
            .lock()
            .unwrap()
            .as_ref()
            .filter(|d| d.active)
            .and_then(|d| d.hover.map(|slot| (d.rail, slot)));
        for (rail, pairs) in [(false, self.bottom_nav_pairs()), (true, self.rail_nav_pairs())] {
            let wraps = self.nav_slot_wrappers(rail);
            for (slot, (idle, on)) in pairs.iter().enumerate() {
                self.clear_button_animators(cx, idle);
                self.clear_button_animators(cx, on);
                let show = slot < order.len();
                self.ui.view(cx, wraps[slot]).set_visible(cx, show);
                if !show {
                    self.ui.button(cx, idle).set_visible(cx, false);
                    self.ui.button(cx, on).set_visible(cx, false);
                    continue;
                }
                let selected = active == Some(order[slot]) || drop == Some((rail, slot));
                self.ui.button(cx, idle).set_visible(cx, !selected);
                self.ui.button(cx, on).set_visible(cx, selected);
            }
        }
    }

    fn sync_venue_brand(&mut self, cx: &mut Cx) {
        let (title, header_title, server_id_short, icon_path, icon_hint, onboard_icon_hint, saved_title, saved_org) = {
            let st = app_state().lock().unwrap();
            // listing_name: restaurant, else business/org, else product name.
            // Never show "Tabiyat" when the owner saved venue or org.
            let title = st
                .store
                .as_ref()
                .and_then(|s| s.listing_name().ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    if st.venue_title.trim().is_empty() {
                        "Tabiyat".to_string()
                    } else {
                        st.display_venue_title()
                    }
                });
            // Short server id beside the restaurant name (smaller sibling label).
            // Client: the paired shop. Server: the open local books id.
            let server_id_short = match st.device_role {
                DeviceRole::Client => st
                    .store
                    .as_ref()
                    .and_then(|s| s.paired_server_id().ok().flatten()),
                _ => st
                    .data_base
                    .as_ref()
                    .and_then(|base| local_servers::active_server_id(base).ok().flatten()),
            }
            .map(|id| id.chars().take(8).collect::<String>())
            .filter(|id| !id.is_empty());
            // The header shows the restaurant's name as the owner saved it
            // (the app's name until they do); the business and who is signed
            // in sit under it (sync_chrome).
            let header_title = title.clone();
            // Draft pick shows immediately; saved key after Save / onboard import.
            let icon_path = st
                .draft_venue_icon_path
                .clone()
                .or_else(|| st.venue_icon_path());
            let icon_hint = if st.draft_venue_icon_path.is_some() {
                st.t("icon_new_selected")
            } else if icon_path.is_some() {
                st.t("icon_custom")
            } else {
                st.t("icon_default")
            };
            let onboard_icon_hint = if icon_path.is_some() {
                st.t("logo_yours")
            } else {
                st.t("logo_default")
            };
            let saved_org = st
                .store
                .as_ref()
                .and_then(|s| s.saved_org_name().ok())
                .unwrap_or_default();
            (
                title,
                header_title,
                server_id_short,
                icon_path,
                icon_hint,
                onboard_icon_hint,
                st.venue_title.clone(),
                saved_org,
            )
        };
        self.ui
            .widget(cx, ids!(brand_title)).set_text(cx, &plain_display_text(&header_title));
        for id in [ids!(setup_brand_title), ids!(lock_brand_title)] {
            self.ui.widget(cx, id).set_text(cx, &plain_display_text(&title));
        }
        let id_label = server_id_short
            .as_ref()
            .map(|id| format!("({id})"))
            .unwrap_or_default();
        let id_visible = server_id_short.is_some();
        for id in [
            ids!(brand_server_id),
            ids!(setup_brand_server_id),
            ids!(lock_brand_server_id),
        ] {
            let w = self.ui.widget(cx, id);
            w.set_text(cx, &plain_display_text(&id_label));
            w.set_visible(cx, id_visible);
        }
        for id in [
            ids!(brand_mark),
            ids!(setup_brand_mark),
            ids!(lock_brand_mark),
            ids!(onboard_brand_mark),
        ] {
            let img = self.ui.image(cx, id);
            // Custom shop mark when on disk; else embedded default. Never leave a
            // visible empty Image (black square on Android when paths miss).
            let loaded = if let Some(ref path) = icon_path {
                load_brand_mark_at(cx, &img, path) || load_default_brand_mark(cx, &img)
            } else {
                load_default_brand_mark(cx, &img)
            };
            img.set_visible(cx, loaded);
        }
        self.ui
            .widget(cx, ids!(venue_icon_status))
            .set_text(cx, &plain_display_text(&icon_hint));
        self.ui
            .widget(cx, ids!(onboard_icon_status))
            .set_text(cx, &plain_display_text(&onboard_icon_hint));
        let (stamp_hint, pay_qr_hint) = {
            let st = app_state().lock().unwrap();
            let lang = st.locale.language.clone();
            let (stamp, qr) = st
                .store
                .as_ref()
                .map(|store| {
                    (
                        store.print_stamp_path().ok().flatten().is_some(),
                        store.payment_qr_path().ok().flatten().is_some(),
                    )
                })
                .unwrap_or((false, false));
            (
                locale::t(&lang, if stamp { "stamp_set" } else { "stamp_none" }),
                locale::t(
                    &lang,
                    if qr {
                        "payment_qr_set"
                    } else {
                        "payment_qr_none"
                    },
                ),
            )
        };
        self.ui
            .widget(cx, ids!(print_stamp_status))
            .set_text(cx, &plain_display_text(&stamp_hint));
        self.ui
            .widget(cx, ids!(payment_qr_status))
            .set_text(cx, &plain_display_text(&pay_qr_hint));
        let field = self.ui.text_input(cx, ids!(venue_title_input));
        if !cx.has_key_focus(field.area()) {
            let shown = field.text();
            if shown != saved_title {
                field.set_text(cx, &saved_title);
            }
        }
        let org_field = self.ui.text_input(cx, ids!(org_name_input));
        if !cx.has_key_focus(org_field.area()) && org_field.text() != saved_org {
            org_field.set_text(cx, &saved_org);
        }
    }

    fn save_venue_brand(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        let title = self.ui.text_input(cx, ids!(venue_title_input)).text();
        let org = self.ui.text_input(cx, ids!(org_name_input)).text();
        let outcome = (|| {
            let mut st = app_state().lock().unwrap();
            let actor = st
                .session
                .clone()
                .ok_or_else(|| "Unlock as Owner to change brand".to_string())?;
            if !actor.is_owner() {
                return Err("Only Owner can change in-app brand".to_string());
            }
            let (icon_filename, icon_bytes, draft_path) =
                if let Some(path) = st.draft_venue_icon_path.clone() {
                    let bytes = std::fs::read(&path)
                        .map_err(|e| format!("Could not read the icon: {e}"))?;
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("venue_icon.webp")
                        .to_string();
                    (filename, bytes, Some(path))
                } else {
                    (String::new(), Vec::new(), None)
                };
            let msg = st.books_op(BooksOp::SetVenueBrand {
                title: title.clone(),
                org: org.clone(),
                icon_filename,
                icon_bytes,
            })?;
            if let Some(path) = draft_path {
                if let Some(store) = st.store.as_ref() {
                    if let Ok(dir) = store.brand_images_dir() {
                        invalidate_image_loads_for_path(&dir.join("venue_icon.webp"));
                    }
                }
                invalidate_image_loads_for_path(&path);
                st.venue_icon_key = Some("custom:venue_icon.webp".into());
                st.draft_venue_icon_path = None;
            }
            st.venue_title = title.trim().to_string();
            if st.device_role != DeviceRole::Client {
                let _ = st.refresh_branches_text();
            }
            Ok(msg)
        })();
        match outcome {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn clear_venue_icon(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        let outcome = (|| {
            let mut st = app_state().lock().unwrap();
            let actor = st
                .session
                .clone()
                .ok_or_else(|| "Unlock as Owner to change brand".to_string())?;
            if !actor.is_owner() {
                return Err("Only Owner can change in-app brand".to_string());
            }
            let msg = st.books_op(BooksOp::ClearVenueIcon)?;
            st.venue_icon_key = None;
            st.draft_venue_icon_path = None;
            Ok(msg)
        })();
        match outcome {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn pick_venue_icon(&mut self, cx: &mut Cx) {
        {
            let st = app_state().lock().unwrap();
            match &st.session {
                Some(a) if a.is_owner() => {}
                Some(_) => {
                    drop(st);
                    self.set_status(cx, "Only Owner can change in-app brand");
                    return;
                }
                None => {
                    drop(st);
                    self.set_status(cx, "Unlock as Owner to change brand");
                    return;
                }
            }
        }
        let dialog = FileDialog::new()
            .set_title("Choose venue icon".into())
            .add_filter(
                "Images".into(),
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                    "gif".into(),
                    "bmp".into(),
                ],
            )
            .set_id(live_id!(venue_icon_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening icon picker…");
    }

    /// Settings → Bill format: stamp or static payment QR image.
    fn pick_print_brand_image(&mut self, cx: &mut Cx, id: LiveId, title: &str) {
        let dialog = FileDialog::new()
            .set_title(title.into())
            .add_filter(
                "Images".into(),
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                    "gif".into(),
                    "bmp".into(),
                ],
            )
            .set_id(id);
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening image picker…");
    }

    /// Onboarding logo pick: same image dialog, but the chosen file is
    /// imported immediately (no draft/Save-brand step during setup).
    fn pick_onboard_icon(&mut self, cx: &mut Cx) {
        let dialog = FileDialog::new()
            .set_title("Choose shop logo".into())
            .add_filter(
                "Images".into(),
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                    "gif".into(),
                    "bmp".into(),
                ],
            )
            .set_id(live_id!(onboard_icon_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening logo picker…");
    }

    /// Push history when `record_history` (both peer jumps and drill-downs).
    /// Entering or leaving Lock/Onboard clears stacks via `screen_keeps_history`
    /// so Back cannot return to setup after the main screen is open. During
    /// Onboard, nested Back still uses the explicit DeviceSetup parent pushed
    /// at owner-setup.
    fn set_screen(&mut self, st: &mut AppState, next: Screen, record_history: bool) {
        if st.screen == next {
            return;
        }
        if record_history
            && screen_keeps_history(st.screen)
            && screen_keeps_history(next)
        {
            st.nav_back.push(st.screen);
            if st.nav_back.len() > 32 {
                st.nav_back.remove(0);
            }
            st.nav_forward.clear();
        } else if !screen_keeps_history(next) || !screen_keeps_history(st.screen) {
            st.nav_back.clear();
            st.nav_forward.clear();
        }
        st.screen = next;
    }

    /// Drill-down under a peer (Settings → Team, Order → Floor pick, …).
    /// Pushes visit history; header ‹ shows when the stack (or an overlay)
    /// means Back can run — see [`header_back_visible`].
    fn go(&mut self, cx: &mut Cx, screen: Screen) {
        self.go_with(cx, screen, false);
    }

    /// Chrome peer switch (bottom nav / side rail / shortcuts).
    /// Pushes visit history like a browser tab change; header ‹ appears on
    /// the destination when `nav_back` is non-empty. Clears in-screen layers
    /// so a Settings section / Order sheet cannot linger. Nested drill-downs
    /// use [`go`]. Platform Back/Forward: [`navigate_back`] / [`navigate_forward`].
    fn go_peer(&mut self, cx: &mut Cx, screen: Screen) {
        self.go_with(cx, screen, true);
    }

    /// `is_peer`: chrome tab jump (clear in-screen layers). Both paths record
    /// visit history when `screen_keeps_history` allows.
    fn go_with(&mut self, cx: &mut Cx, screen: Screen, is_peer: bool) {
        {
            let mut st = app_state().lock().unwrap();
            // Pick-table mode is set only by `pick_table_for_order` after go.
            // Floor from nav must not keep a stale pick banner.
            let was_picking = st.picking_table;
            let pick_from = st.picking_from_table_id.clone();
            st.picking_table = false;
            if screen != Screen::Floor {
                st.picking_from_table_id = None;
            }
            // Left Floor mid-pick via nav (not Back to order): restore the
            // previous table so Order and Floor stay on the same ticket.
            if was_picking && screen != Screen::Floor && screen != Screen::Pos {
                if let Some(from) = pick_from {
                    let _ = st.load_table_cart(&from);
                }
            }
            st.suggest_open = None;
            if st.must_confirm_device_role && screen != Screen::DeviceSetup {
                st.status = st.t("device_use_q");
                self.set_screen(&mut st, Screen::DeviceSetup, false);
            } else if st.awaiting_app_template && screen != Screen::DeviceSetup {
                self.set_screen(&mut st, Screen::DeviceSetup, false);
            } else if st.device_role == DeviceRole::Client
                && !st.client_paired
                && !matches!(
                    screen,
                    Screen::ClientPair | Screen::DeviceSetup | Screen::Lock
                )
            {
                st.status = "Pair this extra till to the shop device first".into();
                self.set_screen(&mut st, Screen::ClientPair, false);
            } else if st.session.is_none()
                && !matches!(
                    screen,
                    Screen::Lock
                        | Screen::DeviceSetup
                        | Screen::Onboard
                        | Screen::ClientPair
                )
            {
                st.status = "Unlock first".into();
                self.set_screen(&mut st, Screen::Lock, false);
            } else if st.session.is_some()
                && st.owner_setup_complete
                && !st.onboarding_complete
                && st.device_role.is_host()
                && !matches!(
                    screen,
                    Screen::Onboard | Screen::DeviceSetup | Screen::Lock
                )
            {
                self.set_screen(&mut st, Screen::Onboard, false);
            } else {
                if is_peer {
                    // Clear in-screen layers so ‹ does not linger on the next
                    // peer. Keep nav_back / nav_forward for platform gestures.
                    st.settings_panel = None;
                    st.line_action = None;
                    st.move_panel_open = false;
                    st.split_mode = false;
                    st.split_pick.clear();
                }
                // Team edit target must not leak onto other screens (station
                // checklist would steal every suggest via open_suggest).
                if st.screen == Screen::Team && screen != Screen::Team {
                    st.team_staff_id = None;
                    if st.suggest_open == Some(SuggestField::TeamKotStations) {
                        st.suggest_open = None;
                    }
                }
                self.set_screen(&mut st, screen, true);
                if screen == Screen::Settings {
                    // Settings tab / peer opens on its hub. History Back into
                    // Settings (set_screen, not go_with) keeps the section.
                    st.settings_panel = None;
                }
                // Person caps drive these screens on a Client till too —
                // shop writes go over `/books`. Do not demote capable people.
                // Order desk: always show the Server ticket for the open table
                // (Floor nav round-trip must not leave a stale local cart).
                if screen == Screen::Pos
                    && let Some(tid) = st.selected_table_id.clone()
                {
                    let _ = st.load_table_cart(&tid);
                }
            }
        }
        // Settings: `sync_chrome` paints the shell and returns. It must not
        // run the full chrome rebuild (that left Settings blank for seconds).
        // Bill & KOT / Printers build only inside `open_settings_panel`.
        diag::breadcrumb(
            if is_peer { "go_peer" } else { "go" },
            format!("{screen:?}"),
        );
        self.sync_chrome(cx);
    }

    /// Settings hub vs one open panel (avoids one endless dump of every control).
    fn sync_settings_panels(&mut self, cx: &mut Cx) {
        let panel = {
            let st = app_state().lock().unwrap();
            st.settings_panel
        };
        let on_hub = panel.is_none();
        self.ui.view(cx, ids!(settings_hub)).set_visible(cx, on_hub);
        // Body Back removed — header_back (left of brand) is the only back control.
        self.ui
            .button(cx, ids!(settings_detail_back))
            .set_visible(cx, false);
        let panels: &[(&[LiveId], &str)] = &[
            (ids!(settings_panel_language), "language"),
            (ids!(settings_panel_country), "country"),
            (ids!(settings_panel_tills), "tills"),
            (ids!(settings_panel_brand), "brand"),
            (ids!(settings_panel_units), "units"),
            (ids!(settings_panel_pickers), "pickers"),
            (ids!(settings_panel_printers), "printers"),
            (ids!(settings_panel_bill), "bill"),
            (ids!(settings_panel_selling), "selling"),
            (ids!(settings_panel_alerts), "alerts"),
            (ids!(settings_panel_backup), "backup"),
            (ids!(settings_panel_diag), "diag"),
        ];
        for (id, key) in panels {
            self.ui
                .view(cx, id)
                .set_visible(cx, panel == Some(*key));
        }
        if panel == Some("selling") {
            self.sync_selling_panel(cx);
        }
        if panel == Some("bill") && !app_state().lock().unwrap().bill_kot_tab_kot {
            self.sync_bill_format_ui(cx);
        }
        {
            let st = app_state().lock().unwrap();
            let s = |k: &str| st.t(k);
            let shop = st
                .session
                .as_ref()
                .map(|m| m.can_access_settings())
                .unwrap_or(false);
            let my_pin_only = st.session.is_some() && !shop;
            {
                let help = self.ui.widget(cx, ids!(settings_hub_help)).as_info_help();
                help.set_title(cx, &s("nav_settings"));
                help.set_nav_icon(cx, NavTab::Settings);
                help.set_tip(&plain_display_text(&s(if my_pin_only {
                    "settings_hub_help_my_pin"
                } else {
                    "settings_hub_help"
                })));
            }
            self.ui
                .button(cx, ids!(settings_detail_back))
                .set_text(cx, &s("back"));
            // Shop-admin hub rows — hide for Waiter/Kitchen/Cashier (My PIN only).
            for (id, key) in [
                (ids!(settings_hub_language), "settings_sec_language"),
                (ids!(settings_hub_country), "settings_sec_country"),
                (ids!(settings_hub_tills), "settings_sec_tills"),
                (ids!(settings_hub_brand), "settings_sec_brand"),
                (ids!(settings_hub_units), "settings_sec_units"),
                (ids!(settings_hub_pickers), "settings_sec_pickers"),
                (ids!(settings_hub_printers), "settings_sec_printers"),
                (ids!(settings_hub_bill), "settings_sec_bill"),
                (ids!(settings_hub_selling), "settings_sec_selling"),
                (ids!(settings_hub_alerts), "settings_sec_alerts"),
                (ids!(settings_hub_backup), "settings_sec_backup"),
                (ids!(settings_hub_diag), "settings_sec_diag"),
                (ids!(settings_hub_sidebar), "sidebar_settings"),
            ] {
                self.ui.button(cx, id).set_text(cx, &s(key));
                self.ui.button(cx, id).set_visible(cx, shop);
            }
            self.ui.button(cx, ids!(settings_hub_team)).set_text(
                cx,
                &s(if my_pin_only {
                    "my_pin_settings"
                } else {
                    "team_pins"
                }),
            );
            self.ui
                .button(cx, ids!(settings_hub_team))
                .set_visible(cx, st.session.is_some());
            self.ui
                .button(cx, ids!(station_remove))
                .set_text(cx, &s("station_remove"));
            self.ui
                .text_input(cx, ids!(station_remove_input))
                .set_empty_text(cx, s("station_remove_ph"));
        }
        if panel == Some("alerts") {
            let (secs, ph, help, save) = {
                let st = app_state().lock().unwrap();
                (
                    format!("{}", st.snack_secs.round() as i64),
                    st.t("snack_secs_ph"),
                    st.t("alerts_help"),
                    st.t("snack_secs_save"),
                )
            };
            self.ui
                .widget(cx, ids!(h_alerts))
                .set_text(cx, &plain_display_text(
                    &app_state().lock().unwrap().t("settings_sec_alerts"),
                ));
            self.ui
                .widget(cx, ids!(h_alerts_help))
                .set_text(cx, &plain_display_text(&help));
            self.ui
                .text_input(cx, ids!(snack_secs_input))
                .set_empty_text(cx, ph);
            let field = self.ui.text_input(cx, ids!(snack_secs_input));
            if !cx.has_key_focus(field.area()) {
                field.set_text(cx, &secs);
            }
            self.ui
                .button(cx, ids!(snack_secs_save))
                .set_text(cx, &save);
        }
        if panel == Some("diag") {
            self.sync_diag_panel(cx);
        }
        // Hub ↔ section: always refresh ‹ (do not rely on a later full sync_chrome).
        self.paint_header_back_chrome(cx);
    }

    /// Show one Settings section (or the hub with `None`). Lightweight — no
    /// full `sync_chrome` (that can freeze phones on Settings taps).
    fn open_settings_panel(&mut self, cx: &mut Cx, panel: Option<&'static str>) {
        {
            let mut st = app_state().lock().unwrap();
            st.settings_panel = panel;
            st.suggest_open = None;
        }
        self.ui
            .view(cx, ids!(settings_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        diag::breadcrumb("settings_panel", panel.unwrap_or("hub"));
        self.sync_settings_panels(cx);
        // Heavy section paints only when that section is opened — not on every
        // Settings hub visit or every sync_chrome while Settings is open.
        if panel == Some("bill") {
            // Fresh open lands on Bill tab (KOT is one tap away).
            app_state().lock().unwrap().bill_kot_tab_kot = false;
            self.fill_bill_format_fields(cx);
            self.sync_bill_kot_tabs(cx);
        }
        if panel == Some("printers") {
            self.paint_printers_panel(cx);
        }
        if panel == Some("backup") {
            self.paint_backup_status(cx);
        }
        if panel == Some("units") {
            let units_rows = {
                let st = app_state().lock().unwrap();
                st.store
                    .as_ref()
                    .map(|s| units_report_rows(s, |k| st.t(k)))
                    .unwrap_or_default()
            };
            self.set_report_rows(cx, ids!(units_label), units_rows);
        }
        if panel == Some("pickers") {
            let sibling_rows = {
                let st = app_state().lock().unwrap();
                st.store
                    .as_ref()
                    .map(|s| sibling_pickers_rows(s, |k| st.t(k)))
                    .unwrap_or_default()
            };
            self.set_report_rows(cx, ids!(sibling_pickers_label), sibling_rows);
        }
        self.ui.redraw(cx);
    }

    /// Settings → Diagnostics: status, level chips, support/share actions.
    fn ensure_diag_ready(&self) {
        let base = app_state().lock().unwrap().data_base.clone();
        if let Some(base) = base {
            diag::ensure_ready(&base, store::APP_ID, env!("CARGO_PKG_VERSION"));
        } else if let Ok(dir) = store::data_dir() {
            diag::ensure_ready(&dir, store::APP_ID, env!("CARGO_PKG_VERSION"));
        }
    }

    fn sync_diag_panel(&mut self, cx: &mut Cx) {
        self.ensure_diag_ready();
        let st = app_state().lock().unwrap();
        let s = |k: &str| st.t(k);
        self.ui
            .widget(cx, ids!(h_diag))
            .set_text(cx, &plain_display_text(&s("settings_sec_diag")));
        self.ui
            .widget(cx, ids!(diag_privacy_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("diag_privacy_help")));
        self.ui
            .widget(cx, ids!(h_diag_level))
            .set_text(cx, &plain_display_text(&s("diag_level")));
        {
            let help = self.ui.widget(cx, ids!(diag_send_help)).as_info_help();
            help.set_title(cx, &s("diag_send_heading"));
            help.set_tip(&plain_display_text(&s("diag_send_help")));
        }
        let start_lbl = s("diag_support_start");
        let stop_lbl = s("diag_support_stop");
        for id in [
            ids!(diag_support_start),
            ids!(diag_support_start_on),
        ] {
            self.ui.button(cx, id).set_text(cx, &start_lbl);
        }
        for id in [ids!(diag_support_stop), ids!(diag_support_stop_on)] {
            self.ui.button(cx, id).set_text(cx, &stop_lbl);
        }
        let diag_busy = diag_export_busy().load(std::sync::atomic::Ordering::SeqCst);
        let diag_send = diag_busy && diag_export_for_send().load(std::sync::atomic::Ordering::SeqCst);
        if !(diag_busy && !diag_send) {
            self.ui
                .button(cx, ids!(diag_export_local))
                .set_text(cx, &s("diag_export_local"));
        }
        self.ui
            .button(cx, ids!(diag_approve_send))
            .set_text(cx, &s("diag_approve_send"));
        if !diag_send {
            self.ui
                .button(cx, ids!(diag_export_send))
                .set_text(cx, &s("diag_export_send"));
        }
        self.ui
            .button(cx, ids!(diag_export_local_cancel))
            .set_text(cx, &s("cancel"));
        self.ui
            .button(cx, ids!(diag_export_send_cancel))
            .set_text(cx, &s("cancel"));
        self.ui
            .button(cx, ids!(diag_clear))
            .set_text(cx, &s("diag_clear"));
        for (id, key) in [
            (ids!(diag_level_error), "diag_level_error"),
            (ids!(diag_level_error_on), "diag_level_error"),
            (ids!(diag_level_warn), "diag_level_warn"),
            (ids!(diag_level_warn_on), "diag_level_warn"),
            (ids!(diag_level_info), "diag_level_info"),
            (ids!(diag_level_info_on), "diag_level_info"),
            (ids!(diag_level_debug), "diag_level_debug"),
            (ids!(diag_level_debug_on), "diag_level_debug"),
        ] {
            self.ui.button(cx, id).set_text(cx, &s(key));
        }
        drop(st);
        self.ui
            .widget(cx, ids!(diag_status_label))
            .set_text(cx, &plain_display_text(&diag::status_text()));
        let status = diag::status();
        // Never invent Info when diag is still down — that lied next to a warn toast.
        let level = status.as_ref().map(|s| s.min_level);
        let support = status.as_ref().is_some_and(|s| s.support_active);
        let approved = status.as_ref().is_some_and(|s| s.send_approval_ready);
        let approved_secs = status
            .as_ref()
            .map(|s| s.send_approval_remaining_secs)
            .unwrap_or(0);
        // paint_choice_pair clears hover/focus animators so a deselected
        // SegBtn cannot stay cream-lit next to the real selection.
        for (off, on, lv) in [
            (ids!(diag_level_error), ids!(diag_level_error_on), diag::Level::Error),
            (ids!(diag_level_warn), ids!(diag_level_warn_on), diag::Level::Warn),
            (ids!(diag_level_info), ids!(diag_level_info_on), diag::Level::Info),
            (ids!(diag_level_debug), ids!(diag_level_debug_on), diag::Level::Debug),
        ] {
            self.paint_choice_pair(cx, off, on, level == Some(lv));
        }
        // Session off → Start is the filled CTA; session on → End is filled.
        self.paint_choice_pair(
            cx,
            ids!(diag_support_start),
            ids!(diag_support_start_on),
            !support,
        );
        self.paint_choice_pair(
            cx,
            ids!(diag_support_stop),
            ids!(diag_support_stop_on),
            support,
        );
        // Short face — the snack still uses the longer `diag_send_approved`.
        let approved_lbl = format!("Approved · {approved_secs}s");
        self.ui
            .button(cx, ids!(diag_approve_send_on))
            .set_text(cx, &approved_lbl);
        self.paint_choice_pair(
            cx,
            ids!(diag_approve_send),
            ids!(diag_approve_send_on),
            approved,
        );
    }

    fn handle_diag_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let on_diag = app_state().lock().unwrap().settings_panel == Some("diag");
        if !on_diag {
            return;
        }
        for (id, level) in [
            (ids!(diag_level_error), diag::Level::Error),
            (ids!(diag_level_error_on), diag::Level::Error),
            (ids!(diag_level_warn), diag::Level::Warn),
            (ids!(diag_level_warn_on), diag::Level::Warn),
            (ids!(diag_level_info), diag::Level::Info),
            (ids!(diag_level_info_on), diag::Level::Info),
            (ids!(diag_level_debug), diag::Level::Debug),
            (ids!(diag_level_debug_on), diag::Level::Debug),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                self.ensure_diag_ready();
                diag::end_support_session();
                diag::set_min_level(level);
                self.sync_diag_panel(cx);
                let shown = diag::status()
                    .map(|s| s.min_level)
                    .unwrap_or(level);
                self.show_ok(cx, format!("Diagnostics level: {}", shown.as_str()));
                return;
            }
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(diag_support_start),
            ids!(diag_support_start_on),
        ) {
            self.ensure_diag_ready();
            diag::start_support_session(None);
            self.sync_diag_panel(cx);
            if diag::status().is_some_and(|s| s.support_active) {
                self.show_ok(cx, app_t("diag_support_started"));
            } else {
                self.show_error(cx, "Diagnostics not ready");
            }
            return;
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(diag_support_stop),
            ids!(diag_support_stop_on),
        ) {
            self.ensure_diag_ready();
            diag::end_support_session();
            self.sync_diag_panel(cx);
            if diag::status().is_some_and(|s| !s.support_active) {
                self.show_ok(cx, app_t("diag_support_ended"));
            } else {
                self.show_error(cx, "Diagnostics not ready");
            }
            return;
        }
        if self.ui.button(cx, ids!(diag_export_local_cancel)).clicked(actions) {
            request_books_job_cancel();
            self.paint_submit_busy_chrome(cx);
            return;
        }
        if self.ui.button(cx, ids!(diag_export_send_cancel)).clicked(actions) {
            request_books_job_cancel();
            self.paint_submit_busy_chrome(cx);
            return;
        }
        if self.ui.button(cx, ids!(diag_export_local)).clicked(actions) {
            if diag_export_busy().load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            self.paint_submit_busy_chrome(cx);
            diag_export_job(false);
            self.paint_submit_busy_chrome(cx);
            return;
        }
        if self.choice_chip_clicked(
            cx,
            actions,
            ids!(diag_approve_send),
            ids!(diag_approve_send_on),
        ) {
            self.ensure_diag_ready();
            diag::approve_send();
            self.sync_diag_panel(cx);
            if diag::status().is_some_and(|s| s.send_approval_ready) {
                self.show_warn(cx, app_t("diag_send_approved"));
            } else {
                self.show_error(cx, "Diagnostics not ready");
            }
            return;
        }
        if self.ui.button(cx, ids!(diag_export_send)).clicked(actions) {
            if diag_export_busy().load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            diag_export_job(true);
            self.paint_submit_busy_chrome(cx);
            return;
        }
        if self.ui.button(cx, ids!(diag_clear)).clicked(actions) {
            match diag::clear_logs() {
                Ok(()) => {
                    self.sync_diag_panel(cx);
                    self.show_ok(cx, app_t("diag_cleared"));
                }
                Err(e) => self.show_error(cx, e),
            }
        }
    }

    /// Settings → Schemes & sales: offers, ₹0 sales, aggregator file import.
    fn sync_selling_panel(&mut self, cx: &mut Cx) {
        let (scheme_rows, allow_zero) = {
            let st = app_state().lock().unwrap();
            let scheme_rows = st
                .store
                .as_ref()
                .map(|store| {
                    let schemes = store.list_schemes().unwrap_or_default();
                    if schemes.is_empty() {
                        vec![ReportRow::Line("No schemes yet.".into())]
                    } else {
                        schemes
                            .into_iter()
                            .map(|s| {
                                ReportRow::Kv(
                                    s.name,
                                    format!(
                                        "{}{}",
                                        s.kind.as_str(),
                                        if s.active { "" } else { " · off" }
                                    ),
                                )
                            })
                            .collect()
                    }
                })
                .unwrap_or_else(|| vec![ReportRow::Line("No schemes yet.".into())]);
            let allow_zero = st.store.as_ref().and_then(|s| s.allow_zero_sales().ok());
            (scheme_rows, allow_zero)
        };
        self.set_report_rows(cx, ids!(schemes_label), scheme_rows);
        self.paint_choice_pair(
            cx,
            ids!(zero_sales_on_btn),
            ids!(zero_sales_on_btn_on),
            allow_zero == Some(true),
        );
        self.paint_choice_pair(
            cx,
            ids!(zero_sales_off_btn),
            ids!(zero_sales_off_btn_on),
            allow_zero == Some(false),
        );
    }

    /// Settings → Bill format shows what the books hold.
    fn fill_bill_format_fields(&mut self, cx: &mut Cx) {
        let (header, fssai, footer, upi_vpa, upi_payee, stamp_hint, pay_qr_hint) = {
            let st = app_state().lock().unwrap();
            let lang = st.locale.language.clone();
            let Some(store) = st.store.as_ref() else {
                return;
            };
            let stamp_hint = if store.print_stamp_path().ok().flatten().is_some() {
                locale::t(&lang, "stamp_set")
            } else {
                locale::t(&lang, "stamp_none")
            };
            let pay_qr_hint = if store.payment_qr_path().ok().flatten().is_some() {
                locale::t(&lang, "payment_qr_set")
            } else {
                locale::t(&lang, "payment_qr_none")
            };
            (
                store.bill_header_lines().unwrap_or_default()
                    .into_iter()
                    .filter(|l| !l.starts_with("FSSAI"))
                    .collect::<Vec<_>>()
                    .join(" | "),
                store.fssai_no().unwrap_or_default(),
                store.bill_footer().unwrap_or_default(),
                store.upi_vpa().unwrap_or_default(),
                store.upi_payee().unwrap_or_default(),
                stamp_hint,
                pay_qr_hint,
            )
        };
        {
            let mut st = app_state().lock().unwrap();
            st.bill_slip_draft = st
                .store
                .as_ref()
                .and_then(|s| s.bill_slip_format().ok())
                .unwrap_or_else(|| {
                    let mut fmt = print_share::BillSlipFormat::default();
                    fmt.header = header.clone();
                    fmt.fssai = fssai.clone();
                    fmt.footer = footer.clone();
                    fmt.upi_vpa = upi_vpa.clone();
                    fmt.upi_payee = upi_payee.clone();
                    fmt
                });
        }
        self.sync_bill_format_ui(cx);
        self.ui
            .widget(cx, ids!(print_stamp_status))
            .set_text(cx, &plain_display_text(&stamp_hint));
        self.ui
            .widget(cx, ids!(payment_qr_status))
            .set_text(cx, &plain_display_text(&pay_qr_hint));
        self.sync_std_print_ui(cx);
        self.fill_kot_format_fields(cx);
        self.sync_bill_kot_tabs(cx);
    }

    /// Settings → Bill & KOT format: Bill vs KOT tab under the section header.
    fn sync_bill_kot_tabs(&mut self, cx: &mut Cx) {
        let (on_kot, lang) = {
            let st = app_state().lock().unwrap();
            (st.bill_kot_tab_kot, st.locale.language.clone())
        };
        let s = |k: &str| locale::t(&lang, k);
        {
            let help = self.ui.widget(cx, ids!(bill_kot_panel_help)).as_info_help();
            help.set_title(cx, &s("settings_sec_bill"));
            help.set_tip(&plain_display_text(&s("bill_kot_panel_help")));
        }
        for (idle, on, key) in [
            (ids!(bill_kot_tab_bill), ids!(bill_kot_tab_bill_on), "bill_kot_tab_bill"),
            (ids!(bill_kot_tab_kot), ids!(bill_kot_tab_kot_on), "bill_kot_tab_kot"),
        ] {
            let t = s(key);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, on).set_text(cx, &t);
        }
        self.paint_choice_pair(cx, ids!(bill_kot_tab_bill), ids!(bill_kot_tab_bill_on), !on_kot);
        self.paint_choice_pair(cx, ids!(bill_kot_tab_kot), ids!(bill_kot_tab_kot_on), on_kot);
        self.ui.view(cx, ids!(bill_format_body)).set_visible(cx, !on_kot);
        self.ui.view(cx, ids!(kot_format_body)).set_visible(cx, on_kot);
        self.sync_content_width_ui(cx);
        self.sync_job_drive_ui(cx);
        // After the Bill body is shown. Painting earlier left the script
        // names ("Shop", "Address") on the rows.
        if !on_kot {
            self.sync_bill_format_ui(cx);
        }
    }

    /// Portrait / landscape and delivery for the bill and for the kitchen ticket.
    fn sync_job_drive_ui(&mut self, cx: &mut Cx) {
        let (bill_land, bill_del, kot_land, kot_del, bill_n, open_bill) = {
            let st = app_state().lock().unwrap();
            let drive = |id: &str| {
                st.printer_draft
                    .formats
                    .iter()
                    .find(|f| f.id == id)
                    .map(|f| (f.landscape, f.delivery_id().to_string()))
                    .unwrap_or((false, print_share::DELIVERY_SYSTEM_SPOOLER.into()))
            };
            let (bill_land, bill_del) = drive(print_share::FORMAT_STANDARD_PORTRAIT);
            let (kot_land, kot_del) = drive(print_share::FORMAT_KOT);
            let bill_n = st
                .printer_draft
                .route_for(print_share::RouteKey::Bill)
                .len();
            (
                bill_land,
                bill_del,
                kot_land,
                kot_del,
                bill_n,
                st.suggest_open == Some(SuggestField::BillPrinters),
            )
        };
        self.paint_choice_pair(
            cx,
            ids!(bill_layout_portrait),
            ids!(bill_layout_portrait_on),
            !bill_land,
        );
        self.paint_choice_pair(
            cx,
            ids!(bill_layout_landscape),
            ids!(bill_layout_landscape_on),
            bill_land,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_layout_portrait),
            ids!(kot_layout_portrait_on),
            !kot_land,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_layout_landscape),
            ids!(kot_layout_landscape_on),
            kot_land,
        );
        let mut paint_del = |this: &mut Self, os, os_on, raw, raw_on, esc, esc_on, del: &str| {
            this.paint_choice_pair(
                cx,
                os,
                os_on,
                del == print_share::DELIVERY_SYSTEM_SPOOLER || del.is_empty(),
            );
            this.paint_choice_pair(cx, raw, raw_on, del == print_share::DELIVERY_RAW_DEVICE);
            this.paint_choice_pair(cx, esc, esc_on, del == print_share::DELIVERY_ESCPOS);
        };
        paint_del(
            self,
            ids!(bill_deliv_os),
            ids!(bill_deliv_os_on),
            ids!(bill_deliv_raw),
            ids!(bill_deliv_raw_on),
            ids!(bill_deliv_escpos),
            ids!(bill_deliv_escpos_on),
            &bill_del,
        );
        paint_del(
            self,
            ids!(kot_deliv_os),
            ids!(kot_deliv_os_on),
            ids!(kot_deliv_raw),
            ids!(kot_deliv_raw_on),
            ids!(kot_deliv_escpos),
            ids!(kot_deliv_escpos_on),
            &kot_del,
        );
        let label = if bill_n == 0 {
            app_t("bill_printers_dd")
        } else {
            app_t("bill_printers_dd_n").replace("{n}", &bill_n.to_string())
        };
        let face = if open_bill {
            app_t("pick_done_dd").replace("{label}", &label)
        } else {
            label
        };
        self.ui.button(cx, ids!(bill_printers_dd)).set_text(cx, &face);
    }

    fn apply_job_drive(
        &mut self,
        cx: &mut Cx,
        standard: bool,
        landscape: Option<bool>,
        delivery: Option<&str>,
    ) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.device_role == DeviceRole::Client {
                return Err(st.t("printers_host_only"));
            }
            let id = if standard {
                print_share::FORMAT_STANDARD_PORTRAIT
            } else {
                print_share::FORMAT_KOT
            };
            let (cur_land, cur_del) = st
                .printer_draft
                .formats
                .iter()
                .find(|f| f.id == id)
                .map(|f| (f.landscape, f.delivery_id().to_string()))
                .unwrap_or((false, print_share::DELIVERY_SYSTEM_SPOOLER.into()));
            let land = landscape.unwrap_or(cur_land);
            let del = delivery.unwrap_or(cur_del.as_str()).to_string();
            st.printer_draft.set_job_drive(standard, land, &del);
            st.save_printer_draft()?;
            print_share::configure(st.printer_draft.clone());
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_job_drive_ui(cx);
        let on_kot = app_state().lock().unwrap().bill_kot_tab_kot;
        if on_kot {
            self.sync_kot_format_ui(cx);
        } else {
            self.sync_bill_format_ui(cx);
        }
    }

    /// Content width sits above the Bill | KOT body. Each tab has its own
    /// millimetre width (bill default 210, KOT default 80).
    fn sync_content_width_ui(&mut self, cx: &mut Cx) {
        let (mm, on_kot, lang) = {
            let st = app_state().lock().unwrap();
            let on_kot = st.bill_kot_tab_kot;
            let mm = if on_kot {
                st.kot_slip_draft.shown_content_width_mm()
            } else {
                st.bill_slip_draft.shown_content_width_mm()
            };
            (mm, on_kot, st.locale.language.clone())
        };
        let s = |k: &str| locale::t(&lang, k);
        let heading = if on_kot {
            s("content_width_kot")
        } else {
            s("content_width_bill")
        };
        let tip = if on_kot {
            s("content_width_kot_help")
        } else {
            s("content_width_bill_help")
        };
        self.ui
            .widget(cx, ids!(h_content_width))
            .set_text(cx, &plain_display_text(&heading));
        self.ui
            .widget(cx, ids!(content_width_help))
            .as_info_help()
            .set_tip(&plain_display_text(&tip));
        let field = self.ui.text_input(cx, ids!(content_width_input));
        field.set_empty_text(cx, s("content_width_ph"));
        if !cx.has_key_focus(field.area()) {
            field.set_text(cx, &mm.to_string());
        }
        for (idle, on, choice, label) in [
            (ids!(content_width_58), ids!(content_width_58_on), 58u32, "58 mm — narrow slip"),
            (ids!(content_width_80), ids!(content_width_80_on), 80, "80 mm — wide slip"),
            (ids!(content_width_112), ids!(content_width_112_on), 112, "112 mm — 4 inch slip"),
            (ids!(content_width_210), ids!(content_width_210_on), 210, "210 mm — page"),
        ] {
            self.ui.button(cx, idle).set_text(cx, label);
            self.ui.button(cx, on).set_text(cx, label);
            self.paint_choice_pair(cx, idle, on, mm == choice);
        }
    }

    fn apply_content_width(&mut self, cx: &mut Cx, mm: u32) {
        let mm = mm.clamp(40, 210);
        let on_kot = app_state().lock().unwrap().bill_kot_tab_kot;
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let msg = st.books_op(BooksOp::SetContentWidth {
                mm,
                kot: on_kot,
            })?;
            let cols = print_share::wrap_cols_for_paper_mm(mm);
            if on_kot {
                st.kot_slip_draft.content_width_mm = mm;
                st.kot_slip_draft.chars_per_line = cols;
                if st.device_role != DeviceRole::Client {
                    st.printer_draft.set_kot_content_line(cols);
                    print_share::configure(st.printer_draft.clone());
                }
            } else {
                st.bill_slip_draft.content_width_mm = mm;
                if st.device_role != DeviceRole::Client {
                    st.printer_draft.set_bill_content_line(cols);
                    print_share::configure(st.printer_draft.clone());
                }
            }
            Ok(msg)
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_content_width_ui(cx);
        if on_kot {
            self.sync_kot_format_ui(cx);
        } else {
            self.sync_bill_format_ui(cx);
        }
    }

    /// Print the kitchen-slip preview on the checked test printers, using the
    /// format currently on screen (width, layout, delivery, field styles).
    fn print_kot_format_preview(&mut self, cx: &mut Cx) {
        let title = self.ui.text_input(cx, ids!(kot_title_input)).text();
        let footer = self.ui.text_input(cx, ids!(kot_footer_input)).text();
        let size_now = self.ui.text_input(cx, ids!(kot_field_size_input)).text();
        let result = (|| -> Result<String, String> {
            let st = app_state().lock().unwrap();
            if st.device_role == DeviceRole::Client {
                return Err(st.t("printers_host_only"));
            }
            let pids = st.test_printer_ids.clone();
            if pids.is_empty() {
                return Err(st.t("test_kot_pick_first"));
            }
            let mut draft = st.kot_slip_draft.clone();
            draft.title = title;
            draft.footer = footer;
            if let Ok(raw) = size_now.trim().parse::<u32>() {
                let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
                let kind = st.kot_edit_kind;
                let field = st.kot_style_field;
                draft.kind_styles.get_mut(kind).get_mut(field).size_em = em;
                if field == print_share::KotStyleField::ItemQty {
                    draft.em = em;
                }
            }
            let kind = st.kot_edit_kind;
            let shop = st
                .store
                .as_ref()
                .and_then(|s| s.venue_title().ok())
                .unwrap_or_else(|| st.display_venue_title());
            let text = draft.render_sample_kind(&shop, kind);
            let mut live = st.printer_draft.clone();
            live.set_kot_content_line(print_share::wrap_cols_for_paper_mm(
                draft.shown_content_width_mm(),
            ));
            if let Some(f) = live
                .formats
                .iter_mut()
                .find(|f| f.id == print_share::FORMAT_KOT)
            {
                draft.apply_to_print_format_for_kind(f, kind);
            }
            let dir = st
                .store
                .as_ref()
                .and_then(|s| s.db_path().parent().map(|p| p.to_path_buf()));
            let n = pids.len();
            drop(st);
            let mut ok = 0usize;
            let mut last_err: Option<String> = None;
            for pid in &pids {
                match print_share::print_one_printer(
                    &live,
                    pid,
                    print_share::PrintKind::Kot,
                    &text,
                    dir.as_deref(),
                ) {
                    Ok(share) if share.opened => ok += 1,
                    Ok(_) => {
                        last_err = print_share::take_problems().into_iter().next();
                    }
                    Err(e) => last_err = Some(e),
                }
            }
            if ok == 0 {
                Err(last_err.unwrap_or_else(|| app_t("test_print_no_printer")))
            } else if ok == 1 && n == 1 {
                Ok(app_t("test_kot_sent"))
            } else {
                Ok(app_t("test_kot_sent_n").replace("{n}", &ok.to_string()))
            }
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
    }

    /// Paint the one bill-line editor and the preview from the draft.
    fn sync_bill_format_ui(&mut self, cx: &mut Cx) {
        // While a box is being typed, the draft must already hold that text
        // so the picture and a save in the same moment agree.
        let size_in = self.ui.text_input(cx, ids!(bill_field_size_input));
        if cx.has_key_focus(size_in.area()) {
            if let Ok(raw) = size_in.text().trim().parse::<u32>() {
                let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
                let mut st = app_state().lock().unwrap();
                let field = st.bill_style_field;
                st.bill_slip_draft.style_mut(field).size_em = em;
            }
        }
        let value_in = self.ui.text_input(cx, ids!(bill_field_value));
        if cx.has_key_focus(value_in.area()) {
            self.commit_bill_field_value(cx);
        }
        let (draft, field, shop, when) = {
            let st = app_state().lock().unwrap();
            let shop = st
                .store
                .as_ref()
                .and_then(|s| s.venue_title().ok())
                .unwrap_or_else(|| st.display_venue_title());
            let when = st
                .store
                .as_ref()
                .and_then(|s| s.fiscal_settings().ok())
                .map(|f| f.format_now())
                .unwrap_or_default();
            (
                st.bill_slip_draft.clone(),
                st.bill_style_field,
                shop,
                when,
            )
        };
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |k: &str| locale::t(&lang, k);
        let style = draft.style(field).clone();
        // Same line the kitchen ticket uses: name · show/hide · size · weight · align.
        let row_label = |kind: print_share::BillStyleField| {
            let row = draft.style(kind);
            let on_off = if row.show {
                s("kot_field_on")
            } else {
                s("kot_field_off")
            };
            format!(
                "{} · {on_off} · {} · {} · {}",
                kind.label(),
                row.size_em,
                s(row.weight.locale_key()),
                s(row.align.locale_key())
            )
        };
        let order = draft.ordered();
        let order_idx = {
            let st = app_state().lock().unwrap();
            st.bill_field_edit_idx
        };
        // The field list is the print order. Move up / down reorders these rows.
        let slots = [
            (ids!(bill_pick_shop), ids!(bill_pick_shop_on)),
            (ids!(bill_pick_header), ids!(bill_pick_header_on)),
            (ids!(bill_pick_fssai), ids!(bill_pick_fssai_on)),
            (ids!(bill_pick_title), ids!(bill_pick_title_on)),
            (ids!(bill_pick_customer), ids!(bill_pick_customer_on)),
            (ids!(bill_pick_billno), ids!(bill_pick_billno_on)),
            (ids!(bill_pick_time), ids!(bill_pick_time_on)),
            (ids!(bill_pick_cashier), ids!(bill_pick_cashier_on)),
            (ids!(bill_pick_table), ids!(bill_pick_table_on)),
            (ids!(bill_pick_items), ids!(bill_pick_items_on)),
            (ids!(bill_pick_total), ids!(bill_pick_total_on)),
            (ids!(bill_pick_footer), ids!(bill_pick_footer_on)),
            (ids!(bill_pick_upi), ids!(bill_pick_upi_on)),
        ];
        for (i, (idle, on)) in slots.into_iter().enumerate() {
            if let Some(kind) = order.get(i) {
                let label = row_label(*kind);
                self.ui.button(cx, idle).set_text(cx, &label);
                self.ui.button(cx, on).set_text(cx, &label);
                self.paint_choice_pair(cx, idle, on, i == order_idx);
            }
        }
        self.ui.button(cx, ids!(bill_field_up)).set_text(cx, &s("kot_field_up"));
        self.ui.button(cx, ids!(bill_field_down)).set_text(cx, &s("kot_field_down"));
        self.ui
            .widget(cx, ids!(h_bill_copies))
            .set_text(cx, &plain_display_text(&s("kot_copies")));
        for (idle, on, n, key) in [
            (ids!(bill_copies_1), ids!(bill_copies_1_on), 1u32, "kot_copies_1"),
            (ids!(bill_copies_2), ids!(bill_copies_2_on), 2, "kot_copies_2"),
            (ids!(bill_copies_3), ids!(bill_copies_3_on), 3, "kot_copies_3"),
        ] {
            let text = s(key);
            self.ui.button(cx, idle).set_text(cx, &text);
            self.ui.button(cx, on).set_text(cx, &text);
            self.paint_choice_pair(cx, idle, on, draft.copies == n);
        }
        self.paint_choice_pair(cx, ids!(bill_field_show), ids!(bill_field_show_on), style.show);
        self.paint_choice_pair(cx, ids!(bill_field_hide), ids!(bill_field_hide_on), !style.show);
        let items_open = field == print_share::BillStyleField::Items;
        let total_open = field == print_share::BillStyleField::Total;
        self.ui.view(cx, ids!(bill_item_pieces)).set_visible(cx, items_open);
        self.ui.view(cx, ids!(bill_total_pieces)).set_visible(cx, total_open);
        if items_open {
            self.ui
                .widget(cx, ids!(h_bill_item_layout))
                .set_text(cx, &plain_display_text(&s("bill_item_layout")));
            self.ui
                .widget(cx, ids!(h_bill_item_header))
                .set_text(cx, &plain_display_text(&s("bill_item_header")));
            self.ui
                .widget(cx, ids!(h_bill_item_name))
                .set_text(cx, &plain_display_text(&s("bill_item_name")));
            self.ui
                .widget(cx, ids!(h_bill_item_qty))
                .set_text(cx, &plain_display_text(&s("bill_item_qty")));
            self.ui
                .widget(cx, ids!(h_bill_item_price))
                .set_text(cx, &plain_display_text(&s("bill_item_price")));
            self.ui
                .widget(cx, ids!(h_bill_item_amount))
                .set_text(cx, &plain_display_text(&s("bill_item_amount")));
            self.ui
                .widget(cx, ids!(h_bill_item_note))
                .set_text(cx, &plain_display_text(&s("bill_item_note")));
            for (idle, on, key) in [
                (ids!(bill_item_layout_columns), ids!(bill_item_layout_columns_on), "bill_item_layout_columns"),
                (ids!(bill_item_layout_stacked), ids!(bill_item_layout_stacked_on), "bill_item_layout_stacked"),
                (ids!(bill_item_header_on), ids!(bill_item_header_on_on), "bill_item_piece_on"),
                (ids!(bill_item_header_off), ids!(bill_item_header_off_on), "bill_item_piece_off"),
                (ids!(bill_item_name_on), ids!(bill_item_name_on_on), "bill_item_piece_on"),
                (ids!(bill_item_name_off), ids!(bill_item_name_off_on), "bill_item_piece_off"),
                (ids!(bill_item_qty_on), ids!(bill_item_qty_on_on), "bill_item_piece_on"),
                (ids!(bill_item_qty_off), ids!(bill_item_qty_off_on), "bill_item_piece_off"),
                (ids!(bill_item_price_on), ids!(bill_item_price_on_on), "bill_item_piece_on"),
                (ids!(bill_item_price_off), ids!(bill_item_price_off_on), "bill_item_piece_off"),
                (ids!(bill_item_amount_on), ids!(bill_item_amount_on_on), "bill_item_piece_on"),
                (ids!(bill_item_amount_off), ids!(bill_item_amount_off_on), "bill_item_piece_off"),
                (ids!(bill_item_note_on), ids!(bill_item_note_on_on), "bill_item_piece_on"),
                (ids!(bill_item_note_off), ids!(bill_item_note_off_on), "bill_item_piece_off"),
            ] {
                let t = s(key);
                self.ui.button(cx, idle).set_text(cx, &t);
                self.ui.button(cx, on).set_text(cx, &t);
            }
            let columns = draft.item_layout == print_share::BillItemLayout::Columns;
            self.paint_choice_pair(cx, ids!(bill_item_layout_columns), ids!(bill_item_layout_columns_on), columns);
            self.paint_choice_pair(cx, ids!(bill_item_layout_stacked), ids!(bill_item_layout_stacked_on), !columns);
            self.paint_choice_pair(cx, ids!(bill_item_header_on), ids!(bill_item_header_on_on), draft.item_show_header);
            self.paint_choice_pair(cx, ids!(bill_item_header_off), ids!(bill_item_header_off_on), !draft.item_show_header);
            self.ui
                .view(cx, ids!(bill_item_header_labels))
                .set_visible(cx, columns && draft.item_show_header);
            if columns && draft.item_show_header {
                self.ui
                    .widget(cx, ids!(h_bill_item_hdr_item))
                    .set_text(cx, &plain_display_text(&s("bill_item_hdr_item")));
                self.ui
                    .widget(cx, ids!(h_bill_item_hdr_qty))
                    .set_text(cx, &plain_display_text(&s("bill_item_hdr_qty")));
                self.ui
                    .widget(cx, ids!(h_bill_item_hdr_price))
                    .set_text(cx, &plain_display_text(&s("bill_item_hdr_price")));
                self.ui
                    .widget(cx, ids!(h_bill_item_hdr_amount))
                    .set_text(cx, &plain_display_text(&s("bill_item_hdr_amount")));
                for (id, val) in [
                    (ids!(bill_item_hdr_item_input), draft.item_header_item.as_str()),
                    (ids!(bill_item_hdr_qty_input), draft.item_header_qty.as_str()),
                    (ids!(bill_item_hdr_price_input), draft.item_header_price.as_str()),
                    (ids!(bill_item_hdr_amount_input), draft.item_header_amount.as_str()),
                ] {
                    let input = self.ui.text_input(cx, id);
                    if !cx.has_key_focus(input.area()) {
                        input.set_text(cx, val);
                    }
                }
            }
            self.paint_choice_pair(cx, ids!(bill_item_name_on), ids!(bill_item_name_on_on), draft.item_show_name);
            self.paint_choice_pair(cx, ids!(bill_item_name_off), ids!(bill_item_name_off_on), !draft.item_show_name);
            self.paint_choice_pair(cx, ids!(bill_item_qty_on), ids!(bill_item_qty_on_on), draft.item_show_qty);
            self.paint_choice_pair(cx, ids!(bill_item_qty_off), ids!(bill_item_qty_off_on), !draft.item_show_qty);
            self.paint_choice_pair(cx, ids!(bill_item_price_on), ids!(bill_item_price_on_on), draft.item_show_price);
            self.paint_choice_pair(cx, ids!(bill_item_price_off), ids!(bill_item_price_off_on), !draft.item_show_price);
            self.paint_choice_pair(cx, ids!(bill_item_amount_on), ids!(bill_item_amount_on_on), draft.item_show_amount);
            self.paint_choice_pair(cx, ids!(bill_item_amount_off), ids!(bill_item_amount_off_on), !draft.item_show_amount);
            self.paint_choice_pair(cx, ids!(bill_item_note_on), ids!(bill_item_note_on_on), draft.item_show_note);
            self.paint_choice_pair(cx, ids!(bill_item_note_off), ids!(bill_item_note_off_on), !draft.item_show_note);
            self.ui
                .view(cx, ids!(bill_item_col_metrics))
                .set_visible(cx, columns);
            if columns {
                self.ui
                    .widget(cx, ids!(h_bill_item_col_order))
                    .set_text(cx, &plain_display_text(&s("bill_item_col_order")));
                self.ui
                    .widget(cx, ids!(h_bill_item_col_sep))
                    .set_text(cx, &plain_display_text(&s("bill_item_col_sep")));
                self.ui
                    .button(cx, ids!(bill_col_ord_up))
                    .set_text(cx, &s("kot_field_up"));
                self.ui
                    .button(cx, ids!(bill_col_ord_down))
                    .set_text(cx, &s("kot_field_down"));
                let col_order = if draft.item_col_order.is_empty() {
                    print_share::BillItemCol::ALL
                        .iter()
                        .map(|c| c.id().to_string())
                        .collect::<Vec<_>>()
                } else {
                    draft.item_col_order.clone()
                };
                let col_idx = {
                    let st = app_state().lock().unwrap();
                    st.bill_item_col_edit_idx.min(col_order.len().saturating_sub(1))
                };
                let col_slots = [
                    (ids!(bill_col_ord_0), ids!(bill_col_ord_0_on)),
                    (ids!(bill_col_ord_1), ids!(bill_col_ord_1_on)),
                    (ids!(bill_col_ord_2), ids!(bill_col_ord_2_on)),
                    (ids!(bill_col_ord_3), ids!(bill_col_ord_3_on)),
                ];
                for (i, (idle, on)) in col_slots.iter().enumerate() {
                    let label = col_order
                        .get(i)
                        .and_then(|id| print_share::BillItemCol::from_id(id))
                        .map(|c| s(match c {
                            print_share::BillItemCol::Name => "bill_item_name",
                            print_share::BillItemCol::Qty => "bill_item_qty",
                            print_share::BillItemCol::Price => "bill_item_price",
                            print_share::BillItemCol::Amount => "bill_item_amount",
                        }))
                        .unwrap_or_else(|| format!("Col {}", i + 1));
                    self.ui.button(cx, *idle).set_text(cx, &label);
                    self.ui.button(cx, *on).set_text(cx, &label);
                    self.paint_choice_pair(cx, *idle, *on, i == col_idx);
                }
                for (idle, on, key) in [
                    (ids!(bill_item_sep_on), ids!(bill_item_sep_on_on), "bill_item_piece_on"),
                    (ids!(bill_item_sep_off), ids!(bill_item_sep_off_on), "bill_item_piece_off"),
                ] {
                    let t = s(key);
                    self.ui.button(cx, idle).set_text(cx, &t);
                    self.ui.button(cx, on).set_text(cx, &t);
                }
                self.paint_choice_pair(cx, ids!(bill_item_sep_on), ids!(bill_item_sep_on_on), draft.item_col_sep);
                self.paint_choice_pair(cx, ids!(bill_item_sep_off), ids!(bill_item_sep_off_on), !draft.item_col_sep);
                self.ui
                    .widget(cx, ids!(h_bill_item_col_gap))
                    .set_text(cx, &plain_display_text(&s("bill_item_col_gap")));
                self.ui
                    .widget(cx, ids!(h_bill_item_num_width))
                    .set_text(cx, &plain_display_text(&s("bill_item_num_width")));
                self.ui
                    .widget(cx, ids!(h_bill_item_num_align))
                    .set_text(cx, &plain_display_text(&s("bill_item_num_align")));
                self.ui
                    .widget(cx, ids!(h_bill_item_gap_em))
                    .set_text(cx, &plain_display_text(&s("bill_item_gap_em")));
                self.ui
                    .widget(cx, ids!(h_bill_item_qty_min))
                    .set_text(cx, &plain_display_text(&s("bill_item_qty_min")));
                self.ui
                    .widget(cx, ids!(h_bill_item_price_min))
                    .set_text(cx, &plain_display_text(&s("bill_item_price_min")));
                self.ui
                    .widget(cx, ids!(h_bill_item_amount_min))
                    .set_text(cx, &plain_display_text(&s("bill_item_amount_min")));
                for (idle, on, key) in [
                    (ids!(bill_item_gap_tight), ids!(bill_item_gap_tight_on), "bill_item_gap_tight"),
                    (ids!(bill_item_gap_comfort), ids!(bill_item_gap_comfort_on), "bill_item_gap_comfort"),
                    (ids!(bill_item_gap_wide), ids!(bill_item_gap_wide_on), "bill_item_gap_wide"),
                    (ids!(bill_item_num_compact), ids!(bill_item_num_compact_on), "bill_item_num_compact"),
                    (ids!(bill_item_num_comfort), ids!(bill_item_num_comfort_on), "bill_item_num_comfort"),
                    (ids!(bill_item_num_wide), ids!(bill_item_num_wide_on), "bill_item_num_wide"),
                    (ids!(bill_item_num_start), ids!(bill_item_num_start_on), "kot_align_start"),
                    (ids!(bill_item_num_center), ids!(bill_item_num_center_on), "kot_align_center"),
                    (ids!(bill_item_num_end), ids!(bill_item_num_end_on), "kot_align_end"),
                ] {
                    let t = s(key);
                    self.ui.button(cx, idle).set_text(cx, &t);
                    self.ui.button(cx, on).set_text(cx, &t);
                }
                let gap_preset = draft.item_col_gap_preset();
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_gap_tight),
                    ids!(bill_item_gap_tight_on),
                    (gap_preset - print_share::BILL_COL_GAP_TIGHT).abs() < 0.01,
                );
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_gap_comfort),
                    ids!(bill_item_gap_comfort_on),
                    (gap_preset - print_share::BILL_COL_GAP_COMFORT).abs() < 0.01,
                );
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_gap_wide),
                    ids!(bill_item_gap_wide_on),
                    (gap_preset - print_share::BILL_COL_GAP_WIDE).abs() < 0.01,
                );
                let num_preset = draft.item_num_min_preset();
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_num_compact),
                    ids!(bill_item_num_compact_on),
                    num_preset == print_share::BILL_NUM_MIN_COMPACT,
                );
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_num_comfort),
                    ids!(bill_item_num_comfort_on),
                    num_preset == print_share::BILL_NUM_MIN_COMFORT,
                );
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_num_wide),
                    ids!(bill_item_num_wide_on),
                    num_preset == print_share::BILL_NUM_MIN_WIDE,
                );
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_num_start),
                    ids!(bill_item_num_start_on),
                    draft.item_num_align == print_share::KotAlign::Start,
                );
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_num_center),
                    ids!(bill_item_num_center_on),
                    draft.item_num_align == print_share::KotAlign::Center,
                );
                self.paint_choice_pair(
                    cx,
                    ids!(bill_item_num_end),
                    ids!(bill_item_num_end_on),
                    draft.item_num_align == print_share::KotAlign::End,
                );
                let fmt_em = |val: f32| -> String {
                    if (val - val.round()).abs() < 0.05 {
                        format!("{}", val.round() as i32)
                    } else {
                        format!("{val:.2}")
                    }
                };
                for (id, val) in [
                    (ids!(bill_item_gap_em_input), draft.item_col_gap_em),
                    (ids!(bill_item_qty_min_input), draft.item_qty_min_em),
                    (ids!(bill_item_price_min_input), draft.item_price_min_em),
                    (ids!(bill_item_amount_min_input), draft.item_amount_min_em),
                ] {
                    let input = self.ui.text_input(cx, id);
                    if !cx.has_key_focus(input.area()) {
                        input.set_text(cx, &fmt_em(val));
                    }
                }
            }
        }
        if total_open {
            self.ui
                .widget(cx, ids!(h_bill_total_qty))
                .set_text(cx, &plain_display_text(&s("bill_total_qty")));
            self.ui
                .widget(cx, ids!(h_bill_total_subtotal))
                .set_text(cx, &plain_display_text(&s("bill_total_subtotal")));
            self.ui
                .widget(cx, ids!(h_bill_total_grand))
                .set_text(cx, &plain_display_text(&s("bill_total_grand")));
            for (idle, on, key) in [
                (ids!(bill_total_qty_on), ids!(bill_total_qty_on_on), "bill_item_piece_on"),
                (ids!(bill_total_qty_off), ids!(bill_total_qty_off_on), "bill_item_piece_off"),
                (ids!(bill_total_subtotal_on), ids!(bill_total_subtotal_on_on), "bill_item_piece_on"),
                (ids!(bill_total_subtotal_off), ids!(bill_total_subtotal_off_on), "bill_item_piece_off"),
                (ids!(bill_total_grand_on), ids!(bill_total_grand_on_on), "bill_item_piece_on"),
                (ids!(bill_total_grand_off), ids!(bill_total_grand_off_on), "bill_item_piece_off"),
            ] {
                let t = s(key);
                self.ui.button(cx, idle).set_text(cx, &t);
                self.ui.button(cx, on).set_text(cx, &t);
            }
            self.paint_choice_pair(cx, ids!(bill_total_qty_on), ids!(bill_total_qty_on_on), draft.total_show_qty);
            self.paint_choice_pair(cx, ids!(bill_total_qty_off), ids!(bill_total_qty_off_on), !draft.total_show_qty);
            self.paint_choice_pair(cx, ids!(bill_total_subtotal_on), ids!(bill_total_subtotal_on_on), draft.total_show_subtotal);
            self.paint_choice_pair(cx, ids!(bill_total_subtotal_off), ids!(bill_total_subtotal_off_on), !draft.total_show_subtotal);
            self.paint_choice_pair(cx, ids!(bill_total_grand_on), ids!(bill_total_grand_on_on), draft.total_show_grand);
            self.paint_choice_pair(cx, ids!(bill_total_grand_off), ids!(bill_total_grand_off_on), !draft.total_show_grand);
            self.ui
                .widget(cx, ids!(h_bill_total_qty_label))
                .set_text(cx, &plain_display_text(&s("bill_total_qty_label")));
            self.ui
                .widget(cx, ids!(h_bill_total_subtotal_label))
                .set_text(cx, &plain_display_text(&s("bill_total_subtotal_label")));
            self.ui
                .widget(cx, ids!(h_bill_total_grand_label))
                .set_text(cx, &plain_display_text(&s("bill_total_grand_label")));
            self.ui
                .widget(cx, ids!(h_bill_total_paid_label))
                .set_text(cx, &plain_display_text(&s("bill_total_paid_label")));
            for (id, val) in [
                (ids!(bill_total_qty_label_input), draft.total_qty_label.as_str()),
                (ids!(bill_total_subtotal_label_input), draft.total_subtotal_label.as_str()),
                (ids!(bill_total_grand_label_input), draft.total_grand_label.as_str()),
                (ids!(bill_total_paid_label_input), draft.total_paid_label.as_str()),
            ] {
                let input = self.ui.text_input(cx, id);
                if !cx.has_key_focus(input.area()) {
                    input.set_text(cx, val);
                }
            }
        }
        let pairable = field.is_pairable();
        self.ui.view(cx, ids!(bill_meta_pair)).set_visible(cx, pairable);
        if pairable {
            self.ui
                .widget(cx, ids!(h_bill_meta_pair))
                .set_text(cx, &plain_display_text(&s("bill_meta_pair")));
            let partner = draft.meta_pair_of(field);
            for (idle, on, label, other) in [
                (ids!(bill_pair_alone), ids!(bill_pair_alone_on), s("bill_pair_alone"), None),
                (ids!(bill_pair_customer), ids!(bill_pair_customer_on), print_share::BillStyleField::Customer.label().to_string(), Some(print_share::BillStyleField::Customer)),
                (ids!(bill_pair_billno), ids!(bill_pair_billno_on), print_share::BillStyleField::BillNo.label().to_string(), Some(print_share::BillStyleField::BillNo)),
                (ids!(bill_pair_time), ids!(bill_pair_time_on), print_share::BillStyleField::Time.label().to_string(), Some(print_share::BillStyleField::Time)),
                (ids!(bill_pair_cashier), ids!(bill_pair_cashier_on), print_share::BillStyleField::Cashier.label().to_string(), Some(print_share::BillStyleField::Cashier)),
                (ids!(bill_pair_table), ids!(bill_pair_table_on), print_share::BillStyleField::Table.label().to_string(), Some(print_share::BillStyleField::Table)),
            ] {
                self.ui.button(cx, idle).set_text(cx, &label);
                self.ui.button(cx, on).set_text(cx, &label);
                if other == Some(field) {
                    self.ui.button(cx, idle).set_visible(cx, false);
                    self.ui.button(cx, on).set_visible(cx, false);
                } else {
                    let selected = match other {
                        None => partner.is_none(),
                        Some(o) => partner == Some(o),
                    };
                    self.paint_choice_pair(cx, idle, on, selected);
                }
            }
        }
        self.paint_choice_pair(
            cx,
            ids!(bill_w_light),
            ids!(bill_w_light_on),
            style.weight == print_share::KotFontWeight::Light,
        );
        self.paint_choice_pair(
            cx,
            ids!(bill_w_reg),
            ids!(bill_w_reg_on),
            style.weight == print_share::KotFontWeight::Regular,
        );
        self.paint_choice_pair(
            cx,
            ids!(bill_w_bold),
            ids!(bill_w_bold_on),
            style.weight == print_share::KotFontWeight::Bold,
        );
        self.paint_choice_pair(
            cx,
            ids!(bill_a_start),
            ids!(bill_a_start_on),
            style.align == print_share::KotAlign::Start,
        );
        self.paint_choice_pair(
            cx,
            ids!(bill_a_center),
            ids!(bill_a_center_on),
            style.align == print_share::KotAlign::Center,
        );
        self.paint_choice_pair(
            cx,
            ids!(bill_a_end),
            ids!(bill_a_end_on),
            style.align == print_share::KotAlign::End,
        );
        let size = self.ui.text_input(cx, ids!(bill_field_size_input));
        if !cx.has_key_focus(size.area()) {
            size.set_text(cx, &style.size_em.to_string());
        }
        let value = self.ui.text_input(cx, ids!(bill_field_value));
        value.set_visible(cx, field.has_text());
        value.set_empty_text(cx, field.value_hint().to_string());
        if field.has_text() && !cx.has_key_focus(value.area()) {
            value.set_text(cx, &draft.text(field));
        }
        let preview = draft.render_sample(&shop, &when);
        let (mm, landscape, base) = {
            let st = app_state().lock().unwrap();
            let mm = st.bill_slip_draft.shown_content_width_mm();
            let base = st
                .printer_draft
                .format_for(print_share::RouteKey::Bill);
            (mm, base.landscape, base)
        };
        let metrics = print_share::metrics_at_content_mm(&base, mm);
        if let Ok((png, pw, ph)) = print_share::slip_page_png(&preview, &metrics, landscape) {
            let type_px = print_share::preview_type_px(style.size_em as f32, metrics.page_dots());
            if let Some(mut slip) = self
                .ui
                .widget(cx, ids!(bill_format_preview))
                .borrow_mut::<KotSlipPreview>()
            {
                slip.set_page(cx, &png, pw, ph, type_px);
            }
        }
        self.ui.view(cx, ids!(bill_format_preview_card)).redraw(cx);
    }

    /// Write the value box into the draft for the line that is open.
    fn commit_bill_field_value(&mut self, cx: &mut Cx) {
        let field = app_state().lock().unwrap().bill_style_field;
        if !field.has_text() {
            return;
        }
        let text = self.ui.text_input(cx, ids!(bill_field_value)).text();
        app_state()
            .lock()
            .unwrap()
            .bill_slip_draft
            .set_text(field, &text);
    }

    /// Address and font size currently in the boxes, onto the open bill line.
    fn read_bill_editor_into_draft(&mut self, cx: &mut Cx) {
        self.commit_bill_field_value(cx);
        let size_now = self.ui.text_input(cx, ids!(bill_field_size_input)).text();
        if let Ok(raw) = size_now.trim().parse::<u32>() {
            let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
            let mut st = app_state().lock().unwrap();
            let field = st.bill_style_field;
            st.bill_slip_draft.style_mut(field).size_em = em;
        }
    }

    /// Push the saved bill and kitchen formats into the live printer config
    /// so the next slip uses them without waiting for a later books refresh.
    fn publish_print_formats(&self) {
        let st = app_state().lock().unwrap();
        if !st.device_role.is_host() {
            return;
        }
        let bill_mm = st.bill_slip_draft.shown_content_width_mm();
        let kot_mm = st.kot_slip_draft.shown_content_width_mm();
        let em = st.bill_slip_draft.style(print_share::BillStyleField::Items).size_em as f32;
        let kot = st.kot_slip_draft.clone();
        let bill = st.bill_slip_draft.clone();
        drop(st);
        let mut live = print_share::printers();
        live.set_standard_em(em);
        live.set_bill_content_line(print_share::wrap_cols_for_paper_mm(bill_mm));
        live.set_kot_content_line(print_share::wrap_cols_for_paper_mm(kot_mm));
        if let Some(f) = live
            .formats
            .iter_mut()
            .find(|f| f.id == print_share::FORMAT_KOT)
        {
            kot.apply_to_print_format(f);
        }
        for f in live.formats.iter_mut() {
            if print_share::is_standard_document(&f.id) {
                bill.apply_to_print_format(f);
            }
        }
        print_share::configure(live);
    }

    /// Print the on-screen bill preview. Each line keeps its size, weight, and align.
    fn print_bill_format_preview(&mut self, cx: &mut Cx) {
        self.read_bill_editor_into_draft(cx);
        let result = (|| -> Result<String, String> {
            let st = app_state().lock().unwrap();
            if st.device_role == DeviceRole::Client {
                return Err(st.t("printers_host_only"));
            }
            let shop = st
                .store
                .as_ref()
                .and_then(|s| s.venue_title().ok())
                .unwrap_or_else(|| st.display_venue_title());
            let when = st
                .store
                .as_ref()
                .and_then(|s| s.fiscal_settings().ok())
                .map(|f| f.format_now())
                .unwrap_or_default();
            let out = st.bill_slip_draft.render_sample(&shop, &when);
            let copies = st.bill_slip_draft.copies.clamp(1, 3);
            let em = st.bill_slip_draft.style(print_share::BillStyleField::Items).size_em as f32;
            let cols = print_share::wrap_cols_for_paper_mm(
                st.bill_slip_draft.shown_content_width_mm(),
            );
            let mut live = st.printer_draft.clone();
            // The bill route already names the delivery. Test bill uses that
            // same format; it does not switch the slip onto a page printer.
            live.set_standard_em(em);
            live.set_bill_content_line(cols);
            let dir = st
                .store
                .as_ref()
                .and_then(|s| s.db_path().parent().map(|p| p.to_path_buf()));
            drop(st);
            if live.destinations_for(print_share::RouteKey::Bill).is_empty() {
                return Err(app_t("test_print_no_printer"));
            }
            let share = print_share::print_route_using(
                &live,
                print_share::PrintKind::Bill,
                print_share::RouteKey::Bill,
                &out,
                dir.as_deref(),
                copies,
            );
            match share {
                Ok(s) if s.opened => Ok(app_t("test_bill_sent")),
                Ok(_) => Err(print_share::take_problems()
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| app_t("test_print_no_printer"))),
                Err(e) => Err(e),
            }
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
    }

    /// Settings → Bill & KOT: Standard document type size (portrait + landscape).
    fn sync_std_print_ui(&mut self, cx: &mut Cx) {
        let (em, lang) = {
            let st = app_state().lock().unwrap();
            (st.printer_draft.standard_em(), st.locale.language.clone())
        };
        let s = |k: &str| locale::t(&lang, k);
        self.ui
            .widget(cx, ids!(h_std_print))
            .set_text(cx, &plain_display_text(&s("std_print")));
        self.ui
            .widget(cx, ids!(std_print_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("std_print_help")));
        self.ui
            .widget(cx, ids!(h_std_size))
            .set_text(cx, &plain_display_text(&s("std_size")));
        self.ui
            .button(cx, ids!(std_print_save))
            .set_text(cx, &s("save_std_print"));
        self.ui
            .text_input(cx, ids!(std_size_input))
            .set_empty_text(cx, s("std_size_ph"));
        let field = self.ui.text_input(cx, ids!(std_size_input));
        if !cx.has_key_focus(field.area()) {
            field.set_text(cx, &format!("{}", em.round() as i64));
        }
    }

    /// Settings → Bill & KOT: load kitchen-slip draft + preview.
    fn fill_kot_format_fields(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            let fmt = st
                .store
                .as_ref()
                .and_then(|s| s.kot_slip_format().ok())
                .unwrap_or_default();
            st.kot_slip_draft = fmt;
        }
        self.sync_kot_format_ui(cx);
    }

    fn sync_kot_format_ui(&mut self, cx: &mut Cx) {
        let size_in = self.ui.text_input(cx, ids!(kot_field_size_input));
        if cx.has_key_focus(size_in.area()) {
            if let Ok(raw) = size_in.text().trim().parse::<u32>() {
                let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
                let mut st = app_state().lock().unwrap();
                let kind = st.kot_edit_kind;
                let field = st.kot_style_field;
                st.kot_slip_draft
                    .kind_styles
                    .get_mut(kind)
                    .get_mut(field)
                    .size_em = em;
                if field == print_share::KotStyleField::ItemQty {
                    st.kot_slip_draft.em = em;
                }
            }
        }
        let (draft, shop, lang, edit_kind, style_field, order_idx) = {
            let st = app_state().lock().unwrap();
            let shop = st
                .store
                .as_ref()
                .and_then(|s| s.venue_title().ok())
                .unwrap_or_else(|| st.display_venue_title());
            (
                st.kot_slip_draft.clone(),
                shop,
                st.locale.language.clone(),
                st.kot_edit_kind,
                st.kot_style_field,
                st.kot_field_edit_idx,
            )
        };
        let s = |k: &str| locale::t(&lang, k);
        let field_style = draft.field_style(edit_kind, style_field);
        let shown = draft.field_shown(style_field);
        // Legacy 58/80-only wrap chips — content width lives above the tabs.
        self.ui.widget(cx, ids!(h_kot_width)).set_visible(cx, false);
        self.ui.button(cx, ids!(kot_width_58)).set_visible(cx, false);
        self.ui.button(cx, ids!(kot_width_58_on)).set_visible(cx, false);
        self.ui.button(cx, ids!(kot_width_80)).set_visible(cx, false);
        self.ui.button(cx, ids!(kot_width_80_on)).set_visible(cx, false);
        self.ui
            .widget(cx, ids!(h_kot_kind))
            .set_text(cx, &plain_display_text(&s("kot_kind_heading")));
        self.ui.widget(cx, ids!(h_kot_format_fields)).set_text(
            cx,
            &plain_display_text(&format!(
                "{} · {}",
                s("kot_format_fields"),
                s(edit_kind.locale_key())
            )),
        );
        self.ui
            .button(cx, ids!(kot_style_field_cycle))
            .set_visible(cx, false);
        self.ui
            .widget(cx, ids!(kot_style_field_label))
            .set_visible(cx, false);
        self.ui
            .widget(cx, ids!(h_kot_field_size))
            .set_text(cx, &plain_display_text(&s("kot_field_size")));
        self.ui
            .widget(cx, ids!(h_kot_copies))
            .set_text(cx, &plain_display_text(&s("kot_copies")));
        self.ui
            .widget(cx, ids!(h_kot_preview))
            .set_text(cx, &plain_display_text(&s("kot_preview")));
        self.ui
            .button(cx, ids!(kot_format_save))
            .set_text(cx, &s("save_kot_format"));
        let (test_n, test_open) = {
            let st = app_state().lock().unwrap();
            (
                st.test_printer_ids.len(),
                st.suggest_open == Some(SuggestField::KotTestPrinters),
            )
        };
        let test_dd = if test_n == 0 {
            s("kot_test_printers_dd")
        } else {
            s("kot_test_printers_dd_n").replace("{n}", &test_n.to_string())
        };
        let test_face = if test_open {
            s("pick_done_dd").replace("{label}", &test_dd)
        } else {
            test_dd
        };
        self.ui
            .button(cx, ids!(kot_test_printers_dd))
            .set_text(cx, &test_face);
        self.ui
            .button(cx, ids!(kot_format_test))
            .set_text(cx, &s("test_kot_format"));
        self.ui
            .text_input(cx, ids!(kot_title_input))
            .set_empty_text(cx, s("kot_title_ph"));
        self.ui
            .text_input(cx, ids!(kot_footer_input))
            .set_empty_text(cx, s("kot_footer_ph"));
        self.ui
            .text_input(cx, ids!(kot_field_size_input))
            .set_empty_text(cx, s("kot_field_size_ph"));
        let title_field = self.ui.text_input(cx, ids!(kot_title_input));
        if !cx.has_key_focus(title_field.area()) {
            title_field.set_text(cx, &draft.title);
        }
        let footer_field = self.ui.text_input(cx, ids!(kot_footer_input));
        if !cx.has_key_focus(footer_field.area()) {
            footer_field.set_text(cx, &draft.footer);
        }
        let size_field = self.ui.text_input(cx, ids!(kot_field_size_input));
        if !cx.has_key_focus(size_field.area()) {
            size_field.set_text(cx, &field_style.size_em.to_string());
        }
        self.ui
            .widget(cx, ids!(kot_fields_summary))
            .set_visible(cx, false);
        self.paint_kot_field_buttons(cx, &draft, edit_kind, style_field, &s);
        for (idle, active, label) in [
            (ids!(kot_kind_original), ids!(kot_kind_original_on), "kot_kind_original"),
            (ids!(kot_kind_duplicate), ids!(kot_kind_duplicate_on), "kot_kind_duplicate"),
            (ids!(kot_kind_modified), ids!(kot_kind_modified_on), "kot_kind_modified"),
            (ids!(kot_kind_cancelled), ids!(kot_kind_cancelled_on), "kot_kind_cancelled"),
            (ids!(kot_field_show), ids!(kot_field_show_on), "kot_field_show"),
            (ids!(kot_field_hide), ids!(kot_field_hide_on), "kot_field_hide"),
            (ids!(kot_field_weight_light), ids!(kot_field_weight_light_on), "kot_weight_light"),
            (ids!(kot_field_weight_reg), ids!(kot_field_weight_reg_on), "kot_weight_regular"),
            (ids!(kot_field_weight_bold), ids!(kot_field_weight_bold_on), "kot_weight_bold"),
            (ids!(kot_copies_1), ids!(kot_copies_1_on), "kot_copies_1"),
            (ids!(kot_copies_2), ids!(kot_copies_2_on), "kot_copies_2"),
            (ids!(kot_copies_3), ids!(kot_copies_3_on), "kot_copies_3"),
        ] {
            let t = s(label);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, active).set_text(cx, &t);
        }
        self.paint_choice_pair(
            cx,
            ids!(kot_kind_original),
            ids!(kot_kind_original_on),
            edit_kind == print_share::KotSlipKind::Original,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_kind_duplicate),
            ids!(kot_kind_duplicate_on),
            edit_kind == print_share::KotSlipKind::Duplicate,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_kind_modified),
            ids!(kot_kind_modified_on),
            edit_kind == print_share::KotSlipKind::Modified,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_kind_cancelled),
            ids!(kot_kind_cancelled_on),
            edit_kind == print_share::KotSlipKind::Cancelled,
        );
        let kind_chrome = draft.kind_styles.get(edit_kind);
        self.ui
            .widget(cx, ids!(h_kot_banner))
            .set_text(cx, &plain_display_text(&s("kot_banner")));
        self.ui
            .widget(cx, ids!(h_kot_item_prefix))
            .set_text(cx, &plain_display_text(&s("kot_item_prefix")));
        self.ui
            .widget(cx, ids!(h_kot_strike))
            .set_text(cx, &plain_display_text(&s("kot_strike")));
        self.ui
            .widget(cx, ids!(h_kot_strike_note))
            .set_text(cx, &plain_display_text(&s("kot_strike_note")));
        self.ui
            .text_input(cx, ids!(kot_banner_input))
            .set_empty_text(cx, s("kot_banner_ph"));
        self.ui
            .text_input(cx, ids!(kot_item_prefix_input))
            .set_empty_text(cx, s("kot_item_prefix_ph"));
        let banner_field = self.ui.text_input(cx, ids!(kot_banner_input));
        if !cx.has_key_focus(banner_field.area()) {
            banner_field.set_text(cx, &kind_chrome.banner);
        }
        let prefix_field = self.ui.text_input(cx, ids!(kot_item_prefix_input));
        if !cx.has_key_focus(prefix_field.area()) {
            prefix_field.set_text(cx, &kind_chrome.item_prefix);
        }
        for (idle, active, label) in [
            (ids!(kot_strike_on), ids!(kot_strike_on_on), "kot_strike_on"),
            (ids!(kot_strike_off), ids!(kot_strike_off_on), "kot_strike_off"),
            (
                ids!(kot_strike_note_on),
                ids!(kot_strike_note_on_on),
                "kot_strike_on",
            ),
            (
                ids!(kot_strike_note_off),
                ids!(kot_strike_note_off_on),
                "kot_strike_off",
            ),
        ] {
            let t = s(label);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, active).set_text(cx, &t);
        }
        self.paint_choice_pair(
            cx,
            ids!(kot_strike_on),
            ids!(kot_strike_on_on),
            kind_chrome.strike_item_name,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_strike_off),
            ids!(kot_strike_off_on),
            !kind_chrome.strike_item_name,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_strike_note_on),
            ids!(kot_strike_note_on_on),
            kind_chrome.strike_item_note,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_strike_note_off),
            ids!(kot_strike_note_off_on),
            !kind_chrome.strike_item_note,
        );
        self.paint_choice_pair(cx, ids!(kot_field_show), ids!(kot_field_show_on), shown);
        self.paint_choice_pair(cx, ids!(kot_field_hide), ids!(kot_field_hide_on), !shown);
        self.paint_choice_pair(
            cx,
            ids!(kot_field_weight_light),
            ids!(kot_field_weight_light_on),
            field_style.weight == print_share::KotFontWeight::Light,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_field_weight_reg),
            ids!(kot_field_weight_reg_on),
            field_style.weight == print_share::KotFontWeight::Regular,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_field_weight_bold),
            ids!(kot_field_weight_bold_on),
            field_style.weight == print_share::KotFontWeight::Bold,
        );
        self.paint_kot_align(cx, field_style.align, &s);
        let qty_open = style_field == print_share::KotStyleField::ItemQty;
        self.ui
            .view(cx, ids!(kot_item_col_metrics))
            .set_visible(cx, qty_open);
        if qty_open {
            self.ui
                .widget(cx, ids!(h_kot_item_col_gap))
                .set_text(cx, &plain_display_text(&s("bill_item_col_gap")));
            self.ui
                .widget(cx, ids!(h_kot_item_qty_width))
                .set_text(cx, &plain_display_text(&s("kot_item_qty_width")));
            self.ui
                .widget(cx, ids!(h_kot_item_num_align))
                .set_text(cx, &plain_display_text(&s("kot_item_num_align")));
            self.ui
                .widget(cx, ids!(h_kot_item_hdr_item))
                .set_text(cx, &plain_display_text(&s("bill_item_hdr_item")));
            self.ui
                .widget(cx, ids!(h_kot_item_hdr_qty))
                .set_text(cx, &plain_display_text(&s("bill_item_hdr_qty")));
            self.ui
                .widget(cx, ids!(h_kot_item_gap_em))
                .set_text(cx, &plain_display_text(&s("bill_item_gap_em")));
            self.ui
                .widget(cx, ids!(h_kot_item_qty_min))
                .set_text(cx, &plain_display_text(&s("bill_item_qty_min")));
            for (idle, on, key) in [
                (ids!(kot_item_gap_tight), ids!(kot_item_gap_tight_on), "bill_item_gap_tight"),
                (ids!(kot_item_gap_comfort), ids!(kot_item_gap_comfort_on), "bill_item_gap_comfort"),
                (ids!(kot_item_gap_wide), ids!(kot_item_gap_wide_on), "bill_item_gap_wide"),
                (ids!(kot_item_num_compact), ids!(kot_item_num_compact_on), "bill_item_num_compact"),
                (ids!(kot_item_num_comfort), ids!(kot_item_num_comfort_on), "bill_item_num_comfort"),
                (ids!(kot_item_num_wide), ids!(kot_item_num_wide_on), "bill_item_num_wide"),
                (ids!(kot_item_num_start), ids!(kot_item_num_start_on), "kot_align_start"),
                (ids!(kot_item_num_center), ids!(kot_item_num_center_on), "kot_align_center"),
                (ids!(kot_item_num_end), ids!(kot_item_num_end_on), "kot_align_end"),
            ] {
                let t = s(key);
                self.ui.button(cx, idle).set_text(cx, &t);
                self.ui.button(cx, on).set_text(cx, &t);
            }
            let gap_preset = draft.item_col_gap_preset();
            self.paint_choice_pair(
                cx,
                ids!(kot_item_gap_tight),
                ids!(kot_item_gap_tight_on),
                (gap_preset - print_share::KOT_COL_GAP_TIGHT).abs() < 0.01,
            );
            self.paint_choice_pair(
                cx,
                ids!(kot_item_gap_comfort),
                ids!(kot_item_gap_comfort_on),
                (gap_preset - print_share::KOT_COL_GAP_COMFORT).abs() < 0.01,
            );
            self.paint_choice_pair(
                cx,
                ids!(kot_item_gap_wide),
                ids!(kot_item_gap_wide_on),
                (gap_preset - print_share::KOT_COL_GAP_WIDE).abs() < 0.01,
            );
            let qty_preset = draft.item_qty_min_preset();
            self.paint_choice_pair(
                cx,
                ids!(kot_item_num_compact),
                ids!(kot_item_num_compact_on),
                (qty_preset - print_share::KOT_QTY_MIN_COMPACT).abs() < 0.01,
            );
            self.paint_choice_pair(
                cx,
                ids!(kot_item_num_comfort),
                ids!(kot_item_num_comfort_on),
                (qty_preset - print_share::KOT_QTY_MIN_COMFORT).abs() < 0.01,
            );
            self.paint_choice_pair(
                cx,
                ids!(kot_item_num_wide),
                ids!(kot_item_num_wide_on),
                (qty_preset - print_share::KOT_QTY_MIN_WIDE).abs() < 0.01,
            );
            self.paint_choice_pair(
                cx,
                ids!(kot_item_num_start),
                ids!(kot_item_num_start_on),
                draft.item_num_align == print_share::KotAlign::Start,
            );
            self.paint_choice_pair(
                cx,
                ids!(kot_item_num_center),
                ids!(kot_item_num_center_on),
                draft.item_num_align == print_share::KotAlign::Center,
            );
            self.paint_choice_pair(
                cx,
                ids!(kot_item_num_end),
                ids!(kot_item_num_end_on),
                draft.item_num_align == print_share::KotAlign::End,
            );
            let fmt_em = |val: f32| -> String {
                if (val - val.round()).abs() < 0.05 {
                    format!("{}", val.round() as i32)
                } else {
                    format!("{val:.2}")
                }
            };
            for (id, val) in [
                (ids!(kot_item_hdr_item_input), draft.item_header_item.as_str()),
                (ids!(kot_item_hdr_qty_input), draft.item_header_qty.as_str()),
            ] {
                let input = self.ui.text_input(cx, id);
                if !cx.has_key_focus(input.area()) {
                    input.set_text(cx, val);
                }
            }
            for (id, val) in [
                (ids!(kot_item_gap_em_input), draft.item_col_gap_em),
                (ids!(kot_item_qty_min_input), draft.item_qty_min_em),
            ] {
                let input = self.ui.text_input(cx, id);
                if !cx.has_key_focus(input.area()) {
                    input.set_text(cx, &fmt_em(val));
                }
            }
        }
        let copies = draft.copies.clamp(
            print_share::KOT_COPIES_MIN,
            print_share::KOT_COPIES_MAX,
        );
        self.paint_choice_pair(cx, ids!(kot_copies_1), ids!(kot_copies_1_on), copies == 1);
        self.paint_choice_pair(cx, ids!(kot_copies_2), ids!(kot_copies_2_on), copies == 2);
        self.paint_choice_pair(cx, ids!(kot_copies_3), ids!(kot_copies_3_on), copies == 3);
        {
            let order = kot_order_ids(&draft);
            self.ui
                .widget(cx, ids!(kot_field_order_label))
                .set_visible(cx, false);
            self.paint_kot_order_buttons(cx, &order, order_idx, &s);
            self.ui
                .widget(cx, ids!(h_kot_field_order))
                .set_text(cx, &plain_display_text(&s("kot_field_order")));
            self.ui.button(cx, ids!(kot_field_up)).set_text(cx, &s("kot_field_up"));
            self.ui.button(cx, ids!(kot_field_down)).set_text(cx, &s("kot_field_down"));
        }
        let preview = draft.render_sample_kind(&shop, edit_kind);
        let (mm, landscape, base) = {
            let st = app_state().lock().unwrap();
            let mm = st.kot_slip_draft.shown_content_width_mm();
            let base = st
                .printer_draft
                .format_for(print_share::RouteKey::Kot { station_id: "" });
            (mm, base.landscape, base)
        };
        let metrics = print_share::metrics_at_content_mm(&base, mm);
        if let Ok((png, pw, ph)) = print_share::slip_page_png(&preview, &metrics, landscape) {
            let type_px =
                print_share::preview_type_px(field_style.size_em as f32, metrics.page_dots());
            if let Some(mut slip) = self
                .ui
                .widget(cx, ids!(kot_format_preview))
                .borrow_mut::<KotSlipPreview>()
            {
                slip.set_page(cx, &png, pw, ph, type_px);
            }
        }
        self.ui
            .view(cx, ids!(kot_format_preview_card))
            .redraw(cx);
    }

    fn paint_kot_field_buttons(
        &mut self,
        cx: &mut Cx,
        draft: &print_share::KotSlipFormat,
        edit_kind: print_share::KotSlipKind,
        style_field: print_share::KotStyleField,
        s: &impl Fn(&str) -> String,
    ) {
        for (idle, on, field) in [
            (ids!(kot_pick_title), ids!(kot_pick_title_on), print_share::KotStyleField::Title),
            (ids!(kot_pick_shop), ids!(kot_pick_shop_on), print_share::KotStyleField::Shop),
            (ids!(kot_pick_time), ids!(kot_pick_time_on), print_share::KotStyleField::Time),
            (ids!(kot_pick_waiter), ids!(kot_pick_waiter_on), print_share::KotStyleField::Waiter),
            (ids!(kot_pick_station), ids!(kot_pick_station_on), print_share::KotStyleField::Station),
            (ids!(kot_pick_table), ids!(kot_pick_table_on), print_share::KotStyleField::Table),
            (ids!(kot_pick_no), ids!(kot_pick_no_on), print_share::KotStyleField::KotNo),
            (ids!(kot_pick_area), ids!(kot_pick_area_on), print_share::KotStyleField::Area),
            (ids!(kot_pick_qty), ids!(kot_pick_qty_on), print_share::KotStyleField::ItemQty),
        ] {
            let st_style = draft.field_style(edit_kind, field);
            let on_off = if draft.field_shown(field) {
                s("kot_field_on")
            } else {
                s("kot_field_off")
            };
            let label = format!(
                "{} · {on_off} · {} · {} · {}",
                s(field.locale_key()),
                st_style.size_em,
                s(st_style.weight.locale_key()),
                s(st_style.align.locale_key())
            );
            self.ui.button(cx, idle).set_text(cx, &label);
            self.ui.button(cx, on).set_text(cx, &label);
            self.paint_choice_pair(cx, idle, on, field == style_field);
        }
    }

    fn paint_kot_align(
        &mut self,
        cx: &mut Cx,
        align: print_share::KotAlign,
        s: &impl Fn(&str) -> String,
    ) {
        self.ui
            .widget(cx, ids!(h_kot_field_align))
            .set_text(cx, &plain_display_text(&s("kot_align")));
        for (idle, on, value, key) in [
            (ids!(kot_align_start), ids!(kot_align_start_on), print_share::KotAlign::Start, "kot_align_start"),
            (ids!(kot_align_center), ids!(kot_align_center_on), print_share::KotAlign::Center, "kot_align_center"),
            (ids!(kot_align_end), ids!(kot_align_end_on), print_share::KotAlign::End, "kot_align_end"),
        ] {
            let text = s(key);
            self.ui.button(cx, idle).set_text(cx, &text);
            self.ui.button(cx, on).set_text(cx, &text);
            self.paint_choice_pair(cx, idle, on, align == value);
        }
    }

    fn paint_kot_order_buttons(
        &mut self,
        cx: &mut Cx,
        order: &[String],
        order_idx: usize,
        s: &impl Fn(&str) -> String,
    ) {
        for (i, idle, on) in [
            (0, ids!(kot_ord_0), ids!(kot_ord_0_on)),
            (1, ids!(kot_ord_1), ids!(kot_ord_1_on)),
            (2, ids!(kot_ord_2), ids!(kot_ord_2_on)),
            (3, ids!(kot_ord_3), ids!(kot_ord_3_on)),
            (4, ids!(kot_ord_4), ids!(kot_ord_4_on)),
            (5, ids!(kot_ord_5), ids!(kot_ord_5_on)),
            (6, ids!(kot_ord_6), ids!(kot_ord_6_on)),
        ] {
            if let Some(id) = order.get(i) {
                let label = kot_field_order_label(id, s);
                self.ui.button(cx, idle).set_text(cx, &label);
                self.ui.button(cx, on).set_text(cx, &label);
                self.paint_choice_pair(cx, idle, on, i == order_idx);
            } else {
                self.ui.button(cx, idle).set_visible(cx, false);
                self.ui.button(cx, on).set_visible(cx, false);
            }
        }
    }


    fn go_if_allowed(
        &mut self,
        cx: &mut Cx,
        screen: Screen,
        allowed: impl Fn(&StaffMember) -> bool,
    ) {
        let ok = {
            let st = app_state().lock().unwrap();
            st.session.as_ref().map(allowed).unwrap_or(false)
        };
        if ok {
            self.go(cx, screen);
        } else {
            self.set_status(cx, "Not allowed for your role");
        }
    }

    fn go_peer_if_allowed(
        &mut self,
        cx: &mut Cx,
        screen: Screen,
        allowed: impl Fn(&StaffMember) -> bool,
    ) {
        let ok = {
            let st = app_state().lock().unwrap();
            st.session.as_ref().map(allowed).unwrap_or(false)
        };
        if ok {
            self.go_peer(cx, screen);
        } else {
            self.set_status(cx, "Not allowed for your role");
        }
    }

    fn copy_text_to_clipboard(&mut self, cx: &mut Cx, text: &str) {
        // A bill's per-line print marks are for the printer, not a paste.
        let text = print_share::strip_kot_style_marks(text);
        // Makepad's clipboard op reaches Android's ClipboardManager, which
        // `arboard` cannot; the desktop call is the immediate-feedback path.
        cx.copy_to_clipboard(&text);
        let desktop_ok = print_share::copy_text(&text);
        println!("Kumkaj share:\n{text}");
        {
            let mut st = app_state().lock().unwrap();
            st.last_receipt = text;
            st.status = if desktop_ok || cfg!(target_os = "android") {
                "Copied to clipboard".into()
            } else {
                "Receipt in strip — use Print / share if clipboard unavailable".into()
            };
        }
        self.sync_chrome(cx);
    }
}


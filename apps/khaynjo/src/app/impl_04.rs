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
            let lane_body = match st.money_lane {
                MoneyLane::Declared => format!("Bill: {declared}"),
                MoneyLane::Owner => format!("Bill: {owner}"),
            };
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
        self.ui.button(cx, ids!(lock_staff_prev)).set_text(cx, &s("prev"));
        self.ui.button(cx, ids!(lock_staff_next)).set_text(cx, &s("next"));
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
            let step = FLOOR_TILE_W + FLOOR_TILE_GAP;
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
        // Sidebar mode only: user can hide the whole rail (zero width).
        // Phone bottom tabs are never affected.
        let show_rail = unlocked && shell_wide && !rail_collapsed;
        let show_bottom = unlocked && !shell_wide;
        self.ui.view(cx, ids!(nav_rail)).set_visible(cx, show_rail);
        self.ui.view(cx, ids!(rail_divider)).set_visible(cx, show_rail);
        self.ui.view(cx, ids!(bottom_nav)).set_visible(cx, show_bottom);
        self.ui
            .button(cx, ids!(rail_expand_btn))
            .set_visible(cx, unlocked && shell_wide && rail_collapsed);
        self.ui
            .button(cx, ids!(rail_collapse_btn))
            .set_visible(cx, show_rail);

        let rail_w = if show_rail {
            let preferred = app_state().lock().unwrap().rail_width;
            let max_rail = (window_w - MAIN_CONTENT_MIN - DIVIDER_W).max(RAIL_WIDTH_MIN);
            let clamped = preferred.clamp(RAIL_WIDTH_MIN, max_rail);
            {
                let mut st = app_state().lock().unwrap();
                st.rail_width = clamped;
            }
            set_view_walk(
                cx,
                &self.ui.view(cx, ids!(nav_rail)),
                Walk {
                    width: Size::Fixed(clamped),
                    height: Size::fill(),
                    ..Default::default()
                },
            );
            clamped
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
            // Never show "Khaynjo" when the owner saved venue or org.
            let title = st
                .store
                .as_ref()
                .and_then(|s| s.listing_name().ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    if st.venue_title.trim().is_empty() {
                        "Khaynjo".to_string()
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
        if screen == Screen::Settings {
            self.fill_bill_format_fields(cx);
            // Do not auto-scan printers here — Find is explicit. A /24 scan on
            // every Settings open made Add printer feel like the UI froze.
        }
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
            (ids!(settings_panel_alerts), "alerts"),
            (ids!(settings_panel_backup), "backup"),
            (ids!(settings_panel_diag), "diag"),
        ];
        for (id, key) in panels {
            self.ui
                .view(cx, id)
                .set_visible(cx, panel == Some(*key));
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
        if panel == Some("bill") {
            self.fill_bill_format_fields(cx);
        }
        self.ui.redraw(cx);
    }

    /// Settings → Diagnostics: status, level chips, support/share actions.
    fn sync_diag_panel(&mut self, cx: &mut Cx) {
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
        self.ui
            .button(cx, ids!(diag_support_start))
            .set_text(cx, &s("diag_support_start"));
        self.ui
            .button(cx, ids!(diag_support_stop))
            .set_text(cx, &s("diag_support_stop"));
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
        let level = diag::status()
            .map(|s| s.min_level)
            .unwrap_or(diag::Level::Info);
        // While support session forces Debug display, paint the configured chip
        // from status.min_level (already effective).
        let pairs = [
            (ids!(diag_level_error), ids!(diag_level_error_on), diag::Level::Error),
            (ids!(diag_level_warn), ids!(diag_level_warn_on), diag::Level::Warn),
            (ids!(diag_level_info), ids!(diag_level_info_on), diag::Level::Info),
            (ids!(diag_level_debug), ids!(diag_level_debug_on), diag::Level::Debug),
        ];
        for (off, on, lv) in pairs {
            let active = level == lv;
            self.ui.button(cx, off).set_visible(cx, !active);
            self.ui.button(cx, on).set_visible(cx, active);
        }
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
                diag::end_support_session();
                diag::set_min_level(level);
                self.sync_diag_panel(cx);
                self.show_ok(cx, format!("Diagnostics level: {}", level.as_str()));
                return;
            }
        }
        if self.ui.button(cx, ids!(diag_support_start)).clicked(actions) {
            diag::start_support_session(None);
            self.sync_diag_panel(cx);
            self.show_ok(cx, app_t("diag_support_started"));
            return;
        }
        if self.ui.button(cx, ids!(diag_support_stop)).clicked(actions) {
            diag::end_support_session();
            self.sync_diag_panel(cx);
            self.show_ok(cx, app_t("diag_support_ended"));
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
        if self.ui.button(cx, ids!(diag_approve_send)).clicked(actions) {
            diag::approve_send();
            self.sync_diag_panel(cx);
            self.show_warn(cx, app_t("diag_send_approved"));
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

    /// Settings → Bill format shows what the books hold.
    fn fill_bill_format_fields(&mut self, cx: &mut Cx) {
        let (header, fssai, footer, upi_vpa, upi_payee, stamp_hint, pay_qr_hint, scheme_rows) = {
            let st = app_state().lock().unwrap();
            let lang = st.locale.language.clone();
            let Some(store) = st.store.as_ref() else {
                return;
            };
            let scheme_rows: Vec<ReportRow> = {
                let schemes = store.list_schemes().unwrap_or_default();
                if schemes.is_empty() {
                    vec![ReportRow::Line("No schemes yet.".into())]
                } else {
                    schemes
                        .into_iter()
                        .map(|s| {
                            ReportRow::Kv(
                                s.name,
                                format!("{}{}", s.kind.as_str(), if s.active { "" } else { " · off" }),
                            )
                        })
                        .collect()
                }
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
                scheme_rows,
            )
        };
        self.ui.text_input(cx, ids!(bill_header_input)).set_text(cx, &header);
        self.ui.text_input(cx, ids!(fssai_input)).set_text(cx, &fssai);
        self.ui.text_input(cx, ids!(upi_vpa_input)).set_text(cx, &upi_vpa);
        self.ui.text_input(cx, ids!(upi_payee_input)).set_text(cx, &upi_payee);
        self.ui.text_input(cx, ids!(bill_footer_input)).set_text(cx, &footer);
        self.ui
            .widget(cx, ids!(print_stamp_status))
            .set_text(cx, &plain_display_text(&stamp_hint));
        self.ui
            .widget(cx, ids!(payment_qr_status))
            .set_text(cx, &plain_display_text(&pay_qr_hint));
        self.set_report_rows(cx, ids!(schemes_label), scheme_rows);
        self.sync_std_print_ui(cx);
        self.fill_kot_format_fields(cx);
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
        for (idle, active, label) in [
            (ids!(std_size_s), ids!(std_size_s_on), "std_size_s"),
            (ids!(std_size_m), ids!(std_size_m_on), "std_size_m"),
            (ids!(std_size_l), ids!(std_size_l_on), "std_size_l"),
        ] {
            let t = s(label);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, active).set_text(cx, &t);
        }
        self.paint_choice_pair(
            cx,
            ids!(std_size_s),
            ids!(std_size_s_on),
            em <= print_share::STD_EM_SMALL + 0.5,
        );
        self.paint_choice_pair(
            cx,
            ids!(std_size_m),
            ids!(std_size_m_on),
            em > print_share::STD_EM_SMALL + 0.5 && em < print_share::STD_EM_LARGE - 0.5,
        );
        self.paint_choice_pair(
            cx,
            ids!(std_size_l),
            ids!(std_size_l_on),
            em >= print_share::STD_EM_LARGE - 0.5,
        );
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
        let (draft, shop, lang) = {
            let st = app_state().lock().unwrap();
            let shop = st
                .store
                .as_ref()
                .and_then(|s| s.venue_title().ok())
                .unwrap_or_else(|| st.display_venue_title());
            (st.kot_slip_draft.clone(), shop, st.locale.language.clone())
        };
        let s = |k: &str| locale::t(&lang, k);
        self.ui
            .widget(cx, ids!(h_kot_format))
            .set_text(cx, &plain_display_text(&s("kot_format")));
        self.ui
            .widget(cx, ids!(kot_format_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("kot_format_help")));
        self.ui
            .widget(cx, ids!(h_kot_format_fields))
            .set_text(cx, &plain_display_text(&s("kot_format_fields")));
        self.ui
            .widget(cx, ids!(h_kot_width))
            .set_text(cx, &plain_display_text(&s("kot_width")));
        self.ui
            .widget(cx, ids!(h_kot_size))
            .set_text(cx, &plain_display_text(&s("kot_size")));
        self.ui
            .widget(cx, ids!(h_kot_copies))
            .set_text(cx, &plain_display_text(&s("kot_copies")));
        self.ui
            .widget(cx, ids!(h_kot_preview))
            .set_text(cx, &plain_display_text(&s("kot_preview")));
        self.ui
            .button(cx, ids!(kot_format_save))
            .set_text(cx, &s("save_kot_format"));
        self.ui
            .text_input(cx, ids!(kot_title_input))
            .set_empty_text(cx, s("kot_title_ph"));
        self.ui
            .text_input(cx, ids!(kot_footer_input))
            .set_empty_text(cx, s("kot_footer_ph"));
        let title_field = self.ui.text_input(cx, ids!(kot_title_input));
        if !cx.has_key_focus(title_field.area()) {
            title_field.set_text(cx, &draft.title);
        }
        let footer_field = self.ui.text_input(cx, ids!(kot_footer_input));
        if !cx.has_key_focus(footer_field.area()) {
            footer_field.set_text(cx, &draft.footer);
        }
        for (idle, active, label) in [
            (ids!(kot_shop_on), ids!(kot_shop_on_on), "kot_show_shop"),
            (ids!(kot_shop_off), ids!(kot_shop_off_on), "kot_hide_shop"),
            (ids!(kot_time_on), ids!(kot_time_on_on), "kot_show_time"),
            (ids!(kot_time_off), ids!(kot_time_off_on), "kot_hide_time"),
            (ids!(kot_waiter_on), ids!(kot_waiter_on_on), "kot_show_waiter"),
            (ids!(kot_waiter_off), ids!(kot_waiter_off_on), "kot_hide_waiter"),
            (ids!(kot_station_on), ids!(kot_station_on_on), "kot_show_station"),
            (ids!(kot_station_off), ids!(kot_station_off_on), "kot_hide_station"),
            (ids!(kot_table_on), ids!(kot_table_on_on), "kot_show_table"),
            (ids!(kot_table_off), ids!(kot_table_off_on), "kot_hide_table"),
            (ids!(kot_no_on), ids!(kot_no_on_on), "kot_show_kot_no"),
            (ids!(kot_no_off), ids!(kot_no_off_on), "kot_hide_kot_no"),
            (ids!(kot_width_58), ids!(kot_width_58_on), "kot_width_58"),
            (ids!(kot_width_80), ids!(kot_width_80_on), "kot_width_80"),
            (ids!(kot_size_s), ids!(kot_size_s_on), "kot_size_s"),
            (ids!(kot_size_m), ids!(kot_size_m_on), "kot_size_m"),
            (ids!(kot_size_l), ids!(kot_size_l_on), "kot_size_l"),
            (ids!(kot_copies_1), ids!(kot_copies_1_on), "kot_copies_1"),
            (ids!(kot_copies_2), ids!(kot_copies_2_on), "kot_copies_2"),
            (ids!(kot_copies_3), ids!(kot_copies_3_on), "kot_copies_3"),
        ] {
            let t = s(label);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, active).set_text(cx, &t);
        }
        self.paint_choice_pair(cx, ids!(kot_shop_on), ids!(kot_shop_on_on), draft.show_shop_name);
        self.paint_choice_pair(cx, ids!(kot_shop_off), ids!(kot_shop_off_on), !draft.show_shop_name);
        self.paint_choice_pair(cx, ids!(kot_title_on), ids!(kot_title_on_on), draft.show_title);
        self.paint_choice_pair(cx, ids!(kot_title_off), ids!(kot_title_off_on), !draft.show_title);
        self.paint_choice_pair(cx, ids!(kot_area_on), ids!(kot_area_on_on), draft.show_area);
        self.paint_choice_pair(cx, ids!(kot_area_off), ids!(kot_area_off_on), !draft.show_area);
        self.paint_choice_pair(
            cx,
            ids!(kot_item_hdr_on),
            ids!(kot_item_hdr_on_on),
            draft.show_item_qty_header,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_item_hdr_off),
            ids!(kot_item_hdr_off_on),
            !draft.show_item_qty_header,
        );
        self.paint_choice_pair(cx, ids!(kot_time_on), ids!(kot_time_on_on), draft.show_time);
        self.paint_choice_pair(cx, ids!(kot_time_off), ids!(kot_time_off_on), !draft.show_time);
        self.paint_choice_pair(cx, ids!(kot_waiter_on), ids!(kot_waiter_on_on), draft.show_waiter);
        self.paint_choice_pair(cx, ids!(kot_waiter_off), ids!(kot_waiter_off_on), !draft.show_waiter);
        self.paint_choice_pair(
            cx,
            ids!(kot_station_on),
            ids!(kot_station_on_on),
            draft.show_station,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_station_off),
            ids!(kot_station_off_on),
            !draft.show_station,
        );
        self.paint_choice_pair(cx, ids!(kot_table_on), ids!(kot_table_on_on), draft.show_table);
        self.paint_choice_pair(cx, ids!(kot_table_off), ids!(kot_table_off_on), !draft.show_table);
        self.paint_choice_pair(cx, ids!(kot_no_on), ids!(kot_no_on_on), draft.show_kot_no);
        self.paint_choice_pair(cx, ids!(kot_no_off), ids!(kot_no_off_on), !draft.show_kot_no);
        let wide = draft.chars_per_line >= print_share::KOT_CHARS_80MM;
        self.paint_choice_pair(cx, ids!(kot_width_58), ids!(kot_width_58_on), !wide);
        self.paint_choice_pair(cx, ids!(kot_width_80), ids!(kot_width_80_on), wide);
        let em = draft.em;
        self.paint_choice_pair(
            cx,
            ids!(kot_size_s),
            ids!(kot_size_s_on),
            em <= print_share::KOT_EM_SMALL,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_size_m),
            ids!(kot_size_m_on),
            em > print_share::KOT_EM_SMALL && em < print_share::KOT_EM_LARGE,
        );
        self.paint_choice_pair(
            cx,
            ids!(kot_size_l),
            ids!(kot_size_l_on),
            em >= print_share::KOT_EM_LARGE,
        );
        let copies = draft.copies.clamp(
            print_share::KOT_COPIES_MIN,
            print_share::KOT_COPIES_MAX,
        );
        self.paint_choice_pair(cx, ids!(kot_copies_1), ids!(kot_copies_1_on), copies == 1);
        self.paint_choice_pair(cx, ids!(kot_copies_2), ids!(kot_copies_2_on), copies == 2);
        self.paint_choice_pair(cx, ids!(kot_copies_3), ids!(kot_copies_3_on), copies == 3);
        {
            let mut order = draft.field_order.clone();
            if order.is_empty() {
                order = vec![
                    "time".into(),
                    "kot_no".into(),
                    "area".into(),
                    "table".into(),
                    "shop".into(),
                    "waiter".into(),
                    "station".into(),
                ];
            }
            let edit = {
                let st = app_state().lock().unwrap();
                st.kot_field_edit_idx.min(order.len().saturating_sub(1))
            };
            let lines: Vec<ReportRow> = order
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    ReportRow::Line(if i == edit {
                        format!("› {id}")
                    } else {
                        format!("  {id}")
                    })
                })
                .collect();
            self.set_report_rows(cx, ids!(kot_field_order_label), lines);
            self.ui
                .widget(cx, ids!(h_kot_field_order))
                .set_text(cx, &plain_display_text(&s("kot_field_order")));
            self.ui.button(cx, ids!(kot_field_up)).set_text(cx, &s("kot_field_up"));
            self.ui.button(cx, ids!(kot_field_down)).set_text(cx, &s("kot_field_down"));
            self.ui.button(cx, ids!(kot_field_cycle)).set_text(cx, &s("kot_field_cycle"));
        }
        let preview = draft.render_sample(&shop);
        let paper_w = draft.preview_paper_width_px();
        if let Some(mut card) = self.ui.view(cx, ids!(kot_format_preview_card)).borrow_mut() {
            card.walk.width = Size::Fixed(paper_w);
        }
        // ReportText keeps slip newlines (Label left Preview blank).
        self.ui
            .widget(cx, ids!(kot_format_preview))
            .set_text(cx, &preview.trim_end());
        self.ui
            .view(cx, ids!(kot_format_preview_card))
            .redraw(cx);
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
        // Makepad's clipboard op reaches Android's ClipboardManager, which
        // `arboard` cannot; the desktop call is the immediate-feedback path.
        cx.copy_to_clipboard(text);
        let desktop_ok = print_share::copy_text(text);
        println!("Kumkaj share:\n{text}");
        {
            let mut st = app_state().lock().unwrap();
            st.last_receipt = text.to_string();
            st.status = if desktop_ok || cfg!(target_os = "android") {
                "Copied to clipboard".into()
            } else {
                "Receipt in strip — use Print / share if clipboard unavailable".into()
            };
        }
        self.sync_chrome(cx);
    }
}

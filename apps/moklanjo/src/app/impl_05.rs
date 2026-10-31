impl App {
fn paint_chrome(&mut self, cx: &mut Cx) {
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |key: &str| locale::t(&lang, key);
        // Short chrome labels for bottom chips + rail (wrap, no ellipsis).
        // Long More-row strings stay on more_* buttons below.
        let books = s("books");
        let sell = s("sell");
        let buy = s("buy");
        let rates = s("items");
        let sales = s("sales");
        let day_close = s("day_close");
        let reports = s("nav_reports");
        let inventory = s("inventory");
        let branches = s("branches");
        let credit = s("credit");
        let team = s("team");
        let settings = s("nav_settings");
        let lock = s("nav_lock");
        let riders = s("riders");
        // Slots pack preference + rights (must match paint_nav_tabs).
        let order = current_nav_chrome_order();
        let label_for = |tab: NavTab| -> &str {
            match tab {
                NavTab::Books => &books,
                NavTab::Sell => &sell,
                NavTab::Buy => &buy,
                NavTab::Rates => &rates,
                NavTab::Ops => "Ops",
                NavTab::Cod => "COD",
                NavTab::Riders => &riders,
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
        self.ui
            .button(cx, ids!(more_items))
            .set_text(cx, &s("items"));
        self.ui
            .button(cx, ids!(more_sales))
            .set_text(cx, &s("sales_refunds"));
        self.ui
            .button(cx, ids!(more_day_close))
            .set_text(cx, &s("day_close"));
        self.ui
            .button(cx, ids!(more_reports))
            .set_text(cx, &s("reports_expenses"));
        self.ui
            .button(cx, ids!(more_inventory))
            .set_text(cx, &s("inventory_stock"));
        self.ui
            .button(cx, ids!(more_branches))
            .set_text(cx, &s("branches_org"));
        self.ui
            .button(cx, ids!(more_credit))
            .set_text(cx, &s("credit_customers"));
        self.ui
            .button(cx, ids!(more_team))
            .set_text(cx, &s("team_pins"));
        self.ui
            .button(cx, ids!(more_settings))
            .set_text(cx, &s("backup_settings"));
        self.ui
            .button(cx, ids!(more_lock))
            .set_text(cx, &s("lock_app"));
        self.ui
            .button(cx, ids!(books_to_sell))
            .set_text(cx, &s("sell_money_in"));
        self.ui
            .button(cx, ids!(books_to_buy))
            .set_text(cx, &s("buy_money_out"));
        self.ui
            .button(cx, ids!(books_day_close))
            .set_text(cx, &s("day_close"));
        self.ui
            .button(cx, ids!(books_reports))
            .set_text(cx, &s("reports_expenses"));
        self.ui
            .button(cx, ids!(books_credit))
            .set_text(cx, &s("credit_customers"));
        self.ui
            .button(cx, ids!(buy_purchase))
            .set_text(cx, &s("record_purchase"));
        self.ui
            .button(cx, ids!(buy_expense_add))
            .set_text(cx, &s("add_expense"));
        self.ui
            .button(cx, ids!(buy_to_inventory))
            .set_text(cx, &s("inventory_stock"));
        {
            let help = self.ui.widget(cx, ids!(buy_help)).as_info_help();
            help.set_title(cx, &s("buy"));
            help.set_nav_icon(cx, NavTab::Buy);
            help.set_tip(&plain_display_text(&s("buy_help")));
        }
        self.ui
            .button(cx, ids!(client_pair_back))
            .set_text(cx, &s("back"));
        self.ui
            .button(cx, ids!(clear_cart))
            .set_text(cx, &s("clear"));
        self.ui
            .button(cx, ids!(pay_mix))
            .set_text(cx, &s("take_payment"));
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
        self.ui
            .button(cx, ids!(menu_new))
            .set_text(cx, &s("new_item"));
        self.ui
            .button(cx, ids!(menu_add))
            .set_text(cx, &s(if editing { "save" } else { "add_item" }));
        self.ui.widget(cx, ids!(menu_mode_label)).set_text(
            cx,
            &plain_display_text(&match editing_name {
                Some(name) => format!("{}: {name}", s("editing_item")),
                None => s("adding_item"),
            }),
        );
        self.ui
            .button(cx, ids!(image_clear))
            .set_text(cx, &s("clear_photo"));
        self.ui
            .button(cx, ids!(image_pick))
            .set_text(cx, &s("photo"));
        self.ui
            .button(cx, ids!(party_image_clear))
            .set_text(cx, &s("clear_logo"));
        self.ui
            .button(cx, ids!(party_image_pick))
            .set_text(cx, &s("pick_logo"));
        for path in [ids!(booking_scope_domestic), ids!(booking_scope_domestic_on)] {
            self.ui.button(cx, path).set_text(cx, &s("domestic"));
        }
        for path in [ids!(booking_scope_intl), ids!(booking_scope_intl_on)] {
            self.ui.button(cx, path).set_text(cx, &s("international"));
        }
        self.ui
            .label(cx, ids!(h_booking_insurance))
            .set_text(cx, &s("insurance_label"));
        for path in [ids!(booking_insurance_no), ids!(booking_insurance_no_on)] {
            self.ui.button(cx, path).set_text(cx, &s("no_label"));
        }
        for path in [ids!(booking_insurance_yes), ids!(booking_insurance_yes_on)] {
            self.ui.button(cx, path).set_text(cx, &s("yes_label"));
        }
        self.ui
            .label(cx, ids!(h_buy_party_kind))
            .set_text(cx, &s("new_party_kind_label"));
        for path in [ids!(buy_kind_supplier), ids!(buy_kind_supplier_on)] {
            self.ui.button(cx, path).set_text(cx, &s("role_supplier"));
        }
        for path in [ids!(buy_kind_carrier), ids!(buy_kind_carrier_on)] {
            self.ui.button(cx, path).set_text(cx, &s("role_carrier"));
        }
        self.ui
            .label(cx, ids!(h_route_pick))
            .set_text(cx, &s("print_route_label"));
        for path in [ids!(route_pick_receipt), ids!(route_pick_receipt_on)] {
            self.ui.button(cx, path).set_text(cx, &s("route_receipt"));
        }
        for path in [ids!(route_pick_label), ids!(route_pick_label_on)] {
            self.ui.button(cx, path).set_text(cx, &s("route_label"));
        }
        for path in [ids!(role_owner), ids!(role_owner_on)] {
            self.ui.button(cx, path).set_text(cx, &s("owner"));
        }
        for path in [ids!(role_staff), ids!(role_staff_on)] {
            self.ui.button(cx, path).set_text(cx, &s("staff"));
        }
        self.ui
            .button(cx, ids!(lock_staff_prev))
            .set_text(cx, &s("prev"));
        self.ui
            .button(cx, ids!(lock_staff_next))
            .set_text(cx, &s("next"));
        self.ui.button(cx, ids!(pin_ok)).set_text(cx, &s("ok"));
        for path in [ids!(device_role_server), ids!(device_role_server_on)] {
            self.ui.button(cx, path).set_text(cx, &s("this_shop"));
        }
        for path in [ids!(device_role_client), ids!(device_role_client_on)] {
            self.ui.button(cx, path).set_text(cx, &s("extra_till"));
        }
        self.ui
            .button(cx, ids!(customer_clear))
            .set_text(cx, &s("clear"));
        self.ui
            .button(cx, ids!(undo_snack_btn))
            .set_text(cx, &s("undo"));
        self.ui
            .button(cx, ids!(inv_adjust_minus))
            .set_text(cx, &s("stock_minus"));
        self.ui
            .button(cx, ids!(inv_adjust_plus))
            .set_text(cx, &s("stock_plus"));
        self.ui
            .button(cx, ids!(inv_track))
            .set_text(cx, &s("start_tracking"));
        self.ui
            .button(cx, ids!(inv_purchase))
            .set_text(cx, &s("record_purchase"));
        self.ui
            .button(cx, ids!(settle_cash))
            .set_text(cx, &s("settle_cash"));
        self.ui
            .button(cx, ids!(settle_card))
            .set_text(cx, &s("settle_card"));
        self.ui
            .button(cx, ids!(amend_pay_cash))
            .set_text(cx, &s("cash"));
        self.ui
            .button(cx, ids!(amend_pay_card))
            .set_text(cx, &s("card"));
        {
            let help = self.ui.widget(cx, ids!(items_help_label)).as_info_help();
            help.set_title(cx, &s("items"));
            help.set_nav_icon(cx, NavTab::Rates);
            help.set_tip(&plain_display_text(&s("items_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(more_help)).as_info_help();
            help.set_title(cx, &s("more"));
            help.set_nav_icon(cx, NavTab::Rates);
            help.set_tip(&plain_display_text(&s("more_help_moklanjo")));
        }
        {
            let help = self.ui.widget(cx, ids!(books_help)).as_info_help();
            help.set_title(cx, &s("books"));
            help.set_nav_icon(cx, NavTab::Books);
            help.set_tip(&plain_display_text(&s("books_help")));
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
        self.ui
            .text_input(cx, ids!(credit_add_kind))
            .set_empty_text(cx, s("party_kind_ph"));
        {
            let help = self.ui.widget(cx, ids!(team_help)).as_info_help();
            help.set_title(cx, &s("team"));
            help.set_nav_icon(cx, NavTab::Team);
            help.set_tip(&plain_display_text(&s("team_owner_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(settings_help)).as_info_help();
            help.set_title(cx, &s("nav_settings"));
            help.set_nav_icon(cx, NavTab::Settings);
            help.set_tip(&plain_display_text(&s("language_setup_hint")));
        }
        self.ui
            .button(cx, ids!(sidebar_settings_open))
            .set_text(cx, &s("sidebar_settings"));
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
            .button(cx, ids!(expense_add))
            .set_text(cx, &s("add_expense"));
        self.ui
            .widget(cx, ids!(reports_expense_heading))
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
        {
            let help = self.ui.widget(cx, ids!(server_pick_help)).as_info_help();
            help.set_title(cx, &s("choose_server"));
            help.set_tip(&plain_display_text(&s("choose_server_hint")));
        }
        {
            let help = self.ui.widget(cx, ids!(lock_login_help)).as_info_help();
            help.set_title(cx, &s("or_unlock_login"));
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
        let picking_server = app_state().lock().unwrap().picking_server;
        self.ui
            .view(cx, ids!(server_pick_section))
            .set_visible(cx, picking_server);
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
            Screen::Pos => ids!(pos_catalog_scroll),
            Screen::Buy => ids!(buy_screen),
            Screen::Menu => ids!(menu_screen),
            Screen::More => ids!(more_screen),
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
            (ids!(menu_screen), screen == Screen::Menu),
            (ids!(more_screen), screen == Screen::More),
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
        self.ui.view(cx, ids!(header)).set_visible(cx, unlocked);
        let show_today = unlocked
            && session_may_see_today(app_state().lock().unwrap().session.as_ref());
        self.ui.view(cx, ids!(today_bar)).set_visible(cx, show_today);
        self.apply_shell_layout(cx, unlocked);
        self.paint_nav_tabs(cx, screen);
        if screen == Screen::Settings {
            self.sync_diag_panel(cx);
        }
    }

    /// Phone (<720): bottom tabs + stacked Booking panes.
    /// Tablet/desktop (≥720): side rail; Parcel | Counter side by side under
    /// sticky `booking_desk_top` (Fixed panes in ScrollX).
    fn apply_shell_layout(&mut self, cx: &mut Cx, unlocked: bool) {
        const DESIGN_DESKTOP_W: f64 = store::DEFAULT_DESKTOP_WINDOW_W;
        const DIVIDER_W: f64 = 6.0;
        const DESK_LEFT_MIN: f64 = 600.0;
        const DESK_RIGHT_MIN: f64 = 520.0;
        const DESK_GAP: f64 = 16.0;
        const POS_SIDE_BY_SIDE_MIN_CONTENT: f64 = 640.0;
        let window_w = {
            let w = self.window_inner_width(cx);
            if w >= 1.0 { w } else { DESIGN_DESKTOP_W }
        };
        let shell_wide = window_w >= store::TABLET_SHELL_BREAKPOINT_CSS;
        let rail_collapsed = app_state().lock().unwrap().rail_collapsed;
        // Sidebar mode only: user can hide the whole rail (zero width).
        // Phone bottom tabs are never affected.
        let show_rail = unlocked && shell_wide && !rail_collapsed;
        let show_bottom = unlocked && !shell_wide;
        self.ui.view(cx, ids!(nav_rail)).set_visible(cx, show_rail);
        self.ui
            .view(cx, ids!(rail_divider))
            .set_visible(cx, show_rail);
        self.ui
            .view(cx, ids!(bottom_nav))
            .set_visible(cx, show_bottom);
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

        let content_w = (window_w
            - if show_rail { rail_w + DIVIDER_W } else { 0.0 }
            - 32.0)
            .max(280.0);
        let side_by_side = content_w >= POS_SIDE_BY_SIDE_MIN_CONTENT;
        let desk_row = self.ui.view(cx, ids!(booking_desk_row));
        set_view_flow(
            cx,
            &desk_row,
            if side_by_side {
                Flow::right()
            } else {
                Flow::Down
            },
        );
        set_view_walk(
            cx,
            &desk_row,
            Walk {
                width: if side_by_side {
                    Size::fit()
                } else {
                    Size::fill()
                },
                height: Size::fill(),
                ..Default::default()
            },
        );
        let (left_w, right_w) = if side_by_side {
            let desk_avail = (content_w - 32.0).max(200.0);
            let min_total = DESK_LEFT_MIN + DESK_RIGHT_MIN + DESK_GAP;
            if desk_avail >= min_total {
                let left = desk_avail * (DESK_LEFT_MIN / (DESK_LEFT_MIN + DESK_RIGHT_MIN));
                (Size::Fixed(left), Size::Fixed(desk_avail - DESK_GAP - left))
            } else {
                (Size::Fixed(DESK_LEFT_MIN), Size::Fixed(DESK_RIGHT_MIN))
            }
        } else {
            (Size::fill(), Size::fill())
        };
        set_view_walk(
            cx,
            &self.ui.view(cx, ids!(pos_catalog_scroll)),
            Walk {
                width: left_w,
                height: Size::fill(),
                ..Default::default()
            },
        );
        set_view_walk(
            cx,
            &self.ui.view(cx, ids!(pos_cart_scroll)),
            Walk {
                width: right_w,
                height: Size::fill(),
                ..Default::default()
            },
        );
        self.ui.view(cx, ids!(pos_catalog_scroll)).set_visible(cx, true);
        self.ui.view(cx, ids!(pos_cart_scroll)).set_visible(cx, true);
    }

    /// Nav slot hardware in DSL order. Slots are positions; which tab a
    /// slot shows comes from `AppState.nav_order` (drag to swap).
    fn bottom_nav_pairs(&self) -> [(&[LiveId], &[LiveId]); NAV_TABS.len()] {
        [
            (ids!(nav_books), ids!(nav_books_on)),
            (ids!(nav_sell), ids!(nav_sell_on)),
            (ids!(nav_buy), ids!(nav_buy_on)),
            (ids!(nav_rates), ids!(nav_rates_on)),
            (ids!(nav_ops), ids!(nav_ops_on)),
            (ids!(nav_cod), ids!(nav_cod_on)),
            (ids!(nav_riders), ids!(nav_riders_on)),
            (ids!(nav_sales), ids!(nav_sales_on)),
            (ids!(nav_day_close), ids!(nav_day_close_on)),
            (ids!(nav_reports), ids!(nav_reports_on)),
            (ids!(nav_inventory), ids!(nav_inventory_on)),
            (ids!(nav_branches), ids!(nav_branches_on)),
            (ids!(nav_credit), ids!(nav_credit_on)),
            (ids!(nav_team), ids!(nav_team_on)),
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
            (ids!(rail_riders), ids!(rail_riders_on)),
            (ids!(rail_sales), ids!(rail_sales_on)),
            (ids!(rail_day_close), ids!(rail_day_close_on)),
            (ids!(rail_reports), ids!(rail_reports_on)),
            (ids!(rail_inventory), ids!(rail_inventory_on)),
            (ids!(rail_branches), ids!(rail_branches_on)),
            (ids!(rail_credit), ids!(rail_credit_on)),
            (ids!(rail_team), ids!(rail_team_on)),
            (ids!(rail_settings), ids!(rail_settings_on)),
            (ids!(rail_lock), ids!(rail_lock_on)),
        ]
    }

    fn bottom_nav_icon_ids(&self) -> [&[LiveId]; NAV_TABS.len()] {
        [
            ids!(nav_books_icon),
            ids!(nav_sell_icon),
            ids!(nav_buy_icon),
            ids!(nav_rates_icon),
            ids!(nav_ops_icon),
            ids!(nav_cod_icon),
            ids!(nav_riders_icon),
            ids!(nav_sales_icon),
            ids!(nav_day_close_icon),
            ids!(nav_reports_icon),
            ids!(nav_inventory_icon),
            ids!(nav_branches_icon),
            ids!(nav_credit_icon),
            ids!(nav_team_icon),
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
            ids!(rail_riders_icon),
            ids!(rail_sales_icon),
            ids!(rail_day_close_icon),
            ids!(rail_reports_icon),
            ids!(rail_inventory_icon),
            ids!(rail_branches_icon),
            ids!(rail_credit_icon),
            ids!(rail_team_icon),
            ids!(rail_settings_icon),
            ids!(rail_lock_icon),
        ]
    }

    /// Slot icons follow the visible chrome order (same tab the caption wears).
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
                ids!(rail_riders_slot),
                ids!(rail_sales_slot),
                ids!(rail_day_close_slot),
                ids!(rail_reports_slot),
                ids!(rail_inventory_slot),
                ids!(rail_branches_slot),
                ids!(rail_credit_slot),
                ids!(rail_team_slot),
                ids!(rail_settings_slot),
                ids!(rail_lock_slot),
            ]
        } else {
            [
                ids!(nav_books_slot),
                ids!(nav_sell_slot),
                ids!(nav_buy_slot),
                ids!(nav_rates_slot),
                ids!(nav_ops_slot),
                ids!(nav_cod_slot),
                ids!(nav_riders_slot),
                ids!(nav_sales_slot),
                ids!(nav_day_close_slot),
                ids!(nav_reports_slot),
                ids!(nav_inventory_slot),
                ids!(nav_branches_slot),
                ids!(nav_credit_slot),
                ids!(nav_team_slot),
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
        let (reports_nav, order) = {
            let reports_nav = app_state().lock().unwrap().reports_nav;
            (reports_nav, current_nav_chrome_order())
        };
        let active = nav_tab_for(screen, reports_nav);
        // While dragging, the slot under the pointer wears the active face
        // so the drop target is unmistakable.
        let drop = nav_drag()
            .lock()
            .unwrap()
            .as_ref()
            .filter(|d| d.active)
            .and_then(|d| d.hover.map(|slot| (d.rail, slot)));
        for (rail, pairs) in [
            (false, self.bottom_nav_pairs()),
            (true, self.rail_nav_pairs()),
        ] {
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

    fn sync_awb_pools_label(&mut self, cx: &mut Cx) {
        let text = app_state()
            .lock()
            .unwrap()
            .store
            .as_ref()
            .and_then(|s| s.awb_pools_status_text().ok())
            .unwrap_or_else(|| "No carrier AWB pools set.\n".into());
        let rows: Vec<ReportRow> = text
            .lines()
            .map(|l| ReportRow::Line(l.to_string()))
            .collect();
        self.set_report_rows(cx, ids!(awb_pools_label), rows);
    }

    fn sync_venue_brand(&mut self, cx: &mut Cx) {
        let (title, header_title, icon_path, icon_hint, onboard_icon_hint, saved_title) = {
            let st = app_state().lock().unwrap();
            // A shop that has not set its name yet still shows the app name
            // next to the logo — never a blank header.
            let title = if st.venue_title.trim().is_empty() {
                "Moklanjo".to_string()
            } else {
                st.display_venue_title()
            };
            // Multi-outlet owners need to see which shop this device is selling
            // for; a single Main-only setup keeps the plain venue name.
            let header_title = {
                let branch_count = st
                    .store
                    .as_ref()
                    .and_then(|s| s.list_branches().ok())
                    .map(|b| b.len())
                    .unwrap_or(0);
                match st.store.as_ref().and_then(|s| s.active_branch().ok()) {
                    Some(branch) if branch_count > 1 => {
                        let tag = if branch.code.trim().is_empty() {
                            branch.name.clone()
                        } else {
                            branch.code.clone()
                        };
                        format!("{title} · {tag}")
                    }
                    _ => title.clone(),
                }
            };
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
            (
                title,
                header_title,
                icon_path,
                icon_hint,
                onboard_icon_hint,
                st.venue_title.clone(),
            )
        };
        self.ui
            .widget(cx, ids!(brand_title))
            .set_text(cx, &plain_display_text(&header_title));
        for id in [ids!(setup_brand_title), ids!(lock_brand_title)] {
            self.ui
                .widget(cx, id)
                .set_text(cx, &plain_display_text(&title));
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
        self.sync_printer_fields(cx);
    }

    /// Pool + routes draft, and the system printers found here.
    fn sync_printer_fields(&mut self, cx: &mut Cx) {
        let (pool_rows, route_rows, route_key) = {
            let st = app_state().lock().unwrap();
            (
                courier_printers_pool_rows(&st.printer_draft),
                courier_printer_route_rows(&st.printer_draft, &st.printer_route_key, |k| st.t(k)),
                st.printer_route_key.clone(),
            )
        };
        self.set_report_rows(cx, ids!(printers_pool_label), pool_rows);
        self.set_report_rows(cx, ids!(printer_routes_label), route_rows);
        let _ = route_key;
        self.show_found_printers(cx);
        self.ui.widget(cx, ids!(printers_found_label)).set_visible(cx, true);
    }

    fn save_venue_brand(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        let title = self.ui.text_input(cx, ids!(venue_title_input)).text();
        let outcome = (|| {
            let mut st = app_state().lock().unwrap();
            let actor = st
                .session
                .clone()
                .ok_or_else(|| "Unlock as Owner to change brand".to_string())?;
            if !actor.is_owner() {
                return Err("Only Owner can change in-app brand".to_string());
            }
            let store = st
                .store
                .as_ref()
                .ok_or_else(|| "Storage not ready".to_string())?;
            store.set_venue_title(&title)?;
            if let Some(path) = st.draft_venue_icon_path.clone() {
                let key = store.import_venue_icon(&path)?;
                if let Ok(dir) = store.brand_images_dir() {
                    invalidate_image_loads_for_path(&dir.join("venue_icon.webp"));
                }
                invalidate_image_loads_for_path(&path);
                st.venue_icon_key = Some(key);
                st.draft_venue_icon_path = None;
            }
            st.venue_title = title.trim().to_string();
            Ok("Brand saved".to_string())
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
            let store = st
                .store
                .as_ref()
                .ok_or_else(|| "Storage not ready".to_string())?;
            store.set_venue_icon_key(None)?;
            st.venue_icon_key = None;
            st.draft_venue_icon_path = None;
            Ok("Icon cleared".to_string())
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

    /// Push history when `record_history` (drill-down via `go`); peers use
    /// `go_peer` which clears stacks then calls with `record_history: false`.
    /// Lock/onboard destinations also clear history.
    fn set_screen(&mut self, st: &mut AppState, next: Screen, record_history: bool) {
        if st.screen == next {
            return;
        }
        if record_history && screen_keeps_history(st.screen) && screen_keeps_history(next) {
            st.nav_back.push(st.screen);
            if st.nav_back.len() > 32 {
                st.nav_back.remove(0);
            }
            st.nav_forward.clear();
        } else if !screen_keeps_history(next) {
            st.nav_back.clear();
            st.nav_forward.clear();
        }
        st.suggest_open = None;
        st.screen = next;
    }

    /// Drill-down: push current screen onto `nav_back` (browser-style).
    fn go(&mut self, cx: &mut Cx, screen: Screen) {
        self.go_with(cx, screen, true);
    }

    /// Chrome peer switch: replace screen as a new root (clears back/forward).
    fn go_peer(&mut self, cx: &mut Cx, screen: Screen) {
        self.go_with(cx, screen, false);
    }

    fn go_with(&mut self, cx: &mut Cx, screen: Screen, push_history: bool) {
        // More hub removed — its destinations are chrome peers.
        let screen = if screen == Screen::More {
            Screen::Menu
        } else {
            screen
        };
        {
            let mut st = app_state().lock().unwrap();
            if st.must_confirm_device_role && screen != Screen::DeviceSetup {
                st.status = st.t("device_use_q");
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
                    Screen::Lock | Screen::DeviceSetup | Screen::Onboard | Screen::ClientPair
                )
            {
                st.status = "Unlock first".into();
                self.set_screen(&mut st, Screen::Lock, false);
            } else if st.session.is_some()
                && st.owner_setup_complete
                && !st.onboarding_complete
                && st.device_role.is_host()
                && !matches!(screen, Screen::Onboard | Screen::DeviceSetup | Screen::Lock)
            {
                self.set_screen(&mut st, Screen::Onboard, false);
            } else {
                if !push_history {
                    st.nav_back.clear();
                    st.nav_forward.clear();
                }
                self.set_screen(&mut st, screen, push_history);
                // A device opens each screen on the office's latest books.
                st.pull_from_server();
                if st.device_role == DeviceRole::Client
                    && matches!(
                        screen,
                        Screen::Menu
                            | Screen::Inventory
                            | Screen::Branches
                            | Screen::Team
                            | Screen::Settings
                            | Screen::SidebarSettings
                    )
                {
                    st.status =
                        "Read-only on this extra till — change it on the shop device".into();
                }
            }
        }
        // Do not auto-scan printers on Settings open — Find is explicit.
        diag::breadcrumb(
            if push_history { "go" } else { "go_peer" },
            format!("{screen:?}"),
        );
        self.sync_chrome(cx);
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

    fn sync_diag_panel(&mut self, cx: &mut Cx) {
        self.ui
            .widget(cx, ids!(diag_status_label))
            .set_text(cx, &plain_display_text(&diag::status_text()));
        let level = diag::status()
            .map(|s| s.min_level)
            .unwrap_or(diag::Level::Info);
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
        if app_state().lock().unwrap().screen != Screen::Settings {
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
            self.show_ok(cx, app_state().lock().unwrap().t("diag_support_started"));
            return;
        }
        if self.ui.button(cx, ids!(diag_support_stop)).clicked(actions) {
            diag::end_support_session();
            self.sync_diag_panel(cx);
            self.show_ok(cx, app_state().lock().unwrap().t("diag_support_ended"));
            return;
        }
        if self.ui.button(cx, ids!(diag_export_local)).clicked(actions) {
            match diag::export_local() {
                Ok(paths) => {
                    self.sync_diag_panel(cx);
                    self.offer_diag_bundle_save(cx, paths, false);
                }
                Err(e) => self.show_error(cx, e),
            }
            return;
        }
        if self.ui.button(cx, ids!(diag_approve_send)).clicked(actions) {
            diag::approve_send();
            self.sync_diag_panel(cx);
            self.show_warn(cx, app_state().lock().unwrap().t("diag_send_approved"));
            return;
        }
        if self.ui.button(cx, ids!(diag_export_send)).clicked(actions) {
            match diag::export_for_send() {
                Ok(paths) => {
                    self.sync_diag_panel(cx);
                    self.offer_diag_bundle_save(cx, paths, true);
                }
                Err(e) => self.show_error(cx, e),
            }
            return;
        }
        if self.ui.button(cx, ids!(diag_clear)).clicked(actions) {
            match diag::clear_logs() {
                Ok(()) => {
                    self.sync_diag_panel(cx);
                    self.show_ok(cx, app_state().lock().unwrap().t("diag_cleared"));
                }
                Err(e) => self.show_error(cx, e),
            }
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

    fn print_or_share_receipt(&mut self, cx: &mut Cx, text: &str) {
        cx.copy_to_clipboard(text);
        let _ = print_share::copy_text(text);
        // Android: ACTION_SEND chooser, unless a network printer takes the
        // bill. Other platforms ignore this and open the file below.
        if !print_share::prints_here(print_share::PrintKind::Receipt) {
            cx.share_text(text);
        }
        let app_files = cx.get_data_dir().map(PathBuf::from);
        match print_share::print_text(print_share::PrintKind::Receipt, text, app_files.as_deref()) {
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

    /// True when a seller text field (not a button) holds key focus / soft keyboard.
    fn text_field_has_focus(&mut self, cx: &mut Cx) -> bool {
        let top_level = [
            ids!(cart_total_input),
            ids!(customer_name_input),
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
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
            ids!(venue_title_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_modifiers_input),
            ids!(inv_item_input),
            ids!(inv_qty_input),
            ids!(inv_note_input),
            ids!(inv_cost_input),
            ids!(inv_supplier_input),
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
        for (list_path, n) in [(ids!(cart_list), cart_n), (ids!(amend_cart_list), amend_n)] {
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
}

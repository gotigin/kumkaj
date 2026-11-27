impl App {
fn paint_chrome(&mut self, cx: &mut Cx) {
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |key: &str| locale::t(&lang, key);
        // Short chrome labels for bottom chips + rail (wrap, no ellipsis).
        // Page titles stay long (`items` = "Menu & items"); chrome uses `nav_items`.
        // Slots pack preference + rights (must match paint_nav_tabs).
        let order = current_nav_chrome_order();
        let my_pin_only = {
            let st = app_state().lock().unwrap();
            st.session.as_ref().is_some_and(|m| !m.can_access_settings())
        };
        let label_for = |tab: NavTab| -> String {
            match tab {
                // Chrome keeps its short Menu label (`nav_items`).
                NavTab::Menu => s("nav_items"),
                _ => s(nav_tab_label_key(tab, my_pin_only)),
            }
        };
        for pairs in [self.bottom_nav_pairs(), self.rail_nav_pairs()] {
            for (slot, (idle, on)) in pairs.iter().enumerate() {
                if let Some(tab) = order.get(slot).copied() {
                    let caption = nav_tab_caption(tab, &label_for(tab));
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
        self.ui
            .button(cx, ids!(device_role_restore))
            .set_text(cx, &s("restore_backup"));
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
            let help = self.ui.widget(cx, ids!(business_day_help)).as_info_help();
            help.set_title(cx, &s("business_day"));
            help.set_tip(&plain_display_text(&s("business_day_help")));
        }
        for path in [ids!(biz_day_until_close), ids!(biz_day_until_close_on)] {
            self.ui
                .button(cx, path)
                .set_text(cx, &s("biz_day_until_close"));
        }
        for path in [ids!(biz_day_midnight), ids!(biz_day_midnight_on)] {
            self.ui.button(cx, path).set_text(cx, &s("biz_day_midnight"));
        }
        {
            let help = self.ui.widget(cx, ids!(reports_help)).as_info_help();
            help.set_title(cx, &s("nav_reports"));
            help.set_nav_icon(cx, NavTab::Reports);
            help.set_tip(&plain_display_text(&s("reports_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(report_day_lens_help)).as_info_help();
            help.set_title(cx, &s("report_day_lens"));
            help.set_tip(&plain_display_text(&s("report_day_lens_help")));
        }
        for path in [ids!(report_lens_day_close), ids!(report_lens_day_close_on)] {
            self.ui
                .button(cx, path)
                .set_text(cx, &s("report_lens_day_close"));
        }
        for path in [ids!(report_lens_midnight), ids!(report_lens_midnight_on)] {
            self.ui
                .button(cx, path)
                .set_text(cx, &s("report_lens_midnight"));
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
            .widget(cx, ids!(lock_cloud_title))
            .set_text(cx, &plain_display_text(&s("cloud_account_title")));
        {
            let help = self.ui.widget(cx, ids!(lock_cloud_help)).as_info_help();
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("cloud_account_help")));
        }
        self.ui
            .text_input(cx, ids!(lock_cloud_name))
            .set_empty_text(cx, s("cloud_name_ph"));
        self.ui
            .text_input(cx, ids!(lock_cloud_email))
            .set_empty_text(cx, s("cloud_email_ph"));
        self.ui
            .text_input(cx, ids!(lock_cloud_password))
            .set_empty_text(cx, s("cloud_password_ph"));
        self.ui
            .button(cx, ids!(lock_cloud_ok))
            .set_text(cx, &s("cloud_continue"));
        self.ui
            .button(cx, ids!(lock_cloud_forgot))
            .set_text(cx, &s("cloud_forgot"));
        self.ui
            .button(cx, ids!(lock_cloud_web))
            .set_text(cx, &s("cloud_web_methods"));
        self.ui
            .button(cx, ids!(lock_cloud_shop))
            .set_text(cx, "Cloud plans / shop");
        self.ui
            .button(cx, ids!(lock_cloud_cancel))
            .set_text(cx, &s("cloud_cancel"));
        self.ui
            .widget(cx, ids!(lock_cloud_progress))
            .set_text(cx, &plain_display_text(&s("cloud_signing_in")));
        self.ui
            .widget(cx, ids!(lock_login_title))
            .set_text(cx, &plain_display_text(&s("or_unlock_shop_login")));
        {
            let help = self.ui.widget(cx, ids!(lock_login_help)).as_info_help();
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("shop_login_help")));
        }
        self.paint_cloud_auth_busy_chrome(cx);
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
            Screen::Drafts => ids!(drafts_screen),
            Screen::Buy => ids!(buy_screen),
            Screen::Floor => ids!(floor_screen),
            Screen::Kitchen => ids!(kitchen_screen),
            Screen::Menu => ids!(menu_screen),
            Screen::Sales => ids!(sales_screen),
            Screen::Credit => ids!(credit_screen),
            Screen::Team => ids!(team_screen),
            Screen::Settings => ids!(settings_screen),
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

    /// Set up Khaynjo opens with the shop's saved names in its fields, so an
    /// app template's name ("Rajdhani") shows there to keep or replace —
    /// never an empty field under a header that already says it (owner,
    /// 2026-11-14). Only on arrival: typing is never overwritten.
    fn prefill_onboard_names(&mut self, cx: &mut Cx) {
        let (title, org) = {
            let st = app_state().lock().unwrap();
            let org = st
                .store
                .as_ref()
                .and_then(|s| s.saved_org_name().ok())
                .unwrap_or_default();
            (st.venue_title.clone(), org)
        };
        for (id, value) in [(ids!(onboard_venue_input), title), (ids!(onboard_org_input), org)] {
            let field = self.ui.text_input(cx, id);
            if !cx.has_key_focus(field.area()) {
                field.set_text(cx, &value);
            }
        }
    }

    fn show_screen(&mut self, cx: &mut Cx, screen: Screen) {
        let _span = stall_span("show_screen");
        // Leaving Settings mid-edit: Bill & KOT format edits are written now.
        self.flush_format_autosave(cx);
        let screen_changed = sel_screen_changed(screen);
        if screen_changed && screen == Screen::Onboard {
            self.prefill_onboard_names(cx);
        }
        if screen_changed {
            self.clear_page_selection(cx);
            // A field on the screen we left keeps key focus while hidden:
            // `text_field_has_focus` then holds sync_chrome in its typing
            // early-return, so the header kept "Locked" after a PIN unlock
            // (the hidden Lock PIN field) until some other screen repainted.
            cx.set_key_focus(Area::Empty);
        }
        let pairs = [
            (ids!(device_setup_screen), screen == Screen::DeviceSetup),
            (ids!(lock_screen), screen == Screen::Lock),
            (ids!(onboard_screen), screen == Screen::Onboard),
            (ids!(client_pair_screen), screen == Screen::ClientPair),
            (ids!(books_screen), screen == Screen::Books),
            (ids!(pos_screen), screen == Screen::Pos),
            (ids!(drafts_screen), screen == Screen::Drafts),
            (ids!(buy_screen), screen == Screen::Buy),
            (ids!(floor_screen), screen == Screen::Floor),
            (ids!(kitchen_screen), screen == Screen::Kitchen),
            (ids!(menu_screen), screen == Screen::Menu),
            (ids!(sales_screen), screen == Screen::Sales),
            (ids!(credit_screen), screen == Screen::Credit),
            (ids!(team_screen), screen == Screen::Team),
            (ids!(settings_screen), screen == Screen::Settings),
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
        // Order | Counter stay broad (owner, 2026-11-13: a 340 | 380 trial made
        // both panes cramped — the wide panes + sideways pan are the design).
        const DESK_LEFT_MIN: f64 = 600.0;
        const DESK_RIGHT_MIN: f64 = 520.0;
        const DESK_GAP: f64 = 16.0;
        // Phone: how much of the next pane shows at the screen's edge, so a
        // sideways swipe to Counter (or back to Order) is plain to see.
        const DESK_PEEK: f64 = 28.0;
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
        // Tabs vs rail decides who carries the gesture-bar pad.
        self.fit_inside_system_bars(cx);
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
        // Bill & KOT format follows the new width too.
        self.layout_slip_editors(cx);

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
            // Side by side, never stacked; ScrollX pans when the mins do not
            // fit. On a phone each pane is one screen less a peek of the other
            // (DESIGN.md, Moklanjo's booking desk) — the fixed 600 px Order
            // pane used to run off a 411 px phone with its buttons cut.
            let (left, right) = order_desk_pane_widths(
                content_w,
                DESK_LEFT_MIN,
                DESK_RIGHT_MIN,
                DESK_GAP,
                DESK_PEEK,
            );
            (Size::Fixed(left), Size::Fixed(right))
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
            (ids!(nav_x14), ids!(nav_x14_on)),
            (ids!(nav_x15), ids!(nav_x15_on)),
            (ids!(nav_x16), ids!(nav_x16_on)),
            (ids!(nav_x17), ids!(nav_x17_on)),
            (ids!(nav_x18), ids!(nav_x18_on)),
            (ids!(nav_x19), ids!(nav_x19_on)),
            (ids!(nav_x20), ids!(nav_x20_on)),
            (ids!(nav_x21), ids!(nav_x21_on)),
            (ids!(nav_x22), ids!(nav_x22_on)),
            (ids!(nav_x23), ids!(nav_x23_on)),
            (ids!(nav_x24), ids!(nav_x24_on)),
            (ids!(nav_x25), ids!(nav_x25_on)),
            (ids!(nav_x26), ids!(nav_x26_on)),
            (ids!(nav_x27), ids!(nav_x27_on)),
            (ids!(nav_x28), ids!(nav_x28_on)),
            (ids!(nav_x29), ids!(nav_x29_on)),
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
            (ids!(rail_x14), ids!(rail_x14_on)),
            (ids!(rail_x15), ids!(rail_x15_on)),
            (ids!(rail_x16), ids!(rail_x16_on)),
            (ids!(rail_x17), ids!(rail_x17_on)),
            (ids!(rail_x18), ids!(rail_x18_on)),
            (ids!(rail_x19), ids!(rail_x19_on)),
            (ids!(rail_x20), ids!(rail_x20_on)),
            (ids!(rail_x21), ids!(rail_x21_on)),
            (ids!(rail_x22), ids!(rail_x22_on)),
            (ids!(rail_x23), ids!(rail_x23_on)),
            (ids!(rail_x24), ids!(rail_x24_on)),
            (ids!(rail_x25), ids!(rail_x25_on)),
            (ids!(rail_x26), ids!(rail_x26_on)),
            (ids!(rail_x27), ids!(rail_x27_on)),
            (ids!(rail_x28), ids!(rail_x28_on)),
            (ids!(rail_x29), ids!(rail_x29_on)),
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
            ids!(nav_x14_icon),
            ids!(nav_x15_icon),
            ids!(nav_x16_icon),
            ids!(nav_x17_icon),
            ids!(nav_x18_icon),
            ids!(nav_x19_icon),
            ids!(nav_x20_icon),
            ids!(nav_x21_icon),
            ids!(nav_x22_icon),
            ids!(nav_x23_icon),
            ids!(nav_x24_icon),
            ids!(nav_x25_icon),
            ids!(nav_x26_icon),
            ids!(nav_x27_icon),
            ids!(nav_x28_icon),
            ids!(nav_x29_icon),
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
            ids!(rail_x14_icon),
            ids!(rail_x15_icon),
            ids!(rail_x16_icon),
            ids!(rail_x17_icon),
            ids!(rail_x18_icon),
            ids!(rail_x19_icon),
            ids!(rail_x20_icon),
            ids!(rail_x21_icon),
            ids!(rail_x22_icon),
            ids!(rail_x23_icon),
            ids!(rail_x24_icon),
            ids!(rail_x25_icon),
            ids!(rail_x26_icon),
            ids!(rail_x27_icon),
            ids!(rail_x28_icon),
            ids!(rail_x29_icon),
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

    /// Settings' sidebar list icons: same load path as the rail, run
    /// from chrome sync (outside PortalList draw) so WebP textures are
    /// ready before the next paint.
    fn paint_sidebar_settings_icons(&mut self, cx: &mut Cx) {
        // The list lives on the Settings hub (was its own Sidebar screen).
        {
            let st = app_state().lock().unwrap();
            if st.screen != Screen::Settings || st.settings_panel.is_some() {
                return;
            }
        }
        let order = app_state().lock().unwrap().nav_order.clone();
        let host = self.ui.widget(cx, ids!(sidebar_nav_list));
        let list = host.portal_list(cx, ids!(list));
        if let Some(mut inner) = list.borrow_mut() {
            inner.set_item_range(cx, 0, order.len());
        }
        let mut fresh = false;
        for (i, tab) in order.iter().enumerate() {
            let item = list.item(cx, i, id!(Item));
            if load_nav_icon_once(cx, &item.image(cx, ids!(icon)), *tab) {
                fresh = true;
            }
        }
        // Texture upload lands next frame — same as SidebarNavList draw.
        if fresh {
            host.redraw(cx);
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
                ids!(rail_x14_slot),
                ids!(rail_x15_slot),
                ids!(rail_x16_slot),
                ids!(rail_x17_slot),
                ids!(rail_x18_slot),
                ids!(rail_x19_slot),
                ids!(rail_x20_slot),
                ids!(rail_x21_slot),
                ids!(rail_x22_slot),
                ids!(rail_x23_slot),
                ids!(rail_x24_slot),
                ids!(rail_x25_slot),
                ids!(rail_x26_slot),
                ids!(rail_x27_slot),
                ids!(rail_x28_slot),
                ids!(rail_x29_slot),
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
                ids!(nav_x14_slot),
                ids!(nav_x15_slot),
                ids!(nav_x16_slot),
                ids!(nav_x17_slot),
                ids!(nav_x18_slot),
                ids!(nav_x19_slot),
                ids!(nav_x20_slot),
                ids!(nav_x21_slot),
                ids!(nav_x22_slot),
                ids!(nav_x23_slot),
                ids!(nav_x24_slot),
                ids!(nav_x25_slot),
                ids!(nav_x26_slot),
                ids!(nav_x27_slot),
                ids!(nav_x28_slot),
                ids!(nav_x29_slot),
            ]
        }
    }

    /// Exactly one nav peer highlighted for `screen` (bottom and/or rail).
    /// Slot order packs preference + rights: peers the signed-in person cannot
    /// open never take a slot (draw used to re-show them after sync_chrome hid
    /// them — that left forbidden tabs visible on Client / staff unlocks).
    fn paint_nav_tabs(&mut self, cx: &mut Cx, screen: Screen) {
        let order = current_nav_chrome_order();
        // A Settings section or Team on the bar lights up itself; otherwise
        // its screen's peer (Settings).
        let own = {
            let mut st = app_state().lock().unwrap();
            let own = match screen {
                Screen::Settings => st.settings_panel.and_then(nav_tab_for_settings_panel),
                Screen::Team => Some(NavTab::Team),
                _ => None,
            };
            // Unlock brings this person back to the item now selected — a
            // Settings section or Team too, whether or not it is on the bar.
            if st.screen == screen {
                st.remember_nav_tab(own.or_else(|| nav_tab_for(screen)));
            }
            own
        };
        let active = own.filter(|t| order.contains(t)).or_else(|| nav_tab_for(screen));
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
                let selected = active == Some(order[slot]);
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
            // Short shop id beside the restaurant name (smaller sibling label):
            // the shop's LAN id on every device — a Client shows the id it
            // paired with, the Server its own (it showed its books folder
            // id, so one shop read "(4f71d56e)" on the Server and
            // "(81e13dca)" on its tablets — owner rule 2026-11-14: devices
            // differ only in connecting). Books folder id only before books.
            let server_id_short = match st.device_role {
                DeviceRole::Client => st
                    .store
                    .as_ref()
                    .and_then(|s| s.paired_server_id().ok().flatten()),
                _ => st
                    .store
                    .as_ref()
                    .and_then(|s| s.lan_server_id().ok())
                    .or_else(|| {
                        st.data_base
                            .as_ref()
                            .and_then(|base| local_servers::active_server_id(base).ok().flatten())
                    }),
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
                locale::t(&lang, if stamp { "stamp_on" } else { "stamp_none" }),
                locale::t(
                    &lang,
                    if qr {
                        "pay_qr_on"
                    } else {
                        "pay_qr_off"
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
        // Every screen change (peer tab, drill-down, Back, Forward) closes an
        // open list: the next screen never inherits one (popup.rs).
        st.sel_menu = None;
        st.suggest_open = None;
        st.dish_picker_open = false;
        st.category_picker_open = false;
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
    ///
    /// A switch to a screen that took ≥ 1 s last time (slow device, big
    /// shop) first paints the loading screen and runs on the next frame, so
    /// the tap is answered at once (owner, 2026-11-13).
    fn go_with(&mut self, cx: &mut Cx, screen: Screen, is_peer: bool) {
        let elsewhere = app_state().lock().unwrap().screen != screen;
        if elsewhere && nav_was_slow(screen) {
            self.ui
                .widget(cx, ids!(nav_loading_label))
                .set_text(cx, &app_t("nav_loading"));
            self.ui.view(cx, ids!(nav_loading)).set_visible(cx, true);
            self.ui.redraw(cx);
            *nav_veil_pending().lock().unwrap() = Some((cx.new_next_frame(), screen, is_peer));
            return;
        }
        self.go_with_now(cx, screen, is_peer);
    }

    /// The loading screen's frame has painted: switch now, then take it down.
    fn finish_nav_veil(&mut self, cx: &mut Cx, e: &NextFrameEvent) {
        let due = {
            let mut pending = nav_veil_pending().lock().unwrap();
            match pending.as_ref() {
                Some((frame, _, _)) if e.set.contains(frame) => pending.take(),
                _ => None,
            }
        };
        if let Some((_, screen, is_peer)) = due {
            self.go_with_now(cx, screen, is_peer);
            self.ui.view(cx, ids!(nav_loading)).set_visible(cx, false);
            self.ui.redraw(cx);
        }
    }

    fn go_with_now(&mut self, cx: &mut Cx, screen: Screen, is_peer: bool) {
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
                if screen == Screen::Kitchen && is_peer {
                    // Peer jump lands on Live tickets (not a buried Recall tab).
                    st.kitchen_panel = KitchenPanel::Live;
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
        // Time this switch up to its first draw (`note_nav_draw`).
        let landed = app_state().lock().unwrap().screen;
        *nav_measure().lock().unwrap() = Some((landed, Instant::now()));
        self.sync_chrome(cx);
        // A Settings section nav item: its section, now that Settings is up.
        let panel = {
            let mut st = app_state().lock().unwrap();
            let p = st.settings_panel_after_go.take();
            p.filter(|_| st.screen == Screen::Settings)
        };
        if panel.is_some() {
            self.open_settings_panel(cx, panel);
        }
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
            (ids!(settings_panel_whatsapp), "whatsapp"),
            (ids!(settings_panel_backup), "backup"),
            (ids!(settings_panel_diag), "diag"),
            (ids!(settings_panel_privacy), "privacy"),
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
                // One list of nav items now: how to open, order and show them.
                let tip = if my_pin_only {
                    s("settings_hub_help_my_pin")
                } else {
                    format!("{}\n\n{}", s("settings_hub_help"), s("sidebar_settings_help"))
                };
                help.set_tip(&plain_display_text(&tip));
            }
            self.ui
                .button(cx, ids!(settings_detail_back))
                .set_text(cx, &s("back"));
            // One list of nav items (sections and screens, owner 2026-11-15)
            // for shop admins; staff who manage only their own PIN get that row.
            self.ui.view(cx, ids!(settings_nav_bulk)).set_visible(cx, shop);
            self.ui.widget(cx, ids!(sidebar_nav_list)).set_visible(cx, shop);
            self.ui.button(cx, ids!(settings_hub_team)).set_text(
                cx,
                &s(if my_pin_only {
                    "my_pin_settings"
                } else {
                    "team_pins"
                }),
            );
            self.ui
                .view(cx, ids!(settings_hub_team_row))
                .set_visible(cx, my_pin_only);
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
        if panel == Some("whatsapp") {
            self.paint_whatsapp_panel(cx);
        }
        if panel == Some("diag") {
            self.sync_diag_panel(cx);
        }
        if panel == Some("privacy") {
            self.paint_privacy_panel(cx);
        }
        // Hub ↔ section: always refresh ‹ (do not rely on a later full sync_chrome).
        self.paint_header_back_chrome(cx);
    }

    /// Settings → WhatsApp Cloud API (Server credentials).
    fn paint_whatsapp_panel(&mut self, cx: &mut Cx) {
        let (title, help, pub_cfg, is_client, enable_on, labels) = {
            let st = app_state().lock().unwrap();
            let pub_cfg = st
                .store
                .as_ref()
                .and_then(|s| s.whatsapp_cloud_public().ok())
                .unwrap_or_default();
            let labels = (
                st.t("whatsapp_phone_id_ph"),
                st.t("whatsapp_token_ph"),
                st.t("whatsapp_template_ph"),
                st.t("whatsapp_template_lang_ph"),
                st.t("whatsapp_save"),
                st.t("whatsapp_test"),
                st.t("whatsapp_disconnect"),
                st.t("whatsapp_server_only"),
                st.t("whatsapp_enable"),
                st.t("whatsapp_disable"),
            );
            (
                st.t("settings_sec_whatsapp"),
                st.t("whatsapp_cloud_help"),
                pub_cfg,
                st.device_role == DeviceRole::Client,
                st.wa_cloud_enabled,
                labels,
            )
        };
        self.ui
            .widget(cx, ids!(h_whatsapp))
            .set_text(cx, &plain_display_text(&title));
        {
            let tip = self.ui.widget(cx, ids!(h_whatsapp_help)).as_info_help();
            tip.set_tip(&plain_display_text(&help));
        }
        self.ui
            .widget(cx, ids!(whatsapp_server_hint))
            .set_text(cx, &plain_display_text(&labels.7));
        self.ui
            .widget(cx, ids!(whatsapp_server_hint))
            .set_visible(cx, is_client);
        self.paint_choice_pair(
            cx,
            ids!(wa_enable_off),
            ids!(wa_enable_off_on),
            !enable_on,
        );
        self.paint_choice_pair(cx, ids!(wa_enable_on), ids!(wa_enable_on_on), enable_on);
        self.ui
            .button(cx, ids!(wa_enable_off))
            .set_text(cx, &labels.9);
        self.ui
            .button(cx, ids!(wa_enable_off_on))
            .set_text(cx, &labels.9);
        self.ui
            .button(cx, ids!(wa_enable_on))
            .set_text(cx, &labels.8);
        self.ui
            .button(cx, ids!(wa_enable_on_on))
            .set_text(cx, &labels.8);

        let phone_field = self.ui.text_input(cx, ids!(wa_phone_id_input));
        phone_field.set_empty_text(cx, labels.0.clone());
        if !cx.has_key_focus(phone_field.area()) && !is_client {
            phone_field.set_text(cx, &pub_cfg.phone_number_id);
        }
        let token_field = self.ui.text_input(cx, ids!(wa_token_input));
        token_field.set_empty_text(cx, labels.1.clone());
        // Never paint the access token into the field.
        if !cx.has_key_focus(token_field.area()) {
            token_field.set_text(cx, "");
        }
        let tmpl = self.ui.text_input(cx, ids!(wa_template_input));
        tmpl.set_empty_text(cx, labels.2.clone());
        if !cx.has_key_focus(tmpl.area()) && !is_client {
            tmpl.set_text(cx, &pub_cfg.template_name);
        }
        let lang = self.ui.text_input(cx, ids!(wa_template_lang_input));
        lang.set_empty_text(cx, labels.3.clone());
        if !cx.has_key_focus(lang.area()) && !is_client {
            lang.set_text(cx, &pub_cfg.template_lang);
        }
        self.ui.button(cx, ids!(wa_save_btn)).set_text(cx, &labels.4);
        self.ui.button(cx, ids!(wa_test_btn)).set_text(cx, &labels.5);
        self.ui
            .button(cx, ids!(wa_disconnect_btn))
            .set_text(cx, &labels.6);
        let editable = !is_client;
        self.ui.button(cx, ids!(wa_save_btn)).set_visible(cx, editable);
        self.ui.button(cx, ids!(wa_test_btn)).set_visible(cx, editable);
        self.ui
            .button(cx, ids!(wa_disconnect_btn))
            .set_visible(cx, editable);
    }

    /// Settings → Privacy: same `PRIVACY.md` as the website, rendered as Markdown
    /// (WrapLabel → Makepad Markdown / pulldown-cmark), not raw source.
    fn paint_privacy_panel(&mut self, cx: &mut Cx) {
        let title = app_state().lock().unwrap().t("settings_sec_privacy");
        self.ui
            .widget(cx, ids!(h_privacy))
            .set_text(cx, &plain_display_text(&title));
        // Feed Markdown source; the widget presents headings/lists/links.
        self.ui
            .widget(cx, ids!(privacy_body))
            .set_text(cx, privacy_policy::PRIVACY_MD);
    }

    /// Show one Settings section (or the hub with `None`). Lightweight — no
    /// full `sync_chrome` (that can freeze phones on Settings taps).
    fn open_settings_panel(&mut self, cx: &mut Cx, panel: Option<&'static str>) {
        // Leaving Bill & KOT format: what was edited is written now.
        self.flush_format_autosave(cx);
        {
            let mut st = app_state().lock().unwrap();
            st.settings_panel = panel;
            st.suggest_open = None;
            st.dish_picker_open = false;
            st.category_picker_open = false;
        }
        // Clear list faces without a full sync_chrome (Settings freeze risk).
        self.redraw_suggest_lists(cx);
        self.ui
            .view(cx, ids!(settings_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        diag::breadcrumb("settings_panel", panel.unwrap_or("hub"));
        self.sync_settings_panels(cx);
        // A section on the rail / bottom bar lights up itself (else Settings).
        self.paint_nav_tabs(cx, Screen::Settings);
        // Heavy section paints only when that section is opened — not on every
        // Settings hub visit or every sync_chrome while Settings is open.
        if panel == Some("bill") {
            // Fresh open lands on Bill tab (KOT is one tap away).
            app_state().lock().unwrap().bill_kot_tab_kot = false;
            // Ends with sync_bill_kot_tabs — a second call here only redrew
            // the same previews again.
            self.fill_bill_format_fields(cx);
        }
        if panel == Some("printers") {
            self.paint_printers_panel(cx);
        }
        if panel == Some("backup") {
            self.paint_backup_status(cx);
        }
        if panel == Some("brand") {
            if app_state().lock().unwrap().catalogue_public.is_empty()
                && app_state().lock().unwrap().catalogue_public_error.is_empty()
            {
                if let Ok(base) = store::data_dir() {
                    catalogue_fetch_job(CatalogueFetchKind::PublicIndex, base);
                }
            }
            self.paint_settings_app_templates(cx);
        }
        if panel == Some("privacy") {
            self.paint_privacy_panel(cx);
        }
        if panel == Some("whatsapp") {
            {
                let mut st = app_state().lock().unwrap();
                if let Some(store) = st.store.as_ref() {
                    if let Ok(p) = store.whatsapp_cloud_public() {
                        st.wa_cloud_enabled = p.enabled;
                    }
                }
            }
            self.paint_whatsapp_panel(cx);
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
                locale::t(&lang, "stamp_on")
            } else {
                locale::t(&lang, "stamp_none")
            };
            let pay_qr_hint = if store.payment_qr_path().ok().flatten().is_some() {
                locale::t(&lang, "pay_qr_on")
            } else {
                locale::t(&lang, "pay_qr_off")
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

    /// Kitchen: Live / Recall / Stations — one panel visible (no long bury).
    fn sync_kitchen_panels(&mut self, cx: &mut Cx) {
        let (panel, lang) = {
            let st = app_state().lock().unwrap();
            (st.kitchen_panel, st.locale.language.clone())
        };
        let s = |k: &str| locale::t(&lang, k);
        for (idle, on, key) in [
            (ids!(kitchen_tab_live), ids!(kitchen_tab_live_on), "kitchen_tab_live"),
            (ids!(kitchen_tab_recall), ids!(kitchen_tab_recall_on), "kitchen_tab_recall"),
            (
                ids!(kitchen_tab_stations),
                ids!(kitchen_tab_stations_on),
                "kitchen_tab_stations",
            ),
        ] {
            let t = s(key);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, on).set_text(cx, &t);
        }
        let live = panel == KitchenPanel::Live;
        let recall = panel == KitchenPanel::Recall;
        let stations = panel == KitchenPanel::Stations;
        self.paint_choice_pair(cx, ids!(kitchen_tab_live), ids!(kitchen_tab_live_on), live);
        self.paint_choice_pair(cx, ids!(kitchen_tab_recall), ids!(kitchen_tab_recall_on), recall);
        self.paint_choice_pair(
            cx,
            ids!(kitchen_tab_stations),
            ids!(kitchen_tab_stations_on),
            stations,
        );
        self.ui.view(cx, ids!(kitchen_panel_live)).set_visible(cx, live);
        self.ui.view(cx, ids!(kitchen_panel_recall)).set_visible(cx, recall);
        self.ui
            .view(cx, ids!(kitchen_panel_stations))
            .set_visible(cx, stations);
        // The accordion state is the one truth for the stations body: the tab
        // used to force the body visible while the state said closed, so the
        // first full repaint (first tap into its form) collapsed it and the
        // page jumped to the top. Opening the tab opens the section once
        // (match_event); after that its header toggles it like any accordion.
        self.sync_floor_sections(cx);
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
        let (bill_land, bill_del, kot_land, kot_del, bill_n) = {
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
        let mut paint_del = |this: &mut Self, os, os_on, esc, esc_on, del: &str| {
            this.paint_choice_pair(
                cx,
                os,
                os_on,
                del == print_share::DELIVERY_SYSTEM_SPOOLER || del.is_empty(),
            );
            // A printer saved as Raw sends the same ESC/POS bytes.
            this.paint_choice_pair(cx, esc, esc_on, del == print_share::DELIVERY_ESCPOS || del == print_share::DELIVERY_RAW_DEVICE);
        };
        paint_del(
            self,
            ids!(bill_deliv_os),
            ids!(bill_deliv_os_on),
            ids!(bill_deliv_escpos),
            ids!(bill_deliv_escpos_on),
            &bill_del,
        );
        paint_del(
            self,
            ids!(kot_deliv_os),
            ids!(kot_deliv_os_on),
            ids!(kot_deliv_escpos),
            ids!(kot_deliv_escpos_on),
            &kot_del,
        );
        let label = if bill_n == 0 {
            app_t("bill_printers_dd")
        } else {
            app_t("bill_printers_dd_n").replace("{n}", &bill_n.to_string())
        };
        // A dropdown face is its label only; ComboField draws ˅ / ˄.
        let face = label;
        self.ui.button(cx, ids!(bill_printers_dd)).set_text(cx, &face);
        let bill_all = app_state()
            .lock()
            .unwrap()
            .printer_draft
            .print_mode_for(print_share::RouteKey::Bill)
            == print_share::RoutePrintMode::All;
        self.paint_choice_pair(cx, ids!(bill_print_first), ids!(bill_print_first_on), !bill_all);
        self.paint_choice_pair(cx, ids!(bill_print_all), ids!(bill_print_all_on), bill_all);
        self.sync_preview_paper_ui(cx);
    }


    /// The 3 × 3 grid beside the Bill or KOT preview: a picture of the
    /// paper, the format's place lit — the square is where the slip lands
    /// (left and right are the paper's own sides in every language;
    /// docs/PRINTER.md § Paper and preview).
    fn paint_place_grid(&mut self, cx: &mut Cx, kot: bool) {
        let place = {
            let st = app_state().lock().unwrap();
            let key = if kot {
                print_share::RouteKey::Kot { station_id: "" }
            } else {
                print_share::RouteKey::Bill
            };
            st.printer_draft.format_for(key).place
        };
        for (down, row) in place_cells(kot).into_iter().enumerate() {
            for (across, (idle, on)) in row.into_iter().enumerate() {
                let lit = place == os_paper::Place::new(down as u8, across as u8);
                self.paint_choice_pair(cx, idle, on, lit);
            }
        }
    }

    /// A square of the grid was picked: the Bill or KOT format keeps the
    /// place with this device's printers (as portrait / landscape is), and
    /// the preview draws it at once.
    fn set_format_place(&mut self, cx: &mut Cx, kot: bool, place: os_paper::Place) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.device_role == DeviceRole::Client {
                return Err(st.t("printers_host_only"));
            }
            st.printer_draft.set_job_place(!kot, place);
            st.save_printer_draft()?;
            print_share::configure(st.printer_draft.clone());
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        if kot {
            self.sync_kot_format_ui(cx);
        } else {
            self.sync_bill_format_ui(cx);
        }
        self.paint_place_grid(cx, kot);
    }
    /// The preview's printer on the Bill and KOT tabs: the dropdown face is
    /// `name · virtual paper` of the printer the preview is drawn on, the two
    /// lines under it read its reported and its virtual paper, the ℹ says
    /// what the preview shows (docs/PRINTER.md § Paper and preview). Every
    /// saved printer can be picked; it changes the preview only. A picture
    /// drawn on another paper than its face names (the tab's printers
    /// changed, Find added one, a paper was typed, a driver answered) is
    /// drawn again: the two always agree.
    fn sync_preview_paper_ui(&mut self, cx: &mut Cx) {
        thread_local! {
            static RESYNC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        }
        for (kot, dd, help, preview, reported_id, virtual_id) in [
            (
                false,
                ids!(bill_preview_paper_dd),
                ids!(bill_preview_help),
                ids!(bill_format_preview),
                ids!(bill_preview_reported),
                ids!(bill_preview_virtual),
            ),
            (
                true,
                ids!(kot_preview_paper_dd),
                ids!(kot_preview_help),
                ids!(kot_format_preview),
                ids!(kot_preview_reported),
                ids!(kot_preview_virtual),
            ),
        ] {
            let (face, tip, now, reported, virtual_line) = {
                let st = app_state().lock().unwrap();
                let now = st.preview_paper(kot);
                let printer = st.preview_printer(kot);
                let face = match printer {
                    Some(p) => printer_paper_line(p, |k| st.t(k)),
                    None => format!(
                        "{} · {}",
                        st.t("no_printer_assigned"),
                        st.t("paper_roll")
                            .replace("{w}", &os_paper::mm_text(print_share::DEFAULT_PAPER_WIDTH_MM))
                    ),
                };
                // The picked printer's two papers, read-only: what its
                // driver reports ("N/A" when nothing does) and the virtual
                // paper the picture is drawn on.
                let reported = match printer {
                    Some(p) => reported_summary(p, |k| st.t(k)),
                    None => st.t("na"),
                };
                (
                    face,
                    st.t("preview_help"),
                    now.clone(),
                    st.t("reported_paper").replace("{paper}", &reported),
                    st.t("virtual_paper")
                        .replace("{paper}", &paper_text(now.virtual_paper, |k| st.t(k))),
                )
            };
            // A dropdown face is its value only; ComboField draws ˅ / ˄.
            let btn = self.ui.button(cx, dd);
            if btn.text() != face {
                btn.set_text(cx, &face);
            }
            self.ui
                .widget(cx, help)
                .as_info_help()
                .set_tip(&plain_display_text(&tip));
            for (id, text) in [(reported_id, &reported), (virtual_id, &virtual_line)] {
                self.ui.widget(cx, id).set_text(cx, &plain_display_text(text));
            }
            self.paint_place_grid(cx, kot);
            let stale = self
                .ui
                .widget(cx, preview)
                .borrow::<KotSlipPreview>()
                .is_some_and(|s| s.shown_paper().is_some_and(|p| *p != now));
            if stale && !RESYNC.with(|r| r.replace(true)) {
                if kot {
                    self.sync_kot_format_ui(cx);
                } else {
                    self.sync_bill_format_ui(cx);
                }
                RESYNC.with(|r| r.set(false));
            }
        }
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
            (ids!(content_width_58), ids!(content_width_58_on), 58u32, "58 mm"),
            (ids!(content_width_80), ids!(content_width_80_on), 80, "80 mm"),
            (ids!(content_width_112), ids!(content_width_112_on), 112, "112 mm"),
            (ids!(content_width_210), ids!(content_width_210_on), 210, "210 mm"),
        ] {
            self.ui.button(cx, idle).set_text(cx, label);
            self.ui.button(cx, on).set_text(cx, label);
            self.paint_choice_pair(cx, idle, on, mm == choice);
        }
    }

    /// Bill & KOT format save themselves (owner, 2026-11-14: "every change
    /// is automatically saved … the user doesn't need to remember a button").
    /// An edit marks its draft and restarts a short timer; the timer, a Bill
    /// | KOT tab switch, or leaving the screen writes it — the same writes
    /// the old Save buttons made. Errors (a Client out of reach) still show.
    fn schedule_format_autosave(&mut self, cx: &mut Cx, bill: bool, kot: bool) {
        let mut st = app_state().lock().unwrap();
        st.format_save_bill |= bill;
        st.format_save_kot |= kot;
        if !st.format_save_timer.is_empty() {
            cx.stop_timer(st.format_save_timer);
        }
        st.format_save_timer = cx.start_timeout(FORMAT_AUTOSAVE_SECS);
    }

    /// Write whatever Bill & KOT format edits are still waiting.
    fn flush_format_autosave(&mut self, cx: &mut Cx) {
        let (bill, kot) = {
            let mut st = app_state().lock().unwrap();
            if !st.format_save_timer.is_empty() {
                cx.stop_timer(st.format_save_timer);
                st.format_save_timer = Timer::empty();
            }
            (
                std::mem::take(&mut st.format_save_bill),
                std::mem::take(&mut st.format_save_kot),
            )
        };
        let mut failed = None;
        if bill && let Err(e) = self.save_bill_format(cx) {
            failed = Some(e);
        }
        if kot && let Err(e) = self.save_kot_format(cx) {
            failed = Some(e);
        }
        if bill || kot {
            self.publish_print_formats();
        }
        if let Some(e) = failed {
            self.set_status(cx, e);
        }
    }

    /// The bill draft as the shop's bill format (header, FSSAI, footer, UPI,
    /// line styles, print size).
    fn save_bill_format(&mut self, cx: &mut Cx) -> Result<(), String> {
        // The open line's typed value lands first.
        self.read_bill_editor_into_draft(cx);
        let mut st = app_state().lock().unwrap();
        let fmt = st.bill_slip_draft.clone().clamped();
        let (cur_vpa, cur_payee) = match st.store.as_ref() {
            Some(store) => (
                store.upi_vpa().unwrap_or_default(),
                store.upi_payee().unwrap_or_default(),
            ),
            None => (String::new(), String::new()),
        };
        let upi_vpa = (fmt.upi_vpa.trim() != cur_vpa.trim()).then(|| fmt.upi_vpa.clone());
        let upi_payee = (fmt.upi_payee.trim() != cur_payee.trim()).then(|| fmt.upi_payee.clone());
        let em = fmt.style(print_share::BillStyleField::Items).size_em as f32;
        st.printer_draft.set_standard_em(em);
        st.books_op_light(BooksOp::SetBillFormat {
            header: fmt.header_lines(),
            fssai: fmt.fssai.clone(),
            footer: fmt.footer.clone(),
            upi_vpa,
            upi_payee,
            styles: fmt.to_json(),
        })?;
        st.books_op_light(BooksOp::SetStandardEm { em })?;
        Ok(())
    }

    /// The KOT draft as the shop's kitchen-ticket format.
    fn save_kot_format(&mut self, cx: &mut Cx) -> Result<(), String> {
        let size_now = self.ui.text_input(cx, ids!(kot_field_size_input)).text();
        let mut st = app_state().lock().unwrap();
        // A size still being typed lands on the field it belongs to.
        if let Ok(raw) = size_now.trim().parse::<u32>() {
            let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
            let kind = st.kot_edit_kind;
            let field = st.kot_style_field;
            st.kot_slip_draft.kind_styles.get_mut(kind).get_mut(field).size_em = em;
            if field == print_share::KotStyleField::ItemQty {
                st.kot_slip_draft.em = em;
            }
        }
        let format = st.kot_slip_draft.clone();
        st.books_op_light(BooksOp::SetKotFormat { format })?;
        Ok(())
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
        // The Paper note says whether the new width fits each printer's paper.
        self.sync_job_drive_ui(cx);
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
            let source = st.kot_preview_source.clone();
            let text = match &source {
                KotPreviewSource::Sample => draft.render_sample_kind(&shop, kind),
                KotPreviewSource::Ticket(id) => st
                    .store
                    .as_ref()
                    .and_then(|store| store.format_kot_text_as(id, &draft).ok())
                    .unwrap_or_else(|| draft.render_sample_kind(&shop, kind)),
            };
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
    /// Bill & KOT format at the window's real width: the open line's
    /// controls in one row when there is room, two rows on a tablet, stacked
    /// on a phone; the preview beside the form only when both fit.
    /// Bill and KOT format tabs on every width: the same rules for both.
    /// Each group of controls goes onto one row when its card is wide
    /// enough and stacks when it is not; form and preview sit side by side
    /// on wide screens.
    fn layout_slip_editors(&mut self, cx: &mut Cx) {
        // Settings card body: the content column less the cards' padding.
        let page = (*content_width().lock().unwrap() - 72.0).max(240.0);
        // Form and preview side by side only when the form keeps ≥ 640 px;
        // every row below is sized for the form card it sits in.
        let side = page >= 1100.0;
        let preview_w = (page * 0.38).clamp(360.0, 520.0);
        let w = if side { page - preview_w - 12.0 } else { page };
        let flow_if = |fits: bool| if fits { Flow::right() } else { Flow::Down };
        // Paper cards span the page; the format forms span their column.
        let rules: [(&[LiveId], bool); 26] = [
            (ids!(content_width_choices), page >= 860.0),
            (ids!(cw_row_a), page >= 400.0),
            (ids!(cw_row_b), page >= 400.0),
            (ids!(bill_deliv_row), page >= 560.0),
            (ids!(kot_deliv_row), page >= 560.0),
            // The open line: all of it in one row, else name + size over
            // weight + align, else everything stacked.
            (ids!(bill_line_head), w >= 1000.0),
            (ids!(bill_line_main), w >= 560.0),
            (ids!(bill_line_style), w >= 560.0),
            (ids!(kot_line_head), w >= 1000.0),
            (ids!(kot_line_main), w >= 560.0),
            (ids!(kot_line_style), w >= 560.0),
            // Switches two per row; totals one row each.
            (ids!(bill_item_opts_a), w >= 560.0),
            (ids!(bill_item_opts_b), w >= 560.0),
            (ids!(bill_total_row_qty), w >= 520.0),
            (ids!(bill_total_row_sub), w >= 520.0),
            (ids!(bill_total_row_grand), w >= 520.0),
            // Logo: the two sizes and the two greys pair up on a phone; the
            // two picture buttons (longer words in many languages) from 400.
            (ids!(bill_logo_btns), w >= 400.0),
            (ids!(bill_logo_size), w >= 300.0),
            (ids!(bill_logo_tones), w >= 260.0),
            (ids!(kot_item_opts_a), w >= 560.0),
            (ids!(kot_item_opts_b), w >= 560.0),
            (ids!(kot_title_pieces), w >= 560.0),
            // KOT kinds: four in a row, else two by two.
            (ids!(kot_kind_row), w >= 560.0),
            (ids!(kot_copy_row), w >= 640.0),
            (ids!(bill_split), side),
            (ids!(kot_split), side),
        ];
        for (path, fits) in rules {
            set_view_flow(cx, &self.ui.view(cx, path), flow_if(fits));
        }
        // Column rows: header box and the three numbers side by side when
        // they fit, else the numbers share their own full row.
        let col_one_row = w >= 640.0;
        for row in BILL_COL_ROWS.into_iter().chain(KOT_COL_ROWS) {
            set_view_flow(cx, &self.ui.view(cx, &[row, id!(col_fields)]), flow_if(col_one_row));
            set_view_walk(
                cx,
                &self.ui.view(cx, &[row, id!(col_nums)]),
                Walk {
                    width: if col_one_row { Size::Fixed(348.0) } else { Size::fill() },
                    height: Size::fit(),
                    ..Default::default()
                },
            );
        }
        for col in [ids!(bill_preview_col), ids!(kot_preview_col)] {
            set_view_walk(
                cx,
                &self.ui.view(cx, col),
                Walk {
                    width: if side { Size::Fixed(preview_w) } else { Size::fill() },
                    height: Size::fit(),
                    ..Default::default()
                },
            );
        }
    }

    /// Form and preview side by side: the preview moves down to sit beside
    /// the open line, so the change and its result are on screen together
    /// with no scrolling back up. Measured after a draw (rows wrap with the
    /// width and the font) and kept inside the form's height, so the page
    /// never grows; stacked screens keep the preview under the form.
    fn align_slip_previews(&mut self, cx: &mut Cx) {
        // Runs after every draw: only while the panel is on screen, and only
        // for the tab shown (widget lookups are not free on a tree this big).
        let shown = {
            let st = app_state().lock().unwrap();
            let open = matches!(st.screen, Screen::Settings) && st.settings_panel == Some("bill");
            open.then_some(usize::from(st.bill_kot_tab_kot))
        };
        let Some(shown) = shown else {
            return;
        };
        for (split, form, editor, col, lead) in SLIP_PREVIEW_ALIGN.into_iter().skip(shown).take(1) {
            let lead_view = self.ui.view(cx, lead);
            let current = match lead_view.borrow().map(|v| v.walk.height) {
                Some(Size::Fixed(h)) => h,
                _ => 0.0,
            };
            let side = self
                .ui
                .view(cx, split)
                .borrow()
                .map(|v| matches!(v.layout.flow, Flow::Right { .. }))
                .unwrap_or(false);
            let editor_view = self.ui.view(cx, editor);
            let form_r = self.ui.view(cx, form).area().rect(cx);
            let editor_r = editor_view.area().rect(cx);
            let col_h = self.ui.view(cx, col).area().rect(cx).size.y;
            let target = if side && editor_view.visible() && editor_r.size.y > 1.0 && form_r.size.y > 1.0 {
                let card_h = (col_h - current).max(0.0);
                (editor_r.pos.y - form_r.pos.y)
                    .min(form_r.size.y - card_h)
                    .max(0.0)
            } else {
                0.0
            };
            if (target - current).abs() > 1.0 {
                set_view_walk(
                    cx,
                    &lead_view,
                    Walk {
                        width: Size::fill(),
                        height: Size::Fixed(target),
                        ..Default::default()
                    },
                );
                // Measured inside a draw, where a plain redraw is dropped:
                // queue the next draw of this page explicitly.
                if let Some(list) = self.ui.view(cx, split).area().draw_list_id() {
                    cx.redraw_list_in_draw(list);
                }
            }
        }
    }

    fn sync_bill_format_ui(&mut self, cx: &mut Cx) {
        // While a box is being typed, the draft must already hold that text
        // so the picture and a save in the same moment agree.
        let size_in = self.ui.text_input(cx, ids!(bill_field_size_input));
        if cx.has_key_focus(size_in.area()) {
            if let Ok(raw) = size_in.text().trim().parse::<u32>() {
                let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
                let mut st = app_state().lock().unwrap();
                if let Some(id) = st.bill_custom_id.clone() {
                    if let Some(line) = st.bill_slip_draft.custom_line_mut(&id) {
                        line.style.size_em = em;
                    }
                } else {
                    let field = st.bill_style_field;
                    st.bill_slip_draft.style_mut(field).size_em = em;
                }
            }
        }
        let value_in = self.ui.text_input(cx, ids!(bill_field_value));
        if cx.has_key_focus(value_in.area()) {
            self.commit_bill_field_value(cx);
        }
        let (draft, field, custom_id, shop, when, preview_source) = {
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
                st.bill_custom_id.clone(),
                shop,
                when,
                st.bill_preview_source.clone(),
            )
        };
        let lang = app_state().lock().unwrap().locale.language.clone();
        let s = |k: &str| locale::t(&lang, k);
        let custom_open = custom_id
            .as_ref()
            .and_then(|id| draft.custom_line(id).cloned());
        let style = if let Some(ref c) = custom_open {
            c.style.clone()
        } else {
            draft.style(field).clone()
        };
        let line_name = |kind: print_share::BillStyleField| s(kind.label_key());
        let custom_label = |c: &print_share::BillCustomLine| {
            let name = {
                let t = c.text.trim();
                if t.is_empty() {
                    s("bill_custom_line")
                } else {
                    t.chars().take(28).collect::<String>()
                }
            };
            let on_off = if c.style.show {
                s("kot_field_on")
            } else {
                s("kot_field_off")
            };
            format!(
                "{name} · {on_off} · {} · {} · {}",
                c.style.size_em,
                s(c.style.weight.locale_key()),
                s(c.style.align.locale_key())
            )
        };
        let row_label_field = |kind: print_share::BillStyleField| {
            let row = draft.style(kind);
            let name = match draft.meta_pair_of(kind) {
                Some(partner) => format!("{} + {}", line_name(kind), line_name(partner)),
                None => line_name(kind),
            };
            let on_off = if row.show {
                s("kot_field_on")
            } else {
                s("kot_field_off")
            };
            if kind == print_share::BillStyleField::Logo {
                let side = |px: u32| if px == 0 { "—".to_string() } else { px.to_string() };
                return format!(
                    "{name} · {on_off} · {} × {} px · {}",
                    side(draft.logo_width_px),
                    side(draft.logo_height_px),
                    s(row.align.locale_key())
                );
            }
            format!(
                "{} · {on_off} · {} · {} · {}",
                name,
                row.size_em,
                s(row.weight.locale_key()),
                s(row.align.locale_key())
            )
        };
        let order_ids = draft.ordered_ids();
        let open_id = if let Some(ref id) = custom_id {
            format!("custom:{id}")
        } else {
            field.id().to_string()
        };
        let open_idx = order_ids.iter().position(|id| id == &open_id).unwrap_or(0);
        for (i, (idle, on)) in BILL_PICK_BTNS.into_iter().enumerate() {
            let before = i < open_idx && i < order_ids.len();
            let after = i > open_idx && i < order_ids.len();
            self.ui.view(cx, BILL_PICK_ROWS[i]).set_visible(cx, before);
            self.ui.view(cx, BILL_AFTER_ROWS[i]).set_visible(cx, after);
            if let Some(oid) = order_ids.get(i) {
                let label = if let Some(c) = draft.custom_line(oid) {
                    custom_label(c)
                } else if let Some(kind) = print_share::BillStyleField::from_id(oid) {
                    row_label_field(kind)
                } else {
                    oid.clone()
                };
                let pairs: [(&[LiveId], &[LiveId]); 2] = [(idle, on), BILL_AFTER_BTNS[i]];
                for (a, b) in pairs {
                    self.ui.button(cx, a).set_text(cx, &label);
                    self.ui.button(cx, b).set_text(cx, &label);
                    self.paint_choice_pair(cx, a, b, false);
                }
            }
        }
        let open_name = if let Some(ref c) = custom_open {
            let t = c.text.trim();
            if t.is_empty() {
                s("bill_custom_line")
            } else {
                t.chars().take(40).collect()
            }
        } else {
            line_name(field)
        };
        self.ui
            .widget(cx, ids!(bill_line_name))
            .set_text(cx, &plain_display_text(&open_name));
        self.ui
            .button(cx, ids!(bill_custom_add))
            .set_text(cx, &s("bill_custom_add"));
        self.ui
            .button(cx, ids!(bill_custom_remove))
            .set_text(cx, &s("bill_custom_remove"));
        self.ui
            .button(cx, ids!(bill_custom_remove))
            .set_visible(cx, custom_open.is_some());
        // Logo has a box size, not a type size or weight.
        let logo_open = custom_open.is_none() && field == print_share::BillStyleField::Logo;

        self.ui.view(cx, ids!(bill_size_box)).set_visible(cx, !logo_open);
        self.ui.view(cx, ids!(bill_line_weight)).set_visible(cx, !logo_open);
        self.ui.view(cx, ids!(bill_logo_pieces)).set_visible(cx, logo_open);
        if logo_open {
            for (id, key) in [
                (ids!(h_bill_logo_picture), "bill_logo_picture"),
                (ids!(h_bill_logo_tones), "bill_logo_tones"),
            ] {
                self.ui.widget(cx, id).set_text(cx, &plain_display_text(&s(key)));
            }
            for (id, key) in [
                (ids!(bill_logo_help), "bill_logo_help"),
                (ids!(bill_logo_tones_help), "bill_logo_tones_help"),
            ] {
                self.ui
                    .widget(cx, id)
                    .as_info_help()
                    .set_tip(&plain_display_text(&s(key)));
            }
            let status = match draft.logo_image.as_ref() {
                Some(img) => format!("{} × {} px", img.width, img.height),
                None => s("bill_logo_none"),
            };
            self.ui
                .widget(cx, ids!(bill_logo_status))
                .set_text(cx, &plain_display_text(&status));
            self.ui.button(cx, ids!(bill_logo_choose)).set_text(cx, &s("bill_logo_choose"));
            self.ui.button(cx, ids!(bill_logo_remove)).set_text(cx, &s("bill_logo_remove"));
            self.ui
                .button(cx, ids!(bill_logo_remove))
                .set_visible(cx, draft.logo_image.is_some());
            // Blank follows the picture; the hint names the box.
            for (id, key, px) in [
                (ids!(bill_logo_width_input), "bill_logo_width", draft.logo_width_px),
                (ids!(bill_logo_height_input), "bill_logo_height", draft.logo_height_px),
            ] {
                let input = self.ui.text_input(cx, id);
                input.set_empty_text(cx, s(key));
                if !cx.has_key_focus(input.area()) {
                    input.set_text(cx, &if px == 0 { String::new() } else { px.to_string() });
                }
            }
            for (idle, on, key, tones) in [
                (ids!(bill_logo_solid), ids!(bill_logo_solid_on), "bill_logo_solid", print_share::BillLogoTones::Solid),
                (ids!(bill_logo_shaded), ids!(bill_logo_shaded_on), "bill_logo_shaded", print_share::BillLogoTones::Shaded),
            ] {
                let t = s(key);
                self.ui.button(cx, idle).set_text(cx, &t);
                self.ui.button(cx, on).set_text(cx, &t);
                self.paint_choice_pair(cx, idle, on, draft.logo_tones == tones);
            }
        }
        self.ui
            .widget(cx, ids!(h_bill_field_size))
            .set_text(cx, &plain_display_text(&s("kot_field_size")));
        self.layout_slip_editors(cx);
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
        // The open line's faces in the owner's language — the same words as
        // on the KOT tab (they were painted for selection only and stayed
        // English here).
        for (idle, on, key) in [
            (ids!(bill_field_show), ids!(bill_field_show_on), "kot_field_show"),
            (ids!(bill_field_hide), ids!(bill_field_hide_on), "kot_field_hide"),
            (ids!(bill_w_light), ids!(bill_w_light_on), "kot_weight_light"),
            (ids!(bill_w_reg), ids!(bill_w_reg_on), "kot_weight_regular"),
            (ids!(bill_w_bold), ids!(bill_w_bold_on), "kot_weight_bold"),
            (ids!(bill_a_start), ids!(bill_a_start_on), "kot_align_start"),
            (ids!(bill_a_center), ids!(bill_a_center_on), "kot_align_center"),
            (ids!(bill_a_end), ids!(bill_a_end_on), "kot_align_end"),
        ] {
            let t = s(key);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, on).set_text(cx, &t);
        }
        self.paint_choice_pair(cx, ids!(bill_field_show), ids!(bill_field_show_on), style.show);
        self.paint_choice_pair(cx, ids!(bill_field_hide), ids!(bill_field_hide_on), !style.show);
        let items_open = custom_open.is_none() && field == print_share::BillStyleField::Items;
        let total_open = custom_open.is_none() && field == print_share::BillStyleField::Total;
        self.ui.view(cx, ids!(bill_item_pieces)).set_visible(cx, items_open);
        self.ui.view(cx, ids!(bill_total_pieces)).set_visible(cx, total_open);
        if items_open {
            for (id, key) in [
                (ids!(h_bill_item_layout), "bill_item_layout"),
                (ids!(h_bill_item_header), "bill_item_header"),
                (ids!(h_bill_item_col_sep), "bill_item_col_sep"),
                (ids!(h_bill_item_note), "bill_item_note"),
                (ids!(h_bill_item_col_order), "slip_columns"),
            ] {
                self.ui.widget(cx, id).set_text(cx, &plain_display_text(&s(key)));
            }
            self.ui
                .widget(cx, ids!(bill_columns_help))
                .as_info_help()
                .set_tip(&plain_display_text(&s("slip_columns_help")));
            let columns = draft.item_layout == print_share::BillItemLayout::Columns;
            for (idle, on, key, selected) in [
                (ids!(bill_item_layout_columns), ids!(bill_item_layout_columns_on), "bill_item_layout_columns", columns),
                (ids!(bill_item_layout_stacked), ids!(bill_item_layout_stacked_on), "bill_item_layout_stacked", !columns),
                (ids!(bill_item_header_on), ids!(bill_item_header_on_on), "bill_item_piece_on", draft.item_show_header),
                (ids!(bill_item_header_off), ids!(bill_item_header_off_on), "bill_item_piece_off", !draft.item_show_header),
                (ids!(bill_item_sep_on), ids!(bill_item_sep_on_on), "bill_item_piece_on", draft.item_col_sep),
                (ids!(bill_item_sep_off), ids!(bill_item_sep_off_on), "bill_item_piece_off", !draft.item_col_sep),
                (ids!(bill_item_note_on), ids!(bill_item_note_on_on), "bill_item_piece_on", draft.item_show_note),
                (ids!(bill_item_note_off), ids!(bill_item_note_off_on), "bill_item_piece_off", !draft.item_show_note),
            ] {
                let t = s(key);
                self.ui.button(cx, idle).set_text(cx, &t);
                self.ui.button(cx, on).set_text(cx, &t);
                self.paint_choice_pair(cx, idle, on, selected);
            }
            self.ui.view(cx, ids!(bill_item_sep_box)).set_visible(cx, columns);
            // One row per column, in print order: show, header, width, start,
            // end and alignment of that column, all in its row.
            let col_order = draft.clone().clamped().item_col_order;
            let cols = draft.item_columns();
            for (i, row) in BILL_COL_ROWS.into_iter().enumerate() {
                let Some(col) = col_order.get(i).and_then(|id| print_share::BillItemCol::from_id(id)) else {
                    self.ui.view(cx, &[row]).set_visible(cx, false);
                    continue;
                };
                self.ui.view(cx, &[row]).set_visible(cx, true);
                // The row's title names the column; its header box is just
                // "Header" — not a second name for the same column.
                let (name_key, shown, header) = match col {
                    print_share::BillItemCol::Name => ("bill_item_name", draft.item_show_name, draft.item_header_item.as_str()),
                    print_share::BillItemCol::Qty => ("bill_item_qty", draft.item_show_qty, draft.item_header_qty.as_str()),
                    print_share::BillItemCol::Price => ("bill_item_price", draft.item_show_price, draft.item_header_price.as_str()),
                    print_share::BillItemCol::Amount => ("bill_item_amount", draft.item_show_amount, draft.item_header_amount.as_str()),
                };
                self.ui
                    .widget(cx, &[row, id!(col_name)])
                    .set_text(cx, &plain_display_text(&s(name_key)));
                for (idle, on, key, selected) in [
                    (id!(col_show_on), id!(col_show_on_on), "bill_item_piece_on", shown),
                    (id!(col_show_off), id!(col_show_off_on), "bill_item_piece_off", !shown),
                ] {
                    let t = s(key);
                    self.ui.button(cx, &[row, idle]).set_text(cx, &t);
                    self.ui.button(cx, &[row, on]).set_text(cx, &t);
                    self.paint_choice_pair(cx, &[row, idle], &[row, on], selected);
                }
                self.ui.view(cx, &[row, id!(col_fields)]).set_visible(cx, columns);
                self.ui.view(cx, &[row, id!(col_align)]).set_visible(cx, columns);
                let hdr = self.ui.text_input(cx, &[row, id!(col_header)]);
                self.ui
                    .view(cx, &[row, id!(col_header_box)])
                    .set_visible(cx, columns && draft.item_show_header);
                hdr.set_empty_text(cx, s("slip_col_header"));
                if !cx.has_key_focus(hdr.area()) {
                    hdr.set_text(cx, header);
                }
                let c = *cols.get(col);
                let width = if (c.width_mm - c.width_mm.round()).abs() < 0.05 {
                    format!("{}", c.width_mm.round() as i32)
                } else {
                    format!("{:.1}", c.width_mm)
                };
                for (child, key, val) in [
                    (id!(col_width), "slip_col_width", width),
                    (id!(col_start), "slip_col_start", c.start_px.to_string()),
                    (id!(col_end), "slip_col_end", c.end_px.to_string()),
                ] {
                    let input = self.ui.text_input(cx, &[row, child]);
                    input.set_empty_text(cx, s(key));
                    if !cx.has_key_focus(input.area()) {
                        input.set_text(cx, &val);
                    }
                }
                for ((idle, on), align) in [
                    (id!(col_a_start), id!(col_a_start_on)),
                    (id!(col_a_center), id!(col_a_center_on)),
                    (id!(col_a_end), id!(col_a_end_on)),
                ]
                .into_iter()
                .zip(print_share::KotAlign::ALL)
                {
                    let t = s(align.locale_key());
                    self.ui.button(cx, &[row, idle]).set_text(cx, &t);
                    self.ui.button(cx, &[row, on]).set_text(cx, &t);
                    self.paint_choice_pair(cx, &[row, idle], &[row, on], c.align == align);
                }
            }
        }
        if total_open {
            for (id, key) in [
                (ids!(h_bill_total_qty), "bill_total_qty"),
                (ids!(h_bill_total_subtotal), "bill_total_subtotal"),
                (ids!(h_bill_total_grand), "bill_total_grand"),
            ] {
                self.ui.widget(cx, id).set_text(cx, &plain_display_text(&s(key)));
            }
            for (idle, on, key, selected) in [
                (ids!(bill_total_qty_on), ids!(bill_total_qty_on_on), "bill_item_piece_on", draft.total_show_qty),
                (ids!(bill_total_qty_off), ids!(bill_total_qty_off_on), "bill_item_piece_off", !draft.total_show_qty),
                (ids!(bill_total_subtotal_on), ids!(bill_total_subtotal_on_on), "bill_item_piece_on", draft.total_show_subtotal),
                (ids!(bill_total_subtotal_off), ids!(bill_total_subtotal_off_on), "bill_item_piece_off", !draft.total_show_subtotal),
                (ids!(bill_total_grand_on), ids!(bill_total_grand_on_on), "bill_item_piece_on", draft.total_show_grand),
                (ids!(bill_total_grand_off), ids!(bill_total_grand_off_on), "bill_item_piece_off", !draft.total_show_grand),
            ] {
                let t = s(key);
                self.ui.button(cx, idle).set_text(cx, &t);
                self.ui.button(cx, on).set_text(cx, &t);
                self.paint_choice_pair(cx, idle, on, selected);
            }
            // Each total's label box sits in its row; its hint names it.
            for (id, key, val) in [
                (ids!(bill_total_qty_label_input), "bill_total_qty_label", draft.total_qty_label.as_str()),
                (ids!(bill_total_subtotal_label_input), "bill_total_subtotal_label", draft.total_subtotal_label.as_str()),
                (ids!(bill_total_grand_label_input), "bill_total_grand_label", draft.total_grand_label.as_str()),
                (ids!(bill_total_paid_label_input), "bill_total_paid_label", draft.total_paid_label.as_str()),
            ] {
                let input = self.ui.text_input(cx, id);
                input.set_empty_text(cx, s(key));
                if !cx.has_key_focus(input.area()) {
                    input.set_text(cx, val);
                }
            }
            // Where each total prints: its number under one of the item
            // columns that print, or on its own line; then its caption.
            // Stacked items have no columns to sit under.
            let columns = draft.item_layout == print_share::BillItemLayout::Columns;
            let printed = draft.ordered_item_cols();
            let col_key = |col: print_share::BillItemCol| match col {
                print_share::BillItemCol::Name => "bill_item_name",
                print_share::BillItemCol::Qty => "bill_item_qty",
                print_share::BillItemCol::Price => "bill_item_price",
                print_share::BillItemCol::Amount => "bill_item_amount",
            };
            let tip = plain_display_text(&s("bill_total_place_help"));
            for (block, kind) in BILL_TOTAL_PLACES.into_iter().zip(print_share::BillTotalKind::ALL) {
                self.ui.view(cx, &[block]).set_visible(cx, columns);
                if !columns {
                    continue;
                }
                let place = draft.total_place(kind);
                // Set under a column that does not print: it has its own line.
                let number = place.number_col().filter(|col| printed.contains(col));
                self.ui
                    .widget(cx, &[block, id!(tp_number_h)])
                    .set_text(cx, &plain_display_text(&s("bill_total_number_under")));
                self.ui.widget(cx, &[block, id!(tp_help)]).as_info_help().set_tip(&tip);
                let face = |this: &mut Self, cx: &mut Cx, idle: LiveId, on: LiveId, text: &str, selected: bool| {
                    this.ui.button(cx, &[block, idle]).set_text(cx, text);
                    this.ui.button(cx, &[block, on]).set_text(cx, text);
                    this.paint_choice_pair(cx, &[block, idle], &[block, on], selected);
                };
                face(self, cx, id!(tp_n_own), id!(tp_n_own_on), &s("bill_total_own_line"), number.is_none());
                for (seg, idle, on, col) in BILL_TOTAL_NUMBER_COLS {
                    let shown = printed.contains(&col);
                    self.ui.view(cx, &[block, seg]).set_visible(cx, shown);
                    if shown {
                        face(self, cx, idle, on, &s(col_key(col)), number == Some(col));
                    }
                }
                self.ui.view(cx, &[block, id!(tp_caption)]).set_visible(cx, number.is_some());
                let Some(number) = number else {
                    continue;
                };
                self.ui
                    .widget(cx, &[block, id!(tp_caption_h)])
                    .set_text(cx, &plain_display_text(&s("bill_total_caption_under")));
                // A caption set under the number's own column, or under one
                // that does not print, goes with the number.
                let caption = match place.caption_place() {
                    print_share::BillTotalCaption::Under(col) if col != number && printed.contains(&col) => {
                        print_share::BillTotalCaption::Under(col)
                    }
                    print_share::BillTotalCaption::Hidden => print_share::BillTotalCaption::Hidden,
                    _ => print_share::BillTotalCaption::WithNumber,
                };
                face(
                    self,
                    cx,
                    id!(tp_c_with),
                    id!(tp_c_with_on),
                    &s("bill_total_caption_with"),
                    caption == print_share::BillTotalCaption::WithNumber,
                );
                face(
                    self,
                    cx,
                    id!(tp_c_none),
                    id!(tp_c_none_on),
                    &s("bill_total_caption_none"),
                    caption == print_share::BillTotalCaption::Hidden,
                );
                for (seg, idle, on, col) in BILL_TOTAL_CAPTION_COLS {
                    let shown = col != number && printed.contains(&col);
                    self.ui.view(cx, &[block, seg]).set_visible(cx, shown);
                    if shown {
                        face(self, cx, idle, on, &s(col_key(col)), caption == print_share::BillTotalCaption::Under(col));
                    }
                }
                let span_ok = matches!(caption, print_share::BillTotalCaption::Under(_));
                self.ui.view(cx, &[block, id!(tp_span)]).set_visible(cx, span_ok);
                if span_ok {
                    face(
                        self,
                        cx,
                        id!(tp_span_off),
                        id!(tp_span_off_on),
                        &s("bill_total_caption_span_off"),
                        !place.caption_span,
                    );
                    face(
                        self,
                        cx,
                        id!(tp_span_on),
                        id!(tp_span_on_on),
                        &s("bill_total_caption_span_on"),
                        place.caption_span,
                    );
                }
            }
        }
        let pairable = custom_open.is_none() && field.is_pairable();
        self.ui.view(cx, ids!(bill_meta_pair)).set_visible(cx, pairable);
        if pairable {
            self.ui
                .widget(cx, ids!(h_bill_meta_pair))
                .set_text(cx, &plain_display_text(&s("bill_meta_pair")));
            let partner = draft.meta_pair_of(field);
            for (idle, on, label, other) in [
                (ids!(bill_pair_alone), ids!(bill_pair_alone_on), s("bill_pair_alone"), None),
                (ids!(bill_pair_customer), ids!(bill_pair_customer_on), s(print_share::BillStyleField::Customer.label_key()), Some(print_share::BillStyleField::Customer)),
                (ids!(bill_pair_billno), ids!(bill_pair_billno_on), s(print_share::BillStyleField::BillNo.label_key()), Some(print_share::BillStyleField::BillNo)),
                (ids!(bill_pair_time), ids!(bill_pair_time_on), s(print_share::BillStyleField::Time.label_key()), Some(print_share::BillStyleField::Time)),
                (ids!(bill_pair_cashier), ids!(bill_pair_cashier_on), s(print_share::BillStyleField::Cashier.label_key()), Some(print_share::BillStyleField::Cashier)),
                (ids!(bill_pair_table), ids!(bill_pair_table_on), s(print_share::BillStyleField::Table.label_key()), Some(print_share::BillStyleField::Table)),
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
        // The input ignores being hidden; its row does not.
        let value_visible = custom_open.is_some() || field.has_text();
        self.ui.view(cx, ids!(bill_value_row)).set_visible(cx, value_visible);
        if let Some(ref c) = custom_open {
            value.set_empty_text(cx, s("bill_custom_value_ph"));
            if !cx.has_key_focus(value.area()) {
                value.set_text(cx, &c.text);
            }
        } else {
            value.set_empty_text(cx, field.value_hint().to_string());
            if field.has_text() && !cx.has_key_focus(value.area()) {
                value.set_text(cx, &draft.text(field));
            }
        }
        let meta = custom_open.is_none() && field.is_meta();
        self.ui.view(cx, ids!(bill_meta_empty)).set_visible(cx, meta);
        if meta {
            self.ui
                .widget(cx, ids!(h_bill_meta_empty))
                .set_text(cx, &plain_display_text(&s("bill_meta_empty_help")));
            let policy = draft.meta_empty(field);
            let always = s("bill_meta_empty_always");
            let omit = s("bill_meta_empty_omit");
            self.ui
                .button(cx, ids!(bill_empty_always))
                .set_text(cx, &always);
            self.ui
                .button(cx, ids!(bill_empty_always_on))
                .set_text(cx, &always);
            self.ui
                .button(cx, ids!(bill_empty_omit))
                .set_text(cx, &omit);
            self.ui
                .button(cx, ids!(bill_empty_omit_on))
                .set_text(cx, &omit);
            self.paint_choice_pair(
                cx,
                ids!(bill_empty_always),
                ids!(bill_empty_always_on),
                policy == print_share::BillMetaEmpty::AlwaysShow,
            );
            self.paint_choice_pair(
                cx,
                ids!(bill_empty_omit),
                ids!(bill_empty_omit_on),
                policy == print_share::BillMetaEmpty::OmitWhenEmpty,
            );
        }

        let preview = self.bill_format_preview_text(&draft, &shop, &when, &preview_source);
        let source_face = match &preview_source {
            BillPreviewSource::Sample => s("bill_preview_sample"),
            BillPreviewSource::Running(_) => s("bill_preview_running"),
            BillPreviewSource::Sale(_) => s("bill_preview_sale"),
        };
        self.ui
            .button(cx, ids!(bill_preview_source_dd))
            .set_text(cx, &source_face);
        let (mm, landscape, base) = {
            let st = app_state().lock().unwrap();
            let mm = st.bill_slip_draft.shown_content_width_mm();
            let base = st
                .printer_draft
                .format_for(print_share::RouteKey::Bill);
            (mm, base.landscape, base)
        };
        let metrics = print_share::metrics_at_content_mm(&base, mm);
        if let Some(mut slip) = self
            .ui
            .widget(cx, ids!(bill_format_preview))
            .borrow_mut::<KotSlipPreview>()
        {
            let paper = app_state().lock().unwrap().preview_paper(false);
            let lay = crate::os_paper::Lay { landscape, place: base.place };
            slip.show_slip_on(cx, &preview, &metrics, lay, style.size_em as f32, &paper);
        }
        self.ui.view(cx, ids!(bill_format_preview_card)).redraw(cx);
    }

    fn bill_format_preview_text(
        &self,
        draft: &print_share::BillSlipFormat,
        shop: &str,
        when: &str,
        source: &BillPreviewSource,
    ) -> String {
        let st = app_state().lock().unwrap();
        let Some(store) = st.store.as_ref() else {
            return draft.render_sample(shop, when);
        };
        let cashier = st
            .session
            .as_ref()
            .map(|sess| sess.name.clone())
            .unwrap_or_default();
        match source {
            BillPreviewSource::Sample => draft.render_sample(shop, when),
            BillPreviewSource::Running(id) => store
                .running_order_by_id(id)
                .ok()
                .flatten()
                .and_then(|order| {
                    store
                        .format_running_bill_text_as(&order, &cashier, draft)
                        .ok()
                })
                .unwrap_or_else(|| draft.render_sample(shop, when)),
            BillPreviewSource::Sale(id) => store
                .format_receipt_text_as(id, draft)
                .unwrap_or_else(|_| draft.render_sample(shop, when)),
        }
    }

    /// Write the value box into the draft for the line that is open.
    fn commit_bill_field_value(&mut self, cx: &mut Cx) {
        let text = self.ui.text_input(cx, ids!(bill_field_value)).text();
        let mut st = app_state().lock().unwrap();
        if let Some(id) = st.bill_custom_id.clone() {
            if let Some(line) = st.bill_slip_draft.custom_line_mut(&id) {
                line.text = text;
            }
            return;
        }
        let field = st.bill_style_field;
        if field.has_text() {
            st.bill_slip_draft.set_text(field, &text);
        }
    }

    /// Address and font size currently in the boxes, onto the open bill line.
    fn read_bill_editor_into_draft(&mut self, cx: &mut Cx) {
        self.commit_bill_field_value(cx);
        let size_now = self.ui.text_input(cx, ids!(bill_field_size_input)).text();
        if let Ok(raw) = size_now.trim().parse::<u32>() {
            let em = raw.clamp(print_share::KOT_EM_MIN, print_share::KOT_EM_MAX);
            let mut st = app_state().lock().unwrap();
            if let Some(id) = st.bill_custom_id.clone() {
                if let Some(line) = st.bill_slip_draft.custom_line_mut(&id) {
                    line.style.size_em = em;
                }
            } else {
                let field = st.bill_style_field;
                st.bill_slip_draft.style_mut(field).size_em = em;
            }
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
            let draft = st.bill_slip_draft.clone();
            let source = st.bill_preview_source.clone();
            let copies = draft.copies.clamp(1, 3);
            let em = draft.style(print_share::BillStyleField::Items).size_em as f32;
            let cols = print_share::wrap_cols_for_paper_mm(draft.shown_content_width_mm());
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
            // Preview text locks app_state — never call it under `st`.
            let out = self.bill_format_preview_text(&draft, &shop, &when, &source);
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
                if let Some(id) = st.kot_custom_id.clone() {
                    if let Some(line) = st.kot_slip_draft.custom_line_mut(&id) {
                        line.style.size_em = em;
                    }
                } else {
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
        }
        let (draft, shop, lang, edit_kind, style_field, custom_id, preview_source) = {
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
                st.kot_custom_id.clone(),
                st.kot_preview_source.clone(),
            )
        };
        let s = |k: &str| locale::t(&lang, k);
        let custom_open = custom_id
            .as_ref()
            .and_then(|id| draft.custom_line(id).cloned());
        let field_style = if let Some(ref c) = custom_open {
            c.style.clone()
        } else {
            draft.field_style(edit_kind, style_field)
        };
        let shown = if let Some(ref c) = custom_open {
            c.show
        } else {
            draft.field_shown(style_field)
        };
        for (id, key) in [
            (ids!(h_kot_kind), "kot_kind_heading"),
            (ids!(h_kot_field_size), "kot_field_size"),
            (ids!(h_kot_copies), "kot_copies"),
            (ids!(h_kot_preview), "kot_preview"),
            (ids!(h_kot_copy_from), "kot_copy_from"),
        ] {
            self.ui.widget(cx, id).set_text(cx, &plain_display_text(&s(key)));
        }
        self.ui.widget(cx, ids!(h_kot_format_fields)).set_text(
            cx,
            &plain_display_text(&format!(
                "{} · {}",
                s("kot_format_fields"),
                s(edit_kind.locale_key())
            )),
        );
        self.ui
            .widget(cx, ids!(kot_copy_from_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("kot_copy_from_help")));
        self.ui
            .button(cx, ids!(kot_format_reset))
            .set_text(cx, &s("reset_kot_format"));
        let test_n = app_state().lock().unwrap().test_printer_ids.len();
        // A dropdown face is its label only; ComboField draws ˅ / ˄.
        let test_face = if test_n == 0 {
            s("kot_test_printers_dd")
        } else {
            s("kot_test_printers_dd_n").replace("{n}", &test_n.to_string())
        };
        self.ui
            .button(cx, ids!(kot_test_printers_dd))
            .set_text(cx, &test_face);
        self.ui
            .button(cx, ids!(kot_format_test))
            .set_text(cx, &s("test_kot_format"));
        // Kinds, copies, and the open line's own chips.
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
            (ids!(kot_align_start), ids!(kot_align_start_on), "kot_align_start"),
            (ids!(kot_align_center), ids!(kot_align_center_on), "kot_align_center"),
            (ids!(kot_align_end), ids!(kot_align_end_on), "kot_align_end"),
            (ids!(kot_copies_1), ids!(kot_copies_1_on), "kot_copies_1"),
            (ids!(kot_copies_2), ids!(kot_copies_2_on), "kot_copies_2"),
            (ids!(kot_copies_3), ids!(kot_copies_3_on), "kot_copies_3"),
        ] {
            let t = s(label);
            self.ui.button(cx, idle).set_text(cx, &t);
            self.ui.button(cx, active).set_text(cx, &t);
        }
        let copies = draft
            .copies
            .clamp(print_share::KOT_COPIES_MIN, print_share::KOT_COPIES_MAX);
        let weight = field_style.weight;
        let align = field_style.align;
        let item_open = custom_open.is_none() && style_field == print_share::KotStyleField::ItemQty;
        self.ui
            .button(cx, ids!(kot_custom_add))
            .set_text(cx, &s("bill_custom_add"));
        self.ui
            .button(cx, ids!(kot_custom_remove))
            .set_text(cx, &s("bill_custom_remove"));
        self.ui
            .button(cx, ids!(kot_custom_remove))
            .set_visible(cx, custom_open.is_some());
        for (idle, on, selected) in [
            (ids!(kot_kind_original), ids!(kot_kind_original_on), edit_kind == print_share::KotSlipKind::Original),
            (ids!(kot_kind_duplicate), ids!(kot_kind_duplicate_on), edit_kind == print_share::KotSlipKind::Duplicate),
            (ids!(kot_kind_modified), ids!(kot_kind_modified_on), edit_kind == print_share::KotSlipKind::Modified),
            (ids!(kot_kind_cancelled), ids!(kot_kind_cancelled_on), edit_kind == print_share::KotSlipKind::Cancelled),
            (ids!(kot_field_show), ids!(kot_field_show_on), shown),
            (ids!(kot_field_hide), ids!(kot_field_hide_on), !shown),
            (ids!(kot_field_weight_light), ids!(kot_field_weight_light_on), weight == print_share::KotFontWeight::Light),
            (ids!(kot_field_weight_reg), ids!(kot_field_weight_reg_on), weight == print_share::KotFontWeight::Regular),
            (ids!(kot_field_weight_bold), ids!(kot_field_weight_bold_on), weight == print_share::KotFontWeight::Bold),
            (ids!(kot_align_start), ids!(kot_align_start_on), align == print_share::KotAlign::Start),
            (ids!(kot_align_center), ids!(kot_align_center_on), align == print_share::KotAlign::Center),
            (ids!(kot_align_end), ids!(kot_align_end_on), align == print_share::KotAlign::End),
            (ids!(kot_copies_1), ids!(kot_copies_1_on), copies == 1),
            (ids!(kot_copies_2), ids!(kot_copies_2_on), copies == 2),
            (ids!(kot_copies_3), ids!(kot_copies_3_on), copies == 3),
        ] {
            self.paint_choice_pair(cx, idle, on, selected);
        }
        // Copy from: the other three kinds (this kind hides its own button).
        for (id, kind, label) in [
            (ids!(kot_copy_original), print_share::KotSlipKind::Original, "kot_kind_original"),
            (ids!(kot_copy_duplicate), print_share::KotSlipKind::Duplicate, "kot_kind_duplicate"),
            (ids!(kot_copy_modified), print_share::KotSlipKind::Modified, "kot_kind_modified"),
            (ids!(kot_copy_cancelled), print_share::KotSlipKind::Cancelled, "kot_kind_cancelled"),
        ] {
            let btn = self.ui.button(cx, id);
            btn.set_text(cx, &s(label));
            btn.set_visible(cx, kind != edit_kind);
        }
        // The list is the print order; the open line is edited where it sits.
        let line_ids = kot_line_ids(&draft);
        let open_id = if let Some(ref id) = custom_id {
            format!("custom:{id}")
        } else {
            style_field.id().to_string()
        };
        let open_idx = line_ids.iter().position(|id| id == &open_id).unwrap_or(0);
        for i in 0..KOT_PICK_ROWS.len() {
            let before = i < open_idx && i < line_ids.len();
            let after = i > open_idx && i < line_ids.len();
            self.ui.view(cx, KOT_PICK_ROWS[i]).set_visible(cx, before);
            self.ui.view(cx, KOT_AFTER_ROWS[i]).set_visible(cx, after);
            if let Some(oid) = line_ids.get(i) {
                let label = if let Some(c) = draft.custom_line(oid) {
                    kot_custom_line_label(c, &s)
                } else if let Some(field) = print_share::KotStyleField::from_id(oid) {
                    kot_line_label(&draft, edit_kind, field, &s)
                } else {
                    oid.clone()
                };
                for (a, b) in [KOT_PICK_BTNS[i], KOT_AFTER_BTNS[i]] {
                    self.ui.button(cx, a).set_text(cx, &label);
                    self.ui.button(cx, b).set_text(cx, &label);
                    self.paint_choice_pair(cx, a, b, false);
                }
            }
        }
        let open_name = if let Some(ref c) = custom_open {
            let t = c.text.trim();
            if t.is_empty() {
                s("bill_custom_line")
            } else {
                t.chars().take(40).collect()
            }
        } else {
            s(style_field.locale_key())
        };
        self.ui
            .widget(cx, ids!(kot_line_name))
            .set_text(cx, &plain_display_text(&open_name));
        // Only the info lines move; Title stays first and Item / Qty last.
        let moves = kot_line_moves_id(&open_id);
        self.ui.button(cx, ids!(kot_field_up)).set_visible(cx, moves);
        self.ui.button(cx, ids!(kot_field_down)).set_visible(cx, moves);
        // Item / Qty always prints; its switch is Column headers below.
        self.ui
            .view(cx, ids!(kot_vis_chips))
            .set_visible(cx, !item_open || custom_open.is_some());
        let size_field = self.ui.text_input(cx, ids!(kot_field_size_input));
        if !cx.has_key_focus(size_field.area()) {
            size_field.set_text(cx, &field_style.size_em.to_string());
        }
        let title_open = custom_open.is_none() && style_field == print_share::KotStyleField::Title;
        self.ui.view(cx, ids!(kot_title_pieces)).set_visible(cx, title_open);
        self.ui.view(cx, ids!(kot_item_pieces)).set_visible(cx, item_open);
        self.ui
            .view(cx, ids!(kot_custom_value_row))
            .set_visible(cx, custom_open.is_some());
        if let Some(ref c) = custom_open {
            let input = self.ui.text_input(cx, ids!(kot_custom_value));
            input.set_empty_text(cx, s("bill_custom_value_ph"));
            if !cx.has_key_focus(input.area()) {
                input.set_text(cx, &c.text);
            }
        }
        let kind_chrome = draft.kind_styles.get(edit_kind);
        let set_box = |me: &mut Self, cx: &mut Cx, id: &[LiveId], hint: String, val: &str| {
            let input = me.ui.text_input(cx, id);
            input.set_empty_text(cx, hint);
            if !cx.has_key_focus(input.area()) {
                input.set_text(cx, val);
            }
        };
        if title_open {
            set_box(self, cx, ids!(kot_title_input), s("kot_title_ph"), &draft.title);
            set_box(self, cx, ids!(kot_banner_input), s("kot_banner"), &kind_chrome.banner);
        }
        if item_open {
            for (id, key) in [
                (ids!(h_kot_item_header), "bill_item_header"),
                (ids!(h_kot_strike), "kot_strike"),
                (ids!(h_kot_strike_note), "kot_strike_note"),
                (ids!(h_kot_item_cols), "slip_columns"),
            ] {
                self.ui.widget(cx, id).set_text(cx, &plain_display_text(&s(key)));
            }
            self.ui
                .widget(cx, ids!(kot_columns_help))
                .as_info_help()
                .set_tip(&plain_display_text(&s("slip_columns_help")));
            let header = draft.show_item_qty_header;
            for (idle, on, key, selected) in [
                (ids!(kot_item_header_on), ids!(kot_item_header_on_on), "bill_item_piece_on", header),
                (ids!(kot_item_header_off), ids!(kot_item_header_off_on), "bill_item_piece_off", !header),
                (ids!(kot_strike_on), ids!(kot_strike_on_on), "kot_strike_on", kind_chrome.strike_item_name),
                (ids!(kot_strike_off), ids!(kot_strike_off_on), "kot_strike_off", !kind_chrome.strike_item_name),
                (ids!(kot_strike_note_on), ids!(kot_strike_note_on_on), "kot_strike_on", kind_chrome.strike_item_note),
                (ids!(kot_strike_note_off), ids!(kot_strike_note_off_on), "kot_strike_off", !kind_chrome.strike_item_note),
            ] {
                let t = s(key);
                self.ui.button(cx, idle).set_text(cx, &t);
                self.ui.button(cx, on).set_text(cx, &t);
                self.paint_choice_pair(cx, idle, on, selected);
            }
            set_box(self, cx, ids!(kot_item_prefix_input), s("kot_item_prefix"), &kind_chrome.item_prefix);
            set_box(self, cx, ids!(kot_footer_input), s("kot_footer_ph"), &draft.footer);
            // One row per column of this kind: header text, width (mm),
            // start / end space (px), alignment.
            let cols = draft.item_columns(edit_kind);
            for (row, col, name_key, hdr) in [
                (KOT_COL_ROWS[0], cols.item, "bill_item_name", draft.item_header_item.as_str()),
                (KOT_COL_ROWS[1], cols.qty, "bill_item_qty", draft.item_header_qty.as_str()),
            ] {
                self.ui
                    .widget(cx, &[row, id!(col_name)])
                    .set_text(cx, &plain_display_text(&s(name_key)));
                self.ui.view(cx, &[row, id!(col_header_box)]).set_visible(cx, header);
                set_box(self, cx, &[row, id!(col_header)], s("slip_col_header"), hdr);
                let width = if (col.width_mm - col.width_mm.round()).abs() < 0.05 {
                    format!("{}", col.width_mm.round() as i32)
                } else {
                    format!("{:.1}", col.width_mm)
                };
                for (id, key, val) in [
                    (id!(col_width), "slip_col_width", width),
                    (id!(col_start), "slip_col_start", col.start_px.to_string()),
                    (id!(col_end), "slip_col_end", col.end_px.to_string()),
                ] {
                    set_box(self, cx, &[row, id], s(key), &val);
                }
                for (idle, on, a) in [
                    (id!(col_a_start), id!(col_a_start_on), print_share::KotAlign::Start),
                    (id!(col_a_center), id!(col_a_center_on), print_share::KotAlign::Center),
                    (id!(col_a_end), id!(col_a_end_on), print_share::KotAlign::End),
                ] {
                    let t = s(a.locale_key());
                    self.ui.button(cx, &[row, idle]).set_text(cx, &t);
                    self.ui.button(cx, &[row, on]).set_text(cx, &t);
                    self.paint_choice_pair(cx, &[row, idle], &[row, on], col.align == a);
                }
            }
        }
        self.layout_slip_editors(cx);
        let source_face = match &preview_source {
            KotPreviewSource::Sample => s("kot_preview_sample"),
            KotPreviewSource::Ticket(_) => s("kot_preview_ticket"),
        };
        self.ui
            .button(cx, ids!(kot_preview_source_dd))
            .set_text(cx, &source_face);
        let preview = match &preview_source {
            KotPreviewSource::Sample => draft.render_sample_kind(&shop, edit_kind),
            KotPreviewSource::Ticket(id) => {
                let st = app_state().lock().unwrap();
                st.store
                    .as_ref()
                    .and_then(|store| store.format_kot_text_as(id, &draft).ok())
                    .unwrap_or_else(|| draft.render_sample_kind(&shop, edit_kind))
            }
        };
        let (mm, landscape, base) = {
            let st = app_state().lock().unwrap();
            let mm = st.kot_slip_draft.shown_content_width_mm();
            let base = st
                .printer_draft
                .format_for(print_share::RouteKey::Kot { station_id: "" });
            (mm, base.landscape, base)
        };
        let metrics = print_share::metrics_at_content_mm(&base, mm);
        if let Some(mut slip) = self
            .ui
            .widget(cx, ids!(kot_format_preview))
            .borrow_mut::<KotSlipPreview>()
        {
            let paper = app_state().lock().unwrap().preview_paper(true);
            let lay = crate::os_paper::Lay { landscape, place: base.place };
            slip.show_slip_on(cx, &preview, &metrics, lay, field_style.size_em as f32, &paper);
        }
        self.ui
            .view(cx, ids!(kot_format_preview_card))
            .redraw(cx);
    }

    /// A column editor field that changed to a number (mm or px). Blank or
    /// half-typed text is left alone until it reads as a number.
    fn slip_column_input(&mut self, cx: &mut Cx, actions: &Actions, id: &[LiveId]) -> Option<f32> {
        let raw = self.ui.text_input(cx, id).changed(actions)?;
        raw.trim().replace(',', ".").parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0.0)
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

    /// Open a chrome peer the way its sidebar / bottom-bar item does — rights
    /// checked, its data refreshed. Settings' sidebar rows call it too
    /// (owner, 2026-11-14: a row taps through to its screen).
    fn open_nav_tab(&mut self, cx: &mut Cx, tab: NavTab) {
        match Some(tab) {
            Some(NavTab::Books) => {
                let allowed = {
                    let st = app_state().lock().unwrap();
                    match st.session.as_ref() {
                        Some(s) => s.is_owner() || s.can_access_sales() || s.can_access_items(),
                        None => false,
                    }
                };
                if !allowed {
                    self.set_status(cx, "Not allowed");
                } else {
                    // Drop leftover focus so the old Sell face cannot look selected.
                    cx.set_key_focus(Area::Empty);
                    self.go_peer(cx, Screen::Books);
                }
            }
            Some(NavTab::Sell) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Pos, |s| s.can_access_sell());
            }
            Some(NavTab::Drafts) => {
                cx.set_key_focus(Area::Empty);
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.refresh_counter_drafts()
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer_if_allowed(cx, Screen::Drafts, |s| s.can_access_sell());
                }
            }
            Some(NavTab::Buy) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Buy, |s| {
                    s.can_access_inventory() || s.can_access_sales() || s.is_owner()
                });
            }
            Some(NavTab::Menu) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Menu, |s| s.can_access_items());
            }
            Some(NavTab::Floor) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Floor, |s| {
                    s.can_access_floor() || s.is_owner()
                });
            }
            Some(NavTab::Kitchen) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Kitchen, |s| {
                    s.can_access_kitchen() || s.is_owner()
                });
            }
            Some(NavTab::Sales) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Sales, |s| s.can_access_sales());
            }
            Some(NavTab::Credit) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Credit, |s| s.can_access_credit());
            }
            Some(NavTab::Team) => {
                // Team & PINs: manage staff, or only one's own PIN.
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Team, |s| {
                    s.can_manage_team() || s.can_change_own_credentials()
                });
            }
            Some(t) if nav_tab_settings_panel(t).is_some() => {
                // A Settings section: the Settings screen, then that section.
                cx.set_key_focus(Area::Empty);
                let allowed = app_state()
                    .lock()
                    .unwrap()
                    .session
                    .as_ref()
                    .is_some_and(|s| s.can_access_settings());
                if allowed {
                    app_state().lock().unwrap().settings_panel_after_go = nav_tab_settings_panel(t);
                }
                self.go_peer_if_allowed(cx, Screen::Settings, |s| s.can_access_settings());
            }
            Some(NavTab::Settings) => {
                cx.set_key_focus(Area::Empty);
                self.go_peer_if_allowed(cx, Screen::Settings, |s| s.can_open_settings_shell());
            }
            Some(NavTab::DayClose) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_sales() {
                        return Err("Not allowed".into());
                    }
                    st.refresh_day_close_text()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::DayClose);
                }
            }
            Some(NavTab::Reports) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.is_owner() && !actor.can_access_sales() {
                        return Err("Not allowed".into());
                    }
                    st.refresh_reports_facet()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::Reports);
                }
            }
            Some(NavTab::Inventory) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.can_access_inventory() && !actor.is_owner() {
                        return Err("Not allowed for your role".into());
                    }
                    st.refresh_inventory_text()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::Inventory);
                }
            }
            Some(NavTab::Branches) => {
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let actor = st.actor()?;
                    if !actor.can_access_branches() && !actor.is_owner() {
                        return Err("Not allowed for your role".into());
                    }
                    st.refresh_branches_text()?;
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.go_peer(cx, Screen::Branches);
                }
            }
            Some(NavTab::Lock) => {
                self.clear_undo_snackbar(cx);
                let dest = {
                    let mut st = app_state().lock().unwrap();
                    if let Some(store) = st.store.as_ref() {
                        let _ = store.set_active_session_staff_id(None);
                    }
                    st.session = None;
                    st.clear_team_edit_selection();
                    if st.suggest_open == Some(SuggestField::TeamKotStations) {
                        st.suggest_open = None;
                    }
                    st.pin_buffer.clear();
                    st.status = "Locked".into();
                    if st.must_confirm_device_role {
                        Screen::DeviceSetup
                    } else {
                        Screen::Lock
                    }
                };
                self.go_peer(cx, dest);
            }
            Some(_) | None => {}
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

/// A screen switch that took at least this long (tap → first draw) shows the
/// loading screen first next time.
const NAV_VEIL_MS: f64 = 1000.0;

/// Last measured switch cost per screen (ms).
fn nav_costs() -> &'static Mutex<Vec<(Screen, f64)>> {
    static COSTS: OnceLock<Mutex<Vec<(Screen, f64)>>> = OnceLock::new();
    COSTS.get_or_init(|| Mutex::new(Vec::new()))
}

/// The switch being timed: target screen and when it started.
fn nav_measure() -> &'static Mutex<Option<(Screen, Instant)>> {
    static M: Mutex<Option<(Screen, Instant)>> = Mutex::new(None);
    &M
}

/// A deferred switch waiting for the loading screen's frame.
fn nav_veil_pending() -> &'static Mutex<Option<(NextFrame, Screen, bool)>> {
    static P: Mutex<Option<(NextFrame, Screen, bool)>> = Mutex::new(None);
    &P
}

fn nav_was_slow(screen: Screen) -> bool {
    // `KUMKAJ_NAV_VEIL_MS` lowers the bar to test the loading screen on a
    // fast machine.
    static LIMIT: OnceLock<f64> = OnceLock::new();
    let limit = *LIMIT.get_or_init(|| {
        std::env::var("KUMKAJ_NAV_VEIL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(NAV_VEIL_MS)
    });
    let last = nav_costs()
        .lock()
        .unwrap()
        .iter()
        .find(|(s, _)| *s == screen)
        .map(|(_, ms)| *ms);
    match last {
        Some(ms) if ms >= limit => {
            makepad_widgets::log!("Kumkaj nav: loading screen for {screen:?} (last switch {ms:.0} ms)");
            true
        }
        _ => false,
    }
}

/// After a draw: if a screen switch was being timed, record what it cost.
fn note_nav_draw() {
    let Some((screen, started)) = nav_measure().lock().unwrap().take() else {
        return;
    };
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut costs = nav_costs().lock().unwrap();
    match costs.iter_mut().find(|(s, _)| *s == screen) {
        Some(entry) => entry.1 = ms,
        None => costs.push((screen, ms)),
    }
}

/// The 3 × 3 grid's buttons beside the Bill or KOT preview, by row (top,
/// middle, bottom) and column (left, centre, right): idle and picked.
fn place_cells(kot: bool) -> [[(&'static [LiveId], &'static [LiveId]); 3]; 3] {
    if kot {
        [
            [
                (ids!(kot_place_0_0), ids!(kot_place_0_0_on)),
                (ids!(kot_place_0_1), ids!(kot_place_0_1_on)),
                (ids!(kot_place_0_2), ids!(kot_place_0_2_on)),
            ],
            [
                (ids!(kot_place_1_0), ids!(kot_place_1_0_on)),
                (ids!(kot_place_1_1), ids!(kot_place_1_1_on)),
                (ids!(kot_place_1_2), ids!(kot_place_1_2_on)),
            ],
            [
                (ids!(kot_place_2_0), ids!(kot_place_2_0_on)),
                (ids!(kot_place_2_1), ids!(kot_place_2_1_on)),
                (ids!(kot_place_2_2), ids!(kot_place_2_2_on)),
            ],
        ]
    } else {
        [
            [
                (ids!(bill_place_0_0), ids!(bill_place_0_0_on)),
                (ids!(bill_place_0_1), ids!(bill_place_0_1_on)),
                (ids!(bill_place_0_2), ids!(bill_place_0_2_on)),
            ],
            [
                (ids!(bill_place_1_0), ids!(bill_place_1_0_on)),
                (ids!(bill_place_1_1), ids!(bill_place_1_1_on)),
                (ids!(bill_place_1_2), ids!(bill_place_1_2_on)),
            ],
            [
                (ids!(bill_place_2_0), ids!(bill_place_2_0_on)),
                (ids!(bill_place_2_1), ids!(bill_place_2_1_on)),
                (ids!(bill_place_2_2), ids!(bill_place_2_2_on)),
            ],
        ]
    }
}

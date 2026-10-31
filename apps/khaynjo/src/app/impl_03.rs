impl App {
fn lift_footer_above_system_bar(&mut self, cx: &mut Cx) {
        let bottom = 8.0 + cx.display_context.safe_area_insets.bottom;
        let nav = self.ui.view(cx, ids!(bottom_nav));
        let changed = if let Some(mut inner) = nav.borrow_mut() {
            if (inner.layout.padding.bottom - bottom).abs() > 0.5 {
                inner.layout.padding.bottom = bottom;
                true
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            nav.redraw(cx);
        }
    }

    /// Makepad caret visibility is `(1 - blink) * focus` from TextInput animators.
    /// Key focus can survive hide/show / list rebind without a new `Hit::KeyFocus`,
    /// so the field stays typable but the caret never blinks. Repair every field
    /// that currently holds key focus — not just one screen's inputs.
    fn repair_focused_carets(&mut self, cx: &mut Cx) {
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
                field.take_key_focus(cx);
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
                        field.take_key_focus(cx);
                    }
                }
            }
        }
        self.paint_chrome(cx);
        self.paint_static_texts(cx);
        self.apply_text_alignment(cx);
        self.paint_choice_state(cx);
        // Reports title after paint_chrome so locale strings do not overwrite it.
        {
            let (screen, lang) = {
                let st = app_state().lock().unwrap();
                (st.screen, st.locale.language.clone())
            };
            if screen == Screen::Reports {
                let help = self.ui.widget(cx, ids!(reports_help)).as_info_help();
                help.set_title(cx, &locale::t(&lang, "reports_facet_title"));
                help.set_nav_icon(cx, NavTab::Reports);
                help.set_tip(&plain_display_text(&locale::t(&lang, "reports_facet_help")));
            }
        }
        // Re-apply after chrome labels — hot-reload / focus must not leave the
        // wrong bottom-nav peer lit (e.g. Sell while Buy is showing).
        let screen = app_state().lock().unwrap().screen;
        self.paint_nav_tabs(cx, screen);
        self.sync_dev_size(cx);
    }

    /// Cut Makepad 2 hover/focus animators so a hidden or deselected face cannot
    /// keep looking "pressed/selected" after navigation.
    fn clear_button_animators(&self, cx: &mut Cx, path: &[LiveId]) {
        let btn = self.ui.button(cx, path);
        btn.reset_hover(cx);
        if let Some(mut inner) = btn.borrow_mut() {
            inner.animator_cut(cx, ids!(hover.off));
            inner.animator_cut(cx, ids!(focus.off));
        }
    }

    /// Selected choice = active face visible; idle face visible otherwise.
    /// Makepad 2 Button has no selected bit — visibility swap is the highlight,
    /// and animators are cleared so previous faces never stay lit.
    fn paint_choice_pair(
        &mut self,
        cx: &mut Cx,
        idle: &[LiveId],
        active: &[LiveId],
        selected: bool,
    ) {
        self.clear_button_animators(cx, idle);
        self.clear_button_animators(cx, active);
        self.ui.button(cx, idle).set_visible(cx, !selected);
        self.ui.button(cx, active).set_visible(cx, selected);
    }

    /// Expense tender parts from the form: the whole amount by the
    /// first tender, or split when a "second way" part is typed (the
    /// first tender takes the remainder).

    fn choice_chip_clicked(
        &self,
        cx: &Cx,
        actions: &Actions,
        idle: &[LiveId],
        active: &[LiveId],
    ) -> bool {
        self.ui.button(cx, idle).clicked(actions) || self.ui.button(cx, active).clicked(actions)
    }

    /// Which nav tab a click belongs to, honouring the user's arrangement:
    /// slots are positions, so a click on slot N means `order[N]` — on
    /// either surface (bottom bar or rail).
    fn nav_tab_clicked(&mut self, cx: &mut Cx, actions: &Actions) -> Option<NavTab> {
        if *nav_suppress_click().lock().unwrap() {
            *nav_suppress_click().lock().unwrap() = false;
            return None;
        }
        let order = current_nav_chrome_order();
        let bottom = self.bottom_nav_pairs();
        let rail = self.rail_nav_pairs();
        for slot in 0..order.len() {
            if self.choice_chip_clicked(cx, actions, bottom[slot].0, bottom[slot].1)
                || self.choice_chip_clicked(cx, actions, rail[slot].0, rail[slot].1)
            {
                return Some(order[slot]);
            }
        }
        None
    }

    /// Bare-minimal **None** (`custom`) when the shop never picked a template.
    fn ensure_default_template(st: &mut AppState) -> Result<(), String> {
        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
        let stored = store.entity_template_id().unwrap_or_default();
        let mem = st.entity_template_id.trim();
        let stored = stored.trim();
        let selected = if !mem.is_empty() && mem != "custom" {
            mem
        } else if !stored.is_empty() && stored != "custom" {
            stored
        } else {
            "custom"
        };
        if selected != "custom" {
            if mem != selected {
                st.entity_template_id = selected.to_string();
            }
            return Ok(());
        }
        let tmpl = store.apply_entity_template("khaynjo_restaurant")?;
        st.entity_template_id = tmpl.id.to_string();
        st.venue_mode = VenueMode::Restaurant;
        Ok(())
    }

    fn paint_choice_state(&mut self, cx: &mut Cx) {
        let (
            money_lane,
            draft_role,
            buy_on_account,
            follow_platform,
            onboard_step,
            books_report_kind,
            device_role,
            picking_server,
        ) = {
            let st = app_state().lock().unwrap();
            (
                st.money_lane,
                st.draft_staff_role,
                st.buy_on_account,
                st.language_follow_platform,
                st.onboard_step,
                st.books_report_kind,
                st.device_role,
                st.picking_server,
            )
        };
        self.paint_choice_pair(
            cx,
            ids!(reports_today),
            ids!(reports_today_on),
            books_report_kind == BooksReportKind::Today,
        );
        self.paint_choice_pair(
            cx,
            ids!(reports_month),
            ids!(reports_month_on),
            books_report_kind == BooksReportKind::Month,
        );
        self.paint_choice_pair(
            cx,
            ids!(reports_aging),
            ids!(reports_aging_on),
            books_report_kind == BooksReportKind::Aging,
        );
        self.paint_choice_pair(
            cx,
            ids!(reports_balance_sheet),
            ids!(reports_balance_sheet_on),
            books_report_kind == BooksReportKind::BalanceSheet,
        );
        if picking_server {
            for path in [
                ids!(device_role_server),
                ids!(device_role_server_on),
                ids!(device_role_client),
                ids!(device_role_client_on),
            ] {
                self.ui.button(cx, path).set_visible(cx, false);
            }
        } else {
            self.paint_choice_pair(
                cx,
                ids!(device_role_server),
                ids!(device_role_server_on),
                device_role == DeviceRole::Server,
            );
            self.paint_choice_pair(
                cx,
                ids!(device_role_client),
                ids!(device_role_client_on),
                device_role == DeviceRole::Client,
            );
        }
        self.paint_choice_pair(
            cx,
            ids!(onboard_fresh),
            ids!(onboard_fresh_on),
            onboard_step == 1,
        );
        self.paint_choice_pair(
            cx,
            ids!(onboard_book),
            ids!(onboard_book_on),
            onboard_step == 2,
        );
        self.paint_choice_pair(
            cx,
            ids!(onboard_template),
            ids!(onboard_template_on),
            onboard_step == 3,
        );
        self.ui
            .view(cx, ids!(onboard_fresh_panel))
            .set_visible(cx, onboard_step == 1);
        self.ui
            .view(cx, ids!(onboard_book_panel))
            .set_visible(cx, onboard_step == 2);
        self.paint_choice_pair(
            cx,
            ids!(lane_declared),
            ids!(lane_declared_on),
            matches!(money_lane, MoneyLane::Declared),
        );
        self.paint_choice_pair(
            cx,
            ids!(lane_owner),
            ids!(lane_owner_on),
            matches!(money_lane, MoneyLane::Owner),
        );
        self.paint_choice_pair(cx, ids!(buy_paid), ids!(buy_paid_on), !buy_on_account);
        self.paint_choice_pair(
            cx,
            ids!(lang_setup_platform),
            ids!(lang_setup_platform_on),
            follow_platform,
        );
        self.paint_choice_pair(
            cx,
            ids!(lang_setup_user),
            ids!(lang_setup_user_on),
            !follow_platform,
        );
        self.paint_choice_pair(
            cx,
            ids!(settings_lang_platform),
            ids!(settings_lang_platform_on),
            follow_platform,
        );
        self.paint_choice_pair(
            cx,
            ids!(settings_lang_user),
            ids!(settings_lang_user_on),
            !follow_platform,
        );
        // Language catalogue only when the user picks a fixed language.
        self.ui
            .widget(cx, ids!(setup_lang_list))
            .set_visible(cx, !follow_platform);
        self.ui
            .widget(cx, ids!(language_list))
            .set_visible(cx, !follow_platform);
        self.paint_choice_pair(
            cx,
            ids!(buy_on_account),
            ids!(buy_on_account_on),
            buy_on_account,
        );
        for (idle, on, role) in [
            (ids!(staff_role_waiter), ids!(staff_role_waiter_on), StaffRole::Waiter),
            (ids!(staff_role_kitchen), ids!(staff_role_kitchen_on), StaffRole::Kitchen),
        ] {
            self.paint_choice_pair(cx, idle, on, draft_role == role);
        }
        self.paint_choice_pair(
            cx,
            ids!(staff_role_cashier),
            ids!(staff_role_cashier_on),
            draft_role == StaffRole::Cashier,
        );
        self.paint_choice_pair(
            cx,
            ids!(staff_role_manager),
            ids!(staff_role_manager_on),
            draft_role == StaffRole::Manager,
        );
        // Party-form kind chips: highlight whatever the kind field currently
        // holds (a picked party's saved kind, or the chip last tapped).
        let kind_norm = domain::normalize_party_roles(
            &self.ui.text_input(cx, ids!(credit_add_kind)).text(),
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_customer),
            ids!(credit_kind_customer_on),
            kind_norm == "Customer",
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_both),
            ids!(credit_kind_both_on),
            kind_norm == "Customer, Supplier",
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_supplier),
            ids!(credit_kind_supplier_on),
            kind_norm == "Supplier",
        );
        // Keep lane labels current on both faces.
        let (declared, owner) = {
            let st = app_state().lock().unwrap();
            (st.lane_declared_label(), st.lane_owner_label())
        };
        for path in [ids!(lane_declared), ids!(lane_declared_on)] {
            self.ui.button(cx, path).set_text(cx, &declared);
        }
        for path in [ids!(lane_owner), ids!(lane_owner_on)] {
            self.ui.button(cx, path).set_text(cx, &owner);
        }
    }

    /// Every static label and button. List rows set their own text while drawing.

    /// Static label / placeholder pass: every fixed caption in the DSL
    /// gets its localized text here (user-typed data is never touched).
    /// Right-align static text while a right-to-left language is active;
    /// restores each label's designed alignment for left-to-right ones.
    /// Makepad shapes and bidi-orders the glyphs itself — this is the
    /// app's half of RTL support (see `locale::is_rtl`).
    fn apply_text_alignment(&mut self, cx: &mut Cx) {
        let rtl = locale::is_rtl(&app_state().lock().unwrap().locale.language);
        self.align_static_texts(cx, rtl);
        self.align_static_inputs(cx, rtl);
    }

    /// Fill a [`ReportText`] card with rows built in code.
    fn set_report_rows(&mut self, cx: &mut Cx, id: &[LiveId], rows: Vec<ReportRow>) {
        if let Some(mut report) = self.ui.widget(cx, id).borrow_mut::<ReportText>() {
            report.set_rows(cx, rows);
        }
    }

    /// Notes & attachments of the picked record: one row each in a card,
    /// or the hint while there are none.
    fn show_attachments(&mut self, cx: &mut Cx, hint: &[LiveId], list: &[LiveId], summary: &str, empty: &str) {
        let rows: Vec<ReportRow> = summary
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && *l != "Attachments:")
            .map(|l| ReportRow::Line(l.to_string()))
            .collect();
        self.ui.widget(cx, hint).set_text(cx, &plain_display_text(empty));
        self.ui.widget(cx, hint).set_visible(cx, rows.is_empty());
        self.set_report_rows(cx, list, rows);
    }

    /// Settings → Printers: what the printer search says, and the printers
    /// it found as a list under it.
    fn show_found_printers(&mut self, cx: &mut Cx) {
        let (line, rows) = {
            let st = app_state().lock().unwrap();
            printers_found_rows(|k| st.t(k))
        };
        self.ui
            .widget(cx, ids!(printers_found_label))
            .set_text(cx, &plain_display_text(&line));
        self.set_report_rows(cx, ids!(printers_found_list), rows);
    }

    fn align_label(&mut self, cx: &mut Cx, id: &[LiveId], rtl: bool) {
        let w = self.ui.widget(cx, id);
        if let Some(mut label) = w.borrow_mut::<Label>() {
            let key = format!("{id:?}");
            if rtl {
                ORIG_TEXT_ALIGN
                    .lock()
                    .unwrap()
                    .entry(key)
                    .or_insert(label.align.x);
                label.align.x = 1.0;
            } else if let Some(x) = ORIG_TEXT_ALIGN.lock().unwrap().get(&key).copied() {
                label.align.x = x;
            }
        } else if let Some(mut label) = w.borrow_mut::<WrapLabel>() {
            label.set_text_align_x(if rtl { 1.0 } else { 0.0 });
        }
    }

    fn align_input(&mut self, cx: &mut Cx, id: &[LiveId], rtl: bool) {
        let w = self.ui.widget(cx, id);
        if let Some(mut input) = w.borrow_mut::<TextInput>() {
            let key = format!("input:{id:?}");
            if rtl {
                ORIG_TEXT_ALIGN
                    .lock()
                    .unwrap()
                    .entry(key)
                    .or_insert(input.label_align.x);
                input.label_align.x = 1.0;
            } else if let Some(x) = ORIG_TEXT_ALIGN.lock().unwrap().get(&key).copied() {
                input.label_align.x = x;
            }
        }
    }

    fn align_static_texts(&mut self, cx: &mut Cx, rtl: bool) {
        self.align_label(cx, ids!(list), rtl);
        self.align_label(cx, ids!(thumb_initials), rtl);
        self.align_label(cx, ids!(name), rtl);
        self.align_label(cx, ids!(price), rtl);
        self.align_label(cx, ids!(stock), rtl);
        self.align_label(cx, ids!(status), rtl);
        self.align_label(cx, ids!(bill_req), rtl);
        self.align_label(cx, ids!(label), rtl);
        self.align_label(cx, ids!(photo), rtl);
        self.align_label(cx, ids!(title), rtl);
        self.align_label(cx, ids!(summary), rtl);
        self.align_label(cx, ids!(state), rtl);
        self.align_label(cx, ids!(qty_label), rtl);
        self.align_label(cx, ids!(sublabel), rtl);
        self.align_label(cx, ids!(pin_status), rtl);
        self.align_label(cx, ids!(device_setup_status), rtl);
        self.align_label(cx, ids!(receipt_label), rtl);
        self.align_label(cx, ids!(today_label), rtl);
        self.align_label(cx, ids!(session_label), rtl);
        self.align_input(cx, ids!(lock_pin_input), rtl);
        self.align_label(cx, ids!(lock_staff_label), rtl);
        self.align_label(cx, ids!(db_path_label), rtl);
        self.align_label(cx, ids!(audit_label), rtl);
        self.align_label(cx, ids!(table_chip), rtl);
        self.align_label(cx, ids!(session_chip), rtl);
        self.align_label(cx, ids!(kitchen_empty), rtl);
        self.align_label(cx, ids!(staff_label), rtl);
        self.align_label(cx, ids!(locale_label), rtl);
        self.align_label(cx, ids!(fiscal_label), rtl);
        self.align_label(cx, ids!(country_hint), rtl);
        self.align_label(cx, ids!(tax_estimate_label), rtl);
        self.align_label(cx, ids!(empty_catalog), rtl);
        self.align_label(cx, ids!(menu_empty), rtl);
        self.align_label(cx, ids!(menu_unpriced_note), rtl);
        self.align_label(cx, ids!(catalog_cta), rtl);
        self.align_label(cx, ids!(product_list), rtl);
        self.align_label(cx, ids!(product_list_menu), rtl);
        self.align_label(cx, ids!(h_removed_items), rtl);
        self.align_label(cx, ids!(removed_help), rtl);
        self.align_label(cx, ids!(removed_list), rtl);
        self.align_label(cx, ids!(menu_status), rtl);
        self.align_label(cx, ids!(cart_empty_hint), rtl);
        self.align_label(cx, ids!(cart_list), rtl);
        self.align_label(cx, ids!(cart_natural_label), rtl);
        self.align_label(cx, ids!(cart_applied_label), rtl);
        self.align_label(cx, ids!(sticky_total), rtl);
        self.align_label(cx, ids!(kot_dest_help), rtl);
        self.align_label(cx, ids!(cart_charge_caption), rtl);
        self.align_label(cx, ids!(cart_round_hint), rtl);
        self.align_label(cx, ids!(undo_snack_label), rtl);
        self.align_label(cx, ids!(snack_label_info), rtl);
        self.align_label(cx, ids!(snack_label_ok), rtl);
        self.align_label(cx, ids!(snack_label_warn), rtl);
        self.align_label(cx, ids!(snack_label_err), rtl);
        self.align_label(cx, ids!(party_selected_label), rtl);
        self.align_label(cx, ids!(credit_chip), rtl);
        self.align_label(cx, ids!(credit_summary_label), rtl);
        self.align_label(cx, ids!(customers_empty_hint), rtl);
        self.align_label(cx, ids!(approval_hint), rtl);
        self.align_label(cx, ids!(sale_selected_label), rtl);
        self.align_label(cx, ids!(sale_attach_hint), rtl);
        self.align_label(cx, ids!(party_attach_hint), rtl);
        self.align_label(cx, ids!(expense_attach_hint), rtl);
        self.align_label(cx, ids!(amend_natural_label), rtl);
        self.align_label(cx, ids!(amend_applied_label), rtl);
        self.align_label(cx, ids!(amend_round_hint), rtl);
        self.align_label(cx, ids!(h_item_photo), rtl);
        self.align_label(cx, ids!(image_status), rtl);
        self.align_label(cx, ids!(draft_preview), rtl);
        self.align_label(cx, ids!(server_pair_label), rtl);
        self.align_label(cx, ids!(pairing_code_value), rtl);
        self.align_label(cx, ids!(settings_tmpl_label), rtl);
        self.align_label(cx, ids!(units_label), rtl);
        self.align_label(cx, ids!(h_onboard_layout), rtl);
        self.align_label(cx, ids!(onboard_tmpl_help), rtl);
        self.align_label(cx, ids!(day_close_label), rtl);
        self.align_label(cx, ids!(books_till_value), rtl);
        self.align_label(cx, ids!(reports_label), rtl);
        self.align_label(cx, ids!(inventory_label), rtl);
        self.align_label(cx, ids!(branches_label), rtl);
        self.align_label(cx, ids!(onboard_status), rtl);
        self.align_label(cx, ids!(client_pair_status), rtl);
        self.align_label(cx, ids!(client_queue_label), rtl);
        self.align_label(cx, ids!(reports_help), rtl);
        self.align_label(cx, ids!(setup_lang_list), rtl);
        self.align_label(cx, ids!(language_list), rtl);
        self.align_label(cx, ids!(h_unit_at_cap), rtl);
        self.align_label(cx, ids!(h_applied), rtl);
        self.align_label(cx, ids!(h_local_badge), rtl);
        self.align_label(cx, ids!(h_offline_badge), rtl);
        self.align_label(cx, ids!(h_local_badge_2), rtl);
        self.align_label(cx, ids!(h_counter_locked), rtl);
        self.align_label(cx, ids!(h_shop), rtl);
        self.align_label(cx, ids!(h_who_unlocking), rtl);
        self.align_label(cx, ids!(h_setup_app_khaynjo), rtl);
        self.align_label(cx, ids!(h_shop_logo_optional), rtl);
        self.align_label(cx, ids!(h_product_locked_khaynjo), rtl);
        self.align_label(cx, ids!(h_how_start), rtl);
        self.align_label(cx, ids!(h_fresh_book_khaynjo), rtl);
        self.align_label(cx, ids!(h_fresh_selected), rtl);
        self.align_label(cx, ids!(h_later_country_pack), rtl);
        self.align_label(cx, ids!(h_connect_to_server), rtl);
        self.align_label(cx, ids!(h_money_in_sales), rtl);
        self.align_label(cx, ids!(h_purchases_cogs), rtl);
        self.align_label(cx, ids!(h_expenses_word), rtl);
        self.align_label(cx, ids!(h_gross_word), rtl);
        self.align_label(cx, ids!(h_net_word), rtl);
        self.align_label(cx, ids!(h_cash), rtl);
        self.align_label(cx, ids!(h_card), rtl);
        self.align_label(cx, ids!(h_they_owe_us), rtl);
        self.align_label(cx, ids!(h_we_owe_them), rtl);
        self.align_label(cx, ids!(h_bank_balance), rtl);
        self.align_label(cx, ids!(h_cash_in_hand), rtl);
        self.align_label(cx, ids!(h_bank_net_today), rtl);
        self.align_label(cx, ids!(h_local_badge_3), rtl);
        self.align_label(cx, ids!(h_actual_total_cap), rtl);
        self.align_label(cx, ids!(h_applied_total_cap), rtl);
        self.align_label(cx, ids!(h_mix_type_hint_khaynjo), rtl);
        self.align_label(cx, ids!(sales_help), rtl);
        self.align_label(cx, ids!(h_rounded_total), rtl);
        self.align_label(cx, ids!(h_account_kind), rtl);
        self.align_label(cx, ids!(team_help), rtl);
        self.align_label(cx, ids!(h_permission_preset), rtl);
        self.align_label(cx, ids!(h_preset_help_khaynjo), rtl);
        self.align_label(cx, ids!(h_optional_login_help), rtl);
        self.align_label(cx, ids!(h_feature_access_help), rtl);
        self.align_label(cx, ids!(h_change_my_pin_heading), rtl);
        self.align_label(cx, ids!(h_duress_heading), rtl);
        self.align_label(cx, ids!(h_duress_help), rtl);
        self.align_label(cx, ids!(duress_status), rtl);
        self.align_label(cx, ids!(h_country_pack_help), rtl);
        self.align_label(cx, ids!(h_language_setup_hint), rtl);
        self.align_label(cx, ids!(h_currency_use_help), rtl);
        self.align_label(cx, ids!(h_product_only_khaynjo), rtl);
        self.align_label(cx, ids!(h_brand_shown_help), rtl);
        self.align_label(cx, ids!(h_units_help), rtl);
        self.align_label(cx, ids!(h_units_picker_help), rtl);
        self.align_label(cx, ids!(h_kot_print_help_khaynjo), rtl);
        self.align_label(cx, ids!(h_restore_help), rtl);
        self.align_label(cx, ids!(day_close_help), rtl);
        self.align_label(cx, ids!(inventory_help), rtl);
        self.align_label(cx, ids!(branches_help), rtl);
        self.align_label(cx, ids!(h_book_add_dish), rtl);
        self.align_label(cx, ids!(h_book_opening_credit), rtl);
        self.align_label(cx, ids!(h_caption_today), rtl);
        self.align_label(cx, ids!(h_caption_bills), rtl);
        self.align_label(cx, ids!(h_caption_payments_today), rtl);
        self.align_label(cx, ids!(h_caption_outstanding), rtl);
        self.align_label(cx, ids!(h_caption_bank), rtl);
        self.align_label(cx, ids!(h_your_categories), rtl);
        self.align_label(cx, ids!(h_goods_received), rtl);
        self.align_label(cx, ids!(h_expense_heading), rtl);
        self.align_label(cx, ids!(h_paid_by), rtl);
        self.align_label(cx, ids!(h_second_way), rtl);
        self.align_label(cx, ids!(h_notes_attach_sale), rtl);
        self.align_label(cx, ids!(h_credit_limit_heading), rtl);
        self.align_label(cx, ids!(h_pay_selected_heading), rtl);
        self.align_label(cx, ids!(h_write_off_heading), rtl);
        self.align_label(cx, ids!(h_pay_ap_heading), rtl);
        self.align_label(cx, ids!(h_notes_attach_party), rtl);
        self.align_label(cx, ids!(h_country_tax), rtl);
        self.align_label(cx, ids!(h_language), rtl);
        self.align_label(cx, ids!(h_currency), rtl);
        self.align_label(cx, ids!(h_extra_tills_heading), rtl);
        self.align_label(cx, ids!(h_other_devices_help_heading), rtl);
        self.align_label(cx, ids!(h_other_devices_help), rtl);
        self.align_label(cx, ids!(h_start_template), rtl);
        self.align_label(cx, ids!(h_your_brand), rtl);
        self.align_label(cx, ids!(h_units_heading), rtl);
        self.align_label(cx, ids!(h_kitchen_printing), rtl);
        self.align_label(cx, ids!(h_printers), rtl);
        self.align_label(cx, ids!(h_printers_help), rtl);
        self.align_label(cx, ids!(route_editing_label), rtl);
        self.align_label(cx, ids!(printers_found_label), rtl);
        self.align_input(cx, ids!(printer_label_input), rtl);
        self.align_input(cx, ids!(printer_target_input), rtl);
        self.align_input(cx, ids!(route_pick_input), rtl);
        self.align_label(cx, ids!(h_backup), rtl);
        self.align_label(cx, ids!(h_diag), rtl);
        self.align_label(cx, ids!(diag_privacy_help), rtl);
        self.align_label(cx, ids!(diag_status_label), rtl);
        self.align_label(cx, ids!(h_diag_level), rtl);
        self.align_label(cx, ids!(diag_send_help), rtl);
        self.align_label(cx, ids!(h_drawer_count), rtl);
        self.align_label(cx, ids!(h_paid_by_2), rtl);
        self.align_label(cx, ids!(h_second_way_2), rtl);
        self.align_label(cx, ids!(h_attachments_last_expense), rtl);
        self.align_label(cx, ids!(h_bank_reconciliation), rtl);
        self.align_label(cx, ids!(h_spreadsheet_csv), rtl);
        self.align_label(cx, ids!(books_help), rtl);
        self.align_label(cx, ids!(sell_help), rtl);
        self.align_label(cx, ids!(buy_help), rtl);
        self.align_label(cx, ids!(money_lane_label), rtl);
        self.align_label(cx, ids!(menu_mode_label), rtl);
        self.align_label(cx, ids!(items_help_label), rtl);
        self.align_label(cx, ids!(credit_help), rtl);
        self.align_label(cx, ids!(settings_hub_help), rtl);
        self.align_label(cx, ids!(sidebar_settings_help), rtl);
        self.align_label(cx, ids!(settings_title), rtl);
        self.align_label(cx, ids!(reports_bank_heading), rtl);
        self.align_label(cx, ids!(device_setup_banner), rtl);
        self.align_label(cx, ids!(device_use_question), rtl);
        self.align_label(cx, ids!(setup_lang_selected), rtl);
        self.align_label(cx, ids!(settings_lang_selected), rtl);
        self.align_label(cx, ids!(brand_title), rtl);
        self.align_label(cx, ids!(h_venue_icon), rtl);
        self.align_label(cx, ids!(venue_icon_status), rtl);
        self.align_label(cx, ids!(onboard_icon_status), rtl);
        self.align_label(cx, ids!(print_stamp_status), rtl);
        self.align_label(cx, ids!(payment_qr_status), rtl);
        self.align_label(cx, ids!(h_stamp), rtl);
        self.align_label(cx, ids!(h_payment_qr), rtl);
        self.align_label(cx, ids!(dev_size_label), rtl);
        self.align_label(cx, ids!(nav_drag_ghost), rtl);
        self.align_label(cx, ids!(nav_drag_ghost_label), rtl);
        self.align_label(cx, ids!(setup_server_list), rtl);
        self.align_label(cx, ids!(floor_list), rtl);
        self.align_label(cx, ids!(kitchen_list), rtl);
        self.align_label(cx, ids!(category_list), rtl);
        self.align_label(cx, ids!(sale_list), rtl);
        self.align_label(cx, ids!(amend_cart_list), rtl);
        self.align_label(cx, ids!(backup_path_label), rtl);
    }

    fn align_static_inputs(&mut self, cx: &mut Cx, rtl: bool) {
        self.align_input(cx, ids!(unit_price), rtl);
        self.align_input(cx, ids!(line_total), rtl);
        self.align_input(cx, ids!(note), rtl);
        self.align_input(cx, ids!(cart_total_input), rtl);
        self.align_input(cx, ids!(approval_pin_input), rtl);
        self.align_input(cx, ids!(amend_total_input), rtl);
        self.align_input(cx, ids!(menu_name_input), rtl);
        for id in [
            ids!(move_target_input),
            ids!(move_pin_input),
            ids!(line_reason_input),
            ids!(line_qty_input),
            ids!(line_target_input),
            ids!(line_pin_input),
            ids!(menu_search_input),
            ids!(tendered_input),
            ids!(tip_input),
            ids!(table_area_input),
            ids!(table_range_input),
            ids!(table_remove_input),
            ids!(bill_header_input),
            ids!(fssai_input),
            ids!(bill_footer_input),
            ids!(team_pick_input),
            ids!(menu_variations_input),
            ids!(menu_group_input),
            ids!(menu_area_prices_input),
            ids!(menu_recipe_input),
            ids!(inv_reorder_input),
            ids!(delivery_by_input),
            ids!(resv_name_input),
            ids!(resv_phone_input),
            ids!(resv_guests_input),
            ids!(resv_when_input),
            ids!(resv_table_input),
            ids!(discount_input),
            ids!(discount_reason_input),
            ids!(discount_pin_input),
            ids!(opening_float_input),
            ids!(shift_counted_input),
        ] {
            self.align_input(cx, id, rtl);
        }
        for id in [
            ids!(move_help),
            ids!(line_action_title),
            ids!(change_label),
            ids!(h_bill_format),
            ids!(h_bill_format_help),
            ids!(h_rights_help),
            ids!(team_selected_label),
            ids!(client_found_label),
        ] {
            self.align_label(cx, id, rtl);
        }
        self.align_input(cx, ids!(customer_name_input), rtl);
        self.align_input(cx, ids!(onboard_cust_name), rtl);
        self.align_input(cx, ids!(credit_add_name), rtl);
        self.align_input(cx, ids!(org_name_input), rtl);
        self.align_input(cx, ids!(credit_add_kind), rtl);
        self.align_input(cx, ids!(branch_switch_input), rtl);
        self.align_input(cx, ids!(business_tax_id_input), rtl);
        self.align_input(cx, ids!(buy_item_input), rtl);
        self.align_input(cx, ids!(buy_note_input), rtl);
        self.align_input(cx, ids!(buy_party_kind), rtl);
        self.align_input(cx, ids!(buy_supplier_input), rtl);
        self.align_input(cx, ids!(client_code_input), rtl);
        self.align_input(cx, ids!(client_host_input), rtl);
        self.align_input(cx, ids!(client_name_input), rtl);
        self.align_input(cx, ids!(credit_limit_party), rtl);
        self.align_input(cx, ids!(day_close_counted_input), rtl);
        self.align_input(cx, ids!(inv_item_input), rtl);
        self.align_input(cx, ids!(inv_note_input), rtl);
        self.align_input(cx, ids!(lock_login_id), rtl);
        self.align_input(cx, ids!(lock_login_password), rtl);
        self.align_input(cx, ids!(mix_card_input), rtl);
        self.align_input(cx, ids!(mix_cash_input), rtl);
        self.align_input(cx, ids!(mix_upi_input), rtl);
        self.align_input(cx, ids!(onboard_org_input), rtl);
        self.align_input(cx, ids!(onboard_venue_input), rtl);
        self.align_input(cx, ids!(party_attach_note), rtl);
        self.align_input(cx, ids!(reason_input), rtl);
        self.align_input(cx, ids!(sale_attach_note), rtl);
        self.align_input(cx, ids!(server_name_input), rtl);
        self.align_input(cx, ids!(venue_title_input), rtl);
        self.align_input(cx, ids!(menu_aliases_input), rtl);
        self.align_input(cx, ids!(menu_price_input), rtl);
        self.align_input(cx, ids!(menu_unit_input), rtl);
        self.align_input(cx, ids!(menu_tax_input), rtl);
        self.align_input(cx, ids!(menu_category_input), rtl);
        self.align_input(cx, ids!(menu_modifiers_input), rtl);
        self.align_input(cx, ids!(menu_station_input), rtl);
        self.align_input(cx, ids!(kot_station_input), rtl);
        self.align_input(cx, ids!(station_remove_input), rtl);
        self.align_input(cx, ids!(settings_station_remove_input), rtl);
        self.align_input(cx, ids!(buy_qty_input), rtl);
        self.align_input(cx, ids!(buy_cost_input), rtl);
        self.align_input(cx, ids!(buy_expense_amount), rtl);
        self.align_input(cx, ids!(buy_expense_note), rtl);
        self.align_input(cx, ids!(mix_credit_input), rtl);
        self.align_input(cx, ids!(credit_add_phone), rtl);
        self.align_input(cx, ids!(credit_limit_input), rtl);
        self.align_input(cx, ids!(writeoff_amount_input), rtl);
        self.align_input(cx, ids!(writeoff_reason_input), rtl);
        self.align_input(cx, ids!(payable_amount_input), rtl);
        self.align_input(cx, ids!(payable_bill_input), rtl);
        self.align_input(cx, ids!(settle_amount_input), rtl);
        self.align_input(cx, ids!(settle_bill_input), rtl);
        self.align_input(cx, ids!(table_name_input), rtl);
        self.align_input(cx, ids!(station_name_input), rtl);
        self.align_input(cx, ids!(staff_name_input), rtl);
        self.align_input(cx, ids!(staff_title_input), rtl);
        self.align_input(cx, ids!(staff_pin_input), rtl);
        self.align_input(cx, ids!(staff_login_input), rtl);
        self.align_input(cx, ids!(staff_password_input), rtl);
        self.align_input(cx, ids!(my_pin_input), rtl);
        self.align_input(cx, ids!(my_password_input), rtl);
        self.align_input(cx, ids!(duress_pin_input), rtl);
        self.align_input(cx, ids!(duress_pin_confirm), rtl);
        self.align_input(cx, ids!(country_code_input), rtl);
        self.align_input(cx, ids!(service_bps_input), rtl);
        self.align_input(cx, ids!(onboard_item_name), rtl);
        self.align_input(cx, ids!(onboard_item_price), rtl);
        self.align_input(cx, ids!(onboard_cust_bal), rtl);
        self.align_input(cx, ids!(expense_amount_input), rtl);
        self.align_input(cx, ids!(expense_note_input), rtl);
        self.align_input(cx, ids!(bank_amount_input), rtl);
        self.align_input(cx, ids!(bank_note_input), rtl);
        self.align_input(cx, ids!(bank_clear_input), rtl);
        self.align_input(cx, ids!(inv_qty_input), rtl);
        self.align_input(cx, ids!(branch_name_input), rtl);
        self.align_input(cx, ids!(branch_code_input), rtl);
        self.align_input(cx, ids!(unit_code_input), rtl);
        self.align_input(cx, ids!(unit_name_input), rtl);
        self.align_input(cx, ids!(conv_from_input), rtl);
        self.align_input(cx, ids!(conv_to_input), rtl);
        self.align_input(cx, ids!(conv_factor_input), rtl);
    }

    fn paint_static_texts(&mut self, cx: &mut Cx) {
        let (lang, country_code) = {
            let st = app_state().lock().unwrap();
            (
                st.locale.language.clone(),
                st.fiscal.country_code.clone(),
            )
        };
        let s = |key: &str| locale::t(&lang, key);
        // Restaurant loop (running table, settle, floor, rights, LAN find).
        for (id, key) in [
            (ids!(print_bill_btn), "print_bill"),
            (ids!(table_move_btn), "move_merge_table"),
            (ids!(move_table_go), "move_here"),
            (ids!(merge_table_go), "merge_into"),
            (ids!(line_action_back), "back"),
            (ids!(line_action_go), "confirm"),
            (ids!(table_range_add), "add_these_tables"),
            (ids!(table_remove_btn), "remove_these_tables"),
            (ids!(client_find_btn), "find_server"),
            (ids!(bill_format_save), "save_bill_format"),
            (ids!(print_stamp_pick), "choose_stamp"),
            (ids!(print_stamp_clear), "clear_stamp"),
            (ids!(payment_qr_pick), "choose_payment_qr"),
            (ids!(payment_qr_clear), "clear_payment_qr"),
            (ids!(aggregator_import_btn), "import_aggregator_file"),
            (ids!(scheme_add), "add_scheme"),
            (ids!(show_upi_qr), "show_upi_qr"),
            (ids!(upi_qr_close), "close_upi_qr"),
            (ids!(recall_kot_btn), "kot_recall"),
            (ids!(discount_toggle), "discount"),
            (ids!(discount_remove), "remove_discount"),
            (ids!(discount_apply), "apply_discount"),
            (ids!(opening_float_save), "save_opening_cash"),
            (ids!(shift_close_btn), "close_shift_print"),
        ] {
            self.ui.button(cx, id).set_text(cx, &s(key));
        }
        for (id, key) in [
            (ids!(move_target_input), "move_target_ph"),
            (ids!(move_pin_input), "approval_pin_ph"),
            (ids!(line_reason_input), "line_reason_ph"),
            (ids!(line_qty_input), "line_qty_ph"),
            (ids!(line_target_input), "line_target_ph"),
            (ids!(line_pin_input), "approval_pin_ph"),
            (ids!(menu_search_input), "menu_search_ph"),
            (ids!(printer_label_input), "printer_label_ph"),
            (ids!(printer_target_input), "printer_target_ph"),
            (ids!(route_pick_input), "route_pick_ph"),
            (ids!(tendered_input), "customer_paid_ph"),
            (ids!(tip_input), "tip_ph"),
            (ids!(table_area_input), "table_area_ph"),
            (ids!(table_range_input), "table_range_ph"),
            (ids!(table_remove_input), "table_remove_ph"),
            (ids!(bill_header_input), "bill_header_ph"),
            (ids!(fssai_input), "fssai_ph"),
            (ids!(upi_vpa_input), "upi_vpa_ph"),
            (ids!(upi_payee_input), "upi_payee_ph"),
            (ids!(scheme_bogo_input), "scheme_bogo_ph"),
            (ids!(menu_schedule_input), "schedule_ph"),
            (ids!(recall_kot_input), "recall_kot_ph"),
            (ids!(bill_footer_input), "bill_footer_ph"),
            (ids!(team_pick_input), "team_pick_ph"),
            (ids!(menu_variations_input), "sizes_as_items_ph"),
            (ids!(menu_group_input), "dish_group_ph"),
            (ids!(menu_area_prices_input), "area_prices_ph"),
            (ids!(menu_recipe_input), "recipe_ph"),
            (ids!(inv_reorder_input), "reorder_ph"),
            (ids!(delivery_by_input), "delivery_by_ph"),
            (ids!(resv_name_input), "resv_name_ph"),
            (ids!(resv_phone_input), "resv_phone_ph"),
            (ids!(resv_guests_input), "resv_guests_ph"),
            (ids!(resv_when_input), "resv_when_ph"),
            (ids!(resv_table_input), "resv_table_ph"),
            (ids!(discount_input), "discount_ph"),
            (ids!(discount_reason_input), "discount_reason_ph"),
            (ids!(discount_pin_input), "approval_pin_ph"),
            (ids!(opening_float_input), "opening_float_ph"),
            (ids!(shift_counted_input), "shift_counted_ph"),
        ] {
            self.ui.text_input(cx, id).set_empty_text(cx, s(key));
        }
        self.ui.text_input(cx, ids!(resv_phone_input)).set_empty_text(
            cx,
            phone_placeholder(&s("resv_phone_ph"), &country_code),
        );
        for (id, key) in [
            (ids!(move_help), "move_help"),
            (ids!(h_bill_format), "bill_format"),
            (ids!(h_bill_format_help), "bill_format_help"),
            (ids!(h_print_brand), "print_brand_kit"),
            (ids!(h_onboard_layout), "onboard_layout"),
            (ids!(h_venue_icon), "app_icon"),
            (ids!(h_item_photo), "photo"),
            (ids!(h_stamp), "stamp_heading"),
            (ids!(h_payment_qr), "payment_qr_heading"),
            (ids!(h_aggregator_import), "aggregator_import"),
            (ids!(h_zero_sales), "zero_sales"),
            (ids!(h_schemes), "schemes"),
            (ids!(upi_qr_help), "upi_qr_help"),
            (ids!(h_rights_help), "rights_help"),
            (ids!(h_opening_float), "opening_float"),
            (ids!(h_shift_close), "shift_handover"),
            (ids!(h_split_payment), "split"),
            (ids!(h_split_bill), "split_bill"),
            (ids!(h_move_table), "move_merge_table"),
        ] {
            self.ui.widget(cx, id).set_text(cx, &plain_display_text(&s(key)));
        }
        self.ui
            .widget(cx, ids!(payment_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("mix_type_hint_khaynjo")));
        self.ui
            .widget(cx, ids!(move_table_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("move_help")));
        self.ui
            .widget(cx, ids!(upi_qr_title_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("upi_qr_help")));
        self.ui.widget(cx, ids!(h_unit_at_cap)).set_text(cx, &plain_display_text(&s("unit_at_cap")));
        self.ui.widget(cx, ids!(h_applied)).set_text(cx, &plain_display_text(&s("applied")));
        self.ui.widget(cx, ids!(h_local_badge)).set_text(cx, &plain_display_text(&s("local_badge")));
        self.ui.widget(cx, ids!(h_offline_badge)).set_text(cx, &plain_display_text(&s("offline_badge")));
        self.ui.widget(cx, ids!(h_local_badge_2)).set_text(cx, &plain_display_text(&s("local_badge")));
        self.ui.widget(cx, ids!(h_counter_locked)).set_text(cx, &plain_display_text(&s("counter_locked")));
        self.ui.widget(cx, ids!(h_shop)).set_text(cx, &plain_display_text(&s("shop")));
        self.ui.widget(cx, ids!(h_who_unlocking)).set_text(cx, &plain_display_text(&s("who_unlocking")));
        self.ui.widget(cx, ids!(h_setup_app_khaynjo)).set_text(cx, &plain_display_text(&s("setup_app_khaynjo")));
        self.ui.widget(cx, ids!(h_shop_logo_optional)).set_text(cx, &plain_display_text(&s("shop_logo_optional")));
        self.ui.widget(cx, ids!(h_product_locked_khaynjo)).set_text(cx, &plain_display_text(&s("product_locked_khaynjo")));
        self.ui.widget(cx, ids!(h_how_start)).set_text(cx, &plain_display_text(&s("how_start")));
        self.ui.widget(cx, ids!(h_fresh_book_khaynjo)).set_text(cx, &plain_display_text(&s("fresh_book_khaynjo")));
        self.ui.widget(cx, ids!(h_fresh_selected)).set_text(cx, &plain_display_text(&s("fresh_selected")));
        self.ui.widget(cx, ids!(h_later_country_pack)).set_text(cx, &plain_display_text(&s("later_country_pack")));
        self.ui.widget(cx, ids!(h_connect_to_server)).set_text(cx, &plain_display_text(&s("connect_to_server")));
        self.ui.widget(cx, ids!(h_money_in_sales)).set_text(cx, &plain_display_text(&s("money_in_sales")));
        self.ui.widget(cx, ids!(h_purchases_cogs)).set_text(cx, &plain_display_text(&s("purchases_cogs")));
        self.ui.widget(cx, ids!(h_expenses_word)).set_text(cx, &plain_display_text(&s("expenses_word")));
        self.ui.widget(cx, ids!(h_gross_word)).set_text(cx, &plain_display_text(&s("gross_word")));
        self.ui.widget(cx, ids!(h_net_word)).set_text(cx, &plain_display_text(&s("net_word")));
        self.ui.widget(cx, ids!(h_cash)).set_text(cx, &plain_display_text(&s("cash")));
        self.ui.widget(cx, ids!(h_card)).set_text(cx, &plain_display_text(&s("card")));
        self.ui.widget(cx, ids!(h_they_owe_us)).set_text(cx, &plain_display_text(&s("they_owe_us")));
        self.ui.widget(cx, ids!(h_we_owe_them)).set_text(cx, &plain_display_text(&s("we_owe_them")));
        self.ui.widget(cx, ids!(h_bank_balance)).set_text(cx, &plain_display_text(&s("bank_balance")));
        self.ui.widget(cx, ids!(h_cash_in_hand)).set_text(cx, &plain_display_text(&s("cash_in_hand")));
        self.ui.widget(cx, ids!(h_bank_net_today)).set_text(cx, &plain_display_text(&s("bank_net_today")));
        self.ui.widget(cx, ids!(h_local_badge_3)).set_text(cx, &plain_display_text(&s("local_badge")));
        self.ui.widget(cx, ids!(h_actual_total_cap)).set_text(cx, &plain_display_text(&s("actual_total_cap")));
        self.ui.widget(cx, ids!(h_applied_total_cap)).set_text(cx, &plain_display_text(&s("applied_total_cap")));
        self.ui.widget(cx, ids!(h_mix_type_hint_khaynjo)).set_text(cx, &plain_display_text(&s("mix_type_hint_khaynjo")));
        self.ui.widget(cx, ids!(h_rounded_total)).set_text(cx, &plain_display_text(&s("rounded_total")));
        self.ui.widget(cx, ids!(h_account_kind)).set_text(cx, &plain_display_text(&s("account_kind")));
        self.ui.widget(cx, ids!(h_permission_preset)).set_text(cx, &plain_display_text(&s("permission_preset")));
        self.ui.widget(cx, ids!(h_preset_help_khaynjo)).set_text(cx, &plain_display_text(&s("preset_help_rest")));
        for (idle, on, key) in [
            (ids!(staff_role_waiter), ids!(staff_role_waiter_on), "role_waiter"),
            (ids!(staff_role_kitchen), ids!(staff_role_kitchen_on), "role_kitchen"),
        ] {
            self.ui.button(cx, idle).set_text(cx, &s(key));
            self.ui.button(cx, on).set_text(cx, &s(key));
        }
        self.ui.widget(cx, ids!(h_optional_login_help)).set_text(cx, &plain_display_text(&s("optional_login_help")));
        self.ui.widget(cx, ids!(h_feature_access_help)).set_text(cx, &plain_display_text(&s("feature_access_help")));
        self.ui.widget(cx, ids!(h_change_my_pin_heading)).set_text(cx, &plain_display_text(&s("change_my_pin_heading")));
        self.ui.widget(cx, ids!(h_duress_heading)).set_text(cx, &plain_display_text(&s("duress_heading")));
        self.ui.widget(cx, ids!(h_duress_help)).set_text(cx, &plain_display_text(&s("duress_help")));
        self.ui.button(cx, ids!(save_duress_pin)).set_text(cx, &s("save_duress_pin"));
        self.ui.button(cx, ids!(clear_duress_pin)).set_text(cx, &s("clear_duress_pin"));
        self.ui.widget(cx, ids!(h_country_pack_help)).set_text(cx, &plain_display_text(&s("country_pack_help")));
        self.ui.widget(cx, ids!(h_language_setup_hint)).set_text(cx, &plain_display_text(&s("language_setup_hint")));
        self.ui.widget(cx, ids!(h_currency_use_help)).set_text(cx, &plain_display_text(&s("currency_use_help")));
        self.ui.widget(cx, ids!(h_product_only_khaynjo)).set_text(cx, &plain_display_text(&s("product_only_khaynjo")));
        self.ui.widget(cx, ids!(h_brand_shown_help)).set_text(cx, &plain_display_text(&s("brand_shown_help")));
        self.ui.widget(cx, ids!(h_units_help)).set_text(cx, &plain_display_text(&s("units_help")));
        self.ui
            .widget(cx, ids!(h_units_picker_help))
            .set_text(cx, &plain_display_text(&s("units_picker_help")));
        self.ui.widget(cx, ids!(h_kot_print_help_khaynjo)).set_text(cx, &plain_display_text(&s("kot_print_help_khaynjo")));
        self.ui.widget(cx, ids!(h_restore_help)).set_text(cx, &plain_display_text(&s("restore_help")));
        self.ui.widget(cx, ids!(stock_empty)).set_text(cx, &plain_display_text(&s("stock_empty")));
        self.ui.widget(cx, ids!(h_book_add_dish)).set_text(cx, &plain_display_text(&s("book_add_dish")));
        self.ui.widget(cx, ids!(h_book_opening_credit)).set_text(cx, &plain_display_text(&s("book_opening_credit")));
        self.ui.widget(cx, ids!(h_caption_today)).set_text(cx, &plain_display_text(&s("caption_today")));
        self.ui.widget(cx, ids!(h_caption_bills)).set_text(cx, &plain_display_text(&s("caption_bills")));
        self.ui.widget(cx, ids!(h_caption_payments_today)).set_text(cx, &plain_display_text(&s("caption_payments_today")));
        self.ui.widget(cx, ids!(h_caption_outstanding)).set_text(cx, &plain_display_text(&s("caption_outstanding")));
        self.ui.widget(cx, ids!(h_caption_bank)).set_text(cx, &plain_display_text(&s("caption_bank")));
        self.ui.widget(cx, ids!(h_your_categories)).set_text(cx, &plain_display_text(&s("your_categories")));
        self.ui.widget(cx, ids!(h_goods_received)).set_text(cx, &plain_display_text(&s("goods_received")));
        self.ui.widget(cx, ids!(h_expense_heading)).set_text(cx, &plain_display_text(&s("expense_heading")));
        self.ui.widget(cx, ids!(h_paid_by)).set_text(cx, &plain_display_text(&s("paid_by")));
        self.ui.widget(cx, ids!(h_second_way)).set_text(cx, &plain_display_text(&s("second_way")));
        self.ui.widget(cx, ids!(h_notes_attach_sale)).set_text(cx, &plain_display_text(&s("notes_attach_sale")));
        self.ui.widget(cx, ids!(h_credit_limit_heading)).set_text(cx, &plain_display_text(&s("credit_limit_heading")));
        self.ui.widget(cx, ids!(h_pay_selected_heading)).set_text(cx, &plain_display_text(&s("pay_selected_heading")));
        self.ui.widget(cx, ids!(h_write_off_heading)).set_text(cx, &plain_display_text(&s("write_off_heading")));
        self.ui.widget(cx, ids!(h_pay_ap_heading)).set_text(cx, &plain_display_text(&s("pay_ap_heading")));
        self.ui.widget(cx, ids!(h_notes_attach_party)).set_text(cx, &plain_display_text(&s("notes_attach_party")));
        self.ui.widget(cx, ids!(h_country_tax)).set_text(cx, &plain_display_text(&s("country_tax")));
        self.ui.widget(cx, ids!(h_language)).set_text(cx, &plain_display_text(&s("language")));
        self.ui.widget(cx, ids!(h_currency)).set_text(cx, &plain_display_text(&s("currency")));
        self.ui.widget(cx, ids!(h_extra_tills_heading)).set_text(cx, &plain_display_text(&s("extra_tills_heading")));
        self.ui.widget(cx, ids!(h_other_devices_help_heading)).set_text(cx, &plain_display_text(&s("other_devices_help_heading")));
        self.ui.widget(cx, ids!(h_other_devices_help)).set_text(cx, &plain_display_text(&s("other_devices_help")));
        self.ui.widget(cx, ids!(h_start_template)).set_text(cx, &plain_display_text(&s("start_template")));
        self.ui.widget(cx, ids!(h_your_brand)).set_text(cx, &plain_display_text(&s("your_brand")));
        self.ui.widget(cx, ids!(h_units_heading)).set_text(cx, &plain_display_text(&s("units_heading")));
        self.ui.widget(cx, ids!(h_kitchen_printing)).set_text(cx, &plain_display_text(&s("settings_sec_printers")));
        self.ui.widget(cx, ids!(h_kot_print_help_khaynjo)).set_text(cx, &plain_display_text(&s("kot_print_help_khaynjo")));
        self.ui.widget(cx, ids!(h_printers)).set_text(cx, &plain_display_text(&s("printers_heading")));
        self.ui.widget(cx, ids!(h_printers_help)).set_text(cx, &plain_display_text(&s("printers_help")));
        self.ui.widget(cx, ids!(h_printers_pool)).set_text(cx, &plain_display_text(&s("printers_pool_heading")));
        self.ui.widget(cx, ids!(h_printers_routes)).set_text(cx, &plain_display_text(&s("printers_routes_heading")));
        self.ui.widget(cx, ids!(h_printers_routes_help)).set_text(cx, &plain_display_text(&s("printers_routes_help")));
        self.ui.widget(cx, ids!(station_manage_heading)).set_text(cx, &plain_display_text(&s("station_manage_heading")));
        self.ui.button(cx, ids!(settings_station_add)).set_text(cx, &s("add_station"));
        self.ui.button(cx, ids!(settings_station_remove)).set_text(cx, &s("station_remove"));
        self.ui
            .text_input(cx, ids!(settings_station_name_input))
            .set_empty_text(cx, s("station_name_ph"));
        self.ui
            .text_input(cx, ids!(settings_station_remove_input))
            .set_empty_text(cx, s("station_remove_ph"));
        self.ui.button(cx, ids!(find_printers_btn)).set_text(cx, &s("find_printers"));
        self.ui
            .widget(cx, ids!(h_printer_wire))
            .set_text(cx, &plain_display_text(&s("printer_wire_heading")));
        for path in [ids!(printer_wire_escpos), ids!(printer_wire_escpos_on)] {
            self.ui.button(cx, path).set_text(cx, &s("printer_wire_escpos"));
        }
        for path in [ids!(printer_wire_os), ids!(printer_wire_os_on)] {
            self.ui.button(cx, path).set_text(cx, &s("printer_wire_os"));
        }
        self.ui.button(cx, ids!(printer_add)).set_text(cx, &s("add_printer"));
        self.ui.button(cx, ids!(printer_remove)).set_text(cx, &s("remove_printer"));
        self.ui.button(cx, ids!(route_up)).set_text(cx, &s("route_up"));
        self.ui.button(cx, ids!(route_down)).set_text(cx, &s("route_down"));
        self.ui.button(cx, ids!(route_remove)).set_text(cx, &s("route_remove"));
        self.ui.button(cx, ids!(route_clear)).set_text(cx, &s("route_clear"));
        self.ui.button(cx, ids!(printers_save)).set_text(cx, &s("save_printers"));
        self.ui.button(cx, ids!(print_test_btn)).set_text(cx, &s("print_test_page"));
        self.ui.widget(cx, ids!(h_backup)).set_text(cx, &plain_display_text(&s("backup")));
        self.ui.widget(cx, ids!(h_drawer_count)).set_text(cx, &plain_display_text(&s("drawer_count")));
        self.ui.widget(cx, ids!(h_paid_by_2)).set_text(cx, &plain_display_text(&s("paid_by")));
        self.ui.widget(cx, ids!(h_second_way_2)).set_text(cx, &plain_display_text(&s("second_way")));
        self.ui.widget(cx, ids!(h_attachments_last_expense)).set_text(cx, &plain_display_text(&s("attachments_last_expense")));
        self.ui.widget(cx, ids!(h_bank_reconciliation)).set_text(cx, &plain_display_text(&s("bank_reconciliation")));
        self.ui.widget(cx, ids!(h_spreadsheet_csv)).set_text(cx, &plain_display_text(&s("spreadsheet_csv")));
        self.ui.button(cx, ids!(add_items_btn)).set_text(cx, &s("add_dishes_items"));
        self.ui.button(cx, ids!(allow_extra_tills)).set_text(cx, &s("allow_extra_tills"));
        self.ui.button(cx, ids!(amend_btn)).set_text(cx, &s("amend"));
        self.ui.button(cx, ids!(backup_btn)).set_text(cx, &s("create_backup"));
        self.ui.button(cx, ids!(bank_bank_to_till)).set_text(cx, &s("bank_to_till"));
        self.ui.button(cx, ids!(bank_in)).set_text(cx, &s("bank_in"));
        self.ui.button(cx, ids!(bank_mark_cleared)).set_text(cx, &s("mark_cleared"));
        self.ui.button(cx, ids!(bank_mark_uncleared)).set_text(cx, &s("unmark"));
        self.ui.button(cx, ids!(bank_out)).set_text(cx, &s("bank_out"));
        self.ui.button(cx, ids!(bank_till_to_bank)).set_text(cx, &s("till_to_bank"));
        self.ui.button(cx, ids!(bill_req)).set_text(cx, &s("request_bill"));
        self.ui.button(cx, ids!(branch_add)).set_text(cx, &s("add_branch"));
        self.ui.button(cx, ids!(branch_consolidated)).set_text(cx, &s("rollup"));
        self.ui.button(cx, ids!(branch_export)).set_text(cx, &s("export_day"));
        self.ui.button(cx, ids!(branch_import)).set_text(cx, &s("import_day"));
        self.ui.button(cx, ids!(branch_switch)).set_text(cx, &s("switch_device"));
        self.ui.text_input(cx, ids!(branch_switch_input)).set_empty_text(cx, s("branch_name_sell_ph"));
        self.ui.text_input(cx, ids!(business_tax_id_input)).set_empty_text(cx, s("business_tax_id_ph"));
        self.ui.text_input(cx, ids!(buy_item_input)).set_empty_text(cx, s("item_name"));
        self.ui.text_input(cx, ids!(buy_note_input)).set_empty_text(cx, s("note_ph"));
        self.ui.button(cx, ids!(buy_on_account)).set_text(cx, &s("on_account_ap"));
        self.ui.button(cx, ids!(buy_on_account_on)).set_text(cx, &s("on_account_ap"));
        self.ui.button(cx, ids!(buy_paid)).set_text(cx, &s("paid_now"));
        self.ui.button(cx, ids!(buy_paid_on)).set_text(cx, &s("paid_now"));
        self.ui.text_input(cx, ids!(buy_party_kind)).set_empty_text(cx, s("new_party_kind_supplier_ph"));
        self.ui.text_input(cx, ids!(buy_supplier_input)).set_empty_text(cx, s("supplier_name_ph"));
        self.ui.button(cx, ids!(change_my_password)).set_text(cx, &s("update_my_password"));
        self.ui.button(cx, ids!(change_my_pin)).set_text(cx, &s("update_my_pin"));
        self.ui.text_input(cx, ids!(client_code_input)).set_empty_text(cx, s("pairing_code_ph"));
        self.ui.text_input(cx, ids!(client_host_input)).set_empty_text(cx, s("server_ip_ph"));
        self.ui.button(cx, ids!(client_pair_btn)).set_text(cx, &s("pair_now"));
        self.ui.button(cx, ids!(client_skip_lock)).set_text(cx, &s("unlock_after_pairing"));
        self.ui.button(cx, ids!(client_sync_btn)).set_text(cx, &s("refresh_from_shop"));
        self.ui.button(cx, ids!(client_unpair)).set_text(cx, &s("forget_shop"));
        self.ui.text_input(cx, ids!(client_name_input)).set_empty_text(cx, s("device_name_ph"));
        {
            // Offer this device's own name; the person may type a better one.
            let field = self.ui.text_input(cx, ids!(client_name_input));
            if field.text().is_empty() && !cx.has_key_focus(field.area()) {
                field.set_text(cx, &device_name());
            }
        }
        self.ui.button(cx, ids!(client_clear_refused)).set_text(cx, &s("clear_not_accepted"));
        self.ui.button(cx, ids!(client_connection_btn)).set_text(cx, &s("connection_to_shop"));
        self.ui.button(cx, ids!(conv_add)).set_text(cx, &s("add_conversion"));
        self.ui.button(cx, ids!(country_apply)).set_text(cx, &s("apply_country_pack"));
        self.ui.button(cx, ids!(credit_kind_both)).set_text(cx, &s("role_both"));
        self.ui.button(cx, ids!(credit_kind_both_on)).set_text(cx, &s("role_both"));
        self.ui.button(cx, ids!(credit_kind_customer)).set_text(cx, &s("role_customer"));
        self.ui.button(cx, ids!(credit_kind_customer_on)).set_text(cx, &s("role_customer"));
        self.ui.button(cx, ids!(credit_kind_supplier)).set_text(cx, &s("supplier_ph"));
        self.ui.button(cx, ids!(credit_kind_supplier_on)).set_text(cx, &s("supplier_ph"));
        self.ui.text_input(cx, ids!(credit_limit_party)).set_empty_text(cx, s("party_name_selected_ph"));
        self.ui.button(cx, ids!(credit_limit_set)).set_text(cx, &s("set_credit_limit"));
        self.ui.button(cx, ids!(credit_writeoff)).set_text(cx, &s("write_off_nil"));
        self.ui.button(cx, ids!(csv_copy_day)).set_text(cx, &s("copy_day_book_csv"));
        self.ui.button(cx, ids!(csv_copy_items)).set_text(cx, &s("copy_items_csv"));
        self.ui.button(cx, ids!(csv_copy_ledger)).set_text(cx, &s("copy_party_ledger_csv"));
        self.ui.button(cx, ids!(csv_copy_outstanding)).set_text(cx, &s("copy_outstanding_csv"));
        self.ui.button(cx, ids!(csv_copy_parties)).set_text(cx, &s("copy_parties_csv"));
        self.ui.button(cx, ids!(csv_import_items)).set_text(cx, &s("import_items_csv_plain"));
        self.ui.button(cx, ids!(csv_import_parties)).set_text(cx, &s("import_parties_csv"));
        self.ui.button(cx, ids!(customer_use)).set_text(cx, &s("add_use_name"));
        self.ui.button(cx, ids!(day_close_commit)).set_text(cx, &s("record_day_close"));
        self.ui.button(cx, ids!(day_close_copy)).set_text(cx, &s("copy_day_close"));
        self.ui.text_input(cx, ids!(day_close_counted_input)).set_empty_text(cx, s("counted_cash_ph"));
        self.ui.button(cx, ids!(device_role_client_on)).set_text(cx, &s("extra_till"));
        self.ui.button(cx, ids!(device_role_server_on)).set_text(cx, &s("this_shop"));
        self.ui.button(cx, ids!(expense_attach_pdf)).set_text(cx, &s("attach_pdf"));
        self.ui.button(cx, ids!(expense_attach_photo)).set_text(cx, &s("attach_bill_photo"));
        self.ui.button(cx, ids!(feat_backup)).set_text(cx, &s("backup"));
        self.ui.button(cx, ids!(feat_backup_on)).set_text(cx, &s("backup"));
        self.ui.button(cx, ids!(feat_branches)).set_text(cx, &s("branches"));
        self.ui.button(cx, ids!(feat_branches_on)).set_text(cx, &s("branches"));
        self.ui.button(cx, ids!(feat_credit)).set_text(cx, &s("credit"));
        self.ui.button(cx, ids!(feat_credit_on)).set_text(cx, &s("credit"));
        self.ui.button(cx, ids!(feat_floor)).set_text(cx, &s("floor"));
        self.ui.button(cx, ids!(feat_floor_on)).set_text(cx, &s("floor"));
        self.ui.button(cx, ids!(feat_inventory)).set_text(cx, &s("inventory"));
        self.ui.button(cx, ids!(feat_inventory_on)).set_text(cx, &s("inventory"));
        self.ui.button(cx, ids!(feat_items)).set_text(cx, &s("items"));
        self.ui.button(cx, ids!(feat_items_on)).set_text(cx, &s("items"));
        self.ui.button(cx, ids!(feat_kitchen)).set_text(cx, &s("kitchen"));
        self.ui.button(cx, ids!(feat_kitchen_on)).set_text(cx, &s("kitchen"));
        self.ui.button(cx, ids!(feat_sales)).set_text(cx, &s("sales"));
        self.ui.button(cx, ids!(feat_sales_on)).set_text(cx, &s("sales"));
        self.ui.button(cx, ids!(feat_save)).set_text(cx, &s("save_features"));
        self.ui.button(cx, ids!(feat_sell)).set_text(cx, &s("sell"));
        self.ui.button(cx, ids!(feat_sell_on)).set_text(cx, &s("sell"));
        self.ui.button(cx, ids!(feat_team)).set_text(cx, &s("team"));
        self.ui.button(cx, ids!(feat_team_on)).set_text(cx, &s("team"));
        self.ui.button(cx, ids!(fiscal_save)).set_text(cx, &s("save_business_id"));
        self.ui.text_input(cx, ids!(inv_item_input)).set_empty_text(cx, s("item_name"));
        self.ui.text_input(cx, ids!(inv_note_input)).set_empty_text(cx, s("reason_note_ph"));
        self.ui.button(cx, ids!(lane_declared_on)).set_text(cx, &s("tax_invoice"));
        self.ui.button(cx, ids!(lane_owner_on)).set_text(cx, &s("cash_memo"));
        self.ui.text_input(cx, ids!(lock_login_id)).set_empty_text(cx, s("login_id_ph"));
        self.ui.button(cx, ids!(lock_login_ok)).set_text(cx, &s("unlock_with_login"));
        self.ui.text_input(cx, ids!(lock_login_password)).set_empty_text(cx, s("password_ph"));
        self.ui.text_input(cx, ids!(mix_card_input)).set_empty_text(cx, s("card_amount_ph"));
        self.ui.text_input(cx, ids!(mix_cash_input)).set_empty_text(cx, s("cash_amount_ph"));
        self.ui.text_input(cx, ids!(mix_upi_input)).set_empty_text(cx, s("upi_amount_ph"));
        self.ui.button(cx, ids!(next_order_btn)).set_text(cx, &s("next_order"));
        self.ui.button(cx, ids!(onboard_add_credit)).set_text(cx, &s("add_opening_balance"));
        self.ui.button(cx, ids!(onboard_add_item)).set_text(cx, &s("add_item_plain"));
        self.ui.button(cx, ids!(onboard_book)).set_text(cx, &s("bring_my_book"));
        self.ui.button(cx, ids!(onboard_book_on)).set_text(cx, &s("bring_my_book"));
        self.ui.button(cx, ids!(onboard_done)).set_text(cx, &s("done_open_books"));
        self.ui.button(cx, ids!(onboard_fresh)).set_text(cx, &s("start_fresh"));
        self.ui.button(cx, ids!(onboard_template)).set_text(cx, &s("start_with_template"));
        self.ui.button(cx, ids!(onboard_template_on)).set_text(cx, &s("start_with_template"));
        self.ui.button(cx, ids!(onboard_fresh_on)).set_text(cx, &s("start_fresh"));
        self.ui.button(cx, ids!(onboard_icon_pick)).set_text(cx, &s("choose_logo"));
        self.ui.text_input(cx, ids!(onboard_org_input)).set_empty_text(cx, s("org_optional_ph"));
        self.ui.button(cx, ids!(onboard_save_venue)).set_text(cx, &s("save_names"));
        self.ui.text_input(cx, ids!(onboard_venue_input)).set_empty_text(cx, s("restaurant_name_ph"));
        self.ui.button(cx, ids!(otype_counter)).set_text(cx, &s("counter"));
        self.ui.button(cx, ids!(otype_counter_on)).set_text(cx, &s("counter"));
        self.ui.button(cx, ids!(otype_delivery)).set_text(cx, &s("delivery"));
        self.ui.button(cx, ids!(otype_delivery_on)).set_text(cx, &s("delivery"));
        self.ui.button(cx, ids!(otype_dinein)).set_text(cx, &s("dine_in"));
        self.ui.button(cx, ids!(otype_dinein_on)).set_text(cx, &s("dine_in"));
        self.ui.button(cx, ids!(otype_takeaway)).set_text(cx, &s("takeaway"));
        self.ui.button(cx, ids!(otype_takeaway_on)).set_text(cx, &s("takeaway"));
        self.ui.button(cx, ids!(party_attach_add_note)).set_text(cx, &s("add_note"));
        self.ui.text_input(cx, ids!(party_attach_note)).set_empty_text(cx, s("note_gstin_ph"));
        self.ui.button(cx, ids!(party_attach_pdf)).set_text(cx, &s("attach_pdf"));
        self.ui.button(cx, ids!(party_attach_photo)).set_text(cx, &s("attach_photo"));
        self.ui.button(cx, ids!(party_delete)).set_text(cx, &s("delete_party"));
        self.ui.button(cx, ids!(party_form_clear)).set_text(cx, &s("new_party_clear"));
        self.ui.button(cx, ids!(payable_card)).set_text(cx, &s("pay_ap_card"));
        self.ui.button(cx, ids!(payable_cash)).set_text(cx, &s("pay_ap_cash"));
        self.ui.button(cx, ids!(payable_upi)).set_text(cx, &s("pay_ap_upi"));
        self.ui.button(cx, ids!(payable_writeoff)).set_text(cx, &s("write_off_ap"));
        self.ui.button(cx, ids!(photo)).set_text(cx, &s("photo"));
        self.ui.text_input(cx, ids!(reason_input)).set_empty_text(cx, s("reason_amend_ph"));
        self.ui.button(cx, ids!(receipt_print_btn)).set_text(cx, &s("print_open"));
        self.ui.button(cx, ids!(receipt_share_btn)).set_text(cx, &s("copy_receipt"));
        self.ui.button(cx, ids!(receipt_whatsapp_btn)).set_text(cx, &s("share_whatsapp"));
        self.ui.button(cx, ids!(waitlist_add)).set_text(cx, &s("waitlist_add"));
        self.ui.button(cx, ids!(bulk_price_apply)).set_text(cx, &s("bulk_price_apply"));
        self.ui
            .widget(cx, ids!(crm_heading))
            .set_text(cx, &plain_display_text(&s("crm_heading")));
        self.ui
            .widget(cx, ids!(crm_help))
            .set_text(cx, &plain_display_text(&s("crm_help")));
        self.ui.button(cx, ids!(crm_queue_sms)).set_text(cx, &s("crm_queue_sms"));
        self.ui.button(cx, ids!(crm_queue_wa)).set_text(cx, &s("crm_queue_wa"));
        self.ui.button(cx, ids!(crm_seg_bday_week)).set_text(cx, &s("crm_seg_bday_week"));
        self.ui.button(cx, ids!(crm_seg_bday_month)).set_text(cx, &s("crm_seg_bday_month"));
        self.ui.button(cx, ids!(crm_seg_phone)).set_text(cx, &s("crm_seg_phone"));
        self.ui.button(cx, ids!(crm_seg_loyalty)).set_text(cx, &s("crm_seg_loyalty"));
        self.ui.button(cx, ids!(crm_seg_inactive)).set_text(cx, &s("crm_seg_inactive"));
        self.ui.button(cx, ids!(crm_seg_top)).set_text(cx, &s("crm_seg_top"));
        self.ui
            .text_input(cx, ids!(credit_add_birthday))
            .set_empty_text(cx, s("birthday_ph"));
        self.ui
            .text_input(cx, ids!(crm_message_input))
            .set_empty_text(cx, s("crm_message_ph"));
        self.ui
            .button(cx, ids!(till_apply_redeem))
            .set_text(cx, &s("till_apply_redeem"));
        self.ui
            .button(cx, ids!(till_clear_redeem))
            .set_text(cx, &s("till_clear_redeem"));
        self.ui
            .button(cx, ids!(till_wallet_topup))
            .set_text(cx, &s("till_wallet_topup"));
        self.ui
            .button(cx, ids!(till_issue_voucher))
            .set_text(cx, &s("till_issue_voucher"));
        self.ui
            .text_input(cx, ids!(till_loyalty_input))
            .set_empty_text(cx, s("till_points_ph"));
        self.ui
            .text_input(cx, ids!(till_wallet_input))
            .set_empty_text(cx, s("till_wallet_ph"));
        self.ui
            .text_input(cx, ids!(till_voucher_input))
            .set_empty_text(cx, s("till_voucher_code_ph"));
        self.ui
            .text_input(cx, ids!(till_voucher_amt_input))
            .set_empty_text(cx, s("till_voucher_amt_ph"));
        self.ui
            .text_input(cx, ids!(till_wallet_topup_input))
            .set_empty_text(cx, s("till_wallet_topup_ph"));
        self.ui
            .text_input(cx, ids!(till_voucher_issue_amt_input))
            .set_empty_text(cx, s("till_voucher_issue_amt_ph"));
        self.ui
            .text_input(cx, ids!(till_voucher_issue_code))
            .set_empty_text(cx, s("till_voucher_issue_code_ph"));
        self.ui
            .widget(cx, ids!(waitlist_heading))
            .set_text(cx, &plain_display_text(&s("waitlist")));
        self.ui
            .widget(cx, ids!(sibling_pickers_help))
            .set_text(cx, &plain_display_text(&s("sibling_pickers_help")));
        {
            let help = self.ui.widget(cx, ids!(floor_help)).as_info_help();
            help.set_title(cx, &s("floor"));
            help.set_nav_icon(cx, NavTab::Floor);
            help.set_tip(&plain_display_text(&s("floor_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(settings_lan_help)).as_info_help();
            help.set_title(cx, &s("lan_pages_heading"));
            help.set_tip(&plain_display_text(&s("lan_pages_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(kitchen_help)).as_info_help();
            help.set_title(cx, &s("kitchen"));
            help.set_nav_icon(cx, NavTab::Kitchen);
            help.set_tip(&plain_display_text(&s("kitchen_help")));
        }
        self.ui
            .widget(cx, ids!(floor_areas_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("floor_areas_help")));
        self.ui
            .widget(cx, ids!(settings_aggregator_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("aggregator_import_help")));
        self.ui
            .widget(cx, ids!(settings_zero_sales_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("zero_sales_help")));
        self.ui
            .widget(cx, ids!(settings_schemes_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("schemes_help")));
        self.ui
            .widget(cx, ids!(onboard_icon_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("logo_help")));
        self.ui
            .widget(cx, ids!(venue_icon_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("brand_shown_help")));
        self.ui
            .widget(cx, ids!(item_photo_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("item_photo_help")));
        self.ui
            .widget(cx, ids!(print_brand_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("print_brand_kit_help")));
        self.ui
            .widget(cx, ids!(print_stamp_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("stamp_image_help")));
        self.ui
            .widget(cx, ids!(payment_qr_image_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("payment_qr_image_help")));
        self.sync_floor_sections(cx);
        self.sync_device_setup_sections(cx);
        self.ui.button(cx, ids!(refund_btn)).set_text(cx, &s("refund"));
        self.ui.button(cx, ids!(reports_copy)).set_text(cx, &s("copy_report"));
        self.ui.button(cx, ids!(restore_btn)).set_text(cx, &s("restore_backup"));
        self.ui.button(cx, ids!(sale_attach_add_note)).set_text(cx, &s("add_note"));
        self.ui.text_input(cx, ids!(sale_attach_note)).set_empty_text(cx, s("extra_note_bill_ph"));
        self.ui.button(cx, ids!(sale_attach_pdf)).set_text(cx, &s("attach_pdf"));
        self.ui.button(cx, ids!(sale_attach_photo)).set_text(cx, &s("attach_photo"));
        // send_kot_btn text is set in sync_chrome (names the kitchens).
        self.ui.button(cx, ids!(staff_add_cashier)).set_text(cx, &s("add_staff"));
        self.ui.button(cx, ids!(staff_deactivate)).set_text(cx, &s("remove_selected"));
        self.ui.button(cx, ids!(staff_role_cashier)).set_text(cx, &s("role_cashier"));
        self.ui.button(cx, ids!(staff_role_cashier_on)).set_text(cx, &s("role_cashier"));
        self.ui.button(cx, ids!(staff_role_manager)).set_text(cx, &s("role_manager"));
        self.ui.button(cx, ids!(staff_role_manager_on)).set_text(cx, &s("role_manager"));
        self.ui.button(cx, ids!(staff_set_login)).set_text(cx, &s("set_login_selected"));
        self.ui.button(cx, ids!(station_add)).set_text(cx, &s("add_station"));
        self.ui.button(cx, ids!(stop_extra_tills)).set_text(cx, &s("stop_extra_tills"));
        self.ui.button(cx, ids!(allow_firewall)).set_text(cx, &s("allow_firewall"));
        self.ui.button(cx, ids!(new_pair_code)).set_text(cx, &s("new_pair_code"));
        // Android has no folder to copy into each day: the owner saves a
        // copy where they choose (USB stick, SD card, Downloads).
        self.ui.button(cx, ids!(backup_copy_btn)).set_text(
            cx,
            &s(if cfg!(target_os = "android") { "save_backup_copy" } else { "backup_copy_btn" }),
        );
        self.ui.button(cx, ids!(autostart_btn)).set_text(
            cx,
            &s(if startup::is_on() { "autostart_on" } else { "autostart_off" }),
        );
        self.ui.button(cx, ids!(table_add)).set_text(cx, &s("add_table"));
        for (idle, on, key) in [
            (ids!(tax_exclusive_btn), ids!(tax_exclusive_btn_on), "tax_added_till"),
            (ids!(tax_inclusive_btn), ids!(tax_inclusive_btn_on), "tax_inside_price"),
            (ids!(zero_sales_on_btn), ids!(zero_sales_on_btn_on), "zero_sales_on"),
            (ids!(zero_sales_off_btn), ids!(zero_sales_off_btn_on), "zero_sales_off"),
        ] {
            self.ui.button(cx, idle).set_text(cx, &s(key));
            self.ui.button(cx, on).set_text(cx, &s(key));
        }
        self.ui.button(cx, ids!(unit_add)).set_text(cx, &s("add_unit"));
        self.ui.button(cx, ids!(venue_brand_save)).set_text(cx, &s("save_brand"));
        self.ui.button(cx, ids!(venue_icon_clear)).set_text(cx, &s("clear_icon"));
        self.ui.button(cx, ids!(venue_icon_pick)).set_text(cx, &s("choose_icon"));
        self.ui.button(cx, ids!(void_btn)).set_text(cx, &s("void"));
        self.ui.text_input(cx, ids!(onboard_item_name)).set_empty_text(cx, s("item_name"));
        self.ui.text_input(cx, ids!(onboard_item_price)).set_empty_text(cx, s("item_price_ph"));
        self.ui.text_input(cx, ids!(onboard_cust_bal)).set_empty_text(cx, s("amount_owed_ph"));
        self.ui.text_input(cx, ids!(mix_credit_input)).set_empty_text(cx, s("credit_khata_amount_ph"));
        self.ui.text_input(cx, ids!(table_name_input)).set_empty_text(cx, s("table_name_ph"));
        self.ui.text_input(cx, ids!(station_name_input)).set_empty_text(cx, s("station_name_ph"));
        self.ui.text_input(cx, ids!(menu_name_input)).set_empty_text(cx, s("dish_search_ph"));
        self.ui.text_input(cx, ids!(menu_aliases_input)).set_empty_text(cx, s("aliases_ph"));
        self.ui.text_input(cx, ids!(menu_unit_input)).set_empty_text(cx, s("unit_plate_ph"));
        self.ui.text_input(cx, ids!(menu_tax_input)).set_empty_text(cx, s("tax_rate_ph"));
        self.ui.text_input(cx, ids!(menu_category_input)).set_empty_text(cx, s("category_search_ph"));
        self.ui.text_input(cx, ids!(menu_station_input)).set_empty_text(cx, s("station_new_ph"));
        self.ui.text_input(cx, ids!(kot_station_input)).set_empty_text(cx, s("kot_station_ph"));
        self.ui.button(cx, ids!(kot_set_station)).set_text(cx, &s("kot_set_station"));
        self.ui.text_input(cx, ids!(menu_modifiers_input)).set_empty_text(cx, s("options_khaynjo_ph"));
        self.ui.text_input(cx, ids!(buy_qty_input)).set_empty_text(cx, s("qty_example_ph"));
        self.ui.text_input(cx, ids!(buy_cost_input)).set_empty_text(cx, s("unit_cost_ph"));
        self.ui.text_input(cx, ids!(buy_expense_amount)).set_empty_text(cx, s("amount_ph"));
        self.ui.text_input(cx, ids!(buy_expense_note)).set_empty_text(cx, s("note_buy_expense_ph"));
        self.ui.text_input(cx, ids!(approval_pin_input)).set_empty_text(cx, s("manager_pin_ph"));
        self.ui.text_input(cx, ids!(credit_add_phone)).set_empty_text(
            cx,
            phone_placeholder(&s("phone_ph"), &country_code),
        );
        self.ui.text_input(cx, ids!(credit_limit_input)).set_empty_text(cx, s("credit_limit_ph"));
        self.ui.text_input(cx, ids!(settle_amount_input)).set_empty_text(cx, s("amount_received_ph"));
        self.ui.text_input(cx, ids!(settle_bill_input)).set_empty_text(cx, s("against_bill_ph"));
        self.ui.text_input(cx, ids!(writeoff_amount_input)).set_empty_text(cx, s("amount_blank_full_ph"));
        self.ui.text_input(cx, ids!(writeoff_reason_input)).set_empty_text(cx, s("reason_required_ph"));
        self.ui.text_input(cx, ids!(payable_amount_input)).set_empty_text(cx, s("amount_we_pay_ph"));
        self.ui.text_input(cx, ids!(payable_bill_input)).set_empty_text(cx, s("against_bill_ph"));
        self.ui.text_input(cx, ids!(staff_name_input)).set_empty_text(cx, s("staff_name_ph"));
        self.ui.text_input(cx, ids!(staff_title_input)).set_empty_text(cx, s("role_title_rest_ph"));
        self.ui.text_input(cx, ids!(staff_pin_input)).set_empty_text(cx, s("starting_pin_ph"));
        self.ui.text_input(cx, ids!(staff_login_input)).set_empty_text(cx, s("login_id_opt_ph"));
        self.ui.text_input(cx, ids!(staff_password_input)).set_empty_text(cx, s("starting_password_ph"));
        self.ui.text_input(cx, ids!(my_pin_input)).set_empty_text(cx, s("new_pin_ph"));
        self.ui.text_input(cx, ids!(my_password_input)).set_empty_text(cx, s("new_password_ph"));
        self.ui.text_input(cx, ids!(duress_pin_input)).set_empty_text(cx, s("duress_pin_ph"));
        self.ui.text_input(cx, ids!(duress_pin_confirm)).set_empty_text(cx, s("duress_confirm_ph"));
        self.ui.text_input(cx, ids!(country_code_input)).set_empty_text(cx, s("country_code_ph"));
        self.ui.text_input(cx, ids!(service_bps_input)).set_empty_text(cx, s("service_pct_ph"));
        self.ui.text_input(cx, ids!(venue_title_input)).set_empty_text(cx, s("place_name_ph"));
        self.ui.text_input(cx, ids!(unit_code_input)).set_empty_text(cx, s("code_bag_ph"));
        self.ui.text_input(cx, ids!(unit_name_input)).set_empty_text(cx, s("unit_name_bag_ph"));
        self.ui.text_input(cx, ids!(conv_from_input)).set_empty_text(cx, s("from_code_ph"));
        self.ui.text_input(cx, ids!(conv_to_input)).set_empty_text(cx, s("to_code_ph"));
        self.ui.text_input(cx, ids!(conv_factor_input)).set_empty_text(cx, s("conv_factor_ph"));
        self.ui.text_input(cx, ids!(bank_amount_input)).set_empty_text(cx, s("amount_ph"));
        self.ui.text_input(cx, ids!(bank_note_input)).set_empty_text(cx, s("note_bank_ph"));
        self.ui.text_input(cx, ids!(bank_clear_input)).set_empty_text(cx, s("uncleared_no_ph"));
        self.ui.text_input(cx, ids!(inv_qty_input)).set_empty_text(cx, s("quantity_ph"));
        self.ui.text_input(cx, ids!(org_name_input)).set_empty_text(cx, s("business_name_ph"));
        self.ui.text_input(cx, ids!(branch_name_input)).set_empty_text(cx, s("new_branch_name_ph"));
        self.ui.text_input(cx, ids!(branch_code_input)).set_empty_text(cx, s("short_code_ph"));
        self.ui.widget(cx, ids!(order_help)).set_text(cx, &plain_display_text(&s("order_help")));
        self.ui.text_input(cx, ids!(layout_x_input)).set_empty_text(cx, s("floor_layout_x"));
        self.ui.text_input(cx, ids!(layout_y_input)).set_empty_text(cx, s("floor_layout_y"));
        self.ui.text_input(cx, ids!(layout_w_input)).set_empty_text(cx, s("floor_layout_w"));
        self.ui.text_input(cx, ids!(layout_h_input)).set_empty_text(cx, s("floor_layout_h"));
        self.ui.button(cx, ids!(layout_save)).set_text(cx, &s("floor_layout_save"));
        self.ui.button(cx, ids!(layout_open)).set_text(cx, &s("floor_layout_open"));
        self.ui.button(cx, ids!(layout_nudge_left)).set_text(cx, &s("floor_layout_nudge_left"));
        self.ui.button(cx, ids!(layout_nudge_right)).set_text(cx, &s("floor_layout_nudge_right"));
        self.ui.button(cx, ids!(layout_nudge_up)).set_text(cx, &s("floor_layout_nudge_up"));
        self.ui.button(cx, ids!(layout_nudge_down)).set_text(cx, &s("floor_layout_nudge_down"));
        self.ui.widget(cx, ids!(floor_add_heading)).set_text(cx, &plain_display_text(&s("add_a_table")));
        self.ui.button(cx, ids!(area_new_add)).set_text(cx, &s("add_area"));
        self.ui.text_input(cx, ids!(area_new_input)).set_empty_text(cx, s("area_new_ph"));
        self.ui.widget(cx, ids!(kitchen_empty)).set_text(cx, &plain_display_text(&s("kitchen_empty")));
        self.ui.widget(cx, ids!(stations_heading)).set_text(cx, &plain_display_text(&s("stations_heading")));
        self.ui
            .widget(cx, ids!(stations_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("stations_help")));
        self.ui.widget(cx, ids!(station_add_heading)).set_text(cx, &plain_display_text(&s("add_station_heading")));
        self.ui.widget(cx, ids!(categories_help)).set_text(cx, &plain_display_text(&s("categories_help")));
        self.ui.widget(cx, ids!(h_removed_items)).set_text(cx, &plain_display_text(&s("removed_items")));
        self.ui.widget(cx, ids!(removed_help)).set_text(cx, &plain_display_text(&s("removed_help")));
    }
}

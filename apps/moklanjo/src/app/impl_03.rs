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
                field.take_key_focus(cx);
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
                        field.take_key_focus(cx);
                    }
                }
            }
        }
        self.paint_chrome(cx);
        self.paint_static_texts(cx);
        self.apply_text_alignment(cx);
        self.paint_choice_state(cx);
        // Facet titles after paint_chrome so locale strings do not overwrite Ops/COD/Riders.
        {
            let (screen, reports_nav, lang) = {
                let st = app_state().lock().unwrap();
                (st.screen, st.reports_nav, st.locale.language.clone())
            };
            if screen == Screen::Reports {
                let (title, help) = match reports_nav {
                    ReportsNav::Bank => (
                        locale::t(&lang, "facet_bank_title"),
                        locale::t(&lang, "facet_bank_help"),
                    ),
                    ReportsNav::Ops => (
                        locale::t(&lang, "facet_ops_title"),
                        locale::t(&lang, "facet_ops_help"),
                    ),
                    ReportsNav::Cod => (
                        locale::t(&lang, "facet_cod_title"),
                        locale::t(&lang, "facet_cod_help"),
                    ),
                    ReportsNav::Riders => (
                        locale::t(&lang, "facet_riders_title"),
                        locale::t(&lang, "facet_riders_help"),
                    ),
                };
                let help_w = self.ui.widget(cx, ids!(reports_help)).as_info_help();
                help_w.set_title(cx, &title);
                help_w.set_tip(&plain_display_text(&help));
                let facet_tab = match reports_nav {
                    ReportsNav::Bank => NavTab::Reports,
                    ReportsNav::Ops => NavTab::Ops,
                    ReportsNav::Cod => NavTab::Cod,
                    ReportsNav::Riders => NavTab::Riders,
                };
                help_w.set_nav_icon(cx, facet_tab);
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
    fn expense_parts(
        total: i64,
        split_text: &str,
        currency: &str,
        tender: PayMethod,
        tender2: PayMethod,
    ) -> Result<Vec<(PayMethod, i64)>, String> {
        let split_text = split_text.trim();
        if split_text.is_empty() {
            return Ok(vec![(tender, total)]);
        }
        let part2 = locale::parse_major_input(split_text, currency)?;
        if part2 <= 0 || part2 >= total {
            return Err("The second-way part must be less than the total".into());
        }
        if tender2 == tender {
            return Ok(vec![(tender, total)]);
        }
        Ok(vec![(tender, total - part2), (tender2, part2)])
    }

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
        let order = {
            let st = app_state().lock().unwrap();
            nav_display_order(&st.nav_order, &st.nav_hidden)
        };
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
        let tmpl = store.apply_entity_template("moklanjo_agency")?;
        st.entity_template_id = tmpl.id.to_string();
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
            amend_pay,
            tax_inclusive,
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
                st.amend_pay,
                st.fiscal.tax_pricing.is_inclusive(),
            )
        };
        self.paint_choice_pair(
            cx,
            ids!(tax_inclusive_btn),
            ids!(tax_inclusive_btn_on),
            tax_inclusive,
        );
        self.paint_choice_pair(
            cx,
            ids!(tax_exclusive_btn),
            ids!(tax_exclusive_btn_on),
            !tax_inclusive,
        );
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
        for (idle, on, m) in [
            (ids!(amend_pay_cash), ids!(amend_pay_cash_on), PayMethod::Cash),
            (ids!(amend_pay_upi), ids!(amend_pay_upi_on), PayMethod::Upi),
            (ids!(amend_pay_card), ids!(amend_pay_card_on), PayMethod::Card),
        ] {
            self.paint_choice_pair(cx, idle, on, amend_pay == m);
        }
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
        let kind_norm =
            domain::normalize_party_roles(&self.ui.text_input(cx, ids!(credit_add_kind)).text());
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_consignor),
            ids!(credit_kind_consignor_on),
            kind_norm == "Consignor",
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_carrier),
            ids!(credit_kind_carrier_on),
            kind_norm == "Carrier",
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_both),
            ids!(credit_kind_both_on),
            kind_norm == "Carrier, Consignor",
        );
        self.paint_choice_pair(
            cx,
            ids!(credit_kind_supplier),
            ids!(credit_kind_supplier_on),
            kind_norm == "Supplier",
        );
        let scope_intl = app_state().lock().unwrap().booking_scope_international;
        self.paint_choice_pair(
            cx,
            ids!(booking_scope_domestic),
            ids!(booking_scope_domestic_on),
            !scope_intl,
        );
        self.paint_choice_pair(
            cx,
            ids!(booking_scope_intl),
            ids!(booking_scope_intl_on),
            scope_intl,
        );
        let (insured, buy_kind, route_key) = {
            let st = app_state().lock().unwrap();
            (
                st.booking_insurance,
                st.buy_party_kind.clone(),
                st.printer_route_key.clone(),
            )
        };
        self.paint_choice_pair(
            cx,
            ids!(booking_insurance_no),
            ids!(booking_insurance_no_on),
            !insured,
        );
        self.paint_choice_pair(
            cx,
            ids!(booking_insurance_yes),
            ids!(booking_insurance_yes_on),
            insured,
        );
        self.paint_choice_pair(
            cx,
            ids!(buy_kind_supplier),
            ids!(buy_kind_supplier_on),
            buy_kind.eq_ignore_ascii_case("Supplier"),
        );
        self.paint_choice_pair(
            cx,
            ids!(buy_kind_carrier),
            ids!(buy_kind_carrier_on),
            buy_kind.eq_ignore_ascii_case("Carrier"),
        );
        self.paint_choice_pair(
            cx,
            ids!(route_pick_receipt),
            ids!(route_pick_receipt_on),
            route_key.eq_ignore_ascii_case("receipt"),
        );
        self.paint_choice_pair(
            cx,
            ids!(route_pick_label),
            ids!(route_pick_label_on),
            route_key.eq_ignore_ascii_case("label"),
        );
        {
            let lang = app_state().lock().unwrap().locale.language.clone();
            let scope_hint = if scope_intl {
                locale::t(&lang, "booking_scope_hint_intl")
            } else {
                locale::t(&lang, "booking_scope_hint_domestic")
            };
            self.ui
                .widget(cx, ids!(booking_scope_hint))
                .set_text(cx, &scope_hint);
            self.ui
                .widget(cx, ids!(booking_scope_help))
                .as_info_help()
                .set_tip(&plain_display_text(&scope_hint));
        }
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
        self.align_label(cx, ids!(label), rtl);
        self.align_label(cx, ids!(qty_label), rtl);
        self.align_label(cx, ids!(line_actual_label), rtl);
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
        self.align_label(cx, ids!(chargeable_kg_label), rtl);
        self.align_label(cx, ids!(last_awb_label), rtl);
        self.align_label(cx, ids!(staff_label), rtl);
        self.align_label(cx, ids!(locale_label), rtl);
        self.align_label(cx, ids!(fiscal_label), rtl);
        self.align_label(cx, ids!(country_hint), rtl);
        self.align_label(cx, ids!(tax_estimate_label), rtl);
        self.align_label(cx, ids!(empty_catalog), rtl);
        self.align_label(cx, ids!(menu_empty), rtl);
        self.align_label(cx, ids!(product_list), rtl);
        self.align_label(cx, ids!(product_list_menu), rtl);
        self.align_label(cx, ids!(cart_empty_hint), rtl);
        self.align_label(cx, ids!(cart_list), rtl);
        self.align_label(cx, ids!(cart_natural_label), rtl);
        self.align_label(cx, ids!(cart_applied_label), rtl);
        self.align_label(cx, ids!(cart_charge_caption), rtl);
        self.align_label(cx, ids!(cart_round_hint), rtl);
        self.align_label(cx, ids!(undo_snack_label), rtl);
        self.align_label(cx, ids!(snack_label_info), rtl);
        self.align_label(cx, ids!(snack_label_ok), rtl);
        self.align_label(cx, ids!(snack_label_warn), rtl);
        self.align_label(cx, ids!(snack_label_err), rtl);
        self.align_label(cx, ids!(credit_chip), rtl);
        self.align_label(cx, ids!(customers_empty_hint), rtl);
        self.align_label(cx, ids!(approval_hint), rtl);
        self.align_label(cx, ids!(sale_selected_label), rtl);
        self.align_label(cx, ids!(sale_attach_hint), rtl);
        self.align_label(cx, ids!(party_attach_hint), rtl);
        self.align_label(cx, ids!(expense_attach_hint), rtl);
        self.align_label(cx, ids!(ops_pod_status), rtl);
        self.align_label(cx, ids!(amend_natural_label), rtl);
        self.align_label(cx, ids!(amend_applied_label), rtl);
        self.align_label(cx, ids!(amend_round_hint), rtl);
        self.align_label(cx, ids!(h_item_photo), rtl);
        self.align_label(cx, ids!(image_status), rtl);
        self.align_label(cx, ids!(draft_preview), rtl);
        self.align_label(cx, ids!(more_mode_hint), rtl);
        self.align_label(cx, ids!(server_pair_label), rtl);
        self.align_label(cx, ids!(pairing_code_value), rtl);
        self.align_label(cx, ids!(settings_tmpl_label), rtl);
        self.align_label(cx, ids!(units_label), rtl);
        self.align_label(cx, ids!(measure_label), rtl);
        self.align_label(cx, ids!(h_onboard_layout), rtl);
        self.align_label(cx, ids!(onboard_tmpl_help), rtl);
        self.align_label(cx, ids!(day_close_label), rtl);
        self.align_label(cx, ids!(books_till_value), rtl);
        self.align_label(cx, ids!(reports_label), rtl);
        self.align_label(cx, ids!(reports_expense_heading), rtl);
        self.align_label(cx, ids!(reports_bank_heading), rtl);
        self.align_label(cx, ids!(inventory_label), rtl);
        self.align_label(cx, ids!(branches_label), rtl);
        self.align_label(cx, ids!(onboard_status), rtl);
        self.align_label(cx, ids!(client_pair_status), rtl);
        self.align_label(cx, ids!(client_queue_label), rtl);
        self.align_label(cx, ids!(setup_lang_list), rtl);
        self.align_label(cx, ids!(language_list), rtl);
        self.align_label(cx, ids!(h_qty_cap), rtl);
        self.align_label(cx, ids!(h_unit_at_cap), rtl);
        self.align_label(cx, ids!(h_applied), rtl);
        self.align_label(cx, ids!(h_local_badge), rtl);
        self.align_label(cx, ids!(h_offline_badge), rtl);
        self.align_label(cx, ids!(h_local_badge_2), rtl);
        self.align_label(cx, ids!(h_counter_locked), rtl);
        self.align_label(cx, ids!(h_shop), rtl);
        self.align_label(cx, ids!(h_who_unlocking), rtl);
        self.align_label(cx, ids!(h_footer_local), rtl);
        self.align_label(cx, ids!(h_setup_app_moklanjo), rtl);
        self.align_label(cx, ids!(h_shop_logo_optional), rtl);
        self.align_label(cx, ids!(h_product_locked_moklanjo), rtl);
        self.align_label(cx, ids!(h_how_start), rtl);
        self.align_label(cx, ids!(h_fresh_book_moklanjo), rtl);
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
        self.align_label(cx, ids!(h_active_ledger), rtl);
        self.align_label(cx, ids!(h_actual_total_cap), rtl);
        self.align_label(cx, ids!(h_applied_total_cap), rtl);
        self.align_label(cx, ids!(h_mix_type_hint_moklanjo), rtl);
        self.align_label(cx, ids!(h_rounded_total), rtl);
        self.align_label(cx, ids!(h_account_kind), rtl);
        self.align_label(cx, ids!(h_permission_preset), rtl);
        self.align_label(cx, ids!(h_preset_help_moklanjo), rtl);
        self.align_label(cx, ids!(h_optional_login_help), rtl);
        self.align_label(cx, ids!(h_feature_access_help), rtl);
        self.align_label(cx, ids!(h_rights_help), rtl);
        self.align_input(cx, ids!(team_pick_input), rtl);
        self.align_label(cx, ids!(h_change_my_pin_heading), rtl);
        self.align_label(cx, ids!(h_duress_heading), rtl);
        self.align_label(cx, ids!(h_duress_help), rtl);
        self.align_label(cx, ids!(duress_status), rtl);
        self.align_label(cx, ids!(h_country_pack_help), rtl);
        self.align_label(cx, ids!(h_language_setup_hint), rtl);
        self.align_label(cx, ids!(h_currency_use_help), rtl);
        self.align_label(cx, ids!(h_product_only_moklanjo), rtl);
        self.align_label(cx, ids!(h_brand_shown_help), rtl);
        self.align_label(cx, ids!(h_units_help), rtl);
        self.align_label(cx, ids!(h_measures_help_moklanjo), rtl);
        self.align_label(cx, ids!(h_restore_help), rtl);
        self.align_label(cx, ids!(h_book_add_product), rtl);
        self.align_label(cx, ids!(h_book_opening_credit), rtl);
        self.align_label(cx, ids!(h_caption_today), rtl);
        self.align_label(cx, ids!(h_caption_bookings), rtl);
        self.align_label(cx, ids!(h_caption_payments_today), rtl);
        self.align_label(cx, ids!(h_caption_outstanding), rtl);
        self.align_label(cx, ids!(h_caption_bank), rtl);
        self.align_label(cx, ids!(h_weight_in), rtl);
        self.align_label(cx, ids!(h_size_in), rtl);
        self.align_label(cx, ids!(h_goods_received), rtl);
        self.align_label(cx, ids!(h_expense_heading), rtl);
        self.align_label(cx, ids!(h_paid_by), rtl);
        self.align_label(cx, ids!(h_second_way), rtl);
        self.align_label(cx, ids!(h_notes_attach_sale), rtl);
        self.align_label(cx, ids!(h_party_custom_heading), rtl);
        self.align_label(cx, ids!(party_custom_help), rtl);
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
        self.align_label(cx, ids!(h_booking_consignor), rtl);
        self.align_label(cx, ids!(booking_consignor_hint), rtl);
        self.align_label(cx, ids!(h_booking_consignee), rtl);
        self.align_label(cx, ids!(h_booking_measures), rtl);
        self.align_label(cx, ids!(h_backup), rtl);
        self.align_label(cx, ids!(h_diag), rtl);
        self.align_label(cx, ids!(diag_privacy_help), rtl);
        self.align_label(cx, ids!(diag_status_label), rtl);
        self.align_label(cx, ids!(h_diag_level), rtl);
        self.align_label(cx, ids!(diag_send_help), rtl);
        self.align_label(cx, ids!(h_printers), rtl);
        self.align_label(cx, ids!(h_printers_help), rtl);
        self.align_label(cx, ids!(printers_found_label), rtl);
        self.align_input(cx, ids!(printer_label_input), rtl);
        self.align_input(cx, ids!(printer_target_input), rtl);
        self.align_input(cx, ids!(route_add_printer_input), rtl);
        self.align_label(cx, ids!(h_drawer_count), rtl);
        self.align_label(cx, ids!(h_paid_by_2), rtl);
        self.align_label(cx, ids!(h_second_way_2), rtl);
        self.align_label(cx, ids!(h_attachments_last_expense), rtl);
        self.align_label(cx, ids!(h_bank_reconciliation), rtl);
        self.align_label(cx, ids!(h_spreadsheet_csv), rtl);
        self.align_label(cx, ids!(h_goods_received_2), rtl);
        self.align_label(cx, ids!(shop_badge), rtl);
        self.align_label(cx, ids!(money_lane_label), rtl);
        self.align_label(cx, ids!(menu_mode_label), rtl);
        self.align_label(cx, ids!(device_setup_banner), rtl);
        self.align_label(cx, ids!(device_use_question), rtl);
        self.align_label(cx, ids!(setup_lang_list), rtl);
        self.align_label(cx, ids!(setup_lang_selected), rtl);
        self.align_label(cx, ids!(settings_lang_selected), rtl);
        self.align_label(cx, ids!(brand_title), rtl);
        self.align_label(cx, ids!(h_venue_icon), rtl);
        self.align_label(cx, ids!(venue_icon_status), rtl);
        self.align_label(cx, ids!(onboard_icon_status), rtl);
        self.align_label(cx, ids!(dev_size_label), rtl);
        self.align_label(cx, ids!(nav_drag_ghost), rtl);
        self.align_label(cx, ids!(nav_drag_ghost_label), rtl);
        self.align_label(cx, ids!(setup_server_list), rtl);
        self.align_label(cx, ids!(sale_list), rtl);
        self.align_label(cx, ids!(amend_cart_list), rtl);
        self.align_label(cx, ids!(backup_path_label), rtl);
    }

    fn align_static_inputs(&mut self, cx: &mut Cx, rtl: bool) {
        self.align_input(cx, ids!(unit_price), rtl);
        self.align_input(cx, ids!(line_total), rtl);
        self.align_input(cx, ids!(note), rtl);
        self.align_input(cx, ids!(booking_actual_kg_input), rtl);
        self.align_input(cx, ids!(booking_length_cm_input), rtl);
        self.align_input(cx, ids!(booking_breadth_cm_input), rtl);
        self.align_input(cx, ids!(booking_height_cm_input), rtl);
        self.align_input(cx, ids!(cart_total_input), rtl);
        self.align_input(cx, ids!(approval_pin_input), rtl);
        self.align_input(cx, ids!(amend_total_input), rtl);
        self.align_input(cx, ids!(menu_name_input), rtl);
        self.align_input(cx, ids!(customer_name_input), rtl);
        self.align_input(cx, ids!(onboard_cust_name), rtl);
        self.align_input(cx, ids!(credit_add_name), rtl);
        self.align_input(cx, ids!(expense_amount_input), rtl);
        self.align_input(cx, ids!(expense_note_input), rtl);
        self.align_input(cx, ids!(bank_amount_input), rtl);
        self.align_input(cx, ids!(bank_note_input), rtl);
        self.align_input(cx, ids!(exp_part2), rtl);
        self.align_input(cx, ids!(bank_clear_input), rtl);
        self.align_input(cx, ids!(org_name_input), rtl);
        self.align_input(cx, ids!(credit_add_kind), rtl);
        self.align_input(cx, ids!(booking_carrier_cost_input), rtl);
        self.align_input(cx, ids!(booking_declared_input), rtl);
        self.align_input(cx, ids!(booking_contents_input), rtl);
        self.align_input(cx, ids!(booking_eway_input), rtl);
        self.align_input(cx, ids!(booking_docket_input), rtl);
        self.align_input(cx, ids!(booking_fuel_input), rtl);
        self.align_input(cx, ids!(booking_zone_input), rtl);
        self.align_input(cx, ids!(booking_carrier_pick_input), rtl);
        self.align_input(cx, ids!(branch_switch_input), rtl);
        self.align_input(cx, ids!(business_tax_id_input), rtl);
        self.align_input(cx, ids!(buy_item_input), rtl);
        self.align_input(cx, ids!(buy_note_input), rtl);
        self.align_input(cx, ids!(buy_supplier_input), rtl);
        self.align_input(cx, ids!(client_code_input), rtl);
        self.align_input(cx, ids!(client_host_input), rtl);
        self.align_input(cx, ids!(client_name_input), rtl);
        self.align_input(cx, ids!(credit_limit_party), rtl);
        self.align_input(cx, ids!(day_close_counted_input), rtl);
        self.align_input(cx, ids!(expense_attach_note), rtl);
        self.align_input(cx, ids!(inv_item_input), rtl);
        self.align_input(cx, ids!(inv_note_input), rtl);
        self.align_input(cx, ids!(inv_supplier_input), rtl);
        self.align_input(cx, ids!(lock_login_id), rtl);
        self.align_input(cx, ids!(lock_login_password), rtl);
        self.align_input(cx, ids!(mix_card_input), rtl);
        self.align_input(cx, ids!(mix_cash_input), rtl);
        self.align_input(cx, ids!(mix_upi_input), rtl);
        self.align_input(cx, ids!(onboard_org_input), rtl);
        self.align_input(cx, ids!(onboard_venue_input), rtl);
        self.align_input(cx, ids!(ops_carrier_input), rtl);
        self.align_input(cx, ids!(ops_note_input), rtl);
        self.align_input(cx, ids!(ops_party_input), rtl);
        self.align_input(cx, ids!(ops_rider_input), rtl);
        self.align_input(cx, ids!(ops_rider_phone), rtl);
        self.align_input(cx, ids!(ops_rider_vehicle), rtl);
        self.align_input(cx, ids!(ops_rider_cod_input), rtl);
        self.align_input(cx, ids!(party_attach_note), rtl);
        self.align_input(cx, ids!(reason_input), rtl);
        self.align_input(cx, ids!(sale_attach_note), rtl);
        self.align_input(cx, ids!(server_name_input), rtl);
        self.align_input(cx, ids!(venue_title_input), rtl);
        self.align_input(cx, ids!(booking_origin_name_input), rtl);
        self.align_input(cx, ids!(booking_origin_phone_input), rtl);
        self.align_input(cx, ids!(booking_origin_address_input), rtl);
        self.align_input(cx, ids!(booking_destination_input), rtl);
        self.align_input(cx, ids!(booking_consignee_input), rtl);
        self.align_input(cx, ids!(booking_consignee_phone_input), rtl);
        self.align_input(cx, ids!(booking_consignee_address_input), rtl);
        self.align_input(cx, ids!(booking_cod_input), rtl);
        self.align_input(cx, ids!(booking_notify_input), rtl);
        self.align_input(cx, ids!(menu_aliases_input), rtl);
        self.align_input(cx, ids!(menu_price_input), rtl);
        self.align_input(cx, ids!(menu_unit_input), rtl);
        self.align_input(cx, ids!(menu_tax_input), rtl);
        self.align_input(cx, ids!(menu_category_input), rtl);
        self.align_input(cx, ids!(menu_modifiers_input), rtl);
        self.align_input(cx, ids!(menu_first_kg_input), rtl);
        self.align_input(cx, ids!(menu_first_amt_input), rtl);
        self.align_input(cx, ids!(menu_extra_kg_input), rtl);
        self.align_input(cx, ids!(menu_zone_input), rtl);
        self.align_input(cx, ids!(menu_carrier_input), rtl);
        self.align_input(cx, ids!(menu_party_input), rtl);
        self.align_input(cx, ids!(buy_qty_input), rtl);
        self.align_input(cx, ids!(buy_cost_input), rtl);
        self.align_input(cx, ids!(buy_expense_amount), rtl);
        self.align_input(cx, ids!(buy_expense_note), rtl);
        self.align_input(cx, ids!(buy_exp_part2), rtl);
        self.align_input(cx, ids!(mix_credit_input), rtl);
        self.align_input(cx, ids!(credit_add_phone), rtl);
        self.align_input(cx, ids!(credit_add_address), rtl);
        self.align_input(cx, ids!(party_country_input), rtl);
        self.align_input(cx, ids!(party_state_input), rtl);
        self.align_input(cx, ids!(party_pincode_input), rtl);
        self.align_input(cx, ids!(party_custom_key), rtl);
        self.align_input(cx, ids!(party_custom_value), rtl);
        self.align_input(cx, ids!(credit_limit_input), rtl);
        self.align_input(cx, ids!(writeoff_amount_input), rtl);
        self.align_input(cx, ids!(writeoff_reason_input), rtl);
        self.align_input(cx, ids!(payable_amount_input), rtl);
        self.align_input(cx, ids!(payable_bill_input), rtl);
        self.align_input(cx, ids!(settle_amount_input), rtl);
        self.align_input(cx, ids!(settle_bill_input), rtl);
        self.align_input(cx, ids!(booking_bulk_input), rtl);
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
        self.align_input(cx, ids!(ops_awb_input), rtl);
        self.align_input(cx, ids!(inv_qty_input), rtl);
        self.align_input(cx, ids!(inv_cost_input), rtl);
        self.align_input(cx, ids!(branch_name_input), rtl);
        self.align_input(cx, ids!(branch_code_input), rtl);
        self.align_input(cx, ids!(unit_code_input), rtl);
        self.align_input(cx, ids!(unit_name_input), rtl);
        self.align_input(cx, ids!(conv_from_input), rtl);
        self.align_input(cx, ids!(conv_to_input), rtl);
        self.align_input(cx, ids!(conv_factor_input), rtl);
        self.align_input(cx, ids!(measure_dim_input), rtl);
        self.align_input(cx, ids!(measure_weight_input), rtl);
        self.align_input(cx, ids!(measure_divisor_input), rtl);
        self.align_input(cx, ids!(measure_slab_input), rtl);
    }
}

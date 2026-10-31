impl App {
fn paint_static_texts(&mut self, cx: &mut Cx) {
        let (lang, country_code, party_country) = {
            let st = app_state().lock().unwrap();
            (
                st.locale.language.clone(),
                st.fiscal.country_code.clone(),
                st.party_form_country.clone(),
            )
        };
        let dial_country = {
            let from_party = country_code_from_label(&party_country);
            if from_party.is_empty() {
                country_code.clone()
            } else {
                from_party
            }
        };
        let s = |key: &str| locale::t(&lang, key);
        self.ui
            .widget(cx, ids!(h_qty_cap))
            .set_text(cx, &plain_display_text(&s("qty_cap")));
        self.ui
            .widget(cx, ids!(h_unit_at_cap))
            .set_text(cx, &plain_display_text(&s("unit_at_cap")));
        self.ui
            .widget(cx, ids!(h_applied))
            .set_text(cx, &plain_display_text(&s("applied")));
        self.ui
            .widget(cx, ids!(h_local_badge))
            .set_text(cx, &plain_display_text(&s("local_badge")));
        self.ui
            .widget(cx, ids!(h_offline_badge))
            .set_text(cx, &plain_display_text(&s("offline_badge")));
        self.ui
            .widget(cx, ids!(h_local_badge_2))
            .set_text(cx, &plain_display_text(&s("local_badge")));
        self.ui
            .widget(cx, ids!(h_counter_locked))
            .set_text(cx, &plain_display_text(&s("counter_locked")));
        self.ui
            .widget(cx, ids!(h_shop))
            .set_text(cx, &plain_display_text(&s("shop")));
        self.ui
            .widget(cx, ids!(h_who_unlocking))
            .set_text(cx, &plain_display_text(&s("who_unlocking")));
        self.ui
            .widget(cx, ids!(h_footer_local))
            .set_text(cx, &plain_display_text(&s("footer_local")));
        self.ui
            .widget(cx, ids!(h_setup_app_moklanjo))
            .set_text(cx, &plain_display_text(&s("setup_app_moklanjo")));
        self.ui
            .widget(cx, ids!(h_shop_logo_optional))
            .set_text(cx, &plain_display_text(&s("shop_logo_optional")));
        self.ui
            .widget(cx, ids!(h_onboard_layout))
            .set_text(cx, &plain_display_text(&s("onboard_layout")));
        self.ui
            .widget(cx, ids!(h_venue_icon))
            .set_text(cx, &plain_display_text(&s("app_icon")));
        self.ui
            .widget(cx, ids!(h_item_photo))
            .set_text(cx, &plain_display_text(&s("photo")));
        self.ui
            .widget(cx, ids!(h_product_locked_moklanjo))
            .set_text(cx, &plain_display_text(&s("product_locked_moklanjo")));
        self.ui
            .widget(cx, ids!(h_how_start))
            .set_text(cx, &plain_display_text(&s("how_start")));
        self.ui
            .widget(cx, ids!(h_fresh_book_moklanjo))
            .set_text(cx, &plain_display_text(&s("fresh_book_moklanjo")));
        self.ui
            .widget(cx, ids!(h_fresh_selected))
            .set_text(cx, &plain_display_text(&s("fresh_selected")));
        self.ui
            .widget(cx, ids!(h_later_country_pack))
            .set_text(cx, &plain_display_text(&s("later_country_pack")));
        self.ui
            .widget(cx, ids!(h_connect_to_server))
            .set_text(cx, &plain_display_text(&s("connect_to_server")));
        self.ui
            .widget(cx, ids!(h_money_in_sales))
            .set_text(cx, &plain_display_text(&s("money_in_sales")));
        self.ui
            .widget(cx, ids!(h_purchases_cogs))
            .set_text(cx, &plain_display_text(&s("purchases_cogs")));
        self.ui
            .widget(cx, ids!(h_expenses_word))
            .set_text(cx, &plain_display_text(&s("expenses_word")));
        self.ui
            .widget(cx, ids!(h_gross_word))
            .set_text(cx, &plain_display_text(&s("gross_word")));
        self.ui
            .widget(cx, ids!(h_net_word))
            .set_text(cx, &plain_display_text(&s("net_word")));
        self.ui
            .widget(cx, ids!(h_cash))
            .set_text(cx, &plain_display_text(&s("cash")));
        self.ui
            .widget(cx, ids!(h_card))
            .set_text(cx, &plain_display_text(&s("card")));
        self.ui
            .widget(cx, ids!(h_they_owe_us))
            .set_text(cx, &plain_display_text(&s("they_owe_us")));
        self.ui
            .widget(cx, ids!(h_we_owe_them))
            .set_text(cx, &plain_display_text(&s("we_owe_them")));
        self.ui
            .widget(cx, ids!(h_bank_balance))
            .set_text(cx, &plain_display_text(&s("bank_balance")));
        self.ui
            .widget(cx, ids!(h_cash_in_hand))
            .set_text(cx, &plain_display_text(&s("cash_in_hand")));
        self.ui
            .widget(cx, ids!(h_bank_net_today))
            .set_text(cx, &plain_display_text(&s("bank_net_today")));
        self.ui
            .widget(cx, ids!(h_local_badge_3))
            .set_text(cx, &plain_display_text(&s("local_badge")));
        self.ui
            .widget(cx, ids!(h_active_ledger))
            .set_text(cx, &plain_display_text(&s("active_ledger")));
        self.ui
            .widget(cx, ids!(h_actual_total_cap))
            .set_text(cx, &plain_display_text(&s("actual_total_cap")));
        self.ui
            .widget(cx, ids!(h_applied_total_cap))
            .set_text(cx, &plain_display_text(&s("applied_total_cap")));
        self.ui
            .widget(cx, ids!(h_mix_type_hint_moklanjo))
            .set_text(cx, &plain_display_text(&s("mix_type_hint_moklanjo")));
        self.ui
            .widget(cx, ids!(payment_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("mix_type_hint_moklanjo")));
        self.ui
            .widget(cx, ids!(h_rounded_total))
            .set_text(cx, &plain_display_text(&s("rounded_total")));
        self.ui
            .widget(cx, ids!(h_account_kind))
            .set_text(cx, &plain_display_text(&s("account_kind")));
        self.ui
            .widget(cx, ids!(h_permission_preset))
            .set_text(cx, &plain_display_text(&s("permission_preset")));
        self.ui
            .widget(cx, ids!(h_preset_help_moklanjo))
            .set_text(cx, &plain_display_text(&s("preset_help_moklanjo")));
        {
            let help = self.ui.widget(cx, ids!(h_optional_login_help)).as_info_help();
            help.set_title(cx, &s("optional_login_heading"));
            help.set_tip(&plain_display_text(&s("optional_login_help")));
        }
        self.ui
            .widget(cx, ids!(h_feature_access_help))
            .set_text(cx, &plain_display_text(&s("feature_access_help")));
        {
            let help = self.ui.widget(cx, ids!(h_rights_help)).as_info_help();
            help.set_title(cx, &s("rights_heading"));
            help.set_tip(&plain_display_text(&s("rights_help")));
        }
        self.ui
            .widget(cx, ids!(h_change_my_pin_heading))
            .set_text(cx, &plain_display_text(&s("change_my_pin_heading")));
        self.ui
            .widget(cx, ids!(h_duress_heading))
            .set_text(cx, &plain_display_text(&s("duress_heading")));
        {
            let help = self.ui.widget(cx, ids!(h_duress_help)).as_info_help();
            // Title is the sibling WrapLabel on the same Right row.
            help.set_title(cx, "");
            help.set_tip(&plain_display_text(&s("duress_help")));
        }
        self.ui
            .button(cx, ids!(save_duress_pin))
            .set_text(cx, &s("save_duress_pin"));
        self.ui
            .button(cx, ids!(clear_duress_pin))
            .set_text(cx, &s("clear_duress_pin"));
        self.ui
            .widget(cx, ids!(h_country_pack_help))
            .set_text(cx, &plain_display_text(&s("country_pack_help")));
        self.ui
            .widget(cx, ids!(h_language_setup_hint))
            .set_text(cx, &plain_display_text(&s("language_setup_hint")));
        self.ui
            .widget(cx, ids!(h_currency_use_help))
            .set_text(cx, &plain_display_text(&s("currency_use_help")));
        self.ui
            .widget(cx, ids!(h_product_only_moklanjo))
            .set_text(cx, &plain_display_text(&s("product_only_moklanjo")));
        self.ui
            .widget(cx, ids!(h_brand_shown_help))
            .set_text(cx, &plain_display_text(&s("brand_shown_help")));
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
            .widget(cx, ids!(h_units_help))
            .set_text(cx, &plain_display_text(&s("units_help")));
        self.ui
            .widget(cx, ids!(h_measures_help_moklanjo))
            .set_text(cx, &plain_display_text(&s("measures_help_moklanjo")));
        self.ui
            .widget(cx, ids!(h_restore_help))
            .set_text(cx, &plain_display_text(&s("restore_help")));
        {
            let help = self.ui.widget(cx, ids!(day_close_help)).as_info_help();
            help.set_title(cx, &s("day_close"));
            help.set_nav_icon(cx, NavTab::DayClose);
            help.set_tip(&plain_display_text(&s("day_close_help")));
        }
        self.ui
            .widget(cx, ids!(h_book_add_product))
            .set_text(cx, &plain_display_text(&s("book_add_product")));
        self.ui
            .widget(cx, ids!(h_book_opening_credit))
            .set_text(cx, &plain_display_text(&s("book_opening_credit")));
        self.ui
            .widget(cx, ids!(h_caption_today))
            .set_text(cx, &plain_display_text(&s("caption_today")));
        self.ui
            .widget(cx, ids!(h_caption_bookings))
            .set_text(cx, &plain_display_text(&s("caption_bookings")));
        self.ui
            .widget(cx, ids!(h_caption_payments_today))
            .set_text(cx, &plain_display_text(&s("caption_payments_today")));
        self.ui
            .widget(cx, ids!(h_caption_outstanding))
            .set_text(cx, &plain_display_text(&s("caption_outstanding")));
        self.ui
            .widget(cx, ids!(h_caption_bank))
            .set_text(cx, &plain_display_text(&s("caption_bank")));
        self.ui
            .widget(cx, ids!(h_weight_in))
            .set_text(cx, &plain_display_text(&s("weight_in")));
        self.ui
            .widget(cx, ids!(h_size_in))
            .set_text(cx, &plain_display_text(&s("size_in")));
        self.ui
            .widget(cx, ids!(h_goods_received))
            .set_text(cx, &plain_display_text(&s("goods_received")));
        self.ui
            .widget(cx, ids!(h_expense_heading))
            .set_text(cx, &plain_display_text(&s("expense_heading")));
        self.ui
            .widget(cx, ids!(h_paid_by))
            .set_text(cx, &plain_display_text(&s("paid_by")));
        self.ui
            .widget(cx, ids!(h_second_way))
            .set_text(cx, &plain_display_text(&s("second_way")));
        self.ui
            .widget(cx, ids!(h_notes_attach_sale))
            .set_text(cx, &plain_display_text(&s("notes_attach_sale")));
        self.ui
            .widget(cx, ids!(h_party_custom_heading))
            .set_text(cx, &plain_display_text(&s("party_custom_fields_heading")));
        self.ui
            .widget(cx, ids!(party_custom_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("party_custom_help")));
        self.ui
            .button(cx, ids!(party_custom_add))
            .set_text(cx, &s("party_custom_add"));
        self.ui
            .button(cx, ids!(party_custom_remove))
            .set_text(cx, &s("party_custom_remove"));
        self.ui
            .widget(cx, ids!(h_credit_limit_heading))
            .set_text(cx, &plain_display_text(&s("credit_limit_heading")));
        self.ui
            .widget(cx, ids!(h_pay_selected_heading))
            .set_text(cx, &plain_display_text(&s("pay_selected_heading")));
        self.ui
            .widget(cx, ids!(h_write_off_heading))
            .set_text(cx, &plain_display_text(&s("write_off_heading")));
        self.ui
            .widget(cx, ids!(h_pay_ap_heading))
            .set_text(cx, &plain_display_text(&s("pay_ap_heading")));
        self.ui
            .widget(cx, ids!(h_notes_attach_party))
            .set_text(cx, &plain_display_text(&s("notes_attach_party")));
        self.ui
            .widget(cx, ids!(h_country_tax))
            .set_text(cx, &plain_display_text(&s("country_tax")));
        self.ui
            .widget(cx, ids!(h_language))
            .set_text(cx, &plain_display_text(&s("language")));
        self.ui
            .widget(cx, ids!(h_currency))
            .set_text(cx, &plain_display_text(&s("currency")));
        self.ui
            .widget(cx, ids!(h_extra_tills_heading))
            .set_text(cx, &plain_display_text(&s("extra_tills_heading")));
        self.ui
            .widget(cx, ids!(h_other_devices_help_heading))
            .set_text(cx, &plain_display_text(&s("other_devices_help_heading")));
        self.ui
            .widget(cx, ids!(h_other_devices_help))
            .set_text(cx, &plain_display_text(&s("other_devices_help")));
        self.ui
            .widget(cx, ids!(h_start_template))
            .set_text(cx, &plain_display_text(&s("start_template")));
        self.ui
            .widget(cx, ids!(h_your_brand))
            .set_text(cx, &plain_display_text(&s("your_brand")));
        self.ui
            .widget(cx, ids!(h_units_heading))
            .set_text(cx, &plain_display_text(&s("units_heading")));
        self.ui
            .widget(cx, ids!(h_booking_measures))
            .set_text(cx, &plain_display_text(&s("booking_measures")));
        self.ui
            .widget(cx, ids!(h_backup))
            .set_text(cx, &plain_display_text(&s("backup")));
        self.ui
            .widget(cx, ids!(h_printers))
            .set_text(cx, &plain_display_text(&s("printers_heading")));
        self.ui
            .widget(cx, ids!(h_printers_help))
            .set_text(cx, &plain_display_text(&s("printers_help")));
        self.ui.widget(cx, ids!(h_printers_pool)).set_text(cx, &plain_display_text(&s("printers_pool_heading")));
        self.ui.widget(cx, ids!(h_printers_routes)).set_text(cx, &plain_display_text(&s("printers_routes_heading")));
        self.ui.button(cx, ids!(find_printers_btn)).set_text(cx, &s("find_printers"));
        self.ui.button(cx, ids!(printer_add)).set_text(cx, &s("add_printer"));
        self.ui.button(cx, ids!(printer_remove)).set_text(cx, &s("remove_printer"));
        self.ui.button(cx, ids!(route_add)).set_text(cx, &s("route_add"));
        self.ui.button(cx, ids!(route_up)).set_text(cx, &s("route_up"));
        self.ui.button(cx, ids!(route_down)).set_text(cx, &s("route_down"));
        self.ui.button(cx, ids!(route_remove)).set_text(cx, &s("route_remove"));
        self.ui.button(cx, ids!(route_clear)).set_text(cx, &s("route_clear"));
        self.ui.button(cx, ids!(printers_save)).set_text(cx, &s("save_printers"));
        self.ui.button(cx, ids!(print_test_btn)).set_text(cx, &s("print_test_page"));
        self.ui
            .text_input(cx, ids!(printer_label_input))
            .set_empty_text(cx, s("printer_label_ph"));
        self.ui
            .text_input(cx, ids!(printer_target_input))
            .set_empty_text(cx, s("printer_target_ph"));
        self.ui
            .text_input(cx, ids!(route_add_printer_input))
            .set_empty_text(cx, s("route_add_printer_ph"));
        self.ui
            .widget(cx, ids!(h_drawer_count))
            .set_text(cx, &plain_display_text(&s("drawer_count")));
        self.ui
            .widget(cx, ids!(h_paid_by_2))
            .set_text(cx, &plain_display_text(&s("paid_by")));
        self.ui
            .widget(cx, ids!(h_second_way_2))
            .set_text(cx, &plain_display_text(&s("second_way")));
        self.ui
            .widget(cx, ids!(h_attachments_last_expense))
            .set_text(cx, &plain_display_text(&s("attachments_last_expense")));
        self.ui
            .widget(cx, ids!(h_bank_reconciliation))
            .set_text(cx, &plain_display_text(&s("bank_reconciliation")));
        self.ui
            .widget(cx, ids!(h_spreadsheet_csv))
            .set_text(cx, &plain_display_text(&s("spreadsheet_csv")));
        self.ui
            .widget(cx, ids!(h_goods_received_2))
            .set_text(cx, &plain_display_text(&s("goods_received")));
        self.ui
            .button(cx, ids!(allow_extra_tills))
            .set_text(cx, &s("allow_extra_tills"));
        self.ui
            .button(cx, ids!(amend_btn))
            .set_text(cx, &s("amend"));
        self.ui
            .button(cx, ids!(backup_btn))
            .set_text(cx, &s("create_backup"));
        self.ui
            .button(cx, ids!(bank_bank_to_till))
            .set_text(cx, &s("bank_to_till"));
        self.ui
            .button(cx, ids!(bank_in))
            .set_text(cx, &s("bank_in"));
        self.ui
            .button(cx, ids!(bank_mark_cleared))
            .set_text(cx, &s("mark_cleared"));
        self.ui
            .button(cx, ids!(bank_mark_uncleared))
            .set_text(cx, &s("unmark"));
        self.ui
            .button(cx, ids!(bank_out))
            .set_text(cx, &s("bank_out"));
        self.ui
            .button(cx, ids!(bank_till_to_bank))
            .set_text(cx, &s("till_to_bank"));
        self.ui
            .button(cx, ids!(booking_bulk_load))
            .set_text(cx, &s("load_bulk_row"));
        self.ui
            .text_input(cx, ids!(booking_carrier_cost_input))
            .set_empty_text(cx, s("carrier_cost_ph"));
        self.ui
            .text_input(cx, ids!(booking_docket_input))
            .set_empty_text(cx, s("docket_charge_ph"));
        self.ui
            .text_input(cx, ids!(booking_fuel_input))
            .set_empty_text(cx, s("fuel_ph"));
        self.ui
            .text_input(cx, ids!(booking_zone_input))
            .set_empty_text(cx, s("zone_ph"));
        self.ui
            .button(cx, ids!(branch_add))
            .set_text(cx, &s("add_branch"));
        self.ui
            .button(cx, ids!(branch_consolidated))
            .set_text(cx, &s("rollup"));
        self.ui
            .button(cx, ids!(branch_export))
            .set_text(cx, &s("export_day"));
        self.ui
            .button(cx, ids!(branch_import))
            .set_text(cx, &s("import_day"));
        self.ui
            .button(cx, ids!(branch_switch))
            .set_text(cx, &s("switch_device"));
        self.ui
            .text_input(cx, ids!(branch_switch_input))
            .set_empty_text(cx, s("branch_name_sell_ph"));
        self.ui
            .text_input(cx, ids!(business_tax_id_input))
            .set_empty_text(cx, s("business_tax_id_ph"));
        self.ui
            .button(cx, ids!(buy_exp_t1_bank))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(buy_exp_t1_bank_on))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(buy_exp_t1_card))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(buy_exp_t1_card_on))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(buy_exp_t1_cash))
            .set_text(cx, &s("cash"));
        self.ui
            .button(cx, ids!(buy_exp_t1_cash_on))
            .set_text(cx, &s("cash"));
        self.ui
            .button(cx, ids!(buy_exp_t1_upi))
            .set_text(cx, &s("upi"));
        self.ui
            .button(cx, ids!(buy_exp_t1_upi_on))
            .set_text(cx, &s("upi"));
        self.ui
            .button(cx, ids!(buy_exp_t2_bank))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(buy_exp_t2_bank_on))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(buy_exp_t2_card))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(buy_exp_t2_card_on))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(buy_exp_t2_cash))
            .set_text(cx, &s("cash"));
        self.ui
            .button(cx, ids!(buy_exp_t2_cash_on))
            .set_text(cx, &s("cash"));
        self.ui
            .button(cx, ids!(buy_exp_t2_upi))
            .set_text(cx, &s("upi"));
        self.ui
            .button(cx, ids!(buy_exp_t2_upi_on))
            .set_text(cx, &s("upi"));
        self.ui
            .text_input(cx, ids!(buy_item_input))
            .set_empty_text(cx, s("item_name"));
        self.ui
            .text_input(cx, ids!(buy_note_input))
            .set_empty_text(cx, s("note_ph"));
        self.ui
            .button(cx, ids!(buy_on_account))
            .set_text(cx, &s("on_account_ap"));
        self.ui
            .button(cx, ids!(buy_on_account_on))
            .set_text(cx, &s("on_account_ap"));
        self.ui
            .button(cx, ids!(buy_paid))
            .set_text(cx, &s("paid_now"));
        self.ui
            .button(cx, ids!(buy_paid_on))
            .set_text(cx, &s("paid_now"));
        self.ui
            .text_input(cx, ids!(buy_supplier_input))
            .set_empty_text(cx, s("supplier_name_ph"));
        self.ui
            .button(cx, ids!(change_my_password))
            .set_text(cx, &s("update_my_password"));
        self.ui
            .button(cx, ids!(change_my_pin))
            .set_text(cx, &s("update_my_pin"));
        self.ui
            .text_input(cx, ids!(client_code_input))
            .set_empty_text(cx, s("pairing_code_ph"));
        self.ui
            .text_input(cx, ids!(client_host_input))
            .set_empty_text(cx, s("server_ip_ph"));
        self.ui
            .button(cx, ids!(client_pair_btn))
            .set_text(cx, &s("pair_now"));
        self.ui
            .button(cx, ids!(client_find_btn))
            .set_text(cx, &s("find_server"));
        self.ui
            .button(cx, ids!(client_skip_lock))
            .set_text(cx, &s("unlock_after_pairing"));
        self.ui
            .button(cx, ids!(client_sync_btn))
            .set_text(cx, &s("refresh_from_shop"));
        self.ui
            .button(cx, ids!(client_unpair))
            .set_text(cx, &s("forget_shop"));
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
        self.ui
            .button(cx, ids!(conv_add))
            .set_text(cx, &s("add_conversion"));
        self.ui
            .button(cx, ids!(country_apply))
            .set_text(cx, &s("apply_country_pack"));
        self.ui
            .button(cx, ids!(country_next))
            .set_text(cx, &s("next_country"));
        self.ui
            .button(cx, ids!(country_prev))
            .set_text(cx, &s("prev_country"));
        self.ui
            .button(cx, ids!(credit_kind_both))
            .set_text(cx, &s("role_both"));
        self.ui
            .button(cx, ids!(credit_kind_both_on))
            .set_text(cx, &s("role_both"));
        self.ui
            .button(cx, ids!(credit_kind_carrier))
            .set_text(cx, &s("role_carrier"));
        self.ui
            .button(cx, ids!(credit_kind_carrier_on))
            .set_text(cx, &s("role_carrier"));
        self.ui
            .button(cx, ids!(credit_kind_consignor))
            .set_text(cx, &s("role_consignor"));
        self.ui
            .button(cx, ids!(credit_kind_consignor_on))
            .set_text(cx, &s("role_consignor"));
        self.ui
            .button(cx, ids!(credit_kind_supplier))
            .set_text(cx, &s("role_supplier"));
        self.ui
            .button(cx, ids!(credit_kind_supplier_on))
            .set_text(cx, &s("role_supplier"));
        self.ui
            .text_input(cx, ids!(credit_limit_party))
            .set_empty_text(cx, s("party_name_selected_ph"));
        self.ui
            .button(cx, ids!(credit_limit_set))
            .set_text(cx, &s("set_credit_limit"));
        self.ui
            .button(cx, ids!(credit_period_bill))
            .set_text(cx, &s("period_bill"));
        self.ui
            .button(cx, ids!(credit_writeoff))
            .set_text(cx, &s("write_off_nil"));
        self.ui
            .button(cx, ids!(csv_copy_day))
            .set_text(cx, &s("copy_day_book_csv"));
        self.ui
            .button(cx, ids!(csv_copy_items))
            .set_text(cx, &s("copy_items_csv"));
        self.ui
            .button(cx, ids!(csv_copy_ledger))
            .set_text(cx, &s("copy_party_ledger_csv"));
        self.ui
            .button(cx, ids!(csv_copy_outstanding))
            .set_text(cx, &s("copy_outstanding_csv"));
        self.ui
            .button(cx, ids!(csv_copy_parties))
            .set_text(cx, &s("copy_parties_csv"));
        self.ui
            .button(cx, ids!(csv_import_items))
            .set_text(cx, &s("import_items_csv"));
        self.ui
            .button(cx, ids!(csv_import_parties))
            .set_text(cx, &s("import_parties_csv"));
        self.ui
            .button(cx, ids!(csv_import_pincodes))
            .set_text(cx, &s("import_pincodes_csv"));
        self.ui
            .button(cx, ids!(csv_import_carrier_bill))
            .set_text(cx, &s("import_carrier_bill_csv"));
        self.ui
            .button(cx, ids!(csv_copy_carrier_bill))
            .set_text(cx, &s("copy_carrier_bill_report"));
        self.ui
            .button(cx, ids!(booking_apply_carrier_cost))
            .set_text(cx, &s("use_carrier_cost"));
        self.ui
            .button(cx, ids!(customer_use))
            .set_text(cx, &s("add_use_name"));
        self.ui
            .button(cx, ids!(day_close_commit))
            .set_text(cx, &s("record_day_close"));
        self.ui
            .button(cx, ids!(day_close_copy))
            .set_text(cx, &s("copy_day_close"));
        self.ui
            .text_input(cx, ids!(day_close_counted_input))
            .set_empty_text(cx, s("counted_cash_ph"));
        self.ui
            .button(cx, ids!(day_close_from_sales))
            .set_text(cx, &s("day_close_summary"));
        self.ui
            .button(cx, ids!(exp_t1_bank))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(exp_t1_bank_on))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(exp_t1_card))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(exp_t1_card_on))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(exp_t1_cash))
            .set_text(cx, &s("cash"));
        self.ui
            .button(cx, ids!(exp_t1_cash_on))
            .set_text(cx, &s("cash"));
        self.ui.button(cx, ids!(exp_t1_upi)).set_text(cx, &s("upi"));
        self.ui
            .button(cx, ids!(exp_t1_upi_on))
            .set_text(cx, &s("upi"));
        self.ui
            .button(cx, ids!(exp_t2_bank))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(exp_t2_bank_on))
            .set_text(cx, &s("bank"));
        self.ui
            .button(cx, ids!(exp_t2_card))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(exp_t2_card_on))
            .set_text(cx, &s("card"));
        self.ui
            .button(cx, ids!(exp_t2_cash))
            .set_text(cx, &s("cash"));
        self.ui
            .button(cx, ids!(exp_t2_cash_on))
            .set_text(cx, &s("cash"));
        self.ui.button(cx, ids!(exp_t2_upi)).set_text(cx, &s("upi"));
        self.ui
            .button(cx, ids!(exp_t2_upi_on))
            .set_text(cx, &s("upi"));
        self.ui
            .button(cx, ids!(expense_attach_add_note))
            .set_text(cx, &s("add_note"));
        self.ui
            .text_input(cx, ids!(expense_attach_note))
            .set_empty_text(cx, s("extra_note_expense_ph"));
        self.ui
            .button(cx, ids!(expense_attach_pdf))
            .set_text(cx, &s("attach_pdf"));
        self.ui
            .button(cx, ids!(expense_attach_photo))
            .set_text(cx, &s("attach_bill_photo"));
        self.ui
            .button(cx, ids!(feat_backup))
            .set_text(cx, &s("backup"));
        self.ui.button(cx, ids!(feat_backup_on)).set_text(cx, &s("backup"));
        self.ui
            .button(cx, ids!(feat_branches))
            .set_text(cx, &s("branches"));
        self.ui.button(cx, ids!(feat_branches_on)).set_text(cx, &s("branches"));
        self.ui
            .button(cx, ids!(feat_credit))
            .set_text(cx, &s("credit"));
        self.ui.button(cx, ids!(feat_credit_on)).set_text(cx, &s("credit"));
        self.ui
            .button(cx, ids!(feat_inventory))
            .set_text(cx, &s("inventory"));
        self.ui.button(cx, ids!(feat_inventory_on)).set_text(cx, &s("inventory"));
        self.ui
            .button(cx, ids!(feat_items))
            .set_text(cx, &s("items"));
        self.ui.button(cx, ids!(feat_items_on)).set_text(cx, &s("items"));
        self.ui
            .button(cx, ids!(feat_sales))
            .set_text(cx, &s("sales"));
        self.ui.button(cx, ids!(feat_sales_on)).set_text(cx, &s("sales"));
        self.ui
            .button(cx, ids!(feat_save))
            .set_text(cx, &s("save_features"));
        self.ui.button(cx, ids!(feat_sell)).set_text(cx, &s("sell"));
        self.ui.button(cx, ids!(feat_sell_on)).set_text(cx, &s("sell"));
        self.ui.button(cx, ids!(feat_team)).set_text(cx, &s("team"));
        self.ui.button(cx, ids!(feat_team_on)).set_text(cx, &s("team"));
        self.ui
            .button(cx, ids!(fiscal_save))
            .set_text(cx, &s("save_business_id"));
        self.ui
            .text_input(cx, ids!(inv_item_input))
            .set_empty_text(cx, s("item_name"));
        self.ui
            .text_input(cx, ids!(inv_note_input))
            .set_empty_text(cx, s("reason_note_ph"));
        self.ui
            .text_input(cx, ids!(inv_supplier_input))
            .set_empty_text(cx, s("supplier_ph"));
        self.ui
            .text_input(cx, ids!(lock_login_id))
            .set_empty_text(cx, s("login_id_ph"));
        self.ui
            .button(cx, ids!(lock_login_ok))
            .set_text(cx, &s("unlock_with_login"));
        self.ui
            .text_input(cx, ids!(lock_login_password))
            .set_empty_text(cx, s("password_ph"));
        self.ui
            .button(cx, ids!(measure_save))
            .set_text(cx, &s("save_booking_measures"));
        self.ui
            .text_input(cx, ids!(mix_card_input))
            .set_empty_text(cx, s("card_amount_ph"));
        self.ui
            .text_input(cx, ids!(mix_cash_input))
            .set_empty_text(cx, s("cash_amount_ph"));
        self.ui
            .text_input(cx, ids!(mix_upi_input))
            .set_empty_text(cx, s("upi_amount_ph"));
        self.ui
            .button(cx, ids!(more_cod))
            .set_text(cx, &s("cod_register"));
        self.ui
            .button(cx, ids!(more_ops))
            .set_text(cx, &s("consignments_dispatch"));
        self.ui
            .button(cx, ids!(more_riders))
            .set_text(cx, &s("riders_own_fleet"));
        self.ui
            .button(cx, ids!(next_booking_btn))
            .set_text(cx, &s("next_booking"));
        self.ui
            .button(cx, ids!(onboard_add_credit))
            .set_text(cx, &s("add_opening_balance"));
        self.ui
            .button(cx, ids!(onboard_add_item))
            .set_text(cx, &s("add_item_plain"));
        self.ui
            .button(cx, ids!(onboard_book))
            .set_text(cx, &s("bring_my_book"));
        self.ui
            .button(cx, ids!(onboard_book_on))
            .set_text(cx, &s("bring_my_book"));
        self.ui
            .button(cx, ids!(onboard_done))
            .set_text(cx, &s("done_open_books"));
        self.ui
            .button(cx, ids!(onboard_fresh))
            .set_text(cx, &s("start_fresh"));
        self.ui
            .button(cx, ids!(onboard_fresh_on))
            .set_text(cx, &s("start_fresh"));
        self.ui
            .button(cx, ids!(onboard_icon_pick))
            .set_text(cx, &s("choose_logo"));
        self.ui
            .text_input(cx, ids!(onboard_org_input))
            .set_empty_text(cx, s("org_optional_ph"));
        self.ui
            .button(cx, ids!(onboard_save_venue))
            .set_text(cx, &s("save_names"));
        self.ui
            .text_input(cx, ids!(onboard_venue_input))
            .set_empty_text(cx, s("counter_agency_ph"));
        self.ui
            .button(cx, ids!(ops_add_rider))
            .set_text(cx, &s("add_rider"));
        self.ui
            .button(cx, ids!(ops_advance_delivered))
            .set_text(cx, &s("mark_delivered"));
        self.ui
            .button(cx, ids!(ops_advance_ofd))
            .set_text(cx, &s("out_for_delivery"));
        self.ui
            .button(cx, ids!(ops_ndr))
            .set_text(cx, &s("failed_delivery"));
        self.ui
            .button(cx, ids!(ops_reattempt))
            .set_text(cx, &s("reattempt"));
        self.ui
            .button(cx, ids!(ops_ndr_door))
            .set_text(cx, &s("ndr_door_locked"));
        self.ui
            .button(cx, ids!(ops_ndr_refused))
            .set_text(cx, &s("ndr_refused"));
        self.ui
            .button(cx, ids!(ops_ndr_address))
            .set_text(cx, &s("ndr_address_wrong"));
        self.ui
            .button(cx, ids!(ops_ndr_unavailable))
            .set_text(cx, &s("ndr_customer_unavailable"));
        self.ui
            .button(cx, ids!(ops_ndr_other))
            .set_text(cx, &s("ndr_other"));
        self.ui
            .text_input(cx, ids!(ops_carrier_input))
            .set_empty_text(cx, s("carrier_party_ph"));
        self.ui
            .button(cx, ids!(ops_cod_collect))
            .set_text(cx, &s("collect_cod"));
        self.ui
            .button(cx, ids!(ops_cod_remit))
            .set_text(cx, &s("remit_cod"));
        self.ui
            .button(cx, ids!(ops_dispatch_fleet))
            .set_text(cx, &s("dispatch_to_rider"));
        self.ui
            .button(cx, ids!(ops_find_awb))
            .set_text(cx, &s("find_scan_awb"));
        self.ui
            .button(cx, ids!(ops_handover_carrier))
            .set_text(cx, &s("handover_carrier"));
        self.ui
            .button(cx, ids!(ops_label))
            .set_text(cx, &s("print_awb_label"));
        self.ui
            .button(cx, ids!(ops_print_run_sheet))
            .set_text(cx, &s("print_run_sheet"));
        self.ui
            .button(cx, ids!(ops_print_drs))
            .set_text(cx, &s("print_drs"));
        self.ui
            .text_input(cx, ids!(ops_note_input))
            .set_empty_text(cx, s("note_pod_ph"));
        self.ui
            .text_input(cx, ids!(ops_party_input))
            .set_empty_text(cx, s("consignor_party_ph"));
        self.ui
            .text_input(cx, ids!(ops_rider_cod_input))
            .set_empty_text(cx, s("rider_cod_handed_ph"));
        self.ui
            .button(cx, ids!(ops_settle_rider_cod))
            .set_text(cx, &s("settle_rider_cod"));
        self.ui
            .button(cx, ids!(ops_period_bill))
            .set_text(cx, &s("period_bill_party"));
        self.ui
            .button(cx, ids!(ops_pod_pick))
            .set_text(cx, &s("pick_pod_photo"));
        self.ui
            .text_input(cx, ids!(ops_rider_input))
            .set_empty_text(cx, s("rider_name_ph"));
        self.ui
            .text_input(cx, ids!(ops_rider_phone))
            .set_empty_text(cx, s("rider_phone_ph"));
        self.ui
            .text_input(cx, ids!(ops_rider_vehicle))
            .set_empty_text(cx, s("vehicle_ph"));
        self.ui
            .button(cx, ids!(org_save))
            .set_text(cx, &s("save_business_name"));
        self.ui
            .button(cx, ids!(party_attach_add_note))
            .set_text(cx, &s("add_note"));
        self.ui
            .text_input(cx, ids!(party_attach_note))
            .set_empty_text(cx, s("note_gstin_ph"));
        self.ui
            .button(cx, ids!(party_attach_pdf))
            .set_text(cx, &s("attach_pdf"));
        self.ui
            .button(cx, ids!(party_attach_photo))
            .set_text(cx, &s("attach_photo"));
        self.ui
            .button(cx, ids!(party_delete))
            .set_text(cx, &s("delete_party"));
        self.ui
            .button(cx, ids!(party_form_clear))
            .set_text(cx, &s("new_party_clear"));
        self.ui
            .button(cx, ids!(payable_card))
            .set_text(cx, &s("pay_ap_card"));
        self.ui
            .button(cx, ids!(payable_cash))
            .set_text(cx, &s("pay_ap_cash"));
        self.ui
            .button(cx, ids!(payable_upi))
            .set_text(cx, &s("pay_ap_upi"));
        self.ui
            .button(cx, ids!(payable_writeoff))
            .set_text(cx, &s("write_off_ap"));
        self.ui
            .text_input(cx, ids!(reason_input))
            .set_empty_text(cx, s("reason_amend_ph"));
        self.ui
            .button(cx, ids!(receipt_print_btn))
            .set_text(cx, &s("print_open"));
        self.ui
            .button(cx, ids!(receipt_share_btn))
            .set_text(cx, &s("copy_receipt"));
        self.ui
            .button(cx, ids!(refund_btn))
            .set_text(cx, &s("refund"));
        self.ui
            .button(cx, ids!(reports_books_btns))
            .set_text(cx, &s("today_books"));
        self.ui
            .button(cx, ids!(reports_copy))
            .set_text(cx, &s("copy_report"));
        self.ui
            .button(cx, ids!(restore_btn))
            .set_text(cx, &s("restore_backup"));
        // Diagnostics block (Settings) — privacy-first local logs.
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
        self.ui
            .button(cx, ids!(diag_export_local))
            .set_text(cx, &s("diag_export_local"));
        self.ui
            .button(cx, ids!(diag_approve_send))
            .set_text(cx, &s("diag_approve_send"));
        self.ui
            .button(cx, ids!(diag_export_send))
            .set_text(cx, &s("diag_export_send"));
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
        self.ui
            .button(cx, ids!(sale_attach_add_note))
            .set_text(cx, &s("add_note"));
        self.ui
            .text_input(cx, ids!(sale_attach_note))
            .set_empty_text(cx, s("extra_note_awb_ph"));
        self.ui
            .button(cx, ids!(sale_attach_pdf))
            .set_text(cx, &s("attach_pdf"));
        self.ui
            .button(cx, ids!(sale_attach_photo))
            .set_text(cx, &s("attach_photo"));
        self.ui
            .button(cx, ids!(settings_open_branches))
            .set_text(cx, &s("branches_org"));
        self.ui
            .button(cx, ids!(staff_add_cashier))
            .set_text(cx, &s("add_staff"));
        self.ui
            .button(cx, ids!(staff_deactivate))
            .set_text(cx, &s("remove_selected"));
        self.ui
            .button(cx, ids!(staff_role_cashier))
            .set_text(cx, &s("role_cashier"));
        self.ui
            .button(cx, ids!(staff_role_cashier_on))
            .set_text(cx, &s("role_cashier"));
        self.ui
            .button(cx, ids!(staff_role_manager))
            .set_text(cx, &s("role_manager"));
        self.ui
            .button(cx, ids!(staff_role_manager_on))
            .set_text(cx, &s("role_manager"));
        self.ui
            .button(cx, ids!(staff_set_login))
            .set_text(cx, &s("set_login_selected"));
        self.ui
            .button(cx, ids!(stop_extra_tills))
            .set_text(cx, &s("stop_extra_tills"));
        self.ui
            .button(cx, ids!(allow_firewall))
            .set_text(cx, &s("allow_firewall"));
        self.ui
            .button(cx, ids!(new_pair_code))
            .set_text(cx, &s("new_pair_code"));
        self.ui
            .button(cx, ids!(backup_copy_btn))
            // Android has no folder to copy into each day: the owner saves a
            // copy where they choose (USB stick, SD card, Downloads).
            .set_text(cx, &s(if cfg!(target_os = "android") { "save_backup_copy" } else { "backup_copy_btn" }));
        self.ui.button(cx, ids!(autostart_btn)).set_text(
            cx,
            &s(if startup::is_on() {
                "autostart_on"
            } else {
                "autostart_off"
            }),
        );
        for path in [ids!(tax_exclusive_btn), ids!(tax_exclusive_btn_on)] {
            self.ui.button(cx, path).set_text(cx, &s("tax_added_till"));
        }
        for path in [ids!(tax_inclusive_btn), ids!(tax_inclusive_btn_on)] {
            self.ui.button(cx, path).set_text(cx, &s("tax_inside_price"));
        }
        self.ui
            .button(cx, ids!(unit_add))
            .set_text(cx, &s("add_unit"));
        self.ui
            .button(cx, ids!(venue_brand_save))
            .set_text(cx, &s("save_brand"));
        self.ui
            .widget(cx, ids!(h_print_faces))
            .set_text(cx, &plain_display_text(&s("print_faces")));
        self.ui
            .widget(cx, ids!(print_faces_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("print_faces_help")));
        self.ui
            .button(cx, ids!(print_stamp_pick))
            .set_text(cx, &s("choose_receipt_stamp"));
        self.ui
            .button(cx, ids!(print_stamp_clear))
            .set_text(cx, &s("clear_stamp"));
        self.ui
            .button(cx, ids!(print_qr_pick))
            .set_text(cx, &s("choose_payment_qr"));
        self.ui
            .button(cx, ids!(print_qr_clear))
            .set_text(cx, &s("clear_payment_qr"));
        self.ui
            .widget(cx, ids!(h_awb_pools))
            .set_text(cx, &plain_display_text(&s("awb_pools_heading")));
        self.ui
            .button(cx, ids!(awb_pool_add))
            .set_text(cx, &s("add_awb_pool"));
        self.ui
            .text_input(cx, ids!(awb_pool_carrier_input))
            .set_empty_text(cx, s("awb_pool_carrier_ph"));
        self.ui
            .text_input(cx, ids!(awb_pool_start_input))
            .set_empty_text(cx, s("awb_pool_start_ph"));
        self.ui
            .text_input(cx, ids!(awb_pool_end_input))
            .set_empty_text(cx, s("awb_pool_end_ph"));
        self.ui
            .text_input(cx, ids!(awb_pool_prefix_input))
            .set_empty_text(cx, s("awb_pool_prefix_ph"));
        self.ui
            .text_input(cx, ids!(awb_pool_label_input))
            .set_empty_text(cx, s("awb_pool_label_ph"));
        self.ui
            .text_input(cx, ids!(booking_pieces_input))
            .set_empty_text(cx, s("pieces_ph"));
        self.ui
            .text_input(cx, ids!(ops_bulk_awb_input))
            .set_empty_text(cx, s("bulk_awbs_ph"));
        self.ui
            .text_input(cx, ids!(settle_tds_input))
            .set_empty_text(cx, s("tds_cut_ph"));
        self.ui
            .text_input(cx, ids!(settle_discount_input))
            .set_empty_text(cx, s("discount_ph"));
        self.ui
            .text_input(cx, ids!(settle_cheque_input))
            .set_empty_text(cx, s("cheque_no_ph"));
        self.ui
            .button(cx, ids!(reports_bill_chase))
            .set_text(cx, &s("bill_reminder_chase"));
        self.ui
            .button(cx, ids!(credit_bill_chase))
            .set_text(cx, &s("bill_reminder_chase"));
        self.ui
            .button(cx, ids!(reports_month_mix))
            .set_text(cx, &s("month_mix_report"));
        self.ui
            .button(cx, ids!(csv_copy_gst_month))
            .set_text(cx, &s("copy_month_gst_csv"));
        self.ui
            .button(cx, ids!(venue_icon_clear))
            .set_text(cx, &s("clear_icon"));
        self.ui
            .button(cx, ids!(venue_icon_pick))
            .set_text(cx, &s("choose_icon"));
        self.ui.button(cx, ids!(void_btn)).set_text(cx, &s("void"));
        self.ui
            .text_input(cx, ids!(onboard_item_name))
            .set_empty_text(cx, s("item_name"));
        self.ui
            .text_input(cx, ids!(onboard_item_price))
            .set_empty_text(cx, s("price_ph"));
        self.ui
            .text_input(cx, ids!(onboard_cust_bal))
            .set_empty_text(cx, s("amount_owed_ph"));
        self.ui
            .widget(cx, ids!(h_booking_consignor))
            .set_text(cx, &s("booking_consignor_heading"));
        self.ui
            .widget(cx, ids!(booking_consignor_hint))
            .set_text(cx, &s("booking_consignor_hint"));
        self.ui
            .widget(cx, ids!(booking_consignor_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("booking_consignor_hint")));
        self.ui
            .widget(cx, ids!(h_booking_consignee))
            .set_text(cx, &s("booking_consignee_heading"));
        self.ui
            .widget(cx, ids!(booking_consignee_help))
            .as_info_help()
            .set_tip(&plain_display_text(&s("booking_consignee_hint")));
        self.ui
            .text_input(cx, ids!(booking_origin_name_input))
            .set_empty_text(cx, s("origin_name_ph"));
        self.ui
            .text_input(cx, ids!(booking_origin_phone_input))
            .set_empty_text(cx, phone_placeholder(&s("origin_phone_ph"), &country_code));
        self.ui
            .text_input(cx, ids!(booking_origin_address_input))
            .set_empty_text(cx, s("origin_address_ph"));
        self.ui
            .text_input(cx, ids!(booking_destination_input))
            .set_empty_text(cx, s("destination_ph"));
        self.ui
            .text_input(cx, ids!(booking_actual_kg_input))
            .set_empty_text(cx, s("weight_kg_ph"));
        self.ui
            .text_input(cx, ids!(booking_consignee_input))
            .set_empty_text(cx, s("consignee_ph"));
        self.ui
            .text_input(cx, ids!(booking_consignee_phone_input))
            .set_empty_text(cx, phone_placeholder(&s("consignee_phone_ph"), &country_code));
        self.ui
            .text_input(cx, ids!(booking_consignee_address_input))
            .set_empty_text(cx, s("consignee_address_ph"));
        self.ui
            .text_input(cx, ids!(booking_cod_input))
            .set_empty_text(cx, s("cod_amount_ph"));
        self.ui
            .text_input(cx, ids!(booking_declared_input))
            .set_empty_text(cx, s("declared_value_ph"));
        self.ui
            .text_input(cx, ids!(booking_contents_input))
            .set_empty_text(cx, s("contents_ph"));
        self.ui
            .text_input(cx, ids!(booking_eway_input))
            .set_empty_text(cx, s("eway_ph"));
        self.ui
            .text_input(cx, ids!(booking_notify_input))
            .set_empty_text(cx, phone_placeholder(&s("notify_phone_ph"), &country_code));
        self.ui
            .text_input(cx, ids!(booking_bulk_input))
            .set_empty_text(cx, s("bulk_csv_ph"));
        self.ui
            .text_input(cx, ids!(mix_credit_input))
            .set_empty_text(cx, s("credit_khata_amount_ph"));
        self.ui
            .text_input(cx, ids!(menu_name_input))
            .set_empty_text(cx, s("item_name_lang_ph"));
        self.ui
            .text_input(cx, ids!(menu_aliases_input))
            .set_empty_text(cx, s("aliases_ph"));
        self.ui
            .text_input(cx, ids!(menu_unit_input))
            .set_empty_text(cx, s("unit_kg_blank_ph"));
        self.ui
            .text_input(cx, ids!(menu_first_kg_input))
            .set_empty_text(cx, s("first_kg_ph"));
        self.ui
            .text_input(cx, ids!(menu_first_amt_input))
            .set_empty_text(cx, s("first_amount_ph"));
        self.ui
            .text_input(cx, ids!(menu_extra_kg_input))
            .set_empty_text(cx, s("extra_per_kg_ph"));
        self.ui
            .text_input(cx, ids!(menu_zone_input))
            .set_empty_text(cx, s("zone_name_ph"));
        self.ui
            .text_input(cx, ids!(menu_carrier_input))
            .set_empty_text(cx, s("rate_carrier_ph"));
        self.ui
            .text_input(cx, ids!(menu_party_input))
            .set_empty_text(cx, s("rate_party_ph"));
        self.ui
            .text_input(cx, ids!(booking_carrier_pick_input))
            .set_empty_text(cx, s("carrier_pick_ph"));
        self.ui
            .text_input(cx, ids!(menu_tax_input))
            .set_empty_text(cx, s("tax_rate_ph"));
        self.ui
            .text_input(cx, ids!(menu_category_input))
            .set_empty_text(cx, s("category_new_ph"));
        self.ui
            .text_input(cx, ids!(menu_modifiers_input))
            .set_empty_text(cx, s("options_moklanjo_ph"));
        self.ui
            .text_input(cx, ids!(buy_qty_input))
            .set_empty_text(cx, s("qty_example_ph"));
        self.ui
            .text_input(cx, ids!(buy_cost_input))
            .set_empty_text(cx, s("unit_cost_ph"));
        self.ui
            .text_input(cx, ids!(buy_expense_amount))
            .set_empty_text(cx, s("amount_ph"));
        self.ui
            .text_input(cx, ids!(buy_expense_note))
            .set_empty_text(cx, s("note_buy_expense_ph"));
        self.ui
            .text_input(cx, ids!(buy_exp_part2))
            .set_empty_text(cx, s("part_second_way_ph"));
        self.ui
            .text_input(cx, ids!(approval_pin_input))
            .set_empty_text(cx, s("manager_pin_ph"));
        self.ui
            .text_input(cx, ids!(credit_add_phone))
            .set_empty_text(cx, phone_placeholder(&s("phone_ph"), &dial_country));
        self.ui
            .text_input(cx, ids!(credit_add_address))
            .set_empty_text(cx, s("address_ph"));
        self.ui
            .text_input(cx, ids!(party_country_input))
            .set_empty_text(cx, "Country".to_string());
        self.ui
            .text_input(cx, ids!(party_state_input))
            .set_empty_text(cx, "State / region".to_string());
        self.ui
            .text_input(cx, ids!(party_pincode_input))
            .set_empty_text(cx, "PIN / ZIP / postcode".to_string());
        self.ui
            .text_input(cx, ids!(party_custom_key))
            .set_empty_text(cx, s("party_custom_key_ph"));
        self.ui
            .text_input(cx, ids!(party_custom_value))
            .set_empty_text(cx, s("party_custom_value_ph"));
        self.ui
            .text_input(cx, ids!(credit_limit_input))
            .set_empty_text(cx, s("credit_limit_ph"));
        self.ui
            .text_input(cx, ids!(settle_amount_input))
            .set_empty_text(cx, s("amount_received_ph"));
        self.ui
            .text_input(cx, ids!(settle_bill_input))
            .set_empty_text(cx, s("against_bill_ph"));
        self.ui
            .text_input(cx, ids!(writeoff_amount_input))
            .set_empty_text(cx, s("amount_blank_full_ph"));
        self.ui
            .text_input(cx, ids!(writeoff_reason_input))
            .set_empty_text(cx, s("reason_required_ph"));
        self.ui
            .text_input(cx, ids!(payable_amount_input))
            .set_empty_text(cx, s("amount_we_pay_ph"));
        self.ui
            .text_input(cx, ids!(payable_bill_input))
            .set_empty_text(cx, s("against_bill_ph"));
        self.ui
            .text_input(cx, ids!(staff_name_input))
            .set_empty_text(cx, s("staff_name_ph"));
        self.ui
            .text_input(cx, ids!(staff_title_input))
            .set_empty_text(cx, s("role_title_ph"));
        self.ui
            .text_input(cx, ids!(staff_pin_input))
            .set_empty_text(cx, s("starting_pin_ph"));
        self.ui
            .text_input(cx, ids!(staff_login_input))
            .set_empty_text(cx, s("login_id_opt_ph"));
        self.ui
            .text_input(cx, ids!(team_pick_input))
            .set_empty_text(cx, s("team_pick_ph"));
        self.ui
            .button(cx, ids!(team_pick_btn))
            .set_text(cx, &s("select_staff"));
        self.ui
            .text_input(cx, ids!(staff_password_input))
            .set_empty_text(cx, s("starting_password_ph"));
        self.ui
            .text_input(cx, ids!(my_pin_input))
            .set_empty_text(cx, s("new_pin_ph"));
        self.ui
            .text_input(cx, ids!(my_password_input))
            .set_empty_text(cx, s("new_password_ph"));
        self.ui
            .text_input(cx, ids!(duress_pin_input))
            .set_empty_text(cx, s("duress_pin_ph"));
        self.ui
            .text_input(cx, ids!(duress_pin_confirm))
            .set_empty_text(cx, s("duress_confirm_ph"));
        self.ui
            .text_input(cx, ids!(country_code_input))
            .set_empty_text(cx, s("country_code_ph"));
        self.ui
            .text_input(cx, ids!(service_bps_input))
            .set_empty_text(cx, s("service_pct_ph"));
        self.ui
            .text_input(cx, ids!(venue_title_input))
            .set_empty_text(cx, s("place_name_ph"));
        self.ui
            .text_input(cx, ids!(unit_code_input))
            .set_empty_text(cx, s("code_bag_ph"));
        self.ui
            .text_input(cx, ids!(unit_name_input))
            .set_empty_text(cx, s("unit_name_bag_ph"));
        self.ui
            .text_input(cx, ids!(conv_from_input))
            .set_empty_text(cx, s("from_code_ph"));
        self.ui
            .text_input(cx, ids!(conv_to_input))
            .set_empty_text(cx, s("to_code_ph"));
        self.ui
            .text_input(cx, ids!(conv_factor_input))
            .set_empty_text(cx, s("conv_factor_ph"));
        self.ui
            .text_input(cx, ids!(measure_dim_input))
            .set_empty_text(cx, s("dim_unit_code_ph"));
        self.ui
            .text_input(cx, ids!(measure_weight_input))
            .set_empty_text(cx, s("weight_unit_code_ph"));
        self.ui
            .text_input(cx, ids!(measure_divisor_input))
            .set_empty_text(cx, s("divisor_ph"));
        self.ui
            .text_input(cx, ids!(measure_slab_input))
            .set_empty_text(cx, s("slab_kg_ph"));
        self.ui
            .text_input(cx, ids!(ops_awb_input))
            .set_empty_text(cx, s("awb_scan_ph"));
        self.ui
            .text_input(cx, ids!(expense_amount_input))
            .set_empty_text(cx, s("amount_ph"));
        self.ui
            .text_input(cx, ids!(expense_note_input))
            .set_empty_text(cx, s("note_expense_ph"));
        self.ui
            .text_input(cx, ids!(exp_part2))
            .set_empty_text(cx, s("part_second_way_ph"));
        self.ui
            .text_input(cx, ids!(bank_amount_input))
            .set_empty_text(cx, s("amount_ph"));
        self.ui
            .text_input(cx, ids!(bank_note_input))
            .set_empty_text(cx, s("note_bank_ph"));
        self.ui
            .text_input(cx, ids!(bank_clear_input))
            .set_empty_text(cx, s("uncleared_no_ph"));
        self.ui
            .text_input(cx, ids!(inv_qty_input))
            .set_empty_text(cx, s("quantity_ph"));
        self.ui
            .text_input(cx, ids!(inv_cost_input))
            .set_empty_text(cx, s("unit_cost_ph"));
        self.ui
            .text_input(cx, ids!(org_name_input))
            .set_empty_text(cx, s("business_name_ph"));
        self.ui
            .text_input(cx, ids!(branch_name_input))
            .set_empty_text(cx, s("new_branch_name_ph"));
        self.ui
            .text_input(cx, ids!(branch_code_input))
            .set_empty_text(cx, s("short_code_ph"));
        {
            let help = self.ui.widget(cx, ids!(booking_help)).as_info_help();
            help.set_title(cx, &s("sell"));
            help.set_nav_icon(cx, NavTab::Sell);
            help.set_tip(&plain_display_text(&s("booking_help")));
        }
        {
            let help = self.ui.widget(cx, ids!(ops_desk_help)).as_info_help();
            help.set_title(cx, &s("facet_ops_title"));
            help.set_nav_icon(cx, NavTab::Ops);
            help.set_tip(&plain_display_text(&s("ops_desk_help")));
        }
    }
}

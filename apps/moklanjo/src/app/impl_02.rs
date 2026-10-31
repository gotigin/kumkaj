impl App {
fn sync_chrome(&mut self, cx: &mut Cx) {
        // While other devices depend on this one, its screen stays on:
        // Android dozes a dark device and drops them (desktops hold the
        // system awake in `net::KeepAwake`).
        let serving = net::server_is_running();
        if SCREEN_KEPT_ON.swap(serving, std::sync::atomic::Ordering::SeqCst) != serving {
            cx.keep_screen_on(serving);
        }
        // After DSL hot-reload, visibility resets to defaults. If a session is
        // still active, never leave the UI stuck on the lock screen.
        {
            let mut st = app_state().lock().unwrap();
            if st.must_confirm_device_role {
                st.screen = Screen::DeviceSetup;
            } else if st.session.is_some() && st.screen == Screen::Lock {
                st.screen = unlocked_home();
            }
            // Follow-OS mode: keep UI language in sync with the device.
            if st.language_follow_platform {
                st.locale.language = locale::detect_platform_language();
            }
        }
        // Keep the showing tab's figures fresh — data is always live, so
        // these screens need no Refresh button.
        {
            let mut st = app_state().lock().unwrap();
            if st.session.is_some() {
                match st.screen {
                    Screen::Books => {
                        let _ = st.refresh_books_overview();
                    }
                    Screen::Inventory => {
                        let _ = st.refresh_inventory_text();
                    }
                    Screen::Reports => {
                        let _ = st.refresh_reports_facet();
                    }
                    _ => {}
                }
            }
        }
        let (
            today,
            session_line,
            status,
            receipt,
            pin_field_text,
            pin_visible,
            pin_reveal_glyph,
            db,
            screen,
            table_chip,
            staff_rows,
            audit,
            locale_line,
            empty_msg,
            product_count,
            cart_count,
            total_input,
            round_hint,
            natural_label,
            applied_label,
            charge_caption,
            session_chip,
            lock_prompt,
            pin_hint,
            lock_staff_label,
            lock_as_owner,
            show_staff_role,
            undo_visible,
            undo_snack_text,
            bill_round_active,
        ) = {
            let mut st = app_state().lock().unwrap();
            let session = match &st.session {
                Some(s) => s.lock_label(),
                None => locale::t(&st.locale.language, "locked_word"),
            };
            // Header second line: the business name the owner saved (none
            // saved, none shown — never a placeholder), then who is signed
            // in, name and role — "Org · Owner (Owner)".
            let org_line = st
                .store
                .as_ref()
                .and_then(|s| s.saved_org_name().ok())
                .unwrap_or_default();
            let session_line = if org_line.trim().is_empty() {
                session.clone()
            } else {
                format!("{} · {}", org_line.trim(), session)
            };
            // A device says how its link to the shop is, and what waits.
            let session_line = match st.device_role {
                DeviceRole::Client if st.client_paired => {
                    let link = phone::link_state();
                    let n = link.waiting.to_string();
                    let mut note = if link.needs_pairing {
                        st.t("lan_pair_again")
                    } else if link.needs_sign_in {
                        st.t("lan_sign_in_to_send").replace("{n}", &n)
                    } else if link.needs_update {
                        st.t("lan_update_app")
                    } else if link.online == Some(false) && link.waiting == 0 {
                        st.t("lan_offline")
                    } else if link.online == Some(false) {
                        st.t("lan_offline_waiting").replace("{n}", &n)
                    } else if link.online.is_none() {
                        st.t("lan_connecting")
                    } else if link.waiting > 0 {
                        st.t("lan_sending").replace("{n}", &n)
                    } else {
                        String::new()
                    };
                    // Offline, the screen shows the office as it last was: say when.
                    if link.online == Some(false) && !link.updated_at.is_empty() {
                        note.push_str(&format!(" · {}", st.t("lan_last_update").replace("{time}", &link.updated_at)));
                    }
                    if link.refused > 0 {
                        let refused = st.t("lan_not_accepted").replace("{n}", &link.refused.to_string());
                        note = if note.is_empty() { refused } else { format!("{note} · {refused}") };
                    }
                    // Times this device writes (and shows) follow its own clock.
                    if let Some(skew) = net::clock_skew_secs().filter(|s| s.abs() >= net::CLOCK_WARN_SECS) {
                        let off = st.t("clock_off").replace("{min}", &(skew.abs() / 60).to_string());
                        note = if note.is_empty() { off } else { format!("{note} · {off}") };
                    }
                    if note.is_empty() {
                        session_line
                    } else {
                        format!("{session_line} · {note}")
                    }
                }
                _ => session_line,
            };

            let table_chip = st.t("ready_to_sell");
            let staff_rows = st
                .staff
                .iter()
                .map(|s| ReportRow::Line(s.lock_label()))
                .collect::<Vec<_>>();
            // Latest audit entries, one row each: who did what, and when.
            let audit = {
                let mut rows = vec![ReportRow::Head("Audit".into())];
                match st.store.as_ref().and_then(|s| s.list_audit(5).ok()) {
                    Some(events) if !events.is_empty() => rows.extend(events.iter().map(|e| {
                        ReportRow::Kv(
                            format!("{} · {} {}", e.actor_name, e.action, e.detail.trim()),
                            short_when(&e.created_at),
                        )
                    })),
                    Some(_) => rows.push(ReportRow::Line("(empty)".into())),
                    None => rows.push(ReportRow::Line("—".into())),
                }
                rows
            };
            let lang = locale::language_by_code(&st.locale.language)
                .map(|l| format!("{} ({})", l.native_name, l.name_en))
                .unwrap_or_else(|| st.locale.language.clone());
            let cur = locale::currency_by_code(&st.locale.currency)
                .map(|c| format!("{} {} — {}", c.symbol, c.code, c.name_en))
                .unwrap_or_else(|| st.locale.currency.clone());
            let locale_line = format!(
                "{}: {}\n{}: {}\n{} languages · {} currencies",
                st.t("language"),
                lang,
                st.t("currency"),
                cur,
                LANGUAGES.len(),
                CURRENCIES.len()
            );
            let empty_msg = st.t("empty_catalog");
            let natural_label = st.money(st.cart.actual_total());
            let applied_label = st.money(st.cart.applied_total());
            let total_input = locale::format_major_input_ex(
                st.cart.rounded_total().paise(),
                &st.locale.currency,
                st.decimal_sep(),
            );
            let round_hint = if st.cart.rounding().paise() != 0 {
                format!(
                    "Bill round {} · Rounded {}",
                    st.money(st.cart.rounding()),
                    st.money(st.cart.rounded_total())
                )
            } else if st.cart.applied_total().paise() != st.cart.actual_total().paise() {
                format!(
                    "Items @×qty {} · Applied {}. Rounded starts equal to Applied.",
                    st.money(st.cart.actual_total()),
                    st.money(st.cart.applied_total())
                )
            } else {
                "Rounded starts equal to Applied. Edit only to round the bill.".into()
            };
            let charge_caption = if st.cart.rounding().paise() != 0 {
                format!("{} ({})", st.t("rounded"), st.money(st.cart.rounding()))
            } else {
                st.t("rounded")
            };
            let pin_field_text = st.pin_buffer.chars().take(12).collect::<String>();
            let pin_visible = st.pin_visible;
            let pin_reveal_glyph = eye_glyph(pin_visible);
            let session_chip = match &st.session {
                Some(s) => s.lock_label(),
                None => locale::t(&st.locale.language, "locked_word"),
            };
            let has_staff = st.staff.iter().any(|s| !s.is_owner());
            let owner_setup_complete = st.owner_setup_complete;
            let pin_reset_required = st.pin_reset_required;
            let lock_prompt = if !owner_setup_complete || pin_reset_required {
                locale::t(&st.locale.language, "set_owner_pin")
            } else if st.lock_as_owner {
                locale::t(&st.locale.language, "enter_owner_passcode")
            } else {
                locale::t(&st.locale.language, "enter_staff_passcode")
            };
            let pin_hint = if !owner_setup_complete {
                locale::t(&st.locale.language, "pin_hint_choose")
            } else if pin_reset_required {
                locale::t(&st.locale.language, "pin_hint_new")
            } else if st.device_role == DeviceRole::Server {
                if st.extra_tills_allowed {
                    let code = st
                        .store
                        .as_ref()
                        .and_then(|s| s.pairing_code().ok())
                        .unwrap_or_default();
                    locale::t(&st.locale.language, "pin_hint_shop_code").replacen("{}", &code, 1)
                } else {
                    locale::t(&st.locale.language, "pin_hint_books")
                }
            } else if st.device_role == DeviceRole::Client {
                if st.client_paired {
                    locale::t(&st.locale.language, "pin_hint_paired")
                } else {
                    locale::t(&st.locale.language, "pin_hint_pair_first")
                }
            } else {
                locale::t(&st.locale.language, "unlock_to_continue")
            };
            let lock_staff_label = if st.lock_as_owner {
                String::new()
            } else {
                st.staff
                    .iter()
                    .filter(|s| !s.is_owner())
                    .find(|s| Some(&s.id) == st.lock_staff_id.as_ref())
                    .map(|s| s.lock_label())
                    .or_else(|| {
                        st.staff
                            .iter()
                            .find(|s| !s.is_owner())
                            .map(|s| s.lock_label())
                    })
                    .unwrap_or_else(|| locale::t(&st.locale.language, "no_staff_yet"))
            };
            // Until Owner finishes setup, and until staff exist, only Owner
            // unlock. A pending PIN reset is the Owner's job too.
            let lock_as_owner = if !owner_setup_complete || !has_staff || pin_reset_required {
                st.lock_as_owner = true;
                true
            } else {
                st.lock_as_owner
            };
            let show_staff_role = owner_setup_complete && has_staff && !pin_reset_required;
            let undo_visible = st.undo_sale_id.is_some()
                && matches!(
                    st.screen,
                    Screen::Books
                        | Screen::Pos
                        | Screen::Buy
                        | Screen::Menu
                        | Screen::More
                        | Screen::Sales
                        | Screen::Credit
                        | Screen::Team
                        | Screen::Settings
                        | Screen::Inventory
                        | Screen::Branches
                );
            let undo_snack_text = st.undo_snack_text.clone();
            let bill_round_active = st.cart.charge_total.is_some();
            let today_visible = amount_key_visible(
                st.amounts_screen_hidden,
                &st.amount_keys_hidden,
                &st.amount_keys_peek,
                "today",
            );
            (
                st.today_label(today_visible),
                session_line,
                st.status.clone(),
                if st.last_receipt == "No sale yet" {
                    st.t("no_sale_yet")
                } else {
                    st.last_receipt.clone()
                },
                pin_field_text,
                pin_visible,
                pin_reveal_glyph,
                st.store
                    .as_ref()
                    .map(|s| s.db_path().display().to_string())
                    .unwrap_or_else(|| "no db".into()),
                st.screen,
                table_chip,
                staff_rows,
                audit,
                locale_line,
                empty_msg,
                st.products.len(),
                st.cart.lines.len(),
                total_input,
                round_hint,
                natural_label,
                applied_label,
                charge_caption,
                session_chip,
                lock_prompt,
                pin_hint,
                lock_staff_label,
                lock_as_owner,
                show_staff_role,
                undo_visible,
                undo_snack_text,
                bill_round_active,
            )
        };

        {
            let st = app_state().lock().unwrap();
            let screen_eye = eye_glyph(!st.amounts_screen_hidden);
            drop(st);
            // Money slots already masked in `today_label`; keep words visible.
            self.ui
                .widget(cx, ids!(today_label))
                .set_text(cx, &plain_display_text(&today));
            // Header eye: screen-level only (icon, no label).
            self.ui.button(cx, ids!(amounts_eye)).set_text(cx, screen_eye);
        }
        self.ui
            .widget(cx, ids!(session_label))
            .set_text(cx, &plain_display_text(&session_line));
        // Client: session_line already carries Offline / Connecting; toast once
        // when the shop server drops so the header eye is not overloaded.
        {
            let is_client_paired = {
                let st = app_state().lock().unwrap();
                st.device_role == DeviceRole::Client && st.client_paired
            };
            if is_client_paired {
                let link = phone::link_state();
                static LAST_ONLINE: std::sync::Mutex<Option<bool>> =
                    std::sync::Mutex::new(None);
                let mut prev = LAST_ONLINE.lock().unwrap();
                let went_unreachable = *prev == Some(true) && link.online == Some(false);
                *prev = link.online;
                if went_unreachable {
                    let msg = app_state().lock().unwrap().t("lan_offline");
                    self.set_status(cx, msg);
                }
            }
        }
        self.ui
            .widget(cx, ids!(receipt_label))
            .set_text(cx, &plain_display_text(&receipt));
        {
            let help = self.ui.widget(cx, ids!(pin_help)).as_info_help();
            help.set_title(cx, &plain_display_text(&lock_prompt));
            help.set_tip(&plain_display_text(&pin_hint));
        }
        {
            let pin_field = self.ui.text_input(cx, ids!(lock_pin_input));
            if !cx.has_key_focus(pin_field.area()) && pin_field.text() != pin_field_text {
                pin_field.set_text(cx, &pin_field_text);
            }
            pin_field.set_is_password(cx, !pin_visible);
        }
        self.ui
            .button(cx, ids!(pin_reveal))
            .set_text(cx, pin_reveal_glyph);
        self.ui.button(cx, ids!(pin_ok)).set_text(
            cx,
            &{
                let st = app_state().lock().unwrap();
                locale::t(&st.locale.language, "unlock")
            },
        );
        // Owner/Staff: Overlay idle+active faces — Owner must not stay lit after Staff.
        self.paint_choice_pair(cx, ids!(role_owner), ids!(role_owner_on), lock_as_owner);
        if show_staff_role {
            self.paint_choice_pair(cx, ids!(role_staff), ids!(role_staff_on), !lock_as_owner);
        } else {
            self.ui.button(cx, ids!(role_staff)).set_visible(cx, false);
            self.ui.button(cx, ids!(role_staff_on)).set_visible(cx, false);
        }
        // On the lock screen, surface status (wrong PIN / no store) under the dots.
        let pin_status_text = if screen == Screen::Lock {
            if status.is_empty() || status.starts_with("Local DB:") {
                " ".to_string()
            } else {
                status.clone()
            }
        } else {
            " ".to_string()
        };
        let setup_status_text = if screen == Screen::DeviceSetup {
            if status.is_empty() || status.starts_with("Local DB:") {
                " ".to_string()
            } else {
                status.clone()
            }
        } else {
            " ".to_string()
        };
        self.ui
            .widget(cx, ids!(pin_status))
            .set_text(cx, &plain_display_text(&pin_status_text));
        self.ui
            .widget(cx, ids!(device_setup_status))
            .set_text(cx, &plain_display_text(&setup_status_text));
        self.ui
            .widget(cx, ids!(lock_staff_label))
            .set_text(cx, &plain_display_text(&lock_staff_label));
        self.ui
            .view(cx, ids!(lock_staff_row))
            .set_visible(cx, !lock_as_owner);
        let _ = db;
        self.ui.widget(cx, ids!(db_path_label)).set_visible(cx, false);
        {
            // Automatic backups: when the last one ran, where they are, and
            // whether the second copy works.
            let st = app_state().lock().unwrap();
            let text = st.store.as_ref().map(|store| {
                let dir = store.auto_backup_dir().display().to_string();
                let mut text = match store.last_backup_at().ok().flatten() {
                    Some(at) => {
                        let when = chrono::DateTime::parse_from_rfc3339(&at)
                            .map(|t| t.format("%d %b %H:%M").to_string())
                            .unwrap_or(at);
                        st.t("backup_auto")
                            .replace("{when}", &when)
                            .replace("{dir}", &dir)
                    }
                    None => st.t("backup_auto_none"),
                };
                if let Ok(Some(extra)) = store.backup_copy_dir() {
                    text.push_str(&format!(
                        " {}",
                        st.t("backup_copy_to")
                            .replace("{dir}", &extra.display().to_string())
                    ));
                }
                if let Ok(e) = store.backup_copy_error()
                    && !e.is_empty()
                {
                    text.push_str(&format!(" {e}"));
                }
                text
            });
            let warning = backup_warning(&st);
            let host = st.device_role.is_host();
            drop(st);
            let text = match text {
                Some(text) if !warning.is_empty() => format!("{warning}\n\n{text}"),
                text => text.unwrap_or_default(),
            };
            self.ui
                .widget(cx, ids!(backup_auto_label))
                .set_text(cx, &plain_display_text(&text));
            self.ui
                .widget(cx, ids!(books_backup_warning))
                .set_text(cx, &plain_display_text(&warning));
            self.ui
                .widget(cx, ids!(books_backup_warning))
                .set_visible(cx, !warning.is_empty());
            self.ui
                .button(cx, ids!(backup_copy_btn))
                .set_visible(cx, host);
        }

        self.set_report_rows(cx, ids!(audit_label), audit);
        self.ui
            .widget(cx, ids!(table_chip))
            .set_text(cx, &plain_display_text(&table_chip));
        self.ui
            .widget(cx, ids!(session_chip))
            .set_text(cx, &plain_display_text(&session_chip));
        // Booking desk: chargeable kg + last AWB / next-booking CTA.
        {
            // Pull typed address/dest/zone/weight so pincode + compare stay current.
            {
                let mut st = app_state().lock().unwrap();
                st.booking_origin_name = self
                    .ui
                    .text_input(cx, ids!(booking_origin_name_input))
                    .text();
                st.booking_origin_phone = self
                    .ui
                    .text_input(cx, ids!(booking_origin_phone_input))
                    .text();
                st.booking_origin_address = self
                    .ui
                    .text_input(cx, ids!(booking_origin_address_input))
                    .text();
                st.booking_destination = self
                    .ui
                    .text_input(cx, ids!(booking_destination_input))
                    .text();
                st.booking_consignee = self
                    .ui
                    .text_input(cx, ids!(booking_consignee_input))
                    .text();
                st.booking_consignee_phone = self
                    .ui
                    .text_input(cx, ids!(booking_consignee_phone_input))
                    .text();
                st.booking_consignee_address = self
                    .ui
                    .text_input(cx, ids!(booking_consignee_address_input))
                    .text();
                st.booking_zone = self.ui.text_input(cx, ids!(booking_zone_input)).text();
                st.booking_actual_kg =
                    self.ui.text_input(cx, ids!(booking_actual_kg_input)).text();
                st.booking_length_cm =
                    self.ui.text_input(cx, ids!(booking_length_cm_input)).text();
                st.booking_breadth_cm = self
                    .ui
                    .text_input(cx, ids!(booking_breadth_cm_input))
                    .text();
                st.booking_height_cm =
                    self.ui.text_input(cx, ids!(booking_height_cm_input)).text();
            }
            let mut st = app_state().lock().unwrap();
            let (actual, vol, chargeable) = st.booking_weights();
            let weight_unit = st.measure_weight_unit.clone();
            let dim_unit = st.measure_dim_unit.clone();
            let weight_factor = st.measure_weight_factor;
            let charge_line = if chargeable > precise::ZERO {
                if weight_factor == rust_decimal::Decimal::ONE {
                    format!(
                        "Chargeable {} kg (actual {} · vol {})",
                        precise::format_kg(chargeable),
                        precise::format_kg(actual),
                        precise::format_kg(vol)
                    )
                } else {
                    // Typed unit first (what the user thinks in), kg after
                    // (what rates and books use).
                    format!(
                        "Chargeable {} {} ({} kg) · actual {} · vol {}",
                        precise::format_kg(booking::from_base(chargeable, weight_factor)),
                        weight_unit,
                        precise::format_kg(chargeable),
                        precise::format_kg(booking::from_base(actual, weight_factor)),
                        precise::format_kg(booking::from_base(vol, weight_factor))
                    )
                }
            } else {
                "Chargeable: set weight or L×B×H".into()
            };
            // Pincode → zone / serves; carrier compare when zone + weight known.
            let dest = st.booking_destination.trim().to_string();
            let mut zone_for_compare = st.booking_zone.trim().to_string();
            let mut pin_line = String::new();
            let mut autofill_zone: Option<String> = None;
            if let Some(store) = st.store.as_ref()
                && rates::is_india_pincode(&dest)
            {
                match store.lookup_pincode(&dest) {
                    Ok(hits) if !hits.is_empty() => {
                        let first = &hits[0];
                        pin_line = first.serves_label();
                        if zone_for_compare.is_empty() && !first.zone_name.is_empty() {
                            zone_for_compare = first.zone_name.clone();
                            autofill_zone = Some(first.zone_name.clone());
                        }
                    }
                    Ok(_) => pin_line = "Pincode not in service table".into(),
                    Err(_) => {}
                }
            }
            let currency = st.locale.currency.clone();
            let consignor = st.selected_customer_id.clone().unwrap_or_default();
            let mut compare_line = String::new();
            let mut compare_rows = Vec::new();
            if chargeable > precise::ZERO
                && !zone_for_compare.is_empty()
                && let Some(store) = st.store.as_ref()
            {
                let zone_id = store.list_zones().ok().and_then(|zones| {
                    zones.into_iter().find(|z| {
                        z.id == zone_for_compare || z.name.eq_ignore_ascii_case(&zone_for_compare)
                    })
                });
                if let Some(z) = zone_id
                    && let Ok((sell, rows)) =
                        store.compare_carriers_for_zone(&z.id, chargeable, &consignor)
                {
                    compare_line = rates::format_carrier_compare(sell, &rows, &currency);
                    compare_rows = rows;
                }
            }
            st.last_carrier_compare = compare_rows;
            let zone_autofill = if let Some(zname) = autofill_zone
                && st.booking_zone.trim().is_empty()
            {
                st.booking_zone = zname.clone();
                Some(zname)
            } else {
                None
            };
            // Booking-desk field hints wear the shop's chosen units.
            self.ui
                .text_input(cx, ids!(booking_actual_kg_input))
                .set_empty_text(cx, format!("Actual {weight_unit}"));
            self.ui
                .text_input(cx, ids!(booking_length_cm_input))
                .set_empty_text(cx, format!("L {dim_unit}"));
            self.ui
                .text_input(cx, ids!(booking_breadth_cm_input))
                .set_empty_text(cx, format!("B {dim_unit}"));
            self.ui
                .text_input(cx, ids!(booking_height_cm_input))
                .set_empty_text(cx, format!("H {dim_unit}"));
            // Desk unit chips: the shop's current units wear the active
            // face (a custom unit selects no chip — it shows in the hints).
            for (idle, on, code) in [
                (ids!(wunit_kg), ids!(wunit_kg_on), "kg"),
                (ids!(wunit_lb), ids!(wunit_lb_on), "lb"),
                (ids!(wunit_g), ids!(wunit_g_on), "g"),
                (ids!(wunit_oz), ids!(wunit_oz_on), "oz"),
            ] {
                self.paint_choice_pair(cx, idle, on, weight_unit == code);
            }
            for (idle, on, code) in [
                (ids!(dunit_cm), ids!(dunit_cm_on), "cm"),
                (ids!(dunit_in), ids!(dunit_in_on), "in"),
                (ids!(dunit_ft), ids!(dunit_ft_on), "ft"),
                (ids!(dunit_m), ids!(dunit_m_on), "m"),
            ] {
                self.paint_choice_pair(cx, idle, on, dim_unit == code);
            }
            // Expense tender chips (both forms share the choice).
            let (exp_t1, exp_t2) = (st.expense_tender, st.expense_tender2);
            for (idle, on, m) in [
                (ids!(exp_t1_cash), ids!(exp_t1_cash_on), PayMethod::Cash),
                (ids!(exp_t1_bank), ids!(exp_t1_bank_on), PayMethod::Bank),
                (ids!(exp_t1_upi), ids!(exp_t1_upi_on), PayMethod::Upi),
                (ids!(exp_t1_card), ids!(exp_t1_card_on), PayMethod::Card),
                (
                    ids!(buy_exp_t1_cash),
                    ids!(buy_exp_t1_cash_on),
                    PayMethod::Cash,
                ),
                (
                    ids!(buy_exp_t1_bank),
                    ids!(buy_exp_t1_bank_on),
                    PayMethod::Bank,
                ),
                (
                    ids!(buy_exp_t1_upi),
                    ids!(buy_exp_t1_upi_on),
                    PayMethod::Upi,
                ),
                (
                    ids!(buy_exp_t1_card),
                    ids!(buy_exp_t1_card_on),
                    PayMethod::Card,
                ),
            ] {
                self.paint_choice_pair(cx, idle, on, exp_t1 == m);
            }
            for (idle, on, m) in [
                (ids!(exp_t2_cash), ids!(exp_t2_cash_on), PayMethod::Cash),
                (ids!(exp_t2_bank), ids!(exp_t2_bank_on), PayMethod::Bank),
                (ids!(exp_t2_upi), ids!(exp_t2_upi_on), PayMethod::Upi),
                (ids!(exp_t2_card), ids!(exp_t2_card_on), PayMethod::Card),
                (
                    ids!(buy_exp_t2_cash),
                    ids!(buy_exp_t2_cash_on),
                    PayMethod::Cash,
                ),
                (
                    ids!(buy_exp_t2_bank),
                    ids!(buy_exp_t2_bank_on),
                    PayMethod::Bank,
                ),
                (
                    ids!(buy_exp_t2_upi),
                    ids!(buy_exp_t2_upi_on),
                    PayMethod::Upi,
                ),
                (
                    ids!(buy_exp_t2_card),
                    ids!(buy_exp_t2_card_on),
                    PayMethod::Card,
                ),
            ] {
                self.paint_choice_pair(cx, idle, on, exp_t2 == m);
            }
            let awb_line = if st.last_awb.is_empty() {
                String::new()
            } else {
                format!("AWB {} — stick label, then Next booking", st.last_awb)
            };
            let show_next = !st.last_awb.is_empty();
            drop(st);
            self.ui
                .widget(cx, ids!(chargeable_kg_label))
                .set_text(cx, &plain_display_text(&charge_line));
            self.ui
                .widget(cx, ids!(booking_pincode_label))
                .set_text(cx, &plain_display_text(&pin_line));
            self.ui
                .widget(cx, ids!(booking_compare_label))
                .set_text(cx, &plain_display_text(&compare_line));
            if let Some(zname) = zone_autofill {
                let field = self.ui.text_input(cx, ids!(booking_zone_input));
                if !cx.has_key_focus(field.area()) {
                    field.set_text(cx, &zname);
                }
            }
            self.ui
                .widget(cx, ids!(last_awb_label))
                .set_text(cx, &plain_display_text(&awb_line));
            self.ui
                .button(cx, ids!(next_booking_btn))
                .set_visible(cx, show_next);
        }
        self.set_report_rows(cx, ids!(staff_label), staff_rows);
        self.ui
            .widget(cx, ids!(locale_label))
            .set_text(cx, &plain_display_text(&locale_line));

        // Country pack: the summary, the pay-rail wording on the Sell buttons,
        // and the exclusive-tax estimate under the totals strip.
        {
            let (fiscal_line, (country_hint, country_pack), tax_line, digital, business_id, service_pct) = {
                let st = app_state().lock().unwrap();
                (
                    st.fiscal.summary(),
                    // The pack being browsed: what applying it would set,
                    // one row each under the line that says how to apply it.
                    match fiscal::country_by_code(&st.country_browse) {
                        Some(p) => (
                            format!("Browsing {} — tap Apply country pack to switch", p.code),
                            p.apply_facts()
                                .into_iter()
                                .map(|(what, value)| ReportRow::Kv(what.to_string(), value))
                                .collect::<Vec<_>>(),
                        ),
                        None => (
                            format!(
                                "{} country packs · type a code or tap ‹ Browse ›",
                                fiscal::countries().len()
                            ),
                            Vec::new(),
                        ),
                    },
                    st.cart_tax_estimate_line(),
                    st.fiscal.digital_pay_label().to_string(),
                    st.fiscal.business_tax_id.clone(),
                    locale::format_percent_bps(
                        st.fiscal.service_charge_bps,
                        st.fiscal.decimal_sep(),
                    ),
                )
            };
            self.ui
                .widget(cx, ids!(fiscal_label))
                .set_text(cx, &plain_display_text(&fiscal_line));
            self.ui
                .widget(cx, ids!(country_hint))
                .set_text(cx, &plain_display_text(&country_hint));
            self.set_report_rows(cx, ids!(country_pack_list), country_pack);
            self.ui
                .widget(cx, ids!(tax_estimate_label))
                .set_text(cx, &plain_display_text(&tax_line));
            self.ui
                .widget(cx, ids!(tax_estimate_label))
                .set_visible(cx, tax_line.trim() != "");
            self.ui
                .button(cx, ids!(amend_pay_upi))
                .set_text(cx, &digital);
            self.ui
                .button(cx, ids!(settle_upi))
                .set_text(cx, &format!("Pay {digital}"));
            for (path, value) in [
                (ids!(business_tax_id_input), business_id),
                (ids!(service_bps_input), service_pct),
            ] {
                let field = self.ui.text_input(cx, path);
                if !cx.has_key_focus(field.area()) && field.text() != value {
                    field.set_text(cx, &value);
                }
            }
        }
        self.ui
            .widget(cx, ids!(empty_catalog))
            .set_text(cx, &plain_display_text(&empty_msg));
        self.ui
            .widget(cx, ids!(menu_empty))
            .set_text(cx, &plain_display_text(&empty_msg));
        self.ui
            .widget(cx, ids!(empty_catalog))
            .set_visible(cx, product_count == 0);
        self.ui
            .widget(cx, ids!(menu_empty))
            .set_visible(cx, product_count == 0);
        self.ui
            .widget(cx, ids!(product_list))
            .set_visible(cx, product_count > 0);
        self.ui
            .widget(cx, ids!(product_list_menu))
            .set_visible(cx, product_count > 0);
        self.ui
            .widget(cx, ids!(cart_empty_hint))
            .set_visible(cx, cart_count == 0);
        self.ui
            .widget(cx, ids!(cart_list))
            .set_visible(cx, cart_count > 0);
        self.ui
            .widget(cx, ids!(cart_natural_label))
            .set_text(cx, &plain_display_text(&natural_label));
        self.ui
            .widget(cx, ids!(cart_applied_label))
            .set_text(cx, &plain_display_text(&applied_label));
        self.ui
            .widget(cx, ids!(cart_charge_caption))
            .set_text(cx, &plain_display_text(&charge_caption));
        self.ui
            .widget(cx, ids!(cart_round_hint))
            .set_text(cx, &plain_display_text(&round_hint));
        {
            let total_field = self.ui.text_input(cx, ids!(cart_total_input));
            let shown = total_field.text();
            if shown != total_input {
                if bill_round_active {
                    // Seller is editing an explicit bill round — don't clobber mid-type.
                    if !cx.has_key_focus(total_field.area()) {
                        total_field.set_text(cx, &total_input);
                    }
                } else {
                    // Default Rounded = Applied. Always mirror the model so a
                    // stale field from the previous line count cannot stick
                    // (especially when the input still holds key focus).
                    if cx.has_key_focus(total_field.area()) {
                        total_field.take_key_focus(cx);
                    }
                    total_field.set_text(cx, &total_input);
                }
            }
        }
        self.ui
            .view(cx, ids!(undo_snackbar))
            .set_visible(cx, undo_visible);
        self.ui
            .widget(cx, ids!(undo_snack_label))
            .set_text(cx, &plain_display_text(&undo_snack_text));
        self.paint_app_snackbar(cx);

        let (credit_chip, empty_hint, parties_heading, party_picked) = {
            let st = app_state().lock().unwrap();
            let credit_chip = match &st.selected_customer_id {
                Some(id) => st
                    .customers
                    .iter()
                    .find(|c| &c.id == id)
                    .map(|c| {
                        if c.balance_paise > 0 {
                            format!(
                                "{} · {} {}",
                                c.name,
                                st.t("owes_word"),
                                st.money(c.balance())
                            )
                        } else if c.balance_paise < 0 {
                            format!(
                                "{} · {} {}",
                                c.name,
                                st.t("advance"),
                                st.money(Money(-c.balance_paise))
                            )
                        } else {
                            format!("{} · {}", c.name, st.t("chip_clear_credit"))
                        }
                    })
                    .unwrap_or_else(|| st.t("chip_missing_customer")),
                None => {
                    // A Credit tap with no name used to land only on the lock
                    // screen's status line, so the button looked dead.
                    if st.status.starts_with("Type a customer")
                        || st.status.starts_with("Pick a customer")
                        || st.status.starts_with("Customer name")
                    {
                        st.status.clone()
                    } else {
                        st.t("chip_no_customer")
                    }
                }
            };
            let empty_hint = if st.customers.is_empty() {
                "No saved customers yet — type a name above."
            } else {
                "Tap Pick to use a saved customer."
            };
            let parties_heading = if st.customers.is_empty() {
                "Saved parties".to_string()
            } else {
                format!("Saved parties ({})", st.customers.len())
            };
            let party_picked = st.selected_customer_id.is_some();
            (credit_chip, empty_hint, parties_heading, party_picked)
        };
        self.ui
            .widget(cx, ids!(parties_list_heading))
            .set_text(cx, &plain_display_text(&parties_heading));
        self.ui
            .view(cx, ids!(party_detail))
            .set_visible(cx, party_picked);
        self.ui
            .button(cx, ids!(party_delete))
            .set_visible(cx, party_picked);
        // Always available — after Add the form clears itself; Clear still
        // abandons an in-progress edit without saving.
        self.ui
            .button(cx, ids!(party_form_clear))
            .set_visible(cx, true);
        {
            let form_btn_label = if party_picked {
                "Save changes"
            } else {
                "Add party"
            };
            self.ui
                .button(cx, ids!(credit_add_btn))
                .set_text(cx, form_btn_label);
        }
        self.ui
            .widget(cx, ids!(credit_chip))
            .set_text(cx, &plain_display_text(&credit_chip));
        self.ui
            .widget(cx, ids!(customers_empty_hint))
            .set_text(cx, &plain_display_text(empty_hint));

        let (
            sale_selected,
            sale_history,
            amend_total_text,
            amend_natural,
            amend_applied,
            amend_round,
        ) = {
            let st = app_state().lock().unwrap();
            let sale_selected = if let Some(id) = &st.selected_sale_id {
                // The lines and totals are right below; the header only says
                // which booking this is.
                let which = st
                    .recent_sales
                    .iter()
                    .find(|s| &s.id == id)
                    .map(|s| s.awb_no.trim().to_string())
                    .filter(|awb| !awb.is_empty())
                    .unwrap_or_else(|| id[..8.min(id.len())].to_string());
                format!(
                    "Editing {which} · {} · {}",
                    st.money(st.amend_cart.rounded_total()),
                    st.amend_pay.label_for(&st.fiscal)
                )
            } else {
                "No sale selected".into()
            };
            // Each change to the booking on its own row: who, what it was,
            // why — and when.
            let mut sale_history = Vec::new();
            if st.selected_sale_id.is_some() {
                sale_history.push(ReportRow::Head("History".into()));
                if st.sale_revisions.is_empty() {
                    sale_history.push(ReportRow::Line("(none yet)".into()));
                }
            }
            sale_history.extend(st.sale_revisions.iter().take(5).map(|r| {
                let reason = if r.reason.trim().is_empty() {
                    String::new()
                } else {
                    format!(" · {}", r.reason.trim())
                };
                ReportRow::Kv(
                    format!("{} · was {}{reason}", r.actor_name, st.money(Money(r.previous.total_paise))),
                    short_when(&r.revised_at),
                )
            }));
            let amend_total_text = locale::format_major_input_ex(
                st.amend_cart.rounded_total().paise(),
                &st.locale.currency,
                st.decimal_sep(),
            );
            let amend_natural = format!("Actual Total {}", st.money(st.amend_cart.actual_total()));
            let amend_applied =
                format!("Applied Total {}", st.money(st.amend_cart.applied_total()));
            let amend_round = if st.amend_cart.is_empty() {
                "Rounded Total starts equal to Applied Total.".into()
            } else if st.amend_cart.rounding().paise() != 0 {
                format!(
                    "Rounding {} · Rounded Total {}",
                    st.money(st.amend_cart.rounding()),
                    st.money(st.amend_cart.rounded_total())
                )
            } else {
                "Rounded Total equals Applied Total.".into()
            };
            (
                sale_selected,
                sale_history,
                amend_total_text,
                amend_natural,
                amend_applied,
                amend_round,
            )
        };
        {
            let may_void = {
                let st = app_state().lock().unwrap();
                // The PIN field shows while any of this person's sale rights
                // needs a manager (Team → Rights).
                st.session
                    .as_ref()
                    .map(|s| {
                        let rights = s.effective_rights();
                        [domain::Right::VoidRefund, domain::Right::AmendSale]
                            .iter()
                            .all(|r| rights.level(*r) == domain::RightLevel::Allowed)
                    })
                    .unwrap_or(false)
            };
            self.ui
                .text_input(cx, ids!(approval_pin_input))
                .set_visible(cx, !may_void);
            self.ui.widget(cx, ids!(approval_hint)).set_text(
                cx,
                &plain_display_text(
                    "Void and refund need an Owner or Manager. Type their PIN above and the approval is recorded with your name.",
                ),
            );
            // Tip lives on sales_help ℹ (amend_help); keep essay off the work surface.
            self.ui
                .widget(cx, ids!(approval_hint))
                .set_visible(cx, false);
        }
        self.ui
            .widget(cx, ids!(sale_selected_label))
            .set_text(cx, &plain_display_text(&sale_selected));
        {
            let attach_txt = {
                let st = app_state().lock().unwrap();
                match (&st.selected_sale_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => store
                        .attachments_summary_text(AttachmentEntity::Sale, id)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    _ => String::new(),
                }
            };
            self.show_attachments(
                cx,
                ids!(sale_attach_hint),
                ids!(sale_attach_list),
                &attach_txt,
                "(no attachments yet — select a sale)",
            );
        }
        {
            let attach_txt = {
                let st = app_state().lock().unwrap();
                match (&st.selected_customer_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => store
                        .attachments_summary_text(AttachmentEntity::Party, id)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    _ => String::new(),
                }
            };
            self.show_attachments(
                cx,
                ids!(party_attach_hint),
                ids!(party_attach_list),
                &attach_txt,
                "(no attachments yet — select a party)",
            );
        }
        {
            let (custom_rows, lang) = {
                let st = app_state().lock().unwrap();
                let lang = st.locale.language.clone();
                let rows = match st
                    .selected_customer_id
                    .as_ref()
                    .and_then(|id| st.customers.iter().find(|c| &c.id == id))
                {
                    Some(c) if !c.custom_fields.is_empty() => c
                        .custom_fields
                        .iter()
                        .map(|f| ReportRow::Kv(f.key.clone(), f.value.clone()))
                        .collect::<Vec<_>>(),
                    Some(_) => vec![ReportRow::Line(locale::t(&lang, "party_custom_empty"))],
                    None => vec![ReportRow::Line(locale::t(&lang, "party_custom_pick_first"))],
                };
                (rows, lang)
            };
            let _ = lang;
            self.set_report_rows(cx, ids!(party_custom_list), custom_rows);
        }
        {
            // Bill-wise lists for the selected party (numbers match the
            // "Against bill #" inputs).
            let (ar_rows, ap_rows) = {
                let st = app_state().lock().unwrap();
                let rows = |bills: Vec<domain::OpenBill>, st: &AppState| -> Vec<ReportRow> {
                    if bills.is_empty() {
                        return Vec::new();
                    }
                    let mut rows = vec![ReportRow::Head("Open bills".into())];
                    rows.extend(bills.iter().enumerate().map(|(i, b)| {
                        ReportRow::Kv(
                            format!("{}) {}", i + 1, &b.created_at[..b.created_at.len().min(10)]),
                            format!(
                                "{} open (of {})",
                                st.money(Money(b.open_paise)),
                                st.money(Money(b.billed_paise))
                            ),
                        )
                    }));
                    rows
                };
                match (&st.selected_customer_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => (
                        rows(store.open_credit_bills(id).unwrap_or_default(), &st),
                        rows(store.open_payable_bills(id).unwrap_or_default(), &st),
                    ),
                    _ => (Vec::new(), Vec::new()),
                }
            };
            self.set_report_rows(cx, ids!(settle_open_bills), ar_rows);
            self.set_report_rows(cx, ids!(payable_open_bills), ap_rows);
        }
        {
            // Bank reconciliation: book vs statement-cleared, plus the
            // uncleared movements numbered for the tick-off input.
            let recon_rows = {
                let st = app_state().lock().unwrap();
                match st.store.as_ref() {
                    Some(store) => {
                        let mut rows = match store.bank_reconciliation() {
                            Ok((book, cleared, n, _net)) => vec![
                                ReportRow::Kv("Book balance".into(), st.money(Money(book))),
                                ReportRow::Kv("Cleared (bank)".into(), st.money(Money(cleared))),
                                ReportRow::Kv("Uncleared".into(), n.to_string()),
                            ],
                            Err(_) => Vec::new(),
                        };
                        let uncleared = store
                            .list_money_movements(50)
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|m| !m.cleared)
                            .take(8)
                            .enumerate()
                            .map(|(i, m)| {
                                let kind = MoneyMoveKind::parse(&m.kind)
                                    .map(|k| k.label())
                                    .unwrap_or("Movement");
                                ReportRow::Kv(
                                    format!(
                                        "{}) {} {}",
                                        i + 1,
                                        &m.created_at[..m.created_at.len().min(10)],
                                        kind
                                    ),
                                    st.money(Money(m.amount_paise)),
                                )
                            })
                            .collect::<Vec<_>>();
                        if !uncleared.is_empty() {
                            rows.push(ReportRow::Head("Uncleared".into()));
                            rows.extend(uncleared);
                        }
                        rows
                    }
                    None => Vec::new(),
                }
            };
            self.set_report_rows(cx, ids!(bank_recon_label), recon_rows);
        }
        {
            let attach_txt = {
                let st = app_state().lock().unwrap();
                match (&st.selected_expense_id, st.store.as_ref()) {
                    (Some(id), Some(store)) => store
                        .attachments_summary_text(AttachmentEntity::Expense, id)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    _ => String::new(),
                }
            };
            self.show_attachments(
                cx,
                ids!(expense_attach_hint),
                ids!(expense_attach_list),
                &attach_txt,
                "(no attachments yet — log an expense)",
            );
        }
        {
            let pod = app_state()
                .lock()
                .unwrap()
                .pending_ops_pod
                .as_ref()
                .map(|p| format!("POD ready: {}", p.display()))
                .unwrap_or_else(|| "No POD photo selected".into());
            self.ui
                .widget(cx, ids!(ops_pod_status))
                .set_text(cx, &plain_display_text(&pod));
        }
        self.set_report_rows(cx, ids!(sale_history_label), sale_history);
        self.ui
            .widget(cx, ids!(amend_natural_label))
            .set_text(cx, &plain_display_text(&amend_natural));
        self.ui
            .widget(cx, ids!(amend_applied_label))
            .set_text(cx, &plain_display_text(&amend_applied));
        self.ui
            .widget(cx, ids!(amend_round_hint))
            .set_text(cx, &plain_display_text(&amend_round));
        {
            let field = self.ui.text_input(cx, ids!(amend_total_input));
            if !cx.has_key_focus(field.area()) {
                let shown = field.text();
                if shown != amend_total_text {
                    field.set_text(cx, &amend_total_text);
                }
            }
        }

        let image_hint = {
            let st = app_state().lock().unwrap();
            let name = self.ui.text_input(cx, ids!(menu_name_input)).text();
            menu_images::describe_image(st.draft_image_key.as_deref(), &name)
        };
        self.ui
            .widget(cx, ids!(image_status))
            .set_text(cx, &plain_display_text(&image_hint));
        {
            let (draft_key, name) = {
                let st = app_state().lock().unwrap();
                (
                    st.draft_image_key.clone(),
                    self.ui.text_input(cx, ids!(menu_name_input)).text(),
                )
            };
            let dir = item_images_dir();
            let path = image_key_path(draft_key.as_deref(), &dir);
            let preview = self.ui.widget(cx, ids!(draft_preview));
            apply_thumb_avatar(cx, &preview, &name, &path);
        }
        let party_image_hint = {
            let st = app_state().lock().unwrap();
            let name = self.ui.text_input(cx, ids!(credit_add_name)).text();
            match st.draft_party_image_key.as_deref() {
                Some(k) if k.starts_with("pending_custom:") => {
                    "Logo: new file selected — tap Add / Save".to_string()
                }
                Some(k) if k.starts_with("custom:") => {
                    format!("Logo: {}", menu_images::describe_image(Some(k), &name))
                }
                Some(_) => menu_images::describe_image(st.draft_party_image_key.as_deref(), &name),
                None => "Logo: none (initials)".to_string(),
            }
        };
        self.ui
            .widget(cx, ids!(party_image_hint))
            .set_text(cx, &plain_display_text(&party_image_hint));
        {
            let (draft_key, name) = {
                let st = app_state().lock().unwrap();
                (
                    st.draft_party_image_key.clone(),
                    self.ui.text_input(cx, ids!(credit_add_name)).text(),
                )
            };
            let dir = party_images_dir();
            let path = image_key_path(draft_key.as_deref(), &dir);
            let preview = self.ui.widget(cx, ids!(party_draft_preview));
            apply_thumb_avatar(cx, &preview, &name, &path);
        }

        // Team: who is being edited, and their rights (draft until Save).
        {
            let (selected, rights, lang) = {
                let st = app_state().lock().unwrap();
                let selected = st
                    .team_staff_id
                    .as_ref()
                    .and_then(|id| st.staff.iter().find(|s| &s.id == id))
                    .map(|s| format!("{}: {} ({})", st.t("editing_staff"), s.name, s.display_role()))
                    .unwrap_or_else(|| " ".to_string());
                (selected, st.team_draft_rights(), st.locale.language.clone())
            };
            let on = app_state().lock().unwrap().draft_features.clone();
            for (idle, active, set) in [
                (ids!(feat_sell), ids!(feat_sell_on), on.sell),
                (ids!(feat_items), ids!(feat_items_on), on.items),
                (ids!(feat_sales), ids!(feat_sales_on), on.sales),
                (ids!(feat_credit), ids!(feat_credit_on), on.credit),
                (ids!(feat_team), ids!(feat_team_on), on.team),
                (ids!(feat_backup), ids!(feat_backup_on), on.backup),
                (ids!(feat_inventory), ids!(feat_inventory_on), on.inventory),
                (ids!(feat_branches), ids!(feat_branches_on), on.branches),
            ] {
                self.paint_choice_pair(cx, idle, active, set);
            }
            self.ui
                .widget(cx, ids!(team_selected_label))
                .set_text(cx, &plain_display_text(&selected));
            for (button, right) in [
                (ids!(right_void_btn), domain::Right::VoidRefund),
                (ids!(right_amend_btn), domain::Right::AmendSale),
            ] {
                let level = match rights.level(right) {
                    domain::RightLevel::Allowed => "right_allowed",
                    domain::RightLevel::WithApproval => "right_with_approval",
                    domain::RightLevel::Denied => "right_denied",
                };
                self.ui.button(cx, button).set_text(
                    cx,
                    &format!("{}: {}", locale::t(&lang, right.label_key()), locale::t(&lang, level)),
                );
            }
        }

        self.sync_venue_brand(cx);
        self.sync_awb_pools_label(cx);

        self.show_screen(cx, screen);
        self.hide_docs_and_debug_chrome(cx);
        self.sync_device_setup_sections(cx);
        {
            let st = app_state().lock().unwrap();
            // Pairing digits live in `pairing_code_value`; this label is status only
            // (Wi‑Fi / firewall / tills off) so listen/firewall notes never hide the code.
            let (pair_label, pair_code, show_pair_code) = if st.device_role == DeviceRole::Server {
                if st.extra_tills_allowed {
                    let code = st
                        .store
                        .as_ref()
                        .and_then(|s| s.pairing_code().ok())
                        .unwrap_or_else(|| "------".into());
                    let mut text = match net::local_ipv4() {
                        Some(ip) => st
                            .t("lan_pair_status")
                            .replace("{code}", &code)
                            .replace("{addr}", &ip.to_string()),
                        None => st.t("lan_join_help_no_addr").replace("{code}", &code),
                    };
                    if !net::server_is_running() {
                        text.push_str(" · ");
                        text.push_str(
                            &st.t("lan_not_listening")
                                .replace("{port}", &net::DEFAULT_PORT.to_string()),
                        );
                    } else {
                        let fw = firewall_state().lock().unwrap();
                        if fw.busy && fw.open == Some(false) {
                            text.push_str(" · ");
                            text.push_str(&st.t("firewall_waiting"));
                        } else if fw.open == Some(false) {
                            text.push_str(" · ");
                            text.push_str(&st.t("firewall_blocked"));
                        }
                    }
                    (text, code, true)
                } else {
                    (st.t("lan_tills_off"), String::new(), false)
                }
            } else if st.device_role == DeviceRole::Client {
                let text = if st.client_paired {
                    let url = st
                        .store
                        .as_ref()
                        .and_then(|s| s.server_url().ok())
                        .flatten()
                        .unwrap_or_default();
                    st.t("paired_with").replace("{url}", &url)
                } else {
                    st.t("not_paired")
                };
                (text, String::new(), false)
            } else {
                (st.t("choose_device_use"), String::new(), false)
            };
            let day_close = st.day_close_text.clone();
            let reports_nav = st.reports_nav;
            let inventory_text = st.ops_panel_text.clone();
            let branches_body = st.branches_text.clone();
            let org_name = st
                .store
                .as_ref()
                .and_then(|s| s.saved_org_name().ok())
                .unwrap_or_default();
            let tmpl = entity_template::resolve(&st.entity_template_id);
            let tmpl_line = tmpl.title.to_string();
            let onboard_status = if st.onboarding_complete {
                st.t("setup_complete")
            } else {
                match st.onboard_step {
                    1 => st.t("onboard_fresh_tmpl").replacen("{}", tmpl.title, 1),
                    2 => st.t("onboard_book_tmpl").replacen("{}", tmpl.title, 1),
                    _ => st.t("onboard_template_tmpl").replacen("{}", tmpl.title, 1),
                }
            };
            let cust_ph = st.t("customer_name_ph");
            let new_cust_ph = st.t("new_customer_ph");
            let tmpl_blurb_s = st.t("tmpl_blurb");
            drop(st);
            self.ui
                .widget(cx, ids!(more_mode_hint))
                .set_visible(cx, false);
            self.ui
                .widget(cx, ids!(server_pair_label))
                .set_text(cx, &plain_display_text(&pair_label));
            self.ui
                .widget(cx, ids!(pairing_code_value))
                .set_text(cx, &plain_display_text(&pair_code));
            self.ui
                .widget(cx, ids!(pairing_code_value))
                .set_visible(cx, show_pair_code);
            self.ui
                .widget(cx, ids!(settings_tmpl_label))
                .set_text(cx, &plain_display_text(&tmpl_line));
            let units_rows = {
                let st = app_state().lock().unwrap();
                st.store.as_ref().map(units_report_rows).unwrap_or_default()
            };
            self.set_report_rows(cx, ids!(units_label), units_rows);
            // Booking measures now in force, one per row.
            let measure_rows = {
                let st = app_state().lock().unwrap();
                vec![
                    ReportRow::Head("Now".into()),
                    ReportRow::Kv("Dimensions in".into(), st.measure_dim_unit.clone()),
                    ReportRow::Kv("Weight in".into(), st.measure_weight_unit.clone()),
                    ReportRow::Kv(
                        "Volumetric divisor".into(),
                        format!("÷ {}", precise::format_kg(st.measure_divisor)),
                    ),
                    ReportRow::Kv(
                        "Chargeable slab".into(),
                        format!("{} kg", precise::format_kg(st.measure_slab_kg)),
                    ),
                ]
            };
            self.set_report_rows(cx, ids!(measure_label), measure_rows);
            // One short ℹ tip — not the full product map.
            let tmpl_hint = if tmpl_blurb_s.is_empty() {
                tmpl.title.to_string()
            } else {
                tmpl_blurb_s.clone()
            };
            self.ui
                .widget(cx, ids!(onboard_tmpl_help))
                .as_info_help()
                .set_tip(&plain_display_text(&tmpl_hint));
            self.ui
                .text_input(cx, ids!(customer_name_input))
                .set_empty_text(cx, cust_ph.clone());
            self.ui
                .text_input(cx, ids!(onboard_cust_name))
                .set_empty_text(cx, cust_ph);
            self.ui
                .text_input(cx, ids!(credit_add_name))
                .set_empty_text(cx, new_cust_ph);
            {
                // Raw rows — ReportText paints per-Kv eyes + masks at draw time.
                self.set_report_rows(cx, ids!(day_close_label), parse_report(&day_close));
            }
            // Books home: each figure has its own eye (persisted).
            {
                let overview = {
                    let st = app_state().lock().unwrap();
                    st.store.as_ref().and_then(|s| s.books_overview().ok())
                };
                if let Some(o) = overview {
                    self.ui
                        .widget(cx, ids!(books_declared_name))
                        .set_text(cx, &plain_display_text(&o.lane_declared_label));
                    self.ui
                        .widget(cx, ids!(books_owner_name))
                        .set_text(cx, &plain_display_text(&o.lane_owner_label));
                    self.ui
                        .widget(cx, ids!(books_digital_name))
                        .set_text(cx, &plain_display_text(&o.digital_label));
                    self.paint_amount_value(
                        cx,
                        ids!(books_in_value),
                        ids!(books_in_eye),
                        "books_in",
                        &o.money_in,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_purchases_value),
                        ids!(books_purchases_eye),
                        "books_purchases",
                        &o.purchases,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_expenses_value),
                        ids!(books_expenses_eye),
                        "books_expenses",
                        &o.expenses,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_gross_value),
                        ids!(books_gross_eye),
                        "books_gross",
                        &o.gross,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_net_value),
                        ids!(books_net_eye),
                        "books_net",
                        &o.net,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_declared_value),
                        ids!(books_declared_eye),
                        "books_declared",
                        &o.lane_declared,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_owner_value),
                        ids!(books_owner_eye),
                        "books_owner",
                        &o.lane_owner,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_cash_value),
                        ids!(books_cash_eye),
                        "books_cash",
                        &o.cash,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_digital_value),
                        ids!(books_digital_eye),
                        "books_digital",
                        &o.digital,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_card_value),
                        ids!(books_card_eye),
                        "books_card",
                        &o.card,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_they_owe_value),
                        ids!(books_they_owe_eye),
                        "books_they_owe",
                        &o.they_owe,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_we_owe_value),
                        ids!(books_we_owe_eye),
                        "books_we_owe",
                        &o.we_owe,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_bank_balance_value),
                        ids!(books_bank_balance_eye),
                        "books_bank_balance",
                        &o.bank_balance,
                    );
                    self.paint_amount_value(
                        cx,
                        ids!(books_bank_net_value),
                        ids!(books_bank_net_eye),
                        "books_bank_net",
                        &o.bank_net_today,
                    );
                }
                let till = {
                    let st = app_state().lock().unwrap();
                    st.store
                        .as_ref()
                        .and_then(|s| s.till_balance_paise().ok())
                        .map(|v| st.money(Money(v)))
                };
                if let Some(till) = till {
                    self.paint_amount_value(
                        cx,
                        ids!(books_till_value),
                        ids!(books_till_eye),
                        "books_till",
                        &till,
                    );
                }
            }
            self.sync_books_sections(cx);
            {
                // Raw rows — ReportText paints per-Kv eyes + masks at draw time.
                self.set_report_rows(cx, ids!(reports_label), parse_report(&day_close));
            }
            let ops_facet = !matches!(reports_nav, ReportsNav::Bank);
            self.ui.view(cx, ids!(ops_desk)).set_visible(cx, ops_facet);
            self.ui
                .view(cx, ids!(reports_books_btns))
                .set_visible(cx, !ops_facet);
            self.ui
                .widget(cx, ids!(reports_expense_heading))
                .set_visible(cx, !ops_facet);
            self.ui
                .text_input(cx, ids!(expense_amount_input))
                .set_visible(cx, !ops_facet);
            self.ui
                .text_input(cx, ids!(expense_note_input))
                .set_visible(cx, !ops_facet);
            self.ui
                .button(cx, ids!(expense_add))
                .set_visible(cx, !ops_facet);
            self.ui
                .widget(cx, ids!(reports_bank_heading))
                .set_visible(cx, !ops_facet);
            self.ui
                .text_input(cx, ids!(bank_amount_input))
                .set_visible(cx, !ops_facet);
            self.ui
                .text_input(cx, ids!(bank_note_input))
                .set_visible(cx, !ops_facet);
            for id in [
                ids!(bank_till_to_bank),
                ids!(bank_bank_to_till),
                ids!(bank_in),
                ids!(bank_out),
            ] {
                self.ui.button(cx, id).set_visible(cx, !ops_facet);
            }
            // Reconciliation + CSV widgets live in the same facet.
            self.ui.widget(cx, ids!(bank_recon_label)).set_visible(cx, !ops_facet);
            for id in [ids!(exp_t1_row), ids!(exp_t2_row)] {
                self.ui.view(cx, id).set_visible(cx, !ops_facet);
            }
            self.ui
                .text_input(cx, ids!(exp_part2))
                .set_visible(cx, !ops_facet);
            self.ui
                .text_input(cx, ids!(bank_clear_input))
                .set_visible(cx, !ops_facet);
            for id in [
                ids!(bank_mark_cleared),
                ids!(bank_mark_uncleared),
                ids!(csv_copy_day),
                ids!(csv_copy_outstanding),
                ids!(csv_copy_parties),
                ids!(csv_copy_items),
                ids!(csv_copy_ledger),
                ids!(csv_import_parties),
                ids!(csv_import_items),
                ids!(csv_import_pincodes),
                ids!(csv_import_carrier_bill),
                ids!(csv_copy_carrier_bill),
            ] {
                self.ui.button(cx, id).set_visible(cx, !ops_facet);
            }
            self.ui
                .widget(cx, ids!(inventory_label))
                .set_text(cx, &plain_display_text(&inventory_text));
            self.ui
                .widget(cx, ids!(branches_label))
                .set_text(cx, &plain_display_text(&branches_body));
            {
                let field = self.ui.text_input(cx, ids!(org_name_input));
                if !cx.has_key_focus(field.area()) && field.text() != org_name {
                    field.set_text(cx, &org_name);
                }
            }
            self.ui
                .widget(cx, ids!(onboard_status))
                .set_text(cx, &plain_display_text(&onboard_status));
            self.ui
                .widget(cx, ids!(client_pair_status)).set_text(cx, &plain_display_text(&{
                    // The last pairing try speaks for itself (refused, waiting…).
                    let st = app_state().lock().unwrap();
                    if !st.pair_note.is_empty() {
                        st.pair_note.clone()
                    } else if st.client_paired {
                        let updated = phone::link_state().updated_at;
                        if updated.is_empty() {
                            st.t("paired_note")
                        } else {
                            format!("{} {}", st.t("paired_note"), st.t("last_update_at").replace("{time}", &updated))
                        }
                    } else {
                        st.t("pair_intro")
                    }
                }));
            // What this device still owes the office, and what the office
            // would not take — said here so nothing waits or vanishes unseen.
            let (queue, refused) = {
                let st = app_state().lock().unwrap();
                match st.store.as_ref().filter(|_| st.device_role == DeviceRole::Client && st.client_paired) {
                    Some(store) => {
                        let waiting = store.lan_outbox().unwrap_or_default();
                        let refused = store.lan_refused().unwrap_or_default();
                        let mut rows = Vec::new();
                        if !waiting.is_empty() {
                            rows.push(report_head(
                                &st.t("queue_waiting").replace("{n}", &waiting.len().to_string()),
                            ));
                            for item in &waiting {
                                // Why it is still here, in the words the last try got.
                                let line = if item.attempts > 0 {
                                    let tries = st
                                        .t("queue_tries")
                                        .replace("{n}", &item.attempts.to_string())
                                        .replace("{e}", item.last_error.trim());
                                    format!("{} — {tries}", item.label)
                                } else {
                                    item.label.clone()
                                };
                                rows.push(ReportRow::Line(line));
                            }
                        }
                        if !refused.is_empty() {
                            rows.push(report_head(
                                &st.t("queue_refused").replace("{n}", &refused.len().to_string()),
                            ));
                            for item in &refused {
                                rows.push(ReportRow::Line(format!(
                                    "{} ({}) — {}",
                                    item.label,
                                    short_when(&item.created_at),
                                    item.error
                                )));
                            }
                        }
                        (rows, !refused.is_empty())
                    }
                    None => (Vec::new(), false),
                }
            };
            self.set_report_rows(cx, ids!(client_queue_label), queue);
            self.ui.button(cx, ids!(client_clear_refused)).set_visible(cx, refused);
        }

        // Role-aware settings rows + chrome peers (rights live in paint_nav_tabs).
        {
            let (is_owner, is_client, is_shop, extra_tills) = {
                let st = app_state().lock().unwrap();
                (
                    st.session.as_ref().is_some_and(|s| s.is_owner()),
                    st.device_role == DeviceRole::Client,
                    st.device_role == DeviceRole::Server,
                    st.extra_tills_allowed,
                )
            };
            self.ui.button(cx, ids!(more_items)).set_visible(cx, false);
            // More hub removed — former rows are chrome peers (bottom + rail).
            self.ui.view(cx, ids!(more_screen)).set_visible(cx, false);
            self.ui
                .button(cx, ids!(allow_extra_tills))
                .set_visible(cx, is_shop && is_owner && !extra_tills);
            self.ui
                .button(cx, ids!(stop_extra_tills))
                .set_visible(cx, is_shop && is_owner && extra_tills);
            let blocked = {
                let fw = firewall_state().lock().unwrap();
                fw.open == Some(false) && !fw.busy
            };
            self.ui
                .button(cx, ids!(allow_firewall))
                .set_visible(cx, is_shop && is_owner && extra_tills && blocked);
            self.ui
                .button(cx, ids!(new_pair_code))
                .set_visible(cx, is_shop && is_owner && extra_tills);
            self.ui
                .widget(cx, ids!(paired_list))
                .set_visible(cx, is_shop && is_owner && extra_tills);
            {
                let paired = app_state().lock().unwrap().client_paired;
                self.ui.button(cx, ids!(client_connection_btn)).set_visible(cx, is_client && paired);
                // Only after Pair now — opens the PIN screen (not a second unlock action).
                self.ui
                    .button(cx, ids!(client_skip_lock))
                    .set_visible(cx, is_client && paired);
            }
            {
                let duress_on = is_shop && is_owner;
                let configured = duress_on
                    && app_state()
                        .lock()
                        .unwrap()
                        .store
                        .as_ref()
                        .and_then(|s| s.duress_pin_configured().ok())
                        .unwrap_or(false);
                for id in [
                    ids!(h_duress_heading),
                    ids!(h_duress_help),
                    ids!(duress_status),
                    ids!(duress_pin_input),
                    ids!(duress_pin_confirm),
                    ids!(save_duress_pin),
                    ids!(clear_duress_pin),
                ] {
                    self.ui.widget(cx, id).set_visible(cx, duress_on);
                }
                let status = if !duress_on {
                    String::new()
                } else if configured {
                    app_state().lock().unwrap().t("duress_on")
                } else {
                    app_state().lock().unwrap().t("duress_off")
                };
                self.ui
                    .widget(cx, ids!(duress_status))
                    .set_text(cx, &plain_display_text(&status));
            }
            self.ui.button(cx, ids!(autostart_btn)).set_visible(
                cx,
                is_shop && is_owner && extra_tills && startup::supported(),
            );
            self.ui.button(cx, ids!(more_sales)).set_visible(cx, false);
            self.ui
                .button(cx, ids!(more_day_close))
                .set_visible(cx, false);
            self.ui
                .button(cx, ids!(more_reports))
                .set_visible(cx, false);
            self.ui
                .button(cx, ids!(more_inventory))
                .set_visible(cx, false);
            self.ui
                .button(cx, ids!(more_branches))
                .set_visible(cx, false);
            self.ui.button(cx, ids!(more_credit)).set_visible(cx, false);
            self.ui.button(cx, ids!(more_team)).set_visible(cx, false);
            self.ui
                .button(cx, ids!(more_settings))
                .set_visible(cx, false);
            self.ui.button(cx, ids!(more_ops)).set_visible(cx, false);
            self.ui.button(cx, ids!(more_cod)).set_visible(cx, false);
            self.ui.button(cx, ids!(more_riders)).set_visible(cx, false);
            self.ui.button(cx, ids!(more_lock)).set_visible(cx, false);
            let screen = app_state().lock().unwrap().screen;
            self.paint_nav_tabs(cx, screen);
        }

        // Header back (‹ left of shop mark). Body Back buttons stay hidden.
        {
            let show = {
                let st = app_state().lock().unwrap();
                st.picking_server
                    || matches!(
                        st.screen,
                        Screen::ClientPair | Screen::SidebarSettings
                    )
                    || !st.nav_back.is_empty()
            };
            self.ui.button(cx, ids!(header_back)).set_visible(cx, show);
            for id in [
                ids!(client_pair_back),
                ids!(server_pick_cancel),
                ids!(sidebar_settings_back),
            ] {
                self.ui.button(cx, id).set_visible(cx, false);
            }
        }

        self.lift_footer_above_system_bar(cx);
        self.repair_focused_carets(cx);
        self.ui.redraw(cx);
    }
}

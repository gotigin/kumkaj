impl App {
fn sync_chrome(&mut self, cx: &mut Cx) {
        // While other devices depend on this one, its screen stays on:
        // Android dozes a dark device and drops them (desktops hold the
        // system awake in `net::KeepAwake`).
        let serving = net::server_is_running();
        if SCREEN_KEPT_ON.swap(serving, std::sync::atomic::Ordering::SeqCst) != serving {
            cx.keep_screen_on(serving);
        }
        // The menu window belongs to the order desk: leaving it any other
        // way (locking, a jump to another screen) closes the window too.
        let stray_menu = {
            let mut st = app_state().lock().unwrap();
            let stray = st.menu_pick_open && (st.screen != Screen::Pos || st.session.is_none());
            if stray {
                st.menu_pick_open = false;
                st.menu_filter.clear();
                st.menu_category = None;
            }
            stray
        };
        if stray_menu {
            self.ui.modal(cx, ids!(menu_modal)).close(cx);
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
                    } else if link.online == Some(false) && link.waiting == 0 {
                        st.t("lan_offline")
                    } else if link.online == Some(false) {
                        st.t("lan_offline_waiting").replace("{n}", &n)
                    } else if link.waiting > 0 {
                        st.t("lan_sending").replace("{n}", &n)
                    } else {
                        String::new()
                    };
                    // Offline, the screen shows the shop as it last was: say when.
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
                    if note.is_empty() { session_line } else { format!("{session_line} · {note}") }
                }
                _ => session_line,
            };
            let table_chip = match st
                .selected_table_id
                .as_ref()
                .and_then(|id| st.tables.iter().find(|t| &t.id == id))
            {
                Some(t) => format!("{} · {}", t.name, st.t("dine_in")),
                None => match st.cart.order_type {
                    OrderType::DineIn => st.t("dine_in"),
                    other => format!("{} order", other.label()),
                },
            };
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
                "{}: {}\n{}: {}",
                st.t("language"),
                lang,
                st.t("currency"),
                cur
            );
            let empty_msg = if st.products.is_empty() {
                "No items yet. Open Menu to add your dishes and items.".to_string()
            } else {
                st.t("empty_catalog")
            };
            let natural_label = st.money(st.cart.actual_total());
            let applied_label = st.money(st.cart.applied_total());
            let total_input =
                locale::format_major_input_ex(
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
                    .unwrap_or_else(|| {
                        locale::t(&st.locale.language, "no_staff_yet")
                    })
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
                // The ticket's status line: one short line (a whole invoice
                // was dumped here after every payment); nothing when idle.
                if st.last_receipt == "No sale yet" {
                    String::new()
                } else {
                    ticket_status_line(&st.last_receipt)
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
            // Top eye: screen-level only (icon, no label). Open = amounts may
            // show; closed = force every figure to dots for this session.
            self.ui.button(cx, ids!(amounts_eye)).set_text(cx, screen_eye);
        }
        self.ui
            .widget(cx, ids!(session_label)).set_text(cx, &plain_display_text(&session_line));
        self.ui
            .widget(cx, ids!(receipt_label)).set_text(cx, &plain_display_text(&receipt));
        self.ui.widget(cx, ids!(receipt_label)).set_visible(cx, !receipt.trim().is_empty());
        // The Menu screen has no ticket, so the same status lands above its form
        // — never the "Local DB: …" boot line (no books jargon on a work surface).
        let menu_status_text = {
            let st = app_state().lock().unwrap();
            // Only news about the menu — not the unlock greeting or boot line.
            if st.status.starts_with("Local DB:") || st.status.starts_with("Welcome") {
                " ".to_string()
            } else {
                st.status.clone()
            }
        };
        self.ui
            .widget(cx, ids!(menu_status))
            .set_text(cx, &plain_display_text(&menu_status_text));
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
        self.ui
            .button(cx, ids!(role_staff))
            .set_visible(cx, show_staff_role);
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
            .widget(cx, ids!(pin_status)).set_text(cx, &plain_display_text(&pin_status_text));
        self.ui
            .widget(cx, ids!(device_setup_status)).set_text(cx, &plain_display_text(&setup_status_text));
        self.ui
            .widget(cx, ids!(lock_staff_label)).set_text(cx, &plain_display_text(&lock_staff_label));
        self.ui
            .view(cx, ids!(lock_staff_row))
            .set_visible(cx, !lock_as_owner);
        // Debug path stays out of the owner-facing UI (docs / support only).
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
                        st.t("backup_auto").replace("{when}", &when).replace("{dir}", &dir)
                    }
                    None => st.t("backup_auto_none"),
                };
                if let Ok(Some(extra)) = store.backup_copy_dir() {
                    text.push_str(&format!(" {}", st.t("backup_copy_to").replace("{dir}", &extra.display().to_string())));
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
            .widget(cx, ids!(table_chip)).set_text(cx, &plain_display_text(&table_chip));
        self.ui
            .widget(cx, ids!(session_chip)).set_text(cx, &plain_display_text(&session_chip));
        // Order desk: order type chips + expense tender chips.
        {
            let st = app_state().lock().unwrap();
            let cart_active = !st.cart.is_empty();
            self.ui
                .button(cx, ids!(next_order_btn))
                .set_visible(cx, cart_active);
            self.ui
                .widget(cx, ids!(kitchen_empty))
                .set_visible(cx, st.open_kots.is_empty());
            let order_type = st.cart.order_type;
            // The ticket's table, as a button: choose one, or change it.
            let table_name = st
                .selected_table_id
                .as_ref()
                .and_then(|id| st.tables.iter().find(|t| &t.id == id))
                .map(|t| t.name.clone());
            let table_btn = match &table_name {
                Some(name) => format!("{name}  ·  {}", st.t("change_table")),
                None => st.t("choose_table"),
            };
            let pending = st.cart.lines.iter().filter(|l| !l.is_sent()).count();
            let pick_label = {
                let base = if st.picking_from_table_id.is_some() {
                    st.t("change_table")
                } else {
                    st.t("pick_table_banner")
                };
                if pending > 0 {
                    format!("{base} · {pending} {}", st.t("items_word"))
                } else {
                    base
                }
            };
            let picking = st.picking_table;
            let cats = st.menu_categories();
            let cat_sel = st.menu_category.clone();
            let searching = !st.menu_filter.trim().is_empty();
            let all_label = st.t("all_items");
            let back_label = st.t("back_to_order");
            self.ui
                .view(cx, ids!(table_pick_wrap))
                .set_visible(cx, order_type == OrderType::DineIn);
            self.ui.button(cx, ids!(table_pick_btn)).set_text(cx, &table_btn);
            self.ui.view(cx, ids!(floor_pick_bar)).set_visible(cx, picking);
            self.ui
                .widget(cx, ids!(floor_pick_label))
                .set_text(cx, &plain_display_text(&pick_label));
            self.ui.button(cx, ids!(floor_pick_back)).set_text(cx, &back_label);
            // Counter stays docked at the foot of the ticket (always shown).
            self.ui.view(cx, ids!(ticket_body)).set_visible(cx, true);
            // The customer is asked for only when it matters: credit (khata),
            // a split that may include credit, or a delivery.
            self.ui.view(cx, ids!(ticket_extras)).set_visible(
                cx,
                !st.cart.is_empty()
                    && (matches!(st.pay_choice, PayChoice::Credit | PayChoice::Split)
                        || st.cart.order_type == OrderType::Delivery),
            );
            self.ui
                .widget(cx, ids!(delivery_by_input))
                .set_visible(cx, st.cart.order_type == OrderType::Delivery);
            // The total box explains itself; the old rounding note did not.
            self.ui.widget(cx, ids!(cart_round_hint)).set_visible(cx, false);
            let resale = st.form_resale;
            self.paint_choice_pair(cx, ids!(menu_kind_dish), ids!(menu_kind_dish_on), !resale);
            self.paint_choice_pair(cx, ids!(menu_kind_resale), ids!(menu_kind_resale_on), resale);
            for (idle, on, key) in [
                (ids!(menu_kind_dish), ids!(menu_kind_dish_on), "kind_dish"),
                (ids!(menu_kind_resale), ids!(menu_kind_resale_on), "kind_resale"),
            ] {
                let text = st.t(key);
                self.ui.button(cx, idle).set_text(cx, &text);
                self.ui.button(cx, on).set_text(cx, &text);
            }
            let price_ph = st.t("price_ph");
            self.ui.text_input(cx, ids!(menu_price_input)).set_empty_text(cx, price_ph);
            self.ui.view(cx, ids!(sale_detail)).set_visible(cx, st.selected_sale_id.is_some());

            // Menu template: what it holds, what is still missing from this menu.
            {
                let shelf = menu_images::shelf();
                let have: std::collections::HashSet<String> =
                    st.products.iter().map(|p| p.name.trim().to_lowercase()).collect();
                let missing = shelf.iter().filter(|d| !have.contains(&d.name.trim().to_lowercase())).count();
                let folders = menu_images::shelf_folders(shelf).len();
                let info = if shelf.is_empty() {
                    String::new()
                } else if missing == 0 {
                    st.t("template_done")
                } else {
                    st.t("template_info")
                        .replace("{c}", &folders.to_string())
                        .replace("{d}", &missing.to_string())
                };
                let confirming = st.template_confirm && missing > 0;
                self.ui.view(cx, ids!(template_card)).set_visible(cx, !shelf.is_empty());
                self.ui.label(cx, ids!(template_heading)).set_text(cx, &st.t("template_heading"));
                self.ui.label(cx, ids!(template_info)).set_text(
                    cx,
                    &if confirming { st.t("template_confirm").replace("{d}", &missing.to_string()) } else { info },
                );
                self.ui.button(cx, ids!(template_import)).set_text(cx, &st.t("template_import"));
                self.ui.button(cx, ids!(template_import)).set_visible(cx, missing > 0 && !confirming);
                self.ui.view(cx, ids!(template_confirm)).set_visible(cx, confirming);
                self.ui.button(cx, ids!(template_yes)).set_text(cx, &st.t("template_yes"));
                self.ui.button(cx, ids!(template_no)).set_text(cx, &st.t("cancel_item_short"));
            }
            let party_picked = st.selected_customer_id.is_some();
            self.ui.view(cx, ids!(party_detail)).set_visible(cx, party_picked);
            self.ui.button(cx, ids!(party_delete)).set_visible(cx, party_picked);
            self.ui.button(cx, ids!(party_form_clear)).set_visible(cx, party_picked);
            self.ui
                .widget(cx, ids!(customer_list_credit))
                .set_visible(cx, st.customers.iter().any(|c| c.active));
            for (idle, on, m) in [
                (ids!(amend_pay_cash), ids!(amend_pay_cash_on), PayMethod::Cash),
                (ids!(amend_pay_upi), ids!(amend_pay_upi_on), PayMethod::Upi),
                (ids!(amend_pay_card), ids!(amend_pay_card_on), PayMethod::Card),
            ] {
                self.paint_choice_pair(cx, idle, on, st.amend_pay == m);
            }
            let inclusive = st.fiscal.tax_pricing == TaxPricing::Inclusive;
            self.paint_choice_pair(cx, ids!(tax_inclusive_btn), ids!(tax_inclusive_btn_on), inclusive);
            self.paint_choice_pair(cx, ids!(tax_exclusive_btn), ids!(tax_exclusive_btn_on), !inclusive);
            let allow_zero = st
                .store
                .as_ref()
                .and_then(|s| s.allow_zero_sales().ok())
                .unwrap_or(true);
            self.paint_choice_pair(cx, ids!(zero_sales_on_btn), ids!(zero_sales_on_btn_on), allow_zero);
            self.paint_choice_pair(cx, ids!(zero_sales_off_btn), ids!(zero_sales_off_btn_on), !allow_zero);
            let has_lines = !st.cart.is_empty();
            self.ui.view(cx, ids!(cart_ticket_card)).set_visible(cx, has_lines);
            self.ui.view(cx, ids!(send_stack)).set_visible(cx, has_lines);
            // Inventory: the list, or a word on where stock comes from.
            let no_stock = st.stock_items().is_empty();
            self.ui.widget(cx, ids!(stock_empty)).set_visible(cx, no_stock);
            self.ui.widget(cx, ids!(stock_list)).redraw(cx);
            // One-tap payment row and what each choice needs.
            let choice = st.pay_choice;
            for (idle, on, c) in [
                (ids!(pm_cash), ids!(pm_cash_on), PayChoice::Cash),
                (ids!(pm_upi), ids!(pm_upi_on), PayChoice::Upi),
                (ids!(pm_card), ids!(pm_card_on), PayChoice::Card),
                (ids!(pm_credit), ids!(pm_credit_on), PayChoice::Credit),
                (ids!(pm_split), ids!(pm_split_on), PayChoice::Split),
            ] {
                self.paint_choice_pair(cx, idle, on, choice == c);
            }
            let labels = [
                (ids!(pm_cash), ids!(pm_cash_on), st.t("cash")),
                (ids!(pm_upi), ids!(pm_upi_on), st.fiscal.digital_pay_label().to_string()),
                (ids!(pm_card), ids!(pm_card_on), st.t("card")),
                (ids!(pm_credit), ids!(pm_credit_on), st.t("pay_credit")),
                (ids!(pm_split), ids!(pm_split_on), st.t("split")),
            ];
            for (idle, on, text) in &labels {
                self.ui.button(cx, *idle).set_text(cx, text);
                self.ui.button(cx, *on).set_text(cx, text);
            }
            self.ui.view(cx, ids!(split_panel)).set_visible(cx, choice == PayChoice::Split);
            self.ui
                .view(cx, ids!(cash_panel))
                .set_visible(cx, matches!(choice, PayChoice::Cash | PayChoice::Split));
            let method_word = match choice {
                PayChoice::Split => None,
                _ => labels
                    .iter()
                    .zip([PayChoice::Cash, PayChoice::Upi, PayChoice::Card, PayChoice::Credit])
                    .find(|(_, c)| *c == choice)
                    .map(|((_, _, t), _)| t.clone()),
            };
            let pay_text = match method_word {
                Some(m) => format!("{} · {} · {m}", st.t("take_payment"), st.money(st.cart.rounded_total())),
                None => format!("{} · {}", st.t("take_payment"), st.money(st.cart.rounded_total())),
            };
            self.ui.button(cx, ids!(pay_mix)).set_text(cx, &pay_text);
            self.ui.label(cx, ids!(total_word)).set_text(cx, &st.t("total"));
            // Menu vs applied only when a line was re-priced.
            self.ui
                .view(cx, ids!(total_detail))
                .set_visible(cx, st.cart.actual_total() != st.cart.applied_total());
            // The menu window: category tabs when the shop files its dishes
            // under categories; the order so far under its title, so nobody
            // closes it to check; a word when the search finds nothing.
            self.ui.view(cx, ids!(cat_strip)).set_visible(cx, !cats.is_empty());
            self.ui.label(cx, ids!(menu_modal_title)).set_text(cx, &st.t("add_dishes"));
            let summary = if st.cart.is_empty() {
                st.t("pick_dishes_help")
            } else {
                let qty: i64 = cart_qty_by_product(&st.cart.lines).values().sum();
                st.t("order_so_far")
                    .replace("{n}", &st.t("dishes_n").replace("{n}", &milli_as_count(qty)))
                    .replace("{total}", &st.money(st.cart.rounded_total()))
            };
            self.ui.label(cx, ids!(menu_modal_summary)).set_text(cx, &summary);
            self.ui.button(cx, ids!(menu_pick_done)).set_text(cx, &st.t("pick_done"));
            let frequent_empty = st.menu_pick_open
                && !searching
                && cat_sel.as_deref() == Some(MENU_FREQUENT)
                && st.visible_products().is_empty();
            let no_match = st.menu_pick_open && searching && st.visible_products().is_empty();
            self.ui
                .label(cx, ids!(menu_no_match))
                .set_visible(cx, no_match || frequent_empty);
            if no_match {
                self.ui
                    .label(cx, ids!(menu_no_match))
                    .set_text(cx, &st.t("no_dish_match").replace("{q}", st.menu_filter.trim()));
            } else if frequent_empty {
                self.ui
                    .label(cx, ids!(menu_no_match))
                    .set_text(cx, &st.t("frequent_empty"));
            }
            self.ui.button(cx, ids!(menu_open_btn)).set_text(cx, &format!("+  {}", st.t("add_dishes")));
            if st.menu_pick_open {
                self.ui.widget(cx, ids!(product_list)).redraw(cx);
            }
            let frequent_label = st.t("frequent");
            let frequent_on = cat_sel.as_deref() == Some(MENU_FREQUENT);
            self.ui.button(cx, ids!(cat_all)).set_text(cx, &all_label);
            self.ui.button(cx, ids!(cat_all_on)).set_text(cx, &all_label);
            self.paint_choice_pair(cx, ids!(cat_all), ids!(cat_all_on), cat_sel.is_none());
            self.ui.button(cx, ids!(cat_frequent)).set_text(cx, &frequent_label);
            self.ui.button(cx, ids!(cat_frequent_on)).set_text(cx, &frequent_label);
            self.paint_choice_pair(cx, ids!(cat_frequent), ids!(cat_frequent_on), frequent_on);
            self.ui.view(cx, ids!(frequent_config)).set_visible(cx, frequent_on);
            if frequent_on {
                self.ui
                    .widget(cx, ids!(frequent_config_help))
                    .set_text(cx, &plain_display_text(&st.t("frequent_config_help")));
                self.ui
                    .button(cx, ids!(frequent_config_save))
                    .set_text(cx, &st.t("frequent_apply"));
                self.ui
                    .text_input(cx, ids!(frequent_days_input))
                    .set_empty_text(cx, st.t("frequent_days_ph"));
                self.ui
                    .text_input(cx, ids!(frequent_limit_input))
                    .set_empty_text(cx, st.t("frequent_limit_ph"));
                // Only push values when the fields are empty so typing is not
                // fought by sync_chrome every frame.
                for (id, val) in [
                    (ids!(frequent_days_input), st.frequent_days.to_string()),
                    (ids!(frequent_limit_input), st.frequent_limit.to_string()),
                ] {
                    let field = self.ui.text_input(cx, id);
                    if field.text().trim().is_empty() {
                        field.set_text(cx, &val);
                    }
                }
            }
            for (i, (idle, on)) in CATEGORY_CHIPS.iter().enumerate() {
                match cats.get(i) {
                    Some(c) => {
                        self.ui.button(cx, idle).set_text(cx, &c.name);
                        self.ui.button(cx, on).set_text(cx, &c.name);
                        self.paint_choice_pair(cx, idle, on, cat_sel.as_deref() == Some(c.id.as_str()));
                    }
                    None => {
                        self.ui.button(cx, idle).set_visible(cx, false);
                        self.ui.button(cx, on).set_visible(cx, false);
                    }
                }
            }
            for (idle, on, kind) in [
                (ids!(otype_counter), ids!(otype_counter_on), OrderType::Counter),
                (ids!(otype_dinein), ids!(otype_dinein_on), OrderType::DineIn),
                (ids!(otype_takeaway), ids!(otype_takeaway_on), OrderType::Takeaway),
                (ids!(otype_delivery), ids!(otype_delivery_on), OrderType::Delivery),
            ] {
                self.paint_choice_pair(cx, idle, on, order_type == kind);
            }
            drop(st);
        }
        self.set_report_rows(cx, ids!(staff_label), staff_rows);
        self.ui
            .widget(cx, ids!(locale_label)).set_text(cx, &plain_display_text(&locale_line));

        // Country pack: the summary, the pay-rail wording on the Sell buttons,
        // and the exclusive-tax estimate under the totals strip.
        {
            let (fiscal_line, country_hint, tax_line, digital, business_id, service_pct) = {
                let st = app_state().lock().unwrap();
                (
                    st.fiscal.summary(),
                    match fiscal::country_by_code(&st.country_browse) {
                        Some(p) => {
                            let mut rows = vec![ReportRow::Head(format!(
                                "Browsing {} — tap Apply country pack to switch",
                                p.code
                            ))];
                            rows.extend(
                                p.apply_facts()
                                    .into_iter()
                                    .map(|(what, value)| ReportRow::Kv(what.to_string(), value)),
                            );
                            rows
                        }
                        // Nothing picked: the summary above already says it all.
                        None => Vec::new(),
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
            self.ui.widget(cx, ids!(fiscal_label)).set_text(cx, &plain_display_text(&fiscal_line));
            self.set_report_rows(cx, ids!(country_hint), country_hint);
            self.ui
                .widget(cx, ids!(tax_estimate_label)).set_text(cx, &plain_display_text(&tax_line));
            self.ui
                .widget(cx, ids!(tax_estimate_label))
                .set_visible(cx, tax_line.trim() != "");
            self.ui.button(cx, ids!(amend_pay_upi_on)).set_text(cx, &digital);
            self.ui.button(cx, ids!(amend_pay_upi)).set_text(cx, &digital);
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
            .widget(cx, ids!(empty_catalog)).set_text(cx, &plain_display_text(&empty_msg));
        self.ui
            .widget(cx, ids!(menu_empty)).set_text(cx, &plain_display_text(&empty_msg));
        self.ui
            .widget(cx, ids!(empty_catalog))
            .set_visible(cx, product_count == 0);
        self.ui
            .widget(cx, ids!(catalog_cta))
            .set_visible(cx, product_count == 0);
        self.ui
            .widget(cx, ids!(menu_empty))
            .set_visible(cx, product_count == 0);
        {
            let (unpriced, only, note, btn) = {
                let st = app_state().lock().unwrap();
                let n = st.unpriced_count();
                let btn = if st.menu_unpriced_only { st.t("show_all_dishes") } else { st.t("show_unpriced") };
                (n, st.menu_unpriced_only, st.t("unpriced_note").replace("{n}", &n.to_string()), btn)
            };
            self.ui
                .widget(cx, ids!(menu_unpriced_note))
                .set_text(cx, &plain_display_text(&note));
            self.ui.widget(cx, ids!(menu_unpriced_note)).set_visible(cx, unpriced > 0);
            self.ui.button(cx, ids!(menu_unpriced_btn)).set_text(cx, &btn);
            let low = {
                let st = app_state().lock().unwrap();
                if st.low_stock_line.is_empty() || st.device_role == DeviceRole::Client {
                    String::new()
                } else {
                    st.t("low_stock_alert").replace("{items}", &st.low_stock_line)
                }
            };
            let counts = {
                let st = app_state().lock().unwrap();
                if st.kitchen_counts_line.is_empty() {
                    String::new()
                } else {
                    st.t("kitchen_all_day").replace("{items}", &st.kitchen_counts_line)
                }
            };
            self.ui.widget(cx, ids!(kitchen_counts)).set_text(cx, &plain_display_text(&counts));
            self.ui.widget(cx, ids!(kitchen_counts)).set_visible(cx, !counts.is_empty());
            {
                let st = app_state().lock().unwrap();
                let station_rows = if st.stations.is_empty() {
                    vec![ReportRow::Line(st.t("stations_none"))]
                } else {
                    st.stations
                        .iter()
                        .enumerate()
                        .map(|(i, s)| ReportRow::Line(format!("{}. {}", i + 1, s.name)))
                        .collect()
                };
                drop(st);
                self.set_report_rows(cx, ids!(stations_label), station_rows);
            }
            {
                let (heading, rows, has) = {
                    let st = app_state().lock().unwrap();
                    let recent = st
                        .store
                        .as_ref()
                        .and_then(|s| s.list_recent_completed_kots(8).ok())
                        .unwrap_or_default();
                    let has = !recent.is_empty();
                    let rows: Vec<ReportRow> = recent
                        .into_iter()
                        .map(|k| {
                            let num = if k.kot_no > 0 {
                                format!("#{} ", k.kot_no)
                            } else {
                                String::new()
                            };
                            ReportRow::Line(format!(
                                "{num}{} · {}",
                                if k.station_name.is_empty() {
                                    st.t("kitchen")
                                } else {
                                    k.station_name
                                },
                                k.summary.chars().take(60).collect::<String>()
                            ))
                        })
                        .collect();
                    (st.t("recently_done"), rows, has)
                };
                self.ui
                    .widget(cx, ids!(recently_done_heading))
                    .set_text(cx, &plain_display_text(&heading));
                self.ui
                    .widget(cx, ids!(recently_done_heading))
                    .set_visible(cx, has);
                self.set_report_rows(cx, ids!(recently_done_label), rows);
                self.ui
                    .widget(cx, ids!(recently_done_label))
                    .set_visible(cx, has);
            }
            {
                let st = app_state().lock().unwrap();
                let none = st.reservations.is_empty();
                self.ui.widget(cx, ids!(resv_heading)).set_text(cx, &plain_display_text(&st.t("resv_heading")));
                self.ui.widget(cx, ids!(resv_empty)).set_text(cx, &plain_display_text(&st.t("resv_none")));
                self.ui.widget(cx, ids!(resv_empty)).set_visible(cx, none);
                self.ui.widget(cx, ids!(resv_list)).set_visible(cx, !none);
                self.ui.button(cx, ids!(resv_add)).set_text(cx, &st.t("resv_add"));
            }
            self.ui.widget(cx, ids!(resv_list)).redraw(cx);
            for id in [ids!(books_low_stock), ids!(buy_low_stock)] {
                self.ui.widget(cx, id).set_text(cx, &plain_display_text(&low));
                self.ui.widget(cx, id).set_visible(cx, !low.is_empty());
            }
            self.ui
                .widget(cx, ids!(menu_unpriced_btn))
                .set_visible(cx, unpriced > 0 || only);
        }
        self.ui
            .widget(cx, ids!(menu_open_btn))
            .set_visible(cx, product_count > 0);
        self.ui
            .widget(cx, ids!(product_list_menu))
            .set_visible(cx, product_count > 0);
        // Removed items — shown only while Remove has taken something off the
        // menu. Remove is an archive, so every row here can come back as it was.
        let removed_count = {
            let st = app_state().lock().unwrap();
            st.removed_products.len()
        };
        for path in [
            ids!(h_removed_items),
            ids!(removed_help),
            ids!(removed_list),
        ] {
            self.ui.widget(cx, path).set_visible(cx, removed_count > 0);
        }
        // Empty ticket: no bordered card (see cart_ticket_card). Hint unused.
        self.ui.widget(cx, ids!(cart_empty_hint)).set_visible(cx, false);
        self.ui
            .widget(cx, ids!(cart_list))
            .set_visible(cx, cart_count > 0);
        self.ui
            .widget(cx, ids!(cart_natural_label)).set_text(cx, &plain_display_text(&natural_label));
        self.ui
            .widget(cx, ids!(cart_applied_label)).set_text(cx, &plain_display_text(&applied_label));
        self.ui
            .widget(cx, ids!(cart_charge_caption)).set_text(cx, &plain_display_text(&charge_caption));
        self.ui
            .widget(cx, ids!(cart_round_hint)).set_text(cx, &plain_display_text(&round_hint));
        // Pinned Order strip: always show the running total next to order type.
        {
            let st = app_state().lock().unwrap();
            let sticky = format!(
                "{} · {}",
                st.t("total"),
                applied_label
            );
            drop(st);
            self.ui
                .label(cx, ids!(sticky_total))
                .set_text(cx, &sticky);
        }
        // Send KOT: name kitchens for unsent lines. After send, keep the station
        // field and say clearly that the last batch went — never a naked "Send KOT"
        // that looks broken when there is nothing left to send.
        {
            let st = app_state().lock().unwrap();
            let unassigned = st.t("kot_unassigned");
            let dests = pending_kot_destinations(
                &st.products,
                &st.stations,
                &st.cart.lines,
                &unassigned,
            );
            let has_unsent = st
                .cart
                .lines
                .iter()
                .any(|l| !l.is_sent() && !l.hold_send);
            let has_held = st
                .cart
                .lines
                .iter()
                .any(|l| !l.is_sent() && l.hold_send);
            let has_sent = st.cart.lines.iter().any(|l| l.is_sent());
            let ticket_open = !st.cart.lines.is_empty();
            let dest_line = format_kot_destinations(&dests);
            let sent_line = {
                let mut out: Vec<(String, usize)> = Vec::new();
                for line in st.cart.lines.iter().filter(|l| l.is_sent()) {
                    let sid = st
                        .products
                        .iter()
                        .find(|p| p.id == line.product_id)
                        .map(|p| p.station_id.as_str())
                        .unwrap_or("");
                    let name = if sid.is_empty() {
                        unassigned.clone()
                    } else {
                        st.stations
                            .iter()
                            .find(|s| s.id == sid)
                            .map(|s| s.name.clone())
                            .unwrap_or_else(|| unassigned.clone())
                    };
                    match out.iter_mut().find(|(n, _)| *n == name) {
                        Some((_, n)) => *n += 1,
                        None => out.push((name, 1)),
                    }
                }
                format_kot_destinations(&out)
            };
            let (btn, hint, hint_on) = if !dests.is_empty() {
                (
                    st.t("send_kot_to").replace("{kitchens}", &dest_line),
                    st.t("kot_dest_hint").replace("{kitchens}", &dest_line),
                    true,
                )
            } else if has_held && !has_unsent {
                (st.t("send_kot_held"), st.t("hold_send_meta"), true)
            } else if has_sent {
                let kitchens = if sent_line.is_empty() {
                    st.t("kitchen")
                } else {
                    sent_line.clone()
                };
                (
                    st.t("send_kot_again"),
                    st.t("kot_sent_hint").replace("{kitchens}", &kitchens),
                    true,
                )
            } else {
                (st.t("send_kot"), String::new(), false)
            };
            drop(st);
            self.ui.button(cx, ids!(send_kot_btn)).set_text(cx, &btn);
            self.ui
                .widget(cx, ids!(kot_dest_help))
                .as_info_help()
                .set_tip(&plain_display_text(&hint));
            self.ui.widget(cx, ids!(kot_dest_help)).set_visible(cx, hint_on);
            // Station picker stays while the ticket has dishes (sent or not).
            self.ui
                .view(cx, ids!(kot_station_wrap))
                .set_visible(cx, ticket_open);
            self.ui
                .button(cx, ids!(kot_set_station))
                .set_visible(cx, has_unsent);
        }
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
            .widget(cx, ids!(undo_snack_label)).set_text(cx, &plain_display_text(&undo_snack_text));
        self.paint_app_snackbar(cx);

        let (credit_chip, credit_summary, credit_recent, empty_hint) = {
            let st = app_state().lock().unwrap();
            let credit_chip = match &st.selected_customer_id {
                Some(id) => st
                    .customers
                    .iter()
                    .find(|c| &c.id == id)
                    .map(|c| {
                        let mut base = if c.balance_paise > 0 {
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
                        };
                        if c.loyalty_points > 0 || c.wallet_paise > 0 {
                            base.push_str(&format!(
                                " · {} pts · wallet {}",
                                c.loyalty_points,
                                st.money(Money(c.wallet_paise))
                            ));
                        }
                        base
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
            let open: i64 = st.customers.iter().map(|c| c.balance_paise.max(0)).sum();
            let prepaid: i64 = st
                .customers
                .iter()
                .map(|c| (-c.balance_paise).max(0))
                .sum();
            let credit_summary_visible = amount_key_visible(
                st.amounts_screen_hidden,
                &st.amount_keys_hidden,
                &st.amount_keys_peek,
                "credit_summary",
            );
            // Per-language sentence with {} slots filled in order — mask money only.
            let credit_summary = st
                .t("credit_summary_fmt")
                .replacen("{}", &st.customers.len().to_string(), 1)
                .replacen(
                    "{}",
                    &display_money(&st.money(Money(open)), credit_summary_visible),
                    1,
                )
                .replacen(
                    "{}",
                    &display_money(&st.money(Money(prepaid)), credit_summary_visible),
                    1,
                );
            // The picked party's latest ledger entries, one row each.
            let mut credit_recent = Vec::new();
            if let Some(id) = &st.selected_customer_id
                && let Some(store) = st.store.as_ref()
                && let Ok(entries) = store.list_credit_ledger(id, 6)
                && !entries.is_empty()
            {
                credit_recent.push(report_head(&st.t("recent_label")));
                credit_recent.extend(entries.iter().map(|entry| {
                    ReportRow::Kv(
                        format!("{} · {}", &entry.created_at[..entry.created_at.len().min(10)], entry.kind),
                        st.money(Money(entry.amount_paise)),
                    )
                }));
            }
            let empty_hint = if st.customers.is_empty() {
                "No saved customers yet — type a name above."
            } else {
                "Tap Pick to use a saved customer."
            };
            (credit_chip, credit_summary, credit_recent, empty_hint)
        };
        {
            let (selected_label, form_btn_label) = {
                let st = app_state().lock().unwrap();
                match st
                    .selected_customer_id
                    .as_ref()
                    .and_then(|id| st.customers.iter().find(|c| &c.id == id))
                {
                    Some(c) => {
                        let kind = if c.kind_label.trim().is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", c.kind_label.trim())
                        };
                        (
                            format!("Editing: {}{}", c.name, kind),
                            "Save changes",
                        )
                    }
                    // A new party needs no instructions: the form and Add party say it.
                    None => (String::new(), "Add party"),
                }
            };
            self.ui
                .widget(cx, ids!(party_selected_label))
                .set_text(cx, &plain_display_text(&selected_label));
            self.ui
                .button(cx, ids!(credit_add_btn))
                .set_text(cx, form_btn_label);
        }
        self.ui
            .widget(cx, ids!(credit_chip)).set_text(cx, &plain_display_text(&credit_chip));
        self.ui
            .widget(cx, ids!(credit_summary_label))
            .set_text(cx, &plain_display_text(&credit_summary));
        self.set_report_rows(cx, ids!(credit_recent_list), credit_recent);
        self.ui
            .widget(cx, ids!(customers_empty_hint)).set_text(cx, &plain_display_text(empty_hint));
        {
            let st = app_state().lock().unwrap();
            let has_customer = st.selected_customer_id.is_some();
            self.ui
                .view(cx, ids!(till_loyalty_panel))
                .set_visible(cx, has_customer);
            let till_line = match st
                .selected_customer_id
                .as_ref()
                .and_then(|id| st.customers.iter().find(|c| &c.id == id))
            {
                Some(c) => {
                    let pending = pending_till_redeem_paise(&st);
                    let mut s = format!(
                        "{} pts · wallet {} · bday {}",
                        c.loyalty_points,
                        st.money(Money(c.wallet_paise)),
                        if c.birthday.is_empty() {
                            "—"
                        } else {
                            c.birthday.as_str()
                        }
                    );
                    if pending > 0 {
                        s.push_str(&format!(" · redeeming {}", st.money(Money(pending))));
                    }
                    s
                }
                None => String::new(),
            };
            self.ui
                .widget(cx, ids!(till_loyalty_chip))
                .set_text(cx, &plain_display_text(&till_line));
            let crm_text = st
                .store
                .as_ref()
                .and_then(|s| s.crm_segment_preview(st.crm_segment).ok())
                .unwrap_or_else(|| "Open as Server to preview guests.".into());
            drop(st);
            self.set_report_rows(cx, ids!(crm_preview), parse_report(&crm_text));
        }

        let (sale_selected, sale_history, amend_total_text, amend_natural, amend_applied, amend_round) = {
            let st = app_state().lock().unwrap();
            let sale_selected = if let Some(id) = &st.selected_sale_id {
                let short = &id[..8.min(id.len())];
                let lines = st
                    .amend_cart
                    .lines
                    .iter()
                    .map(|l| {
                        if l.note.is_empty() {
                            format!("{}×{} @{}", l.qty, l.name, st.money(l.unit_price))
                        } else {
                            format!(
                                "{}×{} @{} ({})",
                                l.qty,
                                l.name,
                                st.money(l.unit_price),
                                l.note
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                // The lines and totals are right below; the header only says
                // which bill this is.
                let _ = (short, lines);
                format!(
                    "{} · {}",
                    st.money(st.amend_cart.rounded_total()),
                    st.amend_pay.label_for(&st.fiscal)
                )
            } else {
                "No sale selected".into()
            };
            // Each change to the bill on its own row: who, what it was, why — and when.
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
            let amend_natural = format!(
                "Actual Total {}",
                st.money(st.amend_cart.actual_total())
            );
            let amend_applied = format!(
                "Applied Total {}",
                st.money(st.amend_cart.applied_total())
            );
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
                st.session
                    .as_ref()
                    .map(|s| s.role.can_void_refund())
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
            .widget(cx, ids!(sale_selected_label)).set_text(cx, &plain_display_text(&sale_selected));
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
        self.set_report_rows(cx, ids!(sale_history_label), sale_history);
        self.ui
            .widget(cx, ids!(amend_natural_label)).set_text(cx, &plain_display_text(&amend_natural));
        self.ui
            .widget(cx, ids!(amend_applied_label)).set_text(cx, &plain_display_text(&amend_applied));
        self.ui
            .widget(cx, ids!(amend_round_hint)).set_text(cx, &plain_display_text(&amend_round));
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
        self.ui.widget(cx, ids!(dish_pick_list)).redraw(cx);
        self.ui.widget(cx, ids!(category_pick_list)).redraw(cx);
        // Ticket: table actions, move/merge panel, the sent-line panel, and
        // Settle & Save's change to return.
        {
            let tendered_text = self.ui.text_input(cx, ids!(tendered_input)).text();
            let tip_text = self.ui.text_input(cx, ids!(tip_input)).text();
            let cash_text = self.ui.text_input(cx, ids!(mix_cash_input)).text();
            let (table_open, move_open, line_action, cooked, line_nc, change_line, lang) = {
                let st = app_state().lock().unwrap();
                let currency = st.locale.currency.clone();
                let parse = |t: &str| -> i64 {
                    let t = t.trim();
                    if t.is_empty() { 0 } else { locale::parse_major_input(t, &currency).unwrap_or(0) }
                };
                let charged = st
                    .cart
                    .to_sale_draft()
                    .ok()
                    .and_then(|d| st.store.as_ref().and_then(|s| s.preview_sale_total(&d).ok()))
                    .unwrap_or(0);
                let tendered = parse(&tendered_text);
                let tip = parse(&tip_text);
                // Cash typed in the mix is what the bill takes in cash; with
                // no mix typed, the whole bill is paid from what was handed over.
                let cash_part = if cash_text.trim().is_empty() { charged } else { parse(&cash_text) };
                let mut parts = Vec::new();
                if tendered > 0 {
                    let change = tendered - cash_part - tip;
                    parts.push(if change >= 0 {
                        format!("{}: {}", st.t("return_to_customer"), st.money(Money(change)))
                    } else {
                        format!("{} {}", st.t("short_by"), st.money(Money(-change)))
                    });
                }
                if tip > 0 {
                    parts.push(format!("{}: {}", st.t("tip_not_sale"), st.money(Money(tip))));
                }
                let line_nc = st.line_action.as_ref().and_then(|(id, _)| {
                    st.cart.lines.iter().find(|l| &l.running_line_id == id).map(|l| l.complimentary)
                });
                // Bill / move only once the kitchen has something for this
                // table — before that there is nothing to bill or move.
                (
                    st.selected_table_id.is_some()
                        && st.screen == Screen::Pos
                        && st.cart.lines.iter().any(|l| l.is_sent()),
                    st.move_panel_open,
                    st.line_action.as_ref().map(|(_, a)| *a),
                    st.line_cooked,
                    line_nc.unwrap_or(false),
                    if parts.is_empty() { " ".to_string() } else { parts.join(" · ") },
                    st.locale.language.clone(),
                )
            };
            let t = |k: &str| locale::t(&lang, k);
            self.ui.view(cx, ids!(table_actions)).set_visible(cx, table_open);
            {
                let st = app_state().lock().unwrap();
                let splitting = table_open && st.split_mode;
                self.ui.view(cx, ids!(split_bar)).set_visible(cx, splitting);
                self.ui.button(cx, ids!(table_split_btn)).set_text(cx, &st.t("split_bill"));
                self.ui.button(cx, ids!(split_evenly_btn)).set_text(cx, &st.t("split_evenly"));
                self.ui.button(cx, ids!(fire_course_btn)).set_text(cx, &st.t("fire_course"));
                self.ui.button(cx, ids!(hold_item_btn)).set_text(cx, &st.t("hold_item"));
                self.ui.button(cx, ids!(waiter_call_btn)).set_text(cx, &st.t("waiter_call"));
                self.ui.button(cx, ids!(split_go)).set_text(cx, &st.t("split_go"));
                self.ui.button(cx, ids!(split_cancel)).set_text(cx, &st.t("cancel"));
                let help = st.t("split_help").replace("{n}", &st.split_pick.len().to_string());
                self.ui.widget(cx, ids!(split_help)).set_text(cx, &plain_display_text(&help));
                self.ui
                    .widget(cx, ids!(split_bill_help))
                    .as_info_help()
                    .set_tip(&plain_display_text(&help));
            }
            self.ui.view(cx, ids!(table_move_panel)).set_visible(cx, table_open && move_open);
            self.ui
                .view(cx, ids!(line_action_panel))
                .set_visible(cx, table_open && line_action.is_some());
            if let Some(action) = line_action {
                let (title, reason, qty, cooked_btn, target) = match action {
                    LineAction::Cancel => ("cancel_item_title", true, true, true, false),
                    LineAction::Complimentary if line_nc => ("charge_again_title", false, false, false, false),
                    LineAction::Complimentary => ("nc_item_title", true, false, false, false),
                    LineAction::Move => ("move_item_title", false, false, false, true),
                };
                self.ui.widget(cx, ids!(line_action_title)).set_text(cx, &plain_display_text(&t(title)));
                self.ui.view(cx, ids!(line_reason_wrap)).set_visible(cx, reason);
                self.ui.view(cx, ids!(line_qty_wrap)).set_visible(cx, qty);
                self.ui.view(cx, ids!(line_cooked_wrap)).set_visible(cx, cooked_btn);
                if cooked_btn {
                    self.paint_choice_pair(
                        cx,
                        ids!(line_cooked_raw),
                        ids!(line_cooked_raw_on),
                        !cooked,
                    );
                    self.paint_choice_pair(
                        cx,
                        ids!(line_cooked_done),
                        ids!(line_cooked_done_on),
                        cooked,
                    );
                    for path in [ids!(line_cooked_raw), ids!(line_cooked_raw_on)] {
                        self.ui.button(cx, path).set_text(cx, &t("not_cooked_yet"));
                    }
                    for path in [ids!(line_cooked_done), ids!(line_cooked_done_on)] {
                        self.ui.button(cx, path).set_text(cx, &t("already_cooked"));
                    }
                }
                self.ui.view(cx, ids!(line_target_wrap)).set_visible(cx, target);
            }
            self.ui.widget(cx, ids!(change_label)).set_text(cx, &plain_display_text(&change_line));
            let (discount_open, discount_line) = {
                let st = app_state().lock().unwrap();
                let line = match &st.cart.discount {
                    Some(d) => format!(
                        "{} −{} · {}",
                        st.t("discount"),
                        st.money(Money(st.cart.discount_paise())),
                        d.reason
                    ),
                    None => " ".to_string(),
                };
                (st.discount_panel_open, line)
            };
            self.ui.view(cx, ids!(discount_panel)).set_visible(cx, discount_open);
            self.ui.widget(cx, ids!(discount_label)).set_text(cx, &plain_display_text(&discount_line));
        }
        // Day close: this shift so far (what the drawer should hold).
        {
            let rows = {
                let st = app_state().lock().unwrap();
                match (st.screen, st.store.as_ref(), st.session.as_ref()) {
                    (Screen::DayClose, Some(store), Some(actor)) if st.device_role.is_host() => {
                        store
                            .shift_so_far(actor)
                            .map(|s| {
                                vec![
                                    ReportRow::Head(format!(
                                        "{} {}",
                                        st.t("shift_since"),
                                        s.started_at.get(11..16).unwrap_or("")
                                    )),
                                    ReportRow::Kv(st.t("sales"), s.bills.to_string()),
                                    ReportRow::Kv(st.t("cash"), st.money(Money(s.cash_paise))),
                                    ReportRow::Kv(st.t("expected_in_drawer"), st.money(Money(s.expected_paise))),
                                ]
                            })
                            .unwrap_or_default()
                    }
                    _ => Vec::new(),
                }
            };
            self.set_report_rows(cx, ids!(shift_so_far_label), rows);
        }
        // Floor: summary line and the colour key's words.
        {
            let (summary, legend) = {
                let st = app_state().lock().unwrap();
                let busy = st.tables.iter().filter(|t| t.status.is_open()).count();
                let to_collect = st
                    .tables
                    .iter()
                    .filter(|t| t.floor_state == domain::FloorState::BillPrinted)
                    .count();
                let on_floor: i64 = st.tables.iter().map(|t| t.running_total_paise).sum();
                let floor_visible = amount_key_visible(
                    st.amounts_screen_hidden,
                    &st.amount_keys_hidden,
                    &st.amount_keys_peek,
                    "floor_summary",
                );
                let summary = format!(
                    "{busy}/{} {} · {to_collect} {} · {} {}",
                    st.tables.len(),
                    st.t("summary_busy"),
                    st.t("summary_to_collect"),
                    display_money(&st.money(Money(on_floor)), floor_visible),
                    st.t("summary_on_floor")
                );
                let legend: Vec<String> = domain::FloorState::ALL.iter().map(|s| st.t(s.tile_key())).collect();
                (summary, legend)
            };
            self.ui
                .widget(cx, ids!(floor_summary))
                .set_text(cx, &plain_display_text(&summary));
            for (id, text) in [
                ids!(lg_free),
                ids!(lg_seated),
                ids!(lg_kitchen),
                ids!(lg_ready),
                ids!(lg_eating),
                ids!(lg_bill),
                ids!(lg_paid),
            ]
            .into_iter()
            .zip(legend.iter())
            {
                self.ui.label(cx, id).set_text(cx, text);
            }
            // Floor is grid-only — no Grid/Layout tabs.
            {
                let mut st = app_state().lock().unwrap();
                st.floor_view_layout = false;
                st.floor_layout_sel = None;
            }
            self.ui.widget(cx, ids!(floor_list)).set_visible(cx, true);
            self.ui.widget(cx, ids!(floor_layout_list)).set_visible(cx, false);
            self.ui.view(cx, ids!(floor_layout_editor)).set_visible(cx, false);
        }
        // Team: who is being edited, and their rights (draft until Save).
        {
            let (selected, rights, lang) = {
                let st = app_state().lock().unwrap();
                let selected = st
                    .team_staff_id
                    .as_ref()
                    .and_then(|id| st.staff.iter().find(|s| &s.id == id))
                    .map(|s| format!("{}: {} ({})", st.t("editing_staff"), s.name, s.role_title))
                    .unwrap_or_else(|| " ".to_string());
                (selected, st.team_draft_rights(), st.locale.language.clone())
            };
            let on = app_state().lock().unwrap().draft_features.clone();
            for (idle, active, set) in [
                (ids!(feat_sell), ids!(feat_sell_on), on.sell),
                (ids!(feat_items), ids!(feat_items_on), on.items),
                (ids!(feat_sales), ids!(feat_sales_on), on.sales),
                (ids!(feat_credit), ids!(feat_credit_on), on.credit),
                (ids!(feat_floor), ids!(feat_floor_on), on.floor),
                (ids!(feat_kitchen), ids!(feat_kitchen_on), on.kitchen),
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
                (ids!(right_settle_btn), domain::Right::Settle),
                (ids!(right_cancel_btn), domain::Right::CancelSentItem),
                (ids!(right_nc_btn), domain::Right::Complimentary),
                (ids!(right_move_btn), domain::Right::MoveTable),
                (ids!(right_reprint_btn), domain::Right::Reprint),
                (ids!(right_discount_btn), domain::Right::Discount),
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

        self.show_screen(cx, screen);
        if screen == Screen::Settings {
            self.sync_settings_panels(cx);
        }
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
                .widget(cx, ids!(server_pair_label)).set_text(cx, &plain_display_text(&pair_label));
            self.ui
                .widget(cx, ids!(pairing_code_value))
                .set_text(cx, &plain_display_text(&pair_code));
            self.ui
                .widget(cx, ids!(pairing_code_value))
                .set_visible(cx, show_pair_code);
            self.ui
                .widget(cx, ids!(settings_tmpl_label))
                .set_text(cx, &plain_display_text(&tmpl_line));
            {
                let on = {
                    let st = app_state().lock().unwrap();
                    st.store
                        .as_ref()
                        .map(|s| s.kot_auto_print())
                        .unwrap_or(true)
                };
                self.paint_choice_pair(cx, ids!(kot_print_off), ids!(kot_print_off_on), !on);
                self.paint_choice_pair(cx, ids!(kot_print_on), ids!(kot_print_on_on), on);
                {
                    let st = app_state().lock().unwrap();
                    for path in [ids!(kot_print_off), ids!(kot_print_off_on)] {
                        self.ui.button(cx, path).set_text(cx, &st.t("kot_print_off"));
                    }
                    for path in [ids!(kot_print_on), ids!(kot_print_on_on)] {
                        self.ui.button(cx, path).set_text(cx, &st.t("kot_print_on"));
                    }
                }
                let mut st = app_state().lock().unwrap();
                // Every kitchen station gets its own route row (empty until configured).
                let station_ids: Vec<String> = st.stations.iter().map(|s| s.id.clone()).collect();
                st.printer_draft.ensure_station_routes(station_ids);
                let selected = st.printer_selected_id.clone();
                let pool_rows =
                    printers_pool_rows(&st.printer_draft, selected.as_deref(), |k| st.t(k));
                let add_labels: Vec<String> = st
                    .printer_draft
                    .printers
                    .iter()
                    .take(ROUTE_ADD_PRINTER_BTNS.len())
                    .map(|p| p.label.clone())
                    .collect();
                // Commit typed Bill/kitchen when the field is not focused —
                // never while typing (that made "Bill" immortal after backspace).
                let route_field = self.ui.text_input(cx, ids!(route_pick_input));
                let route_focused = cx.has_key_focus(route_field.area());
                if !route_focused {
                    let typed = route_field.text();
                    let current_display = printer_route_display_name(
                        &st.printer_route_key,
                        &st.stations,
                        |k| st.t(k),
                    );
                    if typed != current_display {
                        let new_key =
                            resolve_printer_route_key(&typed, &st.stations, |k| st.t(k));
                        if new_key != st.printer_route_key {
                            st.printer_route_key = new_key;
                        }
                    }
                }
                let route_display = printer_route_display_name(
                    &st.printer_route_key,
                    &st.stations,
                    |k| st.t(k),
                );
                if !route_focused && route_field.text() != route_display {
                    route_field.set_text(cx, &route_display);
                }
                let route_rows = printer_route_rows(
                    &st.printer_draft,
                    &st.stations,
                    &st.printer_route_key,
                    |k| st.t(k),
                );
                let editing = st
                    .t("route_editing")
                    .replace("{name}", &route_display);
                drop(st);
                self.show_found_printers(cx);
                self.set_report_rows(cx, ids!(printers_pool_label), pool_rows);
                self.set_report_rows(cx, ids!(printer_routes_label), route_rows);
                self.ui
                    .widget(cx, ids!(route_editing_label))
                    .set_text(cx, &plain_display_text(&editing));
                for (i, id) in ROUTE_ADD_PRINTER_BTNS.iter().enumerate() {
                    let btn = self.ui.button(cx, id);
                    if let Some(label) = add_labels.get(i) {
                        btn.set_text(cx, label);
                        btn.set_visible(cx, true);
                    } else {
                        btn.set_visible(cx, false);
                    }
                }
                self.ui
                    .view(cx, ids!(route_add_printers))
                    .set_visible(cx, !add_labels.is_empty());
            }
            let units_rows = {
                let st = app_state().lock().unwrap();
                st.store
                    .as_ref()
                    .map(|s| units_report_rows(s, |k| st.t(k)))
                    .unwrap_or_default()
            };
            self.set_report_rows(cx, ids!(units_label), units_rows);
            let sibling_rows = {
                let st = app_state().lock().unwrap();
                st.store
                    .as_ref()
                    .map(|s| sibling_pickers_rows(s, |k| st.t(k)))
                    .unwrap_or_default()
            };
            self.set_report_rows(cx, ids!(sibling_pickers_label), sibling_rows);
            {
                let filter = self.ui.text_input(cx, ids!(floor_waiter_filter)).text();
                let mut st = app_state().lock().unwrap();
                st.floor_waiter_filter = filter;
                let ops = st
                    .store
                    .as_ref()
                    .and_then(|s| s.ops_panel_text().ok())
                    .unwrap_or_else(|| st.ops_panel_text.clone());
                drop(st);
                self.ui
                    .widget(cx, ids!(floor_ops_label))
                    .set_text(cx, &plain_display_text(&ops));
            }
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
            // Books home stat cards: label + value + per-figure eye.
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
            self.ui
                .widget(cx, ids!(inventory_label)).set_text(cx, &plain_display_text(&inventory_text));
            self.ui
                .widget(cx, ids!(branches_label)).set_text(cx, &plain_display_text(&branches_body));
            {
                let field = self.ui.text_input(cx, ids!(org_name_input));
                if !cx.has_key_focus(field.area()) && field.text() != org_name {
                    field.set_text(cx, &org_name);
                }
            }
            self.ui
                .widget(cx, ids!(onboard_status)).set_text(cx, &plain_display_text(&onboard_status));
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
            // What this device still owes the shop, and what the shop would
            // not take — said here so nothing waits or vanishes unseen.
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
        
        // Role-aware More rows / tabs
        {
            let (
                _sell,
                items,
                sales,
                credit,
                floor_feat,
                kitchen_feat,
                team,
                backup,
                inventory,
                branches,
                is_owner,
            ) = {
                let st = app_state().lock().unwrap();
                match &st.session {
                    Some(s) => (
                        s.can_access_sell(),
                        s.can_access_items(),
                        s.can_access_sales(),
                        s.can_access_credit(),
                        s.can_access_floor() || s.is_owner(),
                        s.can_access_kitchen() || s.is_owner(),
                        s.can_access_team() || s.is_owner(),
                        s.can_access_backup() || s.is_owner(),
                        s.can_access_inventory() || s.is_owner(),
                        s.can_access_branches() || s.is_owner(),
                        s.is_owner(),
                    ),
                    None => (
                        false, false, false, false, false, false, false, false, false, false,
                        false,
                    ),
                }
            };
            let (is_client, is_shop, extra_tills, branch_count) = {
                let st = app_state().lock().unwrap();
                let branch_count = st
                    .store
                    .as_ref()
                    .and_then(|s| s.list_branches().ok())
                    .map(|b| b.len())
                    .unwrap_or(1);
                (
                    st.device_role == DeviceRole::Client,
                    st.device_role == DeviceRole::Server,
                    st.extra_tills_allowed,
                    branch_count,
                )
            };
            let _ = is_client;
            let books_ok = sales || is_owner || items;
            let buy_ok = inventory || sales || is_owner;
            // Progressive depth (vision §30): Floor/Kitchen peers exist only
            // when this shop runs tables & kitchen — a counter-only shop
            // never sees the floor.
            let floor_on = app_state().lock().unwrap().venue_mode.creates_kot();
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
                .button(cx, ids!(autostart_btn))
                .set_visible(cx, is_shop && is_owner && extra_tills && startup::supported());
            self.ui
                .widget(cx, ids!(paired_list))
                .set_visible(cx, is_shop && is_owner && extra_tills);
            // Emergency unlock PIN — Owner on the Server books only.
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
            {
                let paired = app_state().lock().unwrap().client_paired;
                self.ui.button(cx, ids!(client_connection_btn)).set_visible(cx, is_client && paired);
                // Only after Pair now — opens the PIN screen (not a second unlock action).
                self.ui
                    .button(cx, ids!(client_skip_lock))
                    .set_visible(cx, is_client && paired);
            }
            // Slots pack the visible arrangement; capability gates still apply.
            // (Stock/branches stay Server-only: a Client's copy is a sync
            // cache, so every write there would be lost.)
            let order = {
                let st = app_state().lock().unwrap();
                nav_display_order(&st.nav_order, &st.nav_hidden)
            };
            for rail in [false, true] {
                let wrappers = self.nav_slot_wrappers(rail);
                let pairs = if rail {
                    self.rail_nav_pairs()
                } else {
                    self.bottom_nav_pairs()
                };
                for slot in 0..NAV_TABS.len() {
                    let Some(tab) = order.get(slot).copied() else {
                        self.ui.view(cx, wrappers[slot]).set_visible(cx, false);
                        continue;
                    };
                    let visible = match tab {
                        NavTab::Sell | NavTab::Lock => true,
                        NavTab::Books => books_ok,
                        NavTab::Buy => buy_ok,
                        NavTab::Menu => items,
                        NavTab::Floor => floor_feat && floor_on,
                        NavTab::Kitchen => kitchen_feat && floor_on,
                        NavTab::Sales => sales,
                        NavTab::DayClose | NavTab::Reports => sales || is_owner,
                        NavTab::Inventory => inventory && !is_client,
                        NavTab::Branches => branches && !is_client && branch_count > 1,
                        NavTab::Credit => credit,
                        NavTab::Team => team,
                        NavTab::Settings => backup,
                    };
                    self.ui.view(cx, wrappers[slot]).set_visible(cx, visible);
                    let enabled = match tab {
                        NavTab::Books => books_ok,
                        NavTab::Buy => buy_ok,
                        _ => true,
                    };
                    self.ui.button(cx, pairs[slot].0).set_enabled(cx, enabled);
                    self.ui.button(cx, pairs[slot].1).set_enabled(cx, enabled);
                }
            }
        }

        self.lift_footer_above_system_bar(cx);
        self.repair_focused_carets(cx);
        self.ui.redraw(cx);
    }
}

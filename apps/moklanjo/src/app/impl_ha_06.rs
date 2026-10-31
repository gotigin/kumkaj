impl App {
    fn handle_actions_reports_backup(&mut self, cx: &mut Cx, actions: &Actions) {
        // Spreadsheet (CSV): copy out, or bring parties / items in.
        for (btn, which) in [
            (ids!(csv_copy_day), "day"),
            (ids!(csv_copy_outstanding), "outstanding"),
            (ids!(csv_copy_parties), "parties"),
            (ids!(csv_copy_items), "items"),
            (ids!(csv_copy_ledger), "ledger"),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let text = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    match which {
                        "day" => store.day_book_csv(),
                        "outstanding" => store.outstanding_csv(),
                        "parties" => store.parties_csv(),
                        "items" => store.items_csv(),
                        _ => {
                            let id = st
                                .selected_customer_id
                                .clone()
                                .ok_or_else(|| "Pick a party first (Parties tab)".to_string())?;
                            store.party_ledger_csv(&id)
                        }
                    }
                })();
                match text {
                    Ok(text) => {
                        self.copy_text_to_clipboard(cx, &text);
                        self.set_status(cx, "CSV copied — paste into your spreadsheet");
                    }
                    Err(e) => self.set_status(cx, e),
                }
            }
        }
        if self
            .ui
            .button(cx, ids!(csv_import_parties))
            .clicked(actions)
        {
            let dialog = FileDialog::new()
                .set_title("Choose parties CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_parties_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
        }
        if self.ui.button(cx, ids!(csv_import_items)).clicked(actions) {
            let dialog = FileDialog::new()
                .set_title("Choose items / rates CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_items_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
        }
        if self
            .ui
            .button(cx, ids!(csv_import_pincodes))
            .clicked(actions)
        {
            let dialog = FileDialog::new()
                .set_title("Choose pincodes CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_pincodes_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
        }
        if self
            .ui
            .button(cx, ids!(csv_import_carrier_bill))
            .clicked(actions)
        {
            let dialog = FileDialog::new()
                .set_title("Choose carrier bill CSV".into())
                .add_filter("CSV".into(), vec!["csv".into(), "txt".into()])
                .set_id(live_id!(csv_carrier_bill_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening CSV picker…");
        }
        if self
            .ui
            .button(cx, ids!(csv_copy_carrier_bill))
            .clicked(actions)
        {
            let text = {
                let st = app_state().lock().unwrap();
                st.last_carrier_bill_report.clone()
            };
            if text.trim().is_empty() {
                self.set_status(cx, "Import a carrier bill CSV first");
            } else {
                self.copy_text_to_clipboard(cx, &text);
                self.set_status(cx, "Carrier bill report copied");
            }
        }
        if self
            .ui
            .button(cx, ids!(booking_apply_carrier_cost))
            .clicked(actions)
        {
            let typed = self
                .ui
                .text_input(cx, ids!(booking_carrier_pick_input))
                .text();
            let outcome = {
                let st = app_state().lock().unwrap();
                let currency = st.locale.currency.clone();
                st.last_carrier_compare
                    .iter()
                    .find(|r| {
                        r.carrier_name.eq_ignore_ascii_case(typed.trim())
                            || r.carrier_party_id == typed.trim()
                    })
                    .map(|r| {
                        (
                            r.carrier_name.clone(),
                            locale::format_major_input(r.cost_paise, &currency),
                        )
                    })
            };
            match outcome {
                Some((name, amount)) => {
                    self.ui
                        .text_input(cx, ids!(booking_carrier_cost_input))
                        .set_text(cx, &amount);
                    {
                        let mut st = app_state().lock().unwrap();
                        st.booking_carrier_cost = amount;
                    }
                    self.set_status(cx, format!("Carrier cost set from {name}"));
                }
                None => self.set_status(
                    cx,
                    "Type a carrier name from the compare line (e.g. Delhivery)",
                ),
            }
            self.sync_chrome(cx);
        }

        let inv_minus = self.ui.button(cx, ids!(inv_adjust_minus)).clicked(actions);
        let inv_plus = self.ui.button(cx, ids!(inv_adjust_plus)).clicked(actions);
        if inv_minus || inv_plus {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(inv_qty_input)).text();
            let note = self.ui.text_input(cx, ids!(inv_note_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Adjust stock")?;
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let qty: i64 = qty_text
                    .trim()
                    .parse()
                    .map_err(|_| "Quantity must be a whole number".to_string())?;
                if qty <= 0 {
                    return Err("Quantity must be more than zero".into());
                }
                let product = st.find_product_by_name(&name)?;
                if !product.tracks_stock() {
                    return Err(format!(
                        "{} is not stock tracked — tap Start tracking first",
                        product.name
                    ));
                }
                let delta = if inv_plus { qty } else { -qty };
                let note = note.trim().to_string();
                let reason = if note.is_empty() {
                    "adjust"
                } else {
                    note.as_str()
                };
                let on_hand = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .adjust_stock(&product.id, delta, reason, &note, &actor)?;
                st.reload_all()?;
                st.refresh_inventory_text()?;
                Ok(format!("{} · {on_hand} on hand", product.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_qty_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(inv_track)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(inv_qty_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Start stock tracking")?;
                let actor = st.actor()?;
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let product = st.find_product_by_name(&name)?;
                // Start at zero, then book the typed count as an opening move so
                // the paper trail shows where the first quantity came from.
                let opening: i64 = match qty_text.trim() {
                    "" => 0,
                    other => other
                        .parse()
                        .map_err(|_| "Quantity must be a whole number".to_string())?,
                };
                if opening < 0 {
                    return Err("Quantity cannot be negative".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_product_stock_qty(&product.id, 0)?;
                if opening > 0 {
                    store.adjust_stock(&product.id, opening, "count", "opening count", &actor)?;
                }
                st.reload_all()?;
                st.refresh_inventory_text()?;
                Ok(format!("{} now tracked · {opening} on hand", product.name))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_qty_input)).set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(inv_purchase)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(inv_item_input)).text();
            let qty_text = self.ui.text_input(cx, ids!(inv_qty_input)).text();
            let cost_text = self.ui.text_input(cx, ids!(inv_cost_input)).text();
            let supplier = self.ui.text_input(cx, ids!(inv_supplier_input)).text();
            let note = self.ui.text_input(cx, ids!(inv_note_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Record purchases")?;
                let actor = st.actor()?;
                let qty_milli = parse_qty_to_milli(&qty_text)?;
                let product = st.find_product_by_name(&name)?;
                let currency = st.locale.currency.clone();
                let unit_cost = match cost_text.trim() {
                    "" => 0,
                    other => locale::parse_major_input(other, &currency)?,
                };
                let purchase = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .record_purchase(
                        supplier.trim(),
                        &[(product.id.clone(), qty_milli, unit_cost)],
                        note.trim(),
                        &actor,
                    )?;
                st.reload_all()?;
                st.refresh_inventory_text()?;
                let qty_show = domain::format_qty_display(qty_milli, 0, &product.unit_label);
                Ok(format!(
                    "Received {qty_show} {} · {}",
                    product.name,
                    st.money(Money(purchase.total_paise))
                ))
            })();
            match result {
                Ok(msg) => {
                    self.ui.text_input(cx, ids!(inv_qty_input)).set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(inv_cost_input))
                        .set_text(cx, "");
                    self.set_status(cx, msg);
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(org_save)).clicked(actions) {
            let name = self.ui.text_input(cx, ids!(org_name_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Rename the business")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can rename the business".into());
                }
                st.store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .set_org_name(&name)?;
                st.refresh_branches_text()?;
                Ok("Business name saved".to_string())
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_export)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can export a branch day".into());
                }
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let branch = store.active_branch()?;
                let json = store.export_branch_snapshot_json()?;
                let dir = store::data_dir()?.join("backups");
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                let code = if branch.code.trim().is_empty() {
                    "BRANCH".to_string()
                } else {
                    branch.code.trim().to_uppercase()
                };
                let path = dir.join(format!(
                    "branch_snapshot_{code}_{}.json",
                    chrono::Local::now().format("%Y-%m-%d")
                ));
                std::fs::write(&path, &json).map_err(|e| e.to_string())?;
                let copied = print_share::copy_text(&json);
                Ok(format!(
                    "Snapshot: {}{}",
                    path.display(),
                    if copied { " (also copied)" } else { "" }
                ))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(branch_import)).clicked(actions) {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Import a branch day")?;
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can import a branch day".into());
                }
                let path = store::data_dir()?
                    .join("backups")
                    .join("import_branch_snapshot.json");
                if !path.exists() {
                    return Err("Missing backups/import_branch_snapshot.json".into());
                }
                let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let branch = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .import_branch_snapshot(&json)?;
                st.refresh_branches_text()?;
                Ok(format!("Imported day for {}", branch.label()))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self
            .ui
            .button(cx, ids!(branch_consolidated))
            .clicked(actions)
        {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can see the roll-up".into());
                }
                let report = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .consolidated_report()?;
                let mut lines = vec![
                    format!("{} — all outlets today", report.org_name),
                    format!(
                        "Sales {} · {}",
                        report.totals.sale_count,
                        st.money(report.totals.revenue())
                    ),
                    format!(
                        "Cash {} · UPI {} · Card {}",
                        st.money(report.totals.cash()),
                        st.money(report.totals.upi()),
                        st.money(report.totals.card())
                    ),
                    format!(
                        "Still owed {} · Expenses {}",
                        st.money(Money(report.outstanding_credit_paise)),
                        st.money(Money(report.expenses_paise))
                    ),
                    String::new(),
                    "Per branch:".to_string(),
                ];
                for (branch_name, totals) in &report.branches {
                    lines.push(format!(
                        "  {branch_name} · {} sales · {}",
                        totals.sale_count,
                        st.money(totals.revenue())
                    ));
                }
                if !report.stale_branches.is_empty() {
                    lines.push(format!(
                        "Last known only: {}",
                        report.stale_branches.join(", ")
                    ));
                }
                if !report.missing_branches.is_empty() {
                    lines.push(format!(
                        "No snapshot yet: {}",
                        report.missing_branches.join(", ")
                    ));
                }
                st.branches_text = lines.join("\n");
                st.status = "Roll-up ready".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            }
            self.sync_chrome(cx);
        }

        if self.ui.button(cx, ids!(lock_login_ok)).clicked(actions) {
            self.try_unlock_with_login(cx);
        }

        if self.ui.button(cx, ids!(unit_add)).clicked(actions) {
            let code = self.ui.text_input(cx, ids!(unit_code_input)).text();
            let name = self.ui.text_input(cx, ids!(unit_name_input)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add units")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_unit(&code, &name)?;
                st.status = format!("Unit {code} added");
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.ui
                        .text_input(cx, ids!(unit_code_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(unit_name_input))
                        .set_text(cx, "");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(conv_add)).clicked(actions) {
            let from = self.ui.text_input(cx, ids!(conv_from_input)).text();
            let to = self.ui.text_input(cx, ids!(conv_to_input)).text();
            let factor_text = self.ui.text_input(cx, ids!(conv_factor_input)).text();
            let result = (|| -> Result<(), String> {
                let factor_milli = parse_qty_to_milli(&factor_text)
                    .map_err(|_| "Factor must be a number > 0".to_string())?;
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add unit conversion")?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.add_unit_conversion(&from, &to, factor_milli)?;
                st.status = match store.resolve_unit(&from) {
                    Some((family, factor)) => {
                        let anchor = domain::unit_family_anchor(&family).unwrap_or("");
                        format!(
                            "Added — now 1 {from} = {} {anchor} ({family})",
                            precise::format_kg(factor)
                        )
                    }
                    None => format!(
                        "1 {from} → {to} added — '{from}' is not linked to a standard unit yet"
                    ),
                };
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.ui
                        .text_input(cx, ids!(conv_from_input))
                        .set_text(cx, "");
                    self.ui.text_input(cx, ids!(conv_to_input)).set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(conv_factor_input))
                        .set_text(cx, "");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(find_printers_btn)).clicked(actions) {
            print_share::find_printers_in_background();
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(printer_add)).clicked(actions) {
            let label = self.ui.text_input(cx, ids!(printer_label_input)).text();
            let target = self.ui.text_input(cx, ids!(printer_target_input)).text();
            {
                let mut st = app_state().lock().unwrap();
                st.printer_draft.add_printer(&label, &target);
            }
            self.ui.text_input(cx, ids!(printer_label_input)).set_text(cx, "");
            self.ui.text_input(cx, ids!(printer_target_input)).set_text(cx, "");
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(printer_remove)).clicked(actions) {
            {
                let mut st = app_state().lock().unwrap();
                if let Some(last) = st.printer_draft.printers.last().map(|p| p.id.clone()) {
                    st.printer_draft.remove_printer(&last);
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(route_add)).clicked(actions) {
            let pick = self.ui.text_input(cx, ids!(route_add_printer_input)).text();
            let pick_trim = pick.trim().to_string();
            {
                let mut st = app_state().lock().unwrap();
                let key = parse_courier_route_key(&st.printer_route_key);
                if let Some(p) = st
                    .printer_draft
                    .printers
                    .iter()
                    .find(|p| p.label.eq_ignore_ascii_case(&pick_trim) || p.id == pick_trim)
                    .map(|p| p.id.clone())
                {
                    let route = st.printer_draft.route_mut(key);
                    if !route.iter().any(|id| id == &p) {
                        route.push(p);
                    }
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(route_up)).clicked(actions)
            || self.ui.button(cx, ids!(route_down)).clicked(actions)
            || self.ui.button(cx, ids!(route_remove)).clicked(actions)
            || self.ui.button(cx, ids!(route_clear)).clicked(actions)
        {
            let up = self.ui.button(cx, ids!(route_up)).clicked(actions);
            let down = self.ui.button(cx, ids!(route_down)).clicked(actions);
            let remove = self.ui.button(cx, ids!(route_remove)).clicked(actions);
            let clear = self.ui.button(cx, ids!(route_clear)).clicked(actions);
            {
                let mut st = app_state().lock().unwrap();
                let key_str = st.printer_route_key.clone();
                let key = parse_courier_route_key(&key_str);
                let route = st.printer_draft.route_mut(key);
                if clear {
                    route.clear();
                } else if remove {
                    route.pop();
                } else if up && route.len() >= 2 {
                    let i = route.len() - 1;
                    route.swap(i - 1, i);
                } else if down && route.len() >= 2 {
                    route.swap(0, 1);
                }
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(printers_save)).clicked(actions) {
            let result = {
                let st = app_state().lock().unwrap();
                let printers = st.printer_draft.clone();
                match st.data_base.as_ref() {
                    Some(base) => print_share::save_device_printers(base, &printers)
                        .map(|()| st.t("printers_saved")),
                    None => Err("Storage not ready".to_string()),
                }
            };
            match result {
                Ok(msg) | Err(msg) => self.set_status(cx, msg),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(print_test_btn)).clicked(actions) {
            let (dir, msg, own, target_desc) = {
                let st = app_state().lock().unwrap();
                let mut own: String = st.venue_title.trim().to_string();
                let phrase = st.t("print_test_page");
                if phrase != "Print a test page" {
                    own = format!("{own}\n{phrase}");
                }
                let own = own.trim().lines().map(|l| format!("{l}\n")).collect::<String>();
                let targets = st.printer_draft.targets_for(print_share::RouteKey::Receipt);
                let desc = targets
                    .first()
                    .map(print_share::describe)
                    .or_else(|| {
                        st.printer_draft
                            .printers
                            .first()
                            .map(|p| print_share::describe(&print_share::parse_target(&p.target)))
                    })
                    .unwrap_or_else(|| "No printer".into());
                (st.data_base.clone(), st.t("test_page_sent"), own, desc)
            };
            let page = format!(
                "{app} - test page\n{own}{target_desc}\n{when}\n\nIf you can read this, printing works here.\n",
                app = store::APP_LABEL,
                when = chrono::Local::now().format("%d %b %Y %H:%M"),
            );
            match print_share::print_text(print_share::PrintKind::Receipt, &page, dir.as_deref()) {
                Ok(_) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(measure_save)).clicked(actions) {
            let dim = self.ui.text_input(cx, ids!(measure_dim_input)).text();
            let weight = self.ui.text_input(cx, ids!(measure_weight_input)).text();
            let divisor_text = self.ui.text_input(cx, ids!(measure_divisor_input)).text();
            let slab_text = self.ui.text_input(cx, ids!(measure_slab_input)).text();
            let parsed = (|| -> Result<domain::BookingMeasureSettings, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let mut settings = store.booking_measure_settings();
                if !dim.trim().is_empty() {
                    settings.dim_unit = dim.trim().to_ascii_lowercase();
                }
                if !weight.trim().is_empty() {
                    settings.weight_unit = weight.trim().to_ascii_lowercase();
                }
                if !divisor_text.trim().is_empty() {
                    settings.volumetric_divisor = booking::parse_positive(&divisor_text)
                        .filter(|d| *d > precise::ZERO)
                        .ok_or_else(|| "Volumetric divisor must be a number > 0".to_string())?;
                }
                if !slab_text.trim().is_empty() {
                    settings.slab_kg = booking::parse_positive(&slab_text)
                        .filter(|d| *d > precise::ZERO)
                        .ok_or_else(|| "Slab must be a number > 0 (in kg)".to_string())?;
                }
                Ok(settings)
            })();
            match parsed {
                Ok(settings) => {
                    self.apply_measure_settings(cx, settings);
                    self.ui
                        .text_input(cx, ids!(measure_dim_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(measure_weight_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(measure_divisor_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(measure_slab_input))
                        .set_text(cx, "");
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(venue_brand_save)).clicked(actions) {
            self.save_venue_brand(cx);
        }
        if self.ui.button(cx, ids!(print_stamp_pick)).clicked(actions) {
            let dialog = FileDialog::new()
                .set_title("Choose receipt stamp".into())
                .add_filter(
                    "Images".into(),
                    vec![
                        "webp".into(),
                        "png".into(),
                        "jpg".into(),
                        "jpeg".into(),
                    ],
                )
                .set_id(live_id!(print_stamp_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening stamp picker…");
        }
        if self.ui.button(cx, ids!(print_stamp_clear)).clicked(actions) {
            let outcome = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_receipt_stamp_key(None)?;
                Ok("Stamp cleared".into())
            })();
            match outcome {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(print_qr_pick)).clicked(actions) {
            let dialog = FileDialog::new()
                .set_title("Choose payment QR".into())
                .add_filter(
                    "Images".into(),
                    vec![
                        "webp".into(),
                        "png".into(),
                        "jpg".into(),
                        "jpeg".into(),
                    ],
                )
                .set_id(live_id!(print_qr_pick));
            cx.open_select_file_dialog(dialog);
            self.set_status(cx, "Opening payment QR picker…");
        }
        if self.ui.button(cx, ids!(print_qr_clear)).clicked(actions) {
            let outcome = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_payment_qr_key(None)?;
                Ok("Payment QR cleared".into())
            })();
            match outcome {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(awb_pool_add)).clicked(actions) {
            let carrier = self.ui.text_input(cx, ids!(awb_pool_carrier_input)).text();
            let start_t = self.ui.text_input(cx, ids!(awb_pool_start_input)).text();
            let end_t = self.ui.text_input(cx, ids!(awb_pool_end_input)).text();
            let prefix = self.ui.text_input(cx, ids!(awb_pool_prefix_input)).text();
            let label = self.ui.text_input(cx, ids!(awb_pool_label_input)).text();
            let outcome = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.require_server_role("Add AWB pool")?;
                let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                let party = store
                    .list_customers()?
                    .into_iter()
                    .find(|c| {
                        c.name.eq_ignore_ascii_case(carrier.trim()) || c.id == carrier.trim()
                    })
                    .ok_or_else(|| "Carrier party not found — add them on Parties first".to_string())?;
                let start_n: i64 = start_t
                    .trim()
                    .parse()
                    .map_err(|_| "Start no. must be a number".to_string())?;
                let end_n: i64 = end_t
                    .trim()
                    .parse()
                    .map_err(|_| "End no. must be a number".to_string())?;
                let pool = store.add_awb_pool(&party.id, &prefix, start_n, end_n, &label)?;
                Ok(format!(
                    "AWB pool {}{}–{} for {}",
                    pool.prefix, pool.start_n, pool.end_n, party.name
                ))
            })();
            match outcome {
                Ok(m) => self.set_status(cx, m),
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        for btn in [ids!(reports_bill_chase), ids!(credit_bill_chase)] {
            if self.ui.button(cx, btn).clicked(actions) {
                let text = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.pending_bills_chase_text()
                })();
                match text {
                    Ok(t) => {
                        let rows: Vec<ReportRow> = t
                            .lines()
                            .map(|l| ReportRow::Line(l.to_string()))
                            .collect();
                        self.set_report_rows(cx, ids!(reports_label), rows);
                        self.copy_text_to_clipboard(cx, &t);
                        self.set_status(cx, "Chase list shown · also copied");
                        let mut st = app_state().lock().unwrap();
                        st.nav_back.clear();
                        st.nav_forward.clear();
                        st.screen = Screen::Reports;
                        st.reports_nav = ReportsNav::Bank;
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(reports_month_mix)).clicked(actions) {
            let ym = chrono::Local::now().format("%Y-%m").to_string();
            let text = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.month_mix_report_text(&ym)
            })();
            match text {
                Ok(t) => {
                    let rows: Vec<ReportRow> = t
                        .lines()
                        .map(|l| ReportRow::Line(l.to_string()))
                        .collect();
                    self.set_report_rows(cx, ids!(reports_label), rows);
                    self.copy_text_to_clipboard(cx, &t);
                    self.set_status(cx, format!("Month mix {ym} · copied"));
                }
                Err(e) => self.set_status(cx, e),
            }
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(csv_copy_gst_month)).clicked(actions) {
            let ym = chrono::Local::now().format("%Y-%m").to_string();
            let text = (|| -> Result<String, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.month_gst_summary_csv(&ym)
            })();
            match text {
                Ok(t) => {
                    self.copy_text_to_clipboard(cx, &t);
                    self.set_status(cx, format!("Month GST CSV {ym} copied — for your CA"));
                }
                Err(e) => self.set_status(cx, e),
            }
        }
        if self.ui.button(cx, ids!(venue_icon_pick)).clicked(actions) {
            self.pick_venue_icon(cx);
        }
        if self.ui.button(cx, ids!(venue_icon_clear)).clicked(actions) {
            self.clear_venue_icon(cx);
        }
        if self.ui.button(cx, ids!(onboard_icon_pick)).clicked(actions) {
            self.pick_onboard_icon(cx);
        }

        self.handle_diag_actions(cx, actions);
        if self.ui.button(cx, ids!(backup_btn)).clicked(actions) {
            let result = (|| -> Result<std::path::PathBuf, String> {
                let st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let path = default_backup_path()?;
                store.backup_to(&path)?;
                Ok(path)
            })();
            match result {
                Ok(path) => {
                    self.ui.widget(cx, ids!(backup_path_label)).set_text(
                        cx,
                        &plain_display_text(&format!("Backup: {}", path.display())),
                    );
                    self.set_status(cx, "Backup created");
                }
                Err(e) => self.set_status(cx, e),
            }
        }

        if self.ui.button(cx, ids!(restore_btn)).clicked(actions) {
            // Pick the backup; the automatic ones are where it opens.
            let start = app_state()
                .lock()
                .unwrap()
                .store
                .as_ref()
                .map(|s| s.auto_backup_dir());
            let title = app_state().lock().unwrap().t("restore_backup");
            let mut dialog = FileDialog::new()
                .set_title(title)
                .add_filter("Backups".into(), vec!["sqlite3".into()])
                .set_id(live_id!(restore_pick));
            if let Some(dir) = start {
                dialog = dialog.set_location(dir);
            }
            cx.open_select_file_dialog(dialog);
        }
    }
}

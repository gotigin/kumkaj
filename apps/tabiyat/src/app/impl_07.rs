impl App {
fn confirm_line_action(&mut self, cx: &mut Cx) {
        let reason = self.ui.text_input(cx, ids!(line_reason_input)).text();
        let qty_text = self.ui.text_input(cx, ids!(line_qty_input)).text();
        let target = self.ui.text_input(cx, ids!(line_target_input)).text();
        let pin = self.ui.text_input(cx, ids!(line_pin_input)).text();
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let (line_id, action) = st
                .line_action
                .clone()
                .ok_or_else(|| "Pick an item first".to_string())?;
            let auto_print = st
                .store
                .as_ref()
                .map(|s| s.kot_auto_print())
                .unwrap_or(true);
            let (request, done, name_hint) = match action {
                LineAction::CancelBatch => {
                    let ids: Vec<String> = st
                        .cart
                        .lines
                        .iter()
                        .filter(|l| {
                            l.cancel_pick
                                && l.is_sent()
                                && !l.running_line_id.is_empty()
                                && l.running_line_id != "counter"
                                && !l.running_line_id.starts_with(QUEUED_LINE)
                        })
                        .map(|l| l.running_line_id.clone())
                        .collect();
                    if ids.is_empty() {
                        return Err(st.t("batch_cancel_none"));
                    }
                    let n = ids.len();
                    let done = if auto_print {
                        st.t("batch_cancel_ok")
                            .replace("{n}", &n.to_string())
                            .replace("{slips}", "…")
                            .replace("{kitchens}", &st.t("kitchen"))
                    } else {
                        st.t("batch_cancel_ok_noprint")
                            .replace("{n}", &n.to_string())
                    };
                    (
                        net::RunningAction::CancelBatch {
                            line_ids: ids,
                            reason: reason.trim().to_string(),
                            prepared: st.line_cooked,
                        },
                        done,
                        format!("{n} dishes"),
                    )
                }
                _ => {
                    let line = st
                        .cart
                        .lines
                        .iter()
                        .find(|l| l.running_line_id == line_id)
                        .cloned()
                        .ok_or_else(|| "That item is no longer on the order".to_string())?;
                    let request = match action {
                        LineAction::Cancel => {
                            let qty_milli = match qty_text.trim() {
                                "" => None,
                                typed => Some(parse_qty_to_milli(typed)?),
                            };
                            net::RunningAction::Cancel {
                                line_id: line_id.clone(),
                                qty_milli,
                                reason: reason.trim().to_string(),
                                prepared: st.line_cooked,
                            }
                        }
                        LineAction::CancelBatch => unreachable!(),
                        LineAction::Complimentary => net::RunningAction::Complimentary {
                            line_id: line_id.clone(),
                            on: !line.complimentary,
                            reason: reason.trim().to_string(),
                        },
                        LineAction::Move => {
                            let to = st
                                .tables
                                .iter()
                                .find(|t| t.name.trim().eq_ignore_ascii_case(target.trim()))
                                .map(|t| t.id.clone())
                                .ok_or_else(|| {
                                    format!("No table named \"{}\"", target.trim())
                                })?;
                            net::RunningAction::MoveLines {
                                line_ids: vec![line_id.clone()],
                                to_table_id: to,
                            }
                        }
                    };
                    let done = match action {
                        LineAction::Cancel if !auto_print => format!(
                            "{} cancelled — on the Kitchen screen; KOT auto-print is off, so no cancel slip",
                            line.name
                        ),
                        LineAction::Cancel => {
                            format!("{} cancelled — the kitchen gets a cancel slip", line.name)
                        }
                        LineAction::CancelBatch => unreachable!(),
                        LineAction::Complimentary if line.complimentary => {
                            format!("{} charged again", line.name)
                        }
                        LineAction::Complimentary => {
                            format!("{} is complimentary (NC)", line.name)
                        }
                        LineAction::Move => {
                            format!("{} moved to {}", line.name, target.trim())
                        }
                    };
                    (request, done, line.name.clone())
                }
            };
            let acted = st.table_action(request, &pin, &done, phone::Follow::Refresh)?;
            if matches!(action, LineAction::CancelBatch) {
                for line in &mut st.cart.lines {
                    line.cancel_pick = false;
                }
            }
            st.line_action = None;
            Ok(match (action, acted) {
                (LineAction::CancelBatch, Acted::Here(Some(raw)))
                    if raw.starts_with("batch_noprint:") =>
                {
                    let n = raw.trim_start_matches("batch_noprint:");
                    st.t("batch_cancel_ok_noprint").replace("{n}", n)
                }
                (LineAction::CancelBatch, Acted::Here(Some(raw)))
                    if raw.starts_with("batch_ok:") =>
                {
                    let rest = raw.trim_start_matches("batch_ok:");
                    let (main, err) = rest.split_once('|').unwrap_or((rest, ""));
                    let mut parts = main.splitn(3, ':');
                    let n = parts.next().unwrap_or("0");
                    let slips = parts.next().unwrap_or("0");
                    let kitchens = parts.next().unwrap_or("");
                    let mut msg = st
                        .t("batch_cancel_ok")
                        .replace("{n}", n)
                        .replace("{slips}", slips)
                        .replace("{kitchens}", kitchens);
                    if !err.is_empty() {
                        msg.push_str(" · ");
                        msg.push_str(err);
                    }
                    msg
                }
                (_, Acted::Here(_)) => done,
                (_, Acted::Sent) => format!("{name_hint} — sending…"),
            })
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// One cancel-kind slip per kitchen listing **every** cancelled dish on
    /// the open order. Keeps per-item cancel print; this is the batch path.
    fn print_all_cancelled_kots(&mut self, cx: &mut Cx) -> Result<String, String> {
        let _ = cx;
        let st = app_state().lock().unwrap();
        let Some(session) = st.session.as_ref() else {
            return Err("Unlock first".into());
        };
        let table_id = st
            .selected_table_id
            .clone()
            .ok_or_else(|| st.t("print_cancelled_kot_needed"))?;
        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
        let order = store
            .running_order(&table_id)?
            .ok_or_else(|| st.t("print_cancelled_kot_needed"))?;
        let cancelled = store.cancelled_kot_lines(&order.id)?;
        if cancelled.is_empty() {
            return Err(st.t("print_cancelled_kot_none"));
        }
        let feats = session.effective_features();
        for (sid, _, _, _) in &cancelled {
            if !sid.is_empty() && !feats.allows_kot_station(sid) {
                let name = st
                    .stations
                    .iter()
                    .find(|s| s.id == *sid)
                    .map(|s| s.name.as_str())
                    .unwrap_or(sid.as_str());
                return Err(st
                    .t("kot_station_not_allowed")
                    .replace("{station}", name));
            }
        }
        let order_id = order.id.clone();
        let place = st
            .tables
            .iter()
            .find(|t| t.id == table_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "table".into());
        let dish_n = cancelled.len();
        let on_client = st.device_role == DeviceRole::Client
            || st
                .store
                .as_ref()
                .map(|s| s.is_client_cache_path())
                .unwrap_or(false);
        if on_client {
            let _job = st.lan_job(
                "/kot_cancelled_print",
                &serde_json::json!({ "order_id": order_id }),
                format!("Print cancelled for {place}"),
                st.t("print_cancelled_kot_client")
                    .replace("{n}", &dish_n.to_string())
                    .replace("{place}", &place),
                phone::Follow::Refresh,
                true,
            )?;
            return Ok(st
                .t("print_cancelled_kot_client")
                .replace("{n}", &dish_n.to_string())
                .replace("{place}", &place));
        }
        let actor = st.actor()?;
        let (dish_n, ok, kitchen_names, errors) =
            net::print_cancelled_kots(store, &actor, &order_id)?;
        let kitchens = if kitchen_names.is_empty() {
            st.t("kitchen")
        } else {
            kitchen_names.join(" · ")
        };
        let mut msg = st
            .t("print_cancelled_kot_ok")
            .replace("{n}", &dish_n.to_string())
            .replace("{slips}", &ok.to_string())
            .replace("{kitchens}", &kitchens);
        if !errors.is_empty() {
            msg.push_str(" · ");
            msg.push_str(&errors.join("; "));
        }
        Ok(msg)
    }

    /// Recovery print: kitchen slips for **included** sent lines only
    /// (include/exclude on the ticket). Not a new Send — paper only.
    /// Person `kot_stations` gates which kitchens may reprint; Client/Server
    /// only changes *where* the printers run (LAN → Server kitchen printers).
    /// `Right::Reprint` is bills only — not used here.
    fn reprint_open_kots(&mut self, cx: &mut Cx) -> Result<String, String> {
        let _ = cx;
        let st = app_state().lock().unwrap();
        let Some(session) = st.session.as_ref() else {
            return Err("Unlock first".into());
        };
        let feats = session.effective_features();
        let has_sent = st.cart.lines.iter().any(|l| {
            l.is_sent() && !l.kot_id.is_empty() && l.kot_id != QUEUED_KOT
        });
        if !has_sent {
            return Err(st.t("reprint_kot_needed"));
        }
        let selected: Vec<&domain::CartLine> = st
            .cart
            .lines
            .iter()
            .filter(|l| {
                l.is_sent()
                    && !l.reprint_skip
                    && !l.kot_id.is_empty()
                    && l.kot_id != QUEUED_KOT
            })
            .collect();
        if selected.is_empty() {
            return Err(st.t("reprint_kot_none_selected"));
        }
        for line in &selected {
            let sid = line.kitchen_station_id(&st.products);
            if !sid.is_empty() && !feats.allows_kot_station(sid) {
                let name = st
                    .stations
                    .iter()
                    .find(|s| s.id == sid)
                    .map(|s| s.name.as_str())
                    .unwrap_or(sid);
                return Err(st
                    .t("kot_station_not_allowed")
                    .replace("{station}", name));
            }
        }
        let table_id = st.selected_table_id.clone();
        let table_name = table_id.as_ref().and_then(|id| {
            st.tables
                .iter()
                .find(|t| &t.id == id)
                .map(|t| t.name.clone())
        });
        let lines: Vec<net::RemoteKotReprintLine> = selected
            .iter()
            .map(|l| net::RemoteKotReprintLine {
                kot_id: l.kot_id.clone(),
                line_id: l.running_line_id.clone(),
                name: l.name.clone(),
                qty: l.qty,
                qty_milli: l.qty_milli,
                unit_label: l.unit_label.clone(),
                note: l.note.clone(),
                applied_modifiers: l.applied_modifiers.clone(),
                station_id: l.kitchen_station_id(&st.products).to_string(),
            })
            .collect();
        let req = net::RemoteKotReprintRequest {
            table_name: table_name.clone(),
            table_id: table_id.clone(),
            lines,
        };
        // Client cache / Client role: same person rights; printers are on the
        // Server — ask over LAN. Never refuse capable people, never flip role.
        let on_client = st.device_role == DeviceRole::Client
            || st
                .store
                .as_ref()
                .map(|s| s.is_client_cache_path())
                .unwrap_or(false);
        if on_client {
            let place = table_name
                .clone()
                .unwrap_or_else(|| "counter".into());
            let dish_n = req.lines.len();
            let _job = st.lan_job(
                "/kot_reprint",
                &req,
                format!("Print again for {place}"),
                st.t("reprint_kot_client")
                    .replace("{n}", &dish_n.to_string())
                    .replace("{place}", &place),
                phone::Follow::Refresh,
                true,
            )?;
            return Ok(st
                .t("reprint_kot_client")
                .replace("{n}", &dish_n.to_string())
                .replace("{place}", &place));
        }
        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
        let actor = st.actor()?;
        let (dish_n, ok, kitchen_names, errors) = net::reprint_selected_kots(store, &actor, &req)?;
        let kitchens = if kitchen_names.is_empty() {
            st.t("kitchen")
        } else {
            kitchen_names.join(" · ")
        };
        let mut msg = st
            .t("reprint_kot_ok")
            .replace("{n}", &dish_n.to_string())
            .replace("{slips}", &ok.to_string())
            .replace("{kitchens}", &kitchens);
        if !errors.is_empty() {
            msg.push_str(" · ");
            msg.push_str(&errors.join("; "));
        }
        Ok(msg)
    }

    /// Print the open table's bill (table turns 🟢 on every device).
    fn print_table_bill(&mut self, cx: &mut Cx) {
        self.commit_open_cart_line_fields(cx);
        let pin = self.ui.text_input(cx, ids!(move_pin_input)).text();
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let table_id = st
                .selected_table_id
                .clone()
                .ok_or_else(|| "Open a table first".to_string())?;
            if st.cart.lines.iter().any(|l| !l.is_sent()) {
                return Err("Send the new medicines first, then print the bill".into());
            }
            let done = "Bill printed — take payment when the guests pay";
            let acted = st.table_action(
                net::RunningAction::PrintBill { table_id },
                &pin,
                done,
                phone::Follow::BillText,
            )?;
            // The counter's printer prints it; a tablet's copy printed there.
            let Acted::Here(text) = acted else {
                return Ok("Printing the bill at the counter…".to_string());
            };
            let text = text.unwrap_or_default();
            let dir = st.store.as_ref().and_then(|s| s.db_path().parent().map(|p| p.to_path_buf()));
            // Books already say Printed. Paper is best-effort: a printer error
            // must never fail this action or clear the cart / table.
            st.last_receipt = text.clone();
            let copies = st
                .store
                .as_ref()
                .and_then(|s| s.bill_slip_format().ok())
                .map(|f| f.copies.clamp(1, 3))
                .unwrap_or(1);
            match print_share::print_route_copies(
                print_share::PrintKind::Bill,
                print_share::RouteKey::Bill,
                &text,
                dir.as_deref(),
                copies,
            ) {
                Ok(share) if share.opened => Ok(done.to_string()),
                // The table is on Bill either way; say the paper truth, and
                // that printing again counts as a reprint.
                Ok(_) => Ok(format!("{} · {}", net::NO_BILL_PRINTER, BILL_ON_TABLE_NO_PAPER)),
                Err(e) => Ok(format!("Bill not printed ({e}) · {BILL_ON_TABLE_NO_PAPER}")),
            }
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Move the open table's whole order to a free table, or merge it into a
    /// running one.
    fn move_or_merge_table(&mut self, cx: &mut Cx, merge: bool) {
        let target = self.ui.text_input(cx, ids!(move_target_input)).text();
        let pin = self.ui.text_input(cx, ids!(move_pin_input)).text();
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let from = st
                .selected_table_id
                .clone()
                .ok_or_else(|| "Open a table first".to_string())?;
            let to = st
                .tables
                .iter()
                .find(|t| t.name.trim().eq_ignore_ascii_case(target.trim()))
                .map(|t| t.id.clone())
                .ok_or_else(|| format!("No table named \"{}\"", target.trim()))?;
            let action = if merge {
                net::RunningAction::Merge { from_table_id: from.clone(), into_table_id: to.clone() }
            } else {
                net::RunningAction::MoveTable { from_table_id: from.clone(), to_table_id: to.clone() }
            };
            let done = if merge {
                format!("Merged into {}", target.trim())
            } else {
                format!("Moved to {}", target.trim())
            };
            if st.device_role == DeviceRole::Client {
                // The order follows to the target once the Server agrees.
                st.table_action(action, &pin, &done, phone::Follow::OpenTable(to))?;
                st.move_panel_open = false;
                return Ok(format!("{} — sending…", target.trim()));
            }
            // The order now lives on the target table: follow it.
            st.selected_table_id = Some(to);
            if let Err(e) = st.table_action(action, &pin, &done, phone::Follow::Refresh) {
                st.selected_table_id = Some(from);
                return Err(e);
            }
            st.move_panel_open = false;
            Ok(done)
        })();
        match result {
            Ok(msg) => {
                self.ui.text_input(cx, ids!(move_target_input)).set_text(cx, "");
                self.ui.text_input(cx, ids!(move_pin_input)).set_text(cx, "");
                self.set_status(cx, msg);
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn handle_modifier_chips(
        &mut self,
        cx: &mut Cx,
        actions: &Actions,
        item: &WidgetRef,
        index: usize,
        amend: bool,
    ) {
        let slot = if item.button(cx, ids!(mod_0)).clicked(actions)
            || item.button(cx, ids!(mod_0_on)).clicked(actions)
        {
            Some(0)
        } else if item.button(cx, ids!(mod_1)).clicked(actions)
            || item.button(cx, ids!(mod_1_on)).clicked(actions)
        {
            Some(1)
        } else if item.button(cx, ids!(mod_2)).clicked(actions)
            || item.button(cx, ids!(mod_2_on)).clicked(actions)
        {
            Some(2)
        } else {
            None
        };
        let page_delta = if item.button(cx, ids!(mod_next)).clicked(actions) {
            Some(1)
        } else if item.button(cx, ids!(mod_prev)).clicked(actions) {
            Some(-1)
        } else {
            None
        };
        if slot.is_none() && page_delta.is_none() {
            return;
        }
        // Chip tap can fire while Unit @ / Applied still holds focus — commit
        // typed values before the modifier reprice + sync_chrome.
        if amend {
            self.commit_amend_row_fields(cx, index, item);
        } else {
            self.commit_cart_row_fields(cx, index, item);
        }
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let product_id = {
                let cart = if amend { &st.amend_cart } else { &st.cart };
                cart.lines
                    .get(index)
                    .map(|l| l.product_id.clone())
                    .ok_or_else(|| "Missing cart line".to_string())?
            };
            let (mods, groups) = st
                .products
                .iter()
                .find(|p| p.id == product_id)
                .map(|p| (p.modifiers(), p.modifier_groups()))
                .unwrap_or_default();
            let cart = if amend {
                &mut st.amend_cart
            } else {
                &mut st.cart
            };
            if let Some(delta) = page_delta {
                cart.shift_modifier_page(index, delta, mods.len());
                return Ok(String::new());
            }
            let page = cart
                .lines
                .get(index)
                .map(|l| l.modifier_page)
                .unwrap_or(0);
            let pages = mods.len().div_ceil(domain::MODIFIER_PAGE).max(1);
            let start = (page % pages) * domain::MODIFIER_PAGE;
            let modifier = mods
                .get(start + slot.unwrap_or(0))
                .cloned()
                .ok_or_else(|| "No modifier there".to_string())?;
            let on = cart.toggle_modifier_in_groups(index, &modifier, &groups)?;
            let price = st.money(modifier.price());
            let label = if modifier.group.is_empty() || modifier.group == "Options" {
                modifier.name.clone()
            } else {
                format!("{} · {}", modifier.group, modifier.name)
            };
            Ok(if on {
                format!("{label} · {price}")
            } else {
                format!("{label} removed")
            })
        })();
        match result {
            Ok(status) if !status.is_empty() => self.set_status(cx, status),
            Ok(_) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Cart `−`/`+`. Raising a tracked line stops at the on-hand count so the
    /// refusal lands on the button, not on the customer at the till.
    /// Choose / Change table: Floor in pick mode, then return to Order.
    /// - No table yet: seat the desk's unsent punches onto the tapped table.
    /// - Already on a table (Change table): open that table's ticket only —
    ///   previous table's lines stay put (use Move / merge to relocate).
    /// Floor from the nav bar only browses/opens tickets (same open path).
    fn pick_table_for_order(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.cart.set_order_type(OrderType::DineIn);
            st.picking_from_table_id = st.selected_table_id.clone();
            st.selected_table_id = None;
            st.open_cart_line = None;
            // First seat: keep punched lines so they attach on tap.
            // Change table: drop the old ticket from the desk — the pick
            // reloads the chosen table from the Server.
            if st.picking_from_table_id.is_some() {
                st.cart.clear();
            }
        }
        self.go(cx, Screen::Floor);
        app_state().lock().unwrap().picking_table = true;
        self.sync_chrome(cx);
    }

    /// Leave pick-table mode without choosing: restore the previous table's
    /// Server ticket, or keep the unsent cart if none was attached yet.
    fn cancel_table_pick(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.picking_table = false;
            if let Some(from) = st.picking_from_table_id.take() {
                let _ = st.load_table_cart(&from);
            }
        }
        // Pop Floor → Pos (pick was a drill-down); do not push Pos again.
        if !self.navigate_back(cx) {
            self.go_peer(cx, Screen::Pos);
        }
    }

    /// "Entered ₹300 of ₹500 · ₹200 left" under the expense's ways.
    fn sync_parts_status(&mut self, cx: &mut Cx) {
        let total_text = self.ui.text_input(cx, ids!(buy_expense_amount)).text();
        let text = {
            let st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let total = locale::parse_major_input(total_text.trim(), &currency).unwrap_or(0);
            let entered: i64 = st
                .pay_parts
                .iter()
                .filter_map(|p| locale::parse_major_input(p.amount.trim(), &currency).ok())
                .sum();
            if total <= 0 || st.pay_parts.len() < 2 {
                String::new()
            } else {
                st.t("parts_status")
                    .replace("{a}", &st.money(Money(entered)))
                    .replace("{b}", &st.money(Money(total)))
                    .replace("{c}", &st.money(Money((total - entered).max(0))))
            }
        };
        self.ui.label(cx, ids!(buy_parts_status)).set_text(cx, &text);
        self.ui.button(cx, ids!(buy_parts_add)).set_text(cx, &app_state().lock().unwrap().t("add_another_way"));
    }

    /// Open one ticket line's details, or close them when it is already open.
    fn toggle_cart_line(&mut self, cx: &mut Cx, index: usize) {
        let note = {
            let mut st = app_state().lock().unwrap();
            if st.split_mode && st.screen == Screen::Pos {
                // Splitting: the tap picks the line for the separate bill.
                let id = st.cart.lines.get(index).map(|l| l.running_line_id.clone()).unwrap_or_default();
                if id.is_empty() {
                    Some(st.t("split_send_first"))
                } else {
                    match st.split_pick.iter().position(|p| *p == id) {
                        Some(i) => {
                            st.split_pick.remove(i);
                        }
                        None => st.split_pick.push(id),
                    }
                    None
                }
            } else {
                st.open_cart_line = if st.open_cart_line == Some(index) { None } else { Some(index) };
                None
            }
        };
        if let Some(n) = note {
            self.set_status(cx, n);
        }
        self.sync_chrome(cx);
    }

    /// Make the separate bill from the picked lines and open it.
    fn make_split_bill(&mut self, cx: &mut Cx) {
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let picked = st.split_pick.clone();
            if picked.is_empty() {
                return Err(st.t("split_pick_some"));
            }
            let done = st.t("split_done");
            st.table_action(net::RunningAction::Split { line_ids: picked }, "", &done, phone::Follow::Refresh)?;
            st.split_mode = false;
            st.split_pick.clear();
            Ok(done)
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn bump_cart_line(&mut self, cx: &mut Cx, index: usize, delta: i64) {
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            match st.cart.lines.get(index).cloned() {
                None => None,
                Some(line) if line.is_sent() => None,
                Some(line) => {
                    let blocked = if delta > 0 {
                        st.products
                            .iter()
                            .find(|p| p.id == line.product_id)
                            .filter(|p| p.tracks_stock())
                            .filter(|p| {
                                let step = line.bump_step_milli();
                                st.cart.qty_milli_of(&line.product_id).saturating_add(step)
                                    > p.on_hand_milli()
                            })
                            .map(|p| format!("Only {} of {} in stock", p.format_on_hand(), p.name))
                    } else {
                        None
                    };
                    if let Some(e) = blocked {
                        Some(e)
                    } else {
                        let step = line.bump_step_milli().saturating_mul(delta);
                        if st.selected_table_id.is_some() {
                            match st.bump_table_pending(&line.product_id, step, "Order updated") {
                                Err(e) => Some(e),
                                Ok(Acted::Sent) => {
                                    st.cart.bump_qty(index, delta);
                                    None
                                }
                                Ok(Acted::Here(_)) => None,
                            }
                        } else {
                            st.cart.bump_qty(index, delta);
                            None
                        }
                    }
                }
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Claim the charge slot. False means a charge is already running, and the
    /// caller must return without touching the cart.
    fn begin_pay(&mut self, cx: &mut Cx) -> bool {
        {
            let mut st = app_state().lock().unwrap();
            if st.pay_in_flight {
                st.status = "Charging — one moment".into();
            } else {
                st.pay_in_flight = true;
                return true;
            }
        }
        self.sync_chrome(cx);
        false
    }

    fn end_pay(&mut self) {
        app_state().lock().unwrap().pay_in_flight = false;
    }

    fn show_undo_snackbar(&mut self, cx: &mut Cx, sale_id: String, text: String) {
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
        }
        let secs = SnackKind::clamp_secs(st.snack_secs);
        st.undo_sale_id = Some(sale_id);
        st.undo_snack_text = text;
        st.undo_timer = cx.start_timeout(secs);
    }

    fn clear_undo_snackbar(&mut self, cx: &mut Cx) {
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
            st.undo_timer = Timer::empty();
        }
        st.undo_sale_id = None;
        st.undo_snack_text.clear();
    }

    fn undo_last_sale(&mut self, cx: &mut Cx) {
        let result = (|| -> Result<(), String> {
            let (is_client, sale_id, actor) = {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .undo_sale_id
                    .clone()
                    .ok_or_else(|| "Nothing to undo".to_string())?;
                (st.device_role == DeviceRole::Client, sale_id, actor)
            };
            if is_client {
                let mut st = app_state().lock().unwrap();
                st.lan_job(
                    "/undo",
                    &net::RemoteUndoRequest { sale_id: sale_id.clone() },
                    "Undo".into(),
                    "Sale undone on the shop's books".into(),
                    phone::Follow::Refresh,
                    false,
                )?;
                st.status = "Undoing…".into();
                st.undo_sale_id = None;
                st.undo_snack_text.clear();
                return Ok(());
            }
            let mut st = app_state().lock().unwrap();
            let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
            let sale = store.quick_undo_sale(&sale_id, &actor)?;
            st.cart = Cart::from_sale(&sale);
            st.open_cart_line = None;
            st.selected_table_id = sale.table_id.clone();
            st.selected_customer_id = sale.customer_id.clone();
            st.last_receipt = format!("Undone {}", &sale.id[..8.min(sale.id.len())]);
            st.status = "Sale undone — cart restored".into();
            st.undo_sale_id = None;
            st.undo_snack_text.clear();
            st.reload_all()?;
            Ok(())
        })();
        {
            let mut st = app_state().lock().unwrap();
            if !st.undo_timer.is_empty() {
                cx.stop_timer(st.undo_timer);
                st.undo_timer = Timer::empty();
            }
        }
        if let Err(e) = result {
            self.set_status(cx, e);
            self.clear_undo_snackbar(cx);
        } else {
            self.go_peer(cx, Screen::Pos);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_total_from_input(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, true);
    }

    /// Live Rounded edits must move totals immediately — waiting for blur
    /// leaves the field on the previous total and confuses the cashier.
    fn apply_cart_total_from_input_live(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, false);
    }

    fn apply_cart_total_from_input_inner(&mut self, cx: &mut Cx, report_errors: bool) {
        let total_field = self.ui.text_input(cx, ids!(cart_total_input));
        let text = total_field.text();
        let field_focused = cx.has_key_focus(total_field.area());
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.cart.is_empty() {
                return Err("Cart is empty".into());
            }
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(&text, &currency)?;
            if minor < 0 {
                return Err("Rounded Total cannot be negative".into());
            }
            if minor == 0 {
                let allow = st
                    .store
                    .as_ref()
                    .and_then(|s| s.allow_zero_sales().ok())
                    .unwrap_or(true);
                if !allow {
                    return Err("Zero-value sales are turned off in Settings".into());
                }
            }
            let applied = st.cart.applied_total().paise();
            // Live `changed` can fire with a stale field after ADD. Ignore
            // mismatches when there is no explicit bill round and the seller
            // is not actively editing this field.
            if !report_errors
                && st.cart.charge_total.is_none()
                && minor != applied
                && !field_focused
            {
                return Ok(());
            }
            st.cart.set_charge_total(Money(minor))?;
            st.status = if st.cart.rounding().paise() == 0
                && st.cart.applied_total().paise() == st.cart.actual_total().paise()
            {
                "Rounded Total equals Applied Total".into()
            } else {
                format!(
                    "Actual {} · Applied {} · Rounded {} · round {}",
                    st.money(st.cart.actual_total()),
                    st.money(st.cart.applied_total()),
                    st.money(st.cart.rounded_total()),
                    st.money(st.cart.rounding())
                )
            };
            Ok(())
        })();
        if let Err(e) = result
            && report_errors {
                self.set_status(cx, e);
            }
        self.sync_chrome(cx);
    }

    fn apply_cart_unit_price(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_unit_price(index, Money(minor))?;
            st.persist_cart_line_pricing(index)?;
            st.status = format!(
                "Unit price set · line {}",
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.line_total())
                        .unwrap_or_default()
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_line_total(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_line_total(index, Money(minor))?;
            st.persist_cart_line_pricing(index)?;
            st.status = format!(
                "Line Applied {} · Actual {} · @ unchanged {}",
                st.money(Money(minor)),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.actual_total())
                        .unwrap_or(Money(0))
                ),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.unit_price)
                        .unwrap_or(Money(0))
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_note(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.cart.set_note(index, text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Pull Unit @ / Applied / note from a ticket row into the cart before any
    /// action that sync-paints (close, Cancel/NC/Move, ±, modifiers, pay, Send
    /// KOT). Quiet — no status spam; caller syncs when needed.
    fn commit_cart_row_fields(&mut self, cx: &mut Cx, index: usize, item: &WidgetRef) {
        let unit_text = item.text_input(cx, ids!(unit_price)).text();
        let total_text = item.text_input(cx, ids!(line_total)).text();
        let note_text = item.text_input(cx, ids!(note)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if index >= st.cart.lines.len() {
                return Ok(());
            }
            let currency = st.locale.currency.clone();
            let unit_minor = locale::parse_major_input(&unit_text, &currency)?;
            st.cart.set_unit_price(index, Money(unit_minor))?;
            let total_minor = locale::parse_major_input(&total_text, &currency)?;
            st.cart.set_line_total(index, Money(total_minor))?;
            st.cart.set_note(index, note_text)?;
            st.persist_cart_line_pricing(index)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
    }

    fn commit_amend_row_fields(&mut self, cx: &mut Cx, index: usize, item: &WidgetRef) {
        let unit_text = item.text_input(cx, ids!(unit_price)).text();
        let total_text = item.text_input(cx, ids!(line_total)).text();
        let note_text = item.text_input(cx, ids!(note)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if index >= st.amend_cart.lines.len() {
                return Ok(());
            }
            let currency = st.locale.currency.clone();
            let unit_minor = locale::parse_major_input(&unit_text, &currency)?;
            st.amend_cart.set_unit_price(index, Money(unit_minor))?;
            let total_minor = locale::parse_major_input(&total_text, &currency)?;
            st.amend_cart.set_line_total(index, Money(total_minor))?;
            st.amend_cart.set_note(index, note_text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
    }

    /// Commit the open Order line from its widgets — for Take payment / Send
    /// KOT before the cart portal loop has run this frame.
    fn commit_open_cart_line_fields(&mut self, cx: &mut Cx) {
        let Some(index) = app_state().lock().unwrap().open_cart_line else {
            return;
        };
        let host = self.ui.widget(cx, ids!(cart_list));
        let list = host.portal_list(cx, ids!(list));
        let item = list.item(cx, index, live_id!(Item));
        self.commit_cart_row_fields(cx, index, &item);
    }

    fn clear_menu_form(&mut self, cx: &mut Cx) {
        {
            let mut st = app_state().lock().unwrap();
            st.dish_picker_open = false;
            st.category_picker_open = false;
            st.form_category_photo = None;
            st.form_category_auto = false;
        }
        for path in [
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_barcode_input),
            ids!(menu_hsn_input),
            ids!(menu_composition_input),
            ids!(menu_rx_note_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_modifiers_input),
            ids!(menu_variations_input),
            ids!(menu_group_input),
            ids!(menu_area_prices_input),
            ids!(menu_schedule_input),
            ids!(menu_recipe_input),
            ids!(menu_station_input),
        ] {
            self.ui.text_input(cx, path).set_text(cx, "");
        }
        let mut st = app_state().lock().unwrap();
        st.editing_product_id = None;
        // A new medicine is bought to sell and counted. "Don't count stock" is the exception.
        st.form_resale = true;
        st.draft_image_key = None;
        st.form_category_id = None;
        st.form_category_label.clear();
        st.form_schedule_h = false;
        st.form_rx_required = false;
    }

    /// Leave the item currently in the form and start a blank one.
    fn start_new_menu_item(&mut self, cx: &mut Cx) {
        self.clear_menu_form(cx);
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    /// Tap loads that item into the form. A different item replaces it.
    /// The same item stays as it is, so a second tap does not wipe unsaved edits.
    /// New item is the way back to adding.
    fn open_menu_item(&mut self, cx: &mut Cx, index: usize) {
        app_state().lock().unwrap().dish_picker_open = false;
        let loaded = {
            let st = app_state().lock().unwrap();
            let Some(product) = st.products.get(index).cloned() else {
                return;
            };
            if st.editing_product_id.as_deref() == Some(product.id.as_str()) {
                return;
            } else {
                let currency = st.locale.currency.clone();
                let store = st.store.as_ref();
                let tax = store
                    .map(|s| menu_tax_text(s, &product))
                    .unwrap_or_default();
                let category = store
                    .and_then(|s| s.list_categories().ok())
                    .map(|list| {
                        name_for_id(
                            list.iter().map(|row| (row.id.as_str(), row.name.as_str())),
                            &product.category_id,
                        )
                    })
                    .unwrap_or_default();
                let aliases = product.aliases().join(", ");
                let modifiers = modifiers_as_typed(&product.modifier_groups(), &currency);
                let price = locale::format_major_input(product.price.paise(), &currency);
                let area_prices = options_as_typed(&product.area_prices(), &currency);
                let schedule = schedule_as_typed(&product.schedule(), &currency);
                let recipe = store
                    .and_then(|s| s.recipe(&product.id).ok())
                    .map(|r| store::recipe_as_typed(&r))
                    .unwrap_or_default();
                Some((product, tax, category, aliases, modifiers, price, area_prices, schedule, recipe))
            }
        };
        let Some((product, tax, category, aliases, modifiers, price, area_prices, schedule, recipe)) = loaded else {
            return;
        };
        self.ui.text_input(cx, ids!(menu_recipe_input)).set_text(cx, &recipe);
        self.ui.text_input(cx, ids!(menu_schedule_input)).set_text(cx, &schedule);
        // Sizes are items of their own: the quick-entry field starts empty
        // (filling it adds more sizes as new items).
        self.ui.text_input(cx, ids!(menu_variations_input)).set_text(cx, "");
        self.ui
            .text_input(cx, ids!(menu_group_input))
            .set_text(cx, &product.group_name);
        self.ui
            .text_input(cx, ids!(menu_area_prices_input))
            .set_text(cx, &area_prices);
        self.ui
            .text_input(cx, ids!(menu_name_input))
            .set_text(cx, &product.name);
        self.ui
            .text_input(cx, ids!(menu_aliases_input))
            .set_text(cx, &aliases);
        self.ui
            .text_input(cx, ids!(menu_barcode_input))
            .set_text(cx, &product.barcode);
        self.ui
            .text_input(cx, ids!(menu_hsn_input))
            .set_text(cx, &product.hsn_code);
        self.ui
            .text_input(cx, ids!(menu_composition_input))
            .set_text(cx, &product.composition);
        self.ui
            .text_input(cx, ids!(menu_rx_note_input))
            .set_text(cx, &product.allergens);
        {
            let mut st = app_state().lock().unwrap();
            st.form_schedule_h = product.schedule_h;
            st.form_rx_required = product.rx_required;
        }
        self.ui
            .text_input(cx, ids!(menu_price_input))
            .set_text(cx, &price);
        self.ui
            .text_input(cx, ids!(menu_unit_input))
            .set_text(cx, &product.unit_label);
        self.ui
            .text_input(cx, ids!(menu_tax_input))
            .set_text(cx, &tax);
        self.ui
            .text_input(cx, ids!(menu_category_input))
            .set_text(cx, &category);
        self.ui
            .text_input(cx, ids!(menu_modifiers_input))
            .set_text(cx, &modifiers);
        let station_name = {
            let st = app_state().lock().unwrap();
            st.stations
                .iter()
                .find(|s| s.id == product.station_id)
                .map(|s| s.name.clone())
                .unwrap_or_default()
        };
        self.ui
            .text_input(cx, ids!(menu_station_input))
            .set_text(cx, &station_name);
        {
            let mut st = app_state().lock().unwrap();
            st.editing_product_id = Some(product.id.clone());
            st.form_resale = product.kind == domain::ITEM_KIND_RESALE;
            st.form_schedule_h = product.schedule_h;
            st.form_rx_required = product.rx_required;
            st.draft_image_key = product.image_key.clone();
            // The loaded item's category travels by UUID too.
            if product.category_id.is_empty() {
                st.form_category_id = None;
                st.form_category_label.clear();
            } else {
                st.form_category_id = Some(product.category_id.clone());
                st.form_category_label = category.clone();
            }
            // The item's own category is the owner's choice, never replaced
            // by a later dish pick; no shelf photo pending.
            st.form_category_auto = false;
            st.form_category_photo = None;
            st.category_picker_open = false;
        }
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    /// Put a category in the item form's Category field. `id` links one of
    /// the owner's categories by UUID; `photo` is a shelf folder's photo for
    /// a category created on save; `auto` marks it as filled by the app (a
    /// later pick may replace it, a typed one never is).
    fn set_form_category(
        &mut self,
        cx: &mut Cx,
        name: &str,
        id: Option<String>,
        photo: Option<PathBuf>,
        auto: bool,
    ) {
        {
            let mut st = app_state().lock().unwrap();
            st.form_category_id = id;
            st.form_category_label = name.to_string();
            st.form_category_photo = photo.map(|p| (name.to_string(), p));
            st.form_category_auto = auto;
            st.category_picker_open = false;
        }
        self.ui.text_input(cx, ids!(menu_category_input)).set_text(cx, name);
    }

    /// A picked dish brings its shelf folder as the category, editable:
    /// the owner's own category of that name when there is one (by UUID),
    /// else the folder's name (created on save, with the folder's photo).
    /// A category the owner typed is left alone.
    fn prefill_category_from_dish(&mut self, cx: &mut Cx, dish: &menu_images::ShelfDish) {
        let typed = self.ui.text_input(cx, ids!(menu_category_input)).text();
        let (replace, existing, photo) = {
            let st = app_state().lock().unwrap();
            let replace = typed.trim().is_empty() || st.form_category_auto;
            let existing = st
                .categories
                .iter()
                .find(|c| c.name.trim().eq_ignore_ascii_case(dish.folder.trim()))
                .map(|c| (c.id.clone(), c.name.clone()));
            let photo = menu_images::shelf_folders(menu_images::shelf())
                .into_iter()
                .find(|f| f.label == dish.folder)
                .map(|f| f.photo);
            (replace, existing, photo)
        };
        if !replace || dish.folder.trim().is_empty() {
            return;
        }
        match existing {
            Some((id, name)) => self.set_form_category(cx, &name, Some(id), None, true),
            None => self.set_form_category(cx, &dish.folder, None, photo, true),
        }
    }

    /// Copy a shelf dish into the item form: name + photo, nothing saved yet.
    /// The shelf is bundled, read-only art — picking a dish writes no item,
    /// sets no price; its shelf folder is suggested as the category
    /// (editable, see [`Self::prefill_category_from_dish`]). The owner sets
    /// the price (blank is ₹0) and taps Add item.
    fn use_shelf_dish(&mut self, cx: &mut Cx, dish: &menu_images::ShelfDish) {
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let actor = st.actor()?;
            if !actor.is_owner() && !actor.can_access_items() {
                return Err("Not allowed to manage items".into());
            }
            // A fresh item. The form's own choices (category, station, unit,
            // tax) stay as the owner left them — only name + photo come from
            // the shelf.
            st.editing_product_id = None;
            st.draft_image_key = Some(format!("pending_custom:{}", dish.path.display()));
            Ok(format!("{} — set your price, then Add item", dish.name))
        })();
        match result {
            Ok(msg) => {
                self.ui
                    .text_input(cx, ids!(menu_name_input))
                    .set_text(cx, &dish.name);
                // The price belongs to the dish being added — never inherited
                // from whatever was in the form before.
                self.ui.text_input(cx, ids!(menu_price_input)).set_text(cx, "");
                self.prefill_category_from_dish(cx, dish);
                self.ui
                    .view(cx, ids!(menu_screen))
                    .set_scroll_pos(cx, dvec2(0.0, 0.0));
                self.set_status(cx, msg);
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Keep the typed text as the owner's own item name. A photo that came
    /// from a shelf dish picked earlier belonged to that dish, so it goes;
    /// a photo the owner picked from their own files stays.
    fn use_own_item_name(&mut self, cx: &mut Cx, name: &str) {
        {
            let mut st = app_state().lock().unwrap();
            let from_shelf = st
                .draft_image_key
                .as_deref()
                .and_then(|k| k.strip_prefix("pending_custom:"))
                .zip(menu_images::catalogue_root())
                .is_some_and(|(path, root)| std::path::Path::new(path).starts_with(root));
            if from_shelf {
                st.draft_image_key = None;
            }
        }
        self.ui.text_input(cx, ids!(menu_name_input)).set_text(cx, name);
        self.set_status(cx, format!("{name} — set your price, then Add item"));
    }

    fn remove_kitchen_station(&mut self, cx: &mut Cx, typed: &str) -> Result<String, String> {
        let _ = cx;
        let mut st = app_state().lock().unwrap();
        let needle = typed.trim().to_lowercase();
        if needle.is_empty() {
            return Err(st.t("station_remove_ph"));
        }
        let id = st
            .stations
            .iter()
            .find(|s| s.name.trim().to_lowercase() == needle || s.id == typed.trim())
            .map(|s| s.id.clone())
            .ok_or_else(|| "Station not found".to_string())?;
        let name = st
            .stations
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| id.clone());
        let msg = st.books_op(BooksOp::RemoveStation { id: id.clone() })?;
        if st.device_role != DeviceRole::Client {
            st.printer_draft.remove_station_route(&id);
            let _ = st.save_printer_draft();
        }
        Ok(if st.device_role == DeviceRole::Client {
            msg
        } else {
            st.t("station_removed").replace("{name}", &name)
        })
    }

    /// Switch this device's active branch (list pick or typed name).
    fn switch_active_branch(&mut self, cx: &mut Cx, wanted: &str) -> Result<String, String> {
        let _ = cx;
        let mut st = app_state().lock().unwrap();
        let actor = st.actor()?;
        if !actor.can_access_branches() && !actor.is_owner() {
            return Err("Only Owner can switch branch".into());
        }
        let needle = wanted.trim().to_lowercase();
        if needle.is_empty() {
            return Err("Type the branch name to switch to".into());
        }
        let (id, label) = {
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            let branch = store
                .list_branches()?
                .into_iter()
                .find(|b| {
                    b.name.to_lowercase() == needle
                        || b.code.to_lowercase() == needle
                        || b.name.to_lowercase().contains(&needle)
                })
                .ok_or_else(|| format!("No branch matches \"{}\"", wanted.trim()))?;
            (branch.id.clone(), branch.label())
        };
        let msg = st.books_op(BooksOp::SwitchBranch { id })?;
        st.cart.clear();
        st.open_cart_line = None;
        st.selected_table_id = None;
        if st.device_role != DeviceRole::Client {
            st.refresh_branches_text()?;
        }
        Ok(if st.device_role == DeviceRole::Client {
            msg
        } else {
            format!("Now selling for {label}")
        })
    }

    /// Order desk: Don't use on "Kitchen (no station)" — hold those lines
    /// off this KOT (same as Hold on each line) so Send only names real stations.
    fn hold_unassigned_kot_lines(&mut self, cx: &mut Cx) -> Result<String, String> {
        let mut st = app_state().lock().unwrap();
        // Local cart only — waiters on an extra till must be able to Hold.
        let idxs: Vec<usize> = st
            .cart
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| !l.is_sent() && !l.hold_send)
            .filter(|(_, l)| l.kitchen_station_id(&st.products).is_empty())
            .map(|(i, _)| i)
            .collect();
        let n = idxs.len();
        for i in idxs {
            if let Some(line) = st.cart.lines.get_mut(i) {
                line.hold_send = true;
            }
        }
        let msg = st
            .t("kot_unassigned_held")
            .replace("{n}", &n.to_string());
        drop(st);
        let _ = cx;
        Ok(msg)
    }

    /// Kitchen dropdown: ↑↓ swap rows → update print priority (and row order).
    /// `from`/`to` are indices into the open `suggest_rows` list.
    fn reorder_kot_station_priority(&mut self, cx: &mut Cx, from: usize, to: usize) {
        let mut st = app_state().lock().unwrap();
        if from == to || from >= st.suggest_rows.len() {
            return;
        }
        let row = st.suggest_rows.remove(from);
        let insert_at = to.min(st.suggest_rows.len());
        st.suggest_rows.insert(insert_at, row);
        // Rebuild priority from station rows (skip the Unassigned pseudo-row).
        let unassigned = st.t("kot_unassigned");
        let mut priority = Vec::new();
        for r in &st.suggest_rows {
            if r.value.eq_ignore_ascii_case(&unassigned) {
                continue;
            }
            if let Some(id) = st
                .stations
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(&r.value))
                .map(|s| s.id.clone())
            {
                priority.push(id);
            }
        }
        st.kot_station_priority = priority;
        st.persist_kot_station_prefs();
        // Keep suggest_rev so the open list does not jump to the top mid-drag.
        drop(st);
        self.ui
            .widget(cx, ids!(kot_station_suggest))
            .redraw(cx);
        self.sync_chrome(cx);
    }

    /// Order desk: clear `station_text` from unsent cart dishes that use it
    /// (Don't use in the station list). Held / already-sent lines stay.
    fn clear_kot_station_from_unsent(
        &mut self,
        cx: &mut Cx,
        station_text: &str,
    ) -> Result<String, String> {
        let wanted = station_text.trim();
        if wanted.is_empty() {
            let msg = app_state().lock().unwrap().t("kot_station_needed");
            return Err(msg);
        }
        {
            let st = app_state().lock().unwrap();
            if wanted.eq_ignore_ascii_case(&st.t("kot_unassigned"))
                || wanted.eq_ignore_ascii_case("unassigned")
                || wanted.eq_ignore_ascii_case("unassigned items (no station)")
            {
                drop(st);
                return self.hold_unassigned_kot_lines(cx);
            }
        }
        let mut st = app_state().lock().unwrap();
        let (station_id, station_name) = st
            .stations
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(wanted))
            .map(|s| (s.id.clone(), s.name.clone()))
            .ok_or_else(|| "Station not found".to_string())?;
        // Cart-local clear — do not rewrite the menu from an extra till.
        let idxs: Vec<usize> = st
            .cart
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| !l.is_sent() && !l.hold_send)
            .filter(|(_, l)| l.kitchen_station_id(&st.products) == station_id)
            .map(|(i, _)| i)
            .collect();
        let mut n = 0usize;
        for i in idxs {
            let pid = st.cart.lines[i].product_id.clone();
            let menu_sid = st
                .products
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.station_id.clone())
                .unwrap_or_default();
            let line = &mut st.cart.lines[i];
            line.station_id.clear();
            // Menu still routes here → hold off this KOT instead of reappearing.
            if menu_sid == station_id {
                line.hold_send = true;
            }
            n += 1;
        }
        st.deselect_kot_station(&station_id);
        let msg = st
            .t("kot_station_cleared")
            .replace("{n}", &n.to_string())
            .replace("{station}", &station_name);
        drop(st);
        let _ = cx;
        Ok(msg)
    }

    /// Order desk: put `station_text` on every unsent cart dish for this
    /// ticket (cart-local — works on an extra till). Does not rewrite the
    /// menu. Unknown station names are created via books (Server or Client
    /// LAN). Held lines are left alone.
    fn apply_kot_station_to_unassigned(
        &mut self,
        cx: &mut Cx,
        station_text: &str,
    ) -> Result<String, String> {
        let wanted = station_text.trim();
        if wanted.is_empty() {
            let msg = app_state().lock().unwrap().t("kot_station_needed");
            return Err(msg);
        }
        let mut st = app_state().lock().unwrap();
        let (station_id, station_name) = {
            let existing = st
                .stations
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(wanted))
                .map(|s| (s.id.clone(), s.name.clone()));
            if let Some(pair) = existing {
                pair
            } else {
                let name = wanted.to_string();
                let msg = st.books_op(BooksOp::AddStation {
                    name: name.clone(),
                })?;
                match st
                    .stations
                    .iter()
                    .find(|s| s.name.eq_ignore_ascii_case(&name))
                    .map(|s| (s.id.clone(), s.name.clone()))
                {
                    Some(pair) => pair,
                    // Client: create is on the shop; list refreshes after Sync.
                    None => return Ok(msg),
                }
            }
        };
        if let Some(session) = st.session.as_ref() {
            if !session.effective_features().allows_kot_station(&station_id) {
                return Err(st
                    .t("kot_station_not_allowed")
                    .replace("{station}", &station_name));
            }
        }
        // Multi-check: only assign dishes that still have no kitchen. Never
        // overwrite another station (that felt like exclusive radios).
        let products = st.products.clone();
        let mut n = 0usize;
        for line in st
            .cart
            .lines
            .iter_mut()
            .filter(|l| !l.is_sent() && !l.hold_send)
        {
            if !line.kitchen_station_id(&products).is_empty() {
                continue;
            }
            line.station_id = station_id.clone();
            n += 1;
        }
        st.select_kot_station(&station_id);
        let msg = st
            .t("kot_station_applied")
            .replace("{n}", &n.to_string())
            .replace("{station}", &station_name);
        drop(st);
        let _ = cx;
        Ok(msg)
    }
}

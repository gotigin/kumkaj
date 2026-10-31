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
                        line_id,
                        qty_milli,
                        reason: reason.trim().to_string(),
                        prepared: st.line_cooked,
                    }
                }
                LineAction::Complimentary => net::RunningAction::Complimentary {
                    line_id,
                    on: !line.complimentary,
                    reason: reason.trim().to_string(),
                },
                LineAction::Move => {
                    let to = st
                        .tables
                        .iter()
                        .find(|t| t.name.trim().eq_ignore_ascii_case(target.trim()))
                        .map(|t| t.id.clone())
                        .ok_or_else(|| format!("No table named \"{}\"", target.trim()))?;
                    net::RunningAction::MoveLines {
                        line_ids: vec![line_id],
                        to_table_id: to,
                    }
                }
            };
            let done = match action {
                LineAction::Cancel => format!("{} cancelled — the kitchen gets a cancel slip", line.name),
                LineAction::Complimentary if line.complimentary => format!("{} charged again", line.name),
                LineAction::Complimentary => format!("{} is complimentary (NC)", line.name),
                LineAction::Move => format!("{} moved to {}", line.name, target.trim()),
            };
            let acted = st.table_action(request, &pin, &done, phone::Follow::Refresh)?;
            st.line_action = None;
            Ok(match acted {
                Acted::Here(_) => done,
                Acted::Sent => format!("{} — sending…", line.name),
            })
        })();
        match result {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Print the open table's bill (table turns 🟢 on every device).
    fn print_table_bill(&mut self, cx: &mut Cx) {
        let pin = self.ui.text_input(cx, ids!(move_pin_input)).text();
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let table_id = st
                .selected_table_id
                .clone()
                .ok_or_else(|| "Open a table first".to_string())?;
            if st.cart.lines.iter().any(|l| !l.is_sent()) {
                return Err("Send the new items to the kitchen first (Send KOT), then Print bill".into());
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
            match print_share::print_route(
                print_share::PrintKind::Bill,
                print_share::RouteKey::Bill,
                &text,
                dir.as_deref(),
            ) {
                Ok(share) if share.opened => Ok(done.to_string()),
                Ok(_) => Ok(format!(
                    "{done} · add a bill printer in Settings → Where things print (station printers are for KOTs only)"
                )),
                Err(e) => Ok(format!("{done} · paper not sent ({e})")),
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
                            .filter(|p| st.cart.qty_of(&line.product_id) + delta > p.stock_qty)
                            .map(|p| format!("Only {} of {} in stock", p.stock_qty, p.name))
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

    /// Single-tender charge (full bill one method). Sell UI uses amount fields +
    /// Take payment; kept for amend/automation paths that still need one tender.
    #[allow(dead_code)]
    fn pay(&mut self, cx: &mut Cx, method: PayMethod) {
        // A remote charge holds no lock while it waits on the LAN, so a second
        // tap would otherwise start a second bill for the same cart.
        if !self.begin_pay(cx) {
            return;
        }
        // Apply typed total even if seller didn't press Enter.
        let typed_total = self.ui.text_input(cx, ids!(cart_total_input)).text();
        let typed_customer = self.ui.text_input(cx, ids!(customer_name_input)).text();
        let result = (|| -> Result<Option<(String, String)>, String> {
            enum Pending {
                LocalDone { sale: Sale, snack: String },
                Remote {
                    draft: domain::SaleDraft,
                    table_id: Option<String>,
                    table_name: Option<String>,
                    customer_id: Option<String>,
                    customer_name: Option<String>,
                },
            }
            let pending = {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_sell() {
                    return Err("Not allowed to sell".into());
                }
                let currency = st.locale.currency.clone();
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
                st.require_cart_modifiers()?;
                if st.selected_table_id.is_some()
                    && st.cart.lines.iter().any(|l| !l.is_sent())
                {
                    return Err(
                        "Send the new items to the kitchen first (Send KOT), then Take payment"
                            .into(),
                    );
                }
                if st.cart.charge_total.is_some()
                    && let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                        && minor > 0 {
                            let _ = st.cart.set_charge_total(Money(minor));
                        }
                let mut draft = st.cart.to_sale_draft()?;
                draft.money_lane = st.money_lane;
                let table_id = st.selected_table_id.clone();
                let is_client = st.device_role == DeviceRole::Client;
                let customer_id = if method.is_credit() {
                    if let Some(id) = st.selected_customer_id.clone() {
                        Some(id)
                    } else if typed_customer.trim().is_empty() {
                        return Err("Type a customer name, then tap Credit (khata)".into());
                    } else if is_client {
                        // Leave the id unset so the Server resolves the name in
                        // its own customer book. Inventing one here would bill
                        // credit against a customer the Server has never heard of.
                        None
                    } else {
                        let name = typed_customer.trim();
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        let customer = store.find_or_create_customer(name)?;
                        st.selected_customer_id = Some(customer.id.clone());
                        Some(customer.id)
                    }
                } else {
                    // A cash bill may still carry the picked customer's name.
                    st.selected_customer_id.clone()
                };
                if st.device_role == DeviceRole::Client {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    if !store.client_paired()? {
                        return Err("Client is not paired to a Server — connect first".into());
                    }
                    let table_name = table_id.as_ref().and_then(|tid| {
                        st.tables.iter().find(|t| &t.id == tid).map(|t| t.name.clone())
                    });
                    let customer_name = if method.is_credit() {
                        Some(typed_customer.trim().to_string()).filter(|s| !s.is_empty())
                    } else {
                        None
                    };
                    Pending::Remote {
                        draft,
                        table_id,
                        table_name,
                        customer_id,
                        customer_name,
                    }
                } else {
                    let sale = {
                        let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                        store.finalize_sale(
                            &draft,
                            method,
                            table_id.as_deref(),
                            customer_id.as_deref(),
                            &actor,
                        )?
                    };
                    let amount = st.money(Money(sale.total_paise));
                    let tender = method.label_for(&st.fiscal);
                    let receipt = if sale.rounding_paise != 0 {
                        format!(
                            "Sale {} · Rounded {} (Applied {} · round {}) · {} · {}",
                            &sale.id[..8.min(sale.id.len())],
                            amount,
                            st.money(Money(sale.lines_total_paise)),
                            st.money(Money(sale.rounding_paise)),
                            tender,
                            &sale.created_at[..sale.created_at.len().min(19)]
                        )
                    } else {
                        format!(
                            "Sale {} · Rounded {} · {} · {}",
                            &sale.id[..8.min(sale.id.len())],
                            amount,
                            tender,
                            &sale.created_at[..sale.created_at.len().min(19)]
                        )
                    };
                    let full = st
                        .store
                        .as_ref()
                        .and_then(|s| s.format_receipt_text(&sale.id).ok())
                        .unwrap_or(receipt);
                    st.cart.clear();
                    st.open_cart_line = None;
                    st.selected_table_id = None;
                    if method.is_credit() {
                        st.selected_customer_id = None;
                    }
                    st.last_receipt = full;
                    st.status = format!("Billed {} · {}", sale.invoice_no, method.label());
                    let snack = format!("{} · {} billed", amount, method.label());
                    st.reload_all()?;
                    Pending::LocalDone { sale, snack }
                }
            };
            match pending {
                Pending::LocalDone { sale, snack } => Ok(Some((sale.id, snack))),
                Pending::Remote {
                    draft,
                    table_id,
                    table_name,
                    customer_id,
                    customer_name,
                } => {
                    // Saved on this device first, then sent: a Wi‑Fi drop
                    // delays the bill, it never loses it.
                    let mut st = app_state().lock().unwrap();
                    let amount = st.money(Money(draft.total_paise));
                    let req = net::RemoteSaleRequest {
                        draft,
                        pay: method.as_str().into(),
                        customer_id,
                        customer_name,
                        table_name: table_name.clone(),
                        table_id,
                        payments: Vec::new(),
                        tendered_paise: 0,
                        tip_paise: 0,
                    };
                    let place = table_name.unwrap_or_else(|| "counter".into());
                    st.lan_job(
                        "/sale",
                        &req,
                        format!("Payment {amount} · {place}"),
                        format!("{amount} · {} recorded", method.label()),
                        phone::Follow::Sale,
                        true,
                    )?;
                    st.cart.clear();
                    st.open_cart_line = None;
                    st.selected_table_id = None;
                    if method.is_credit() {
                        st.selected_customer_id = None;
                    }
                    st.status = format!("{amount} · {} saved — sending to the counter", method.label());
                    Ok(None)
                }
            }
        })();
        self.end_pay();
        match result {
            Err(e) => {
                // The ledger line is what the seller is looking at. Pay
                // failures must show there, not only on the lock screen.
                app_state().lock().unwrap().last_receipt = e.clone();
                if e.contains("customer") {
                    self.ui
                        .text_input(cx, ids!(customer_name_input))
                        .set_key_focus(cx);
                }
                self.set_status(cx, e);
            }
            Ok(done) => {
                if method.is_credit() {
                    self.ui
                        .text_input(cx, ids!(customer_name_input))
                        .set_text(cx, "");
                }
                match done {
                    Some((sale_id, snack)) => self.show_undo_snackbar(cx, sale_id, snack),
                    // Queued on this device: say so where the seller looks.
                    None => {
                        let msg = app_state().lock().unwrap().status.clone();
                        self.set_status(cx, msg);
                    }
                }
            }
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
        st.form_resale = false;
        st.draft_image_key = None;
        st.form_category_id = None;
        st.form_category_label.clear();
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
            st.require_server_role("Add items")?;
            let actor = st.actor()?;
            if !actor.role.can_manage_menu() {
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
        st.require_server_role("Remove kitchen stations")?;
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
        let name = {
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.remove_station(&id)?
        };
        st.stations = st
            .store
            .as_ref()
            .map(|s| s.list_stations().unwrap_or_default())
            .unwrap_or_default();
        st.printer_draft.remove_station_route(&id);
        if let Some(base) = st.data_base.as_ref() {
            let _ = print_share::save_device_printers(base, &st.printer_draft);
        }
        Ok(st.t("station_removed").replace("{name}", &name))
    }

    /// Switch this device's active branch (list pick or typed name).
    fn switch_active_branch(&mut self, cx: &mut Cx, wanted: &str) -> Result<String, String> {
        let _ = cx;
        let mut st = app_state().lock().unwrap();
        st.require_server_role("Switch branch")?;
        let actor = st.actor()?;
        if !actor.can_access_branches() && !actor.is_owner() {
            return Err("Only Owner can switch branch".into());
        }
        let needle = wanted.trim().to_lowercase();
        if needle.is_empty() {
            return Err("Type the branch name to switch to".into());
        }
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
        store.set_active_branch_id(&branch.id)?;
        st.cart.clear();
        st.open_cart_line = None;
        st.selected_table_id = None;
        st.reload_all()?;
        st.refresh_branches_text()?;
        Ok(format!("Now selling for {}", branch.label()))
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
    /// menu. Unknown station names: create on the shop device only; on a
    /// client, pick an existing kitchen. Held lines are left alone.
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
            } else if st.device_role == DeviceRole::Client {
                return Err(
                    "Unknown kitchen — pick one that exists, or add stations on the shop device"
                        .into(),
                );
            } else {
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let s = store.add_station(wanted)?;
                st.stations = store.list_stations().unwrap_or_default();
                (s.id, s.name)
            }
        };
        let mut n = 0usize;
        for line in st
            .cart
            .lines
            .iter_mut()
            .filter(|l| !l.is_sent() && !l.hold_send)
        {
            if line.station_id == station_id {
                continue;
            }
            line.station_id = station_id.clone();
            n += 1;
        }
        let msg = st
            .t("kot_station_applied")
            .replace("{n}", &n.to_string())
            .replace("{station}", &station_name);
        drop(st);
        let _ = cx;
        Ok(msg)
    }
}

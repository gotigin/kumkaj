impl App {
    fn handle_actions_ops(&mut self, cx: &mut Cx, actions: &Actions) {
        // Moklanjo ops desk actions (Ops / COD / Riders facets on Reports).
        {
            let awb_raw = self.ui.text_input(cx, ids!(ops_awb_input)).text();
            let awb = courier_ops_view::normalize_awb_input(&awb_raw);
            // Wedge scanners usually send Enter after the code.
            let awb_enter = self
                .ui
                .text_input(cx, ids!(ops_awb_input))
                .returned(actions)
                .is_some();
            let find_clicked = self.ui.button(cx, ids!(ops_find_awb)).clicked(actions);
            if (find_clicked || awb_enter) && !awb.is_empty() {
                if awb != awb_raw.trim() {
                    self.ui
                        .text_input(cx, ids!(ops_awb_input))
                        .set_text(cx, &awb);
                }
                match self.resolve_ops_awb_card(cx, &awb) {
                    Ok(card) => {
                        self.set_status(cx, format!("Found {awb}"));
                        let _ = card;
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }

            let note = self.ui.text_input(cx, ids!(ops_note_input)).text();
            let rider = self.ui.text_input(cx, ids!(ops_rider_input)).text();
            let carrier = self.ui.text_input(cx, ids!(ops_carrier_input)).text();
            let party = self.ui.text_input(cx, ids!(ops_party_input)).text();
            let rider_phone = self.ui.text_input(cx, ids!(ops_rider_phone)).text();
            let rider_vehicle = self.ui.text_input(cx, ids!(ops_rider_vehicle)).text();

            let mut ops_msg: Option<Result<String, String>> = None;
            if self.ui.button(cx, ids!(ops_pod_pick)).clicked(actions) {
                self.attach_pick_ops_pod(cx);
            }
            // NDR reason chips fill the note field for Failed delivery.
            let ndr_chip = if self.ui.button(cx, ids!(ops_ndr_door)).clicked(actions) {
                Some("door_locked")
            } else if self.ui.button(cx, ids!(ops_ndr_refused)).clicked(actions) {
                Some("refused")
            } else if self.ui.button(cx, ids!(ops_ndr_address)).clicked(actions) {
                Some("address_wrong")
            } else if self.ui.button(cx, ids!(ops_ndr_unavailable)).clicked(actions) {
                Some("customer_unavailable")
            } else if self.ui.button(cx, ids!(ops_ndr_other)).clicked(actions) {
                Some("other")
            } else {
                None
            };
            if let Some(reason) = ndr_chip {
                self.ui
                    .text_input(cx, ids!(ops_note_input))
                    .set_text(cx, reason);
                self.set_status(cx, format!("NDR reason: {reason}"));
            }
            let bulk_raw = self.ui.text_input(cx, ids!(ops_bulk_awb_input)).text();
            let bulk_awbs = store::Store::parse_awb_list(&bulk_raw);
            if self.ui.button(cx, ids!(ops_advance_ofd)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Advance tracking")?;
                    let actor = st.actor()?;
                    let use_bulk = bulk_awbs.len() > 1 || (bulk_awbs.len() == 1 && awb.is_empty());
                    let msg = if use_bulk {
                        let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                        let (ok, errs) = store.bulk_advance_tracking(
                            &bulk_awbs,
                            courier_ops_view::TrackingStatus::OutForDelivery,
                            &note,
                            &actor,
                        )?;
                        let mut m = format!("Bulk OFD · ok {ok}");
                        for e in errs.iter().take(5) {
                            m.push('\n');
                            m.push_str(e);
                        }
                        m
                    } else {
                        let pod = st.pending_ops_pod.take();
                        let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                        courier_ops_view::advance_awb_with_pod(
                            store,
                            &awb,
                            courier_ops_view::TrackingStatus::OutForDelivery,
                            &note,
                            pod.as_deref(),
                            &actor,
                        )?
                    };
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self
                .ui
                .button(cx, ids!(ops_advance_delivered))
                .clicked(actions)
            {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Mark delivered")?;
                    let actor = st.actor()?;
                    let use_bulk =
                        bulk_awbs.len() > 1 || (bulk_awbs.len() == 1 && awb.is_empty());
                    let msg = if use_bulk {
                        let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                        let (ok, errs) = store.bulk_advance_tracking(
                            &bulk_awbs,
                            courier_ops_view::TrackingStatus::Delivered,
                            &note,
                            &actor,
                        )?;
                        let mut m = format!("Bulk delivered · ok {ok}");
                        for e in errs.iter().take(5) {
                            m.push('\n');
                            m.push_str(e);
                        }
                        m
                    } else {
                        let pod = st.pending_ops_pod.take();
                        let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                        courier_ops_view::advance_awb_with_pod(
                            store,
                            &awb,
                            courier_ops_view::TrackingStatus::Delivered,
                            &note,
                            pod.as_deref(),
                            &actor,
                        )?
                    };
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_label)).clicked(actions) {
                ops_msg = Some((|| {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    courier_ops_view::print_label(store, &awb)
                })());
            } else if self.ui.button(cx, ids!(ops_ndr)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Failed delivery")?;
                    let actor = st.actor()?;
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let (reason, next) = {
                        let raw = note.trim();
                        if let Some((r, d)) = raw.split_once('|') {
                            (r.trim().to_string(), d.trim().to_string())
                        } else {
                            (raw.to_string(), String::new())
                        }
                    };
                    let reason = if reason.is_empty() {
                        "other".into()
                    } else {
                        reason
                    };
                    let msg = courier_ops_view::record_failed_delivery(
                        store, &awb, &reason, &next, &actor,
                    )?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_reattempt)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Reattempt")?;
                    let actor = st.actor()?;
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let msg = courier_ops_view::reattempt_awb(store, &awb, &actor)?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_print_run_sheet)).clicked(actions)
                || self.ui.button(cx, ids!(ops_print_drs)).clicked(actions)
            {
                ops_msg = Some((|| {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    courier_ops_view::print_latest_run_sheet(store)
                })());
            } else if self
                .ui
                .button(cx, ids!(ops_settle_rider_cod))
                .clicked(actions)
            {
                let handed = self.ui.text_input(cx, ids!(ops_rider_cod_input)).text();
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Settle rider COD")?;
                    let actor = st.actor()?;
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let msg =
                        courier_ops_view::settle_rider_cod_line(store, &rider, &handed, &actor)?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self
                .ui
                .button(cx, ids!(ops_dispatch_fleet))
                .clicked(actions)
            {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Dispatch own fleet")?;
                    let actor = st.actor()?;
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let msg =
                        courier_ops_view::dispatch_booked_to_rider(store, &rider, &note, &actor)?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self
                .ui
                .button(cx, ids!(ops_handover_carrier))
                .clicked(actions)
            {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Carrier handover")?;
                    let actor = st.actor()?;
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let msg = courier_ops_view::handover_booked_to_carrier(
                        store, &carrier, &note, &actor,
                    )?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_cod_collect)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Collect COD")?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let msg = courier_ops_view::collect_cod_for_awb(store, &awb)?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_cod_remit)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Remit COD")?;
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let msg = courier_ops_view::remit_collected_for_party(store, &party)?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_add_rider)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Add rider")?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let msg = courier_ops_view::add_rider_line(
                        store,
                        &rider,
                        &rider_phone,
                        &rider_vehicle,
                    )?;
                    st.reports_nav = ReportsNav::Riders;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_period_bill)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Period bill")?;
                    let actor = st.actor()?;
                    let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                    let msg = courier_ops_view::period_bill_party(store, &party, &actor)?;
                    st.refresh_reports_facet()?;
                    Ok(msg)
                })());
            }
            if let Some(result) = ops_msg {
                match result {
                    Ok(msg) => {
                        self.set_status(cx, msg);
                        if !awb.is_empty() {
                            let _ = self.resolve_ops_awb_card(cx, &awb);
                        }
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }
        if self.ui.button(cx, ids!(reports_copy)).clicked(actions) {
            let text = app_state().lock().unwrap().day_close_text.clone();
            if text.trim().is_empty() {
                self.set_status(cx, "Open a report first");
            } else {
                self.copy_text_to_clipboard(cx, &text);
            }
        }
        if self.ui.button(cx, ids!(expense_add)).clicked(actions) {
            let amount_text = self.ui.text_input(cx, ids!(expense_amount_input)).text();
            let note = self.ui.text_input(cx, ids!(expense_note_input)).text();
            let part2_text = self.ui.text_input(cx, ids!(exp_part2)).text();
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if st.device_role == DeviceRole::Client {
                    return Err("Log expenses on the shop device — this is an extra till".into());
                }
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                let parts = Self::expense_parts(
                    minor,
                    &part2_text,
                    &currency,
                    st.expense_tender,
                    st.expense_tender2,
                )?;
                let expense_id = {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.add_expense_tenders(minor, "misc", &note, &actor, &parts)?
                };
                st.selected_expense_id = Some(expense_id);
                st.day_close_text = st
                    .store
                    .as_ref()
                    .ok_or_else(|| "No store".to_string())?
                    .books_report_text("today")?;
                st.nav_back.clear();
                st.nav_forward.clear();
                st.screen = Screen::Reports;
                st.status = "Expense logged".into();
                Ok(())
            })();
            if let Err(e) = result {
                self.set_status(cx, e);
            } else {
                self.ui
                    .text_input(cx, ids!(expense_amount_input))
                    .set_text(cx, "");
                self.ui
                    .text_input(cx, ids!(expense_note_input))
                    .set_text(cx, "");
                self.ui.text_input(cx, ids!(exp_part2)).set_text(cx, "");
            }
            self.sync_chrome(cx);
        }
        if self
            .ui
            .button(cx, ids!(sale_attach_add_note))
            .clicked(actions)
        {
            self.add_entry_note(cx, AttachmentEntity::Sale, ids!(sale_attach_note));
        }
        if self.ui.button(cx, ids!(sale_attach_photo)).clicked(actions) {
            self.attach_pick_image(cx, AttachmentEntity::Sale);
        }
        if self.ui.button(cx, ids!(sale_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Sale);
        }
        if self
            .ui
            .button(cx, ids!(party_attach_add_note))
            .clicked(actions)
        {
            self.add_entry_note(cx, AttachmentEntity::Party, ids!(party_attach_note));
        }
        if self
            .ui
            .button(cx, ids!(party_attach_photo))
            .clicked(actions)
        {
            self.attach_pick_image(cx, AttachmentEntity::Party);
        }
        if self.ui.button(cx, ids!(party_attach_pdf)).clicked(actions) {
            self.attach_pick_pdf(cx, AttachmentEntity::Party);
        }
        if self
            .ui
            .button(cx, ids!(expense_attach_add_note))
            .clicked(actions)
        {
            self.add_entry_note(cx, AttachmentEntity::Expense, ids!(expense_attach_note));
        }
        if self
            .ui
            .button(cx, ids!(expense_attach_photo))
            .clicked(actions)
        {
            self.attach_pick_image(cx, AttachmentEntity::Expense);
        }
        if self
            .ui
            .button(cx, ids!(expense_attach_pdf))
            .clicked(actions)
        {
            self.attach_pick_pdf(cx, AttachmentEntity::Expense);
        }
        for (id, kind) in [
            (ids!(bank_till_to_bank), MoneyMoveKind::TillToBank),
            (ids!(bank_bank_to_till), MoneyMoveKind::BankToTill),
            (ids!(bank_in), MoneyMoveKind::BankIn),
            (ids!(bank_out), MoneyMoveKind::BankOut),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                let amount_text = self.ui.text_input(cx, ids!(bank_amount_input)).text();
                let note = self.ui.text_input(cx, ids!(bank_note_input)).text();
                let result = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    st.require_server_role("Post bank / cashbook")?;
                    let actor = st.actor()?;
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&amount_text, &currency)?;
                    st.store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .post_money_movement(kind, minor, &note, &actor)?;
                    st.day_close_text = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .books_report_text("today")?;
                    st.nav_back.clear();
                    st.nav_forward.clear();
                    st.screen = Screen::Reports;
                    st.status = format!("{} posted", kind.label());
                    Ok(())
                })();
                if let Err(e) = result {
                    self.set_status(cx, e);
                } else {
                    self.ui
                        .text_input(cx, ids!(bank_amount_input))
                        .set_text(cx, "");
                    self.ui
                        .text_input(cx, ids!(bank_note_input))
                        .set_text(cx, "");
                }
                self.sync_chrome(cx);
            }
        }

        // Bank reconciliation tick-off: # refers to the numbered
        // uncleared list under Bank / cashbook.
        for (btn, cleared) in [
            (ids!(bank_mark_cleared), true),
            (ids!(bank_mark_uncleared), false),
        ] {
            if self.ui.button(cx, btn).clicked(actions) {
                let num_text = self.ui.text_input(cx, ids!(bank_clear_input)).text();
                let result = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    st.require_server_role("Reconcile bank movements")?;
                    let actor = st.actor()?;
                    let n: usize = num_text
                        .trim()
                        .parse()
                        .map_err(|_| "Type the uncleared # from the list".to_string())?;
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let uncleared: Vec<String> = store
                        .list_money_movements(50)?
                        .into_iter()
                        .filter(|m| !m.cleared)
                        .map(|m| m.id)
                        .collect();
                    let id = uncleared
                        .get(n.wrapping_sub(1))
                        .cloned()
                        .ok_or_else(|| "No such uncleared movement".to_string())?;
                    store.set_money_movement_cleared(&id, cleared, &actor)?;
                    Ok(if cleared {
                        "Marked cleared — seen on the statement".into()
                    } else {
                        "Unmarked".into()
                    })
                })();
                match result {
                    Ok(msg) => {
                        self.ui
                            .text_input(cx, ids!(bank_clear_input))
                            .set_text(cx, "");
                        self.set_status(cx, msg);
                    }
                    Err(e) => self.set_status(cx, e),
                }
                self.sync_chrome(cx);
            }
        }

    }
}

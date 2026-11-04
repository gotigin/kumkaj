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
                ops_msg = Some(self.books_advance_tracking(
                    &awb,
                    &bulk_awbs,
                    &note,
                    "out_for_delivery",
                ));
            } else if self
                .ui
                .button(cx, ids!(ops_advance_delivered))
                .clicked(actions)
            {
                ops_msg = Some(self.books_advance_tracking(
                    &awb,
                    &bulk_awbs,
                    &note,
                    "delivered",
                ));
            } else if self.ui.button(cx, ids!(ops_label)).clicked(actions) {
                ops_msg = Some((|| {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    courier_ops_view::print_label(store, &awb)
                })());
            } else if self.ui.button(cx, ids!(ops_ndr)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::FailedDelivery {
                        awb: awb.clone(),
                        note: note.clone(),
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_reattempt)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::Reattempt { awb: awb.clone() })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
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
                    let msg = st.books_op(BooksOp::SettleRiderCod {
                        rider: rider.clone(),
                        handed,
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
                    Ok(msg)
                })());
            } else if self
                .ui
                .button(cx, ids!(ops_dispatch_fleet))
                .clicked(actions)
            {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::DispatchFleet {
                        rider: rider.clone(),
                        note: note.clone(),
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
                    Ok(msg)
                })());
            } else if self
                .ui
                .button(cx, ids!(ops_handover_carrier))
                .clicked(actions)
            {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::CarrierHandover {
                        carrier: carrier.clone(),
                        note: note.clone(),
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_cod_collect)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::CollectCod { awb: awb.clone() })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_cod_remit)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::RemitCod {
                        party: party.clone(),
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_add_rider)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::AddRider {
                        name: rider.clone(),
                        phone: rider_phone.clone(),
                        vehicle: rider_vehicle.clone(),
                    })?;
                    st.reports_nav = ReportsNav::Riders;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
                    Ok(msg)
                })());
            } else if self.ui.button(cx, ids!(ops_period_bill)).clicked(actions) {
                ops_msg = Some((|| {
                    let mut st = app_state().lock().unwrap();
                    let msg = st.books_op(BooksOp::PeriodBill {
                        party_name: party.clone(),
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.refresh_reports_facet()?;
                    }
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
                let currency = st.locale.currency.clone();
                let minor = locale::parse_major_input(&amount_text, &currency)?;
                let tender = st.expense_tender;
                let tender2 = st.expense_tender2;
                let parts = Self::expense_parts(minor, &part2_text, &currency, tender, tender2)?;
                let msg = st.books_op(BooksOp::LogExpense {
                    amount_paise: minor,
                    note,
                    category: "misc".into(),
                    parts,
                })?;
                let status = match msg.split_once(books_op::EXPENSE_ID_MARK) {
                    Some((status, id)) => {
                        st.selected_expense_id = Some(id.to_string());
                        status.to_string()
                    }
                    None => msg,
                };
                if st.device_role != DeviceRole::Client {
                    st.day_close_text = st
                        .store
                        .as_ref()
                        .ok_or_else(|| "No store".to_string())?
                        .books_report_text("today")?;
                }
                st.nav_back.clear();
                st.nav_forward.clear();
                st.screen = Screen::Reports;
                st.status = status;
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
                    let currency = st.locale.currency.clone();
                    let minor = locale::parse_major_input(&amount_text, &currency)?;
                    let msg = st.books_op(BooksOp::PostMoney {
                        kind: kind.as_str().to_string(),
                        amount_paise: minor,
                        note,
                    })?;
                    if st.device_role != DeviceRole::Client {
                        st.day_close_text = st
                            .store
                            .as_ref()
                            .ok_or_else(|| "No store".to_string())?
                            .books_report_text("today")?;
                    }
                    st.nav_back.clear();
                    st.nav_forward.clear();
                    st.screen = Screen::Reports;
                    st.status = msg;
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
                    let mut st = app_state().lock().unwrap();
                    let number: usize = num_text
                        .trim()
                        .parse()
                        .map_err(|_| "Type the uncleared # from the list".to_string())?;
                    st.books_op(BooksOp::SetMovementCleared { number, cleared })
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

    /// Out for delivery / delivered, one AWB or a pasted list. POD bytes go
    /// with a single AWB; the shop attaches them.
    fn books_advance_tracking(
        &mut self,
        awb: &str,
        bulk_awbs: &[String],
        note: &str,
        status: &str,
    ) -> Result<String, String> {
        let mut st = app_state().lock().unwrap();
        let use_bulk = bulk_awbs.len() > 1 || (bulk_awbs.len() == 1 && awb.is_empty());
        let (pod_bytes, pod_filename) = if use_bulk {
            (Vec::new(), String::new())
        } else {
            match st.pending_ops_pod.take() {
                Some(path) => {
                    let bytes = std::fs::read(&path)
                        .map_err(|e| format!("Could not read the POD photo: {e}"))?;
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("pod.webp")
                        .to_string();
                    (bytes, filename)
                }
                None => (Vec::new(), String::new()),
            }
        };
        let awbs = if use_bulk {
            bulk_awbs.to_vec()
        } else {
            vec![awb.to_string()]
        };
        let msg = st.books_op(BooksOp::AdvanceTracking {
            awbs,
            bulk: use_bulk,
            status: status.to_string(),
            note: note.to_string(),
            pod_filename,
            pod_bytes,
        })?;
        if st.device_role != DeviceRole::Client {
            st.refresh_reports_facet()?;
        }
        Ok(msg)
    }
}

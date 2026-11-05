impl App {
    fn handle_actions_ops(&mut self, cx: &mut Cx, actions: &Actions) {
        // Ops board (its own module), then the COD and Riders desks.
        self.handle_ops_board_actions(cx, actions);
        self.handle_cod_riders_desks(cx, actions);
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

    /// COD desk (collect a parcel's COD, pay a consignor out, period bill)
    /// and Riders desk (settle what a rider hands in, add a rider).
    fn handle_cod_riders_desks(&mut self, cx: &mut Cx, actions: &Actions) {
        let collect = self.ui.button(cx, ids!(cod_collect)).clicked(actions)
            || self.ui.text_input(cx, ids!(cod_awb_input)).returned(actions).is_some();
        let remit = self.ui.button(cx, ids!(ops_cod_remit)).clicked(actions);
        let bill = self.ui.button(cx, ids!(ops_period_bill)).clicked(actions);
        let settle = self.ui.button(cx, ids!(ops_settle_rider_cod)).clicked(actions);
        let add = self.ui.button(cx, ids!(ops_add_rider)).clicked(actions);
        if !(collect || remit || bill || settle || add) {
            return;
        }
        let text = |s: &mut Self, cx: &mut Cx, id: &[LiveId]| s.ui.text_input(cx, id).text();
        let cod_awbs = store::Store::parse_awb_list(&text(self, cx, ids!(cod_awb_input)));
        let party = text(self, cx, ids!(ops_party_input));
        let settle_rider = text(self, cx, ids!(settle_rider_input));
        let handed = text(self, cx, ids!(ops_rider_cod_input));
        let new_rider = text(self, cx, ids!(new_rider_input));
        let phone = text(self, cx, ids!(ops_rider_phone));
        let vehicle = text(self, cx, ids!(ops_rider_vehicle));
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let msg = if collect {
                if cod_awbs.is_empty() {
                    return Err(st.t("awb_scan_ph"));
                }
                let mut done = Vec::new();
                for awb in cod_awbs {
                    done.push(st.books_op(BooksOp::CollectCod { awb })?);
                }
                done.join("\n")
            } else if remit {
                st.books_op(BooksOp::RemitCod { party })?
            } else if bill {
                st.books_op(BooksOp::PeriodBill { party_name: party })?
            } else if settle {
                st.books_op(BooksOp::SettleRiderCod {
                    rider: settle_rider,
                    handed,
                })?
            } else {
                st.books_op(BooksOp::AddRider {
                    name: new_rider,
                    phone,
                    vehicle,
                })?
            };
            if st.device_role != DeviceRole::Client {
                st.refresh_reports_facet()?;
            }
            Ok(msg)
        })();
        if result.is_ok() {
            {
                let clear: &[&[LiveId]] = if collect {
                    &[ids!(cod_awb_input)]
                } else if settle {
                    &[ids!(ops_rider_cod_input)]
                } else if add {
                    &[ids!(new_rider_input), ids!(ops_rider_phone), ids!(ops_rider_vehicle)]
                } else {
                    &[]
                };
                for id in clear {
                    self.ui.text_input(cx, id).set_text(cx, "");
                }
            }
        }
        self.set_status_as(cx, result);
        self.sync_chrome(cx);
    }
}

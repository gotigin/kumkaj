/// Ops board lanes in screen order: (idle face, on face).
const OPS_LANE_BTNS: [(&[LiveId], &[LiveId]); 5] = [
    (ids!(lane_0), ids!(lane_0_on)),
    (ids!(lane_1), ids!(lane_1_on)),
    (ids!(lane_2), ids!(lane_2_on)),
    (ids!(lane_3), ids!(lane_3_on)),
    (ids!(lane_4), ids!(lane_4_on)),
];

/// Failed-delivery reason chips → the code the books keep.
const OPS_REASON_BTNS: [(&[LiveId], manifest::NdrReason); 5] = [
    (ids!(ops_ndr_door), manifest::NdrReason::DoorLocked),
    (ids!(ops_ndr_refused), manifest::NdrReason::Refused),
    (ids!(ops_ndr_address), manifest::NdrReason::AddressWrong),
    (ids!(ops_ndr_unavailable), manifest::NdrReason::CustomerUnavailable),
    (ids!(ops_ndr_other), manifest::NdrReason::Other),
];

/// What a lane lets the desk do with the picked parcels (the store still
/// refuses any move that does not fit — this only keeps the face honest).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct OpsActs {
    dispatch: bool,
    carrier_update: bool,
    deliver: bool,
    fail: bool,
    failed: bool,
    reasons: bool,
}

fn ops_lane_acts(lane: manifest::OpsLane) -> OpsActs {
    use manifest::OpsLane;
    match lane {
        OpsLane::ToDispatch => OpsActs { dispatch: true, ..Default::default() },
        OpsLane::WithCarrier => OpsActs {
            carrier_update: true,
            deliver: true,
            fail: true,
            reasons: true,
            ..Default::default()
        },
        OpsLane::OutForDelivery => OpsActs {
            deliver: true,
            fail: true,
            reasons: true,
            ..Default::default()
        },
        OpsLane::Failed => OpsActs { failed: true, reasons: true, ..Default::default() },
        OpsLane::Done => OpsActs::default(),
    }
}

impl App {
    fn handle_ops_board_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (i, (off, on)) in OPS_LANE_BTNS.iter().enumerate() {
            if self.ui.button(cx, off).clicked(actions) || self.ui.button(cx, on).clicked(actions) {
                self.ops_switch_lane(cx, manifest::OpsLane::ALL[i]);
                return;
            }
        }

        // A tap anywhere on a card picks it, a second tap lets it go.
        let list = self.ui.widget(cx, ids!(ops_list)).portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            if item
                .view(cx, ids!(card))
                .finger_up(actions)
                .is_some_and(finger_up_is_tap)
            {
                let mut st = app_state().lock().unwrap();
                if let Some(id) = st.ops_rows.get(item_id).map(|r| r.sale_id.clone()) {
                    if let Some(at) = st.ops_picked.iter().position(|p| *p == id) {
                        st.ops_picked.remove(at);
                    } else {
                        st.ops_picked.push(id);
                    }
                    let _ = st.refresh_ops_board();
                }
                drop(st);
                self.sync_chrome(cx);
            }
        }

        let runs = self.ui.widget(cx, ids!(ops_runs)).portal_list(cx, ids!(list));
        for (item_id, item) in runs.items_with_actions(actions) {
            if item.button(cx, ids!(print)).clicked(actions) {
                let result = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    let m = st
                        .ops_runs
                        .get(item_id)
                        .cloned()
                        .ok_or_else(|| "That run is gone".to_string())?;
                    courier_ops_view::print_manifest_named(store, &m)
                })();
                self.set_status_as(cx, result);
                self.sync_chrome(cx);
            }
        }

        // Scan or type one AWB (a wedge scanner ends with Enter) or paste
        // many: each is picked in its lane.
        let enter = self.ui.text_input(cx, ids!(ops_awb_input)).returned(actions).is_some();
        if enter || self.ui.button(cx, ids!(ops_find_awb)).clicked(actions) {
            let typed = self.ui.text_input(cx, ids!(ops_awb_input)).text();
            let result = self.ops_pick_typed(&typed);
            if result.is_ok() {
                self.ui.text_input(cx, ids!(ops_awb_input)).set_text(cx, "");
            }
            self.set_status_as(cx, result);
            self.sync_chrome(cx);
            // Keep the cursor in the scan field for the next parcel.
            self.ui.text_input(cx, ids!(ops_awb_input)).set_key_focus(cx);
        }

        if self.ui.button(cx, ids!(ops_pick_all)).clicked(actions) {
            let mut st = app_state().lock().unwrap();
            st.ops_picked = st.ops_rows.iter().map(|r| r.sale_id.clone()).collect();
            let _ = st.refresh_ops_board();
            drop(st);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(ops_pick_clear)).clicked(actions) {
            let mut st = app_state().lock().unwrap();
            st.ops_picked.clear();
            let _ = st.refresh_ops_board();
            drop(st);
            self.sync_chrome(cx);
        }
        if self.ui.button(cx, ids!(ops_pod_pick)).clicked(actions) {
            self.attach_pick_ops_pod(cx);
        }
        for (btn, reason) in OPS_REASON_BTNS {
            if self.ui.button(cx, btn).clicked(actions) {
                let label = {
                    let mut st = app_state().lock().unwrap();
                    st.ops_reason = Some(reason);
                    st.t(reason.locale_key())
                };
                self.ui.text_input(cx, ids!(ops_note_input)).set_text(cx, &label);
                self.sync_chrome(cx);
            }
        }

        let clicked = |id: &[LiveId]| self.ui.button(cx, id).clicked(actions);
        let op = if clicked(ids!(ops_handover_carrier)) {
            Some(OpsAct::Bag)
        } else if clicked(ids!(ops_dispatch_fleet)) {
            Some(OpsAct::Run)
        } else if clicked(ids!(ops_advance_ofd)) {
            Some(OpsAct::OutForDelivery)
        } else if clicked(ids!(ops_advance_delivered)) {
            Some(OpsAct::Delivered)
        } else if clicked(ids!(ops_ndr)) {
            Some(OpsAct::Failed)
        } else if clicked(ids!(ops_reattempt)) {
            Some(OpsAct::OutAgain)
        } else if clicked(ids!(ops_return)) {
            Some(OpsAct::Return)
        } else if clicked(ids!(ops_label)) {
            Some(OpsAct::Label)
        } else if clicked(ids!(ops_cod_collect)) {
            Some(OpsAct::CollectCod)
        } else {
            None
        };
        if let Some(op) = op {
            let carrier = self.ui.text_input(cx, ids!(ops_carrier_input)).text();
            let rider = self.ui.text_input(cx, ids!(ops_rider_input)).text();
            let note = self.ui.text_input(cx, ids!(ops_note_input)).text();
            let result = self.ops_run_act(op, &carrier, &rider, &note);
            if result.is_ok() && !matches!(op, OpsAct::Label) {
                self.ui.text_input(cx, ids!(ops_note_input)).set_text(cx, "");
                app_state().lock().unwrap().ops_reason = None;
            }
            self.set_status_as(cx, result);
            self.sync_chrome(cx);
        }
    }

    /// Show another lane; picks belong to a lane, so they start empty.
    fn ops_switch_lane(&mut self, cx: &mut Cx, lane: manifest::OpsLane) {
        {
            let mut st = app_state().lock().unwrap();
            if st.ops_lane != lane {
                st.ops_picked.clear();
                st.ops_reason = None;
            }
            st.ops_lane = lane;
            if let Err(e) = st.refresh_ops_board() {
                st.status = e;
            }
        }
        self.ui.text_input(cx, ids!(ops_note_input)).set_text(cx, "");
        self.sync_chrome(cx);
    }

    /// Pick every AWB in `typed` (scan, type or paste). The board moves to
    /// the lane of the first one found; AWBs in other lanes are named, not
    /// picked — one batch never mixes lanes.
    fn ops_pick_typed(&mut self, typed: &str) -> Result<String, String> {
        let awbs = store::Store::parse_awb_list(typed);
        if awbs.is_empty() {
            return Err(app_state().lock().unwrap().t("awb_scan_ph"));
        }
        let mut st = app_state().lock().unwrap();
        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
        let mut found: Vec<(Sale, manifest::OpsLane)> = Vec::new();
        let mut missing: Vec<String> = Vec::new();
        for awb in &awbs {
            match courier_ops_view::find_sale_for_scan(store, awb) {
                Ok(sale) if sale.status == "completed" => {
                    let lane = manifest::TrackingStatus::parse(&sale.tracking_status).lane();
                    found.push((sale, lane));
                }
                Ok(sale) => missing.push(format!("{} was {}", sale.awb_no, sale.status)),
                Err(e) => missing.push(e),
            }
        }
        let Some(lane) = found.first().map(|(_, l)| *l) else {
            return Err(missing.join("\n"));
        };
        if st.ops_lane != lane {
            st.ops_lane = lane;
            st.ops_picked.clear();
            st.ops_reason = None;
        }
        let mut picked = 0usize;
        let mut elsewhere: Vec<String> = Vec::new();
        for (sale, l) in &found {
            if *l != lane {
                elsewhere.push(format!("{} · {}", sale.awb_no, st.t(l.label_key())));
                continue;
            }
            if !st.ops_picked.contains(&sale.id) {
                st.ops_picked.push(sale.id.clone());
            }
            picked += 1;
        }
        st.refresh_ops_board()?;
        let mut msg = st
            .t("picked_fmt")
            .replace("{n}", &st.ops_picked.len().to_string());
        if picked == 1 && found.len() == 1 {
            msg = format!("{} · {msg}", found[0].0.awb_no);
        }
        for line in elsewhere.iter().chain(missing.iter()).take(4) {
            msg.push('\n');
            msg.push_str(line);
        }
        Ok(msg)
    }

    /// Run one board action on the picked parcels, through the books (the
    /// Server checks the person and the parcel's state on every till).
    fn ops_run_act(
        &mut self,
        op: OpsAct,
        carrier: &str,
        rider: &str,
        note: &str,
    ) -> Result<String, String> {
        let mut st = app_state().lock().unwrap();
        let awbs: Vec<String> = st
            .ops_picked
            .iter()
            .filter_map(|id| st.ops_rows.iter().find(|r| r.sale_id == *id))
            .map(|r| r.awb_no.clone())
            .collect();
        if awbs.is_empty() {
            return Err(st.t("ops_pick_first"));
        }
        // A reason chip sends its code; a note the person typed goes as typed.
        let reason = match st.ops_reason {
            Some(r) if note.trim() == st.t(r.locale_key()) => r.as_str().to_string(),
            _ => note.trim().to_string(),
        };
        let msg = match op {
            OpsAct::Bag => st.books_op(BooksOp::CarrierHandover {
                carrier: carrier.to_string(),
                note: note.to_string(),
                awbs,
            })?,
            OpsAct::Run => st.books_op(BooksOp::DispatchFleet {
                rider: rider.to_string(),
                note: note.to_string(),
                awbs,
            })?,
            OpsAct::OutForDelivery | OpsAct::Delivered => {
                let status = if op == OpsAct::Delivered {
                    "delivered"
                } else {
                    "out_for_delivery"
                };
                // One parcel may carry its POD photo; a batch never does.
                let single = awbs.len() == 1;
                let (pod_bytes, pod_filename) = match (single, st.pending_ops_pod.take()) {
                    (true, Some(path)) => {
                        let bytes = std::fs::read(&path)
                            .map_err(|e| format!("Could not read the POD photo: {e}"))?;
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("pod.webp")
                            .to_string();
                        (bytes, name)
                    }
                    (_, other) => {
                        st.pending_ops_pod = other;
                        (Vec::new(), String::new())
                    }
                };
                st.books_op(BooksOp::AdvanceTracking {
                    awbs,
                    bulk: !single,
                    status: status.to_string(),
                    note: note.trim().to_string(),
                    pod_filename,
                    pod_bytes,
                })?
            }
            OpsAct::Failed => st.books_op(BooksOp::FailedDelivery {
                awb: String::new(),
                note: reason,
                awbs,
            })?,
            OpsAct::OutAgain => st.books_op(BooksOp::Reattempt {
                awb: String::new(),
                awbs,
            })?,
            OpsAct::Return => st.books_op(BooksOp::ReturnToSender { awbs, reason })?,
            OpsAct::CollectCod => {
                let mut done = Vec::new();
                for awb in awbs {
                    done.push(st.books_op(BooksOp::CollectCod { awb })?);
                }
                done.join("\n")
            }
            OpsAct::Label => {
                // Labels print on this device's own label printer.
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let mut last = String::new();
                for awb in &awbs {
                    last = courier_ops_view::print_label(store, awb)?;
                }
                if awbs.len() > 1 {
                    format!("{} · {last}", locale::parcels(&st.locale.language, awbs.len() as i64))
                } else {
                    last
                }
            }
        };
        if st.device_role != DeviceRole::Client {
            st.refresh_ops_board()?;
        }
        Ok(msg)
    }

    /// Paint the board: lane faces with counts, the action card for what is
    /// picked (only this lane's moves), the empty line, history, POD state.
    fn paint_ops_board(&mut self, cx: &mut Cx) {
        let (lane, counts, picked, rows_len, cod_picked, history, pod, lang) = {
            let st = app_state().lock().unwrap();
            let cod_picked = st
                .ops_rows
                .iter()
                .filter(|r| st.ops_picked.contains(&r.sale_id))
                .map(|r| r.cod_paise)
                .sum::<i64>();
            (
                st.ops_lane,
                st.ops_counts,
                st.ops_picked.len(),
                st.ops_rows.len(),
                cod_picked,
                st.ops_history.clone(),
                st.pending_ops_pod
                    .as_ref()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())),
                st.locale.language.clone(),
            )
        };
        let t = |k: &str| locale::t(&lang, k);
        for (i, (off, on)) in OPS_LANE_BTNS.iter().enumerate() {
            let l = manifest::OpsLane::ALL[i];
            let text = format!("{} · {}", t(l.label_key()), counts[i]);
            let active = l == lane;
            self.ui.button(cx, off).set_text(cx, &text);
            self.ui.button(cx, on).set_text(cx, &text);
            self.ui.button(cx, off).set_visible(cx, !active);
            self.ui.button(cx, on).set_visible(cx, active);
        }
        let acts = ops_lane_acts(lane);
        let mut head = t("picked_fmt").replace("{n}", &picked.to_string());
        if cod_picked > 0 {
            let st = app_state().lock().unwrap();
            head.push_str(&format!(" · COD {}", st.money(Money(cod_picked))));
        }
        self.ui.label(cx, ids!(ops_picked_label)).set_text(cx, &head);
        // Visibility follows the lane only (never what is picked), so the
        // card keeps its height while the person taps parcels.
        self.ui.view(cx, ids!(ops_act_dispatch)).set_visible(cx, acts.dispatch);
        self.ui.view(cx, ids!(ops_act_carrier)).set_visible(cx, acts.carrier_update);
        self.ui.view(cx, ids!(ops_act_deliver)).set_visible(cx, acts.deliver);
        self.ui.view(cx, ids!(ops_act_reasons)).set_visible(cx, acts.reasons);
        self.ui.view(cx, ids!(ops_act_fail)).set_visible(cx, acts.fail);
        self.ui.view(cx, ids!(ops_act_failed)).set_visible(cx, acts.failed);
        self.ui
            .button(cx, ids!(ops_cod_collect))
            .set_visible(cx, lane == manifest::OpsLane::Done);
        self.ui.label(cx, ids!(ops_pod_status)).set_text(cx, &pod.unwrap_or_default());
        let empty = if rows_len == 0 { t("lane_empty") } else { String::new() };
        self.ui.widget(cx, ids!(ops_lane_empty)).set_visible(cx, rows_len == 0);
        self.ui.widget(cx, ids!(ops_lane_empty)).set_text(cx, &plain_display_text(&empty));
        let has_history = !history.is_empty();
        self.set_report_rows(cx, ids!(ops_history), history);
        self.ui.widget(cx, ids!(ops_history)).set_visible(cx, has_history);
        let has_runs = !app_state().lock().unwrap().ops_runs.is_empty();
        self.ui.widget(cx, ids!(h_ops_runs)).set_visible(cx, has_runs);
        self.ui.widget(cx, ids!(ops_list)).redraw(cx);
        self.ui.widget(cx, ids!(ops_runs)).redraw(cx);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OpsAct {
    Bag,
    Run,
    OutForDelivery,
    Delivered,
    Failed,
    OutAgain,
    Return,
    Label,
    CollectCod,
}

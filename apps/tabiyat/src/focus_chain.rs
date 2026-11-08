// Tab / Shift+Tab among multi-field forms (pair, lock, onboard, pay, settings).
// Makepad `NavControl` walks every drawn `Button` before the app sees Tab, so we
// remember the last focused form field (`form_tab_anchor`) and move within an
// explicit chain — overwriting chrome focus when the user was typing in a form.

/// True when this field is on-screen and laid out (skips hidden parent panels).
fn field_tab_eligible(cx: &Cx, field: &TextInputRef) -> bool {
    if !field.visible() {
        return false;
    }
    let area = field.area();
    if area.is_empty() {
        return false;
    }
    let r = area.rect(cx);
    r.size.x >= 1.0 && r.size.y >= 1.0
}

fn field_live_id(path: &[LiveId]) -> Option<LiveId> {
    path.last().copied()
}

/// Move focus to next/prev eligible field in `chain`. Uses current focus when a
/// chain member still has it; otherwise `form_tab_anchor` (survives NavControl
/// stealing Tab onto a Button). Returns true when focus was moved.
fn tab_focus_chain(
    cx: &mut Cx,
    ui: &WidgetRef,
    chain: &[&[LiveId]],
    reverse: bool,
    anchor: Option<LiveId>,
) -> bool {
    let mut eligible: Vec<(usize, LiveId)> = Vec::new();
    for (i, path) in chain.iter().enumerate() {
        let field = ui.text_input(cx, *path);
        if !field_tab_eligible(cx, &field) {
            continue;
        }
        let Some(id) = field_live_id(path) else {
            continue;
        };
        eligible.push((i, id));
    }
    if eligible.is_empty() {
        return false;
    }

    let focused_pos = eligible
        .iter()
        .position(|(i, _)| cx.has_key_focus(ui.text_input(cx, chain[*i]).area()));
    let from_pos = focused_pos.or_else(|| {
        let a = anchor?;
        eligible.iter().position(|(_, id)| *id == a)
    });

    let next_pos = match from_pos {
        Some(pos) if reverse => {
            if pos == 0 {
                eligible.len() - 1
            } else {
                pos - 1
            }
        }
        Some(pos) => (pos + 1) % eligible.len(),
        None if reverse => eligible.len() - 1,
        None => 0,
    };

    // Only steal focus when we know the user was in this chain (or on a
    // dedicated form screen the caller already gated).
    if from_pos.is_none() {
        return false;
    }

    let path = chain[eligible[next_pos].0];
    ui.text_input(cx, path).take_key_focus(cx);
    true
}

/// Soft-keyboard Next / Enter on a non-last chain field → next field.
/// Returns true when focus advanced (caller should skip submit).
fn advance_focus_after_returned(
    cx: &mut Cx,
    ui: &WidgetRef,
    actions: &Actions,
    chain: &[&[LiveId]],
) -> bool {
    let mut eligible: Vec<usize> = Vec::new();
    for (i, path) in chain.iter().enumerate() {
        if field_tab_eligible(cx, &ui.text_input(cx, *path)) {
            eligible.push(i);
        }
    }
    if eligible.len() < 2 {
        return false;
    }
    for (pos, &idx) in eligible.iter().enumerate() {
        if ui.text_input(cx, chain[idx]).returned(actions).is_some() {
            if pos + 1 >= eligible.len() {
                return false; // last field → let submit handlers run
            }
            let next = chain[eligible[pos + 1]];
            ui.text_input(cx, next).take_key_focus(cx);
            return true;
        }
    }
    false
}

impl App {
    /// Remember which form field holds key focus (survives NavControl Tab theft).
    fn refresh_form_tab_anchor(&mut self, cx: &mut Cx) {
        let paths = [
            ids!(client_host_input),
            ids!(client_code_input),
            ids!(client_name_input),
            ids!(lock_pin_input),
            ids!(lock_login_id),
            ids!(lock_login_password),
            ids!(onboard_venue_input),
            ids!(onboard_org_input),
            ids!(onboard_item_name),
            ids!(onboard_item_price),
            ids!(onboard_cust_name),
            ids!(onboard_cust_bal),
            ids!(venue_title_input),
            ids!(org_name_input),
            ids!(business_tax_id_input),
            ids!(service_bps_input),
            ids!(discount_input),
            ids!(discount_reason_input),
            ids!(discount_pin_input),
            ids!(mix_cash_input),
            ids!(mix_upi_input),
            ids!(mix_card_input),
            ids!(mix_credit_input),
            ids!(tendered_input),
            ids!(tip_input),
        ];
        for path in paths {
            let field = self.ui.text_input(cx, path);
            let area = field.area();
            if !area.is_empty() && cx.has_key_focus(area) {
                if let Some(id) = field_live_id(path) {
                    app_state().lock().unwrap().form_tab_anchor = Some(id);
                    return;
                }
            }
        }
        // Focus left the form (button / empty) — drop anchor so Tab does not yank back.
        app_state().lock().unwrap().form_tab_anchor = None;
    }

    /// Tab / Shift+Tab within the active form chain. Returns true if handled.
    fn handle_form_field_tab(&mut self, cx: &mut Cx, reverse: bool) -> bool {
        let (screen, panel, pay_choice, discount_open, onboard_step, anchor) = {
            let st = app_state().lock().unwrap();
            (
                st.screen,
                st.settings_panel,
                st.pay_choice,
                st.discount_panel_open,
                st.onboard_step,
                st.form_tab_anchor,
            )
        };

        let moved = match screen {
            Screen::ClientPair => tab_focus_chain(
                cx,
                &self.ui,
                &[
                    ids!(client_host_input),
                    ids!(client_code_input),
                    ids!(client_name_input),
                ],
                reverse,
                anchor,
            ),
            Screen::Lock => tab_focus_chain(
                cx,
                &self.ui,
                &[
                    ids!(lock_pin_input),
                    ids!(lock_login_id),
                    ids!(lock_login_password),
                ],
                reverse,
                anchor,
            ),
            Screen::Onboard => {
                if onboard_step == 2 {
                    tab_focus_chain(
                        cx,
                        &self.ui,
                        &[
                            ids!(onboard_item_name),
                            ids!(onboard_item_price),
                            ids!(onboard_cust_name),
                            ids!(onboard_cust_bal),
                        ],
                        reverse,
                        anchor,
                    )
                } else {
                    tab_focus_chain(
                        cx,
                        &self.ui,
                        &[ids!(onboard_venue_input), ids!(onboard_org_input)],
                        reverse,
                        anchor,
                    )
                }
            }
            Screen::Settings => match panel {
                Some("brand") => tab_focus_chain(
                    cx,
                    &self.ui,
                    &[ids!(venue_title_input), ids!(org_name_input)],
                    reverse,
                    anchor,
                ),
                Some("country") => tab_focus_chain(
                    cx,
                    &self.ui,
                    &[ids!(business_tax_id_input), ids!(service_bps_input)],
                    reverse,
                    anchor,
                ),
                _ => false,
            },
            Screen::Pos => {
                let mut chain: Vec<&[LiveId]> = Vec::new();
                if discount_open {
                    chain.extend_from_slice(&[
                        ids!(discount_input),
                        ids!(discount_reason_input),
                        ids!(discount_pin_input),
                    ]);
                }
                if pay_choice == PayChoice::Split {
                    chain.extend_from_slice(&[
                        ids!(mix_cash_input),
                        ids!(mix_upi_input),
                        ids!(mix_card_input),
                        ids!(mix_credit_input),
                    ]);
                }
                if matches!(pay_choice, PayChoice::Cash | PayChoice::Split) {
                    chain.extend_from_slice(&[ids!(tendered_input), ids!(tip_input)]);
                }
                if chain.is_empty() {
                    false
                } else {
                    tab_focus_chain(cx, &self.ui, &chain, reverse, anchor)
                }
            }
            _ => false,
        };

        if moved {
            self.refresh_form_tab_anchor(cx);
        }
        moved
    }

    /// Enter / IME Next: advance within chain; false = let existing submit run.
    fn handle_form_field_returned(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        let (screen, panel, pay_choice, discount_open, onboard_step) = {
            let st = app_state().lock().unwrap();
            (
                st.screen,
                st.settings_panel,
                st.pay_choice,
                st.discount_panel_open,
                st.onboard_step,
            )
        };

        let advanced = match screen {
            Screen::ClientPair => advance_focus_after_returned(
                cx,
                &self.ui,
                actions,
                &[
                    ids!(client_host_input),
                    ids!(client_code_input),
                    ids!(client_name_input),
                ],
            ),
            Screen::Lock => {
                // PIN Done still unlocks (handled elsewhere). Login id → password.
                if self
                    .ui
                    .text_input(cx, ids!(lock_login_id))
                    .returned(actions)
                    .is_some()
                {
                    if field_tab_eligible(cx, &self.ui.text_input(cx, ids!(lock_login_password))) {
                        self.ui
                            .text_input(cx, ids!(lock_login_password))
                            .take_key_focus(cx);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Screen::Onboard => {
                if onboard_step == 2 {
                    advance_focus_after_returned(
                        cx,
                        &self.ui,
                        actions,
                        &[
                            ids!(onboard_item_name),
                            ids!(onboard_item_price),
                            ids!(onboard_cust_name),
                            ids!(onboard_cust_bal),
                        ],
                    )
                } else {
                    advance_focus_after_returned(
                        cx,
                        &self.ui,
                        actions,
                        &[ids!(onboard_venue_input), ids!(onboard_org_input)],
                    )
                }
            }
            Screen::Settings => match panel {
                Some("brand") => advance_focus_after_returned(
                    cx,
                    &self.ui,
                    actions,
                    &[ids!(venue_title_input), ids!(org_name_input)],
                ),
                Some("country") => advance_focus_after_returned(
                    cx,
                    &self.ui,
                    actions,
                    &[ids!(business_tax_id_input), ids!(service_bps_input)],
                ),
                _ => false,
            },
            Screen::Pos => {
                let mut chain: Vec<&[LiveId]> = Vec::new();
                if discount_open {
                    chain.extend_from_slice(&[
                        ids!(discount_input),
                        ids!(discount_reason_input),
                        ids!(discount_pin_input),
                    ]);
                }
                if pay_choice == PayChoice::Split {
                    chain.extend_from_slice(&[
                        ids!(mix_cash_input),
                        ids!(mix_upi_input),
                        ids!(mix_card_input),
                        ids!(mix_credit_input),
                    ]);
                }
                if matches!(pay_choice, PayChoice::Cash | PayChoice::Split) {
                    chain.extend_from_slice(&[ids!(tendered_input), ids!(tip_input)]);
                }
                if chain.is_empty() {
                    false
                } else {
                    advance_focus_after_returned(cx, &self.ui, actions, &chain)
                }
            }
            _ => false,
        };

        if advanced {
            self.refresh_form_tab_anchor(cx);
        }
        advanced
    }
}

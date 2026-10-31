#[derive(Script, ScriptHook, Widget)]
struct CustomerList {
    #[deref]
    view: View,
}

impl Widget for CustomerList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (customers, currency, fiscal, selected_id, lang) = {
            let st = app_state().lock().unwrap();
            (
                st.customers.clone(),
                st.locale.currency.clone(),
                st.fiscal.clone(),
                st.selected_customer_id.clone(),
                st.locale.language.clone(),
            )
        };
        // Full content height + page scroll (SaleList rule). A 3-row capped
        // nested PortalList kept an inner scroll offset → blank gap under
        // "Saved parties" with rows stuck at the bottom of the viewport.
        let row = *customer_row_height().lock().unwrap();
        let n = customers.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, n);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(c) = customers.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let is_selected = selected_id.as_deref() == Some(c.id.as_str());
                    let mark = if is_selected { "✓ " } else { "" };
                    let role = c.kind_label.trim();
                    let name_bit = if role.is_empty() {
                        c.name.clone()
                    } else {
                        format!("{} · {}", c.name, role)
                    };
                    let text = if c.balance_paise > 0 {
                        format!(
                            "{}{} · {} {}",
                            mark,
                            name_bit,
                            locale::t(&lang, "owes_word"),
                            Money(c.balance_paise).format_localized(&currency, &fiscal)
                        )
                    } else if c.balance_paise < 0 {
                        format!(
                            "{}{} · {} {}",
                            mark,
                            name_bit,
                            locale::t(&lang, "advance"),
                            Money(-c.balance_paise).format_localized(&currency, &fiscal)
                        )
                    } else {
                        format!("{}{}", mark, name_bit)
                    };
                    // Khaynjo practise: WrapLabel via widget() + plain_display_text
                    // (Markdown escapes). `.label()` on WrapLabel drew blank rows.
                    item.widget(cx, ids!(label))
                        .set_text(cx, &plain_display_text(&text));
                    let phone = c.phone.trim();
                    let addr = c.address.trim();
                    let sub = match (phone.is_empty(), addr.is_empty()) {
                        (true, true) => String::new(),
                        (false, true) => phone.to_string(),
                        (true, false) => addr.to_string(),
                        (false, false) => format!("{phone} · {addr}"),
                    };
                    item.widget(cx, ids!(sublabel))
                        .set_text(cx, &plain_display_text(&sub));
                    let dir = party_images_dir();
                    let path = if c.image_key.trim().is_empty() {
                        std::path::PathBuf::new()
                    } else {
                        image_key_path(Some(c.image_key.as_str()), &dir)
                    };
                    apply_thumb_avatar(cx, &item, &c.name, &path);
                    let pick_text = if is_selected {
                        locale::t(&lang, "selected_word")
                    } else {
                        locale::t(&lang, "pick")
                    };
                    item.button(cx, ids!(pick)).set_text(cx, &pick_text);
                    item.draw_all_unscoped(cx);
                }
                if n > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / n as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, customer_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct LanguageList {
    #[deref]
    view: View,
}

impl Widget for LanguageList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let selected = app_state().lock().unwrap().locale.language.clone();
        // Cap visible height so Settings still scrolls the page; list scrolls inside.
        let walk = Walk {
            height: Size::Fixed(220.0),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, LANGUAGES.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(lang) = LANGUAGES.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let mark = if lang.code == selected { "✓ " } else { "" };
                    item.button(cx, ids!(pick)).set_text(
                        cx,
                        &format!(
                            "{mark}{} · {} · {}",
                            lang.native_name, lang.name_en, lang.region
                        ),
                    );
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Pointer over this capped list = inner owns the wheel; page must not move.
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct ServerList {
    #[deref]
    view: View,
}

impl Widget for ServerList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (servers, open_label) = {
            let st = app_state().lock().unwrap();
            (st.local_servers.clone(), st.t("open_server"))
        };
        // Viewport shows ~3 LocaleRows; pitch is measured (old 72 clipped Open).
        let row = *locale_row_height().lock().unwrap();
        let n = servers.len().max(1);
        let want = list_content_height(n.min(3).max(1), row, 0.0).max(row + 16.0);
        let walk = Walk {
            height: Size::Fixed(want.min(280.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, servers.len());
                let mut drew = 0usize;
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(server) = servers.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let short = server.id.chars().take(8).collect::<String>();
                    item.widget(cx, ids!(label)).set_text(
                        cx,
                        &plain_display_text(&format!("{} · {short}", server.name)),
                    );
                    item.button(cx, ids!(pick)).set_text(cx, &open_label);
                    item.draw_all_unscoped(cx);
                    drew += 1;
                }
                if drew > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / drew as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, locale_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

/// Devices paired to this Server. Forget = that device must pair again
/// (lost phone, staff left); its sign-ins stop at once.
#[derive(Script, ScriptHook, Widget)]
struct PairedList {
    #[deref]
    view: View,
}

impl Widget for PairedList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (devices, forget, seen, now_on) = {
            let st = app_state().lock().unwrap();
            (
                st.lan_devices.clone(),
                st.t("forget_device"),
                st.t("last_seen"),
                st.t("connected_now"),
            )
        };
        let row = *locale_row_height().lock().unwrap();
        let n = devices.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row, 0.0).min(300.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, n);
                let mut drew = 0usize;
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(device) = devices.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    // "2026-10-24T18:40:12+05:30" → "24 Oct 18:40"; a device
                    // holding a `/wait` open is seen every half minute.
                    let seen_at = chrono::DateTime::parse_from_rfc3339(&device.last_seen_at).ok();
                    let recent = seen_at.is_some_and(|t| {
                        chrono::Local::now().signed_duration_since(t)
                            < chrono::Duration::seconds(3 * store::LAN_SEEN_EVERY_SECS)
                    });
                    let when = seen_at.map(|t| t.format("%d %b %H:%M").to_string()).unwrap_or_default();
                    let line = if recent {
                        format!("{} · {now_on}", device.name)
                    } else {
                        format!("{} · {seen} {when}", device.name)
                    };
                    item.widget(cx, ids!(label)).set_text(cx, &plain_display_text(&line));
                    item.button(cx, ids!(pick)).set_text(cx, &forget);
                    item.draw_all_unscoped(cx);
                    drew += 1;
                }
                if drew > 0 {
                    let extent = list.drawn_range_extent();
                    if extent > 1.0 {
                        measured_row = extent / drew as f64;
                    }
                }
            }
        }
        remember_content_row(cx, &mut self.view, locale_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

#[derive(Script, ScriptHook, Widget)]
struct CurrencyList {
    #[deref]
    view: View,
}

impl Widget for CurrencyList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let selected = app_state().lock().unwrap().locale.currency.clone();
        let use_label = {
            let st = app_state().lock().unwrap();
            st.t("use")
        };
        let walk = Walk {
            height: Size::Fixed(280.0),
            ..walk
        };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, CURRENCIES.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(cur) = CURRENCIES.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let mark = if cur.code == selected { "✓ " } else { "" };
                    item.widget(cx, ids!(label)).set_text(
                        cx,
                        &plain_display_text(&format!(
                            "{mark}{} {} — {}",
                            cur.symbol, cur.code, cur.name_en
                        )),
                    );
                    item.button(cx, ids!(pick)).set_text(cx, &use_label);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        claim_list_nested_scroll(cx, &self.view, event);
        self.view.handle_event(cx, event, scope);
        claim_list_nested_scroll(cx, &self.view, event);
    }
}

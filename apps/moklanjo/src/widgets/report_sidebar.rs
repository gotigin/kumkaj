/// One line of a report as it is laid out on screen.
#[derive(Clone, Debug, PartialEq)]
enum ReportRow {
    Head(String),
    Kv(String, String),
    Line(String),
    /// An indented item under a heading ("  Parcel tape · 3 left").
    Sub(String),
    Gap,
}

/// Lay out a printable report: "=== X ===", "— X —", "Title:", a title
/// followed by indented lines, the first line of a section with more under
/// it, and a word or two over figures ("Assets") become headings; figures
/// become label/value rows ([`report_values`]); "----" and blank lines
/// become a gap.
fn parse_report(text: &str) -> Vec<ReportRow> {
    // Undo plain_display_text's Markdown escapes.
    let text = text.replace("\\*", "*").replace("\\_", "_").replace("\\`", "`").replace("\\#", "#");
    let lines: Vec<&str> = text.lines().collect();
    let is_gap = |l: &str| {
        let l = l.trim();
        l.is_empty() || l.chars().all(|c| c == '-' || c == '=' || c == '—')
    };
    let mut rows: Vec<ReportRow> = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let indented = raw.starts_with("  ") || raw.starts_with('\t');
        let l = raw.trim();
        if is_gap(l) {
            if !matches!(rows.last(), None | Some(ReportRow::Gap)) {
                rows.push(ReportRow::Gap);
            }
            continue;
        }
        if indented {
            rows.push(ReportRow::Sub(l.to_string()));
            continue;
        }
        if l.starts_with("===") {
            rows.push(ReportRow::Head(l.trim_matches('=').trim().to_string()));
            continue;
        }
        // "— Barcode desk —": an ops section.
        if let Some(inner) = l.strip_prefix('—').and_then(|rest| rest.strip_suffix('—'))
            && !inner.trim().is_empty()
        {
            rows.push(ReportRow::Head(inner.trim().to_string()));
            continue;
        }
        if let Some(head) = l.strip_suffix(':') {
            rows.push(ReportRow::Head(head.trim().to_string()));
            continue;
        }
        let next = lines.get(i + 1).copied().filter(|n| !is_gap(n));
        if next.is_some_and(|n| n.starts_with("  ") || n.starts_with('\t')) {
            rows.push(ReportRow::Head(l.to_string()));
            continue;
        }
        if let Some(values) = report_values(l) {
            rows.extend(values);
            continue;
        }
        // "Party outstanding (aging)" opening its section; "Assets" over
        // the figures under it.
        let opens_section = matches!(rows.last(), None | Some(ReportRow::Gap)) && next.is_some();
        let names_figures = !l.chars().any(|c| c.is_ascii_digit())
            && l.split_whitespace().count() <= 3
            && next.is_some_and(|n| report_values(n.trim()).is_some());
        if opens_section || names_figures {
            rows.push(ReportRow::Head(l.to_string()));
        } else {
            rows.push(ReportRow::Line(l.to_string()));
        }
    }
    while matches!(rows.last(), Some(ReportRow::Gap)) {
        rows.pop();
    }
    if matches!(rows.first(), Some(ReportRow::Gap)) {
        rows.remove(0);
    }
    rows
}

/// A line of figures as label/value rows: "Bookings: 3", "Revenue ₹120.00",
/// "Expenses ₹50.00 (2)" (an amount and its count), and one row per part
/// of "Cash ₹120.00 · UPI ₹0.00". `None` when the line is not figures — a
/// long value too ("Next: Dispatch … or Out for delivery"): at the right of
/// a row it would squeeze its label on a phone, so it stays a full line.
fn report_values(l: &str) -> Option<Vec<ReportRow>> {
    if let Some((k, v)) = l.split_once(": ")
        && !k.trim().is_empty()
        && !v.trim().is_empty()
        && k.chars().count() <= 48
        && v.trim().chars().count() <= 32
    {
        return Some(vec![ReportRow::Kv(k.trim().to_string(), v.trim().to_string())]);
    }
    if let Some((k, v)) = trailing_amount(l) {
        return Some(vec![ReportRow::Kv(k, v)]);
    }
    if let Some((rest, count)) = l.rsplit_once(' ')
        && count.len() > 2
        && count.starts_with('(')
        && count.ends_with(')')
        && count[1..count.len() - 1].chars().all(|c| c.is_ascii_digit())
        && let Some((k, v)) = trailing_amount(rest)
    {
        return Some(vec![ReportRow::Kv(k, format!("{v} {count}"))]);
    }
    if l.contains(" · ") {
        let parts: Option<Vec<ReportRow>> = l
            .split(" · ")
            .map(|part| trailing_amount(part.trim()).map(|(k, v)| ReportRow::Kv(k, v)))
            .collect();
        return parts.filter(|parts| parts.len() > 1);
    }
    None
}

/// "Revenue ₹120.00" → ("Revenue", "₹120.00"): a label followed by one
/// amount or count. Not for lists ("·") or ranges/dates ("→", "2026-10-24").
fn trailing_amount(line: &str) -> Option<(String, String)> {
    if line.contains('·') || line.contains('→') {
        return None;
    }
    let (label, last) = line.rsplit_once(' ')?;
    let digits = last.trim_start_matches(|c: char| !c.is_ascii_digit());
    let prefix = &last[..last.len() - digits.len()];
    let amount_like = !digits.is_empty()
        && prefix.chars().count() <= 3
        && !prefix.chars().any(|c| c.is_alphabetic())
        && digits.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == '%')
        && !label.trim().is_empty();
    amount_like.then(|| (label.trim().to_string(), last.to_string()))
}

/// A heading row from a sentence that introduces a list ("Recent:"): the
/// card shows it as a heading, so the colon goes.
fn report_head(text: &str) -> ReportRow {
    ReportRow::Head(text.trim_end().trim_end_matches([':', '：']).trim_end().to_string())
}

/// "2026-10-26T12:21:47+05:30" → "26 Oct 12:21" (as Backup shows times);
/// anything else as written, minus seconds.
fn short_when(at: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(at)
        .map(|t| t.format("%d %b %H:%M").to_string())
        .unwrap_or_else(|_| at.get(..16).unwrap_or(at).replace('T', " "))
}

/// A report row's text, right-aligned in right-to-left languages.
fn set_report_label(cx: &mut Cx, item: &WidgetRef, id: &[LiveId], text: &str, align_x: f64) {
    let w = item.widget(cx, id);
    w.set_text(cx, text);
    if let Some(mut label) = w.borrow_mut::<WrapLabel>() {
        label.set_text_align_x(align_x);
    }
}

/// Rows as plain lines (what a [`ReportText`] fed with rows reads back as).
fn report_rows_text(rows: &[ReportRow]) -> String {
    rows.iter()
        .map(|row| match row {
            ReportRow::Head(t) => format!("{t}:"),
            ReportRow::Kv(k, v) => format!("{k}: {v}"),
            ReportRow::Line(t) => t.clone(),
            ReportRow::Sub(t) => format!("  {t}"),
            ReportRow::Gap => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A report or list on screen, laid out by [`parse_report`] (`set_text`)
/// or given as rows (`set_rows`).
#[derive(Script, ScriptHook, Widget)]
struct ReportText {
    #[deref]
    view: View,
    #[rust]
    text: String,
    #[rust]
    rows: Vec<ReportRow>,
    /// Content height measured at the last draw (rows wrap).
    #[rust]
    measured: f64,
    #[rust]
    settle: NextFrame,
}

impl Widget for ReportText {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.rows.is_empty() {
            return DrawStep::done();
        }
        let estimate = if self.measured > 1.0 { self.measured } else { self.rows.len() as f64 * 30.0 };
        let mut extent = 0.0;
        // The list fills the card; the card is as tall as the rows measured
        // last frame (plus its padding).
        if let Some(mut card) = self.view.view(cx, ids!(card)).borrow_mut() {
            card.walk.height = Size::Fixed(estimate + 14.0);
        }
        let rows = self.rows.clone();
        let (rtl, screen_hidden, keys_hidden, keys_peek) = {
            let st = app_state().lock().unwrap();
            (
                locale::is_rtl(&st.locale.language),
                st.amounts_screen_hidden,
                st.amount_keys_hidden.clone(),
                st.amount_keys_peek.clone(),
            )
        };
        let align_x = if rtl { 1.0 } else { 0.0 };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = rows.get(item_id) else {
                        continue;
                    };
                    let item = match row {
                        ReportRow::Head(t) => {
                            let item = list.item(cx, item_id, id!(Head));
                            set_report_label(cx, &item, ids!(head), t, align_x);
                            item
                        }
                        ReportRow::Kv(k, v) => {
                            let item = list.item(cx, item_id, if rtl { id!(KvRtl) } else { id!(Kv) });
                            set_report_label(cx, &item, ids!(k), k, align_x);
                            let (core, _) = split_trailing_count(v);
                            let money = looks_like_money_value(core);
                            let eye = item.button(cx, ids!(eye));
                            eye.set_visible(cx, money);
                            let shown = if money {
                                let key = report_kv_amount_key(k);
                                let visible = amount_key_visible(
                                    screen_hidden,
                                    &keys_hidden,
                                    &keys_peek,
                                    &key,
                                );
                                eye.set_text(cx, eye_glyph(visible));
                                if visible {
                                    v.clone()
                                } else {
                                    mask_kv_amount(v)
                                }
                            } else {
                                v.clone()
                            };
                            set_report_label(cx, &item, ids!(v), &shown, align_x);
                            item
                        }
                        ReportRow::Line(t) => {
                            let item = list.item(cx, item_id, id!(Line));
                            let shown = if screen_hidden && t.chars().any(|c| c.is_ascii_digit()) {
                                mask_money_tokens(t)
                            } else {
                                t.clone()
                            };
                            set_report_label(cx, &item, ids!(line), &shown, align_x);
                            item
                        }
                        ReportRow::Sub(t) => {
                            let item = list.item(cx, item_id, id!(Sub));
                            let shown = if screen_hidden && t.chars().any(|c| c.is_ascii_digit()) {
                                mask_money_tokens(t)
                            } else {
                                t.clone()
                            };
                            set_report_label(cx, &item, ids!(line), &shown, align_x);
                            item
                        }
                        ReportRow::Gap => list.item(cx, item_id, id!(Gap)),
                    };
                    item.draw_all_unscoped(cx);
                }
                extent = list.drawn_range_extent();
            }
        }
        if extent > 1.0 && (extent - self.measured).abs() > 1.0 {
            self.measured = extent;
            self.settle = cx.new_next_frame();
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.settle.is_event(event).is_some() {
            self.view.redraw(cx);
        }
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
        let Event::Actions(actions) = event else {
            return;
        };
        let list = self.view.portal_list(cx, ids!(list));
        let mut toggled = false;
        for (item_id, item) in list.items_with_actions(actions) {
            let Some(ReportRow::Kv(k, v)) = self.rows.get(item_id) else {
                continue;
            };
            let (core, _) = split_trailing_count(v);
            if !looks_like_money_value(core) {
                continue;
            }
            if !item.button(cx, ids!(eye)).clicked(actions) {
                continue;
            }
            toggle_amount_key_prefs(&report_kv_amount_key(k));
            toggled = true;
            break;
        }
        if toggled {
            self.view.redraw(cx);
        }
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        if self.text != v {
            self.text = v.to_string();
            self.rows = parse_report(v);
            self.measured = 0.0;
            self.view.redraw(cx);
        }
    }
}

impl ReportText {
    /// Rows built in code, for lists that never print (units, printers,
    /// audit …): nothing is guessed, so a name the office typed ("Rent:
    /// March") never turns into a heading or a figure.
    pub fn set_rows(&mut self, cx: &mut Cx, rows: Vec<ReportRow>) {
        if self.rows != rows {
            self.text = report_rows_text(&rows);
            self.rows = rows;
            self.measured = 0.0;
            self.view.redraw(cx);
        }
    }
}

/// Settings → Sidebar: one row per chrome peer (order + show/hide).
#[derive(Script, ScriptHook, Widget)]
struct SidebarNavList {
    #[deref]
    view: View,
    /// Set by `pointer_sidebar_settings_drag` when a reorder gesture
    /// went active, so a release does not also toggle show/hide.
    #[rust]
    suppress_face_click: bool,
}

impl Widget for SidebarNavList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (order, hidden) = {
            let st = app_state().lock().unwrap();
            (st.nav_order.clone(), st.nav_hidden.clone())
        };
        let row = *sidebar_nav_row_height().lock().unwrap();
        let n = order.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row, 0.0)),
            ..walk
        };
        let mut icons_fresh = false;
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, n);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(tab) = order.get(item_id).copied() else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    let label = {
                        let lang = app_state().lock().unwrap().locale.language.clone();
                        match tab {
                            NavTab::Books => locale::t(&lang, "books"),
                            NavTab::Sell => locale::t(&lang, "sell"),
                            NavTab::Buy => locale::t(&lang, "buy"),
                            NavTab::Rates => locale::t(&lang, "sidebar_menu_items"),
                            NavTab::Ops => "Ops".into(),
                            NavTab::Cod => "COD".into(),
                            NavTab::Riders => locale::t(&lang, "riders"),
                            NavTab::Sales => locale::t(&lang, "sales"),
                            NavTab::DayClose => locale::t(&lang, "day_close"),
                            NavTab::Reports => locale::t(&lang, "nav_reports"),
                            NavTab::Inventory => locale::t(&lang, "inventory"),
                            NavTab::Branches => locale::t(&lang, "branches"),
                            NavTab::Credit => locale::t(&lang, "credit"),
                            NavTab::Team => locale::t(&lang, "team"),
                            NavTab::Settings => locale::t(&lang, "nav_settings"),
                            NavTab::Lock => locale::t(&lang, "nav_lock"),
                        }
                    };
                    let visible = nav_tab_is_visible(&hidden, tab);
                    let always = nav_tab_always_visible(tab);
                    let always_lbl = {
                        let lang = app_state().lock().unwrap().locale.language.clone();
                        locale::t(&lang, "sidebar_always_on_short")
                    };
                    {
                        let face = item.button(cx, ids!(face));
                        face.set_text(cx, &label);
                        face.set_enabled(cx, true);
                    }
                    {
                        let icon = item.image(cx, ids!(icon));
                        if load_nav_icon_once(cx, &icon, tab) {
                            icons_fresh = true;
                        }
                    }
                    let on = visible || always;
                    item.view(cx, ids!(track_off)).set_visible(cx, !on);
                    item.view(cx, ids!(track_on)).set_visible(cx, on);
                    item.view(cx, ids!(thumb_off)).set_visible(cx, !on);
                    item.view(cx, ids!(thumb_on)).set_visible(cx, on);
                    {
                        let hit = item.button(cx, ids!(toggle_hit));
                        hit.set_enabled(cx, !always);
                    }
                    item.label(cx, ids!(always_label)).set_visible(cx, always);
                    item.label(cx, ids!(always_label)).set_text(cx, &always_lbl);
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
        remember_content_row(cx, &mut self.view, sidebar_nav_row_height(), row, measured_row);
        // Texture upload from load_image_from_data lands next frame —
        // without a redraw the rows stay blank (same reason rail icons
        // are painted from sync_chrome, not only inside draw).
        if icons_fresh {
            self.view.redraw(cx);
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Reorder is owned by App::pointer_sidebar_settings_drag (same
        // ghost chip as the rail). This widget only draws + face taps.
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

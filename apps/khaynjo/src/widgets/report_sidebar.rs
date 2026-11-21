/// One line of a report as it is laid out on screen.
#[derive(Clone, Debug, PartialEq)]
enum ReportRow {
    Head(String),
    Kv(String, String),
    Line(String),
    /// An indented item under a heading ("  Paneer · 3 · ₹720").
    Sub(String),
    /// Print `\t` columns (bill Item|Qty|Price|Amount, Date|Table, …) as a
    /// real responsive row — never paint `\t` (tofu) or hope monospace aligns.
    Cols(Vec<String>),
    Gap,
}

/// RFC3339 → owner-facing date/time via fiscal helpers; fallback strips seconds.
fn short_when(at: &str) -> String {
    let shown = crate::fiscal::format_datetime(at, "dmy", "24h");
    if shown.is_empty() {
        at.get(..16).unwrap_or(at).replace('T', " ")
    } else {
        shown
    }
}

/// Lay out a printable report: "=== X ===", "Title:", a title followed by
/// indented lines, the first line of a section with more under it, and a
/// word or two over figures ("Assets") become headings; figures become
/// label/value rows ([`report_values`]); "----" and blank lines become a gap.
fn parse_report(text: &str) -> Vec<ReportRow> {
    // A bill or KOT shown on screen (Floor → View bill) carries each line's
    // print marks; the screen shows the words.
    let text = print_share::strip_kot_style_marks(text);
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
        // Mid-line tabs = print columns (bill / KOT / paired meta). Do this
        // before ": " / trailing-amount heuristics so "Thali\t5\t…" never
        // becomes a Kv with a tofu tab in the label.
        if let Some(cols) = report_tab_cols(l) {
            rows.push(cols);
            continue;
        }
        if l.starts_with("===") {
            rows.push(ReportRow::Head(l.trim_matches('=').trim().to_string()));
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

/// A line of figures as label/value rows: "Bills: 3", "Revenue ₹120.00",
/// "Expenses ₹50.00 (2)" (an amount and its count), and one row per part
/// of "Cash ₹120.00 · UPI ₹0.00". `None` when the line is not figures — a
/// long value too ("Next: dispatch … or deliver"): at the right of a row
/// it would squeeze its label on a phone, so it stays a full-width line.
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

/// A heading row from a sentence that introduces a list ("Recent:"): the
/// card shows it as a heading, so the colon goes.
fn report_head(text: &str) -> ReportRow {
    ReportRow::Head(text.trim_end().trim_end_matches([':', '：']).trim_end().to_string())
}

/// `Item\tQty.\tPrice\tAmount` / `Date\tTable` → real columns (cap 4). Empty
/// cells stay so trailing slots still align under the header.
fn report_tab_cols(line: &str) -> Option<ReportRow> {
    if !line.contains('\t') {
        return None;
    }
    let mut cells: Vec<String> = line.split('\t').map(|c| c.trim().to_string()).collect();
    while cells.last().is_some_and(|c| c.is_empty()) {
        cells.pop();
    }
    if cells.len() < 2 {
        return None;
    }
    if cells.len() > 4 {
        let rest = cells.split_off(4);
        if let Some(last) = cells.last_mut() {
            last.push(' ');
            last.push_str(&rest.join(" "));
        }
    }
    Some(ReportRow::Cols(cells))
}

/// A report row's text, right-aligned in right-to-left languages.
fn set_preview_em(cx: &mut Cx, item: &WidgetRef, id: &[LiveId], em: f32) {
    if let Some(mut label) = item.widget(cx, id).borrow_mut::<WrapLabel>() {
        label.markdown.text_flow.font_size = em;
    }
}

fn set_report_label(cx: &mut Cx, item: &WidgetRef, id: &[LiveId], text: &str, align_x: f64) {
    let w = item.widget(cx, id);
    w.set_text(cx, text);
    if let Some(mut label) = w.borrow_mut::<WrapLabel>() {
        label.set_text_align_x(align_x);
    }
}

/// Up to four print columns: first cell flexes (item / left meta); Qty /
/// Price / Amount sit in Fixed slots (see `Cols` DSL) so every row shares
/// the same gutters. Paired meta (`Date\tTable`) puts the trailing cell in
/// the Amount slot — not the skinny Qty gutter.
fn paint_report_cols(
    cx: &mut Cx,
    item: &WidgetRef,
    cells: &[String],
    align_x: f64,
    screen_hidden: bool,
) {
    const IDS: [&[LiveId]; 4] = [ids!(c0), ids!(c1), ids!(c2), ids!(c3)];
    let n = cells.len().min(4);
    // Logical cell → slot. 2-col meta uses Amount's column so "Table: T1"
    // is not crushed into the Qty width.
    let slot_for: [Option<usize>; 4] = match n {
        0 => [None, None, None, None],
        1 => [Some(0), None, None, None],
        2 => [Some(0), None, None, Some(1)],
        3 => [Some(0), Some(1), Some(2), None],
        _ => [Some(0), Some(1), Some(2), Some(3)],
    };
    let headerish = cells.iter().all(|c| !c.chars().any(|ch| ch.is_ascii_digit()));
    for (slot, path) in IDS.iter().enumerate() {
        let cell = slot_for[slot].and_then(|i| cells.get(i));
        let w = item.widget(cx, path);
        w.set_visible(cx, cell.is_some());
        let Some(text) = cell else {
            continue;
        };
        let logical = slot_for[slot].unwrap_or(0);
        let shown = if screen_hidden && logical > 0 && text.chars().any(|c| c.is_ascii_digit()) {
            mask_money_tokens(text)
        } else {
            text.clone()
        };
        // Name / left meta follow script direction; numeric slots stay
        // trailing-edge aligned so Qty/Price/Amount line up under the header.
        let ax = if slot == 0 {
            align_x
        } else if align_x > 0.5 {
            0.0
        } else {
            1.0
        };
        set_report_label(cx, item, path, &shown, ax);
        if let Some(mut label) = w.borrow_mut::<WrapLabel>() {
            if headerish {
                label.markdown.text_flow.font_color = vec4(0.42, 0.29, 0.25, 1.0);
            } else {
                label.markdown.text_flow.font_color = vec4(0.15, 0.04, 0.03, 1.0);
            }
        }
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
            ReportRow::Cols(cells) => cells.join("\t"),
            ReportRow::Gap => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n")
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

/// The least width a label keeps beside its value, and the gap between the
/// two (`spacing` of the `Kv` row in the layout).
const KV_MIN_LABEL_W: f64 = 96.0;
const KV_GAP: f64 = 8.0;

/// Whether a label/value row is drawn stacked — the label above, the value
/// under it, both wrapping at the row's width — instead of side by side:
/// when the value at its own width would leave the label less than
/// [`KV_MIN_LABEL_W`]. Makepad has no flex-shrink: the value keeps its
/// width and the label gets what is left, so a long value ("Standard
/// Portrait · No printer assigned" on a phone) left the label a few px,
/// wrapped one letter per line, and ran off the card itself (DEV.md bug 7j).
fn kv_stacks(row_w: f64, value_w: f64) -> bool {
    row_w > 1.0 && value_w > 0.0 && value_w + KV_GAP + KV_MIN_LABEL_W > row_w
}

/// The print-column row's three number slots and their gaps (`Cols` in the
/// layout: 44 + 96 + 96 and three gaps of 14), and the least the name
/// beside them may get.
const COLS_FIXED_W: f64 = 44.0 + 96.0 + 96.0 + 3.0 * 14.0;
const COLS_MIN_NAME_W: f64 = 110.0;

/// Whether a print-column row is drawn stacked — the name on its own line,
/// the numbers under it (`ColsNarrow`): when the card is too narrow to give
/// the name [`COLS_MIN_NAME_W`] beside the Fixed number slots. On a 360 px
/// phone the name got a few px, wrapped one letter per line and the bill on
/// screen had no dish names (2026-11-21).
fn cols_stack(row_w: f64) -> bool {
    row_w > 1.0 && row_w < COLS_FIXED_W + COLS_MIN_NAME_W
}

/// A report on screen, laid out by [`parse_report`]. Set with `set_text`.
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
    /// Each label/value row's value width as drawn beside its label
    /// (0 = not measured yet) and the rows' width, for [`kv_stacks`].
    #[rust]
    kv_value_w: Vec<f64>,
    #[rust]
    kv_row_w: f64,
    /// Rows are tap targets (Settings → Areas · stations · tax rates, Units,
    /// printer pool): the row takes the press (a View emits FingerUp only
    /// with a cursor — without one these taps never fired) and its text is
    /// not selectable (DESIGN § Text selection: text inside a tap target).
    #[live]
    tappable: bool,
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
            card.walk.height = Size::Fixed(estimate + 18.0);
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
        let mut value_w = std::mem::take(&mut self.kv_value_w);
        let mut row_w = self.kv_row_w;
        let mut restack = false;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = rows.get(item_id) else {
                        continue;
                    };
                    let is_kv = matches!(row, ReportRow::Kv(..));
                    let stacked =
                        is_kv && kv_stacks(row_w, value_w.get(item_id).copied().unwrap_or(0.0));
                    let mut kv_eye = false;
                    let item = match row {
                        ReportRow::Head(t) => {
                            let item = list.item(cx, item_id, id!(Head));
                            set_report_label(cx, &item, ids!(head), t, align_x);
                            item
                        }
                        ReportRow::Kv(k, v) => {
                            let template = match (stacked, rtl) {
                                (false, false) => id!(Kv),
                                (false, true) => id!(KvRtl),
                                (true, false) => id!(KvStack),
                                (true, true) => id!(KvStackRtl),
                            };
                            let item = list.item(cx, item_id, template);
                            set_report_label(cx, &item, ids!(row.k), k, align_x);
                            let (core, _) = split_trailing_count(v);
                            let money = looks_like_money_value(core);
                            let eye = item.button(cx, ids!(row.eye));
                            eye.set_visible(cx, money);
                            kv_eye = money;
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
                            set_report_label(cx, &item, ids!(row.v), &shown, align_x);
                            if self.tappable {
                                if let Some(mut r) = item.view(cx, ids!(row)).borrow_mut() {
                                    r.cursor = Some(MouseCursor::Hand);
                                }
                                for id in [ids!(row.k), ids!(row.v)] {
                                    if let Some(mut l) = item.widget(cx, id).borrow_mut::<WrapLabel>() {
                                        l.markdown.text_flow.selectable = false;
                                    }
                                }
                            }
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
                        ReportRow::Cols(cells) => {
                            let template = match (cols_stack(row_w), rtl) {
                                (false, false) => id!(Cols),
                                (false, true) => id!(ColsRtl),
                                // Two cells are a meta pair, not Item | Qty | …
                                (true, _) if cells.len() == 2 => id!(ColsNarrowPair),
                                (true, false) => id!(ColsNarrow),
                                (true, true) => id!(ColsNarrowRtl),
                            };
                            let item = list.item(cx, item_id, template);
                            paint_report_cols(
                                cx,
                                &item,
                                cells,
                                align_x,
                                screen_hidden,
                            );
                            item
                        }
                        ReportRow::Gap => list.item(cx, item_id, id!(Gap)),
                    };
                    item.draw_all_unscoped(cx);
                    if matches!(row, ReportRow::Cols(..)) {
                        // The card's width is known once a row is drawn; a
                        // row drawn in the wrong shape is drawn again.
                        let was = cols_stack(row_w);
                        let w = item.area().rect(cx).size.x;
                        if w > 1.0 {
                            row_w = w;
                        }
                        if cols_stack(row_w) != was {
                            restack = true;
                        }
                    }
                    if is_kv {
                        // The rows' width, and (side by side only: stacked,
                        // the value fills the row) the value's own width.
                        let w = item.area().rect(cx).size.x;
                        let mut seen = value_w.get(item_id).copied().unwrap_or(0.0);
                        if !stacked {
                            seen = item.widget(cx, ids!(row.v)).area().rect(cx).size.x;
                            if kv_eye {
                                seen += item.widget(cx, ids!(row.eye)).area().rect(cx).size.x + KV_GAP;
                            }
                            if value_w.len() < rows.len() {
                                value_w.resize(rows.len(), 0.0);
                            }
                            value_w[item_id] = seen;
                        }
                        if w > 1.0 {
                            row_w = w;
                        }
                        if kv_stacks(row_w, seen) != stacked {
                            restack = true;
                        }
                    }
                }
                extent = list.drawn_range_extent();
            }
        }
        self.kv_value_w = value_w;
        self.kv_row_w = row_w;
        if extent > 1.0 && (extent - self.measured).abs() > 1.0 {
            self.measured = extent;
            self.settle = cx.new_next_frame();
        }
        if restack {
            // A row changes shape (stacked / side by side): draw it again.
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
            if !item.button(cx, ids!(row.eye)).clicked(actions) {
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
            self.kv_value_w.clear();
            self.view.redraw(cx);
        }
    }
}

impl ReportText {
    /// Rows built in code, for lists that never print (units, printers,
    /// audit …): nothing is guessed, so a name the shop typed ("Rent:
    /// March") never turns into a heading or a figure.
    pub fn set_rows(&mut self, cx: &mut Cx, rows: Vec<ReportRow>) {
        if self.rows != rows {
            self.text = report_rows_text(&rows);
            self.rows = rows;
            self.measured = 0.0;
            self.kv_value_w.clear();
            self.view.redraw(cx);
        }
    }

}

/// Settings → Sidebar: one row per chrome peer (order + show/hide).
#[derive(Script, ScriptHook, Widget)]
struct SidebarNavList {
    #[deref]
    view: View,
    /// Set when a Settings → Sidebar ↑↓ reorder should suppress the
    /// following face click (so release does not also toggle show/hide).
    #[rust]
    suppress_face_click: bool,
}

impl Widget for SidebarNavList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (order, hidden) = {
            let st = app_state().lock().unwrap();
            (st.nav_order.clone(), st.nav_hidden.clone())
        };
        // "Always on" beside ↑ ↓ and the switch needs a wide row: on a phone
        // it left the row's own label no room (the switch, on and not
        // tappable, already says it).
        let roomy = cx.turtle().next_walk_width(walk.width, walk.margin) >= 420.0;
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
                        // Shown to shop admins only, so Team reads "Team & PINs".
                        locale::t(&lang, nav_tab_label_key(tab, false))
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
                    item.label(cx, ids!(always_label)).set_visible(cx, always && roomy);
                    item.label(cx, ids!(always_label)).set_text(cx, &always_lbl);
                    item.button(cx, ids!(move_up)).set_visible(cx, item_id > 0);
                    item.button(cx, ids!(move_down))
                        .set_visible(cx, item_id + 1 < n);
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
        // Reorder is ↑↓ on each row (App::handle_sidebar_nav_list_actions).
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// Inventory: every stock item as a row (raw materials first).
#[derive(Script, ScriptHook, Widget)]
struct StockList {
    #[deref]
    view: View,
}

impl Widget for StockList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (items, raw, resale, low, levels) = {
            let st = app_state().lock().unwrap();
            (st.stock_items(), st.t("kind_raw"), st.t("kind_resale"), st.t("low_badge"), st.reorder_levels.clone())
        };
        // Full content height + page scroll (CustomerList / SaleList rule).
        // A short Fixed pitch (old 62) clipped the last Fit rows into the
        // tip-pool ReportText and left the page with nothing to scroll.
        let row = *stock_row_height().lock().unwrap();
        let n = items.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, n);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(p) = items.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    item.label(cx, ids!(stock_name)).set_text(cx, &p.name);
                    item.label(cx, ids!(stock_kind))
                        .set_text(cx, if p.is_raw() { &raw } else { &resale });
                    let on_hand = p.stock_qty.max(0);
                    let unit = p.unit_label.trim();
                    item.label(cx, ids!(stock_qty))
                        .set_text(cx, &if unit.is_empty() { format!("{on_hand}") } else { format!("{on_hand} {unit}") });
                    let reorder_at = levels.get(&p.id).copied().unwrap_or(store::DEFAULT_REORDER_AT);
                    item.view(cx, ids!(stock_low)).set_visible(cx, on_hand <= reorder_at);
                    item.label(cx, ids!(stock_low_text)).set_text(cx, &low);
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
        remember_content_row(cx, &mut self.view, stock_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// The expense form's ways of paying (see [`PayPart`]).
#[derive(Script, ScriptHook, Widget)]
struct PartsList {
    #[deref]
    view: View,
}

/// Chip ids per method in a [`PartsList`] row, in [`PART_METHODS`] order.
const PART_CHIPS: [(&[LiveId], &[LiveId]); 4] = [
    (ids!(part_cash), ids!(part_cash_on)),
    (ids!(part_upi), ids!(part_upi_on)),
    (ids!(part_card), ids!(part_card_on)),
    (ids!(part_bank), ids!(part_bank_on)),
];

impl Widget for PartsList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (parts, labels, rest) = {
            let st = app_state().lock().unwrap();
            (
                st.pay_parts.clone(),
                [
                    st.t("cash"),
                    st.fiscal.digital_pay_label().to_string(),
                    st.t("card"),
                    st.t("bank"),
                ],
                st.t("rest_ph"),
            )
        };
        let row = *parts_row_height().lock().unwrap();
        let n = parts.len();
        let walk = Walk {
            height: Size::Fixed(list_content_height(n, row, 0.0)),
            ..walk
        };
        let mut measured_row = 0.0;
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, n);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(part) = parts.get(item_id) else {
                        continue;
                    };
                    let item = list.item(cx, item_id, id!(Item));
                    for (i, (idle, on)) in PART_CHIPS.iter().enumerate() {
                        let picked = PART_METHODS[i] == part.method;
                        item.button(cx, idle).set_text(cx, &labels[i]);
                        item.button(cx, on).set_text(cx, &labels[i]);
                        item.button(cx, idle).set_visible(cx, !picked);
                        item.button(cx, on).set_visible(cx, picked);
                    }
                    let amount = item.text_input(cx, ids!(part_amount));
                    amount.set_empty_text(cx, rest.clone());
                    if !cx.has_key_focus(amount.area()) && amount.text() != part.amount {
                        amount.set_text(cx, &part.amount);
                    }
                    item.button(cx, ids!(part_remove)).set_visible(cx, n > 1);
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
        remember_content_row(cx, &mut self.view, parts_row_height(), row, measured_row);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }
}

/// One preview line. A tab in the slip text is a name / Qty. column.
#[derive(Clone, Debug, PartialEq)]
enum SlipPreviewRow {
    /// Text, horizontal align, and point size.
    Line(String, f64, f32),
    Qty(String, String, f64, f32),
    Gap,
}

/// Kitchen-slip preview. Not a report: no headings, no money eye.
/// `Item` and `Qty.` use the same row as each dish and its count.
#[derive(Script, ScriptHook, Widget)]
struct KotSlipPreview {
    #[deref]
    view: View,
    #[rust]
    rows: Vec<SlipPreviewRow>,
    #[rust]
    measured: f64,
    #[rust]
    settle: NextFrame,
    /// The preview is the print picture, not a second layout.
    #[rust]
    page: bool,
    /// Print picture size in pixels, and the body-type height inside it.
    #[rust]
    page_px: (f64, f64),
    #[rust]
    type_px: f64,
    /// Last column width that laid out. Used when this frame's width is not ready yet.
    #[rust]
    page_width: f64,
    /// `slip_preview_key` of the picture on screen (0 = none). Bill & KOT
    /// repaints run several times per tap; the same picture is not redrawn.
    #[rust]
    page_key: u64,
    /// The paper the picture was last drawn on (`show_slip_on`); a face that
    /// names another printer draws it again.
    #[rust]
    shown_paper: Option<PreviewPaper>,
}

fn preview_viewport(cx: &Cx) -> f64 {
    let mut best = 0.0f64;
    for id in cx.windows.id_iter() {
        let h = cx.windows[id].get_inner_size().y;
        if h.is_finite() && h > best {
            best = h;
        }
    }
    if best < 200.0 {
        640.0
    } else {
        // Room for the settings header above the picture.
        (best * 0.62).clamp(220.0, 900.0)
    }
}

fn preview_align_x(align: print_share::KotAlign, rtl: bool) -> f64 {
    match align {
        print_share::KotAlign::Center => 0.5,
        print_share::KotAlign::Start if rtl => 1.0,
        print_share::KotAlign::End if rtl => 0.0,
        print_share::KotAlign::Start => 0.0,
        print_share::KotAlign::End => 1.0,
    }
}

fn slip_preview_rows(text: &str, rtl: bool) -> Vec<SlipPreviewRow> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        let (align, em, body) = print_share::kot_preview_metrics(line);
        let x = preview_align_x(align, rtl);
        let em = em.clamp(12, 48) as f32;
        if body.trim().is_empty() {
            if !matches!(rows.last(), None | Some(SlipPreviewRow::Gap)) {
                rows.push(SlipPreviewRow::Gap);
            }
            continue;
        }
        if let Some((left, right)) = body.split_once('\t') {
            rows.push(SlipPreviewRow::Qty(
                left.trim().to_string(),
                print_share::kot_qty_column(right),
                x,
                em,
            ));
        } else {
            rows.push(SlipPreviewRow::Line(body.trim().to_string(), x, em));
        }
    }
    while matches!(rows.last(), Some(SlipPreviewRow::Gap)) {
        rows.pop();
    }
    rows
}

impl Widget for KotSlipPreview {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.page {
            // The column width is whatever this screen laid out. The picture
            // keeps its own proportions inside that width — no fixed pixel size.
            let rect = cx.peek_walk_turtle(walk);
            let mut avail = rect.size.x;
            if !avail.is_finite() || avail < 40.0 {
                avail = self.page_width;
            }
            if avail < 40.0 {
                // The column has not been laid out yet. Try again next frame
                // instead of keeping the previous picture at the wrong size.
                self.settle = cx.new_next_frame();
            } else {
                self.page_width = avail;
            }
            let (png_w, png_h) = self.page_px;
            let mut draw_walk = walk;
            if avail > 40.0 && png_w > 1.0 && png_h > 1.0 {
                let viewport = preview_viewport(cx.cx);
                let (w, h) = print_share::preview_frame(png_w, png_h, avail, viewport, self.type_px);
                draw_walk.width = Size::Fixed(w);
                draw_walk.height = Size::Fixed(h);
            }
            while self.view.draw_walk(cx, scope, draw_walk).step().is_some() {}
            return DrawStep::done();
        }
        if self.rows.is_empty() {
            return DrawStep::done();
        }
        let estimate = if self.measured > 1.0 {
            self.measured
        } else {
            self.rows.len() as f64 * 28.0
        };
        let mut extent = 0.0;
        if let Some(mut card) = self.view.view(cx, ids!(card)).borrow_mut() {
            card.walk.height = Size::Fixed(estimate + 18.0);
        }
        let rows = self.rows.clone();
        let rtl = locale::is_rtl(&app_state().lock().unwrap().locale.language);
        let qty_align = if rtl { 0.0 } else { 1.0 };
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = rows.get(item_id) else {
                        continue;
                    };
                    let item = match row {
                        SlipPreviewRow::Line(t, x, em) => {
                            let item = list.item(cx, item_id, id!(Line));
                            set_report_label(cx, &item, ids!(line), t, *x);
                            set_preview_em(cx, &item, ids!(line), *em);
                            item
                        }
                        SlipPreviewRow::Qty(name, qty, x, em) => {
                            let item = list.item(
                                cx,
                                item_id,
                                if rtl { id!(QtyRtl) } else { id!(Qty) },
                            );
                            set_report_label(cx, &item, ids!(k), name, *x);
                            set_report_label(cx, &item, ids!(v), qty, qty_align);
                            set_preview_em(cx, &item, ids!(k), *em);
                            set_preview_em(cx, &item, ids!(v), *em);
                            item
                        }
                        SlipPreviewRow::Gap => list.item(cx, item_id, id!(Gap)),
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
        // The picture does not scroll on its own. Wheel, trackpad, and a
        // finger drag belong to the settings page, including over the preview.
        if self.page || page_owns_scroll(event) {
            return;
        }
        self.view.handle_event(cx, event, scope);
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        self.set_slip(cx, v);
    }
}

impl KotSlipPreview {
    /// Slip text after style marks are stripped. A tab is the Qty. column.
    pub fn set_slip(&mut self, cx: &mut Cx, text: &str) {
        let rtl = locale::is_rtl(&app_state().lock().unwrap().locale.language);
        let rows = slip_preview_rows(text, rtl);
        if self.rows != rows {
            self.rows = rows;
            self.measured = 0.0;
            self.page = false;
            self.page_key = 0;
            self.view.redraw(cx);
        }
    }

    /// Show `text` on a printer's virtual paper exactly as it prints there
    /// (docs/PRINTER.md § Paper and preview): OS document — the print's own
    /// sheet (`os_paper::sheet_for`: the part of the printer's own page
    /// that is paper) and page plan; the virtual paper drawn whole, what
    /// the printer cannot reach of it shaded, a long slip on its next
    /// pages, a wide one cut where the print cuts it and what it cuts off
    /// drawn faded; ESC/POS — the roll the virtual paper's width wide, the
    /// picture the printer gets at its place across it (what the roll cuts
    /// off past it, faded).
    /// One scale, chosen on draw, keeps the print's ratio: big enough to
    /// read, and fitted to the screen when that still is. The same picture
    /// is not drawn again; it goes to the screen as pixels (no PNG round trip).
    pub fn show_slip_on(
        &mut self,
        cx: &mut Cx,
        text: &str,
        metrics: &receipt_raster::RasterMetrics,
        lay: os_paper::Lay,
        size_em: f32,
        paper: &PreviewPaper,
    ) {
        // Before any early return: the face compares against this.
        self.shown_paper = Some(paper.clone());
        let landscape = lay.landscape;
        let type_px = print_share::preview_type_px(size_em, metrics.page_dots());
        let key = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            print_share::slip_preview_key(text, metrics, landscape, type_px).hash(&mut h);
            format!("{paper:?}{lay:?}").hash(&mut h);
            h.finish()
        };
        if self.page && self.page_key == key {
            return;
        }
        let (px, w, h) = print_share::slip_page_gray(text, metrics, false);
        let Some(upright) = ::image::GrayImage::from_raw(w, h, px) else {
            return;
        };
        // The picture draws each printer dot as a block of pixels.
        let dot = (w as usize / metrics.page_dots().max(1)).max(1) as u32;
        let content_mm = print_share::os_document_content_mm(metrics);
        let pic = if paper.os_document {
            let sheet = os_paper::sheet_for(
                paper.virtual_paper,
                paper.own.as_ref(),
                print_share::slip_mm(&upright, content_mm),
                landscape,
            );
            os_paper::preview_picture(&upright, dot, content_mm, Some(&sheet), 0.0, None, lay)
        } else {
            let turned = landscape.then(|| ::image::imageops::rotate90(&upright));
            os_paper::preview_picture(&upright, dot, content_mm, None, paper.virtual_paper.width_mm, turned.as_ref(), lay)
        };
        let (pw, ph) = pic.gray.dimensions();
        let Ok(buf) = makepad_widgets::makepad_draw::image_cache::ImageBuffer::new(
            pic.gray.as_raw(),
            pw as usize,
            ph as usize,
        ) else {
            return;
        };
        let img = self.view.image(cx, ids!(slip));
        let texture = buf.into_new_texture(cx);
        img.set_texture(cx, Some(texture));
        if let Some(mut inner) = img.borrow_mut() {
            inner.walk.width = Size::fill();
            inner.walk.height = Size::fill();
            inner.redraw(cx);
        }
        self.page = true;
        self.page_key = key;
        self.page_px = (pw.max(1) as f64, ph.max(1) as f64);
        // The readable check is in the picture's own pixels.
        self.type_px = (type_px / pic.per_px.max(1e-6)).max(1.0);
        self.rows.clear();
        self.walk.width = Size::fill();
        self.walk.height = Size::fit();
        if let Some(mut card) = self.view.view(cx, ids!(card)).borrow_mut() {
            card.walk.width = Size::fill();
            card.walk.height = Size::fill();
        }
        self.view.redraw(cx);
    }

    /// The paper the picture was last drawn on (`show_slip_on`).
    pub fn shown_paper(&self) -> Option<&PreviewPaper> {
        self.shown_paper.as_ref()
    }
}

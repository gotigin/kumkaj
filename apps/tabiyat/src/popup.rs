// Dropdowns and pick lists: one rule set for every one of them, for a mouse
// (with or without a wheel), a trackpad, and a finger — DEV.md § Dropdowns.
//
// 1. **The app state is the only switch.** A list is open while
//    `suggest_open` names it; every list and popup reads that at draw time.
//    Nothing keeps a second open/closed flag that can disagree (a Makepad
//    `Modal` used for the Floor Columns pick stayed open after the state
//    closed, invisible, and swallowed every press on every screen).
// 2. **A tap opens, focus never does.** A press and release on the field
//    (or its dropdown button) opens the list, also when the field already
//    has focus. Typing opens it narrowed; ↓ opens it from the keyboard.
//    Focus alone opens nothing: closing a modal hands focus back to the
//    field it came from, and the Columns pick reopened itself on every pick
//    and every outside tap that way.
// 3. **One router closes.** A press anywhere outside the open list, its
//    field, and its dropdown button closes it before the press is
//    delivered; the press still reaches whatever is under it. A pick
//    closes a single-choice list; Back / Escape close it; leaving the
//    screen closes it.
// 4. **Screen rects only.** Pointer positions are tested against
//    `clipped_rect` — the same rect Makepad's own hit test uses — never
//    `rect()` (layout space: the hidden part of a field scrolled under
//    the header still counted as the field).
// 5. **Tips too.** An ℹ callout closes on the next press anywhere (a
//    finger never sends the hover-out a mouse does).
// 6. **A floating list follows its field.** `AnchoredPopup` draws on the
//    overlay at its `PopupAnchor`'s rect *every frame*, so it stays under
//    the field while the page scrolls, flips above it when there is no
//    room below, and keeps inside the window. Placing it once at open left
//    a gap of exactly the scroll distance.

/// The on-screen part of an area, as Makepad's hit test sees it: clipped by
/// every scroll view around it. `None` when it is not on screen this frame.
fn screen_rect(cx: &Cx, area: Area) -> Option<Rect> {
    if area.is_empty() || !area.is_valid(cx) {
        return None;
    }
    let r = area.clipped_rect(cx);
    (r.size.x > 0.5 && r.size.y > 0.5).then_some(r)
}

fn area_hit(cx: &Cx, area: Area, abs: DVec2) -> bool {
    screen_rect(cx, area).is_some_and(|r| r.contains(abs))
}

/// Travel and time within which press → release is a tap, not a scroll or
/// a text selection (same 24 px as `finger_up_is_tap`; a long press on a
/// touch field starts selection).
const POPUP_TAP_TRAVEL: f64 = 24.0;
const POPUP_TAP_SECS: f64 = 0.8;

fn popup_is_tap(press: DVec2, press_time: f64, release: DVec2, release_time: f64) -> bool {
    (release - press).length() < POPUP_TAP_TRAVEL
        && (release_time - press_time) >= 0.0
        && (release_time - press_time) < POPUP_TAP_SECS
}

/// One pointer phase of a mouse (primary button) or the first finger.
#[derive(Clone, Copy, Debug, PartialEq)]
enum PopupPointer {
    Press { abs: DVec2, time: f64 },
    Move { abs: DVec2 },
    Release { abs: DVec2, time: f64 },
}

fn popup_pointer(event: &Event) -> Option<PopupPointer> {
    match event {
        Event::MouseDown(e) if e.button.is_primary() => Some(PopupPointer::Press {
            abs: e.abs,
            time: e.time,
        }),
        Event::MouseMove(e) => Some(PopupPointer::Move { abs: e.abs }),
        Event::MouseUp(e) if e.button.is_primary() => Some(PopupPointer::Release {
            abs: e.abs,
            time: e.time,
        }),
        Event::TouchUpdate(e) => {
            let t = e.touches.first()?;
            match t.state {
                TouchState::Start => Some(PopupPointer::Press { abs: t.abs, time: t.time }),
                TouchState::Move => Some(PopupPointer::Move { abs: t.abs }),
                TouchState::Stop => Some(PopupPointer::Release { abs: t.abs, time: t.time }),
                _ => None,
            }
        }
        _ => None,
    }
}

/// A press that began on a type-or-pick field: it opens that field's list
/// when it ends as a tap on the same field.
#[derive(Clone, Copy, Debug)]
struct PopupPress {
    list: OpenList,
    abs: DVec2,
    time: f64,
}

fn popup_press() -> &'static Mutex<Option<PopupPress>> {
    static PRESS: Mutex<Option<PopupPress>> = Mutex::new(None);
    &PRESS
}

/// Where each `PopupAnchor` and `AnchoredPopup` panel drew last, by key.
#[derive(Clone, Copy, Debug, Default)]
struct PopupGeom {
    anchor: Area,
    anchor_rect: Rect,
    panel: Area,
}

fn popup_geoms() -> &'static Mutex<Vec<(String, PopupGeom)>> {
    static GEOMS: Mutex<Vec<(String, PopupGeom)>> = Mutex::new(Vec::new());
    &GEOMS
}

fn popup_geom(key: &str) -> Option<PopupGeom> {
    popup_geoms()
        .lock()
        .unwrap()
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, g)| *g)
}

fn popup_geom_update(key: &str, f: impl FnOnce(&mut PopupGeom)) {
    let mut geoms = popup_geoms().lock().unwrap();
    if let Some((_, g)) = geoms.iter_mut().find(|(k, _)| k == key) {
        f(g);
    } else {
        let mut g = PopupGeom::default();
        f(&mut g);
        geoms.push((key.to_string(), g));
    }
}

/// True while the app state has this popup's list open.
fn popup_open(key: &str) -> bool {
    app_state().lock().unwrap().suggest_open.map(|f| f.key()) == Some(key)
}

thread_local! {
    /// Tallest the list being drawn inside an `AnchoredPopup` may be.
    static POPUP_LIMIT: std::cell::Cell<Option<f64>> = const { std::cell::Cell::new(None) };
}

/// Inside an `AnchoredPopup` draw: the height its list must fit in.
fn popup_draw_limit() -> Option<f64> {
    POPUP_LIMIT.with(|l| l.get())
}

/// The field is on screen: at least half of it inside the page's visible
/// band (`page_viewport`: top, height). Scrolled under the header or off
/// the bottom, its list hides until it is back (the list stays open).
fn popup_anchor_on_page(anchor: Rect, page: (f64, f64)) -> bool {
    let (top, h) = page;
    if anchor.size.x < 0.5 || anchor.size.y < 0.5 {
        return false;
    }
    if h < 1.0 {
        return true;
    }
    let shown = (anchor.pos.y + anchor.size.y).min(top + h) - anchor.pos.y.max(top);
    shown >= anchor.size.y * 0.5
}

/// Where a floating list goes against its field.
#[derive(Clone, Copy, Debug, PartialEq)]
struct PopupPlace {
    pos: DVec2,
    width: f64,
    /// The panel may be at most this tall (it scrolls inside beyond that).
    max_h: f64,
    above: bool,
}

/// Under the field when the list fits there (or there is more room below
/// than above), otherwise above it; never past the window edges.
/// `want_h` is the panel's height last frame (0 before its first draw).
fn place_popup(anchor: Rect, want_h: f64, window: DVec2, gap: f64, edge: f64, min_w: f64) -> PopupPlace {
    let width = anchor.size.x.max(min_w).min((window.x - 2.0 * edge).max(1.0));
    let x = anchor
        .pos
        .x
        .min(window.x - edge - width)
        .max(edge);
    let below = (window.y - edge) - (anchor.pos.y + anchor.size.y + gap);
    let above_room = (anchor.pos.y - gap) - edge;
    let want = want_h.max(1.0);
    let above = want > below && above_room > below;
    if above {
        let max_h = above_room.max(1.0);
        let h = want.min(max_h);
        PopupPlace {
            pos: dvec2(x, anchor.pos.y - gap - h),
            width,
            max_h,
            above: true,
        }
    } else {
        PopupPlace {
            pos: dvec2(x, anchor.pos.y + anchor.size.y + gap),
            width,
            max_h: below.max(1.0),
            above: false,
        }
    }
}

/// An invisible marker inside a dropdown's field box (last child of a
/// `flow: Overlay` box, `width: Fill height: Fill`): reports where the
/// field is every frame so its `AnchoredPopup` can follow it.
#[derive(Script, ScriptHook, Widget)]
struct PopupAnchor {
    #[deref]
    view: View,
    #[live]
    key: String,
}

impl Widget for PopupAnchor {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let step = self.view.draw_walk(cx, scope, walk);
        let area = self.view.area();
        let rect = area.rect(cx);
        let mut moved = false;
        let mut panel = Area::Empty;
        popup_geom_update(&self.key, |g| {
            moved = g.anchor_rect != rect;
            g.anchor = area;
            g.anchor_rect = rect;
            panel = g.panel;
        });
        // The page scrolled or reflowed under an open list: draw the list
        // again at the field's new place (next frame at the latest).
        if moved && !panel.is_empty() && popup_open(&self.key) {
            cx.redraw_area(panel);
        }
        step
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}
}

/// A list that floats over the screen under its field (`key` = the
/// `PopupAnchor` and the `SuggestField::key` it shows). Lives at the
/// window root, after the page, so it draws last and gets events first.
/// Claims no layout space; never takes key focus or blocks scrolling.
#[derive(Script, Widget)]
struct AnchoredPopup {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[live]
    key: String,
    /// Space between the field and the list.
    #[live(4.0)]
    gap: f64,
    /// Keep this far from the window edges.
    #[live(8.0)]
    edge: f64,
    /// Narrowest the list may be (it is at least as wide as the field).
    #[live(0.0)]
    min_width: f64,
    #[rust]
    draw_list: Option<DrawList2d>,
    #[rust]
    last_h: f64,
}

impl ScriptHook for AnchoredPopup {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        self.draw_list = Some(DrawList2d::script_new(vm));
    }

    fn on_after_apply(
        &mut self,
        _vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        // Like Modal: an overlay takes no slot in its parent's layout.
        self.view.walk = Walk::empty();
    }
}

impl Widget for AnchoredPopup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !popup_open(&self.key) {
            return;
        }
        let content = self.view.widget(cx, ids!(content));
        content.handle_event(cx, event, scope);
        let area = content.area();
        // A press on the panel's own padding stays on the panel: it must
        // not reach a table tile or button drawn under the list.
        let _ = event.hits(cx, area);
        // The wheel over the list never scrolls the page behind it.
        if let Event::Scroll(e) = event
            && area_hit(cx, area, e.abs)
        {
            e.handled_x.set(true);
            e.handled_y.set(true);
        }
        // A window resize moves the field: follow it.
        if matches!(event, Event::WindowGeomChange(_)) {
            self.redraw_overlay(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, _walk: Walk) -> DrawStep {
        let open = popup_open(&self.key);
        // Only while the field itself is on screen this frame. Clipped rects
        // are not resolved until the window's root turtle ends (after this
        // draw), so test the field against the page's visible band instead.
        let anchor = popup_geom(&self.key)
            .filter(|g| g.anchor.is_valid(cx))
            .map(|g| g.anchor_rect)
            .filter(|r| popup_anchor_on_page(*r, *page_viewport().lock().unwrap()));
        let draw_list = self.draw_list.as_mut().unwrap();
        // Last overlay slot: Floor tiles / ScrollX bars must never paint over
        // the open list (Fit-height content used to leave an empty card and
        // the Floor showed through the gaps — looked like a ghost menu).
        draw_list.begin_overlay_last(cx);
        cx.begin_root_turtle_for_pass(Layout::flow_down());
        let mut panel = Area::Empty;
        if open && let Some(anchor) = anchor {
            let window = cx.current_pass_size();
            let place = place_popup(anchor, self.last_h, window, self.gap, self.edge, self.min_width);
            let content = self.view.widget(cx, ids!(content));
            let pad = self
                .view
                .view(cx, ids!(content))
                .borrow()
                .map(|v| v.layout.padding.top + v.layout.padding.bottom)
                .unwrap_or(0.0);
            // Fixed height so the RoundedView card paints an opaque fill for
            // the whole panel (Fit + PortalList left draw_bg at ~0 height —
            // only the row chips drew, Floor scrolled underneath).
            let list_limit = (place.max_h - pad).max(1.0);
            let panel_h = anchored_popup_panel_height(&self.key, list_limit, pad, self.last_h);
            let walk = Walk {
                abs_pos: Some(place.pos),
                width: Size::Fixed(place.width),
                height: Size::Fixed(panel_h),
                margin: Inset::default(),
                ..Walk::default()
            };
            POPUP_LIMIT.with(|l| l.set(Some(list_limit)));
            let _ = content.draw_walk(cx, scope, walk);
            POPUP_LIMIT.with(|l| l.set(None));
            panel = content.area();
            let h = panel.rect(cx).size.y;
            if h > 0.5 && (h - self.last_h).abs() > 0.5 {
                // Placed above with last frame's height: settle once.
                let redo = place.above;
                self.last_h = h;
                if redo {
                    cx.redraw_area(panel);
                }
            } else if h > 0.5 {
                self.last_h = h;
            }
        } else {
            self.last_h = 0.0;
        }
        cx.end_pass_sized_turtle();
        self.draw_list.as_mut().unwrap().end(cx);
        popup_geom_update(&self.key, |g| g.panel = panel);
        DrawStep::done()
    }
}

/// Opaque card height for an open floating list: prefer last frame's measure;
/// on the first open frame estimate from the current suggest rows so the
/// cream fill covers the Floor immediately (no one-frame ghost).
fn anchored_popup_panel_height(key: &str, list_limit: f64, pad: f64, last_h: f64) -> f64 {
    if last_h > 0.5 {
        return last_h.min(list_limit + pad).max(1.0);
    }
    let (n, row_h) = {
        let st = app_state().lock().unwrap();
        let n = if st.suggest_open.map(|f| f.key()) == Some(key) {
            st.suggest_rows.len()
        } else {
            0
        };
        (n, *suggest_row_height().lock().unwrap())
    };
    const SUGGEST_CAP: f64 = 330.0;
    let list_h = if n == 0 {
        48.0
    } else {
        list_content_height(n, row_h, 6.0).min(SUGGEST_CAP).min(list_limit)
    };
    (list_h + pad).min(list_limit + pad).max(1.0)
}

impl AnchoredPopup {
    /// Repaint the overlay (the derived `redraw` only reaches `view`, which
    /// never draws: the list lives on this widget's own overlay list).
    fn redraw_overlay(&mut self, cx: &mut Cx) {
        if let Some(draw_list) = &self.draw_list {
            draw_list.redraw(cx);
        }
    }
}

/// A list that is open on screen: a type-or-pick list, or one of the two
/// Menu-form pickers that keep their own open flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpenList {
    Suggest(SuggestField),
    DishPicker,
    CategoryPicker,
}

impl OpenList {
    /// What the app state has open right now (at most one).
    fn current() -> Option<OpenList> {
        let st = app_state().lock().unwrap();
        if let Some(f) = st.suggest_open {
            Some(OpenList::Suggest(f))
        } else if st.dish_picker_open {
            Some(OpenList::DishPicker)
        } else if st.category_picker_open {
            Some(OpenList::CategoryPicker)
        } else {
            None
        }
    }

    fn input(self) -> &'static [LiveId] {
        match self {
            OpenList::Suggest(f) => f.input(),
            OpenList::DishPicker => ids!(menu_name_input),
            OpenList::CategoryPicker => ids!(menu_category_input),
        }
    }

    fn list(self) -> &'static [LiveId] {
        match self {
            OpenList::Suggest(f) => f.list(),
            OpenList::DishPicker => ids!(dish_pick_list),
            OpenList::CategoryPicker => ids!(category_pick_list),
        }
    }

    /// Every list a tap on its field opens.
    fn tap_targets() -> impl Iterator<Item = OpenList> {
        SuggestField::ALL
            .into_iter()
            .filter(|f| !f.opens_by_chrome())
            .map(OpenList::Suggest)
            .chain([OpenList::DishPicker, OpenList::CategoryPicker])
    }
}

/// Every `AnchoredPopup` in the window (each draws only while open).
const APP_POPUPS: [&[LiveId]; 0] = [];

impl App {
    /// The open list's own ground: its field, its dropdown button, the list
    /// (in the page or floating), and nothing else.
    fn list_owns_point(&self, cx: &Cx, list: OpenList, abs: DVec2) -> bool {
        if area_hit(cx, self.ui.text_input(cx, list.input()).area(), abs) {
            return true;
        }
        if area_hit(cx, self.ui.widget(cx, list.list()).area(), abs) {
            return true;
        }
        if let OpenList::Suggest(field) = list {
            if let Some(opener) = field.opener()
                && area_hit(cx, self.ui.button(cx, opener).area(), abs)
            {
                return true;
            }
            if let Some(g) = popup_geom(field.key())
                && (area_hit(cx, g.anchor, abs) || area_hit(cx, g.panel, abs))
            {
                return true;
            }
        }
        false
    }

    /// The type-or-pick field under a screen point, if any.
    fn list_field_at(&self, cx: &Cx, abs: DVec2) -> Option<OpenList> {
        OpenList::tap_targets()
            .find(|l| area_hit(cx, self.ui.text_input(cx, l.input()).area(), abs))
    }

    /// The type-or-pick field holding key focus, if any.
    fn list_field_focused(&self, cx: &Cx) -> Option<OpenList> {
        OpenList::tap_targets().find(|l| {
            let area = self.ui.text_input(cx, l.input()).area();
            !area.is_empty() && cx.has_key_focus(area)
        })
    }

    /// Open a list from a tap on its field or ↓ — never from focus alone.
    fn open_list_from_user(&mut self, cx: &mut Cx, list: OpenList) {
        if OpenList::current() == Some(list) {
            return;
        }
        let text = self.ui.text_input(cx, list.input()).text();
        {
            let mut st = app_state().lock().unwrap();
            match list {
                OpenList::Suggest(field) => st.open_suggest(field, &text),
                OpenList::DishPicker => {
                    st.suggest_open = None;
                    st.category_picker_open = false;
                    st.dish_picker_open = true;
                    st.refresh_dish_picks(&text);
                }
                OpenList::CategoryPicker => {
                    st.suggest_open = None;
                    st.dish_picker_open = false;
                    st.category_picker_open = true;
                    st.refresh_category_picks(&text);
                }
            }
        }
        self.sync_chrome(cx);
        self.redraw_suggest_lists(cx);
    }

    /// Close whatever list is open. True when one closed.
    fn close_open_list(&mut self, cx: &mut Cx) -> bool {
        let Some(list) = OpenList::current() else {
            return false;
        };
        self.force_close_list(cx, list);
        true
    }

    /// Back / Escape / screen change: close any open list, including ones
    /// that stay open through an outside press (Team stations checklist).
    fn dismiss_open_list_for_nav(&mut self, cx: &mut Cx) -> bool {
        let Some(list) = OpenList::current() else {
            return false;
        };
        self.force_close_list(cx, list);
        true
    }

    fn force_close_list(&mut self, cx: &mut Cx, list: OpenList) {
        {
            let mut st = app_state().lock().unwrap();
            match list {
                OpenList::Suggest(_) => st.suggest_open = None,
                OpenList::DishPicker => st.dish_picker_open = false,
                OpenList::CategoryPicker => st.category_picker_open = false,
            }
        }
        self.ui.widget(cx, list.list()).redraw(cx);
        self.redraw_popups(cx);
    }

    fn redraw_suggest_lists(&mut self, cx: &mut Cx) {
        for field in SuggestField::ALL {
            self.ui.widget(cx, field.list()).redraw(cx);
        }
        self.ui.widget(cx, ids!(dish_pick_list)).redraw(cx);
        self.ui.widget(cx, ids!(category_pick_list)).redraw(cx);
        self.redraw_popups(cx);
    }

    fn redraw_popups(&mut self, cx: &mut Cx) {
        for path in APP_POPUPS {
            if let Some(mut popup) = self.ui.widget(cx, path).borrow_mut::<AnchoredPopup>() {
                popup.redraw_overlay(cx);
            }
        }
    }

    /// Rules 2 and 3 for one input event, before the UI or any pointer
    /// handler sees it (a nav-tab tap returns early from those and used to
    /// leave the list open on the next screen).
    fn route_popup_input(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::KeyDown(k) = event {
            // ↓ in a focused type-or-pick field shows its list.
            if k.key_code == KeyCode::ArrowDown
                && !k.modifiers.shift
                && !k.modifiers.control
                && !k.modifiers.alt
                && !k.modifiers.logo
                && OpenList::current().is_none()
                && let Some(list) = self.list_field_focused(cx)
            {
                self.open_list_from_user(cx, list);
            }
            return;
        }
        let Some(pointer) = popup_pointer(event) else {
            return;
        };
        match pointer {
            PopupPointer::Press { abs, time } => {
                // An ℹ tip closes on the next press anywhere. A finger never
                // sends the hover-out that hides it with a mouse, so on a
                // touch screen a tip stayed up until another tip replaced it.
                // A tap on an ℹ shows its tip again on release.
                self.ui.callout_tooltip(cx, ids!(app_callout)).hide(cx);
                if let Some(list) = OpenList::current()
                    && !self.list_owns_point(cx, list, abs)
                {
                    self.close_open_list(cx);
                }
                *popup_press().lock().unwrap() = self
                    .list_field_at(cx, abs)
                    .map(|list| PopupPress { list, abs, time });
            }
            PopupPointer::Move { abs } => {
                let mut press = popup_press().lock().unwrap();
                if press.is_some_and(|p| (abs - p.abs).length() >= POPUP_TAP_TRAVEL) {
                    // A drag (page scroll, text selection) is not a tap.
                    *press = None;
                }
            }
            PopupPointer::Release { abs, time } => {
                let press = popup_press().lock().unwrap().take();
                if let Some(p) = press
                    && popup_is_tap(p.abs, p.time, abs, time)
                    && self.list_field_at(cx, abs) == Some(p.list)
                {
                    self.open_list_from_user(cx, p.list);
                }
            }
        }
    }
}

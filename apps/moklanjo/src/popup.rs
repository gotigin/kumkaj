// Type-or-pick lists: one rule set for every one of them, for a mouse
// (with or without a wheel), a trackpad, and a finger — DEV.md § Dropdowns.
// (Khaynjo keeps its own copy, with a floating `AnchoredPopup`; Moklanjo's
// lists all sit in the page under their field, so it needs only the rules.)
//
// 1. **The app state is the only switch.** A list is open while
//    `suggest_open` names it; every list reads that at draw time.
// 2. **A tap opens, focus never does.** A press and release on the field
//    opens its list, also when the field already has focus. Typing opens
//    it narrowed; ↓ opens it from the keyboard. Focus alone opens nothing
//    (a focus handed back by a closing sheet must not reopen a list).
// 3. **One router closes.** A press anywhere outside the open list and its
//    field closes it before the press is delivered; the press still
//    reaches whatever is under it. Before this, nothing closed a list on a
//    touch screen except picking from it. A pick closes it; Back / Escape
//    close it; leaving the screen closes it.
// 4. **Tips too.** An ℹ callout closes on the next press anywhere (a
//    finger never sends the hover-out a mouse does).
// 5. **Screen rects only.** Pointer positions are tested against
//    `clipped_rect` — the rect Makepad's own hit test uses — never `rect()`.

/// The on-screen part of an area, as Makepad's hit test sees it: clipped by
/// every scroll view around it. `None` when it is not on screen.
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
/// a text selection (24 px like `finger_up_is_tap`; a long press on a touch
/// field starts selection).
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
    field: SuggestField,
    abs: DVec2,
    time: f64,
}

fn popup_press() -> &'static Mutex<Option<PopupPress>> {
    static PRESS: Mutex<Option<PopupPress>> = Mutex::new(None);
    &PRESS
}

impl App {
    /// The open list's own ground: its field and the list, nothing else.
    fn suggest_owns_point(&self, cx: &Cx, field: SuggestField, abs: DVec2) -> bool {
        area_hit(cx, self.ui.text_input(cx, field.input()).area(), abs)
            || area_hit(cx, self.ui.widget(cx, field.list()).area(), abs)
    }

    /// The type-or-pick field under a screen point, if any.
    fn suggest_field_at(&self, cx: &Cx, abs: DVec2) -> Option<SuggestField> {
        SuggestField::ALL
            .into_iter()
            .find(|f| area_hit(cx, self.ui.text_input(cx, f.input()).area(), abs))
    }

    /// The type-or-pick field holding key focus, if any.
    fn suggest_field_focused(&self, cx: &Cx) -> Option<SuggestField> {
        SuggestField::ALL.into_iter().find(|f| {
            let area = self.ui.text_input(cx, f.input()).area();
            !area.is_empty() && cx.has_key_focus(area)
        })
    }

    /// Open (or re-narrow) a field's list with `text` as the query. Parties
    /// form: State lists the states of the country typed beside it.
    fn open_suggest_with(&mut self, cx: &mut Cx, field: SuggestField, text: &str) {
        let party_country = match field {
            SuggestField::PartyCountry => text.to_string(),
            SuggestField::PartyState => self.ui.text_input(cx, ids!(party_country_input)).text(),
            _ => String::new(),
        };
        let mut st = app_state().lock().unwrap();
        if matches!(field, SuggestField::PartyCountry | SuggestField::PartyState) {
            st.party_form_country = party_country;
        }
        st.open_suggest(field, text);
    }

    /// Open a list from a tap on its field or ↓ — never from focus alone.
    fn open_suggest_from_user(&mut self, cx: &mut Cx, field: SuggestField) {
        if app_state().lock().unwrap().suggest_open == Some(field) {
            return;
        }
        let text = self.ui.text_input(cx, field.input()).text();
        self.open_suggest_with(cx, field, &text);
        self.sync_chrome(cx);
        self.redraw_suggest_lists(cx);
    }

    /// Close whatever list is open. True when one closed.
    fn close_open_suggest(&mut self, cx: &mut Cx) -> bool {
        let Some(field) = app_state().lock().unwrap().suggest_open.take() else {
            return false;
        };
        self.ui.widget(cx, field.list()).redraw(cx);
        true
    }

    fn redraw_suggest_lists(&mut self, cx: &mut Cx) {
        for field in SuggestField::ALL {
            self.ui.widget(cx, field.list()).redraw(cx);
        }
    }

    /// Rules 2 and 3 for one input event, before the UI or any pointer
    /// handler sees it (a nav-tab tap returns early from those).
    fn route_popup_input(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::KeyDown(k) = event {
            // ↓ in a focused type-or-pick field shows its list.
            if k.key_code == KeyCode::ArrowDown
                && !k.modifiers.shift
                && !k.modifiers.control
                && !k.modifiers.alt
                && !k.modifiers.logo
                && app_state().lock().unwrap().suggest_open.is_none()
                && let Some(field) = self.suggest_field_focused(cx)
            {
                self.open_suggest_from_user(cx, field);
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
                let open = app_state().lock().unwrap().suggest_open;
                if let Some(field) = open
                    && !self.suggest_owns_point(cx, field, abs)
                {
                    self.close_open_suggest(cx);
                }
                *popup_press().lock().unwrap() = self
                    .suggest_field_at(cx, abs)
                    .map(|field| PopupPress { field, abs, time });
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
                    && self.suggest_field_at(cx, abs) == Some(p.field)
                {
                    self.open_suggest_from_user(cx, p.field);
                }
            }
        }
    }
}

#[cfg(test)]
mod popup_tests {
    use super::*;

    /// Press → release is a tap only within 24 px and under 0.8 s: a page
    /// swipe that starts on a field, or a long-press text selection, must
    /// not open its list.
    #[test]
    fn only_a_tap_opens_a_list() {
        assert!(popup_is_tap(dvec2(10.0, 10.0), 1.0, dvec2(20.0, 15.0), 1.2));
        assert!(!popup_is_tap(dvec2(10.0, 10.0), 1.0, dvec2(10.0, 60.0), 1.1));
        assert!(!popup_is_tap(dvec2(10.0, 10.0), 1.0, dvec2(10.0, 10.0), 2.0));
    }

    /// Mouse and the first finger map to the same three phases; other mouse
    /// buttons (Back / Forward) are not presses on a list.
    #[test]
    fn mouse_and_finger_share_one_press_path() {
        let down = Event::MouseDown(MouseDownEvent {
            abs: dvec2(5.0, 6.0),
            button: MouseButton::PRIMARY,
            window_id: WindowId(0, 0),
            modifiers: Default::default(),
            handled: Default::default(),
            time: 2.0,
        });
        assert_eq!(
            popup_pointer(&down),
            Some(PopupPointer::Press { abs: dvec2(5.0, 6.0), time: 2.0 })
        );
    }
}

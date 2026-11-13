// Text selection & copy — one model for every Khaynjo screen
// (docs/DESIGN.md § Text selection & copy, docs/DEV.md § Text selection).
//
// 1. **Display text selects like a browser page.** A `WrapLabel` / `ReportText`
//    row is a selectable TextFlow: a mouse drag selects and runs on across
//    blocks (the root carries it; wheel and the window edge scroll under it);
//    double-click a word, triple-click the block; a finger long-presses a
//    word, then drags. Text inside a tap target (`selectable: false`) is not
//    copy. A drag on whitespace scrolls the page (apps), as before.
// 2. **One root per window: `main_column` (`selection_root: true`).**
//    Ctrl/Cmd+A — when no text field has focus — selects every selectable
//    block the screen shows (header and Today bar included); Ctrl/Cmd+C then
//    copies them in reading order. A text field keeps its own Ctrl+A / Copy.
// 3. **Any press clears a page-wide Select all**, and so does every screen
//    change; a block's own selection clears when focus leaves it.
// 4. **The menu.** Right-click (mouse) or the end of a long-press selection
//    (touch, where the OS has no bar of its own) opens a small Copy / Select
//    all menu at the pointer. Copy runs the same path as Ctrl/Cmd+C (a
//    `TextCopy` through the UI), taken when the menu opens. Android shows the
//    OS Copy / Select all bar for touch instead; its Select all is Ctrl+A.
// 5. **Same rules as a dropdown** (popup.rs): open state lives in app state
//    and is read at draw time; a press outside closes it (the press still
//    reaches what is under it); Escape, Back and any screen change close it.

/// Where the menu goes: at a pointer (right-click), or above a touch
/// selection.
#[derive(Clone, Copy, Debug)]
enum SelMenuAnchor {
    Point(DVec2),
    Above(Rect),
}

impl Default for SelMenuAnchor {
    fn default() -> Self {
        Self::Point(DVec2::default())
    }
}

/// The open Copy / Select all menu (`AppState::sel_menu`).
#[derive(Clone, Debug, Default)]
struct SelMenuOpen {
    /// Where the menu opened.
    abs: SelMenuAnchor,
    /// What Copy puts on the clipboard (empty → Copy is disabled).
    copy_text: String,
}

const SEL_MENU_W: f64 = 208.0;
const SEL_MENU_EDGE: f64 = 8.0;

/// Copy / Select all over the window, at `AppState::sel_menu.abs`. Lives at
/// the window root after the page so it draws last and gets events first.
#[derive(Script, Widget)]
struct SelectionMenu {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[rust]
    draw_list: Option<DrawList2d>,
    #[rust]
    last_h: f64,
}

impl ScriptHook for SelectionMenu {
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
        // An overlay takes no slot in its parent's layout.
        self.view.walk = Walk::empty();
    }
}

fn sel_menu_open() -> bool {
    app_state().lock().unwrap().sel_menu.is_some()
}

/// Where the menu goes: at the pointer, flipped left / up to stay inside.
fn sel_menu_place(anchor: SelMenuAnchor, h: f64, window: DVec2) -> DVec2 {
    let abs = match anchor {
        SelMenuAnchor::Point(abs) => abs,
        SelMenuAnchor::Above(rect) => {
            // Clear of the selection and of the knobs that hang under it.
            const GAP: f64 = 10.0;
            const KNOBS: f64 = 30.0;
            let x = (rect.pos.x + rect.size.x * 0.5 - SEL_MENU_W * 0.5)
                .min(window.x - SEL_MENU_W - SEL_MENU_EDGE)
                .max(SEL_MENU_EDGE);
            let above = rect.pos.y - GAP - h;
            let y = if above >= SEL_MENU_EDGE {
                above
            } else {
                (rect.pos.y + rect.size.y + KNOBS)
                    .min(window.y - h - SEL_MENU_EDGE)
                    .max(SEL_MENU_EDGE)
            };
            return dvec2(x, y);
        }
    };
    let x = if abs.x + SEL_MENU_W + SEL_MENU_EDGE > window.x {
        (abs.x - SEL_MENU_W).max(SEL_MENU_EDGE)
    } else {
        abs.x
    };
    let y = if abs.y + h + SEL_MENU_EDGE > window.y {
        (abs.y - h).max(SEL_MENU_EDGE)
    } else {
        abs.y
    };
    dvec2(x.max(SEL_MENU_EDGE), y.max(SEL_MENU_EDGE))
}

impl Widget for SelectionMenu {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !sel_menu_open() {
            return;
        }
        let content = self.view.widget(cx, ids!(content));
        content.handle_event(cx, event, scope);
        let area = content.area();
        // A press on the card's padding stays on the card.
        let _ = event.hits(cx, area);
        if let Event::Scroll(e) = event
            && area_hit(cx, area, e.abs)
        {
            e.handled_x.set(true);
            e.handled_y.set(true);
        }
        if matches!(event, Event::WindowGeomChange(_)) {
            self.redraw_overlay(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, _walk: Walk) -> DrawStep {
        let at = app_state().lock().unwrap().sel_menu.as_ref().map(|m| m.abs);
        let draw_list = self.draw_list.as_mut().unwrap();
        draw_list.begin_overlay_last(cx);
        cx.begin_root_turtle_for_pass(Layout::flow_down());
        if let Some(abs) = at {
            let window = cx.current_pass_size();
            let guess = if self.last_h > 0.5 { self.last_h } else { 104.0 };
            let pos = sel_menu_place(abs, guess, window);
            let content = self.view.widget(cx, ids!(content));
            let walk = Walk {
                abs_pos: Some(pos),
                width: Size::Fixed(SEL_MENU_W),
                height: Size::fit(),
                margin: Inset::default(),
                ..Walk::default()
            };
            let _ = content.draw_walk(cx, scope, walk);
            let h = content.area().rect(cx).size.y;
            if h > 0.5 && (h - self.last_h).abs() > 0.5 {
                self.last_h = h;
                // Placed with a guessed height: settle once.
                cx.redraw_area(content.area());
            }
        } else {
            self.last_h = 0.0;
        }
        cx.end_pass_sized_turtle();
        self.draw_list.as_mut().unwrap().end(cx);
        DrawStep::done()
    }
}

impl SelectionMenu {
    fn redraw_overlay(&mut self, cx: &mut Cx) {
        if let Some(draw_list) = &self.draw_list {
            draw_list.redraw(cx);
        }
    }
}

/// True where the platform shows its own Copy / Select all bar for a touch
/// selection (TextFlow already asked for it).
fn os_has_touch_selection_bar() -> bool {
    cfg!(any(target_os = "android", target_os = "ios"))
}

impl App {
    /// What Ctrl/Cmd+C would copy right now: the focused block's selection,
    /// or every block after a page-wide Select all.
    fn selection_copy_text(&mut self, cx: &mut Cx) -> String {
        let response = std::rc::Rc::new(std::cell::RefCell::new(None));
        let event = Event::TextCopy(TextClipboardEvent {
            response: response.clone(),
        });
        self.ui.handle_event(cx, &event, &mut Scope::empty());
        response.borrow().clone().unwrap_or_default()
    }

    fn open_selection_menu(&mut self, cx: &mut Cx, abs: SelMenuAnchor) {
        let copy_text = self.selection_copy_text(cx);
        let (copy_label, all_label) = {
            let mut st = app_state().lock().unwrap();
            st.sel_menu = Some(SelMenuOpen {
                abs,
                copy_text: copy_text.clone(),
            });
            (st.t("sel_copy"), st.t("sel_select_all"))
        };
        let copy = self.ui.button(cx, ids!(sel_menu_copy));
        copy.set_text(cx, &copy_label);
        copy.set_enabled(cx, !copy_text.is_empty());
        self.ui
            .button(cx, ids!(sel_menu_all))
            .set_text(cx, &all_label);
        self.redraw_selection_menu(cx);
    }

    /// The menu lives on its own overlay draw list: repaint that list.
    fn redraw_selection_menu(&mut self, cx: &mut Cx) {
        if let Some(mut menu) = self.ui.widget(cx, ids!(sel_menu)).borrow_mut::<SelectionMenu>() {
            menu.redraw_overlay(cx);
        }
    }

    /// Close the menu; true when it was open.
    fn close_selection_menu(&mut self, cx: &mut Cx) -> bool {
        let was = app_state().lock().unwrap().sel_menu.take().is_some();
        if was {
            self.redraw_selection_menu(cx);
        }
        was
    }

    /// The surface Ctrl/Cmd+A belongs to now: an open sheet, else the page.
    fn active_selection_root(&self, cx: &Cx) -> ViewRef {
        if app_state().lock().unwrap().menu_pick_open {
            self.ui.view(cx, ids!(menu_panel))
        } else if self.ui.modal(cx, ids!(upi_qr_modal)).is_open() {
            self.ui.view(cx, ids!(upi_qr_sheet))
        } else {
            self.ui.view(cx, ids!(main_column))
        }
    }

    /// Every screen change: drop a page-wide Select all and the menu.
    fn clear_page_selection(&mut self, cx: &mut Cx) {
        for root in [ids!(main_column), ids!(menu_panel), ids!(upi_qr_sheet)] {
            self.ui.view(cx, root).selection_root_clear(cx);
        }
        self.close_selection_menu(cx);
    }

    /// Rule 5, before the UI sees the event: a press outside the open menu
    /// closes it (the press still lands); Escape closes it.
    fn route_selection_menu_input(&mut self, cx: &mut Cx, event: &Event) {
        if !sel_menu_open() {
            return;
        }
        let press = match event {
            Event::MouseDown(e) => Some(e.abs),
            Event::TouchUpdate(e) => e
                .touches
                .iter()
                .find(|t| matches!(t.state, TouchState::Start))
                .map(|t| t.abs),
            Event::KeyDown(k) if k.key_code == KeyCode::Escape => {
                self.close_selection_menu(cx);
                return;
            }
            _ => None,
        };
        if let Some(abs) = press {
            let card = self
                .ui
                .widget(cx, ids!(sel_menu))
                .widget(cx, ids!(content))
                .area();
            if !area_hit(cx, card, abs) {
                self.close_selection_menu(cx);
            }
        }
    }

    /// Menu requests from the selection root (right-click) and from a
    /// TextFlow (end of a touch selection), and the menu's two rows.
    fn handle_selection_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for wa in actions.iter().filter_map(|a| a.as_widget_action()) {
            // `cx.widget_action` wraps the payload: unwrap it from WidgetAction.
            if let TextSelectionAction::Menu { abs, touch, rect } = wa.cast::<TextSelectionAction>() {
                if touch && os_has_touch_selection_bar() {
                    continue;
                }
                log!("Kumkaj selection: menu at {abs:?} (touch={touch})");
                // Touch: above the selection and centred on it, below the
                // knobs when there is no room (a browser's selection bar).
                let anchor = if touch {
                    SelMenuAnchor::Above(rect)
                } else {
                    SelMenuAnchor::Point(abs)
                };
                self.open_selection_menu(cx, anchor);
            }
        }
        if self.ui.button(cx, ids!(sel_menu_copy)).clicked(actions) {
            let text = app_state()
                .lock()
                .unwrap()
                .sel_menu
                .as_ref()
                .map(|m| m.copy_text.clone())
                .unwrap_or_default();
            self.close_selection_menu(cx);
            if !text.is_empty() {
                // Same pair as every other Copy in the app (Makepad + arboard).
                cx.copy_to_clipboard(&text);
                let _ = print_share::copy_text(&text);
                self.show_snack(cx, SnackKind::Info, app_t("sel_copied"));
            }
        }
        if self.ui.button(cx, ids!(sel_menu_all)).clicked(actions) {
            self.close_selection_menu(cx);
            self.active_selection_root(cx).selection_root_select_all(cx);
        }
    }
}

/// The screen the selection root last showed (a page-wide Select all and an
/// open menu never carry onto the next screen; a repaint of the same screen
/// keeps them).
fn sel_screen_changed(screen: Screen) -> bool {
    static LAST: Mutex<Option<Screen>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap();
    let changed = *last != Some(screen);
    *last = Some(screen);
    changed
}

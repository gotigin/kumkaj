/// Selectable display text (Markdown) with Label-like `visible` support.
/// Stock Markdown/TextFlow ignore `set_visible`, which left empty-state copy
/// on screen after products/cart lines appeared.
#[derive(Script, ScriptHook, Widget)]
struct WrapLabel {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    markdown: Markdown,
    #[live(true)]
    #[visible]
    visible: bool,
}

impl Widget for WrapLabel {
    fn is_interactive(&self) -> bool {
        self.visible && self.markdown.text_flow.selectable
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.visible && event.requires_visibility() {
            return;
        }
        self.markdown.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        // Keep scroll blocked across redraws while a drag-select is live.
        if self.markdown.text_flow.is_drag_selecting() {
            cx.block_scrolling_except_within(Area::Empty);
        }
        self.markdown.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.markdown.text()
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        self.markdown.set_text(cx, v);
    }
}

impl WrapLabel {
    /// Right-to-left support: paragraph alignment inside the flow
    /// (vendor patch: `TextFlow::paragraph_align_x`).
    pub fn set_text_align_x(&mut self, x: f64) {
        self.markdown.text_flow.paragraph_align_x = x;
    }
}

/// Title row with an info control. Emits [`TooltipAction`] so the window
/// [`CalloutTooltip`] can place a callout without covering the icon
/// (below by default; flips / clamps when space is tight).
#[derive(Script, ScriptHook, Widget)]
struct InfoHelp {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// Body shown in the callout (locale string, may be multi-line).
    #[live]
    tip: String,
    #[rust]
    hovered: bool,
}

impl Widget for InfoHelp {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let claimed_before = if let Event::MouseMove(_) = event {
            event.pointer_claimed_area()
        } else {
            Area::Empty
        };
        self.view.handle_event(cx, event, scope);

        if self.tip.is_empty() {
            return;
        }

        let uid = self.widget_uid();
        if let Event::Actions(actions) = event {
            if self.view.button(cx, ids!(tip_btn)).clicked(actions) {
                let rect = self.view.button(cx, ids!(tip_btn)).area().rect(cx);
                cx.widget_action(
                    uid,
                    TooltipAction::HoverIn {
                        text: self.tip.clone(),
                        widget_rect: rect,
                        options: info_help_tooltip_options(rect),
                    },
                );
            }
        }

        match event {
            Event::MouseMove(_) => {
                let claimed_after = event.pointer_claimed_area();
                let over = claimed_after != claimed_before && !claimed_after.is_empty();
                if over != self.hovered {
                    self.hovered = over;
                    if over {
                        let rect = self.view.button(cx, ids!(tip_btn)).area().rect(cx);
                        cx.widget_action(
                            uid,
                            TooltipAction::HoverIn {
                                text: self.tip.clone(),
                                widget_rect: rect,
                                options: info_help_tooltip_options(rect),
                            },
                        );
                    } else {
                        cx.widget_action(uid, TooltipAction::HoverOut);
                    }
                }
            }
            Event::MouseLeave(_) | Event::ClearHover => {
                if self.hovered {
                    self.hovered = false;
                    cx.widget_action(uid, TooltipAction::HoverOut);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.tip.clone()
    }

    fn set_text(&mut self, _cx: &mut Cx, v: &str) {
        self.tip = v.to_string();
    }
}

impl InfoHelpRef {
    fn set_title(&self, cx: &mut Cx, title: &str) {
        self.label(cx, ids!(title)).set_text(cx, title);
    }

    fn set_nav_icon(&self, cx: &mut Cx, tab: NavTab) {
        load_nav_icon_once(cx, &self.image(cx, ids!(icon)), tab);
    }

    fn set_tip(&self, tip: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.tip = tip.to_string();
        }
    }
}

/// Decode a nav WebP into `img` only when the tab (or empty image) changed.
/// Re-loading + `redraw` on every chrome sync scheduled endless frames.
/// Returns `true` when bytes were freshly loaded (caller may need a
/// follow-up redraw so the texture is visible).
fn load_nav_icon_once(cx: &mut Cx, img: &ImageRef, tab: NavTab) -> bool {
    static LOADED: OnceLock<Mutex<std::collections::HashMap<u64, &'static str>>> = OnceLock::new();
    let loaded = LOADED.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let uid = img.widget_uid().0;
    let key = nav_tab_key(tab);
    {
        let map = loaded.lock().unwrap();
        if map.get(&uid).copied() == Some(key) {
            img.set_visible(cx, true);
            return false;
        }
    }
    let _ = img.load_image_from_data(cx, nav_tab_icon_bytes(tab));
    loaded.lock().unwrap().insert(uid, key);
    img.set_visible(cx, true);
    true
}

fn info_help_tooltip_options(widget_rect: Rect) -> CalloutTooltipOptions {
    // Prefer below (toolbar-like). If the icon sits near the right edge,
    // open to the left so a wide tip stays on-screen; near the left edge,
    // open to the right. CalloutTooltip still flips/clamps within that side.
    let icon_right = widget_rect.pos.x + widget_rect.size.x;
    let position = if widget_rect.pos.x < 120.0 {
        TooltipPosition::Right
    } else if icon_right > 640.0 {
        // Typical phone/tablet width; CalloutTooltip also clamps to the pass.
        TooltipPosition::Left
    } else {
        TooltipPosition::Bottom
    };
    CalloutTooltipOptions {
        position,
        text_color: vec4(1.0, 1.0, 1.0, 1.0),
        bg_color: vec4(0.027, 0.125, 0.345, 0.97), // #072058
        triangle_height: 8.0,
    }
}

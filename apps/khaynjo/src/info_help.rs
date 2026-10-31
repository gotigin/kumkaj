/// Title row with an info control. Emits [`TooltipAction`] so the window
/// [`CalloutTooltip`] can place a callout without covering the icon.
#[derive(Script, ScriptHook, Widget)]
struct InfoHelp {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
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

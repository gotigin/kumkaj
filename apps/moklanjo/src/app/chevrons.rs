impl App {
    /// ‹ shows only while the strip can scroll left, › only while it can
    /// scroll right. Changes only what changed (a set_visible redraws).
    fn sync_tab_chevrons(&mut self, cx: &mut Cx) {
        for (wrap, scroller) in TAB_STRIPS {
            let tabs = self.ui.view(cx, scroller);
            let x = tabs.get_scroll_pos().x;
            let total = tabs.get_scroll_view_total().x;
            let seen = tabs.get_scroll_view_visible().x;
            let can_left = x > 1.0;
            let can_right = total > seen + 1.0 && x + seen < total - 1.0;
            let wrap = self.ui.view(cx, wrap);
            for (chev, want) in [(ids!(chev_l), can_left), (ids!(chev_r), can_right)] {
                let b = wrap.button(cx, chev);
                if b.visible() != want {
                    b.set_visible(cx, want);
                }
            }
        }
    }

    /// A chevron scrolls its strip by most of what is visible.
    fn tab_chevron_clicks(&mut self, cx: &mut Cx, actions: &Actions) {
        for (wrap, scroller) in TAB_STRIPS {
            let wrap_view = self.ui.view(cx, wrap);
            let left = wrap_view.button(cx, ids!(chev_l)).clicked(actions);
            let right = wrap_view.button(cx, ids!(chev_r)).clicked(actions);
            if !left && !right {
                continue;
            }
            let tabs = self.ui.view(cx, scroller);
            let pos = tabs.get_scroll_pos();
            let total = tabs.get_scroll_view_total().x;
            let seen = tabs.get_scroll_view_visible().x;
            let step = (seen * 0.7).max(80.0);
            let max_x = (total - seen).max(0.0);
            let x = if left {
                (pos.x - step).max(0.0)
            } else {
                (pos.x + step).min(max_x)
            };
            tabs.set_scroll_pos(cx, dvec2(x, pos.y));
            tabs.redraw(cx);
            self.sync_tab_chevrons(cx);
        }
    }
}

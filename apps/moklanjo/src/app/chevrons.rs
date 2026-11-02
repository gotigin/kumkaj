impl App {
    /// Chevron face width (`ChevronBtn`). Overlayed on strip ends — scroll
    /// targets must clear this or a tab sits forever under ‹/›.
    const TAB_CHEVRON_W: f64 = 46.0;

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

    /// Chevron scrolls bring the next partially/fully hidden chip into the
    /// clear band between ‹ and ›. A fixed fraction of the viewport used to
    /// jump over tabs on phones so some never sat fully on screen (2026-11).
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
            let max_x = (total - seen).max(0.0);
            let clear = Self::TAB_CHEVRON_W + 4.0;
            let x = if scroller == ids!(bottom_nav_scroll) {
                self.nav_bar_chevron_snap_x(cx, left, pos.x, seen, max_x, clear)
            } else {
                let usable = (seen - 2.0 * clear).max(64.0);
                let step = (usable * 0.5).clamp(64.0, usable);
                if left {
                    (pos.x - step).max(0.0)
                } else {
                    (pos.x + step).min(max_x)
                }
            };
            tabs.set_scroll_pos(cx, dvec2(x, pos.y));
            tabs.redraw(cx);
            self.sync_tab_chevrons(cx);
        }
    }

    /// Next bottom-nav tab into the clear band between chevrons (content x).
    fn nav_bar_chevron_snap_x(
        &mut self,
        cx: &mut Cx,
        go_left: bool,
        scroll_x: f64,
        seen: f64,
        max_x: f64,
        clear: f64,
    ) -> f64 {
        let fallback = || {
            let usable = (seen - 2.0 * clear).max(64.0);
            let step = (usable * 0.5).clamp(64.0, usable);
            if go_left {
                (scroll_x - step).max(0.0)
            } else {
                (scroll_x + step).min(max_x)
            }
        };
        let scroll_view = self.ui.view(cx, ids!(bottom_nav_scroll));
        let view = scroll_view.area().rect(cx);
        if view.size.x <= 1.0 {
            return fallback();
        }
        let wraps = self.nav_slot_wrappers(false);
        let mut edges: Vec<(f64, f64)> = Vec::with_capacity(wraps.len());
        for path in wraps {
            let r = self.ui.view(cx, path).area().rect(cx);
            if r.size.x <= 1.0 {
                continue;
            }
            let left = r.pos.x - view.pos.x + scroll_x;
            edges.push((left, left + r.size.x));
        }
        if edges.is_empty() {
            return fallback();
        }
        let clear_left = scroll_x + clear;
        let clear_right = scroll_x + seen - clear;
        let target = if go_left {
            edges.iter().rev().find_map(|&(l, r)| {
                if r < clear_left - 1.0 || l < clear_left - 1.0 {
                    Some((l - clear).max(0.0))
                } else {
                    None
                }
            })
        } else {
            edges.iter().find_map(|&(l, r)| {
                if l > clear_right + 1.0 || r > clear_right + 1.0 {
                    Some((l - clear).clamp(0.0, max_x))
                } else {
                    None
                }
            })
        };
        target.unwrap_or_else(fallback).clamp(0.0, max_x)
    }
}

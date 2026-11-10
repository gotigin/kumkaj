#[cfg(test)]
mod order_desk_tests {
    use super::*;

    const MINS: (f64, f64, f64, f64) = (600.0, 520.0, 16.0, 28.0);

    fn panes(view_w: f64) -> (f64, f64) {
        order_desk_pane_widths(view_w, MINS.0, MINS.1, MINS.2, MINS.3)
    }

    /// Order | Counter sit side by side at every width: on a phone each pane
    /// is one screen less a peek of the other (never a 600 px pane inside a
    /// 411 px screen, which cut Add dishes off the right edge); on a tablet
    /// each keeps its min and the desk pans; when both fit they share.
    #[test]
    fn order_panes_fit_a_phone_and_share_a_desktop() {
        // 411 px phone: the desk shows 379 px.
        assert_eq!(panes(379.0), (351.0, 351.0));
        // Small tablet: Order capped at the screen, Counter keeps its min.
        assert_eq!(panes(560.0), (532.0, 520.0));
        // Both mins do not fit but each fits the screen: mins, pan sideways.
        assert_eq!(panes(1000.0), (600.0, 520.0));
        // Wide: Order takes its share of the room, Counter the rest less the gap.
        let (l, r) = panes(1568.0);
        let room = 1568.0 - 32.0;
        assert!((l - room * 600.0 / 1120.0).abs() < 1e-9, "{l}");
        assert!((l + MINS.2 + r - room).abs() < 1e-9, "{r}");
        // The layout floor never gives a pane less than a usable width.
        let (l, r) = panes(288.0);
        assert!(l >= 240.0 && r >= 240.0, "{l} {r}");
    }
}

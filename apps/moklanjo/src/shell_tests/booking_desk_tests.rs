#[cfg(test)]
mod booking_desk_tests {
    use super::*;

    const MINS: (f64, f64, f64, f64) = (600.0, 520.0, 16.0, 28.0);

    fn panes(view_w: f64) -> (f64, f64) {
        booking_desk_pane_widths(view_w, MINS.0, MINS.1, MINS.2, MINS.3)
    }

    /// Parcel | Counter sit side by side at every width: on a phone each
    /// pane is one screen less a peek of the other (never a 600 px form
    /// inside a 360 px screen); on a tablet each keeps its min and the desk
    /// pans; when both fit they share the room.
    #[test]
    fn panes_sit_side_by_side_from_phone_to_desktop() {
        // 390 px phone: the desk shows 358 px.
        assert_eq!(panes(358.0), (330.0, 330.0));
        // Small tablet: Parcel capped at the screen, Counter keeps its min.
        assert_eq!(panes(560.0), (532.0, 520.0));
        // Both mins do not fit but each fits the screen: mins, pan sideways.
        assert_eq!(panes(1000.0), (600.0, 520.0));
        // Wide: Parcel takes its share of the room, Counter the rest less the gap.
        let (l, r) = panes(1568.0);
        let room = 1568.0 - 32.0;
        assert!((l - room * 600.0 / 1120.0).abs() < 1e-9, "{l}");
        assert!((l + MINS.2 + r - room).abs() < 1e-9, "{r}");
        // The layout floor never gives a pane less than a usable width.
        let (l, r) = panes(288.0);
        assert!(l >= 240.0 && r >= 240.0, "{l} {r}");
    }
}

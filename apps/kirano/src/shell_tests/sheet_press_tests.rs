#[cfg(test)]
mod sheet_press_tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect {
            pos: dvec2(x, y),
            size: dvec2(w, h),
        }
    }

    /// Phone window 390 × 800: the sheet is inset 16 at the sides and 20 at
    /// top and bottom; Done is its top-right corner.
    #[test]
    fn add_dishes_sheet_sits_inside_the_window() {
        let sheet = menu_sheet_fallback(dvec2(390.0, 800.0));
        assert_eq!(sheet, rect(16.0, 20.0, 358.0, 760.0));
        let wide = menu_sheet_fallback(dvec2(1600.0, 900.0));
        assert_eq!(wide.size.x, 980.0);
        assert_eq!(wide.pos.x, 310.0);
        let done = menu_done_corner(sheet);
        assert_eq!(done, rect(198.0, 20.0, 176.0, 84.0));
    }

    /// The dimmed strip and Done close on the press; the dishes never close.
    #[test]
    fn press_on_dim_or_done_closes_press_on_dishes_does_not() {
        let sheet = menu_sheet_fallback(dvec2(390.0, 800.0));
        let done = [menu_done_corner(sheet)];
        let dim = sheet_press_target(sheet, &done, dvec2(6.0, 400.0));
        let dish = sheet_press_target(sheet, &done, dvec2(120.0, 500.0));
        let on_done = sheet_press_target(sheet, &done, dvec2(360.0, 40.0));
        assert_eq!(dim, SheetPress::Outside);
        assert_eq!(dish, SheetPress::Inside);
        assert_eq!(on_done, SheetPress::Done);
        // Done sits inside the right 48 px edge strip, where a swipe would
        // otherwise start: the control wins.
        assert!(390.0 - 360.0 < 48.0);
        assert!(sheet_press_closes(None, false, dim));
        assert!(sheet_press_closes(None, false, on_done));
        assert!(!sheet_press_closes(None, false, dish));
    }

    /// A scroll that starts on the dishes and ends on the dim, or over Done,
    /// is still a scroll.
    #[test]
    fn a_scroll_from_the_dishes_never_closes() {
        let sheet = menu_sheet_fallback(dvec2(390.0, 800.0));
        let done = [menu_done_corner(sheet)];
        let began = Some(SheetPress::Inside);
        for end in [dvec2(4.0, 790.0), dvec2(360.0, 40.0), dvec2(200.0, 300.0)] {
            let target = sheet_press_target(sheet, &done, end);
            assert!(!sheet_press_closes(began, true, target), "{end:?}");
        }
    }

    /// When another handler took the press, a release on Done still closes;
    /// a release anywhere else does not.
    #[test]
    fn release_on_done_without_a_seen_press_closes() {
        let sheet = menu_sheet_fallback(dvec2(1280.0, 800.0));
        let done = [menu_done_corner(sheet)];
        let on_done = sheet_press_target(sheet, &done, dvec2(sheet.pos.x + sheet.size.x - 30.0, 50.0));
        assert!(sheet_press_closes(None, true, on_done));
        let dim = sheet_press_target(sheet, &done, dvec2(10.0, 10.0));
        assert!(!sheet_press_closes(None, true, dim));
    }

    /// An empty hit box (overlay not drawn yet) never counts as a hit.
    #[test]
    fn empty_rects_never_hit() {
        let empty = rect(0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            sheet_press_target(rect(10.0, 10.0, 100.0, 100.0), &[empty], dvec2(0.0, 0.0)),
            SheetPress::Outside
        );
    }
}

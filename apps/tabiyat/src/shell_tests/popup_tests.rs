#[cfg(test)]
mod popup_tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect {
            pos: dvec2(x, y),
            size: dvec2(w, h),
        }
    }

    #[test]
    fn list_hangs_under_its_field() {
        let field = rect(242.0, 270.0, 176.0, 60.0);
        let p = place_popup(field, 340.0, dvec2(1024.0, 768.0), 4.0, 8.0, 176.0);
        assert!(!p.above);
        assert_eq!(p.pos, dvec2(242.0, 334.0));
        assert_eq!(p.width, 176.0);
        assert_eq!(p.max_h, 768.0 - 8.0 - 334.0);
    }

    #[test]
    fn list_follows_a_scrolled_field() {
        let before = place_popup(rect(242.0, 270.0, 176.0, 60.0), 340.0, dvec2(1024.0, 768.0), 4.0, 8.0, 0.0);
        let after = place_popup(rect(242.0, 90.0, 176.0, 60.0), 340.0, dvec2(1024.0, 768.0), 4.0, 8.0, 0.0);
        assert_eq!(before.pos.y - 270.0 - 60.0, 4.0);
        assert_eq!(after.pos.y - 90.0 - 60.0, 4.0);
    }

    #[test]
    fn list_flips_above_near_the_bottom() {
        let field = rect(100.0, 650.0, 176.0, 56.0);
        let p = place_popup(field, 300.0, dvec2(1024.0, 768.0), 4.0, 8.0, 0.0);
        assert!(p.above);
        assert_eq!(p.pos.y + 300.0, 650.0 - 4.0);
        assert!(p.max_h >= 300.0);
    }

    #[test]
    fn list_takes_the_bigger_side_and_scrolls() {
        let field = rect(10.0, 150.0, 176.0, 56.0);
        let p = place_popup(field, 900.0, dvec2(400.0, 360.0), 4.0, 8.0, 0.0);
        assert!(!p.above, "{p:?}");
        assert_eq!(p.max_h, 360.0 - 8.0 - 210.0);
    }

    #[test]
    fn list_stays_inside_the_window_sideways() {
        let p = place_popup(rect(300.0, 100.0, 80.0, 40.0), 100.0, dvec2(390.0, 800.0), 4.0, 8.0, 176.0);
        assert_eq!(p.width, 176.0);
        assert_eq!(p.pos.x, 390.0 - 8.0 - 176.0);
        let narrow = place_popup(rect(0.0, 100.0, 500.0, 40.0), 100.0, dvec2(300.0, 800.0), 4.0, 8.0, 0.0);
        assert_eq!(narrow.width, 284.0);
        assert_eq!(narrow.pos.x, 8.0);
    }

    #[test]
    fn only_a_tap_opens_a_list() {
        assert!(popup_is_tap(dvec2(10.0, 10.0), 1.0, dvec2(20.0, 15.0), 1.2));
        assert!(!popup_is_tap(dvec2(10.0, 10.0), 1.0, dvec2(10.0, 60.0), 1.1));
        assert!(!popup_is_tap(dvec2(10.0, 10.0), 1.0, dvec2(10.0, 10.0), 2.0));
    }

    #[test]
    fn list_hides_while_its_field_is_scrolled_away() {
        let page = (135.0, 690.0);
        assert!(popup_anchor_on_page(rect(0.0, 300.0, 176.0, 60.0), page));
        assert!(popup_anchor_on_page(rect(0.0, 110.0, 176.0, 60.0), page));
        assert!(!popup_anchor_on_page(rect(0.0, 90.0, 176.0, 60.0), page));
        assert!(!popup_anchor_on_page(rect(0.0, 800.0, 176.0, 60.0), page));
        assert!(!popup_anchor_on_page(rect(0.0, 300.0, 0.0, 0.0), page));
    }

    /// Lists that a dropdown button opens never open from a field tap, and
    /// only the Team checklist survives an outside press.
    #[test]
    fn chrome_opened_lists_are_marked() {
        for f in SuggestField::ALL {
            if f.opener().is_some() {
                assert!(f.opens_by_chrome(), "{f:?}");
            }
            assert_eq!(f.stays_open(), f == SuggestField::TeamKotStations);
        }
        assert!(SuggestField::TeamKotStations.opens_by_chrome());
        assert!(SuggestField::FloorColumns.opens_by_chrome());
        assert!(SuggestField::FloorColumns.opens_unfiltered());
        assert!(!OpenList::tap_targets().any(|l| l == OpenList::Suggest(SuggestField::FloorColumns)));
        assert!(!OpenList::tap_targets().any(|l| l == OpenList::Suggest(SuggestField::KotStation)));
        assert!(OpenList::tap_targets().any(|l| l == OpenList::DishPicker));
    }
}

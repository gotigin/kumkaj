#[cfg(test)]
mod nav_order_tests {
    use super::*;

    #[test]
    fn saved_order_wins_and_missing_tabs_append() {
        let order = parse_nav_order(Some("credit,books,bogus,credit"));
        assert_eq!(order.len(), NAV_TABS.len());
        assert_eq!(order[0], NavTab::Credit);
        assert_eq!(order[1], NavTab::Books);
        // Every tab appears exactly once.
        for tab in NAV_TABS {
            assert_eq!(order.iter().filter(|t| **t == tab).count(), 1);
        }
        // Round-trip through CSV is stable.
        assert_eq!(parse_nav_order(Some(&nav_order_csv(&order))), order);
    }

    #[test]
    fn no_saved_order_is_default() {
        assert_eq!(parse_nav_order(None), NAV_TABS.to_vec());
        assert_eq!(parse_nav_order(Some("")), NAV_TABS.to_vec());
    }
}

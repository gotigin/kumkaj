#[cfg(test)]
mod menu_window_tests {
    use super::*;

    fn dish(name: &str, category: &str, aliases: &[&str]) -> Product {
        let mut p = Product::new_basic(new_id(), name, Money::from_rupees(100));
        p.category_id = category.into();
        p.aliases_json = serde_json::to_string(aliases).unwrap();
        p
    }

    /// The menu window: a tab narrows to its category, typing looks through
    /// what the tab shows, and a short code typed exactly comes first.
    #[test]
    fn tabs_narrow_and_short_codes_lead() {
        let mut st = AppState::boot();
        st.products = vec![
            dish("Paneer Tikka (Half)", "starters", &[]),
            dish("Masala Dosa", "south", &["101"]),
            dish("Dosa 1012 Special", "south", &[]),
            dish("Chaas", "drinks", &[]),
        ];
        let names = |st: &AppState| st.visible_products().into_iter().map(|p| p.name).collect::<Vec<_>>();
        assert_eq!(names(&st).len(), 4);
        st.menu_category = Some("south".into());
        assert_eq!(names(&st), ["Masala Dosa", "Dosa 1012 Special"]);
        st.menu_category = None;
        st.menu_filter = "101".into();
        assert_eq!(names(&st), ["Masala Dosa", "Dosa 1012 Special"]);
        st.menu_filter = "zz".into();
        assert!(names(&st).is_empty());
    }

    /// Never fewer than two tiles a row, even on the narrowest phone.
    #[test]
    fn menu_grid_has_at_least_two_columns() {
        assert_eq!(product_columns(200.0), 2);
        assert_eq!(product_columns(296.0), 2);
        assert_eq!(product_columns(948.0), 6);
        assert_eq!(product_columns(4000.0), PRODUCT_MAX_COLUMNS);
    }
}

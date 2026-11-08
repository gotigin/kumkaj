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

    /// Menu "Find in your menu" must look at aliases, not only the primary
    /// name. Exact alias / name lead; then prefix, word, and substring.
    #[test]
    fn menu_find_searches_aliases_and_ranks_exact_first() {
        let mut st = AppState::boot();
        st.products = vec![
            dish("ઈંડા કરી", "egg", &["Inda Curry"]),
            dish("આમલેટ", "egg", &["Omlet 2 Number"]),
            dish("Exact Code", "misc", &["A"]),
            dish("Soup", "misc", &["Idli"]),
        ];
        st.menu_list_filter = "a".into();
        let names = |st: &AppState| {
            st.menu_list_indices()
                .into_iter()
                .map(|i| st.products[i].name.clone())
                .collect::<Vec<_>>()
        };
        let hit = names(&st);
        assert_eq!(hit[0], "Exact Code", "exact alias \"A\" must lead");
        assert!(
            hit.iter().any(|n| n == "ઈંડા કરી"),
            "alias \"Inda Curry\" must match \"a\": {hit:?}"
        );
        assert!(
            !hit.iter().any(|n| n == "Soup"),
            "\"Soup\" / \"Idli\" have no a: {hit:?}"
        );
        assert!(
            !hit.iter().any(|n| n == "આમલેટ"),
            "\"Omlet 2 Number\" has no a — not a false hit: {hit:?}"
        );
        st.menu_list_filter = "omlet".into();
        assert_eq!(names(&st), ["આમલેટ"]);
        st.menu_list_filter = String::new();
        assert_eq!(names(&st).len(), 4);
    }

    /// Prefer two readable cards; three only when the window is truly wide.
    #[test]
    fn menu_grid_columns_for_wide_split_tiles() {
        assert_eq!(product_columns(200.0), 1);
        assert_eq!(product_columns(500.0), 1);
        assert_eq!(product_columns(640.0), 2);
        assert_eq!(product_columns(900.0), 2);
        assert_eq!(product_columns(1000.0), 3);
        assert_eq!(product_columns(4000.0), PRODUCT_MAX_COLUMNS);
    }
}

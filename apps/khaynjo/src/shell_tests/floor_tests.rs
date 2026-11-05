#[cfg(test)]
mod floor_tests {
    use super::*;

    #[test]
    fn table_ranges_read_the_way_owners_write_them() {
        assert_eq!(parse_table_range("D1-D20").unwrap(), ("D".into(), 1, 20));
        assert_eq!(parse_table_range("P 1 - 20").unwrap(), ("P ".into(), 1, 20));
        assert_eq!(parse_table_range("AC1 – AC10").unwrap(), ("AC".into(), 1, 10));
        assert!(parse_table_range("D20-D1").is_err());
        assert!(parse_table_range("D1").is_err());
    }

    #[test]
    fn area_headings_precede_their_tables() {
        let table = |name: &str, area: &str| DiningTable {
            id: name.into(),
            name: name.into(),
            status: domain::TableStatus::Available,
            active_sale_id: None,
            occupied_at: String::new(),
            running_total_paise: 0,
            area: area.into(),
            paid_at: String::new(),
            floor_state: Default::default(),
            waiter_staff_id: String::new(),
            layout_x: 0.0,
            layout_y: 0.0,
            layout_w: 0.0,
            layout_h: 0.0,
        };
        let tables = vec![
            table("D1", "Dine In"),
            table("D2", "Dine In"),
            table("D3", "Dine In"),
            table("P1", "Parcel"),
        ];
        assert_eq!(
            floor_rows(&tables, 2),
            vec![
                FloorRow::Area("Dine In".into()),
                FloorRow::Tiles(vec![0, 1]),
                FloorRow::Tiles(vec![2]),
                FloorRow::Area("Parcel".into()),
                FloorRow::Tiles(vec![3]),
            ]
        );
        // A floor with no areas named shows no headings at all.
        let plain = vec![table("T1", ""), table("T2", "")];
        assert_eq!(floor_rows(&plain, 6), vec![FloorRow::Tiles(vec![0, 1])]);
    }

    /// Tiles never shrink: the owner's column count is what a row holds,
    /// with no upper limit; "Fit screen" counts whole tiles across.
    #[test]
    fn columns_have_no_upper_limit_and_fit_counts_whole_tiles() {
        assert_eq!(floor_columns_for(375.0), 1, "app floor (375) shows one 280px tile");
        assert_eq!(floor_columns_for(569.0), 1);
        assert_eq!(floor_columns_for(570.0), 2, "2 × 280 + one 10px gap");
        assert_eq!(floor_columns_for(2900.0), 10, "no cap of 8 any more");
        assert_eq!(floor_grid_width(1, 1.0), 280.0);
        // Columns=3 is always 3×280 + 2×10 — never the ~2-tile viewport width.
        assert_eq!(floor_grid_width(3, 1.0), 860.0);
        assert!(
            floor_grid_width(3, 1.0) > 570.0,
            "3-col body must be wider than a phone viewport so ScrollX can show the third tile"
        );
        // Auto grows tiles into spare width; clamps at max.
        let auto = floor_auto_scale_for(5, 2000.0);
        assert!(auto > 1.0, "wide board should grow tiles, got {auto}");
        assert!(auto <= FLOOR_ZOOM_MAX);
        assert_eq!(floor_zoom_clamp(0.1), FLOOR_ZOOM_MIN);
        assert_eq!(floor_zoom_clamp(9.0), FLOOR_ZOOM_MAX);
        let (w, h) = floor_tile_wh(2.0);
        assert!((w / h - FLOOR_TILE_W / FLOOR_TILE_H).abs() < 0.001);
        let table = |i: usize| DiningTable {
            id: i.to_string(),
            name: format!("T{i}"),
            status: domain::TableStatus::Available,
            active_sale_id: None,
            occupied_at: String::new(),
            running_total_paise: 0,
            area: String::new(),
            paid_at: String::new(),
            floor_state: Default::default(),
            waiter_staff_id: String::new(),
            layout_x: 0.0,
            layout_y: 0.0,
            layout_w: 0.0,
            layout_h: 0.0,
        };
        let tables: Vec<_> = (0..40).map(table).collect();
        let rows = floor_rows(&tables, 30);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], FloorRow::Tiles((0..30).collect()));
        assert_eq!(floor_rows(&tables, 0).len(), 40, "0 is read as 1 column");
    }

    /// Only the columns in the sideways viewport are drawn — scrolled or
    /// not, a few slots, never a whole 100-column row.
    #[test]
    fn column_window_follows_the_sideways_scroll() {
        // Not laid out yet: the first columns, at most one row of slots.
        assert_eq!(floor_col_window(0.0, 0.0, 0.0, 100, 1.0), (0, FLOOR_ROW_SLOTS));
        assert_eq!(floor_col_window(0.0, 0.0, 0.0, 3, 1.0), (0, 3));
        // Unscrolled 1000px viewport: columns 0..4 (the 4th partly).
        assert_eq!(floor_col_window(16.0, 16.0, 1000.0, 100, 1.0), (0, 4));
        // Scrolled 2900px (10 columns) to the left.
        let (first, last) = floor_col_window(16.0 - 2900.0, 16.0, 1000.0, 100, 1.0);
        assert_eq!((first, last), (10, 14));
        // Near the end: never past the last column.
        let (first, last) = floor_col_window(16.0 - 2900.0, 16.0, 1000.0, 12, 1.0);
        assert_eq!((first, last), (10, 12));
        // A huge monitor still fits in one row of slots.
        let (first, last) = floor_col_window(0.0, 0.0, 20_000.0, 500, 1.0);
        assert_eq!(last - first, FLOOR_ROW_SLOTS);
    }

    #[test]
    fn area_order_prefers_saved_then_appends_new() {
        let present = vec![
            "".to_string(),
            "Dine In".to_string(),
            "Parcel".to_string(),
            "AC".to_string(),
        ];
        let preferred = vec!["Parcel".to_string(), "Dine In".to_string()];
        assert_eq!(
            order_floor_areas(&present, &preferred),
            vec![
                "Parcel".to_string(),
                "Dine In".to_string(),
                "".to_string(),
                "AC".to_string(),
            ]
        );
        assert_eq!(
            parse_floor_area_order(Some("Parcel,Dine In")),
            vec!["Parcel".to_string(), "Dine In".to_string()]
        );
        assert_eq!(
            floor_area_order_csv(&["Parcel".into(), "".into(), "AC".into()]),
            "Parcel,,AC"
        );
    }

    #[test]
    fn area_groups_keep_each_area_separate_for_tabs() {
        let table = |name: &str, area: &str| DiningTable {
            id: name.into(),
            name: name.into(),
            status: domain::TableStatus::Available,
            active_sale_id: None,
            occupied_at: String::new(),
            running_total_paise: 0,
            area: area.into(),
            paid_at: String::new(),
            floor_state: Default::default(),
            waiter_staff_id: String::new(),
            layout_x: 0.0,
            layout_y: 0.0,
            layout_w: 0.0,
            layout_h: 0.0,
        };
        let tables = vec![
            table("D1", "Dine In"),
            table("D2", "Dine In"),
            table("P1", "Parcel"),
        ];
        let groups = floor_area_groups(&tables);
        assert_eq!(
            groups,
            vec![
                ("Dine In".into(), vec![0, 1]),
                ("Parcel".into(), vec![2]),
            ]
        );
        // Selected tab body uses only that group's indices.
        let parcel = groups
            .iter()
            .find(|(a, _)| a == "Parcel")
            .map(|(_, idxs)| idxs.clone())
            .unwrap();
        assert_eq!(parcel, vec![2]);
    }

    #[test]
    fn floor_page_window_only_covers_the_viewport() {
        let rows: Vec<FloorRow> = (0..40).map(|i| FloorRow::Tiles(vec![i])).collect();
        *page_viewport().lock().unwrap() = (100.0, 400.0);
        // List starts at y=100 (flush with the viewport top): first slice
        // must be a handful of rows, not the whole floor.
        let (first, last, before, slice) = floor_page_window(100.0, &rows, 10.0);
        assert_eq!(first, 0);
        assert!(last < rows.len(), "must not draw all {n} rows", n = rows.len());
        assert!(last <= 6, "≈400px viewport fits a few 186px tiles, got last={last}");
        assert_eq!(before, 0.0);
        assert!(slice > 0.0 && slice < 2000.0);
        // Scrolled so row 10 is near the top.
        let (first, last, before, _) = floor_page_window(-1800.0, &rows, 10.0);
        assert!(first >= 8, "scrolled deep: first={first}");
        assert!(last - first <= 6, "window stayed small: {first}..{last}");
        assert!(before > 0.0);
    }
}

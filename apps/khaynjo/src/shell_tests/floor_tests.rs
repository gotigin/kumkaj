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

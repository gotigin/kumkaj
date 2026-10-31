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
}

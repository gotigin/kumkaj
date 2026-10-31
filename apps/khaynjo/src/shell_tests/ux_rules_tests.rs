#[cfg(test)]
mod ux_rules_tests {
    use super::*;

    fn part(method: PayMethod, amount: &str) -> PayPart {
        PayPart { method, amount: amount.into() }
    }

    /// Change table must not seat/move the previous ticket onto the new one.
    #[test]
    fn change_table_opens_only_first_choose_seats_cart() {
        assert!(pick_seats_desk_cart(None), "first Choose a table seats desk cart");
        assert!(
            !pick_seats_desk_cart(Some("table-a")),
            "Change table must not drag lines onto the picked table"
        );
    }

    #[test]
    fn one_blank_way_takes_the_rest_and_same_ways_merge() {
        let parts = [part(PayMethod::Cash, "300"), part(PayMethod::Upi, ""), part(PayMethod::Cash, "50")];
        let got = settle_pay_parts(50_000, &parts, "INR").unwrap();
        assert_eq!(got, vec![(PayMethod::Cash, 35_000), (PayMethod::Upi, 15_000)]);
    }

    #[test]
    fn ways_must_add_up_and_only_one_may_be_blank() {
        assert!(settle_pay_parts(50_000, &[part(PayMethod::Cash, "100"), part(PayMethod::Card, "100")], "INR").is_err());
        assert!(settle_pay_parts(50_000, &[part(PayMethod::Cash, ""), part(PayMethod::Card, "")], "INR").is_err());
        assert_eq!(
            settle_pay_parts(50_000, &[part(PayMethod::Bank, "")], "INR").unwrap(),
            vec![(PayMethod::Bank, 50_000)]
        );
    }

    #[test]
    fn a_report_reads_as_headings_rows_and_lines() {
        let rows = parse_report(
            "Rajdhani — day close\nSales: 1\nRevenue ₹120.00\nPeriod 2026-10-24 → 2026-10-24\n\nTop items\n  Masala Dosa · 1 · ₹120.00\n-----",
        );
        assert_eq!(
            rows,
            vec![
                ReportRow::Head("Rajdhani — day close".into()),
                ReportRow::Kv("Sales".into(), "1".into()),
                ReportRow::Kv("Revenue".into(), "₹120.00".into()),
                ReportRow::Line("Period 2026-10-24 → 2026-10-24".into()),
                ReportRow::Gap,
                ReportRow::Head("Top items".into()),
                ReportRow::Sub("Masala Dosa · 1 · ₹120.00".into()),
            ]
        );
    }

    /// Settings → Units: each family under its own heading from smallest to
    /// largest (a shop's own unit among them), unlinked units apart, and
    /// one conversions row per unit they start from.
    #[test]
    fn units_list_by_family_not_one_paragraph() {
        let home = std::env::temp_dir().join(format!("khaynjo-units-{}", new_id()));
        std::fs::create_dir_all(&home).unwrap();
        let store = Store::open_path(&home.join("books.sqlite3")).unwrap();
        store.add_unit("bag", "Bag").unwrap();
        store.add_unit_conversion("bag", "kg", 25_000).unwrap();
        store.add_unit("tray", "Tray").unwrap();
        let t = |k: &str| match k {
            "unit_picker_on" => "On".into(),
            "unit_picker_off" => "Off".into(),
            _ => k.into(),
        };
        let rows = units_report_rows(&store, t);
        let heads: Vec<&str> = rows
            .iter()
            .filter_map(|r| match r {
                ReportRow::Head(h) => Some(h.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            heads,
            [
                "Weight (kg)",
                "Length (cm)",
                "Volume (ml)",
                "Count (pc)",
                "Not linked to a standard unit yet",
                "Conversions (1 from = factor to)",
            ]
        );
        let weight: Vec<&ReportRow> = rows.iter().skip(1).take_while(|r| !matches!(r, ReportRow::Head(_))).collect();
        assert_eq!(
            weight.first(),
            Some(&&ReportRow::Kv("g — Gram · 1 g = 0.001 kg".into(), "On".into()))
        );
        let kg = weight
            .iter()
            .position(|r| **r == ReportRow::Kv("kg — Kilogram · base unit".into(), "On".into()))
            .unwrap();
        assert_eq!(
            *weight[kg + 1],
            ReportRow::Kv("bag — Bag (custom) · 1 bag = 25 kg".into(), "On".into())
        );
        assert!(rows.contains(&ReportRow::Kv("tray — Tray (custom)".into(), "On".into())));
        assert!(rows.contains(&ReportRow::Line("1 bag = 25 kg".into())));
        let _ = std::fs::remove_dir_all(home);
    }

    /// Run-on figure lines split into rows; section titles and the words
    /// over figures become headings; a closing sentence stays a line.
    #[test]
    fn report_figures_never_run_on() {
        let rows = parse_report(
            "Cash ₹120.00 · UPI ₹0.00 · Card ₹5.00\nExpenses ₹50.00 (2)\n\nParty outstanding (aging)\nNo balances.\n\nAssets\nBank ₹0.00\nRavi · Customer · ₹500.00 · 3d · current\n\nNo day close recorded yet.",
        );
        assert_eq!(
            rows,
            vec![
                ReportRow::Kv("Cash".into(), "₹120.00".into()),
                ReportRow::Kv("UPI".into(), "₹0.00".into()),
                ReportRow::Kv("Card".into(), "₹5.00".into()),
                ReportRow::Kv("Expenses".into(), "₹50.00 (2)".into()),
                ReportRow::Gap,
                ReportRow::Head("Party outstanding (aging)".into()),
                ReportRow::Line("No balances.".into()),
                ReportRow::Gap,
                ReportRow::Head("Assets".into()),
                ReportRow::Kv("Bank".into(), "₹0.00".into()),
                ReportRow::Line("Ravi · Customer · ₹500.00 · 3d · current".into()),
                ReportRow::Gap,
                ReportRow::Line("No day close recorded yet.".into()),
            ]
        );
    }
}

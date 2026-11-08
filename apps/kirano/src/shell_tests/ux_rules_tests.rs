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
        let home = std::env::temp_dir().join(format!("kirano-units-{}", new_id()));
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
        // Picker state carries its dot (● On / ○ Off) since c98d4c2.
        let weight: Vec<&ReportRow> = rows.iter().skip(1).take_while(|r| !matches!(r, ReportRow::Head(_))).collect();
        assert_eq!(
            weight.first(),
            Some(&&ReportRow::Kv("g — Gram · 1 g = 0.001 kg".into(), "● On".into()))
        );
        let kg = weight
            .iter()
            .position(|r| **r == ReportRow::Kv("kg — Kilogram · base unit".into(), "● On".into()))
            .unwrap();
        assert_eq!(
            *weight[kg + 1],
            ReportRow::Kv("bag — Bag (custom) · 1 bag = 25 kg".into(), "● On".into())
        );
        assert!(rows.contains(&ReportRow::Kv("tray — Tray (custom)".into(), "● On".into())));
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

    /// Kitchen: the whole ticket card advances like its button. Without a
    /// cursor the View never hears the finger, so match_event's card tap
    /// was dead code and only the small button worked.
    #[test]
    fn kitchen_card_is_one_tap_target() {
        let ui = include_str!("../ui_script.rs");
        let card = ui
            .split("let KotCard = View{")
            .nth(1)
            .and_then(|s| s.split("bg_ok :=").next())
            .unwrap_or("");
        assert!(card.contains("cursor: MouseCursor.Hand"), "KotCard needs a cursor to take the tap");
        let handler = include_str!("../app/match_event.rs");
        let tap = handler
            .split("Kitchen: advance a ticket")
            .nth(1)
            .and_then(|s| s.split("table_range_add").next())
            .unwrap_or("");
        assert!(
            tap.contains("ids!(kotcard))") && tap.contains(".finger_up(actions)"),
            "card tap is read"
        );
        assert!(tap.contains("finger_up_is_tap"), "a scroll drag is not a tap");
        assert!(tap.contains("advance_kot_from"), "the tap only leaves the state it showed");
    }

    /// Send KOT on the shop device: the line names the kitchens (no
    /// programmer "KOT(s)"), and a paper failure stays red in every language —
    /// the kind is set in code, not guessed from English words.
    #[test]
    fn kot_send_outcome_names_kitchens_and_keeps_paper_failures_red() {
        let en = |k: &str| locale::t("en", k);
        let (kind, msg) = kot_sent_status(&en, "Hot kitchen, Bar", true, None);
        assert_eq!(kind, SnackKind::Success);
        assert_eq!(msg, "KOT sent to Hot kitchen, Bar — printing");
        let (kind, msg) = kot_sent_status(&en, "Hot kitchen", true, Some("printer offline"));
        assert_eq!(kind, SnackKind::Error);
        assert!(msg.contains("printer offline") && msg.contains("Tell the kitchen"), "{msg}");
        let (kind, msg) = kot_sent_status(&en, "Hot kitchen", false, None);
        assert_eq!((kind, msg.as_str()), (SnackKind::Success, "KOT sent to Hot kitchen"));
        assert!(!msg.contains("KOT(s)"));

        let gu = |k: &str| locale::t("gu", k);
        let (kind, msg) = kot_sent_status(&gu, "Hot", true, Some("offline"));
        assert_eq!(kind, SnackKind::Error, "{msg}");
        assert!(!msg.contains("NOT printed") && msg.contains("Hot") && msg.contains("offline"), "{msg}");
        let (kind, msg) = kot_sent_status(&gu, "Hot", true, None);
        assert_eq!(kind, SnackKind::Success);
        assert_ne!(
            snack::classify(&msg),
            SnackKind::Success,
            "guessing from English words would not colour a Gujarati line: {msg}"
        );
    }

    /// Empty states name the grocery rail (Sell, Items), not a restaurant floor.
    #[test]
    fn empty_states_name_the_rail_screens() {
        let menu = locale::t("en", "empty_catalog");
        assert!(menu.contains("Items") && !menu.contains("Menu & items"), "{menu}");
        assert_eq!(locale::t("en", "nav_items"), "Items");
        assert_eq!(locale::t("en", "sell"), "Sell");
    }
}

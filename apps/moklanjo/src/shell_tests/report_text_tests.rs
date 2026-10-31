#[cfg(test)]
mod report_text_tests {
    use super::*;

    /// Day close, books and ops text read as headings, label/value rows and
    /// lines in a card — never one run-on paragraph.
    #[test]
    fn a_report_reads_as_headings_rows_and_lines() {
        let rows = parse_report(
            "Swift Courier — day close\nSales: 3\nCash ₹120.00 · UPI ₹0.00 · Card ₹5.00\nExpenses ₹50.00 (2)\n\nAssets\nBank ₹0.00\nMehta Traders · Consignor · ₹12.00 · 0d · current\n\nNo day close recorded yet.",
        );
        assert_eq!(
            rows,
            vec![
                ReportRow::Head("Swift Courier — day close".into()),
                ReportRow::Kv("Sales".into(), "3".into()),
                ReportRow::Kv("Cash".into(), "₹120.00".into()),
                ReportRow::Kv("UPI".into(), "₹0.00".into()),
                ReportRow::Kv("Card".into(), "₹5.00".into()),
                ReportRow::Kv("Expenses".into(), "₹50.00 (2)".into()),
                ReportRow::Gap,
                ReportRow::Head("Assets".into()),
                ReportRow::Kv("Bank".into(), "₹0.00".into()),
                ReportRow::Line("Mehta Traders · Consignor · ₹12.00 · 0d · current".into()),
                ReportRow::Gap,
                ReportRow::Line("No day close recorded yet.".into()),
            ]
        );
    }

    /// The ops desk: "Ops:" and "— Barcode desk —" are headings, each count
    /// its own row, consignments stay whole lines, and a long "Next: …"
    /// stays a full line instead of squeezing its label.
    #[test]
    fn ops_sections_are_headed_and_counts_are_rows() {
        let rows = parse_report(
            "Ops:\nBooked: 1\nCOD pending: ₹500.00\n\n— Consignments (recent AWBs) —\nAWB-261026-000002 · Mumbai · out_for_delivery · unbilled · ₹12.00\nNext: Dispatch to rider / Hand to carrier · or Out for delivery",
        );
        assert_eq!(
            rows,
            vec![
                ReportRow::Head("Ops".into()),
                ReportRow::Kv("Booked".into(), "1".into()),
                ReportRow::Kv("COD pending".into(), "₹500.00".into()),
                ReportRow::Gap,
                ReportRow::Head("Consignments (recent AWBs)".into()),
                ReportRow::Line("AWB-261026-000002 · Mumbai · out_for_delivery · unbilled · ₹12.00".into()),
                ReportRow::Line("Next: Dispatch to rider / Hand to carrier · or Out for delivery".into()),
            ]
        );
    }

    /// Settings → Units: each family under its own heading from smallest to
    /// largest (an office's own unit among them), unlinked units apart, and
    /// one conversions row per unit they start from.
    #[test]
    fn units_list_by_family_not_one_paragraph() {
        let home = std::env::temp_dir().join(format!("courier-units-{}", new_id()));
        std::fs::create_dir_all(&home).unwrap();
        let store = Store::open_path(&home.join("books.sqlite3")).unwrap();
        store.add_unit("bag", "Bag").unwrap();
        store.add_unit_conversion("bag", "kg", 25_000).unwrap();
        store.add_unit("crate", "Crate").unwrap();
        let rows = units_report_rows(&store);
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
        let kg = rows.iter().position(|r| *r == ReportRow::Kv("kg — Kilogram".into(), "base unit".into())).unwrap();
        assert_eq!(rows[kg + 1], ReportRow::Kv("bag — Bag (custom)".into(), "1 bag = 25 kg".into()));
        assert!(rows.contains(&ReportRow::Line("crate — Crate (custom)".into())));
        assert!(rows.contains(&ReportRow::Line("1 bag = 25 kg".into())));
        let _ = std::fs::remove_dir_all(home);
    }
}

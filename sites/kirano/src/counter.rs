//! Brand-chapter counter bill toy: tap to walk a grocery sale.
//!
//! Copied ideas only — a site never depends on an app crate. Words and
//! example lines are site illustrations (marked Example in the UI), not
//! app screenshots.

/// One step of a counter sale, in the order the toy walks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillStep {
    Ready,
    Line1,
    Line2,
    Line3,
    Settling,
    Paid,
}

impl BillStep {
    pub fn word(self) -> &'static str {
        match self {
            BillStep::Ready => "Ready",
            BillStep::Line1 | BillStep::Line2 | BillStep::Line3 => "Selling",
            BillStep::Settling => "Settle",
            BillStep::Paid => "Paid",
        }
    }

    pub fn next(self) -> BillStep {
        match self {
            BillStep::Ready => BillStep::Line1,
            BillStep::Line1 => BillStep::Line2,
            BillStep::Line2 => BillStep::Line3,
            BillStep::Line3 => BillStep::Settling,
            BillStep::Settling => BillStep::Paid,
            BillStep::Paid => BillStep::Ready,
        }
    }

    /// How many catalogue lines are visible on the example bill.
    pub fn lines_shown(self) -> usize {
        match self {
            BillStep::Ready | BillStep::Paid => 0,
            BillStep::Line1 => 1,
            BillStep::Line2 => 2,
            BillStep::Line3 | BillStep::Settling => 3,
        }
    }

    pub fn total(self) -> u32 {
        match self.lines_shown() {
            0 => 0,
            1 => LINES[0].1,
            2 => LINES[0].1 + LINES[1].1,
            _ => LINES[0].1 + LINES[1].1 + LINES[2].1,
        }
    }

    pub fn tender_line(self) -> &'static str {
        match self {
            BillStep::Ready => "Tap to add the first item",
            BillStep::Line1 | BillStep::Line2 => "Tap to add the next item",
            BillStep::Line3 => "Tap to settle — cash or UPI",
            BillStep::Settling => "Cash ₹ 600 · return ₹ 15",
            BillStep::Paid => "Paid · day book updated",
        }
    }
}

/// Example grocery lines: (label, amount ₹).
pub const LINES: [(&str, u32); 3] = [
    ("Atta 5 kg", 280),
    ("Sunflower oil 1 L", 145),
    ("Toor dal 1 kg", 160),
];

/// 1240 → "1,240" (Indian grouping is the same below one lakh).
pub fn rupees(amount: u32) -> String {
    let s = amount.to_string();
    if s.len() <= 3 {
        return s;
    }
    let (head, tail) = s.split_at(s.len() - 3);
    let mut out = String::new();
    for (i, ch) in head.chars().enumerate() {
        if i > 0 && (head.len() - i) % 2 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.push(',');
    out.push_str(tail);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sale_walks_ready_to_paid_then_resets() {
        let order = [
            BillStep::Ready,
            BillStep::Line1,
            BillStep::Line2,
            BillStep::Line3,
            BillStep::Settling,
            BillStep::Paid,
        ];
        let mut v = BillStep::Ready;
        let mut seen = vec![v];
        for _ in 0..6 {
            v = v.next();
            seen.push(v);
        }
        assert_eq!(&seen[..6], &order);
        assert_eq!(seen[6], BillStep::Ready);
    }

    #[test]
    fn totals_accumulate_with_lines() {
        assert_eq!(BillStep::Ready.total(), 0);
        assert_eq!(BillStep::Line1.total(), 280);
        assert_eq!(BillStep::Line3.total(), 585);
        assert_eq!(BillStep::Settling.total(), 585);
    }

    #[test]
    fn rupees_group_like_a_bill() {
        assert_eq!(rupees(280), "280");
        assert_eq!(rupees(1240), "1,240");
        assert_eq!(rupees(20150), "20,150");
        assert_eq!(rupees(125000), "1,25,000");
    }
}

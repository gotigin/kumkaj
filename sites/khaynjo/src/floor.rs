//! The "Lunch rush" floor toy: tap a table to walk it through a visit.
//!
//! Words and colours are Khaynjo's own (copied, not imported — a site never
//! depends on an app crate): `domain::FloorState` tile keys and English
//! locale rows (`tile_free`, `tile_occupied`, `floor_in_kitchen`,
//! `tile_food_ready`, `floor_eating`, `tile_bill_printed`, `tile_paid`), and
//! the `FloorTile` background layers in apps/khaynjo/src/ui_script.rs.
//! As in the app, a table only frees after it is paid (`free_table` refuses
//! while the bill still has money due), so Bill printed → Paid → Free.

/// One visit, in the order the app's floor shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit {
    Free,
    Occupied,
    InKitchen,
    FoodReady,
    Served,
    BillPrinted,
    Paid,
}

impl Visit {
    pub const ALL: [Visit; 7] = [
        Visit::Free,
        Visit::Occupied,
        Visit::InKitchen,
        Visit::FoodReady,
        Visit::Served,
        Visit::BillPrinted,
        Visit::Paid,
    ];

    /// The word on the tile (Khaynjo English locale).
    pub fn word(self) -> &'static str {
        match self {
            Visit::Free => "Free",
            Visit::Occupied => "Occupied",
            Visit::InKitchen => "In kitchen",
            Visit::FoodReady => "Food ready",
            Visit::Served => "Served",
            Visit::BillPrinted => "Bill printed",
            Visit::Paid => "Paid",
        }
    }

    /// Tap → the next step of the visit; a paid table becomes free.
    pub fn next(self) -> Visit {
        match self {
            Visit::Free => Visit::Occupied,
            Visit::Occupied => Visit::InKitchen,
            Visit::InKitchen => Visit::FoodReady,
            Visit::FoodReady => Visit::Served,
            Visit::Served => Visit::BillPrinted,
            Visit::BillPrinted => Visit::Paid,
            Visit::Paid => Visit::Free,
        }
    }

    pub fn index(self) -> usize {
        Visit::ALL.iter().position(|v| *v == self).unwrap_or(0)
    }

    /// Minutes since the guests sat down, as the tile shows them (example).
    fn minutes(self) -> Option<u32> {
        match self {
            Visit::Free | Visit::Paid => None,
            Visit::Occupied => Some(2),
            Visit::InKitchen => Some(9),
            Visit::FoodReady => Some(18),
            Visit::Served => Some(26),
            Visit::BillPrinted => Some(41),
        }
    }
}

/// The small line under the state word: minutes and the running bill.
/// `amount` is the table's example bill in rupees.
pub fn meta(visit: Visit, amount: u32) -> String {
    match (visit, visit.minutes()) {
        (Visit::Free, _) => String::new(),
        (Visit::Paid, _) => format!("₹ {} paid", rupees(amount)),
        (Visit::Occupied, Some(min)) => format!("{min} min"),
        (_, Some(min)) => format!("{min} min · ₹ {}", rupees(amount)),
        (_, None) => String::new(),
    }
}

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
    fn a_visit_walks_the_apps_order_and_frees_only_after_paid() {
        let mut v = Visit::Free;
        let mut seen = vec![v];
        for _ in 0..7 {
            v = v.next();
            seen.push(v);
        }
        assert_eq!(&seen[..7], &Visit::ALL);
        assert_eq!(seen[7], Visit::Free);
        // Bill printed never jumps straight to Free.
        assert_eq!(Visit::BillPrinted.next(), Visit::Paid);
    }

    #[test]
    fn tile_words_match_the_app_locale() {
        let words: Vec<_> = Visit::ALL.iter().map(|v| v.word()).collect();
        assert_eq!(
            words,
            ["Free", "Occupied", "In kitchen", "Food ready", "Served", "Bill printed", "Paid"]
        );
    }

    #[test]
    fn meta_shows_minutes_then_the_running_bill() {
        assert_eq!(meta(Visit::Free, 640), "");
        assert_eq!(meta(Visit::Occupied, 640), "2 min");
        assert_eq!(meta(Visit::InKitchen, 640), "9 min · ₹ 640");
        assert_eq!(meta(Visit::Paid, 1240), "₹ 1,240 paid");
    }

    #[test]
    fn rupees_group_like_a_bill() {
        assert_eq!(rupees(640), "640");
        assert_eq!(rupees(1240), "1,240");
        assert_eq!(rupees(20150), "20,150");
        assert_eq!(rupees(125000), "1,25,000");
    }
}

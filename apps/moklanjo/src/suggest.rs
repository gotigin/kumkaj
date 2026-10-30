//! "Type or pick" for every field whose value is usually something the
//! counter already has — a consignor, carrier, zone, unit, tax rate,
//! destination. Nobody should have to remember how a name was spelt: the
//! field offers what exists as soon as it is focused (empty = everything),
//! narrows while typing, and a tap fills it. What the person typed stays
//! theirs; the list never forces a choice.

/// One row under a field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestRow {
    /// What a tap writes into the field.
    pub value: String,
    /// Small second line: phone, role, rate — whatever tells two similar
    /// rows apart.
    pub subtitle: String,
    /// Right-hand action label (`Use`, …). Empty → locale `use`.
    pub action: String,
}

impl SuggestRow {
    pub fn new(value: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            subtitle: subtitle.into(),
            action: String::new(),
        }
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }
}

/// Most rows a list keeps; the field narrows past that.
pub const SUGGEST_LIMIT: usize = 60;

fn fold(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Exact → prefix → word-prefix → substring → all-words. Lower is better.
pub fn match_rank(text: &str, query: &str) -> Option<u8> {
    let q = fold(query);
    if q.is_empty() {
        return None;
    }
    let t = fold(text);
    if t == q {
        Some(0)
    } else if t.starts_with(&q) {
        Some(1)
    } else if t.split(' ').any(|w| w.starts_with(&q)) {
        Some(2)
    } else if t.contains(&q) {
        Some(3)
    } else if q.split(' ').all(|w| t.contains(w)) {
        Some(4)
    } else {
        None
    }
}

/// Rows for `query`: empty = all candidates in their given order; otherwise
/// the ones whose value (or subtitle) matches, best match first. Duplicate
/// values (case-insensitive) keep only their first row. A row equal to the
/// query is left out — the field already says it.
pub fn rank(candidates: Vec<SuggestRow>, query: &str) -> Vec<SuggestRow> {
    let q = query.trim();
    let mut seen = std::collections::HashSet::new();
    let unique = candidates
        .into_iter()
        .filter(|r| !r.value.trim().is_empty())
        .filter(|r| seen.insert(r.value.trim().to_lowercase()));
    if q.is_empty() {
        return unique.take(SUGGEST_LIMIT).collect();
    }
    let mut scored: Vec<(u8, usize, SuggestRow)> = unique
        .enumerate()
        .filter(|(_, r)| !r.value.trim().eq_ignore_ascii_case(q))
        .filter_map(|(i, r)| {
            let by_value = match_rank(&r.value, q);
            // Subtitle matches rank after every name match.
            let by_sub = match_rank(&r.subtitle, q).map(|s| s.saturating_add(100));
            by_value.or(by_sub).map(|s| (s, i, r))
        })
        .collect();
    scored.sort_by_key(|(s, i, _)| (*s, *i));
    scored
        .into_iter()
        .take(SUGGEST_LIMIT)
        .map(|(_, _, r)| r)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(names: &[&str]) -> Vec<SuggestRow> {
        names.iter().map(|n| SuggestRow::new(*n, "")).collect()
    }

    #[test]
    fn empty_query_offers_everything_in_order_without_duplicates() {
        let got = rank(rows(&["Tata", "BlueDart", "tata", ""]), "  ");
        let values: Vec<_> = got.iter().map(|r| r.value.as_str()).collect();
        assert_eq!(values, ["Tata", "BlueDart"]);
    }

    #[test]
    fn typing_narrows_and_puts_the_best_match_first() {
        let got = rank(rows(&["Pune Hub", "Pune West", "Mumbai"]), "pun");
        let values: Vec<_> = got.iter().map(|r| r.value.as_str()).collect();
        assert_eq!(values.first(), Some(&"Pune Hub"));
        assert!(!values.contains(&"Mumbai"));
    }

    #[test]
    fn an_exact_match_is_not_offered_back() {
        let got = rank(rows(&["Kg", "Kgs"]), "kg");
        assert!(got.iter().all(|r| r.value != "Kg"));
    }

    #[test]
    fn subtitle_finds_a_row_after_name_matches() {
        let candidates = vec![
            SuggestRow::new("D5", "Parcel"),
            SuggestRow::new("Parcel corner", "Dine In"),
        ];
        let got = rank(candidates, "parcel");
        assert_eq!(got[0].value, "Parcel corner");
        assert_eq!(got[1].value, "D5");
    }
}

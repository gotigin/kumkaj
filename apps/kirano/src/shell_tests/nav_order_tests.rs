#[cfg(test)]
mod nav_order_tests {
    use super::*;

    #[test]
    fn saved_order_wins_and_missing_tabs_append() {
        let order = parse_nav_order(Some("credit,books,bogus,credit"));
        assert_eq!(order.len(), NAV_TABS.len());
        assert_eq!(order[0], NavTab::Credit);
        assert_eq!(order[1], NavTab::Books);
        // Every tab appears exactly once.
        for tab in NAV_TABS {
            assert_eq!(order.iter().filter(|t| **t == tab).count(), 1);
        }
        // Round-trip through CSV is stable.
        assert_eq!(parse_nav_order(Some(&nav_order_csv(&order))), order);
    }

    #[test]
    fn no_saved_order_is_default() {
        assert_eq!(parse_nav_order(None), NAV_TABS.to_vec());
        assert_eq!(parse_nav_order(Some("")), NAV_TABS.to_vec());
    }

    /// Every button inside `header` must be in HEADER_CONTROLS, or the desktop
    /// window drag (WindowDragQuery → Caption) eats its press — the collapsed
    /// sidebar's `>` never opened on Windows / Linux (2026-11-10).
    #[test]
    fn header_buttons_are_not_window_drag() {
        let ui = include_str!("../ui_script.rs");
        let start = ui.find("header := SolidView{").expect("header block in ui_script.rs");
        let mut depth = 0i32;
        let mut end = ui.len();
        for (i, ch) in ui[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let block = &ui[start..end];
        let mut buttons = Vec::new();
        for line in block.lines() {
            let Some((name, rest)) = line.trim().split_once(" := ") else {
                continue;
            };
            let ty = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
            if ty.ends_with("Btn") || ty.contains("Button") {
                buttons.push(name.trim().to_string());
            }
        }
        assert!(!buttons.is_empty(), "no buttons found in header — parser drifted");
        for name in &buttons {
            assert!(
                HEADER_CONTROLS.contains(&name.as_str()),
                "header button `{name}` is missing from HEADER_CONTROLS (desktop window drag would eat its press)"
            );
        }
        for name in HEADER_CONTROLS {
            assert!(buttons.iter().any(|b| b == name), "HEADER_CONTROLS lists `{name}`, which is not a header button");
        }
    }

    /// A window-filling Modal not in INSET_SHEETS draws under the status and
    /// gesture bars on an edge-to-edge phone; an anchored one must stay out.
    #[test]
    fn every_modal_is_an_inset_sheet_or_anchored() {
        const ANCHORED: &[&str] = &[];
        let ui = include_str!("../ui_script.rs");
        let modals: Vec<&str> = ui
            .lines()
            .filter_map(|l| l.trim().strip_suffix(" := Modal{"))
            .collect();
        assert!(!modals.is_empty(), "no Modal found — parser drifted");
        for name in &modals {
            assert!(
                INSET_SHEETS.contains(name) || ANCHORED.contains(name),
                "Modal `{name}`: add it to INSET_SHEETS (fills the window) or ANCHORED here"
            );
        }
        for name in INSET_SHEETS {
            assert!(modals.contains(name), "INSET_SHEETS lists `{name}`, which is not a Modal");
        }
    }
}

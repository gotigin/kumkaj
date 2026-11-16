#[cfg(test)]
mod nav_order_tests {
    use super::*;

    /// The app's tests never open this machine's books or printers and
    /// never print: they once loaded the shop's printers.json through
    /// `boot` and sent real tickets to its printer on every run (2026-11-15).
    #[test]
    fn app_tests_never_reach_this_machines_printers() {
        let st = AppState::boot();
        assert!(st.data_base.is_none(), "boot opened this machine's data folder");
        assert!(st.store.is_none(), "boot opened this machine's books");
        assert_eq!(print_share::sink(), print_share::PrintSink::Capture);
        assert!(print_share::printers().printers.is_empty(), "this machine's printers loaded");
    }

    /// Unlock lands where each person last was on this device; the first
    /// time on their first item; never a fixed screen, never Lock; an item
    /// they may no longer open → their first item (owner, 2026-11-15).
    #[test]
    fn unlock_returns_each_person_to_their_last_item() {
        let path = kumkaj_moklanjo::test_dirs::run_root().join(format!("moklanjo-last-nav-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let cash_id = store
            .add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner)
            .unwrap();
        let cashier = store.list_staff().unwrap().into_iter().find(|m| m.id == cash_id).unwrap();
        // The app's default layout, whatever `boot` started with.
        let fresh = |st: &mut AppState| {
            st.nav_order = default_nav_order("en");
            st.nav_hidden = parse_nav_layout(None, None, "en").1;
        };
        let mut st = AppState::boot();
        fresh(&mut st);
        st.store = Some(store);
        st.session = Some(owner.clone());
        let first = first_allowed_work_tab(&st).unwrap();
        assert_eq!(unlocked_home_from(&mut st), screen_for_nav_tab(first));
        assert_eq!(st.nav_tab_after_unlock, Some(first));
        // Another item on the bar, then a Settings section (not on the
        // bar): back there each time.
        let pick = nav_chrome_order_from(&st)
            .into_iter()
            .rfind(|t| *t != first && *t != NavTab::Lock && !nav_tab_default_hidden(*t))
            .unwrap();
        st.remember_nav_tab(Some(pick));
        assert_eq!(unlocked_home_from(&mut st), screen_for_nav_tab(pick));
        assert_eq!(st.nav_tab_after_unlock, Some(pick));
        // COD and Riders share the Reports screen: unlock shows that one.
        st.remember_nav_tab(Some(NavTab::Cod));
        assert_eq!(unlocked_home_from(&mut st), Screen::Reports);
        assert!(st.reports_nav == ReportsNav::Cod);
        st.remember_nav_tab(Some(NavTab::SetPrinters));
        st.remember_nav_tab(Some(NavTab::Lock));
        assert_eq!(unlocked_home_from(&mut st), Screen::Settings);
        assert_eq!(st.nav_tab_after_unlock, Some(NavTab::SetPrinters));
        // The cashier has their own: first time their first item.
        st.session = Some(cashier.clone());
        let cash_first = first_allowed_work_tab(&st).unwrap();
        assert_eq!(unlocked_home_from(&mut st), screen_for_nav_tab(cash_first));
        let other = nav_chrome_order_from(&st)
            .into_iter()
            .find(|t| {
                *t != cash_first
                    && *t != NavTab::Lock
                    && !nav_tab_default_hidden(*t)
                    && !nav_tab_always_visible(*t)
            })
            .unwrap();
        st.remember_nav_tab(Some(other));
        assert_eq!(unlocked_home_from(&mut st), screen_for_nav_tab(other));
        // Taken off the bar: their first item again, not a dead end.
        st.nav_hidden.push(other);
        assert_eq!(unlocked_home_from(&mut st), screen_for_nav_tab(cash_first));
        // A Settings section without the rights for it: first item too.
        st.remember_nav_tab(Some(NavTab::SetBackup));
        assert_eq!(unlocked_home_from(&mut st), screen_for_nav_tab(cash_first));
        // The owner's survives a restart of the app on the same books.
        let store = st.store.take().unwrap();
        let mut again = AppState::boot();
        fresh(&mut again);
        again.store = Some(store);
        again.session = Some(owner);
        assert_eq!(unlocked_home_from(&mut again), Screen::Settings);
        assert_eq!(again.nav_tab_after_unlock, Some(NavTab::SetPrinters));
        again.store = None;
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn saved_order_wins_and_missing_tabs_take_their_default_place() {
        let order = parse_nav_order(Some("credit,books,bogus,credit"), "en");
        assert_eq!(order.len(), NAV_TABS.len());
        for tab in NAV_TABS {
            assert_eq!(order.iter().filter(|t| **t == tab).count(), 1);
        }
        let at = |t: NavTab| order.iter().position(|x| *x == t).unwrap();
        assert!(at(NavTab::Credit) < at(NavTab::Books));
        assert_eq!(parse_nav_order(Some(&nav_order_csv(&order)), "en"), order);
    }

    /// Default order = names in dictionary order, ignoring case (Khaynjo
    /// owner, 2026-11-15); sections join older layouts hidden on the bars.
    #[test]
    fn default_order_is_by_name_and_sections_start_hidden() {
        for lang in ["en", "hi", "ar"] {
            let names: Vec<String> = default_nav_order(lang).iter().map(|t| nav_tab_label(*t, lang).to_lowercase()).collect();
            let mut sorted = names.clone();
            sorted.sort();
            assert_eq!(names, sorted, "{lang}");
        }
        for shipped in SHIPPED_NAV_ORDERS {
            assert_eq!(parse_nav_order(Some(shipped), "en"), default_nav_order("en"));
        }
        let arranged = "credit,books,sell,buy,rates,ops,cod,riders,sales,day_close,reports,inventory,branches,team,settings,lock";
        let (_, hidden) = parse_nav_layout(Some(arranged), None, "en");
        for t in NAV_TABS.iter().filter(|t| nav_tab_default_hidden(**t)) {
            assert!(hidden.contains(t), "{t:?} hidden on the bars");
        }
        assert!(!hidden.contains(&NavTab::Team), "Team stays where it was");
    }

    #[test]
    fn slot_icons_have_no_layout_picture() {
        let ui = include_str!("../ui_script.rs");
        let mut slots = 0;
        let mut rest = ui;
        while let Some(i) = rest.find("_icon := Image{") {
            let head_start = rest[..i].rfind(char::is_whitespace).map_or(0, |j| j + 1);
            let id = &rest[head_start..i];
            let body_end = rest[i..].find('}').map_or(rest.len(), |j| i + j);
            if id.starts_with("rail_") || id.starts_with("nav_") {
                slots += 1;
                assert!(!rest[i..body_end].contains("src:"), "{id}_icon has a layout src");
            }
            rest = &rest[body_end..];
        }
        assert!(slots >= 2 * 12, "found only {slots} slot icons");
    }

    #[test]
    fn no_saved_order_is_default() {
        assert_eq!(parse_nav_order(None, "en"), default_nav_order("en"));
        assert_eq!(parse_nav_order(Some(""), "en"), default_nav_order("en"));
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

    /// The sidebar's empty corner and the space under its rows move the
    /// window (owner, 2026-11-14). Every button in `nav_rail` must sit in a
    /// nav row (`rail_*_slot`, never a drag) or be in RAIL_CONTROLS, or a
    /// desktop press on it would move the window instead of pressing it.
    #[test]
    fn rail_controls_are_not_window_drag() {
        let ui = include_str!("../ui_script.rs");
        let start = ui.find("nav_rail := SolidView{").expect("nav_rail block in ui_script.rs");
        let mut depth = 0i32;
        let mut slot_depth: Option<i32> = None;
        let mut loose = Vec::new();
        let mut in_slots = 0usize;
        for line in ui[start..].lines() {
            let t = line.trim();
            if let Some((name, rest)) = t.split_once(" := ") {
                let ty = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
                if name.trim().ends_with("_slot") && slot_depth.is_none() {
                    slot_depth = Some(depth);
                } else if ty.contains("Btn") || ty.contains("Button") {
                    if slot_depth.is_some() {
                        in_slots += 1;
                    } else {
                        loose.push(name.trim().to_string());
                    }
                }
            }
            depth += t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if slot_depth.is_some_and(|d| depth <= d) {
                slot_depth = None;
            }
            if depth <= 0 && line.contains('}') {
                break;
            }
        }
        assert!(in_slots >= 2 * NAV_TABS.len(), "rail rows not found — parser drifted ({in_slots})");
        for name in &loose {
            assert!(
                RAIL_CONTROLS.contains(&name.as_str()),
                "sidebar button `{name}` is outside a nav row and missing from RAIL_CONTROLS (a press would move the window)"
            );
        }
        for name in RAIL_CONTROLS {
            assert!(loose.iter().any(|b| b == name), "RAIL_CONTROLS lists `{name}`, which is not a sidebar button");
        }
    }

    /// A window-filling Modal not in INSET_SHEETS draws under the status and
    /// gesture bars on an edge-to-edge phone; an anchored one must stay out.
    #[test]
    fn every_modal_is_an_inset_sheet_or_anchored() {
        const ANCHORED: &[&str] = &["nav_drag_modal", "floor_cols_dd_modal"];
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

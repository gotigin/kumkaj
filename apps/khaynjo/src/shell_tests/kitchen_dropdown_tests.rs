#[cfg(test)]
mod kitchen_dropdown_tests {
    use super::*;

    /// A shop on a temp file: Tandoor, Grill and Bar; Naan routes to Tandoor
    /// on the menu, Chaas has no menu station; table D1; owner signed in.
    struct DeskShop {
        st: AppState,
        path: std::path::PathBuf,
        table_id: String,
    }

    impl Drop for DeskShop {
        fn drop(&mut self) {
            self.st.store = None;
            let _ = std::fs::remove_file(&self.path);
        }
    }

    impl DeskShop {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("khaynjo-kitchen-dd-{}.sqlite3", new_id()));
            let store = Store::open_path(&path).unwrap();
            let owner = store.list_staff().unwrap().into_iter().next().unwrap();
            let tandoor = store.add_station("Tandoor").unwrap();
            store.add_station("Grill").unwrap();
            store.add_station("Bar").unwrap();
            let mut naan = Product::new_basic(new_id(), "Naan", Money::from_rupees(40));
            naan.station_id = tandoor.id.clone();
            store.upsert_product(&naan).unwrap();
            store
                .upsert_product(&Product::new_basic(new_id(), "Chaas", Money::from_rupees(30)))
                .unwrap();
            let table = store.add_table_in_area("D1", "Dine In").unwrap();
            let mut st = AppState::boot();
            st.products = store.list_products(false).unwrap();
            st.stations = store.list_stations().unwrap();
            st.session = Some(owner);
            st.store = Some(store);
            DeskShop { st, path, table_id: table.id }
        }

        fn station(&self, name: &str) -> String {
            self.st.stations.iter().find(|s| s.name == name).unwrap().id.clone()
        }

        fn product(&self, name: &str) -> Product {
            self.st.products.iter().find(|p| p.name == name).unwrap().clone()
        }

        /// Punch dishes on the desk the way the Order screen does: the
        /// remembered kitchen goes on a new dish when it is added.
        fn punch(&mut self, names: &[&str]) {
            for name in names {
                let p = self.product(name);
                self.st.cart.add_or_bump(&p);
                self.st.apply_remembered_kot_to_unassigned();
            }
        }

        /// Seat the desk's dishes on D1 (pending) and open its ticket.
        fn seat_and_open(&mut self) {
            let owner = self.st.session.clone().unwrap();
            let lines = self.st.cart.lines.clone();
            self.st
                .store
                .as_ref()
                .unwrap()
                .seat_unsent_lines(&self.table_id, &lines, &owner)
                .unwrap();
            let tid = self.table_id.clone();
            self.st.load_table_cart(&tid).unwrap();
        }

        fn line(&self, name: &str) -> &CartLine {
            self.st.cart.lines.iter().find(|l| l.name == name).unwrap()
        }

        fn kitchen_of(&self, name: &str) -> String {
            self.line(name).kitchen_station_id(&self.st.products).to_string()
        }

        fn send_names(&self) -> Vec<String> {
            let mut v: Vec<String> =
                self.st.kot_send_destinations().into_iter().map(|(n, _)| n).collect();
            v.sort();
            v
        }

        /// Rows the Kitchen dropdown ticks (action = Don't use).
        fn ticked(&self) -> Vec<String> {
            let dont = self.st.t("dont_use");
            let mut v: Vec<String> = self
                .st
                .suggest_candidates(SuggestField::KotStation)
                .into_iter()
                .filter(|r| r.action == dont)
                .map(|r| r.value)
                .collect();
            v.sort();
            v
        }
    }

    /// The dropdown ticks exactly the kitchens the Send button names. A
    /// remembered-but-unused kitchen (Bar) used to show ticked while Send
    /// went elsewhere.
    #[test]
    fn dropdown_ticks_and_send_button_name_the_same_kitchens() {
        let mut shop = DeskShop::new();
        shop.st.kot_station_default = shop.station("Grill");
        shop.st.kot_stations_selected = vec![shop.station("Bar"), shop.station("Grill")];
        shop.punch(&["Chaas", "Naan"]);
        assert_eq!(shop.kitchen_of("Chaas"), shop.station("Grill"), "new dish: remembered kitchen");
        assert_eq!(shop.kitchen_of("Naan"), shop.station("Tandoor"), "menu kitchen wins");
        assert_eq!(shop.send_names(), ["Grill", "Tandoor"]);
        assert_eq!(shop.ticked(), shop.send_names(), "ticks = where Send goes");
        // Nothing waiting: the tick shows where the next dish goes.
        shop.st.cart.lines.clear();
        assert_eq!(shop.ticked(), ["Grill"]);
    }

    /// A dish taken off its kitchen stays on "Kitchen (no station)": Send
    /// used to put the remembered kitchen back on it and print there.
    #[test]
    fn a_dish_taken_off_its_kitchen_is_not_sent_back_there() {
        let mut shop = DeskShop::new();
        shop.st.kot_station_default = shop.station("Grill");
        shop.punch(&["Chaas"]);
        // Don't use Grill: off the dish, and no longer the next dish's kitchen.
        let grill = shop.station("Grill");
        shop.st.cart.lines[0].station_id.clear();
        shop.st.deselect_kot_station(&grill);
        assert!(shop.st.kot_station_default.is_empty());
        shop.st.apply_remembered_kot_to_unassigned();
        assert_eq!(shop.kitchen_of("Chaas"), "");
        let none = shop.st.t("kot_unassigned");
        assert_eq!(shop.send_names(), [none.clone()]);
        assert_eq!(shop.ticked(), [none], "only Kitchen (no station) is ticked");
    }

    /// The kitchen new dishes go to is the one last ticked — print priority
    /// used to override it, so a dish put on Grill came back as Tandoor.
    #[test]
    fn remembered_kitchen_is_the_last_ticked_not_print_priority() {
        let mut shop = DeskShop::new();
        shop.st.kot_station_priority = vec![shop.station("Tandoor"), shop.station("Grill")];
        shop.st.kot_stations_selected = vec![shop.station("Tandoor"), shop.station("Grill")];
        shop.st.kot_station_default = shop.station("Grill");
        shop.punch(&["Chaas"]);
        assert_eq!(shop.kitchen_of("Chaas"), shop.station("Grill"));
    }

    /// A kitchen picked for a dish already on the table survives the reload
    /// every table action does; so does taking the kitchen off it.
    #[test]
    fn a_table_dish_keeps_its_kitchen_across_reloads() {
        let mut shop = DeskShop::new();
        shop.st.kot_station_default = shop.station("Tandoor");
        shop.punch(&["Chaas"]);
        shop.seat_and_open();
        assert_eq!(shop.kitchen_of("Chaas"), shop.station("Tandoor"));
        // The waiter moves it to Grill from the dropdown.
        let rid = shop.line("Chaas").running_line_id.clone();
        assert!(is_table_running_line(&rid), "seated on the order: {rid}");
        let grill = shop.station("Grill");
        let i = shop.st.cart.lines.iter().position(|l| l.running_line_id == rid).unwrap();
        shop.st.cart.lines[i].station_id = grill.clone();
        shop.st.remember_line_station(&rid, &grill);
        let tid = shop.table_id.clone();
        shop.st.load_table_cart(&tid).unwrap();
        assert_eq!(shop.kitchen_of("Chaas"), grill, "the pick, not the remembered kitchen");
        // …and off every kitchen: still none after the next reload.
        shop.st.remember_line_station(&rid, "");
        shop.st.load_table_cart(&tid).unwrap();
        assert_eq!(shop.kitchen_of("Chaas"), "");
    }

    /// Holding from the dropdown is written to the table order, like the
    /// line's own Hold — a cart-only hold came back unheld on refresh.
    #[test]
    fn a_dropdown_hold_on_a_table_survives_reload() {
        let mut shop = DeskShop::new();
        shop.punch(&["Chaas", "Naan"]);
        shop.seat_and_open();
        let i = shop.st.cart.lines.iter().position(|l| l.name == "Chaas").unwrap();
        shop.st.hold_unsent_lines(&[i]).unwrap();
        let tid = shop.table_id.clone();
        shop.st.load_table_cart(&tid).unwrap();
        assert!(shop.line("Chaas").hold_send, "held on the order");
        assert!(!shop.line("Naan").hold_send);
        assert_eq!(shop.send_names(), ["Tandoor"], "a held dish is not sent");
    }

    /// With stations in the shop, a dish with no kitchen gets no paper.
    #[test]
    fn no_station_dishes_get_no_paper_once_the_shop_has_stations() {
        let shop = DeskShop::new();
        assert!(!shop.st.station_gets_kot_paper(""));
        assert!(shop.st.station_gets_kot_paper(&shop.station("Grill")));
        let mut single = AppState::boot();
        single.stations.clear();
        assert!(single.station_gets_kot_paper(""), "one-kitchen shop keeps its paper");
    }

    /// Menu kitchen is on the line when punched — Don't use clears it to
    /// "Kitchen (no station)" (no Hold) and Send must not print paper.
    #[test]
    fn send_with_cleared_station_does_not_print_menu_kitchen() {
        let mut shop = DeskShop::new();
        shop.punch(&["Naan"]);
        assert_eq!(
            shop.st.cart.lines[0].station_id,
            shop.station("Tandoor"),
            "menu kitchen lands on the line at punch"
        );
        // Same as Don't use Tandoor on the Order dropdown.
        shop.st.cart.lines[0].station_id.clear();
        shop.st.remember_line_station(&shop.st.cart.lines[0].running_line_id.clone(), "");
        shop.st.deselect_kot_station(&shop.station("Tandoor"));
        let none = shop.st.t("kot_unassigned");
        assert_eq!(shop.send_names(), [none.clone()]);
        assert!(!shop.st.cart.lines[0].hold_send, "Don't use is clear, not Hold");
        let owner = shop.st.session.clone().unwrap();
        let draft = shop.st.cart.to_kot_draft().unwrap();
        assert!(
            draft.lines.iter().all(|l| l.station_id.trim().is_empty()),
            "draft must carry empty station, not the menu id"
        );
        let store = shop.st.store.as_ref().unwrap();
        let kot_ids = store.send_kot(&draft, None, &owner).unwrap();
        assert_eq!(kot_ids.len(), 1);
        assert_eq!(store.station_id_for_kot(&kot_ids[0]).unwrap(), "");
        assert!(
            !store.kot_gets_paper(&kot_ids[0]),
            "no-station KOT must not get paper"
        );
        assert!(
            net::print_sent_kots(store, &kot_ids).is_none(),
            "auto-print must skip no-station tickets"
        );
        assert!(
            store.print_kot(&kot_ids[0]).is_err(),
            "print_kot itself must refuse no-station paper"
        );
    }

    /// A seated Don't use stays cleared across reload — menu must not bounce
    /// back onto the dish and print that kitchen.
    #[test]
    fn seated_dont_use_survives_reload_without_menu_bounce() {
        let mut shop = DeskShop::new();
        shop.punch(&["Naan"]);
        shop.seat_and_open();
        let rid = shop.line("Naan").running_line_id.clone();
        let i = shop.st.cart.lines.iter().position(|l| l.running_line_id == rid).unwrap();
        shop.st.cart.lines[i].station_id.clear();
        shop.st.remember_line_station(&rid, "");
        shop.st
            .store
            .as_ref()
            .unwrap()
            .set_pending_line_station(&rid, "", &shop.st.session.clone().unwrap())
            .unwrap();
        shop.st.deselect_kot_station(&shop.station("Tandoor"));
        shop.st.kot_line_station.clear(); // as after restart
        let tid = shop.table_id.clone();
        shop.st.load_table_cart(&tid).unwrap();
        assert_eq!(shop.kitchen_of("Naan"), "", "cleared stay cleared");
        let none = shop.st.t("kot_unassigned");
        assert_eq!(shop.send_names(), [none]);
    }

    /// Send button label and Kitchen dropdown ticks stay the same after Use /
    /// Don't use on the open checklist (the face under the dropdown).
    #[test]
    fn use_and_dont_use_keep_send_button_and_ticks_aligned() {
        let mut shop = DeskShop::new();
        shop.punch(&["Chaas"]);
        assert_eq!(shop.kitchen_of("Chaas"), "");
        shop.st.select_kot_station(&shop.station("Grill"));
        shop.st.apply_remembered_kot_to_unassigned();
        assert_eq!(shop.kitchen_of("Chaas"), shop.station("Grill"));
        assert_eq!(shop.send_names(), shop.ticked());
        assert_eq!(shop.send_names(), ["Grill"]);
        // Don't use Grill: off the dish and off the next-dish default.
        shop.st.cart.lines[0].station_id.clear();
        shop.st.deselect_kot_station(&shop.station("Grill"));
        shop.st.apply_remembered_kot_to_unassigned();
        let none = shop.st.t("kot_unassigned");
        assert_eq!(shop.send_names(), [none.clone()]);
        assert_eq!(shop.ticked(), [none]);
        assert!(shop.st.kot_station_default.is_empty());
    }
}

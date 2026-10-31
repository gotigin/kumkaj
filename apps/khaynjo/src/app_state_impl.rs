impl AppState {
    fn boot() -> Self {
        let mut state = Self {
            store: None,
            products: Vec::new(),
            cart: Cart::default(),
            amend_cart: Cart::default(),
            amend_pay: PayMethod::Cash,
            sale_revisions: Vec::new(),
            selected_table_id: None,
            selected_customer_id: None,
            crm_segment: store::CrmSegment::BirthdayThisWeek,
            pending_loyalty_points: 0,
            pending_wallet_paise: 0,
            pending_voucher_code: String::new(),
            pending_voucher_paise: 0,
            customers: Vec::new(),
            tables: Vec::new(),
            recent_sales: Vec::new(),
            staff: Vec::new(),
            session: None,
            lock_as_owner: true,
            lock_staff_id: None,
            draft_staff_role: StaffRole::Cashier,
            draft_features: StaffFeatures::for_role(StaffRole::Cashier),
            team_staff_id: None,
            device_role: DeviceRole::Unset,
            must_confirm_device_role: true,
            language_follow_platform: true,
            picking_server: false,
            local_servers: Vec::new(),
            lan_devices: Vec::new(),
            data_base: None,
            printer_draft: print_share::DevicePrinters::default(),
            printer_route_key: "bill".into(),
            settings_panel: None,
            printer_selected_id: None,
            owner_setup_complete: false,
            onboarding_complete: false,
            venue_mode: VenueMode::Restaurant,
            client_paired: false,
            extra_tills_allowed: false,
            onboard_step: 1,
            entity_template_id: "khaynjo_restaurant".into(),
            money_lane: MoneyLane::Declared,
            buy_on_account: false,
            day_close_text: String::new(),
            ops_panel_text: String::new(),
            floor_waiter_filter: String::new(),
            floor_view_layout: false,
            floor_layout_sel: None,
            branches_text: String::new(),
            screen: Screen::DeviceSetup,
            status: String::new(),
            last_receipt: "No sale yet".into(),
            undo_sale_id: None,
            undo_snack_text: String::new(),
            undo_timer: Timer::empty(),
            snack_text: String::new(),
            snack_kind: SnackKind::Info,
            snack_timer: Timer::empty(),
            live_timer: Timer::empty(),
            pin_buffer: String::new(),
            pin_visible: false,
            selected_sale_id: None,
            selected_expense_id: None,
            attach_pick: None,
            locale: LocaleSettings::default(),
            fiscal: FiscalSettings::default(),
            country_browse: String::new(),
            venue_title: String::new(),
            venue_icon_key: None,
            nav_order: NAV_TABS.to_vec(),
            nav_hidden: Vec::new(),
            amounts_screen_hidden: false,
            amount_keys_hidden: Vec::new(),
            amount_keys_peek: Vec::new(),
            books_sections_open: Vec::new(),
            floor_sections_open: Vec::new(),
            device_setup_sections_open: vec!["role".into()],
            draft_venue_icon_path: None,
            pending_backup_copy: None,
            pending_diag_export: None,
            draft_image_key: None,
            pay_parts: vec![PayPart::default()],
            open_kots: Vec::new(),
            stations: Vec::new(),
            categories: Vec::new(),
            dish_picker_open: false,
            dish_picks: Vec::new(),
            dish_picks_rev: 0,
            category_picker_open: false,
            category_picks: Vec::new(),
            category_picks_rev: 0,
            form_category_photo: None,
            form_category_auto: false,
            menu_filter: String::new(),
            menu_pick_open: false,
            menu_pick_rev: 0,
            line_action: None,
            open_cart_line: None,
            picking_table: false,
            picking_from_table_id: None,
            suggest_open: None,
            suggest_rows: Vec::new(),
            suggest_rev: 0,
            team_select_pending: false,
            pay_choice: PayChoice::Cash,
            template_confirm: false,
            menu_list_filter: String::new(),
            menu_unpriced_only: false,
            form_resale: false,
            kot_heads: std::collections::HashMap::new(),
            raw_materials: Vec::new(),
            reorder_levels: std::collections::HashMap::new(),
            low_stock_line: String::new(),
            kitchen_counts_line: String::new(),
            reservations: Vec::new(),
            menu_category: None,
            frequent_ids: Vec::new(),
            frequent_days: 1,
            frequent_limit: 24,
            line_cooked: false,
            move_panel_open: false,
            split_mode: false,
            split_pick: Vec::new(),
            discount_panel_open: false,
            found_servers: Vec::new(),
            category_photo_id: None,
            removed_products: Vec::new(),
            form_category_id: None,
            form_category_label: String::new(),
            editing_product_id: None,
            pay_in_flight: false,
            lan_sign_in_busy: false,
            lan_shown: phone::LinkState::default(),
            pair_note: String::new(),
            failed_unlocks: 0,
            unlock_blocked_until: None,
            pin_reset_required: false,
            nav_back: Vec::new(),
            nav_forward: Vec::new(),
            rail_width: RAIL_WIDTH_DEFAULT,
            rail_collapsed: false,
            books_report_kind: BooksReportKind::Today,
        };
        if let Ok(dir) = store::data_dir() {
            state.data_base = Some(dir.clone());
            print_share::use_device_dir(&dir);
            state.printer_draft = print_share::printers();
            let _ = state.refresh_local_servers();
        }
        match Store::open_default() {
            Ok(store) => {
                state.status = String::new();
                state.store = Some(store);
                let _ = state.reload_all();
                // Always require PIN after a process start. Session restore was
                // skipping the lock screen and jumping straight into Sell.
                if let Some(store) = state.store.as_mut() {
                    let _ = store.set_active_session_staff_id(None);
                }
                state.session = None;
                state.pin_buffer.clear();
                state.route_after_stored_role();
            }
            Err(e) => {
                // No local server yet is normal — DeviceSetup will create/pick.
                state.status = if e.contains("No local server") {
                    String::new()
                } else {
                    format!("Storage: {e}")
                };
                state.session = None;
                state.must_confirm_device_role = true;
                state.apply_platform_language_preview();
                state.screen = Screen::DeviceSetup;
                state.pin_buffer.clear();
            }
        }
        state
    }

    fn refresh_local_servers(&mut self) -> Result<(), String> {
        let base = self
            .data_base
            .clone()
            .or_else(|| store::data_dir().ok())
            .ok_or_else(|| "No data directory".to_string())?;
        self.data_base = Some(base.clone());
        print_share::use_device_dir(&base);
        self.printer_draft = print_share::printers();
        self.local_servers = local_servers::list_local_servers(&base)?;
        Ok(())
    }

    fn reload_all(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        if self.device_role.is_host() {
            self.lan_devices = store.list_lan_devices().unwrap_or_default();
            // A split bill's companion table goes once its bill is paid.
            let _ = store.tidy_split_tables();
        }
        // The menu (what is sold) and the raw materials (what the kitchen
        // buys and uses up) are separate lists: nothing raw ever reaches the
        // order desk or the menu screen.
        let (raw, menu): (Vec<Product>, Vec<Product>) =
            store.list_products(false)?.into_iter().partition(|p| p.is_raw());
        self.products = menu;
        self.reorder_levels = store.reorder_levels().unwrap_or_default();
        self.kitchen_counts_line = kitchen_counts_line(store);
        self.reservations = store.upcoming_reservations().unwrap_or_default();
        self.low_stock_line = store
            .low_stock()
            .unwrap_or_default()
            .iter()
            .map(|l| {
                let unit = l.product.unit_label.trim();
                let qty = l.product.stock_qty.max(0);
                if unit.is_empty() { format!("{} {qty}", l.product.name) } else { format!("{} {qty} {unit}", l.product.name) }
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.raw_materials = raw;
        self.removed_products = store.list_removed_products()?;
        self.tables = store.list_tables()?;
        // Floor is grid-only (Layout mode / tabs removed).
        self.floor_view_layout = false;
        let _ = store.set_floor_view("grid");
        self.recent_sales = store.recent_sales(30)?;
        self.staff = store.list_staff()?;
        self.customers = store.list_customers().unwrap_or_default();
        self.locale = store.locale_settings()?;
        self.fiscal = store.fiscal_settings()?;
        self.device_role = DeviceRole::from_meta(store.device_role()?.as_deref());
        self.language_follow_platform =
            store.language_source().unwrap_or(LanguageSource::User) == LanguageSource::Platform;
        self.extra_tills_allowed = store.extra_tills_allowed().unwrap_or(false);
        self.venue_title = store.venue_title()?;
        self.venue_icon_key = store.venue_icon_key()?;
        self.nav_order = parse_nav_order(store.nav_order_csv()?.as_deref());
        self.nav_hidden = parse_nav_hidden(store.nav_hidden_csv()?.as_deref());
        if let Some(user) = self.session.as_ref().map(lan_key) {
            self.amounts_screen_hidden = store.amounts_screen_hidden(&user).unwrap_or(false);
            self.amount_keys_hidden = parse_amount_keys_hidden(
                store.amount_keys_hidden_csv(&user).ok().flatten().as_deref(),
            );
            self.amount_keys_peek = parse_amount_keys_hidden(
                store.amount_keys_peek_csv(&user).ok().flatten().as_deref(),
            );
        }
        self.books_sections_open = parse_books_sections(store.books_sections_csv()?.as_deref());
        self.open_kots = store.list_open_kots().unwrap_or_default();
        self.stations = store.list_stations().unwrap_or_default();
        self.categories = store.list_categories().unwrap_or_default();
        self.owner_setup_complete = store.owner_setup_complete().unwrap_or(true);
        self.onboarding_complete = store.onboarding_complete().unwrap_or(true);
        // Khaynjo product: restaurant venue (tables + KOT). Counter-only
        // shops simply never open the floor; the venue stays Restaurant so
        // KOT creation is available the moment tables are enabled.
        if store.venue_mode().unwrap_or(VenueMode::Restaurant) != VenueMode::Restaurant {
            let _ = store.set_venue_mode(VenueMode::Restaurant);
        }
        self.venue_mode = VenueMode::Restaurant;
        self.client_paired = store.client_paired().unwrap_or(false);
        self.pin_reset_required = store.pin_reset_required().unwrap_or(false);
        self.entity_template_id = store
            .entity_template_id()
            .unwrap_or_else(|_| "custom".into());
        self.refresh_frequent_cache();
        Ok(())
    }

    fn uses_lan(&self) -> bool {
        self.device_role == DeviceRole::Server && self.extra_tills_allowed
    }

    /// Every process start: language + Server/Client on one screen.
    /// UI opens in the platform language so non-English users can read it;
    /// changing language updates this screen immediately.
    fn route_after_stored_role(&mut self) {
        self.must_confirm_device_role = true;
        self.picking_server = false;
        let _ = self.refresh_local_servers();
        self.apply_platform_language_preview();
        self.screen = Screen::DeviceSetup;
        self.status = match self.device_role {
            DeviceRole::Server => self.t("last_server_mode"),
            DeviceRole::Client => self.t("last_client_mode"),
            DeviceRole::Unset => self.t("device_use_q"),
        };
        // Warm LAN only after they confirm Server + extra tills on Lock path.
        // Do not start the server while the chooser is open.
    }

    /// The app-global remembered language choice for this install, if any.
    /// One choice for the whole device — not per shop, not per staff role.
    fn remembered_language(&self) -> Option<(bool, String)> {
        let base = self.data_base.as_ref()?;
        local_servers::load_app_language(base)
    }

    /// Apply the remembered app-global language (and sync the opened shop's
    /// books file so store-side formatting matches). Returns false when the
    /// user has never picked a language on this install.
    fn apply_remembered_language(&mut self) -> bool {
        let Some((follow, code)) = self.remembered_language() else {
            return false;
        };
        self.language_follow_platform = follow;
        if follow {
            self.locale.language = locale::detect_platform_language();
            if let Some(store) = self.store.as_ref() {
                let _ = store.set_platform_language();
            }
        } else if locale::language_by_code(&code).is_some() {
            self.locale.language = code.clone();
            if let Some(store) = self.store.as_ref()
                && let Ok(loc) = store.set_user_language(&code)
            {
                self.locale = loc;
            }
        }
        true
    }

    /// Remember the current language choice app-globally (this device).
    fn save_remembered_language(&self) {
        if let Some(base) = self.data_base.as_ref() {
            let _ = local_servers::save_app_language(
                base,
                self.language_follow_platform,
                &self.locale.language,
            );
        }
    }

    /// DeviceSetup previews in the remembered language when the user has
    /// picked one on this install; only a first-ever run previews in the
    /// OS language. Never overwrite a remembered choice with the OS default.
    fn apply_platform_language_preview(&mut self) {
        if self.apply_remembered_language() {
            return;
        }
        self.language_follow_platform = true;
        let platform = locale::detect_platform_language();
        self.locale.language = platform;
    }

    /// Guard for anything the shop device alone may write. An extra till's catalogue,
    /// stock, staff and settings tables are a sync cache, so editing them here
    /// would vanish on the next snapshot.
    fn require_server_role(&self, what: &str) -> Result<(), String> {
        if self.device_role == DeviceRole::Client {
            return Err(format!("{what} on the shop device — this is an extra till"));
        }
        Ok(())
    }

    /// LAN details a Client needs to ask the Server to do the writing.
    /// Hand a write to the phone's LAN worker (Client only). `queued`
    /// writes (KOT, payment, "bill, please") wait through a Wi‑Fi drop;
    /// the rest need the Server now. `done` is the status line once the
    /// Server agreed; its refusal shows in its own words instead.
    fn lan_job(
        &self,
        path: &str,
        body: &impl serde::Serialize,
        label: String,
        done: String,
        follow: phone::Follow,
        queued: bool,
    ) -> Result<String, String> {
        let actor = self.actor()?;
        let store = self.store.as_ref().ok_or("No store")?;
        if !store.client_paired()? {
            return Err("Pair this device with the shop first".into());
        }
        let id = new_id();
        let job = phone::Job {
            id: id.clone(),
            path: path.into(),
            body: serde_json::to_string(body).map_err(|e| e.to_string())?,
            label,
            done,
            follow,
            table_id: self.selected_table_id.clone().unwrap_or_default(),
            staff_key: lan_key(&actor),
            queued,
        };
        phone::send(store, job)?;
        Ok(id)
    }

    /// Rights shown on the Team screen for the selected staff: the draft if
    /// the owner touched it, else that person's role preset.
    fn team_draft_rights(&self) -> domain::StaffRights {
        if let Some(r) = &self.draft_features.rights {
            return r.clone();
        }
        let role = self
            .team_staff_id
            .as_ref()
            .and_then(|id| self.staff.iter().find(|s| &s.id == id))
            .map(|s| s.role)
            .unwrap_or(StaffRole::Cashier);
        domain::StaffRights::for_role(role)
    }

    /// On the Server: the floor and the kitchen read again from the books
    /// (a waiter's KOT, a payment from a tablet just landed). Today's
    /// figures follow in `sync_chrome`, which reads them itself.
    fn refresh_live_lists(&mut self) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let tables = store.list_tables().unwrap_or_default();
        let kots = store.list_open_kots().unwrap_or_default();
        let counts = kitchen_counts_line(store);
        self.tables = tables;
        self.open_kots = kots;
        self.kitchen_counts_line = counts;
    }

    /// A paired device asks for the Server's books (tables, running orders,
    /// KOTs …); the phone worker fetches them and the screen reloads when
    /// they land. Never waits. No-op on the Server.
    fn pull_from_server(&mut self) {
        if self.device_role == DeviceRole::Client {
            phone::want_sync();
        }
    }

    /// Punch ± qty onto the open table's pending lines (Server SoT).
    /// No table selected: caller keeps lines in the local cart only.
    fn bump_table_pending(
        &mut self,
        product_id: &str,
        delta_milli: i64,
        done: &str,
    ) -> Result<Acted, String> {
        let table_id = self
            .selected_table_id
            .clone()
            .ok_or_else(|| "Open a table first".to_string())?;
        let area = self.current_area();
        self.table_action(
            net::RunningAction::BumpPending {
                table_id,
                product_id: product_id.to_string(),
                delta_milli,
                area,
            },
            "",
            done,
            phone::Follow::Refresh,
        )
    }

    /// Open a table on the Order desk from the Server running order (pending +
    /// sent). First-seat / Change-table side-effects live in `open_table`.
    fn load_table_cart(&mut self, table_id: &str) -> Result<(), String> {
        if self.selected_table_id.as_deref() != Some(table_id) {
            self.split_mode = false;
            self.split_pick.clear();
        }
        let lines = self
            .store
            .as_ref()
            .ok_or("No store")?
            .running_order(table_id)?
            .map(|o| o.to_cart_lines())
            .unwrap_or_default();
        self.cart.clear();
        self.open_cart_line = None;
        self.cart.set_order_type(OrderType::DineIn);
        self.cart.lines = lines;
        self.cart.lines.extend(self.queued_kot_lines(table_id));
        self.selected_table_id = Some(table_id.to_string());
        let mut ids: Vec<String> = Vec::new();
        for l in &self.cart.lines {
            if !l.kot_id.is_empty() && !ids.contains(&l.kot_id) {
                ids.push(l.kot_id.clone());
            }
        }
        if let Some(store) = self.store.as_ref() {
            for (id, no, at) in store.kot_heads(&ids).unwrap_or_default() {
                // "2026-10-23T20:40:12+05:30" → "20:40"
                let time = at.get(11..16).unwrap_or("").to_string();
                self.kot_heads.insert(id, (no, time));
            }
        }
        Ok(())
    }

    /// First seat: write the desk's unsent lines onto `table_id` as pending.
    fn seat_cart_onto_table(&mut self, table_id: &str) -> Result<(), String> {
        let to_seat: Vec<CartLine> = self
            .cart
            .lines
            .iter()
            .filter(|l| !l.is_sent() && l.qty_milli > 0)
            .cloned()
            .collect();
        if to_seat.is_empty() {
            return Ok(());
        }
        let actor = self.actor()?;
        if self.device_role == DeviceRole::Client {
            let _ = self.table_action(
                net::RunningAction::SeatPending {
                    table_id: table_id.to_string(),
                    lines: to_seat.clone(),
                },
                "",
                "Seated",
                phone::Follow::Refresh,
            )?;
            // Host may not have applied yet — keep punches visible until sync.
            self.selected_table_id = Some(table_id.to_string());
            self.cart.set_order_type(OrderType::DineIn);
            self.cart.lines = to_seat;
            return Ok(());
        }
        let store = self.store.as_ref().ok_or("No store")?;
        store.seat_unsent_lines(table_id, &to_seat, &actor)?;
        Ok(())
    }

    /// A device's KOTs for this table still waiting to reach the Server,
    /// as ticket lines under "Waiting to send" (rebuilt from the saved
    /// order, so they show again after a restart).
    fn queued_kot_lines(&self, table_id: &str) -> Vec<CartLine> {
        if self.device_role != DeviceRole::Client {
            return Vec::new();
        }
        let Some(store) = self.store.as_ref() else {
            return Vec::new();
        };
        let mut lines = Vec::new();
        for item in store.lan_outbox().unwrap_or_default() {
            if item.path != "/kot" || item.table_id != table_id {
                continue;
            }
            let Ok(req) = serde_json::from_str::<net::RemoteKotRequest>(&item.body) else {
                continue;
            };
            for l in req.draft.lines.into_iter().filter(|l| !l.kot_sent) {
                lines.push(CartLine {
                    product_id: l.product_id,
                    name: l.name,
                    unit_price: Money(l.unit_price_paise),
                    qty: l.qty,
                    qty_milli: l.qty_milli,
                    unit_label: l.unit_label,
                    note: l.note,
                    base_unit_paise: l.unit_price_paise,
                    running_line_id: format!("{QUEUED_LINE}{}", item.id),
                    kot_id: QUEUED_KOT.into(),
                    ..CartLine::default()
                });
            }
        }
        lines
    }

    /// Run a table action where the books are: on the Server directly, or
    /// over the LAN from a paired device. Rights are checked by the Server
    /// either way. `done` is what to say once it is done; a device says it
    /// when the Server's answer comes back.
    fn table_action(
        &mut self,
        action: net::RunningAction,
        approval_pin: &str,
        done: &str,
        follow: phone::Follow,
    ) -> Result<Acted, String> {
        let actor = self.actor()?;
        if self.device_role == DeviceRole::Client {
            self.lan_job(
                "/running",
                &net::RemoteRunningRequest {
                    approval_pin: approval_pin.trim().to_string(),
                    request: action,
                },
                done.to_string(),
                done.to_string(),
                follow,
                false,
            )?;
            return Ok(Acted::Sent);
        }
        let store = self.store.as_ref().ok_or("No store")?;
        let pin = Some(approval_pin).filter(|p| !p.trim().is_empty());
        let text = net::apply_running_action(store, &actor, pin, &action)?;
        self.reload_all()?;
        if let Some(tid) = self.selected_table_id.clone() {
            self.load_table_cart(&tid)?;
        }
        Ok(Acted::Here(text))
    }

    /// Floor area of the table open on the Order desk ("" = counter / no
    /// area), for area prices.
    fn current_area(&self) -> String {
        self.selected_table_id
            .as_ref()
            .and_then(|id| self.tables.iter().find(|t| &t.id == id))
            .map(|t| t.area.clone())
            .unwrap_or_default()
    }

    /// What a "type or pick" field can offer, before narrowing.
    fn suggest_candidates(&self, field: SuggestField) -> Vec<suggest::SuggestRow> {
        use suggest::SuggestRow;
        let lang = self.locale.language.clone();
        let parties = |suppliers_first: bool| {
            let mut v: Vec<&Customer> = self.customers.iter().filter(|c| c.active).collect();
            if suppliers_first {
                v.sort_by_key(|c| !c.kind_label.to_lowercase().contains("supplier"));
            }
            v.into_iter()
                .map(|c| {
                    let sub = [c.kind_label.trim(), c.phone.trim()]
                        .into_iter()
                        .filter(|x| !x.is_empty())
                        .collect::<Vec<_>>()
                        .join(" · ");
                    SuggestRow::new(c.name.clone(), sub)
                })
                .collect::<Vec<_>>()
        };
        let units = || {
            let all = self
                .store
                .as_ref()
                .and_then(|s| s.list_units().ok())
                .unwrap_or_default();
            // Shop-enabled units only; still offer a unit already on the
            // product being edited so a disabled code stays editable.
            let editing = self
                .editing_product_id
                .as_ref()
                .and_then(|id| self.products.iter().find(|p| &p.id == id))
                .map(|p| p.unit_label.trim().to_ascii_lowercase())
                .filter(|u| !u.is_empty());
            let keep = |code: &str| {
                all.iter().any(|u| {
                    u.code == code
                        && (u.picker_enabled || editing.as_ref().is_some_and(|e| e == code))
                }) || (!code.is_empty() && all.iter().all(|u| u.code != code))
            };
            let mut v: Vec<SuggestRow> = all
                .iter()
                .filter(|u| u.picker_enabled || editing.as_ref().is_some_and(|e| e == &u.code))
                .map(|u| SuggestRow::new(u.code.clone(), u.name.clone()))
                .collect();
            // What the menu already uses comes first.
            let mut used: Vec<SuggestRow> = self
                .products
                .iter()
                .map(|p| p.unit_label.trim().to_string())
                .filter(|u| !u.is_empty() && keep(u))
                .map(|u| SuggestRow::new(u, ""))
                .collect();
            used.append(&mut v);
            used
        };
        match field {
            SuggestField::MoveTable | SuggestField::LineTable => {
                let current = self.selected_table_id.clone();
                self.tables
                    .iter()
                    .filter(|t| Some(&t.id) != current.as_ref())
                    .map(|t| {
                        let state = locale::t(&lang, t.floor_state.tile_key());
                        let sub = if t.area.is_empty() { state } else { format!("{} · {state}", t.area) };
                        SuggestRow::new(t.name.clone(), sub)
                    })
                    .collect()
            }
            SuggestField::TableArea => {
                let enabled: Vec<String> = self
                    .store
                    .as_ref()
                    .and_then(|s| s.list_picker_areas().ok())
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|(_, on)| *on)
                    .map(|(n, _)| n)
                    .collect();
                let mut areas: Vec<String> = Vec::new();
                for t in &self.tables {
                    if !t.area.is_empty() && !areas.contains(&t.area) {
                        if enabled.is_empty() || enabled.iter().any(|e| e.eq_ignore_ascii_case(&t.area))
                        {
                            areas.push(t.area.clone());
                        }
                    }
                }
                for a in enabled {
                    if !areas.iter().any(|x| x.eq_ignore_ascii_case(&a)) {
                        areas.push(a);
                    }
                }
                areas
                    .into_iter()
                    .map(|a| {
                        let n = self.tables.iter().filter(|t| t.area == a).count();
                        SuggestRow::new(a, format!("{n}"))
                    })
                    .collect()
            }
            SuggestField::MenuStation => self
                .store
                .as_ref()
                .and_then(|s| s.list_picker_stations().ok())
                .unwrap_or_else(|| self.stations.clone())
                .into_iter()
                .map(|st| SuggestRow::new(st.name.clone(), ""))
                .collect(),
            // All stations (incl. picker-off) — remove needs the full list.
            SuggestField::StationRemove | SuggestField::SettingsStationRemove => self
                .stations
                .iter()
                .map(|st| SuggestRow::new(st.name.clone(), ""))
                .collect(),
            // Order desk: stations already on unsent dishes show Don't use
            // (clear), others show Use. Unassigned dishes get a Don't use row
            // that holds them off this KOT (same as Hold on the line).
            SuggestField::KotStation => {
                let stations = self
                    .store
                    .as_ref()
                    .and_then(|s| s.list_picker_stations().ok())
                    .unwrap_or_else(|| self.stations.clone());
                let mut counts: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                let mut unassigned_n = 0usize;
                for line in self.cart.lines.iter().filter(|l| !l.is_sent() && !l.hold_send) {
                    let sid = self
                        .products
                        .iter()
                        .find(|p| p.id == line.product_id)
                        .map(|p| p.station_id.as_str())
                        .unwrap_or("");
                    if sid.is_empty() {
                        unassigned_n += 1;
                    } else {
                        *counts.entry(sid.to_string()).or_default() += 1;
                    }
                }
                let use_l = self.t("use");
                let clear_l = self.t("dont_use");
                let mut rows = Vec::new();
                if unassigned_n > 0 {
                    rows.push(
                        SuggestRow::new(self.t("kot_unassigned"), format!("{unassigned_n}"))
                            .with_action(clear_l.clone()),
                    );
                }
                rows.extend(stations.into_iter().map(|st| {
                    let n = counts.get(&st.id).copied().unwrap_or(0);
                    if n > 0 {
                        SuggestRow::new(st.name.clone(), format!("{n}"))
                            .with_action(clear_l.clone())
                    } else {
                        SuggestRow::new(st.name.clone(), "").with_action(use_l.clone())
                    }
                }));
                rows
            }
            SuggestField::MenuUnit | SuggestField::ConvFrom | SuggestField::ConvTo | SuggestField::BuyUnit => units(),
            SuggestField::MenuTax => self
                .store
                .as_ref()
                .and_then(|s| s.list_picker_tax_rates().ok())
                .unwrap_or_default()
                .into_iter()
                .filter(|r| r.active)
                .map(|r| {
                    let pct = format!("{:.2}", r.bps as f64 / 100.0);
                    let pct = format!("{}%", pct.trim_end_matches('0').trim_end_matches('.'));
                    SuggestRow::new(r.name, pct)
                })
                .collect(),
            SuggestField::MenuGroup => self
                .products
                .iter()
                .filter(|p| !p.group_name.trim().is_empty())
                .map(|p| SuggestRow::new(p.group_name.trim(), ""))
                .collect(),
            // A restaurant buys raw material (and a few goods it resells) —
            // never its own dishes.
            SuggestField::BuyItem | SuggestField::InvItem => self
                .stock_items()
                .into_iter()
                .map(|p| {
                    let kind = locale::t(&lang, if p.is_raw() { "kind_raw" } else { "kind_resale" });
                    let unit = p.unit_label.trim();
                    let sub = format!(
                        "{kind} · {} {}{}",
                        p.stock_qty.max(0),
                        unit,
                        if unit.is_empty() { locale::t(&lang, "in_stock") } else { String::new() }
                    );
                    SuggestRow::new(p.name.clone(), sub.trim().to_string())
                })
                .collect(),
            SuggestField::BuySupplier => parties(true),
            SuggestField::CreditParty => parties(false),
            SuggestField::TeamPick => self
                .staff
                .iter()
                .map(|m| SuggestRow::new(m.name.clone(), m.lock_label()))
                .collect(),
            // System printers found on this device (network address is typed).
            SuggestField::PrinterTarget => {
                let here = self.t("printers_heading");
                print_share::found_printers()
                    .into_iter()
                    .map(|name| SuggestRow::new(name, here.clone()))
                    .collect()
            }
            SuggestField::RoutePick => {
                // Bill + real stations only — no invented "No station" row.
                let mut rows = vec![SuggestRow::new(self.t("route_bill"), "")];
                for s in &self.stations {
                    rows.push(SuggestRow::new(s.name.clone(), ""));
                }
                rows
            }
            // Who took deliveries before, then the active staff.
            SuggestField::DeliveryBy => {
                let mut rows: Vec<SuggestRow> = self
                    .store
                    .as_ref()
                    .and_then(|s| s.delivery_people().ok())
                    .unwrap_or_default()
                    .into_iter()
                    .map(|n| SuggestRow::new(n, String::new()))
                    .collect();
                rows.extend(
                    self.staff
                        .iter()
                        .filter(|m| m.active)
                        .map(|m| SuggestRow::new(m.name.clone(), m.lock_label())),
                );
                rows
            }
            // Code to apply, country name beside it (typing either finds it).
            SuggestField::Country => fiscal::COUNTRY_PROFILES
                .iter()
                .map(|c| SuggestRow::new(c.code, c.name_en))
                .collect(),
            SuggestField::BranchSwitch => self
                .store
                .as_ref()
                .and_then(|s| s.list_branches().ok())
                .unwrap_or_default()
                .into_iter()
                .filter(|b| b.active)
                .map(|b| SuggestRow::new(b.name, b.code))
                .collect(),
            SuggestField::CustomerName => parties(false),
            SuggestField::ResvTable => self
                .tables
                .iter()
                .map(|t| {
                    let state = locale::t(&lang, t.floor_state.tile_key());
                    let sub = if t.area.is_empty() {
                        state
                    } else {
                        format!("{} · {state}", t.area)
                    };
                    SuggestRow::new(t.name.clone(), sub)
                })
                .collect(),
            SuggestField::FloorWaiter => self
                .staff
                .iter()
                .filter(|m| m.active)
                .map(|m| SuggestRow::new(m.name.clone(), m.lock_label()))
                .collect(),
            SuggestField::BulkCat => self
                .categories
                .iter()
                .map(|c| SuggestRow::new(c.name.clone(), ""))
                .collect(),
        }
    }

    /// Open (or re-narrow) a field's list; closes every other picker.
    fn open_suggest(&mut self, field: SuggestField, query: &str) {
        self.dish_picker_open = false;
        self.category_picker_open = false;
        self.suggest_rows = suggest::rank(self.suggest_candidates(field), query);
        self.suggest_open = Some(field);
        self.suggest_rev = self.suggest_rev.wrapping_add(1);
    }

    /// Menu screen rows: indices into `products` matching the list search
    /// (all of them while it is empty).
    fn menu_list_indices(&self) -> Vec<usize> {
        let q = self.menu_list_filter.trim();
        self.products
            .iter()
            .enumerate()
            .filter(|(_, p)| q.is_empty() || menu_images::match_rank(&p.name, q).is_some())
            .filter(|(_, p)| !self.menu_unpriced_only || p.price.0 == 0)
            .map(|(i, _)| i)
            .collect()
    }

    /// Dishes on the menu still at no price.
    fn unpriced_count(&self) -> usize {
        self.products.iter().filter(|p| p.price.0 == 0).count()
    }

    /// Categories that have at least one dish on the menu, in the shop's
    /// order — the order desk's chips (after All + Frequent).
    fn menu_categories(&self) -> Vec<domain::Category> {
        let mut cats: Vec<_> = self
            .categories
            .iter()
            .filter(|c| self.products.iter().any(|p| p.category_id == c.id))
            .cloned()
            .collect();
        cats.sort_by_key(|c| c.sort_order);
        cats.truncate(CATEGORY_CHIPS.len());
        cats
    }

    fn refresh_frequent_cache(&mut self) {
        let Some(store) = self.store.as_ref() else {
            self.frequent_ids.clear();
            self.frequent_days = 1;
            self.frequent_limit = 24;
            return;
        };
        let (days, limit) = store.frequent_config().unwrap_or((1, 24));
        self.frequent_days = days;
        self.frequent_limit = limit;
        self.frequent_ids = store
            .frequent_product_ids(days, limit)
            .unwrap_or_default();
    }

    /// The menu window's dish tiles: the picked category's dishes (all of
    /// them under "All items"), narrowed by what is typed — name, alias /
    /// short code, or dish group ("paneer" finds Paneer Tikka (Half) and
    /// (Full)). Best matches first: a short code typed exactly ("101") leads.
    /// Each size is an item of its own, so each is its own tile.
    /// Frequent ranks by recent completed sales (see Frequent config).
    fn visible_products(&self) -> Vec<Product> {
        let q = self.menu_filter.trim();
        let now = chrono::Local::now();
        if self.menu_category.as_deref() == Some(MENU_FREQUENT) {
            let by_id: std::collections::HashMap<&str, &Product> =
                self.products.iter().map(|p| (p.id.as_str(), p)).collect();
            let mut out: Vec<Product> = self
                .frequent_ids
                .iter()
                .filter_map(|id| by_id.get(id.as_str()).copied())
                .filter(|p| {
                    let windows = p.schedule();
                    let vis: Vec<_> = windows.iter().filter(|w| w.price_paise.is_none()).collect();
                    vis.is_empty() || vis.iter().any(|w| w.contains(now))
                })
                .cloned()
                .collect();
            if !q.is_empty() {
                out.retain(|p| {
                    let aliases = p.aliases();
                    aliases.iter().any(|a| a.trim().eq_ignore_ascii_case(q))
                        || menu_images::match_rank(&p.name, q).is_some()
                        || (!p.group_name.is_empty()
                            && menu_images::match_rank(&p.group_name, q).is_some())
                        || aliases.iter().any(|a| menu_images::match_rank(a, q).is_some())
                });
            }
            return out;
        }
        let in_category = |p: &&Product| match &self.menu_category {
            Some(cat) => &p.category_id == cat,
            None => true,
        };
        // Schedule visibility (breakfast windows…); sold-out still dims.
        let on_menu_now = |p: &&Product| p.available_now(now) || !p.available;
        let base = self
            .products
            .iter()
            .filter(in_category)
            .filter(|p| {
                // Hide when outside visibility windows; keep sold-out for the
                // dimmed "Unavailable" face when they would otherwise show.
                let windows = p.schedule();
                let vis: Vec<_> = windows.iter().filter(|w| w.price_paise.is_none()).collect();
                if vis.is_empty() {
                    true
                } else {
                    vis.iter().any(|w| w.contains(now))
                }
            });
        let _ = on_menu_now;
        if q.is_empty() {
            return base.cloned().collect();
        }
        let mut ranked: Vec<(u8, &Product)> = base
            .filter_map(|p| {
                let aliases = p.aliases();
                if aliases.iter().any(|a| a.trim().eq_ignore_ascii_case(q)) {
                    return Some((0, p));
                }
                let best = std::iter::once(menu_images::match_rank(&p.name, q))
                    .chain(std::iter::once(
                        (!p.group_name.is_empty())
                            .then(|| menu_images::match_rank(&p.group_name, q))
                            .flatten(),
                    ))
                    .chain(aliases.iter().map(|a| menu_images::match_rank(a, q)))
                    .flatten()
                    .min()?;
                Some((best + 1, p))
            })
            .collect();
        ranked.sort_by_key(|(rank, _)| *rank);
        ranked.into_iter().map(|(_, p)| p.clone()).collect()
    }

    /// Item lookup for the ops screens: exact name first, then unique-ish
    /// "contains" so sellers can type "chai" for "Masala Chai".
    /// What the restaurant buys and counts: raw materials first, then the
    /// menu items it resells (or already counts). Never its own dishes.
    fn stock_items(&self) -> Vec<Product> {
        let mut items = self.raw_materials.clone();
        items.extend(self.products.iter().filter(|p| p.is_bought()).cloned());
        items
    }

    /// The stock item this name means: an exact name / alias first, else a
    /// single partial match. `None` = nothing like it yet (a new raw
    /// material); an error when the name is ambiguous.
    fn find_stock_item(&self, name: &str) -> Result<Option<Product>, String> {
        let needle = name.trim();
        if needle.is_empty() {
            return Err("Type or pick the item first".into());
        }
        let items = self.stock_items();
        let exact: Vec<&Product> = items.iter().filter(|p| p.matches_label(needle)).collect();
        match exact.len() {
            1 => return Ok(Some(exact[0].clone())),
            n if n > 1 => {
                return Err(format!(
                    "More than one item is called \"{needle}\" — pick it from the list."
                ));
            }
            _ => {}
        }
        Ok(None)
    }

    /// The stock item for `name`, created as a raw material (with `unit`)
    /// when the restaurant has never bought it before.
    fn stock_item_or_new(&mut self, name: &str, unit: &str) -> Result<Product, String> {
        if let Some(p) = self.find_stock_item(name)? {
            return Ok(p);
        }
        let p = Product::new_raw_material(name, unit);
        self.store.as_ref().ok_or("No store")?.upsert_product(&p)?;
        self.reload_all()?;
        Ok(p)
    }

    fn refresh_inventory_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let mut lines = store.ops_panel_text().unwrap_or_default();
        let movements = store.list_stock_movements(12)?;
        if !movements.is_empty() {
            lines.push('\n');
            lines.push_str(&format!("{}:", self.t("recent_changes")));
            lines.push('\n');
            for m in &movements {
                let when = m.created_at.get(11..16).unwrap_or("");
                let sign = if m.delta_qty >= 0 { "+" } else { "" };
                let note = if m.note.trim().is_empty() {
                    String::new()
                } else {
                    format!(" · {}", m.note.trim())
                };
                lines.push_str(&format!(
                    "{when}  {}  {sign}{}{note}\n",
                    m.product_name, m.delta_qty
                ));
            }
        }
        self.ops_panel_text = lines;
        Ok(())
    }

    /// Today's cash-up, with the last recorded close underneath it so the owner
    /// can tell a fresh day from one already signed off.
    fn refresh_day_close_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let mut text = store.day_close_summary()?;
        match store.last_day_close_at()? {
            Some(at) => {
                let shown = at[..at.len().min(19)].replace('T', " ");
                text.push_str(&format!("\nLast recorded close: {shown}\n"));
            }
            None => text.push_str("\nNo day close recorded yet.\n"),
        }
        self.day_close_text = text;
        Ok(())
    }

    /// Books home: money in/out, AR/AP, cash vs bank (not sales-only day close).
    fn refresh_books_overview(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        self.day_close_text = store.books_overview_text()?;
        Ok(())
    }

    /// Load the bank-books body for [`Self::books_report_kind`] into `day_close_text`.
    fn load_books_report(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or_else(|| "No store".to_string())?;
        self.day_close_text = match self.books_report_kind {
            BooksReportKind::Today => store.books_report_text("today")?,
            BooksReportKind::Month => store.books_report_text("month")?,
            BooksReportKind::Aging => store.outstanding_pack_text()?,
            BooksReportKind::BalanceSheet => store.balance_sheet_text()?,
        };
        Ok(())
    }

    /// Reports screen body (bank books / day reports).
    fn refresh_reports_facet(&mut self) -> Result<(), String> {
        self.load_books_report()
    }

    fn refresh_branches_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let org = store.org_name()?;
        let branches = store.list_branches()?;
        let active_id = store.active_branch_id()?;
        // Each branch is its own restaurant (vision §5): this list is this
        // restaurant plus other branches' imported day snapshots.
        let mut lines = vec![
            format!("Business: {org}"),
            String::new(),
            "This restaurant and other branches' imported days:".to_string(),
        ];
        for b in &branches {
            let marker = if b.id == active_id { "●" } else { "○" };
            let state = if b.is_this_device || b.id == active_id {
                "selling from this device".to_string()
            } else if b.last_sync_at.trim().is_empty() {
                "no snapshot imported yet".to_string()
            } else {
                format!(
                    "snapshot {}",
                    &b.last_sync_at[..b.last_sync_at.len().min(16)]
                )
            };
            lines.push(format!("  {marker} {} · {state}", b.label()));
        }
        self.branches_text = lines.join("\n");
        Ok(())
    }

    /// Rebuild the category picker for what is typed in the Category field:
    /// the typed name as a new category (unless one already has that name),
    /// then the owner's categories, then shelf folders not already among
    /// them. A blank field lists everything.
    fn refresh_category_picks(&mut self, query: &str) {
        let query = query.trim();
        let mut picks = Vec::new();
        let exact = self
            .categories
            .iter()
            .any(|c| c.name.trim().eq_ignore_ascii_case(query));
        if !query.is_empty() && !exact {
            picks.push(CategoryPick::New(query.to_string()));
        }
        let mut mine: Vec<(u8, &str, &str)> = self
            .categories
            .iter()
            .filter_map(|c| {
                let rank = if query.is_empty() { Some(0) } else { menu_images::match_rank(&c.name, query) };
                rank.map(|r| (r, c.name.as_str(), c.id.as_str()))
            })
            .collect();
        mine.sort();
        let mine: Vec<CategoryPick> = mine
            .into_iter()
            .map(|(_, _, id)| CategoryPick::Mine(id.to_string()))
            .collect();
        picks.extend(mine);
        for folder in menu_images::shelf_folders(menu_images::shelf()) {
            let taken = self
                .categories
                .iter()
                .any(|c| c.name.trim().eq_ignore_ascii_case(&folder.label));
            if !taken && (query.is_empty() || menu_images::match_rank(&folder.label, query).is_some()) {
                picks.push(CategoryPick::Shelf(folder));
            }
        }
        self.category_picks = picks;
        self.category_picks_rev = self.category_picks_rev.wrapping_add(1);
    }

    /// Rebuild the item picker's rows for what is typed in the name field:
    /// your own name first (when something is typed), then matching items
    /// already on your menu, then matching shelf dishes. A blank name
    /// browses the whole shelf.
    fn refresh_dish_picks(&mut self, query: &str) {
        let query = query.trim();
        let mut picks = Vec::new();
        if !query.is_empty() {
            picks.push(DishPick::Own(query.to_string()));
            let mut menu: Vec<(u8, &str, &str)> = self
                .products
                .iter()
                .filter_map(|p| {
                    // Names and the owner's aliases are search handles only —
                    // a hit offers the item; identity stays its UUID.
                    let aliases = p.aliases();
                    std::iter::once(p.name.as_str())
                        .chain(std::iter::once(p.group_name.as_str()).filter(|g| !g.is_empty()))
                        .chain(aliases.iter().map(String::as_str))
                        .filter_map(|text| menu_images::match_rank(text, query))
                        .min()
                        .map(|r| (r, p.name.as_str(), p.id.as_str()))
                })
                .collect();
            menu.sort();
            let menu: Vec<DishPick> = menu
                .into_iter()
                .map(|(_, _, id)| DishPick::Menu(id.to_string()))
                .collect();
            picks.extend(menu);
        }
        picks.extend(
            menu_images::search_shelf(menu_images::shelf(), query)
                .into_iter()
                .map(DishPick::Shelf),
        );
        self.dish_picks = picks;
        self.dish_picks_rev = self.dish_picks_rev.wrapping_add(1);
    }

    fn display_venue_title(&self) -> String {
        let t = self.venue_title.trim();
        if t.is_empty() {
            "Your shop".into()
        } else {
            t.to_string()
        }
    }

    /// Owner-chosen in-app mark only. Never falls back to the Kumkaj package icon.
    fn venue_icon_path(&self) -> Option<PathBuf> {
        match self.venue_icon_key.as_deref() {
            Some(key) if key.starts_with("custom:") => {
                let file = &key["custom:".len()..];
                self.store
                    .as_ref()
                    .and_then(|s| s.brand_images_dir().ok())
                    .map(|d| d.join(file))
                    .filter(|p| p.is_file())
            }
            _ => None,
        }
    }

    fn actor(&self) -> Result<StaffMember, String> {
        self.session
            .clone()
            .ok_or_else(|| "Unlock with PIN first".to_string())
    }

    /// Load this signed-in user's amount-visibility prefs (global across screens).
    fn reload_amount_visibility(&mut self) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let Some(user) = self.session.as_ref().map(lan_key) else {
            return;
        };
        self.amounts_screen_hidden = store.amounts_screen_hidden(&user).unwrap_or(false);
        self.amount_keys_hidden =
            parse_amount_keys_hidden(store.amount_keys_hidden_csv(&user).ok().flatten().as_deref());
        self.amount_keys_peek =
            parse_amount_keys_hidden(store.amount_keys_peek_csv(&user).ok().flatten().as_deref());
    }

    fn money(&self, m: Money) -> String {
        m.format_localized(&self.locale.currency, &self.fiscal)
    }

    /// Required Size / Crust (etc.) must be picked before Send KOT or pay.
    fn require_cart_modifiers(&self) -> Result<(), String> {
        let products = &self.products;
        let missing = self.cart.missing_required_modifiers(|id| {
            products
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.modifier_groups())
                .unwrap_or_default()
        });
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing.join(" · "))
        }
    }

    /// Decimal mark the seller's country writes money with. Edit fields are
    /// filled with it; parsing accepts either mark regardless.
    fn decimal_sep(&self) -> &str {
        self.fiscal.decimal_sep()
    }

    /// Tax the open cart will have added at the till. Always 0 for a shop that
    /// prices tax-inclusive, where the strip above already shows the real bill.
    fn cart_tax_estimate_paise(&self) -> i64 {
        if self.fiscal.tax_pricing.is_inclusive() {
            return 0;
        }
        self.cart
            .lines
            .iter()
            .map(|line| {
                let bps = self
                    .products
                    .iter()
                    .find(|p| p.id == line.product_id)
                    .map(|p| p.tax_bps)
                    .unwrap_or(0);
                fiscal::tax_on_amount(line.applied_total().paise(), bps, TaxPricing::Exclusive)
            })
            .sum()
    }

    /// Today's ops report, worded for the shop's country pack.
    #[allow(dead_code)] // Kept for a short till snapshot; Reports Today uses full books_report_text.
    fn ops_report_text(&self, report: &domain::OpsReport) -> String {
        let tax_caption = if self.fiscal.tax_pricing.is_inclusive() {
            format!("{} inside today's sales", self.fiscal.tax_label())
        } else {
            format!("{} collected today", self.fiscal.tax_label())
        };
        let mut lines = vec![
            format!("Sales today: {}", report.day.sale_count),
            format!("Cash {}", self.money(Money(report.day.cash_paise))),
            format!(
                "{} {}",
                self.fiscal.digital_pay_label(),
                self.money(Money(report.day.upi_paise))
            ),
            format!("Card {}", self.money(Money(report.day.card_paise))),
            format!(
                "{} sales {}",
                self.fiscal.credit_label(),
                self.money(Money(report.day.credit_paise))
            ),
            format!(
                "{} {}",
                self.lane_declared_label(),
                self.money(Money(report.day.declared_paise))
            ),
            format!(
                "{} {}",
                self.lane_owner_label(),
                self.money(Money(report.day.owner_lane_paise))
            ),
            format!(
                "Still owed {}",
                self.money(Money(report.outstanding_credit_paise))
            ),
            format!(
                "Expenses {} ({})",
                self.money(Money(report.expenses_paise)),
                report.expense_count
            ),
            format!(
                "Bank net today {}",
                self.money(Money(report.bank_net_paise))
            ),
            format!("{tax_caption} {}", self.money(Money(report.day.tax_paise))),
            "Top items:".into(),
        ];
        for (name, qty, rev) in &report.top_items {
            lines.push(format!("  {name} ×{qty} · {}", self.money(Money(*rev))));
        }
        lines.push("Use Month / Aging / Bank below for full books".into());
        lines.join("\n")
    }

    /// One line under the totals strip: silent unless exclusive tax is about
    /// to change what the customer actually pays.
    fn cart_tax_estimate_line(&self) -> String {
        let tax = self.cart_tax_estimate_paise();
        let service = self.fiscal.service_on(self.cart.line_sum().paise());
        if tax <= 0 && service <= 0 {
            return " ".to_string();
        }
        let mut parts = Vec::new();
        if tax > 0 {
            parts.push(format!("+ {} {}", self.fiscal.tax_label(), self.money(Money(tax))));
        }
        if service > 0 {
            parts.push(format!("+ Service {}", self.money(Money(service))));
        }
        format!(
            "{} · pay {}",
            parts.join(" "),
            self.money(Money(self.cart.rounded_total().paise() + tax + service))
        )
    }

    fn t(&self, key: &str) -> String {
        let tmpl = entity_template::resolve(&self.entity_template_id);
        locale::t_overlay(&self.locale.language, key, tmpl.label_overrides)
    }

    /// Reset the order desk for the next order: empty cart, no table or
    /// customer attached. Called after a bill is paid / a KOT is sent.
    fn clear_order_desk(&mut self) {
        self.cart.clear();
        self.pay_choice = PayChoice::Cash;
        self.open_cart_line = None;
        self.selected_table_id = None;
        self.selected_customer_id = None;
        self.split_mode = false;
        self.split_pick.clear();
    }

    fn lane_declared_label(&self) -> String {
        entity_template::resolve(&self.entity_template_id)
            .lane_declared_label
            .into()
    }

    fn lane_owner_label(&self) -> String {
        entity_template::resolve(&self.entity_template_id)
            .lane_owner_label
            .into()
    }

    /// One-line “today” strip. Word prefixes and counts stay readable; only
    /// money slots go through [`display_money`].
    fn today_label(&self, amounts_visible: bool) -> String {
        match self.store.as_ref().and_then(|s| s.today_totals().ok()) {
            Some(t) => {
                let outstanding = self
                    .store
                    .as_ref()
                    .and_then(|s| s.outstanding_credit_total().ok())
                    .unwrap_or(0);
                let money = |m: Money| display_money(&self.money(m), amounts_visible);
                // One line on every screen: what came in today, and only the
                // ways it came in that are not zero. The bill-type split
                // lives on Day close and Reports.
                let mut parts = vec![format!(
                    "{}: {} · {}",
                    self.t("today_prefix"),
                    t.sale_count,
                    money(t.revenue())
                )];
                for (label, amount) in [
                    (self.t("cash"), t.cash()),
                    (self.t("upi"), t.upi()),
                    (self.t("card"), t.card()),
                    (self.t("pay_credit"), t.credit()),
                ] {
                    if amount.paise() != 0 {
                        parts.push(format!("{label} {}", money(amount)));
                    }
                }
                if outstanding != 0 {
                    parts.push(format!("{} {}", self.t("owed"), money(Money(outstanding))));
                }
                parts.join("   ·   ")
            }
            None => format!("{}: —", self.t("today_prefix")),
        }
    }
}

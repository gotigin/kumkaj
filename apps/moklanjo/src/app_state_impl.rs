impl AppState {
    fn boot() -> Self {
        let mut state = Self {
            store: None,
            products: Vec::new(),
            cart: Cart::default(),
            amend_cart: Cart::default(),
            amend_pay: PayMethod::Cash,
            sale_revisions: Vec::new(),
            selected_customer_id: None,
            customers: Vec::new(),
            recent_sales: Vec::new(),
            staff: Vec::new(),
            session: None,
            lock_as_owner: true,
            lock_staff_id: None,
            hire_staff_role: StaffRole::Cashier,
            hire_features: StaffFeatures::for_role(StaffRole::Cashier),
            draft_staff_role: StaffRole::Cashier,
            draft_features: StaffFeatures::for_role(StaffRole::Cashier),
            team_staff_id: None,
            team_select_pending: false,
            device_role: DeviceRole::Unset,
            must_confirm_device_role: true,
            language_follow_platform: true,
            picking_server: false,
            local_servers: Vec::new(),
            wipe_sel_trading: false,
            wipe_sel_schema: false,
            wipe_sel_ui: false,
            found_servers: Vec::new(),
            lan_devices: Vec::new(),
            data_base: None,
            printer_draft: print_share::DevicePrinters::default(),
            printer_route_key: "receipt".into(),
            owner_setup_complete: false,
            onboarding_complete: false,
            client_paired: false,
            extra_tills_allowed: false,
            onboard_step: 1,
            entity_template_id: "moklanjo_agency".into(),
            money_lane: MoneyLane::Declared,
            buy_on_account: false,
            buy_party_kind: "Supplier".into(),
            day_close_text: String::new(),
            ops_panel_text: String::new(),
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
            pending_ops_pod: None,
            locale: LocaleSettings::default(),
            fiscal: FiscalSettings::default(),
            country_browse: String::new(),
            venue_title: String::new(),
            venue_icon_key: None,
            nav_order: NAV_TABS.to_vec(),
            nav_hidden: Vec::new(),
            amounts_screen_hidden: false,
            snack_secs: SnackKind::DEFAULT_SECS,
            settings_panel: None,
            form_tab_anchor: None,
            form_tab_skip_submit: false,
            amount_keys_hidden: Vec::new(),
            amount_keys_peek: Vec::new(),
            books_sections_open: Vec::new(),
            device_setup_sections_open: vec!["role".into()],
            draft_venue_icon_path: None,
            pending_backup_copy: None,
            pending_diag_export: None,
            draft_image_key: None,
            draft_party_image_key: None,
            party_form_country: String::new(),
            booking_scope_international: false,
            booking_origin_name: String::new(),
            booking_origin_phone: String::new(),
            booking_origin_address: String::new(),
            booking_destination: String::new(),
            booking_consignee: String::new(),
            booking_consignee_phone: String::new(),
            booking_consignee_address: String::new(),
            booking_actual_kg: String::new(),
            booking_length_cm: String::new(),
            booking_breadth_cm: String::new(),
            booking_height_cm: String::new(),
            booking_cod: String::new(),
            booking_carrier_cost: String::new(),
            booking_declared: String::new(),
            booking_contents: String::new(),
            booking_eway: String::new(),
            booking_insurance: false,
            booking_notify_phone: String::new(),
            booking_fuel: String::new(),
            booking_docket: String::new(),
            booking_zone: String::new(),
            booking_pieces: String::new(),
            booking_carrier_pick: String::new(),
            last_carrier_bill_report: String::new(),
            last_carrier_compare: Vec::new(),
            measure_dim_unit: "cm".into(),
            measure_weight_unit: "kg".into(),
            measure_divisor: rust_decimal::Decimal::from(5000),
            measure_slab_kg: rust_decimal::Decimal::new(5, 1),
            measure_dim_factor: rust_decimal::Decimal::ONE,
            measure_weight_factor: rust_decimal::Decimal::ONE,
            expense_tender: PayMethod::Cash,
            expense_tender2: PayMethod::Bank,
            last_awb: String::new(),
            editing_product_id: None,
            pay_in_flight: false,
            lan_sign_in_busy: false,
            pair_note: String::new(),
            failed_unlocks: 0,
            unlock_blocked_until: None,
            pin_reset_required: false,
            nav_back: Vec::new(),
            nav_forward: Vec::new(),
            rail_width: RAIL_WIDTH_DEFAULT,
            rail_collapsed: false,
            reports_nav: ReportsNav::Bank,
            ops_lane: manifest::OpsLane::ToDispatch,
            ops_rows: Vec::new(),
            ops_counts: [0; 5],
            ops_picked: Vec::new(),
            ops_runs: Vec::new(),
            ops_history: Vec::new(),
            ops_reason: None,
            books_report_kind: BooksReportKind::Today,
            suggest_open: None,
            suggest_rows: Vec::new(),
            suggest_rev: 0,
        };
        if let Ok(dir) = store::data_dir() {
            state.data_base = Some(dir.clone());
            print_share::use_device_dir(&dir);
            diag::ensure_ready(&dir, store::APP_ID, env!("CARGO_PKG_VERSION"));
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
        }
        self.products = store.list_products(false)?;
        self.recent_sales = store.recent_sales(30)?;
        self.staff = store.list_staff()?;
        // Surface list failures — silent empty was "I added 12 parties and see none".
        self.customers = store.list_customers()?;
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
            self.snack_secs = store
                .snack_secs(&user)
                .unwrap_or(SnackKind::DEFAULT_SECS);
        } else {
            self.snack_secs = SnackKind::DEFAULT_SECS;
        }
        self.books_sections_open = parse_books_sections(store.books_sections_csv()?.as_deref());
        let measure = store.booking_measure_settings();
        self.measure_dim_factor = store
            .factor_to_cm(&measure.dim_unit)
            .unwrap_or(rust_decimal::Decimal::ONE);
        self.measure_weight_factor = store
            .factor_to_kg(&measure.weight_unit)
            .unwrap_or(rust_decimal::Decimal::ONE);
        self.measure_dim_unit = measure.dim_unit;
        self.measure_weight_unit = measure.weight_unit;
        self.measure_divisor = measure.volumetric_divisor;
        self.measure_slab_kg = measure.slab_kg;
        self.owner_setup_complete = store.owner_setup_complete().unwrap_or(true);
        self.onboarding_complete = store.onboarding_complete().unwrap_or(true);
        self.client_paired = store.client_paired().unwrap_or(false);
        self.pin_reset_required = store.pin_reset_required().unwrap_or(false);
        self.entity_template_id = store
            .entity_template_id()
            .unwrap_or_else(|_| "custom".into());
        // Person rights live in the staff roster (synced over LAN). Refresh the
        // signed-in session copy so Team changes on the Server show on chrome
        // without forcing unlock again.
        self.refresh_session_from_staff_roster();
        Ok(())
    }

    /// Drop Team edit target + hire/edit drafts so the next sign-in does not
    /// inherit another person's selection or unsaved Save payload.
    fn clear_team_edit_selection(&mut self) {
        self.team_staff_id = None;
        self.team_select_pending = false;
        self.hire_staff_role = StaffRole::Cashier;
        // X12: hire defaults at actor ceiling when signed in.
        self.hire_features = self
            .session
            .as_ref()
            .map(|a| Store::features_for_hire(StaffRole::Cashier, a))
            .unwrap_or_else(|| StaffFeatures::for_role(StaffRole::Cashier));
        self.draft_staff_role = StaffRole::Cashier;
        self.draft_features = StaffFeatures::for_role(StaffRole::Cashier);
    }

    /// Overlay live `staff` features/rights onto `session` (same person id).
    /// Missing or inactive → clear session and lock (do not keep stale rights).
    fn refresh_session_from_staff_roster(&mut self) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        let id = session.id.clone();
        let fresh = self
            .staff
            .iter()
            .find(|s| s.id == id && s.active)
            .cloned();
        let Some(fresh) = fresh else {
            self.session = None;
            self.clear_team_edit_selection();
            self.pin_buffer.clear();
            self.screen = Screen::Lock;
            return;
        };
        if let Some(session) = self.session.as_mut() {
            session.name = fresh.name;
            session.role = fresh.role;
            session.role_title = fresh.role_title;
            session.features = fresh.features;
            session.active = fresh.active;
            session.login_id = fresh.login_id;
        }
        // If Team stripped Credit/Items/… while they were on that screen,
        // leave it for the first peer they may still open (or Lock).
        if !session_may_stay_on(self.screen, self) {
            self.screen = first_allowed_work_screen(self);
        }
    }

    /// Booking desk catalogue: sell cards only; party overrides for the
    /// selected consignor (or defaults when no party).
    fn booking_catalog_products(&self) -> Vec<Product> {
        let selected = self.selected_customer_id.as_deref().unwrap_or("");
        self.products
            .iter()
            .filter(|p| {
                if !p.is_sell_card() {
                    return false;
                }
                let pid = p.party_id.trim();
                pid.is_empty() || (!selected.is_empty() && pid == selected)
            })
            .cloned()
            .collect()
    }

    /// Prefer a party-specific sell card for the same zone when consignor is set.
    fn resolve_booking_product(&self, product: &Product) -> Product {
        if !product.is_sell_card() {
            return product.clone();
        }
        let Some(cid) = self.selected_customer_id.as_deref() else {
            return product.clone();
        };
        if !product.party_id.trim().is_empty() || product.zone_id.trim().is_empty() {
            return product.clone();
        }
        self.products
            .iter()
            .find(|p| {
                p.is_sell_card()
                    && p.zone_id == product.zone_id
                    && p.party_id == cid
                    && p.available
                    && !p.archived
            })
            .cloned()
            .unwrap_or_else(|| product.clone())
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

    /// Lock Owner/Staff chip: remembered preference, else first-time default
    /// (Client → Staff, Server → Owner). Forces Owner when setup/staff/PIN
    /// reset require it.
    fn apply_lock_role_selection(&mut self) {
        let has_staff = self.staff.iter().any(|s| !s.is_owner());
        if !self.owner_setup_complete || !has_staff || self.pin_reset_required {
            self.lock_as_owner = true;
            return;
        }
        let prefer = self
            .store
            .as_ref()
            .and_then(|s| s.lock_role_prefer_owner().ok())
            .flatten();
        self.lock_as_owner = match prefer {
            Some(as_owner) => as_owner,
            // First time on this books file: Client → Staff, Server → Owner.
            None => !matches!(self.device_role, DeviceRole::Client),
        };
        if !self.lock_as_owner && self.lock_staff_id.is_none() {
            self.lock_staff_id = self
                .staff
                .iter()
                .find(|s| !s.is_owner())
                .map(|s| s.id.clone());
        }
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

    /// Device exception: only the shop host may do this (backup folder, extra
    /// tills, pairing code, forget a device, restore). A Client's copy is not
    /// the shop.
    fn require_server_host(&self, what: &str) -> Result<(), String> {
        if self.device_role == DeviceRole::Client {
            return Err(format!("{what} on the shop device — this is an extra till"));
        }
        Ok(())
    }

    /// Person-level book write. A Client asks the shop over `POST /books`;
    /// the shop applies it and this device reloads after a local apply.
    fn books_op(&mut self, op: BooksOp) -> Result<String, String> {
        if self.device_role == DeviceRole::Client {
            let label = op.label();
            let done = op.done_message();
            self.lan_job("/books", &op, label, done.clone(), phone::Follow::Refresh, false)?;
            return Ok(done);
        }
        let actor = self.actor()?;
        let store = self.store.as_ref().ok_or_else(|| "No store".to_string())?;
        let msg = store.apply_books_op(op, &actor)?;
        let _ = self.reload_all();
        Ok(msg)
    }

    /// Hand a write to the device's LAN worker (Client only). `queued`
    /// writes (a booking's payment, a settlement) wait through a Wi‑Fi
    /// drop; the rest need the Server now. `done` is the status line once
    /// the Server agreed; its refusal shows in its own words instead.
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
            return Err("Pair this device with the office first".into());
        }
        let id = new_id();
        let job = phone::Job {
            id: id.clone(),
            path: path.into(),
            body: serde_json::to_string(body).map_err(|e| e.to_string())?,
            label,
            done,
            follow,
            table_id: String::new(),
            staff_key: lan_key(&actor),
            queued,
        };
        phone::send(store, job)?;
        Ok(id)
    }

    /// A paired device asks for the Server's books; the device worker
    /// fetches them and the screen reloads when they land. Never waits.
    fn pull_from_server(&mut self) {
        if self.device_role == DeviceRole::Client {
            phone::want_sync();
        }
    }

    /// Item lookup for the ops screens: exact name first, then unique-ish
    /// "contains" so sellers can type "chai" for "Masala Chai".
    fn find_product_by_name(&self, name: &str) -> Result<Product, String> {
        let needle = name.trim();
        if needle.is_empty() {
            return Err("Type an item name first".into());
        }
        let lower = needle.to_lowercase();
        let exact: Vec<&Product> = self
            .products
            .iter()
            .filter(|p| p.matches_label(needle))
            .collect();
        if exact.len() > 1 {
            return Err(format!(
                "More than one item is called \"{needle}\". Names are labels; pick the item on the list."
            ));
        }
        if let Some(product) = exact.first() {
            return Ok((*product).clone());
        }
        let partial: Vec<&Product> = self
            .products
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&lower)
                    || p.aliases()
                        .iter()
                        .any(|alias| alias.to_lowercase().contains(&lower))
            })
            .collect();
        if partial.len() > 1 {
            return Err(format!(
                "\"{needle}\" matches more than one item. Type more of the name."
            ));
        }
        partial
            .first()
            .cloned()
            .cloned()
            .ok_or_else(|| format!("No item matches \"{needle}\""))
    }

    fn refresh_inventory_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let low = store.list_low_stock(5)?;
        let movements = store.list_stock_movements(12)?;
        let tracked: Vec<&Product> = self.products.iter().filter(|p| p.tracks_stock()).collect();
        let mut lines = Vec::new();
        if tracked.is_empty() {
            lines.push("No item is stock tracked yet — type a name and tap Start tracking.".into());
        } else {
            lines.push("On hand:".to_string());
            for p in tracked.iter().take(20) {
                lines.push(format!("  {} · {}", p.name, p.stock_qty));
            }
        }
        lines.push(String::new());
        if low.is_empty() {
            lines.push("Low stock: nothing at or below 5.".to_string());
        } else {
            lines.push("Low stock (5 or less):".to_string());
            for p in &low {
                lines.push(format!("  {} · {} left", p.name, p.stock_qty));
            }
        }
        lines.push(String::new());
        if movements.is_empty() {
            lines.push("No stock movements yet.".to_string());
        } else {
            lines.push("Recent movements:".to_string());
            for m in &movements {
                let when = &m.created_at[..m.created_at.len().min(16)];
                let sign = if m.delta_qty >= 0 { "+" } else { "" };
                let note = if m.note.trim().is_empty() {
                    String::new()
                } else {
                    format!(" ({})", m.note.trim())
                };
                lines.push(format!(
                    "  {when} {} {sign}{} {}{note}",
                    m.product_name, m.delta_qty, m.reason
                ));
            }
        }
        let purchases = store.list_purchases(5)?;
        lines.push(String::new());
        if purchases.is_empty() {
            lines.push("No purchases yet.".to_string());
        } else {
            lines.push("Recent purchases:".to_string());
            for purchase in &purchases {
                let line_count = store.purchase_lines(&purchase.id)?.len();
                let supplier = if purchase.supplier.trim().is_empty() {
                    "Supplier"
                } else {
                    purchase.supplier.trim()
                };
                lines.push(format!(
                    "  {supplier} · {} · {line_count} lines",
                    self.money(purchase.total())
                ));
            }
        }
        self.ops_panel_text = lines.join("\n");
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
    /// On the Server: what other devices just changed — today's bookings,
    /// party balances, which devices are on — read again from the books.
    /// Today's figures follow in `sync_chrome`, which reads them itself.
    fn refresh_live_lists(&mut self) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        // Prefer keep-last on failure — never wipe a full party book to empty.
        if let Ok(recent) = store.recent_sales(30) {
            self.recent_sales = recent;
        }
        if let Ok(customers) = store.list_customers() {
            self.customers = customers;
        }
        if let Ok(devices) = store.list_lan_devices() {
            self.lan_devices = devices;
        }
    }

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

    /// Reports screen body for the active facet (bank books / ops / COD / riders).
    fn refresh_reports_facet(&mut self) -> Result<(), String> {
        match self.reports_nav {
            ReportsNav::Bank => self.load_books_report(),
            ReportsNav::Ops => self.refresh_ops_board(),
            other => {
                let store = self.store.as_ref().ok_or("No store")?;
                let facet = match other {
                    ReportsNav::Bank => unreachable!(),
                    ReportsNav::Ops => courier_ops_view::OpsFacet::Ops,
                    ReportsNav::Cod => courier_ops_view::OpsFacet::Cod,
                    ReportsNav::Riders => courier_ops_view::OpsFacet::Riders,
                };
                self.day_close_text = courier_ops_view::facet_text(store, facet)?;
                Ok(())
            }
        }
    }

    /// Re-read the Ops board from the books: lane counts, the lane's
    /// parcels, recent runs, and the picked parcel's history. Picks that
    /// left the lane (moved on another till, or by this action) drop off.
    fn refresh_ops_board(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        self.ops_counts = store.ops_lane_counts()?;
        self.ops_rows = store.ops_board_rows(self.ops_lane, OPS_BOARD_LIMIT)?;
        let in_lane: std::collections::HashSet<&str> =
            self.ops_rows.iter().map(|r| r.sale_id.as_str()).collect();
        self.ops_picked.retain(|id| in_lane.contains(id.as_str()));
        self.ops_runs = store.list_manifests(8)?;
        self.ops_history = match self.ops_picked.as_slice() {
            [one] => {
                let events = store.list_tracking_events(one)?;
                let awb = self
                    .ops_rows
                    .iter()
                    .find(|r| r.sale_id == *one)
                    .map(|r| r.awb_no.clone())
                    .unwrap_or_default();
                let mut rows = vec![ReportRow::Head(format!("{} · {awb}", self.t("parcel_history")))];
                for e in events.iter().rev() {
                    let label = self.t(manifest::TrackingStatus::parse(&e.status).label_key());
                    let when = self.fiscal.format_datetime(&e.created_at);
                    rows.push(ReportRow::Kv(format!("{when} · {label}"), e.actor_name.clone()));
                    if !e.note.trim().is_empty() {
                        rows.push(ReportRow::Sub(e.note.trim().to_string()));
                    }
                }
                rows
            }
            _ => Vec::new(),
        };
        // Plain text of the lane for Copy report (and a non-empty marker so
        // the screen does not re-read the books on every repaint).
        let lane = self.t(self.ops_lane.label_key());
        let mut text = format!("{lane} · {}\n", self.ops_rows.len());
        for r in &self.ops_rows {
            text.push_str(&format!("{} · {} · {}\n", r.awb_no, r.destination, r.consignee_name));
        }
        self.day_close_text = text;
        Ok(())
    }

    fn refresh_branches_text(&mut self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("No store")?;
        let org = store.org_name()?;
        let branches = store.list_branches()?;
        let active_id = store.active_branch_id()?;
        // Each branch is its own business (vision §5): this list is these
        // books plus other branches' imported day snapshots.
        let mut lines = vec![
            format!("Business: {org}"),
            String::new(),
            "This branch and other branches' imported days:".to_string(),
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
            self.snack_secs = SnackKind::DEFAULT_SECS;
            return;
        };
        self.amounts_screen_hidden = store.amounts_screen_hidden(&user).unwrap_or(false);
        self.amount_keys_hidden =
            parse_amount_keys_hidden(store.amount_keys_hidden_csv(&user).ok().flatten().as_deref());
        self.amount_keys_peek =
            parse_amount_keys_hidden(store.amount_keys_peek_csv(&user).ok().flatten().as_deref());
        self.snack_secs = store
            .snack_secs(&user)
            .unwrap_or(SnackKind::DEFAULT_SECS);
    }

    fn money(&self, m: Money) -> String {
        m.format_localized(&self.locale.currency, &self.fiscal)
    }

    /// Decimal mark the seller's country writes money with. Edit fields are
    /// filled with it; parsing accepts either mark regardless.
    fn decimal_sep(&self) -> &str {
        self.fiscal.decimal_sep()
    }

    /// Tax the open cart will have added at the till. Always 0 for a shop that
    /// prices with Tax inside price, where the strip above already shows the real bill.
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
                fiscal::tax_on_amount(line.applied_total().paise(), bps, TaxPricing::AddedAtTill)
            })
            .sum()
    }

    /// Today's ops report, worded for the shop's country pack.
    #[allow(dead_code)] // Kept for a short till snapshot; Reports Today uses full books_report_text.
    fn ops_report_text(&self, report: &domain::OpsReport) -> String {
        let tax_caption = if self.fiscal.tax_pricing.is_inclusive() {
            format!("{} inclusive in today's sales", self.fiscal.tax_label())
        } else {
            format!("{} exclusive collected today", self.fiscal.tax_label())
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
            // Report text is English data output (vision §23): the
            // template's own words, not the translated chip labels.
            format!(
                "{} {}",
                entity_template::resolve(&self.entity_template_id).lane_declared_label,
                self.money(Money(report.day.declared_paise))
            ),
            format!(
                "{} {}",
                entity_template::resolve(&self.entity_template_id).lane_owner_label,
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
            parts.push(format!(
                "+ {} {}",
                self.fiscal.tax_label(),
                self.money(Money(tax))
            ));
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

    /// Rights shown on the Team screen for the selected staff: the draft if
    /// the owner touched it, else that person's role preset.
    fn team_draft_rights(&self) -> domain::Rights {
        if let Some(r) = &self.draft_features.rights {
            return r.clone();
        }
        let role = self
            .team_staff_id
            .as_ref()
            .and_then(|id| self.staff.iter().find(|s| &s.id == id))
            .map(|s| s.role)
            .unwrap_or(StaffRole::Cashier);
        domain::Rights::for_role(role)
    }

    fn t(&self, key: &str) -> String {
        let tmpl = entity_template::resolve(&self.entity_template_id);
        locale::t_overlay(&self.locale.language, key, tmpl.label_overrides)
    }

    /// Actual / volumetric / chargeable kg from the booking desk fields.
    /// The user types in their chosen units; everything returned here is
    /// canonical kg (dims converted to cm first) — what the books record.
    fn booking_weights(
        &self,
    ) -> (
        rust_decimal::Decimal,
        rust_decimal::Decimal,
        rust_decimal::Decimal,
    ) {
        let zero = precise::ZERO;
        let typed = |raw: &str| booking::parse_positive(raw).unwrap_or(zero);
        let actual = booking::to_base(typed(&self.booking_actual_kg), self.measure_weight_factor);
        let l = booking::to_base(typed(&self.booking_length_cm), self.measure_dim_factor);
        let b = booking::to_base(typed(&self.booking_breadth_cm), self.measure_dim_factor);
        let h = booking::to_base(typed(&self.booking_height_cm), self.measure_dim_factor);
        let divisor = if self.booking_scope_international {
            booking::volumetric_divisor_international()
        } else {
            self.measure_divisor
        };
        let vol = booking::volumetric_kg(l, b, h, divisor);
        let chargeable = booking::chargeable_kg(actual, l, b, h, divisor, self.measure_slab_kg);
        (actual, vol, chargeable)
    }

    fn apply_booking_fields(&self, draft: &mut domain::SaleDraft) {
        let dial = &self.fiscal.country_code;
        draft.origin_name = self.booking_origin_name.trim().to_string();
        draft.origin_phone = with_calling_code(&self.booking_origin_phone, dial);
        draft.origin_address = self.booking_origin_address.trim().to_string();
        draft.destination = self.booking_destination.trim().to_string();
        draft.consignee_name = self.booking_consignee.trim().to_string();
        draft.consignee_phone = with_calling_code(&self.booking_consignee_phone, dial);
        draft.consignee_address = self.booking_consignee_address.trim().to_string();
        let (actual, vol, chargeable) = self.booking_weights();
        draft.actual_weight_kg = actual;
        draft.volumetric_weight_kg = vol;
        draft.chargeable_weight_kg = chargeable;
        // Resolve typed zone name (or id) to the zone master id — never store a free-text name.
        let zone_typed = self.booking_zone.trim();
        draft.zone_id = if zone_typed.is_empty() {
            String::new()
        } else {
            self.store
                .as_ref()
                .and_then(|s| s.list_zones().ok())
                .and_then(|zones| {
                    zones
                        .into_iter()
                        .find(|z| z.id == zone_typed || z.name.eq_ignore_ascii_case(zone_typed))
                })
                .map(|z| z.id)
                .unwrap_or_default()
        };
        let currency = self.locale.currency.clone();
        draft.cod_paise = booking::parse_extra_amount(&self.booking_cod, &currency).unwrap_or(0);
        draft.carrier_cost_paise =
            booking::parse_extra_amount(&self.booking_carrier_cost, &currency).unwrap_or(0);
        draft.declared_value_paise =
            booking::parse_extra_amount(&self.booking_declared, &currency).unwrap_or(0);
        draft.contents = self.booking_contents.trim().to_string();
        draft.eway_bill_no = self.booking_eway.trim().to_string();
        draft.insurance = self.booking_insurance;
        draft.notify_phone = with_calling_code(&self.booking_notify_phone, dial);
        let mut extras = Vec::new();
        if let Some(p) =
            booking::parse_extra_amount(&self.booking_fuel, &currency).filter(|p| *p > 0)
        {
            extras.push(booking::booking_extra(rates::ExtraChargeKind::Fuel, p));
        }
        if let Some(p) =
            booking::parse_extra_amount(&self.booking_docket, &currency).filter(|p| *p > 0)
        {
            extras.push(booking::booking_extra(rates::ExtraChargeKind::Docket, p));
        }
        draft.extras_json = serde_json::to_string(&extras).unwrap_or_else(|_| "[]".into());
        let pieces = self.booking_pieces.trim();
        draft.piece_count = if pieces.is_empty() {
            1
        } else {
            pieces.parse::<i64>().unwrap_or(1).max(1)
        };
        let carrier_typed = self.booking_carrier_pick.trim();
        draft.carrier_party_id = if carrier_typed.is_empty() {
            String::new()
        } else {
            self.store
                .as_ref()
                .and_then(|s| s.list_customers().ok())
                .and_then(|parties| {
                    parties.into_iter().find(|p| {
                        p.id == carrier_typed || p.name.eq_ignore_ascii_case(carrier_typed)
                    })
                })
                .map(|p| p.id)
                .unwrap_or_default()
        };
    }

    fn clear_booking_desk(&mut self) {
        self.cart.clear();
        self.booking_origin_name.clear();
        self.booking_origin_phone.clear();
        self.booking_origin_address.clear();
        self.booking_destination.clear();
        self.booking_consignee.clear();
        self.booking_consignee_phone.clear();
        self.booking_consignee_address.clear();
        self.booking_actual_kg.clear();
        self.booking_length_cm.clear();
        self.booking_breadth_cm.clear();
        self.booking_height_cm.clear();
        self.booking_cod.clear();
        self.booking_carrier_cost.clear();
        self.booking_declared.clear();
        self.booking_contents.clear();
        self.booking_eway.clear();
        self.booking_insurance = false;
        self.booking_notify_phone.clear();
        self.booking_fuel.clear();
        self.booking_docket.clear();
        self.booking_zone.clear();
        self.booking_pieces.clear();
        self.booking_carrier_pick.clear();
        self.last_awb.clear();
        self.selected_customer_id = None;
    }

    /// What a "type or pick" field can offer, before narrowing.
    fn suggest_candidates(&self, field: SuggestField) -> Vec<suggest::SuggestRow> {
        use suggest::SuggestRow;
        let parties = |filter: fn(&Customer) -> bool| {
            self.customers
                .iter()
                .filter(|c| c.active && filter(c))
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
        let all_parties = || parties(|_| true);
        let carriers = || parties(|c| c.is_carrier());
        let consignors = || parties(|c| c.is_consignor());
        let suppliers = || {
            parties(|c| domain::party_has_role(&c.kind_label, domain::PARTY_ROLE_SUPPLIER))
        };
        let units = || {
            let all = self
                .store
                .as_ref()
                .and_then(|s| s.list_units().ok())
                .unwrap_or_default();
            let mut v: Vec<SuggestRow> = all
                .iter()
                .map(|u| SuggestRow::new(u.code.clone(), u.name.clone()))
                .collect();
            let mut used: Vec<SuggestRow> = self
                .products
                .iter()
                .map(|p| p.unit_label.trim().to_string())
                .filter(|u| !u.is_empty())
                .map(|u| SuggestRow::new(u, ""))
                .collect();
            used.append(&mut v);
            used
        };
        let zones = || {
            self.store
                .as_ref()
                .and_then(|s| s.list_zones().ok())
                .unwrap_or_default()
                .into_iter()
                .map(|z| SuggestRow::new(z.name, ""))
                .collect::<Vec<_>>()
        };
        match field {
            SuggestField::BookingDestination => self
                .store
                .as_ref()
                .and_then(|s| s.suggest_destinations("", 60).ok())
                .unwrap_or_default()
                .into_iter()
                .map(|d| SuggestRow::new(d, ""))
                .collect(),
            SuggestField::BookingZone | SuggestField::MenuZone => zones(),
            SuggestField::BookingCarrier
            | SuggestField::MenuCarrier
            | SuggestField::AwbPoolCarrier
            | SuggestField::OpsCarrier => {
                let mut rows = carriers();
                for r in &self.last_carrier_compare {
                    let sub = locale::format_major_input(r.cost_paise, &self.locale.currency);
                    rows.insert(0, SuggestRow::new(r.carrier_name.clone(), sub));
                }
                rows
            }
            SuggestField::BookingOrigin
            | SuggestField::CustomerName
            | SuggestField::CreditParty
            | SuggestField::MenuParty
            | SuggestField::OpsParty => {
                let mut rows = consignors();
                rows.extend(all_parties());
                rows
            }
            SuggestField::MenuUnit
            | SuggestField::ConvFrom
            | SuggestField::ConvTo
            | SuggestField::MeasureDim
            | SuggestField::MeasureWeight => units(),
            SuggestField::MenuTax => self
                .store
                .as_ref()
                .and_then(|s| s.list_tax_rates().ok())
                .unwrap_or_default()
                .into_iter()
                .filter(|r| r.active)
                .map(|r| {
                    let pct = format!("{:.2}", r.bps as f64 / 100.0);
                    let pct = format!("{}%", pct.trim_end_matches('0').trim_end_matches('.'));
                    SuggestRow::new(r.name, pct)
                })
                .collect(),
            SuggestField::MenuCategory => self
                .store
                .as_ref()
                .and_then(|s| s.list_categories().ok())
                .unwrap_or_default()
                .into_iter()
                .map(|c| SuggestRow::new(c.name, ""))
                .collect(),
            SuggestField::BuyItem | SuggestField::InvItem => self
                .products
                .iter()
                .map(|p| {
                    let unit = p.unit_label.trim();
                    SuggestRow::new(p.name.clone(), unit.to_string())
                })
                .collect(),
            SuggestField::BuySupplier | SuggestField::InvSupplier => {
                let mut rows = suppliers();
                rows.extend(carriers());
                rows.extend(all_parties());
                rows
            }
            SuggestField::TeamPick => self
                .staff
                .iter()
                .filter(|m| m.active)
                .map(|m| {
                    // Value must be unique — bare display names can collide and
                    // rank() would drop duplicates, then pick the wrong person.
                    let login = m.login_id.trim();
                    let sub = if login.is_empty() {
                        m.name.clone()
                    } else {
                        format!("{} · {login}", m.name)
                    };
                    SuggestRow::new(m.lock_label(), sub)
                })
                .collect(),
            SuggestField::Country => fiscal::COUNTRY_PROFILES
                .iter()
                .map(|c| SuggestRow::new(c.code, c.name_en))
                .collect(),
            SuggestField::PrinterTarget => {
                let here = self.t("printers_heading");
                print_share::found_printers()
                    .into_iter()
                    .map(|name| SuggestRow::new(name, here.clone()))
                    .collect()
            }
            SuggestField::RouteAddPrinter => self
                .printer_draft
                .printers
                .iter()
                .map(|p| SuggestRow::new(p.label.clone(), p.target.clone()))
                .collect(),
            SuggestField::OpsRider | SuggestField::SettleRider => self
                .store
                .as_ref()
                .and_then(|s| s.list_riders().ok())
                .unwrap_or_default()
                .into_iter()
                .map(|r| SuggestRow::new(r.name, r.phone))
                .collect(),
            SuggestField::PartyCountry => fiscal::COUNTRY_PROFILES
                .iter()
                .filter(|c| c.code != fiscal::GENERIC_CODE)
                .map(|c| {
                    let dial = fiscal::calling_code(c.code);
                    let sub = if dial.is_empty() {
                        c.code.to_string()
                    } else {
                        format!("{} · {dial}", c.code)
                    };
                    SuggestRow::new(c.name_en, sub)
                })
                .collect(),
            SuggestField::PartyState => {
                let typed = self.party_form_country.trim();
                let code = if typed.is_empty() {
                    self.fiscal.country_code.clone()
                } else {
                    country_code_from_label(typed)
                };
                let mut rows: Vec<SuggestRow> = fiscal::admin_areas(&code)
                    .iter()
                    .map(|s| SuggestRow::new(*s, ""))
                    .collect();
                // Also offer areas already typed on parties.
                for c in &self.customers {
                    for part in c.address.split(|ch| ch == ',' || ch == '\n') {
                        let p = part.trim();
                        if p.len() >= 3 && p.len() <= 48 {
                            rows.push(SuggestRow::new(p, ""));
                        }
                    }
                }
                rows
            }
            SuggestField::PartyPincode => {
                let mut rows: Vec<SuggestRow> = Vec::new();
                let mut seen = std::collections::HashSet::new();
                for c in &self.customers {
                    for token in c.address.split(|ch: char| !(ch.is_ascii_alphanumeric())) {
                        let digits: String =
                            token.chars().filter(|ch| ch.is_ascii_digit()).collect();
                        if digits.len() >= 4
                            && digits.len() <= 10
                            && digits.len() >= token.len().saturating_sub(1)
                            && seen.insert(digits.clone())
                        {
                            rows.push(SuggestRow::new(digits, ""));
                        }
                    }
                }
                rows
            }
        }
    }

    /// Open (or re-narrow) a field's list.
    fn open_suggest(&mut self, field: SuggestField, query: &str) {
        let q = if matches!(field, SuggestField::BookingDestination) && !query.trim().is_empty() {
            // Destination uses store-side ranking (pins + past sales).
            let ranked = self
                .store
                .as_ref()
                .and_then(|s| s.suggest_destinations(query, 60).ok())
                .unwrap_or_default()
                .into_iter()
                .map(|d| suggest::SuggestRow::new(d, ""))
                .collect::<Vec<_>>();
            self.suggest_rows = suggest::rank(ranked, "");
            self.suggest_open = Some(field);
            self.suggest_rev = self.suggest_rev.wrapping_add(1);
            return;
        } else {
            query
        };
        self.suggest_rows = suggest::rank(self.suggest_candidates(field), q);
        self.suggest_open = Some(field);
        self.suggest_rev = self.suggest_rev.wrapping_add(1);
    }

    /// Money-lane chip words in the UI language (`lane_*_label` keys). The
    /// template's English copy stays for books / report text only.
    fn lane_declared_label(&self) -> String {
        self.t("lane_declared_label")
    }

    fn lane_owner_label(&self) -> String {
        self.t("lane_owner_label")
    }

    /// One-line/multi-line “today” strip. Word prefixes and counts stay
    /// readable; only money slots go through [`display_money`].
    fn today_label(&self, amounts_visible: bool) -> String {
        match self.store.as_ref().and_then(|s| s.today_totals().ok()) {
            Some(t) => {
                let outstanding = self
                    .store
                    .as_ref()
                    .and_then(|s| s.outstanding_credit_total().ok())
                    .unwrap_or(0);
                let money = |m: Money| display_money(&self.money(m), amounts_visible);
                let mut lines = vec![format!(
                    "{}: {} · {}",
                    self.t("today_prefix"),
                    t.sale_count,
                    money(t.revenue())
                )];
                if t.declared_paise != 0 || t.owner_lane_paise != 0 {
                    lines.push(format!(
                        "{} {} · {} {}",
                        self.lane_declared_label(),
                        money(Money(t.declared_paise)),
                        self.lane_owner_label(),
                        money(Money(t.owner_lane_paise))
                    ));
                }
                lines.push(format!(
                    "{} {} · {} {} · {} {}",
                    self.t("cash"),
                    money(t.cash()),
                    self.t("upi"),
                    money(t.upi()),
                    self.t("card"),
                    money(t.card())
                ));
                if t.credit_paise != 0 || outstanding != 0 {
                    lines.push(format!(
                        "{} {} {} · {} {}",
                        self.t("credit"),
                        self.t("today_mid"),
                        money(t.credit()),
                        self.t("owed"),
                        money(Money(outstanding))
                    ));
                }
                if t.rounding_paise != 0 {
                    lines.push(format!(
                        "{} {}",
                        self.t("round_word"),
                        money(t.rounding())
                    ));
                }
                // Markdown treats a single \\n as a soft break (often one
                // clipped line). Paragraph breaks so WrapLabel wraps each row
                // inside the bar instead of truncating at the right edge.
                lines.join("\n\n")
            }
            None => format!("{}: —", self.t("today_prefix")),
        }
    }
}

impl App {
fn activate_existing_server(&mut self, cx: &mut Cx, server_id: &str) -> Result<(), String> {
        self.resolve_data_base(cx);
        let base = {
            let st = app_state().lock().unwrap();
            st.data_base
                .clone()
                .ok_or_else(|| "No data directory".to_string())?
        };
        net::stop_server();
        let store = local_servers::open_server(&base, server_id)?;
        {
            let mut st = app_state().lock().unwrap();
            let previous = st.device_role;
            st.store = Some(store);
            st.picking_server = false;
            let _ = st.refresh_local_servers();
            // Language is app-global: keep the choice showing on DeviceSetup
            // across the reload (which reads this shop's own meta).
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            self.finish_as_server(&mut st, previous != DeviceRole::Server);
        }
        self.sync_chrome(cx);
        Ok(())
    }

    fn finish_as_server(&mut self, st: &mut AppState, switching_in: bool) {
        st.device_role = DeviceRole::Server;
        st.must_confirm_device_role = false;
        st.picking_server = false;
        st.lock_as_owner = true;
        st.pin_buffer.clear();
        st.nav_back.clear();
        st.nav_forward.clear();
        if let Some(store) = st.store.as_ref() {
            let _ = store.set_device_role(DeviceRole::Server.as_str());
            if switching_in {
                let _ = store.set_extra_tills_allowed(false);
                st.extra_tills_allowed = false;
                net::stop_server();
            } else if st.extra_tills_allowed {
                let path = store.db_path().to_path_buf();
                if net::ensure_server(path).is_ok() {
                    firewall_job(|| Ok(net::firewall_allows_phones()));
                }
            }
        }
        st.screen = Screen::Lock;
        st.status = if !st.owner_setup_complete {
            locale::t(&st.locale.language, "status_server_set_pin")
        } else if st.extra_tills_allowed {
            locale::t(&st.locale.language, "status_server_tills")
        } else {
            locale::t(&st.locale.language, "status_server_unlock")
        };
    }

    fn begin_client_mode(&mut self, cx: &mut Cx) {
        self.resolve_data_base(cx);
        let result = (|| -> Result<(), String> {
            let base = {
                let st = app_state().lock().unwrap();
                st.data_base
                    .clone()
                    .ok_or_else(|| "No data directory".to_string())?
            };
            net::stop_server();
            let store = local_servers::open_client_store(&base)?;
            let mut st = app_state().lock().unwrap();
            st.store = Some(store);
            st.picking_server = false;
            // Language is app-global: keep the choice showing on DeviceSetup
            // across the reload (which reads the client cache's own meta).
            let chosen = (st.language_follow_platform, st.locale.language.clone());
            let _ = st.reload_all();
            st.language_follow_platform = chosen.0;
            st.locale.language = chosen.1;
            Self::persist_setup_language(&mut st);
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.prepare_as_client_terminal()?;
            // Saved like Server Mode is: a reload (pairing, sync) reads the
            // role back from the store and must not find it unset.
            store.set_device_role(DeviceRole::Client.as_str())?;
            st.device_role = DeviceRole::Client;
            st.must_confirm_device_role = false;
            st.owner_setup_complete = true;
            st.onboarding_complete = true;
            st.extra_tills_allowed = false;
            st.lock_as_owner = true;
            st.pin_buffer.clear();
            st.nav_back.clear();
            st.nav_forward.clear();
            let _ = st.reload_all();
            st.client_paired = store_client_paired(st.store.as_ref());
            if st.staff.iter().any(|s| !s.is_owner()) {
                st.lock_as_owner = false;
                if st.lock_staff_id.is_none() {
                    st.lock_staff_id = st
                        .staff
                        .iter()
                        .find(|s| !s.is_owner())
                        .map(|s| s.id.clone());
                }
            }
            if st.client_paired {
                st.screen = Screen::Lock;
                st.status = locale::t(&st.locale.language, "status_client_unlock");
            } else {
                st.screen = Screen::ClientPair;
                st.status = "Client Mode — pair to the Server before selling".into();
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn set_extra_tills(&mut self, cx: &mut Cx, allowed: bool) {
        self.ensure_store(cx);
        let err = {
            let mut st = app_state().lock().unwrap();
            if let Err(e) = st.require_server_role("Change extra tills") {
                Some(e)
            } else if st.store.is_none() {
                Some("Storage not ready".into())
            } else if let Err(e) = st.store.as_ref().unwrap().set_extra_tills_allowed(allowed) {
                Some(e)
            } else {
                st.extra_tills_allowed = allowed;
                if allowed {
                    let store = st.store.as_ref().unwrap();
                    let code = store.ensure_server_pairing_code().unwrap_or_default();
                    let path = store.db_path().to_path_buf();
                    if let Err(e) = net::ensure_server(path) {
                        st.status = format!("Extra tills on, LAN listen failed: {e}");
                    } else {
                        st.status = format!("Extra tills on · pairing code {code}");
                        firewall_job(|| Ok(net::firewall_allows_phones()));
                    }
                } else {
                    net::stop_server();
                    st.status = "Extra tills off — this device still holds the shop books".into();
                }
                None
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Copy digits from the lock PIN field into `pin_buffer` (max 12).
    fn sync_pin_buffer_from_field(&mut self, cx: &mut Cx) {
        let text = self.ui.text_input(cx, ids!(lock_pin_input)).text();
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).take(12).collect();
        let mut st = app_state().lock().unwrap();
        st.pin_buffer = digits;
    }

    /// Seconds left on the wrong-PIN block, or None when unlock may proceed.
    fn unlock_block_remaining(&self) -> Option<u64> {
        let mut st = app_state().lock().unwrap();
        let until = st.unlock_blocked_until?;
        match until.checked_duration_since(std::time::Instant::now()) {
            Some(left) => Some(left.as_secs() + 1),
            None => {
                st.unlock_blocked_until = None;
                None
            }
        }
    }

    /// What the device's LAN threads brought back: pairing, sign-in, the
    /// Server's answers to writes, fresh books.
    fn take_lan_answers(&mut self, cx: &mut Cx) {
        let once = lan_once().lock().unwrap().take();
        match once {
            Some(LanOnce::SignIn {
                staff_id,
                pin,
                reply,
            }) => self.finish_lan_sign_in(cx, staff_id, pin, reply),
            Some(LanOnce::Paired(reply)) => {
                let result = (|| -> Result<String, String> {
                    let (link, snap) = reply?;
                    let mut st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    store.set_client_device_pairing(&link.base, &link.device)?;
                    // Kept to find the Server again if its address changes.
                    store.set_paired_server_id(&link.server_id)?;
                    phone::forget_revision();
                    net::apply_snapshot(store, &snap)?;
                    phone::start(store.db_path().to_path_buf());
                    st.client_paired = true;
                    st.reload_all()?;
                    // The person signs in with their office PIN: this device's
                    // own sign-in is not one the Server knows.
                    st.session = None;
                    st.screen = Screen::Lock;
                    // The office's name as it saved it; unnamed, "the office".
                    let name = [snap.venue_title.trim(), snap.org_name.trim()]
                        .into_iter()
                        .find(|n| !n.is_empty())
                        .unwrap_or("the office")
                        .to_string();
                    Ok(format!("Paired with {name} — sign in with your office PIN"))
                })();
                match result {
                    Ok(msg) => {
                        app_state().lock().unwrap().pair_note.clear();
                        self.set_status(cx, msg)
                    }
                    Err(e) => {
                        app_state().lock().unwrap().pair_note = e.clone();
                        self.set_status(cx, e)
                    }
                }
                self.sync_chrome(cx);
            }
            None => {}
        }
        // The office forgot this device (Forget on the Server): back to
        // pairing, saying why. Waiting bookings stay on the device for after.
        let forgotten = {
            let mut st = app_state().lock().unwrap();
            if st.device_role == DeviceRole::Client
                && st.client_paired
                && phone::link_state().needs_pairing
            {
                st.client_paired = false;
                if let Some(store) = st.store.as_ref() {
                    let _ = store.clear_client_pairing();
                }
                st.session = None;
                st.pair_note = st.t("device_forgotten");
                st.screen = Screen::ClientPair;
                true
            } else {
                false
            }
        };
        let answers = phone::take_done();
        let synced = phone::take_synced();
        if answers.is_empty() && !synced {
            // The header's link line may have changed (offline, waiting).
            self.sync_chrome(cx);
            return;
        }
        let _ = forgotten;
        let mut snack = None;
        let mut said = None;
        {
            let mut st = app_state().lock().unwrap();
            for answer in answers {
                match answer.reply {
                    Err(e) => {
                        // The Server's own words, said about what it was.
                        said = Some(if e.is_offline() {
                            format!("{} — {e}", answer.label)
                        } else {
                            format!("{}: {e}", answer.label)
                        });
                        if e.code() == "pair_again" {
                            st.client_paired = false;
                        }
                    }
                    Ok(reply) => {
                        said = Some(answer.done.clone());
                        if answer.follow == phone::Follow::Sale
                            && let Ok(sale) = serde_json::from_value::<Sale>(reply)
                        {
                            st.last_awb = sale.awb_no.clone();
                            said = Some(if sale.awb_no.is_empty() {
                                format!("Booked · {}", answer.done)
                            } else {
                                format!("Booked · AWB {} · {}", sale.awb_no, answer.done)
                            });
                            snack = Some((sale.id, answer.done));
                        }
                    }
                }
            }
            if synced {
                let _ = st.reload_all();
            }
        }
        if let Some(msg) = said {
            self.set_status(cx, msg);
        }
        if let Some((sale_id, _)) = &snack {
            // A booking's own receipt beats the status on the receipt line.
            let mut st = app_state().lock().unwrap();
            if let Some(text) = st
                .store
                .as_ref()
                .and_then(|s| s.format_receipt_text(sale_id).ok())
            {
                st.last_receipt = text;
            }
        }
        if let Some((sale_id, text)) = snack {
            self.show_undo_snackbar(cx, sale_id, text);
        }
        self.sync_chrome(cx);
    }

    fn show_found_servers(&mut self, cx: &mut Cx, found: Vec<net::FoundServer>) {
        let (text, rows) = {
            let mut st = app_state().lock().unwrap();
            let text = if found.is_empty() { st.t("no_server_found") } else { String::new() };
            // One row per office found: its name, and the address.
            let rows = found
                .iter()
                .map(|f| {
                    let venue = if f.venue.trim().is_empty() {
                        "Moklanjo"
                    } else {
                        f.venue.trim()
                    };
                    ReportRow::Kv(venue.to_string(), f.host.clone())
                })
                .collect::<Vec<_>>();
            st.found_servers = found;
            (text, rows)
        };
        let first = app_state()
            .lock()
            .unwrap()
            .found_servers
            .first()
            .map(|f| f.host.clone());
        if let Some(host) = first {
            self.ui
                .text_input(cx, ids!(client_host_input))
                .set_text(cx, &host);
        }
        self.ui
            .widget(cx, ids!(client_found_label))
            .set_text(cx, &plain_display_text(&text));
        self.ui.widget(cx, ids!(client_found_label)).set_visible(cx, !text.is_empty());
        self.set_report_rows(cx, ids!(client_found_list), rows);
        self.sync_chrome(cx);
    }

    fn try_unlock(&mut self, cx: &mut Cx) {
        self.ensure_store(cx);
        {
            let st = app_state().lock().unwrap();
            if st.must_confirm_device_role || st.device_role == DeviceRole::Unset {
                drop(st);
                self.go(cx, Screen::DeviceSetup);
                return;
            }
        }
        let (pin, lock_as_owner, lock_staff_id, has_store, owner_setup_complete, pin_reset) = {
            let st = app_state().lock().unwrap();
            (
                st.pin_buffer.clone(),
                st.lock_as_owner,
                st.lock_staff_id.clone(),
                st.store.is_some(),
                st.owner_setup_complete,
                st.pin_reset_required,
            )
        };
        if !has_store {
            self.set_status(cx, "Storage not ready — restart the app");
            return;
        }
        // Too many wrong PINs: sit out the block before trying again.
        if let Some(left) = self.unlock_block_remaining() {
            self.set_status(
                cx,
                format!("Too many wrong PINs — wait {left}s, then try again"),
            );
            return;
        }
        if pin.len() < 4 {
            self.set_status(cx, "Enter at least 4 digits, then OK");
            return;
        }

        // First-run Owner PIN setup (Server shop UX).
        if !owner_setup_complete {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let member = store.complete_owner_setup(&pin)?;
                let _ = store.set_active_session_staff_id(Some(&member.id));
                st.owner_setup_complete = true;
                st.onboarding_complete = false;
                st.onboard_step = 1;
                st.session = Some(member);
                st.reload_amount_visibility();
                st.pin_buffer.clear();
                st.pin_visible = false;
                st.lock_as_owner = true;
                st.screen = Screen::Onboard;
                st.status = "Owner ready — set up your shop".into();
                let _ = st.reload_all();
                Ok(())
            })();
            match result {
                Ok(()) => log!("Kumkaj owner setup ok"),
                Err(ref e) => {
                    log!("Kumkaj owner setup failed: {e}");
                    let mut st = app_state().lock().unwrap();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    drop(st);
                    self.set_status(cx, e.clone());
                }
            }
            self.sync_chrome(cx);
            return;
        }

        // The upgrade that replaced the PIN hasher handed every account back as
        // 0000. Reuse the first-run setup flow to get a real Owner PIN before
        // the till opens.
        if pin_reset {
            let result = (|| -> Result<(), String> {
                let mut st = app_state().lock().unwrap();
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                let member = store.complete_pin_reset(&pin)?;
                let _ = store.set_active_session_staff_id(Some(&member.id));
                st.session = Some(member.clone());
                st.reload_amount_visibility();
                st.pin_buffer.clear();
                st.pin_visible = false;
                st.lock_as_owner = true;
                st.failed_unlocks = 0;
                st.screen = unlocked_home();
                st.status = format!("New Owner PIN set — welcome back, {}", member.lock_label());
                let _ = st.refresh_books_overview();
                let _ = st.reload_all();
                Ok(())
            })();
            match result {
                Ok(()) => log!("Kumkaj owner PIN reset ok"),
                Err(ref e) => {
                    log!("Kumkaj owner PIN reset failed: {e}");
                    let mut st = app_state().lock().unwrap();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    drop(st);
                    self.set_status(cx, e.clone());
                }
            }
            self.sync_chrome(cx);
            return;
        }

        // A paired device signs in on the Server (PINs are checked there,
        // never on the device), off the UI thread.
        let lan = {
            let st = app_state().lock().unwrap();
            let staff_id = if lock_as_owner {
                None
            } else {
                lock_staff_id.clone().or_else(|| {
                    st.staff
                        .iter()
                        .find(|s| !s.is_owner())
                        .map(|s| s.id.clone())
                })
            };
            let link = (st.device_role == DeviceRole::Client && st.client_paired)
                .then(|| {
                    st.store.as_ref().and_then(|s| {
                        Some(net::Link {
                            base: s.server_url().ok()??,
                            device: s.client_device_token().ok()??,
                            ..net::Link::default()
                        })
                    })
                })
                .flatten();
            link.map(|link| (link, staff_id))
        };
        if let Some((link, staff_id)) = lan {
            {
                let mut st = app_state().lock().unwrap();
                if st.lan_sign_in_busy {
                    return;
                }
                st.lan_sign_in_busy = true;
                st.status = "Checking your PIN with the shop…".into();
            }
            let pin = pin.clone();
            let _ = std::thread::Builder::new()
                .name("moklanjo-sign-in".into())
                .spawn(move || {
                    let reply = net::client_login(&link, staff_id.as_deref(), &pin);
                    *lan_once().lock().unwrap() = Some(LanOnce::SignIn {
                        staff_id,
                        pin,
                        reply,
                    });
                    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                });
            self.sync_chrome(cx);
            return;
        }

        // Auth runs without holding the app mutex (PIN verify must not freeze the UI).
        // Emergency unlock PIN is checked first (wipe + open as Owner, looks normal).
        let auth = {
            let st = app_state().lock().unwrap();
            match st.store.as_ref() {
                Some(store) => {
                    if let Ok(Some(owner)) = store.try_duress_unlock(&pin) {
                        Ok(Some(owner))
                    } else if lock_as_owner {
                        store.authenticate_owner_pin(&pin)
                    } else {
                        let staff_id = lock_staff_id.clone().or_else(|| {
                            st.staff
                                .iter()
                                .find(|s| !s.is_owner())
                                .map(|s| s.id.clone())
                        });
                        match staff_id {
                            Some(id) => store.authenticate_staff_pin(&id, &pin),
                            None => Ok(None),
                        }
                    }
                }
                None => Err("No store".into()),
            }
        };
        self.finish_unlock(cx, auth, lock_as_owner, None);
    }

    /// The Server's answer to a device's sign-in. Out of reach: whoever last
    /// signed in on this device may unlock with the session saved then.
    fn finish_lan_sign_in(
        &mut self,
        cx: &mut Cx,
        staff_id: Option<String>,
        pin: String,
        reply: Result<net::LoginOk, net::LanError>,
    ) {
        let lock_as_owner = staff_id.is_none();
        let (auth, note) = {
            let mut st = app_state().lock().unwrap();
            st.lan_sign_in_busy = false;
            let Some(store) = st.store.as_ref() else {
                return;
            };
            match reply {
                Ok(ok) => {
                    let saved = store.remember_lan_sign_in(&ok.staff, &pin, &ok.session);
                    phone::signed_in();
                    (saved.map(|()| Some(ok.staff)), None)
                }
                Err(e) if e.is_offline() => {
                    match store.offline_lan_sign_in(staff_id.as_deref(), &pin) {
                        Ok(Some((staff, _))) => (
                            Ok(Some(staff)),
                            Some(
                                "Offline — orders wait on this device until the Wi‑Fi is back"
                                    .to_string(),
                            ),
                        ),
                        Ok(None) => (
                            Err(format!(
                                "{e} — and this person has not signed in on this device before"
                            )),
                            None,
                        ),
                        Err(err) => (Err(err), None),
                    }
                }
                Err(e) if e.code() == "wrong_pin" => (Ok(None), None),
                Err(e) if e.code() == "pair_again" => {
                    // Forgotten by the shop: pair again, and say why.
                    let _ = store.clear_client_pairing();
                    st.client_paired = false;
                    st.pair_note = st.t("device_forgotten");
                    st.screen = Screen::ClientPair;
                    (Err(st.pair_note.clone()), None)
                }
                Err(e) => (Err(e.to_string()), None),
            }
        };
        self.finish_unlock(cx, auth, lock_as_owner, note);
    }

    fn finish_unlock(
        &mut self,
        cx: &mut Cx,
        auth: Result<Option<StaffMember>, String>,
        lock_as_owner: bool,
        note: Option<String>,
    ) {
        let result = {
            let mut st = app_state().lock().unwrap();
            if !st.lock_as_owner && st.lock_staff_id.is_none() {
                st.lock_staff_id = st
                    .staff
                    .iter()
                    .find(|s| !s.is_owner())
                    .map(|s| s.id.clone());
            }
            match auth {
                Ok(Some(member)) => {
                    if let Some(store) = st.store.as_ref() {
                        let _ = store.set_active_session_staff_id(Some(&member.id));
                    }
                    st.session = Some(member.clone());
                    st.reload_amount_visibility();
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    st.failed_unlocks = 0;
                    st.unlock_blocked_until = None;
                    if st.device_role == DeviceRole::Client && !st.client_paired {
                        st.screen = Screen::ClientPair;
                        st.status = "Pair to the shop device before selling".into();
                    } else if st.device_role.is_host() && !st.onboarding_complete {
                        st.screen = Screen::Onboard;
                        st.status = format!("Welcome, {} — finish shop setup", member.lock_label());
                    } else {
                        st.screen = unlocked_home();
                        st.status = match &note {
                            Some(note) => format!("Welcome, {} · {note}", member.lock_label()),
                            None => format!("Welcome, {}", member.lock_label()),
                        };
                        let _ = st.refresh_books_overview();
                    }
                    if st.device_role == DeviceRole::Client
                        && st.client_paired
                        && let Some(store) = st.store.as_ref()
                    {
                        phone::start(store.db_path().to_path_buf());
                    }
                    // The books live here: back them up every day.
                    if st.device_role.is_host()
                        && let Some(store) = st.store.as_ref()
                    {
                        start_auto_backups(store.db_path().to_path_buf());
                    }
                    if st.uses_lan()
                        && let Some(store) = st.store.as_ref()
                    {
                        let path = store.db_path().to_path_buf();
                        if net::ensure_server(path).is_ok() {
                            firewall_job(|| Ok(net::firewall_allows_phones()));
                        }
                    }
                    Ok(())
                }
                Ok(None) => {
                    st.pin_buffer.clear();
                    st.pin_visible = false;
                    st.failed_unlocks = st.failed_unlocks.saturating_add(1);
                    if st.failed_unlocks >= UNLOCK_ATTEMPT_LIMIT {
                        st.failed_unlocks = 0;
                        st.unlock_blocked_until = Some(std::time::Instant::now() + UNLOCK_BLOCK);
                        Err(format!(
                            "Too many wrong PINs — locked for {}s",
                            UNLOCK_BLOCK.as_secs()
                        ))
                    } else {
                        let left = UNLOCK_ATTEMPT_LIMIT - st.failed_unlocks;
                        Err(if lock_as_owner {
                            format!("Wrong Owner PIN — {left} tries left")
                        } else if st.lock_staff_id.is_none() {
                            "Add staff in Team first, or unlock as Owner".into()
                        } else {
                            format!("Wrong Staff PIN — {left} tries left")
                        })
                    }
                }
                Err(e) => Err(e),
            }
        };
        match result {
            Ok(()) => log!("Kumkaj unlock ok"),
            Err(ref e) => log!("Kumkaj unlock failed: {e}"),
        }
        match result {
            Ok(()) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Soft-keyboard Done / Unlock-with-login: same path as the login button.
    fn try_unlock_with_login(&mut self, cx: &mut Cx) {
        let login = self.ui.text_input(cx, ids!(lock_login_id)).text();
        let password = self.ui.text_input(cx, ids!(lock_login_password)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.pin_reset_required {
                return Err("Owner must set a new PIN first".into());
            }
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            let member = if let Some(owner) = store.try_duress_unlock(&password)? {
                owner
            } else {
                store
                    .authenticate_login(&login, &password)?
                    .ok_or_else(|| "Wrong login or password".to_string())?
            };
            let _ = store.set_active_session_staff_id(Some(&member.id));
            st.session = Some(member.clone());
            st.reload_amount_visibility();
            st.screen = if st.device_role.is_host() && !st.onboarding_complete {
                Screen::Onboard
            } else {
                unlocked_home()
            };
            st.status = format!("Welcome, {}", member.lock_label());
            let _ = st.reload_all();
            if st.screen == Screen::Books {
                let _ = st.refresh_books_overview();
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn sync_booking_desk_from_ui(&mut self, cx: &mut Cx) {
        let mut st = app_state().lock().unwrap();
        st.booking_origin_name = self
            .ui
            .text_input(cx, ids!(booking_origin_name_input))
            .text();
        st.booking_origin_phone = self
            .ui
            .text_input(cx, ids!(booking_origin_phone_input))
            .text();
        st.booking_origin_address = self
            .ui
            .text_input(cx, ids!(booking_origin_address_input))
            .text();
        st.booking_destination = self
            .ui
            .text_input(cx, ids!(booking_destination_input))
            .text();
        st.booking_zone = self.ui.text_input(cx, ids!(booking_zone_input)).text();
        st.booking_consignee = self.ui.text_input(cx, ids!(booking_consignee_input)).text();
        st.booking_consignee_phone = self
            .ui
            .text_input(cx, ids!(booking_consignee_phone_input))
            .text();
        st.booking_consignee_address = self
            .ui
            .text_input(cx, ids!(booking_consignee_address_input))
            .text();
        st.booking_actual_kg = self.ui.text_input(cx, ids!(booking_actual_kg_input)).text();
        st.booking_length_cm = self.ui.text_input(cx, ids!(booking_length_cm_input)).text();
        st.booking_breadth_cm = self
            .ui
            .text_input(cx, ids!(booking_breadth_cm_input))
            .text();
        st.booking_height_cm = self.ui.text_input(cx, ids!(booking_height_cm_input)).text();
        st.booking_cod = self.ui.text_input(cx, ids!(booking_cod_input)).text();
        st.booking_carrier_cost = self
            .ui
            .text_input(cx, ids!(booking_carrier_cost_input))
            .text();
        st.booking_pieces = self.ui.text_input(cx, ids!(booking_pieces_input)).text();
        st.booking_carrier_pick = self
            .ui
            .text_input(cx, ids!(booking_carrier_pick_input))
            .text();
        st.booking_declared = self
            .ui
            .text_input(cx, ids!(booking_declared_input))
            .text();
        st.booking_contents = self
            .ui
            .text_input(cx, ids!(booking_contents_input))
            .text();
        st.booking_eway = self.ui.text_input(cx, ids!(booking_eway_input)).text();
        st.booking_fuel = self.ui.text_input(cx, ids!(booking_fuel_input)).text();
        st.booking_docket = self.ui.text_input(cx, ids!(booking_docket_input)).text();
        st.booking_notify_phone = self.ui.text_input(cx, ids!(booking_notify_input)).text();
    }

    /// Persist + live-apply booking measure settings. The desk unit chips
    /// and the Settings → Booking measures form both land here, so the
    /// desk reflects a change immediately.
    fn apply_measure_settings(&mut self, cx: &mut Cx, settings: domain::BookingMeasureSettings) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.set_booking_measure_settings(&settings)?;
            let refreshed = {
                let measure = store.booking_measure_settings();
                (
                    store.factor_to_cm(&measure.dim_unit),
                    store.factor_to_kg(&measure.weight_unit),
                    measure,
                )
            };
            let (dim_factor, weight_factor, measure) = refreshed;
            st.measure_dim_factor = dim_factor.unwrap_or(rust_decimal::Decimal::ONE);
            st.measure_weight_factor = weight_factor.unwrap_or(rust_decimal::Decimal::ONE);
            st.measure_dim_unit = measure.dim_unit;
            st.measure_weight_unit = measure.weight_unit;
            st.measure_divisor = measure.volumetric_divisor;
            st.measure_slab_kg = measure.slab_kg;
            st.status = "Booking measures saved".into();
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Desk chip shortcut: change only the dimension and/or weight unit.
    fn apply_measure_units(&mut self, cx: &mut Cx, dim: Option<String>, weight: Option<String>) {
        let current = {
            let st = app_state().lock().unwrap();
            st.store.as_ref().map(|s| s.booking_measure_settings())
        };
        let Some(mut settings) = current else {
            return;
        };
        if let Some(dim) = dim {
            settings.dim_unit = dim;
        }
        if let Some(weight) = weight {
            settings.weight_unit = weight;
        }
        self.apply_measure_settings(cx, settings);
    }

    fn clear_booking_desk_ui(&mut self, cx: &mut Cx) {
        for id in [
            ids!(booking_origin_name_input),
            ids!(booking_origin_phone_input),
            ids!(booking_origin_address_input),
            ids!(booking_destination_input),
            ids!(booking_zone_input),
            ids!(booking_consignee_input),
            ids!(booking_consignee_phone_input),
            ids!(booking_consignee_address_input),
            ids!(booking_actual_kg_input),
            ids!(booking_length_cm_input),
            ids!(booking_breadth_cm_input),
            ids!(booking_height_cm_input),
            ids!(booking_cod_input),
            ids!(booking_carrier_cost_input),
            ids!(booking_declared_input),
            ids!(booking_contents_input),
            ids!(booking_eway_input),
            ids!(booking_fuel_input),
            ids!(booking_docket_input),
            ids!(booking_notify_input),
        ] {
            self.ui.text_input(cx, id).set_text(cx, "");
        }
    }

    /// Party = consignor / customer / sender. Fill desk FROM fields; leave
    /// blank cells alone when the party record has no phone/address yet.
    fn fill_booking_consignor_from_party(&mut self, cx: &mut Cx, party: &domain::Customer) {
        {
            let mut st = app_state().lock().unwrap();
            st.booking_origin_name = party.name.clone();
            if !party.phone.trim().is_empty() {
                st.booking_origin_phone = party.phone.clone();
            }
            if !party.address.trim().is_empty() {
                st.booking_origin_address = party.address.clone();
            }
        }
        self.ui
            .text_input(cx, ids!(booking_origin_name_input))
            .set_text(cx, &party.name);
        if !party.phone.trim().is_empty() {
            self.ui
                .text_input(cx, ids!(booking_origin_phone_input))
                .set_text(cx, &party.phone);
        }
        if !party.address.trim().is_empty() {
            self.ui
                .text_input(cx, ids!(booking_origin_address_input))
                .set_text(cx, &party.address);
        }
    }

    fn add_product_to_cart(&mut self, cx: &mut Cx, index: usize) {
        self.sync_booking_desk_from_ui(cx);
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            if st.session.as_ref().map(|s| s.can_access_sell()) != Some(true) {
                Some("Not allowed to book".into())
            } else if let Some(raw) = st.booking_catalog_products().get(index).cloned() {
                let product = st.resolve_booking_product(&raw);
                match st.cart.add_or_bump_in_stock(&product) {
                    Err(e) => Some(e),
                    Ok(()) => {
                        let (_, _, chargeable) = st.booking_weights();
                        if let Some(line) = st.cart.lines.last_mut() {
                            if let Err(e) =
                                booking::apply_weight_and_slab(line, &product, chargeable)
                            {
                                Some(e)
                            } else {
                                st.status = format!("{} on booking", product.name);
                                None
                            }
                        } else {
                            st.status = format!("{} on booking", product.name);
                            None
                        }
                    }
                }
            } else {
                Some("Missing rate".into())
            }
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn handle_modifier_chips(
        &mut self,
        cx: &mut Cx,
        actions: &Actions,
        item: &WidgetRef,
        index: usize,
        amend: bool,
    ) {
        let slot = if item.button(cx, ids!(mod_0)).clicked(actions) {
            Some(0)
        } else if item.button(cx, ids!(mod_1)).clicked(actions) {
            Some(1)
        } else if item.button(cx, ids!(mod_2)).clicked(actions) {
            Some(2)
        } else {
            None
        };
        let page_delta = if item.button(cx, ids!(mod_next)).clicked(actions) {
            Some(1)
        } else if item.button(cx, ids!(mod_prev)).clicked(actions) {
            Some(-1)
        } else {
            None
        };
        if slot.is_none() && page_delta.is_none() {
            return;
        }
        let result = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            let product_id = {
                let cart = if amend { &st.amend_cart } else { &st.cart };
                cart.lines
                    .get(index)
                    .map(|l| l.product_id.clone())
                    .ok_or_else(|| "Missing cart line".to_string())?
            };
            let mods = st
                .products
                .iter()
                .find(|p| p.id == product_id)
                .map(|p| p.modifiers())
                .unwrap_or_default();
            let cart = if amend {
                &mut st.amend_cart
            } else {
                &mut st.cart
            };
            if let Some(delta) = page_delta {
                cart.shift_modifier_page(index, delta, mods.len());
                return Ok(String::new());
            }
            let page = cart.lines.get(index).map(|l| l.modifier_page).unwrap_or(0);
            let pages = mods.len().div_ceil(domain::MODIFIER_PAGE).max(1);
            let start = (page % pages) * domain::MODIFIER_PAGE;
            let modifier = mods
                .get(start + slot.unwrap_or(0))
                .cloned()
                .ok_or_else(|| "No modifier there".to_string())?;
            let on = cart.toggle_modifier(index, &modifier)?;
            let price = st.money(modifier.price());
            Ok(if on {
                format!("{} added · {price}", modifier.name)
            } else {
                format!("{} removed", modifier.name)
            })
        })();
        match result {
            Ok(status) if !status.is_empty() => self.set_status(cx, status),
            Ok(_) => {}
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Cart `−`/`+`. Raising a tracked line stops at the on-hand count so the
    /// refusal lands on the button, not on the customer at the till.
    fn bump_cart_line(&mut self, cx: &mut Cx, index: usize, delta: i64) {
        let err: Option<String> = {
            let mut st = app_state().lock().unwrap();
            let blocked = if delta > 0 {
                match st.cart.lines.get(index).map(|l| l.product_id.clone()) {
                    Some(product_id) => st
                        .products
                        .iter()
                        .find(|p| p.id == product_id)
                        .filter(|p| p.tracks_stock())
                        .filter(|p| st.cart.qty_of(&product_id) + delta > p.stock_qty)
                        .map(|p| format!("Only {} of {} in stock", p.stock_qty, p.name)),
                    None => None,
                }
            } else {
                None
            };
            if blocked.is_none() {
                st.cart.bump_qty(index, delta);
            }
            blocked
        };
        if let Some(e) = err {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    /// Single-tender charge (full bill one method). Sell UI uses amount fields +
    /// Take payment; kept for amend/automation paths that still need one tender.
    #[allow(dead_code)]
    fn pay(&mut self, cx: &mut Cx, method: PayMethod) {
        // A remote charge holds no lock while it waits on the LAN, so a second
        // tap would otherwise start a second bill for the same cart.
        if !self.begin_pay(cx) {
            return;
        }
        // Apply typed total even if seller didn't press Enter.
        let typed_total = self.ui.text_input(cx, ids!(cart_total_input)).text();
        let typed_customer = self.ui.text_input(cx, ids!(customer_name_input)).text();
        self.sync_booking_desk_from_ui(cx);
        let result = (|| -> Result<Option<(String, String)>, String> {
            enum Pending {
                LocalDone {
                    sale: Sale,
                    snack: String,
                },
                Remote {
                    draft: domain::SaleDraft,
                    customer_id: Option<String>,
                    customer_name: Option<String>,
                },
            }
            let pending = {
                let mut st = app_state().lock().unwrap();
                let actor = st.actor()?;
                if !actor.can_access_sell() {
                    return Err("Not allowed to sell".into());
                }
                let currency = st.locale.currency.clone();
                if let Some(p) =
                    booking::parse_extra_amount(&st.booking_fuel, &currency).filter(|p| *p > 0)
                {
                    let extra = booking::booking_extra(rates::ExtraChargeKind::Fuel, p);
                    if !st
                        .cart
                        .lines
                        .iter()
                        .any(|l| l.product_id.starts_with("extra:fuel"))
                    {
                        st.cart.lines.push(booking::extra_as_line(&extra));
                    }
                }
                if let Some(p) =
                    booking::parse_extra_amount(&st.booking_docket, &currency).filter(|p| *p > 0)
                {
                    let extra = booking::booking_extra(rates::ExtraChargeKind::Docket, p);
                    if !st
                        .cart
                        .lines
                        .iter()
                        .any(|l| l.product_id.starts_with("extra:docket"))
                    {
                        st.cart.lines.push(booking::extra_as_line(&extra));
                    }
                }
                if st.cart.is_empty() {
                    return Err("Cart is empty".into());
                }
                if st.cart.charge_total.is_some()
                    && let Ok(minor) = locale::parse_major_input(&typed_total, &currency)
                    && minor > 0
                {
                    let _ = st.cart.set_charge_total(Money(minor));
                }
                let mut draft = st.cart.to_sale_draft()?;
                draft.money_lane = st.money_lane;
                st.apply_booking_fields(&mut draft);
                let is_client = st.device_role == DeviceRole::Client;
                let customer_id = if method.is_credit() {
                    if let Some(id) = st.selected_customer_id.clone() {
                        Some(id)
                    } else if typed_customer.trim().is_empty() {
                        return Err("Type a party name, then tap Credit (account booking)".into());
                    } else if is_client {
                        // Leave the id unset so the Server resolves the name in
                        // its own customer book. Inventing one here would bill
                        // credit against a customer the Server has never heard of.
                        None
                    } else {
                        let name = typed_customer.trim();
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        let customer = store.find_or_create_customer(name)?;
                        st.selected_customer_id = Some(customer.id.clone());
                        Some(customer.id)
                    }
                } else {
                    // Cash booking may still attach a consignor when selected.
                    st.selected_customer_id.clone()
                };
                if st.device_role == DeviceRole::Client {
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    if !store.client_paired()? {
                        return Err("Client is not paired to a Server — connect first".into());
                    }
                    let customer_name = if method.is_credit() {
                        Some(typed_customer.trim().to_string()).filter(|s| !s.is_empty())
                    } else {
                        None
                    };
                    Pending::Remote {
                        draft,
                        customer_id,
                        customer_name,
                    }
                } else {
                    let sale = {
                        let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
                        store.finalize_sale(&draft, method, customer_id.as_deref(), &actor)?
                    };
                    let amount = st.money(Money(sale.total_paise));
                    let tender = method.label_for(&st.fiscal);
                    let receipt = if sale.rounding_paise != 0 {
                        format!(
                            "Sale {} · Rounded {} (Applied {} · round {}) · {} · {}",
                            &sale.id[..8.min(sale.id.len())],
                            amount,
                            st.money(Money(sale.lines_total_paise)),
                            st.money(Money(sale.rounding_paise)),
                            tender,
                            &sale.created_at[..sale.created_at.len().min(19)]
                        )
                    } else {
                        format!(
                            "Sale {} · Rounded {} · {} · {}",
                            &sale.id[..8.min(sale.id.len())],
                            amount,
                            tender,
                            &sale.created_at[..sale.created_at.len().min(19)]
                        )
                    };
                    let full = st
                        .store
                        .as_ref()
                        .and_then(|s| s.format_receipt_text(&sale.id).ok())
                        .unwrap_or(receipt);
                    st.cart.clear();
                    if method.is_credit() {
                        st.selected_customer_id = None;
                    }
                    st.last_awb = sale.awb_no.clone();
                    if let Some(store) = st.store.as_ref()
                        && !sale.awb_no.is_empty()
                    {
                        let _ = store.write_awb_label(&sale.id);
                    }
                    st.last_receipt = full;
                    st.status = if sale.awb_no.is_empty() {
                        format!("Booked {}", method.label())
                    } else {
                        format!("Booked {} · AWB {}", method.label(), sale.awb_no)
                    };
                    let snack = if sale.awb_no.is_empty() {
                        format!("{} · {} booked", amount, method.label())
                    } else {
                        format!("{} · AWB {} · {}", amount, sale.awb_no, method.label())
                    };
                    st.reload_all()?;
                    Pending::LocalDone { sale, snack }
                }
            };
            match pending {
                Pending::LocalDone { sale, snack } => Ok(Some((sale.id, snack))),
                Pending::Remote {
                    draft,
                    customer_id,
                    customer_name,
                } => {
                    // Saved on this device first, then sent: a Wi‑Fi drop
                    // delays the booking (and its AWB), it never loses it.
                    let mut st = app_state().lock().unwrap();
                    let amount = st.money(Money(draft.total_paise));
                    let req = net::RemoteSaleRequest {
                        draft,
                        pay: method.as_str().into(),
                        customer_id,
                        customer_name,
                        payments: Vec::new(),
                    };
                    st.lan_job(
                        "/sale",
                        &req,
                        format!("Booking {amount}"),
                        format!("{amount} · {} recorded", method.label()),
                        phone::Follow::Sale,
                        true,
                    )?;
                    st.cart.clear();
                    if method.is_credit() {
                        st.selected_customer_id = None;
                    }
                    st.status = format!(
                        "{amount} · {} saved — the AWB comes when the office is reachable",
                        method.label()
                    );
                    Ok(None)
                }
            }
        })();
        self.end_pay();
        match result {
            Err(e) => {
                // The ledger line is what the seller is looking at. Pay
                // failures must show there, not only on the lock screen.
                app_state().lock().unwrap().last_receipt = e.clone();
                if e.contains("customer") {
                    self.ui
                        .text_input(cx, ids!(customer_name_input))
                        .set_key_focus(cx);
                }
                self.set_status(cx, e);
            }
            Ok(done) => {
                if method.is_credit() {
                    self.ui
                        .text_input(cx, ids!(customer_name_input))
                        .set_text(cx, "");
                }
                match done {
                    Some((sale_id, snack)) => self.show_undo_snackbar(cx, sale_id, snack),
                    // Queued on this device: say so where the seller looks.
                    None => {
                        let msg = app_state().lock().unwrap().status.clone();
                        self.set_status(cx, msg);
                    }
                }
            }
        }
        self.sync_chrome(cx);
    }

    /// Claim the charge slot. False means a charge is already running, and the
    /// caller must return without touching the cart.
    fn begin_pay(&mut self, cx: &mut Cx) -> bool {
        {
            let mut st = app_state().lock().unwrap();
            if st.pay_in_flight {
                st.status = "Charging — one moment".into();
            } else {
                st.pay_in_flight = true;
                return true;
            }
        }
        self.sync_chrome(cx);
        false
    }

    fn end_pay(&mut self) {
        app_state().lock().unwrap().pay_in_flight = false;
    }

    fn show_undo_snackbar(&mut self, cx: &mut Cx, sale_id: String, text: String) {
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
        }
        st.undo_sale_id = Some(sale_id);
        st.undo_snack_text = text;
        st.undo_timer = cx.start_timeout(10.0);
    }

    fn clear_undo_snackbar(&mut self, cx: &mut Cx) {
        let mut st = app_state().lock().unwrap();
        if !st.undo_timer.is_empty() {
            cx.stop_timer(st.undo_timer);
            st.undo_timer = Timer::empty();
        }
        st.undo_sale_id = None;
        st.undo_snack_text.clear();
    }

    fn undo_last_sale(&mut self, cx: &mut Cx) {
        let result = (|| -> Result<(), String> {
            let (is_client, sale_id, actor) = {
                let st = app_state().lock().unwrap();
                let actor = st.actor()?;
                let sale_id = st
                    .undo_sale_id
                    .clone()
                    .ok_or_else(|| "Nothing to undo".to_string())?;
                (st.device_role == DeviceRole::Client, sale_id, actor)
            };
            if is_client {
                let mut st = app_state().lock().unwrap();
                st.lan_job(
                    "/undo",
                    &net::RemoteUndoRequest {
                        sale_id: sale_id.clone(),
                    },
                    "Undo".into(),
                    "Booking undone on the office's books".into(),
                    phone::Follow::Refresh,
                    false,
                )?;
                st.status = "Undoing…".into();
                st.undo_sale_id = None;
                st.undo_snack_text.clear();
                return Ok(());
            }
            let mut st = app_state().lock().unwrap();
            let store = st.store.as_mut().ok_or_else(|| "No store".to_string())?;
            let sale = store.quick_undo_sale(&sale_id, &actor)?;
            st.cart = Cart::from_sale(&sale);
            st.selected_customer_id = sale.customer_id.clone();
            st.last_receipt = format!("Undone {}", &sale.id[..8.min(sale.id.len())]);
            st.status = "Sale undone — cart restored".into();
            st.undo_sale_id = None;
            st.undo_snack_text.clear();
            st.reload_all()?;
            Ok(())
        })();
        {
            let mut st = app_state().lock().unwrap();
            if !st.undo_timer.is_empty() {
                cx.stop_timer(st.undo_timer);
                st.undo_timer = Timer::empty();
            }
        }
        if let Err(e) = result {
            self.set_status(cx, e);
            self.clear_undo_snackbar(cx);
        } else {
            self.go_peer(cx, Screen::Pos);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_total_from_input(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, true);
    }

    /// Live Rounded edits must move totals immediately — waiting for blur
    /// leaves the field on the previous total and confuses the cashier.
    fn apply_cart_total_from_input_live(&mut self, cx: &mut Cx) {
        self.apply_cart_total_from_input_inner(cx, false);
    }

    fn apply_cart_total_from_input_inner(&mut self, cx: &mut Cx, report_errors: bool) {
        let total_field = self.ui.text_input(cx, ids!(cart_total_input));
        let text = total_field.text();
        let field_focused = cx.has_key_focus(total_field.area());
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if st.cart.is_empty() {
                return Err("Cart is empty".into());
            }
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(&text, &currency)?;
            if minor <= 0 {
                return Err("Rounded Total must be > 0".into());
            }
            let applied = st.cart.applied_total().paise();
            // Live `changed` can fire with a stale field after ADD. Ignore
            // mismatches when there is no explicit bill round and the seller
            // is not actively editing this field.
            if !report_errors
                && st.cart.charge_total.is_none()
                && minor != applied
                && !field_focused
            {
                return Ok(());
            }
            st.cart.set_charge_total(Money(minor))?;
            st.status = if st.cart.rounding().paise() == 0
                && st.cart.applied_total().paise() == st.cart.actual_total().paise()
            {
                "Rounded Total equals Applied Total".into()
            } else {
                format!(
                    "Actual {} · Applied {} · Rounded {} · round {}",
                    st.money(st.cart.actual_total()),
                    st.money(st.cart.applied_total()),
                    st.money(st.cart.rounded_total()),
                    st.money(st.cart.rounding())
                )
            };
            Ok(())
        })();
        if let Err(e) = result
            && report_errors
        {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_unit_price(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_unit_price(index, Money(minor))?;
            st.status = format!(
                "Unit price set · line {}",
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.line_total())
                        .unwrap_or_default()
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn apply_cart_line_total(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let currency = st.locale.currency.clone();
            let minor = locale::parse_major_input(text, &currency)?;
            st.cart.set_line_total(index, Money(minor))?;
            st.status = format!(
                "Line Applied {} · Actual {} · @ unchanged {}",
                st.money(Money(minor)),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.actual_total())
                        .unwrap_or(Money(0))
                ),
                st.money(
                    st.cart
                        .lines
                        .get(index)
                        .map(|l| l.unit_price)
                        .unwrap_or(Money(0))
                )
            );
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }
}

// In-app UI smoke — drives the real shell the way chrome taps do.
// Opt-in only (`KUMKAJ_SMOKE_SCRIPT=…`). Ignored when `KUMKAJ_ENV=production`.
// Modes: peers | full | soak. Handshake: write need_shot; runner saves PNG.


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmokeMode {
    Peers,
    Full,
    Soak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmokePhase {
    WaitUnlock,
    Peers,
    Actions,
    SoakLoop,
    Done,
}

struct SmokeState {
    mode: SmokeMode,
    phase: SmokePhase,
    step_i: usize,
    next_at: Instant,
    out: PathBuf,
    step_ms: u64,
    soak_until: Option<Instant>,
    failures: Vec<String>,
    peers: Vec<Screen>,
    actions_done: usize,
    soak_cycles: u32,
    /// Last step label written for need_shot.
    awaiting_shot: bool,
}

fn smoke_state() -> &'static Mutex<Option<SmokeState>> {
    static S: std::sync::OnceLock<Mutex<Option<SmokeState>>> = std::sync::OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

fn smoke_parse_mode() -> Option<SmokeMode> {
    if std::env::var("KUMKAJ_ENV").ok().as_deref() == Some("production") {
        return None;
    }
    match std::env::var("KUMKAJ_SMOKE_SCRIPT")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "peers" => Some(SmokeMode::Peers),
        "full" => Some(SmokeMode::Full),
        "soak" => Some(SmokeMode::Soak),
        "" | "0" | "off" | "false" | "no" => None,
        other => {
            log!("Kumkaj smoke: unknown KUMKAJ_SMOKE_SCRIPT={other} (use peers|full|soak)");
            None
        }
    }
}

fn smoke_out_dir() -> PathBuf {
    if let Ok(p) = std::env::var("KUMKAJ_SMOKE_OUT") {
        let p = PathBuf::from(p);
        let _ = std::fs::create_dir_all(&p);
        p
    } else if let Ok(d) = std::env::var("KUMKAJ_DATA_DIR") {
        let p = PathBuf::from(d).join("smoke-out");
        let _ = std::fs::create_dir_all(&p);
        p
    } else {
        let p = std::env::temp_dir().join(format!("kirano-smoke-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        p
    }
}

fn smoke_step_ms() -> u64 {
    std::env::var("KUMKAJ_SMOKE_STEP_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(900)
        .clamp(200, 30_000)
}

fn smoke_soak_secs() -> u64 {
    std::env::var("KUMKAJ_SMOKE_SOAK_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn smoke_shop_has_floor() -> bool {
    app_state().lock().unwrap().venue_mode.creates_kot()
}

fn smoke_peer_tour() -> Vec<Screen> {
    let mut peers = vec![
        Screen::Books,
        Screen::Pos,
        Screen::Buy,
        Screen::Menu,
        Screen::Floor,
        Screen::Kitchen,
        Screen::Sales,
        Screen::DayClose,
        Screen::Reports,
        Screen::Inventory,
        Screen::Credit,
        Screen::Settings,
        // Team lives under Settings → Team (not a chrome peer).
    ];
    // Counter shops (Kirano) have no tables or kitchen. Do not tour those screens.
    if !smoke_shop_has_floor() {
        peers.retain(|s| !matches!(s, Screen::Floor | Screen::Kitchen));
    }
    peers
}

impl App {
    /// Schedule / advance the in-app smoke script. Call from Signal + NextFrame.
    fn tick_smoke_script(&mut self, cx: &mut Cx) {
        let Some(mode) = smoke_parse_mode() else {
            return;
        };
        {
            let mut slot = smoke_state().lock().unwrap();
            if slot.is_none() {
                let out = smoke_out_dir();
                let soak_secs = smoke_soak_secs();
                let soak_until = if mode == SmokeMode::Soak && soak_secs > 0 {
                    Some(Instant::now() + Duration::from_secs(soak_secs))
                } else if mode == SmokeMode::Soak {
                    // Default soak: 30 minutes when mode=soak and secs unset.
                    Some(Instant::now() + Duration::from_secs(30 * 60))
                } else {
                    None
                };
                log!(
                    "Kumkaj smoke: start mode={mode:?} out={} step_ms={} soak={:?}",
                    out.display(),
                    smoke_step_ms(),
                    soak_until.map(|t| t.saturating_duration_since(Instant::now()))
                );
                *slot = Some(SmokeState {
                    mode,
                    phase: SmokePhase::WaitUnlock,
                    step_i: 0,
                    next_at: Instant::now() + Duration::from_millis(400),
                    out,
                    step_ms: smoke_step_ms(),
                    soak_until,
                    failures: Vec::new(),
                    peers: smoke_peer_tour(),
                    actions_done: 0,
                    soak_cycles: 0,
                    awaiting_shot: false,
                });
            }
        }
        // Keep frames coming while smoke runs (no worker Signal required).
        let _ = cx.new_next_frame();

        // Wait for external screenshot ack so the PNG matches this step.
        {
            let mut slot = smoke_state().lock().unwrap();
            let Some(st) = slot.as_mut() else {
                return;
            };
            if st.awaiting_shot {
                let need = st.out.join("need_shot");
                if need.exists() {
                    return;
                }
                st.awaiting_shot = false;
                st.next_at = Instant::now() + Duration::from_millis(st.step_ms / 3);
            }
            if Instant::now() < st.next_at {
                return;
            }
        }

        self.smoke_advance(cx);
    }

    fn smoke_advance(&mut self, cx: &mut Cx) {
        let (phase, step_i, mode) = {
            let st = smoke_state().lock().unwrap();
            let s = st.as_ref().unwrap();
            (s.phase, s.step_i, s.mode)
        };
        match phase {
            SmokePhase::WaitUnlock => {
                let unlocked = app_state().lock().unwrap().session.is_some();
                if !unlocked {
                    smoke_state().lock().unwrap().as_mut().unwrap().next_at =
                        Instant::now() + Duration::from_millis(300);
                    return;
                }
                self.smoke_mark(cx, "000_unlocked");
                let mut st = smoke_state().lock().unwrap();
                let s = st.as_mut().unwrap();
                s.phase = SmokePhase::Peers;
                s.step_i = 0;
            }
            SmokePhase::Peers => {
                let peer = {
                    let st = smoke_state().lock().unwrap();
                    let s = st.as_ref().unwrap();
                    s.peers.get(step_i).copied()
                };
                if let Some(screen) = peer {
                    self.go_peer(cx, screen);
                    self.sync_chrome(cx);
                    let got = app_state().lock().unwrap().screen;
                    let name = format!("{:03}_peer_{screen:?}", step_i + 1);
                    if got != screen {
                        // Branches / Team may bounce — record soft fail, continue.
                        self.smoke_note_fail(format!(
                            "{name}: wanted {screen:?}, landed on {got:?}"
                        ));
                    }
                    self.smoke_mark(cx, &name);
                    smoke_state().lock().unwrap().as_mut().unwrap().step_i = step_i + 1;
                } else {
                    let mut st = smoke_state().lock().unwrap();
                    let s = st.as_mut().unwrap();
                    s.phase = match mode {
                        SmokeMode::Peers => SmokePhase::Done,
                        SmokeMode::Full | SmokeMode::Soak => SmokePhase::Actions,
                    };
                    s.step_i = 0;
                    s.actions_done = 0;
                    s.next_at = Instant::now() + Duration::from_millis(s.step_ms);
                }
            }
            SmokePhase::Actions => {
                let done = self.smoke_run_action(cx, step_i);
                if done {
                    let mut st = smoke_state().lock().unwrap();
                    let s = st.as_mut().unwrap();
                    if s.mode == SmokeMode::Soak {
                        s.phase = SmokePhase::SoakLoop;
                        s.step_i = 0;
                        s.soak_cycles = 0;
                    } else {
                        s.phase = SmokePhase::Done;
                    }
                } else {
                    smoke_state().lock().unwrap().as_mut().unwrap().step_i = step_i + 1;
                }
            }
            SmokePhase::SoakLoop => {
                let (until, peers_len) = {
                    let st = smoke_state().lock().unwrap();
                    let s = st.as_ref().unwrap();
                    (s.soak_until, s.peers.len())
                };
                if until.is_some_and(|u| Instant::now() >= u) {
                    smoke_state().lock().unwrap().as_mut().unwrap().phase = SmokePhase::Done;
                    return;
                }
                // Health: store still open, session still Owner.
                {
                    let st = app_state().lock().unwrap();
                    if st.session.is_none() || st.store.is_none() {
                        self.smoke_note_fail("soak: session or store gone".into());
                        smoke_state().lock().unwrap().as_mut().unwrap().phase = SmokePhase::Done;
                        return;
                    }
                }
                let peer = {
                    let st = smoke_state().lock().unwrap();
                    let s = st.as_ref().unwrap();
                    s.peers[step_i % peers_len.max(1)]
                };
                self.go_peer(cx, peer);
                self.sync_chrome(cx);
                // Light books pulse every cycle: day book text must still build.
                if step_i % peers_len.max(1) == 0 {
                    let ok = (|| -> Result<(), String> {
                        let st = app_state().lock().unwrap();
                        let store = st.store.as_ref().ok_or("no store")?;
                        let _ = store.books_report_text("today")?;
                        Ok(())
                    })();
                    if let Err(e) = ok {
                        self.smoke_note_fail(format!("soak books_report: {e}"));
                    }
                    let mut st = smoke_state().lock().unwrap();
                    let s = st.as_mut().unwrap();
                    s.soak_cycles = s.soak_cycles.saturating_add(1);
                    let name = format!("soak_cycle_{:04}", s.soak_cycles);
                    drop(st);
                    self.smoke_mark(cx, &name);
                }
                smoke_state().lock().unwrap().as_mut().unwrap().step_i = step_i + 1;
            }
            SmokePhase::Done => {
                self.smoke_finish();
            }
        }
    }

    /// One action step. Returns true when the action list is finished.
    fn smoke_run_action(&mut self, cx: &mut Cx, i: usize) -> bool {
        match i {
            0 => {
                if !smoke_shop_has_floor() {
                    self.go_peer(cx, Screen::Pos);
                    self.sync_chrome(cx);
                    self.smoke_mark(cx, "100_counter_sell");
                    return false;
                }
                self.go_peer(cx, Screen::Floor);
                self.sync_chrome(cx);
                // Open first non-free table into the order cart if any.
                let tid = {
                    let st = app_state().lock().unwrap();
                    st.tables
                        .iter()
                        .find(|t| t.floor_state != domain::FloorState::Free)
                        .map(|t| t.id.clone())
                };
                if let Some(tid) = tid {
                    let mut st = app_state().lock().unwrap();
                    let _ = st.load_table_cart(&tid);
                }
                self.sync_chrome(cx);
                self.smoke_mark(cx, "100_floor_live_tables");
                false
            }
            1 => {
                if !smoke_shop_has_floor() {
                    self.go_peer(cx, Screen::Books);
                    self.sync_chrome(cx);
                    self.smoke_mark(cx, "101_counter_books");
                    return false;
                }
                self.go_peer(cx, Screen::Kitchen);
                self.sync_chrome(cx);
                let advanced = (|| -> Result<bool, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or("no store")?;
                    let actor = st.session.clone().ok_or("no session")?;
                    let open = store.list_open_kots()?;
                    let Some(kot) = open.into_iter().find(|k| !k.is_cancel()) else {
                        return Ok(false);
                    };
                    store.advance_kot(&kot.id, &actor)?;
                    Ok(true)
                })();
                match advanced {
                    Ok(true) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.sync_chrome(cx);
                        self.smoke_mark(cx, "101_kitchen_advance");
                    }
                    Ok(false) => self.smoke_mark(cx, "101_kitchen_no_open_kot"),
                    Err(e) => {
                        self.smoke_note_fail(format!("kitchen advance: {e}"));
                        self.smoke_mark(cx, "101_kitchen_advance_fail");
                    }
                }
                false
            }
            2 => {
                self.go_peer(cx, Screen::Sales);
                self.sync_chrome(cx);
                let sold = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or("no store")?;
                    let actor = st.session.clone().ok_or("no session")?;
                    let products = store.list_products(false)?;
                    let p = products
                        .into_iter()
                        .find(|p| p.name.contains("Chaas") || p.name.contains("Chai"))
                        .ok_or("no drink product")?;
                    let mut cart = Cart::default();
                    cart.add_or_bump(&p);
                    let draft = cart.to_sale_draft()?;
                    let sale = store.finalize_sale(&draft, PayMethod::Cash, None, None, &actor)?;
                    Ok(sale.id)
                })();
                match sold {
                    Ok(id) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.sync_chrome(cx);
                        self.smoke_mark(cx, &format!("102_sales_cash_{}", &id[..8.min(id.len())]));
                    }
                    Err(e) => {
                        self.smoke_note_fail(format!("cash sale: {e}"));
                        self.smoke_mark(cx, "102_sales_cash_fail");
                    }
                }
                false
            }
            3 => {
                self.go_peer(cx, Screen::Books);
                self.sync_chrome(cx);
                let ok = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let _ = st.refresh_books_overview();
                    let store = st.store.as_ref().ok_or("no store")?;
                    let text = store.books_report_text("today")?;
                    if text.trim().is_empty() {
                        return Err("empty day books".into());
                    }
                    Ok(())
                })();
                if let Err(e) = ok {
                    self.smoke_note_fail(format!("books: {e}"));
                }
                self.smoke_mark(cx, "103_day_book");
                false
            }
            4 => {
                self.go_peer(cx, Screen::Settings);
                self.sync_chrome(cx);
                for (panel, tag) in [
                    ("printers", "104a_settings_printers"),
                    ("bill", "104b_settings_bill"),
                    ("backup", "104c_settings_backup"),
                    ("diag", "104d_settings_diag"),
                    ("language", "104e_settings_language"),
                ] {
                    self.open_settings_panel(cx, Some(panel));
                    self.sync_chrome(cx);
                    // Mark each panel; runner may only catch the last if fast —
                    // still exercises paint paths.
                    self.smoke_log_step(tag);
                }
                self.open_settings_panel(cx, None);
                self.smoke_mark(cx, "104_settings_panels");
                false
            }
            5 => {
                self.go_peer(cx, Screen::Menu);
                self.sync_chrome(cx);
                self.smoke_mark(cx, "105_menu");
                false
            }
            6 => {
                // Back/Forward visit history: peer jumps must leave ‹ usable.
                if smoke_shop_has_floor() {
                    self.go_peer(cx, Screen::Floor);
                    self.go_peer(cx, Screen::Kitchen);
                } else {
                    self.go_peer(cx, Screen::Menu);
                    self.go_peer(cx, Screen::Pos);
                }
                let can_back = {
                    let st = app_state().lock().unwrap();
                    !st.nav_back.is_empty()
                };
                if !can_back {
                    self.smoke_note_fail("nav_back empty after peer jumps".into());
                }
                let _ = self.navigate_back(cx);
                self.sync_chrome(cx);
                self.smoke_mark(cx, "106_nav_back");
                false
            }
            _ => true,
        }
    }

    fn smoke_mark(&mut self, cx: &mut Cx, name: &str) {
        self.smoke_log_step(name);
        let out = {
            let mut st = smoke_state().lock().unwrap();
            let s = st.as_mut().unwrap();
            s.awaiting_shot = true;
            s.next_at = Instant::now() + Duration::from_secs(30); // cap wait for runner
            s.out.clone()
        };
        let _ = std::fs::write(out.join("current_step"), name);
        let _ = std::fs::write(out.join("need_shot"), name);
        // Force a paint so the shot sees this screen.
        self.sync_chrome(cx);
        self.ui.redraw(cx);
        log!("Kumkaj smoke: step {name}");
    }

    fn smoke_log_step(&mut self, name: &str) {
        let out = smoke_state()
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.out.clone())
            .unwrap_or_else(smoke_out_dir);
        let screen = format!("{:?}", app_state().lock().unwrap().screen);
        let line = format!(
            "{{\"t\":\"{}\",\"step\":\"{name}\",\"screen\":\"{screen}\"}}\n",
            chrono::Local::now().to_rfc3339()
        );
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("steps.jsonl"))
        {
            let _ = f.write_all(line.as_bytes());
        }
    }

    fn smoke_note_fail(&mut self, msg: String) {
        log!("Kumkaj smoke FAIL: {msg}");
        if let Some(s) = smoke_state().lock().unwrap().as_mut() {
            s.failures.push(msg);
        }
    }

    fn smoke_finish(&mut self) {
        static FINISHED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if FINISHED.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let (out, failures, cycles) = {
            let st = smoke_state().lock().unwrap();
            let s = st.as_ref().unwrap();
            (s.out.clone(), s.failures.clone(), s.soak_cycles)
        };
        let pass = failures.is_empty();
        let report = format!(
            "result={}\nfailures={}\nsoak_cycles={cycles}\n{}\n",
            if pass { "PASS" } else { "FAIL" },
            failures.len(),
            failures.join("\n")
        );
        let _ = std::fs::write(out.join("RESULT.txt"), &report);
        let _ = std::fs::write(
            out.join(if pass { "PASS" } else { "FAIL" }),
            &report,
        );
        log!("Kumkaj smoke finished: {}", report.replace('\n', " | "));
        // Give the runner a moment to grab the last shot, then exit.
        let quit = !matches!(
            std::env::var("KUMKAJ_SMOKE_QUIT").ok().as_deref(),
            Some("0" | "false" | "no")
        );
        if quit {
            std::thread::sleep(Duration::from_millis(800));
            std::process::exit(if pass { 0 } else { 1 });
        }
    }
}


// In-app UI smoke — drives the real shell the way chrome taps do.
// Opt-in only (`KUMKAJ_SMOKE_SCRIPT=…`). Ignored when `KUMKAJ_ENV=production`.

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
    peers: Vec<NavTab>,
    soak_cycles: u32,
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
            log!("Kumkaj smoke: unknown KUMKAJ_SMOKE_SCRIPT={other}");
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
        let p = std::env::temp_dir().join(format!("moklanjo-smoke-{}", std::process::id()));
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

fn smoke_peer_tabs() -> Vec<NavTab> {
    vec![
        NavTab::Books,
        NavTab::Sell,
        NavTab::Buy,
        NavTab::Rates,
        NavTab::Ops,
        NavTab::Cod,
        NavTab::Riders,
        NavTab::Sales,
        NavTab::DayClose,
        NavTab::Reports,
        NavTab::Inventory,
        NavTab::Credit,
        NavTab::Settings,
        NavTab::Team,
    ]
}

impl App {
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
                    Some(Instant::now() + Duration::from_secs(30 * 60))
                } else {
                    None
                };
                log!(
                    "Kumkaj smoke: start mode={mode:?} out={} step_ms={}",
                    out.display(),
                    smoke_step_ms()
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
                    peers: smoke_peer_tabs(),
                    soak_cycles: 0,
                    awaiting_shot: false,
                });
            }
        }
        let _ = cx.new_next_frame();

        {
            let mut slot = smoke_state().lock().unwrap();
            let Some(st) = slot.as_mut() else {
                return;
            };
            if st.awaiting_shot {
                if st.out.join("need_shot").exists() {
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

    fn smoke_go_tab(&mut self, cx: &mut Cx, tab: NavTab) {
        {
            let mut st = app_state().lock().unwrap();
            match tab {
                NavTab::Ops => st.reports_nav = ReportsNav::Ops,
                NavTab::Cod => st.reports_nav = ReportsNav::Cod,
                NavTab::Riders => st.reports_nav = ReportsNav::Riders,
                NavTab::Reports => st.reports_nav = ReportsNav::Bank,
                _ => {}
            }
        }
        self.go_peer(cx, screen_for_nav_tab(tab));
        self.sync_chrome(cx);
    }

    fn smoke_advance(&mut self, cx: &mut Cx) {
        let (phase, step_i, mode) = {
            let st = smoke_state().lock().unwrap();
            let s = st.as_ref().unwrap();
            (s.phase, s.step_i, s.mode)
        };
        match phase {
            SmokePhase::WaitUnlock => {
                if app_state().lock().unwrap().session.is_none() {
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
                let tab = {
                    let st = smoke_state().lock().unwrap();
                    st.as_ref().unwrap().peers.get(step_i).copied()
                };
                if let Some(tab) = tab {
                    self.smoke_go_tab(cx, tab);
                    let name = format!("{:03}_peer_{tab:?}", step_i + 1);
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
                    s.next_at = Instant::now() + Duration::from_millis(s.step_ms);
                }
            }
            SmokePhase::Actions => {
                if self.smoke_run_action(cx, step_i) {
                    let mut st = smoke_state().lock().unwrap();
                    let s = st.as_mut().unwrap();
                    s.phase = if s.mode == SmokeMode::Soak {
                        SmokePhase::SoakLoop
                    } else {
                        SmokePhase::Done
                    };
                    s.step_i = 0;
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
                {
                    let st = app_state().lock().unwrap();
                    if st.session.is_none() || st.store.is_none() {
                        self.smoke_note_fail("soak: session or store gone".into());
                        smoke_state().lock().unwrap().as_mut().unwrap().phase = SmokePhase::Done;
                        return;
                    }
                }
                let tab = {
                    let st = smoke_state().lock().unwrap();
                    let s = st.as_ref().unwrap();
                    s.peers[step_i % peers_len.max(1)]
                };
                self.smoke_go_tab(cx, tab);
                if step_i % peers_len.max(1) == 0 {
                    let ok = (|| -> Result<(), String> {
                        let st = app_state().lock().unwrap();
                        let store = st.store.as_ref().ok_or("no store")?;
                        let _ = store.books_report_text("today")?;
                        Ok(())
                    })();
                    if let Err(e) = ok {
                        self.smoke_note_fail(format!("soak books: {e}"));
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
            SmokePhase::Done => self.smoke_finish(),
        }
    }

    fn smoke_run_action(&mut self, cx: &mut Cx, i: usize) -> bool {
        match i {
            0 => {
                self.smoke_go_tab(cx, NavTab::Books);
                let ok = (|| -> Result<(), String> {
                    let mut st = app_state().lock().unwrap();
                    let _ = st.refresh_books_overview();
                    let store = st.store.as_ref().ok_or("no store")?;
                    let text = store.books_report_text("today")?;
                    if text.trim().is_empty() {
                        return Err("empty books".into());
                    }
                    Ok(())
                })();
                if let Err(e) = ok {
                    self.smoke_note_fail(format!("books: {e}"));
                }
                self.smoke_mark(cx, "100_day_book");
                false
            }
            1 => {
                self.smoke_go_tab(cx, NavTab::Sell);
                let booked = (|| -> Result<String, String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or("no store")?;
                    let actor = st.session.clone().ok_or("no session")?;
                    let rates = store.list_products(false)?;
                    let rate = rates.into_iter().next().ok_or("no rate")?;
                    let mut cart = Cart::default();
                    cart.add_or_bump(&rate);
                    if let Some(line) = cart.lines.last_mut() {
                        let _ = booking::apply_weight_and_slab(line, &rate, "1".parse().unwrap());
                    }
                    let mut draft = cart.to_sale_draft()?;
                    draft.destination = "Smoke Town".into();
                    draft.consignee_name = "Smoke Consignee".into();
                    draft.chargeable_weight_kg = "1".parse().unwrap();
                    draft.actual_weight_kg = "1".parse().unwrap();
                    draft.zone_id = rate.zone_id.clone();
                    let sale = store.finalize_sale(&draft, PayMethod::Cash, None, &actor)?;
                    Ok(sale.awb_no)
                })();
                match booked {
                    Ok(awb) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.sync_chrome(cx);
                        self.smoke_mark(cx, &format!("101_booked_{awb}"));
                    }
                    Err(e) => {
                        self.smoke_note_fail(format!("book: {e}"));
                        self.smoke_mark(cx, "101_book_fail");
                    }
                }
                false
            }
            2 => {
                self.smoke_go_tab(cx, NavTab::Ops);
                let dispatched = (|| -> Result<(), String> {
                    let st = app_state().lock().unwrap();
                    let store = st.store.as_ref().ok_or("no store")?;
                    let actor = st.session.clone().ok_or("no session")?;
                    let riders = store.list_riders()?;
                    let rider = riders.into_iter().next().ok_or("no rider")?;
                    let sales = store.recent_sales(30)?;
                    let ids: Vec<String> = sales
                        .into_iter()
                        .filter(|s| {
                            s.status == "completed"
                                && manifest::TrackingStatus::parse(&s.tracking_status)
                                    .is_dispatchable()
                        })
                        .take(2)
                        .map(|s| s.id)
                        .collect();
                    if ids.is_empty() {
                        return Err("no dispatchable AWB".into());
                    }
                    store.create_manifest(
                        manifest::HandoverKind::OwnFleet,
                        "",
                        &rider.id,
                        &ids,
                        "smoke run",
                        &actor,
                    )?;
                    Ok(())
                })();
                match dispatched {
                    Ok(()) => {
                        let _ = app_state().lock().unwrap().reload_all();
                        self.sync_chrome(cx);
                        self.smoke_mark(cx, "102_ops_dispatch");
                    }
                    Err(e) => {
                        // Soft: seed may have already dispatched everything.
                        log!("Kumkaj smoke: ops skip ({e})");
                        self.smoke_mark(cx, "102_ops_skip");
                    }
                }
                false
            }
            3 => {
                self.smoke_go_tab(cx, NavTab::Cod);
                self.smoke_mark(cx, "103_cod_board");
                false
            }
            4 => {
                self.smoke_go_tab(cx, NavTab::Rates);
                self.smoke_mark(cx, "104_rates");
                false
            }
            5 => {
                self.smoke_go_tab(cx, NavTab::Settings);
                for panel in ["printers", "bill", "backup", "diag", "language"] {
                    self.open_settings_panel(cx, Some(panel));
                    self.sync_chrome(cx);
                    self.smoke_log_step(&format!("105_settings_{panel}"));
                }
                self.open_settings_panel(cx, None);
                self.smoke_mark(cx, "105_settings_panels");
                false
            }
            6 => {
                self.smoke_go_tab(cx, NavTab::Sell);
                self.smoke_go_tab(cx, NavTab::Ops);
                if app_state().lock().unwrap().nav_back.is_empty() {
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
            s.next_at = Instant::now() + Duration::from_secs(30);
            s.out.clone()
        };
        let _ = std::fs::write(out.join("current_step"), name);
        let _ = std::fs::write(out.join("need_shot"), name);
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
        let _ = std::fs::write(out.join(if pass { "PASS" } else { "FAIL" }), &report);
        log!("Kumkaj smoke finished: {}", report.replace('\n', " | "));
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

fn app_state() -> &'static Mutex<AppState> {
    static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(AppState::boot()))
}

/// Pending desktop window restore. Applied at Startup via `configure_window`,
/// then re-checked on the first `WindowGeomChange` (Windows sometimes creates
/// with the DSL default and ignores create_inner_size until a post-create resize).
fn pending_window_geom() -> &'static Mutex<Option<store::WindowGeomPrefs>> {
    static PENDING: OnceLock<Mutex<Option<store::WindowGeomPrefs>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(None))
}

/// Edge-swipe tracking: left edge → back, right edge → forward.
#[derive(Clone, Copy)]
enum SwipeNav {
    Back,
    Forward,
}

fn swipe_origin() -> &'static Mutex<Option<(u64, DVec2, SwipeNav)>> {
    static ORIGIN: OnceLock<Mutex<Option<(u64, DVec2, SwipeNav)>>> = OnceLock::new();
    ORIGIN.get_or_init(|| Mutex::new(None))
}

/// Desktop mouse/trackpad edge swipe (Makepad has no Event::FingerUp at app level).
fn mouse_swipe_origin() -> &'static Mutex<Option<(DVec2, SwipeNav)>> {
    static ORIGIN: OnceLock<Mutex<Option<(DVec2, SwipeNav)>>> = OnceLock::new();
    ORIGIN.get_or_init(|| Mutex::new(None))
}

/// Dragging the rail/main split: stores grab offset (pointer x − rail width).
/// Windows firewall for the LAN listener: checked and opened on worker
/// threads (`netsh`, the admin prompt), read by `sync_chrome`.
#[derive(Default)]
struct FirewallState {
    /// None = not checked yet; Some(false) = phones are blocked.
    open: Option<bool>,
    /// Waiting for the check or for the admin prompt.
    busy: bool,
    /// Why the last attempt failed.
    note: String,
    /// The UI has not shown the latest answer yet.
    fresh: bool,
}

fn firewall_state() -> &'static Mutex<FirewallState> {
    static SLOT: OnceLock<Mutex<FirewallState>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(FirewallState::default()))
}

/// Run `job` (check or open) off the UI thread and wake the UI with its answer.
fn firewall_job(job: fn() -> Result<bool, String>) {
    {
        let mut fw = firewall_state().lock().unwrap();
        if fw.busy {
            return;
        }
        fw.busy = true;
    }
    let _ = std::thread::Builder::new()
        .name("moklanjo-firewall".into())
        .spawn(move || {
            let answer = job();
            {
                let mut fw = firewall_state().lock().unwrap();
                fw.busy = false;
                fw.fresh = true;
                match answer {
                    Ok(open) => {
                        fw.open = Some(open);
                        fw.note.clear();
                    }
                    Err(e) => fw.note = e,
                }
            }
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

/// A device's one-off talk with the Server (pairing, signing in) runs on a
/// thread; its answer waits here until the UI wakes.
enum LanOnce {
    Paired(Result<(net::Link, net::SyncSnapshot), net::LanError>),
    SignIn {
        staff_id: Option<String>,
        pin: String,
        reply: Result<net::LoginOk, net::LanError>,
    },
}

fn lan_once() -> &'static Mutex<Option<LanOnce>> {
    static SLOT: OnceLock<Mutex<Option<LanOnce>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// The name a Server lists this device under.
fn device_name() -> String {
    if cfg!(target_os = "android") {
        return "Android device".into();
    }
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|n| !n.trim().is_empty())
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|n| n.trim().to_string())
        })
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Device".into())
}

/// "Find the shop's server" runs on a worker thread; its answer waits here
/// until the UI wakes (`Event::Signal`).
fn discovery_result() -> &'static Mutex<Option<Vec<net::FoundServer>>> {
    static SLOT: OnceLock<Mutex<Option<Vec<net::FoundServer>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// The books the automatic backup looks after (the open server's), and
/// whether its thread runs. It checks every few minutes and backs up once
/// a day; the UI wakes when a backup was made.
fn auto_backup_target() -> &'static Mutex<(Option<PathBuf>, bool)> {
    static SLOT: OnceLock<Mutex<(Option<PathBuf>, bool)>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new((None, false)))
}

/// A backup was just made: the Settings line says so on the next wake.
static BACKUP_MADE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Manual "Create backup" result (VACUUM can take seconds — never on the UI thread).
fn backup_once() -> &'static Mutex<Option<Result<PathBuf, String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<PathBuf, String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn backup_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn backup_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

/// Run a books backup off the UI thread; the Signal handler paints the result.
fn backup_job(db_path: PathBuf, dest: PathBuf) {
    if backup_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    backup_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Creating backup…");
    let _ = std::thread::Builder::new()
        .name("moklanjo-backup-once".into())
        .spawn(move || {
            let result = if backup_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                Err("Cancelled".into())
            } else {
                Store::open_path(&db_path).and_then(|s| s.backup_to(&dest).map(|()| dest))
            };
            *backup_once().lock().unwrap() = Some(result);
            backup_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

/// Shared "work in progress" line for long jobs (restore, CSV, backup copy…).
fn books_work_status() -> &'static Mutex<Option<String>> {
    static SLOT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn set_books_work_status(msg: &str) {
    *books_work_status().lock().unwrap() = Some(msg.to_string());
    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
}

fn clear_books_work_status() {
    *books_work_status().lock().unwrap() = None;
}

fn restore_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn restore_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn restore_once() -> &'static Mutex<Option<Result<(PathBuf, PathBuf), String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<(PathBuf, PathBuf), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn restore_job(db_path: PathBuf, backup: PathBuf, before: PathBuf) {
    if restore_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    restore_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Restoring backup…");
    let _ = std::thread::Builder::new()
        .name("moklanjo-restore".into())
        .spawn(move || {
            let result = (|| -> Result<(PathBuf, PathBuf), String> {
                if restore_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                store::check_backup_file(&backup)?;
                if restore_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                {
                    let s = Store::open_path(&db_path)?;
                    s.backup_to(&before)?;
                }
                if restore_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let _ = Store::restore_from(&backup)?;
                Ok((backup, before))
            })();
            *restore_once().lock().unwrap() = Some(result);
            restore_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn backup_copy_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn backup_copy_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn backup_copy_once() -> &'static Mutex<Option<Result<(PathBuf, String), String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<(PathBuf, String), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn backup_copy_job(db_path: PathBuf, dest: PathBuf, suggested_name: String) {
    if backup_copy_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    backup_copy_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Making backup copy…");
    let _ = std::thread::Builder::new()
        .name("moklanjo-backup-copy".into())
        .spawn(move || {
            let result = if backup_copy_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                Err("Cancelled".into())
            } else {
                Store::open_path(&db_path)
                    .and_then(|s| s.backup_to(&dest).map(|()| (dest, suggested_name)))
            };
            *backup_copy_once().lock().unwrap() = Some(result);
            backup_copy_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn csv_import_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn csv_import_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn csv_import_once() -> &'static Mutex<Option<Result<String, String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<String, String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// `kind`: "parties" | "items" | "pincodes" | "carrier_bill"
fn csv_import_job(db_path: PathBuf, path: PathBuf, kind: &'static str) {
    if csv_import_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    csv_import_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    let label = match kind {
        "parties" => "Importing parties…",
        "pincodes" => "Importing pincodes…",
        "carrier_bill" => "Importing carrier bill…",
        _ => "Importing items…",
    };
    set_books_work_status(label);
    let _ = std::thread::Builder::new()
        .name("moklanjo-csv-import".into())
        .spawn(move || {
            let result = (|| -> Result<String, String> {
                if csv_import_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Could not read the file: {e}"))?;
                if csv_import_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let store = Store::open_path(&db_path)?;
                match kind {
                    "carrier_bill" => {
                        let report = store.reconcile_carrier_bill_csv(&text)?;
                        let diffs = report
                            .lines()
                            .skip(1)
                            .filter(|l| {
                                l.contains("charge_diff")
                                    || l.contains("weight_diff")
                                    || l.contains("missing")
                            })
                            .count();
                        let (applied, skipped) = store.apply_carrier_bill_costs(&text)?;
                        // Report text is stored on Signal via a side channel.
                        *carrier_bill_report_once().lock().unwrap() = Some(report);
                        Ok(format!(
                            "Carrier bill: {diffs} flagged · applied cost on {applied} · skipped {skipped}. Copy report from Reports."
                        ))
                    }
                    other => {
                        let (added, skipped, problems) = match other {
                            "parties" => store.import_parties_csv(&text)?,
                            "pincodes" => store.import_pincodes_csv(&text)?,
                            _ => store.import_items_csv(&text)?,
                        };
                        let mut msg = format!(
                            "Imported {added} new, skipped {skipped} (already there or unusable)"
                        );
                        if let Some(p) = problems.first() {
                            msg.push_str(&format!(" · first problem: {p}"));
                        }
                        Ok(msg)
                    }
                }
            })();
            *csv_import_once().lock().unwrap() = Some(result);
            csv_import_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

/// Set cancel for whichever books/settings job is currently busy (best-effort
/// for VACUUM mid-flight; real abort before open / between phases).
fn request_books_job_cancel() {
    if backup_busy().load(std::sync::atomic::Ordering::SeqCst) {
        backup_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if restore_busy().load(std::sync::atomic::Ordering::SeqCst) {
        restore_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if backup_copy_busy().load(std::sync::atomic::Ordering::SeqCst) {
        backup_copy_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if csv_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
        csv_import_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if branch_busy().load(std::sync::atomic::Ordering::SeqCst) {
        branch_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if diag_export_busy().load(std::sync::atomic::Ordering::SeqCst) {
        diag_export_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if printer_add_busy().load(std::sync::atomic::Ordering::SeqCst) {
        printer_add_cancel_flag().store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

fn printer_add_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn printer_add_cancel_flag() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn printer_add_once() -> &'static Mutex<Option<Result<(), String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<(), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Persist printer pool off the UI thread (file IO can hitch Settings).
fn printer_add_job(base: PathBuf, draft: print_share::DevicePrinters) {
    if printer_add_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    printer_add_cancel_flag().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Saving printer…");
    let _ = std::thread::Builder::new()
        .name("moklanjo-printer-add".into())
        .spawn(move || {
            let result = if printer_add_cancel_flag().load(std::sync::atomic::Ordering::SeqCst) {
                Err("Cancelled".into())
            } else {
                print_share::save_device_printers(&base, &draft)
            };
            *printer_add_once().lock().unwrap() = Some(result);
            printer_add_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn branch_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn branch_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn branch_once() -> &'static Mutex<Option<Result<String, String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<String, String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn branch_export_job(db_path: PathBuf) {
    if branch_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    branch_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Exporting day…");
    let _ = std::thread::Builder::new()
        .name("moklanjo-branch-export".into())
        .spawn(move || {
            let result = (|| -> Result<String, String> {
                if branch_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let store = Store::open_path(&db_path)?;
                let branch = store.active_branch()?;
                let json = store.export_branch_snapshot_json()?;
                if branch_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let dir = store::data_dir()?.join("backups");
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                let code = if branch.code.trim().is_empty() {
                    "BRANCH".to_string()
                } else {
                    branch.code.trim().to_uppercase()
                };
                let path = dir.join(format!(
                    "branch_snapshot_{code}_{}.json",
                    chrono::Local::now().format("%Y-%m-%d")
                ));
                std::fs::write(&path, &json).map_err(|e| e.to_string())?;
                let copied = print_share::copy_text(&json);
                Ok(format!(
                    "Snapshot: {}{}",
                    path.display(),
                    if copied { " (also copied)" } else { "" }
                ))
            })();
            *branch_once().lock().unwrap() = Some(result);
            branch_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn branch_import_job(db_path: PathBuf) {
    if branch_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    branch_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Importing day…");
    let _ = std::thread::Builder::new()
        .name("moklanjo-branch-import".into())
        .spawn(move || {
            let result = (|| -> Result<String, String> {
                if branch_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let path = store::data_dir()?
                    .join("backups")
                    .join("import_branch_snapshot.json");
                if !path.exists() {
                    return Err("Missing backups/import_branch_snapshot.json".into());
                }
                let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                if branch_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let store = Store::open_path(&db_path)?;
                let branch = store.import_branch_snapshot(&json)?;
                Ok(format!("Imported day for {}", branch.label()))
            })();
            *branch_once().lock().unwrap() = Some(result);
            branch_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn diag_export_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn diag_export_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn diag_export_once() -> &'static Mutex<Option<Result<(diag::ExportPaths, bool), String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<(diag::ExportPaths, bool), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn diag_export_job(for_send: bool) {
    if diag_export_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    diag_export_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status(if for_send {
        "Creating share package…"
    } else {
        "Exporting diagnostics…"
    });
    let _ = std::thread::Builder::new()
        .name("moklanjo-diag-export".into())
        .spawn(move || {
            let result = if diag_export_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                Err("Cancelled".into())
            } else if for_send {
                diag::export_for_send().map(|p| (p, true))
            } else {
                diag::export_local().map(|p| (p, false))
            };
            *diag_export_once().lock().unwrap() = Some(result);
            diag_export_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn client_find_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn client_find_cancel_flag() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn client_find_job() {
    if client_find_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    client_find_cancel_flag().store(false, std::sync::atomic::Ordering::SeqCst);
    let _ = std::thread::Builder::new()
        .name("moklanjo-find-server".into())
        .spawn(|| {
            let found = net::discover_servers_cancellable(
                std::time::Duration::from_millis(3000),
                || client_find_cancel_flag().load(std::sync::atomic::Ordering::SeqCst),
            );
            *discovery_result().lock().unwrap() = Some(found);
            client_find_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn carrier_bill_report_once() -> &'static Mutex<Option<String>> {
    static SLOT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// How often Settings re-reads which devices are connected.
const LIVE_REFRESH_SECS: f64 = 8.0;

/// The screen is being kept on because this device serves others.
static SCREEN_KEPT_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The Server's books changed under this screen (a booking from a counter
/// device): today's figures and lists refresh on the next wake.
static BOOKS_CHANGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn start_auto_backups(db_path: PathBuf) {
    let mut slot = auto_backup_target().lock().unwrap();
    slot.0 = Some(db_path);
    if slot.1 {
        return;
    }
    slot.1 = true;
    let _ = std::thread::Builder::new()
        .name("moklanjo-backup".into())
        .spawn(|| {
            loop {
                let path = auto_backup_target().lock().unwrap().0.clone();
                if let Some(path) = path
                    && let Ok(store) = Store::open_path(&path)
                    && let Ok(Some(_)) = store.auto_backup_if_due()
                {
                    BACKUP_MADE.store(true, std::sync::atomic::Ordering::SeqCst);
                    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
                }
                std::thread::sleep(Duration::from_secs(600));
            }
        });
}

fn rail_resize_drag() -> &'static Mutex<Option<f64>> {
    static DRAG: OnceLock<Mutex<Option<f64>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// In-progress sidebar / bottom-bar drag-to-swap (peeked at App level,
/// same style as the rail resize drag). `rail` picks the surface, `slot`
/// is the position the drag started from, `hover` the position under the
/// pointer right now (drop target).
#[derive(Clone)]
struct NavDrag {
    rail: bool,
    slot: usize,
    start: DVec2,
    /// Latest pointer pos while dragging; edge auto-scroll reads it
    /// on frames where the pointer itself is standing still.
    last: DVec2,
    /// Window pos of the grabbed item's face when the drag began; the
    /// floating ghost chip tracks `origin + (pointer - start)`.
    origin: DVec2,
    /// Every slot's face rect in the bar's *content* coordinates,
    /// captured at pickup. Widget areas are only trustworthy while
    /// the layout is settled — during edge auto-scroll they lag the
    /// scrolling content — so drag-time hit tests use these frozen
    /// rects plus the live scroll offset instead.
    rects: Vec<Rect>,
    active: bool,
    hover: Option<usize>,
}

fn nav_drag() -> &'static Mutex<Option<NavDrag>> {
    static DRAG: OnceLock<Mutex<Option<NavDrag>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// Settings → Sidebar list drag-reordered; App syncs chrome when taken.
fn sidebar_order_dirty() -> &'static Mutex<bool> {
    static FLAG: OnceLock<Mutex<bool>> = OnceLock::new();
    FLAG.get_or_init(|| Mutex::new(false))
}

/// Settings → Sidebar list: same ghost-chip drag as the rail/bottom bar
/// (`pointer_sidebar_settings_drag`), not an in-list swap.
#[derive(Clone)]
struct SettingsNavDrag {
    slot: usize,
    start: DVec2,
    last: DVec2,
    origin: DVec2,
    rects: Vec<Rect>,
    active: bool,
    hover: Option<usize>,
}

fn settings_nav_drag() -> &'static Mutex<Option<SettingsNavDrag>> {
    static DRAG: OnceLock<Mutex<Option<SettingsNavDrag>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// Pending frame callback that keeps edge auto-scroll alive while a
/// nav drag is held against the bar's edge (see `nav_drag_autoscroll`).
fn nav_autoscroll_frame() -> &'static Mutex<Option<NextFrame>> {
    static FRAME: OnceLock<Mutex<Option<NextFrame>>> = OnceLock::new();
    FRAME.get_or_init(|| Mutex::new(None))
}

/// Font Awesome solid (`WorldFonts` icons member): chevron-right / chevron-down.
/// Noto Sans often has no ▸/▾, so those looked like missing icons.
const ACCORDION_MARK_CLOSED: &str = "\u{f054}";
const ACCORDION_MARK_OPEN: &str = "\u{f078}";

/// Books accordion open/close height animation (~220ms ease-out).
struct AccordionAnim {
    /// Visual progress 0 = collapsed, 1 = expanded.
    from: f64,
    to: f64,
    content_h: f64,
    started: std::time::Instant,
}

fn accordion_anims() -> &'static Mutex<std::collections::HashMap<String, AccordionAnim>> {
    static ANIMS: OnceLock<Mutex<std::collections::HashMap<String, AccordionAnim>>> = OnceLock::new();
    ANIMS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn accordion_frame() -> &'static Mutex<Option<NextFrame>> {
    static FRAME: OnceLock<Mutex<Option<NextFrame>>> = OnceLock::new();
    FRAME.get_or_init(|| Mutex::new(None))
}

const ACCORDION_ANIM_MS: f64 = 220.0;

fn accordion_ease(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn accordion_estimate_h(key: &str) -> f64 {
    match key {
        "today" => 168.0,
        "bookings" | "bills" => 72.0,
        "payments" => 108.0,
        "outstanding" => 72.0,
        "bank" => 108.0,
        _ => 100.0,
    }
}

/// Set when a drag (not a click) ends on a nav item, so the Button's
/// resulting `clicked` action is ignored once instead of navigating.
fn nav_suppress_click() -> &'static Mutex<bool> {
    static SUPPRESS: OnceLock<Mutex<bool>> = OnceLock::new();
    SUPPRESS.get_or_init(|| Mutex::new(false))
}

/// Side rail width floor while the user drags the divider.
const RAIL_WIDTH_MIN: f64 = 148.0;
/// Default rail width (fits short chrome labels without ellipsis).
const RAIL_WIDTH_DEFAULT: f64 = 220.0;
/// Main column floor while resizing the rail (css px).
const MAIN_CONTENT_MIN: f64 = 360.0;

/// Linux often emits both MouseButton::BACK and XF86Back for one click.
fn nav_gesture_ok() -> bool {
    static LAST: OnceLock<Mutex<Option<std::time::Instant>>> = OnceLock::new();
    let last = LAST.get_or_init(|| Mutex::new(None));
    let mut guard = last.lock().unwrap();
    let now = std::time::Instant::now();
    if let Some(prev) = *guard
        && now.duration_since(prev) < Duration::from_millis(140)
    {
        return false;
    }
    *guard = Some(now);
    true
}

/// Void, refund and amend follow each person's rights (Team → Rights). A
/// cashier alone at the counter should not have to hand the device over or log
/// the manager in mid-queue, so when their right is "With approval" an
/// Owner/Manager PIN typed on the spot stands in: the approver becomes the
/// recorded actor, and the note says who asked for it.
///
/// Returns the actor to record and the reason to store. When the actor may
/// already do this, nothing changes and no PIN is read.
fn approve_for(
    store: &Store,
    actor: &StaffMember,
    right: domain::Right,
    pin: &str,
    reason: &str,
) -> Result<(StaffMember, String), String> {
    match store.check_right(actor, right, Some(pin))? {
        None => Ok((actor.clone(), reason.trim().to_string())),
        Some(approver) => Ok((
            approver.clone(),
            format!(
                "{} · approved by {} for {}",
                reason.trim(),
                approver.name,
                actor.name
            ),
        )),
    }
}

/// Whose saved device sign-in sends a job: a staff id, or "owner".
fn lan_key(member: &StaffMember) -> String {
    if member.is_owner() {
        "owner".into()
    } else {
        member.id.clone()
    }
}

fn store_client_paired(store: Option<&Store>) -> bool {
    store.and_then(|s| s.client_paired().ok()).unwrap_or(false)
}

/// After soft-keyboard DidHide, WindowGotFocus may restore the TextInput.
/// Suppress that for one event cycle so the keyboard stays down.
fn ime_suppress_refocus() -> &'static Mutex<bool> {
    static FLAG: OnceLock<Mutex<bool>> = OnceLock::new();
    FLAG.get_or_init(|| Mutex::new(false))
}

fn app_state() -> &'static Mutex<AppState> {
    static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(AppState::boot()))
}

/// Locale string with the lock released before return.
///
/// Never write `set_status(cx, app_state().lock().unwrap().t(...))` — the
/// temporary MutexGuard lives for the whole statement, so `set_status` (which
/// locks again) deadlocks the UI thread. gdb: Bill place tap →
/// `match_event` → `set_status` → `set_status_line_only` stuck on
/// `app_state::STATE` (2026-11-03).
fn app_t(key: &str) -> String {
    app_state().lock().unwrap().t(key)
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
/// "Find the shop's server" runs on a worker thread; its answer waits here
/// until the UI wakes (`Event::Signal`).
fn discovery_result() -> &'static Mutex<Option<Vec<net::FoundServer>>> {
    static SLOT: OnceLock<Mutex<Option<Vec<net::FoundServer>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

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
        .name("khaynjo-firewall".into())
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
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok().map(|n| n.trim().to_string()))
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Device".into())
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
        .name("khaynjo-backup-once".into())
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

/// Shared "work in progress" line for long jobs (restore, CSV, shelf photos…).
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
    // Ok((backup_path, before_restore_path))
    static SLOT: OnceLock<Mutex<Option<Result<(PathBuf, PathBuf), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// VACUUM + SQLite restore off the UI thread. Caller must drop `st.store` first.
fn restore_job(db_path: PathBuf, backup: PathBuf, before: PathBuf) {
    if restore_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    restore_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Restoring backup…");
    let _ = std::thread::Builder::new()
        .name("khaynjo-restore".into())
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
    // Ok((file, suggested save name))
    static SLOT: OnceLock<Mutex<Option<Result<(PathBuf, String), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Android "Save a backup copy": VACUUM off UI, then Signal opens the save dialog.
fn backup_copy_job(db_path: PathBuf, dest: PathBuf, suggested_name: String) {
    if backup_copy_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    backup_copy_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Making backup copy…");
    let _ = std::thread::Builder::new()
        .name("khaynjo-backup-copy".into())
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

fn csv_import_job(db_path: PathBuf, path: PathBuf, parties: bool) {
    if csv_import_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    csv_import_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status(if parties {
        "Importing parties…"
    } else {
        "Importing items…"
    });
    let _ = std::thread::Builder::new()
        .name("khaynjo-csv-import".into())
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
                let (added, skipped, problems) = if parties {
                    store.import_parties_csv(&text)?
                } else {
                    store.import_items_csv(&text)?
                };
                let mut msg = format!(
                    "Imported {added} new, skipped {skipped} (already there or unusable)"
                );
                if let Some(p) = problems.first() {
                    msg.push_str(&format!(" · first problem: {p}"));
                }
                Ok(msg)
            })();
            *csv_import_once().lock().unwrap() = Some(result);
            csv_import_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

fn shelf_mat_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn shelf_mat_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn shelf_mat_once() -> &'static Mutex<Option<Result<usize, String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<usize, String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn shelf_mat_progress_tick(done: usize, total: usize) {
    if total == 0 {
        set_books_work_status("Preparing menu photos…");
    } else {
        *books_work_status().lock().unwrap() =
            Some(format!("Preparing menu photos… {done}/{total}"));
    }
    static LAST: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let prev = LAST.load(std::sync::atomic::Ordering::Relaxed);
    if done == 0 || done == total || done.saturating_sub(prev) >= 8 {
        LAST.store(done, std::sync::atomic::Ordering::Relaxed);
        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
    }
}

fn shelf_mat_cancel_poll() -> bool {
    shelf_mat_cancel().load(std::sync::atomic::Ordering::SeqCst)
}

/// Materialise `shelf:…` photos into `item_images` off the UI thread.
fn shelf_materialize_job(db_path: PathBuf) {
    if shelf_mat_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    shelf_mat_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Preparing menu photos…");
    store::set_import_progress_notify(Some(shelf_mat_progress_tick));
    store::set_import_cancel_check(Some(shelf_mat_cancel_poll));
    let _ = std::thread::Builder::new()
        .name("khaynjo-shelf-mat".into())
        .spawn(move || {
            let result = Store::open_path(&db_path).and_then(|s| s.materialize_shelf_photos());
            store::set_import_progress_notify(None);
            store::set_import_cancel_check(None);
            *shelf_mat_once().lock().unwrap() = Some(result);
            shelf_mat_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

/// Menu template / shelf catalogue import — never on the UI thread (webp copy
/// of hundreds of dishes freezes the window for a long time).
#[derive(Clone, Debug)]
enum MenuImportKind {
    Shelf,
    Client { id: String, title: String },
}

struct MenuImportOutcome {
    kind: MenuImportKind,
    result: Result<store::CatalogueImport, String>,
}

fn menu_import_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn menu_import_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn menu_import_progress() -> &'static Mutex<(usize, usize)> {
    static SLOT: OnceLock<Mutex<(usize, usize)>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new((0, 0)))
}

fn menu_import_once() -> &'static Mutex<Option<MenuImportOutcome>> {
    static SLOT: OnceLock<Mutex<Option<MenuImportOutcome>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn menu_import_running() -> &'static Mutex<Option<MenuImportKind>> {
    static SLOT: OnceLock<Mutex<Option<MenuImportKind>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn menu_import_progress_tick(done: usize, total: usize) {
    *menu_import_progress().lock().unwrap() = (done, total);
    *books_work_status().lock().unwrap() = Some(format!("Importing… {done}/{total}"));
    // Throttle wakes so a 1,400-dish shelf import does not flood the UI queue.
    static LAST: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let prev = LAST.load(std::sync::atomic::Ordering::Relaxed);
    if done == 0 || done == total || done.saturating_sub(prev) >= 3 {
        LAST.store(done, std::sync::atomic::Ordering::Relaxed);
        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
    }
}

fn menu_import_cancel_poll() -> bool {
    menu_import_cancel().load(std::sync::atomic::Ordering::SeqCst)
}

fn menu_import_job(db_path: PathBuf, kind: MenuImportKind) {
    if menu_import_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    menu_import_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    // Known total before the worker starts so the progress button paints 0/N
    // on the same click frame (not after the first encode).
    let known_total = match &kind {
        MenuImportKind::Shelf => menu_images::shelf().len().max(1),
        MenuImportKind::Client { id, .. } => menu_templates::find(id)
            .map(|t| {
                t.categories
                    .iter()
                    .map(|c| c.items.len().saturating_add(1))
                    .sum::<usize>()
                    .max(1)
            })
            .unwrap_or(1),
    };
    *menu_import_progress().lock().unwrap() = (0, known_total);
    *menu_import_once().lock().unwrap() = None;
    *menu_import_running().lock().unwrap() = Some(kind.clone());
    set_books_work_status(&format!("Importing… 0/{known_total}"));
    store::set_import_progress_notify(Some(menu_import_progress_tick));
    store::set_import_cancel_check(Some(menu_import_cancel_poll));
    let _ = std::thread::Builder::new()
        .name("khaynjo-menu-import".into())
        .spawn(move || {
            let result = (|| -> Result<store::CatalogueImport, String> {
                let store = Store::open_path(&db_path)?;
                match &kind {
                    MenuImportKind::Shelf => store.import_catalogue(menu_images::shelf()),
                    MenuImportKind::Client { id, .. } => {
                        let tmpl = menu_templates::find(id)
                            .ok_or_else(|| format!("Unknown menu template '{id}'"))?;
                        store.import_menu_template(tmpl)
                    }
                }
            })();
            store::set_import_progress_notify(None);
            store::set_import_cancel_check(None);
            *menu_import_once().lock().unwrap() = Some(MenuImportOutcome { kind, result });
            *menu_import_running().lock().unwrap() = None;
            menu_import_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

/// Cancel whichever books / import job is currently busy (best-effort for VACUUM).
fn request_books_job_cancel() {
    if menu_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
        menu_import_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if shelf_mat_busy().load(std::sync::atomic::Ordering::SeqCst) {
        shelf_mat_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if csv_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
        csv_import_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if backup_busy().load(std::sync::atomic::Ordering::SeqCst) {
        backup_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if restore_busy().load(std::sync::atomic::Ordering::SeqCst) {
        restore_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if backup_copy_busy().load(std::sync::atomic::Ordering::SeqCst) {
        backup_copy_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if aggregator_import_busy().load(std::sync::atomic::Ordering::SeqCst) {
        aggregator_import_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if diag_export_busy().load(std::sync::atomic::Ordering::SeqCst) {
        diag_export_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

fn printer_add_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn printer_add_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn printer_add_status() -> &'static Mutex<Option<String>> {
    static SLOT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn printer_add_once() -> &'static Mutex<Option<Result<(), String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<(), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn printer_add_result_draft() -> &'static Mutex<Option<print_share::DevicePrinters>> {
    static SLOT: OnceLock<Mutex<Option<print_share::DevicePrinters>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn client_find_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn client_find_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn aggregator_import_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn aggregator_import_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn aggregator_import_once() -> &'static Mutex<Option<Result<String, String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<String, String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn aggregator_import_job(db_path: PathBuf) {
    if aggregator_import_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    aggregator_import_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Importing aggregator file…");
    let _ = std::thread::Builder::new()
        .name("khaynjo-agg-import".into())
        .spawn(move || {
            let result = (|| -> Result<String, String> {
                if aggregator_import_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let store = Store::open_path(&db_path)?;
                if aggregator_import_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let n = store.import_aggregator_orders_drop_file()?;
                Ok(format!("Imported {n} aggregator order(s) into the inbox"))
            })();
            *aggregator_import_once().lock().unwrap() = Some(result);
            aggregator_import_busy().store(false, std::sync::atomic::Ordering::SeqCst);
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

fn diag_export_for_send() -> &'static std::sync::atomic::AtomicBool {
    static FOR_SEND: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &FOR_SEND
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
    diag_export_for_send().store(for_send, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status(if for_send {
        "Creating share package…"
    } else {
        "Exporting diagnostics…"
    });
    let _ = std::thread::Builder::new()
        .name("khaynjo-diag-export".into())
        .spawn(move || {
            let result = (|| -> Result<(diag::ExportPaths, bool), String> {
                if diag_export_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                let paths = if for_send {
                    diag::export_for_send()?
                } else {
                    diag::export_local()?
                };
                if diag_export_cancel().load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Cancelled".into());
                }
                Ok((paths, for_send))
            })();
            *diag_export_once().lock().unwrap() = Some(result);
            diag_export_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
}

/// The screen is being kept on because this device serves others.
static SCREEN_KEPT_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The Server's books changed under this screen (a device's KOT, payment,
/// table move): the floor and kitchen refresh on the next wake, not in 8 s.
static BOOKS_CHANGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Books / Inventory / Reports facet text needs a fresh SQL pass. Cleared by
/// `sync_chrome` after it refreshes — avoids re-running `period_books` on every
/// Signal (thumbs / LAN / printer) which froze Android while those tabs show.
static FACET_DIRTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn start_auto_backups(db_path: PathBuf) {
    let mut slot = auto_backup_target().lock().unwrap();
    slot.0 = Some(db_path);
    if slot.1 {
        return;
    }
    slot.1 = true;
    let _ = std::thread::Builder::new()
        .name("khaynjo-backup".into())
        .spawn(|| loop {
            let path = auto_backup_target().lock().unwrap().0.clone();
            if let Some(path) = path
                && let Ok(store) = Store::open_path(&path)
                && let Ok(Some(_)) = store.auto_backup_if_due()
            {
                BACKUP_MADE.store(true, std::sync::atomic::Ordering::SeqCst);
                makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
            }
            std::thread::sleep(Duration::from_secs(600));
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
    /// Touch finger that owns this drag (`None` = mouse / desktop).
    /// Android only sends `TouchUpdate`, so without this the drag never arms.
    finger: Option<u64>,
}

fn nav_drag() -> &'static Mutex<Option<NavDrag>> {
    static DRAG: OnceLock<Mutex<Option<NavDrag>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// Floor area ChipStrip drag-to-swap (same contract as [`NavDrag`]:
/// tap selects; press + move swaps and persists `ui_floor_area_order`).
#[derive(Clone)]
struct FloorAreaDrag {
    slot: usize,
    start: DVec2,
    last: DVec2,
    origin: DVec2,
    rects: Vec<Rect>,
    active: bool,
    hover: Option<usize>,
    finger: Option<u64>,
}

fn floor_area_drag() -> &'static Mutex<Option<FloorAreaDrag>> {
    static DRAG: OnceLock<Mutex<Option<FloorAreaDrag>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

fn floor_area_autoscroll_frame() -> &'static Mutex<Option<NextFrame>> {
    static FRAME: OnceLock<Mutex<Option<NextFrame>>> = OnceLock::new();
    FRAME.get_or_init(|| Mutex::new(None))
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
    /// Touch finger that owns this drag (`None` = mouse / desktop).
    finger: Option<u64>,
}

fn settings_nav_drag() -> &'static Mutex<Option<SettingsNavDrag>> {
    static DRAG: OnceLock<Mutex<Option<SettingsNavDrag>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// Settings → printers: drag-to-swap printers on the focused place.
#[derive(Clone)]
struct RoutePrinterDrag {
    slot: usize,
    start: DVec2,
    last: DVec2,
    origin: DVec2,
    rects: Vec<Rect>,
    active: bool,
    hover: Option<usize>,
    finger: Option<u64>,
    label: String,
}

fn route_printer_drag() -> &'static Mutex<Option<RoutePrinterDrag>> {
    static DRAG: OnceLock<Mutex<Option<RoutePrinterDrag>>> = OnceLock::new();
    DRAG.get_or_init(|| Mutex::new(None))
}

/// After a drag-swap, suppress the following tap-to-remove on that list.
fn route_printer_tap_suppress() -> &'static Mutex<bool> {
    static FLAG: OnceLock<Mutex<bool>> = OnceLock::new();
    FLAG.get_or_init(|| Mutex::new(false))
}

#[allow(dead_code)] // Assigned list is read-only; remove via checklist.
fn route_printer_tap_suppressed() -> bool {
    let mut g = route_printer_tap_suppress().lock().unwrap();
    let v = *g;
    *g = false;
    v
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

/// Void and refund are Owner/Manager work. A cashier alone at
/// the counter should not have to hand the device over or log the manager in
/// mid-queue, so an Owner/Manager PIN typed on the spot stands in: the approver
/// becomes the recorded actor, and the note says who asked for it.
///
/// Returns the actor to record and the reason to store. When the actor may
/// already do this, nothing changes and no PIN is read.
fn approve_for(
    store: &Store,
    actor: &StaffMember,
    pin: &str,
    reason: &str,
) -> Result<(StaffMember, String), String> {
    if actor.role.can_void_refund() {
        return Ok((actor.clone(), reason.trim().to_string()));
    }
    let approver = store.verify_approval_pin(pin)?;
    Ok((
        approver.clone(),
        format!(
            "{} · approved by {} for {}",
            reason.trim(),
            approver.name,
            actor.name
        ),
    ))
}

/// A device's KOT line still waiting for the Server: `kot_id` groups it
/// under "Waiting to send"; `running_line_id` is this prefix + the job id.
const QUEUED_KOT: &str = "queued";
const QUEUED_LINE: &str = "queued:";

/// A table action done on this device (with bill text when one printed),
/// or sent to the Server — its answer shows when it comes back.
enum Acted {
    Here(Option<String>),
    Sent,
}

/// Whose saved phone sign-in sends a job: a staff id, or "owner".
fn lan_key(member: &StaffMember) -> String {
    if member.is_owner() { "owner".into() } else { member.id.clone() }
}

fn store_client_paired(store: Option<&Store>) -> bool {
    store
        .and_then(|s| s.client_paired().ok())
        .unwrap_or(false)
}

/// After soft-keyboard DidHide, WindowGotFocus may restore the TextInput.
/// Suppress that for one event cycle so the keyboard stays down.
fn ime_suppress_refocus() -> &'static Mutex<bool> {
    static FLAG: OnceLock<Mutex<bool>> = OnceLock::new();
    FLAG.get_or_init(|| Mutex::new(false))
}

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
        .name("kirano-firewall".into())
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
    /// HTTP pair reply — books apply continues on a worker (`PairApplied`).
    /// The `u64` is this Pair now attempt so a late reply cannot apply over Cancel
    /// or a newer tap.
    Paired(u64, Result<(net::Link, net::SyncSnapshot), net::LanError>),
    /// Snapshot written off the UI thread (venue label for the status line).
    PairApplied(u64, Result<String, String>),
    /// Books reloaded off the UI thread; the screen may paint Lock.
    PairBooksReady(u64, Result<String, String>),
    /// Cancel, or this attempt was replaced. The UI ignores a stale id.
    PairCancelled(u64),
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
        .name("kirano-backup-once".into())
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
    // Wake the UI so a progress line still on screen is taken down now
    // (`App::clear_stale_work_status`), not on some later event.
    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
}

/// The job progress text last painted on the status line (`App::show_work_status`).
/// When the job ends without a result message the line still showed it — the
/// "Preparing desk…" that stayed on DeviceSetup / Lock long after the desk opened.
fn work_status_shown() -> &'static Mutex<Option<String>> {
    static SLOT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
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
        .name("kirano-restore".into())
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
        .name("kirano-backup-copy".into())
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

/// Result of writing the backup copy where the owner picked
/// (`backup_copy_write_job`): `Err("Cancelled")` when Cancel was tapped.
fn backup_copy_write_once() -> &'static Mutex<Option<Result<(), String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<(), String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// "Save a backup copy", second half: copy the finished file to the picked
/// place — a content:// URI on Android (the whole file through JNI, safe on a
/// worker since `android_jni_detach_on_exit`) or a plain path — off the UI
/// thread. Same busy flag, so the same Progress + Cancel chrome shows.
fn backup_copy_write_job(file: PathBuf, target: String) {
    if backup_copy_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        let _ = std::fs::remove_file(&file);
        *backup_copy_write_once().lock().unwrap() = Some(Err("Busy".into()));
        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        return;
    }
    backup_copy_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status("Making backup copy…");
    let spawned = std::thread::Builder::new()
        .name("backup-copy-save".into())
        .spawn(move || {
            let cancelled = || backup_copy_cancel().load(std::sync::atomic::Ordering::SeqCst);
            let result = if cancelled() {
                Err("Cancelled".to_string())
            } else {
                match copy_file_to_picked_target(&file, &target) {
                    Ok(()) if cancelled() => Err("Cancelled".to_string()),
                    other => other,
                }
            };
            let _ = std::fs::remove_file(&file);
            *backup_copy_write_once().lock().unwrap() = Some(result);
            backup_copy_busy().store(false, std::sync::atomic::Ordering::SeqCst);
            clear_books_work_status();
            makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
        });
    if spawned.is_err() {
        backup_copy_busy().store(false, std::sync::atomic::Ordering::SeqCst);
        clear_books_work_status();
        *backup_copy_write_once().lock().unwrap() = Some(Err("Could not start the copy".into()));
        makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
    }
}

/// Copy `file` to what a save dialog returned: a content:// URI (Android
/// picker) or a filesystem path. Runs on a worker.
fn copy_file_to_picked_target(file: &std::path::Path, target: &str) -> Result<(), String> {
    if target.starts_with("content://") {
        let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
        #[cfg(target_os = "android")]
        {
            let ok = unsafe {
                makepad_widgets::makepad_platform::os::linux::android::android_jni::to_java_write_content_uri(
                    target.to_owned(),
                    &bytes,
                )
            };
            return if ok { Ok(()) } else { Err("content:// write failed".into()) };
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = bytes;
            return Err("content:// targets exist on Android only".into());
        }
    }
    std::fs::copy(file, target)
        .map(|_| ())
        .map_err(|e| e.to_string())
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
        .name("kirano-csv-import".into())
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
        .name("kirano-shelf-mat".into())
        .spawn(move || {
            if let Ok(base) = store::data_dir() {
                menu_images::ensure_android_menu_assets(&base);
            }
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
    /// Post-create DeviceSetup offer — same menu import as Client, different chrome.
    AppTemplate { id: String, title: String },
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
        MenuImportKind::Client { id, .. } | MenuImportKind::AppTemplate { id, .. } => {
            menu_templates::find(id)
                .map(|t| {
                    t.categories
                        .iter()
                        .map(|c| c.items.len().saturating_add(1))
                        .sum::<usize>()
                        .max(1)
                })
                .unwrap_or(1)
        }
    };
    *menu_import_progress().lock().unwrap() = (0, known_total);
    *menu_import_once().lock().unwrap() = None;
    *menu_import_running().lock().unwrap() = Some(kind.clone());
    set_books_work_status(&format!("Importing… 0/{known_total}"));
    store::set_import_progress_notify(Some(menu_import_progress_tick));
    store::set_import_cancel_check(Some(menu_import_cancel_poll));
    let _ = std::thread::Builder::new()
        .name("kirano-menu-import".into())
        .spawn(move || {
            let result = (|| -> Result<store::CatalogueImport, String> {
                if let Ok(base) = store::data_dir() {
                    menu_images::ensure_android_menu_assets(&base);
                }
                let store = Store::open_path(&db_path)?;
                match &kind {
                    MenuImportKind::Shelf => store.import_catalogue(menu_images::shelf()),
                    MenuImportKind::Client { id, .. } => {
                        let tmpl = menu_templates::find(id)
                            .ok_or_else(|| format!("Unknown menu template '{id}'"))?;
                        store.import_menu_template(tmpl)
                    }
                    MenuImportKind::AppTemplate { .. } => store.apply_rajdhani_app_template(),
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

/// Pair now is in flight (HTTP, snapshot apply, or opening the till).
fn client_pair_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn client_pair_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

/// Bumped on every Pair now so a late worker cannot publish over a newer tap.
fn client_pair_gen() -> &'static std::sync::atomic::AtomicU64 {
    static GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    &GEN
}

fn client_pair_progress() -> &'static Mutex<String> {
    static SLOT: OnceLock<Mutex<String>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(String::new()))
}

fn client_pair_set_progress(status: &str) {
    *client_pair_progress().lock().unwrap() = status.to_string();
}

/// True while this attempt is still the one on screen and Cancel has not been tapped.
fn client_pair_current(attempt: u64) -> bool {
    client_pair_gen().load(std::sync::atomic::Ordering::SeqCst) == attempt
        && !client_pair_cancel().load(std::sync::atomic::Ordering::SeqCst)
}

fn client_pair_begin(status: &str) -> u64 {
    let attempt = client_pair_gen().fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    client_pair_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    net::set_pair_abandoned(false);
    client_pair_set_progress(status);
    client_pair_busy().store(true, std::sync::atomic::Ordering::SeqCst);
    attempt
}

fn client_pair_clear() {
    client_pair_busy().store(false, std::sync::atomic::Ordering::SeqCst);
    client_pair_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    net::set_pair_abandoned(false);
    client_pair_progress().lock().unwrap().clear();
}

/// Publish only if this attempt is still current. Holds `lan_once` across the
/// generation check so a newer Pair now cannot be overwritten.
fn publish_pair(attempt: u64, msg: LanOnce) {
    {
        let mut slot = lan_once().lock().unwrap();
        if client_pair_gen().load(std::sync::atomic::Ordering::SeqCst) != attempt {
            return;
        }
        *slot = Some(msg);
    }
    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
}

/// DeviceSetup role open (Client / new Server / existing Server) off the UI
/// thread — first SQLite migrate can take a long time on Android.
fn role_open_busy() -> &'static std::sync::atomic::AtomicBool {
    static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &BUSY
}

fn role_open_cancel() -> &'static std::sync::atomic::AtomicBool {
    static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    &CANCEL
}

fn request_role_open_cancel() {
    role_open_cancel().store(true, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status(&app_t("cancelling"));
    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
}

enum RoleOpenReady {
    Client(Store),
    ServerNew(Store),
    /// `switching_in` = previous role was not Server (extra-tills reset).
    ServerExisting { store: Store, switching_in: bool },
}

fn role_open_once() -> &'static Mutex<Option<Result<RoleOpenReady, String>>> {
    static SLOT: OnceLock<Mutex<Option<Result<RoleOpenReady, String>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn role_open_start(status: &str) -> bool {
    if role_open_busy().swap(true, std::sync::atomic::Ordering::SeqCst) {
        return false;
    }
    role_open_cancel().store(false, std::sync::atomic::Ordering::SeqCst);
    set_books_work_status(status);
    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
    true
}

fn role_open_finish(result: Result<RoleOpenReady, String>) {
    let ok = result.is_ok();
    *role_open_once().lock().unwrap() = Some(result);
    if ok {
        // Keep busy chrome until finish_* drops the old books + reload_all +
        // sync_chrome (that can take a long time on a phone — clearing busy
        // here left a silent freeze with Server/Client still on screen).
        set_books_work_status(&app_t("preparing_desk"));
    } else {
        role_open_busy().store(false, std::sync::atomic::Ordering::SeqCst);
        clear_books_work_status();
    }
    makepad_widgets::makepad_platform::thread::SignalToUI::set_ui_signal();
}

/// Drop a previous books file off the UI thread — SQLite/WAL close of a large
/// shop DB can stall the main thread for a long time on Android.
fn drop_store_off_ui(old: Option<Store>) {
    let Some(store) = old else {
        return;
    };
    let _ = std::thread::Builder::new()
        .name("kirano-drop-store".into())
        .spawn(move || drop(store));
}

fn end_role_open_busy() {
    role_open_busy().store(false, std::sync::atomic::Ordering::SeqCst);
    clear_books_work_status();
}

fn role_cancelled() -> bool {
    role_open_cancel().load(std::sync::atomic::Ordering::SeqCst)
}

/// Open the client terminal store on a worker.
fn client_mode_job(base: PathBuf) {
    if !role_open_start(&app_t("opening_client")) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("kirano-client-mode".into())
        .spawn(move || {
            net::stop_server();
            let result = (|| -> Result<RoleOpenReady, String> {
                let store = local_servers::open_client_store(&base)?;
                if role_cancelled() {
                    return Err("Cancelled".into());
                }
                store.prepare_as_client_terminal()?;
                store.set_device_role("client")?;
                if role_cancelled() {
                    return Err("Cancelled".into());
                }
                Ok(RoleOpenReady::Client(store))
            })();
            role_open_finish(result);
        });
}

/// Create a new local server books file on a worker.
fn server_new_job(base: PathBuf, name: String) {
    if !role_open_start(&app_t("creating_server")) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("kirano-server-new".into())
        .spawn(move || {
            net::stop_server();
            let result = (|| -> Result<RoleOpenReady, String> {
                // Unpack APK menu shelf/templates into filesDir before offer/import.
                menu_images::ensure_android_menu_assets(&base);
                let (_info, store) = local_servers::create_local_server(&base, &name)?;
                if role_cancelled() {
                    return Err("Cancelled".into());
                }
                Ok(RoleOpenReady::ServerNew(store))
            })();
            role_open_finish(result);
        });
}

/// Open an existing local server on a worker.
fn server_existing_job(base: PathBuf, server_id: String, switching_in: bool) {
    if !role_open_start(&app_t("opening_server")) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("kirano-server-open".into())
        .spawn(move || {
            net::stop_server();
            let result = (|| -> Result<RoleOpenReady, String> {
                menu_images::ensure_android_menu_assets(&base);
                let store = local_servers::open_server(&base, &server_id)?;
                if role_cancelled() {
                    return Err("Cancelled".into());
                }
                Ok(RoleOpenReady::ServerExisting {
                    store,
                    switching_in,
                })
            })();
            role_open_finish(result);
        });
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
        .name("kirano-agg-import".into())
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
        .name("kirano-diag-export".into())
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
        .name("kirano-backup".into())
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

fn menu_pick_viewport_w() -> &'static Mutex<f64> {
    static W: OnceLock<Mutex<f64>> = OnceLock::new();
    W.get_or_init(|| Mutex::new(0.0))
}

fn pending_menu_pick_tab() -> &'static Mutex<Option<usize>> {
    static P: OnceLock<Mutex<Option<usize>>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(None))
}

/// Horizontal pager stepped scroll (one “page” per category jump).
struct MenuPickScrollAnim {
    step_x: Vec<f64>,
    step_i: usize,
    step_start: std::time::Instant,
    step_duration: f64,
}

fn menu_pick_scroll_anim() -> &'static Mutex<Option<MenuPickScrollAnim>> {
    static A: OnceLock<Mutex<Option<MenuPickScrollAnim>>> = OnceLock::new();
    A.get_or_init(|| Mutex::new(None))
}

fn menu_pick_scroll_anim_frame() -> &'static Mutex<Option<NextFrame>> {
    static F: OnceLock<Mutex<Option<NextFrame>>> = OnceLock::new();
    F.get_or_init(|| Mutex::new(None))
}

/// After DeviceSetup role open: paint ClientPair on this frame, run full
/// `sync_chrome` on the next so Cancel stays live during the transition.
fn deferred_role_open_sync_frame() -> &'static Mutex<Option<NextFrame>> {
    static F: OnceLock<Mutex<Option<NextFrame>>> = OnceLock::new();
    F.get_or_init(|| Mutex::new(None))
}

fn menu_pick_scroll_programmatic() -> bool {
    menu_pick_scroll_anim().lock().unwrap().is_some()
}

fn menu_pick_tab_keys(st: &AppState) -> Vec<Option<String>> {
    let cats = st.menu_categories();
    let mut keys = vec![
        None,
        Some(MENU_FREQUENT.to_string()),
        Some(MENU_ADDED.to_string()),
    ];
    keys.extend(cats.into_iter().map(|c| Some(c.id.clone())));
    keys
}

fn menu_pick_tab_index(keys: &[Option<String>], category: &Option<String>) -> usize {
    keys.iter().position(|k| k == category).unwrap_or(0)
}

fn queue_menu_pick_tab_from_scroll(idx: usize) {
    if menu_pick_scroll_programmatic() {
        return;
    }
    let st = app_state().lock().unwrap();
    if !st.menu_pick_open || !st.menu_filter.trim().is_empty() {
        return;
    }
    let keys = menu_pick_tab_keys(&st);
    if idx >= keys.len() {
        return;
    }
    let cat = keys.get(idx).cloned().flatten();
    if st.menu_category == cat {
        return;
    }
    drop(st);
    *pending_menu_pick_tab().lock().unwrap() = Some(idx);
}

/// Settings → Sidebar list reordered (↑↓); App syncs chrome when taken.

/// Settings → Sidebar list reordered (↑↓); App syncs chrome when taken.
fn sidebar_order_dirty() -> &'static Mutex<bool> {
    static FLAG: OnceLock<Mutex<bool>> = OnceLock::new();
    FLAG.get_or_init(|| Mutex::new(false))
}

/// Font Awesome solid (`WorldFonts` icons member): chevron-right / chevron-down.
/// Noto Sans often has no ▸/▾, so those looked like missing icons.
const ACCORDION_MARK_CLOSED: &str = "\u{f054}";
const ACCORDION_MARK_OPEN: &str = "\u{f078}";

/// Section / list open-close height animation (~220ms ease-out).
/// `from`/`to` are visual progress 0 = collapsed, 1 = expanded.
#[derive(Clone)]
struct AccordionAnim {
    from: f64,
    to: f64,
    content_h: f64,
    started: std::time::Instant,
}

/// Height tweens for SuggestList / CheckList / pick lists (keyed by field or widget id).
fn list_height_anims() -> &'static Mutex<std::collections::HashMap<String, AccordionAnim>> {
    static ANIMS: OnceLock<Mutex<std::collections::HashMap<String, AccordionAnim>>> = OnceLock::new();
    ANIMS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

/// Keys that were visually open last frame (so close can tween from full height).
fn list_height_drawn() -> &'static Mutex<std::collections::HashSet<String>> {
    static DRAWN: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    DRAWN.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

/// Drive open/close height for a dropdown list. Returns `(height, should_draw)`.
fn list_height_tween(key: &str, logic_open: bool, full_h: f64) -> (f64, bool) {
    let now = std::time::Instant::now();
    let mut anims = list_height_anims().lock().unwrap();
    let mut drawn = list_height_drawn().lock().unwrap();
    let was_drawn = drawn.contains(key);
    let current_p = anims
        .get(key)
        .map(|a| accordion_anim_progress(a, now).0);
    let want_to = if logic_open { 1.0 } else { 0.0 };

    if logic_open && !was_drawn {
        let from = current_p.unwrap_or(0.0);
        anims.insert(
            key.to_string(),
            AccordionAnim {
                from,
                to: 1.0,
                content_h: full_h.max(1.0),
                started: now,
            },
        );
        drawn.insert(key.to_string());
    } else if !logic_open && was_drawn {
        let from = current_p.unwrap_or(1.0);
        let content_h = anims
            .get(key)
            .map(|a| a.content_h)
            .unwrap_or(full_h)
            .max(1.0);
        anims.insert(
            key.to_string(),
            AccordionAnim {
                from,
                to: 0.0,
                content_h,
                started: now,
            },
        );
        // Drop drawn so we do not re-start close every frame; anim keeps drawing.
        drawn.remove(key);
    } else if logic_open {
        drawn.insert(key.to_string());
        if let Some(a) = anims.get_mut(key) {
            if full_h > 1.0 {
                a.content_h = full_h;
            }
            // Reverse a mid-close if the list opened again.
            if a.to <= 0.0 {
                let from = current_p.unwrap_or(0.0);
                a.from = from;
                a.to = 1.0;
                a.started = now;
            }
        }
    } else if anims.get(key).map(|a| a.to > 0.5).unwrap_or(false) {
        // Logic flipped closed while opening — reverse from current progress.
        let from = current_p.unwrap_or(0.0);
        let content_h = anims
            .get(key)
            .map(|a| a.content_h)
            .unwrap_or(full_h.max(1.0));
        anims.insert(
            key.to_string(),
            AccordionAnim {
                from,
                to: 0.0,
                content_h,
                started: now,
            },
        );
    }

    if let Some(a) = anims.get(key).cloned() {
        let (p, done) = accordion_anim_progress(&a, now);
        let h = a.content_h * p;
        if done {
            anims.remove(key);
            if want_to <= 0.0 {
                drawn.remove(key);
                return (0.0, false);
            }
            return (a.content_h, true);
        }
        return (h, true);
    }
    if logic_open {
        (full_h.max(0.0), full_h > 0.5)
    } else {
        drawn.remove(key);
        (0.0, false)
    }
}

/// True when any dropdown height tween is still running (keeps NextFrame alive).
fn list_height_anims_live() -> bool {
    let now = std::time::Instant::now();
    let anims = list_height_anims().lock().unwrap();
    anims
        .values()
        .any(|a| !accordion_anim_progress(a, now).1)
}

fn accordion_anims() -> &'static Mutex<std::collections::HashMap<String, AccordionAnim>> {
    static ANIMS: OnceLock<Mutex<std::collections::HashMap<String, AccordionAnim>>> = OnceLock::new();
    ANIMS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn accordion_frame() -> &'static Mutex<Option<NextFrame>> {
    static FRAME: OnceLock<Mutex<Option<NextFrame>>> = OnceLock::new();
    FRAME.get_or_init(|| Mutex::new(None))
}

/// Side rail collapse/expand width tween (same duration as accordion).
struct RailWidthAnim {
    from: f64,
    to: f64,
    started: std::time::Instant,
}

fn rail_width_anim() -> &'static Mutex<Option<RailWidthAnim>> {
    static ANIM: OnceLock<Mutex<Option<RailWidthAnim>>> = OnceLock::new();
    ANIM.get_or_init(|| Mutex::new(None))
}

const ACCORDION_ANIM_MS: f64 = 220.0;

fn accordion_ease(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn accordion_anim_progress(anim: &AccordionAnim, now: std::time::Instant) -> (f64, bool) {
    let t = now.duration_since(anim.started).as_secs_f64() * 1000.0 / ACCORDION_ANIM_MS;
    let p = accordion_ease(t);
    let progress = anim.from + (anim.to - anim.from) * p;
    (progress, t >= 1.0)
}

fn schedule_ui_tween_frame(cx: &mut Cx) {
    *accordion_frame().lock().unwrap() = Some(cx.new_next_frame());
}

fn accordion_estimate_h(key: &str) -> f64 {
    match key {
        "today" => 168.0,
        "bookings" | "bills" => 72.0,
        "payments" => 108.0,
        "outstanding" => 72.0,
        "bank" => 108.0,
        "language" => 220.0,
        "role" => 160.0,
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

/// The shift slip goes to the bill printer; say when it did not (the
/// drawer count is saved either way, and the slip stays on the ticket).
fn shift_slip_paper_note(sent: Result<print_share::ReceiptShare, String>) -> Option<String> {
    match sent {
        Ok(share) if share.opened => None,
        Ok(_) => Some(
            "shift slip not printed — no bill printer on this device (shop device: Settings → Bill & KOT format → Bill); the slip stays on the ticket"
                .into(),
        ),
        Err(e) => Some(format!("shift slip not printed ({e}); the slip stays on the ticket")),
    }
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

/// UI-thread stall meter (every app has its own copy). One event that takes
/// this long froze the window for that long: taps queue up, a Cancel cannot
/// paint, Android may show "isn't responding". Each one is logged with the
/// event and the screen (`Kumkaj stall:` in the log, `ui_stall` in local
/// diagnostics) so the slow path can be moved off the UI thread with
/// Progress + Cancel (DEV.md 7b). Long ones (≥ 1 s) are warnings.
const UI_STALL_LOG_MS: f64 = 250.0;

fn note_ui_stall(event: &Event, took: Duration, tree: (u64, u64, u64, u64, u64)) {
    let ms = took.as_secs_f64() * 1000.0;
    let parts = take_stall_spans();
    if ms < UI_STALL_LOG_MS {
        return;
    }
    // Never wait on app_state here: a stall is often a long holder of it.
    let screen = app_state()
        .try_lock()
        .map(|st| format!("{:?}", st.screen))
        .unwrap_or_else(|_| "busy".into());
    let msg = if parts.is_empty() {
        format!("{} took {:.0} ms on {}", event.name(), ms, screen)
    } else {
        format!("{} took {:.0} ms on {} — {parts}", event.name(), ms, screen)
    };
    // Widget lookups: a cold path cache walks every node per lookup.
    let msg = if tree.2 > 0 {
        format!(
            "{msg} [lookups {} misses {} walked {} invalidations {} unstored {}]",
            tree.0, tree.1, tree.2, tree.3, tree.4
        )
    } else {
        msg
    };
    makepad_widgets::log!("Kumkaj stall: {msg}");
    if ms >= 1000.0 {
        diag::warn(diag::Cat::Ui, "ui_stall", &msg);
    } else {
        diag::info(diag::Cat::Ui, "ui_stall", &msg);
    }
}

thread_local! {
    /// Named slow parts of the event being handled (`stall_span`), read by
    /// `note_ui_stall` so a stall line says *what* was slow, not just which
    /// event. Cleared after every event.
    static STALL_SPANS: std::cell::RefCell<Vec<(&'static str, f64)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Times one heavy part of the UI thread's work (`let _span = stall_span("sync_chrome");`).
/// Only parts ≥ 20 ms are kept; nested spans each report their own time.
struct StallSpan {
    name: &'static str,
    started: Instant,
}

impl Drop for StallSpan {
    fn drop(&mut self) {
        let ms = self.started.elapsed().as_secs_f64() * 1000.0;
        if ms >= 20.0 {
            STALL_SPANS.with(|s| s.borrow_mut().push((self.name, ms)));
        }
    }
}

fn stall_span(name: &'static str) -> StallSpan {
    StallSpan {
        name,
        started: Instant::now(),
    }
}

/// "sync_chrome 3× 840 ms, reload_all 1× 410 ms" — the slow parts of the
/// event just handled, largest first. Empties the list for the next event.
fn take_stall_spans() -> String {
    let spans = STALL_SPANS.with(|s| std::mem::take(&mut *s.borrow_mut()));
    let mut by_name: Vec<(&'static str, usize, f64)> = Vec::new();
    for (name, ms) in spans {
        match by_name.iter_mut().find(|(n, _, _)| *n == name) {
            Some(e) => {
                e.1 += 1;
                e.2 += ms;
            }
            None => by_name.push((name, 1, ms)),
        }
    }
    by_name.sort_by(|a, b| b.2.total_cmp(&a.2));
    by_name
        .iter()
        .take(6)
        .map(|(n, c, ms)| format!("{n} {c}× {ms:.0} ms"))
        .collect::<Vec<_>>()
        .join(", ")
}

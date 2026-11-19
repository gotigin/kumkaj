/// What the owner must hear about the books' safety on the device that
/// holds them: a failing backup, none for days, a disk filling up. Empty
/// when all is well.
fn backup_warning(st: &AppState) -> String {
    let Some(store) = st.store.as_ref().filter(|_| st.device_role.is_host()) else {
        return String::new();
    };
    let Ok(health) = store.backup_health() else {
        return String::new();
    };
    let mut out = Vec::new();
    if !health.error.is_empty() {
        out.push(st.t("backup_failed").replace("{e}", &health.error));
    } else if let Some(days) = health.days_since.filter(|d| *d >= 2) {
        out.push(st.t("backup_overdue").replace("{n}", &days.to_string()));
    }
    if health.low_space {
        out.push(st.t("disk_low").replace("{mb}", &health.free_mb.unwrap_or(0).to_string()));
    }
    // Every backup kept on this one device goes with it if it is lost.
    if let Some(days) = health.copy_overdue_days {
        out.push(st.t("backup_copy_overdue").replace("{n}", &days.to_string()));
    }
    out.join(" ")
}

fn list_content_height(count: usize, row: f64, gap: f64) -> f64 {
    let n = count as f64;
    if n <= 0.0 {
        0.0
    } else {
        n * row + (n - 1.0) * gap
    }
}

fn remember_content_row(cx: &mut Cx, view: &mut View, slot: &Mutex<f64>, used: f64, measured: f64) {
    if measured > 1.0 && (measured - used).abs() > 1.0 {
        *slot.lock().unwrap() = measured;
        view.redraw(cx);
    }
}

fn product_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed clears thumb + name + price + the ADD face. The draw replaces it
    // with the measured row.
    HEIGHT.get_or_init(|| Mutex::new(260.0))
}

fn menu_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(220.0))
}

fn cart_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed ≈ one closed line; draw shrinks/grows from measured extent.
    // 320 left a huge empty band under a short cart (same class as Khaynjo).
    HEIGHT.get_or_init(|| Mutex::new(140.0))
}

fn sale_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed clears the AWB line, the when / paid / by line and Select. The
    // draw replaces it with the measured row.
    HEIGHT.get_or_init(|| Mutex::new(156.0))
}

fn customer_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: ThumbAvatar row + item pad; draw replaces with measured extent.
    HEIGHT.get_or_init(|| Mutex::new(90.0))
}

fn suggest_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: SuggestRowView pad 12+12 + title. Old 51 under-cut Fit rows.
    HEIGHT.get_or_init(|| Mutex::new(64.0))
}

fn sidebar_nav_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: SidebarNavRow pad 12+12 (wrap can grow). Old 52 clipped Settings.
    HEIGHT.get_or_init(|| Mutex::new(64.0))
}

fn locale_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: LocaleRow Use 48 + pads + item pad. Old 72 short for Open/Forget.
    HEIGHT.get_or_init(|| Mutex::new(88.0))
}

/// Nested PortalList inside a page `ScrollYView` — **scroll chaining**
/// (`docs/DEV.md`): the list keeps the wheel only while it can still move
/// that way; at the edge the page takes over (browser behaviour). Claiming
/// Y unconditionally trapped the page whenever the pointer sat on a capped
/// list (Parties, Settings picks, …).
fn claim_list_nested_scroll(cx: &mut Cx, host: &View, event: &Event) {
    let Event::Scroll(e) = event else {
        return;
    };
    let list = host.portal_list(cx, ids!(list));
    let list_area = list.area();
    let area = if list_area.is_valid(cx) {
        list_area
    } else {
        host.area()
    };
    if !area.is_valid(cx) || !area.clipped_rect(cx).contains(e.abs) {
        return;
    }
    let dy = e.scroll.y;
    // PortalList: delta = -scroll → scroll.y > 0 moves toward the end.
    if dy != 0.0 {
        let toward_end = dy > 0.0;
        let can_move = if toward_end {
            !list.is_at_end()
        } else {
            list.first_id() > 0 || list.scroll_position() > 0.5
        };
        if can_move {
            e.handled_y.set(true);
        }
    }
    // Vertical lists do not scroll sideways — leave X for `app_hscroll` /
    // ChipStrip so a diagonal trackpad flick does not freeze the page.
}


/// Walk the country list one step from `from`, wrapping at both ends. A code
/// that matches nothing (half-typed, say) resumes from the applied pack.
fn step_country(from: &str, step: i64, applied: &str) -> &'static fiscal::CountryProfile {
    let list = fiscal::countries();
    let at = list
        .iter()
        .position(|c| c.code.eq_ignore_ascii_case(from.trim()))
        .or_else(|| {
            list.iter()
                .position(|c| c.code.eq_ignore_ascii_case(applied.trim()))
        })
        .unwrap_or(0) as i64;
    let n = list.len() as i64;
    &list[(((at + step) % n + n) % n) as usize]
}

/// A tap on a card (Ops parcels, menu cards) — not Makepad `was_tap()`.
/// `was_tap` needs <0.5 s and <5 px, but the page's `axis_lock` waits 8 px
/// before a scroll starts, so a steady press or a 6 px jitter on a touch
/// counter opened nothing while looking like a tap (Khaynjo's floor and
/// menu cards learned this first, 2026-11).
fn finger_up_is_tap(e: FingerUpEvent) -> bool {
    e.is_over && !e.has_long_press_occurred && (e.abs_start - e.abs).length() < 24.0
}

/// Expand-to-content lists: let the page ScrollYView own wheel/trackpad.
/// Forwarding Scroll into PortalList makes it eat the gesture at the bottom edge
/// so Tea / pay / anything below never comes into view.
fn page_owns_scroll(event: &Event) -> bool {
    matches!(event, Event::Scroll(_))
}

/// Resolve on-disk path for a rate photo / party logo / draft image key.
/// A paired device holds the keys only: its copy of the Server's file, or
/// empty (initials) while `lan_images` fetches it.
fn image_key_path(
    key: Option<&str>,
    dir: &std::path::Path,
    kind: lan_images::PhotoKind,
) -> std::path::PathBuf {
    match key {
        Some(k) if k.starts_with("pending_custom:") => {
            std::path::PathBuf::from(&k["pending_custom:".len()..])
        }
        Some(k) if k.starts_with("custom:") => lan_images::ensure_or_request(kind, k, dir),
        other => menu_images::resolve_image_path(other, "", dir),
    }
}

/// Small display copies of photos live beside the shop's item images.
fn thumb_cache_dir() -> std::path::PathBuf {
    let images = item_images_dir();
    images
        .parent()
        .map(|p| p.join("thumbs"))
        .unwrap_or_else(|| images.join("thumbs"))
}

/// Bind `ThumbAvatar` (`thumb` Image + `thumb_initials` Label).
/// Photo when its small copy is ready; otherwise name initials. Full-size
/// decode is never done on the UI thread — see [`thumbs`].
fn apply_thumb_avatar(cx: &mut Cx, root: &WidgetRef, name: &str, path: &std::path::Path) {
    apply_thumb_avatar_kind(cx, root, name, path, thumbs::ThumbKind::Row);
}

fn apply_thumb_avatar_kind(
    cx: &mut Cx,
    root: &WidgetRef,
    name: &str,
    path: &std::path::Path,
    kind: thumbs::ThumbKind,
) {
    let img = root.image(cx, ids!(thumb));
    let initials_lbl = root.label(cx, ids!(thumb_initials));
    let thumb = if path.as_os_str().is_empty() {
        None
    } else {
        thumbs::cached_or_request_kind(path, &thumb_cache_dir(), kind)
    };
    let uid = img.widget_uid().0;
    let queued = thumb
        .as_deref()
        .is_some_and(|small| load_thumb_at(cx, &img, small));
    // Only show the Image when the texture actually loaded — a missing path
    // used to leave a black placeholder (especially on Android).
    let show_photo = thumb.as_deref().is_some_and(|small| {
        thumb_load_cache()
            .lock()
            .unwrap()
            .get(&uid)
            .is_some_and(|(p, _)| p.as_path() == small)
    });
    if show_photo {
        img.set_visible(cx, true);
        initials_lbl.set_visible(cx, false);
        initials_lbl.set_text(cx, "");
    } else {
        img.set_visible(cx, false);
        let initials = menu_images::name_initials(name);
        initials_lbl.set_text(cx, &initials);
        initials_lbl.set_visible(cx, !initials.is_empty());
    }
    if queued {
        root.as_view().redraw_texture_cache();
    }
}

fn load_product_thumb(cx: &mut Cx, item: &WidgetRef, product: &Product) {
    let dir = item_images_dir();
    let path = image_key_path(product.image_key.as_deref(), &dir, lan_images::PhotoKind::Item);
    apply_thumb_avatar(cx, item, &product.name, &path);
}

/// A printer's virtual paper for people: `80 mm roll`, `210 × 297 mm`, or
/// that it is not set yet (docs/PRINTER.md § Paper and preview).
fn paper_summary(p: &print_share::NamedPrinter, t: impl Fn(&str) -> String) -> String {
    match p.virtual_paper() {
        Some(pp) => paper_text(pp, &t),
        None => t("paper_not_set"),
    }
}

/// A virtual paper for people: `80 mm roll` or `210 × 297 mm`.
fn paper_text(paper: os_paper::PaperMm, t: impl Fn(&str) -> String) -> String {
    let mm = os_paper::mm_text;
    if paper.is_roll() {
        t("paper_roll").replace("{w}", &mm(paper.width_mm))
    } else {
        t("paper_size")
            .replace("{w}", &mm(paper.width_mm))
            .replace("{h}", &mm(paper.length_mm))
    }
}

/// Ask the driver of every saved system printer what paper it is set to
/// (once each, on its own thread). The answer becomes the printer's
/// reported paper, fills a virtual paper that is still blank — never one
/// already set — and redraws (`main_impl`). ESC/POS devices have no driver
/// to ask.
fn ask_drivers(printers: &[print_share::NamedPrinter]) {
    for p in printers {
        if p.has_driver() {
            let _ = os_paper::papers_for(&p.target);
        }
    }
}

/// A printer's reported paper for people — what its driver says it is set
/// to — or "N/A" when nothing reported it.
fn reported_summary(p: &print_share::NamedPrinter, t: impl Fn(&str) -> String) -> String {
    let mm = os_paper::mm_text;
    match p.reported_paper() {
        Some(r) => t("paper_size")
            .replace("{w}", &mm(r.width_mm))
            .replace("{h}", &mm(r.height_mm)),
        None => t("na"),
    }
}

/// A saved printer's name for people: its label (Find names it), else its
/// address, else this computer's usual printer.
fn printer_paper_name(p: &print_share::NamedPrinter, t: impl Fn(&str) -> String) -> String {
    let label = p.label.trim();
    if !label.is_empty() {
        return label.to_string();
    }
    let target = p.target.trim();
    if !target.is_empty() {
        return target.to_string();
    }
    t("printer_target_default")
}

/// One saved printer as the lists show it: `name · paper`.
fn printer_paper_line(p: &print_share::NamedPrinter, t: impl Fn(&str) -> String) -> String {
    format!("{} · {}", printer_paper_name(p, &t), paper_summary(p, &t))
}

/// Checklist rows for Receipt / Label routes. `subtitle` = printer id.
fn courier_route_printer_suggest_rows(
    draft: &print_share::DevicePrinters,
    key: print_share::RouteKey,
    t: impl Fn(&str) -> String,
) -> Vec<suggest::SuggestRow> {
    use suggest::SuggestRow;
    let attached: Vec<String> = draft
        .route_for(key)
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let use_l = t("use");
    let clear_l = t("dont_use");
    let mut rows = Vec::new();
    for pid in &attached {
        if let Some(p) = draft.printer_by_id(pid) {
            rows.push(
                SuggestRow::new(printer_paper_name(p, &t), p.id.clone()).with_action(clear_l.clone()),
            );
        }
    }
    for p in &draft.printers {
        if attached.iter().any(|id| id == &p.id) {
            continue;
        }
        rows.push(
            SuggestRow::new(printer_paper_name(p, &t), p.id.clone()).with_action(use_l.clone()),
        );
    }
    if rows.is_empty() {
        rows.push(SuggestRow::new(t("find_printers"), FIND_PRINTERS_ROW.to_string()));
    }
    rows
}

fn route_key_for_suggest(field: SuggestField) -> print_share::RouteKey {
    match field {
        SuggestField::LabelPrinters => print_share::RouteKey::Label,
        _ => print_share::RouteKey::Receipt,
    }
}

/// Toggle a pool printer on Receipt or Label; saves when changed.
fn toggle_courier_route_printer(st: &mut AppState, field: SuggestField, printer_id: &str) -> Result<String, String> {
    if st.device_role == DeviceRole::Client {
        return Err(st.t("printers_host_only"));
    }
    let key = route_key_for_suggest(field);
    match st.printer_draft.toggle_printer_on_route(key, printer_id) {
        Some(true) => {
            if let Some(base) = st.data_base.clone() {
                print_share::save_device_printers(&base, &st.printer_draft)?;
            }
            Ok(st.t("printers_saved"))
        }
        Some(false) => {
            if let Some(base) = st.data_base.clone() {
                print_share::save_device_printers(&base, &st.printer_draft)?;
            }
            Ok(st.t("printers_saved"))
        }
        None => Err(st.t("printers_tap_find")),
    }
}

fn reorder_courier_route_priority(st: &mut AppState, field: SuggestField, from_row: usize, to_row: usize) -> bool {
    if st.device_role == DeviceRole::Client {
        return false;
    }
    let clear_l = st.t("dont_use");
    let from_id = st
        .suggest_rows
        .get(from_row)
        .filter(|r| r.action == clear_l)
        .map(|r| r.subtitle.clone());
    let to_id = st
        .suggest_rows
        .get(to_row)
        .filter(|r| r.action == clear_l)
        .map(|r| r.subtitle.clone());
    let (Some(from_id), Some(to_id)) = (from_id, to_id) else {
        return false;
    };
    let key = route_key_for_suggest(field);
    let route = st.printer_draft.route_mut(key);
    let from = route.iter().position(|id| id == &from_id);
    let to = route.iter().position(|id| id == &to_id);
    let (Some(from), Some(to)) = (from, to) else {
        return false;
    };
    let entry = route.remove(from);
    let to = if to >= route.len() { route.len() } else { to };
    route.insert(to, entry);
    if let Some(base) = st.data_base.clone() {
        let _ = print_share::save_device_printers(&base, &st.printer_draft);
    }
    true
}

/// One row of the preview's printer dropdown: every saved printer is listed;
/// picking one only changes the preview.
struct PreviewPaperRow {
    title: String,
    printer_id: String,
    selected: bool,
}

fn preview_paper_rows(
    printers: &[print_share::NamedPrinter],
    active: Option<&str>,
    t: impl Fn(&str) -> String,
) -> Vec<PreviewPaperRow> {
    let lines: Vec<String> = printers.iter().map(|p| printer_paper_line(p, &t)).collect();
    printers
        .iter()
        .zip(&lines)
        .map(|(p, line)| {
            // Two printers that read the same are told apart by their id.
            let twin = lines.iter().filter(|l| *l == line).count() > 1;
            PreviewPaperRow {
                title: if twin { format!("{line} ({})", p.id) } else { line.clone() },
                printer_id: p.id.clone(),
                selected: active == Some(p.id.as_str()),
            }
        })
        .collect()
}

/// What the preview draws its slip on: the active printer's virtual paper,
/// how the slip is sent there, and its own page as its driver reported it
/// — the page an OS document prints on. `None`: nothing to ask (ESC/POS)
/// or not reported.
#[derive(Clone, Debug, PartialEq)]
struct PreviewPaper {
    virtual_paper: os_paper::PaperMm,
    os_document: bool,
    own: Option<os_paper::Paper>,
}

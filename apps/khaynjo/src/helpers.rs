fn typed_aliases(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Parse `1`, `1.5`, `1,5` into qty milli (1000 = 1 whole).
/// `D1-D20`, `P 1-20`, `AC1 – AC10` → ("D", 1, 20). The prefix is whatever
/// comes before the first number; the second side may repeat it or not.
fn parse_table_range(text: &str) -> Result<(String, u32, u32), String> {
    let text = text.trim().replace('–', "-");
    let (left, right) = text
        .split_once('-')
        .ok_or_else(|| "Type a range like D1-D20".to_string())?;
    let split = |side: &str| -> (String, Option<u32>) {
        let side = side.trim();
        let digits_at = side.find(|c: char| c.is_ascii_digit()).unwrap_or(side.len());
        (side[..digits_at].to_string(), side[digits_at..].trim().parse().ok())
    };
    let (prefix, from) = split(left);
    let (_, to) = split(right);
    match (from, to) {
        (Some(from), Some(to)) if from >= 1 && to >= from => Ok((prefix, from, to)),
        _ => Err("Type a range like D1-D20".into()),
    }
}

fn parse_qty_to_milli(text: &str) -> Result<i64, String> {
    let t = text.trim().replace(',', ".");
    if t.is_empty() {
        return Err("Quantity required".into());
    }
    if t.contains('.') {
        let v: f64 = t
            .parse()
            .map_err(|_| "Quantity must be a number".to_string())?;
        if v <= 0.0 {
            return Err("Quantity must be > 0".into());
        }
        Ok((v * 1000.0).round() as i64)
    } else {
        let whole: i64 = t
            .parse()
            .map_err(|_| "Quantity must be a number".to_string())?;
        if whole <= 0 {
            return Err("Quantity must be > 0".into());
        }
        Ok(domain::qty_to_milli(whole))
    }
}

fn modifiers_as_typed(groups: &[domain::ModifierGroup], currency: &str) -> String {
    domain::ModifierGroup::format_human_list(groups, currency)
}

/// Flat `Name=price` list (area prices, or a single Options group).
fn options_as_typed(mods: &[domain::ModifierOption], currency: &str) -> String {
    mods.iter()
        .map(|modifier| {
            let price = locale::format_major_input(modifier.price_paise, currency);
            if price.is_empty() || price == "0" {
                modifier.name.clone()
            } else {
                format!("{}={price}", modifier.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn schedule_as_typed(windows: &[domain::ScheduleWindow], currency: &str) -> String {
    const DAYS: &[&str] = &["", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    windows
        .iter()
        .map(|w| {
            let days = if w.days.is_empty() {
                String::new()
            } else if w.days == [1, 2, 3, 4, 5] {
                "Mon-Fri ".into()
            } else {
                w.days
                    .iter()
                    .filter_map(|d| DAYS.get(*d as usize).copied())
                    .collect::<Vec<_>>()
                    .join(",")
                    + " "
            };
            let range = format!("{}-{}", w.start, w.end);
            match w.price_paise {
                Some(p) => format!(
                    "{days}{range}={}",
                    locale::format_major_input(p, currency)
                ),
                None => format!("{days}{range}"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn menu_tax_text(store: &Store, product: &Product) -> String {
    if !product.tax_rate_id.is_empty()
        && let Ok(rates) = store.list_tax_rates()
        && let Some(rate) = rates.iter().find(|rate| rate.id == product.tax_rate_id)
    {
        return rate.name.clone();
    }
    locale::format_percent_bps(product.tax_bps, ".")
}

fn name_for_id<'a>(mut pairs: impl Iterator<Item = (&'a str, &'a str)>, id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    pairs
        .find(|(row_id, _)| *row_id == id)
        .map(|(_, name)| name.to_string())
        .unwrap_or_default()
}

fn item_images_dir() -> std::path::PathBuf {
    app_state()
        .lock()
        .unwrap()
        .store
        .as_ref()
        .and_then(|s| s.item_images_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Start a thumbnail decode for this widget and path.
///
/// Returns true only when a new decode was queued. PortalList rebinds every
/// draw; calling the loader again while a decode is in flight calls
/// `begin_async_load`, which redraws, which calls the loader again.
fn set_view_walk(cx: &mut Cx, view: &ViewRef, walk: Walk) {
    if let Some(mut inner) = view.borrow_mut() {
        inner.walk = walk;
    }
    view.redraw(cx);
}

fn set_view_flow(cx: &mut Cx, view: &ViewRef, flow: Flow) {
    if let Some(mut inner) = view.borrow_mut() {
        inner.layout.flow = flow;
    }
    view.redraw(cx);
}

fn size_fill(weight: f64, min: f64) -> Size {
    Size::Fill {
        weight,
        min: Some(min),
        max: None,
    }
}

/// First line of a status / receipt message, kept short for the ticket.
fn ticket_status_line(msg: &str) -> String {
    let first = msg.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let mut out: String = first.chars().take(90).collect();
    if first.chars().count() > 90 {
        out.push('…');
    }
    out
}

/// Bundled default mark — `CARGO_MANIFEST_DIR` / next-to-exe paths are missing
/// on Android APK installs; `include_bytes` is what actually ships.
const DEFAULT_BRAND_MARK_BYTES: &[u8] = include_bytes!("../resources/kumkaj-mark.webp");
const DEFAULT_BRAND_MARK_EMBEDDED: &str = "embedded:kumkaj-mark.webp";

fn default_brand_mark_path() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        for candidate in [
            parent.join("kumkaj_khaynjo/resources/kumkaj-mark.webp"),
            parent.join("kumkaj-khaynjo/resources/kumkaj-mark.webp"),
            parent.join("resources/kumkaj-mark.webp"),
        ] {
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/kumkaj-mark.webp")
}

/// File identity for image widgets: same path with new bytes (re-upload) must
/// reload. Path alone is not enough — `venue_icon.webp` / `item_images/{id}.webp`
/// keep stable names.
fn image_file_fingerprint(path: &std::path::Path) -> u128 {
    let Ok(meta) = std::fs::metadata(path) else {
        return 0;
    };
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    ((meta.len() as u128) << 64) | (mtime as u128)
}

fn thumb_load_cache() -> &'static Mutex<std::collections::HashMap<u64, (std::path::PathBuf, u128)>> {
    static CACHE: OnceLock<Mutex<std::collections::HashMap<u64, (std::path::PathBuf, u128)>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn brand_load_cache() -> &'static Mutex<std::collections::HashMap<u64, (std::path::PathBuf, u128)>> {
    static CACHE: OnceLock<Mutex<std::collections::HashMap<u64, (std::path::PathBuf, u128)>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn load_thumb_at(cx: &mut Cx, image: &ImageRef, path: &std::path::Path) -> bool {
    let uid = image.widget_uid().0;
    let fp = image_file_fingerprint(path);
    {
        let map = thumb_load_cache().lock().unwrap();
        if map
            .get(&uid)
            .is_some_and(|(p, f)| p.as_path() == path && *f == fp)
        {
            return false;
        }
    }
    // Record before the call so a redraw from the load does not re-queue.
    thumb_load_cache()
        .lock()
        .unwrap()
        .insert(uid, (path.to_path_buf(), fp));
    // Synchronous decode. Async left pooled PortalList thumbs blank until an
    // unrelated scroll; completion never repainted recycled list widgets.
    let loaded = if path.is_file() {
        image.load_image_file_by_path(cx, path).is_ok()
    } else {
        false
    };
    if !loaded {
        thumb_load_cache().lock().unwrap().remove(&uid);
    } else {
        image.redraw(cx);
    }
    loaded
}

/// Drop cached loads so a re-upload at the same path is decoded again.
fn invalidate_image_loads_for_path(path: &std::path::Path) {
    thumb_load_cache()
        .lock()
        .unwrap()
        .retain(|_, (p, _)| p.as_path() != path);
    brand_load_cache()
        .lock()
        .unwrap()
        .retain(|_, (p, _)| p.as_path() != path);
    thumbs::forget_failed(path);
}

/// Brand marks load inline on every sync. DSL `src` used to queue an async
/// load of the bundled mark whose completion overwrote the shop logo. Brand
/// widgets carry no DSL `src`; every load here ends with a redraw. Fingerprint
/// skips no-op reloads until the file on disk changes.
fn load_brand_mark_at(cx: &mut Cx, image: &ImageRef, path: &std::path::Path) -> bool {
    let uid = image.widget_uid().0;
    let fp = image_file_fingerprint(path);
    {
        let map = brand_load_cache().lock().unwrap();
        if map
            .get(&uid)
            .is_some_and(|(p, f)| p.as_path() == path && *f == fp)
        {
            return true;
        }
    }
    let ok = path.is_file() && image.load_image_file_by_path(cx, path).is_ok();
    if ok {
        brand_load_cache()
            .lock()
            .unwrap()
            .insert(uid, (path.to_path_buf(), fp));
        image.redraw(cx);
    } else {
        brand_load_cache().lock().unwrap().remove(&uid);
    }
    ok
}

/// Default Kumkaj mark from disk when present, else embedded bytes (Android).
fn load_default_brand_mark(cx: &mut Cx, image: &ImageRef) -> bool {
    let path = default_brand_mark_path();
    if load_brand_mark_at(cx, image, &path) {
        return true;
    }
    let uid = image.widget_uid().0;
    let embed = std::path::PathBuf::from(DEFAULT_BRAND_MARK_EMBEDDED);
    {
        let map = brand_load_cache().lock().unwrap();
        if map
            .get(&uid)
            .is_some_and(|(p, _)| p.as_path() == embed.as_path())
        {
            return true;
        }
    }
    let ok = image
        .load_image_from_data(cx, DEFAULT_BRAND_MARK_BYTES)
        .is_ok();
    if ok {
        brand_load_cache()
            .lock()
            .unwrap()
            .insert(uid, (embed, 1));
        image.redraw(cx);
    } else {
        brand_load_cache().lock().unwrap().remove(&uid);
    }
    ok
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

fn kot_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(150.0))
}

fn customer_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: party row + item pad; draw replaces with measured extent.
    HEIGHT.get_or_init(|| Mutex::new(90.0))
}

fn cart_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(320.0))
}

fn stock_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed clears name + kind + qty (+ Low). Must start *above* the real
    // Fit row — a short Fixed pitch clips into the next sibling and the
    // page ScrollYView has nothing left to scroll (Inventory Masala2).
    HEIGHT.get_or_init(|| Mutex::new(96.0))
}

fn parts_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: method chips + FieldInput + remove. Draw replaces with extent.
    HEIGHT.get_or_init(|| Mutex::new(88.0))
}

fn sale_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: SaleRow two-line card + amount. Draw replaces with extent.
    HEIGHT.get_or_init(|| Mutex::new(96.0))
}

fn category_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: CategoryRow (thumb 48 / Use 48) + Item bottom pad 10.
    HEIGHT.get_or_init(|| Mutex::new(100.0))
}

fn resv_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: ResvRow + Seat/Cancel. WrapLabel can grow; measure catches it.
    HEIGHT.get_or_init(|| Mutex::new(96.0))
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

/// Nested PortalList inside a page `ScrollYView` — **scroll chaining**:
/// the list keeps the wheel only while it can still move that way; at the
/// edge the page takes over. Claiming Y unconditionally trapped the page
/// whenever the pointer sat on a capped list (Language, Settings picks, …).
/// Leave X free for `pos_ticket` / `app_hscroll`.
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
}

/// Horizontal `ScrollXView` (chip strip / order ticket / app shell): claim X
/// only while the strip can still move that way; never claim Y.
fn claim_hstrip_nested_scroll(cx: &mut Cx, strip: &ViewRef, event: &Event) {
    let Event::Scroll(e) = event else {
        return;
    };
    let area = strip.area();
    if !area.is_valid(cx) || !area.clipped_rect(cx).contains(e.abs) {
        return;
    }
    let dx = e.scroll.x;
    if dx == 0.0 {
        return;
    }
    let pos = strip.get_scroll_pos().x;
    let total = strip.get_scroll_view_total().x;
    let seen = strip.get_scroll_view_visible().x;
    let max_x = (total - seen).max(0.0);
    let toward_left = dx > 0.0;
    let can_move = if toward_left {
        pos > 1.0
    } else {
        pos < max_x - 1.0
    };
    if can_move {
        e.handled_x.set(true);
    }
}

/// Expand-to-content lists: let the page ScrollYView own wheel/trackpad.
/// Forwarding Scroll into PortalList makes it eat the gesture at the bottom edge
/// so Tea / pay / anything below never comes into view.
fn page_owns_scroll(event: &Event) -> bool {
    matches!(event, Event::Scroll(_))
}

/// Resolve on-disk path for a product/draft image key (custom photo or empty).
fn image_key_path(key: Option<&str>, dir: &std::path::Path) -> std::path::PathBuf {
    match key {
        Some(k) if k.starts_with("pending_custom:") => {
            std::path::PathBuf::from(&k["pending_custom:".len()..])
        }
        Some(k) if k.starts_with("custom:") => dir.join(&k["custom:".len()..]),
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
/// Photo when its small copy is ready; otherwise name initials (1 word → 2
/// letters, 2+ words → first letter of each of the first two). The full-size
/// photo is never decoded here — that froze every list that showed photos
/// (see [`thumbs`]); the copy is made off the UI thread and the shell redraws
/// when it lands.
fn apply_thumb_avatar(cx: &mut Cx, root: &WidgetRef, name: &str, path: &std::path::Path) {
    apply_thumb_avatar_kind(cx, root, name, path, thumbs::ThumbKind::Row);
}

/// [`apply_thumb_avatar`] with the copy cut for where it shows: a row's
/// small square, or a menu card's full face.
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
    let path = image_key_path(product.image_key.as_deref(), &dir);
    apply_thumb_avatar(cx, item, &product.name, &path);
}

/// The menu window's card: the dish's photo cut to the card's shape.
fn load_product_card_photo(cx: &mut Cx, tile: &WidgetRef, product: &Product) {
    let dir = item_images_dir();
    let path = image_key_path(product.image_key.as_deref(), &dir);
    apply_thumb_avatar_kind(cx, tile, &product.name, &path, thumbs::ThumbKind::Card);
}


fn typed_aliases(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// A cart line that is seated on a table's running order (its id is a real
/// `running_lines` row) — not the counter desk and not a Client's queued KOT.
fn is_table_running_line(running_line_id: &str) -> bool {
    !running_line_id.is_empty()
        && running_line_id != "counter"
        && !running_line_id.starts_with(QUEUED_LINE)
}

/// What the counter says after Send KOT on the shop device: in the staff's
/// language, with the snack kind decided here rather than guessed from
/// English words — a Gujarati "not printed" must still show red, because
/// the kitchen has no paper.
fn kot_sent_status(
    tr: &dyn Fn(&str) -> String,
    kitchens: &str,
    auto_print: bool,
    paper_problem: Option<&str>,
) -> (SnackKind, String) {
    match (auto_print, paper_problem) {
        (true, Some(why)) => (
            SnackKind::Error,
            tr("kot_sent_not_printed")
                .replace("{kitchens}", kitchens)
                .replace("{why}", why),
        ),
        (true, None) => (
            SnackKind::Success,
            tr("kot_sent_printing").replace("{kitchens}", kitchens),
        ),
        (false, _) => (
            SnackKind::Success,
            tr("kot_sent_screen").replace("{kitchens}", kitchens),
        ),
    }
}

/// Order → Floor table pick: seat the desk's unsent punches only on **first**
/// Choose a table (`from` is None). Change table (`from` is Some) must open
/// the picked ticket alone — never drag the previous table's lines over.
fn pick_seats_desk_cart(from_table_id: Option<&str>) -> bool {
    from_table_id.is_none()
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

/// First line of a status / receipt message, kept short for the ticket.
fn ticket_status_line(msg: &str) -> String {
    let msg = print_share::strip_kot_style_marks(msg);
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
    if !path.is_file() {
        thumb_load_cache().lock().unwrap().remove(&uid);
        return false;
    }
    // While a list draws, decode on Makepad's image pool, never here: the
    // first + Add dishes froze ~1 s decoding WebP thumbs one by one inside
    // draw (2026-11-13). The tile keeps its initials until the decode lands;
    // `App::handle_thumb_decodes` then redraws the window and this call finds
    // the texture in the cache. (Async used to leave recycled PortalList
    // thumbs blank because nothing redrew them when the decode finished.)
    // Outside draw (a one-off preview painted from sync_chrome) nothing would
    // come back to it, so that one still decodes here.
    if cx.in_draw_event() {
        match load_image_file_by_path_async(cx, path) {
            Ok(AsyncLoadResult::Loaded) => {}
            Ok(AsyncLoadResult::Loading(..)) => return false,
            Err(_) => {
                thumb_load_cache().lock().unwrap().remove(&uid);
                return false;
            }
        }
    }
    // Record before the call so a redraw from the load does not re-queue.
    thumb_load_cache()
        .lock()
        .unwrap()
        .insert(uid, (path.to_path_buf(), fp));
    // A cache hit after the async decode; a decode here outside draw.
    let loaded = image.load_image_file_by_path(cx, path).is_ok();
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

/// Remember a content row pitch. No redraw from here (redraw+remeasure loops
/// flicker). Updates when the measured average moves by more than 1px.
fn remember_content_row(cx: &mut Cx, view: &mut View, slot: &Mutex<f64>, used: f64, measured: f64) {
    let _ = (cx, view);
    if measured > 1.0 && (measured - used).abs() > 1.0 {
        *slot.lock().unwrap() = measured;
    }
}

fn customer_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    // Seed: party row + item pad; draw replaces with measured extent.
    HEIGHT.get_or_init(|| Mutex::new(90.0))
}

/// Closed cart row pitch only. Open details never live here — see
/// [`cart_open_extra`]. Seed must clear a wrapped name + Hold/In print chips
/// (96 clipped them into Kitchen, 2026-11-02). Cap stays below open-details
/// height so a just-closed line's tall PortalList sample cannot become the
/// closed pitch (cream band above Kitchen, 2026-11-08).
const CART_CLOSED_ROW_SEED: f64 = 140.0;
const CART_CLOSED_ROW_MIN: f64 = 72.0;
const CART_CLOSED_ROW_MAX: f64 = 176.0;

fn cart_row_height() -> &'static Mutex<f64> {
    static HEIGHT: OnceLock<Mutex<f64>> = OnceLock::new();
    HEIGHT.get_or_init(|| Mutex::new(CART_CLOSED_ROW_SEED))
}

/// Last measured **closed** cart list extent (all lines, no open details) and
/// the line count it belongs to. Pitch-only shrink left a cream band when
/// PortalList's sample would not move `cart_row_height` enough; the next
/// closed frame uses this extent as the Fixed walk instead.
fn cart_closed_extent() -> &'static Mutex<(usize, f64)> {
    static EXT: OnceLock<Mutex<(usize, f64)>> = OnceLock::new();
    EXT.get_or_init(|| Mutex::new((0, 0.0)))
}

/// Extra height for **one** open ticket line (unit, applied, modifiers, note,
/// Cancel / NC / Move). Separate from [`cart_row_height`]: folding this into
/// the closed pitch after close leaves a permanent cream band under every
/// closed row. CartList measures it while a line is open — too short clips
/// Cancel / NC / Move into Kitchen; too tall is empty cream under the open
/// details (shrinks on the next settle frame).
fn cart_open_extra() -> &'static Mutex<f64> {
    static EXTRA: OnceLock<Mutex<f64>> = OnceLock::new();
    EXTRA.get_or_init(|| Mutex::new(320.0))
}

/// Reserved open-details height for the cart list (bug 8a). Zero when nothing
/// is open, or when the open index is past the cart (stale after lines were
/// removed) — that case must not leave cream above Kitchen with no details.
fn cart_open_extra_reserved(open_line: Option<usize>, line_count: usize, extra: f64) -> f64 {
    if open_line.filter(|&i| i < line_count).is_some() {
        extra.max(0.0)
    } else {
        0.0
    }
}

/// Closed pitch after a PortalList sample: keep open-tall samples out of the
/// closed slot (cream band regression).
fn cart_closed_pitch_from_sample(sample: f64, current: f64) -> f64 {
    if sample > CART_CLOSED_ROW_MAX {
        CART_CLOSED_ROW_SEED
    } else if sample > 1.0 {
        sample.clamp(CART_CLOSED_ROW_MIN, CART_CLOSED_ROW_MAX)
    } else {
        current.clamp(CART_CLOSED_ROW_MIN, CART_CLOSED_ROW_MAX)
    }
}

/// Kitchen Live card body: one dish per line (easy to scan), not a single
/// `·`-joined run that truncates mid-name. Store summaries stay compact for
/// print/sync; the card reformats for the cook.
fn kitchen_card_body(summary: &str) -> String {
    let s = summary.trim();
    if s.is_empty() {
        return String::new();
    }
    let (main, reason) = match s.rsplit_once(" — ") {
        Some((a, b)) if !b.trim().is_empty() => (a.trim(), Some(b.trim())),
        _ => (s, None),
    };
    // Cook summaries often start with `[Station] ` (store send path).
    let (station, main) = if main.starts_with('[') {
        if let Some((tag, rest)) = main.split_once(']') {
            (Some(format!("{tag}]")), rest.trim())
        } else {
            (None, main)
        }
    } else {
        (None, main)
    };
    let sep = if main.contains(" · ") { " · " } else { ", " };
    let parts: Vec<&str> = main
        .split(sep)
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() <= 1 && reason.is_none() && station.is_none() {
        // Already a single blob (or one item) — still normalize qty display.
        return kitchen_format_item_part(main);
    }
    let mut lines: Vec<String> = Vec::new();
    if let Some(st) = station {
        lines.push(st);
    }
    let mut start = 0usize;
    if let Some(first) = parts.first() {
        if !kitchen_part_is_item(first) {
            lines.push((*first).to_string());
            start = 1;
        }
    }
    if parts
        .get(start)
        .is_some_and(|p| p.eq_ignore_ascii_case("CANCEL"))
    {
        start += 1;
    }
    for p in &parts[start..] {
        lines.push(kitchen_format_item_part(p));
    }
    if let Some(r) = reason {
        lines.push(r.to_string());
    }
    lines.join("\n")
}

fn kitchen_part_is_item(p: &str) -> bool {
    let t = p.trim();
    if t.is_empty() || t.eq_ignore_ascii_case("CANCEL") {
        return false;
    }
    if t.contains('|') || t.starts_with("CANCEL ") {
        return true;
    }
    kitchen_split_qty(t).is_some()
}

fn kitchen_format_item_part(p: &str) -> String {
    let mut t = p.trim();
    let cancel = if let Some(rest) = t.strip_prefix("CANCEL ") {
        t = rest.trim();
        true
    } else {
        false
    };
    let body = if let Some((name, qty)) = t.rsplit_once('|') {
        let qty = qty
            .trim()
            .trim_start_matches(['x', 'X', '×', '✕'])
            .trim();
        format!("×{qty}  {}", name.trim())
    } else if let Some((name, qty)) = kitchen_split_qty(t) {
        format!("×{qty}  {name}")
    } else {
        t.to_string()
    };
    if cancel {
        format!("CANCEL  {body}")
    } else {
        body
    }
}

/// Leading qty on a summary fragment: `x1 Name`, `2×Name`, `1.5 kg×Rice`.
fn kitchen_split_qty(line: &str) -> Option<(String, String)> {
    let s = line.trim();
    let mut rest = s;
    for mark in ['x', 'X', '×', '✕'] {
        if let Some(stripped) = rest.strip_prefix(mark) {
            if stripped.starts_with(|c: char| c.is_ascii_digit()) {
                rest = stripped;
            }
            break;
        }
    }
    if !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let mut end = 0usize;
    let mut seen_digit = false;
    let mut seen_sep = false;
    for (i, ch) in rest.char_indices() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            end = i + ch.len_utf8();
        } else if (ch == '.' || ch == ',') && seen_digit && !seen_sep {
            seen_sep = true;
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    if !seen_digit {
        return None;
    }
    let qty_num = rest[..end].to_string();
    let after = rest[end..].trim_start();
    // Optional unit before ×: `kg× Rice` / `kg x Rice`
    let x_at = after
        .char_indices()
        .find(|(_, ch)| matches!(ch, '×' | 'x' | 'X' | '✕'))
        .map(|(i, _)| i);
    let (qty, name) = if let Some(x_at) = x_at {
        let unit = after[..x_at].trim();
        let mark_len = after[x_at..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        let name = after[x_at + mark_len..]
            .trim()
            .trim_start_matches(['×', 'x', 'X', '✕', ' '])
            .trim()
            .to_string();
        let qty = if unit.is_empty() {
            qty_num
        } else {
            format!("{qty_num} {unit}")
        };
        (qty, name)
    } else if after.is_empty() {
        return None;
    } else {
        // `2 Name` / `x1 Name` after the leading mark was stripped
        (qty_num, after.to_string())
    };
    if name.is_empty() {
        None
    } else {
        Some((name, qty))
    }
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

/// Wheel / trackpad into a nested `ScrollXView` that lives inside an
/// expand-to-content list (`page_owns_scroll` blocks forwarding Scroll into
/// the PortalList). Applies X (and Shift+vertical, same as `axis_lock` bars)
/// then claims so the page does not also pan sideways.
fn apply_hstrip_wheel(cx: &mut Cx, strip: &ViewRef, event: &Event) {
    let Event::Scroll(e) = event else {
        return;
    };
    let area = strip.area();
    if !area.is_valid(cx) || !area.clipped_rect(cx).contains(e.abs) {
        return;
    }
    let mut dx = e.scroll.x;
    if dx == 0.0 && e.modifiers.shift {
        dx = e.scroll.y;
    }
    if dx == 0.0 {
        return;
    }
    let pos = strip.get_scroll_pos();
    let max_x = (strip.get_scroll_view_total().x - strip.get_scroll_view_visible().x).max(0.0);
    // ScrollBar uses `scroll_pos + scroll` for wheel/trackpad deltas.
    let new_x = (pos.x + dx).clamp(0.0, max_x);
    if (new_x - pos.x).abs() < 0.5 {
        return;
    }
    strip.set_scroll_pos(cx, dvec2(new_x, pos.y));
    strip.redraw(cx);
    e.handled_x.set(true);
}

/// Where a press lands on a sheet that covers the window (Add dishes, UPI).
///
/// Makepad closes a modal only when a release lands outside its `content`
/// view. These sheets give `content` the whole window, so nothing is ever
/// outside and the dimmed area never dismissed them. The visible sheet is
/// the real inside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SheetPress {
    /// On the dimmed area around the sheet.
    Outside,
    /// On the sheet's own close control.
    Done,
    /// Anywhere else on the sheet: dishes, search, tabs.
    Inside,
}

fn sheet_press_target(sheet: Rect, done: &[Rect], abs: DVec2) -> SheetPress {
    let hit = |r: &Rect| r.size.x > 0.0 && r.size.y > 0.0 && r.contains(abs);
    if done.iter().any(hit) {
        SheetPress::Done
    } else if hit(&sheet) {
        SheetPress::Inside
    } else {
        SheetPress::Outside
    }
}

/// What the sheet does with one pointer phase.
///
/// The press decides the gesture. A press on the dimmed area or on Done
/// closes the sheet at once. A press on the dishes belongs to the dishes for
/// the whole gesture: a scroll that ends outside the sheet, or over Done,
/// never closes it. A release over Done closes only when its press was not
/// seen (another handler took the down).
fn sheet_press_closes(
    pressed: Option<SheetPress>,
    is_release: bool,
    target: SheetPress,
) -> bool {
    if !is_release {
        return matches!(target, SheetPress::Outside | SheetPress::Done);
    }
    match pressed {
        Some(_) => false,
        None => target == SheetPress::Done,
    }
}

/// The Add dishes sheet as laid out against the window: the modal pads it
/// 16 px at the sides and 20 px at top and bottom, at most 980 wide. Used
/// when the overlay has not reported its own rect yet.
fn menu_sheet_fallback(window: DVec2) -> Rect {
    let w = (window.x - 32.0).clamp(160.0, 980.0);
    let h = (window.y - 40.0).max(160.0);
    Rect {
        pos: dvec2((window.x - w) * 0.5, (window.y - h) * 0.5),
        size: dvec2(w, h),
    }
}

/// Done on Add dishes: the top-right corner of the sheet, about 176 × 84.
/// Its own hit box is unreliable while the search field holds focus.
fn menu_done_corner(sheet: Rect) -> Rect {
    Rect {
        pos: dvec2(sheet.pos.x + sheet.size.x - 176.0, sheet.pos.y),
        size: dvec2(176.0_f64.min(sheet.size.x), 84.0_f64.min(sheet.size.y)),
    }
}

/// Tap on a Floor tile / menu card — not Makepad `was_tap()`.
/// `was_tap` needs <0.5s and <5px, but `axis_lock` waits 8px before a
/// scroll starts, so a steady press or 6px jitter opened nothing while
/// looking like a tap. Same rule as the menu cards (2026-11).
fn finger_up_is_tap(e: FingerUpEvent) -> bool {
    e.is_over
        && !e.has_long_press_occurred
        && (e.abs_start - e.abs).length() < 24.0
}

/// Expand-to-content lists: let the page ScrollYView own wheel/trackpad.
/// Forwarding Scroll into PortalList makes it eat the gesture at the bottom edge
/// so Tea / pay / anything below never comes into view.
fn page_owns_scroll(event: &Event) -> bool {
    matches!(event, Event::Scroll(_))
}

/// Resolve on-disk path for a product/draft image key (custom / shelf / LAN cache).
fn image_key_path(key: Option<&str>, dir: &std::path::Path) -> std::path::PathBuf {
    let Some(k) = key.map(str::trim).filter(|s| !s.is_empty()) else {
        return std::path::PathBuf::new();
    };
    // Client devices: pull missing photos from the Server (Android has no shelf pack).
    let link = {
        let st = app_state().lock().unwrap();
        if st.device_role == DeviceRole::Client {
            st.store.as_ref().and_then(|store| {
                let base = store.server_url().ok().flatten()?;
                let device = store.client_device_token().ok().flatten()?;
                let server_id = store.paired_server_id().ok().flatten().unwrap_or_default();
                Some(net::Link {
                    base,
                    device,
                    session: String::new(),
                    server_id,
                })
            })
        } else {
            None
        }
    };
    lan_images::ensure_or_request(k, dir, link.as_ref())
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

/// Show the first alias under a name (ProductCard / MenuCard / DishRow /
/// CategoryRow). Hidden when there is none.
fn paint_first_alias(cx: &mut Cx, root: &WidgetRef, alias: Option<&str>) {
    // WrapLabel or Label — widget() covers both (MenuCard / ProductCard).
    let w = root.widget(cx, ids!(alias));
    match alias.map(str::trim).filter(|a| !a.is_empty()) {
        Some(text) => {
            w.set_visible(cx, true);
            w.set_text(cx, &plain_display_text(text));
        }
        None => {
            w.set_visible(cx, false);
            w.set_text(cx, "");
        }
    }
}


/// Order desk pane widths (Order, Counter) for a content width `view_w`
/// (window less rail). Both mins fit: they share the room in the mins'
/// ratio. Else each keeps its min and the desk pans sideways — but a pane is
/// never wider than the screen less a `peek` of its neighbour, so on a phone
/// each pane is one screen and the other shows at the edge (same rule as
/// Moklanjo's booking desk, DESIGN.md).
fn order_desk_pane_widths(
    view_w: f64,
    left_min: f64,
    right_min: f64,
    gap: f64,
    peek: f64,
) -> (f64, f64) {
    let desk_avail = (view_w - 32.0).max(200.0);
    if desk_avail >= left_min + right_min + gap {
        let left = desk_avail * (left_min / (left_min + right_min));
        return (left, desk_avail - gap - left);
    }
    // `view_w` is the desk's visible width (its 16 px side padding taken off).
    let screen = (view_w - peek).max(240.0);
    (left_min.min(screen), right_min.min(screen))
}

/// Set `view`'s padding to its DSL padding plus `extra`. The DSL value is
/// captured the first time per `key`, so calling this on every chrome sync
/// never stacks the extra. Redraws only on a real change.
fn pad_view_over_base(cx: &mut Cx, view: &ViewRef, key: LiveId, extra: Inset) {
    static BASES: OnceLock<Mutex<Vec<(LiveId, Inset)>>> = OnceLock::new();
    let changed = {
        let Some(mut inner) = view.borrow_mut() else {
            return;
        };
        let base = {
            let mut bases = BASES.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
            match bases.iter().find(|(k, _)| *k == key) {
                Some((_, base)) => *base,
                None => {
                    let base = inner.layout.padding;
                    bases.push((key, base));
                    base
                }
            }
        };
        let want = Inset {
            left: base.left + extra.left,
            right: base.right + extra.right,
            top: base.top + extra.top,
            bottom: base.bottom + extra.bottom,
        };
        let have = inner.layout.padding;
        let differs = (have.left - want.left).abs() > 0.5
            || (have.right - want.right).abs() > 0.5
            || (have.top - want.top).abs() > 0.5
            || (have.bottom - want.bottom).abs() > 0.5;
        if differs {
            inner.layout.padding = want;
        }
        differs
    };
    if changed {
        view.redraw(cx);
    }
}

/// Window-filling sheets (Modal) whose content [`App::fit_inside_system_bars`]
/// keeps off the status / gesture bars. Anchored overlays (the nav drag
/// ghost, column dropdowns) are left out: they place themselves.
/// `nav_order_tests::every_modal_is_an_inset_sheet_or_anchored` checks this.
const INSET_SHEETS: &[&str] = &["menu_modal", "lock_staff_modal", "upi_qr_modal"];

/// The client / brand name inside a catalogue pack title, so each screen can
/// name its own level: onboarding "{name} App Template", Menu "{name}
/// Template". The private pack is one download for both (menu + shop
/// settings); its title in the catalogue is e.g. "Rajdhani Menu Template".
fn template_client_name(title: &str) -> String {
    let t = title.trim();
    let lower = t.to_lowercase();
    for suffix in [
        " app template",
        " menu template",
        " items template",
        " full template",
        " template",
        " menu",
    ] {
        if lower.ends_with(suffix) && lower.len() > suffix.len() {
            return t[..t.len() - suffix.len()].trim().to_string();
        }
    }
    t.to_string()
}

#[cfg(test)]
mod template_name_tests {
    use super::*;

    #[test]
    fn pack_titles_give_the_client_name_for_each_level() {
        assert_eq!(template_client_name("Rajdhani Menu Template"), "Rajdhani");
        assert_eq!(template_client_name("Rajdhani App Template"), "Rajdhani");
        assert_eq!(template_client_name("Khaynjo Full Template"), "Khaynjo");
        assert_eq!(template_client_name("Rajdhani"), "Rajdhani");
        assert_eq!(template_client_name("  Template "), "Template");
    }
}

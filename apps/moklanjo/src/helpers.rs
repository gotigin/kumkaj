fn typed_aliases(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Parse `1`, `1.5`, `1,5` into qty milli (1000 = 1 whole).
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

fn modifiers_as_typed(mods: &[domain::ModifierOption], currency: &str) -> String {
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

/// Booking desk pane widths (Parcel, Counter) for a content width `view_w`
/// (window less rail). Both mins fit: they share the room in the mins'
/// ratio. Else each keeps its min and the desk pans sideways — but a pane is
/// never wider than the screen less a `peek` of its neighbour, so on a phone
/// each pane is one screen and the other shows at the edge.
fn booking_desk_pane_widths(
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

fn item_images_dir() -> std::path::PathBuf {
    app_state()
        .lock()
        .unwrap()
        .store
        .as_ref()
        .and_then(|s| s.item_images_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn party_images_dir() -> std::path::PathBuf {
    app_state()
        .lock()
        .unwrap()
        .store
        .as_ref()
        .and_then(|s| s.party_images_dir().ok())
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

/// Bundled default mark — `CARGO_MANIFEST_DIR` / next-to-exe paths are missing
/// on Android APK installs; `include_bytes` is what actually ships.
const DEFAULT_BRAND_MARK_BYTES: &[u8] = include_bytes!("../resources/kumkaj-mark.webp");
const DEFAULT_BRAND_MARK_EMBEDDED: &str = "embedded:kumkaj-mark.webp";

fn default_brand_mark_path() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        for candidate in [
            parent.join("kumkaj_moklanjo/resources/kumkaj-mark.webp"),
            parent.join("kumkaj-moklanjo/resources/kumkaj-mark.webp"),
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
/// reload. Path alone is not enough — stable names like `venue_icon.webp`.
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
/// widgets carry no DSL `src`; every load here ends with a redraw.
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
const INSET_SHEETS: &[&str] = &[];

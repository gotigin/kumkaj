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
    thumb_load_cache()
        .lock()
        .unwrap()
        .insert(uid, (path.to_path_buf(), fp));
    // Synchronous decode (same as Khaynjo). Async never repainted recycled
    // PortalList thumbs — photos only appeared after leaving the screen.
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

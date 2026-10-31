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
    HEIGHT.get_or_init(|| Mutex::new(320.0))
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

/// Horizontal `ScrollXView` (chip / tab strip / bottom nav): claim X only
/// while the strip can still move that way; never claim Y so the page keeps
/// vertical scroll when the pointer is over chips.
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
    // scroll.x > 0 typically reveals content to the left (pos decreases).
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
    let path = image_key_path(product.image_key.as_deref(), &dir);
    apply_thumb_avatar(cx, item, &product.name, &path);
}

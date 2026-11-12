//! khaynjo.com — Khaynjo's site as a fullpage Makepad 2 app (wasm in the
//! browser; the same crate runs native for previews and as an offline
//! presenter in a restaurant). Story: apps/khaynjo/docs/KHAYNJO_WEBSITE.md.
//! Copied from sites/kumkaj, then diverged (copy-then-diverge; no shared
//! site engine).
//!
//! One scrolling page (redesign 2026-11-12; was a 10-slide deck): sticky
//! top bar with section links and the Play CTA; sections hero · how ·
//! features · menus · offline · pricing · faq · closing · footer.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:menus` opens a 390×844 window scrolled to a
//!   section (id, old chapter id, or nothing for the top).
//! - `SITE_LINKS=local` sends the kumkaj.com link to the local wasm server.
//!
//! In the browser, `#menus` (or an old chapter link such as `#kot`) opens
//! that section; a page served from 127.0.0.1 / localhost links back to the
//! local kumkaj.com (:8010).

pub use makepad_widgets;

use makepad_widgets::*;

mod floor;
use floor::Visit;

mod world_shelf;
use world_shelf::{WorldDish, WORLD};

app_main!(App);

include!("ui.rs");

// ---- Owner switches (docs/WEBSITES.md §7 launch gates, §8 owner inputs) --

/// `kot-autoprint` and the multi-device half of `offline-lan` wait for the
/// owner's real-hardware LAN pilot (docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md
/// §3; KHAYNJO_WEBSITE.md §5 "Gate").
const LAN_PILOT_PASSED: bool = false;

/// Play listing is live (`com.khaynjo`). Desktop installers still land on
/// the download page when we publish MSI/AUR packages there.
const DOWNLOADS_LIVE: bool = true;
/// "Download free" → Google Play (Android). Desktop packages stay on
/// khaynjo.com/download when that page ships installers.
const DOWNLOAD_URL: &str = "https://play.google.com/store/apps/details?id=com.khaynjo";

/// TODO(owner): Khaynjo's WhatsApp Business number — digits only, with the
/// country code (no "+", no spaces). Agents must not invent one. Until it
/// is set, the button says the demo line is coming and shows the message.
const WHATSAPP_NUMBER: Option<&str> = None;
const WHATSAPP_MESSAGE: &str = "Khaynjo demo — restaurant type, tables, city";

/// Same-host legal pages (deploy-firebase.sh stages them next to the wasm).
const PRIVACY_PATH: &str = "/privacy";
const TERMS_PATH: &str = "/terms/";
const PUBLIC_HOST: &str = "https://khaynjo.com";

// ---- Sections ----------------------------------------------------------

/// Page sections in order: (hash id, widget path).
const SECTIONS: [(&str, &[LiveId]); 8] = [
    ("top", ids!(hero)),
    ("how", ids!(how)),
    ("features", ids!(features)),
    ("menus", ids!(menus)),
    ("offline", ids!(offline)),
    ("pricing", ids!(pricing)),
    ("faq", ids!(faq)),
    ("start", ids!(closing)),
];

/// Links from the old 10-chapter deck keep landing somewhere sensible.
const OLD_CHAPTERS: [(&str, &str); 10] = [
    ("rush", "top"),
    ("order", "how"),
    ("kot", "how"),
    ("kitchen", "how"),
    ("changes", "features"),
    ("settle", "how"),
    ("dayclose", "how"),
    ("menu", "menus"),
    ("offline", "offline"),
    ("start", "start"),
];

/// Top-bar links → section ids.
const NAV_LINKS: [(&[LiveId], &str); 6] = [
    (ids!(go_how), "how"),
    (ids!(go_features), "features"),
    (ids!(go_menus), "menus"),
    (ids!(go_offline), "offline"),
    (ids!(go_pricing), "pricing"),
    (ids!(go_faq), "faq"),
];

/// Each band's content column; side padding / max width set per tier.
const WRAPS: [&[LiveId]; 9] = [
    ids!(hero_wrap),
    ids!(how_wrap),
    ids!(features_wrap),
    ids!(menus_wrap),
    ids!(offline_wrap),
    ids!(pricing_wrap),
    ids!(faq_wrap),
    ids!(closing_wrap),
    ids!(footer_wrap),
];

/// Floor toy tables: path and example running bill (₹).
const TABLES: [(&[LiveId], u32); 6] = [
    (ids!(floor.t1), 640),
    (ids!(floor.t2), 1240),
    (ids!(floor.t3), 380),
    (ids!(floor.t4), 880),
    (ids!(floor.t5), 1560),
    (ids!(floor.t6), 520),
];
/// The floor opens mid-lunch, one table in each colour but Paid.
const OPENING_FLOOR: [Visit; 6] = [
    Visit::Free,
    Visit::Occupied,
    Visit::InKitchen,
    Visit::FoodReady,
    Visit::Served,
    Visit::BillPrinted,
];
const TILE_LAYERS: [LiveId; 7] = [
    live_id!(bg_free),
    live_id!(bg_seated),
    live_id!(bg_kitchen),
    live_id!(bg_ready),
    live_id!(bg_served),
    live_id!(bg_bill),
    live_id!(bg_paid),
];

/// App layout floor: design for ≥375 CSS px. Narrower viewports scroll
/// horizontally (web shell CSS); native still lays out at this width.
const MIN_WIDTH: f64 = 375.0;

/// Rows that sit side by side only when the content column is at least
/// this wide (px), and stack otherwise.
const ROWS: [(&[LiveId], f64); 26] = [
    (ids!(hero_row), 900.0),
    (ids!(hero_cta), 420.0),
    (ids!(visit_key), 760.0),
    (ids!(key_r0), 0.0),
    (ids!(step_order), 820.0),
    (ids!(step_kot), 820.0),
    (ids!(kitchen_head), 820.0),
    (ids!(kds), 640.0),
    (ids!(step_settle), 820.0),
    (ids!(step_close), 820.0),
    (ids!(feat_r0), 640.0),
    (ids!(feat_r1), 640.0),
    (ids!(feat_r2), 640.0),
    (ids!(feat_r3), 640.0),
    (ids!(tmpl_row), 760.0),
    (ids!(menus.shelf_q0), SHELF_WIDE),
    (ids!(menus.shelf_q1), SHELF_WIDE),
    (ids!(menus.shelf_bar), 520.0),
    (ids!(offline_row), 860.0),
    (ids!(price_row), 720.0),
    (ids!(faq_r0), 720.0),
    (ids!(faq_r1), 720.0),
    (ids!(faq_r2), 720.0),
    (ids!(close_cta), 420.0),
    (ids!(footer_row), 640.0),
    (ids!(footer_links), 0.0),
];
/// Fixed-width props: (path, desk width, the min width of their row).
/// When the row stacks, the prop goes full width.
const PROPS: [(&[LiveId], f64, f64); 6] = [
    (ids!(floor_card), 520.0, 900.0),
    (ids!(order_card), 380.0, 820.0),
    (ids!(ticket), 340.0, 820.0),
    (ids!(bill), 380.0, 820.0),
    (ids!(drawer), 380.0, 820.0),
    (ids!(status_card), 400.0, 860.0),
];
/// Top-bar section links need this much bar width.
const NAV_LINKS_FROM: f64 = 1000.0;

const MAX_CONTENT: f64 = 1160.0;
const MAX_CONTENT_WIDE: f64 = 1280.0;

/// Content width from which the world shelf shows 8 cards (4 × 2) a page;
/// narrower shows 4 (2 × 2) so a phone page stays short.
const SHELF_WIDE: f64 = 700.0;
/// World shelf card slots, in reading order (s4–s7 live in `shelf_q1`).
const SHELF_CARDS: [&[LiveId]; 8] = [
    ids!(menus.s0.card),
    ids!(menus.s1.card),
    ids!(menus.s2.card),
    ids!(menus.s3.card),
    ids!(menus.s4.card),
    ids!(menus.s5.card),
    ids!(menus.s6.card),
    ids!(menus.s7.card),
];

/// Countries on the shelf that have a photo.
fn world_photo_count() -> usize {
    WORLD.iter().filter(|d| d.photo.is_some()).count()
}

/// The shelf list for the current filter (photos only, or every country).
fn shelf_entries(all: bool) -> Vec<&'static WorldDish> {
    WORLD.iter().filter(|d| all || d.photo.is_some()).collect()
}

/// (never repeat the English dish twice).
fn shelf_native_line(d: &WorldDish) -> Option<String> {
    let mut parts = Vec::new();
    if d.country_native != d.country {
        parts.push(d.country_native);
    }
    if d.dish_native != d.dish && d.dish_native != d.country_native {
        parts.push(d.dish_native);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}


/// (page index, page count) for the first card shown and the page size.
fn shelf_page_of(first: usize, per_page: usize, len: usize) -> (usize, usize) {
    let per = per_page.max(1);
    let pages = len.div_ceil(per).max(1);
    ((first / per).min(pages - 1), pages)
}

/// Web path of a world photo inside the Makepad package
/// (`<base>/khaynjo_site/resources/dishes/world/<file>`), where `<base>` is
/// the page path without a trailing file name — same rule Makepad uses for
/// `crate_resource` so a sub-path deploy keeps working.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn world_photo_web_path(pathname: &str, file: &str) -> String {
    let path = pathname.trim_end_matches('/');
    let last = path.rsplit('/').next().unwrap_or_default();
    let base = if last.contains('.') {
        path.rsplit_once('/').map_or("", |(b, _)| b)
    } else {
        path
    };
    let base = base.trim_matches('/');
    if base.is_empty() {
        format!("/khaynjo_site/resources/dishes/world/{file}")
    } else {
        format!("/{base}/khaynjo_site/resources/dishes/world/{file}")
    }
}

/// Section id for a URL hash: a section id, an old chapter id, or an old
/// chapter number (`#3`). Unknown → None (stay at the top).
fn section_from_hash(hash: &str) -> Option<&'static str> {
    let key = hash.trim_start_matches('#').trim().to_ascii_lowercase();
    if key.is_empty() {
        return None;
    }
    if let Some((id, _)) = SECTIONS.iter().find(|(id, _)| *id == key) {
        return Some(id);
    }
    if let Some((_, to)) = OLD_CHAPTERS.iter().find(|(old, _)| *old == key) {
        return Some(to);
    }
    let n: usize = key.parse().ok()?;
    OLD_CHAPTERS.get(n.checked_sub(1)?).map(|(_, to)| *to)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tier {
    Phone,
    Narrow,
    Tablet,
    Desk,
    /// Large monitors and TVs (≥ 1600 × 900): type steps up.
    Wide,
}

impl Tier {
    fn from_size(w: f64, h: f64) -> Self {
        if w < 480.0 {
            Self::Phone
        } else if w < 720.0 {
            Self::Narrow
        } else if w < 1024.0 {
            Self::Tablet
        } else if w >= 1600.0 && h >= 900.0 {
            Self::Wide
        } else {
            Self::Desk
        }
    }

    /// Content never grows wider than this; wider windows get wider margins.
    fn max_content(self) -> f64 {
        if self == Self::Wide { MAX_CONTENT_WIDE } else { MAX_CONTENT }
    }

    fn type_scale(self, desk_size: f32) -> f32 {
        let (display, heading, body) = match self {
            Self::Phone => (0.62, 0.8, 0.95),
            Self::Narrow => (0.72, 0.86, 0.97),
            Self::Tablet => (0.86, 0.94, 1.0),
            Self::Desk => (1.0, 1.0, 1.0),
            Self::Wide => (1.12, 1.06, 1.04),
        };
        if desk_size >= 40.0 {
            display
        } else if desk_size >= 19.0 {
            heading
        } else if desk_size >= 15.0 {
            body
        } else {
            1.0
        }
    }

    /// (minimum side padding, vertical band padding).
    fn spacing(self) -> (f64, f64) {
        match self {
            Self::Phone => (18.0, 44.0),
            Self::Narrow => (26.0, 52.0),
            Self::Tablet => (40.0, 64.0),
            Self::Desk => (56.0, 76.0),
            Self::Wide => (72.0, 88.0),
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    laid_out: Option<(Tier, i64, i64)>,
    #[rust]
    desk_font: Vec<f32>,
    #[rust]
    tables: Option<[Visit; 6]>,
    /// Section to scroll to once the page has a layout (hash / preview).
    #[rust]
    pending_section: PendingSection,
    /// Frames left to re-apply the deep-link scroll while layout settles
    /// (a resize right after startup moves every section).
    #[rust]
    pending_frames: u8,
    /// Last target y and how many frames it held still.
    #[rust]
    pending_last_y: f64,
    #[rust]
    pending_stable: u8,
    /// World shelf: index of the first card on the page (kept across
    /// resizes so the same countries stay in view), filter, page size.
    #[rust]
    shelf_first: usize,
    #[rust]
    shelf_all: bool,
    #[rust]
    shelf_per_page: usize,
    /// World photo bytes by file (fetched once, decoded on demand) and the
    /// HTTP requests still in flight (wasm).
    #[rust]
    photo_bytes: PhotoBytes,
    #[rust]
    photo_pending: PhotoPending,
}

/// World photo bytes by file name (the Script derive wants plain field types).
type PhotoBytes = std::collections::HashMap<String, std::rc::Rc<Vec<u8>>>;
/// Section to scroll to after layout (the Script derive wants plain types).
type PendingSection = Option<&'static str>;
/// In-flight photo fetches: (request id, file name).
type PhotoPending = Vec<(LiveId, String)>;

impl App {
    /// Scroll the page so `section` sits under the top bar.
    fn scroll_to(&mut self, cx: &mut Cx, section: &str) -> f64 {
        let Some(&(id, path)) = SECTIONS.iter().find(|(id, _)| *id == section) else {
            return 0.0;
        };
        let page = self.ui.view(cx, ids!(page));
        let page_rect = page.area().rect(cx);
        let target = self.ui.widget(cx, path).area().rect(cx);
        // Offset from the first band: both rects come from the same draw, so
        // the answer does not depend on where the page is scrolled now.
        let top = self.ui.widget(cx, ids!(hero)).area().rect(cx);
        if page_rect.size.y <= 0.0 {
            self.pending_section = Some(id);
            return 0.0;
        }
        let y = (target.pos.y - top.pos.y).max(0.0);
        page.set_scroll_pos(cx, dvec2(0.0, y));
        self.ui.redraw(cx);
        y
    }

    /// Paint each floor tile from its visit: one colour layer, the app's word,
    /// minutes and running bill.
    fn paint_floor(&mut self, cx: &mut Cx) {
        let tables = self.tables.unwrap_or(OPENING_FLOOR);
        for (i, ((path, amount), visit)) in TABLES.iter().zip(tables).enumerate() {
            let tile = self.ui.widget(cx, path);
            tile.label(cx, ids!(name)).set_text(cx, &format!("T{}", i + 1));
            for (i, layer) in TILE_LAYERS.iter().enumerate() {
                tile.view(cx, &[*layer]).set_visible(cx, i == visit.index());
            }
            tile.label(cx, ids!(word)).set_text(cx, visit.word());
            tile.label(cx, ids!(meta)).set_text(cx, &floor::meta(visit, *amount));
        }
        self.ui.redraw(cx);
    }

    fn handle_floor(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut tables = self.tables.unwrap_or(OPENING_FLOOR);
        let mut changed = false;
        for (i, (path, _)) in TABLES.iter().enumerate() {
            if self.ui.widget(cx, path).button(cx, ids!(hit)).clicked(actions) {
                tables[i] = tables[i].next();
                changed = true;
            }
        }
        if changed {
            self.tables = Some(tables);
            self.paint_floor(cx);
        }
    }

    fn set_cta_notes(&mut self, cx: &mut Cx, text: &str) {
        for path in [ids!(cta_note_text), ids!(close_note_text)] {
            self.ui.widget(cx, path).set_text(cx, text);
        }
        self.ui.redraw(cx);
    }

    fn download(&mut self, cx: &mut Cx) {
        if DOWNLOADS_LIVE {
            open_link(cx, DOWNLOAD_URL);
        } else {
            self.set_cta_notes(
                cx,
                "Installers go up here once each one passes a clean-machine install. Until then, ask for a demo on WhatsApp.",
            );
        }
    }

    fn whatsapp(&mut self, cx: &mut Cx) {
        match WHATSAPP_NUMBER {
            Some(number) => {
                let url = format!("https://wa.me/{number}?text={}", url_encode(WHATSAPP_MESSAGE));
                open_link(cx, &url);
            }
            None => self.set_cta_notes(
                cx,
                &format!("The WhatsApp demo line opens soon. Your message will read: “{WHATSAPP_MESSAGE}”."),
            ),
        }
    }

    fn kumkaj_url(&self, cx: &Cx) -> String {
        if let OsType::Web(page) = cx.os_type() {
            if page.hostname == "127.0.0.1" || page.hostname == "localhost" {
                return format!("{}//{}:8010/", page.protocol, page.hostname);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var("SITE_LINKS").as_deref() == Ok("local") {
            return "http://127.0.0.1:8010/".to_string();
        }
        "https://kumkaj.com/".to_string()
    }

    /// Same-host page (privacy, terms): relative in the browser, the public
    /// host natively.
    fn site_url(&self, cx: &Cx, path: &str) -> String {
        if matches!(cx.os_type(), OsType::Web(_)) {
            path.to_string()
        } else {
            format!("{PUBLIC_HOST}{path}")
        }
    }

    fn apply_gates(&mut self, cx: &mut Cx) {
        for path in [ids!(kot_lan), ids!(offline_lan)] {
            self.ui.widget(cx, path).set_visible(cx, LAN_PILOT_PASSED);
        }
    }

    fn section_from_page(&self, cx: &Cx) -> Option<&'static str> {
        let OsType::Web(page) = cx.os_type() else {
            return None;
        };
        section_from_hash(&page.hash)
    }

    /// Floor toy: three tiles a row when the card is wide enough, else two
    /// (T3 / T6 drop on phones), tiles sized to the card.
    fn layout_floor(&mut self, cx: &mut Cx, content_w: f64) {
        let card_w = if content_w >= 900.0 { 520.0 } else { content_w };
        let inner = card_w - 40.0;
        let per_row = if inner >= 3.0 * 112.0 + 16.0 { 3 } else { 2 };
        let tile_w = ((inner - (per_row as f64 - 1.0) * 8.0) / per_row as f64).clamp(100.0, 170.0);
        for (path, _) in TABLES {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.walk.width = Size::Fixed(tile_w);
                view.walk.height = Size::Fixed(108.0);
            }
        }
        for path in [ids!(floor.t3), ids!(floor.t6)] {
            self.ui.widget(cx, path).set_visible(cx, per_row == 3);
        }
    }

    fn apply_responsive(&mut self, cx: &mut Cx) {
        let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
        if size.x < 1.0 || size.y < 1.0 {
            return;
        }
        // Layout math never goes below MIN_WIDTH; web shell scrolls when the
        // device is narrower (see static/css/site-shell.css).
        let layout_w = size.x.max(MIN_WIDTH);
        let tier = Tier::from_size(layout_w, size.y);
        let key = (tier, layout_w.round() as i64, size.y.round() as i64);
        if self.laid_out == Some(key) {
            return;
        }
        let tier_changed = self.laid_out.map(|(t, _, _)| t) != Some(tier);
        self.laid_out = Some(key);

        let (min_side, pad_v) = tier.spacing();
        let side = min_side.max((layout_w - tier.max_content()) / 2.0);
        let content_w = layout_w - 2.0 * side;
        for path in WRAPS {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                let footer = path == ids!(footer_wrap);
                view.layout.padding = Inset {
                    left: side,
                    right: side,
                    top: if footer { 32.0 } else { pad_v },
                    bottom: if footer { 36.0 } else { pad_v },
                };
            }
        }
        for (path, min_w) in ROWS {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.layout.flow = if content_w >= min_w { Flow::right() } else { Flow::Down };
            }
        }
        for (path, desk_w, row_min_w) in PROPS {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.walk.width = if content_w >= row_min_w { Size::Fixed(desk_w) } else { Size::fill() };
            }
        }
        self.layout_floor(cx, content_w);
        // Top bar: section links on wide windows; tighter padding on phones.
        self.ui
            .widget(cx, ids!(nav_links))
            .set_visible(cx, layout_w >= NAV_LINKS_FROM);
        if let Some(mut bar) = self.ui.view(cx, ids!(top_bar)).borrow_mut() {
            let pad = if tier == Tier::Phone { 14.0 } else { 24.0 };
            bar.layout.padding = Inset { left: pad, right: pad.min(20.0), top: 0.0, bottom: 0.0 };
        }
        // World shelf page size follows the width; the first card stays put.
        let per_page = if content_w >= SHELF_WIDE { 8 } else { 4 };
        self.ui
            .view(cx, ids!(menus.shelf_q1))
            .set_visible(cx, per_page == 8);
        if per_page != self.shelf_per_page {
            self.shelf_per_page = per_page;
            self.paint_shelf(cx);
        }
        if tier_changed {
            let page = self.ui.widget(cx, ids!(page));
            if self.desk_font.is_empty() {
                collect_label_sizes(&page, &mut self.desk_font);
            }
            let mut next = 0;
            scale_labels(&page, tier, &self.desk_font, &mut next);
        }
        self.ui.redraw(cx);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_native_preview(&mut self, cx: &mut Cx) {
        let Ok(spec) = std::env::var("SITE_PREVIEW") else {
            return;
        };
        let (size, section) = parse_preview(&spec);
        if let Some((w, h)) = size {
            self.ui
                .window(cx, ids!(main_window))
                .resize(cx, dvec2(w, h));
        }
        if section.is_some() {
            self.pending_section = section;
            self.pending_frames = 120;
        self.pending_stable = 0;
        }
    }

    fn paint_shelf(&mut self, cx: &mut Cx) {
        let list = shelf_entries(self.shelf_all);
        let per = self.shelf_per_page.clamp(1, SHELF_CARDS.len());
        let (page, pages) = shelf_page_of(self.shelf_first, per, list.len());
        self.shelf_first = page * per;
        self.ui
            .label(cx, ids!(menus.shelf_page))
            .set_text(cx, &format!("{} / {}", page + 1, pages));
        let toggle = if self.shelf_all {
            format!("Only the {} with photos", world_photo_count())
        } else {
            format!("Show all {} countries", WORLD.len())
        };
        self.ui.button(cx, ids!(menus.shelf_toggle)).set_text(cx, &toggle);
        for (i, path) in SHELF_CARDS.iter().enumerate() {
            let entry = (i < per).then(|| list.get(self.shelf_first + i)).flatten();
            let card = self.ui.widget(cx, path);
            self.ui.view(cx, path).set_visible(cx, entry.is_some());
            let Some(d) = entry else { continue };
            card.widget(cx, ids!(country)).set_text(cx, d.country);
            let native = card.widget(cx, ids!(native));
            let line = shelf_native_line(d);
            native.set_text(cx, line.as_deref().unwrap_or(""));
            card.view(cx, ids!(native_wrap)).set_visible(cx, line.is_some());
            card.widget(cx, ids!(dish)).set_text(cx, d.dish);
            let photo = card.image(cx, ids!(photo));
            photo.set_visible(cx, d.photo.is_some());
            let soon = card.view(cx, ids!(soon));
            soon.set_visible(cx, d.photo.is_none());
            if let Some(file) = d.photo {
                self.show_world_photo(cx, &photo, file);
            }
        }
        self.ui.redraw(cx);
    }

    /// Put `file` into a card's Image: decode cached bytes now, else clear
    /// the slot (never the previous country's photo) and fetch the bytes.
    /// Synchronous WebP decode on purpose — Makepad's async image path
    /// decodes on a thread pool, which the `--no-threads` wasm build lacks
    /// (cards stayed black in the browser).
    fn show_world_photo(&mut self, cx: &mut Cx, image: &ImageRef, file: &'static str) {
        if let Some(bytes) = self.photo_bytes.get(file).cloned() {
            if let Err(err) = image.load_webp_from_data(cx, &bytes) {
                log!("khaynjo-site: world photo {file}: {err:?}");
            }
            return;
        }
        image.set_texture(cx, None);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/dishes/world")
                .join(file);
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let bytes = std::rc::Rc::new(bytes);
                    self.photo_bytes.insert(file.to_string(), bytes.clone());
                    let _ = image.load_webp_from_data(cx, &bytes);
                }
                Err(err) => log!("khaynjo-site: {}: {err}", path.display()),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if self.photo_pending.iter().any(|(_, f)| f == file) {
                return;
            }
            let pathname = match cx.os_type() {
                OsType::Web(page) => page.pathname.clone(),
                _ => String::new(),
            };
            let id = LiveId::unique();
            cx.http_request(
                id,
                HttpRequest::new(world_photo_web_path(&pathname, file), HttpMethod::GET),
            );
            self.photo_pending.push((id, file.to_string()));
        }
    }

    /// Fetched photo bytes land here; repaint the photos of the page shown.
    fn handle_photo_responses(&mut self, cx: &mut Cx, responses: &NetworkResponsesEvent) {
        let mut landed = false;
        for response in responses {
            let (request_id, body) = match response {
                NetworkResponse::HttpResponse { request_id, response } => (
                    *request_id,
                    (200..300)
                        .contains(&response.status_code)
                        .then(|| response.body.clone())
                        .flatten(),
                ),
                NetworkResponse::HttpError { request_id, .. } => (*request_id, None),
                _ => continue,
            };
            let Some(pos) = self.photo_pending.iter().position(|(id, _)| *id == request_id) else {
                continue;
            };
            let (_, file) = self.photo_pending.remove(pos);
            match body {
                Some(bytes) => {
                    self.photo_bytes.insert(file, std::rc::Rc::new(bytes));
                    landed = true;
                }
                None => log!("khaynjo-site: world photo {file} did not load"),
            }
        }
        if landed {
            self.paint_shelf(cx);
        }
    }

    fn shelf_step(&mut self, cx: &mut Cx, forward: bool) {
        let len = shelf_entries(self.shelf_all).len();
        let per = self.shelf_per_page.max(1);
        let (page, pages) = shelf_page_of(self.shelf_first, per, len);
        // Wraps at both ends — the shelf is a carousel, not a dead end.
        let next = if forward {
            (page + 1) % pages
        } else {
            (page + pages - 1) % pages
        };
        self.shelf_first = next * per;
        self.paint_shelf(cx);
    }


    /// Counts in the copy come from the data, never hand-typed.
    fn paint_world_counts(&mut self, cx: &mut Cx) {
        let n = WORLD.len();
        let photos = world_photo_count();
        self.ui.widget(cx, ids!(menus.menu_note_text)).set_text(
            cx,
            &format!(
                "{n} countries · {photos} public-domain / CC0 photos · \u{201C}Photo coming\u{201D} where we have none yet."
            ),
        );
    }
}

fn collect_label_sizes(widget: &WidgetRef, out: &mut Vec<f32>) {
    widget.children(&mut |_, child| {
        if let Some(label) = child.borrow::<Label>() {
            out.push(label.draw_text.text_style.font_size);
        } else if let Some(md) = child.borrow::<Markdown>() {
            out.push(md.draw_text.text_style.font_size);
        }
        collect_label_sizes(&child, out);
    });
}

fn scale_labels(widget: &WidgetRef, tier: Tier, desk: &[f32], next: &mut usize) {
    widget.children(&mut |_, child| {
        if let Some(mut label) = child.borrow_mut::<Label>() {
            if let Some(&base) = desk.get(*next) {
                label.draw_text.text_style.font_size = base * tier.type_scale(base);
            }
            *next += 1;
        } else if let Some(mut md) = child.borrow_mut::<Markdown>() {
            if let Some(&base) = desk.get(*next) {
                let scaled = base * tier.type_scale(base);
                md.font_size = scaled;
                md.draw_text.text_style.font_size = scaled;
                md.text_style_normal.font_size = scaled;
                md.text_style_bold.font_size = scaled;
                md.text_style_italic.font_size = scaled;
                md.text_style_bold_italic.font_size = scaled;
            }
            *next += 1;
        }
        scale_labels(&child, tier, desk, next);
    });
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn parse_preview(spec: &str) -> (Option<(f64, f64)>, Option<&'static str>) {
    let (size_part, section_part) = match spec.split_once(':') {
        Some((s, c)) => (s, Some(c)),
        None if spec.contains('x') => (spec, None),
        None => ("", Some(spec)),
    };
    let size = size_part.split_once('x').and_then(|(w, h)| {
        let w = w.trim().parse::<f64>().ok()?;
        let h = h.trim().parse::<f64>().ok()?;
        (w >= 200.0 && h >= 200.0).then_some((w, h))
    });
    let section = section_part.and_then(section_from_hash);
    (size, section)
}

fn url_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn open_link(cx: &mut Cx, url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        cx.open_url(url, OpenUrlInPlace::No);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = cx;
        #[cfg(target_os = "windows")]
        let spawned = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
        #[cfg(target_os = "macos")]
        let spawned = std::process::Command::new("open").arg(url).spawn();
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let spawned = std::process::Command::new("xdg-open").arg(url).spawn();
        if let Err(err) = spawned {
            log!("could not open {url}: {err}");
        }
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.apply_gates(cx);
        self.paint_floor(cx);
        self.paint_world_counts(cx);
        self.pending_section = self.section_from_page(cx);
        self.pending_frames = 120;
        self.pending_stable = 0;
        #[cfg(not(target_arch = "wasm32"))]
        self.apply_native_preview(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (path, section) in NAV_LINKS {
            if self.ui.button(cx, path).clicked(actions) {
                let _ = self.scroll_to(cx, section);
            }
        }
        if self.ui.button(cx, ids!(brand_hit)).clicked(actions) {
            let _ = self.scroll_to(cx, "top");
        }
        self.handle_floor(cx, actions);
        if self.ui.button(cx, ids!(menus.shelf_prev)).clicked(actions) {
            self.shelf_step(cx, false);
        }
        if self.ui.button(cx, ids!(menus.shelf_next)).clicked(actions) {
            self.shelf_step(cx, true);
        }
        if self.ui.button(cx, ids!(menus.shelf_toggle)).clicked(actions) {
            self.shelf_all = !self.shelf_all;
            self.shelf_first = 0;
            self.paint_shelf(cx);
        }
        let download = self.ui.button(cx, ids!(bar_cta)).clicked(actions)
            || self.ui.button(cx, ids!(hero_cta.download)).clicked(actions)
            || self.ui.button(cx, ids!(close_cta.download)).clicked(actions)
            || self.ui.button(cx, ids!(to_play)).clicked(actions);
        if download {
            self.download(cx);
        }
        let whatsapp = self.ui.button(cx, ids!(hero_cta.whatsapp)).clicked(actions)
            || self.ui.button(cx, ids!(close_cta.whatsapp)).clicked(actions);
        if whatsapp {
            self.whatsapp(cx);
        }
        if self.ui.button(cx, ids!(to_privacy)).clicked(actions) {
            let url = self.site_url(cx, PRIVACY_PATH);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(to_terms)).clicked(actions) {
            let url = self.site_url(cx, TERMS_PATH);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(to_kumkaj)).clicked(actions) {
            let url = self.kumkaj_url(cx);
            open_link(cx, &url);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::NetworkResponses(responses) = event {
            self.handle_photo_responses(cx, responses);
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        match event {
            Event::Startup | Event::WindowGeomChange(_) => self.apply_responsive(cx),
            Event::Draw(_) | Event::NextFrame(_) => {
                if self.laid_out.is_none() {
                    self.apply_responsive(cx);
                }
                // Deep link (#menus) once the page has drawn and has a size.
                if let Some(section) = self.pending_section {
                    let page_h = self.ui.view(cx, ids!(page)).area().rect(cx).size.y;
                    if page_h > 0.0 {
                        let y = self.scroll_to(cx, section);
                        self.pending_frames = self.pending_frames.saturating_sub(1);
                        if (y - self.pending_last_y).abs() < 0.5 {
                            self.pending_stable += 1;
                        } else {
                            self.pending_stable = 0;
                        }
                        self.pending_last_y = y;
                        if self.pending_frames == 0 || self.pending_stable >= 3 {
                            self.pending_section = None;
                        }
                    }
                    let _ = cx.new_next_frame();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_opens_a_section_or_an_old_chapter() {
        assert_eq!(section_from_hash("#menus"), Some("menus"));
        assert_eq!(section_from_hash("#FAQ"), Some("faq"));
        // Old deck links keep working.
        assert_eq!(section_from_hash("#kot"), Some("how"));
        assert_eq!(section_from_hash("#menu"), Some("menus"));
        assert_eq!(section_from_hash("#3"), Some("how"));
        assert_eq!(section_from_hash("#10"), Some("start"));
        assert_eq!(section_from_hash("#11"), None);
        assert_eq!(section_from_hash(""), None);
        // Every old chapter maps to a real section.
        for (_, to) in OLD_CHAPTERS {
            assert!(SECTIONS.iter().any(|(id, _)| *id == to), "{to}");
        }
        for (_, to) in NAV_LINKS {
            assert!(SECTIONS.iter().any(|(id, _)| *id == to), "{to}");
        }
    }

    #[test]
    fn preview_spec_parses_size_and_section() {
        assert_eq!(parse_preview("390x844:menus"), (Some((390.0, 844.0)), Some("menus")));
        assert_eq!(parse_preview("1280x800"), (Some((1280.0, 800.0)), None));
        assert_eq!(parse_preview("kot"), (None, Some("how")));
    }

    #[test]
    fn opening_floor_shows_every_colour_but_paid() {
        let mut idx: Vec<_> = OPENING_FLOOR.iter().map(|v| v.index()).collect();
        idx.sort();
        assert_eq!(idx, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(TILE_LAYERS.len(), Visit::ALL.len());
    }

    #[test]
    fn owner_switches_match_shipped_state() {
        assert!(WHATSAPP_NUMBER.is_none());
        assert!(DOWNLOADS_LIVE);
        assert!(DOWNLOAD_URL.contains("play.google.com/store/apps/details?id=com.khaynjo"));
        assert!(!LAN_PILOT_PASSED);
    }

    #[test]
    fn layout_floor_is_phone_width() {
        assert_eq!(MIN_WIDTH, 375.0);
        assert!(Tier::from_size(MIN_WIDTH, 800.0) == Tier::Phone);
    }

    #[test]
    fn whatsapp_prefill_is_percent_encoded() {
        assert_eq!(
            url_encode(WHATSAPP_MESSAGE),
            "Khaynjo%20demo%20%E2%80%94%20restaurant%20type%2C%20tables%2C%20city"
        );
    }

    fn workspace(rel: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
    }

    /// The Menus section quotes the public catalogue — keep the numbers true.
    #[test]
    fn template_cards_match_the_public_catalogue() {
        let ui = include_str!("ui.rs");
        let cdn = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("static/v1/catalogue/public");
        let index = std::fs::read_to_string(cdn.join("index.json")).unwrap();
        for id in ["khaynjo-starter", "cafe-starter", "khaynjo-items"] {
            assert!(index.contains(&format!("\"id\": \"{id}\"")), "{id} in public/index.json");
        }
        // No private pack is ever named on the public page.
        assert!(!ui.to_lowercase().contains("rajdhani"));
        let count = |rel: &str, needle: &str| {
            std::fs::read_to_string(cdn.join(rel)).unwrap().matches(needle).count()
        };
        let starter_dishes = count("templates/khaynjo-starter/template.json", "\"price\"");
        let starter_cats = count("templates/khaynjo-starter/template.json", "\"items\"");
        let cafe_dishes = count("templates/cafe-starter/template.json", "\"price\"");
        let cafe_cats = count("templates/cafe-starter/template.json", "\"items\"");
        assert!(ui.contains(&format!("{starter_dishes} dishes · {starter_cats} categories")), "starter {starter_dishes}/{starter_cats}");
        assert!(ui.contains(&format!("{cafe_dishes} dishes · {cafe_cats} categories")), "cafe {cafe_dishes}/{cafe_cats}");
        // Shelf photo count comes from the items index (synced at publish).
        if let Ok(items) = std::fs::read_to_string(cdn.join("items/index.json")) {
            let n: usize = items
                .split("\"count\":")
                .nth(1)
                .and_then(|s| s.trim_start().split(|c: char| !c.is_ascii_digit()).next())
                .and_then(|s| s.parse().ok())
                .unwrap();
            let shown = format!("{},{:03} dish photos", n / 1000, n % 1000);
            assert!(ui.contains(&shown), "shelf shows {shown}");
        }
    }

    /// "115 languages" is the app's own catalogue size.
    #[test]
    fn language_count_matches_the_app() {
        let locale = std::fs::read_to_string(workspace("apps/khaynjo/src/locale.rs")).unwrap();
        let block = locale
            .split("pub const LANGUAGES: &[LanguageInfo] = &[")
            .nth(1)
            .and_then(|s| s.split("\n];").next())
            .unwrap();
        let n = block.matches("LanguageInfo {").count();
        let ui = include_str!("ui.rs");
        assert!(ui.contains(&format!("{n} languages")), "site says {n} languages");
    }

    #[test]
    fn world_shelf_data_is_complete_and_photos_exist() {
        // 193 UN members + 6 extras from the research pack.
        assert_eq!(WORLD.len(), 199);
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/dishes/world");
        let mut seen = std::collections::HashSet::new();
        for d in WORLD {
            assert!(!d.country.trim().is_empty() && !d.dish.trim().is_empty());
            assert!(!d.country_native.trim().is_empty() && !d.dish_native.trim().is_empty(), "{}", d.country);
            assert!(seen.insert(d.country), "duplicate {}", d.country);
            if let Some(file) = d.photo {
                assert!(dir.join(file).is_file(), "{file} shipped");
            }
        }
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".webp") {
                assert!(WORLD.iter().any(|d| d.photo == Some(name.as_str())), "orphan {name}");
            }
        }
        assert!(world_photo_count() >= 60, "most of the shelf has photos");
    }

    #[test]
    fn shelf_pages_cover_every_country_once() {
        for all in [false, true] {
            let list = shelf_entries(all);
            for per in [4, 8] {
                let (_, pages) = shelf_page_of(0, per, list.len());
                let mut shown = 0;
                for p in 0..pages {
                    let first = p * per;
                    assert_eq!(shelf_page_of(first, per, list.len()).0, p);
                    shown += list.len().saturating_sub(first).min(per);
                }
                assert_eq!(shown, list.len());
            }
        }
        assert_eq!(shelf_page_of(10_000, 8, 199), (24, 25));
    }

    #[test]
    fn world_photo_web_path_follows_page_path() {
        assert_eq!(
            world_photo_web_path("/", "india.webp"),
            "/khaynjo_site/resources/dishes/world/india.webp"
        );
        assert_eq!(
            world_photo_web_path("/preview/", "india.webp"),
            "/preview/khaynjo_site/resources/dishes/world/india.webp"
        );
    }

    /// Icons on the page are the app's own sidebar art, shipped with the site.
    #[test]
    fn feature_icons_are_shipped() {
        let ui = include_str!("ui.rs");
        for part in ui.split("self:resources/icons/").skip(1) {
            let file = part.split('"').next().unwrap();
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/icons").join(file);
            assert!(path.is_file(), "{file}");
        }
    }
}

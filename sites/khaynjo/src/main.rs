//! khaynjo.com — Khaynjo's site as a fullpage Makepad 2 app (wasm in the
//! browser; the same crate runs native for previews and as an offline
//! presenter in a restaurant). Story: apps/khaynjo/docs/KHAYNJO_WEBSITE.md.
//! Copied from sites/kumkaj, then diverged (copy-then-diverge; no shared
//! site engine).
//!
//! One scrolling page (redesign 2026-11-12; was a 10-slide deck): sticky
//! top bar with section links and the Play CTA; sections hero · menus ·
//! how · features · offline · pricing · faq · closing · footer.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:menus` opens a 390×844 window scrolled to a
//!   section (id, old chapter id, or nothing for the top).
//! - `SITE_SHELF_QUERY=Antigua` fills the world-shelf search (with preview).
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

mod locale;

#[path = "../../_shared/i18n/site_i18n.rs"]
mod site_i18n;
use site_i18n::{ScriptGroup, SiteLang};

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
const CANCELLATION_PATH: &str = "/cancellation";
const PHOTO_CREDITS_PATH: &str = "/photo-credits";
const SHOP_PATH: &str = "/shop";
const ACCOUNT_PATH: &str = "/account";
const PUBLIC_HOST: &str = "https://khaynjo.com";

// ---- Sections ----------------------------------------------------------

/// Page sections in order: (hash id, widget path).
const SECTIONS: [(&str, &[LiveId]); 8] = [
    ("top", ids!(hero)),
    ("menus", ids!(menus)),
    ("how", ids!(how)),
    ("features", ids!(features)),
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
    (ids!(go_menus), "menus"),
    (ids!(go_how), "how"),
    (ids!(go_features), "features"),
    (ids!(go_offline), "offline"),
    (ids!(go_pricing), "pricing"),
    (ids!(go_faq), "faq"),
];

/// Hidden until first-paint warmup ends — keeps Mali-T830 WebGL from compiling
/// the whole long page + shelf Images + WorldFont in one wave (blank cream).
const DEFERRED_FIRST_PAINT: [&[LiveId]; 9] = [
    ids!(floor_card),
    ids!(menus),
    ids!(how),
    ids!(features),
    ids!(menu_starters),
    ids!(offline),
    ids!(pricing),
    ids!(faq),
    ids!(closing),
];

/// Each band's content column; side padding / max width set per tier.
const WRAPS: [&[LiveId]; 10] = [
    ids!(hero_wrap),
    ids!(menus_wrap),
    ids!(how_wrap),
    ids!(features_wrap),
    ids!(menu_starters_wrap),
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

/// App layout floor: design for ≥320 CSS px (360 is the common Android phone width). Narrower viewports scroll
/// horizontally (web shell CSS); native still lays out at this width.
const MIN_WIDTH: f64 = 320.0;

/// Rows that sit side by side only when the content column is at least
/// this wide (px), and stack otherwise.
/// Do **not** list shelf PagerBtns here — under ~520 px that turns each
/// chip into its own row (phone 2026-11-20). `shelf_bar` is one ScrollX
/// strip of Fit chips (play/pause icons + speed number).
const ROWS: [(&[LiveId], f64); 22] = [
    (ids!(hero_row), 900.0),
    // Both labels ("Get it on Google Play" + "WhatsApp for a demo") need
    // about 480 px plus slack; 420 left them overlapping on a narrow phone.
    (ids!(hero_cta), 560.0),
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
    (ids!(offline_row), 860.0),
    (ids!(price_row), 720.0),
    (ids!(faq_r0), 720.0),
    (ids!(faq_r1), 720.0),
    (ids!(faq_r2), 720.0),
    (ids!(close_cta), 560.0),
    // Footer brand+links stack in ui.rs (flow Down). Links stay one row until phone.
    (ids!(footer_links), 720.0),
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
/// Top-bar section links (six labels + brand + Get it free) need this width
/// or the CTA is cropped on the right (desk evidence 2026-11-22 at ~1100–1200).
const NAV_LINKS_FROM: f64 = 1280.0;

const MAX_CONTENT: f64 = 1160.0;
const MAX_CONTENT_WIDE: f64 = 1280.0;

/// Fixed card width + gap for the world parade (must match ShelfSlot / spacing).
const SHELF_CARD_W: f64 = 200.0;
const SHELF_GAP: f64 = 12.0;
const SHELF_PITCH: f64 = SHELF_CARD_W + SHELF_GAP;
/// Left inset so a paused/reset strip shows a full first card with a clear
/// right-edge peek of the next — not a flush crop that reads as a layout bug
/// on a phone (evidence 2026-11-22). 16 px keeps the peek off the content edge.
const SHELF_EDGE_INSET: f64 = 16.0;
/// Parade speed: level 0 = still, then +SHELF_SPEED_STEP px/s per click.
/// Cap is high so a long press of + can race the strip; default stays gentle.
const SHELF_SPEED_STEP: f64 = 40.0;
const SHELF_SPEED_MAX_LEVEL: u8 = 24;
const SHELF_SPEED_DEFAULT_LEVEL: u8 = 2;
/// Parade slots (abs-positioned shells) and the card path under each.
const SHELF_SLOTS: [&[LiveId]; 10] = [
    ids!(menus.s0),
    ids!(menus.s1),
    ids!(menus.s2),
    ids!(menus.s3),
    ids!(menus.s4),
    ids!(menus.s5),
    ids!(menus.s6),
    ids!(menus.s7),
    ids!(menus.s8),
    ids!(menus.s9),
];
const SHELF_CARDS: [&[LiveId]; 10] = [
    ids!(menus.s0.card),
    ids!(menus.s1.card),
    ids!(menus.s2.card),
    ids!(menus.s3.card),
    ids!(menus.s4.card),
    ids!(menus.s5.card),
    ids!(menus.s6.card),
    ids!(menus.s7.card),
    ids!(menus.s8.card),
    ids!(menus.s9.card),
];
const SHELF_SLOT_COUNT: usize = 10;
const SHELF_TRACK: f64 = SHELF_PITCH * SHELF_SLOT_COUNT as f64;

/// Width of one line of `text` in `draw_text`'s font and size (logical px).
fn text_width(cx: &mut Cx, draw_text: &DrawText, text: &str) -> f64 {
    draw_text
        .layout(cx, 0.0, 0.0, None, false, Align::default(), text)
        .size_in_lpxs
        .width as f64
}

/// Per widget: the font size the page asked for and the one a fit pass set.
/// A current size that is neither means `scale_labels` set a new base.
type FitSizes = std::collections::HashMap<WidgetUid, (f32, f32)>;

/// Smallest a fit pass shrinks text, as a share of the size asked for.
const FIT_MIN_SCALE: f32 = 0.7;

fn fit_base(sizes: &FitSizes, uid: WidgetUid, current: f32) -> f32 {
    match sizes.get(&uid) {
        Some(&(base, applied)) if (applied - current).abs() < 0.01 => base,
        _ => current,
    }
}

fn fit_size(base: f32, need: f64, avail: f64) -> f32 {
    if need <= avail || need <= 0.0 || avail <= 0.0 {
        base
    } else {
        (base * (avail / need) as f32).max(base * FIT_MIN_SCALE)
    }
}

/// Shrink a Markdown until its text — or, `by_word`, its widest word — fits
/// `avail` px on one line. True when the size changed.
fn fit_markdown(cx: &mut Cx, w: &WidgetRef, avail: f64, by_word: bool, sizes: &mut FitSizes) -> bool {
    let text = w.text();
    let uid = w.widget_uid();
    let Some(mut md) = w.borrow_mut::<Markdown>() else {
        return false;
    };
    let current = md.text_style_normal.font_size;
    let base = fit_base(sizes, uid, current);
    // Plain body text draws with the normal style; measure in that face.
    let saved = md.draw_text.text_style.clone();
    md.draw_text.text_style = md.text_style_normal.clone();
    md.draw_text.text_style.font_size = base;
    let need = if by_word {
        text.split_whitespace()
            .map(|word| text_width(cx, &md.draw_text, word))
            .fold(0.0, f64::max)
    } else {
        text_width(cx, &md.draw_text, &text)
    };
    md.draw_text.text_style = saved;
    let size = fit_size(base, need, avail);
    sizes.insert(uid, (base, size));
    if (size - current).abs() < 0.01 {
        return false;
    }
    md.font_size = size;
    md.draw_text.text_style.font_size = size;
    md.text_style_normal.font_size = size;
    md.text_style_bold.font_size = size;
    md.text_style_italic.font_size = size;
    md.text_style_bold_italic.font_size = size;
    true
}

/// Shrink a Button's label until it fits `avail` px. True when it changed.
fn fit_button(cx: &mut Cx, w: &WidgetRef, avail: f64, sizes: &mut FitSizes) -> bool {
    let text = w.text();
    let uid = w.widget_uid();
    let Some(mut b) = w.borrow_mut::<Button>() else {
        return false;
    };
    let current = b.draw_text.text_style.font_size;
    let base = fit_base(sizes, uid, current);
    b.draw_text.text_style.font_size = base;
    let need = text_width(cx, &b.draw_text, &text);
    let size = fit_size(base, need, avail);
    b.draw_text.text_style.font_size = size;
    sizes.insert(uid, (base, size));
    (size - current).abs() >= 0.01
}

/// Countries on the shelf that have a photo.
#[cfg(test)]
fn world_photo_count() -> usize {
    WORLD.iter().filter(|d| d.photo.is_some()).count()
}

/// Countries matching a search (country / native / dish); empty query = all.
fn shelf_filter_hits(query: &str) -> Vec<&'static WorldDish> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return WORLD.iter().collect();
    }
    WORLD
        .iter()
        .filter(|d| {
            d.country.to_lowercase().contains(&q)
                || d.country_native.to_lowercase().contains(&q)
                || d.dish.to_lowercase().contains(&q)
                || d.dish_native.to_lowercase().contains(&q)
        })
        .collect()
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


fn shelf_wrap_index(i: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let n = len as isize;
    (((i % n) + n) % n) as usize
}

/// Page-path prefix for Makepad package URLs (same rule as `crate_resource`).
fn package_web_base(pathname: &str) -> String {
    let path = pathname.trim_end_matches('/');
    let last = path.rsplit('/').next().unwrap_or_default();
    let base = if last.contains('.') {
        path.rsplit_once('/').map_or("", |(b, _)| b)
    } else {
        path
    };
    base.trim_matches('/').to_string()
}

/// Web path of a world photo inside the Makepad package
/// (`<base>/khaynjo_site/resources/dishes/world/<file>`), where `<base>` is
/// the page path without a trailing file name — same rule Makepad uses for
/// `crate_resource` so a sub-path deploy keeps working.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn world_photo_web_path(pathname: &str, file: &str) -> String {
    let base = package_web_base(pathname);
    if base.is_empty() {
        format!("/khaynjo_site/resources/dishes/world/{file}")
    } else {
        format!("/{base}/khaynjo_site/resources/dishes/world/{file}")
    }
}

/// Feature-card icons (`resources/icons/<file>`) — fetched after first paint.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn feature_icon_web_path(pathname: &str, file: &str) -> String {
    let base = package_web_base(pathname);
    if base.is_empty() {
        format!("/khaynjo_site/resources/icons/{file}")
    } else {
        format!("/{base}/khaynjo_site/resources/icons/{file}")
    }
}

/// Feature cards → sidebar icon file (bound over HTTP after shelf warmup).
const FEATURE_ICONS: [(&[LiveId], &str); 8] = [
    (
        ids!(features.features_wrap.feat_grid.feat_r0.feat_floor.icon),
        "floor.webp",
    ),
    (
        ids!(features.features_wrap.feat_grid.feat_r0.feat_kitchen.icon),
        "kitchen.webp",
    ),
    (
        ids!(features.features_wrap.feat_grid.feat_r1.feat_sales.icon),
        "sales.webp",
    ),
    (
        ids!(features.features_wrap.feat_grid.feat_r1.feat_team.icon),
        "team.webp",
    ),
    (
        ids!(features.features_wrap.feat_grid.feat_r2.feat_menu.icon),
        "menu.webp",
    ),
    (
        ids!(features.features_wrap.feat_grid.feat_r2.feat_rates.icon),
        "rates.webp",
    ),
    (
        ids!(features.features_wrap.feat_grid.feat_r3.feat_credit.icon),
        "credit.webp",
    ),
    (
        ids!(features.features_wrap.feat_grid.feat_r3.feat_reports.icon),
        "reports.webp",
    ),
];

/// Hash segments (`#menus`, `#auth=LO`, `#menus&auth=LO`). `auth=` is the
/// signed-in chip bridge from firebase-init (Makepad cannot use web-sys).
fn hash_segments(hash: &str) -> impl Iterator<Item = &str> {
    hash.trim_start_matches('#')
        .split('&')
        .map(str::trim)
        .filter(|p| !p.is_empty())
}

/// Section id for a URL hash: a section id, an old chapter id, or an old
/// chapter number (`#3`). Unknown → None (stay at the top).
fn section_from_hash(hash: &str) -> Option<&'static str> {
    for part in hash_segments(hash) {
        // `#l=hi` (language), `#auth=LO` (signed-in chip): not sections.
        if site_i18n::is_bridge_segment(part) {
            continue;
        }
        let key = part.to_ascii_lowercase();
        if let Some((id, _)) = SECTIONS.iter().find(|(id, _)| *id == key) {
            return Some(id);
        }
        if let Some((_, to)) = OLD_CHAPTERS.iter().find(|(old, _)| *old == key) {
            return Some(to);
        }
        if let Ok(n) = key.parse::<usize>() {
            if let Some((_, to)) = OLD_CHAPTERS.get(n.checked_sub(1)?) {
                return Some(to);
            }
        }
    }
    None
}

/// Initials / label from `#auth=XX` (set by firebase-init.js).
fn auth_label_from_hash(hash: &str) -> Option<String> {
    for part in hash_segments(hash) {
        let Some(raw) = part
            .strip_prefix("auth=")
            .or_else(|| part.strip_prefix("AUTH="))
        else {
            continue;
        };
        let label = percent_decode(raw);
        let label = label.trim().to_string();
        if !label.is_empty() {
            return Some(label);
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = |c: u8| -> Option<u8> {
                match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                }
            };
            if let (Some(a), Some(b)) = (h(bytes[i + 1]), h(bytes[i + 2])) {
                out.push((a << 4) | b);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
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

/// Script derive wants plain field type names (not inline generics / arrays).
type PhotoBytes = std::collections::HashMap<String, std::rc::Rc<Vec<u8>>>;
type PendingSection = Option<&'static str>;
type PhotoPending = Vec<(LiveId, String)>;
type ShelfHits = Vec<&'static WorldDish>;
type ShelfSlotTags = [Option<&'static str>; 10];
type ShelfSlotXs = [f64; 10];
type ShelfSlotItems = [usize; 10];
/// Deferred photo binds: (slot, file) — one decode per frame keeps motion smooth.
type ShelfPhotoQueue = Vec<(usize, &'static str)>;

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
    /// Floor tiles are under 150 px wide: minutes and bill on two lines.
    #[rust]
    floor_narrow: bool,
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
    /// Native-only: apply shelf search once the menus section has settled.
    #[rust]
    pending_shelf_query: Option<String>,
    /// Topbar Login chip label from Firebase session (`khaynjo.auth.chip` in localStorage).
    #[rust]
    auth_chip_label: Option<String>,
    /// Frames left to re-read the auth chip while firebase-init settles.
    #[rust]
    auth_chip_poll: u8,
    /// Top-bar parts shown by `fit_top_bar` (links, wordmark, CTA, Login).
    #[rust]
    bar_fit: Option<[bool; 4]>,
    /// Font sizes `fit_hero_lines` works from and set (see `FitSizes`).
    #[rust]
    fit_sizes: FitSizes,
    /// Playing the continuous flow (search always forces pause).
    #[rust]
    shelf_playing: bool,
    /// Last play/pause choice to restore when search is cleared.
    #[rust]
    shelf_want_playing: bool,
    /// True = world flows left (default); false = flows right.
    #[rust]
    shelf_flow_left: bool,
    /// Speed level 0..=SHELF_SPEED_MAX_LEVEL (0 = no motion).
    #[rust]
    shelf_speed_level: u8,
    /// Active search text (empty = full world).
    #[rust]
    shelf_query: String,
    /// Cached hits for the current query — rebuilt only when search changes.
    #[rust]
    shelf_hits: ShelfHits,
    /// Per-slot X (abs) and hit index — each card wraps alone.
    #[rust]
    shelf_slot_x: ShelfSlotXs,
    #[rust]
    shelf_slot_item: ShelfSlotItems,
    /// Next hit index to assign when a card wraps off the left / right.
    #[rust]
    shelf_next_item: usize,
    #[rust]
    shelf_prev_item: usize,
    /// Country currently painted in each parade slot (skip photo reload if same).
    #[rust]
    shelf_slot_country: ShelfSlotTags,
    /// Photo file already decoded into that slot (avoid re-decode on backspace).
    #[rust]
    shelf_slot_photo: ShelfSlotTags,
    #[rust]
    shelf_photo_queue: ShelfPhotoQueue,
    /// Search text changed — wait for typing to settle before painting cards.
    #[rust]
    shelf_search_dirty: bool,
    /// NextFrame time when a dirty search may paint (0 = arm on next tick).
    #[rust]
    shelf_search_quiet_until: f64,
    #[rust]
    shelf_anim_frame: NextFrame,
    #[rust]
    shelf_last_t: f64,
    /// Frames left before revealing deferred bands + first `paint_shelf`
    /// (Mali-T830 first-wave budget: hero copy only, then the rest).
    #[rust]
    shelf_warmup_frames: u8,
    /// Top-bar nav section currently in view (scroll spy).
    #[rust]
    nav_active: PendingSection,
    /// World photo bytes by file (fetched once, decoded on demand) and the
    /// HTTP requests still in flight (wasm).
    #[rust]
    photo_bytes: PhotoBytes,
    #[rust]
    photo_pending: PhotoPending,
    /// Language the page shows (`#l=` from lang-nav.js), its packs and which
    /// widget shows which `locale::EN` key.
    #[rust]
    lang: SiteLang,
    /// CTA note currently shown under the buttons (repainted on a language change).
    #[rust]
    cta_note_key: Option<String>,
}

/// Font files that draw a script's copy (crate-relative) — the `Ui*`
/// families in `ui.rs`. `locale::tests::every_translation_draws_with_the_site_fonts`
/// checks every pack against them. Latin languages keep each widget's own
/// family (Baloo / Plex, with Plex as the fallback everywhere).
pub fn script_font_files(group: ScriptGroup) -> Vec<&'static str> {
    const PLEX: &str = "resources/fonts/IBMPlexSans-Text.ttf";
    let script = match group {
        ScriptGroup::Latin | ScriptGroup::Cyrillic | ScriptGroup::Greek => None,
        ScriptGroup::Devanagari => Some("resources/fonts/NotoSansDevanagari-Regular.ttf"),
        ScriptGroup::Bengali => Some("resources/fonts/NotoSansBengali-Regular.ttf"),
        ScriptGroup::Gurmukhi => Some("resources/fonts/NotoSansGurmukhi-Regular.ttf"),
        ScriptGroup::Gujarati => Some("resources/fonts/NotoSansGujarati-Regular.ttf"),
        ScriptGroup::Oriya => Some("resources/fonts/NotoSansOriya-Regular.ttf"),
        ScriptGroup::Tamil => Some("resources/fonts/NotoSansTamil-Regular.ttf"),
        ScriptGroup::Telugu => Some("resources/fonts/NotoSansTelugu-Regular.ttf"),
        ScriptGroup::Kannada => Some("resources/fonts/NotoSansKannada-Regular.ttf"),
        ScriptGroup::Malayalam => Some("resources/fonts/NotoSansMalayalam-Regular.ttf"),
        ScriptGroup::Sinhala => Some("resources/fonts/NotoSansSinhala-Regular.ttf"),
        ScriptGroup::Arabic => Some("resources/fonts/NotoSansArabic-Regular.ttf"),
        ScriptGroup::Hebrew => Some("resources/fonts/NotoSansHebrew-Regular.ttf"),
        ScriptGroup::Thaana => Some("resources/fonts/NotoSansThaana-Regular.ttf"),
        ScriptGroup::Tibetan => Some("resources/fonts/NotoSerifTibetan-Regular.ttf"),
        ScriptGroup::OlChiki => Some("resources/fonts/NotoSansOlChiki-Regular.ttf"),
        ScriptGroup::MeeteiMayek => Some("resources/fonts/NotoSansMeeteiMayek-Regular.ttf"),
        ScriptGroup::Thai => Some("resources/fonts/NotoSansThai-Regular.ttf"),
        ScriptGroup::Lao => Some("resources/fonts/NotoSansLao-Regular.ttf"),
        ScriptGroup::Khmer => Some("resources/fonts/NotoSansKhmer-Regular.ttf"),
        ScriptGroup::Myanmar => Some("resources/fonts/NotoSansMyanmar-Regular.ttf"),
        ScriptGroup::Ethiopic => Some("resources/fonts/NotoSansEthiopic-Regular.ttf"),
        ScriptGroup::Armenian => Some("resources/fonts/NotoSansArmenian-Regular.ttf"),
        ScriptGroup::Georgian => Some("resources/fonts/NotoSansGeorgian-Regular.ttf"),
        ScriptGroup::CjkSc => Some("resources/fonts/SiteCJK-SC.otf"),
        ScriptGroup::CjkTc => Some("resources/fonts/SiteCJK-TC.otf"),
        ScriptGroup::CjkJp => Some("resources/fonts/SiteCJK-JP.otf"),
        ScriptGroup::CjkKr => Some("resources/fonts/SiteCJK-KR.otf"),
        ScriptGroup::LatinExt => Some("resources/fonts/NotoSans-Regular.ttf"),
    };
    let mut files = vec![PLEX];
    files.extend(script);
    files
}

/// Labels the page paints from code (their words come from `App::tr`); they
/// take the script font / RTL alignment of the shown language too.
const CODE_TEXT: &[&[LiveId]] = &[
    ids!(cta_note_text),
    ids!(close_note_text),
    ids!(menus.menu_note_text),
    ids!(go_login),
];

// ---- Language (sites/_shared/i18n/site_i18n.rs) -------------------------

impl App {
    /// Site copy for `key` in the language the page shows.
    fn tr(&self, key: &str) -> String {
        self.lang.text(key, locale::en(key))
    }

    /// Language named by the URL hash (`#l=hi`, written by lang-nav.js), or
    /// `SITE_LANG=hi` for a native preview.
    fn hash_lang(cx: &Cx) -> Option<&'static str> {
        if let OsType::Web(page) = cx.os_type() {
            return site_i18n::lang_from_hash(&page.hash);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::env::var("SITE_LANG")
                .ok()
                .and_then(|c| site_i18n::language_by_code(&c))
                .map(|l| l.code)
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }

    /// Startup / hashchange: follow the picker's choice.
    fn sync_lang(&mut self, cx: &mut Cx) {
        let want = Self::hash_lang(cx).unwrap_or("en");
        if self.lang.last_hash_lang.as_deref() == Some(want) {
            return;
        }
        self.lang.last_hash_lang = Some(want.to_string());
        if let Some(table) = self.lang.want(want) {
            self.fetch_pack(cx, table);
        } else {
            self.apply_lang(cx);
        }
        self.paint_lang_chip(cx);
    }

    /// `/i18n/<code>.tsv` next to the wasm (staged from `translations/`);
    /// native reads the crate's file.
    fn fetch_pack(&mut self, cx: &mut Cx, table: String) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("translations")
                .join(format!("{table}.tsv"));
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    self.lang.insert_pack(&table, &text);
                    self.apply_lang(cx);
                }
                Err(err) => log!("khaynjo-site: {}: {err}", path.display()),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let id = LiveId::unique();
            cx.http_request(
                id,
                HttpRequest::new(site_i18n::pack_web_path(&table), HttpMethod::GET),
            );
            self.lang.set_pending(id, table);
        }
    }

    fn handle_lang_responses(&mut self, cx: &mut Cx, responses: &NetworkResponsesEvent) {
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
            if self.lang.take_response(request_id, body.as_deref()) {
                self.apply_lang(cx);
            }
        }
    }

    /// Show the wanted language everywhere once its pack is here.
    fn apply_lang(&mut self, cx: &mut Cx) {
        let lang = self.lang.wanted.clone();
        if lang.is_empty() || !self.lang.has_pack(&lang) {
            return;
        }
        let font = site_i18n::script_family(cx, &self.ui, site_i18n::script_group(&lang));
        let rtl = site_i18n::is_rtl(&lang);
        let skip = std::collections::HashSet::new();
        let ui = self.ui.clone();
        self.lang
            .apply(cx, &ui, &lang, &locale::key_for, &locale::en, font.as_ref(), &skip);
        // Text painted from code: new words, and the script font / alignment.
        for path in CODE_TEXT {
            let w = self.ui.widget(cx, path);
            self.lang.style_widget(&w, font.as_ref(), rtl);
        }
        for (path, _) in TABLES {
            let tile = self.ui.widget(cx, path);
            for part in [ids!(word), ids!(meta)] {
                let w = tile.widget(cx, part);
                self.lang.style_widget(&w, font.as_ref(), rtl);
            }
        }
        self.paint_floor(cx);
        self.paint_shelf_note(cx);
        if let Some(key) = self.cta_note_key.clone() {
            self.show_cta_note(cx, &key);
        }
        let label = self
            .auth_chip_label
            .clone()
            .unwrap_or_else(|| self.tr("login"));
        self.ui.button(cx, ids!(go_login)).set_text(cx, &label);
        self.paint_lang_chip(cx);
        self.fit_top_bar(cx);
        self.ui.redraw(cx);
    }

    /// Top-bar language chip shows the language in use (`HI`, `UR`, `EN`).
    fn paint_lang_chip(&mut self, cx: &mut Cx) {
        let code = if self.lang.wanted.is_empty() {
            "en".to_string()
        } else {
            self.lang.wanted.clone()
        };
        self.ui
            .button(cx, ids!(go_lang))
            .set_text(cx, &code.to_uppercase());
    }

    /// Note under the world shelf: no match / searching / the parade.
    fn paint_shelf_note(&mut self, cx: &mut Cx) {
        let key = if self.shelf_hits.is_empty() && !self.shelf_query.trim().is_empty() {
            "shelf_no_match"
        } else if !self.shelf_query.trim().is_empty() {
            "shelf_search_note"
        } else {
            "one_famous_dish_per_country"
        };
        let text = self.tr(key);
        self.ui
            .widget(cx, ids!(menus.menu_note_text))
            .set_text(cx, &text);
    }

    /// Floor tile second line: minutes, minutes + running bill, or paid.
    fn floor_meta(&self, visit: Visit, amount: u32) -> String {
        if self.lang.is_english() {
            return floor::meta(visit, amount);
        }
        let rupees = floor::rupees(amount);
        match (visit, visit.minutes()) {
            (Visit::Free, _) => String::new(),
            (Visit::Paid, _) => self.tr("floor_meta_paid").replace("{amount}", &rupees),
            (Visit::Occupied, Some(min)) => {
                self.tr("floor_meta_min").replace("{min}", &min.to_string())
            }
            (_, Some(min)) => self
                .tr("floor_meta_min_bill")
                .replace("{min}", &min.to_string())
                .replace("{amount}", &rupees),
            (_, None) => String::new(),
        }
    }
}

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
        self.paint_nav_active(cx);
        self.ui.redraw(cx);
        y
    }

    /// Paint each floor tile from its visit: one colour layer, the app's word,
    /// minutes and running bill.
    fn paint_floor(&mut self, cx: &mut Cx) {
        let tables = self.tables.unwrap_or(OPENING_FLOOR);
        for (i, ((path, amount), visit)) in TABLES.iter().zip(tables).enumerate() {
            let tile = self.ui.widget(cx, path);
            tile.widget(cx, ids!(name)).set_text(cx, &format!("T{}", i + 1));
            for (i, layer) in TILE_LAYERS.iter().enumerate() {
                tile.view(cx, &[*layer]).set_visible(cx, i == visit.index());
            }
            let word = locale::key_for(visit.word())
                .map(|k| self.tr(k))
                .unwrap_or_else(|| visit.word().to_string());
            tile.widget(cx, ids!(word)).set_text(cx, &word);
            // A narrow tile cannot hold "26 min · ₹ 1,560" on one line (it was
            // cut at the tile edge on 360 px phones): minutes, then the bill.
            let meta = self.floor_meta(visit, *amount);
            let meta = if self.floor_narrow {
                meta.replace(" · ", "\n\n")
            } else {
                meta
            };
            tile.widget(cx, ids!(meta)).set_text(cx, &meta);
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

    /// Note under both CTA rows (`locale::EN` key; repainted on a language change).
    fn show_cta_note(&mut self, cx: &mut Cx, key: &str) {
        self.cta_note_key = Some(key.to_string());
        let text = self.tr(key).replace("{message}", WHATSAPP_MESSAGE);
        for path in [ids!(cta_note_text), ids!(close_note_text)] {
            self.ui.widget(cx, path).set_text(cx, &text);
        }
        self.ui.redraw(cx);
    }

    fn download(&mut self, cx: &mut Cx) {
        if DOWNLOADS_LIVE {
            open_link(cx, DOWNLOAD_URL);
        } else {
            self.show_cta_note(cx, "cta_installers_note");
        }
    }

    fn whatsapp(&mut self, cx: &mut Cx) {
        match WHATSAPP_NUMBER {
            Some(number) => {
                let url = format!("https://wa.me/{number}?text={}", url_encode(WHATSAPP_MESSAGE));
                open_link(cx, &url);
            }
            None => self.show_cta_note(cx, "cta_whatsapp_soon"),
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
        // Under 150 px a tile stacks its minutes and bill (see paint_floor),
        // so it is one text line taller.
        let narrow = tile_w < 150.0;
        let tile_h = if narrow { 126.0 } else { 108.0 };
        for (path, _) in TABLES {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.walk.width = Size::Fixed(tile_w);
                view.walk.height = Size::Fixed(tile_h);
            }
        }
        for path in [ids!(floor.t3), ids!(floor.t6)] {
            self.ui.widget(cx, path).set_visible(cx, per_row == 3);
        }
        if narrow != self.floor_narrow {
            self.floor_narrow = narrow;
            self.paint_floor(cx);
        }
    }

    /// Receipt lines are up to 25 columns of a mono face. Under 360 px the
    /// paper is narrower than that at desk size (each line broke before its
    /// amount at 320), so the lines step down and stay whole.
    fn fit_ticket_lines(&mut self, cx: &mut Cx, layout_w: f64) {
        let size: f32 = if layout_w < 360.0 { 12.0 } else { 14.0 };
        let lines = self.ui.widget(cx, ids!(ticket.lines));
        lines.children(&mut |_, child| {
            if let Some(mut md) = child.borrow_mut::<Markdown>() {
                md.font_size = size;
                md.draw_text.text_style.font_size = size;
                md.text_style_normal.font_size = size;
                md.text_style_bold.font_size = size;
            }
        });
    }

    /// Hide top-bar parts, least needed first, until the words of the language
    /// in use fit the width: the section links, the wordmark (the icon stays
    /// the home link), "Get it free" (the hero has its own download button),
    /// Login (the footer has Account). The language chip always stays, so a
    /// visitor can switch back. Widths are measured in the font each part
    /// draws with, so this holds for every pack and script (Burmese "Login"
    /// pushed the chip off a 320 px screen). Redraws only when the choice
    /// changes; runs again on draws so late-loading script faces count.
    fn fit_top_bar(&mut self, cx: &mut Cx) {
        let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
        if size.x < 1.0 || size.y < 1.0 {
            return;
        }
        let layout_w = size.x.max(MIN_WIDTH);
        let compact = layout_w < 360.0;
        let pad = if compact {
            12.0
        } else if Tier::from_size(layout_w, size.y) == Tier::Phone {
            16.0
        } else {
            20.0
        };
        let gap = if compact { 6.0 } else { 10.0 };
        let button_w = |cx: &mut Cx, ui: &WidgetRef, path: &[LiveId], side_pad: f64| -> f64 {
            let text = ui.widget(cx, path).text();
            match ui.button(cx, path).borrow() {
                Some(b) => text_width(cx, &b.draw_text, &text) + side_pad,
                None => 0.0,
            }
        };
        let ui = self.ui.clone();
        let name_w = {
            let text = ui.widget(cx, ids!(brand_name)).text();
            match ui.label(cx, ids!(brand_name)).borrow() {
                Some(l) => text_width(cx, &l.draw_text, &text),
                None => 0.0,
            }
        };
        let login_w = button_w(cx, &ui, ids!(go_login), 20.0);
        let lang_w = button_w(cx, &ui, ids!(go_lang), 20.0);
        let cta_w = button_w(cx, &ui, ids!(bar_cta), 32.0);
        let mut links_w = 0.0;
        for path in [
            ids!(go_menus),
            ids!(go_how),
            ids!(go_features),
            ids!(go_offline),
            ids!(go_pricing),
            ids!(go_faq),
        ] {
            links_w += button_w(cx, &ui, path, 20.0) + 2.0;
        }
        // [links, wordmark, CTA, Login]
        let need = |show: [bool; 4]| -> f64 {
            // brand (icon + its own 10 px gap + name) and the Fill spacer.
            let mut parts = vec![36.0 + if show[1] { 10.0 + name_w } else { 0.0 }, 0.0];
            if show[0] {
                parts.push(links_w);
            }
            if show[3] {
                parts.push(login_w);
            }
            parts.push(lang_w);
            if show[2] {
                parts.push(cta_w);
            }
            2.0 * pad + parts.iter().sum::<f64>() + gap * (parts.len() as f64 - 1.0) + 4.0
        };
        let mut show = [layout_w >= NAV_LINKS_FROM, true, true, true];
        for part in 0..4 {
            if need(show) <= layout_w {
                break;
            }
            show[part] = false;
        }
        if self.bar_fit == Some(show) {
            return;
        }
        self.bar_fit = Some(show);
        self.ui.widget(cx, ids!(nav_links)).set_visible(cx, show[0]);
        self.ui.widget(cx, ids!(brand_name)).set_visible(cx, show[1]);
        self.ui.widget(cx, ids!(bar_cta)).set_visible(cx, show[2]);
        self.ui.widget(cx, ids!(go_login)).set_visible(cx, show[3]);
        self.ui.redraw(cx);
    }

    /// Shrink one-line hero copy that is wider than the hero column in the
    /// language in use: the pill, the title's widest word (German and Finnish
    /// compounds broke inside the word at 320 px) and the two CTA buttons
    /// (Tibetan, Khmer, Lao and Burmese ran off the screen). Never below
    /// `FIT_MIN_SCALE` of the size the page asked for. Titles in languages
    /// written without spaces are left alone: their "widest word" is the line.
    /// Runs after draws, when the column has its width.
    fn fit_hero_lines(&mut self, cx: &mut Cx) {
        let col_w = self.ui.view(cx, ids!(hero_copy)).area().rect(cx).size.x;
        if col_w < 1.0 {
            return;
        }
        let spaced = !matches!(
            self.lang.wanted.as_str(),
            "th" | "lo" | "km" | "my" | "bo" | "dz" | "zh" | "zh-Hant" | "ja"
        );
        let mut sizes = std::mem::take(&mut self.fit_sizes);
        let mut changed = false;
        // Pill padding 12 + 12 and its 1 px border.
        let pill = self.ui.widget(cx, ids!(hero_pill.label));
        changed |= fit_markdown(cx, &pill, col_w - 26.0, false, &mut sizes);
        if spaced {
            let title = self.ui.widget(cx, ids!(hero_title));
            changed |= fit_markdown(cx, &title, col_w, true, &mut sizes);
        }
        // PrimaryBtn 22 + 22, SecondaryBtn 20 + 20, label margins 2 + 2.
        for (path, chrome) in [(ids!(hero_cta.download), 48.0), (ids!(hero_cta.whatsapp), 44.0)] {
            let button = self.ui.widget(cx, path);
            changed |= fit_button(cx, &button, col_w - chrome, &mut sizes);
        }
        // Floor tiles (Khmer "Bill printed" was cut at the tile edge): each
        // tile's own width, less its 12 + 8 px inner padding.
        for (path, _) in TABLES {
            let tile = self.ui.widget(cx, path);
            let tile_w = tile.area().rect(cx).size.x;
            if tile_w < 1.0 {
                continue;
            }
            for part in [ids!(word), ids!(meta)] {
                let line = tile.widget(cx, part);
                changed |= fit_markdown(cx, &line, tile_w - 20.0, false, &mut sizes);
            }
        }
        self.fit_sizes = sizes;
        if changed {
            self.ui.redraw(cx);
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
        // Top bar: section links on wide windows (when they fit — see
        // fit_top_bar); tighter padding on phones.
        // Under 360 px the brand and "Get it free" do not both fit at desk
        // size: the button ran off the right edge at 320.
        let compact_bar = layout_w < 360.0;
        if let Some(mut bar) = self.ui.view(cx, ids!(top_bar)).borrow_mut() {
            let pad = if compact_bar {
                12.0
            } else if tier == Tier::Phone {
                16.0
            } else {
                20.0
            };
            bar.layout.padding = Inset { left: pad, right: pad, top: 0.0, bottom: 0.0 };
            bar.layout.spacing = if compact_bar { 6.0 } else { 10.0 };
        }
        if let Some(mut name) = self.ui.label(cx, ids!(brand_name)).borrow_mut() {
            name.draw_text.text_style.font_size = if compact_bar { 21.0 } else { 25.0 };
        }
        self.fit_top_bar(cx);
        // Parade keeps its own width; restart the NextFrame pump after layout.
        if self.shelf_playing {
            self.shelf_arm_anim(cx);
        }
        if tier_changed {
            let page = self.ui.widget(cx, ids!(page));
            if self.desk_font.is_empty() {
                collect_label_sizes(&page, &mut self.desk_font);
            }
            let mut next = 0;
            scale_labels(&page, tier, &self.desk_font, &mut next);
        }
        self.fit_ticket_lines(cx, layout_w);
        self.sync_auth_chip(cx);
        self.ui.redraw(cx);
    }

    /// Login → initials when firebase-init writes `#auth=XX` into the URL hash.
    fn sync_auth_chip(&mut self, cx: &mut Cx) {
        let next = read_auth_chip_label(cx);
        if next != self.auth_chip_label {
            self.auth_chip_label = next.clone();
            let text = next.clone().unwrap_or_else(|| self.tr("login"));
            self.ui.button(cx, ids!(go_login)).set_text(cx, &text);
            self.fit_top_bar(cx);
            self.ui.redraw(cx);
        }
        if self.auth_chip_poll > 0 {
            self.auth_chip_poll = self.auth_chip_poll.saturating_sub(1);
            let _ = cx.new_next_frame();
        }
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
        if let Ok(q) = std::env::var("SITE_SHELF_QUERY") {
            let q = q.trim().to_string();
            if !q.is_empty() {
                self.pending_shelf_query = Some(q);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn flush_pending_shelf_query(&mut self, cx: &mut Cx) {
        let Some(q) = self.pending_shelf_query.take() else {
            return;
        };
        self.ui
            .text_input(cx, ids!(menus.shelf_find))
            .set_text(cx, &q);
        self.ui
            .widget(cx, ids!(menus.shelf_find_hint))
            .set_visible(cx, q.is_empty());
        self.set_shelf_query(cx, q);
        // Skip the typing quiet window so preview screenshots see the card.
        self.shelf_search_quiet_until = 0.0;
        self.shelf_search_dirty = true;
        let _ = self.flush_shelf_search_paint(cx, 0.0);
        let _ = self.flush_shelf_search_paint(cx, 1.0);
    }

    /// Which content section owns the viewport (just under the sticky bar).
    fn active_nav_section(&self, cx: &Cx) -> Option<&'static str> {
        let page = self.ui.view(cx, ids!(page));
        if !page.area().is_valid(cx) {
            return None;
        }
        let bar = self.ui.view(cx, ids!(top_bar));
        let mark = if bar.area().is_valid(cx) {
            bar.area().rect(cx).pos.y + bar.area().rect(cx).size.y + 12.0
        } else {
            page.area().rect(cx).pos.y + 72.0
        };
        let mut active: Option<&'static str> = None;
        for (id, path) in SECTIONS {
            let w = self.ui.widget(cx, path);
            if !w.area().is_valid(cx) {
                continue;
            }
            let top = w.area().rect(cx).pos.y;
            if top <= mark {
                active = Some(id);
            }
        }
        active
    }

    /// Highlight the top-bar link for the section on screen.
    fn paint_nav_active(&mut self, cx: &mut Cx) {
        let active = self.active_nav_section(cx);
        // Only when the active section flips. Re-colouring every Draw/NextFrame
        // while the finger scrolls smears glyphs on wasm (Android Chrome).
        if active == self.nav_active {
            return;
        }
        self.nav_active = active;
        // Quiet peach idle; white when that section is under the bar.
        let idle = vec4(0.965, 0.863, 0.796, 1.0); // #F6DCCB
        let on = vec4(1.0, 1.0, 1.0, 1.0);
        for (path, section) in NAV_LINKS {
            let color = if active == Some(section) { on } else { idle };
            if let Some(mut btn) = self.ui.button(cx, path).borrow_mut() {
                btn.draw_text.color = color;
            }
        }
        self.ui.view(cx, ids!(nav_links)).redraw(cx);
    }

    fn shelf_speed_px(&self) -> f64 {
        SHELF_SPEED_STEP * f64::from(self.shelf_speed_level)
    }

    fn paint_shelf_chrome(&mut self, cx: &mut Cx) {
        // While searching, the parade is forced paused; the button still
        // shows the remembered choice that comes back when search clears.
        // Icons only — no "Pause"/"Play"/"Speed" labels (desk one-row + phone scroll).
        let playing = if self.shelf_query.trim().is_empty() {
            self.shelf_playing
        } else {
            self.shelf_want_playing
        };
        // FA solid pause / play (f04c / f04b). Not Unicode ⏸/▶ — those
        // paint .notdef tofu on Baloo/Plex (live khaynjo.com 2026-11-21).
        let play = if playing { "\u{f04c}" } else { "\u{f04b}" };
        self.ui
            .button(cx, ids!(menus.shelf_play))
            .set_text(cx, play);
        self.ui.button(cx, ids!(menus.shelf_speed)).set_text(
            cx,
            &format!("{}", self.shelf_speed_level),
        );
    }

    fn shelf_arm_anim(&mut self, cx: &mut Cx) {
        self.shelf_last_t = 0.0;
        self.shelf_anim_frame = cx.new_next_frame();
    }

    fn rebuild_shelf_hits(&mut self) {
        self.shelf_hits = shelf_filter_hits(&self.shelf_query);
    }

    /// Lay slots out in a row and assign consecutive hits (call after hits change).
    fn reset_shelf_slots(&mut self) {
        let len = self.shelf_hits.len();
        self.shelf_slot_country = [None; 10];
        self.shelf_slot_photo = [None; 10];
        if len == 0 {
            self.shelf_slot_x = [0.0; 10];
            self.shelf_slot_item = [0; 10];
            self.shelf_next_item = 0;
            self.shelf_prev_item = 0;
            return;
        }
        for i in 0..SHELF_SLOT_COUNT {
            self.shelf_slot_x[i] = i as f64 * SHELF_PITCH;
            self.shelf_slot_item[i] = i % len;
        }
        self.shelf_next_item = SHELF_SLOT_COUNT % len;
        self.shelf_prev_item = len.saturating_sub(1);
    }

    fn place_shelf_slot(&mut self, cx: &mut Cx, slot: usize) {
        // Overlay + margin.left acts like abs X (this Makepad has no walk.abs_pos).
        // Inset keeps the left of a snapped card off the content edge so the
        // right-hand peek reads as intentional carousel, not overflow.
        if let Some(mut view) = self.ui.view(cx, SHELF_SLOTS[slot]).borrow_mut() {
            view.walk.margin.left = self.shelf_slot_x[slot] + SHELF_EDGE_INSET;
            view.walk.margin.top = 0.0;
            view.walk.margin.right = 0.0;
            view.walk.margin.bottom = 0.0;
        }
    }

    /// Shift every slot so the nearest card sits at x=0 (plus edge inset on
    /// place). Used when pausing so a mid-scroll half-card does not look like
    /// a crop bug on a phone.
    fn snap_shelf_to_nearest_card(&mut self, cx: &mut Cx) {
        let mut best_i = 0usize;
        let mut best_abs = f64::INFINITY;
        for i in 0..SHELF_SLOT_COUNT {
            let a = self.shelf_slot_x[i].abs();
            if a < best_abs {
                best_abs = a;
                best_i = i;
            }
        }
        let shift = self.shelf_slot_x[best_i];
        if shift.abs() < 0.5 {
            return;
        }
        for i in 0..SHELF_SLOT_COUNT {
            self.shelf_slot_x[i] -= shift;
        }
        self.place_all_shelf_slots(cx);
        self.ui.view(cx, ids!(menus.shelf_parade)).redraw(cx);
    }

    fn place_all_shelf_slots(&mut self, cx: &mut Cx) {
        for i in 0..SHELF_SLOT_COUNT {
            self.place_shelf_slot(cx, i);
        }
    }

    /// Bind one slot's card to `shelf_slot_item[slot]`.
    /// `defer_photo`: queue WebP bind for a later frame (wrap path); false = load now.
    /// On wrap we keep the previous texture visible until the new file is ready —
    /// blanking the Image (or `set_texture(None)` while HTTP is in flight) is what
    /// showed as black frames in the parade.
    fn paint_shelf_slot(&mut self, cx: &mut Cx, slot: usize, defer_photo: bool) {
        let len = self.shelf_hits.len();
        if len == 0 {
            self.ui.view(cx, SHELF_SLOTS[slot]).set_visible(cx, false);
            return;
        }
        let item = self.shelf_slot_item[slot] % len;
        let d = self.shelf_hits[item];
        self.ui.view(cx, SHELF_SLOTS[slot]).set_visible(cx, true);
        let card = self.ui.widget(cx, SHELF_CARDS[slot]);
        if self.shelf_slot_country[slot] == Some(d.country) {
            return;
        }
        self.shelf_slot_country[slot] = Some(d.country);
        card.widget(cx, ids!(country)).set_text(cx, d.country);
        let line = shelf_native_line(d);
        card.widget(cx, ids!(native))
            .set_text(cx, line.as_deref().unwrap_or(""));
        card.widget(cx, ids!(dish)).set_text(cx, d.dish);
        let photo = card.image(cx, ids!(photo));
        let soon = card.view(cx, ids!(soon));
        if let Some(file) = d.photo {
            if self.shelf_slot_photo[slot] == Some(file) {
                photo.set_visible(cx, true);
                soon.set_visible(cx, false);
                return;
            }
            if defer_photo {
                // Keep whatever texture is already on the Image; swap next idle frame.
                self.shelf_photo_queue.retain(|(s, _)| *s != slot);
                self.shelf_photo_queue.push((slot, file));
                self.ensure_world_photo_bytes(cx, file);
            } else if self.bind_world_photo(cx, &photo, file) {
                self.shelf_slot_photo[slot] = Some(file);
                photo.set_visible(cx, true);
                soon.set_visible(cx, false);
            } else {
                // First paint, bytes not ready yet — placeholder until fetch lands.
                photo.set_visible(cx, false);
                soon.set_visible(cx, true);
                self.shelf_photo_queue.retain(|(s, _)| *s != slot);
                self.shelf_photo_queue.push((slot, file));
            }
        } else {
            photo.set_visible(cx, false);
            soon.set_visible(cx, true);
            if self.shelf_slot_photo[slot].is_some() {
                photo.set_texture(cx, None);
                self.shelf_slot_photo[slot] = None;
            }
        }
    }

    /// Bind at most one queued photo when bytes are already cached.
    /// Never clears the Image while a fetch is still in flight.
    fn flush_one_shelf_photo(&mut self, cx: &mut Cx) {
        let Some(qi) = self.shelf_photo_queue.iter().position(|(slot, file)| {
            if *slot >= SHELF_SLOT_COUNT {
                return false;
            }
            let len = self.shelf_hits.len();
            if len == 0 {
                return false;
            }
            let want = self.shelf_hits[self.shelf_slot_item[*slot] % len].photo;
            want == Some(*file) && self.photo_bytes.contains_key(*file)
        }) else {
            // Still waiting on HTTP — kick prefetch, leave textures alone.
            if let Some((_, file)) = self.shelf_photo_queue.first().copied() {
                self.ensure_world_photo_bytes(cx, file);
            }
            return;
        };
        let (slot, file) = self.shelf_photo_queue.remove(qi);
        let card = self.ui.widget(cx, SHELF_CARDS[slot]);
        let photo = card.image(cx, ids!(photo));
        let soon = card.view(cx, ids!(soon));
        if self.bind_world_photo(cx, &photo, file) {
            self.shelf_slot_photo[slot] = Some(file);
            photo.set_visible(cx, true);
            soon.set_visible(cx, false);
        }
    }

    /// Ask for the next few dishes' photos so a wrap rarely hits an empty cache.
    fn prefetch_shelf_photos(&mut self, cx: &mut Cx) {
        let len = self.shelf_hits.len();
        if len == 0 {
            return;
        }
        let start = if self.shelf_flow_left {
            self.shelf_next_item
        } else {
            self.shelf_prev_item
        };
        for k in 0..SHELF_SLOT_COUNT {
            let idx = if self.shelf_flow_left {
                (start + k) % len
            } else {
                shelf_wrap_index(start as isize - k as isize, len)
            };
            if let Some(file) = self.shelf_hits[idx].photo {
                self.ensure_world_photo_bytes(cx, file);
            }
        }
    }

    fn paint_shelf(&mut self, cx: &mut Cx) {
        if self.shelf_hits.is_empty() && self.shelf_query.trim().is_empty() {
            self.rebuild_shelf_hits();
            self.reset_shelf_slots();
        }
        let list_len = self.shelf_hits.len();
        if list_len == 0 {
            for path in SHELF_SLOTS {
                self.ui.view(cx, path).set_visible(cx, false);
            }
            self.paint_shelf_note(cx);
            self.paint_shelf_chrome(cx);
            self.ui.view(cx, ids!(menus.shelf_parade)).redraw(cx);
            return;
        }
        self.paint_shelf_note(cx);
        for i in 0..SHELF_SLOT_COUNT {
            self.paint_shelf_slot(cx, i, false);
            self.place_shelf_slot(cx, i);
        }
        self.prefetch_shelf_photos(cx);
        self.ui
            .view(cx, ids!(menus.shelf_parade))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.paint_shelf_chrome(cx);
        self.ui.view(cx, ids!(menus.shelf_parade)).redraw(cx);
    }

    /// Backspace / clear: rebuild hits immediately, paint cards after a short quiet.
    fn flush_shelf_search_paint(&mut self, cx: &mut Cx, time: f64) -> bool {
        if !self.shelf_search_dirty {
            return false;
        }
        const QUIET: f64 = 0.14;
        if self.shelf_search_quiet_until <= 0.0 {
            self.shelf_search_quiet_until = time + QUIET;
            self.shelf_anim_frame = cx.new_next_frame();
            return true;
        }
        if time < self.shelf_search_quiet_until {
            self.shelf_anim_frame = cx.new_next_frame();
            return true;
        }
        self.shelf_search_dirty = false;
        self.shelf_search_quiet_until = 0.0;
        self.paint_shelf(cx);
        if self.shelf_playing {
            self.shelf_arm_anim(cx);
        }
        true
    }

    fn tick_shelf_anim(&mut self, cx: &mut Cx, time: f64) {
        // While the search field is being edited (backspace/clear), only flush
        // the deferred card paint — do not also scroll the parade this frame.
        if self.shelf_search_dirty {
            self.flush_shelf_search_paint(cx, time);
            return;
        }
        if !self.shelf_playing || self.shelf_speed_level == 0 {
            return;
        }
        let list_len = self.shelf_hits.len();
        if list_len < 2 {
            return;
        }
        if self.shelf_last_t <= 0.0 {
            self.shelf_last_t = time;
            self.shelf_anim_frame = cx.new_next_frame();
            return;
        }
        let dt = (time - self.shelf_last_t).clamp(0.0, 0.05);
        self.shelf_last_t = time;
        let step = self.shelf_speed_px() * dt;
        // Move each card; when one leaves, wrap it alone and rebind only that card.
        let mut wrapped: Option<usize> = None;
        if self.shelf_flow_left {
            for i in 0..SHELF_SLOT_COUNT {
                self.shelf_slot_x[i] -= step;
                if self.shelf_slot_x[i] <= -SHELF_PITCH {
                    self.shelf_slot_x[i] += SHELF_TRACK;
                    self.shelf_slot_item[i] = self.shelf_next_item;
                    self.shelf_next_item = (self.shelf_next_item + 1) % list_len;
                    self.shelf_prev_item = shelf_wrap_index(
                        self.shelf_slot_item[i] as isize - 1,
                        list_len,
                    );
                    wrapped = Some(i);
                }
            }
        } else {
            for i in 0..SHELF_SLOT_COUNT {
                self.shelf_slot_x[i] += step;
                if self.shelf_slot_x[i] >= SHELF_TRACK - SHELF_PITCH {
                    self.shelf_slot_x[i] -= SHELF_TRACK;
                    self.shelf_slot_item[i] = self.shelf_prev_item;
                    self.shelf_prev_item = shelf_wrap_index(
                        self.shelf_prev_item as isize - 1,
                        list_len,
                    );
                    self.shelf_next_item = shelf_wrap_index(
                        self.shelf_slot_item[i] as isize + SHELF_SLOT_COUNT as isize,
                        list_len,
                    );
                    wrapped = Some(i);
                }
            }
        }
        if let Some(slot) = wrapped {
            // Rebind labels; keep the old texture until the new file is cached.
            self.shelf_slot_country[slot] = None;
            self.paint_shelf_slot(cx, slot, true);
            self.prefetch_shelf_photos(cx);
        } else {
            // Idle frame: bind at most one deferred photo (never on a wrap frame).
            self.flush_one_shelf_photo(cx);
            self.prefetch_shelf_photos(cx);
        }
        self.place_all_shelf_slots(cx);
        self.ui.view(cx, ids!(menus.shelf_parade)).redraw(cx);
        self.shelf_anim_frame = cx.new_next_frame();
    }

    fn set_shelf_playing(&mut self, cx: &mut Cx, playing: bool) {
        self.shelf_playing = playing;
        if self.shelf_query.trim().is_empty() {
            self.shelf_want_playing = playing;
        }
        self.paint_shelf_chrome(cx);
        if playing {
            self.shelf_arm_anim(cx);
        } else {
            // Pause: land on a full card + intentional next-card peek.
            self.snap_shelf_to_nearest_card(cx);
        }
    }

    /// After hero shaders settle, reveal deferred bands and bind the parade.
    fn maybe_finish_shelf_warmup(&mut self, cx: &mut Cx) {
        if self.shelf_warmup_frames == 0 {
            return;
        }
        self.shelf_warmup_frames = self.shelf_warmup_frames.saturating_sub(1);
        if self.shelf_warmup_frames > 0 {
            let _ = cx.new_next_frame();
            return;
        }
        for path in DEFERRED_FIRST_PAINT {
            self.ui.view(cx, path).set_visible(cx, true);
        }
        self.paint_floor(cx);
        self.paint_shelf(cx);
        self.bind_feature_icons(cx);
        if self.shelf_want_playing {
            self.set_shelf_playing(cx, true);
        }
        self.ui.redraw(cx);
    }

    fn set_shelf_speed(&mut self, cx: &mut Cx, level: u8) {
        self.shelf_speed_level = level.min(SHELF_SPEED_MAX_LEVEL);
        self.paint_shelf_chrome(cx);
        if self.shelf_playing && self.shelf_speed_level > 0 {
            self.shelf_arm_anim(cx);
        }
    }

    fn set_shelf_query(&mut self, cx: &mut Cx, query: String) {
        if query == self.shelf_query {
            return;
        }
        let was_searching = !self.shelf_query.trim().is_empty();
        let searching = !query.trim().is_empty();
        if searching && !was_searching {
            self.shelf_want_playing = self.shelf_playing;
            self.shelf_playing = false;
        } else if !searching && was_searching {
            self.shelf_playing = self.shelf_want_playing;
        } else if searching {
            self.shelf_playing = false;
        }
        self.shelf_query = query;
        self.rebuild_shelf_hits();
        self.reset_shelf_slots();
        self.shelf_photo_queue.clear();
        // Paint after typing settles (flush_shelf_search_paint).
        self.shelf_search_dirty = true;
        self.shelf_search_quiet_until = 0.0;
        self.paint_shelf_note(cx);
        self.paint_shelf_chrome(cx);
        self.shelf_anim_frame = cx.new_next_frame();
    }

    /// Put `file` into a card's Image: decode cached bytes now, else clear
    /// Decode `file` into `image` when bytes are already cached. Returns false
    /// if the bytes are still missing (caller must not blank the Image).
    /// Synchronous WebP decode on purpose — Makepad's async image path
    /// decodes on a thread pool, which the `--no-threads` wasm build lacks
    /// (cards stayed black in the browser).
    fn bind_world_photo(&mut self, cx: &mut Cx, image: &ImageRef, file: &'static str) -> bool {
        self.ensure_world_photo_bytes(cx, file);
        let Some(bytes) = self.photo_bytes.get(file).cloned() else {
            return false;
        };
        if let Err(err) = image.load_webp_from_data(cx, &bytes) {
            log!("khaynjo-site: world photo {file}: {err:?}");
            return false;
        }
        true
    }

    /// Load disk / start HTTP for `file` without touching any Image texture.
    fn ensure_world_photo_bytes(&mut self, cx: &mut Cx, file: &'static str) {
        if self.photo_bytes.contains_key(file) {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = cx;
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/dishes/world")
                .join(file);
            match std::fs::read(&path) {
                Ok(bytes) => {
                    self.photo_bytes
                        .insert(file.to_string(), std::rc::Rc::new(bytes));
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

    /// Start HTTP / disk load for a feature-card icon (after first paint).
    fn ensure_feature_icon_bytes(&mut self, cx: &mut Cx, file: &'static str) {
        if self.photo_bytes.contains_key(file) {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = cx;
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/icons")
                .join(file);
            match std::fs::read(&path) {
                Ok(bytes) => {
                    self.photo_bytes
                        .insert(file.to_string(), std::rc::Rc::new(bytes));
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
                HttpRequest::new(feature_icon_web_path(&pathname, file), HttpMethod::GET),
            );
            self.photo_pending.push((id, file.to_string()));
        }
    }

    /// Decode cached feature icons into the deferred features band.
    fn bind_feature_icons(&mut self, cx: &mut Cx) {
        for (path, file) in FEATURE_ICONS {
            self.ensure_feature_icon_bytes(cx, file);
            let Some(bytes) = self.photo_bytes.get(file).cloned() else {
                continue;
            };
            let image = self.ui.image(cx, path);
            if let Err(err) = image.load_webp_from_data(cx, &bytes) {
                log!("khaynjo-site: feature icon {file}: {err:?}");
            }
        }
    }

    /// Fetched photo/icon bytes land here; rebind shelf slots and feature cards.
    fn handle_photo_responses(&mut self, cx: &mut Cx, responses: &NetworkResponsesEvent) {
        let mut landed: Vec<String> = Vec::new();
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
                    self.photo_bytes
                        .insert(file.clone(), std::rc::Rc::new(bytes));
                    landed.push(file);
                }
                None => log!("khaynjo-site: asset {file} did not load"),
            }
        }
        if landed.is_empty() {
            return;
        }
        if FEATURE_ICONS
            .iter()
            .any(|(_, file)| landed.iter().any(|f| f == *file))
        {
            self.bind_feature_icons(cx);
        }
        // Queue every visible slot that needs a just-landed file, then flush one.
        let len = self.shelf_hits.len();
        if len == 0 {
            return;
        }
        for slot in 0..SHELF_SLOT_COUNT {
            let want = self.shelf_hits[self.shelf_slot_item[slot] % len].photo;
            let Some(file) = want else {
                continue;
            };
            if !landed.iter().any(|f| f == file) {
                continue;
            }
            if self.shelf_slot_photo[slot] == Some(file) {
                continue;
            }
            self.shelf_photo_queue.retain(|(s, _)| *s != slot);
            self.shelf_photo_queue.push((slot, file));
        }
        self.flush_one_shelf_photo(cx);
        self.ui.view(cx, ids!(menus.shelf_parade)).redraw(cx);
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
        // External destinations (Play, WhatsApp, other brands) → new tab.
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

/// Same-site paths (Login /account, privacy, shop, …) replace this tab.
fn open_same_site(cx: &mut Cx, url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        cx.open_url(url, OpenUrlInPlace::Yes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        open_link(cx, url);
    }
}

/// Makepad EN chip → `#lang` (keep `#auth=` so the Login chip stays signed-in).
fn open_lang_picker(cx: &mut Cx) {
    #[cfg(target_arch = "wasm32")]
    {
        let url = bridge_hash(cx, "lang");
        cx.open_url(&url, OpenUrlInPlace::Yes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = cx;
        log!("language picker is web-only");
    }
}

/// Signed-in chip → `#authmenu` (HTML Logout / Account menu).
fn open_auth_menu(cx: &mut Cx) {
    #[cfg(target_arch = "wasm32")]
    {
        let url = bridge_hash(cx, "authmenu");
        cx.open_url(&url, OpenUrlInPlace::Yes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = cx;
        log!("auth menu is web-only");
    }
}

/// `#<action>` for the HTML chrome (`lang` picker, `authmenu`) that keeps the
/// page's `l=` (language) and `auth=` (signed-in chip) segments.
#[cfg(target_arch = "wasm32")]
fn bridge_hash(cx: &Cx, action: &str) -> String {
    let mut url = format!("#{action}");
    if let OsType::Web(page) = cx.os_type() {
        if let Some(lang) = site_i18n::lang_from_hash(&page.hash) {
            url.push_str(&format!("&l={lang}"));
        }
    }
    if let Some(label) = read_auth_chip_label(cx) {
        url.push_str(&format!("&auth={}", url_encode(&label)));
    }
    url
}

/// Firebase session chip via `#auth=` hash (firebase-init.js). No web-sys —
/// that pulls wasm-bindgen imports Makepad's bridge does not provide.
fn read_auth_chip_label(cx: &Cx) -> Option<String> {
    let OsType::Web(page) = cx.os_type() else {
        return None;
    };
    auth_label_from_hash(&page.hash)
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Script #[rust] bools/u8 default to false/0 — set parade defaults here.
        // Shelf starts paused/hidden: first-wave WebGL on Mali-T830 cannot
        // compile hero + 10 Images + WorldFont together (blank cream canvas).
        self.shelf_playing = false;
        self.shelf_want_playing = true;
        self.shelf_flow_left = true;
        self.shelf_speed_level = SHELF_SPEED_DEFAULT_LEVEL;
        self.rebuild_shelf_hits();
        self.reset_shelf_slots();
        self.apply_gates(cx);
        for path in DEFERRED_FIRST_PAINT {
            self.ui.view(cx, path).set_visible(cx, false);
        }
        for path in SHELF_SLOTS {
            self.ui.view(cx, path).set_visible(cx, false);
        }
        self.paint_nav_active(cx);
        // Keys go to the page from the start (the `page` selection root also
        // answers Ctrl/Cmd+A with nothing focused).
        self.ui.view(cx, ids!(page)).set_key_focus(cx);
        self.pending_section = self.section_from_page(cx);
        self.pending_frames = 120;
        self.pending_stable = 0;
        // ~0.25s at 60fps — enough for hero RoundedView/text to compile alone.
        self.shelf_warmup_frames = 24;
        self.auth_chip_poll = 90;
        let _ = cx.new_next_frame();
        #[cfg(not(target_arch = "wasm32"))]
        self.apply_native_preview(cx);
        // `#l=<code>` from lang-nav.js (or SITE_LANG natively): fetch that pack.
        self.sync_lang(cx);
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
        if self.ui.button(cx, ids!(menus.shelf_flow_left)).clicked(actions) {
            // ← — world flows left (default).
            self.shelf_flow_left = true;
            if self.shelf_playing {
                self.shelf_arm_anim(cx);
            }
        }
        if self.ui.button(cx, ids!(menus.shelf_flow_right)).clicked(actions) {
            // → — world flows right.
            self.shelf_flow_left = false;
            if self.shelf_playing {
                self.shelf_arm_anim(cx);
            }
        }
        if self.ui.button(cx, ids!(menus.shelf_play)).clicked(actions) {
            if !self.shelf_query.trim().is_empty() {
                self.shelf_want_playing = !self.shelf_want_playing;
                self.paint_shelf_chrome(cx);
            } else {
                self.set_shelf_playing(cx, !self.shelf_playing);
            }
        }
        if self.ui.button(cx, ids!(menus.shelf_speed_down)).clicked(actions) {
            let next = self.shelf_speed_level.saturating_sub(1);
            self.set_shelf_speed(cx, next);
        }
        if self.ui.button(cx, ids!(menus.shelf_speed_up)).clicked(actions) {
            let next = self.shelf_speed_level.saturating_add(1);
            self.set_shelf_speed(cx, next);
        }
        if let Some(text) = self.ui.text_input(cx, ids!(menus.shelf_find)).changed(actions) {
            // The placeholder is a Label over the input (translatable, any script).
            self.ui
                .widget(cx, ids!(menus.shelf_find_hint))
                .set_visible(cx, text.is_empty());
            self.set_shelf_query(cx, text);
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
            open_same_site(cx, &url);
        }
        if self.ui.button(cx, ids!(to_terms)).clicked(actions) {
            let url = self.site_url(cx, TERMS_PATH);
            open_same_site(cx, &url);
        }
        if self.ui.button(cx, ids!(to_cancellation)).clicked(actions) {
            let url = self.site_url(cx, CANCELLATION_PATH);
            open_same_site(cx, &url);
        }
        if self.ui.button(cx, ids!(to_photo_credits)).clicked(actions) {
            let url = self.site_url(cx, PHOTO_CREDITS_PATH);
            open_same_site(cx, &url);
        }
        if self.ui.button(cx, ids!(to_shop)).clicked(actions)
            || self.ui.button(cx, ids!(go_cloud_shop)).clicked(actions)
        {
            let url = self.site_url(cx, SHOP_PATH);
            open_same_site(cx, &url);
        }

        // Footer "Account" → manage account page (no bounce-back).
        if self.ui.button(cx, ids!(to_account)).clicked(actions) {
            let url = self.site_url(cx, ACCOUNT_PATH);
            open_same_site(cx, &url);
        }
        // Topbar Login → account; signed-in initials → Logout/Account menu.
        if self.ui.button(cx, ids!(go_login)).clicked(actions) {
            if self.auth_chip_label.is_some() {
                open_auth_menu(cx);
            } else {
                let url = self.site_url(cx, "/account?next=/");
                open_same_site(cx, &url);
            }
        }
        // EN is a real topbar NavLink; opens the HTML 115-lang picker (hash bridge).
        if self.ui.button(cx, ids!(go_lang)).clicked(actions) {
            open_lang_picker(cx);
        }
        if self.ui.button(cx, ids!(to_kumkaj)).clicked(actions)
            || self.ui.button(cx, ids!(foot_kumkaj)).clicked(actions)
        {
            let url = self.kumkaj_url(cx);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(foot_gotigin)).clicked(actions) {
            open_link(cx, "https://gotigin.com/");
        }
        if self.ui.button(cx, ids!(foot_product)).clicked(actions) {
            let url = self.site_url(cx, "/");
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
            self.handle_lang_responses(cx, responses);
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        match event {
            Event::Startup | Event::WindowGeomChange(_) => {
                if matches!(event, Event::Startup) {
                    self.auth_chip_poll = 90; // ~1.5s at 60fps for Auth restore
                    let _ = cx.new_next_frame();
                }
                self.apply_responsive(cx);
            }
            // hashchange (incl. `#auth=`) arrives as Signal on web.
            Event::Signal => {
                self.sync_auth_chip(cx);
                self.sync_lang(cx);
            }
            Event::Scroll(_) => {
                self.paint_nav_active(cx);
            }
            Event::NextFrame(ne) => {
                if self.laid_out.is_none() {
                    self.apply_responsive(cx);
                }
                // firebase-init may write the chip after first paint.
                self.sync_auth_chip(cx);
                self.maybe_finish_shelf_warmup(cx);
                self.paint_nav_active(cx);
                if ne.set.contains(&self.shelf_anim_frame) {
                    self.tick_shelf_anim(cx, ne.time);
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
                            #[cfg(not(target_arch = "wasm32"))]
                            self.flush_pending_shelf_query(cx);
                        }
                    }
                    let _ = cx.new_next_frame();
                } else {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.flush_pending_shelf_query(cx);
                }
            }
            Event::Draw(_) => {
                if self.laid_out.is_none() {
                    self.apply_responsive(cx);
                }
                // Script faces load after the first draw; measure again.
                self.fit_top_bar(cx);
                self.fit_hero_lines(cx);
                self.paint_nav_active(cx);
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
                            #[cfg(not(target_arch = "wasm32"))]
                            self.flush_pending_shelf_query(cx);
                        }
                    }
                    let _ = cx.new_next_frame();
                } else {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.flush_pending_shelf_query(cx);
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
        assert_eq!(section_from_hash("#auth=LO"), None);
        assert_eq!(section_from_hash("#menus&auth=LO"), Some("menus"));
        assert_eq!(auth_label_from_hash("#auth=LO"), Some("LO".into()));
        assert_eq!(auth_label_from_hash("#menus&auth=AB"), Some("AB".into()));
        assert_eq!(auth_label_from_hash("#menus"), None);
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
        assert_eq!(MIN_WIDTH, 320.0);
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

    /// The Menus section does not offer a private code. The live list is
    /// `public/index.json` (the photo shelf, plus authored packs when published).
    #[test]
    fn template_cards_match_the_public_catalogue() {
        let ui = include_str!("ui.rs");
        let cdn = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("static/v1/catalogue/public");
        let index = std::fs::read_to_string(cdn.join("index.json")).unwrap();
        assert!(
            index.contains("\"id\": \"khaynjo-items\""),
            "khaynjo-items in public/index.json"
        );
        let lower = ui.to_lowercase();
        assert!(!lower.contains("12-digit"), "no private catalogue code on the site");
        assert!(!lower.contains("private code"));
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
    fn shelf_parade_wraps_the_whole_world() {
        let list = shelf_filter_hits("");
        assert_eq!(list.len(), WORLD.len());
        let mut seen = std::collections::HashSet::new();
        let mut i = 0isize;
        for _ in 0..list.len() {
            let idx = shelf_wrap_index(i, list.len());
            assert!(seen.insert(list[idx].country));
            i += 1;
        }
        assert_eq!(seen.len(), list.len());
        assert_eq!(shelf_wrap_index(-1, 199), 198);
        assert_eq!(shelf_filter_hits("india").len(), 1);
        assert!(shelf_filter_hits("zzzz-nope").is_empty());
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

    #[test]
    fn feature_icon_web_path_follows_page_path() {
        assert_eq!(
            feature_icon_web_path("/", "floor.webp"),
            "/khaynjo_site/resources/icons/floor.webp"
        );
        assert_eq!(
            feature_icon_web_path("/preview/", "floor.webp"),
            "/preview/khaynjo_site/resources/icons/floor.webp"
        );
    }

    /// Feature icons are shipped under resources/icons and listed for lazy bind.
    #[test]
    fn feature_icons_are_shipped() {
        for (_, file) in FEATURE_ICONS {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/icons")
                .join(file);
            assert!(path.is_file(), "{file}");
        }
    }
}

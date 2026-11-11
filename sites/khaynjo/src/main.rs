//! khaynjo.com — Khaynjo's site as a fullpage Makepad 2 app (wasm in the
//! browser; the same crate runs native for previews and as an offline
//! presenter in a restaurant). Story: apps/khaynjo/docs/KHAYNJO_WEBSITE.md.
//! Copied from sites/kumkaj, then diverged (copy-then-diverge; no shared
//! site engine).
//!
//! Scroll / swipe up / tap / ↓ → PgDn Space flip chapters; ↑ ← PgUp go back;
//! Home / End jump. The small table tiles in the top bar jump to a chapter.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:3` opens a 390×844 window on chapter 3 (1-based).
//! - `SITE_LINKS=local` sends the kumkaj.com link to the local wasm server.
//!
//! In the browser, `#kot` or `#3` opens that chapter; a page served from
//! 127.0.0.1 / localhost links back to the local kumkaj.com (:8010).

pub use makepad_widgets;

use makepad_widgets::*;

mod floor;
use floor::Visit;

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

// ---- Chapters ---------------------------------------------------------

const CHAPTERS: [(&str, &str); 10] = [
    ("rush", "Lunch rush"),
    ("order", "Order taken"),
    ("kot", "The KOT"),
    ("kitchen", "Kitchen in step"),
    ("changes", "Service changes"),
    ("settle", "Settle"),
    ("dayclose", "Close the day"),
    ("menu", "Your menu"),
    ("offline", "Internet down"),
    ("start", "Start"),
];

const SLIDES: [&[LiveId]; 10] = [
    ids!(rush),
    ids!(order),
    ids!(kot),
    ids!(kitchen),
    ids!(changes),
    ids!(settle),
    ids!(dayclose),
    ids!(menu),
    ids!(offline),
    ids!(start),
];

/// Top-bar tiles: [hit, idle, here].
const NAV: [[&[LiveId]; 3]; 10] = [
    [ids!(nav.n0.hit), ids!(nav.n0.idle), ids!(nav.n0.here)],
    [ids!(nav.n1.hit), ids!(nav.n1.idle), ids!(nav.n1.here)],
    [ids!(nav.n2.hit), ids!(nav.n2.idle), ids!(nav.n2.here)],
    [ids!(nav.n3.hit), ids!(nav.n3.idle), ids!(nav.n3.here)],
    [ids!(nav.n4.hit), ids!(nav.n4.idle), ids!(nav.n4.here)],
    [ids!(nav.n5.hit), ids!(nav.n5.idle), ids!(nav.n5.here)],
    [ids!(nav.n6.hit), ids!(nav.n6.idle), ids!(nav.n6.here)],
    [ids!(nav.n7.hit), ids!(nav.n7.idle), ids!(nav.n7.here)],
    [ids!(nav.n8.hit), ids!(nav.n8.idle), ids!(nav.n8.here)],
    [ids!(nav.n9.hit), ids!(nav.n9.idle), ids!(nav.n9.here)],
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
/// this wide (px), and stack otherwise — width-based, not tier-based: a
/// fixed prop beside a headline needs its own width plus a readable text
/// column whatever the device.
const ROWS: [(&[LiveId], f64); 16] = [
    (ids!(rush.rush_row), 1024.0),
    (ids!(rush.rush_cta.buttons), 420.0),
    (ids!(rush.rush_world_a), 520.0),
    (ids!(rush.rush_world_b), 520.0),
    (ids!(order.order_row), 760.0),
    (ids!(kot.kot_row), 758.0),
    (ids!(kitchen.kds), 700.0),
    (ids!(kitchen.kds_points), 760.0),
    (ids!(changes.changes_grid), 880.0),
    (ids!(settle.settle_row), 800.0),
    (ids!(dayclose.close_row), 760.0),
    // Two equal columns of 8 world lines when wide enough; else stack.
    (ids!(menu.menu_list), 720.0),
    (ids!(menu.menu_photos), 700.0),
    (ids!(start.start_world), 700.0),
    (ids!(start.close_cta.buttons), 420.0),
    (ids!(start.footer), 480.0),
];
/// Fixed-width props: (path, desk width, the min width of their row).
/// When the row stacks, the prop goes full width.
const PROPS: [(&[LiveId], f64, f64); 4] = [
    (ids!(rush.floor_card), 560.0, 1024.0),
    (ids!(kot.ticket), 330.0, 758.0),
    (ids!(order.order_card), 360.0, 760.0),
    (ids!(settle.bill), 360.0, 800.0),
];
const DETAIL_WHEN_STACKED: [&[LiveId]; 4] = [
    ids!(order.order_points),
    ids!(kitchen.kds_points),
    ids!(settle.settle_points),
    ids!(dayclose.close_points),
];
const DETAIL_ON_PHONE: [&[LiveId]; 9] = [
    ids!(changes.changes_note),
    // A phone floor shows four tables (two by two).
    ids!(floor.t3),
    ids!(floor.t6),
    ids!(kot.kot_points),
    ids!(menu.menu_note),
    ids!(menu.menu_photos),
    ids!(rush.floor_note),
    ids!(rush.rush_banner),
    ids!(start.start_world),
];
/// Detail a short, not-wide window (under 860 px tall and 1200 px wide,
/// e.g. a 1024 × 768 laptop) drops so the chapter's buttons stay on
/// screen. Wider short windows (1280 × 720, 1366 × 768) fit everything.
const DETAIL_WHEN_SHORT: [&[LiveId]; 5] = [
    ids!(settle.settle_free),
    ids!(rush.rush_banner),
    ids!(menu.menu_photos),
    ids!(menu.menu_script),
    ids!(rush.floor_note),
];
const SHORT_BELOW: f64 = 860.0;
const SHORT_NARROWER_THAN: f64 = 1200.0;

const MAX_CONTENT: f64 = 1160.0;
const MAX_CONTENT_WIDE: f64 = 1320.0;

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
        let by_width = if w < 480.0 {
            Self::Phone
        } else if w < 720.0 {
            Self::Narrow
        } else if w < 1024.0 {
            Self::Tablet
        } else if w >= 1600.0 && h >= 900.0 {
            Self::Wide
        } else {
            Self::Desk
        };
        let steps_down = if h < 480.0 {
            2
        } else if h < 660.0 {
            1
        } else {
            0
        };
        (0..steps_down).fold(by_width, |t, _| t.smaller())
    }

    fn smaller(self) -> Self {
        match self {
            Self::Wide => Self::Desk,
            Self::Desk => Self::Tablet,
            Self::Tablet => Self::Narrow,
            _ => Self::Phone,
        }
    }

    /// Content never grows wider than this; wider windows get wider margins.
    fn max_content(self) -> f64 {
        if self == Self::Wide { MAX_CONTENT_WIDE } else { MAX_CONTENT }
    }

    fn stacked(self) -> bool {
        matches!(self, Self::Phone | Self::Narrow)
    }

    fn type_scale(self, desk_size: f32) -> f32 {
        let (display, heading, body) = match self {
            Self::Phone => (0.6, 0.86, 0.94),
            Self::Narrow => (0.7, 0.9, 0.97),
            Self::Tablet => (0.84, 0.95, 1.0),
            Self::Desk => (1.0, 1.0, 1.0),
            Self::Wide => (1.18, 1.08, 1.04),
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

    fn spacing(self) -> (f64, f64, f64) {
        match self {
            Self::Phone => (16.0, 14.0, 12.0),
            Self::Narrow => (26.0, 20.0, 14.0),
            Self::Tablet => (40.0, 26.0, 18.0),
            Self::Desk => (60.0, 30.0, 20.0),
            Self::Wide => (80.0, 44.0, 26.0),
        }
    }

    /// Floor toy tile size: two columns on a phone, three elsewhere.
    fn tile(self) -> (f64, f64) {
        match self {
            Self::Phone => (150.0, 100.0),
            Self::Wide => (160.0, 106.0),
            _ => (160.0, 100.0),
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
    desk_font: Vec<Vec<f32>>,
    #[rust]
    chapter: usize,
    #[rust]
    tables: Option<[Visit; 6]>,
    #[rust]
    focused_deck: bool,
}

impl App {
    fn slides(&self, cx: &mut Cx) -> SlidesViewRef {
        self.ui.slides_view(cx, ids!(slides))
    }

    fn go_to(&mut self, cx: &mut Cx, chapter: usize) {
        let chapter = chapter.min(SLIDES.len() - 1);
        self.slides(cx).set_goal_slide(cx, chapter);
        self.paint_chrome(cx, chapter);
    }

    fn paint_chrome(&mut self, cx: &mut Cx, chapter: usize) {
        self.chapter = chapter;
        self.ui
            .widget(cx, ids!(chapter_name))
            .set_text(cx, CHAPTERS[chapter].1);
        self.ui
            .widget(cx, ids!(counter))
            .set_text(cx, &format!("{} / {}", chapter + 1, CHAPTERS.len()));
        for (i, [_, idle, here]) in NAV.iter().enumerate() {
            self.ui.view(cx, idle).set_visible(cx, i != chapter);
            self.ui.view(cx, here).set_visible(cx, i == chapter);
        }
        self.ui.redraw(cx);
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
        for path in [ids!(rush.rush_cta.note), ids!(start.close_cta.note)] {
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

    fn apply_gates(&mut self, cx: &mut Cx) {
        for path in [ids!(kot.kot_lan), ids!(offline.offline_lan)] {
            self.ui.widget(cx, path).set_visible(cx, LAN_PILOT_PASSED);
        }
    }

    fn chapter_from_page(&self, cx: &Cx) -> Option<usize> {
        let OsType::Web(page) = cx.os_type() else {
            return None;
        };
        chapter_from_hash(&page.hash)
    }

    /// One pass over every optional detail: hidden when stacked (phone /
    /// narrow), on the smallest tier, or on a short window — whichever rules
    /// list it. Runs on every layout so the rules never overwrite each other.
    fn paint_details(&mut self, cx: &mut Cx, tier: Tier, short: bool) {
        let all = DETAIL_WHEN_STACKED
            .iter()
            .chain(DETAIL_ON_PHONE.iter())
            .chain(DETAIL_WHEN_SHORT.iter());
        for path in all {
            let hide = (tier.stacked() && DETAIL_WHEN_STACKED.contains(path))
                || (tier == Tier::Phone && DETAIL_ON_PHONE.contains(path))
                || (short && DETAIL_WHEN_SHORT.contains(path));
            self.ui.widget(cx, path).set_visible(cx, !hide);
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

        let (min_side, pad_v, spacing) = tier.spacing();
        let side = min_side.max((layout_w - tier.max_content()) / 2.0);
        let content_w = layout_w - 2.0 * side;
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
        // Keep the world shelf + CTA; drop the long lede only when stacked and short.
        let short = size.y < SHORT_BELOW && layout_w < SHORT_NARROWER_THAN;
        self.ui
            .widget(cx, ids!(rush.rush_lede))
            .set_visible(cx, !(tier.stacked() && short));
        self.paint_details(cx, tier, short);
        for path in SLIDES {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.layout.padding = Inset {
                    left: side,
                    right: side,
                    top: pad_v,
                    bottom: pad_v * 1.2 + 6.0,
                };
                view.layout.spacing = spacing;
            }
        }
        // Top bar: tighter padding on phone so nav tiles stay on-screen.
        if let Some(mut bar) = self.ui.view(cx, ids!(top_bar)).borrow_mut() {
            let pad = if tier == Tier::Phone { 10.0 } else { 24.0 };
            bar.layout.padding = Inset {
                left: pad,
                right: pad.min(20.0),
                top: 0.0,
                bottom: 0.0,
            };
        }
        if tier_changed {
            let stacked = tier.stacked();
            let (tile_w, tile_h) = tier.tile();
            for (path, _) in TABLES {
                if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                    view.walk.width = Size::Fixed(tile_w);
                    view.walk.height = Size::Fixed(tile_h);
                }
            }
            let phone = tier == Tier::Phone;
            self.ui.widget(cx, ids!(tagline)).set_visible(cx, !phone);
            self.ui.widget(cx, ids!(chapter_name)).set_visible(cx, !stacked);
            let first_pass = self.desk_font.is_empty();
            for (i, path) in SLIDES.iter().enumerate() {
                let slide = self.ui.widget(cx, path);
                if first_pass {
                    let mut sizes = Vec::new();
                    collect_label_sizes(&slide, &mut sizes);
                    self.desk_font.push(sizes);
                }
                let mut next = 0;
                scale_labels(&slide, tier, &self.desk_font[i], &mut next);
            }
        }
        self.ui.redraw(cx);
    }

    fn focus_deck_once(&mut self, cx: &mut Cx) {
        if self.focused_deck {
            return;
        }
        let area = self.ui.widget(cx, ids!(slides)).area();
        if area.is_valid(cx) {
            cx.set_key_focus(area);
            self.focused_deck = true;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_native_preview(&mut self, cx: &mut Cx) {
        let Ok(spec) = std::env::var("SITE_PREVIEW") else {
            return;
        };
        let (size, chapter) = parse_preview(&spec);
        if let Some((w, h)) = size {
            self.ui
                .window(cx, ids!(main_window))
                .resize(cx, dvec2(w, h));
        }
        if let Some(chapter) = chapter {
            self.slides(cx).set_current_slide(cx, chapter);
            self.paint_chrome(cx, chapter);
        }
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

fn chapter_from_hash(hash: &str) -> Option<usize> {
    let key = hash.trim_start_matches('#').trim();
    if key.is_empty() {
        return None;
    }
    if let Ok(n) = key.parse::<usize>() {
        return (1..=CHAPTERS.len()).contains(&n).then(|| n - 1);
    }
    CHAPTERS.iter().position(|(id, _)| *id == key)
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn parse_preview(spec: &str) -> (Option<(f64, f64)>, Option<usize>) {
    let (size_part, chapter_part) = match spec.split_once(':') {
        Some((s, c)) => (s, Some(c)),
        None if spec.contains('x') => (spec, None),
        None => ("", Some(spec)),
    };
    let size = size_part.split_once('x').and_then(|(w, h)| {
        let w = w.trim().parse::<f64>().ok()?;
        let h = h.trim().parse::<f64>().ok()?;
        (w >= 200.0 && h >= 200.0).then_some((w, h))
    });
    let chapter = chapter_part.and_then(chapter_from_hash);
    (size, chapter)
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
        let start = self.chapter_from_page(cx).unwrap_or(0);
        if start > 0 {
            self.slides(cx).set_current_slide(cx, start);
        }
        self.paint_chrome(cx, start);
        #[cfg(not(target_arch = "wasm32"))]
        self.apply_native_preview(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(chapter) = self.slides(cx).flipped(actions) {
            self.paint_chrome(cx, chapter);
        }
        for (i, [hit, _, _]) in NAV.iter().enumerate() {
            if self.ui.button(cx, hit).clicked(actions) {
                self.go_to(cx, i);
            }
        }
        self.handle_floor(cx, actions);
        let download = self.ui.button(cx, ids!(rush.rush_cta.download)).clicked(actions)
            || self.ui.button(cx, ids!(start.close_cta.download)).clicked(actions);
        if download {
            self.download(cx);
        }
        let whatsapp = self.ui.button(cx, ids!(rush.rush_cta.whatsapp)).clicked(actions)
            || self.ui.button(cx, ids!(start.close_cta.whatsapp)).clicked(actions);
        if whatsapp {
            self.whatsapp(cx);
        }
        if self.ui.button(cx, ids!(start.to_kumkaj)).clicked(actions) {
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
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        match event {
            Event::Startup | Event::WindowGeomChange(_) => self.apply_responsive(cx),
            Event::Draw(_) | Event::NextFrame(_) => {
                if self.laid_out.is_none() {
                    self.apply_responsive(cx);
                }
                self.focus_deck_once(cx);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_opens_a_chapter_by_id_or_number() {
        assert_eq!(chapter_from_hash("#kot"), Some(2));
        assert_eq!(chapter_from_hash("#10"), Some(9));
        assert_eq!(chapter_from_hash("#11"), None);
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
}

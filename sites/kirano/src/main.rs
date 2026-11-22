//! kirano.shop — Kirano's site as a fullpage Makepad 2 app (wasm in the
//! browser; the same crate runs native for previews and as an offline
//! presenter at a grocery counter). Story: apps/kirano/docs/KIRANO_WEBSITE.md.
//! Copied from sites/khaynjo, then diverged (copy-then-diverge; no shared
//! site engine). Grocery counter story — no Floor / Kitchen / KOT / tables.
//!
//! One scrolling page: sticky top bar with chapter tiles; sections brand ·
//! sell · khata · buy · daybook · lan · free · start · footer.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:sell` opens a 390×844 window scrolled to a
//!   section (id, old chapter number, or nothing for the top).
//! - `SITE_LINKS=local` sends the kumkaj.com link to the local wasm server.
//!
//! In the browser, `#sell` or `#2` opens that section; a page served from
//! 127.0.0.1 / localhost links back to the local kumkaj.com (:8010).

pub use makepad_widgets;

use makepad_widgets::*;

mod counter;
use counter::BillStep;

app_main!(App);

include!("ui.rs");

// ---- Owner switches (docs/WEBSITES.md §7 launch gates, §8 owner inputs) --

/// The multi-device half of `offline-lan` waits for the owner's real-hardware
/// LAN pilot (docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md §3).
const LAN_PILOT_PASSED: bool = false;

/// Download links go live only after the installers pass a clean-machine
/// install. Until then the button says so instead of linking.
const DOWNLOADS_LIVE: bool = false;
/// Where "Download free" goes once `DOWNLOADS_LIVE` is true.
const DOWNLOAD_URL: &str = "https://kirano.shop/download";

/// TODO(owner): Kirano's WhatsApp Business number — digits only, with the
/// country code (no "+", no spaces). Agents must not invent one. Until it
/// is set, the button says the demo line is coming and shows the message.
const WHATSAPP_NUMBER: Option<&str> = None;
const WHATSAPP_MESSAGE: &str = "Kirano demo — shop type, city";

/// Same-host legal pages (deploy-firebase.sh stages them next to the wasm).
const PRIVACY_PATH: &str = "/privacy";
const TERMS_PATH: &str = "/terms/";
const PUBLIC_HOST: &str = "https://kirano.shop";

/// App layout floor: design for ≥320 CSS px (360 is the common Android phone width). Narrower viewports scroll
/// horizontally (web shell CSS); native still lays out at this width.
const MIN_WIDTH: f64 = 320.0;

// ---- Sections (KIRANO_WEBSITE.md §2 Phase 1) ---------------------------

const CHAPTERS: [(&str, &str); 8] = [
    ("brand", "Busy counter"),
    ("sell", "Counter sell"),
    ("khata", "Khata"),
    ("buy", "Buy & stock"),
    ("daybook", "Day book"),
    ("lan", if LAN_PILOT_PASSED { "Extra till" } else { "Offline" }),
    ("free", "Free forever"),
    ("start", "Start"),
];

const SECTIONS: [(&str, &[LiveId]); 8] = [
    ("brand", ids!(brand)),
    ("sell", ids!(sell)),
    ("khata", ids!(khata)),
    ("buy", ids!(buy)),
    ("daybook", ids!(daybook)),
    ("lan", ids!(lan)),
    ("free", ids!(free)),
    ("start", ids!(start)),
];

/// Top-bar tiles: [hit, idle, here].
const NAV: [[&[LiveId]; 3]; 8] = [
    [ids!(nav.n0.hit), ids!(nav.n0.idle), ids!(nav.n0.here)],
    [ids!(nav.n1.hit), ids!(nav.n1.idle), ids!(nav.n1.here)],
    [ids!(nav.n2.hit), ids!(nav.n2.idle), ids!(nav.n2.here)],
    [ids!(nav.n3.hit), ids!(nav.n3.idle), ids!(nav.n3.here)],
    [ids!(nav.n4.hit), ids!(nav.n4.idle), ids!(nav.n4.here)],
    [ids!(nav.n5.hit), ids!(nav.n5.idle), ids!(nav.n5.here)],
    [ids!(nav.n6.hit), ids!(nav.n6.idle), ids!(nav.n6.here)],
    [ids!(nav.n7.hit), ids!(nav.n7.idle), ids!(nav.n7.here)],
];

const BILL_LINES: [(&[LiveId], &str, u32); 3] = [
    (ids!(bill.line1), counter::LINES[0].0, counter::LINES[0].1),
    (ids!(bill.line2), counter::LINES[1].0, counter::LINES[1].1),
    (ids!(bill.line3), counter::LINES[2].0, counter::LINES[2].1),
];

/// Rows that sit side by side only when the content column is at least
/// this wide (px), and stack otherwise.
const ROWS: [(&[LiveId], f64); 10] = [
    (ids!(brand.brand_row), 1024.0),
    (ids!(brand.brand_cta.buttons), 420.0),
    (ids!(sell.sell_row), 760.0),
    (ids!(khata.khata_row), 760.0),
    (ids!(buy.buy_row), 760.0),
    (ids!(daybook.day_row), 800.0),
    (ids!(lan.lan_row), 700.0),
    (ids!(free.free_row), 700.0),
    (ids!(start.close_cta.buttons), 420.0),
    (ids!(footer.footer_row), 480.0),
];
const PROPS: [(&[LiveId], f64, f64); 2] = [
    (ids!(brand.bill_card), 420.0, 1024.0),
    (ids!(sell.ticket), 330.0, 760.0),
];

const SECTION_WRAPS: [&[LiveId]; 9] = [
    ids!(brand),
    ids!(sell),
    ids!(khata),
    ids!(buy),
    ids!(daybook),
    ids!(lan),
    ids!(free),
    ids!(start),
    ids!(footer),
];

/// The eight top-bar tiles (their width shrinks on the narrowest phones).
const NAV_TILES: [&[LiveId]; 8] = [
    ids!(nav.n0),
    ids!(nav.n1),
    ids!(nav.n2),
    ids!(nav.n3),
    ids!(nav.n4),
    ids!(nav.n5),
    ids!(nav.n6),
    ids!(nav.n7),
];

const MAX_CONTENT: f64 = 1160.0;
const MAX_CONTENT_WIDE: f64 = 1320.0;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tier {
    Phone,
    Narrow,
    Tablet,
    Desk,
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

    fn max_content(self) -> f64 {
        if self == Self::Wide {
            MAX_CONTENT_WIDE
        } else {
            MAX_CONTENT
        }
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
            // pad_v was 14 — section heads sat flush under the sticky bar
            // when scrolled (phone evidence 2026-11-22: "wholesaler" eaten).
            Self::Phone => (16.0, 36.0, 14.0),
            Self::Narrow => (26.0, 20.0, 14.0),
            Self::Tablet => (40.0, 26.0, 18.0),
            Self::Desk => (60.0, 30.0, 20.0),
            Self::Wide => (80.0, 44.0, 26.0),
        }
    }
}

type PendingSection = Option<&'static str>;

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    laid_out: Option<(Tier, i64, i64)>,
    #[rust]
    desk_font: Vec<f32>,
    #[rust]
    chapter: usize,
    #[rust]
    bill: Option<BillStep>,
    #[rust]
    pending_section: PendingSection,
    #[rust]
    pending_frames: u8,
    #[rust]
    pending_last_y: f64,
    #[rust]
    pending_stable: u8,
}

impl App {
    fn page(&self, cx: &mut Cx) -> ViewRef {
        self.ui.view(cx, ids!(page))
    }

    fn scroll_to(&mut self, cx: &mut Cx, section: &str) -> f64 {
        let Some(&(id, path)) = SECTIONS.iter().find(|(sid, _)| *sid == section) else {
            return 0.0;
        };
        let page = self.page(cx);
        let page_rect = page.area().rect(cx);
        let target = self.ui.widget(cx, path).area().rect(cx);
        let top = self.ui.widget(cx, ids!(brand)).area().rect(cx);
        if page_rect.size.y <= 0.0 {
            self.pending_section = Some(id);
            return 0.0;
        }
        // Leave a little previous-band peek so the heading is not flush
        // against the sticky top bar (scroll-margin under chrome).
        let y = (target.pos.y - top.pos.y - 20.0).max(0.0);
        page.set_scroll_pos(cx, dvec2(0.0, y));
        if let Some(i) = CHAPTERS.iter().position(|(cid, _)| *cid == id) {
            self.paint_chrome(cx, i);
        }
        self.ui.redraw(cx);
        y
    }

    fn go_to(&mut self, cx: &mut Cx, chapter: usize) {
        let chapter = chapter.min(CHAPTERS.len() - 1);
        let _ = self.scroll_to(cx, CHAPTERS[chapter].0);
    }

    fn paint_chrome(&mut self, cx: &mut Cx, chapter: usize) {
        self.chapter = chapter;
        self.ui
            .label(cx, ids!(chapter_name))
            .set_text(cx, CHAPTERS[chapter].1);
        self.ui
            .label(cx, ids!(counter))
            .set_text(cx, &format!("{} / {}", chapter + 1, CHAPTERS.len()));
        for (i, [_, idle, here]) in NAV.iter().enumerate() {
            self.ui.view(cx, idle).set_visible(cx, i != chapter);
            self.ui.view(cx, here).set_visible(cx, i == chapter);
        }
    }

    fn active_chapter(&self, cx: &Cx) -> usize {
        let page = self.ui.view(cx, ids!(page));
        if !page.area().is_valid(cx) {
            return self.chapter;
        }
        let bar = self.ui.view(cx, ids!(top_bar));
        let mark = if bar.area().is_valid(cx) {
            bar.area().rect(cx).pos.y + bar.area().rect(cx).size.y + 12.0
        } else {
            page.area().rect(cx).pos.y + 72.0
        };
        let mut idx = 0usize;
        for (i, (_, path)) in SECTIONS.iter().enumerate() {
            let w = self.ui.widget(cx, path);
            if !w.area().is_valid(cx) {
                continue;
            }
            if w.area().rect(cx).pos.y <= mark {
                idx = i;
            }
        }
        idx
    }

    fn paint_scroll_chrome(&mut self, cx: &mut Cx) {
        let active = self.active_chapter(cx);
        if active != self.chapter {
            self.paint_chrome(cx, active);
        }
    }

    fn paint_bill(&mut self, cx: &mut Cx) {
        let step = self.bill.unwrap_or(BillStep::Ready);
        let shown = step.lines_shown();
        self.ui
            .widget(cx, ids!(bill.status))
            .set_text(cx, step.word());
        for (i, (path, name, amount)) in BILL_LINES.iter().enumerate() {
            let visible = i < shown;
            let row = self.ui.widget(cx, path);
            row.set_visible(cx, visible);
            if visible {
                row.widget(cx, ids!(name)).set_text(cx, name);
                row.widget(cx, ids!(amt))
                    .set_text(cx, &format!("₹ {}", counter::rupees(*amount)));
            }
        }
        let total = step.total();
        let total_visible = shown > 0 || step == BillStep::Settling || step == BillStep::Paid;
        self.ui
            .widget(cx, ids!(bill.total_row))
            .set_visible(cx, total_visible);
        self.ui
            .widget(cx, ids!(bill.total_amt))
            .set_text(cx, &format!("₹ {}", counter::rupees(total)));
        self.ui
            .widget(cx, ids!(bill.tender))
            .set_text(cx, step.tender_line());
        self.ui.redraw(cx);
    }

    fn handle_bill(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(cx, ids!(bill.hit)).clicked(actions) {
            let next = self.bill.unwrap_or(BillStep::Ready).next();
            self.bill = Some(next);
            self.paint_bill(cx);
        }
    }

    fn set_cta_notes(&mut self, cx: &mut Cx, text: &str) {
        for path in [ids!(brand.brand_cta.note), ids!(start.close_cta.note)] {
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

    fn site_url(&self, cx: &Cx, path: &str) -> String {
        if matches!(cx.os_type(), OsType::Web(_)) {
            path.to_string()
        } else {
            format!("{PUBLIC_HOST}{path}")
        }
    }

    fn apply_gates(&mut self, cx: &mut Cx) {
        for path in [ids!(lan.lan_extra), ids!(lan.lan_devices)] {
            self.ui.widget(cx, path).set_visible(cx, LAN_PILOT_PASSED);
        }
        // The chapter's own title is part of the gated claim.
        if LAN_PILOT_PASSED {
            self.ui
                .widget(cx, ids!(lan.lan_eyebrow))
                .set_text(cx, "6 · EXTRA TILL");
            self.ui
                .widget(cx, ids!(lan.lan_headline))
                .set_text(cx, "Another till on the shop Wi‑Fi.");
        }
    }

    fn section_from_page(&self, cx: &Cx) -> Option<&'static str> {
        let OsType::Web(page) = cx.os_type() else {
            return None;
        };
        section_from_hash(&page.hash)
    }

    /// Receipt lines are up to 25 columns of a mono face. Under 360 px the
    /// paper is narrower than that at desk size (each line broke before its
    /// amount at 320), so the lines step down and stay whole.
    fn fit_ticket_lines(&mut self, cx: &mut Cx, layout_w: f64) {
        let size: f32 = if layout_w < 360.0 { 12.0 } else { 14.0 };
        let lines = self.ui.widget(cx, ids!(sell.ticket.lines));
        lines.children(&mut |_, child| {
            if let Some(mut md) = child.borrow_mut::<Markdown>() {
                md.font_size = size;
                md.draw_text.text_style.font_size = size;
                md.text_style_normal.font_size = size;
                md.text_style_bold.font_size = size;
            }
        });
    }

    fn apply_responsive(&mut self, cx: &mut Cx) {
        let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
        if size.x < 1.0 || size.y < 1.0 {
            return;
        }
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
                view.layout.flow = if content_w >= min_w {
                    Flow::right()
                } else {
                    Flow::Down
                };
            }
        }
        for (path, desk_w, row_min_w) in PROPS {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.walk.width = if content_w >= row_min_w {
                    Size::Fixed(desk_w)
                } else {
                    Size::fill()
                };
            }
        }
        let brand_across = ROWS
            .iter()
            .find(|(path, _)| *path == ids!(brand.brand_row))
            .is_some_and(|(_, min_w)| content_w >= *min_w);
        self.ui
            .widget(cx, ids!(brand.brand_lede))
            .set_visible(cx, brand_across || (size.y >= 900.0 && !tier.stacked()));
        for path in SECTION_WRAPS {
            let band = self.ui.view(cx, path);
            if let Some(mut view) = band.borrow_mut() {
                let footer = path == ids!(footer);
                view.layout.padding = Inset {
                    left: side,
                    right: side,
                    top: if footer {
                        pad_v * 0.8
                    } else {
                        pad_v
                    },
                    bottom: if footer {
                        pad_v * 1.1
                    } else {
                        pad_v * 1.2 + 6.0
                    },
                };
                if !footer {
                    view.layout.spacing = spacing;
                }
            }
        }
        let bar = self.ui.view(cx, ids!(top_bar));
        if let Some(mut view) = bar.borrow_mut() {
            let edge = if tier.stacked() {
                16.0
            } else {
                side.min(40.0).max(20.0)
            };
            view.layout.padding.left = edge;
            view.layout.padding.right = edge - 4.0;
            // Phone: brand + eight tiles must fit in 375. Drop the gaps.
            view.layout.spacing = if tier == Tier::Phone { 6.0 } else { 12.0 };
        }
        if let Some(mut nav) = self.ui.view(cx, ids!(nav)).borrow_mut() {
            nav.layout.spacing = if tier == Tier::Phone { 0.0 } else { 2.0 };
        }
        // Under 360 px the brand and eight 22 px tiles are wider than the bar:
        // the last tile was cut off at 320.
        let tile_w = if layout_w < 360.0 { 18.0 } else { 22.0 };
        for path in NAV_TILES {
            if let Some(mut tile) = self.ui.view(cx, path).borrow_mut() {
                tile.walk.width = Size::Fixed(tile_w);
            }
        }
        if tier_changed {
            let stacked = tier.stacked();
            let phone = tier == Tier::Phone;
            // Tagline plus the chapter name overflow the fixed 58px bar
            // from 480px through the rest of Narrow.
            self.ui.label(cx, ids!(tagline)).set_visible(cx, !stacked);
            self.ui
                .label(cx, ids!(chapter_name))
                .set_visible(cx, !stacked);
            self.ui.label(cx, ids!(counter)).set_visible(cx, !phone);
            let page = self.ui.widget(cx, ids!(page));
            if self.desk_font.is_empty() {
                collect_label_sizes(&page, &mut self.desk_font);
            }
            let mut next = 0;
            scale_labels(&page, tier, &self.desk_font, &mut next);
        }
        self.fit_ticket_lines(cx, layout_w);
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

/// `#sell`, `#2` (1-based chapter number) → section id.
fn section_from_hash(hash: &str) -> Option<&'static str> {
    let key = hash.trim_start_matches('#').trim().to_ascii_lowercase();
    if key.is_empty() {
        return None;
    }
    if let Some((id, _)) = CHAPTERS.iter().find(|(id, _)| *id == key) {
        return Some(id);
    }
    let n: usize = key.parse().ok()?;
    CHAPTERS
        .get(n.checked_sub(1)?)
        .map(|(id, _)| *id)
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
        self.paint_bill(cx);
        self.paint_chrome(cx, 0);
        self.page(cx).set_key_focus(cx);
        self.pending_section = self.section_from_page(cx);
        self.pending_frames = 120;
        self.pending_stable = 0;
        #[cfg(not(target_arch = "wasm32"))]
        self.apply_native_preview(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (i, [hit, _, _]) in NAV.iter().enumerate() {
            if self.ui.button(cx, hit).clicked(actions) {
                self.go_to(cx, i);
            }
        }
        self.handle_bill(cx, actions);
        let download = self.ui.button(cx, ids!(brand.brand_cta.download)).clicked(actions)
            || self.ui.button(cx, ids!(start.close_cta.download)).clicked(actions);
        if download {
            self.download(cx);
        }
        let whatsapp = self.ui.button(cx, ids!(brand.brand_cta.whatsapp)).clicked(actions)
            || self.ui.button(cx, ids!(start.close_cta.whatsapp)).clicked(actions);
        if whatsapp {
            self.whatsapp(cx);
        }
        if self.ui.button(cx, ids!(start.to_kumkaj)).clicked(actions)
            || self.ui.button(cx, ids!(foot_kumkaj)).clicked(actions)
        {
            let url = self.kumkaj_url(cx);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(to_privacy)).clicked(actions) {
            let url = self.site_url(cx, PRIVACY_PATH);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(to_terms)).clicked(actions) {
            let url = self.site_url(cx, TERMS_PATH);
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
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        match event {
            Event::Startup | Event::WindowGeomChange(_) => self.apply_responsive(cx),
            Event::Scroll(_) => self.paint_scroll_chrome(cx),
            Event::Draw(_) | Event::NextFrame(_) => {
                if self.laid_out.is_none() {
                    self.apply_responsive(cx);
                }
                self.paint_scroll_chrome(cx);
                if let Some(section) = self.pending_section {
                    let page_h = self.page(cx).area().rect(cx).size.y;
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
    fn hash_opens_a_section_by_id_or_number() {
        assert_eq!(section_from_hash("#sell"), Some("sell"));
        assert_eq!(section_from_hash("#1"), Some("brand"));
        assert_eq!(section_from_hash("#8"), Some("start"));
        assert_eq!(section_from_hash("#9"), None);
        assert_eq!(section_from_hash("#0"), None);
        assert_eq!(section_from_hash(""), None);
        assert_eq!(section_from_hash("#nope"), None);
    }

    #[test]
    fn preview_spec_parses_size_and_section() {
        assert_eq!(
            parse_preview("390x844:sell"),
            (Some((390.0, 844.0)), Some("sell"))
        );
        assert_eq!(parse_preview("1280x800"), (Some((1280.0, 800.0)), None));
        assert_eq!(parse_preview("khata"), (None, Some("khata")));
        assert_eq!(parse_preview("10x10:2"), (None, Some("sell")));
    }

    #[test]
    fn owner_switches_ship_closed() {
        assert!(WHATSAPP_NUMBER.is_none());
        assert!(!DOWNLOADS_LIVE);
        assert!(!LAN_PILOT_PASSED);
    }

    #[test]
    fn whatsapp_prefill_is_percent_encoded() {
        assert_eq!(
            url_encode(WHATSAPP_MESSAGE),
            "Kirano%20demo%20%E2%80%94%20shop%20type%2C%20city"
        );
    }

    #[test]
    fn download_url_is_kirano_shop() {
        assert!(DOWNLOAD_URL.contains("kirano.shop"));
        assert!(!DOWNLOAD_URL.contains("kirano.com"));
    }

    #[test]
    fn public_host_is_kirano_shop() {
        assert_eq!(PUBLIC_HOST, "https://kirano.shop");
    }

    #[test]
    fn min_width_matches_app_floor() {
        assert_eq!(MIN_WIDTH, 320.0);
        assert_eq!(Tier::from_size(MIN_WIDTH, 800.0), Tier::Phone);
    }
}
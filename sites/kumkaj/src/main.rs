//! kumkaj.com — the Kumkaj brand house as a fullpage Makepad 2 app
//! (wasm in the browser; the same crate runs native for previews and as an
//! offline presenter). Story and rules: docs/WEBSITES.md.
//!
//! One scrolling page: sticky top bar with chapter dots; sections hero ·
//! trade · books · offline · free · languages · gotigin · footer.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:trade` opens a 390×844 window scrolled to a
//!   section (id, old chapter number, or nothing for the top).
//! - `SITE_LINKS=local` sends product links to the local wasm servers.
//!
//! In the browser, `#trade` or `#2` in the URL opens that section, and a
//! page served from 127.0.0.1 / localhost links to the local product sites
//! (moklanjo :8011, khaynjo :8012, kirano :8013, tabiyat :8014) instead of
//! the public domains.

pub use makepad_widgets;

use makepad_widgets::*;

app_main!(App);

include!("ui.rs");

/// Multi-device sentences ("extra counters and tablets join over your own
/// Wi‑Fi") stay hidden until the owner's real-hardware LAN pilot passes
/// (docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md §3, the `lan-pilot` gate in
/// both product website docs). The owner flips this; agents do not.
const LAN_PILOT_PASSED: bool = false;

/// Same-host legal pages (deploy-firebase.sh stages them next to the wasm).
const PRIVACY_PATH: &str = "/privacy";
const TERMS_PATH: &str = "/terms/";
const CANCELLATION_PATH: &str = "/cancellation";
const PUBLIC_HOST: &str = "https://kumkaj.com";

/// App layout floor: design for ≥320 CSS px (360 is the common Android phone width). Narrower viewports scroll
/// horizontally (web shell CSS); native still lays out at this width.
const MIN_WIDTH: f64 = 320.0;

/// Chapter ids (URL hash) and the names shown in the top bar.
const CHAPTERS: [(&str, &str); 7] = [
    ("hero", "The promise"),
    ("trade", "Built for your trade"),
    ("books", "Your books"),
    ("offline", "Try it offline"),
    ("free", "Price"),
    ("languages", "Languages"),
    ("gotigin", "Who makes it"),
];

/// Page sections in order: (hash id, widget path).
const SECTIONS: [(&str, &[LiveId]); 7] = [
    ("hero", ids!(hero)),
    ("trade", ids!(trade)),
    ("books", ids!(books)),
    ("offline", ids!(offline)),
    ("free", ids!(free)),
    ("languages", ids!(languages)),
    ("gotigin", ids!(gotigin)),
];

const DOTS: [[&[LiveId]; 3]; 7] = [
    [ids!(dots.d0.hit), ids!(dots.d0.idle), ids!(dots.d0.on)],
    [ids!(dots.d1.hit), ids!(dots.d1.idle), ids!(dots.d1.on)],
    [ids!(dots.d2.hit), ids!(dots.d2.idle), ids!(dots.d2.on)],
    [ids!(dots.d3.hit), ids!(dots.d3.idle), ids!(dots.d3.on)],
    [ids!(dots.d4.hit), ids!(dots.d4.idle), ids!(dots.d4.on)],
    [ids!(dots.d5.hit), ids!(dots.d5.idle), ids!(dots.d5.on)],
    [ids!(dots.d6.hit), ids!(dots.d6.idle), ids!(dots.d6.on)],
];

/// The seven dot slots themselves (their width shrinks on a phone).
const DOT_SLOTS: [&[LiveId]; 7] = [
    ids!(dots.d0),
    ids!(dots.d1),
    ids!(dots.d2),
    ids!(dots.d3),
    ids!(dots.d4),
    ids!(dots.d5),
    ids!(dots.d6),
];

/// Rows that sit side by side only when the content column is at least
/// this wide (px), and stack otherwise — width-based, not tier-based: a
/// fixed prop beside a headline needs its own width plus a readable text
/// column whatever the device.
const ROWS: [(&[LiveId], f64); 10] = [
    (ids!(hero.hero_row), 848.0),
    (ids!(hero.hero_ctas), 480.0),
    // `trade.cards` stays Down (two pairs). Targeting it used to lay the
    // pairs side by side, and the unnamed pair rows never stacked on a phone.
    (ids!(trade.cards.pair_top), 640.0),
    (ids!(trade.cards.pair_bot), 640.0),
    (ids!(books.tiles), 720.0),
    (ids!(offline.toy_head), 420.0),
    // Cloud copy + Planned pill: stack before the pill clips on landscape.
    (ids!(free.prices.cloud_row), 520.0),
    (ids!(gotigin.product_row_1), 640.0),
    (ids!(gotigin.product_row_2), 640.0),
    (ids!(footer.footer_row), 640.0),
];
/// Fixed-width props: (path, desk width, the min width of their row).
/// When the row stacks, the prop goes full width.
const PROPS: [(&[LiveId], f64, f64); 1] = [
    (ids!(hero.status_card), 360.0, 848.0),
];

const MAX_CONTENT: f64 = 1160.0;
const MAX_CONTENT_WIDE: f64 = 1320.0;

/// Section padding targets (every chapter band plus the footer).
const SECTION_WRAPS: [&[LiveId]; 8] = [
    ids!(hero),
    ids!(trade),
    ids!(books),
    ids!(offline),
    ids!(free),
    ids!(languages),
    ids!(gotigin),
    ids!(footer),
];

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
    /// Width picks the tier (the apps' 480 / 720 / 1024 switch points); a
    /// short window (landscape phone, small laptop) steps down so a chapter
    /// still fits one screen.
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
        } else if h < 640.0 {
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
        if self == Self::Wide {
            MAX_CONTENT_WIDE
        } else {
            MAX_CONTENT
        }
    }

    fn stacked(self) -> bool {
        matches!(self, Self::Phone | Self::Narrow)
    }

    /// Display type shrinks most, body text barely, small print never.
    fn type_scale(self, desk_size: f32) -> f32 {
        let (display, heading, body) = match self {
            Self::Phone => (0.58, 0.8, 0.93),
            Self::Narrow => (0.7, 0.88, 0.97),
            Self::Tablet => (0.84, 0.94, 1.0),
            Self::Desk => (1.0, 1.0, 1.0),
            Self::Wide => (1.18, 1.08, 1.04),
        };
        if desk_size >= 40.0 {
            display
        } else if desk_size >= 22.0 {
            heading
        } else if desk_size >= 15.0 {
            body
        } else {
            1.0
        }
    }

    /// (minimum side padding, vertical padding, chapter spacing)
    fn spacing(self) -> (f64, f64, f64) {
        match self {
            Self::Phone => (20.0, 18.0, 14.0),
            Self::Narrow => (28.0, 24.0, 16.0),
            Self::Tablet => (44.0, 30.0, 20.0),
            Self::Desk => (64.0, 36.0, 22.0),
            Self::Wide => (80.0, 44.0, 26.0),
        }
    }
}

#[derive(Clone, Copy)]
enum Product {
    Moklanjo,
    Khaynjo,
    Kirano,
    Tabiyat,
}

impl Product {
    fn domain(self) -> &'static str {
        match self {
            Self::Moklanjo => "moklanjo.com",
            Self::Khaynjo => "khaynjo.com",
            Self::Kirano => "kirano.shop",
            Self::Tabiyat => "tabiyat.shop",
        }
    }

    /// Ports the local run scripts serve each site on.
    fn local_port(self) -> u16 {
        match self {
            Self::Moklanjo => 8011,
            Self::Khaynjo => 8012,
            Self::Kirano => 8013,
            Self::Tabiyat => 8014,
        }
    }
}

/// Section to scroll to after layout (the Script derive wants plain types).
type PendingSection = Option<&'static str>;

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    /// Last laid-out (tier, width, height); re-layout only when it changes.
    #[rust]
    laid_out: Option<(Tier, i64, i64)>,
    /// Desk (DSL) font size of every label per page, in tree order, so tier
    /// changes never compound. (Child refs do not carry stable uids.)
    #[rust]
    desk_font: Vec<f32>,
    #[rust]
    chapter: usize,
    #[rust]
    internet_cut: bool,
    /// Section to scroll to once the page has a layout (hash / preview).
    #[rust]
    pending_section: PendingSection,
    /// Frames left to re-apply the deep-link scroll while layout settles.
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

    /// Scroll the page so `section` sits under the top bar.
    fn scroll_to(&mut self, cx: &mut Cx, section: &str) -> f64 {
        let Some(&(id, path)) = SECTIONS.iter().find(|(sid, _)| *sid == section) else {
            return 0.0;
        };
        let page = self.page(cx);
        let page_rect = page.area().rect(cx);
        let target = self.ui.widget(cx, path).area().rect(cx);
        let top = self.ui.widget(cx, ids!(hero)).area().rect(cx);
        if page_rect.size.y <= 0.0 {
            self.pending_section = Some(id);
            return 0.0;
        }
        let y = (target.pos.y - top.pos.y).max(0.0);
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
        for (i, [_, idle, on]) in DOTS.iter().enumerate() {
            self.ui.view(cx, idle).set_visible(cx, i != chapter);
            self.ui.view(cx, on).set_visible(cx, i == chapter);
        }
    }

    /// Vertical scroll cannot go further (short last chapter never crosses the spy).
    fn at_page_end(&self, cx: &Cx) -> bool {
        let page = self.ui.view(cx, ids!(page));
        if !page.area().is_valid(cx) {
            return false;
        }
        let max = (page.get_scroll_view_total().y - page.get_scroll_view_visible().y).max(0.0);
        let y = page.get_scroll_pos().y;
        max <= 1.0 || y >= max - 4.0
    }

    /// Which content section owns the viewport (just under the top bar).
    fn active_chapter(&self, cx: &Cx) -> usize {
        let page = self.ui.view(cx, ids!(page));
        if !page.area().is_valid(cx) {
            return self.chapter;
        }
        let n = SECTIONS.len();
        if n == 0 {
            return self.chapter;
        }
        if self.at_page_end(cx) {
            return n - 1;
        }
        let bar = self.ui.view(cx, ids!(top_bar));
        let mark = if bar.area().is_valid(cx) {
            bar.area().rect(cx).pos.y + bar.area().rect(cx).size.y + 12.0
        } else {
            page.area().rect(cx).pos.y + 72.0
        };
        let page_rect = page.area().rect(cx);
        let promote_y = page_rect.pos.y + page_rect.size.y * 0.55;
        let mut idx = 0usize;
        for (i, (_, path)) in SECTIONS.iter().enumerate() {
            let w = self.ui.widget(cx, path);
            if !w.area().is_valid(cx) {
                continue;
            }
            let top = w.area().rect(cx).pos.y;
            if top <= mark {
                idx = i;
            }
            if i == n - 1 && top <= promote_y {
                return n - 1;
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

    fn paint_toy(&mut self, cx: &mut Cx) {
        let cut = self.internet_cut;
        self.ui.button(cx, ids!(offline.toggle)).set_text(
            cx,
            if cut {
                "Bring the internet back"
            } else {
                "Cut the internet"
            },
        );
        let state = self.ui.widget(cx, ids!(offline.net_state));
        state.set_text(cx, if cut { "Internet: off" } else { "Internet: on" });
        let color = Vec4f::from_u32(if cut { 0xF2846FFF } else { 0x6CCB94FF });
        if let Some(mut md) = state.borrow_mut::<Markdown>() {
            md.font_color = color;
            md.draw_text.color = color;
        } else if let Some(mut label) = state.borrow_mut::<Label>() {
            label.draw_text.color = color;
        }
        self.ui.widget(cx, ids!(offline.verdict)).set_text(
            cx,
            if cut {
                "Internet off — and nothing on the counter changed."
            } else {
                "The internet is on — and the counter isn't using it."
            },
        );
        self.ui.redraw(cx);
    }

    /// Claims gated on owner steps (docs/WEBSITES.md §1 rule 3).
    fn apply_gates(&mut self, cx: &mut Cx) {
        self.ui
            .widget(cx, ids!(books.books_lan))
            .set_visible(cx, LAN_PILOT_PASSED);
    }

    fn product_url(&self, cx: &Cx, product: Product) -> String {
        if let OsType::Web(page) = cx.os_type() {
            if page.hostname == "127.0.0.1" || page.hostname == "localhost" {
                return format!(
                    "{}//{}:{}/",
                    page.protocol,
                    page.hostname,
                    product.local_port()
                );
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var("SITE_LINKS").as_deref() == Ok("local") {
            return format!("http://127.0.0.1:{}/", product.local_port());
        }
        format!("https://{}/", product.domain())
    }

    fn open_product(&mut self, cx: &mut Cx, product: Product) {
        let url = self.product_url(cx, product);
        open_link(cx, &url);
    }

    fn site_url(&self, cx: &Cx, path: &str) -> String {
        if matches!(cx.os_type(), OsType::Web(_)) {
            path.to_string()
        } else {
            format!("{PUBLIC_HOST}{path}")
        }
    }

    fn section_from_page(&self, cx: &Cx) -> Option<&'static str> {
        let OsType::Web(page) = cx.os_type() else {
            return None;
        };
        section_from_hash(&page.hash)
    }

    /// Re-lay the page for this window: tier type sizes, stacking, margins.
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
        for path in SECTION_WRAPS {
            let band = self.ui.view(cx, path);
            if let Some(mut view) = band.borrow_mut() {
                let footer = path == ids!(footer);
                view.layout.padding = Inset {
                    left: side,
                    right: side,
                    top: if footer { pad_v * 0.8 } else { pad_v },
                    bottom: if footer { pad_v * 1.1 } else { pad_v * 1.2 },
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
        }
        if tier_changed {
            let stacked = tier.stacked();
            self.ui
                .label(cx, ids!(chapter_name))
                .set_visible(cx, !stacked);
            // Brand + seven dots + "1 / 7" is 395 px: the counter was cut off
            // on a phone. The dots already say where you are.
            self.ui.label(cx, ids!(counter)).set_visible(cx, !stacked);
            // 14 px dots on a phone so brand + dots fit 320.
            let dot_w = if tier == Tier::Phone { 14.0 } else { 20.0 };
            for path in DOT_SLOTS {
                if let Some(mut dot) = self.ui.view(cx, path).borrow_mut() {
                    dot.walk.width = Size::Fixed(dot_w);
                }
            }
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
}

/// Every label/Markdown DSL (desk) font size under `widget`, in tree order.
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

/// Size every label/Markdown under `widget` for the tier from its desk size.
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

/// `#trade`, `#2` (1-based chapter number) → section id.
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

/// `390x844:trade` → (Some((390, 844)), Some("trade")); either half may be omitted.
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

/// Open a link the platform's way: the browser tab in wasm, the system
/// browser on desktop (Makepad's `open_url` is not implemented on Linux).
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
        self.paint_toy(cx);
        self.paint_chrome(cx, 0);
        self.page(cx).set_key_focus(cx);
        self.pending_section = self.section_from_page(cx);
        self.pending_frames = 120;
        self.pending_stable = 0;
        #[cfg(not(target_arch = "wasm32"))]
        self.apply_native_preview(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (i, [hit, _, _]) in DOTS.iter().enumerate() {
            if self.ui.button(cx, hit).clicked(actions) {
                self.go_to(cx, i);
            }
        }
        if self.ui.button(cx, ids!(hero.to_trade)).clicked(actions) {
            self.go_to(cx, 1);
        }
        if self.ui.button(cx, ids!(hero.to_offline)).clicked(actions) {
            self.go_to(cx, 3);
        }
        if self.ui.button(cx, ids!(offline.toggle)).clicked(actions) {
            self.internet_cut = !self.internet_cut;
            self.paint_toy(cx);
        }
        let moklanjo = self.ui.button(cx, ids!(trade.to_moklanjo)).clicked(actions)
            || self.ui.button(cx, ids!(gotigin.close_moklanjo)).clicked(actions);
        if moklanjo {
            self.open_product(cx, Product::Moklanjo);
        }
        let khaynjo = self.ui.button(cx, ids!(trade.to_khaynjo)).clicked(actions)
            || self.ui.button(cx, ids!(gotigin.close_khaynjo)).clicked(actions);
        if khaynjo {
            self.open_product(cx, Product::Khaynjo);
        }
        let kirano = self.ui.button(cx, ids!(trade.to_kirano)).clicked(actions)
            || self.ui.button(cx, ids!(gotigin.close_kirano)).clicked(actions);
        if kirano {
            self.open_product(cx, Product::Kirano);
        }
        let tabiyat = self.ui.button(cx, ids!(trade.to_tabiyat)).clicked(actions)
            || self.ui.button(cx, ids!(gotigin.close_tabiyat)).clicked(actions);
        if tabiyat {
            self.open_product(cx, Product::Tabiyat);
        }
        if self.ui.button(cx, ids!(to_privacy)).clicked(actions) {
            let url = self.site_url(cx, PRIVACY_PATH);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(to_terms)).clicked(actions) {
            let url = self.site_url(cx, TERMS_PATH);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(to_cancellation)).clicked(actions) {
            let url = self.site_url(cx, CANCELLATION_PATH);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(foot_gotigin)).clicked(actions) {
            open_link(cx, "https://gotigin.com/");
        }
        if self.ui.button(cx, ids!(foot_kumkaj)).clicked(actions) {
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
        assert_eq!(section_from_hash("#trade"), Some("trade"));
        assert_eq!(section_from_hash("#1"), Some("hero"));
        assert_eq!(section_from_hash("#7"), Some("gotigin"));
        assert_eq!(section_from_hash("#8"), None);
        assert_eq!(section_from_hash("#0"), None);
        assert_eq!(section_from_hash(""), None);
        assert_eq!(section_from_hash("#nope"), None);
    }

    #[test]
    fn preview_spec_parses_size_and_section() {
        assert_eq!(
            parse_preview("390x844:trade"),
            (Some((390.0, 844.0)), Some("trade"))
        );
        assert_eq!(parse_preview("1280x800"), (Some((1280.0, 800.0)), None));
        assert_eq!(parse_preview("free"), (None, Some("free")));
        assert_eq!(parse_preview("10x10:2"), (None, Some("trade")));
    }

    #[test]
    fn short_windows_step_down_a_tier() {
        assert_eq!(Tier::from_size(1280.0, 800.0), Tier::Desk);
        assert_eq!(Tier::from_size(1280.0, 600.0), Tier::Tablet);
        assert_eq!(Tier::from_size(844.0, 390.0), Tier::Phone);
        assert_eq!(Tier::from_size(390.0, 844.0), Tier::Phone);
        assert_eq!(Tier::from_size(600.0, 900.0), Tier::Narrow);
    }

    #[test]
    fn display_type_shrinks_more_than_body_on_phones() {
        let display = Tier::Phone.type_scale(54.0);
        let body = Tier::Phone.type_scale(15.5);
        assert!(display < body);
        assert_eq!(Tier::Phone.type_scale(13.0), 1.0);
        assert_eq!(Tier::Desk.type_scale(54.0), 1.0);
    }
}
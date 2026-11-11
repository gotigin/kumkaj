//! kumkaj.com — the Kumkaj brand house as a fullpage Makepad 2 app
//! (wasm in the browser; the same crate runs native for previews and as an
//! offline presenter). Story and rules: docs/WEBSITES.md.
//!
//! Scroll / swipe up / tap / ↓ → PgDn Space flip chapters; ↑ ← PgUp go back;
//! Home / End jump. The top-bar dots jump straight to a chapter.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:4` opens a 390×844 window on chapter 4 (1-based).
//! - `SITE_LINKS=local` sends product links to the local wasm servers.
//!
//! In the browser, `#trade` or `#3` in the URL opens that chapter, and a
//! page served from 127.0.0.1 / localhost links to the local product sites
//! (moklanjo :8011, khaynjo :8012, kirano :8013, tabiyat :8014) instead of the public domains.

pub use makepad_widgets;

use makepad_widgets::*;

app_main!(App);

include!("ui.rs");

/// Multi-device sentences ("extra counters and tablets join over your own
/// Wi‑Fi") stay hidden until the owner's real-hardware LAN pilot passes
/// (docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md §3, the `lan-pilot` gate in
/// both product website docs). The owner flips this; agents do not.
const LAN_PILOT_PASSED: bool = false;

/// Chapter ids (URL hash) and the names shown in the top bar.
const CHAPTERS: [(&str, &str); 7] = [
    ("hero", "The promise"),
    ("trade", "Built for your trade"),
    ("books", "Your books"),
    ("offline", "Try it offline"),
    ("free", "Price"),
    ("languages", "Languages"),
    ("gotigin", "Made by Gotigin"),
];

const SLIDES: [&[LiveId]; 7] = [
    ids!(hero),
    ids!(trade),
    ids!(books),
    ids!(offline),
    ids!(free),
    ids!(languages),
    ids!(gotigin),
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

/// Rows that sit side by side only when the content column is at least
/// this wide (px), and stack otherwise — width-based, not tier-based: a
/// fixed prop beside a headline needs its own width plus a readable text
/// column whatever the device.
const ROWS: [(&[LiveId], f64); 7] = [
    (ids!(hero.hero_row), 848.0),
    (ids!(hero.hero_ctas), 480.0),
    (ids!(trade.cards), 640.0),
    (ids!(books.tiles), 720.0),
    (ids!(offline.toy_head), 420.0),
    (ids!(gotigin.map), 720.0),
    (ids!(gotigin.close_ctas), 580.0),
];
/// Fixed-width props: (path, desk width, the min width of their row).
/// When the row stacks, the prop goes full width.
const PROPS: [(&[LiveId], f64, f64); 1] = [
    (ids!(hero.status_card), 360.0, 848.0),
];

/// Detail that a phone screen drops so each chapter still fits one screen.
const DETAIL_WHEN_STACKED: [&[LiveId]; 4] = [
    ids!(hero.status_card),
    ids!(trade.trade_lede),
    ids!(trade.mok_points),
    ids!(trade.kha_points),
];
const DETAIL_ON_PHONE: [&[LiveId]; 5] = [
    ids!(gotigin.gotigin_lede),
    ids!(offline.offline_note),
    ids!(books.books_lede),
    ids!(trade.mok_kicker),
    ids!(trade.kha_kicker),
];
/// Detail a short, not-wide window (under 860 px tall and 1200 px wide,
/// e.g. a 1024 × 768 laptop) drops so the chapter's buttons stay on
/// screen. Wider short windows (1280 × 720, 1366 × 768) fit everything.
const DETAIL_WHEN_SHORT: [&[LiveId]; 3] = [
    ids!(trade.trade_lede),
    ids!(trade.mok_points),
    ids!(trade.kha_points),
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
        if self == Self::Wide { MAX_CONTENT_WIDE } else { MAX_CONTENT }
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

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    /// Last laid-out (tier, width, height); re-layout only when it changes.
    #[rust]
    laid_out: Option<(Tier, i64, i64)>,
    /// Desk (DSL) font size of every label per chapter, in tree order, so
    /// tier changes never compound. (Child refs do not carry stable uids.)
    #[rust]
    desk_font: Vec<Vec<f32>>,
    #[rust]
    chapter: usize,
    #[rust]
    internet_cut: bool,
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
        for (i, [_, idle, on]) in DOTS.iter().enumerate() {
            self.ui.view(cx, idle).set_visible(cx, i != chapter);
            self.ui.view(cx, on).set_visible(cx, i == chapter);
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

    /// The chapter named by the page's `#hash` (`#trade` or `#2`), if any.
    fn chapter_from_page(&self, cx: &Cx) -> Option<usize> {
        let OsType::Web(page) = cx.os_type() else {
            return None;
        };
        chapter_from_hash(&page.hash)
    }

    /// Re-lay the page for this window: tier type sizes, stacking, margins.
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
        let tier = Tier::from_size(size.x, size.y);
        let key = (tier, size.x.round() as i64, size.y.round() as i64);
        if self.laid_out == Some(key) {
            return;
        }
        let tier_changed = self.laid_out.map(|(t, _, _)| t) != Some(tier);
        self.laid_out = Some(key);

        let (min_side, pad_v, spacing) = tier.spacing();
        let side = min_side.max((size.x - tier.max_content()) / 2.0);
        let content_w = size.x - 2.0 * side;
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
        // The brand map's arrows point the way its row runs.
        let map_across = ROWS
            .iter()
            .find(|(path, _)| *path == ids!(gotigin.map))
            .is_some_and(|(_, min_w)| content_w >= *min_w);
        for path in [ids!(gotigin.arrow_1), ids!(gotigin.arrow_2)] {
            self.ui.widget(cx, path).set_text(cx, if map_across { "→" } else { "↓" });
        }
        self.paint_details(cx, tier, size.y < SHORT_BELOW && size.x < SHORT_NARROWER_THAN);
        for path in SLIDES {
            let slide = self.ui.view(cx, path);
            if let Some(mut view) = slide.borrow_mut() {
                view.layout.padding = Inset {
                    left: side,
                    right: side,
                    top: pad_v,
                    bottom: pad_v * 1.2,
                };
                view.layout.spacing = spacing;
            }
        }
        let bar = self.ui.view(cx, ids!(top_bar));
        if let Some(mut view) = bar.borrow_mut() {
            let edge = if tier.stacked() { 16.0 } else { side.min(40.0).max(20.0) };
            view.layout.padding.left = edge;
            view.layout.padding.right = edge - 4.0;
        }
        if tier_changed {
            let stacked = tier.stacked();
            self.ui
                .widget(cx, ids!(chapter_name))
                .set_visible(cx, !stacked);
            self.ui
                .widget(cx, ids!(byline))
                .set_visible(cx, tier != Tier::Phone);
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

    /// The deck takes arrow / clicker keys without a first click.
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

/// `#trade`, `#2` (1-based) → chapter index.
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

/// `390x844:4` → (Some((390, 844)), Some(3)); either half may be left out.
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
    let chapter = chapter_part.and_then(|c| chapter_from_hash(c));
    (size, chapter)
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
                // First layout can land after Startup on the web (window 0
                // gets its size once the canvas is measured).
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
        assert_eq!(chapter_from_hash("#trade"), Some(1));
        assert_eq!(chapter_from_hash("#1"), Some(0));
        assert_eq!(chapter_from_hash("#7"), Some(6));
        assert_eq!(chapter_from_hash("#8"), None);
        assert_eq!(chapter_from_hash("#0"), None);
        assert_eq!(chapter_from_hash(""), None);
        assert_eq!(chapter_from_hash("#nope"), None);
    }

    #[test]
    fn preview_spec_parses_size_and_chapter() {
        assert_eq!(parse_preview("390x844:4"), (Some((390.0, 844.0)), Some(3)));
        assert_eq!(parse_preview("1280x800"), (Some((1280.0, 800.0)), None));
        assert_eq!(parse_preview("free"), (None, Some(4)));
        assert_eq!(parse_preview("10x10:2"), (None, Some(1)));
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
        let body = Tier::Phone.type_scale(16.0);
        assert!(display < body);
        assert_eq!(Tier::Phone.type_scale(13.0), 1.0);
        assert_eq!(Tier::Desk.type_scale(54.0), 1.0);
    }
}

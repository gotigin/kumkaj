//! moklanjo.com — Moklanjo's site as a fullpage Makepad 2 app (wasm in the
//! browser; the same crate runs native for previews and as an offline
//! presenter in a courier office). Story: apps/moklanjo/docs/MOKLANJO_WEBSITE.md.
//! Copied from sites/kumkaj, then diverged (copy-then-diverge; no shared
//! site engine).
//!
//! Scroll / swipe up / tap / ↓ → PgDn Space flip chapters; ↑ ← PgUp go back;
//! Home / End jump. The beads on the tracking rail jump to a chapter.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:2` opens a 390×844 window on chapter 2 (1-based).
//! - `SITE_LINKS=local` sends the kumkaj.com link to the local wasm server.
//!
//! In the browser, `#weigh` or `#2` opens that chapter; a page served from
//! 127.0.0.1 / localhost links back to the local kumkaj.com (:8010).

pub use makepad_widgets;

use makepad_widgets::*;

mod weight;
use weight::{Parcel, DOMESTIC_DIVISOR, INTERNATIONAL_DIVISOR};

app_main!(App);

include!("ui.rs");

// ---- Owner switches (docs/WEBSITES.md §7 launch gates, §8 owner inputs) --

/// "A second counter and a rider phone stay joined over the office Wi‑Fi"
/// waits for the owner's real-hardware LAN pilot
/// (docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md §3; `offline-lan` claim).
const LAN_PILOT_PASSED: bool = false;

/// Download links go live only after the installers pass a clean-machine
/// install. Until then the button says so instead of linking.
const DOWNLOADS_LIVE: bool = false;
/// Where "Download free" goes once `DOWNLOADS_LIVE` is true.
const DOWNLOAD_URL: &str = "https://moklanjo.com/download";

/// TODO(owner): Moklanjo's WhatsApp Business number — digits only, with the
/// country code (no "+", no spaces). Agents must not invent one. Until it
/// is set, the button says the demo line is coming and shows the message.
const WHATSAPP_NUMBER: Option<&str> = None;
const WHATSAPP_MESSAGE: &str = "Moklanjo demo — city, parcels a day";

// ---- Chapters ---------------------------------------------------------

/// Chapter ids (URL hash) and the rail / top-bar names.
const CHAPTERS: [(&str, &str); 9] = [
    ("walkin", "Walk-in"),
    ("weigh", "Weigh it right"),
    ("price", "Price it"),
    ("labels", "Label it"),
    ("roads", "Hand over"),
    ("track", "Track and prove"),
    ("cod", "Money comes home"),
    ("offline", "Internet down"),
    ("start", "Start"),
];

const SLIDES: [&[LiveId]; 9] = [
    ids!(walkin),
    ids!(weigh),
    ids!(price),
    ids!(labels),
    ids!(roads),
    ids!(track),
    ids!(cod),
    ids!(offline),
    ids!(start),
];

/// Rail beads: [hit, ahead, passed, here, whole bead, name].
const BEADS: [[&[LiveId]; 6]; 9] = [
    [ids!(beads.b0.hit), ids!(beads.b0.ahead), ids!(beads.b0.passed), ids!(beads.b0.here), ids!(beads.b0), ids!(beads.b0.name)],
    [ids!(beads.b1.hit), ids!(beads.b1.ahead), ids!(beads.b1.passed), ids!(beads.b1.here), ids!(beads.b1), ids!(beads.b1.name)],
    [ids!(beads.b2.hit), ids!(beads.b2.ahead), ids!(beads.b2.passed), ids!(beads.b2.here), ids!(beads.b2), ids!(beads.b2.name)],
    [ids!(beads.b3.hit), ids!(beads.b3.ahead), ids!(beads.b3.passed), ids!(beads.b3.here), ids!(beads.b3), ids!(beads.b3.name)],
    [ids!(beads.b4.hit), ids!(beads.b4.ahead), ids!(beads.b4.passed), ids!(beads.b4.here), ids!(beads.b4), ids!(beads.b4.name)],
    [ids!(beads.b5.hit), ids!(beads.b5.ahead), ids!(beads.b5.passed), ids!(beads.b5.here), ids!(beads.b5), ids!(beads.b5.name)],
    [ids!(beads.b6.hit), ids!(beads.b6.ahead), ids!(beads.b6.passed), ids!(beads.b6.here), ids!(beads.b6), ids!(beads.b6.name)],
    [ids!(beads.b7.hit), ids!(beads.b7.ahead), ids!(beads.b7.passed), ids!(beads.b7.here), ids!(beads.b7), ids!(beads.b7.name)],
    [ids!(beads.b8.hit), ids!(beads.b8.ahead), ids!(beads.b8.passed), ids!(beads.b8.here), ids!(beads.b8), ids!(beads.b8.name)],
];

/// Rows that sit side by side only when the content column is at least
/// this wide (px), and stack otherwise — width-based, not tier-based: a
/// fixed prop beside a headline needs its own width plus a readable text
/// column whatever the device.
const ROWS: [(&[LiveId], f64); 11] = [
    (ids!(walkin.hero_row), 824.0),
    (ids!(walkin.hero_cta.buttons), 400.0),
    (ids!(weigh.weigh_row), 820.0),
    (ids!(price.price_row), 820.0),
    (ids!(labels.label_row), 808.0),
    (ids!(roads.roads_row), 640.0),
    (ids!(track.track_facts), 760.0),
    (ids!(cod.cod_row), 820.0),
    (ids!(offline.offline_row), 640.0),
    (ids!(start.close_cta.buttons), 400.0),
    (ids!(start.footer), 480.0),
];
/// Fixed-width props: (path, desk width, the min width of their row).
/// When the row stacks, the prop goes full width.
const PROPS: [(&[LiveId], f64, f64); 2] = [
    (ids!(walkin.loop_card), 340.0, 824.0),
    (ids!(labels.awb_label), 380.0, 808.0),
];

/// Detail a stacked (phone / narrow) layout drops to fit one screen.
const DETAIL_WHEN_STACKED: [&[LiveId]; 5] = [
    ids!(walkin.loop_card),
    ids!(track.track_line),
    ids!(weigh.weigh_note),
    ids!(price.price_facts),
    ids!(cod.cod_facts),
];
/// Extra detail only the smallest tier drops.
/// Shown instead of a dropped detail when stacked.
const ONLY_WHEN_STACKED: [&[LiveId]; 1] = [ids!(track.track_compact)];
const DETAIL_ON_PHONE: [&[LiveId]; 4] = [
    ids!(weigh.actual),
    ids!(weigh.toy_title),
    ids!(weigh.rule),
    ids!(labels.label_facts),
];
/// Detail a short, not-wide window (under 860 px tall and 1200 px wide,
/// e.g. a 1024 × 768 laptop) drops so the chapter's buttons stay on
/// screen. Wider short windows (1280 × 720, 1366 × 768) fit everything.
const DETAIL_WHEN_SHORT: [&[LiveId]; 1] = [
    ids!(weigh.weigh_note),
];
const SHORT_BELOW: f64 = 860.0;
const SHORT_NARROWER_THAN: f64 = 1200.0;

const MAX_CONTENT: f64 = 1140.0;
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
            Self::Phone => (0.56, 0.84, 0.94),
            Self::Narrow => (0.68, 0.9, 0.97),
            Self::Tablet => (0.82, 0.95, 1.0),
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

    /// (minimum side padding, vertical padding, chapter spacing)
    fn spacing(self) -> (f64, f64, f64) {
        match self {
            Self::Phone => (18.0, 14.0, 12.0),
            Self::Narrow => (26.0, 20.0, 14.0),
            Self::Tablet => (40.0, 26.0, 18.0),
            Self::Desk => (60.0, 30.0, 20.0),
            Self::Wide => (80.0, 44.0, 26.0),
        }
    }

    /// Rail bead width; names show only when there is room.
    fn bead(self) -> (f64, bool) {
        match self {
            Self::Phone => (24.0, false),
            Self::Narrow => (34.0, false),
            Self::Tablet => (64.0, true),
            Self::Desk => (78.0, true),
            Self::Wide => (86.0, true),
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
    parcel: Option<Parcel>,
    #[rust]
    focused_deck: bool,
}

impl App {
    fn slides(&self, cx: &mut Cx) -> SlidesViewRef {
        self.ui.slides_view(cx, ids!(slides))
    }

    fn parcel(&self) -> Parcel {
        self.parcel.unwrap_or(Parcel::EXAMPLE)
    }

    fn go_to(&mut self, cx: &mut Cx, chapter: usize) {
        let chapter = chapter.min(SLIDES.len() - 1);
        self.slides(cx).set_goal_slide(cx, chapter);
        self.paint_chrome(cx, chapter);
    }

    /// Rail: beads behind the parcel are passed (yellow), the current one is
    /// here (orange), the rest are ahead — like a docket's tracking line.
    fn paint_chrome(&mut self, cx: &mut Cx, chapter: usize) {
        self.chapter = chapter;
        self.ui.widget(cx, ids!(step_name)).set_text(
            cx,
            &format!("{} / {} · {}", chapter + 1, CHAPTERS.len(), CHAPTERS[chapter].1),
        );
        for (i, [_, ahead, passed, here, _, _]) in BEADS.iter().enumerate() {
            self.ui.view(cx, ahead).set_visible(cx, i > chapter);
            self.ui.view(cx, passed).set_visible(cx, i < chapter);
            self.ui.view(cx, here).set_visible(cx, i == chapter);
        }
        self.ui.redraw(cx);
    }

    fn paint_weight(&mut self, cx: &mut Cx) {
        let p = self.parcel();
        let set = |cx: &mut Cx, ui: &WidgetRef, path: &[LiveId], text: &str| {
            ui.widget(cx, path).set_text(cx, text);
        };
        set(cx, &self.ui, ids!(weigh.len.value), &format!("{} cm", p.length_cm));
        set(cx, &self.ui, ids!(weigh.bre.value), &format!("{} cm", p.breadth_cm));
        set(cx, &self.ui, ids!(weigh.hei.value), &format!("{} cm", p.height_cm));
        set(cx, &self.ui, ids!(weigh.act.value), &format!("{} kg", weight::fmt_kg_1(p.actual_g)));
        set(cx, &self.ui, ids!(weigh.charge), &format!("{} kg", weight::fmt_kg_1(p.chargeable_g())));
        set(
            cx,
            &self.ui,
            ids!(weigh.working),
            &format!(
                "{} × {} × {} ÷ {} = {} kg",
                p.length_cm,
                p.breadth_cm,
                p.height_cm,
                p.divisor,
                weight::fmt_centi_kg(p.volumetric_centi_kg())
            ),
        );
        set(
            cx,
            &self.ui,
            ids!(weigh.actual),
            &format!("On the scale: {} kg", weight::fmt_kg_1(p.actual_g)),
        );
        set(
            cx,
            &self.ui,
            ids!(weigh.winner),
            if p.volumetric_wins() {
                "The box's size sets the charge."
            } else {
                "The weight on the scale sets the charge."
            },
        );
        let international = p.divisor == INTERNATIONAL_DIVISOR;
        self.ui.view(cx, ids!(weigh.dom.on)).set_visible(cx, !international);
        self.ui.view(cx, ids!(weigh.dom.off)).set_visible(cx, international);
        self.ui.view(cx, ids!(weigh.intl.on)).set_visible(cx, international);
        self.ui.view(cx, ids!(weigh.intl.off)).set_visible(cx, !international);
        self.ui.redraw(cx);
    }

    fn handle_weight(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut p = self.parcel();
        let before = p;
        // Box sides step 5 cm (5–150); the scale steps 0.1 kg (0.1–50 kg).
        let sides: [(&[LiveId], &[LiveId], fn(&mut Parcel) -> &mut u64); 3] = [
            (ids!(weigh.len.minus), ids!(weigh.len.plus), |p| &mut p.length_cm),
            (ids!(weigh.bre.minus), ids!(weigh.bre.plus), |p| &mut p.breadth_cm),
            (ids!(weigh.hei.minus), ids!(weigh.hei.plus), |p| &mut p.height_cm),
        ];
        for (minus, plus, side) in sides {
            if self.ui.button(cx, minus).clicked(actions) {
                let v = side(&mut p);
                *v = v.saturating_sub(5).max(5);
            }
            if self.ui.button(cx, plus).clicked(actions) {
                let v = side(&mut p);
                *v = (*v + 5).min(150);
            }
        }
        if self.ui.button(cx, ids!(weigh.act.minus)).clicked(actions) {
            p.actual_g = p.actual_g.saturating_sub(100).max(100);
        }
        if self.ui.button(cx, ids!(weigh.act.plus)).clicked(actions) {
            p.actual_g = (p.actual_g + 100).min(50_000);
        }
        if self.ui.button(cx, ids!(weigh.dom.hit)).clicked(actions) {
            p.divisor = DOMESTIC_DIVISOR;
        }
        if self.ui.button(cx, ids!(weigh.intl.hit)).clicked(actions) {
            p.divisor = INTERNATIONAL_DIVISOR;
        }
        if p != before {
            self.parcel = Some(p);
            self.paint_weight(cx);
        }
    }

    fn set_cta_notes(&mut self, cx: &mut Cx, text: &str) {
        for path in [ids!(walkin.hero_cta.note), ids!(start.close_cta.note)] {
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
        self.ui
            .widget(cx, ids!(offline.offline_lan))
            .set_visible(cx, LAN_PILOT_PASSED);
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
        self.paint_details(cx, tier, size.y < SHORT_BELOW && size.x < SHORT_NARROWER_THAN);
        for path in SLIDES {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.layout.padding = Inset {
                    left: side,
                    right: side,
                    top: pad_v,
                    bottom: pad_v * 1.2,
                };
                view.layout.spacing = spacing;
            }
        }
        if tier_changed {
            let stacked = tier.stacked();
            for path in ONLY_WHEN_STACKED {
                self.ui.widget(cx, path).set_visible(cx, stacked);
            }
            let (bead_w, names) = tier.bead();
            for [_, _, _, _, bead, name] in BEADS {
                if let Some(mut view) = self.ui.view(cx, bead).borrow_mut() {
                    view.walk.width = Size::Fixed(bead_w);
                    view.walk.height = Size::Fixed(if names { 42.0 } else { 24.0 });
                }
                self.ui.widget(cx, name).set_visible(cx, names);
            }
            if let Some(mut view) = self.ui.view(cx, ids!(rail.rail_line)).borrow_mut() {
                view.walk.margin.left = bead_w / 2.0;
                view.walk.margin.right = bead_w / 2.0;
            }
            self.ui
                .widget(cx, ids!(tagline))
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

/// Percent-encode a WhatsApp prefill (RFC 3986 unreserved stay as is).
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
        self.paint_weight(cx);
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
        for (i, [hit, ..]) in BEADS.iter().enumerate() {
            if self.ui.button(cx, hit).clicked(actions) {
                self.go_to(cx, i);
            }
        }
        self.handle_weight(cx, actions);
        let download = self.ui.button(cx, ids!(walkin.hero_cta.download)).clicked(actions)
            || self.ui.button(cx, ids!(start.close_cta.download)).clicked(actions);
        if download {
            self.download(cx);
        }
        let whatsapp = self.ui.button(cx, ids!(walkin.hero_cta.whatsapp)).clicked(actions)
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
        assert_eq!(chapter_from_hash("#weigh"), Some(1));
        assert_eq!(chapter_from_hash("#9"), Some(8));
        assert_eq!(chapter_from_hash("#10"), None);
        assert_eq!(chapter_from_hash("#"), None);
    }

    #[test]
    fn whatsapp_prefill_is_percent_encoded() {
        assert_eq!(url_encode("Moklanjo demo"), "Moklanjo%20demo");
        // The em dash is three UTF-8 bytes.
        assert_eq!(url_encode("a — b"), "a%20%E2%80%94%20b");
    }

    #[test]
    fn owner_switches_ship_closed() {
        // Agents never invent a number or open downloads / LAN claims.
        assert!(WHATSAPP_NUMBER.is_none());
        assert!(!DOWNLOADS_LIVE);
        assert!(!LAN_PILOT_PASSED);
    }

    #[test]
    fn phone_layout_hides_rail_names() {
        assert_eq!(Tier::Phone.bead(), (24.0, false));
        assert!(Tier::Desk.bead().1);
    }
}

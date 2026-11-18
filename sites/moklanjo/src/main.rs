//! moklanjo.com — Moklanjo's site as a fullpage Makepad 2 app (wasm in the
//! browser; the same crate runs native for previews and as an offline
//! presenter in a courier office). Story: apps/moklanjo/docs/MOKLANJO_WEBSITE.md.
//! Copied from sites/kumkaj, then diverged (copy-then-diverge; no shared
//! site engine).
//!
//! One scrolling page (was a nine-slide deck): sticky top bar with the
//! tracking-bead rail; chapters walk-in through start, then a footer.
//!
//! Native preview helpers (ignored in the browser):
//! - `SITE_PREVIEW=390x844:weigh` opens a 390×844 window scrolled to a
//!   section (chapter id, 1-based number, or nothing for the top).
//! - `SITE_LINKS=local` sends the kumkaj.com link to the local wasm server.
//!
//! In the browser, `#weigh` (or `#2`) scrolls to that section; a page served
//! from 127.0.0.1 / localhost links back to the local kumkaj.com (:8010).

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

/// Same-host legal pages (deploy-firebase.sh stages them next to the wasm).
const PRIVACY_PATH: &str = "/privacy";
const TERMS_PATH: &str = "/terms/";
const PUBLIC_HOST: &str = "https://moklanjo.com";

// ---- Chapters ---------------------------------------------------------

/// Chapter ids (URL hash), widget path, and the rail bead name.
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

const SECTIONS: [(&str, &[LiveId]); 9] = [
    ("walkin", ids!(walkin)),
    ("weigh", ids!(weigh)),
    ("price", ids!(price)),
    ("labels", ids!(labels)),
    ("roads", ids!(roads)),
    ("track", ids!(track)),
    ("cod", ids!(cod)),
    ("offline", ids!(offline)),
    ("start", ids!(start)),
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
    (ids!(footer.footer_row), 480.0),
];
/// Fixed-width props: (path, desk width, the min width of their row).
/// When the row stacks, the prop goes full width.
const PROPS: [(&[LiveId], f64, f64); 2] = [
    (ids!(walkin.loop_card), 340.0, 824.0),
    (ids!(labels.awb_label), 380.0, 808.0),
];

/// App layout floor: design for ≥375 CSS px.
const MIN_WIDTH: f64 = 375.0;

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

/// Section to scroll to after layout (the Script derive wants plain types).
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
    parcel: Option<Parcel>,
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

    fn parcel(&self) -> Parcel {
        self.parcel.unwrap_or(Parcel::EXAMPLE)
    }

    /// Scroll the page so `section` sits under the top bar.
    fn scroll_to(&mut self, cx: &mut Cx, section: &str) -> f64 {
        let Some(&(id, path)) = SECTIONS.iter().find(|(sid, _)| *sid == section) else {
            return 0.0;
        };
        let page = self.page(cx);
        let page_rect = page.area().rect(cx);
        let target = self.ui.widget(cx, path).area().rect(cx);
        let top = self.ui.widget(cx, ids!(walkin)).area().rect(cx);
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

    /// Which chapter owns the viewport (just under the top bar).
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

    /// Rail: beads behind the parcel are passed (yellow), the current one is
    /// here (orange), the rest are ahead — like a docket's tracking line.
    fn paint_chrome(&mut self, cx: &mut Cx, chapter: usize) {
        self.chapter = chapter;
        self.ui.label(cx, ids!(step_name)).set_text(
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
                view.layout.flow = if content_w >= min_w { Flow::right() } else { Flow::Down };
            }
        }
        for (path, desk_w, row_min_w) in PROPS {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.walk.width = if content_w >= row_min_w { Size::Fixed(desk_w) } else { Size::fill() };
            }
        }
        for (_, path) in SECTIONS {
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
        if let Some(mut view) = self.ui.view(cx, ids!(footer)).borrow_mut() {
            view.layout.padding = Inset {
                left: side,
                right: side,
                top: pad_v * 0.8,
                bottom: pad_v * 1.1,
            };
        }
        if tier_changed {
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
                .label(cx, ids!(tagline))
                .set_visible(cx, tier != Tier::Phone);
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

/// `#weigh`, `#2` (1-based chapter number) → section id.
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

/// `390x844:weigh` → (Some((390, 844)), Some("weigh")); either half may be omitted.
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
        self.paint_chrome(cx, 0);
        self.page(cx).set_key_focus(cx);
        self.pending_section = self.section_from_page(cx);
        self.pending_frames = 120;
        self.pending_stable = 0;
        #[cfg(not(target_arch = "wasm32"))]
        self.apply_native_preview(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
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
        if self.ui.button(cx, ids!(to_kumkaj)).clicked(actions) {
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
        assert_eq!(section_from_hash("#weigh"), Some("weigh"));
        assert_eq!(section_from_hash("#9"), Some("start"));
        assert_eq!(section_from_hash("#10"), None);
        assert_eq!(section_from_hash("#"), None);
    }

    #[test]
    fn preview_parses_section_id() {
        let (_, section) = parse_preview("390x844:weigh");
        assert_eq!(section, Some("weigh"));
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

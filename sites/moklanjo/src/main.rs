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
const CANCELLATION_PATH: &str = "/cancellation";
const SHOP_PATH: &str = "/shop";
const ACCOUNT_PATH: &str = "/account";
const PUBLIC_HOST: &str = "https://moklanjo.com";

// ---- Chapters ---------------------------------------------------------

/// Chapter ids (URL hash), widget path, and the rail bead name.
/// Top-rail bead captions (equal Fill columns — no per-bead fixed width).
const BEAD_LABELS: [&str; 9] = [
    "Walk-in",
    "Weigh",
    "Price",
    "Label",
    "Hand over",
    "Track",
    "COD",
    "Offline",
    "Start",
];
/// Tighter captions when the content column is narrow but labels stay visible.
const BEAD_LABELS_SHORT: [&str; 9] = [
    "Walk-in",
    "Weigh",
    "Price",
    "Label",
    "Hand",
    "Track",
    "COD",
    "Offln",
    "Start",
];

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

/// Rail beads: [hit, ahead, passed, here, ring, parcel_chip, whole bead, name].
const BEADS: [[&[LiveId]; 8]; 9] = [
    [
        ids!(beads.b0.hit),
        ids!(beads.b0.col.dot_row.ahead),
        ids!(beads.b0.col.dot_row.passed),
        ids!(beads.b0.col.dot_row.here),
        ids!(beads.b0.col.dot_row.ring),
        ids!(beads.b0.col.dot_row.parcel_chip),
        ids!(beads.b0),
        ids!(beads.b0.col.name),
    ],
    [
        ids!(beads.b1.hit),
        ids!(beads.b1.col.dot_row.ahead),
        ids!(beads.b1.col.dot_row.passed),
        ids!(beads.b1.col.dot_row.here),
        ids!(beads.b1.col.dot_row.ring),
        ids!(beads.b1.col.dot_row.parcel_chip),
        ids!(beads.b1),
        ids!(beads.b1.col.name),
    ],
    [
        ids!(beads.b2.hit),
        ids!(beads.b2.col.dot_row.ahead),
        ids!(beads.b2.col.dot_row.passed),
        ids!(beads.b2.col.dot_row.here),
        ids!(beads.b2.col.dot_row.ring),
        ids!(beads.b2.col.dot_row.parcel_chip),
        ids!(beads.b2),
        ids!(beads.b2.col.name),
    ],
    [
        ids!(beads.b3.hit),
        ids!(beads.b3.col.dot_row.ahead),
        ids!(beads.b3.col.dot_row.passed),
        ids!(beads.b3.col.dot_row.here),
        ids!(beads.b3.col.dot_row.ring),
        ids!(beads.b3.col.dot_row.parcel_chip),
        ids!(beads.b3),
        ids!(beads.b3.col.name),
    ],
    [
        ids!(beads.b4.hit),
        ids!(beads.b4.col.dot_row.ahead),
        ids!(beads.b4.col.dot_row.passed),
        ids!(beads.b4.col.dot_row.here),
        ids!(beads.b4.col.dot_row.ring),
        ids!(beads.b4.col.dot_row.parcel_chip),
        ids!(beads.b4),
        ids!(beads.b4.col.name),
    ],
    [
        ids!(beads.b5.hit),
        ids!(beads.b5.col.dot_row.ahead),
        ids!(beads.b5.col.dot_row.passed),
        ids!(beads.b5.col.dot_row.here),
        ids!(beads.b5.col.dot_row.ring),
        ids!(beads.b5.col.dot_row.parcel_chip),
        ids!(beads.b5),
        ids!(beads.b5.col.name),
    ],
    [
        ids!(beads.b6.hit),
        ids!(beads.b6.col.dot_row.ahead),
        ids!(beads.b6.col.dot_row.passed),
        ids!(beads.b6.col.dot_row.here),
        ids!(beads.b6.col.dot_row.ring),
        ids!(beads.b6.col.dot_row.parcel_chip),
        ids!(beads.b6),
        ids!(beads.b6.col.name),
    ],
    [
        ids!(beads.b7.hit),
        ids!(beads.b7.col.dot_row.ahead),
        ids!(beads.b7.col.dot_row.passed),
        ids!(beads.b7.col.dot_row.here),
        ids!(beads.b7.col.dot_row.ring),
        ids!(beads.b7.col.dot_row.parcel_chip),
        ids!(beads.b7),
        ids!(beads.b7.col.name),
    ],
    [
        ids!(beads.b8.hit),
        ids!(beads.b8.col.dot_row.ahead),
        ids!(beads.b8.col.dot_row.passed),
        ids!(beads.b8.col.dot_row.here),
        ids!(beads.b8.col.dot_row.ring),
        ids!(beads.b8.col.dot_row.parcel_chip),
        ids!(beads.b8),
        ids!(beads.b8.col.name),
    ],
];

/// Rows that sit side by side only when the content column is at least
/// this wide (px), and stack otherwise — width-based, not tier-based: a
/// fixed prop beside a headline needs its own width plus a readable text
/// column whatever the device.
const ROWS: [(&[LiveId], f64); 13] = [
    (ids!(walkin.hero_row), 844.0),
    // "WhatsApp for a demo" + "Download free" need ~420px side-by-side;
    // WebKit at 400 CSS px still clipped the secondary label at 400.0.
    (ids!(walkin.hero_cta.buttons), 440.0),
    (ids!(weigh.weigh_row), 820.0),
    // "Try a box" beside the two chips needs the whole card.
    (ids!(weigh.toy_head), 400.0),
    (ids!(price.price_row), 820.0),
    (ids!(labels.label_row), 808.0),
    (ids!(roads.roads_row), 640.0),
    (ids!(track.track_facts), 760.0),
    (ids!(cod.cod_row), 820.0),
    (ids!(offline.offline_row), 640.0),
    (ids!(start.close_cta.buttons), 440.0),
    (ids!(footer.footer_row), 480.0),
    // Privacy · Terms · kumkaj.com overflow a phone content column.
    (ids!(footer.footer_links), 420.0),
];
/// The weigh toy's four steppers: the name sits above − value + when the
/// card has no room for a label column (under 340 px of content).
const STEPPERS: [&[LiveId]; 4] = [
    ids!(weigh.len),
    ids!(weigh.bre),
    ids!(weigh.hei),
    ids!(weigh.act),
];
const STEPPERS_ROW_FROM: f64 = 340.0;
/// Number columns of the rate card and the rider-cash table:
/// (path, desk width, phone width). Under 420 px of content they take the
/// phone width so the first column keeps whole words ("Within state ·
/// Express" broke inside "Express" at 360). The rider-cash amounts are
/// longer ("₹ 4,250"), so that table keeps 88.
const NUM_COLS: [(&[LiveId], f64, f64); 14] = [
    (ids!(price.rc_h1), 96.0, 72.0),
    (ids!(price.rc_h2), 100.0, 72.0),
    (ids!(price.rc_a1), 96.0, 72.0),
    (ids!(price.rc_a2), 100.0, 72.0),
    (ids!(price.rc_b1), 96.0, 72.0),
    (ids!(price.rc_b2), 100.0, 72.0),
    (ids!(price.rc_c1), 96.0, 72.0),
    (ids!(price.rc_c2), 100.0, 72.0),
    (ids!(cod.cod_h1), 96.0, 88.0),
    (ids!(cod.cod_h2), 96.0, 88.0),
    (ids!(cod.cod_a1), 96.0, 88.0),
    (ids!(cod.cod_a2), 96.0, 88.0),
    (ids!(cod.cod_b1), 96.0, 88.0),
    (ids!(cod.cod_b2), 96.0, 88.0),
];
const NUM_COLS_FULL_FROM: f64 = 420.0;
/// Fixed-width props: (path, desk width, the min width of their row).
/// When the row stacks, the prop goes full width.
const PROPS: [(&[LiveId], f64, f64); 2] = [
    (ids!(walkin.loop_card), 360.0, 844.0),
    (ids!(labels.awb_label), 380.0, 808.0),
];

/// App layout floor: design for ≥320 CSS px (360 is the common Android phone width).
const MIN_WIDTH: f64 = 320.0;

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

    /// (rail bead row height, show caption under each dot).
    fn bead(self) -> (f64, bool) {
        match self {
            Self::Phone => (30.0, false),
            Self::Narrow => (30.0, false),
            Self::Tablet => (58.0, true),
            Self::Desk => (58.0, true),
            Self::Wide => (60.0, true),
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
    /// Optional ease when a rail bead jumps to a chapter (not used for wheel).
    #[rust]
    scroll_anim_from: f64,
    #[rust]
    scroll_anim_to: Option<f64>,
    /// NextFrame time when a bead jump started (0 = take next tick).
    #[rust]
    scroll_anim_t0: f64,
    #[rust]
    scroll_anim_frame: NextFrame,
    /// Scroll Y last used for rail / chapter chrome (free-scroll + post-layout).
    #[rust]
    last_chrome_scroll_y: f64,
    /// NextFrame token for chrome sync after layout catches up to scroll_pos.
    #[rust]
    chrome_sync_frame: NextFrame,
    /// True when a chrome sync was requested (avoid mid-Draw walk.width mutation).
    #[rust]
    chrome_sync_pending: bool,
}

impl App {
    fn page(&self, cx: &mut Cx) -> ViewRef {
        self.ui.view(cx, ids!(page))
    }

    fn parcel(&self) -> Parcel {
        self.parcel.unwrap_or(Parcel::EXAMPLE)
    }

    /// Viewport mark used by scroll-spy (just under the sticky top bar).
    fn spy_mark(&self, cx: &Cx) -> f64 {
        let page = self.ui.view(cx, ids!(page));
        let bar = self.ui.view(cx, ids!(top_bar));
        if bar.area().is_valid(cx) {
            bar.area().rect(cx).pos.y + bar.area().rect(cx).size.y + 12.0
        } else if page.area().is_valid(cx) {
            page.area().rect(cx).pos.y + 72.0
        } else {
            72.0
        }
    }

    /// True when the user has scrolled to the bottom (short last chapter never
    /// crosses the spy mark). Must **not** fire before scroll metrics exist —
    /// `total ≈ visible` on first frames was wrongly treated as “end” and filled
    /// the whole orange rail while still on Walk-in.
    fn at_page_end(&self, cx: &Cx) -> bool {
        let page = self.ui.view(cx, ids!(page));
        if !page.area().is_valid(cx) {
            return false;
        }
        let total = page.get_scroll_view_total().y;
        let visible = page.get_scroll_view_visible().y;
        // Layout / scroll view not ready yet.
        if total < 32.0 || visible < 32.0 {
            return false;
        }
        let max = (total - visible).max(0.0);
        // Page still fits in one view — not "scrolled to end"; use geometry only.
        if max <= 4.0 {
            return false;
        }
        let y = page.get_scroll_pos().y;
        y >= max - 4.0
    }

    /// Section tops in document Y (same space as `spy_mark`).
    fn section_tops(&self, cx: &Cx) -> [f64; 9] {
        let mut tops = [0.0; 9];
        for (i, (_, path)) in SECTIONS.iter().enumerate() {
            let w = self.ui.widget(cx, path);
            tops[i] = if w.area().is_valid(cx) {
                w.area().rect(cx).pos.y
            } else {
                f64::NAN
            };
        }
        // Fill holes from the previous valid top so progress never jumps to 1.0
        // because every chapter collapsed to the same Y before layout.
        let mut last = tops[0];
        if !last.is_finite() {
            last = 0.0;
            tops[0] = 0.0;
        }
        for i in 1..tops.len() {
            if tops[i].is_finite() {
                last = tops[i];
            } else {
                tops[i] = last;
            }
        }
        tops
    }

    /// Continuous journey progress 0..=1 from scroll position (not discrete chapters).
    fn journey_progress(&self, cx: &Cx) -> f64 {
        let at_end = self.at_page_end(cx);
        let mark = self.spy_mark(cx);
        let tops = self.section_tops(cx);
        journey_progress_from(&tops, mark, at_end)
    }

    /// Schedule chrome sync on the next frame (after layout reflects scroll_pos).
    fn request_chrome_sync(&mut self, cx: &mut Cx) {
        self.chrome_sync_pending = true;
        self.chrome_sync_frame = cx.new_next_frame();
    }

    fn run_chrome_sync(&mut self, cx: &mut Cx) {
        if !self.chrome_sync_pending {
            return;
        }
        if self.scroll_animating() {
            // Keep pending; try again after the page-step settles.
            self.chrome_sync_frame = cx.new_next_frame();
            return;
        }
        self.chrome_sync_pending = false;
        self.last_chrome_scroll_y = self.page(cx).get_scroll_pos().y;
        self.paint_scroll_chrome(cx);
        self.ui.view(cx, ids!(track_row)).redraw(cx);
        self.ui.view(cx, ids!(step_pill)).redraw(cx);
    }

    fn track_width(&self, cx: &Cx) -> f64 {
        // Match ui.rs rail_track margins (left/right 44) inside the padded rail.
        const RAIL_SIDE: f64 = 44.0;
        let row = self.ui.view(cx, ids!(track_row));
        if row.area().is_valid(cx) {
            return (row.area().rect(cx).size.x - 2.0 * RAIL_SIDE).max(80.0);
        }
        let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
        if size.x < 1.0 {
            return 800.0;
        }
        let layout_w = size.x.max(MIN_WIDTH);
        let tier = Tier::from_size(layout_w, size.y);
        let (min_side, _, _) = tier.spacing();
        let side = min_side.max((layout_w - tier.max_content()) / 2.0);
        (layout_w - 2.0 * side - 2.0 * RAIL_SIDE - 40.0).max(80.0)
    }

    /// Snap the orange progress fill to scroll progress. No easing: a
    /// NextFrame width animation redraw loop smeared text on wasm.
    fn snap_rail_fill(&mut self, cx: &mut Cx) {
        const RAIL_INSET: f64 = 44.0;
        let progress = self.journey_progress(cx);
        // Tiny stub at 0 so the first bead still has a glow tick; never a long
        // bar while progress is still at the start of the story.
        let width = if progress <= 0.001 {
            24.0
        } else {
            (self.track_width(cx) * progress).max(24.0)
        };
        if let Some(mut view) = self.ui.view(cx, ids!(track_row.rail_prog)).borrow_mut() {
            let cur = match view.walk.width {
                Size::Fixed(w) => w,
                _ => -1.0,
            };
            if (cur - width).abs() <= 0.5 && (view.walk.margin.left - RAIL_INSET).abs() <= 0.5 {
                return;
            }
            view.walk.width = Size::Fixed(width);
            view.walk.margin.left = RAIL_INSET;
        }
    }

    fn max_scroll_y(&self, cx: &mut Cx) -> f64 {
        let page = self.page(cx);
        let total = page.get_scroll_view_total().y;
        let visible = page.get_scroll_view_visible().y;
        (total - visible).max(0.0)
    }

    fn viewport_step(&self, cx: &mut Cx) -> f64 {
        self.page(cx).get_scroll_view_visible().y.max(240.0) * 0.92
    }

    fn scroll_animating(&self) -> bool {
        self.scroll_anim_to.is_some()
    }

    fn scroll_to_y_now(&mut self, cx: &mut Cx, y: f64) {
        let y = y.clamp(0.0, self.max_scroll_y(cx));
        self.scroll_anim_to = None;
        self.scroll_anim_t0 = 0.0;
        self.page(cx).set_scroll_pos(cx, dvec2(0.0, y));
        self.paint_scroll_chrome(cx);
        // Layout lags scroll_pos by a frame — re-snap rail/chapter with fresh tops.
        self.request_chrome_sync(cx);
        self.ui.redraw(cx);
    }

    /// Bead / deep-link jump only — wheel and trackpad free-scroll the ScrollYView.
    fn animate_scroll_to_y(&mut self, cx: &mut Cx, y: f64) {
        let y = y.clamp(0.0, self.max_scroll_y(cx));
        let cur = self.page(cx).get_scroll_pos().y;
        if (y - cur).abs() < 1.0 {
            self.scroll_to_y_now(cx, y);
            return;
        }
        self.scroll_anim_from = cur;
        self.scroll_anim_to = Some(y);
        self.scroll_anim_t0 = 0.0;
        self.scroll_anim_frame = cx.new_next_frame();
        self.ui.redraw(cx);
    }

    fn tick_scroll_anim(&mut self, cx: &mut Cx, time: f64) {
        let Some(target) = self.scroll_anim_to else {
            return;
        };
        if self.scroll_anim_t0 <= 0.0 {
            self.scroll_anim_t0 = time;
        }
        const DUR: f64 = 0.4;
        let t = ((time - self.scroll_anim_t0) / DUR).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - t).powi(3);
        let y = self.scroll_anim_from + (target - self.scroll_anim_from) * ease;
        let page = self.page(cx);
        page.set_scroll_pos(cx, dvec2(0.0, y));
        // Mid-flight: orange fill only — no bead label rewrites (wasm smear).
        self.snap_rail_fill(cx);
        if t >= 1.0 {
            page.set_scroll_pos(cx, dvec2(0.0, target));
            self.scroll_anim_to = None;
            self.scroll_anim_t0 = 0.0;
            self.paint_scroll_chrome(cx);
            self.request_chrome_sync(cx);
            self.ui.redraw(cx);
            return;
        }
        self.scroll_anim_frame = cx.new_next_frame();
        self.ui.view(cx, ids!(page)).redraw(cx);
        self.ui.view(cx, ids!(track_row)).redraw(cx);
        self.ui.view(cx, ids!(step_pill)).redraw(cx);
    }

    fn scroll_by(&mut self, cx: &mut Cx, delta: f64) {
        let cur = self.page(cx).get_scroll_pos().y;
        self.scroll_to_y_now(cx, cur + delta);
    }

    fn scroll_to_section(&mut self, cx: &mut Cx, section: &str, animate: bool) -> f64 {
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
        if animate {
            self.animate_scroll_to_y(cx, y);
        } else {
            self.scroll_to_y_now(cx, y);
        }
        y
    }

    fn go_to(&mut self, cx: &mut Cx, chapter: usize) {
        let chapter = chapter.min(CHAPTERS.len() - 1);
        let _ = self.scroll_to_section(cx, CHAPTERS[chapter].0, true);
    }

    /// Which chapter owns the viewport (just under the top bar).
    fn active_chapter(&self, cx: &Cx) -> usize {
        let n = SECTIONS.len();
        let at_end = self.at_page_end(cx);
        let mark = self.spy_mark(cx);
        let tops = self.section_tops(cx);
        // Start + footer are shorter than the viewport, so Start's top often
        // never crosses the sticky spy mark. Promote once Start reaches the
        // mid-viewport, or when scroll is already at the end.
        let page = self.ui.view(cx, ids!(page));
        let last_promoted = if page.area().is_valid(cx) {
            let pr = page.area().rect(cx);
            tops[n - 1] <= pr.pos.y + pr.size.y * 0.55
        } else {
            false
        };
        active_chapter_from(&tops, mark, at_end, last_promoted)
    }

    fn paint_scroll_chrome(&mut self, cx: &mut Cx) {
        let active = self.active_chapter(cx);
        if active != self.chapter {
            self.paint_chrome(cx, active);
        } else {
            // Same chapter: still advance the orange fill with scroll.
            self.snap_rail_fill(cx);
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
        let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
        let short_labels = if size.x >= 1.0 {
            let layout_w = size.x.max(MIN_WIDTH);
            let tier = Tier::from_size(layout_w, size.y);
            let (min_side, _, _) = tier.spacing();
            let side = min_side.max((layout_w - tier.max_content()) / 2.0);
            let content_w = layout_w - 2.0 * side;
            content_w < 820.0 && tier.bead().1
        } else {
            false
        };
        // Keep label font size constant — resizing glyphs reflowed the rail and jumped.
        const LABEL_SIZE: f32 = 12.5;
        for (i, [_, ahead, passed, here, ring, parcel_chip, _, name]) in BEADS.iter().enumerate() {
            self.ui.view(cx, ahead).set_visible(cx, i > chapter);
            self.ui.view(cx, passed).set_visible(cx, i < chapter);
            self.ui.view(cx, here).set_visible(cx, i == chapter);
            self.ui.view(cx, ring).set_visible(cx, i == chapter);
            self.ui.view(cx, parcel_chip).set_visible(cx, i == chapter);
            let caption = if short_labels {
                BEAD_LABELS_SHORT[i]
            } else {
                BEAD_LABELS[i]
            };
            self.ui.label(cx, name).set_text(cx, caption);
            if let Some(mut label) = self.ui.label(cx, name).borrow_mut() {
                let hex = if i < chapter {
                    0xF7B043FFu32
                } else if i == chapter {
                    0xFFFFFFFFu32
                } else {
                    0x6F8AC4FFu32
                };
                label.draw_text.color = Vec4f::from_u32(hex);
                label.draw_text.text_style.font_size = LABEL_SIZE;
            }
        }
        self.snap_rail_fill(cx);
        self.ui.redraw(cx);
    }

    fn handle_scroll_keys(&mut self, cx: &mut Cx, ke: &KeyEvent) {
        if ke.modifiers.control || ke.modifiers.logo || ke.modifiers.alt {
            return;
        }
        // Shift+arrows stay available for text selection; plain arrows nudge.
        if ke.modifiers.shift
            && matches!(ke.key_code, KeyCode::ArrowUp | KeyCode::ArrowDown)
        {
            return;
        }
        match ke.key_code {
            KeyCode::ArrowDown => self.scroll_by(cx, 64.0),
            KeyCode::ArrowUp => self.scroll_by(cx, -64.0),
            KeyCode::PageDown | KeyCode::Space if !ke.modifiers.shift => {
                let step = self.viewport_step(cx);
                self.scroll_by(cx, step);
            }
            KeyCode::PageUp => {
                let step = self.viewport_step(cx);
                self.scroll_by(cx, -step);
            }
            KeyCode::Home => self.scroll_to_y_now(cx, 0.0),
            KeyCode::End => {
                let end = self.max_scroll_y(cx);
                self.scroll_to_y_now(cx, end);
            }
            _ => {}
        }
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
        let stepper_row = content_w >= STEPPERS_ROW_FROM;
        for path in STEPPERS {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.layout.flow = if stepper_row { Flow::right() } else { Flow::Down };
                view.layout.spacing = if stepper_row { 10.0 } else { 6.0 };
            }
        }
        for (path, desk_w, phone_w) in NUM_COLS {
            if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                view.walk.width =
                    Size::Fixed(if content_w >= NUM_COLS_FULL_FROM { desk_w } else { phone_w });
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
        let track_diagram = content_w >= 520.0;
        self.ui
            .view(cx, ids!(track.timeline.track_line))
            .set_visible(cx, track_diagram);
        self.ui
            .widget(cx, ids!(track.timeline.track_compact))
            .set_visible(cx, !track_diagram);
        if tier_changed {
            let (row_h, names) = tier.bead();
            for [_, _, _, _, _, _, bead, name] in BEADS {
                if let Some(mut view) = self.ui.view(cx, bead).borrow_mut() {
                    view.walk.width = Size::fill();
                    view.walk.height = Size::Fixed(if names { row_h } else { 30.0 });
                }
                self.ui.widget(cx, name).set_visible(cx, names);
            }
            const RAIL_INSET: f64 = 44.0;
            for path in [ids!(track_row.rail_track), ids!(track_row.rail_prog)] {
                if let Some(mut view) = self.ui.view(cx, path).borrow_mut() {
                    view.walk.margin.left = RAIL_INSET;
                    if path == ids!(track_row.rail_track) {
                        view.walk.margin.right = RAIL_INSET;
                    }
                }
            }
            self.ui
                .label(cx, ids!(rail_hint))
                .set_visible(cx, tier != Tier::Phone && tier != Tier::Narrow);
            self.ui
                .label(cx, ids!(tagline))
                .set_visible(cx, !matches!(tier, Tier::Phone | Tier::Narrow));
            self.ui
                .widget(cx, ids!(step_pill))
                .set_visible(cx, tier != Tier::Phone);
            let page = self.ui.widget(cx, ids!(page));
            if self.desk_font.is_empty() {
                collect_label_sizes(&page, &mut self.desk_font);
            }
            let mut next = 0;
            scale_labels(&page, tier, &self.desk_font, &mut next);
            self.paint_chrome(cx, self.chapter);
        }
        // First layout (and every reflow) — re-measure section tops so the rail
        // is not stuck full from a pre-layout snap.
        self.request_chrome_sync(cx);
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

/// Continuous rail fill 0..=1 from section tops + spy mark.
/// `scroll_at_end` forces a full rail — short last chapters never cross the mark.
fn journey_progress_from(tops: &[f64], mark: f64, scroll_at_end: bool) -> f64 {
    let n = tops.len();
    if n <= 1 {
        return 0.0;
    }
    let first = tops[0];
    let last = tops[n - 1];
    let laid_out = first.is_finite() && last.is_finite() && (last - first).abs() >= 80.0;
    // Chapters not laid out yet — stay at the start. Do not honor scroll_at_end
    // either: that flag was the bug that filled the rail on first paint.
    if !laid_out {
        return 0.0;
    }
    if scroll_at_end {
        return 1.0;
    }
    let mut idx = 0usize;
    for (i, top) in tops.iter().enumerate() {
        if *top <= mark {
            idx = i;
        }
    }
    if idx >= n - 1 {
        return 1.0;
    }
    let a = tops[idx];
    let b = tops[idx + 1];
    let span = (b - a).max(1.0);
    let local = ((mark - a) / span).clamp(0.0, 1.0);
    (idx as f64 + local) / (n as f64 - 1.0)
}

/// Active chapter index from the same geometry as the rail fill.
fn active_chapter_from(
    tops: &[f64],
    mark: f64,
    scroll_at_end: bool,
    last_promoted: bool,
) -> usize {
    let n = tops.len();
    if n == 0 {
        return 0;
    }
    if scroll_at_end || last_promoted {
        return n - 1;
    }
    let mut idx = 0usize;
    for (i, top) in tops.iter().enumerate() {
        if *top <= mark {
            idx = i;
        }
    }
    idx
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
        if self.ui.button(cx, ids!(to_shop)).clicked(actions) {
            let url = self.site_url(cx, SHOP_PATH);
            open_link(cx, &url);
        }
        if self.ui.button(cx, ids!(to_account)).clicked(actions) {
            let url = self.site_url(cx, ACCOUNT_PATH);
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
            Event::KeyDown(ke) => self.handle_scroll_keys(cx, ke),
            // Free wheel / trackpad: ScrollYView owns the motion; keep the rail synced.
            Event::Scroll(_) => {
                if !self.scroll_animating() {
                    self.request_chrome_sync(cx);
                }
            }
            Event::NextFrame(ne) => {
                if ne.set.contains(&self.scroll_anim_frame) {
                    self.tick_scroll_anim(cx, ne.time);
                }
                if ne.set.contains(&self.chrome_sync_frame) {
                    self.run_chrome_sync(cx);
                }
                if self.laid_out.is_none() {
                    self.apply_responsive(cx);
                }
                if let Some(section) = self.pending_section {
                    let page_h = self.page(cx).area().rect(cx).size.y;
                    if page_h > 0.0 {
                        let y = self.scroll_to_section(cx, section, false);
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
            Event::Draw(_) => {
                if self.laid_out.is_none() {
                    self.apply_responsive(cx);
                }
                // Finger free-scroll + post-layout: do not mutate walk.width mid-Draw
                // (wasm text smear). Detect scroll change and sync on NextFrame.
                if !self.scroll_animating() {
                    let y = self.page(cx).get_scroll_pos().y;
                    if (y - self.last_chrome_scroll_y).abs() > 0.25 {
                        self.request_chrome_sync(cx);
                    }
                }
                if let Some(section) = self.pending_section {
                    let page_h = self.page(cx).area().rect(cx).size.y;
                    if page_h > 0.0 {
                        let y = self.scroll_to_section(cx, section, false);
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
        assert_eq!(Tier::Phone.bead(), (30.0, false));
        assert!(Tier::Desk.bead().1);
    }

    #[test]
    fn rail_fill_tracks_mark_between_chapters() {
        // Nine chapter tops spaced 1000 px; spy mark halfway through chapter 0 → Weigh.
        let tops: [f64; 9] = std::array::from_fn(|i| i as f64 * 1000.0);
        let mid_walkin = 500.0;
        let p = journey_progress_from(&tops, mid_walkin, false);
        // (0 + 0.5) / 8 = 0.0625 — fill has left the first bead toward Weigh.
        assert!((p - 0.0625).abs() < 1e-9, "got {p}");
        assert_eq!(active_chapter_from(&tops, mid_walkin, false, false), 0);

        let into_weigh = 1000.0;
        let p2 = journey_progress_from(&tops, into_weigh, false);
        assert!((p2 - 0.125).abs() < 1e-9, "got {p2}");
        assert_eq!(active_chapter_from(&tops, into_weigh, false, false), 1);
    }

    #[test]
    fn rail_stays_empty_when_chapters_not_laid_out_yet() {
        // Before layout, tops often collapse to the same Y — must not fill the rail.
        let stacked = [100.0; 9];
        assert_eq!(journey_progress_from(&stacked, 120.0, false), 0.0);
        // And must not treat a false "at end" as full when geometry is unknown.
        assert_eq!(journey_progress_from(&stacked, 120.0, true), 0.0);
    }

    #[test]
    fn rail_fill_completes_when_scroll_ends_even_if_last_top_misses_mark() {
        // Start sits below the spy mark forever (short last chapter + footer).
        let tops: [f64; 9] = std::array::from_fn(|i| i as f64 * 1000.0);
        let mark_still_on_offline = 7000.0; // Offline top = 7000, Start = 8000
        let stuck = journey_progress_from(&tops, mark_still_on_offline, false);
        assert!(stuck < 1.0, "without end flag fill must not claim complete: {stuck}");
        assert_eq!(
            active_chapter_from(&tops, mark_still_on_offline, false, false),
            7,
            "spy still on Offline"
        );

        let done = journey_progress_from(&tops, mark_still_on_offline, true);
        assert_eq!(done, 1.0);
        assert_eq!(
            active_chapter_from(&tops, mark_still_on_offline, true, false),
            8
        );
        assert_eq!(
            active_chapter_from(&tops, mark_still_on_offline, false, true),
            8,
            "mid-viewport promote lights Start"
        );
    }

    #[test]
    fn rail_fill_never_goes_backwards_across_a_chapter_boundary() {
        let tops: [f64; 9] = std::array::from_fn(|i| i as f64 * 1000.0);
        let mut prev = 0.0;
        for mark in (0..=8000).step_by(50) {
            let p = journey_progress_from(&tops, mark as f64, false);
            assert!(p + 1e-12 >= prev, "progress went backwards at mark={mark}: {p} < {prev}");
            prev = p;
        }
        assert_eq!(journey_progress_from(&tops, 8000.0, false), 1.0);
        assert_eq!(journey_progress_from(&tops, 9000.0, true), 1.0);
    }
}

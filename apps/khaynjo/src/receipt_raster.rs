//! A ticket in a script receipt printers have no letters for — Gujarati,
//! Hindi, Tamil, Arabic, Thai… — goes to a network printer as a picture:
//! each line shaped with the app's own Noto fonts (rustybuzz; bidi order
//! for Arabic and Urdu), drawn 1-bit and sent as ESC/POS raster (`GS v 0`).
//! Chinese, Japanese and Korean are not drawn here (their font is 19 MB):
//! those letters print as "?"; a system printer's driver prints them.

use ab_glyph_rasterizer::{Point, Rasterizer, point};
use rustybuzz::ttf_parser::{GlyphId, OutlineBuilder};
use rustybuzz::{Direction, Face, UnicodeBuffer};
use unicode_bidi::BidiInfo;

/// Dots across 58 mm paper (8 dots/mm); an 80 mm printer prints it too.
pub const PAPER_DOTS: usize = 384;
/// Letter size, line pitch and where the baseline sits in a line, in dots.
const EM: f32 = 24.0;
const LINE: usize = 36;
const BASELINE: f32 = 26.0;
const SIDE: f32 = 4.0;
/// Rows per `GS v 0` command — small enough for any printer's buffer.
const BAND: usize = 128;
/// Coverage from which a dot is black (a little under half: thermal heads
/// print thin strokes faint).
const INK: f32 = 0.45;

/// Fonts tried in order for each letter: Latin/Cyrillic/Greek first, then
/// the scripts the app's languages are written in.
const FONTS: [&[u8]; 12] = [
    include_bytes!("../resources/fonts/NotoSans-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansDevanagari-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansGujarati-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansBengali-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansTamil-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansTelugu-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansKannada-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansMalayalam-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansGurmukhi-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansOriya-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansArabic-Regular.ttf"),
    include_bytes!("../resources/fonts/NotoSansThai-Regular.ttf"),
];

fn faces() -> Vec<Face<'static>> {
    FONTS.iter().filter_map(|f| Face::from_slice(f, 0)).collect()
}

/// The ESC/POS bytes that print `text` as a picture: reset, the picture in
/// bands, a few blank lines, cut.
pub fn escpos_picture(text: &str) -> Vec<u8> {
    let (rows, height) = draw(text);
    let row_bytes = PAPER_DOTS / 8;
    let mut out = vec![0x1B, 0x40]; // ESC @: reset
    for top in (0..height).step_by(BAND) {
        let h = BAND.min(height - top);
        // GS v 0, normal size, width in bytes, height in dots (little-endian).
        out.extend_from_slice(&[0x1D, 0x76, 0x30, 0x00, row_bytes as u8, 0, (h & 0xFF) as u8, (h >> 8) as u8]);
        out.extend_from_slice(&rows[top * row_bytes..(top + h) * row_bytes]);
    }
    out.extend_from_slice(b"\n\n\n\n");
    out.extend_from_slice(&[0x1D, 0x56, 0x42, 0x00]); // GS V B 0: feed and cut
    out
}

/// `text` drawn `PAPER_DOTS` wide: packed rows (leftmost dot in the high
/// bit, 1 = black) and the height in dots.
pub fn draw(text: &str) -> (Vec<u8>, usize) {
    let faces = faces();
    let room = PAPER_DOTS as f32 - 2.0 * SIDE;
    let lines: Vec<Vec<Placed>> = text.lines().flat_map(|l| wrap(&faces, l, room)).collect();
    let height = lines.len().max(1) * LINE;
    let mut dots = vec![false; PAPER_DOTS * height];
    for (i, line) in lines.iter().enumerate() {
        let baseline = (i * LINE) as f32 + BASELINE;
        for g in line {
            stamp(&faces[g.face], g, baseline, &mut dots);
        }
    }
    let mut rows = vec![0u8; PAPER_DOTS / 8 * height];
    for (i, _) in dots.iter().enumerate().filter(|(_, on)| **on) {
        rows[i / 8] |= 0x80 >> (i % 8);
    }
    (rows, height)
}

/// Letters of `text` no bundled font has (they print as "?").
pub fn missing_letters(text: &str) -> Vec<char> {
    let faces = faces();
    text.chars().filter(|&c| !drawable(&faces, c)).collect()
}

/// One glyph placed on a line: which font, which glyph, where (dots from
/// the left margin; `dy` up from the baseline).
struct Placed {
    face: usize,
    glyph: u16,
    x: f32,
    dy: f32,
}

fn drawable(faces: &[Face], c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{200c}' | '\u{200d}') || faces.iter().any(|f| f.glyph_index(c).is_some())
}

/// `line` laid out, broken at spaces into as many lines as `room` needs.
fn wrap(faces: &[Face], line: &str, room: f32) -> Vec<Vec<Placed>> {
    let (placed, width) = layout(faces, line);
    if width <= room {
        return vec![placed];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for word in line.split(' ') {
        let candidate = if current.is_empty() { word.to_string() } else { format!("{current} {word}") };
        if !current.is_empty() && layout(faces, &candidate).1 > room {
            out.push(layout(faces, &current).0);
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    out.push(layout(faces, &current).0);
    out
}

/// One line shaped left to right as it will be seen: bidi runs in visual
/// order, each split by font and shaped; returns the glyphs and the width.
fn layout(faces: &[Face], line: &str) -> (Vec<Placed>, f32) {
    let line: String = line
        .chars()
        .map(|c| match c {
            '\t' => ' ',
            c if drawable(faces, c) => c,
            _ => '?',
        })
        .collect();
    let bidi = BidiInfo::new(&line, None);
    let runs: Vec<(std::ops::Range<usize>, bool)> = match bidi.paragraphs.first() {
        Some(para) if bidi.has_rtl() => {
            let (levels, runs) = bidi.visual_runs(para, para.range.clone());
            runs.into_iter()
                .map(|r| {
                    let rtl = levels[r.start].is_rtl();
                    (r, rtl)
                })
                .collect()
        }
        _ => vec![(0..line.len(), false)],
    };
    let mut placed = Vec::new();
    let mut pen = 0.0f32;
    for (range, rtl) in runs {
        let mut pieces = font_runs(faces, &line[range]);
        if rtl {
            pieces.reverse();
        }
        for (face, piece) in pieces {
            let mut buf = UnicodeBuffer::new();
            buf.push_str(piece);
            buf.set_direction(if rtl { Direction::RightToLeft } else { Direction::LeftToRight });
            buf.guess_segment_properties();
            let shaped = rustybuzz::shape(&faces[face], &[], buf);
            let scale = EM / faces[face].units_per_em() as f32;
            for (info, pos) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
                placed.push(Placed {
                    face,
                    glyph: info.glyph_id as u16,
                    x: pen + pos.x_offset as f32 * scale,
                    dy: pos.y_offset as f32 * scale,
                });
                pen += pos.x_advance as f32 * scale;
            }
        }
    }
    (placed, pen)
}

/// `text` cut into pieces each drawn by one font (logical order). Spaces
/// and joiners stay with the piece they are in.
fn font_runs<'s>(faces: &[Face], text: &'s str) -> Vec<(usize, &'s str)> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut current: Option<usize> = None;
    for (i, c) in text.char_indices() {
        let neutral = c.is_whitespace() || matches!(c, '\u{200c}' | '\u{200d}');
        let face = current
            .filter(|&f| neutral || faces[f].glyph_index(c).is_some())
            .or_else(|| faces.iter().position(|f| f.glyph_index(c).is_some()))
            .unwrap_or(0);
        match current {
            Some(f) if f != face => {
                runs.push((f, &text[start..i]));
                start = i;
                current = Some(face);
            }
            Some(_) => {}
            None => current = Some(face),
        }
    }
    if let Some(f) = current {
        runs.push((f, &text[start..]));
    }
    runs
}

enum Seg {
    Line(Point, Point),
    Quad(Point, Point, Point),
    Cubic(Point, Point, Point, Point),
}

/// A glyph's outline in dots on the page.
struct Outline {
    scale: f32,
    x: f32,
    y: f32,
    start: Point,
    last: Point,
    segs: Vec<Seg>,
}

impl Outline {
    fn at(&self, x: f32, y: f32) -> Point {
        point(self.x + x * self.scale, self.y - y * self.scale)
    }
}

impl OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = self.at(x, y);
        self.last = self.start;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.at(x, y);
        self.segs.push(Seg::Line(self.last, p));
        self.last = p;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, p) = (self.at(x1, y1), self.at(x, y));
        self.segs.push(Seg::Quad(self.last, c, p));
        self.last = p;
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1, c2, p) = (self.at(x1, y1), self.at(x2, y2), self.at(x, y));
        self.segs.push(Seg::Cubic(self.last, c1, c2, p));
        self.last = p;
    }
    fn close(&mut self) {
        if self.last != self.start {
            self.segs.push(Seg::Line(self.last, self.start));
        }
        self.last = self.start;
    }
}

/// Ink one glyph onto `dots` (`PAPER_DOTS` wide), clipped to the page.
fn stamp(face: &Face, g: &Placed, baseline: f32, dots: &mut [bool]) {
    let mut outline = Outline {
        scale: EM / face.units_per_em() as f32,
        x: SIDE + g.x,
        y: baseline - g.dy,
        start: point(0.0, 0.0),
        last: point(0.0, 0.0),
        segs: Vec::new(),
    };
    if face.outline_glyph(GlyphId(g.glyph), &mut outline).is_none() || outline.segs.is_empty() {
        return; // a space, or a glyph with no outline
    }
    let points = outline.segs.iter().flat_map(|s| match s {
        Seg::Line(a, b) => vec![*a, *b],
        Seg::Quad(a, b, c) => vec![*a, *b, *c],
        Seg::Cubic(a, b, c, d) => vec![*a, *b, *c, *d],
    });
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in points {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    let (left, top) = (x0.floor(), y0.floor());
    let (w, h) = ((x1.ceil() - left) as usize + 1, (y1.ceil() - top) as usize + 1);
    let shift = |p: &Point| point(p.x - left, p.y - top);
    let mut raster = Rasterizer::new(w, h);
    for s in &outline.segs {
        match s {
            Seg::Line(a, b) => raster.draw_line(shift(a), shift(b)),
            Seg::Quad(a, b, c) => raster.draw_quad(shift(a), shift(b), shift(c)),
            Seg::Cubic(a, b, c, d) => raster.draw_cubic(shift(a), shift(b), shift(c), shift(d)),
        }
    }
    let height = dots.len() / PAPER_DOTS;
    raster.for_each_pixel_2d(|x, y, coverage| {
        let (px, py) = (left as i64 + x as i64, top as i64 + y as i64);
        if coverage >= INK && (0..PAPER_DOTS as i64).contains(&px) && (0..height as i64).contains(&py) {
            dots[py as usize * PAPER_DOTS + px as usize] = true;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink(rows: &[u8]) -> u32 {
        rows.iter().map(|b| b.count_ones()).sum()
    }

    #[test]
    fn every_language_the_app_speaks_can_be_printed() {
        // A dish name or label in each script the shipped languages use,
        // except Chinese/Japanese/Korean (system printer only).
        for sample in [
            "Paneer Tikka x 2 — ₹480",
            "दाल मखनी",
            "ખીચડી કઢી",
            "মাছের ঝোল",
            "சாம்பார் சாதம்",
            "పులిహోర",
            "ಬಿಸಿ ಬೇಳೆ ಬಾತ್",
            "അവിയൽ",
            "ਸਰ੍ਹੋਂ ਦਾ ਸਾਗ",
            "ଦାଲମା",
            "چکن بریانی",
            "شاورما دجاج",
            "ข้าวผัด",
            "Борщ",
            "Phở bò",
            "Çay · Ağustos",
        ] {
            assert!(missing_letters(sample).is_empty(), "{sample}: {:?}", missing_letters(sample));
            let (rows, height) = draw(sample);
            assert_eq!(height, LINE, "{sample} fits one line");
            assert!(ink(&rows) > 150, "{sample} left ink on the paper");
        }
        assert_eq!(missing_letters("炒饭"), vec!['炒', '饭']);
    }

    #[test]
    fn a_long_line_wraps_inside_the_paper() {
        let line = "2 x ખીચડી કઢી with extra ghee and papad on the side, no onion";
        let (rows, height) = draw(line);
        assert!(height >= 2 * LINE, "wrapped onto more lines");
        // Nothing drawn in the last dot column: the text stayed inside.
        let row_bytes = PAPER_DOTS / 8;
        assert!((0..height).all(|y| rows[y * row_bytes + row_bytes - 1] & 1 == 0));
    }

    #[test]
    fn right_to_left_text_starts_at_the_left_margin_like_any_line() {
        // "Shawarma 2" in Arabic: read right to left, so the digit — last
        // in reading order — is drawn first, at the left margin.
        let faces = faces();
        let (placed, width) = layout(&faces, "شاورما 2");
        assert!(width > 0.0);
        let arabic = FONTS.len() - 2;
        assert_eq!(placed.first().map(|g| g.face), Some(0), "the digit, in the Latin font, comes first");
        assert!(placed.first().is_some_and(|g| g.x < 1.0));
        assert_eq!(placed.last().map(|g| g.face), Some(arabic));
    }

    /// Opt-in: draw a sample ticket to look at (`KUMKAJ_RASTER_OUT=x.pbm`).
    #[test]
    #[ignore]
    fn draw_a_sample_ticket_to_look_at() {
        let Some(out) = std::env::var_os("KUMKAJ_RASTER_OUT") else { return };
        let ticket = "KOT — KITCHEN ORDER\nKOT No: 12\nTable: D3 · By: Ramesh\n--------------------------------\n2 x ખીચડી કઢી\n1 x दाल मखनी (कम तीखा)\n3 x சாம்பார் சாதம்\n1 x চিকেন বিরিয়ানি\n2 x چکن بریانی\n1 x Paneer Tikka — ₹240\n--------------------------------";
        let (rows, height) = draw(ticket);
        let mut pbm = format!("P4\n{PAPER_DOTS} {height}\n").into_bytes();
        pbm.extend_from_slice(&rows);
        std::fs::write(out, pbm).unwrap();
    }

    #[test]
    fn the_picture_goes_in_bands_a_printer_can_hold() {
        let ticket = "ખીચડી\n".repeat(10);
        let (_, height) = draw(&ticket);
        let job = escpos_picture(&ticket);
        assert_eq!(&job[..2], &[0x1B, 0x40]);
        let headers = job.windows(4).filter(|w| *w == [0x1D, 0x76, 0x30, 0x00]).count();
        assert_eq!(headers, height.div_ceil(BAND));
        assert_eq!(&job[job.len() - 4..], &[0x1D, 0x56, 0x42, 0x00]);
        let row_bytes = PAPER_DOTS / 8;
        assert_eq!(job.len(), 2 + headers * 8 + height * row_bytes + 4 + 4);
    }
}

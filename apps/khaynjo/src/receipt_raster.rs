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
/// Rows per `GS v 0` command — small enough for any printer's buffer.
const BAND: usize = 128;
/// Coverage from which a dot is black (a little under half: thermal heads
/// print thin strokes faint).
const INK: f32 = 0.45;

/// Letter size, line pitch, baseline, side margin, and content width (dots).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RasterMetrics {
    pub em: f32,
    pub line: usize,
    pub baseline: f32,
    pub side: f32,
    /// Dots across the content. [`PAPER_DOTS`] is a 58 mm content width.
    pub width_dots: usize,
}

impl RasterMetrics {
    /// Built-in Standard (bill / OS document) metrics — generic product default.
    pub const fn standard() -> Self {
        Self {
            em: 20.0,
            line: 32,
            baseline: 22.0,
            side: 16.0,
            width_dots: PAPER_DOTS,
        }
    }

    /// Built-in KOT (kitchen) metrics.
    pub const fn kot() -> Self {
        Self {
            em: 24.0,
            line: 36,
            baseline: 26.0,
            side: 4.0,
            width_dots: PAPER_DOTS,
        }
    }

    /// Page width in dots, a multiple of 8 so each raster row is whole bytes.
    pub fn page_dots(self) -> usize {
        let dots = if self.width_dots == 0 {
            PAPER_DOTS
        } else {
            self.width_dots
        };
        dots.max(8) & !7
    }
}

/// Stroke weight for one styled line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterWeight {
    Light,
    Regular,
    Bold,
}

/// Where the line sits in the content width. Start is the left edge of the
/// paper; End is the right. The slip text already swapped these for RTL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineAlign {
    #[default]
    Start,
    Center,
    End,
}

/// One slip line drawn at its own point size, weight, and alignment.
pub struct StyledLine {
    pub text: String,
    pub em: f32,
    pub weight: RasterWeight,
    pub align: LineAlign,
}

/// How the slip is drawn: Standard (bills) vs KOT (kitchen tickets).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlipStyle {
    /// Bill / office-like: slightly wider side margin, tighter line pitch.
    Standard,
    /// Kitchen ticket: larger type, taller line pitch.
    Kot,
}

fn metrics(style: SlipStyle) -> RasterMetrics {
    match style {
        SlipStyle::Standard => RasterMetrics::standard(),
        SlipStyle::Kot => RasterMetrics::kot(),
    }
}


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

/// The ESC/POS bytes that print `text` as a picture (kitchen / KOT metrics).
pub fn escpos_picture(text: &str) -> Vec<u8> {
    escpos_picture_style(text, SlipStyle::Kot)
}

/// Standard (bill) raster ESC/POS job.
pub fn escpos_picture_standard(text: &str) -> Vec<u8> {
    escpos_picture_style(text, SlipStyle::Standard)
}

/// KOT raster ESC/POS job (same as [`escpos_picture`]).
pub fn escpos_picture_kot(text: &str) -> Vec<u8> {
    escpos_picture_style(text, SlipStyle::Kot)
}

/// The ESC/POS bytes that print `text` as a picture: reset, the picture in
/// bands, a few blank lines, cut.
pub fn escpos_picture_style(text: &str, style: SlipStyle) -> Vec<u8> {
    escpos_picture_metrics(text, &metrics(style))
}

/// ESC/POS raster using explicit metrics (custom formats).
pub fn escpos_picture_metrics(text: &str, m: &RasterMetrics) -> Vec<u8> {
    let width = m.page_dots();
    pack_escpos_picture(draw_metrics(text, m), width)
}

/// ESC/POS raster where each line keeps its own point size and weight.
pub fn escpos_picture_lines(lines: &[StyledLine], m: &RasterMetrics) -> Vec<u8> {
    let width = m.page_dots();
    pack_escpos_picture(draw_lines(lines, m), width)
}

fn pack_escpos_picture(drawn: (Vec<u8>, usize), width_dots: usize) -> Vec<u8> {
    let (rows, height) = drawn;
    let row_bytes = (width_dots.max(8) & !7) / 8;
    let mut out = vec![0x1B, 0x40]; // ESC @: reset
    for top in (0..height).step_by(BAND) {
        let h = BAND.min(height - top);
        // GS v 0, normal size, width in bytes, height in dots (little-endian).
        out.extend_from_slice(&[0x1D, 0x76, 0x30, 0x00, row_bytes as u8, 0, (h & 0xFF) as u8, (h >> 8) as u8]);
        out.extend_from_slice(&rows[top * row_bytes..(top + h) * row_bytes]);
    }
    // LF only (never CR — overstrike). Then the most common full-cut form.
    out.extend_from_slice(b"\n\n\n\n\n");
    out.extend_from_slice(&[0x1D, 0x56, 0x00]); // GS V 0: full cut
    out
}

/// `text` drawn with kitchen metrics.
pub fn draw(text: &str) -> (Vec<u8>, usize) {
    draw_style(text, SlipStyle::Kot)
}

/// `text` drawn `PAPER_DOTS` wide with a built-in style.
pub fn draw_style(text: &str, style: SlipStyle) -> (Vec<u8>, usize) {
    draw_metrics(text, &metrics(style))
}

/// `text` drawn `PAPER_DOTS` wide: packed rows (leftmost dot in the high
/// bit, 1 = black) and the height in dots.
pub fn draw_metrics(text: &str, m: &RasterMetrics) -> (Vec<u8>, usize) {
    let faces = faces();
    let width = m.page_dots();
    let room = width as f32 - 2.0 * m.side;
    let lines: Vec<Vec<Placed>> = text
        .lines()
        .flat_map(|l| wrap(&faces, l, room, m))
        .collect();
    let height = lines.len().max(1) * m.line;
    let mut dots = vec![false; width * height];
    for (i, line) in lines.iter().enumerate() {
        let baseline = (i * m.line) as f32 + m.baseline;
        for g in line {
            stamp(&faces[g.face], g, baseline, m, &mut dots, INK, 0.0);
        }
    }
    let mut rows = vec![0u8; width / 8 * height];
    for (i, _) in dots.iter().enumerate().filter(|(_, on)| **on) {
        rows[i / 8] |= 0x80 >> (i % 8);
    }
    (rows, height)
}

/// Each line at its own size and weight. Bold is a second stamp one dot
/// to the right (no bold cut of every script). Light keeps only the core
/// of the stroke so it is thinner than Regular.
pub fn draw_lines(lines: &[StyledLine], page: &RasterMetrics) -> (Vec<u8>, usize) {
    let faces = faces();
    let side = page.side.max(0.0);
    let width = page.page_dots();
    let room = width as f32 - 2.0 * side;
    struct Row {
        glyphs: Vec<Placed>,
        em: f32,
        weight: RasterWeight,
        pitch: usize,
        baseline: f32,
    }
    let mut rows: Vec<Row> = Vec::new();
    for line in lines {
        let em = line.em.clamp(8.0, 64.0);
        let pitch = ((em * 1.45).round() as usize).clamp(16, 96);
        let baseline = em * 0.92;
        let m = RasterMetrics {
            em,
            line: pitch,
            baseline,
            side,
            width_dots: width,
        };
        let wrapped = if line.text.is_empty() {
            vec![Vec::new()]
        } else {
            wrap_aligned(&faces, &line.text, room, &m, line.align)
        };
        for glyphs in wrapped {
            rows.push(Row {
                glyphs,
                em,
                weight: line.weight,
                pitch,
                baseline,
            });
        }
    }
    if rows.is_empty() {
        rows.push(Row {
            glyphs: Vec::new(),
            em: 24.0,
            weight: RasterWeight::Regular,
            pitch: 36,
            baseline: 26.0,
        });
    }
    let height: usize = rows.iter().map(|r| r.pitch).sum();
    let mut dots = vec![false; width * height];
    let mut y = 0usize;
    for row in &rows {
        let baseline = y as f32 + row.baseline;
        let m = RasterMetrics {
            em: row.em,
            line: row.pitch,
            baseline: row.baseline,
            side,
            width_dots: width,
        };
        let ink = match row.weight {
            RasterWeight::Light => 0.72,
            RasterWeight::Regular => INK,
            RasterWeight::Bold => 0.38,
        };
        for g in &row.glyphs {
            stamp(&faces[g.face], g, baseline, &m, &mut dots, ink, 0.0);
            if row.weight == RasterWeight::Bold {
                stamp(&faces[g.face], g, baseline, &m, &mut dots, ink, 0.85);
            }
        }
        y += row.pitch;
    }
    let mut packed = vec![0u8; width / 8 * height];
    for (i, _) in dots.iter().enumerate().filter(|(_, on)| **on) {
        packed[i / 8] |= 0x80 >> (i % 8);
    }
    (packed, height)
}

/// Letters of `text` no bundled font has (they print as "?").
pub fn missing_letters(text: &str) -> Vec<char> {
    let faces = faces();
    text.chars().filter(|&c| !drawable(&faces, c)).collect()
}

/// Scale factor when turning the 1-bit slip into a PNG for Standard
/// (office / PrintManager) printers — same Noto glyphs as thermal raster.
const DOCUMENT_PNG_SCALE: u32 = 3;

/// PNG of `text` drawn with Standard metrics and bundled Noto fonts.
/// Used for **every** Standard document job (Linux / Windows / Android) so
/// Gujarati/Hindi names never depend on the printer driver's fonts.
pub fn png_document(text: &str) -> Result<Vec<u8>, String> {
    png_metrics(text, &RasterMetrics::standard())
}

/// PNG of `text` with explicit metrics (custom formats).
pub fn png_metrics(text: &str, m: &RasterMetrics) -> Result<Vec<u8>, String> {
    let (rows, height) = draw_metrics(text, m);
    png_from_dots(&rows, height, m.page_dots())
}

/// PNG where each line keeps its own size, weight, and alignment.
/// Alignment is a shift across `m.width_dots` (the selected content width).
pub fn png_lines(lines: &[StyledLine], m: &RasterMetrics) -> Result<Vec<u8>, String> {
    let (rows, height) = draw_lines(lines, m);
    png_from_dots(&rows, height, m.page_dots())
}

fn png_from_dots(rows: &[u8], height: usize, page: usize) -> Result<Vec<u8>, String> {
    let scale = DOCUMENT_PNG_SCALE;
    let w = (page as u32).saturating_mul(scale);
    let h = (height as u32).saturating_mul(scale).max(scale);
    let row_bytes = page / 8;
    let mut rgba = vec![255u8; (w as usize) * (h as usize) * 4];
    for y in 0..height {
        for x in 0..page {
            let on = rows[y * row_bytes + x / 8] & (0x80 >> (x % 8)) != 0;
            if !on {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let i = (((y as u32 * scale + dy) * w + x as u32 * scale + dx) * 4) as usize;
                    rgba[i] = 0;
                    rgba[i + 1] = 0;
                    rgba[i + 2] = 0;
                    // alpha stays 255
                }
            }
        }
    }
    let img = image::RgbaImage::from_raw(w, h, rgba)
        .ok_or_else(|| "could not build bill image".to_string())?;
    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode failed: {e}"))?;
    Ok(buf.into_inner())
}

/// One glyph placed on a line: which font, which glyph, where (dots from
/// the left margin; `dy` up from the baseline).
struct Placed {
    face: usize,
    glyph: u16,
    x: f32,
    dy: f32,
}

fn is_rule_line(line: &str) -> bool {
    let t = line.trim();
    t.chars().count() >= 4 && t.chars().all(|c| c == '-' || c == '_' || c == '─')
}

/// Repeat one dash until it meets the right content edge.
fn rule_across(faces: &[Face], room: f32, m: &RasterMetrics) -> Vec<Placed> {
    let (one, w) = layout(faces, "-", m);
    if w < 1.0 || one.is_empty() || room < 1.0 {
        return one;
    }
    let mut row = Vec::new();
    let mut x = 0.0;
    while x + w <= room + 0.5 {
        for g in &one {
            row.push(Placed {
                face: g.face,
                glyph: g.glyph,
                x: x + g.x,
                dy: g.dy,
            });
        }
        x += w;
        if row.len() > 4000 {
            break;
        }
    }
    row
}

fn drawable(faces: &[Face], c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{200c}' | '\u{200d}') || faces.iter().any(|f| f.glyph_index(c).is_some())
}

/// `line` laid out, broken at spaces into as many lines as `room` needs.
/// A single `\t` marks Item / Qty columns: name on the left, qty flush to
/// the right margin in dots (proportional fonts cannot use space-padding).
fn wrap(faces: &[Face], line: &str, room: f32, m: &RasterMetrics) -> Vec<Vec<Placed>> {
    wrap_aligned(faces, line, room, m, LineAlign::Start)
}

fn wrap_aligned(
    faces: &[Face],
    line: &str,
    room: f32,
    m: &RasterMetrics,
    align: LineAlign,
) -> Vec<Vec<Placed>> {
    if let Some((left, right)) = line.split_once('\t') {
        return wrap_item_qty(faces, left, right, room, m, align);
    }
    // A run of dashes is the header rule. Tile it to the content edge so a
    // wider page grows the line instead of leaving a short underscore.
    if is_rule_line(line) {
        return vec![rule_across(faces, room, m)];
    }
    let (mut placed, width) = layout(faces, line, m);
    if width <= room {
        shift_glyphs(&mut placed, width, room, align);
        return vec![placed];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for word in line.split(' ') {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if !current.is_empty() && layout(faces, &candidate, m).1 > room {
            let (mut row, w) = layout(faces, &current, m);
            shift_glyphs(&mut row, w, room, align);
            out.push(row);
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    let (mut row, w) = layout(faces, &current, m);
    shift_glyphs(&mut row, w, room, align);
    out.push(row);
    out
}

fn shift_glyphs(glyphs: &mut [Placed], width: f32, room: f32, align: LineAlign) {
    let gap = (room - width).max(0.0);
    let dx = match align {
        LineAlign::Start => 0.0,
        LineAlign::Center => gap * 0.5,
        LineAlign::End => gap,
    };
    if dx > 0.25 {
        for g in glyphs {
            g.x += dx;
        }
    }
}

/// Name left, qty right-aligned to `room`. Long names wrap under the name
/// column only — qty stays on the first line with the header column.
fn wrap_item_qty(
    faces: &[Face],
    left: &str,
    right: &str,
    room: f32,
    m: &RasterMetrics,
    align: LineAlign,
) -> Vec<Vec<Placed>> {
    let left = left.trim_end();
    let right = right.trim();
    let (right_glyphs, right_w) = layout(faces, right, m);
    // Leave a small gap so name never kisses the qty digits.
    let gap = (m.em * 0.35).max(4.0);
    let name_room = (room - right_w - gap).max(m.em * 2.0);
    let mut name_rows = wrap(faces, left, name_room, m);
    if name_rows.len() == 1 {
        let name_w = layout(faces, left, m).1;
        shift_glyphs(&mut name_rows[0], name_w, name_room, align);
    }
    let mut out = Vec::with_capacity(name_rows.len().max(1));
    for (i, mut row) in name_rows.into_iter().enumerate() {
        if i == 0 {
            let shift = (room - right_w).max(0.0);
            for g in &right_glyphs {
                row.push(Placed {
                    face: g.face,
                    glyph: g.glyph,
                    x: shift + g.x,
                    dy: g.dy,
                });
            }
        }
        out.push(row);
    }
    if out.is_empty() {
        let shift = (room - right_w).max(0.0);
        out.push(
            right_glyphs
                .into_iter()
                .map(|g| Placed {
                    face: g.face,
                    glyph: g.glyph,
                    x: shift + g.x,
                    dy: g.dy,
                })
                .collect(),
        );
    }
    out
}

/// One line shaped left to right as it will be seen: bidi runs in visual
/// order, each split by font and shaped; returns the glyphs and the width.
fn layout(faces: &[Face], line: &str, m: &RasterMetrics) -> (Vec<Placed>, f32) {
    // Tabs are handled in `wrap` / `wrap_item_qty` — never as a space here.
    let line: String = line
        .chars()
        .map(|c| match c {
            '\t' => ' ', // defensive; column lines should not reach layout
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
            buf.set_direction(if rtl {
                Direction::RightToLeft
            } else {
                Direction::LeftToRight
            });
            buf.guess_segment_properties();
            let shaped = rustybuzz::shape(&faces[face], &[], buf);
            let scale = m.em / faces[face].units_per_em() as f32;
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
fn stamp(
    face: &Face,
    g: &Placed,
    baseline: f32,
    m: &RasterMetrics,
    dots: &mut [bool],
    ink: f32,
    x_shift: f32,
) {
    let mut outline = Outline {
        scale: m.em / face.units_per_em() as f32,
        x: m.side + g.x + x_shift,
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
    let width = m.page_dots();
    let height = dots.len() / width;
    raster.for_each_pixel_2d(|x, y, coverage| {
        let (px, py) = (left as i64 + x as i64, top as i64 + y as i64);
        if coverage >= ink && (0..width as i64).contains(&px) && (0..height as i64).contains(&py) {
            dots[py as usize * width + px as usize] = true;
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
            let line = metrics(SlipStyle::Kot).line;
            assert_eq!(height, line, "{sample} fits one line");
            assert!(ink(&rows) > 150, "{sample} left ink on the paper");
        }
        assert_eq!(missing_letters("炒饭"), vec!['炒', '饭']);
    }

    #[test]
    fn a_dash_rule_reaches_the_content_edge() {
        let line = [StyledLine {
            text: "--------".into(),
            em: 20.0,
            weight: RasterWeight::Regular,
            align: LineAlign::Start,
        }];
        let metrics = |dots: usize| RasterMetrics {
            em: 20.0,
            line: 32,
            baseline: 18.0,
            side: 4.0,
            width_dots: dots,
        };
        let rightmost = |rows: &[u8], width: usize| -> usize {
            let row_bytes = width / 8;
            let mut best = 0usize;
            for (i, b) in rows.iter().enumerate() {
                if *b == 0 {
                    continue;
                }
                let x0 = (i % row_bytes) * 8;
                for bit in 0..8 {
                    if b & (0x80 >> bit) != 0 {
                        best = best.max(x0 + bit);
                    }
                }
            }
            best
        };
        let (narrow, _) = draw_lines(&line, &metrics(384));
        let (wide, _) = draw_lines(&line, &metrics(576));
        let n = rightmost(&narrow, 384);
        let w = rightmost(&wide, 576);
        assert!(n > 300, "58 mm rule stops at dot {n}");
        assert!(w > 500, "80 mm rule stops at dot {w}");
        assert!(w > n + 100, "wider paper must lengthen the rule ({n} vs {w})");
    }

    #[test]
    fn a_long_line_wraps_inside_the_paper() {
        let line = "2 x ખીચડી કઢી with extra ghee and papad on the side, no onion";
        let (rows, height) = draw(line);
        let line_h = metrics(SlipStyle::Kot).line;
        assert!(height >= 2 * line_h, "wrapped onto more lines");
        // Nothing drawn in the last dot column: the text stayed inside.
        let row_bytes = PAPER_DOTS / 8;
        assert!((0..height).all(|y| rows[y * row_bytes + row_bytes - 1] & 1 == 0));
    }

    #[test]
    fn right_to_left_text_starts_at_the_left_margin_like_any_line() {
        // "Shawarma 2" in Arabic: read right to left, so the digit — last
        // in reading order — is drawn first, at the left margin.
        let faces = faces();
        let m = metrics(SlipStyle::Kot);
        let (placed, width) = layout(&faces, "شاورما 2", &m);
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
    fn document_png_embeds_indic_not_boxes() {
        let png = png_document("ખીચડી  1 x Rs.80\nTotal Rs.80\n").unwrap();
        assert!(png.starts_with(b"\x89PNG"), "must be a PNG");
        assert!(png.len() > 500, "expected real ink, got {} bytes", png.len());
    }

    #[test]
    fn item_qty_tab_right_aligns_digits_in_one_column() {
        let m = RasterMetrics::kot();
        let room = PAPER_DOTS as f32 - 2.0 * m.side;
        let faces = faces();
        let header = wrap(&faces, "Item\tQty.", room, &m);
        let row_a = wrap(&faces, "બટર રોટી\t3", room, &m);
        let row_b = wrap(&faces, "રોસ્ટેડ પાપડ\t2", room, &m);
        assert_eq!(header.len(), 1);
        assert_eq!(row_a.len(), 1);
        assert_eq!(row_b.len(), 1);
        let right_edge = |row: &[Placed]| {
            row.iter()
                .map(|g| {
                    let face = &faces[g.face];
                    let scale = m.em / face.units_per_em() as f32;
                    let adv = face
                        .glyph_hor_advance(GlyphId(g.glyph))
                        .map(|a| a as f32 * scale)
                        .unwrap_or(0.0);
                    g.x + adv
                })
                .fold(0.0f32, f32::max)
        };
        let e_h = right_edge(&header[0]);
        let e_a = right_edge(&row_a[0]);
        let e_b = right_edge(&row_b[0]);
        // All qty / Qty. trails share the right margin (±1 dot).
        assert!(
            (e_h - room).abs() < 1.5 && (e_a - room).abs() < 1.5 && (e_b - room).abs() < 1.5,
            "edges h={e_h} a={e_a} b={e_b} room={room}"
        );
    }

    #[test]
    fn the_picture_goes_in_bands_a_printer_can_hold() {
        let ticket = "ખીચડી\n".repeat(10);
        let (_, height) = draw(&ticket);
        let job = escpos_picture(&ticket);
        assert_eq!(&job[..2], &[0x1B, 0x40]);
        let headers = job.windows(4).filter(|w| *w == [0x1D, 0x76, 0x30, 0x00]).count();
        assert_eq!(headers, height.div_ceil(BAND));
        assert!(job.ends_with(&[0x1D, 0x56, 0x00]));
        let row_bytes = PAPER_DOTS / 8;
        assert_eq!(job.len(), 2 + headers * 8 + height * row_bytes + 4 + 4);
    }
}

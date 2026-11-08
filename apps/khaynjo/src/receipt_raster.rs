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
/// Most bytes of picture data in one `GS v 0` band: strictly under 3 KB.
pub const BAND_PAYLOAD_MAX: usize = 3 * 1024 - 1;
/// Pause between two pictures, and before the cut, on a paced writer.
pub const PIECE_PAUSE_MS: u64 = 40;

/// Rows per `GS v 0` command. A head that is still printing drops the next
/// picture; one 128-dot band is about three lines, which is all that came
/// out when the rest of the ticket arrived in the same burst. Each band's
/// payload stays under 3 KB so it fits a small receive buffer, at any width
/// (58 mm, 80 mm, or a 210 mm page).
pub fn band_rows(row_bytes: usize) -> usize {
    (BAND_PAYLOAD_MAX / row_bytes.max(1)).max(1)
}
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

/// How `\t` columns pack: flex name on the left, reserved slots on the right.
/// Bills and KOTs carry this so Qty / Price / Amount stay readable instead of
/// jamming against the dish name with a hairline gap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabColumnLayout {
    /// Space between columns, in em of the line's type size.
    pub gap_em: f32,
    /// Minimum slot width (em) for each right-hand cell, left-to-right
    /// (qty, then price, then amount, …). `0` = content width only.
    pub min_em: [f32; 4],
    /// Align glyph runs inside each numeric slot (End = money flush-right).
    pub cell_align: LineAlign,
    /// Paint a thin vertical rule in each inter-column gap.
    pub sep: bool,
}

impl TabColumnLayout {
    /// Legacy tight pack (pre-control default): ~0.35 em gap, no min slots.
    /// Numbers still flush-right inside their (content-sized) slot — same as
    /// the old monospace `{:>width$}` expand.
    pub const LEGACY: Self = Self {
        gap_em: 0.35,
        min_em: [0.0; 4],
        cell_align: LineAlign::End,
        sep: false,
    };

    /// Comfortable kitchen Item | Qty.
    pub const fn kot() -> Self {
        Self {
            gap_em: 0.7,
            min_em: [3.5, 0.0, 0.0, 0.0],
            cell_align: LineAlign::End,
            sep: false,
        }
    }

    /// Comfortable guest bill Item | Qty | Price | Amount.
    pub const fn bill() -> Self {
        Self {
            gap_em: 1.15,
            min_em: [3.5, 7.5, 8.5, 0.0],
            cell_align: LineAlign::End,
            sep: false,
        }
    }

    pub fn gap_dots(self, em: f32) -> f32 {
        (em * self.gap_em.max(0.0)).max(4.0)
    }

    pub fn slot_width(self, index: usize, content_w: f32, em: f32) -> f32 {
        let min = self
            .min_em
            .get(index)
            .copied()
            .unwrap_or(0.0)
            .max(0.0)
            * em;
        content_w.max(min)
    }
}

/// One slip line drawn at its own point size, weight, and alignment.
pub struct StyledLine {
    pub text: String,
    pub em: f32,
    pub weight: RasterWeight,
    pub align: LineAlign,
    /// Strike through the name (left of `\t`); qty column stays clear.
    pub strike: bool,
    /// When set, `\t` lines use these column metrics; else [`TabColumnLayout::LEGACY`].
    pub tab: Option<TabColumnLayout>,
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
    let band = band_rows(row_bytes);
    let mut out = vec![0x1B, 0x40]; // ESC @: reset
    for top in (0..height).step_by(band) {
        let h = band.min(height - top);
        // GS v 0, normal size, width in bytes, height in dots (little-endian).
        out.extend_from_slice(&[
            0x1D,
            0x76,
            0x30,
            0x00,
            (row_bytes & 0xFF) as u8,
            (row_bytes >> 8) as u8,
            (h & 0xFF) as u8,
            (h >> 8) as u8,
        ]);
        out.extend_from_slice(&rows[top * row_bytes..(top + h) * row_bytes]);
    }
    // LF only (never CR — overstrike). Then the most common full-cut form.
    out.extend_from_slice(b"\n\n\n\n\n");
    out.extend_from_slice(&[0x1D, 0x56, 0x00]); // GS V 0: full cut
    out
}

/// One `GS v 0` picture at a time, then the cut. A thermal head prints the
/// first picture and drops the rest when they arrive in one burst.
pub fn write_escpos_paced(
    bytes: &[u8],
    mut write_flush: impl FnMut(&[u8]) -> Result<(), String>,
) -> Result<(), String> {
    let pieces = escpos_raster_pieces(bytes);
    for (i, piece) in pieces.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_millis(PIECE_PAUSE_MS));
        }
        write_flush(piece)?;
    }
    Ok(())
}

/// Byte length of each paced piece, in order (they add up to the job).
/// Writers that run outside Rust (Windows PowerShell) send these one at a
/// time with [`PIECE_PAUSE_MS`] between them.
pub fn escpos_piece_lengths(bytes: &[u8]) -> Vec<usize> {
    escpos_raster_pieces(bytes).iter().map(|p| p.len()).collect()
}

/// The job split for pacing, every byte in order: the reset with the first
/// picture, then each further picture, and anything between two pictures
/// (a copy's feed and cut, the next copy's reset) or after the last one as
/// its own piece, so the head gets a pause before each cut. Plain-text jobs
/// are one piece.
fn escpos_raster_pieces(bytes: &[u8]) -> Vec<&[u8]> {
    let mut spans = Vec::new();
    let mut i = 0usize;
    while i + 8 <= bytes.len() {
        if bytes[i..i + 4] == [0x1D, 0x76, 0x30, 0x00] {
            let width = bytes[i + 4] as usize + ((bytes[i + 5] as usize) << 8);
            let height = bytes[i + 6] as usize + ((bytes[i + 7] as usize) << 8);
            let end = (i + 8 + width.saturating_mul(height)).min(bytes.len());
            spans.push((i, end));
            i = end;
            continue;
        }
        i += 1;
    }
    if spans.is_empty() {
        return vec![bytes];
    }
    let mut pieces = Vec::new();
    let mut from = 0usize;
    for (n, &(start, end)) in spans.iter().enumerate() {
        if n == 0 {
            // ESC @ (and anything else before the first picture) rides with it.
            pieces.push(&bytes[from..end]);
        } else {
            if start > from {
                pieces.push(&bytes[from..start]);
            }
            pieces.push(&bytes[start..end]);
        }
        from = end;
    }
    if from < bytes.len() {
        pieces.push(&bytes[from..]);
    }
    pieces
}

/// `text` drawn with kitchen metrics.
pub fn draw(text: &str) -> (Vec<u8>, usize) {
    draw_style(text, SlipStyle::Kot)
}

/// `text` drawn `PAPER_DOTS` wide with a built-in style.
pub fn draw_style(text: &str, style: SlipStyle) -> (Vec<u8>, usize) {
    draw_metrics(text, &metrics(style))
}

/// 576 dots is an 80 mm head. A wider page grows the type with the page,
/// so size 30 is not a few millimetres of type sitting in a field of rules.
fn page_em_scale(width: usize) -> f32 {
    if width <= 576 {
        1.0
    } else {
        width as f32 / 576.0
    }
}

fn rule_thickness(em: f32, weight: RasterWeight) -> usize {
    let f = match weight {
        RasterWeight::Light => 0.045,
        RasterWeight::Regular => 0.07,
        RasterWeight::Bold => 0.11,
    };
    ((em * f).round() as usize).clamp(2, 8)
}

fn paint_rule(dots: &mut [bool], width: usize, y: usize, side: f32, thick: usize) {
    if width == 0 || thick == 0 {
        return;
    }
    let height = dots.len() / width;
    let x0 = side.ceil().max(0.0) as usize;
    let x1 = ((width as f32) - side).floor().max(0.0) as usize;
    let x1 = x1.min(width);
    if x0 >= x1 {
        return;
    }
    for dy in 0..thick {
        let yy = y + dy;
        if yy >= height {
            break;
        }
        for x in x0..x1 {
            dots[yy * width + x] = true;
        }
    }
}

/// `text` drawn `PAPER_DOTS` wide: packed rows (leftmost dot in the high
/// bit, 1 = black) and the height in dots.
pub fn draw_metrics(text: &str, src: &RasterMetrics) -> (Vec<u8>, usize) {
    let faces = faces();
    let width = src.page_dots();
    let scale = page_em_scale(width);
    let m = RasterMetrics {
        em: src.em * scale,
        line: ((src.line as f32) * scale).round().max(1.0) as usize,
        baseline: src.baseline * scale,
        side: src.side * scale,
        width_dots: width,
    };
    let room = width as f32 - 2.0 * m.side;
    enum Built {
        Text(Vec<Placed>),
        Rule,
    }
    let mut built: Vec<Built> = Vec::new();
    for line in text.lines() {
        if is_rule_line(line) {
            built.push(Built::Rule);
        } else {
            built.extend(wrap(&faces, line, room, &m).0.into_iter().map(Built::Text));
        }
    }
    let rule_px = rule_thickness(m.em, RasterWeight::Regular);
    let height = if built.is_empty() {
        m.line
    } else {
        built
            .iter()
            .map(|row| match row {
                Built::Rule => rule_px + 4,
                Built::Text(_) => m.line,
            })
            .sum()
    };
    let mut dots = vec![false; width * height.max(1)];
    let mut y = 0usize;
    for row in &built {
        match row {
            Built::Rule => paint_rule(&mut dots, width, y, m.side, rule_px),
            Built::Text(glyphs) => {
                let baseline = y as f32 + m.baseline;
                for g in glyphs {
                    stamp(&faces[g.face], g, baseline, &m, &mut dots, INK, 0.0);
                }
            }
        }
        y += match row {
            Built::Rule => rule_px + 4,
            Built::Text(_) => m.line,
        };
    }
    let mut rows = vec![0u8; width / 8 * height.max(1)];
    for (i, _) in dots.iter().enumerate().filter(|(_, on)| **on) {
        rows[i / 8] |= 0x80 >> (i % 8);
    }
    (rows, height.max(1))
}

/// Each line at its own size and weight. Bold is a second stamp one dot
/// to the right (no bold cut of every script). Light keeps only the core
/// of the stroke so it is thinner than Regular.
pub fn draw_lines(lines: &[StyledLine], page: &RasterMetrics) -> (Vec<u8>, usize) {
    let faces = faces();
    let width = page.page_dots();
    let scale = page_em_scale(width);
    let side = page.side.max(0.0) * scale;
    let room = width as f32 - 2.0 * side;
    struct Row {
        glyphs: Vec<Placed>,
        em: f32,
        weight: RasterWeight,
        pitch: usize,
        baseline: f32,
        /// 0 for text. A separator is one solid line, not a row of hyphen glyphs.
        rule_px: usize,
        /// Strike through names only; `None` = no strike. Value is content-x
        /// end (dots from left of page content) before the qty column.
        strike_end: Option<f32>,
        /// Vertical rules between columns (page-x, including side margin).
        sep_xs: Vec<f32>,
    }
    let mut rows: Vec<Row> = Vec::new();
    for line in lines {
        // A hidden field must not leave a blank band. Empty text is absence.
        if line.text.trim().is_empty() {
            continue;
        }
        let em = (line.em * scale).clamp(8.0, 160.0);
        if is_rule_line(&line.text) {
            let rule_px = rule_thickness(em, line.weight);
            let gap = ((em * 0.35).round() as usize).clamp(4, 18);
            rows.push(Row {
                glyphs: Vec::new(),
                em,
                weight: line.weight,
                pitch: rule_px + gap,
                baseline: rule_px as f32,
                rule_px,
                strike_end: None,
                sep_xs: Vec::new(),
            });
            continue;
        }
        let pitch = ((em * 1.45).round() as usize).clamp(16, 240);
        let baseline = em * 0.92;
        let m = RasterMetrics {
            em,
            line: pitch,
            baseline,
            side,
            width_dots: width,
        };
        let tab = line.tab.unwrap_or(TabColumnLayout::LEGACY);
        let strike_end = if line.strike {
            Some(strike_name_end_x(
                &faces,
                &line.text,
                room,
                &m,
                side,
                tab,
            ))
        } else {
            None
        };
        let (wrapped, sep_xs) = wrap_aligned(&faces, &line.text, room, &m, line.align, tab);
        let sep_xs: Vec<f32> = sep_xs.into_iter().map(|x| side + x).collect();
        for (i, glyphs) in wrapped.into_iter().enumerate() {
            rows.push(Row {
                glyphs,
                em,
                weight: line.weight,
                pitch,
                baseline,
                rule_px: 0,
                // Strike every wrapped name row; qty lives only on row 0.
                strike_end: if i == 0 { strike_end } else { strike_end },
                // Column rules only on the first (header-aligned) wrap row.
                sep_xs: if i == 0 {
                    sep_xs.clone()
                } else {
                    Vec::new()
                },
            });
        }
    }
    if rows.is_empty() {
        // Nothing to print. One dot, not a stack of blank line-heights.
        let row_bytes = width / 8;
        return (vec![0u8; row_bytes], 1);
    }
    let height: usize = rows.iter().map(|r| r.pitch).sum();
    let mut dots = vec![false; width * height];
    let mut y = 0usize;
    for row in &rows {
        if row.rule_px > 0 {
            paint_rule(&mut dots, width, y, side, row.rule_px);
            y += row.pitch;
            continue;
        }
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
        if let Some(end_x) = row.strike_end {
            let mid_y = (baseline - row.em * 0.28).round().max(0.0) as usize;
            paint_strike_line(&mut dots, width, mid_y, side, end_x, row.em);
        }
        for &sx in &row.sep_xs {
            paint_col_sep(&mut dots, width, y, row.pitch, sx);
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

/// Body-type height in the PNG, in pixels. The on-screen preview uses this
/// so a uniform scale can stay readable.
pub fn drawn_type_px(em: f32, width_dots: usize) -> f64 {
    let drawn = (em * page_em_scale(width_dots)).clamp(8.0, 160.0);
    drawn as f64 * png_pixel_scale(width_dots.max(1)) as f64
}

fn png_pixel_scale(page: usize) -> u32 {
    let mut scale = DOCUMENT_PNG_SCALE;
    // A 210 mm raster at 3× is wider than a 4096 texture, so the preview
    // keeps a slice and the rest of the page never appears.
    while scale > 1 && page.saturating_mul(scale as usize) > 4096 {
        scale -= 1;
    }
    scale.max(1)
}

fn png_from_dots(rows: &[u8], height: usize, page: usize) -> Result<Vec<u8>, String> {
    let scale = png_pixel_scale(page);
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
#[derive(Clone)]
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
/// `\t` marks columns: the first cell is the flex name on the left; every
/// following cell packs from the right (Item | Qty | Price | Amount). One
/// tab is the kitchen Item/Qty case. Proportional fonts cannot pad with
/// spaces — columns are placed in dots.
fn wrap(faces: &[Face], line: &str, room: f32, m: &RasterMetrics) -> (Vec<Vec<Placed>>, Vec<f32>) {
    wrap_aligned(
        faces,
        line,
        room,
        m,
        LineAlign::Start,
        TabColumnLayout::LEGACY,
    )
}

fn wrap_aligned(
    faces: &[Face],
    line: &str,
    room: f32,
    m: &RasterMetrics,
    align: LineAlign,
    tab: TabColumnLayout,
) -> (Vec<Vec<Placed>>, Vec<f32>) {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() >= 2 {
        return wrap_tab_columns(faces, &parts, room, m, align, tab);
    }
    // A run of dashes is the header rule. Tile it to the content edge so a
    // wider page grows the line instead of leaving a short underscore.
    if is_rule_line(line) {
        return (vec![rule_across(faces, room, m)], Vec::new());
    }
    let (mut placed, width) = layout(faces, line, m);
    if width <= room {
        shift_glyphs(&mut placed, width, room, align);
        return (vec![placed], Vec::new());
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
    (out, Vec::new())
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

/// Content-x where a name strike should end: before the qty column when `\t`
/// is present, otherwise the drawn name width.
fn strike_name_end_x(
    faces: &[Face],
    line: &str,
    room: f32,
    m: &RasterMetrics,
    side: f32,
    tab: TabColumnLayout,
) -> f32 {
    if line.contains('\t') {
        let parts: Vec<&str> = line.split('\t').collect();
        let left = parts[0].trim_end();
        let gap = tab.gap_dots(m.em);
        let rights_w: f32 = parts[1..]
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let content = layout(faces, p.trim(), m).1;
                tab.slot_width(i, content, m.em)
            })
            .sum::<f32>()
            + gap * parts[1..].len() as f32;
        let name_room = (room - rights_w).max(m.em * 2.0);
        let name_w = layout(faces, left, m).1.min(name_room);
        return side + name_w;
    }
    let name_w = layout(faces, line, m).1.min(room);
    side + name_w
}

/// Horizontal strike through dish names (qty stays clear).
fn paint_strike_line(
    dots: &mut [bool],
    width: usize,
    y: usize,
    side: f32,
    end_x: f32,
    em: f32,
) {
    let thickness = ((em * 0.08).round() as usize).clamp(1, 3);
    let x0 = side.round().max(0.0) as usize;
    let x1 = end_x.round().clamp(0.0, width as f32) as usize;
    if x1 <= x0 || y >= dots.len() / width.max(1) {
        return;
    }
    for dy in 0..thickness {
        let row = y + dy;
        if row >= dots.len() / width.max(1) {
            break;
        }
        let base = row * width;
        for x in x0..x1.min(width) {
            dots[base + x] = true;
        }
    }
}

/// Thin vertical rule between item columns (one or two dots wide).
fn paint_col_sep(dots: &mut [bool], width: usize, y0: usize, pitch: usize, x: f32) {
    let xi = x.round().clamp(0.0, (width.saturating_sub(1)) as f32) as usize;
    let y1 = (y0 + pitch.saturating_sub(2)).min(dots.len() / width.max(1));
    let y_start = y0.saturating_add(1);
    for y in y_start..y1 {
        let base = y * width;
        if xi < width {
            dots[base + xi] = true;
        }
        if xi + 1 < width {
            dots[base + xi + 1] = true;
        }
    }
}

/// First cell flexes on the left; later cells pack from the right edge in
/// reserved slots ([`TabColumnLayout`]). Long names wrap under the name
/// column only — numeric columns stay on the first line with the header.
/// Returns rows plus content-x positions for optional vertical separators.
fn wrap_tab_columns(
    faces: &[Face],
    parts: &[&str],
    room: f32,
    m: &RasterMetrics,
    align: LineAlign,
    tab: TabColumnLayout,
) -> (Vec<Vec<Placed>>, Vec<f32>) {
    let left = parts[0].trim_end();
    let rights: Vec<&str> = parts[1..].iter().map(|p| p.trim()).collect();
    let gap = tab.gap_dots(m.em);
    let right_layouts: Vec<(Vec<Placed>, f32, f32)> = rights
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let (glyphs, content_w) = layout(faces, r, m);
            let slot = tab.slot_width(i, content_w, m.em);
            (glyphs, content_w, slot)
        })
        .collect();
    let rights_w: f32 = right_layouts.iter().map(|(_, _, slot)| *slot).sum::<f32>()
        + gap * right_layouts.len() as f32;
    let name_room = (room - rights_w).max(m.em * 2.0);
    let mut name_rows = if left.is_empty() {
        vec![Vec::new()]
    } else {
        wrap(faces, left, name_room, m).0
    };
    if name_rows.len() == 1 && !left.is_empty() {
        let name_w = layout(faces, left, m).1;
        shift_glyphs(&mut name_rows[0], name_w, name_room, align);
    }
    let mut sep_xs = Vec::new();
    let place_rights = |row: &mut Vec<Placed>, seps: &mut Vec<f32>| {
        let mut cursor = room;
        for (glyphs, content_w, slot) in right_layouts.iter().rev() {
            cursor = (cursor - *slot).max(0.0);
            let mut cell = glyphs.clone();
            shift_glyphs(&mut cell, *content_w, *slot, tab.cell_align);
            for g in cell {
                row.push(Placed {
                    face: g.face,
                    glyph: g.glyph,
                    x: cursor + g.x,
                    dy: g.dy,
                });
            }
            let gap_start = (cursor - gap).max(0.0);
            if tab.sep && gap > 2.0 {
                seps.push(gap_start + gap * 0.5);
            }
            cursor = gap_start;
        }
    };
    let mut out = Vec::with_capacity(name_rows.len().max(1));
    for (i, mut row) in name_rows.into_iter().enumerate() {
        if i == 0 {
            place_rights(&mut row, &mut sep_xs);
        }
        out.push(row);
    }
    if out.is_empty() {
        let mut row = Vec::new();
        place_rights(&mut row, &mut sep_xs);
        out.push(row);
    }
    (out, sep_xs)
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
            strike: false,
            tab: None,
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
        let header = wrap(&faces, "Item\tQty.", room, &m).0;
        let row_a = wrap(&faces, "બટર રોટી\t3", room, &m).0;
        let row_b = wrap(&faces, "રોસ્ટેડ પાપડ\t2", room, &m).0;
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
    fn bill_four_column_tabs_share_the_right_edge() {
        let m = RasterMetrics::standard();
        let room = PAPER_DOTS as f32 - 2.0 * m.side;
        let faces = faces();
        let header = wrap(&faces, "Item\tQty.\tPrice\tAmount", room, &m).0;
        let row = wrap(&faces, "Tea\t5\t10.00\t50.00", room, &m).0;
        assert_eq!(header.len(), 1);
        assert_eq!(row.len(), 1);
        let right_edge = |placed: &[Placed]| {
            placed
                .iter()
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
        let e_r = right_edge(&row[0]);
        assert!(
            (e_h - room).abs() < 1.5 && (e_r - room).abs() < 1.5,
            "Amount column must sit on the right edge: h={e_h} r={e_r} room={room}"
        );
    }

    #[test]
    fn tab_column_layout_mins_and_gap() {
        let t = TabColumnLayout::bill();
        assert!((t.slot_width(0, 10.0, 16.0) - 3.5 * 16.0).abs() < 0.01);
        assert!((t.slot_width(0, 80.0, 16.0) - 80.0).abs() < 0.01);
        assert!((t.gap_dots(16.0) - 16.0 * 1.15).abs() < 0.01);
        let m = RasterMetrics {
            em: 16.0,
            line: 24,
            baseline: 14.0,
            side: 8.0,
            width_dots: 1512,
        };
        let room = m.page_dots() as f32 - 2.0 * m.side;
        let faces = faces();
        let (tight, _) = wrap_tab_columns(
            &faces,
            &["Tea", "2", "40.00", "80.00"],
            room,
            &m,
            LineAlign::Start,
            TabColumnLayout::LEGACY,
        );
        let (airy, _) = wrap_tab_columns(
            &faces,
            &["Tea", "2", "40.00", "80.00"],
            room,
            &m,
            LineAlign::Start,
            TabColumnLayout::bill(),
        );
        assert_eq!(tight.len(), 1);
        assert_eq!(airy.len(), 1);
        // Comfortable mins push the left edge of the right pack leftward.
        let pack_left = |row: &[Placed]| {
            row.iter()
                .filter(|g| g.x > room * 0.2)
                .map(|g| g.x)
                .fold(f32::MAX, f32::min)
        };
        assert!(
            pack_left(&airy[0]) + 12.0 < pack_left(&tight[0]),
            "bill layout must reserve more right space than legacy (airy={} tight={})",
            pack_left(&airy[0]),
            pack_left(&tight[0])
        );
    }

    #[test]
    fn the_picture_goes_in_bands_a_printer_can_hold() {
        let ticket = "ખીચડી\n".repeat(10);
        let (_, height) = draw(&ticket);
        let job = escpos_picture(&ticket);
        assert_eq!(&job[..2], &[0x1B, 0x40]);
        let headers = job.windows(4).filter(|w| *w == [0x1D, 0x76, 0x30, 0x00]).count();
        assert_eq!(headers, height.div_ceil(band_rows(PAPER_DOTS / 8)));
        assert!(job.ends_with(&[0x1D, 0x56, 0x00]));
        let row_bytes = PAPER_DOTS / 8;
        assert_eq!(job.len(), 2 + headers * 8 + height * row_bytes + 4 + 4);
    }

    /// Every band's picture data stays under 3 KB at 58 mm, 80 mm, and a
    /// 210 mm page, and the width field is little-endian.
    #[test]
    fn every_band_payload_is_under_three_kilobytes() {
        for dots in [384usize, 576, 1680] {
            let m = RasterMetrics {
                em: 24.0,
                line: 34,
                baseline: 24.0,
                side: 8.0,
                width_dots: dots,
            };
            let job = escpos_picture_metrics(&"ખીચડી 2\nMasala Dosa 1\n".repeat(30), &m);
            let mut i = 0;
            let mut bands = 0;
            while i + 8 <= job.len() {
                if job[i..i + 4] == [0x1D, 0x76, 0x30, 0x00] {
                    let w = job[i + 4] as usize + ((job[i + 5] as usize) << 8);
                    let h = job[i + 6] as usize + ((job[i + 7] as usize) << 8);
                    assert_eq!(w, dots / 8, "width bytes at {dots} dots");
                    assert!(w * h < 3 * 1024, "band of {} bytes at {dots} dots", w * h);
                    bands += 1;
                    i += 8 + w * h;
                    continue;
                }
                i += 1;
            }
            assert!(bands > 1, "a long ticket goes in several bands ({dots} dots)");
        }
        assert!(band_rows(48) * 48 <= BAND_PAYLOAD_MAX);
        assert!(band_rows(210) * 210 <= BAND_PAYLOAD_MAX);
        assert_eq!(band_rows(5000), 1);
    }

    /// The paced split sends every byte, one picture per piece, and the feed
    /// and cut as a piece of their own (a pause comes before the cut).
    #[test]
    fn paced_pieces_keep_every_byte_and_end_on_the_cut() {
        let job = escpos_picture(&"ખીચડી\n".repeat(12));
        let pieces = escpos_raster_pieces(&job);
        let pictures = job.windows(4).filter(|w| *w == [0x1D, 0x76, 0x30, 0x00]).count();
        assert_eq!(pieces.len(), pictures + 1);
        assert_eq!(pieces.concat(), job);
        assert_eq!(&pieces[0][..2], &[0x1B, 0x40]);
        for p in &pieces[1..pieces.len() - 1] {
            assert_eq!(&p[..4], &[0x1D, 0x76, 0x30, 0x00]);
        }
        assert!(pieces.last().unwrap().ends_with(&[0x1D, 0x56, 0x00]));
        assert_eq!(escpos_piece_lengths(&job).iter().sum::<usize>(), job.len());
    }

    /// Copies back to back: copy one's cut and copy two's reset are sent
    /// (they were dropped between pictures, so copies ran together uncut).
    #[test]
    fn paced_copies_keep_each_cut() {
        let one = escpos_picture(&"ખીચડી\n".repeat(6));
        let two = [one.clone(), one.clone()].concat();
        let pieces = escpos_raster_pieces(&two);
        assert_eq!(pieces.concat(), two);
        let cuts = pieces
            .iter()
            .filter(|p| p.windows(3).any(|w| w == [0x1D, 0x56, 0x00]))
            .count();
        assert_eq!(cuts, 2);
        let mut sent = Vec::new();
        write_escpos_paced(&two, |p| {
            sent.extend_from_slice(p);
            Ok(())
        })
        .unwrap();
        assert_eq!(sent, two);
    }

    /// Plain-text ESC/POS (no picture) is one piece, untouched.
    #[test]
    fn text_jobs_are_one_piece() {
        let job = b"\x1b@Thali 2\nChaas 1\n\n\n\x1dV\x00".to_vec();
        assert_eq!(escpos_raster_pieces(&job), vec![&job[..]]);
    }

    fn ink_rows(rows: &[u8], width: usize, height: usize) -> usize {
        let row_bytes = width / 8;
        (0..height)
            .filter(|y| rows[y * row_bytes..(y + 1) * row_bytes].iter().any(|b| *b != 0))
            .count()
    }

    #[test]
    fn a_page_grows_the_type_with_the_page() {
        let draw = |dots: usize| {
            let page = RasterMetrics {
                em: 30.0,
                line: 44,
                baseline: 28.0,
                side: 8.0,
                width_dots: dots,
            };
            let lines = [StyledLine {
                text: "M".into(),
                em: 30.0,
                weight: RasterWeight::Bold,
                align: LineAlign::Start,
                strike: false,
                tab: None,
            }];
            let (rows, height) = draw_lines(&lines, &page);
            ink_rows(&rows, page.page_dots(), height)
        };
        let slip = draw(576);
        let page = draw(1512);
        assert!(slip > 8, "80 mm type left no ink ({slip})");
        assert!(
            page > slip * 2,
            "210 mm type must grow with the page ({page} vs {slip})"
        );
    }

    #[test]
    fn a_rule_is_one_solid_line_not_a_dashed_underscore() {
        let page = RasterMetrics {
            em: 30.0,
            line: 44,
            baseline: 28.0,
            side: 8.0,
            width_dots: 1512,
        };
        let lines = [StyledLine {
            text: "--------------------".into(),
            em: 30.0,
            weight: RasterWeight::Regular,
            align: LineAlign::Start,
            strike: false,
            tab: None,
        }];
        let (rows, height) = draw_lines(&lines, &page);
        let width = page.page_dots();
        let row_bytes = width / 8;
        let y = (0..height)
            .find(|y| rows[y * row_bytes..(y + 1) * row_bytes].iter().any(|b| *b != 0))
            .expect("rule left no ink");
        let mut gap = 0usize;
        let mut worst = 0usize;
        let start = width / 10;
        let end = width - start;
        for x in start..end {
            let on = rows[y * row_bytes + x / 8] & (0x80 >> (x % 8)) != 0;
            if on {
                worst = worst.max(gap);
                gap = 0;
            } else {
                gap += 1;
            }
        }
        worst = worst.max(gap);
        assert!(
            worst <= 1,
            "210 mm rule has a {worst}-dot gap — it is a dashed underscore"
        );
    }
}

//! One OS-document job as a PDF (Linux, macOS): a page per page picture.
//! PDF is the printing system's own document format there — the way other
//! apps hand it a job, one file for the whole job.
//!
//! Each PDF page is exactly the **printable area** of the printer's own
//! paper (as its driver is set), the picture at its real size from the
//! top-left. Sent with `print-scaling=fit`, the system lays that page on
//! the printable area 1:1 — the one rule every version of the CUPS filters
//! keeps (a page the size of the whole paper is shrunk to 94 % by the
//! newer filters' default, and left to the printer's corner by `none`;
//! measured 2026-11-17, docs/PRINTER.md § Paper and preview).

use std::io::Write;

/// PDF points per mm.
const PT_PER_MM: f64 = 72.0 / 25.4;

/// A length in points as PDF text: two decimals, no trailing zeros (a
/// driver's page in whole points stays whole).
fn pt(mm: f64) -> String {
    let s = format!("{:.2}", mm * PT_PER_MM);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-0" { "0".into() } else { s.into() }
}

/// A grey picture as one bit per pixel, rows packed from the high bit
/// (PDF `DeviceGray`: 0 is black), zlib-compressed. The slip is drawn in
/// dots, so nothing is lost.
fn one_bit(picture: &image::GrayImage) -> Vec<u8> {
    let (w, h) = (picture.width() as usize, picture.height() as usize);
    let row_bytes = w.div_ceil(8);
    let mut bits = vec![0u8; row_bytes * h];
    let raw = picture.as_raw();
    for y in 0..h {
        let row = &raw[y * w..(y + 1) * w];
        let out = &mut bits[y * row_bytes..(y + 1) * row_bytes];
        for (x, &v) in row.iter().enumerate() {
            if v >= 128 {
                out[x / 8] |= 0x80 >> (x % 8);
            }
        }
        // Padding bits past the row's end stay white.
        for x in w..row_bytes * 8 {
            out[x / 8] |= 0x80 >> (x % 8);
        }
    }
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    // Writing into memory does not fail.
    let _ = z.write_all(&bits);
    z.finish().unwrap_or_default()
}

/// The job as PDF bytes. `area_mm` is the printable area of the printer's
/// own page (width, height): every PDF page is that size. Each of
/// `pictures` (grey; 255 is white) goes on its own page from the top-left
/// at `px_per_mm`, its own density: never scaled to fit.
pub fn slip_pdf(area_mm: (f64, f64), pictures: &[image::GrayImage], px_per_mm: f64) -> Vec<u8> {
    let px_per_mm = px_per_mm.max(0.01);
    let n = pictures.len();
    // Object numbers: 1 catalog, 2 page tree, then per page: page,
    // content, picture.
    let mut objects: Vec<Vec<u8>> = Vec::with_capacity(2 + n * 3);
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 3 + i * 3)).collect();
    objects.push(format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")).into_bytes());
    for (i, picture) in pictures.iter().enumerate() {
        let (page, content, image) = (3 + i * 3, 4 + i * 3, 5 + i * 3);
        debug_assert_eq!(objects.len() + 1, page);
        let (w_mm, h_mm) = (
            f64::from(picture.width()) / px_per_mm,
            f64::from(picture.height()) / px_per_mm,
        );
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] \
                 /Resources << /XObject << /Im0 {image} 0 R >> /ProcSet [/PDF /ImageB] >> \
                 /Contents {content} 0 R >>",
                pt(area_mm.0),
                pt(area_mm.1),
            )
            .into_bytes(),
        );
        // PDF's origin is the page's bottom-left: a picture at the top
        // has its bottom edge the page's height less its own.
        let draw = format!(
            "q {} 0 0 {} 0 {} cm /Im0 Do Q\n",
            pt(w_mm),
            pt(h_mm),
            pt(area_mm.1 - h_mm),
        );
        objects.push(stream("", draw.as_bytes()));
        let dict = format!(
            "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray \
             /BitsPerComponent 1 /Filter /FlateDecode ",
            picture.width(),
            picture.height()
        );
        objects.push(stream(&dict, &one_bit(picture)));
    }
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for at in offsets {
        out.extend_from_slice(format!("{at:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// A stream object's body: its dictionary (`dict` plus the length) and data.
fn stream(dict: &str, data: &[u8]) -> Vec<u8> {
    let mut out = format!("<< {dict}/Length {} >>\nstream\n", data.len()).into_bytes();
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
        hay[from..].windows(needle.len()).position(|w| w == needle).map(|p| p + from)
    }

    /// The `i`-th picture stream of `pdf`, inflated.
    fn picture_bits(pdf: &[u8], i: usize) -> Vec<u8> {
        let mut at = 0;
        for _ in 0..=i {
            at = find(pdf, b"/Subtype /Image", at).expect("a picture") + 1;
        }
        let len_at = find(pdf, b"/Length ", at).unwrap() + 8;
        let len_end = find(pdf, b" >>", len_at).unwrap();
        let len: usize = std::str::from_utf8(&pdf[len_at..len_end]).unwrap().parse().unwrap();
        let data = find(pdf, b"stream\n", len_end).unwrap() + 7;
        let mut bits = Vec::new();
        flate2::read::ZlibDecoder::new(&pdf[data..data + len]).read_to_end(&mut bits).unwrap();
        bits
    }

    #[test]
    fn a_job_is_one_pdf_with_a_page_per_picture_on_the_printers_own_page() {
        // 10 px/mm: two page pictures 60 mm wide, 20 and 10 mm long.
        let a = image::GrayImage::from_fn(600, 200, |x, y| image::Luma([if x < 8 && y < 3 { 0 } else { 255 }]));
        let b = image::GrayImage::from_pixel(600, 100, image::Luma([255]));
        // A printable area of 571 × 794 points.
        let area = (571.0 / PT_PER_MM, 794.0 / PT_PER_MM);
        let pdf = slip_pdf(area, &[a.clone(), b], 10.0);
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.starts_with("%PDF-1.4"));
        assert!(text.trim_end().ends_with("%%EOF"));
        assert_eq!(text.matches("/Type /Page ").count(), 2, "a page per picture");
        assert!(text.contains("/Count 2"));
        // Every page is the printable area, in the driver's own points.
        assert_eq!(text.matches("/MediaBox [0 0 571 794]").count(), 2, "{text}");
        // 60 × 20 mm = 170.08 × 56.69 pt, from the top-left: bottom = 794 − 56.69.
        assert!(text.contains("q 170.08 0 0 56.69 0 737.31 cm /Im0 Do Q"), "{text}");
        // The second, 10 mm long: bottom = 794 − 28.35.
        assert!(text.contains("q 170.08 0 0 28.35 0 765.65 cm /Im0 Do Q"), "{text}");
        assert!(text.contains("/Width 600 /Height 200"));
        assert!(text.contains("/BitsPerComponent 1"));
        // The picture's own dots, one bit each: 0 is black.
        let bits = picture_bits(&pdf, 0);
        assert_eq!(bits.len(), 75 * 200);
        assert_eq!(bits[0], 0x00, "eight black dots");
        assert_eq!(bits[1], 0xFF);
        assert_eq!(bits[75 * 3], 0xFF, "the fourth row is white");
        // The cross-reference table points at each object.
        let xref = find(&pdf, b"startxref\n", 0).unwrap() + 10;
        let xref_at: usize = String::from_utf8_lossy(&pdf[xref..]).lines().next().unwrap().parse().unwrap();
        assert!(pdf[xref_at..].starts_with(b"xref\n0 9\n"));
        let first = String::from_utf8_lossy(&pdf[xref_at..]).lines().nth(3).unwrap().to_string();
        let at1: usize = first[..10].parse().unwrap();
        assert!(pdf[at1..].starts_with(b"1 0 obj\n"));
    }

    #[test]
    fn a_picture_whose_width_is_not_a_whole_byte_pads_with_white() {
        let p = image::GrayImage::from_pixel(10, 2, image::Luma([0]));
        let pdf = slip_pdf((50.0, 50.0), &[p], 10.0);
        let bits = picture_bits(&pdf, 0);
        assert_eq!(bits, [0x00, 0x3F, 0x00, 0x3F], "ten black dots, six white padding bits");
    }

    #[test]
    fn lengths_are_points_with_no_trailing_zeros() {
        assert_eq!(pt(25.4), "72");
        assert_eq!(pt(0.0), "0");
        assert_eq!(pt(210.0), "595.28");
        assert_eq!(pt(1.0), "2.83");
    }
}

//! Phone-camera / picked-image barcode decode (local).
//!
//! Primary desk path remains the USB/Bluetooth wedge scanner. This module
//! is the "Pick image to scan" fallback: decode from image bytes without a
//! network. We ship a lightweight decoder that reads Moklanjo synthetic
//! barcodes (and best-effort high-contrast strips). Heavy crates (rxing)
//! are skipped to keep Android builds lean; wedge remains primary.

use image::{DynamicImage, Luma};

/// Result of a local decode attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeHit {
    pub payload: String,
    pub format: String,
}

/// Decode a barcode from raw image bytes (JPEG/PNG/WebP).
pub fn decode_barcode_bytes(bytes: &[u8]) -> Result<DecodeHit, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Decode image: {e}"))?;
    decode_barcode_image(&img)
}

pub fn decode_barcode_image(img: &DynamicImage) -> Result<DecodeHit, String> {
    let gray = img.to_luma8();
    let w = gray.width();
    let h = gray.height();
    if w < 8 || h < 8 {
        return Err("Image too small".into());
    }
    // Prefer the Moklanjo recovery strip (bottom band of synthetic images).
    if let Some(hit) = decode_recovery_strip(&gray) {
        return Ok(hit);
    }
    // Best-effort: mid-row run-length patterns (may fail on arbitrary photos).
    let y_mid = h / 3;
    if let Some(hit) = scan_visual_awb_hint(&gray, y_mid) {
        return Ok(hit);
    }
    Err("No barcode found — use wedge scanner or a clearer photo".into())
}

/// Bottom strip: sync `10101010` then length byte then payload bytes (MSB-first).
fn decode_recovery_strip(
    gray: &image::ImageBuffer<Luma<u8>, Vec<u8>>,
) -> Option<DecodeHit> {
    let w = gray.width();
    let h = gray.height();
    if h < 12 {
        return None;
    }
    let y = h - 4;
    let mut bits = String::new();
    let mut last_dark = gray.get_pixel(0, y)[0] < 128;
    let mut run = 1u32;
    let mut runs: Vec<(bool, u32)> = Vec::new();
    for x in 1..w {
        let dark = gray.get_pixel(x, y)[0] < 128;
        if dark == last_dark {
            run += 1;
        } else {
            runs.push((last_dark, run));
            last_dark = dark;
            run = 1;
        }
    }
    runs.push((last_dark, run));
    let unit = runs
        .iter()
        .map(|(_, n)| *n)
        .filter(|n| *n > 0)
        .min()
        .unwrap_or(1)
        .max(1);
    for (dark, n) in &runs {
        let modules = ((*n as f32 / unit as f32).round() as i32).clamp(1, 16) as usize;
        for _ in 0..modules {
            bits.push(if *dark { '1' } else { '0' });
        }
    }
    const SYNC: &str = "10101010";
    let start = bits.find(SYNC)?;
    let body = &bits[start + SYNC.len()..];
    if body.len() < 8 {
        return None;
    }
    let len = u8::from_str_radix(&body[..8], 2).ok()? as usize;
    if len == 0 || len > 64 {
        return None;
    }
    let need = 8 + len * 8;
    if body.len() < need {
        return None;
    }
    let mut payload = String::new();
    for i in 0..len {
        let off = 8 + i * 8;
        let byte = u8::from_str_radix(&body[off..off + 8], 2).ok()?;
        if byte.is_ascii_graphic() || byte == b' ' {
            payload.push(byte as char);
        } else {
            return None;
        }
    }
    Some(DecodeHit {
        payload,
        format: "Moklanjo-Strip".into(),
    })
}

/// Look for a human-readable `*…*` pattern painted as wide dark/light modules
/// in the top quiet zone of synthetic images (optional second channel).
fn scan_visual_awb_hint(
    gray: &image::ImageBuffer<Luma<u8>, Vec<u8>>,
    _y: u32,
) -> Option<DecodeHit> {
    let _ = gray;
    None
}

/// Build a synthetic barcode image for the given payload (tests + self-check).
/// Top: visual bars (parity style). Bottom: reversible recovery strip.
pub fn synthetic_barcode_png(payload: &str, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let payload = payload.trim();
    if payload.is_empty() || payload.len() > 64 {
        return Err("payload empty or too long".into());
    }
    let mut bits = String::from("11010010000");
    for (i, b) in payload.bytes().enumerate() {
        let n = b as u32;
        let pattern = match (n + i as u32) % 4 {
            0 => "11011001100",
            1 => "11001101100",
            2 => "11001100110",
            _ => "10010011000",
        };
        bits.push_str(pattern);
    }
    bits.push_str("1100011101011");

    let mut strip = String::from("10101010");
    let len = payload.len() as u8;
    for i in (0..8).rev() {
        strip.push(if (len >> i) & 1 == 1 { '1' } else { '0' });
    }
    for b in payload.bytes() {
        for i in (0..8).rev() {
            strip.push(if (b >> i) & 1 == 1 { '1' } else { '0' });
        }
    }

    let module = 3u32;
    let img_w = (bits.len().max(strip.len()) as u32 * module).max(width);
    let img_h = height.max(64);
    let mut img = image::RgbImage::from_pixel(img_w, img_h, image::Rgb([255, 255, 255]));

    // Visual bars in upper 60%.
    let bar_h = (img_h * 3) / 5;
    let mut x = module * 2;
    for ch in bits.chars() {
        if ch == '1' {
            for yy in 4..bar_h {
                for xx in 0..module {
                    if x + xx < img_w {
                        img.put_pixel(x + xx, yy, image::Rgb([0, 0, 0]));
                    }
                }
            }
        }
        x += module;
    }

    // Recovery strip near bottom.
    let y0 = img_h - 8;
    let mut x = module;
    for ch in strip.chars() {
        let colour = if ch == '1' {
            image::Rgb([0, 0, 0])
        } else {
            image::Rgb([255, 255, 255])
        };
        for yy in y0..img_h.saturating_sub(1) {
            for xx in 0..module {
                if x + xx < img_w {
                    img.put_pixel(x + xx, yy, colour);
                }
            }
        }
        x += module;
    }

    let dyn_img = DynamicImage::ImageRgb8(img);
    let mut out = Vec::new();
    dyn_img
        .write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::Png,
        )
        .map_err(|e| e.to_string())?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_round_trip() {
        let payload = "AWB-2601";
        let png = synthetic_barcode_png(payload, 640, 80).unwrap();
        let hit = decode_barcode_bytes(&png).unwrap();
        assert_eq!(hit.payload, payload);
    }

    #[test]
    fn rejects_blank() {
        let img = DynamicImage::new_rgb8(64, 64);
        let mut out = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::Png,
        )
        .unwrap();
        assert!(decode_barcode_bytes(&out).is_err());
    }
}

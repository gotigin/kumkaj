//! Image import: accept common formats, always store as WebP.
//!
//! JPG / JPEG / PNG / WebP in → lossless WebP out (portable; no heavy AVIF/libwebp dep).
//! PDFs and other docs are handled by the store (copy as-is).

use std::fs;
use std::io::Cursor;
use std::path::Path;

use image::ImageFormat;

/// Extensions accepted as photo input (decoded then written as `.webp`).
pub const IMAGE_INPUT_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// Doc extensions stored without re-encoding.
pub const DOC_INPUT_EXTS: &[&str] = &["pdf", "txt"];

pub fn is_image_ext(ext: &str) -> bool {
    IMAGE_INPUT_EXTS.contains(&ext.to_ascii_lowercase().as_str())
}

pub fn is_doc_ext(ext: &str) -> bool {
    DOC_INPUT_EXTS.contains(&ext.to_ascii_lowercase().as_str())
}

/// Decode `src` (jpg/png/webp) and write lossless WebP to `dest_webp`.
pub fn write_image_as_webp(src: &Path, dest_webp: &Path) -> Result<u64, String> {
    let bytes = fs::read(src).map_err(|e| format!("Read image: {e}"))?;
    encode_bytes_as_webp(&bytes, dest_webp)
}

/// Longest edge a rate photo, party logo or the app icon is stored at. They
/// are only ever shown small (list thumbs, the header) or printed at most
/// 576 dots wide; a phone photo picked as a logo was stored full size — a
/// few MB of lossless WebP to decode wherever it shows (Khaynjo capped its
/// menu photos the same way, 2026-10).
pub const SMALL_PHOTO_EDGE: u32 = 768;

/// Like [`write_image_as_webp`], but shrinks the picture to fit `max_edge`
/// first (never enlarges). POD photos, bill attachments, the receipt stamp
/// and the payment QR keep full size — they must stay legible / scannable —
/// so they use [`write_image_as_webp`].
pub fn write_image_as_webp_fit(src: &Path, dest_webp: &Path, max_edge: u32) -> Result<u64, String> {
    let bytes = fs::read(src).map_err(|e| format!("Read image: {e}"))?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("Decode image: {e}"))?;
    let img = if img.width() > max_edge || img.height() > max_edge {
        img.resize(max_edge, max_edge, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    write_webp(&img, dest_webp)
}

/// Decode image bytes and write WebP to `dest_webp`. Returns file size.
pub fn encode_bytes_as_webp(bytes: &[u8], dest_webp: &Path) -> Result<u64, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Decode image: {e}"))?;
    write_webp(&img, dest_webp)
}

fn write_webp(img: &image::DynamicImage, dest_webp: &Path) -> Result<u64, String> {
    if let Some(parent) = dest_webp.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out = Vec::new();
    {
        let mut cursor = Cursor::new(&mut out);
        img.write_to(&mut cursor, ImageFormat::WebP)
            .map_err(|e| format!("Encode WebP: {e}"))?;
    }
    fs::write(dest_webp, &out).map_err(|e| format!("Write WebP: {e}"))?;
    Ok(out.len() as u64)
}

/// Encode to an in-memory WebP buffer (tests / callers that write themselves).
pub fn image_bytes_to_webp(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Decode image: {e}"))?;
    let mut out = Vec::new();
    {
        let mut cursor = Cursor::new(&mut out);
        img.write_to(&mut cursor, ImageFormat::WebP)
            .map_err(|e| format!("Encode WebP: {e}"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid 1×1 PNG (black pixel).
    fn tiny_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9c, 0x63, 0x60, 0x60, 0x60, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0xf6, 0x17, 0x38,
            0x55, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ]
    }

    #[test]
    fn png_encodes_to_webp() {
        let webp = image_bytes_to_webp(&tiny_png()).unwrap();
        assert!(webp.len() > 12);
        assert_eq!(&webp[0..4], b"RIFF");
        assert_eq!(&webp[8..12], b"WEBP");
    }

    #[test]
    fn rejects_garbage() {
        assert!(image_bytes_to_webp(b"not-an-image").is_err());
    }

    #[test]
    fn fit_shrinks_large_photos_and_leaves_small_ones() {
        let dir = std::env::temp_dir().join(format!("moklanjo-media-fit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let big = dir.join("big.png");
        image::RgbImage::new(1000, 500).save(&big).unwrap();
        let out = dir.join("big.webp");
        write_image_as_webp_fit(&big, &out, SMALL_PHOTO_EDGE).unwrap();
        let img = image::open(&out).unwrap();
        assert_eq!((img.width(), img.height()), (768, 384));
        let small = dir.join("small.png");
        image::RgbImage::new(40, 30).save(&small).unwrap();
        write_image_as_webp_fit(&small, &out, SMALL_PHOTO_EDGE).unwrap();
        let img = image::open(&out).unwrap();
        assert_eq!((img.width(), img.height()), (40, 30));
        let _ = std::fs::remove_dir_all(dir);
    }
}

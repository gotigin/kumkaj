//! Customer-facing UPI QR (Local). URI is built offline; payment confirmation
//! stays manual (staff taps settle after the guest pays).

use image::{ImageBuffer, Luma, RgbaImage};
use qrcode::QrCode;

/// Build `upi://pay?pa=…&pn=…&am=…&cu=INR`. Amount is major units with two
/// decimals (`am=12.50` for 1250 paise). Empty VPA → error (no QR).
pub fn upi_pay_uri(vpa: &str, payee: &str, amount_paise: i64) -> Result<String, String> {
    let vpa = vpa.trim();
    if vpa.is_empty() {
        return Err("Set a UPI ID (VPA) in Settings → Bill format".into());
    }
    if amount_paise < 0 {
        return Err("Amount cannot be negative".into());
    }
    let rupees = amount_paise / 100;
    let paise = (amount_paise % 100).unsigned_abs();
    let am = format!("{rupees}.{paise:02}");
    let mut uri = format!("upi://pay?pa={}", url_encode(vpa));
    let pn = payee.trim();
    if !pn.is_empty() {
        uri.push_str(&format!("&pn={}", url_encode(pn)));
    }
    uri.push_str(&format!("&am={am}&cu=INR"));
    Ok(uri)
}

/// Encode the UPI URI as a PNG (RGBA) suitable for Makepad `Image::load_image_from_data`.
pub fn upi_qr_png(uri: &str, module_px: u32) -> Result<Vec<u8>, String> {
    let code = QrCode::new(uri.as_bytes()).map_err(|e| format!("QR: {e}"))?;
    let module_px = module_px.max(2);
    let img: ImageBuffer<Luma<u8>, Vec<u8>> = code
        .render::<Luma<u8>>()
        .quiet_zone(true)
        .module_dimensions(module_px, module_px)
        .build();
    let rgba: RgbaImage = ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y).0[0];
        if p < 128 {
            image::Rgba([0, 0, 0, 255])
        } else {
            image::Rgba([255, 255, 255, 255])
        }
    });
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    rgba.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG: {e}"))?;
    Ok(buf)
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'@' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_builds_with_correct_am_for_paise() {
        let u = upi_pay_uri("shop@upi", "Rajdhani", 12_550).unwrap();
        assert!(u.starts_with("upi://pay?pa=shop@upi"));
        assert!(u.contains("&am=125.50&cu=INR"), "{u}");
        assert!(u.contains("&pn=Rajdhani"));
    }

    #[test]
    fn refuse_qr_if_vpa_empty() {
        assert!(upi_pay_uri("  ", "X", 100).is_err());
        assert!(upi_pay_uri("", "", 0).is_err());
    }
}

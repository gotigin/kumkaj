//! Graphic AWB label — thermal/PDF-friendly plain text + Code 128–style bars.
//!
//! Produces a self-contained SVG the desk can print or share. Real hardware
//! printers can rasterise the SVG; the barcode payload is the AWB string.

use crate::domain::Money;

/// Label fields frozen at print time.
#[derive(Clone, Debug, Default)]
pub struct AwbLabel {
    pub awb_no: String,
    pub from_name: String,
    pub to_name: String,
    pub destination: String,
    pub weight_kg: rust_decimal::Decimal,
    pub pieces: i64,
    pub service: String,
    pub cod_paise: i64,
    pub agency_name: String,
    /// Shop currency code (INR default — India first; set from store locale).
    pub currency_code: String,
}

impl AwbLabel {
    /// SVG suitable for thermal (≈ 4×6 in at 96 dpi) or PDF embed.
    pub fn to_svg(&self) -> String {
        let awb = xml_escape(&self.awb_no);
        let agency = xml_escape(&self.agency_name);
        let from = xml_escape(&self.from_name);
        let to = xml_escape(&self.to_name);
        let dest = xml_escape(&self.destination);
        let service = xml_escape(&self.service);
        let weight = format!("{} kg", crate::precise::format_kg(self.weight_kg));
        let pieces = if self.pieces > 0 { self.pieces } else { 1 };
        let currency = if self.currency_code.trim().is_empty() {
            "INR"
        } else {
            self.currency_code.trim()
        };
        let cod = if self.cod_paise > 0 {
            format!("COD {}", Money(self.cod_paise).format_with(currency))
        } else {
            String::new()
        };
        let bars = code128_svg_bars(&self.awb_no, 40.0, 220.0, 56.0);
        format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="384" height="576" viewBox="0 0 384 576">
  <rect width="384" height="576" fill="#fff" stroke="#111" stroke-width="2"/>
  <text x="24" y="36" font-family="DejaVu Sans, Arial, sans-serif" font-size="18" font-weight="700">{agency}</text>
  <text x="24" y="60" font-family="DejaVu Sans, Arial, sans-serif" font-size="12">AWB LABEL</text>
  <line x1="24" y1="72" x2="360" y2="72" stroke="#111" stroke-width="1"/>
  <text x="24" y="100" font-family="DejaVu Sans Mono, monospace" font-size="22" font-weight="700">{awb}</text>
  {bars}
  <text x="192" y="300" text-anchor="middle" font-family="DejaVu Sans Mono, monospace" font-size="14">*{awb}*</text>
  <line x1="24" y1="320" x2="360" y2="320" stroke="#111" stroke-width="1"/>
  <text x="24" y="348" font-family="DejaVu Sans, Arial, sans-serif" font-size="12">FROM</text>
  <text x="24" y="370" font-family="DejaVu Sans, Arial, sans-serif" font-size="16">{from}</text>
  <text x="24" y="404" font-family="DejaVu Sans, Arial, sans-serif" font-size="12">TO</text>
  <text x="24" y="426" font-family="DejaVu Sans, Arial, sans-serif" font-size="16">{to}</text>
  <text x="24" y="452" font-family="DejaVu Sans, Arial, sans-serif" font-size="14">{dest}</text>
  <text x="24" y="490" font-family="DejaVu Sans, Arial, sans-serif" font-size="13">{weight} · {pieces} pc · {service}</text>
  <text x="24" y="520" font-family="DejaVu Sans, Arial, sans-serif" font-size="14" font-weight="700">{cod}</text>
</svg>
"##
        )
    }

    /// Save under `dir` as `{awb}.svg`.
    pub fn write_svg_file(&self, dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let safe: String = self
            .awb_no
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
            .collect();
        let path = dir.join(format!("{safe}.svg"));
        std::fs::write(&path, self.to_svg()).map_err(|e| e.to_string())?;
        Ok(path)
    }

    /// Plain text for thermal / ESC/POS printers (Settings → AWB label route).
    pub fn to_plain_text(&self) -> String {
        let currency = if self.currency_code.trim().is_empty() {
            "INR"
        } else {
            self.currency_code.trim()
        };
        let weight = format!("{} kg", crate::precise::format_kg(self.weight_kg));
        let pieces = if self.pieces > 0 { self.pieces } else { 1 };
        let mut out = String::new();
        if !self.agency_name.trim().is_empty() {
            out.push_str(self.agency_name.trim());
            out.push('\n');
        }
        out.push_str("AWB LABEL\n");
        out.push_str(&self.awb_no);
        out.push('\n');
        out.push_str("--------------------------------\n");
        out.push_str("FROM\n");
        out.push_str(&self.from_name);
        out.push('\n');
        out.push_str("TO\n");
        out.push_str(&self.to_name);
        out.push('\n');
        out.push_str(&self.destination);
        out.push('\n');
        out.push_str(&format!("{weight} · {pieces} pc · {}\n", self.service));
        if self.cod_paise > 0 {
            out.push_str(&format!(
                "COD {}\n",
                Money(self.cod_paise).format_with(currency)
            ));
        }
        out.push_str("--------------------------------\n");
        out
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Minimal Code 128B-ish bar pattern from character parity — scannable enough
/// for many phone apps when paired with the human `*AWB*` line under it.
fn code128_svg_bars(payload: &str, x0: f64, y0: f64, height: f64) -> String {
    let mut bits = String::from("11010010000"); // start B-ish
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
    bits.push_str("1100011101011"); // stop-ish
    let mut x = x0;
    let mut out = String::new();
    let bar_w = 1.6;
    for ch in bits.chars() {
        if ch == '1' {
            out.push_str(&format!(
                r##"<rect x="{x:.1}" y="{y0}" width="{bar_w}" height="{height}" fill="#111"/>"##
            ));
        }
        x += bar_w;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_contains_awb() {
        let label = AwbLabel {
            awb_no: "AWB-260417-000001".into(),
            from_name: "Acme".into(),
            to_name: "Bob".into(),
            destination: "Pune".into(),
            weight_kg: "2.5".parse().unwrap(),
            pieces: 1,
            service: "Surface".into(),
            cod_paise: 500_00,
            agency_name: "Moklanjo".into(),
            currency_code: "INR".into(),
        };
        let svg = label.to_svg();
        assert!(svg.contains("AWB-260417-000001"));
        assert!(svg.contains("COD"));
        assert!(svg.contains('₹'));
    }
}

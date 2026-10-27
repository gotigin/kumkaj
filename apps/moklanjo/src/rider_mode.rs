//! Rider app mode — same binary, offline-first stop list + POD queue.
//!
//! At the office on LAN the rider downloads today's run into a local cache.
//! On the road (no Wi‑Fi): mark OFD / Delivered / NDR, attach POD photo /
//! signature WebP, record COD collected. Writes go into `lan_outbox` and
//! sync when back on office Wi‑Fi. Never needs internet or Kumkaj Cloud.

use crate::domain::new_id;
use chrono::Local;

/// Desk vs rider shell role (same Android/desktop binary).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMode {
    Desk,
    Rider,
}

impl AppMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desk => "desk",
            Self::Rider => "rider",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "rider" => Self::Rider,
            _ => Self::Desk,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Desk => "Desk",
            Self::Rider => "Rider",
        }
    }
}

/// One stop on the rider's cached run (local, may be ahead of the Server).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RiderStop {
    pub sale_id: String,
    pub awb_no: String,
    pub destination: String,
    pub consignee_name: String,
    pub pincode: String,
    pub area: String,
    pub cod_paise: i64,
    pub weight_kg: String,
    pub status: String,
    pub stop_order: i64,
    pub cod_collected_paise: i64,
    pub ndr_reason: String,
    pub pod_attachment_id: String,
    pub signature_attachment_id: String,
    pub feedback_score: i64,
    pub lat: f64,
    pub lon: f64,
    pub pending_sync: bool,
}

/// Snapshot of today's run for one rider (downloaded at office).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RiderRunCache {
    pub rider_id: String,
    pub rider_name: String,
    pub manifest_id: String,
    pub downloaded_at: String,
    pub stops: Vec<RiderStop>,
}

impl RiderRunCache {
    pub fn waiting_count(&self) -> usize {
        self.stops.iter().filter(|s| s.pending_sync).count()
    }

    pub fn waiting_label(&self) -> String {
        let n = self.waiting_count();
        if n == 0 {
            "All synced with office".into()
        } else {
            format!("{n} waiting to reach office")
        }
    }
}

/// Offline POD / status event queued for LAN sync.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RiderPodEvent {
    pub id: String,
    pub sale_id: String,
    pub awb_no: String,
    pub status: String,
    pub note: String,
    pub ndr_reason: String,
    pub cod_collected_paise: i64,
    pub pod_attachment_id: String,
    pub signature_attachment_id: String,
    pub feedback_score: i64,
    /// Optional last-known position from the OS (0 = unset).
    pub lat: f64,
    pub lon: f64,
    pub created_at: String,
}

impl RiderPodEvent {
    pub fn new(sale_id: &str, awb: &str, status: &str) -> Self {
        Self {
            id: new_id(),
            sale_id: sale_id.into(),
            awb_no: awb.into(),
            status: status.into(),
            note: String::new(),
            ndr_reason: String::new(),
            cod_collected_paise: 0,
            pod_attachment_id: String::new(),
            signature_attachment_id: String::new(),
            feedback_score: 0,
            lat: 0.0,
            lon: 0.0,
            created_at: Local::now().to_rfc3339(),
        }
    }

    /// LAN outbox path the Server accepts.
    pub fn lan_path() -> &'static str {
        "/rider_pod"
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

/// One stroke point for signature capture (normalised 0..1 canvas coords).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SigPoint {
    pub x: f32,
    pub y: f32,
}

/// A continuous stroke (pen down → pen up).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SigStroke {
    pub points: Vec<SigPoint>,
}

/// Render signature strokes to a white PNG (then callers encode WebP).
/// Returns RGBA8 pixels + width/height.
pub fn signature_rgba(strokes: &[SigStroke], width: u32, height: u32) -> Vec<u8> {
    let w = width.max(64);
    let h = height.max(64);
    let mut px = vec![255u8; (w * h * 4) as usize];
    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }
        for win in stroke.points.windows(2) {
            let a = &win[0];
            let b = &win[1];
            draw_line(
                &mut px,
                w,
                h,
                (a.x.clamp(0.0, 1.0) * (w - 1) as f32) as i32,
                (a.y.clamp(0.0, 1.0) * (h - 1) as f32) as i32,
                (b.x.clamp(0.0, 1.0) * (w - 1) as f32) as i32,
                (b.y.clamp(0.0, 1.0) * (h - 1) as f32) as i32,
            );
        }
    }
    px
}

fn draw_line(px: &mut [u8], w: u32, h: u32, x0: i32, y0: i32, x1: i32, y1: i32) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        put_ink(px, w, h, x, y);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn put_ink(px: &mut [u8], w: u32, h: u32, x: i32, y: i32) {
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return;
    }
    // 3×3 brush for readable signature.
    for oy in -1i32..=1 {
        for ox in -1i32..=1 {
            let xx = x + ox;
            let yy = y + oy;
            if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 {
                continue;
            }
            let i = ((yy as u32 * w + xx as u32) * 4) as usize;
            px[i] = 17;
            px[i + 1] = 17;
            px[i + 2] = 17;
            px[i + 3] = 255;
        }
    }
}

/// Encode strokes as lossless WebP bytes (POD signature attachment).
pub fn signature_to_webp(strokes: &[SigStroke], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let w = width.max(64);
    let h = height.max(64);
    let rgba = signature_rgba(strokes, w, h);
    let img = image::RgbaImage::from_raw(w, h, rgba)
        .ok_or_else(|| "signature buffer".to_string())?;
    let dyn_img = image::DynamicImage::ImageRgba8(img);
    let mut out = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut out);
        dyn_img
            .write_to(&mut cursor, image::ImageFormat::WebP)
            .map_err(|e| format!("Encode signature WebP: {e}"))?;
    }
    Ok(out)
}

/// Sort stops by area then pincode (never locked — caller may reorder further).
pub fn sort_stops_by_area_pincode(stops: &mut [RiderStop]) {
    stops.sort_by(|a, b| {
        a.area
            .cmp(&b.area)
            .then_with(|| a.pincode.cmp(&b.pincode))
            .then_with(|| a.awb_no.cmp(&b.awb_no))
    });
    for (i, s) in stops.iter_mut().enumerate() {
        s.stop_order = i as i64;
    }
}

/// Move stop at `from` to `to` (editable anytime).
pub fn reorder_stop(stops: &mut [RiderStop], from: usize, to: usize) {
    if from >= stops.len() || to >= stops.len() || from == to {
        return;
    }
    let item = stops[from].clone();
    let mut v: Vec<RiderStop> = stops.to_vec();
    v.remove(from);
    v.insert(to, item);
    for (i, s) in v.iter_mut().enumerate() {
        s.stop_order = i as i64;
    }
    stops.clone_from_slice(&v);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_webp_non_empty() {
        let strokes = vec![SigStroke {
            points: vec![
                SigPoint { x: 0.1, y: 0.5 },
                SigPoint { x: 0.5, y: 0.2 },
                SigPoint { x: 0.9, y: 0.8 },
            ],
        }];
        let webp = signature_to_webp(&strokes, 320, 120).unwrap();
        assert!(webp.len() > 12);
        assert_eq!(&webp[0..4], b"RIFF");
        assert_eq!(&webp[8..12], b"WEBP");
    }

    #[test]
    fn waiting_label_counts_pending() {
        let mut cache = RiderRunCache {
            rider_id: "r1".into(),
            ..Default::default()
        };
        cache.stops.push(RiderStop {
            pending_sync: true,
            ..Default::default()
        });
        cache.stops.push(RiderStop {
            pending_sync: true,
            ..Default::default()
        });
        cache.stops.push(RiderStop {
            pending_sync: false,
            ..Default::default()
        });
        assert_eq!(cache.waiting_count(), 2);
        assert!(cache.waiting_label().contains("2 waiting"));
    }

    #[test]
    fn reorder_and_sort_stops() {
        let mut stops = vec![
            RiderStop {
                awb_no: "B".into(),
                area: "South".into(),
                pincode: "560002".into(),
                ..Default::default()
            },
            RiderStop {
                awb_no: "A".into(),
                area: "North".into(),
                pincode: "560001".into(),
                ..Default::default()
            },
        ];
        sort_stops_by_area_pincode(&mut stops);
        assert_eq!(stops[0].awb_no, "A");
        reorder_stop(&mut stops, 0, 1);
        assert_eq!(stops[0].awb_no, "B");
        assert_eq!(stops[1].stop_order, 1);
    }
}

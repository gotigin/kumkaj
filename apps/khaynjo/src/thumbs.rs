//! Small display copies of item and shelf photos, made off the UI thread.
//!
//! Lists used to decode every full-size photo synchronously inside
//! `draw_walk`. A shelf folder holds ~100 dishes at ~470×1024, and a page
//! list draws all of its rows, so opening the Menu screen decoded them all in
//! one frame — ~250 ms each in a dev build, the whole app frozen for tens of
//! seconds; item photos (stored full size) froze the Order desk the same way
//! (2026-10-23). Rows now ask for a [`THUMB_EDGE`] copy: if it is cached they
//! get its path, otherwise a background worker makes it, the row shows name
//! initials meanwhile, and the worker calls the notify hook so the shell
//! redraws once the copy exists.
//!
//! The cache key is the source path plus its size and mtime, so replacing an
//! item's photo (same `<product_id>.webp` file name) gets a fresh thumb.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};

use sha2::{Digest, Sha256};

/// Longest edge of a thumb, in pixels. Rows show 48–96 css px; 192 stays
/// sharp on 2× screens.
pub const THUMB_EDGE: u32 = 192;

/// A dish card in the menu window shows its photo as the whole card
/// (about 150×176 css px), so it gets a bigger copy already cropped to the
/// card's shape: sharp on 2× screens, and no wasted pixels to decode.
pub const CARD_W: u32 = 320;
pub const CARD_H: u32 = 368;

/// What a thumb is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThumbKind {
    /// List rows and avatars: longest edge [`THUMB_EDGE`], whole photo kept.
    Row,
    /// A dish card's background: [`CARD_W`]×[`CARD_H`], cropped to fill.
    Card,
}

/// Newest requests win: when the owner scrolls past rows, the rows now on
/// screen are made first, and stale requests beyond this many are dropped
/// (they are asked for again if their row comes back).
const QUEUE_CAP: usize = 48;

const WORKERS: usize = 2;

struct Queue {
    jobs: Vec<(PathBuf, PathBuf, ThumbKind)>,
    /// Thumbs queued or being made (by destination) — never queued twice.
    pending: HashSet<PathBuf>,
    /// Sources that could not be decoded — rows keep their initials.
    failed: HashSet<PathBuf>,
    started: bool,
}

struct Shared {
    queue: Mutex<Queue>,
    wake: Condvar,
    notify: Mutex<Option<fn()>>,
}

static READY: AtomicBool = AtomicBool::new(false);

fn shared() -> &'static Shared {
    static SHARED: OnceLock<Shared> = OnceLock::new();
    SHARED.get_or_init(|| Shared {
        queue: Mutex::new(Queue {
            jobs: Vec::new(),
            pending: HashSet::new(),
            failed: HashSet::new(),
            started: false,
        }),
        wake: Condvar::new(),
        notify: Mutex::new(None),
    })
}

/// Called (from a worker thread) whenever a new thumb lands. The shell sets
/// this to wake its UI loop.
pub fn set_notify(hook: fn()) {
    *shared().notify.lock().unwrap() = Some(hook);
}

/// True once since the last call if any thumb was made — the shell redraws.
pub fn take_ready() -> bool {
    READY.swap(false, Ordering::SeqCst)
}

/// Allow a source path to be tried again after a failed decode or a re-upload
/// that overwrote the same file (same path, new bytes).
pub fn forget_failed(src: &Path) {
    shared().queue.lock().unwrap().failed.remove(src);
}

/// Where the row thumb for `src` lives (made or not), or `None` when `src`
/// is not a readable file.
pub fn thumb_path(src: &Path, cache_dir: &Path) -> Option<PathBuf> {
    thumb_path_kind(src, cache_dir, ThumbKind::Row)
}

/// Where the `kind` thumb for `src` lives (made or not).
pub fn thumb_path_kind(src: &Path, cache_dir: &Path, kind: ThumbKind) -> Option<PathBuf> {
    let meta = std::fs::metadata(src).ok()?;
    if !meta.is_file() {
        return None;
    }
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut hash = Sha256::new();
    hash.update(src.to_string_lossy().as_bytes());
    hash.update(meta.len().to_le_bytes());
    hash.update(mtime.to_le_bytes());
    let key = hex::encode(&hash.finalize()[..16]);
    let name = match kind {
        ThumbKind::Row => format!("{key}-{THUMB_EDGE}.webp"),
        ThumbKind::Card => format!("{key}-card{CARD_W}x{CARD_H}.webp"),
    };
    Some(cache_dir.join(name))
}

/// The row thumb for `src` if it is already made; otherwise queue it and
/// return `None` (show initials until [`take_ready`] says to redraw). Never
/// decodes on the calling thread.
pub fn cached_or_request(src: &Path, cache_dir: &Path) -> Option<PathBuf> {
    cached_or_request_kind(src, cache_dir, ThumbKind::Row)
}

/// [`cached_or_request`] for a thumb of `kind`.
pub fn cached_or_request_kind(src: &Path, cache_dir: &Path, kind: ThumbKind) -> Option<PathBuf> {
    let dest = thumb_path_kind(src, cache_dir, kind)?;
    if dest.is_file() {
        return Some(dest);
    }
    let shared = shared();
    let mut q = shared.queue.lock().unwrap();
    if q.failed.contains(src) || q.pending.contains(&dest) {
        return None;
    }
    q.pending.insert(dest.clone());
    q.jobs.push((src.to_path_buf(), dest, kind));
    if q.jobs.len() > QUEUE_CAP {
        let excess = q.jobs.len() - QUEUE_CAP;
        let stale: Vec<_> = q.jobs.drain(..excess).collect();
        for (_, dest, _) in stale {
            q.pending.remove(&dest);
        }
    }
    if !q.started {
        q.started = true;
        for _ in 0..WORKERS {
            let _ = std::thread::Builder::new()
                .name("khaynjo-thumbs".into())
                .spawn(worker);
        }
    }
    drop(q);
    shared.wake.notify_one();
    None
}

fn worker() {
    let shared = shared();
    loop {
        let (src, dest, kind) = {
            let mut q = shared.queue.lock().unwrap();
            loop {
                if let Some(job) = q.jobs.pop() {
                    break job;
                }
                q = shared.wake.wait(q).unwrap();
            }
        };
        let made = make_thumb_kind(&src, &dest, kind);
        {
            let mut q = shared.queue.lock().unwrap();
            q.pending.remove(&dest);
            if made.is_err() {
                q.failed.insert(src);
            }
        }
        if made.is_ok() {
            READY.store(true, Ordering::SeqCst);
            if let Some(hook) = *shared.notify.lock().unwrap() {
                hook();
            }
        }
    }
}

/// Decode `src`, shrink it to fit [`THUMB_EDGE`], and write it to `dest`
/// (via a temp file, so a reader never sees half a thumb).
pub fn make_thumb(src: &Path, dest: &Path) -> Result<(), String> {
    make_thumb_kind(src, dest, ThumbKind::Row)
}

/// [`make_thumb`] for a thumb of `kind`: a card thumb is resized and
/// centre-cropped to exactly [`CARD_W`]×[`CARD_H`].
pub fn make_thumb_kind(src: &Path, dest: &Path, kind: ThumbKind) -> Result<(), String> {
    let bytes = std::fs::read(src).map_err(|e| format!("Read image: {e}"))?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("Decode image: {e}"))?;
    let small = match kind {
        ThumbKind::Row => img.thumbnail(THUMB_EDGE, THUMB_EDGE),
        ThumbKind::Card => img.resize_to_fill(CARD_W, CARD_H, image::imageops::FilterType::Triangle),
    };
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out = Vec::new();
    small
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::WebP)
        .map_err(|e| format!("Encode WebP: {e}"))?;
    let tmp = dest.with_extension(format!("tmp{}", std::process::id()));
    std::fs::write(&tmp, &out).map_err(|e| format!("Write thumb: {e}"))?;
    std::fs::rename(&tmp, dest).map_err(|e| format!("Write thumb: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("khaynjo-thumbs-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_png(path: &Path, w: u32, h: u32) {
        let img = image::RgbImage::from_pixel(w, h, image::Rgb([200, 40, 20]));
        img.save(path).unwrap();
    }

    #[test]
    fn thumb_fits_the_edge_and_keeps_aspect() {
        let dir = temp_dir("fit");
        let src = dir.join("dish.png");
        write_png(&src, 400, 800);
        let dest = thumb_path(&src, &dir).unwrap();
        make_thumb(&src, &dest).unwrap();
        let thumb = image::open(&dest).unwrap();
        assert_eq!((thumb.width(), thumb.height()), (THUMB_EDGE / 2, THUMB_EDGE));
        let _ = std::fs::remove_dir_all(dir);
    }

    /// A card thumb fills the card's shape exactly (cropped, not letterboxed)
    /// and is cached apart from the row thumb of the same photo.
    #[test]
    fn card_thumb_fills_the_card_shape() {
        let dir = temp_dir("card");
        let src = dir.join("dish.png");
        write_png(&src, 470, 1024);
        let dest = thumb_path_kind(&src, &dir, ThumbKind::Card).unwrap();
        assert_ne!(Some(dest.clone()), thumb_path(&src, &dir));
        make_thumb_kind(&src, &dest, ThumbKind::Card).unwrap();
        let thumb = image::open(&dest).unwrap();
        assert_eq!((thumb.width(), thumb.height()), (CARD_W, CARD_H));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn replaced_photo_gets_a_new_thumb_key() {
        let dir = temp_dir("key");
        let src = dir.join("item.png");
        write_png(&src, 10, 10);
        let first = thumb_path(&src, &dir).unwrap();
        // Same file name, different contents (an item's photo re-picked).
        write_png(&src, 20, 10);
        let second = thumb_path(&src, &dir).unwrap();
        assert_ne!(first, second);
        assert!(thumb_path(&dir.join("missing.png"), &dir).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn request_never_decodes_on_the_caller_and_lands_in_the_cache() {
        let dir = temp_dir("queue");
        let src = dir.join("dish.png");
        write_png(&src, 300, 300);
        assert!(cached_or_request(&src, &dir).is_none(), "first ask is queued");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        let made = loop {
            if let Some(path) = cached_or_request(&src, &dir) {
                break path;
            }
            assert!(std::time::Instant::now() < deadline, "worker never made the thumb");
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        assert!(made.is_file());
        let _ = std::fs::remove_dir_all(dir);
    }
}

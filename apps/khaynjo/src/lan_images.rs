//! Pull dish / category photos from the Server onto a Client (and onto any
//! device whose local `item_images` copy is missing).
//!
//! Sync only carries `image_key` strings — the WebP files stay on the Server.
//! Android never ships the ~130 MB shelf pack, so `shelf:…` keys resolve to
//! nothing there until this module asks the Server and caches the bytes.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};

use sha2::{Digest, Sha256};

use crate::net::{self, Link};
use crate::store::Store;

const QUEUE_CAP: usize = 64;
const WORKERS: usize = 2;

struct Queue {
    /// `(image_key, dest_path)`
    jobs: Vec<(String, PathBuf)>,
    pending: HashSet<String>,
    failed: HashSet<String>,
    started: bool,
    /// Pairing used by workers (refreshed when a job is queued).
    link: Option<Link>,
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
            link: None,
        }),
        wake: Condvar::new(),
        notify: Mutex::new(None),
    })
}

/// Wake the UI when a photo lands (same hook as thumbs).
pub fn set_notify(hook: fn()) {
    *shared().notify.lock().unwrap() = Some(hook);
}

/// True once since the last call if any LAN photo was written.
pub fn take_ready() -> bool {
    READY.swap(false, Ordering::SeqCst)
}

/// On-disk cache path for a remote `image_key` (under `item_images/lan/`).
pub fn cache_path(key: &str, item_images: &Path) -> PathBuf {
    let mut hash = Sha256::new();
    hash.update(key.as_bytes());
    let name = hex::encode(&hash.finalize()[..16]);
    item_images.join("lan").join(format!("{name}.webp"))
}

/// Queue a fetch when `key` is non-empty and the cache file is missing.
/// `link` is the Client's pairing; ignored when `None` (Server / unpaired).
pub fn request(key: &str, item_images: &Path, link: Option<&Link>) {
    let key = key.trim();
    if key.is_empty() {
        return;
    }
    let Some(link) = link else {
        return;
    };
    let dest = cache_path(key, item_images);
    if dest.is_file() {
        return;
    }
    let shared = shared();
    let mut q = shared.queue.lock().unwrap();
    if q.failed.contains(key) || q.pending.contains(key) {
        return;
    }
    q.link = Some(link.clone());
    q.pending.insert(key.to_string());
    q.jobs.push((key.to_string(), dest));
    if q.jobs.len() > QUEUE_CAP {
        let excess = q.jobs.len() - QUEUE_CAP;
        let stale: Vec<_> = q.jobs.drain(..excess).collect();
        for (k, _) in stale {
            q.pending.remove(&k);
        }
    }
    if !q.started {
        q.started = true;
        for _ in 0..WORKERS {
            let _ = std::thread::Builder::new()
                .name("khaynjo-lan-img".into())
                .spawn(worker);
        }
    }
    drop(q);
    shared.wake.notify_one();
}

/// After a sync: ask for every product/category photo the Client does not have.
pub fn request_missing_from_store(store: &Store, link: &Link) {
    let Ok(dir) = store.item_images_dir() else {
        return;
    };
    let products = store.list_products(true).unwrap_or_default();
    for p in products {
        if let Some(key) = p.image_key.as_deref() {
            ensure_or_request(key, &dir, Some(link));
        }
    }
    for c in store.list_categories().unwrap_or_default() {
        if !c.image_key.trim().is_empty() {
            ensure_or_request(&c.image_key, &dir, Some(link));
        }
    }
}

/// Resolve a local path for `key`: real file, LAN cache, or empty + queued fetch.
pub fn ensure_or_request(key: &str, item_images: &Path, link: Option<&Link>) -> PathBuf {
    let key = key.trim();
    if key.is_empty() {
        return PathBuf::new();
    }
    // Prefer the Server-side / already-materialised file.
    let primary = match key {
        k if k.starts_with("pending_custom:") => PathBuf::from(&k["pending_custom:".len()..]),
        k if k.starts_with("custom:") => item_images.join(&k["custom:".len()..]),
        other => crate::menu_images::resolve_image_path(Some(other), "", item_images),
    };
    if primary.is_file() {
        return primary;
    }
    let cached = cache_path(key, item_images);
    if cached.is_file() {
        return cached;
    }
    request(key, item_images, link);
    cached
}

fn worker() {
    let shared = shared();
    loop {
        let (key, dest, link) = {
            let mut q = shared.queue.lock().unwrap();
            loop {
                if let Some((key, dest)) = q.jobs.pop() {
                    let link = q.link.clone();
                    break (key, dest, link);
                }
                q = shared.wake.wait(q).unwrap();
            }
        };
        let ok = link
            .as_ref()
            .and_then(|l| net::client_fetch_item_image(l, &key).ok())
            .and_then(|bytes| {
                if bytes.is_empty() {
                    return None;
                }
                if let Some(parent) = dest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let tmp = dest.with_extension("webp.part");
                fs::write(&tmp, &bytes).ok()?;
                fs::rename(&tmp, &dest).ok()?;
                Some(())
            })
            .is_some();
        {
            let mut q = shared.queue.lock().unwrap();
            q.pending.remove(&key);
            if !ok {
                q.failed.insert(key);
            }
        }
        if ok {
            READY.store(true, Ordering::SeqCst);
            if let Some(hook) = *shared.notify.lock().unwrap() {
                hook();
            }
        }
    }
}

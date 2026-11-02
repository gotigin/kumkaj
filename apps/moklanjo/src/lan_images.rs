//! Pull rate photos and party logos from the Server onto a paired device.
//!
//! The sync snapshot carries `image_key`s only (`custom:<id>.webp`); the WebP
//! files stay in the Server's `item_images/` and `party_images/`. Without
//! this a paired counter showed initials for every rate and party that has
//! a photo. Copied from Khaynjo's `lan_images` (2026-11) and diverged: two
//! photo kinds, bare file names only on the wire (never a path), a failed
//! fetch is tried again later, and copies refresh so a changed logo reaches
//! every device.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use crate::net::{self, Link};
use crate::store::Store;

const QUEUE_CAP: usize = 64;
const WORKERS: usize = 2;
/// A photo the Server could not give is asked for again after this.
const RETRY_FAILED_AFTER: Duration = Duration::from_secs(300);
/// A device's copy older than this is fetched again at the next sync (the
/// key of a re-picked logo does not change — `custom:<party id>.webp`).
const REFRESH_AFTER: Duration = Duration::from_secs(6 * 3600);

/// Which of the Server's photo folders a key lives in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhotoKind {
    /// Rate / service photo (`item_images/`).
    Item,
    /// Party logo (`party_images/`).
    Party,
}

impl PhotoKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Item => "item",
            Self::Party => "party",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "item" => Some(Self::Item),
            "party" => Some(Self::Party),
            _ => None,
        }
    }

    /// The store's folder for this kind.
    pub fn dir(self, store: &Store) -> Result<PathBuf, String> {
        match self {
            Self::Item => store.item_images_dir(),
            Self::Party => store.party_images_dir(),
        }
    }
}

/// The file a `custom:` key names when it is a bare `.webp` file name —
/// never a path, so neither side can be walked out of a photo folder.
pub fn key_file(key: &str) -> Option<&str> {
    let file = key.trim().strip_prefix("custom:")?;
    let bare = !file.is_empty()
        && file.len() <= 128
        && !file.starts_with('.')
        && file.ends_with(".webp")
        && file
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    bare.then_some(file)
}

/// Where a device keeps its copy of a Server photo.
pub fn cache_path(dir: &Path, file: &str) -> PathBuf {
    dir.join("lan").join(file)
}

type JobKey = (PhotoKind, String);

struct Queue {
    /// `(kind, file, dest)`
    jobs: Vec<(PhotoKind, String, PathBuf)>,
    pending: HashSet<JobKey>,
    failed: HashMap<JobKey, Instant>,
    started: bool,
}

struct Shared {
    queue: Mutex<Queue>,
    wake: Condvar,
    notify: Mutex<Option<fn()>>,
    /// The pairing photos are fetched with (set after each sync).
    link: Mutex<Option<Link>>,
}

static READY: AtomicBool = AtomicBool::new(false);

fn shared() -> &'static Shared {
    static SHARED: OnceLock<Shared> = OnceLock::new();
    SHARED.get_or_init(|| Shared {
        queue: Mutex::new(Queue {
            jobs: Vec::new(),
            pending: HashSet::new(),
            failed: HashMap::new(),
            started: false,
        }),
        wake: Condvar::new(),
        notify: Mutex::new(None),
        link: Mutex::new(None),
    })
}

/// Wake the UI when a photo lands (same hook as thumbs).
pub fn set_notify(hook: fn()) {
    *shared().notify.lock().unwrap() = Some(hook);
}

/// True once since the last call if any photo copy was written.
pub fn take_ready() -> bool {
    READY.swap(false, Ordering::SeqCst)
}

/// The pairing in use, or `None` on a Server / an unpaired device.
fn current_link() -> Option<Link> {
    shared().link.lock().unwrap().clone()
}

/// After a sync on a paired device: remember the pairing and ask for every
/// rate photo and party logo this device has no fresh copy of.
pub fn request_missing_from_store(store: &Store, link: &Link) {
    *shared().link.lock().unwrap() = Some(link.clone());
    let mut wanted: Vec<(PhotoKind, String)> = Vec::new();
    for p in store.list_products(true).unwrap_or_default() {
        if let Some(key) = p.image_key {
            wanted.push((PhotoKind::Item, key));
        }
    }
    for c in store.list_customers().unwrap_or_default() {
        wanted.push((PhotoKind::Party, c.image_key));
    }
    for (kind, key) in wanted {
        let (Some(file), Ok(dir)) = (key_file(&key), kind.dir(store)) else {
            continue;
        };
        if dir.join(file).is_file() {
            continue;
        }
        let copy = cache_path(&dir, file);
        let fresh = fs::metadata(&copy)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .is_some_and(|age| age < REFRESH_AFTER);
        if !fresh {
            request(kind, file, copy);
        }
    }
}

/// Path to show for `key`: the file itself, else this device's copy of the
/// Server's, else empty (initials) while a fetch is queued.
pub fn ensure_or_request(kind: PhotoKind, key: &str, dir: &Path) -> PathBuf {
    let Some(file) = key_file(key) else {
        return PathBuf::new();
    };
    let own = dir.join(file);
    if own.is_file() {
        return own;
    }
    let copy = cache_path(dir, file);
    if copy.is_file() {
        return copy;
    }
    request(kind, file, copy);
    PathBuf::new()
}

fn request(kind: PhotoKind, file: &str, dest: PathBuf) {
    if current_link().is_none() {
        return;
    }
    let job: JobKey = (kind, file.to_string());
    let shared = shared();
    let mut q = shared.queue.lock().unwrap();
    if q.pending.contains(&job)
        || q.failed.get(&job).is_some_and(|at| at.elapsed() < RETRY_FAILED_AFTER)
    {
        return;
    }
    q.failed.remove(&job);
    q.pending.insert(job);
    q.jobs.push((kind, file.to_string(), dest));
    if q.jobs.len() > QUEUE_CAP {
        let excess = q.jobs.len() - QUEUE_CAP;
        let stale: Vec<_> = q.jobs.drain(..excess).collect();
        for (k, f, _) in stale {
            q.pending.remove(&(k, f));
        }
    }
    if !q.started {
        q.started = true;
        for _ in 0..WORKERS {
            let _ = std::thread::Builder::new()
                .name("moklanjo-lan-img".into())
                .spawn(worker);
        }
    }
    drop(q);
    shared.wake.notify_one();
}

fn worker() {
    let shared = shared();
    loop {
        let (kind, file, dest) = {
            let mut q = shared.queue.lock().unwrap();
            loop {
                if let Some(job) = q.jobs.pop() {
                    break job;
                }
                q = shared.wake.wait(q).unwrap();
            }
        };
        let ok = current_link()
            .and_then(|link| net::client_fetch_photo(&link, kind.as_str(), &file).ok())
            .filter(|bytes| !bytes.is_empty())
            .and_then(|bytes| {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).ok()?;
                }
                // Unchanged on the Server: keep the copy (and its age reset).
                if fs::read(&dest).is_ok_and(|have| have == bytes) {
                    let _ = fs::File::options()
                        .append(true)
                        .open(&dest)
                        .and_then(|f| f.set_modified(SystemTime::now()));
                    return Some(false);
                }
                let tmp = dest.with_extension("webp.part");
                fs::write(&tmp, &bytes).ok()?;
                fs::rename(&tmp, &dest).ok()?;
                Some(true)
            });
        {
            let mut q = shared.queue.lock().unwrap();
            let job = (kind, file);
            q.pending.remove(&job);
            if ok.is_none() {
                q.failed.insert(job, Instant::now());
            }
        }
        if ok == Some(true) {
            READY.store(true, Ordering::SeqCst);
            if let Some(hook) = *shared.notify.lock().unwrap() {
                hook();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only bare `.webp` names travel; anything that could name another
    /// file on the Server is refused before it is ever asked for.
    #[test]
    fn only_bare_webp_names_are_photo_files() {
        assert_eq!(key_file("custom:5f1c-2a.webp"), Some("5f1c-2a.webp"));
        assert_eq!(key_file(" custom:abc_1.webp "), Some("abc_1.webp"));
        for bad in [
            "",
            "abc.webp",
            "custom:",
            "custom:../moklanjo.sqlite3",
            "custom:../../x.webp",
            "custom:sub/x.webp",
            "custom:..\\x.webp",
            "custom:.hidden.webp",
            "custom:x.png",
            "custom:x y.webp",
            "pending_custom:/tmp/x.webp",
        ] {
            assert_eq!(key_file(bad), None, "{bad}");
        }
        assert_eq!(PhotoKind::parse("party"), Some(PhotoKind::Party));
        assert_eq!(PhotoKind::parse("item").map(PhotoKind::as_str), Some("item"));
        assert_eq!(PhotoKind::parse("../"), None);
    }
}

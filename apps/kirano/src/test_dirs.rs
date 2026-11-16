//! Where tests put their books, images and scratch folders: one folder per
//! test process inside the system temp folder, in place of
//! `std::env::temp_dir()`. Nothing a test leaves piles up there — a
//! panicking test skips its clean-up, and images, receipts and backups land
//! beside the books — because each run first removes the folders of runs
//! whose process has ended. Test code only: the app's own temporary files
//! (print jobs, uploads) stay in `std::env::temp_dir()`.
//! See docs/DEV.md § Tests leave nothing in the temp folder.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const PREFIX: &str = "kumkaj-test-run-";

/// This test process's folder, made on first use. Tests use it wherever
/// they would use `std::env::temp_dir()`.
pub fn run_root() -> PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let base = std::env::temp_dir();
        sweep_ended_runs(&base);
        let root = base.join(format!("{PREFIX}{}", std::process::id()));
        let _ = std::fs::create_dir_all(&root);
        root
    })
    .clone()
}

/// Removes `kumkaj-test-run-<pid>` folders whose process has ended.
fn sweep_ended_runs(base: &Path) {
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name
            .to_str()
            .and_then(|n| n.strip_prefix(PREFIX))
            .and_then(|p| p.parse::<u32>().ok())
        else {
            continue;
        };
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir && pid != std::process::id() && run_ended(pid, &entry.path()) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// Linux asks `/proc`. Elsewhere (or without `/proc`) a folder is taken as
/// ended once it has not changed for a day, so a run still going is never
/// swept from under itself.
fn run_ended(pid: u32, dir: &Path) -> bool {
    #[cfg(target_os = "linux")]
    if Path::new("/proc/self").exists() {
        return !Path::new(&format!("/proc/{pid}")).exists();
    }
    let _ = pid;
    std::fs::metadata(dir)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age > std::time::Duration::from_secs(24 * 60 * 60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_sweeps_folders_of_ended_runs_and_keeps_its_own_and_live_ones() {
        let base = run_root().join("sweep-check");
        std::fs::create_dir_all(&base).unwrap();
        // pid 0 is never a user process: an ended run.
        let ended = base.join(format!("{PREFIX}0"));
        std::fs::create_dir_all(ended.join("item_images")).unwrap();
        std::fs::write(ended.join("item_images/a.png"), b"x").unwrap();
        let live = base.join(format!("{PREFIX}{}", std::process::id()));
        std::fs::create_dir_all(&live).unwrap();
        let other = base.join("kumkaj-test-run-notapid");
        std::fs::create_dir_all(&other).unwrap();
        sweep_ended_runs(&base);
        #[cfg(target_os = "linux")]
        assert!(!ended.exists(), "an ended run's folder is removed");
        assert!(live.exists(), "this run's folder stays");
        assert!(other.exists(), "only <prefix><pid> folders are touched");
        let _ = std::fs::remove_dir_all(&base);
    }

    /// Test code that writes into the temp folder itself piles up there
    /// again (~100 GB in `/tmp` by 2026-11-16, until builds failed on the
    /// quota): every test path goes through [`run_root`] — and app code
    /// never does, so the app keeps its own temporary files where they were.
    #[test]
    fn test_code_never_writes_to_the_bare_temp_folder() {
        let bare = concat!("std::env::", "temp_dir()");
        let ours = concat!("test_dirs::", "run_root()");
        let app = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        rust_files(&app.join("src"), &mut files);
        rust_files(&app.join("tests"), &mut files);
        assert!(files.len() > 10, "scanned {} files", files.len());
        let (mut bare_in_tests, mut ours_in_app, mut ours_in_tests) = (Vec::new(), Vec::new(), 0);
        for file in &files {
            let rel = file.strip_prefix(app).unwrap_or(file);
            let all_test = rel
                .components()
                .any(|c| matches!(c.as_os_str().to_str(), Some("shell_tests" | "tests")));
            let text = std::fs::read_to_string(file).unwrap_or_default();
            for (line, in_test) in code_hits(&text, all_test, bare) {
                if in_test {
                    bare_in_tests.push(format!("{}:{line}", rel.display()));
                }
            }
            for (line, in_test) in code_hits(&text, all_test, ours) {
                if in_test {
                    ours_in_tests += 1;
                } else {
                    ours_in_app.push(format!("{}:{line}", rel.display()));
                }
            }
        }
        assert!(bare_in_tests.is_empty(), "use test_dirs::run_root() in test code: {bare_in_tests:?}");
        assert!(ours_in_app.is_empty(), "app code in the test folder: {ours_in_app:?}");
        // The scan sees test code at all: the folder is used from it.
        assert!(ours_in_tests > 20, "only {ours_in_tests} test uses of run_root found");
    }

    #[test]
    fn the_scan_tells_test_code_from_app_code() {
        let needle = concat!("std::env::", "temp_dir()");
        let src = r#####"
fn app() { let _ = std::env::temp_dir(); }
#[cfg(test)]
mod tests {
    fn a() { let s = "}"; let c = '}'; let q = '\''; let u = '\u{7d}'; let r = r#"}"#; /* } */ let _ = std::env::temp_dir(); }
    // std::env::temp_dir() in a comment
    fn b<'a>(x: &'a str) { let _ = "std::env::temp_dir()"; }
}
fn after() { let _ = std::env::temp_dir(); }
#[cfg(all(test, target_os = "linux"))]
fn linux_check() { let _ = std::env::temp_dir(); }
#[cfg(any(test, target_os = "linux"))]
fn shared() { let _ = std::env::temp_dir(); }
"#####;
        let lines = [2, 5, 9, 11, 13];
        let in_test = [false, true, false, true, false];
        let want: Vec<_> = lines.into_iter().zip(in_test).collect();
        assert_eq!(code_hits(src, false, needle), want);
        let whole: Vec<_> = lines.into_iter().map(|l| (l, true)).collect();
        assert_eq!(code_hits(src, true, needle), whole);
    }

    fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// 1-based lines where `needle` is in code (not a string or comment),
    /// each with whether it is test code: inside a `#[cfg(test)]` /
    /// `#[cfg(all(test, …))]` item, or anywhere when the whole file is.
    fn code_hits(text: &str, all_test: bool, needle: &str) -> Vec<(usize, bool)> {
        #[derive(Clone, Copy, PartialEq)]
        enum St {
            Code,
            Str,
            Raw(usize),
            Block(usize),
        }
        let ident = |c: char| c.is_alphanumeric() || c == '_';
        let (mut st, mut depth, mut test_depth, mut pending) = (St::Code, 0usize, None, false);
        let needle: Vec<char> = needle.chars().collect();
        let mut hits = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let t = line.trim_start();
            let all_terms = t.strip_prefix("#[cfg(all(").map(|rest| {
                rest.split(')').next().unwrap_or("").split(',').any(|p| p.trim() == "test")
            });
            if st == St::Code && (t.starts_with("#[cfg(test)") || all_terms == Some(true)) {
                pending = true;
            }
            let ch: Vec<char> = line.chars().collect();
            // The line's code chars, each with whether it sits in test code
            // (a one-line test item opens and closes on the same line).
            let (mut code, mut in_test) = (Vec::new(), Vec::new());
            let mut i = 0;
            while i < ch.len() {
                let c = ch[i];
                match st {
                    St::Str => {
                        if c == '\\' {
                            i += 1;
                        } else if c == '"' {
                            st = St::Code;
                        }
                    }
                    St::Raw(h) => {
                        if c == '"' && ch[i + 1..].iter().take_while(|&&x| x == '#').count() >= h {
                            st = St::Code;
                            i += h;
                        }
                    }
                    St::Block(d) => {
                        if c == '*' && ch.get(i + 1) == Some(&'/') {
                            st = if d == 1 { St::Code } else { St::Block(d - 1) };
                            i += 1;
                        } else if c == '/' && ch.get(i + 1) == Some(&'*') {
                            st = St::Block(d + 1);
                            i += 1;
                        }
                    }
                    St::Code => {
                        let prev = i.checked_sub(1).map(|p| ch[p]);
                        let raw_start = c == 'r'
                            && (prev.is_none_or(|p| !ident(p))
                                || (prev == Some('b') && (i < 2 || !ident(ch[i - 2]))));
                        let hashes = ch[i + 1..].iter().take_while(|&&x| x == '#').count();
                        if c == '/' && ch.get(i + 1) == Some(&'/') {
                            break;
                        } else if c == '/' && ch.get(i + 1) == Some(&'*') {
                            st = St::Block(1);
                            i += 1;
                        } else if c == '"' {
                            st = St::Str;
                        } else if raw_start && ch.get(i + 1 + hashes) == Some(&'"') {
                            st = St::Raw(hashes);
                            i += 1 + hashes;
                        } else if c == '\'' && ch.get(i + 1) == Some(&'\\') {
                            // '\'' '\n' '\u{7b}': skip the escaped char, then to the close.
                            i += 3;
                            while i < ch.len() && ch[i] != '\'' {
                                i += 1;
                            }
                        } else if c == '\'' && ch.get(i + 2) == Some(&'\'') {
                            i += 2;
                        } else {
                            code.push(c);
                            in_test.push(all_test || test_depth.is_some());
                            if c == '{' {
                                if pending && test_depth.is_none() {
                                    test_depth = Some(depth);
                                    pending = false;
                                }
                                depth += 1;
                            } else if c == '}' {
                                depth = depth.saturating_sub(1);
                                if test_depth == Some(depth) {
                                    test_depth = None;
                                }
                            } else if c == ';' && test_depth.is_none() {
                                pending = false;
                            }
                        }
                    }
                }
                i += 1;
            }
            let first = code
                .windows(needle.len())
                .position(|w| w == needle.as_slice());
            if let Some(k) = first {
                hits.push((n + 1, in_test[k]));
            }
        }
        hits
    }
}

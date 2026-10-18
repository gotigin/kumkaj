//! Receipt share / print helpers (local free POS — no cloud).
//!
//! Desktop hands the file to the OS (print dialog, else default opener).
//! Android writes the same file and the caller opens the share sheet with
//! `Cx::share_text` (`ACTION_SEND`). The clipboard path is still
//! `Cx::copy_to_clipboard`.

use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "android"))]
use std::process::Command;

/// What happened to a receipt: always written, sometimes handed to the OS.
pub struct ReceiptShare {
    pub path: PathBuf,
    /// True when the OS took over (print dialog / default app). False on
    /// Android, where the caller falls back to the clipboard.
    pub opened: bool,
}

/// Where receipts land. `app_files` is Makepad's data dir (`Cx::get_data_dir`,
/// `Activity.getFilesDir` on Android) — the temp dir is not somewhere an
/// Android file manager will look.
fn receipt_dir(app_files: Option<&Path>) -> PathBuf {
    match app_files {
        Some(base) => base.join("receipts"),
        None => std::env::temp_dir().join("kumkaj-receipts"),
    }
}

/// Write receipt text next to the shop's data and return the file.
pub fn write_receipt(text: &str, app_files: Option<&Path>) -> Result<PathBuf, String> {
    let dir = receipt_dir(app_files);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!(
        "receipt-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    ));
    fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Write the receipt, then ask the OS to print / share it where that is possible.
pub fn open_receipt_for_print(
    text: &str,
    app_files: Option<&Path>,
) -> Result<ReceiptShare, String> {
    let path = write_receipt(text, app_files)?;
    let opened = open_path(&path)?;
    Ok(ReceiptShare { path, opened })
}

/// True when the OS accepted the file. Android always returns false — there is
/// nothing to hand it to from here.
fn open_path(path: &Path) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        // Prefer system print dialog when available.
        if Command::new("lp").arg(path).status().map(|s| s.success()).unwrap_or(false) {
            return Ok(true);
        }
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("open receipt: {e}"))?;
        Ok(true)
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("open receipt: {e}"))?;
        return Ok(true);
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()
            .map_err(|e| format!("open receipt: {e}"))?;
        return Ok(true);
    }
    #[cfg(target_os = "android")]
    {
        // The share sheet is `Cx::share_text`, not a file open. The file is
        // still written so the receipt exists after the chooser closes.
        let _ = path;
        return Ok(false);
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "android"
    )))]
    {
        let _ = path;
        Ok(false)
    }
}

/// Desktop clipboard. Android goes through `Cx::copy_to_clipboard`, which is
/// wired to `ClipboardManager` in Makepad's activity, so callers that hold a
/// `Cx` should prefer that on every platform.
pub fn copy_text(text: &str) -> bool {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        if let Ok(mut clip) = arboard::Clipboard::new() {
            return clip.set_text(text.to_string()).is_ok();
        }
    }
    let _ = text;
    false
}

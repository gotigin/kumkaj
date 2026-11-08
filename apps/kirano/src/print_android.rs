//! Android print bridges: Bluetooth SPP, USB host, PrintManager.
//! JNI methods live on MakepadActivity (patches/makepad/MakepadActivity-print.java).

use makepad_widgets::makepad_platform::os::linux::android::android_jni;

/// Paired Bluetooth devices as `bt:MAC` lines.
pub fn list_bluetooth() -> Vec<String> {
    unsafe { android_jni::to_java_list_bluetooth_printers() }
}

/// Write ESC/POS bytes to a paired SPP printer.
pub fn write_bluetooth(mac: &str, bytes: &[u8]) -> Result<(), String> {
    unsafe {
        if android_jni::to_java_write_bluetooth_spp(mac.to_owned(), bytes) {
            Ok(())
        } else {
            Err(format!(
                "Bluetooth write failed for {mac} — pair the printer and allow Bluetooth"
            ))
        }
    }
}

/// Attached USB printer candidates as `usb:VID:PID` lines.
pub fn list_usb() -> Vec<String> {
    unsafe { android_jni::to_java_list_usb_printers() }
}

pub fn write_usb(vid: u16, pid: u16, serial: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    let key = match serial {
        Some(s) if !s.is_empty() => format!("{vid:04x}:{pid:04x}:{s}"),
        _ => format!("{vid:04x}:{pid:04x}"),
    };
    unsafe {
        if android_jni::to_java_write_usb_printer(key.clone(), bytes) {
            Ok(())
        } else {
            Err(format!(
                "USB write failed for usb:{key} — plug in with OTG and grant USB permission"
            ))
        }
    }
}

/// Standard slip via Android PrintManager (system print UI).
/// Prefer [`print_png`] — plain text depends on the device font.
pub fn print_document(job_name: &str, text: &str) -> Result<(), String> {
    unsafe {
        if android_jni::to_java_print_document(job_name.to_owned(), text.to_owned()) {
            Ok(())
        } else {
            Err("Android PrintManager refused the job".into())
        }
    }
}

/// Standard bill as a Noto-raster PNG (same glyphs as Linux/Windows).
/// `landscape` selects A4 landscape vs portrait in PrintManager.
pub fn print_png(job_name: &str, png: &[u8], landscape: bool) -> Result<(), String> {
    unsafe {
        if android_jni::to_java_print_png(job_name.to_owned(), png, landscape) {
            Ok(())
        } else {
            Err("Android PrintManager refused the image job".into())
        }
    }
}

/// Android print-service style targets for Find.
pub fn list_print_services() -> Vec<String> {
    vec!["android-print:".into()]
}

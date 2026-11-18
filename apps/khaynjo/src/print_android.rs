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

/// OS document: the upright slip (the preview's picture) and its width on
/// paper. The system print dialog picks the printer and its paper; each
/// page draws the slip at its real size on the part of that paper the
/// printer reaches and the virtual paper covers, at the format's place
/// (docs/PRINTER.md § Paper and preview; KumkajSlipLayout.java). `lay`
/// turns the sheet and gives the place; `virtual_paper` is the paper the
/// owner says the printer holds (`None`: the dialog's paper is all paper).
pub fn print_slip(
    job_name: &str,
    png: &[u8],
    content_mm: f64,
    lay: crate::os_paper::Lay,
    virtual_paper: Option<crate::os_paper::PaperMm>,
) -> Result<(), String> {
    let um = |mm: f64| (mm * 1000.0).round().clamp(0.0, f64::from(i32::MAX)) as i32;
    let content_um = um(content_mm).max(1);
    // The place as MakepadActivity-print.java reads it: down × 3 + across.
    let place = i32::from(lay.place.down.min(2)) * 3 + i32::from(lay.place.across.min(2));
    // A length of 0 is a roll; a width of 0 no virtual paper.
    let (virtual_width_um, virtual_length_um) =
        virtual_paper.map_or((0, 0), |p| (um(p.width_mm), um(p.length_mm)));
    unsafe {
        if android_jni::to_java_print_slip_placed(
            job_name.to_owned(),
            png,
            content_um,
            lay.landscape,
            place,
            virtual_width_um,
            virtual_length_um,
        ) {
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

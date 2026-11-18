//! The Windows print API, called directly — no helper process (owner,
//! 2026-11-17: every print to a Windows printer, every paper read and every
//! Find used to start a hidden PowerShell). Four things: which printers
//! this computer has ([`printers`]), what a printer's driver is set to
//! ([`papers`]: the page a job prints on, with the edges the printer cannot
//! reach), printing an OS document's pages on that page ([`print_pages`]),
//! and sending a job's bytes untouched ([`print_raw`], ESC/POS).
//!
//! An OS document goes the standard GDI way, as any Windows app prints:
//! the printer's own `DEVMODE` as its driver is set (`DocumentProperties`),
//! a device context for it (`CreateDC`), then `StartDoc` / `StartPage` /
//! the page's bitmap / `EndPage` / `EndDoc`. No paper size is ever set: the
//! page is the driver's. Only the orientation is — portrait, always: a
//! landscape slip is turned by the app before it is drawn
//! (`os_paper::render_sheets`), so it lands the same on every driver (each
//! driver turns landscape its own way, a quarter or three).
//!
//! Raw bytes go the standard spooler way: `OpenPrinter`, `StartDocPrinter`
//! with the "RAW" data type, `WritePrinter`, `EndDocPrinter` — the spooler
//! and the driver pass such a job on as it is.
//!
//! USB and Bluetooth printers are found through the device list Windows
//! itself keeps (SetupAPI): a USB printer is a device its printer-class
//! driver runs ([`usb_printers`]; its bytes go through Windows' queue for
//! that device, or straight to the device when it has none), a Bluetooth
//! printer a paired device whose Serial Port service Windows shows as a
//! COM port ([`bluetooth_devices`], [`bluetooth_port`], [`serial_write`]).
//! Always the very device that was picked: one that is missing is an
//! error, never a reason to print on another.

use crate::os_paper::Paper;

/// What a printer's device context says of its page, in device pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageCaps {
    /// Dots per inch, across and down.
    pub dpi: (i32, i32),
    /// The whole sheet.
    pub physical: (i32, i32),
    /// The printable area's top-left, from the sheet's corner.
    pub offset: (i32, i32),
    /// The printable area.
    pub printable: (i32, i32),
}

impl PageCaps {
    /// The page in mm with the edges the printer cannot reach; `None` when
    /// the driver reports no size or resolution.
    pub fn paper(&self, name: &str) -> Option<Paper> {
        let (dx, dy) = self.dpi;
        let (pw, ph) = self.physical;
        if dx <= 0 || dy <= 0 || pw <= 0 || ph <= 0 {
            return None;
        }
        let mm = |px: i32, dpi: i32| f64::from(px.max(0)) * 25.4 / f64::from(dpi);
        let (ox, oy) = self.offset;
        let (hr, vr) = self.printable;
        Some(Paper {
            id: name.to_string(),
            label: name.to_string(),
            width_mm: mm(pw, dx),
            height_mm: mm(ph, dy),
            margins_mm: [mm(ox, dx), mm(oy, dy), mm(pw - ox - hr, dx), mm(ph - oy - vr, dy)],
        })
    }
}

/// A page picture on the printer's own dot grid, as a Windows bitmap: one
/// bit per pixel (1 is white), the bottom row first, each row padded to 32
/// bits.
#[derive(Clone, Debug, PartialEq)]
pub struct DevicePicture {
    pub bits: Vec<u8>,
    pub width: i32,
    pub height: i32,
    /// The device pixels it covers: `width` × `height` times a whole number.
    pub dest: (i32, i32),
}

/// One picture pixel is drawn over this many device dots, each way: 1 up to
/// 600 dots per inch; a finer printer gets the 600-class picture stretched
/// by a whole number, so a page stays small in the spooler and no dot is
/// ever halved.
fn stretch_for(dpi: (i32, i32)) -> i32 {
    ((dpi.0.max(dpi.1) + 599) / 600).max(1)
}

/// `page` (grey; 255 is white; `px_per_mm` its density) drawn at its real
/// size on a printer with `dpi` dots per inch. The app lays the picture on
/// the printer's own dot grid itself, so no driver scales or smooths the
/// slip its own way: every driver gets plain black and white dots.
pub fn device_picture(page: &image::GrayImage, px_per_mm: f64, dpi: (i32, i32)) -> DevicePicture {
    let k = stretch_for(dpi);
    let px_per_mm = px_per_mm.max(0.01);
    let dots = |px: u32, dpi: i32| {
        ((f64::from(px) / px_per_mm * f64::from(dpi.max(1)) / 25.4 / f64::from(k)).round() as i32).max(1)
    };
    let (w, h) = (dots(page.width(), dpi.0), dots(page.height(), dpi.1));
    let fitted;
    let grid = if (w as u32, h as u32) == page.dimensions() {
        page
    } else {
        fitted = image::imageops::resize(page, w as u32, h as u32, image::imageops::FilterType::Triangle);
        &fitted
    };
    let (wu, hu) = (w as usize, h as usize);
    let stride = wu.div_ceil(32) * 4;
    let mut bits = vec![0u8; stride * hu];
    let raw = grid.as_raw();
    for y in 0..hu {
        let row = &raw[y * wu..(y + 1) * wu];
        let out = &mut bits[(hu - 1 - y) * stride..(hu - y) * stride];
        for (x, &v) in row.iter().enumerate() {
            if v >= 128 {
                out[x / 8] |= 0x80 >> (x % 8);
            }
        }
    }
    DevicePicture { bits, width: w, height: h, dest: (w * k, h * k) }
}

/// The vendor id, product id and serial number in a USB device's instance
/// id (`USB\VID_04B8&PID_0E15\X6HN012345`). The last part is the serial
/// number only when the device has one: an id Windows made up itself
/// (`6&2C2B3C4D&0&2`) has `&` in it and is not a serial.
pub fn usb_ids(instance_id: &str) -> Option<(u16, u16, Option<String>)> {
    let mut parts = instance_id.split('\\');
    if !parts.next()?.eq_ignore_ascii_case("USB") {
        return None;
    }
    let ids = parts.next()?.to_ascii_uppercase();
    let hex_after = |tag: &str| {
        let at = ids.find(tag)? + tag.len();
        u16::from_str_radix(ids.get(at..at + 4)?, 16).ok()
    };
    let (vid, pid) = (hex_after("VID_")?, hex_after("PID_")?);
    let serial = parts.next().map(str::trim).filter(|s| !s.is_empty() && !s.contains('&')).map(str::to_string);
    Some((vid, pid, serial))
}

/// The address of a paired Bluetooth device from its instance id
/// (`BTHENUM\DEV_AABBCCDDEEFF\…`), as `AA:BB:CC:DD:EE:FF`. Only a device's
/// own entry names it so: a service's entry (`BTHENUM\{00001101-…}…`) is
/// not a device, and the hex digits in its service id are not an address.
pub fn bluetooth_address_of(instance_id: &str) -> Option<String> {
    let mut parts = instance_id.split('\\');
    if !parts.next()?.eq_ignore_ascii_case("BTHENUM") {
        return None;
    }
    let dev = parts.next()?.to_ascii_uppercase();
    let hex = dev.strip_prefix("DEV_")?;
    if hex.len() != 12 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(hex.as_bytes().chunks(2).map(|c| String::from_utf8_lossy(c).into_owned()).collect::<Vec<_>>().join(":"))
}

/// The COM port of the Bluetooth device `mac` among `devices` (each one's
/// instance id and, for a serial port, its port's name): the port whose
/// device id carries that address. `None` when it has none — never
/// another device's port.
pub fn bluetooth_port_among(devices: &[(String, Option<String>)], mac: &str) -> Option<String> {
    let hex: String = mac.chars().filter(char::is_ascii_hexdigit).collect::<String>().to_ascii_uppercase();
    if hex.len() != 12 {
        return None;
    }
    devices.iter().find_map(|(id, port)| {
        let port = port.as_deref().map(str::trim).filter(|p| !p.is_empty())?;
        id.to_ascii_uppercase().contains(&hex).then(|| port.to_string())
    })
}

#[cfg(target_os = "windows")]
pub use sys::{
    bluetooth_devices, bluetooth_port, papers, present_devices, print_pages, print_raw, printers, serial_write,
    usb_printers, usb_write, UsbPrinter,
};
#[cfg(all(target_os = "windows", test))]
pub use sys::{jobs_waiting, raw_job_as_spooled};

#[cfg(target_os = "windows")]
mod sys {
    use super::{device_picture, PageCaps};
    use crate::os_paper::{Paper, PrinterPapers};
    use std::path::Path;
    use windows_sys::core::GUID;
    use windows_sys::Win32::Devices::Communication::{
        ClearCommError, GetCommState, SetCommState, SetCommTimeouts, COMMTIMEOUTS, COMSTAT, DCB,
    };
    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
        SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceInterfaceDetailW, SetupDiOpenDevRegKey,
        SetupDiOpenDeviceInterfaceRegKey, DICS_FLAG_GLOBAL, DIGCF_ALLCLASSES, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT,
        DIREG_DEV, HDEVINFO, SP_DEVICE_INTERFACE_DATA, SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA,
    };
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Storage::FileSystem::{CreateFileW, WriteFile, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING};
    use windows_sys::Win32::System::Registry::{RegCloseKey, RegQueryValueExW, HKEY, KEY_READ, REG_DWORD, REG_SZ};
    use windows_sys::Win32::Graphics::Gdi::{
        CreateDCW, CreateICW, DeleteDC, GetDeviceCaps, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DEVMODEW, DIB_RGB_COLORS, DMORIENT_PORTRAIT, DM_FORMNAME, DM_IN_BUFFER, DM_ORIENTATION, DM_OUT_BUFFER,
        HDC, HORZRES, LOGPIXELSX, LOGPIXELSY, PHYSICALHEIGHT, PHYSICALOFFSETX, PHYSICALOFFSETY, PHYSICALWIDTH,
        RGBQUAD, SRCCOPY, VERTRES,
    };
    use windows_sys::Win32::Graphics::Printing::{
        AbortPrinter, ClosePrinter, DocumentPropertiesW, EndDocPrinter, EndPagePrinter, EnumPrintersW,
        GetDefaultPrinterW, OpenPrinterW, StartDocPrinterW, StartPagePrinter, WritePrinter, DOC_INFO_1W,
        PRINTER_ATTRIBUTE_WORK_OFFLINE, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL, PRINTER_HANDLE,
        PRINTER_INFO_2W, PRINTER_STATUS_OFFLINE,
    };
    use windows_sys::Win32::Storage::Xps::{
        AbortDoc, DeviceCapabilitiesW, EndDoc, EndPage, StartDocW, StartPage, DC_PAPERNAMES, DC_PAPERS,
        DC_PAPERSIZE, DOCINFOW,
    };

    /// `DocumentProperties` answers this when it filled the buffer.
    const IDOK: i32 = 1;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn from_wide(w: &[u16]) -> String {
        let end = w.iter().position(|&c| c == 0).unwrap_or(w.len());
        String::from_utf16_lossy(&w[..end]).trim().to_string()
    }

    /// The printer a job with no name goes to.
    fn default_printer() -> Result<String, String> {
        let mut len = 0u32;
        // The first call only says how long the name is.
        unsafe { GetDefaultPrinterW(std::ptr::null_mut(), &mut len) };
        if len == 0 {
            return Err("this computer has no default printer".into());
        }
        let mut buf = vec![0u16; len as usize];
        if unsafe { GetDefaultPrinterW(buf.as_mut_ptr(), &mut len) } == 0 {
            return Err("this computer has no default printer".into());
        }
        Ok(from_wide(&buf))
    }

    fn printer_name(printer: Option<&str>) -> Result<String, String> {
        match printer.map(str::trim).filter(|n| !n.is_empty()) {
            Some(n) => Ok(n.to_string()),
            None => default_printer(),
        }
    }

    /// The text Windows gave at `p`, up to its closing zero (`None` for no
    /// text at all).
    ///
    /// # Safety
    /// `p` is null or points at a zero-ended UTF-16 string.
    unsafe fn text_at(p: *const u16) -> Option<String> {
        if p.is_null() {
            return None;
        }
        let mut len = 0usize;
        // The caller's promise: every unit up to the closing zero is there.
        unsafe {
            while *p.add(len) != 0 {
                len += 1;
            }
            Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, len)))
        }
    }

    /// An open printer; closed when dropped.
    struct Printer(PRINTER_HANDLE);

    impl Drop for Printer {
        fn drop(&mut self) {
            unsafe { ClosePrinter(self.0) };
        }
    }

    impl Printer {
        fn open(name: &str) -> Result<Self, String> {
            let wname = wide(name);
            let mut handle = PRINTER_HANDLE::default();
            if unsafe { OpenPrinterW(wname.as_ptr(), &mut handle, std::ptr::null()) } == 0 {
                return Err(format!("Windows does not know the printer \"{name}\""));
            }
            Ok(Self(handle))
        }
    }

    /// One printer in Windows' own list (`EnumPrinters`).
    struct Queue {
        name: String,
        /// The port, or ports, it prints to (`USB001`, `LPT1:,COM3:`).
        ports: String,
        /// Set to work offline, or reported offline: a removed or unplugged
        /// ghost.
        offline: bool,
    }

    /// The printers this computer has now: its own and the shared ones it
    /// is connected to.
    fn queues() -> Vec<Queue> {
        let which = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
        // Whole `u64`s: the list starts with structures that hold pointers.
        let mut room: Vec<u64> = Vec::new();
        // A printer added between the two calls needs one more round.
        for _ in 0..4 {
            let (mut need, mut count) = (0u32, 0u32);
            let size = (room.len() * 8) as u32;
            let filled = unsafe {
                EnumPrintersW(which, std::ptr::null(), 2, room.as_mut_ptr().cast(), size, &mut need, &mut count)
            };
            if filled != 0 {
                // Filled by Windows with `count` structures, aligned as `u64`.
                let list = unsafe { std::slice::from_raw_parts(room.as_ptr().cast::<PRINTER_INFO_2W>(), count as usize) };
                return list
                    .iter()
                    .filter_map(|p| {
                        Some(Queue {
                            name: unsafe { text_at(p.pPrinterName) }.filter(|name| !name.trim().is_empty())?,
                            ports: unsafe { text_at(p.pPortName) }.unwrap_or_default(),
                            offline: p.Attributes & PRINTER_ATTRIBUTE_WORK_OFFLINE != 0
                                || p.Status & PRINTER_STATUS_OFFLINE != 0,
                        })
                    })
                    .collect();
            }
            if need <= size {
                break;
            }
            room = vec![0u64; (need as usize).div_ceil(8)];
        }
        Vec::new()
    }

    /// The printers this computer has now, by name (`EnumPrinters`). One set
    /// to work offline, or reported offline, is left out — a removed or
    /// unplugged ghost.
    pub fn printers() -> Vec<String> {
        queues().into_iter().filter(|q| !q.offline).map(|q| q.name).collect()
    }

    // ---- USB and Bluetooth printers: Windows' own device list ----------

    /// A set of devices from SetupAPI; given back when dropped.
    struct DeviceSet(HDEVINFO);

    impl Drop for DeviceSet {
        fn drop(&mut self) {
            unsafe { SetupDiDestroyDeviceInfoList(self.0) };
        }
    }

    impl DeviceSet {
        /// The devices present now: of one bus driver (`enumerator`, any
        /// class), or showing one device interface (`interface`).
        fn present(enumerator: Option<&str>, interface: Option<&GUID>) -> Option<Self> {
            let name = enumerator.map(wide);
            let mut flags = DIGCF_PRESENT;
            flags |= if interface.is_some() { DIGCF_DEVICEINTERFACE } else { DIGCF_ALLCLASSES };
            let set = unsafe {
                SetupDiGetClassDevsW(
                    interface.map_or(std::ptr::null(), |g| g as *const GUID),
                    name.as_ref().map_or(std::ptr::null(), |n| n.as_ptr()),
                    std::ptr::null_mut(),
                    flags,
                )
            };
            // `INVALID_HANDLE_VALUE` for a set that could not be made.
            (set != -1).then_some(Self(set))
        }

        fn instance_id(&self, device: &SP_DEVINFO_DATA) -> Option<String> {
            let mut need = 0u32;
            unsafe { SetupDiGetDeviceInstanceIdW(self.0, device, std::ptr::null_mut(), 0, &mut need) };
            let mut id = vec![0u16; need.max(1) as usize];
            let ok = unsafe { SetupDiGetDeviceInstanceIdW(self.0, device, id.as_mut_ptr(), id.len() as u32, &mut need) };
            (ok != 0).then(|| from_wide(&id))
        }
    }

    /// A registry key Windows opened; closed when dropped.
    struct Key(HKEY);

    impl Drop for Key {
        fn drop(&mut self) {
            unsafe { RegCloseKey(self.0) };
        }
    }

    impl Key {
        /// `None` for the "no key" answer of the SetupAPI calls.
        fn opened(key: HKEY) -> Option<Self> {
            (!key.is_null() && key != INVALID_HANDLE_VALUE).then_some(Self(key))
        }

        /// A value's data when it has the kind `want` and fits `room`.
        fn read(&self, name: &str, want: u32, room: &mut [u8]) -> Option<usize> {
            let wname = wide(name);
            let (mut kind, mut size) = (0u32, room.len() as u32);
            let status = unsafe {
                RegQueryValueExW(self.0, wname.as_ptr(), std::ptr::null(), &mut kind, room.as_mut_ptr(), &mut size)
            };
            (status == 0 && kind == want).then_some(size as usize)
        }

        fn text(&self, name: &str) -> Option<String> {
            let mut room = [0u8; 520];
            let size = self.read(name, REG_SZ, &mut room)?;
            let units: Vec<u16> = room[..size].chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect();
            Some(from_wide(&units)).filter(|s| !s.is_empty())
        }

        fn number(&self, name: &str) -> Option<u32> {
            let mut room = [0u8; 4];
            (self.read(name, REG_DWORD, &mut room)? == 4).then(|| u32::from_le_bytes(room))
        }
    }

    fn device_data() -> SP_DEVINFO_DATA {
        SP_DEVINFO_DATA { cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32, ..Default::default() }
    }

    /// The devices one bus driver reports as present now (`enumerator`:
    /// `BTHENUM` for paired Bluetooth devices and their services, `USB`,
    /// `ACPI`…): each one's instance id and, when Windows gave it a serial
    /// port, that port's name (`COM5`).
    pub fn present_devices(enumerator: &str) -> Vec<(String, Option<String>)> {
        let Some(set) = DeviceSet::present(Some(enumerator), None) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for index in 0.. {
            let mut device = device_data();
            if unsafe { SetupDiEnumDeviceInfo(set.0, index, &mut device) } == 0 {
                break;
            }
            let Some(id) = set.instance_id(&device) else {
                continue;
            };
            // A serial port's name is kept with the device's own settings.
            let port = Key::opened(unsafe {
                SetupDiOpenDevRegKey(set.0, &device, DICS_FLAG_GLOBAL, 0, DIREG_DEV, KEY_READ)
            })
            .and_then(|key| key.text("PortName"));
            found.push((id, port));
        }
        found
    }

    /// The paired Bluetooth devices, by address (`AA:BB:CC:DD:EE:FF`).
    pub fn bluetooth_devices() -> Vec<String> {
        let mut found: Vec<String> =
            present_devices("BTHENUM").iter().filter_map(|(id, _)| super::bluetooth_address_of(id)).collect();
        found.sort();
        found.dedup();
        found
    }

    /// The COM port Windows gave the Bluetooth device `mac` (its Serial
    /// Port service); `None` when it has none.
    pub fn bluetooth_port(mac: &str) -> Option<String> {
        super::bluetooth_port_among(&present_devices("BTHENUM"), mac)
    }

    /// A file handle; closed when dropped.
    struct File(HANDLE);

    impl Drop for File {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    impl File {
        /// Hand `bytes` to the device until all of it is taken (a write may
        /// take a part). `Err` carries Windows' error number, or 0 when the
        /// device took nothing within its time.
        fn write_all(&self, bytes: &[u8]) -> Result<(), u32> {
            let mut at = 0usize;
            while at < bytes.len() {
                let rest = &bytes[at..];
                let ask = rest.len().min(u32::MAX as usize) as u32;
                let mut took = 0u32;
                if unsafe { WriteFile(self.0, rest.as_ptr(), ask, &mut took, std::ptr::null_mut()) } == 0 {
                    return Err(unsafe { GetLastError() });
                }
                if took == 0 {
                    return Err(0);
                }
                at += took as usize;
            }
            Ok(())
        }
    }

    /// Send `bytes` out of the serial port `port` (`COM5`), all of them, and
    /// wait until the port has sent them: closing a port drops what it
    /// still holds. `what` names the device in a message ("the Bluetooth
    /// printer"). 9600 baud, 8 data bits, no parity, one stop bit, no
    /// handshake — what a serial printer expects; a Bluetooth port ignores
    /// the speed.
    pub fn serial_write(port: &str, what: &str, bytes: &[u8]) -> Result<(), String> {
        // The `\\.\` form opens every port, COM10 and above too.
        let path = wide(&format!(r"\\.\{port}"));
        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            return Err(format!("cannot open {port} for {what} (Windows error {code}) — is it switched on and in reach?"));
        }
        let file = File(handle);
        let failed = |doing: &str| {
            let code = unsafe { GetLastError() };
            format!("{port} could not be {doing} (Windows error {code})")
        };
        let mut settings = DCB { DCBlength: std::mem::size_of::<DCB>() as u32, ..Default::default() };
        if unsafe { GetCommState(file.0, &mut settings) } == 0 {
            return Err(failed("read"));
        }
        settings.BaudRate = 9600;
        settings.ByteSize = 8;
        settings.Parity = 0; // none
        settings.StopBits = 0; // one
        // Binary mode (the only one Windows has) and nothing else: no
        // handshake of any kind, DTR and RTS left off.
        settings._bitfield = 1;
        if unsafe { SetCommState(file.0, &settings) } == 0 {
            return Err(failed("set up"));
        }
        // A write that the device does not take within 15 s gives up.
        let times = COMMTIMEOUTS { WriteTotalTimeoutConstant: 15_000, ..Default::default() };
        if unsafe { SetCommTimeouts(file.0, &times) } == 0 {
            return Err(failed("set up"));
        }
        file.write_all(bytes).map_err(|code| match code {
            0 => format!("{what} did not take the whole ticket"),
            code => format!("{port} stopped taking the ticket (Windows error {code})"),
        })?;
        // What the port still holds goes out before it is closed.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            let mut errors = 0u32;
            let mut state = COMSTAT::default();
            if unsafe { ClearCommError(file.0, &mut errors, &mut state) } == 0 {
                return Err(failed("asked what it still holds"));
            }
            if state.cbOutQue == 0 {
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(format!("{what} did not take the whole ticket"));
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    /// `GUID_DEVINTERFACE_USBPRINT`: the interface Windows' USB
    /// printer-class driver (usbprint) shows for every printer it runs.
    const USB_PRINTER_INTERFACE: GUID = GUID::from_u128(0x28d78fad_5a12_11d1_ae5b_0000f803a8c2);

    /// A USB printer as Windows runs it.
    pub struct UsbPrinter {
        pub vid: u16,
        pub pid: u16,
        pub serial: Option<String>,
        /// The device's own path: bytes written to it go to the printer.
        path: Vec<u16>,
        /// The port Windows' USB port monitor made for it (`USB001`), when
        /// the driver says which.
        pub port: Option<String>,
    }

    /// The USB printers attached now: every device the printer-class driver
    /// runs, with the ids its instance id carries. No guess from a name.
    pub fn usb_printers() -> Vec<UsbPrinter> {
        let Some(set) = DeviceSet::present(None, Some(&USB_PRINTER_INTERFACE)) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for index in 0.. {
            let mut interface =
                SP_DEVICE_INTERFACE_DATA { cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32, ..Default::default() };
            if unsafe {
                SetupDiEnumDeviceInterfaces(set.0, std::ptr::null(), &USB_PRINTER_INTERFACE, index, &mut interface)
            } == 0
            {
                break;
            }
            // The first call only says how long the path is.
            let mut need = 0u32;
            unsafe {
                SetupDiGetDeviceInterfaceDetailW(set.0, &interface, std::ptr::null_mut(), 0, &mut need, std::ptr::null_mut())
            };
            if (need as usize) < std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() {
                continue;
            }
            // Whole `u32`s: the detail starts with one.
            let mut room = vec![0u32; (need as usize).div_ceil(4)];
            let detail = room.as_mut_ptr().cast::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>();
            let mut device = device_data();
            let ok = unsafe {
                (*detail).cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
                SetupDiGetDeviceInterfaceDetailW(set.0, &interface, detail, need, std::ptr::null_mut(), &mut device)
            };
            if ok == 0 {
                continue;
            }
            // The path follows the size, zero-ended, inside `room`.
            let Some(path) = (unsafe { text_at(std::ptr::addr_of!((*detail).DevicePath).cast::<u16>()) }) else {
                continue;
            };
            let Some((vid, pid, serial)) = set.instance_id(&device).as_deref().and_then(super::usb_ids) else {
                continue;
            };
            // The port monitor names its port from these two values the
            // driver keeps with the interface: `USB` + 001.
            let port = Key::opened(unsafe { SetupDiOpenDeviceInterfaceRegKey(set.0, &interface, 0, KEY_READ) }).and_then(|key| {
                let number = key.number("Port Number")?;
                Some(format!("{}{number:03}", key.text("Base Name").unwrap_or_else(|| "USB".into())))
            });
            found.push(UsbPrinter { vid, pid, serial, path: wide(&path), port });
        }
        found
    }

    /// Send `bytes` untouched to the USB printer with these ids (a serial
    /// number, when given, must be its own). Through Windows' queue for
    /// that very device when it has one — the spooler owns the device
    /// then, and jobs from other programs keep their turn — else straight
    /// to the device. A printer that is not attached is an error: the job
    /// never goes to another USB printer.
    pub fn usb_write(vid: u16, pid: u16, serial: Option<&str>, doc_name: &str, bytes: &[u8]) -> Result<(), String> {
        let wanted = |p: &UsbPrinter| {
            p.vid == vid
                && p.pid == pid
                && match serial {
                    Some(s) => p.serial.as_deref().is_some_and(|have| have.eq_ignore_ascii_case(s)),
                    None => true,
                }
        };
        let Some(device) = usb_printers().into_iter().find(wanted) else {
            return Err(format!("USB printer {vid:04x}:{pid:04x} not found — plug it in or use host:9100"));
        };
        let queue = device.port.as_deref().and_then(|port| {
            queues().into_iter().find(|q| q.ports.split(',').any(|p| p.trim().trim_end_matches(':').eq_ignore_ascii_case(port)))
        });
        if let Some(queue) = queue {
            return print_raw(&queue.name, doc_name, bytes);
        }
        let handle = unsafe {
            CreateFileW(
                device.path.as_ptr(),
                GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            let code = unsafe { GetLastError() };
            return Err(format!("cannot open USB printer {vid:04x}:{pid:04x} (Windows error {code})"));
        }
        File(handle).write_all(bytes).map_err(|code| match code {
            0 => format!("USB printer {vid:04x}:{pid:04x} did not take the whole ticket"),
            code => format!("USB printer {vid:04x}:{pid:04x} stopped taking the ticket (Windows error {code})"),
        })
    }

    /// Send `bytes` to `printer` untouched, as one job (ESC/POS). The
    /// standard raw job: `StartDocPrinter` with the "RAW" data type — the
    /// spooler and the driver pass such a job on as it is — one page,
    /// `WritePrinter` until every byte is taken (it may take a job in
    /// parts), `EndDocPrinter`. A job that fails part-way is dropped whole
    /// (`AbortPrinter`), never left half sent.
    pub fn print_raw(printer: &str, doc_name: &str, bytes: &[u8]) -> Result<(), String> {
        raw_job(printer, doc_name, bytes, |_, _| Ok(())).map(|_| ())
    }

    /// [`print_raw`]; `started` is told the job's number before any byte is
    /// written (a check holds the job there). Gives that number back.
    fn raw_job(
        printer: &str,
        doc_name: &str,
        bytes: &[u8],
        started: impl FnOnce(PRINTER_HANDLE, u32) -> Result<(), String>,
    ) -> Result<u32, String> {
        let queue = Printer::open(printer)?;
        let failed = |what: &str| {
            // Read first: nothing else may run between the call and this.
            let code = unsafe { GetLastError() };
            format!("\"{printer}\" {what} (Windows error {code})")
        };
        let mut doc = wide(doc_name);
        let mut kind = wide("RAW");
        let info =
            DOC_INFO_1W { pDocName: doc.as_mut_ptr(), pOutputFile: std::ptr::null_mut(), pDatatype: kind.as_mut_ptr() };
        let job = unsafe { StartDocPrinterW(queue.0, 1, &info) };
        if job == 0 {
            return Err(failed("did not take the job"));
        }
        let sent = (|| -> Result<(), String> {
            started(queue.0, job)?;
            if unsafe { StartPagePrinter(queue.0) } == 0 {
                return Err(failed("did not start the job"));
            }
            let mut at = 0usize;
            while at < bytes.len() {
                let rest = &bytes[at..];
                let ask = rest.len().min(u32::MAX as usize) as u32;
                let mut took = 0u32;
                if unsafe { WritePrinter(queue.0, rest.as_ptr().cast(), ask, &mut took) } == 0 {
                    return Err(failed("stopped taking the job"));
                }
                if took == 0 {
                    return Err(format!("\"{printer}\" took none of the job"));
                }
                at += took as usize;
            }
            if unsafe { EndPagePrinter(queue.0) } == 0 {
                return Err(failed("did not finish the job"));
            }
            Ok(())
        })();
        match sent {
            Ok(()) => {
                if unsafe { EndDocPrinter(queue.0) } == 0 {
                    return Err(failed("did not finish the job"));
                }
                Ok(job)
            }
            Err(why) => {
                unsafe { AbortPrinter(queue.0) };
                Err(why)
            }
        }
    }

    /// The check of [`print_raw`], with no device and no paper: `bytes` go
    /// out as a raw job that is held before its first byte, what the
    /// spooler holds for the job is read back, and the job is deleted.
    /// Gives what the spooler held and the size it counted.
    #[cfg(test)]
    pub fn raw_job_as_spooled(printer: &str, bytes: &[u8]) -> Result<(Vec<u8>, u32), String> {
        use windows_sys::Win32::Graphics::Printing::{
            GetJobW, ReadPrinter, SetJobW, JOB_CONTROL_DELETE, JOB_CONTROL_PAUSE, JOB_INFO_2W,
        };
        let code = || unsafe { GetLastError() };
        let job = raw_job(printer, "Kirano check", bytes, |queue, job| {
            if unsafe { SetJobW(queue, job, 0, std::ptr::null(), JOB_CONTROL_PAUSE) } == 0 {
                return Err(format!("the job could not be held (Windows error {})", code()));
            }
            Ok(())
        })?;
        let queue = Printer::open(printer)?;
        // Whatever the reading says, the held job is deleted after it.
        let read = (|| -> Result<(Vec<u8>, u32), String> {
            let mut need = 0u32;
            unsafe { GetJobW(queue.0, job, 2, std::ptr::null_mut(), 0, &mut need) };
            let mut room = vec![0u64; (need as usize).div_ceil(8).max(1)];
            if unsafe { GetJobW(queue.0, job, 2, room.as_mut_ptr().cast(), (room.len() * 8) as u32, &mut need) } == 0 {
                return Err(format!("the job's size could not be read (Windows error {})", code()));
            }
            let size = unsafe { (*room.as_ptr().cast::<JOB_INFO_2W>()).Size };
            let spooled = Printer::open(&format!("{printer},Job {job}"))?;
            let mut all = Vec::new();
            let mut part = vec![0u8; 64 * 1024];
            loop {
                let mut got = 0u32;
                if unsafe { ReadPrinter(spooled.0, part.as_mut_ptr().cast(), part.len() as u32, &mut got) } == 0 {
                    return Err(format!("the spooled job could not be read (Windows error {}); size {size}", code()));
                }
                if got == 0 {
                    break;
                }
                all.extend_from_slice(&part[..got as usize]);
            }
            Ok((all, size))
        })();
        if unsafe { SetJobW(queue.0, job, 0, std::ptr::null(), JOB_CONTROL_DELETE) } == 0 {
            return Err(format!("held job {job} could not be deleted (Windows error {}); read: {:?}", code(), read.map(|r| r.1)));
        }
        read
    }

    /// The jobs waiting on `printer` now (`None` when it is not listed).
    #[cfg(test)]
    pub fn jobs_waiting(printer: &str) -> Option<u32> {
        use windows_sys::Win32::Graphics::Printing::{GetPrinterW, PRINTER_INFO_2W as Info};
        let queue = Printer::open(printer).ok()?;
        let mut need = 0u32;
        unsafe { GetPrinterW(queue.0, 2, std::ptr::null_mut(), 0, &mut need) };
        let mut room = vec![0u64; (need as usize).div_ceil(8).max(1)];
        if unsafe { GetPrinterW(queue.0, 2, room.as_mut_ptr().cast(), (room.len() * 8) as u32, &mut need) } == 0 {
            return None;
        }
        Some(unsafe { (*room.as_ptr().cast::<Info>()).cJobs })
    }

    /// A printer's settings as its driver holds them now, with the
    /// orientation set to portrait. Kept in whole `u32`s so the `DEVMODE`
    /// in it is aligned; a driver's private part follows the public one.
    struct Settings(Vec<u32>);

    impl Settings {
        fn read(name: &str) -> Result<Self, String> {
            let wname = wide(name);
            let printer = Printer::open(name)?;
            let null_window = std::ptr::null_mut();
            let size = unsafe {
                DocumentPropertiesW(null_window, printer.0, wname.as_ptr(), std::ptr::null_mut(), std::ptr::null(), 0)
            };
            if size < std::mem::size_of::<DEVMODEW>() as i32 {
                return Err(format!("the driver of \"{name}\" did not give its settings"));
            }
            let mut words = vec![0u32; (size as usize).div_ceil(4)];
            let dm = words.as_mut_ptr().cast::<DEVMODEW>();
            let got = unsafe {
                DocumentPropertiesW(null_window, printer.0, wname.as_ptr(), dm, std::ptr::null(), DM_OUT_BUFFER)
            };
            if got != IDOK {
                return Err(format!("the driver of \"{name}\" did not give its settings"));
            }
            // Portrait: a landscape slip was turned before it is drawn. The
            // driver takes the change into its own settings (the standard
            // in-and-out call); if it refuses, its settings stay as read.
            unsafe {
                (*dm).Anonymous1.Anonymous1.dmOrientation = DMORIENT_PORTRAIT as i16;
                (*dm).dmFields |= DM_ORIENTATION;
                DocumentPropertiesW(null_window, printer.0, wname.as_ptr(), dm, dm, DM_IN_BUFFER | DM_OUT_BUFFER);
            }
            Ok(Self(words))
        }

        fn devmode(&self) -> &DEVMODEW {
            // Read from a buffer at least `size_of::<DEVMODEW>()` long,
            // filled by the driver, aligned as `u32`.
            unsafe { &*self.0.as_ptr().cast::<DEVMODEW>() }
        }

        /// The driver's name for the paper it is set to: the size's name in
        /// its own list, else the form it names, else its size.
        fn paper_name(&self, sizes: &[DriverSize], fallback: &Paper) -> String {
            let dm = self.devmode();
            let id = unsafe { dm.Anonymous1.Anonymous1.dmPaperSize };
            if let Some(s) = sizes.iter().find(|s| i32::from(s.id) == i32::from(id) && !s.name.is_empty()) {
                return s.name.clone();
            }
            if dm.dmFields & DM_FORMNAME != 0 {
                let form = from_wide(&dm.dmFormName);
                if !form.is_empty() {
                    return form;
                }
            }
            format!(
                "{} × {} mm",
                crate::os_paper::mm_text(fallback.width_mm),
                crate::os_paper::mm_text(fallback.height_mm)
            )
        }
    }

    /// A device (or information) context for a printer; closed when dropped.
    struct Dc(HDC);

    impl Drop for Dc {
        fn drop(&mut self) {
            unsafe { DeleteDC(self.0) };
        }
    }

    impl Dc {
        fn open(name: &str, settings: &Settings, info_only: bool) -> Result<Self, String> {
            let wname = wide(name);
            let dm: *const DEVMODEW = settings.devmode();
            let hdc = unsafe {
                if info_only {
                    CreateICW(std::ptr::null(), wname.as_ptr(), std::ptr::null(), dm)
                } else {
                    CreateDCW(std::ptr::null(), wname.as_ptr(), std::ptr::null(), dm)
                }
            };
            if hdc.is_null() {
                return Err(format!("the driver of \"{name}\" did not open"));
            }
            Ok(Self(hdc))
        }

        fn caps(&self) -> PageCaps {
            let cap = |index: u32| unsafe { GetDeviceCaps(self.0, index as i32) };
            PageCaps {
                dpi: (cap(LOGPIXELSX), cap(LOGPIXELSY)),
                physical: (cap(PHYSICALWIDTH), cap(PHYSICALHEIGHT)),
                offset: (cap(PHYSICALOFFSETX), cap(PHYSICALOFFSETY)),
                printable: (cap(HORZRES), cap(VERTRES)),
            }
        }
    }

    /// One size in a driver's own list.
    struct DriverSize {
        id: u16,
        name: String,
        width_mm: f64,
        height_mm: f64,
    }

    /// The sizes a driver lists (`DeviceCapabilities`): ids, names and
    /// sizes in tenths of a millimetre. Empty when it lists none.
    fn driver_sizes(name: &str) -> Vec<DriverSize> {
        let wname = wide(name);
        // The port is not needed to read a driver's sizes.
        let port = wide("");
        let ask = |what: u16, out: *mut u16| unsafe {
            DeviceCapabilitiesW(wname.as_ptr(), port.as_ptr(), what, out, std::ptr::null())
        };
        let n = ask(DC_PAPERS, std::ptr::null_mut());
        if n <= 0 {
            return Vec::new();
        }
        let n = n as usize;
        let mut ids = vec![0u16; n];
        // Two `i32`s (x, y) for each size.
        let mut points = vec![0i32; n * 2];
        // 64 characters for each name.
        let mut names = vec![0u16; n * 64];
        if ask(DC_PAPERS, ids.as_mut_ptr()) != n as i32
            || ask(DC_PAPERSIZE, points.as_mut_ptr().cast()) != n as i32
            || ask(DC_PAPERNAMES, names.as_mut_ptr()) != n as i32
        {
            return Vec::new();
        }
        (0..n)
            .map(|i| DriverSize {
                id: ids[i],
                name: from_wide(&names[i * 64..(i + 1) * 64]),
                width_mm: f64::from(points[i * 2]) / 10.0,
                height_mm: f64::from(points[i * 2 + 1]) / 10.0,
            })
            .collect()
    }

    /// What `printer`'s driver is set to (`None` = the default printer): its
    /// own page first — the one a job prints on, with the edges it cannot
    /// reach, read from the same kind of device context that prints — then
    /// the other sizes it lists, by name.
    pub fn papers(printer: Option<&str>) -> Result<PrinterPapers, String> {
        let name = printer_name(printer)?;
        let settings = Settings::read(&name)?;
        let caps = Dc::open(&name, &settings, true)?.caps();
        let mut own = caps
            .paper("")
            .ok_or_else(|| format!("the driver of \"{name}\" does not say which paper it is set to"))?;
        let sizes = driver_sizes(&name);
        let own_name = settings.paper_name(&sizes, &own);
        own.id = own_name.clone();
        own.label = own_name.clone();
        let mut papers = vec![own.clone()];
        for s in sizes {
            if s.name.is_empty() || s.width_mm <= 0.0 || s.height_mm <= 0.0 || papers.iter().any(|p| p.id == s.name) {
                continue;
            }
            papers.push(Paper {
                id: s.name.clone(),
                label: s.name,
                width_mm: s.width_mm,
                height_mm: s.height_mm,
                // Only the page the driver is set to has been measured.
                margins_mm: own.margins_mm,
            });
        }
        Ok(PrinterPapers { default_id: own_name, papers })
    }

    /// A two-colour bitmap's header: index 0 black, 1 white.
    #[repr(C)]
    struct TwoColours {
        header: BITMAPINFOHEADER,
        colours: [RGBQUAD; 2],
    }

    /// Print an OS document on `printer` (`None` = the default printer), on
    /// its own page exactly as its driver is set. `lay_out` is given that
    /// page — read from the very device context that prints — and returns
    /// the pages' pictures (grey; 255 is white) and their px per mm; each
    /// goes at its real size from the page's printable corner. `copies`
    /// sends the pages that many times in the one job. `to_file` sends the
    /// driver's output to a file instead of the printer (the test harness:
    /// "Microsoft Print to PDF" writes its PDF there, with no dialog).
    pub fn print_pages(
        printer: Option<&str>,
        doc_name: &str,
        copies: u32,
        to_file: Option<&Path>,
        lay_out: impl FnOnce(&Paper) -> (Vec<image::GrayImage>, f64),
    ) -> Result<(), String> {
        let name = printer_name(printer)?;
        let settings = Settings::read(&name)?;
        let dc = Dc::open(&name, &settings, false)?;
        let caps = dc.caps();
        let own = caps
            .paper(&name)
            .ok_or_else(|| format!("the driver of \"{name}\" does not say which paper it is set to"))?;
        let (pages, px_per_mm) = lay_out(&own);
        let wdoc = wide(doc_name);
        let wfile = to_file.map(|p| wide(&p.display().to_string()));
        let info = DOCINFOW {
            cbSize: std::mem::size_of::<DOCINFOW>() as i32,
            lpszDocName: wdoc.as_ptr(),
            lpszOutput: wfile.as_ref().map_or(std::ptr::null(), |w| w.as_ptr()),
            lpszDatatype: std::ptr::null(),
            fwType: 0,
        };
        if unsafe { StartDocW(dc.0, &info) } <= 0 {
            return Err(format!("\"{name}\" did not take the job"));
        }
        let sent = (|| -> Result<(), String> {
            for _ in 0..copies.max(1) {
                for page in &pages {
                    let picture = device_picture(page, px_per_mm, caps.dpi);
                    let bitmap = TwoColours {
                        header: BITMAPINFOHEADER {
                            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                            biWidth: picture.width,
                            // Positive: the bottom row comes first.
                            biHeight: picture.height,
                            biPlanes: 1,
                            biBitCount: 1,
                            biCompression: BI_RGB,
                            biSizeImage: picture.bits.len() as u32,
                            biXPelsPerMeter: 0,
                            biYPelsPerMeter: 0,
                            biClrUsed: 2,
                            biClrImportant: 2,
                        },
                        colours: [
                            RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 },
                            RGBQUAD { rgbBlue: 255, rgbGreen: 255, rgbRed: 255, rgbReserved: 0 },
                        ],
                    };
                    if unsafe { StartPage(dc.0) } <= 0 {
                        return Err(format!("\"{name}\" did not start a page"));
                    }
                    // A printer's device context starts at the printable
                    // area's top-left: (0, 0) is the corner it can reach.
                    let drawn = unsafe {
                        StretchDIBits(
                            dc.0,
                            0,
                            0,
                            picture.dest.0,
                            picture.dest.1,
                            0,
                            0,
                            picture.width,
                            picture.height,
                            picture.bits.as_ptr().cast(),
                            (&bitmap as *const TwoColours).cast::<BITMAPINFO>(),
                            DIB_RGB_COLORS,
                            SRCCOPY,
                        )
                    };
                    if drawn == 0 {
                        return Err(format!("\"{name}\" did not take the page"));
                    }
                    if unsafe { EndPage(dc.0) } <= 0 {
                        return Err(format!("\"{name}\" did not finish a page"));
                    }
                }
            }
            Ok(())
        })();
        match sent {
            Ok(()) => {
                if unsafe { EndDoc(dc.0) } <= 0 {
                    return Err(format!("\"{name}\" did not finish the job"));
                }
                Ok(())
            }
            Err(why) => {
                unsafe { AbortDoc(dc.0) };
                Err(why)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_device_context_gives_the_page_in_mm_with_its_unreachable_edges() {
        // 600 dots per inch: 4960 × 7016 dots is 210 × 297 mm; the printer
        // starts 100 dots in and 120 down and reaches 4760 × 6776 dots.
        let caps = PageCaps { dpi: (600, 600), physical: (4960, 7016), offset: (100, 120), printable: (4760, 6776) };
        let p = caps.paper("Counter").unwrap();
        assert_eq!(p.id, "Counter");
        assert!((p.width_mm - 209.97).abs() < 0.01 && (p.height_mm - 297.01).abs() < 0.01, "{p:?}");
        let dot = 25.4 / 600.0;
        for (got, want) in p.margins_mm.iter().zip([100.0 * dot, 120.0 * dot, 100.0 * dot, 120.0 * dot]) {
            assert!((got - want).abs() < 1e-9, "{p:?}");
        }
        // Dots that are not square: each side by its own resolution.
        let caps = PageCaps { dpi: (360, 180), physical: (3600, 1800), offset: (0, 0), printable: (3600, 1800) };
        let p = caps.paper("Strip").unwrap();
        assert!((p.width_mm - 254.0).abs() < 1e-9 && (p.height_mm - 254.0).abs() < 1e-9);
        // A driver that reports nothing.
        assert!(PageCaps { dpi: (0, 0), physical: (0, 0), offset: (0, 0), printable: (0, 0) }.paper("X").is_none());
    }

    #[test]
    fn a_page_picture_goes_on_the_printers_dots_at_its_real_size() {
        // 10 px/mm: a 60 × 20 mm picture, its left half black.
        let page = image::GrayImage::from_fn(600, 200, |x, _| image::Luma([if x < 300 { 0 } else { 255 }]));
        // 254 dots per inch is 10 dots per mm: the picture's own dots.
        let p = device_picture(&page, 10.0, (254, 254));
        assert_eq!((p.width, p.height, p.dest), (600, 200, (600, 200)));
        // Rows padded to 32 bits: 600 bits → 76 bytes.
        assert_eq!(p.bits.len(), 76 * 200);
        assert_eq!(p.bits[0], 0x00, "black is 0");
        assert_eq!(p.bits[37], 0x0F, "the picture's own edge between dots 299 and 300");
        assert_eq!(p.bits[38], 0xFF, "white is 1");
        // 203 dots per inch (8 per mm, near enough): 60 mm is 480 dots.
        let p = device_picture(&page, 10.0, (203, 203));
        assert_eq!((p.width, p.height), (480, 160));
        assert_eq!(p.dest, (480, 160), "never stretched below 600 dots per inch");
        let black = p.bits[..60].iter().map(|b| b.count_zeros()).sum::<u32>();
        assert!((239..=241).contains(&black), "half the row stays black: {black}");
        // 1200 dots per inch: the 600-class picture, each dot drawn 2 × 2.
        let p = device_picture(&page, 10.0, (1200, 1200));
        assert_eq!((p.width, p.height), (1417, 472));
        assert_eq!(p.dest, (2834, 944));
        // Real size: 60 mm at 1200 dots per inch is 2834.6 dots.
        assert!((f64::from(p.dest.0) - 60.0 * 1200.0 / 25.4).abs() <= 2.0);
    }

    #[test]
    fn the_bottom_row_comes_first() {
        // Top row black, bottom row white.
        let page = image::GrayImage::from_fn(8, 2, |_, y| image::Luma([if y == 0 { 0 } else { 255 }]));
        let p = device_picture(&page, 10.0, (254, 254));
        assert_eq!(p.bits, [0xFF, 0, 0, 0, 0x00, 0, 0, 0]);
    }

    #[test]
    fn a_usb_devices_ids_are_read_from_its_instance_id() {
        assert_eq!(usb_ids(r"USB\VID_04B8&PID_0E15\X6HN012345"), Some((0x04b8, 0x0e15, Some("X6HN012345".into()))));
        // An id Windows made up (no serial number on the device) is not one.
        assert_eq!(usb_ids(r"USB\VID_0483&PID_5743\6&2C2B3C4D&0&2"), Some((0x0483, 0x5743, None)));
        // One function of a device with several: its ids, no serial.
        assert_eq!(usb_ids(r"usb\vid_0483&pid_5743&mi_00\7&1a2b3c4d&0&0000"), Some((0x0483, 0x5743, None)));
        assert_eq!(usb_ids(r"USB\ROOT_HUB30\4&1C0F7D9&0&0"), None);
        assert_eq!(usb_ids(r"BTHENUM\DEV_AABBCCDDEEFF\8&2F7B1E6&0&BLUETOOTHDEVICE_AABBCCDDEEFF"), None);
        assert_eq!(usb_ids(""), None);
    }

    #[test]
    fn a_bluetooth_address_comes_only_from_a_devices_own_entry() {
        assert_eq!(
            bluetooth_address_of(r"BTHENUM\DEV_AABBCCDDEEFF\8&2F7B1E6&0&BLUETOOTHDEVICE_AABBCCDDEEFF").as_deref(),
            Some("AA:BB:CC:DD:EE:FF")
        );
        assert_eq!(bluetooth_address_of(r"bthenum\dev_0011223344ff\9&1&0&x").as_deref(), Some("00:11:22:33:44:FF"));
        // A service's entry: the twelve hex digits that end the Serial Port
        // service id are not an address (they were listed as one).
        assert_eq!(
            bluetooth_address_of(r"BTHENUM\{00001101-0000-1000-8000-00805F9B34FB}_LOCALMFG&0002\7&1F2E3D4C&0&AABBCCDDEEFF_C00000000"),
            None
        );
        assert_eq!(bluetooth_address_of(r"BTHLE\DEV_AABBCCDDEEFF\8&1&0&x"), None);
        assert_eq!(bluetooth_address_of(r"BTHENUM\DEV_AABBCC\x"), None);
    }

    #[test]
    fn a_bluetooth_printers_port_is_its_own_and_never_anothers() {
        let devices = vec![
            (r"BTHENUM\DEV_AABBCCDDEEFF\8&2F7B1E6&0&BLUETOOTHDEVICE_AABBCCDDEEFF".to_string(), None),
            // The incoming port every Bluetooth radio gets: no device's address.
            (r"BTHENUM\{00001101-0000-1000-8000-00805F9B34FB}_LOCALMFG&0000\7&1F2E3D4C&0&000000000000_00000001".to_string(), Some("COM3".to_string())),
            (r"BTHENUM\{00001101-0000-1000-8000-00805F9B34FB}_LOCALMFG&0002\7&1F2E3D4C&0&AABBCCDDEEFF_C00000000".to_string(), Some("COM5".to_string())),
            (r"BTHENUM\{00001101-0000-1000-8000-00805F9B34FB}_LOCALMFG&0002\7&1F2E3D4C&0&112233445566_C00000000".to_string(), Some("COM7".to_string())),
        ];
        assert_eq!(bluetooth_port_among(&devices, "AA:BB:CC:DD:EE:FF").as_deref(), Some("COM5"));
        assert_eq!(bluetooth_port_among(&devices, "aa:bb:cc:dd:ee:ff").as_deref(), Some("COM5"));
        assert_eq!(bluetooth_port_among(&devices, "11:22:33:44:55:66").as_deref(), Some("COM7"));
        // Paired, but Windows made no port for it; or not paired at all.
        assert_eq!(bluetooth_port_among(&devices[..2], "AA:BB:CC:DD:EE:FF"), None);
        assert_eq!(bluetooth_port_among(&devices, "DE:AD:BE:EF:00:01"), None);
        assert_eq!(bluetooth_port_among(&devices, "not an address"), None);
    }

    /// Manual check on a Windows machine (never part of `cargo test`), with
    /// no printer: Windows' device list is read (how many devices, which
    /// serial ports — never a device's own name or id), and jobs go out of
    /// the serial port named in `KIRANO_COM` with the writer a Bluetooth
    /// printer's port gets; what was sent is saved, to compare with what
    /// came out of the port's other end.
    #[cfg(target_os = "windows")]
    #[test]
    #[ignore]
    fn windows_devices_check() {
        let var = |k: &str| std::env::var(k).unwrap_or_default();
        let out = std::path::PathBuf::from(var("KIRANO_CHECK_OUT"));
        std::fs::create_dir_all(&out).unwrap();
        let mut lines = String::new();
        for bus in ["ACPI", "USB", "BTHENUM"] {
            let devices = present_devices(bus);
            let mut ports: Vec<String> = devices.iter().filter_map(|(_, port)| port.clone()).collect();
            ports.sort();
            let usb = devices.iter().filter(|(id, _)| usb_ids(id).is_some()).count();
            let bluetooth = devices.iter().filter(|(id, _)| bluetooth_address_of(id).is_some()).count();
            lines.push_str(&format!(
                "{bus}: {} devices, serial ports {ports:?}, with USB ids {usb}, with a Bluetooth address {bluetooth}\n",
                devices.len()
            ));
        }
        let usb = usb_printers();
        lines.push_str(&format!(
            "USB printers: {}, with a port named {}\n",
            usb.len(),
            usb.iter().filter(|p| p.port.is_some()).count()
        ));
        lines.push_str(&format!("Bluetooth devices: {}\n", bluetooth_devices().len()));
        lines.push_str(&format!("port of a Bluetooth printer that is not there: {:?}\n", bluetooth_port("02:00:00:00:00:01")));
        lines.push_str(&format!(
            "a USB printer that is not there: {:?}\n",
            usb_write(0x1209, 0x0001, Some("NOT-THERE"), "Kirano check", b"x")
        ));
        let port = var("KIRANO_COM");
        if !port.trim().is_empty() {
            let every_byte: Vec<u8> = (0..=255u8).cycle().take(256 * 64).collect();
            let jobs = [
                ("a kitchen ticket", crate::print_share::escpos_job("KOT 7\n2 x Thali\n1 x Roti")),
                ("every byte value, 64 times", every_byte),
            ];
            for (i, (what, bytes)) in jobs.iter().enumerate() {
                std::fs::write(out.join(format!("serial-sent-{i}.bin")), bytes).unwrap();
                let started = std::time::Instant::now();
                let result = serial_write(port.trim(), "the check's port", bytes);
                lines.push_str(&format!(
                    "{what} out of {}: {} bytes, {result:?}, {:.1} s\n",
                    port.trim(),
                    bytes.len(),
                    started.elapsed().as_secs_f32()
                ));
            }
            lines.push_str(&format!(
                "a port that is not there: {:?}\n",
                serial_write("COM250", "the check's port", b"x")
            ));
        }
        std::fs::write(out.join("devices.txt"), lines).unwrap();
        std::fs::write(out.join("done.txt"), "done\n").unwrap();
    }

    /// Manual check on a Windows machine (never part of `cargo test`): a
    /// raw job reaches the spooler byte for byte, and the printers listed
    /// are the ones Windows itself lists. Nothing prints and no file is
    /// asked for: each job is held before its first byte, read back from
    /// the spooler and deleted, on a printer that only ever writes files.
    #[cfg(target_os = "windows")]
    #[test]
    #[ignore]
    fn windows_raw_check() {
        let var = |k: &str| std::env::var(k).unwrap_or_default();
        let out = std::path::PathBuf::from(var("KIRANO_CHECK_OUT"));
        std::fs::create_dir_all(&out).unwrap();
        let printer = Some(var("KIRANO_WIN_PRINTER"))
            .filter(|p| !p.trim().is_empty())
            .unwrap_or("Microsoft Print to PDF".into());
        assert!(
            ["Microsoft Print to PDF", "Microsoft XPS Document Writer"].contains(&printer.as_str()),
            "a check uses a file-only printer, never a real one"
        );
        let mut lines = format!("jobs waiting before: {:?}\n", jobs_waiting(&printer));
        let every_byte: Vec<u8> = (0..=255u8).cycle().take(256 * 700).collect();
        let jobs = [
            ("a kitchen ticket", crate::print_share::escpos_job("KOT 7\n2 x Thali\n1 x Roti")),
            ("every byte value, 700 times", every_byte),
        ];
        for (what, bytes) in &jobs {
            match raw_job_as_spooled(&printer, bytes) {
                Ok((spooled, size)) => {
                    let first_difference = spooled.iter().zip(bytes.iter()).position(|(a, b)| a != b);
                    lines.push_str(&format!(
                        "{what}: sent {} bytes, spooler counted {size}, spooler held {}, same {}, first difference {first_difference:?}\n",
                        bytes.len(),
                        spooled.len(),
                        &spooled == bytes,
                    ));
                }
                Err(why) => lines.push_str(&format!("{what}: sent {} bytes, error: {why}\n", bytes.len())),
            }
        }
        lines.push_str(&format!("jobs waiting after: {:?}\n", jobs_waiting(&printer)));
        // The list Windows' own command gives, with the same two kinds left
        // out; only counts are written, never a printer's name.
        let listed = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Console]::OutputEncoding = [Text.Encoding]::UTF8; Get-Printer | Where-Object { -not $_.WorkOffline -and $_.PrinterStatus -ne 'Offline' } | ForEach-Object { $_.Name }",
            ])
            .output()
            .unwrap();
        let mut theirs: Vec<String> = String::from_utf8_lossy(&listed.stdout)
            .lines()
            .map(|l| l.trim_end_matches(['\r', '\n']).to_string())
            .filter(|l| !l.trim().is_empty())
            .collect();
        let mut ours = printers();
        theirs.sort();
        ours.sort();
        lines.push_str(&format!(
            "printers: the app lists {}, Windows lists {}, same {}, the check's printer listed {}\n",
            ours.len(),
            theirs.len(),
            ours == theirs,
            ours.contains(&printer),
        ));
        std::fs::write(out.join("raw.txt"), lines).unwrap();
        std::fs::write(out.join("done.txt"), "done\n").unwrap();
    }
}

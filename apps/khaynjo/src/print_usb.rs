//! USB host ESC/POS write (`usb:VID:PID` or `usb:VID:PID:serial`).

use std::io::Write;

/// Parsed USB printer identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsbId {
    pub vid: u16,
    pub pid: u16,
    pub serial: Option<String>,
}

/// Parse `usb:04b8:0e15` or `usb:04b8:0e15:SERIAL`.
pub fn parse_id(raw: &str) -> Result<UsbId, String> {
    let s = raw
        .trim()
        .strip_prefix("usb:")
        .or_else(|| raw.trim().strip_prefix("USB:"))
        .unwrap_or(raw.trim());
    let mut parts = s.split(':');
    let vid = u16::from_str_radix(parts.next().unwrap_or(""), 16)
        .map_err(|_| format!("bad USB id {raw} — use usb:VID:PID"))?;
    let pid = u16::from_str_radix(parts.next().unwrap_or(""), 16)
        .map_err(|_| format!("bad USB id {raw} — use usb:VID:PID"))?;
    let serial = parts.next().map(|x| x.to_string()).filter(|x| !x.is_empty());
    Ok(UsbId { vid, pid, serial })
}

pub fn format_id(id: &UsbId) -> String {
    match &id.serial {
        Some(s) => format!("usb:{:04x}:{:04x}:{s}", id.vid, id.pid),
        None => format!("usb:{:04x}:{:04x}", id.vid, id.pid),
    }
}

/// Attached USB printer candidates for Find.
pub fn list_printers() -> Vec<String> {
    #[cfg(target_os = "android")]
    {
        return crate::print_android::list_usb();
    }
    #[cfg(target_os = "linux")]
    {
        return list_linux();
    }
    #[cfg(target_os = "windows")]
    {
        return list_windows();
    }
    #[cfg(target_os = "macos")]
    {
        return list_macos();
    }
    #[cfg(not(any(
        target_os = "android",
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        Vec::new()
    }
}

pub fn write_bytes(target: &str, bytes: &[u8]) -> Result<(), String> {
    let id = parse_id(target)?;
    #[cfg(target_os = "android")]
    {
        return crate::print_android::write_usb(id.vid, id.pid, id.serial.as_deref(), bytes);
    }
    #[cfg(target_os = "linux")]
    {
        return write_linux(&id, bytes);
    }
    #[cfg(target_os = "windows")]
    {
        return write_windows(&id, bytes);
    }
    #[cfg(target_os = "macos")]
    {
        return write_macos(&id, bytes);
    }
    #[cfg(not(any(
        target_os = "android",
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        let _ = (id, bytes);
        Err("USB printing is not supported on this system".into())
    }
}

#[cfg(target_os = "linux")]
fn list_linux() -> Vec<String> {
    let mut out = Vec::new();
    // usblp nodes — common for ESC/POS.
    if let Ok(rd) = std::fs::read_dir("/dev/usb") {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("lp") {
                if let Some(id) = sysfs_usb_id_for_usblp(&name) {
                    let t = format_id(&id);
                    if !out.contains(&t) {
                        out.push(t);
                    }
                }
            }
        }
    }
    // Also walk /sys/bus/usb/devices for printer class (7).
    if let Ok(rd) = std::fs::read_dir("/sys/bus/usb/devices") {
        for e in rd.flatten() {
            let path = e.path();
            let class = std::fs::read_to_string(path.join("bInterfaceClass"))
                .or_else(|_| {
                    // device dir — check interfaces
                    Err(std::io::Error::other("skip"))
                })
                .unwrap_or_default();
            let _ = class;
            let vid = read_hex_u16(&path.join("idVendor"));
            let pid = read_hex_u16(&path.join("idProduct"));
            if let (Some(vid), Some(pid)) = (vid, pid) {
                // Prefer devices that look like printers: usblp child or class 7 iface.
                let is_printer = path.join("bDeviceClass").exists()
                    && (has_iface_class(&path, 0x07) || usblp_bound(&path));
                if is_printer {
                    let serial = std::fs::read_to_string(path.join("serial"))
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    let id = UsbId { vid, pid, serial };
                    let t = format_id(&id);
                    if !out.contains(&t) {
                        out.push(t);
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(target_os = "linux")]
fn read_hex_u16(path: &std::path::Path) -> Option<u16> {
    let s = std::fs::read_to_string(path).ok()?;
    u16::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok()
}

#[cfg(target_os = "linux")]
fn has_iface_class(dev: &std::path::Path, class: u8) -> bool {
    let Ok(rd) = std::fs::read_dir(dev) else {
        return false;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.contains(':') {
            continue;
        }
        if let Ok(c) = std::fs::read_to_string(e.path().join("bInterfaceClass")) {
            if u8::from_str_radix(c.trim(), 16).ok() == Some(class) {
                return true;
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn usblp_bound(dev: &std::path::Path) -> bool {
    let Ok(rd) = std::fs::read_dir(dev) else {
        return false;
    };
    rd.flatten().any(|e| {
        let p = e.path().join("driver");
        std::fs::read_link(p)
            .ok()
            .map(|l| l.to_string_lossy().contains("usblp"))
            .unwrap_or(false)
    })
}

#[cfg(target_os = "linux")]
fn sysfs_usb_id_for_usblp(lp_name: &str) -> Option<UsbId> {
    let link = std::fs::read_link(format!("/sys/class/usb/{lp_name}/device")).ok()?;
    // Walk up until idVendor appears.
    let mut cur = link;
    for _ in 0..8 {
        if let (Some(vid), Some(pid)) = (
            read_hex_u16(&cur.join("idVendor")),
            read_hex_u16(&cur.join("idProduct")),
        ) {
            let serial = std::fs::read_to_string(cur.join("serial"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            return Some(UsbId { vid, pid, serial });
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn write_linux(id: &UsbId, bytes: &[u8]) -> Result<(), String> {
    // Path-style leftover from discovery.
    if id.vid == 0 && id.pid == 0 {
        return Err("bad USB id".into());
    }
    // Direct path target: usb:/dev/usb/lp0 stored oddly — handle via target string in caller.
    if let Some(path) = find_usblp_path(id) {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|e| {
                format!(
                    "cannot open {}: {e} — check cable / permissions (dialout group)",
                    path.display()
                )
            })?;
        f.write_all(bytes).map_err(|e| e.to_string())?;
        return f.flush().map_err(|e| e.to_string());
    }
    // Fallback: any /dev/usb/lp*
    if let Ok(rd) = std::fs::read_dir("/dev/usb") {
        let mut lps: Vec<_> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().starts_with("lp"))
                    .unwrap_or(false)
            })
            .collect();
        lps.sort();
        if let Some(path) = lps.into_iter().next() {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
            f.write_all(bytes).map_err(|e| e.to_string())?;
            return f.flush().map_err(|e| e.to_string());
        }
    }
    Err(format!(
        "USB printer {:04x}:{:04x} not found — plug it in (OTG/host) or use host:9100",
        id.vid, id.pid
    ))
}

#[cfg(target_os = "linux")]
fn find_usblp_path(id: &UsbId) -> Option<std::path::PathBuf> {
    let rd = std::fs::read_dir("/dev/usb").ok()?;
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.starts_with("lp") {
            continue;
        }
        if let Some(found) = sysfs_usb_id_for_usblp(&name) {
            if found.vid == id.vid && found.pid == id.pid {
                if id.serial.is_none() || id.serial == found.serial {
                    return Some(e.path());
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn list_windows() -> Vec<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script = r#"
Get-PnpDevice -Class USB -Status OK -ErrorAction SilentlyContinue |
  Where-Object { $_.InstanceId -match 'VID_([0-9A-Fa-f]{4}).*PID_([0-9A-Fa-f]{4})' } |
  ForEach-Object {
    if ($_.InstanceId -match 'VID_([0-9A-Fa-f]{4}).*PID_([0-9A-Fa-f]{4})') {
      $vid = $Matches[1].ToLower(); $pid = $Matches[2].ToLower()
      # Heuristic: printer-ish friendly name
      if ($_.FriendlyName -match 'Print|POS|Receipt|Thermal|EPSON|Star|Citizen|Bixolon') {
        "usb:${vid}:${pid}"
      }
    }
  } | Select-Object -Unique
"#;
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(str::trim)
                .filter(|l| l.starts_with("usb:"))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(target_os = "windows")]
fn write_windows(id: &UsbId, bytes: &[u8]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let dir = std::env::temp_dir().join("kumkaj-usb-raw");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let path_s = path.display().to_string().replace('\'', "''");
    let vid = format!("{:04X}", id.vid);
    let pid = format!("{:04X}", id.pid);
    // Find a Winspool queue whose port is USB00n and whose PnP id matches VID/PID,
    // then WritePrinter RAW (same path as system raw KOT).
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$vid = '{vid}'; $pid = '{pid}'
$printers = Get-Printer -ErrorAction SilentlyContinue
$match = $null
foreach ($p in $printers) {{
  $port = Get-PrinterPort -Name $p.PortName -ErrorAction SilentlyContinue
  $inst = (Get-PnpDevice -FriendlyName $p.Name -ErrorAction SilentlyContinue | Select-Object -First 1).InstanceId
  if (-not $inst) {{ $inst = $p.DeviceID }}
  if ($inst -and $inst.ToUpper().Contains("VID_$vid") -and $inst.ToUpper().Contains("PID_$pid")) {{
    $match = $p.Name; break
  }}
}}
if (-not $match) {{
  # Fall back: any USB00 printer
  $match = ($printers | Where-Object {{ $_.PortName -like 'USB*' }} | Select-Object -First 1).Name
}}
if (-not $match) {{ throw "USB printer VID_$vid PID_$pid not found as a Windows printer — install it or use host:9100" }}
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class KumkajUsbRaw {{
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)]
  public class DOCINFOA {{
    [MarshalAs(UnmanagedType.LPStr)] public string pDocName;
    [MarshalAs(UnmanagedType.LPStr)] public string pOutputFile;
    [MarshalAs(UnmanagedType.LPStr)] public string pDataType;
  }}
  [DllImport("winspool.Drv", EntryPoint = "OpenPrinterA", SetLastError = true, CharSet = CharSet.Ansi)]
  public static extern bool OpenPrinter([MarshalAs(UnmanagedType.LPStr)] string szPrinter, out IntPtr hPrinter, IntPtr pd);
  [DllImport("winspool.Drv", EntryPoint = "ClosePrinter", SetLastError = true)]
  public static extern bool ClosePrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "StartDocPrinterA", SetLastError = true, CharSet = CharSet.Ansi)]
  public static extern bool StartDocPrinter(IntPtr hPrinter, int level, [In] DOCINFOA di);
  [DllImport("winspool.Drv", EntryPoint = "EndDocPrinter", SetLastError = true)]
  public static extern bool EndDocPrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "StartPagePrinter", SetLastError = true)]
  public static extern bool StartPagePrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "EndPagePrinter", SetLastError = true)]
  public static extern bool EndPagePrinter(IntPtr hPrinter);
  [DllImport("winspool.Drv", EntryPoint = "WritePrinter", SetLastError = true)]
  public static extern bool WritePrinter(IntPtr hPrinter, IntPtr pBytes, int dwCount, out int dwWritten);
}}
"@
$bytes = [IO.File]::ReadAllBytes('{path_s}')
$h = [IntPtr]::Zero
if (-not [KumkajUsbRaw]::OpenPrinter($match, [ref]$h, [IntPtr]::Zero)) {{ throw "OpenPrinter failed for $match" }}
try {{
  $di = New-Object KumkajUsbRaw+DOCINFOA
  $di.pDocName = 'KUMKAJ'
  $di.pDataType = 'RAW'
  if (-not [KumkajUsbRaw]::StartDocPrinter($h, 1, $di)) {{ throw "StartDocPrinter failed" }}
  try {{
    if (-not [KumkajUsbRaw]::StartPagePrinter($h)) {{ throw "StartPagePrinter failed" }}
    try {{
      $pinned = [Runtime.InteropServices.GCHandle]::Alloc($bytes, [Runtime.InteropServices.GCHandleType]::Pinned)
      try {{
        $offset = 0
        while ($offset -lt $bytes.Length) {{
          $written = 0
          $ptr = [IntPtr]::Add($pinned.AddrOfPinnedObject(), $offset)
          $chunk = $bytes.Length - $offset
          if (-not [KumkajUsbRaw]::WritePrinter($h, $ptr, $chunk, [ref]$written)) {{ throw "WritePrinter failed" }}
          if ($written -le 0) {{ throw "WritePrinter wrote nothing" }}
          $offset += $written
        }}
      }} finally {{ $pinned.Free() }}
    }} finally {{ [void][KumkajUsbRaw]::EndPagePrinter($h) }}
  }} finally {{ [void][KumkajUsbRaw]::EndDocPrinter($h) }}
}} finally {{ [void][KumkajUsbRaw]::ClosePrinter($h) }}
"#,
        vid = vid,
        pid = pid,
        path_s = path_s,
    );
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("PowerShell did not start ({e})"))?;
    let _ = std::fs::remove_file(&path);
    if out.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&out.stderr);
        let line = detail
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("USB write failed");
        Err(line.into())
    }
}

#[cfg(target_os = "macos")]
fn list_macos() -> Vec<String> {
    use std::process::Command;
    let out = Command::new("system_profiler")
        .args(["SPUSBDataType", "-json"])
        .output()
        .ok()
        .filter(|o| o.status.success());
    let Some(out) = out else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    fn walk(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(map) => {
                let vid = map
                    .get("vendor_id")
                    .and_then(|x| x.as_str())
                    .and_then(parse_apple_hex);
                let pid = map
                    .get("product_id")
                    .and_then(|x| x.as_str())
                    .and_then(parse_apple_hex);
                if let (Some(vid), Some(pid)) = (vid, pid) {
                    let name = map
                        .get("_name")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if name.contains("print")
                        || name.contains("pos")
                        || name.contains("receipt")
                        || name.contains("epson")
                        || name.contains("star")
                        || map.get("bInterfaceClass").and_then(|x| x.as_str()) == Some("0x07")
                    {
                        let t = format!("usb:{vid:04x}:{pid:04x}");
                        if !out.contains(&t) {
                            out.push(t);
                        }
                    }
                }
                for val in map.values() {
                    walk(val, out);
                }
            }
            serde_json::Value::Array(a) => {
                for val in a {
                    walk(val, out);
                }
            }
            _ => {}
        }
    }
    walk(&v, &mut found);
    found
}

#[cfg(target_os = "macos")]
fn parse_apple_hex(s: &str) -> Option<u16> {
    let s = s.split_whitespace().next()?.trim_start_matches("0x");
    u16::from_str_radix(s, 16).ok()
}

#[cfg(target_os = "macos")]
fn write_macos(id: &UsbId, bytes: &[u8]) -> Result<(), String> {
    // Prefer CUPS raw queue whose URI contains the USB VID/PID, else lp to USB.
    use std::process::Command;
    let vid = format!("{:04x}", id.vid);
    let pid = format!("{:04x}", id.pid);
    let lpstat = Command::new("lpstat").args(["-v"]).output().ok();
    let queue = lpstat.and_then(|o| {
        String::from_utf8_lossy(&o.stdout)
            .lines()
            .find(|l| {
                let l = l.to_ascii_lowercase();
                l.contains(&vid) && l.contains(&pid)
            })
            .and_then(|l| l.split_whitespace().nth(2).map(|s| s.trim_end_matches(':').to_string()))
    });
    let dir = std::env::temp_dir().join("kumkaj-usb-raw");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let mut cmd = Command::new("lp");
    cmd.args(["-o", "raw"]);
    if let Some(q) = queue.as_deref() {
        cmd.args(["-d", q]);
    }
    cmd.arg("--").arg(&path);
    let result = match cmd.output() {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let detail = String::from_utf8_lossy(&out.stderr);
            let line = detail
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("lp raw failed");
            Err(format!(
                "{line} — USB {:04x}:{:04x} needs a CUPS raw queue or use host:9100",
                id.vid, id.pid
            ))
        }
        Err(e) => Err(format!("lp did not start ({e})")),
    };
    let _ = std::fs::remove_file(&path);
    result
}

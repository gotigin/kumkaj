//! Bluetooth SPP raw write for ESC/POS printers (`bt:AA:BB:CC:DD:EE:FF`).

use std::io::Write;

/// Normalize `bt:…` / bare MAC into uppercase `AA:BB:CC:DD:EE:FF`.
pub fn parse_mac(raw: &str) -> Result<String, String> {
    let s = raw
        .trim()
        .strip_prefix("bt:")
        .or_else(|| raw.trim().strip_prefix("BT:"))
        .unwrap_or(raw.trim());
    let hex: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() != 12 {
        return Err(format!(
            "Bluetooth address must be six bytes (got {s}) — use bt:AA:BB:CC:DD:EE:FF"
        ));
    }
    let parts: Vec<String> = hex
        .as_bytes()
        .chunks(2)
        .map(|c| String::from_utf8_lossy(c).to_ascii_uppercase())
        .collect();
    Ok(parts.join(":"))
}

/// Paired / known Bluetooth printer targets for Find (`bt:MAC` lines).
pub fn list_printers() -> Vec<String> {
    #[cfg(target_os = "android")]
    {
        return crate::print_android::list_bluetooth();
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

/// Write raw ESC/POS bytes over Bluetooth SPP.
pub fn write_bytes(mac_or_target: &str, bytes: &[u8]) -> Result<(), String> {
    let mac = parse_mac(mac_or_target)?;
    #[cfg(target_os = "android")]
    {
        return crate::print_android::write_bluetooth(&mac, bytes);
    }
    #[cfg(target_os = "linux")]
    {
        return write_linux(&mac, bytes);
    }
    #[cfg(target_os = "windows")]
    {
        return write_windows(&mac, bytes);
    }
    #[cfg(target_os = "macos")]
    {
        return write_macos(&mac, bytes);
    }
    #[cfg(not(any(
        target_os = "android",
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        let _ = (mac, bytes);
        Err("Bluetooth printing is not supported on this system".into())
    }
}

#[cfg(target_os = "linux")]
fn list_linux() -> Vec<String> {
    use std::process::Command;
    let out = Command::new("bluetoothctl")
        .args(["devices", "Paired"])
        .output()
        .ok()
        .or_else(|| Command::new("bluetoothctl").args(["devices"]).output().ok());
    let Some(out) = out.filter(|o| o.status.success()) else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            // Device AA:BB:CC:DD:EE:FF Name…
            let mut parts = l.split_whitespace();
            if parts.next()? != "Device" {
                return None;
            }
            let mac = parts.next()?;
            parse_mac(mac).ok().map(|m| format!("bt:{m}"))
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn write_linux(mac: &str, bytes: &[u8]) -> Result<(), String> {
    // RFCOMM channel 1 is the usual ESC/POS SPP channel.
    write_linux_rfcomm(mac, 1, bytes)
}

#[cfg(target_os = "linux")]
fn write_linux_rfcomm(mac: &str, channel: u8, bytes: &[u8]) -> Result<(), String> {
    use std::os::fd::FromRawFd;

    const AF_BLUETOOTH: libc::c_int = 31;
    const BTPROTO_RFCOMM: libc::c_int = 3;
    const BDADDR_BREDR: u8 = 0;

    #[repr(C)]
    struct BdAddr {
        b: [u8; 6],
    }
    #[repr(C)]
    struct SockaddrRc {
        rc_family: libc::sa_family_t,
        rc_bdaddr: BdAddr,
        rc_channel: u8,
    }

    let parts: Vec<u8> = mac
        .split(':')
        .filter_map(|p| u8::from_str_radix(p, 16).ok())
        .collect();
    if parts.len() != 6 {
        return Err(format!("bad Bluetooth address {mac}"));
    }
    // BlueZ stores address bytes in reverse order in sockaddr.
    let mut bd = [0u8; 6];
    for i in 0..6 {
        bd[i] = parts[5 - i];
    }

    let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_STREAM, BTPROTO_RFCOMM) };
    if fd < 0 {
        return Err(
            "cannot open Bluetooth socket — is BlueZ installed and Bluetooth on?".into(),
        );
    }
    let addr = SockaddrRc {
        rc_family: AF_BLUETOOTH as libc::sa_family_t,
        rc_bdaddr: BdAddr { b: bd },
        rc_channel: channel,
    };
    // Mark BREDR family in the unused high bits of channel on some stacks —
    // channel alone is enough for classic SPP.
    let _ = BDADDR_BREDR;
    let rc = unsafe {
        libc::connect(
            fd,
            &addr as *const _ as *const libc::sockaddr,
            std::mem::size_of::<SockaddrRc>() as libc::socklen_t,
        )
    };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        unsafe { libc::close(fd) };
        return Err(format!(
            "cannot reach Bluetooth printer {mac} ({err}) — pair it in system settings first"
        ));
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn list_windows() -> Vec<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // Paired Bluetooth devices with a MAC in InstanceId / BlindAddress.
    let script = r#"
Get-PnpDevice -Class Bluetooth -Status OK -ErrorAction SilentlyContinue |
  ForEach-Object {
    $id = $_.InstanceId
    if ($id -match '([0-9A-Fa-f]{2})([0-9A-Fa-f]{2})([0-9A-Fa-f]{2})([0-9A-Fa-f]{2})([0-9A-Fa-f]{2})([0-9A-Fa-f]{2})') {
      $m = ($Matches[1..6] -join ':').ToUpper()
      "bt:$m"
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
                .filter(|l| l.starts_with("bt:"))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(target_os = "windows")]
fn write_windows(mac: &str, bytes: &[u8]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let dir = std::env::temp_dir().join("kumkaj-bt-raw");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::new_v4()));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let path_s = path.display().to_string().replace('\'', "''");
    let mac_s = mac.replace('\'', "''");
    // Prefer an outgoing RFCOMM COM port whose name/device id mentions the MAC.
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$mac = '{mac_s}'.Replace(':','').ToUpper()
$ports = Get-CimInstance Win32_SerialPort -ErrorAction SilentlyContinue |
  Where-Object {{ $_.PNPDeviceID -and ($_.PNPDeviceID.ToUpper() -like "*$mac*") }}
if (-not $ports) {{
  $ports = Get-CimInstance Win32_SerialPort -ErrorAction SilentlyContinue |
    Where-Object {{ $_.Description -match 'Bluetooth|RFCOMM|Standard Serial' }}
}}
if (-not $ports) {{ throw "no Bluetooth COM port for {mac_s} — pair the printer and enable a serial port" }}
$portName = $ports | Select-Object -First 1 -ExpandProperty DeviceID
$bytes = [IO.File]::ReadAllBytes('{path_s}')
$p = New-Object System.IO.Ports.SerialPort $portName, 9600, None, 8, One
$p.WriteTimeout = 15000
$p.Open()
try {{ $p.Write($bytes, 0, $bytes.Length) }} finally {{ $p.Close() }}
"#,
        mac_s = mac_s,
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
            .unwrap_or("Bluetooth write failed");
        Err(line.into())
    }
}

#[cfg(target_os = "macos")]
fn list_macos() -> Vec<String> {
    use std::process::Command;
    // system_profiler lists classic Bluetooth devices with addresses.
    let out = Command::new("system_profiler")
        .args(["SPBluetoothDataType", "-json"])
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
                if let Some(addr) = map
                    .get("device_address")
                    .or_else(|| map.get("address"))
                    .and_then(|x| x.as_str())
                {
                    if let Ok(m) = parse_mac(addr) {
                        let t = format!("bt:{m}");
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
fn write_macos(mac: &str, bytes: &[u8]) -> Result<(), String> {
    // Prefer a /dev/cu.* that embeds the MAC (Bluetooth serial).
    let compact: String = mac.chars().filter(|c| *c != ':').collect();
    let compact_l = compact.to_ascii_lowercase();
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir("/dev") {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            if !(name.starts_with("cu.") || name.starts_with("tty.")) {
                continue;
            }
            if name.contains("bluetooth") || name.contains(&compact_l) {
                candidates.push(e.path());
            }
        }
    }
    candidates.sort();
    let Some(path) = candidates.into_iter().next() else {
        return Err(format!(
            "no Bluetooth serial device for {mac} — pair the printer in System Settings"
        ));
    };
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    f.write_all(bytes).map_err(|e| e.to_string())?;
    f.flush().map_err(|e| e.to_string())
}

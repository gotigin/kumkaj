//! "Start with this computer": the shop's server PC comes back after a power
//! cut or a restart without anyone opening the app — devices depend on it.
//! Per user, no admin: the Windows Run key, a Linux autostart entry, or a
//! macOS LaunchAgent (written as Apple documents it; no Mac to try it on yet).

/// The name the entry is kept under.
#[cfg(any(windows, target_os = "linux"))]
const ENTRY: &str = "Kumkaj Khaynjo";

/// Passed by the startup entry: serve devices at once, before anyone
/// unlocks the screen.
pub const SERVE_ARG: &str = "--serve";

/// This run was started by the computer, not by a person.
pub fn launched_at_boot() -> bool {
    std::env::args().any(|a| a == SERVE_ARG)
}

/// This system lets the app start with it.
pub fn supported() -> bool {
    cfg!(any(windows, target_os = "linux", target_os = "macos"))
}

/// The app starts with this computer now.
pub fn is_on() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("reg")
            .args(["query", RUN_KEY, "/v", ENTRY])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(target_os = "linux")]
    {
        linux_entry().is_some_and(|p| p.exists())
    }
    #[cfg(target_os = "macos")]
    {
        mac_agent().is_some_and(|p| p.exists())
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

/// Turn starting with this computer on or off.
pub fn set(on: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let quoted = format!("\"{}\" {SERVE_ARG}", exe.display());
        let args: Vec<&str> = if on {
            vec!["add", RUN_KEY, "/v", ENTRY, "/t", "REG_SZ", "/d", &quoted, "/f"]
        } else {
            vec!["delete", RUN_KEY, "/v", ENTRY, "/f"]
        };
        let out = std::process::Command::new("reg")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() && on {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let entry = linux_entry().ok_or("No autostart folder on this system")?;
        set_linux_entry(&entry, on, &exe)
    }
    #[cfg(target_os = "macos")]
    {
        let agent = mac_agent().ok_or("No home folder on this system")?;
        if !on {
            let _ = std::fs::remove_file(&agent);
            return Ok(());
        }
        if let Some(dir) = agent.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&agent, launch_agent_plist(&mac_label(), &exe.display().to_string())).map_err(|e| e.to_string())
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = (on, exe);
        Err("Starting with this computer is not available here".into())
    }
}

#[cfg(windows)]
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

/// Write (or remove) the desktop's autostart entry at `entry`.
#[cfg(target_os = "linux")]
fn set_linux_entry(entry: &std::path::Path, on: bool, exe: &std::path::Path) -> Result<(), String> {
    if !on {
        let _ = std::fs::remove_file(entry);
        return Ok(());
    }
    if let Some(dir) = entry.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let desktop = format!(
        "[Desktop Entry]\nType=Application\nName={ENTRY}\nExec=\"{}\" {SERVE_ARG}\nX-GNOME-Autostart-enabled=true\n",
        exe.display()
    );
    std::fs::write(entry, desktop).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn linux_entry() -> Option<std::path::PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
    Some(config.join("autostart").join("kumkaj-khaynjo.desktop"))
}

/// The LaunchAgent's label: the app id, so dev and production never clash.
#[cfg(any(target_os = "macos", test))]
fn mac_label() -> String {
    format!("{}.startup", crate::store::APP_ID)
}

#[cfg(target_os = "macos")]
fn mac_agent() -> Option<std::path::PathBuf> {
    Some(dirs::home_dir()?.join("Library").join("LaunchAgents").join(format!("{}.plist", mac_label())))
}

/// A per-user LaunchAgent that opens the app at login with `--serve`.
#[cfg(any(target_os = "macos", test))]
fn launch_agent_plist(label: &str, exe: &str) -> String {
    let xml = |s: &str| {
        s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n\
         \t<key>Label</key><string>{}</string>\n\
         \t<key>ProgramArguments</key>\n\t<array><string>{}</string><string>{SERVE_ARG}</string></array>\n\
         \t<key>RunAtLoad</key><true/>\n\
         </dict>\n</plist>\n",
        xml(label),
        xml(exe),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The macOS LaunchAgent names the app by its id and passes `--serve`;
    /// a path with odd characters stays valid XML.
    #[test]
    fn the_mac_launch_agent_starts_the_server_at_login() {
        let plist = launch_agent_plist(&mac_label(), "/Applications/Khaynjo & Co.app/Contents/MacOS/kumkaj-khaynjo");
        assert!(plist.contains(&format!("<string>{}.startup</string>", crate::store::APP_ID)));
        assert!(plist.contains("<string>/Applications/Khaynjo &amp; Co.app/Contents/MacOS/kumkaj-khaynjo</string>"));
        assert!(plist.contains("<string>--serve</string>"));
        assert!(plist.contains("<key>RunAtLoad</key><true/>"));
    }

    /// The Linux autostart entry is written with `--serve` and removed again.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_linux_autostart_entry_comes_and_goes() {
        let dir = std::env::temp_dir().join(format!("khaynjo-autostart-{}", crate::domain::new_id()));
        let entry = dir.join("autostart").join("kumkaj-khaynjo.desktop");
        set_linux_entry(&entry, true, std::path::Path::new("/opt/khaynjo/kumkaj-khaynjo")).unwrap();
        let text = std::fs::read_to_string(&entry).unwrap();
        assert!(text.contains("Exec=\"/opt/khaynjo/kumkaj-khaynjo\" --serve"), "{text}");
        assert!(text.contains("X-GNOME-Autostart-enabled=true"));
        set_linux_entry(&entry, false, std::path::Path::new("/opt/khaynjo/kumkaj-khaynjo")).unwrap();
        assert!(!entry.exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}

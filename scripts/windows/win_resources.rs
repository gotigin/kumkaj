// Windows .exe resources for every Kumkaj app — `include!`d by
// apps/<app>/build.rs and called once from its `main`.
//
// The Windows App Certification Kit (docs/DEV.md § Windows App Certification
// Kit) and the Store read three things from a desktop .exe that Makepad does
// not produce: an application manifest (UAC level `asInvoker`, Windows 10 / 11
// as the supported OS, per-monitor DPI awareness — the same PerMonitorV2 that
// Makepad asks for at startup), version info (company, product, file and
// product version from Cargo.toml) and the app icon (`resources/icon.ico`).
//
// Cross builds from Linux (`x86_64-pc-windows-gnu`) compile them with mingw
// `windres`; MSVC builds with `rc.exe` when it is on PATH. Without a resource
// compiler the build still succeeds and says so (cargo:warning) — the exe
// then has no icon, no version tab and fails WACK's manifest checks.
#[allow(dead_code)]
fn embed_windows_resources(manifest_dir: &std::path::Path, label: &str) {
    use std::path::PathBuf;
    use std::process::Command;
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = manifest_dir.join("resources").join("icon.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut nums: Vec<u16> = version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .take(3)
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    nums.resize(3, 0);
    let (a, b, c) = (nums[0], nums[1], nums[2]);
    let bin = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "app".into());
    // Plain ASCII inside the .rc / manifest: no quoting or code-page surprises.
    let clean = |s: &str| -> String {
        s.chars()
            .map(|ch| if ch.is_ascii() && ch != '"' && ch != '<' && ch != '>' && ch != '&' { ch } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let product = clean(label);
    let product = if product.is_empty() { clean(&bin) } else { product };
    let description = clean(&std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default());
    let description = if description.is_empty() { product.clone() } else { description };
    let ident: String = product.chars().filter(|ch| ch.is_ascii_alphanumeric()).collect();

    let manifest = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity type="win32" name="Kumkaj.{ident}" version="{a}.{b}.{c}.0" processorArchitecture="amd64"/>
  <description>{description}</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 and Windows 11 -->
      <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#
    );
    std::fs::write(out.join("app.manifest"), manifest).expect("write app.manifest");
    let has_icon = icon.is_file() && std::fs::copy(&icon, out.join("app.ico")).is_ok();
    if !has_icon {
        println!("cargo:warning=Windows build: no {} — the .exe gets no icon", icon.display());
    }
    // Numeric constants instead of <winver.h>: VOS_NT_WINDOWS32 0x40004,
    // VFT_APP 1; 0x409 / 1200 = US English, Unicode.
    let rc = format!(
        r#"1 24 "app.manifest"
{icon_line}
1 VERSIONINFO
FILEVERSION {a},{b},{c},0
PRODUCTVERSION {a},{b},{c},0
FILEFLAGSMASK 0x3f
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE 0x1
FILESUBTYPE 0x0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "Kumkaj"
      VALUE "FileDescription", "{product}"
      VALUE "FileVersion", "{a}.{b}.{c}.0"
      VALUE "InternalName", "{bin}"
      VALUE "LegalCopyright", "Copyright (C) Kumkaj"
      VALUE "OriginalFilename", "{bin}.exe"
      VALUE "ProductName", "{product}"
      VALUE "ProductVersion", "{a}.{b}.{c}.0"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#,
        icon_line = if has_icon { "1 ICON \"app.ico\"" } else { "" },
    );
    std::fs::write(out.join("app.rc"), rc).expect("write app.rc");

    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    let (tool, args, obj): (String, Vec<String>, PathBuf) = if msvc {
        let res = out.join("app.res");
        (
            "rc.exe".into(),
            vec!["/nologo".into(), "/fo".into(), res.display().to_string(), "app.rc".into()],
            res,
        )
    } else {
        let obj = out.join("app_res.o");
        let tool = std::env::var("WINDRES").unwrap_or_else(|_| {
            if cfg!(windows) { "windres".into() } else { "x86_64-w64-mingw32-windres".into() }
        });
        (
            tool,
            vec!["-i".into(), "app.rc".into(), "-O".into(), "coff".into(), "-o".into(), obj.display().to_string()],
            obj,
        )
    };
    println!("cargo:rerun-if-env-changed=WINDRES");
    match Command::new(&tool).args(&args).current_dir(&out).status() {
        Ok(s) if s.success() => {
            // A plain object on the link line is always linked in whole, so the
            // unreferenced resource section is kept.
            println!("cargo:rustc-link-arg-bins={}", obj.display());
        }
        Ok(s) => println!("cargo:warning=Windows resources: {tool} failed ({s}) — .exe without manifest / version / icon"),
        Err(e) => println!("cargo:warning=Windows resources: {tool} not found ({e}) — .exe without manifest / version / icon"),
    }
}

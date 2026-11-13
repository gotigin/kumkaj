//! Bake `env/development.env` or `env/production.env` (under this crate) into the binary.
//!
//! Selection:
//! - `KUMKAJ_ENV=development|production` if set (scripts export this)
//! - else Cargo `PROFILE=release` → production, otherwise development

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn defaults(profile: &str) -> HashMap<&'static str, &'static str> {
    match profile {
        "production" => HashMap::from([
            ("KUMKAJ_ENV", "production"),
            ("KUMKAJ_APP_ID", "shop.kirano"),
            ("KUMKAJ_APP_LABEL", "Kirano"),
            ("KUMKAJ_SHOW_SIZE_BADGE", "0"),
        ]),
        _ => HashMap::from([
            ("KUMKAJ_ENV", "development"),
            ("KUMKAJ_APP_ID", "shop.kirano.debug"),
            ("KUMKAJ_APP_LABEL", "Kirano Dev"),
            ("KUMKAJ_SHOW_SIZE_BADGE", "1"),
        ]),
    }
}

fn main() {
    // Tell rustc about custom cfgs set below (avoids unexpected_cfgs warnings).
    println!("cargo:rustc-check-cfg=cfg(kumkaj_hide_console)");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let kumkaj_env = env::var("KUMKAJ_ENV").unwrap_or_else(|_| {
        match env::var("PROFILE").as_deref() {
            Ok("release") => "production".into(),
            _ => "development".into(),
        }
    });

    let env_path = manifest_dir.join("env").join(format!("{kumkaj_env}.env"));
    println!("cargo:rerun-if-changed={}", env_path.display());
    println!("cargo:rerun-if-env-changed=KUMKAJ_ENV");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let mut vars: HashMap<String, String> = defaults(&kumkaj_env)
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    if let Ok(text) = fs::read_to_string(&env_path) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !key.is_empty() {
                vars.insert(key.to_string(), value.to_string());
            }
        }
    } else {
        println!(
            "cargo:warning=Kirano env file missing at {} — using built-in {kumkaj_env} defaults",
            env_path.display()
        );
    }

    // Ensure ENV key matches the selected profile even if the file omits it.
    vars.insert("KUMKAJ_ENV".into(), kumkaj_env.clone());

    // Production Windows PE: no console when the user double-clicks the exe.
    // Dev packages keep the console. Driven by KUMKAJ_ENV, not Cargo profile
    // (Windows Dev packages also use --release — Cargo debug PE is huge).
    if kumkaj_env == "production" {
        println!("cargo:rustc-cfg=kumkaj_hide_console");
    }

    for (key, value) in vars {
        println!("cargo:rustc-env={key}={value}");
    }

    write_android_menu_assets_file_list(&manifest_dir);
    bake_translations(&manifest_dir);
}

/// UI translations: `translations/<code>.tsv` (one `key<TAB>text` line per
/// English key in `locale.rs` `en`) → one deflated blob per language in
/// OUT_DIR plus `translations.rs`, the `PACKS` table `locale.rs` includes.
/// A language's table is inflated the first time it is shown, so the
/// binary carries ~110 languages without ~8 MB of raw text.
fn bake_translations(manifest_dir: &PathBuf) {
    use std::io::Write;
    let dir = manifest_dir.join("translations");
    println!("cargo:rerun-if-changed={}", dir.display());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut codes: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("tsv") {
                continue;
            }
            let code = path.file_stem().unwrap().to_string_lossy().to_string();
            let text = fs::read(&path).expect("read translation table");
            let mut enc =
                flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
            enc.write_all(&text).unwrap();
            fs::write(out_dir.join(format!("tr_{code}.deflate")), enc.finish().unwrap())
                .expect("write deflated translation table");
            codes.push(code);
        }
    }
    codes.sort();
    let mut src = String::from("pub const PACKS: &[(&str, &[u8])] = &[\n");
    for code in &codes {
        src.push_str(&format!(
            "    ({code:?}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/tr_{code}.deflate\"))),\n"
        ));
    }
    src.push_str("];\n");
    fs::write(out_dir.join("translations.rs"), src).expect("write translations.rs");
}

/// Paths under `assets/items` + `assets/templates` for Android first-run extract
/// from the APK (desktop already stages those folders beside the exe).
fn write_android_menu_assets_file_list(manifest_dir: &PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let list_path = out_dir.join("menu_assets_file_list.txt");
    let mut lines: Vec<String> = Vec::new();
    for sub in ["items", "templates"] {
        let root = manifest_dir.join("assets").join(sub);
        println!("cargo:rerun-if-changed={}", root.display());
        if !root.is_dir() {
            continue;
        }
        collect_asset_files(&root, sub, &mut lines);
    }
    lines.sort();
    fs::write(&list_path, lines.join("\n")).expect("write menu_assets_file_list.txt");
}

fn collect_asset_files(dir: &PathBuf, rel: &str, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        let child_rel = if rel.is_empty() {
            name.to_string()
        } else {
            format!("{rel}/{name}")
        };
        if path.is_dir() {
            collect_asset_files(&path, &child_rel, out);
        } else if path.is_file() {
            out.push(child_rel);
        }
    }
}

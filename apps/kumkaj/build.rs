//! Bake `env/development.env` or `env/production.env` into the binary.
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
            ("KUMKAJ_APP_ID", "com.kumkaj.app"),
            ("KUMKAJ_APP_LABEL", "Kumkaj"),
            ("KUMKAJ_SHOW_SIZE_BADGE", "0"),
        ]),
        _ => HashMap::from([
            ("KUMKAJ_ENV", "development"),
            ("KUMKAJ_APP_ID", "com.kumkaj.app.debug"),
            ("KUMKAJ_APP_LABEL", "Kumkaj Dev"),
            ("KUMKAJ_SHOW_SIZE_BADGE", "1"),
        ]),
    }
}

fn main() {
    // Tell rustc about custom cfgs set below (avoids unexpected_cfgs warnings).
    println!("cargo:rustc-check-cfg=cfg(kumkaj_hide_console)");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("apps/kumkaj → workspace root")
        .to_path_buf();

    let kumkaj_env = env::var("KUMKAJ_ENV").unwrap_or_else(|_| {
        match env::var("PROFILE").as_deref() {
            Ok("release") => "production".into(),
            _ => "development".into(),
        }
    });

    let env_path = workspace.join("env").join(format!("{kumkaj_env}.env"));
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
            "cargo:warning=Kumkaj env file missing at {} — using built-in {kumkaj_env} defaults",
            env_path.display()
        );
    }

    // Ensure ENV key matches the selected profile even if the file omits it.
    vars.insert("KUMKAJ_ENV".into(), kumkaj_env.clone());

    // Production Windows PE: no console window when the user double-clicks the
    // exe. Dev keeps the console (logs / panic output). Driven by KUMKAJ_ENV
    // because windows-dev also builds with Cargo `--release`.
    if kumkaj_env == "production" {
        println!("cargo:rustc-cfg=kumkaj_hide_console");
    }

    for (key, value) in vars {
        println!("cargo:rustc-env={key}={value}");
    }
}

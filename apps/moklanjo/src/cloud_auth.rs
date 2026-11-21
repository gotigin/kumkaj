//! Firebase Auth for the Moklanjo app via Identity Toolkit REST.
//!
//! Same project as https://moklanjo.com/account (`sites/moklanjo/client/firebase-init.js`).
//! Email register / sign-in / forgot / refresh. Federated providers open the
//! website (no in-app OAuth popup in Makepad).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Public web API key (Firebase Console → Project settings). Not a secret.
pub const FIREBASE_API_KEY: &str = "AIzaSyB5E2V97YXGraGHdwnXYQhPsLdFpOIpDy0";
pub const FIREBASE_PROJECT_ID: &str = "moklanjo-com";
pub const FIREBASE_AUTH_DOMAIN: &str = "moklanjo.com";
pub const ACCOUNT_WEB_URL: &str = "https://moklanjo.com/account";

const IDENTITY_BASE: &str = "https://identitytoolkit.googleapis.com/v1";
const SECURE_TOKEN_BASE: &str = "https://securetoken.googleapis.com/v1";

const SECRET_FILE: &str = "cloud_auth.secret";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub local_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub id_token: String,
    pub refresh_token: String,
    /// Unix seconds when `id_token` is expected to expire.
    pub expires_at: u64,
}

#[derive(Deserialize)]
struct SignInResponse {
    #[serde(rename = "idToken")]
    id_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: String,
    #[serde(rename = "localId")]
    local_id: String,
    email: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct RefreshResponse {
    #[serde(rename = "id_token")]
    id_token: String,
    #[serde(rename = "refresh_token")]
    refresh_token: String,
    #[serde(rename = "expires_in")]
    expires_in: String,
    #[serde(rename = "user_id")]
    user_id: String,
}

#[derive(Deserialize)]
struct FirebaseErrorBody {
    error: Option<FirebaseErrorDetail>,
}

#[derive(Deserialize)]
struct FirebaseErrorDetail {
    message: Option<String>,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn expires_at_from_in(expires_in: &str) -> u64 {
    let secs: u64 = expires_in.parse().unwrap_or(3600);
    now_unix().saturating_add(secs.saturating_sub(60))
}

fn session_from_sign_in(r: SignInResponse) -> AuthSession {
    AuthSession {
        local_id: r.local_id,
        email: r.email,
        display_name: r.display_name.filter(|s| !s.is_empty()),
        id_token: r.id_token,
        refresh_token: r.refresh_token,
        expires_at: expires_at_from_in(&r.expires_in),
    }
}

fn parse_error_message(body: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<FirebaseErrorBody>(body) {
        if let Some(msg) = parsed.error.and_then(|e| e.message) {
            return friendly_firebase_message(&msg);
        }
    }
    if body.trim().is_empty() {
        "Something went wrong.".into()
    } else {
        body.chars().take(200).collect()
    }
}

/// Map Identity Toolkit `error.message` codes to shop-facing copy.
pub fn friendly_firebase_message(code: &str) -> String {
    let upper = code.trim();
    if upper.starts_with("WEAK_PASSWORD") {
        return "Use a password with at least 6 characters.".into();
    }
    match upper {
        "EMAIL_EXISTS" | "EMAIL_ALREADY_IN_USE" => {
            "That email is already used by another account.".into()
        }
        "EMAIL_NOT_FOUND" => {
            "No account with that email. Continue to create one, or use another method.".into()
        }
        "INVALID_PASSWORD" | "INVALID_LOGIN_CREDENTIALS" | "INVALID_EMAIL" => {
            "Email or password is wrong.".into()
        }
        "TOO_MANY_ATTEMPTS_TRY_LATER" => "Too many attempts. Try again later.".into(),
        "USER_DISABLED" => "This account is disabled.".into(),
        "OPERATION_NOT_ALLOWED" => "That sign-in method is not enabled yet.".into(),
        other => other.to_string(),
    }
}

fn post_json<T: for<'de> Deserialize<'de>>(url: &str, body: &serde_json::Value) -> Result<T, String> {
    let resp = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_json(body.clone())
        .map_err(|e| match e {
            ureq::Error::Status(code, r) => {
                let text = r.into_string().unwrap_or_default();
                if code == 400 || code == 401 || code == 403 {
                    parse_error_message(&text)
                } else {
                    format!("Auth HTTP {code}")
                }
            }
            other => other.to_string(),
        })?;
    resp.into_json::<T>()
        .map_err(|e| format!("Auth response: {e}"))
}

/// Register with email/password. Caller should fall back to sign-in on EMAIL_EXISTS.
pub fn sign_up_email(email: &str, password: &str) -> Result<AuthSession, String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("Enter an email address.".into());
    }
    if password.len() < 6 {
        return Err("Use a password with at least 6 characters.".into());
    }
    let url = format!("{IDENTITY_BASE}/accounts:signUp?key={FIREBASE_API_KEY}");
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "returnSecureToken": true,
    });
    let r: SignInResponse = post_json(&url, &body)?;
    Ok(session_from_sign_in(r))
}

pub fn sign_in_email(email: &str, password: &str) -> Result<AuthSession, String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("Enter an email address.".into());
    }
    if password.is_empty() {
        return Err("Enter your password.".into());
    }
    let url = format!("{IDENTITY_BASE}/accounts:signInWithPassword?key={FIREBASE_API_KEY}");
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "returnSecureToken": true,
    });
    let r: SignInResponse = post_json(&url, &body)?;
    Ok(session_from_sign_in(r))
}

/// Site-style Continue: create, or sign in if the email already exists.
pub fn continue_email(
    email: &str,
    password: &str,
    display_name: Option<&str>,
) -> Result<(AuthSession, bool), String> {
    match sign_up_email(email, password) {
        Ok(mut session) => {
            let created = true;
            if let Some(name) = display_name.map(str::trim).filter(|s| !s.is_empty()) {
                if let Ok(updated) = update_display_name(&session.id_token, name) {
                    session = updated;
                } else {
                    session.display_name = Some(name.to_string());
                }
            }
            let _ = send_email_verification(&session.id_token);
            Ok((session, created))
        }
        Err(e) => {
            let lower = e.to_lowercase();
            if lower.contains("already") || lower.contains("email_exists") {
                let session = sign_in_email(email, password)?;
                Ok((session, false))
            } else {
                Err(e)
            }
        }
    }
}

pub fn update_display_name(id_token: &str, display_name: &str) -> Result<AuthSession, String> {
    let url = format!("{IDENTITY_BASE}/accounts:update?key={FIREBASE_API_KEY}");
    let body = serde_json::json!({
        "idToken": id_token,
        "displayName": display_name,
        "returnSecureToken": true,
    });
    let r: SignInResponse = post_json(&url, &body)?;
    Ok(session_from_sign_in(r))
}

pub fn send_email_verification(id_token: &str) -> Result<(), String> {
    let url = format!("{IDENTITY_BASE}/accounts:sendOobCode?key={FIREBASE_API_KEY}");
    let body = serde_json::json!({
        "requestType": "VERIFY_EMAIL",
        "idToken": id_token,
    });
    let _: serde_json::Value = post_json(&url, &body)?;
    Ok(())
}

/// Soft success when the email is unknown (no account enumeration).
pub fn send_password_reset(email: &str) -> Result<(), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("Enter an email address.".into());
    }
    let url = format!("{IDENTITY_BASE}/accounts:sendOobCode?key={FIREBASE_API_KEY}");
    let body = serde_json::json!({
        "requestType": "PASSWORD_RESET",
        "email": email,
    });
    match post_json::<serde_json::Value>(&url, &body) {
        Ok(_) => Ok(()),
        Err(e) => {
            let lower = e.to_lowercase();
            if lower.contains("email_not_found") || lower.contains("no account") {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

pub fn refresh_id_token(refresh_token: &str) -> Result<AuthSession, String> {
    let url = format!("{SECURE_TOKEN_BASE}/token?key={FIREBASE_API_KEY}");
    let form = format!(
        "grant_type=refresh_token&refresh_token={}",
        urlencoding_minimal(refresh_token)
    );
    let resp = ureq::post(&url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&form)
        .map_err(|e| match e {
            ureq::Error::Status(_, r) => {
                parse_error_message(&r.into_string().unwrap_or_default())
            }
            other => other.to_string(),
        })?;
    let r: RefreshResponse = resp
        .into_json()
        .map_err(|e| format!("Refresh response: {e}"))?;
    Ok(AuthSession {
        local_id: r.user_id,
        email: None,
        display_name: None,
        id_token: r.id_token,
        refresh_token: r.refresh_token,
        expires_at: expires_at_from_in(&r.expires_in),
    })
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Return a session with a fresh id token if within ~60s of expiry.
pub fn ensure_fresh_id_token(session: &AuthSession) -> Result<AuthSession, String> {
    if session.expires_at > now_unix() + 30 && !session.id_token.is_empty() {
        return Ok(session.clone());
    }
    let mut refreshed = refresh_id_token(&session.refresh_token)?;
    if refreshed.email.is_none() {
        refreshed.email = session.email.clone();
    }
    if refreshed.display_name.is_none() {
        refreshed.display_name = session.display_name.clone();
    }
    Ok(refreshed)
}

pub fn secret_path(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(SECRET_FILE)
}

pub fn load_session(db_path: &Path) -> Option<AuthSession> {
    let path = secret_path(db_path);
    let raw = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_session(db_path: &Path, session: &AuthSession) -> Result<(), String> {
    let path = secret_path(db_path);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("secret.tmp");
    fs::write(&tmp, raw).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_session(db_path: &Path) {
    let _ = fs::remove_file(secret_path(db_path));
}

/// Open the product `/account` page for Google / Microsoft / GitHub / phone.
pub fn open_account_in_browser() -> Result<(), String> {
    open_url(ACCOUNT_WEB_URL)
}

pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "android")]
    {
        let _ = url;
        return Err("Open https://moklanjo.com/account in a browser for Google and other methods.".into());
    }
    #[allow(unreachable_code)]
    Err("Cannot open a browser on this device.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_maps_common_codes() {
        assert!(friendly_firebase_message("EMAIL_EXISTS").contains("already"));
        assert!(friendly_firebase_message("INVALID_PASSWORD").contains("wrong"));
        assert!(friendly_firebase_message("WEAK_PASSWORD").contains("6"));
    }

    #[test]
    fn session_roundtrip_file() {
        let dir = crate::test_dirs::run_root().join("cloud_auth_secret");
        let _ = std::fs::create_dir_all(&dir);
        let db = dir.join("books.sqlite");
        let session = AuthSession {
            local_id: "uid1".into(),
            email: Some("a@b.c".into()),
            display_name: Some("A".into()),
            id_token: "id".into(),
            refresh_token: "ref".into(),
            expires_at: 99,
        };
        save_session(&db, &session).unwrap();
        let loaded = load_session(&db).unwrap();
        assert_eq!(loaded.local_id, "uid1");
        clear_session(&db);
        assert!(load_session(&db).is_none());
    }
}

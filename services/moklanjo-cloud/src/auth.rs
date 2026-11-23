//! Verify Firebase ID tokens for the Khaynjo Firebase project only.

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const CERTS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

#[derive(Debug, Clone)]
pub struct FirebaseClaims {
    pub uid: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    phone_number: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    email_verified: bool,
    aud: String,
    iss: String,
}

struct CachedCerts {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

pub struct FirebaseVerifier {
    project_id: String,
    http: reqwest::Client,
    cache: RwLock<Option<CachedCerts>>,
}

impl FirebaseVerifier {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            http: reqwest::Client::new(),
            cache: RwLock::new(None),
        }
    }

    pub fn issuer(&self) -> String {
        format!("https://securetoken.google.com/{}", self.project_id)
    }

    pub async fn verify(&self, token: &str) -> Result<FirebaseClaims, String> {
        // Local debug smoke only. Never set DEV_AUTH_BYPASS on AWS / release builds.
        // Not a substitute for Auth Emulator on product UI paths.
        if cfg!(debug_assertions)
            && std::env::var("DEV_AUTH_BYPASS").ok().as_deref() == Some("1")
        {
            if let Some(rest) = token.strip_prefix("dev:") {
                let mut parts = rest.splitn(2, ':');
                let uid = parts.next().unwrap_or("").trim();
                let email = parts.next().map(str::trim).filter(|s| !s.is_empty());
                if uid.is_empty() {
                    return Err("dev token needs uid (Bearer dev:<uid> or dev:<uid>:<email>)".into());
                }
                tracing::warn!(%uid, "DEV_AUTH_BYPASS accepted — local only");
                return Ok(FirebaseClaims {
                    uid: uid.to_string(),
                    email: email.map(str::to_string),
                    phone: None,
                    name: None,
                    email_verified: email.is_some(),
                });
            }
        }

        // Local Auth Emulator (FIREBASE_AUTH_EMULATOR_HOST). Debug builds only —
        // never set on AWS. Accepts emulator ID tokens without Google cert check.
        if cfg!(debug_assertions) && std::env::var("FIREBASE_AUTH_EMULATOR_HOST").is_ok() {
            return self.verify_emulator_token(token);
        }

        let header = decode_header(token).map_err(|e| format!("bad token header: {e}"))?;
        let kid = header
            .kid
            .ok_or_else(|| "token missing kid".to_string())?;
        let key = self.decoding_key(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.project_id.clone()]);
        validation.set_issuer(&[self.issuer()]);

        let data = decode::<IdTokenClaims>(token, &key, &validation)
            .map_err(|e| format!("token invalid: {e}"))?;
        self.claims_from_token(data.claims)
    }

    fn verify_emulator_token(&self, token: &str) -> Result<FirebaseClaims, String> {
        // Auth Emulator issues `alg: none` JWTs. jsonwebtoken rejects that header
        // even with insecure_disable_signature_validation — decode payload by hand.
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() < 2 {
            return Err("emulator token malformed".into());
        }
        let payload_json = b64url_decode(parts[1])
            .map_err(|e| format!("emulator token payload b64: {e}"))?;
        let c: IdTokenClaims = serde_json::from_slice(&payload_json)
            .map_err(|e| format!("emulator token payload json: {e}"))?;
        if c.aud != self.project_id {
            return Err(format!(
                "emulator token aud mismatch: got {}, want {}",
                c.aud, self.project_id
            ));
        }
        if c.iss != self.issuer() {
            return Err(format!(
                "emulator token iss mismatch: got {}, want {}",
                c.iss,
                self.issuer()
            ));
        }
        tracing::warn!("FIREBASE_AUTH_EMULATOR_HOST accepted token — local only");
        self.claims_from_token(c)
    }

    fn claims_from_token(&self, c: IdTokenClaims) -> Result<FirebaseClaims, String> {
        if c.sub.is_empty() {
            return Err("token missing sub".into());
        }
        Ok(FirebaseClaims {
            uid: c.sub,
            email: c.email,
            phone: c.phone_number,
            name: c.name,
            email_verified: c.email_verified,
        })
    }

    async fn decoding_key(&self, kid: &str) -> Result<DecodingKey, String> {
        {
            let guard = self.cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.fetched_at.elapsed() < Duration::from_secs(3600) {
                    if let Some(k) = cached.keys.get(kid) {
                        return Ok(k.clone());
                    }
                }
            }
        }
        self.refresh_certs().await?;
        let guard = self.cache.read().await;
        guard
            .as_ref()
            .and_then(|c| c.keys.get(kid).cloned())
            .ok_or_else(|| format!("unknown token kid {kid}"))
    }

    async fn refresh_certs(&self) -> Result<(), String> {
        let map: HashMap<String, String> = self
            .http
            .get(CERTS_URL)
            .send()
            .await
            .map_err(|e| format!("fetch google certs: {e}"))?
            .error_for_status()
            .map_err(|e| format!("google certs status: {e}"))?
            .json()
            .await
            .map_err(|e| format!("google certs json: {e}"))?;

        let mut keys = HashMap::new();
        for (kid, pem) in map {
            match DecodingKey::from_rsa_pem(pem.as_bytes()) {
                Ok(k) => {
                    keys.insert(kid, k);
                }
                Err(e) => tracing::warn!(%kid, error = %e, "skip bad google cert"),
            }
        }
        *self.cache.write().await = Some(CachedCerts {
            keys,
            fetched_at: Instant::now(),
        });
        Ok(())
    }
}

fn b64url_decode(input: &str) -> Result<Vec<u8>, String> {
    fn val(c: u8) -> Result<u8, String> {
        Ok(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => return Err(format!("bad b64 char {}", c as char)),
        })
    }
    let clean: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        .collect();
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    let mut i = 0;
    while i + 4 <= clean.len() {
        let n = ((val(clean[i])? as u32) << 18)
            | ((val(clean[i + 1])? as u32) << 12)
            | ((val(clean[i + 2])? as u32) << 6)
            | (val(clean[i + 3])? as u32);
        out.push(((n >> 16) & 0xff) as u8);
        out.push(((n >> 8) & 0xff) as u8);
        out.push((n & 0xff) as u8);
        i += 4;
    }
    let rem = clean.len() - i;
    if rem == 2 {
        let n = ((val(clean[i])? as u32) << 18) | ((val(clean[i + 1])? as u32) << 12);
        out.push(((n >> 16) & 0xff) as u8);
    } else if rem == 3 {
        let n = ((val(clean[i])? as u32) << 18)
            | ((val(clean[i + 1])? as u32) << 12)
            | ((val(clean[i + 2])? as u32) << 6);
        out.push(((n >> 16) & 0xff) as u8);
        out.push(((n >> 8) & 0xff) as u8);
    } else if rem == 1 {
        return Err("truncated b64".into());
    }
    Ok(out)
}

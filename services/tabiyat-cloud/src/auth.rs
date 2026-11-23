//! Verify Firebase ID tokens for the Tabiyat Firebase project only.

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
    #[allow(dead_code)]
    aud: String,
    #[allow(dead_code)]
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
        let c = data.claims;
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

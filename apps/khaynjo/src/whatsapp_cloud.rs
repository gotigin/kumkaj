//! Meta WhatsApp Cloud API — shop-owned credentials, direct Graph calls.
//!
//! No Kumkaj backend. The shop Server device holds the token and calls
//! `graph.facebook.com`. See plan: Phone Number ID + Access Token.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Pinned Graph API version (one constant — do not scatter version strings).
pub const GRAPH_API_VERSION: &str = "v21.0";

pub const STATUS_PENDING: &str = "Pending";
pub const STATUS_UPLOADING: &str = "Uploading";
pub const STATUS_SENDING: &str = "Sending";
pub const STATUS_ACCEPTED: &str = "Accepted";
pub const STATUS_FAILED: &str = "Failed";
pub const STATUS_UNKNOWN: &str = "Unknown";

const GRAPH_HOST: &str = "https://graph.facebook.com";
const HTTP_TIMEOUT: Duration = Duration::from_secs(45);

/// Public settings the UI may show (never includes the raw access token).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WhatsappCloudPublic {
    pub enabled: bool,
    pub phone_number_id: String,
    pub has_token: bool,
    pub template_name: String,
    pub template_lang: String,
}

/// Full credentials for Server-side Graph calls only.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WhatsappCloudCreds {
    pub enabled: bool,
    pub phone_number_id: String,
    pub access_token: String,
    pub template_name: String,
    pub template_lang: String,
}

impl WhatsappCloudCreds {
    pub fn is_configured(&self) -> bool {
        self.enabled
            && !self.phone_number_id.trim().is_empty()
            && !self.access_token.trim().is_empty()
    }

    pub fn to_public(&self) -> WhatsappCloudPublic {
        WhatsappCloudPublic {
            enabled: self.enabled,
            phone_number_id: self.phone_number_id.clone(),
            has_token: !self.access_token.trim().is_empty(),
            template_name: self.template_name.clone(),
            template_lang: if self.template_lang.trim().is_empty() {
                "en".into()
            } else {
                self.template_lang.clone()
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum GraphError {
    /// Token / permissions / phone id — do not template-fallback.
    Auth(String),
    /// Network / transport — do not template-fallback; may be Unknown.
    Network(String),
    /// Outside customer-service window / template required.
    NeedsTemplate(String),
    /// Other Meta rejection.
    Rejected(String),
    /// Request may have been accepted but the reply was lost.
    Uncertain(String),
}

impl GraphError {
    pub fn message(&self) -> &str {
        match self {
            Self::Auth(s)
            | Self::Network(s)
            | Self::NeedsTemplate(s)
            | Self::Rejected(s)
            | Self::Uncertain(s) => s,
        }
    }

    pub fn is_needs_template(&self) -> bool {
        matches!(self, Self::NeedsTemplate(_))
    }

    pub fn is_uncertain(&self) -> bool {
        matches!(self, Self::Uncertain(_))
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build()
}

fn auth_header(token: &str) -> String {
    format!("Bearer {}", token.trim())
}

fn classify_meta_error(status: u16, body: &str) -> GraphError {
    let lower = body.to_ascii_lowercase();
    let code = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            v.get("error")
                .and_then(|e| e.get("code"))
                .and_then(|c| c.as_i64())
        })
        .unwrap_or(0);

    // Expired / invalid token, permissions, wrong phone number id.
    if status == 401
        || status == 403
        || matches!(code, 190 | 10 | 200 | 100)
        || lower.contains("access token")
        || lower.contains("oauth")
        || lower.contains("permission")
        || lower.contains("expired")
        || lower.contains("revoked")
    {
        return GraphError::Auth(human_meta_error(body));
    }

    // Session window / re-engagement / must use template.
    if matches!(code, 131047 | 131026 | 131048 | 131051)
        || lower.contains("24 hour")
        || lower.contains("24-hour")
        || lower.contains("re-engagement")
        || lower.contains("customer service window")
        || (lower.contains("template") && lower.contains("required"))
    {
        return GraphError::NeedsTemplate(human_meta_error(body));
    }

    GraphError::Rejected(human_meta_error(body))
}

fn human_meta_error(body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(body) {
        if let Some(msg) = v
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return msg.to_string();
        }
    }
    let t = body.trim();
    if t.is_empty() {
        "Meta rejected the request".into()
    } else if t.len() > 240 {
        format!("{}…", &t[..240])
    } else {
        t.to_string()
    }
}

/// GET phone-number node — validates token + Phone Number ID without sending a message.
pub fn test_connection(phone_number_id: &str, access_token: &str) -> Result<String, GraphError> {
    let id = phone_number_id.trim();
    let token = access_token.trim();
    if id.is_empty() || token.is_empty() {
        return Err(GraphError::Auth(
            "Phone Number ID and Access Token are required".into(),
        ));
    }
    let url = format!(
        "{GRAPH_HOST}/{GRAPH_API_VERSION}/{id}?fields=display_phone_number,verified_name,quality_rating"
    );
    let resp = agent()
        .get(&url)
        .set("Authorization", &auth_header(token))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, r) => {
                let body = r.into_string().unwrap_or_default();
                classify_meta_error(code, &body)
            }
            other => GraphError::Network(other.to_string()),
        })?;
    let body = resp
        .into_string()
        .map_err(|e| GraphError::Network(e.to_string()))?;
    let v: Value =
        serde_json::from_str(&body).map_err(|e| GraphError::Rejected(e.to_string()))?;
    let display = v
        .get("display_phone_number")
        .and_then(|x| x.as_str())
        .unwrap_or(id);
    let name = v
        .get("verified_name")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    if name.is_empty() {
        Ok(format!("Connected — {display}"))
    } else {
        Ok(format!("Connected — {name} ({display})"))
    }
}

/// Upload PDF bytes; returns Meta media id.
pub fn upload_document_pdf(
    phone_number_id: &str,
    access_token: &str,
    filename: &str,
    pdf: &[u8],
) -> Result<String, GraphError> {
    let id = phone_number_id.trim();
    let token = access_token.trim();
    if id.is_empty() || token.is_empty() {
        return Err(GraphError::Auth(
            "Phone Number ID and Access Token are required".into(),
        ));
    }
    let url = format!("{GRAPH_HOST}/{GRAPH_API_VERSION}/{id}/media");
    let file_name = if filename.trim().is_empty() {
        "bill.pdf"
    } else {
        filename.trim()
    };

    // multipart/form-data by hand (ureq has no built-in multipart helper here).
    let boundary = format!("----KumkajWa{}", chrono_like_id());
    let mut body = Vec::new();
    push_multipart_field(&mut body, &boundary, "messaging_product", "whatsapp");
    push_multipart_field(&mut body, &boundary, "type", "application/pdf");
    push_multipart_file(&mut body, &boundary, "file", file_name, "application/pdf", pdf);
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let resp = agent()
        .post(&url)
        .set("Authorization", &auth_header(token))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .send_bytes(&body)
        .map_err(|e| map_ureq_uncertain(e))?;

    let text = resp
        .into_string()
        .map_err(|e| GraphError::Uncertain(e.to_string()))?;
    let v: Value = serde_json::from_str(&text)
        .map_err(|e| GraphError::Uncertain(format!("upload reply: {e}")))?;
    if let Some(media_id) = v.get("id").and_then(|x| x.as_str()).filter(|s| !s.is_empty()) {
        return Ok(media_id.to_string());
    }
    Err(GraphError::Rejected(human_meta_error(&text)))
}

fn map_ureq_uncertain(e: ureq::Error) -> GraphError {
    match e {
        ureq::Error::Status(code, r) => {
            let body = r.into_string().unwrap_or_default();
            classify_meta_error(code, &body)
        }
        // Transport failure mid-flight — treat as uncertain when we cannot know.
        other => GraphError::Network(other.to_string()),
    }
}

fn map_ureq_send(e: ureq::Error) -> GraphError {
    match e {
        ureq::Error::Status(code, r) => {
            let body = r.into_string().unwrap_or_default();
            classify_meta_error(code, &body)
        }
        other => GraphError::Uncertain(other.to_string()),
    }
}

fn push_multipart_field(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(value.as_bytes());
    body.extend_from_slice(b"\r\n");
}

fn push_multipart_file(
    body: &mut Vec<u8>,
    boundary: &str,
    name: &str,
    filename: &str,
    content_type: &str,
    data: &[u8],
) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
    body.extend_from_slice(data);
    body.extend_from_slice(b"\r\n");
}

fn chrono_like_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{n}")
}

/// Free-form document message (inside customer-service window).
pub fn send_document(
    phone_number_id: &str,
    access_token: &str,
    to_e164: &str,
    media_id: &str,
    filename: &str,
    caption: &str,
) -> Result<String, GraphError> {
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": digits_only(to_e164),
        "type": "document",
        "document": {
            "id": media_id,
            "filename": if filename.trim().is_empty() { "bill.pdf" } else { filename.trim() },
            "caption": caption,
        }
    });
    post_messages(phone_number_id, access_token, &payload)
}

/// Approved template with a document header (media id).
pub fn send_document_template(
    phone_number_id: &str,
    access_token: &str,
    to_e164: &str,
    template_name: &str,
    template_lang: &str,
    media_id: &str,
    filename: &str,
) -> Result<String, GraphError> {
    let name = template_name.trim();
    if name.is_empty() {
        return Err(GraphError::NeedsTemplate(
            "Configure a bill template name in Settings → WhatsApp".into(),
        ));
    }
    let lang = if template_lang.trim().is_empty() {
        "en"
    } else {
        template_lang.trim()
    };
    let file = if filename.trim().is_empty() {
        "bill.pdf"
    } else {
        filename.trim()
    };
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "to": digits_only(to_e164),
        "type": "template",
        "template": {
            "name": name,
            "language": { "code": lang },
            "components": [{
                "type": "header",
                "parameters": [{
                    "type": "document",
                    "document": {
                        "id": media_id,
                        "filename": file
                    }
                }]
            }]
        }
    });
    post_messages(phone_number_id, access_token, &payload)
}

fn post_messages(
    phone_number_id: &str,
    access_token: &str,
    payload: &Value,
) -> Result<String, GraphError> {
    let id = phone_number_id.trim();
    let token = access_token.trim();
    let url = format!("{GRAPH_HOST}/{GRAPH_API_VERSION}/{id}/messages");
    let resp = agent()
        .post(&url)
        .set("Authorization", &auth_header(token))
        .set("Content-Type", "application/json")
        .send_json(payload.clone())
        .map_err(map_ureq_send)?;
    let text = resp
        .into_string()
        .map_err(|e| GraphError::Uncertain(e.to_string()))?;
    parse_message_id(&text)
}

fn parse_message_id(body: &str) -> Result<String, GraphError> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| GraphError::Uncertain(format!("send reply: {e}")))?;
    if let Some(id) = v
        .pointer("/messages/0/id")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
    {
        return Ok(id.to_string());
    }
    if v.get("error").is_some() {
        return Err(classify_meta_error(400, body));
    }
    Err(GraphError::Uncertain(human_meta_error(body)))
}

/// Upload PDF then send document; on NeedsTemplate only, retry as template.
pub fn send_bill_pdf(
    creds: &WhatsappCloudCreds,
    to_e164: &str,
    filename: &str,
    pdf: &[u8],
    caption: &str,
) -> Result<String, GraphError> {
    let media_id = upload_document_pdf(
        &creds.phone_number_id,
        &creds.access_token,
        filename,
        pdf,
    )?;
    match send_document(
        &creds.phone_number_id,
        &creds.access_token,
        to_e164,
        &media_id,
        filename,
        caption,
    ) {
        Ok(mid) => Ok(mid),
        Err(e) if e.is_needs_template() => send_document_template(
            &creds.phone_number_id,
            &creds.access_token,
            to_e164,
            &creds.template_name,
            &creds.template_lang,
            &media_id,
            filename,
        ),
        Err(e) => Err(e),
    }
}

pub fn digits_only(phone: &str) -> String {
    phone.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Minimal single-page PDF from plain receipt text (Latin + replaced others).
pub fn receipt_text_to_pdf(text: &str) -> Vec<u8> {
    let lines: Vec<String> = text
        .lines()
        .map(|l| pdf_safe_line(l))
        .take(60)
        .collect();
    let mut content = String::from("BT /F1 10 Tf 40 800 Td 12 TL\n");
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            content.push_str("T*\n");
        }
        content.push('(');
        content.push_str(&pdf_escape(line));
        content.push_str(") Tj\n");
    }
    content.push_str("ET");
    let content_bytes = content.as_bytes();

    let objects: Vec<Vec<u8>> = vec![
        b"1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj\n".to_vec(),
        b"2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1 >>endobj\n".to_vec(),
        b"3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>endobj\n".to_vec(),
        {
            let mut o = format!(
                "4 0 obj<< /Length {} >>stream\n",
                content_bytes.len()
            )
            .into_bytes();
            o.extend_from_slice(content_bytes);
            o.extend_from_slice(b"\nendstream\nendobj\n");
            o
        },
        b"5 0 obj<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>endobj\n".to_vec(),
    ];

    let mut out: Vec<u8> = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize];
    for obj in &objects {
        offsets.push(out.len());
        out.extend_from_slice(obj);
    }
    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

fn pdf_safe_line(line: &str) -> String {
    line.chars()
        .map(|c| {
            if c.is_ascii() && !c.is_control() {
                c
            } else {
                '?'
            }
        })
        .take(90)
        .collect()
}

fn pdf_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// AES-256-GCM seal (key 32 bytes). Returns nonce||ciphertext as hex.
pub fn seal_secret(key: &[u8; 32], plaintext: &str) -> Result<String, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use rand::RngCore;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| "encrypt failed".to_string())?;
    let mut packed = Vec::with_capacity(12 + ct.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ct);
    Ok(hex::encode(packed))
}

pub fn open_secret(key: &[u8; 32], packed_hex: &str) -> Result<String, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    let packed = hex::decode(packed_hex.trim()).map_err(|e| e.to_string())?;
    if packed.len() < 13 {
        return Err("secret blob too short".into());
    }
    let (nonce_bytes, ct) = packed.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let pt = cipher
        .decrypt(nonce, ct)
        .map_err(|_| "decrypt failed — reconnect WhatsApp credentials".to_string())?;
    String::from_utf8(pt).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_starts_with_header() {
        let pdf = receipt_text_to_pdf("Hello\nBill 1\nTotal 10");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
    }

    #[test]
    fn seal_roundtrip() {
        let key = [7u8; 32];
        let enc = seal_secret(&key, "EAA_test_token").unwrap();
        assert_ne!(enc, "EAA_test_token");
        assert_eq!(open_secret(&key, &enc).unwrap(), "EAA_test_token");
    }

    #[test]
    fn classify_session_window() {
        let body = r#"{"error":{"message":"Re-engagement message","code":131047}}"#;
        assert!(classify_meta_error(400, body).is_needs_template());
    }

    #[test]
    fn classify_auth() {
        let body = r#"{"error":{"message":"Invalid OAuth access token","code":190}}"#;
        assert!(matches!(classify_meta_error(401, body), GraphError::Auth(_)));
    }

    #[test]
    fn digits_strip() {
        assert_eq!(digits_only("+91 98765-43210"), "919876543210");
    }
}

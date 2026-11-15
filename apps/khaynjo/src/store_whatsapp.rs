//! WhatsApp Cloud API — Server-only credentials + bill send (child of `store`).

use super::Store;
use crate::domain::{new_id, StaffMember};
use crate::notify::NotifyOutboxRow;
use crate::whatsapp_cloud::{
    self, receipt_text_to_pdf, GraphError, WhatsappCloudCreds, WhatsappCloudPublic, STATUS_ACCEPTED,
    STATUS_FAILED, STATUS_PENDING, STATUS_SENDING, STATUS_UNKNOWN, STATUS_UPLOADING,
};
use chrono::Local;
use rand::RngCore;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const META_PUBLIC: &str = "whatsapp_cloud_public";
const SECRET_FILE: &str = "whatsapp_cloud.secret";
const KEY_FILE: &str = "whatsapp_cloud.key";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PublicMeta {
    enabled: bool,
    phone_number_id: String,
    template_name: String,
    template_lang: String,
    /// Whether a token blob exists on disk (never the token itself).
    has_token: bool,
}

impl Store {
    fn whatsapp_secret_dir(&self) -> PathBuf {
        self.path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn whatsapp_key_path(&self) -> PathBuf {
        self.whatsapp_secret_dir().join(KEY_FILE)
    }

    fn whatsapp_secret_path(&self) -> PathBuf {
        self.whatsapp_secret_dir().join(SECRET_FILE)
    }

    fn load_or_create_whatsapp_key(&self) -> Result<[u8; 32], String> {
        let path = self.whatsapp_key_path();
        if path.is_file() {
            let hex = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let bytes = hex::decode(hex.trim()).map_err(|e| e.to_string())?;
            if bytes.len() != 32 {
                return Err("WhatsApp key file is damaged — Disconnect and reconnect".into());
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        fs::write(&path, hex::encode(key)).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(key)
    }

    fn read_public_meta(&self) -> Result<PublicMeta, String> {
        Ok(match self.meta_get(META_PUBLIC)? {
            Some(raw) if !raw.trim().is_empty() => {
                serde_json::from_str(&raw).unwrap_or_default()
            }
            _ => PublicMeta::default(),
        })
    }

    fn write_public_meta(&self, m: &PublicMeta) -> Result<(), String> {
        self.meta_set(
            META_PUBLIC,
            &serde_json::to_string(m).map_err(|e| e.to_string())?,
        )
    }

    /// Public view for Settings / Client UI (no access token).
    pub fn whatsapp_cloud_public(&self) -> Result<WhatsappCloudPublic, String> {
        let m = self.read_public_meta()?;
        Ok(WhatsappCloudPublic {
            enabled: m.enabled,
            phone_number_id: m.phone_number_id,
            has_token: m.has_token && self.whatsapp_secret_path().is_file(),
            template_name: m.template_name,
            template_lang: if m.template_lang.trim().is_empty() {
                "en".into()
            } else {
                m.template_lang
            },
        })
    }

    /// Full credentials — Server only. Never send this struct over LAN.
    pub fn whatsapp_cloud_creds(&self) -> Result<WhatsappCloudCreds, String> {
        self.assert_server_writer("WhatsApp Cloud credentials")?;
        let m = self.read_public_meta()?;
        let token = if self.whatsapp_secret_path().is_file() {
            let key = self.load_or_create_whatsapp_key()?;
            let packed = fs::read_to_string(self.whatsapp_secret_path())
                .map_err(|e| e.to_string())?;
            whatsapp_cloud::open_secret(&key, packed.trim())?
        } else {
            String::new()
        };
        Ok(WhatsappCloudCreds {
            enabled: m.enabled,
            phone_number_id: m.phone_number_id,
            access_token: token,
            template_name: m.template_name,
            template_lang: m.template_lang,
        })
    }

    /// Save credentials on the Server. Empty `access_token` keeps the existing token.
    pub fn set_whatsapp_cloud_creds(
        &self,
        enabled: bool,
        phone_number_id: &str,
        access_token: &str,
        template_name: &str,
        template_lang: &str,
        actor: &StaffMember,
    ) -> Result<String, String> {
        self.assert_server_writer("WhatsApp Cloud settings")?;
        if !actor.is_owner() && !actor.can_access_settings() {
            return Err("Not allowed to change WhatsApp settings".into());
        }
        let mut m = self.read_public_meta()?;
        m.enabled = enabled;
        m.phone_number_id = phone_number_id.trim().to_string();
        m.template_name = template_name.trim().to_string();
        m.template_lang = if template_lang.trim().is_empty() {
            "en".into()
        } else {
            template_lang.trim().to_string()
        };

        let token_in = access_token.trim();
        if !token_in.is_empty() {
            let key = self.load_or_create_whatsapp_key()?;
            let packed = whatsapp_cloud::seal_secret(&key, token_in)?;
            let path = self.whatsapp_secret_path();
            fs::write(&path, packed).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
            }
            m.has_token = true;
        } else {
            m.has_token = self.whatsapp_secret_path().is_file();
        }
        self.write_public_meta(&m)?;
        let _ = self.audit(
            actor,
            "whatsapp_cloud.save",
            "WhatsApp Cloud settings saved (token redacted)",
        );
        Ok("WhatsApp settings saved".into())
    }

    /// Clear token + disable. Keeps bill / outbox history.
    pub fn disconnect_whatsapp_cloud(&self, actor: &StaffMember) -> Result<String, String> {
        self.assert_server_writer("WhatsApp Cloud disconnect")?;
        if !actor.is_owner() && !actor.can_access_settings() {
            return Err("Not allowed to change WhatsApp settings".into());
        }
        let _ = fs::remove_file(self.whatsapp_secret_path());
        let _ = fs::remove_file(self.whatsapp_key_path());
        let mut m = self.read_public_meta()?;
        m.enabled = false;
        m.has_token = false;
        m.phone_number_id.clear();
        // Keep template name/lang so reconnect is easier.
        self.write_public_meta(&m)?;
        let _ = self.audit(actor, "whatsapp_cloud.disconnect", "WhatsApp Cloud disconnected");
        Ok("WhatsApp disconnected".into())
    }

    pub fn test_whatsapp_cloud(&self, actor: &StaffMember) -> Result<String, String> {
        self.assert_server_writer("WhatsApp Cloud test")?;
        if !actor.is_owner() && !actor.can_access_settings() {
            return Err("Not allowed to test WhatsApp".into());
        }
        let creds = self.whatsapp_cloud_creds()?;
        whatsapp_cloud::test_connection(&creds.phone_number_id, &creds.access_token)
            .map_err(|e| e.message().to_string())
    }

    pub fn party_whatsapp_ok(&self, party_id: &str) -> Result<bool, String> {
        let key = format!("whatsapp_ok:{}", party_id.trim());
        Ok(matches!(self.meta_get(&key)?.as_deref(), Some("1")))
    }

    pub fn set_party_whatsapp_ok(
        &self,
        party_id: &str,
        ok: bool,
        actor: &StaffMember,
    ) -> Result<String, String> {
        self.assert_server_writer("WhatsApp consent")?;
        if !actor.is_owner() && !actor.can_access_credit() && !actor.can_access_sell() {
            return Err("Not allowed to change WhatsApp consent".into());
        }
        let id = party_id.trim();
        if id.is_empty() {
            return Err("Pick a party first".into());
        }
        let key = format!("whatsapp_ok:{id}");
        self.meta_set(&key, if ok { "1" } else { "0" })?;
        Ok(if ok {
            "WhatsApp consent recorded".into()
        } else {
            "WhatsApp consent cleared".into()
        })
    }

    fn update_notify_row(
        &self,
        id: &str,
        status: &str,
        last_error: &str,
        payload_json: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE notify_outbox SET status = ?1, last_error = ?2, payload_json = ?3 WHERE id = ?4",
                params![status, last_error, payload_json, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// In-flight or settled Cloud bill send for this sale + phone (dedupe).
    pub fn whatsapp_bill_send_blocks_duplicate(
        &self,
        sale_id: &str,
        phone: &str,
    ) -> Result<Option<String>, String> {
        let phone = whatsapp_cloud::digits_only(phone);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, status, payload_json FROM notify_outbox
                 WHERE channel = 'whatsapp_cloud' AND to_addr = ?1
                 ORDER BY created_at DESC LIMIT 40",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![phone], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, status, payload) = row.map_err(|e| e.to_string())?;
            let sid = serde_json::from_str::<serde_json::Value>(&payload)
                .ok()
                .and_then(|v| {
                    v.get("sale_id")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_default();
            if sid != sale_id {
                continue;
            }
            if matches!(
                status.as_str(),
                STATUS_PENDING
                    | STATUS_UPLOADING
                    | STATUS_SENDING
                    | STATUS_ACCEPTED
                    | STATUS_UNKNOWN
            ) {
                return Ok(Some(format!(
                    "Already {status} for this bill (outbox {id}). Inspect before sending again."
                )));
            }
        }
        Ok(None)
    }

    /// Send bill PDF via Meta. Server only. Updates outbox statuses.
    /// Returns a short status line for the UI (Accepted ≠ Delivered).
    pub fn send_whatsapp_bill_pdf(
        &self,
        sale_id: &str,
        force_retry: bool,
        actor: &StaffMember,
    ) -> Result<String, String> {
        self.assert_server_writer("WhatsApp bill send")?;
        if !actor.is_owner()
            && !actor.can_access_sales()
            && !actor.can_access_sell()
            && !actor.can_access_settings()
        {
            return Err("Not allowed to send WhatsApp bills".into());
        }

        let sale = self
            .get_sale(sale_id)?
            .ok_or_else(|| "Sale not found".to_string())?;
        let party_id = sale
            .customer_id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "This bill has no customer — pick a party with phone and consent".to_string())?;
        let customers = self.list_customers()?;
        let party = customers
            .iter()
            .find(|c| c.id == party_id)
            .ok_or_else(|| "Customer not found".to_string())?;
        let phone = party.phone.trim();
        if phone.is_empty() {
            return Err("Customer has no phone number".into());
        }
        if !self.party_whatsapp_ok(&party_id)? {
            return Err("Customer has not consented to WhatsApp messages".into());
        }

        let creds = self.whatsapp_cloud_creds()?;
        if !creds.is_configured() {
            return Err("WhatsApp Cloud is not configured on the Server".into());
        }

        if !force_retry {
            if let Some(msg) = self.whatsapp_bill_send_blocks_duplicate(sale_id, phone)? {
                return Err(msg);
            }
        }

        let receipt = self.format_receipt_text(sale_id)?;
        let pdf = receipt_text_to_pdf(&receipt);
        let filename = format!("bill-{}.pdf", sale_id.chars().take(12).collect::<String>());
        let inv = sale.invoice_no.trim();
        let caption = if inv.is_empty() {
            format!("Bill {sale_id}")
        } else {
            format!("Bill {inv}")
        };

        let row_id = new_id();
        let mut payload = serde_json::json!({
            "sale_id": sale_id,
            "filename": filename,
            "intent": "whatsapp_cloud_document",
        });
        let row = NotifyOutboxRow {
            id: row_id.clone(),
            created_at: Local::now().to_rfc3339(),
            channel: "whatsapp_cloud".into(),
            to_addr: whatsapp_cloud::digits_only(phone),
            template_key: "e_bill_pdf".into(),
            payload_json: payload.to_string(),
            status: STATUS_PENDING.into(),
            last_error: String::new(),
        };
        self.queue_notify(&row)?;

        let mut set = |status: &str, err: &str, extra: serde_json::Value| {
            payload = serde_json::json!({
                "sale_id": sale_id,
                "filename": filename,
                "intent": "whatsapp_cloud_document",
                "extra": extra,
            });
            self.update_notify_row(&row_id, status, err, &payload.to_string())
        };

        set(STATUS_UPLOADING, "", serde_json::json!({}))?;

        let media_result = whatsapp_cloud::upload_document_pdf(
            &creds.phone_number_id,
            &creds.access_token,
            &filename,
            &pdf,
        );
        let media_id = match media_result {
            Ok(id) => id,
            Err(GraphError::Network(m)) | Err(GraphError::Uncertain(m)) => {
                let _ = set(STATUS_UNKNOWN, &m, serde_json::json!({}));
                return Err(format!(
                    "WhatsApp outcome unknown after upload — inspect outbox before retry ({m})"
                ));
            }
            Err(e) => {
                let _ = set(STATUS_FAILED, e.message(), serde_json::json!({}));
                return Err(e.message().to_string());
            }
        };

        set(
            STATUS_SENDING,
            "",
            serde_json::json!({ "media_id": media_id }),
        )?;

        let send_result = match whatsapp_cloud::send_document(
            &creds.phone_number_id,
            &creds.access_token,
            phone,
            &media_id,
            &filename,
            &caption,
        ) {
            Ok(mid) => Ok(mid),
            Err(e) if e.is_needs_template() => whatsapp_cloud::send_document_template(
                &creds.phone_number_id,
                &creds.access_token,
                phone,
                &creds.template_name,
                &creds.template_lang,
                &media_id,
                &filename,
            ),
            Err(e) => Err(e),
        };

        match send_result {
            Ok(message_id) => {
                set(
                    STATUS_ACCEPTED,
                    "",
                    serde_json::json!({ "media_id": media_id, "message_id": message_id }),
                )?;
                let _ = self.audit(
                    actor,
                    "whatsapp_cloud.bill_accepted",
                    &format!("sale={sale_id} message_id={message_id}"),
                );
                Ok(format!(
                    "WhatsApp accepted the bill (not delivery confirmation) · {message_id}"
                ))
            }
            Err(GraphError::Uncertain(m)) | Err(GraphError::Network(m)) => {
                let _ = set(
                    STATUS_UNKNOWN,
                    &m,
                    serde_json::json!({ "media_id": media_id }),
                );
                Err(format!(
                    "WhatsApp outcome unknown — inspect outbox before retry ({m})"
                ))
            }
            Err(e) => {
                let _ = set(
                    STATUS_FAILED,
                    e.message(),
                    serde_json::json!({ "media_id": media_id }),
                );
                Err(e.message().to_string())
            }
        }
    }
}

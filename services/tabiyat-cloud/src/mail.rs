//! Product confirmation email via Amazon SES (Tabiyat only — not a shared Kumkaj mailer).
//! Hierarchy on the mail: Company → Brand (Kumkaj) → Product → Plan.
//! Best-effort: failures are logged; billing activation must not roll back on mail errors.

use aws_config::BehaviorVersion;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client as SesClient;

#[derive(Debug, Clone)]
pub struct MailConfig {
    pub enabled: bool,
    pub from: String,
    pub company_name: String,
    pub brand_name: String,
    pub product_name: String,
    pub account_url: String,
}

impl MailConfig {
    pub fn from_env() -> Self {
        let enabled = matches!(
            std::env::var("MAIL_ENABLED").unwrap_or_else(|_| "0".into()).as_str(),
            "1" | "true" | "yes" | "on"
        );
        Self {
            enabled,
            from: std::env::var("MAIL_FROM")
                .unwrap_or_else(|_| "noreply@tabiyat.shop".into()),
            company_name: std::env::var("MAIL_COMPANY_NAME").unwrap_or_else(|_| {
                "GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED".into()
            }),
            brand_name: std::env::var("MAIL_BRAND_NAME")
                .unwrap_or_else(|_| "Kumkaj".into()),
            product_name: std::env::var("MAIL_PRODUCT_NAME")
                .unwrap_or_else(|_| "Tabiyat".into()),
            account_url: std::env::var("MAIL_ACCOUNT_URL")
                .unwrap_or_else(|_| "https://tabiyat.shop/account".into()),
        }
    }
}

pub async fn send_plan_activated(
    cfg: &MailConfig,
    to: &str,
    plan_title: &str,
    ends_at_iso: &str,
) -> Result<(), String> {
    if !cfg.enabled {
        tracing::info!(%to, "MAIL_ENABLED off — skip plan confirmation");
        return Ok(());
    }
    if to.trim().is_empty() || !to.contains('@') {
        return Err("missing recipient email".into());
    }
    let subject = format!(
        "{} · {} — {} is active",
        cfg.brand_name, cfg.product_name, plan_title
    );
    let text = format!(
        "{company}\n\
{brand}\n\
{product}\n\
{plan}\n\n\
Your plan is active until {ends}.\n\n\
Manage your account: {url}\n",
        company = cfg.company_name,
        brand = cfg.brand_name,
        product = cfg.product_name,
        plan = plan_title,
        ends = ends_at_iso,
        url = cfg.account_url,
    );
    // Prefer env/profile (local AWS_PROFILE=gotigin) or ECS task role.
    // Region must match where identities were created (ap-south-1).
    let region = std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "ap-south-1".into());
    let aws_cfg = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_config::Region::new(region))
        .load()
        .await;
    let client = SesClient::new(&aws_cfg);
    client
        .send_email()
        .from_email_address(&cfg.from)
        .destination(Destination::builder().to_addresses(to).build())
        .content(
            EmailContent::builder()
                .simple(
                    Message::builder()
                        .subject(
                            Content::builder()
                                .data(subject)
                                .charset("UTF-8")
                                .build()
                                .map_err(|e| e.to_string())?,
                        )
                        .body(
                            Body::builder()
                                .text(
                                    Content::builder()
                                        .data(text)
                                        .charset("UTF-8")
                                        .build()
                                        .map_err(|e| e.to_string())?,
                                )
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .map_err(|e| format!("SES send failed: {e:?}"))?;
    tracing::info!(%to, from = %cfg.from, "sent plan confirmation email");
    Ok(())
}

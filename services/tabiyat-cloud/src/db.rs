use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::auth::FirebaseClaims;

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[derive(Debug, Clone)]
pub struct Account {
    pub account_id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub display_name: Option<String>,
    pub email_verified_hint: bool,
}

#[derive(Debug, Clone)]
pub struct PlanRow {
    pub plan_id: String,
    pub sort_order: i32,
    pub title: String,
    pub quota_gb: Option<i32>,
    pub bandwidth_band: String,
    pub price_inr_paise: Option<i64>,
    pub includes_live_cloud: bool,
}

#[derive(Debug, Clone)]
pub struct SubscriptionRow {
    pub plan_id: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub title: String,
    pub quota_gb: Option<i32>,
    pub bandwidth_band: String,
    pub price_inr_paise: Option<i64>,
    pub includes_live_cloud: bool,
}

pub async fn ensure_account(
    pool: &PgPool,
    product_code: &str,
    auth_issuer: &str,
    claims: &FirebaseClaims,
) -> Result<Account, sqlx::Error> {
    if let Some(existing) = find_by_firebase_uid(pool, &claims.uid).await? {
        return Ok(Account {
            email_verified_hint: claims.email_verified,
            ..existing
        });
    }

    let account_id = Uuid::new_v4();
    let username = unique_username(pool, claims).await?;
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO accounts (
            account_id, firebase_uid, username, email, phone, display_name,
            product_code, auth_issuer
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(account_id)
    .bind(&claims.uid)
    .bind(&username)
    .bind(&claims.email)
    .bind(&claims.phone)
    .bind(&claims.name)
    .bind(product_code)
    .bind(auth_issuer)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO subscriptions (account_id, plan_id, status)
        VALUES ($1, 'free', 'active')
        "#,
    )
    .bind(account_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Account {
        account_id,
        username,
        email: claims.email.clone(),
        phone: claims.phone.clone(),
        display_name: claims.name.clone(),
        email_verified_hint: claims.email_verified,
    })
}

async fn find_by_firebase_uid(
    pool: &PgPool,
    firebase_uid: &str,
) -> Result<Option<Account>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT account_id, username, email, phone, display_name
        FROM accounts WHERE firebase_uid = $1
        "#,
    )
    .bind(firebase_uid)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Account {
        account_id: r.get("account_id"),
        username: r.get("username"),
        email: r.get("email"),
        phone: r.get("phone"),
        display_name: r.get("display_name"),
        email_verified_hint: false,
    }))
}

async fn unique_username(pool: &PgPool, claims: &FirebaseClaims) -> Result<String, sqlx::Error> {
    let base = username_base(claims);
    for n in 0..50 {
        let candidate = if n == 0 {
            base.clone()
        } else {
            format!("{base}{n}")
        };
        let taken: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM accounts WHERE username = $1)")
            .bind(&candidate)
            .fetch_one(pool)
            .await?;
        if !taken {
            return Ok(candidate);
        }
    }
    Ok(format!("user_{}", &Uuid::new_v4().to_string()[..8]))
}

fn username_base(claims: &FirebaseClaims) -> String {
    let raw = claims
        .email
        .as_deref()
        .and_then(|e| e.split('@').next())
        .or(claims.phone.as_deref())
        .or(claims.name.as_deref())
        .unwrap_or("user");
    let mut s: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    while s.contains("__") {
        s = s.replace("__", "_");
    }
    let s = s.trim_matches('_').to_string();
    if s.is_empty() {
        "user".into()
    } else {
        s.chars().take(32).collect()
    }
}

pub async fn list_plans(pool: &PgPool) -> Result<Vec<PlanRow>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT plan_id, sort_order, title, quota_gb, bandwidth_band,
               price_inr_paise, includes_live_cloud
        FROM plans WHERE active = TRUE
        ORDER BY sort_order
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PlanRow {
            plan_id: r.get("plan_id"),
            sort_order: r.get("sort_order"),
            title: r.get("title"),
            quota_gb: r.get("quota_gb"),
            bandwidth_band: r.get("bandwidth_band"),
            price_inr_paise: r.get("price_inr_paise"),
            includes_live_cloud: r.get("includes_live_cloud"),
        })
        .collect())
}

pub async fn subscription_for(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<SubscriptionRow>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT s.plan_id, s.status, s.started_at, s.ends_at,
               p.title, p.quota_gb, p.bandwidth_band, p.price_inr_paise, p.includes_live_cloud
        FROM subscriptions s
        JOIN plans p ON p.plan_id = s.plan_id
        WHERE s.account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| SubscriptionRow {
        plan_id: r.get("plan_id"),
        status: r.get("status"),
        started_at: r.get("started_at"),
        ends_at: r.get("ends_at"),
        title: r.get("title"),
        quota_gb: r.get("quota_gb"),
        bandwidth_band: r.get("bandwidth_band"),
        price_inr_paise: r.get("price_inr_paise"),
        includes_live_cloud: r.get("includes_live_cloud"),
    }))
}

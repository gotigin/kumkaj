//! Kirano Cloud API — Auth account + plans + sandbox billing (Stripe/Paddle/mock).
//! Own Postgres; clients never talk to the DB. Firebase Auth: `kirano-shop`.
//! Stripe/Paddle ops: Chrome profile **MEARAJ (Gotigin)**.

mod auth;
mod billing;
mod db;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use billing::BillingConfig;

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub product_code: &'static str,
    pub firebase_project_id: String,
    pub verifier: auth::FirebaseVerifier,
    pub billing: Option<BillingConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is required (Postgres for Kirano cloud only)")?;
    let firebase_project_id =
        std::env::var("FIREBASE_PROJECT_ID").unwrap_or_else(|_| "kirano-shop".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let pool = db::connect_and_migrate(&database_url).await?;
    let verifier = auth::FirebaseVerifier::new(firebase_project_id.clone());
    let billing_cfg = BillingConfig::from_env();
    if billing_cfg.enabled {
        tracing::info!(
            mode = billing_cfg.mode.as_str(),
            default_provider = billing_cfg.default_provider.as_str(),
            providers = ?billing_cfg.enabled_providers(),
            "billing enabled"
        );
    } else {
        tracing::info!("billing disabled (set BILLING_ENABLED=1 for sandbox/live checkout)");
    }

    let state = Arc::new(AppState {
        pool,
        product_code: "kirano",
        firebase_project_id,
        verifier,
        billing: Some(billing_cfg),
    });

    let app = Router::new()
        .merge(routes::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(%addr, product = "kirano", "kirano-cloud listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown");
}

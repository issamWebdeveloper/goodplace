//! GoodPlace API — site vitrine et blog d'Issam Gharsallah.

mod app;
mod cache;
mod config;
mod error;
mod mail;
mod markdown;
mod modules;
mod scheduler;
mod seed;

use std::sync::Arc;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use crate::{app::AppState, cache::AppCache, config::Config, mail::Mailer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("goodplace_api=info,tower_http=info")),
        )
        .init();

    let config = Config::from_env()?;

    let db = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .context("connexion à PostgreSQL impossible")?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        cache: AppCache::new(),
        mailer: Mailer::new(&config)?,
        config: Arc::new(config),
    };

    modules::auth::service::ensure_admin(&state).await?;
    if state.config.seed_content {
        seed::run(&state.db).await?;
    }
    // Rattrapage immédiat des publications programmées manquées pendant un arrêt.
    modules::articles::service::publish_scheduled(&state).await?;
    let _scheduler = scheduler::start(state.clone()).await?;

    let listener = tokio::net::TcpListener::bind(&state.config.bind_addr).await?;
    tracing::info!("API en écoute sur http://{}", state.config.bind_addr);
    axum::serve(listener, app::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

//! État partagé et assemblage du routeur HTTP.

use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

use crate::{cache::AppCache, config::Config, mail::Mailer, modules};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cache: AppCache,
    pub mailer: Mailer,
    pub config: Arc<Config>,
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(|| async { Json(json!({ "status": "ok" })) }))
        .nest("/auth", modules::auth::routes::router())
        .nest("/account", modules::users::routes::account_router())
        .nest("/profile", modules::profile::router())
        .nest("/articles", modules::articles::routes::public_router())
        .nest("/admin/articles", modules::articles::routes::admin_router())
        .nest("/admin/users", modules::users::routes::admin_router());

    let mut app = Router::new()
        .nest("/api", api)
        .merge(modules::seo::router())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    // En production, le front et l'API partagent la même origine (proxy) : CORS inutile.
    if !state.config.cors_origins.is_empty() {
        let origins: Vec<HeaderValue> = state
            .config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        app = app.layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE]),
        );
    }

    app.with_state(state)
}

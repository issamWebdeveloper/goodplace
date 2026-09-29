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
    cors::{AllowOrigin, CorsLayer},
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
    // Lecture publique : seules routes ouvertes à d'autres sites (ex. gharsallah.fr).
    let public = Router::new()
        .nest("/profile", modules::profile::router())
        .nest("/articles", modules::articles::routes::public_router())
        .layer(public_cors(&state.config.cors_origins));

    let api = Router::new()
        .route("/health", get(|| async { Json(json!({ "status": "ok" })) }))
        .nest("/auth", modules::auth::routes::router())
        .nest("/account", modules::users::routes::account_router())
        .nest("/admin/articles", modules::articles::routes::admin_router())
        .nest("/admin/users", modules::users::routes::admin_router())
        .merge(public);

    Router::new()
        .nest("/api", api)
        .merge(modules::seo::router())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// CORS limité aux lectures publiques : GET uniquement, sans cookie.
/// Le front et l'API partagent la même origine (proxy) : les routes authentifiées n'en ont pas besoin.
fn public_cors(origins: &[String]) -> CorsLayer {
    let origins: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET])
}

//! Profil / CV (format JSON Resume) affiché sur la page d'accueil.

use axum::{
    http::header,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::app::AppState;

/// CV embarqué dans le binaire : une mise à jour du fichier nécessite une recompilation,
/// mais la réponse ne coûte ainsi aucune entrée/sortie.
const PROFILE_JSON: &str = include_str!("../../../data/profile.json");

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(profile))
}

async fn profile() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        PROFILE_JSON,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn profile_is_valid_json() {
        let value: serde_json::Value = serde_json::from_str(super::PROFILE_JSON).unwrap();
        assert!(value["basics"]["name"].is_string());
    }
}

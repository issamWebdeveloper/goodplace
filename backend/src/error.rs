//! Type d'erreur unique de l'API, converti en réponse JSON `{ "error": "..." }`.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("Authentification requise")]
    Unauthorized,
    #[error("Identifiants invalides")]
    InvalidCredentials,
    #[error("Accès refusé")]
    Forbidden,
    #[error("Ressource introuvable")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
    #[error("Lien invalide ou expiré")]
    InvalidToken,
    #[error("Trop de demandes, réessayez dans une minute")]
    TooManyRequests,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::InvalidToken => StatusCode::BAD_REQUEST,
            Self::Unauthorized | Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = if status.is_server_error() {
            tracing::error!(error = ?self, "erreur interne");
            "Erreur interne du serveur".to_string()
        } else if matches!(self, Self::Database(sqlx::Error::RowNotFound)) {
            Self::NotFound.to_string()
        } else {
            self.to_string()
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

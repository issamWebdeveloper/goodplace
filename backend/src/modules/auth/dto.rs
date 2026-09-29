use serde::{Deserialize, Serialize};

use crate::modules::users::model::{Role, UserView};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct EmailRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}

impl MessageResponse {
    pub fn new(message: &'static str) -> Self {
        Self { message }
    }
}

#[derive(Serialize)]
pub struct VerifyEmailResponse {
    pub role: Role,
    /// Présent pour un utilisateur standard : jeton (10 min) permettant de définir son mot de passe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_password_token: Option<String>,
    /// Présent si la vérification a ouvert une session (abonné).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserView>,
}

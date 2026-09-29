use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
    Subscriber,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: Role,
    pub password_hash: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub session_version: i32,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Représentation publique d'un utilisateur (jamais de hash de mot de passe).
#[derive(Debug, Serialize)]
pub struct UserView {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: Role,
    pub email_verified: bool,
    pub is_active: bool,
    pub has_password: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<&User> for UserView {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            name: u.name.clone(),
            role: u.role,
            email_verified: u.email_verified_at.is_some(),
            is_active: u.is_active,
            has_password: u.password_hash.is_some(),
            last_login_at: u.last_login_at,
            created_at: u.created_at,
        }
    }
}

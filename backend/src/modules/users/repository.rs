//! Accès SQL à la table `users`.

use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Role, User};

const COLUMNS: &str = "id, email, name, role, password_hash, email_verified_at, is_active, \
                       session_version, last_login_at, created_at";

pub async fn find_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_email(db: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE lower(email) = lower($1)"))
        .bind(email)
        .fetch_optional(db)
        .await
}

pub async fn create(db: &PgPool, email: &str, name: Option<&str>, role: Role) -> sqlx::Result<User> {
    sqlx::query_as(&format!(
        "INSERT INTO users (email, name, role) VALUES ($1, $2, $3) RETURNING {COLUMNS}"
    ))
    .bind(email)
    .bind(name)
    .bind(role)
    .fetch_one(db)
    .await
}

pub async fn create_admin(db: &PgPool, email: &str, password_hash: &str) -> sqlx::Result<User> {
    sqlx::query_as(&format!(
        "INSERT INTO users (email, name, role, password_hash, email_verified_at, is_active)
         VALUES ($1, 'Administrateur', 'admin', $2, now(), TRUE) RETURNING {COLUMNS}"
    ))
    .bind(email)
    .bind(password_hash)
    .fetch_one(db)
    .await
}

pub async fn find_admin(db: &PgPool) -> sqlx::Result<Option<User>> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE role = 'admin'"))
        .fetch_optional(db)
        .await
}

pub async fn mark_verified(db: &PgPool, id: Uuid) -> sqlx::Result<User> {
    sqlx::query_as(&format!(
        "UPDATE users SET email_verified_at = COALESCE(email_verified_at, now()), is_active = TRUE,
                updated_at = now()
         WHERE id = $1 RETURNING {COLUMNS}"
    ))
    .bind(id)
    .fetch_one(db)
    .await
}

/// Change le mot de passe et invalide toutes les sessions ouvertes.
pub async fn set_password(db: &PgPool, id: Uuid, password_hash: &str) -> sqlx::Result<User> {
    sqlx::query_as(&format!(
        "UPDATE users SET password_hash = $2, session_version = session_version + 1, updated_at = now()
         WHERE id = $1 RETURNING {COLUMNS}"
    ))
    .bind(id)
    .bind(password_hash)
    .fetch_one(db)
    .await
}

pub async fn update_name(db: &PgPool, id: Uuid, name: Option<&str>) -> sqlx::Result<User> {
    sqlx::query_as(&format!(
        "UPDATE users SET name = $2, updated_at = now() WHERE id = $1 RETURNING {COLUMNS}"
    ))
    .bind(id)
    .bind(name)
    .fetch_one(db)
    .await
}

pub async fn touch_login(db: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET last_login_at = now() WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn list(db: &PgPool, role: Option<Role>) -> sqlx::Result<Vec<User>> {
    sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM users WHERE ($1::user_role IS NULL OR role = $1) ORDER BY created_at DESC"
    ))
    .bind(role)
    .fetch_all(db)
    .await
}

pub async fn delete(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let res = sqlx::query("DELETE FROM users WHERE id = $1 AND role <> 'admin'")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Emails des abonnés confirmés (destinataires de la newsletter).
pub async fn subscriber_emails(db: &PgPool) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar(
        "SELECT email FROM users WHERE role = 'subscriber' AND is_active AND email_verified_at IS NOT NULL",
    )
    .fetch_all(db)
    .await
}

pub async fn count_by_role(db: &PgPool) -> sqlx::Result<Vec<(Role, i64)>> {
    sqlx::query_as("SELECT role, COUNT(*) FROM users WHERE is_active GROUP BY role")
        .fetch_all(db)
        .await
}

//! Jetons à usage unique envoyés par email (vérification, lien magique, mot de passe).

use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "token_purpose", rename_all = "snake_case")]
pub enum Purpose {
    VerifyEmail,
    SetPassword,
    ResetPassword,
    MagicLogin,
}

impl Purpose {
    fn as_str(self) -> &'static str {
        match self {
            Purpose::VerifyEmail => "verify_email",
            Purpose::SetPassword => "set_password",
            Purpose::ResetPassword => "reset_password",
            Purpose::MagicLogin => "magic_login",
        }
    }

    /// Durée de validité : 10 minutes pour tout ce qui donne accès au compte.
    pub fn ttl(self) -> Duration {
        match self {
            Purpose::VerifyEmail => Duration::hours(24),
            Purpose::SetPassword | Purpose::ResetPassword | Purpose::MagicLogin => {
                Duration::minutes(10)
            }
        }
    }
}

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Crée un jeton et retourne sa valeur en clair (seul le hash est persisté).
/// Les jetons précédents de même usage pour cet utilisateur sont révoqués.
pub async fn create(db: &PgPool, user_id: Uuid, purpose: Purpose) -> sqlx::Result<String> {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = hex::encode(bytes);

    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM email_tokens WHERE user_id = $1 AND purpose = $2 AND used_at IS NULL")
        .bind(user_id)
        .bind(purpose)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO email_tokens (user_id, purpose, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(purpose)
    .bind(hash(&token))
    .bind(Utc::now() + purpose.ttl())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(token)
}

/// Consomme un jeton valide (non expiré, non utilisé, d'un des usages acceptés)
/// et retourne l'utilisateur associé ainsi que l'usage du jeton.
pub async fn consume(
    db: &PgPool,
    token: &str,
    accepted: &[Purpose],
) -> sqlx::Result<Option<(Uuid, Purpose)>> {
    if token.len() != 64 {
        return Ok(None);
    }
    let accepted: Vec<&str> = accepted.iter().map(|p| p.as_str()).collect();
    sqlx::query_as(
        "UPDATE email_tokens SET used_at = now()
         WHERE token_hash = $1 AND purpose::text = ANY($2) AND used_at IS NULL AND expires_at > now()
         RETURNING user_id, purpose",
    )
    .bind(hash(token))
    .bind(accepted)
    .fetch_optional(db)
    .await
}

/// Nettoyage périodique (appelé par le planificateur).
pub async fn purge_expired(db: &PgPool) -> sqlx::Result<u64> {
    let res = sqlx::query(
        "DELETE FROM email_tokens WHERE expires_at < now() - interval '1 day' OR used_at < now() - interval '1 day'",
    )
    .execute(db)
    .await?;
    Ok(res.rows_affected())
}

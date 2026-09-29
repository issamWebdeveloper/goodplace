//! Hachage des mots de passe (Argon2id).

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::{AppError, AppResult};

pub const MIN_LENGTH: usize = 10;

pub fn validate(password: &str) -> AppResult<()> {
    let long_enough = password.chars().count() >= MIN_LENGTH;
    let has_letter = password.chars().any(char::is_alphabetic);
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !(long_enough && has_letter && has_digit) {
        return Err(AppError::bad_request(format!(
            "Le mot de passe doit contenir au moins {MIN_LENGTH} caractères, dont des lettres et des chiffres"
        )));
    }
    if password.len() > 256 {
        return Err(AppError::bad_request("Mot de passe trop long"));
    }
    Ok(())
}

/// Le hachage est coûteux en CPU : on l'exécute hors du runtime async.
pub async fn hash(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| anyhow::anyhow!("hachage impossible : {e}"))
    })
    .await
    .map_err(anyhow::Error::from)?
    .map_err(AppError::from)
}

pub async fn verify(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

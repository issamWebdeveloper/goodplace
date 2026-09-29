//! Session : JWT signé stocké dans un cookie HttpOnly.

use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::Config, modules::users::model::User};

pub const COOKIE_NAME: &str = "gp_session";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    /// Version de session : doit correspondre à `users.session_version`.
    pub sv: i32,
    pub exp: i64,
    pub iat: i64,
}

pub fn issue(config: &Config, user: &User) -> anyhow::Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user.id,
        sv: user.session_version,
        iat: now.timestamp(),
        exp: (now + Duration::days(config.session_days)).timestamp(),
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?)
}

pub fn decode_token(config: &Config, token: &str) -> Option<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}

pub fn cookie(config: &Config, token: String) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, token))
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time_duration(config.session_days))
        .build()
}

pub fn removal_cookie(config: &Config) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, ""))
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time_duration(0))
        .build()
}

fn time_duration(days: i64) -> time::Duration {
    time::Duration::days(days)
}

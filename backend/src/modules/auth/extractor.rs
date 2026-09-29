//! Extracteurs Axum : `CurrentUser` (toute session valide), `AdminUser` (administrateur).

use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::{
    app::AppState,
    error::AppError,
    modules::users::{
        model::{Role, User},
        repository,
    },
};

use super::session;

pub struct CurrentUser(pub Arc<User>);

pub struct AdminUser(pub Arc<User>);

/// Session facultative (visiteur anonyme accepté).
pub struct MaybeUser(pub Option<Arc<User>>);

async fn resolve(parts: &mut Parts, state: &AppState) -> Result<Option<Arc<User>>, AppError> {
    let jar = CookieJar::from_headers(&parts.headers);
    let Some(token) = jar.get(session::COOKIE_NAME).map(|c| c.value().to_string()) else {
        return Ok(None);
    };
    let Some(claims) = session::decode_token(&state.config, &token) else {
        return Ok(None);
    };

    let user = match state.cache.users.get(&claims.sub).await {
        Some(user) => user,
        None => {
            let Some(user) = repository::find_by_id(&state.db, claims.sub).await? else {
                return Ok(None);
            };
            let user = Arc::new(user);
            state.cache.users.insert(user.id, user.clone()).await;
            user
        }
    };

    let valid = user.is_active && user.session_version == claims.sv;
    Ok(valid.then_some(user))
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        resolve(parts, state).await?.map(CurrentUser).ok_or(AppError::Unauthorized)
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = resolve(parts, state).await?.ok_or(AppError::Unauthorized)?;
        if user.role != Role::Admin {
            return Err(AppError::Forbidden);
        }
        Ok(AdminUser(user))
    }
}

impl FromRequestParts<AppState> for MaybeUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(MaybeUser(resolve(parts, state).await?))
    }
}

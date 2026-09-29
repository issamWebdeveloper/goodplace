use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    modules::auth::{
        extractor::{AdminUser, CurrentUser},
        session,
    },
};

use super::{
    model::{Role, UserView},
    repository,
};

/// Espace compte de l'utilisateur connecté.
pub fn account_router() -> Router<AppState> {
    Router::new().route("/", patch(update_account).delete(delete_account))
}

/// Gestion des utilisateurs par l'administrateur.
pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", delete(delete_user))
}

#[derive(Deserialize)]
struct UpdateAccount {
    name: Option<String>,
}

async fn update_account(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<UpdateAccount>,
) -> AppResult<Json<UserView>> {
    let name = body.name.as_deref().map(str::trim).filter(|n| !n.is_empty());
    if name.is_some_and(|n| n.chars().count() > 80) {
        return Err(AppError::bad_request("Nom trop long (80 caractères maximum)"));
    }
    let updated = repository::update_name(&state.db, user.id, name).await?;
    state.cache.invalidate_user(user.id).await;
    Ok(Json(UserView::from(&updated)))
}

/// Suppression du compte (ou désinscription de la newsletter pour un abonné).
async fn delete_account(
    State(state): State<AppState>,
    jar: CookieJar,
    CurrentUser(user): CurrentUser,
) -> AppResult<(CookieJar, StatusCode)> {
    if user.role == Role::Admin {
        return Err(AppError::Forbidden);
    }
    repository::delete(&state.db, user.id).await?;
    state.cache.invalidate_user(user.id).await;
    Ok((jar.add(session::removal_cookie(&state.config)), StatusCode::NO_CONTENT))
}

#[derive(Deserialize)]
struct ListQuery {
    role: Option<Role>,
}

async fn list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<UserView>>> {
    let users = repository::list(&state.db, query.role).await?;
    Ok(Json(users.iter().map(UserView::from).collect()))
}

async fn delete_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    if !repository::delete(&state.db, id).await? {
        return Err(AppError::NotFound);
    }
    state.cache.invalidate_user(id).await;
    Ok(StatusCode::NO_CONTENT)
}

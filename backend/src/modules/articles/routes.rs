use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    markdown,
    modules::{auth::extractor::AdminUser, users::repository as users},
};

use super::{
    dto::*,
    model::{Article, ArticlePage, ArticleStatus},
    repository, service,
};

const PUBLIC_CACHE: &str = "public, max-age=60, stale-while-revalidate=300";

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_published))
        .route("/tags", get(tags))
        .route("/{slug}", get(get_published))
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/", get(admin_list).post(admin_create))
        .route("/preview", post(preview))
        .route("/stats", get(stats))
        .route("/{id}", get(admin_get).put(admin_update).delete(admin_delete))
        .route("/{id}/validate", post(admin_validate))
        .route("/{id}/publish", post(admin_publish))
        .route("/{id}/unpublish", post(admin_unpublish))
}

// ---------- Public ----------

async fn list_published(
    State(state): State<AppState>,
    Query(q): Query<PublicListQuery>,
) -> AppResult<impl IntoResponse> {
    let per_page = q.per_page.unwrap_or(9).clamp(1, 50);
    let page = q.page.unwrap_or(1).max(1);
    let tag = q.tag.as_deref().map(str::trim).filter(|t| !t.is_empty());
    let key = format!("{page}:{per_page}:{}", tag.unwrap_or(""));

    let result = match state.cache.article_pages.get(&key).await {
        Some(hit) => hit,
        None => {
            let (items, total) =
                repository::list_published(&state.db, tag, per_page, (page - 1) * per_page).await?;
            let page = Arc::new(ArticlePage {
                items,
                page,
                per_page,
                total,
                total_pages: (total + per_page - 1) / per_page,
            });
            state.cache.article_pages.insert(key, page.clone()).await;
            page
        }
    };
    Ok(([(header::CACHE_CONTROL, PUBLIC_CACHE)], Json(result)))
}

async fn get_published(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<impl IntoResponse> {
    let article = match state.cache.articles.get(&slug).await {
        Some(hit) => hit,
        None => {
            let article = Arc::new(
                repository::find_published(&state.db, &slug).await?.ok_or(AppError::NotFound)?,
            );
            state.cache.articles.insert(slug, article.clone()).await;
            article
        }
    };
    Ok(([(header::CACHE_CONTROL, PUBLIC_CACHE)], Json(article)))
}

async fn tags(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    Ok(([(header::CACHE_CONTROL, PUBLIC_CACHE)], Json(repository::tag_counts(&state.db).await?)))
}

// ---------- Administration ----------

async fn admin_list(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(q): Query<AdminListQuery>,
) -> AppResult<Json<Vec<Article>>> {
    Ok(Json(repository::list_all(&state.db, q.status).await?))
}

async fn admin_get(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Article>> {
    Ok(Json(repository::find_by_id(&state.db, id).await?.ok_or(AppError::NotFound)?))
}

async fn admin_create(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Json(input): Json<ArticleInput>,
) -> AppResult<(StatusCode, Json<Article>)> {
    let article = service::create(&state, input, admin.id).await?;
    Ok((StatusCode::CREATED, Json(article)))
}

async fn admin_update(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ArticleInput>,
) -> AppResult<Json<Article>> {
    Ok(Json(service::update(&state, id, input).await?))
}

async fn admin_delete(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    service::delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn admin_validate(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ValidateInput>,
) -> AppResult<Json<Article>> {
    Ok(Json(service::validate_article(&state, id, input.scheduled_at).await?))
}

async fn admin_publish(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Article>> {
    Ok(Json(service::publish_now(&state, id).await?))
}

async fn admin_unpublish(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Article>> {
    Ok(Json(service::unpublish(&state, id).await?))
}

async fn preview(_admin: AdminUser, Json(input): Json<PreviewInput>) -> Json<serde_json::Value> {
    Json(json!({
        "html": markdown::render(&input.content_md),
        "reading_minutes": markdown::reading_minutes(&input.content_md),
    }))
}

#[derive(Serialize, Default)]
struct Stats {
    drafts: i64,
    validated: i64,
    published: i64,
    users: i64,
    subscribers: i64,
}

async fn stats(State(state): State<AppState>, _admin: AdminUser) -> AppResult<Json<Stats>> {
    let mut stats = Stats::default();
    for (status, n) in repository::count_by_status(&state.db).await? {
        match status {
            ArticleStatus::Draft => stats.drafts = n,
            ArticleStatus::Validated => stats.validated = n,
            ArticleStatus::Published => stats.published = n,
        }
    }
    for (role, n) in users::count_by_role(&state.db).await? {
        match role {
            crate::modules::users::model::Role::User => stats.users = n,
            crate::modules::users::model::Role::Subscriber => stats.subscribers = n,
            crate::modules::users::model::Role::Admin => {}
        }
    }
    Ok(Json(stats))
}

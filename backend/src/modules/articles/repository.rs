//! Accès SQL à la table `articles`.

use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use super::model::{Article, ArticleStatus, ArticleSummary, PublicArticle, TagCount};

const COLUMNS: &str = "id, slug, title, excerpt, content_md, content_html, cover_image, tags, \
                       reading_minutes, status, scheduled_at, published_at, newsletter_sent_at, \
                       seo_title, seo_description, author_id, created_at, updated_at";

/// Champs calculés prêts à être enregistrés.
pub struct ArticleWrite<'a> {
    pub slug: &'a str,
    pub title: &'a str,
    pub excerpt: &'a str,
    pub content_md: &'a str,
    pub content_html: &'a str,
    pub cover_image: Option<&'a str>,
    pub tags: &'a [String],
    pub reading_minutes: i32,
    pub seo_title: Option<&'a str>,
    pub seo_description: Option<&'a str>,
}

pub async fn list_published(
    db: &PgPool,
    tag: Option<&str>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<(Vec<ArticleSummary>, i64)> {
    let items = sqlx::query_as(
        "SELECT slug, title, excerpt, cover_image, tags, reading_minutes, published_at
         FROM articles
         WHERE status = 'published' AND ($1::text IS NULL OR $1 = ANY(tags))
         ORDER BY published_at DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(tag)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;
    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM articles WHERE status = 'published' AND ($1::text IS NULL OR $1 = ANY(tags))",
    )
    .bind(tag)
    .fetch_one(db)
    .await?;
    Ok((items, total))
}

pub async fn find_published(db: &PgPool, slug: &str) -> sqlx::Result<Option<PublicArticle>> {
    sqlx::query_as(
        "SELECT slug, title, excerpt, content_html, cover_image, tags, reading_minutes,
                published_at, updated_at, seo_title, seo_description
         FROM articles WHERE slug = $1 AND status = 'published'",
    )
    .bind(slug)
    .fetch_optional(db)
    .await
}

pub async fn tag_counts(db: &PgPool) -> sqlx::Result<Vec<TagCount>> {
    sqlx::query_as(
        "SELECT tag, COUNT(*) AS count
         FROM articles, unnest(tags) AS tag
         WHERE status = 'published'
         GROUP BY tag ORDER BY count DESC, tag",
    )
    .fetch_all(db)
    .await
}

pub async fn list_all(db: &PgPool, status: Option<ArticleStatus>) -> sqlx::Result<Vec<Article>> {
    sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM articles
         WHERE ($1::article_status IS NULL OR status = $1)
         ORDER BY COALESCE(published_at, scheduled_at, updated_at) DESC"
    ))
    .bind(status)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Article>> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM articles WHERE id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn slug_taken(db: &PgPool, slug: &str, except: Option<Uuid>) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM articles WHERE slug = $1 AND ($2::uuid IS NULL OR id <> $2))")
        .bind(slug)
        .bind(except)
        .fetch_one(db)
        .await
}

pub async fn insert<'e>(
    db: impl PgExecutor<'e>,
    data: &ArticleWrite<'_>,
    author: Option<Uuid>,
) -> sqlx::Result<Article> {
    sqlx::query_as(&format!(
        "INSERT INTO articles (slug, title, excerpt, content_md, content_html, cover_image, tags,
                               reading_minutes, seo_title, seo_description, author_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING {COLUMNS}"
    ))
    .bind(data.slug)
    .bind(data.title)
    .bind(data.excerpt)
    .bind(data.content_md)
    .bind(data.content_html)
    .bind(data.cover_image)
    .bind(data.tags)
    .bind(data.reading_minutes)
    .bind(data.seo_title)
    .bind(data.seo_description)
    .bind(author)
    .fetch_one(db)
    .await
}

pub async fn update(db: &PgPool, id: Uuid, data: &ArticleWrite<'_>) -> sqlx::Result<Option<Article>> {
    sqlx::query_as(&format!(
        "UPDATE articles SET slug = $2, title = $3, excerpt = $4, content_md = $5, content_html = $6,
                cover_image = $7, tags = $8, reading_minutes = $9, seo_title = $10,
                seo_description = $11, updated_at = now()
         WHERE id = $1 RETURNING {COLUMNS}"
    ))
    .bind(id)
    .bind(data.slug)
    .bind(data.title)
    .bind(data.excerpt)
    .bind(data.content_md)
    .bind(data.content_html)
    .bind(data.cover_image)
    .bind(data.tags)
    .bind(data.reading_minutes)
    .bind(data.seo_title)
    .bind(data.seo_description)
    .fetch_optional(db)
    .await
}

pub async fn delete(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let res = sqlx::query("DELETE FROM articles WHERE id = $1").bind(id).execute(db).await?;
    Ok(res.rows_affected() > 0)
}

/// Change le statut. `scheduled_at` n'est conservé que pour un article validé ;
/// `published_at` est fixé lors de la publication.
pub async fn set_status<'e>(
    db: impl PgExecutor<'e>,
    id: Uuid,
    status: ArticleStatus,
    scheduled_at: Option<DateTime<Utc>>,
    published_at: Option<DateTime<Utc>>,
) -> sqlx::Result<Option<Article>> {
    sqlx::query_as(&format!(
        "UPDATE articles SET status = $2, scheduled_at = $3, published_at = $4, updated_at = now()
         WHERE id = $1 RETURNING {COLUMNS}"
    ))
    .bind(id)
    .bind(status)
    .bind(scheduled_at)
    .bind(published_at)
    .fetch_optional(db)
    .await
}

/// Publie les articles validés dont la date programmée est atteinte (job cron).
pub async fn publish_due(db: &PgPool) -> sqlx::Result<Vec<Article>> {
    sqlx::query_as(&format!(
        "UPDATE articles SET status = 'published', published_at = scheduled_at, updated_at = now()
         WHERE status = 'validated' AND scheduled_at IS NOT NULL AND scheduled_at <= now()
         RETURNING {COLUMNS}"
    ))
    .fetch_all(db)
    .await
}

/// Marque la newsletter comme envoyée ; retourne `false` si elle l'était déjà.
pub async fn mark_newsletter_sent(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let res = sqlx::query(
        "UPDATE articles SET newsletter_sent_at = now() WHERE id = $1 AND newsletter_sent_at IS NULL",
    )
    .bind(id)
    .execute(db)
    .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn count(db: &PgPool) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM articles").fetch_one(db).await
}

pub async fn count_by_status(db: &PgPool) -> sqlx::Result<Vec<(ArticleStatus, i64)>> {
    sqlx::query_as("SELECT status, COUNT(*) FROM articles GROUP BY status")
        .fetch_all(db)
        .await
}

/// Données minimales pour le sitemap et le flux RSS.
#[derive(sqlx::FromRow)]
pub struct FeedEntry {
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

pub async fn feed_entries(db: &PgPool) -> sqlx::Result<Vec<FeedEntry>> {
    sqlx::query_as(
        "SELECT slug, title, excerpt, tags, published_at, updated_at FROM articles
         WHERE status = 'published' ORDER BY published_at DESC",
    )
    .fetch_all(db)
    .await
}

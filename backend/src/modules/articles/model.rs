use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cycle de vie : brouillon → validé (éventuellement programmé) → publié.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "article_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ArticleStatus {
    Draft,
    Validated,
    Published,
}

/// Article complet (administration).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Article {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub content_md: String,
    pub content_html: String,
    pub cover_image: Option<String>,
    pub tags: Vec<String>,
    pub reading_minutes: i32,
    pub status: ArticleStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub newsletter_sent_at: Option<DateTime<Utc>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub author_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Résumé affiché dans les listes publiques.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ArticleSummary {
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub cover_image: Option<String>,
    pub tags: Vec<String>,
    pub reading_minutes: i32,
    pub published_at: Option<DateTime<Utc>>,
}

/// Article publié tel qu'exposé publiquement.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PublicArticle {
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub content_html: String,
    pub cover_image: Option<String>,
    pub tags: Vec<String>,
    pub reading_minutes: i32,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArticlePage {
    pub items: Vec<ArticleSummary>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TagCount {
    pub tag: String,
    pub count: i64,
}

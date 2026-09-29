use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::model::ArticleStatus;

#[derive(Debug, Deserialize)]
pub struct ArticleInput {
    pub title: String,
    pub slug: Option<String>,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub content_md: String,
    pub cover_image: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateInput {
    /// Date de publication automatique (facultative).
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewInput {
    pub content_md: String,
}

#[derive(Debug, Deserialize)]
pub struct PublicListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminListQuery {
    pub status: Option<ArticleStatus>,
}

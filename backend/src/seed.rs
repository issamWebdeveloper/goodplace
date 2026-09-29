//! Contenu initial du blog, injecté au premier démarrage si la table `articles` est vide.
//!
//! Chaque fichier Markdown commence par un en-tête TOML entre deux lignes `+++`.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    markdown,
    modules::{
        articles::{
            model::ArticleStatus,
            repository::{self, ArticleWrite},
        },
        users::repository as users,
    },
};

const ARTICLES: &[&str] = &[
    include_str!("../content/articles/typographie-web-choisir-les-bonnes-polices.md"),
    include_str!("../content/articles/performance-web-les-bases-essentielles.md"),
    include_str!("../content/articles/l-art-de-la-simplicite-numerique.md"),
    include_str!("../content/articles/leitner-learning-angular-aws.md"),
    include_str!("../content/articles/ansaab-genealogie-gedcomx.md"),
    include_str!("../content/articles/angular-amplify-graphql-simplecv.md"),
    include_str!("../content/articles/devkit-devcontainers-ia.md"),
    include_str!("../content/articles/goodplace-rust-axum-angular-ssr.md"),
];

#[derive(Deserialize)]
struct FrontMatter {
    title: String,
    slug: String,
    excerpt: String,
    #[serde(default)]
    tags: Vec<String>,
    cover_image: Option<String>,
    seo_title: Option<String>,
    seo_description: Option<String>,
    status: ArticleStatus,
    published_at: Option<DateTime<Utc>>,
    scheduled_at: Option<DateTime<Utc>>,
}

fn parse(source: &str) -> anyhow::Result<(FrontMatter, &str)> {
    let rest = source
        .strip_prefix("+++")
        .ok_or_else(|| anyhow::anyhow!("en-tête TOML manquant"))?;
    let end = rest.find("\n+++").ok_or_else(|| anyhow::anyhow!("en-tête TOML non terminé"))?;
    let front: FrontMatter = toml::from_str(&rest[..end])?;
    Ok((front, rest[end + 4..].trim_start()))
}

pub async fn run(db: &PgPool) -> anyhow::Result<()> {
    if repository::count(db).await? > 0 {
        return Ok(());
    }
    let author = users::find_admin(db).await?.map(|a| a.id);
    let mut tx = db.begin().await?;
    for source in ARTICLES {
        let (front, body) = parse(source)?;
        let html = markdown::render(body);
        let data = ArticleWrite {
            slug: &front.slug,
            title: &front.title,
            excerpt: &front.excerpt,
            content_md: body,
            content_html: &html,
            cover_image: front.cover_image.as_deref(),
            tags: &front.tags,
            reading_minutes: markdown::reading_minutes(body),
            seo_title: front.seo_title.as_deref(),
            seo_description: front.seo_description.as_deref(),
        };
        let article = repository::insert(&mut *tx, &data, author).await?;
        repository::set_status(&mut *tx, article.id, front.status, front.scheduled_at, front.published_at)
            .await?;
        // Le contenu initial ne déclenche pas de newsletter.
        sqlx::query("UPDATE articles SET newsletter_sent_at = published_at, created_at = COALESCE(published_at, created_at), updated_at = COALESCE(published_at, updated_at) WHERE id = $1")
            .bind(article.id)
            .execute(&mut *tx)
            .await?;
        tracing::info!(slug = %front.slug, minutes = data.reading_minutes, "article initial créé");
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_seed_articles_parse() {
        for source in ARTICLES {
            let (front, body) = parse(source).expect("front matter valide");
            assert!(!front.title.is_empty());
            assert!(body.len() > 500, "{} est trop court", front.slug);
            if front.status == ArticleStatus::Published {
                assert!(front.published_at.is_some(), "{} sans date", front.slug);
            }
        }
    }
}

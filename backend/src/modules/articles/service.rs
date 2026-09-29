//! Règles métier des articles : préparation du contenu, transitions de statut, publication.

use chrono::Utc;
use uuid::Uuid;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    mail::templates,
    markdown,
    modules::users::repository as users,
};

use super::{
    dto::ArticleInput,
    model::{Article, ArticleStatus},
    repository::{self, ArticleWrite},
};

fn clean_opt(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut out: Vec<String> = tags
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty() && t.chars().count() <= 40)
        .collect();
    out.dedup();
    out.truncate(10);
    out
}

async fn unique_slug(state: &AppState, input: &ArticleInput, id: Option<Uuid>) -> AppResult<String> {
    let base = slug::slugify(clean_opt(&input.slug).unwrap_or(&input.title));
    if base.is_empty() {
        return Err(AppError::bad_request("Impossible de générer un slug à partir du titre"));
    }
    if clean_opt(&input.slug).is_some() {
        // Slug choisi explicitement : il doit être libre.
        if repository::slug_taken(&state.db, &base, id).await? {
            return Err(AppError::Conflict("Ce slug est déjà utilisé".into()));
        }
        return Ok(base);
    }
    let mut candidate = base.clone();
    let mut n = 2;
    while repository::slug_taken(&state.db, &candidate, id).await? {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    Ok(candidate)
}

fn validate(input: &ArticleInput) -> AppResult<()> {
    let title_len = input.title.trim().chars().count();
    if title_len == 0 || title_len > 200 {
        return Err(AppError::bad_request("Le titre est obligatoire (200 caractères maximum)"));
    }
    if input.excerpt.chars().count() > 500 {
        return Err(AppError::bad_request("Le résumé ne doit pas dépasser 500 caractères"));
    }
    Ok(())
}

pub async fn create(state: &AppState, input: ArticleInput, author: Uuid) -> AppResult<Article> {
    validate(&input)?;
    let slug = unique_slug(state, &input, None).await?;
    let html = markdown::render(&input.content_md);
    let tags = normalize_tags(&input.tags);
    let data = ArticleWrite {
        slug: &slug,
        title: input.title.trim(),
        excerpt: input.excerpt.trim(),
        content_md: &input.content_md,
        content_html: &html,
        cover_image: clean_opt(&input.cover_image),
        tags: &tags,
        reading_minutes: markdown::reading_minutes(&input.content_md),
        seo_title: clean_opt(&input.seo_title),
        seo_description: clean_opt(&input.seo_description),
    };
    Ok(repository::insert(&state.db, &data, Some(author)).await?)
}

pub async fn update(state: &AppState, id: Uuid, input: ArticleInput) -> AppResult<Article> {
    validate(&input)?;
    let slug = unique_slug(state, &input, Some(id)).await?;
    let html = markdown::render(&input.content_md);
    let tags = normalize_tags(&input.tags);
    let data = ArticleWrite {
        slug: &slug,
        title: input.title.trim(),
        excerpt: input.excerpt.trim(),
        content_md: &input.content_md,
        content_html: &html,
        cover_image: clean_opt(&input.cover_image),
        tags: &tags,
        reading_minutes: markdown::reading_minutes(&input.content_md),
        seo_title: clean_opt(&input.seo_title),
        seo_description: clean_opt(&input.seo_description),
    };
    let article = repository::update(&state.db, id, &data).await?.ok_or(AppError::NotFound)?;
    if article.status == ArticleStatus::Published {
        state.cache.invalidate_articles();
    }
    Ok(article)
}

async fn get(state: &AppState, id: Uuid) -> AppResult<Article> {
    repository::find_by_id(&state.db, id).await?.ok_or(AppError::NotFound)
}

/// Valide un article, avec éventuellement une date de publication automatique.
pub async fn validate_article(
    state: &AppState,
    id: Uuid,
    scheduled_at: Option<chrono::DateTime<Utc>>,
) -> AppResult<Article> {
    let article = get(state, id).await?;
    if article.status == ArticleStatus::Published {
        return Err(AppError::Conflict(
            "Article déjà publié : dépubliez-le avant de le programmer".into(),
        ));
    }
    if scheduled_at.is_some_and(|at| at <= Utc::now()) {
        return Err(AppError::bad_request("La date de publication doit être dans le futur"));
    }
    Ok(repository::set_status(&state.db, id, ArticleStatus::Validated, scheduled_at, None)
        .await?
        .ok_or(AppError::NotFound)?)
}

pub async fn publish_now(state: &AppState, id: Uuid) -> AppResult<Article> {
    let article = get(state, id).await?;
    if article.status == ArticleStatus::Published {
        return Ok(article);
    }
    let article = repository::set_status(&state.db, id, ArticleStatus::Published, None, Some(Utc::now()))
        .await?
        .ok_or(AppError::NotFound)?;
    after_publication(state, std::slice::from_ref(&article));
    Ok(article)
}

/// Retour en brouillon (retire l'article du site et annule la programmation).
pub async fn unpublish(state: &AppState, id: Uuid) -> AppResult<Article> {
    let before = get(state, id).await?;
    let article = repository::set_status(&state.db, id, ArticleStatus::Draft, None, None)
        .await?
        .ok_or(AppError::NotFound)?;
    if before.status == ArticleStatus::Published {
        state.cache.invalidate_articles();
    }
    Ok(article)
}

pub async fn delete(state: &AppState, id: Uuid) -> AppResult<()> {
    if !repository::delete(&state.db, id).await? {
        return Err(AppError::NotFound);
    }
    state.cache.invalidate_articles();
    Ok(())
}

/// Job cron : publie les articles programmés arrivés à échéance.
pub async fn publish_scheduled(state: &AppState) -> anyhow::Result<usize> {
    let published = repository::publish_due(&state.db).await?;
    if !published.is_empty() {
        for article in &published {
            tracing::info!(slug = %article.slug, "article publié automatiquement");
        }
        after_publication(state, &published);
    }
    Ok(published.len())
}

/// Invalide le cache et prévient les abonnés (une seule fois par article).
fn after_publication(state: &AppState, articles: &[Article]) {
    state.cache.invalidate_articles();
    if !state.config.newsletter_on_publish {
        return;
    }
    for article in articles.iter().cloned() {
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = notify_subscribers(&state, &article).await {
                tracing::error!(slug = %article.slug, error = ?err, "échec de la newsletter");
            }
        });
    }
}

async fn notify_subscribers(state: &AppState, article: &Article) -> anyhow::Result<()> {
    if !repository::mark_newsletter_sent(&state.db, article.id).await? {
        return Ok(());
    }
    let url = state.mailer.link(&format!("/blog/{}", article.slug));
    let emails = users::subscriber_emails(&state.db).await?;
    tracing::info!(slug = %article.slug, count = emails.len(), "envoi de la newsletter");
    for email in emails {
        let content = templates::new_article(&state.mailer.site_name, &article.title, &article.excerpt, &url);
        if let Err(err) = state.mailer.send(&email, content).await {
            tracing::warn!(%email, error = ?err, "newsletter non délivrée");
        }
    }
    Ok(())
}

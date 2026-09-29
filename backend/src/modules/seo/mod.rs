//! Documents SEO générés dynamiquement : sitemap.xml et flux RSS.

use std::sync::Arc;

use axum::{extract::State, http::header, response::IntoResponse, routing::get, Router};

use crate::{app::AppState, error::AppResult, modules::articles::repository};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sitemap.xml", get(sitemap))
        .route("/rss.xml", get(rss))
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

async fn cached<F, Fut>(state: &AppState, key: &'static str, build: F) -> AppResult<Arc<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<String>>,
{
    if let Some(doc) = state.cache.documents.get(&key).await {
        return Ok(doc);
    }
    let doc = Arc::new(build().await?);
    state.cache.documents.insert(key, doc.clone()).await;
    Ok(doc)
}

async fn sitemap(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let doc = cached(&state, "sitemap", || async {
        let site = &state.config.site_url;
        let entries = repository::feed_entries(&state.db).await?;
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
        );
        let last = entries.first().map(|e| e.updated_at.format("%Y-%m-%d").to_string());
        for (path, priority) in [("/", "1.0"), ("/blog", "0.9")] {
            xml.push_str(&format!("  <url><loc>{site}{path}</loc>"));
            if let Some(last) = &last {
                xml.push_str(&format!("<lastmod>{last}</lastmod>"));
            }
            xml.push_str(&format!("<priority>{priority}</priority></url>\n"));
        }
        for e in &entries {
            xml.push_str(&format!(
                "  <url><loc>{site}/blog/{}</loc><lastmod>{}</lastmod><priority>0.8</priority></url>\n",
                escape(&e.slug),
                e.updated_at.format("%Y-%m-%d")
            ));
        }
        xml.push_str("</urlset>\n");
        Ok(xml)
    })
    .await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        doc.as_str().to_owned(),
    ))
}

async fn rss(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let doc = cached(&state, "rss", || async {
        let site = &state.config.site_url;
        let name = escape(&state.config.site_name);
        let entries = repository::feed_entries(&state.db).await?;
        let mut xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
  <title>{name} — Blog</title>
  <link>{site}/blog</link>
  <atom:link href="{site}/rss.xml" rel="self" type="application/rss+xml"/>
  <description>Développement web, Angular, Rust et IA par Issam Gharsallah</description>
  <language>fr-FR</language>
"#
        );
        for e in entries.iter().take(30) {
            let url = format!("{site}/blog/{}", escape(&e.slug));
            let date = e.published_at.unwrap_or(e.updated_at).to_rfc2822();
            xml.push_str(&format!(
                "  <item><title>{}</title><link>{url}</link><guid>{url}</guid><pubDate>{date}</pubDate><description>{}</description>",
                escape(&e.title),
                escape(&e.excerpt)
            ));
            for tag in &e.tags {
                xml.push_str(&format!("<category>{}</category>", escape(tag)));
            }
            xml.push_str("</item>\n");
        }
        xml.push_str("</channel>\n</rss>\n");
        Ok(xml)
    })
    .await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/rss+xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=900"),
        ],
        doc.as_str().to_owned(),
    ))
}

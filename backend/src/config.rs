//! Configuration de l'application, lue depuis les variables d'environnement.

use std::env;

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub enum MailTransport {
    /// Envoi réel via SMTP.
    Smtp {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
        starttls: bool,
    },
    /// Les emails sont uniquement écrits dans les logs (développement).
    Log,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub database_max_connections: u32,
    /// URL publique du site (front), utilisée dans les emails, le sitemap et le flux RSS.
    pub site_url: String,
    pub site_name: String,
    pub jwt_secret: String,
    pub session_days: i64,
    pub cookie_secure: bool,
    pub admin_email: String,
    pub admin_password: Option<String>,
    pub mail_from: String,
    pub mail_transport: MailTransport,
    /// Expression cron (6 champs, secondes incluses) du job de publication programmée.
    pub publish_cron: String,
    pub newsletter_on_publish: bool,
    pub seed_content: bool,
    pub cors_origins: Vec<String>,
}

fn var(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.trim().is_empty())
}

fn var_or(key: &str, default: &str) -> String {
    var(key).unwrap_or_else(|| default.to_string())
}

fn flag(key: &str, default: bool) -> bool {
    var(key)
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let jwt_secret = var("JWT_SECRET").context("JWT_SECRET est obligatoire")?;
        anyhow::ensure!(
            jwt_secret.len() >= 32,
            "JWT_SECRET doit contenir au moins 32 caractères"
        );

        let mail_transport = match var_or("MAIL_TRANSPORT", "log").as_str() {
            "smtp" => MailTransport::Smtp {
                host: var("SMTP_HOST").context("SMTP_HOST est obligatoire en mode smtp")?,
                port: var_or("SMTP_PORT", "587").parse().context("SMTP_PORT invalide")?,
                username: var("SMTP_USERNAME"),
                password: var("SMTP_PASSWORD"),
                starttls: flag("SMTP_STARTTLS", true),
            },
            _ => MailTransport::Log,
        };

        Ok(Self {
            bind_addr: var_or("BIND_ADDR", "0.0.0.0:8080"),
            database_url: var("DATABASE_URL").context("DATABASE_URL est obligatoire")?,
            database_max_connections: var_or("DATABASE_MAX_CONNECTIONS", "10").parse()?,
            site_url: var_or("SITE_URL", "http://localhost:4000")
                .trim_end_matches('/')
                .to_string(),
            site_name: var_or("SITE_NAME", "GoodPlace"),
            jwt_secret,
            session_days: var_or("SESSION_DAYS", "7").parse()?,
            cookie_secure: flag("COOKIE_SECURE", false),
            admin_email: var_or("ADMIN_EMAIL", "issam.webdeveloper@gmail.com").to_lowercase(),
            admin_password: var("ADMIN_PASSWORD"),
            mail_from: var_or("MAIL_FROM", "GoodPlace <no-reply@goodplace.ovh>"),
            mail_transport,
            publish_cron: var_or("PUBLISH_CRON", "0 * * * * *"),
            newsletter_on_publish: flag("NEWSLETTER_ON_PUBLISH", true),
            seed_content: flag("SEED_CONTENT", true),
            cors_origins: var("CORS_ORIGINS")
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
        })
    }
}

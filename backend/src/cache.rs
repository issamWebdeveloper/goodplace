//! Cache applicatif en mémoire (moka) pour fluidifier les lectures fréquentes.
//!
//! - les pages publiques (liste d'articles, article, profil) sont servies depuis le cache
//!   et invalidées à chaque écriture côté administration ou publication par le cron ;
//! - les utilisateurs authentifiés sont mis en cache pour éviter une requête SQL par appel ;
//! - un cache à durée de vie courte sert d'anti-spam pour les envois d'emails.

use std::{sync::Arc, time::Duration};

use moka::future::Cache;
use uuid::Uuid;

use crate::modules::{
    articles::model::{ArticlePage, PublicArticle},
    users::model::User,
};

#[derive(Clone)]
pub struct AppCache {
    pub article_pages: Cache<String, Arc<ArticlePage>>,
    pub articles: Cache<String, Arc<PublicArticle>>,
    pub documents: Cache<&'static str, Arc<String>>,
    pub users: Cache<Uuid, Arc<User>>,
    /// Compteur d'échecs de connexion par email (fenêtre de 15 minutes).
    pub login_attempts: Cache<String, u32>,
    throttle: Cache<String, ()>,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            article_pages: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(600))
                .build(),
            articles: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(600))
                .build(),
            documents: Cache::builder()
                .max_capacity(16)
                .time_to_live(Duration::from_secs(600))
                .build(),
            users: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(300))
                .build(),
            login_attempts: Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_secs(900))
                .build(),
            throttle: Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_secs(60))
                .build(),
        }
    }

    /// Invalide tout ce qui dépend de la liste des articles publiés.
    pub fn invalidate_articles(&self) {
        self.article_pages.invalidate_all();
        self.articles.invalidate_all();
        self.documents.invalidate_all();
    }

    pub async fn invalidate_user(&self, id: Uuid) {
        self.users.invalidate(&id).await;
    }

    /// Retourne `false` si la même action a déjà été déclenchée dans la dernière minute.
    pub async fn try_throttle(&self, key: String) -> bool {
        let entry = self.throttle.entry(key).or_insert(()).await;
        entry.is_fresh()
    }
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}

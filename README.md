# GoodPlace

Site vitrine et blog d'**Issam Gharsallah** — développement web (Angular, full-stack) et IA.

- **API** : Rust + [Axum](https://github.com/tokio-rs/axum), PostgreSQL (sqlx), cache mémoire [moka](https://github.com/moka-rs/moka), tâches cron (`tokio-cron-scheduler`), emails SMTP (`lettre`)
- **Front** : Angular 22 (standalone, signals, zoneless) avec **rendu serveur (SSR)** et hydratation
- **Design** : inspiré du thème Sonic (YOOtheme), bleu `#32beff` / dégradé `#19b1f0 → #0171cd`, Lato + Open Sans
  auto-hébergées (`@fontsource`, aucune requête vers Google Fonts), logo « GP »
- **Portfolio associé** : [gharsallah.fr](https://gharsallah.fr) (anglais) ; GoodPlace est le blog technique en français

## Fonctionnalités

### Vitrine
Page d'accueil générée à partir du CV (format [JSON Resume](https://jsonresume.org), `backend/data/profile.json`) :
compétences, parcours, projets GitHub récents, derniers articles, newsletter, contact.

### Blog et circuit éditorial
| Statut | Visibilité | Description |
| --- | --- | --- |
| **Brouillon** | privé | en cours de rédaction |
| **Validé** | privé | relu ; peut recevoir une **date de publication automatique** |
| **Publié** | public | en ligne, dans le sitemap et le flux RSS ; les abonnés sont notifiés |

Un job cron (par défaut chaque minute, `PUBLISH_CRON`) publie les articles validés dont la date est atteinte.
Il est aussi exécuté au démarrage pour rattraper une publication manquée pendant un arrêt.
Les articles sont écrits en Markdown, rendus en HTML et assainis côté serveur.

### Profils et authentification
| Profil | Inscription | Connexion |
| --- | --- | --- |
| **Administrateur** (unique : `ADMIN_EMAIL`) | créé depuis `ADMIN_PASSWORD` au démarrage | email + mot de passe |
| **Utilisateur** | nom + email → lien de vérification (24 h) → compte actif → définition du mot de passe (lien 10 min) | email + mot de passe ; modification du mot de passe via lien email **10 min** |
| **Abonné newsletter** | email seul → lien de vérification | **lien magique** par email, valable **10 min** |

- Jetons email aléatoires (256 bits), stockés uniquement sous forme d'empreinte SHA-256, à usage unique.
- Session : JWT dans un cookie `HttpOnly` + `SameSite=Lax` ; changer de mot de passe déconnecte toutes les autres sessions.
- Mots de passe hachés avec Argon2id ; limitation des tentatives de connexion et des envois d'emails.
- Réponses identiques que l'email existe ou non (pas d'énumération des comptes).

### SEO et performance
- SSR pour l'accueil et le blog : titre, description, canonical, Open Graph, Twitter Card et JSON-LD (`Person`, `Blog`, `BlogPosting`) dans le HTML initial.
- Vrais codes 404 côté serveur, `sitemap.xml`, `rss.xml`, `robots.txt`.
- Données transférées du rendu serveur au navigateur (pas de double appel API).
- Cache applicatif (listes, articles, sitemap/RSS, sessions) invalidé à chaque publication, en-têtes `Cache-Control`.
- CORS limité à la lecture publique (`/api/articles`, `/api/profile`, GET, sans cookie) pour les origines de
  `CORS_ORIGINS` — utilisé par gharsallah.fr pour afficher les derniers articles.

## Architecture

```
backend/                     API Rust (un module par domaine fonctionnel)
├── migrations/              schéma PostgreSQL (appliqué au démarrage)
├── content/articles/        articles initiaux (Markdown + en-tête TOML)
├── data/profile.json        CV au format JSON Resume
└── src/
    ├── main.rs              démarrage : config, migrations, admin, seed, cron, serveur
    ├── app.rs               état partagé + routeur
    ├── config.rs · error.rs · cache.rs · markdown.rs · scheduler.rs · seed.rs
    ├── mail/                transport SMTP / log + gabarits
    └── modules/
        ├── auth/            sessions, jetons email, extracteurs, inscription/connexion
        ├── users/           modèle, SQL, espace compte, administration des comptes
        ├── articles/        modèle, SQL, règles éditoriales, routes publiques et admin
        ├── profile/         CV
        └── seo/             sitemap.xml, rss.xml

frontend/src/
├── server.ts                serveur Express SSR + proxy /api vers l'API Rust
└── app/
    ├── core/                modèles, auth (store + guards), SEO, profil
    ├── shared/              logo, carte d'article, formulaire newsletter
    ├── layout/              en-tête, pied de page
    └── features/            home, blog, auth, account, admin, not-found
```

Le serveur Node du front est la seule porte d'entrée : il sert les pages et proxifie `/api`, `/sitemap.xml`
et `/rss.xml` vers l'API. Navigateur et API partagent donc la même origine (cookie `HttpOnly`, pas de CORS).

## API

| Méthode | Route | Accès |
| --- | --- | --- |
| GET | `/api/articles?page=&per_page=&tag=` · `/api/articles/tags` · `/api/articles/{slug}` | public |
| GET | `/api/profile` · `/sitemap.xml` · `/rss.xml` · `/api/health` | public |
| POST | `/api/auth/register` · `/subscribe` · `/verify-email` · `/password` · `/password/forgot` | public |
| POST | `/api/auth/login` · `/magic-link` · `/magic-link/consume` · `/logout` | public |
| GET | `/api/auth/me` (`null` si anonyme) | public |
| POST | `/api/auth/password/change-request` · PATCH/DELETE `/api/account` | connecté |
| GET/POST | `/api/admin/articles` · GET/PUT/DELETE `/api/admin/articles/{id}` | admin |
| POST | `/api/admin/articles/{id}/validate` (`{ scheduled_at }`) · `/publish` · `/unpublish` · `/preview` | admin |
| GET | `/api/admin/articles/stats` · `/api/admin/users?role=` · DELETE `/api/admin/users/{id}` | admin |

## Démarrage

### En production (Docker, derrière le Caddy du serveur)

```bash
cp .env.example .env        # POSTGRES_PASSWORD, JWT_SECRET, ADMIN_PASSWORD, SMTP_* ...
docker compose -p goodplace up -d --build
```

Le front écoute sur `127.0.0.1:4000`. Ajouter le bloc du fichier `Caddyfile` à `/etc/caddy/Caddyfile`,
puis `sudo systemctl reload caddy` (HTTPS automatique).

Mise à jour : `git pull && docker compose -p goodplace up -d --build`.
Logs : `docker compose -p goodplace logs -f api`.

En local, `docker compose --profile dev up` ajoute [Mailpit](http://localhost:8025) pour lire les emails.

### En développement

Prérequis : Rust stable, Node.js 22.22+ ou 24.15+, PostgreSQL.

```bash
# API (http://localhost:8080)
cd backend
cp ../.env.example .env     # DATABASE_URL, JWT_SECRET, ADMIN_PASSWORD, MAIL_TRANSPORT=log, COOKIE_SECURE=false
cargo run

# Front (build SSR puis serveur, http://localhost:4000)
cd frontend
npm ci
npm run build
API_URL=http://127.0.0.1:8080 npm run serve:ssr:frontend
```

Avec `MAIL_TRANSPORT=log`, les liens de vérification, de connexion et de mot de passe apparaissent dans les logs de l'API.

### Tests

```bash
cd backend && cargo test     # rendu Markdown, temps de lecture, articles initiaux, profil
```

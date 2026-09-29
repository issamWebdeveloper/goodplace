+++
title = "GoodPlace : les coulisses d'un site en Rust, Axum et Angular SSR"
slug = "goodplace-rust-axum-angular-ssr"
excerpt = "L'architecture de ce site : une API Rust/Axum découpée en modules, PostgreSQL, un cache mémoire, une publication programmée par cron et un front Angular rendu côté serveur."
tags = ["Rust", "Angular", "Architecture", "Projet"]
cover_image = "/images/blog/goodplace.png"
seo_description = "Architecture de GoodPlace : API Rust Axum modulaire, PostgreSQL, cache moka, cron de publication, connexion par lien magique et Angular SSR pour le SEO."
status = "validated"
scheduled_at = "2026-10-06T08:00:00Z"
+++

Le site que vous lisez est à la fois ma vitrine et mon terrain d'expérimentation. Pour sa nouvelle version, je me suis fixé un cahier des charges précis : un blog avec un vrai circuit éditorial, plusieurs profils d'utilisateurs, une connexion sans mot de passe pour les abonnés, un excellent référencement et des temps de réponse aussi courts que possible. Voici comment il est construit. Le code source est disponible sur [GitHub](https://github.com/issamWebdeveloper/goodplace).

## Vue d'ensemble

```text
Navigateur ──► Angular SSR (Node) ──► API Rust / Axum ──► PostgreSQL
                  │   /api/* proxifié        │
                  └── HTML rendu serveur     └── cache mémoire (moka) + cron
```

- Le **front Angular** est rendu côté serveur : chaque page arrive avec son contenu, ses balises meta et ses données structurées.
- Le serveur Node du front **proxifie `/api`** vers l'API Rust : navigateur et API partagent la même origine, le cookie de session reste `HttpOnly` et `SameSite`, sans configuration CORS.
- L'**API Rust** gère le contenu, l'authentification, l'envoi des emails et la publication programmée.

## Pourquoi Rust pour l'API ?

Mon quotidien est surtout TypeScript et Angular. Rust apporte ici ce qui me manque parfois côté Node : un binaire unique qui consomme quelques mégaoctets de mémoire, un typage qui rend impossibles des catégories entières d'erreurs, et des performances qui permettent d'héberger le tout sur une petite machine. **Axum**, construit sur Tokio et Tower, offre une ergonomie proche de ce qu'on connaît avec Express, avec des extracteurs typés à la place des `req.body` non vérifiés.

## Un découpage par modules fonctionnels

L'API est organisée par **domaine**, et non par couche technique. Chaque module regroupe tout ce qui le concerne :

```text
src/
├── app.rs            # état partagé et assemblage des routes
├── cache.rs          # caches applicatifs
├── scheduler.rs      # tâches cron
├── mail/             # transport SMTP et gabarits
└── modules/
    ├── auth/         # sessions, jetons email, extracteurs
    ├── users/        # modèle, requêtes SQL, espace compte
    ├── articles/     # modèle, requêtes, règles éditoriales, routes
    ├── profile/      # CV au format JSON Resume
    └── seo/          # sitemap.xml et flux RSS
```

Dans chaque module, les responsabilités sont séparées : `model.rs` décrit les données, `repository.rs` contient le SQL, `service.rs` les règles métier et `routes.rs` le HTTP. Un gestionnaire de route ne fait qu'extraire, déléguer et répondre.

## Un circuit éditorial en trois états

Un article passe par trois statuts :

1. **Brouillon** : invisible, modifiable librement.
2. **Validé** : relu et prêt, mais pas encore en ligne. On peut lui attribuer une **date de publication automatique**.
3. **Publié** : visible sur le site, dans le sitemap et le flux RSS.

La publication programmée repose sur une tâche cron (`tokio-cron-scheduler`) exécutée chaque minute. Elle tient en une seule requête SQL atomique :

```sql
UPDATE articles
SET status = 'published', published_at = scheduled_at, updated_at = now()
WHERE status = 'validated' AND scheduled_at <= now()
RETURNING *;
```

Les articles renvoyés déclenchent l'invalidation du cache et l'envoi d'une notification aux abonnés, une seule fois par article grâce à un horodatage dédié. La même fonction est appelée au démarrage : une publication prévue pendant une maintenance n'est jamais perdue. Cet article, d'ailleurs, a été programmé de cette façon.

## Trois profils, trois façons de se connecter

- **L'administrateur**, unique, se connecte par email et mot de passe. Son compte est créé et synchronisé depuis la configuration du serveur ; une contrainte d'unicité en base empêche l'existence d'un second administrateur.
- **Les utilisateurs** s'inscrivent avec leur nom et leur email, confirment leur adresse via un lien, puis définissent leur mot de passe. Pour le modifier, ils reçoivent un lien valable **10 minutes**.
- **Les abonnés** à la newsletter ne donnent que leur email. Après confirmation, ils se connectent via un **lien magique** envoyé par email, lui aussi valable 10 minutes.

Tous ces liens reposent sur le même mécanisme : un jeton aléatoire de 256 bits dont **seule l'empreinte SHA-256 est stockée** en base, avec un usage, une date d'expiration et une date d'utilisation. La consommation est atomique : un `UPDATE … RETURNING` qui ne réussit que si le jeton est inutilisé, non expiré et du bon type.

La session est un JWT dans un cookie `HttpOnly`. Il contient une **version de session** comparée à celle de l'utilisateur à chaque requête : changer son mot de passe incrémente cette version et déconnecte instantanément tous les autres appareils. Les mots de passe sont hachés avec Argon2id, dans un thread dédié pour ne pas bloquer le runtime asynchrone.

Enfin, les formulaires qui déclenchent un email répondent toujours le même message, que l'adresse existe ou non : impossible de s'en servir pour deviner qui possède un compte.

## Le cache, pour fluidifier

PostgreSQL est rapide, mais la meilleure requête reste celle qu'on n'exécute pas. L'API utilise **moka**, un cache concurrent en mémoire, à plusieurs niveaux :

- les **pages de liste** et les **articles publiés**, invalidés à chaque modification éditoriale ;
- le **sitemap** et le **flux RSS**, régénérés uniquement après une publication ;
- les **utilisateurs connectés**, pour ne pas interroger la base à chaque requête authentifiée ;
- des **compteurs anti-abus** à durée de vie courte : un email par minute et par adresse, dix tentatives de connexion par quart d'heure.

Un cache en mémoire suffit pour une instance unique. Le jour où l'API tournera sur plusieurs instances, ces caches pourront migrer vers Redis sans toucher aux modules : ils sont tous regroupés dans `cache.rs`.

## Angular SSR et référencement

Le front est une application Angular récente : composants standalone, signals, détection de changements sans Zone.js, et rendu côté serveur avec hydratation. Pour le SEO, chaque page :

- définit son titre, sa description, son URL canonique et ses balises Open Graph via un service dédié ;
- insère des **données structurées JSON-LD** (`Person` pour l'accueil, `BlogPosting` pour les articles) ;
- transfère les données chargées côté serveur au navigateur, qui ne refait donc pas les appels à l'API lors de l'hydratation.

Le sitemap et le flux RSS sont générés par l'API à partir des articles publiés.

## Ce que j'en retiens

- **Découper par domaine** rend le code lisible : on sait immédiatement où chercher.
- **Rust et Axum** se prêtent très bien à une API web classique, pour peu qu'on accepte un temps de compilation plus long.
- **Un cron et une requête atomique** suffisent pour une publication programmée fiable.
- **Les liens magiques** offrent une expérience plus simple aux abonnés, à condition de soigner l'expiration et l'usage unique des jetons.

Le site continuera d'évoluer, et ce blog en racontera les étapes.

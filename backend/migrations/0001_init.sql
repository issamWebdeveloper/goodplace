-- Schéma initial GoodPlace

CREATE TYPE user_role AS ENUM ('admin', 'user', 'subscriber');
CREATE TYPE article_status AS ENUM ('draft', 'validated', 'published');
CREATE TYPE token_purpose AS ENUM ('verify_email', 'set_password', 'reset_password', 'magic_login');

CREATE TABLE users (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email             TEXT NOT NULL,
    name              TEXT,
    role              user_role NOT NULL,
    password_hash     TEXT,
    email_verified_at TIMESTAMPTZ,
    is_active         BOOLEAN NOT NULL DEFAULT FALSE,
    -- Incrémenté à chaque changement de mot de passe : invalide les sessions existantes.
    session_version   INTEGER NOT NULL DEFAULT 0,
    last_login_at     TIMESTAMPTZ,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX users_email_unique ON users (lower(email));
-- Un seul administrateur possible.
CREATE UNIQUE INDEX users_single_admin ON users (role) WHERE role = 'admin';

CREATE TABLE email_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    purpose     token_purpose NOT NULL,
    -- Seul le hash SHA-256 du jeton est stocké.
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX email_tokens_user_purpose ON email_tokens (user_id, purpose);

CREATE TABLE articles (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug            TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    excerpt         TEXT NOT NULL DEFAULT '',
    content_md      TEXT NOT NULL DEFAULT '',
    content_html    TEXT NOT NULL DEFAULT '',
    cover_image     TEXT,
    tags            TEXT[] NOT NULL DEFAULT '{}',
    reading_minutes INTEGER NOT NULL DEFAULT 1,
    status          article_status NOT NULL DEFAULT 'draft',
    -- Date de publication automatique pour un article validé (traitée par le cron).
    scheduled_at    TIMESTAMPTZ,
    published_at    TIMESTAMPTZ,
    -- Date d'envoi de la notification aux abonnés (évite les doublons en cas de republication).
    newsletter_sent_at TIMESTAMPTZ,
    seo_title       TEXT,
    seo_description TEXT,
    author_id       UUID REFERENCES users (id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX articles_published ON articles (published_at DESC) WHERE status = 'published';
CREATE INDEX articles_scheduled ON articles (scheduled_at) WHERE status = 'validated';
CREATE INDEX articles_tags ON articles USING GIN (tags);

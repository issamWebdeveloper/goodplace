//! Règles métier de l'authentification.
//!
//! Profils :
//! - **admin** : compte unique (ADMIN_EMAIL), mot de passe défini par la configuration ;
//! - **user** : inscription → vérification de l'email (compte actif) → définition du mot de passe ;
//!   modification du mot de passe via un lien email valable 10 minutes ;
//! - **subscriber** : abonné newsletter, email uniquement, vérifié par lien ;
//!   se connecte ensuite via un lien magique valable 10 minutes.
//!
//! Les demandes par email répondent toujours de la même façon, que l'adresse existe ou non,
//! pour ne pas divulguer la liste des comptes.

use std::sync::Arc;

use lettre::Address;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    mail::templates,
    modules::users::{
        model::{Role, User},
        repository as users,
    },
};

use super::{
    password,
    tokens::{self, Purpose},
};

const MAX_LOGIN_ATTEMPTS: u32 = 10;

pub fn normalize_email(raw: &str) -> AppResult<String> {
    let email = raw.trim().to_lowercase();
    if email.len() > 254 || email.parse::<Address>().is_err() {
        return Err(AppError::bad_request("Adresse email invalide"));
    }
    Ok(email)
}

async fn throttle(state: &AppState, action: &str, email: &str) -> AppResult<()> {
    if state.cache.try_throttle(format!("{action}:{email}")).await {
        Ok(())
    } else {
        Err(AppError::TooManyRequests)
    }
}

async fn send_verification(state: &AppState, user: &User) -> AppResult<()> {
    let token = tokens::create(&state.db, user.id, Purpose::VerifyEmail).await?;
    let url = state.mailer.link(&format!("/auth/verification?token={token}"));
    let site = &state.mailer.site_name;
    let content = match user.role {
        Role::Subscriber => templates::verify_subscription(site, &url),
        _ => templates::verify_account(site, &url),
    };
    state.mailer.send_in_background(user.email.clone(), content);
    Ok(())
}

pub async fn register(state: &AppState, name: &str, email: &str) -> AppResult<()> {
    let email = normalize_email(email)?;
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(AppError::bad_request("Le nom est obligatoire (80 caractères maximum)"));
    }
    if email == state.config.admin_email {
        return Ok(());
    }
    throttle(state, "register", &email).await?;

    let user = match users::find_by_email(&state.db, &email).await? {
        None => users::create(&state.db, &email, Some(name), Role::User).await?,
        // Compte non vérifié : on renvoie la vérification (un abonné non confirmé devient utilisateur).
        Some(existing) if existing.email_verified_at.is_none() && existing.role != Role::Admin => {
            sqlx::query("UPDATE users SET role = 'user', name = $2, updated_at = now() WHERE id = $1")
                .bind(existing.id)
                .bind(name)
                .execute(&state.db)
                .await?;
            User { role: Role::User, name: Some(name.to_string()), ..existing }
        }
        Some(_) => return Ok(()),
    };
    send_verification(state, &user).await
}

pub async fn subscribe(state: &AppState, email: &str) -> AppResult<()> {
    let email = normalize_email(email)?;
    if email == state.config.admin_email {
        return Ok(());
    }
    throttle(state, "subscribe", &email).await?;

    let user = match users::find_by_email(&state.db, &email).await? {
        None => users::create(&state.db, &email, None, Role::Subscriber).await?,
        Some(existing) if existing.role == Role::Subscriber && existing.email_verified_at.is_none() => existing,
        Some(_) => return Ok(()),
    };
    send_verification(state, &user).await
}

pub struct VerifyOutcome {
    pub user: User,
    pub set_password_token: Option<String>,
}

pub async fn verify_email(state: &AppState, token: &str) -> AppResult<VerifyOutcome> {
    let (user_id, _) = tokens::consume(&state.db, token, &[Purpose::VerifyEmail])
        .await?
        .ok_or(AppError::InvalidToken)?;
    let user = users::mark_verified(&state.db, user_id).await?;
    state.cache.invalidate_user(user.id).await;

    let set_password_token = if user.role == Role::User && user.password_hash.is_none() {
        Some(tokens::create(&state.db, user.id, Purpose::SetPassword).await?)
    } else {
        None
    };
    Ok(VerifyOutcome { user, set_password_token })
}

/// Définit (création) ou modifie (réinitialisation) le mot de passe d'un utilisateur standard.
pub async fn set_password(state: &AppState, token: &str, new_password: &str) -> AppResult<User> {
    password::validate(new_password)?;
    let (user_id, _) = tokens::consume(
        &state.db,
        token,
        &[Purpose::SetPassword, Purpose::ResetPassword],
    )
    .await?
    .ok_or(AppError::InvalidToken)?;

    let user = users::find_by_id(&state.db, user_id).await?.ok_or(AppError::InvalidToken)?;
    if user.role != Role::User || !user.is_active {
        return Err(AppError::InvalidToken);
    }
    let hash = password::hash(new_password.to_string()).await?;
    let user = users::set_password(&state.db, user.id, &hash).await?;
    state.cache.invalidate_user(user.id).await;
    users::touch_login(&state.db, user.id).await?;
    Ok(user)
}

pub async fn login(state: &AppState, email: &str, pass: &str) -> AppResult<User> {
    let email = normalize_email(email).map_err(|_| AppError::InvalidCredentials)?;
    let attempts_key = format!("login:{email}");
    let attempts = state.cache.login_attempts.get(&attempts_key).await.unwrap_or(0);
    if attempts >= MAX_LOGIN_ATTEMPTS {
        return Err(AppError::TooManyRequests);
    }

    let user = users::find_by_email(&state.db, &email).await?;
    let authenticated = match &user {
        Some(u) if matches!(u.role, Role::Admin | Role::User) && u.is_active => match &u.password_hash {
            Some(hash) => password::verify(pass.to_string(), hash.clone()).await,
            None => false,
        },
        _ => false,
    };

    if !authenticated {
        state.cache.login_attempts.insert(attempts_key, attempts + 1).await;
        return Err(AppError::InvalidCredentials);
    }
    state.cache.login_attempts.invalidate(&attempts_key).await;
    let user = user.expect("utilisateur authentifié");
    users::touch_login(&state.db, user.id).await?;
    Ok(user)
}

pub async fn request_magic_link(state: &AppState, email: &str) -> AppResult<()> {
    let email = normalize_email(email)?;
    throttle(state, "magic", &email).await?;
    let Some(user) = users::find_by_email(&state.db, &email).await? else {
        return Ok(());
    };
    if user.role != Role::Subscriber || !user.is_active || user.email_verified_at.is_none() {
        return Ok(());
    }
    let token = tokens::create(&state.db, user.id, Purpose::MagicLogin).await?;
    let url = state.mailer.link(&format!("/auth/lien-magique?token={token}"));
    state
        .mailer
        .send_in_background(user.email, templates::magic_link(&state.mailer.site_name, &url));
    Ok(())
}

pub async fn consume_magic_link(state: &AppState, token: &str) -> AppResult<User> {
    let (user_id, _) = tokens::consume(&state.db, token, &[Purpose::MagicLogin])
        .await?
        .ok_or(AppError::InvalidToken)?;
    let user = users::find_by_id(&state.db, user_id).await?.ok_or(AppError::InvalidToken)?;
    if user.role != Role::Subscriber || !user.is_active {
        return Err(AppError::InvalidToken);
    }
    users::touch_login(&state.db, user.id).await?;
    Ok(user)
}

async fn send_password_reset(state: &AppState, user: &User) -> AppResult<()> {
    let token = tokens::create(&state.db, user.id, Purpose::ResetPassword).await?;
    let url = state.mailer.link(&format!("/auth/mot-de-passe?token={token}"));
    state
        .mailer
        .send_in_background(user.email.clone(), templates::reset_password(&state.mailer.site_name, &url));
    Ok(())
}

/// « Mot de passe oublié » (non connecté).
pub async fn request_password_reset(state: &AppState, email: &str) -> AppResult<()> {
    let email = normalize_email(email)?;
    throttle(state, "reset", &email).await?;
    match users::find_by_email(&state.db, &email).await? {
        Some(user) if user.role == Role::User && user.is_active => send_password_reset(state, &user).await,
        _ => Ok(()),
    }
}

/// Modification du mot de passe depuis l'espace compte : passe aussi par un lien email.
pub async fn request_password_change(state: &AppState, user: &Arc<User>) -> AppResult<()> {
    if user.role != Role::User {
        return Err(AppError::Forbidden);
    }
    throttle(state, "reset", &user.email).await?;
    send_password_reset(state, user).await
}

/// Crée ou synchronise le compte administrateur unique à partir de la configuration.
pub async fn ensure_admin(state: &AppState) -> anyhow::Result<()> {
    let email = &state.config.admin_email;
    let existing = users::find_admin(&state.db).await?;
    match (existing, &state.config.admin_password) {
        (None, Some(pass)) => {
            password::validate(pass).map_err(|e| anyhow::anyhow!("ADMIN_PASSWORD : {e}"))?;
            let hash = password::hash(pass.clone()).await.map_err(|e| anyhow::anyhow!("{e}"))?;
            users::create_admin(&state.db, email, &hash).await?;
            tracing::info!(%email, "compte administrateur créé");
        }
        (None, None) => {
            tracing::warn!("aucun administrateur : définissez ADMIN_PASSWORD pour le créer");
        }
        (Some(admin), Some(pass)) => {
            anyhow::ensure!(
                admin.email.eq_ignore_ascii_case(email),
                "l'administrateur existant ({}) ne correspond pas à ADMIN_EMAIL",
                admin.email
            );
            let same = match &admin.password_hash {
                Some(hash) => password::verify(pass.clone(), hash.clone()).await,
                None => false,
            };
            if !same {
                password::validate(pass).map_err(|e| anyhow::anyhow!("ADMIN_PASSWORD : {e}"))?;
                let hash = password::hash(pass.clone()).await.map_err(|e| anyhow::anyhow!("{e}"))?;
                users::set_password(&state.db, admin.id, &hash).await?;
                tracing::info!("mot de passe administrateur mis à jour depuis ADMIN_PASSWORD");
            }
        }
        (Some(_), None) => {}
    }
    Ok(())
}

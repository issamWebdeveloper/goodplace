use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;

use crate::{
    app::AppState,
    error::AppResult,
    modules::users::model::{User, UserView},
};

use super::{
    dto::*,
    extractor::{CurrentUser, MaybeUser},
    service, session,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/subscribe", post(subscribe))
        .route("/verify-email", post(verify_email))
        .route("/password", post(set_password))
        .route("/password/forgot", post(forgot_password))
        .route("/password/change-request", post(change_password_request))
        .route("/login", post(login))
        .route("/magic-link", post(magic_link))
        .route("/magic-link/consume", post(consume_magic_link))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

fn open_session(state: &AppState, jar: CookieJar, user: &User) -> AppResult<(CookieJar, Json<UserView>)> {
    let token = session::issue(&state.config, user)?;
    Ok((jar.add(session::cookie(&state.config, token)), Json(UserView::from(user))))
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<MessageResponse>)> {
    service::register(&state, &body.name, &body.email).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse::new(
            "Si cette adresse peut être utilisée, un email de confirmation vient d'être envoyé.",
        )),
    ))
}

async fn subscribe(
    State(state): State<AppState>,
    Json(body): Json<EmailRequest>,
) -> AppResult<(StatusCode, Json<MessageResponse>)> {
    service::subscribe(&state, &body.email).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse::new(
            "Merci ! Consultez votre boîte mail pour confirmer votre inscription.",
        )),
    ))
}

async fn verify_email(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<TokenRequest>,
) -> AppResult<(CookieJar, Json<VerifyEmailResponse>)> {
    let outcome = service::verify_email(&state, &body.token).await?;
    let role = outcome.user.role;
    // Un abonné a prouvé la possession de son email : on ouvre directement sa session.
    if role == crate::modules::users::model::Role::Subscriber {
        let (jar, Json(user)) = open_session(&state, jar, &outcome.user)?;
        return Ok((jar, Json(VerifyEmailResponse { role, set_password_token: None, user: Some(user) })));
    }
    Ok((
        jar,
        Json(VerifyEmailResponse { role, set_password_token: outcome.set_password_token, user: None }),
    ))
}

async fn set_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<SetPasswordRequest>,
) -> AppResult<(CookieJar, Json<UserView>)> {
    let user = service::set_password(&state, &body.token, &body.password).await?;
    open_session(&state, jar, &user)
}

async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<EmailRequest>,
) -> AppResult<(StatusCode, Json<MessageResponse>)> {
    service::request_password_reset(&state, &body.email).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse::new(
            "Si un compte correspond à cette adresse, un lien valable 10 minutes vient d'être envoyé.",
        )),
    ))
}

async fn change_password_request(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<(StatusCode, Json<MessageResponse>)> {
    service::request_password_change(&state, &user).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse::new(
            "Un lien valable 10 minutes vient d'être envoyé à votre adresse email.",
        )),
    ))
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserView>)> {
    let user = service::login(&state, &body.email, &body.password).await?;
    open_session(&state, jar, &user)
}

async fn magic_link(
    State(state): State<AppState>,
    Json(body): Json<EmailRequest>,
) -> AppResult<(StatusCode, Json<MessageResponse>)> {
    service::request_magic_link(&state, &body.email).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(MessageResponse::new(
            "Si vous êtes abonné, un lien de connexion valable 10 minutes vient d'être envoyé.",
        )),
    ))
}

async fn consume_magic_link(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<TokenRequest>,
) -> AppResult<(CookieJar, Json<UserView>)> {
    let user = service::consume_magic_link(&state, &body.token).await?;
    open_session(&state, jar, &user)
}

async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, StatusCode) {
    (jar.add(session::removal_cookie(&state.config)), StatusCode::NO_CONTENT)
}

/// Session courante, ou `null` pour un visiteur anonyme.
async fn me(MaybeUser(user): MaybeUser) -> Json<Option<UserView>> {
    Json(user.map(|u| UserView::from(u.as_ref())))
}

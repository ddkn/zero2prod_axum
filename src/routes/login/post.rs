use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::error_chain_fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use secrecy::Secret;
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(skip(form, pool), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(
    Extension(pool): Extension<SqlitePool>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current()
        .record("username", tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool).await.map_err(
        |e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => {
                LoginError::UnexpectedError(e.into())
            }
        },
    )?;

    tracing::Span::current()
        .record("user_id", tracing::field::display(&user_id));

    Ok(Redirect::to("/"))
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        match self {
            LoginError::AuthError(_) => {
                (StatusCode::UNAUTHORIZED).into_response()
            }
            LoginError::UnexpectedError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}

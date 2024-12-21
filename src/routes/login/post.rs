use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::error_chain_fmt;
use crate::startup::HmacSecret;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(skip(form, pool, secret), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(
    Extension(pool): Extension<SqlitePool>,
    Extension(secret): Extension<HmacSecret>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current()
        .record("username", tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current()
                .record("user_id", tracing::field::display(&user_id));
            Ok(Redirect::to("/"))
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => {
                    let query_string =
                        encoded_hmac_error(&e.to_string(), secret);
                    LoginError::AuthError(e.into(), query_string)
                }
                AuthError::UnexpectedError(_) => {
                    LoginError::UnexpectedError(e.into())
                }
            };
            Err(e)
        }
    }
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error, String),
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
            LoginError::AuthError(_, query_string) => {
                Redirect::to(&format!("/login?{query_string}")).into_response()
            }
            LoginError::UnexpectedError(_) => {
                tracing::error!(error = ?self, "Login Error");
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}

fn encoded_hmac_error(error: &str, secret: HmacSecret) -> String {
    let encoded_error = format!("error={}", urlencoding::Encoded::new(error));
    let hmac_tag = {
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
            secret.0.expose_secret().as_bytes(),
        )
        .unwrap();
        mac.update(encoded_error.as_bytes());
        mac.finalize().into_bytes()
    };
    format!("{encoded_error}&tag={hmac_tag:x}")
}

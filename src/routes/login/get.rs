use crate::startup::HmacSecret;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Extension;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string =
            format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
            secret.0.expose_secret().as_bytes(),
        )
        .unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(
    query: Option<Query<QueryParams>>,
    secret: Extension<HmacSecret>,
) -> impl IntoResponse {
    let error_html = match query {
        None => "".into(),
        Some(Query(query)) => match query.verify(&secret) {
            Ok(error) => {
                format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
            }
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "failed to verify query parameters using he HMAC tag"
                );
                "".into()
            }
        },
    };
    let login_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
  </head>
  <body>
    {error_html}
    <form method="post">
      <label>Username
        <input
          type="text"
          placeholder="Enter Username"
          name="username"
        >
      </label>
      <label>Password
        <input
          type="password"
          placeholder="Enter Password"
          name="password"
        >
      </label>

      <button type="submit">Login</button>
    </form>
  </body>
</html>"#
    );
    (StatusCode::OK, Html::from(login_html))
}

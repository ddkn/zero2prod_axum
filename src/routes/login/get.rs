use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(Query(query): Query<QueryParams>) -> impl IntoResponse {
    let error_html = match query.error {
        Some(e) => format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&e)),
        None => "".into(),
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

//
// Last Modification: 2024-08-09 22:39:55
//

use crate::models::users;
use crate::models::tokens;
use anyhow::Result;

use axum::{
    extract::{Extension, FromRequestParts, Query},
    http::{request::Parts, HeaderMap, HeaderValue, StatusCode},
    response::Html,
};

use tera::{Tera, Context};
use std::collections::HashMap;

pub struct RequireAuth {
    pub role: users::UserRoles,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    // type Rejection = Redirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {

        // let session = SessionPgSession::from_request_parts(parts, state)
        //     .await
        //     .expect("Missing session");

        let Extension(pool) = Extension::<sqlx::Pool<sqlx::Postgres>>::from_request_parts(parts, state)
            .await
            .expect("Missing PgPool");

        println!("parts {:?}", parts);
        // println!("headers {:?}", parts.headers);

        let mut cookies = HashMap::new();

        if let Some(cookie_str) = parts.headers
            .get(axum::http::header::COOKIE)
            .and_then(|value| value.to_str().ok()) {

            println!("Cookie {:?}", cookie_str);

            for cookie in cookie_str.split("; ") {
                let mut parts = cookie.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    cookies.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        if let Some(token) = cookies.get("token") {
            // println!("token: {}", token);

            let token_manager = tokens::Tokens::new(pool);
            match token_manager.is_valid(&token).await {
                Ok(role) => {
                    return Ok(Self {
                        role,
                    });
                },
                Err(err) => println!("error: {:?}", err),
            };
        }

        // if parts.method == axum::http::Method::GET && parts.uri.path() == "/admin" {
        //    return Ok(Self {
        //        role: types::UserRoles::Anonymous,
        //    });
        // }

        // Err(Redirect::to("/admin"))
        Err(StatusCode::UNAUTHORIZED)

    }
}


pub async fn login(
    Query(parameters): Query<HashMap<String, String>>,
    Extension(tera): Extension<Tera>) -> (HeaderMap, Html<String>) {

    let mut headers = HeaderMap::new();
    if parameters.contains_key("action") && parameters.get("action").unwrap() == "logout" {
        headers
            .insert(axum::http::header::SET_COOKIE, HeaderValue::from_str("token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT")
            .unwrap());
    }

    let data = Context::new();
    let rendered = tera.render("login.html", &data).unwrap();
    return (headers, Html(rendered));
}
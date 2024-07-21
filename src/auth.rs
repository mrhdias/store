//
// Last Modification: 2024-06-29 20:32:35
//

use crate::types;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use anyhow::{Result, anyhow};

use axum::{
    extract::{Extension, FromRequestParts},
    http::{request::Parts, StatusCode},
};

use sqlx::Row;

use std::collections::HashMap;

pub struct RequireAuth {
    pub role: types::UserRoles,
}

async fn token_is_valid(
    pool: &sqlx::Pool<sqlx::Postgres>,
    token: &str) -> Result<types::UserRoles, anyhow::Error> {

    let row = sqlx::query(r#"
        SELECT tokens.user_id, tokens.expires, users.role FROM tokens, users WHERE token = $1;
    "#)
        .bind(token)
        .fetch_one(pool)
        .await?;
    
    println!("TOKEN IS VALID: {}", token);

    let role = row.get::<types::UserRoles, _>("role");
    let expires = row.get::<NaiveDateTime, _>("expires");

    // let expires_utc: DateTime<Utc> = DateTime::from_utc(expires, Utc);
    let expires_utc: DateTime<Utc> = Utc.from_utc_datetime(&expires);

    println!(">>> Expires: {}", expires);

    let now: DateTime<Utc> = Utc::now();
    if now > expires_utc {
        // Remove the expired token
        sqlx::query(r#"
            DELETE FROM tokens WHERE token = $1;
        "#)
        .bind(token)
        .execute(pool)
        .await?;

        return Err(anyhow!("Token expired"));
    }

    // if expired remove token

    println!("Role {:?}: {:?}", role, expires);

    Ok(role)
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
        println!("headers {:?}", parts.headers);

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

        // println!("Cookies {:?}", cookies);

        if let Some(token) = cookies.get("token") {
            // println!("token: {}", token);

            match token_is_valid(&pool, &token).await {
                Ok(role) => {
                    return Ok(Self {
                        role,
                    });
                },
                Err(err) => println!("error: {:?}", err),
            }

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

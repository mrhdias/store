//
// Last Modification: 2024-08-09 22:39:40
//

use crate::models::users;
use anyhow::{Result, anyhow};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc, Duration};
use uuid::Uuid;
use sqlx::Row;

pub struct Tokens {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Tokens {

    pub async fn delete(&self, token: &str) -> Result<(), anyhow::Error> {
        // Implementation to delete a token
        sqlx::query(r#"
            DELETE FROM tokens WHERE token = $1;
        "#)
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn is_valid(&self, token: &str) -> Result<users::UserRoles, anyhow::Error> {
        let row = sqlx::query(r#"
            SELECT tokens.user_id, tokens.expires, users.role FROM tokens, users WHERE token = $1;
        "#)
            .bind(token)
            .fetch_one(&self.pool)
            .await?;

        println!("TOKEN IS VALID: {}", token);

        let role = row.get::<users::UserRoles, _>("role");
        let expires = row.get::<NaiveDateTime, _>("expires");
        
        // let expires_utc: DateTime<Utc> = DateTime::from_utc(expires, Utc);
        let expires_utc: DateTime<Utc> = Utc.from_utc_datetime(&expires);

        println!(">>> Expires: {}", expires);

        let now: DateTime<Utc> = Utc::now();
        if now > expires_utc {
            // Remove the expired token
            let error = match self.delete(token).await {
                Ok(_) => "Token expired".to_string(),
                Err(err) => format!("Failed to delete expired token: {}", err),
            };

            return Err(anyhow!(error));
        }

        // if expired remove token

        println!("Role {:?}: {:?}", role, expires);

        Ok(role)
    }

    pub async fn add(&self, user_id: &i32) -> Result<String, anyhow::Error> {
        // Implementation to add a new token

        let token = Uuid::new_v4().to_string();
        let expires_time = Utc::now().naive_utc() + Duration::hours(24);

        sqlx::query(r#"
            INSERT INTO tokens (token, user_id, expires) VALUES ($1, $2, $3)
            ON CONFLICT (user_id)
            DO UPDATE SET token = EXCLUDED.token, expires = EXCLUDED.expires;
        "#)
            .bind(&token)
            .bind(&user_id)
            .bind(&expires_time)
            .execute(&self.pool)
            .await?;

        Ok(token)
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Tokens {
            pool,
        }
    }
}
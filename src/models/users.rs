//
// Model Users
//

use crate::types;
use anyhow;
use chrono::NaiveDateTime;

use sqlx::{
    postgres::PgRow,
    Row,
};

use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_roles", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum UserRoles {
    Admin,
    Customer,
    Guest,
}

impl UserRoles {
    fn as_str(&self) -> &str {
        match self {
            UserRoles::Admin => "admin",
            UserRoles::Customer => "customer",
            UserRoles::Guest => "guest",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: i32,
    username: String,
    email: String,
    phone: String,
    first_name: String,
    last_name: String,
    role: UserRoles,
    avatar_url: String,
    date_created: NaiveDateTime,
    date_modified: NaiveDateTime,
}

pub struct Users {
    pool: sqlx::Pool<sqlx::Postgres>,
    pub total_count: i32,
}

impl Users {

    fn update(&self) {
        // Implementation to update a user by ID
    }

    fn delete(&self) {
        // Implementation to delete a user by ID
    }

    pub async fn get_all(&self,
        page: i32,
        per_page: i32,
        order: types::Order) -> Result<Vec<User>, anyhow::Error> {
        // Implementation to get all users

        let offset = (page - 1) * per_page;

        let users = sqlx::query(&format!(r#"
            SELECT
                id, username, first_name, last_name, email, phone, role, avatar_url, date_created, date_modified
            FROM users
            ORDER BY date_created {}
            LIMIT $1 OFFSET $2;
        "#, order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| User {
                id: row.get::<i32, _>("id"),
                username: row.get::<String, _>("username"),
                first_name: row.get::<String, _>("first_name"),
                last_name: row.get::<String, _>("last_name"),
                email: row.get::<String, _>("email"),
                phone: row.get::<String, _>("phone"),
                role: row.get::<UserRoles, _>("role"),
                avatar_url: row.get::<String, _>("avatar_url"),
                date_created: row.get::<NaiveDateTime, _>("date_created"),
                date_modified: row.get::<NaiveDateTime, _>("date_modified"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    fn get(&self) {
        // Implementation to get a user by ID
    }

    fn add(&self) {
        // Implementation to add a new user
    }

    pub async fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("Failed to count users");

        Users {
            pool,
            total_count: total_count.0 as i32,
        }
    }
}
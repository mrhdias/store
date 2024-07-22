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

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_roles", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRoles {
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

    pub fn is_admin(&self) -> bool {
        self == &UserRoles::Admin
    }
}

pub struct Cardentials {
    pub user_id: i32,
    pub password: String,
    pub role: UserRoles,
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
}

impl Users {

    fn update(&self) {
        // Implementation to update a user by ID
    }

    fn delete(&self) {
        // Implementation to delete a user by ID
    }

    pub async fn cardentials(&self, user: &str) -> Result<Cardentials, anyhow::Error> {
        // Implementation to get users by username and password

        let row = sqlx::query(r#"
            SELECT id, password, role FROM users WHERE email = $1;
        "#)
            .bind(user)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Cardentials {
                user_id: row.get::<i32, _>("id"),
                password: row.get::<String, _>("password"),
                role: row.get::<UserRoles, _>("role"),
            })
        } else {
            Err(anyhow::Error::msg("User not found"))
        }
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

    pub async fn count(&self) -> Result<i32, anyhow::Error> {
        let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok(total_count.0 as i32)
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Users {
            pool,
        }
    }
}
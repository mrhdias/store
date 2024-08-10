//
// Last Modification: 2024-08-09 19:01:11
//

use crate::types;

use chrono::NaiveDateTime;
use anyhow;

use sqlx::{
    postgres::PgRow,
    Row,
};

use serde::{
    Deserialize,
    Serialize,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRoles,
    pub avatar_url: String,
    pub date_created: NaiveDateTime,
    pub date_modified: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserShort {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRoles,
    pub avatar_url: String,
    pub date_created: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPage {
    pub users: Vec<UserShort>,
    pub total_count: i32,
    pub current_page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl UserPage {
    pub fn new() -> Self {
        UserPage {
            users: Vec::new(),
            total_count: 0,
            current_page: 0,
            per_page: 0,
            total_pages: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Parameters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<types::Order>,
    pub order_by: Option<String>, // id, include, name, slug, term_group, description and count. Default is name
    pub exclude: Option<String>, // array - Ensure result set excludes specific IDs.
    pub include: Option<String>, // array - Limit result set to specific ids.
    pub email: Option<String>,
    pub role: Option<UserRoles>,
}

pub struct Cardentials {
    pub user_id: i32,
    pub password: String,
    pub role: UserRoles,
}

fn users_order_by(parameter: &Option<String>) -> &str {
    match parameter.as_ref() {
        Some(v) => match v.as_str() {
            "id" => "id",
            // "included" => "included",
            "name" => "first_name, last_name",
            "registered_date" => "date_created",
            _ => "first_name, last_name", // The default case
        },
        None => "first_name, last_name",
    }
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

    fn get(&self) {
        // Implementation to get a user by ID
    }

    fn add(&self) {
        // Implementation to add a new user
    }

    pub async fn get_page(&self,
        parameters: &Parameters,
    ) -> Result<UserPage, anyhow::Error> {

        let per_page = parameters.per_page.unwrap_or(3) as i32;

        let order = parameters.order.as_ref().unwrap_or(&types::Order::Asc);
        let order_by = users_order_by(&parameters.order_by);

        let total: (i64, ) = sqlx::query_as(r#"
            SELECT COUNT(*) FROM users;
        "#)
            .fetch_one(&self.pool)
            .await?;

        if total.0 == 0 {
            return Ok(UserPage {
                users: vec![],
                total_pages: 0,
                current_page: 0,
                total_count: 0,
                per_page: 0,
            });
        }

        let total_pages: i32 = (total.0 as f32 / per_page as f32).ceil() as i32;

        let page = || -> i32 {
            let page = parameters.page.unwrap_or(1) as i32;
            if page > total_pages {
                return total_pages;
            }
            if page == 0 {
                return 1;
            }
            page
        }();

        let offset = (page - 1) * per_page;

        let users = sqlx::query(&format!(r#"
            SELECT
                id, username, first_name, last_name, email, phone, role, avatar_url, date_created, date_modified
            FROM users
            ORDER BY {} {}
            LIMIT $1 OFFSET $2;
        "#, order_by, order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| UserShort {
                id: row.get::<i32, _>("id"),
                username: row.get::<String, _>("username"),
                email: row.get::<String, _>("email"),
                first_name: row.get::<String, _>("first_name"),
                last_name: row.get::<String, _>("last_name"),
                role: row.get::<UserRoles, _>("role"),
                avatar_url: row.get::<String, _>("avatar_url"),
                date_created: row.get::<NaiveDateTime, _>("date_created"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(UserPage {
            users,
            total_pages,
            current_page: page,
            total_count: total.0 as i32,
            per_page,
        })
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Users {
            pool,
        }
    }
}
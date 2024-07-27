//
// Last Mofification: 2024-07-27 18:35:21
//

use crate::types;
use crate::models::users::Users;

use axum::{
    extract::{Extension, Path, Query},
    response::Html,
};

use tera::{
    Tera,
    Context
};


pub async fn edit(
    Path(id):Path<i32>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    Html("edit user".to_string())
}

pub async fn list(
    Query(pagination): Query<types::Pagination>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    let users_manager = Users::new(pool);
    let total_count = users_manager.count().await.unwrap_or(0);

    let per_page = pagination.per_page.unwrap_or(10);

    let total_pages: i32 = (total_count as f32 / per_page as f32).ceil() as i32;

    match users_manager.get_all(
        pagination.page.unwrap_or(1) as i32, 
        pagination.per_page.unwrap_or(10) as i32,
        pagination.order.unwrap_or(types::Order::Desc)).await {
        Ok(users) => {
            let mut data = Context::new();
            data.insert("partial", "users");
            data.insert("users", &users);
            data.insert("current_page", &pagination.page.unwrap_or(1));
            data.insert("total_users", &total_count);
            data.insert("per_page", &per_page);
            data.insert("total_pages", &total_pages);
            let rendered = tera.render("backend/admin.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html(String::from("An error occurred while fetching users."))
        },
    }
}
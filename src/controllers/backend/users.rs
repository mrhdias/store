//
// Last Mofification: 2024-08-09 19:21:51
//

use crate::models::users;

use axum::{
    extract::{Extension, Path, Query},
    response::Html,
};

use tera::{
    Tera,
    Context
};

pub async fn new(
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
) -> Html<String> {

    Html("Add new user unimplemented".to_string())
}

pub async fn edit(
    Path(id):Path<i32>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    Html("edit user unimplemented".to_string())
}

pub async fn list(
    Query(parameters): Query<users::Parameters>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {

    let users_manager = users::Users::new(pool);

    let page = match users_manager
        .get_page(&parameters)
        .await {
        Ok(page) => page,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error happened while fetching users".to_string());
        },
    };

    let mut data = Context::new();
    data.insert("partial", "users");
    data.insert("title", "Users");
    data.insert("users", &page.users);
    data.insert("current_page", &page.current_page);
    data.insert("total_users", &page.total_count);
    data.insert("per_page", &page.per_page);
    data.insert("total_pages", &page.total_pages);

    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}
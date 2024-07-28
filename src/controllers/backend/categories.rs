//
// Description: Manage product categories.
// Last Modification: 2024-07-28 17:25:47
//

use crate::models::categories;

use axum::{
    extract::Extension,
    response::Html,
};

use tera::{
    Tera,
    Context
};

pub async fn list(
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    let categories_manager = categories::Categories::new(pool);


    let mut data = Context::new();
    data.insert("partial", "categories");
    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}

//
// Description: Manage product categories.
// Last Modification: 2024-07-28 17:25:47
//

use crate::models::categories;

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

    Html("Add new category unimplemented".to_string())
}

pub async fn edit(
    Path(id):Path<i32>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {

    Html("Edit category unimplemented".to_string())
}

pub async fn list(
    Query(parameters): Query<categories::Parameters>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {

    let categories_manager = categories::Categories::new(pool);

    let page = match categories_manager.backend()
        .get_page(&parameters)
        .await {
        Ok(page) => page,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error happened while fetching products".to_string());
        },
    };

    let mut data = Context::new();
    data.insert("partial", "categories");
    data.insert("categories", &page.categories);
    data.insert("current_page", &page.current_page);
    data.insert("total_categories", &page.total_count);
    data.insert("per_page", &page.per_page);
    data.insert("total_pages", &page.total_pages);

    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}

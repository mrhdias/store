//
// Description: Manage product categories.
// Last Modification: 2024-08-14 22:19:37
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


    let categories_manager = categories::Categories::new(pool.clone());

    match categories_manager.backend().get(id).await {
        Ok(category) => {
            let mut data = Context::new();
            data.insert("partial", "category");
            data.insert("title", "Category");
            data.insert("category", &category);
        
            let rendered = tera.render("backend/admin.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error happened while fetching category".to_string());
        },
    }
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
    data.insert("title", "Categories");
    data.insert("categories", &page.categories);
    data.insert("tree", &parameters.order_by.is_none());
    data.insert("current_page", &page.current_page);
    data.insert("total_categories", &page.total_count);
    data.insert("per_page", &page.per_page);
    data.insert("total_pages", &page.total_pages);

    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}

//
// Last Modification: 2024-06-27 12:05:50
//

use crate::models::media;

use axum::{
    extract::Extension,
    response::Html,
};

use tera::{
    Tera,
    Context
};

pub async fn library(
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    let media_manager = media::MediaLibrary::new(pool);
    let rows = media_manager
        .get_all()
        .await
        .expect("Failed to fetch media list");

    let mut data = Context::new();
    data.insert("library", &rows);
    let rendered = tera.render("admin/media.html", &data).unwrap();
    Html(rendered)
}

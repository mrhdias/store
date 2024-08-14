//
// Description: Manage product media.
// Last Modification: 2024-07-27 18:36:44
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
    data.insert("partial", "media");
    data.insert("title", "Media");
    data.insert("library", &rows);
    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}

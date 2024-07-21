//
// Last Modification: 2024-06-27 12:05:50
//

use axum::{
    extract::Extension,
    response::Html,
};

use tera::{
    Tera,
    Context
};

use sqlx::{
    postgres::PgRow,
    Row
};

use chrono::NaiveDateTime;

use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize)]
struct Media {
    id: i32,
    src: String,
    name: String,
    alt: String,
    date_created: String,
    date_modified: String,
}

pub async fn library(
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>) -> Html<String> {

    let rows = sqlx::query(r#"
        SELECT
            id, src, name, alt, date_created, date_modified
        FROM media
        ORDER BY date_created DESC;
    "#)
        .map(|row: PgRow| Media {
            id: row.get::<i32, _>("id"),
            src: row.get::<String, _>("src"),
            name: row.get::<String, _>("name"),
            alt: row.get::<String, _>("alt"),
            date_created: || -> String {
                let date_created = row.get::<NaiveDateTime, _>("date_created");
                date_created.format("%Y/%m/%d at %H:%M:%S").to_string()
            }(),
            date_modified: || -> String {
                let date_modified = row.get::<NaiveDateTime, _>("date_modified");
                date_modified.format("%Y/%m/%d at %H:%M:%S").to_string()
            }(),
        })
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch media list");

    let mut data = Context::new();
    data.insert("library", &rows);
    let rendered = tera.render("admin/media.html", &data).unwrap();
    Html(rendered)
}

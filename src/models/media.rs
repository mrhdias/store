//
// Last Modification: 2024-07-23 19:29:54
//

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
pub struct Media {
    id: i32,
    src: String,
    name: String,
    alt: String,
    date_created: String,
    date_modified: String,
}

pub struct MediaLibrary {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl MediaLibrary {

    pub async fn get_all(&self) -> Result<Vec<Media>, anyhow::Error> {
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
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        MediaLibrary {
            pool,
        }
    }
}
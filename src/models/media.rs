//
// Last Modification: 2024-08-14 20:33:09
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
    date_created: Option<String>,
    date_modified: Option<String>,
}

impl Media {

    pub fn default() -> Media {
        Media {
            id: 0,
            src: "".to_string(),
            name: "".to_string(),
            alt: "".to_string(),
            date_created: None,
            date_modified: None,
        }
    }
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
                date_created: || -> Option<String> {
                    let date_created = row.get::<NaiveDateTime, _>("date_created");
                    Some(date_created.format("%Y/%m/%d at %H:%M:%S").to_string())
                }(),
                date_modified: || -> Option<String> {
                    let date_modified = row.get::<NaiveDateTime, _>("date_modified");
                    Some(date_modified.format("%Y/%m/%d at %H:%M:%S").to_string())
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
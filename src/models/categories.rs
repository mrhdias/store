//
// Last Modification: 2024-08-14 19:17:46
//

use crate::types;
use crate::models::backend;
use crate::models::media;

use anyhow::Ok;
use slug::slugify;

use sqlx::{
    postgres::PgRow,
    Row,
    types::Json,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Parameters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<types::Order>,
    pub order_by: Option<String>, // id, include, name, slug, term_group, description and count. Default is name
    pub exclude: Option<String>, // array - Ensure result set excludes specific IDs.
    pub include: Option<String>, // array - Limit result set to specific ids.
    pub product: Option<i32>,
    pub parent: Option<i32>,
    pub slug: Option<String>,
}

fn categories_order_by(parameter: &Option<String>) -> &str {
    match parameter.as_ref() {
        Some(v) => match v.as_str() {
            "id" => "id",
            // "included" => "included",
            "name" => "name",
            "slug" => "slug",
            // "term_group" => "?",
            // "description" => "?",
            // "count" => "?",
            _ => "path", // The default case
        },
        None => "path",
    }
}

pub struct Categories {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Categories {

    pub fn backend(&self) -> Backend {
        Backend::new(&self.pool)
    }

    pub fn frontend(&self) -> Frontend {
        Frontend::new(&self.pool)
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        // Implementation to create a new instance of Categories

        Categories {
            pool,
        }
    }
}


pub struct Frontend<'a> {
    pool: &'a sqlx::Pool<sqlx::Postgres>,
}

impl<'a> Frontend<'a> {

    pub fn new(pool: &'a sqlx::Pool<sqlx::Postgres>) -> Self {
        Frontend {
            pool,
        }
    }
}

pub struct Backend<'a> {
    pool: &'a sqlx::Pool<sqlx::Postgres>,
}

impl<'a> Backend<'a> {

    pub async fn get_tree(&self) -> Result<Vec<backend::CategoryTree>, anyhow::Error> {
        // Implementation to get categories
    
        // Execute the query and fetch the results
        let categories: Vec<backend::CategoryTree> = sqlx::query_as::<_, backend::CategoryTree>(r#"
            WITH RECURSIVE category_tree AS (
                SELECT id, name, slug, parent, name::VARCHAR AS path, EXISTS(SELECT 1 FROM categories c2 WHERE c2.parent = c.id) AS has_childs, 1 AS branches FROM categories c WHERE parent = 0
                UNION ALL
                SELECT c.id, c.name, c.slug, c.parent, (ct.path || ' > ' || c.name)::VARCHAR AS path, EXISTS(SELECT 1 FROM categories c2 WHERE c2.parent = c.id) AS has_childs, ct.branches + 1 AS branches FROM categories c
                INNER JOIN category_tree ct ON ct.id = c.parent
            )
            SELECT id, name, slug, parent, path, has_childs, branches FROM category_tree ORDER BY path;
        "#)
            .fetch_all(self.pool)
            .await?;

        // for category in categories {
        //    println!("{:?}", category);
        // }

        Ok(categories)
    }

    pub async fn add(&self, name: &str, parent: i32) -> Result<i32, anyhow::Error> {
        // Implementation to add a new category
        let category_id: i32 = sqlx::query(r#"
            INSERT INTO categories (name, slug, parent)
            VALUES ($1, $2, $3) RETURNING id;
        "#)
            .bind(&name)
            .bind(slugify(&name))
            .bind(&parent)
            .fetch_one(self.pool)
            .await?
            .get(0);

        Ok(category_id)
    }

    pub async fn get(&self,
        id: i32,
    ) -> Result<backend::Category, anyhow::Error> {

        let row = sqlx::query(r#"
            SELECT
                name, slug, parent, description, media_id
            FROM categories WHERE id = $1;
        "#)
            .bind(&id)
            .fetch_one(self.pool)
            .await?;


        Ok(backend::Category{
            id,
            name: row.get::<String, _>("name"),
            slug: row.get::<String, _>("slug"),
            parent: row.get::<i32, _>("parent"),
            description: row.get::<String, _>("description"),
            image: Json(media::Media::default()),
        })
    }

    pub async fn get_page(&self,
        parameters: &Parameters,
    ) -> Result<backend::CategoryPage, anyhow::Error> {

        let per_page = parameters.per_page.unwrap_or(3) as i32;

        let order = parameters.order.as_ref().unwrap_or(&types::Order::Asc);
        let order_by = categories_order_by(&parameters.order_by);

        let total: (i64, ) = sqlx::query_as(r#"
            SELECT COUNT(*) FROM categories;
        "#)
            .fetch_one(self.pool)
            .await?;

        if total.0 == 0 {
            return Ok(backend::CategoryPage {
                categories: vec![],
                total_pages: 0,
                current_page: 0,
                total_count: 0,
                per_page: 0,
            });
        }

        let total_pages: i32 = (total.0 as f32 / per_page as f32).ceil() as i32;

        let page = || -> i32 {
            let page = parameters.page.unwrap_or(1) as i32;
            if page > total_pages {
                return total_pages;
            }
            if page == 0 {
                return 1;
            }
            page
        }();

        let offset = (page - 1) * per_page;

        let categories = sqlx::query(&format!(r#"
            WITH RECURSIVE category_tree AS (
                SELECT id, name, slug, parent, description, media_id, name::VARCHAR AS path,
                    EXISTS(SELECT 1 FROM categories c2 WHERE c2.parent = c.id) AS has_childs,
                    1 AS branches FROM categories c WHERE parent = 0
                UNION ALL
                SELECT c.id, c.name, c.slug, c.parent, c.description, c.media_id, (ct.path || ' > ' || c.name)::VARCHAR AS path,
                    EXISTS(SELECT 1 FROM categories c2 WHERE c2.parent = c.id) AS has_childs,
                    ct.branches + 1 AS branches FROM categories c
                INNER JOIN category_tree ct ON ct.id = c.parent
            ),
            product_count AS (SELECT category_id, COUNT(*) AS count FROM product_categories GROUP BY category_id),
            category_with_counts AS (
                SELECT ct.*, COALESCE(pc.count, 0) AS count FROM category_tree ct
                LEFT JOIN product_count pc ON ct.id = pc.category_id
            )
            SELECT id, name, slug, parent, description,
                COALESCE( (SELECT
                    jsonb_build_object('id', media.id, 'src', media.src, 'name', media.name, 'alt', media.alt, 'date_created', null, 'date_modified', null)
                    FROM media WHERE media.id=category_with_counts.media_id LIMIT 1), '{{"id": 0, "src": "../assets/images/placeholder-300x300.png", "name": "Thumbnail", "alt": "Thumbnail"}}') AS image,
                path, has_childs, branches, count
            FROM category_with_counts
                ORDER BY {} {}
            LIMIT $1 OFFSET $2;
        "#, order_by, order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| backend::CategoryShort {
                id: row.get::<i32, _>("id"),
                name: row.get::<String, _>("name"),
                slug: row.get::<String, _>("slug"),
                parent: row.get::<i32, _>("parent"),
                description: |desc: String| -> String {
                    if desc.is_empty() {
                        return "-".to_string();
                    }
                    desc
                }(row.get::<String, _>("description")),
                image: row.get::<Json<media::Media>, _>("image"),
                has_childs: row.get::<bool, _>("has_childs"),
                branches: row.get::<i32, _>("branches"),
                count: row.get::<i64, _>("count") as i32,
            })
            .fetch_all(self.pool)
            .await?;

        Ok(backend::CategoryPage {
            categories,
            total_pages,
            current_page: page,
            total_count: total.0 as i32,
            per_page,
        })
    }

    pub fn new(pool: &'a sqlx::Pool<sqlx::Postgres>) -> Self {
        Backend {
            pool,
        }
    }
}
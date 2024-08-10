//
// Last Modification: 2024-08-09 22:41:14
//

use crate::types;
use crate::models::backend;
use slug::slugify;

use sqlx::{
    postgres::PgRow,
    Row,
};

use serde::{
    Deserialize,
    Serialize,
};

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    id: i32,
    name: String,
    slug: String,
    parent: i32,
    path: String,
    has_childs: bool, // if has childs
    branches: i32, // number of branches in the tree
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
            _ => "name", // The default case
        },
        None => "name",
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

    pub async fn get_all(&self) -> Result<Vec<Category>, anyhow::Error> {
        // Implementation to get categories
    
        // Execute the query and fetch the results
        let categories: Vec<Category> = sqlx::query_as::<_, Category>(r#"
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
            SELECT categories.id, categories.name, categories.slug, categories.parent, categories.description, (
                SELECT COUNT(*) FROM product_categories WHERE product_categories.category_id=categories.id
            ) AS count
            FROM categories
            ORDER BY
                categories.{} {}
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
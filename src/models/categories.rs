//
// Last Modification: 2024-07-22 18:59:33
//


use slug::slugify;

use sqlx::Row;

use serde::{
    Serialize,
    Deserialize
};

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

pub struct Categories {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Categories {

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
            .fetch_all(&self.pool)
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
            .fetch_one(&self.pool)
            .await?
            .get(0);

        Ok(category_id)
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        // Implementation to create a new instance of Categories

        Categories {
            pool,
        }
    }
}
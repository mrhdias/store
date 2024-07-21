//
// Model Products
//

use crate::types;

use anyhow;
use num_traits::ToPrimitive;

use strum::EnumIter;

use sqlx::{
    postgres::PgRow,
    types::{Json, Decimal},
    FromRow,
    Row
};

use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, EnumIter)]
#[sqlx(type_name = "status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Draft,
    Pending,
    Private,
    Publish,
}

impl Status{
    fn as_str(&self) -> &str {
        match self {
            Status::Draft => "draft",
            Status::Pending => "pending",
            Status::Private => "private",
            Status::Publish => "publish",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "stock_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum StockStatus {
    InStock,
    OutOfStock,
    OnBackorder
}

impl StockStatus {
    fn as_str(&self) -> &str {
        match self {
            StockStatus::InStock => "instock",
            StockStatus::OutOfStock => "outofstock",
            StockStatus::OnBackorder => "onbackorder",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Media {
    id: i32,
    src: String,
    name: String,
    alt: String,
    position: i32,
}

impl Media {
    fn clone(&self) -> Media {
        Media {
            id: self.id,
            src: self.src.to_string(),
            name: self.name.to_string(),
            alt: self.alt.to_string(),
            position: self.position,
        }
    }

    fn default(& mut self) {
        self.id = 0;
        self.src = "".to_string();
        self.name = "".to_string();
        self.alt = "".to_string();
        self.position = 0;
    }

    fn new() -> Self {
        Media {
            id: 0,
            src: "".to_string(),
            name: "".to_string(),
            alt: "".to_string(),
            position: 0
        }
    }
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
    product_count: i64, // number of products
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ProductRow {
    id: i32,
    sku: String,
    name: String,
    slug: String,
    permalink: String,
    description: String,
    short_description: String,
    price: f32,
    sale_price: f32,
    regular_price: f32,
    on_sale: bool,
    stock_status: StockStatus,
    stock_quantity: i32,
    weight: i32,
    gallery: Json<Vec<Media>>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Product {
    id: i32,
    sku: String,
    name: String,
    slug: String,
    permalink: String,
    description: String,
    short_description: String,
    price: f32,
    regular_price: f32,
    sale_price: f32,
    on_sale: bool,
    stock_quantity: i32,
    stock_status: types::StockStatus,
    weight: i32,
    // categories: Vec<Category>,
    gallery: Json<Vec<Media>>,
}

pub struct Products {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Products {

    pub async fn count_all_category_by_slug(&self, slug: &str)  -> Result<i32, anyhow::Error> {
        let total_count: (i64, ) = sqlx::query_as(&format!(r#"
            SELECT COUNT(*)
                FROM products, product_categories, categories
            WHERE
                products.status = 'publish' AND
                products.id = product_categories.product_id AND
                categories.id = product_categories.category_id AND
                categories.slug = '{}';
        "#, slug))
            .fetch_one(&self.pool)
            .await?;

        Ok(total_count.0 as i32)
    }

    pub async fn get_category_by_slug(&self,
        slug: &str,
        page: i32,
        per_page: i32,
        order: types::Order) -> Result<Vec<ProductRow>, anyhow::Error> {

        let offset = (page - 1) * per_page;

        let products = sqlx::query(&format!(r#"
            SELECT
                products.id, products.sku, products.name, products.slug, products.permalink, products.description,
                products.short_description, products.price, products.regular_price, products.sale_price,
                products.on_sale, products.stock_quantity, products.stock_status, products.weight,
                products.date_created, products.date_modified,
                (SELECT (JSON_AGG(ti)::jsonb) FROM (
                    SELECT media.id, media.src, media.name, media.alt, media.date_created, media.date_modified, product_media.position
                    FROM media, product_media
                    WHERE product_media.product_id = products.id AND product_media.media_id = media.id
                    ORDER BY product_media.position
                ) ti
            ) AS gallery
            FROM products, product_categories, categories
            WHERE products.status = 'publish' AND products.id = product_categories.product_id AND categories.id = product_categories.category_id AND categories.slug = $1
            ORDER BY
                products.date_created {}
            LIMIT $2 OFFSET $3;
        "#, order.as_str()))
            .bind(slug)
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| ProductRow {
                id: row.get::<i32, _>("id"),
                sku: row.get::<String, _>("sku"),
                name: row.get::<String, _>("name"),
                slug: row.get::<String, _>("slug"),
                permalink: row.get::<String, _>("permalink"),
                description: row.get::<String, _>("description"),
                short_description: row.get::<String, _>("short_description"),
                price: match row.get::<Decimal, _>("price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                sale_price: match row.get::<Decimal, _>("sale_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                on_sale: row.get::<bool, _>("on_sale"),
                stock_quantity: row.get::<i32, _>("stock_quantity"),
                stock_status: row.get::<StockStatus, _>("stock_status"),
                weight: row.get::<i32, _>("weight"),
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(products)
    }

    pub async fn categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let categories: Vec<Category> = sqlx::query_as::<_, Category>(r#"
            WITH RECURSIVE category_tree AS (
                SELECT id, name, slug, parent, name::VARCHAR AS path,
                    EXISTS(SELECT 1 FROM categories c2 WHERE c2.parent = c.id) AS has_childs,
                    1 AS branches FROM categories c WHERE parent = 0
                UNION ALL
                SELECT c.id, c.name, c.slug, c.parent, (ct.path || ' > ' || c.name)::VARCHAR AS path,
                    EXISTS(SELECT 1 FROM categories c2 WHERE c2.parent = c.id) AS has_childs,
                    ct.branches + 1 AS branches FROM categories c
                INNER JOIN category_tree ct ON ct.id = c.parent
            ),
            product_counts AS (SELECT category_id, COUNT(*) AS product_count FROM product_categories GROUP BY category_id),
            category_with_products AS (
                SELECT ct.*, COALESCE(pc.product_count, 0) AS product_count FROM category_tree ct
                LEFT JOIN product_counts pc ON ct.id = pc.category_id
            )
            SELECT id, name, slug, parent, path, has_childs, branches, product_count FROM category_with_products ORDER BY path;
        "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(categories)
    }

    pub async fn get_by_parameters(&self,
        ids: &Vec<i32>,
        skus: &Vec<String>,
        page: i32,
        per_page: i32,
        order: types::Order) -> Result<Vec<ProductRow>, anyhow::Error> {

        let mut where_parts = Vec::new();
        where_parts.push("products.status = 'publish'".to_string());
        if !ids.is_empty() {
            let ids_string: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
            where_parts.push(format!("products.id IN ({})", ids_string.join(",")));
        }

        if !skus.is_empty() {
            where_parts.push(format!("products.sku IN ({})", skus.join(",")));
        }

        let offset = (page - 1) * per_page;

        let products = sqlx::query(&format!(r#"
            SELECT
                products.id, products.sku, products.name, products.slug, products.permalink, products.description,
                products.short_description, products.price, products.regular_price, products.sale_price,
                products.on_sale, products.stock_quantity, products.stock_status, products.weight,
                products.date_created, products.date_modified,
                (SELECT (JSON_AGG(ti)::jsonb) FROM (
                    SELECT media.id, media.src, media.name, media.alt, media.date_created, media.date_modified, product_media.position
                    FROM media, product_media
                    WHERE product_media.product_id = products.id AND product_media.media_id = media.id
                    ORDER BY product_media.position
                ) ti
            ) AS gallery
            FROM products WHERE {}
            ORDER BY
                products.date_created {}
            LIMIT $1 OFFSET $2;
        "#, where_parts.join(" AND "), order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| ProductRow {
                id: row.get::<i32, _>("id"),
                sku: row.get::<String, _>("sku"),
                name: row.get::<String, _>("name"),
                slug: row.get::<String, _>("slug"),
                permalink: row.get::<String, _>("permalink"),
                description: row.get::<String, _>("description"),
                short_description: row.get::<String, _>("short_description"),
                price: match row.get::<Decimal, _>("price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                sale_price: match row.get::<Decimal, _>("sale_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                on_sale: row.get::<bool, _>("on_sale"),
                stock_quantity: row.get::<i32, _>("stock_quantity"),
                stock_status: row.get::<StockStatus, _>("stock_status"),
                weight: row.get::<i32, _>("weight"),
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(products)
    }

    pub async fn get_one_by_slug(&self, slug: &str) -> Result<Product, anyhow::Error> {

        let product = sqlx::query(r#"
            SELECT
                products.id, products.sku, products.name, products.slug, products.permalink,
                products.description, products.short_description,products.price, products.regular_price,
                products.sale_price, products.on_sale, products.stock_quantity, products.stock_status,
                products.weight, products.date_created, products.date_modified,
                (SELECT (JSON_AGG(ti)::jsonb) FROM (
                    SELECT media.id, media.src, media.name, media.alt, media.date_created, media.date_modified, product_media.position
                    FROM media, product_media WHERE product_media.product_id = products.id AND product_media.media_id = media.id
                    ORDER BY product_media.position) ti) AS gallery
            FROM products
            WHERE products.slug = $1 AND products.status = 'publish';
        "#)
            .bind(slug)
            .map(|row: PgRow| Product {
                id: row.get::<i32, _>("id"),
                sku: row.get::<String, _>("sku"),
                name: row.get::<String, _>("name"),
                slug: row.get::<String, _>("slug"),
                permalink: row.get::<String, _>("permalink"),
                description: row.get::<String, _>("description"),
                short_description: row.get::<String, _>("short_description"),
                price: match row.get::<Decimal, _>("price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                sale_price: match row.get::<Decimal, _>("sale_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                on_sale: row.get::<bool, _>("on_sale"),
                stock_quantity: row.get::<i32, _>("stock_quantity"),
                stock_status: row.get::<types::StockStatus, _>("stock_status"),
                weight: row.get::<i32, _>("weight"),
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_one(&self.pool)
            .await?;

        Ok(product)
    }

    pub async fn count_all(&mut self, status: Option<Status>) -> Result<i32, anyhow::Error> {
        let from = match status {
            Some(status) => format!("products WHERE status = '{}'", status.as_str()),
            None => "products".to_string(),
        };

        let total_count: (i64, ) = sqlx::query_as(&format!("SELECT COUNT(*) FROM {};", from))
            .fetch_one(&self.pool)
            .await?;

        Ok(total_count.0 as i32)
    }

    pub async fn get_all(&self,
        status: Option<Status>,
        page: i32,
        per_page: i32,
        order: types::Order) -> Result<Vec<ProductRow>, anyhow::Error> {

        let offset = (page - 1) * per_page;

        let from = match status {
            Some(status) => format!("products WHERE products.status = '{}'", status.as_str()),
            None => "products".to_string(),
        };

        let products = sqlx::query(&format!(r#"
            SELECT
                products.id, products.sku, products.name, products.slug, products.permalink, products.description,
                products.short_description, products.price, products.regular_price, products.sale_price,
                products.on_sale, products.stock_quantity, products.stock_status, products.weight,
                products.date_created, products.date_modified,
                (SELECT (JSON_AGG(ti)::jsonb) FROM (
                    SELECT media.id, media.src, media.name, media.alt, media.date_created, media.date_modified, product_media.position
                    FROM media, product_media
                    WHERE product_media.product_id = products.id AND product_media.media_id = media.id
                    ORDER BY product_media.position
                ) ti
            ) AS gallery
            FROM {}
            ORDER BY
                products.date_created {}
            LIMIT $1 OFFSET $2;
        "#, from, order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| ProductRow {
                id: row.get::<i32, _>("id"),
                sku: row.get::<String, _>("sku"),
                name: row.get::<String, _>("name"),
                slug: row.get::<String, _>("slug"),
                permalink: row.get::<String, _>("permalink"),
                description: row.get::<String, _>("description"),
                short_description: row.get::<String, _>("short_description"),
                price: match row.get::<Decimal, _>("price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                sale_price: match row.get::<Decimal, _>("sale_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                on_sale: row.get::<bool, _>("on_sale"),
                stock_quantity: row.get::<i32, _>("stock_quantity"),
                stock_status: row.get::<StockStatus, _>("stock_status"),
                weight: row.get::<i32, _>("weight"),
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(products)
    }

    async fn delete(&self, product_id: i32) -> Result<(), anyhow::Error> {
        // Implementation to delete a product

        Ok(())
    }

    pub async fn update(&self, product_id: i32) -> Result<(), anyhow::Error> {
        // Implementation to update a product

        Ok(())
    }

    async fn add(&self, product: &Product) -> Result<i32, anyhow::Error> {
        // Implementation to add a new product

        Ok(0)
    }

    pub async fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Products {
            pool,
        }
    }
}
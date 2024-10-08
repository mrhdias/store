//
// Last Modifications: 2024-08-14 19:18:50
//

use crate::types;
use crate::models::frontend;
use crate::models::backend;

use anyhow;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use url::Url;
use chrono::NaiveDateTime;

use std::{
    fs,
    path::PathBuf,
};

use strum::EnumIter;

use sqlx::{
    postgres::PgRow,
    types::{Json, Decimal},
    Row,
};

use serde::{
    Serialize,
    Deserialize
};
use serde_json::Value as JsonValue;

use super::frontend::ProductPage;

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
    pub fn as_str(&self) -> &str {
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
pub enum StockStatus {
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

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "catalog_visibility", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CatalogVisibility {
    Visible, // Default
    Catalog,
    Search,
    Hidden,
}

#[derive(Debug, Deserialize)]
pub struct Parameters {
    pub status: Option<Status>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<types::Order>,
    pub order_by: Option<String>,
    pub featured: Option<bool>,
    pub category: Option<u32>, // Limit result set to products assigned a specific category ID.
    pub sku: Option<String>, // Limit result set to products with a specific SKU.
    pub exclude: Option<String>, // array - Ensure result set excludes specific IDs.
    pub include: Option<String>, // array - Limit result set to specific ids.
    pub on_sale: Option<bool>,
    pub min_price: Option<f32>,
    pub max_price: Option<f32>,
    pub stock_status: Option<StockStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
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
    count: i64, // number of products
}

pub fn products_order_by(parameter: &Option<String>) -> &str {
    match parameter.as_ref() {
        Some(v) => match v.as_str() {
            "date" => "date_created",
            "modified" => "date_modified",
            "id" => "id",
            // "included" => "?",
            "title" => "name",
            // "slug" => "?",
            "price" => "price",
            // "popularity" => "?",
            // "rating" => "?",
            _ => "date_created", // The default case
        },
        None => "date_created",
    }
}

fn where_clause_parts(parameters: &Parameters) -> Vec<String> {
    let mut where_parts = Vec::new();

    if parameters.status.is_some() {
        where_parts.push(format!("products.status = '{}'",
            parameters.status.as_ref().unwrap().as_str().to_string()));
    }
    if parameters.on_sale.is_some() {
        where_parts.push(format!("products.on_sale = {}",
            parameters.on_sale.unwrap()));
    }
    if parameters.featured.is_some() {
        where_parts.push(format!("products.featured = {}",
            parameters.featured.unwrap()));
    }
    if parameters.min_price.is_some() {
        where_parts.push(format!("products.price >= {}",
            parameters.min_price.unwrap()));
    }
    if parameters.max_price.is_some() {
        where_parts.push(format!("products.price <= {}",
            parameters.max_price.unwrap()));
    }
    if parameters.stock_status.is_some() {
        where_parts.push(format!("products.stock_status = '{}'",
            parameters.stock_status.as_ref().unwrap().as_str().to_string()));
    }
    if parameters.sku.is_some() {
        where_parts.push(format!("products.sku IN ({})",
            parameters.sku.as_ref().unwrap().as_str().to_string()));
    }
    if parameters.include.is_some() {
        where_parts.push(format!("products.id IN ({})",
            parameters.include.as_ref().unwrap().as_str().to_string()));
    }
    if parameters.exclude.is_some() {
        where_parts.push(format!("products.id NOT IN ({})",
            parameters.exclude.as_ref().unwrap().as_str().to_string()));
    }

    where_parts
}

pub struct Products {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Products {

    pub fn backend(&self) -> Backend {
        Backend::new(&self.pool)
    }

    pub fn frontend(&self) -> Frontend {
        Frontend::new(&self.pool)
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Products {
            pool,
        }
    }
}

//
// Frontend implementation
//

pub struct Frontend<'a> {
    pool: &'a sqlx::Pool<sqlx::Postgres>,
}

impl<'a> Frontend<'a> {

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
            .fetch_one(self.pool)
            .await?;

        Ok(total_count.0 as i32)
    }

    pub async fn get_category_by_slug(&self,
        slug: &str,
        page: i32,
        per_page: i32,
        order_by: String,
        order: types::Order) -> Result<Vec<frontend::ProductShort>, anyhow::Error> {

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
                products.{} {}
            LIMIT $2 OFFSET $3;
        "#, order_by.as_str(), order.as_str()))
            .bind(slug)
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| frontend::ProductShort {
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
                weight: row.get::<i32, _>("weight") as u32,
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_all(self.pool)
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
            product_count AS (SELECT category_id, COUNT(*) AS count FROM product_categories GROUP BY category_id),
            category_with_products AS (
                SELECT ct.*, COALESCE(pc.count, 0) AS count FROM category_tree ct
                LEFT JOIN product_count pc ON ct.id = pc.category_id
            )
            SELECT id, name, slug, parent, path, has_childs, branches, count FROM category_with_products ORDER BY path;
        "#)
            .fetch_all(self.pool)
            .await?;

        Ok(categories)
    }

    pub async fn get_by_parameters(&self,
        ids: &Vec<i32>,
        skus: &Vec<String>,
        page: i32,
        per_page: i32,
        order: types::Order) -> Result<Vec<frontend::ProductShort>, anyhow::Error> {

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
            .map(|row: PgRow| frontend::ProductShort {
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
                weight: row.get::<i32, _>("weight") as u32,
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_all(self.pool)
            .await?;

        Ok(products)
    }

    pub async fn get_one_by_slug(&self, slug: &str) -> Result<frontend::Product, anyhow::Error> {

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
            .map(|row: PgRow| frontend::Product {
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
                weight: row.get::<i32, _>("weight") as u32,
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_one(self.pool)
            .await?;

        Ok(product)
    }

    pub async fn get_page(&self,
        parameters: &Parameters,
        category_slug: Option<&str>,
    ) -> Result<frontend::ProductPage, anyhow::Error> {

        let per_page = parameters.per_page.unwrap_or(3) as i32;

        let order = parameters.order.as_ref().unwrap_or(&types::Order::Desc);
        let order_by = products_order_by(&parameters.order_by);

        // println!("{:?}", parameters);

        let (min_price, max_price): (Decimal, Decimal) = sqlx::query_as(r#"
            SELECT
                COALESCE(MIN(price), 0.00) AS min_price,
                COALESCE(MAX(price), 0.00) AS max_price
            FROM products WHERE products.status = $1;
        "#)
            .bind(Status::Publish)
            .fetch_one(self.pool)
            .await?;


        let mut where_parts = where_clause_parts(&parameters);

        let mut tables = vec!["products"];

        if parameters.category.is_some() || category_slug.is_some() {

            // http://127.0.0.1:8080/product-category/nam-vitae-magna/
            if parameters.category.is_some() {
                where_parts.push(format!("product_categories.category_id={} AND product_categories.product_id=products.id",
                    parameters.category.unwrap()));
            }

            // http://127.0.0.1:8080/product-category/nam-vitae-magna/?category=1
            if category_slug.is_some() {
                let slug = category_slug.unwrap();
                where_parts.push(format!("categories.slug='{}' AND categories.id=product_categories.category_id AND product_categories.product_id=products.id", slug));
                tables.push("categories");
            };

            tables.push("product_categories");
        }

        let from = if where_parts.is_empty() {
            "products".to_string()
        } else {
            format!("{} WHERE {}", tables.join(", "), where_parts.join(" AND "))
        };

        let total: (i64, ) = sqlx::query_as(&format!(r#"
            SELECT COUNT(*) FROM {};
        "#, from))
            .fetch_one(self.pool)
            .await?;

        if total.0 == 0 {
            return Ok(ProductPage {
                products: Vec::new(),
                total_count: 0,
                current_page: 0,
                per_page: 0,
                total_pages: 0,
                min_price: min_price.to_f32().unwrap(),
                max_price: max_price.to_f32().unwrap(),
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
                products.{} {}
            LIMIT $1 OFFSET $2;
        "#, from, order_by, order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| frontend::ProductShort {
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
                weight: row.get::<i32, _>("weight") as u32,
                gallery: row.get::<Json<Vec<Media>>, _>("gallery"),
            })
            .fetch_all(self.pool)
            .await?;

        Ok(ProductPage{
            products,
            total_count: total.0 as i32,
            current_page: page,
            per_page,
            total_pages,
            min_price: min_price.to_f32().unwrap(),
            max_price: max_price.to_f32().unwrap(),
        })
    }

    pub fn new(pool: &'a sqlx::Pool<sqlx::Postgres>) -> Self {
        Frontend {
            pool,
        }
    }
}

//
// Backend implementation
//

#[derive(Debug, PartialEq)]
pub enum ImageOperation {
    Delete,
    Update,
    Insert,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductImage {
    pub id: i32,
    pub src: String,
    pub name: String,
    pub alt: String,
    pub position: i32,
}

impl ProductImage {
    pub fn clone(&self) -> ProductImage {
        ProductImage {
            id: self.id,
            src: self.src.to_string(),
            name: self.name.to_string(),
            alt: self.alt.to_string(),
            position: self.position,
        }
    }

    pub fn default(& mut self) {
        self.id = 0;
        self.src = "".to_string();
        self.name = "".to_string();
        self.alt = "".to_string();
        self.position = 0;
    }

    pub fn new() -> Self {
        ProductImage {
            id: 0,
            src: "".to_string(),
            name: "".to_string(),
            alt: "".to_string(),
            position: 0
        }
    }
}

pub struct Backend<'a> {
    pool: &'a sqlx::Pool<sqlx::Postgres>,
}

impl<'a> Backend<'a> {

    pub async fn add_to_media(&self,
        filepath: &str,
        name: &str,
        alt: &str) -> Result<i32, anyhow::Error> {
        // Implementation to add a item to the media list

        let media_row: (i32, ) = sqlx::query_as(r#"
            INSERT INTO media (src, name, alt)
            VALUES ($1, $2, $3) RETURNING id;
        "#)
            .bind(format!("{}/{}", "http://127.0.0.1:8080", filepath))
            .bind(name)
            .bind(alt)
            .fetch_one(self.pool)
            .await?;

        Ok(media_row.0)
    }

    pub async fn delete_categories(&self,
        product_id: i32,
    ) -> Result<(), anyhow::Error> {

        sqlx::query(r#"
            DELETE FROM product_categories WHERE product_id = $1;
        "#)
            .bind(&product_id)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_category(&self,
        category_id: &i32,
        product_id: &i32) -> Result<(), anyhow::Error> {
        // Implementation to add a category to a product

        println!("adding category {} to product {}", category_id, product_id);

        sqlx::query(r#"
            INSERT INTO product_categories (product_id, category_id)
            VALUES ($1, $2);
        "#)
           .bind(product_id)
           .bind(category_id)
           .execute(self.pool)
           .await?;

        Ok(())
    }

    pub async fn update(&self,
        product: &backend::Product,
        images: &HashMap<i32, ImageOperation>,
        delete_media: bool) -> Result<(), anyhow::Error> {
        // Implementation to update a product

        sqlx::query(r#"
            UPDATE products
            SET name = $1, slug = $2, description = $3, short_description = $4, sku = $5,
                price = $6, regular_price = $7, sale_price = $8, on_sale = $9,
                stock_quantity = $10, stock_status= $11, weight = $12, permalink = $13, status = $14, primary_category = $15
            WHERE id = $16;
        "#)
            .bind(&product.name)
            .bind(&product.slug)
            .bind(&product.description)
            .bind(&product.short_description)
            .bind(&product.sku)
            .bind(&product.price)
            .bind(&product.regular_price)
            .bind(&product.sale_price)
            .bind(&product.on_sale)
            .bind(product.stock_quantity)
            .bind(&product.stock_status)
            .bind(product.weight as i32)
            .bind(&product.permalink)
            .bind(&product.status)
            .bind(&product.primary_category)
            .bind(&product.id)
            .execute(self.pool)
            .await?;


        for image in &product.images {
            println!("IMAGE {:?}", image);
            if !images.contains_key(&image.id) {
                continue;
            }

            let operation = match images.get(&image.id) {
                Some(operation) => operation,
                None => continue,
            };

            println!("IMAGE OPERATION {:?}", operation);

            if *operation == ImageOperation::Insert {
                // insert image in product_media
                sqlx::query(r#"
                    INSERT INTO product_media (product_id, media_id, position)
                    VALUES ($1, $2, $3);
                "#)
                    .bind(&product.id)
                    .bind(&image.id)
                    .bind(&image.position)
                    .execute(self.pool)
                    .await?;

                // images.remove(&image.id);

                continue;
            }

            if *operation == ImageOperation::Update {
                // update image from product_media
                sqlx::query(r#"
                    UPDATE product_media
                    SET position = $1
                    WHERE product_id = $2 AND media_id = $3;
                "#)
                    .bind(&image.position)
                    .bind(&product.id)
                    .bind(&image.id)
                    .execute(self.pool)
                    .await?;

                // update media name and alt in table media
                sqlx::query(r#"
                    UPDATE media
                    SET name = $1, alt = $2
                    WHERE id = $3;
                "#)
                    .bind(&image.name)
                    .bind(&image.alt)
                    .bind(&image.id)
                    .execute(self.pool)
                    .await?;
            }
        }
        
        for (image_id, operation) in images {
            if *operation != ImageOperation::Delete {
                continue;
            }
            println!("Deleting image {}", image_id);

            sqlx::query(r#"
                DELETE FROM product_media
                WHERE media_id = $1;
            "#)
                .bind(&image_id)
                .execute(self.pool)
                .await?;

            if delete_media {
                let image_src: String = sqlx::query(r#"
                    DELETE FROM media
                    WHERE id = $1
                    RETURNING src;
                "#)
                    .bind(&image_id)
                    .fetch_one(self.pool)
                    .await?
                    .get(0);


                // http://127.0.0.1:8080/uploads/2024-06-18/file.png
                // let image_src = image_row.get::<String, _>("src");
                let parsed_url = match Url::parse(&image_src) {
                    Ok(url) => url,
                    Err(_) => {
                        return Err(anyhow::anyhow!("Invalid URL: {}", image_src));
                    }
                };

                let path = parsed_url.path();
                let file_path = PathBuf::from(format!("static/{}", path));
            
                if file_path.exists() {
                    fs::remove_file(file_path).expect("Failed to remove file");
                }
            }
        }

        Ok(())
    }

    pub async fn get(&self, product_id: i32) -> Result<backend::Product, anyhow::Error> {
        // Implementation to get a product by ID

        let row = sqlx::query(r#"
            SELECT
                products.id, products.sku, products.name, products.slug,
                products.description, products.short_description,
                products.price, products.regular_price, products.sale_price, products.on_sale,
                products.stock_quantity, products.stock_status, products.weight, products.permalink,
                products.date_created, products.status, products.primary_category,
                COALESCE( (SELECT (JSON_AGG(ti)::jsonb)
                FROM (
                    SELECT media.id, media.src, media.name, media.alt, product_media.position
                    FROM media, product_media
                    WHERE media.id = product_media.media_id AND product_media.product_id = products.id
                    ORDER BY product_media.position
                ) ti), '[]') AS images,
                COALESCE( (SELECT to_jsonb(ARRAY_AGG(category_id))
                FROM product_categories WHERE product_categories.product_id = products.id), '[]') AS categories
            FROM products WHERE products.id = $1;
        "#)
            .bind(&product_id)
            .fetch_one(self.pool)
            .await?;

        let images_json: JsonValue = row.get("images");
        let categories_json: JsonValue = row.get("categories");

        Ok(backend::Product {
            id: row.get::<i32, _>("id"),
            sku: row.get::<String, _>("sku"),
            name: row.get::<String, _>("name"),
            slug: row.get::<String, _>("slug"),
            description: row.get::<String, _>("description"),
            short_description: row.get::<String, _>("short_description"),
            regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
                Some(f) => f,
                None => 0.00,
            },
            price: match row.get::<Decimal, _>("price").to_f32() {
                Some(f) => f,
                None => 0.00,
            },
            sale_price: match row.get::<Decimal, _>("sale_price").to_f32() {
                Some(f) => f,
                None => 0.00,
            },
            on_sale: row.get::<bool, _>("on_sale"),
            stock_status: row.get::<StockStatus, _>("stock_status"),
            stock_quantity: row.get::<i32, _>("stock_quantity"),
            weight: row.get::<i32, _>("weight") as u32,
            status: row.get::<Status, _>("status"),
            permalink: row.get::<String, _>("permalink"),
            primary_category: row.get::<i32, _>("primary_category"),
            images: serde_json::from_value(images_json).unwrap(),
            categories: serde_json::from_value(categories_json).unwrap(),
        })
    }

    pub async fn add(&self,
        product: &backend::Product,
    ) -> Result<i32, anyhow::Error> {
        // Implementation to add a new product

        let product_id: i32 = sqlx::query(r#"
            INSERT INTO products (
                name, slug, description, sku,
                price, regular_price, sale_price, on_sale,
                stock_quantity, stock_status, weight, permalink, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id;
        "#)
           .bind(&product.name)
           .bind(&product.slug)
           .bind(&product.description)
           .bind(&product.sku)
           .bind(&product.price)
           .bind(&product.regular_price)
           .bind(&product.sale_price)
           .bind(&product.on_sale)
           .bind(&product.stock_quantity)
           .bind(&product.stock_status)
           .bind(product.weight as i32)
           .bind(&product.permalink)
           .bind(&product.status)
           .fetch_one(self.pool)
           .await?
           .get(0);

        for image in &product.images {
            sqlx::query(r#"
                INSERT INTO product_media (product_id, media_id, position)
                VALUES ($1, $2, $3);
            "#)
               .bind(&product_id)
               .bind(&image.id)
               .bind(&image.position)
               .execute(self.pool)
               .await?;
        }

        Ok(product_id)
    }

    pub async fn count_all(&self) -> Result<i32, anyhow::Error> {
        let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
            .fetch_one(self.pool)
            .await?;

        Ok(total_count.0 as i32)
    }

    pub async fn get_page(&self,
        parameters: &Parameters,
    ) -> Result<backend::ProductPage, anyhow::Error> {

        let per_page = parameters.per_page.unwrap_or(3) as i32;

        let order = parameters.order.as_ref().unwrap_or(&types::Order::Asc);
        let order_by = products_order_by(&parameters.order_by);

        let where_parts = where_clause_parts(&parameters);

        let from = if where_parts.is_empty() {
            "products".to_string()
        } else {
            format!("products WHERE {}", where_parts.join(" AND "))
        };

        let total: (i64, ) = sqlx::query_as(&format!(r#"
            SELECT 
                COUNT(*)
            FROM {};
        "#, from))
            .fetch_one(self.pool)
            .await?;

        if total.0 == 0 {
            return Ok(backend::ProductPage::new());
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

        let products = sqlx::query(&format!(r#"
            SELECT
                products.id, products.sku, products.name, products.price,
                products.regular_price, products.sale_price, products.on_sale,
                products.stock_quantity, products.stock_status, products.date_created,
                products.status, products.primary_category,
                COALESCE(image.src, '/assets/images/product.jpg') AS image_src, 
                COALESCE(image.name, 'Unnamed product') AS image_name, 
                COALESCE(image.alt, 'Unnamed product') AS image_alt
            FROM products
            LEFT JOIN LATERAL (
                SELECT
                    media.src, media.name, media.alt
                FROM product_media
                JOIN media
                ON product_media.media_id = media.id WHERE product_media.product_id = products.id
                ORDER BY product_media.position LIMIT 1
            ) AS image ON true 
            ORDER BY 
                products.{} {}
            LIMIT $1 OFFSET $2;
        "#, order_by, order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| backend::ProductShort {
                id: row.get::<i32, _>("id"),
                sku: row.get::<String, _>("sku"),
                name: row.get::<String, _>("name"),
                regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                sale_price: match row.get::<Decimal, _>("sale_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                on_sale: row.get::<bool, _>("on_sale"),
                stock_status: row.get::<StockStatus, _>("stock_status"),
                stock_quantity: row.get::<i32, _>("stock_quantity"),
                image_src: row.get::<String, _>("image_src"),
                image_alt: row.get::<String, _>("image_alt"),
                date_created: || -> String {
                    let date_created = row.get::<NaiveDateTime, _>("date_created");
                    date_created.format("%Y/%m/%d at %H:%M:%S").to_string()
                }(),
                status: row.get::<Status, _>("status"),
            })
            .fetch_all(self.pool)
            .await?;

        Ok(backend::ProductPage {
            products,
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
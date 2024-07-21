//
// Last Modification: 2024-07-01 21:37:21
//

pub mod media;

use crate::admin::categories;
use crate::types;
use crate::utils;

use anyhow;
use slug::slugify;
use std::collections::HashMap;
use url::Url;

use std::{
    fs,
    fs::File,
    io::Write,
    path::PathBuf,
};

use strum::{
    EnumIter,
    IntoEnumIterator
};

use chrono::{
    Local,
    NaiveDateTime
};
use num_traits::ToPrimitive;

use axum::{
    extract::{Extension, Query, Path, Multipart},
    response::Html,
};

use sqlx::{
    postgres::PgRow,
    types::Decimal,
    Row,
};

use tera::{
    Tera,
    Context
};

use serde::{
    Serialize,
    Deserialize
};
use serde_json::Value as JsonValue;

#[derive(Debug, PartialEq)]
enum ImageOperation {
    Delete,
    Update,
    Insert,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, EnumIter)]
#[sqlx(type_name = "status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Status {
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
struct ProductImage {
    id: i32,
    src: String,
    name: String,
    alt: String,
    position: i32,
}

impl ProductImage {
    fn clone(&self) -> ProductImage {
        ProductImage {
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
        ProductImage {
            id: 0,
            src: "".to_string(),
            name: "".to_string(),
            alt: "".to_string(),
            position: 0
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
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
    stock_status: StockStatus,
    stock_quantity: i32,
    status: Status,
    primary_category: i32,
    categories: Vec<i32>,
    images: Vec<ProductImage>
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductRow {
    id: i32,
    sku: String,
    name: String,
    regular_price: f32,
    stock_status: StockStatus,
    stock_quantity: i32,
    image_src: String,
    image_alt: String,
    date_created: String,
    status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductForm {
    name: String,
    slug: std::string::String,
    description: String,
    sku: String,
    regular_price: f32,
    stock_quantity: i32,
    status: Status,
}

pub struct Products {
    pool: sqlx::Pool<sqlx::Postgres>,
    total_count: i32,
}

impl Products {

    async fn delete_categories(&self, product_id: i32) -> Result<(), anyhow::Error> {
        sqlx::query(r#"
            DELETE FROM product_categories WHERE product_id = $1;
        "#)
            .bind(&product_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn add_category(&self, category_id: &i32, product_id: &i32) -> Result<(), anyhow::Error> {
        // Implementation to add a category to a product

        println!("adding category {} to product {}", category_id, product_id);

        sqlx::query(r#"
            INSERT INTO product_categories (product_id, category_id)
            VALUES ($1, $2);
        "#)
           .bind(product_id)
           .bind(category_id)
           .execute(&self.pool)
           .await?;

        Ok(())
    }

    async fn add(&self, product: &Product) -> Result<i32, anyhow::Error> {
        // Implementation to add a new product

        let product_id: i32 = sqlx::query(r#"
            INSERT INTO products (
                name, slug, description, sku, price, regular_price, stock_quantity, stock_status, permalink, status)
            VALUES ($1, $2, $3, $4, $5, $5, $6, $7, $8, $9) RETURNING id;
        "#)
           .bind(&product.name)
           .bind(&product.slug)
           .bind(&product.description)
           .bind(&product.sku)
           .bind(&product.regular_price)
           .bind(&product.stock_quantity)
           .bind(&product.stock_status)
           .bind(&product.permalink)
           .bind(&product.status)
           .fetch_one(&self.pool)
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
               .execute(&self.pool)
               .await?;
        }

        Ok(product_id)
    }

    async fn get(&self, product_id: i32) -> Result<Product, anyhow::Error> {
        // Implementation to get a product by ID

        let row = sqlx::query(r#"
            SELECT
                products.id, products.sku, products.name, products.slug,
                products.description, products.short_description,
                products.price, products.regular_price, products.sale_price, products.on_sale,
                products.stock_quantity, products.stock_status, products.permalink,
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
            .fetch_one(&self.pool)
            .await?;

        let images_json: JsonValue = row.get("images");
        let categories_json: JsonValue = row.get("categories");

        Ok(Product {
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
            stock_status: row.get::<StockStatus, _>("stock_status"),
            stock_quantity: row.get::<i32, _>("stock_quantity"),
            status: row.get::<Status, _>("status"),
            permalink: row.get::<String, _>("permalink"),
            primary_category: row.get::<i32, _>("primary_category"),
            images: serde_json::from_value(images_json).unwrap(),
            categories: serde_json::from_value(categories_json).unwrap(),
        })
    }

    async fn get_all(&self,
        page: i32,
        per_page: i32,
        order: types::Order) -> Result<Vec<ProductRow>, anyhow::Error> {
        // Implementation to get products

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
                products.date_created {}
            LIMIT $1 OFFSET $2;
        "#, order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| ProductRow {
                id: row.get::<i32, _>("id"),
                sku: row.get::<String, _>("sku"),
                name: row.get::<String, _>("name"),
                regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
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
            .fetch_all(&self.pool)
            .await?;

        Ok(products)
    }

    async fn update(&self,
        product: &Product,
        images: &HashMap<i32, ImageOperation>,
        delete_media: bool) -> Result<(), anyhow::Error> {
        // Implementation to update a product
    
        sqlx::query(r#"
            UPDATE products
            SET name = $1, slug = $2, description = $3, short_description = $4,
                sku = $5, price = $6, regular_price = $6,
                stock_quantity = $7, stock_status= $8, permalink = $9, status = $10, primary_category = $11
            WHERE id = $12;
        "#)
            .bind(&product.name)
            .bind(&product.slug)
            .bind(&product.description)
            .bind(&product.short_description)
            .bind(&product.sku)
            .bind(&product.regular_price)
            .bind(&product.stock_quantity)
            .bind(&product.stock_status)
            .bind(&product.permalink)
            .bind(&product.status)
            .bind(&product.primary_category)
            .bind(&product.id)
            .execute(&self.pool)
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
                    .execute(&self.pool)
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
                    .execute(&self.pool)
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
                    .execute(&self.pool)
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
                .execute(&self.pool)
                .await?;

            if delete_media {
                let image_src: String = sqlx::query(r#"
                    DELETE FROM media
                    WHERE id = $1
                    RETURNING src;
                "#)
                    .bind(&image_id)
                    .fetch_one(&self.pool)
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

    async fn delete(&self, product_id: i32) {
        // Implementation to delete a product
    }

    async fn add_to_media(&self, filepath: &str, name: &str, alt: &str) -> Result<i32, anyhow::Error> {
        // Implementation to add a item to the media list

        let media_row: (i32, ) = sqlx::query_as(r#"
            INSERT INTO media (src, name, alt)
            VALUES ($1, $2, $3) RETURNING id;
        "#)
            .bind(format!("{}/{}", "http://127.0.0.1:8080", filepath))
            .bind(name)
            .bind(alt)
            .fetch_one(&self.pool)
            .await?;

        Ok(media_row.0)
    }

    async fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {

        let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
            .fetch_one(&pool)
            .await
            .expect("Failed to count products");

        Products {
            pool,
            total_count: total_count.0 as i32,
        }
    }
}


fn generate_unique_filename(dir: &std::path::Path, filename: &str) -> PathBuf {
    let mut path = dir.join(filename);
    let mut counter = 1;

    // Extract the base name and extension
    let (base, ext) = match path.extension() {
        Some(ext) => {
            let ext_str = ext.to_str().unwrap_or("");
            let base = path.file_stem().unwrap_or_else(|| path.as_os_str()).to_str().unwrap_or("");
            (base.to_string(), ext_str.to_string())
        }
        None => (filename.to_string(), "".to_string()),
    };

    // Loop to find a unique filename
    while path.exists() {
        let new_filename = format!("{}-{}.{}", base, counter, ext);
        path = dir.join(&new_filename);
        counter += 1;
    }

    path
}

fn upload_image(data: &axum::body::Bytes, filename: &str) -> Result<PathBuf, anyhow::Error> {
    // Implementation to upload images

    let today = Local::now();
    // let today_date = today.format("%Y-%m-%d").to_string();
    let today_year = today.format("%Y").to_string();
    let today_month = today.format("%m").to_string();

    let mut upload_dir = PathBuf::new();
    upload_dir.push("static");
    upload_dir.push("uploads");
    upload_dir.push(&today_year);
    upload_dir.push(&today_month);

    if !fs::metadata(&upload_dir).is_ok() {
        fs::create_dir_all(&upload_dir)?;
        println!("Directory created: {}", upload_dir.display());
    }

    // test if the file exists and add a suffix
    let filepath = generate_unique_filename(&upload_dir, &filename);

    println!("file path: {}", filepath.display());

    // let file_path = upload_dir.join(&filename);


    let file = File::create(&filepath);
    match file {
        Ok(mut f) => {
            f.write_all(data).unwrap();
        },
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to create file: error: {}", e));
        }
    };

    match filepath.strip_prefix("static") {
        Ok(path) => {
            return Ok(path.to_path_buf());
        },
        Err(_) => {
            return Err(anyhow::anyhow!("Failed to strip prefix from file path"));
        }
    }
}

pub async fn handle(
    Path(id):Path<i32>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
    mut multipart: Multipart) -> Html<String> {

    let mut category_name = "".to_string();
    let mut parent_category = 0;
    let mut categories_ids: Vec<i32> = vec![];
    let mut primary_category = 0;

    let mut product = Product {
        id,
        sku: "".to_string(),
        name: "".to_string(),
        slug: "".to_string(),
        description: "".to_string(),
        short_description: "".to_string(),
        regular_price: 0.00,
        price: 0.00,
        sale_price: 0.00,
        stock_status: StockStatus::OutOfStock,
        stock_quantity: 0,
        permalink: "".to_string(),
        status: Status::Draft,
        primary_category: 0,
        images: vec![],
        categories: vec![],
    };

    let mut delete_media = false;
    let mut images: HashMap<i32, ImageOperation> = HashMap::new();
    let mut image = ProductImage::new();
    let mut count = 0;

    let products_manager = Products::new(pool.clone()).await;

    while let Some(field) = multipart.next_field().await.unwrap() {

        // println!("Field: {:?}", field);
        // id, src, name, position
        if count == 4 {
            if !images.contains_key(&image.id) { // remove
                images.insert(image.id, ImageOperation::Update);
                product.images.push(image.clone());
            }
            image.default();

            count = 0;
        }

        let field_name = match field.name() {
            Some(n) => n.to_string(),
            None => "".to_string()
        };

        match field_name.as_str() {
            "name" => {
                product.name = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing product Name: {}", e);
                        return Html("An error occurred while parsing product Name".to_string());
                    }
                }
            },
            "slug" => {
                product.slug = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing product Slug: {}", e);
                        return Html("An error occurred while parsing product Slug".to_string());
                    }
                }
            },
            "description" => {
                product.description = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing product Description: {}", e);
                        return Html("An error occurred while parsing product Description".to_string());
                    }
                }
            },
            "short_description" => {
                product.short_description = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing product Short Description: {}", e);
                        return Html("An error occurred while parsing product Short Description".to_string());
                    }
                }
            },
            "sku" => {
                product.sku = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing product SKU: {}", e);
                        return Html("An error occurred while parsing product SKU".to_string());
                    }
                }
            },
            "regular_price" => {
                product.regular_price = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string to f32"),
                    Err(e) => {
                        eprintln!("Error parsing product Regular Price: {}", e);
                        return Html("An error occurred while parsing product Regular Price".to_string());
                    }
                }
            },
            "stock_quantity" => {
                product.stock_quantity = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string to i32"),
                    Err(e) => {
                        eprintln!("Error parsing product Stock Quantity: {}", e);
                        return Html("An error occurred while parsing product Stock Quantity".to_string());
                    }
                }
            },
            "status" => {
                product.status = match field.text().await {
                    Ok(value) => {
                        match value.as_str() {
                            "publish" => Status::Publish,
                            "draft" => Status::Draft,
                            "pending" => Status::Pending,
                            "private" => Status::Private,
                            _ => Status::Draft,
                        }
                    },
                    Err(e) => {
                        eprintln!("Error parsing product Status: {}", e);
                        return Html("An error occurred while parsing product Status".to_string());
                    }
                }
            },
            "category_id" => {
                // Category id
                let category_id: i32 = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string to i32"),
                    Err(e) => {
                        eprintln!("Error parsing product Category Id: {}", e);
                        return Html("An error occurred while parsing product Category Id".to_string());
                    }
                };
                product.categories.push(category_id);
            },
            "primary_category" => {
                // Primary category
                primary_category = match field.text().await {
                    Ok(value) => {
                        value.parse().unwrap_or_else(|e| {
                            println!("Failed to parse the string to i32: {}", e);
                            0 // Default value or handle the error in another way
                        })
                    },
                    Err(e) => {
                        eprintln!("Error parsing product Primary Category: {}", e);
                        return Html("An error occurred while parsing product Primary Category".to_string());
                    }
                };
            },
            "new_category" => {
                // Add new category
                category_name = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing category name: {}", e);
                        return Html("An error occurred while parsing category name".to_string());
                    }
                };
            },
            "parent_category" => {
                println!("Parent category");
                // Parent new category
                parent_category = match field.text().await {
                    Ok(value) => {
                        value.parse().unwrap_or_else(|e| {
                            println!("Failed to parse the string to i32: {}", e);
                            0 // Default value or handle the error in another way
                        })
                    },
                    Err(e) => {
                        eprintln!("Error parsing product Parent Category: {}", e);
                        return Html("An error occurred while parsing product Parent Category".to_string());
                    }
                };
            },
            "image_id" => {
                image.id = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string to i32"),
                    Err(e) => {
                        eprintln!("Error parsing product Image ID: {}", e);
                        return Html("An error occurred while parsing product Image ID".to_string());
                    }
                };
                count += 1;
            },
            "image_src" => {
                image.src = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing product Image Src: {}", e);
                        return Html("An error occurred while parsing product Image Src".to_string());
                    }
                };
                count += 1;
            },
            "image_name" => {
                image.name = match field.text().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error parsing product Image Name: {}", e);
                        return Html("An error occurred while parsing product Image Name".to_string());
                    }
                };
                image.alt = image.name.to_string();
                count += 1;
            },
            "image_position" => {
                image.position = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string to i32"),
                    Err(e) => {
                        eprintln!("Error parsing product Image Position: {}", e);
                        return Html("An error occurred while parsing product Image Position".to_string());
                    }
                };
                count += 1;
            },
            "image_remove" => {
                // if exists if map the id remove from the map
                images.insert(match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string to i32"),
                    Err(e) => {
                        eprintln!("Error parsing product Image Remove: {}", e);
                        return Html("An error occurred while parsing product Image Remove".to_string());
                    }
                }, ImageOperation::Delete);
            },
            "delete_images" => {
                delete_media = true;
            },
            "files" => {
                let filename = match field.file_name() {
                    Some(name) => {
                        if name.is_empty() {
                            println!("file name is empty");
                            continue;
                        }
                        name.to_string()
                    },
                    None => {
                        println!("file name not exists");
                        continue;
                    }
                };

                // let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
                let field_data = match field.bytes().await {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error reading product Image: {}", e);
                        return Html("An error occurred while reading product Image".to_string());
                    }
                };
                
                if field_data.is_empty() {
                    println!("the uploaded file {} is empty", filename);
                    continue;
                }

                match upload_image(&field_data, &filename) {
                    Ok(filepath) => {

                        let id = match products_manager.add_to_media(
                            &filepath.to_string_lossy(), 
                            "Unnamed", 
                            "Unnamed").await {
                            Ok(id) => id,
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                return Html("An error occurred while adding product Image to media.".to_string());
                            }
                        };

                        product.images.push(ProductImage {
                            id,
                            name: "Unnamed".to_string(),
                            src: format!("/{}", filepath.to_string_lossy().to_string()),
                            position: product.images.len() as i32,
                            alt: "Unnamed".to_string(),
                        });

                        images.insert(id, ImageOperation::Insert);
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return Html("An error occurred while uploading the product Image.".to_string());
                    }
                }

            },
            _ => println!("Product Unknown field {}", field_name),
        }

        // println!("name: {:?}", field_name);
    }

    let categories_manager = categories::Categories::new(pool);

    if !category_name.is_empty() {
        match categories_manager.add(&category_name, parent_category).await {
            Ok(()) => {
                println!("Category {} added successfully", category_name);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while adding new category.".to_string());
            }
        };
    }

    product.primary_category = 0;
    if !product.categories.is_empty() {
        match products_manager.delete_categories(product.id).await {
            Ok(()) => {
                println!("Product categories deleted successfully for product id {}", product.id);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while delete categories from product.".to_string());
            }
        };
        for id in &product.categories {
            match products_manager.add_category(id, &product.id).await {
                Ok(()) => {
                    println!("Category {} added to product successfully", id);
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Html("An error occurred while adding category to product.".to_string());
                }
            };
            if primary_category == *id {
                product.primary_category = primary_category;
            }
        }
    }

    if product.slug.is_empty() {
        product.slug = slugify(&product.name);
    }

    product.stock_status = if product.stock_quantity == 0 {
        StockStatus::OutOfStock
    } else {
        StockStatus::InStock
    };

    product.permalink = format!("/product/{}", product.slug);

    println!("Product: {:?}", product);
    println!("Images: {:?}", images);

    let mut status_names = vec![];
    for status in Status::iter() {
        status_names.push(status.as_str().to_string());
    }

    let categories = match categories_manager.get_all().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error occurred while fetching categories.".to_string());
        }
    };

    println!("Categories: {:?}", categories);

    if product.id == 0 { // new product to add
        match products_manager.add(&product).await {
            Ok(id) => {
                product.id = id;
    
                tera.register_filter("round_and_format", utils::round_and_format_filter);
                
                let mut data = Context::new();
                data.insert("partial", "product");
                data.insert("product", &product);
                data.insert("categories", &categories);
                data.insert("alert", "Product added");
                data.insert("status", &status_names);
                let rendered = tera.render("admin/admin.html", &data).unwrap();
                return Html(rendered);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while adding the product.".to_string());
            }
        };
    } else { // product to update
        match products_manager.update(&product, &images, delete_media).await {
            Ok(_) => {
                println!("Product updated successfully.");

                tera.register_filter("round_and_format", utils::round_and_format_filter);

                let mut data = Context::new();
                data.insert("partial", "product");
                data.insert("product", &product);
                data.insert("categories", &categories);
                data.insert("alert", "Product updated");
                data.insert("status", &status_names);
                let rendered = tera.render("admin/admin.html", &data).unwrap();
                return Html(rendered);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while updating the product.".to_string());
            }
        }
    }

}

pub async fn edit(
    Path(id):Path<i32>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let products_manager = Products::new(pool.clone()).await;

    match products_manager.get(id).await {
        Ok(product) => {

            let categories_manager = categories::Categories::new(pool);
            let categories = match categories_manager.get_all().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Html("An error occurred while fetching categories.".to_string());
                }
            };
        
            println!("Categories: {:?}", categories);

            let mut status_names = vec![];
            for status in Status::iter() {
                status_names.push(status.as_str().to_string());
            }

            tera.register_filter("round_and_format", utils::round_and_format_filter);
            
            let mut data = Context::new();
            data.insert("partial", "product");
            data.insert("product", &product);
            data.insert("categories", &categories);
            data.insert("status", &status_names);
            let rendered = tera.render("admin/admin.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html(String::from("An error occurred while fetching the product."))
        },
    }
}


pub async fn new(
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    // let categories = vec![];

    let product = Product {
        id: 0,
        sku: "".to_string(),
        name: "".to_string(),
        slug: "".to_string(),
        description: "".to_string(),
        short_description: "".to_string(),
        regular_price: 0.00,
        price: 0.00,
        sale_price: 0.00,
        stock_status: StockStatus::OutOfStock,
        stock_quantity: 0,
        permalink: "".to_string(),
        status: Status::Draft,
        primary_category: 0,
        images: vec![],
        categories: vec![],
    };

    let categories_manager = categories::Categories::new(pool);
    let categories = match categories_manager.get_all().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error occurred while fetching categories.".to_string());
        }
    };

    let mut status_names = vec![];
    for status in Status::iter() {
        status_names.push(status.as_str().to_string());
    }

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();
    data.insert("partial", "product");
    data.insert("product", &product);
    data.insert("categories", &categories);
    data.insert("status", &status_names);
    let rendered = tera.render("admin/admin.html", &data).unwrap();
    Html(rendered)
}

pub async fn list(
    Query(pagination): Query<types::Pagination>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let products_manager = Products::new(pool).await;

    let per_page = pagination.per_page.unwrap_or(3);

    let total_pages: i32 = (products_manager.total_count as f32 / per_page as f32).ceil() as i32;

    let mut page = pagination.page.unwrap_or(1) as i32;
    if page > total_pages {
        page = total_pages;
    } else if page == 0 {
        page = 1;
    }

    match products_manager.get_all(
        page, 
        per_page as i32,
        pagination.order.unwrap_or(types::Order::Desc)).await {
        Ok(products) => {

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let mut data = Context::new();
            data.insert("partial", "products");
            data.insert("products", &products);
            data.insert("current_page", &page);
            data.insert("total_products", &products_manager.total_count);
            data.insert("per_page", &per_page);
            data.insert("total_pages", &total_pages);
            let rendered = tera.render("admin/admin.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("An error occurred while fetching products.".to_string())
        },
    }
}
//
// Last Modification: 2024-07-27 19:20:48
//

use crate::types;
use crate::utils;
use crate::models::categories;
use crate::models::products;

use anyhow;
use slug::slugify;
use std::collections::HashMap;

use std::{
    fs,
    fs::File,
    io::Write,
    path::PathBuf,
};

use strum::IntoEnumIterator;

use chrono::Local;

use axum::{
    extract::{Extension, Query, Path, Multipart},
    response::Html,
};

use tera::{
    Tera,
    Context
};

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

    let mut sale_price = -1.00;
    let mut category_name = "".to_string();
    let mut parent_category = 0;
    let mut categories_ids: Vec<i32> = vec![];
    let mut primary_category = 0;

    let mut product = products::ProductBackend {
        id,
        sku: "".to_string(),
        name: "".to_string(),
        slug: "".to_string(),
        description: "".to_string(),
        short_description: "".to_string(),
        regular_price: 0.00,
        price: 0.00,
        sale_price: 0.00,
        on_sale: false,
        stock_status: products::StockStatus::OutOfStock,
        stock_quantity: 0,
        weight: 0,
        permalink: "".to_string(),
        status: products::Status::Draft,
        primary_category: 0,
        images: vec![],
        categories: vec![],
    };

    let mut delete_media = false;
    let mut images: HashMap<i32, products::ImageOperation> = HashMap::new();
    let mut image = products::ProductImage::new();
    let mut count = 0;

    let products_manager = products::Products::new(pool.clone()).await;

    while let Some(field) = multipart.next_field().await.unwrap() {

        // println!("Field: {:?}", field);
        // id, src, name, position
        if count == 4 {
            if !images.contains_key(&image.id) { // remove
                images.insert(image.id, products::ImageOperation::Update);
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
                        eprintln!("Error parsing the product field \"sku\": {}", e);
                        return Html("An error occurred while parsing the product field \"sku\"".to_string());
                    }
                }
            },
            "regular_price" => {
                product.regular_price = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string \"regular_price\" to f32"),
                    Err(e) => {
                        eprintln!("Error parsing the product field \"regular_price\": {}", e);
                        return Html("An error occurred while parsing the product field \"regular_price\"".to_string());
                    }
                }
            },
            "sale_price" => {
                sale_price = match field.text().await {
                    Ok(value) => {
                        if value.is_empty() {
                            -1.00
                        } else {
                            value.parse().expect("Failed to parse the string \"sale_price\" to f32")
                        }
                    },
                    Err(e) => {
                        eprintln!("Error parsing the product field \"sale_price\": {}", e);
                        return Html("An error occurred while parsing the product field \"sale_price\"".to_string());
                    }
                }
            },
            "stock_quantity" => {
                product.stock_quantity = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string \"stock_quantity\" to u32"),
                    Err(e) => {
                        eprintln!("Error parsing the product field \"stock_quantity\": {}", e);
                        return Html("An error occurred while parsing the product field \"stock_quantity\"".to_string());
                    }
                }
            },
            "weight" => {
                product.weight = match field.text().await {
                    Ok(value) => value.parse().expect("Failed to parse the string \"weight\" to u32"),
                    Err(e) => {
                        eprintln!("Error parsing the product field \"weight\": {}", e);
                        return Html("An error occurred while parsing the product field \"weight\"".to_string());
                    }
                }
            },
            "status" => {
                product.status = match field.text().await {
                    Ok(value) => {
                        match value.as_str() {
                            "publish" => products::Status::Publish,
                            "draft" => products::Status::Draft,
                            "pending" => products::Status::Pending,
                            "private" => products::Status::Private,
                            _ => products::Status::Draft,
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
                }, products::ImageOperation::Delete);
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

                        let id = match products_manager.backend().add_to_media(
                            &filepath.to_string_lossy(), 
                            "Unnamed", 
                            "Unnamed").await {
                            Ok(id) => id,
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                return Html("An error occurred while adding product Image to media.".to_string());
                            }
                        };

                        product.images.push(products::ProductImage {
                            id,
                            name: "Unnamed".to_string(),
                            src: format!("/{}", filepath.to_string_lossy().to_string()),
                            position: product.images.len() as i32,
                            alt: "Unnamed".to_string(),
                        });

                        images.insert(id, products::ImageOperation::Insert);
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
        let category_id = match categories_manager.add(&category_name, parent_category).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while adding new category to the database.".to_string());
            }
        };
        product.categories.push(category_id);
    }

    product.primary_category = 0;
    if !product.categories.is_empty() {
        match products_manager.backend().delete_categories(product.id).await {
            Ok(()) => {
                println!("Product categories deleted successfully for product id {}", product.id);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while delete categories from product.".to_string());
            }
        };
        for id in &product.categories {
            match products_manager.backend().add_category(id, &product.id).await {
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

    product.price = product.regular_price;
    if sale_price < 0.00 {
        product.sale_price = 0.00;
        product.on_sale = false;
    } else if sale_price < product.regular_price {
        product.sale_price = sale_price;
        product.on_sale = true;
        product.price = sale_price;
    }

    if product.slug.is_empty() {
        product.slug = slugify(&product.name);
    }

    product.stock_status = if product.stock_quantity == 0 {
        products::StockStatus::OutOfStock
    } else {
        products::StockStatus::InStock
    };

    product.permalink = format!("/product/{}", product.slug);

    println!("Product: {:?}", product);
    println!("Images: {:?}", images);

    let mut status_names = vec![];
    for status in products::Status::iter() {
        status_names.push(status.as_str().to_string());
    }

    let categories = match categories_manager.get_all().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error occurred while fetching categories.".to_string());
        }
    };

    println!("Product: {:?}", product);
    println!("Categories: {:?}", categories);

    if product.id == 0 { // new product to add
        match products_manager.backend().add(&product).await {
            Ok(id) => {
                product.id = id;
    
                tera.register_filter("round_and_format", utils::round_and_format_filter);
                
                let mut data = Context::new();
                data.insert("partial", "product");
                data.insert("product", &product);
                data.insert("categories", &categories);
                data.insert("alert", "Product added");
                data.insert("status", &status_names);
                let rendered = tera.render("backend/admin.html", &data).unwrap();
                return Html(rendered);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while adding the product.".to_string());
            }
        };
    } else { // product to update
        match products_manager.backend().update(&product, &images, delete_media).await {
            Ok(_) => {
                println!("Product updated successfully.");

                tera.register_filter("round_and_format", utils::round_and_format_filter);

                let mut data = Context::new();
                data.insert("partial", "product");
                data.insert("product", &product);
                data.insert("categories", &categories);
                data.insert("alert", "Product updated");
                data.insert("status", &status_names);
                let rendered = tera.render("backend/admin.html", &data).unwrap();
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

    let products_manager = products::Products::new(pool.clone()).await;

    match products_manager.backend().get(id).await {
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
            for status in products::Status::iter() {
                status_names.push(status.as_str().to_string());
            }

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            println!("Product: {:?}", product);
            
            let mut data = Context::new();
            data.insert("partial", "product");
            data.insert("product", &product);
            data.insert("categories", &categories);
            data.insert("status", &status_names);
            let rendered = tera.render("backend/admin.html", &data).unwrap();
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

    let product = products::ProductBackend {
        id: 0,
        sku: "".to_string(),
        name: "".to_string(),
        slug: "".to_string(),
        description: "".to_string(),
        short_description: "".to_string(),
        regular_price: 0.00,
        price: 0.00,
        sale_price: 0.00,
        on_sale: false,
        stock_status: products::StockStatus::OutOfStock,
        stock_quantity: 0,
        weight: 0,
        permalink: "".to_string(),
        status: products::Status::Draft,
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
    for status in products::Status::iter() {
        status_names.push(status.as_str().to_string());
    }

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();
    data.insert("partial", "product");
    data.insert("product", &product);
    data.insert("categories", &categories);
    data.insert("status", &status_names);
    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}

pub async fn list(
    Query(pagination): Query<types::Pagination>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let products_manager = products::Products::new(pool).await;

    let total_count = products_manager
        .backend()
        .count_all()
        .await
        .unwrap_or(0);

    let per_page = pagination.per_page.unwrap_or(3);

    let total_pages: i32 = (total_count as f32 / per_page as f32).ceil() as i32;

    let mut page = pagination.page.unwrap_or(1) as i32;
    if page > total_pages {
        page = total_pages;
    } else if page == 0 {
        page = 1;
    }

    let products = if total_count > 0 {
        match products_manager.backend().get_all(
            page, 
            per_page as i32,
            pagination.order.unwrap_or(types::Order::Desc)).await {
            Ok(products) => products,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while fetching products.".to_string());
            },
        }
    } else {
        vec![]
    };

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();
    data.insert("partial", "products");
    data.insert("products", &products);
    data.insert("current_page", &page);
    data.insert("total_products", &total_count);
    data.insert("per_page", &per_page);
    data.insert("total_pages", &total_pages);

    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)
}
//
// Last Modification: 2024-08-14 19:43:31
//

use anyhow;
use crate::models::products;
use crate::types;
use crate::utils;

use axum::{
    extract::{
        Extension,
        Path,
        Query
    },
    response::Html,
};

use tera::{
    Tera,
    Context
};

pub async fn product(
    Path(slug):Path<String>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let products_manager = products::Products::new(pool);

    match products_manager
        .frontend()
        .get_one_by_slug(&slug)
        .await {
        Ok(product) => {
            println!("select product with PgRow:\n{:?}", product);

            tera.register_filter("round_and_format", utils::round_and_format_filter);
        
            let mut data = Context::new();
            data.insert("product", &product);
        
            let rendered = tera.render("frontend/product.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("An error occurred while fetching product.".to_string())
        },
    }
}

pub async fn get_products_data(
    pool: &sqlx::Pool<sqlx::Postgres>,
    data: &mut Context,
    parameters: &products::Parameters,
    category_slug: Option<&str>,
) -> Result<(), anyhow::Error> {

    let products_manager = products::Products::new(pool.clone());

    let page = match products_manager.frontend()
        .get_page(&parameters, category_slug)
        .await {
        Ok(page) => page,
        Err(e) => {
            return Err(anyhow::anyhow!("An error happened while fetching products: {}", e));
        },
    };
    
    let categories = match products_manager.frontend()
        .categories()
        .await {
        Ok(categories) => categories,
        Err(e) => {
            return Err(anyhow::anyhow!("An error occurred while fetching categories: {}", e));
        }
    };

    let mut query_parts = vec![];

    if parameters.min_price.is_some() {
        query_parts.push(format!("min_price={}",
            page.min_price));
    }
    if parameters.max_price.is_some() {
        query_parts.push(format!("max_price={}",
            page.max_price));
    }
    if parameters.on_sale.is_some() {
        query_parts.push(format!("on_sale={}",
            parameters.on_sale.unwrap_or(false)));
    }

    let order = |o: &Option<types::Order>| -> String {
        if o.is_some() {
            let order = o.as_ref().unwrap_or(&types::Order::Desc);
            query_parts.push(format!("order={}", order.as_str()));
            return order.as_str().to_string();
        }
        "".to_string()
    }(&parameters.order);

    let order_by = |o: &Option<String>| -> String {
        if o.is_some() {
            let order_by = products::products_order_by(&o);
            query_parts.push(format!("order_by={}", order_by));
            return order_by.to_string();
        }
        "".to_string()
    }(&parameters.order_by);

    data.insert("categories", &categories);

    // range price
    data.insert("default_min_price", &page.min_price);
    data.insert("default_max_price", &page.max_price);
    data.insert("min_price", match &parameters.min_price {
        Some(value) => value,
        None => &-1.00,
    });
    data.insert("max_price", match &parameters.max_price {
        Some(value) => value,
        None => &-1.00,
    });

    data.insert("order", &order);
    data.insert("order_by", &order_by);

    data.insert("query", &query_parts.join("&"));

    data.insert("products", &page.products);
    data.insert("current_page", &page.current_page);
    data.insert("total_products", &page.total_count);
    data.insert("per_page", &page.per_page);
    data.insert("total_pages", &page.total_pages);
    data.insert("on_sale", match &parameters.on_sale {
        Some(value) => value,
        None => &false,
    });

    Ok(())
}

pub async fn product_category(
    Path(slug):Path<String>,
    Query(parameters): Query<products::Parameters>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();

    let result = get_products_data(&pool, &mut data, &parameters, Some(&slug)).await;
    if result.is_err() {
        return Html(result.err().unwrap().to_string());
    }

    data.insert("path", &format!("/product-category/{}", slug));

    let rendered = tera.render("frontend/products.html", &data).unwrap();
    Html(rendered)
}


pub async fn list(
    Query(parameters): Query<products::Parameters>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
) -> Html<String> {

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();

    let result = get_products_data(&pool, &mut data, &parameters, None).await;
    if result.is_err() {
        return Html(result.err().unwrap().to_string());
    }

    data.insert("path", "/products");

    let rendered = tera.render("frontend/products.html", &data).unwrap();
    Html(rendered)
}
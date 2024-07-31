//
// Last Modification: 2024-07-27 19:20:38
//

use crate::types;
use crate::utils;
use crate::models::products;

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

    let products_manager = products::Products::new(pool).await;

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

pub async fn product_category(
    Path(slug):Path<String>,
    Query(pagination): Query<types::Pagination>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

        let products_manager = products::Products::new(pool.clone()).await;

        let per_page = pagination.per_page.unwrap_or(3);
        
        let total = products_manager
            .frontend()
            .count_all_category_by_slug(&slug)
            .await
            .unwrap_or(0);

        if total == 0 {
            return Html(format!("There are no products available for \"{}\" category", slug));
        }

        let total_pages: i32 = (total as f32 / per_page as f32).ceil() as i32;
    
        let mut page = pagination.page.unwrap_or(1) as i32;
        if page > total_pages {
            page = total_pages;
        } else if page == 0 {
            page = 1;
        }

        match products_manager.frontend().get_category_by_slug(
            &slug,
            page, 
            per_page as i32,
            pagination.order.unwrap_or(types::Order::Desc)).await {
            Ok(products) => {
    
                let categories = match products_manager.frontend().categories().await {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return Html("An error occurred while fetching categories.".to_string());
                    }
                };
    
                tera.register_filter("round_and_format", utils::round_and_format_filter);
    
                let mut data = Context::new();
                data.insert("path", &format!("/product-category/{}", slug));
                data.insert("categories", &categories);
                data.insert("products", &products);
                data.insert("current_page", &page);
                data.insert("total_products", &total);
                data.insert("per_page", &per_page);
                data.insert("total_pages", &total_pages);
                let rendered = tera.render("frontend/products.html", &data).unwrap();
                Html(rendered)
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                Html("An error occurred while fetching products.".to_string())
            },
        }

}

pub async fn list(
    Query(pagination): Query<types::Pagination>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>)  -> Html<String> {

    let products_manager = products::Products::new(pool.clone()).await;

    let per_page = pagination.per_page.unwrap_or(3);
    
    let total = match products_manager
        .frontend()
        .count_all(Some(products::Status::Publish))
        .await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error occurred while fetching product count.".to_string());
        }
    };

    let total_pages: i32 = (total as f32 / per_page as f32).ceil() as i32;

    let mut page = pagination.page.unwrap_or(1) as i32;
    if page > total_pages {
        page = total_pages;
    } else if page == 0 {
        page = 1;
    }

    let products = if total > 0 {
        match products_manager.frontend().get_all(
            Some(products::Status::Publish),
            page, 
            per_page as i32,
            pagination.order.unwrap_or(types::Order::Desc)).await {
                Ok(products) => products,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Html("An error occurred while fetching products.".to_string());
                }
        }
    } else {
        vec![]
    };

    let categories = match products_manager.frontend().categories().await {
        Ok(categories) => categories,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error occurred while fetching categories.".to_string());
        }
    };

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();
    data.insert("path", "/products");
    data.insert("categories", &categories);
    data.insert("products", &products);
    data.insert("current_page", &page);
    data.insert("total_products", &total);
    data.insert("per_page", &per_page);
    data.insert("total_pages", &total_pages);

    let rendered = tera.render("frontend/products.html", &data).unwrap();
    Html(rendered)
}
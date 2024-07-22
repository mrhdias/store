//
// Last Modification: 2024-07-22 18:39:38
//

// https://woocommerce.com/document/woocommerce-shortcodes/products/
// [products limit="4" columns="4" orderby="popularity" class="quick-sale" on_sale="true" ]
// products(limit=4, order="DESC", skus="6756443,6543237", ids="1,2,3,4,5,6,7")

use axum::{
    extract::{Extension, Query},
    response::Html,
};

use tera::{
    Tera,
    Context
};

use serde::{
    Serialize,
    Deserialize
};

use crate::types;
use crate::models::products;


#[derive(Debug, Serialize, Deserialize)]
struct Product {
    id: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductsParams {
    limit: Option<i32>,
    ids: Option<String>,
    skus: Option<String>,
    on_sale: Option<bool>,
    order: Option<String>,
    order_by: Option<String>,
}

//
// Example
// <div data-swap="outer" data-shortcode="/shortcode/products?ids=1,3,5&limit=4&skus=6654343,7548765&on_sale=true&order=desc"></div>
//
pub async fn products(
    Query(parameters): Query<ProductsParams>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>)  -> Html<String> {

    println!("Parameters: {:?}", parameters);

    let ids = match parameters.ids {
        Some(ids) => {
            ids.split(',')
               .map(|s| s.trim().parse::<i32>().unwrap())
               .collect::<Vec<i32>>()
        },
        None => Vec::new(),
    };

    let skus = match parameters.skus {
        Some(skus) => {
            skus.split(',')
               .map(|s| s.trim().to_string())
               .collect::<Vec<String>>()
        },
        None => Vec::new(),
    };

    let on_sale = match parameters.on_sale {
        Some(os) => os,
        None => false,
    };

    let per_page = match parameters.limit {
        Some(l) => l,
        None => 10,
    };


    let order = match parameters.order {
        Some(o) => {
            match o.as_str() {
                "asc" => types::Order::Asc,
                "desc" => types::Order::Desc,
                _ => types::Order::Asc,
            }
        },
        None => types::Order::Asc,
    };


    let order_by = match parameters.order_by {
        Some(ob) => ob,
        None => "name".to_string(),
    };

    let products_manager = products::Products::new(pool).await;
    let products = products_manager
        .frontend()
        .get_by_parameters(&ids, &skus, 1, per_page, order)
        .await.expect("unable to get the products");


    let mut data = Context::new();
    data.insert("products", &products);
    let rendered = tera.render("shortcodes/products.html", &data).unwrap();
    Html(rendered)
}


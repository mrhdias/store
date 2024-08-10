//
// Description: List all orders
// Last Modification: 2024-08-09 21:23:32
//

use crate::models::orders;
use crate::types;
use crate::utils;

use axum::{
    extract::{Extension, Path, Query},
    response::Html,
};

use tera::{
    Tera,
    Context
};

pub async fn new(
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    Html("Add new order unimplemented".to_string())
}

pub async fn edit(
    Path(id):Path<i32>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    Html("Edit order unimplemented".to_string())
}

/*
pub async fn list(
    Query(parameters): Query<orders::Parameters>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let orders_manager = orders::Orders::new(pool);

    let total_count = orders_manager
        .count_all()
        .await
        .unwrap_or(0);

    let per_page = parameters.per_page.unwrap_or(3);

    let total_pages: i32 = (total_count as f32 / per_page as f32).ceil() as i32;

    let mut page = parameters.page.unwrap_or(1) as i32;
    if page > total_pages {
        page = total_pages;
    } else if page == 0 {
        page = 1;
    }

    // println!("page: {} total_count: {} per_page: {} total_pages: {}",
    //     page, total_count, per_page, total_pages);

    let orders = if total_count > 0 {
        match orders_manager.get_all(
            page, 
            per_page as i32,
            parameters.order_by.unwrap_or(orders::OrderBy::Date),
            parameters.order.unwrap_or(types::Order::Desc)).await {
            Ok(orders) => orders,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Html("An error occurred while fetching orders.".to_string());
            },
        }
    } else {
        vec![]
    };

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();
    data.insert("partial", "orders");
    data.insert("orders", &orders);
    data.insert("current_page", &page);
    data.insert("total_orders", &total_count);
    data.insert("per_page", &per_page);
    data.insert("total_pages", &total_pages);
    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)

}
*/

pub async fn list(
    Query(parameters): Query<orders::Parameters>,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
) -> Html<String> {

    let orders_manager = orders::Orders::new(pool);

    let page = match orders_manager
        .get_page(&parameters)
        .await {
        Ok(page) => page,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Html("An error happened while fetching orders".to_string());
        },
    };

    tera.register_filter("round_and_format", utils::round_and_format_filter);

    let mut data = Context::new();
    data.insert("partial", "orders");
    data.insert("orders", &page.orders);
    data.insert("current_page", &page.current_page);
    data.insert("total_orders", &page.total_count);
    data.insert("per_page", &page.per_page);
    data.insert("total_pages", &page.total_pages);

    let rendered = tera.render("backend/admin.html", &data).unwrap();
    Html(rendered)

}
//
// Last Modified: 2024-07-30 18:41:46
//

use crate::models::cart;
use crate::models::cart::Product;
use crate::models::shipping;
use crate::models::orders;
use crate::utils;
use crate::notifications;
use std::collections::HashMap;

use axum::{
    extract::{Extension, Form},
    http::HeaderMap,
    response::Html,
};

use tower_sessions::Session;

use tera::{Tera, Context};

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct RawOrder {
    billing_first_name: String,
    billing_last_name: String,
    email: String,
    phone: String,
    billing_address: String,
    billing_postcode: String,
    billing_city: String,
    billing_country: String,
    tax_id_number: Option<String>,
    ship_to_different_address: Option<bool>,
    shipping_first_name: String,
    shipping_last_name: String,
    shipping_address: String,
    shipping_postcode: String,
    shipping_city: String,
    shipping_country: String,
    terms: Option<bool>,
    payment_method: String,
    order_comments: Option<String>,
    calculate_shipping: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Billing {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
    address: String,
    postcode: String,
    city: String,
    country_code: String,
    tax_id_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Shipping {
    first_name: String,
    last_name: String,
    address: String,
    postcode: String,
    city: String,
    country_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Country {
    code: String,
    name: String,
}

fn checkout_data(
    products: Vec<Product>,
    payload: RawOrder,
    total_shipping: &f32,
    alert: &str) -> Context {

    let countries = vec![
        Country{
            code: "FR".to_string(),
            name: "France".to_string(),
        },
        Country{
            code: "PT".to_string(),
            name: "Portugal".to_string(),
        },
        Country{
            code: "ES".to_string(),
            name: "Spain".to_string(),
        },
    ];

    let mut data = Context::new();
    data.insert("partial", "checkout");
    data.insert("title", "Checkout");
    if !alert.is_empty() {
        data.insert("alert", &alert);
    }
    data.insert("countries", &countries);
    data.insert("billing", &Billing {
        first_name: payload.billing_first_name,
        last_name: payload.billing_last_name,
        email: payload.email,
        phone: payload.phone,
        address: payload.billing_address,
        postcode: payload.billing_postcode,
        city: payload.billing_city,
        country_code: payload.billing_country,
        tax_id_number: match payload.tax_id_number {
            Some(value) => value,
            None => "".to_string(),
        },
    });
    data.insert("ship_to_different_address", &match payload.ship_to_different_address {
        Some(value) => value,
        None => false,
    });
    data.insert("shipping", &Shipping {
        first_name: payload.shipping_first_name,
        last_name: payload.shipping_last_name,
        address: payload.shipping_address,
        postcode: payload.shipping_postcode,
        city: payload.shipping_city,
        country_code: payload.shipping_country,
    });
    data.insert("order_comments", &match payload.order_comments {
        Some(value) => value,
        None => "".to_string(),
    });
    data.insert("cart", &products);
    data.insert("shipping_total", &total_shipping);

    data
}

pub async fn place_order(
    headers: HeaderMap,
    session: Session,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
    Form(payload): Form<RawOrder>) -> Html<String> {

    println!("Order Billing: {:?}", payload);

    let mut current_cart: HashMap<i32, i32> = match session.get("cart").await.unwrap() {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool.clone(), &mut current_cart);
    match cart.get().await {
        Ok(products) => {

            let total_weight = cart.total_weight;
            println!("Total weight: {}", total_weight);

            let shipping = shipping::Shipping::new(pool.clone());

            let mut alert = "".to_string();
            let total_shipping = match shipping.calculate(
                &payload.shipping_country,
                &payload.shipping_postcode,
                &cart.total_weight,
                &cart.total_order) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("Error calculating shipping: {}", e);
                        alert = format!("Error calculating shipping: {}", e);
                        -1.00
                    }
                };

            println!("Shipping Total: {} {:?}", total_shipping, payload.calculate_shipping);

            if payload.calculate_shipping.is_some() {

                println!("Show Shipping Calculations: {}", total_shipping);

                session.insert("cart", current_cart).await.unwrap();

                tera.register_filter("round_and_format", utils::round_and_format_filter);

                let data = checkout_data(products, payload, &total_shipping, &alert);
                let rendered = tera.render("frontend/shopping.html", &data).unwrap();
                Html(rendered)

            } else {

                println!("Place order...");

                let client_ip = headers
                    .get("X-Forwarded-For")
                    .and_then(|value| value.to_str().ok())
                    .or_else(|| {
                        headers
                            .get("X-Real-IP")
                            .and_then(|value| value.to_str().ok())
                    })
                    .unwrap_or("Unknown");

                // Get the User-Agent header
                let user_agent = headers
                    .get(axum::http::header::USER_AGENT)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("Unknown");

                let mut total = 0.00;

                let order = orders::Order {
                    customer_ip_address: client_ip.to_string(),
                    customer_user_agent: user_agent.to_string(),
                    billing: orders::Billing {
                        first_name: payload.billing_first_name,
                        last_name: payload.billing_last_name,
                        email: payload.email,
                        phone: payload.phone,
                        address: payload.billing_address,
                        postcode: payload.billing_postcode,
                        city: payload.billing_city,
                        country_code: payload.billing_country,
                        tax_id_number: match payload.tax_id_number {
                            Some(value) => value,
                            None => "".to_string(),
                        },
                    },
                    shipping: orders::Shipping {
                        first_name: payload.shipping_first_name,
                        last_name: payload.shipping_last_name,
                        address: payload.shipping_address,
                        postcode: payload.shipping_postcode,
                        city: payload.shipping_city,
                        country_code: payload.shipping_country,
                    },
                    line_items: || -> Vec<orders::LineItem> {
                        let mut line_items = Vec::new();
                        for product in products {
                            line_items.push(orders::LineItem {
                                product_id: product.id,
                                sku: product.sku,
                                name: product.name,
                                price: product.price,
                                quantity: product.quantity,
                                subtotal: product.price * product.quantity as f32,
                                total: product.price * product.quantity as f32,
                            });
                            total += product.price * product.quantity as f32;
                        }
                        line_items
                    }(),
                    shipping_items: || -> Vec<orders::ShippingLine> {
                        let mut shipping_items = Vec::new();
                        shipping_items.push(orders::ShippingLine {
                            total: total_shipping,
                        });
                        shipping_items
                    }(),
                    total: total + total_shipping,
                    customer_note: match payload.order_comments {
                        Some(value) => value,
                        None => "".to_string(),
                    },
                    status: orders::OrderStatus::Pending,
                };
                let order_manager = orders::Orders::new(pool.clone());

                match order_manager.add(&order).await {
                    Ok(order_id) => {
                        println!("Added order id: {}", order_id);

                        // let mailer = notifications::SMTP::new(pool);
                        // mailer.send().await;

                        cart.reset();
                        session.insert("cart", current_cart).await.unwrap();

                        tera.register_filter("round_and_format", utils::round_and_format_filter);

                        let mut data = Context::new();
                        data.insert("partial", "order_details");
                        data.insert("title", "Order Details");
                        data.insert("cart", "yes");
                        data.insert("order", &order);
                        let rendered = tera.render("frontend/shopping.html", &data).unwrap();
                        Html(rendered)
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        alert = format!("Happened an error adding order: {}", e);
                        Html("Happened an error adding order".to_string())
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}

pub async fn show(
    session: Session,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart").await.unwrap() {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    match cart.get().await {
        Ok(products) => {
            session.insert("cart", current_cart).await.unwrap();

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let countries = vec![
                Country{
                    code: "FR".to_string(),
                    name: "France".to_string(),
                },
                Country{
                    code: "PT".to_string(),
                    name: "Portugal".to_string(),
                },
                Country{
                    code: "ES".to_string(),
                    name: "Spain".to_string(),
                },
            ];

            let mut data = Context::new();
            data.insert("partial", "checkout");
            data.insert("title", "Checkout");
            data.insert("countries", &countries);
            data.insert("billing", &Billing {
                first_name: "".to_string(),
                last_name: "".to_string(),
                email: "".to_string(),
                phone: "".to_string(),
                address: "".to_string(),
                postcode: "".to_string(),
                city: "".to_string(),
                country_code: "".to_string(),
                tax_id_number: "".to_string(),
            });
            data.insert("shipping_to_different_address", &false);
            data.insert("shipping", &Shipping {
                first_name: "".to_string(),
                last_name: "".to_string(),
                address: "".to_string(),
                postcode: "".to_string(),
                city: "".to_string(),
                country_code: "".to_string(),
            });
            data.insert("order_comments", "");
            data.insert("cart", &products);
            data.insert("shipping_total", &-1.00);
            let rendered = tera.render("frontend/shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}
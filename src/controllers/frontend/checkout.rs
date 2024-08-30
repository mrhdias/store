//
// Last Modified: 2024-08-30 19:30:09
// References:
// https://woocommerce.com/document/managing-orders/order-statuses/
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


#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
struct Shipping {
    first_name: String,
    last_name: String,
    address: String,
    postcode: String,
    city: String,
    country_code: String,
}

#[derive(Debug, Serialize)]
struct Country {
    code: String,
    name: String,
}

fn calc_tax_value(
    price: f32,
    tax_rate: f32,
    price_include_tax: bool) -> f32 {

    if price_include_tax {
        return price / (1.00 + tax_rate / 100.00)
    }
    price * tax_rate / 100.00
}

fn checkout_data(
    payload: RawOrder,
    products: &Vec<Product>,
    total_shipping: &f32,
    alert: Option<&str>
) -> Context {

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

    if alert.is_some() && !alert.unwrap().is_empty() {
        data.insert("alert", alert.unwrap());
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
    data.insert("cart", products);
    data.insert("shipping_total", total_shipping);

    data
}

pub async fn place_order(
    headers: HeaderMap,
    session: Session,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(tera): Extension<Tera>,
    Form(payload): Form<RawOrder>) -> Html<String> {

    println!("Order Billing: {:?}", payload);

    let mut current_cart: HashMap<i32, i32> = match session.get("cart").await.unwrap() {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool.clone(), &mut current_cart);
    if cart.is_empty() {

        let products: Vec<Product> = vec![];

        let mut data = Context::new();
        data.insert("partial", "cart");
        data.insert("title", "Cart - Empty Cart");
        data.insert("cart", &products);
        let rendered = tera.render("frontend/shopping.html", &data).unwrap();

        return Html(rendered);
    }

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

                let data = checkout_data(
                    payload,
                    &products,
                    &total_shipping,
                    Some(&alert));
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

                let tax_rate = 23.00; // This is temporary, as soon as possible put in the database
                let prices_include_tax = true; // This is temporary, as soon as possible put in the database

                let mut order = orders::Order {
                    order_key: "order_58d2d042d1d".to_string(),
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
                    line_items: vec![],
                    shipping_items: vec![],
                    payment_method: "bacs".to_string(),
                    payment_method_title: "Direct Bank Transfer".to_string(),
                    currency: orders::Currency::EUR,
                    discount_total: 0.00,
                    discount_tax: 0.00,
                    shipping_total: 0.00,
                    shipping_tax: 0.00,
                    cart_tax: 0.00,
                    total: 0.00,
                    total_tax: 0.00,
                    prices_include_tax,
                    customer_note: match payload.order_comments {
                        Some(value) => value,
                        None => "".to_string(),
                    },
                    status: orders::OrderStatus::OnHold,
                    cart_hash: "".to_string(),
                };

                order.line_items = || -> Vec<orders::LineItem> {
                    let mut line_items = Vec::new();
                    for product in products {

                        let line_item = orders::LineItem {
                            product_id: product.id,
                            sku: product.sku,
                            name: product.name,
                            price: product.price,
                            quantity: product.quantity,
                            subtotal: product.regular_price * product.quantity as f32, // Line subtotal (before discounts)
                            subtotal_tax: calc_tax_value(product.regular_price,
                                tax_rate, prices_include_tax) * product.quantity as f32,
                            total: product.price * product.quantity as f32, // Line total (after discounts).
                            total_tax: calc_tax_value(product.price,
                                tax_rate, prices_include_tax) * product.quantity as f32,
                        };

                        order.discount_total += (product.regular_price - product.price) * product.quantity as f32;
                        order.discount_tax += calc_tax_value(product.regular_price - product.price,
                            tax_rate, prices_include_tax) * product.quantity as f32;

                        order.total += line_item.total;
                        order.total_tax += line_item.total_tax;
                        order.cart_tax += line_item.total_tax;

                        line_items.push(line_item);

                    }
                    line_items
                }();

                order.shipping_items = || -> Vec<orders::ShippingLine> {
                    let mut shipping_items = Vec::new();

                    let shipping_item = orders::ShippingLine {
                        total: total_shipping,
                        total_tax: calc_tax_value(total_shipping,
                            tax_rate, prices_include_tax),
                    };

                    order.shipping_total += shipping_item.total;
                    order.total += shipping_item.total;
                    order.total_tax += shipping_item.total_tax;

                    shipping_items.push(shipping_item);

                    shipping_items
                }();

                let order_manager = orders::Orders::new(pool.clone());

                match order_manager.add(&order).await {
                    Ok(order_id) => {
                        println!("Added order id: {}", order_id);

                        // let mailer = notifications::SMTP::new(pool);
                        // mailer.send().await;

                        cart.reset();
                        session.insert("cart", current_cart).await.unwrap();

                        let mut data = Context::new();
                        data.insert("partial", "order_details");
                        data.insert("title", "Order Details");
                        data.insert("number", &order_id);
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
    Extension(tera): Extension<Tera>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart").await.unwrap() {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = cart::Cart::new(pool, &mut current_cart);
    match cart.get().await {
        Ok(products) => {
            session.insert("cart", current_cart).await.unwrap();

            let data = checkout_data(
                RawOrder {
                    billing_first_name: "".to_string(),
                    billing_last_name: "".to_string(),
                    email: "".to_string(),
                    phone: "".to_string(),
                    billing_address: "".to_string(),
                    billing_postcode: "".to_string(),
                    billing_city: "".to_string(),
                    billing_country: "".to_string(),
                    tax_id_number: None,
                    ship_to_different_address: None,
                    shipping_first_name: "".to_string(),
                    shipping_last_name: "".to_string(),
                    shipping_address: "".to_string(),
                    shipping_postcode: "".to_string(),
                    shipping_city: "".to_string(),
                    shipping_country: "".to_string(),
                    terms: None,
                    payment_method: "".to_string(),
                    order_comments: None,
                    calculate_shipping: None,
                },
                &products,
                &-1.00,
                None);


            let rendered = tera.render("frontend/shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}
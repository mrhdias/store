//
// Last Modified: 2024-07-25 19:35:33
//

use crate::models::cart;
use crate::utils;
use crate::notifications;
use std::collections::HashMap;
use regex::Regex;

use anyhow;
use axum::{
    extract::{Extension, Form},
    response::Html,
};

use tower_sessions::Session;

use tera::{Tera, Context};

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
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


struct ShippingWeightTable {
    label: String,
    postalcode_regex: Vec<String>,
    prices: Vec<(u32, f32)>,
    freeshipping: f32,
}

fn calculate_shipping(
    country: &str,
    postalcode: &str,
    total_weight: &u32,
    total_order: &f32) -> Result<f32, anyhow::Error> {

    let mut countries = HashMap::new();
    countries.insert("PT".to_string(), vec![
        ShippingWeightTable {
            label: "mainland".to_string(),
            postalcode_regex: vec![r#"^[12345678]\d+"#.to_string()],
            prices: vec![
                (1000, 4.90),
                (2000, 8.30),
                (3000, 12.70),
                (4000, 17.10),
                (5000, 21.50),
            ],
            freeshipping: 100.0,
        },
        ShippingWeightTable {
            label: "madeira".to_string(),
            postalcode_regex: vec![r#"^9[012344]\d+"#.to_string()],
            prices: vec![
                (1000, 5.90),
                (2000, 9.30),
                (3000, 13.70),
                (4000, 18.10),
                (5000, 22.50)
            ],
            freeshipping: 100.0,
        },
        ShippingWeightTable {
            label: "acores".to_string(),
            postalcode_regex: vec![r#"^9[56789]\d+"#.to_string()],
            prices: vec![
                (1000, 6.90),
                (2000, 10.30),
                (3000, 14.70),
                (4000, 19.10),
                (5000, 23.50)
            ],
            freeshipping: 300.0,
        },
    ]);
    countries.insert("ES".to_string(), vec![
        ShippingWeightTable {
            label: "mainland".to_string(),
            postalcode_regex: vec![],
            prices: vec![
                (1000, 7.90),
                (2000, 11.30),
                (3000, 15.70),
                (4000, 20.10),
                (5000, 24.50)
            ],
            freeshipping: 100.0,
        },
    ]);

    if countries.is_empty() {
        return Err(anyhow::anyhow!("shipping tables is empty"));
    }

    if !countries.contains_key(country) {
        return Err(anyhow::anyhow!("shipping table not found for country: {}", country));
    }

    let country_tables = countries.get(country).unwrap();
    for table in country_tables {
        if total_order >= &table.freeshipping {
            return Ok(0.00);
        }
        for pattern in &table.postalcode_regex {
            let regex = Regex::new(&pattern).unwrap();
            if regex.is_match(postalcode) {
                for (weight, price) in &table.prices {
                    if total_weight <= &weight {
                        return Ok(*price);
                    }
                }
                return Err(anyhow::anyhow!("price shipping not found for weight: {}/{}", country, total_weight));
            }
        }
    }
    return Err(anyhow::anyhow!("shipping table is empty for country: {}", country));
}

pub async fn place_order(
    session: Session,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
    Form(payload): Form<Order>) -> Html<String> {

    println!("Order Billing: {:?}", payload);

    if Some(payload.calculate_shipping).is_some() {
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
                data.insert("shipping_total", &1.00);
                let rendered = tera.render("shopping.html", &data).unwrap();
                Html(rendered)
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                Html("Happen an error when get the cart".to_string())
            },
        }
    } else {
        // remove the cart from session after place the order

        let mailer = notifications::SMTP::new(pool);
        mailer.send().await;

        Html("Ok".to_string())
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
            data.insert("shipping_total", &0.00);
            let rendered = tera.render("shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}
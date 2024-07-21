//
// Last Modified: 2024-07-03 16:45:16
//

use crate::utils;

use anyhow;
use std::collections::HashMap;
use num_traits::ToPrimitive;

use axum::{
    extract::{Extension, Form, RawForm},
    response::Html,
};

use tera::{
    Tera,
    Context
};

use serde::{Serialize, Deserialize};

use sqlx::{
    types::{Decimal, Json},
    FromRow,
    Row
};

use axum_session_sqlx::SessionPgSession;

#[derive(Debug, Serialize, Deserialize)]
struct Media {
    id: i32,
    src: String,
    name: String,
    alt: String,
    date_created: String,
    date_modified: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Product {
    id: i32,
    name: String,
    price: f32,
    quantity: i32,
    permalink: String,
    image: Json<Media>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductToCart {
    product_id: i32,
    product_quantity: i32,
}

pub struct Cart<'a> {
    pool: sqlx::Pool<sqlx::Postgres>,
    cart: &'a mut HashMap<i32, i32>,
}

impl<'a> Cart<'a> {

    fn add(&mut self, product_to_cart: &ProductToCart) {
        if self.cart.contains_key(&product_to_cart.product_id) {
            if let Some(value) = self.cart.get_mut(&product_to_cart.product_id) {
                *value += product_to_cart.product_quantity;
            }
        } else {
            self.cart.insert(product_to_cart.product_id, product_to_cart.product_quantity);
        }
    }

    fn update(&mut self, raw_form_data: &str) {

        let v: Vec<&str> = raw_form_data.split('&').collect();
    
        let mut current_key = 0;
        for key_value in v.iter() {
            key_value.split_once('=').map(|(k, v)| {
                println!("key: {}, value: {}", k, v);
                match k {
                    "id" => {
                        current_key = v.parse::<i32>().unwrap();
                        self.cart.insert(current_key, 0);
                    },
                    "quantity" => {
                        if let Some(value) = self.cart.get_mut(&current_key) {
                            *value += &v.parse::<i32>().unwrap();
                        }
                    },
                    "remove" => {
                        self.cart.remove(&current_key);
                    },
                    _ => {
                        eprintln!("Invalid cart key: {:?}", current_key);
                    }
                }
            });
        }
    }


    pub async fn get(&mut self) -> Result<Vec<Product>, anyhow::Error> {
        let product_ids: Vec<i32> = self.cart.keys().cloned().collect();

        let products = sqlx::query(r#"
            SELECT
                products.id, products.name, products.permalink, products.price,
                products.stock_quantity, products.stock_status, (
                SELECT json_build_object(
                    'id', media.id,
                    'src', media.src,
                    'name', media.name,
                    'alt', media.alt,
                    'date_created', date_created,
                    'date_modified', date_modified)
                FROM product_media, media
                WHERE product_media.media_id = media.id AND product_media.product_id = products.id
                ORDER BY product_media.position DESC LIMIT 1) AS image
            FROM products
            WHERE products.id = ANY($1) AND products.status = 'publish';
        "#)
            .bind(product_ids)
            .map(|row| Product {
                id: row.get::<i32, _>("id"),
                name: row.get::<String, _>("name"),
                price: match row.get::<Decimal, _>("price").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                quantity: match self.cart.get(&row.get::<i32, _>("id")) {
                    Some(q) => {
                        let id = &row.get::<i32, _>("id");
                        let stock_quantity = row.get::<i32, _>("stock_quantity");
                        
                        let mut quantity = *q;
                        if stock_quantity == 0 {
                            self.cart.remove(id);
                            quantity = 0;
                        } else if *q > stock_quantity {
                            quantity = stock_quantity;
                        }
                        quantity
    
                    },
                    None => 0,
                },
                permalink: row.get::<String, _>("permalink"),
                image: row.get::<Json<Media>, _>("image"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(products)
    }


    pub fn new(
        pool: sqlx::Pool<sqlx::Postgres>,
        cart: &'a mut HashMap<i32, i32>) -> Self {

        Cart {
            pool,
            cart,
        }
    }
}


pub async fn update_cart(
    session: SessionPgSession,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
    RawForm(form): RawForm) -> Html<String> {

    let raw_form_data = String::from_utf8(form.to_vec()).unwrap();

    let mut current_cart: HashMap<i32, i32> = match session.get("cart") {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = Cart::new(pool, &mut current_cart);
    cart.update(&raw_form_data);

    match cart.get().await {
        Ok(products) => {
            session.set("cart", current_cart);

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}

pub async fn add_to_cart(
    session: SessionPgSession,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>,
    Form(payload): Form<ProductToCart>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart") {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = Cart::new(pool, &mut current_cart);
    cart.add(&payload);

    match cart.get().await {
        Ok(products) => {
            session.set("cart", current_cart);

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }
}

pub async fn show(
    session: SessionPgSession,
    Extension(pool): Extension<sqlx::Pool<sqlx::Postgres>>,
    Extension(mut tera): Extension<Tera>) -> Html<String> {

    let mut current_cart: HashMap<i32, i32> = match session.get("cart") {
        Some(cart) => cart,
        None => HashMap::new()
    };

    let mut cart = Cart::new(pool, &mut current_cart);
    match cart.get().await {
        Ok(products) => {
            session.set("cart", current_cart);

            tera.register_filter("round_and_format", utils::round_and_format_filter);

            let mut data = Context::new();
            data.insert("partial", "cart");
            data.insert("title", "Cart");
            data.insert("cart", &products);
            let rendered = tera.render("shopping.html", &data).unwrap();
            Html(rendered)
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            Html("Happen an error when get the cart".to_string())
        },
    }

}
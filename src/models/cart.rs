//
// Last Modification: 2024-07-30 19:16:25
//

use anyhow;
use std::collections::HashMap;
use num_traits::ToPrimitive;
use serde::{Serialize, Deserialize};

use sqlx::{
    types::{Decimal, Json},
    FromRow,
    Row,
};

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
    pub id: i32,
    pub name: String,
    pub sku: String,
    pub price: f32,
    pub regular_price: f32,
    pub quantity: i32,
    weight: u32,
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
    pub total_weight: u32,
    pub total_order: f32,
}

impl<'a> Cart<'a> {

    pub fn reset(&mut self) {
        self.cart.clear();
        self.total_weight = 0;
        self.total_order = 0.0;
    }

    pub fn add(&mut self, product_to_cart: &ProductToCart) {
        if self.cart.contains_key(&product_to_cart.product_id) {
            if let Some(value) = self.cart.get_mut(&product_to_cart.product_id) {
                *value += product_to_cart.product_quantity;
            }
        } else {
            self.cart.insert(product_to_cart.product_id, product_to_cart.product_quantity);
        }
    }

    pub fn update(&mut self, raw_form_data: &str) {

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
                products.id, products.name, products.sku, products.permalink, products.price, products.regular_price,
                products.stock_quantity, products.weight, products.stock_status, (
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
            .map(|row| -> Product {

                let product = Product {
                    id: row.get::<i32, _>("id"),
                    name: row.get::<String, _>("name"),
                    sku: row.get::<String, _>("sku"),
                    price: match row.get::<Decimal, _>("price").to_f32() {
                        Some(f) => f,
                        None => 0.00,
                    },
                    regular_price: match row.get::<Decimal, _>("regular_price").to_f32() {
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
                    weight: row.get::<i32, _>("weight") as u32,
                    permalink: row.get::<String, _>("permalink"),
                    image: row.get::<Json<Media>, _>("image"),
                };

                self.total_weight += product.weight * product.quantity as u32;
                self.total_order += product.price * product.quantity as f32;

                product
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
            total_weight: 0,
            total_order: 0.00,
        }
    }
}
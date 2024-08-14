//
// Last Modification: 2024-08-14 19:18:21
//

use crate::models::products::StockStatus;
use crate::models::products::Media;

use sqlx::types::Json;

use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductShort {
    pub id: i32,
    pub sku: String,
    pub name: String,
    pub slug: String,
    pub permalink: String,
    pub description: String,
    pub short_description: String,
    pub price: f32, // current product price (read-only)
    pub regular_price: f32, // product regular price
    pub sale_price: f32, // product sale price
    pub on_sale: bool, // shows if the product is on sale (read-only)
    pub stock_quantity: i32,
    pub stock_status: StockStatus,
    pub weight: u32,
    pub gallery: Json<Vec<Media>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub sku: String,
    pub name: String,
    pub slug: String,
    pub permalink: String,
    pub description: String,
    pub short_description: String,
    pub price: f32, // current product price (read-only)
    pub regular_price: f32, // product regular price
    pub sale_price: f32, // product sale price
    pub on_sale: bool, // shows if the product is on sale (read-only)
    pub stock_quantity: i32,
    pub stock_status: StockStatus,
    pub weight: u32,
    // categories: Vec<Category>,
    pub gallery: Json<Vec<Media>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductPage {
    pub products: Vec<ProductShort>,
    pub total_count: i32,
    pub current_page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub min_price: f32,
    pub max_price: f32,
}

impl ProductPage {
    pub fn new() -> Self {
        ProductPage {
            products: Vec::new(),
            total_count: 0,
            current_page: 0,
            per_page: 0,
            total_pages: 0,
            min_price: 0.00,
            max_price: 0.00,
        }
    }
}
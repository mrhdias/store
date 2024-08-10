//
// Last Modification: 2024-08-01 18:41:41
//

use crate::models::products::StockStatus;
use crate::models::products::Status;
use crate::models::products::ProductImage;

use serde::{
    Serialize,
    Deserialize
};

// Products

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductShort {
    pub id: i32,
    pub sku: String,
    pub name: String,
    pub regular_price: f32, // product regular price
    pub sale_price: f32, // product sale price
    pub on_sale: bool, // shows if the product is on sale (read-only)
    pub stock_status: StockStatus,
    pub stock_quantity: i32,
    pub image_src: String,
    pub image_alt: String,
    pub date_created: String,
    pub status: Status,
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
    pub stock_status: StockStatus,
    pub stock_quantity: i32,
    pub weight: u32,
    pub status: Status,
    pub primary_category: i32,
    pub categories: Vec<i32>,
    pub images: Vec<ProductImage>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ProductPage {
    pub products: Vec<ProductShort>,
    pub total_count: i32,
    pub current_page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl ProductPage {
    pub fn new() -> Self {
        ProductPage {
            products: Vec::new(),
            total_count: 0,
            current_page: 0,
            per_page: 0,
            total_pages: 0,
        }
    }
}

// Categories

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryShort {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub parent: i32,
    pub description: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryPage {
    pub categories: Vec<CategoryShort>,
    pub total_count: i32,
    pub current_page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl CategoryPage {
    pub fn new() -> Self {
        CategoryPage {
            categories: Vec::new(),
            total_count: 0,
            current_page: 0,
            per_page: 0,
            total_pages: 0,
        }
    }
}

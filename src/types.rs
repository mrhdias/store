//
// Last Modification: 2024-08-02 18:55:08
//


use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "stock_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum StockStatus {
    InStock,
    OutOfStock,
    OnBackorder
}

impl StockStatus {
    fn as_str(&self) -> &str {
        match self {
            StockStatus::InStock => "instock",
            StockStatus::OutOfStock => "outofstock",
            StockStatus::OnBackorder => "onbackorder",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn as_str(&self) -> &str {
        match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<Order>,
}
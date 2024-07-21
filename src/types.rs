//
// Last Modification: 2024-07-17 19:46:12
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

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_roles", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRoles {
    Admin,
    Customer,
    Anonymous,
}

impl UserRoles {
    fn as_str(&self) -> &str {
        match self {
            UserRoles::Admin => "admin",
            UserRoles::Customer => "customer",
            UserRoles::Anonymous => "anonymous",
        }
    }

    pub fn is_admin(&self) -> bool {
        self == &UserRoles::Admin
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn as_str(&self) -> &str {
        match self {
            Order::Asc => "ASC",
            Order::Desc => "DESC",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<Order>,
}
//
// Description: Manage customer orders
// Last Moficication: 2024-08-01 22:15:47
//

use crate::types;
use chrono::{DateTime, NaiveDateTime, Utc};
use num_traits::ToPrimitive;
use anyhow;

use sqlx::{
    postgres::PgRow,
    types::Decimal,
    Row,
};

use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "currency", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    EUR, // Default
    USD,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending, // Default
    Processing,
    OnHold,
    Completed,
    Cancelled,
    Refunded,
    Failed,
    Trash,
}

impl OrderStatus {
    fn as_str(&self) -> &str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Processing => "processing",
            OrderStatus::OnHold => "on-hold",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
            OrderStatus::Refunded => "refunded",
            OrderStatus::Failed => "failed",
            OrderStatus::Trash => "trash",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderBy {
    Date, // default
    Modified,
    Id,
    // Include,
    // Title,
    // Slug,
    // Price,
}

impl OrderBy {
    pub fn as_str(&self) -> &str {
        match self {
            OrderBy::Date => "date_created",
            OrderBy::Modified => "date_modified",
            OrderBy::Id => "id",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Parameters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<types::Order>,
    pub order_by: Option<OrderBy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Billing {
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub postcode: String,
    pub country_code: String,
    pub email: String,
    pub phone: String,
    pub tax_id_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shipping {
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub postcode: String,
    pub country_code: String,
}

// https://woocommerce.github.io/woocommerce-rest-api-docs/#order-line-items-properties
#[derive(Debug, Serialize, Deserialize)]
pub struct LineItem {
    pub product_id: i32,
    pub sku: String,
    pub name: String,
    pub price: f32,
    pub quantity: i32,
    pub subtotal: f32, // before discounts
    pub subtotal_tax: f32, // before discounts
    pub total: f32, // after discounts
    pub total_tax: f32, // after discounts
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShippingLine {
    pub total: f32,
    pub total_tax: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_key: String,
    pub customer_ip_address: String,
    pub customer_user_agent: String,
    pub billing: Billing,
    pub shipping: Shipping,
    pub line_items: Vec<LineItem>,
    pub shipping_items: Vec<ShippingLine>,
    pub payment_method: String,
    pub payment_method_title: String,
    pub currency: Currency,
    pub discount_total: f32,
    pub discount_tax: f32,
    pub shipping_total: f32,
    pub shipping_tax: f32,
    pub cart_tax: f32,
    pub total: f32,
    pub total_tax: f32,
    pub prices_include_tax: bool,
    pub customer_note: String,
    pub status: OrderStatus,
    pub cart_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRow {
    id: i32,
    date_created: String,
    customer_name: String,
    status: String,
    total: f32,
}

pub struct Orders {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Orders {

    pub async fn get_all(&self,
        page: i32,
        per_page: i32,
        order_by: OrderBy,
        order: types::Order) -> Result<Vec<OrderRow>, anyhow::Error> {
        // Implementation to get orders

        let offset = (page - 1) * per_page;

        let orders = sqlx::query(&format!(r#"
            SELECT
                orders.id, orders.date_created, orders.status,
                (billing->>'first_name') || ' ' || (billing->>'last_name') AS customer_name,
                orders.total
            FROM orders
            ORDER BY 
                orders.{} {}
            LIMIT $1 OFFSET $2;
        "#, order_by.as_str(), order.as_str()))
            .bind(per_page)
            .bind(offset)
            .map(|row: PgRow| OrderRow {
                id: row.get::<i32, _>("id"),
                date_created: || -> String {

                    let date_created = row.get::<NaiveDateTime, _>("date_created");
                    let date_created_utc: DateTime<Utc> = DateTime::from_naive_utc_and_offset(date_created, Utc);

                    let now = Utc::now();
                    let duration = now.signed_duration_since(date_created_utc);

                    match duration {
                        d if d.num_days() > 0 => date_created.format("%b %d, %Y").to_string(),
                        d if d.num_hours() > 0 => format!("{} hours ago", d.num_hours()),
                        d if d.num_minutes() > 0 => format!("{} minutes ago", d.num_minutes()),
                        d if d.num_seconds() > 0 => format!("{} seconds ago", d.num_seconds()),
                        _ => "Just now".to_string(),
                    }

                    // date_created.format("%b %d, %Y").to_string()
                    // date_created.format("%Y/%m/%d at %H:%M:%S").to_string()
                }(),
                status: || -> String {
                    let status = row.get::<OrderStatus, _>("status");
                    status.as_str().to_string()
                }(),
                total: match row.get::<Decimal, _>("total").to_f32() {
                    Some(f) => f,
                    None => 0.00,
                },
                customer_name: row.get::<String, _>("customer_name"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(orders)
    }

    pub async fn add(&self, order: &Order) -> Result<i32, anyhow::Error> {

        let billing_json = serde_json::to_string(&order.billing)?;
        let shipping_json = serde_json::to_string(&order.shipping)?;
        let line_items_json = serde_json::to_string(&order.line_items)?;
        let shipping_items_json = serde_json::to_string(&order.shipping_items)?;

        let order_id: i32 = sqlx::query(r#"
            INSERT INTO orders (
                order_key, customer_ip_address, customer_user_agent, customer_note,
                billing, shipping, line_items, shipping_lines,
                payment_method, payment_method_title, status, currency,
                discount_total, discount_tax,
                shipping_total, shipping_tax,
                cart_tax,
                total, total_tax,
                prices_include_tax,
                cart_hash
            )
            VALUES (
                $1, $2, $3, $4,
                $5::jsonb, $6::jsonb, $7::jsonb, $8::jsonb,
                $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21
            ) RETURNING id;
        "#)
            .bind(&order.order_key)
            .bind(&order.customer_ip_address)
            .bind(&order.customer_user_agent)
            .bind(&order.customer_note)
            .bind(billing_json)
            .bind(shipping_json)
            .bind(line_items_json)
            .bind(shipping_items_json)
            .bind(&order.payment_method)
            .bind(&order.payment_method_title)
            .bind(&order.status)
            .bind(&order.currency)
            .bind(&order.discount_total)
            .bind(&order.discount_tax)
            .bind(&order.shipping_total)
            .bind(&order.shipping_tax)
            .bind(&order.cart_tax)
            .bind(&order.total)
            .bind(&order.total_tax)
            .bind(&order.prices_include_tax)
            .bind(&order.cart_hash)
            .fetch_one(&self.pool)
            .await?
            .get(0);

        Ok(order_id)
    }

    pub async fn count_all(&self) -> Result<i32, anyhow::Error> {
        let total_count: i64 = sqlx::query("SELECT COUNT(*) FROM orders")
            .fetch_one(&self.pool)
            .await?
            .get(0);

        Ok(total_count as i32)
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        // Implementation to create a new instance of Orders

        Orders {
            pool,
        }
    }
}
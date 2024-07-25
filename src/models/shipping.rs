//
// Last Modification: 2024-07-24 18:54:43
//

struct Table {
    id: i32,
    label: String,
    postalcode_regex: Vec<String>,
    prices: Vec<(u32, f32)>, // (Weight, Price)
    freeshipping: f32,
}

pub struct Shipping {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Shipping {

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        // Implementation to create a new instance of Shipping

        Shipping {
            pool,
        }
    }
}
//
// Last Moficication: 2024-07-22 19:01:57
//

pub struct Order {
    id: i32,
}

pub struct Orders {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Orders {

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        // Implementation to create a new instance of Orders

        Orders {
            pool,
        }
    }
}
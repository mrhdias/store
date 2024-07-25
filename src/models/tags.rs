//
// Last Modification: 2024-07-24 19:15:21
//

pub struct Tag {
    id: i32,
}

pub struct Tags {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Tags {

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        // Implementation to create a new instance of Tags

        Tags {
            pool,
        }
    }
}
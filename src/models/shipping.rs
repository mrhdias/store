//
// Last Modification: 2024-07-28 11:57:06
//

use std::collections::HashMap;
use regex::Regex;

struct TableWeight {
    label: String,
    postcode_regex: Vec<String>,
    prices: Vec<(u32, f32)>, // (Weight, Price)
    freeshipping: f32,
}

pub struct Shipping {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Shipping {

    pub fn calculate(&self,
        country: &str,
        postcode: &str,
        total_weight: &u32,
        total_order: &f32) -> Result<f32, anyhow::Error> {
        // Implementation to calculate shipping based on the selected shipping table
        // and the given country, postcode, total weight, and total order amount
        // and return the calculated shipping cost.

        let mut countries = HashMap::new();
        countries.insert("PT".to_string(), vec![
            TableWeight {
                label: "mainland".to_string(),
                postcode_regex: vec![r#"^[12345678]\d+"#.to_string()],
                prices: vec![
                    (1000, 4.90),
                    (2000, 8.30),
                    (3000, 12.70),
                    (4000, 17.10),
                    (5000, 21.50),
                ],
                freeshipping: 100.0,
            },
            TableWeight {
                label: "madeira".to_string(),
                postcode_regex: vec![r#"^9[01234]\d+"#.to_string()],
                prices: vec![
                    (1000, 5.90),
                    (2000, 9.30),
                    (3000, 13.70),
                    (4000, 18.10),
                    (5000, 22.50)
                ],
                freeshipping: 200.0,
            },
            TableWeight {
                label: "acores".to_string(),
                postcode_regex: vec![r#"^9[56789]\d+"#.to_string()],
                prices: vec![
                    (1000, 6.90),
                    (2000, 10.30),
                    (3000, 14.70),
                    (4000, 19.10),
                    (5000, 23.50)
                ],
                freeshipping: 300.0,
            },
        ]);
        countries.insert("ES".to_string(), vec![
            TableWeight {
                label: "mainland".to_string(),
                postcode_regex: vec![],
                prices: vec![
                    (1000, 7.90),
                    (2000, 11.30),
                    (3000, 15.70),
                    (4000, 20.10),
                    (5000, 24.50)
                ],
                freeshipping: 100.0,
            },
        ]);
    
        if countries.is_empty() {
            return Err(anyhow::anyhow!("shipping tables is empty"));
        }
    
        if !countries.contains_key(country) {
            return Err(anyhow::anyhow!("shipping table not found for country: {}", country));
        }
    
        let country_tables = countries.get(country).unwrap();
        for table in country_tables {
            for pattern in &table.postcode_regex {
                let regex = Regex::new(&pattern).unwrap();
                if regex.is_match(postcode) {
                    if total_order >= &table.freeshipping {
                        return Ok(0.00);
                    }
                    for (weight, price) in &table.prices {
                        if total_weight <= &weight {
                            return Ok(*price);
                        }
                    }
                    return Err(anyhow::anyhow!("price shipping not found for weight: {}/{}",
                        country, total_weight));
                }
            }
        }
    
        Err(anyhow::anyhow!("unable to calculate the shipping for country/postcode: {}/{}",
            country, postcode))
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        // Implementation to create a new instance of Shipping

        Shipping {
            pool,
        }
    }
}
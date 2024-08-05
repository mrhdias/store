//
// Last Modification: 2024-08-02 22:10:32
//

use tera::{Result, Value};
use std::collections::HashMap;

pub fn round_and_format_filter(value: &Value, params: &HashMap<String, Value>) -> Result<Value> {
    let num = value.as_f64().ok_or_else(|| tera::Error::msg("Filter can only be applied to numbers"))?;
    let decimal_places = params.get("places")
        .and_then(Value::as_u64)
        .ok_or_else(|| tera::Error::msg("Filter parameter 'places' is required and must be a positive integer"))?;

    Ok(Value::String(format!("{:.1$}", num, decimal_places as usize)))
}

pub fn round_to_two_decimal_places(value: &f32) -> f32 {
    (value * 100.00).round() / 100.0
}
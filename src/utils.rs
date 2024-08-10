//
// Last Modification: 2024-08-08 19:25:58
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



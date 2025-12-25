use serde_json::{Map, Value};

pub fn merge_json(base: &mut Value, update: &Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, update_value);
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
        }
        (base, update) => *base = update.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_json() {
        let mut base = json!({
            "name": "Alice",
            "details": {
                "age": 30,
                "city": "London"
            }
        });

        let update = json!({
            "details": {
                "city": "Paris",
                "country": "France"
            },
            "active": true
        });

        merge_json(&mut base, &update);

        assert_eq!(
            base,
            json!({
                "name": "Alice",
                "details": {
                    "age": 30,
                    "city": "Paris",
                    "country": "France"
                },
                "active": true
            })
        );
    }
}
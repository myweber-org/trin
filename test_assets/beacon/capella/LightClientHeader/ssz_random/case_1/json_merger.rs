use std::collections::HashMap;
use serde_json::{Value, Map};

pub fn merge_json(base: &mut Value, extension: &Value) {
    match (base, extension) {
        (Value::Object(base_map), Value::Object(ext_map)) => {
            for (key, ext_value) in ext_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    merge_json(base_value, ext_value);
                } else {
                    base_map.insert(key.clone(), ext_value.clone());
                }
            }
        }
        (base, extension) => {
            *base = extension.clone();
        }
    }
}

pub fn merge_json_with_strategy(
    base: &mut Value,
    extension: &Value,
    strategy: MergeStrategy,
) -> Result<(), String> {
    match strategy {
        MergeStrategy::Deep => {
            merge_json(base, extension);
            Ok(())
        }
        MergeStrategy::Shallow => {
            *base = extension.clone();
            Ok(())
        }
        MergeStrategy::Custom(merge_fn) => merge_fn(base, extension),
    }
}

pub enum MergeStrategy {
    Deep,
    Shallow,
    Custom(fn(&mut Value, &Value) -> Result<(), String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            }
        });

        let extension = json!({
            "b": {
                "d": 4,
                "e": 5
            },
            "f": 6
        });

        merge_json(&mut base, &extension);

        assert_eq!(
            base,
            json!({
                "a": 1,
                "b": {
                    "c": 2,
                    "d": 4,
                    "e": 5
                },
                "f": 6
            })
        );
    }

    #[test]
    fn test_shallow_merge() {
        let mut base = json!({"a": {"b": 1}});
        let extension = json!({"a": {"c": 2}});

        merge_json_with_strategy(&mut base, &extension, MergeStrategy::Shallow)
            .unwrap();

        assert_eq!(base, json!({"a": {"c": 2}}));
    }
}
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;

pub fn merge_json_files(file_paths: &[String], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();
    let mut seen_ids = HashSet::new();

    for file_path in file_paths {
        let content = fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content)?;

        match json_value {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        if !seen_ids.contains(id) {
                            seen_ids.insert(id.to_string());
                            merged_array.push(item);
                        }
                    } else {
                        merged_array.push(item);
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    if !seen_ids.contains(id) {
                        seen_ids.insert(id.to_string());
                        merged_array.push(json!(obj));
                    }
                } else {
                    merged_array.push(json!(obj));
                }
            }
            _ => return Err("Unsupported JSON structure".into()),
        }
    }

    let output_json = json!(merged_array);
    fs::write(output_path, output_json.to_string())?;
    Ok(())
}
use serde_json::{Map, Value};
use std::collections::HashSet;

pub enum MergeStrategy {
    PreferFirst,
    PreferSecond,
    CombineArrays,
    FailOnConflict,
}

pub fn merge_json(
    mut base: Value,
    overlay: Value,
    strategy: MergeStrategy,
) -> Result<Value, String> {
    if !base.is_object() || !overlay.is_object() {
        return Err("Both inputs must be JSON objects".to_string());
    }

    let base_obj = base.as_object_mut().unwrap();
    let overlay_obj = overlay.as_object().unwrap();

    for (key, overlay_value) in overlay_obj {
        match base_obj.get_mut(key) {
            Some(base_value) => {
                if base_value.is_object() && overlay_value.is_object() {
                    let merged = merge_json(
                        base_value.clone(),
                        overlay_value.clone(),
                        strategy.clone(),
                    )?;
                    *base_value = merged;
                } else if base_value.is_array() && overlay_value.is_array() {
                    if let MergeStrategy::CombineArrays = strategy {
                        let mut combined = base_value.as_array().unwrap().clone();
                        combined.extend(overlay_value.as_array().unwrap().iter().cloned());
                        *base_value = Value::Array(combined);
                    } else {
                        return handle_conflict(key, base_value, overlay_value, &strategy)?;
                    }
                } else if base_value != overlay_value {
                    return handle_conflict(key, base_value, overlay_value, &strategy)?;
                }
            }
            None => {
                base_obj.insert(key.clone(), overlay_value.clone());
            }
        }
    }

    Ok(Value::Object(base_obj.clone()))
}

fn handle_conflict(
    key: &str,
    base_value: &Value,
    overlay_value: &Value,
    strategy: &MergeStrategy,
) -> Result<Value, String> {
    match strategy {
        MergeStrategy::PreferFirst => Ok(base_value.clone()),
        MergeStrategy::PreferSecond => Ok(overlay_value.clone()),
        MergeStrategy::FailOnConflict => Err(format!(
            "Conflict at key '{}': {:?} vs {:?}",
            key, base_value, overlay_value
        )),
        MergeStrategy::CombineArrays => Err("Cannot combine non-array values".to_string()),
    }
}

pub fn find_common_keys(a: &Value, b: &Value) -> HashSet<String> {
    let mut common = HashSet::new();
    
    if let (Some(a_obj), Some(b_obj)) = (a.as_object(), b.as_object()) {
        for key in a_obj.keys() {
            if b_obj.contains_key(key) {
                common.insert(key.clone());
            }
        }
    }
    
    common
}
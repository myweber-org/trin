use serde_json::{Result, Value};
use std::fs;

fn parse_json_file(file_path: &str) -> Result<Value> {
    let data = fs::read_to_string(file_path)?;
    let json_value: Value = serde_json::from_str(&data)?;
    Ok(json_value)
}

fn validate_json_structure(value: &Value, expected_type: &str) -> bool {
    match expected_type {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn pretty_print_json(value: &Value, indent: usize) {
    match value {
        Value::Object(map) => {
            println!("{}", "{");
            for (key, val) in map {
                print!("{:width$}\"{}\": ", "", key, width = indent + 2);
                pretty_print_json(val, indent + 2);
                if key != map.keys().last().unwrap() {
                    println!(",");
                }
            }
            println!("\n{:width$}{}", "", "}", width = indent);
        }
        Value::Array(arr) => {
            println!("[");
            for (i, item) in arr.iter().enumerate() {
                print!("{:width$}", "", width = indent + 2);
                pretty_print_json(item, indent + 2);
                if i != arr.len() - 1 {
                    println!(",");
                }
            }
            println!("\n{:width$}]", "", width = indent);
        }
        Value::String(s) => print!("\"{}\"", s),
        Value::Number(n) => print!("{}", n),
        Value::Bool(b) => print!("{}", b),
        Value::Null => print!("null"),
    }
}

fn main() -> Result<()> {
    let file_path = "data.json";
    
    match parse_json_file(file_path) {
        Ok(json_value) => {
            println!("JSON parsed successfully.");
            
            if validate_json_structure(&json_value, "object") {
                println!("Root is a JSON object.");
            }
            
            println!("\nPretty printed JSON:");
            pretty_print_json(&json_value, 0);
            println!();
            
            if let Some(name) = json_value.get("name").and_then(|v| v.as_str()) {
                println!("Extracted 'name' field: {}", name);
            }
        }
        Err(e) => eprintln!("Failed to parse JSON: {}", e),
    }
    
    Ok(())
}
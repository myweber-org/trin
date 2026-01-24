use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    hobbies: Vec<String>,
    metadata: HashMap<String, Value>,
}

fn parse_json_file(file_path: &str) -> Result<Person> {
    let data = fs::read_to_string(file_path).expect("Unable to read file");
    let person: Person = serde_json::from_str(&data)?;
    Ok(person)
}

fn validate_person(person: &Person) -> bool {
    !person.name.is_empty() && person.age > 0
}

fn pretty_print_person(person: &Person) -> String {
    serde_json::to_string_pretty(person).unwrap_or_else(|_| String::from("Invalid JSON"))
}

fn process_json(file_path: &str) -> Option<String> {
    match parse_json_file(file_path) {
        Ok(person) => {
            if validate_person(&person) {
                Some(pretty_print_person(&person))
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            None
        }
    }
}

fn main() {
    let file_path = "data.json";
    if let Some(result) = process_json(file_path) {
        println!("Processed JSON:\n{}", result);
    } else {
        println!("Failed to process JSON file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_json_parsing() {
        let json_data = r#"
        {
            "name": "Alice",
            "age": 30,
            "hobbies": ["reading", "hiking"],
            "metadata": {"city": "New York"}
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let result = process_json(temp_file.path().to_str().unwrap());
        assert!(result.is_some());
    }

    #[test]
    fn test_invalid_json() {
        let json_data = r#"{"name": "", "age": 0}"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let result = process_json(temp_file.path().to_str().unwrap());
        assert!(result.is_none());
    }
}
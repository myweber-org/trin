use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
    tags: Vec<String>,
    metadata: HashMap<String, Value>,
}

impl User {
    fn new(id: u64, name: &str, email: &str) -> Self {
        User {
            id,
            name: name.to_string(),
            email: email.to_string(),
            active: true,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    fn set_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
    }
}

fn parse_json_file(file_path: &str) -> Result<Vec<User>> {
    let content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", file_path));
    
    serde_json::from_str(&content)
}

fn validate_users(users: &[User]) -> bool {
    for user in users {
        if user.name.is_empty() || !user.email.contains('@') {
            return false;
        }
    }
    true
}

fn pretty_print_users(users: &[User]) {
    for user in users {
        println!("{:#?}", user);
        println!("---");
    }
}

fn create_sample_json() -> String {
    let mut user1 = User::new(1, "Alice", "alice@example.com");
    user1.add_tag("admin");
    user1.add_tag("rust");
    user1.set_metadata("department", Value::String("Engineering".to_string()));
    
    let mut user2 = User::new(2, "Bob", "bob@example.com");
    user2.add_tag("user");
    user2.set_metadata("age", Value::Number(30.into()));
    
    let users = vec![user1, user2];
    
    serde_json::to_string_pretty(&users).unwrap()
}

fn main() {
    let sample_json = create_sample_json();
    println!("Generated JSON:\n{}\n", sample_json);
    
    match parse_json_file("users.json") {
        Ok(users) => {
            if validate_users(&users) {
                println!("All users are valid!");
                pretty_print_users(&users);
            } else {
                println!("Some users failed validation");
            }
        }
        Err(e) => {
            println!("Failed to parse JSON: {}", e);
            println!("Creating sample file...");
            
            fs::write("users.json", &sample_json)
                .expect("Failed to write sample file");
            
            println!("Sample file created. Please run again.");
        }
    }
}
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
    email: String,
    active: bool,
}

fn parse_json_file(file_path: &str) -> Result<Vec<User>> {
    let data = fs::read_to_string(file_path)?;
    let users: Vec<User> = serde_json::from_str(&data)?;
    Ok(users)
}

fn validate_user(user: &User) -> bool {
    !user.name.is_empty() && user.age > 0 && user.email.contains('@')
}

fn pretty_print_users(users: &[User]) {
    for (index, user) in users.iter().enumerate() {
        println!("User #{}:", index + 1);
        println!("  Name: {}", user.name);
        println!("  Age: {}", user.age);
        println!("  Email: {}", user.email);
        println!("  Active: {}", user.active);
        println!("  Valid: {}", validate_user(user));
        println!();
    }
}

fn filter_active_users(users: &[User]) -> Vec<&User> {
    users.iter().filter(|user| user.active).collect()
}

fn calculate_average_age(users: &[User]) -> f64 {
    if users.is_empty() {
        return 0.0;
    }
    let total_age: u32 = users.iter().map(|user| user.age).sum();
    total_age as f64 / users.len() as f64
}

fn main() -> Result<()> {
    let file_path = "users.json";
    
    match parse_json_file(file_path) {
        Ok(users) => {
            println!("Successfully parsed {} users", users.len());
            pretty_print_users(&users);
            
            let active_users = filter_active_users(&users);
            println!("Active users: {}", active_users.len());
            
            let avg_age = calculate_average_age(&users);
            println!("Average age: {:.2}", avg_age);
            
            let valid_users = users.iter().filter(|u| validate_user(u)).count();
            println!("Valid users: {}", valid_users);
        }
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
        }
    }
    
    Ok(())
}
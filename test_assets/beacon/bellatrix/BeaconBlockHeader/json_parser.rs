use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
}

pub fn parse_user_json(json_str: &str) -> Result<User> {
    let user: User = serde_json::from_str(json_str)?;
    Ok(user)
}

pub fn serialize_user(user: &User) -> Result<String> {
    let json = serde_json::to_string_pretty(user)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parsing() {
        let json_data = r#"
        {
            "id": 42,
            "username": "rustacean",
            "email": "user@example.com",
            "active": true
        }"#;

        match parse_user_json(json_data) {
            Ok(user) => {
                assert_eq!(user.id, 42);
                assert_eq!(user.username, "rustacean");
                assert_eq!(user.email, "user@example.com");
                assert!(user.active);
                
                let serialized = serialize_user(&user).unwrap();
                assert!(serialized.contains("\"username\": \"rustacean\""));
            }
            Err(e) => panic!("Failed to parse JSON: {}", e),
        }
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{ invalid: json }"#;
        assert!(parse_user_json(invalid_json).is_err());
    }
}

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    id: u32,
    name: String,
    email: String,
    age: u8,
}

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid age: {0}")]
    InvalidAge(u8),
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
    #[error("Empty name field")]
    EmptyName,
}

impl UserData {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if self.age > 120 {
            return Err(DataError::InvalidAge(self.age));
        }
        
        if !self.email.contains('@') || !self.email.contains('.') {
            return Err(DataError::InvalidEmail(self.email.clone()));
        }
        
        Ok(())
    }
    
    pub fn transform_to_uppercase(&mut self) {
        self.name = self.name.to_uppercase();
        self.email = self.email.to_uppercase();
    }
    
    pub fn is_adult(&self) -> bool {
        self.age >= 18
    }
}

pub fn process_user_data(users: &mut [UserData]) -> Vec<Result<(), DataError>> {
    users.iter_mut()
        .map(|user| {
            user.transform_to_uppercase();
            user.validate()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_user() {
        let user = UserData {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: 25,
        };
        
        assert!(user.validate().is_ok());
        assert!(user.is_adult());
    }
    
    #[test]
    fn test_invalid_email() {
        let user = UserData {
            id: 2,
            name: "Jane Smith".to_string(),
            email: "invalid-email".to_string(),
            age: 30,
        };
        
        assert!(matches!(user.validate(), Err(DataError::InvalidEmail(_))));
    }
    
    #[test]
    fn test_uppercase_transformation() {
        let mut user = UserData {
            id: 3,
            name: "alice".to_string(),
            email: "alice@test.org".to_string(),
            age: 22,
        };
        
        user.transform_to_uppercase();
        assert_eq!(user.name, "ALICE");
        assert_eq!(user.email, "ALICE@TEST.ORG");
    }
}
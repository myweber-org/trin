
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyField,
    InvalidLength(usize, usize),
    InvalidFormat(String),
    OutOfRange(f64, f64, f64),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyField => write!(f, "Field cannot be empty"),
            ValidationError::InvalidLength(min, max) => write!(
                f, "Length must be between {} and {} characters", min, max
            ),
            ValidationError::InvalidFormat(expected) => write!(
                f, "Invalid format, expected: {}", expected
            ),
            ValidationError::OutOfRange(value, min, max) => write!(
                f, "Value {} is out of range [{}, {}]", value, min, max
            ),
        }
    }
}

impl Error for ValidationError {}

pub struct DataValidator;

impl DataValidator {
    pub fn validate_string(
        input: &str,
        min_len: usize,
        max_len: usize,
    ) -> Result<(), ValidationError> {
        if input.trim().is_empty() {
            return Err(ValidationError::EmptyField);
        }
        
        let length = input.chars().count();
        if length < min_len || length > max_len {
            return Err(ValidationError::InvalidLength(min_len, max_len));
        }
        
        Ok(())
    }
    
    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        Self::validate_string(email, 3, 254)?;
        
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::InvalidFormat(
                "valid email address with @ symbol".to_string()
            ));
        }
        
        if !parts[1].contains('.') {
            return Err(ValidationError::InvalidFormat(
                "domain with dot separator".to_string()
            ));
        }
        
        Ok(())
    }
    
    pub fn validate_numeric_range(
        value: f64,
        min: f64,
        max: f64,
    ) -> Result<(), ValidationError> {
        if value < min || value > max {
            return Err(ValidationError::OutOfRange(value, min, max));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_string() {
        assert!(DataValidator::validate_string("test", 1, 10).is_ok());
        assert!(DataValidator::validate_string("", 1, 10).is_err());
        assert!(DataValidator::validate_string("toolongstring", 1, 5).is_err());
    }
    
    #[test]
    fn test_validate_email() {
        assert!(DataValidator::validate_email("user@example.com").is_ok());
        assert!(DataValidator::validate_email("invalid").is_err());
        assert!(DataValidator::validate_email("user@com").is_err());
    }
    
    #[test]
    fn test_validate_numeric_range() {
        assert!(DataValidator::validate_numeric_range(5.0, 0.0, 10.0).is_ok());
        assert!(DataValidator::validate_numeric_range(15.0, 0.0, 10.0).is_err());
    }
}
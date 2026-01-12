
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    EmptyField,
    InvalidFormat,
    OutOfRange,
    Custom(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyField => write!(f, "Field cannot be empty"),
            ValidationError::InvalidFormat => write!(f, "Invalid format"),
            ValidationError::OutOfRange => write!(f, "Value out of acceptable range"),
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ValidationError {}

pub struct Validator;

impl Validator {
    pub fn validate_non_empty(input: &str) -> Result<(), ValidationError> {
        if input.trim().is_empty() {
            Err(ValidationError::EmptyField)
        } else {
            Ok(())
        }
    }

    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        Self::validate_non_empty(email)?;
        
        if !email.contains('@') || !email.contains('.') {
            return Err(ValidationError::InvalidFormat);
        }
        
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::InvalidFormat);
        }
        
        Ok(())
    }

    pub fn validate_numeric_range(
        value: i32,
        min: i32,
        max: i32,
    ) -> Result<(), ValidationError> {
        if value < min || value > max {
            Err(ValidationError::OutOfRange)
        } else {
            Ok(())
        }
    }

    pub fn validate_with_custom<T, F>(
        value: &T,
        predicate: F,
        error_message: &str,
    ) -> Result<(), ValidationError>
    where
        F: Fn(&T) -> bool,
    {
        if predicate(value) {
            Ok(())
        } else {
            Err(ValidationError::Custom(error_message.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert!(Validator::validate_non_empty("test").is_ok());
        assert!(Validator::validate_non_empty("").is_err());
        assert!(Validator::validate_non_empty("   ").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(Validator::validate_email("user@example.com").is_ok());
        assert!(Validator::validate_email("invalid").is_err());
        assert!(Validator::validate_email("@example.com").is_err());
        assert!(Validator::validate_email("user@").is_err());
    }

    #[test]
    fn test_validate_numeric_range() {
        assert!(Validator::validate_numeric_range(5, 1, 10).is_ok());
        assert!(Validator::validate_numeric_range(0, 1, 10).is_err());
        assert!(Validator::validate_numeric_range(11, 1, 10).is_err());
    }

    #[test]
    fn test_validate_with_custom() {
        let is_even = |x: &i32| x % 2 == 0;
        
        assert!(Validator::validate_with_custom(&4, is_even, "Must be even").is_ok());
        assert!(Validator::validate_with_custom(&5, is_even, "Must be even").is_err());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyString,
    NegativeNumber,
    OutOfRange { min: i32, max: i32, value: i32 },
    InvalidFormat(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyString => write!(f, "String cannot be empty"),
            ValidationError::NegativeNumber => write!(f, "Number cannot be negative"),
            ValidationError::OutOfRange { min, max, value } => {
                write!(f, "Value {} is outside range [{}, {}]", value, min, max)
            }
            ValidationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl Error for ValidationError {}

pub struct DataValidator;

impl DataValidator {
    pub fn validate_string(input: &str) -> Result<(), ValidationError> {
        if input.trim().is_empty() {
            return Err(ValidationError::EmptyString);
        }
        Ok(())
    }

    pub fn validate_positive_number(num: i32) -> Result<(), ValidationError> {
        if num < 0 {
            return Err(ValidationError::NegativeNumber);
        }
        Ok(())
    }

    pub fn validate_range(num: i32, min: i32, max: i32) -> Result<(), ValidationError> {
        if num < min || num > max {
            return Err(ValidationError::OutOfRange { min, max, value: num });
        }
        Ok(())
    }

    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        if !email.contains('@') || !email.contains('.') {
            return Err(ValidationError::InvalidFormat(
                "Email must contain '@' and '.'".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string() {
        assert!(DataValidator::validate_string("hello").is_ok());
        assert_eq!(
            DataValidator::validate_string(""),
            Err(ValidationError::EmptyString)
        );
        assert_eq!(
            DataValidator::validate_string("   "),
            Err(ValidationError::EmptyString)
        );
    }

    #[test]
    fn test_validate_positive_number() {
        assert!(DataValidator::validate_positive_number(42).is_ok());
        assert_eq!(
            DataValidator::validate_positive_number(-5),
            Err(ValidationError::NegativeNumber)
        );
    }

    #[test]
    fn test_validate_range() {
        assert!(DataValidator::validate_range(10, 0, 100).is_ok());
        assert_eq!(
            DataValidator::validate_range(-5, 0, 100),
            Err(ValidationError::OutOfRange {
                min: 0,
                max: 100,
                value: -5
            })
        );
    }

    #[test]
    fn test_validate_email() {
        assert!(DataValidator::validate_email("test@example.com").is_ok());
        assert!(DataValidator::validate_email("user@domain.co.uk").is_ok());
        assert_eq!(
            DataValidator::validate_email("invalid-email"),
            Err(ValidationError::InvalidFormat(
                "Email must contain '@' and '.'".to_string()
            ))
        );
    }
}
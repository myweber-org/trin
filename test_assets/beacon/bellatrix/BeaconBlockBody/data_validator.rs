
use regex::Regex;
use std::error::Error;

#[derive(Debug)]
pub enum ValidationError {
    InvalidEmail,
    InvalidPhone,
    InvalidLength { min: usize, max: usize },
    ContainsInvalidChars,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidEmail => write!(f, "Invalid email format"),
            ValidationError::InvalidPhone => write!(f, "Invalid phone number format"),
            ValidationError::InvalidLength { min, max } => {
                write!(f, "Length must be between {} and {} characters", min, max)
            }
            ValidationError::ContainsInvalidChars => write!(f, "Contains invalid characters"),
        }
    }
}

impl Error for ValidationError {}

pub struct Validator {
    email_regex: Regex,
    phone_regex: Regex,
    sanitization_regex: Regex,
}

impl Validator {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Validator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?,
            phone_regex: Regex::new(r"^\+?[1-9]\d{1,14}$")?,
            sanitization_regex: Regex::new(r"[<>\"'&;]")?,
        })
    }

    pub fn validate_email(&self, email: &str) -> Result<(), ValidationError> {
        if !self.email_regex.is_match(email) {
            return Err(ValidationError::InvalidEmail);
        }
        Ok(())
    }

    pub fn validate_phone(&self, phone: &str) -> Result<(), ValidationError> {
        if !self.phone_regex.is_match(phone) {
            return Err(ValidationError::InvalidPhone);
        }
        Ok(())
    }

    pub fn validate_length(
        &self,
        input: &str,
        min: usize,
        max: usize,
    ) -> Result<(), ValidationError> {
        let len = input.len();
        if len < min || len > max {
            return Err(ValidationError::InvalidLength { min, max });
        }
        Ok(())
    }

    pub fn sanitize_input(&self, input: &str) -> Result<String, ValidationError> {
        if self.sanitization_regex.is_match(input) {
            return Err(ValidationError::ContainsInvalidChars);
        }
        Ok(input.to_string())
    }

    pub fn validate_user_input(
        &self,
        email: &str,
        phone: &str,
        username: &str,
    ) -> Result<(), ValidationError> {
        self.validate_email(email)?;
        self.validate_phone(phone)?;
        self.validate_length(username, 3, 30)?;
        self.sanitize_input(username)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_email("test@example.com").is_ok());
        assert!(validator.validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_valid_phone() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_phone("+1234567890").is_ok());
        assert!(validator.validate_phone("invalid").is_err());
    }

    #[test]
    fn test_sanitization() {
        let validator = Validator::new().unwrap();
        assert!(validator.sanitize_input("safe input").is_ok());
        assert!(validator.sanitize_input("<script>alert('xss')</script>").is_err());
    }
}
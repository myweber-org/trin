use regex::Regex;

pub struct Validator {
    email_regex: Regex,
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }

    pub fn validate_password_strength(&self, password: &str) -> bool {
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*".contains(c));
        let length_valid = password.len() >= 8;

        has_uppercase && has_lowercase && has_digit && has_special && length_valid
    }

    pub fn validate_credentials(&self, email: &str, password: &str) -> (bool, Vec<String>) {
        let mut errors = Vec::new();

        if !self.validate_email(email) {
            errors.push("Invalid email format".to_string());
        }

        if !self.validate_password_strength(password) {
            errors.push("Password does not meet strength requirements".to_string());
        }

        let is_valid = errors.is_empty();
        (is_valid, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let validator = Validator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(validator.validate_email("user.name@domain.co.uk"));
        assert!(!validator.validate_email("invalid-email"));
        assert!(!validator.validate_email("missing@tld."));
    }

    #[test]
    fn test_password_strength() {
        let validator = Validator::new();
        assert!(validator.validate_password_strength("StrongP@ss1"));
        assert!(!validator.validate_password_strength("weak"));
        assert!(!validator.validate_password_strength("NoSpecial1"));
        assert!(!validator.validate_password_strength("NOLOWER1@"));
    }

    #[test]
    fn test_credential_validation() {
        let validator = Validator::new();
        let (valid, errors) = validator.validate_credentials("test@example.com", "StrongP@ss1");
        assert!(valid);
        assert!(errors.is_empty());

        let (invalid, errors) = validator.validate_credentials("invalid", "weak");
        assert!(!invalid);
        assert_eq!(errors.len(), 2);
    }
}
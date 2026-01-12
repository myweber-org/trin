
use regex::Regex;
use std::collections::HashSet;

pub struct Validator {
    email_regex: Regex,
    forbidden_usernames: HashSet<String>,
}

impl Validator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let mut forbidden = HashSet::new();
        forbidden.insert("admin".to_string());
        forbidden.insert("root".to_string());
        forbidden.insert("system".to_string());

        Validator {
            email_regex: Regex::new(email_pattern).unwrap(),
            forbidden_usernames: forbidden,
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }

    pub fn validate_username(&self, username: &str) -> bool {
        let length_ok = username.len() >= 3 && username.len() <= 20;
        let alphanumeric = username.chars().all(|c| c.is_alphanumeric() || c == '_');
        let not_forbidden = !self.forbidden_usernames.contains(username);

        length_ok && alphanumeric && not_forbidden
    }

    pub fn validate_password_strength(&self, password: &str) -> bool {
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| !c.is_alphanumeric());
        let length_ok = password.len() >= 8;

        has_upper && has_lower && has_digit && has_special && length_ok
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new();
        assert!(validator.validate_email("user@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_username_validation() {
        let validator = Validator::new();
        assert!(validator.validate_username("valid_user123"));
        assert!(!validator.validate_username("admin"));
        assert!(!validator.validate_username("ab"));
    }

    #[test]
    fn test_password_strength() {
        let validator = Validator::new();
        assert!(validator.validate_password_strength("StrongPass123!"));
        assert!(!validator.validate_password_strength("weak"));
    }

    #[test]
    fn test_input_sanitization() {
        let validator = Validator::new();
        let sanitized = validator.sanitize_input("Hello<script>alert('xss')</script>World!");
        assert_eq!(sanitized, "HelloalertxssWorld");
    }
}
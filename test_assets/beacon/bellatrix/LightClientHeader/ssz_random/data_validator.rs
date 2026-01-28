
use regex::Regex;
use std::collections::HashSet;

pub struct InputValidator {
    email_regex: Regex,
    forbidden_words: HashSet<String>,
}

impl InputValidator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let forbidden = vec![
            "malicious".to_string(),
            "injection".to_string(),
            "script".to_string(),
        ]
        .into_iter()
        .collect();

        InputValidator {
            email_regex: Regex::new(email_pattern).unwrap(),
            forbidden_words: forbidden,
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email.trim())
    }

    pub fn sanitize_text(&self, text: &str) -> String {
        let mut sanitized = text.to_string();
        for word in &self.forbidden_words {
            let replacement = "*".repeat(word.len());
            sanitized = sanitized.replace(word, &replacement);
        }
        sanitized
    }

    pub fn check_password_strength(&self, password: &str) -> u8 {
        let mut score = 0;
        if password.len() >= 8 {
            score += 1;
        }
        if password.chars().any(|c| c.is_ascii_uppercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_ascii_lowercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_ascii_digit()) {
            score += 1;
        }
        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 1;
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = InputValidator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_text_sanitization() {
        let validator = InputValidator::new();
        let input = "This is a malicious script injection attempt";
        let output = validator.sanitize_text(input);
        assert!(!output.contains("malicious"));
        assert!(!output.contains("injection"));
    }

    #[test]
    fn test_password_strength() {
        let validator = InputValidator::new();
        assert_eq!(validator.check_password_strength("weak"), 1);
        assert_eq!(validator.check_password_strength("StrongPass123!"), 5);
    }
}

use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_phone(phone: &str) -> bool {
    let digits_only: String = phone.chars().filter(|c| c.is_digit(10)).collect();
    digits_only.len() >= 10 && digits_only.len() <= 15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("john.doe@company.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("user@.com"));
    }

    #[test]
    fn test_valid_phones() {
        assert!(is_valid_phone("+1 (555) 123-4567"));
        assert!(is_valid_phone("5551234567"));
        assert!(!is_valid_phone("123"));
        assert!(!is_valid_phone("555-1234"));
    }
}
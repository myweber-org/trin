use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"^https?://(?:[-\w]+\.)+[-\w]+(?:/[-\w\./?%&=]*)?$"
        ).expect("Invalid regex pattern");
        
        UrlValidator { pattern }
    }

    pub fn validate(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new();
        assert!(validator.validate("https://example.com"));
        assert!(validator.validate("http://sub.domain.co.uk/path"));
        assert!(validator.validate("https://api.service.io/v1/resource?id=123"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        assert!(!validator.validate("not-a-url"));
        assert!(!validator.validate("ftp://invalid.protocol"));
        assert!(!validator.validate("https://"));
    }
}
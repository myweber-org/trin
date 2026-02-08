use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let url_pattern = Regex::new(
        r"^(https?|ftp)://[^\s/$.?#].[^\s]*$"
    ).unwrap();
    
    url_pattern.is_match(url)
}

pub fn extract_domain(url: &str) -> Option<String> {
    let domain_pattern = Regex::new(
        r"^(?:https?://)?([^/:]+)"
    ).unwrap();
    
    domain_pattern.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://www.rust-lang.org"));
        assert!(is_valid_url("ftp://files.example.com/data.txt"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("http://"));
        assert!(!is_valid_url("example.com"));
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(
            extract_domain("https://github.com/rust-lang/rust"),
            Some("github.com".to_string())
        );
        assert_eq!(
            extract_domain("http://localhost:8080/api"),
            Some("localhost".to_string())
        );
        assert_eq!(extract_domain("invalid-url"), None);
    }
}use regex::Regex;
use std::error::Error;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let pattern = Regex::new(r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)$")?;
        Ok(UrlValidator { pattern })
    }

    pub fn is_valid(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if !self.is_valid(url) {
            return None;
        }
        
        let domain_start = url.find("://").map(|i| i + 3).unwrap_or(0);
        let domain_end = url[domain_start..]
            .find('/')
            .map(|i| domain_start + i)
            .unwrap_or(url.len());
        
        let domain = &url[domain_start..domain_end];
        
        if domain.starts_with("www.") {
            Some(domain[4..].to_string())
        } else {
            Some(domain.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(validator.is_valid("https://example.com"));
        assert!(validator.is_valid("http://sub.example.com/path"));
        assert!(validator.is_valid("https://www.example.co.uk/search?q=test"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(!validator.is_valid("not-a-url"));
        assert!(!validator.is_valid("ftp://example.com"));
        assert!(!validator.is_valid("https://"));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new().unwrap();
        assert_eq!(validator.extract_domain("https://example.com"), Some("example.com".to_string()));
        assert_eq!(validator.extract_domain("http://www.google.com/search"), Some("google.com".to_string()));
        assert_eq!(validator.extract_domain("invalid-url"), None);
    }
}
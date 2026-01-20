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
}
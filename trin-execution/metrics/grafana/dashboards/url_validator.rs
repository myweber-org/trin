use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let url_pattern = Regex::new(
        r"^https?://([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(:\d+)?(/[^\s]*)?$"
    ).unwrap();
    
    url_pattern.is_match(url)
}

pub fn extract_domain(url: &str) -> Option<String> {
    if !is_valid_url(url) {
        return None;
    }
    
    let domain_pattern = Regex::new(r"^https?://([^/:]+)").unwrap();
    domain_pattern.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://sub.example.co.uk/path"));
        assert!(is_valid_url("https://api.example.com:8080/endpoint"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("https://"));
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(extract_domain("https://example.com"), Some("example.com".to_string()));
        assert_eq!(extract_domain("http://sub.example.co.uk/path"), Some("sub.example.co.uk".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }
}
use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let url_pattern = Regex::new(
        r"^https?://(?:[-\w]+\.)+[-\w]{2,}(?::\d+)?(?:/[-\w\.%]*)*(?:\?[-\w=&%\.]*)?(?:#[-\w]*)?$"
    ).unwrap();
    
    url_pattern.is_match(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://www.example.com/path"));
        assert!(is_valid_url("http://sub.domain.co.uk:8080/page?query=value"));
        assert!(is_valid_url("https://api.example.com/v1/resource#section"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("http://"));
        assert!(!is_valid_url("https://example."));
    }
}
use regex::Regex;

pub fn extract_domain(url: &str) -> Option<String> {
    let re = Regex::new(r"^(?:https?://)?(?:www\.)?([^/]+)").unwrap();
    re.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.example.com/path"), Some("example.com".to_string()));
        assert_eq!(extract_domain("http://example.com"), Some("example.com".to_string()));
        assert_eq!(extract_domain("www.example.com"), Some("example.com".to_string()));
        assert_eq!(extract_domain("example.com"), Some("example.com".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }
}
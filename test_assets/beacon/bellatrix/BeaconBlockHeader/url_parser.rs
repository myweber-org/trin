use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser {
    url: String,
}

impl UrlParser {
    pub fn new(url: &str) -> Self {
        UrlParser {
            url: url.to_string(),
        }
    }

    pub fn get_domain(&self) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/]+)").unwrap();
        re.captures(&self.url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn get_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let re = Regex::new(r"[?&]([^=]+)=([^&]+)").unwrap();

        for cap in re.captures_iter(&self.url) {
            if let (Some(key), Some(value)) = (cap.get(1), cap.get(2)) {
                params.insert(key.as_str().to_string(), value.as_str().to_string());
            }
        }
        params
    }

    pub fn is_valid_url(&self) -> bool {
        let re = Regex::new(r"^(https?://)?([\w-]+\.)+[\w-]+(/[^?#]*)?(\?[^#]*)?(#.*)?$").unwrap();
        re.is_match(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://www.example.com/path?query=test");
        assert_eq!(parser.get_domain(), Some("www.example.com".to_string()));
    }

    #[test]
    fn test_query_params() {
        let parser = UrlParser::new("https://example.com?name=john&age=30");
        let params = parser.get_query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_validation() {
        let valid_parser = UrlParser::new("https://example.com");
        assert!(valid_parser.is_valid_url());

        let invalid_parser = UrlParser::new("not-a-valid-url");
        assert!(!invalid_parser.is_valid_url());
    }
}
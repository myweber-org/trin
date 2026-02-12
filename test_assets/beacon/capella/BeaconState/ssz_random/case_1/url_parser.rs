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

    pub fn extract_domain(&self) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/?#]+)").unwrap();
        re.captures(&self.url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = self.url.find('?');

        if let Some(start) = query_start {
            let query_string = &self.url[start + 1..];
            for pair in query_string.split('&') {
                let mut kv = pair.split('=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        params
    }

    pub fn is_valid_url(&self) -> bool {
        let url_pattern = Regex::new(
            r"^(https?://)?([\w\-]+\.)+[\w\-]+(/[\w\-./?%&=]*)?$"
        ).unwrap();
        url_pattern.is_match(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://example.com/path?query=1");
        assert_eq!(parser.extract_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_query_parsing() {
        let parser = UrlParser::new("https://test.com?name=john&age=25");
        let params = parser.parse_query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"25".to_string()));
    }

    #[test]
    fn test_url_validation() {
        let valid = UrlParser::new("https://valid-domain.com");
        let invalid = UrlParser::new("not-a-valid-url");
        
        assert!(valid.is_valid_url());
        assert!(!invalid.is_valid_url());
    }
}
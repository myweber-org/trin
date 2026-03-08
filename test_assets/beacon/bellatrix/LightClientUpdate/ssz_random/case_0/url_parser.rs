
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut key_value = pair.split('=');
                if let (Some(key), Some(value)) = (key_value.next(), key_value.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") {
            let without_protocol = &url_lower[7..];
            return without_protocol.split('/').next().map(|s| s.to_string());
        }
        
        if url_lower.starts_with("https://") {
            let without_protocol = &url_lower[8..];
            return without_protocol.split('/').next().map(|s| s.to_string());
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&sort=date";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"date".to_string()));
        assert_eq!(params.get("nonexistent"), None);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://api.test.com:8080/resource"),
            Some("api.test.com:8080".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            None
        );
    }
    
    #[test]
    fn test_empty_query_string() {
        let url = "https://example.com/page";
        let params = UrlParser::parse_query_params(url);
        assert!(params.is_empty());
    }
}
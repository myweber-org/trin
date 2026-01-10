
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for param_pair in query_string.split('&') {
                let mut parts = param_pair.split('=');
                if let Some(key) = parts.next() {
                    if let Some(value) = parts.next() {
                        params.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") {
            Some(url[7..].split('/').next().unwrap_or("").to_string())
        } else if url_lower.starts_with("https://") {
            Some(url[8..].split('/').next().unwrap_or("").to_string())
        } else {
            url.split('/').next().map(|s| s.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&page=1";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
        assert_eq!(params.get("missing"), None);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk/api"),
            Some("sub.domain.co.uk".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("example.com/resource"),
            Some("example.com".to_string())
        );
    }
}
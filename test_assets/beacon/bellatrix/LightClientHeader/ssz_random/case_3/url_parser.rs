
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> Option<HashMap<String, String>> {
        let query_start = url.find('?')?;
        let query_str = &url[query_start + 1..];
        
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        Some(params)
    }
    
    pub fn extract_domain(url: &str) -> Option<&str> {
        let after_protocol = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            url
        };
        
        after_protocol.split('/').next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&page=2&sort=recent";
        let params = UrlParser::parse_query_string(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"recent".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        assert_eq!(UrlParser::extract_domain("https://github.com/rust-lang"), Some("github.com"));
        assert_eq!(UrlParser::extract_domain("http://localhost:8080/api"), Some("localhost:8080"));
        assert_eq!(UrlParser::extract_domain("example.com/path"), Some("example.com"));
    }
}
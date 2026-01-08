
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
                if !key.is_empty() {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        Some(params)
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let after_protocol = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            url
        };
        
        let domain_end = after_protocol.find('/').unwrap_or(after_protocol.len());
        let domain = &after_protocol[..domain_end];
        
        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_string() {
        let url = "https://example.com/search?q=rust&lang=en&sort=date";
        let params = UrlParser::parse_query_string(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"date".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://github.com/rust-lang/rust"),
            Some("github.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://localhost:8080/api"),
            Some("localhost:8080".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            None
        );
    }
    
    #[test]
    fn test_empty_query() {
        let url = "https://example.com/page";
        assert!(UrlParser::parse_query_string(url).is_none());
    }
}
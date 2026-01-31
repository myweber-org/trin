
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for param_pair in query_string.split('&') {
                let parts: Vec<&str> = param_pair.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    params.insert(key, value);
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let prefixes = ["http://", "https://", "www."];
        let mut processed_url = url;
        
        for prefix in &prefixes {
            if url.starts_with(prefix) {
                processed_url = &url[prefix.len()..];
                break;
            }
        }
        
        if let Some(slash_pos) = processed_url.find('/') {
            Some(processed_url[..slash_pos].to_string())
        } else {
            Some(processed_url.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        let urls = vec![
            "https://www.example.com/path",
            "http://example.com/page",
            "www.test.org/resource",
            "simple.com",
        ];
        
        let expected = vec![
            "example.com",
            "example.com",
            "test.org",
            "simple.com",
        ];
        
        for (url, expected_domain) in urls.iter().zip(expected.iter()) {
            assert_eq!(UrlParser::extract_domain(url), Some(expected_domain.to_string()));
        }
    }
}
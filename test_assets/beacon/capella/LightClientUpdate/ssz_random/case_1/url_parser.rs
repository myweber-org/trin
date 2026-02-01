use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0].to_string();
                let value = parts[1].to_string();
                params.insert(key, value);
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") {
            let without_protocol = &url_lower[7..];
            return Some(without_protocol.split('/').next().unwrap_or("").to_string());
        }
        
        if url_lower.starts_with("https://") {
            let without_protocol = &url_lower[8..];
            return Some(without_protocol.split('/').next().unwrap_or("").to_string());
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=newyork";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://api.test.net/resource"),
            Some("api.test.net".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("ftp://invalid.com"),
            None
        );
    }
}
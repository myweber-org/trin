
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(
                    key.to_string(),
                    urlencoding::decode(value)
                        .unwrap_or_else(|_| value.into())
                        .to_string(),
                );
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url = url.trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("//");
        
        let domain_end = url.find('/').unwrap_or(url.len());
        let domain = &url[..domain_end];
        
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
        let query = "name=John%20Doe&age=30&city=New%20York";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            Some("invalid-url".to_string())
        );
    }
}
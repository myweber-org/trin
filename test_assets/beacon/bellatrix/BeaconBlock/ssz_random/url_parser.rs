use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut domain_start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                domain_start = prefix.len();
                break;
            }
        }

        let remaining = &url[domain_start..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        let domain = &remaining[..domain_end];

        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    params.insert(key, value);
                }
            }
        }
        
        params
    }

    pub fn extract_path(url: &str) -> Option<String> {
        if let Some(domain_end) = url.find('/') {
            if let Some(query_start) = url[domain_end..].find('?') {
                Some(url[domain_end..domain_end + query_start].to_string())
            } else {
                Some(url[domain_end..].to_string())
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(UrlParser::parse_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("http://www.test.org"), Some("test.org".to_string()));
        assert_eq!(UrlParser::parse_domain("invalid"), Some("invalid".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

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
    fn test_extract_path() {
        assert_eq!(UrlParser::extract_path("https://example.com/api/users"), Some("/api/users".to_string()));
        assert_eq!(UrlParser::extract_path("https://example.com/search?q=test"), Some("/search".to_string()));
        assert_eq!(UrlParser::extract_path("example.com"), None);
    }
}
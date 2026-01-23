use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let after_protocol = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
            if let Some(slash_pos) = after_protocol.find('/') {
                return Some(after_protocol[..slash_pos].to_string());
            }
            return Some(after_protocol.to_string());
        }
        
        None
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
    }
    
    #[test]
    fn test_extract_domain() {
        let url = "https://www.example.com/path/to/resource";
        let domain = UrlParser::extract_domain(url);
        
        assert_eq!(domain, Some("www.example.com".to_string()));
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut processed_url = url_lower.as_str();
        for prefix in prefixes.iter() {
            if processed_url.starts_with(prefix) {
                processed_url = &processed_url[prefix.len()..];
            }
        }

        let domain_end = processed_url.find('/').unwrap_or(processed_url.len());
        let domain = &processed_url[..domain_end];
        
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
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let scheme_end = if url.starts_with("http://") {
            Some(7)
        } else if url.starts_with("https://") {
            Some(8)
        } else {
            None
        };

        let url_without_scheme = if let Some(end) = scheme_end {
            &url[end..]
        } else {
            url
        };

        let url_without_scheme = if url_without_scheme.starts_with("www.") {
            &url_without_scheme[4..]
        } else {
            url_without_scheme
        };

        if let Some(path_start) = url_without_scheme.find('/') {
            Some(url_without_scheme[path_start..].to_string())
        } else {
            Some("/".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(UrlParser::parse_domain("https://www.example.com/path"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("http://example.com"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("www.test.org"), Some("test.org".to_string()));
        assert_eq!(UrlParser::parse_domain("invalid"), Some("invalid".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com?name=john&age=30");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        
        let empty_params = UrlParser::parse_query_params("https://example.com");
        assert!(empty_params.is_empty());
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(UrlParser::extract_path("https://example.com/api/users"), Some("/api/users".to_string()));
        assert_eq!(UrlParser::extract_path("http://www.test.com/"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path("example.com"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path(""), None);
    }
}
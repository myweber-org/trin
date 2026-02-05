use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "www."];
        
        let mut processed_url = url_lower.as_str();
        for prefix in prefixes.iter() {
            if processed_url.starts_with(prefix) {
                processed_url = &processed_url[prefix.len()..];
                break;
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
                    let key = parts[0].trim().to_string();
                    let value = parts[1].trim().to_string();
                    if !key.is_empty() {
                        params.insert(key, value);
                    }
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

        let protocol_end = if let Some(pos) = url.find("://") {
            pos + 3
        } else {
            0
        };

        let url_without_protocol = &url[protocol_end..];
        let domain_end = url_without_protocol.find('/').unwrap_or(url_without_protocol.len());
        
        if domain_end < url_without_protocol.len() {
            Some(url_without_protocol[domain_end..].to_string())
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
        assert_eq!(UrlParser::parse_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("http://www.test.org"), Some("test.org".to_string()));
        assert_eq!(UrlParser::parse_domain("ftp://files.server.net/dir"), Some("files.server.net".to_string()));
        assert_eq!(UrlParser::parse_domain("invalid-url"), Some("invalid-url".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com?name=john&age=30&city=new+york");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
        
        let empty_params = UrlParser::parse_query_params("https://example.com");
        assert!(empty_params.is_empty());
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(UrlParser::extract_path("https://example.com/api/users"), Some("/api/users".to_string()));
        assert_eq!(UrlParser::extract_path("http://test.com/"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path("example.com"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path(""), None);
    }
}
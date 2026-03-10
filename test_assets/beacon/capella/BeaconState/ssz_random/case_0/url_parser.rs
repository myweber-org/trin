use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start_index = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start_index = prefix.len();
                break;
            }
        }

        let remaining = &url[start_index..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        let domain_part = &remaining[..domain_end];

        if domain_part.is_empty() {
            None
        } else {
            Some(domain_part.to_string())
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

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start_index = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start_index = prefix.len();
                break;
            }
        }

        let remaining = &url[start_index..];
        if let Some(slash_pos) = remaining.find('/') {
            let path_with_query = &remaining[slash_pos..];
            if let Some(query_pos) = path_with_query.find('?') {
                Some(path_with_query[..query_pos].to_string())
            } else {
                Some(path_with_query.to_string())
            }
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
        assert_eq!(
            UrlParser::parse_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://sub.domain.co.uk:8080/page"),
            Some("sub.domain.co.uk:8080".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params(
            "https://example.com/search?q=rust&lang=en&sort=date"
        );
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"date".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/page?param=value"),
            Some("/page".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com"),
            Some("/".to_string())
        );
    }
}
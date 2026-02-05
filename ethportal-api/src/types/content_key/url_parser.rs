use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<host>[^/]+)(?P<path>/.*)?$").unwrap();
    let captures = re.captures(url)?;

    let protocol = captures.name("protocol")?.as_str().to_string();
    let host = captures.name("host")?.as_str().to_string();
    let path = captures
        .name("path")
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());

    Some(ParsedUrl {
        protocol,
        host,
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let url = "http://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let url = "https://api.github.com";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "api.github.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        let parsed = parse_url(url);
        assert!(parsed.is_none());
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
                if let Some(equal_pos) = pair.find('=') {
                    let key = &pair[..equal_pos];
                    let value = &pair[equal_pos + 1..];
                    
                    if !key.is_empty() {
                        params.insert(key.to_string(), value.to_string());
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

        let scheme_pos = url.find("://").unwrap_or(0);
        let url_without_scheme = if scheme_pos > 0 {
            &url[scheme_pos + 3..]
        } else {
            url
        };

        let domain_end = url_without_scheme.find('/');
        match domain_end {
            Some(pos) => {
                let path = &url_without_scheme[pos..];
                if path.is_empty() {
                    Some("/".to_string())
                } else {
                    Some(path.to_string())
                }
            }
            None => Some("/".to_string()),
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
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://subdomain.example.co.uk"),
            Some("subdomain.example.co.uk".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params(
            "https://example.com/search?q=rust&lang=en&page=1"
        );
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("http://example.com"),
            Some("/".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("example.com/blog"),
            Some("/blog".to_string())
        );
    }
}
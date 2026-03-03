use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut domain = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        protocol = parts[0].to_string();
        let rest = parts[1];

        let domain_end = rest.find('/').unwrap_or(rest.len());
        let authority = &rest[..domain_end];
        let path_and_query = &rest[domain_end..];

        let authority_parts: Vec<&str> = authority.split(':').collect();
        domain = authority_parts[0].to_string();

        if authority_parts.len() > 1 {
            if let Ok(p) = authority_parts[1].parse::<u16>() {
                port = Some(p);
            } else {
                return Err("Invalid port number".to_string());
            }
        }

        let path_parts: Vec<&str> = path_and_query.split('?').collect();
        path = if !path_parts[0].is_empty() {
            path_parts[0].to_string()
        } else {
            "/".to_string()
        };

        if path_parts.len() > 1 {
            for param in path_parts[1].split('&') {
                let kv: Vec<&str> = param.split('=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            port,
            path,
            query_params,
        })
    }

    pub fn full_domain(&self) -> String {
        if let Some(p) = self.port {
            format!("{}:{}", self.domain, p)
        } else {
            self.domain.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::parse(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = "http://localhost:8080/api/data";
        let parsed = ParsedUrl::parse(url).unwrap();
        
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com/search?q=rust&limit=10&sort=desc";
        let parsed = ParsedUrl::parse(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"10".to_string()));
        assert_eq!(parsed.query_params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
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
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start = prefix.len();
                break;
            }
        }

        let url_from_start = &url[start..];
        let domain_end = url_from_start.find('/').unwrap_or(url_from_start.len());
        let domain = &url_from_start[..domain_end];

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

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start = prefix.len();
                break;
            }
        }

        let url_from_start = &url[start..];
        if let Some(slash_pos) = url_from_start.find('/') {
            let path_start = slash_pos;
            let path_and_query = &url_from_start[path_start..];
            
            if let Some(query_start) = path_and_query.find('?') {
                Some(path_and_query[..query_start].to_string())
            } else {
                Some(path_and_query.to_string())
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
            UrlParser::parse_domain("http://sub.domain.co.uk/page?q=test"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com/page?name=john&age=30&city=ny");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"ny".to_string()));
        
        let empty_params = UrlParser::parse_query_params("https://example.com/page");
        assert!(empty_params.is_empty());
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://www.example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/page?query=test"),
            Some("/page".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com"),
            Some("/".to_string())
        );
    }
}
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
}
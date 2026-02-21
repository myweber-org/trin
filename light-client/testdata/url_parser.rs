use std::collections::HashMap;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format");
        }

        let protocol = parts[0].to_string();
        let rest = parts[1];

        let domain_path: Vec<&str> = rest.splitn(2, '/').collect();
        let domain = domain_path[0].to_string();

        let path_and_query = if domain_path.len() > 1 {
            domain_path[1]
        } else {
            ""
        };

        let path_query: Vec<&str> = path_and_query.splitn(2, '?').collect();
        let path = if !path_query[0].is_empty() {
            format!("/{}", path_query[0])
        } else {
            "/".to_string()
        };

        let mut query_params = HashMap::new();
        if path_query.len() > 1 {
            for pair in path_query[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
        })
    }

    pub fn full_path(&self) -> String {
        if self.query_params.is_empty() {
            self.path.clone()
        } else {
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("{}?{}", self.path, query_string.join("&"))
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
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com/search?q=rust&limit=10";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_parse_url_root() {
        let url = "http://localhost:8080";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "localhost:8080");
        assert_eq!(parsed.path, "/");
    }
}
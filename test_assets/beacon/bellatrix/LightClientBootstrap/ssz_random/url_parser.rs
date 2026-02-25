use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url_str: &str) -> Result<Self, String> {
        let mut scheme = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url_str.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        scheme = parts[0].to_string();
        let rest = parts[1];

        let host_path_query: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_query[0];
        let path_and_query = if host_path_query.len() > 1 {
            format!("/{}", host_path_query[1])
        } else {
            "/".to_string()
        };

        let authority_parts: Vec<&str> = authority.split(':').collect();
        host = authority_parts[0].to_string();
        if authority_parts.len() == 2 {
            if let Ok(p) = authority_parts[1].parse::<u16>() {
                port = Some(p);
            } else {
                return Err("Invalid port number".to_string());
            }
        }

        let path_query_parts: Vec<&str> = path_and_query.splitn(2, '?').collect();
        path = path_query_parts[0].to_string();

        if path_query_parts.len() == 2 {
            for pair in path_query_parts[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            scheme,
            host,
            port,
            path,
            query_params,
        })
    }

    pub fn build_url(&self) -> String {
        let mut url = format!("{}://{}", self.scheme, self.host);
        if let Some(p) = self.port {
            url.push_str(&format!(":{}", p));
        }
        url.push_str(&self.path);

        if !self.query_params.is_empty() {
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push('?');
            url.push_str(&query_string.join("&"));
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = ParsedUrl::parse("https://example.com/path").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path");
        assert!(url.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = ParsedUrl::parse("http://localhost:8080/api").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api");
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = ParsedUrl::parse("https://api.test.com/search?q=rust&limit=10").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "api.test.com");
        assert_eq!(url.path, "/search");
        assert_eq!(url.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(url.query_params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_build_url() {
        let mut query_params = HashMap::new();
        query_params.insert("page".to_string(), "2".to_string());
        query_params.insert("sort".to_string(), "desc".to_string());

        let parsed_url = ParsedUrl {
            scheme: "https".to_string(),
            host: "example.org".to_string(),
            port: Some(443),
            path: "/data".to_string(),
            query_params,
        };

        let built = parsed_url.build_url();
        assert!(built.starts_with("https://example.org:443/data?"));
        assert!(built.contains("page=2"));
        assert!(built.contains("sort=desc"));
    }
}
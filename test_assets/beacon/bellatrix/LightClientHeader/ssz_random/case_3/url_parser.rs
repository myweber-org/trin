use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url_str: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url_str.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        protocol = parts[0].to_string();
        let rest = parts[1];

        let host_path_split: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_split[0];
        let path_and_query = if host_path_split.len() > 1 {
            format!("/{}", host_path_split[1])
        } else {
            "/".to_string()
        };

        let host_port_split: Vec<&str> = authority.split(':').collect();
        host = host_port_split[0].to_string();
        if host_port_split.len() == 2 {
            port = Some(host_port_split[1].parse().map_err(|_| "Invalid port number")?);
        }

        let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
        path = path_query_split[0].to_string();

        if path_query_split.len() == 2 {
            for pair in path_query_split[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            protocol,
            host,
            port,
            path,
            query_params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = ParsedUrl::parse("https://example.com/path/to/resource").unwrap();
        assert_eq!(url.protocol, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path/to/resource");
        assert!(url.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = ParsedUrl::parse("http://localhost:8080/api?key=value&sort=desc").unwrap();
        assert_eq!(url.protocol, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api");
        assert_eq!(url.query_params.get("key"), Some(&"value".to_string()));
        assert_eq!(url.query_params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = ParsedUrl::parse("not_a_valid_url");
        assert!(result.is_err());
    }
}
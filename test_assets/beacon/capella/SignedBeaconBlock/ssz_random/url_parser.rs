use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url: &str) -> Result<ParsedUrl, String> {
    let mut protocol = String::new();
    let mut host = String::new();
    let mut port = None;
    let mut path = String::new();
    let mut query_params = HashMap::new();

    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return Err("Invalid URL format".to_string());
    }
    protocol = parts[0].to_string();

    let rest = parts[1];
    let host_path_query: Vec<&str> = rest.splitn(2, '/').collect();
    let authority = host_path_query[0];

    let host_port: Vec<&str> = authority.split(':').collect();
    host = host_port[0].to_string();
    if host_port.len() == 2 {
        port = Some(host_port[1].parse().map_err(|_| "Invalid port number")?);
    }

    if host_path_query.len() == 2 {
        let path_query = host_path_query[1];
        let path_query_parts: Vec<&str> = path_query.splitn(2, '?').collect();
        path = format!("/{}", path_query_parts[0]);

        if path_query_parts.len() == 2 {
            for pair in path_query_parts[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }
    } else {
        path = "/".to_string();
    }

    Ok(ParsedUrl {
        protocol,
        host,
        port,
        path,
        query_params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = "http://localhost:8080/api/data?user=john&page=2";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
        assert_eq!(parsed.query_params.get("user"), Some(&"john".to_string()));
        assert_eq!(parsed.query_params.get("page"), Some(&"2".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not_a_valid_url";
        let result = parse_url(url);
        assert!(result.is_err());
    }
}
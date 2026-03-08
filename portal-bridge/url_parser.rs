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

        let host_path_query: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_query[0];
        let mut path_query = if host_path_query.len() > 1 {
            format!("/{}", host_path_query[1])
        } else {
            "/".to_string()
        };

        let host_port: Vec<&str> = authority.split(':').collect();
        host = host_port[0].to_string();
        if host_port.len() == 2 {
            port = Some(host_port[1].parse().map_err(|_| "Invalid port number")?);
        }

        let path_query_parts: Vec<&str> = path_query.splitn(2, '?').collect();
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
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = "http://localhost:8080/api/data?page=2&limit=50";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
        assert_eq!(parsed.query_params.get("page"), Some(&"2".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"50".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "invalid-url-without-protocol";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0].to_string();
                let value = parts[1].to_string();
                params.insert(key, value);
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let without_scheme = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
            let domain_end = without_scheme.find('/').unwrap_or(without_scheme.len());
            Some(without_scheme[..domain_end].to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=newyork";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://localhost:8080/api"),
            Some("localhost:8080".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("ftp://example.com"),
            None
        );
    }
}
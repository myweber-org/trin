use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/?#]+)").unwrap();
        re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = url.find('?');
        
        if let Some(start) = query_start {
            let query_str = &url[start + 1..];
            for pair in query_str.split('&') {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        let re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        re.is_match(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let url = "https://www.example.com/path?query=value";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
        
        let url_no_protocol = "example.com/resource";
        assert_eq!(UrlParser::parse_domain(url_no_protocol), Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=30&city=newyork";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("http://example.com"));
        assert!(UrlParser::is_valid_url("https://sub.domain.co.uk/path"));
        assert!(!UrlParser::is_valid_url("not-a-url"));
        assert!(!UrlParser::is_valid_url("ftp://invalid.protocol"));
    }
}use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<domain>[^/]+)(?P<path>/.*)?$").unwrap();
    
    re.captures(url).map(|caps| {
        let protocol = caps.name("protocol").map_or("", |m| m.as_str()).to_string();
        let domain = caps.name("domain").map_or("", |m| m.as_str()).to_string();
        let path = caps.name("path").map_or("/", |m| m.as_str()).to_string();
        
        ParsedUrl { protocol, domain, path }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_url() {
        let result = parse_url("https://www.example.com/path/to/resource");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "www.example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_url_without_path() {
        let result = parse_url("http://example.com");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = parse_url("not-a-valid-url");
        assert!(result.is_none());
    }
}
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: String,
    query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, &'static str> {
        let mut scheme = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format");
        }

        scheme = parts[0].to_string();
        let rest = parts[1];

        let host_path_query: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_query[0];
        let path_query = if host_path_query.len() > 1 {
            format!("/{}", host_path_query[1])
        } else {
            String::from("/")
        };

        let authority_parts: Vec<&str> = authority.split(':').collect();
        host = authority_parts[0].to_string();
        if authority_parts.len() == 2 {
            port = Some(authority_parts[1].parse().map_err(|_| "Invalid port")?);
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
            scheme,
            host,
            port,
            path,
            query_params,
        })
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.scheme(), "https");
        assert_eq!(parsed.host(), "example.com");
        assert_eq!(parsed.port(), None);
        assert_eq!(parsed.path(), "/path");
        assert!(parsed.query_params().is_empty());
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = "http://localhost:8080/api";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.scheme(), "http");
        assert_eq!(parsed.host(), "localhost");
        assert_eq!(parsed.port(), Some(8080));
        assert_eq!(parsed.path(), "/api");
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com/search?q=rust&limit=10";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.scheme(), "https");
        assert_eq!(parsed.host(), "api.example.com");
        assert_eq!(parsed.path(), "/search");
        assert_eq!(parsed.query_param("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_param("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }
}
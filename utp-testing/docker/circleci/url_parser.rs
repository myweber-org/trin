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

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Debug)]
pub enum UrlParseError {
    MissingScheme,
    InvalidScheme,
    MissingHost,
    InvalidPort,
    MalformedUrl,
}

impl fmt::Display for UrlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlParseError::MissingScheme => write!(f, "URL scheme is missing"),
            UrlParseError::InvalidScheme => write!(f, "URL scheme is invalid"),
            UrlParseError::MissingHost => write!(f, "URL host is missing"),
            UrlParseError::InvalidPort => write!(f, "URL port is invalid"),
            UrlParseError::MalformedUrl => write!(f, "URL is malformed"),
        }
    }
}

impl Error for UrlParseError {}

impl ParsedUrl {
    pub fn parse(url_str: &str) -> Result<Self, UrlParseError> {
        let url_str = url_str.trim();
        
        let scheme_end = url_str.find("://")
            .ok_or(UrlParseError::MissingScheme)?;
        
        let scheme = &url_str[..scheme_end];
        if scheme.is_empty() {
            return Err(UrlParseError::InvalidScheme);
        }
        
        let remaining = &url_str[scheme_end + 3..];
        
        let (authority, path_and_more) = split_once(remaining, '/');
        
        let (host_port, _) = split_once(authority, '@');
        
        let (host, port_str) = split_once(host_port, ':');
        
        if host.is_empty() {
            return Err(UrlParseError::MissingHost);
        }
        
        let port = if let Some(port_str) = port_str {
            let port = port_str.parse::<u16>()
                .map_err(|_| UrlParseError::InvalidPort)?;
            if port == 0 {
                return Err(UrlParseError::InvalidPort);
            }
            Some(port)
        } else {
            None
        };
        
        let (path, query_fragment) = split_once(path_and_more.unwrap_or(""), '?');
        
        let (query, fragment) = if let Some(query_fragment) = query_fragment {
            let (q, f) = split_once(query_fragment, '#');
            (Some(q.to_string()), f.map(|s| s.to_string()))
        } else {
            (None, None)
        };
        
        Ok(ParsedUrl {
            scheme: scheme.to_string(),
            host: host.to_string(),
            port,
            path: format!("/{}", path.unwrap_or("")),
            query,
            fragment,
        })
    }
    
    pub fn to_string(&self) -> String {
        let mut result = format!("{}://{}", self.scheme, self.host);
        
        if let Some(port) = self.port {
            result.push_str(&format!(":{}", port));
        }
        
        result.push_str(&self.path);
        
        if let Some(query) = &self.query {
            result.push_str(&format!("?{}", query));
        }
        
        if let Some(fragment) = &self.fragment {
            result.push_str(&format!("#{}", fragment));
        }
        
        result
    }
    
    pub fn is_secure(&self) -> bool {
        self.scheme == "https" || self.scheme == "wss"
    }
}

fn split_once(s: &str, delimiter: char) -> (&str, Option<&str>) {
    match s.find(delimiter) {
        Some(pos) => (&s[..pos], Some(&s[pos + 1..])),
        None => (s, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_url() {
        let url = ParsedUrl::parse("https://example.com:8080/path?query=value#fragment").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/path");
        assert_eq!(url.query, Some("query=value".to_string()));
        assert_eq!(url.fragment, Some("fragment".to_string()));
        assert!(url.is_secure());
    }
    
    #[test]
    fn test_parse_url_without_port() {
        let url = ParsedUrl::parse("http://example.com/path").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path");
        assert!(!url.is_secure());
    }
    
    #[test]
    fn test_parse_url_missing_scheme() {
        let result = ParsedUrl::parse("example.com/path");
        assert!(matches!(result, Err(UrlParseError::MissingScheme)));
    }
    
    #[test]
    fn test_parse_url_empty_host() {
        let result = ParsedUrl::parse("https:///path");
        assert!(matches!(result, Err(UrlParseError::MissingHost)));
    }
    
    #[test]
    fn test_to_string() {
        let url = ParsedUrl::parse("https://example.com:443/api/v1/users?active=true#section").unwrap();
        let reconstructed = url.to_string();
        assert_eq!(reconstructed, "https://example.com:443/api/v1/users?active=true#section");
    }
}use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"https?://([^/]+)").unwrap();
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
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    params.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let url = "https://www.example.com/path?query=123";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com?name=john&age=30";
        let params = UrlParser::parse_query_params(url);
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://example.com"));
        assert!(!UrlParser::is_valid_url("ftp://example.com"));
    }
}
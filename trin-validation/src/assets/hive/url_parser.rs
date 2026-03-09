use std::collections::HashMap;

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

        let remaining = &url[start..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        let domain = &remaining[..domain_end];

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

        let remaining = &url[start..];
        if let Some(slash_pos) = remaining.find('/') {
            let path_start = slash_pos;
            let path_and_query = &remaining[path_start..];
            
            if let Some(query_pos) = path_and_query.find('?') {
                Some(path_and_query[..query_pos].to_string())
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
            UrlParser::parse_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("ftp://files.server.com/file.txt"),
            Some("files.server.com".to_string())
        );
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params(
            "https://example.com/search?q=rust&lang=en&page=1"
        );
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/users"),
            Some("/api/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/"),
            Some("/".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com"),
            Some("/".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/page?query=test"),
            Some("/page".to_string())
        );
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() < 2 {
            return None;
        }

        let domain_part = parts[1];
        let domain_end = domain_part.find('/').unwrap_or(domain_part.len());
        let domain = &domain_part[..domain_end];

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
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 {
                    params.insert(kv[0].to_string(), kv[1].to_string());
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

        if let Some(domain_end) = url.find("://") {
            let after_protocol = &url[domain_end + 3..];
            if let Some(path_start) = after_protocol.find('/') {
                let path = &after_protocol[path_start..];
                if let Some(query_start) = path.find('?') {
                    return Some(path[..query_start].to_string());
                }
                return Some(path.to_string());
            }
        }
        
        None
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
            UrlParser::parse_domain("http://localhost:8080/api"),
            Some("localhost:8080".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/users"),
            Some("/api/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/search?q=test"),
            Some("/search".to_string())
        );
        assert_eq!(UrlParser::extract_path("invalid"), None);
    }
}
use regex::Regex;
use std::collections::HashSet;

pub struct UrlParser {
    domain_blacklist: HashSet<String>,
}

impl UrlParser {
    pub fn new() -> Self {
        let mut blacklist = HashSet::new();
        blacklist.insert("localhost".to_string());
        blacklist.insert("127.0.0.1".to_string());
        blacklist.insert("::1".to_string());
        blacklist.insert("0.0.0.0".to_string());
        
        UrlParser {
            domain_blacklist: blacklist,
        }
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?(?:www\.)?([^/:]+)").unwrap();
        
        if let Some(captures) = re.captures(url) {
            let domain = captures.get(1)?.as_str().to_lowercase();
            
            if self.is_valid_domain(&domain) {
                return Some(domain);
            }
        }
        None
    }

    pub fn is_valid_url(&self, url: &str) -> bool {
        let url_regex = Regex::new(
            r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"
        ).unwrap();
        
        url_regex.is_match(url) && self.extract_domain(url).is_some()
    }

    fn is_valid_domain(&self, domain: &str) -> bool {
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }

        if self.domain_blacklist.contains(domain) {
            return false;
        }

        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return false;
        }

        for part in parts {
            if part.is_empty() || part.len() > 63 {
                return false;
            }
            
            if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
            
            if part.starts_with('-') || part.ends_with('-') {
                return false;
            }
        }

        true
    }

    pub fn normalize_url(&self, url: &str) -> Option<String> {
        if !self.is_valid_url(url) {
            return None;
        }

        let domain = self.extract_domain(url)?;
        let mut normalized = format!("https://{}", domain);

        if let Some(path_start) = url.find(&domain) {
            let path = &url[path_start + domain.len()..];
            if !path.is_empty() {
                normalized.push_str(path);
            }
        }

        Some(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(parser.extract_domain("http://www.test.org"), Some("test.org".to_string()));
        assert_eq!(parser.extract_domain("invalid-url"), None);
        assert_eq!(parser.extract_domain("https://localhost/api"), None);
    }

    #[test]
    fn test_is_valid_url() {
        let parser = UrlParser::new();
        
        assert!(parser.is_valid_url("https://example.com"));
        assert!(parser.is_valid_url("http://sub.domain.co.uk/path?query=value"));
        assert!(!parser.is_valid_url("not-a-url"));
        assert!(!parser.is_valid_url("https://localhost"));
    }

    #[test]
    fn test_normalize_url() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.normalize_url("http://example.com"), Some("https://example.com".to_string()));
        assert_eq!(parser.normalize_url("https://www.test.org/path"), Some("https://test.org/path".to_string()));
        assert_eq!(parser.normalize_url("invalid"), None);
    }
}
use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<domain>[^/]+)(?P<path>/.*)?$").unwrap();
    let caps = re.captures(url)?;

    let protocol = caps.name("protocol")?.as_str().to_string();
    let domain = caps.name("domain")?.as_str().to_string();
    let path = caps.name("path").map_or("/".to_string(), |m| m.as_str().to_string());

    Some(ParsedUrl {
        protocol,
        domain,
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_http_url() {
        let parsed = parse_url("http://example.com/path/to/resource").unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_valid_https_url_without_path() {
        let parsed = parse_url("https://example.com").unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let parsed = parse_url("not-a-valid-url");
        assert!(parsed.is_none());
    }
}
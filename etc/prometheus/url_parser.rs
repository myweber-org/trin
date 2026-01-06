use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<host>[^/]+)(?P<path>/.*)?$").unwrap();
    
    re.captures(url).map(|caps| {
        let protocol = caps.name("protocol").map_or("", |m| m.as_str()).to_string();
        let host = caps.name("host").map_or("", |m| m.as_str()).to_string();
        let path = caps.name("path").map_or("/", |m| m.as_str()).to_string();
        
        ParsedUrl { protocol, host, path }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let url = "http://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let url = "https://api.github.com";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "api.github.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        let parsed = parse_url(url);
        
        assert!(parsed.is_none());
    }
}
use std::collections::HashMap;
use url::Url;

pub struct UrlParser {
    url: Url,
}

impl UrlParser {
    pub fn new(url_str: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(url_str)?;
        Ok(UrlParser { url })
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.host_str()
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        self.url.query_pairs()
            .into_owned()
            .collect()
    }

    pub fn path_segments(&self) -> Vec<String> {
        self.url.path_segments()
            .map(|segments| segments.map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let parser = UrlParser::new("https://example.com/api/v1/users?page=2&limit=50").unwrap();
        
        assert_eq!(parser.domain(), Some("example.com"));
        
        let params = parser.query_params();
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("limit"), Some(&"50".to_string()));
        
        let segments = parser.path_segments();
        assert_eq!(segments, vec!["api", "v1", "users"]);
    }

    #[test]
    fn test_invalid_url() {
        let result = UrlParser::new("not-a-valid-url");
        assert!(result.is_err());
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
        if let Some(query_start) = url.find('?') {
            let query_str = &url[query_start + 1..];
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
        assert!(!UrlParser::is_valid_url("invalid-url"));
    }
}
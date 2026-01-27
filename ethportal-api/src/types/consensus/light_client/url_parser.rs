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
}
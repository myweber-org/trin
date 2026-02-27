use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url: &str) -> Option<ParsedUrl> {
        let url_regex = Regex::new(r"^(https?)://([^/]+)(/[^?#]*)?(\?[^#]*)?(#.*)?$").ok()?;
        let captures = url_regex.captures(url)?;

        let scheme = captures.get(1)?.as_str().to_string();
        let host = captures.get(2)?.as_str().to_string();
        let path = captures.get(3).map_or("/", |m| m.as_str()).to_string();
        let query_string = captures.get(4).map(|m| m.as_str()[1..].to_string());
        let fragment = captures.get(5).map(|m| m.as_str()[1..].to_string());

        let query_params = query_string
            .as_ref()
            .map(|qs| Self::parse_query_string(qs))
            .unwrap_or_else(HashMap::new);

        Some(ParsedUrl {
            scheme,
            host,
            path,
            query_params,
            fragment,
        })
    }

    fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if !key.is_empty() {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        Self::parse(url).is_some()
    }
}

pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn has_query_params(&self) -> bool {
        !self.query_params.is_empty()
    }

    pub fn build_full_url(&self) -> String {
        let mut url = format!("{}://{}{}", self.scheme, self.host, self.path);
        
        if !self.query_params.is_empty() {
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push('?');
            url.push_str(&query_string.join("&"));
        }
        
        if let Some(ref fragment) = self.fragment {
            url.push('#');
            url.push_str(fragment);
        }
        
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_parsing() {
        let url = "https://example.com/path?key1=value1&key2=value2#section";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path");
        assert_eq!(parsed.get_query_param("key1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("key2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_url_without_query_or_fragment() {
        let url = "http://localhost:8080/api/users";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "localhost:8080");
        assert_eq!(parsed.path, "/api/users");
        assert!(parsed.query_params.is_empty());
        assert!(parsed.fragment.is_none());
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        assert!(UrlParser::parse(url).is_none());
        assert!(!UrlParser::is_valid_url(url));
    }

    #[test]
    fn test_build_full_url() {
        let mut query_params = HashMap::new();
        query_params.insert("page".to_string(), "1".to_string());
        query_params.insert("sort".to_string(), "desc".to_string());
        
        let parsed = ParsedUrl {
            scheme: "https".to_string(),
            host: "api.example.com".to_string(),
            path: "/data".to_string(),
            query_params,
            fragment: Some("results".to_string()),
        };
        
        let built_url = parsed.build_full_url();
        assert!(built_url.contains("https://api.example.com/data"));
        assert!(built_url.contains("?page=1&sort=desc"));
        assert!(built_url.contains("#results"));
    }
}
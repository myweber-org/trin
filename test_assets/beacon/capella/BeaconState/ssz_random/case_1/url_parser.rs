use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser {
    url: String,
}

impl UrlParser {
    pub fn new(url: &str) -> Self {
        UrlParser {
            url: url.to_string(),
        }
    }

    pub fn extract_domain(&self) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/?#]+)").unwrap();
        re.captures(&self.url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = self.url.find('?');

        if let Some(start) = query_start {
            let query_string = &self.url[start + 1..];
            for pair in query_string.split('&') {
                let mut kv = pair.split('=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        params
    }

    pub fn is_valid_url(&self) -> bool {
        let url_pattern = Regex::new(
            r"^(https?://)?([\w\-]+\.)+[\w\-]+(/[\w\-./?%&=]*)?$"
        ).unwrap();
        url_pattern.is_match(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://example.com/path?query=1");
        assert_eq!(parser.extract_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_query_parsing() {
        let parser = UrlParser::new("https://test.com?name=john&age=25");
        let params = parser.parse_query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"25".to_string()));
    }

    #[test]
    fn test_url_validation() {
        let valid = UrlParser::new("https://valid-domain.com");
        let invalid = UrlParser::new("not-a-valid-url");
        
        assert!(valid.is_valid_url());
        assert!(!invalid.is_valid_url());
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
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
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
            UrlParser::extract_domain("https://www.example.com/path/to/page"),
            Some("www.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("ftp://files.server.com/documents"),
            Some("files.server.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_no_protocol() {
        assert_eq!(
            UrlParser::extract_domain("//cdn.example.com/assets"),
            Some("cdn.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("example.com/page"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_invalid_urls() {
        assert_eq!(UrlParser::extract_domain(""), None);
        assert_eq!(UrlParser::extract_domain("://"), None);
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url: &str) -> Option<ParsedUrl> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let (scheme, rest) = Self::extract_scheme(url);
        let (host, path_and_query) = Self::extract_host(rest)?;
        let (path, query) = Self::split_path_and_query(path_and_query);
        let query_params = Self::parse_query_string(query);

        Some(ParsedUrl {
            scheme: scheme.to_string(),
            host: host.to_string(),
            path: path.to_string(),
            query_params,
        })
    }

    fn extract_scheme(url: &str) -> (&str, &str) {
        if let Some(pos) = url.find("://") {
            (&url[..pos], &url[pos + 3..])
        } else {
            ("http", url)
        }
    }

    fn extract_host<'a>(rest: &'a str) -> Option<(&'a str, &'a str)> {
        let end = rest.find('/').unwrap_or(rest.len());
        let host = &rest[..end];
        if host.is_empty() {
            return None;
        }
        let remaining = if end < rest.len() { &rest[end..] } else { "" };
        Some((host, remaining))
    }

    fn split_path_and_query(path_and_query: &str) -> (&str, &str) {
        if let Some(pos) = path_and_query.find('?') {
            (&path_and_query[..pos], &path_and_query[pos + 1..])
        } else {
            (path_and_query, "")
        }
    }

    fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        params
    }

    pub fn get_domain(url: &str) -> Option<String> {
        Self::parse(url).map(|parsed| parsed.host)
    }

    pub fn get_query_param(url: &str, key: &str) -> Option<String> {
        Self::parse(url)
            .and_then(|parsed| parsed.query_params.get(key).cloned())
    }
}

pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn full_url(&self) -> String {
        let mut url = format!("{}://{}", self.scheme, self.host);
        if !self.path.is_empty() && !self.path.starts_with('/') {
            url.push('/');
        }
        url.push_str(&self.path);

        if !self.query_params.is_empty() {
            url.push('?');
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&query_string.join("&"));
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://example.com/path/to/resource?param1=value1&param2=value2";
        let parsed = UrlParser::parse(url).unwrap();

        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.query_params.get("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.query_params.get("param2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_parse_url_without_scheme() {
        let url = "example.com/path";
        let parsed = UrlParser::parse(url).unwrap();

        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_without_path() {
        let url = "https://example.com";
        let parsed = UrlParser::parse(url).unwrap();

        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_get_domain() {
        assert_eq!(
            UrlParser::get_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(UrlParser::get_domain("invalid"), None);
    }

    #[test]
    fn test_get_query_param() {
        let url = "https://example.com?key1=val1&key2=val2";
        assert_eq!(
            UrlParser::get_query_param(url, "key1"),
            Some("val1".to_string())
        );
        assert_eq!(
            UrlParser::get_query_param(url, "key2"),
            Some("val2".to_string())
        );
        assert_eq!(UrlParser::get_query_param(url, "key3"), None);
    }

    #[test]
    fn test_full_url_reconstruction() {
        let url = "https://example.com/path?param1=value1&param2=value2";
        let parsed = UrlParser::parse(url).unwrap();
        assert_eq!(parsed.full_url(), url);
    }
}
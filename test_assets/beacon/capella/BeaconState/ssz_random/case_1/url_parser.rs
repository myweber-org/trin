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
}
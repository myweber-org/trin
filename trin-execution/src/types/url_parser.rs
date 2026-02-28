use std::collections::HashMap;
use url::Url;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_url(url_str: &str) -> Result<ParsedUrl, String> {
        let url = Url::parse(url_str).map_err(|e| e.to_string())?;
        
        let domain = url.host_str()
            .map(|h| h.to_string())
            .unwrap_or_default();
        
        let query_params: HashMap<String, String> = url.query_pairs()
            .into_owned()
            .collect();
        
        Ok(ParsedUrl {
            original: url_str.to_string(),
            domain,
            query_params,
        })
    }
}

pub struct ParsedUrl {
    pub original: String,
    pub domain: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    pub fn has_param(&self, key: &str) -> bool {
        self.query_params.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_url() {
        let url = "https://example.com/search?q=rust&lang=en";
        let parsed = UrlParser::parse_url(url).unwrap();
        
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.get_param("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_param("lang"), Some(&"en".to_string()));
        assert!(parsed.has_param("q"));
        assert!(!parsed.has_param("nonexistent"));
    }
    
    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        let result = UrlParser::parse_url(url);
        assert!(result.is_err());
    }
}
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let mut remaining = url;

        if let Some(proto_end) = remaining.find("://") {
            protocol = remaining[..proto_end].to_string();
            remaining = &remaining[proto_end + 3..];
        }

        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        domain = remaining[..domain_end].to_string();
        remaining = &remaining[domain_end..];

        if let Some(query_start) = remaining.find('?') {
            path = remaining[..query_start].to_string();
            remaining = &remaining[query_start + 1..];

            if let Some(fragment_start) = remaining.find('#') {
                parse_query_string(&remaining[..fragment_start], &mut query_params);
                fragment = Some(remaining[fragment_start + 1..].to_string());
            } else {
                parse_query_string(remaining, &mut query_params);
            }
        } else if let Some(fragment_start) = remaining.find('#') {
            path = remaining[..fragment_start].to_string();
            fragment = Some(remaining[fragment_start + 1..].to_string());
        } else {
            path = remaining.to_string();
        }

        if domain.is_empty() {
            return Err("Domain cannot be empty".to_string());
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    pub fn get_root_domain(&self) -> Option<String> {
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() >= 2 {
            let last_two = parts[parts.len() - 2..].join(".");
            Some(last_two)
        } else {
            None
        }
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
}

fn parse_query_string(query_str: &str, params: &mut HashMap<String, String>) {
    for pair in query_str.split('&') {
        if let Some(equal_pos) = pair.find('=') {
            let key = &pair[..equal_pos];
            let value = &pair[equal_pos + 1..];
            if !key.is_empty() {
                params.insert(key.to_string(), value.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://www.example.com/path/to/page?param1=value1&param2=value2#section";
        let parsed = ParsedUrl::new(url).unwrap();

        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "www.example.com");
        assert_eq!(parsed.path, "/path/to/page");
        assert_eq!(parsed.get_query_param("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("param2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
        assert_eq!(parsed.get_root_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_url_no_protocol() {
        let url = "example.com/path";
        let parsed = ParsedUrl::new(url).unwrap();

        assert_eq!(parsed.protocol, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_invalid() {
        let url = "";
        let result = ParsedUrl::new(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_root_domain_extraction() {
        let url = "https://sub.domain.example.co.uk/path";
        let parsed = ParsedUrl::new(url).unwrap();
        assert_eq!(parsed.get_root_domain(), Some("co.uk".to_string()));
    }
}
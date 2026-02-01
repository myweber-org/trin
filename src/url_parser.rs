
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let mut protocol = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let url_lower = url.to_lowercase();
        let url_str = url_lower.trim();

        if url_str.is_empty() {
            return Err("Empty URL provided".into());
        }

        let protocol_end = url_str.find("://");
        let remaining = if let Some(end) = protocol_end {
            protocol = url_str[..end].to_string();
            &url_str[end + 3..]
        } else {
            url_str
        };

        let hash_pos = remaining.find('#');
        let before_fragment = if let Some(pos) = hash_pos {
            fragment = Some(remaining[pos + 1..].to_string());
            &remaining[..pos]
        } else {
            remaining
        };

        let query_pos = before_fragment.find('?');
        let before_query = if let Some(pos) = query_pos {
            let query_str = &before_fragment[pos + 1..];
            for pair in query_str.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
                    if !key.is_empty() {
                        query_params.insert(key.to_string(), value.to_string());
                    }
                }
            }
            &before_fragment[..pos]
        } else {
            before_fragment
        };

        let slash_pos = before_query.find('/');
        if let Some(pos) = slash_pos {
            domain = before_query[..pos].to_string();
            path = before_query[pos..].to_string();
        } else {
            domain = before_query.to_string();
            path = "/".to_string();
        }

        if domain.is_empty() {
            return Err("No domain found in URL".into());
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

    pub fn has_query_param(&self, key: &str) -> bool {
        self.query_params.contains_key(key)
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
}

pub fn extract_domain(url: &str) -> Result<String, Box<dyn Error>> {
    let parsed = ParsedUrl::new(url)?;
    Ok(parsed.domain)
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
        assert_eq!(parsed.query_params.get("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.query_params.get("param2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_parse_url_without_protocol() {
        let url = "example.com/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path");
    }

    #[test]
    fn test_root_domain_extraction() {
        let url = "https://subdomain.example.co.uk/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.get_root_domain(), Some("co.uk".to_string()));
    }

    #[test]
    fn test_query_param_operations() {
        let url = "https://example.com/?search=rust&page=1";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert!(parsed.has_query_param("search"));
        assert_eq!(parsed.get_query_param("search"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_query_param("nonexistent"), None);
    }

    #[test]
    fn test_invalid_url() {
        let result = ParsedUrl::new("");
        assert!(result.is_err());
    }
}
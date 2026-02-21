use std::collections::HashMap;

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
        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut cleaned_url = url_lower.as_str();
        for prefix in prefixes.iter() {
            if cleaned_url.starts_with(prefix) {
                cleaned_url = &cleaned_url[prefix.len()..];
            }
        }

        let domain_end = cleaned_url.find('/').unwrap_or(cleaned_url.len());
        let domain = &cleaned_url[..domain_end];
        
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
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk/api"),
            Some("sub.domain.co.uk".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            None
        );
    }
}
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let mut scheme = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let url_lower = url.to_lowercase();
        let url_str = url_lower.trim();

        if let Some(scheme_end) = url_str.find("://") {
            scheme = url_str[..scheme_end].to_string();
            let remaining = &url_str[scheme_end + 3..];

            let (domain_part, rest) = if let Some(path_start) = remaining.find('/') {
                (&remaining[..path_start], &remaining[path_start..])
            } else {
                (remaining, "")
            };

            domain = domain_part.to_string();

            let (path_part, query_fragment) = if let Some(query_start) = rest.find('?') {
                (&rest[..query_start], &rest[query_start..])
            } else if let Some(fragment_start) = rest.find('#') {
                (&rest[..fragment_start], &rest[fragment_start..])
            } else {
                (rest, "")
            };

            path = if path_part.is_empty() { "/".to_string() } else { path_part.to_string() };

            let mut remaining_query = query_fragment;

            if let Some(query_start) = remaining_query.find('?') {
                remaining_query = &remaining_query[query_start + 1..];
                let (query_str, fragment_part) = if let Some(fragment_start) = remaining_query.find('#') {
                    (&remaining_query[..fragment_start], &remaining_query[fragment_start..])
                } else {
                    (remaining_query, "")
                };

                for param in query_str.split('&') {
                    if param.is_empty() {
                        continue;
                    }
                    let pair: Vec<&str> = param.splitn(2, '=').collect();
                    if pair.len() == 2 {
                        query_params.insert(pair[0].to_string(), pair[1].to_string());
                    } else if !pair[0].is_empty() {
                        query_params.insert(pair[0].to_string(), String::new());
                    }
                }

                if !fragment_part.is_empty() && fragment_part.starts_with('#') {
                    fragment = Some(fragment_part[1..].to_string());
                }
            } else if remaining_query.starts_with('#') {
                fragment = Some(remaining_query[1..].to_string());
            }
        } else {
            return Err("Invalid URL: missing scheme".into());
        }

        if domain.is_empty() {
            return Err("Invalid URL: missing domain".into());
        }

        Ok(ParsedUrl {
            scheme,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn has_query_params(&self) -> bool {
        !self.query_params.is_empty()
    }

    pub fn root_domain(&self) -> Option<String> {
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() >= 2 {
            let last_two = parts[parts.len() - 2..].join(".");
            Some(last_two)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_url() {
        let url = "https://www.example.com/path/to/resource";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "www.example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.service.com/search?q=rust&limit=10&sort=desc";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "api.service.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.get_query_param("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_query_param("limit"), Some(&"10".to_string()));
        assert_eq!(parsed.get_query_param("sort"), Some(&"desc".to_string()));
        assert_eq!(parsed.has_query_params(), true);
    }

    #[test]
    fn test_parse_url_with_fragment() {
        let url = "https://docs.rs/serde/latest/serde/#modules";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "docs.rs");
        assert_eq!(parsed.path, "/serde/latest/serde/");
        assert_eq!(parsed.fragment, Some("modules".to_string()));
    }

    #[test]
    fn test_root_domain_extraction() {
        let url = "https://sub.domain.example.co.uk/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.root_domain(), Some("co.uk".to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::new(url);
        
        assert!(result.is_err());
    }
}
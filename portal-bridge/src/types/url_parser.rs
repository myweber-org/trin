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
        
        let mut clean_url = url;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                clean_url = &url[prefix.len()..];
                break;
            }
        }

        let domain_end = clean_url.find('/').unwrap_or(clean_url.len());
        let domain = &clean_url[..domain_end];
        
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
        
        let mut clean_url = url;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                clean_url = &url[prefix.len()..];
                break;
            }
        }

        if let Some(path_start) = clean_url.find('/') {
            let path = &clean_url[path_start..];
            
            if let Some(query_start) = path.find('?') {
                Some(path[..query_start].to_string())
            } else {
                Some(path.to_string())
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
            UrlParser::parse_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://sub.example.co.uk:8080"),
            Some("sub.example.co.uk:8080".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), Some("invalid-url".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com?name=john&age=30&city=nyc");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"nyc".to_string()));
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/search?q=rust"),
            Some("/search".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("ftp://server.com"),
            Some("/".to_string())
        );
        assert_eq!(UrlParser::extract_path(""), None);
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
        let (path, query_string) = Self::split_path_and_query(path_and_query);
        let query_params = Self::parse_query_string(query_string);

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
            ("https", url)
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

    fn split_path_and_query(path_and_query: &str) -> (&str, Option<&str>) {
        let query_start = path_and_query.find('?');
        match query_start {
            Some(pos) => (&path_and_query[..pos], Some(&path_and_query[pos + 1..])),
            None => (path_and_query, None),
        }
    }

    fn parse_query_string(query_string: Option<&str>) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if let Some(qs) = query_string {
            for pair in qs.split('&') {
                let mut parts = pair.splitn(2, '=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("").to_string();
                    params.insert(key.to_string(), value);
                }
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
        let mut url = format!("{}://{}{}", self.scheme, self.host, self.path);
        if !self.query_params.is_empty() {
            url.push('?');
            let params: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&params.join("&"));
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
    fn test_get_domain() {
        assert_eq!(
            UrlParser::get_domain("https://api.github.com/users/octocat"),
            Some("api.github.com".to_string())
        );
        assert_eq!(
            UrlParser::get_domain("http://localhost:8080/api/data"),
            Some("localhost:8080".to_string())
        );
    }

    #[test]
    fn test_get_query_param() {
        let url = "https://search.com?q=rust&lang=en&sort=recent";
        assert_eq!(
            UrlParser::get_query_param(url, "q"),
            Some("rust".to_string())
        );
        assert_eq!(
            UrlParser::get_query_param(url, "lang"),
            Some("en".to_string())
        );
        assert_eq!(UrlParser::get_query_param(url, "missing"), None);
    }

    #[test]
    fn test_full_url_reconstruction() {
        let original = "https://example.com/api/v1/users?active=true&role=admin";
        let parsed = UrlParser::parse(original).unwrap();
        assert_eq!(parsed.full_url(), original);
    }

    #[test]
    fn test_url_without_scheme() {
        let parsed = UrlParser::parse("example.com/path").unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path");
    }

    #[test]
    fn test_empty_url() {
        assert!(UrlParser::parse("").is_none());
        assert!(UrlParser::parse("   ").is_none());
    }
}
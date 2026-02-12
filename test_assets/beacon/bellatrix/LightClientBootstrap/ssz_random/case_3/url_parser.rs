use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url_str: &str) -> Result<ParsedUrl, String> {
    let parts: Vec<&str> = url_str.split("://").collect();
    if parts.len() != 2 {
        return Err("Invalid URL format".to_string());
    }

    let scheme = parts[0].to_string();
    let rest = parts[1];

    let host_path_split: Vec<&str> = rest.splitn(2, '/').collect();
    let host = host_path_split[0].to_string();

    let path_and_query = if host_path_split.len() > 1 {
        host_path_split[1]
    } else {
        ""
    };

    let path_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
    let path = if !path_split[0].is_empty() {
        format!("/{}", path_split[0])
    } else {
        "/".to_string()
    };

    let mut query_params = HashMap::new();
    if path_split.len() > 1 {
        for pair in path_split[1].split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 {
                query_params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    Ok(ParsedUrl {
        scheme,
        host,
        path,
        query_params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "http://example.com/search?q=rust&lang=en";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("lang"), Some(&"en".to_string()));
    }

    #[test]
    fn test_parse_url_root_path() {
        let url = "ftp://fileserver.net";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.scheme, "ftp");
        assert_eq!(parsed.host, "fileserver.net");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        let result = parse_url(url);
        assert!(result.is_err());
    }
}
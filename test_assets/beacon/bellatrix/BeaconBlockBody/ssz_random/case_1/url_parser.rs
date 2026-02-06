use std::collections::HashMap;

pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return None;
    }

    let protocol = parts[0].to_string();
    let rest = parts[1];

    let host_path_split: Vec<&str> = rest.splitn(2, '/').collect();
    let host = host_path_split[0].to_string();

    let path_and_query = if host_path_split.len() > 1 {
        host_path_split[1]
    } else {
        ""
    };

    let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
    let path = if !path_query_split[0].is_empty() {
        format!("/{}", path_query_split[0])
    } else {
        "/".to_string()
    };

    let mut query_params = HashMap::new();
    if path_query_split.len() > 1 {
        for pair in path_query_split[1].split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 {
                query_params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    Some(ParsedUrl {
        protocol,
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
        let url = "https://www.example.com/index.html";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "www.example.com");
        assert_eq!(parsed.path, "/index.html");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com/search?q=rust&lang=en";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("lang"), Some(&"en".to_string()));
    }

    #[test]
    fn test_parse_url_no_path() {
        let url = "ftp://fileserver.example.com";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.host, "fileserver.example.com");
        assert_eq!(parsed.path, "/");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        assert!(parse_url(url).is_none());
    }
}
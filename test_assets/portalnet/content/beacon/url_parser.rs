use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn parse(url_str: &str) -> Result<Self, String> {
        let mut scheme = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let parts: Vec<&str> = url_str.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        scheme = parts[0].to_string();
        let rest = parts[1];

        let (host_port, path_fragment) = match rest.find('/') {
            Some(pos) => (&rest[..pos], &rest[pos..]),
            None => (rest, ""),
        };

        let (host_part, port_part) = match host_port.find(':') {
            Some(pos) => (&host_port[..pos], Some(&host_port[pos + 1..])),
            None => (host_port, None),
        };

        host = host_part.to_string();

        if let Some(p) = port_part {
            port = Some(p.parse::<u16>().map_err(|_| "Invalid port number")?);
        }

        let (path_part, fragment_part) = match path_fragment.find('#') {
            Some(pos) => (&path_fragment[..pos], Some(&path_fragment[pos + 1..])),
            None => (path_fragment, None),
        };

        let (path_only, query_part) = match path_part.find('?') {
            Some(pos) => (&path_part[..pos], Some(&path_part[pos + 1..])),
            None => (path_part, None),
        };

        path = if path_only.is_empty() { "/".to_string() } else { path_only.to_string() };

        if let Some(query_str) = query_part {
            for pair in query_str.split('&') {
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        fragment = fragment_part.map(|s| s.to_string());

        Ok(ParsedUrl {
            scheme,
            host,
            port,
            path,
            query_params,
            fragment,
        })
    }

    pub fn build_url(&self) -> String {
        let mut url = format!("{}://{}", self.scheme, self.host);
        
        if let Some(port) = self.port {
            url.push_str(&format!(":{}", port));
        }
        
        url.push_str(&self.path);
        
        if !self.query_params.is_empty() {
            url.push('?');
            let queries: Vec<String> = self.query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&queries.join("&"));
        }
        
        if let Some(fragment) = &self.fragment {
            url.push_str(&format!("#{}", fragment));
        }
        
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = ParsedUrl::parse("https://example.com/path/to/resource").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path/to/resource");
        assert!(url.query_params.is_empty());
        assert_eq!(url.fragment, None);
    }

    #[test]
    fn test_parse_url_with_all_components() {
        let url = ParsedUrl::parse("https://example.com:8080/api/v1/users?name=john&age=30#section").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api/v1/users");
        assert_eq!(url.query_params.get("name"), Some(&"john".to_string()));
        assert_eq!(url.query_params.get("age"), Some(&"30".to_string()));
        assert_eq!(url.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_build_url() {
        let mut query_params = HashMap::new();
        query_params.insert("page".to_string(), "2".to_string());
        query_params.insert("sort".to_string(), "desc".to_string());
        
        let parsed_url = ParsedUrl {
            scheme: "https".to_string(),
            host: "api.example.com".to_string(),
            port: Some(443),
            path: "/data".to_string(),
            query_params,
            fragment: Some("results".to_string()),
        };
        
        let built_url = parsed_url.build_url();
        assert!(built_url.contains("https://api.example.com:443/data?"));
        assert!(built_url.contains("page=2"));
        assert!(built_url.contains("sort=desc"));
        assert!(built_url.ends_with("#results"));
    }
}
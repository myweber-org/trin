use regex::Regex;

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
        let re = Regex::new(r"https?://([^/]+)").unwrap();
        re.captures(&self.url)
            .map(|caps| caps[1].to_string())
    }

    pub fn extract_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        let re = Regex::new(r"[?&]([^=]+)=([^&]+)").unwrap();
        
        for cap in re.captures_iter(&self.url) {
            let key = cap[1].to_string();
            let value = cap[2].to_string();
            params.push((key, value));
        }
        
        params
    }

    pub fn has_secure_protocol(&self) -> bool {
        self.url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://example.com/path?query=value");
        assert_eq!(parser.extract_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_query_params() {
        let parser = UrlParser::new("https://example.com?name=john&age=30");
        let params = parser.extract_query_params();
        assert_eq!(params.len(), 2);
        assert!(params.contains(&("name".to_string(), "john".to_string())));
    }

    #[test]
    fn test_secure_protocol() {
        let secure_parser = UrlParser::new("https://secure.com");
        let insecure_parser = UrlParser::new("http://insecure.com");
        
        assert!(secure_parser.has_secure_protocol());
        assert!(!insecure_parser.has_secure_protocol());
    }
}
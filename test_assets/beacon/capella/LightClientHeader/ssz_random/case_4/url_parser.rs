use regex::Regex;
use std::collections::HashSet;

pub struct UrlParser {
    domain_blacklist: HashSet<String>,
}

impl UrlParser {
    pub fn new() -> Self {
        let mut blacklist = HashSet::new();
        blacklist.insert("localhost".to_string());
        blacklist.insert("127.0.0.1".to_string());
        blacklist.insert("::1".to_string());
        
        UrlParser {
            domain_blacklist: blacklist,
        }
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        let pattern = r"^(?:https?://)?(?:www\.)?([^/:]+)";
        let re = Regex::new(pattern).unwrap();
        
        re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_lowercase())
            .filter(|domain| !self.domain_blacklist.contains(domain))
    }

    pub fn is_valid_url(&self, url: &str) -> bool {
        let url_pattern = r"^(https?://)?(www\.)?[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(/\S*)?$";
        let re = Regex::new(url_pattern).unwrap();
        
        re.is_match(url) && self.extract_domain(url).is_some()
    }

    pub fn normalize_url(&self, url: &str) -> Option<String> {
        if !self.is_valid_url(url) {
            return None;
        }

        let domain = self.extract_domain(url)?;
        let protocol = if url.starts_with("https") { "https" } else { "http" };
        
        Some(format!("{}://{}", protocol, domain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.extract_domain("https://example.com/page"), Some("example.com".to_string()));
        assert_eq!(parser.extract_domain("http://www.github.com/user/repo"), Some("github.com".to_string()));
        assert_eq!(parser.extract_domain("invalid-url"), None);
    }

    #[test]
    fn test_url_validation() {
        let parser = UrlParser::new();
        
        assert!(parser.is_valid_url("https://example.com"));
        assert!(parser.is_valid_url("http://sub.domain.co.uk/path"));
        assert!(!parser.is_valid_url("localhost:8080"));
        assert!(!parser.is_valid_url("not-a-url"));
    }

    #[test]
    fn test_url_normalization() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.normalize_url("https://EXAMPLE.COM"), Some("https://example.com".to_string()));
        assert_eq!(parser.normalize_url("http://www.github.com"), Some("http://github.com".to_string()));
        assert_eq!(parser.normalize_url("invalid"), None);
    }
}
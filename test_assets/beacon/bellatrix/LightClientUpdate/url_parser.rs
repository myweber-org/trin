
use regex::Regex;
use std::collections::HashSet;

lazy_static::lazy_static! {
    static ref DOMAIN_REGEX: Regex = Regex::new(r"^(?:https?://)?(?:www\.)?([^/:]+)").unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
}

pub struct UrlParser {
    allowed_domains: HashSet<String>,
}

impl UrlParser {
    pub fn new() -> Self {
        UrlParser {
            allowed_domains: HashSet::new(),
        }
    }

    pub fn with_allowed_domains(domains: Vec<&str>) -> Self {
        let mut parser = Self::new();
        for domain in domains {
            parser.add_allowed_domain(domain);
        }
        parser
    }

    pub fn add_allowed_domain(&mut self, domain: &str) {
        self.allowed_domains.insert(domain.to_lowercase());
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        DOMAIN_REGEX.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_lowercase())
    }

    pub fn is_valid_url(&self, url: &str) -> bool {
        URL_REGEX.is_match(url)
    }

    pub fn is_allowed_domain(&self, url: &str) -> bool {
        if let Some(domain) = self.extract_domain(url) {
            self.allowed_domains.contains(&domain)
        } else {
            false
        }
    }

    pub fn normalize_url(&self, url: &str) -> String {
        let mut normalized = url.trim().to_lowercase();
        
        if !normalized.starts_with("http") {
            normalized = format!("https://{}", normalized);
        }
        
        if let Some(domain) = self.extract_domain(&normalized) {
            if domain.starts_with("www.") {
                normalized = normalized.replacen(&format!("www.{}", &domain[4..]), &domain[4..], 1);
            }
        }
        
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new();
        
        assert_eq!(
            parser.extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            parser.extract_domain("http://subdomain.example.co.uk:8080"),
            Some("subdomain.example.co.uk".to_string())
        );
    }

    #[test]
    fn test_url_validation() {
        let parser = UrlParser::new();
        
        assert!(parser.is_valid_url("https://example.com"));
        assert!(parser.is_valid_url("http://example.com/path?query=1"));
        assert!(!parser.is_valid_url("not-a-url"));
        assert!(!parser.is_valid_url("ftp://example.com"));
    }

    #[test]
    fn test_allowed_domains() {
        let parser = UrlParser::with_allowed_domains(vec!["example.com", "api.github.com"]);
        
        assert!(parser.is_allowed_domain("https://example.com/page"));
        assert!(parser.is_allowed_domain("http://api.github.com/users"));
        assert!(!parser.is_allowed_domain("https://forbidden.com"));
    }

    #[test]
    fn test_url_normalization() {
        let parser = UrlParser::new();
        
        assert_eq!(
            parser.normalize_url("  HTTPS://WWW.EXAMPLE.COM/Path  "),
            "https://example.com/path"
        );
        
        assert_eq!(
            parser.normalize_url("example.com"),
            "https://example.com"
        );
    }
}
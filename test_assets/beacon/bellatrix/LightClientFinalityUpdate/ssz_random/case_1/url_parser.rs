
use regex::Regex;
use std::collections::HashSet;

pub struct UrlParser {
    domain_regex: Regex,
    tlds: HashSet<String>,
}

impl UrlParser {
    pub fn new() -> Self {
        let domain_pattern = r"^(?:https?://)?(?:www\.)?([a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*\.[a-zA-Z]{2,})(?:/|$)";
        let tld_list = vec![
            "com", "org", "net", "edu", "gov", "io", "co", "uk", "de", "fr",
            "it", "es", "nl", "ru", "jp", "cn", "br", "au", "ca", "in",
        ];

        UrlParser {
            domain_regex: Regex::new(domain_pattern).unwrap(),
            tlds: tld_list.into_iter().map(String::from).collect(),
        }
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        self.domain_regex.captures(url).map(|caps| {
            caps.get(1).unwrap().as_str().to_lowercase()
        })
    }

    pub fn is_valid_tld(&self, domain: &str) -> bool {
        domain.split('.').last()
            .map(|tld| self.tlds.contains(tld))
            .unwrap_or(false)
    }

    pub fn parse(&self, url: &str) -> Option<ParsedUrl> {
        self.extract_domain(url).and_then(|domain| {
            if self.is_valid_tld(&domain) {
                Some(ParsedUrl {
                    original: url.to_string(),
                    domain,
                    is_secure: url.starts_with("https://"),
                })
            } else {
                None
            }
        })
    }
}

pub struct ParsedUrl {
    pub original: String,
    pub domain: String,
    pub is_secure: bool,
}

impl ParsedUrl {
    pub fn display(&self) -> String {
        format!(
            "URL: {}\nDomain: {}\nSecure: {}",
            self.original, self.domain, self.is_secure
        )
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
            parser.extract_domain("http://subdomain.example.co.uk"),
            Some("subdomain.example.co.uk".to_string())
        );
        
        assert_eq!(
            parser.extract_domain("invalid-url"),
            None
        );
    }

    #[test]
    fn test_tld_validation() {
        let parser = UrlParser::new();
        
        assert!(parser.is_valid_tld("example.com"));
        assert!(parser.is_valid_tld("test.co.uk"));
        assert!(!parser.is_valid_tld("example.invalid"));
    }

    #[test]
    fn test_full_parse() {
        let parser = UrlParser::new();
        
        let parsed = parser.parse("https://api.github.com/users/rust-lang");
        assert!(parsed.is_some());
        
        let parsed = parsed.unwrap();
        assert_eq!(parsed.domain, "api.github.com");
        assert!(parsed.is_secure);
        
        assert!(parser.parse("http://invalid.tld.xyz").is_none());
    }
}
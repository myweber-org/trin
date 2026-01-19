use regex::Regex;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum UrlValidationError {
    InvalidFormat,
    UnsupportedProtocol,
    MissingHost,
}

impl fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlValidationError::InvalidFormat => write!(f, "URL format is invalid"),
            UrlValidationError::UnsupportedProtocol => write!(f, "URL protocol is not supported"),
            UrlValidationError::MissingHost => write!(f, "URL must contain a host"),
        }
    }
}

impl Error for UrlValidationError {}

pub struct UrlValidator {
    protocol_regex: Regex,
    url_regex: Regex,
    allowed_protocols: Vec<String>,
}

impl UrlValidator {
    pub fn new(allowed_protocols: Option<Vec<String>>) -> Result<Self, regex::Error> {
        let protocols = allowed_protocols.unwrap_or_else(|| vec!["http".into(), "https".into()]);
        
        let protocol_pattern = protocols
            .iter()
            .map(|p| regex::escape(p))
            .collect::<Vec<_>>()
            .join("|");

        let protocol_regex = Regex::new(&format!("^(?:{})://", protocol_pattern))?;
        let url_regex = Regex::new(
            r"^(https?|ftp)://([a-zA-Z0-9.-]+)(?::([0-9]+))?(/[^?#]*)?(\?[^#]*)?(#.*)?$"
        )?;

        Ok(UrlValidator {
            protocol_regex,
            url_regex,
            allowed_protocols: protocols,
        })
    }

    pub fn validate(&self, url: &str) -> Result<(), UrlValidationError> {
        if url.trim().is_empty() {
            return Err(UrlValidationError::InvalidFormat);
        }

        if !self.protocol_regex.is_match(url) {
            return Err(UrlValidationError::UnsupportedProtocol);
        }

        if !self.url_regex.is_match(url) {
            return Err(UrlValidationError::InvalidFormat);
        }

        if let Some(captures) = self.url_regex.captures(url) {
            if captures.get(2).is_none() {
                return Err(UrlValidationError::MissingHost);
            }
        }

        Ok(())
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        self.url_regex
            .captures(url)
            .and_then(|caps| caps.get(2))
            .map(|m| m.as_str().to_string())
    }

    pub fn get_allowed_protocols(&self) -> &[String] {
        &self.allowed_protocols
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_http_url() {
        let validator = UrlValidator::new(None).unwrap();
        assert!(validator.validate("http://example.com").is_ok());
    }

    #[test]
    fn test_valid_https_url() {
        let validator = UrlValidator::new(None).unwrap();
        assert!(validator.validate("https://example.com/path?query=1").is_ok());
    }

    #[test]
    fn test_invalid_protocol() {
        let validator = UrlValidator::new(None).unwrap();
        assert_eq!(
            validator.validate("ftp://example.com"),
            Err(UrlValidationError::UnsupportedProtocol)
        );
    }

    #[test]
    fn test_custom_protocols() {
        let validator = UrlValidator::new(Some(vec!["ftp".into(), "sftp".into()])).unwrap();
        assert!(validator.validate("ftp://example.com").is_ok());
        assert!(validator.validate("sftp://example.com").is_ok());
        assert_eq!(
            validator.validate("http://example.com"),
            Err(UrlValidationError::UnsupportedProtocol)
        );
    }

    #[test]
    fn test_extract_domain() {
        let validator = UrlValidator::new(None).unwrap();
        assert_eq!(
            validator.extract_domain("https://sub.example.com:8080/path"),
            Some("sub.example.com".into())
        );
    }

    #[test]
    fn test_empty_url() {
        let validator = UrlValidator::new(None).unwrap();
        assert_eq!(
            validator.validate(""),
            Err(UrlValidationError::InvalidFormat)
        );
    }
}
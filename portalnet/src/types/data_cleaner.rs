
use std::collections::HashSet;

pub struct DataCleaner {
    pub deduplicate: bool,
    pub validate_emails: bool,
    pub max_length: Option<usize>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplicate: true,
            validate_emails: false,
            max_length: None,
        }
    }

    pub fn clean_strings(&self, strings: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for s in strings {
            let mut processed = s.trim().to_string();

            if let Some(max_len) = self.max_length {
                if processed.len() > max_len {
                    processed.truncate(max_len);
                }
            }

            if self.deduplicate {
                if seen.contains(&processed) {
                    continue;
                }
                seen.insert(processed.clone());
            }

            if self.validate_emails && !is_valid_email(&processed) {
                continue;
            }

            result.push(processed);
        }

        result
    }

    pub fn with_deduplication(mut self, deduplicate: bool) -> Self {
        self.deduplicate = deduplicate;
        self
    }

    pub fn with_email_validation(mut self, validate: bool) -> Self {
        self.validate_emails = validate;
        self
    }

    pub fn with_max_length(mut self, max_length: Option<usize>) -> Self {
        self.max_length = max_length;
        self
    }
}

fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    !local.is_empty() && 
    !domain.is_empty() && 
    domain.contains('.') &&
    !domain.starts_with('.') &&
    !domain.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new().with_deduplication(true);
        let input = vec![
            "test@example.com".to_string(),
            "test@example.com".to_string(),
            "another@test.com".to_string(),
        ];
        
        let result = cleaner.clean_strings(input);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_email_validation() {
        let cleaner = DataCleaner::new().with_email_validation(true);
        let input = vec![
            "valid@example.com".to_string(),
            "invalid-email".to_string(),
            "another@test.org".to_string(),
        ];
        
        let result = cleaner.clean_strings(input);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_max_length() {
        let cleaner = DataCleaner::new().with_max_length(Some(10));
        let input = vec![
            "short".to_string(),
            "very_long_string_here".to_string(),
            "exact_len".to_string(),
        ];
        
        let result = cleaner.clean_strings(input);
        assert_eq!(result[1].len(), 10);
    }
}
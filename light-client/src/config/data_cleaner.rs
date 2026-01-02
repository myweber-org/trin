use std::collections::HashSet;

pub struct DataCleaner {
    seen_ids: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            seen_ids: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, id: &str) -> bool {
        if self.seen_ids.contains(id) {
            false
        } else {
            self.seen_ids.insert(id.to_string());
            true
        }
    }

    pub fn validate_email(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && 
        !parts[0].is_empty() && 
        !domain_parts.iter().any(|part| part.is_empty())
    }

    pub fn clean_whitespace(input: &str) -> String {
        input.trim().to_string()
    }

    pub fn get_unique_count(&self) -> usize {
        self.seen_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("id1"));
        assert!(!cleaner.deduplicate("id1"));
        assert!(cleaner.deduplicate("id2"));
    }

    #[test]
    fn test_validate_email() {
        assert!(DataCleaner::validate_email("test@example.com"));
        assert!(!DataCleaner::validate_email("invalid-email"));
        assert!(!DataCleaner::validate_email("@domain.com"));
        assert!(!DataCleaner::validate_email("user@.com"));
    }

    #[test]
    fn test_clean_whitespace() {
        assert_eq!(DataCleaner::clean_whitespace("  hello  "), "hello");
        assert_eq!(DataCleaner::clean_whitespace("no_spaces"), "no_spaces");
    }
}
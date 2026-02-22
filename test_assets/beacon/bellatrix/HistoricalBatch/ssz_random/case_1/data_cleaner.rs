use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let total = self.records.len();
        let valid = self.records
            .iter()
            .filter(|record| !record.trim().is_empty())
            .count();

        (total, valid)
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let (total, valid) = self.validate_records();
        let duplicates = total - self.records.len();
        
        (valid, duplicates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test1".to_string());
        cleaner.add_record("test2".to_string());
        cleaner.add_record("test1".to_string());
        
        let unique = cleaner.deduplicate();
        assert_eq!(unique.len(), 2);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("  ".to_string());
        
        let (total, valid) = cleaner.validate_records();
        assert_eq!(total, 3);
        assert_eq!(valid, 1);
    }
}
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    unique_ids: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_ids: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, id: &str) -> bool {
        self.unique_ids.insert(id.to_string())
    }

    pub fn validate_email(email: &str) -> Result<(), Box<dyn Error>> {
        if email.is_empty() {
            return Err("Email cannot be empty".into());
        }
        
        if !email.contains('@') {
            return Err("Email must contain @ symbol".into());
        }
        
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Invalid email format".into());
        }
        
        Ok(())
    }

    pub fn normalize_phone_number(phone: &str) -> String {
        phone.chars()
            .filter(|c| c.is_ascii_digit())
            .collect()
    }

    pub fn clean_text(text: &str) -> String {
        text.trim()
            .replace('\t', " ")
            .replace('\n', " ")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("user123"));
        assert!(!cleaner.deduplicate("user123"));
        assert!(cleaner.deduplicate("user456"));
    }

    #[test]
    fn test_validate_email() {
        assert!(DataCleaner::validate_email("test@example.com").is_ok());
        assert!(DataCleaner::validate_email("invalid").is_err());
        assert!(DataCleaner::validate_email("").is_err());
    }

    #[test]
    fn test_normalize_phone_number() {
        assert_eq!(DataCleaner::normalize_phone_number("(123) 456-7890"), "1234567890");
        assert_eq!(DataCleaner::normalize_phone_number("+1-800-555-1234"), "18005551234");
    }

    #[test]
    fn test_clean_text() {
        assert_eq!(DataCleaner::clean_text("  Hello\tworld\n\n"), "Hello world");
        assert_eq!(DataCleaner::clean_text("Multiple   spaces   here"), "Multiple spaces here");
    }
}
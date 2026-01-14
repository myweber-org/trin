use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    records: Vec<String>,
    seen: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> Result<(), Box<dyn Error>> {
        let trimmed = record.trim();
        
        if trimmed.is_empty() {
            return Err("Empty record not allowed".into());
        }

        if trimmed.len() > 1000 {
            return Err("Record exceeds maximum length".into());
        }

        if self.seen.contains(trimmed) {
            return Err("Duplicate record detected".into());
        }

        self.seen.insert(trimmed.to_string());
        self.records.push(trimmed.to_string());
        Ok(())
    }

    pub fn validate_email(&self, index: usize) -> bool {
        if let Some(record) = self.records.get(index) {
            record.contains('@') && record.contains('.')
        } else {
            false
        }
    }

    pub fn get_unique_records(&self) -> Vec<String> {
        self.records.clone()
    }

    pub fn clean_whitespace(&mut self) {
        self.records = self.records
            .iter()
            .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
            .collect();
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test@example.com").unwrap();
        assert!(cleaner.add_record("test@example.com").is_err());
        assert_eq!(cleaner.count_records(), 1);
    }

    #[test]
    fn test_email_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid@example.com").unwrap();
        cleaner.add_record("invalid").unwrap();
        
        assert!(cleaner.validate_email(0));
        assert!(!cleaner.validate_email(1));
    }

    #[test]
    fn test_whitespace_cleaning() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces  ").unwrap();
        cleaner.clean_whitespace();
        
        assert_eq!(cleaner.records[0], "multiple spaces");
    }
}
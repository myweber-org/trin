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

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| !record.trim().is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let total = self.records.len();
        let valid_count = self.validate_records()
            .iter()
            .filter(|&&is_valid| is_valid)
            .count();
        
        (total, valid_count)
    }
}

pub fn sanitize_input(input: &str) -> String {
    input.trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let unique = cleaner.deduplicate();
        assert_eq!(unique.len(), 2);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("   ".to_string());
        
        let validation = cleaner.validate_records();
        assert_eq!(validation, vec![true, false]);
    }

    #[test]
    fn test_sanitize() {
        let input = "Hello, World! 123";
        let sanitized = sanitize_input(input);
        assert_eq!(sanitized, "Hello World 123");
    }
}
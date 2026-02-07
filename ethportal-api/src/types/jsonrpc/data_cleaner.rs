
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    deduplicated: bool,
}

impl DataCleaner {
    pub fn new(records: Vec<String>) -> Self {
        DataCleaner {
            records,
            deduplicated: false,
        }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        if !self.deduplicated {
            let mut seen = HashSet::new();
            self.records = self
                .records
                .iter()
                .filter(|record| seen.insert(record.trim().to_lowercase()))
                .cloned()
                .collect();
            self.deduplicated = true;
        }
        self
    }

    pub fn validate_emails(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                record.contains('@')
                    && record.split('@').count() == 2
                    && !record.starts_with('@')
                    && !record.ends_with('@')
            })
            .collect()
    }

    pub fn filter_valid_emails(&self) -> Vec<String> {
        let validation = self.validate_emails();
        self.records
            .iter()
            .zip(validation.iter())
            .filter_map(|(record, &valid)| if valid { Some(record.clone()) } else { None })
            .collect()
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let records = vec![
            "test@example.com".to_string(),
            "TEST@example.com".to_string(),
            "another@test.org".to_string(),
            "test@example.com".to_string(),
        ];

        let mut cleaner = DataCleaner::new(records);
        cleaner.deduplicate();

        assert_eq!(cleaner.count(), 2);
        assert!(cleaner.deduplicated);
    }

    #[test]
    fn test_email_validation() {
        let records = vec![
            "valid@example.com".to_string(),
            "invalid-email".to_string(),
            "@missinglocal.org".to_string(),
            "missingdomain@".to_string(),
        ];

        let cleaner = DataCleaner::new(records);
        let validation = cleaner.validate_emails();

        assert_eq!(validation, vec![true, false, false, false]);
    }

    #[test]
    fn test_filter_valid_emails() {
        let records = vec![
            "user@domain.com".to_string(),
            "not-an-email".to_string(),
            "admin@server.org".to_string(),
        ];

        let cleaner = DataCleaner::new(records);
        let valid_emails = cleaner.filter_valid_emails();

        assert_eq!(valid_emails.len(), 2);
        assert!(valid_emails.contains(&"user@domain.com".to_string()));
        assert!(valid_emails.contains(&"admin@server.org".to_string()));
    }
}
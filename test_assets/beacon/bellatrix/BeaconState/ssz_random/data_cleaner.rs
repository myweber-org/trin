use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_case: bool,
}

impl DataCleaner {
    pub fn new(remove_duplicates: bool, normalize_case: bool) -> Self {
        DataCleaner {
            remove_duplicates,
            normalize_case,
        }
    }

    pub fn clean_data(&self, input: Vec<String>) -> Vec<String> {
        let mut processed = input;

        if self.normalize_case {
            processed = processed
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed.into_iter().collect();
            processed = unique_set.into_iter().collect();
        }

        processed.sort();
        processed
    }

    pub fn validate_email(&self, email: &str) -> bool {
        let email = email.trim();
        let parts: Vec<&str> = email.split('@').collect();
        
        if parts.len() != 2 {
            return false;
        }

        let local_part = parts[0];
        let domain_part = parts[1];

        !local_part.is_empty() 
            && !domain_part.is_empty()
            && domain_part.contains('.')
            && !email.contains(' ')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data_with_duplicates() {
        let cleaner = DataCleaner::new(true, false);
        let input = vec![
            "Apple".to_string(),
            "banana".to_string(),
            "Apple".to_string(),
            "Cherry".to_string(),
        ];
        
        let result = cleaner.clean_data(input);
        assert_eq!(result, vec!["Apple", "Cherry", "banana"]);
    }

    #[test]
    fn test_clean_data_normalize_case() {
        let cleaner = DataCleaner::new(false, true);
        let input = vec![
            "APPLE".to_string(),
            "Banana".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.clean_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_validate_email() {
        let cleaner = DataCleaner::new(false, false);
        
        assert!(cleaner.validate_email("user@example.com"));
        assert!(!cleaner.validate_email("invalid-email"));
        assert!(!cleaner.validate_email("user@com"));
        assert!(!cleaner.validate_email("user @example.com"));
    }
}
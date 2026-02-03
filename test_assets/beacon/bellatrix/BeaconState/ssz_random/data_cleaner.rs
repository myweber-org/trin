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
}use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_empty_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(deduplicate(input), expected);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let expected = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(normalize_strings(input), expected);
    }

    #[test]
    fn test_filter_empty_strings() {
        let input = vec!["hello".to_string(), "".to_string(), "world".to_string()];
        let expected = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(filter_empty_strings(input), expected);
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        for item in data {
            if self.deduplicate(&item) {
                cleaned.push(item);
            }
        }
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "APPLE".to_string(),
            " banana ".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];

        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}
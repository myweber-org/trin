use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    result
}

pub fn normalize_strings(strings: &[String]) -> Vec<String> {
    strings
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn clean_data(strings: &[String]) -> Vec<String> {
    let normalized = normalize_strings(strings);
    deduplicate(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(deduplicate(&input), expected);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  Hello  ".to_string(),
            "WORLD".to_string(),
            "  TeSt  ".to_string(),
        ];
        let expected = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
        assert_eq!(normalize_strings(&input), expected);
    }

    #[test]
    fn test_clean_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "  BANANA  ".to_string(),
            "Banana".to_string(),
            "  APPLE  ".to_string(),
        ];
        let expected = vec!["apple".to_string(), "banana".to_string()];
        assert_eq!(clean_data(&input), expected);
    }
}
use std::collections::HashSet;

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_strings(data: Vec<String>) -> Vec<String> {
        let mut unique_set: HashSet<String> = HashSet::new();
        
        for item in data {
            unique_set.insert(item);
        }
        
        let mut result: Vec<String> = unique_set.into_iter().collect();
        result.sort();
        result
    }
    
    pub fn clean_numbers(data: Vec<i32>) -> Vec<i32> {
        let mut unique_set: HashSet<i32> = HashSet::new();
        
        for item in data {
            unique_set.insert(item);
        }
        
        let mut result: Vec<i32> = unique_set.into_iter().collect();
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_strings() {
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
            "banana".to_string(),
        ];
        
        let result = DataCleaner::clean_strings(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_clean_numbers() {
        let input = vec![5, 2, 8, 2, 5, 1, 8];
        
        let result = DataCleaner::clean_numbers(input);
        assert_eq!(result, vec![1, 2, 5, 8]);
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_cache: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_cache: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn remove_duplicates(&mut self, items: Vec<String>) -> Vec<String> {
        let mut unique_items = Vec::new();
        
        for item in items {
            let normalized = self.normalize_text(&item);
            if self.deduplication_cache.insert(normalized.clone()) {
                unique_items.push(item);
            }
        }
        
        unique_items
    }

    pub fn validate_email(&self, email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && 
        !parts[0].is_empty() && 
        !domain_parts.iter().any(|part| part.is_empty())
    }

    pub fn clean_whitespace(&self, input: &str) -> String {
        input.split_whitespace().collect::<Vec<&str>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "test@example.com".to_string(),
            "TEST@example.com".to_string(),
            "another@test.com".to_string(),
        ];
        
        let result = cleaner.remove_duplicates(data);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_validate_email() {
        let cleaner = DataCleaner::new();
        assert!(cleaner.validate_email("user@domain.com"));
        assert!(!cleaner.validate_email("invalid-email"));
    }
}
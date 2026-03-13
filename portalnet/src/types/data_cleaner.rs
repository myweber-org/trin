
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
}use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_enabled: bool,
    normalization_rules: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_enabled: true,
            normalization_rules: Vec::new(),
        }
    }

    pub fn enable_deduplication(&mut self, enabled: bool) {
        self.deduplication_enabled = enabled;
    }

    pub fn add_normalization_rule(&mut self, rule: String) {
        self.normalization_rules.push(rule);
    }

    pub fn clean_data(&self, input: Vec<String>) -> Vec<String> {
        let mut processed_data = input;

        for rule in &self.normalization_rules {
            processed_data = processed_data
                .iter()
                .map(|item| self.apply_normalization(item, rule))
                .collect();
        }

        if self.deduplication_enabled {
            processed_data = self.remove_duplicates(processed_data);
        }

        processed_data
    }

    fn apply_normalization(&self, item: &str, rule: &str) -> String {
        match rule.as_str() {
            "trim_whitespace" => item.trim().to_string(),
            "lowercase" => item.to_lowercase(),
            "remove_special_chars" => item.chars().filter(|c| c.is_alphanumeric()).collect(),
            _ => item.to_string(),
        }
    }

    fn remove_duplicates(&self, data: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in data {
            if seen.insert(item.clone()) {
                result.push(item);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];

        let result = cleaner.clean_data(input);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_normalization_rule("trim_whitespace".to_string());
        cleaner.add_normalization_rule("lowercase".to_string());

        let input = vec![
            "  APPLE  ".to_string(),
            "Banana".to_string(),
            "CHERRY".to_string(),
        ];

        let result = cleaner.clean_data(input);
        assert_eq!(result[0], "apple");
        assert_eq!(result[1], "banana");
        assert_eq!(result[2], "cherry");
    }
}
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn remove_nulls(self) -> Self
    where
        T: PartialEq,
    {
        let filtered_data: Vec<T> = self
            .data
            .into_iter()
            .filter(|item| *item != None)
            .collect();
        Self {
            data: filtered_data,
        }
    }

    pub fn deduplicate(self) -> Self
    where
        T: Eq + std::hash::Hash + Clone,
    {
        let mut seen = HashSet::new();
        let unique_data: Vec<T> = self
            .data
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect();
        Self {
            data: unique_data,
        }
    }

    pub fn get_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_dataset<T>(data: Vec<T>) -> Vec<T>
where
    T: Eq + std::hash::Hash + Clone + PartialEq,
{
    let cleaner = DataCleaner::new(data);
    cleaner.remove_nulls().deduplicate().get_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let input = vec![Some(1), None, Some(2), Some(1), None, Some(2)];
        let cleaned = clean_dataset(input);
        assert_eq!(cleaned, vec![Some(1), Some(2)]);
    }

    #[test]
    fn test_remove_nulls() {
        let cleaner = DataCleaner::new(vec![Some("a"), None, Some("b"), None]);
        let result = cleaner.remove_nulls().get_data();
        assert_eq!(result, vec![Some("a"), Some("b")]);
    }

    #[test]
    fn test_deduplicate() {
        let cleaner = DataCleaner::new(vec![1, 2, 2, 3, 1, 4]);
        let result = cleaner.deduplicate().get_data();
        assert_eq!(result, vec![1, 2, 3, 4]);
    }
}
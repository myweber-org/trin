use std::collections::HashSet;

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

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            true
        } else {
            self.dedupe_set.insert(normalized);
            false
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        for item in data {
            if !self.is_duplicate(&item) {
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
            "Apple".to_string(),
            "apple".to_string(),
            "BANANA".to_string(),
            "banana ".to_string(),
            "Cherry".to_string(),
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
}use std::collections::HashSet;

pub struct DataCleaner {
    pub records: Vec<String>,
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

    pub fn deduplicate(&mut self) -> usize {
        let unique_set: HashSet<String> = self.records.drain(..).collect();
        let original_count = self.records.len();
        self.records = unique_set.into_iter().collect();
        original_count - self.records.len()
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| !record.trim().is_empty() && record.len() <= 255)
            .collect()
    }

    pub fn clean_all(&mut self) -> (usize, Vec<bool>) {
        let duplicates_removed = self.deduplicate();
        let validation_results = self.validate_records();
        (duplicates_removed, validation_results)
    }
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
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("a".repeat(256));
        
        let results = cleaner.validate_records();
        assert_eq!(results, vec![true, false, false]);
    }
}use regex::Regex;

pub fn sanitize_input(input: &str) -> String {
    let trimmed = input.trim();
    
    let re = Regex::new(r"\s+").unwrap();
    let normalized_whitespace = re.replace_all(trimmed, " ");
    
    let re_special = Regex::new(r"[^\w\s\-.,!?]").unwrap();
    let cleaned = re_special.replace_all(&normalized_whitespace, "");
    
    cleaned.to_string()
}

pub fn normalize_case(input: &str, mode: &str) -> String {
    match mode {
        "upper" => input.to_uppercase(),
        "lower" => input.to_lowercase(),
        "title" => {
            let mut result = String::new();
            let mut capitalize_next = true;
            
            for c in input.chars() {
                if c.is_whitespace() {
                    capitalize_next = true;
                    result.push(c);
                } else if capitalize_next {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c.to_ascii_lowercase());
                }
            }
            result
        }
        _ => input.to_string()
    }
}

pub fn remove_duplicate_spaces(input: &str) -> String {
    let re = Regex::new(r"\s{2,}").unwrap();
    re.replace_all(input, " ").to_string()
}
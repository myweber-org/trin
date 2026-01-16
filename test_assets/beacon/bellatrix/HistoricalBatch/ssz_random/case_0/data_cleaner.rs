
use std::collections::HashSet;

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_strings(strings: Vec<String>) -> Vec<String> {
        let mut unique_strings: HashSet<String> = strings.into_iter().collect();
        let mut sorted_strings: Vec<String> = unique_strings.into_iter().collect();
        sorted_strings.sort();
        sorted_strings
    }
    
    pub fn clean_integers(numbers: Vec<i32>) -> Vec<i32> {
        let mut unique_numbers: HashSet<i32> = numbers.into_iter().collect();
        let mut sorted_numbers: Vec<i32> = unique_numbers.into_iter().collect();
        sorted_numbers.sort();
        sorted_numbers
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
    fn test_clean_integers() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let result = DataCleaner::clean_integers(input);
        assert_eq!(result, vec![1, 2, 5, 8, 9]);
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
        let mut seen = HashSet::new();
        let mut deduped = Vec::new();
        
        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                deduped.push(record);
            }
        }
        
        let removed = self.records.capacity() - deduped.len();
        self.records = deduped;
        removed
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        
        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 1000 {
                valid_count += 1;
            }
        }
        
        (valid_count, self.records.len() - valid_count)
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let total_chars: usize = self.records.iter().map(|r| r.len()).sum();
        let avg_length = if self.records.is_empty() {
            0
        } else {
            total_chars / self.records.len()
        };
        
        (self.records.len(), avg_length)
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
        
        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 1);
    }
}
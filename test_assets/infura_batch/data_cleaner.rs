use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<String>,
}

impl DataCleaner {
    pub fn new(data: Vec<String>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn validate_non_empty(&self) -> Result<(), String> {
        for (index, item) in self.data.iter().enumerate() {
            if item.trim().is_empty() {
                return Err(format!("Empty string found at index {}", index));
            }
        }
        Ok(())
    }

    pub fn transform_to_uppercase(&mut self) -> &mut Self {
        for item in &mut self.data {
            *item = item.to_uppercase();
        }
        self
    }

    pub fn get_data(&self) -> &Vec<String> {
        &self.data
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new(vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ]);
        
        cleaner.deduplicate();
        assert_eq!(cleaner.count(), 3);
        assert_eq!(cleaner.get_data(), &vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ]);
    }

    #[test]
    fn test_validation() {
        let cleaner = DataCleaner::new(vec![
            "valid".to_string(),
            "".to_string(),
            "also valid".to_string(),
        ]);
        
        let result = cleaner.validate_non_empty();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty string found at index 1");
    }

    #[test]
    fn test_uppercase_transformation() {
        let mut cleaner = DataCleaner::new(vec![
            "hello".to_string(),
            "world".to_string(),
        ]);
        
        cleaner.transform_to_uppercase();
        assert_eq!(cleaner.get_data(), &vec![
            "HELLO".to_string(),
            "WORLD".to_string(),
        ]);
    }
}
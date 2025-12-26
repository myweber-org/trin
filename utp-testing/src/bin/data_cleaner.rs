use std::collections::HashMap;

pub struct DataCleaner {
    filters: Vec<Box<dyn Fn(&str) -> bool>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            filters: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn clean_data(&self, data: Vec<String>) -> Vec<String> {
        data.into_iter()
            .filter(|entry| self.filters.iter().all(|filter| filter(entry)))
            .collect()
    }

    pub fn create_default_cleaner() -> Self {
        let mut cleaner = DataCleaner::new();
        cleaner.add_filter(|s| !s.trim().is_empty());
        cleaner.add_filter(|s| s.len() <= 100);
        cleaner.add_filter(|s| !s.contains("NULL"));
        cleaner
    }
}

pub fn process_dataset(dataset: HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    let cleaner = DataCleaner::create_default_cleaner();
    
    dataset
        .into_iter()
        .map(|(key, values)| (key, cleaner.clean_data(values)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_filters_empty_strings() {
        let cleaner = DataCleaner::create_default_cleaner();
        let data = vec![
            "valid".to_string(),
            "".to_string(),
            "   ".to_string(),
            "another_valid".to_string(),
        ];
        
        let cleaned = cleaner.clean_data(data);
        assert_eq!(cleaned, vec!["valid", "another_valid"]);
    }

    #[test]
    fn test_custom_filter() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_filter(|s| s.starts_with("A"));
        
        let data = vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Apricot".to_string(),
        ];
        
        let cleaned = cleaner.clean_data(data);
        assert_eq!(cleaned, vec!["Apple", "Apricot"]);
    }
}
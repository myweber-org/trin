
use std::collections::HashMap;

pub struct DataCleaner {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            filters: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn clean_data(&self, records: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        records
            .into_iter()
            .filter(|record| self.filters.iter().all(|filter| filter(record)))
            .collect()
    }
}

pub fn create_default_cleaner() -> DataCleaner {
    let mut cleaner = DataCleaner::new();
    
    cleaner.add_filter(|record| {
        record.contains_key("id") && !record.get("id").unwrap().is_empty()
    });
    
    cleaner.add_filter(|record| {
        record.get("timestamp")
            .and_then(|ts| ts.parse::<u64>().ok())
            .map_or(false, |timestamp| timestamp > 0)
    });
    
    cleaner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let cleaner = create_default_cleaner();
        
        let mut valid_record = HashMap::new();
        valid_record.insert("id".to_string(), "123".to_string());
        valid_record.insert("timestamp".to_string(), "1672531200".to_string());
        
        let mut invalid_record = HashMap::new();
        invalid_record.insert("id".to_string(), "".to_string());
        invalid_record.insert("timestamp".to_string(), "0".to_string());
        
        let records = vec![valid_record.clone(), invalid_record];
        let cleaned = cleaner.clean_data(records);
        
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].get("id").unwrap(), "123");
    }
}
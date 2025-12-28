
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
        };
        
        processor.register_validator("email", |s| {
            s.contains('@') && s.contains('.') && s.len() > 5
        });
        
        processor.register_validator("phone", |s| {
            s.chars().all(|c| c.is_ascii_digit()) && s.len() >= 10
        });
        
        processor
    }
    
    pub fn register_validator<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name.to_string(), Box::new(validator));
    }
    
    pub fn validate(&self, validator_name: &str, data: &str) -> bool {
        match self.validators.get(validator_name) {
            Some(validator) => validator(data),
            None => false,
        }
    }
    
    pub fn transform_data(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
            .collect::<String>()
            .to_uppercase()
    }
    
    pub fn process_batch(&self, items: Vec<&str>) -> Vec<String> {
        items
            .into_iter()
            .map(|item| self.transform_data(item))
            .filter(|item| !item.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }
    
    #[test]
    fn test_phone_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("phone", "1234567890"));
        assert!(!processor.validate("phone", "abc123"));
    }
    
    #[test]
    fn test_data_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform_data("hello world! 123");
        assert_eq!(result, "HELLO WORLD 123");
    }
    
    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new();
        let items = vec!["item1", "item2", "item3"];
        let processed = processor.process_batch(items);
        assert_eq!(processed, vec!["ITEM1", "ITEM2", "ITEM3"]);
    }
}
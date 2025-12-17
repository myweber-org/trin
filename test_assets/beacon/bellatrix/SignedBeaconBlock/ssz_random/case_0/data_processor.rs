
use std::collections::HashMap;

pub struct DataProcessor {
    validators: Vec<Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: Vec::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator<F>(&mut self, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.push(Box::new(validator));
    }

    pub fn add_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.to_string(), Box::new(transformer));
    }

    pub fn validate(&self, input: &str) -> bool {
        self.validators.iter().all(|v| v(input))
    }

    pub fn transform(&self, name: &str, input: String) -> Option<String> {
        self.transformers.get(name).map(|t| t(input))
    }

    pub fn process(&self, input: &str) -> Result<Vec<String>, String> {
        if !self.validate(input) {
            return Err("Validation failed".to_string());
        }

        let mut results = Vec::new();
        for (name, transformer) in &self.transformers {
            if let Some(result) = self.transform(name, input.to_string()) {
                results.push(result);
            }
        }

        Ok(results)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validator(|s| !s.trim().is_empty());
    processor.add_validator(|s| s.len() <= 1000);

    processor.add_transformer("uppercase", |s| s.to_uppercase());
    processor.add_transformer("reverse", |s| s.chars().rev().collect());
    processor.add_transformer("trim", |s| s.trim().to_string());

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("test"));
        assert!(!processor.validate(""));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_full_process() {
        let processor = create_default_processor();
        let results = processor.process(" test ").unwrap();
        assert!(results.contains(&"TEST".to_string()));
        assert!(results.contains(&"tset ".to_string()));
        assert!(results.contains(&"test".to_string()));
    }
}
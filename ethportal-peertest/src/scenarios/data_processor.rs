
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn add_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        self.validators
            .get(name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process(&self, data: &str) -> Result<String, String> {
        if !self.validate("email", data) {
            return Err("Invalid email format".to_string());
        }

        let transformed = self.transform("uppercase", data.to_string())
            .ok_or("Transformation failed")?;

        Ok(transformed)
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        let mut processor = DataProcessor::new();
        
        processor.add_validator("email", Box::new(|s| s.contains('@')));
        processor.add_transformer("uppercase", Box::new(|s| s.to_uppercase()));
        processor.add_transformer("trim", Box::new(|s| s.trim().to_string()));
        
        processor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::default();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::default();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::default();
        let result = processor.process("user@domain.com");
        assert_eq!(result, Ok("USER@DOMAIN.COM".to_string()));
    }
}

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

    pub fn register_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn register_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        match self.validators.get(name) {
            Some(validator) => validator(data),
            None => false,
        }
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers.get(name).map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, operations: &[(&str, &str)]) -> Result<String, String> {
        let mut current_data = data;

        for (op_type, op_name) in operations {
            match *op_type {
                "validate" => {
                    if !self.validate(op_name, &current_data) {
                        return Err(format!("Validation '{}' failed for data: {}", op_name, current_data));
                    }
                }
                "transform" => {
                    current_data = match self.transform(op_name, current_data) {
                        Some(transformed) => transformed,
                        None => return Err(format!("Transformer '{}' not found", op_name)),
                    };
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }

        Ok(current_data)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("non_empty", Box::new(|data| !data.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|data| data.chars().all(|c| c.is_ascii_digit())));

    processor.register_transformer("uppercase", Box::new(|data| data.to_uppercase()));
    processor.register_transformer("trim", Box::new(|data| data.trim().to_string()));
    processor.register_transformer("reverse", Box::new(|data| data.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("non_empty", "test"));
        assert!(!processor.validate("non_empty", ""));
        assert!(processor.validate("is_numeric", "12345"));
        assert!(!processor.validate("is_numeric", "abc123"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("trim", "  test  ".to_string()), Some("test".to_string()));
        assert_eq!(processor.transform("reverse", "abc".to_string()), Some("cba".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let operations = [
            ("validate", "non_empty"),
            ("transform", "trim"),
            ("transform", "uppercase"),
        ];

        let result = processor.process_pipeline("  hello world  ".to_string(), &operations);
        assert_eq!(result, Ok("HELLO WORLD".to_string()));

        let invalid_result = processor.process_pipeline("".to_string(), &operations);
        assert!(invalid_result.is_err());
    }
}
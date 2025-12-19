
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
        self.validators
            .get(name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, steps: &[(&str, &str)]) -> Result<String, String> {
        let mut current_data = data;

        for (step_type, step_name) in steps {
            match *step_type {
                "validate" => {
                    if !self.validate(step_name, &current_data) {
                        return Err(format!("Validation failed at step: {}", step_name));
                    }
                }
                "transform" => {
                    current_data = self.transform(step_name, current_data)
                        .ok_or_else(|| format!("Transformer not found: {}", step_name))?;
                }
                _ => return Err(format!("Unknown step type: {}", step_type)),
            }
        }

        Ok(current_data)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("non_empty", Box::new(|s| !s.trim().is_empty()));
    processor.register_validator("is_numeric", Box::new(|s| s.parse::<f64>().is_ok()));
    
    processor.register_transformer("to_uppercase", Box::new(|s| s.to_uppercase()));
    processor.register_transformer("trim_spaces", Box::new(|s| s.trim().to_string()));
    processor.register_transformer("reverse_string", Box::new(|s| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("non_empty", "test"));
        assert!(!processor.validate("non_empty", "   "));
        assert!(processor.validate("is_numeric", "123.45"));
        assert!(!processor.validate("is_numeric", "abc"));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("to_uppercase", "hello".to_string()), Some("HELLO".to_string()));
        assert_eq!(processor.transform("trim_spaces", "  test  ".to_string()), Some("test".to_string()));
        assert_eq!(processor.transform("reverse_string", "abc".to_string()), Some("cba".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let steps = vec![
            ("validate", "non_empty"),
            ("transform", "to_uppercase"),
            ("transform", "reverse_string"),
        ];
        
        let result = processor.process_pipeline("hello".to_string(), &steps);
        assert_eq!(result, Ok("OLLEH".to_string()));
        
        let invalid_result = processor.process_pipeline("   ".to_string(), &steps);
        assert!(invalid_result.is_err());
    }
}

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
        match self.validators.get(name) {
            Some(validator) => validator(data),
            None => false,
        }
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers.get(name).map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: String, validators: &[&str], transformers: &[&str]) -> Option<String> {
        for validator_name in validators {
            if !self.validate(validator_name, &data) {
                return None;
            }
        }

        let mut result = data;
        for transformer_name in transformers {
            match self.transform(transformer_name, result) {
                Some(transformed) => result = transformed,
                None => return None,
            }
        }

        Some(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validator("is_numeric", Box::new(|s: &str| s.chars().all(|c| c.is_ascii_digit())));

    processor.add_validator("is_alpha", Box::new(|s: &str| s.chars().all(|c| c.is_ascii_alphabetic())));

    processor.add_transformer("to_uppercase", Box::new(|s: String| s.to_uppercase()));

    processor.add_transformer("to_lowercase", Box::new(|s: String| s.to_lowercase()));

    processor.add_transformer("trim_spaces", Box::new(|s: String| s.trim().to_string()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("is_numeric", "12345"));
        assert!(!processor.validate("is_numeric", "123a45"));
    }

    #[test]
    fn test_transformation_pipeline() {
        let processor = create_default_processor();
        let result = processor.process_pipeline(
            "  Hello World  ".to_string(),
            &["is_alpha"],
            &["trim_spaces", "to_uppercase"]
        );
        assert_eq!(result, Some("HELLO WORLD".to_string()));
    }

    #[test]
    fn test_failed_validation() {
        let processor = create_default_processor();
        let result = processor.process_pipeline(
            "123abc".to_string(),
            &["is_numeric"],
            &["to_uppercase"]
        );
        assert_eq!(result, None);
    }
}
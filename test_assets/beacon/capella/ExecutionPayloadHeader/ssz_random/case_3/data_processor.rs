
use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&str) -> bool>>,
    transformations: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformations: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformation<F>(&mut self, name: &str, transform: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformations
            .insert(name.to_string(), Box::new(transform));
    }

    pub fn process_data(&self, input: &str) -> Option<String> {
        if !self.filters.iter().all(|f| f(input)) {
            return None;
        }

        let mut result = input.to_string();
        for transform in self.transformations.values() {
            result = transform(result);
        }

        Some(result)
    }

    pub fn batch_process(&self, inputs: Vec<&str>) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_data(input))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_filter(|s| s.len() > 3);
        processor.add_transformation("uppercase", |s| s.to_uppercase());
        processor.add_transformation("trim", |s| s.trim().to_string());

        let result = processor.process_data("  test data  ");
        assert_eq!(result, Some("TEST DATA".to_string()));

        let batch_result = processor.batch_process(vec!["a", "valid data", "  another  "]);
        assert_eq!(batch_result, vec!["VALID DATA".to_string(), "ANOTHER".to_string()]);
    }
}
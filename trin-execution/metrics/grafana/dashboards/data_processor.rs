
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if let (Some(mean), Some(std_dev)) = (self.calculate_mean(), self.calculate_standard_deviation()) {
            self.data
                .iter()
                .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
                .cloned()
                .collect()
        } else {
            self.data.clone()
        }
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Data points: {}, Mean: {:.2}, Std Dev: {:.2}",
            self.data.len(),
            self.calculate_mean().unwrap_or(0.0),
            self.calculate_standard_deviation().unwrap_or(0.0)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n15.2\n12.8\n14.1\n11.9").unwrap();
        
        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.data.len(), 5);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 12.9).abs() < 0.1);
        
        let filtered = processor.filter_outliers(2.0);
        assert_eq!(filtered.len(), 5);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    field_name: String,
    rule_type: ValidationType,
    parameters: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationType {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Range(f64, f64),
}

#[derive(Debug, Clone)]
pub struct Transformation {
    name: String,
    function: fn(&mut HashMap<String, String>) -> Result<(), String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.insert(rule.field_name.clone(), rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process_data(&self, data: &mut HashMap<String, String>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (field, rule) in &self.validation_rules {
            if let Some(value) = data.get(field) {
                if let Err(e) = self.validate_field(value, rule) {
                    errors.push(format!("Field '{}': {}", field, e));
                }
            } else if matches!(rule.rule_type, ValidationType::Required) {
                errors.push(format!("Field '{}' is required", field));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        for transformation in &self.transformation_pipeline {
            if let Err(e) = (transformation.function)(data) {
                errors.push(format!("Transformation '{}' failed: {}", transformation.name, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_field(&self, value: &str, rule: &ValidationRule) -> Result<(), String> {
        match &rule.rule_type {
            ValidationType::Required => {
                if value.trim().is_empty() {
                    Err("Field is required".to_string())
                } else {
                    Ok(())
                }
            }
            ValidationType::MinLength(min) => {
                if value.len() >= *min {
                    Ok(())
                } else {
                    Err(format!("Minimum length is {}", min))
                }
            }
            ValidationType::MaxLength(max) => {
                if value.len() <= *max {
                    Ok(())
                } else {
                    Err(format!("Maximum length is {}", max))
                }
            }
            ValidationType::Pattern(pattern) => {
                let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
                if re.is_match(value) {
                    Ok(())
                } else {
                    Err("Pattern does not match".to_string())
                }
            }
            ValidationType::Range(min, max) => {
                if let Ok(num) = value.parse::<f64>() {
                    if num >= *min && num <= *max {
                        Ok(())
                    } else {
                        Err(format!("Value must be between {} and {}", min, max))
                    }
                } else {
                    Err("Invalid numeric value".to_string())
                }
            }
        }
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(ValidationRule {
        field_name: "username".to_string(),
        rule_type: ValidationType::MinLength(3),
        parameters: vec![],
    });

    processor.add_validation_rule(ValidationRule {
        field_name: "email".to_string(),
        rule_type: ValidationType::Pattern(r"^[^@\s]+@[^@\s]+\.[^@\s]+$".to_string()),
        parameters: vec![],
    });

    processor.add_transformation(Transformation {
        name: "trim_whitespace".to_string(),
        function: |data| {
            for value in data.values_mut() {
                *value = value.trim().to_string();
            }
            Ok(())
        },
    });

    processor
}
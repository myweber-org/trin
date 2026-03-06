
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        self.validate_data(data)?;
        
        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<(), String> {
        for value in data {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }

        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err(format!("Field '{}' is required but empty", rule.field_name));
            }
        }

        Ok(())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        let sum: f64 = data.iter().sum();
        sum / data.len() as f64
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            total_datasets: self.cache.len(),
            total_values: self.cache.values().map(|v| v.len()).sum(),
        }
    }
}

pub struct CacheStats {
    pub total_datasets: usize,
    pub total_values: usize,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;
            for field in record.iter() {
                if let Ok(value) = field.parse::<f64>() {
                    self.data.push(value);
                }
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
            let lower_bound = mean - threshold * std_dev;
            let upper_bound = mean + threshold * std_dev;

            self.data
                .iter()
                .filter(|&&x| x >= lower_bound && x <= upper_bound)
                .copied()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Data points: {}, Mean: {:.4}, Std Dev: {:.4}",
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5,20.3,15.7\n8.9,12.1,18.4").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data.len(), 6);
        
        let mean = processor.calculate_mean();
        assert!(mean.is_some());
        assert!((mean.unwrap() - 14.3167).abs() < 0.001);
        
        let filtered = processor.filter_outliers(2.0);
        assert!(!filtered.is_empty());
    }
}
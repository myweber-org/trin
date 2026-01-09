
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error for field '{}': {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    pub threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_input(&self, value: f64) -> Result<(), ValidationError> {
        if value.is_nan() {
            return Err(ValidationError {
                field: "input".to_string(),
                message: "Value cannot be NaN".to_string(),
            });
        }

        if value.is_infinite() {
            return Err(ValidationError {
                field: "input".to_string(),
                message: "Value cannot be infinite".to_string(),
            });
        }

        if value < 0.0 {
            return Err(ValidationError {
                field: "input".to_string(),
                message: "Value must be non-negative".to_string(),
            });
        }

        Ok(())
    }

    pub fn process_data(&self, data: &[f64]) -> Result<Vec<f64>, ValidationError> {
        let mut processed = Vec::with_capacity(data.len());

        for (i, &value) in data.iter().enumerate() {
            self.validate_input(value).map_err(|mut e| {
                e.field = format!("data[{}]", i);
                e
            })?;

            let transformed = if value > self.threshold {
                value.ln()
            } else {
                value.sqrt()
            };

            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64), ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                field: "data".to_string(),
                message: "Cannot calculate statistics for empty dataset".to_string(),
            });
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_input() {
        let processor = DataProcessor::new(10.0);
        assert!(processor.validate_input(5.0).is_ok());
        assert!(processor.validate_input(0.0).is_ok());
        assert!(processor.validate_input(15.0).is_ok());
    }

    #[test]
    fn test_validation_invalid_input() {
        let processor = DataProcessor::new(10.0);
        assert!(processor.validate_input(-5.0).is_err());
        assert!(processor.validate_input(f64::NAN).is_err());
        assert!(processor.validate_input(f64::INFINITY).is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(10.0);
        let data = vec![4.0, 16.0, 25.0];
        let result = processor.process_data(&data).unwrap();
        
        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 1e-10);
        assert!((result[1] - 2.772588722239781).abs() < 1e-10);
        assert!((result[2] - 3.2188758248682006).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(10.0);
        let data = vec![2.0, 4.0, 6.0, 8.0];
        let (mean, std_dev) = processor.calculate_statistics(&data).unwrap();
        
        assert!((mean - 5.0).abs() < 1e-10);
        assert!((std_dev - 2.23606797749979).abs() < 1e-10);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.3,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.7,CategoryA").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);

        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);

        let stats = processor.get_statistics();
        assert!(stats.0 > 0.0);
        assert!(stats.1 > 0.0);
        assert!(stats.2 > 0.0);
    }
}
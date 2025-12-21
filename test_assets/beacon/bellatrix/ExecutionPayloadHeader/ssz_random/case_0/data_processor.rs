
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
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
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2];

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e);
                }
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

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total_value() / self.records.len() as f64)
        }
    }

    pub fn find_max_value_record(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }

    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(1, 10.0, "test").unwrap();
        assert_eq!(record.calculate_adjusted_value(2.5), 25.0);
    }

    #[test]
    fn test_load_from_csv() {
        let mut csv_content = "id,value,category\n".to_string();
        csv_content.push_str("1,10.5,category_a\n");
        csv_content.push_str("2,20.3,category_b\n");
        csv_content.push_str("3,15.7,category_a\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.records.len(), 3);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "alpha").unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "beta").unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "alpha").unwrap());

        let filtered = processor.filter_by_category("alpha");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "alpha"));
    }

    #[test]
    fn test_calculate_total_value() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test").unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "test").unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "test").unwrap());

        assert_eq!(processor.calculate_total_value(), 60.0);
    }

    #[test]
    fn test_get_average_value() {
        let mut processor = DataProcessor::new();
        assert!(processor.get_average_value().is_none());

        processor.records.push(DataRecord::new(1, 10.0, "test").unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "test").unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "test").unwrap());

        assert_eq!(processor.get_average_value(), Some(20.0));
    }

    #[test]
    fn test_find_max_value_record() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test").unwrap());
        processor.records.push(DataRecord::new(2, 30.0, "test").unwrap());
        processor.records.push(DataRecord::new(3, 20.0, "test").unwrap());

        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 30.0);
    }
}
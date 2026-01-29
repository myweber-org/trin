use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].trim().to_string();

            if category.is_empty() {
                return Err(format!("Empty category at line {}", line_num + 1).into());
            }

            self.records.push(DataRecord::new(id, value, category));
            count += 1;
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

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_record_count(&self) -> usize {
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
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_load_valid_csv() {
        let mut csv_content = "1,42.5,category_a\n".to_string();
        csv_content.push_str("2,38.2,category_b\n");
        csv_content.push_str("3,55.9,category_a");

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", csv_content).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_record_count(), 3);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "alpha".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "beta".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "alpha".to_string()));

        let filtered = processor.filter_by_category("alpha");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "alpha"));
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), None);

        processor.records.push(DataRecord::new(1, 10.0, "test".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "test".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "test".to_string()));

        assert_eq!(processor.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_clear_records() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test".to_string()));
        assert_eq!(processor.get_record_count(), 1);

        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }
}
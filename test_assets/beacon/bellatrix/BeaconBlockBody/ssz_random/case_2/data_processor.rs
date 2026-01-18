
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        Self {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            if record.is_valid() {
                self.total_value += record.get_value();
                self.valid_count += 1;
            }
            self.records.push(record);
            line_count += 1;
        }

        Ok(line_count)
    }

    pub fn get_average_value(&self) -> f64 {
        if self.valid_count == 0 {
            0.0
        } else {
            self.total_value / self.valid_count as f64
        }
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn get_invalid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| !r.is_valid()).collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -5.0, "B".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 7.5, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.get_average_value(), 0.0);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,category")?;
        writeln!(temp_file, "1,10.5,TypeA")?;
        writeln!(temp_file, "2,15.0,TypeB")?;
        writeln!(temp_file, "3,-5.0,TypeC")?;

        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(count, 3);
        assert_eq!(processor.get_valid_records().len(), 2);
        assert_eq!(processor.get_invalid_records().len(), 1);
        assert_eq!(processor.get_average_value(), 12.75);
        
        Ok(())
    }
}
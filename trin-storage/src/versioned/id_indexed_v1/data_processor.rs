
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
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

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].trim();

            let record = DataRecord::new(id, value, category);
            self.total_value += record.value;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.total_value / self.records.len() as f64
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,type_a").unwrap();
        writeln!(temp_file, "2,200.0,type_b").unwrap();
        writeln!(temp_file, "3,300.75,type_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.get_valid_records().len(), 3);
        
        let type_a_records = processor.filter_by_category("type_a");
        assert_eq!(type_a_records.len(), 2);
        
        let avg = processor.get_average_value();
        assert!(avg > 200.0 && avg < 201.0);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(1, -10.0, "invalid");
        assert!(!record.valid);
        
        let record2 = DataRecord::new(2, 1500.0, "invalid");
        assert!(!record2.valid);
        
        let record3 = DataRecord::new(3, 500.0, "valid");
        assert!(record3.valid);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: DataRecord = result?;
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "A".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, -1.0, "".to_string());
        assert!(!record.is_valid());
    }

    #[test]
    fn test_load_and_process() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,category")?;
        writeln!(temp_file, "1,10.5,Alpha")?;
        writeln!(temp_file, "2,20.5,Beta")?;
        writeln!(temp_file, "3,30.5,Gamma")?;

        let records = load_csv_data(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 3);

        let avg = calculate_average(&records).unwrap();
        assert!((avg - 20.5).abs() < 0.001);

        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub is_valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let is_valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            is_valid,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), String> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
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
            if let Err(e) = self.add_record(record) {
                eprintln!("Warning: Skipping invalid record at line {}: {}", line_num + 1, e);
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter()
            .filter(|r| r.is_valid)
            .map(|r| r.value)
            .sum();
        
        let count = self.records.iter()
            .filter(|r| r.is_valid)
            .count();
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|r| r.category == category && r.is_valid)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn valid_records(&self) -> usize {
        self.records.iter().filter(|r| r.is_valid).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test");
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, -1.0, "");
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 10.0, "A");
        let record2 = DataRecord::new(2, 20.0, "B");
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert_eq!(processor.total_records(), 2);
        assert_eq!(processor.valid_records(), 2);
        
        let avg = processor.calculate_average();
        assert_eq!(avg, Some(15.0));
    }
}
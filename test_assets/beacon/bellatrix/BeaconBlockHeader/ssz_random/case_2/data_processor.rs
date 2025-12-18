
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

    pub fn to_csv(&self) -> String {
        format!("{},{},{},{}", self.id, self.value, self.category, self.valid)
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

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 3 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let value = parts[1].parse::<f64>().unwrap_or(0.0);
                let category = parts[2].to_string();
                
                let record = DataRecord::new(id, value, &category);
                self.add_record(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn export_valid_records<P: AsRef<Path>>(&self, path: P) -> Result<usize, Box<dyn Error>> {
        let mut file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(&mut file);
        
        writeln!(writer, "id,value,category,valid")?;
        
        let valid_records = self.filter_valid();
        for record in &valid_records {
            writeln!(writer, "{}", record.to_csv())?;
        }
        
        Ok(valid_records.len())
    }

    pub fn get_statistics(&self) -> (usize, usize, Option<f64>) {
        let total = self.records.len();
        let valid_count = self.filter_valid().len();
        let average = self.calculate_average();
        
        (total, valid_count, average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "B");
        assert!(!record.valid);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 100.0, "Test"));
        processor.add_record(DataRecord::new(2, 200.0, "Test"));
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 2);
        assert_eq!(stats.1, 2);
        assert!(stats.2.is_some());
        assert_eq!(stats.2.unwrap(), 150.0);
    }

    #[test]
    fn test_file_operations() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,category")?;
        writeln!(temp_file, "1,100.0,CategoryA")?;
        writeln!(temp_file, "2,200.0,CategoryB")?;
        writeln!(temp_file, "3,-50.0,CategoryC")?;
        
        let mut processor = DataProcessor::new();
        let count = processor.load_from_file(temp_file.path())?;
        
        assert_eq!(count, 3);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 2);
        
        Ok(())
    }
}
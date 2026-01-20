
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut count = 0;
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= 0.0 && !record.category.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);
        
        let avg = processor.calculate_average();
        assert!(avg.is_none());
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,category")?;
        writeln!(temp_file, "1,10.5,TypeA")?;
        writeln!(temp_file, "2,20.3,TypeB")?;
        
        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path().to_str().unwrap())?;
        
        assert_eq!(count, 2);
        assert_eq!(processor.get_record_count(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.4).abs() < 0.0001);
        
        Ok(())
    }
}
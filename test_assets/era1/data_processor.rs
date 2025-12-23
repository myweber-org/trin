
use csv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > 0.0 && !record.name.is_empty())
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn export_valid_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let valid_records = self.validate_records();
        let mut wtr = csv::Writer::from_path(output_path)?;
        
        for record in valid_records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let total = self.calculate_total();
        let average = if count > 0 { total / count as f64 } else { 0.0 };
        
        (count, total, average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,active").unwrap();
        writeln!(file, "1,ItemA,10.5,true").unwrap();
        writeln!(file, "2,ItemB,20.0,false").unwrap();
        writeln!(file, "3,,15.0,true").unwrap();
        file
    }

    #[test]
    fn test_load_and_validate() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        assert!(processor.load_from_csv(test_file.path().to_str().unwrap()).is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let valid = processor.validate_records();
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn test_calculations() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_from_csv(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.calculate_total(), 45.5);
        assert!(processor.find_by_id(1).is_some());
        assert!(processor.find_by_id(99).is_none());
    }
}
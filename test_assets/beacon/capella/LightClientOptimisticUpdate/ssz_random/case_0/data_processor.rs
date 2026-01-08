use csv::Reader;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
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
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold && record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        serde_json::to_writer_pretty(file, &self.records)?;
        Ok(())
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let active_count = self.records.iter().filter(|r| r.active).count();
        let max_value = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
        let min_value = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        
        Statistics {
            total_records: count,
            active_records: active_count,
            max_value,
            min_value,
            average_value: self.calculate_average().unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Statistics {
    total_records: usize,
    active_records: usize,
    max_value: f64,
    min_value: f64,
    average_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let csv_data = "id,name,value,active\n1,ItemA,10.5,true\n2,ItemB,5.2,false\n3,ItemC,15.8,true\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let filtered = processor.filter_by_value(10.0);
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.active_records, 2);
        assert_eq!(stats.max_value, 15.8);
        assert_eq!(stats.min_value, 5.2);
    }
}

use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
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

    pub fn export_active_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(output_path)?;

        for record in self.records.iter().filter(|r| r.active) {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, "A".to_string(), 10.5, true);
        let record2 = DataRecord::new(2, "B".to_string(), 20.0, false);
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.get_record_count(), 2);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 15.25);
    }

    #[test]
    fn test_export_active() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, "Test".to_string(), 100.0, true));
        processor.add_record(DataRecord::new(2, "Test".to_string(), 200.0, false));
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path().to_str().unwrap();
        
        processor.export_active_records(path)?;
        
        Ok(())
    }
}
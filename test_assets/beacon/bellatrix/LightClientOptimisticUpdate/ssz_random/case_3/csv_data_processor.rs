
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

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_to_csv(&self, file_path: &str, records: &[DataRecord]) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for record in records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

pub fn process_data_file(input_path: &str, output_path: &str, category_filter: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_csv(input_path)?;
    
    let filtered = processor.filter_by_category(category_filter);
    
    if !filtered.is_empty() {
        processor.export_to_csv(output_path, &filtered)?;
        println!("Exported {} records to {}", filtered.len(), output_path);
        
        if let Some(avg) = processor.calculate_average() {
            println!("Average value: {:.2}", avg);
        }
        
        if let Some(max_record) = processor.find_max_value() {
            println!("Maximum value record: ID {}", max_record.id);
        }
    } else {
        println!("No records found for category: {}", category_filter);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, "test".to_string(), 100.0, true);
        assert_eq!(record.id, 1);
        assert_eq!(record.category, "test");
        assert_eq!(record.value, 100.0);
        assert!(record.active);
    }
    
    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "A".to_string(), 10.0, true));
        processor.records.push(DataRecord::new(2, "B".to_string(), 20.0, true));
        processor.records.push(DataRecord::new(3, "A".to_string(), 30.0, false));
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
    
    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "test".to_string(), 10.0, true));
        processor.records.push(DataRecord::new(2, "test".to_string(), 20.0, true));
        
        assert_eq!(processor.calculate_average(), Some(15.0));
    }
}
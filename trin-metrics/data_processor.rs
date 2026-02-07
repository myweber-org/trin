
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
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
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,ItemA,10.5,Alpha").unwrap();
        writeln!(file, "2,ItemB,-3.2,Beta").unwrap();
        writeln!(file, "3,ItemC,7.8,Alpha").unwrap();
        writeln!(file, "4,,15.0,Gamma").unwrap();
        file
    }

    #[test]
    fn test_load_and_validate() {
        let mut processor = DataProcessor::new();
        let test_file = create_test_csv();
        
        assert!(processor.load_from_csv(test_file.path()).is_ok());
        assert_eq!(processor.count_records(), 4);
        
        let valid = processor.validate_records();
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        let test_file = create_test_csv();
        
        processor.load_from_csv(test_file.path()).unwrap();
        let avg = processor.calculate_average().unwrap();
        let expected = (10.5 + (-3.2) + 7.8 + 15.0) / 4.0;
        assert!((avg - expected).abs() < 0.0001);
    }

    #[test]
    fn test_category_filter() {
        let mut processor = DataProcessor::new();
        let test_file = create_test_csv();
        
        processor.load_from_csv(test_file.path()).unwrap();
        let alpha_items = processor.filter_by_category("Alpha");
        assert_eq!(alpha_items.len(), 2);
    }
}
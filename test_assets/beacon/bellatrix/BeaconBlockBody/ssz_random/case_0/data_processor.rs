
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.name.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self
            .records
            .iter()
            .map(|record| record.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,15.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,20.0,Electronics").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 15.166666666666666).abs() < 0.0001);
        
        let (count, _, max) = processor.get_statistics();
        assert_eq!(count, 3);
        assert_eq!(max, 20.0);
    }
}

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

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        let count = values.len();
        let min = values.iter().copied().reduce(f64::min);
        let max = values.iter().copied().reduce(f64::max);
        
        (count, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n\
                        1,ItemA,10.5,Category1\n\
                        2,ItemB,15.2,Category2\n\
                        3,ItemC,8.7,Category1\n\
                        4,ItemD,-5.0,Category2\n\
                        5,,20.1,Category3";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.validate_records().len(), 4);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 13.625).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("Category1").unwrap().len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 5);
        assert_eq!(stats.1, Some(-5.0));
        assert_eq!(stats.2, Some(20.1));
    }
}
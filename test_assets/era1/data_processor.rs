
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let unique_categories = self.frequency_map.len();
        
        format!(
            "Data Summary:\nTotal records: {}\nUnique categories: {}\nMean value: {:.2}",
            count, unique_categories, mean
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        writeln!(temp_file, "C,8.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(13.85));
        assert_eq!(processor.data.len(), 4);
        assert_eq!(processor.get_frequency_distribution().get("A"), Some(&2));
    }
}
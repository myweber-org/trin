
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();
            
            if !self.validate_record(&name, value) {
                continue;
            }
            
            let record = CsvRecord {
                id,
                name,
                value,
                category,
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    fn validate_record(&self, name: &str, value: f64) -> bool {
        !name.is_empty() && value >= 0.0
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total_value() / self.records.len() as f64)
        }
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut processor = CsvProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,50.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,75.25,Electronics").unwrap();
        
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let total = processor.calculate_total_value();
        assert!((total - 225.75).abs() < 0.001);
        
        let avg = processor.calculate_average_value();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 75.25).abs() < 0.001);
    }
}
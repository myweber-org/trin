use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        if let Some(header) = lines.next() {
            let headers: Vec<String> = header?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            for line in lines {
                let line = line?;
                let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        record.insert(header.clone(), values[i].to_string());
                    }
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self, column: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for record in &self.records {
            if let Some(value) = record.get(column) {
                if let Ok(num) = value.parse::<f64>() {
                    sum += num;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn get_unique_values(&self, column: &str) -> Vec<String> {
        let mut unique_values = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column) {
                if !seen.contains(value) {
                    seen.insert(value.clone());
                    unique_values.push(value.clone());
                }
            }
        }
        
        unique_values
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,88.0").unwrap();
        writeln!(temp_file, "Charlie,25,92.3").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = DataProcessor::new();
        processor.load_csv(file_path).unwrap();
        
        assert_eq!(processor.record_count(), 3);
        
        let avg_age = processor.calculate_average("age").unwrap();
        assert!((avg_age - 26.6667).abs() < 0.001);
        
        let unique_ages = processor.get_unique_values("age");
        assert_eq!(unique_ages.len(), 2);
        
        let filtered = processor.filter_records(|record| {
            record.get("age").unwrap() == "25"
        });
        assert_eq!(filtered.len(), 2);
    }
}
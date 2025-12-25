use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_data = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > filter_column && columns[filter_column] == filter_value {
                filtered_data.push(columns);
            }
        }

        Ok(filtered_data)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut sum = 0.0;
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if let Some(value_str) = columns.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,25,New York").unwrap();
        writeln!(temp_file, "Bob,30,London").unwrap();
        writeln!(temp_file, "Charlie,25,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(2, "Paris").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["Charlie", "25", "Paris"]);
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,score").unwrap();
        writeln!(temp_file, "Alice,85.5").unwrap();
        writeln!(temp_file, "Bob,92.0").unwrap();
        writeln!(temp_file, "Charlie,78.5").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();

        assert!((average - 85.333).abs() < 0.001);
    }
}use csv::Reader;
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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.find_max_value().map(|r| r.value);
        
        (count, avg, max)
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
        writeln!(file, "2,ItemB,15.2,Beta").unwrap();
        writeln!(file, "3,ItemC,8.7,Alpha").unwrap();
        writeln!(file, "4,ItemD,20.1,Beta").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(test_file.path().to_str().unwrap()).unwrap();
        
        let alpha_records = processor.filter_by_category("Alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let beta_records = processor.filter_by_category("Beta");
        assert_eq!(beta_records.len(), 2);
    }

    #[test]
    fn test_calculations() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(test_file.path().to_str().unwrap()).unwrap();
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 13.625).abs() < 0.001);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.value, 20.1);
        assert_eq!(max_record.name, "ItemD");
    }
}
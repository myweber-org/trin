use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn get_min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).min_by(|a, b| a.partial_cmp(b).unwrap())
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.5).abs() < 0.01);

        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);

        assert_eq!(processor.get_max_value(), Some(20.3));
        assert_eq!(processor.get_min_value(), Some(10.5));
    }
}
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

    pub fn process_with_filter(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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
            
            if columns.len() > column_index {
                if let Ok(value) = columns[column_index].parse::<f64>() {
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

pub fn validate_data_format(data: &[Vec<String>], expected_columns: usize) -> bool {
    data.iter().all(|row| row.len() == expected_columns)
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter: ',',
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn process_data<F>(&self, filter_predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if filter_predicate(&fields) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let count = reader.lines().count();
        Ok(count)
    }
}

pub fn validate_numeric_field(fields: &[String], index: usize) -> bool {
    if let Some(field) = fields.get(index) {
        field.parse::<f64>().is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        
        let adults = processor.process_data(|fields| {
            if fields.len() >= 2 {
                if let Ok(age) = fields[1].parse::<i32>() {
                    return age >= 30;
                }
            }
            false
        }).unwrap();

        assert_eq!(adults.len(), 2);
        assert_eq!(adults[0][0], "Alice");
        assert_eq!(adults[1][0], "Charlie");
    }

    #[test]
    fn test_record_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "data1,data2").unwrap();
        writeln!(temp_file, "data3,data4").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let count = processor.count_records().unwrap();
        
        assert_eq!(count, 3);
    }
}
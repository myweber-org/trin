use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub fn process_csv_file(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
        
        let value = parts[1].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
        
        let category = parts[2].trim().to_string();

        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
        }
    }

    if records.is_empty() {
        return Err("No valid records found in file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        
        let invalid_record = DataRecord::new(2, -10.0, "test".to_string());
        assert!(invalid_record.is_err());
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.5,category_a").unwrap();
        writeln!(temp_file, "2,200.0,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        
        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].calculate_adjusted_value(2.0), 201.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 10.0, "a".to_string()).unwrap(),
            DataRecord::new(2, 20.0, "b".to_string()).unwrap(),
            DataRecord::new(3, 30.0, "c".to_string()).unwrap(),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            // Skip header and empty lines
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();
            
            self.records.push(DataRecord { id, value, category });
            count += 1;
        }
        
        Ok(count)
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
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
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
        let mut processor = DataProcessor::new();
        
        // Create temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        // Test loading
        let count = processor.load_from_csv(file_path).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.record_count(), 3);
        
        // Test filtering
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        // Test calculations
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.1);
        
        // Test max value
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);
        assert!((max_record.value - 20.3).abs() < 0.1);
    }
}use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            statistics: HashMap::new(),
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

    pub fn calculate_statistics(&mut self) {
        let mut category_sums: HashMap<String, f64> = HashMap::new();
        let mut category_counts: HashMap<String, u32> = HashMap::new();
        
        for record in &self.records {
            let sum = category_sums.entry(record.category.clone()).or_insert(0.0);
            *sum += record.value;
            
            let count = category_counts.entry(record.category.clone()).or_insert(0);
            *count += 1;
        }
        
        for (category, sum) in category_sums {
            if let Some(&count) = category_counts.get(&category) {
                if count > 0 {
                    self.statistics.insert(category, sum / count as f64);
                }
            }
        }
    }

    pub fn get_average(&self, category: &str) -> Option<f64> {
        self.statistics.get(category).copied()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
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
        writeln!(temp_file, "id,category,value,timestamp").unwrap();
        writeln!(temp_file, "1,electronics,299.99,2024-01-15").unwrap();
        writeln!(temp_file, "2,clothing,49.95,2024-01-16").unwrap();
        writeln!(temp_file, "3,electronics,599.99,2024-01-17").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.total_records(), 3);
        
        processor.calculate_statistics();
        
        let electronics_avg = processor.get_average("electronics");
        assert!(electronics_avg.is_some());
        assert!((electronics_avg.unwrap() - 449.99).abs() < 0.01);
        
        let filtered = processor.filter_by_category("electronics");
        assert_eq!(filtered.len(), 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No valid records found".to_string());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    i + 1,
                    record.len(),
                    expected_len
                ));
            }
        }

        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if records.is_empty() {
            return Err("No records available".to_string());
        }

        if column_index >= records[0].len() {
            return Err(format!(
                "Column index {} out of bounds (max {})",
                column_index,
                records[0].len() - 1
            ));
        }

        let column_data: Vec<String> = records
            .iter()
            .map(|record| record[column_index].clone())
            .collect();

        Ok(column_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let processor = DataProcessor::new(',', false);
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        assert!(processor.validate_records(&valid_records).is_ok());
    }

    #[test]
    fn test_extract_column() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let column = processor.extract_column(&records, 1).unwrap();
        assert_eq!(column, vec!["b".to_string(), "d".to_string()]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.has_header && index == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in data {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
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
        writeln!(temp_file, "Charlie,35,92.3").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        assert!(processor.validate_record(&result[0]));
        
        let avg_score = processor.calculate_statistics(&result, 2);
        assert!(avg_score.is_some());
        assert!((avg_score.unwrap() - 91.933).abs() < 0.001);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available for statistics".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "John,30,50000").unwrap();
        writeln!(temp_file, "Jane,25,45000").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "50000"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let (mean, std_dev) = processor.calculate_statistics(&data, 0).unwrap();
        
        assert!((mean - 12.666666666666666).abs() < 0.0001);
        assert!((std_dev - 2.054804667).abs() < 0.0001);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        let valid_record = vec!["data".to_string(), "value".to_string()];
        assert!(processor.validate_record(&valid_record));
        
        let invalid_record = vec!["".to_string(), "value".to_string()];
        assert!(!processor.validate_record(&invalid_record));
    }
}
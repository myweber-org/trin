
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
            
            if line_number == 0 && self.has_header {
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

    fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to process".into());
        }

        let mut values = Vec::new();
        for record in records {
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
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
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
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
        
        let stats = processor.calculate_statistics(&records, 2).unwrap();
        assert!((stats.0 - 50000.0).abs() < 0.01);
    }

    #[test]
    fn test_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,thirty").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        let result = processor.calculate_statistics(&records, 1);
        
        assert!(result.is_err());
    }
}
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".into());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

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
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();

        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
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
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                self.parse_header(&line);
                continue;
            }
            
            if let Some(value) = self.parse_value(&line) {
                self.data.push(value);
            }
        }
        
        Ok(())
    }

    fn parse_header(&mut self, header_line: &str) {
        let parts: Vec<&str> = header_line.split(',').collect();
        if parts.len() >= 2 {
            self.metadata.insert("source".to_string(), parts[0].to_string());
            self.metadata.insert("unit".to_string(), parts[1].to_string());
        }
    }

    fn parse_value(&self, line: &str) -> Option<f64> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.is_empty() {
            return None;
        }
        
        match parts[0].trim().parse::<f64>() {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    pub fn calculate_statistics(&self) -> Statistics {
        if self.data.is_empty() {
            return Statistics::default();
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len();
        let mean = sum / count as f64;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Statistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        }
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn filter_data<F>(&self, predicate: F) -> Vec<f64>
    where
        F: Fn(f64) -> bool,
    {
        self.data.iter()
            .filter(|&&x| predicate(x))
            .copied()
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics: count={}, mean={:.2}, std_dev={:.2}, min={:.2}, max={:.2}, sum={:.2}",
            self.count, self.mean, self.std_dev, self.min, self.max, self.sum
        )
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, f64>>,
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

        if let Some(header_result) = lines.next() {
            let header = header_result?;
            let columns: Vec<&str> = header.split(',').collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<&str> = line.split(',').collect();
                
                if values.len() != columns.len() {
                    continue;
                }

                let mut record = HashMap::new();
                for (i, column) in columns.iter().enumerate() {
                    if let Ok(num) = values[i].parse::<f64>() {
                        record.insert(column.to_string(), num);
                    }
                }

                if !record.is_empty() {
                    self.records.push(record);
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column_name).copied())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }

    pub fn filter_records(&self, column_name: &str, threshold: f64) -> Vec<HashMap<String, f64>> {
        self.records
            .iter()
            .filter(|record| {
                record.get(column_name)
                    .map(|&value| value > threshold)
                    .unwrap_or(false)
            })
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
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,score").unwrap();
        writeln!(temp_file, "1,10.5,0.8").unwrap();
        writeln!(temp_file, "2,15.2,0.9").unwrap();
        writeln!(temp_file, "3,8.7,0.7").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let stats = processor.calculate_statistics("value");
        assert!(stats.is_some());
        
        let filtered = processor.filter_records("value", 10.0);
        assert_eq!(filtered.len(), 2);
    }
}
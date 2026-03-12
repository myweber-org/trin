
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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
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

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.is_empty() || record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
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
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1]);
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1);
        
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}
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

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }
    
    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, average, active_count)
}

pub fn filter_records(records: Vec<Record>, min_value: f64) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.value >= min_value)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_csv() {
        let csv_data = "id,name,value,active\n1,test1,10.5,true\n2,test2,-5.0,false\n3,test3,20.0,true\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "test1");
        assert_eq!(records[1].name, "test3");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, avg, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 5.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 15.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 25.0, active: true },
        ];
        
        let filtered = filter_records(records, 10.0);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 2);
        assert_eq!(filtered[1].id, 3);
    }
}
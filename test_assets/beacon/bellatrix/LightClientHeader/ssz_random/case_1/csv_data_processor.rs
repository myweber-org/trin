use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[&Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "ItemA".to_string(),
                value: 10.5,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "ItemB".to_string(),
                value: 25.0,
                category: "Books".to_string(),
            },
        ];

        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "ItemA");
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                value: 10.0,
                category: "Test".to_string(),
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                value: 20.0,
                category: "Test".to_string(),
            },
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs).unwrap();
        assert_eq!(avg, 15.0);
    }
}
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

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn read_csv_file(file_path: &Path) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let id = parts[0].parse::<u32>().unwrap_or(0);
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>().unwrap_or(0.0);
        let category = parts[3].to_string();

        let record = CsvRecord::new(id, name, value, category);
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<CsvRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn process_records(records: &mut [CsvRecord], multiplier: f64) {
    for record in records.iter_mut() {
        record.transform_value(multiplier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = CsvRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();

        let records = read_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
            CsvRecord::new(3, "C".to_string(), 30.0, "X".to_string()),
        ];

        let filtered = filter_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
            CsvRecord::new(3, "C".to_string(), 30.0, "Z".to_string()),
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
        ];

        process_records(&mut records, 2.0);
        assert_eq!(records[0].value, 20.0);
        assert_eq!(records[1].value, 40.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.trim().to_string()).collect(),
            _ => return Err("Empty CSV file".into()),
        };
        
        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }
        
        Ok(CsvProcessor { headers, records })
    }
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        self.records.iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, group_by: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let group_index = match self.headers.iter().position(|h| h == group_by) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };
        
        let agg_index = match self.headers.iter().position(|h| h == aggregate_column) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };
        
        let mut result = HashMap::new();
        for record in &self.records {
            if let (Some(group_val), Some(agg_val)) = (record.get(group_index), record.get(agg_index)) {
                if let Ok(num) = agg_val.parse::<f64>() {
                    *result.entry(group_val.clone()).or_insert(0.0) += num;
                }
            }
        }
        
        result
    }
    
    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let csv_data = "name,age,salary\nAlice,30,50000\nBob,25,45000\nAlice,35,55000";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.get_headers(), &["name", "age", "salary"]);
        
        let filtered = processor.filter_by_column("name", |name| name == "Alice");
        assert_eq!(filtered.len(), 2);
        
        let aggregated = processor.aggregate_numeric_column("name", "salary");
        assert_eq!(aggregated.get("Alice"), Some(&105000.0));
        assert_eq!(aggregated.get("Bob"), Some(&45000.0));
    }
}
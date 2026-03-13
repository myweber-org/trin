use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line_number == 1 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    Ok(records)
}

pub fn filter_active_records(records: &[Record]) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.active)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn transform_records(records: Vec<Record>) -> Vec<Record> {
    records
        .into_iter()
        .map(|mut r| {
            if r.value > 100.0 {
                r.value *= 0.9;
            }
            r
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,50.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,150.0,false").unwrap();
        
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 150.0);
    }

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let active = filter_active_records(&records);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|r| r.active));
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        assert_eq!(calculate_total_value(&records), 60.0);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 30.0, active: true },
            Record { id: 3, name: "C".to_string(), value: 20.0, active: true },
        ];
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 30.0);
    }

    #[test]
    fn test_transform_records() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 50.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 150.0, active: true },
        ];
        
        let transformed = transform_records(records);
        assert_eq!(transformed[0].value, 50.0);
        assert_eq!(transformed[1].value, 135.0);
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
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }
        
        Ok(CsvProcessor { headers, records })
    }
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);
        
        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();
        
        if numeric_values.is_empty() {
            return None;
        }
        
        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }
    
    pub fn group_by_column(&self, group_column: &str, agg_column: &str) -> HashMap<String, f64> {
        let group_idx = self.headers.iter()
            .position(|h| h == group_column);
        let agg_idx = self.headers.iter()
            .position(|h| h == agg_column);
        
        let mut result = HashMap::new();
        
        if let (Some(group_idx), Some(agg_idx)) = (group_idx, agg_idx) {
            for record in &self.records {
                if let (Some(group_val), Ok(agg_val)) = (
                    record.get(group_idx),
                    record.get(agg_idx).and_then(|v| v.parse::<f64>().ok())
                ) {
                    *result.entry(group_val.clone()).or_insert(0.0) += agg_val;
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
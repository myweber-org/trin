use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.to_string()).collect(),
            _ => return Err("Failed to read headers".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(index) => index,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<(usize, Vec<String>)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        let mut unique_values = Vec::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                if !unique_values.contains(value) {
                    unique_values.push(value.clone());
                }
            }
        }

        Some((unique_values.len(), unique_values))
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    records: Vec<HashMap<String, String>>,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line.split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if values.len() != headers.len() {
                continue;
            }

            let mut record = HashMap::new();
            for (i, header) in headers.iter().enumerate() {
                record.insert(header.clone(), values[i].to_string());
            }
            records.push(record);
        }

        Ok(CsvAnalyzer { headers, records })
    }

    pub fn column_stats(&self, column_name: &str) -> Option<ColumnStats> {
        let values: Vec<f64> = self.records.iter()
            .filter_map(|record| record.get(column_name))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len();
        let mean = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();

        Some(ColumnStats {
            count,
            mean,
            std_dev,
            min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<&HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records.iter()
            .filter(|record| predicate(record))
            .collect()
    }

    pub fn unique_values(&self, column_name: &str) -> Vec<String> {
        let mut values: Vec<String> = self.records.iter()
            .filter_map(|record| record.get(column_name))
            .cloned()
            .collect();
        
        values.sort();
        values.dedup();
        values
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}

pub struct ColumnStats {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Count: {}, Mean: {:.2}, StdDev: {:.2}, Min: {:.2}, Max: {:.2}",
               self.count, self.mean, self.std_dev, self.min, self.max)
    }
}
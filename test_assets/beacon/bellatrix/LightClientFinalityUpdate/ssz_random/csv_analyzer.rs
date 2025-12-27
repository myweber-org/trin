use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
    pub numeric_columns: HashMap<String, Vec<f64>>,
    pub text_columns: HashMap<String, Vec<String>>,
}

impl CsvStats {
    pub fn new() -> Self {
        CsvStats {
            row_count: 0,
            column_count: 0,
            column_names: Vec::new(),
            numeric_columns: HashMap::new(),
            text_columns: HashMap::new(),
        }
    }

    pub fn analyze_file(path: &str, has_header: bool) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = CsvStats::new();
        let mut lines = reader.lines();

        if has_header {
            if let Some(header) = lines.next() {
                stats.column_names = header?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                stats.column_count = stats.column_names.len();
            }
        }

        for line_result in lines {
            let line = line_result?;
            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if stats.column_count == 0 {
                stats.column_count = values.len();
                stats.column_names = (0..stats.column_count)
                    .map(|i| format!("Column_{}", i + 1))
                    .collect();
            }

            if values.len() == stats.column_count {
                stats.row_count += 1;
                
                for (i, value) in values.iter().enumerate() {
                    let col_name = &stats.column_names[i];
                    
                    if let Ok(num) = value.parse::<f64>() {
                        stats.numeric_columns
                            .entry(col_name.clone())
                            .or_insert_with(Vec::new)
                            .push(num);
                    } else {
                        stats.text_columns
                            .entry(col_name.clone())
                            .or_insert_with(Vec::new)
                            .push(value.to_string());
                    }
                }
            }
        }

        Ok(stats)
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<ColumnSummary> {
        if let Some(numbers) = self.numeric_columns.get(column_name) {
            if numbers.is_empty() {
                return None;
            }

            let sum: f64 = numbers.iter().sum();
            let count = numbers.len();
            let mean = sum / count as f64;
            let min = numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            Some(ColumnSummary::Numeric {
                count,
                mean,
                min,
                max,
                sum,
            })
        } else if let Some(texts) = self.text_columns.get(column_name) {
            let unique_count = texts.iter().collect::<std::collections::HashSet<_>>().len();
            let sample = texts.iter().take(3).cloned().collect();
            
            Some(ColumnSummary::Text {
                count: texts.len(),
                unique_count,
                sample,
            })
        } else {
            None
        }
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<usize>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        let mut matching_rows = Vec::new();
        
        for row_idx in 0..self.row_count {
            let mut row_data = HashMap::new();
            
            for col_name in &self.column_names {
                if let Some(numbers) = self.numeric_columns.get(col_name) {
                    if row_idx < numbers.len() {
                        row_data.insert(col_name.clone(), numbers[row_idx].to_string());
                    }
                } else if let Some(texts) = self.text_columns.get(col_name) {
                    if row_idx < texts.len() {
                        row_data.insert(col_name.clone(), texts[row_idx].clone());
                    }
                }
            }
            
            if predicate(&row_data) {
                matching_rows.push(row_idx);
            }
        }
        
        matching_rows
    }
}

#[derive(Debug)]
pub enum ColumnSummary {
    Numeric {
        count: usize,
        mean: f64,
        min: f64,
        max: f64,
        sum: f64,
    },
    Text {
        count: usize,
        unique_count: usize,
        sample: Vec<String>,
    },
}

pub fn find_duplicate_rows(stats: &CsvStats) -> HashMap<String, Vec<usize>> {
    let mut row_signatures: HashMap<String, Vec<usize>> = HashMap::new();
    
    for row_idx in 0..stats.row_count {
        let mut signature_parts = Vec::new();
        
        for col_name in &stats.column_names {
            if let Some(numbers) = stats.numeric_columns.get(col_name) {
                if row_idx < numbers.len() {
                    signature_parts.push(format!("{}:{}", col_name, numbers[row_idx]));
                }
            } else if let Some(texts) = stats.text_columns.get(col_name) {
                if row_idx < texts.len() {
                    signature_parts.push(format!("{}:{}", col_name, texts[row_idx]));
                }
            }
        }
        
        let signature = signature_parts.join("|");
        row_signatures
            .entry(signature)
            .or_insert_with(Vec::new)
            .push(row_idx);
    }
    
    row_signatures.retain(|_, rows| rows.len() > 1);
    row_signatures
}
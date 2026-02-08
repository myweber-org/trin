use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    pub fn analyze_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = CsvStats::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                stats.column_names = line.split(',').map(|s| s.trim().to_string()).collect();
                stats.column_count = stats.column_names.len();
                continue;
            }

            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if values.len() != stats.column_count {
                return Err(format!("Row {} has {} columns, expected {}", 
                    index + 1, values.len(), stats.column_count).into());
            }

            stats.row_count += 1;

            for (col_index, value) in values.iter().enumerate() {
                let column_name = &stats.column_names[col_index];
                
                if let Ok(num) = value.parse::<f64>() {
                    stats.numeric_columns
                        .entry(column_name.clone())
                        .or_insert_with(Vec::new)
                        .push(num);
                } else {
                    stats.text_columns
                        .entry(column_name.clone())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
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
            
            let mut sorted = numbers.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count % 2 == 0 {
                (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
            } else {
                sorted[count / 2]
            };

            let min = *sorted.first().unwrap();
            let max = *sorted.last().unwrap();

            Some(ColumnSummary::Numeric {
                count,
                mean,
                median,
                min,
                max,
                sum,
            })
        } else if let Some(texts) = self.text_columns.get(column_name) {
            let count = texts.len();
            let unique_count = texts.iter().collect::<std::collections::HashSet<_>>().len();
            
            Some(ColumnSummary::Text {
                count,
                unique_count,
            })
        } else {
            None
        }
    }

    pub fn validate_data(&self) -> Vec<DataIssue> {
        let mut issues = Vec::new();
        
        for (col_name, numbers) in &self.numeric_columns {
            if numbers.len() < self.row_count {
                issues.push(DataIssue::MissingValues {
                    column: col_name.clone(),
                    missing_count: self.row_count - numbers.len(),
                });
            }
            
            if numbers.iter().any(|&n| n.is_nan() || n.is_infinite()) {
                issues.push(DataIssue::InvalidNumbers {
                    column: col_name.clone(),
                });
            }
        }

        for (col_name, texts) in &self.text_columns {
            if texts.len() < self.row_count {
                issues.push(DataIssue::MissingValues {
                    column: col_name.clone(),
                    missing_count: self.row_count - texts.len(),
                });
            }
            
            if texts.iter().any(|t| t.trim().is_empty()) {
                issues.push(DataIssue::EmptyStrings {
                    column: col_name.clone(),
                });
            }
        }

        issues
    }
}

#[derive(Debug)]
pub enum ColumnSummary {
    Numeric {
        count: usize,
        mean: f64,
        median: f64,
        min: f64,
        max: f64,
        sum: f64,
    },
    Text {
        count: usize,
        unique_count: usize,
    },
}

#[derive(Debug)]
pub enum DataIssue {
    MissingValues {
        column: String,
        missing_count: usize,
    },
    InvalidNumbers {
        column: String,
    },
    EmptyStrings {
        column: String,
    },
}

pub fn print_analysis(stats: &CsvStats) {
    println!("CSV Analysis Summary:");
    println!("Rows: {}", stats.row_count);
    println!("Columns: {}", stats.column_count);
    println!("\nColumn Names:");
    for name in &stats.column_names {
        println!("  - {}", name);
    }

    println!("\nColumn Statistics:");
    for name in &stats.column_names {
        if let Some(summary) = stats.get_column_summary(name) {
            match summary {
                ColumnSummary::Numeric { count, mean, median, min, max, sum } => {
                    println!("  {} (numeric):", name);
                    println!("    Count: {}, Mean: {:.2}, Median: {:.2}", count, mean, median);
                    println!("    Min: {:.2}, Max: {:.2}, Sum: {:.2}", min, max, sum);
                }
                ColumnSummary::Text { count, unique_count } => {
                    println!("  {} (text):", name);
                    println!("    Count: {}, Unique values: {}", count, unique_count);
                }
            }
        }
    }

    let issues = stats.validate_data();
    if !issues.is_empty() {
        println!("\nData Issues Found:");
        for issue in issues {
            match issue {
                DataIssue::MissingValues { column, missing_count } => {
                    println!("  {}: {} missing values", column, missing_count);
                }
                DataIssue::InvalidNumbers { column } => {
                    println!("  {}: contains invalid numbers (NaN or infinite)", column);
                }
                DataIssue::EmptyStrings { column } => {
                    println!("  {}: contains empty strings", column);
                }
            }
        }
    }
}
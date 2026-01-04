use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvAnalyzer {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvAnalyzer {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        
        let mut data = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            data.push(row);
        }
        
        Ok(CsvAnalyzer { data, headers })
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
    
    pub fn column_stats(&self, column_index: usize) -> Result<ColumnStats, Box<dyn Error>> {
        if column_index >= self.headers.len() {
            return Err("Column index out of bounds".into());
        }
        
        let mut numeric_values = Vec::new();
        let mut text_values = Vec::new();
        
        for row in &self.data {
            if column_index < row.len() {
                let value = &row[column_index];
                if let Ok(num) = value.parse::<f64>() {
                    numeric_values.push(num);
                } else {
                    text_values.push(value.clone());
                }
            }
        }
        
        Ok(ColumnStats {
            column_name: self.headers[column_index].clone(),
            numeric_count: numeric_values.len(),
            text_count: text_values.len(),
            numeric_stats: if !numeric_values.is_empty() {
                Some(NumericStats::from_values(&numeric_values))
            } else {
                None
            },
            unique_text_count: if !text_values.is_empty() {
                let unique: std::collections::HashSet<_> = text_values.iter().collect();
                Some(unique.len())
            } else {
                None
            },
        })
    }
    
    pub fn validate_data(&self) -> Vec<DataIssue> {
        let mut issues = Vec::new();
        
        for (row_idx, row) in self.data.iter().enumerate() {
            if row.len() != self.headers.len() {
                issues.push(DataIssue::ColumnMismatch {
                    row: row_idx + 1,
                    expected: self.headers.len(),
                    actual: row.len(),
                });
            }
            
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.trim().is_empty() {
                    issues.push(DataIssue::EmptyCell {
                        row: row_idx + 1,
                        column: col_idx + 1,
                        column_name: self.headers[col_idx].clone(),
                    });
                }
            }
        }
        
        issues
    }
}

pub struct ColumnStats {
    pub column_name: String,
    pub numeric_count: usize,
    pub text_count: usize,
    pub numeric_stats: Option<NumericStats>,
    pub unique_text_count: Option<usize>,
}

pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub sum: f64,
}

impl NumericStats {
    fn from_values(values: &[f64]) -> Self {
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;
        
        NumericStats { min, max, avg, sum }
    }
}

pub enum DataIssue {
    ColumnMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    EmptyCell {
        row: usize,
        column: usize,
        column_name: String,
    },
}

impl std::fmt::Display for DataIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataIssue::ColumnMismatch { row, expected, actual } => {
                write!(f, "Row {}: Expected {} columns, found {}", row, expected, actual)
            }
            DataIssue::EmptyCell { row, column, column_name } => {
                write!(f, "Row {}, Column {} ({}): Empty cell", row, column, column_name)
            }
        }
    }
}
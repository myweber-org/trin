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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
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
        
        let mut data = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let row: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        Ok(CsvAnalyzer { headers, data })
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
    
    pub fn get_column_stats(&self, column_index: usize) -> Option<HashMap<String, usize>> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let mut stats = HashMap::new();
        for row in &self.data {
            if let Some(value) = row.get(column_index) {
                *stats.entry(value.clone()).or_insert(0) += 1;
            }
        }
        
        Some(stats)
    }
    
    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data.iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
    
    pub fn find_duplicates(&self, column_indices: &[usize]) -> HashMap<Vec<String>, usize> {
        let mut frequency_map = HashMap::new();
        
        for row in &self.data {
            let key: Vec<String> = column_indices.iter()
                .filter_map(|&idx| row.get(idx).cloned())
                .collect();
            
            if !key.is_empty() {
                *frequency_map.entry(key).or_insert(0) += 1;
            }
        }
        
        frequency_map.into_iter()
            .filter(|(_, count)| *count > 1)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();
        temp_file
    }
    
    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.row_count(), 4);
        assert_eq!(analyzer.column_count(), 3);
        assert_eq!(analyzer.headers, vec!["name", "age", "city"]);
    }
    
    #[test]
    fn test_column_stats() {
        let temp_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let stats = analyzer.get_column_stats(0).unwrap();
        assert_eq!(stats.get("Alice"), Some(&2));
        assert_eq!(stats.get("Bob"), Some(&1));
        assert_eq!(stats.get("Charlie"), Some(&1));
    }
    
    #[test]
    fn test_duplicate_finding() {
        let temp_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let duplicates = analyzer.find_duplicates(&[0, 1, 2]);
        let expected_key = vec!["Alice".to_string(), "30".to_string(), "New York".to_string()];
        assert_eq!(duplicates.get(&expected_key), Some(&2));
    }
}
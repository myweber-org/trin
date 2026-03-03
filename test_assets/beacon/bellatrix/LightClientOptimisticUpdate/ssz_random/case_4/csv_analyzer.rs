use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
    column_types: HashMap<String, String>,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        let column_types = Self::detect_column_types(&headers, &records);

        Ok(CsvAnalyzer {
            headers,
            records,
            column_types,
        })
    }

    fn detect_column_types(headers: &[String], records: &[Vec<String>]) -> HashMap<String, String> {
        let mut types = HashMap::new();
        
        for (idx, header) in headers.iter().enumerate() {
            let mut is_numeric = true;
            let mut has_content = false;
            
            for record in records {
                if idx < record.len() {
                    let value = &record[idx];
                    if !value.is_empty() {
                        has_content = true;
                        if value.parse::<f64>().is_err() {
                            is_numeric = false;
                            break;
                        }
                    }
                }
            }
            
            let col_type = if !has_content {
                "empty".to_string()
            } else if is_numeric {
                "numeric".to_string()
            } else {
                "text".to_string()
            };
            
            types.insert(header.clone(), col_type);
        }
        
        types
    }

    pub fn get_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        summary.insert("total_rows".to_string(), self.records.len().to_string());
        summary.insert("total_columns".to_string(), self.headers.len().to_string());
        
        let numeric_columns: Vec<&String> = self.column_types
            .iter()
            .filter(|(_, v)| **v == "numeric")
            .map(|(k, _)| k)
            .collect();
        
        summary.insert("numeric_columns".to_string(), numeric_columns.len().to_string());
        
        summary
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        if let Some(column_index) = self.headers.iter().position(|h| h == column_name) {
            self.records
                .iter()
                .filter(|record| {
                    column_index < record.len() && record[column_index] == value
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn calculate_column_stats(&self, column_name: &str) -> Option<HashMap<String, f64>> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| {
                if column_index < record.len() {
                    record[column_index].parse::<f64>().ok()
                } else {
                    None
                }
            })
            .collect();
        
        if numeric_values.is_empty() {
            return None;
        }
        
        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = numeric_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let mut sorted_values = numeric_values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[count as usize / 2]
        };
        
        let mut stats = HashMap::new();
        stats.insert("mean".to_string(), mean);
        stats.insert("median".to_string(), median);
        stats.insert("min".to_string(), *sorted_values.first().unwrap());
        stats.insert("max".to_string(), *sorted_values.last().unwrap());
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        Some(stats)
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        if let Some(column_index) = self.headers.iter().position(|h| h == column_name) {
            let mut unique_values = std::collections::HashSet::new();
            
            for record in &self.records {
                if column_index < record.len() {
                    unique_values.insert(record[column_index].clone());
                }
            }
            
            unique_values.into_iter().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary,department").unwrap();
        writeln!(temp_file, "Alice,30,50000.0,Engineering").unwrap();
        writeln!(temp_file, "Bob,25,45000.0,Marketing").unwrap();
        writeln!(temp_file, "Charlie,35,60000.0,Engineering").unwrap();
        writeln!(temp_file, "Diana,28,52000.0,Sales").unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.headers.len(), 4);
        assert_eq!(analyzer.records.len(), 4);
    }

    #[test]
    fn test_summary() {
        let temp_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        let summary = analyzer.get_summary();
        
        assert_eq!(summary.get("total_rows").unwrap(), "4");
        assert_eq!(summary.get("total_columns").unwrap(), "4");
    }

    #[test]
    fn test_filtering() {
        let temp_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = analyzer.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
    }

    #[test]
    fn test_stats_calculation() {
        let temp_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let stats = analyzer.calculate_column_stats("salary").unwrap();
        assert!(stats.get("mean").unwrap() > &0.0);
        assert!(stats.get("median").unwrap() > &0.0);
    }
}
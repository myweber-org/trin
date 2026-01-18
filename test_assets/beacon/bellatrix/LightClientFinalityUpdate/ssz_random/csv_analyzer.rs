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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct CSVStats {
    row_count: usize,
    column_count: usize,
    column_types: HashMap<String, String>,
    numeric_columns: Vec<String>,
    text_columns: Vec<String>,
}

impl CSVStats {
    fn new() -> Self {
        CSVStats {
            row_count: 0,
            column_count: 0,
            column_types: HashMap::new(),
            numeric_columns: Vec::new(),
            text_columns: Vec::new(),
        }
    }

    fn analyze_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = CSVStats::new();
        let mut headers: Vec<String> = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(',').collect();

            if index == 0 {
                stats.column_count = columns.len();
                headers = columns.iter().map(|s| s.to_string()).collect();
                for header in &headers {
                    stats.column_types.insert(header.clone(), "unknown".to_string());
                }
                continue;
            }

            stats.row_count += 1;

            for (i, value) in columns.iter().enumerate() {
                if i >= headers.len() {
                    break;
                }

                let header = &headers[i];
                let current_type = stats.column_types.get(header).unwrap();

                if current_type == "unknown" {
                    let new_type = if value.parse::<f64>().is_ok() {
                        "numeric"
                    } else {
                        "text"
                    };
                    stats.column_types.insert(header.clone(), new_type.to_string());
                }
            }
        }

        for (header, col_type) in &stats.column_types {
            match col_type.as_str() {
                "numeric" => stats.numeric_columns.push(header.clone()),
                "text" => stats.text_columns.push(header.clone()),
                _ => {}
            }
        }

        Ok(stats)
    }

    fn filter_numeric_data(&self, path: &str, column: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut numeric_data = Vec::new();
        let mut headers: Vec<String> = Vec::new();
        let mut target_index = None;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(',').collect();

            if index == 0 {
                headers = columns.iter().map(|s| s.to_string()).collect();
                target_index = headers.iter().position(|h| h == column);
                continue;
            }

            if let Some(col_index) = target_index {
                if col_index < columns.len() {
                    if let Ok(value) = columns[col_index].parse::<f64>() {
                        numeric_data.push(value);
                    }
                }
            }
        }

        Ok(numeric_data)
    }

    fn calculate_stats(&self, data: &[f64]) -> (f64, f64, f64) {
        if data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <csv_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    
    match CSVStats::analyze_file(file_path) {
        Ok(stats) => {
            println!("CSV Analysis Results:");
            println!("Rows: {}", stats.row_count);
            println!("Columns: {}", stats.column_count);
            println!("Numeric columns: {:?}", stats.numeric_columns);
            println!("Text columns: {:?}", stats.text_columns);

            if !stats.numeric_columns.is_empty() {
                let first_numeric = &stats.numeric_columns[0];
                println!("\nAnalyzing column: {}", first_numeric);
                
                match stats.filter_numeric_data(file_path, first_numeric) {
                    Ok(data) => {
                        let (mean, variance, std_dev) = stats.calculate_stats(&data);
                        println!("Data points: {}", data.len());
                        println!("Mean: {:.4}", mean);
                        println!("Variance: {:.4}", variance);
                        println!("Standard Deviation: {:.4}", std_dev);
                    }
                    Err(e) => eprintln!("Error filtering data: {}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Error analyzing CSV: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000.0").unwrap();
        writeln!(temp_file, "2,Bob,25,45000.0").unwrap();
        writeln!(temp_file, "3,Charlie,35,55000.0").unwrap();

        let stats = CSVStats::analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.numeric_columns.len(), 2);
        assert_eq!(stats.text_columns.len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = CSVStats::new();
        let (mean, variance, std_dev) = stats.calculate_stats(&data);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct ColumnStats {
    name: String,
    numeric_values: Vec<f64>,
    string_values: Vec<String>,
    is_numeric: bool,
    min: Option<f64>,
    max: Option<f64>,
    mean: Option<f64>,
    std_dev: Option<f64>,
}

impl ColumnStats {
    fn new(name: &str) -> Self {
        ColumnStats {
            name: name.to_string(),
            numeric_values: Vec::new(),
            string_values: Vec::new(),
            is_numeric: false,
            min: None,
            max: None,
            mean: None,
            std_dev: None,
        }
    }

    fn analyze(&mut self) {
        if !self.numeric_values.is_empty() {
            self.is_numeric = true;
            self.min = self.numeric_values.iter().copied().fold(f64::INFINITY, f64::min);
            self.max = self.numeric_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            
            let sum: f64 = self.numeric_values.iter().sum();
            self.mean = Some(sum / self.numeric_values.len() as f64);
            
            let variance: f64 = self.numeric_values.iter()
                .map(|&x| (x - self.mean.unwrap()).powi(2))
                .sum::<f64>() / self.numeric_values.len() as f64;
            self.std_dev = Some(variance.sqrt());
        }
    }

    fn detect_outliers(&self, threshold: f64) -> Vec<(usize, f64)> {
        if !self.is_numeric || self.std_dev.is_none() {
            return Vec::new();
        }
        
        let mean = self.mean.unwrap();
        let std_dev = self.std_dev.unwrap();
        let mut outliers = Vec::new();
        
        for (idx, &value) in self.numeric_values.iter().enumerate() {
            let z_score = (value - mean).abs() / std_dev;
            if z_score > threshold {
                outliers.push((idx, value));
            }
        }
        
        outliers
    }
}

struct CSVAnalyzer {
    columns: HashMap<String, ColumnStats>,
    row_count: usize,
}

impl CSVAnalyzer {
    fn new() -> Self {
        CSVAnalyzer {
            columns: HashMap::new(),
            row_count: 0,
        }
    }

    fn load_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        if let Some(header) = lines.next() {
            let header_line = header?;
            let column_names: Vec<&str> = header_line.split(',').collect();
            
            for &name in &column_names {
                self.columns.insert(name.to_string(), ColumnStats::new(name));
            }
            
            for line in lines {
                let record = line?;
                let values: Vec<&str> = record.split(',').collect();
                
                if values.len() == column_names.len() {
                    for (i, &value) in values.iter().enumerate() {
                        let col_name = column_names[i].to_string();
                        if let Some(col_stats) = self.columns.get_mut(&col_name) {
                            if let Ok(num) = value.parse::<f64>() {
                                col_stats.numeric_values.push(num);
                            } else {
                                col_stats.string_values.push(value.to_string());
                            }
                        }
                    }
                    self.row_count += 1;
                }
            }
        }
        
        for col_stats in self.columns.values_mut() {
            col_stats.analyze();
        }
        
        Ok(())
    }

    fn print_summary(&self) {
        println!("CSV Analysis Summary");
        println!("Total Rows: {}", self.row_count);
        println!("Total Columns: {}", self.columns.len());
        println!("\nColumn Statistics:");
        
        for (name, stats) in &self.columns {
            println!("\nColumn: {}", name);
            if stats.is_numeric {
                println!("  Type: Numeric");
                println!("  Min: {:.4}", stats.min.unwrap_or(0.0));
                println!("  Max: {:.4}", stats.max.unwrap_or(0.0));
                println!("  Mean: {:.4}", stats.mean.unwrap_or(0.0));
                println!("  Std Dev: {:.4}", stats.std_dev.unwrap_or(0.0));
                println!("  Sample Count: {}", stats.numeric_values.len());
                
                let outliers = stats.detect_outliers(3.0);
                if !outliers.is_empty() {
                    println!("  Outliers (z-score > 3): {}", outliers.len());
                    for (row_idx, value) in outliers.iter().take(5) {
                        println!("    Row {}: {:.4}", row_idx + 1, value);
                    }
                }
            } else {
                println!("  Type: Text");
                println!("  Unique Values: {}", stats.string_values.len());
                if !stats.string_values.is_empty() {
                    let mut value_counts: HashMap<&String, usize> = HashMap::new();
                    for value in &stats.string_values {
                        *value_counts.entry(value).or_insert(0) += 1;
                    }
                    
                    let mut sorted_values: Vec<_> = value_counts.iter().collect();
                    sorted_values.sort_by(|a, b| b.1.cmp(a.1));
                    
                    println!("  Top 5 Values:");
                    for (value, count) in sorted_values.iter().take(5) {
                        println!("    {}: {} occurrences", value, count);
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut analyzer = CSVAnalyzer::new();
    
    match analyzer.load_file("data.csv") {
        Ok(_) => {
            analyzer.print_summary();
            Ok(())
        }
        Err(e) => {
            eprintln!("Error loading file: {}", e);
            Ok(())
        }
    }
}
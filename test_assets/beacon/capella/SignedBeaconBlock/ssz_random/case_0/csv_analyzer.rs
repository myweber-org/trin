use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
    column_stats: Vec<ColumnStats>,
}

#[derive(Debug, Clone)]
pub struct ColumnStats {
    name: String,
    count: usize,
    unique_count: usize,
    min_length: Option<usize>,
    max_length: Option<usize>,
    numeric_count: usize,
    empty_count: usize,
}

impl CsvAnalyzer {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        let mut records = Vec::new();
        let mut column_data: Vec<Vec<String>> = vec![Vec::new(); headers.len()];
        
        for result in rdr.records() {
            let record = result?;
            let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            
            for (i, field) in fields.iter().enumerate() {
                if i < column_data.len() {
                    column_data[i].push(field.clone());
                }
            }
            
            records.push(fields);
        }
        
        let column_stats = headers.iter().enumerate().map(|(i, header)| {
            Self::calculate_column_stats(header, &column_data[i])
        }).collect();
        
        Ok(Self {
            headers,
            records,
            column_stats,
        })
    }
    
    fn calculate_column_stats(header: &str, data: &[String]) -> ColumnStats {
        let mut unique_set = std::collections::HashSet::new();
        let mut min_length = None;
        let mut max_length = None;
        let mut numeric_count = 0;
        let mut empty_count = 0;
        
        for value in data {
            unique_set.insert(value.clone());
            
            let length = value.len();
            min_length = Some(min_length.map_or(length, |min| min.min(length)));
            max_length = Some(max_length.map_or(length, |max| max.max(length)));
            
            if value.trim().is_empty() {
                empty_count += 1;
            }
            
            if value.parse::<f64>().is_ok() {
                numeric_count += 1;
            }
        }
        
        ColumnStats {
            name: header.to_string(),
            count: data.len(),
            unique_count: unique_set.len(),
            min_length,
            max_length,
            numeric_count,
            empty_count,
        }
    }
    
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary");
        println!("====================");
        println!("Total Records: {}", self.records.len());
        println!("Total Columns: {}", self.headers.len());
        println!();
        
        println!("Column Statistics:");
        println!("{:<20} {:<10} {:<12} {:<10} {:<10} {:<12} {:<10}", 
                 "Name", "Count", "Unique", "Min Len", "Max Len", "Numeric", "Empty");
        println!("{}", "-".repeat(94));
        
        for stats in &self.column_stats {
            println!("{:<20} {:<10} {:<12} {:<10} {:<10} {:<12} {:<10}",
                     stats.name,
                     stats.count,
                     stats.unique_count,
                     stats.min_length.unwrap_or(0),
                     stats.max_length.unwrap_or(0),
                     stats.numeric_count,
                     stats.empty_count);
        }
    }
    
    pub fn validate_data(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        for stats in &self.column_stats {
            if stats.empty_count > 0 {
                issues.push(format!("Column '{}' has {} empty values", stats.name, stats.empty_count));
            }
            
            if stats.unique_count == 1 && stats.count > 1 {
                issues.push(format!("Column '{}' has only 1 unique value across {} records", stats.name, stats.count));
            }
            
            let completeness = (stats.count - stats.empty_count) as f64 / stats.count as f64;
            if completeness < 0.8 {
                issues.push(format!("Column '{}' is only {:.1}% complete", stats.name, completeness * 100.0));
            }
        }
        
        issues
    }
    
    pub fn get_column_names(&self) -> &[String] {
        &self.headers
    }
    
    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,").unwrap();
        
        let analyzer = CsvAnalyzer::from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.get_record_count(), 3);
        assert_eq!(analyzer.get_column_names(), vec!["name", "age", "city"]);
        
        let issues = analyzer.validate_data();
        assert!(issues.iter().any(|i| i.contains("city") && i.contains("empty")));
    }
}
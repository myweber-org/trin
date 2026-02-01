use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    pub file_path: String,
    pub delimiter: char,
    pub has_header: bool,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Self {
        CsvAnalyzer {
            file_path: file_path.to_string(),
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut row_count = 0;
        let mut column_count = 0;
        let mut column_stats: HashMap<usize, ColumnStatistics> = HashMap::new();
        let mut sample_rows: Vec<Vec<String>> = Vec::new();

        if let Some(first_line) = lines.next() {
            let first_line = first_line?;
            let columns: Vec<&str> = first_line.split(self.delimiter).collect();
            column_count = columns.len();

            if self.has_header {
                for (idx, header) in columns.iter().enumerate() {
                    let mut stats = ColumnStatistics::new();
                    stats.header = Some(header.to_string());
                    column_stats.insert(idx, stats);
                }
            } else {
                for idx in 0..column_count {
                    column_stats.insert(idx, ColumnStatistics::new());
                }
                self.process_row(&columns, &mut column_stats, row_count);
                row_count += 1;
                if row_count <= 5 {
                    sample_rows.push(columns.iter().map(|s| s.to_string()).collect());
                }
            }
        }

        for line in lines {
            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if columns.len() != column_count {
                return Err(format!("Row {} has {} columns, expected {}", 
                    row_count + 1, columns.len(), column_count).into());
            }

            self.process_row(&columns, &mut column_stats, row_count);
            row_count += 1;

            if row_count <= 5 {
                sample_rows.push(columns.iter().map(|s| s.to_string()).collect());
            }
        }

        Ok(AnalysisResult {
            row_count,
            column_count,
            column_stats,
            sample_rows,
        })
    }

    fn process_row(&self, columns: &[&str], stats: &mut HashMap<usize, ColumnStatistics>, row_idx: usize) {
        for (col_idx, value) in columns.iter().enumerate() {
            if let Some(col_stat) = stats.get_mut(&col_idx) {
                col_stat.total_count += 1;
                
                if value.is_empty() {
                    col_stat.empty_count += 1;
                } else {
                    col_stat.non_empty_count += 1;
                }

                if let Ok(num) = value.parse::<f64>() {
                    col_stat.numeric_count += 1;
                    col_stat.sum += num;
                    col_stat.min = col_stat.min.min(num);
                    col_stat.max = col_stat.max.max(num);
                }

                if row_idx == 0 && !self.has_header {
                    col_stat.first_value = Some(value.to_string());
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub header: Option<String>,
    pub total_count: usize,
    pub empty_count: usize,
    pub non_empty_count: usize,
    pub numeric_count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub first_value: Option<String>,
}

impl ColumnStatistics {
    pub fn new() -> Self {
        ColumnStatistics {
            header: None,
            total_count: 0,
            empty_count: 0,
            non_empty_count: 0,
            numeric_count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
            first_value: None,
        }
    }

    pub fn average(&self) -> Option<f64> {
        if self.numeric_count > 0 {
            Some(self.sum / self.numeric_count as f64)
        } else {
            None
        }
    }

    pub fn empty_percentage(&self) -> f64 {
        if self.total_count > 0 {
            (self.empty_count as f64 / self.total_count as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn numeric_percentage(&self) -> f64 {
        if self.total_count > 0 {
            (self.numeric_count as f64 / self.total_count as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub row_count: usize,
    pub column_count: usize,
    pub column_stats: HashMap<usize, ColumnStatistics>,
    pub sample_rows: Vec<Vec<String>>,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary:");
        println!("Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!();

        for col_idx in 0..self.column_count {
            if let Some(stats) = self.column_stats.get(&col_idx) {
                println!("Column {}:", col_idx + 1);
                if let Some(header) = &stats.header {
                    println!("  Header: {}", header);
                }
                println!("  Total values: {}", stats.total_count);
                println!("  Empty values: {} ({:.2}%)", 
                    stats.empty_count, stats.empty_percentage());
                println!("  Non-empty values: {}", stats.non_empty_count);
                println!("  Numeric values: {} ({:.2}%)", 
                    stats.numeric_count, stats.numeric_percentage());
                
                if stats.numeric_count > 0 {
                    println!("  Min: {:.2}", stats.min);
                    println!("  Max: {:.2}", stats.max);
                    if let Some(avg) = stats.average() {
                        println!("  Average: {:.2}", avg);
                    }
                }
                
                if let Some(first_val) = &stats.first_value {
                    println!("  First value: {}", first_val);
                }
                println!();
            }
        }

        if !self.sample_rows.is_empty() {
            println!("Sample rows (first {}):", self.sample_rows.len());
            for (idx, row) in self.sample_rows.iter().enumerate() {
                println!("  Row {}: {:?}", idx + 1, row);
            }
        }
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,,60000").unwrap();
        writeln!(temp_file, "Charlie,25,").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 3);
        assert_eq!(result.column_count, 3);
        
        let age_stats = result.column_stats.get(&1).unwrap();
        assert_eq!(age_stats.empty_count, 1);
        assert_eq!(age_stats.numeric_count, 2);
        assert_eq!(age_stats.min, 25.0);
        assert_eq!(age_stats.max, 30.0);
    }
}
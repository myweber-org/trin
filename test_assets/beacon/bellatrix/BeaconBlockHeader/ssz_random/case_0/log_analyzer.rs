use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogAnalyzer {
    error_counts: HashMap<String, usize>,
    total_lines: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_counts: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.total_lines += 1;

            if line.contains("ERROR") {
                let error_type = Self::extract_error_type(&line);
                *self.error_counts.entry(error_type).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    fn extract_error_type(line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 2 {
            parts[2].to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary");
        println!("====================");
        println!("Total lines processed: {}", self.total_lines);
        println!("\nError frequency:");
        
        let mut errors: Vec<_> = self.error_counts.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));

        for (error_type, count) in errors {
            println!("  {}: {} occurrences", error_type, count);
        }
    }

    pub fn most_common_error(&self) -> Option<(&String, &usize)> {
        self.error_counts
            .iter()
            .max_by_key(|&(_, count)| count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_error_extraction() {
        let log_line = "2024-01-15 10:30:45 ERROR DatabaseConnection Failed to connect";
        let error_type = LogAnalyzer::extract_error_type(log_line);
        assert_eq!(error_type, "DatabaseConnection");
    }

    #[test]
    fn test_analysis() {
        let mut log_content = Vec::new();
        writeln!(log_content, "2024-01-15 10:30:45 INFO System started").unwrap();
        writeln!(log_content, "2024-01-15 10:31:00 ERROR DatabaseConnection Timeout").unwrap();
        writeln!(log_content, "2024-01-15 10:32:15 ERROR FileSystem Permission denied").unwrap();
        writeln!(log_content, "2024-01-15 10:33:00 ERROR DatabaseConnection Connection refused").unwrap();

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&log_content).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(file.path()).unwrap();

        assert_eq!(analyzer.total_lines, 4);
        assert_eq!(analyzer.error_counts.get("DatabaseConnection"), Some(&2));
        assert_eq!(analyzer.error_counts.get("FileSystem"), Some(&1));
    }
}
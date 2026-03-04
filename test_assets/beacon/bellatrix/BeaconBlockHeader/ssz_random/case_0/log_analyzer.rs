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
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warn_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"(?i)error").unwrap(),
            warn_pattern: Regex::new(r"(?i)warn").unwrap(),
            info_pattern: Regex::new(r"(?i)info").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<LogSummary, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut summary = LogSummary::new();

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line, &mut summary);
        }

        Ok(summary)
    }

    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        summary.total_lines += 1;

        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.error_lines.push(line.to_string());
        } else if self.warn_pattern.is_match(line) {
            summary.warn_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }

        if line.contains("HTTP") {
            summary.http_requests += 1;
        }

        if line.contains("database") || line.contains("DB") {
            summary.database_operations += 1;
        }
    }

    pub fn get_top_errors(&self, summary: &LogSummary, limit: usize) -> Vec<String> {
        let mut error_map = HashMap::new();
        
        for error_line in &summary.error_lines {
            let key = error_line.split_whitespace()
                .skip_while(|w| !w.contains("error"))
                .take(3)
                .collect::<Vec<&str>>()
                .join(" ");
            
            *error_map.entry(key).or_insert(0) += 1;
        }

        let mut sorted_errors: Vec<_> = error_map.into_iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted_errors
            .into_iter()
            .take(limit)
            .map(|(error, count)| format!("{} ({} occurrences)", error, count))
            .collect()
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub http_requests: usize,
    pub database_operations: usize,
    pub error_lines: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            http_requests: 0,
            database_operations: 0,
            error_lines: Vec::new(),
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warn_count);
        println!("Info messages: {}", self.info_count);
        println!("HTTP requests: {}", self.http_requests);
        println!("Database operations: {}", self.database_operations);
        println!("Error rate: {:.2}%", self.error_rate());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert!(analyzer.error_pattern.is_match("ERROR: Something went wrong"));
        assert!(analyzer.warn_pattern.is_match("WARNING: Disk space low"));
        assert!(analyzer.info_pattern.is_match("INFO: Process started"));
    }

    #[test]
    fn test_summary_calculation() {
        let mut summary = LogSummary::new();
        summary.total_lines = 100;
        summary.error_count = 5;
        
        assert_eq!(summary.error_rate(), 5.0);
    }
}
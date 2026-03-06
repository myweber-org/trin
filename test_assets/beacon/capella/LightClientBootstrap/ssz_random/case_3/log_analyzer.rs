use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_patterns: HashMap<String, Regex>,
    warning_patterns: HashMap<String, Regex>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let mut error_patterns = HashMap::new();
        let mut warning_patterns = HashMap::new();

        error_patterns.insert(
            "connection_error".to_string(),
            Regex::new(r"Connection failed|Timeout|Disconnected").unwrap(),
        );
        error_patterns.insert(
            "auth_error".to_string(),
            Regex::new(r"Authentication failed|Invalid credentials|Access denied").unwrap(),
        );

        warning_patterns.insert(
            "resource_warning".to_string(),
            Regex::new(r"Low memory|High CPU|Disk space critical").unwrap(),
        );
        warning_patterns.insert(
            "performance_warning".to_string(),
            Regex::new(r"Slow response|High latency|Queue full").unwrap(),
        );

        LogAnalyzer {
            error_patterns,
            warning_patterns,
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        let mut summary = LogSummary::new();
        let timestamp_regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| e.to_string())?;
            
            if timestamp_regex.is_match(&line) {
                summary.total_entries += 1;
            }

            for (error_type, pattern) in &self.error_patterns {
                if pattern.is_match(&line) {
                    summary.error_counts.entry(error_type.clone()).and_modify(|e| *e += 1).or_insert(1);
                    summary.total_errors += 1;
                }
            }

            for (warning_type, pattern) in &self.warning_patterns {
                if pattern.is_match(&line) {
                    summary.warning_counts.entry(warning_type.clone()).and_modify(|w| *w += 1).or_insert(1);
                    summary.total_warnings += 1;
                }
            }

            if line.contains("ERROR") {
                summary.error_lines.push((line_number + 1, line.clone()));
            } else if line.contains("WARN") {
                summary.warning_lines.push((line_number + 1, line.clone()));
            }
        }

        Ok(summary)
    }

    pub fn add_custom_pattern(&mut self, category: &str, pattern: &str, is_error: bool) -> Result<(), String> {
        let regex = Regex::new(pattern).map_err(|e| e.to_string())?;
        
        if is_error {
            self.error_patterns.insert(category.to_string(), regex);
        } else {
            self.warning_patterns.insert(category.to_string(), regex);
        }
        
        Ok(())
    }
}

pub struct LogSummary {
    pub total_entries: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub error_counts: HashMap<String, usize>,
    pub warning_counts: HashMap<String, usize>,
    pub error_lines: Vec<(usize, String)>,
    pub warning_lines: Vec<(usize, String)>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_entries: 0,
            total_errors: 0,
            total_warnings: 0,
            error_counts: HashMap::new(),
            warning_counts: HashMap::new(),
            error_lines: Vec::new(),
            warning_lines: Vec::new(),
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total entries: {}", self.total_entries);
        println!("Total errors: {}", self.total_errors);
        println!("Total warnings: {}", self.total_warnings);
        
        if !self.error_counts.is_empty() {
            println!("\nError breakdown:");
            for (error_type, count) in &self.error_counts {
                println!("  {}: {}", error_type, count);
            }
        }
        
        if !self.warning_counts.is_empty() {
            println!("\nWarning breakdown:");
            for (warning_type, count) in &self.warning_counts {
                println!("  {}: {}", warning_type, count);
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
    fn test_log_analysis() {
        let mut test_log = NamedTempFile::new().unwrap();
        writeln!(test_log, "2024-01-15 10:30:00 INFO Application started").unwrap();
        writeln!(test_log, "2024-01-15 10:31:00 ERROR Connection failed to database").unwrap();
        writeln!(test_log, "2024-01-15 10:32:00 WARN Low memory detected").unwrap();
        writeln!(test_log, "2024-01-15 10:33:00 ERROR Authentication failed for user").unwrap();

        let analyzer = LogAnalyzer::new();
        let summary = analyzer.analyze_log_file(test_log.path().to_str().unwrap()).unwrap();

        assert_eq!(summary.total_entries, 4);
        assert_eq!(summary.total_errors, 2);
        assert_eq!(summary.total_warnings, 1);
        assert_eq!(summary.error_counts.get("connection_error"), Some(&1));
        assert_eq!(summary.error_counts.get("auth_error"), Some(&1));
        assert_eq!(summary.warning_counts.get("resource_warning"), Some(&1));
    }
}
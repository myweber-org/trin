use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warning_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR|error|Error").unwrap(),
            warning_pattern: Regex::new(r"WARN|warn|Warn|WARNING|warning|Warning").unwrap(),
            info_pattern: Regex::new(r"INFO|info|Info").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<LogSummary, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary::new();
        
        for line in reader.lines() {
            let line = line?;
            self.analyze_line(&line, &mut summary);
        }
        
        Ok(summary)
    }
    
    fn analyze_line(&self, line: &str, summary: &mut LogSummary) {
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.error_lines.push(line.to_string());
        } else if self.warning_pattern.is_match(line) {
            summary.warning_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        summary.total_lines += 1;
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub error_lines: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            error_lines: Vec::new(),
        }
    }
    
    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        
        if !self.error_lines.is_empty() {
            println!("\nError lines found:");
            for (i, line) in self.error_lines.iter().enumerate().take(5) {
                println!("  {}. {}", i + 1, line);
            }
            if self.error_lines.len() > 5 {
                println!("  ... and {} more errors", self.error_lines.len() - 5);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert!(analyzer.error_pattern.is_match("ERROR: Something went wrong"));
        assert!(analyzer.warning_pattern.is_match("WARNING: This is a warning"));
        assert!(analyzer.info_pattern.is_match("INFO: Process completed"));
    }
    
    #[test]
    fn test_log_summary() {
        let summary = LogSummary::new();
        assert_eq!(summary.total_lines, 0);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.warning_count, 0);
        assert_eq!(summary.info_count, 0);
        assert!(summary.error_lines.is_empty());
    }
}
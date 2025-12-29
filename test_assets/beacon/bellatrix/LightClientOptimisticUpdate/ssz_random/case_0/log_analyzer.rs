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

    pub fn analyze_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            self.process_line(&line, &mut summary);
        }
        
        Ok(summary)
    }
    
    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        summary.total_lines += 1;
        
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.error_lines.push(line.to_string());
        } else if self.warning_pattern.is_match(line) {
            summary.warning_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        if line.contains("exception") || line.contains("Exception") {
            summary.exception_count += 1;
        }
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub exception_count: usize,
    pub error_lines: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            exception_count: 0,
            error_lines: Vec::new(),
        }
    }
    
    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        println!("Exceptions: {}", self.exception_count);
        
        if !self.error_lines.is_empty() {
            println!("\nError lines found:");
            for (i, line) in self.error_lines.iter().enumerate().take(5) {
                println!("  {}. {}", i + 1, line);
            }
            if self.error_lines.len() > 5 {
                println!("  ... and {} more", self.error_lines.len() - 5);
            }
        }
    }
}

pub fn analyze_multiple_files(file_paths: &[&str]) -> HashMap<String, LogSummary> {
    let analyzer = LogAnalyzer::new();
    let mut results = HashMap::new();
    
    for &file_path in file_paths {
        if let Ok(summary) = analyzer.analyze_file(file_path) {
            results.insert(file_path.to_string(), summary);
        }
    }
    
    results
}
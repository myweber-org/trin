
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warning_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR.*").unwrap(),
            warning_pattern: Regex::new(r"WARN.*").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<AnalysisResult, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut error_counts = HashMap::new();
        let mut warning_counts = HashMap::new();
        let mut total_lines = 0;
        let mut error_lines = 0;
        let mut warning_lines = 0;

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            total_lines += 1;

            if self.error_pattern.is_match(&line) {
                error_lines += 1;
                let error_type = self.extract_error_type(&line);
                *error_counts.entry(error_type).or_insert(0) += 1;
            } else if self.warning_pattern.is_match(&line) {
                warning_lines += 1;
                let warning_type = self.extract_warning_type(&line);
                *warning_counts.entry(warning_type).or_insert(0) += 1;
            }
        }

        Ok(AnalysisResult {
            total_lines,
            error_lines,
            warning_lines,
            error_counts,
            warning_counts,
        })
    }

    fn extract_error_type(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            parts[1].to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn extract_warning_type(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            parts[1].to_string()
        } else {
            "unknown".to_string()
        }
    }
}

pub struct AnalysisResult {
    pub total_lines: usize,
    pub error_lines: usize,
    pub warning_lines: usize,
    pub error_counts: HashMap<String, usize>,
    pub warning_counts: HashMap<String, usize>,
}

impl AnalysisResult {
    pub fn summary(&self) -> String {
        format!(
            "Total lines: {}, Errors: {}, Warnings: {}",
            self.total_lines, self.error_lines, self.warning_lines
        )
    }

    pub fn top_errors(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut entries: Vec<_> = self.error_counts.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries.into_iter().take(n).collect()
    }

    pub fn top_warnings(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut entries: Vec<_> = self.warning_counts.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries.into_iter().take(n).collect()
    }
}
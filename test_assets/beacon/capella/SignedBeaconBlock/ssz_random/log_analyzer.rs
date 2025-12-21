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
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warning_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
            if self.error_pattern.is_match(&line) {
                *stats.entry("errors".to_string()).or_insert(0) += 1;
            } else if self.warning_pattern.is_match(&line) {
                *stats.entry("warnings".to_string()).or_insert(0) += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.entry("info".to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(stats)
    }

    pub fn generate_summary(&self, stats: &HashMap<String, usize>) -> String {
        let errors = stats.get("errors").unwrap_or(&0);
        let warnings = stats.get("warnings").unwrap_or(&0);
        let info = stats.get("info").unwrap_or(&0);
        
        format!(
            "Log Summary: {} errors, {} warnings, {} info messages",
            errors, warnings, info
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 INFO: Application started").unwrap();
        writeln!(temp_file, "2023-10-01 WARN: Low memory").unwrap();
        writeln!(temp_file, "2023-10-01 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 INFO: Retrying connection").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        let summary = analyzer.generate_summary(&stats);
        
        assert_eq!(*stats.get("errors").unwrap(), 1);
        assert_eq!(*stats.get("warnings").unwrap(), 1);
        assert_eq!(*stats.get("info").unwrap(), 2);
        assert!(summary.contains("1 errors"));
        assert!(summary.contains("1 warnings"));
        assert!(summary.contains("2 info messages"));
    }
}
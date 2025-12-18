use std::collections::HashMap;
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
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warn_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        stats.insert("total_lines".to_string(), 0);
        stats.insert("error_count".to_string(), 0);
        stats.insert("warn_count".to_string(), 0);
        stats.insert("info_count".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            
            *stats.get_mut("total_lines").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *stats.get_mut("error_count").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.get_mut("warn_count").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info_count").unwrap() += 1;
            }
        }
        
        Ok(stats)
    }

    pub fn generate_summary(&self, stats: &HashMap<String, usize>) -> String {
        let total = stats.get("total_lines").unwrap_or(&0);
        let errors = stats.get("error_count").unwrap_or(&0);
        let warnings = stats.get("warn_count").unwrap_or(&0);
        let infos = stats.get("info_count").unwrap_or(&0);
        
        format!(
            "Log Analysis Summary:\n\
            Total Lines: {}\n\
            Errors: {}\n\
            Warnings: {}\n\
            Info Messages: {}",
            total, errors, warnings, infos
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
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("total_lines"), Some(&4));
        assert_eq!(stats.get("error_count"), Some(&1));
        assert_eq!(stats.get("warn_count"), Some(&1));
        assert_eq!(stats.get("info_count"), Some(&2));
    }
}
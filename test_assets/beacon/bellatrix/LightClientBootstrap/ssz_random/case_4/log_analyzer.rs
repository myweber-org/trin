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

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut stats = HashMap::new();
        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?;
            *stats.get_mut("total_lines").unwrap() += 1;

            if self.error_pattern.is_match(&line) {
                *stats.get_mut("errors").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info").unwrap() += 1;
            }
        }

        Ok(stats)
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::new();
        report.push_str(&format!("Log Analysis Report\n"));
        report.push_str(&format!("===================\n"));
        report.push_str(&format!("Total lines: {}\n", stats["total_lines"]));
        report.push_str(&format!("Errors: {}\n", stats["errors"]));
        report.push_str(&format!("Warnings: {}\n", stats["warnings"]));
        report.push_str(&format!("Info messages: {}\n", stats["info"]));
        
        let other = stats["total_lines"] - (stats["errors"] + stats["warnings"] + stats["info"]);
        report.push_str(&format!("Other entries: {}\n", other));
        
        report
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
        writeln!(temp_file, "ERROR: Connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing complete").unwrap();
        writeln!(temp_file, "DEBUG: Detailed trace").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(stats["total_lines"], 5);
        assert_eq!(stats["errors"], 1);
        assert_eq!(stats["warnings"], 1);
        assert_eq!(stats["info"], 2);
        
        let report = analyzer.generate_report(&stats);
        assert!(report.contains("Total lines: 5"));
        assert!(report.contains("Errors: 1"));
    }
}
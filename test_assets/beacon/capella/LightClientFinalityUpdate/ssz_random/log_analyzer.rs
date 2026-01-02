use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_patterns: HashMap<String, usize>,
    warning_count: usize,
    info_count: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_patterns: HashMap::new(),
            warning_count: 0,
            info_count: 0,
        }
    }

    pub fn analyze_file(&mut self, file_path: &str) -> Result<(), std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let error_re = Regex::new(r"ERROR.*").unwrap();
        let warning_re = Regex::new(r"WARN.*").unwrap();
        let info_re = Regex::new(r"INFO.*").unwrap();

        for line in reader.lines() {
            let line = line?;
            if error_re.is_match(&line) {
                let error_key = self.extract_error_type(&line);
                *self.error_patterns.entry(error_key).or_insert(0) += 1;
            } else if warning_re.is_match(&line) {
                self.warning_count += 1;
            } else if info_re.is_match(&line) {
                self.info_count += 1;
            }
        }
        Ok(())
    }

    fn extract_error_type(&self, line: &str) -> String {
        let re = Regex::new(r"ERROR:\s*(\w+)").unwrap();
        re.captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total INFO entries: {}\n", self.info_count));
        report.push_str(&format!("Total WARN entries: {}\n", self.warning_count));
        report.push_str("Error distribution:\n");
        
        for (error_type, count) in &self.error_patterns {
            report.push_str(&format!("  {}: {}\n", error_type, count));
        }
        
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
        let mut log_data = String::new();
        log_data.push_str("INFO: Application started\n");
        log_data.push_str("WARN: Deprecated API used\n");
        log_data.push_str("ERROR: Database connection failed\n");
        log_data.push_str("ERROR: Database connection failed\n");
        log_data.push_str("ERROR: File not found\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let report = analyzer.generate_report();
        assert!(report.contains("INFO entries: 1"));
        assert!(report.contains("WARN entries: 1"));
        assert!(report.contains("Database: 2"));
        assert!(report.contains("File: 1"));
    }
}
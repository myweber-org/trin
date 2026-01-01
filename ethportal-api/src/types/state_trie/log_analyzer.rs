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
        
        let mut counts = HashMap::new();
        counts.insert("ERROR".to_string(), 0);
        counts.insert("WARN".to_string(), 0);
        counts.insert("INFO".to_string(), 0);
        
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            
            if self.error_pattern.is_match(&line) {
                *counts.get_mut("ERROR").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *counts.get_mut("WARN").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *counts.get_mut("INFO").unwrap() += 1;
            }
        }
        
        Ok(counts)
    }
    
    pub fn generate_report(&self, counts: &HashMap<String, usize>) -> String {
        let total: usize = counts.values().sum();
        let error_rate = if total > 0 {
            (counts["ERROR"] as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        format!(
            "Log Analysis Report:\n\
            Total entries: {}\n\
            INFO: {}\n\
            WARN: {}\n\
            ERROR: {}\n\
            Error rate: {:.2}%",
            total,
            counts["INFO"],
            counts["WARN"],
            counts["ERROR"],
            error_rate
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
        writeln!(temp_file, "2023-10-01 INFO: Request processed").unwrap();
        
        let counts = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(counts["INFO"], 2);
        assert_eq!(counts["WARN"], 1);
        assert_eq!(counts["ERROR"], 1);
        
        let report = analyzer.generate_report(&counts);
        assert!(report.contains("Total entries: 4"));
        assert!(report.contains("Error rate: 25.00%"));
    }
}
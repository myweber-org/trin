use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message,
                };

                *self.level_counts.entry(level).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn contains_error(&self) -> bool {
        self.level_counts.contains_key("ERROR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "2023-10-05 14:30:00 [INFO] Application started\n\
             2023-10-05 14:31:00 [ERROR] Failed to connect to database\n\
             2023-10-05 14:32:00 [WARN] High memory usage detected"
        )
        .unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer
            .parse_file(temp_file.path().to_str().unwrap())
            .unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert!(analyzer.contains_error());
        assert_eq!(analyzer.get_level_summary().get("INFO"), Some(&1));
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
            let line = line_result
                .map_err(|e| format!("Failed to read line: {}", e))?;
            
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

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total = stats.get("total_lines").unwrap_or(&0);
        let errors = stats.get("error_count").unwrap_or(&0);
        let warnings = stats.get("warn_count").unwrap_or(&0);
        let infos = stats.get("info_count").unwrap_or(&0);
        
        format!(
            "Log Analysis Report:\n\
            Total lines: {}\n\
            Errors: {}\n\
            Warnings: {}\n\
            Info messages: {}\n\
            Error rate: {:.2}%",
            total,
            errors,
            warnings,
            infos,
            (*errors as f64 / *total as f64) * 100.0
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
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap())
            .expect("Failed to analyze log file");
        
        assert_eq!(stats.get("total_lines").unwrap(), &4);
        assert_eq!(stats.get("error_count").unwrap(), &1);
        assert_eq!(stats.get("warn_count").unwrap(), &1);
        assert_eq!(stats.get("info_count").unwrap(), &2);
        
        let report = analyzer.generate_report(&stats);
        assert!(report.contains("Total lines: 4"));
        assert!(report.contains("Errors: 1"));
    }
}
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
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.error_lines.push(line.to_string());
        } else if self.warning_pattern.is_match(line) {
            summary.warning_count += 1;
            summary.warning_lines.push(line.to_string());
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        summary.total_lines += 1;
    }
    
    pub fn get_error_distribution(&self, lines: &[String]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        let error_regex = Regex::new(r"ERROR:\s*(\w+)").unwrap();
        
        for line in lines {
            if let Some(caps) = error_regex.captures(line) {
                let error_type = caps.get(1).unwrap().as_str().to_string();
                *distribution.entry(error_type).or_insert(0) += 1;
            }
        }
        
        distribution
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub error_lines: Vec<String>,
    pub warning_lines: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            error_lines: Vec::new(),
            warning_lines: Vec::new(),
        }
    }
    
    pub fn error_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        }
    }
    
    pub fn warning_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.warning_count as f64 / self.total_lines as f64) * 100.0
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
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Low memory detected").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing request").unwrap();
        writeln!(temp_file, "ERROR: File not found").unwrap();
        
        let summary = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(summary.total_lines, 5);
        assert_eq!(summary.error_count, 2);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 2);
        assert_eq!(summary.error_lines.len(), 2);
        assert_eq!(summary.warning_lines.len(), 1);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub component: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogAnalyzer {
    pub entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() < 5 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level = parts[1].trim().to_string();
        let component = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();
        let metadata_str = parts[4].trim();

        let timestamp = DateTime::parse_from_rfc3339(timestamp_str).ok()?;
        
        let mut metadata = HashMap::new();
        for pair in metadata_str.split(',') {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                metadata.insert(kv[0].to_string(), kv[1].to_string());
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            component,
            message,
            metadata,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.component.contains(component))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn get_time_range(&self) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut min_time = &self.entries[0].timestamp;
        let mut max_time = &self.entries[0].timestamp;

        for entry in &self.entries[1..] {
            if entry.timestamp < *min_time {
                min_time = &entry.timestamp;
            }
            if entry.timestamp > *max_time {
                max_time = &entry.timestamp;
            }
        }

        Some((*min_time, *max_time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_parsing() {
        let analyzer = LogAnalyzer::new();
        let line = "2023-10-15T14:30:00+00:00 | INFO | network | Connection established | ip=192.168.1.1,port=8080";
        
        let entry = analyzer.parse_log_line(line).unwrap();
        
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.component, "network");
        assert_eq!(entry.message, "Connection established");
        assert_eq!(entry.metadata.get("ip"), Some(&"192.168.1.1".to_string()));
        assert_eq!(entry.metadata.get("port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_invalid_log_line() {
        let analyzer = LogAnalyzer::new();
        let line = "Invalid log format";
        
        assert!(analyzer.parse_log_line(line).is_none());
    }
}
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
}
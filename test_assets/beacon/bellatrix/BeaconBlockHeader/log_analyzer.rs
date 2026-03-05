use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogAnalyzer {
    error_counts: HashMap<String, u32>,
    warning_counts: HashMap<String, u32>,
    total_lines: u32,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_counts: HashMap::new(),
            warning_counts: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line);
            self.total_lines += 1;
        }

        Ok(())
    }

    fn process_line(&mut self, line: &str) {
        if line.contains("ERROR") {
            let error_type = self.extract_error_type(line);
            *self.error_counts.entry(error_type).or_insert(0) += 1;
        } else if line.contains("WARNING") {
            let warning_type = self.extract_warning_type(line);
            *self.warning_counts.entry(warning_type).or_insert(0) += 1;
        }
    }

    fn extract_error_type(&self, line: &str) -> String {
        line.split_whitespace()
            .find(|word| word.contains("ERROR"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "UNKNOWN_ERROR".to_string())
    }

    fn extract_warning_type(&self, line: &str) -> String {
        line.split_whitespace()
            .find(|word| word.contains("WARNING"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "UNKNOWN_WARNING".to_string())
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total lines processed: {}\n", self.total_lines));
        report.push_str(&format!("Unique errors: {}\n", self.error_counts.len()));
        report.push_str(&format!("Unique warnings: {}\n", self.warning_counts.len()));

        if !self.error_counts.is_empty() {
            report.push_str("\nError breakdown:\n");
            for (error, count) in &self.error_counts {
                report.push_str(&format!("  {}: {}\n", error, count));
            }
        }

        if !self.warning_counts.is_empty() {
            report.push_str("\nWarning breakdown:\n");
            for (warning, count) in &self.warning_counts {
                report.push_str(&format!("  {}: {}\n", warning, count));
            }
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
        log_data.push_str("ERROR: Database connection failed\n");
        log_data.push_str("WARNING: High memory usage detected\n");
        log_data.push_str("ERROR: Database connection failed\n");
        log_data.push_str("INFO: Processing complete\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(temp_file.path()).unwrap();

        let report = analyzer.generate_report();
        assert!(report.contains("Total lines processed: 5"));
        assert!(report.contains("Database connection failed: 2"));
        assert!(report.contains("High memory usage detected: 1"));
    }
}
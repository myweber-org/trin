use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let pattern = r"ERROR\s+\[(?P<module>.*?)\]\s+(?P<message>.*)";
        LogAnalyzer {
            error_pattern: Regex::new(pattern).unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut error_counts = HashMap::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if let Some(caps) = self.error_pattern.captures(&line) {
                let module = caps.name("module").unwrap().as_str().to_string();
                *error_counts.entry(module).or_insert(0) += 1;
            }
        }

        Ok(error_counts)
    }

    pub fn generate_report(&self, counts: &HashMap<String, usize>) -> String {
        if counts.is_empty() {
            return String::from("No errors found in log file.");
        }

        let mut report = String::from("Error Frequency Report:\n");
        let mut sorted_counts: Vec<_> = counts.iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

        for (module, count) in sorted_counts {
            report.push_str(&format!("  {}: {} errors\n", module, count));
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
    fn test_analyze_log_file() {
        let analyzer = LogAnalyzer::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO [network] Connection established").unwrap();
        writeln!(temp_file, "ERROR [database] Connection timeout").unwrap();
        writeln!(temp_file, "ERROR [database] Query failed").unwrap();
        writeln!(temp_file, "WARN [cache] Memory low").unwrap();
        writeln!(temp_file, "ERROR [auth] Invalid credentials").unwrap();

        let counts = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(counts.get("database"), Some(&2));
        assert_eq!(counts.get("auth"), Some(&1));
        assert_eq!(counts.get("network"), None);

        let report = analyzer.generate_report(&counts);
        assert!(report.contains("database: 2 errors"));
        assert!(report.contains("auth: 1 errors"));
    }
}
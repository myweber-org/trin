use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogAnalyzer {
    log_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            log_counts: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line);
        }

        Ok(())
    }

    fn process_line(&mut self, line: &str) {
        let levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        
        for level in levels.iter() {
            if line.contains(level) {
                *self.log_counts.entry(level.to_string()).or_insert(0) += 1;
                break;
            }
        }
    }

    pub fn get_counts(&self) -> &HashMap<String, usize> {
        &self.log_counts
    }

    pub fn print_summary(&self) {
        println!("Log Level Summary:");
        println!("==================");
        
        let mut sorted_counts: Vec<(&String, &usize)> = self.log_counts.iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

        for (level, count) in sorted_counts {
            println!("{:<10}: {}", level, count);
        }

        let total: usize = self.log_counts.values().sum();
        println!("==================");
        println!("Total logs analyzed: {}", total);
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
        log_data.push_str("2023-10-01 INFO: Application started\n");
        log_data.push_str("2023-10-01 ERROR: Failed to connect to database\n");
        log_data.push_str("2023-10-01 WARN: High memory usage detected\n");
        log_data.push_str("2023-10-01 INFO: Processing complete\n");
        log_data.push_str("2023-10-01 ERROR: Invalid user input\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(temp_file.path()).unwrap();

        let counts = analyzer.get_counts();
        assert_eq!(counts.get("INFO"), Some(&2));
        assert_eq!(counts.get("ERROR"), Some(&2));
        assert_eq!(counts.get("WARN"), Some(&1));
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warning" => Some(LogLevel::Warning),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

pub struct LogAnalyzer {
    counts: HashMap<LogLevel, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            counts: HashMap::new(),
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
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Some(level) = LogLevel::from_str(parts[0]) {
                *self.counts.entry(level).or_insert(0) += 1;
            }
        }
    }

    pub fn get_counts(&self) -> &HashMap<LogLevel, usize> {
        &self.counts
    }

    pub fn total_errors(&self) -> usize {
        self.counts.get(&LogLevel::Error).copied().unwrap_or(0)
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("====================");
        
        let levels = [
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];

        for level in levels.iter() {
            if let Some(count) = self.counts.get(level) {
                println!("{:?}: {}", level, count);
            }
        }
        
        println!("Total entries: {}", self.counts.values().sum::<usize>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("Warning"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_process_line() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.process_line("ERROR Database connection failed");
        analyzer.process_line("INFO User login successful");
        analyzer.process_line("ERROR Invalid input received");
        
        let counts = analyzer.get_counts();
        assert_eq!(counts.get(&LogLevel::Error), Some(&2));
        assert_eq!(counts.get(&LogLevel::Info), Some(&1));
        assert_eq!(analyzer.total_errors(), 2);
    }
}
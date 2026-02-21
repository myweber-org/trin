use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
enum LogSeverity {
    Info,
    Warning,
    Error,
    Debug,
    Unknown,
}

impl From<&str> for LogSeverity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "info" => LogSeverity::Info,
            "warning" => LogSeverity::Warning,
            "error" => LogSeverity::Error,
            "debug" => LogSeverity::Debug,
            _ => LogSeverity::Unknown,
        }
    }
}

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    severity: LogSeverity,
    message: String,
}

impl LogEntry {
    fn new(timestamp: &str, severity: &str, message: &str) -> Self {
        LogEntry {
            timestamp: timestamp.to_string(),
            severity: LogSeverity::from(severity),
            message: message.to_string(),
        }
    }
}

struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() == 3 {
            Some(LogEntry::new(parts[0].trim(), parts[1].trim(), parts[2].trim()))
        } else {
            None
        }
    }

    fn filter_by_severity(&self, severity: LogSeverity) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.severity == severity)
            .collect()
    }

    fn count_by_severity(&self) -> Vec<(LogSeverity, usize)> {
        let mut counts = vec![
            (LogSeverity::Info, 0),
            (LogSeverity::Warning, 0),
            (LogSeverity::Error, 0),
            (LogSeverity::Debug, 0),
            (LogSeverity::Unknown, 0),
        ];

        for entry in &self.entries {
            match entry.severity {
                LogSeverity::Info => counts[0].1 += 1,
                LogSeverity::Warning => counts[1].1 += 1,
                LogSeverity::Error => counts[2].1 += 1,
                LogSeverity::Debug => counts[3].1 += 1,
                LogSeverity::Unknown => counts[4].1 += 1,
            }
        }

        counts.retain(|(_, count)| *count > 0);
        counts
    }

    fn get_errors(&self) -> Vec<&LogEntry> {
        self.filter_by_severity(LogSeverity::Error)
    }

    fn get_warnings(&self) -> Vec<&LogEntry> {
        self.filter_by_severity(LogSeverity::Warning)
    }
}

fn main() -> io::Result<()> {
    let mut analyzer = LogAnalyzer::new();
    
    analyzer.load_from_file("application.log")?;
    
    println!("Total log entries: {}", analyzer.entries.len());
    
    let severity_counts = analyzer.count_by_severity();
    for (severity, count) in severity_counts {
        println!("{:?}: {}", severity, count);
    }
    
    let errors = analyzer.get_errors();
    if !errors.is_empty() {
        println!("\nError entries:");
        for error in errors {
            println!("{} - {}", error.timestamp, error.message);
        }
    }
    
    Ok(())
}
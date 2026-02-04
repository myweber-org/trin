use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warning" => Some(LogLevel::Warning),
            "error" => Some(LogLevel::Error),
            "critical" => Some(LogLevel::Critical),
            _ => None,
        }
    }

    pub fn severity_value(&self) -> u8 {
        match self {
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warning => 3,
            LogLevel::Error => 4,
            LogLevel::Critical => 5,
        }
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
}

pub struct LogParser {
    min_severity: LogLevel,
    filter_component: Option<String>,
}

impl LogParser {
    pub fn new(min_severity: LogLevel) -> Self {
        LogParser {
            min_severity,
            filter_component: None,
        }
    }

    pub fn with_component_filter(mut self, component: &str) -> Self {
        self.filter_component = Some(component.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp = parts[0].trim().to_string();
        let level_str = parts[1].trim();
        let component = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        let level = LogLevel::from_str(level_str)?;

        if level.severity_value() < self.min_severity.severity_value() {
            return None;
        }

        if let Some(ref filter) = self.filter_component {
            if component != *filter {
                return None;
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            component,
            message,
        })
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> Vec<(LogLevel, usize)> {
        let mut counts = vec![
            (LogLevel::Debug, 0),
            (LogLevel::Info, 0),
            (LogLevel::Warning, 0),
            (LogLevel::Error, 0),
            (LogLevel::Critical, 0),
        ];

        for entry in entries {
            match entry.level {
                LogLevel::Debug => counts[0].1 += 1,
                LogLevel::Info => counts[1].1 += 1,
                LogLevel::Warning => counts[2].1 += 1,
                LogLevel::Error => counts[3].1 += 1,
                LogLevel::Critical => counts[4].1 += 1,
            }
        }

        counts.retain(|(_, count)| *count > 0);
        counts
    }
}

pub fn format_log_entry(entry: &LogEntry) -> String {
    format!(
        "[{}] {:?} {}: {}",
        entry.timestamp, entry.level, entry.component, entry.message
    )
}
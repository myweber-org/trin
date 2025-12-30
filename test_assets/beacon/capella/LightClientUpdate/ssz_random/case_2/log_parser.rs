
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct LogParser {
    pattern: Regex,
}

impl LogParser {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] (?P<level>\w+): (?P<message>.+)")?;
        Ok(LogParser { pattern })
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        self.pattern.captures(line).map(|caps| LogEntry {
            timestamp: caps["timestamp"].to_string(),
            level: caps["level"].to_string(),
            message: caps["message"].to_string(),
        })
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        
        let mut entries = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(entry) = self.parse_line(&line) {
                    entries.push(entry);
                }
            }
        }
        
        Ok(entries)
    }
}

pub fn filter_errors(entries: &[LogEntry]) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level == "ERROR")
        .collect()
}use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub fn extract_errors(log_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let error_pattern = Regex::new(r"ERROR.*").unwrap();
    
    let mut errors = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if error_pattern.is_match(&line) {
            errors.push(line);
        }
    }
    
    Ok(errors)
}

pub fn count_errors_by_component(log_path: &str) -> io::Result<std::collections::HashMap<String, usize>> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let component_pattern = Regex::new(r"ERROR.*\[(\w+)\]").unwrap();
    
    let mut component_counts = std::collections::HashMap::new();
    
    for line in reader.lines() {
        let line = line?;
        if let Some(captures) = component_pattern.captures(&line) {
            if let Some(component) = captures.get(1) {
                *component_counts.entry(component.as_str().to_string())
                    .or_insert(0) += 1;
            }
        }
    }
    
    Ok(component_counts)
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    error_patterns: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            error_patterns: HashMap::new(),
        }
    }

    fn parse_log_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
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
                    message: message.clone(),
                };

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;

                if level == "ERROR" {
                    self.extract_error_pattern(&message);
                }
            }
        }

        Ok(())
    }

    fn extract_error_pattern(&mut self, message: &str) {
        let error_regex = Regex::new(r"failed to (\w+)").unwrap();
        if let Some(captures) = error_regex.captures(message) {
            let pattern = format!("failed to {}", &captures[1]);
            *self.error_patterns.entry(pattern).or_insert(0) += 1;
        }
    }

    fn generate_report(&self) {
        println!("Log Analysis Report");
        println!("===================");
        println!("Total entries: {}", self.entries.len());
        println!("\nLog Level Distribution:");

        for (level, count) in &self.level_counts {
            let percentage = (*count as f32 / self.entries.len() as f32) * 100.0;
            println!("  {}: {} ({:.1}%)", level, count, percentage);
        }

        if !self.error_patterns.is_empty() {
            println!("\nCommon Error Patterns:");
            for (pattern, count) in &self.error_patterns {
                println!("  {}: {}", pattern, count);
            }
        }

        if let Some(latest) = self.entries.last() {
            println!("\nLatest log entry:");
            println!("  Time: {}", latest.timestamp);
            println!("  Level: {}", latest.level);
            println!("  Message: {}", latest.message);
        }
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();
    
    if let Err(e) = analyzer.parse_log_file("app.log") {
        eprintln!("Failed to parse log file: {}", e);
        return;
    }
    
    analyzer.generate_report();
}
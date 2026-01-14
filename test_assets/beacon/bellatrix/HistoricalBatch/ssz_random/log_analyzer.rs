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
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    fn parse_log_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    message: captures[3].to_string(),
                };

                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn generate_summary(&self) {
        println!("Log Analysis Summary");
        println!("====================");
        println!("Total entries: {}", self.entries.len());
        println!("\nLog level distribution:");

        for (level, count) in &self.level_counts {
            let percentage = (*count as f64 / self.entries.len() as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", level, count, percentage);
        }

        if let Some(recent) = self.entries.last() {
            println!("\nMost recent log entry:");
            println!("  Time: {}", recent.timestamp);
            println!("  Level: {}", recent.level);
            println!("  Message: {}", recent.message);
        }
    }

    fn find_errors(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == "ERROR")
            .collect()
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();

    if let Err(e) = analyzer.parse_log_file("app.log") {
        eprintln!("Failed to parse log file: {}", e);
        return;
    }

    analyzer.generate_summary();

    let errors = analyzer.find_errors();
    if !errors.is_empty() {
        println!("\nFound {} error entries:", errors.len());
        for error in errors.iter().take(5) {
            println!("  [{}] {}", error.timestamp, error.message);
        }
    }
}
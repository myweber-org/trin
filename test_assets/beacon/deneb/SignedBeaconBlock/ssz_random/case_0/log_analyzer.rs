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
                    message,
                };

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    fn get_level_summary(&self) -> Vec<(String, usize)> {
        let mut summary: Vec<_> = self.level_counts.iter().collect();
        summary.sort_by(|a, b| b.1.cmp(a.1));
        summary.into_iter().map(|(k, v)| (k.clone(), *v)).collect()
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    fn get_total_entries(&self) -> usize {
        self.entries.len()
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();
    
    if let Err(e) = analyzer.parse_log_file("app.log") {
        eprintln!("Failed to parse log file: {}", e);
        return;
    }

    println!("Total log entries: {}", analyzer.get_total_entries());
    println!("\nLog level summary:");
    
    for (level, count) in analyzer.get_level_summary() {
        println!("  {}: {}", level, count);
    }

    let error_logs = analyzer.filter_by_level("ERROR");
    if !error_logs.is_empty() {
        println!("\nRecent ERROR logs:");
        for log in error_logs.iter().take(5) {
            println!("  [{}] {}", log.timestamp, log.message);
        }
    }
}
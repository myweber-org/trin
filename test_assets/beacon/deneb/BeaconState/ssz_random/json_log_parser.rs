
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    services: HashMap<String, usize>,
    time_range: (String, String),
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            services: HashMap::new(),
            time_range: (String::new(), String::new()),
        }
    }
}

struct LogParser {
    min_level: Option<String>,
    service_filter: Option<String>,
    keyword_filter: Option<String>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            min_level: None,
            service_filter: None,
            keyword_filter: None,
        }
    }

    fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    fn with_keyword_filter(mut self, keyword: &str) -> Self {
        self.keyword_filter = Some(keyword.to_lowercase());
        self
    }

    fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<(Vec<LogEntry>, LogSummary), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut summary = LogSummary::new();
        let mut first_timestamp = None;
        let mut last_timestamp = None;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(mut entry) => {
                    if self.filter_entry(&entry) {
                        self.update_summary(&entry, &mut summary, line_num);
                        
                        if first_timestamp.is_none() {
                            first_timestamp = Some(entry.timestamp.clone());
                        }
                        last_timestamp = Some(entry.timestamp.clone());
                        
                        entries.push(entry);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse line {}: {}", line_num + 1, e);
                }
            }
        }

        if let (Some(first), Some(last)) = (first_timestamp, last_timestamp) {
            summary.time_range = (first, last);
        }

        Ok((entries, summary))
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let entry_level = entry.level.to_lowercase();
            let level_priority = |level: &str| match level {
                "error" => 3,
                "warning" => 2,
                "info" => 1,
                "debug" => 0,
                _ => 0,
            };
            
            if level_priority(&entry_level) < level_priority(min_level) {
                return false;
            }
        }

        if let Some(service_filter) = &self.service_filter {
            if &entry.service != service_filter {
                return false;
            }
        }

        if let Some(keyword_filter) = &self.keyword_filter {
            if !entry.message.to_lowercase().contains(keyword_filter) {
                return false;
            }
        }

        true
    }

    fn update_summary(&self, entry: &LogEntry, summary: &mut LogSummary, line_num: usize) {
        summary.total_entries += 1;
        
        match entry.level.to_lowercase().as_str() {
            "error" => summary.error_count += 1,
            "warning" => summary.warning_count += 1,
            _ => {}
        }
        
        *summary.services.entry(entry.service.clone()).or_insert(0) += 1;
    }
}

fn display_summary(summary: &LogSummary) {
    println!("Log Analysis Summary:");
    println!("=====================");
    println!("Total entries: {}", summary.total_entries);
    println!("Errors: {}", summary.error_count);
    println!("Warnings: {}", summary.warning_count);
    println!("\nService distribution:");
    for (service, count) in &summary.services {
        println!("  {}: {}", service, count);
    }
    if !summary.time_range.0.is_empty() && !summary.time_range.1.is_empty() {
        println!("\nTime range: {} to {}", summary.time_range.0, summary.time_range.1);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = LogParser::new()
        .with_min_level("warning")
        .with_service_filter("api-server")
        .with_keyword_filter("timeout");

    let (entries, summary) = parser.parse_file("logs/app.log")?;
    
    println!("Found {} relevant log entries", entries.len());
    display_summary(&summary);
    
    if !entries.is_empty() {
        println!("\nFirst 5 matching entries:");
        for (i, entry) in entries.iter().take(5).enumerate() {
            println!("\n{}. [{}] {} - {}", 
                i + 1, 
                entry.timestamp, 
                entry.level, 
                entry.message
            );
        }
    }
    
    Ok(())
}
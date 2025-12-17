use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub struct LogParser {
    error_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = r"ERROR: (.+)";
        let error_pattern = Regex::new(pattern).unwrap();
        LogParser { error_pattern }
    }

    pub fn parse_file(&self, path: &str) -> io::Result<Vec<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = self.error_pattern.captures(&line) {
                if let Some(error_msg) = captures.get(1) {
                    errors.push(error_msg.as_str().to_string());
                }
            }
        }

        Ok(errors)
    }

    pub fn analyze_errors(&self, errors: &[String]) -> Vec<(String, usize)> {
        let mut error_counts = std::collections::HashMap::new();
        
        for error in errors {
            *error_counts.entry(error.clone()).or_insert(0) += 1;
        }

        let mut sorted_errors: Vec<(String, usize)> = error_counts.into_iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_errors
    }
}

pub fn process_log_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parser = LogParser::new();
    let errors = parser.parse_file(path)?;
    
    if errors.is_empty() {
        println!("No errors found in log file.");
        return Ok(());
    }

    let analysis = parser.analyze_errors(&errors);
    
    println!("Found {} total errors:", errors.len());
    for (error, count) in analysis {
        println!("  {} ({} occurrences)", error, count);
    }

    Ok(())
}
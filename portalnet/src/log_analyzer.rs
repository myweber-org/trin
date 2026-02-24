
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogStats {
    total_lines: usize,
    error_count: usize,
    error_rate: f64,
    top_errors: Vec<(String, usize)>,
}

impl LogStats {
    pub fn new() -> Self {
        LogStats {
            total_lines: 0,
            error_count: 0,
            error_rate: 0.0,
            top_errors: Vec::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut error_map: HashMap<String, usize> = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            self.total_lines += 1;

            if line.to_lowercase().contains("error") {
                self.error_count += 1;
                let error_key = Self::extract_error_type(&line);
                *error_map.entry(error_key).or_insert(0) += 1;
            }
        }

        self.error_rate = if self.total_lines > 0 {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        } else {
            0.0
        };

        self.top_errors = error_map.into_iter().collect();
        self.top_errors.sort_by(|a, b| b.1.cmp(&a.1));
        self.top_errors.truncate(5);

        Ok(())
    }

    fn extract_error_type(line: &str) -> String {
        let words: Vec<&str> = line.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase().contains("error") && i + 1 < words.len() {
                return format!("{} {}", word, words[i + 1]);
            }
        }
        "Unknown error".to_string()
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines processed: {}", self.total_lines);
        println!("Error count: {}", self.error_count);
        println!("Error rate: {:.2}%", self.error_rate);
        println!("\nTop 5 error types:");
        for (i, (error, count)) in self.top_errors.iter().enumerate() {
            println!("{}. {} ({} occurrences)", i + 1, error, count);
        }
    }
}

pub fn analyze_logs<P: AsRef<Path>>(log_path: P) -> Result<LogStats, std::io::Error> {
    let mut analyzer = LogStats::new();
    analyzer.analyze_file(log_path)?;
    Ok(analyzer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: System started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARNING: High memory usage").unwrap();
        writeln!(temp_file, "ERROR: File not found").unwrap();
        writeln!(temp_file, "INFO: Process completed").unwrap();

        let mut analyzer = LogStats::new();
        analyzer.analyze_file(temp_file.path()).unwrap();

        assert_eq!(analyzer.total_lines, 5);
        assert_eq!(analyzer.error_count, 2);
        assert_eq!(analyzer.error_rate, 40.0);
        assert_eq!(analyzer.top_errors.len(), 2);
    }
}
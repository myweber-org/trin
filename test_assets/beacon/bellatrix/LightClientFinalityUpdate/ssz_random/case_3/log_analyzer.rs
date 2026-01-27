
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogStats {
    total_lines: usize,
    error_count: usize,
    error_messages: HashMap<String, usize>,
}

impl LogStats {
    pub fn new() -> Self {
        LogStats {
            total_lines: 0,
            error_count: 0,
            error_messages: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.total_lines += 1;

            if line.to_lowercase().contains("error") {
                self.error_count += 1;
                let message = extract_error_message(&line);
                *self.error_messages.entry(message).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        }
    }

    pub fn top_errors(&self, limit: usize) -> Vec<(&String, &usize)> {
        let mut entries: Vec<_> = self.error_messages.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries.into_iter().take(limit).collect()
    }
}

fn extract_error_message(line: &str) -> String {
    let parts: Vec<&str> = line.split("error:").collect();
    if parts.len() > 1 {
        parts[1].trim().to_string()
    } else {
        line.trim().to_string()
    }
}

pub fn process_log_directory<P: AsRef<Path>>(dir_path: P) -> Result<LogStats, std::io::Error> {
    let mut stats = LogStats::new();
    
    if dir_path.as_ref().is_dir() {
        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                stats.analyze_file(path)?;
            }
        }
    } else {
        stats.analyze_file(dir_path)?;
    }
    
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_error_detection() {
        let mut stats = LogStats::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: System started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "ERROR: Disk space low").unwrap();
        
        stats.analyze_file(temp_file.path()).unwrap();
        
        assert_eq!(stats.total_lines, 4);
        assert_eq!(stats.error_count, 2);
        assert_eq!(stats.error_rate(), 50.0);
    }

    #[test]
    fn test_error_message_extraction() {
        let line = "2024-01-15 ERROR: Connection timeout";
        let message = extract_error_message(line);
        assert_eq!(message, "Connection timeout");
    }
}
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogProcessor {
    error_pattern: Regex,
    warning_pattern: Regex,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            error_pattern: Regex::new(r"ERROR: (.+)").unwrap(),
            warning_pattern: Regex::new(r"WARN: (.+)").unwrap(),
        }
    }

    pub fn process_log_file(&self, file_path: &str) -> Result<HashMap<String, Vec<String>>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut results = HashMap::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;
            
            if let Some(cap) = self.error_pattern.captures(&line) {
                errors.push(format!("Line {}: {}", line_num + 1, cap[1].to_string()));
            }
            
            if let Some(cap) = self.warning_pattern.captures(&line) {
                warnings.push(format!("Line {}: {}", line_num + 1, cap[1].to_string()));
            }
        }

        results.insert("errors".to_string(), errors);
        results.insert("warnings".to_string(), warnings);
        
        Ok(results)
    }

    pub fn count_severity_levels(&self, results: &HashMap<String, Vec<String>>) -> HashMap<String, usize> {
        results.iter()
            .map(|(key, value)| (key.clone(), value.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let processor = LogProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Disk space running low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage detected").unwrap();
        
        let results = processor.process_log_file(temp_file.path().to_str().unwrap()).unwrap();
        let counts = processor.count_severity_levels(&results);
        
        assert_eq!(counts.get("errors").unwrap(), &1);
        assert_eq!(counts.get("warnings").unwrap(), &2);
        assert_eq!(results.get("errors").unwrap()[0], "Line 3: Database connection failed");
    }
}
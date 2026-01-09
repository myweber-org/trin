
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: String,
    service_filter: Option<String>,
    keyword_filters: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: &str) -> Self {
        LogProcessor {
            min_level: min_level.to_lowercase(),
            service_filter: None,
            keyword_filters: Vec::new(),
        }
    }

    pub fn set_service_filter(&mut self, service: &str) {
        self.service_filter = Some(service.to_string());
    }

    pub fn add_keyword_filter(&mut self, keyword: &str) {
        self.keyword_filters.push(keyword.to_lowercase());
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            if let Some(entry) = self.parse_line(&line) {
                if self.passes_filters(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() != 5 {
            return None;
        }

        let mut metadata = HashMap::new();
        let meta_parts: Vec<&str> = parts[4].split(',').collect();
        for meta in meta_parts {
            let kv: Vec<&str> = meta.splitn(2, '=').collect();
            if kv.len() == 2 {
                metadata.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }

        Some(LogEntry {
            timestamp: parts[0].trim().to_string(),
            level: parts[1].trim().to_string(),
            service: parts[2].trim().to_string(),
            message: parts[3].trim().to_string(),
            metadata,
        })
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        let level_order = ["debug", "info", "warn", "error", "critical"];
        let entry_level_idx = level_order.iter()
            .position(|&l| l == entry.level.to_lowercase())
            .unwrap_or(0);
        let min_level_idx = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);

        if entry_level_idx < min_level_idx {
            return false;
        }

        if let Some(ref service) = self.service_filter {
            if entry.service.to_lowercase() != service.to_lowercase() {
                return false;
            }
        }

        if !self.keyword_filters.is_empty() {
            let msg_lower = entry.message.to_lowercase();
            if !self.keyword_filters.iter().any(|k| msg_lower.contains(k)) {
                return false;
            }
        }

        true
    }

    pub fn format_entries(&self, entries: &[LogEntry], format: &str) -> String {
        let mut output = String::new();
        
        for entry in entries {
            let formatted = match format {
                "json" => self.format_json(entry),
                "csv" => self.format_csv(entry),
                "simple" => self.format_simple(entry),
                _ => self.format_simple(entry),
            };
            output.push_str(&formatted);
            output.push('\n');
        }
        
        output
    }

    fn format_json(&self, entry: &LogEntry) -> String {
        let mut meta_json = String::from("{");
        for (i, (k, v)) in entry.metadata.iter().enumerate() {
            if i > 0 {
                meta_json.push(',');
            }
            meta_json.push_str(&format!("\"{}\":\"{}\"", k, v));
        }
        meta_json.push('}');

        format!(
            "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"service\":\"{}\",\"message\":\"{}\",\"metadata\":{}}}",
            entry.timestamp, entry.level, entry.service, entry.message, meta_json
        )
    }

    fn format_csv(&self, entry: &LogEntry) -> String {
        let meta_str = entry.metadata.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(";");
        
        format!(
            "{},{},{},{},{}",
            entry.timestamp, entry.level, entry.service, entry.message, meta_str
        )
    }

    fn format_simple(&self, entry: &LogEntry) -> String {
        format!(
            "[{}] {} {}: {}",
            entry.timestamp, entry.level, entry.service, entry.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut processor = LogProcessor::new("info");
        processor.set_service_filter("api");
        processor.add_keyword_filter("error");

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-05T10:30:00Z|INFO|api|Request processed|user_id=123,method=GET").unwrap();
        writeln!(temp_file, "2023-10-05T10:31:00Z|ERROR|api|Database connection failed|user_id=456,retry_count=3").unwrap();
        writeln!(temp_file, "2023-10-05T10:32:00Z|DEBUG|auth|Token validation|user_id=789").unwrap();

        let entries = processor.process_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "ERROR");
        assert_eq!(entries[0].message, "Database connection failed");
    }

    #[test]
    fn test_level_filtering() {
        let processor = LogProcessor::new("warn");
        
        let test_entry = LogEntry {
            timestamp: "2023-10-05T10:30:00Z".to_string(),
            level: "INFO".to_string(),
            service: "test".to_string(),
            message: "Test message".to_string(),
            metadata: HashMap::new(),
        };

        assert!(!processor.passes_filters(&test_entry));
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    Json(serde_json::Error),
    MalformedLine(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO error: {}", e),
            ParseError::Json(e) => write!(f, "JSON parsing error: {}", e),
            ParseError::MalformedLine(line) => write!(f, "Malformed log line: {}", line),
        }
    }
}

impl Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::Io(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::Json(err)
    }
}

pub struct LogParser {
    reader: BufReader<File>,
}

impl LogParser {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(LogParser { reader })
    }

    pub fn parse_next(&mut self) -> Option<Result<LogEntry, ParseError>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    self.parse_next()
                } else {
                    Some(self.parse_line(trimmed))
                }
            }
            Err(e) => Some(Err(ParseError::Io(e))),
        }
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, ParseError> {
        let entry: LogEntry = serde_json::from_str(line)?;
        
        if entry.timestamp.is_empty() || entry.level.is_empty() || entry.message.is_empty() {
            return Err(ParseError::MalformedLine(line.to_string()));
        }
        
        Ok(entry)
    }
}

pub fn filter_logs_by_level<P: AsRef<Path>>(
    path: P,
    level: &str,
) -> Result<Vec<LogEntry>, ParseError> {
    let mut parser = LogParser::new(path)?;
    let mut filtered = Vec::new();

    while let Some(result) = parser.parse_next() {
        match result {
            Ok(entry) if entry.level.to_lowercase() == level.to_lowercase() => {
                filtered.push(entry);
            }
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let json_line = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"Service started","service":"api"}"#;
        let parser = LogParser::new("dummy").unwrap();
        let entry = parser.parse_line(json_line).unwrap();
        
        assert_eq!(entry.timestamp, "2023-10-01T12:00:00Z");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Service started");
        assert_eq!(entry.extra.get("service").unwrap().as_str().unwrap(), "api");
    }

    #[test]
    fn test_filter_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2023-10-01T12:00:00Z","level":"ERROR","message":"Failed to connect"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2023-10-01T12:01:00Z","level":"INFO","message":"Connected successfully"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2023-10-01T12:02:00Z","level":"ERROR","message":"Timeout occurred"}}"#).unwrap();
        
        let errors = filter_logs_by_level(temp_file.path(), "ERROR").unwrap();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].message, "Failed to connect");
        assert_eq!(errors[1].message, "Timeout occurred");
    }
}
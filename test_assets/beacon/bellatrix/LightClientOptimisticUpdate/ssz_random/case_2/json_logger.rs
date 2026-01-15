
use serde::Serialize;
use std::io::{self, Write};
use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

#[derive(Serialize)]
struct LogEntry<'a> {
    timestamp: String,
    level: &'static str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_data: Option<serde_json::Value>,
}

pub struct JsonLogger {
    min_level: LogLevel,
    output: Box<dyn Write + Send>,
}

impl JsonLogger {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            output: Box::new(io::stdout()),
        }
    }

    pub fn with_output<W: Write + Send + 'static>(min_level: LogLevel, output: W) -> Self {
        Self {
            min_level,
            output: Box::new(output),
        }
    }

    pub fn log(&mut self, level: LogLevel, message: &str) -> io::Result<()> {
        if level as u8 <= self.min_level as u8 {
            let entry = LogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: level.as_str(),
                message,
                module: None,
                additional_data: None,
            };

            let json = serde_json::to_string(&entry)?;
            writeln!(self.output, "{}", json)?;
        }
        Ok(())
    }

    pub fn log_with_context(
        &mut self,
        level: LogLevel,
        message: &str,
        module: Option<&str>,
        additional_data: Option<serde_json::Value>,
    ) -> io::Result<()> {
        if level as u8 <= self.min_level as u8 {
            let entry = LogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: level.as_str(),
                message,
                module,
                additional_data,
            };

            let json = serde_json::to_string(&entry)?;
            writeln!(self.output, "{}", json)?;
        }
        Ok(())
    }

    pub fn error(&mut self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Error, message)
    }

    pub fn warn(&mut self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Warn, message)
    }

    pub fn info(&mut self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Info, message)
    }

    pub fn debug(&mut self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Debug, message)
    }

    pub fn trace(&mut self, message: &str) -> io::Result<()> {
        self.log(LogLevel::Trace, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_level_filtering() {
        let mut buffer = Vec::new();
        let mut logger = JsonLogger::with_output(LogLevel::Info, &mut buffer);

        logger.debug("This should not appear").unwrap();
        logger.info("This should appear").unwrap();
        logger.error("This should also appear").unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let lines: Vec<&str> = output.trim().split('\n').collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("INFO"));
        assert!(lines[1].contains("ERROR"));
    }

    #[test]
    fn test_log_with_context() {
        let mut buffer = Vec::new();
        let mut logger = JsonLogger::with_output(LogLevel::Debug, &mut buffer);

        let data = json!({
            "user_id": 42,
            "action": "login"
        });

        logger
            .log_with_context(
                LogLevel::Info,
                "User action performed",
                Some("auth"),
                Some(data),
            )
            .unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["level"], "INFO");
        assert_eq!(parsed["message"], "User action performed");
        assert_eq!(parsed["module"], "auth");
        assert_eq!(parsed["additional_data"]["user_id"], 42);
    }
}
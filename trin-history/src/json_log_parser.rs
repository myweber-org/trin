use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    filters: Vec<Filter>,
    format_options: FormatOptions,
}

#[derive(Debug, Clone)]
pub struct Filter {
    field: String,
    value: Value,
    operator: FilterOperator,
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
}

pub struct FormatOptions {
    pub show_timestamp: bool,
    pub show_level: bool,
    pub show_fields: bool,
    pub indent: usize,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: Vec::new(),
            format_options: FormatOptions {
                show_timestamp: true,
                show_level: true,
                show_fields: false,
                indent: 2,
            },
        }
    }

    pub fn add_filter(&mut self, filter: Filter) -> &mut Self {
        self.filters.push(filter);
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                if self.passes_filters(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;
        
        let timestamp = json.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let level = json.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("INFO")
            .to_string();

        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        for filter in &self.filters {
            if !filter.matches(entry) {
                return false;
            }
        }
        true
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut parts = Vec::new();

        if self.format_options.show_timestamp && !entry.timestamp.is_empty() {
            parts.push(format!("[{}]", entry.timestamp));
        }

        if self.format_options.show_level {
            parts.push(format!("{}:", entry.level));
        }

        parts.push(entry.message.clone());

        if self.format_options.show_fields && !entry.fields.is_empty() {
            let fields_str = entry.fields.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("{{{}}}", fields_str));
        }

        parts.join(" ")
    }

    pub fn set_format_options(&mut self, options: FormatOptions) -> &mut Self {
        self.format_options = options;
        self
    }
}

impl Filter {
    pub fn new(field: String, value: Value, operator: FilterOperator) -> Self {
        Filter { field, value, operator }
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        match self.field.as_str() {
            "level" => self.compare_value(&entry.level),
            "message" => self.compare_value(&entry.message),
            _ => {
                if let Some(field_value) = entry.fields.get(&self.field) {
                    self.compare_json_value(field_value)
                } else {
                    false
                }
            }
        }
    }

    fn compare_value(&self, actual: &str) -> bool {
        match self.operator {
            FilterOperator::Equals => actual == self.value.as_str().unwrap_or(""),
            FilterOperator::Contains => actual.contains(self.value.as_str().unwrap_or("")),
            _ => false,
        }
    }

    fn compare_json_value(&self, actual: &Value) -> bool {
        match self.operator {
            FilterOperator::Equals => actual == &self.value,
            FilterOperator::Contains => {
                if let Some(actual_str) = actual.as_str() {
                    if let Some(filter_str) = self.value.as_str() {
                        actual_str.contains(filter_str)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            FilterOperator::GreaterThan => {
                if let (Some(actual_num), Some(filter_num)) = (actual.as_f64(), self.value.as_f64()) {
                    actual_num > filter_num
                } else {
                    false
                }
            }
            FilterOperator::LessThan => {
                if let (Some(actual_num), Some(filter_num)) = (actual.as_f64(), self.value.as_f64()) {
                    actual_num < filter_num
                } else {
                    false
                }
            }
        }
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}
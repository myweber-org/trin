use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub features: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: String::from("postgresql://localhost:5432"),
            max_connections: 10,
            timeout_seconds: 30,
            features: vec![String::from("logging"), String::from("caching")],
            metadata: HashMap::new(),
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::default();
        let mut current_section = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len()-1].to_string();
                continue;
            }

            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos+1..].trim().to_string();

                match (current_section.as_str(), key.as_str()) {
                    ("database", "url") => config.database_url = value,
                    ("database", "max_connections") => {
                        config.max_connections = value.parse()
                            .map_err(|_| format!("Invalid max_connections at line {}", line_num + 1))?
                    },
                    ("network", "timeout") => {
                        config.timeout_seconds = value.parse()
                            .map_err(|_| format!("Invalid timeout at line {}", line_num + 1))?
                    },
                    ("features", _) => {
                        if !config.features.contains(&value) {
                            config.features.push(value);
                        }
                    },
                    ("metadata", _) => {
                        config.metadata.insert(key, value);
                    },
                    _ => return Err(format!("Unknown configuration key '{}' at line {}", key, line_num + 1)),
                }
            } else {
                return Err(format!("Invalid configuration line {}: {}", line_num + 1, line));
            }
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }

        if self.max_connections == 0 {
            errors.push("Max connections must be greater than 0".to_string());
        }

        if self.timeout_seconds > 3600 {
            errors.push("Timeout cannot exceed 3600 seconds".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();
        
        output.push_str("[database]\n");
        output.push_str(&format!("url = {}\n", self.database_url));
        output.push_str(&format!("max_connections = {}\n\n", self.max_connections));
        
        output.push_str("[network]\n");
        output.push_str(&format!("timeout = {}\n\n", self.timeout_seconds));
        
        output.push_str("[features]\n");
        for feature in &self.features {
            output.push_str(&format!("enabled = {}\n", feature));
        }
        
        if !self.metadata.is_empty() {
            output.push_str("\n[metadata]\n");
            for (key, value) in &self.metadata {
                output.push_str(&format!("{} = {}\n", key, value));
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database_url, "postgresql://localhost:5432");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.features.contains(&"logging".to_string()));
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        config.database_url = String::new();
        assert!(config.validate().is_err());

        config.database_url = "postgresql://localhost:5432".to_string();
        config.max_connections = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_parsing() {
        let config_content = r#"
[database]
url = postgresql://prod:5432
max_connections = 20

[network]
timeout = 60

[features]
enabled = monitoring
enabled = analytics

[metadata]
version = 1.0.0
environment = production
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.database_url, "postgresql://prod:5432");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.features.contains(&"monitoring".to_string()));
        assert!(config.features.contains(&"analytics".to_string()));
        assert_eq!(config.metadata.get("version"), Some(&"1.0.0".to_string()));
    }
}
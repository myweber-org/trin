
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub max_connections: u32,
    pub debug_mode: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut variables = HashMap::new();
        for (key, value) in env::vars() {
            variables.insert(key, value);
        }

        let mut config_map = HashMap::new();
        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let mut value = parts[1].trim().to_string();

            for (var_name, var_value) in &variables {
                let placeholder = format!("${{{}}}", var_name);
                value = value.replace(&placeholder, var_value);
            }

            config_map.insert(key, value);
        }

        let database_url = config_map
            .get("DATABASE_URL")
            .ok_or("Missing DATABASE_URL")?
            .clone();

        let api_key = config_map
            .get("API_KEY")
            .ok_or("Missing API_KEY")?
            .clone();

        let max_connections = config_map
            .get("MAX_CONNECTIONS")
            .ok_or("Missing MAX_CONNECTIONS")?
            .parse::<u32>()
            .map_err(|e| format!("Invalid MAX_CONNECTIONS: {}", e))?;

        let debug_mode = config_map
            .get("DEBUG_MODE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Config {
            database_url,
            api_key,
            max_connections,
            debug_mode,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL cannot be empty".to_string());
        }

        if self.api_key.len() < 16 {
            return Err("API_KEY must be at least 16 characters".to_string());
        }

        if self.max_connections == 0 {
            return Err("MAX_CONNECTIONS must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "API_KEY=my_secret_key_123456").unwrap();
        writeln!(temp_file, "MAX_CONNECTIONS=10").unwrap();
        writeln!(temp_file, "DEBUG_MODE=true").unwrap();

        env::set_var("CUSTOM_VAR", "replaced_value");
        writeln!(temp_file, "CUSTOM_SETTING=${{CUSTOM_VAR}}").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.api_key, "my_secret_key_123456");
        assert_eq!(config.max_connections, 10);
        assert!(config.debug_mode);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            api_key: "short".to_string(),
            max_connections: 0,
            debug_mode: false,
        };

        assert!(config.validate().is_err());
    }
}
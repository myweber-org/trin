use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = Self::resolve_value(parts[1].trim());
                config_map.insert(key, value);
            }
        }

        Ok(Config {
            database_url: config_map
                .get("DATABASE_URL")
                .cloned()
                .unwrap_or_else(|| "postgres://localhost:5432".to_string()),
            port: config_map
                .get("PORT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            debug_mode: config_map
                .get("DEBUG")
                .map(|v| v == "true")
                .unwrap_or(false),
            api_keys: config_map
                .iter()
                .filter(|(k, _)| k.starts_with("API_KEY_"))
                .map(|(k, v)| (k[8..].to_string(), v.clone()))
                .collect(),
        })
    }

    fn resolve_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://prod:5432").unwrap();
        writeln!(file, "PORT=3000").unwrap();
        writeln!(file, "DEBUG=true").unwrap();
        writeln!(file, "API_KEY_WEATHER=abc123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://prod:5432");
        assert_eq!(config.port, 3000);
        assert!(config.debug_mode);
        assert_eq!(config.api_keys.get("WEATHER"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_env_variable_resolution() {
        env::set_var("SECRET_KEY", "env_value");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "API_KEY=$SECRET_KEY").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.api_keys.get(""), Some(&"env_value".to_string()));
    }
}
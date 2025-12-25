use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                settings.insert(key, value);
            }
        }

        Self::from_map(&settings)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut settings = HashMap::new();
        settings.insert("DATABASE_URL".to_string(), env::var("DATABASE_URL").unwrap_or_default());
        settings.insert("API_KEY".to_string(), env::var("API_KEY").unwrap_or_default());
        settings.insert("DEBUG_MODE".to_string(), env::var("DEBUG_MODE").unwrap_or_default());
        settings.insert("PORT".to_string(), env::var("PORT").unwrap_or_default());

        Self::from_map(&settings)
    }

    fn from_map(settings: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = settings.get("DATABASE_URL")
            .ok_or("DATABASE_URL not found")?
            .to_string();

        let api_key = settings.get("API_KEY")
            .ok_or("API_KEY not found")?
            .to_string();

        let debug_mode = settings.get("DEBUG_MODE")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let port = settings.get("PORT")
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL cannot be empty".to_string());
        }
        if self.api_key.is_empty() {
            return Err("API_KEY cannot be empty".to_string());
        }
        if self.port == 0 {
            return Err("PORT must be greater than 0".to_string());
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
    fn test_config_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();
        writeln!(file, "DEBUG_MODE=true").unwrap();
        writeln!(file, "PORT=3000").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.api_key, "secret123");
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            api_key: "key".to_string(),
            debug_mode: false,
            port: 8080,
        };
        assert!(config.validate().is_err());
    }
}
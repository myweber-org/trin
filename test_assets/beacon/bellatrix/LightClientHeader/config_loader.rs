use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                self.settings.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn load_from_env(&mut self, prefix: &str) {
        for (key, value) in env::vars() {
            if key.starts_with(prefix) {
                let config_key = key.trim_start_matches(prefix).to_lowercase();
                self.settings.insert(config_key, value);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_loading() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "server_host=localhost\nserver_port=8080").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("server_port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_env_loading() {
        env::set_var("APP_DATABASE_URL", "postgres://localhost/test");
        
        let mut config = Config::new();
        config.load_from_env("APP_");
        
        assert_eq!(config.get("database_url"), Some(&"postgres://localhost/test".to_string()));
    }
}
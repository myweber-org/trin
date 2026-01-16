use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let mut value = value.trim().to_string();

                if value.starts_with("${") && value.ends_with('}') {
                    let env_var = &value[2..value.len() - 1];
                    value = env::var(env_var).unwrap_or_else(|_| String::new());
                }

                values.insert(key, value);
            }
        }

        Ok(Config { values })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_HOST=localhost").unwrap();
        writeln!(file, "DATABASE_PORT=5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_env_variable_expansion() {
        env::set_var("APP_SECRET", "env_secret");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${APP_SECRET}").unwrap();
        writeln!(file, "MISSING=${NON_EXISTENT_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET"), Some(&"env_secret".to_string()));
        assert_eq!(config.get("MISSING"), Some(&"".to_string()));
    }
}

use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = HashMap::new();
        let var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = line.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                let mut processed_value = value.to_string();
                for cap in var_regex.captures_iter(value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            processed_value = processed_value.replace(&cap[0], &env_value);
                        }
                    }
                }

                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
            host=localhost
            port=8080
            # This is a comment
            timeout=30
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("host").unwrap(), "localhost");
        assert_eq!(config.get("port").unwrap(), "8080");
        assert_eq!(config.get("timeout").unwrap(), "30");
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "postgres-server");
        
        let content = r#"
            database_host=${DB_HOST}
            fallback_value=${NON_EXISTENT_VAR}
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("database_host").unwrap(), "postgres-server");
        assert_eq!(config.get("fallback_value").unwrap(), "");
    }

    #[test]
    fn test_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "key1=value1\nkey2=value2").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("key1").unwrap(), "value1");
        assert_eq!(config.get("key2").unwrap(), "value2");
    }
}
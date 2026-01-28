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
                let processed_value = Self::substitute_env_vars(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &env_value);
            }
        }
        result
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
    use std::env;

    #[test]
    fn test_basic_parsing() {
        let test_content = "HOST=localhost\nPORT=8080\n# Comment\nDEBUG=true";
        let temp_path = "test_config.tmp";
        fs::write(temp_path, test_content).unwrap();

        let config = Config::from_file(temp_path).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.get("MISSING"), None);

        fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_ENV", "production");
        let test_content = "ENVIRONMENT=${APP_ENV}\nHOST=api.${APP_ENV}.example.com";
        let temp_path = "test_env.tmp";
        fs::write(temp_path, test_content).unwrap();

        let config = Config::from_file(temp_path).unwrap();
        assert_eq!(config.get("ENVIRONMENT"), Some(&"production".to_string()));
        assert_eq!(
            config.get("HOST"),
            Some(&"api.production.example.com".to_string())
        );

        fs::remove_file(temp_path).unwrap();
        env::remove_var("APP_ENV");
    }
}
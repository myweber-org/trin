
use std::collections::HashMap;
use std::env;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        let mut in_braces = false;
        let mut var_name = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip the '{'
                in_braces = true;
                var_name.clear();
                continue;
            }

            if in_braces {
                if ch == '}' {
                    in_braces = false;
                    if let Ok(env_value) = env::var(&var_name) {
                        result.push_str(&env_value);
                    }
                    var_name.clear();
                } else {
                    var_name.push(ch);
                }
            } else {
                result.push(ch);
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = "DATABASE_URL=postgres://localhost:5432/mydb\nDEBUG=true\n";
        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432/mydb");
        assert_eq!(config.get("DEBUG").unwrap(), "true");
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        let content = "PORT=${APP_PORT}\nHOST=localhost";
        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost");
    }

    #[test]
    fn test_file_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "API_KEY=secret123\nTIMEOUT=30").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
    }

    #[test]
    fn test_default_value() {
        let content = "EXISTING=value";
        let config = Config::parse(content).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}
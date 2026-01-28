use std::collections::HashMap;
use std::env;
use std::fs;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut parser = ConfigParser {
            values: HashMap::new(),
        };
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = parser.process_value(value.trim());
                parser.values.insert(key, processed_value);
            }
        }
        
        Ok(parser)
    }
    
    fn process_value(&self, value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else if let Some(config_value) = self.values.get(&var_name) {
                    result.push_str(config_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
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
    
    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/mydb").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();
        
        let parser = ConfigParser::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(parser.get("DATABASE_URL").unwrap(), "postgres://localhost/mydb");
        assert_eq!(parser.get("API_KEY").unwrap(), "secret123");
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${APP_PORT}").unwrap();
        writeln!(file, "HOST=localhost:${PORT}").unwrap();
        
        let parser = ConfigParser::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(parser.get("PORT").unwrap(), "8080");
        assert_eq!(parser.get("HOST").unwrap(), "localhost:8080");
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

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

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut config_map = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                config_map.insert(key.trim_start_matches("APP_").to_string(), value);
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map
            .get("DATABASE_URL")
            .ok_or("Missing DATABASE_URL")?
            .to_string();

        let api_key = map
            .get("API_KEY")
            .ok_or("Missing API_KEY")?
            .to_string();

        let debug_mode = map
            .get("DEBUG_MODE")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let port = map
            .get("PORT")
            .map(|s| s.parse::<u16>().unwrap_or(8080))
            .unwrap_or(8080);

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }

    pub fn merge(self, other: Self) -> Self {
        Config {
            database_url: other.database_url,
            api_key: other.api_key,
            debug_mode: other.debug_mode || self.debug_mode,
            port: other.port,
        }
    }
}
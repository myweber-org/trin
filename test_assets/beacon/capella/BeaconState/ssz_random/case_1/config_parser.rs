
use std::collections::HashMap;
use std::env;
use std::fs;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        let re = Regex::new(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*?)\s*$").unwrap();
        let var_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();
                
                value = var_re.replace_all(&value, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    env::var(var_name).unwrap_or_else(|_| {
                        eprintln!("Warning: Environment variable {} not found", var_name);
                        String::new()
                    })
                }).to_string();
                
                if value.starts_with('"') && value.ends_with('"') {
                    value = value[1..value.len()-1].to_string();
                }
                
                values.insert(key, value);
            } else {
                return Err(format!("Invalid config syntax at line {}", line_num + 1));
            }
        }
        
        Ok(Config { values })
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
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
        writeln!(file, "DATABASE_HOST=localhost").unwrap();
        writeln!(file, "DATABASE_PORT=5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "APP_NAME=\"My Application\"").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("APP_NAME"), Some(&"My Application".to_string()));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }
    
    #[test]
    fn test_env_var_substitution() {
        env::set_var("DB_USER", "admin");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "USERNAME=${DB_USER}").unwrap();
        writeln!(file, "PASSWORD=${NON_EXISTENT_VAR}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("USERNAME"), Some(&"admin".to_string()));
        assert_eq!(config.get("PASSWORD"), Some(&"".to_string()));
    }
    
    #[test]
    fn test_invalid_syntax() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "INVALID LINE").unwrap();
        
        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}
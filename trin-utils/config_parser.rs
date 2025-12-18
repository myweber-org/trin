use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.*?)\s*$").unwrap();
        let env_re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();

                for env_cap in env_re.captures_iter(&value) {
                    let env_var = &env_cap[1];
                    if let Ok(env_value) = env::var(env_var) {
                        value = value.replace(&env_cap[0], &env_value);
                    } else {
                        return Err(format!(
                            "Line {}: Environment variable '{}' not found",
                            line_num + 1,
                            env_var
                        ));
                    }
                }

                self.values.insert(key, value);
            } else {
                return Err(format!("Line {}: Invalid syntax", line_num + 1));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
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

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = "DATABASE_HOST=localhost\nDATABASE_PORT=5432\n";
        parser.load_from_str(config).unwrap();

        assert_eq!(parser.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(parser.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "my_secret_key");
        
        let mut parser = ConfigParser::new();
        let config = "SECRET_KEY=${APP_SECRET}\nHOST=localhost";
        parser.load_from_str(config).unwrap();

        assert_eq!(parser.get("SECRET_KEY"), Some(&"my_secret_key".to_string()));
        assert_eq!(parser.get("HOST"), Some(&"localhost".to_string()));
    }

    #[test]
    fn test_invalid_syntax() {
        let mut parser = ConfigParser::new();
        let config = "INVALID LINE";
        let result = parser.load_from_str(config);
        assert!(result.is_err());
    }
}
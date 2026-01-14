use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sections: HashMap::new(),
        }
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let mut config = Config::new();
        let mut current_section = String::from("default");
        let mut section_map = HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if !section_map.is_empty() {
                    config.sections.insert(current_section.clone(), section_map);
                    section_map = HashMap::new();
                }
                current_section = trimmed[1..trimmed.len()-1].to_string();
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::resolve_env_vars(&raw_value);
            
            section_map.insert(key, value);
        }

        if !section_map.is_empty() {
            config.sections.insert(current_section, section_map);
        }

        Ok(config)
    }

    fn resolve_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                match env::var(&var_name) {
                    Ok(env_value) => result.push_str(&env_value),
                    Err(_) => result.push_str(&format!("${{{}}}", var_name)),
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.get(key)
    }

    pub fn section_keys(&self, section: &str) -> Option<Vec<&String>> {
        self.sections.get(section).map(|map| map.keys().collect())
    }

    pub fn all_sections(&self) -> Vec<&String> {
        self.sections.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
[database]
host = localhost
port = 5432

[server]
address = 0.0.0.0
port = 8080
"#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("database", "host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("server", "port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("APP_PORT", "3000");
        
        let content = r#"
[app]
port = ${APP_PORT}
name = test_app
"#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("app", "port"), Some(&"3000".to_string()));
    }
}
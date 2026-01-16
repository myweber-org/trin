
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::interpolate_env_vars(&raw_value);
            settings.insert(key, value);
        }

        let database_url = settings
            .get("DATABASE_URL")
            .ok_or("Missing DATABASE_URL setting")?
            .clone();

        let port = settings
            .get("PORT")
            .ok_or("Missing PORT setting")?
            .parse::<u16>()
            .map_err(|e| format!("Invalid PORT value: {}", e))?;

        let log_level = settings
            .get("LOG_LEVEL")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "info".to_string());

        let cache_size = settings
            .get("CACHE_SIZE")
            .map(|s| s.parse::<usize>().unwrap_or(1000))
            .unwrap_or(1000);

        Ok(Config {
            database_url,
            port,
            log_level,
            cache_size,
        })
    }

    fn interpolate_env_vars(value: &str) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(config_file, "PORT=5432").unwrap();
        writeln!(config_file, "# This is a comment").unwrap();
        writeln!(config_file, "LOG_LEVEL=debug").unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.port, 5432);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_size, 1000);
    }

    #[test]
    fn test_env_var_interpolation() {
        env::set_var("DB_HOST", "localhost");
        let interpolated = Config::interpolate_env_vars("postgres://${DB_HOST}/db");
        assert_eq!(interpolated, "postgres://localhost/db");
    }
}

use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::interpolate_env_vars(&raw_value);
            settings.insert(key, value);
        }

        Ok(Config {
            database_url: settings
                .get("DATABASE_URL")
                .ok_or("Missing DATABASE_URL")?
                .clone(),
            server_port: settings
                .get("SERVER_PORT")
                .ok_or("Missing SERVER_PORT")?
                .parse()
                .map_err(|_| "Invalid SERVER_PORT value")?,
            log_level: settings
                .get("LOG_LEVEL")
                .unwrap_or(&"info".to_string())
                .clone(),
            cache_size: settings
                .get("CACHE_SIZE")
                .unwrap_or(&"1000".to_string())
                .parse()
                .unwrap_or(1000),
        })
    }

    fn interpolate_env_vars(value: &str) -> String {
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

                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
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
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "SERVER_PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "LOG_LEVEL=debug").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_size, 1000);
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("DB_HOST", "localhost");
        let value = Config::interpolate_env_vars("postgres://${DB_HOST}/db");
        assert_eq!(value, "postgres://localhost/db");
    }
}
use std::collections::HashMap;
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

        let mut parsed = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = Self::resolve_value(value.trim());
                parsed.insert(key, value);
            }
        }

        Ok(Config {
            database_url: parsed
                .get("DATABASE_URL")
                .ok_or("Missing DATABASE_URL")?
                .clone(),
            api_key: parsed
                .get("API_KEY")
                .ok_or("Missing API_KEY")?
                .clone(),
            debug_mode: parsed
                .get("DEBUG")
                .map(|s| s == "true")
                .unwrap_or(false),
            port: parsed
                .get("PORT")
                .map(|s| s.parse().unwrap_or(8080))
                .unwrap_or(8080),
        })
    }

    fn resolve_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "API_KEY=$SECRET_KEY").unwrap();
        writeln!(file, "DEBUG=true").unwrap();
        writeln!(file, "PORT=3000").unwrap();

        env::set_var("SECRET_KEY", "abc123");
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.api_key, "abc123");
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.port, 3000);
    }
}
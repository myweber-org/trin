
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use url::Url;

lazy_static! {
    static ref STORAGE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

const SHORT_CODE_LENGTH: usize = 7;
const BASE_URL: &str = "https://short.url/";

pub fn shorten(original_url: &str) -> Result<String, String> {
    let parsed_url = Url::parse(original_url)
        .map_err(|_| "Invalid URL provided".to_string())?;

    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err("Only HTTP and HTTPS URLs are supported".to_string());
    }

    let short_code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SHORT_CODE_LENGTH)
        .map(char::from)
        .collect();

    let short_url = format!("{}{}", BASE_URL, short_code);

    STORAGE.write()
        .map_err(|_| "Storage lock poisoned".to_string())?
        .insert(short_code.clone(), original_url.to_string());

    Ok(short_url)
}

pub fn resolve(short_url: &str) -> Option<String> {
    let short_code = short_url.strip_prefix(BASE_URL)?;
    
    STORAGE.read()
        .ok()
        .and_then(|storage| storage.get(short_code).cloned())
}

pub fn get_stats() -> usize {
    STORAGE.read()
        .map(|storage| storage.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_shortening() {
        let url = "https://www.rust-lang.org";
        let result = shorten(url);
        assert!(result.is_ok());
        
        let short_url = result.unwrap();
        assert!(short_url.starts_with(BASE_URL));
        assert_eq!(short_url.len(), BASE_URL.len() + SHORT_CODE_LENGTH);
    }

    #[test]
    fn test_invalid_url_scheme() {
        let url = "ftp://example.com/file.txt";
        let result = shorten(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolution() {
        let original = "https://docs.rs";
        let short_url = shorten(original).unwrap();
        let resolved = resolve(&short_url);
        
        assert_eq!(resolved, Some(original.to_string()));
    }

    #[test]
    fn test_stats_counter() {
        let initial_count = get_stats();
        
        let _ = shorten("https://example1.com");
        let _ = shorten("https://example2.com");
        
        assert_eq!(get_stats(), initial_count + 2);
    }
}
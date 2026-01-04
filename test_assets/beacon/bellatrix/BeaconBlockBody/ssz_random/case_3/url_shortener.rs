
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

lazy_static! {
    static ref URL_STORE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref URL_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
}

pub struct UrlShortener;

impl UrlShortener {
    pub fn shorten(url: &str) -> Result<String, &'static str> {
        if !URL_REGEX.is_match(url) {
            return Err("Invalid URL format");
        }

        let key: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        URL_STORE.write().unwrap().insert(key.clone(), url.to_string());
        Ok(key)
    }

    pub fn resolve(key: &str) -> Option<String> {
        URL_STORE.read().unwrap().get(key).cloned()
    }

    pub fn list_all() -> Vec<(String, String)> {
        URL_STORE.read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn delete(key: &str) -> bool {
        URL_STORE.write().unwrap().remove(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_shortening() {
        let url = "https://www.example.com";
        let result = UrlShortener::shorten(url);
        assert!(result.is_ok());
        
        let key = result.unwrap();
        let resolved = UrlShortener::resolve(&key);
        assert_eq!(resolved, Some(url.to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let result = UrlShortener::shorten("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_key() {
        let result = UrlShortener::resolve("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_functionality() {
        let url = "https://www.test.com";
        let key = UrlShortener::shorten(url).unwrap();
        
        assert!(UrlShortener::delete(&key));
        assert!(UrlShortener::resolve(&key).is_none());
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use regex::Regex;
use nanoid::nanoid;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
}

#[derive(Clone)]
pub struct UrlShortener {
    storage: Arc<Mutex<HashMap<String, String>>>,
    base_url: String,
}

impl UrlShortener {
    pub fn new(base_url: String) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            base_url,
        }
    }

    pub fn shorten(&self, original_url: &str) -> Result<String, &'static str> {
        if !URL_REGEX.is_match(original_url) {
            return Err("Invalid URL format");
        }

        let id = nanoid!(8);
        let short_url = format!("{}/{}", self.base_url, id);

        {
            let mut storage = self.storage.lock().unwrap();
            storage.insert(id.clone(), original_url.to_string());
        }

        Ok(short_url)
    }

    pub fn resolve(&self, short_id: &str) -> Option<String> {
        let storage = self.storage.lock().unwrap();
        storage.get(short_id).cloned()
    }

    pub fn get_stats(&self) -> usize {
        let storage = self.storage.lock().unwrap();
        storage.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_shortener() {
        let shortener = UrlShortener::new("https://short.ly".to_string());
        
        let url = "https://www.example.com/some/long/path";
        let short_url = shortener.shorten(url).unwrap();
        
        assert!(short_url.starts_with("https://short.ly/"));
        assert_eq!(shortener.get_stats(), 1);
        
        let id = short_url.split('/').last().unwrap();
        let resolved = shortener.resolve(id).unwrap();
        assert_eq!(resolved, url);
    }

    #[test]
    fn test_invalid_url() {
        let shortener = UrlShortener::new("https://short.ly".to_string());
        let result = shortener.shorten("not-a-valid-url");
        assert!(result.is_err());
    }
}
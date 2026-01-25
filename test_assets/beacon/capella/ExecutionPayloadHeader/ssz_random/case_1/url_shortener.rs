
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct UrlShortener {
    storage: Arc<RwLock<HashMap<String, String>>>,
    counter: Arc<RwLock<u64>>,
}

impl UrlShortener {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            counter: Arc::new(RwLock::new(0)),
        }
    }

    pub fn shorten(&self, url: &str) -> String {
        let mut counter = self.counter.write().unwrap();
        *counter += 1;
        let key = base62_encode(*counter);
        
        let mut storage = self.storage.write().unwrap();
        storage.insert(key.clone(), url.to_string());
        
        key
    }

    pub fn expand(&self, key: &str) -> Option<String> {
        let storage = self.storage.read().unwrap();
        storage.get(key).cloned()
    }

    pub fn count(&self) -> usize {
        let storage = self.storage.read().unwrap();
        storage.len()
    }
}

fn base62_encode(mut num: u64) -> String {
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut result = String::new();
    
    while num > 0 {
        result.push(CHARSET[(num % 62) as usize] as char);
        num /= 62;
    }
    
    if result.is_empty() {
        result.push('0');
    }
    
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_shortener() {
        let shortener = UrlShortener::new();
        
        let url1 = "https://www.example.com/very/long/path";
        let key1 = shortener.shorten(url1);
        
        let url2 = "https://www.rust-lang.org";
        let key2 = shortener.shorten(url2);
        
        assert_eq!(shortener.expand(&key1), Some(url1.to_string()));
        assert_eq!(shortener.expand(&key2), Some(url2.to_string()));
        assert_eq!(shortener.expand("invalid"), None);
        assert_eq!(shortener.count(), 2);
    }

    #[test]
    fn test_base62_encoding() {
        assert_eq!(base62_encode(0), "0");
        assert_eq!(base62_encode(1), "1");
        assert_eq!(base62_encode(61), "z");
        assert_eq!(base62_encode(62), "10");
        assert_eq!(base62_encode(3844), "100");
    }
}
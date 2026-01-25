
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

struct UrlShortener {
    store: Arc<Mutex<HashMap<String, String>>>,
    counter: Arc<Mutex<u64>>,
}

impl UrlShortener {
    fn new() -> Self {
        UrlShortener {
            store: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    fn encode_base62(&self, mut num: u64) -> String {
        if num == 0 {
            return String::from("0");
        }
        
        let mut result = Vec::new();
        while num > 0 {
            let remainder = (num % 62) as usize;
            result.push(BASE62_CHARS[remainder] as char);
            num /= 62;
        }
        result.iter().rev().collect()
    }

    fn shorten(&self, long_url: &str) -> String {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        let short_code = self.encode_base62(*counter);
        
        let mut store = self.store.lock().unwrap();
        store.insert(short_code.clone(), long_url.to_string());
        
        format!("https://short.url/{}", short_code)
    }

    fn expand(&self, short_url: &str) -> Option<String> {
        let code = short_url.trim_start_matches("https://short.url/");
        let store = self.store.lock().unwrap();
        store.get(code).cloned()
    }
}

fn main() {
    let shortener = UrlShortener::new();
    
    let urls = vec![
        "https://www.rust-lang.org/",
        "https://docs.rs/",
        "https://crates.io/",
    ];
    
    let mut shortened = Vec::new();
    for url in urls {
        let short = shortener.shorten(url);
        println!("Shortened: {} -> {}", url, short);
        shortened.push(short);
    }
    
    println!("\nExpanding shortened URLs:");
    for short_url in shortened {
        if let Some(original) = shortener.expand(&short_url) {
            println!("{} -> {}", short_url, original);
        }
    }
}
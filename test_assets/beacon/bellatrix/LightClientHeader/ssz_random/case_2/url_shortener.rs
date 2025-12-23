use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

struct UrlShortener {
    storage: HashMap<String, String>,
    counter: u64,
}

impl UrlShortener {
    fn new() -> Self {
        UrlShortener {
            storage: HashMap::new(),
            counter: 0,
        }
    }

    fn shorten(&mut self, long_url: &str) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        long_url.hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut num = self.counter.wrapping_add(hash);
        let mut short_code = String::new();
        
        while num > 0 {
            let remainder = (num % 62) as usize;
            short_code.push(BASE62[remainder] as char);
            num /= 62;
        }
        
        if short_code.is_empty() {
            short_code.push(BASE62[0] as char);
        }
        
        self.storage.insert(short_code.clone(), long_url.to_string());
        self.counter = self.counter.wrapping_add(1);
        
        format!("https://short.url/{}", short_code)
    }

    fn expand(&self, short_url: &str) -> Option<&String> {
        let code = short_url.trim_start_matches("https://short.url/");
        self.storage.get(code)
    }
}

fn main() {
    let mut shortener = UrlShortener::new();
    
    let urls = vec![
        "https://www.rust-lang.org/",
        "https://docs.rs/",
        "https://crates.io/",
    ];
    
    for url in urls {
        let short = shortener.shorten(url);
        println!("Original: {}", url);
        println!("Shortened: {}", short);
        
        if let Some(expanded) = shortener.expand(&short) {
            println!("Expanded: {}", expanded);
        }
        println!("---");
    }
}
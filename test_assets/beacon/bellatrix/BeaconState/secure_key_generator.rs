use rand::{thread_rng, Rng};
use std::collections::HashSet;

const DEFAULT_LENGTH: usize = 32;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

pub struct KeyGenerator {
    length: usize,
    character_set: String,
    exclude_similar: bool,
}

impl KeyGenerator {
    pub fn new() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            character_set: format!("{}{}{}{}", UPPERCASE, LOWERCASE, DIGITS, SYMBOLS),
            exclude_similar: false,
        }
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn exclude_similar(mut self, exclude: bool) -> Self {
        self.exclude_similar = exclude;
        self
    }

    pub fn include_uppercase(mut self, include: bool) -> Self {
        if !include {
            self.character_set = self.character_set.replace(UPPERCASE, "");
        } else if !self.character_set.contains(UPPERCASE) {
            self.character_set.push_str(UPPERCASE);
        }
        self
    }

    pub fn include_lowercase(mut self, include: bool) -> Self {
        if !include {
            self.character_set = self.character_set.replace(LOWERCASE, "");
        } else if !self.character_set.contains(LOWERCASE) {
            self.character_set.push_str(LOWERCASE);
        }
        self
    }

    pub fn include_digits(mut self, include: bool) -> Self {
        if !include {
            self.character_set = self.character_set.replace(DIGITS, "");
        } else if !self.character_set.contains(DIGITS) {
            self.character_set.push_str(DIGITS);
        }
        self
    }

    pub fn include_symbols(mut self, include: bool) -> Self {
        if !include {
            self.character_set = self.character_set.replace(SYMBOLS, "");
        } else if !self.character_set.contains(SYMBOLS) {
            self.character_set.push_str(SYMBOLS);
        }
        self
    }

    pub fn generate(&self) -> Result<String, String> {
        if self.character_set.is_empty() {
            return Err("Character set cannot be empty".to_string());
        }

        if self.length == 0 {
            return Err("Key length must be greater than zero".to_string());
        }

        let mut rng = thread_rng();
        let mut result = String::with_capacity(self.length);
        let chars: Vec<char> = if self.exclude_similar {
            self.filter_similar_chars()
        } else {
            self.character_set.chars().collect()
        };

        for _ in 0..self.length {
            let idx = rng.gen_range(0..chars.len());
            result.push(chars[idx]);
        }

        Ok(result)
    }

    fn filter_similar_chars(&self) -> Vec<char> {
        let similar_chars: HashSet<char> = ['I', 'l', '1', 'O', '0', 'S', '5']
            .iter()
            .cloned()
            .collect();
        
        self.character_set
            .chars()
            .filter(|c| !similar_chars.contains(c))
            .collect()
    }
}

impl Default for KeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generation() {
        let generator = KeyGenerator::new();
        let key = generator.generate().unwrap();
        assert_eq!(key.len(), DEFAULT_LENGTH);
    }

    #[test]
    fn test_custom_length() {
        let generator = KeyGenerator::new().length(64);
        let key = generator.generate().unwrap();
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn test_exclude_similar() {
        let generator = KeyGenerator::new()
            .exclude_similar(true)
            .length(100);
        let key = generator.generate().unwrap();
        assert!(!key.contains('I'));
        assert!(!key.contains('l'));
        assert!(!key.contains('1'));
        assert!(!key.contains('O'));
        assert!(!key.contains('0'));
    }

    #[test]
    fn test_character_set_validation() {
        let generator = KeyGenerator::new()
            .include_uppercase(false)
            .include_lowercase(false)
            .include_digits(false)
            .include_symbols(false);
        assert!(generator.generate().is_err());
    }
}
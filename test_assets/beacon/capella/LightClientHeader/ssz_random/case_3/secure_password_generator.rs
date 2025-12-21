
use rand::{rngs::OsRng, RngCore};
use std::fmt::Write;

const DEFAULT_LENGTH: usize = 16;
const CHARACTER_SETS: [&str; 4] = [
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    "!@#$%^&*()_+-=[]{}|;:,.<>?",
];

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl PasswordGenerator {
    pub fn new() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length.max(4);
        self
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn numbers(mut self, enable: bool) -> Self {
        self.use_numbers = enable;
        self
    }

    pub fn symbols(mut self, enable: bool) -> Self {
        self.use_symbols = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length < 4 {
            return Err("Password length must be at least 4 characters");
        }

        let mut character_pool = String::new();
        character_pool.push_str(CHARACTER_SETS[0]);

        if self.use_uppercase {
            character_pool.push_str(CHARACTER_SETS[1]);
        }
        if self.use_numbers {
            character_pool.push_str(CHARACTER_SETS[2]);
        }
        if self.use_symbols {
            character_pool.push_str(CHARACTER_SETS[3]);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut password = String::with_capacity(self.length);
        let mut rng = OsRng;

        for _ in 0..self.length {
            let random_index = (rng.next_u32() as usize) % character_pool.len();
            password.push(
                character_pool
                    .chars()
                    .nth(random_index)
                    .expect("Index should be valid"),
            );
        }

        Ok(password)
    }
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let generator = PasswordGenerator::new();
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), DEFAULT_LENGTH);
    }

    #[test]
    fn test_custom_length() {
        let generator = PasswordGenerator::new().length(20);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_character_sets() {
        let generator = PasswordGenerator::new()
            .uppercase(false)
            .numbers(false)
            .symbols(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_minimum_length() {
        let generator = PasswordGenerator::new().length(3);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 4);
    }
}
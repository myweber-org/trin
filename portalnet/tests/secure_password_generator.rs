
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub struct PasswordGenerator {
    length: usize,
    include_uppercase: bool,
    include_numbers: bool,
    include_symbols: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
            length,
            include_uppercase: true,
            include_numbers: true,
            include_symbols: true,
        }
    }

    pub fn uppercase(mut self, include: bool) -> Self {
        self.include_uppercase = include;
        self
    }

    pub fn numbers(mut self, include: bool) -> Self {
        self.include_numbers = include;
        self
    }

    pub fn symbols(mut self, include: bool) -> Self {
        self.include_symbols = include;
        self
    }

    pub fn generate(&self) -> String {
        let mut charset = String::from("abcdefghijklmnopqrstuvwxyz");
        
        if self.include_uppercase {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        
        if self.include_numbers {
            charset.push_str("0123456789");
        }
        
        if self.include_symbols {
            charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        let charset_bytes = charset.as_bytes();
        let mut rng = thread_rng();
        
        (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..charset_bytes.len());
                charset_bytes[idx] as char
            })
            .collect()
    }

    pub fn generate_alphanumeric(length: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
}

pub fn generate_strong_password() -> String {
    let generator = PasswordGenerator::new(16)
        .uppercase(true)
        .numbers(true)
        .symbols(true);
    
    generator.generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_alphanumeric_generator() {
        let password = PasswordGenerator::generate_alphanumeric(10);
        assert_eq!(password.len(), 10);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_custom_charset() {
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .numbers(false)
            .symbols(false);
        
        let password = generator.generate();
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }
}
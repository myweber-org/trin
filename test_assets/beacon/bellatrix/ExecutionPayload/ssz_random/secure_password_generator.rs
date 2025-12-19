use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;

#[derive(Debug)]
pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn set_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn set_character_sets(
        mut self,
        uppercase: bool,
        lowercase: bool,
        digits: bool,
        special: bool,
    ) -> Self {
        self.use_uppercase = uppercase;
        self.use_lowercase = lowercase;
        self.use_digits = digits;
        self.use_special = special;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        if !self.use_uppercase
            && !self.use_lowercase
            && !self.use_digits
            && !self.use_special
        {
            return Err("At least one character set must be enabled");
        }

        let mut character_pool = Vec::new();
        let mut rng = rand::thread_rng();

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        if self.use_special {
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx] as char
            })
            .collect();

        Ok(password)
    }
}

fn main() {
    println!("Secure Password Generator");
    println!("=========================");

    let generator = PasswordGenerator::new()
        .set_length(20)
        .set_character_sets(true, true, true, true);

    match generator.generate() {
        Ok(password) => {
            println!("Generated password: {}", password);
            println!("Password length: {}", password.len());
        }
        Err(e) => {
            eprintln!("Error generating password: {}", e);
        }
    }

    println!("\nCustom configuration example:");
    let custom_gen = PasswordGenerator::new()
        .set_length(12)
        .set_character_sets(true, true, true, false);

    match custom_gen.generate() {
        Ok(pwd) => println!("Custom password (no special chars): {}", pwd),
        Err(e) => eprintln!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let gen = PasswordGenerator::new();
        let result = gen.generate();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), DEFAULT_LENGTH);
    }

    #[test]
    fn test_custom_length() {
        let gen = PasswordGenerator::new().set_length(32);
        let result = gen.generate();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_no_character_sets() {
        let gen = PasswordGenerator::new().set_character_sets(false, false, false, false);
        let result = gen.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_length() {
        let gen = PasswordGenerator::new().set_length(0);
        let result = gen.generate();
        assert!(result.is_err());
    }
}
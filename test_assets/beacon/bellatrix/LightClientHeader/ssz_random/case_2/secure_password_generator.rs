use rand::Rng;

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than zero");
        }

        let mut character_pool = Vec::new();
        let lowercase_chars = "abcdefghijklmnopqrstuvwxyz";
        let uppercase_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let digit_chars = "0123456789";
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";

        if self.use_lowercase {
            character_pool.extend(lowercase_chars.chars());
        }
        if self.use_uppercase {
            character_pool.extend(uppercase_chars.chars());
        }
        if self.use_digits {
            character_pool.extend(digit_chars.chars());
        }
        if self.use_special {
            character_pool.extend(special_chars.chars());
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx]
            })
            .collect();

        Ok(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_length() {
        let generator = PasswordGenerator::new(20);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_only_digits() {
        let generator = PasswordGenerator::new(8)
            .lowercase(false)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new(0);
        let result = generator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        let result = generator.generate();
        assert!(result.is_err());
    }
}
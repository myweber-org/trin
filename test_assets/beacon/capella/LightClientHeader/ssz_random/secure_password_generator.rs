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

    pub fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn with_digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn with_special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        if !self.use_lowercase && !self.use_uppercase && !self.use_digits && !self.use_special {
            return Err("At least one character set must be enabled");
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
    fn test_basic_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .with_lowercase(true)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_invalid_length() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .with_lowercase(false)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        assert!(generator.generate().is_err());
    }
}use rand::Rng;
use std::collections::HashSet;

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
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
        let mut required_chars = HashSet::new();

        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_chars.insert(self.random_char_from_range(b'a'..=b'z'));
        }

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.insert(self.random_char_from_range(b'A'..=b'Z'));
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.insert(self.random_char_from_range(b'0'..=b'9'));
        }

        if self.use_special {
            let special_chars = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
            character_pool.extend_from_slice(special_chars);
            required_chars.insert(self.random_char_from_slice(special_chars));
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars: Vec<char> = required_chars.into_iter().collect();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[idx] as char);
        }

        // Shuffle to avoid predictable patterns
        for i in 0..password_chars.len() {
            let j = rng.gen_range(0..password_chars.len());
            password_chars.swap(i, j);
        }

        Ok(password_chars.into_iter().collect())
    }

    fn random_char_from_range<R: rand::distributions::uniform::SampleRange<u8>>(
        &self,
        range: R,
    ) -> char {
        let mut rng = rand::thread_rng();
        rng.gen_range(range) as char
    }

    fn random_char_from_slice(&self, slice: &[u8]) -> char {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..slice.len());
        slice[idx] as char
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
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
    fn test_character_set_disabling() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        
        // Should only contain lowercase and digits
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit()));
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        
        assert!(generator.generate().is_err());
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());
    }
}
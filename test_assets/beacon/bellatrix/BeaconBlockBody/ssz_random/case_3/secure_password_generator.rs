use rand::Rng;
use std::collections::HashSet;

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
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
            return Err("Password length must be greater than 0");
        }

        let mut character_pool = Vec::new();
        
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
            character_pool.extend(b'!'..=b'/');
            character_pool.extend(b':'..=b'@');
            character_pool.extend(b'['..=b'`');
            character_pool.extend(b'{'..=b'~');
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        while password.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            
            if used_chars.insert(ch) || password.len() >= character_pool.len().min(self.length) {
                password.push(ch);
            }
        }

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
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .lowercase(false)
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
use rand::Rng;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct PasswordConfig {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 16,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }
}

impl PasswordConfig {
    pub fn new(
        length: usize,
        use_uppercase: bool,
        use_lowercase: bool,
        use_digits: bool,
        use_special: bool,
    ) -> Result<Self, Box<dyn Error>> {
        if length < 8 {
            return Err("Password length must be at least 8 characters".into());
        }
        
        if !use_uppercase && !use_lowercase && !use_digits && !use_special {
            return Err("At least one character set must be enabled".into());
        }

        Ok(Self {
            length,
            use_uppercase,
            use_lowercase,
            use_digits,
            use_special,
        })
    }

    pub fn generate_password(&self) -> String {
        let mut rng = rand::thread_rng();
        let mut character_pool = Vec::new();
        
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

        password
    }

    pub fn validate_strength(&self, password: &str) -> bool {
        if password.len() < self.length {
            return false;
        }

        let mut has_upper = !self.use_uppercase;
        let mut has_lower = !self.use_lowercase;
        let mut has_digit = !self.use_digits;
        let mut has_special = !self.use_special;

        for ch in password.chars() {
            if ch.is_ascii_uppercase() {
                has_upper = true;
            } else if ch.is_ascii_lowercase() {
                has_lower = true;
            } else if ch.is_ascii_digit() {
                has_digit = true;
            } else if ch.is_ascii_punctuation() {
                has_special = true;
            }
        }

        has_upper && has_lower && has_digit && has_special
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PasswordConfig::default();
        let password = config.generate_password();
        assert_eq!(password.len(), 16);
        assert!(config.validate_strength(&password));
    }

    #[test]
    fn test_custom_config() {
        let config = PasswordConfig::new(12, true, true, true, false).unwrap();
        let password = config.generate_password();
        assert_eq!(password.len(), 12);
        assert!(config.validate_strength(&password));
    }

    #[test]
    fn test_invalid_length() {
        let result = PasswordConfig::new(6, true, true, true, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let result = PasswordConfig::new(10, false, false, false, false);
        assert!(result.is_err());
    }
}
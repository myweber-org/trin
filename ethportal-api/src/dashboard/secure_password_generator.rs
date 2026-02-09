
use rand::Rng;
use std::collections::HashSet;

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
    exclude_chars: HashSet<char>,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_special: true,
            exclude_chars: HashSet::new(),
        }
    }

    pub fn exclude_chars(mut self, chars: &[char]) -> Self {
        self.exclude_chars.extend(chars);
        self
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

    pub fn generate(&self) -> Option<String> {
        let mut character_pool = Vec::new();

        if self.use_lowercase {
            character_pool.extend(('a'..='z').filter(|c| !self.exclude_chars.contains(c)));
        }

        if self.use_uppercase {
            character_pool.extend(('A'..='Z').filter(|c| !self.exclude_chars.contains(c)));
        }

        if self.use_digits {
            character_pool.extend(('0'..='9').filter(|c| !self.exclude_chars.contains(c)));
        }

        if self.use_special {
            let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            character_pool.extend(special_chars.chars().filter(|c| !self.exclude_chars.contains(c)));
        }

        if character_pool.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx]
            })
            .collect();

        Some(password)
    }

    pub fn validate(&self, password: &str) -> bool {
        if password.len() != self.length {
            return false;
        }

        let mut has_lowercase = false;
        let mut has_uppercase = false;
        let mut has_digit = false;
        let mut has_special = false;

        for ch in password.chars() {
            if self.exclude_chars.contains(&ch) {
                return false;
            }

            if ch.is_ascii_lowercase() {
                has_lowercase = true;
            } else if ch.is_ascii_uppercase() {
                has_uppercase = true;
            } else if ch.is_ascii_digit() {
                has_digit = true;
            } else {
                has_special = true;
            }
        }

        (!self.use_lowercase || has_lowercase) &&
        (!self.use_uppercase || has_uppercase) &&
        (!self.use_digits || has_digit) &&
        (!self.use_special || has_special)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate();
        assert!(password.is_some());
        assert_eq!(password.unwrap().len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .with_special(false)
            .with_digits(false);

        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_alphabetic()));
    }

    #[test]
    fn test_exclude_characters() {
        let generator = PasswordGenerator::new(10)
            .exclude_chars(&['a', 'A', '1', '!']);

        let password = generator.generate().unwrap();
        assert!(!password.contains('a'));
        assert!(!password.contains('A'));
        assert!(!password.contains('1'));
        assert!(!password.contains('!'));
    }

    #[test]
    fn test_password_validation() {
        let generator = PasswordGenerator::new(10)
            .with_special(true)
            .with_digits(true);

        let password = generator.generate().unwrap();
        assert!(generator.validate(&password));
    }
}
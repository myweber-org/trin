use rand::Rng;
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
        let mut required_sets = Vec::new();

        if self.use_lowercase {
            let set: Vec<char> = ('a'..='z').collect();
            required_sets.push(set.clone());
            character_pool.extend(set);
        }

        if self.use_uppercase {
            let set: Vec<char> = ('A'..='Z').collect();
            required_sets.push(set.clone());
            character_pool.extend(set);
        }

        if self.use_digits {
            let set: Vec<char> = ('0'..='9').collect();
            required_sets.push(set.clone());
            character_pool.extend(set);
        }

        if self.use_special {
            let set: Vec<char> = "!@#$%^&*()-_=+[]{}|;:,.<>?".chars().collect();
            required_sets.push(set.clone());
            character_pool.extend(set);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_sets = HashSet::new();

        for set in &required_sets {
            let random_char = set[rng.gen_range(0..set.len())];
            password_chars.push(random_char);
            used_sets.insert(set.as_ptr());
        }

        while password_chars.len() < self.length {
            let random_index = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[random_index]);
        }

        for i in (1..password_chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            password_chars.swap(i, j);
        }

        Ok(password_chars.into_iter().collect())
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
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(!password.chars().any(|c| c.is_uppercase()));
        assert!(!password.chars().any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c)));
    }

    #[test]
    fn test_invalid_length() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());
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
}